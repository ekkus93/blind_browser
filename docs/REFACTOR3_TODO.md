# Refactor TODO 3 — blind_browser

Based on a structural analysis of the codebase in June 2026.  
**Status key:** PENDING · IN PROGRESS · DONE

---

## Goal

After REFACTOR2, seven production files remain above the 600-line target:

| File | Lines |
|------|-------|
| `src-tauri/src/config/mod.rs` | 851 |
| `src-tauri/src/app_core/mod.rs` | 850 |
| `src-tauri/src/app_core/extraction_tools.rs` | 849 |
| `src-tauri/src/app_core/interaction_tools.rs` | 779 |
| `src-tauri/src/commands/planner_executor.rs` | 685 |
| `src-tauri/src/browser/mod.rs` | 685 |
| `src-tauri/src/app_core/form_fill.rs` | 676 |

This third pass continues the structural cleanup with the same constraints as REFACTOR1/2:
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

Simpler within-directory splits first (phases 1–3), then file-to-directory
promotions (phases 4–7), then final validation (phase 8).

1. Split `app_core/mod.rs` — add subfiles within the existing `app_core/` directory.
2. Split `browser/mod.rs` — add subfiles within the existing `browser/` directory.
3. Split `config/mod.rs` — add a `persistence.rs` subfile within the existing `config/` directory.
4. Promote `commands/planner_executor.rs` → `commands/planner_executor/` directory.
5. Promote `app_core/extraction_tools.rs` → `app_core/extraction_tools/` directory.
6. Promote `app_core/interaction_tools.rs` → `app_core/interaction_tools/` directory.
7. Promote `app_core/form_fill.rs` → `app_core/form_fill/` directory.
8. Final validation and documentation.

---

## Phase 1 — Split `app_core/mod.rs` (~850 lines)

`mod.rs` still holds three distinct concerns inline: the main command-resolution logic
(including the 268-line `resolve_command_with_recent_results`), the confirmation workflow,
and result/state reporting. The struct, constructor, field-context helpers, error helpers,
ID generators, and path helpers are the natural remainder.

### Target layout

```
src-tauri/src/app_core/
  mod.rs                  # AppCore struct + constructor, update_audio_settings,
                          #   field-context helpers, error helpers, ID generators,
                          #   path/cache helpers, module declarations (~320 lines target)
  command_dispatch.rs     # impl AppCore: resolve_command, execute_command_with_replanning,
                          #   execute_planner_output, resolve_command_with_recent_results
  confirmation_workflow.rs # impl AppCore: resume_after_confirmation,
                          #   submit_confirmation_response, execute_confirm_action
  result_reporting.rs     # impl AppCore: execute_get_agent_state,
                          #   execute_get_runtime_status, execute_report_result
```

### 1.1 Extract `command_dispatch.rs`

- [x] Move `resolve_command`, `execute_command_with_replanning`, `execute_planner_output`,
      and `resolve_command_with_recent_results` into `app_core/command_dispatch.rs` as a
      second `impl AppCore` block.
- [x] Add `mod command_dispatch;` in `app_core/mod.rs`.
- [x] Run `cargo check`.

### 1.2 Extract `confirmation_workflow.rs`

- [x] Move `resume_after_confirmation`, `submit_confirmation_response`,
      `execute_confirm_action` into `app_core/confirmation_workflow.rs`.
- [x] Add `mod confirmation_workflow;` in `app_core/mod.rs`.
- [x] Run `cargo check`.

### 1.3 Extract `result_reporting.rs`

- [x] Move `execute_get_agent_state`, `execute_get_runtime_status`, `execute_report_result`
      into `app_core/result_reporting.rs`.
- [x] Add `mod result_reporting;` in `app_core/mod.rs`.
- [x] Run full validation gate. `mod.rs` should be under 400 lines.

---

## Phase 2 — Split `browser/mod.rs` (~685 lines)

`browser/mod.rs` still contains all element-interaction methods and page-inspection methods
alongside the lightweight session management and URL-opening code. The interaction methods
alone are ~430 lines.

### Target layout

