# Refactor TODO 2 — blind_browser

Based on a structural analysis of the codebase in June 2026.  
**Status key:** PENDING · IN PROGRESS · DONE

---

## Goal

After REFACTOR1, the largest files are still well over 600 lines:
`app_core/mod.rs` (~1902), `browser.rs` (~1779), `commands/contracts.rs` (~1308),
`app_core/interaction_tools.rs` (~1311), `lib.rs` (~1073), `commands/routing/mod.rs` (~1061),
`commands/registry.rs` (~1054), `tts.rs` (~958), `commands/validators.rs` (~808),
`app_core/voice_tools.rs` (~799), `asr.rs` (~756), `src/tauri-api.ts` (~999).

This second pass continues the structural cleanup with the same constraints as REFACTOR1:
pure structural refactors, no behavior changes, full validation after every phase.

### Validation gate
```bash
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
pnpm lint
pnpm test:ui
pnpm build
```

---

## Implementation strategy

Work in this order to keep the codebase green at every step:

1. Split `app_core/mod.rs` — reduces the God object at the center of the backend.
2. Split `browser.rs` — largest self-contained module; moves the embedded JS blob out.
3. Split `tts.rs` and `asr.rs` — clear local/remote separation in both.
4. Split `commands/contracts.rs` — type-only splits; many files depend on it, use `pub use` carefully.
5. Split large `app_core/` submodules — `interaction_tools.rs`, `extraction_tools.rs`, `form_fill.rs`, `voice_tools.rs`.
6. Split `commands/validators.rs` — group by tool category.
7. Split `commands/registry.rs` — skill parsing separate from tool registry.
8. Split `lib.rs` — move command handler groups to `command_handlers/` submodule.
9. Split `commands/routing/mod.rs` — move `resolve_direct_*` functions to dedicated files.
10. Split `src/tauri-api.ts` — separate types from API functions.
11. Split `src/app-shell.ts` — separate theme, navigation components, and control state.

---

## Phase 1 — Split `app_core/mod.rs` (~1902 lines) — DONE

`mod.rs` still contains the `AppCore` struct, constructor, all configuration setters, API key
management, model management, narration state, core command dispatch, state snapshots, remote
planner HTTP integration, error utilities, and ID generation — at least six distinct concerns.

### Target layout

```
src-tauri/src/app_core/
  mod.rs              # AppCore struct + field declarations, new(), submodule declarations,
                      #   core dispatch methods: execute_planner_output, resolve_command,
                      #   resume_after_confirmation, execute_planner_output_validated;
                      #   error/ID utilities (~550 lines target)
  runtime_config.rs   # impl AppCore: apply_audio_settings, set_tts_provider_mode,
                      #   set_asr_provider_mode, set_planner_provider_mode, set_tts_voice,
                      #   set_tts_model_selection, test_remote_planner_api_key,
                      #   list_remote_planner_models, test_remote_tts_api_key,
                      #   test_remote_asr_api_key, set_model_management_settings,
                      #   download_active_local_tts_model, download_active_local_asr_model,
                      #   start_listening, stop_listening
  narration.rs        # impl AppCore: start_speaking_region, stop_speaking, narrate_text,
                      #   current_page_region, read_region_at_index, read_next_region,
                      #   read_previous_region, read_page, stop_reading, and region helpers
  remote_planner.rs   # impl AppCore: resolve_with_openai_planner, resolve_with_ollama_planner,
                      #   build_planner_messages, planner HTTP helpers (~330 lines)
  state_snapshots.rs  # impl AppCore: agent_state_data(), runtime_status_data(),
                      #   and all AgentStateData builder helpers (~110 lines)
```

### 1.1 Extract `runtime_config.rs` (lines ~238–569)

- [x] Move all `impl AppCore` methods for audio/TTS/ASR/planner configuration, API key
      testing, and model downloads into `app_core/runtime_config.rs`.
      Functions: `apply_audio_settings`, `set_tts_provider_mode`, `set_asr_provider_mode`,
      `set_planner_provider_mode`, `set_tts_voice`, `set_tts_model_selection`,
      `test_remote_planner_api_key`, `list_remote_planner_models`, `test_remote_tts_api_key`,
      `test_remote_asr_api_key`, `set_model_management_settings`,
      `download_active_local_tts_model`, `download_active_local_asr_model`.
- [x] Add `mod runtime_config;` in `mod.rs`.
- [x] Run `cargo check`.

### 1.2 Extract `narration.rs` (lines ~1172–1331)

- [x] Move all `impl AppCore` narration and audio-playback region methods into
      `app_core/narration.rs`: `start_speaking_region`, `stop_speaking`, `narrate_text`,
      `current_page_region`, `read_region_at_index`, `read_next_region`,
      `read_previous_region`, `read_page`, `stop_reading`, and any region-state helpers.
- [x] Add `mod narration;` in `mod.rs`.
- [x] Run `cargo check`.

### 1.3 Extract `remote_planner.rs` (lines ~1446–1774)

- [x] Move all remote planner HTTP integration methods into `app_core/remote_planner.rs`:
      `resolve_with_openai_planner`, `resolve_with_ollama_planner`, `build_planner_messages`,
      and all supporting HTTP/JSON helpers in that block.
- [x] Add `mod remote_planner;` in `mod.rs`.
- [x] Run `cargo check`.

### 1.4 Extract `state_snapshots.rs` (lines ~1333–1443)

- [x] Move `agent_state_data()`, `runtime_status_data()`, and all their builder helper
      functions into `app_core/state_snapshots.rs`.
- [x] Add `mod state_snapshots;` in `mod.rs`.
- [x] Run `cargo check`.

### 1.5 Final validation

- [x] Run full validation gate. `mod.rs` should be under 600 lines after all extractions.

---

## Phase 2 — Split `browser.rs` (~1779 lines) into `browser/` module — DONE

