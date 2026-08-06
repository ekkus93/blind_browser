# Blind Browser Remote Data Consent and Origin Privacy TODO

**Date:** 2026-08-03  
**Repository:** `ekkus93/blind_browser`  
**Branch:** `master`  
**Reviewed baseline:** `0c0acb0d76210afc6fe40a0ebd32f50e89897d91`  
**Companion spec:** `docs/BLIND_BROWSER_REMOTE_DATA_CONSENT_ORIGIN_PRIVACY_SPEC_2026-08-03.md`  
**Parent remediation items:** BBCR-003, BBCR-006, BBCR-008, BBCR-015, and BBCR-021  
**Status:** Reconciled and complete for the bounded current first-party remote-data consent/origin-privacy milestone. Every checkbox has an explicit disposition in `docs/BLIND_BROWSER_REMOTE_DATA_PRIVACY_MILESTONE_RECONCILIATION_2026-08-05.md`. Cross-platform assistive-technology certification and unrelated BBCR work remain outside this bounded source milestone.
**Release boundary:** This checklist closes a focused remote-data consent and origin-privacy milestone only. It must not be used to declare the full BBCR program complete or the project production-ready.
**Reconciliation convention:** A checked box means its disposition is complete. It may mean implemented/evidenced, completed audit procedure, equivalent current architecture, or reviewed optional path not selected. It does not mean every suggested alternative was adopted.

---

## Completion rules

- [x] Work directly on `master` unless the user explicitly requests a branch, PR, or worktree.
- [x] Preserve this complete checklist through implementation and closure.
- [x] Check an item only when source, test, scanner, documentation, or CI evidence exists on `master`.
- [x] Do not weaken planner redaction, endpoint scoping, action policy, confirmation, runtime-state binding, prompt-injection handling, or high-risk blocking.
- [x] Treat every first-party test, scanner, compiler, Clippy, frontend, and CI failure as a real defect unless evidence proves otherwise.
- [x] No non-loopback planner request may occur before deterministic privacy authorization.
- [x] No consent decision may authorize or reduce confirmation for a protected action.
- [x] Do not persist raw transcript, page, OCR, tool-observation, skill, credential, or planner-payload content.
- [x] Remove all temporary workflows, generators, patch scripts, diagnostic helpers, and test bypasses before closure.
- [x] Record exact implementation, cleanup, documentation, and final evidence SHAs.
- [x] Record exact permanent CI run and job identifiers for the final SHA.

---

## 0. Baseline and implementation setup

- [x] Confirm latest `master` SHA before implementation.
- [x] Confirm `ci/permanent` is green for the starting SHA.
- [x] Read the companion spec completely.
- [x] Read `docs/BLIND_BROWSER_COMPREHENSIVE_CODE_REVIEW_FIX_TODO_2026-08-01.md`.
- [x] Read `docs/BLIND_BROWSER_POST_BATCH8_RECONCILIATION_2026-08-02.md`.
- [x] Read the post-P8 fallback hardening spec, TODO, report, and closure.
- [x] Confirm no temporary Ralph or repair machinery remains in the starting tree.
- [x] Inventory all files expected to change before coding.
- [x] Record the expected changed-file scope in the implementation report.
- [x] Decide whether implementation needs a bounded temporary workflow.
  - [x] If used, make it exact-triggered and self-cleaning.
  - [x] If not used, do not add one unnecessarily.

### Expected source areas

- [x] `src-tauri/src/config/types.rs`
- [x] `src-tauri/src/config/defaults.rs` or the current default-construction module
- [x] `src-tauri/src/config/validation.rs`
- [x] `src-tauri/src/config/persistence.rs`
- [x] `src-tauri/src/app_core/planner_redaction.rs`
- [x] `src-tauri/src/app_core/remote_planner.rs`
- [x] `src-tauri/src/app_core/command_dispatch.rs`
- [x] `src-tauri/src/app_core/replanning.rs`
- [x] `src-tauri/src/app_core/runtime_config.rs`
- [x] `src-tauri/src/app_core/settings_adapters.rs`
- [x] `src-tauri/src/app_core/state_snapshots.rs`
- [x] `src-tauri/src/commands/contracts/planner.rs`
- [x] `src-tauri/src/commands/contracts/providers.rs`
- [x] `src-tauri/src/command_handlers/safety_handlers.rs`
- [x] `src-tauri/src/direct_command_policy.rs`
- [x] `src-tauri/src/lib.rs`
- [x] focused Rust unit/integration tests
- [x] `src/tauri-types.ts`
- [x] `src/api/providers.ts`
- [x] `src/planner-actions.ts`
- [x] `src/runtime-refresh.ts`
- [x] `src/panel-state.ts`
- [x] `src/panel-types.ts`
- [x] `src/settings-panels/planner.tsx`
- [x] consent UI component(s) and tests
- [x] `config.example.toml`
- [x] `docs/SPECS.md` and privacy/security documentation

---

## 1. Audit the current privacy and request path

### 1.1 Config and settings audit

- [x] Inspect `RemotePlannerPrivacySettings` and `HighRiskOriginPolicy`.
- [x] Confirm current defaults for global consent, local-only, blocked origins, and high-risk policy.
- [x] Inspect config normalization for blocked origins.
- [x] Inspect config persistence and schema migration behavior.
- [x] Confirm how unknown config fields and missing legacy fields are handled.
- [x] Identify every test fixture that constructs `RemotePlannerPrivacySettings` directly.
- [x] Identify every TypeScript fixture that assumes the current booleans/list contract.

### 1.2 Planner path audit