```
src-tauri/src/browser/
  mod.rs                  # BrowserController struct, new(), open_url(), switch_visibility(),
                          #   ensure_session() (private), wait_for_page_settle() (~120 lines)
  element_interaction.rs  # impl BrowserController: click_element, focus_element,
                          #   type_into_element, submit_active_form (~430 lines)
  page_inspection.rs      # impl BrowserController: capture_screenshot, get_html, eval_js;
                          #   free fn: png_dimensions (~160 lines)
```

### 2.1 Extract `element_interaction.rs`

- [x] Move `click_element`, `focus_element`, `type_into_element`, `submit_active_form`
      into `browser/element_interaction.rs` as a second `impl BrowserController` block.
- [x] Add `mod element_interaction;` in `browser/mod.rs`.
- [x] Run `cargo check`.

### 2.2 Extract `page_inspection.rs`

- [x] Move `capture_screenshot`, `get_html`, `eval_js`, and the free function
      `png_dimensions` into `browser/page_inspection.rs`.
- [x] Add `mod page_inspection;` in `browser/mod.rs`.
- [x] Run full validation gate. `mod.rs` should be under 200 lines.

---

## Phase 3 — Split `config/mod.rs` (~851 lines)

`config/mod.rs` contains all `persist_*` and `reset_*` methods on `AppConfig` — roughly
550 lines of persistence logic mixed with the config loading and construction logic.
Extracting persistence to its own file leaves `mod.rs` holding constants, module
declarations, loading wrappers, `from_raw()`, and `impl Default`.

### Target layout

```
src-tauri/src/config/
  mod.rs          # constants, module declarations, pub use, impl AppConfig:
                  #   default_template, config_path_for_app, load_for_app,
                  #   load_from_path, load_from_str; from_raw(); impl Default (~300 lines)
  persistence.rs  # impl AppConfig: all persist_*_for_app(), persist_*_at_path(),
                  #   reset_*_for_app(), reset_*_at_path() methods (~550 lines)
```

### 3.1 Extract `persistence.rs`

- [x] Move all `persist_*` and `reset_*` methods (both `_for_app` and `_at_path` variants)
      out of `impl AppConfig` in `config/mod.rs` into a second `impl AppConfig` block in
      `config/persistence.rs`.
- [x] Add `mod persistence;` in `config/mod.rs`.
- [x] Run full validation gate. `mod.rs` should be under 350 lines.

---

## Phase 4 — Promote `commands/planner_executor.rs` (~685 lines) to a directory

`planner_executor.rs` mixes three distinct concerns: the tool-dispatch switch
(`execute_planned_step` at ~190 lines), the execution orchestration functions
(`execute_steps_with_runner` at ~149 lines and its siblings), and a suite of smaller
helpers for step navigation, confirmation extraction, serialization, and tool classification.

### Target layout

```
src-tauri/src/commands/planner_executor/
  mod.rs          # public API re-exports and thin wrappers:
                  #   execute_planned_step (pub), execute_planner_output (pub),
                  #   resume_after_confirmation (pub); module declarations (~30 lines)
  tool_dispatch.rs # execute_planned_step (big tool switch), execute_serialized_tool,
                  #   is_side_effecting_tool (~240 lines)
  execution.rs    # execute_planner_output_with_runner,
                  #   resume_after_confirmation_with_runner,
                  #   execute_steps_with_runner (~290 lines)
  step_helpers.rs # build_step_positions, queued_step_ids_after, queued_steps_after,
                  #   extract_confirmation_id, extract_confirmation_prompt_text,
                  #   serialize_tool_result, inferred_request_id (~165 lines)
```

### 4.1 Create `planner_executor/` directory and `mod.rs`

- [x] Create `src-tauri/src/commands/planner_executor/` directory.
- [x] Move `commands/planner_executor.rs` → `commands/planner_executor/mod.rs`.
- [x] Run `cargo check` — Rust resolves `planner_executor/mod.rs` automatically.

### 4.2 Extract `tool_dispatch.rs`

- [x] Move `execute_planned_step`, `execute_serialized_tool`, `is_side_effecting_tool`
      into `planner_executor/tool_dispatch.rs`.
- [x] Add `mod tool_dispatch;` in `planner_executor/mod.rs`.
- [x] Run `cargo check`.

### 4.3 Extract `execution.rs`

