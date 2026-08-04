# Blind Browser Remote Data Consent and Origin Privacy — Stage 2A Reconciliation

**Date:** 2026-08-04  
**Repository:** `ekkus93/blind_browser`  
**Branch:** `master`  
**Governing checklist:** `docs/BLIND_BROWSER_REMOTE_DATA_CONSENT_ORIGIN_PRIVACY_TODO_2026-08-03.md`  
**Stage 2A implementation SHA:** `8ef7f5710daa76061806692a37cc2a13b05710c8`  
**Status:** Backend consent transaction boundary implemented and validated; full remote-data-consent/origin-privacy milestone remains open.

## Purpose

The governing checklist still contains a Stage 1-only status statement and many unchecked items in Sections 5–11. This reconciliation records which Stage 2A backend requirements are evidenced by source and validation on `master`, which requirements are only partially evidenced, and which remain open.

This document does not declare the complete remote-data-consent/origin-privacy milestone finished. Runtime status/settings APIs, TypeScript/frontend integration, accessible consent UI, structured rule management, comprehensive adversarial tests, scanner extensions, privacy documentation, BBCR reconciliation, and exact-final-SHA signoff remain open.

## Exact evidence

- Stage 2A trigger/baseline SHA: `166814c048e5c11b9200243ea6cb7bbe23c9bd78`
- Published Stage 2A implementation SHA: `8ef7f5710daa76061806692a37cc2a13b05710c8`
- Stage 2A repair and validation run: `30954014288`
- Stage 2A repair and validation job: `92142680353`
- Repair job conclusion: `success`
- Permanent CI run on the trigger SHA: `30954014221`
- Permanent CI job on the trigger SHA: `92142681677`
- Permanent CI conclusion: `success`

The successful repair job generated the final backend implementation, ran the complete recorded validation sequence against that generated tree, committed it as `8ef7f5710daa76061806692a37cc2a13b05710c8`, removed all temporary Stage 2A workflows/scripts/payloads/triggers, and pushed the result to `master`.

GitHub did not start a separate workflow for the bot-authored output commit. Therefore, there is no independent workflow run or commit status attached to `8ef7f5710daa76061806692a37cc2a13b05710c8`. The authoritative Stage 2A evidence is job `92142680353`, which validated the generated implementation immediately before creating and pushing that commit. This limitation must remain visible until a later human-authored documentation or implementation commit receives permanent CI on its exact SHA.

## Validation recorded by job 92142680353

The job recorded all of the following as successful:

- security fallback scanner self-test and audit;
- security fallback inventory self-test and audit;
- sensitive diagnostics scanner self-test and audit;
- `cargo check`;
- strict all-target/all-feature Clippy with warnings denied;
- five focused `remote_data_consent` tests;
- the complete all-feature Rust suite: 483 passed, 0 failed;
- hostile-content corpus: 4 passed, 0 failed;
- post-Batch-8 direct-command policy evidence: 6 passed, 0 failed;
- frontend lint;
- frontend UI tests: 2 passed, 0 failed;
- frontend production build;
- cleanup and absence checks for all temporary Stage 2A machinery.

The focused tests cover loopback precedence, local-only/high-risk precedence, persistent block precedence, destination/version-bound persistent allow behavior, and scoped/bounded session and one-shot grants. They do not constitute the full request-count, replay, concurrency, invalidation, serialization-privacy, or adversarial matrix required for final milestone closure.

## Section 5 — Runtime-only ephemeral grants

### Evidenced complete

- `EphemeralConsentKind::Once` and `EphemeralConsentKind::Session` exist.
- `RemotePlannerEphemeralGrant` binds normalized page origin, endpoint scope, policy version, expiry, and one-shot challenge digest.
- One-shot grants use an atomic remaining-use counter and compare-and-exchange consumption.
- Grant storage is held only in `AppCore`; it is not part of serializable `AppState` or persisted configuration.
- Expired grants are pruned before policy evaluation.
- One-shot authorization is consumed exactly once.
- Concurrent duplicate one-shot consumption is prevented by the atomic counter.
- Grant count is bounded to 64.
- Matching session grants are deduplicated.
- Changing privacy settings, planner endpoint, or planner connection settings clears pending consent and all ephemeral grants.
- Reconstructing `AppCore` naturally starts with no grants.

### Partially evidenced or open