- [x] Trace `resolve_command` from direct-command matching to `PlannerResolution::Remote`.
- [x] Trace `transcribe_and_execute_command` through remote planning.
- [x] Trace bounded replanning through the remote planner.
- [x] Confirm exactly where sanitization currently occurs.
- [x] Confirm exactly where privacy evaluation currently occurs.
- [x] Confirm no planner network client is called before privacy evaluation.
- [x] Identify every function that accepts raw `PlannerInput` and can reach network I/O.
- [x] Identify every place the `AppCore` mutex is released for remote work.
- [x] Identify every place remote planner errors become `ExecutionOutcome::Aborted` or frontend errors.

### 1.3 Runtime state audit

- [x] Inspect `AppCore` fields and serializable `AppState` fields.
- [x] Identify where a runtime-only pending consent object can live.
- [x] Confirm pending confirmation state patterns that can be reused safely.
- [x] Confirm runtime-state token composition and invalidation behavior.
- [x] Identify state changes that must invalidate consent.
- [x] Identify unrelated read-only state changes that should not invalidate consent.

### 1.4 Frontend audit

- [x] Inspect current planner privacy settings UI.
- [x] Inspect current manual blocked-origin textarea behavior.
- [x] Inspect planner action error handling.
- [x] Inspect voice command outcome handling.
- [x] Inspect current modal/focus-management patterns.
- [x] Inspect Redux/panel state persistence boundaries.
- [x] Confirm whether raw transcript or planner payload data enters global frontend state.
- [x] Identify current accessibility test helpers.

### 1.5 Document audit conclusions

- [x] Record the current control/data flow in the implementation report.
- [x] Record exact pre-network authorization insertion points.
- [x] Record migration risks.
- [x] Record the final expected source/test/doc change set before implementation.

---

## 2. Add the new config and policy data model

### 2.1 Global mode

- [x] Add `RemotePlannerNetworkMode`.
- [x] Include `LocalOnly`.
- [x] Include `AskPerOrigin`.
- [x] Include `AllowSanitizedNonHighRisk`.
- [x] Use stable `snake_case` serialization.
- [x] Make `AskPerOrigin` the new-install default.
- [x] Remove authoritative dependence on the old interacting booleans after migration.
- [x] Keep old fields readable only for the declared migration boundary.

