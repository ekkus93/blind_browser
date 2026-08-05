# Blind Browser Remote Data Privacy Milestone Closure TODO

**Date:** 2026-08-05  
**Repository:** `ekkus93/blind_browser`  
**Branch:** `master`  
**Reviewed closure baseline:** `97fc24d80dec9275d2d5fc2d470fa220df102cce`  
**Baseline permanent CI:** run `31044019503`, job `92435010766`, conclusion `success`  
**Companion specification:** `docs/BLIND_BROWSER_REMOTE_DATA_PRIVACY_MILESTONE_CLOSURE_SPEC_2026-08-05.md`  
**Companion specification commit:** `e100b119dc7e8ceb827f606fe5ed7379e79737a1`  
**Predecessor TODO:** `docs/BLIND_BROWSER_REMOTE_DATA_CONSENT_ORIGIN_PRIVACY_TODO_2026-08-03.md`  
**Parent remediation items:** BBCR-003, BBCR-006, BBCR-008, BBCR-015, and BBCR-021  
**Status:** Open — closure, reconciliation, residual evidence, documentation, cleanup, and exact-final-SHA signoff remain.  
**Release boundary:** Completing this checklist closes only the remote-data consent and origin-privacy milestone. It does not complete the broader BBCR program or establish general production release readiness.

---

## 0. Completion rules

- [x] Work directly on `master` unless the user explicitly requests a branch, PR, or worktree.
- [x] Start from exact baseline SHA `97fc24d80dec9275d2d5fc2d470fa220df102cce` with permanent CI success.
- [x] Preserve the predecessor specification, TODO, reports, and historical evidence documents.
- [ ] Read the companion closure specification completely before implementation.
- [ ] Read the predecessor privacy specification completely.
- [ ] Read the predecessor privacy TODO completely.
- [ ] Read the Stage 1, Stage 2A, Stage 2B, and supplemental evidence documents.
- [ ] Read the authoritative BBCR TODO and post-Batch-8 reconciliation.
- [ ] Do not bulk-check predecessor items from source-name similarity or conversational memory.
- [ ] Classify every applicable predecessor item as:
  - [ ] implemented and evidenced;
  - [ ] implemented but missing dedicated evidence;
  - [ ] genuinely open and required;
  - [ ] not selected or no longer applicable, with rationale.
- [ ] Check an item only when exact source plus test/scanner/documentation/CI evidence exists.
- [ ] Preserve deterministic fail-closed privacy behavior.
- [ ] Preserve planner redaction, endpoint scoping, action policy, immutable confirmation, runtime-state binding, prompt-injection handling, and high-risk blocking.
- [ ] Treat every first-party test, scanner, compiler, Clippy, frontend, and CI failure as a real defect unless exact evidence proves a harness assumption is wrong.
- [ ] Do not add a compatibility adapter that recreates the removed boolean/list privacy mutation path.
- [ ] Do not add a test-only broad allow, authorization bypass, fallback default, or ignored failure.
- [ ] Do not log or persist raw transcript, page, OCR, tool, skill, credential, sanitized pending draft, or complete planner payload content.
- [ ] Keep transmission consent separate from protected-action confirmation.
- [ ] Remove all temporary workflows, generators, probes, triggers, repair scripts, and diagnostics before closure.
- [ ] Require permanent CI success on the exact final SHA.
- [ ] Do not mutate the final validated SHA after signoff.

---

## 1. Record the authoritative current baseline

### 1.1 Repository state

- [ ] Confirm current `master` has not moved unexpectedly before implementation.
- [ ] Record current `master` SHA.
- [ ] Confirm no open branch or PR is required for this direct-`master` pass.
- [ ] Confirm `ci/permanent` is green on the starting SHA.
- [ ] Record starting permanent CI run and job.
- [ ] Confirm no unrelated working automation or temporary workflow is present.
- [ ] Inventory `.github/workflows` and `.github` root artifacts.
- [ ] Record the expected source/test/scanner/doc change set before editing.

### 1.2 Existing exact evidence

- [x] Stage 1 foundation evidence exists.
- [x] Stage 2A backend consent transaction evidence exists.
- [x] Stage 2B typed frontend/settings evidence exists.
- [x] Legacy frontend privacy adapter removal evidence exists.
- [x] Supplemental request-count/replay/concurrency/expiry/invalidation/persistence/hostile-state evidence exists.
- [x] Supplemental evidence source SHA is `247717dd25372b05110b6fe6d382954d88c10a9f`.
- [x] Supplemental evidence permanent CI is run `31043067601`, job `92431872883`, success.
- [x] Current documentation SHA is `97fc24d80dec9275d2d5fc2d470fa220df102cce`.
- [x] Current documentation permanent CI is run `31044019503`, job `92435010766`, success.
- [ ] Build an evidence index mapping every relevant commit/run/job to the behavior it actually proved.
- [ ] Do not claim an older run proved a test that was added later.

---

## 2. Reconcile the predecessor TODO item by item

### 2.1 Reconciliation mechanics

- [ ] Create a reconciliation table or appendix containing:
  - [ ] predecessor section/item identifier;
  - [ ] selected classification;
  - [ ] authoritative source file/function/type;
  - [ ] focused test/scanner/compile-time evidence;
  - [ ] exact SHA and permanent CI run/job;
  - [ ] remaining action or non-applicability rationale.
