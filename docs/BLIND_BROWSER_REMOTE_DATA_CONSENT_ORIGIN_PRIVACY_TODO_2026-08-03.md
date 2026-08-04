# Blind Browser Remote Data Consent and Origin Privacy TODO

**Date:** 2026-08-03  
**Repository:** `ekkus93/blind_browser`  
**Branch:** `master`  
**Reviewed baseline:** `0c0acb0d76210afc6fe40a0ebd32f50e89897d91`  
**Companion spec:** `docs/BLIND_BROWSER_REMOTE_DATA_CONSENT_ORIGIN_PRIVACY_SPEC_2026-08-03.md`  
**Parent remediation items:** BBCR-003, BBCR-006, BBCR-008, BBCR-015, and BBCR-021  
**Status:** In progress — Stage 1 versioned config, migration, origin-rule validation, deterministic policy evaluation, and pre-serialization planner enforcement are complete. Prepared requests, runtime grants, consent challenges/responses, status APIs, frontend UX, and full milestone closure remain open.
**Release boundary:** This checklist closes a focused remote-data consent and origin-privacy milestone only. It must not be used to declare the full BBCR program complete or the project production-ready.

---

## Completion rules

- [x] Work directly on `master` unless the user explicitly requests a branch, PR, or worktree.
- [x] Preserve this complete checklist through implementation and closure.
- [ ] Check an item only when source, test, scanner, documentation, or CI evidence exists on `master`.
- [ ] Do not weaken planner redaction, endpoint scoping, action policy, confirmation, runtime-state binding, prompt-injection handling, or high-risk blocking.
- [x] Treat every first-party test, scanner, compiler, Clippy, frontend, and CI failure as a real defect unless evidence proves otherwise.
- [x] No non-loopback planner request may occur before deterministic privacy authorization.
- [ ] No consent decision may authorize or reduce confirmation for a protected action.
- [ ] Do not persist raw transcript, page, OCR, tool-observation, skill, credential, or planner-payload content.
- [x] Remove all temporary workflows, generators, patch scripts, diagnostic helpers, and test bypasses before closure.
- [ ] Record exact implementation, cleanup, documentation, and final evidence SHAs.
- [ ] Record exact permanent CI run and job identifiers for the final SHA.

---

## 0. Baseline and implementation setup

- [x] Confirm latest `master` SHA before implementation.
- [x] Confirm `ci/permanent` is green for the starting SHA.
- [x] Read the companion spec completely.
- [ ] Read `docs/BLIND_BROWSER_COMPREHENSIVE_CODE_REVIEW_FIX_TODO_2026-08-01.md`.
- [ ] Read `docs/BLIND_BROWSER_POST_BATCH8_RECONCILIATION_2026-08-02.md`.
- [ ] Read the post-P8 fallback hardening spec, TODO, report, and closure.
- [x] Confirm no temporary Ralph or repair machinery remains in the starting tree.
- [x] Inventory all files expected to change before coding.
- [x] Record the expected changed-file scope in the implementation report.
- [x] Decide whether implementation needs a bounded temporary workflow.
  - [x] If used, make it exact-triggered and self-cleaning.
  - [ ] If not used, do not add one unnecessarily.

### Expected source areas

- [ ] `src-tauri/src/config/types.rs`
- [ ] `src-tauri/src/config/defaults.rs` or the current default-construction module
- [ ] `src-tauri/src/config/validation.rs`
- [ ] `src-tauri/src/config/persistence.rs`
- [ ] `src-tauri/src/app_core/planner_redaction.rs`
- [ ] `src-tauri/src/app_core/remote_planner.rs`
- [ ] `src-tauri/src/app_core/command_dispatch.rs`
- [ ] `src-tauri/src/app_core/replanning.rs`
- [ ] `src-tauri/src/app_core/runtime_config.rs`
- [ ] `src-tauri/src/app_core/settings_adapters.rs`
- [ ] `src-tauri/src/app_core/state_snapshots.rs`
- [ ] `src-tauri/src/commands/contracts/planner.rs`
- [ ] `src-tauri/src/commands/contracts/providers.rs`
- [ ] `src-tauri/src/command_handlers/safety_handlers.rs`
- [ ] `src-tauri/src/direct_command_policy.rs`
- [ ] `src-tauri/src/lib.rs`
- [ ] focused Rust unit/integration tests
- [ ] `src/tauri-types.ts`
- [ ] `src/api/providers.ts`
- [ ] `src/planner-actions.ts`
- [ ] `src/runtime-refresh.ts`
- [ ] `src/panel-state.ts`
- [ ] `src/panel-types.ts`
- [ ] `src/settings-panels/planner.tsx`
- [ ] consent UI component(s) and tests
- [ ] `config.example.toml`
- [ ] `docs/SPECS.md` and privacy/security documentation

---

## 1. Audit the current privacy and request path

### 1.1 Config and settings audit

- [x] Inspect `RemotePlannerPrivacySettings` and `HighRiskOriginPolicy`.
- [ ] Confirm current defaults for global consent, local-only, blocked origins, and high-risk policy.
- [ ] Inspect config normalization for blocked origins.
- [ ] Inspect config persistence and schema migration behavior.
- [ ] Confirm how unknown config fields and missing legacy fields are handled.
- [ ] Identify every test fixture that constructs `RemotePlannerPrivacySettings` directly.
- [ ] Identify every TypeScript fixture that assumes the current booleans/list contract.

