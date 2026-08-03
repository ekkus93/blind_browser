# Blind Browser Post-Batch-8 Security Hardening TODO

**Original date:** 2026-08-02  
**Closed:** 2026-08-03  
**Repository:** `ekkus93/blind_browser`  
**Branch:** `master`  
**Companion spec:** `docs/BLIND_BROWSER_POST_BATCH8_SECURITY_HARDENING_SPEC_2026-08-02.md`  
**Implementation report:** `docs/BLIND_BROWSER_POST_BATCH8_SECURITY_HARDENING_IMPLEMENTATION_REPORT_2026-08-02.md`  
**Reconciliation:** `docs/BLIND_BROWSER_POST_BATCH8_RECONCILIATION_2026-08-02.md`  
**Status:** Complete for the bounded post-Batch-8 security-hardening scope.  
**Release boundary:** The broader BBCR remediation remains open. Completion of this TODO is not a general production release-readiness or comprehensive-security-completion declaration.

## Final evidence

- **Starting SHA:** `00d2e5c5cf5cf42f26dd73872fe58ddc7420ea6a`
- **Generated source-evidence commit:** `a56c87f8597bbd53db90874658de71164d0c5005`
- **Ralph validation:** run `30836879809`, job `91764098763`, success
- **Cleaned implementation SHA:** `ad92e5a071784204cc55a370ffc23362de7dc54a`
- **Cleaned implementation permanent CI:** run `30837538008`, job `91766346922`, success
- **Finalized report/reconciliation candidate SHA:** `5c7fa17dea3ad5a44e890576f8973d8e982de5b5`
- **Candidate permanent CI:** run `30838272062`, job `91768801304`, success
- **Publication method:** direct commits to `master`; no branch, PR, or worktree is part of the final delivery
- **Temporary automation:** removed before cleaned-tree validation

A commit cannot embed its own SHA or the workflow run created after that commit exists. The exact final closure SHA and its `ci/permanent` result are therefore canonical GitHub commit metadata; the implementation report records the exact validated implementation and documentation-candidate evidence above.

This final form consolidates the completed checklist. The earlier line-by-line partial-status reconciliation remains available in Git history.

---

## 0. Operating rules

- [x] Work from latest `master`.
- [x] Do not weaken existing Batch 1-8 planner, confirmation, credential, privacy, or diagnostic guards.
- [x] Prefer typed errors over quiet fallbacks.
- [x] Do not add network calls, side effects, or persisted settings without explicit tests.
- [x] Require permanent CI on the exact validated implementation/documentation candidate.
- [x] Remove temporary bounded workflows, scripts, and triggers from the final production tree.

---

## P8-001 — Model-download integrity and network hardening

### Inventory and manifests

- [x] Inventory every known TTS and ASR download plan and assembled URL.
- [x] Define verified model-file metadata with immutable revision, SHA-256, and size bounds.
- [x] Add integrity metadata for every known Kitten TTS and Whisper ASR file.
- [x] Make unknown model IDs fail closed.
- [x] Reject invalid manifest entries in tests.

### Network and activation policy

- [x] Apply connect/request timeouts.
- [x] Refuse uncontrolled redirects and enforce the documented HTTPS host policy.
- [x] Return typed redirect, timeout, status, size, hash, sync, replacement, and cleanup failures.
- [x] Avoid exposing full response bodies or unrelated private paths.
- [x] Download only to a partial file.
- [x] Enforce streaming byte ceilings.
- [x] Verify exact hash and configured size bounds before activation.
- [x] Sync before atomic replacement.
- [x] Remove partial files on failure and preserve the previous target.

### Tests and enforcement

- [x] Test successful verified writes.
- [x] Test hash and size rejection before replacement.
- [x] Test previous-target preservation.
- [x] Test partial cleanup and explicit cleanup failure.
- [x] Test redirect/host policy and timeout configuration.
- [x] Test path-private model errors.
- [x] Test direct model handlers depend on verified activation.

---

## P8-002 — Direct non-planner command policy

### Inventory and registry

