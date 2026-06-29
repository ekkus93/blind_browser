# Blind Browser Security and Silent-Failure Hardening 2 TODO

## How to use this file

This TODO is a focused follow-up to the prior security/silent-failure hardening pass. Do not redo already-completed work unless a task explicitly touches it.

Status key: `PENDING` · `IN PROGRESS` · `DONE` · `BLOCKED`

Priority key:

- `P0`: security/safety invariant or data-integrity issue that can mislead users or corrupt durable state.
- `P1`: correctness or silent-failure risk likely to cause bad behavior.
- `P2`: UX clarity, diagnostics, or conservative hardening.
- `P3`: cleanup/static guardrail after behavior is safe.

Validation gate:

```bash
pnpm install
pnpm test
pnpm build
cd src-tauri
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Do not mark this TODO complete unless the validation gate actually passes.

---

## P0-1 — Surface configured remote planner API-key resolution failures

**Status:** PENDING  
**Files:**

- `src-tauri/src/app_core/runtime_config.rs`
- relevant app-core/runtime-config tests

### Problem

`list_remote_planner_models()` still treats configured API-key resolution failure as `None`:

```rust
resolve_secret_ref(&profile.api_key).ok()
```

That is a dangerous silent fallback. If the configured keyring/env/file secret is unreadable, the app sends an unauthenticated model-list request and the user sees a generic remote failure instead of the real configuration error.

### Required behavior

- If `api_key_override` is non-empty, use it.
- If no override is provided and the profile has a configured secret reference, resolve it.
- If resolving the configured secret fails, return a clear error.
- Do not silently convert secret-resolution failure into `None`.
- If anonymous/no-key endpoints are intentionally supported, make that an explicit profile setting later. Do not implement that implicit fallback here.

### Suggested patch shape

Find the current API key logic in `list_remote_planner_models()` and replace it with a helper.

```rust
fn resolve_optional_remote_planner_api_key(
    profile: &RemotePlannerProfile,
    api_key_override: Option<&str>,
) -> Result<Option<String>, String> {
    if let Some(override_value) = api_key_override.map(str::trim) {
        if !override_value.is_empty() {
            return Ok(Some(override_value.to_string()));
        }
    }

    match resolve_secret_ref(&profile.api_key) {
        Ok(value) => Ok(Some(value)),
        Err(reason) => Err(format!(
            "Remote planner model list could not read the configured API key: {reason}"
        )),
    }
}
```

Then use it:

```rust
let api_key = resolve_optional_remote_planner_api_key(profile, api_key_override)
    .map_err(|message| RemotePlannerConfigError::InvalidProfile(message))?;