`browser.rs` contains `BrowserController`, `LiveBrowserSession`, six groups of helper
functions, and ~356 lines of embedded JavaScript for DOM extraction. The JS blob alone is
20% of the file.

### Target layout

```
src-tauri/src/browser/
  mod.rs           # BrowserController struct, pub API methods: open_url, click_element,
                   #   focus_element, type_into_element, submit_active_form,
                   #   capture_screenshot, get_html, eval_js, switch_visibility,
                   #   ensure_session (~450 lines target)
  config.rs        # BrowserSessionConfig, BrowserError enum, LoadState,
                   #   BrowserVisibilityMode, ScrollDirection, ScrollTarget,
                   #   and all other standalone state/config types
  session.rs       # LiveBrowserSession struct + impl, build_browser_config(),
                   #   ensure_live_element(), stable_dom_selector(), snapshot_page_state()
  navigation.rs    # impl BrowserController: go_back, go_forward, reload_page,
                   #   navigate_history; history-state helper functions
  page_metrics.rs  # impl BrowserController: get_page_metrics, scroll_page;
                   #   scroll/bounds helper functions
  dom_extraction.rs # impl BrowserController: extract_page_model; the embedded
                   #   DOM-extraction JavaScript as a `const` string or include_str!()
```

### 2.1 Create `browser/` directory and `mod.rs`

- [x] Create `src-tauri/src/browser/` directory.
- [x] Move `src-tauri/src/browser.rs` → `src-tauri/src/browser/mod.rs` as starting point.
- [x] Run `cargo check` — Rust resolves `browser/mod.rs` automatically.

### 2.2 Extract `config.rs`

- [x] Move `LoadState`, `BrowserVisibilityMode`, `ScrollDirection`, `ScrollTarget`,
      `BrowserSessionConfig`, `BrowserError`, and any other top-level enums/structs
      (not impl blocks) into `browser/config.rs`.
- [x] Add `mod config; pub use config::*;` in `browser/mod.rs`.
- [x] Run `cargo check`.

### 2.3 Extract `session.rs`

- [x] Move `LiveBrowserSession` struct and its `impl`, `build_browser_config()`,
      `ensure_live_element()`, `stable_dom_selector()`, `snapshot_page_state()`
      into `browser/session.rs`.
- [x] Add `mod session;` in `browser/mod.rs`.
- [x] Run `cargo check`.

### 2.4 Extract `navigation.rs`

- [x] Move `go_back()`, `go_forward()`, `reload_page()`, `navigate_history()`, and
      navigation-history helper functions into a second `impl BrowserController` block
      in `browser/navigation.rs`.
- [x] Add `mod navigation;` in `browser/mod.rs`.
- [x] Run `cargo check`.

### 2.5 Extract `page_metrics.rs`

- [x] Move `get_page_metrics()`, `scroll_page()`, and all scroll/metrics helper functions
      into `browser/page_metrics.rs`.
- [x] Add `mod page_metrics;` in `browser/mod.rs`.
- [x] Run `cargo check`.

### 2.6 Extract `dom_extraction.rs`

- [x] Move `extract_page_model()` and the large embedded JavaScript string into
      `browser/dom_extraction.rs`. If the JS constant is over 200 lines, move the raw
      JavaScript to `src-tauri/assets/extract_page_model.js` and load it via
      `include_str!("../assets/extract_page_model.js")`.
- [x] Add `mod dom_extraction;` in `browser/mod.rs`.
- [x] Run full validation gate.

---

## Phase 3 — Split `tts.rs` (~958 lines) into `tts/` module — DONE

`tts.rs` mixes local KittenTTS logic, remote OpenAI TTS, voice resolution helpers, WAV
decoding, and speech caching into one file.

### Target layout

```
src-tauri/src/tts/
  mod.rs   # TtsController struct, synthesize_narration() entry point, caching methods
           #   (cached_speech, store_cached_speech), cache types (CachedSpeechKey,
           #   CachedSynthesizedSpeech, CachedLocalTtsModel), TtsSettings, SynthesizedSpeech,
           #   TtsRuntimeError, TtsProviderKind
  local.rs # impl TtsController: synthesize_local(), local_model(),
           #   generate_local_samples(), normalized_model_path()
  remote.rs # impl TtsController: synthesize_remote(), synthesize_with_openai_remote(),
            #   resolved_voice(), resolved_remote_voice(), is_openai_builtin_voice(),
            #   openai_speech_response_format_value()
  wav.rs   # DecodedWav struct, decode_wav_samples() — PCM8/16/24/32-bit WAV decoding
```

### 3.1 Create `tts/` directory and `mod.rs`

- [x] Create `src-tauri/src/tts/` directory.
- [x] Move `src-tauri/src/tts.rs` → `src-tauri/src/tts/mod.rs`.
- [x] Run `cargo check`.

### 3.2 Extract `wav.rs`

- [x] Move `DecodedWav` and `decode_wav_samples()` into `tts/wav.rs`.
- [x] Add `mod wav; use wav::DecodedWav;` in `tts/mod.rs`.
- [x] Run `cargo check`.

### 3.3 Extract `local.rs`

- [x] Move `synthesize_local()`, `local_model()`, `generate_local_samples()`,
      `normalized_model_path()` into `tts/local.rs` as a second `impl TtsController` block.
- [x] Add `mod local;` in `tts/mod.rs`.
- [x] Run `cargo check`.

### 3.4 Extract `remote.rs`

- [x] Move `synthesize_remote()`, `synthesize_with_openai_remote()`, `resolved_voice()`,
      `resolved_remote_voice()`, `is_openai_builtin_voice()`,
      `openai_speech_response_format_value()` into `tts/remote.rs`.
- [x] Add `mod remote;` in `tts/mod.rs`.
- [x] Run full validation gate.

---

