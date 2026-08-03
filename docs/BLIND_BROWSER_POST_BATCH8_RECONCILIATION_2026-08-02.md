# Blind Browser Post-Batch-8 Reconciliation

**Date:** 2026-08-02  
**Repository:** `ekkus93/blind_browser`  
**Reviewed starting point:** `master` at `00d2e5c5cf5cf42f26dd73872fe58ddc7420ea6a`  
**Authoritative comprehensive TODO:** `docs/BLIND_BROWSER_COMPREHENSIVE_CODE_REVIEW_FIX_TODO_2026-08-01.md`  
**Prior implementation report:** `docs/BLIND_BROWSER_COMPREHENSIVE_CODE_REVIEW_FIX_IMPLEMENTATION_REPORT_2026-08-01.md`  
**Post-Batch-8 spec:** `docs/BLIND_BROWSER_POST_BATCH8_SECURITY_HARDENING_SPEC_2026-08-02.md`  
**Post-Batch-8 TODO:** `docs/BLIND_BROWSER_POST_BATCH8_SECURITY_HARDENING_TODO_2026-08-02.md`  
**Status:** Reconciled; the broader comprehensive remediation remains open and no release-readiness claim is made.

## Purpose

This addendum distinguishes four states that the prior implementation report sometimes compressed together:

- **Complete:** the original BBCR acceptance criterion is satisfied and has implementation/test evidence.
- **Partially complete:** meaningful implementation is present, but at least one original task or acceptance criterion remains open.
- **Covered by post-Batch-8 hardening:** a previously open subtask was completed by the post-Batch-8 work, without implying that the entire BBCR item is complete.
- **Still open:** the post-Batch-8 scope did not close the original item.

The post-Batch-8 work primarily closes residual model-download integrity, direct-command policy, confirmation-summary degradation, diagnostic/privacy, fallback-audit, and hostile-content-corpus gaps. It does not silently convert the full P1/P2/P3 program into a completed release review.

## Item-by-item reconciliation

