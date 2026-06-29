# Blind Browser Security and Silent-Failure Hardening TODO

**Target repository:** `blind_browser-master_2606290437(1).zip`  
**Suggested destination:** `docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING_TODO.md`  
**Companion spec:** `docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING_SPEC.md`

## Priority legend

- **P0:** security/safety invariant; fix before new features.
- **P1:** correctness or silent-failure risk likely to cause bad behavior.
- **P2:** UX/state clarity or conservative hardening.
- **P3:** cleanup/refactor after behavior is safe.

## P0-1 — Restore a restrictive Tauri CSP

**Files:**

- `src-tauri/tauri.conf.json`

### Tasks

- [x] Replace `"csp": null` with a restrictive CSP.
- [x] Confirm app dev launch and production build still work. (`pnpm build` verified in the P0 gate; dev launch needs a human with a display.)
- [x] Do not disable CSP again to fix a build/runtime issue. If a directive needs expansion, document why.

### Suggested patch

```json
"security": {
  "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self' data:; connect-src 'self' http://127.0.0.1:* https://api.openai.com https:; object-src 'none'; base-uri 'self'; frame-ancestors 'none'"
}
```

### Acceptance checks

- [ ] `grep -R '"csp": null' src-tauri/tauri.conf.json` returns nothing.
- [ ] `pnpm build` passes.
- [ ] Tauri app starts in dev mode.

---

## P0-2 — Add a shared web-only URL policy

**Files:**

- Add `src-tauri/src/url_policy.rs`
- Update `src-tauri/src/lib.rs`
- Update `src-tauri/src/app_core/navigation_tools.rs`
- Update `src-tauri/src/commands/validators/navigation.rs`
- Update `src-tauri/src/app_core/tests/browser_tests.rs`
- Add/update command validator tests under `src-tauri/src/commands/tests/planner_flow/input_validation.rs`

### Why

Internal browser navigation currently accepts any syntactically valid absolute scheme. Planner-driven `OpenUrl` must fail closed to `http` and `https` only.

### Tasks

- [x] Create `src-tauri/src/url_policy.rs`.
- [x] Export it from `src-tauri/src/lib.rs` with `pub mod url_policy;`.
- [x] Replace permissive URL validation in `normalize_absolute_url` with the shared policy.
- [x] Replace permissive planner validation in `validate_open_url_input` with the same shared policy.
- [x] Change tests that currently accept `about:blank`; planner/user navigation should reject it.
- [x] Add explicit tests for dangerous schemes.

### Drop-in starting point: `src-tauri/src/url_policy.rs`

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_http_and_https_urls_with_hosts() {
        assert_eq!(
            normalize_browser_navigation_url("  https://example.com/path  ").unwrap(),
            "https://example.com/path"
        );
        assert_eq!(
            normalize_browser_navigation_url("http://localhost:3000").unwrap(),
            "http://localhost:3000"
        );
    }

    #[test]
    fn rejects_non_web_and_malformed_urls() {
        for raw in [
            "",
            "   ",
            "/relative/path",
            "//example.com",
            "about:blank",
            "file:///etc/passwd",
            "javascript:alert(1)",
            "JaVaScRiPt:alert(1)",
            "data:text/html,<h1>x</h1>",
            "chrome://version",
            "http:example.com",
            "https:///missing-host",
        ] {
            assert!(
                normalize_browser_navigation_url(raw).is_err(),
                "expected URL to be rejected: {raw}"
            );
        }
    }
}
```

### Patch shape for `src-tauri/src/lib.rs`

```rust
pub mod url_policy;
```

### Patch shape for `src-tauri/src/app_core/navigation_tools.rs`

Replace the current body of `normalize_absolute_url` with:

```rust
pub(crate) fn normalize_absolute_url(url: &str) -> Result<String, ToolError> {
    crate::url_policy::normalize_browser_navigation_url(url).map_err(|error| ToolError {
        code: String::from("invalid_url"),
        message: String::from(error.user_message()),
        retryable: false,
        details: Some(error.details()),
    })
}
```

Keep the function name if many call sites/tests already use it. The behavior is now “normalize allowed browser navigation URL,” not “any absolute URL.”

### Patch shape for `src-tauri/src/commands/validators/navigation.rs`

Replace the current scheme-only validation in `validate_open_url_input` with:

```rust
pub(super) fn validate_open_url_input(input: &OpenUrlInput) -> Result<(), ToolError> {
    crate::url_policy::normalize_browser_navigation_url(&input.url).map_err(|error| {
        invalid_planner_output(error.user_message(), Some(error.details()))
    })?;
    Ok(())
}
```

### Update existing test

In `src-tauri/src/app_core/tests/browser_tests.rs`, change the test that currently accepts `about:blank`.

Suggested replacement:

```rust
#[test]
fn normalize_absolute_url_accepts_trimmed_web_urls() {
    assert_eq!(
        normalize_absolute_url("  https://example.com/page  ").unwrap(),
        String::from("https://example.com/page")
    );
    assert_eq!(
        normalize_absolute_url("http://localhost:3000").unwrap(),
        String::from("http://localhost:3000")
    );
}