## Phase 4 — Split `asr.rs` (~756 lines) into `asr/` module — DONE

`asr.rs` combines audio capture hardware management, audio signal processing, local Whisper
transcription, remote OpenAI ASR, and WAV encoding.

### Target layout

```
src-tauri/src/asr/
  mod.rs        # AsrController struct, start_listening, stop_listening, is_listening,
                #   transcribe_command, capture_audio() orchestrator;
                #   AsrSettings, AsrTranscription, AsrProviderKind, AsrRuntimeError
  capture.rs    # CaptureSession struct + impl, build_input_stream(),
                #   build_typed_input_stream(), capture_input_data()
  processing.rs # CapturedAudio struct, interleaved_to_mono(), resample_linear()
  local.rs      # impl AsrController: transcribe_local(), transcribe_with_whisper(),
                #   normalized_model_path()
  remote.rs     # impl AsrController: transcribe_remote(), transcribe_with_openai_remote(),
                #   build_openai_transcription_request()
  wav.rs        # encode_wav_pcm16() — PCM16 WAV encoding for remote ASR upload
```

### 4.1 Create `asr/` directory and `mod.rs`

- [x] Create `src-tauri/src/asr/` directory.
- [x] Move `src-tauri/src/asr.rs` → `src-tauri/src/asr/mod.rs`.
- [x] Run `cargo check`.

### 4.2 Extract `capture.rs`

- [x] Move `CaptureSession` struct and all capture helper functions
      (`build_input_stream`, `build_typed_input_stream`, `capture_input_data`)
      into `asr/capture.rs`.
- [x] Add `mod capture;` in `asr/mod.rs`.
- [x] Run `cargo check`.

### 4.3 Extract `processing.rs`

- [x] Move `CapturedAudio`, `interleaved_to_mono()`, `resample_linear()` into
      `asr/processing.rs`.
- [x] Add `mod processing;` in `asr/mod.rs`.
- [x] Run `cargo check`.

### 4.4 Extract `remote.rs`

- [x] Move `transcribe_remote()`, `transcribe_with_openai_remote()`,
      `build_openai_transcription_request()` into `asr/remote.rs` as a second
      `impl AsrController` block.
- [x] Add `mod remote;` in `asr/mod.rs`.
- [x] Run `cargo check`.

### 4.5 Extract `local.rs`

- [x] Move `transcribe_local()`, `transcribe_with_whisper()`, `normalized_model_path()`
      into `asr/local.rs` as a second `impl AsrController` block.
- [x] Add `mod local;` in `asr/mod.rs`.
- [x] Run `cargo check`.

### 4.6 Extract `wav.rs`

- [x] Move `encode_wav_pcm16()` into `asr/wav.rs`.
- [x] Add `mod wav;` in `asr/mod.rs`.
- [x] Run full validation gate.

---

## Phase 5 — Split `commands/contracts.rs` (~1308 lines) — DONE

`contracts.rs` defines all data types for tool contracts, provider settings, planner I/O,
interaction primitives, and 34 tool input/output pairs. These fall into four distinct groups.

> **Note:** Many files `use` types from `commands::contracts`. All new sub-files must be
> `pub use`-re-exported from `contracts/mod.rs` so no callers change.

### Target layout

```
src-tauri/src/commands/contracts/
  mod.rs         # re-exports everything with `pub use`; ToolName enum, ToolError,
                 #   ToolWarning, ToolResult<T>, SerializedToolResult,
                 #   DeterministicToolExecutor trait, AvailableTool, SkillSummary,
                 #   PlannerToolHistoryEntry, LastToolCallSummary
  providers.rs   # ProviderSelectionStatus, TtsModelSettings, LocalTtsModelSettings,
                 #   TtsVoiceSettings, TtsProviderSettings, AsrProviderSettings,
                 #   LocalAsrModelSettings, RemotePlannerSettings, RemoteTtsSettings,
                 #   RemoteAsrSettings, ProviderFailoverSettings, ConfirmationSettings,
                 #   OcrThresholdSettings, AgentStateData, PageSnapshotData
  planner.rs     # PlannerSafetySettings, PlannerInput, PlannerStatus, BlockedReason,
                 #   IntentName, IntentSummary, StepTransition, PlannedStep,
                 #   PlannerOutput, PendingPlanExecutionState
  interaction.rs # NarrationInterruptionMode, NarrationBoundary, ElementVisibilityFilter,
                 #   ReloadMode, ClickMode, TextEntryMode, TextEntrySubmitMode,
                 #   TranscriptionStopMode, ScreenshotScope, ReportStatus,
                 #   ConfirmActionData, ConfirmActionInput, ConfirmActionResolution,
                 #   ReportResultData, ReportResultInput
  tools.rs       # All 34 tool input/output struct pairs
```

### 5.1 Create `contracts/` directory and `mod.rs`

- [x] Create `src-tauri/src/commands/contracts/` directory.
- [x] Move `commands/contracts.rs` → `commands/contracts/mod.rs`.
- [x] Run `cargo check` — verify all existing callers still compile.

### 5.2 Extract `providers.rs`

- [x] Move provider settings types (`ProviderSelectionStatus`, `TtsModelSettings`,
      `LocalTtsModelSettings`, `TtsVoiceSettings`, `TtsProviderSettings`,
      `AsrProviderSettings`, `LocalAsrModelSettings`, `RemotePlannerSettings`,
      `RemoteTtsSettings`, `RemoteAsrSettings`, `ProviderFailoverSettings`,
      `ConfirmationSettings`, `OcrThresholdSettings`, `AgentStateData`, `PageSnapshotData`)
      into `contracts/providers.rs`.
- [x] Add `mod providers; pub use providers::*;` in `contracts/mod.rs`.
- [x] Run `cargo check`.

### 5.3 Extract `planner.rs`

