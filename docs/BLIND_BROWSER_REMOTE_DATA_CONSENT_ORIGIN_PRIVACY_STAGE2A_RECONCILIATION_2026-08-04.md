# Blind Browser Remote Data Consent and Origin Privacy — Stage 2A Reconciliation

**Date:** 2026-08-04  
**Repository:** `ekkus93/blind_browser`  
**Branch:** `master`  
**Governing checklist:** `docs/BLIND_BROWSER_REMOTE_DATA_CONSENT_ORIGIN_PRIVACY_TODO_2026-08-03.md`  
**Stage 2A implementation SHA:** `8ef7f5710daa76061806692a37cc2a13b05710c8`  
**Status:** Backend consent transaction boundary implemented and validated; cleanup and formatting defects repaired during reconciliation; full milestone remains open.

## Purpose

The governing checklist still contains a Stage 1-only status statement and unchecked items in Sections 5–11. This document records which Stage 2A backend requirements are evidenced, which are only partially evidenced, and which remain open.

This is a bounded backend closeout. It does not declare the full remote-data-consent/origin-privacy milestone or the broader BBCR program complete.

## Exact implementation and validation evidence

- Stage 2A trigger/baseline SHA: `166814c048e5c11b9200243ea6cb7bbe23c9bd78`
- Published Stage 2A implementation SHA: `8ef7f5710daa76061806692a37cc2a13b05710c8`
- Stage 2A repair/validation run: `30954014288`
- Stage 2A repair/validation job: `92142680353`
- Repair job conclusion: `success`
- Permanent CI run on trigger SHA: `30954014221`
- Permanent CI job on trigger SHA: `92142681677`
- Permanent CI conclusion: `success`
- Initial reconciliation commit: `3c0709e6e84801ab22beec2751889f5f17ef9ab2`
- Initial implementation-report correction: `b074db0ae4d31cbe505f614db678772687ac15b1`

Job `92142680353` generated the Stage 2A backend tree, validated the checks included in that temporary job, and committed it as `8ef7f5710daa76061806692a37cc2a13b05710c8`.

Because that implementation commit was authored and pushed by GitHub Actions, GitHub did not start a separate workflow on the output SHA. The successful generating job is implementation evidence, but later permanent CI exposed that its validation sequence had omitted the repository's Rust-formatting gate. The original Stage 2A job therefore must not be represented as complete permanent-CI equivalence.

## Cleanup defect discovered during reconciliation

The initial reconciliation incorrectly accepted the repair job's cleanup output as proof that every temporary Stage 2A file had been removed. The first later human-authored documentation push exposed a remaining workflow:

- stale workflow: `.github/workflows/remote-data-consent-stage2a-v2-guard-fix2.yml`
- stale trigger: `.github/remote-data-consent-stage2a-v2-guard-fix2.trigger`
- failed workflow run exposing the defect: `30956038519`
- workflow cleanup commit: `8c51835a2ba60e2b96c99217497f955614dbf653`
- trigger cleanup commit: `d9274f69f8feb76c780f29382d86c2aa4edcf35f`

After `d9274f69f8feb76c780f29382d86c2aa4edcf35f`, the `.github/workflows` directory contains only the permanent `ci.yml`, `publish-ci-status.yml`, and `ralph-loop-apply.yml` workflows. The `.github` root contains no Stage 2A trigger, payload, repair script, or helper file.

Therefore, temporary Stage 2A machinery is now removed, but that cleanup was completed by the reconciliation commits—not solely by `8ef7f5710daa76061806692a37cc2a13b05710c8`.

## Formatting defect discovered by permanent CI

The first clean permanent-CI run on the reconciled tree exposed an unformatted expression in `src-tauri/src/app_core/remote_planner.rs`:

- failed reconciliation SHA: `53bf88bf68164e655ef6dd4b9eba3472e9a45cad`
- failed permanent CI run: `30956246911`
- failed permanent CI job: `92149918326`
- failing gate: `cargo fmt --manifest-path src-tauri/Cargo.toml --check`
- formatter-only repair commit: `aa5136b99de6e10f42547a3de699ecea5b9773db`

The repair applies exactly the stable `rustfmt` transformation to the Ollama credential-resolution expression. It changes no behavior, policy, authorization, credential scope, error mapping, or network logic.