- [ ] Preserve every predecessor task and subtask.
- [ ] Do not delete suggested alternatives; mark them superseded or not selected.
- [ ] Correct the predecessor header/status so it no longer says implemented Stage 2A/2B features remain wholly absent.
- [ ] Reconcile the predecessor final-evidence section with exact values.
- [ ] Keep the broader BBCR boundary explicit.

### 2.2 Reconcile foundation and policy sections

- [ ] Reconcile config and settings audit items.
- [ ] Reconcile planner path audit items.
- [ ] Reconcile runtime-state audit items.
- [ ] Reconcile frontend audit items.
- [ ] Reconcile global network mode items.
- [ ] Reconcile persistent origin-rule field and validation items.
- [ ] Reconcile stale allow behavior.
- [ ] Reconcile config serialization/schema/debug-safety items.
- [ ] Reconcile legacy migration mapping and failure behavior.
- [ ] Reconcile migration tests.
- [ ] Reconcile pure evaluator types and reason codes.
- [ ] Reconcile every evaluator precedence branch.
- [ ] Reconcile table-driven policy tests.

### 2.3 Reconcile runtime and request-boundary sections

- [ ] Reconcile ephemeral grant representation and lifecycle.
- [ ] Reconcile one-shot atomic consumption.
- [ ] Reconcile session grant scope and expiry.
- [ ] Reconcile prepared-request construction and authorization.
- [ ] Reconcile network sender input restrictions.
- [ ] Reconcile disclosure classes/counts.
- [ ] Reconcile challenge fields and canonical digest binding.
- [ ] Reconcile pending consent storage and lifecycle.
- [ ] Reconcile consent-required Rust and TypeScript outcomes.
- [ ] Reconcile consent-response command registration and policy classification.
- [ ] Reconcile response validation and decision application.
- [ ] Reconcile lock release and duplicate-response prevention.
- [ ] Reconcile runtime-state invalidation matrix.

### 2.4 Reconcile status, frontend, and UI sections

- [ ] Reconcile `RemotePlannerPrivacyStatus` fields.
- [ ] Reconcile typed privacy operations.
- [ ] Reconcile authoritative status refresh/no-op reporting.
- [ ] Reconcile TypeScript contracts and wrappers.
- [ ] Reconcile frontend challenge state.
- [ ] Reconcile typed, voice, and replanning outcome handling.
- [ ] Reconcile consent dialog content and decisions.
- [ ] Reconcile consent dialog accessibility requirements.
- [ ] Reconcile voice-flow requirements.
- [ ] Reconcile privacy settings redesign.
- [ ] Reconcile current-origin card and structured rule management.
- [ ] Reconcile always-visible privacy status.
- [ ] Reconcile high-risk and opaque-origin presentation.

### 2.5 Reconcile testing, scanners, docs, and signoff sections

- [ ] Reconcile Rust config/origin/policy/challenge tests.
- [ ] Reconcile network request-count tests.
- [ ] Reconcile lock/state/reconstruction tests.
- [ ] Reconcile frontend rendering/interaction/accessibility/state-privacy tests.
- [ ] Reconcile scanner and direct-command evidence.
- [ ] Reconcile focused permanent CI wiring.
- [ ] Reconcile documentation requirements.
- [ ] Reconcile cleanup requirements.
- [ ] Reconcile exact final-evidence requirements.

---

## 3. Verify the production architecture before adding tests

### 3.1 Deterministic authorization

- [ ] Confirm Rust remains the sole privacy authority.
- [ ] Confirm frontend state cannot directly authorize transmission.
- [ ] Confirm invalid/missing planner destination fails before consent evaluation.
- [ ] Confirm loopback classification uses `ProviderEndpointScope`.
- [ ] Confirm `LocalOnly` overrides all non-loopback allows.
- [ ] Confirm high-risk classification overrides all allows and grants.
- [ ] Confirm persistent block overrides broad global allow.
- [ ] Confirm persistent allow matches exact origin, destination, and policy version.
- [ ] Confirm malformed/stale rules cannot authorize.
- [ ] Confirm no permissive default branch exists in evaluator matches.

### 3.2 Prepared-request-only sender

- [ ] Confirm sanitization occurs before authorization.
- [ ] Confirm disclosure classes/counts and payload digest derive from sanitized input.
- [ ] Confirm preparation performs no network I/O.
- [ ] Confirm the network sender accepts only an authorized prepared request.
- [ ] Confirm no public or compatibility function accepts raw `PlannerInput` and reaches network I/O.
- [ ] Confirm endpoint-bound credential behavior remains intact.
- [ ] Confirm redirect refusal and timeout behavior remain intact.
- [ ] Confirm output semantic/action validation remains intact after consent.
- [ ] Confirm network wait occurs outside the `AppCore` lock.

### 3.3 Consent transaction lifecycle

- [ ] Confirm at most one pending consent transaction exists.
- [ ] Confirm replacement removes the previous transaction.
- [ ] Confirm challenge response validates ID/digest/expiry/state/destination/policy.
- [ ] Confirm terminal response consumes or clears pending state.
- [ ] Confirm pending state is removed before network send.
- [ ] Confirm persistence occurs before persistent authorization.
- [ ] Confirm persistence failure leaves no effective grant.
- [ ] Confirm deny and persistent block perform no network I/O.
- [ ] Confirm duplicate responses cannot both obtain a prepared request.
- [ ] Confirm consent cannot reduce action confirmation.

### 3.4 Frontend authority boundary