- [x] Move `PlannerSafetySettings`, `PlannerInput`, `PlannerStatus`, `BlockedReason`,
      `IntentName`, `IntentSummary`, `StepTransition`, `PlannedStep`, `PlannerOutput`,
      `PendingPlanExecutionState` into `contracts/planner.rs`.
- [x] Add `mod planner; pub use planner::*;` in `contracts/mod.rs`.
- [x] Run `cargo check`.

### 5.4 Extract `interaction.rs`

- [x] Move all interaction primitive enums and confirmation types into
      `contracts/interaction.rs`: `NarrationInterruptionMode`, `NarrationBoundary`,
      `ElementVisibilityFilter`, `ReloadMode`, `ClickMode`, `TextEntryMode`,
      `TextEntrySubmitMode`, `TranscriptionStopMode`, `ScreenshotScope`, `ReportStatus`,
      `ConfirmActionData`, `ConfirmActionInput`, `ConfirmActionResolution`,
      `ReportResultData`, `ReportResultInput`.
- [x] Add `mod interaction; pub use interaction::*;` in `contracts/mod.rs`.
- [x] Run `cargo check`.

### 5.5 Extract `tools.rs`

- [x] Move all 34 tool input/output struct pairs into `contracts/tools.rs`.
- [x] Add `mod tools; pub use tools::*;` in `contracts/mod.rs`.
- [x] Run full validation gate.

---

## Phase 6 — Split large `app_core/` submodules

Four submodules extracted in REFACTOR1 are still over 800 lines and contain natural seams.

### 6A — Split `interaction_tools.rs` (~1311 lines) — DONE

`interaction_tools.rs` contains `impl AppCore` element-interaction methods plus a large,
self-contained element-scoring subsystem that has no dependency on `AppCore`.

#### Target layout

```
src-tauri/src/app_core/
  interaction_tools.rs  # impl AppCore: execute_list_interactive_elements,
                        #   execute_find_element, execute_click_element,
                        #   execute_focus_element, execute_type_into_element,
                        #   execute_submit_active_form, resolve_clickable_element,
                        #   resolve_typeable_element, resolve_form_element (~400 lines target)
  element_scoring.rs    # Free functions: FindElementScore, AttributeHintSpec,
                        #   score_interactive_element, score_text_query_against_element,
                        #   score_attribute_hint, text_overlap_score,
                        #   rank_find_element_candidates, determine_find_element_resolution,
                        #   build_find_element_query, FindElementQuery,
                        #   filter_interactive_elements, focusable_field_elements,
                        #   submittable_form_elements, summarize_candidate_names,
                        #   summarize_form_candidate_names, describe_field_element,
                        #   describe_form_element, region_bbox_by_id,
                        #   normalize_optional_text, normalize_search_text,
                        #   tokenize_search_text
```

#### 6.A.1 Extract `element_scoring.rs`

- [x] Find the boundary in `interaction_tools.rs` where the scoring subsystem begins
      (the block of free functions around `FindElementScore` and `score_interactive_element`).
- [x] Move all scoring-subsystem free functions and types into `app_core/element_scoring.rs`.
- [x] Import from `interaction_tools.rs` via `use super::element_scoring::*;` or
      declare `pub(super) mod element_scoring;` in `app_core/mod.rs` and use from there.
- [x] Run `cargo check`.

#### 6.A.2 Final validation for 6A

- [x] Run full validation gate.

---

### 6B — Split `extraction_tools.rs` (~1087 lines)

`extraction_tools.rs` contains OCR execution, OCR text merging, and page model building —
three distinct concerns.

#### Target layout

```
src-tauri/src/app_core/
  extraction_tools.rs   # impl AppCore: execute_run_ocr, execute_merge_ocr_into_page_model,
                        #   execute_get_page_snapshot, execute_extract_page_model,
                        #   should_trigger_extract_page_model_ocr_fallback (~350 lines target)
  ocr_merge.rs          # merge_ocr_text_into_page_model, merged_region_text,
                        #   region_first_ocr_target_ids, extracted_text_metrics,
                        #   has_positive_bbox
  page_model_builder.rs # build_extracted_page_model, build_visible_text_excerpt,
                        #   infer_extraction_source, nested_tool_failure_as_extract_page_model,
                        #   extract_page_model_internal_failure, ocr_runtime_error_to_tool_error
```

#### 6.B.1 Extract `ocr_merge.rs`

- [ ] Move OCR merge and region-text helper functions into `app_core/ocr_merge.rs`.
- [ ] Add `mod ocr_merge;` in `app_core/mod.rs`.
- [ ] Run `cargo check`.

#### 6.B.2 Extract `page_model_builder.rs`

- [ ] Move page model construction helpers into `app_core/page_model_builder.rs`.
- [ ] Add `mod page_model_builder;` in `app_core/mod.rs`.
- [ ] Run full validation gate.

---

### 6C — Split `form_fill.rs` (~1123 lines)

`form_fill.rs` resolves direct fill commands and corrects recent fills — two distinct
command-handling concerns with different type dependencies.

#### Target layout

```
src-tauri/src/app_core/
  form_fill.rs        # impl AppCore: resolve_direct_focus_field_command,
                      #   resolve_direct_fill_field_command,
                      #   resolve_direct_fill_and_submit_command,
                      #   resolve_direct_submit_form_command,
                      #   resolve_direct_fill_command_internal (shared resolver)
  fill_correction.rs  # impl AppCore: resolve_recent_fill_correction_command;
                      #   PendingRecentFieldContext, RecentFieldContext,
                      #   ResolvedDirectFieldCommand, DirectFollowUpSpec,
                      #   selected_skills_for_fill_command, build_direct_fill_ready_output,
                      #   build_direct_fill_and_submit_ready_output,
                      #   build_direct_follow_up_output
```

#### 6.C.1 Extract `fill_correction.rs`

