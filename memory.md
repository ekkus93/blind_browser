## 2026-03-23T08:51:47Z - GPT-5.4 - Initial project memory
- Project: blind_browser, a Rust + Tauri voice-first web browser for vision-impaired users.
- Core architecture: bounded LLM planner over deterministic Rust tools; Pi-style skills guide planning but do not replace tool execution.
- Browser/runtime decisions: chromiumoxide backend, standard Tauri conventions, visible or headless browser modes.
- Media/provider defaults: remote OpenAI planner with optional local Qwen failover, local KittenTTS for TTS, local Whisper for ASR.
- Safety and UX: voice-first operation, short spoken responses by default, submit actions always require confirmation, ambiguous targets should prompt the user to clarify.
- Documentation state: `.github/copilot-instructions.md`, `docs/SPECS.md`, `docs/SKILLS.md`, and `docs/TODO.md` were created and committed as the v1 implementation baseline.

## 2026-03-23T09:09:25Z - GPT-5.4 - Phase 0 scaffold baseline
- Added an in-place Tauri + Vite scaffold under `src-tauri/` and `src/` with standard Tauri layout.
- Added initial Rust module boundaries for app_core, browser, extractor, dom_inspector, page_model, narration, tts, asr, audio_io, commands, ocr, config, state, and logging.
- Added baseline schema types for deterministic tools, planner input/output, provider config, runtime state, and bundled integration feature flags in `src-tauri/Cargo.toml`.
- Added `config.example.toml` and expanded `README.md` with setup and Linux prerequisite notes.
- Validation result: `pnpm build` passes and editor diagnostics are clean on the main Rust and TypeScript files; `cargo check` is currently blocked by missing Linux GTK/WebKit development packages required by Tauri.

## 2026-03-23T09:14:42Z - GPT-5.4 - Linux validation unblocked
- After installing the Linux GTK/WebKit prerequisites, `cargo check` now passes for the Phase 0 Tauri scaffold.
- Remaining scaffold fixes were project-local: removed invalid `Eq` derives from `ElementSearchResult` and `AppConfig`, and added a valid RGBA `src-tauri/icons/icon.png` required by `tauri::generate_context!()`.
- README prerequisite guidance was already sufficient; no further documentation change was needed for the apt packages.

## 2026-03-23T09:16:01Z - GPT-5.4 - Phase 0 TODO alignment
- Updated `docs/TODO.md` to mark the completed setup scaffolding and config-schema defaults as done.
- Left unchecked the items that still require real implementation work, especially SKILL loading, per-tool argument validation, TOML loading/validation, persistent audio settings, local LLM integration, and actual KittenTTS integration.

## 2026-03-23T09:25:54Z - GPT-5.4 - TOML config loading implemented
- `src-tauri/src/config.rs` now loads config from TOML, validates provider/profile references and key numeric ranges, and falls back explicitly to the embedded shipped template when no app config file exists yet.
- `AppCore::new` now initializes runtime config through the validated loader instead of hardcoded `AppConfig::default()`.
- Added focused tests covering shipped-template parsing, missing required remote profiles, and invalid playback-speed validation.
- Updated `config.example.toml` so the `qwen2.5-3b-q4` table key is quoted correctly for TOML parsing.

## 2026-03-23T09:30:48Z - GPT-5.4 - Audio settings persistence implemented
- `src-tauri/src/config.rs` now persists updated audio settings back to `config.toml`, creating the app config directory when needed and preserving the rest of the TOML document.
- `src-tauri/src/app_core.rs` now exposes persisted audio-update helpers for playback volume, playback speed, and default TTS voice.
- `src-tauri/src/state.rs` and `src-tauri/src/audio_io.rs` now hydrate runtime audio state from the loaded config so startup reflects persisted values.
- Added a persistence test covering write-and-reload for audio settings; `cargo test config::tests` and `cargo check` both pass.

## 2026-03-23T09:33:57Z - GPT-5.4 - Deterministic audio-setting tools wired
- `src-tauri/src/commands.rs` now defines `SetTtsVoiceInput/Data`, `SetPlaybackVolumeInput/Data`, and `SetPlaybackSpeedInput/Data`, plus reusable `ToolResult::success` and `ToolResult::failure` helpers.
- `src-tauri/src/app_core.rs` now exposes deterministic tool execution methods for `set_tts_voice`, `set_playback_volume`, and `set_playback_speed` that return structured tool envelopes.
- Playback volume and speed tool execution clamp to the configured supported ranges before persisting, and tool observations report when clamping occurred.
- `docs/TODO.md` now marks the three Wave 1 audio-setting tools and their exposure hooks as implemented.

## 2026-03-23T09:42:17Z - GPT-5.4 - Planner/executor dispatch added
- `src-tauri/src/commands.rs` now contains a `DeterministicToolExecutor` trait and `execute_planned_step` dispatcher that validates `PlannedStep.arguments` and routes supported tool calls by `tool_name`.
- Planner/executor dispatch is currently wired for `SetTtsVoice`, `SetPlaybackVolume`, and `SetPlaybackSpeed`, returning serialized `ToolResult<serde_json::Value>` envelopes suitable for a generic executor path.
- Unsupported tools now fail with a stable `unsupported_tool` error, and malformed tool arguments fail with `invalid_tool_arguments` before handler execution.
- Added focused commands-layer tests covering successful dispatch and argument-validation failure; `cargo test commands::tests` and `cargo check` both pass.

## 2026-03-23T09:46:52Z - GPT-5.4 - Runtime-state tool dispatch extended
- `execute_planned_step` now also routes `SetBrowserVisibility`, `GetAgentState`, and `GetRuntimeStatus` through the same deterministic executor path.
- `src-tauri/src/app_core.rs` now returns spec-aligned runtime payloads for browser visibility, agent state, and runtime status using current in-memory state plus configured provider modes.
- `src-tauri/src/commands.rs` now defines the missing tool input/output shapes for those three tools and adds focused dispatcher tests for successful invocation.
- `docs/TODO.md` now marks `set_browser_visibility`, `get_agent_state`, and `get_runtime_status` as implemented in Wave 1.

## 2026-03-23T09:54:15Z - GPT-5.4 - Planner executor loop implemented
- `src-tauri/src/commands.rs` now includes `execute_planner_output`, which walks `PlannerOutput.steps`, executes each step, and follows `on_success` or `on_failure` transitions until `Complete`, `NeedsReplan`, or `AwaitingConfirmation`.
- The executor loop validates duplicate or missing step ids, blocks side-effecting steps in `NeedsConfirmation` plans until a real confirmation step runs, and captures queued step ids plus `confirmation_id` into `PendingPlanExecutionState` when a transition requests confirmation.
- Added commands-layer tests covering multi-step success, failure-to-replan, confirmation wait, invalid next-step transitions, and confirmation-gated side-effect protection; `cargo test commands::tests` and `cargo check` pass.

## 2026-03-23T19:18:14Z - GPT-5.4 - Pending execution state persisted in runtime
- `src-tauri/src/state.rs` now stores `pending_confirmation_id` and `pending_plan_execution` directly in `AppState`, with helpers to persist `AwaitingConfirmation` outcomes and clear the state for terminal outcomes.
- `src-tauri/src/app_core.rs` now exposes an `execute_planner_output` wrapper that runs the commands-layer executor loop and immediately applies the resulting pending execution state to the shared runtime state managed by Tauri.
- `get_agent_state` and `get_runtime_status` now report the persisted pending confirmation fields from live runtime state so confirmation waits survive later command-boundary reads.
- Added focused state tests for storing and clearing pending execution state; `cargo test state::tests`, `cargo test commands::tests`, and `cargo check` pass.

## 2026-03-23T19:23:08Z - GPT-5.4 - Confirmation resume path implemented
- `src-tauri/src/commands.rs` now persists queued `PlannedStep` values inside `PendingPlanExecutionState` and exposes `resume_after_confirmation`, which validates the confirmation id, returns `NeedsReplan` on rejection, and resumes deterministic execution from the stored next step when confirmed.
- The step runner was refactored so both initial execution and resumed execution share the same transition handling, including repeat-cycle detection, missing-step validation, and re-entry into `AwaitingConfirmation` if a resumed step chain asks for another confirmation.
- `src-tauri/src/app_core.rs` now exposes a runtime-aware `resume_after_confirmation` wrapper that reads the persisted pending state, preserves it on mismatched confirmation ids, and applies the resumed outcome back into `AppState` when the confirmation matches.
- Updated `docs/SPECS.md` and `docs/TODO.md` to reflect that pending execution state now includes queued steps for deterministic resume bookkeeping; `cargo test state::tests`, `cargo test commands::tests`, and `cargo check` pass.

## 2026-03-23T19:33:21Z - GPT-5.4 - Confirm action dispatch and response handling wired
- `src-tauri/src/commands.rs` now defines `ConfirmActionInput`, dispatches `ToolName::ConfirmAction` through the deterministic executor trait, and persists the confirmation prompt text alongside the pending confirmation id and queued steps.
- `src-tauri/src/app_core.rs` now implements the real `execute_confirm_action` tool for issuing confirmation prompts and adds `submit_confirmation_response`, which converts the collected user answer into a `ConfirmActionData` result and calls `resume_after_confirmation` immediately to continue or replan.
- The runtime confirmation response path now treats `timed_out = true` the same as a rejection for resume purposes while still returning explicit confirmation tool data to the caller.
- Added commands-layer coverage for `confirm_action` step dispatch and updated docs/TODO to mark `confirm_action` implemented; `cargo test state::tests`, `cargo test commands::tests`, and `cargo check` pass.

## 2026-03-23T19:36:22Z - GPT-5.4 - Tauri confirmation command exposed
- `src-tauri/src/lib.rs` now registers a `submit_confirmation_response` Tauri command and manages `AppCore` behind `Mutex<AppCore>` so frontend invocations can safely mutate runtime confirmation state.
- The command returns `ConfirmActionResolution` on success and a structured `ToolError` if the runtime state lock cannot be acquired.
- `docs/TODO.md` now marks the implemented post-confirmation resume and rejection/timeout replan behaviors complete because the frontend command surface exists to trigger them.
- Validation after the Tauri command wiring: `cargo check`, `cargo test commands::tests`, and `cargo test state::tests` all pass.

