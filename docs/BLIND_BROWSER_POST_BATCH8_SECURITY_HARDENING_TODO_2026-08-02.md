# Blind Browser Post-Batch-8 Security Hardening TODO

**Date:** 2026-08-02  
**Repository:** `ekkus93/blind_browser`  
**Target branch:** `master`  
**Companion spec:** `docs/BLIND_BROWSER_POST_BATCH8_SECURITY_HARDENING_SPEC_2026-08-02.md`  
**Status:** Partially complete — audited against `master` implementation baseline `e8364ded5d9c017e8513640d6762d6f348d3d5e7`; unchecked items remain unproven or incomplete.  
**Completion rule:** No release-readiness or comprehensive-security-completion claim until every checkbox in this TODO is complete and permanent CI passes on the exact final `master` SHA.

> **2026-08-02 checkbox reconciliation:** Every task and subtask below was checked against the current source, tests, scanners, CI workflow, and supporting reports. A checked item has direct implementation or evidence in the repository. An unchecked item identifies a concrete evidence, documentation, or final-validation gap; unchecked items were not marked complete merely because adjacent code exists.

---

## 0. Operating rules

- [x] Work from latest `master`.
- [x] Do not weaken existing Batch 1-8 planner, confirmation, credential, privacy, or diagnostic guards.
- [x] Prefer typed errors over quiet fallbacks.
- [x] Do not add new network calls, side effects, or persisted settings without explicit tests.
- [x] Do not claim this TODO is complete until the permanent CI workflow passes on the final exact SHA.
- [x] Keep any temporary bounded workflow, script, or trigger out of the final production tree unless it is intentionally promoted to permanent CI.

---

## P8-001 — Model-download integrity and network hardening

### P8-001.1 Inventory current model download behavior

- [x] Inspect `src-tauri/src/app_core/model_management.rs`.
- [x] List every known TTS download plan.
- [x] List every known ASR download plan.
- [x] Identify every URL assembled for Hugging Face downloads.
- [x] Identify all existing availability checks and their limitations.
- [x] Confirm no direct final-path writes exist.

### P8-001.2 Add verified model file metadata

- [x] Add a model file manifest type, for example `VerifiedModelFile`.
- [x] Add expected SHA-256 for every file in each known Kitten TTS plan.
- [x] Add expected SHA-256 for every file in each known Whisper ASR plan.
- [x] Add `min_bytes` for every file.
- [x] Add `max_bytes` where practical.
- [x] Make unknown model IDs fail closed unless integrity metadata is added.
- [x] Add tests that reject invalid manifest entries.

### P8-001.3 Harden model download client

- [x] Add a request timeout to model downloads.
- [x] Disable redirects with `reqwest::redirect::Policy::none()` or implement a strict allowlist.
- [x] If an allowlist is implemented, test every accepted redirect host.
- [x] Return a typed failure for redirect attempts.
- [x] Return a typed failure for timeout.
- [x] Return a typed failure for non-success status.
- [x] Avoid logging full response bodies.

### P8-001.4 Verify before replace

- [x] Download to `.part` only.
- [x] Compute SHA-256 on the downloaded bytes.
- [x] Check exact SHA-256 before replacing target file.
- [x] Check minimum size before replacing target file.
- [x] Check maximum size before replacing target file when configured.
- [x] Sync temporary file before replacement.
- [x] Atomically replace target only after verification succeeds.
- [x] Remove `.part` after every failure path.
- [x] Preserve the old target file after every failed verification path.

### P8-001.5 Add tests

- [x] Unit test successful verified file write.
- [x] Unit test hash mismatch rejects before replacement.
- [x] Unit test too-small file rejects before replacement.
- [x] Unit test too-large file rejects before replacement when max exists.
- [x] Unit test old target survives failed verification.
- [x] Unit test `.part` cleanup after failure.
- [x] Unit test redirect refusal or allowlist behavior.
- [x] Unit test timeout configuration without real network.
- [x] Extend `scripts/check-silent-fallbacks.sh` if needed to prevent direct final-path model writes.