- [ ] Confirm frontend operations are tagged typed operations only.
- [ ] Confirm the removed legacy API/callback/action exports remain absent.
- [ ] Confirm successful operations replace state from authoritative Rust status.
- [ ] Confirm failed operations remain visible and do not optimistically mutate state.
- [ ] Confirm endpoint scope for persistent allow is selected by Rust, not frontend input.
- [ ] Confirm stale challenge metadata alone cannot cause a request.

---

## 4. Close the backend request-count matrix

For each item, first locate existing exact evidence. Add a new focused test only when the current production path is not already proven.

### 4.1 Zero-request cases

- [x] Deny sends zero requests in supplemental Wry evidence.
- [x] Consent preparation alone sends zero requests before authorization in supplemental evidence.
- [ ] Persistent block decision sends zero requests.
- [ ] Pre-existing origin block sends zero requests.
- [ ] High-risk page/context sends zero requests.
- [ ] Local-only mode sends zero non-loopback requests.
- [ ] Opaque/unsupported origin sends zero requests.
- [ ] Expired response sends zero requests.
- [ ] Wrong challenge ID sends zero requests.
- [ ] Wrong challenge digest sends zero requests.
- [ ] State mismatch sends zero requests.
- [ ] Destination mismatch sends zero requests.
- [ ] Persistence failure sends zero requests.
- [ ] Replayed response sends zero additional requests.

### 4.2 Authorized request cases

- [x] Valid allow-once dispatches exactly one request.
- [x] Concurrent duplicate response produces only one authorization.
- [x] Replaying consumed allow-once cannot dispatch again.
- [ ] Session grant permits one request per matching new command without a new challenge.
- [ ] Session grant does not add an extra consent/probe request.
- [ ] Persistent exact allow permits one request per matching command.
- [ ] Broad sanitized mode permits one request for eligible non-high-risk context.
- [ ] Loopback planner proceeds without network-remote consent challenge.
- [ ] Every allowed path still uses sanitized prepared input.

### 4.3 Replacement and terminal behavior

- [x] Creating later consent work does not send the earlier request.
- [ ] Old challenge response after replacement cannot send.
- [ ] Provider/network failure leaves no reusable pending request.
- [ ] Planner parse/semantic failure leaves no reusable pending request.
- [ ] User cancellation during frontend submission cannot cause a retry send.
- [ ] Repeated frontend activation while busy results in one backend invocation.

### 4.4 Test-server requirements

- [ ] Use a bounded local test server with explicit request counters.
- [ ] Fail the test if an unexpected second connection occurs.
- [ ] Do not treat client error alone as proof of request count.
- [ ] Assert exact request path and method without logging credentials or payload content.
- [ ] Ensure proxy environment cannot redirect loopback test traffic.
- [ ] Ensure server threads terminate on all success/failure paths.
- [ ] Keep real Wry tests process-isolated where required.
- [ ] Wire every ignored closure-critical Wry test into the permanent runner.

---

## 5. Close challenge binding and invalidation evidence

### 5.1 Identity and digest validation

- [ ] Wrong challenge ID fails closed and consumes/clears according to the declared contract.
- [ ] Wrong challenge digest fails closed.
- [ ] Missing challenge fails visibly rather than becoming denial success.
- [ ] Unknown decision serialization fails closed.
- [ ] Duplicate response returns a stable missing/replayed outcome.
- [ ] Challenge digest does not appear in ambient status/state.
- [x] Challenge digest remains present in the explicit challenge response contract.

### 5.2 Digest mutation matrix

- [ ] Mutating request identity changes or invalidates the challenge.
- [ ] Mutating page origin changes or invalidates the challenge.
- [ ] Mutating endpoint scheme changes or invalidates the challenge.
- [ ] Mutating endpoint host changes or invalidates the challenge.
- [ ] Mutating endpoint effective port changes or invalidates the challenge.
- [ ] Mutating endpoint path prefix changes or invalidates the challenge.
- [ ] Mutating profile/model destination identity changes or invalidates the challenge.
- [ ] Mutating policy version changes or invalidates the challenge.
- [ ] Mutating disclosure classes changes or invalidates the challenge.
- [ ] Mutating disclosure counts changes or invalidates the challenge.
- [ ] Mutating sanitized payload digest changes or invalidates the challenge.
- [ ] Mutating relevant runtime-state binding changes or invalidates the challenge.
- [ ] Field ordering cannot change semantic equality unexpectedly.

### 5.3 Runtime invalidation matrix

- [ ] Page ID change invalidates.
- [x] Page/document generation change invalidates.
- [ ] Normalized origin change invalidates.
- [ ] Endpoint scheme change invalidates.
- [ ] Endpoint host change invalidates.
- [ ] Endpoint port change invalidates.
- [ ] Endpoint path-prefix change invalidates.
- [x] Profile/model destination change invalidates.
- [x] Network mode change invalidates.
- [x] Persistent block addition invalidates.
- [ ] High-risk classification change invalidates.
- [ ] Privacy-policy version change invalidates.
- [ ] Relevant safety/config change invalidates when it changes the prepared request contract.
- [ ] Expiry invalidates and clears pending state.
- [ ] Define one unrelated read-only UI/state change that should not invalidate, then test it.
- [ ] If current runtime-token design intentionally invalidates that change, document the conservative behavior and test it.

---

## 6. Prove restart and reconstruction behavior

### 6.1 Runtime-only grants

