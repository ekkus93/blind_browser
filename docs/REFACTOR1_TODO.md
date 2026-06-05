# Refactor TODO — blind_browser

Based on a structural analysis of the codebase in June 2026.  
**Status key:** PENDING · IN PROGRESS · DONE

---

## Goal

The largest files have grown far past the point where a reader can hold them in their head.
`app_core.rs` at ~11 600 lines is doing at least eight distinct jobs. `main.ts` at ~1 970 lines
wires all panel state, handlers, and planner orchestration in one place.
`commands/tests.rs` at ~8 800 lines is a single flat test file with no grouping.
`config.rs` at ~2 500 lines mixes type definitions, loading, validation, and keyring I/O.

These refactors are **structural, not behavioral** — every one must leave the public API and
observable behavior identical. Run the full validation gate after each phase.

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

Work in this order to minimise merge conflicts and keep the codebase green at every step:

1. Split `app_core.rs` — biggest file, most critical, drives all other backend complexity.
2. Split `commands/tests.rs` — second largest, already self-contained; split mirrors Phase 1.
3. Split `config.rs` — isolates types from loading/validation; unblocks future config work.
4. Split `commands/routing.rs` — thin dispatch layer; easy to fan out once config is clean.
5. Split `src/main.ts` — frontend; split into handler modules, reducing the God component.

---

## Phase 1 — Split `src-tauri/src/app_core.rs` (~11 600 lines)

`app_core.rs` currently contains: the `AppCore` struct and its ~80 methods, the replanning
loop, deterministic tool implementations, settings/config adapters, model download logic,
element search and scoring, form-fill resolution, the planner system prompt, and an inline
test module. Each of these belongs in its own file.

### Target module layout

```
src-tauri/src/
  app_core/
    mod.rs              # AppCore struct, impl new(), impl ReplanningRuntime, re-exports
    replanning.rs       # execute_bounded_replanning_loop + supporting types
    settings_adapters.rs # build_* functions that convert AppConfig → settings structs
    model_management.rs # download_*, kitten/whisper download plans, local model availability
    tool_executor.rs    # impl DeterministicToolExecutor for AppCore
    navigation_tools.rs # execute_open_url, go_back, go_forward, reload_page
    content_tools.rs    # execute_get_html, eval_js, scroll_page, capture_screenshot
    extraction_tools.rs # execute_run_ocr, merge_ocr, get_page_snapshot, extract_page_model
    interaction_tools.rs # find/click/focus/type/submit + element scoring helpers
    form_fill.rs        # resolve_direct_fill_* + resolve_recent_fill_correction_command
    voice_tools.rs      # execute_read_region, read_next/prev, stop_speaking, start/stop_listening, transcribe
    planner_prompt.rs   # planner_system_prompt(), page/text extraction helpers
    api_key_tools.rs    # test_remote_*_api_key, fetch_openai_compatible_models, openai error helpers
    remote_api.rs       # HTTP client helpers for OpenAI-compatible endpoints
```

### 1.1 Create `app_core/` directory and `mod.rs` — DONE

- [x] Create `src-tauri/src/app_core/` directory.
- [x] Move `src-tauri/src/app_core.rs` → `src-tauri/src/app_core/mod.rs` as a starting point.
- [x] Update `src-tauri/src/lib.rs`: change `mod app_core;` to `mod app_core;` (Rust resolves
      `app_core/mod.rs` automatically — no change needed, but verify it compiles).
- [x] Run `cargo check` to confirm nothing broke before splitting.

### 1.2 Extract `replanning.rs` (lines ~212–343) — DONE

- [x] Move `execute_bounded_replanning_loop`, `execution_trace_to_tool_history_entries`,
      `append_execution_trace`, `merge_execution_outcome_trace`, `replanning_request_id`,
      and the `ReplanningRuntime` trait + its impl block into `app_core/replanning.rs`.
- [x] In `mod.rs`, add `mod replanning; use replanning::execute_bounded_replanning_loop;` and remove extracted items.
- [x] Run `cargo check`.

### 1.3 Extract `settings_adapters.rs` (lines ~4 798–5 233)

