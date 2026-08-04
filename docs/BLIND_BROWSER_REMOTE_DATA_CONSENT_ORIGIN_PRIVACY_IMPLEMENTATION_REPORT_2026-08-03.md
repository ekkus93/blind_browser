# Blind Browser Remote Data Consent and Origin Privacy Implementation Report

**Report date:** 2026-08-04  
**Repository:** `ekkus93/blind_browser`  
**Branch:** `master`  
**Status:** Stage 1 foundation and Stage 2A backend consent transaction boundary are complete; the full milestone remains in progress.

## Bounded conclusion

Stage 1 established the versioned privacy configuration, legacy migration, origin-rule validation, deterministic privacy evaluator, and fail-closed pre-network enforcement.

Stage 2A added the backend just-in-time consent transaction boundary: runtime grants, prepared-request-only networking, disclosure manifests, bounded consent challenges, runtime-only pending state, typed consent-required outcomes, a consent-response command, exact sanitized-request resume, and lock-safe network orchestration.

Stage 2A does not complete the frontend consent experience or the full milestone. Typed privacy status/settings APIs, TypeScript contracts, safe frontend state, accessible consent UI, structured rule management, comprehensive request-count/replay/concurrency/invalidation/accessibility/adversarial tests, scanner extensions, privacy documentation, BBCR reconciliation, and exact-final-SHA signoff remain open.

The task-by-task Stage 2A reconciliation is recorded in:

- `docs/BLIND_BROWSER_REMOTE_DATA_CONSENT_ORIGIN_PRIVACY_STAGE2A_RECONCILIATION_2026-08-04.md`

## Stage 1 baseline and exact evidence

- Starting SHA: `043c788346cc9b39942f4198f11c199aaea69ddf`
- Starting permanent CI: run `30886133291`, job `91917696317`, `success`
- Primary foundation implementation: `e6210ae423fb0c5fe08cea9ddc68f463e96b823f`
- Strict-Clippy repair: `14216226b223c092e1a4ff5da5b29c8129f67527`
- Fixture migration repair: `158672218048f4482879232d7ffc0ea779e9bd07`
- Final trigger-free Stage 1 implementation SHA: `ee967b2fb0d23a762bb8316f369d72c987f31df6`
- Guarded fixture repair: run `30900135542`, job `91962264055`, `success`
- Permanent validation trigger: run `30927205924`, job `92052482518`, `success`
- Exact trigger-free permanent CI: run `30928002322`, job `92055223608`, `success`

## Stage 2A exact evidence

- Stage 2A trigger/baseline SHA: `166814c048e5c11b9200243ea6cb7bbe23c9bd78`
- Published Stage 2A implementation SHA: `8ef7f5710daa76061806692a37cc2a13b05710c8`
- Repair and validation run: `30954014288`
- Repair and validation job: `92142680353`
- Repair job conclusion: `success`
- Permanent CI run on trigger SHA: `30954014221`
- Permanent CI job on trigger SHA: `92142681677`
- Permanent CI conclusion: `success`
- Stage 2A reconciliation commit: `3c0709e6e84801ab22beec2751889f5f17ef9ab2`

The repair job generated the final Stage 2A tree, validated it, committed it as `8ef7f5710daa76061806692a37cc2a13b05710c8`, removed the temporary workflows/scripts/payloads/triggers, and pushed it to `master`.

Because that output commit was authored and pushed by GitHub Actions, GitHub did not start a separate workflow for the output SHA. There is no independent run or status attached to `8ef7f5710daa76061806692a37cc2a13b05710c8`. Job `92142680353` is the authoritative Stage 2A evidence because it validated the generated tree immediately before committing and pushing it. A later human-authored documentation commit must receive permanent CI on its exact SHA before this limitation can be retired.

## Implemented architecture

### Versioned privacy model

`RemotePlannerPrivacySettings` uses authoritative `network_mode`, `origin_rules`, `policy_schema_version`, and `migration_notice_pending` fields. Legacy consent, local-only, and blocked-origin fields remain readable only across the migration boundary and are synchronized from the new model after normalization. The new-install default is `AskPerOrigin`.

### Persistent decisions

Persistent blocks are origin-wide and contain no endpoint scope. Persistent allows require the exact normalized planner endpoint and current policy version. Rules are normalized with the URL library, sorted and deduplicated deterministically, and bounded to 256 entries.

### Legacy migration

Schema-zero settings map legacy local-only and consent booleans into the new global mode. Legacy blocked origins become origin-wide blocks. Migration is idempotent, sets a bounded notice, and does not manufacture destination-specific allows from broad legacy consent.