### 1.2 Planner path audit

- [ ] Trace `resolve_command` from direct-command matching to `PlannerResolution::Remote`.
- [ ] Trace `transcribe_and_execute_command` through remote planning.
- [ ] Trace bounded replanning through the remote planner.
- [x] Confirm exactly where sanitization currently occurs.
- [x] Confirm exactly where privacy evaluation currently occurs.
- [x] Confirm no planner network client is called before privacy evaluation.
- [ ] Identify every function that accepts raw `PlannerInput` and can reach network I/O.
- [ ] Identify every place the `AppCore` mutex is released for remote work.
- [ ] Identify every place remote planner errors become `ExecutionOutcome::Aborted` or frontend errors.

### 1.3 Runtime state audit

- [ ] Inspect `AppCore` fields and serializable `AppState` fields.
- [ ] Identify where a runtime-only pending consent object can live.
- [ ] Confirm pending confirmation state patterns that can be reused safely.
- [x] Confirm runtime-state token composition and invalidation behavior.
- [ ] Identify state changes that must invalidate consent.
- [ ] Identify unrelated read-only state changes that should not invalidate consent.

### 1.4 Frontend audit

- [ ] Inspect current planner privacy settings UI.
- [ ] Inspect current manual blocked-origin textarea behavior.
- [ ] Inspect planner action error handling.
- [ ] Inspect voice command outcome handling.
- [ ] Inspect current modal/focus-management patterns.
- [ ] Inspect Redux/panel state persistence boundaries.
- [ ] Confirm whether raw transcript or planner payload data enters global frontend state.
- [ ] Identify current accessibility test helpers.

### 1.5 Document audit conclusions

- [x] Record the current control/data flow in the implementation report.
- [x] Record exact pre-network authorization insertion points.
- [x] Record migration risks.
- [ ] Record the final expected source/test/doc change set before implementation.

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
- [ ] Include normalized page origin.
- [ ] Include decision.
- [ ] Include optional endpoint scope.
- [ ] Include privacy-policy version.
- [ ] Include non-sensitive creation timestamp.
- [x] Add `REMOTE_DATA_POLICY_VERSION`.
- [x] Require `Block` rules to have no endpoint scope.
- [x] Require `Allow` rules to have an exact normalized endpoint scope.
- [x] Make persistent blocks apply across all non-loopback destinations.
- [x] Make persistent allows destination- and policy-version-bound.
- [x] Define deterministic rule identity and sort order.
- [x] Limit persistent rules to at most 256.
- [ ] Define stale allow behavior.
- [ ] Ensure stale allows are visible but non-authorizing.

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

- [ ] Replace the current privacy fields with `network_mode` and `origin_rules`.
- [x] Retain `high_risk_origin_policy`.
- [ ] Keep serialization deterministic.
- [ ] Keep config debug formatting free of sensitive data.
- [ ] Update JSON schema expectations.
- [x] Update all direct Rust fixture initializers.
- [x] Avoid partial fixture publication; search the complete Rust test tree.

### 2.4 Config validation

- [ ] Add one shared normalized HTTP(S) page-origin type/helper.
- [x] Reject paths.
- [x] Reject queries.
- [x] Reject fragments.
- [x] Reject username/password.
- [ ] Reject opaque or `null` origins.
- [x] Reject non-HTTP(S) schemes.
- [ ] Normalize scheme, host, effective port, and IDNA consistently through the URL library.
- [x] Validate endpoint scopes using `ProviderEndpointScope`.
- [x] Reject allow rules with missing endpoint scope.
- [x] Reject block rules with an endpoint scope.
- [ ] Reject zero/future unsupported policy versions.
- [x] Deduplicate exact duplicate rules deterministically.
- [ ] Define conflict handling for allow and block rules on the same origin.
  - [x] Persistent block must win.
  - [ ] Validation must not silently discard a block in favor of allow.
- [ ] Add bounded, non-secret validation errors.

---

## 3. Implement legacy configuration migration

### 3.1 Mapping

- [x] Map legacy `local_only = true` to `LocalOnly`.
- [x] Map legacy `local_only = false` and `consent = false` to `AskPerOrigin`.
- [x] Map legacy `local_only = false` and `consent = true` to `AllowSanitizedNonHighRisk`.
- [x] Convert each legacy blocked origin to an origin-wide persistent `Block` rule.
- [ ] Preserve `HighRiskOriginPolicy::Block`.
- [x] Do not manufacture destination-bound allows from global legacy consent.

### 3.2 Migration safety

- [x] Make migration idempotent.
- [ ] Validate before durable write.
- [ ] Ensure failed migration leaves the previous config intact.
- [ ] Ensure malformed legacy blocked origins fail closed.
- [x] Add a migration schema/version marker if needed.
- [ ] Preserve a safe rollback/read path for the supported migration boundary.
- [ ] Avoid writing partially migrated settings.
- [x] Add a bounded one-time migration notice to runtime/settings status.
- [x] Update `config.example.toml`.

### 3.3 Migration tests

- [ ] Test every legacy boolean combination.
- [ ] Test legacy blocked-origin conversion.
- [ ] Test duplicate legacy origins.
- [ ] Test malformed legacy origin failure.
- [x] Test migration idempotence.
- [ ] Test deterministic serialization order.
- [ ] Test new-install default.
- [x] Test existing broad consent remains broad mode rather than becoming per-destination allow.
- [ ] Test migration failure preserves old config bytes.

