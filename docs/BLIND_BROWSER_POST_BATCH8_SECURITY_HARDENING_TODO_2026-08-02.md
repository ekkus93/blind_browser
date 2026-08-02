# Blind Browser Post-Batch-8 Security Hardening TODO

**Date:** 2026-08-02  
**Repository:** `ekkus93/blind_browser`  
**Target branch:** `master`  
**Companion spec:** `docs/BLIND_BROWSER_POST_BATCH8_SECURITY_HARDENING_SPEC_2026-08-02.md`  
**Status:** Open  
**Completion rule:** No release-readiness or comprehensive-security-completion claim until every checkbox in this TODO is complete and permanent CI passes on the exact final `master` SHA.

---

## 0. Operating rules

- [ ] Work from latest `master`.
- [ ] Do not weaken existing Batch 1-8 planner, confirmation, credential, privacy, or diagnostic guards.
- [ ] Prefer typed errors over quiet fallbacks.
- [ ] Do not add new network calls, side effects, or persisted settings without explicit tests.
- [ ] Do not claim this TODO is complete until the permanent CI workflow passes on the final exact SHA.
- [ ] Keep any temporary bounded workflow, script, or trigger out of the final production tree unless it is intentionally promoted to permanent CI.

---

## P8-001 — Model-download integrity and network hardening

### P8-001.1 Inventory current model download behavior

- [ ] Inspect `src-tauri/src/app_core/model_management.rs`.
- [ ] List every known TTS download plan.
- [ ] List every known ASR download plan.
- [ ] Identify every URL assembled for Hugging Face downloads.
- [ ] Identify all existing availability checks and their limitations.
- [ ] Confirm no direct final-path writes exist.

### P8-001.2 Add verified model file metadata

- [ ] Add a model file manifest type, for example `VerifiedModelFile`.
- [ ] Add expected SHA-256 for every file in each known Kitten TTS plan.
- [ ] Add expected SHA-256 for every file in each known Whisper ASR plan.
- [ ] Add `min_bytes` for every file.
- [ ] Add `max_bytes` where practical.
- [ ] Make unknown model IDs fail closed unless integrity metadata is added.
- [ ] Add tests that reject invalid manifest entries.

### P8-001.3 Harden model download client

- [ ] Add a request timeout to model downloads.
- [ ] Disable redirects with `reqwest::redirect::Policy::none()` or implement a strict allowlist.
- [ ] If an allowlist is implemented, test every accepted redirect host.
- [ ] Return a typed failure for redirect attempts.
- [ ] Return a typed failure for timeout.
- [ ] Return a typed failure for non-success status.
- [ ] Avoid logging full response bodies.

### P8-001.4 Verify before replace

- [ ] Download to `.part` only.
- [ ] Compute SHA-256 on the downloaded bytes.
- [ ] Check exact SHA-256 before replacing target file.
- [ ] Check minimum size before replacing target file.
- [ ] Check maximum size before replacing target file when configured.
- [ ] Sync temporary file before replacement.
- [ ] Atomically replace target only after verification succeeds.
- [ ] Remove `.part` after every failure path.
- [ ] Preserve the old target file after every failed verification path.

### P8-001.5 Add tests

- [ ] Unit test successful verified file write.
- [ ] Unit test hash mismatch rejects before replacement.
- [ ] Unit test too-small file rejects before replacement.
- [ ] Unit test too-large file rejects before replacement when max exists.
- [ ] Unit test old target survives failed verification.
- [ ] Unit test `.part` cleanup after failure.
- [ ] Unit test redirect refusal or allowlist behavior.
- [ ] Unit test timeout configuration without real network.
- [ ] Extend `scripts/check-silent-fallbacks.sh` if needed to prevent direct final-path model writes.

---

## P8-002 — Direct non-planner command policy audit

### P8-002.1 Inventory Tauri command surface

- [ ] Inspect `src-tauri/src/lib.rs` and copy every command in `tauri::generate_handler!` into a direct-command inventory.
- [ ] Include command names for planner execution, confirmation, audio, model, provider, safety, URL, voice, and API-key handlers.
- [ ] Identify commands that mutate runtime state.
- [ ] Identify commands that mutate persisted config.
- [ ] Identify commands that persist secrets.
- [ ] Identify commands that perform network I/O.
- [ ] Identify commands that download executable/model artifacts.
- [ ] Identify commands that launch external programs.
- [ ] Identify commands that can transmit page/OCR context.

### P8-002.2 Add direct-command policy registry

