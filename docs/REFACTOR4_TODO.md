# Refactor TODO 4 — blind_browser

Structural split of large test files. Same constraints as REFACTOR1–3: pure structural
refactors, no behavior changes, full validation after every phase.

---

## Goal

Five test files exceed the 600-line target after REFACTOR3:

| File | Lines | Root cause |
|------|-------|------------|
| `app_core/tests.rs` | 3594 | 85 tests across unrelated subsystems; 457-line helper block |
| `commands/tests/fixtures.rs` | 2023 | `impl DeterministicToolExecutor for MockExecutor` (640 lines) embedded *inside* `assert_planner_skill_fixture` function body; large step/schema fixture blocks |
| `commands/tests/direct_commands.rs` | 1865 | 20 tests each with 80–165 lines of inline `PlannerOutput` assertions |
| `commands/tests/planner_flow.rs` | 1470 | 35 tests across execution flow + output + input validation |
| `commands/tests/tool_dispatch.rs` | 1033 | 30 dispatch tests covering all tools in one file |

### Validation gate
```bash
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
pnpm lint
pnpm test:ui
pnpm build
```

---

## Implementation notes

### Module visibility in test directories

When promoting a test file to a directory, sub-files need access to shared fixtures. The
safe pattern used throughout: in each sub-file, import directly from the ancestor module:

```rust
// in commands/tests/tool_dispatch/nav_tools.rs
use super::super::fixtures::*;  // or use super::super::*; if mod.rs re-exports
use super::super::*;
```

Alternatively, `tool_dispatch/mod.rs` can re-export with `pub(super) use super::fixtures::*;`
and sub-files use `use super::*;`. Either approach is fine; be consistent within a phase.

### Key fix for Phase 1 (fixtures.rs)

The `impl DeterministicToolExecutor for MockExecutor` block (lines 539–1380 in fixtures.rs)
is currently inside the body of `assert_planner_skill_fixture`. This is valid Rust but makes
the function 938 lines. Move the impl block to module level (in `mock_executor.rs`) so
`assert_planner_skill_fixture` shrinks to ~80 lines of assertion logic.

---

## Phase 1 — Promote `commands/tests/fixtures.rs` to directory

`fixtures.rs` mixes: path helpers, mock executor struct + impls, agent/page fixture builders,
skill fixture runner, step fixtures, and schema assertion helpers.

### Target layout

```
commands/tests/fixtures/
  mod.rs              # use super::*; + module declarations + pub(super) re-exports
  path_helpers.rs     # unique_temp_path, write_skill_document
  mock_executor.rs    # MockExecutor struct + Default + impl + DeterministicToolExecutor impl
                      #   (move impl block OUT of assert_planner_skill_fixture)
  page_fixtures.rs    # fixture_agent_state*, fixture_runtime_status,
                      #   fixture_page_model*, fixture_problematic_*
  skill_fixtures.rs   # PlannerSkillFixture, resolve_planner_skill_fixture,
                      #   assert_planner_skill_fixture (now ~80 lines after impl move)
  step_fixtures.rs    # sample_planned_step, sample_planned_steps_for_registered_tools
  schema_helpers.rs   # assert_json_matches_schema*, resolve_schema_reference,
                      #   json_matches_type, json_matches_single_type
```

### 1.1 Create `fixtures/` directory and move `fixtures.rs` → `fixtures/mod.rs`

- [ ] Create `commands/tests/fixtures/` directory.
- [ ] Move `commands/tests/fixtures.rs` → `commands/tests/fixtures/mod.rs`.
- [ ] Run `cargo test --all-features` to confirm nothing breaks.

### 1.2 Extract `mock_executor.rs` — move `impl DeterministicToolExecutor` out of function

- [ ] Create `fixtures/mock_executor.rs`.
- [ ] Move `MockExecutor` struct (lines ~20–56), `impl Default for MockExecutor` (~57–97),
      and `impl MockExecutor` (~98–190) into `mock_executor.rs`.