- [ ] Install a session grant in a real `AppCore` instance.
- [ ] Reconstruct `AppCore` against the same persisted config.
- [ ] Prove the session grant is absent.
- [ ] Install or prepare one-shot authorization state.
- [ ] Reconstruct `AppCore`.
- [ ] Prove one-shot authorization is absent.
- [ ] Confirm no runtime grant is serialized into config or `AppState`.

### 6.2 Pending consent

- [ ] Store a real pending consent transaction.
- [ ] Reconstruct `AppCore` or restart the isolated process.
- [ ] Prove pending consent is absent.
- [ ] Prove the sanitized pending draft is absent.
- [ ] Prove a stale frontend challenge cannot restore pending backend state.
- [ ] Prove submission of the stale challenge fails closed and sends zero requests.

### 6.3 Persistent rules

- [ ] Persist an exact allow successfully.
- [ ] Reconstruct `AppCore`.
- [ ] Prove the rule survives and remains exact-destination/policy-bound.
- [ ] Persist an origin-wide block successfully.
- [ ] Reconstruct `AppCore`.
- [ ] Prove the block survives and applies across non-loopback destinations.
- [ ] Prove a failed persistent write survives neither in memory nor after reconstruction.

### 6.4 Process-isolation requirements

- [ ] Prefer a process-isolated real Wry test where application singleton/global state can affect reconstruction.
- [ ] Avoid unsafe parallel mutation of `XDG_CONFIG_HOME` or equivalent process-global environment.
- [ ] Ensure temporary config directories are unique and removed.
- [ ] Do not use serialized test execution as a substitute for true restart evidence unless justified and documented.

---

## 7. Close backend serialization and diagnostic privacy

### 7.1 Persisted and runtime state

- [ ] Persisted configuration contains no pending consent object.
- [ ] Persisted configuration contains no session/one-shot grant.
- [ ] Serialized `AppState` contains no pending transaction or sanitized draft.
- [ ] Agent-state snapshots contain no sanitized draft.
- [ ] Runtime status contains only approved challenge summary metadata.
- [ ] Privacy status contains no challenge digest.
- [ ] Privacy status contains no raw or sanitized content.
- [ ] State snapshots expose normalized origins and sanitized endpoint displays only through approved fields.

### 7.2 Debug, errors, and logs

- [ ] `Debug` formatting for pending consent types does not expose sanitized input.
- [ ] Tool errors do not include transcript/page/OCR/tool/skill content.
- [ ] Persistence errors do not include config bytes or rule payload content beyond approved metadata.
- [ ] Network/provider errors do not include request payloads or raw remote response bodies.
- [ ] Failure-only logging does not expose challenge digest or pending input.
- [ ] Test assertion messages do not print full sensitive structures.
- [ ] CI logs and artifacts contain only synthetic sentinels and approved metadata.

### 7.3 Hostile-state corpus

- [x] Supplemental evidence excludes hostile transcript sentinel from tested backend serialized surfaces.
- [x] Supplemental evidence excludes internal sanitized input field/content from tested surfaces.
- [ ] Add hostile OCR-derived content coverage.
- [ ] Add hostile tool-observation-summary coverage.
- [ ] Add hostile skill-summary coverage.
- [ ] Add secret-shaped URL/query/form-value sentinels where the planner sanitizer accepts safe summaries.
- [ ] Prove explicit challenge disclosure metadata does not contain excerpts.
- [ ] Keep synthetic markers obviously non-secret.

---

## 8. Extend permanent scanner enforcement

### 8.1 Sensitive diagnostics scanner

- [ ] Inventory current `check-sensitive-diagnostics.py` coverage for consent types.
- [ ] Add positive fixture: pending sanitized input exposed in public state must fail.
- [ ] Add positive fixture: challenge/status raw content field must fail.
- [ ] Add positive fixture: challenge digest in ambient status must fail.
- [ ] Add positive fixture: frontend global state storing sanitized payload must fail.
- [ ] Add positive fixture: logging pending challenge/request content must fail.
- [ ] Add safe fixture: explicit challenge digest used only for response binding must pass.
- [ ] Add safe fixture: normalized origin/sanitized endpoint display in approved status fields must pass.
- [ ] Add or update scanner self-tests.
- [ ] Run self-test before audit in permanent CI.

### 8.2 Silent/security fallback scanners

- [ ] Search consent/privacy code for swallowed errors and default authorization.
- [ ] Search for broad `unwrap_or`/default behavior around decisions, rules, status, and pending state.
- [ ] Search for persistence failure converted into another allow class.
- [ ] Search for missing pending state converted into success.
- [ ] Search for frontend catch paths that close the dialog without an authoritative outcome.
- [ ] Add exact scanner rules only for dangerous patterns that can be expressed without excessive false positives.
- [ ] Add self-tests for every new scanner rule.
- [ ] Inventory any reviewed fallback exactly by file, expression, rationale, and test.
- [ ] Do not add broad directory or file exclusions.

### 8.3 Frontend state/action scanner

- [ ] Determine whether the existing scanner can reliably inspect TypeScript consent state/actions.
- [ ] Add a dedicated narrow scanner if the existing scanner cannot express the invariant safely.
- [ ] Reject fields named or typed as raw/sanitized planner payload in global consent state.
- [ ] Reject logging/instrumentation of challenge digest, request payload, transcript, page, OCR, tool, and skill content.
- [ ] Permit only approved challenge metadata and response-binding fields in the ephemeral dialog state.
- [ ] Add safe and unsafe fixtures.
- [ ] Wire the scanner permanently into CI.

