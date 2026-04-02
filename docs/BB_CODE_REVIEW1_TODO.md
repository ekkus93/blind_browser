# BB_CODE_REVIEW_TODO.md

## Goal

Fix the major architectural, correctness, UX, accessibility, testing, and documentation issues identified in the code review for `blind_browser`.

This TODO is written as an implementation plan for GitHub Copilot. It is intentionally detailed and task-oriented.

## Implementation strategy

Work in this order unless a dependency forces a different sequence:

1. Build a safety net with tests.
2. Fix the frontend rerender architecture.
3. Fix control-scoped busy-state behavior.
4. Fix TTS provider failure propagation.
5. Refactor runtime refresh to support partial success.
6. Address incomplete placeholders and documentation drift.
7. Modularize oversized files.
8. Run final validation and cleanup.

---

## 1. Establish a safety net before major refactors — IN PROGRESS

### 1.1 Add frontend regression tests for current high-risk behaviors — IN PROGRESS

- Create targeted tests for `src/main.ts` behavior, not just pure render output.
- Add tests that simulate user interactions through the DOM and event listeners.
- Add tests for live `input` handling on:
  - URL input
  - remote API key fields
  - model directory input
  - slider controls
- Add tests for `change` handling on:
  - volume
  - speed
  - ASR provider
  - TTS provider
  - TTS model
  - TTS voice
  - confirmation settings
  - OCR threshold settings

### 1.2 Add focus and cursor regression tests — PENDING

- Add a test proving that typing into text fields does not:
  - drop focus
  - reset cursor position
  - lose partially typed values
- Add a test for slider interaction proving that moving a control does not remount the entire form subtree.
- Add a test for URL input proving the draft value remains intact during rerender-related state changes.

### 1.3 Add busy-state regression tests — DONE

- Add a test that proves a busy volume save does **not** block unrelated controls.
- Add a test that proves a busy TTS panel does **not** block ASR interactions.
- Add a test that duplicate submissions for the **same** control are prevented.
- Add a test that the correct controls become disabled while their own save is in flight.

### 1.4 Add TTS provider failure regression tests — DONE

- Add a test covering provider-switch failure where:
  - provider selection rolls back correctly
  - model state rolls back correctly
  - voice state rolls back correctly
  - the current failure message is visible consistently wherever intended
- Add a test ensuring stale panel errors are not preserved when a new provider-switch error occurs.

### 1.5 Add runtime refresh regression tests — DONE

- Add a test where `getAgentState()` succeeds but `getModelManagementSettings()` fails.
- Add a test where `getModelManagementSettings()` succeeds but `getAgentState()` fails.
- Add a test ensuring successful panel data is still applied when another fetch fails.
- Add a test ensuring errors are panel-specific instead of globally sprayed everywhere.

### 1.6 Improve validation scripts and local review instructions — PENDING

- Confirm the intended install/build workflow for the frontend and Tauri bridge.
- Make sure contributors know how to run:
  - frontend tests
  - TypeScript build
  - Rust tests
  - Rust linting and formatting
- If needed, update the README or docs so the required local setup is obvious.

---

## 2. Replace full-app rerendering with targeted UI updates — IN PROGRESS

### 2.1 Audit current rerender behavior in `src/main.ts` — DONE

- Identify every state setter that calls `rerender()`.
- Group state by owning panel or owning UI area.
- Identify which state changes truly require structural rerendering and which only require targeted value updates.
- Identify controls that currently update on every `input` event.

### 2.2 Introduce a more structured frontend update model — IN PROGRESS

Implement one of the following approaches consistently:

- targeted DOM patch/update functions per panel, or
- small panel-level components with stable DOM roots, or
- a minimal internal store + subscriber model where only affected sections rerender

The important constraint is this:

- do **not** continue replacing the entire `app.innerHTML` tree for routine state changes

### 2.3 Preserve stable DOM roots for interactive panels — IN PROGRESS

- Create persistent root elements for major panels.
- Ensure text inputs, selects, sliders, and buttons are not recreated unnecessarily.
- Only update the affected panel subtree when its state changes.
- Keep event delegation or direct listeners stable after updates.

### 2.4 Separate “initial render” from “state update” — IN PROGRESS