#[test]
fn normalize_absolute_url_rejects_non_web_schemes() {
    for raw in [
        "about:blank",
        "file:///etc/passwd",
        "javascript:alert(1)",
        "data:text/html,<h1>x</h1>",
        "chrome://version",
    ] {
        let error = normalize_absolute_url(raw).unwrap_err();
        assert_eq!(error.code, "invalid_url");
    }
}
```

### Add planner validation test

In `src-tauri/src/commands/tests/planner_flow/input_validation.rs`:

```rust
#[test]
fn validate_planner_output_rejects_open_url_with_non_web_scheme() {
    let available_tools = planner_available_tools();

    for url in [
        "about:blank",
        "file:///etc/passwd",
        "javascript:alert(1)",
        "data:text/html,<h1>x</h1>",
        "chrome://version",
        "http:example.com",
        "https:///missing-host",
    ] {
        let planner_output = PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::OpenUrl,
                goal: String::from("open a page"),
                target_description: None,
            },
            selected_skills: vec![String::from("open_url_direct")],
            steps: vec![PlannedStep {
                step_id: String::from("step-open-url"),
                tool_name: ToolName::OpenUrl,
                arguments: serde_json::json!({
                    "request_id": "req-open-url",
                    "url": url,
                    "wait_for_load_state": "Load"
                }),
                purpose: String::from("open a page"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        };

        let error = validate_planner_output(
            &planner_output,
            &available_tools,
            &[String::from("open_url_direct")],
        )
        .expect_err("validation should reject non-web open_url values");
        assert!(
            error.message.contains("http") || error.message.contains("scheme") || error.message.contains("host"),
            "unexpected error for {url}: {}",
            error.message
        );
    }
}
```

### Acceptance checks

- [x] `cargo test url_policy` passes.
- [x] `cargo test validate_planner_output_rejects_open_url` passes.
- [x] Internal `OpenUrl` rejects `file:`, `javascript:`, `data:`, `chrome:`, and `about:` before browser execution (validator + runtime `normalize_absolute_url`, defense in depth).

---

## P0-3 — Implement or explicitly reject navigation load-state/timeout behavior

**Files:**

- `src-tauri/src/browser/mod.rs`
- `src-tauri/src/browser/navigation.rs`
- Possibly `src-tauri/src/browser/config.rs`
- Tests under `src-tauri/src/browser` or app-core seams if available

### Why

`open_url`, `reload_page`, and history navigation accept `LoadState` and `timeout_ms`, but current browser implementations discard them. This is a false contract and can cause stale page models after slow navigation.

### Tasks

- [x] Search for `let _ = load_state;` and `let _ = timeout_ms;` in production code.
- [x] For browser navigation, implement timeout and wait behavior or reject unsupported states explicitly.
- [x] Add tests around timeout helper/seam.
- [x] Do not leave ignored semantic parameters in production code. (Navigation: done — `load_state` discards eliminated entirely; `timeout_ms` honored. Remaining `let _ = timeout_ms;` are `#[cfg(not(feature = "browser"))]` stubs or non-navigation handlers (e.g. set-volume) where it is genuinely unused — pre-existing, out of P0-3 scope.)

### Search command

```bash
grep -R "let _ = load_state\|let _ = timeout_ms" -n src-tauri/src
```

### Suggested helper

Use this as a starting point. Adjust the timeout primitive to whatever is available through the current Tauri runtime.

```rust
#[cfg(feature = "browser")]
async fn with_browser_timeout<T, F>(
    operation: F,
    timeout_ms: Option<u64>,
    operation_name: &'static str,
) -> Result<T, BrowserError>
where
    F: std::future::Future<Output = Result<T, BrowserError>>,
{
    let timeout_ms = timeout_ms.unwrap_or(30_000).clamp(1, 120_000);
    let timeout = std::time::Duration::from_millis(timeout_ms);

    match tauri::async_runtime::timeout(timeout, operation).await {
        Ok(result) => result,
        Err(_) => Err(BrowserError::Navigate(format!(
            "{operation_name} timed out after {timeout_ms}ms"
        ))),
    }
}
```

If `tauri::async_runtime::timeout` does not compile, use `tokio::time::timeout` and add:

```toml
tokio = { version = "1", features = ["time"] }
```

### Suggested load-state policy helper

```rust
#[cfg(feature = "browser")]
fn ensure_supported_load_state(load_state: LoadState) -> Result<(), BrowserError> {
    match load_state {
        LoadState::DomContentLoaded | LoadState::Load => Ok(()),
        LoadState::NetworkIdle => Err(BrowserError::Navigate(String::from(
            "NetworkIdle load waiting is not implemented yet; use Load or DomContentLoaded"
        ))),
    }
}
```

This is acceptable only as an interim fix because it is honest. A real `NetworkIdle` implementation is better, but silently treating it as `Load` is not acceptable.

### Patch shape for `open_url`

```rust
ensure_supported_load_state(load_state)?;
let user_agent = self.config.user_agent.clone();
let session = self.ensure_session()?;
let page = session.ensure_page(user_agent.as_deref())?;

tauri::async_runtime::block_on(async {
    with_browser_timeout(
        async {
            page.goto(url)
                .await
                .map_err(|error| BrowserError::Navigate(error.to_string()))?;
            Ok(())
        },
        timeout_ms,
        "browser navigation",
    )
    .await?;

    snapshot_page_state(&page).await
})
```

### Acceptance checks

- [x] There are no `let _ = load_state;` or `let _ = timeout_ms;` lines in live browser navigation code.
- [x] A requested unsupported load state returns a structured error. (`NetworkIdle` → `BrowserError::Navigate`.)
- [x] A timeout bounds the real async navigation/wait future. (`with_browser_timeout` via `tokio::time::timeout`.)
- [x] Tests cover timeout/unsupported-state behavior at least at a seam/helper level.

---

## P1-1 — Make browser visibility switching two-phase

**Files:**

- `src-tauri/src/browser/mod.rs`
- Possibly `src-tauri/src/browser/session.rs`

### Why

Current code sets `self.session = None` before the replacement browser has launched and navigated to the prior URL. On failure, the user loses the prior working session.

### Tasks

- [x] Refactor `switch_visibility` so it builds a candidate session first.
- [x] Do not commit `self.config.visibility = mode` until the candidate session succeeds.
- [x] Do not drop old `self.session` until the candidate session is ready.
- [x] Replace `.ok().flatten()` URL reads with explicit error handling (`read_current_non_blank_url` → `BrowserError::Inspect`).
- [~] Tests: the candidate-session seam requires a live Chromium; the commit-after-prove invariant is satisfied by construction (verified by code + `no premature session drop` grep). Behavioral test needs a display.

### Suggested helper

```rust
#[cfg(feature = "browser")]
fn read_current_non_blank_url(&self) -> Result<Option<String>, BrowserError> {
    let Some(session) = self.session.as_ref() else {
        return Ok(None);
    };
    let Some(page) = session.page.clone() else {
        return Ok(None);
    };

    tauri::async_runtime::block_on(async {
        page.url()
            .await
            .map_err(|error| BrowserError::Inspect(error.to_string()))
            .map(|url| url.filter(|url| url != "about:blank" && !url.is_empty()))
    })
}
```

### Suggested `switch_visibility` shape

```rust
#[cfg(feature = "browser")]
pub fn switch_visibility(&mut self, mode: BrowserVisibilityMode) -> Result<Option<String>, BrowserError> {
    let prior_url = self.read_current_non_blank_url()?;

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

### Acceptance checks

- [ ] No assignment to `self.session = None` occurs before successful replacement launch.
- [ ] If prior URL read fails, the error is surfaced.
- [ ] If replacement launch/navigation fails, old session remains available.

---

## P1-2 — Make current page snapshot metric failures explicit

**Files:**

- `src-tauri/src/app_core/state_snapshots.rs`
- Callers such as `src-tauri/src/app_core/command_dispatch.rs`
- Types if needed for snapshot diagnostics

### Why

`self.browser.get_page_metrics().ok()?` hides browser metric failures by returning `None`. That looks the same as “there is no current page.”

### Tasks

- [x] Change `current_page_snapshot(...) -> Option<PageSnapshotData>` to return `Result<Option<PageSnapshotData>, ToolError>` or add an explicit diagnostic field.
- [x] Update callers to propagate structured snapshot failure or include it in tool output. (`build_planner_resolution` now `?`-propagates.)
- [~] Add a test/seam proving metrics failure does not silently become `None`. (Structurally guaranteed by the Result type + `?`; a behavioral test needs Chromium or a no-browser test build the `--all-features` gate doesn't run — same seam limit as P1-1.)

### Suggested patch shape

```rust
let metrics = self.browser.get_page_metrics().map_err(|error| ToolError {
    code: String::from("browser_metrics_failed"),
    message: String::from("failed to read browser page metrics for current page snapshot"),
    retryable: true,
    details: Some(serde_json::json!({ "reason": error.to_string() })),
})?;
```

Then use `metrics.scroll_y`, `metrics.viewport_width`, etc.

### Acceptance checks

- [ ] No `.ok()?` remains on browser metric reads.
- [ ] Callers can distinguish no-current-page from metric-read-failed.

---

## P1-3 — Fix remote ASR timeout so it cancels/bounds the request

**Files:**

- `src-tauri/src/asr/remote.rs`
- Possibly `src-tauri/Cargo.toml`

### Why

Current code spawns a thread and waits on `recv_timeout`. When timeout occurs, the spawned network request keeps running. Repeated timeouts can leak in-flight requests/threads and still spend API/network resources.

### Tasks

- [x] Remove the unbounded `thread::spawn + recv_timeout` pattern.
- [x] Use an actual HTTP/request timeout. (`reqwest::blocking` `.timeout(...)`, mirroring remote TTS.)
- [x] Do not ignore send/cancellation errors as a meaningful runtime path. (No more `let _ = sender.send(...)`.)
- [~] Add a test at the seam if possible. (Timeout is delegated to reqwest's tested `.timeout()`; a seam test needs a mock-HTTP dependency, not added. Removal verified by grep.)

### Implementation option A — prefer request-level timeout

If `async_openai` allows injecting a client/configured timeout, use that. The timeout must bound the actual HTTP request, not just the caller wait.

### Implementation option B — use `reqwest::blocking` multipart

The project already depends on blocking `reqwest` for remote TTS-style calls. A direct blocking HTTP implementation with `.timeout(...)` is acceptable if it is simpler and well-tested.

Sketch only; adjust field names to OpenAI transcription API and existing profile types:

```rust
let timeout = Duration::from_millis(profile.timeout_ms.max(1));
let client = reqwest::blocking::Client::builder()
    .timeout(timeout)
    .build()
    .map_err(|error| AsrRuntimeError::RemoteRequestFailed {
        reason: error.to_string(),
    })?;

let part = reqwest::blocking::multipart::Part::bytes(audio_bytes)
    .file_name("command.wav")
    .mime_str("audio/wav")
    .map_err(|error| AsrRuntimeError::RemoteRequestBuildFailed {
        reason: error.to_string(),
    })?;

let form = reqwest::blocking::multipart::Form::new()
    .text("model", profile.model.clone())
    .text("response_format", "json")
    .part("file", part);

let response = client
    .post(format!("{}/audio/transcriptions", profile.base_url.trim_end_matches('/')))
    .bearer_auth(api_key)
    .multipart(form)
    .send()
    .map_err(|error| {
        if error.is_timeout() {
            AsrRuntimeError::RemoteRequestTimedOut {
                timeout_ms: profile.timeout_ms.max(1),
            }
        } else {
            AsrRuntimeError::RemoteRequestFailed {
                reason: error.to_string(),
            }
        }
    })?;
```

If using multipart with current `reqwest` features requires enabling the `multipart` feature, update `Cargo.toml`:

```toml
reqwest = { version = "0.12", default-features = false, features = ["blocking", "rustls-tls", "multipart", "json"] }
```

### Acceptance checks

- [x] `src-tauri/src/asr/remote.rs` no longer uses unbounded `thread::spawn` for timeout.
- [x] Timeout returns `AsrRuntimeError::RemoteRequestTimedOut` and bounds the actual request.
- [x] No `let _ = sender.send(result);` remains in this path.

---

## P1-4 — Fail local ASR on segment conversion errors

**Files:**

- `src-tauri/src/asr/local.rs`

### Why

Current code silently drops transcript segments that fail conversion. That can turn a user’s spoken command into a partial command.

### Tasks

- [x] Replace `filter_map(...ok())` with explicit error handling.
- [x] Add a test if a seam is feasible; otherwise add a small helper for segment collection and test it. (Extracted `collect_transcript_segments`, tested for both join/trim and decode-error-fails.)

### Suggested patch

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

If `error` does not implement `Display`, use:

```rust
reason: format!("failed to decode whisper transcript segment: {error:?}"),
```

### Acceptance checks

- [x] No ASR transcript segment conversion uses `filter_map(...ok())`.
- [x] Segment conversion failure returns `AsrRuntimeError::TranscriptionFailed`.

---

## P1-5 — Fail loudly on invalid bundled skill metadata

**Files:**

- `src-tauri/src/commands/skill_parser.rs`
- `src-tauri/src/commands/skill_loader.rs`
- Tests under `src-tauri/src/commands/tests`

### Why

Current bundled skill parser defaults malformed `requires_confirmation` to `false` and skips invalid bundled skills with a warning. Confirmation metadata must fail closed.

### Tasks

- [x] Change `parse_bundled_skills` to return `Result<Vec<LoadedSkill>, String>` or a typed error.
- [x] Make malformed `requires_confirmation` an error.
- [x] Make invalid bundled skill frontmatter/tool names an error.
- [x] Update `skill_loader` and callers to propagate the error. (Bundled skills are a compile-time `include_str!` asset, so `discover_skills` `.expect()`s the parse — a build defect fails loudly at startup; a regression test parses the shipped bundle so CI catches it first.)
- [x] Preserve best-effort behavior only for user/project skills if the UI surfaces errors. (Unchanged — only the bundled path was hardened.)
- [x] Add parser tests. (Shipped-bundle parses; malformed requires_confirmation rejected; unknown tool rejected.)

### Snippet: replace malformed bool default

Bad:

```rust
requires_confirmation_value = parse_bool_value(value).unwrap_or(false);
```

Good shape:

```rust
requires_confirmation_value = parse_bool_value(value).ok_or_else(|| {
    format!(
        "invalid requires_confirmation value for bundled skill {}: {}",
        current_name.as_deref().unwrap_or("<unknown>"),
        value.trim()
    )
})?;
```

### Snippet: make flush fallible

Current `flush_skill` closure logs and skips invalid bundled skills. Change its return type to `Result<(), String>` and do this instead:

```rust
let frontmatter = skill_frontmatter_from_parts(scalar_fields, list_fields, available_tool_names)
    .map_err(|error| {
        format!(
            "invalid bundled skill {}: {error}",
            current_name_for_error
        )
    })?;

skills.push(LoadedSkill {
    summary: skill_summary_from_frontmatter(frontmatter),
    body: description.trim().to_string(),
    source: SkillSource::Bundled,
});
```

### Suggested tests

```rust
#[test]
fn parse_bundled_skills_rejects_invalid_requires_confirmation() {
    let markdown = r#"
#### risky_skill
- intent_tags: `submit`
- allowed_tools: `SubmitForm`
- requires_confirmation: maybe
- description: Submit a form.
"#;

    let error = parse_bundled_skills(markdown, &planner_available_tools())
        .expect_err("invalid requires_confirmation must fail bundled skill parsing");
    assert!(error.contains("requires_confirmation"));
}

#[test]
fn parse_bundled_skills_rejects_unknown_tool() {
    let markdown = r#"
#### bad_tool_skill
- intent_tags: `open`
- allowed_tools: `DefinitelyNotATool`
- requires_confirmation: false
- description: Bad tool.
"#;

    let error = parse_bundled_skills(markdown, &planner_available_tools())
        .expect_err("unknown bundled tool must fail bundled skill parsing");
    assert!(error.contains("DefinitelyNotATool"));
}
```

Adjust helper names/imports to match the existing command test module.

### Acceptance checks

- [x] Malformed bundled skill confirmation metadata cannot produce `requires_confirmation = false`.
- [x] Invalid bundled skills fail tests/startup instead of being skipped.
- [x] User-visible skill loading errors remain actionable. (User/project skip path unchanged.)

---

## P1-6 — Preserve truthful voice listening state on frontend failures

**Files:**

- `src/voice-loop.ts`
- Existing voice-loop or panel-state tests

### Why

Several catch blocks force `isListening: false` after backend errors. If the backend failed to stop/transcribe, the runtime may still be listening.

### Tasks

- [ ] Add a helper that reports failure without inventing listening state.
- [ ] Use it in `stopContinuousListeningAfterFailure`, `cancelPushToTalk`, and `releasePushToTalk` catch paths.
- [ ] Consider using it in `beginPushToTalk` catch path too, unless start failure always proves no capture started.
- [ ] Add tests proving previous listening state is preserved until refresh gives authoritative state.

### Suggested helper

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

Use the project’s normal error formatter instead of `String(refreshError)` if one exists for refresh failures.

### Example replacement

Bad:

```ts
catch (error: unknown) {
  setPushToTalkState({
    isListening: false,
    isBusy: false,
    lastError: describePushToTalkFailure(error),
  });
}
```

Better:

```ts
catch (error: unknown) {
  await reportPushToTalkFailureWithoutInventingListeningState(
    describePushToTalkFailure(error),
  );
}
```

For non-async function paths, make the caller async or call `void report...` only if the UI is updated immediately to “unknown/pending refresh.”

### Acceptance checks

- [ ] No voice-loop catch block sets `isListening: false` without backend confirmation.
- [ ] Tests prove failed stop/transcribe preserves prior listening state.
- [ ] User sees an explicit error when runtime state cannot be confirmed.

---

## P2-1 — Surface stale confirmation submission attempts

**Files:**

- `src/voice-loop.ts`
- `src/planner-orchestration.ts` or confirmation UI store if needed
- Confirmation panel tests

### Why

`submitConfirmationAction` silently returns when the submitted confirmation ID is stale/mismatched. Silent no-ops are bad accessibility UX.

### Tasks

- [ ] Keep stale-click guard.
- [ ] Add visible or at least logged feedback for mismatched active confirmation ID.
- [ ] Add a test.

### Suggested patch shape

```ts
if (confirmationId !== confirmationState.confirmationId) {
  const message = "That confirmation is no longer active. Review the current confirmation before approving.";
  uiStore.setConfirmationError(confirmationState.confirmationId, message);
  console.warn("Ignored stale confirmation response.", {
    submittedConfirmationId: confirmationId,
    activeConfirmationId: confirmationState.confirmationId,
  });
  return;
}
```

If `setConfirmationError` requires the submitted ID rather than active ID, use the existing store semantics. The important part is: do not disappear silently when another confirmation is active.

### Acceptance checks

- [ ] Stale confirmation ID mismatch produces visible/debuggable feedback.
- [ ] Duplicate submission while already submitting remains safely ignored.

---

## P2-2 — Stop generic runtime refresh from clearing unrelated action errors

**Files:**

- `src/runtime-refresh.ts`
- `src/panel-state.ts`
- `src/panel-state-setters.ts`
- Existing runtime-refresh tests

### Why

Runtime refresh currently clears several panel errors. Some are stale runtime errors, but some may be user-action errors that should remain until retried/dismissed.

### Tasks

- [ ] Identify every `lastError: null` / action-error clear in `runtime-refresh.ts`.
- [ ] Classify each as runtime-owned or action-owned.
- [ ] Add separate fields if needed: `runtimeRefreshError`, `lastActionError`.
- [ ] Ensure generic refresh clears only runtime-owned errors.
- [ ] Add regression tests.

### Suggested model

```ts
type PanelErrorState = {
  runtimeRefreshError: string | null;
  lastActionError: string | null;
};
```

Do not necessarily apply this exact type everywhere if the existing state shape has a better pattern. The invariant is what matters.

### Acceptance checks

- [ ] Failed API key save/test errors are not cleared by unrelated runtime refresh.
- [ ] Failed URL/audio/confirmation action errors are not cleared by unrelated runtime refresh.
- [ ] Runtime refresh success can clear only refresh-owned errors.

---

## P2-3 — Default ordinary clicks to confirmation

**Files:**

- `config.example.toml`
- Possibly docs/README/settings docs
- Tests if config defaults are tested

### Why

For an LLM/planner-controlled blind browser, conservative click confirmation is safer by default.

### Tasks

- [ ] Change example/default config to `allow_click_without_confirmation = false`.
- [ ] Update docs explaining how to opt into faster unconfirmed clicks.
- [ ] Ensure submit confirmation remains always-on by default.

### Suggested patch

```toml
[safety]
confirmation_confidence_threshold = 0.90
allow_click_without_confirmation = false
always_confirm_submit = true
```

### Acceptance checks

- [ ] Fresh config defaults are conservative.
- [ ] User can still intentionally enable unconfirmed safe clicks from settings/config.

---

## P2-4 — Replace timestamp-zero fallback with explicit unique IDs or errors

**Files:**

- `src-tauri/src/commands/mod.rs`
- `src-tauri/src/app_core/mod.rs`

### Why

Returning `0` when system time is before UNIX epoch can create duplicate/non-informative IDs. Low risk, but easy to clean up.

### Tasks

- [ ] Search for `Err(_) => 0` in timestamp helpers.
- [ ] Replace with an atomic counter fallback or return an error where feasible.
- [ ] Add a tiny helper test if possible.

### Suggested fallback

```rust
use std::sync::atomic::{AtomicU64, Ordering};

static FALLBACK_TIMESTAMP_COUNTER: AtomicU64 = AtomicU64::new(1);

fn current_timestamp_ms() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as u64,
        Err(_) => FALLBACK_TIMESTAMP_COUNTER.fetch_add(1, Ordering::Relaxed),
    }
}
```

Prefer a scoped helper name if there are multiple modules.

### Acceptance checks

- [ ] No timestamp helper silently returns zero.

---

## P3-1 — Add no-silent-fallback guardrails to CI/static checks

**Files:**

- `.github/workflows/ci.yml`
- Possibly a script under `scripts/`

### Tasks

- [ ] Add a lightweight grep check for risky ignored semantic parameters.
- [ ] Do not ban every `let _ =` globally; some are legitimate. Start with a targeted denylist.

### Suggested script

```bash
#!/usr/bin/env bash
set -euo pipefail

