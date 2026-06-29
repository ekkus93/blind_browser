# Blind Browser Security and Silent-Failure Hardening 2 Spec

## Purpose

This spec covers the follow-up hardening work after `BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING_TODO`.

The previous hardening pass fixed the major requested items: CSP was restored, browser navigation is limited to web schemes, navigation timeout/load-state behavior is no longer silently ignored, visibility switching is two-phase, browser metric failures are explicit, ASR timeout and segment failures are safer, bundled skill metadata fails loudly, voice listening state is more truthful, stale confirmations surface feedback, runtime refresh no longer clears action-owned errors, and conservative click confirmation defaults are in place.

This second pass should focus only on the remaining silent-failure and data-integrity issues found during review.

## Goals

1. Stop treating secret-resolution failures as “no API key”.
2. Prevent partial/corrupt model downloads from looking installed.
3. Prevent config corruption from crash/power-loss during writes.
4. Treat malformed remote ASR success responses as errors, not silence.
5. Strengthen browser URL policy by using real URL parsing and host validation.
6. Avoid settings UI presenting keyring/secret errors as “no masked key available”.
7. Add regression tests and static guardrails for these exact failure modes.

## Non-goals

Do not:

- Redo the completed CSP/URL-policy/navigation-timeout/visibility-switching/voice-loop hardening.
- Replace the global app-alert architecture.
- Rewrite the configuration system.
- Add broad new UI features.
- Introduce a large dependency-injection framework.
- Add noisy static checks that ban all `.ok()` or all `unwrap_or(...)` uses.
- Mark tasks done without running the project validation gate.

## Design principles

### Fail closed for configured secrets

If a profile is configured with an API-key reference, failure to resolve that reference is an error. It must not be silently interpreted as “anonymous request is okay.”

Anonymous/no-key operation should be explicit. Do not use failed secret reads as an implicit anonymous fallback.

### Atomic writes for durable user data

Config files and downloaded model files must not be written directly to final paths. Use a temporary sibling path, flush/sync, then rename. Remove partial files on error.

### “Available” should mean usable enough to try

A model file that exists but is zero bytes or obviously too small should not be considered available.

This pass does not require full cryptographic verification for every model, but minimum-size checks and partial-file cleanup are required. If checksums are available in the project metadata, use them.

### Malformed provider responses are not silence

A remote ASR response with successful HTTP status but no string `text` field is malformed. It should surface a structured ASR error. An empty string is acceptable only if the field exists and is a string.

### URL policy should use a real parser

The current web-only URL policy blocks dangerous schemes but uses hand-rolled authority checks. It should parse with the `url` crate and require a host for `http`/`https`.

### UI should distinguish “missing” from “failed to inspect”

If settings cannot read or mask a configured secret reference, the UI should show an explicit warning/error, not make it look like no secret exists.

## Expected files touched

Likely Rust files:

- `src-tauri/src/app_core/runtime_config.rs`
- `src-tauri/src/app_core/model_management.rs`
- `src-tauri/src/config/persistence.rs`
- `src-tauri/src/asr/remote.rs`
- `src-tauri/src/url_policy.rs`
- `src-tauri/src/commands/settings_adapters.rs`
- relevant Rust tests under `src-tauri/src/**/tests`
- `src-tauri/Cargo.toml` if the `url` crate is not already available

Likely frontend files only if surfacing masked-secret diagnostics requires panel state/types:

- `src/panel-types.ts`
- settings panel renderers for remote planner/ASR/TTS key references
- related UI tests

Docs/CI:

- `docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING2_TODO.md`
- `memory.md`
- optional `scripts/check-silent-fallbacks.sh` additions for exact regressions

## Acceptance summary

The implementation is complete only when:

```bash
pnpm install
pnpm test
pnpm build
cd src-tauri
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

passes in the developer environment, and the targeted static checks in the TODO also pass.

## Targeted invariants

### Remote planner model listing

- If an API-key override is provided, use it.
- If no override is provided and a configured API-key reference exists, resolve it.
- If resolving the configured secret fails, return a clear structured error.
- Do not convert secret failure into `None`.

### Model downloads

- Downloads write to `.part`/temporary file first.
- Final path is created only after successful write/sync/rename.
- Failed downloads remove the partial file.
- Availability checks reject empty/obviously partial files.

### Config persistence

- Config writes use temp-write + sync + rename.
- Existing config is not truncated by a failed write.
- Tests cover failure-safe behavior where practical.

### Remote ASR response parsing

- Missing `text` field is an error.
- Non-string `text` field is an error.
- Empty string is allowed only if `text` exists and is a string.

### URL policy

- Only `http` and `https` are accepted.
- URL parsing uses the `url` crate or equivalent robust parser.
- Host must be present.
- Malformed hosts / control characters / whitespace inside the URL are rejected before browser execution.

### Settings secret display

- Failed secret masking/inspection is surfaced as a warning/error.
- “No API key configured” and “API key configured but unreadable” are visually and semantically distinct.