---

## 4. Implement a pure deterministic privacy evaluator

### 4.1 Types

- [ ] Add `RemotePlannerEffectiveDecision`.
- [x] Add `RemotePlannerDataAuthorization`.
- [ ] Add typed privacy block/ask reasons.
- [ ] Add safe public reason-code conversion.
- [x] Keep evaluator inputs explicit and immutable.
- [x] Keep evaluator independent of frontend state.

Suggested effective decisions:

- [ ] `LoopbackLocal`
- [ ] `LocalOnly`
- [ ] `HighRiskBlocked`
- [ ] `OriginBlocked`
- [ ] `AllowedGlobal`
- [ ] `AllowedPersistent`
- [ ] `AllowedSession`
- [ ] `AllowedOnce`
- [ ] `ConsentRequired`
- [ ] `OriginUnavailable`
- [ ] `PlannerUnavailable`

### 4.2 Precedence

- [ ] Invalid/missing endpoint fails before consent evaluation.
- [x] Loopback returns local authorization.
- [x] Local-only blocks all non-loopback destinations.
- [x] Unknown/opaque/non-HTTP(S) page origin blocks network page-context planning.
- [x] High-risk context blocks before all grants/allows.
- [x] Persistent block overrides global allow.
- [ ] One-shot grant requires exact challenge binding.
- [x] Session grant requires exact origin/destination/version match.
- [x] Persistent allow requires exact origin/destination/version match.
- [x] Broad global allow permits only sanitized non-high-risk context.
- [x] Ask mode returns a challenge requirement.
- [x] No fallback path silently authorizes transmission.

### 4.3 Pure evaluator tests

- [ ] Create a table-driven test for every precedence branch.
- [ ] Test local-only versus persistent allow.
- [ ] Test high-risk versus every allow type.
- [ ] Test persistent block versus broad global allow.
- [ ] Test exact persistent allow.
- [ ] Test scheme change.
- [ ] Test host change.
- [ ] Test effective-port change.
- [ ] Test endpoint path-prefix change.
- [ ] Test policy-version change.
- [ ] Test expired grant.
- [ ] Test one-shot remaining-use behavior.
- [ ] Test no-rule ask behavior.
- [ ] Test unknown origin.
- [ ] Test malformed rule input cannot authorize.

---

## 5. Add runtime-only ephemeral grants

### 5.1 Grant representation

- [ ] Add `EphemeralConsentKind::Once`.
- [ ] Add `EphemeralConsentKind::Session`.
- [ ] Add `RemotePlannerEphemeralGrant`.
- [ ] Bind page origin.
- [ ] Bind endpoint scope.
- [ ] Bind policy version.
- [ ] Bind one-shot challenge digest.
- [ ] Add expiry.
- [ ] Add atomic remaining-use count for one-shot grants.
- [ ] Keep grant storage runtime-only.

### 5.2 Lifecycle

- [ ] Clear all grants on application exit naturally by not persisting them.
- [ ] Clear grants when network mode becomes `LocalOnly`.
- [ ] Make endpoint changes invalidate destination-bound grants.
- [ ] Make policy-version changes invalidate grants.
- [ ] Remove expired grants before evaluation.
- [ ] Consume one-shot grant exactly once.
- [ ] Prevent concurrent duplicate consumption.
- [ ] Bound grant count.
- [ ] Deduplicate matching session grants.
- [ ] Avoid logging full grant structures if they contain challenge digests.

### 5.3 Tests

- [ ] Session grant survives multiple matching requests in one process.
- [ ] Session grant does not survive reconstructed `AppCore`.
- [ ] Session grant does not match another origin.
- [ ] Session grant does not match another destination.
- [ ] One-shot grant succeeds once.
- [ ] One-shot replay fails.
- [ ] Mode change invalidates grants.
- [ ] Endpoint change invalidates grants.
- [ ] Expired grants are removed.

---

## 6. Split request preparation from network sending

### 6.1 Preparation boundary

- [ ] Add `prepare_remote_planner_request` or equivalent.
- [ ] Move destination validation into the preparation stage.
- [ ] Move sanitization into the preparation stage.
- [ ] Calculate disclosure classes.
- [ ] Calculate disclosure counts.
- [ ] Calculate sanitized serialized byte estimate.
- [ ] Calculate sanitized payload digest.
- [ ] Evaluate deterministic privacy.
- [ ] Return an authorized prepared request or a typed consent requirement.
- [ ] Ensure preparation performs no network I/O.

### 6.2 Prepared request type

- [ ] Add `PreparedRemotePlannerRequest`.
- [ ] Require `RemotePlannerDataAuthorization` in its constructor.
- [ ] Make fields private where practical.
- [ ] Prevent direct construction outside the privacy boundary.
- [ ] Store only sanitized planner input.
- [ ] Include normalized endpoint scope.
- [ ] Include profile name/profile snapshot.
- [ ] Include available tools and active skill names needed for output validation.
- [ ] Avoid raw `PlannerInput` storage.

### 6.3 Network sender

- [ ] Change the network sender to accept only `PreparedRemotePlannerRequest`.
- [ ] Remove or make private any network function accepting raw `PlannerInput`.
- [ ] Reject missing/invalid authorization defensively.
- [ ] Preserve endpoint-bound credential resolution.
- [ ] Preserve redirect refusal.
- [ ] Preserve planner timeout behavior currently implemented.
- [ ] Preserve response parsing and semantic validation.
- [ ] Preserve lock release before network I/O.

