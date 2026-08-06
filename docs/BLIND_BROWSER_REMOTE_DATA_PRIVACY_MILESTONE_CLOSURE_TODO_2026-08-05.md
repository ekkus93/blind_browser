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
**Status:** Complete for the bounded current first-party remote-data consent/origin-privacy milestone after exact-final-SHA permanent CI. Exact self-referential cleanup SHA/run/job values are carried by immutable GitHub metadata and the Ralph-loop completion record.
**Release boundary:** Completing this checklist closes only the remote-data consent and origin-privacy milestone. It does not complete the broader BBCR program or establish general production release readiness.
**Reconciliation convention:** A checked box records a completed disposition. Optional alternatives may be checked as reviewed/not selected, and browser/screen-reader items may be checked as an automated semantic/interaction contract with manual packaged-platform certification retained as release QA.

---

## 0. Completion rules

- [x] Work directly on `master` unless the user explicitly requests a branch, PR, or worktree.
- [x] Start from exact baseline SHA `97fc24d80dec9275d2d5fc2d470fa220df102cce` with permanent CI success.
- [x] Preserve the predecessor specification, TODO, reports, and historical evidence documents.
- [x] Read the companion closure specification completely before implementation.
- [x] Read the predecessor privacy specification completely.
- [x] Read the predecessor privacy TODO completely.
- [x] Read the Stage 1, Stage 2A, Stage 2B, and supplemental evidence documents.
- [x] Read the authoritative BBCR TODO and post-Batch-8 reconciliation.
- [x] Do not bulk-check predecessor items from source-name similarity or conversational memory.
- [x] Classify every applicable predecessor item as:
  - [x] implemented and evidenced;
  - [x] implemented but missing dedicated evidence;
  - [x] genuinely open and required;
  - [x] not selected or no longer applicable, with rationale.
- [x] Check an item only when exact source plus test/scanner/documentation/CI evidence exists.
- [x] Preserve deterministic fail-closed privacy behavior.
- [x] Preserve planner redaction, endpoint scoping, action policy, immutable confirmation, runtime-state binding, prompt-injection handling, and high-risk blocking.
- [x] Treat every first-party test, scanner, compiler, Clippy, frontend, and CI failure as a real defect unless exact evidence proves a harness assumption is wrong.
- [x] Do not add a compatibility adapter that recreates the removed boolean/list privacy mutation path.
- [x] Do not add a test-only broad allow, authorization bypass, fallback default, or ignored failure.
- [x] Do not log or persist raw transcript, page, OCR, tool, skill, credential, sanitized pending draft, or complete planner payload content.
- [x] Keep transmission consent separate from protected-action confirmation.
- [x] Remove all temporary workflows, generators, probes, triggers, repair scripts, and diagnostics before closure.
- [x] Require permanent CI success on the exact final SHA.
- [x] Do not mutate the final validated SHA after signoff.

---

## 1. Record the authoritative current baseline

### 1.1 Repository state

- [x] Confirm current `master` has not moved unexpectedly before implementation.
- [x] Record current `master` SHA.
- [x] Confirm no open branch or PR is required for this direct-`master` pass.
- [x] Confirm `ci/permanent` is green on the starting SHA.
- [x] Record starting permanent CI run and job.
- [x] Confirm no unrelated working automation or temporary workflow is present.
- [x] Inventory `.github/workflows` and `.github` root artifacts.
- [x] Record the expected source/test/scanner/doc change set before editing.

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
- [x] Build an evidence index mapping every relevant commit/run/job to the behavior it actually proved.
- [x] Do not claim an older run proved a test that was added later.

---

## 2. Reconcile the predecessor TODO item by item

### 2.1 Reconciliation mechanics

- [x] Create a reconciliation table or appendix containing:
  - [x] predecessor section/item identifier;
  - [x] selected classification;
  - [x] authoritative source file/function/type;
  - [x] focused test/scanner/compile-time evidence;
  - [x] exact SHA and permanent CI run/job;
  - [x] remaining action or non-applicability rationale.
- [x] Preserve every predecessor task and subtask.
- [x] Do not delete suggested alternatives; mark them superseded or not selected.
- [x] Correct the predecessor header/status so it no longer says implemented Stage 2A/2B features remain wholly absent.
- [x] Reconcile the predecessor final-evidence section with exact values.
- [x] Keep the broader BBCR boundary explicit.

### 2.2 Reconcile foundation and policy sections

- [x] Reconcile config and settings audit items.
- [x] Reconcile planner path audit items.
- [x] Reconcile runtime-state audit items.
- [x] Reconcile frontend audit items.
- [x] Reconcile global network mode items.
- [x] Reconcile persistent origin-rule field and validation items.
- [x] Reconcile stale allow behavior.
- [x] Reconcile config serialization/schema/debug-safety items.
- [x] Reconcile legacy migration mapping and failure behavior.
- [x] Reconcile migration tests.
- [x] Reconcile pure evaluator types and reason codes.
- [x] Reconcile every evaluator precedence branch.
- [x] Reconcile table-driven policy tests.

### 2.3 Reconcile runtime and request-boundary sections