- [ ] Add a `DirectCommandName` enum or equivalent registry.
- [ ] Add `DirectCommandPolicy` metadata for every direct command.
- [ ] Reuse existing `ActionClass` where applicable.
- [ ] Include `requires_user_gesture` or equivalent UI-origin requirement.
- [ ] Include `mutates_config`.
- [ ] Include `persists_secret`.
- [ ] Include `performs_network_io`.
- [ ] Include `credential_bearing_network_io`.
- [ ] Include `transmits_page_context`.
- [ ] Include `downloads_executable_or_model_artifact`.
- [ ] Add a test that fails when `tauri::generate_handler!` contains a command missing from the registry.

### P8-002.3 Harden `open_external_url`

- [ ] Replace string-prefix `https://` validation with parsed `url::Url` validation.
- [ ] Require `scheme == "https"`.
- [ ] Require a host.
- [ ] Reject control characters.
- [ ] Reject username/password unless there is a strong reason to allow them.
- [ ] Strip or reject fragments and query strings if they are not needed.
- [ ] Normalize the URL before launching.
- [ ] Add tests for non-HTTPS, missing host, control characters, malformed URLs, embedded credentials, and valid HTTPS.

### P8-002.4 Harden API-key persistence result handling

- [ ] Replace post-persist `.unwrap_or_default()` API-key reference handling with an explicit invariant check.
- [ ] Return a typed error if a successful persist does not produce a non-empty API-key reference.
- [ ] Add tests for planner API-key reference invariant.
- [ ] Add tests for TTS API-key reference invariant.
- [ ] Add tests for ASR API-key reference invariant.

### P8-002.5 Side-effect parity tests

- [ ] Test that every direct command marked side-effecting has a policy classification.
- [ ] Test that every direct command marked networked has a timeout/redirect policy documented or enforced.
- [ ] Test that every direct command marked credential-bearing uses endpoint-bound credential resolution where applicable.
- [ ] Test that direct model downloads rely on P8-001 verified downloads.
- [ ] Test that no direct page-context transmission bypasses remote planner privacy settings.

---

## P8-003 — Confirmation-summary fail-closed behavior

### P8-003.1 Inventory current summary behavior

- [ ] Inspect `src-tauri/src/commands/confirmation_manifest.rs`.
- [ ] Inspect `src-tauri/src/app_core/click_authorization.rs` submit/type/click annotations.
- [ ] List every protected action summary path.
- [ ] Identify every path that can fall back to generic wording.
- [ ] Decide which degraded summary cases should abort and which should show an explicit warning.

### P8-003.2 Add degraded-summary metadata

- [ ] Add an explicit representation for unavailable form label.
- [ ] Add an explicit representation for unavailable destination.
- [ ] Add an explicit representation for unavailable field inventory.
- [ ] Add an explicit representation for omitted sensitive fields.
- [ ] Make degraded-summary state affect the confirmation manifest digest.
- [ ] Ensure degraded metadata does not serialize raw values.

### P8-003.3 Improve submit summaries

- [ ] For full metadata, include safe form label, destination origin, and safe field labels.
- [ ] For missing page model, include a warning that form identity and destination could not be verified.
- [ ] For ambiguous form model, include a warning that the active form could not be uniquely identified.
- [ ] For unknown destination, include a warning that destination could not be verified.
- [ ] For omitted sensitive fields, state that sensitive/hidden fields may be omitted.
- [ ] Never include field values.
- [ ] Never downgrade from confirmation to no-confirmation because metadata is missing.

### P8-003.4 Improve type-then-submit summaries

- [ ] Confirm type-then-submit still requires confirmation.
- [ ] Summarize typed text only by character count.
- [ ] Include safe field label when available.
- [ ] Include degraded warning when field label is unavailable.
- [ ] Ensure text value is not in manifest JSON, prompt text, logs, or serialized pending state.

### P8-003.5 Add tests

- [ ] Submit with full metadata.
- [ ] Submit with no page model.
- [ ] Submit with page model but no unique form.
- [ ] Submit with unknown destination.
- [ ] Submit with sensitive fields omitted.
- [ ] Type-then-submit redacts text and reports only length.
- [ ] Degraded summary changes manifest digest.
- [ ] Planner-authored confirmation text remains ignored.

---

## P8-004 — Silent-fallback audit expansion

### P8-004.1 Create accepted fallback inventory

