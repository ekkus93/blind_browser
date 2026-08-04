# Blind Browser Remote Data Consent and Origin Privacy Implementation Report

**Report date:** 2026-08-04  
**Repository:** `ekkus93/blind_browser`  
**Branch:** `master`  
**Status:** Stage 1 foundation and Stage 2A backend consent transaction boundary are complete after reconciliation cleanup; the full milestone remains in progress.

## Bounded conclusion

Stage 1 established the versioned privacy configuration, migration, origin-rule validation, deterministic evaluator, and fail-closed pre-network enforcement.

Stage 2A established the backend just-in-time consent transaction boundary: runtime grants, prepared-request-only networking, disclosure manifests, bounded challenges, runtime-only pending state, consent-required outcomes, a consent-response command, exact sanitized-request resume, and lock-safe network orchestration.

Stage 2A does not complete the frontend consent experience or the full milestone. Typed status/settings APIs, TypeScript contracts, safe frontend state, accessible consent UI, structured rule management, comprehensive request-count/replay/concurrency/invalidation/accessibility/adversarial tests, scanner extensions, documentation, BBCR reconciliation, and exact-final-SHA signoff remain open.

The task-level reconciliation is maintained in:

- `docs/BLIND_BROWSER_REMOTE_DATA_CONSENT_ORIGIN_PRIVACY_STAGE2A_RECONCILIATION_2026-08-04.md`

## Stage 1 evidence

- Starting SHA: `043c788346cc9b39942f4198f11c199aaea69ddf`
- Starting permanent CI: run `30886133291`, job `91917696317`, `success`
- Primary implementation: `e6210ae423fb0c5fe08cea9ddc68f463e96b823f`
- Strict-Clippy repair: `14216226b223c092e1a4ff5da5b29c8129f67527`
- Fixture repair: `158672218048f4482879232d7ffc0ea779e9bd07`
- Final trigger-free implementation: `ee967b2fb0d23a762bb8316f369d72c987f31df6`
- Guarded fixture repair: run `30900135542`, job `91962264055`, `success`
- Permanent validation trigger: run `30927205924`, job `92052482518`, `success`
- Exact trigger-free permanent CI: run `30928002322`, job `92055223608`, `success`

## Stage 2A implementation evidence

- Trigger/baseline SHA: `166814c048e5c11b9200243ea6cb7bbe23c9bd78`
- Published implementation SHA: `8ef7f5710daa76061806692a37cc2a13b05710c8`
- Repair/validation run: `30954014288`
- Repair/validation job: `92142680353`
- Repair conclusion: `success`
- Permanent CI run on trigger SHA: `30954014221`
- Permanent CI job on trigger SHA: `92142681677`
- Permanent CI conclusion: `success`

Job `92142680353` generated and validated the final backend tree immediately before committing it as `8ef7f5710daa76061806692a37cc2a13b05710c8`. Because GitHub Actions authored and pushed that commit, GitHub did not start a separate workflow on the output SHA. The successful generating job remains the authoritative implementation evidence.

## Reconciliation and cleanup evidence

- Initial task-by-task reconciliation: `3c0709e6e84801ab22beec2751889f5f17ef9ab2`
- Initial report correction: `b074db0ae4d31cbe505f614db678772687ac15b1`
- Failed run exposing a stale temporary workflow: `30956038519`
- Removed stale workflow: `8c51835a2ba60e2b96c99217497f955614dbf653`
- Removed stale trigger: `d9274f69f8feb76c780f29382d86c2aa4edcf35f`
- Corrected reconciliation evidence: `77eae7ec06a3e887458af2045439feed24da6184`

The first human-authored documentation push exposed that `.github/workflows/remote-data-consent-stage2a-v2-guard-fix2.yml` and `.github/remote-data-consent-stage2a-v2-guard-fix2.trigger` had survived the original cleanup. They were removed directly from `master`.

After `d9274f69f8feb76c780f29382d86c2aa4edcf35f`, `.github/workflows` contains only the permanent `ci.yml`, `publish-ci-status.yml`, and `ralph-loop-apply.yml` workflows. The `.github` root contains no Stage 2A trigger, payload, repair script, or helper file.

This corrects the earlier overstatement that the implementation commit alone removed every temporary Stage 2A artifact.

## Implemented architecture

### Versioned privacy model and persistent decisions

`RemotePlannerPrivacySettings` uses authoritative `network_mode`, `origin_rules`, `policy_schema_version`, and `migration_notice_pending` fields. The new-install default is `AskPerOrigin`. Legacy fields remain readable only for migration.

Persistent blocks are origin-wide. Persistent allows require exact normalized destination scope and current policy version. Rules are URL-normalized, deterministically sorted/deduplicated, and bounded to 256 entries.

### Migration and deterministic evaluator