| BBCR item | Reconciled status | Post-Batch-8 delta | Remaining work / evidence boundary |
|---|---|---|---|
| **BBCR-001 — Deterministic action safety** | **Partially complete; post-Batch-8 closes the direct-command inventory/policy gap.** | `src-tauri/src/direct_command_policy.rs` exhaustively mirrors `tauri::generate_handler!`, classifies runtime/config/secret/network/page-context/model/external-launch effects, validates invariants at startup, and has parity tests. `docs/BLIND_BROWSER_DIRECT_COMMAND_POLICY_2026-08-02.md` records the contract. | The original comprehensive checklist still contains documentation granularity and broader “all side-effect entry points share the same policy” wording. Direct Tauri command parity is now evidenced, but future non-Tauri side-effect surfaces must continue to be added to the central policy model. `EvalJs` remains prohibited rather than constrained/retained. |
| **BBCR-002 — Immutable confirmation manifests** | **Core complete; post-Batch-8 closes the explicit degraded-summary gap.** | Confirmation actions now carry digest-bound structured warning codes for unavailable page model, form label/identity, destination, field inventory, omitted sensitive fields, and target labels. Submit/type summaries are user-visible, planner wording remains ignored, and typed values remain redacted. | Locator re-resolution remains action-specific: click targets are live-revalidated; submit form metadata is re-annotated from the current page model but a generic future protected locator framework does not yet exist. JavaScript confirmation summary is inapplicable because planner-generated `EvalJs` is prohibited. |
| **BBCR-003 — Remote planner redaction/privacy** | **Partially complete; post-Batch-8 closes the repository diagnostic/privacy audit subtask.** | Added the audit report, expanded recursive frontend redaction, removed raw frontend error logging, constrained provider/model error metadata, added remote-body omission tests, path-private skill/model diagnostics, and a self-testing permanent scanner. | Original explicit remote-transmission consent, per-origin opt-out/local-only UX, and high-risk-origin product policy remain open. Local relevance selection can be improved further. Therefore the full BBCR-003 acceptance criterion is not closed. |
| **BBCR-004 — Destination-bound credentials** | **Complete.** | Post-Batch-8 API-key setters now additionally fail closed if a successful persistence operation does not return a non-empty scoped key reference. | Continue applying `ProviderEndpointScope` to every future credential-bearing provider operation. |
| **BBCR-005 — Opaque contained image handles** | **Core complete; broader filesystem audit still open.** | Post-Batch-8 hostile OCR corpus references the existing real image fixture and preserves the opaque-handle/contained-cache boundary. Model and skill diagnostic paths were separately audited for disclosure. | Original repository-wide audit of all future export/import/configured paths, plus a true capture-to-Tesseract end-to-end assertion on supported packaged platforms, remains open. |
| **BBCR-006 — Hostile page/OCR prompt injection** | **Core complete; post-Batch-8 closes the named corpus gap.** | Added deterministic DOM/OCR corpus data for hidden/off-screen text, ARIA/title/alt/data attributes, comments/scripts/styles, overlays, malicious form labels, nearby button/input instructions, confirmation bypass, credential exfiltration, mixed OCR regions, high-risk receipt context, and OCR action-authority attempts. Manifest tests enforce required categories, monotonic-security invariants, allowed security effects, and presence of a real OCR image. Existing behavioral tests prove caution-only telemetry cannot authorize, lower confirmation, mark destructive clicks safe, bypass high-risk policy, or enter trusted prompt sections. | Maintain corpus coverage as new DOM/OCR channels are introduced; model behavior remains defense in depth and deterministic policy remains authoritative. |
| **BBCR-007 — Model download supply chain** | **Substantially implemented by post-Batch-8; original provenance/update UX remains partially open.** | Known Kitten/Whisper files use code-pinned immutable revisions, SHA-256, min/max sizes, bounded/redirect-controlled client behavior, streaming ceilings, `.part` writes, sync-before-replace, atomic activation, cleanup, and old-target preservation. Unknown models fail closed. Tests cover manifests, hash/size failure, redirect hosts, timeout configuration, cleanup, and path-private errors. | The original BBCR asks for persisted provenance/install time exposed in runtime status, an explicit user-facing update workflow, and fully transactional unique staging-directory activation for multi-file model directories. Current known plans verify each file before replacement, but the broader directory-level update/provenance product contract should remain open unless separately implemented. |
| **BBCR-008 — Planner timeout/cancellation** | **Partially complete from earlier batches; not closed by post-Batch-8.** | Direct-command policy documents planner network behavior; diagnostics avoid remote response bodies. | Need exact request-level cancellation semantics, bounded planner response body, error taxonomy, mutex/cancellation tests for both OpenAI- and Ollama-compatible paths. |
| **BBCR-009 — Strict remote endpoint policy** | **Complete for current provider operations.** | Existing `ProviderEndpointScope` remains authoritative; post-Batch-8 URL/API-key tests do not weaken it. | Add future provider schemes/operations only through this policy and its tests. |
| **BBCR-010 — Transactional config/keyring updates** | **Still open / partially implemented.** | Post-Batch-8 removes false-success API-key-reference defaults. | Full rollback/compensation, fault injection, in-memory/durable consistency, and orphan cleanup remain required. |
| **BBCR-011 — Crash-durable/concurrent-safe persistence** | **Still open / partially implemented.** | Verified model files are synced and atomically activated; config persistence already has some atomic-write behavior. | Unique temp files across all persistence, parent-directory fsync, cross-instance locking, platform guarantees, startup cleanup, and fault injection remain open. |
| **BBCR-012 — Resource and payload limits** | **Partially complete.** | Planner-safe page/OCR payloads and model downloads have important limits; screenshot cache and several provider paths have bounded behavior. | Centralized budgets and tests for every untrusted/remote byte stream, remote response sizes, concurrency/rate limits, and packaged-platform stress behavior remain open. |
| **BBCR-013 — Remove raw API-key drafts from global Redux state** | **Still open unless separately proven.** | Diagnostic redaction prevents entered keys from being echoed by errors/logs. | Must prove raw drafts never enter Redux actions/state, clear on every lifecycle path, production DevTools behavior, and accessible save/test UX. The diagnostic audit does not substitute for this product-state redesign. |
| **BBCR-014 — Tighten CSP/frontend network boundaries** | **Still open.** | Frontend external URL failures are sanitized; credential/provider traffic remains backend mediated. | Restrict production CSP/connect-src, add assertions, and prove arbitrary frontend fetch/XHR/WebSocket exfiltration is unavailable. |
| **BBCR-015 — Runtime state snapshot revalidation** | **Complete for current protected planner execution.** | Post-Batch-8 confirmation warning metadata is digest-bound and cannot weaken runtime-state validation. | Extend the invalidation matrix when future protected tools or runtime state dimensions are added. |
| **BBCR-016 — Automated secret scanning** | **Still open.** | Diagnostics/fallback scanners are not secret scanners. No exposed live API key was identified by the source audit performed here. | Add mandatory current-tree and history scanning, narrow synthetic-fixture baseline, repository push protection, historical/admin scan evidence, and incident response. |
| **BBCR-017 — `.gitignore` and local secret hygiene** | **Still open unless separately completed.** | No post-Batch-8 change claims completion. | Add forbidden secret filename patterns, safe examples, tracked-file CI checks, and documentation. |
| **BBCR-018 — Dependency/license/SAST gates** | **Still open.** | Permanent CI runs build/lint/test/security pattern scanners. | Add advisory/license/source policy, JS audit policy, CodeQL/SAST, immutable Action SHAs, and dependency update automation. |
| **BBCR-019 — Platform and packaged-app CI** | **Still open.** | Current permanent CI is Linux/Xvfb source validation. | Windows/macOS gates, packaged artifacts, platform path/keyring/persistence/browser-launch tests, packaged CSP/capability verification, and bounded smoke tests remain open. |
| **BBCR-020 — Fuzzing/property/mutation coverage** | **Still open, with deterministic adversarial corpora now stronger.** | Hostile DOM/OCR corpus and scanner self-tests improve adversarial regression coverage. | Add fuzzers, redaction/resource-limit properties, mutation testing, and measurable coverage expectations. |
| **BBCR-021 — Architecture/security documentation** | **Partially complete.** | Direct-command policy, accepted-fallback inventory, diagnostic/privacy audit, hostile-content corpus, and this reconciliation add required security documentation. | Update primary architecture/README/security reporting/threat model, explicit remote-data consent, model provenance/update process, resource limits, supported-platform persistence guarantees, and final operational guidance. |