- [ ] Create `docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md`.
- [ ] Inventory `.ok()` uses in security-sensitive Rust modules.
- [ ] Inventory `.unwrap_or_default()` uses in security-sensitive Rust modules.
- [ ] Inventory ignored `Result` values in security-sensitive Rust modules.
- [ ] Inventory fallback strings/default values in frontend invoke/settings/error code.
- [ ] For each accepted fallback, document file, function, expression, justification, user visibility, side-effect impact, test coverage, and future replacement plan.

### P8-004.2 Convert unjustified fallbacks

- [ ] Convert unsafe `.ok()` fallbacks to typed errors.
- [ ] Convert unsafe `.unwrap_or_default()` fallbacks to typed errors.
- [ ] Convert unsafe ignored `Result` values to handled errors.
- [ ] Convert unavoidable best-effort cleanup failures to explicit comments and tests.
- [ ] Convert capability-degrading fallbacks to user-visible warnings where appropriate.

### P8-004.3 Improve scanner coverage

- [ ] Extend `scripts/check-silent-fallbacks.sh` or add a new scanner.
- [ ] Flag suspicious `.ok()` uses in security-sensitive paths.
- [ ] Flag suspicious `.unwrap_or_default()` uses in security-sensitive paths.
- [ ] Flag `let _ =` ignored results in security-sensitive paths, except allowlisted cleanup.
- [ ] Flag direct final-path model writes.
- [ ] Flag unchecked diagnostic or serialization fallbacks.
- [ ] Add an allowlist mechanism for reviewed fallbacks.
- [ ] Add the new scanner to permanent CI.

### P8-004.4 Add tests

- [ ] Test scanner fails on a synthetic unsafe `.ok()` pattern.
- [ ] Test scanner permits a documented accepted fallback.
- [ ] Test scanner fails on a synthetic direct model final-path write.
- [ ] Test scanner fails on a synthetic API-key-reference `.unwrap_or_default()` pattern.

---

## P8-005 — Diagnostic and privacy audit completion

### P8-005.1 Backend diagnostic audit

- [ ] Audit all Rust tracing/logging macro calls.
- [ ] Audit all custom `Debug` implementations.
- [ ] Audit all `derive(Debug)` on planner, command, state, config, and provider structs.
- [ ] Audit `ToolError` construction sites for sensitive details.
- [ ] Audit remote planner error handling.
- [ ] Audit remote ASR error handling.
- [ ] Audit remote TTS error handling.
- [ ] Audit API-key testing/model-listing errors.
- [ ] Audit model-download errors.
- [ ] Audit panic/expect messages in security-sensitive code.

### P8-005.2 Frontend diagnostic audit

- [ ] Audit `src/api/errors.ts`.
- [ ] Audit `src/privacy-redaction.ts`.
- [ ] Audit UI display of backend errors.
- [ ] Audit settings panels for API-key/model/profile errors.
- [ ] Audit tests for snapshots or thrown errors containing secrets.
- [ ] Audit any console logging.
- [ ] Audit Redux/state-like persistence if present.

### P8-005.3 Expand redaction tests

- [ ] Nested JSON with sensitive key names.
- [ ] Token-shaped strings.
- [ ] URLs with username/password/query/fragment.
- [ ] Remote HTTP error with sensitive response body.
- [ ] Frontend `Error.message` containing credential-like token.
- [ ] Serialized `ToolError` with raw arguments in details.
- [ ] Planner errors include safe provider/model/base URL metadata but not response body.
- [ ] Model download errors expose file identity but not unrelated local sensitive paths.

### P8-005.4 Improve diagnostic scanner

- [ ] Add sensitive names found during audit to `scripts/check-sensitive-diagnostics.py`.
- [ ] Detect diagnostic logging across multiline frontend expressions.
- [ ] Detect accidental `Debug` derive on sensitive structs.
- [ ] Detect raw planner input serialization in diagnostics.
- [ ] Detect raw remote response body diagnostics.
- [ ] Add scanner to permanent CI if not already present.

---

## P8-006 — Hidden DOM and OCR hostile-content corpus

### P8-006.1 Hidden DOM fixtures

- [ ] Hidden input containing prompt injection.
- [ ] Off-screen CSS text containing prompt injection.
- [ ] `aria-label` prompt injection.
- [ ] `title` or `alt` prompt injection.
- [ ] `data-*` attribute prompt injection.
- [ ] Script/style/comment injection.
- [ ] Invisible overlay text.
- [ ] Malicious form label text.
- [ ] Confirmation-bypass instruction near real button.
- [ ] Credential-exfiltration instruction near real input.

### P8-006.2 OCR hostile fixtures