- [ ] Move `resolve_recent_fill_correction_command` and all its associated types and helpers
      (`PendingRecentFieldContext`, `RecentFieldContext`, `ResolvedDirectFieldCommand`,
      `DirectFollowUpSpec`, `selected_skills_for_fill_command`, `build_direct_fill_ready_output`,
      `build_direct_fill_and_submit_ready_output`, `build_direct_follow_up_output`)
      into `app_core/fill_correction.rs`.
- [ ] Add `mod fill_correction;` in `app_core/mod.rs`.
- [ ] Run full validation gate.

---

### 6D — Split `voice_tools.rs` (~799 lines)

`voice_tools.rs` contains two distinct groups: reading/narration tools and listening/
transcription tools.

#### Target layout

```
src-tauri/src/app_core/
  voice_tools.rs   # impl AppCore: execute_set_tts_voice, execute_set_playback_volume,
                   #   execute_set_playback_speed, execute_set_browser_visibility;
                   #   tts_runtime_error_to_tool_error, audio_playback_error_to_tool_error,
                   #   asr_runtime_error_to_tool_error
  reading_tools.rs # impl AppCore: execute_read_region, execute_read_next_region,
                   #   execute_read_previous_region, execute_stop_speaking
  listening_tools.rs # impl AppCore: execute_start_listening, execute_stop_listening,
                     #   execute_transcribe_command, transcribe_and_execute_command
```

#### 6.D.1 Extract `reading_tools.rs`

- [ ] Move `execute_read_region`, `execute_read_next_region`, `execute_read_previous_region`,
      `execute_stop_speaking` into `app_core/reading_tools.rs`.
- [ ] Add `mod reading_tools;` in `app_core/mod.rs`.
- [ ] Run `cargo check`.

#### 6.D.2 Extract `listening_tools.rs`

- [ ] Move `execute_start_listening`, `execute_stop_listening`, `execute_transcribe_command`,
      `transcribe_and_execute_command` into `app_core/listening_tools.rs`.
- [ ] Add `mod listening_tools;` in `app_core/mod.rs`.
- [ ] Run full validation gate.

---

## Phase 7 — Split `commands/validators.rs` (~808 lines)

`validators.rs` has a 116-line master validator and a large dispatch function that routes
per-tool argument validation to helper functions by tool name — these helpers fall into five
natural groups.

### Target layout

```
src-tauri/src/commands/validators/
  mod.rs        # validate_planner_output, validate_submit_confirmation_policy,
                #   validate_confirmation_policy, validate_tool_arguments() dispatch (~200 lines)
  navigation.rs # validate_open_url_args, validate_go_back_args, validate_go_forward_args,
                #   validate_reload_page_args, validate_eval_js_args, validate_scroll_page_args
  element.rs    # validate_find_element_args, validate_click_element_args,
                #   validate_focus_element_args, validate_type_into_element_args,
                #   validate_submit_active_form_args
  extraction.rs # validate_get_html_args, validate_capture_screenshot_args,
                #   validate_run_ocr_args, validate_get_page_snapshot_args,
                #   validate_extract_page_model_args, validate_merge_ocr_args
  audio.rs      # validate_set_playback_volume_args, validate_set_playback_speed_args,
                #   validate_set_tts_voice_args, validate_set_browser_visibility_args
  voice.rs      # validate_start_listening_args, validate_stop_listening_args,
                #   validate_transcribe_command_args, validate_transcribe_and_execute_args
  planner.rs    # validate_confirm_action_args, validate_report_result_args,
                #   step-transition validators
```

### 7.1 Create `validators/` directory and `mod.rs`

- [ ] Create `src-tauri/src/commands/validators/` directory.
- [ ] Move `commands/validators.rs` → `commands/validators/mod.rs`.
- [ ] Run `cargo check`.

### 7.2 Extract `navigation.rs`

- [ ] Move URL, scroll, reload, eval_js, and history-navigation argument validators into
      `validators/navigation.rs`.
- [ ] Add `mod navigation;` in `validators/mod.rs` and call through from the dispatch function.
- [ ] Run `cargo check`.

### 7.3 Extract `element.rs`

- [ ] Move element interaction validators (find, click, focus, type, submit) into
      `validators/element.rs`.
- [ ] Run `cargo check`.

### 7.4 Extract `extraction.rs`

- [ ] Move OCR, screenshot, and page-model validators into `validators/extraction.rs`.
- [ ] Run `cargo check`.

### 7.5 Extract `audio.rs`

- [ ] Move playback and browser-visibility validators into `validators/audio.rs`.
- [ ] Run `cargo check`.

### 7.6 Extract `voice.rs`

- [ ] Move listening and transcription validators into `validators/voice.rs`.
- [ ] Run `cargo check`.

### 7.7 Extract `planner.rs`

- [ ] Move confirmation-action, report-result, and step-transition validators into
      `validators/planner.rs`.
- [ ] Run full validation gate.

---

## Phase 8 — Split `commands/registry.rs` (~1054 lines)

`registry.rs` combines tool registration, JSON schema generation, example output generation,
skill file discovery, and skill parsing/scoring.

### Target layout

```
src-tauri/src/commands/
  registry.rs     # registered_tools(), planner_available_tools(), is_plannable_tool(),
                  #   build_planner_skill_selection() — the core registry surface (~200 lines)
  schemas.rs      # planner_output_schema(), canonical_planner_output_examples(),
                  #   tool_input_schema(), tool_output_schema()
  skill_parser.rs # parse_skill_document(), parse_skill_frontmatter(),
                  #   parse_bundled_skills(), skill_summary_from_frontmatter(),
                  #   score_skill(), and all parsing/scoring helper functions
  skill_loader.rs # discover_skills(), load_skills_from_directory(), filesystem helpers
```

### 8.1 Extract `schemas.rs`

- [ ] Move `planner_output_schema()`, `canonical_planner_output_examples()`,
      `tool_input_schema()`, `tool_output_schema()` into `commands/schemas.rs`.