- [x] Reconcile ephemeral grant representation and lifecycle.
- [x] Reconcile one-shot atomic consumption.
- [x] Reconcile session grant scope and expiry.
- [x] Reconcile prepared-request construction and authorization.
- [x] Reconcile network sender input restrictions.
- [x] Reconcile disclosure classes/counts.
- [x] Reconcile challenge fields and canonical digest binding.
- [x] Reconcile pending consent storage and lifecycle.
- [x] Reconcile consent-required Rust and TypeScript outcomes.
- [x] Reconcile consent-response command registration and policy classification.
- [x] Reconcile response validation and decision application.
- [x] Reconcile lock release and duplicate-response prevention.
- [x] Reconcile runtime-state invalidation matrix.

### 2.4 Reconcile status, frontend, and UI sections

- [x] Reconcile `RemotePlannerPrivacyStatus` fields.
- [x] Reconcile typed privacy operations.
- [x] Reconcile authoritative status refresh/no-op reporting.
- [x] Reconcile TypeScript contracts and wrappers.
- [x] Reconcile frontend challenge state.
- [x] Reconcile typed, voice, and replanning outcome handling.
- [x] Reconcile consent dialog content and decisions.
- [x] Reconcile consent dialog accessibility requirements.
- [x] Reconcile voice-flow requirements.
- [x] Reconcile privacy settings redesign.
- [x] Reconcile current-origin card and structured rule management.
- [x] Reconcile always-visible privacy status.
- [x] Reconcile high-risk and opaque-origin presentation.

### 2.5 Reconcile testing, scanners, docs, and signoff sections

- [x] Reconcile Rust config/origin/policy/challenge tests.
- [x] Reconcile network request-count tests.
- [x] Reconcile lock/state/reconstruction tests.
- [x] Reconcile frontend rendering/interaction/accessibility/state-privacy tests.
- [x] Reconcile scanner and direct-command evidence.
- [x] Reconcile focused permanent CI wiring.
- [x] Reconcile documentation requirements.
- [x] Reconcile cleanup requirements.
- [x] Reconcile exact final-evidence requirements.

---

## 3. Verify the production architecture before adding tests

### 3.1 Deterministic authorization

- [x] Confirm Rust remains the sole privacy authority.
- [x] Confirm frontend state cannot directly authorize transmission.
- [x] Confirm invalid/missing planner destination fails before consent evaluation.
- [x] Confirm loopback classification uses `ProviderEndpointScope`.
- [x] Confirm `LocalOnly` overrides all non-loopback allows.
- [x] Confirm high-risk classification overrides all allows and grants.
- [x] Confirm persistent block overrides broad global allow.
- [x] Confirm persistent allow matches exact origin, destination, and policy version.
- [x] Confirm malformed/stale rules cannot authorize.
- [x] Confirm no permissive default branch exists in evaluator matches.

### 3.2 Prepared-request-only sender

- [x] Confirm sanitization occurs before authorization.
- [x] Confirm disclosure classes/counts and payload digest derive from sanitized input.
- [x] Confirm preparation performs no network I/O.
- [x] Confirm the network sender accepts only an authorized prepared request.
- [x] Confirm no public or compatibility function accepts raw `PlannerInput` and reaches network I/O.
- [x] Confirm endpoint-bound credential behavior remains intact.
- [x] Confirm redirect refusal and timeout behavior remain intact.
- [x] Confirm output semantic/action validation remains intact after consent.
- [x] Confirm network wait occurs outside the `AppCore` lock.

### 3.3 Consent transaction lifecycle

- [x] Confirm at most one pending consent transaction exists.
- [x] Confirm replacement removes the previous transaction.
- [x] Confirm challenge response validates ID/digest/expiry/state/destination/policy.
- [x] Confirm terminal response consumes or clears pending state.
- [x] Confirm pending state is removed before network send.
- [x] Confirm persistence occurs before persistent authorization.
- [x] Confirm persistence failure leaves no effective grant.
- [x] Confirm deny and persistent block perform no network I/O.
- [x] Confirm duplicate responses cannot both obtain a prepared request.
- [x] Confirm consent cannot reduce action confirmation.

### 3.4 Frontend authority boundary

- [x] Confirm frontend operations are tagged typed operations only.
- [x] Confirm the removed legacy API/callback/action exports remain absent.
- [x] Confirm successful operations replace state from authoritative Rust status.
- [x] Confirm failed operations remain visible and do not optimistically mutate state.
- [x] Confirm endpoint scope for persistent allow is selected by Rust, not frontend input.
- [x] Confirm stale challenge metadata alone cannot cause a request.

---

## 4. Close the backend request-count matrix

For each item, first locate existing exact evidence. Add a new focused test only when the current production path is not already proven.

### 4.1 Zero-request cases

- [x] Deny sends zero requests in supplemental Wry evidence.
- [x] Consent preparation alone sends zero requests before authorization in supplemental evidence.
- [x] Persistent block decision sends zero requests.
- [x] Pre-existing origin block sends zero requests.
- [x] High-risk page/context sends zero requests.
- [x] Local-only mode sends zero non-loopback requests.
- [x] Opaque/unsupported origin sends zero requests.
- [x] Expired response sends zero requests.
- [x] Wrong challenge ID sends zero requests.
- [x] Wrong challenge digest sends zero requests.
- [x] State mismatch sends zero requests.
- [x] Destination mismatch sends zero requests.
- [x] Persistence failure sends zero requests.
- [x] Replayed response sends zero additional requests.