### 8.4 Scanner quality

- [ ] Scanner output identifies exact file and line/pattern.
- [ ] Scanner failure is non-zero and cannot be ignored.
- [ ] Scanner code itself has tests for malformed input and fixture discovery.
- [ ] Scanner fixtures cannot be mistaken for production files.
- [ ] Scanner rules do not silently skip unreadable files.
- [ ] Scanner rules do not silently pass when expected source paths disappear.

---

## 9. Close frontend state privacy

### 9.1 Store shape

- [ ] Inventory every frontend field that stores remote privacy status or consent state.
- [ ] Confirm only explicit challenge metadata required for display/response is retained.
- [ ] Confirm no sanitized planner input is retained.
- [ ] Confirm no raw transcript, page, OCR, tool, or skill content is copied for the dialog.
- [ ] Confirm no full backend response object is retained when a bounded state shape suffices.
- [ ] Confirm challenge digest is not rendered or logged.
- [ ] Confirm stale endpoint scope/request ID fields are not rendered.

### 9.2 Lifecycle

- [ ] Challenge state clears after allow-once success.
- [ ] Challenge state clears after session allow success.
- [ ] Challenge state clears after persistent allow success.
- [ ] Challenge state clears after persistent block success.
- [ ] Challenge state clears after deny.
- [ ] Challenge state clears after expiry.
- [ ] Challenge state clears after authoritative state mismatch.
- [ ] Challenge state clears after destination mismatch.
- [ ] Challenge state clears on application reset/unmount.
- [ ] Challenge state is not restored from persisted frontend state after restart.
- [ ] Submission busy state clears on every terminal error path.

### 9.3 Errors and refresh

- [ ] Persistence failure remains visible and does not show allowed status.
- [ ] Missing/replayed challenge prompts authoritative refresh rather than retry with broad consent.
- [ ] Expired challenge explains that the command must be prepared again.
- [ ] Destination/state mismatch removes stale dialog controls.
- [ ] Status refresh failure remains visible and does not infer an allowed decision.
- [ ] Operation no-op remains distinguishable from failure.
- [ ] Duplicate operations remain rejected while busy.

### 9.4 Instrumentation

- [ ] Inventory production logging, analytics, Redux/dev instrumentation, and error reporting for consent state.
- [ ] Confirm no raw or sanitized content is emitted.
- [ ] Confirm challenge digest/request payload is not emitted.
- [ ] Confirm test-only debug output cannot ship in production builds.
- [ ] Add regression tests or scanners for the selected instrumentation boundary.

---

## 10. Close consent-dialog interaction and accessibility evidence

### 10.1 Semantic structure

- [x] Source uses a real dialog semantic with `aria-modal`.
- [x] Source defines accessible title and description relationships.
- [ ] Interaction test confirms the computed accessible dialog name.
- [ ] Interaction test confirms the computed accessible description.
- [ ] Interaction test confirms errors are announced through an alert/live region.
- [ ] Interaction test confirms busy/status updates do not repeat excessively.

### 10.2 Focus behavior

- [x] Source focuses cancel on open.
- [x] Source traps focus forward and backward.
- [x] Source restores focus when possible.
- [ ] Browser/DOM interaction test proves initial focus is cancel/deny.
- [ ] Browser/DOM interaction test proves forward focus wrap.
- [ ] Browser/DOM interaction test proves reverse focus wrap.
- [ ] Browser/DOM interaction test proves focus restoration.
- [ ] Test the fallback when the invoking element no longer exists.
- [ ] Test zero-focusable-element defensive behavior if reachable.

### 10.3 Keyboard and submission behavior

- [x] Source maps Escape to deny.
- [x] Source has no form or implicit default allow.
- [x] Static rendering proves allow controls have no `autofocus`.
- [x] Source disables all decision controls while submitting.
- [ ] Interaction test proves Escape invokes deny once.
- [ ] Interaction test proves Enter/Space activates only the focused control.
- [ ] Interaction test proves rapid double click submits once.
- [ ] Interaction test proves repeated keyboard activation while busy submits once.
- [ ] Interaction test proves all controls remain disabled while backend response is pending.

### 10.4 Accessible decision distinctions

- [x] Source defines distinct labels for once/session/persistent/block/deny.
- [ ] Accessibility test verifies each computed label.
- [ ] Verify visible labels and accessible labels do not contradict duration/scope.
- [ ] Verify persistent allow identifies exact site and planner scope in surrounding context.
- [ ] Verify persistent block identifies origin-wide local behavior.

### 10.5 High-risk and privacy status

- [x] Static rendering proves high-risk guidance has no decision controls.
- [ ] Interaction/DOM test proves no hidden allow control exists.
- [ ] Test local-only, ask, session, persistent, global, origin-blocked, high-risk, loopback, opaque, and unavailable statuses.
- [ ] Verify status is not communicated by color alone.
- [ ] Verify stale allow warning is announced accessibly.

### 10.6 Zoom, reflow, and contrast

- [ ] Define an executable or documented manual validation method for 200% zoom/reflow.
- [ ] Define an executable or documented validation method for high-contrast/forced-colors behavior.
- [ ] Fix overflow, clipping, focus visibility, or decision-order problems found.
- [ ] Record screenshots only if they contain no private content and are useful as evidence.
- [ ] Do not treat screenshots alone as semantic accessibility evidence.

---

## 11. Close privacy settings interaction evidence