### 6.4 Structural evidence tests

- [ ] Add a test proving raw `PlannerInput` cannot reach the network sender API.
- [ ] Add a test proving ask mode increments no test-server request count.
- [ ] Add a test proving allowed prepared request increments exactly once.
- [ ] Add direct-command evidence showing page-context paths still pass through the privacy preparation boundary.

---

## 7. Add typed consent challenge contracts

### 7.1 Disclosure types

- [ ] Add `RemotePlannerDisclosureClass`.
- [ ] Include user transcript.
- [ ] Include page origin/sanitized URL.
- [ ] Include selected page regions.
- [ ] Include selected element metadata.
- [ ] Include OCR-derived regions.
- [ ] Include tool-observation summaries.
- [ ] Include skill summaries.
- [ ] Include trusted runtime contracts.
- [ ] Add deterministic ordering.

### 7.2 Disclosure counts

- [ ] Add selected region count.
- [ ] Add selected element count.
- [ ] Add OCR-derived region count.
- [ ] Add tool-history count.
- [ ] Add skill-summary count.
- [ ] Add sanitized serialized byte estimate.
- [ ] Do not include content excerpts.

### 7.3 Challenge type

- [ ] Add `RemotePlannerConsentChallenge`.
- [ ] Add random challenge ID.
- [ ] Add challenge digest.
- [ ] Add request ID.
- [ ] Add normalized page origin.
- [ ] Add sanitized endpoint display.
- [ ] Add exact endpoint scope.
- [ ] Add profile name.
- [ ] Add model label.
- [ ] Add policy version.
- [ ] Add disclosure classes/counts.
- [ ] Add expiry.
- [ ] Add available decision flags.
- [ ] Ensure serialization excludes raw payload.

### 7.4 Challenge digest

- [ ] Define canonical challenge-manifest serialization.
- [ ] Bind request ID.
- [ ] Bind page origin.
- [ ] Bind endpoint scope.
- [ ] Bind policy version.
- [ ] Bind disclosure classes/counts.
- [ ] Bind sanitized payload digest.
- [ ] Bind relevant runtime-state token/digest.
- [ ] Use stable hashing already approved in the repository or add an appropriate crate deliberately.
- [ ] Ensure field reorder cannot alter semantic equality unexpectedly.
- [ ] Add mutation tests for each bound field.

---

## 8. Add runtime-only pending consent state

### 8.1 Pending object

- [ ] Add `PendingRemotePlannerConsent` to `AppCore`.
- [ ] Keep it out of serializable `AppState`.
- [ ] Keep it out of public runtime status except a safe summary.
- [ ] Store the challenge.
- [ ] Store runtime-state token.
- [ ] Store sanitized payload digest.
- [ ] Store sanitized input.
- [ ] Store profile/destination snapshot.
- [ ] Store available tools/active skills required for validation.
- [ ] Do not store unrestricted page model.
- [ ] Do not store unrestricted transcript.

### 8.2 Bounded lifecycle

- [ ] Permit at most one pending consent request.
- [ ] Define replacement behavior when a new remote request arrives.
- [ ] Drop the old pending request when replaced.
- [ ] Use a 120-second initial expiry unless tests justify another value.
- [ ] Clear pending consent on deny.
- [ ] Clear pending consent on invalid response.
- [ ] Clear pending consent on state mismatch.
- [ ] Clear pending consent on endpoint/mode change.
- [ ] Clear pending consent after successful consumption.
- [ ] Clear pending consent after network send starts so duplicate response cannot resend.

### 8.3 Serialization/redaction tests

- [ ] Serialized `AppState` contains no pending consent payload.
- [ ] `Debug`/diagnostic formatting contains no raw sanitized input.
- [ ] Runtime status exposes only challenge summary.
- [ ] Sensitive diagnostic scanner covers pending consent types.
- [ ] Frontend state receives no raw pending payload.

---

## 9. Add `NeedsRemoteDataConsent` execution outcome

### 9.1 Rust contract

- [ ] Add `ExecutionOutcome::NeedsRemoteDataConsent`.
- [ ] Include existing execution trace shape.
- [ ] Include typed challenge.
- [ ] Update exhaustive matches in Rust.
- [ ] Update state application behavior.
- [ ] Ensure it does not create pending protected-action confirmation state.
- [ ] Ensure it does not report execution success.

### 9.2 TypeScript contract

- [ ] Regenerate/update `ExecutionOutcome` TypeScript type.
- [ ] Add challenge/disclosure/decision types.
- [ ] Update exhaustive frontend switches.
- [ ] Add compile-time/exhaustiveness tests where practical.

### 9.3 Behavior

- [ ] Direct command match never returns a consent challenge.
- [ ] Network planner ask result returns the challenge.
- [ ] Local-only/high-risk/persistent-block outcomes remain terminal local blocks, not override challenges.
- [ ] No network request occurs before the challenge.
- [ ] Voice and typed command paths surface the same outcome.

---

## 10. Add the consent-response command

### 10.1 Command policy and registration

- [ ] Add `submit_remote_planner_consent_response`.
- [ ] Add it to `tauri::generate_handler!`.
- [ ] Add it to the exhaustive direct-command registry.
- [ ] Classify it as a user-gesture-required privacy/config/runtime operation.
- [ ] Mark whether each decision mutates config or runtime state.
- [ ] Ensure the registry parity tests fail without the entry.