for pattern in \
  "let _ = load_state" \
  "let _ = timeout_ms" \
  "unwrap_or(false)" \
  "filter_map(|segment| segment.to_str_lossy().ok())" \
  '"csp": null'
do
  if grep -R "$pattern" -n src src-tauri; then
    echo "Found forbidden silent-fallback pattern: $pattern" >&2
    exit 1
  fi
done
```

Tune as needed to avoid false positives, but keep guardrails for the exact regressions fixed in this pass.

---

## Final validation checklist

Run and record results in the PR/commit message:

```bash
pnpm install
pnpm test
pnpm build
cd src-tauri
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Completion criteria:

- [ ] CSP is non-null.
- [ ] Internal planner/browser navigation is HTTP/HTTPS only.
- [ ] Dangerous schemes are rejected in both validator and runtime paths.
- [ ] Navigation load/timeout behavior is implemented or explicit unsupported errors are returned.
- [ ] Visibility switching does not destroy the old session until replacement succeeds.
- [ ] Snapshot metric failures are explicit.
- [ ] Remote ASR timeout bounds/cancels the request.
- [ ] Local ASR does not silently drop transcript segments.
- [ ] Bundled skill metadata parse defects fail loudly.
- [ ] Voice UI does not invent `isListening: false` after backend failure.
- [ ] Stale confirmation submissions are visible/debuggable.
- [ ] Runtime refresh does not clear unrelated action errors.
- [ ] Default click confirmation is conservative.
- [ ] Tests cover the safety regressions.