```

Adjust the error type to the actual return type in `runtime_config.rs`. The important invariant is: **configured secret failure is an error, not `None`**.

If the existing `resolve_secret_ref` returns a “not configured” error for an intentionally absent key, split that case explicitly:

```rust
match resolve_secret_ref(&profile.api_key) {
    Ok(value) => Ok(Some(value)),
    Err(SecretRefError::NotConfigured) => Ok(None),
    Err(reason) => Err(format!(
        "Remote planner model list could not read the configured API key: {reason}"
    )),
}
```

Use the actual secret error enum if one exists. Do not string-match error messages unless no typed error exists.

### Tests

Add tests for:

1. override key wins and does not inspect configured secret,
2. configured secret resolves successfully,
3. configured secret resolution failure returns a visible/configuration error,
4. failure is not converted to `None`.

Suggested test skeleton, adapt to existing helpers:

```rust
#[test]
fn list_remote_planner_models_reports_configured_secret_resolution_failure() {
    let profile = remote_planner_profile_with_api_key_ref("keyring://missing/planner");

    let error = resolve_optional_remote_planner_api_key(&profile, None)
        .expect_err("configured secret resolution failure must be surfaced");

    assert!(
        error.contains("could not read the configured API key"),
        "unexpected error: {error}"
    );
}
```

### Acceptance checks

```bash
rg -n "resolve_secret_ref\(&profile\.api_key\)\.ok\(\)" src-tauri/src/app_core/runtime_config.rs
```

Expected: no matches.

---

## P0-2 — Make model downloads atomic and partial-file-safe

**Status:** PENDING  
**Files:**

- `src-tauri/src/app_core/model_management.rs`
- tests under app-core/model-management if present

### Problem

Model downloads currently write directly to the final target path:

```rust
let mut output = fs::File::create(target_path)?;
response.copy_to(&mut output)?;
```

If the download fails halfway, the final file still exists and may later be treated as installed.

### Required behavior

- Download to a temporary sibling file such as `model.bin.part`.
- Flush/sync the temporary file.
- Rename the temporary file to the final path only after successful write.
- Delete the partial file on failure.
- Never leave a failed download at the final path.

### Suggested helper

Add a helper near the existing download code:

```rust
fn download_response_to_file_atomically(
    mut response: reqwest::blocking::Response,
    target_path: &Path,
) -> Result<(), String> {
    let parent = target_path
        .parent()
        .ok_or_else(|| format!("download target {} has no parent directory", target_path.display()))?;

    fs::create_dir_all(parent)
        .map_err(|error| format!("failed to create model directory {}: {error}", parent.display()))?;

    let file_name = target_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("download target {} has no valid file name", target_path.display()))?;

    let tmp_path = target_path.with_file_name(format!("{file_name}.part"));

    let result = (|| -> Result<(), String> {
        let mut output = fs::File::create(&tmp_path)
            .map_err(|error| format!("failed to create temporary model file {}: {error}", tmp_path.display()))?;

        response
            .copy_to(&mut output)
            .map_err(|error| format!("failed to write model file {}: {error}", tmp_path.display()))?;

        output
            .sync_all()
            .map_err(|error| format!("failed to sync model file {}: {error}", tmp_path.display()))?;

        fs::rename(&tmp_path, target_path)
            .map_err(|error| {
                format!(
                    "failed to finalize model file {} from {}: {error}",
                    target_path.display(),
                    tmp_path.display()
                )
            })?;

        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }

    result
}
```

Then replace direct final-path writes:

```rust
download_response_to_file_atomically(response, target_path)?;
```

If the existing error type is not `String`, map errors into the project’s model-management error type.

### Tests

Add a helper-level test if practical:

```rust
#[test]
fn failed_model_download_does_not_leave_final_target_file() {
    // Use a fake writer/response seam if available.
    // If not available, extract the temp/finalize path logic into a small helper
    // that can be tested without network.
}
```

At minimum, test temp-path cleanup around a simulated failure by extracting the finalization helper.

### Acceptance checks

```bash
rg -n "File::create\(target_path\)|fs::File::create\(target_path\)|copy_to\(&mut output\)" src-tauri/src/app_core/model_management.rs
```

Expected: no direct final-path download write remains, or any remaining match is inside the atomic helper and writes to the temporary path.

---

## P0-3 — Strengthen model availability checks

**Status:** PENDING  
**Files:**

- `src-tauri/src/app_core/model_management.rs`
- model-management tests

### Problem

Local model availability checks are too weak. For ASR:

```rust
Path::new(profile.model_path.trim()).is_file()
```

An empty/truncated file can be considered installed.

TTS checks presence of expected files and any `.onnx`, but does not reject tiny/empty files.

### Required behavior

- Empty files are not available.
- Very small files are not available.
- Local ASR model availability requires at least a conservative minimum size.
- TTS required files must be non-empty.
- At least one `.onnx` model must be non-empty and plausibly sized.

This pass does not require full checksum validation unless checksum metadata already exists.

### Suggested constants

```rust
const MIN_LOCAL_ASR_MODEL_BYTES: u64 = 1_000_000; // 1 MB sanity floor
const MIN_TTS_CONFIG_BYTES: u64 = 2;
const MIN_TTS_VOICES_BYTES: u64 = 1_000;
const MIN_TTS_ONNX_BYTES: u64 = 1_000_000;
```

Tune values if tests/real models require different thresholds. The goal is to reject obviously corrupt files, not prove model correctness.

### Suggested helper

```rust
fn file_size_at_least(path: &Path, min_bytes: u64) -> bool {
    path.metadata()
        .map(|metadata| metadata.is_file() && metadata.len() >= min_bytes)
        .unwrap_or(false)
}
```

Patch ASR:

```rust
pub(crate) fn local_asr_model_is_available(profile: &LocalAsrProfile) -> bool {
    let path = Path::new(profile.model_path.trim());
    file_size_at_least(path, MIN_LOCAL_ASR_MODEL_BYTES)
}
```

Patch TTS logic wherever it checks `config.json`, `voices.npz`, and `.onnx`:

```rust
let config_ok = file_size_at_least(&model_dir.join("config.json"), MIN_TTS_CONFIG_BYTES);
let voices_ok = file_size_at_least(&model_dir.join("voices.npz"), MIN_TTS_VOICES_BYTES);
let has_onnx = fs::read_dir(model_dir)
    .ok()
    .into_iter()
    .flat_map(|entries| entries.filter_map(Result::ok))
    .map(|entry| entry.path())
    .any(|path| {
        path.extension().and_then(|ext| ext.to_str()) == Some("onnx")
            && file_size_at_least(&path, MIN_TTS_ONNX_BYTES)
    });