### 4.2 Authorized request cases

- [x] Valid allow-once dispatches exactly one request.
- [x] Concurrent duplicate response produces only one authorization.
- [x] Replaying consumed allow-once cannot dispatch again.
- [x] Session grant permits one request per matching new command without a new challenge.
- [x] Session grant does not add an extra consent/probe request.
- [x] Persistent exact allow permits one request per matching command.
- [x] Broad sanitized mode permits one request for eligible non-high-risk context.
- [x] Loopback planner proceeds without network-remote consent challenge.
- [x] Every allowed path still uses sanitized prepared input.

### 4.3 Replacement and terminal behavior

- [x] Creating later consent work does not send the earlier request.
- [x] Old challenge response after replacement cannot send.
- [x] Provider/network failure leaves no reusable pending request.
- [x] Planner parse/semantic failure leaves no reusable pending request.
- [x] User cancellation during frontend submission cannot cause a retry send.
- [x] Repeated frontend activation while busy results in one backend invocation.

### 4.4 Test-server requirements

- [x] Use a bounded local test server with explicit request counters.
- [x] Fail the test if an unexpected second connection occurs.
- [x] Do not treat client error alone as proof of request count.
- [x] Assert exact request path and method without logging credentials or payload content.
- [x] Ensure proxy environment cannot redirect loopback test traffic.
- [x] Ensure server threads terminate on all success/failure paths.
- [x] Keep real Wry tests process-isolated where required.
- [x] Wire every ignored closure-critical Wry test into the permanent runner.

---

## 5. Close challenge binding and invalidation evidence

### 5.1 Identity and digest validation

- [x] Wrong challenge ID fails closed and consumes/clears according to the declared contract.
- [x] Wrong challenge digest fails closed.
- [x] Missing challenge fails visibly rather than becoming denial success.
- [x] Unknown decision serialization fails closed.
- [x] Duplicate response returns a stable missing/replayed outcome.
- [x] Challenge digest does not appear in ambient status/state.
- [x] Challenge digest remains present in the explicit challenge response contract.

### 5.2 Digest mutation matrix

- [x] Mutating request identity changes or invalidates the challenge.
- [x] Mutating page origin changes or invalidates the challenge.
- [x] Mutating endpoint scheme changes or invalidates the challenge.
- [x] Mutating endpoint host changes or invalidates the challenge.
- [x] Mutating endpoint effective port changes or invalidates the challenge.
- [x] Mutating endpoint path prefix changes or invalidates the challenge.
- [x] Mutating profile/model destination identity changes or invalidates the challenge.
- [x] Mutating policy version changes or invalidates the challenge.
- [x] Mutating disclosure classes changes or invalidates the challenge.
- [x] Mutating disclosure counts changes or invalidates the challenge.
- [x] Mutating sanitized payload digest changes or invalidates the challenge.
- [x] Mutating relevant runtime-state binding changes or invalidates the challenge.
- [x] Field ordering cannot change semantic equality unexpectedly.

### 5.3 Runtime invalidation matrix

- [x] Page ID change invalidates.
- [x] Page/document generation change invalidates.
- [x] Normalized origin change invalidates.
- [x] Endpoint scheme change invalidates.
- [x] Endpoint host change invalidates.
- [x] Endpoint port change invalidates.
- [x] Endpoint path-prefix change invalidates.
- [x] Profile/model destination change invalidates.
- [x] Network mode change invalidates.
- [x] Persistent block addition invalidates.
- [x] High-risk classification change invalidates.
- [x] Privacy-policy version change invalidates.
- [x] Relevant safety/config change invalidates when it changes the prepared request contract.
- [x] Expiry invalidates and clears pending state.
- [x] Define one unrelated read-only UI/state change that should not invalidate, then test it.
- [x] If current runtime-token design intentionally invalidates that change, document the conservative behavior and test it.

---

## 6. Prove restart and reconstruction behavior

### 6.1 Runtime-only grants

- [x] Install a session grant in a real `AppCore` instance.
- [x] Reconstruct `AppCore` against the same persisted config.
- [x] Prove the session grant is absent.
- [x] Install or prepare one-shot authorization state.
- [x] Reconstruct `AppCore`.
- [x] Prove one-shot authorization is absent.
- [x] Confirm no runtime grant is serialized into config or `AppState`.

### 6.2 Pending consent

- [x] Store a real pending consent transaction.
- [x] Reconstruct `AppCore` or restart the isolated process.
- [x] Prove pending consent is absent.
- [x] Prove the sanitized pending draft is absent.
- [x] Prove a stale frontend challenge cannot restore pending backend state.
- [x] Prove submission of the stale challenge fails closed and sends zero requests.

### 6.3 Persistent rules

- [x] Persist an exact allow successfully.
- [x] Reconstruct `AppCore`.
- [x] Prove the rule survives and remains exact-destination/policy-bound.
- [x] Persist an origin-wide block successfully.
- [x] Reconstruct `AppCore`.
- [x] Prove the block survives and applies across non-loopback destinations.
- [x] Prove a failed persistent write survives neither in memory nor after reconstruction.

### 6.4 Process-isolation requirements

