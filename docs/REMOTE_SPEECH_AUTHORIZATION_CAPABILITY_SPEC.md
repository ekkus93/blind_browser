# Remote Speech Authorization Capability Specification

**Project:** Blind Browser
**Status:** Implementation specification
**Related review:** `docs/BB_CODE_REVIEW3_TODO.md`

## 1. Purpose

Blind Browser must never send speech-related user data to a remote provider unless that exact operation has been authorized by the active privacy policy. This specification defines the required authorization model for remote TTS/narration and remote ASR/transcription.

The design is intentionally fail-closed. Authorization is represented as an explicit capability that must cross the remote-provider boundary. Consent is resolved before sensitive microphone capture for remote ASR. Silent fallbacks, permissive defaults, and compatibility bypasses are prohibited.

## 2. Problem statement

Two issues motivate this work.

First, privacy decisions and provider execution are currently separable enough that a future call site could accidentally bypass the intended gate unless the remote provider itself requires proof of authorization.

Second, one-shot consent can be consumed too early. The broken pattern is:

```text
Allow once
  -> consume one-shot grant
  -> restart original operation
  -> evaluate privacy policy again
  -> no one-shot grant remains
  -> ask again
```

The correct model is:

```text
request
  -> privacy evaluation
  -> authorization capability
  -> exactly one approved remote operation
```

Once a capability has been minted for a pending operation, that exact operation must not re-enter ordinary privacy evaluation before execution.

## 3. Goals

The implementation MUST:

1. Make remote TTS dispatch structurally require authorization.
2. Make remote ASR dispatch structurally require authorization.
3. Fix `Allow once` so one approval authorizes one operation.
4. Resolve remote ASR consent before microphone capture.
5. Keep local speech outside the remote-consent mechanism.
6. Fail closed on invalid, missing, ambiguous, or unavailable privacy state.
7. Reject stale, duplicate, cancelled, or mismatched consent actions.
8. Prevent silent local-to-remote or remote-to-remote provider substitution.
9. Preserve all existing strict compiler, Clippy, privacy, lint, and test gates.
10. Add regression tests that make future bypasses difficult.

## 4. Non-goals

This work does not require redesigning the complete privacy subsystem, adding providers, introducing accounts/cloud state, weakening scanners, or retaining obsolete compatibility APIs that conflict with the typed authorization path.

## 5. Terminology

**Remote speech operation:** A TTS or ASR operation that transmits user-derived data to a non-local endpoint.

**Privacy decision:** The result of evaluating current policy and consent state for a specific operation.

**Authorization capability:** A crate-internal, unforgeable value proving that the applicable remote operation has already been approved.

**Pending consent operation:** Metadata describing an operation awaiting user consent. For ASR it must not contain recorded microphone audio while consent is unresolved.

## 6. Non-negotiable security invariants

### INV-1 — No capability, no remote dispatch

Every remote TTS and remote ASR provider entry point MUST require the appropriate authorization capability in its function/type-state boundary.

### INV-2 — Capability construction is restricted

Authorization constructors MUST remain private to the trusted privacy/controller layer. Ordinary call sites, frontend commands, provider wrappers, and tests must not be able to mint authorization freely.

### INV-3 — Purpose separation

TTS authorization must not authorize ASR. ASR authorization must not authorize TTS.

### INV-4 — Capabilities are consumed

One-operation capabilities SHOULD be passed by value into provider dispatch and consumed there. Accidental reuse must be difficult.

### INV-5 — `Allow once` does not re-evaluate before dispatch

After approval, validate the pending operation, mint authorization, consume/invalidate the pending record, and resume the exact operation directly. Do not run ordinary policy evaluation again before that dispatch.

### INV-6 — Remote ASR consent precedes capture

If remote ASR requires consent, resolve consent before microphone capture begins for that remote request. While consent is pending, retain metadata only.

### INV-7 — Denial/cancellation means zero unauthorized activity

Denial or cancellation must produce zero remote dispatches. For pending remote ASR it must also produce zero microphone capture for that request.

### INV-8 — Local speech is not remotely gated

Local TTS and local ASR do not need remote authorization and must remain usable when remote consent is denied.

### INV-9 — `local_only` fails closed