This failure proves that the temporary Stage 2A repair workflow did not execute the exact permanent repository validation sequence even though it passed compilation, strict Clippy, tests, scanners, and frontend checks. Permanent CI remains authoritative.

## Validation recorded by job 92142680353

The Stage 2A job passed:

- security fallback scanner self-test and audit;
- security fallback inventory self-test and audit;
- sensitive diagnostics scanner self-test and audit;
- `cargo check`;
- strict all-target/all-feature Clippy with warnings denied;
- five focused `remote_data_consent` tests;
- all-feature Rust tests: 483 passed, 0 failed;
- hostile-content corpus: 4 passed, 0 failed;
- post-Batch-8 direct-command policy evidence: 6 passed, 0 failed;
- frontend lint;
- frontend UI tests: 2 passed, 0 failed;
- frontend production build.

The job did not run the permanent Rust-formatting gate. Its results validate the substantive backend implementation but do not replace permanent CI or the checklist's still-open request-count, replay, concurrency, invalidation, serialization-privacy, accessibility, and adversarial integration tests.

## Section 5 — Runtime-only ephemeral grants

### Evidenced complete

- `EphemeralConsentKind::Once` and `EphemeralConsentKind::Session` exist.
- `RemotePlannerEphemeralGrant` binds page origin, endpoint scope, policy version, expiry, and one-shot challenge digest.
- One-shot use is consumed through an atomic remaining-use counter.
- Grants are held only in runtime `AppCore`, not serializable `AppState` or persisted configuration.
- Expired grants are pruned before evaluation.
- Grant count is bounded to 64.
- Matching session grants are deduplicated.
- Privacy and planner-connection changes clear grants and pending consent.
- A reconstructed `AppCore` naturally starts without grants.

### Open evidence

- Complete lifecycle, reconstructed-core, repeated-session, expiry, endpoint/mode invalidation, and concurrent-consumption tests remain open.
- Dedicated diagnostic-formatting coverage for challenge digests remains open.

## Section 6 — Prepared request boundary

### Evidenced complete

- `prepare_remote_planner_request` validates destination scope before authorization.
- Privacy evaluation occurs before remote serialization or network I/O.
- Sanitization, disclosure classes/counts, sanitized byte count, and payload digest are calculated during request drafting.
- Preparation returns an authorized prepared request, a typed consent requirement, or a terminal typed block.
- Preparation performs no network I/O.
- `PreparedRemotePlannerRequest` contains sanitized input, exact endpoint scope, profile snapshot, typed authorization, origin, and runtime-state binding.
- The network sender accepts only `PreparedRemotePlannerRequest`, not raw `PlannerInput`.
- Endpoint-bound credentials, redirect refusal, timeout, parsing, semantic validation, and safety validation remain enforced.
- Typed resolution and bounded replanning both use this boundary.
- The `AppCore` lock is released before network I/O.

### Open evidence

- Dedicated request-count tests remain open.
- A compile-fail or equivalent structural test proving raw `PlannerInput` cannot reach the sender remains open.

## Section 7 — Disclosure and challenge contracts

### Evidenced complete

- Disclosure classes cover transcript, page origin, selected regions, selected element metadata, OCR-derived regions, tool-observation summaries, skill summaries, and trusted runtime contracts.
- Disclosure ordering is deterministic.
- Counts include region/element/OCR/tool/skill cardinalities and sanitized serialized bytes without content excerpts.
- The challenge contains random challenge/request IDs, digest, normalized origin, endpoint display/scope, profile/model, policy version, disclosure summary, expiry, and available-decision flags.
- The public challenge excludes sanitized request content and raw page/transcript/OCR/tool/skill payloads.
- Canonical hashing binds IDs, origin, endpoint, profile/model, policy version, disclosure summary, payload digest, runtime-state token, and expiry.

### Open evidence

- Per-field challenge-digest mutation tests remain open.
- Dedicated challenge-JSON raw-data exclusion tests remain open.

## Section 8 — Runtime-only pending consent

### Evidenced complete

- `PendingRemotePlannerConsent` lives only in `AppCore`, outside serializable `AppState`.
- It stores a public challenge, sanitized draft, runtime binding, profile/destination snapshot, planning snapshot, and continuation.
- It does not store unrestricted raw planner input.
- Storage is bounded to one pending request; replacement drops the prior request.
- Challenge lifetime is 120 seconds.
- Response handling atomically removes pending state through `Option::take`.
- Denial, invalid response, expiry, state mismatch, destination/policy change, authorization, and send initiation leave no reusable pending request.
- Privacy and endpoint changes clear pending state.