### 11.1 Network mode

- [x] Structured settings implementation exists.
- [ ] Test all three mutually exclusive modes as an actual radio group.
- [ ] Test broad sanitized-network confirmation initial focus, Escape, trap, and return focus.
- [ ] Test cancel leaves authoritative mode unchanged.
- [ ] Test backend failure leaves mode unchanged and visible.

### 11.2 Current-site operations

- [ ] Test current-site persistent block operation.
- [ ] Test exact destination-bound current-site allow operation.
- [ ] Test loopback does not expose persistent remote allow.
- [ ] Test local-only does not expose persistent allow.
- [ ] Test opaque origin does not expose persistent rules.
- [ ] Test high-risk context does not expose allow.
- [ ] Test persistent block prevents allow control.
- [ ] Test exact revoke refreshes authoritative status.

### 11.3 Rule management

- [ ] Test active allow display includes sanitized destination.
- [ ] Test block display contains no endpoint scope.
- [ ] Test stale allow remains visible and non-authorizing.
- [ ] Test manual origin entry is normalized/validated by backend.
- [ ] Test frontend cannot submit an endpoint scope for allow creation.
- [ ] Test clear session grants.
- [ ] Test clear persistent allows retains blocks.
- [ ] Test clear all requires explicit confirmation.
- [ ] Test clear-all request always carries `confirmed: true` from the confirmed UI path.
- [ ] Test operation busy state prevents duplicate mutation.
- [ ] Test changed/no-op/error announcements.

### 11.4 Migration notice

- [ ] Test migration notice renders only when pending.
- [ ] Test notice explains broad legacy consent was not converted into destination-bound allows.
- [ ] Test acknowledgment persists authoritatively.
- [ ] Test acknowledgment failure remains visible.

---

## 12. Complete migration and origin evidence

### 12.1 Migration mapping

- [ ] Test legacy local-only mapping.
- [ ] Test legacy ask mapping.
- [ ] Test legacy broad sanitized-network mapping.
- [ ] Test legacy blocked-origin conversion.
- [ ] Test duplicate legacy origins.
- [ ] Test malformed legacy origin failure.
- [ ] Test migration idempotence.
- [ ] Test new-install default.
- [ ] Test legacy broad consent does not create destination-bound allows.
- [ ] Test migration failure preserves prior config bytes.
- [ ] Test deterministic serialization order.

### 12.2 Origin normalization

- [ ] Test scheme/host case normalization.
- [ ] Test default port normalization.
- [ ] Test non-default port preservation.
- [ ] Test IPv4.
- [ ] Test IPv6.
- [ ] Test IDNA normalization.
- [ ] Test Unicode/confusable input through the URL library.
- [ ] Reject path.
- [ ] Reject query.
- [ ] Reject fragment.
- [ ] Reject userinfo.
- [ ] Reject non-HTTP(S).
- [ ] Reject opaque/`null` origin.
- [ ] Reject malformed/missing-host URLs.
- [ ] Confirm bounded non-secret validation errors.

### 12.3 Rule validation and conflicts

- [ ] Test allow requires exact endpoint scope.
- [ ] Test block rejects endpoint scope.
- [ ] Test unsupported/future policy version.
- [ ] Test deterministic duplicate handling.
- [ ] Test allow/block conflict keeps block authoritative.
- [ ] Test 256-rule limit.
- [ ] Test stale allow remains visible but cannot authorize.
- [ ] Test endpoint scheme/host/port/path change makes allow stale.

---

## 13. Preserve existing security regressions

- [ ] Existing planner sanitization tests remain green.
- [ ] Existing hostile page/OCR tests remain green.
- [ ] Existing direct-command policy evidence remains green.
- [ ] Existing deterministic action-policy tests remain green.
- [ ] Existing immutable confirmation digest/replay tests remain green.
- [ ] Existing runtime snapshot revalidation tests remain green.
- [ ] Existing endpoint-bound credential tests remain green.
- [ ] Existing redirect refusal tests remain green.
- [ ] Existing fallback scanners remain green.
- [ ] Existing sensitive-diagnostics scanner remains green.
- [ ] Existing config atomicity/durability tests remain green.
- [ ] Existing frontend lint/UI/build remain green.
- [ ] No new privacy test changes production safety policy solely to satisfy the harness.

---

## 14. Update maintainer architecture documentation

- [ ] Update `docs/SPECS.md` or the current architecture index.
- [ ] Document authoritative privacy policy precedence.
- [ ] Document prepared-request-only network boundary.
- [ ] Document exact challenge digest inputs.
- [ ] Document pending consent lifecycle and replacement.
- [ ] Document runtime-only versus persistent state.
- [ ] Document lock boundary and duplicate-response prevention.
- [ ] Document persistent-write-before-authorization rule.
- [ ] Document separation from protected-action confirmation.
- [ ] Document stable public reason/error codes.
- [ ] Document approved diagnostic metadata and prohibited content.
- [ ] Document process-isolated real-Wry test requirements.

---

## 15. Update user-facing privacy documentation