---

## P8-002 — Direct non-planner command policy audit

### P8-002.1 Inventory Tauri command surface

- [x] Inspect `src-tauri/src/lib.rs` and copy every command in `tauri::generate_handler!` into a direct-command inventory.
- [x] Include command names for planner execution, confirmation, audio, model, provider, safety, URL, voice, and API-key handlers.
- [x] Identify commands that mutate runtime state.
- [x] Identify commands that mutate persisted config.
- [x] Identify commands that persist secrets.
- [x] Identify commands that perform network I/O.
- [x] Identify commands that download executable/model artifacts.
- [x] Identify commands that launch external programs.
- [x] Identify commands that can transmit page/OCR context.

### P8-002.2 Add direct-command policy registry

- [x] Add a `DirectCommandName` enum or equivalent registry.
- [x] Add `DirectCommandPolicy` metadata for every direct command.
- [x] Reuse existing `ActionClass` where applicable.
- [x] Include `requires_user_gesture` or equivalent UI-origin requirement.
- [x] Include `mutates_config`.
- [x] Include `persists_secret`.
- [x] Include `performs_network_io`.
- [x] Include `credential_bearing_network_io`.
- [x] Include `transmits_page_context`.
- [x] Include `downloads_executable_or_model_artifact`.
- [x] Add a test that fails when `tauri::generate_handler!` contains a command missing from the registry.

### P8-002.3 Harden `open_external_url`

- [x] Replace string-prefix `https://` validation with parsed `url::Url` validation.
- [x] Require `scheme == "https"`.
- [x] Require a host.
- [x] Reject control characters.
- [x] Reject username/password unless there is a strong reason to allow them.
- [x] Strip or reject fragments and query strings if they are not needed.
- [x] Normalize the URL before launching.
- [x] Add tests for non-HTTPS, missing host, control characters, malformed URLs, embedded credentials, and valid HTTPS.

### P8-002.4 Harden API-key persistence result handling

- [x] Replace post-persist `.unwrap_or_default()` API-key reference handling with an explicit invariant check.
- [x] Return a typed error if a successful persist does not produce a non-empty API-key reference.
- [x] Add tests for planner API-key reference invariant.
- [x] Add tests for TTS API-key reference invariant.
- [x] Add tests for ASR API-key reference invariant.

### P8-002.5 Side-effect parity tests

- [x] Test that every direct command marked side-effecting has a policy classification.
- [ ] Test that every direct command marked networked has a timeout/redirect policy documented or enforced.
  - **Open evidence gap:** Network policies are documented and individual clients have tests, but no exhaustive registry-driven test proves the contract for every command marked `performs_network_io`.
- [ ] Test that every direct command marked credential-bearing uses endpoint-bound credential resolution where applicable.
  - **Open evidence gap:** Endpoint-scoping tests exist, but no exhaustive registry-driven test connects every `credential_bearing_network_io` command to endpoint-bound resolution.
- [ ] Test that direct model downloads rely on P8-001 verified downloads.
  - **Open evidence gap:** The handlers route into the verified model-management implementation, but no direct-command parity test asserts this dependency.
- [ ] Test that no direct page-context transmission bypasses remote planner privacy settings.
  - **Open evidence gap:** Current planner paths call the privacy sanitizer, but no exhaustive direct-command test proves that all commands marked `transmits_page_context` must pass through it.

---

## P8-003 — Confirmation-summary fail-closed behavior

### P8-003.1 Inventory current summary behavior

- [x] Inspect `src-tauri/src/commands/confirmation_manifest.rs`.
- [x] Inspect `src-tauri/src/app_core/click_authorization.rs` submit/type/click annotations.
- [x] List every protected action summary path.
- [x] Identify every path that can fall back to generic wording.
- [x] Decide which degraded summary cases should abort and which should show an explicit warning.

### P8-003.2 Add degraded-summary metadata