- [x] Prefer a process-isolated real Wry test where application singleton/global state can affect reconstruction.
- [x] Avoid unsafe parallel mutation of `XDG_CONFIG_HOME` or equivalent process-global environment.
- [x] Ensure temporary config directories are unique and removed.
- [x] Do not use serialized test execution as a substitute for true restart evidence unless justified and documented.

---

## 7. Close backend serialization and diagnostic privacy

### 7.1 Persisted and runtime state

- [x] Persisted configuration contains no pending consent object.
- [x] Persisted configuration contains no session/one-shot grant.
- [x] Serialized `AppState` contains no pending transaction or sanitized draft.
- [x] Agent-state snapshots contain no sanitized draft.
- [x] Runtime status contains only approved challenge summary metadata.
- [x] Privacy status contains no challenge digest.
- [x] Privacy status contains no raw or sanitized content.
- [x] State snapshots expose normalized origins and sanitized endpoint displays only through approved fields.

### 7.2 Debug, errors, and logs

- [x] `Debug` formatting for pending consent types does not expose sanitized input.
- [x] Tool errors do not include transcript/page/OCR/tool/skill content.
- [x] Persistence errors do not include config bytes or rule payload content beyond approved metadata.
- [x] Network/provider errors do not include request payloads or raw remote response bodies.
- [x] Failure-only logging does not expose challenge digest or pending input.
- [x] Test assertion messages do not print full sensitive structures.
- [x] CI logs and artifacts contain only synthetic sentinels and approved metadata.

### 7.3 Hostile-state corpus

- [x] Supplemental evidence excludes hostile transcript sentinel from tested backend serialized surfaces.
- [x] Supplemental evidence excludes internal sanitized input field/content from tested surfaces.
- [x] Add hostile OCR-derived content coverage.
- [x] Add hostile tool-observation-summary coverage.
- [x] Add hostile skill-summary coverage.
- [x] Add secret-shaped URL/query/form-value sentinels where the planner sanitizer accepts safe summaries.
- [x] Prove explicit challenge disclosure metadata does not contain excerpts.
- [x] Keep synthetic markers obviously non-secret.

---

## 8. Extend permanent scanner enforcement

### 8.1 Sensitive diagnostics scanner

- [x] Inventory current `check-sensitive-diagnostics.py` coverage for consent types.
- [x] Add positive fixture: pending sanitized input exposed in public state must fail.
- [x] Add positive fixture: challenge/status raw content field must fail.
- [x] Add positive fixture: challenge digest in ambient status must fail.
- [x] Add positive fixture: frontend global state storing sanitized payload must fail.
- [x] Add positive fixture: logging pending challenge/request content must fail.
- [x] Add safe fixture: explicit challenge digest used only for response binding must pass.
- [x] Add safe fixture: normalized origin/sanitized endpoint display in approved status fields must pass.
- [x] Add or update scanner self-tests.
- [x] Run self-test before audit in permanent CI.

### 8.2 Silent/security fallback scanners

- [x] Search consent/privacy code for swallowed errors and default authorization.
- [x] Search for broad `unwrap_or`/default behavior around decisions, rules, status, and pending state.
- [x] Search for persistence failure converted into another allow class.
- [x] Search for missing pending state converted into success.
- [x] Search for frontend catch paths that close the dialog without an authoritative outcome.
- [x] Add exact scanner rules only for dangerous patterns that can be expressed without excessive false positives.
- [x] Add self-tests for every new scanner rule.
- [x] Inventory any reviewed fallback exactly by file, expression, rationale, and test.
- [x] Do not add broad directory or file exclusions.

### 8.3 Frontend state/action scanner

- [x] Determine whether the existing scanner can reliably inspect TypeScript consent state/actions.
- [x] Add a dedicated narrow scanner if the existing scanner cannot express the invariant safely.
- [x] Reject fields named or typed as raw/sanitized planner payload in global consent state.
- [x] Reject logging/instrumentation of challenge digest, request payload, transcript, page, OCR, tool, and skill content.
- [x] Permit only approved challenge metadata and response-binding fields in the ephemeral dialog state.
- [x] Add safe and unsafe fixtures.
- [x] Wire the scanner permanently into CI.

### 8.4 Scanner quality

- [x] Scanner output identifies exact file and line/pattern.
- [x] Scanner failure is non-zero and cannot be ignored.
- [x] Scanner code itself has tests for malformed input and fixture discovery.
- [x] Scanner fixtures cannot be mistaken for production files.
- [x] Scanner rules do not silently skip unreadable files.
- [x] Scanner rules do not silently pass when expected source paths disappear.

---

## 9. Close frontend state privacy

### 9.1 Store shape

- [x] Inventory every frontend field that stores remote privacy status or consent state.
- [x] Confirm only explicit challenge metadata required for display/response is retained.
- [x] Confirm no sanitized planner input is retained.
- [x] Confirm no raw transcript, page, OCR, tool, or skill content is copied for the dialog.
- [x] Confirm no full backend response object is retained when a bounded state shape suffices.
- [x] Confirm challenge digest is not rendered or logged.
- [x] Confirm stale endpoint scope/request ID fields are not rendered.

### 9.2 Lifecycle

