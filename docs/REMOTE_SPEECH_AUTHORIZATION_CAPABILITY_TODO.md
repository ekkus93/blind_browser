# Remote Speech Authorization Capability TODO

**Project:** Blind Browser
**Execution mode:** Ralph Loop
**Specification:** `docs/REMOTE_SPEECH_AUTHORIZATION_CAPABILITY_SPEC.md`
**Related review:** `docs/BB_CODE_REVIEW3_TODO.md`

## 0. Ralph Loop rules

Work this checklist until all actionable tasks are complete or a genuine external/hardware/human blocker is reached.

For every batch:

- [ ] Inspect current code before editing.
- [ ] Record the exact baseline SHA.
- [ ] Implement the smallest complete change satisfying the spec.
- [ ] Add/update tests in the same batch.
- [ ] Run applicable validation.
- [ ] Fix failures rather than bypassing them.
- [ ] Review adjacent code for unsafe fallback and silent failure.
- [ ] Record exact evidence.
- [ ] Mark items complete only when acceptance criteria are proven.

Non-negotiable:

- [ ] Do not weaken compiler, Clippy, privacy, scanner, lint, or test gates.
- [ ] Do not add broad `#[allow(...)]` solely to pass CI.
- [ ] Do not silently fall back from local speech to remote speech.
- [ ] Do not silently change remote providers.
- [ ] Do not treat invalid/missing privacy state as permission.
- [ ] Do not retain compatibility APIs that bypass typed authorization.
- [ ] Do not claim completion on a SHA different from the validated SHA.
- [ ] Keep unvalidated implementation changes off `master`.

# Phase 1 — Reconcile current state

## 1.1 Establish authoritative baseline

- [ ] Fetch/inspect current `master`.
- [ ] Record exact baseline SHA.
- [ ] Read current `docs/BB_CODE_REVIEW3_TODO.md` P1.1/P1.2 state.
- [ ] Inspect remote TTS/narration privacy flow.
- [ ] Inspect remote ASR/transcription privacy flow.
- [ ] Inspect existing microphone privacy config/persistence.
- [ ] Inspect consent dialog/controller path.
- [ ] Inspect all speech provider abstractions/callers.
- [ ] Identify partial work from prior P1.1 attempts.
- [ ] Reconcile/remove abandoned partial code only where necessary.

Acceptance:

- [ ] Work starts from a known exact SHA.
- [ ] No stale archive/old validation branch is treated as authoritative.

# Phase 2 — Authorization type-state foundation

## 2.1 Define capabilities

- [ ] Add crate-private `RemoteTtsAuthorization` or equivalent.
- [ ] Add crate-private `RemoteAsrAuthorization` or equivalent.
- [ ] Keep constructors inside trusted privacy/controller code.
- [ ] Prevent TTS capability from satisfying ASR dispatch.
- [ ] Prevent ASR capability from satisfying TTS dispatch.
- [ ] Do not serialize/persist capabilities.

## 2.2 Consumption semantics

- [ ] Pass one-operation authorization by value into remote dispatch where practical.
- [ ] Ensure authorization cannot casually be reused.
- [ ] Keep capability lifetime scoped to the approved operation.

Tests:

- [ ] Purpose-separation test.
- [ ] Structural provider-signature test.
- [ ] Constructor visibility regression test where practical.

# Phase 3 — Privacy decision controller

- [ ] Identify/create one authoritative remote-speech privacy decision path.
- [ ] Represent authorized, consent-required, and denied/error explicitly.
- [ ] Make invalid/missing policy fail closed.
- [ ] Do not use permissive defaults.
- [ ] Bind pending consent to operation identity/generation.
- [ ] Bind purpose/provider sufficiently to reject cross-use.
- [ ] Define duplicate/stale consent behavior.

Tests:

- [ ] Invalid policy fails closed.
- [ ] Unknown provider fails toward remote unless proven local.
- [ ] Stale consent cannot authorize.
- [ ] Duplicate approval cannot dispatch twice.

# Phase 4 — Fix remote narration/TTS

## 4.1 Audit paths