- [ ] Move all `build_*` free functions that convert `&AppConfig` into settings structs
      (`build_tts_model_settings`, `build_local_tts_model_settings`, `build_tts_provider_settings`,
      `build_remote_planner_settings`, `build_remote_tts_settings`, `build_remote_asr_settings`,
      `build_confirmation_settings`, `build_ocr_threshold_settings`, `build_asr_provider_settings`,
      `build_local_asr_model_settings`, `build_model_management_settings`,
      `build_tts_voice_settings`, `active_local_tts_profile`, `active_local_asr_profile`,
      `resolved_models_dir_for_app`, `remote_provider_label`, `masked_secret_value`) into
      `app_core/settings_adapters.rs`.
- [ ] Update `mod.rs` to `mod settings_adapters; use settings_adapters::*;`.
- [ ] Run `cargo check`.

### 1.4 Extract `model_management.rs` (lines ~5 285–5 513)

- [ ] Move `local_tts_model_is_available`, `local_asr_model_is_available`,
      `KittenDownloadPlan`, `kitten_download_plan_for_model_id`, `WhisperDownloadPlan`,
      `whisper_download_plan_for_model_id`, `download_hugging_face_directory`,
      `download_hugging_face_file` into `app_core/model_management.rs`.
- [ ] Move the `AppCore` methods `download_active_local_tts_model` and
      `download_active_local_asr_model` into an `impl AppCore` block in this file,
      or keep them in `mod.rs` with the helpers in scope via `use`.
- [ ] Update `mod.rs`.
- [ ] Run `cargo check`.

### 1.5 Extract `api_key_tools.rs` (lines ~4 994–5 170)

- [ ] Move `test_remote_openai_profile_api_key`, `test_openai_api_key_connectivity`,
      `OpenAiCompatibleModelsResponse`, `OpenAiCompatibleModelEntry`,
      `fetch_openai_compatible_models`, `openai_api_key_test_failure_message`
      into `app_core/api_key_tools.rs`.
- [ ] The `AppCore` methods `test_remote_planner_api_key`, `list_remote_planner_models`,
      `test_remote_tts_api_key`, `test_remote_asr_api_key` may remain in `mod.rs` calling
      into `api_key_tools`.
- [ ] Update `mod.rs`.
- [ ] Run `cargo check`.

### 1.6 Extract navigation tool methods into `navigation_tools.rs`

- [ ] Create `app_core/navigation_tools.rs` with a second `impl AppCore` block.
- [ ] Move `execute_open_url`, `execute_go_back`, `execute_go_forward`, `execute_reload_page`,
      `refresh_current_page_after_navigation`, `clear_navigation_follow_up_state`,
      `normalize_absolute_url`, `browser_error_to_tool_error` into this file.
- [ ] Add `mod navigation_tools;` to `mod.rs`.
- [ ] Run `cargo check`.

### 1.7 Extract content tool methods into `content_tools.rs`

- [ ] Create `app_core/content_tools.rs`.
- [ ] Move `execute_get_html`, `execute_eval_js`, `execute_scroll_page`,
      `execute_capture_screenshot` into a second `impl AppCore` block here.
- [ ] Run `cargo check`.

### 1.8 Extract OCR/extraction tools into `extraction_tools.rs`

- [ ] Create `app_core/extraction_tools.rs`.
- [ ] Move `execute_run_ocr`, `execute_merge_ocr_into_page_model`, `execute_get_page_snapshot`,
      `execute_extract_page_model`, `merge_ocr_text_into_page_model`, `extracted_text_metrics`,
      `has_positive_bbox`, `region_first_ocr_target_ids`,
      `should_trigger_extract_page_model_ocr_fallback`, `merged_region_text`,
      `ocr_runtime_error_to_tool_error`, `extract_page_model_internal_failure`,
      `nested_tool_failure_as_extract_page_model`, `build_extracted_page_model`,
      `build_visible_text_excerpt`, `infer_extraction_source` here.
- [ ] Run `cargo check`.

### 1.9 Extract element interaction tools into `interaction_tools.rs`