- Basic scoped/bounded grant behavior has focused unit coverage.
- Full lifecycle tests remain open: reconstructed-core behavior, repeated matching session requests, endpoint/mode invalidation, expiry removal through complete command flows, and concurrent duplicate response tests.
- There is no dedicated test proving diagnostic formatting cannot disclose a challenge digest.

## Section 6 — Prepared request boundary

### Evidenced complete

- `AppCore::prepare_remote_planner_request` parses and validates the configured endpoint before authorization.
- Privacy evaluation occurs before remote serialization and before network I/O.
- Sanitization, serialized-byte measurement, sanitized payload digest calculation, disclosure classification, and disclosure counts occur during request drafting.
- Preparation returns either an authorized `PreparedRemotePlannerRequest`, a typed consent challenge with bounded pending draft, or a typed terminal block.
- Preparation performs no network I/O.
- `PreparedRemotePlannerRequest` contains sanitized planner input, normalized endpoint scope, profile snapshot, typed authorization, page origin, and runtime-state binding.
- Its construction is confined to the consent module through the draft authorization transition.
- The network sender `resolve_remote_planner` accepts only `PreparedRemotePlannerRequest`; it no longer accepts raw `PlannerInput`.
- The sender defensively rejects missing origin/runtime bindings.
- Endpoint-bound credentials, redirect refusal, configured timeout behavior, response parsing, semantic validation, and post-response safety validation are preserved.
- The `AppCore` lock is released before network I/O.
- Typed resolution and bounded replanning both pass through the preparation boundary.

### Partially evidenced or open

- Direct-command source-drift evidence confirms page-context commands retain the sanitizer/preparation path.
- Dedicated request-count tests remain open: zero requests before consent/deny/block and exactly one request after a valid allow.
- A compile-fail or equivalent structural test proving raw `PlannerInput` cannot reach the sender remains open; the source API currently enforces it but lacks that dedicated test.

## Section 7 — Disclosure and challenge contracts

### Evidenced complete

- `RemotePlannerDisclosureClass` covers user transcript, page origin, selected page regions, selected element metadata, OCR-derived regions, tool-observation summaries, skill summaries, and trusted runtime contracts.
- Disclosure classes are sorted deterministically.
- `RemotePlannerDisclosureCounts` includes selected-region, selected-element, OCR-derived-region, tool-history, skill-summary, and sanitized serialized-byte counts.
- No content excerpts are included in disclosure counts.
- `RemotePlannerConsentChallenge` includes random challenge ID, challenge digest, request ID, normalized page origin, sanitized endpoint display, exact endpoint scope, profile, model, policy version, disclosure classes/counts, expiry, and available-decision flags.
- The public challenge excludes sanitized planner input and raw page/transcript/OCR/tool/skill payloads.
- Canonical manifest hashing binds challenge ID, request ID, origin, endpoint, profile/model, policy version, disclosure classes/counts, sanitized payload digest, runtime-state token, and expiry.

### Open

- Per-field mutation tests for the challenge digest remain open.
- A dedicated JSON privacy test proving no raw payload enters the challenge remains open.

## Section 8 — Runtime-only pending consent state

### Evidenced complete

- `PendingRemotePlannerConsent` lives only inside `AppCore`, outside serializable `AppState`.
- It stores the public challenge, sanitized request draft, runtime-state token through the draft, payload digest, profile/destination snapshot, planning snapshot, and continuation mode.
- It does not store unrestricted raw `PlannerInput`; the draft stores only sanitized planner input.
- Only one pending request can exist because storage is an `Option`; a newer request replaces and drops the prior pending request.
- Challenge lifetime is 120 seconds.
- Resolving a response atomically removes the pending object using `Option::take`.
- Denial, invalid response, expiry, state mismatch, destination change, policy block, successful authorization, and send initiation leave no reusable pending request.
- Privacy or endpoint settings changes clear pending consent.
- Duplicate responses cannot obtain the pending request twice.

### Open

- Dedicated serialized-state, `Debug`, runtime-status, frontend-state, and sensitive-diagnostics-scanner tests for pending consent remain open.
- A typed safe pending-challenge status summary is Stage 2B work.

## Section 9 — Consent-required outcomes

### Evidenced complete

