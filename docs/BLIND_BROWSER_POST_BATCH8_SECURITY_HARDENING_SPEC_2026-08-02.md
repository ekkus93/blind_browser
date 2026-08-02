# Blind Browser Post-Batch-8 Security Hardening Spec

**Date:** 2026-08-02  
**Repository:** `ekkus93/blind_browser`  
**Target branch:** `master`  
**Authoritative prior TODO:** `docs/BLIND_BROWSER_COMPREHENSIVE_CODE_REVIEW_FIX_TODO_2026-08-01.md`  
**Prior implementation report:** `docs/BLIND_BROWSER_COMPREHENSIVE_CODE_REVIEW_FIX_IMPLEMENTATION_REPORT_2026-08-01.md`  
**Status:** New post-Batch-8 remediation scope. No release-readiness claim.

---

## 1. Purpose

Batch 1 through Batch 8 substantially hardened planner-mediated browser automation, remote-planner redaction, confirmation manifests, credential endpoint binding, runtime click authorization, and privacy settings. The remaining risk is no longer concentrated in one planner prompt or one obvious API-key leak. The remaining risk is distributed across:

1. model download supply-chain integrity;
2. direct non-planner Tauri command entry points;
3. degraded confirmation summaries;
4. narrow silent-fallback regression scanning;
5. incomplete diagnostic/privacy auditing;
6. hidden DOM and OCR prompt-injection fixtures;
7. stale status reporting in the implementation report.

This spec defines a bounded hardening pass for those remaining issues. It must not re-open already validated Batch 1-8 behavior unless the new work reveals a concrete bug.

---

## 2. Current security baseline

The current `master` already has several important protections that this pass must preserve:

- deterministic planner action policy in `src-tauri/src/commands/action_policy.rs`;
- executor-level refusal of prohibited and unconfirmed protected planner actions;
- runtime-owned click authorizations in `src-tauri/src/app_core/click_authorization.rs`;
- state-bound planning snapshots in `src-tauri/src/app_core/planning_snapshot.rs`;
- immutable confirmation manifests in `src-tauri/src/commands/confirmation_manifest.rs`;
- origin-bound remote credentials in `src-tauri/src/provider_endpoint.rs` and `src-tauri/src/config/keyring_store.rs`;
- no-redirect credential-bearing clients in `src-tauri/src/app_core/api_key_tools.rs`, remote ASR, remote TTS, and remote planner paths;
- typed remote-planner redaction boundary in `src-tauri/src/app_core/planner_redaction.rs`;
- backend and frontend diagnostic redaction in `src-tauri/src/diagnostic_redaction.rs`, `src-tauri/src/commands/contracts/mod.rs`, `src/api/errors.ts`, and `src/privacy-redaction.ts`;
- CI guards `scripts/check-silent-fallbacks.sh` and `scripts/check-sensitive-diagnostics.py`.

New work must keep these protections fail-closed. If a new feature cannot preserve the relevant guard, it must return a typed error instead of silently falling back.

---

## 3. Security principles for this pass

### 3.1 Fail closed before side effects

Any operation that can navigate, click, type, submit, mutate config, persist credentials, download executable/model artifacts, invoke external programs, or transmit page/OCR context must have an explicit policy decision before the side effect happens.

### 3.2 No quiet degradation in safety UI

If the app lacks enough runtime metadata to produce a trustworthy confirmation summary, it must say so explicitly. A generic confirmation prompt is acceptable only if it includes a clear degradation warning.

### 3.3 No unauthenticated model replacement

Atomic file replacement prevents partial files, not malicious or wrong files. Model downloads must verify expected bytes before replacing an existing model path.

### 3.4 Sanitization must be auditable

Privacy filters, fallback paths, and diagnostic redaction must leave enough safe evidence to debug failures without exposing secrets, page text, transcripts, OCR text, raw HTML, request bodies, credentials, or raw tool arguments.

### 3.5 Every accepted fallback must be named

A fallback is acceptable only when it is explicitly named, documented, tested, and cannot silently authorize a more dangerous action. Otherwise it must become an error.

---

## 4. Scope

This pass covers seven workstreams:

- **P8-001:** Model-download integrity and network hardening.
- **P8-002:** Direct non-planner command policy audit.
- **P8-003:** Confirmation-summary fail-closed behavior.
- **P8-004:** Silent-fallback audit expansion.
- **P8-005:** Diagnostic and privacy audit completion.
- **P8-006:** Hidden DOM and OCR hostile-content corpus.
- **P8-007:** Implementation-report reconciliation.

The prefix `P8` means post-Batch-8. It does not mean this is Batch 8 work. If this is implemented as a new Ralph Loop batch, it may become Batch 9.

---

## 5. Non-goals

This pass must not:

- redesign the planner schema from scratch;
- replace the remote planner provider implementation;
- add new browser automation capabilities beyond what is needed for safety checks;
- add cloud sync, accounts, or telemetry infrastructure;
- claim the whole project is release-ready;
- weaken existing Batch 1-8 guards to make tests easier.

---

## 6. P8-001 — Model-download integrity and network hardening

### 6.1 Problem

`src-tauri/src/app_core/model_management.rs` downloads model files from Hugging Face and atomically replaces local files. It checks only rough availability floors after the fact. It does not currently pin SHA-256 hashes, verify signatures, disable redirects, or set a request timeout for model download requests.

Atomic replacement protects users from partial files. It does not protect users from:

- wrong upstream file;
- compromised upstream file;
- CDN/proxy substitution;
- redirect to an unexpected host;
- truncated file above the minimum-size sanity floor;
- old file replaced by unverified new bytes.

### 6.2 Required behavior

Model downloads must use a manifest with explicit file metadata:

```rust
pub(crate) struct VerifiedModelFile {
    pub(crate) file_name: &'static str,
    pub(crate) sha256_hex: &'static str,
    pub(crate) min_bytes: u64,
    pub(crate) max_bytes: Option<u64>,
}
```

Each known local model download plan must declare all files with expected SHA-256 values. Unknown model IDs must remain unsupported unless their integrity metadata is added.

The download flow must be:

1. create parent directory;
2. download to a `.part` file;
3. compute SHA-256 while writing or immediately after writing;
4. verify expected hash and size bounds;
5. `sync_all` the file;
6. atomically replace the target;
7. optionally sync the parent directory where supported;
8. remove `.part` on failure;
9. return a typed error that identifies the model, file name, and failure class without leaking unrelated paths or secrets.

Credential-bearing API redirect policy already exists. Model downloads are not credential-bearing, but they still must not silently follow arbitrary redirects. The model client must either:

- use `redirect(Policy::none())`; or
- use a strict allowlist that permits only expected Hugging Face/CDN hosts and records the final host in safe diagnostics.

Default requirement: use `Policy::none()` unless a test proves Hugging Face requires a specific redirect and an allowlist is implemented.

All model download HTTP clients must set a request timeout.

### 6.3 Required errors

Add specific error strings or enum variants for:

- model download client build failure;
- request failure;
- redirect refusal;
- non-success status;
- response too small;
- response too large;
- hash mismatch;
- temp-file create/write/sync failure;
- atomic replace failure;
- invalid file manifest.

Do not collapse hash mismatch into a generic network error.

### 6.4 Required tests

Add tests for:

- successful hash verification using local fixture bytes;
- hash mismatch rejects before replacement;
- too-small file rejects before replacement;
- too-large file rejects before replacement when max is present;
- existing target survives failed verification;
- `.part` is removed after failure;
- direct final-path writes remain forbidden by `scripts/check-silent-fallbacks.sh`;
- redirect response is rejected or allowlisted explicitly;
- timeout is configured on the model download client.

Tests should not require real network access.

---

## 7. P8-002 — Direct non-planner command policy audit

### 7.1 Problem

Planner-mediated actions now pass through deterministic policy, validation, runtime state binding, confirmation manifests, and executor preflight. Direct Tauri invoke commands do not all pass through the same policy shape.

Direct commands are not automatically wrong. UI settings panels need direct commands. However, every direct command must have an explicit policy classification and a test proving its safety contract.

### 7.2 Required direct-command registry

Add a registry for Tauri command entry points, separate from planner `ToolName` if necessary:

```rust
pub(crate) enum DirectCommandName {
    ResolveCommand,
    ExecutePlannerOutput,
    SubmitConfirmationResponse,
    OpenUrl,
    OpenExternalUrl,
    SetRemotePlannerApiKey,
    DownloadActiveLocalTtsModel,
    DownloadActiveLocalAsrModel,
    // ...every command in tauri::generate_handler!
}

pub(crate) struct DirectCommandPolicy {
    pub(crate) class: ActionClass,
    pub(crate) requires_user_gesture: bool,
    pub(crate) mutates_config: bool,
    pub(crate) persists_secret: bool,
    pub(crate) performs_network_io: bool,
    pub(crate) transmits_page_context: bool,
    pub(crate) downloads_executable_or_model_artifact: bool,
}
```

The registry must be exhaustive for every command listed in `tauri::generate_handler!` in `src-tauri/src/lib.rs`. Adding a new command must fail a test until it is classified.

### 7.3 Required command-specific hardening

- `open_external_url` must parse URLs with `url::Url`, require HTTPS, require a host, reject control characters, normalize the URL, and avoid shell interpretation hazards.
- API-key persistence commands must fail if a successful persist does not produce an expected non-empty reference.
- Model download commands must be classified as model-artifact side effects and rely on P8-001 verification.
- Privacy, OCR, model, provider, and audio setting mutations must be classified as config mutations.
- Commands that perform network I/O must state whether they are credential-bearing and whether redirect following is allowed.
- Commands that transmit page/OCR context must be impossible to invoke without the privacy policy path used by the remote planner.