- Keep one function for initial DOM creation.
- Add update functions for each panel, for example:
  - `updateAudioControlsPanel(...)`
  - `updateUrlInputPanel(...)`
  - `updateTtsProviderPanel(...)`
  - `updateTtsModelPanel(...)`
  - `updateTtsVoicePanel(...)`
  - `updateStatusPanel(...)`
  - `updateConfirmationPanel(...)`
- Ensure those functions patch content instead of recreating the whole app shell.

### 2.5 Preserve user interaction state during updates — IN PROGRESS

- Preserve focused element where possible.
- Preserve input selection and cursor position for text inputs.
- Preserve slider thumb interaction continuity.
- Do not clear draft text while async operations are in flight unless explicitly intended.

### 2.6 Re-check accessibility after refactor — PENDING

- Ensure labels remain associated with inputs.
- Ensure ARIA descriptions and busy indicators still work.
- Ensure screen-reader-announced status regions are not spammed by full subtree replacement.
- Verify keyboard navigation order remains stable.

### 2.7 Add acceptance criteria for rerender refactor — PENDING

Refactor is complete only when:

- typing in text inputs does not lose focus or cursor position
- unrelated panels do not remount on each keystroke
- slider movement is smooth
- event listeners continue to work after updates
- existing UI tests still pass
- new focus/cursor regression tests pass

---

## 3. Replace global busy gating with panel-scoped busy behavior — IN PROGRESS

### 3.1 Audit current global gate in the `change` handler — DONE

- Review `src/main.ts` logic around the shared busy-state early return.
- Identify all controls currently blocked by unrelated busy flags.
- Document the desired busy owner for each setting.

### 3.2 Define ownership for each busy state — IN PROGRESS

For each interactive control, decide which busy flag should block it.

Examples:

- volume control should only be blocked by volume/audio-control save state
- speed control should only be blocked by speed/audio-control save state
- ASR provider control should only be blocked by ASR-provider save state
- TTS provider/model/voice controls should be blocked by their own relevant TTS state
- confirmation settings controls should be blocked only by confirmation-settings save state
- OCR threshold controls should be blocked only by OCR-threshold save state
- model-management actions should be blocked only by model-management operations

### 3.3 Refactor event handling to use scoped guards — DONE

- Remove the single broad early return that blocks everything.
- Replace it with control-specific or panel-specific guard logic.
- If necessary, create helper predicates such as:
  - `isAudioControlBusy(...)`
  - `isTtsProviderActionBusy(...)`
  - `isAsrProviderActionBusy(...)`
  - `isConfirmationSettingsBusy(...)`
- Keep the logic readable and explicit.

### 3.4 Update control disabling logic in the UI — IN PROGRESS

- Ensure disabled attributes match the new scoped busy semantics.
- Do not show a control as enabled if its handler will ignore the event.
- Do not disable unrelated controls during a narrow save.

### 3.5 Prevent duplicate in-flight operations for the same control — DONE

- Ensure repeated user actions on the same control are ignored or deduplicated safely.
- Do not allow rapid double-submit on a single panel to create race conditions.
- Consider request ownership IDs if that simplifies deconfliction.

### 3.6 Validate busy-state behavior — IN PROGRESS

- Confirm unrelated controls remain interactive during independent saves.
- Confirm same-control duplicate requests are blocked.
- Confirm the UI reflects actual interactivity accurately.

---

## 4. Fix TTS provider failure propagation and rollback consistency — DONE

### 4.1 Review the provider-switch flow end to end — DONE

- Review the current implementation of `persistTtsProviderSelection(...)`.
- Identify the intended rollback behavior for:
  - provider mode
  - model profile
  - voice selection
  - error state
  - busy state

### 4.2 Decide the desired failure UX explicitly — DONE

When provider switching fails, decide and document:

- which panels should display the new failure message
- which panels should keep their previous selected value
- whether dependent panels should show inherited provider failure copy or only the provider panel should show it

Recommendation:

- restore previous values for provider/model/voice
- clear busy flags everywhere involved
- surface the *current* failure message consistently where users need it
- avoid stale error text from previous unrelated failures

### 4.3 Refactor rollback into a transactional helper — DONE