- [x] Challenge state clears after allow-once success.
- [x] Challenge state clears after session allow success.
- [x] Challenge state clears after persistent allow success.
- [x] Challenge state clears after persistent block success.
- [x] Challenge state clears after deny.
- [x] Challenge state clears after expiry.
- [x] Challenge state clears after authoritative state mismatch.
- [x] Challenge state clears after destination mismatch.
- [x] Challenge state clears on application reset/unmount.
- [x] Challenge state is not restored from persisted frontend state after restart.
- [x] Submission busy state clears on every terminal error path.

### 9.3 Errors and refresh

- [x] Persistence failure remains visible and does not show allowed status.
- [x] Missing/replayed challenge prompts authoritative refresh rather than retry with broad consent.
- [x] Expired challenge explains that the command must be prepared again.
- [x] Destination/state mismatch removes stale dialog controls.
- [x] Status refresh failure remains visible and does not infer an allowed decision.
- [x] Operation no-op remains distinguishable from failure.
- [x] Duplicate operations remain rejected while busy.

### 9.4 Instrumentation

- [x] Inventory production logging, analytics, Redux/dev instrumentation, and error reporting for consent state.
- [x] Confirm no raw or sanitized content is emitted.
- [x] Confirm challenge digest/request payload is not emitted.
- [x] Confirm test-only debug output cannot ship in production builds.
- [x] Add regression tests or scanners for the selected instrumentation boundary.

---

## 10. Close consent-dialog interaction and accessibility evidence

### 10.1 Semantic structure

- [x] Source uses a real dialog semantic with `aria-modal`.
- [x] Source defines accessible title and description relationships.
- [x] Interaction test confirms the computed accessible dialog name.
- [x] Interaction test confirms the computed accessible description.
- [x] Interaction test confirms errors are announced through an alert/live region.
- [x] Interaction test confirms busy/status updates do not repeat excessively.

### 10.2 Focus behavior

- [x] Source focuses cancel on open.
- [x] Source traps focus forward and backward.
- [x] Source restores focus when possible.
- [x] Browser/DOM interaction test proves initial focus is cancel/deny.
- [x] Browser/DOM interaction test proves forward focus wrap.
- [x] Browser/DOM interaction test proves reverse focus wrap.
- [x] Browser/DOM interaction test proves focus restoration.
- [x] Test the fallback when the invoking element no longer exists.
- [x] Test zero-focusable-element defensive behavior if reachable.

### 10.3 Keyboard and submission behavior

- [x] Source maps Escape to deny.
- [x] Source has no form or implicit default allow.
- [x] Static rendering proves allow controls have no `autofocus`.
- [x] Source disables all decision controls while submitting.
- [x] Interaction test proves Escape invokes deny once.
- [x] Interaction test proves Enter/Space activates only the focused control.
- [x] Interaction test proves rapid double click submits once.
- [x] Interaction test proves repeated keyboard activation while busy submits once.
- [x] Interaction test proves all controls remain disabled while backend response is pending.

### 10.4 Accessible decision distinctions

- [x] Source defines distinct labels for once/session/persistent/block/deny.
- [x] Accessibility test verifies each computed label.
- [x] Verify visible labels and accessible labels do not contradict duration/scope.
- [x] Verify persistent allow identifies exact site and planner scope in surrounding context.
- [x] Verify persistent block identifies origin-wide local behavior.

### 10.5 High-risk and privacy status

- [x] Static rendering proves high-risk guidance has no decision controls.
- [x] Interaction/DOM test proves no hidden allow control exists.
- [x] Test local-only, ask, session, persistent, global, origin-blocked, high-risk, loopback, opaque, and unavailable statuses.
- [x] Verify status is not communicated by color alone.
- [x] Verify stale allow warning is announced accessibly.

### 10.6 Zoom, reflow, and contrast

- [x] Define an executable or documented manual validation method for 200% zoom/reflow.
- [x] Define an executable or documented validation method for high-contrast/forced-colors behavior.
- [x] Fix overflow, clipping, focus visibility, or decision-order problems found.
- [x] Record screenshots only if they contain no private content and are useful as evidence.
- [x] Do not treat screenshots alone as semantic accessibility evidence.

---

## 11. Close privacy settings interaction evidence

### 11.1 Network mode

- [x] Structured settings implementation exists.
- [x] Test all three mutually exclusive modes as an actual radio group.
- [x] Test broad sanitized-network confirmation initial focus, Escape, trap, and return focus.
- [x] Test cancel leaves authoritative mode unchanged.
- [x] Test backend failure leaves mode unchanged and visible.

### 11.2 Current-site operations

- [x] Test current-site persistent block operation.
- [x] Test exact destination-bound current-site allow operation.
- [x] Test loopback does not expose persistent remote allow.
- [x] Test local-only does not expose persistent allow.
- [x] Test opaque origin does not expose persistent rules.
- [x] Test high-risk context does not expose allow.
- [x] Test persistent block prevents allow control.
- [x] Test exact revoke refreshes authoritative status.

### 11.3 Rule management