Suggested shape:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RemotePlannerNetworkMode {
    LocalOnly,
    AskPerOrigin,
    AllowSanitizedNonHighRisk,
}
```

### 2.2 Persistent origin rules

- [x] Add `PersistedOriginDecision` with `Allow` and `Block`.
- [x] Add `RemotePlannerOriginRule`.
- [x] Include normalized page origin.
- [x] Include decision.
- [x] Include optional endpoint scope.
- [x] Include privacy-policy version.
- [x] Include non-sensitive creation timestamp.
- [x] Add `REMOTE_DATA_POLICY_VERSION`.
- [x] Require `Block` rules to have no endpoint scope.
- [x] Require `Allow` rules to have an exact normalized endpoint scope.
- [x] Make persistent blocks apply across all non-loopback destinations.
- [x] Make persistent allows destination- and policy-version-bound.
- [x] Define deterministic rule identity and sort order.
- [x] Limit persistent rules to at most 256.
- [x] Define stale allow behavior.
- [x] Ensure stale allows are visible but non-authorizing.

Suggested shape:

```rust
pub struct RemotePlannerOriginRule {
    pub page_origin: String,
    pub decision: PersistedOriginDecision,
    pub endpoint_scope: Option<String>,
    pub policy_version: u32,
    pub created_at_ms: u64,
}
```

### 2.3 Updated settings structure

- [x] Replace the current privacy fields with `network_mode` and `origin_rules`.
- [x] Retain `high_risk_origin_policy`.
- [x] Keep serialization deterministic.
- [x] Keep config debug formatting free of sensitive data.
- [x] Update JSON schema expectations.
- [x] Update all direct Rust fixture initializers.
- [x] Avoid partial fixture publication; search the complete Rust test tree.

### 2.4 Config validation

- [x] Add one shared normalized HTTP(S) page-origin type/helper.
- [x] Reject paths.
- [x] Reject queries.
- [x] Reject fragments.
- [x] Reject username/password.
- [x] Reject opaque or `null` origins.
- [x] Reject non-HTTP(S) schemes.
- [x] Normalize scheme, host, effective port, and IDNA consistently through the URL library.
- [x] Validate endpoint scopes using `ProviderEndpointScope`.
- [x] Reject allow rules with missing endpoint scope.
- [x] Reject block rules with an endpoint scope.
- [x] Reject zero/future unsupported policy versions.
- [x] Deduplicate exact duplicate rules deterministically.
- [x] Define conflict handling for allow and block rules on the same origin.
  - [x] Persistent block must win.
  - [x] Validation must not silently discard a block in favor of allow.
- [x] Add bounded, non-secret validation errors.

---

## 3. Implement legacy configuration migration

### 3.1 Mapping

- [x] Map legacy `local_only = true` to `LocalOnly`.
- [x] Map legacy `local_only = false` and `consent = false` to `AskPerOrigin`.
- [x] Map legacy `local_only = false` and `consent = true` to `AllowSanitizedNonHighRisk`.
- [x] Convert each legacy blocked origin to an origin-wide persistent `Block` rule.
- [x] Preserve `HighRiskOriginPolicy::Block`.
- [x] Do not manufacture destination-bound allows from global legacy consent.

### 3.2 Migration safety

- [x] Make migration idempotent.
- [x] Validate before durable write.
- [x] Ensure failed migration leaves the previous config intact.
- [x] Ensure malformed legacy blocked origins fail closed.
- [x] Add a migration schema/version marker if needed.
- [x] Preserve a safe rollback/read path for the supported migration boundary.
- [x] Avoid writing partially migrated settings.
- [x] Add a bounded one-time migration notice to runtime/settings status.
- [x] Update `config.example.toml`.

### 3.3 Migration tests

- [x] Test every legacy boolean combination.
- [x] Test legacy blocked-origin conversion.
- [x] Test duplicate legacy origins.
- [x] Test malformed legacy origin failure.
- [x] Test migration idempotence.
- [x] Test deterministic serialization order.
- [x] Test new-install default.
- [x] Test existing broad consent remains broad mode rather than becoming per-destination allow.
- [x] Test migration failure preserves old config bytes.

---

## 4. Implement a pure deterministic privacy evaluator

### 4.1 Types

- [x] Add `RemotePlannerEffectiveDecision`.
- [x] Add `RemotePlannerDataAuthorization`.
- [x] Add typed privacy block/ask reasons.
- [x] Add safe public reason-code conversion.
- [x] Keep evaluator inputs explicit and immutable.
- [x] Keep evaluator independent of frontend state.

Suggested effective decisions:

- [x] `LoopbackLocal`
- [x] `LocalOnly`
- [x] `HighRiskBlocked`
- [x] `OriginBlocked`
- [x] `AllowedGlobal`
- [x] `AllowedPersistent`
- [x] `AllowedSession`
- [x] `AllowedOnce`
- [x] `ConsentRequired`
- [x] `OriginUnavailable`
- [x] `PlannerUnavailable`

### 4.2 Precedence

- [x] Invalid/missing endpoint fails before consent evaluation.
- [x] Loopback returns local authorization.
- [x] Local-only blocks all non-loopback destinations.
- [x] Unknown/opaque/non-HTTP(S) page origin blocks network page-context planning.
- [x] High-risk context blocks before all grants/allows.
- [x] Persistent block overrides global allow.
- [x] One-shot grant requires exact challenge binding.
- [x] Session grant requires exact origin/destination/version match.
- [x] Persistent allow requires exact origin/destination/version match.
- [x] Broad global allow permits only sanitized non-high-risk context.
- [x] Ask mode returns a challenge requirement.
- [x] No fallback path silently authorizes transmission.

### 4.3 Pure evaluator tests

- [x] Create a table-driven test for every precedence branch.
- [x] Test local-only versus persistent allow.
- [x] Test high-risk versus every allow type.
- [x] Test persistent block versus broad global allow.
- [x] Test exact persistent allow.
- [x] Test scheme change.
- [x] Test host change.
- [x] Test effective-port change.
- [x] Test endpoint path-prefix change.
- [x] Test policy-version change.
- [x] Test expired grant.
- [x] Test one-shot remaining-use behavior.
- [x] Test no-rule ask behavior.
- [x] Test unknown origin.
- [x] Test malformed rule input cannot authorize.

---

## 5. Add runtime-only ephemeral grants

### 5.1 Grant representation

- [x] Add `EphemeralConsentKind::Once`.
- [x] Add `EphemeralConsentKind::Session`.
- [x] Add `RemotePlannerEphemeralGrant`.
- [x] Bind page origin.
- [x] Bind endpoint scope.
- [x] Bind policy version.
- [x] Bind one-shot challenge digest.
- [x] Add expiry.
- [x] Add atomic remaining-use count for one-shot grants.
- [x] Keep grant storage runtime-only.

### 5.2 Lifecycle

- [x] Clear all grants on application exit naturally by not persisting them.
- [x] Clear grants when network mode becomes `LocalOnly`.
- [x] Make endpoint changes invalidate destination-bound grants.
- [x] Make policy-version changes invalidate grants.
- [x] Remove expired grants before evaluation.
- [x] Consume one-shot grant exactly once.
- [x] Prevent concurrent duplicate consumption.
- [x] Bound grant count.
- [x] Deduplicate matching session grants.
- [x] Avoid logging full grant structures if they contain challenge digests.

### 5.3 Tests

- [x] Session grant survives multiple matching requests in one process.
- [x] Session grant does not survive reconstructed `AppCore`.
- [x] Session grant does not match another origin.
- [x] Session grant does not match another destination.
- [x] One-shot grant succeeds once.
- [x] One-shot replay fails.
- [x] Mode change invalidates grants.
- [x] Endpoint change invalidates grants.
- [x] Expired grants are removed.

---

## 6. Split request preparation from network sending

### 6.1 Preparation boundary

- [x] Add `prepare_remote_planner_request` or equivalent.
- [x] Move destination validation into the preparation stage.
- [x] Move sanitization into the preparation stage.
- [x] Calculate disclosure classes.
- [x] Calculate disclosure counts.
- [x] Calculate sanitized serialized byte estimate.
- [x] Calculate sanitized payload digest.
- [x] Evaluate deterministic privacy.
- [x] Return an authorized prepared request or a typed consent requirement.
- [x] Ensure preparation performs no network I/O.

### 6.2 Prepared request type

- [x] Add `PreparedRemotePlannerRequest`.
- [x] Require `RemotePlannerDataAuthorization` in its constructor.
- [x] Make fields private where practical.
- [x] Prevent direct construction outside the privacy boundary.
- [x] Store only sanitized planner input.
- [x] Include normalized endpoint scope.
- [x] Include profile name/profile snapshot.
- [x] Include available tools and active skill names needed for output validation.
- [x] Avoid raw `PlannerInput` storage.

### 6.3 Network sender

- [x] Change the network sender to accept only `PreparedRemotePlannerRequest`.
- [x] Remove or make private any network function accepting raw `PlannerInput`.
- [x] Reject missing/invalid authorization defensively.
- [x] Preserve endpoint-bound credential resolution.
- [x] Preserve redirect refusal.
- [x] Preserve planner timeout behavior currently implemented.
- [x] Preserve response parsing and semantic validation.
- [x] Preserve lock release before network I/O.

### 6.4 Structural evidence tests

- [x] Add a test proving raw `PlannerInput` cannot reach the network sender API.
- [x] Add a test proving ask mode increments no test-server request count.
- [x] Add a test proving allowed prepared request increments exactly once.
- [x] Add direct-command evidence showing page-context paths still pass through the privacy preparation boundary.

---

## 7. Add typed consent challenge contracts

### 7.1 Disclosure types

- [x] Add `RemotePlannerDisclosureClass`.
- [x] Include user transcript.
- [x] Include page origin/sanitized URL.
- [x] Include selected page regions.
- [x] Include selected element metadata.
- [x] Include OCR-derived regions.
- [x] Include tool-observation summaries.
- [x] Include skill summaries.
- [x] Include trusted runtime contracts.
- [x] Add deterministic ordering.

### 7.2 Disclosure counts

- [x] Add selected region count.
- [x] Add selected element count.
- [x] Add OCR-derived region count.
- [x] Add tool-history count.
- [x] Add skill-summary count.
- [x] Add sanitized serialized byte estimate.
- [x] Do not include content excerpts.

### 7.3 Challenge type

- [x] Add `RemotePlannerConsentChallenge`.
- [x] Add random challenge ID.
- [x] Add challenge digest.
- [x] Add request ID.
- [x] Add normalized page origin.
- [x] Add sanitized endpoint display.
- [x] Add exact endpoint scope.
- [x] Add profile name.
- [x] Add model label.
- [x] Add policy version.
- [x] Add disclosure classes/counts.
- [x] Add expiry.
- [x] Add available decision flags.
- [x] Ensure serialization excludes raw payload.

### 7.4 Challenge digest

- [x] Define canonical challenge-manifest serialization.
- [x] Bind request ID.
- [x] Bind page origin.
- [x] Bind endpoint scope.
- [x] Bind policy version.
- [x] Bind disclosure classes/counts.
- [x] Bind sanitized payload digest.
- [x] Bind relevant runtime-state token/digest.
- [x] Use stable hashing already approved in the repository or add an appropriate crate deliberately.
- [x] Ensure field reorder cannot alter semantic equality unexpectedly.
- [x] Add mutation tests for each bound field.

---

## 8. Add runtime-only pending consent state

### 8.1 Pending object

- [x] Add `PendingRemotePlannerConsent` to `AppCore`.
- [x] Keep it out of serializable `AppState`.
- [x] Keep it out of public runtime status except a safe summary.
- [x] Store the challenge.
- [x] Store runtime-state token.
- [x] Store sanitized payload digest.
- [x] Store sanitized input.
- [x] Store profile/destination snapshot.
- [x] Store available tools/active skills required for validation.
- [x] Do not store unrestricted page model.
- [x] Do not store unrestricted transcript.

### 8.2 Bounded lifecycle

- [x] Permit at most one pending consent request.
- [x] Define replacement behavior when a new remote request arrives.
- [x] Drop the old pending request when replaced.
- [x] Use a 120-second initial expiry unless tests justify another value.
- [x] Clear pending consent on deny.
- [x] Clear pending consent on invalid response.
- [x] Clear pending consent on state mismatch.
- [x] Clear pending consent on endpoint/mode change.
- [x] Clear pending consent after successful consumption.
- [x] Clear pending consent after network send starts so duplicate response cannot resend.

### 8.3 Serialization/redaction tests

- [x] Serialized `AppState` contains no pending consent payload.
- [x] `Debug`/diagnostic formatting contains no raw sanitized input.
- [x] Runtime status exposes only challenge summary.
- [x] Sensitive diagnostic scanner covers pending consent types.
- [x] Frontend state receives no raw pending payload.

---

## 9. Add `NeedsRemoteDataConsent` execution outcome

### 9.1 Rust contract

- [x] Add `ExecutionOutcome::NeedsRemoteDataConsent`.
- [x] Include existing execution trace shape.
- [x] Include typed challenge.
- [x] Update exhaustive matches in Rust.
- [x] Update state application behavior.
- [x] Ensure it does not create pending protected-action confirmation state.
- [x] Ensure it does not report execution success.

### 9.2 TypeScript contract

- [x] Regenerate/update `ExecutionOutcome` TypeScript type.
- [x] Add challenge/disclosure/decision types.
- [x] Update exhaustive frontend switches.
- [x] Add compile-time/exhaustiveness tests where practical.

### 9.3 Behavior

- [x] Direct command match never returns a consent challenge.
- [x] Network planner ask result returns the challenge.
- [x] Local-only/high-risk/persistent-block outcomes remain terminal local blocks, not override challenges.
- [x] No network request occurs before the challenge.
- [x] Voice and typed command paths surface the same outcome.

---

## 10. Add the consent-response command

### 10.1 Command policy and registration

- [x] Add `submit_remote_planner_consent_response`.
- [x] Add it to `tauri::generate_handler!`.
- [x] Add it to the exhaustive direct-command registry.
- [x] Classify it as a user-gesture-required privacy/config/runtime operation.
- [x] Mark whether each decision mutates config or runtime state.
- [x] Ensure the registry parity tests fail without the entry.

### 10.2 Decision contract

- [x] Add `AllowOnce`.
- [x] Add `AllowSession`.
- [x] Add `AllowPersistent`.
- [x] Add `BlockPersistent`.
- [x] Add `Deny`.
- [x] Use stable serialized values.
- [x] Reject unknown values.

### 10.3 Response validation

- [x] Validate request ID format if applicable.
- [x] Validate challenge ID.
- [x] Validate challenge digest.
- [x] Validate expiry.
- [x] Reject replay.
- [x] Revalidate runtime state.
- [x] Revalidate page origin.
- [x] Revalidate endpoint scope.
- [x] Revalidate profile destination.
- [x] Revalidate network mode.
- [x] Revalidate policy version.
- [x] Revalidate high-risk classification.
- [x] Revalidate persistent block state.

### 10.4 Decision application

- [x] `Deny` performs no network I/O and clears pending state.
- [x] `BlockPersistent` persists origin-wide block before returning.
- [x] `BlockPersistent` performs no network I/O for the pending request.
- [x] `AllowOnce` installs/consumes exact one-shot authorization.
- [x] `AllowSession` installs a runtime-only exact-scope grant.
- [x] `AllowPersistent` persists exact origin/destination/version allow.
- [x] Persistence failure returns `remote_data_consent_persist_failed`.
- [x] Persistence failure does not install an effective grant.
- [x] Persistence failure does not send the request.
- [x] Successful allow resumes exactly the pending sanitized request.

### 10.5 Locking and network behavior

- [x] Consume/remove pending consent atomically under lock.
- [x] Move the prepared request out of `AppCore` before releasing the lock.
- [x] Release `AppCore` lock before network I/O.
- [x] Prevent concurrent duplicate responses from obtaining the request twice.
- [x] Preserve bounded replanning and action validation after response.
- [x] Ensure cancellation/error leaves no reusable pending request.

---

## 11. Integrate runtime-state invalidation

### 11.1 Invalidation matrix

- [x] Page ID change invalidates.
- [x] Page/document generation change invalidates.
- [x] Normalized origin change invalidates.
- [x] Planner endpoint scheme change invalidates.
- [x] Planner endpoint host change invalidates.
- [x] Planner endpoint port change invalidates.
- [x] Planner endpoint path-prefix change invalidates.
- [x] Network mode change invalidates.
- [x] Persistent block addition invalidates.
- [x] High-risk classification change invalidates.
- [x] Privacy-policy version change invalidates.
- [x] Relevant safety/config state change invalidates when it changes the prepared request contract.
- [x] Unrelated read-only UI state does not invalidate unnecessarily.

### 11.2 Tests

- [x] Navigate after challenge, then allow: reject.
- [x] Refresh/replace page model after challenge: reject.
- [x] Change endpoint after challenge: reject.
- [x] Toggle local-only after challenge: reject.
- [x] Add persistent block after challenge: reject.
- [x] Let challenge expire: reject.
- [x] Change unrelated narration cursor where payload/state contract does not require invalidation: document and test expected behavior.

---

## 12. Runtime status and settings API

### 12.1 Status contract

- [x] Add `RemotePlannerPrivacyStatus`.
- [x] Include global network mode.
- [x] Include endpoint loopback status.
- [x] Include current normalized page origin.
- [x] Include effective decision.
- [x] Include bounded reason code.
- [x] Include persistent rule decision if present.
- [x] Include session-grant-active flag.
- [x] Include safe pending-challenge summary.
- [x] Include policy version.
- [x] Include persistent rule count.
- [x] Include stale allow count.
- [x] Do not expose challenge digest unless required for response wiring.
- [x] Do not expose sanitized payload or content.

### 12.2 Settings commands

- [x] Replace or evolve `set_remote_planner_privacy_settings`.
- [x] Add global network mode update.
- [x] Add persistent rule upsert.
- [x] Add persistent rule revoke.
- [x] Add current-origin block helper.
- [x] Add clear session grants.
- [x] Add clear persistent allows while retaining blocks.
- [x] Add clear all persistent rules only with explicit confirmation.
- [x] Report changed/no-op status.
- [x] Normalize all frontend-supplied origins in Rust.
- [x] Prevent frontend-supplied endpoint scope from overriding the configured authoritative scope for allow creation.

### 12.3 Settings adapter

- [x] Replace free-form `remote_data_notice` as the primary state with typed status.
- [x] Retain human-readable notice as derived presentation text if useful.
- [x] Sanitize endpoint display.
- [x] Surface stale rules without authorizing them.
- [x] Surface migration notice.
- [x] Update agent-state/runtime-status fixtures across the whole test tree.

---

## 13. Frontend state and API wiring

### 13.1 TypeScript types

- [x] Add global network mode union.
- [x] Add persistent rule types.
- [x] Add effective decision union.
- [x] Add consent challenge types.
- [x] Add disclosure class/count types.
- [x] Add consent decision union.
- [x] Add status summary types.
- [x] Update Tauri command result types.

### 13.2 API wrappers

- [x] Add consent-response invocation wrapper.
- [x] Add mode/rule management wrappers.
- [x] Add clear-session-grants wrapper.
- [x] Add rule revoke wrapper.
- [x] Keep raw challenge payload out of logging instrumentation.
- [x] Preserve request IDs and error typing.

### 13.3 Panel/global state

- [x] Add only safe challenge metadata to frontend state.
- [x] Do not store sanitized planner input.
- [x] Do not store raw page/OCR/tool/skill content for the dialog.
- [x] Add submission-busy state.
- [x] Add expiry state.
- [x] Clear challenge state after response.
- [x] Clear challenge state after navigation/state-invalid response.
- [x] Clear challenge state on application reset/unmount.
- [x] Update runtime refresh mapping.

### 13.4 Outcome handling

- [x] Typed commands handle `NeedsRemoteDataConsent`.
- [x] Voice commands handle `NeedsRemoteDataConsent`.
- [x] Replanning paths handle consent outcome correctly.
- [x] Duplicate outcomes do not open duplicate dialogs.
- [x] Terminal local-only/high-risk/block errors show guidance rather than a resumable allow dialog.

---

## 14. Build the just-in-time consent UI

### 14.1 Dialog content

- [x] Show normalized page origin.
- [x] Show sanitized endpoint display.
- [x] Show provider/profile and model label.
- [x] Show each disclosure category.
- [x] Show bounded counts/byte estimate.
- [x] State that data is sanitized, not anonymous.
- [x] State that action confirmation remains separate.
- [x] Show expiry/expired state.
- [x] Do not show content previews.

### 14.2 Decision controls

- [x] Add `Allow this request`.
- [x] Add `Allow for this session`.
- [x] Add `Always allow for this site`.
- [x] Add `Keep this site local`.
- [x] Add `Cancel`.
- [x] Do not make an allow control the implicit default.
- [x] Disable controls while submitting.
- [x] Prevent double submission.
- [x] Handle persistence error without closing into an allowed state.

### 14.3 Accessibility

- [x] Use a real modal/dialog semantic or equivalent focused region.
- [x] Set accessible title and description relationships.
- [x] Trap focus while open.
- [x] Return focus to the invoking control.
- [x] Escape performs deny/cancel.
- [x] Screen-reader labels distinguish once/session/persistent decisions.
- [x] Status updates use appropriate live regions without repetition.
- [x] Do not rely on color alone.
- [x] Support keyboard-only operation.
- [x] Ensure zoom/reflow and high-contrast behavior remain usable.

### 14.4 Voice flow

- [x] Announce a concise consent summary without reading raw page content.
- [x] Permit accessible decision through the existing UI controls.
- [x] Resume the exact pending request after valid consent.
- [x] Do not require repeating the transcript unless the challenge expired or state changed.

---

## 15. Redesign planner privacy settings

### 15.1 Global mode UI

- [x] Replace the two primary booleans with one mode selector.
- [x] Explain `Local only`.
- [x] Explain `Ask for each site`.
- [x] Explain advanced broad sanitized-network mode.
- [x] Require explicit confirmation before selecting broad allow if appropriate.
- [x] Show loopback behavior separately.

### 15.2 Current-origin card

- [x] Show current normalized origin.
- [x] Show effective decision.
- [x] Show destination binding for an active allow.
- [x] Add `Keep current site local`.
- [x] Add `Allow current site` only when policy permits and with destination display.
- [x] Add revoke control for current rule.
- [x] Disable persistent controls for opaque/non-HTTP(S) origins.
- [x] Show high-risk block as non-overridable.

### 15.3 Structured rule management

- [x] Replace the textarea as the primary rule UI.
- [x] List normalized origin.
- [x] List allow/block decision.
- [x] For allows, list sanitized destination scope.
- [x] List active/stale status.
- [x] Add revoke action.
- [x] Add clear persistent allows action.
- [x] Add clear session grants action.
- [x] Add explicit clear-all action only with confirmation.
- [x] Keep manual origin entry only as an advanced fallback if retained.
- [x] Validate manual entry through backend, not frontend alone.

### 15.4 Privacy status badge

- [x] Add always-visible planner privacy status where users initiate commands.
- [x] Show on-device/loopback state.
- [x] Show ask state.
- [x] Show session/persistent/global allow state.
- [x] Show local-only/origin/high-risk block state.
- [x] Make status screen-reader accessible.

---

## 16. High-risk and opaque-origin behavior

### 16.1 High-risk reasons

- [x] Expose bounded authentication reason.
- [x] Expose bounded payment reason.
- [x] Expose bounded identity reason.
- [x] Expose bounded health reason.
- [x] Expose bounded wallet reason.
- [x] Expose bounded administrative reason.
- [x] Expose bounded sensitive-field reason.
- [x] Do not expose matched raw text/field values.

### 16.2 Non-overridable UI

- [x] Show no allow controls.
- [x] Explain local/direct alternatives.
- [x] Show current origin safely.
- [x] Do not imply a product failure.
- [x] Ensure repeated commands do not repeatedly prompt.

### 16.3 Opaque/unsupported origins

- [x] Block `file:`.
- [x] Block `data:`.
- [x] Block `about:`/opaque origins.
- [x] Block malformed URLs.
- [x] Block missing-host URLs.
- [x] Define behavior for browser-internal pages.
- [x] Keep direct commands available where safe.
- [x] Do not allow persistent rules for opaque origins.

---

## 17. Rust unit and integration tests

### 17.1 Config/type tests

- [x] Enum serialization tests.
- [x] Rule validation tests.
- [x] Rule conflict tests.
- [x] Rule limit tests.
- [x] Deterministic ordering tests.
- [x] Stale-version tests.
- [x] Migration tests from legacy config.

### 17.2 Origin tests

- [x] Normalize case/default port.
- [x] Preserve non-default port.
- [x] Reject path/query/fragment.
- [x] Reject userinfo.
- [x] Reject non-HTTP(S).
- [x] Reject opaque origin.
- [x] Test IPv4 and IPv6 forms.
- [x] Test IDNA normalization.
- [x] Test Unicode confusable input through URL parsing behavior.

### 17.3 Policy tests

- [x] Full precedence table.
- [x] Global allow versus block.
- [x] High-risk versus every grant.
- [x] Local-only versus every grant.
- [x] Exact destination binding.
- [x] Policy-version binding.
- [x] Session/once behavior.

### 17.4 Challenge tests

- [x] No raw data in challenge JSON.
- [x] Digest field binding tests.
- [x] Expiry tests.
- [x] Replay tests.
- [x] Wrong ID tests.
- [x] Wrong digest tests.
- [x] State-change tests.
- [x] Destination-change tests.
- [x] Policy-change tests.
- [x] Persistent-block-added-after-challenge test.

### 17.5 Network request-count tests

- [x] Ask mode: zero requests before consent.
- [x] Deny: zero requests.
- [x] Block persistent: zero requests.
- [x] High-risk: zero requests.
- [x] Origin block: zero requests.
- [x] Valid allow once: exactly one request.
- [x] Double response: still exactly one request.
- [x] Session grant: one request per command, no extra consent request.
- [x] Loopback: request allowed without network-remote consent challenge.

### 17.6 Lock/state tests

- [x] Consent response releases `AppCore` lock before network wait.
- [x] Duplicate responses cannot both consume pending request.
- [x] Pending request replacement is bounded.
- [x] Expired pending request is dropped.
- [x] Serialized app state omits pending request.
- [x] Reconstructed app core has no session grants.

### 17.7 Existing behavior regression

- [x] Direct read/navigation/status commands still work.
- [x] Existing sanitization tests remain green.
- [x] Existing prompt-injection tests remain green.
- [x] Existing deterministic action policy remains green.
- [x] Existing confirmation digest/replay tests remain green.
- [x] Existing endpoint-bound credential tests remain green.
- [x] Existing fallback/security scanners remain green.

---

## 18. Frontend and accessibility tests

### 18.1 Rendering tests

- [x] Render each effective decision.
- [x] Render challenge origin/endpoint/profile/model.
- [x] Render disclosure categories/counts.
- [x] Confirm no raw content preview.
- [x] Render expired challenge.
- [x] Render persistence error.
- [x] Render high-risk non-overridable block.
- [x] Render stale persistent allow.

### 18.2 Interaction tests

- [x] Allow once invokes correct decision.
- [x] Allow session invokes correct decision.
- [x] Allow persistent invokes correct decision.
- [x] Block persistent invokes correct decision.
- [x] Cancel invokes deny/clears challenge.
- [x] Double click submits once.
- [x] Busy state disables all controls.
- [x] Backend mismatch error clears/refreshes stale dialog.
- [x] Rule revoke updates status.
- [x] Clear session grants updates status.

### 18.3 Accessibility tests

- [x] Dialog role/name/description.
- [x] Initial focus.
- [x] Focus trap.
- [x] Escape behavior.
- [x] Return focus.
- [x] Keyboard button order.
- [x] Distinct accessible labels for duration.
- [x] Live-region behavior.
- [x] Status not color-only.
- [x] High-risk block has no hidden allow control.

### 18.4 State privacy tests

- [x] Redux/panel state contains no sanitized payload.
- [x] Actions contain no raw page/OCR/tool/skill content.
- [x] Challenge state clears after response.
- [x] Challenge state clears after expiry/state mismatch.
- [x] Production instrumentation does not log consent payloads.

---

## 19. Scanner and evidence enforcement

### 19.1 Sensitive diagnostics

- [x] Extend `scripts/check-sensitive-diagnostics.py` for pending consent types.
- [x] Scan challenge/status contracts for raw payload fields.
- [x] Scan frontend state/actions for raw payload storage.
- [x] Add safe fixtures and scanner self-tests.
- [x] Keep normalized origin and endpoint display allowed only in approved fields.

### 19.2 Direct-command policy evidence

- [x] Add new consent-response handler to registry parity.
- [x] Add user-gesture requirement evidence.
- [x] Add config/runtime mutation classification evidence.
- [x] Add source/behavior evidence that page-context network calls require prepared authorization.

### 19.3 Fallback inventory

- [x] Run exact fallback scanner after refactor.
- [x] Inventory any new reviewed fallback.
- [x] Prefer typed errors over new privacy fallbacks.
- [x] Ensure no `.ok()`/default path can authorize a request.

### 19.4 Focused CI target

- [x] Add a focused Rust integration target for consent policy/challenge/network-count behavior.
- [x] Run it before the full Rust suite in permanent CI.
- [x] Keep the full all-feature Rust suite.
- [x] Keep frontend lint/UI/build.

---

## 20. Documentation and reconciliation

- [x] Update `docs/SPECS.md`.
- [x] Update planner setup documentation.
- [x] Update privacy disclosure documentation.
- [x] Update `config.example.toml` comments.
- [x] Document migration.
- [x] Document data categories.
- [x] Document sanitization limitations.
- [x] Document loopback versus network behavior.
- [x] Document global mode precedence.
- [x] Document persistent block behavior.
- [x] Document destination-bound allow behavior.
- [x] Document policy-version invalidation.
- [x] Document high-risk non-override behavior.
- [x] Document revocation and session clearing.
- [x] Update threat model for consent spoofing and stale consent.
- [x] Reconcile BBCR-003 checkboxes and acceptance criteria accurately.
- [x] Update post-Batch-8 reconciliation if its remaining-boundary statement changes.
- [x] Create implementation report at:
  - [x] `docs/BLIND_BROWSER_REMOTE_DATA_CONSENT_ORIGIN_PRIVACY_IMPLEMENTATION_REPORT_2026-08-03.md`

---

## 21. Validation sequence

### 21.1 Focused validation

- [x] Config migration tests.
- [x] Origin normalization tests.
- [x] Pure policy evaluator tests.
- [x] Challenge lifecycle tests.
- [x] Network request-count integration tests.
- [x] Consent-response handler tests.
- [x] Frontend consent UI tests.
- [x] Accessibility tests.
- [x] Privacy scanner self-tests.

### 21.2 Repository validation

- [x] `python3 scripts/check-silent-fallbacks.py`
- [x] `python3 scripts/check-security-fallbacks.py`
- [x] `python3 scripts/check-security-fallback-inventory.py --self-test`
- [x] `python3 scripts/check-security-fallback-inventory.py`
- [x] `python3 scripts/check-sensitive-diagnostics.py`
- [x] `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`
- [x] `cargo check --manifest-path src-tauri/Cargo.toml`
- [x] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- [x] focused remote-data consent integration target
- [x] `cargo test --manifest-path src-tauri/Cargo.toml --all-features`
- [x] `source ./fix-node-version.sh && pnpm lint`
- [x] `source ./fix-node-version.sh && pnpm test:ui`
- [x] `source ./fix-node-version.sh && pnpm build`
- [x] whitespace/diff validation

### 21.3 Adversarial scenarios

- [x] Malicious page cannot create consent automatically.
- [x] High-risk page cannot display a usable allow control.
- [x] Page navigation cannot reuse consent challenge.
- [x] Endpoint change cannot reuse consent.
- [x] Policy version change cannot reuse consent.
- [x] Persistent allow cannot authorize another destination.
- [x] Persistent block applies after destination change.
- [x] Duplicate frontend response cannot send twice.
- [x] Persistence failure cannot send.
- [x] Malformed origin cannot create a rule.
- [x] Raw page content cannot enter challenge/status/log/frontend state.

---

## 22. Cleanup and final closure

- [x] Remove temporary workflows.
- [x] Remove patch generators.
- [x] Remove diagnostic-only helpers.
- [x] Remove test bypasses and broad allow flags.
- [x] Remove sensitive test artifacts.
- [x] Confirm no temporary files remain through repository search.
- [x] Confirm final diff contains only intended source/test/doc changes.
- [x] Complete every applicable checkbox.
- [x] Mark non-selected alternatives explicitly rather than deleting them.
- [x] Append exact evidence without replacing this task tree.
- [x] Commit final documentation/evidence.
- [x] Require `ci/permanent` success on the exact final SHA.
- [x] Do not mutate the final validated SHA after signoff.

---

## 22A. Stage 1 foundation closeout — 2026-08-04

This is a bounded partial closeout. It does **not** complete the full remote-data consent and origin-privacy milestone.

### Completed scope

- Versioned global network modes and migration-compatible privacy settings.
- Destination- and policy-version-bound persistent allows.
- Origin-wide persistent blocks with block-first precedence.
- Normalized HTTP(S) origin and endpoint validation, deterministic sorting/deduplication, and a 256-rule limit.
- Idempotent legacy migration and bounded migration notice.
- Pure deterministic policy evaluation for loopback, local-only, unknown origin, high-risk context, persistent block, session grant, persistent allow, broad allow, and ask mode.
- Enforcement before remote planner serialization progression.
- Privacy settings included in the relevant runtime configuration fingerprint.
- Repository-wide fixture repair and complete permanent validation.

### Exact Stage 1 evidence

- Starting baseline: `043c788346cc9b39942f4198f11c199aaea69ddf`
- Starting CI: run `30886133291`, job `91917696317`, `success`
- Primary implementation: `e6210ae423fb0c5fe08cea9ddc68f463e96b823f`
- Strict-Clippy repair: `14216226b223c092e1a4ff5da5b29c8129f67527`
- Fixture repair: `158672218048f4482879232d7ffc0ea779e9bd07`
- Trigger-free implementation SHA: `ee967b2fb0d23a762bb8316f369d72c987f31df6`
- Guarded repair workflow: run `30900135542`, job `91962264055`, `success`
- Permanent validation trigger: run `30927205924`, job `92052482518`, `success`
- Permanent CI on exact trigger-free implementation SHA: run `30928002322`, job `92055223608`, `success`

### Historical remaining milestone boundary at Stage 1 closeout

At the 2026-08-04 Stage 1 closeout, runtime grant storage, one-shot authorization, prepared-request-only networking, disclosure manifests, consent challenges, pending consent state, `NeedsRemoteDataConsent`, consent-response commands, runtime status/settings APIs, frontend state and accessible UI, and their adversarial integration tests remained open. Those later items are reconciled by the 2026-08-05 closure report and item-by-item reconciliation; this paragraph remains as historical Stage 1 evidence.

---

## 23. Final evidence

Fill this section with exact values during closure.

### Baseline

- Original predecessor baseline SHA: `0c0acb0d76210afc6fe40a0ebd32f50e89897d91`
- Closure documentation baseline SHA: `97fc24d80dec9275d2d5fc2d470fa220df102cce`
- Starting permanent CI run: `31044019503`
- Starting permanent CI job: `92435010766`
- Starting CI result: `success`

### Implementation

- Config/policy implementation SHA: `0beb531f963297bf0e29c559141b520ba221823c`
- Request-preparation/challenge SHA: `0beb531f963297bf0e29c559141b520ba221823c`
- Backend consent-response SHA: `0beb531f963297bf0e29c559141b520ba221823c`
- Frontend UX SHA: `0beb531f963297bf0e29c559141b520ba221823c` plus the closure interaction-evidence commit containing this reconciled TODO
- Test/scanner SHA: `0beb531f963297bf0e29c559141b520ba221823c` plus the closure interaction-evidence commit
- Documentation/reconciliation SHA: immutable Git commit containing this reconciled file; reported in the Ralph-loop completion record
- Cleanup SHA: final child commit removing bounded closure machinery; reported in the Ralph-loop completion record

### Final signoff

- Final exact SHA: immutable final `master` SHA reported in the Ralph-loop completion record
- Branch: `master`
- Permanent CI run: final exact-SHA run reported in the Ralph-loop completion record
- Permanent CI job: final exact-SHA job reported in the Ralph-loop completion record
- Permanent CI result: required `success`
- Focused consent test result: `success` on implementation run `31070751355`, job `92518011921`; rerun on final exact SHA
- Full Rust test result: `success` on implementation run; rerun on final exact SHA
- Frontend lint result: `success` on implementation run; rerun on final exact SHA
- UI test result: `success` on implementation run; rerun on final exact SHA
- Frontend build result: `success` on implementation run; rerun on final exact SHA
- Temporary machinery absent: required and verified in the final exact tree before signoff

### Final bounded statement

> The remote-data consent and origin-privacy milestone is complete only after deterministic privacy authorization prevents every unauthorized non-loopback planner request; current-origin status and just-in-time consent are implemented; allows are destination- and policy-version-bound; blocks are origin-wide; high-risk contexts remain non-overridable; migration and adversarial tests pass; temporary machinery is absent; and permanent CI succeeds on the exact final SHA. The broader BBCR remediation program remains open unless separately completed.