- [ ] Explain information categories that may be sent.
- [ ] Explain that sanitization is not anonymity.
- [ ] Explain loopback/on-device versus network planners.
- [ ] Explain local-only mode.
- [ ] Explain ask-per-origin mode.
- [ ] Explain broad sanitized-network mode and its limitations.
- [ ] Explain allow this request.
- [ ] Explain allow for session.
- [ ] Explain persistent exact-destination allow.
- [ ] Explain origin-wide persistent block.
- [ ] Explain deny/cancel.
- [ ] Explain high-risk non-overridable behavior.
- [ ] Explain challenge expiry and state-change invalidation.
- [ ] Explain rule revocation.
- [ ] Explain clearing session grants.
- [ ] Explain clearing persistent allows while retaining blocks.
- [ ] Explain migration from legacy settings.
- [ ] Update planner setup documentation.
- [ ] Update configuration examples if any stale field remains documented.

---

## 16. Update the threat model

- [ ] Malicious page attempts to trigger consent automatically.
- [ ] Malicious page attempts to spoof the privacy dialog.
- [ ] Prompt injection attempts to alter privacy or action policy.
- [ ] Hostile OCR content attempts to authorize network access.
- [ ] Compromised planner attempts to return unsafe actions after consent.
- [ ] Redirect or endpoint substitution after consent display.
- [ ] Scheme/host/port/path change after challenge.
- [ ] Profile/model change after challenge.
- [ ] Replay of consumed challenge.
- [ ] Concurrent duplicate challenge responses.
- [ ] Persistence failure or partial write.
- [ ] Frontend store tampering.
- [ ] Stale frontend challenge after backend restart.
- [ ] Diagnostic/log leakage on failure paths.
- [ ] Hidden or inaccessible allow control on high-risk pages.
- [ ] Broad-mode misunderstanding and stale allows.
- [ ] Record mitigations, residual risk, and non-goals for each threat.

---

## 17. Reconcile BBCR and post-Batch-8 records

### 17.1 BBCR-003

- [ ] Reconcile every remote planner redaction/privacy checkbox.
- [ ] Mark explicit consent/per-origin controls according to exact evidence.
- [ ] Keep local relevance-selection residuals open if still applicable.
- [ ] Record frontend-state/scanner evidence.
- [ ] Record exact closure SHA/run/job.

### 17.2 Related BBCR items

- [ ] Reconcile BBCR-006 hostile page/OCR evidence without claiming the entire hostile-input program complete if residuals remain.
- [ ] Reconcile BBCR-008 only for privacy pause/lock/request-count behavior actually proved; keep cancellation/response-body residuals open.
- [ ] Reconcile BBCR-015 for consent invalidation and existing protected-action snapshot behavior.
- [ ] Reconcile BBCR-021 privacy/threat-model/documentation evidence.
- [ ] Do not close BBCR-013, BBCR-014, BBCR-016, BBCR-018, BBCR-019, BBCR-020, or other unrelated items through this milestone.

### 17.3 Post-Batch-8 reconciliation

- [ ] Update the statement that explicit transmission consent/per-origin controls remain open.
- [ ] Preserve all still-open broader remediation boundaries.
- [ ] Record that privacy milestone completion is not general release readiness.
- [ ] Preserve historical bounded evidence and publication corrections.

---

## 18. Update privacy implementation and closure reports

- [ ] Correct stale “still open” lists in the implementation report.
- [ ] Preserve Stage 1/2A historical evidence accurately.
- [ ] Record Stage 2B frontend/settings implementation evidence.
- [ ] Record legacy adapter removal evidence.
- [ ] Record supplemental backend evidence and harness repairs.
- [ ] Record restart/scanner/accessibility evidence added by this closure pass.
- [ ] Create a final closure report or addendum rather than rewriting history misleadingly.
- [ ] Include an explicit implemented/evidenced versus broader-open boundary.
- [ ] Include exact changed-file inventory.
- [ ] Include exact validation commands and results.
- [ ] Include exact final SHA/run/job.

---

## 19. Permanent validation sequence

### 19.1 Focused tests

- [ ] Origin normalization and rule validation tests.
- [ ] Migration tests.
- [ ] Pure policy evaluator tests.
- [ ] Challenge digest mutation tests.
- [ ] Challenge lifecycle and invalidation tests.
- [ ] Request-count matrix tests.
- [ ] Restart/reconstruction tests.
- [ ] Serialization/diagnostic privacy tests.
- [ ] Consent-response command tests.
- [ ] Frontend consent state-lifecycle tests.
- [ ] Consent-dialog interaction/accessibility tests.
- [ ] Privacy settings interaction tests.
- [ ] Scanner self-tests.

### 19.2 Repository gates