### 10.2 Decision contract

- [ ] Add `AllowOnce`.
- [ ] Add `AllowSession`.
- [ ] Add `AllowPersistent`.
- [ ] Add `BlockPersistent`.
- [ ] Add `Deny`.
- [ ] Use stable serialized values.
- [ ] Reject unknown values.

### 10.3 Response validation

- [ ] Validate request ID format if applicable.
- [ ] Validate challenge ID.
- [ ] Validate challenge digest.
- [ ] Validate expiry.
- [ ] Reject replay.
- [ ] Revalidate runtime state.
- [ ] Revalidate page origin.
- [ ] Revalidate endpoint scope.
- [ ] Revalidate profile destination.
- [ ] Revalidate network mode.
- [ ] Revalidate policy version.
- [ ] Revalidate high-risk classification.
- [ ] Revalidate persistent block state.

### 10.4 Decision application

- [ ] `Deny` performs no network I/O and clears pending state.
- [ ] `BlockPersistent` persists origin-wide block before returning.
- [ ] `BlockPersistent` performs no network I/O for the pending request.
- [ ] `AllowOnce` installs/consumes exact one-shot authorization.
- [ ] `AllowSession` installs a runtime-only exact-scope grant.
- [ ] `AllowPersistent` persists exact origin/destination/version allow.
- [ ] Persistence failure returns `remote_data_consent_persist_failed`.
- [ ] Persistence failure does not install an effective grant.
- [ ] Persistence failure does not send the request.
- [ ] Successful allow resumes exactly the pending sanitized request.

### 10.5 Locking and network behavior

- [ ] Consume/remove pending consent atomically under lock.
- [ ] Move the prepared request out of `AppCore` before releasing the lock.
- [ ] Release `AppCore` lock before network I/O.
- [ ] Prevent concurrent duplicate responses from obtaining the request twice.
- [ ] Preserve bounded replanning and action validation after response.
- [ ] Ensure cancellation/error leaves no reusable pending request.

---

## 11. Integrate runtime-state invalidation

### 11.1 Invalidation matrix

- [ ] Page ID change invalidates.
- [ ] Page/document generation change invalidates.
- [ ] Normalized origin change invalidates.
- [ ] Planner endpoint scheme change invalidates.
- [ ] Planner endpoint host change invalidates.
- [ ] Planner endpoint port change invalidates.
- [ ] Planner endpoint path-prefix change invalidates.
- [ ] Network mode change invalidates.
- [ ] Persistent block addition invalidates.
- [ ] High-risk classification change invalidates.
- [ ] Privacy-policy version change invalidates.
- [ ] Relevant safety/config state change invalidates when it changes the prepared request contract.
- [ ] Unrelated read-only UI state does not invalidate unnecessarily.

### 11.2 Tests

- [ ] Navigate after challenge, then allow: reject.
- [ ] Refresh/replace page model after challenge: reject.
- [ ] Change endpoint after challenge: reject.
- [ ] Toggle local-only after challenge: reject.
- [ ] Add persistent block after challenge: reject.
- [ ] Let challenge expire: reject.
- [ ] Change unrelated narration cursor where payload/state contract does not require invalidation: document and test expected behavior.

---

## 12. Runtime status and settings API

### 12.1 Status contract

- [ ] Add `RemotePlannerPrivacyStatus`.
- [ ] Include global network mode.
- [ ] Include endpoint loopback status.
- [ ] Include current normalized page origin.
- [ ] Include effective decision.
- [ ] Include bounded reason code.
- [ ] Include persistent rule decision if present.
- [ ] Include session-grant-active flag.
- [ ] Include safe pending-challenge summary.
- [ ] Include policy version.
- [ ] Include persistent rule count.
- [ ] Include stale allow count.
- [ ] Do not expose challenge digest unless required for response wiring.
- [ ] Do not expose sanitized payload or content.

### 12.2 Settings commands

- [ ] Replace or evolve `set_remote_planner_privacy_settings`.
- [ ] Add global network mode update.
- [ ] Add persistent rule upsert.
- [ ] Add persistent rule revoke.
- [ ] Add current-origin block helper.
- [ ] Add clear session grants.
- [ ] Add clear persistent allows while retaining blocks.
- [ ] Add clear all persistent rules only with explicit confirmation.
- [ ] Report changed/no-op status.
- [ ] Normalize all frontend-supplied origins in Rust.
- [ ] Prevent frontend-supplied endpoint scope from overriding the configured authoritative scope for allow creation.

### 12.3 Settings adapter

- [ ] Replace free-form `remote_data_notice` as the primary state with typed status.
- [ ] Retain human-readable notice as derived presentation text if useful.
- [ ] Sanitize endpoint display.
- [ ] Surface stale rules without authorizing them.
- [ ] Surface migration notice.
- [ ] Update agent-state/runtime-status fixtures across the whole test tree.

---

## 13. Frontend state and API wiring

### 13.1 TypeScript types

- [ ] Add global network mode union.
- [ ] Add persistent rule types.
- [ ] Add effective decision union.
- [ ] Add consent challenge types.
- [ ] Add disclosure class/count types.
- [ ] Add consent decision union.
- [ ] Add status summary types.
- [ ] Update Tauri command result types.

### 13.2 API wrappers