## 2026-03-23T19:39:11Z - GPT-5.4 - Tauri planner execution command exposed
- `src-tauri/src/lib.rs` now registers an `execute_planner_output` Tauri command so the frontend can submit a structured `PlannerOutput` and receive the resulting `ExecutionOutcome` directly from the shared runtime.
- The Tauri command surface now covers both sides of the flow: planner execution starts the plan and may enter `AwaitingConfirmation`, and `submit_confirmation_response` completes the confirmation branch and resumes or replans.
- `AppCore` locking for Tauri commands is centralized through a small helper that returns a structured `ToolError` on lock failure.
- `docs/TODO.md` now marks planner-output deterministic tool execution complete; validation after the new command: `cargo check`, `cargo test commands::tests`, and `cargo test state::tests` all pass.

## 2026-03-23T19:44:04Z - GPT-5.4 - Frontend typed invoke wrappers added
- Added `src/tauri-api.ts` with typed frontend wrappers for `execute_planner_output` and `submit_confirmation_response`, plus TypeScript representations of the relevant Rust command payloads and outcomes.
- `src/main.ts` now re-exports the frontend wrapper functions and their core command/result types so the thin UI layer can consume the Tauri API without reaching into backend-specific details.
- Frontend validation: `pnpm build` passes after adjusting the confirmation invoke arguments to an explicit object literal compatible with `@tauri-apps/api/core` typing.

## 2026-03-23T19:47:58Z - GPT-5.4 - Frontend confirmation orchestration helper added
- Added `src/planner-orchestration.ts`, a small frontend helper that detects `ExecutionOutcome.AwaitingConfirmation`, derives an explicit confirmation UI state, and wraps both planner execution and confirmation response calls so the UI can drive the plan → confirm → resume loop from one place.
- `src/main.ts` now re-exports the orchestration helpers and notes in the scaffold copy that the frontend can open a dedicated confirmation UI state whenever the runtime enters `AwaitingConfirmation`.
- Frontend validation after the helper addition: `pnpm build` passes under strict TypeScript settings.

## 2026-03-23T19:51:07Z - GPT-5.4 - Frontend confirmation panel component added
- Added `src/confirmation-panel.ts`, a small presentational helper that renders a confirmation panel only when `ConfirmationUiState.kind === "awaiting-confirmation"`, showing the prompt, confirmation id, request id, next step, selected skills, and queued step ids.
- `src/main.ts` now exports the confirmation panel renderer and includes the rendered panel area in the scaffold layout so the UI has a dedicated surface for confirmation state.
- `src/styles.css` now includes focused confirmation panel styles that match the existing scaffold while distinguishing approval-required states.
- Frontend validation after the panel addition: `pnpm build` passes after switching the HTML escaping helper to ES2020-compatible `replace` calls.

## 2026-03-23T19:54:07Z - GPT-5.4 - Live frontend execution UI store wired
- `src/planner-orchestration.ts` now exposes `createExecutionUiStore`, a tiny subscribable state holder that keeps the current execution UI state live and can apply new `ExecutionOutcome` values from planner execution or confirmation resolution.
- `runPlannerExecution` and `resolveConfirmationResponse` can now optionally update that shared frontend store directly instead of only returning detached snapshots.
- `src/main.ts` now renders from the live orchestration store via a subscribe/render loop instead of a one-time `createInitialExecutionUiState()` snapshot, so future planner outcomes can update the confirmation panel in place.
- Frontend validation after the live-state wiring: `pnpm build` passes.

## 2026-03-23T19:57:13Z - GPT-5.4 - Frontend confirmation actions wired
- `src/confirmation-panel.ts` now renders explicit approve and reject buttons for `awaiting-confirmation` state, carrying the active confirmation id in data attributes.
- `src/main.ts` now delegates confirmation button clicks from the app root and calls `resolveConfirmationResponse(..., uiStore)` so confirmation decisions flow through the existing Tauri resume path.
- `src/styles.css` now styles the confirmation action controls with distinct approve and reject treatments that match the existing panel design.
- Frontend validation after the action wiring: `pnpm build` passes.

## 2026-03-23T19:59:31Z - GPT-5.4 - Frontend confirmation submitting state added
- `src/planner-orchestration.ts` now carries an `isSubmitting` flag in awaiting-confirmation UI state and exposes a store helper to toggle that flag by confirmation id.
- `src/main.ts` now sets the confirmation state to submitting before calling `resolveConfirmationResponse`, blocks repeat clicks during the request, and restores the buttons if the request fails.
- `src/confirmation-panel.ts` and `src/styles.css` now show a temporary submitting status and disable both confirmation buttons while the response is in flight.
- Frontend validation after the submitting-state change: `pnpm build` passes.

## 2026-03-23T20:00:53Z - GPT-5.4 - Frontend confirmation submission errors surfaced
- `src/planner-orchestration.ts` now stores a transient `submissionError` on awaiting-confirmation UI state and exposes a store helper to set or clear that error by confirmation id.
- `src/main.ts` now converts failed confirmation submissions into a user-facing error message instead of only re-enabling the buttons.
- `src/confirmation-panel.ts` and `src/styles.css` now render the failure message inline with alert semantics so submission problems stay visible until the next attempt or a new outcome arrives.
- Frontend validation after the error-state change: `pnpm build` passes.

## 2026-03-23T20:02:45Z - GPT-5.4 - Confirmation failures classified by source
- `src/tauri-api.ts` now classifies invoke failures as either backend `ToolError` payloads or transport/runtime failures so the UI can react to each category differently.
- `src/planner-orchestration.ts` now maps those failure categories into structured confirmation-panel messages with source-specific guidance and retryability metadata for backend errors.
- `src/confirmation-panel.ts` now renders richer failure detail, including backend error codes and retryability, while transport failures instruct the user to check the desktop runtime connection.
- Frontend validation after failure classification: `pnpm build` passes.

## 2026-03-23T20:05:37Z - GPT-5.4 - Non-retryable backend failures styled as hard stops
- `src/confirmation-panel.ts` now emits distinct CSS classes for transport failures, retryable backend tool errors, and non-retryable backend tool errors.
- `src/styles.css` now gives non-retryable backend failures a heavier hard-stop treatment with a darker border, stronger background, and more emphatic title/guidance styling so they read as more severe than transport issues.
- Retryable backend failures and transport failures keep lighter treatments to preserve the visual distinction between recoverable connection problems and hard-stop runtime rejections.
- Frontend validation after the hard-stop styling change: `pnpm build` passes.

## 2026-03-23T20:07:53Z - GPT-5.4 - Non-retryable backend errors now carry a badge
- `src/confirmation-panel.ts` now renders a dedicated `Requires planner change` badge only for non-retryable backend tool errors.
- `src/styles.css` now styles that badge as part of the hard-stop treatment so it reads as an explicit non-retryable indicator instead of generic error copy.
- Transport failures and retryable backend failures do not render the badge, preserving the severity distinction across failure types.
- Frontend validation after the badge addition: `pnpm build` passes.

## 2026-03-23T20:13:54Z - GPT-5.4 - Backend errors now state retry status explicitly
- `src/confirmation-panel.ts` now renders a second short retry-status line in backend error metadata: `Can retry.` for retryable backend failures and `Cannot retry.` for non-retryable backend failures.
- `src/styles.css` now styles that retry-status line so it remains readable in both retryable and hard-stop backend error variants.
- The confirmation panel no longer relies only on badge and color to communicate whether a backend error can be retried.
- Frontend validation after the retry-status copy change: `pnpm build` passes.

## 2026-03-23T20:16:47Z - GPT-5.4 - Added render test for non-retryable retry copy
- Added `src/confirmation-panel.test.mjs`, a Node built-in render test that verifies `Cannot retry.` appears for non-retryable backend tool errors and does not appear for retryable backend or transport failures.
- Added a dependency-free `pnpm test:ui` script in `package.json` using Node's built-in test runner with strip-types support.
- Validation after the test addition: `pnpm test:ui` passes and `pnpm build` still passes.

## 2026-03-23T20:24:11Z - GPT-5.4 - Added render-test coverage for hard-stop badge
- `src/confirmation-panel.test.mjs` now also verifies that the `Requires planner change` badge appears only for non-retryable backend tool errors and does not appear for retryable backend or transport failures.
- The badge and `Cannot retry.` copy are now covered by the same three render fixtures so those signals stay aligned.
- Validation after the test update: `pnpm test:ui` passes.

## 2026-03-23T20:26:44Z - GPT-5.4 - Retry-copy render coverage is now symmetric
- `src/confirmation-panel.test.mjs` now also verifies that `Can retry.` appears only for retryable backend tool errors and does not appear for non-retryable backend or transport failures.

## 2026-03-24T18:12:24Z - GPT-5.4 - Local planner and failover implemented
- `src-tauri/src/app_core.rs` now supports local planner resolution through `llama-cpp-2`, using the configured local planner profile, model chat template, bounded token generation, and explicit extraction of the first complete JSON object before `PlannerOutput` deserialization.
- Remote planner resolution now falls back to the configured local planner only when `providers.planner.failover_to_local = true`, and logs that failover path explicitly instead of silently changing providers.
- `src-tauri/Cargo.toml` now enables `llama-cpp-2` with shared-library linking to avoid static `ggml` collisions with `whisper-rs`; validation passed with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`, with a current upstream workaround of clearing copied `libllama.so`/`libggml*.so` symlinks between Cargo phases because `llama-cpp-sys-2` panics if those copied shared-library symlinks already exist in `target/debug`, `target/debug/deps`, or `target/debug/examples`.
- The confirmation panel test now covers both backend retry states symmetrically across the same three render fixtures.
- Validation after the symmetric retry-copy assertions: `pnpm test:ui` passes.

## 2026-03-23T20:31:17Z - GPT-5.4 - Confirmation panel render tests split by behavior
- `src/confirmation-panel.test.mjs` now uses shared fixtures plus two focused tests: one for retry-copy behavior and one for the `Requires planner change` badge.
- The split improves failure output without changing the existing coverage for retryable, non-retryable, and transport error variants.
- Validation after the test split: `pnpm test:ui` passes.

## 2026-03-23T20:33:26Z - GPT-5.4 - Added focused metadata-block render coverage
- `src/confirmation-panel.test.mjs` now includes a third focused test that verifies the backend metadata block structure and exact retry-status lines for retryable and non-retryable backend errors.
- The test intentionally ignores incidental whitespace while still pinning the `confirmation-error-meta-block`, backend error-code line, and retry-status line content.
- Validation after the metadata-block test addition: `pnpm test:ui` passes.

## 2026-03-23T20:40:35Z - GPT-5.4 - Lint and test validation status
- `cargo fmt --manifest-path src-tauri/Cargo.toml --all --check` completed successfully before the lint pass reached Clippy.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` is currently blocked by a missing native OCR dependency: `leptonica-sys` cannot find `lept.pc` via `pkg-config`.
- `cargo test --manifest-path src-tauri/Cargo.toml` passes with 20 Rust tests green.
- `pnpm test:ui` passes with 3 confirmation-panel render tests green.