- `ExecutionOutcome::NeedsRemoteDataConsent` exists with execution trace and typed challenge.
- `ResolveCommandOutcome::NeedsRemoteDataConsent` exists for resolve-only flows.
- Rust exhaustive matching was updated across replanning and orchestration.
- Consent-required outcomes are returned before network I/O.
- Direct command matches return resolved direct plans and do not create remote-data consent challenges.
- Local-only, high-risk, opaque-origin, and persistent-block cases remain terminal typed errors rather than override dialogs.
- Typed resolution and voice/bounded-execution backend paths share the same consent preparation and pending continuation mechanism.
- Remote-data consent does not create or bypass protected-action confirmation state; planner output still passes normal safety validation and execution confirmation boundaries.

### Open

- TypeScript contract updates and frontend exhaustive handling remain open.
- Frontend typed-command, voice-command, and dialog behavior remain open.

## Section 10 — Consent-response command

### Evidenced complete

- `submit_remote_planner_consent_response` exists as a Tauri command and is registered in `tauri::generate_handler!`.
- It is included in the exhaustive direct-command registry.
- Policy evidence classifies it as user-gesture-required, runtime/config mutating, credential-bearing network I/O that transmits sanitized page context.
- Stable decisions exist for allow once, allow session, allow persistent, block persistent, and deny.
- Serde enum deserialization rejects unknown decision values.
- Response validation checks pending existence, challenge ID, challenge digest, expiry, runtime-state token, profile name, base URL, model, current privacy policy, high-risk/persistent-block precedence, endpoint scope through the retained draft, and policy version through current evaluation and grant/rule matching.
- Deny and persistent block perform no network I/O for the pending request.
- Persistent block is written before returning and removes grants for that origin.
- Persistent allow is exact-origin/exact-destination/current-policy-version bound.
- Persistence failure returns `remote_data_consent_persist_failed`, installs no effective authorization, and cannot send the request.
- Allow once installs and atomically consumes an exact one-shot grant.
- Allow session installs an exact runtime-only grant.
- Successful allows resume the exact sanitized pending request.
- Pending state is consumed atomically under lock.
- The prepared request is moved out before lock release.
- Network I/O occurs with the `AppCore` lock released.
- Planner output is revalidated for tools, skills, and safety before returning or executing.
- Resolve-only and execute continuations are preserved.

### Partially evidenced or open

- Registry parity and source-drift evidence passed.
- Comprehensive wrong-ID, wrong-digest, replay, double-response, persistence-failure, unlocked-network-wait, and exact-request-count integration tests remain open.

## Section 11 — Runtime-state invalidation

### Evidenced implementation

- The challenge digest and pending draft bind the request to the captured runtime-state token.
- Response handling rejects any changed runtime-state token.
- The current profile name, base URL, and model must still match the pending draft.
- Privacy mode/rule changes and planner endpoint/model changes clear pending consent and all grants.
- Current policy is re-evaluated before applying an allow, so a newly added persistent block, local-only mode, or other terminal policy change prevents resume.
- Policy-version matching is required for grants and persistent allows.

### Open

- The complete explicit invalidation matrix remains unproven by dedicated tests.
- Navigation, document-generation replacement, normalized-origin changes, path-prefix changes, policy-version changes, newly added block, expiry, and unrelated read-only state behavior need focused end-to-end tests.
- The exact set of unrelated state changes that should not invalidate consent still requires documented contract and tests.

## Checklist items that remain intentionally open

The following broader areas must not be inferred complete from Stage 2A:

- Section 12: typed runtime privacy status and settings API;
- Section 13: TypeScript contracts, API wrappers, and safe frontend state;
- Section 14: accessible just-in-time consent dialog;
- Section 15: redesigned privacy settings and structured rule management;
- Section 16: complete high-risk and opaque-origin UI behavior;
- Sections 17–18: comprehensive Rust, frontend, interaction, accessibility, and state-privacy test matrices;
- Section 19: sensitive-diagnostics scanner extensions and focused permanent-CI target;
- Section 20: complete privacy documentation, threat model, and BBCR reconciliation;
- Section 21: the full focused, repository, and adversarial validation sequence;
- Section 22: exact-final-SHA permanent CI and final milestone signoff.

## Correct bounded conclusion

Stage 2A is complete only as a backend implementation stage: the runtime grant model, prepared-request-only network boundary, disclosure/challenge contracts, pending consent transaction, consent-required outcomes, response command, exact request resume, and lock-safe network orchestration are present on `master` and passed the recorded Stage 2A validation job.

The full remote-data-consent/origin-privacy milestone is not complete. The next bounded stage is Stage 2B: typed runtime privacy status/settings APIs, TypeScript contracts, safe frontend state, and the accessible consent experience, followed by the missing test/scanner/documentation/final-signoff work.
