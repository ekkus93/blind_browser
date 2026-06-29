# Blind Browser Security and Silent-Failure Hardening Spec

**Target repository:** `blind_browser-master_2606290437(1).zip`  
**Suggested destination:** `docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING_SPEC.md`  
**Companion TODO:** `docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING_TODO.md`  
**Primary reviewer intent:** make the browser/planner/runtime boundary fail closed, make runtime state truthful, and remove false contracts where options are accepted but ignored.

## 1. Problem statement

The current app has a strong planner/confirmation core, but several surrounding systems still have high-risk gaps:

1. Tauri CSP is disabled.
2. Internal browser navigation accepts any syntactically valid absolute scheme instead of an explicit web URL allowlist.
3. Browser navigation APIs accept `LoadState` and `timeout_ms`, but some implementations discard them with `let _ = ...`.
4. Browser visibility switching tears down the current session before proving the replacement session can launch and recover the prior URL.
5. Page snapshot collection silently returns `None` when page metrics fail.
6. Remote ASR timeout returns to the caller while the spawned request thread continues running.
7. Local ASR silently drops segments that fail conversion.
8. Bundled skill parsing defaults malformed `requires_confirmation` metadata to `false` and skips invalid bundled skills with only a warning.
9. Frontend voice UI sometimes sets `isListening: false` after backend stop/transcribe failure even though runtime state may be unknown.
10. Runtime refresh clears several panel errors without distinguishing runtime-state errors from user-action errors.

This spec turns those review findings into explicit implementation requirements for Claude Code.

## 2. Non-goals

This hardening pass must not redesign the planner, replace Chromium, redesign the React UI, add a new accessibility workflow, or introduce a large new dependency unless a smaller local fix is clearly worse.

Do not add silent compatibility fallbacks. If behavior is unsupported, surface a structured error and add a test.

## 3. Required invariants

### 3.1 No false contracts

If a public command, planner tool schema, config field, or Rust function accepts a parameter, that parameter must either:

- be implemented;
- be rejected as unsupported with a structured error; or
- be removed from the public contract and all callers.

No production implementation may use `let _ = parameter;` for a semantically meaningful input such as `timeout_ms`, `load_state`, confirmation policy, URL policy, ASR segment conversion, or runtime state.

### 3.2 Browser navigation is web-only by default

Planner-driven internal navigation must only allow:

- `http://...`
- `https://...`

Reject these explicitly:

- `file://...`
- `javascript:...`
- `data:...`
- `chrome://...`
- `about:blank` unless there is a very narrow, documented internal-only call site that cannot be reached by planner/user URL input.
- scheme-relative URLs like `//example.com`
- `http:example.com` / `https:example.com` without `//`
- empty hosts such as `https:///path`

External link opening must remain stricter than internal navigation: keep `open_external_url` HTTPS-only unless there is a separate reviewed requirement to allow HTTP.

### 3.3 Confirmation policy fails closed

Malformed bundled skill confirmation metadata is a build/runtime defect, not a best-effort parse condition. Any malformed `requires_confirmation` value in bundled skills must fail skill loading loudly.

Invalid bundled skills must not be skipped in shipped builds. They should fail tests and fail application startup or planner initialization with an actionable error.

### 3.4 Runtime state must stay truthful

The UI must not claim that the app is no longer listening unless the backend returned a listening state confirming that. If a backend operation fails and the frontend cannot determine the true listening state, the UI must expose an explicit “runtime state unknown” / “failed to refresh runtime state” error rather than inventing a safe-looking state.

### 3.5 Browser session replacement must be atomic from the user perspective

Changing browser visibility must not destroy the active working browser session until the replacement session has successfully launched and recovered the prior URL, or until the code has made an explicit decision that the prior URL cannot be preserved and has surfaced that fact.

### 3.6 Degraded snapshots must be explicit

If page metrics fail during snapshot generation, the caller must receive either:

- a snapshot with an explicit warning/diagnostic field; or
- a structured error explaining that snapshot collection failed.

It must not silently return `None` in a way that is indistinguishable from “there is no current page.”

## 4. Architecture changes

## 4.1 Add a shared URL policy module

Create a shared Rust module so planner validation and runtime execution cannot drift.

Suggested file:

```text
src-tauri/src/url_policy.rs
```

Expose at least:

```rust
pub fn normalize_browser_navigation_url(raw: &str) -> Result<String, UrlPolicyError>;
pub fn is_allowed_browser_navigation_scheme(scheme: &str) -> bool;
```

The validator path and runtime path must both call `normalize_browser_navigation_url` or derive from the same helper.

Add to `src-tauri/src/lib.rs`:

```rust
pub mod url_policy;
```

Suggested implementation:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlPolicyError {
    Empty,
    MissingScheme { url: String },
    InvalidScheme { url: String, scheme: String },
    UnsupportedScheme { url: String, scheme: String },
    MissingAuthority { url: String, scheme: String },
}

impl UrlPolicyError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Empty => "empty_url",
            Self::MissingScheme { .. } => "missing_scheme",
            Self::InvalidScheme { .. } => "invalid_scheme",
            Self::UnsupportedScheme { .. } => "unsupported_scheme",
            Self::MissingAuthority { .. } => "missing_authority",
        }
    }

    pub fn user_message(&self) -> &'static str {
        match self {
            Self::Empty => "open_url requires a non-empty URL",
            Self::MissingScheme { .. } => "open_url requires an absolute http or https URL",
            Self::InvalidScheme { .. } => "open_url requires a URL with a valid scheme",
            Self::UnsupportedScheme { .. } => "open_url only supports http and https URLs",
            Self::MissingAuthority { .. } => "open_url requires http/https URLs to include // and a host",
        }
    }

    pub fn details(&self) -> serde_json::Value {
        match self {
            Self::Empty => serde_json::json!({}),
            Self::MissingScheme { url } => serde_json::json!({ "url": url }),
            Self::InvalidScheme { url, scheme }
            | Self::UnsupportedScheme { url, scheme }
            | Self::MissingAuthority { url, scheme } => {
                serde_json::json!({ "url": url, "scheme": scheme })
            }
        }
    }
}

pub fn is_allowed_browser_navigation_scheme(scheme: &str) -> bool {
    matches!(scheme.to_ascii_lowercase().as_str(), "http" | "https")
}