- [ ] Locate every remote TTS provider entry point.
- [ ] Locate every caller/wrapper.
- [ ] Identify compatibility/direct-call bypasses.

## 4.2 Enforce provider-boundary authorization

- [ ] Require TTS authorization at remote dispatch.
- [ ] Update trusted authorized callers.
- [ ] Remove/migrate unguarded alternatives.
- [ ] Keep local TTS authorization-free.

## 4.3 Fix Allow once

Required:

```text
pending narration
  -> Allow once
  -> validate pending record
  -> mint authorization
  -> consume/invalidate pending record
  -> resume exact narration directly
  -> remote provider consumes authorization
```

- [ ] Do not re-enter ordinary privacy evaluation before approved dispatch.
- [ ] Ensure exactly one dispatch.
- [ ] Reject second consent action for same pending ID.

## 4.4 Session approval

- [ ] Update session policy.
- [ ] Mint capability for current pending narration.
- [ ] Resume current narration directly.
- [ ] Let future requests use normal evaluation.

## 4.5 Persistent approval

- [ ] Persist policy successfully before claiming persistence.
- [ ] Surface save failure.
- [ ] Do not silently downgrade to session approval.
- [ ] Mint capability for current operation after successful persistence.
- [ ] Resume current narration directly.

Tests:

- [ ] Local narration never prompts.
- [ ] Remote narration requires authorization.
- [ ] Allow once -> exactly one remote call.
- [ ] Deny -> zero remote calls.
- [ ] Cancel -> zero remote calls.
- [ ] Duplicate approval -> zero duplicate calls.
- [ ] Session allow -> current request succeeds immediately.
- [ ] Persistent allow -> current request succeeds after save.
- [ ] Persistence failure -> no false success.
- [ ] `local_only` -> remote narration rejected.

Acceptance:

- [ ] Known narration Allow-once bug is fixed and regression-tested.

# Phase 5 — Remote ASR/microphone privacy

## 5.1 Map transcription pipeline

- [ ] Document current request -> capture -> drain/finalize -> provider -> result flow.
- [ ] Identify seam for authorization before capture.
- [ ] Confirm provider locality is known early enough.
- [ ] Prove no hidden capture begins while consent is pending.

## 5.2 Enforce ASR capability

- [ ] Require ASR authorization at remote dispatch.
- [ ] Update all remote ASR callers.
- [ ] Remove/migrate unguarded wrappers.
- [ ] Keep local ASR authorization-free.

## 5.3 Consent before capture

Required:

```text
remote ASR request
  -> privacy evaluation
  -> authorization
  -> microphone capture
  -> drain/finalize
  -> remote dispatch
```

- [ ] Ensure remote privacy evaluation is before microphone capture.
- [ ] Store metadata only while consent is pending.
- [ ] Do not retain pending-request microphone audio.
- [ ] Begin fresh capture only after approval.
- [ ] Denial/cancellation while pending -> zero capture and zero upload.

## 5.4 Allow once/session/persistent ASR

- [ ] Allow once mints one ASR capability.
- [ ] Resume exact pending transcription directly.
- [ ] Capture starts only after capability creation.
- [ ] Capability is consumed by one remote dispatch.
- [ ] Duplicate/stale consent is rejected.
- [ ] Session approval updates session policy and authorizes current request.
- [ ] Persistent approval saves policy and authorizes current request.
- [ ] Persistence failure remains visible/fail-closed.

Tests:

- [ ] Local ASR needs no remote authorization.
- [ ] Remote ASR requires authorization.
- [ ] `local_only` rejects non-loopback remote ASR.
- [ ] supported loopback/local endpoint remains ungated.
- [ ] pending consent -> zero capture.
- [ ] deny -> zero capture/upload.
- [ ] cancel -> zero capture/upload.
- [ ] Allow once -> one capture/one remote dispatch.
- [ ] duplicate approval -> no duplicate upload.
- [ ] stale approval -> zero capture/upload.
- [ ] provider error -> capability cannot be reused.
- [ ] cancellation after approval but before dispatch -> no remote call.

Acceptance:

- [ ] Remote microphone data is never captured while consent for that request is unresolved.