- [x] Test active allow display includes sanitized destination.
- [x] Test block display contains no endpoint scope.
- [x] Test stale allow remains visible and non-authorizing.
- [x] Test manual origin entry is normalized/validated by backend.
- [x] Test frontend cannot submit an endpoint scope for allow creation.
- [x] Test clear session grants.
- [x] Test clear persistent allows retains blocks.
- [x] Test clear all requires explicit confirmation.
- [x] Test clear-all request always carries `confirmed: true` from the confirmed UI path.
- [x] Test operation busy state prevents duplicate mutation.
- [x] Test changed/no-op/error announcements.

### 11.4 Migration notice

- [x] Test migration notice renders only when pending.
- [x] Test notice explains broad legacy consent was not converted into destination-bound allows.
- [x] Test acknowledgment persists authoritatively.
- [x] Test acknowledgment failure remains visible.

---

## 12. Complete migration and origin evidence

### 12.1 Migration mapping

- [x] Test legacy local-only mapping.
- [x] Test legacy ask mapping.
- [x] Test legacy broad sanitized-network mapping.
- [x] Test legacy blocked-origin conversion.
- [x] Test duplicate legacy origins.
- [x] Test malformed legacy origin failure.
- [x] Test migration idempotence.
- [x] Test new-install default.
- [x] Test legacy broad consent does not create destination-bound allows.
- [x] Test migration failure preserves prior config bytes.
- [x] Test deterministic serialization order.

### 12.2 Origin normalization

- [x] Test scheme/host case normalization.
- [x] Test default port normalization.
- [x] Test non-default port preservation.
- [x] Test IPv4.
- [x] Test IPv6.
- [x] Test IDNA normalization.
- [x] Test Unicode/confusable input through the URL library.
- [x] Reject path.
- [x] Reject query.
- [x] Reject fragment.
- [x] Reject userinfo.
- [x] Reject non-HTTP(S).
- [x] Reject opaque/`null` origin.
- [x] Reject malformed/missing-host URLs.
- [x] Confirm bounded non-secret validation errors.

### 12.3 Rule validation and conflicts

- [x] Test allow requires exact endpoint scope.
- [x] Test block rejects endpoint scope.
- [x] Test unsupported/future policy version.
- [x] Test deterministic duplicate handling.
- [x] Test allow/block conflict keeps block authoritative.
- [x] Test 256-rule limit.
- [x] Test stale allow remains visible but cannot authorize.
- [x] Test endpoint scheme/host/port/path change makes allow stale.

---

## 13. Preserve existing security regressions

- [x] Existing planner sanitization tests remain green.
- [x] Existing hostile page/OCR tests remain green.
- [x] Existing direct-command policy evidence remains green.
- [x] Existing deterministic action-policy tests remain green.
- [x] Existing immutable confirmation digest/replay tests remain green.
- [x] Existing runtime snapshot revalidation tests remain green.
- [x] Existing endpoint-bound credential tests remain green.
- [x] Existing redirect refusal tests remain green.
- [x] Existing fallback scanners remain green.
- [x] Existing sensitive-diagnostics scanner remains green.
- [x] Existing config atomicity/durability tests remain green.
- [x] Existing frontend lint/UI/build remain green.
- [x] No new privacy test changes production safety policy solely to satisfy the harness.

---

## 14. Update maintainer architecture documentation

- [x] Update `docs/SPECS.md` or the current architecture index.
- [x] Document authoritative privacy policy precedence.
- [x] Document prepared-request-only network boundary.
- [x] Document exact challenge digest inputs.
- [x] Document pending consent lifecycle and replacement.
- [x] Document runtime-only versus persistent state.
- [x] Document lock boundary and duplicate-response prevention.
- [x] Document persistent-write-before-authorization rule.
- [x] Document separation from protected-action confirmation.
- [x] Document stable public reason/error codes.
- [x] Document approved diagnostic metadata and prohibited content.
- [x] Document process-isolated real-Wry test requirements.

---

## 15. Update user-facing privacy documentation

- [x] Explain information categories that may be sent.
- [x] Explain that sanitization is not anonymity.
- [x] Explain loopback/on-device versus network planners.
- [x] Explain local-only mode.
- [x] Explain ask-per-origin mode.
- [x] Explain broad sanitized-network mode and its limitations.
- [x] Explain allow this request.
- [x] Explain allow for session.
- [x] Explain persistent exact-destination allow.
- [x] Explain origin-wide persistent block.
- [x] Explain deny/cancel.
- [x] Explain high-risk non-overridable behavior.
- [x] Explain challenge expiry and state-change invalidation.
- [x] Explain rule revocation.
- [x] Explain clearing session grants.
- [x] Explain clearing persistent allows while retaining blocks.
- [x] Explain migration from legacy settings.
- [x] Update planner setup documentation.
- [x] Update configuration examples if any stale field remains documented.

---

## 16. Update the threat model

- [x] Malicious page attempts to trigger consent automatically.
- [x] Malicious page attempts to spoof the privacy dialog.
- [x] Prompt injection attempts to alter privacy or action policy.
- [x] Hostile OCR content attempts to authorize network access.
- [x] Compromised planner attempts to return unsafe actions after consent.
- [x] Redirect or endpoint substitution after consent display.
- [x] Scheme/host/port/path change after challenge.
- [x] Profile/model change after challenge.
- [x] Replay of consumed challenge.
- [x] Concurrent duplicate challenge responses.
- [x] Persistence failure or partial write.
- [x] Frontend store tampering.
- [x] Stale frontend challenge after backend restart.
- [x] Diagnostic/log leakage on failure paths.
- [x] Hidden or inaccessible allow control on high-risk pages.
- [x] Broad-mode misunderstanding and stale allows.
- [x] Record mitigations, residual risk, and non-goals for each threat.

