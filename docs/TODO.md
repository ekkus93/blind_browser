# TODO

## Scope

This document tracks the remaining frontend follow-up after the settings cleanup, React shell migration, and panel-root migration completed on 2026-04-18. It is intentionally focused on the current React and UI architecture work rather than the original scaffold-era project plan.

## Current Baseline

- [x] React, React DOM, Material UI, Emotion, Redux Toolkit, and React Redux are installed and in use.
- [x] The app shell renders through React instead of static template strings.
- [x] Workspace versus Settings navigation state is stored in Redux.
- [x] Planner, TTS, ASR, and Runtime settings now live on dedicated settings subpages.
- [x] Live panel roots render React nodes through dedicated React roots instead of replacing `innerHTML` on the runtime path.
- [x] The client bundle no longer pulls `react-dom/server` into the live browser graph.
- [x] Frontend validation is green with `pnpm lint`, `pnpm test:ui`, and `pnpm build`.
- [x] Repository validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` and `cargo test --manifest-path src-tauri/Cargo.toml --all-features`.

## Priority 1: Finish The React Ownership Model

- [x] Mount a single top-level React app from `src/main.ts` instead of treating each panel as an individually rerendered island.
- [x] Move panel state now held in module-level mutable variables in `src/main.ts` into Redux slices or React-owned state.
- [x] Replace the manual `rerender*Panel()` orchestration in `src/main.ts` with selector-driven React rendering.
- [x] Move settings-subpage routing and app-view transitions into the React tree so DOM class toggling becomes an implementation detail, not the main control flow.
- [x] Define a clear boundary between application state, async effects, and presentational components so the frontend no longer depends on imperative render sequencing.

## Priority 2: Remove Legacy HTML Rendering Seams

- [x] Remove the remaining string-render helpers in `src/settings-status-panels.ts`, especially `renderSettingsVolumePanel(...)` and `renderSettingsSpeedPanel(...)`.
- [x] Migrate tests that still depend on HTML-string renderers to assert against React-rendered DOM instead.
- [x] Retire `renderPanelRoot(panelRoots, rootKey, html)` from `src/app-shell.ts` after the DOM seam tests no longer require the `innerHTML` replacement path.
- [x] Delete the last `innerHTML`-based render seam once focus-preservation behavior is covered by React-based tests.
- [x] Keep all SSR-only helpers test-local or lazily loaded so browser bundles stay free of server-rendering code.

## Priority 3: Break Up The Frontend Into Smaller React Units

- [x] Split `src/settings-status-panels.ts` into focused components or modules by domain: playback, planner, TTS, ASR, runtime, and shared controls.
- [x] Split `src/confirmation-panel.ts` into focused components so confirmation UI, push-to-talk UI, status UI, and URL entry are easier to reason about independently.
- [x] Extract reusable React controls for masked API key entry, slider settings, select-card layouts, and status or error blocks.
- [x] Reduce duplicated field-label and card-layout logic by promoting shared component primitives instead of helper-generated markup.
- [x] Keep exported prop and state types explicit so panel contracts stay stable while files are decomposed.

## Priority 4: Move Interaction Handling Toward React

- [x] Replace broad DOM event delegation with React event handlers where practical, while preserving the voice-first interaction model.
- [x] Centralize busy, success, and error state transitions for remote settings actions such as save, test, load models, reset, and download.
- [x] Preserve focus and selection restoration for active inputs during rerenders without relying on HTML replacement.
- [x] Review keyboard, pointer, and push-to-talk interactions after the React ownership shift to ensure there are no regressions in accessibility or control flow.
- [x] Keep Tauri command boundaries explicit so React components trigger deterministic actions without embedding backend-specific logic in presentation code.

## Priority 5: Strengthen Frontend Test Coverage

- [x] Add component-level tests for the shell navigation flow between Workspace and Settings.
- [x] Add component-level tests for settings subpage navigation, including planner, TTS, ASR, and Runtime transitions.
- [x] Add focused tests for masked API key inputs, including focus, blur, replacement, restore, and latest-test-result states.
- [x] Add tests for focus preservation during React rerenders so the legacy DOM seam can be removed safely.
- [x] Add tests for Redux-driven view changes and panel-state updates without depending on serialized HTML snapshots.
- [x] Keep `pnpm lint`, `pnpm test:ui`, and `pnpm build` green after each migration slice.

## Priority 6: Documentation And Cleanup

- [x] Update `README.md` to describe the current React plus Redux shell architecture once the app is mounted as a single React tree.
- [x] Update `docs/SPECS.md` if the frontend ownership model changes in ways that matter to runtime boundaries, confirmation behavior, or settings flow.
- [x] Add a short frontend architecture note describing where state lives, how Tauri actions are invoked, and what remains intentionally imperative.
- [x] Record the final removal of the HTML rendering seam in `memory.md` and repo memory when that cleanup lands.
- [x] Delete obsolete compatibility helpers, dead exports, and test-only bridges as soon as their callers are gone.

## Suggested Execution Order

- [x] Introduce a top-level React app component and move shell plus subpage navigation under it.
- [x] Move panel state ownership out of `src/main.ts` into Redux or React state.
- [x] Convert remaining string renderers and retire the HTML seam.
- [x] Replace legacy DOM-seam tests with React-centric DOM tests.
- [x] Split the large frontend modules after the rendering model is stable.
- [x] Refresh docs once the architecture is settled.

## Validation Checklist

- [x] `source ./fix-node-version.sh && pnpm lint`
- [x] `source ./fix-node-version.sh && pnpm test:ui`
- [x] `source ./fix-node-version.sh && pnpm build`
- [x] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- [x] `cargo test --manifest-path src-tauri/Cargo.toml --all-features`
- [x] LLM provider selection behavior
- [x] Default local model profile selection behavior
- [x] TTS provider selection behavior
- [x] ASR provider selection behavior
- [x] Deterministic tool result schemas
- [x] Common tool envelope serialization/deserialization
- [x] Planner input/output schema serialization/deserialization
- [x] Per-tool input schema validation
- [x] Enum serialization/deserialization and validation
- [x] Provider config serialization/deserialization and validation
- [x] Secret reference resolution and masking behavior
- [x] Audio settings persistence and validation
- [x] Browser history state serialization and boundary behavior
- [x] Runtime status schema serialization and provider-mode reporting
- [x] Deterministic listening state transitions and one-shot transcription tool behavior
- [x] Deterministic browser visibility and audio-setting tool clamping behavior
- [x] Voice command parsing for playback volume and playback speed
- [x] Volume normalization from percent, decimal, and relative phrases
- [x] Playback speed normalization from multiplier, percent, and relative phrases
- [x] Volume and playback speed query command normalization and spoken response formatting
- [x] SKILL.md frontmatter validation and precedence resolution
- [x] Skill ranking and top-N selection behavior
- [x] Reject unknown tools and invalid planner transitions
- [x] Reject invalid tool arguments before execution
- [x] Element matching and resolution behavior
- [x] Pending plan execution state serialization and resume bookkeeping
- [x] ExecutionOutcome mapping from PlannerStatus and step transitions
- [x] Page model building
- [x] Navigation logic

### Integration Tests
- [x] Load page → extract → read
- [x] ASR → command → action
- [x] Planner output → deterministic tool execution
- [x] Back/forward/reload tools update browser history state correctly
- [x] Browser visibility changes are reflected in runtime status and UI state
- [x] Listening start/stop/transcribe tools update runtime state correctly
- [x] Deterministic audio-setting tools persist and report the updated values
- [x] Planner requests confirmation before risky execution
- [x] Queued confirmation flows resume at the stored follow-up step after explicit user approval
- [x] Rejected or timed-out confirmation flows clear pending state and replan without executing the queued side-effecting step
- [x] Submit actions always require confirmation
- [x] Click actions may proceed without confirmation when configured
- [x] Fill-field workflows resolve the intended input and write the requested value
- [x] Fill-and-submit workflows require confirmation before form submission
- [x] Ambiguous element matches ask the user to clarify instead of silently choosing one
- [x] Mixed commands such as fill-and-submit are decomposed into safe bounded plans
- [x] Follow-up corrections such as `no, the other field` reuse recent context when available
- [x] Replanning after tool failure or ambiguous result
- [x] LLM unavailable with no local provider → report command interpretation unavailable
- [x] Remote TTS selected → speech output succeeds
- [x] Remote ASR selected → transcript is returned
- [x] Playback volume and speed changes persist across app restart
- [x] Voice command changes to playback volume and speed persist across app restart
- [x] Changed speech settings apply on the next utterance only

### Agentic Tests
- [x] Add planner-skill regression fixtures with browser state, transcript, expected selected skills, and expected tool sequence
- [x] Assert that the correct bundled skills were selected for representative tasks
- [x] Add fixtures for ambiguous clicks, form filling, fill-and-submit, and follow-up corrections
- [x] Build a growing corpus of in-the-wild problematic pages for agentic regression coverage

---

## Phase 9: v2 Notes (DO NOT IMPLEMENT)

### Wake Word
- [ ] Evaluate TensorFlow Lite micro_speech

### LLM Action Resolver
- [ ] Candidate extraction system
- [ ] LLM ranking
- [ ] Confidence gating
- [ ] Evaluate broader open-ended UI grounding beyond the deterministic v1 tool layer

### Deferred Exploration
- [ ] Evaluate whether advanced UI action grounding should remain v2-only

---

## Deliverables

- [ ] Working desktop app
- [ ] README.md
- [ ] SPECS.md
- [ ] TODO.md
- [ ] Example configs