# Phase 6 — Provider locality classification

- [ ] Find all speech-provider locality checks.
- [ ] Centralize duplicated classification where practical.
- [ ] Known local providers classify local.
- [ ] Explicitly supported loopback endpoints classify local.
- [ ] Non-loopback endpoints classify remote.
- [ ] Unknown/custom/ambiguous endpoints fail toward remote.
- [ ] Malformed URLs/config do not default to local.
- [ ] Remove fragile hostname substring assumptions.

Tests:

- [ ] `localhost` behavior.
- [ ] IPv4 loopback behavior.
- [ ] IPv6 loopback behavior where supported.
- [ ] non-loopback address behavior.
- [ ] malformed/unknown endpoint fails safely.

# Phase 7 — Pending consent state machine

Implement/verify semantics equivalent to:

```text
Pending
ApprovedAndConsumed
Denied
Cancelled
Expired
Superseded
```

- [ ] Only Pending can authorize.
- [ ] Terminal states reject later consent.
- [ ] Pending record consumption is atomic where practical.
- [ ] Consent is bound to operation ID/generation.
- [ ] Define explicit policy for a second request: reject, queue, or supersede.
- [ ] Do not silently replace a pending request.

Race tests:

- [ ] double-click Allow.
- [ ] duplicate frontend event.
- [ ] approval vs cancellation.
- [ ] approval vs expiration.
- [ ] duplicate provider dispatch.

Acceptance:

- [ ] Exactly one terminal outcome wins per pending operation.

# Phase 8 — Frontend consent integration

- [ ] Promote narration privacy challenges into the existing consent dialog.
- [ ] Promote ASR privacy challenges into the existing consent dialog.
- [ ] Do not collapse privacy challenges into generic speech/planner errors.
- [ ] TTS copy identifies remote narration/text transfer.
- [ ] ASR copy identifies remote microphone/audio transfer.
- [ ] ASR copy matches consent-before-capture behavior.
- [ ] Route Allow once correctly.
- [ ] Route session/persistent approval correctly.
- [ ] Route deny/cancel correctly.
- [ ] Surface stale consent failures.

Frontend tests:

- [ ] TTS challenge opens dialog.
- [ ] ASR challenge opens dialog.
- [ ] purpose-specific copy is correct.
- [ ] operation ID/action routing is correct.
- [ ] stale failure is visible.
- [ ] generic error handler does not mask privacy failures.

# Phase 9 — Settings UI

- [ ] Show active remote narration privacy policy.
- [ ] Show active remote microphone privacy policy.
- [ ] Reuse existing persisted microphone privacy policy where applicable.
- [ ] Preserve independence of narration/microphone policies where backend model is independent.
- [ ] Ensure settings edits use typed backend operations.
- [ ] Do not reintroduce legacy privacy adapters.
- [ ] Surface loading failures.
- [ ] Surface save failures.
- [ ] Surface invalid state.
- [ ] Never default failed config read to remote allow.

Tests:

- [ ] narration policy renders correctly.
- [ ] microphone policy renders correctly.
- [ ] save failure is surfaced.
- [ ] invalid config is not shown as allowed.

# Phase 10 — Unsafe fallback/silent failure audit

Rust review targets:

- [ ] `unwrap_or(...)`
- [ ] `unwrap_or_default()`
- [ ] `.ok()`
- [ ] `let _ =`
- [ ] ignored `Result`s
- [ ] log-and-continue behavior
- [ ] catch-all allow/default branches
- [ ] provider fallbacks
- [ ] compatibility bypasses

Frontend review targets:

- [ ] swallowed promise rejections.
- [ ] failed privacy fetch defaulting to permissive state.
- [ ] generic error while execution continues.
- [ ] fallback provider selection.
- [ ] stale callbacks ignored while dispatch continues.

Prove:

- [ ] local TTS failure does not silently invoke remote TTS.
- [ ] local ASR failure does not silently invoke remote ASR.
- [ ] remote provider failure does not silently invoke another provider.
- [ ] malformed privacy state does not become permission.

For every relevant fallback/failure:

- [ ] classify as safe/unsafe.
- [ ] remove unsafe behavior.
- [ ] document narrowly justified behavior.

# Phase 11 — Compatibility and bypass audit

- [ ] Search for legacy speech/privacy wrappers.
- [ ] Search for direct remote provider calls bypassing authorization.
- [ ] Search tests/helpers for capability-construction exposure.
- [ ] Search command handlers for alternate dispatch paths.
- [ ] Remove obsolete bypass wrappers.
- [ ] Update callers.
- [ ] Add structural absence tests where useful.

Acceptance:

- [ ] Each remote speech provider has one authoritative guarded entry path.

# Phase 12 — Error taxonomy and logging

Ensure explicit handling for:

- [ ] consent required.
- [ ] denied.
- [ ] cancelled.
- [ ] stale pending request.
- [ ] invalid policy.
- [ ] persistence failure.
- [ ] local-only violation.
- [ ] capture failure.
- [ ] remote-provider failure.
- [ ] authorization invariant failure.

Logging audit:

- [ ] no microphone audio in logs.
- [ ] no API keys/secrets in logs.
- [ ] avoid sensitive full speech payloads.
- [ ] safe operation/purpose/provider/outcome metadata only.

# Phase 13 — Required regression coverage

## 13.1 Rust unit tests

TTS:

- [ ] unauthorized remote dispatch impossible/fails.
- [ ] Allow once exactly once.
- [ ] deny/cancel zero dispatch.
- [ ] duplicate approval no duplicate dispatch.
- [ ] session approval current request succeeds.
- [ ] persistent approval current request succeeds.
- [ ] persistence failure explicit.
- [ ] local-only blocks remote.

ASR:

- [ ] unauthorized remote dispatch impossible/fails.
- [ ] pending consent zero capture.
- [ ] deny/cancel zero capture/upload.
- [ ] Allow once one capture/upload.
- [ ] duplicate approval no duplicate upload.
- [ ] stale approval zero capture/upload.
- [ ] local-only blocks remote.
- [ ] loopback/local remains ungated.
- [ ] provider failure cannot reuse capability.

State machine:

- [ ] pending record consumed at most once.
- [ ] expired cannot authorize.
- [ ] cancelled cannot authorize.
- [ ] wrong purpose cannot authorize.
- [ ] wrong operation binding cannot authorize where supported.

## 13.2 Integration tests

- [ ] remote TTS consent -> Allow once -> one success.
- [ ] remote TTS consent -> deny -> no provider call.
- [ ] remote ASR consent -> Allow once -> capture only after approval.
- [ ] remote ASR consent -> deny -> no capture/provider call.
- [ ] session consent current and next request.
- [ ] persistent consent current/later request.
- [ ] persistent-save failure.
- [ ] local-only local-vs-remote behavior.
- [ ] provider failure without substitution.

## 13.3 Frontend tests

- [ ] narration challenge promotion.
- [ ] ASR challenge promotion.
- [ ] purpose-specific dialog content.
- [ ] consent action routing.
- [ ] stale failure visibility.
- [ ] settings policy visibility.
- [ ] privacy errors not swallowed.

## 13.4 Structural tests

- [ ] remote TTS provider requires authorization type.
- [ ] remote ASR provider requires authorization type.
- [ ] constructors not publicly exported.
- [ ] obsolete unguarded wrappers absent.
- [ ] direct bypass call patterns absent.

# Phase 14 — Strict validation

Rust/permanent gates:

- [ ] Rust formatting check.
- [ ] repository whitespace/hygiene checks.
- [ ] privacy/fallback/security scanners.
- [ ] `cargo check`.
- [ ] strict all-target/all-feature Clippy with warnings denied.
- [ ] full Rust unit tests.
- [ ] full Rust integration/Wry tests where applicable.

Frontend:

- [ ] dependency/install integrity gate.
- [ ] frontend lint.
- [ ] frontend unit tests.
- [ ] UI tests.
- [ ] production build.

For every failure:

- [ ] record exact diagnostic.
- [ ] fix root cause.
- [ ] rerun required gate.
- [ ] do not broadly suppress.
- [ ] do not skip the failing test class.