### Pure evaluator

The evaluator applies fail-closed precedence:

1. loopback local service;
2. local-only mode;
3. missing or opaque origin;
4. high-risk context;
5. persistent origin block;
6. exact unexpired session grant;
7. exact current-version persistent allow;
8. broad sanitized non-high-risk allow;
9. explicit consent required.

No default or fallback branch authorizes transmission.

### Runtime ephemeral grants

`AppCore` stores runtime-only session and one-shot grants. Grants bind page origin, endpoint scope, privacy-policy version, and expiry. One-shot grants additionally bind the challenge digest and use an atomic remaining-use counter, preventing duplicate consumption. Grant storage is bounded, expired grants are pruned, matching session grants are deduplicated, and privacy/endpoint configuration changes clear grants and pending consent.

### Prepared-request-only network boundary

`prepare_remote_planner_request` validates destination scope, evaluates privacy, sanitizes planner input, computes disclosure classes/counts, measures serialized sanitized bytes, computes a payload digest, and returns either an authorized prepared request, a typed consent requirement, or a terminal policy error. Preparation performs no network I/O.

The network sender accepts only `PreparedRemotePlannerRequest`; it does not accept raw `PlannerInput`. The prepared request contains sanitized input, typed authorization, normalized endpoint scope, profile snapshot, page origin, and runtime-state binding. Existing endpoint-bound credential resolution, redirect refusal, timeout behavior, response parsing, semantic validation, and safety validation remain in force.

### Disclosure manifest and consent challenge

The disclosure contract classifies transcript, page origin, selected regions, selected element metadata, OCR-derived regions, tool-observation summaries, skill summaries, and trusted runtime contracts. Counts contain only bounded cardinalities and sanitized serialized byte size, not content excerpts.

A random consent challenge binds challenge/request IDs, normalized origin, exact endpoint, profile/model, policy version, disclosure classes/counts, sanitized payload digest, runtime-state token, and expiry through canonical serialized SHA-256 hashing. The public challenge excludes sanitized request content and raw page/transcript/OCR/tool/skill payloads.

### Pending transaction and typed outcomes

`PendingRemotePlannerConsent` is runtime-only inside `AppCore`, outside serializable `AppState`. A single bounded pending transaction stores the public challenge, sanitized request draft, planning snapshot, and resolve-or-execute continuation. New requests replace old pending state. Responses atomically remove the pending object before validation or resume, preventing replay and duplicate consumption.

Rust contracts include `ExecutionOutcome::NeedsRemoteDataConsent`, `ResolveCommandOutcome::NeedsRemoteDataConsent`, the disclosure/challenge types, stable consent decisions, and typed response outcomes.

### Consent-response command

`submit_remote_planner_consent_response` is a registered Tauri command and an exhaustive direct-command-policy entry. It validates challenge ID/digest, expiry, runtime state, current profile/destination/model, current policy, and terminal block precedence.

Deny and persistent block perform no network I/O for the pending request. Persistent writes complete before authorization. Persistence failure returns `remote_data_consent_persist_failed`, installs no effective grant, and cannot send. Allow-once, session, and persistent decisions authorize only the exact bound request/scope. Protected-action confirmation remains a separate downstream boundary.

### Lock-safe exact resume

Both typed resolution and bounded replanning use the same preparation boundary. Consent responses consume pending state under the `AppCore` lock, move the prepared request out, release the lock for network I/O, validate planner output, and reacquire the lock only to register the planning snapshot and optionally execute. Resolve-only and execute continuations resume the exact sanitized pending request.

## Stage 1 changed source areas

The Stage 1 implementation changed:

- `config.example.toml`
- `src-tauri/src/app_core/mod.rs`
- `src-tauri/src/app_core/planner_redaction.rs`
- `src-tauri/src/app_core/planning_snapshot.rs`
- `src-tauri/src/app_core/remote_data_consent.rs`
- `src-tauri/src/app_core/runtime_config.rs`
- `src-tauri/src/config/persistence.rs`
- `src-tauri/src/config/types.rs`
- `src-tauri/src/config/validation.rs`

## Stage 2A changed source and evidence areas

The published Stage 2A commit changed the backend and evidence surface in:

- `src-tauri/src/app_core/mod.rs`
- `src-tauri/src/app_core/planner_redaction.rs`
- `src-tauri/src/app_core/remote_data_consent.rs`
- `src-tauri/src/app_core/remote_planner.rs`
- `src-tauri/src/app_core/replanning.rs`
- `src-tauri/src/app_core/replanning_orchestrator.rs`
- `src-tauri/src/app_core/runtime_config.rs`
- `src-tauri/src/app_core/tests/planner_tests.rs`
- `src-tauri/src/command_handlers/core_handlers.rs`
- `src-tauri/src/commands/contracts/planner.rs`
- `src-tauri/src/direct_command_policy.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/state.rs`
- `src-tauri/tests/post_batch8_direct_command_policy_evidence.rs`

All temporary Stage 2A repair workflows, scripts, payloads, triggers, and diagnostic machinery were deleted by the published commit. No temporary Stage 2A machinery remains in the final tree.

## Failed intermediate evidence and repairs

The first Stage 1 validation wrapper was invalidated because it captured `$?` after a shell `if` compound command, allowing a failed validation command to appear successful. Permanent CI—not the wrapper—remained authoritative.

Permanent run `30897812057`, job `91954789324`, then failed strict Clippy on an unnecessary cloned test slice. Commit `14216226b223c092e1a4ff5da5b29c8129f67527` corrected it.

Permanent run `30898762353`, job `91957861759`, passed Clippy and exposed eight stale planner-redaction fixtures. A count-guarded migration updated them to the versioned fields and canonical `remote_data_*` error codes. The repair workflow passed in run `30900135542`, job `91962264055`, and published `158672218048f4482879232d7ffc0ea779e9bd07`.

Stage 2A also required multiple bounded repair iterations. The final repair job is authoritative; failed or superseded temporary workflows are not completion evidence. The final job used explicit logged exit-code handling, passed all validation, committed the generated tree, and removed all temporary machinery.

## Validation

### Stage 1

The exact trigger-free Stage 1 implementation passed fallback and sensitive-diagnostic scanners, Rust formatting, default compilation, strict all-target/all-feature Clippy, focused direct-command semantic evidence, all-feature Rust tests, frontend lint, UI tests, and frontend production build in run `30928002322`, job `92055223608`.

### Stage 2A

Run `30954014288`, job `92142680353`, passed:

- security fallback scanner self-test and audit;
- security fallback inventory self-test and audit;
- sensitive diagnostics scanner self-test and audit;
- `cargo check`;
- strict all-target/all-feature Clippy with warnings denied;
- five focused remote-data-consent unit tests;
- all-feature Rust tests: 483 passed, 0 failed;
- hostile-content corpus: 4 passed, 0 failed;
- direct-command policy evidence: 6 passed, 0 failed;
- frontend lint;
- frontend UI tests: 2 passed, 0 failed;
- frontend production build;
- temporary machinery cleanup and absence checks.

These results validate the Stage 2A implementation but do not substitute for the checklist's still-open dedicated request-count, replay, concurrency, invalidation, serialization-privacy, accessibility, and adversarial integration tests.

## Still open

The following are not claimed complete:

- typed runtime privacy status and settings-management APIs;
- TypeScript consent/privacy contracts and Tauri wrappers;
- safe frontend consent state and exhaustive outcome handling;
- accessible just-in-time consent dialog;
- redesigned privacy mode/current-origin/rule-management UI;
- complete high-risk and opaque-origin presentation;
- dedicated request-count tests for ask/deny/block/allow and duplicate response;
- comprehensive replay, concurrency, persistence-failure, expiry, and state-invalidation tests;
- per-field challenge-digest mutation tests;
- serialized-state, diagnostic-formatting, scanner, and frontend-state privacy tests;
- sensitive-diagnostics scanner extensions for pending consent and frontend consent state;
- focused permanent-CI consent target;
- complete privacy, migration, disclosure, revocation, and threat-model documentation;
- BBCR and post-Batch-8 reconciliation;
- exact-final-SHA permanent CI and final milestone signoff.

## Recommended next stage

Stage 2B should proceed in this order:

1. add typed `RemotePlannerPrivacyStatus` and settings/rule-management commands;
2. stabilize TypeScript contracts and Tauri API wrappers;
3. add safe frontend challenge state and exhaustive typed/voice/replanning outcome handling;
4. build the accessible just-in-time consent dialog;
5. redesign planner privacy settings and structured rule management;
6. complete request-count, replay, concurrency, invalidation, accessibility, scanner, adversarial, and documentation closure.

## Bounded statement

Stage 1 and the Stage 2A backend consent transaction boundary are implemented and green under their recorded evidence. The full remote-data-consent/origin-privacy milestone and the broader BBCR program remain open.