## Source, tests, evidence, and next-batch index

The table below makes the evidence/disposition fields explicit for every BBCR item. Historical commit/run identifiers remain in the original TODO, batch evidence documents, prior implementation report, PRs, and issue #5. The final post-Batch-8 branch/master SHA and CI identifiers belong in the post-Batch-8 implementation report and PR #12 evidence because they do not exist until publication/merge.

| Item | Primary source / documentation | Tests or validation evidence | Residual risk and next-batch disposition |
|---|---|---|---|
| BBCR-001 | `commands/action_policy.rs`, `app_core/click_authorization.rs`, `app_core/tool_executor.rs`, `direct_command_policy.rs`, direct-command policy doc | security-policy, planner-output, click-authorization, executor defense-in-depth, direct-handler parity tests | Retain in the next comprehensive architecture/documentation batch for non-Tauri future surfaces and classification rationale maintenance. |
| BBCR-002 | `commands/confirmation_manifest.rs`, confirmation runtime state and response handlers | confirmation digest, replay, expiry, mutation, origin/page change, redaction, degraded-summary tests | Future protected locator classes must add equivalent live re-resolution and summary contracts. |
| BBCR-003 | planner-safe page/OCR types, `planner_redaction.rs`, privacy-redaction frontend, diagnostic audit doc | planner redaction/cap tests, OCR tests, frontend privacy tests, diagnostics scanner | Move consent, per-origin controls, local-only mode, and high-risk-origin UX into the next privacy product batch. |
| BBCR-004 | `provider_endpoint.rs`, config/keyring scoping, API-key handlers | endpoint normalization, changed host/scheme/port/path, redirect refusal, key-reference invariant tests | Maintenance-only unless a new provider or credential-bearing operation is added. |
| BBCR-005 | image registry/cache/OCR routing modules and Batch 6 evidence | traversal, separators, symlink, stale/cross-page, cleanup, real fixture presence | Broader filesystem identifiers and packaged capture-to-real-OCR E2E belong in the platform/filesystem batch. |
| BBCR-006 | trusted/untrusted planner payload types, `planner_redaction.rs`, hostile corpus JSON and manifest test | prompt-injection indicators, high-risk remote block, submit/click refusal, hostile observation, real-image OCR tests | Seed future fuzz/property corpus and extend for new extraction channels. |
| BBCR-007 | `model_management/{manifest,download,tests}.rs`, accepted fallback doc | manifest/hash/size/old-target/cleanup/redirect/timeout/path-privacy tests and silent-fallback scanner | Provenance/status/update UX and multi-file directory transaction move to model-management/persistence batch. |
| BBCR-008 | remote planner clients/orchestration | existing network and lock-release tests are incomplete for full original criterion | Keep open for planner cancellation/body-limit/error-taxonomy batch. |
| BBCR-009 | `provider_endpoint.rs` and provider client builders | endpoint policy and credential redirect tests | Maintenance-only; require policy extension for new endpoints. |
| BBCR-010 | API-key/config persistence code | key-reference invariant tests only close the false-success residual | Keep open for transaction/fault-injection/orphan-cleanup batch. |
| BBCR-011 | atomic config/model write primitives | model sync/replace/old-target tests; broader persistence suite incomplete | Keep open for durability/concurrency/platform semantics batch. |
| BBCR-012 | planner/OCR/resource caps, screenshot cache, model download ceilings | truncation/cache/model size tests; not every stream/concurrency path is covered | Keep open for centralized budgets and stress/concurrency batch. |
| BBCR-013 | frontend settings components/store/actions | diagnostic redaction tests do not prove secret-state absence | Keep open for ephemeral secret-input/state-lifecycle batch. |
| BBCR-014 | Tauri config/CSP and backend provider routing | no full arbitrary-frontend-network denial test | Keep open for CSP/capability hardening batch. |
| BBCR-015 | planning snapshot/state token/revalidation modules and Batch 5 evidence | navigation/page/safety change, read-only tolerance, confirmation replay/state-change tests | Maintenance-only; extend invalidation matrix for new protected tools. |
| BBCR-016 | none in this bounded pass | diagnostic/fallback scanners are explicitly not secret-history scanners | Keep open for Gitleaks/TruffleHog, push protection, current/history/admin evidence batch. |
| BBCR-017 | `.gitignore`, repository hygiene docs/CI | no completion evidence in this pass | Keep open with BBCR-016 secret-hygiene batch. |
| BBCR-018 | Cargo/pnpm lockfiles and current CI | build/lint/test gates only; no full advisory/license/SAST gate | Keep open for dependency/security automation batch. |
| BBCR-019 | Linux permanent CI | Xvfb source validation only | Keep open for Windows/macOS/package/smoke/platform semantics batch. |
| BBCR-020 | hostile corpus and scanner self-tests | deterministic adversarial cases; no fuzz/property/mutation gate | Keep open for fuzz/property/mutation batch. |
| BBCR-021 | direct-command, fallback, audit, reconciliation, implementation-report docs | documentation consistency checked by review/CI presence; primary docs remain incomplete | Keep open for primary SPECS/README/SECURITY/threat-model/operations batch. |