- Create a helper that captures pre-change TTS state before a provider switch.
- Create a rollback path that restores values deterministically.
- Make error propagation explicit rather than ad hoc.
- Avoid hand-updating related panels in slightly different ways.

### 4.4 Normalize error propagation across related TTS panels — DONE

- Decide whether model and voice panels should:
  - receive the current provider-switch error, or
  - clear their own local error and rely on the provider panel only
- Implement the chosen policy consistently.
- Ensure stale `previous*.error` values are not incorrectly restored.

### 4.5 Add logging for TTS provider transitions — DONE

- Add debug-level logs for:
  - attempted provider transition
  - provider transition success
  - provider transition rollback
  - propagated error message
- Keep logs structured enough to diagnose future issues.

### 4.6 Validate the final TTS interaction behavior — DONE

- Provider switch success updates all relevant panels correctly.
- Provider switch failure restores prior selections correctly.
- Error messaging is consistent and current.
- No stale subordinate errors remain after failure.

---

## 5. Refactor runtime refresh to support partial success and panel-specific errors — DONE

### 5.1 Audit `refreshRuntimePanelsFromRuntime()` — DONE

- Identify every runtime call made during refresh.
- Identify which panels depend on each call.
- Identify where successful results are currently discarded because another call failed.

### 5.2 Replace coarse `Promise.all(...)` behavior with partial reconciliation — DONE

Implement a safer pattern such as:

- independent awaits with individual `try/catch`, or
- `Promise.allSettled(...)`, followed by targeted reconciliation

The desired behavior:

- successful calls should still update the panels they own
- failed calls should affect only the panels that depend on them

### 5.3 Build per-call result handlers — DONE

- Create dedicated reconciliation helpers for agent-state-derived panels.
- Create dedicated reconciliation helpers for model-management-derived panels.
- Only clear or overwrite the fields actually owned by the call that succeeded.
- Do not reset unrelated panel state on a partial failure.

### 5.4 Improve user-facing error specificity — DONE

- Replace generic catch-all error fan-out with scoped error messages.
- Distinguish between:
  - agent/runtime-state refresh failure
  - model-management settings refresh failure
  - audio settings refresh failure, if applicable
- Ensure panels show errors relevant to what actually failed.

### 5.5 Preserve good state on partial failures — DONE

- Keep the last known good value for unaffected panels.
- Avoid resetting busy flags or values unnecessarily across unrelated panels.
- Ensure successful data continues to render even when one refresh path is degraded.

### 5.6 Add explicit acceptance tests — DONE

- One backend call fails, the other succeeds, and good state remains visible.
- Error copy is limited to the affected panel(s).
- No unrelated panel gets an incorrect error or reset.

---

## 6. Address incomplete features, placeholders, and status drift

### 6.1 Clarify the provider failover story

- Decide whether provider failover is:
  - intentionally not implemented yet,
  - partially implemented but runtime-disabled, or
  - planned but not started
- Ensure the UI, docs, and runtime capability reporting all say the same thing.
- If appropriate, expose a real runtime capability flag instead of hard-coded summary text.

### 6.2 Implement or gate page snapshot placeholder metrics

Review `src-tauri/src/app_core.rs` page snapshot behavior.

Subtasks:

- Determine whether the browser backend can already provide:
  - `scroll_y`
  - `viewport_width`
  - `viewport_height`
  - `document_height`
- If yes, wire those values into `GetPageSnapshot`.
- If not, make the placeholder status explicit in contracts, docs, and any downstream usage.
- Add tests once real values are supported.

### 6.3 Update README and visible app copy to match the actual state of the project

- Update `README.md` current-status language so it no longer undersells the implementation.
- Update any “Phase 0 scaffold” UI copy if it no longer reflects reality.
- Keep the messaging honest about what is complete versus still placeholder.

### 6.4 Align docs with actual runtime capabilities

- Compare current docs in `docs/` against live code behavior.
- Correct any stale descriptions around:
  - browser control
  - settings panels
  - planner orchestration
  - ASR/TTS capabilities
  - provider failover
  - page snapshot output

---

## 7. Split oversized files into maintainable modules

### 7.1 Refactor `src/main.ts`