pub fn normalize_browser_navigation_url(raw: &str) -> Result<String, UrlPolicyError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(UrlPolicyError::Empty);
    }

    let Some(separator_index) = trimmed.find(':') else {
        return Err(UrlPolicyError::MissingScheme {
            url: trimmed.to_string(),
        });
    };

    let scheme = trimmed[..separator_index].to_ascii_lowercase();
    let valid_scheme = scheme.chars().enumerate().all(|(index, ch)| match index {
        0 => ch.is_ascii_alphabetic(),
        _ => ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'),
    });
    if !valid_scheme {
        return Err(UrlPolicyError::InvalidScheme {
            url: trimmed.to_string(),
            scheme,
        });
    }

    if !is_allowed_browser_navigation_scheme(&scheme) {
        return Err(UrlPolicyError::UnsupportedScheme {
            url: trimmed.to_string(),
            scheme,
        });
    }

    let after_scheme = &trimmed[separator_index + 1..];
    if !after_scheme.starts_with("//") {
        return Err(UrlPolicyError::MissingAuthority {
            url: trimmed.to_string(),
            scheme,
        });
    }

    let authority = after_scheme[2..]
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default()
        .trim();
    if authority.is_empty() {
        return Err(UrlPolicyError::MissingAuthority {
            url: trimmed.to_string(),
            scheme,
        });
    }

    Ok(trimmed.to_string())
}
```

Use this in `src-tauri/src/app_core/navigation_tools.rs` instead of the current permissive `normalize_absolute_url` logic.

Use the same helper in `src-tauri/src/commands/validators/navigation.rs` instead of duplicating scheme syntax checks.

## 4.2 Restore a restrictive Tauri CSP

Replace this:

```json
"security": {
  "csp": null
}
```

with a restrictive policy. Start here and adjust only when tests/manual app launch prove a specific directive needs expansion:

```json
"security": {
  "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self' data:; connect-src 'self' http://127.0.0.1:* https://api.openai.com https:; object-src 'none'; base-uri 'self'; frame-ancestors 'none'"
}
```

Acceptance rule: CSP may not be set to `null` again. If development requires a different dev policy, document it and keep production restrictive.

## 4.3 Implement or remove navigation load/timeout behavior

Current risky pattern:

```rust
let _ = load_state;
let _ = timeout_ms;
```

This exists around `BrowserController::open_url`, `reload_page`, and history navigation.

Required behavior:

1. `timeout_ms` must bound the navigation and waiting operation.
2. `LoadState::Load` must at least wait for the browser navigation completion signal already used elsewhere.
3. `LoadState::DomContentLoaded` and `LoadState::NetworkIdle` must either be implemented using Chromium lifecycle/ready-state checks or rejected with a structured `BrowserError`/`ToolError` stating that the requested wait state is unsupported.
4. Snapshot extraction must happen only after the chosen wait policy completes or intentionally times out.

Minimum acceptable v1 implementation:

- Apply timeout to `page.goto`, `page.reload`, `page.wait_for_navigation`, and history navigation waits where the Chromium API returns futures.
- Treat `LoadState::DomContentLoaded` and `LoadState::Load` as supported by the best available navigation wait.
- For `LoadState::NetworkIdle`, either implement a real network-idle wait or return an explicit unsupported error; do not degrade it silently to `Load`.

Preferred helper shape:

```rust
#[cfg(feature = "browser")]
async fn wait_for_navigation_policy<F, T>(
    operation: F,
    timeout_ms: Option<u64>,
    operation_name: &'static str,
) -> Result<T, BrowserError>
where
    F: std::future::Future<Output = Result<T, BrowserError>>,
{
    let timeout = std::time::Duration::from_millis(timeout_ms.unwrap_or(30_000).clamp(1, 120_000));
    match tauri::async_runtime::timeout(timeout, operation).await {
        Ok(result) => result,
        Err(_) => Err(BrowserError::Navigate(format!(
            "{operation_name} timed out after {}ms",
            timeout.as_millis()
        ))),
    }
}
```

If `tauri::async_runtime::timeout` is unavailable in this project’s exact dependency graph, add `tokio = { version = "1", features = ["time"] }` and use `tokio::time::timeout`, or use the timeout primitive exposed by the current Tauri runtime. Do not fake a timeout with `std::thread::sleep`.

## 4.4 Make visibility switching two-phase

Current flow drops `self.session` before confirming the replacement works. Replace it with a two-phase flow:

1. Read prior URL.
2. Create a candidate session using the new visibility config.
3. If there was a prior URL, navigate candidate session to it.
4. Only after candidate success, assign `self.session = Some(candidate_session)` and commit `self.config.visibility = mode`.
5. If candidate setup fails, keep the old session and return an error.

Suggested structural pattern:

```rust
#[cfg(feature = "browser")]
pub fn switch_visibility(&mut self, mode: BrowserVisibilityMode) -> Result<Option<String>, BrowserError> {
    let prior_url = self.current_non_blank_url()?;

    let mut candidate_config = self.config.clone();
    candidate_config.visibility = mode;
    let mut candidate_session = LiveBrowserSession::launch(&candidate_config)?;

    if let Some(ref url) = prior_url {
        let user_agent = candidate_config.user_agent.clone();
        let page = candidate_session.ensure_page(user_agent.as_deref())?;
        tauri::async_runtime::block_on(async {
            page.goto(url)
                .await
                .map_err(|error| BrowserError::Navigate(error.to_string()))
        })?;
    }

    self.config = candidate_config;
    self.session = Some(candidate_session);
    Ok(prior_url)
}
```

Add a helper for URL extraction. Do not use `.ok().flatten()` for a state read whose failure changes behavior. Either return the read error or explicitly log and surface a warning.

## 4.5 Make page snapshot failures explicit

Current behavior:

```rust
let BrowserPageMetrics { ... } = self.browser.get_page_metrics().ok()?;
```

This hides browser metric failures by returning `None`.

Preferred shape:

```rust
pub(super) fn current_page_snapshot(
    &mut self,
    text_excerpt_max_chars: Option<usize>,
    include_interactive_elements: bool,
) -> Result<Option<PageSnapshotData>, ToolError> {
    let Some(page_id) = self.state.current_page_id.clone() else {
        return Ok(None);
    };
    let Some(current_page) = self.state.current_page.as_ref() else {
        return Ok(None);
    };
    let Some(url) = current_page.url.clone() else {
        return Ok(None);
    };

    let title = current_page.title.clone();
    let visible_text_excerpt = build_visible_text_excerpt(current_page, text_excerpt_max_chars);
    let interactive_elements = if include_interactive_elements {
        current_page.interactive_elements.clone()
    } else {
        Vec::new()
    };

    let metrics = self.browser.get_page_metrics().map_err(|error| ToolError {
        code: String::from("browser_metrics_failed"),
        message: String::from("failed to read browser page metrics for current page snapshot"),
        retryable: true,
        details: Some(serde_json::json!({ "reason": error.to_string() })),
    })?;

    Ok(Some(PageSnapshotData {
        page_id,
        url,
        title,
        visible_text_excerpt,
        interactive_elements,
        scroll_y: metrics.scroll_y,
        viewport_width: metrics.viewport_width,
        viewport_height: metrics.viewport_height,
        document_height: metrics.document_height,
    }))
}
```

Then update callers such as command dispatch to propagate or include the structured failure instead of collapsing it to no snapshot.

## 4.6 Fix remote ASR timeout semantics

Current behavior spawns a thread, waits on `recv_timeout`, then leaves the underlying request running after timeout.

Required behavior:

- Remote ASR request timeout must cancel/bound the actual HTTP request.
- Do not spawn unbounded request threads for each transcription unless there is an explicit bounded worker pool.
- Ignore-send patterns like `let _ = sender.send(result);` must not hide important cancellation/resource behavior.

Preferred implementation options:

1. Use `async_openai` with an HTTP client configured with request timeout if the library supports it.
2. Replace the remote ASR path with a direct `reqwest::blocking::Client::builder().timeout(...).build()` multipart request, matching the TTS remote style.
3. Move ASR remote into an async task and use a real timeout around the future so timeout cancels the future.

Do not keep the current pattern as-is.

## 4.7 Do not silently drop local ASR segments

Replace this:

```rust
.filter_map(|segment| segment.to_str_lossy().ok())
```

with explicit error handling:

```rust
let mut segments = Vec::new();
for segment in state.as_iter() {
    let text = segment.to_str_lossy().map_err(|error| AsrRuntimeError::TranscriptionFailed {
        reason: format!("failed to decode whisper transcript segment: {error}"),
    })?;
    let text = text.trim();
    if !text.is_empty() {
        segments.push(text.to_string());
    }
}
let transcript = segments.join(" ");
```

If the exact error type does not implement `Display`, use `format!("{error:?}")`.

## 4.8 Fail loudly on bundled skill parse defects

Change `parse_bundled_skills` so it returns `Result<Vec<LoadedSkill>, String>` or a project-specific error type. Then propagate that error through skill loading and planner initialization.

For project/user skills, best-effort skip may be acceptable if the UI surfaces the file and error. For bundled skills, skip is not acceptable.

Bad current behavior:

```rust
requires_confirmation_value = parse_bool_value(value).unwrap_or(false);
```

Required behavior:

```rust
requires_confirmation_value = parse_bool_value(value).ok_or_else(|| {
    format!("invalid requires_confirmation value for bundled skill {current_name:?}: {value}")
})?;
```

This will require the parser loop and flush helper to become fallible.

## 4.9 Keep voice UI synchronized with backend state

Frontend catch blocks must not assert `isListening: false` unless a backend response confirmed it.

Replace stop/transcribe failure handling with:

1. Set `isBusy: false` and a clear error message.
2. Attempt `refreshRuntimePanels()`.
3. If refresh succeeds, let refreshed runtime state update `isListening`.
4. If refresh fails, keep previous `isListening` or set a dedicated `runtimeStateUnknown`/`lastError` message. Do not invent `false`.

A small helper is acceptable:

```ts
async function reportPushToTalkFailureWithoutInventingListeningState(message: string) {
  const previous = getPushToTalkState();
  setPushToTalkState({
    isBusy: false,
    isListening: previous.isListening,
    lastError: `${message} Runtime listening state could not be confirmed yet.`,
  });

  try {
    await refreshRuntimePanels();
  } catch (refreshError: unknown) {
    setPushToTalkState({
      isBusy: false,
      isListening: previous.isListening,
      lastError: `${message} Also failed to refresh runtime state: ${String(refreshError)}`,
    });
  }
}
```

Use a better app-local error formatter if available; avoid raw `String(refreshError)` if the project already has typed error formatting.

## 4.10 Stale confirmation submissions should not disappear silently

Current behavior returns silently when confirmation ID mismatches. Keep the stale-click guard, but surface a stateful UI error or at least a console warning.

Preferred UX:

- If confirmation state is no longer awaiting, ignore duplicate click but log debug-level detail.
- If ID mismatches while another confirmation is awaiting, set a confirmation error: “That confirmation is no longer active. Review the current confirmation before approving.”

## 4.11 Runtime refresh error ownership

Audit `src/runtime-refresh.ts` so it only clears errors it owns.

Policy:

- Runtime state refresh may clear stale runtime-status errors after a successful refresh.
- Runtime state refresh must not clear user-action errors such as failed save, failed provider test, failed ASR/TTS API key update, failed URL entry, or confirmation submission failure unless the associated action is retried, edited, dismissed, or replaced by a successful result for the same action.

A minimal implementation can split errors into separate fields:

```ts
runtimeRefreshError: string | null
lastActionError: string | null
```

Do not clear `lastActionError` inside generic refresh.

## 4.12 Default confirmation policy

Current config defaults ordinary clicks to no confirmation. For a blind, planner-controlled browser, the default should be conservative.

Change `config.example.toml`:

```toml
[safety]
confirmation_confidence_threshold = 0.90
allow_click_without_confirmation = false
always_confirm_submit = true
```

If this is too disruptive, introduce a clearly named onboarding setting, but do not silently ship a riskier default.

## 5. Testing requirements

Add or update tests in these areas:

### 5.1 Rust URL policy tests

Add tests for:

- accepts trimmed `https://example.com/page`
- accepts `http://localhost:3000`
- rejects blank URL
- rejects relative path
- rejects `about:blank`
- rejects `file:///etc/passwd`
- rejects `javascript:alert(1)`
- rejects `data:text/html,<h1>x</h1>`
- rejects `chrome://version`
- rejects `https:///missing-host`
- rejects `http:example.com`
- rejects scheme-relative `//example.com`
- rejects mixed unsupported scheme casing like `JaVaScRiPt:alert(1)`