- [ ] Move `impl DeterministicToolExecutor for MockExecutor` (currently inside
      `assert_planner_skill_fixture`, lines ~539–1380) to module level in `mock_executor.rs`.
- [ ] Update `assert_planner_skill_fixture` to use `MockExecutor` directly (the impl is now
      at module level so the function body just needs `let mut executor = MockExecutor::default();`).
- [ ] Add `mod mock_executor; use mock_executor::*;` in `fixtures/mod.rs`.
- [ ] Run `cargo test --all-features`.

### 1.3 Extract `page_fixtures.rs`

- [ ] Create `fixtures/page_fixtures.rs`.
- [ ] Move `fixture_agent_state()`, `fixture_runtime_status()`, `fixture_page_model_without_regions()`,
      `fixture_agent_state_for_page()`, `fixture_problematic_article_page_without_regions()`,
      `fixture_problematic_docs_agent_state()` into `page_fixtures.rs`.
- [ ] Add `mod page_fixtures; use page_fixtures::*;` in `fixtures/mod.rs`.
- [ ] Run `cargo test --all-features`.

### 1.4 Extract `path_helpers.rs`, `skill_fixtures.rs`, `step_fixtures.rs`, `schema_helpers.rs`

- [ ] `path_helpers.rs` — `unique_temp_path`, `write_skill_document`.
- [ ] `skill_fixtures.rs` — `PlannerSkillFixture` struct, `resolve_planner_skill_fixture`,
      `assert_planner_skill_fixture` (now ~80 lines after impl move).
- [ ] `step_fixtures.rs` — `sample_planned_step`, `sample_planned_steps_for_registered_tools`.
- [ ] `schema_helpers.rs` — `assert_json_matches_schema`, `assert_json_matches_schema_at`,
      `resolve_schema_reference`, `json_matches_type`, `json_matches_single_type`.
- [ ] Trim `fixtures/mod.rs` to only module declarations + re-exports.
- [ ] Run full validation gate.

---

## Phase 2 — Promote `commands/tests/tool_dispatch.rs` to directory

30 dispatch tests, one per registered tool. Natural grouping by tool category.

### Target layout

```
commands/tests/tool_dispatch/
  mod.rs              # use super::*; + module declarations
  nav_tools.rs        # open_url, go_back, go_forward, reload_page, browser_visibility
  content_tools.rs    # get_html, eval_js, scroll_page, read_region (×3), stop_speaking
  voice_tools.rs      # start_listening, stop_listening, transcribe_command
  ocr_tools.rs        # capture_screenshot, run_ocr, merge_ocr, get_page_snapshot
  element_tools.rs    # list_interactive_elements, find_element, click_element,
                      #   focus_element, type_into_element, submit_active_form,
                      #   extract_page_model
  validation_tests.rs # rejects_invalid_tool_arguments, validate_schema_mismatch,
                      #   set_playback_volume, get_runtime_status
```

### 2.1 Create directory and move file

- [x] Create `commands/tests/tool_dispatch/` directory.
- [x] Move `commands/tests/tool_dispatch.rs` → `commands/tests/tool_dispatch/mod.rs`.
- [x] Run `cargo test --all-features`.

### 2.2 Extract subfiles

- [x] Extract `nav_tools.rs` — 5 tests.
- [x] Extract `content_tools.rs` — 7 tests.
- [x] Extract `voice_tools.rs` — 3 tests.
- [x] Extract `ocr_tools.rs` — 4 tests.
- [x] Extract `element_tools.rs` — 7 tests.
- [x] Extract `validation_tests.rs` — 4 tests.
- [x] Trim `tool_dispatch/mod.rs` to module declarations only.
- [x] Run full validation gate.

---

## Phase 3 — Promote `commands/tests/direct_commands.rs` to directory

20 tests covering all direct voice-command resolvers. Each test is 80–165 lines of inline
`PlannerOutput` assertions. Split by command category.

### Target layout