- [x] Add an explicit representation for unavailable form label.
- [x] Add an explicit representation for unavailable destination.
- [x] Add an explicit representation for unavailable field inventory.
- [x] Add an explicit representation for omitted sensitive fields.
- [x] Make degraded-summary state affect the confirmation manifest digest.
- [x] Ensure degraded metadata does not serialize raw values.

### P8-003.3 Improve submit summaries

- [x] For full metadata, include safe form label, destination origin, and safe field labels.
- [x] For missing page model, include a warning that form identity and destination could not be verified.
- [x] For ambiguous form model, include a warning that the active form could not be uniquely identified.
- [x] For unknown destination, include a warning that destination could not be verified.
- [x] For omitted sensitive fields, state that sensitive/hidden fields may be omitted.
- [x] Never include field values.
- [x] Never downgrade from confirmation to no-confirmation because metadata is missing.

### P8-003.4 Improve type-then-submit summaries

- [x] Confirm type-then-submit still requires confirmation.
- [x] Summarize typed text only by character count.
- [x] Include safe field label when available.
- [x] Include degraded warning when field label is unavailable.
- [x] Ensure text value is not in manifest JSON, prompt text, logs, or serialized pending state.

### P8-003.5 Add tests

- [x] Submit with full metadata.
- [x] Submit with no page model.
- [x] Submit with page model but no unique form.
- [x] Submit with unknown destination.
- [x] Submit with sensitive fields omitted.
- [x] Type-then-submit redacts text and reports only length.
- [x] Degraded summary changes manifest digest.
- [x] Planner-authored confirmation text remains ignored.

---

## P8-004 — Silent-fallback audit expansion

### P8-004.1 Create accepted fallback inventory

- [x] Create `docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md`.
- [x] Inventory `.ok()` uses in security-sensitive Rust modules.
- [x] Inventory `.unwrap_or_default()` uses in security-sensitive Rust modules.
- [x] Inventory ignored `Result` values in security-sensitive Rust modules.
- [x] Inventory fallback strings/default values in frontend invoke/settings/error code.
- [ ] For each accepted fallback, document file, function, expression, justification, user visibility, side-effect impact, test coverage, and future replacement plan.
  - **Open documentation gap:** The document provides category-level evidence and the allowlist provides exact file/expression entries, but it does not provide every required field—including the containing function—for every individual allowlisted expression.

### P8-004.2 Convert unjustified fallbacks

- [x] Convert unsafe `.ok()` fallbacks to typed errors.
- [x] Convert unsafe `.unwrap_or_default()` fallbacks to typed errors.
- [x] Convert unsafe ignored `Result` values to handled errors.
- [ ] Convert unavoidable best-effort cleanup failures to explicit comments and tests.
  - **Open evidence gap:** Accepted cleanup fallbacks are documented and allowlisted, but explicit source comments and focused tests were not found for every remaining best-effort cleanup expression.
- [x] Convert capability-degrading fallbacks to user-visible warnings where appropriate.

### P8-004.3 Improve scanner coverage

- [x] Extend `scripts/check-silent-fallbacks.sh` or add a new scanner.
- [x] Flag suspicious `.ok()` uses in security-sensitive paths.
- [x] Flag suspicious `.unwrap_or_default()` uses in security-sensitive paths.
- [x] Flag `let _ =` ignored results in security-sensitive paths, except allowlisted cleanup.
- [x] Flag direct final-path model writes.
- [x] Flag unchecked diagnostic or serialization fallbacks.
- [x] Add an allowlist mechanism for reviewed fallbacks.
- [x] Add the new scanner to permanent CI.

### P8-004.4 Add tests

- [x] Test scanner fails on a synthetic unsafe `.ok()` pattern.
- [x] Test scanner permits a documented accepted fallback.
- [x] Test scanner fails on a synthetic direct model final-path write.
- [x] Test scanner fails on a synthetic API-key-reference `.unwrap_or_default()` pattern.

---

## P8-005 — Diagnostic and privacy audit completion

### P8-005.1 Backend diagnostic audit