- [x] Inventory every command in `tauri::generate_handler!`.
- [x] Classify runtime/config mutation, secret persistence, network I/O, credential-bearing I/O, page-context transmission, model download, and external launch behavior.
- [x] Add exhaustive registry-to-handler parity testing.
- [x] Fail validation when a handler lacks policy metadata.

### Direct-command hardening

- [x] Parse and normalize external URLs with `url::Url`.
- [x] Require HTTPS and a host.
- [x] Reject malformed input, control characters, embedded credentials, query strings, and fragments.
- [x] Pass only normalized approved URLs to the OS opener.
- [x] Fail closed when API-key persistence does not return a non-empty scoped reference.

### Exhaustive side-effect evidence

- [x] Prove every networked direct command has timeout/redirect evidence.
- [x] Prove every credential-bearing direct command uses endpoint-bound credential resolution where applicable.
- [x] Prove direct model downloads use P8-001 verified activation.
- [x] Prove every page-context-transmitting direct command passes through remote-planner privacy sanitization.

---

## P8-003 — Confirmation-summary fail-closed behavior

- [x] Inventory every protected-action summary path and degradation case.
- [x] Add structured warning codes for unavailable page model, form label/identity, destination, field inventory, omitted sensitive fields, and target labels.
- [x] Bind warning metadata into the canonical confirmation digest.
- [x] Keep warning metadata free of raw sensitive values.
- [x] Include safe form labels, destination origins, and safe field labels when available.
- [x] Show explicit warnings for unavailable or ambiguous metadata.
- [x] Represent typed text only by character count.
- [x] Keep planner-authored confirmation wording non-authoritative.
- [x] Never lower confirmation because metadata is missing.
- [x] Test full and degraded submit/type paths, digest changes, and raw-value absence.

---

## P8-004 — Silent-fallback audit expansion

### Exact accepted-fallback inventory

- [x] Maintain `docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md`.
- [x] Maintain exact machine-readable entries in `scripts/security-fallback-inventory.json`.
- [x] Record path, containing function, exact expression, justification, user visibility, side-effect impact, tests/enforcement, and replacement plan for every accepted expression.
- [x] Keep the exact allowlist synchronized with live source.

### Conversion and enforcement

- [x] Convert unsafe `.ok()`, `.unwrap_or_default()`, and ignored-result fallbacks to typed handling.
- [x] Replace silent screenshot cleanup with explicit failure handling and tests.
- [x] Replace silent config temporary-file cleanup with explicit failure handling and tests.
- [x] Keep only reviewed capability-reducing or optional-detail fallbacks.
- [x] Run `check-security-fallbacks.py` and `check-security-fallback-inventory.py` self-tests and audits in permanent CI.

---

## P8-005 — Diagnostic and privacy audit

- [x] Audit backend logging, `Debug`, `ToolError`, provider, API-key, model, panic, and path diagnostics.
- [x] Audit frontend errors, redaction, settings surfaces, console logging, alerts, and state-like persistence.
- [x] Redact nested sensitive keys, credential/token shapes, URL userinfo/query/fragment data, remote response bodies, raw arguments, and generic error messages.
- [x] Keep safe provider/model/base-URL metadata without exposing bodies or credentials.
- [x] Keep model and skill diagnostics path-private.
- [x] Detect multiline sensitive logging, raw frontend error objects, sensitive `Debug` derives, raw planner input, and raw provider response bodies.
- [x] Run diagnostic scanner self-tests and repository scan in permanent CI.

---

## P8-006 — Hidden DOM and OCR hostile-content corpus

### Corpus coverage

- [x] Cover hidden inputs, off-screen text, ARIA/title/alt/data attributes, scripts/styles/comments, overlays, malicious labels, and nearby click/input instructions.
- [x] Cover OCR instruction override, authority impersonation, confirmation bypass, credential exfiltration, payment/receipt-like data, mixed regions, and action-authority attempts.
- [x] Maintain deterministic text fixtures and a real image fixture.

### Security invariants and tests