## 2026-03-23T20:43:24Z - GPT-5.4 - OCR system prerequisites documented
- `README.md` now documents the Linux OCR prerequisites separately from the base Tauri/Linux runtime packages.
- The README now explicitly lists `libleptonica-dev`, `libtesseract-dev`, and `tesseract-ocr` and notes that `cargo clippy --all-features` may fail until they are installed.
- The README also calls out `lept.pc` lookup failures as a likely sign that `libleptonica-dev` is missing.

## 2026-03-23T21:23:18Z - GPT-5.4 - ALSA prerequisite documented after Clippy run
- After the OCR dependency blocker was resolved, `cargo clippy --all-features` advanced to the audio dependency chain and failed in `alsa-sys` because `pkg-config` could not find `alsa.pc`.
- `README.md` now includes `libasound2-dev` in the Linux prerequisite install command and notes that a missing `alsa.pc` usually means that package is not installed.

## 2026-03-23T21:39:20Z - GPT-5.4 - Clippy issues fixed and full validation green
- Fixed `clippy::too_many_arguments` in `src-tauri/src/commands.rs` by grouping step-runner inputs into a small `StepExecutionContext` struct without changing execution behavior.
- Fixed `clippy::derivable_impls` in `src-tauri/src/page_model.rs` by deriving `Default` for `PageModel`.
- Fixed `clippy::field_reassign_with_default` in `src-tauri/src/state.rs` test code by constructing `AppState` with a struct update instead of reassigning fields after `Default::default()`.
- Validation after the fixes: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml`, and `pnpm test:ui` all pass.

## 2026-03-23T21:40:24Z - GPT-5.4 - TODO updated with Linux validation milestone
- `docs/TODO.md` now records that the Linux development baseline is validated when the documented native dependencies are installed.
- The tracked baseline explicitly includes `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `pnpm test:ui`.

## 2026-03-23T21:44:15Z - GPT-5.4 - Fresh lint and test pass completed
- A follow-up `cargo fmt --check` found one formatting drift in `src-tauri/src/commands.rs`; running `cargo fmt` resolved it without behavioral changes.
- Validation after the format fix: `cargo fmt --manifest-path src-tauri/Cargo.toml --all --check`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml`, and `pnpm test:ui` all pass.

## 2026-03-23T21:45:36Z - GPT-5.4 - Final frontend production build passes
- Ran `pnpm build` as a final frontend sanity check after the clean lint and test run.
- The Vite production build completed successfully, producing the current `dist/` assets without errors.

## 2026-03-23T21:53:07Z - GPT-5.4 - Implementation baseline committed to master
- The current Rust/Tauri scaffold, planner execution flow, frontend confirmation UI, Linux prerequisite docs, and validation updates are being committed as one baseline changeset.
- The intended git action for this milestone is a push to `origin/master` after staging the working tree.

## 2026-03-23T22:07:03Z - GPT-5.4 - report_result wired through runtime and executor
- Added `ReportResultInput`, deterministic executor trait support, planner-step dispatch, and `AppCore::execute_report_result` so plans can end with a structured reporting tool.
- `report_result` now rejects empty summaries, trims optional text fields, and has focused Rust tests covering dispatcher wiring and normalization behavior.

## 2026-03-23T22:28:03Z - GPT-5.4 - open_url wired through runtime state and executor
- Added `LoadState`, `OpenUrlInput`, and `OpenUrlData`, plus planner-step dispatch and deterministic executor support for `open_url`.
- `open_url` now validates absolute URLs, records a new `page_id`, updates runtime page/history state, and returns structured navigation metadata while leaving title and HTTP status unavailable until Chromium backend integration lands.

## 2026-03-23T22:43:01Z - GPT-5.4 - get_page_snapshot wired through runtime state and executor
- Added `GetPageSnapshotInput` and expanded `PageSnapshotData` to the spec-aligned snapshot shape, then wired deterministic dispatch and runtime handling for `get_page_snapshot`.
- The snapshot tool now reads the active page from runtime state, returns excerpted visible text plus optional interactive elements, and uses placeholder scroll/viewport metrics until the browser backend is integrated.

## 2026-03-24T04:04:39Z - GPT-5.4 - extract_page_model wired through runtime state and executor
- Added `ExtractPageModelInput`, `ExtractPageModelData`, and `ExtractionSource`, then wired deterministic dispatch and runtime handling for `extract_page_model`.
- The extraction tool now clones the current runtime page model, can omit link elements on request, counts readable regions, and infers a conservative extraction source from existing region metadata while documenting current heading/OCR limitations.

## 2026-03-24T04:22:39Z - GPT-5.4 - list_interactive_elements wired through runtime state and executor
- Added `ListInteractiveElementsInput`, `ListInteractiveElementsData`, deterministic executor trait support, and planner-step dispatch for `list_interactive_elements`.
- The tool now lists interactive elements from the current runtime page model, supports `visible_only` and optional role filtering, and has focused Rust tests covering dispatcher wiring and filter behavior.

## 2026-03-24T04:39:37Z - GPT-5.4 - find_element wired through runtime state and executor
- Added `FindElementInput`, `FindElementData`, deterministic executor support, and planner-step dispatch for `find_element`.
- `find_element` now scores candidates from the same filtered runtime interactive-element data used by `list_interactive_elements`, returns a strongest match when it is clear, and marks close top candidates as requiring planner clarification before side effects.

## 2026-03-24T06:05:36Z - GPT-5.4 - click_element now uses the live Chromium backend
- Added a lazy Chromium session controller in `src-tauri/src/browser.rs` and a direct `futures` dependency so the browser handler can be polled in the background.
- `open_url` now drives the live Chromium page and updates runtime URL/title state from the browser instead of runtime-only placeholders.
- `click_element` now resolves the chosen deterministic `element_id` to a live DOM node by scoring page elements against `InteractiveElement` metadata, then triggers a real Chromium click or double-click and updates runtime navigation state from the resulting page.
- Validation: `cargo test --manifest-path src-tauri/Cargo.toml --features browser` and `cargo clippy --manifest-path src-tauri/Cargo.toml --features browser --all-targets -- -D warnings` both pass.

## 2026-03-24T06:15:04Z - GPT-5.4 - page model now carries stable DOM locators
- Added `dom_locator: Option<String>` to `InteractiveElement` and updated the shared spec so DOM-backed actions can rely on a persisted page-model locator instead of re-deriving selectors at execution time.
- `src-tauri/src/browser.rs` now requires that stored locator for `click_element` and fails clearly when the page model lacks one or when the locator no longer matches a live DOM node.
- Added a focused regression test for missing `dom_locator`, and validation remains green for `cargo test --manifest-path src-tauri/Cargo.toml --features browser` and `cargo clippy --manifest-path src-tauri/Cargo.toml --features browser --all-targets -- -D warnings`.

## 2026-03-24T06:20:44Z - GPT-5.4 - stable DOM locator contract enforced in live source
- Added `dom_locator: Option<String>` to `InteractiveElement` in the live Rust source and updated deterministic fixtures so browser-backed actions have a persisted locator to consume.
- `click_element` now rejects missing locators in `AppCore`, uses the stored locator directly in the Chromium backend, and no longer depends on browser-side heuristic DOM matching.
- Added a regression test for missing `dom_locator`; validation passes with `cargo test --manifest-path src-tauri/Cargo.toml --features browser` and `cargo clippy --manifest-path src-tauri/Cargo.toml --features browser --all-targets -- -D warnings`.

## 2026-03-24T06:34:40Z - GPT-5.4 - extract_page_model now populates actionable locators from Chromium
- Added a live DOM extraction path in `src-tauri/src/browser.rs` that walks the current Chromium page, captures visible text regions plus interactive elements, and emits stable `dom_locator` values alongside extracted element metadata.
- `AppCore::execute_extract_page_model` now refreshes `state.current_page` from that live browser extraction when DOM extraction is requested, then applies request-specific filtering only to the returned payload so later `click_element` calls remain actionable.
- Validation passes with `cargo test --manifest-path src-tauri/Cargo.toml --features browser` and `cargo clippy --manifest-path src-tauri/Cargo.toml --features browser --all-targets -- -D warnings`.

## 2026-03-24T07:41:04Z - GPT-5.4 - Handoff for moving development to another machine
- Current shared checkpoint: commit `0d949fb790353df9483decfbc0c2d109b3cd7f57` on `origin/master` with subject `Wire live browser extraction and DOM actions`.
- Where work stopped: deterministic browser tooling now covers `open_url`, `get_page_snapshot`, `extract_page_model`, `list_interactive_elements`, `find_element`, `click_element`, and `report_result`; `click_element` uses stored `dom_locator` values and `extract_page_model` now repopulates those locators from the live Chromium DOM.
- Runtime/browser state: `AppCore` owns a live `BrowserController`; navigation updates `current_page_id` and browser history, and DOM extraction persists the full fresh `PageModel` into runtime state before request-specific filtering is applied to the returned payload.
- Validation state at handoff: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` passes, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` passes with 39 tests green, `pnpm test:ui` passed earlier with 3 tests green, and `pnpm build` was already green before this browser-extraction milestone.
- Environment bring-up pointer: use `README.md` as the source of truth, especially `Local Development`, `Validation`, `Linux Tauri Prerequisites`, and `Linux OCR Prerequisites`.
- Fast setup order on a new Linux machine: install Rust stable via `rustup`, install Node.js 20+ and enable `pnpm` with `corepack enable`, install the apt packages listed in `README.md`, run `pnpm install`, then run the four validation commands from the README to confirm the machine is ready.
- Native dependency failure hints: missing `alsa.pc` usually means `libasound2-dev` is missing, and missing `lept.pc` usually means `libleptonica-dev` is missing; full-feature Rust builds also need the GTK/WebKit packages listed in the README.
- Next likely work from this checkpoint: continue Wave 1 browser tools after `click_element`, wire real scroll/read-region behaviors against the live browser state, and keep the voice-first confirmation/safety path aligned with the deterministic tool contracts.

## 2026-03-24T09:05:32Z - GPT-5.4 - Project docs re-read for onboarding
- Re-read `README.md`, `memory.md`, `docs/SPECS.md`, `docs/SKILLS.md`, and `docs/TODO.md` to refresh the current architecture, shipped defaults, planner/tool contracts, and active implementation frontier.
- Core product shape remains a voice-first Rust + Tauri desktop browser for vision-impaired users with a bounded LLM planner over deterministic Rust tools, Chromium via `chromiumoxide`, DOM-first extraction, and OCR fallback.
- Current implementation checkpoint is beyond initial scaffold work: live browser-backed `open_url`, `get_page_snapshot`, `extract_page_model`, `list_interactive_elements`, `find_element`, `click_element`, confirmation resume flow, and frontend confirmation UI are in place; major remaining Wave 1 gaps include `go_back`, `go_forward`, `reload_page`, `scroll_page`, narration tools, and listening/transcription tools.
- Spec-critical rules to preserve in future work: stable `dom_locator` values for DOM-backed actions, submit actions always require confirmation, clicks may skip confirmation by default, ambiguous element matches must route through clarification, and planner output must stay bounded to registered deterministic tools plus validated schemas.

## 2026-03-24T09:09:26Z - GPT-5.4 - Environment readiness check found setup blockers
- Ran an environment readiness check on Ubuntu 24.04.4 by probing the Rust/Node toolchain and native libraries required by the README validation flow.
- Current state: `node` is present via `nvm` at `v22.11.0`, but `rustc`, `cargo`, and `rustup` are not installed; `pkg-config` cannot find `webkit2gtk-4.1`, `alsa`, `lept`, or `tesseract`, so the documented Tauri, audio, and OCR native prerequisites are missing.
- `corepack enable` did not complete cleanly in this environment because it hit a signature/key mismatch while trying to resolve `pnpm`, so even the JavaScript package manager is not yet ready from the current shell state.
- Result: the machine is not ready to run `pnpm install`, `cargo clippy`, `cargo test`, `pnpm test:ui`, or `pnpm build` until Rust, pnpm, and the Linux native dependencies are installed per `README.md`.

## 2026-03-24T09:13:26Z - GPT-5.4 - Validation attempt partially blocked by environment
- Tried to lint and run the repo test suite from the current workspace state after confirming the declared scripts in `package.json` and Rust feature/dependency layout in `src-tauri/Cargo.toml`.
- Rust validation could not start because `rustc` and `cargo` are still missing, so `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` and `cargo test --manifest-path src-tauri/Cargo.toml --all-features` remain blocked before project code execution.
- Frontend/UI validation status from the current checkout: `npm run test:ui` passes with 3 tests green even without `node_modules`, but `npm run build` fails immediately with `tsc: not found` because dependencies are not installed and `node_modules/.bin/tsc` is absent.
- Net result: only the dependency-free UI render tests could be executed successfully in the current environment; full lint/test/build validation still requires installing Rust plus the JavaScript and native prerequisites documented in `README.md`.

## 2026-03-24T09:16:15Z - GPT-5.4 - Added runnable dev environment setup script
- Added `setup-dev-env.sh` at the repo root so the user can run one script to install Rust, ensure `pnpm` is available, install the Ubuntu native dependencies from `README.md`, install JavaScript dependencies, and run the documented validation commands.
- The script sources `$HOME/.cargo/env`, falls back to `npm install -g pnpm` if `corepack enable pnpm` fails, verifies the required `pkg-config` entries, and then runs `cargo clippy`, `cargo test`, `pnpm test:ui`, and `pnpm build`.

## 2026-03-24T09:22:43Z - GPT-5.4 - setup-dev-env.sh hardened against corepack pnpm failures
- Updated `setup-dev-env.sh` so the `pnpm` bootstrap path no longer depends on the failing `corepack enable pnpm` flow seen on this machine.
- The script now tries a quieter `corepack enable` only if `pnpm` is still missing, then installs `pnpm` with `npm install -g pnpm` and adds the npm global bin directory to `PATH` for the current run before verifying `pnpm --version`.

## 2026-03-24T09:24:58Z - GPT-5.4 - Removed corepack entirely from setup-dev-env.sh
- The previous hardening was not sufficient because the script still contained a `corepack` branch and the user continued hitting the same key-signature failure during the `pnpm` bootstrap step.
- `setup-dev-env.sh` now skips `corepack` completely and uses only `npm install -g pnpm`, then prepends the npm global bin directory to `PATH` for the current script execution before verifying `pnpm --version`.

## 2026-03-24T09:26:36Z - GPT-5.4 - setup-dev-env.sh now removes the corepack pnpm shim
- Confirmed the machine's `pnpm` path was `/home/phil/.nvm/versions/node/v22.11.0/bin/pnpm`, a symlink to `../lib/node_modules/corepack/dist/pnpm.js`, so merely checking `command -v pnpm` was not enough.
- Updated `setup-dev-env.sh` to detect and remove that shim, run `npm install -g --force pnpm`, and then use the npm-global `pnpm` binary path directly for version check, dependency install, UI tests, and build.

## 2026-03-24T09:36:21Z - GPT-5.4 - Validation status after environment repair
- The shell session can now see `rustc 1.94.0`, `cargo 1.94.0`, and `pnpm 10.32.1`, so the basic Rust/Node toolchain setup is usable from the current environment.
- `cargo test --manifest-path src-tauri/Cargo.toml` now passes with 39 Rust tests green, and `pnpm test:ui` passes with 3 UI tests green.
- Full-feature validation is still blocked by native dependency/toolchain issues outside the Rust source: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` failed in `openssl-sys` because OpenSSL was not discoverable via `pkg-config`, and `cargo test --manifest-path src-tauri/Cargo.toml --all-features` failed in `leptonica-sys` because Clang could not find the standard header `stddef.h` while generating bindings.
- The passing default Rust test run also surfaced code warnings in `src-tauri/src/browser.rs` (unused imports and an unread `config` field), which Clippy will require fixing once the remaining native environment blockers are resolved.