- [x] Audit all Rust tracing/logging macro calls.
- [x] Audit all custom `Debug` implementations.
- [x] Audit all `derive(Debug)` on planner, command, state, config, and provider structs.
- [x] Audit `ToolError` construction sites for sensitive details.
- [x] Audit remote planner error handling.
- [x] Audit remote ASR error handling.
- [x] Audit remote TTS error handling.
- [x] Audit API-key testing/model-listing errors.
- [x] Audit model-download errors.
- [x] Audit panic/expect messages in security-sensitive code.

### P8-005.2 Frontend diagnostic audit

- [x] Audit `src/api/errors.ts`.
- [x] Audit `src/privacy-redaction.ts`.
- [x] Audit UI display of backend errors.
- [x] Audit settings panels for API-key/model/profile errors.
- [x] Audit tests for snapshots or thrown errors containing secrets.
- [x] Audit any console logging.
- [x] Audit Redux/state-like persistence if present.

### P8-005.3 Expand redaction tests

- [x] Nested JSON with sensitive key names.
- [x] Token-shaped strings.
- [x] URLs with username/password/query/fragment.
- [x] Remote HTTP error with sensitive response body.
- [x] Frontend `Error.message` containing credential-like token.
- [x] Serialized `ToolError` with raw arguments in details.
- [x] Planner errors include safe provider/model/base URL metadata but not response body.
- [x] Model download errors expose file identity but not unrelated local sensitive paths.

### P8-005.4 Improve diagnostic scanner

- [x] Add sensitive names found during audit to `scripts/check-sensitive-diagnostics.py`.
- [x] Detect diagnostic logging across multiline frontend expressions.
- [x] Detect accidental `Debug` derive on sensitive structs.
- [x] Detect raw planner input serialization in diagnostics.
- [x] Detect raw remote response body diagnostics.
- [x] Add scanner to permanent CI if not already present.

---

## P8-006 — Hidden DOM and OCR hostile-content corpus

### P8-006.1 Hidden DOM fixtures

- [x] Hidden input containing prompt injection.
- [x] Off-screen CSS text containing prompt injection.
- [x] `aria-label` prompt injection.
- [x] `title` or `alt` prompt injection.
- [x] `data-*` attribute prompt injection.
- [x] Script/style/comment injection.
- [x] Invisible overlay text.
- [x] Malicious form label text.
- [x] Confirmation-bypass instruction near real button.
- [x] Credential-exfiltration instruction near real input.

### P8-006.2 OCR hostile fixtures

- [x] OCR text saying “ignore previous instructions”.
- [x] OCR text impersonating system/developer messages.
- [x] OCR text asking to skip confirmation.
- [x] OCR text asking to reveal credentials.
- [x] OCR text near payment/receipt-like data.
- [x] OCR text mixed with benign page regions.
- [x] OCR text attempting to authorize click/submit.
- [x] At least one deterministic OCR-text fixture.
- [x] At least one real image fixture or documented reason why real image fixture is deferred.

### P8-006.3 Required hostile-content invariants

- [x] Hostile DOM content can only increase caution, redact, block, require confirmation, abort, or replan.
- [x] Hostile OCR content can only increase caution, redact, block, require confirmation, abort, or replan.
- [x] Hostile content cannot lower confirmation requirements.
- [x] Hostile content cannot create a runtime click authorization token.
- [x] Hostile content cannot mark a destructive click safe.
- [x] Hostile content cannot bypass high-risk origin policy.
- [x] Hostile content cannot appear in trusted runtime prompt sections.

### P8-006.4 Tests

- [x] Add unit tests for hidden DOM sanitization.
- [x] Add unit tests for OCR sanitization.
- [x] Add tests that prompt-injection indicators remain caution-only.
- [x] Add tests that malicious content cannot authorize a submit.
- [ ] Add tests that malicious content cannot authorize a click.
  - **Open evidence gap:** Runtime click authorization is deterministic and page content cannot directly mint a token, but no focused hostile-content regression test was found for this exact condition.