- [ ] Create `app_core/interaction_tools.rs`.
- [ ] Move `execute_list_interactive_elements`, `execute_find_element`,
      `execute_click_element`, `execute_focus_element`, `execute_type_into_element`,
      `execute_submit_active_form`, all `resolve_clickable_element`,
      `resolve_typeable_element`, `resolve_form_element` helpers, and the full element
      scoring subsystem (`FindElementScore`, `AttributeHintSpec`, `score_interactive_element`,
      `score_text_query_against_element`, `score_attribute_hint`, `text_overlap_score`,
      `rank_find_element_candidates`, `determine_find_element_resolution`,
      `build_find_element_query`, `FindElementQuery`, `filter_interactive_elements`,
      `focusable_field_elements`, `submittable_form_elements`, `summarize_candidate_names`,
      `summarize_form_candidate_names`, `describe_field_element`, `describe_form_element`,
      `region_bbox_by_id`, `normalize_optional_text`, `normalize_search_text`,
      `tokenize_search_text`) here.
- [ ] Run `cargo check`.

### 1.10 Extract form-fill resolution into `form_fill.rs`

- [ ] Create `app_core/form_fill.rs`.
- [ ] Move `resolve_direct_focus_field_command`, `resolve_direct_fill_command_internal`,
      `resolve_direct_fill_field_command`, `resolve_direct_fill_and_submit_command`,
      `resolve_direct_submit_form_command`, `resolve_recent_fill_correction_command`,
      `selected_skills_for_fill_command`, `build_direct_fill_ready_output`,
      `build_direct_fill_and_submit_ready_output`, `build_direct_follow_up_output`,
      `DirectFollowUpSpec`, `PendingRecentFieldContext`, `RecentFieldContext`,
      `ResolvedDirectFieldCommand`, `normalize_fill_value`, `normalize_field_target`,
      `strip_fill_and_submit_suffix`, `parse_fill_with_pattern`, `parse_into_field_pattern`,
      `parse_fill_field_description_only`, `split_case_insensitive_once`,
      `collapse_transcript_whitespace` here.
- [ ] Run `cargo check`.

### 1.11 Extract voice tools into `voice_tools.rs`

- [ ] Create `app_core/voice_tools.rs`.
- [ ] Move `execute_read_region`, `execute_read_next_region`, `execute_read_previous_region`,
      `execute_stop_speaking`, `execute_start_listening`, `execute_stop_listening`,
      `execute_transcribe_command`, `transcribe_and_execute_command`,
      `execute_set_tts_voice`, `execute_set_playback_volume`, `execute_set_playback_speed`,
      `execute_set_browser_visibility`,
      `tts_runtime_error_to_tool_error`, `audio_playback_error_to_tool_error`,
      `asr_runtime_error_to_tool_error`, `first_readable_region_id`,
      `format_playback_volume`, `format_playback_speed` here.
- [ ] Run `cargo check`.

### 1.12 Extract planner prompt into `planner_prompt.rs`

- [ ] Create `app_core/planner_prompt.rs`.
- [ ] Move `planner_system_prompt`, `PlannerPromptPayload`, `planner_interpretation_unavailable_error`,
      `planner_available_tools_include_all_wave_two_tools` (if not in tests),
      `build_single_step_planner_output`, `build_audio_set_planner_output`,
      `build_audio_report_planner_output`, `build_browser_visibility_planner_output`,
      `build_status_query_planner_output`, `build_report_result_step`,
      `format_*` summary helpers, `current_page_label*` helpers here.
- [ ] Run `cargo check`.

### 1.13 Extract `impl DeterministicToolExecutor` into `tool_executor.rs`

- [ ] Create `app_core/tool_executor.rs`.
- [ ] Move the `impl DeterministicToolExecutor for AppCore` block (lines ~5 585–5 769)
      into this file.
- [ ] Run `cargo check`.

### 1.14 Move inline tests out of `mod.rs`

- [ ] The inline `mod tests` block at the bottom of `app_core.rs` (line ~8 025) should
      move to `app_core/tests.rs` (or be deleted in favour of the unified test file in
      Phase 2).