- [x] Move `execute_planner_output_with_runner`, `resume_after_confirmation_with_runner`,
      `execute_steps_with_runner` into `planner_executor/execution.rs`.
- [x] Add `mod execution;` in `planner_executor/mod.rs`.
- [x] Run `cargo check`.

### 4.4 Extract `step_helpers.rs`

- [x] Move `build_step_positions`, `queued_step_ids_after`, `queued_steps_after`,
      `extract_confirmation_id`, `extract_confirmation_prompt_text`,
      `serialize_tool_result`, `inferred_request_id` into `planner_executor/step_helpers.rs`.
- [x] Add `mod step_helpers;` in `planner_executor/mod.rs`.
- [x] Run full validation gate.

---

## Phase 5 — Promote `app_core/extraction_tools.rs` (~849 lines) to a directory

`extraction_tools.rs` contains four `impl AppCore` methods with very different sizes:
`execute_extract_page_model` alone is ~338 lines. Splitting by concern (OCR vs page
extraction) produces two files both under 450 lines.

### Target layout

```
src-tauri/src/app_core/extraction_tools/
  mod.rs           # use super::*; module declarations; no AppCore methods (~10 lines)
  ocr_tools.rs     # impl AppCore: execute_run_ocr, execute_merge_ocr_into_page_model;
                   #   free fn: should_trigger_extract_page_model_ocr_fallback (~390 lines)
  page_extraction.rs # impl AppCore: execute_get_page_snapshot, execute_extract_page_model
                   #   (~445 lines)
```

### 5.1 Create `extraction_tools/` directory and `mod.rs`

- [x] Create `src-tauri/src/app_core/extraction_tools/` directory.
- [x] Move `app_core/extraction_tools.rs` → `app_core/extraction_tools/mod.rs`.
- [x] Run `cargo check`.

### 5.2 Extract `ocr_tools.rs`

- [x] Move `execute_run_ocr`, `execute_merge_ocr_into_page_model`, and
      `should_trigger_extract_page_model_ocr_fallback` into
      `extraction_tools/ocr_tools.rs`.
- [x] Add `mod ocr_tools;` in `extraction_tools/mod.rs`.
- [x] Run `cargo check`.

### 5.3 Extract `page_extraction.rs`

- [x] Move `execute_get_page_snapshot` and `execute_extract_page_model` into
      `extraction_tools/page_extraction.rs`.
- [x] Add `mod page_extraction;` in `extraction_tools/mod.rs`.
- [x] Run full validation gate. `mod.rs` should be under 20 lines.

---

## Phase 6 — Promote `app_core/interaction_tools.rs` (~779 lines) to a directory

`interaction_tools.rs` contains six `impl AppCore` methods and three free-function
element-resolution helpers. Methods group naturally by interaction type.

### Target layout

```
src-tauri/src/app_core/interaction_tools/
  mod.rs            # use super::*; module declarations (~10 lines)
  element_queries.rs # impl AppCore: execute_list_interactive_elements,
                    #   execute_find_element (~190 lines)
  click_focus.rs    # impl AppCore: execute_click_element, execute_focus_element;
                    #   free fn: resolve_clickable_element (~275 lines)
  text_entry.rs     # impl AppCore: execute_type_into_element, execute_submit_active_form;
                    #   free fns: resolve_typeable_element, resolve_form_element (~300 lines)
```

### 6.1 Create `interaction_tools/` directory and `mod.rs`

- [ ] Create `src-tauri/src/app_core/interaction_tools/` directory.
- [ ] Move `app_core/interaction_tools.rs` → `app_core/interaction_tools/mod.rs`.
- [ ] Run `cargo check`.

### 6.2 Extract `element_queries.rs`

- [ ] Move `execute_list_interactive_elements` and `execute_find_element` into
      `interaction_tools/element_queries.rs`.
- [ ] Add `mod element_queries;` in `interaction_tools/mod.rs`.
- [ ] Run `cargo check`.

### 6.3 Extract `click_focus.rs`

- [ ] Move `execute_click_element`, `execute_focus_element`, and the free function
      `resolve_clickable_element` into `interaction_tools/click_focus.rs`.
- [ ] Add `mod click_focus;` in `interaction_tools/mod.rs`.
- [ ] Run `cargo check`.

