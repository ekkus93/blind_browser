# Refactor TODO 5 — blind_browser

Structural split of remaining large test files. Same constraints as REFACTOR1–4: pure
structural refactors, no behavior changes, full validation after every phase.

---

## Goal

Three test files over the 600-line target after REFACTOR4:

| File | Lines | Root cause |
|------|-------|------------|
| `src/confirmation-panel.test.mjs` | 1537 | 52 tests across 6 panel categories in one flat file |
| `commands/tests/contracts.rs` | 707 | 16 tests spanning tool schemas, result envelope, and planner contracts |
| `config/tests/load_tests.rs` | 704 | 10 tests spanning enum serialization, valid parse, and invalid parse |

`commands/tests/fixtures/mock_executor_impl.rs` (853 lines) is a confirmed exception:
it is entirely one `impl DeterministicToolExecutor for MockExecutor` block. Rust does not
allow splitting a trait impl across multiple files. No further splitting is possible.

### Validation gate
```bash
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
pnpm lint
pnpm test:ui
pnpm build
```

---

## Phase 1 — Split `commands/tests/contracts.rs` to directory

16 tests across three themes: tool schema coverage, ToolResult envelope serialization,
and planner output/input contract round-trips.

### Target layout

```
commands/tests/contracts/
  mod.rs                   # use super::*; + 3 mod declarations
  tool_schemas.rs          # registered_tools_all_expose_input_schemas,
                           #   sample_planned_steps_match_generated_tool_input_schemas,
                           #   registered_tools_all_expose_output_schemas,
                           #   registered_tools_include_output_schema_refs  — 4 tests
  tool_result_envelope.rs  # tool_result_success_populates_common_envelope_fields,
                           #   tool_result_failure_populates_common_envelope_fields,
                           #   serialized_tool_result_round_trips_with_warning_and_error_details,
                           #   typed_tool_result_deserializes_common_envelope_and_payload,
                           #   shared_contract_enums_serialize_expected_variants  — 5 tests
  planner_contracts.rs     # canonical_planner_output_examples_serialize_expected_strings_and_fields,
                           #   canonical_planner_output_examples_match_generated_planner_output_schema,
                           #   planner_output_round_trips_with_confirmation_metadata_and_matches_schema,
                           #   canonical_planner_output_step_arguments_match_generated_tool_input_schemas,
                           #   planner_input_round_trips_with_nested_runtime_context_and_matches_schema,
                           #   planner_input_serializes_safety_settings_for_click_policy,
                           #   sample_serialized_tool_results_match_generated_tool_output_schemas  — 7 tests
```

### 1.1 Create directory and move file

- [x] Create `commands/tests/contracts/` directory.
- [x] Move `commands/tests/contracts.rs` → `commands/tests/contracts/mod.rs`.
- [x] Run `cargo test --all-features`.

### 1.2 Extract subfiles

- [x] Extract `tool_schemas.rs` — 4 tests (lines ~3–63).
- [x] Extract `tool_result_envelope.rs` — 5 tests (lines ~65–308).
- [x] Extract `planner_contracts.rs` — 7 tests (lines ~310–706).
- [x] Trim `contracts/mod.rs` to module declarations only.
- [x] Run full validation gate.

---

## Phase 2 — Split `config/tests/load_tests.rs` to directory

10 tests: enum serialization, valid config parses, and invalid config rejection.

### Target layout

```
config/tests/load_tests/
  mod.rs               # use super::super::*; + 3 mod declarations (inherits config test helpers)
  enum_serialization.rs  # config_enums_round_trip_and_reject_invalid_variants,
                         #   provider_configs_round_trip_through_json  — 2 tests
  valid_configs.rs     # parses_default_template,
                       #   parses_ollama_planner_profile_when_selected  — 2 tests
  invalid_configs.rs   # rejects_missing_selected_remote_planner_profile_reference,
                       #   rejects_inline_secret_refs,
                       #   rejects_local_planner_configuration,
                       #   rejects_missing_remote_profile_for_remote_mode,
                       #   rejects_missing_selected_profiles_for_tts_and_asr_modes,
                       #   rejects_missing_selected_local_profile_references_for_tts_and_asr  — 6 tests
```