- [x] Hostile content can only increase caution, redact, block, require confirmation, abort, or replan.
- [x] Hostile content cannot lower confirmation requirements.
- [x] Hostile content cannot create click authorization or mark a destructive click safe.
- [x] Hostile content cannot bypass high-risk origin policy or enter trusted prompt sections.
- [x] Test hidden DOM and OCR sanitization.
- [x] Test prompt-injection indicators remain caution-only.
- [x] Test malicious content cannot authorize submit or click.
- [x] Test high-risk OCR/page context blocks network remote planning.

---

## P8-007 — Implementation-report reconciliation

- [x] Maintain a reconciliation document without rewriting historical reports as though they were complete earlier.
- [x] Reconcile every `BBCR-001` through `BBCR-021` item as complete, partially complete, or open.
- [x] Record relevant source/tests and the remaining risk boundary.
- [x] Record the final direct-`master` implementation SHA and permanent CI evidence.
- [x] Remove stale branch/PR workflow language.
- [x] Distinguish Batch 7 residuals, Batch 8 completion, and post-Batch-8 closure.
- [x] Identify the next comprehensive remediation scope.

---

## P8-008 — Documentation and developer handoff

- [x] Finalize the implementation report.
- [x] Include exact validated implementation and documentation-candidate SHAs.
- [x] Include exact Ralph and permanent CI run/job IDs.
- [x] Include the full changed-file inventory.
- [x] Include tests and scanner changes.
- [x] Include the exact accepted-fallback inventory summary.
- [x] Include unresolved risks.
- [x] State clearly that broader remediation remains open and the repository is not generally declared release-ready.

---

## Validation gate

The permanent workflow successfully ran:

```text
bash scripts/check-silent-fallbacks.sh
python3 scripts/check-security-fallbacks.py --self-test
python3 scripts/check-security-fallbacks.py
python3 scripts/check-security-fallback-inventory.py --self-test
python3 scripts/check-security-fallback-inventory.py
python3 scripts/check-sensitive-diagnostics.py --self-test
python3 scripts/check-sensitive-diagnostics.py
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
xvfb-run -a cargo test --manifest-path src-tauri/Cargo.toml --all-features
pnpm lint
pnpm test:ui
pnpm build
```

- [x] Silent-fallback scanner passes.
- [x] Reviewed fallback scanner and self-test pass.
- [x] Exact fallback inventory scanner and self-test pass.
- [x] Sensitive diagnostic scanner and self-test pass.
- [x] Rust formatting passes.
- [x] Default Rust check passes.
- [x] Deny-warning Clippy passes.
- [x] Full Rust test suite passes.
- [x] Frontend lint passes.
- [x] UI tests pass.
- [x] Frontend production build passes.
- [x] Permanent CI passes on the exact finalized report/reconciliation candidate SHA.

---

## Completion checklist

- [x] P8-001 complete.
- [x] P8-002 complete.
- [x] P8-003 complete.
- [x] P8-004 complete.
- [x] P8-005 complete.
- [x] P8-006 complete.
- [x] P8-007 complete.
- [x] P8-008 complete.
- [x] No temporary workflow, generator, trigger, branch, PR, or worktree is required by the final delivery.
- [x] Permanent CI passed on the cleaned implementation and finalized documentation candidate.
- [x] Final implementation report states exactly what was completed and what remains open.

## Remaining work outside this TODO

The following broader items remain open in the comprehensive BBCR program and are not regressions in this bounded pass:

- generic protected locator re-resolution;
- remote-data consent, per-origin/local-only controls, and high-risk-origin UX;
- model provenance/update UX and whole-directory transactions;
- planner cancellation and bounded response bodies;
- config/keyring rollback, durable consistency, locking, fsync, and fault injection;
- centralized resource budgets, concurrency/rate limits, and stress tests;
- raw API-key draft removal from global frontend state;
- production CSP/frontend network-boundary proof;
- secret-history scanning and push protection;
- dependency/license/SAST gates;
- cross-platform packaged CI;
- fuzzing/property/mutation coverage;
- primary architecture/security/privacy/provenance/resource/platform/operations documentation.

## Final bounded statement

> The post-Batch-8 security-hardening TODO is complete for its bounded scope. The broader comprehensive remediation remains open, so this is not a general production release-readiness or comprehensive-security-completion declaration.