```

### Tests

Add tests:

```rust
#[test]
fn local_asr_model_availability_rejects_empty_file() {
    let temp = tempfile::tempdir().unwrap();
    let model_path = temp.path().join("tiny.gguf");
    std::fs::write(&model_path, []).unwrap();

    let profile = LocalAsrProfile {
        model_path: model_path.display().to_string(),
        // fill other fields from default/test helper
    };

    assert!(!local_asr_model_is_available(&profile));
}
```

Use existing config/profile test helpers if available.

### Acceptance checks

- Empty ASR model file is unavailable.
- Tiny ASR model file is unavailable.
- Empty TTS required files are unavailable.
- Existing valid fixture/model checks still behave correctly.

---

## P0-4 — Make config persistence writes atomic

**Status:** PENDING  
**Files:**

- `src-tauri/src/config/persistence.rs`
- config persistence tests

### Problem

Config persistence uses direct `fs::write(path, serialized)` in multiple places. A crash, disk-full error, or power loss can corrupt/truncate the config file.

### Required behavior

- Write to temporary sibling file.
- Flush and sync the temporary file.
- Rename over the final path only after successful write.
- Do not truncate the old config if the new write fails.
- Use one shared helper for config writes.

### Suggested helper

Add to `persistence.rs`:

```rust
fn write_config_atomic(path: &Path, serialized: &str) -> Result<(), ConfigError> {
    let parent = path.parent().ok_or_else(|| {
        ConfigError::Validation(format!(
            "config path {} has no parent directory",
            path.display()
        ))
    })?;

    fs::create_dir_all(parent).map_err(|source| ConfigError::CreateDir {
        path: parent.to_path_buf(),
        source,
    })?;

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            ConfigError::Validation(format!(
                "config path {} has no valid file name",
                path.display()
            ))
        })?;

    let tmp_path = path.with_file_name(format!("{file_name}.tmp"));

    let write_result = (|| -> Result<(), ConfigError> {
        use std::io::Write;

        let mut file = fs::File::create(&tmp_path).map_err(|source| ConfigError::Write {
            path: tmp_path.clone(),
            source,
        })?;

        file.write_all(serialized.as_bytes())
            .map_err(|source| ConfigError::Write {
                path: tmp_path.clone(),
                source,
            })?;

        file.sync_all().map_err(|source| ConfigError::Write {
            path: tmp_path.clone(),
            source,
        })?;

        fs::rename(&tmp_path, path).map_err(|source| ConfigError::Write {
            path: path.to_path_buf(),
            source,
        })?;

        Ok(())
    })();

    if write_result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }

    write_result
}
```

Then replace direct writes:

```rust
fs::write(path, serialized).map_err(...)
```

with:

```rust
write_config_atomic(path, &serialized)?;
```

Adjust variable names/types to match existing functions.

### Tests

Add tests where practical:

```rust
#[test]
fn write_config_atomic_writes_expected_content() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("config.toml");

    write_config_atomic(&path, "value = 1
").unwrap();

    assert_eq!(std::fs::read_to_string(path).unwrap(), "value = 1
");
}
```

If simulating rename/write failure is hard cross-platform, test helper behavior and ensure no direct `fs::write` remains in persistence paths.

### Acceptance checks

```bash
rg -n "fs::write\(" src-tauri/src/config/persistence.rs
```

Expected: no direct config persistence write remains, except inside tests if intentionally testing old behavior.

---

## P1-1 — Treat malformed remote ASR success JSON as an error

**Status:** PENDING  
**Files:**

- `src-tauri/src/asr/remote.rs`
- remote ASR tests

### Problem

Remote ASR currently treats a successful JSON response with no string `text` field as an empty transcript:

```rust
Ok(parsed
    .get("text")
    .and_then(|value| value.as_str())
    .unwrap_or_default()
    .to_string())
```

That is a quiet failure. Malformed provider responses should not look like user silence.

### Required behavior

- Missing `text` is an error.
- Non-string `text` is an error.
- Empty string is allowed only if `text` exists and is a string.

### Suggested helper

Extract parsing:

```rust
fn parse_remote_transcription_text(parsed: &serde_json::Value) -> Result<String, AsrRuntimeError> {
    let text = parsed
        .get("text")
        .and_then(|value| value.as_str())
        .ok_or_else(|| AsrRuntimeError::RemoteRequestFailed {
            reason: String::from(
                "remote transcription response did not contain a string 'text' field",
            ),
        })?;

    Ok(text.to_string())
}
```

Then replace the current `unwrap_or_default()` block:

```rust
let parsed = response
    .json::<serde_json::Value>()
    .map_err(|error| AsrRuntimeError::RemoteRequestFailed {
        reason: error.to_string(),
    })?;

parse_remote_transcription_text(&parsed)
```

### Tests

```rust
#[test]
fn parse_remote_transcription_text_requires_text_field() {
    let parsed = serde_json::json!({ "duration": 1.0 });

    let error = parse_remote_transcription_text(&parsed)
        .expect_err("missing text must be an error");

    assert!(
        error.to_string().contains("text"),
        "unexpected error: {error}"
    );
}

#[test]
fn parse_remote_transcription_text_rejects_non_string_text() {
    let parsed = serde_json::json!({ "text": 42 });

    assert!(parse_remote_transcription_text(&parsed).is_err());
}

#[test]
fn parse_remote_transcription_text_allows_empty_string_text() {
    let parsed = serde_json::json!({ "text": "" });

    assert_eq!(parse_remote_transcription_text(&parsed).unwrap(), "");
}
```

Make the helper `pub(crate)` if tests live in another module.

### Acceptance checks

```bash
rg -n "unwrap_or_default\(\).*to_string\(\)" src-tauri/src/asr/remote.rs
```

Expected: no remote ASR response text fallback remains.

---

## P1-2 — Strengthen URL policy with real URL parsing and host validation

**Status:** PENDING  
**Files:**

- `src-tauri/src/url_policy.rs`
- `src-tauri/Cargo.toml` if needed
- URL policy tests
- planner validation tests if error details/messages change

### Problem

The current URL policy blocks dangerous schemes but uses hand-rolled authority checks. It may accept malformed HTTP(S) inputs that a real parser would reject or normalize unexpectedly.

Examples to reject before browser execution:

```text
http://:80
https://exa mple.com
https://
https://
example.com
```

### Required behavior

- Use the `url` crate or equivalent robust parser.
- Accept only `http` and `https`.
- Require a valid host.
- Reject control characters and whitespace inside the URL.
- Return structured `UrlPolicyError` values with useful messages/details.
- Keep planner validation and runtime navigation using the same shared policy.

### Cargo dependency

If not already present:

```toml
url = "2"
```

### Suggested enum addition

Add a general parse variant if useful:

```rust
InvalidUrl { url: String, reason: String },
```

Update `code()`, `user_message()`, and `details()`:

```rust
Self::InvalidUrl { .. } => "invalid_url",
```

```rust
Self::InvalidUrl { .. } => "open_url requires a valid absolute http or https URL",
```

```rust
Self::InvalidUrl { url, reason } => serde_json::json!({
    "url": url,
    "reason": reason,
}),
```

### Suggested implementation

```rust
pub fn normalize_browser_navigation_url(raw: &str) -> Result<String, UrlPolicyError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(UrlPolicyError::Empty);
    }

    if trimmed.chars().any(|ch| ch.is_control() || ch.is_whitespace()) {
        return Err(UrlPolicyError::InvalidUrl {
            url: trimmed.to_string(),
            reason: String::from("URL contains whitespace or control characters"),
        });
    }

    let parsed = url::Url::parse(trimmed).map_err(|error| UrlPolicyError::InvalidUrl {
        url: trimmed.to_string(),
        reason: error.to_string(),
    })?;

    let scheme = parsed.scheme().to_ascii_lowercase();
    if !is_allowed_browser_navigation_scheme(&scheme) {
        return Err(UrlPolicyError::UnsupportedScheme {
            url: trimmed.to_string(),
            scheme,
        });
    }

    if parsed.host_str().is_none() {
        return Err(UrlPolicyError::MissingAuthority {
            url: trimmed.to_string(),
            scheme,
        });
    }

    Ok(parsed.to_string())
}
```

If preserving exact user input formatting is important, return `trimmed.to_string()` only after parser validation. But using `parsed.to_string()` is usually safer because it normalizes the URL.

### Tests

Add cases:

```rust
#[test]
fn rejects_malformed_http_authorities() {
    for raw in [
        "http://:80",
        "https://",
        "https:///missing-host",
        "https://exa mple.com",
        "https://
example.com",
        "https://	example.com",
    ] {
        assert!(
            normalize_browser_navigation_url(raw).is_err(),
            "expected malformed URL to be rejected: {raw:?}"
        );
    }
}
```

Keep existing dangerous-scheme tests.

### Acceptance checks

- Runtime navigation still rejects `file:`, `javascript:`, `data:`, `chrome:`, and `about:`.
- Runtime navigation rejects malformed HTTP(S) authorities.
- Planner validation uses the same policy.
- Tests cover both dangerous schemes and malformed HTTP(S) authorities.

---

## P2-1 — Surface masked-secret inspection failures in settings

**Status:** PENDING  
**Files:**

- `src-tauri/src/commands/settings_adapters.rs`
- panel types/renderers for settings if necessary
- settings tests

### Problem

Settings code contains patterns like:

```rust
masked_secret_value(...).ok()?
```

A keyring/secret failure can look like “no masked key available.” That is lower severity than the model-list request path, but it still misleads users.

### Required behavior

- “No secret configured” and “configured secret could not be inspected” should be distinct.
- Settings should surface a warning/error when a configured secret reference cannot be masked/read.
- Do not silently collapse secret-inspection failure to `None`.

### Suggested backend type

If the existing settings DTO can change:

```rust
#[derive(Debug, Clone, Serialize)]
pub struct SecretReferenceStatus {
    pub label: Option<String>,
    pub error: Option<String>,
}
```

Or minimally add an error field next to existing reference fields:

```rust
api_key_reference: Option<String>,
api_key_reference_error: Option<String>,
```

### Suggested helper

```rust
fn masked_secret_reference_status(reference: &SecretReference) -> SecretReferenceStatus {
    match masked_secret_value(reference) {
        Ok(label) => SecretReferenceStatus {
            label: Some(label),
            error: None,
        },
        Err(error) => SecretReferenceStatus {
            label: None,
            error: Some(format!("Configured secret could not be inspected: {error}")),
        },
    }
}
```

If `masked_secret_value` returns `Option` for not-configured, preserve that distinction:

```rust
match masked_secret_value(reference) {
    Ok(Some(label)) => SecretReferenceStatus { label: Some(label), error: None },
    Ok(None) => SecretReferenceStatus { label: None, error: None },
    Err(error) => SecretReferenceStatus {
        label: None,
        error: Some(format!("Configured secret could not be inspected: {error}")),
    },
}
```

Use actual return types.

### Frontend rendering

In the relevant settings panel, display the error as a warning near the key reference:

```tsx
{state.apiKeyReferenceError ? (
  <p className="settings-warning" role="alert">
    {state.apiKeyReferenceError}
  </p>
) : null}
```

### Acceptance checks

```bash
rg -n "masked_secret_value.*\.ok\(\)\?" src-tauri/src/commands/settings_adapters.rs
```

Expected: no matches.

---

## P2-2 — Extend silent-fallback guardrails for exact new regressions

**Status:** PENDING  
**Files:**

- `scripts/check-silent-fallbacks.sh`
- `.github/workflows/ci.yml` if needed

### Problem

The existing guardrail script caught the previous patterns. Add narrowly targeted checks for the new exact anti-patterns after they are fixed.

### Add exact denylist checks

Add only patterns specific enough to avoid false positives:

```bash
patterns=(
  'resolve_secret_ref(&profile.api_key).ok()'
  'unwrap_or_default().to_string()'
  'masked_secret_value.*\.ok\(\)\?'
)
```

Because shell grep does not handle all regex forms portably in basic mode, use `grep -R -E` for regex patterns or separate fixed-string checks.

Suggested implementation style:

```bash
if grep -R -F 'resolve_secret_ref(&profile.api_key).ok()' src-tauri/src; then
  echo "Found forbidden remote planner secret fallback" >&2
  exit 1