A genuinely remote endpoint must be rejected under `local_only`. Do not silently reinterpret it as local and do not silently switch providers.

### INV-10 — Invalid policy fails closed

Malformed, unavailable, contradictory, or unknown privacy state must never become implicit permission.

### INV-11 — Consent is operation-bound

A response for one pending operation must not authorize another operation, another purpose, or an expired/superseded operation.

### INV-12 — No silent provider substitution

Failure of a local provider must not silently invoke a remote one. Failure of one remote provider must not silently select another provider with different privacy characteristics.

## 7. Authorization type model

Preferred conceptual model:

```rust
pub(crate) struct RemoteTtsAuthorization {
    // private proof/state
}

pub(crate) struct RemoteAsrAuthorization {
    // private proof/state
}
```

A generic capability is acceptable only if type parameters make purpose mixing impossible.

Capabilities MUST NOT be persisted or restored across process restarts.

## 8. Privacy decision model

Privacy evaluation should yield an explicit outcome equivalent to:

```text
Authorized(capability)
ConsentRequired(pending_id)
Denied(reason)
```

Unknown or invalid states belong on the denied/error side, not the allowed side.

Already-authorized session/persistent policy:

```text
request -> evaluate -> mint capability -> dispatch
```

Consent required:

```text
request -> evaluate -> store pending metadata -> show dialog
```

No provider call occurs. For remote ASR, no microphone capture occurs.

## 9. `Allow once` semantics

Required flow:

```text
pending operation
  -> user selects Allow once
  -> validate pending operation is still live
  -> mint one-operation capability
  -> atomically consume/invalidate pending record
  -> resume exact operation directly
  -> provider consumes capability
```

Do not convert Allow once into a temporary boolean that is then consumed by another policy evaluation.

## 10. Session consent semantics

For session approval:

1. Validate the pending operation.
2. Update in-memory session policy.
3. Mint authorization for the current pending operation.
4. Resume the current operation directly.
5. Let future matching operations use ordinary policy evaluation to mint new capabilities.

The current operation must not be forced through a second policy evaluation.

## 11. Persistent consent semantics

For persistent approval:

1. Validate the pending operation.
2. Persist the requested policy successfully.
3. Update active policy state.
4. Mint authorization for the current pending operation.
5. Resume that operation directly.

If persistence fails, surface the error. Do not pretend the policy was saved, silently downgrade to session approval, or silently dispatch under weaker semantics.

## 12. Remote TTS/narration flow

```text
narration request
  -> classify provider locality
  -> local: execute locally
  -> remote: evaluate privacy
      -> authorized: RemoteTtsAuthorization -> remote provider
      -> consent required: pending metadata -> consent UI
      -> denied/error: explicit failure
```

Conceptual provider boundary:

```rust
fn synthesize_remote(
    request: RemoteTtsRequest,
    authorization: RemoteTtsAuthorization,
) -> Result<...>
```

All alternate remote-provider call paths must be migrated or removed.

## 13. Remote ASR/transcription flow

Required ordering:

```text
transcription request
  -> classify provider locality
  -> if remote, evaluate microphone privacy
  -> obtain RemoteAsrAuthorization
  -> begin microphone capture
  -> drain/finalize audio
  -> remote ASR dispatch consuming authorization
  -> record/result handling
```

When consent is required:

```text
request -> privacy evaluation -> pending metadata -> consent UI
```

At this point there must be no recorded audio retained for the request and no remote traffic.

After approval:

```text
validate pending operation
  -> mint RemoteAsrAuthorization
  -> start fresh capture
  -> finalize audio
  -> dispatch once
```

Cancellation after approval but before dispatch must prevent remote dispatch if it has not already begun, discard temporary audio under existing lifecycle rules, and prevent capability reuse.

## 14. Provider locality classification

Provider locality must be explicit and deterministic. Centralize classification where practical.

- Known local providers are local.
- Explicitly supported loopback endpoints may be local.
- Non-loopback HTTP(S) endpoints are remote.
- Unknown/custom/ambiguous endpoints should be treated as remote unless proven local.
- Malformed endpoint configuration must not be interpreted as local by default.

Avoid fragile hostname substring tests.

## 15. Pending consent data model