## 2026-03-24T09:46:49Z - GPT-5.4 - Added clang/libclang to setup prerequisites
- Updated `README.md` and confirmed `setup-dev-env.sh` includes `clang` and `libclang-dev` in the Ubuntu prerequisite list.
- This is meant to unblock bindgen-driven crates such as `leptonica-sys`, which can fail with `fatal error: 'stddef.h' file not found` when libclang is missing even though GCC headers are present.

## 2026-03-24T09:59:51Z - GPT-5.4 - setup-dev-env.sh now upgrades unsupported Node installs
- Updated `setup-dev-env.sh` to require a Vite-compatible Node version and upgrade through `nvm` to `22.12.0` when the current shell is on an older release such as `22.11.0`.
- The script now removes `node_modules` before reinstalling JavaScript dependencies so optional native packages like rolldown bindings are refreshed after a Node version change.
- Updated `README.md` to document the supported Node ranges (`20.19+` or `22.12+`) and the manual recovery steps for the Vite/rolldown build failure seen in `output.log`.

## 2026-03-24T10:04:24Z - GPT-5.4 - Full lint and unit tests green after environment fixes
- Re-ran validation with `rustc 1.94.0`, `cargo 1.94.0`, `Node.js v22.12.0`, and `pnpm 10.32.1` visible in the active shell after the setup-script environment repairs.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` completed successfully, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` passed with 39 Rust tests green, and `pnpm test:ui` passed with 3 UI tests green.
- The previous native dependency blockers (`openssl-sys`, `leptonica-sys`, and the Vite/rolldown Node mismatch) are no longer preventing the documented validation flow on this machine.

## 2026-03-24T10:40:41Z - GPT-5.4 - Wave 1 browser history and scroll tools implemented
- Implemented `go_back`, `go_forward`, `reload_page`, and `scroll_page` end to end across `src-tauri/src/browser.rs`, `src-tauri/src/app_core.rs`, and `src-tauri/src/commands.rs`, including planner dispatch, executor trait support, mock dispatch tests, live Chromium history navigation, reload, and JS-backed scrolling.
- Added shared `ScrollDirection` and `ScrollTarget` enums from the spec, and now derive runtime `BrowserHistoryState` from Chromium's actual navigation history instead of only synthetic state advancement.
- Updated `docs/TODO.md` to mark the four Wave 1 browser tools complete and to mark browser history signal tracking complete.
- Validation after the implementation: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` passed, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` passed with 43 Rust tests green, and `pnpm test:ui` passed with 3 UI tests green.

## 2026-03-24T10:47:30Z - GPT-5.4 - Browser tool changes pushed to origin/master
- Committed the browser-history and setup-script changes as `548336f` (`Implement browser history tools`) and pushed that commit to `origin/master`.
- At push time, the validation baseline was still green: clippy passed, Rust unit tests passed with 43 tests, and UI tests passed with 3 tests.

## 2026-03-24T11:05:56Z - GPT-5.4 - Wave 1 narration control tools implemented
- Implemented `read_region`, `read_next_region`, `read_previous_region`, and `stop_speaking` across `src-tauri/src/commands.rs`, `src-tauri/src/app_core.rs`, `src-tauri/src/state.rs`, and `src-tauri/src/narration.rs`.
- The narration slice currently covers deterministic cursor navigation and speaking-state tracking, including interrupt handling, reached-start/end behavior, and runtime speaking metadata; it does not yet introduce a full TTS playback engine.
- Added focused narration/state/dispatcher tests and revalidated the repo with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (52 Rust tests green), and `pnpm test:ui` (3 UI tests green).

## 2026-03-24T11:10:35Z - GPT-5.4 - Validation rerun still green
- Re-ran the full documented validation flow without code changes to confirm the current worktree is still healthy.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` passed, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` passed with 52 Rust tests green, and `pnpm test:ui` passed with 3 UI tests green.

## 2026-03-24T11:12:17Z - GPT-5.4 - Recommended next slice is real narration playback
- After landing the narration control/tooling layer, the next highest-value gap is actual speech output: `read_region` and related tools update runtime narration state but do not yet synthesize and play audio.
- Recommended next implementation slice: wire `tts.rs` and `audio_io.rs` for real narration playback, interruption, and persisted volume/speed application so the voice-first path becomes end-to-end functional.

## 2026-03-24T11:24:41Z - GPT-5.4 - Real narration playback wired through TTS and rodio
- `src-tauri/src/tts.rs` now resolves the configured local TTS profile, validates the KittenTTS backend/model path/sample rate, lazily loads `kitten_tts_rs`, and synthesizes narration audio with the persisted voice and playback-speed settings.
- `src-tauri/src/audio_io.rs` now owns live rodio playback handles, starts sample-buffer playback, applies active volume updates, and supports explicit interruption/active-playback checks so runtime speaking state can stay synchronized with the real sink.
- `src-tauri/src/app_core.rs` now stops narration on navigation/reload, starts real playback for `read_region` / `read_next_region` / `read_previous_region`, stops real playback for `stop_speaking`, and surfaces explicit tool errors for unavailable TTS/audio backends instead of silent fallback behavior.
- Updated `docs/TODO.md` and the session plan to mark the landed local playback work complete; validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (56 Rust tests green), and `pnpm test:ui` (3 UI tests green).

