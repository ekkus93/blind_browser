# Blind Browser Post-Batch-8 Diagnostic and Privacy Audit

**Date:** 2026-08-02  
**Repository:** `ekkus93/blind_browser`  
**Authoritative work item:** P8-005 in `docs/BLIND_BROWSER_POST_BATCH8_SECURITY_HARDENING_TODO_2026-08-02.md`  
**Audit branch:** `post-batch8-hardening-continuation`  
**Status:** Source audit complete; final exact-SHA CI evidence remains to be recorded in the implementation report.

## Audit standard

Diagnostics may retain bounded operational metadata needed to identify a failing subsystem, provider, model, endpoint origin/path scope, file identity, request ID, error class, or retryability. They must not retain or display:

- API keys, bearer tokens, JWTs, cookies, credentials, passwords, or keyring references;
- raw planner input, raw page models, DOM/form values, OCR text, transcripts, tool arguments, or tool results;
- remote response bodies;
- URL username/password, query strings, or fragments;
- unrelated local filesystem parent paths;
- raw frontend `Error` objects that can carry any of the preceding data.

A redaction failure may reduce diagnostic detail. It must never convert a failed protected operation into success or weaken confirmation or policy enforcement.

## Backend audit

### Tracing and logging

Reviewed the Rust diagnostic surface under `src-tauri/src`, including `tracing::*`, `log::*`, `println!`, `eprintln!`, and `dbg!` patterns.

Findings and disposition:

- No production diagnostic call intentionally serializes planner input, page models, OCR text, transcripts, tool arguments, tool results, API keys, authorization headers, cookies, or response bodies.
- Skill discovery previously logged full project/user skill paths. `src-tauri/src/commands/skill_loader.rs` now logs only the source class (`project`, `user`, or `bundled`), the leaf skill directory name, and the I/O error kind. A regression test proves parent directories such as a user home or project name are excluded.
- Browser/audio/runtime diagnostics retain bounded state or error-class information and do not include captured speech or page content.

### `Debug` implementations and derives

Reviewed custom `Debug` implementations and `derive(Debug)` on planner, command, state, config, provider, and API-key result structures.

- Sensitive planner input and API-key result types are scanner-protected from accidental `Debug` derivation.
- `ToolError` uses the centralized redacting serializer rather than an unrestricted derived serializer.
- Configuration/profile debugging is not used as a diagnostic serialization path for secrets; persisted credentials are represented by scoped references rather than raw values.

### `ToolError` construction

Reviewed error construction in planner execution, confirmation, provider, API-key, model-management, and command-handler paths.

- `ToolError.details` is recursively redacted by sensitive key name and token/URL content before crossing the Tauri boundary.
- Raw planner arguments are not serialized into confirmation or frontend-visible pending state.
- API-key persistence now fails with typed planner/TTS/ASR invariant errors when a successful keyring write does not produce a non-empty reference.

### Remote planner

`src-tauri/src/app_core/remote_planner.rs` now centralizes request-failure construction in `planner_request_failed_error`.

Allowed metadata:

- provider name;
- configured model name;
- normalized provider base URL.

Explicitly excluded:

- remote response body;
- authorization headers or API keys;
- serialized planner input;
- raw transport error text that might contain provider content.

A regression test serializes the resulting `ToolError` and verifies that only safe connection metadata remains.

### Remote ASR

`src-tauri/src/asr/remote.rs` reports bounded transport/parse failures and does not echo the parsed provider body when the expected `text` field is absent or malformed.

A regression test supplies a hostile response object containing an authorization header, token-shaped value, and `response_body` field, then verifies none appears in the error string.

### Remote TTS

Reviewed `src-tauri/src/tts/remote.rs` and its callers.

- HTTP status failures are converted to bounded status/transport errors.
- Provider response bodies are not copied into diagnostics.
- Credential resolution remains endpoint scoped and redirect refusal remains enforced by the shared provider-client policy.

### API-key tests and model listing

Reviewed planner/TTS/ASR key testing and planner model listing.

- Errors do not include entered API keys.
- Credential-bearing requests use endpoint-bound secret resolution, request timeouts, and redirect refusal.
- Changed-endpoint operations cannot attach the configured endpoint's secret or organization/project headers without an explicitly entered key for the displayed endpoint.

### Model downloads

Reviewed `src-tauri/src/app_core/model_management/{download,manifest,tests}.rs`.

- Download errors identify the manifest file but do not include unrelated local target parent paths.
- Redirect errors retain only the destination host.
- HTTP failures retain only status and file identity, not response bodies.
- Hash, size, sync, replacement, and cleanup failures are typed.
- A regression test creates a target below a private-looking local parent path and proves a hash-mismatch error exposes only `fixture.bin`.

### Panic and `expect` messages

Reviewed security-sensitive production `expect`/panic sites.

