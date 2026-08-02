# BBCR-005 Runtime State Binding

## Authority model

The remote planner receives an opaque `runtime_state_token` in `PlannerInput`, but the token is informational only. Rust preserves the authoritative `PlanningStateSnapshot` server-side and binds it to the exact serialized `PlannerOutput` digest. Planner output cannot create, replace, or weaken that snapshot.

## State represented by the token

The token is a SHA-256 digest over:

- current page ID;
- page/document generation;
- normalized origin;
- browser-history position and boundaries;
- deterministic safety settings;
- a relevant-configuration fingerprint covering provider selections, OCR policy, browser visibility, runtime audio, and listening state;
- pending confirmation identity.

The token deliberately excludes timestamps so two captures of unchanged state produce the same token. Issue and expiry timestamps remain server-side in the snapshot record.

## Tool invalidation matrix

| State change | Invalidated operations | Runtime response |
|---|---|---|
| Navigation, reload, back/forward, page ID change, origin change | Click, focus, type, submit, page-relative narration, OCR merge, navigation-dependent side effects | Reject current plan and enter the bounded replan loop |
| Page-model replacement, OCR merge, or same-page DOM mutation | Element-targeted click/focus/type/submit and confirmations containing those targets | Increment page generation, clear click authorizations and pending confirmation, then replan |
| Safety-setting change | Every side effect resolved under the prior confirmation policy | Reject current plan and replan under current settings |
| Provider selection, OCR policy, browser visibility, audio, or listening-state change | Side effects whose semantics depend on the changed configuration/state | Reject current plan and replan |
| Confirmation created, consumed, cleared, or replaced | Any plan resolved against the prior pending-confirmation state | Reject current plan and replan |
| Unrelated state change during a status-only/read-only plan | `GetAgentState`, `GetRuntimeStatus`, `GetCurrentUrl`, and equivalent non-mutating reads | No snapshot requirement; execute normally |

## Click authorization

A click requires a runtime-owned opaque authorization record bound to page ID, page generation, origin, element ID, DOM locator, element fingerprint, deterministic confidence, ambiguity, destructive classification, issue time, and expiry. Immediately before dispatch, Rust re-extracts the live DOM and re-resolves the target. Changed, hidden, disabled, stale, expired, ambiguous, low-confidence, or destructive targets cannot use the click-without-confirmation exception.

## Confirmation interaction

Confirmation manifests include the generation-qualified page identity. On confirmation response, Rust rebuilds the manifest and revalidates every queued click against the live DOM before consuming the single-use pending state. A page-generation change clears pending confirmation state before a stale response can resume execution.

## Concurrency

`AppCore` mutations remain serialized by its mutex. Releasing the lock during remote planning is safe because execution must consume the server-side snapshot for that exact output. If another frontend command changes relevant state while planning is in flight, execution returns `NeedsReplan`; the voice-command orchestrator permits at most the configured bounded replan count.