## 2026-03-24T11:47:25Z - GPT-5.4 - Preparing playback slice push
- The local narration playback slice is ready to commit from a green worktree state: clippy passed, Rust unit tests passed with 56 tests, and UI tests passed with 3 tests.
- The tracked changes to be pushed are limited to the playback-related Rust/backend files plus `docs/TODO.md` and `memory.md`; the untracked local artifact `output.log` remains intentionally excluded.

## 2026-03-24T11:49:11Z - GPT-5.4 - Recommended next slice is ASR and listening tools
- With narration playback now committed and pushed, the next largest voice-first gap is spoken input: the deterministic `start_listening`, `stop_listening`, and `transcribe_command` tools remain unimplemented.
- Recommended next implementation slice: wire local Whisper-backed ASR and runtime listening lifecycle state first, then add optional remote OpenAI ASR later without changing the deterministic tool contracts.

## 2026-03-24T12:58:19Z - GPT-5.4 - Wave 1 ASR and listening tools implemented
- `src-tauri/src/asr.rs` now provides real microphone capture and local Whisper transcription via `cpal` and `whisper-rs`, including mono conversion, 16 kHz resampling, explicit runtime errors, and one-shot transcription support.
- `src-tauri/src/app_core.rs`, `src-tauri/src/commands.rs`, and `src-tauri/src/state.rs` now wire `start_listening`, `stop_listening`, and `transcribe_command` through the deterministic executor path, keep runtime listening state synchronized, and expose the real `last_transcript` through `get_agent_state`.
- Updated `docs/TODO.md` and the session plan to mark the landed ASR slice complete; validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (62 Rust tests green), and `pnpm test:ui` (3 UI tests green).

## 2026-03-24T13:00:45Z - GPT-5.4 - ASR listening tools pushed to origin/master
- Committed the ASR/listening slice as `49f6abb` (`Implement ASR listening tools`) and pushed it to `origin/master`.
- The pushed state includes the local Whisper-backed listening lifecycle, deterministic transcription tools, updated TODO tracking, and the green validation baseline of clippy plus 62 Rust tests and 3 UI tests.