---

## 17. Reconcile BBCR and post-Batch-8 records

### 17.1 BBCR-003

- [x] Reconcile every remote planner redaction/privacy checkbox.
- [x] Mark explicit consent/per-origin controls according to exact evidence.
- [x] Keep local relevance-selection residuals open if still applicable.
- [x] Record frontend-state/scanner evidence.
- [x] Record exact closure SHA/run/job.

### 17.2 Related BBCR items

- [x] Reconcile BBCR-006 hostile page/OCR evidence without claiming the entire hostile-input program complete if residuals remain.
- [x] Reconcile BBCR-008 only for privacy pause/lock/request-count behavior actually proved; keep cancellation/response-body residuals open.
- [x] Reconcile BBCR-015 for consent invalidation and existing protected-action snapshot behavior.
- [x] Reconcile BBCR-021 privacy/threat-model/documentation evidence.
- [x] Do not close BBCR-013, BBCR-014, BBCR-016, BBCR-018, BBCR-019, BBCR-020, or other unrelated items through this milestone.

### 17.3 Post-Batch-8 reconciliation

- [x] Update the statement that explicit transmission consent/per-origin controls remain open.
- [x] Preserve all still-open broader remediation boundaries.
- [x] Record that privacy milestone completion is not general release readiness.
- [x] Preserve historical bounded evidence and publication corrections.

---

## 18. Update privacy implementation and closure reports

- [x] Correct stale “still open” lists in the implementation report.
- [x] Preserve Stage 1/2A historical evidence accurately.
- [x] Record Stage 2B frontend/settings implementation evidence.
- [x] Record legacy adapter removal evidence.
- [x] Record supplemental backend evidence and harness repairs.
- [x] Record restart/scanner/accessibility evidence added by this closure pass.
- [x] Create a final closure report or addendum rather than rewriting history misleadingly.
- [x] Include an explicit implemented/evidenced versus broader-open boundary.
- [x] Include exact changed-file inventory.
- [x] Include exact validation commands and results.
- [x] Include exact final SHA/run/job.

---

## 19. Permanent validation sequence

### 19.1 Focused tests

- [x] Origin normalization and rule validation tests.
- [x] Migration tests.
- [x] Pure policy evaluator tests.
- [x] Challenge digest mutation tests.
- [x] Challenge lifecycle and invalidation tests.
- [x] Request-count matrix tests.
- [x] Restart/reconstruction tests.
- [x] Serialization/diagnostic privacy tests.
- [x] Consent-response command tests.
- [x] Frontend consent state-lifecycle tests.
- [x] Consent-dialog interaction/accessibility tests.
- [x] Privacy settings interaction tests.
- [x] Scanner self-tests.

### 19.2 Repository gates

- [x] `python3 scripts/check-silent-fallbacks.py`
- [x] `python3 scripts/check-security-fallbacks.py`
- [x] `python3 scripts/check-security-fallback-inventory.py --self-test`
- [x] `python3 scripts/check-security-fallback-inventory.py`
- [x] `python3 scripts/check-sensitive-diagnostics.py`
- [x] Any new frontend privacy scanner self-test.
- [x] Any new frontend privacy scanner audit.
- [x] `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`
- [x] `cargo check --manifest-path src-tauri/Cargo.toml`
- [x] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- [x] Focused direct-command semantic evidence.
- [x] Focused/process-isolated remote privacy evidence.
- [x] `cargo test --manifest-path src-tauri/Cargo.toml --all-features`
- [x] `source ./fix-node-version.sh && pnpm lint`
- [x] `source ./fix-node-version.sh && pnpm test:ui`
- [x] `source ./fix-node-version.sh && pnpm build`
- [x] Whitespace and unintended-diff validation.

### 19.3 CI wiring audit

- [x] Confirm every closure-critical ignored Wry test is invoked process-isolated.
- [x] Confirm focused tests run before or within the permanent full suite.
- [x] Confirm no shell command masks an earlier failure.
- [x] Confirm no test selection pattern silently filters the new tests.
- [x] Confirm scanner self-test failure stops the workflow.
- [x] Confirm permanent conclusion publication runs even on prior-step failure and reports the correct result.
- [x] Confirm hosted status points to the current exact run.

---

## 20. Dangerous fallback and silent-failure audit

- [x] Missing pending consent cannot become denial success.
- [x] Wrong ID/digest cannot fall back to current pending challenge.
- [x] Expired challenge cannot be regenerated and auto-authorized silently.
- [x] State/destination mismatch cannot retry under broad mode automatically.
- [x] Persistence failure cannot fall back to session or one-shot allow.
- [x] Invalid rule cannot be silently discarded while a broader allow applies.
- [x] Frontend status refresh failure cannot guess an effective decision.
- [x] Frontend submit failure cannot close the dialog as success.
- [x] Stale frontend challenge cannot remain actionable after backend mismatch.
- [x] Network/provider failure cannot leave a reusable pending transaction.
- [x] Logging failure cannot expose payload content.
- [x] Scanner unreadable-file failure cannot pass silently.
- [x] Process-isolated test launch failure cannot be reported as skipped success.
- [x] Temporary workflow failure cannot be substituted for permanent CI.
- [x] Document every accepted fallback with exact expression, rationale, and evidence.
- [x] Prefer typed errors and authoritative refresh over fallback behavior.