- [ ] Add `#[cfg(test)] mod tests;` in `mod.rs` pointing to the new file.
- [ ] Run full validation gate.

---

## Phase 2 — Split `src-tauri/src/commands/tests.rs` (~8 800 lines)

`tests.rs` is a single flat `mod tests` with ~170 test functions and ~400 lines of shared
fixtures and helpers. It should be split into focused sub-modules mirroring the tool groups
in Phase 1.

### Target layout

```
src-tauri/src/commands/tests/
  mod.rs              # shared fixtures, MockExecutor, helper fns — no test fns
  fixtures.rs         # fixture_agent_state, fixture_page_model_*, fixture_runtime_status
  tool_dispatch.rs    # dispatches_* tests for every registered tool
  playback_controls.rs # set_playback_volume/speed clamp + readback tests
  browser_state.rs    # set_browser_visibility, browser_history_navigation, visibility changes
  runtime_status.rs   # get_runtime_status* tests, provider_selection round-trips
  listening.rs        # listening_tools*, transcribe_command* tests
  planner_flow.rs     # executes_*, returns_*, resumes_*, aborts_*, rejects_* flow tests
  skill_selection.rs  # build_planner_skill_selection*, discover_skills*, parse_skill_document*
  confirmation.rs     # dispatches_confirm_action*, resumes_*_confirmation*, rejects_resume_*
```

### 2.1 Create `tests/` directory and extract shared infrastructure