## 2026-03-24T13:23:58Z - GPT-5.4 - Planner command-resolution path implemented
- `src-tauri/src/commands.rs` now exposes the planner-facing helper layer for the Commands slice: plannable-tool filtering, input-schema lookup, planner-output validation, bundled `docs/SKILLS.md` parsing, project/user `SKILL.md` discovery, and precedence-aware skill ranking.
- `src-tauri/src/app_core.rs` and `src-tauri/src/lib.rs` now add a real `resolve_command` entrypoint that assembles `PlannerInput` from runtime state, calls the configured remote OpenAI planner with structured JSON output requirements, and returns validated `PlannerOutput` values through Tauri; local planner mode still returns an explicit unimplemented error.
- Updated `docs/TODO.md`, the session plan, and the frontend `tauri-api.ts` / `main.ts` exports to reflect the new planner path; validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`.

## 2026-03-24T13:26:11Z - GPT-5.4 - Planner command-resolution slice pushed to origin/master
- Committed the planner command-resolution slice as `f337fde` (`Implement planner command resolution`) and pushed it to `origin/master`.
- The pushed state includes `resolve_command`, bundled/project/user skill loading and ranking, remote OpenAI planner integration, planner-output validation, and the green validation baseline of clippy plus 66 Rust tests, 3 UI tests, and a passing frontend build.

## 2026-03-24T18:25:30Z - GPT-5.4 - Ollama planner support added and validated
- Added `RemoteProviderKind::Ollama`, an `ollama-default` remote planner profile, and runtime planner dispatch that targets Ollama's OpenAI-compatible `/v1/chat/completions` endpoint without replacing the existing OpenAI or local `llama.cpp` paths.
- `src-tauri/src/app_core.rs` now uses an Ollama-specific request shape (`response_format = JsonObject`, `max_tokens`) while preserving the same bounded `PlannerOutput` validation path used by the other planner providers.
- Replaced a brittle config-template assertion with a focused test that selects `ollama-default` explicitly and verifies the loaded provider/model/base URL, matching the config loader's behavior of materializing referenced profiles.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (68 Rust tests green), `pnpm test:ui` (3 tests green), and `pnpm build`; the known `llama-cpp-sys-2` shared-library symlink cleanup workaround between Cargo phases is still required.

## 2026-03-24T19:05:55Z - GPT-5.4 - Removed llama.cpp planner support in favor of remote providers
- Removed the `llama-cpp-2` dependency, the `local-planner` Cargo feature, the local planner runtime path in `src-tauri/src/app_core.rs`, and the shipped planner-local config/profile entries so command planning now uses only remote providers.
- Planner config validation in `src-tauri/src/config.rs` now enforces `providers.planner.mode = "remote"` and rejects `local_profile` or `failover_to_local` for planning, while leaving local TTS and local ASR support intact.
- Updated `config.example.toml`, `docs/SPECS.md`, `docs/TODO.md`, and `README.md` so the documented planner story is OpenAI or Ollama only, with no remaining `llama.cpp` or local-Qwen references in source/docs/config.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (68 Rust tests green), `pnpm test:ui` (3 tests green), and `pnpm build`; the Vite Node 22.11.0 warning still appears in this environment but the build succeeds.

## 2026-03-24T19:09:00Z - GPT-5.4 - Added manual Node upgrade helper script
- Added `fix-node-version.sh` at the repo root as a small manual helper that switches the shell to `Node.js 22.12.0` via `nvm`, clears `node_modules`, and reinstalls dependencies with `pnpm install`.
- The script is intentionally narrower than `setup-dev-env.sh`: it only addresses the Vite warning caused by running under unsupported `Node.js 22.11.0`.

## 2026-03-24T19:27:47Z - GPT-5.4 - Remote OpenAI TTS selection implemented
- `src-tauri/src/tts.rs` now implements the remote TTS path for `providers.tts.mode = "remote"` using the existing `async-openai` client with its `audio` feature enabled, issuing OpenAI speech requests and decoding WAV responses into the same `SynthesizedSpeech` sample buffer shape used by local playback.
- Remote voice selection now uses an OpenAI-compatible runtime voice override when the current voice is one of the built-in OpenAI voices; otherwise it falls back to the configured remote profile voice so the default local `Bruno` voice does not break remote TTS mode.
- Added focused TTS unit tests for remote voice resolution, remote audio-format validation, and WAV decoding, and updated `docs/TODO.md` to mark remote OpenAI TTS, remote TTS provider selection, and remote native speed control as implemented.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (72 Rust tests green), `pnpm test:ui` (3 tests green), and `pnpm build`; the current shell still prints the known Vite warning because it is using Node 22.11.0.

## 2026-03-24T19:31:07Z - GPT-5.4 - Validation rerun after remote TTS landed
- Re-ran the standard validation commands successfully after sourcing Cargo's shell environment in this session: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, and `pnpm test:ui`.
- Current validated baseline is `72` passing Rust tests and `3` passing UI tests with no lint errors; the only remaining noise is Node's experimental warning during `pnpm test:ui`.

## 2026-03-24T19:33:06Z - GPT-5.4 - Verified Node 22.12.0 clears the Vite version warning
- Switching the shell to `Node.js 22.12.0` with `nvm use 22.12.0` removes the Vite unsupported-version warning seen under `Node.js 22.11.0`; `pnpm build` then runs cleanly with no Node-version warning.

## 2026-03-24T19:37:57Z - GPT-5.4 - UI test script now suppresses only the known Node experimental warning
- Updated `package.json` so `pnpm test:ui` runs `node --disable-warning=ExperimentalWarning --experimental-strip-types --experimental-specifier-resolution=node --test src/**/*.test.mjs`, preserving the direct `.ts` test import path while silencing only the known process warning emitted by Node's type-stripping runtime.
- Verified the script still passes cleanly under `Node.js 22.12.0` with `3` passing UI tests and no experimental warning noise.

## 2026-03-24T19:40:00Z - GPT-5.4 - Cargo is now available automatically in fresh bash shells
- Moved the guarded `~/.cargo/env` sourcing to the top of `~/.bashrc`, before the existing non-interactive early return, so fresh bash shells now get Cargo on `PATH` without manually running `source "$HOME/.cargo/env"`.
- Verified with both `bash -lc 'command -v cargo && cargo --version'` and `bash -ic 'command -v cargo && cargo --version'`.

## 2026-03-24T19:51:32Z - GPT-5.4 - Remote OpenAI ASR selection implemented
- `src-tauri/src/asr.rs` now implements the remote ASR path for `providers.asr.mode = "remote"` by resolving the configured remote ASR profile, supporting `RemoteProviderKind::OpenAi`, encoding captured microphone audio as mono 16 kHz PCM16 WAV, and sending it through `async-openai`'s `/audio/transcriptions` client.
- Remote ASR now resolves configured secrets explicitly, returns typed errors for missing profiles, unsupported providers, secret failures, WAV encoding failures, request-build failures, request timeouts, and request failures, and maps those errors through `src-tauri/src/app_core.rs` to bounded tool-error codes.
- `docs/TODO.md` now marks optional OpenAI-backed remote ASR, ASR provider selection, and the "Remote ASR selected → transcript is returned" slice as complete.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (74 Rust tests green), and `pnpm test:ui` (3 tests green).

## 2026-03-24T20:12:40Z - GPT-5.4 - Push-to-talk landed through the existing deterministic listening pipeline
- Added direct Tauri invoke commands for `start_listening`, `stop_listening`, and `transcribe_command`, then used those from the frontend so push-to-talk stays on the same deterministic Rust tool path as planner-driven listening and transcription.
- `src/main.ts` now renders a push-to-talk panel, supports hold-to-talk via both Space-bar and press-and-hold pointer interaction, starts microphone capture on press, transcribes immediately on release using the active capture buffer with `auto_stop = true`, and then routes any non-empty transcript through `resolve_command` and `execute_planner_output`.
- `src/confirmation-panel.ts` now also renders the push-to-talk status panel UI, `docs/TODO.md` marks both push-to-talk and the push-to-talk button complete, and the UI test file covers idle, active, and error push-to-talk rendering states.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (74 Rust tests green), `pnpm test:ui` (6 UI tests green), and `pnpm build`.

## 2026-03-24T20:27:57Z - GPT-5.4 - Playback volume and speed persistence behavior is now explicitly covered
- The existing persistence path was already correct: `AppConfig::persist_audio_settings_for_app()` writes updated audio settings to disk, `AppState::from_config()` hydrates runtime audio state on startup, `AudioPlaybackController` uses the hydrated runtime volume for playback, and `TtsController` uses the hydrated runtime speed for both local and remote narration synthesis.
- Added focused tests in `src-tauri/src/audio_io.rs` and `src-tauri/src/state.rs` to verify `RuntimeAudioState` reflects persisted volume/speed/voice settings, zero volume maps to muted runtime state, and startup state hydration uses persisted audio values.
- Updated `docs/TODO.md` to mark audio settings persistence/validation, startup application of persisted playback volume and speed, deterministic audio-setting tool persistence, and restart persistence for playback volume/speed as complete.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (77 Rust tests green), and `pnpm test:ui` (6 UI tests green).

## 2026-03-24T20:50:38Z - GPT-5.4 - Nearby playback controls are now available in the UI
- `src-tauri/src/lib.rs` now exposes direct Tauri commands for `get_agent_state`, `set_playback_volume`, and `set_playback_speed`, and `src/tauri-api.ts` provides typed wrappers that keep the frontend on the existing deterministic Rust audio-setting path.
- `src/main.ts`, `src/confirmation-panel.ts`, and `src/styles.css` now render a nearby playback controls panel beside push-to-talk, initialize it from runtime audio state, update the displayed slider values locally while dragging, persist volume/speed on commit, and refresh from runtime after push-to-talk planner execution or confirmation resume so voice-driven audio changes stay in sync.
- `src/confirmation-panel.test.mjs` now covers nearby playback control rendering, disabled-saving state, and inline error rendering, and `docs/TODO.md` marks the nearby playback volume/speed UI items complete.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (77 Rust tests green), `pnpm test:ui` (9 UI tests green), and `pnpm build` under Node `22.12.0`.

## 2026-03-24T21:06:51Z - GPT-5.4 - Voice playback volume and speed commands now normalize locally
- `src-tauri/src/commands.rs` now deterministically normalizes playback volume and speed voice commands before remote planner resolution, including absolute volume percent/decimal input, relative volume/speed steps with small and large variants, mute, and current-value queries for both settings.
- Query phrases now infer `GetPlaybackVolume` and `GetPlaybackSpeed` instead of falling through to setter intents, and the direct resolver emits bounded `PlannerOutput` values that route through the existing `set_playback_volume`, `set_playback_speed`, and `report_result` tools.
- `src-tauri/src/app_core.rs` now checks the direct audio resolver before calling the remote planner and speaks `report_result` summaries through the existing TTS/audio playback path so hands-free volume/speed queries and confirmations answer aloud.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (81 Rust tests green), `pnpm test:ui` (9 UI tests green), and `pnpm build` under Node `22.12.0`.

## 2026-03-24T21:14:51Z - GPT-5.4 - Status UI now mirrors live runtime state in the frontend
- `src/confirmation-panel.ts` now renders a compact status panel for page title, current region, listening state, speaking state, browser visibility, and history availability, and `src/confirmation-panel.test.mjs` adds render coverage for both populated and error/fallback status states.
- `src/main.ts` now tracks a dedicated `StatusPanelState` and uses a shared `get_agent_state` refresh path to update both the status panel and nearby audio controls together on initial load, after successful push-to-talk transitions, and after planner execution or confirmation resume.
- `src/styles.css` now styles the status panel to match the nearby voice/audio controls, and `docs/TODO.md` marks the Status UI checklist items complete.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (81 Rust tests green), `pnpm test:ui` (11 UI tests green), and `pnpm build` under Node `22.12.0`.

## 2026-03-24T22:36:47Z - GPT-5.4 - Visible/headless browser mode is now directly toggleable in the UI
- `src-tauri/src/lib.rs` now exposes a direct `set_browser_visibility` Tauri command on top of the existing deterministic Rust tool, and `src/tauri-api.ts` adds the typed frontend wrapper and response type for browser mode changes.
- `src/confirmation-panel.ts`, `src/main.ts`, and `src/styles.css` now add a compact two-button `Visible`/`Headless` toggle inside the status panel, disable repeat clicks while a visibility change is in flight, refresh runtime panels after success, and surface inline errors while restoring the prior mode on failure.
- `src/confirmation-panel.test.mjs` now covers the toggle markup and disabled in-flight state, and `docs/TODO.md` marks the visible/headless toggle item complete.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (81 Rust tests green), `pnpm test:ui` (12 UI tests green), and `pnpm build` under Node `22.12.0`.

## 2026-03-25T00:22:46Z - GPT-5.4 - Browser visibility voice commands now normalize locally
- `src-tauri/src/commands.rs` now recognizes browser-visibility phrases such as `go headless`, `show the browser`, `hide the browser`, `make the browser visible`, and `toggle browser visibility`, infers `SetBrowserVisibility`, and emits deterministic planner steps for `set_browser_visibility` followed by spoken `report_result` feedback.
- `src-tauri/src/app_core.rs` now checks the direct browser-visibility resolver before the existing direct audio resolver so visibility mode changes stay bounded and do not depend on remote planner behavior.
- Added commands-layer regression coverage for browser-visibility intent inference, explicit headless normalization, and toggle behavior, and `docs/TODO.md` now marks browser-visibility command normalization complete.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (84 Rust tests green), `pnpm test:ui` (12 UI tests green), and `pnpm build` under Node `22.12.0`.

## 2026-03-25T00:29:36Z - GPT-5.4 - Status and history voice queries now normalize locally
- `src-tauri/src/commands.rs` now recognizes current-URL, status, history-availability, and listening-state phrases such as `what page am I on`, `can I go back`, `can I go forward`, and `are you listening`, infers `GetCurrentUrl` or `GetStatus`, and emits deterministic planner steps that explicitly route through `get_agent_state` or `get_runtime_status` before spoken `report_result` feedback.
- `src-tauri/src/app_core.rs` now checks the direct status-query resolver before the direct audio resolver so runtime-question voice commands stay bounded and do not depend on remote planner behavior.
- Added commands-layer regression coverage for status/history/listening intent inference plus current-URL, back-history, and listening summaries, and `docs/TODO.md` now marks the current-URL/runtime-status routing and status/history/listening normalization items complete.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (88 Rust tests green), `pnpm test:ui` (12 UI tests green), and `pnpm build` under Node `22.12.0`.

## 2026-03-25T00:36:26Z - GPT-5.4 - Form-filling voice phrases now normalize to form intent families
- `src-tauri/src/commands.rs` now recognizes focus/fill/type/submit/fill-and-submit phrases such as `focus the email field`, `fill the password field`, `type hello into the search field`, `submit this form`, and `fill the email field and then submit`, mapping them into `FillInput` or `SubmitForm` intent families before broader click/open routing.
- This slice intentionally normalizes only the intent families; the underlying form-execution tools remain future work, so the router now matches the documented command families without claiming the full form workflow is implemented.
- Added commands-layer regression coverage for form-filling and form-submission phrase inference, and `docs/TODO.md` now marks the form normalization item complete.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (89 Rust tests green), `pnpm test:ui` (12 UI tests green), and `pnpm build` under Node `22.12.0`.

## 2026-03-25T00:45:30Z - GPT-5.4 - Normalization examples now cover mixed and follow-up form utterances
- `docs/SPECS.md` now explicitly documents that mixed commands like `fill the email field and then submit` normalize to `SubmitForm`, while bounded ambiguous and follow-up form utterances like `choose California from the state list`, `no, the other field`, and `put Seattle there instead` remain in the `FillInput` family.
- `src-tauri/src/commands.rs` now classifies `the other field` and `there instead` as bounded `FillInput` follow-up phrases, and the command tests assert those documented examples directly so the spec and router stay aligned.
- This slice still stops at intent-family normalization; later context resolution is still responsible for deciding which field or value a follow-up correction refers to.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (89 Rust tests green), `pnpm test:ui` (12 UI tests green), and `pnpm build` under Node `22.12.0`.

## 2026-03-25T00:53:41Z - GPT-5.4 - Bounded fuzzy matching now corrects small command-keyword drift
- `src-tauri/src/commands.rs` now routes transcripts through a bounded canonicalization layer that fixes unambiguous single-typo or ASR-drift variants of existing command keywords such as `volum`, `spead`, `browsr`, `listenin`, `submitt`, `curent`, and `feild`, and also merges split compounds like `play back` and `head less`.
- The fuzzy layer stays deterministic and narrow: it only corrects a small whitelist of already-supported command words when the correction is unambiguous, and it does not add open-ended semantic recovery.
- Added regression coverage for fuzzy audio, browser visibility, status/current-URL, and form utterances, and `docs/TODO.md` plus `docs/SPECS.md` now mark and describe this bounded fuzzy-matching behavior.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (89 Rust tests green), `pnpm test:ui` (12 UI tests green), and `pnpm build` under Node `22.12.0`.

## 2026-03-25T03:24:27Z - GPT-5.4 - Push-to-talk now routes ASR directly into bounded command execution
- `src-tauri/src/app_core.rs` now provides `transcribe_and_execute_command(...)`, which runs deterministic transcription, then reuses the existing `resolve_command(...)` and `execute_planner_output(...)` flow when a transcript is present, returning either an `ExecutionOutcome` or a command-resolution `ToolError` alongside the transcription payload.
- `src-tauri/src/lib.rs` and `src/tauri-api.ts` now expose a typed `transcribe_and_execute_command` Tauri command, and `src/main.ts` uses it during push-to-talk release instead of manually chaining separate transcribe, resolve, and execute calls in the frontend.
- This keeps the voice-first flow bounded in the Rust runtime while still preserving the captured transcript in the UI when post-transcription routing fails, and `docs/TODO.md` now marks `Route ASR → command → action` complete.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (89 Rust tests green), `pnpm test:ui` (12 UI tests green), and `pnpm build` under Node `22.12.0`.

## 2026-03-25T03:32:55Z - GPT-5.4 - Runtime status UI now shows the last spoken transcript
- `src/main.ts` now hydrates `StatusPanelState.lastTranscript` from runtime `get_agent_state(...)` refreshes with `includeLastTranscript: true`, so the status panel reflects the same last spoken command tracked by the backend runtime.
- `src/tauri-api.ts`, `src/confirmation-panel.ts`, and `src/styles.css` now support and render a dedicated `Last transcript` status card with readable wrapping, while the existing inline push-to-talk transcript remains in place for immediate feedback.
- `src/confirmation-panel.test.mjs` now covers both transcript-present and transcript-empty status-panel rendering, and `docs/TODO.md` now marks `Display transcript in UI` complete.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (89 Rust tests green), `pnpm test:ui` (12 UI tests green), and `pnpm build` under Node `22.12.0`.

## 2026-03-25T03:51:12Z - GPT-5.4 - Repeat-region voice commands now resolve directly from the narration cursor
- `src-tauri/src/commands.rs` now recognizes bounded repeat phrases such as `repeat`, `repeat that`, `read that again`, and `say that again`, and `resolve_direct_repeat_command(...)` converts them into a deterministic `read_region` replay against the current narration cursor with `interrupt_current = true`.
- When no current narration region is available yet, the repeat resolver now returns a bounded `report_result` follow-up message instead of guessing what to replay, preserving the voice-first flow without open-ended fallback behavior.
- `src-tauri/src/app_core.rs` now checks the direct repeat resolver before broader planner resolution, `docs/TODO.md` marks `Repeat region` complete, and `docs/SPECS.md` documents the cursor-based repeat behavior.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (92 Rust tests green), `pnpm test:ui` (12 UI tests green), and `pnpm build` under Node `22.12.0`.

## 2026-03-25T04:07:40Z - GPT-5.4 - Read-title voice commands now resolve directly from current page state
- `src-tauri/src/commands.rs` now defines `IntentName::ReadTitle`, recognizes bounded title-reading phrases such as `read title`, `read the page title`, and `what is the title`, and `resolve_direct_read_title_command(...)` converts them into a deterministic spoken `report_result` step based on the current page title.
- When the current page does not have a readable title yet, the title resolver now returns a clear bounded spoken message instead of inventing one, preserving the voice-first flow without open-ended fallback behavior.
- `src-tauri/src/app_core.rs` now checks the direct read-title resolver before broader planner resolution, `docs/SKILLS.md` aligns the bundled `read_title` skill with `intent:ReadTitle`, and `docs/TODO.md` plus `docs/SPECS.md` now mark and describe the read-title behavior.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (95 Rust tests green), `pnpm test:ui` (12 UI tests green), and `pnpm build` under Node `22.12.0`.

## 2026-03-25T04:18:09Z - GPT-5.4 - Read-title slice prepared for commit and push
- The user requested that the validated `Read title` slice be checked in and pushed to `master`, so the current worktree should contain only the bounded title-reading command work plus its docs and memory updates.
- This keeps the project history aligned with the thin-slice workflow used so far: implement a bounded voice-first feature, validate it with the standard Rust and UI commands, then commit and push as a focused change.

## 2026-03-25T04:35:37Z - GPT-5.4 - Intent alignment now covers scroll and TTS voice command families
- `docs/SKILLS.md` now includes a bundled `scroll_page` skill tagged with `intent:Scroll`, closing a real gap where `Scroll` was a planner-visible intent without explicit bundled skill coverage.
- `src-tauri/src/commands.rs` now normalizes bounded TTS voice-setting phrases such as `change the voice to Bruno` and `switch to the Bella voice` to `IntentName::SetTtsVoice`, and the fuzzy command keyword set now includes `voice` so minor ASR drift like `voise` still routes cleanly.
- Commands-layer regression tests now verify explicit bundled intent coverage for the currently planner-visible command families and assert that matching voice-setting transcripts rank the bundled `set_tts_voice` skill first.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (98 Rust tests green), `pnpm test:ui` (12 UI tests green), and `pnpm build`; the current shell still warns on Node `22.11.0`, so the repo baseline remains Node `22.12.0+` for warning-free frontend builds.

## 2026-03-25T04:45:56Z - GPT-5.4 - Canonical planner JSON examples are now generated from real PlannerOutput values
- `src-tauri/src/commands.rs` now exposes `canonical_planner_output_examples()` with canonical `PlannerOutput` payloads for `GetStatus`, `ReadTitle`, `SetPlaybackVolume`, and a confirmation-gated `ClickElement` flow, keeping the examples tied to the real Rust contract instead of hand-written ad hoc JSON.
- `src-tauri/src/app_core.rs` now sends those examples in `PlannerPromptPayload` as shape references, and the planner system prompt now explicitly says the returned JSON must still use the current `planner_input.available_tools`, `planner_input.active_skill_names`, and exact `tool_input_schemas`.
- Commands-layer tests now verify both that the canonical planner examples validate against the live planner contract and that they serialize with the expected enum strings, snake_case argument keys, and the actual serde `StepTransition` representation used by the code today.
- `docs/SPECS.md` now mirrors those canonical planner payloads and corrects the documented `StepTransition` JSON shape to the real externally tagged serde form; validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (100 Rust tests green), `pnpm test:ui` (12 UI tests green), and `pnpm build`, with the same Node `22.11.0` warning still present in the current shell.

## 2026-03-25T04:56:20Z - GPT-5.4 - Canonical planner fixtures now validate against generated JSON schema
- `src-tauri/src/commands.rs` now includes test-only schema-matching helpers for the generated schemars features used by the planner contracts here, including local `$ref`, `type`, `required`, `properties`, `items`, `const`, `enum`, `allOf`, `anyOf`, and `oneOf`.
- Commands-layer regression tests now verify that every canonical `PlannerOutput` example matches the generated `planner_output_schema()` and that each step `arguments` object matches the generated `tool_input_schema(...)` for that step's `tool_name`.
- This closes the contract loop between canonical examples, generated schema, and runtime planner validation without adding a new schema-validation dependency.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (102 Rust tests green), `pnpm test:ui` (12 UI tests green), and `pnpm build`; the frontend build still warns because the current shell is on Node `22.11.0` rather than the repo baseline `22.12.0+`.

## 2026-03-25T05:01:47Z - GPT-5.4 - Submit-form planner outputs now require the confirmation path
- `src-tauri/src/commands.rs` now enforces a submit-specific planner validation rule: any `SubmitForm` plan must use `NeedsConfirmation`, set `requires_confirmation`, include non-empty `confirmation_reason` and `user_message`, and include a `confirm_action` step whose success transition is `RequestConfirmation`.
- `src-tauri/src/app_core.rs` now tells the planner explicitly that `SubmitForm` plans must always use `NeedsConfirmation` with `confirm_action` before any submit side effect, so the model guidance matches the executor contract.
- Commands-layer regression tests now cover both rejecting unsafe submit-form planner outputs and accepting a correctly confirmation-gated one.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (105 Rust tests green), `pnpm test:ui` (12 UI tests green), and `pnpm build`; the current shell still warns on Node `22.11.0`, so the repo baseline remains `22.12.0+`.

## 2026-03-25T05:12:13Z - GPT-5.4 - Planner input now carries safety settings for click and submit policy
- `src-tauri/src/commands.rs` now defines `PlannerSafetySettings` and includes it on `PlannerInput`, so remote planners receive the configured `confirmation_confidence_threshold`, `allow_click_without_confirmation`, and `always_confirm_submit` values as part of the planner contract.
- `src-tauri/src/app_core.rs` now populates `planner_input.safety` from config and the planner system prompt explicitly allows ordinary `ClickElement` plans to use `Ready` when `planner_input.safety.allow_click_without_confirmation` is true, while still reserving `NeedsConfirmation` for ambiguous or risky clicks.
- Canonical planner examples now include both `click_element_ready` and `click_element_with_confirmation`, and `docs/SPECS.md` mirrors that split so the contract demonstrates both the default click path and the protected click path.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (107 Rust tests green), `pnpm test:ui` (12 UI tests green), and `pnpm build`; the current shell still warns on Node `22.11.0`, so the repo baseline remains `22.12.0+`.

## 2026-03-25T05:16:09Z - GPT-5.4 - Click confirmation now uses the configured confidence threshold
- `src-tauri/src/app_core.rs` now feeds `self.config.safety.confirmation_confidence_threshold` into the find-element resolution helper, and that helper now requires confirmation whenever the best grounded candidate falls below the configured threshold, even when there is no close runner-up.
- The separate ambiguity-margin safeguard still applies, so confirmation is also required when the top two click candidates are too close to choose deterministically even if the top score clears the configured threshold.
- The planner system prompt now explicitly references `planner_input.safety.confirmation_confidence_threshold`, and `docs/SPECS.md` now documents that grounding-dependent side effects should require confirmation when the best confidence falls below that threshold.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (108 Rust tests green), `pnpm test:ui` (12 UI tests green), and `pnpm build`; the current shell still warns on Node `22.11.0`, so the repo baseline remains `22.12.0+`.

## 2026-03-25T05:22:21Z - GPT-5.4 - Confirmation gating is now a generic planner contract
- `src-tauri/src/commands.rs` now enforces confirmation generically: any `NeedsConfirmation` planner output must set `requires_confirmation = true`, include non-empty `confirmation_reason` and `user_message`, include a `confirm_action` step, and request confirmation from that step.
- Conversely, `Ready`, `Blocked`, and `Complete` planner outputs may no longer include `confirm_action`, set `requires_confirmation = true`, or carry `confirmation_reason`, so confirmation-only metadata cannot leak into non-gated plans.
- The submit-specific validator still runs first so `SubmitForm` retains its clearer specialized diagnostics, but click and other risky/ambiguous plans now follow the same bounded confirmation contract.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (111 Rust tests green), `pnpm test:ui` (12 UI tests green), and `pnpm build`; the current shell still warns on Node `22.11.0`, so the repo baseline remains `22.12.0+`.

## 2026-03-25T05:33:15Z - GPT-5.4 - Push-to-talk execution now uses a bounded replanning loop
- `src-tauri/src/app_core.rs` now implements an app-level bounded replanning loop above the low-level step runner. It converts accumulated execution traces into `recent_tool_results`, replans once with fresh runtime state, and aborts with `replan_limit_exceeded` if replanning is requested again.
- The low-level commands executor still emits `NeedsReplan` as before, but `transcribe_and_execute_command(...)` now routes spoken commands through the bounded loop so the user-facing push-to-talk path actually exercises the replanning contract.
- App-core regression tests now cover both a successful single replan with carried-forward tool history and the capped second replan case, preventing silent infinite-loop regressions.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (113 Rust tests green), `pnpm test:ui` (12 UI tests green), and `pnpm build`; the current shell still warns on Node `22.11.0`, so the repo baseline remains `22.12.0+`.

## 2026-03-25T05:39:43Z - GPT-5.4 - Bounded replanning slice revalidated for check-in
- Re-ran the standard validation set immediately before check-in: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, and `pnpm test:ui`.
- The bounded replanning slice remained green at `113` Rust tests and `12` UI tests, so the current repo baseline for this area is still clean going into commit and push.

## 2026-03-25T05:46:52Z - GPT-5.4 - Agent state now reports structured last tool-call metadata
- `src-tauri/src/commands.rs` now replaces the leftover `AgentStateData.last_action: Option<String>` field with `last_tool_call: Option<LastToolCallSummary>`, using structured `request_id`, `tool_name`, `ok`, and observation summary fields instead of free-form action text.
- `src-tauri/src/state.rs` now derives that runtime field from the latest serialized tool result in each `ExecutionOutcome` trace, so agent-state snapshots carry real executed-tool metadata rather than a placeholder string.
- `src/tauri-api.ts` and `docs/SPECS.md` now mirror the same structured contract, and `docs/TODO.md` marks `Return structured tool calls instead of free-form action text` complete.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (113 Rust tests green), `pnpm test:ui` (12 UI tests green), and `pnpm build`; the current shell still warns on Node `22.11.0`, so the repo baseline remains `22.12.0+`.

## 2026-03-25T05:48:59Z - GPT-5.4 - Structured tool-call slice revalidated for check-in
- Re-ran the standard validation set immediately before check-in: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, and `pnpm test:ui`.
- The structured tool-call slice remained green at `113` Rust tests and `12` UI tests, so the current repo baseline for this agent-state contract change is clean going into commit and push.

## 2026-03-25T05:59:44Z - GPT-5.4 - Navigation/readback phrase routing now bypasses the planner
- `src-tauri/src/commands.rs` now adds a dedicated `resolve_direct_navigation_readback_command(...)` path for `back`, `go forward`, `reload`, `next`, `previous`, and `stop reading` style phrases, generating bounded direct `PlannerOutput` values instead of leaving those simple commands to the LLM.
- The command normalizer now fuzzy-matches additional navigation/readback keywords such as `back`, `next`, `previous`, `repeat`, `stop`, `title`, and `transcribe`, so mild ASR drift like `refesh`, `prevous`, and `stpo` still maps cleanly.
- `src-tauri/src/app_core.rs` now runs that direct resolver before the planner path, and `docs/TODO.md` now checks off the covered phrase-to-intent items within the still-open broader backlog.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (116 Rust tests green), `pnpm test:ui` (12 UI tests green), and `pnpm build`; the current shell still warns on Node `22.11.0`, so the repo baseline remains `22.12.0+`.

## 2026-03-25T06:11:52Z - GPT-5.4 - Navigation/readback phrase slice revalidated for check-in
- Re-ran the standard validation set immediately before check-in: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, and `pnpm test:ui`.
- The navigation/readback phrase-routing slice remained green at `116` Rust tests and `12` UI tests, so the current repo baseline for this direct-routing change is clean going into commit and push.

## 2026-03-25T06:16:20Z - GPT-5.4 - Voice-input phrase routing now bypasses the planner
- `src-tauri/src/commands.rs` now adds `resolve_direct_voice_input_command(...)`, generating bounded direct `PlannerOutput` values for `start listening`, `stop listening`, and `transcribe command` style phrases instead of sending those simple requests through the planner.
- The shared phrase helpers now drive both `infer_intent_hint(...)` and the direct voice-input resolver, keeping intent hinting aligned with runtime shortcut behavior for phrases like `listen now`, `stop listenin`, and `what did i just say`.
- `src-tauri/src/app_core.rs` now runs that voice-input resolver before the planner path, and `docs/TODO.md` now checks off `start listening`, `stop listening`, and `transcribe command` within the broader phrase-mapping backlog.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (119 Rust tests green), `pnpm test:ui` (12 UI tests green), and `pnpm build`; the current shell still warns on Node `22.11.0`, so the repo baseline remains `22.12.0+`.

## 2026-03-25T06:20:37Z - GPT-5.4 - Voice-input phrase slice revalidated for check-in
- Re-ran the standard validation set immediately before check-in: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, and `pnpm test:ui`.
- The voice-input phrase-routing slice remained green at `119` Rust tests and `12` UI tests, so the current repo baseline for this direct-routing change is clean going into commit and push.

## 2026-03-25T06:33:33Z - GPT-5.4 - Open-url phrase routing now bypasses the planner
- `src-tauri/src/commands.rs` now adds `resolve_direct_open_url_command(...)`, generating a bounded direct `PlannerOutput` for spoken open-url commands instead of sending those simple requests through the planner.
- The open-url shortcut includes a small spoken URL normalizer that accepts explicit absolute URLs plus bounded spoken forms such as `github dot com slash features` and `localhost colon 3000`, defaulting to `https://` for ordinary hosts and `http://` for local development targets.
- `src-tauri/src/app_core.rs` now runs that resolver before the planner path, and `docs/TODO.md` now checks off `open url` within the broader phrase-mapping backlog.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (121 Rust tests green), `pnpm test:ui` (12 UI tests green), and `pnpm build`; the current shell still warns on Node `22.11.0`, so the repo baseline remains `22.12.0+`.