fi

if grep -R -E 'masked_secret_value.*\.ok\(\)\?' src-tauri/src/commands/settings_adapters.rs; then
  echo "Found forbidden masked-secret inspection fallback" >&2
  exit 1
fi

if grep -R -F 'unwrap_or_default().to_string()' src-tauri/src/asr/remote.rs; then
  echo "Found forbidden remote ASR missing-text fallback" >&2
  exit 1
fi
```

Do **not** ban all `.ok()` or all `unwrap_or_default()` globally.

### Acceptance checks

- Script passes after fixes.
- Script fails if one of the exact removed patterns is reintroduced.
- CI still runs the script.

---

## P2-3 — Final validation and memory entry

**Status:** PENDING  
**Files:**

- `memory.md`
- this TODO file, if tracked in repo

### Tasks

1. Run static guardrail script:

```bash
bash scripts/check-silent-fallbacks.sh
```

2. Run full validation:

```bash
pnpm install
pnpm test
pnpm build
cd src-tauri
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

3. Update `memory.md` with a real UTC timestamp:

```bash
date -u +"%Y-%m-%dT%H:%M:%SZ"
```

Suggested memory entry:

```md
- 2026-XX-XXTXX:XX:XXZ — Completed Security/Silent-Failure Hardening 2: surfaced configured secret-resolution failures, made model downloads/config writes atomic, strengthened model availability and URL parsing, rejected malformed remote ASR responses, surfaced masked-secret inspection failures, extended silent-fallback guardrails, and ran full validation.
```

Do not fabricate the timestamp. Use the command output.

---

## Suggested commit sequence

1. `fix(planner): surface remote planner secret resolution failures`
2. `fix(models): make downloads atomic and validate model availability`
3. `fix(config): write persisted config atomically`
4. `fix(asr): reject malformed remote transcription responses`
5. `fix(security): parse and validate navigation URLs with host checks`
6. `fix(settings): surface masked secret inspection failures`
7. `test: extend silent-failure regression guardrails`
8. `docs: record hardening 2 validation`

---

## Final done checklist

- [ ] Remote planner model listing does not swallow configured API-key resolution failures.
- [ ] Model downloads use temp-file + sync + rename and clean up partial files on failure.
- [ ] Model availability checks reject empty/obviously partial files.
- [ ] Config persistence writes are atomic.
- [ ] Remote ASR rejects missing/non-string `text` in success JSON.
- [ ] URL policy uses robust parsing and rejects malformed HTTP(S) authorities.
- [ ] Settings UI distinguishes missing secret from secret-inspection failure.
- [ ] Silent-fallback guard script covers the new exact regressions.
- [ ] Full validation gate passes.
- [ ] `memory.md` has a real UTC completion entry.