```
commands/tests/direct_commands/
  mod.rs                # use super::*; + module declarations
  audio_commands.rs     # resolve_direct_audio_command (×3),
                        #   resolve_direct_browser_visibility_command (×2)
  navigation_commands.rs # resolve_direct_navigation_readback_command (×2),
                        #   resolve_direct_voice_input_command (×2),
                        #   resolve_direct_open_url_command (×1)
  reading_commands.rs   # resolve_direct_read_page_command (×3)
  status_commands.rs    # resolve_direct_status_query_command (×3)
  playback_commands.rs  # resolve_direct_repeat_command (×2),
                        #   resolve_direct_read_title_command (×2)
```

### 3.1 Create directory and move file

- [x] Create `commands/tests/direct_commands/` directory.
- [x] Move `commands/tests/direct_commands.rs` → `commands/tests/direct_commands/mod.rs`.
- [x] Run `cargo test --all-features`.

### 3.2 Extract subfiles

- [x] Extract `audio_commands.rs` — 5 tests (~161 lines).
- [x] Extract `navigation_commands.rs` — 5 tests (~194 lines).
- [x] Extract `reading_commands.rs` — 3 tests (~456 lines).
- [x] Extract `status_commands.rs` — 3 tests (~480 lines).
- [x] Extract `playback_commands.rs` — 4 tests (~572 lines).
- [x] Trim `direct_commands/mod.rs` to module declarations only.
- [x] Run full validation gate.

---

## Phase 4 — Promote `commands/tests/planner_flow.rs` to directory

35 tests: execution flow, planner output validation, input validation.

### Target layout

```
commands/tests/planner_flow/
  mod.rs               # use super::*; + module declarations
  execution.rs         # executes_next_step_chain, executes_load_page_extract_and_read,
                       #   executes_resolved_spoken_command, follows_failure_transition,
                       #   returns_awaiting_confirmation, aborts_when_next_step_missing,
                       #   aborts_needs_confirmation_before_side_effecting (~364 lines, 7 tests)
  output_validation.rs # planner_available_tools_include_all_wave_two_tools,
                       #   validate_planner_output_rejects_* (13 tests),
                       #   validate_planner_output_accepts_* (2 tests)  (~590 lines)
  input_validation.rs  # set_tts_voice_input, validate_planner_output_rejects_open_url,
                       #   validate_eval_js_input, remaining validation tests (~515 lines)
```

### 4.1 Create directory and move file

- [x] Create `commands/tests/planner_flow/` directory.
- [x] Move `commands/tests/planner_flow.rs` → `commands/tests/planner_flow/mod.rs`.
- [x] Run `cargo test --all-features`.

### 4.2 Extract subfiles

- [x] Extract `execution.rs` — 7 step-execution tests.
- [x] Extract `output_validation.rs` — validate_planner_output tests.
- [x] Extract `input_validation.rs` — remaining input/argument validation tests.
- [x] Trim `planner_flow/mod.rs` to module declarations only.
- [x] Run full validation gate.

---

## Phase 5 — Promote `app_core/tests.rs` to directory

3594 lines, 85 tests across 12 subsystems + 457 lines of shared helpers.

### Target layout