## 2026-03-25T06:40:44Z - GPT-5.4 - Open-url phrase slice revalidated for check-in
- Re-ran the standard validation set immediately before check-in: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, and `pnpm test:ui`.
- The open-url phrase-routing slice remained green at `121` Rust tests and `12` UI tests, so the current repo baseline for this direct-routing change is clean going into commit and push.

## 2026-03-25T06:46:56Z - GPT-5.4 - Read-page phrase routing now bypasses the planner
- `src-tauri/src/commands.rs` now adds `resolve_direct_read_page_command(...)`, generating a bounded direct `PlannerOutput` for page-reading phrases such as `read page`, `read this page`, and `read current page` instead of sending those simple requests through the planner.
- When runtime page regions are already available, the direct resolver restarts narration from the first readable region with `read_region`; when regions are missing but an active page exists, it refreshes the page model with `extract_page_model` and then begins from the top with `read_next_region`.
- If there is no active page yet, the runtime now returns a bounded spoken follow-up via `report_result` instead of failing later with a missing-page tool error, and `docs/TODO.md` now checks off `read page` within the broader phrase-mapping backlog.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (125 Rust tests green), `pnpm test:ui` (12 UI tests green), and `pnpm build`; the current shell still warns on Node `22.11.0`, so the repo baseline remains `22.12.0+`.

