# BB_CODE_REVIEW.md

## Purpose

This document summarizes the current code review findings for the `blind_browser` project so GitHub Copilot has concrete implementation context for the companion TODO file.

## Review scope

Reviewed repository areas:

- `src/main.ts`
- `src/confirmation-panel.ts`
- `src/planner-orchestration.ts`
- `src/tauri-api.ts`
- `src-tauri/src/lib.rs`
- `src-tauri/src/app_core.rs`
- `src-tauri/src/commands.rs`
- `README.md`
- test and CI scaffolding

## What was validated during review

### Confirmed in this review environment

- `npm run test:ui` passed: 49 / 49 tests.
- Static review of TypeScript and Rust structure was completed.

### Not fully validated here

- Full Rust validation was not run because the Rust toolchain was not available in this review environment.
- Full frontend build validation was not conclusive here because `@tauri-apps/api/core` was unresolved in the local environment when running `npm run build`. That may be an environment or dependency setup issue rather than a source-code defect.

## What is good about the code

### 1. The architecture has a real separation of concerns

The project is not a random prototype. There is a visible split between:

- frontend UI rendering and orchestration
- Tauri invoke contracts
- Rust runtime state and command handling
- domain-specific browser, audio, OCR, ASR, TTS, and planner logic

That is a strong base for a Tauri application.

### 2. Type discipline is good

The TypeScript side uses explicit state shapes and contract types. The Rust side mirrors that with strongly structured command inputs and outputs. This is a good sign for long-term maintainability.

### 3. Error handling is better than average

The code is clearly trying to model failures intentionally instead of just throwing generic errors. That is especially useful for a voice-first application where user guidance matters.

### 4. HTML escaping is present in the confirmation panel

`src/confirmation-panel.ts:1582-1588` defines `escapeHtml()`, and the renderer uses it in multiple places. That is the right default for string-built HTML.

### 5. Test investment already exists

There is meaningful existing test coverage on the frontend side, and the Rust side appears to include a large unit-test footprint. CI is also present under `.github/workflows/ci.yml`.

### 6. Feature gating in Rust is clean

The feature structure in `src-tauri/Cargo.toml` is a good design choice because it keeps optional runtime capabilities explicit and easier to reason about.

## Main weaknesses

### 1. The frontend rendering model is too coarse

The single biggest technical issue is that the app rebuilds the entire UI with `app.innerHTML`.

Relevant code:

- `src/main.ts:384-441` — full app render via `app.innerHTML = ...`
- `src/main.ts:444-466` — `rerender()` calls full render
- `src/main.ts:468-609` — nearly every state setter triggers a full rerender
- `src/main.ts:2069-2153` — many `input` events update state on every keystroke

This pattern is risky because it can cause:

- lost input focus
- lost cursor position
- fragile event interactions
- unnecessary layout/repaint work
- accessibility regressions for keyboard and screen-reader users

For a voice-first and accessibility-focused app, this is a high-priority architectural problem.

### 2. Some critical files are too large

The following files are already larger than they should be:

- `src/main.ts` — about 2299 lines
- `src/confirmation-panel.ts` — about 1589 lines
- `src-tauri/src/commands.rs` — about 15009 lines

This does not make the code wrong by itself, but it raises change risk and review cost. It also makes hidden coupling more likely.

### 3. Some features are explicitly incomplete

Examples:

- `src/main.ts:248-254` — provider failover panel states that live runtime failover is not currently available
- `src-tauri/src/app_core.rs:1628-1638` — page snapshot scroll and viewport metrics are placeholder values
- `README.md:13-20` — current status still describes the app as a Phase 0 setup, which understates the actual amount of implementation now present

These are not all bugs, but they are implementation and product-completeness gaps.

### 4. Frontend testing is narrow relative to current behavior complexity

The current frontend tests are useful, but they mostly validate rendered output. They do not yet appear to cover several higher-risk behaviors such as:

- focus preservation during input edits
- event-handler interactions during async saves
- partial refresh behavior when one runtime fetch succeeds and another fails
- state rollback correctness after failed saves

## Concrete issues and likely bugs

### Issue 1: Full rerendering can break focus, typing, and accessibility

Relevant code:

- `src/main.ts:384-441`
- `src/main.ts:444-466`
- `src/main.ts:468-609`
- `src/main.ts:2069-2153`

Because inputs write state on `input` and every setter rerenders the entire app, the DOM can be replaced repeatedly while the user is typing.

Likely effects:

- text cursor jumps or resets
- focus may move unexpectedly
- assistive technology can lose context
- controls may feel unstable under fast typing or slider movement

Even if some controls appear to work today, this is still a fragile pattern and should be corrected.