```
app_core/tests/
  mod.rs                  # module declarations + re-exports of helpers
  helpers.rs              # shared infra: imports, MockBrowser, fixture_page*,
                          #   planner_tool_sequence, resolve_app_core_planner_fixture,
                          #   assert_app_core_planner_fixture, spawn_openai_models_test_server
  settings_tests.rs       # build_remote_planner_settings (×2),
                          #   build_remote_tts/asr_settings, build_provider_failover,
                          #   build_confirmation_settings, build_local_*_model_settings,
                          #   build_ocr_threshold, build_asr/tts_provider_settings,
                          #   build_tts_voice_settings (×3) — 12 tests
  browser_tests.rs        # normalize_optional_text, normalize_absolute_url,
                          #   browser_error_to_tool_error, refresh_current_page,
                          #   clear_navigation_follow_up_state — 5 tests
  extraction_tests.rs     # build_visible_text_excerpt, region_bbox_by_id,
                          #   build_extracted_page_model_* (×4),
                          #   infer_extraction_source_* (×3) — 8 tests
  ocr_threshold_tests.rs  # should_trigger_*_ocr_fallback (×6),
                          #   extracted_text_metrics_counts — 7 tests
  ocr_merge_tests.rs      # region_first_ocr_target_ids (×2), merged_region_text,
                          #   merge_ocr_text_into_page_model_* (×5) — 8 tests
  focus_fill_tests.rs     # filter_interactive_elements,
                          #   resolve_direct_focus_field_command (×3),
                          #   resolve_direct_fill_field_command (×2),
                          #   resolve_direct_fill_and_submit_command (×2) — 8 tests
  fill_correction_tests.rs # resolve_recent_fill_correction_command (×3),
                           #   resolve_typeable_element,
                           #   resolve_direct_submit_form_command (×2) — 6 tests
  regression_tests.rs     # app_core_form_regression_fixtures,
                          #   ambiguous_click_regression_fixtures,
                          #   problematic_page_regression_fixtures — 3 tests
  element_scoring_tests.rs # resolve_form_element_rejects_non_form_roles,
                           #   rank_find_element_candidates (×2),
                           #   build_find_element_query,
                           #   determine_find_element_resolution (×2) — 6 tests
  planner_tests.rs        # planner_system_prompt_mentions_click_confirmation,
                          #   bounded_replanning_loop (×3),
                          #   resolve_clickable_element (×3),
                          #   test_openai_api_key_connectivity (×2),
                          #   fetch_openai_compatible_models (×2) — 11 tests
```

### 5.1 Create directory and move file

- [x] Create `app_core/tests/` directory.
- [x] Move `app_core/tests.rs` → `app_core/tests/mod.rs`.
- [x] Run `cargo test --all-features`.

### 5.2 Extract `helpers.rs`

- [x] Create `app_core/tests/helpers.rs`.
- [x] Move lines 1–457 (all shared infrastructure) into `helpers.rs`.
- [x] Add `mod helpers; use helpers::*;` in `tests/mod.rs`.
- [x] Run `cargo test --all-features`.

### 5.3 Extract thematic test files (all 10)

Work through these in order; run `cargo test --all-features` after each:

- [x] `settings_tests.rs` — 15 tests.
- [x] `browser_tests.rs` — 5 tests.
- [x] `extraction_tests.rs` — 8 tests.
- [x] `ocr_threshold_tests.rs` — 9 tests.
- [x] `ocr_merge_tests.rs` — 8 tests.
- [x] `focus_fill_tests.rs` — 8 tests.
- [x] `fill_correction_tests.rs` — 6 tests.
- [x] `regression_tests.rs` — 3 tests.
- [x] `element_scoring_tests.rs` — 6 tests.
- [x] `planner_tests.rs` — 11 tests.
- [x] Trim `tests/mod.rs` to module declarations + re-exports only.
- [x] Run full validation gate.

---

## Phase 6 — Final validation and documentation

### 6.1 Run the full validation suite

- [ ] `source ./fix-node-version.sh`
- [ ] `pnpm lint`
- [ ] `pnpm test:ui`
- [ ] `pnpm build`
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml --all-features`

### 6.2 Verify file sizes

- [ ] Run `find . -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.mjs" \) -not -path "*/node_modules/*" -not -path "*/dist/*" -not -path "*/target/*" | xargs wc -l | sort -rn | head -30`.
- [ ] Target: no source file over 600 lines.

### 6.3 Update `memory.md`

- [ ] Run `date -u +"%Y-%m-%dT%H:%M:%SZ"` and add an entry to `memory.md`.

---

## Suggested commit sequence

```
Commit 1:  Phase 1 — promote commands/tests/fixtures.rs to directory
Commit 2:  Phase 2 — promote commands/tests/tool_dispatch.rs to directory
Commit 3:  Phase 3 — promote commands/tests/direct_commands.rs to directory
Commit 4:  Phase 4 — promote commands/tests/planner_flow.rs to directory
Commit 5:  Phase 5 — promote app_core/tests.rs to directory
Commit 6:  Phase 6 — final validation and documentation
```