## 2026-03-25T07:06:18Z - GPT-5.4 - Read-page phrase slice revalidated for check-in
- Re-ran the standard validation set immediately before check-in: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, and `pnpm test:ui`.
- The read-page phrase-routing slice remained green at `125` Rust tests and `12` UI tests, so the current repo baseline for this direct-routing change is clean going into commit and push.

## 2026-03-25T08:59:41Z - GPT-5.4 - Focus-field phrase routing now bypasses the planner when grounding is deterministic
- `src-tauri/src/commands.rs` now adds `parse_direct_focus_field_command(...)`, keeping `focus ... field` parsing aligned with the existing intent-hint normalization and ASR-drift handling.
- `src-tauri/src/app_core.rs` now resolves `focus field` phrases directly against the current page model by scoring only visible enabled `Input`, `TextArea`, and `Select` controls with stable `dom_locator` values, then emitting a bounded `click_element` plan when a single deterministic field match exists.
- When the request is under-specified, ambiguous, or there is no active/focusable field available, the runtime now returns a bounded spoken follow-up via `report_result` instead of guessing, and `docs/SKILLS.md` now aligns `focus_field` with the actual runtime tool surface by using `click_element`.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (129 Rust tests green), `pnpm test:ui` (12 UI tests green), and `pnpm build`; the current shell still warns on Node `22.11.0`, so the repo baseline remains `22.12.0+`.

## 2026-03-25T17:33:09Z - GPT-5.4 - Focus-field phrase slice revalidated for check-in
- Re-ran the standard validation set immediately before check-in: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, and `pnpm test:ui`.
- The focus-field phrase-routing slice remained green at `129` Rust tests and `12` UI tests, so the current repo baseline for this direct-routing change is clean going into commit and push.

## 2026-03-25T17:51:29Z - GPT-5.4 - Fill-field direct routing landed
- `src-tauri/src/browser.rs` now implements live `focus_element` and `type_into_element` DOM actions, including deterministic selector resolution, value updates, and browser history/title snapshots after the action.
- `src-tauri/src/app_core.rs` now executes those form-entry tools, updates current-page field values after successful typing, routes `fill field` / `type into field` directly when the target field is deterministic, and upgrades the existing `focus field` shortcut to use `focus_element`.
- `src-tauri/src/commands.rs` now exposes the new tool schemas/dispatcher paths and parses fill phrases without destroying the dictated text payload.
- Validation baseline for this slice is green: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build` all pass; the Vite build still emits the existing Node `22.11.0` vs `22.12.0+` warning.

## 2026-03-25T17:55:01Z - GPT-5.4 - Repo revalidated after fill-field slice
- Re-ran the standard validation set on the current worktree: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, and `pnpm test:ui`.
- The repo remains green at `135` Rust tests and `12` UI tests, with no new validation failures introduced since the fill-field changes landed.

## 2026-03-25T17:56:13Z - GPT-5.4 - Fill-field slice revalidated for check-in
- Re-ran the standard validation set immediately before check-in: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, and `pnpm test:ui`.
- The fill-field slice remains green at `135` Rust tests and `12` UI tests, so the current repo baseline for this direct-routing and form-entry-tool change is clean going into commit and push.

## 2026-03-25T18:07:55Z - GPT-5.4 - Submit-form direct routing landed
- `src-tauri/src/commands.rs` now exposes `submit_active_form`, validates it as a planner-visible tool, adds it to `registered_tools()`, and prevents `fill ... and submit` utterances from being consumed by the direct fill-field shortcut.
- `src-tauri/src/browser.rs` now implements live `submit_active_form(...)`, resolving a specific form, the focused form, or the sole visible form before submitting and then refreshing page state/history metadata.
- `src-tauri/src/app_core.rs` now executes `submit_active_form` and routes direct `submit form` utterances into a confirmation-gated `confirm_action` + `submit_active_form` plan when the form target is deterministic, while using bounded follow-ups for ambiguity.
- Validation baseline for this slice is green: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build` all pass; the Vite build still emits the existing Node `22.11.0` vs `22.12.0+` warning.

## 2026-03-25T18:14:42Z - GPT-5.4 - Submit-form slice revalidated for check-in
- Re-ran the standard validation set immediately before check-in: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, and `pnpm test:ui`.
- The submit-form slice remains green at `139` Rust tests and `12` UI tests, so the current repo baseline for this direct-routing and form-submission-tool change is clean going into commit and push.