Note: `config/tests/mod.rs` defines shared helpers (`test_config_path`, `test_temp_path`).
Sub-files use `use super::*;` to inherit them (since `load_tests/mod.rs` uses `use super::*;`
from `config/tests/mod.rs`).

### 2.1 Create directory and move file

- [x] Create `config/tests/load_tests/` directory.
- [x] Move `config/tests/load_tests.rs` → `config/tests/load_tests/mod.rs`.
- [x] Run `cargo test --all-features`.

### 2.2 Extract subfiles

- [x] Extract `enum_serialization.rs` — 2 tests (lines ~3–174).
- [x] Extract `valid_configs.rs` — 2 tests (lines ~177–280).
- [x] Extract `invalid_configs.rs` — 6 tests (lines ~282–703).
- [x] Trim `load_tests/mod.rs` to module declarations only.
- [x] Run full validation gate.

---

## Phase 3 — Split `src/confirmation-panel.test.mjs` to themed files

52 JS tests across 6 panel categories. The file has a 249-line shared helper block
(HTML serializer + 18 render wrappers + renderFixtures) that all test files need.

### Target layout

```
src/
  confirmation-panel-test-helpers.mjs    # VOID_ELEMENTS, escapeHtml, mapAttributeName,
                                         #   renderNodeMarkup (mini HTML serializer),
                                         #   18 render wrapper functions, renderFixtures()
  confirmation-panel-core.test.mjs       # confirmation panel (4 tests) +
                                         #   push-to-talk panel (5 tests) — 9 tests
  confirmation-panel-url-audio.test.mjs  # URL input panel (7 tests) +
                                         #   audio controls panel (3 tests) — 10 tests
  confirmation-panel-settings.test.mjs   # all settings panels (remote planner, failover,
                                         #   confirmation, OCR, guidance, ASR, TTS) — ~24 tests
  confirmation-panel-status.test.mjs     # status panel (4 tests) +
                                         #   voice status strip (5 tests) — 9 tests
```

Note: the original `confirmation-panel.test.mjs` is deleted (replaced by the 4 test files).
The helpers file has no `.test.` in its name so the test runner does not pick it up as tests.
Each test file imports all helpers: `import { ... } from './confirmation-panel-test-helpers.mjs';`

### 3.1 Extract shared helpers

- [x] Create `src/confirmation-panel-test-helpers.mjs`.
- [x] Move VOID_ELEMENTS, escapeHtml, mapAttributeName, renderNodeMarkup, all 19 render
      wrapper functions, and renderFixtures() into the helpers file.
- [x] Export everything from the helpers file.
- [x] Run `pnpm test:ui`.

### 3.2 Extract themed test files

- [x] Create `confirmation-panel-core.test.mjs` — confirmation panel + push-to-talk tests (9 tests).
- [x] Create `confirmation-panel-url-audio.test.mjs` — URL input + audio controls tests (11 tests).
- [x] Create `confirmation-panel-settings.test.mjs` — all settings panel tests (36 tests).
- [x] Create `confirmation-panel-status.test.mjs` — status + voice strip tests (9 tests).
- [x] Delete original `confirmation-panel.test.mjs`.
- [x] Run `pnpm test:ui`.

### 3.3 Run full validation gate

- [x] `pnpm lint`
- [x] `pnpm test:ui`
- [x] `pnpm build`

---

## Phase 4 — Final validation and documentation

### 4.1 Run the full validation suite

- [ ] `source ./fix-node-version.sh`
- [ ] `pnpm lint`
- [ ] `pnpm test:ui`
- [ ] `pnpm build`
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml --all-features`

### 4.2 Verify file sizes

- [ ] Run `find . -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.mjs" \) -not -path "*/node_modules/*" -not -path "*/dist/*" -not -path "*/target/*" | xargs wc -l | sort -rn | head -30`.
- [ ] Target: no source file over 600 lines (except `mock_executor_impl.rs` — documented exception).

### 4.3 Update `memory.md`

- [ ] Run `date -u +"%Y-%m-%dT%H:%M:%SZ"` and add an entry to `memory.md`.

---

## Suggested commit sequence

```
Commit 1:  Phase 1 — split commands/tests/contracts.rs to directory
Commit 2:  Phase 2 — split config/tests/load_tests.rs to directory
Commit 3:  Phase 3 — split confirmation-panel.test.mjs into 4 themed files
Commit 4:  Phase 4 — final validation and documentation
```