- [ ] Create `src-tauri/src/commands/tests/`.
- [ ] Move `tests.rs` → `commands/tests/mod.rs` as a starting point.
- [ ] Change `mod tests;` in `commands/mod.rs` (or wherever it's declared) to resolve the
      new path. Rust auto-resolves `tests/mod.rs`.
- [ ] Run `cargo test` to confirm green.

### 2.2 Extract `fixtures.rs`

- [ ] Move `fixture_agent_state`, `fixture_runtime_status`, `fixture_page_model_without_regions`,
      `fixture_agent_state_for_page`, `fixture_problematic_article_page_without_regions`,
      `fixture_problematic_docs_agent_state`, `sample_planned_step`,
      `sample_planned_steps_for_registered_tools`, `MockExecutor` and its `impl` blocks,
      `PlannerSkillFixtureResolver`, `PlannerSkillFixture`, `resolve_planner_skill_fixture`,
      `assert_planner_skill_fixture`, and `unique_temp_path`, `write_skill_document`
      into `tests/fixtures.rs`.
- [ ] Add `mod fixtures; use fixtures::*;` in `tests/mod.rs`.
- [ ] Run `cargo test`.

### 2.3 Extract `tool_dispatch.rs`

- [ ] Move all `dispatches_*_from_planned_step` tests and `rejects_invalid_tool_arguments*`,
      `validate_planned_step_arguments*` into `tests/tool_dispatch.rs`.
- [ ] Add `mod tool_dispatch;` in `tests/mod.rs`.
- [ ] Run `cargo test`.

### 2.4 Extract `playback_controls.rs`

- [ ] Move `set_playback_volume_clamps_*`, `set_playback_speed_clamps_*` into
      `tests/playback_controls.rs`.
- [ ] Run `cargo test`.

### 2.5 Extract `browser_state.rs`

- [ ] Move `set_browser_visibility_reports_*`, `browser_visibility_changes_*`,
      `browser_history_navigation_*` into `tests/browser_state.rs`.
- [ ] Run `cargo test`.

### 2.6 Extract `runtime_status.rs`

- [ ] Move `get_runtime_status_*`, `provider_selection_status_*`, `shared_command_enums_*`
      into `tests/runtime_status.rs`.
- [ ] Run `cargo test`.

### 2.7 Extract `listening.rs`

- [ ] Move `listening_tools_*`, `transcribe_command_*` into `tests/listening.rs`.
- [ ] Run `cargo test`.

### 2.8 Extract `planner_flow.rs`

- [ ] Move `executes_next_step_chain_*`, `executes_load_page_extract_*`,
      `executes_resolved_spoken_*`, `follows_failure_transition_*`,
      `aborts_when_*`, `aborts_needs_confirmation_*`,
      `planner_available_tools_*` into `tests/planner_flow.rs`.
- [ ] Run `cargo test`.

### 2.9 Extract `skill_selection.rs`

- [ ] Move `build_planner_skill_selection_*`, `discover_skills_*`, `parse_skill_document_*`
      into `tests/skill_selection.rs`.
- [ ] Run `cargo test`.

### 2.10 Extract `confirmation.rs`

- [ ] Move `returns_awaiting_confirmation_*`, `resumes_confirmed_*`,
      `resumes_rejected_*`, `rejects_resume_*`, `dispatches_confirm_action_*`,
      `dispatches_report_result_*`, `dispatches_get_agent_state_*`
      into `tests/confirmation.rs`.
- [ ] Run `cargo test`.

### 2.11 Final: confirm `tests/mod.rs` contains only `mod` declarations and shared infra

- [ ] `tests/mod.rs` should contain no `#[test]` functions — only `mod` declarations,
      shared imports, and infra that every sub-module needs.
- [ ] Run full validation gate.

---

## Phase 3 — Split `src-tauri/src/config.rs` (~2 500 lines)

`config.rs` currently contains type definitions, TOML loading logic, profile resolution,
validation, keyring I/O, and a large inline test module with embedded fixture strings.

### Target layout

```
src-tauri/src/config/
  mod.rs          # AppConfig struct, impl AppConfig, ConfigError, SecretRef, re-exports
  types.rs        # all plain data types (profiles, enums, settings structs)
  loading.rs      # load_document_table_from_path/str, load_planner_profiles, load_provider_profiles, resolve_profile
  validation.rs   # validate_audio_settings, validate_safety_settings, validate_ocr_settings, validate_model_settings, normalize_remote_endpoint
  keyring.rs      # keyring_ref_for_remote_api_key, secret_ref_reference, resolve_secret_ref, session_keyring_store, set/get_keyring_secret, cache_keyring_secret
  tests/          # (optional) split test fixtures into per-concern files
    mod.rs
    load_tests.rs
    validation_tests.rs
    keyring_tests.rs
```

### 3.1 Create `config/` directory and `mod.rs`

- [ ] Create `src-tauri/src/config/` directory.
- [ ] Move `config.rs` → `config/mod.rs`.
- [ ] Update `lib.rs` if needed (Rust resolves `config/mod.rs` automatically).
- [ ] Run `cargo check`.

### 3.2 Extract `types.rs`

- [ ] Move all `pub struct`, `pub enum`, and their simple `impl` blocks that contain only
      `Display`, `Default`, or derivable impls (no I/O, no parsing) into `config/types.rs`.
      This includes: `ConfigError`, `ProviderMode`, `KeyringRef`, `SecretRef`,
      `ProviderSelection`, `ProviderSelections`, `AudioSettings`, `SafetySettings`,
      `ModelManagementSettings`, `SpeechFeedbackStyle`, `SpeechFeedbackSettings`,
      `RemoteProviderKind`, `RemoteTtsAudioFormat`, `LocalTtsBackend`, `LocalAsrBackend`,
      `RemotePlannerProfile`, `RemoteTtsProfile`, `RemoteAsrProfile`, `LocalTtsProfile`,
      `LocalAsrProfile`, `AppConfig`, `RawAppConfig`.
- [ ] Add `mod types; pub use types::*;` in `config/mod.rs`.
- [ ] Run `cargo check`.

### 3.3 Extract `loading.rs`

- [ ] Move `AppConfig::from_path`, `AppConfig::from_str` (or whichever methods drive TOML
      loading), `load_document_table_from_path`, `load_document_table_from_str`,
      `load_planner_profiles`, `load_provider_profiles`, `resolve_profile`
      into `config/loading.rs`.
- [ ] Add `mod loading;` in `config/mod.rs`.
- [ ] Run `cargo check`.

### 3.4 Extract `validation.rs`

- [ ] Move `validate_audio_settings`, `validate_safety_settings`, `validate_ocr_settings`,
      `validate_model_settings`, `normalize_remote_endpoint` into `config/validation.rs`.
- [ ] Add `mod validation;` in `config/mod.rs`.
- [ ] Run `cargo check`.

### 3.5 Extract `keyring.rs`

- [ ] Move `keyring_ref_for_remote_api_key`, `secret_ref_reference`, `resolve_secret_ref`,
      `cache_keyring_secret`, `cached_keyring_secret`, `session_keyring_store`,
      `set_keyring_secret`, `get_keyring_secret` (both cfg-gated variants) into
      `config/keyring.rs`.
- [ ] Add `mod keyring; pub use keyring::*;` in `config/mod.rs`.
- [ ] Run `cargo check`.

### 3.6 Split inline config tests

- [ ] Move the `mod tests` block (line ~1 372) out of `config/mod.rs` into
      `config/tests/mod.rs`, `config/tests/load_tests.rs`, etc., if the test bodies
      are large enough to warrant it. (The fixture TOML strings alone are ~800 lines.)
- [ ] Separate load tests, validation tests, and keyring tests into distinct files.
- [ ] Run `cargo test`.

### 3.7 Final validation

- [ ] Run full validation gate.

---

## Phase 4 — Split `src-tauri/src/commands/routing.rs` (~2 200 lines)

`routing.rs` contains intent parsing (`infer_intent_hint` and ~150 helper functions) and the
Tauri command handler functions. These are conceptually unrelated — intent classification is
a pure NLP layer; Tauri handlers are the API surface.

### Target layout

```
src-tauri/src/commands/
  routing/
    mod.rs            # pub use all; registers command handlers for lib.rs invoke_handler
    intent.rs         # infer_intent_hint + all is_*_phrase, parse_*, normalize_* helpers
    audio_commands.rs # parse_volume_command, parse_speed_command, format_* helpers
    url_commands.rs   # parse_direct_open_url_target, normalize_spoken_url_target, URL helpers
    status_commands.rs # build_status_query_planner_output, format_runtime_status_* helpers
    planner_outputs.rs # build_single_step_planner_output, build_audio_*, build_browser_*
```

### 4.1 Create `routing/` directory and `mod.rs`

- [ ] Create `src-tauri/src/commands/routing/`.
- [ ] Move `routing.rs` → `commands/routing/mod.rs`.
- [ ] Verify `commands/mod.rs` continues to expose the right surface.
- [ ] Run `cargo check`.

### 4.2 Extract `intent.rs`

- [ ] Move `infer_intent_hint` and all `is_*_phrase`, `normalize_*`, `collapse_transcript_whitespace`,
      `merge_compound_command_tokens`, `canonicalize_command_token`,
      `is_unambiguous_fuzzy_keyword_match`, `is_single_edit_or_transposition`,
      `is_single_insertion_or_deletion`, `selected_skill`, `selected_audio_skill`,
      `selected_stop_skill` into `routing/intent.rs`.
- [ ] Run `cargo check`.

### 4.3 Extract `audio_commands.rs`

- [ ] Move `NormalizedAudioSetting`, `parse_volume_command`, `parse_speed_command`,
      `parse_absolute_volume_value`, `parse_absolute_speed_value`, `parse_multiplier_token`,
      `volume_relative_step`, `speed_relative_step`, `volume_step_size`, `speed_step_size`,
      `is_volume_query_phrase`, `is_speed_query_phrase`, `format_playback_volume`,
      `format_playback_speed` into `routing/audio_commands.rs`.
- [ ] Run `cargo check`.

### 4.4 Extract `url_commands.rs`

- [ ] Move `parse_direct_open_url_target`, `normalize_spoken_url_target`,
      `looks_like_host_without_scheme`, `prepend_default_scheme`,
      `is_current_url_query_phrase`, `first_readable_region_id`,
      `format_current_url_summary` into `routing/url_commands.rs`.
- [ ] Run `cargo check`.

### 4.5 Extract `planner_outputs.rs`

- [ ] Move `build_single_step_planner_output`, `build_audio_set_planner_output`,
      `build_audio_report_planner_output`, `build_browser_visibility_planner_output`,
      `build_status_query_planner_output`, `build_report_result_step`,
      `build_browser_visibility_planner_output`, `AudioSetPlanSpec`, `StatusQueryPlanSpec`
      into `routing/planner_outputs.rs`.
- [ ] Run `cargo check`.

### 4.6 Extract `status_commands.rs`

- [ ] Move `format_runtime_status_summary`, `current_page_label*`, `format_back_history_summary`,
      `format_forward_history_summary`, `format_listening_summary`, `format_speaking_summary`,
      `format_browser_mode_summary`, `format_browser_visibility_mode`,
      `is_status_query_phrase`, `is_history_query_phrase`, `is_back_history_query_phrase`,
      `is_forward_history_query_phrase` into `routing/status_commands.rs`.
- [ ] Run full validation gate.

---

## Phase 5 — Split `src/main.ts` (~1 970 lines)

`main.ts` is the React app entry point and currently does everything: it declares the Redux
store adapter, renders the entire React tree, wires all ~35 panel state setters, all ~30
async persist/action functions, the PTT/keyboard event loop, settings views, confirmation
handling, and runtime refresh. Each of these clusters belongs in its own module.

### Target layout

```
src/
  main.ts                 # thin entry: import App, mount to DOM — ~30 lines
  app.tsx (or app.ts)     # BlindBrowserApp component + syncLocalStateFromStore — ~100 lines
  panel-state-setters.ts  # all set*State() functions + createRequestId
  settings-actions.ts     # persistPlaybackVolume/Speed, persistBrowserVisibility,
                          #   persistAsrProvider, persistTtsProvider, persistTtsModel,
                          #   persistTtsVoice, persistConfirmationThreshold,
                          #   persistAllowClickWithoutConfirmation,
                          #   persistOcrThresholds, persistModelManagementSettings
  planner-actions.ts      # persistRemotePlannerApiKey, loadRemotePlannerModels,
                          #   persistRemotePlannerConnection, resetRemotePlannerConnectionToDefaults,
                          #   persistRemoteTtsApiKey, persistRemoteAsrApiKey,
                          #   testConfiguredRemote*ApiKey, downloadManagedLocal*Model
  browser-actions.ts      # openDraftUrl, readCurrentPage, stopCurrentReading,
                          #   readNextRegion, readPreviousRegion
  voice-loop.ts           # beginPushToTalk, cancelPushToTalk, releasePushToTalk,
                          #   ensureContinuousListeningLoop, stopContinuousListeningAfterFailure,
                          #   executeUrlPanelPlannerCommand, submitConfirmationAction
  shell-event-handlers.ts # isPushToTalkKeyEvent, isEditableTarget, isSettingsActionBusy,
                          #   isUrlInputActionBusy, keyboard/pointer event handler wiring
  settings-statuses.ts    # deriveSettingsStatuses + SettingsStatuses type
```

### 5.1 Extract `panel-state-setters.ts`

- [ ] Move all `function set*State(...)` functions (lines ~541–608) plus `createRequestId`
      into `src/panel-state-setters.ts`.
- [ ] Import them back in `main.ts`.
- [ ] Run `pnpm lint && pnpm test:ui && pnpm build`.

### 5.2 Extract `settings-statuses.ts`

- [ ] Move `deriveSettingsStatuses` (line ~498) and any local type aliases it needs into
      `src/settings-statuses.ts`.
- [ ] Import in `main.ts`.
- [ ] Run validation.

### 5.3 Extract `settings-actions.ts`

- [ ] Move the persist/save functions for audio, provider, confirmation, and OCR settings
      (`persistPlaybackVolume`, `persistPlaybackSpeed`, `persistBrowserVisibility`,
      `persistAsrProviderSelection`, `persistTtsProviderSelection`, `persistTtsModelSelection`,
      `persistTtsVoiceSelection`, `persistConfirmationThreshold`,
      `persistAllowClickWithoutConfirmation`, `persistOcrThresholds`,
      `persistModelManagementSettings`) into `src/settings-actions.ts`.
- [ ] Run validation.

### 5.4 Extract `planner-actions.ts`

- [ ] Move `persistRemotePlannerApiKey`, `loadRemotePlannerModels`,
      `persistRemotePlannerConnection`, `resetRemotePlannerConnectionToDefaults`,
      `persistRemoteTtsApiKey`, `persistRemoteAsrApiKey`,
      `testConfiguredRemotePlannerApiKey`, `testConfiguredRemoteTtsApiKey`,
      `testConfiguredRemoteAsrApiKey`, `downloadManagedLocalTtsModel`,
      `downloadManagedLocalAsrModel` into `src/planner-actions.ts`.
- [ ] Run validation.

### 5.5 Extract `browser-actions.ts`

- [ ] Move `openDraftUrl`, `readCurrentPage`, `stopCurrentReading`, `readNextRegion`,
      `readPreviousRegion` into `src/browser-actions.ts`.
- [ ] Run validation.

### 5.6 Extract `voice-loop.ts`

- [ ] Move `beginPushToTalk`, `cancelPushToTalk`, `releasePushToTalk`,
      `ensureContinuousListeningLoop`, `stopContinuousListeningAfterFailure`,
      `executeUrlPanelPlannerCommand`, `submitConfirmationAction`
      into `src/voice-loop.ts`.
- [ ] Run validation.

### 5.7 Extract `shell-event-handlers.ts`

- [ ] Move `isPushToTalkKeyEvent`, `isEditableTarget`, `isSettingsActionBusy`,
      `isUrlInputActionBusy` and the event-handler registrations (keyboard/pointer
      listeners) into `src/shell-event-handlers.ts`.
- [ ] Run validation.

### 5.8 Slim down `BlindBrowserApp` component

- [ ] After extracting all action functions, the `BlindBrowserApp` React component
      (line ~222) should contain only: panel content wiring (the `panelContent` record
      passed to AppShellRuntime), navigation handler wiring, and the render call.
- [ ] Split it into a separate `src/app.ts` (or `app.tsx` if JSX is introduced) and
      import it in the new thin `main.ts`.
- [ ] Run full validation gate.

---

## Phase 6 — Final validation and documentation

### 6.1 Run the full validation suite

- [ ] `source ./fix-node-version.sh`
- [ ] `pnpm lint`
- [ ] `pnpm test:ui`
- [ ] `pnpm build`
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml --all-features`
- [ ] Fix any lint errors or test failures before committing.

### 6.2 Verify no file exceeds 600 lines (target)

- [ ] Run `find . -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.mjs" \) -not -path "*/node_modules/*" -not -path "*/dist/*" -not -path "*/target/*" | xargs wc -l | sort -rn | head -20`.
- [ ] Any file still over 600 lines should be reviewed and split further if the split is
      clean and natural. Files like `styles.css` and fixture-heavy test files are exempt
      if the content is genuinely cohesive.

### 6.3 Update memory.md

- [ ] Run `date -u +"%Y-%m-%dT%H:%M:%SZ"` and add an entry to `memory.md` summarizing
      the refactor phases completed, the final commit hash, and validation status.

---

## Suggested commit sequence

### Commit 1
Phase 1: split `app_core.rs` into `app_core/` module — replanning, settings adapters, model management, api key tools, navigation/content/extraction/interaction/voice/form-fill tools, planner prompt, tool executor.

### Commit 2
Phase 2: split `commands/tests.rs` into `commands/tests/` — fixtures, tool dispatch, browser state, runtime status, listening, planner flow, skill selection, confirmation.

### Commit 3
Phase 3: split `config.rs` into `config/` — types, loading, validation, keyring, tests.

### Commit 4
Phase 4: split `commands/routing.rs` into `commands/routing/` — intent, audio commands, URL commands, planner outputs, status commands.

### Commit 5
Phase 5: split `src/main.ts` — panel state setters, settings statuses, settings actions, planner actions, browser actions, voice loop, shell event handlers, slim App component.