### Open evidence

- Serialized-state, `Debug`, runtime-status, frontend-state, and scanner tests remain open.
- A typed safe pending-challenge status summary remains Stage 2B work.

## Section 9 — Consent-required outcomes

### Evidenced complete

- `ExecutionOutcome::NeedsRemoteDataConsent` and `ResolveCommandOutcome::NeedsRemoteDataConsent` exist.
- Rust exhaustive matches were updated.
- Consent-required outcomes occur before network I/O.
- Direct command matches do not create remote-data challenges.
- Local-only, high-risk, opaque-origin, and persistent-block results remain terminal blocks.
- Typed resolve and voice/bounded execution use the same backend consent mechanism.
- Consent cannot authorize or reduce protected-action confirmation; planner output still passes the existing safety boundary.

### Open

- TypeScript types, frontend exhaustive handling, typed-command UI, and voice UI remain open.

## Section 10 — Consent-response command

### Evidenced complete

- `submit_remote_planner_consent_response` is a registered Tauri command and direct-command-policy entry.
- Policy evidence classifies it as user-gesture-required and as runtime/config-mutating credential-bearing network behavior with sanitized page context.
- Stable decisions exist for allow once, allow session, allow persistent, block persistent, and deny.
- Unknown serialized decisions are rejected by enum deserialization.
- Response handling validates pending existence, challenge ID/digest, expiry, runtime state, current profile/base URL/model, current privacy policy, and terminal block precedence.
- Deny and persistent block perform no network I/O for the pending request.
- Persistent writes complete before authorization.
- Persistence failure returns `remote_data_consent_persist_failed`, installs no authorization, and cannot send.
- Once/session/persistent allows are bound to the pending request and exact scope.
- Successful allow resumes the exact sanitized request.
- Pending state is consumed under lock, the prepared request is moved out, and network I/O runs after lock release.
- Planner output is revalidated before resolve or execution completion.

### Open evidence

- Comprehensive wrong-ID, wrong-digest, replay, duplicate-response, persistence-failure, exact-request-count, and unlocked-network-wait integration tests remain open.

## Section 11 — Runtime-state invalidation

### Evidenced implementation

- Challenge and draft bind the runtime-state token.
- Response rejects changed runtime state.
- Current profile name, base URL, and model must still match.
- Privacy and planner settings changes clear grants and pending consent.
- Current privacy policy is re-evaluated before applying an allow.
- Policy-version matching is required for grants and persistent allows.

### Open evidence

- The complete navigation/document/origin/endpoint/policy/block/expiry invalidation matrix remains unproven by focused tests.
- Unrelated read-only state behavior still requires a documented contract and tests.

## Remaining milestone boundary

The following remain open and must not be inferred complete from Stage 2A:

- typed runtime privacy status and settings APIs;
- TypeScript contracts and Tauri wrappers;
- safe frontend challenge state and exhaustive outcome handling;
- accessible just-in-time consent dialog;
- structured origin-rule and session-grant management;
- high-risk and opaque-origin presentation;
- complete Rust/frontend/request-count/replay/concurrency/invalidation/accessibility/adversarial tests;
- sensitive-diagnostics and frontend-state scanner extensions;
- focused permanent-CI consent target;
- privacy, migration, disclosure, revocation, and threat-model documentation;
- BBCR and post-Batch-8 reconciliation;
- exact-final-SHA permanent CI and final milestone signoff.

## Correct bounded conclusion

Stage 2A is complete as a backend implementation stage after reconciliation repairs: the runtime grant model, prepared-request-only sender boundary, disclosure/challenge contracts, pending transaction, consent-required outcomes, response command, exact request resume, and lock-safe orchestration are present. The stale temporary workflow/trigger and formatting defect found by later permanent validation have been repaired.

The full remote-data-consent/origin-privacy milestone remains open. The next bounded stage is Stage 2B: typed runtime privacy status/settings APIs, TypeScript contracts, safe frontend state, and accessible consent UX, followed by the missing test/scanner/documentation/final-signoff work.