- [ ] Import in `registry.rs` where needed.
- [ ] Run `cargo check`.

### 8.2 Extract `skill_parser.rs`

- [ ] Move `parse_skill_document()`, `parse_skill_frontmatter()`, `parse_bundled_skills()`,
      `skill_summary_from_frontmatter()`, `score_skill()`, and all their helper functions
      into `commands/skill_parser.rs`.
- [ ] Update `registry.rs` to import from `skill_parser`.
- [ ] Run `cargo check`.

### 8.3 Extract `skill_loader.rs`

- [ ] Move `discover_skills()`, `load_skills_from_directory()`, and filesystem/path helpers
      into `commands/skill_loader.rs`.
- [ ] Update `registry.rs` to import from `skill_loader`.
- [ ] Run full validation gate.

---

## Phase 9 — Split `lib.rs` (~1073 lines)

`lib.rs` is the Tauri entry point (`run()` + `invoke_handler!()`) and a collection of ~40
`#[tauri::command]` handler functions. The handlers can move to a `command_handlers/`
submodule; `lib.rs` imports them into scope for `tauri::generate_handler![]`.

> **Note:** Keep `#[tauri::command]` on every moved function. The attribute must be on the
> definition, not the import site.

### Target layout

```
src-tauri/src/
  lib.rs                    # module declarations, lock_app_core(), run() + generate_handler![]
                            #   (~120 lines after extraction)
  command_handlers/
    mod.rs                  # pub(crate) mod declarations; no handler functions
    core_handlers.rs        # execute_planner_output, resolve_command,
                            #   submit_confirmation_response, get_agent_state
    voice_handlers.rs       # start_listening, stop_listening, transcribe_command,
                            #   transcribe_and_execute_command
    url_handlers.rs         # open_url, validate_external_url, launch_external_url,
                            #   open_external_url
    audio_handlers.rs       # set_playback_volume, set_playback_speed,
                            #   set_browser_visibility, set_tts_voice
    provider_handlers.rs    # set_asr_provider_selection, set_tts_provider_selection,
                            #   set_tts_model_selection, set_remote_planner_connection,
                            #   reset_remote_planner_connection_to_defaults
    safety_handlers.rs      # set_confirmation_threshold,
                            #   set_allow_click_without_confirmation, set_ocr_thresholds
    api_key_handlers.rs     # set/test_remote_planner_api_key, list_remote_planner_models,
                            #   set/test_remote_tts_api_key, set/test_remote_asr_api_key
    model_handlers.rs       # get_model_management_settings, set_model_management_settings,
                            #   download_active_local_tts_model,
                            #   download_active_local_asr_model
```

### 9.1 Create `command_handlers/` directory and `mod.rs`

- [ ] Create `src-tauri/src/command_handlers/` directory.
- [ ] Create `command_handlers/mod.rs` with `pub(crate) mod` declarations for each submodule.
- [ ] Add `mod command_handlers;` and `use command_handlers::*::*;` imports in `lib.rs`.
- [ ] Run `cargo check`.

### 9.2 Extract `core_handlers.rs`

- [ ] Move `execute_planner_output`, `resolve_command`, `submit_confirmation_response`,
      `get_agent_state` (and the shared `lock_app_core` helper if not used elsewhere)
      into `command_handlers/core_handlers.rs`, keeping `#[tauri::command]` on each.
- [ ] Run `cargo check`.

### 9.3 Extract `voice_handlers.rs`

- [ ] Move `start_listening`, `stop_listening`, `transcribe_command`,
      `transcribe_and_execute_command` into `command_handlers/voice_handlers.rs`.
- [ ] Run `cargo check`.

### 9.4 Extract `url_handlers.rs`

- [ ] Move `open_url`, `validate_external_url`, `launch_external_url`, `open_external_url`
      into `command_handlers/url_handlers.rs`.
- [ ] Run `cargo check`.

### 9.5 Extract `audio_handlers.rs`

- [ ] Move `set_playback_volume`, `set_playback_speed`, `set_browser_visibility`,
      `set_tts_voice` into `command_handlers/audio_handlers.rs`.
- [ ] Run `cargo check`.

### 9.6 Extract `provider_handlers.rs`

- [ ] Move `set_asr_provider_selection`, `set_tts_provider_selection`,
      `set_tts_model_selection`, `set_remote_planner_connection`,
      `reset_remote_planner_connection_to_defaults` into `command_handlers/provider_handlers.rs`.
- [ ] Run `cargo check`.

### 9.7 Extract `safety_handlers.rs`

- [ ] Move `set_confirmation_threshold`, `set_allow_click_without_confirmation`,
      `set_ocr_thresholds` into `command_handlers/safety_handlers.rs`.
- [ ] Run `cargo check`.

### 9.8 Extract `api_key_handlers.rs`

- [ ] Move all remote API key set/test handlers and `list_remote_planner_models` into
      `command_handlers/api_key_handlers.rs`.
- [ ] Run `cargo check`.

### 9.9 Extract `model_handlers.rs`

- [ ] Move `get_model_management_settings`, `set_model_management_settings`,
      `download_active_local_tts_model`, `download_active_local_asr_model` into
      `command_handlers/model_handlers.rs`.
- [ ] Run full validation gate.

---

## Phase 10 — Split `commands/routing/mod.rs` (~1061 lines)

`routing/mod.rs` contains all `resolve_direct_*` dispatcher functions inline. Each function
handles a distinct command category and should move to a dedicated file, parallel to the
already-extracted `audio_commands.rs`, `url_commands.rs`, `intent.rs`, etc.

### Target layout