### Issue 2: Global busy gating in the `change` handler can drop unrelated user actions

Relevant code:

- `src/main.ts:2156-2173`

The current `change` handler exits early when *any* of several panel states are busy.

That means one save can block unrelated controls. For example:

- volume save is in flight
- user changes ASR provider or another unrelated field
- the handler may silently ignore that change

This is a logic bug and a UX bug. Busy state should be scoped to the specific control or specific panel involved in the current interaction.

### Issue 3: TTS provider failure handling propagates errors inconsistently

Relevant code:

- `src/main.ts:1245-1279`

On provider-switch failure:

- the provider panel gets the new failure message
- the TTS model panel and TTS voice panel revert to previous values
- but those subordinate panels restore their previous `error` fields instead of receiving the new failure message

That can leave the UI in a confusing state where:

- the provider panel shows the actual current failure
- the dependent panels show stale error text or no error at all

This is not fatal, but it is inconsistent and will make diagnosis harder.

### Issue 4: Runtime refresh is too coarse and loses partial success

Relevant code:

- `src/main.ts:997-1005`
- `src/main.ts:1022-1050`

`refreshRuntimePanelsFromRuntime()` uses a coarse `Promise.all(...)` flow and then applies a generic error across multiple panels if anything fails.

That means:

- one successful runtime fetch can be discarded because another failed
- unrelated panels can receive the same generic error
- valid state can be overwritten too aggressively

This should be refactored to support partial success and panel-specific error recovery.

### Issue 5: Product signaling and implementation state are out of sync

Relevant code:

- `README.md:13-20`
- `src/main.ts:387-393`

The app and README still present themselves as a thin Phase 0 scaffold, but the codebase now includes much more runtime, settings, and interaction logic. That mismatch can confuse contributors and any coding agent trying to decide what is already real versus still placeholder.

## Areas that are incomplete but not necessarily defective

### Provider failover panel

`src/main.ts:248-254` clearly marks automatic provider failover as unavailable. That is acceptable if intentional, but the codebase should make that status explicit and consistent in both UI and docs.

### Page snapshot metrics

`src-tauri/src/app_core.rs:1628-1631` currently returns placeholder zeros for:

- `scroll_y`
- `viewport_width`
- `viewport_height`
- `document_height`

If the browser backend is not ready yet, that is fine as a temporary measure. But it should either be implemented or clearly feature-flagged and documented so downstream consumers do not treat those values as real.

## Refactor targets

### Frontend

Primary refactor target:

- `src/main.ts`

Strong candidates for extraction:

- state store helpers
- panel-specific render functions
- panel-specific action handlers
- async persistence flows
- event delegation and input/change handling
- runtime refresh / reconciliation logic

### Confirmation UI

Strong candidate for separation:

- `src/confirmation-panel.ts`

Possible extraction boundaries:

- state-to-view formatting helpers
- backend failure formatting helpers
- submission-state rendering
- generic markup helpers

### Rust command layer

Strong candidate for modularization:

- `src-tauri/src/commands.rs`

Possible extraction boundaries:

- planner commands
- browser commands
- narration/audio commands
- ASR commands
- TTS commands
- settings/config commands
- test-only helpers

## Testing gaps that should be closed

### Frontend tests to add

- focus preservation during text input edits
- cursor preservation while typing into live-rendered controls
- slider/input updates that do not remount controls
- panel-specific busy-state behavior
- TTS provider-switch failure propagation
- partial runtime refresh when one backend call fails and another succeeds
- regression tests for URL input and API-key draft editing under rerender pressure

### Rust / integration tests to add or strengthen

- tests for real page snapshot metrics once implemented
- command tests around failover capability reporting if that feature is added
- settings persistence tests around panel-specific error handling and rollback

## Suggested implementation priority

### Highest priority

1. Replace full-app rerendering with targeted updates or a more structured rendering model.
2. Fix global busy-state gating so unrelated controls are not silently dropped.
3. Fix inconsistent TTS provider failure propagation.
4. Refactor runtime refresh to support partial success.

### Medium priority

5. Split oversized frontend and Rust modules.
6. Expand interaction-focused tests.
7. Update README and UI status copy to match actual implementation state.

### Lower priority but still worthwhile

8. Implement or explicitly gate placeholder page snapshot metrics.
9. Clarify or implement provider failover capability reporting.

## Bottom line

This is a good codebase with real structure and a better-than-average foundation. The main risk is not that the project is sloppy. The main risk is that the frontend rendering model is now too primitive for the amount of stateful behavior the app already has.

The most important thing for implementation is to treat the rerender architecture, busy-state scoping, error-propagation consistency, and partial-refresh logic as the first wave of work. The rest should follow after those core interaction and correctness issues are stabilized.