- [ ] Add tests that high-risk OCR/page context blocks network remote planning.
  - **Open evidence gap:** Sensitive form controls and high-risk URL paths are tested as network blocks, and hostile OCR is tested as untrusted/caution-only, but no focused test proves that high-risk OCR content itself triggers the network block.

---

## P8-007 — Implementation-report reconciliation

### P8-007.1 Create reconciliation addendum

- [x] Create `docs/BLIND_BROWSER_POST_BATCH8_RECONCILIATION_2026-08-02.md`.
- [x] Do not rewrite old reports to pretend they were complete earlier.
- [x] Reference `docs/BLIND_BROWSER_COMPREHENSIVE_CODE_REVIEW_FIX_IMPLEMENTATION_REPORT_2026-08-01.md`.
- [x] Reference this post-Batch-8 spec and TODO.

### P8-007.2 Reconcile BBCR status

For each BBCR item in the original comprehensive TODO:

- [x] Mark complete and validated, partially complete, open, superseded, or needs re-audit.
- [x] List relevant source files.
- [x] List relevant tests.
- [ ] List relevant commits or final master SHA.
  - **Open evidence gap:** The addendum delegates historical commit evidence elsewhere and still contains stale branch/PR wording; it does not record the final direct-`master` implementation SHA.
- [x] List remaining risk.
- [x] Decide whether it belongs in the next batch.

### P8-007.3 Reconcile Batch 7/8 confusion

- [x] Explicitly list Batch 7 residuals that Batch 8 later closed.
- [x] Explicitly list Batch 7 residuals still open.
- [x] Explicitly list Batch 8 tasks that are complete.
- [x] Explicitly list post-Batch-8 tasks moved into this TODO.
- [x] State whether `docs/BLIND_BROWSER_COMPREHENSIVE_CODE_REVIEW_FIX_IMPLEMENTATION_REPORT_2026-08-01.md` is stale, and in what way.

---

## P8-008 — Documentation and developer handoff

- [x] Update or add implementation report for this hardening pass.
- [ ] Include exact final SHA.
  - **Open documentation gap:** The implementation report still says the final branch and final `master` SHA are pending.
- [ ] Include exact permanent CI run ID and job ID.
  - **Open documentation gap:** Successful run `30789617109` and job `91610265720` are not yet recorded in the implementation report.
- [ ] Include all changed files.
  - **Open documentation gap:** The report labels its inventory as a pre-report snapshot and does not include the later TypeScript narrowing fix or this reconciliation update.
- [x] Include tests added.
- [x] Include scanner changes.
- [x] Include accepted fallback inventory summary.
- [x] Include unresolved risks.
- [x] Include “not release-ready” statement if any item remains open.

---

## Validation gate

Run locally or in CI:

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

- [x] Existing permanent validation gate passes.
- [x] New fallback scanner, if separate, passes.
- [x] New model-integrity tests pass.
- [x] New direct-command policy tests pass.
- [x] New confirmation-summary degraded-metadata tests pass.
- [x] New diagnostic/privacy tests pass.
- [x] New hidden DOM/OCR hostile-content tests pass.
- [ ] Permanent GitHub Actions CI passes on exact final `master` SHA.
  - **Pending final validation:** The implementation baseline passed permanent CI, but this TODO reconciliation creates a new `master` SHA that must pass before this item can be checked.

---

## Completion checklist

This TODO is complete only when all are true:

- [x] P8-001 complete.
- [ ] P8-002 complete.
- [x] P8-003 complete.
- [ ] P8-004 complete.
- [x] P8-005 complete.
- [ ] P8-006 complete.
- [ ] P8-007 complete.
- [ ] P8-008 complete.
- [x] No temporary workflow/script/trigger remains unless intentionally promoted.
- [ ] Permanent CI passed on final exact SHA.
- [ ] Final implementation report states exactly what was completed and what remains open.

Do not mark this TODO done just because code compiles. The purpose of this pass is to eliminate quiet, unsafe, or unaudited behavior that can otherwise survive ordinary compilation and happy-path tests.