### 6.4 Extract `text_entry.rs`

- [ ] Move `execute_type_into_element`, `execute_submit_active_form`,
      `resolve_typeable_element`, and `resolve_form_element` into
      `interaction_tools/text_entry.rs`.
- [ ] Add `mod text_entry;` in `interaction_tools/mod.rs`.
- [ ] Run full validation gate. `mod.rs` should be under 20 lines.

---

## Phase 7 — Promote `app_core/form_fill.rs` (~676 lines) to a directory

`form_fill.rs` contains three conceptually distinct resolution functions: focus-field
command building (~163 lines), fill-field command building including the 304-line internal
resolver, and submit-form command building (~144 lines).

### Target layout

```
src-tauri/src/app_core/form_fill/
  mod.rs          # use super::*; module declarations (~10 lines)
  field_focus.rs  # resolve_direct_focus_field_command (~163 lines)
  field_fill.rs   # resolve_direct_fill_command_internal;
                  #   #[cfg(test)] resolve_direct_fill_field_command,
                  #   resolve_direct_fill_and_submit_command (~340 lines)
  form_submit.rs  # resolve_direct_submit_form_command (~144 lines)
```

### 7.1 Create `form_fill/` directory and `mod.rs`

- [ ] Create `src-tauri/src/app_core/form_fill/` directory.
- [ ] Move `app_core/form_fill.rs` → `app_core/form_fill/mod.rs`.
- [ ] Run `cargo check`.

### 7.2 Extract `field_focus.rs`

- [ ] Move `resolve_direct_focus_field_command` into `form_fill/field_focus.rs`.
- [ ] Add `mod field_focus;` in `form_fill/mod.rs`.
- [ ] Run `cargo check`.

### 7.3 Extract `form_submit.rs`

- [ ] Move `resolve_direct_submit_form_command` into `form_fill/form_submit.rs`.
- [ ] Add `mod form_submit;` in `form_fill/mod.rs`.
- [ ] Run `cargo check`.

### 7.4 Extract `field_fill.rs`

- [ ] Move `resolve_direct_fill_command_internal` and the `#[cfg(test)]` wrappers
      (`resolve_direct_fill_field_command`, `resolve_direct_fill_and_submit_command`)
      into `form_fill/field_fill.rs`.
- [ ] Add `mod field_fill;` in `form_fill/mod.rs`.
- [ ] Run full validation gate. `mod.rs` should be under 20 lines.

---

## Phase 8 — Final validation and documentation

### 8.1 Run the full validation suite

- [ ] `source ./fix-node-version.sh`
- [ ] `pnpm lint`
- [ ] `pnpm test:ui`
- [ ] `pnpm build`
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml --all-features`
- [ ] Fix any lint errors or test failures before committing.

### 8.2 Verify file sizes

- [ ] Run `find . -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.mjs" \) -not -path "*/node_modules/*" -not -path "*/dist/*" -not -path "*/target/*" | xargs wc -l | sort -rn | head -30`.
- [ ] Target: no production source file over 600 lines. Fixture-heavy test files are exempt
      if content is genuinely cohesive.

### 8.3 Update `memory.md`

- [ ] Run `date -u +"%Y-%m-%dT%H:%M:%SZ"` and add an entry to `memory.md` summarizing
      completed phases, final commit hash, and validation status.

---

## Suggested commit sequence

```
Commit 1:  Phase 1 — split app_core/mod.rs (command_dispatch, confirmation_workflow, result_reporting)
Commit 2:  Phase 2 — split browser/mod.rs (element_interaction, page_inspection)
Commit 3:  Phase 3 — split config/mod.rs (persistence)
Commit 4:  Phase 4 — promote commands/planner_executor.rs to directory (tool_dispatch, execution, step_helpers)
Commit 5:  Phase 5 — promote app_core/extraction_tools.rs to directory (ocr_tools, page_extraction)
Commit 6:  Phase 6 — promote app_core/interaction_tools.rs to directory (element_queries, click_focus, text_entry)
Commit 7:  Phase 7 — promote app_core/form_fill.rs to directory (field_focus, field_fill, form_submit)
Commit 8:  Phase 8 — final validation and documentation
```