- The bundled-skill parser `expect` covers a compile-time asset and is paired with CI parsing tests; it contains no user data.
- Runtime network, credential, confirmation, model-integrity, and planner paths return typed failures rather than panicking with request or content data.

## Frontend audit

### Error classification

`src/api/errors.ts` classifies Tauri `ToolError` values and generic frontend errors only after privacy redaction.

- Generic `Error.message` values are passed through `redactDiagnosticText`.
- Structured backend errors are passed through `sanitizeToolError`.
- Unknown values receive bounded generic text rather than unrestricted object serialization.

### Redaction boundary

`src/privacy-redaction.ts` now covers:

- sensitive key names at arbitrary nesting depth;
- authorization/password/API-key/session markers;
- common provider-token prefixes;
- JWT-shaped values;
- URL username/password, query strings, and fragments;
- arrays and arbitrary nested objects.

The redaction result intentionally sacrifices detail when a string resembles a credential.

### Console logging

Reviewed production `console.debug/info/warn/error` calls.

- `src/voice-loop.ts` previously logged the raw caught confirmation-submission error. It now logs the output of `classifyInvokeFailure`.
- `src/panel-state-setters.ts` now logs a classified failure and builds user guidance from a sanitized HTTPS URL with credentials, query data, and fragment removed.
- Benign warning objects contain bounded confirmation IDs/state labels and do not include queued action arguments or page data.

The permanent diagnostic scanner rejects raw frontend `error`/`*Error` arguments unless the expression invokes an approved classifier/redactor.

### UI error display and settings panels

Reviewed global alerts, planner/provider settings, API-key test/model-list errors, ASR/TTS panels, model-management status, and confirmation errors.

- Backend error details reach UI state only through the redaction boundary.
- API-key values are not stored in persistent Redux-like application state; entered values remain transient form state and are cleared according to existing settings flows.
- External-link failure guidance never repeats URL credentials/query/fragment data.
- Confirmation pending state excludes raw queued steps and typed values.

### Tests and artifacts

Reviewed frontend tests and build artifacts for secret-bearing snapshots or thrown values.

- Added tests for token prefixes, JWTs, nested sensitive keys, sensitive response bodies, URL credentials/query/fragment data, serialized `ToolError`, frontend `Error.message`, and external-link alerts.
- Test fixtures use synthetic credentials only and assert their absence from outputs.
- No test intentionally snapshots or publishes a real secret.

## Scanner changes

`scripts/check-sensitive-diagnostics.py` now:

- parses multiline Rust and frontend diagnostic expressions;
- flags planner input, page model, transcript, API-key, authorization, cookie, response-body, raw-response, tool-result, and raw argument references;
- rejects raw frontend error-object logging unless an approved classifier/redactor is present;
- detects accidental `Debug` derives on named sensitive structures;
- verifies the centralized `ToolError` serializer and frontend error-redaction boundary remain present;
- provides `--self-test` hostile and benign fixtures.

Permanent CI runs both:

```text
python3 scripts/check-sensitive-diagnostics.py --self-test
python3 scripts/check-sensitive-diagnostics.py
```

The scanner is a regression tripwire, not a proof that arbitrary future diagnostic code is safe. New sensitive types and aliases must be added when introduced.

## Regression evidence inventory

| Requirement | Evidence |
|---|---|
| Nested sensitive JSON | `src/privacy-redaction.test.mjs` |
| Token-shaped and JWT strings | `src/privacy-redaction.test.mjs` |
| URL credentials/query/fragment | `src/privacy-redaction.test.mjs`, `src/external-link.test.mjs` |
| Remote response-body omission | `src-tauri/src/asr/remote.rs`, `src-tauri/src/app_core/remote_planner.rs` tests |
| Frontend `Error.message` credential | `src/privacy-redaction.test.mjs` |
| Serialized `ToolError` raw details | `src/privacy-redaction.test.mjs` and centralized Rust serializer tests |
| Planner safe provider/model/base URL metadata | `src-tauri/src/app_core/remote_planner.rs` test |
| Model error file identity without parent path | `src-tauri/src/app_core/model_management/tests.rs` |
| Local skill path privacy | `src-tauri/src/commands/skill_loader.rs` test |
| Scanner multiline/derive/raw-error detection | `scripts/check-sensitive-diagnostics.py --self-test` |

## Residual risks

- Regex/static scanning cannot establish semantic noninterference; code review remains required for new diagnostics.
- Third-party libraries may emit their own diagnostics outside these wrappers. Production logging configuration should remain conservative.
- Safe provider/model/base-URL metadata can still be operationally sensitive in some threat models, but it does not contain credentials, response content, page content, or local paths.
- A future persistent frontend telemetry or crash-reporting feature would require a separate explicit consent, retention, and redaction design before enablement.

No diagnostic/privacy completion or release-readiness claim is valid until permanent CI passes on the exact final repository SHA and the authoritative TODO and implementation report are reconciled.