```
src-tauri/src/commands/routing/
  mod.rs                # resolve_command_from_transcript() entry point and routing dispatch
                        #   only (~80 lines target)
  # existing submodules — unchanged:
  intent.rs
  audio_commands.rs     # extend: add resolve_direct_audio_command(),
                        #   resolve_direct_browser_visibility_command()
  url_commands.rs       # extend: add resolve_direct_open_url_command()
  status_commands.rs    # extend: add resolve_direct_status_query_command(),
                        #   resolve_direct_read_title_command(), resolve_direct_repeat_command()
  planner_outputs.rs    # unchanged
  # new submodules:
  navigation_routing.rs # resolve_direct_navigation_readback_command()
  voice_routing.rs      # resolve_direct_voice_input_command()
  reading_routing.rs    # resolve_direct_read_page_command()
  field_routing.rs      # parse_direct_focus_field_command(), parse_direct_fill_field_command(),
                        #   parse_direct_fill_and_submit_command(),
                        #   is_direct_submit_form_command()
```

### 10.1 Move audio/visibility routing into `audio_commands.rs`

- [ ] Move `resolve_direct_audio_command()` and
      `resolve_direct_browser_visibility_command()` into the existing
      `routing/audio_commands.rs`.
- [ ] Update `mod.rs` to call through.
- [ ] Run `cargo check`.

### 10.2 Move URL routing into `url_commands.rs`

- [ ] Move `resolve_direct_open_url_command()` into the existing `routing/url_commands.rs`.
- [ ] Run `cargo check`.

### 10.3 Move status/query routing into `status_commands.rs`

- [ ] Move `resolve_direct_status_query_command()`, `resolve_direct_read_title_command()`,
      `resolve_direct_repeat_command()` into the existing `routing/status_commands.rs`.
- [ ] Run `cargo check`.

### 10.4 Extract `navigation_routing.rs`

- [ ] Move `resolve_direct_navigation_readback_command()` into
      `routing/navigation_routing.rs`.
- [ ] Add `mod navigation_routing;` in `routing/mod.rs`.
- [ ] Run `cargo check`.

### 10.5 Extract `voice_routing.rs`

- [ ] Move `resolve_direct_voice_input_command()` into `routing/voice_routing.rs`.
- [ ] Add `mod voice_routing;` in `routing/mod.rs`.
- [ ] Run `cargo check`.

### 10.6 Extract `reading_routing.rs`

- [ ] Move `resolve_direct_read_page_command()` into `routing/reading_routing.rs`.
- [ ] Add `mod reading_routing;` in `routing/mod.rs`.
- [ ] Run `cargo check`.

### 10.7 Extract `field_routing.rs`

- [ ] Move `parse_direct_focus_field_command()`, `parse_direct_fill_field_command()`,
      `parse_direct_fill_and_submit_command()`, `is_direct_submit_form_command()` into
      `routing/field_routing.rs`.
- [ ] Add `mod field_routing;` in `routing/mod.rs`.
- [ ] Run full validation gate.

---

## Phase 11 — Split `src/tauri-api.ts` (~999 lines)

`tauri-api.ts` mixes ~512 lines of type definitions with ~487 lines of API function
implementations. Separating types lets callers import interfaces without pulling the full
API surface. Keep `tauri-api.ts` as a barrel re-export so no existing callers change.

### Target layout

```
src/
  tauri-api.ts        # barrel: export * from "./tauri-types"; export * from "./api/*"
  tauri-types.ts      # all type aliases, interface definitions, and const enums (~512 lines):
                      #   ProviderMode, BrowserVisibilityMode, ToolName, IntentName,
                      #   AgentStateData, ProviderSelectionStatus, all settings interfaces,
                      #   ToolError, PlannerOutput, ConfirmActionData, etc.
  api/
    errors.ts         # classifyInvokeFailure, parseToolError, unwrapToolResult,
                      #   isRecord, InvokeFailure type, invokeCommand() helper
    planner.ts        # executePlannerOutput, resolveCommand, submitConfirmationResponse
    voice.ts          # startListening, stopListening, transcribeCommand,
                      #   transcribeAndExecuteCommand
    navigation.ts     # openUrl, openExternalUrl
    audio.ts          # setPlaybackVolume, setPlaybackSpeed, setBrowserVisibility, setTtsVoice
    providers.ts      # setAsrProviderSelection, setTtsProviderSelection,
                      #   setTtsModelSelection, setRemotePlannerConnection,
                      #   resetRemotePlannerConnectionToDefaults
    safety.ts         # setConfirmationThreshold, setAllowClickWithoutConfirmation,
                      #   setOcrThresholds
    remote-keys.ts    # set/test remote API key functions for planner, TTS, ASR;
                      #   listRemotePlannerModels
    models.ts         # getModelManagementSettings, setModelManagementSettings,
                      #   downloadActiveLocal*Model functions
```

### 11.1 Extract `tauri-types.ts`

- [ ] Move all `type`, `interface`, and `enum` declarations (no function bodies) from
      `tauri-api.ts` into `src/tauri-types.ts`.
- [ ] Add `export * from "./tauri-types";` in `tauri-api.ts`.
- [ ] Run `pnpm lint && pnpm test:ui && pnpm build`.

### 11.2 Create `src/api/` and extract `errors.ts`

- [ ] Create `src/api/` directory.
- [ ] Move `classifyInvokeFailure`, `parseToolError`, `unwrapToolResult`, `isRecord`,
      `InvokeFailure`, and the `invokeCommand` helper into `src/api/errors.ts`.
- [ ] Re-export from `tauri-api.ts`.
- [ ] Run validation.

### 11.3 Extract `api/planner.ts`

- [ ] Move `executePlannerOutput`, `resolveCommand`, `submitConfirmationResponse` into
      `src/api/planner.ts`.
- [ ] Re-export from `tauri-api.ts`.
- [ ] Run validation.

### 11.4 Extract `api/voice.ts`

- [ ] Move `startListening`, `stopListening`, `transcribeCommand`,
      `transcribeAndExecuteCommand` into `src/api/voice.ts`.