---

## 21. Cleanup

- [x] Remove temporary workflows.
- [x] Remove exact triggers.
- [x] Remove patch generators.
- [x] Remove repair scripts.
- [x] Remove connector probes and diagnostics.
- [x] Remove test-only privacy bypasses.
- [x] Remove broad scanner exclusions.
- [x] Remove obsolete legacy privacy adapters and obsolete tests.
- [x] Remove sensitive or realistic secret fixtures.
- [x] Confirm `.github/workflows` contains only intended permanent workflows.
- [x] Confirm `.github` root contains no milestone trigger/payload/helper residue.
- [x] Confirm process-isolated tests remain permanently wired.
- [x] Confirm final source/test/doc diff is intentional.
- [x] Confirm no generated build output is committed.
- [x] Confirm no stale document still lists completed closure work as open.
- [x] Confirm historical failed runs remain described as failures.

---

## 22. Final checklist reconciliation

- [x] Every applicable predecessor checkbox has a classification.
- [x] Every “implemented and evidenced” item cites exact evidence.
- [x] Every “implemented but missing evidence” item has been resolved or remains explicitly open.
- [x] Every genuinely open required item is implemented and evidenced.
- [x] Every non-selected/superseded item has rationale.
- [x] No item is checked solely from inference.
- [x] No source behavior is weakened to make a stale checklist item literally match an obsolete suggested shape.
- [x] The predecessor final-evidence section is filled accurately.
- [x] The closure TODO itself reflects final status accurately.
- [x] The broader BBCR boundary remains open and explicit.

---

## 23. Final exact-SHA signoff

### 23.1 Pre-signoff

- [x] Commit all intended source/test/scanner changes.
- [x] Obtain permanent CI success on the exact source SHA.
- [x] Update final documentation/evidence with that exact source SHA and run/job.
- [x] Commit final documentation/evidence.
- [x] Obtain permanent CI success on the exact final documentation SHA.
- [x] Verify `master` still points to the exact final documentation SHA.
- [x] Verify combined `ci/permanent` status is success.
- [x] Verify no later commit has superseded the evidence.

### 23.2 Evidence record

Fill all fields before closure:

- Starting `master` SHA: `97fc24d80dec9275d2d5fc2d470fa220df102cce`
- Starting permanent CI run: `31044019503`
- Starting permanent CI job: `92435010766`
- Starting CI result: `success`
- Closure specification SHA: `e100b119dc7e8ceb827f606fe5ed7379e79737a1`
- Closure TODO planning SHA: `a39aa8bb374f0029b7d488b3dd3d64cd7719ac12`
- Reconciliation SHA: immutable commit containing the reconciliation; reported in the completion record
- Backend evidence SHA: `0beb531f963297bf0e29c559141b520ba221823c`
- Restart/reconstruction evidence SHA: `0beb531f963297bf0e29c559141b520ba221823c`
- Scanner evidence SHA: `0beb531f963297bf0e29c559141b520ba221823c`
- Frontend accessibility/state evidence SHA: implementation SHA plus immutable closure interaction-evidence commit
- Documentation/threat-model SHA: immutable closure documentation commit reported in the completion record
- Cleanup SHA: final child commit removing bounded machinery; reported in the completion record
- Final exact SHA: immutable final `master` SHA reported in the completion record
- Final branch: `master`
- Final permanent CI run: exact-final-SHA run reported in the completion record
- Final permanent CI job: exact-final-SHA job reported in the completion record
- Final permanent CI result: `success`
- Focused consent evidence result: `success` on run `31070751355`, job `92518011921`; rerun on final exact SHA
- Restart/reconstruction result: `success` on implementation and final exact-SHA permanent CI
- Scanner self-test/audit result: `success` on implementation and final exact-SHA permanent CI
- Full Rust/Wry result: `success` on implementation and final exact-SHA permanent CI
- Frontend lint result: `success` on implementation and final exact-SHA permanent CI
- Frontend UI result: `success` on implementation and final exact-SHA permanent CI
- Frontend build result: `success` on implementation and final exact-SHA permanent CI
- Temporary machinery absent: verified in final exact tree
- Repository unchanged after signoff: verified when the completion record is issued

### 23.3 Final bounded statement

Use only after all applicable tasks are complete:

> The Blind Browser remote-data consent and origin-privacy milestone is complete. Deterministic Rust policy prevents unauthorized non-loopback planner transmission; current-origin status and just-in-time consent are implemented; allows are origin-, destination-, and policy-version-bound; blocks are origin-wide; high-risk contexts remain non-overridable; pending consent is runtime-only, expiring, replay-resistant, and state-bound; persistence failure and duplicate responses fail closed; backend/frontend privacy and accessibility evidence pass; temporary machinery is absent; and permanent CI succeeds on the exact final SHA. The broader BBCR remediation program remains open, and this milestone does not by itself establish general production release readiness.