- [ ] Add consent-response invocation wrapper.
- [ ] Add mode/rule management wrappers.
- [ ] Add clear-session-grants wrapper.
- [ ] Add rule revoke wrapper.
- [ ] Keep raw challenge payload out of logging instrumentation.
- [ ] Preserve request IDs and error typing.

### 13.3 Panel/global state

- [ ] Add only safe challenge metadata to frontend state.
- [ ] Do not store sanitized planner input.
- [ ] Do not store raw page/OCR/tool/skill content for the dialog.
- [ ] Add submission-busy state.
- [ ] Add expiry state.
- [ ] Clear challenge state after response.
- [ ] Clear challenge state after navigation/state-invalid response.
- [ ] Clear challenge state on application reset/unmount.
- [ ] Update runtime refresh mapping.

### 13.4 Outcome handling

- [ ] Typed commands handle `NeedsRemoteDataConsent`.
- [ ] Voice commands handle `NeedsRemoteDataConsent`.
- [ ] Replanning paths handle consent outcome correctly.
- [ ] Duplicate outcomes do not open duplicate dialogs.
- [ ] Terminal local-only/high-risk/block errors show guidance rather than a resumable allow dialog.

---

## 14. Build the just-in-time consent UI

### 14.1 Dialog content

- [ ] Show normalized page origin.
- [ ] Show sanitized endpoint display.
- [ ] Show provider/profile and model label.
- [ ] Show each disclosure category.
- [ ] Show bounded counts/byte estimate.
- [ ] State that data is sanitized, not anonymous.
- [ ] State that action confirmation remains separate.
- [ ] Show expiry/expired state.
- [ ] Do not show content previews.

### 14.2 Decision controls

- [ ] Add `Allow this request`.
- [ ] Add `Allow for this session`.
- [ ] Add `Always allow for this site`.
- [ ] Add `Keep this site local`.
- [ ] Add `Cancel`.
- [ ] Do not make an allow control the implicit default.
- [ ] Disable controls while submitting.
- [ ] Prevent double submission.
- [ ] Handle persistence error without closing into an allowed state.

### 14.3 Accessibility

- [ ] Use a real modal/dialog semantic or equivalent focused region.
- [ ] Set accessible title and description relationships.
- [ ] Trap focus while open.
- [ ] Return focus to the invoking control.
- [ ] Escape performs deny/cancel.
- [ ] Screen-reader labels distinguish once/session/persistent decisions.
- [ ] Status updates use appropriate live regions without repetition.
- [ ] Do not rely on color alone.
- [ ] Support keyboard-only operation.
- [ ] Ensure zoom/reflow and high-contrast behavior remain usable.

### 14.4 Voice flow

- [ ] Announce a concise consent summary without reading raw page content.
- [ ] Permit accessible decision through the existing UI controls.
- [ ] Resume the exact pending request after valid consent.
- [ ] Do not require repeating the transcript unless the challenge expired or state changed.

---

## 15. Redesign planner privacy settings

### 15.1 Global mode UI

- [ ] Replace the two primary booleans with one mode selector.
- [ ] Explain `Local only`.
- [ ] Explain `Ask for each site`.
- [ ] Explain advanced broad sanitized-network mode.
- [ ] Require explicit confirmation before selecting broad allow if appropriate.
- [ ] Show loopback behavior separately.

### 15.2 Current-origin card

- [ ] Show current normalized origin.
- [ ] Show effective decision.
- [ ] Show destination binding for an active allow.
- [ ] Add `Keep current site local`.
- [ ] Add `Allow current site` only when policy permits and with destination display.
- [ ] Add revoke control for current rule.
- [ ] Disable persistent controls for opaque/non-HTTP(S) origins.
- [ ] Show high-risk block as non-overridable.

### 15.3 Structured rule management

- [ ] Replace the textarea as the primary rule UI.
- [ ] List normalized origin.
- [ ] List allow/block decision.
- [ ] For allows, list sanitized destination scope.
- [ ] List active/stale status.
- [ ] Add revoke action.
- [ ] Add clear persistent allows action.
- [ ] Add clear session grants action.
- [ ] Add explicit clear-all action only with confirmation.
- [ ] Keep manual origin entry only as an advanced fallback if retained.
- [ ] Validate manual entry through backend, not frontend alone.

### 15.4 Privacy status badge

- [ ] Add always-visible planner privacy status where users initiate commands.
- [ ] Show on-device/loopback state.
- [ ] Show ask state.
- [ ] Show session/persistent/global allow state.
- [ ] Show local-only/origin/high-risk block state.
- [ ] Make status screen-reader accessible.

---

## 16. High-risk and opaque-origin behavior

### 16.1 High-risk reasons

- [ ] Expose bounded authentication reason.
- [ ] Expose bounded payment reason.
- [ ] Expose bounded identity reason.
- [ ] Expose bounded health reason.
- [ ] Expose bounded wallet reason.
- [ ] Expose bounded administrative reason.
- [ ] Expose bounded sensitive-field reason.
- [ ] Do not expose matched raw text/field values.

### 16.2 Non-overridable UI

- [ ] Show no allow controls.
- [ ] Explain local/direct alternatives.
- [ ] Show current origin safely.
- [ ] Do not imply a product failure.
- [ ] Ensure repeated commands do not repeatedly prompt.

### 16.3 Opaque/unsupported origins

- [ ] Block `file:`.
- [ ] Block `data:`.
- [ ] Block `about:`/opaque origins.
- [ ] Block malformed URLs.
- [ ] Block missing-host URLs.
- [ ] Define behavior for browser-internal pages.
- [ ] Keep direct commands available where safe.
- [ ] Do not allow persistent rules for opaque origins.