# Phase 15 — Final correctness review

Authorization:

- [ ] Any remote TTS path without authorization?
- [ ] Any remote ASR path without authorization?
- [ ] Can authorization be forged via visibility/helper APIs?
- [ ] Can one capability be reused?
- [ ] Can TTS capability authorize ASR or vice versa?

Consent:

- [ ] Allow once exactly one pending operation?
- [ ] Any second privacy evaluation before approved dispatch?
- [ ] Session/persistent choice authorizes current request immediately?
- [ ] Stale consent rejected?

ASR lifecycle:

- [ ] Remote consent definitely precedes capture?
- [ ] Any path capturing while consent pending?
- [ ] Any denial/cancellation leaving queued audio?
- [ ] Any retry reusing old audio/authorization unexpectedly?

Fallback review:

- [ ] No fail-open branches.
- [ ] No silent local-to-remote substitution.
- [ ] No silent provider substitution.
- [ ] No quiet persistence failure.
- [ ] No generic error path that makes privacy failure appear successful.

# Phase 16 — Documentation/evidence

- [ ] Update `docs/BB_CODE_REVIEW3_TODO.md` P1.1 only after acceptance is proven.
- [ ] Reference this spec/TODO where appropriate.
- [ ] Preserve truthful P1.2 blocker status if still blocked.
- [ ] Record baseline SHA.
- [ ] Record implementation SHA.
- [ ] Record final documentation/evidence SHA if different.
- [ ] Record exact CI run ID.
- [ ] Record exact job ID(s).
- [ ] Record final CI conclusion.
- [ ] Record useful test counts.
- [ ] Record any narrowly justified lint suppression.
- [ ] Record remaining blockers/limitations.

# Phase 17 — Master safety

- [ ] Keep implementation isolated until full validation is green.
- [ ] Confirm final validated SHA contains exactly intended changes.
- [ ] Remove temporary workflow/checkpoint artifacts unless intentionally permanent.
- [ ] Merge/push only validated implementation.
- [ ] Re-run permanent CI on exact final `master` SHA if required.
- [ ] Do not claim completion until final `master` evidence is green.

# Final acceptance checklist

P1.1 is complete only when all are true:

- [ ] Remote TTS structurally requires authorization.
- [ ] Remote ASR structurally requires authorization.
- [ ] Authorization capabilities are crate-private/unforgeable from ordinary callers.
- [ ] Capabilities are purpose-specific.
- [ ] Allow once works exactly once.
- [ ] Allow once does not re-enter policy evaluation before approved dispatch.
- [ ] Session consent immediately authorizes current pending operation.
- [ ] Persistent consent immediately authorizes current pending operation after successful persistence.
- [ ] Persistent-save failure is explicit/fail-closed.
- [ ] Remote ASR consent happens before microphone capture.
- [ ] Pending remote ASR stores no captured audio.
- [ ] Denial/cancellation causes zero unauthorized capture/upload/dispatch.
- [ ] Local TTS/ASR remain outside remote authorization.
- [ ] `local_only` rejects remote providers.
- [ ] Unknown/ambiguous providers do not default to local.
- [ ] No silent provider substitution exists.
- [ ] No fail-open privacy fallback exists.
- [ ] Stale/duplicate consent cannot cause dispatch.
- [ ] Required Rust tests pass.
- [ ] Required integration tests pass.
- [ ] Required frontend/UI tests pass.
- [ ] Structural bypass tests pass.
- [ ] Privacy/fallback scanners pass.
- [ ] Strict Clippy passes unchanged.
- [ ] Production build passes.
- [ ] Exact final SHA is validated.
- [ ] `docs/BB_CODE_REVIEW3_TODO.md` is updated with truthful evidence.

# Completion evidence

Fill only after the Ralph Loop is complete.

```text
Baseline SHA:
Implementation SHA:
Final master SHA:
CI run:
CI job(s):
CI conclusion:

Rust tests:
Frontend tests:
UI tests:
Build:

Fallback/silent-failure audit:
Authorization bypass audit:
Remaining blocker(s):
```