- [ ] OCR text saying “ignore previous instructions”.
- [ ] OCR text impersonating system/developer messages.
- [ ] OCR text asking to skip confirmation.
- [ ] OCR text asking to reveal credentials.
- [ ] OCR text near payment/receipt-like data.
- [ ] OCR text mixed with benign page regions.
- [ ] OCR text attempting to authorize click/submit.
- [ ] At least one deterministic OCR-text fixture.
- [ ] At least one real image fixture or documented reason why real image fixture is deferred.

### P8-006.3 Required hostile-content invariants

- [ ] Hostile DOM content can only increase caution, redact, block, require confirmation, abort, or replan.
- [ ] Hostile OCR content can only increase caution, redact, block, require confirmation, abort, or replan.
- [ ] Hostile content cannot lower confirmation requirements.
- [ ] Hostile content cannot create a runtime click authorization token.
- [ ] Hostile content cannot mark a destructive click safe.
- [ ] Hostile content cannot bypass high-risk origin policy.
- [ ] Hostile content cannot appear in trusted runtime prompt sections.

### P8-006.4 Tests

- [ ] Add unit tests for hidden DOM sanitization.
- [ ] Add unit tests for OCR sanitization.
- [ ] Add tests that prompt-injection indicators remain caution-only.
- [ ] Add tests that malicious content cannot authorize a submit.
- [ ] Add tests that malicious content cannot authorize a click.
- [ ] Add tests that high-risk OCR/page context blocks network remote planning.

---

## P8-007 — Implementation-report reconciliation

### P8-007.1 Create reconciliation addendum

- [ ] Create `docs/BLIND_BROWSER_POST_BATCH8_RECONCILIATION_2026-08-02.md`.
- [ ] Do not rewrite old reports to pretend they were complete earlier.
- [ ] Reference `docs/BLIND_BROWSER_COMPREHENSIVE_CODE_REVIEW_FIX_IMPLEMENTATION_REPORT_2026-08-01.md`.
- [ ] Reference this post-Batch-8 spec and TODO.

### P8-007.2 Reconcile BBCR status

For each BBCR item in the original comprehensive TODO:

- [ ] Mark complete and validated, partially complete, open, superseded, or needs re-audit.
- [ ] List relevant source files.
- [ ] List relevant tests.
- [ ] List relevant commits or final master SHA.
- [ ] List remaining risk.
- [ ] Decide whether it belongs in the next batch.

### P8-007.3 Reconcile Batch 7/8 confusion

- [ ] Explicitly list Batch 7 residuals that Batch 8 later closed.
- [ ] Explicitly list Batch 7 residuals still open.
- [ ] Explicitly list Batch 8 tasks that are complete.
- [ ] Explicitly list post-Batch-8 tasks moved into this TODO.
- [ ] State whether `docs/BLIND_BROWSER_COMPREHENSIVE_CODE_REVIEW_FIX_IMPLEMENTATION_REPORT_2026-08-01.md` is stale, and in what way.

---

## P8-008 — Documentation and developer handoff

- [ ] Update or add implementation report for this hardening pass.
- [ ] Include exact final SHA.
- [ ] Include exact permanent CI run ID and job ID.
- [ ] Include all changed files.
- [ ] Include tests added.
- [ ] Include scanner changes.
- [ ] Include accepted fallback inventory summary.
- [ ] Include unresolved risks.
- [ ] Include “not release-ready” statement if any item remains open.

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

- [ ] Existing permanent validation gate passes.
- [ ] New fallback scanner, if separate, passes.
- [ ] New model-integrity tests pass.
- [ ] New direct-command policy tests pass.
- [ ] New confirmation-summary degraded-metadata tests pass.
- [ ] New diagnostic/privacy tests pass.
- [ ] New hidden DOM/OCR hostile-content tests pass.
- [ ] Permanent GitHub Actions CI passes on exact final `master` SHA.

---

## Completion checklist

This TODO is complete only when all are true:

- [ ] P8-001 complete.
- [ ] P8-002 complete.
- [ ] P8-003 complete.
- [ ] P8-004 complete.
- [ ] P8-005 complete.
- [ ] P8-006 complete.
- [ ] P8-007 complete.
- [ ] P8-008 complete.
- [ ] No temporary workflow/script/trigger remains unless intentionally promoted.
- [ ] Permanent CI passed on final exact SHA.
- [ ] Final implementation report states exactly what was completed and what remains open.

Do not mark this TODO done just because code compiles. The purpose of this pass is to eliminate quiet, unsafe, or unaudited behavior that can otherwise survive ordinary compilation and happy-path tests.