A pending record should contain only what is needed to display, validate, bind, and resume the operation, for example:

```text
id
purpose
target provider identity
request binding/fingerprint or generation
created_at / expiration
display-safe metadata
```

For unresolved remote ASR consent it MUST NOT contain captured microphone audio.

## 16. Pending-operation lifecycle

Use explicit semantics equivalent to:

```text
Pending
  -> ApprovedAndConsumed
  -> Denied
  -> Cancelled
  -> Expired
  -> Superseded
```

Only `Pending` may produce authorization. All terminal states reject later consent actions.

Behavior for a second request while one is pending must be explicit: reject, queue, or supersede. Silent replacement is not acceptable.

## 17. Concurrency requirements

Protect against:

- double-clicking Allow;
- duplicate frontend messages;
- two threads consuming the same pending record;
- approval racing cancellation/expiration;
- stale UI responses;
- duplicated provider dispatch.

Exactly one terminal outcome should win per pending operation. Pending consumption should be atomic where practical.

## 18. Error handling

Keep privacy failures distinguishable from generic provider failures where practical:

- consent required;
- denied;
- cancelled;
- stale pending operation;
- invalid policy;
- persistence failure;
- local-only violation;
- capture failure;
- remote-provider failure;
- authorization mismatch/internal invariant failure.

Do not catch, log, and continue for privacy-sensitive failures. Do not include microphone audio, secrets, or unnecessarily sensitive speech content in logs.

## 19. Frontend consent UI

Reuse the existing consent UI where appropriate, but make the purpose explicit.

TTS/narration consent must state that narration text will be sent to a remote speech service.

ASR consent must state that microphone audio will be captured/sent to a remote speech service, and the UI must be consistent with the invariant that capture begins only after approval.

Privacy challenges must be promoted into visible consent-dialog state rather than collapsed into generic planner/speech errors.

## 20. Settings UI

Expose the privacy policy governing:

- remote narration/TTS;
- remote microphone/ASR.

Where the backend already persists independent policies, represent them independently. Loading/save/configuration failures must be visible. Never default a failed settings read to remote allow.

## 21. Provider-boundary audit

For every speech provider entry point, answer:

1. Is it local or remote?
2. If remote, does it require the correct capability?
3. Can another wrapper bypass that parameter?
4. Can a compatibility API reach it unguarded?
5. Can tests or public APIs freely construct authorization?
6. Can a generic endpoint path invoke remote service without authorization?

Any bypass must be removed, migrated, or structurally restricted.

## 22. Prohibited fallbacks

Unless separately specified and explicitly visible to the user, prohibit:

- local provider failure -> remote provider;
- selected remote provider failure -> another provider;
- persistent-consent failure -> session consent;
- invalid/missing privacy config -> allow;
- missing privacy controller -> allow;
- stale consent -> allow;
- consent UI failure -> allow;
- capability-construction failure -> allow.

When no safe path exists, return an explicit error.

## 23. Compatibility policy

Do not preserve obsolete speech/privacy APIs if they undermine typed authorization. Compatibility wrappers must not construct capabilities, bypass operation binding, or invoke remote providers without authorization.

## 24. Required Rust tests

TTS:

- local TTS needs no remote capability;
- remote TTS requires authorization;
- `local_only` rejects remote TTS;
- Allow once dispatches exactly once;
- duplicate approval cannot dispatch twice;
- deny/cancel dispatch zero times;
- session approval authorizes current operation immediately;
- persistent approval authorizes current operation after successful persistence;
- persistence failure does not appear successful.

ASR:

- local ASR needs no remote capability;
- remote ASR requires authorization;
- `local_only` rejects non-loopback ASR;
- supported loopback/local ASR remains ungated;
- pending remote consent captures zero audio;
- deny/cancel captures and uploads zero audio;
- Allow once causes one capture/one dispatch;
- duplicate/stale approval cannot upload;
- provider failure cannot reuse capability.

State machine:

- pending operation is consumed at most once;
- expired/cancelled/superseded operation cannot authorize;
- wrong purpose cannot authorize;
- mismatched operation binding cannot authorize where binding exists.

## 25. Required integration tests