Create a cleaner module structure. Suggested extraction targets:

- `ui-store.ts` or `state-store.ts`
- `render-app.ts`
- `render-settings-panels.ts`
- `render-status-panels.ts`
- `audio-actions.ts`
- `tts-actions.ts`
- `asr-actions.ts`
- `runtime-refresh.ts`
- `event-handlers.ts`
- `dom-helpers.ts`

Subtasks:

- move state types and helpers out of the main entry file where appropriate
- move async persistence flows into focused modules
- isolate rendering from side effects
- keep the main entry point small and legible

### 7.2 Refactor `src/confirmation-panel.ts`

Possible extraction targets:

- formatting helpers
- error rendering helpers
- badge/metadata rendering helpers
- state-to-markup mapping helpers
- HTML helper utilities such as escaping and shared markup generation

### 7.3 Refactor `src-tauri/src/commands.rs`

Split by capability or command family. Suggested structure:

- browser commands
- page extraction commands
- narration/audio commands
- planner commands
- config/settings commands
- ASR commands
- TTS commands
- shared command utilities

### 7.4 Preserve test coverage while refactoring

- Keep public interfaces stable where practical.
- Move tests with the code they cover.
- Avoid a purely cosmetic split; improve boundaries and ownership at the same time.

---

## 8. Expand tests beyond render snapshots and basic happy paths

### 8.1 Frontend interaction tests

Add targeted tests for:

- event delegation after targeted DOM updates
- focus/cursor persistence during typing
- scoped control disablement during async saves
- recovery after failed save operations
- runtime partial-refresh behavior
- TTS provider rollback behavior

### 8.2 Tauri bridge contract tests

- Add tests around `src/tauri-api.ts` wrappers if missing.
- Verify expected request and response shapes for high-risk settings operations.
- Ensure error-shape handling is consistent.

### 8.3 Rust-side tests

- Add or extend tests for page snapshot metrics once implemented.
- Add tests for capability reporting if provider failover becomes runtime-query-driven.
- Add tests for settings rollback and error handling where practical.

### 8.4 CI improvements

- Ensure CI runs the correct frontend test command.
- Ensure CI runs TypeScript validation in an environment with required dependencies installed.
- Ensure Rust formatting, lint, and tests are part of the normal validation pipeline where intended.

---

## 9. Do a final UX and accessibility pass after the refactors

### 9.1 Keyboard and focus audit

- Verify tab order across all panels.
- Verify focus remains stable during async save transitions.
- Verify returning focus after modal-like confirmation interactions if applicable.

### 9.2 Screen-reader and status message audit

- Confirm busy states are announced appropriately.
- Confirm error messages are tied to relevant controls.
- Confirm status regions are not excessively re-announced after removing full rerenders.

### 9.3 Manual interaction audit

Manually test at least:

- typing into URL input
- typing into planner/TTS/ASR API key fields
- changing TTS provider, model, and voice
- changing ASR provider
- changing audio sliders while other settings are saving
- confirmation submission and retry flows
- runtime refresh after successful and failed actions

---

## 10. Final validation checklist

Complete this checklist before considering the review fixes done:

- all existing frontend tests pass
- all new regression tests pass
- TypeScript build passes in a correctly provisioned environment
- Rust format/lint/tests pass in a correctly provisioned environment
- no full-app `app.innerHTML` replacement is used for routine state updates
- unrelated controls are no longer blocked by global busy gates
- TTS provider rollback and error behavior is consistent
- runtime refresh supports partial success
- placeholder features are either implemented or explicitly gated/documented
- README and visible product copy are aligned with actual project status
- large files have been split into more maintainable modules where practical

---

## Suggested first PR sequence

### PR 1

- add regression tests for focus, busy-state, TTS rollback, and partial refresh
- no major architecture change yet

### PR 2

- refactor frontend rendering away from full-app rerendering
- preserve stable panel roots

### PR 3

- fix scoped busy-state logic
- fix TTS provider failure propagation

### PR 4

- refactor runtime refresh to support partial success
- update error-scoping behavior

### PR 5

- implement or explicitly gate placeholder features
- update README and UI status copy

### PR 6

- split oversized frontend and Rust files into focused modules
- do final cleanup and documentation alignment