## Batch 7, Batch 8, and post-Batch-8 confusion resolved

### Batch 7 residuals later closed

- The repository-wide diagnostic/error/privacy audit residual under BBCR-003 is covered by P8-005 and its audit report, redaction tests, scanner self-test, and permanent CI integration.
- The named hidden-DOM and real-image OCR hostile-content corpus residual under BBCR-006 is covered by P8-006.
- Confirmation summaries now explicitly disclose unavailable form/destination/field metadata rather than silently relying on generic wording.

### Batch 7 residuals still open

- Explicit remote page-data consent, local-only/per-origin controls, and high-risk-origin product policy.
- Broader filesystem identifier audit and packaged capture-to-Tesseract E2E.
- Comprehensive P1/P2/P3 items outside the bounded Batch 7/8 safety core.

### Batch 8 work already complete at the starting SHA

- Deterministic planner/executor action policy and click authorization.
- Immutable confirmation ID/digest/replay/expiry/state binding.
- Destination-bound provider credentials and redirect refusal.
- Planner-safe page/OCR payload boundary and high-risk remote-planning controls.
- Opaque screenshot handles/cache containment.
- Permanent baseline CI, fallback scanner, and final Clippy cleanup at `00d2e5c5cf5cf42f26dd73872fe58ddc7420ea6a`.