### 5.2 Planner validator tests

Add validation tests proving `OpenUrl` planner steps reject non-http(s) URLs before execution.

### 5.3 Runtime execution tests

Add unit tests for `normalize_absolute_url` or replace that function with `normalize_browser_navigation_url` and update existing tests. The existing test currently accepts `about:blank`; change that expected behavior unless `about:blank` is made internal-only outside planner/user input.

### 5.4 Skill parser tests

Add tests proving:

- bundled skill parse rejects malformed `requires_confirmation`.
- bundled skill parse rejects unknown tool names.
- bundled skill parse does not silently skip invalid bundled entries.

### 5.5 Frontend voice-state tests

Add tests proving catch blocks do not force `isListening: false` when backend stop/transcribe fails. Test that the previous listening state is preserved until refresh gives authoritative state.

### 5.6 Runtime refresh tests

Add tests proving generic runtime refresh does not clear action-scoped errors.

### 5.7 Navigation wait/timeout tests

If direct Chromium integration tests are difficult, add at least unit-level tests around the timeout helper or a seam/mocked browser operation. The key assertion: timeout results in a structured error, not an ignored parameter.

## 6. Acceptance criteria

This hardening pass is complete when all of the following are true:

1. `src-tauri/tauri.conf.json` has non-null CSP.
2. Planner and runtime internal navigation reject every non-http(s) scheme listed above.
3. No semantically meaningful production parameter in the touched areas is discarded with `let _ = ...`.
4. `LoadState`/`timeout_ms` are implemented or unsupported states are explicitly rejected.
5. Visibility switching is two-phase and preserves the old session on candidate failure.
6. Page snapshot metric failures produce structured errors or explicit degraded state, not silent `None`.
7. Remote ASR timeout bounds/cancels the actual HTTP request.
8. Local ASR segment conversion errors fail transcription instead of silently dropping text.
9. Bundled skill parse defects fail loudly.
10. Frontend voice catch paths do not invent `isListening: false`.
11. Stale confirmation submissions are visible/debuggable.
12. Runtime refresh no longer clears unrelated user-action errors.
13. `config.example.toml` defaults `allow_click_without_confirmation = false`, or a documented alternative conservative default is implemented.
14. Rust and frontend tests cover the new safety behavior.
15. `pnpm test`, `pnpm build`, and `cargo test` pass in the normal default-feature build.

## 7. Suggested validation commands

Use the project’s existing tooling if available. At minimum:

```bash
pnpm install
pnpm test
pnpm build
cd src-tauri
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

If local system dependencies make full `cargo test` impossible, document exactly which dependency blocked it and run the maximum subset possible. Do not mark a task complete without stating which validation was run.