---

## 17. Rust unit and integration tests

### 17.1 Config/type tests

- [ ] Enum serialization tests.
- [ ] Rule validation tests.
- [ ] Rule conflict tests.
- [ ] Rule limit tests.
- [ ] Deterministic ordering tests.
- [ ] Stale-version tests.
- [ ] Migration tests from legacy config.

### 17.2 Origin tests

- [ ] Normalize case/default port.
- [ ] Preserve non-default port.
- [ ] Reject path/query/fragment.
- [ ] Reject userinfo.
- [ ] Reject non-HTTP(S).
- [ ] Reject opaque origin.
- [ ] Test IPv4 and IPv6 forms.
- [ ] Test IDNA normalization.
- [ ] Test Unicode confusable input through URL parsing behavior.

### 17.3 Policy tests

- [ ] Full precedence table.
- [ ] Global allow versus block.
- [ ] High-risk versus every grant.
- [ ] Local-only versus every grant.
- [ ] Exact destination binding.
- [ ] Policy-version binding.
- [ ] Session/once behavior.

### 17.4 Challenge tests

- [ ] No raw data in challenge JSON.
- [ ] Digest field binding tests.
- [ ] Expiry tests.
- [ ] Replay tests.
- [ ] Wrong ID tests.
- [ ] Wrong digest tests.
- [ ] State-change tests.
- [ ] Destination-change tests.
- [ ] Policy-change tests.
- [ ] Persistent-block-added-after-challenge test.

### 17.5 Network request-count tests

- [ ] Ask mode: zero requests before consent.
- [ ] Deny: zero requests.
- [ ] Block persistent: zero requests.
- [ ] High-risk: zero requests.
- [ ] Origin block: zero requests.
- [ ] Valid allow once: exactly one request.
- [ ] Double response: still exactly one request.
- [ ] Session grant: one request per command, no extra consent request.
- [ ] Loopback: request allowed without network-remote consent challenge.

### 17.6 Lock/state tests

- [ ] Consent response releases `AppCore` lock before network wait.
- [ ] Duplicate responses cannot both consume pending request.
- [ ] Pending request replacement is bounded.
- [ ] Expired pending request is dropped.
- [ ] Serialized app state omits pending request.
- [ ] Reconstructed app core has no session grants.

### 17.7 Existing behavior regression

- [ ] Direct read/navigation/status commands still work.
- [ ] Existing sanitization tests remain green.
- [ ] Existing prompt-injection tests remain green.
- [ ] Existing deterministic action policy remains green.
- [ ] Existing confirmation digest/replay tests remain green.
- [ ] Existing endpoint-bound credential tests remain green.
- [ ] Existing fallback/security scanners remain green.

---

## 18. Frontend and accessibility tests

### 18.1 Rendering tests

- [ ] Render each effective decision.
- [ ] Render challenge origin/endpoint/profile/model.
- [ ] Render disclosure categories/counts.
- [ ] Confirm no raw content preview.
- [ ] Render expired challenge.
- [ ] Render persistence error.
- [ ] Render high-risk non-overridable block.
- [ ] Render stale persistent allow.

### 18.2 Interaction tests

- [ ] Allow once invokes correct decision.
- [ ] Allow session invokes correct decision.
- [ ] Allow persistent invokes correct decision.
- [ ] Block persistent invokes correct decision.
- [ ] Cancel invokes deny/clears challenge.
- [ ] Double click submits once.
- [ ] Busy state disables all controls.
- [ ] Backend mismatch error clears/refreshes stale dialog.
- [ ] Rule revoke updates status.
- [ ] Clear session grants updates status.

### 18.3 Accessibility tests

- [ ] Dialog role/name/description.
- [ ] Initial focus.
- [ ] Focus trap.
- [ ] Escape behavior.
- [ ] Return focus.
- [ ] Keyboard button order.
- [ ] Distinct accessible labels for duration.
- [ ] Live-region behavior.
- [ ] Status not color-only.
- [ ] High-risk block has no hidden allow control.

### 18.4 State privacy tests

- [ ] Redux/panel state contains no sanitized payload.
- [ ] Actions contain no raw page/OCR/tool/skill content.
- [ ] Challenge state clears after response.
- [ ] Challenge state clears after expiry/state mismatch.
- [ ] Production instrumentation does not log consent payloads.

---

## 19. Scanner and evidence enforcement

### 19.1 Sensitive diagnostics

- [ ] Extend `scripts/check-sensitive-diagnostics.py` for pending consent types.
- [ ] Scan challenge/status contracts for raw payload fields.
- [ ] Scan frontend state/actions for raw payload storage.
- [ ] Add safe fixtures and scanner self-tests.
- [ ] Keep normalized origin and endpoint display allowed only in approved fields.

### 19.2 Direct-command policy evidence

- [ ] Add new consent-response handler to registry parity.
- [ ] Add user-gesture requirement evidence.
- [ ] Add config/runtime mutation classification evidence.
- [ ] Add source/behavior evidence that page-context network calls require prepared authorization.

### 19.3 Fallback inventory

- [x] Run exact fallback scanner after refactor.
- [ ] Inventory any new reviewed fallback.
- [x] Prefer typed errors over new privacy fallbacks.
- [x] Ensure no `.ok()`/default path can authorize a request.