- [ ] `python3 scripts/check-silent-fallbacks.py`
- [ ] `python3 scripts/check-security-fallbacks.py`
- [ ] `python3 scripts/check-security-fallback-inventory.py --self-test`
- [ ] `python3 scripts/check-security-fallback-inventory.py`
- [ ] `python3 scripts/check-sensitive-diagnostics.py`
- [ ] Any new frontend privacy scanner self-test.
- [ ] Any new frontend privacy scanner audit.
- [ ] `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`
- [ ] `cargo check --manifest-path src-tauri/Cargo.toml`
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- [ ] Focused direct-command semantic evidence.
- [ ] Focused/process-isolated remote privacy evidence.
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml --all-features`
- [ ] `source ./fix-node-version.sh && pnpm lint`
- [ ] `source ./fix-node-version.sh && pnpm test:ui`
- [ ] `source ./fix-node-version.sh && pnpm build`
- [ ] Whitespace and unintended-diff validation.

### 19.3 CI wiring audit

- [ ] Confirm every closure-critical ignored Wry test is invoked process-isolated.
- [ ] Confirm focused tests run before or within the permanent full suite.
- [ ] Confirm no shell command masks an earlier failure.
- [ ] Confirm no test selection pattern silently filters the new tests.
- [ ] Confirm scanner self-test failure stops the workflow.
- [ ] Confirm permanent conclusion publication runs even on prior-step failure and reports the correct result.
- [ ] Confirm hosted status points to the current exact run.

---

## 20. Dangerous fallback and silent-failure audit

- [ ] Missing pending consent cannot become denial success.
- [ ] Wrong ID/digest cannot fall back to current pending challenge.
- [ ] Expired challenge cannot be regenerated and auto-authorized silently.
- [ ] State/destination mismatch cannot retry under broad mode automatically.
- [ ] Persistence failure cannot fall back to session or one-shot allow.
- [ ] Invalid rule cannot be silently discarded while a broader allow applies.
- [ ] Frontend status refresh failure cannot guess an effective decision.
- [ ] Frontend submit failure cannot close the dialog as success.
- [ ] Stale frontend challenge cannot remain actionable after backend mismatch.
- [ ] Network/provider failure cannot leave a reusable pending transaction.
- [ ] Logging failure cannot expose payload content.
- [ ] Scanner unreadable-file failure cannot pass silently.
- [ ] Process-isolated test launch failure cannot be reported as skipped success.
- [ ] Temporary workflow failure cannot be substituted for permanent CI.
- [ ] Document every accepted fallback with exact expression, rationale, and evidence.
- [ ] Prefer typed errors and authoritative refresh over fallback behavior.

---

## 21. Cleanup

- [ ] Remove temporary workflows.
- [ ] Remove exact triggers.
- [ ] Remove patch generators.
- [ ] Remove repair scripts.
- [ ] Remove connector probes and diagnostics.
- [ ] Remove test-only privacy bypasses.
- [ ] Remove broad scanner exclusions.
- [ ] Remove obsolete legacy privacy adapters and obsolete tests.
- [ ] Remove sensitive or realistic secret fixtures.
- [ ] Confirm `.github/workflows` contains only intended permanent workflows.
- [ ] Confirm `.github` root contains no milestone trigger/payload/helper residue.
- [ ] Confirm process-isolated tests remain permanently wired.
- [ ] Confirm final source/test/doc diff is intentional.
- [ ] Confirm no generated build output is committed.
- [ ] Confirm no stale document still lists completed closure work as open.
- [ ] Confirm historical failed runs remain described as failures.

---

## 22. Final checklist reconciliation

- [ ] Every applicable predecessor checkbox has a classification.
- [ ] Every “implemented and evidenced” item cites exact evidence.
- [ ] Every “implemented but missing evidence” item has been resolved or remains explicitly open.
- [ ] Every genuinely open required item is implemented and evidenced.
- [ ] Every non-selected/superseded item has rationale.
- [ ] No item is checked solely from inference.
- [ ] No source behavior is weakened to make a stale checklist item literally match an obsolete suggested shape.
- [ ] The predecessor final-evidence section is filled accurately.
- [ ] The closure TODO itself reflects final status accurately.
- [ ] The broader BBCR boundary remains open and explicit.

---

## 23. Final exact-SHA signoff

### 23.1 Pre-signoff

- [ ] Commit all intended source/test/scanner changes.
- [ ] Obtain permanent CI success on the exact source SHA.
- [ ] Update final documentation/evidence with that exact source SHA and run/job.
- [ ] Commit final documentation/evidence.
- [ ] Obtain permanent CI success on the exact final documentation SHA.
- [ ] Verify `master` still points to the exact final documentation SHA.
- [ ] Verify combined `ci/permanent` status is success.
- [ ] Verify no later commit has superseded the evidence.

### 23.2 Evidence record

Fill all fields before closure:

- Starting `master` SHA: `97fc24d80dec9275d2d5fc2d470fa220df102cce`
- Starting permanent CI run: `31044019503`
- Starting permanent CI job: `92435010766`
- Starting CI result: `success`
- Closure specification SHA: `e100b119dc7e8ceb827f606fe5ed7379e79737a1`
- Closure TODO SHA:
- Reconciliation SHA:
- Backend evidence SHA:
- Restart/reconstruction evidence SHA:
- Scanner evidence SHA:
- Frontend accessibility/state evidence SHA:
- Documentation/threat-model SHA:
- Cleanup SHA:
- Final exact SHA:
- Final branch: `master`
- Final permanent CI run:
- Final permanent CI job:
- Final permanent CI result:
- Focused consent evidence result:
- Restart/reconstruction result:
- Scanner self-test/audit result:
- Full Rust/Wry result:
- Frontend lint result:
- Frontend UI result:
- Frontend build result:
- Temporary machinery absent:
- Repository unchanged after signoff:

### 23.3 Final bounded statement

Use only after all applicable tasks are complete:

> The Blind Browser remote-data consent and origin-privacy milestone is complete. Deterministic Rust policy prevents unauthorized non-loopback planner transmission; current-origin status and just-in-time consent are implemented; allows are origin-, destination-, and policy-version-bound; blocks are origin-wide; high-risk contexts remain non-overridable; pending consent is runtime-only, expiring, replay-resistant, and state-bound; persistence failure and duplicate responses fail closed; backend/frontend privacy and accessibility evidence pass; temporary machinery is absent; and permanent CI succeeds on the exact final SHA. The broader BBCR remediation program remains open, and this milestone does not by itself establish general production release readiness.