- [ ] Re-export from `tauri-api.ts`.
- [ ] Run validation.

### 11.5 Extract `api/audio.ts`

- [ ] Move `setPlaybackVolume`, `setPlaybackSpeed`, `setBrowserVisibility`, `setTtsVoice`
      into `src/api/audio.ts`.
- [ ] Re-export from `tauri-api.ts`.
- [ ] Run validation.

### 11.6 Extract `api/navigation.ts`

- [ ] Move `openUrl`, `openExternalUrl` into `src/api/navigation.ts`.
- [ ] Re-export from `tauri-api.ts`.
- [ ] Run validation.

### 11.7 Extract `api/providers.ts`

- [ ] Move all provider selection functions into `src/api/providers.ts`.
- [ ] Re-export from `tauri-api.ts`.
- [ ] Run validation.

### 11.8 Extract `api/safety.ts`

- [ ] Move `setConfirmationThreshold`, `setAllowClickWithoutConfirmation`, `setOcrThresholds`
      into `src/api/safety.ts`.
- [ ] Re-export from `tauri-api.ts`.
- [ ] Run validation.

### 11.9 Extract `api/remote-keys.ts`

- [ ] Move all remote API key set/test functions into `src/api/remote-keys.ts`.
- [ ] Re-export from `tauri-api.ts`.
- [ ] Run validation.

### 11.10 Extract `api/models.ts`

- [ ] Move model management functions into `src/api/models.ts`.
- [ ] Re-export from `tauri-api.ts`.
- [ ] Run full validation gate.

---

## Phase 12 — Split `src/app-shell.ts` (~542 lines)

`app-shell.ts` mixes Material-UI theme definition, navigation component rendering, full shell
markup, SSR rendering, and panel control state management.

### Target layout

```
src/
  app-shell.ts            # AppShellMarkup component, renderShellTree(), AppShellRuntime(),
                          #   renderAppShell() — shell tree assembly only (~200 lines)
  app-shell-theme.ts      # appShellTheme (MUI theme object), any color/font constants
  app-shell-nav.ts        # renderAppViewActionButton, renderSettingsSubpageBackButton,
                          #   renderSettingsSubpageLink, renderPanelRootPlaceholderElement,
                          #   renderPanelContent, and their prop types
  app-shell-controls.ts   # captureActivePanelControl, restoreActivePanelControl,
                          #   preserveActivePanelControl, and the ActivePanelControl type
```

### 12.1 Extract `app-shell-theme.ts`

- [ ] Move `appShellTheme` (Material-UI `createTheme(...)` call) and any supporting color
      or font constants into `src/app-shell-theme.ts`.
- [ ] Import in `app-shell.ts`.
- [ ] Run `pnpm lint && pnpm test:ui && pnpm build`.

### 12.2 Extract `app-shell-nav.ts`

- [ ] Move `renderAppViewActionButton`, `renderSettingsSubpageBackButton`,
      `renderSettingsSubpageLink`, `renderPanelRootPlaceholderElement`,
      `renderPanelContent`, and any supporting prop types into `src/app-shell-nav.ts`.
- [ ] Import in `app-shell.ts`.
- [ ] Run validation.

### 12.3 Extract `app-shell-controls.ts`

- [ ] Move `captureActivePanelControl`, `restoreActivePanelControl`,
      `preserveActivePanelControl`, and the `ActivePanelControl` type into
      `src/app-shell-controls.ts`.
- [ ] Import in `app-shell.ts`.
- [ ] Run full validation gate.

---

## Phase 13 — Final validation and documentation

### 13.1 Run the full validation suite

- [ ] `source ./fix-node-version.sh`
- [ ] `pnpm lint`
- [ ] `pnpm test:ui`
- [ ] `pnpm build`
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml --all-features`
- [ ] Fix any lint errors or test failures before committing.

### 13.2 Verify file sizes

- [ ] Run `find . -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.mjs" \) -not -path "*/node_modules/*" -not -path "*/dist/*" -not -path "*/target/*" | xargs wc -l | sort -rn | head -30`.
- [ ] Target: no production source file over 600 lines. Fixture-heavy test files are exempt
      if content is genuinely cohesive.

### 13.3 Update memory.md

- [ ] Run `date -u +"%Y-%m-%dT%H:%M:%SZ"` and add an entry to `memory.md` summarizing
      completed phases, final commit hash, and validation status.

---

## Suggested commit sequence

```
Commit 1:  Phase 1  — split app_core/mod.rs (runtime_config, narration, remote_planner, state_snapshots)
Commit 2:  Phase 2  — split browser.rs into browser/ (config, session, navigation, page_metrics, dom_extraction)
Commit 3:  Phase 3  — split tts.rs into tts/ (local, remote, wav)
Commit 4:  Phase 4  — split asr.rs into asr/ (capture, processing, local, remote, wav)
Commit 5:  Phase 5  — split commands/contracts.rs into contracts/ (providers, planner, interaction, tools)
Commit 6:  Phase 6  — split large app_core submodules (element_scoring, ocr_merge, page_model_builder, fill_correction, reading_tools, listening_tools)
Commit 7:  Phase 7  — split commands/validators.rs into validators/ (navigation, element, extraction, audio, voice, planner)
Commit 8:  Phase 8  — split commands/registry.rs (schemas, skill_parser, skill_loader)
Commit 9:  Phase 9  — split lib.rs into command_handlers/ (core, voice, url, audio, provider, safety, api_key, model)
Commit 10: Phase 10 — split commands/routing/mod.rs (audio, url, status, navigation, voice, reading, field routing)
Commit 11: Phase 11 — split src/tauri-api.ts (tauri-types.ts + api/ subdirectory)
Commit 12: Phase 12 — split src/app-shell.ts (theme, nav, controls)
```