### 19.4 Focused CI target

- [ ] Add a focused Rust integration target for consent policy/challenge/network-count behavior.
- [ ] Run it before the full Rust suite in permanent CI.
- [ ] Keep the full all-feature Rust suite.
- [ ] Keep frontend lint/UI/build.

---

## 20. Documentation and reconciliation

- [ ] Update `docs/SPECS.md`.
- [ ] Update planner setup documentation.
- [ ] Update privacy disclosure documentation.
- [x] Update `config.example.toml` comments.
- [ ] Document migration.
- [ ] Document data categories.
- [ ] Document sanitization limitations.
- [ ] Document loopback versus network behavior.
- [ ] Document global mode precedence.
- [ ] Document persistent block behavior.
- [ ] Document destination-bound allow behavior.
- [ ] Document policy-version invalidation.
- [ ] Document high-risk non-override behavior.
- [ ] Document revocation and session clearing.
- [ ] Update threat model for consent spoofing and stale consent.
- [ ] Reconcile BBCR-003 checkboxes and acceptance criteria accurately.
- [ ] Update post-Batch-8 reconciliation if its remaining-boundary statement changes.
- [x] Create implementation report at:
  - [ ] `docs/BLIND_BROWSER_REMOTE_DATA_CONSENT_ORIGIN_PRIVACY_IMPLEMENTATION_REPORT_2026-08-03.md`

---

## 21. Validation sequence

### 21.1 Focused validation

- [ ] Config migration tests.
- [ ] Origin normalization tests.
- [ ] Pure policy evaluator tests.
- [ ] Challenge lifecycle tests.
- [ ] Network request-count integration tests.
- [ ] Consent-response handler tests.
- [ ] Frontend consent UI tests.
- [ ] Accessibility tests.
- [ ] Privacy scanner self-tests.

### 21.2 Repository validation

- [ ] `python3 scripts/check-silent-fallbacks.py`
- [ ] `python3 scripts/check-security-fallbacks.py`
- [ ] `python3 scripts/check-security-fallback-inventory.py --self-test`
- [ ] `python3 scripts/check-security-fallback-inventory.py`
- [ ] `python3 scripts/check-sensitive-diagnostics.py`
- [ ] `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`
- [ ] `cargo check --manifest-path src-tauri/Cargo.toml`
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- [ ] focused remote-data consent integration target
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml --all-features`
- [ ] `source ./fix-node-version.sh && pnpm lint`
- [ ] `source ./fix-node-version.sh && pnpm test:ui`
- [ ] `source ./fix-node-version.sh && pnpm build`
- [ ] whitespace/diff validation

### 21.3 Adversarial scenarios

- [ ] Malicious page cannot create consent automatically.
- [ ] High-risk page cannot display a usable allow control.
- [ ] Page navigation cannot reuse consent challenge.
- [ ] Endpoint change cannot reuse consent.
- [ ] Policy version change cannot reuse consent.
- [ ] Persistent allow cannot authorize another destination.
- [ ] Persistent block applies after destination change.
- [ ] Duplicate frontend response cannot send twice.
- [ ] Persistence failure cannot send.
- [ ] Malformed origin cannot create a rule.
- [ ] Raw page content cannot enter challenge/status/log/frontend state.

---

## 22. Cleanup and final closure

- [x] Remove temporary workflows.
- [x] Remove patch generators.
- [x] Remove diagnostic-only helpers.
- [ ] Remove test bypasses and broad allow flags.
- [ ] Remove sensitive test artifacts.
- [x] Confirm no temporary files remain through repository search.
- [ ] Confirm final diff contains only intended source/test/doc changes.
- [ ] Complete every applicable checkbox.
- [ ] Mark non-selected alternatives explicitly rather than deleting them.
- [x] Append exact evidence without replacing this task tree.
- [ ] Commit final documentation/evidence.
- [ ] Require `ci/permanent` success on the exact final SHA.
- [ ] Do not mutate the final validated SHA after signoff.

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

### Remaining milestone boundary

Runtime grant storage, one-shot authorization, prepared-request-only networking, disclosure manifests, consent challenges, pending consent state, `NeedsRemoteDataConsent`, consent-response commands, runtime status/settings APIs, frontend state and accessible UI, and their adversarial integration tests remain open.

---

## 23. Final evidence

Fill this section with exact values during closure.

### Baseline

- Starting `master` SHA:
- Starting permanent CI run:
- Starting permanent CI job:
- Starting CI result:

### Implementation

- Config/policy implementation SHA:
- Request-preparation/challenge SHA:
- Backend consent-response SHA:
- Frontend UX SHA:
- Test/scanner SHA:
- Documentation/reconciliation SHA:
- Cleanup SHA:

### Final signoff

- Final exact SHA:
- Branch:
- Permanent CI run:
- Permanent CI job:
- Permanent CI result:
- Focused consent test result:
- Full Rust test result:
- Frontend lint result:
- UI test result:
- Frontend build result:
- Temporary machinery absent:

### Final bounded statement

> The remote-data consent and origin-privacy milestone is complete only after deterministic privacy authorization prevents every unauthorized non-loopback planner request; current-origin status and just-in-time consent are implemented; allows are destination- and policy-version-bound; blocks are origin-wide; high-risk contexts remain non-overridable; migration and adversarial tests pass; temporary machinery is absent; and permanent CI succeeds on the exact final SHA. The broader BBCR remediation program remains open unless separately completed.