1. Remote narration -> consent -> Allow once -> one success.
2. Remote narration -> deny -> zero provider calls.
3. Remote ASR -> consent -> Allow once -> capture begins after approval.
4. Remote ASR -> deny -> zero capture/provider calls.
5. Session consent -> current request and next matching request behave correctly.
6. Persistent consent -> persistence succeeds -> current/later matching requests work.
7. Persistent-consent persistence failure -> explicit failure and no false state.
8. `local_only` -> local provider works, remote provider is rejected.
9. Provider failure -> no silent substitution.

## 26. Required frontend tests

Verify:

- TTS challenge opens the consent dialog;
- ASR challenge opens the consent dialog;
- correct purpose-specific copy is shown;
- Allow once/session/persistent/deny/cancel route to the correct operation;
- stale backend responses are visible;
- narration and microphone privacy settings display correctly;
- privacy-specific failures are not swallowed by generic speech error handling.

## 27. Structural regression tests

Where practical, add tests/scanners proving:

- remote TTS provider requires `RemoteTtsAuthorization` or equivalent;
- remote ASR provider requires `RemoteAsrAuthorization` or equivalent;
- authorization constructors are not publicly exported;
- obsolete unguarded wrappers are absent;
- no fallback path directly invokes remote speech providers.

## 28. Unsafe fallback / silent failure audit

Review modified and adjacent code specifically for:

- `unwrap_or(...)` and `unwrap_or_default()` used as permissive privacy defaults;
- `.ok()` on privacy-sensitive results;
- `let _ =` for policy/persistence/provider errors;
- ignored `Result`s;
- log-and-continue behavior;
- broad catch-all branches;
- silent provider selection/substitution;
- stale compatibility APIs;
- duplicate dispatch paths.

Frontend review should include swallowed promise rejections, failed privacy reads defaulting to permissive values, generic errors while execution continues, and fallback provider selection.

Every relevant occurrence must be explicitly classified and either justified or removed.

## 29. Validation requirements

The implementation is not complete until the exact final source SHA passes all applicable permanent gates, including:

```text
rustfmt
repository whitespace/hygiene checks
privacy/fallback/security scanners
cargo check
strict all-target/all-feature Clippy
full Rust unit/integration tests
frontend lint
frontend unit/UI tests
production frontend build
```

Do not weaken validation to get green:

- no removal of `-D warnings`;
- no broad Clippy suppression;
- no disabling privacy scanners;
- no skipping failing test classes;
- no marking completion based on a different SHA.

## 30. Clippy policy

Treat Clippy failures as implementation feedback. Prefer, in order:

1. idiomatic ownership/visibility/API fixes;
2. simplification of unnecessary wrappers;
3. removal of dead/unreachable code;
4. signature corrections;
5. narrower visibility;
6. a narrowly scoped, documented suppression only for a genuine false positive or intentional construct.

## 31. Completion criteria

This work is complete only when:

1. Remote TTS requires typed authorization.
2. Remote ASR requires typed authorization.
3. Allow once authorizes exactly one approved operation.
4. There is no second privacy evaluation between one-shot approval and approved dispatch.
5. Session/persistent approval immediately authorizes the current pending operation.
6. Remote ASR consent precedes microphone capture.
7. Denial/cancellation causes zero unauthorized capture/upload/dispatch.
8. Local speech remains outside remote authorization.
9. `local_only` fails closed for remote providers.
10. Unknown endpoints do not default to local.
11. No silent provider substitution exists.
12. All bypass call sites are removed or guarded.
13. Required Rust, integration, frontend, and structural tests exist and pass.
14. The fallback/silent-failure audit is complete.
15. The exact final implementation SHA passes permanent CI.
16. `docs/BB_CODE_REVIEW3_TODO.md` is updated only after evidence supports completion.

## 32. Architectural summary

```text
request
  -> classify provider
      -> local -> local execution
      -> remote -> privacy evaluation
                     -> denied/error -> explicit failure
                     -> consent needed -> pending metadata -> consent UI
                     -> authorized -> capability -> remote provider
```

For remote ASR specifically:

```text
remote ASR request
  -> privacy authorization
  -> RemoteAsrAuthorization
  -> microphone capture
  -> finalize audio
  -> remote provider
```

That ordering is mandatory.