### Work moved into the post-Batch-8 TODO

- Per-file model manifest/download integrity reconciliation.
- Direct Tauri command inventory/parity and strict external launcher URL validation.
- Planner/TTS/ASR post-persistence key-reference invariants.
- Structured degraded confirmation-summary metadata.
- Accepted fallback reconciliation and scanner self-tests.
- Backend/frontend diagnostic privacy audit and scanner expansion.
- Deterministic hostile hidden-DOM/OCR corpus.
- Full BBCR status reconciliation and implementation handoff.

### Status of the prior implementation report

`docs/BLIND_BROWSER_COMPREHENSIVE_CODE_REVIEW_FIX_IMPLEMENTATION_REPORT_2026-08-01.md` is retained as historical evidence but is stale as a current completion statement. It predates the post-Batch-8 defects and residual audit described above, and it does not reconcile the still-open BBCR P1/P2/P3 items with the bounded Batch 7/8 accomplishments. This addendum and the post-Batch-8 implementation report supersede it for current status; they do not rewrite or erase what the older report claimed at the time.

## Post-Batch-8 workstream mapping

| Post-Batch-8 workstream | Primary BBCR items affected | Reconciled result |
|---|---|---|
| P8-001 model integrity | BBCR-007, BBCR-011, BBCR-012 | Closes per-file integrity, bounded client, verified activation, and failure-path tests; broader provenance, directory transaction, and persistence program remain open. |
| P8-002 direct commands | BBCR-001, BBCR-004, BBCR-009 | Closes Tauri surface inventory/policy parity, strict external URL validation, and API-key reference invariants. |
| P8-003 confirmations | BBCR-002, BBCR-015 | Closes implicit generic-summary degradation by making warnings structured, visible, redacted, and digest-bound. |
| P8-004 fallback audit | Cross-cutting | Establishes a reviewed allowlist and permanent self-testing scanner; accepted entries may only reduce capability/optional detail. |
| P8-005 diagnostics/privacy | BBCR-003, BBCR-007, BBCR-013 | Closes the repository logging/error/response-body audit gap, but does not close remote-data-consent or Redux secret-draft redesign. |
| P8-006 hostile corpus | BBCR-006, BBCR-020 | Closes named DOM/OCR corpus gaps and real-image presence; fuzzing/mutation remains open. |
| P8-007 reconciliation | BBCR-001 through BBCR-021 | Prevents bounded Batch-8 completion from being misreported as full comprehensive remediation. |
| P8-008 handoff | BBCR-021 | Final implementation report must record exact branch/master SHA, CI run/job, changed files, tests, scanners, risks, and the non-release-ready boundary. |

## Correct release statement

The correct statement after successful exact-SHA permanent CI is:

> The post-Batch-8 hardening TODO is complete for its bounded scope. It closes verified model-file downloads, direct Tauri command policy parity, strict external URL validation, API-key reference invariants, explicit degraded confirmation summaries, fallback and diagnostic scanners, repository diagnostic/privacy audit gaps, and the named hostile DOM/OCR corpus. The broader comprehensive remediation remains open for the unreconciled BBCR P1/P2/P3 items listed above; therefore this is not a general production release-readiness or comprehensive-security-completion declaration.

Until permanent CI passes on the exact final `master` SHA, even that bounded completion statement remains provisional.