Legacy local-only/consent settings map into the new network modes, while legacy blocked origins become origin-wide blocks. Migration is idempotent and does not invent destination-specific allows.

The evaluator applies fail-closed precedence: loopback, local-only, missing/opaque origin, high-risk context, persistent block, exact session grant, exact persistent allow, broad sanitized allow, then explicit consent requirement. No default or fallback authorizes transmission.

### Runtime grants

`AppCore` stores bounded runtime-only session and one-shot grants. Grants bind origin, endpoint, policy version, and expiry. One-shot grants also bind challenge digest and use atomic single-use consumption. Expired grants are pruned, matching sessions are deduplicated, and relevant privacy/planner changes clear grants and pending consent.

### Prepared-request-only network boundary

`prepare_remote_planner_request` validates endpoint scope, evaluates privacy, sanitizes input, calculates disclosure classes/counts, measures serialized sanitized bytes, computes a payload digest, and returns an authorized prepared request, typed consent requirement, or terminal policy error without performing network I/O.

The network sender accepts only `PreparedRemotePlannerRequest`, never raw `PlannerInput`. Endpoint-bound credentials, redirect refusal, timeout behavior, parsing, semantic validation, and safety validation remain enforced.

### Disclosure challenge

Disclosure contracts identify transcript, page-origin, selected-region, element-metadata, OCR, tool-observation, skill-summary, and trusted-runtime categories without content excerpts. The challenge includes bounded metadata and a canonical SHA-256 digest binding request/challenge IDs, origin, endpoint, profile/model, policy version, disclosure summary, payload digest, runtime token, and expiry. It excludes request content.

### Pending transaction and outcomes

One runtime-only pending transaction stores the challenge, sanitized draft, planning snapshot, and resolve/execute continuation outside serializable `AppState`. New pending work replaces old work. Response handling atomically removes pending state, preventing replay and duplicate consumption.

Rust contracts include consent-required resolve/execution outcomes, disclosure/challenge types, stable consent decisions, and typed response outcomes.

### Consent response and lock boundary

`submit_remote_planner_consent_response` is registered with Tauri and the direct-command policy. It validates challenge identity/digest, expiry, runtime state, current planner profile/destination/model, and current privacy policy.

Deny and persistent block perform no request send. Persistent decisions must be durably written before authorization. Persistence failure returns `remote_data_consent_persist_failed` and cannot send. Valid allows resume only the exact sanitized pending request.

Pending state is consumed under the `AppCore` lock, the prepared request is moved out, and network I/O occurs after lock release. Planner output is revalidated before resolve or execution completion. Remote-data consent does not replace protected-action confirmation.

## Stage 2A changed source areas

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

## Stage 2A validation

Run `30954014288`, job `92142680353`, passed:

- security fallback scanner self-test/audit;
- security fallback inventory self-test/audit;
- sensitive diagnostics scanner self-test/audit;
- `cargo check`;
- strict all-target/all-feature Clippy;
- five focused remote-data-consent tests;
- all-feature Rust tests: 483 passed, 0 failed;
- hostile-content corpus: 4 passed, 0 failed;
- direct-command policy evidence: 6 passed, 0 failed;
- frontend lint;
- frontend UI tests: 2 passed, 0 failed;
- frontend production build.

These results do not substitute for the checklist's still-open dedicated request-count, replay, concurrency, invalidation, serialization-privacy, accessibility, and adversarial tests.

## Still open

- typed runtime privacy status and settings-management APIs;
- TypeScript privacy/consent contracts and Tauri wrappers;
- safe frontend consent state and exhaustive outcome handling;
- accessible just-in-time consent dialog;
- redesigned privacy settings and structured rule management;
- complete high-risk and opaque-origin presentation;
- request-count tests for ask/deny/block/allow and duplicate response;
- comprehensive replay, concurrency, persistence-failure, expiry, and invalidation tests;
- challenge-digest mutation and serialized-state privacy tests;
- sensitive-diagnostics/frontend-state scanner extensions;
- focused permanent-CI consent target;
- complete privacy, migration, disclosure, revocation, and threat-model documentation;
- BBCR and post-Batch-8 reconciliation;
- exact-final-SHA permanent CI and final milestone signoff.

## Next stage

Stage 2B should proceed in this order:

1. typed `RemotePlannerPrivacyStatus` and settings/rule-management commands;
2. TypeScript contracts and Tauri wrappers;
3. safe frontend challenge state and exhaustive typed/voice/replanning outcome handling;
4. accessible just-in-time consent dialog;
5. planner privacy settings and structured rule-management redesign;
6. missing request-count, replay, concurrency, invalidation, accessibility, scanner, adversarial, and documentation closure.

## Bounded statement

Stage 1 and Stage 2A are complete under their bounded definitions after the reconciliation cleanup. The full remote-data-consent/origin-privacy milestone and broader BBCR program remain open.