### 7.4 Required tests

Add tests that:

- enumerate all generated Tauri command names and compare them to the direct-command policy registry;
- prove `open_external_url` rejects non-HTTPS, missing host, control characters, and malformed URLs;
- prove API-key persistence returns an error if the resulting settings lack an API-key reference;
- prove no direct command marked as side-effecting is left with an unspecified policy;
- prove no direct command silently falls back from a protected failure to an apparently successful no-op.

---

## 8. P8-003 — Confirmation-summary fail-closed behavior

### 8.1 Problem

Confirmation manifests are deterministic and bound to action digests. However, user-facing summaries can still degrade when runtime metadata is missing. The most important case is form submission: if form identity, destination, or field inventory cannot be resolved, the prompt may become generic.

Generic confirmation is safer than no confirmation, but it can mislead the user into thinking the app knows what will happen.

### 8.2 Required behavior

For protected actions, confirmation summaries must include either:

- a specific safe summary; or
- an explicit degradation warning.

For form submission, the prompt must distinguish:

- known form label;
- unknown form label;
- known destination origin;
- unknown destination;
- known safe field inventory;
- unavailable field inventory;
- hidden/sensitive field inventory omitted.

The summary must never include field values. It may include safe field labels if they pass the existing sensitive-field filter.

Example acceptable degraded summary:

> Submit the active form. Warning: form identity and destination could not be verified from the current page model. Sensitive or hidden fields may be present.

### 8.3 Required fail-closed conditions

The implementation may choose either strict failure or warning-based confirmation for degraded summaries. However:

- unconfirmed execution must never proceed;
- degraded metadata must never be silently omitted;
- the manifest digest must bind the degraded-summary state;
- tests must verify the exact degraded prompt content.

### 8.4 Required tests

Add tests for:

- submit with full metadata;
- submit with no page model;
- submit with page model but no unique form;
- submit with unknown destination;
- submit with sensitive fields omitted;
- type-then-submit summary redacts text and reports only length;
- degraded summary state changes the confirmation manifest digest.

---

## 9. P8-004 — Silent-fallback audit expansion

### 9.1 Problem

`scripts/check-silent-fallbacks.sh` blocks exact known-bad patterns. It does not provide broad coverage for new `.ok()`, `.unwrap_or_default()`, ignored `Result`, or best-effort fallback patterns.

### 9.2 Required inventory

Create a reviewed fallback inventory document:

`docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md`

Each entry must include:

- file path and function;
- fallback expression or behavior;
- why the fallback is acceptable;
- whether it is user-visible;
- whether it can affect a side effect;
- whether it has a regression test;
- whether it should be converted to a typed error later.

### 9.3 Required code changes

For security-sensitive modules, every fallback must be one of:

- removed;
- converted to typed error;
- converted to explicit warning in `ToolResult`;
- documented in the accepted fallback inventory and covered by a test.

Security-sensitive modules include at least:

- `src-tauri/src/app_core/**`;
- `src-tauri/src/commands/**`;
- `src-tauri/src/config/**`;
- `src-tauri/src/asr/**`;
- `src-tauri/src/tts/**`;
- `src-tauri/src/ocr/**`;
- `src/api/**`;
- `src/*.ts` and `src/*.tsx` files involved in invoke/error/settings handling.

### 9.4 Required scanner improvement

Add a new script, or extend the existing scanner, to flag suspicious fallback forms in security-sensitive paths. It should not blindly fail the build for every `.ok()` or ignored result; instead, use a checked allowlist file, for example:

`docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md`

or a machine-readable companion file:

`scripts/accepted-fallbacks.toml`

CI must fail when a new suspicious fallback appears without an allowlist entry.

---

## 10. P8-005 — Diagnostic and privacy audit completion

### 10.1 Problem

Backend `ToolError` redaction and frontend invoke redaction are significantly improved, and CI now has a sensitive-diagnostics scanner. The scanner is intentionally narrow and does not prove that all Redux/UI state, logs, test artifacts, panic messages, network errors, or remote-response handling are safe.

### 10.2 Required audit coverage

Audit and harden:

- Rust tracing/logging macros;
- `Debug` derives on sensitive structs;
- serde serialization for planner input, pending state, tool errors, and runtime state;
- frontend invoke error handling;
- frontend state snapshots and test output;
- UI display of remote planner failures;
- remote planner response errors;
- remote ASR/TTS response errors;
- model download errors;
- API-key testing and model-listing errors;
- GitHub Actions logs and artifacts.

### 10.3 Required rule

No diagnostic output may include:

- API keys;
- authorization headers;
- cookies;
- raw transcripts;
- OCR text;
- raw page text;
- raw HTML;
- raw planner input;
- raw tool arguments;
- raw remote response body;
- local model filesystem paths when sent to remote planner or external diagnostics.

### 10.4 Required tests

Add tests for representative redaction of:

- nested JSON with sensitive key names;
- strings containing token-like values;
- URLs with username/password/query/fragment;
- remote HTTP error text with sensitive body;
- frontend `Error.message` containing a credential-like token;
- serialized `ToolError` with raw arguments in details;
- planner errors that include provider/model/base URL metadata but not response body.

---

## 11. P8-006 — Hidden DOM and OCR hostile-content corpus

### 11.1 Problem

The remote-planner boundary treats page/OCR/tool/skill text as untrusted and includes caution-only prompt-injection indicators. Existing tests cover many string fixtures. The remaining gap is a fuller corpus that better matches real hostile pages and OCR output.

### 11.2 Required hidden DOM fixtures

Add fixtures for:

- hidden inputs containing prompt injection;
- off-screen CSS text;
- aria-label injection;
- title/alt injection;
- `data-*` attribute injection;
- script/style/comment injection;
- invisible overlay text;
- malicious text in form labels;
- confirmation-bypass instructions near real buttons.

### 11.3 Required OCR fixtures

Add OCR fixture inputs that simulate:

- screenshot text saying “ignore previous instructions”;
- fake system/developer messages in page images;
- QR-code-adjacent malicious instruction text;
- receipt/payment-like text containing high-risk data;
- mixed benign and malicious OCR regions;
- OCR text that tries to authorize click/submit without confirmation.

Real image fixtures are preferred for at least a small subset. If real OCR is too slow or non-deterministic for unit tests, include deterministic OCR-text fixtures and one optional integration fixture gated separately.

### 11.4 Required invariant

Hostile content may only:

- increase caution;
- cause redaction;
- block remote planning for high-risk context;
- require confirmation;
- abort or replan.

Hostile content must never:

- authorize an action;
- lower confirmation requirements;
- mark a click as non-destructive;
- provide a runtime authorization token;
- bypass high-risk origin policy;
- appear as trusted runtime instructions.

---

## 12. P8-007 — Implementation-report reconciliation

### 12.1 Problem

`docs/BLIND_BROWSER_COMPREHENSIVE_CODE_REVIEW_FIX_IMPLEMENTATION_REPORT_2026-08-01.md` is stale. It correctly says no comprehensive completion claim is made, but some residual Batch 7 items were later implemented by Batch 8. This creates confusion when deciding what is actually still open.

### 12.2 Required reconciliation document

Create a new reconciliation addendum:

`docs/BLIND_BROWSER_POST_BATCH8_RECONCILIATION_2026-08-02.md`

It must list each BBCR item and classify it as:

- complete and validated;
- partially complete;
- open;
- superseded by later implementation;
- needs re-audit.

For every item, include:

- relevant commits or final `master` SHA;
- relevant source files;
- tests or CI evidence;
- remaining risk;
- whether it belongs in the next batch.

Do not rewrite history by editing old reports to imply they were complete at the time. Add a new dated addendum instead.

---

## 13. Required validation gate

The final implementation must pass the existing permanent gate:

```text
bash scripts/check-silent-fallbacks.sh
python3 scripts/check-sensitive-diagnostics.py
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
pnpm lint
pnpm test:ui
pnpm build
git diff --check
```

New work must add any new scripts to permanent CI. A bounded workflow may be used during implementation, but final completion requires the permanent repository CI to pass on the exact final `master` SHA.

---

## 14. Required evidence

The final implementation report for this hardening pass must record:

- final source commit SHA;
- exact permanent CI run ID and job ID;
- list of changed files;
- summary of each P8 workstream;
- every accepted fallback and why it remains acceptable;
- model download hash manifest and verification tests;
- direct command policy registry completeness test;
- diagnostic scanner results;
- hostile DOM/OCR corpus tests;
- explicit statement that this is or is not comprehensive security signoff.

---

## 15. Acceptance criteria

This pass is complete only when:

1. model downloads verify pinned hashes before replacing target files;
2. model download requests are timeout-bound and redirect-safe;
3. every direct Tauri command has a policy classification;
4. side-effecting direct commands cannot bypass the relevant safety policy unnoticed;
5. confirmation summaries explicitly disclose degraded metadata;
6. new suspicious fallbacks fail CI unless reviewed and allowlisted;
7. diagnostic redaction covers backend, frontend, and representative network/error paths;
8. hidden DOM and OCR hostile-content fixtures prove untrusted content cannot authorize action;
9. a reconciliation addendum resolves stale Batch 7/8 report status;
10. permanent CI passes on the exact final `master` SHA.

Until all criteria are met, do not claim release readiness or comprehensive security completion.
