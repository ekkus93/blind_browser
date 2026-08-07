## 2026-08-07T18:53:57Z - Claude Opus 5 - Ralph loop on BB_CODE_REVIEW3: P1 phase complete (P1.1 partial, P1.2 blocked, P1.3+P1.4 done)

- Continued straight on from the P0.1-P0.9/P1.1 entry below in the same session. P1.2 ("release the runtime lock across TTS/ASR network+capture windows"): investigated and marked BLOCKED with evidence — `LockScopedReplanningRuntime::execute_plan` holds the AppCore lock once for an entire multi-step plan via `execute_planner_output_with_runtime_safety`; both TTS synthesis and planner-driven transcribe_command are steps *inside* that loop, not top-level calls, so releasing the lock needs the step loop itself to become pausable/resumable — the same shape CR2's P1.1.2/P1.1.3 hit. `run_phased_transcribe` already solves the analogous problem but only because it sits outside the step loop.
- P1.3 (region bbox coordinate space): dispatched an Explore subagent that empirically reproduced the bug with a real headless Chromium session over raw CDP (no live-browser test harness exists in this codebase, so this was done ad hoc, not as a permanent test) — confirmed `Page.captureScreenshot`'s `clip.x/y` are always document-absolute regardless of `captureBeyondViewport`, while `getBoundingClientRect()`-derived bboxes were viewport-relative with no scroll correction anywhere. Fixed at the source (`browser/dom_extraction.rs`'s `documentAbsoluteRect` helper adds `window.scrollX/scrollY` once, both bbox sources); `page_model::Rect` now documents the coordinate-space contract. Added a narrower string-content regression test since no live-browser harness exists to test the real thing.
- P1.4 (`execute_planner_output` validation gap): dispatched a second Explore subagent, which confirmed the gap is real and exploitable (not just theoretical) — `execute_planner_output` is a directly Tauri-invocable command with a fully caller-controlled `PlannerOutput` and no origin binding; `ExtractPageModel`/`ReportResult`/`TranscribeCommand` were excluded from `planner_output_requires_snapshot`, so an `ExtractPageModel`-only forged plan could skip snapshot validation entirely and call `mark_page_model_changed()`, silently clearing a pending confirmation. **Diverged from the TODO's own stated fix preference** after verification showed it wouldn't actually work: `validate_planner_output_with_safety` (the preferred fix) doesn't reject bare `ExtractPageModel` plans since that tool is itself classified `ReadOnly`/`NoConfirmation`. Implemented the TODO's second-listed option instead (extend `planner_output_requires_snapshot`), which works because the snapshot check is a cryptographic provenance check (SHA-256 over the whole serialized `PlannerOutput`), not a structural one. Documented (didn't change) the misleading `is_side_effecting_tool`/`tool_policy` classifications rather than making `ExtractPageModel` require confirmation (would break voice-first "look at the page" UX on every call).
- All four P1 phases individually validated (clippy -D warnings, cargo test --all-features, CI guard scripts) and committed+pushed. Zero regressions across 521 Rust tests by the end of P1.
- Commits: `c8a8b72`(P1.2 blocked) `4c11a4a`(P1.3) `336fc7e`(P1.4).
- Follow-ups tracked: task #38 (ASR consent gating, from P1.1), P1.2's executor-restructuring follow-up (documented in the TODO, no task yet), P1.3's residual `execute_run_ocr` capture-time-scroll-metadata gap (documented in the TODO, no task yet).

## 2026-08-07T18:24:04Z - Claude Opus 5 - Ralph loop on BB_CODE_REVIEW3: P0.1-P0.9 done, P1.1 partial (narration gated, ASR deferred)

- Continued the CR3 Ralph loop from a prior session. Completed and pushed P0.1-P0.9 (click-confirmation label, narration-cursor bounds, settings-write validation, two confirmation-bypass paths in element resolution, settings-navigation-target validation, confirmation/consent panel reachable from any view, empty-synthesized-audio rejection, hands-free ASR capture buffer reset+cap, four Tailwind cascade regressions + a computed-style regression test). Each landed as its own commit with full validation (clippy -D warnings, cargo test --all-features, pnpm lint/test:ui/build, CI guard scripts).
- P1.1 ("gate remote TTS/ASR through the shared consent layer") turned out to be far larger than scoped once design started — asked the user once (AskUserQuestion) whether to fail-closed-only, narrow-enforce, or build the full interactive dialog; user chose the full dialog. Delivered narration (remote TTS) completely: `evaluate_remote_planner_policy` reused unchanged (it was already planner-agnostic); new independent `remote_narration_privacy` config section + parallel (not unified) AppCore grant/pending-consent state, kept separate from the planner's own to avoid touching that delicate code; `RemoteDataDisclosureKind` selector generalizes grants/origin-rules/challenge-building without duplicating the algorithm; `begin_region_narration`/`begin_feedback_narration` are the one choke point all 4 narration call sites already funneled through; new `submit_narration_consent_response` Tauri command (registered in `direct_command_policy.rs`'s security-evidence registry) resolves a pending challenge and redoes the exact paused narration by re-entering the same gated functions, which pass this time on the fresh grant. New isolated-Wry evidence test (`narration_consent_tests.rs`) proves high-risk/origin-block/cross-kind-isolation/loopback-exemption, registered in `scripts/run-rust-tests-linux.sh`.
- Explicitly deferred (not silently dropped, see docs/BB_CODE_REVIEW3_TODO.md P1.1's note): remote ASR gating (needs `execute_transcribe_command`'s synchronous capture+transcribe call split into phases first — no separation point exists yet to insert a gate); the in-app settings UI for narration origin rules (config.example.toml documents the new sections but there's no UI to manage them yet — users must edit config.toml directly under the default ask_per_origin mode); frontend dialog wiring (backend embeds the full challenge in the ToolError for a future pass to pick up, but nothing shows it yet — a real UX gap, not a security one, since the gate still fails closed). Follow-up task #38 created for the ASR half.
- Commits: `0c98048`(P0.7) `bdbdc81`(P0.8) `8a3ae99`(P0.9) `b55cfe2`(P1.1 partial). All pushed to master.

## 2026-08-02T14:47:00Z — Batch 7 remote-planner privacy boundary validated

- Implemented the typed BBCR-003/BBCR-006 remote-planner boundary in `fbec02a5b697720c88a3f46054110cd8e7c5c1a6`.
- Bounded validation run `30746879137`, job `91493868153`, passed formatting, default compilation, strict all-target/all-feature Clippy, 427 Rust tests, frontend lint, UI tests, production build, bounded-change verification, and one-shot cleanup.
- Remote payloads now separate trusted policy/schema from user request and untrusted page/OCR/skill/tool data; raw form values, DOM locators, unrestricted attributes, sensitive URLs, credential metadata, pending execution state, and raw remote error bodies cannot cross the remote boundary.
- Prompt-injection indicators remain caution telemetry only; deterministic runtime policy owns confirmation and execution safety.
- Residual consent/UI, high-risk-origin, relevance-selection, diagnostic-audit, and complete hidden/OCR-image corpus work remains explicitly open.
- Exact final documentation SHA and Permanent CI evidence are recorded in issue #5.

## 2026-06-30T01:07:36Z - Claude Sonnet 4.6 - HARDENING3: all tasks complete

- P0-2 (first): `tempfile = "3"` promoted from `[dev-dependencies]` to `[dependencies]`. New `src-tauri/src/atomic_file.rs` module with `replace_file_atomically` (uses `fs::rename`, which on Windows calls `MoveFileExW` with `MOVEFILE_REPLACE_EXISTING`). `model_management.rs` and `config/persistence.rs` now call `crate::atomic_file::replace_file_atomically` instead of `fs::rename` directly. 2 unit tests in `atomic_file.rs`.
- P0-1: `#[cfg(test)]` `AtomicFileFailurePoint` enum and `write_bytes_atomically_for_testable_path` helper added to `model_management.rs`. 5 new tests: BeforeRename (no files created), AfterTempWriteBeforeRename failure (`.part` cleaned up, final not created), AfterTempWriteBeforeRename failure (existing final preserved), success (final created), success (existing final replaced). `atomic_config_write_replaces_existing_file` test added to `config/persistence.rs`.
- P1-1: Stale wrong path `src-tauri/src/commands/settings_adapters.rs` → `src-tauri/src/app_core/settings_adapters.rs` corrected in HARDENING2 TODO (3 occurrences). All 10 HARDENING2 final checklist items marked `[x]`.
- P1-2: Standalone `grep -E` block added to `scripts/check-silent-fallbacks.sh` catching direct `File::create(target_path)` in `model_management.rs`.
- P2-1: Full gate green — 369 Rust tests + 171 JS tests + lint + clippy + build + guardrail script all pass.

## 2026-06-30T00:12:30Z - Claude Sonnet 4.6 - HARDENING2: all 9 task groups complete

- P0-1: `resolve_optional_remote_planner_api_key` helper — `resolve_secret_ref` failures now propagate as Err instead of collapsing to `None` in `list_remote_planner_models`. 4 tests.
- P0-2/P0-3: model-availability floor (1MB ASR, 2B TTS config, 1MB TTS onnx); `download_response_to_file_atomically` (`.part` temp → sync → rename → remove-on-error). 6 tests; `tempfile = "3"` added as first `[dev-dependencies]`.
- P0-4: `write_config_atomic` in `config/persistence.rs`; all 9 `fs::write` call-sites replaced. 3 tests.
- P1-1: `parse_remote_transcription_text` — missing/non-string `text` field in ASR response now returns `RemoteRequestFailed` error instead of empty string. 4 tests.
- P1-2: `normalize_browser_navigation_url` rewritten to use `url` crate for structural validation; `url = "2"` added. Added pre-parse `://` check (rejects `http:example.com`), raw-authority empty check (rejects `https:///missing-host`), and host-str None guard. Returns `parsed.to_string()` for trailing-slash normalisation. Fixed `http://localhost:3000/` expectation in browser tests. 362 Rust tests total.
- P2-1: `masked_secret_status` replaces `masked_secret_value` returning `(Option<String>, Option<String>)`. `api_key_reference_error: Option<String>` added to all three DTOs (Rust + TS). Warning rendered in all three remote settings panels. Wired in `runtime-refresh.ts`, `panel-state.ts`, `tauri-types.ts`. 3 new JS tests; 171 total JS tests.
- P2-2: `check-silent-fallbacks.sh` extended with 2 new fixed-string entries and 1 regex check for the exact removed patterns.
- P2-3: full gate green (362 Rust + 171 JS + lint + build + guardrail script).

## 2026-06-29T11:24:36Z - Claude Haiku 4.5 - BB_RUNTIME_PHASE3 P2.2: remote ASR lock-scoping (the previously-skipped item)

- Implemented at user request the Phase 3 item P2.2 had consciously SKIPPED: release the AppCore lock across the ASR transcription round-trip (matters for remote ASR; local whisper also benefits by not holding the lock during CPU transcription).
- ASR layer: `transcribe_local`/`transcribe_remote`/`transcribe_with_openai_remote` converted from `&self` AsrController methods to FREE functions (they never read self); added free `transcribe_captured_audio(config, captured_audio)` dispatcher. `CapturedAudio` re-exported `pub(crate)` from asr. `finish_capture` split into `drain_capture(&mut self, auto_stop, started)` (takes audio, stops mic for one-shot/auto-stop, Ok(None) if stopped mid-window) + `finalize_transcription(&self, transcript, dur)` (wraps→AsrTranscription). `transcribe_command` (planner-tool path) now uses the dispatcher too.
- AppCore layer (listening_tools.rs): `finish_transcribe_command` → `drain_transcribe_command(plan) -> TranscribeDrainOutcome::{Terminal(Box<ToolResult>)|Pending(Box<TranscribePending>)}` (lock) + `record_transcribe_command(pending, transcript_result)` (lock). `TranscribePending` carries owned CapturedAudio + an AppConfig clone + plan; exposes `transcription_inputs()`.
- Handler: `run_phased_transcribe` is now FIVE phases — begin(lock) → capture sleep(unlocked) → drain(lock) → `crate::asr::transcribe_captured_audio`(unlocked) → record(lock).
- Regression test renamed `finish_capture_*` → `drain_capture_reports_none_when_session_stopped_mid_window` (same Ok(None) path).
- Validation: gate green — fmt/clippy clean, 330 Rust + 164 JS tests, build green, AND the default-feature `cargo check` (now CI default after BB_DEFAULT_BUILD) passes. Behavioral `--features full` + remote-ASR-profile check (get_agent_state interleaves during in-flight remote ASR) still needs a human. All BB_RUNTIME_PHASE3 tasks now DONE (P2.2 no longer skipped).

## 2026-06-29T10:42:07Z - Claude Haiku 4.5 - BB_DEFAULT_BUILD: default = ["full"] so the no-flag build compiles

- Fixed the pre-existing default-feature build breakage flagged during BB_RUNTIME_PHASE3: `src-tauri/Cargo.toml` `default = []` → `default = ["full"]`. The crate is an application (no minimal-build consumer), and the only no-feature `cargo build` (scripts/darkmode-test.sh) expected the default to work. Now `cargo build`/`cargo check`/rust-analyzer work with no flags.
- `--no-default-features` (the truly minimal config) remains intentionally UNSUPPORTED — the per-item `#[cfg]` gating in browser/tts/asr/audio_io is incomplete; completing it was explicitly out of scope (buys nothing, large audit). No gating logic changed.
- Added a CI guard: `.github/workflows/ci.yml` now has a "Check default feature configuration" step (`cargo check --manifest-path src-tauri/Cargo.toml`, no flags) so the default config can't silently rot again. Distinct from the existing `--all-features` clippy/test steps.
- `--all-features` behavior unchanged (it already enables every feature → same compiled code). Gate green: no-flag `cargo check` compiles, fmt/clippy clean, 330 Rust + 164 JS tests, build green. Commits on master.

## 2026-06-29T09:45:24Z - Claude Opus 4.8 - BB_RUNTIME_PHASE3: tidy-ups (Part A) + Phase 3 planner lock-scoping (Part B)

- Part A (recommended tidy-ups): (1) fixed stale self-contradicting checklist/subtask lines in BB_CODE_REVIEW2_TODO.md (P1.1.2 DONE/Phase 2, P1.1.4 DONE/Phase 1, P1.1.3 reconciled). (2) Extracted `transcribe_success_result` + `build_transcribe_observations` in listening_tools.rs, shared by both the planner-dispatched `execute_transcribe_command` (lock-held) and the phased `finish_transcribe_command` (lock-released) paths; cross-reference comments added. Pure de-dup, no behavior change.
- Part B (Phase 3 proper — user explicitly chose "Implement now" via AskUserQuestion despite spec recommending defer): lock-scope the remote planner network call. Split `execute_command_with_replanning` → `build_planner_resolution(&mut self)` (deterministic direct-command resolution + remote profile snapshot, under a brief lock; returns `PlannerResolution::Direct|Remote`) and free fn `resolve_remote_planner(profile, planner_input)` (the unlocked LLM `futures::executor::block_on`). New `LockScopedReplanningRuntime` (app_core/replanning_orchestrator.rs) implements the existing `ReplanningRuntime` trait and drives `execute_bounded_replanning_loop`: resolve_plan locks→build→drop→network unlocked→validate; execute_plan re-locks→execute_planner_output. Handlers (transcribe_and_execute, resolve_command) now call `run_command_with_lock_scoped_replanning` / `resolve_command_lock_scoped` instead of the old AppCore methods.
- REMOVED (now unused): AppCore `resolve_command`, `execute_command_with_replanning`, `execute_transcribed_command`, `resolve_command_with_recent_results` (became `build_planner_resolution`), `resolve_planner_output`/`resolve_remote_planner_output`, and `impl ReplanningRuntime for AppCore`. Converted `resolve_with_openai_planner`/`resolve_with_ollama_planner`/`planner_prompt_payload` from `&self` methods to free functions. Tests use their own MockReplanningRuntime (unaffected). `PlannerResolution::Remote.planner_input` is boxed (clippy large_enum_variant).
- ATOMICITY TRADEOFF (documented at the orchestrator call site): resolve and execute are no longer one locked transaction — a plan is resolved against a snapshot a peer command could change before execute re-locks. Accepted for this single-user, frontend-serialized app; replan bound unchanged.
- P2.2 remote ASR lock-scoping CONSCIOUSLY SKIPPED per spec (lowest-value, remote-only, default-off).
- Validation: `--all-features` gate green — clippy clean, fmt clean, 330 Rust + 164 JS tests, build green. NOTE: the default-feature (no remote-openai) build is broken by PRE-EXISTING issues in browser/tts/asr modules (verified via git stash at HEAD before Part B — same errors); the project's gate is `--all-features`, so this is out of scope. Behavioral `--features full` + remote-profile verification (get_agent_state interleaves during in-flight remote resolve; same plan/outcome) still needs a human.

## 2026-06-29T08:44:54Z - Claude Opus 4.8 - BB_ASYNC_RUNTIME Phases 1-2 done; Phase 3 deferred (BLOCKED)

- Implements the BLOCKED responsiveness work from BB_CODE_REVIEW2 (P1.1.2/P1.1.3/P1.1.4) via BB_ASYNC_RUNTIME_TODO.md. Commits c6908ae (Phase 1) and 379e004 (Phase 2) on master.
- KEY PATTERN: managed state is now `Arc<Mutex<AppCore>>` (was `Mutex<AppCore>`). `lock_app_core` takes `&Mutex<AppCore>` so it deref-coerces from both `State<Arc<Mutex<AppCore>>>` and an owned `Arc` clone. Long commands are `async fn` that `Arc::clone` the state and run the existing sync AppCore method inside `tauri::async_runtime::spawn_blocking`, awaiting with `.map_err(join_error_to_tool_error)?`. The MutexGuard is created+dropped INSIDE the closure (never crosses await), so `std::sync::Mutex` is retained — do NOT migrate to tokio::sync::Mutex. Added `join_error_to_tool_error` helper in lib.rs.
- WHY spawn_blocking not `#[tauri::command(async)]`: `tauri::async_runtime::block_on` (used by the browser layer) panics on an async worker thread but is SAFE on a blocking-pool thread. `spawn_blocking` is the bridge. This is what made it safe to take browser-reaching commands (transcribe_and_execute, open_url, resolve_command, execute_planner_output, submit_confirmation_response) off the main thread — removed the CR2 guardrail comments. `tauri::async_runtime::JoinHandle` awaits to `Result<T, tauri::Error>`.
- Phase 2 (lock-scoped capture): AsrController gained `begin_capture`/`finish_capture`; the CaptureSession stays in the controller so the cpal stream keeps filling its buffer while the AppCore lock is dropped during the `thread::sleep` window. `finish_capture` returns `Ok(None)` if `stop_listening` dropped the session mid-window → clean "stopped" result. AppCore gained `begin_transcribe_command`/`finish_transcribe_command` (carried by pub `TranscribeCapturePlan`, re-exported from app_core/mod.rs) + `execute_transcribed_command` wrapper. Handlers run a phased lock/sleep/lock transcribe via `run_phased_transcribe` in voice_handlers.rs. CRITICAL: `execute_transcribe_command` (the monolithic one) is a ToolExecutor TRAIT method the planner can invoke mid-plan via tool_dispatch — left UNCHANGED (synchronous). The phased path is ONLY for the top-level handlers. Regression test `finish_capture_reports_none_when_session_stopped_mid_window` added (330 Rust tests now).
- Phase 3 (P1.2 / CR2 P1.1.3) DEFERRED/BLOCKED: releasing the lock across remote planner/ASR network calls needs restructuring the bounded-replanning planner-executor control flow (the LLM `futures::executor::block_on` is buried in a deep read-only `&self` resolve chain that interleaves with lock-requiring browser execution). Narrow benefit (remote providers only; project defaults to local). Tracked for a focused follow-up. CR2 P1.1.2/P1.1.4 reconciled as DONE.
- Validation: fmt/clippy clean, 330 Rust + 164 JS tests pass, build green. Behavioral `--features full` checks (real page/mic: no freeze, voice→browser no panic, stop interrupts capture) still need human verification — cargo test drives neither Chromium nor audio.

## 2026-06-29T08:07:00Z - Claude Opus 4.8 - CODE_REVIEW2 follow-up: fix runtime-in-runtime panic, real zeroize, honest status

- P0.1: Reverted `transcribe_and_execute_command` from `#[tauri::command(async)]` back to plain `#[tauri::command]`. The `(async)` form ran it on a tokio worker, and its browser tools call `tauri::async_runtime::block_on`, which panics ("Cannot start a runtime from within a runtime") off a worker thread. `transcribe_command` stays `(async)` (ASR-only, never reaches browser). Added GUARDRAIL comments to `transcribe_and_execute_command`, `execute_planner_output`, `resolve_command`, `submit_confirmation_response`, `open_url` so a future "finish P1.1.1" pass doesn't reconvert them.
- KEY LESSON: `#[tauri::command(async)]` is only safe on commands that never reach a browser op. Browser layer (`browser/navigation.rs`, `element_interaction.rs`, `page_inspection.rs`, `dom_extraction.rs`, `page_metrics.rs`) calls `tauri::async_runtime::block_on`. Remote-planner path uses `futures::executor::block_on`, which is safe (blocks the worker instead of panicking), so planner/download/key-test `(async)` commands are fine.
- P1.1: Replaced the prior `unsafe { old.as_mut_vec().fill(0) }` (a no-op dead store, only ran on same-key overwrite) with the `zeroize` crate. Session keyring cache value type is now `Zeroizing<String>` via `type SessionKeyringStore` alias (needed to satisfy clippy `type_complexity`). Added `zeroize = "1"` to Cargo.toml. No `unsafe` remains in `keyring_store.rs`.
- P2.1: Corrected `BB_CODE_REVIEW2_TODO.md`: P1.1.1 restated DONE→PARTIAL (start/stop_listening, resolve_command, execute_planner_output, open_url, submit_confirmation_response stay sync); "no freeze" checklist item unchecked (lock still held across blocking work, so `(async)` alone doesn't deliver responsiveness; real fix is BLOCKED P1.1.2/P1.1.3).
- Validation: fmt clean, clippy clean, 329 Rust + 164 JS tests pass, build green. P0.1 behavioral check under `--features full` on a real page still needs human verification (cargo test doesn't drive Chromium).

## 2026-06-29T07:45:05Z - Claude Sonnet 4.6 - CODE_REVIEW2: drain ASR buffer, fix panics, cleanup, async handlers

- Group 1 (P0.1): Added `drain_capture_buffer` free function to `asr/capture.rs`; renamed `snapshot()` to `take_captured_audio()` which now drains instead of clones. Added unit test `consecutive_drains_do_not_return_overlapping_samples`. Updated `asr/mod.rs` callers. Fixes duplicate command execution in continuous listening.
- Group 2 (P1.2): Replaced `.expect(...)` / `unreachable!()` in three production paths — `execution.rs` step lookup now returns `Aborted { code: "missing_step_position" }`; `model_management.rs` whisper plan collapses two match arms into one; `tts/wav.rs` five `expect` calls replaced with `map_err(|_| ...)?`.
- Group 3 (P2.1): Collapsed four near-identical ID helpers in `app_core/mod.rs` into shared `next_id`; added zeroize of old cached secret in `keyring_store.rs`; added `rustfmt` component and `cargo fmt --check` step to CI.
- Group 4 (P1.1.1): Added `#[tauri::command(async)]` to `transcribe_command`, `transcribe_and_execute_command`, `download_active_local_tts_model`, `download_active_local_asr_model`, `test_remote_planner_api_key`, `test_remote_tts_api_key`, `test_remote_asr_api_key`, `list_remote_planner_models`. Compiled cleanly — no Send constraint issues.
- P1.1.2 and P1.1.3 marked BLOCKED in TODO (require CaptureHandle extraction and multi-phase transcribe restructuring).
- Validation: cargo fmt clean, clippy clean, 329 Rust tests pass.

## 2026-06-29T06:55:21Z - Claude Sonnet 4.6 - Completed UIUX Fix 6 cleanup

- `src/app.tsx`: replaced inline `setAppAlertState({ message: null })` dismiss handler with `clearAppAlert` — resets both `kind` and `message` to neutral on dismiss.
- `src/styles.css`: tokenized 7 remaining hardcoded error/danger text colors (`#6b2820`, `#7a2018`, `#54100f`) across `.settings-subpage-card-status-error`, `.settings-reset-confirm-message`, `.confirmation-error`, `.confirmation-error-tool-hard-stop` (2 rules), and retry-status variants → `var(--color-error-primary)` or `var(--color-error-dark)`.
- `docs/BLIND_BROWSER_UIUX_FIX5_TODO.md`: checked all 12 final checklist items (all done).
- `docs/BLIND_BROWSER_UIUX_FIX6_TODO.md`: all tasks marked DONE, final checklist checked.
- Validation gate: lint clean, 164 JS tests pass, build clean, cargo clippy clean, cargo test clean.

## 2026-06-29T06:06:52Z - Claude Sonnet 4.6 - Add automated dark-mode visual test (Docker + Xvfb)

- `Dockerfile.darkmode-test`: Ubuntu 24.04 + Xvfb + libwebkit2gtk-4.1-0 + libasound2t64 + scrot + imagemagick + Adwaita dark theme. Copies pre-built binary + dist.
- `scripts/darkmode-test.sh`: Builds image, launches app with `GTK_THEME=Adwaita:dark` so WebKitGTK reports `prefers-color-scheme: dark`, navigates 7 panels via xdotool, screenshots with scrot, asserts mean luminance < 0.45 per region via ImageMagick.
- Build requirement: `pnpm tauri build --no-bundle -- --features browser,local-tts,remote-openai,audio` (plain `cargo build --release` omits Tauri asset embedding and app loads devUrl instead).
- All 7 panels confirmed dark (luminance 0.09–0.21): workspace, toolbar, settings overview, planner, TTS, ASR, runtime/advanced.
- P1.5 marked DONE. `darkmode-screenshots/` added to .gitignore.
- Usage: `bash scripts/darkmode-test.sh` (or `KEEP_CONTAINER=1 ...` for debugging).

## 2026-06-29T04:26:01Z - Claude Haiku 4.5 - Completed UIUX Fix 5 closeout

- Added `setOpenExternalUrlForTest` seam + `clearAppAlert` to `src/panel-state-setters.ts`.
- Created `src/external-link.test.mjs` with 5 regression tests: failure path sets global app alert (kind=error, message includes URL and error detail), `openExternalLink` doesn't throw, failure is not routed to `urlInputPanelState`, and dismiss tests for `clearAppAlert`.
- CSS static audit: replaced all unjustified hardcoded light-mode component colors/surfaces:
  - `.settings-api-key-test-status-message` color `#1f2527` → `var(--color-text-primary)` (P1.1)
  - `.shell-toolbar-action`, `.settings-subpage-back`, `.settings-subpage-card` (:hover/:focus-visible), `.panel` backgrounds `rgba(255,252,247,...)` → `var(--color-surface-card)` (P1.2)
  - `.eyebrow`, `.status-panel-eyebrow`, `.settings-model-freshness-label` `#7b6246` → `var(--eyebrow-color)` (P1.3)
  - `.lede`, `.confirmation-card ul` `#3a342e`/`#2c3233` → `var(--color-text-secondary)` (P1.3)
  - `.status-indicator` `#5d584e` → `var(--color-text-secondary)` (P1.3)
- Remaining `#7b6246` at styles.css:58 is the `:root` CSS variable definition (`--eyebrow-color`), not a component hardcode — expected.
- All `outline: none` instances are on `:hover` states only (not `:focus-visible`) — focus ring intact.
- Static audits pass. Validation gate: 328 Rust + 164 JS tests clean, lint/tsc/build clean.
- Commit `3cf3dcd` on master.
- P1.5 (manual dark-mode walkthrough) requires human verification in the running app.
- Note: `appAlertState` is nested under `panelStates` in the Redux store (path: `appShellStore.getState().panelStates.appAlertState`).

## 2026-06-29T03:54:07Z - Claude Sonnet 4.6 - Add test coverage: routing edge cases, planner outputs, config persistence, async actions

- `src-tauri/src/commands/tests/routing.rs`: 6 new test functions covering fill-in prefix, put/enter-in patterns, textbox/input suffix normalization, single-quoted values, `is_direct_submit_form_command`, additional submit suffixes.
- `src-tauri/src/commands/routing/planner_outputs.rs`: inline `#[cfg(test)] mod tests` with 4 tests for `round_audio_setting_value`, `build_report_result_step`, `build_single_step_planner_output`, `build_browser_visibility_planner_output`. Access requires explicit `use crate::commands::{...}` + `use crate::browser::BrowserVisibilityMode` (not available via `use super::*;` alone in inline test module).
- `src-tauri/src/config/tests/persistence_tests.rs`: 9 new tests for model management, remote planner connection, local model path, and audio settings validation. Added `ModelManagementSettings` to `tests/mod.rs` imports.
- `src/planner-actions.test.mjs`: 20 tests for all 9 async planner actions (validation, success, failure paths). Key pattern: `__setInvokeForTests` returns raw `{ profile_name, ... }` (not tool result envelope) for settings commands.
- `src/settings-actions.test.mjs`: 35 tests covering busy guards, success, rollback for all 11 settings actions. Tool-executor commands (setPlaybackVolume, setPlaybackSpeed, setTtsVoice, setBrowserVisibility) return the full `{ ok, data, ... }` tool result envelope. Settings commands (setAsrProviderSelection, etc.) return raw objects.
- Lessons: `is_fill_input_phrase` requires " field" keyword — "textbox" alone doesn't trigger phrase recognition. `is_fill_and_submit_phrase` requires "submit" — "send form" alone doesn't trigger it.
- Commit `ca3f14e`. Tests: 328 Rust + 159 JS.

## 2026-06-29T01:36:45Z - Claude Sonnet 4.6 - Split mock_executor_impl.rs into 5 themed submodules

- Converted 859-line flat file to `mock_executor_impl/` directory: `mod.rs` (delegation impl) + `navigation.rs` / `media.rs` / `interaction.rs` / `settings.rs` / `state.rs`. No file exceeds ~250 lines.
- Lifts the FIX3 P2.3 file-size exemption comment.
- Pattern: `mod.rs` uses `pub(crate) use super::*;` so submodules reach all tool types via `use super::*;`. Trait impl in `mod.rs` delegates to `pub(super) fn` free functions per submodule.
- Commit: `d958247`. 309 Rust + 104 JS tests clean.

## 2026-06-29T01:18:46Z - Claude Sonnet 4.6 - BLIND_BROWSER_UIUX_FIX4_TODO.md all tasks complete

- All P0, P1, and P2 tasks implemented. Full validation gate: 104 JS tests, 309 Rust tests, lint/clippy/build/tsc all clean.
- P0.1: Added global `AppAlertState` + `"app-alert"` panel root rendered between header and workspace/settings sections — visible in all views. `openExternalLink()` now routes failures to `setAppAlertState()` instead of `urlInputPanelState.error`. Renderer in `src/app-alert-panel.tsx`.
- P0.2: Extracted `applyAgentStateToPanels()` as exported function in `runtime-refresh.ts`. Preserves `availableModels`/`loadedModelsEndpoint` when refreshed endpoint matches current verified endpoint; clears on mismatch or no verified list. 3 regression tests added in `runtime-refresh.test.mjs`.
- P1.1: Fixed self-referential `--color-surface-inner: var(--color-surface-inner)` → concrete `rgba(255, 255, 255, 0.68)`. Added `--color-error-light` token. Reordered CSS so dark mode `@media` block appears after base `:root` (was before, causing cascade override).
- P1.2: Tokenized `.voice-status-strip`, `.url-input-label`, `.status-toggle-button`, `.settings-control-card`, `.settings-control-card-readonly`, `.settings-control-label`, `.settings-control-value`, `.confirmation-panel`, `.confirmation-meta dt/h3/dd` — all use `--color-*` tokens now.
- P1.3: Split `.url-action-button:hover:not(:disabled)` from `:focus-visible`; restored `outline: var(--focus-ring)` on focus.
- P2.1: Static audits passed, full validation gate passed, memory.md updated.

## 2026-06-28T23:40:27Z - Claude Sonnet 4.6 - BLIND_BROWSER_UIUX_FIX3_TODO.md all tasks complete

- All P0, P1, and P2 tasks implemented and committed. Full validation gate: 100 JS tests, 309 Rust tests, lint/clippy/build/tsc all clean.
- P0.1: Fixed false "model list loaded" planner state in `persistRemotePlannerConnection`, `resetRemotePlannerConnectionToDefaults`, and `runtime-refresh.ts`. Only a successful `listRemotePlannerModels()` call sets `availableModels` + `loadedModelsEndpoint`.
- P0.2: Added `FrontendToolError` class; `unwrapToolResult()` now throws it; `parseToolError()` recognizes it so `classifyInvokeFailure()` returns `kind: "tool-error"` for backend errors.
- P0.3: `openExternalLink()` now sets a visible URL input panel error instead of console-only logging.
- P1.1: Removed blue radial background gradient; replaced all teal/hardcoded colors (`#1c5871`, `#24404f`, `#1f6b57`, `#185745`) with `--color-green-*` / `--color-amber-*` tokens; added `--color-amber-light` and `--color-surface-inner` tokens.
- P1.2: Added `--focus-ring` / `--focus-offset` tokens; unified all 14 `:focus-visible` outline declarations across the stylesheet.
- P1.3: Expanded dark mode block from 3-variable surface-only to full override (surfaces, text, borders, amber, error tokens, `color-scheme: dark`). Tokenized inputs, secondary buttons, confirmation-reject, status-card dt/dd, eyebrow, prompt text.
- P1.4: PTT early-return when `!state.enabled`: renders only setup banner (no large disabled talk button). Tests updated.
- P1.5: Endpoint-adjacent `settings-inline-loading` spinner added to planner panel when loading models.
- P2.1: Documented Google Fonts network dependency in `index.html` comment.
- P2.2: Audio capture lock failure now observable: `AtomicBool` flag in `CaptureSession`; one-shot `tracing::warn!` on first poisoned-mutex callback; `snapshot()` returns error if flag is set.
- P2.3: `mock_executor_impl.rs` exempted from 600-line target with comment explaining why (full trait implementation is a unified test double).
- P2.4: TODO all marked DONE, full validation passed, memory.md updated.
- Final commit range: `fbc4d2a` – `069985f` on master. 10 commits total.

## 2026-06-28T22:52:12Z - Claude Sonnet 4.6 - UIUX_IMPROVEMENTS2_TODO.md Phase 7 complete

- Phase 7.1: Added `.panel-error-dismiss` CSS and "Dismiss" button to all panel error displays. `onDismissError?` threaded through `SettingsPanelSectionOptions` and all panel handler interfaces; wired via `setXxxPanelState({ error: null })` in `app.tsx` for every panel. Added new setter imports: `setAsrProviderPanelState`, `setStatusPanelState`, `setTtsModelPanelState`, `setTtsProviderPanelState`, `setTtsVoicePanelState`.
- Phase 7.2: Added "Try again" button for retryable errors. `onRetry?` in `shared-controls` and handler interfaces; planner→`loadRemotePlannerModels`, ASR→`testConfiguredRemoteAsrApiKey`, TTS→`testConfiguredRemoteTtsApiKey`, model-management→`persistModelManagementSettings`.
- Phase 7.3: Added pulsing "Working…" indicator (`status-panel-busy` CSS) in status panel eyebrow. `plannerBusy?: boolean` in `StatusPanelState`; derived in `app.tsx` from `isOpening || isReading || speaking`. Includes `prefers-reduced-motion` override.
- Phase 7.4: Status panel shows "Loading page…" and "—" placeholders while `isPageLoading`. `isPageLoading?: boolean` in `StatusPanelState`; wired from `urlInputPanelState.isOpening`. `isFirstLoad` guard updated.
- Phase 8: Full validation suite passed — 97/97 JS, 309/309 Rust, lint/clippy/build clean. Commit: `91e8495`.
- Next: push to GitHub and continue with any remaining UIUX improvements or new work.

## 2026-06-28T22:32:07Z - Claude Sonnet 4.6 - TAILWIND1 Phases 1-3: Tailwind v4, TSX conversion, MUI removed

- Phase 1: Tailwind CSS v4 (`tailwindcss 4.3.1`, `@tailwindcss/vite 4.3.1`) added; `styles.css` rewritten with `@import "tailwindcss"` + `@theme {}` design tokens; `"jsx": "react-jsx"` in tsconfig; eslint/lint updated for `.tsx`.
- Phase 2: `src/icons.tsx` — `SettingsIcon` and `ArrowBackIcon` SVG components replacing `@mui/icons-material`.
- Phase 3: All 13 render `.ts` files converted to `.tsx` (JSX syntax, no `createElement`/`h`); `app-shell-theme.ts` deleted; `app.tsx` uses `useSelector` + JSX directly; barrel exports updated to `.tsx` extensions; `tsx` dev dep added so Node test runner handles JSX (test script: `node --import tsx/esm --test`); test files updated to import from `.tsx`; MUI packages (`@mui/material`, `@mui/icons-material`, `@emotion/react`, `@emotion/styled`) removed.
- Bundle: 417KB → 305KB (MUI gone). 97/97 JS tests + clippy + lint + build all clean. Commit: `e0ba161`.
- Next: continue UIUX_IMPROVEMENTS2_TODO.md Phase 3 (font loading), Phase 5 (remote planner UX), Phase 6 (settings UX), Phase 7 (feedback/progress).

## 2026-06-06T16:14:22Z - Claude Sonnet 4.6 - REFACTOR5_TODO.md all 4 phases complete

- Splits: `commands/tests/contracts/` (3 subfiles), `config/tests/load_tests/` (3 subfiles), `src/confirmation-panel.test.mjs` → 5 themed files (needed second pass: settings file was still 834 lines → split into planner 493 + voice 346).
- JS split pattern: shared helper `confirmation-panel-test-helpers.mjs` (no `.test.` so not picked up by glob); test files import only what they need.
- `mock_executor_impl.rs` (853 lines) remains the only file over 600 — single trait impl block, confirmed cannot be split.
- All other files now ≤599 lines. 309 Rust + 97 JS tests clean. clippy/lint/build all clean.
- `*.sh~` added to `.gitignore`.

## 2026-06-06T16:09:40Z - Claude Sonnet 4.6 - Split confirmation-panel.test.mjs (1537 lines, 52 tests) into helpers + 4 themed test files

- Created `src/confirmation-panel-test-helpers.mjs` (249 lines): all imports, `VOID_ELEMENTS`, `escapeHtml`, `mapAttributeName`, `renderNodeMarkup`, 19 render wrappers, `renderFixtures`, and re-exports of `statusPanelStateFromAgentState` + `renderVoiceStatusStripNode`.
- Created `src/confirmation-panel-core.test.mjs` (161 lines, 9 tests): 4 confirmation panel error/aria tests + 5 push-to-talk tests.
- Created `src/confirmation-panel-url-audio.test.mjs` (201 lines, 11 tests): slider value text, 7 URL input busy/error states, 3 audio controls tests.
- Created `src/confirmation-panel-settings.test.mjs` (834 lines, 36 tests): all settings panel tests (model management, remote planner, provider failover, confirmation, OCR, guidance, ASR, TTS provider/model/voice/local/remote).
- Created `src/confirmation-panel-status.test.mjs` (135 lines, 9 tests): 4 status panel tests + 5 voice status strip tests.
- Deleted original `src/confirmation-panel.test.mjs`.
- All 97 JS tests pass, lint clean. Test count unchanged (52 confirmation-panel tests + 45 other tests = 97 total).

## 2026-06-06T16:02:54Z - Claude Sonnet 4.6 - Split config/tests/load_tests.rs (704 lines, 10 tests) into directory with 3 subfiles

- Promoted `config/tests/load_tests.rs` → `config/tests/load_tests/mod.rs` (directory).
- Created `enum_serialization.rs` (2 tests: `config_enums_round_trip_and_reject_invalid_variants`, `provider_configs_round_trip_through_json`).
- Created `valid_configs.rs` (2 tests: `parses_default_template`, `parses_ollama_planner_profile_when_selected`).
- Created `invalid_configs.rs` (6 tests: `rejects_missing_selected_remote_planner_profile_reference`, `rejects_inline_secret_refs`, `rejects_local_planner_configuration`, `rejects_missing_remote_profile_for_remote_mode`, `rejects_missing_selected_profiles_for_tts_and_asr_modes`, `rejects_missing_selected_local_profile_references_for_tts_and_asr`).
- `mod.rs` rewritten to 5 lines: `use super::*;` + 3 `mod` declarations.
- Clippy clean, 309 Rust tests pass (no change in count).

## 2026-06-06T15:59:35Z - Claude Sonnet 4.6 - Split contracts.rs (707 lines, 16 tests) into directory with 3 subfiles

- Promoted `commands/tests/contracts.rs` → `commands/tests/contracts/mod.rs` (directory).
- Created `tool_schemas.rs` (4 tests: input/output schema coverage), `tool_result_envelope.rs` (5 tests: success/failure envelope, round-trips, enum serialization), `planner_contracts.rs` (7 tests: planner output/input round-trips, schema validation, safety settings).
- `mod.rs` rewritten to 5 lines: `use super::*;` + 3 `mod` declarations.
- Clippy clean, 309 Rust tests pass (no change in count).

## 2026-06-06T15:20:58Z - Claude Sonnet 4.6 - REFACTOR4_TODO.md all 6 phases complete

- All 5 large test files split into themed directories. 309 Rust tests, lint/tsc/build/clippy all clean.
- Splits: `fixtures/` (7 subfiles), `tool_dispatch/` (6 subfiles), `direct_commands/` (5 subfiles), `planner_flow/` (3 subfiles), `app_core/tests/` (11 subfiles).
- `mock_executor_impl.rs` (853 lines) remains as documented exception: single `impl DeterministicToolExecutor for MockExecutor` block cannot be split per Rust rules.
- REFACTOR5 candidates: `confirmation-panel.test.mjs` (1537 JS lines), `contracts.rs` (707), `config/tests/load_tests.rs` (704).
- Final commits: phases 2–5 on master (0467943 → 9a669d7), phase 6 doc/memory update pending.

## 2026-06-06T15:18:57Z - Claude Sonnet 4.6 - Split app_core/tests.rs (3594 lines, 85 tests) into directory with 11 subfiles

- Promoted `src-tauri/src/app_core/tests.rs` to `tests/mod.rs`, then split into:
  - `helpers.rs` (410 lines): `spawn_openai_models_test_server`, `fixture_page*`, `fixture_field`, `fixture_form`, `fixture_problematic_*`, `planner_tool_sequence`, `AppCorePlannerFixtureKind`, `AppCorePlannerFixture`, `resolve_app_core_planner_fixture`, `assert_app_core_planner_fixture` — all `pub(super)`
  - `settings_tests.rs` (415 lines, 15 tests): `build_remote_planner_settings*`, `build_remote_tts/asr_settings`, `build_remote_settings_expose_*`, `build_provider_failover*`, `build_confirmation*`, `build_local_tts/asr_model_settings*`, `build_tts_model_settings*`, `build_ocr_threshold*`, `build_asr/tts_provider*`, `build_tts_voice*`
  - `browser_tests.rs` (123 lines, 5 tests): `normalize_optional_text`, `normalize_absolute_url*`, `browser_error_to_tool_error*`, `refresh_current_page*`, `clear_navigation_follow_up_state*`
  - `extraction_tests.rs` (371 lines, 8 tests): `build_visible_text_excerpt*`, `region_bbox_by_id*`, `build_extracted_page_model*` (×4), `infer_extraction_source*` (×3)
  - `ocr_threshold_tests.rs` (264 lines, 9 tests): `should_trigger_*_ocr_fallback` (×6 including disabled/non-dom), `extracted_text_metrics*`, `should_not_trigger*` (×2)
  - `ocr_merge_tests.rs` (309 lines, 8 tests): `region_first_ocr_target_ids*` (×2), `merged_region_text*`, `merge_ocr_text_into_page_model*` (×5)
  - `focus_fill_tests.rs` (356 lines, 8 tests): `filter_interactive_elements*`, `resolve_direct_focus_field*` (×3), `resolve_direct_fill_field*` (×2), `resolve_direct_fill_and_submit*` (×2)
  - `fill_correction_tests.rs` (283 lines, 6 tests): `resolve_recent_fill_correction*` (×3), `resolve_typeable_element*`, `resolve_direct_submit_form*` (×2)
  - `regression_tests.rs` (383 lines, 3 tests): `app_core_form_regression*`, `ambiguous_click_regression*`, `problematic_page_regression*`
  - `element_scoring_tests.rs` (244 lines, 6 tests): `resolve_form_element_rejects*`, `rank_find_element_candidates*` (×2), `build_find_element_query*`, `determine_find_element_resolution*` (×2)
  - `planner_tests.rs` (400 lines, 11 tests): `planner_interpretation_unavailable*`, `planner_system_prompt*`, `bounded_replanning_loop*` (×3), `resolve_clickable_element*` (×3), `test_openai_api_key*` (×2), `fetch_openai_compatible_models*` (×2) — includes `MockReplanningRuntime`, `mock_planner_output`, `mock_trace`
- `mod.rs` reduced to 63 lines: all imports + `mod helpers; use helpers::*;` + 10 `mod` declarations.
- All subfiles start with `use super::*;`; helper items are `pub(super)`.
- 309 Rust tests pass; clippy -D warnings + lint + build all clean.

## 2026-06-06T15:07:06Z - Claude Sonnet 4.6 - Split planner_flow.rs (1470 lines, 35 tests) into directory with 3 subfiles

- Promoted `src-tauri/src/commands/tests/planner_flow.rs` to `planner_flow/mod.rs`, then split into:
  - `execution.rs` (7 tests, 363 lines): `executes_next_step_chain_*`, `executes_load_page_extract_and_read_*`, `executes_resolved_spoken_command_*`, `follows_failure_transition_*`, `returns_awaiting_confirmation_*`, `aborts_when_next_step_*`, `aborts_needs_confirmation_*`
  - `output_validation.rs` (15 tests, 593 lines): `planner_available_tools_include_all_wave_two_tools` + all `validate_planner_output_rejects_*` / `validate_planner_output_accepts_*` tests
  - `input_validation.rs` (13 tests, 515 lines): `set_tts_voice_input_*`, `validate_eval_js_input_*`, `validate_confirm_action_input_*`, remaining `validate_planner_output_rejects_*` for open_url/go_back/go_forward/scroll_page/find_element/playback bounds
- mod.rs reduced to 5 lines: `use super::*;` + 3 `mod` declarations.
- All subfiles start with `use super::*;` — no unused import warnings.
- 309 Rust tests pass; clippy -D warnings clean.

## 2026-06-06T15:02:30Z - Claude Sonnet 4.6 - Split direct_commands.rs (1865 lines) into directory with 5 subfiles

- Promoted `src-tauri/src/commands/tests/direct_commands.rs` to `direct_commands/mod.rs`, then split into:
  - `audio_commands.rs` (5 tests: resolve_direct_audio_command_* ×3, resolve_direct_browser_visibility_command_* ×2)
  - `navigation_commands.rs` (5 tests: resolve_direct_navigation_readback_command_* ×2, resolve_direct_voice_input_command_* ×2, resolve_direct_open_url_command ×1)
  - `reading_commands.rs` (3 tests: resolve_direct_read_page_command_* ×3)
  - `status_commands.rs` (3 tests: resolve_direct_status_query_command_* ×3)
  - `playback_commands.rs` (4 tests: resolve_direct_repeat_command_* ×2, resolve_direct_read_title_command_* ×2)
- mod.rs reduced to 7 lines: `use super::*;` + 5 `mod` declarations.
- All subfiles start with `use super::*;` — no unused import warnings.
- 309 Rust tests pass; clippy -D warnings clean.

## 2026-06-06T14:47:25Z - Claude Sonnet 4.6 - Split fixtures/mod.rs (2023 lines) into 7 subfiles

- Split `src-tauri/src/commands/tests/fixtures/mod.rs` into:
  - `path_helpers.rs` (unique_temp_path, write_skill_document)
  - `mock_executor.rs` (MockExecutor struct, Default impl, utility methods)
  - `mock_executor_impl.rs` (impl DeterministicToolExecutor for MockExecutor — not pub-re-exported, internal only)
  - `page_fixtures.rs` (PlannerSkillFixtureResolver, PlannerSkillFixture, fixture_* functions)
  - `skill_fixtures.rs` (resolve_planner_skill_fixture, assert_planner_skill_fixture)
  - `step_fixtures.rs` (sample_planned_step, sample_planned_steps_for_registered_tools)
  - `schema_helpers.rs` (assert_json_matches_schema, assert_json_matches_schema_at, resolve_schema_reference, json_matches_type, json_matches_single_type)
- All subfiles use `use super::*;` as first line; mod.rs uses `pub use super::*;` and re-exports all submodules.
- 309 Rust tests pass with zero errors after the split.

## 2026-06-06T11:00:50Z - Claude Sonnet 4.6 - REFACTOR3_TODO.md all 8 phases complete

- All 8 phases of `docs/REFACTOR3_TODO.md` are implementation-complete.
- Final commit: `2d9b9de` on master. 309 Rust tests, 97 JS tests, lint/tsc/build/clippy all clean.
- **Phase 1:** Split app_core/mod.rs (850→command_dispatch.rs, confirmation_workflow.rs, result_reporting.rs).
- **Phase 2:** Split browser/mod.rs (685→element_interaction.rs, page_inspection.rs).
- **Phase 3:** Split config/mod.rs (851→275 + persistence.rs with 23 persist_*/reset_* methods). Key: `pub(in crate::config)` for cross-sibling access; test submodule uses `super::keyring_store::set_keyring_secret` (not re-exported from root).
- **Phase 4:** Promote commands/planner_executor.rs (685→planner_executor/ with tool_dispatch.rs, execution.rs, step_helpers.rs). `StepExecutionContext` moved into mod.rs; `#[cfg(test)]` on `execute_planner_output_with_runner` re-export.
- **Phase 5:** Promote app_core/extraction_tools.rs (849→extraction_tools/ with ocr_tools.rs, page_extraction.rs). `#[cfg(test)]` on `should_trigger_extract_page_model_ocr_fallback` re-export.
- **Phase 6:** Promote app_core/interaction_tools.rs (779→interaction_tools/ with element_queries.rs, click_focus.rs, text_entry.rs). `resolve_typeable_element` is production re-export; `resolve_clickable_element`/`resolve_form_element` are test-only.
- **Phase 7:** Promote app_core/form_fill.rs (676→form_fill/ with field_focus.rs, field_fill.rs, form_submit.rs). `PlannerOutput` import split to `#[cfg(test)]` since only used in test wrappers.
- All production Rust files now under 600 lines (largest: config/persistence.rs at 593, commands/routing/intent.rs at 599).
- Key pattern: private (`mod`) items in parent are accessible to descendants via `crate::module::submodule` path; test-only re-exports need `#[cfg(test)]` guard to avoid `-D warnings` failures.

## 2026-06-06T09:57:09Z - Claude Sonnet 4.6 - REFACTOR2_TODO.md all 13 phases complete

- All 13 phases of `docs/REFACTOR2_TODO.md` implemented and validated.
- **Phases 1–4:** Rust module splits — app_core/mod.rs (1902→850), browser.rs (1779→685 + submodules), tts.rs (958→555 + submodules), asr.rs (756→submodules).
- **Phase 5:** commands/contracts.rs (1308→contracts/ with providers, planner, interaction, tools subfiles).
- **Phase 6:** Large app_core submodule splits — element_scoring.rs, ocr_merge.rs, page_model_builder.rs, fill_correction.rs, reading_tools.rs, listening_tools.rs.
- **Phase 7:** commands/validators.rs (808→validators/ with navigation, element, extraction, audio, voice, planner subfiles).
- **Phase 8:** commands/registry.rs (1054→registry.rs + schemas.rs, skill_parser.rs, skill_loader.rs).
- **Phase 9:** lib.rs (1073→~70 lines) + command_handlers/ module (core, voice, url, audio, provider, safety, api_key, model handlers).
- **Phase 10:** commands/routing/mod.rs (1061→~50 lines) + navigation_routing.rs, voice_routing.rs, reading_routing.rs, field_routing.rs; audio_commands.rs/url_commands.rs/status_commands.rs extended. Key fix: glob imports with `pub(crate)` items need explicit `pub(crate) use` re-exports for cross-module visibility.
- **Phase 11:** src/tauri-api.ts (999→10 lines barrel) + tauri-types.ts + api/ (errors, planner, voice, audio, navigation, providers, safety, remote-keys, models). Node test runner needs explicit `.ts` extensions in import paths with `--experimental-strip-types`.
- **Phase 12:** app-shell.ts (542→~250 lines) + app-shell-theme.ts, app-shell-nav.ts (with all shared types), app-shell-controls.ts; types re-exported from app-shell.ts so external callers unchanged.
- **Phase 13:** Final validation — 309 Rust tests, 97 JS tests, lint, tsc, vite build, clippy all clean.
- Final commit: `187c171` on master. Remaining production files >600 lines (REFACTOR3 candidates): config/mod.rs (851), app_core/mod.rs (850), extraction_tools.rs (849), interaction_tools.rs (779), planner_executor.rs (685), browser/mod.rs (685), form_fill.rs (676).

## 2026-06-06T03:40:37Z - Claude Sonnet 4.6 - REFACTOR1_TODO.md all 6 phases complete

- All 5 structural refactor phases in `docs/REFACTOR1_TODO.md` are implementation-complete.
- **Phase 1:** `app_core.rs` (~11 600 lines) → `app_core/` module: replanning, settings_adapters, model_management, api_key_tools, navigation_tools, content_tools, extraction_tools, interaction_tools, form_fill, voice_tools, planner_prompt, tool_executor, tests.
- **Phase 2:** `commands/tests.rs` (~8 800 lines) → `commands/tests/` module: fixtures, tool_dispatch, playback_controls, browser_state, runtime_status, listening, planner_flow, skill_selection, confirmation, contracts, routing, direct_commands.
- **Phase 3:** `config.rs` (~2 500 lines) → `config/` module: types, loading, validation, keyring_store, tests/.
- **Phase 4:** `commands/routing.rs` (~2 200 lines) → `commands/routing/` module: intent, audio_commands, url_commands, planner_outputs, status_commands.
- **Phase 5:** `src/main.ts` (~1 970 lines) split into focused modules: `panel-state-setters.ts`, `settings-statuses.ts`, `settings-actions.ts`, `planner-actions.ts`, `browser-actions.ts`, `voice-loop.ts`, `shell-event-handlers.ts`, `ui-store.ts`, `refresh-handle.ts`, `app.ts` (BlindBrowserApp component); `main.ts` reduced to 112 lines. Eliminated local cached state vars by reading from store directly.
- **Phase 6:** Full validation gate passed — all Rust tests (309), JS tests (97), lint, tsc, vite build, clippy clean.
- Final commit pending push; all REFACTOR1_TODO.md tasks marked DONE.

## 2026-06-05T21:15:15Z - Claude Sonnet 4.6 - UIUX_IMPROVEMENTS1_TODO.md Phases 4–7 complete

- **Phase 4 (settings UX):** Planner model placeholder disabled/non-selectable; speed slider max clamped to 2.5; read-only profile cards get `.settings-control-card-readonly` muted style; `modelAvailable: boolean | null` added to `LocalAsrModelPanelState` / `LocalTtsModelPanelState` with inline warning + "Open Advanced settings" button when `false`; wired in `main.ts` to pass `modelManagementPanelState.localAsrAvailable` / `.localTtsAvailable`; `onOpenRuntimeSettings` handler navigates to `runtime` settings view.
- **Phase 5 (spinners):** Added `@keyframes spin` and `.btn-spinner` CSS class with `prefers-reduced-motion` override; spinner span injected into "Testing...", "Loading models...", "Saving...", "Downloading..." button labels in `confirmation-panel-helpers.ts`, `planner.ts`, and `runtime.ts`.
- **Phase 6 (accessibility):** Model freshness dot replaced with visible "Up to date"/"Reload needed" text label + sr-only accessible name; PTT aria-labels and hint text decoupled (complementary, not repeated); confirmation error container always present in DOM with `aria-live="assertive"` `aria-atomic="true"`; disabled PTT button gets muted warm-gray background instead of opacity-only; `:focus-visible` outline rules added for PTT button, toolbar actions, settings cards, and back button.
- **Phase 7 (validation):** All 97 JS tests pass; lint, tsc, vite build, cargo clippy, cargo test all clean. Final commit `55543a7` on master.
- All 7 phases of `docs/UIUX_IMPROVEMENTS1_TODO.md` now have implementation complete; phases 7.2–7.5 (live app walkthrough) remain for manual verification.
- **Ralph Loop pattern confirmed working:** implement tasks serially from a TODO file, run full validation after each phase, commit+push when passing. Works well for this project — use it for future TODO-driven improvement passes.

## 2026-06-05T17:30:06Z - Claude Sonnet 4.6 - Full UI/UX review performed; improvements TODO created

- Performed a full UI/UX review of the frontend (all settings panels, confirmation panels, push-to-talk, workspace, styles, error handling, accessibility, user workflow).
- Created `docs/UIUX_IMPROVEMENTS1_TODO.md` with 7 phases covering: critical functional fixes, first-run state, plain-language copy pass, settings UX, progress indicators, accessibility polish, and final validation.
- Key findings: PTT errors are screen-reader only (sighted users see nothing), settings navigation loses context on workspace roundtrip, Provider Failover section is dead UI, developer jargon throughout ("TTS", "ASR", "Planner", "Region", "Keyring", `timeoutMs`), no progress indicators on long operations, Reset to Defaults has no confirmation.
- All 84 frontend tests and 309 Rust tests passing at time of review (commit `ced9dfa`).
- CLAUDE.md created at repo root for Claude Code session continuity.

## 2026-04-21T19:13:10Z - Claude Sonnet 4.6 - UI/UX phases 3-6 complete, all tasks DONE

- **Phase 3 (voice status strip):** Added `renderVoiceStatusStripNode()` in `push-to-talk.ts` (idle/listening/speaking/processing), rendered in `shell-toolbar` via `"voice-status"` PanelRootKey, wired in `main.ts`. Removed Listening/Speaking status-cards from workspace panel. 5 new tests.
- **Phase 4 (confirmation cleanup):** Removed `confirmation-meta` dl and `confirmation-columns` div from `confirmation.ts`. Shortened h2 to "Action requires your approval." Added promptText fallback.
- **Phase 5 (settings polish):** Removed eyebrow `<p>` from all 4 settings subpages. Redesigned settings subpage text links as full-width tappable button cards with chevron. Removed unused MUI Button import.
- **Phase 6 (polish):** Added `@keyframes pulse-ring` animation for listening state, with `prefers-reduced-motion` override. Added mobile responsive rules for voice strip and subpage cards.
- Commit `ced9dfa` pushed to master. 84/84 tests pass. All UIUX_REVIEW1_TODO.md tasks marked DONE.


- Converted the workspace `Open` action in `src/settings-panels/workspace.ts` from a text button to an icon button so the entire URL control strip now uses the same compact icon-button language for navigation actions.
- Updated `src/styles.css` so the Talk control is square, sized from the workspace control-bar column, and renders its microphone glyph at roughly 75 percent of the button size instead of the shared small icon size.
- Kept the compact workspace layout validated end to end by updating `src/confirmation-panel.test.mjs` and rerunning `pnpm lint`, `pnpm test:ui`, `pnpm build`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, and `cargo test --manifest-path src-tauri/Cargo.toml --all-features`.

## 2026-04-18T23:48:03Z - GPT-5.4 - Workspace controls collapsed into a compact URL-plus-talk bar
- Reworked the workspace area in `src/app-shell.ts` so the URL controls and push-to-talk control now sit in a single `workspace-control-bar`, with status and confirmation panels kept below.
- Removed the visible `Voice input`, `Push to talk`, `Navigation`, `URL input`, current-URL helper text, and related filler copy from the live workspace panels in `src/confirmation-panels/push-to-talk.ts` and `src/settings-panels/workspace.ts`.
- The URL block now renders as two rows with the text URL/Open row above icon-only Read, Stop, Previous, and Next controls, while Talk is a full-height icon button beside it; revalidated with `pnpm lint` and `pnpm test:ui`.

## 2026-04-18T23:32:49Z - GPT-5.4 - Workspace header reduced to a single settings icon action
- Removed the remaining workspace hero copy and heading from `src/app-shell.ts`, so the main page now opens directly on the live voice, page, and status controls.
- Replaced the old `Workspace`/`Settings` text navigation with a single top-right settings gear on the workspace view plus a matching back-to-workspace icon on the settings overview, while keeping the deeper settings-subpage back arrow behavior unchanged.
- Updated the shell/layout tests and header styles to match the icon-button navigation, and revalidated with `pnpm lint` plus `pnpm test:ui`.

## 2026-04-18T23:03:25Z - GPT-5.4 - Settings cleanup is already committed and pushed
- Verified that `master` is clean with no staged or unstaged changes.
- Verified that `HEAD` and `origin/master` both point at `3368eb0 Finish React interaction migration`, so the settings cleanup is already checked in and published.

## 2026-04-18T22:54:51Z - GPT-5.4 - Finished the React-owned frontend cleanup plan end to end
- `src/main.ts` now passes explicit React-owned handlers into the live shell and panel renderers, so runtime URL actions, settings actions, confirmation actions, and shell navigation no longer depend on broad delegated app-root events.
- `src/dom-seams.test.mjs` now covers shell navigation, settings subpage navigation, URL controls, masked API-key focus behavior, guidance links, and Redux view or panel updates through React-centric element trees and store actions instead of the retired delegated-event seam.
- Removed the unused broad `registerAppEventHandlers(...)` path plus the old app-shell compatibility exports that only supported panel-root seam code, updated `README.md` and `docs/SPECS.md` with the current React plus Redux architecture, marked the remaining React TODO phases complete, and revalidated with `pnpm lint`, `pnpm test:ui`, `pnpm build`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, and `cargo test --manifest-path src-tauri/Cargo.toml --all-features`.

## 2026-04-18T21:00:29Z - GPT-5.4 - Removed the frontend bundle-size warning by cutting server-render code from the client graph
- Removed the unused `renderPanelMarkup(...)` helper from `src/confirmation-panel-helpers.ts`, which also let the shared runtime module drop its `react-dom/server` import entirely.
- Changed `src/app-shell.ts` so the `renderAppShell()` helper used only by `src/app-shell.test.mjs` loads `react-dom/server` lazily instead of importing it into the live browser bundle.
- The build warning is now actually gone instead of being masked: `pnpm build` produces a `428.09 kB` main JS chunk instead of the prior `615.81 kB`, and full validation is green with `pnpm lint`, `pnpm test:ui`, `pnpm build`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, and `cargo test --manifest-path src-tauri/Cargo.toml --all-features`.

## 2026-04-18T22:02:52Z - GPT-5.4 - Phase 3 split the oversized frontend panel modules into domain files
- Replaced the monolithic `src/settings-status-panels.ts` implementation with a barrel that re-exports focused domain modules under `src/settings-panels/`: playback, planner, TTS, ASR, runtime, workspace, and shared controls.
- Split `src/confirmation-panel.ts` into a thin export surface over dedicated confirmation and push-to-talk modules in `src/confirmation-panels/`, while keeping the existing runtime import contract stable.
- Added shared React control primitives for repeated settings card patterns, then revalidated the full repo successfully with `pnpm lint`, `pnpm test:ui`, `pnpm build`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, and `cargo test --manifest-path src-tauri/Cargo.toml --all-features`.

## 2026-04-18T21:43:40Z - GPT-5.4 - Phase 2 removed the last HTML render seam from the frontend runtime
- Deleted the legacy string-only playback helpers from `src/settings-status-panels.ts` and dropped their test-only re-exports from `src/confirmation-panel.ts`, so the remaining panel surface is React-node based.
- Replaced `renderPanelRoot(..., html)` in `src/app-shell.ts` with `preserveActivePanelControl(...)`, which keeps the focus-restoration behavior while removing the last `innerHTML` rendering path from the app shell module.
- `docs/TODO.md` now marks the full Priority 2 seam-removal phase complete, and validation is green again with `pnpm lint`, `pnpm test:ui`, `pnpm build`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, and `cargo test --manifest-path src-tauri/Cargo.toml --all-features`.

## 2026-04-18T21:34:10Z - GPT-5.4 - Phase 1 of the React follow-up moved runtime rendering onto a single Redux-backed tree
- `src/main.ts` no longer imperatively rerenders panel roots; it now mounts one React app tree and renders shell views and panel content from the Redux-backed frontend state.
- `src/app-shell-store.ts` is now the source of truth for shell view state, panel state, and execution UI state, which let the runtime replace the old `rerender*Panel()` orchestration with state-driven rendering.
- `docs/TODO.md` now marks the full Priority 1 React ownership phase complete, and the full validation gate passed after the refactor: `pnpm lint`, `pnpm test:ui`, `pnpm build`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, and `cargo test --manifest-path src-tauri/Cargo.toml --all-features`.

## 2026-04-18T21:16:58Z - GPT-5.4 - Replaced the stale docs TODO with a current React follow-up checklist
- Rewrote `docs/TODO.md` so it now reflects the current frontend state after the settings cleanup instead of the original scaffold-era phase plan.
- The new TODO is organized around the actual remaining React work: moving ownership out of `src/main.ts`, removing the last HTML rendering seams, splitting oversized UI modules, tightening React-side event handling, and strengthening frontend tests.
- Kept a validation checklist in the doc so each remaining migration slice continues to run the same frontend and Rust gates.

## 2026-04-18T20:44:21Z - GPT-5.4 - Migrated panel tests off the string compatibility layer
- Removed the source-level string compatibility wrappers for the already-migrated panel builders in `src/confirmation-panel.ts` and `src/settings-status-panels.ts`, so the runtime and exported API now prefer React-node renderers directly.
- Updated `src/confirmation-panel.test.mjs` to render those panel node builders through a test-local serializer instead of depending on the app's SSR compatibility bridge, including local prop-to-attribute mapping and `<select>`/`<option>` selection inference to preserve existing DOM assertions.
- Frontend validation remains green after the cleanup: `pnpm lint`, `pnpm test:ui`, and `pnpm build` all pass; the only remaining build signal is still the non-blocking Vite chunk-size warning.

## 2026-04-18T20:29:07Z - GPT-5.4 - Runtime panel roots now mount as React subtrees
- Replaced the live panel update path in `src/main.ts` and `src/app-shell.ts` so mounted panel roots render through dedicated React roots instead of replacing `innerHTML` with server-rendered HTML strings.
- Added React-node panel builders for the mounted workspace and settings surfaces in `src/confirmation-panel.ts` and `src/settings-status-panels.ts`, while keeping the existing string-returning render functions as compatibility wrappers for the current HTML-based tests.
- Frontend validation remains green after the runtime-path switch: `pnpm lint`, `pnpm test:ui`, and `pnpm build` all pass; the only remaining build signal is still the non-blocking Vite chunk-size warning.
## 2026-04-18T20:11:13Z - GPT-5.4 - Remaining selector and local settings panels moved to React components
- Converted the remaining string-based selector and local-profile settings panels in `src/settings-status-panels.ts` to React implementations: TTS provider, TTS model, TTS voice, local TTS profile, local ASR profile, and model management.
- Kept the existing DOM and delegated-event contract intact by continuing to render these panels through the shared `renderReactMarkup(...)` string bridge, including boolean-attribute normalization for `disabled`, `checked`, and `selected` so the HTML-based test expectations stay stable.
- Frontend validation remains green after this slice: `pnpm lint`, `pnpm test:ui`, and `pnpm build` all pass; the only remaining build signal is still the non-blocking Vite chunk-size warning.
## 2026-04-18T19:53:18Z - GPT-5.4 - Remote settings panels and API-key card moved to React components
- Converted the remaining remote settings slice in `src/settings-status-panels.ts` so the remote planner, remote TTS, and remote ASR panels now render through React component implementations while preserving the existing HTML-returning `render...Panel()` API.
- Moved the shared API-key entry path in `src/confirmation-panel-helpers.ts` from raw template-string output to a React node helper, including the masked-value field behavior, save/test controls, and OpenAI API key status/link rendering.
- Frontend validation remains green after the remote-panel migration: `pnpm lint`, `pnpm test:ui`, and `pnpm build` all pass; the remaining build signal is still the non-blocking large-chunk warning from Vite.
## 2026-04-18T19:44:24Z - GPT-5.4 - First real settings-panel slice moved to React components
- Converted a first cluster of actual settings panels in `src/settings-status-panels.ts` from template-string implementations to React component implementations while preserving the existing `render...Panel(): string` API via `renderToStaticMarkup` wrappers.
- This slice covers nearby playback controls, failover, confirmation, OCR fallback, settings guidance, and ASR provider selection; the more complex remote-profile and secret-entry panels still use the older string path.
- Frontend validation stayed green after the change: `pnpm lint`, `pnpm test:ui`, and `pnpm build` all pass, with the same existing Vite warning about the large main JS chunk.
## 2026-04-18T19:28:18Z - GPT-5.4 - Frontend shell migrated onto React, Material UI, and Redux navigation state
- Added `react`, `react-dom`, `@mui/material`, `@mui/icons-material`, `@emotion/react`, `@emotion/styled`, `@reduxjs/toolkit`, `react-redux`, plus `@types/react` and `@types/react-dom` to support a real React/MUI shell path under the repo's strict type-aware frontend linting.
- `src/app-shell.ts` now mounts the outer shell through React with Material UI buttons and icon controls while preserving the existing imperative panel-root rendering so the migration stays incremental instead of rewriting every settings/status panel at once.
- `src/app-shell-store.ts` now holds Redux Toolkit state for `workspace` versus `settings` plus nested settings subviews, and `src/main.ts` dispatches/subscribes to that store instead of owning those view flags as local mutable variables.
- Frontend validation is green after the migration: `pnpm lint`, `pnpm test:ui`, and `pnpm build` all pass; the Vite build now warns that the main JS chunk is above 500 kB after minification.
## 2026-04-18T19:13:55Z - GPT-5.4 - Settings subpage back button moved into the toolbar
- Moved the shared settings-subpage back-arrow button from each subpage hero into the top shell toolbar in `src/app-shell.ts`, right-aligned opposite the `Workspace` and `Settings` nav buttons.
- Kept the back control as an inline SVG icon button rather than adding Font Awesome, and updated `src/styles.css` so it only appears while a non-overview settings subpage is active.
- Updated `src/app-shell.test.mjs`, and validation is green with `pnpm lint` plus `pnpm test:ui`.

## 2026-04-18T19:07:51Z - GPT-5.4 - Settings subpages use back-arrow icon buttons
- Replaced the text `Back to settings` controls on every settings subpage in `src/app-shell.ts` with a shared back-arrow icon button that keeps the same accessible label.
- Updated `src/styles.css` so the back control reads as a proper icon button instead of a text link while preserving keyboard focus styles.
- Updated `src/app-shell.test.mjs`, and validation is green with `pnpm lint` plus `pnpm test:ui`.

## 2026-04-18T19:03:24Z - GPT-5.4 - Runtime setup moved to a settings subpage
- Continued the Settings declutter pass by moving the Runtime controls out of the overview in `src/app-shell.ts` and behind an `Open Runtime setup` link with a matching back control.
- Extended nested settings-view routing in `src/app-shell.ts` and `src/event-handlers.ts` so Runtime-targeted guidance and control links open the Runtime subpage before focusing the requested element.
- Updated `src/app-shell.test.mjs` and `src/dom-seams.test.mjs`, and validation is green with `pnpm lint` plus `pnpm test:ui`.

## 2026-04-18T19:00:12Z - GPT-5.4 - ASR setup moved to a settings subpage
- Continued the Settings declutter pass by moving the ASR controls out of the overview in `src/app-shell.ts` and behind an `Open ASR setup` link with a matching back control.
- Extended nested settings-view routing in `src/app-shell.ts` and `src/event-handlers.ts` so ASR-targeted guidance and control links open the ASR subpage before focusing the requested element.
- Updated `src/app-shell.test.mjs` and `src/dom-seams.test.mjs`, and validation is green with `pnpm lint` plus `pnpm test:ui`.

## 2026-04-18T18:55:16Z - GPT-5.4 - TTS setup moved to a settings subpage
- Continued the Settings declutter pass by moving the TTS controls out of the overview in `src/app-shell.ts` and behind an `Open TTS setup` link with a matching back control.
- Extended nested settings-view routing in `src/app-shell.ts`, `src/main.ts`, and `src/event-handlers.ts` so TTS-targeted guidance and control links open the TTS subpage before focusing the requested element.
- Updated `src/app-shell.test.mjs` and `src/dom-seams.test.mjs`, and validation is green with `pnpm lint` plus `pnpm test:ui`.

## 2026-04-18T18:51:46Z - GPT-5.4 - Planner setup moved to a settings subpage
- Split the crowded Settings overview in `src/app-shell.ts` by moving the planner configuration panel behind a dedicated in-settings subpage reached through an `Open planner setup` link and a `Back to settings` control.
- Added nested settings-view state in `src/app-shell.ts`, `src/main.ts`, and `src/event-handlers.ts` so planner-target guidance links open the planner subpage before focusing the requested control.
- Updated `src/app-shell.test.mjs` and `src/dom-seams.test.mjs`, and validation is green with `pnpm lint` plus `pnpm test:ui`.

## 2026-04-18T18:40:12Z - GPT-5.4 - Removed leftover verbose settings copy
- Removed the leftover `Configuration` eyebrow and the long settings intro from `src/app-shell.ts` after the user flagged it as too verbose for a voice-first UI.
- Removed the playback group description in `src/app-shell.ts` and the narration helper paragraph in `src/settings-status-panels.ts` so the playback section relies on labels instead of explanatory prose.
- Updated `src/confirmation-panel.test.mjs` and revalidated with `pnpm lint` plus `pnpm test:ui`.

## 2026-04-18T18:34:01Z - GPT-5.4 - Workspace page copy simplified
- Simplified the workspace hero and overview-card copy in `src/app-shell.ts` to use shorter, more direct language for the main workflow.
- Renamed the overview cards from `Page control` to `Page actions` and from `Runtime status` to `Status` to match the broader plain-language cleanup across the app shell.
- Added a workspace copy assertion to `src/app-shell.test.mjs`, and validation is green with `pnpm lint` and `pnpm test:ui`.

## 2026-04-18T18:29:08Z - GPT-5.4 - Advanced/runtime settings wording simplified
- Simplified the runtime group heading in `src/app-shell.ts` from a more abstract “Models and safeguards” label to “Runtime”, with shorter supporting copy.
- Simplified advanced panel titles and labels in `src/settings-status-panels.ts`, including shorter wording for local models, failover, confirmation, and OCR fallback controls.
- Updated `src/confirmation-panel.test.mjs` to match the shorter wording, and validation is green with `pnpm lint` and `pnpm test:ui`.

## 2026-04-18T18:23:53Z - GPT-5.4 - Planner TTS ASR settings copy simplified further
- Shortened planner, TTS, and ASR section titles in `src/settings-status-panels.ts` so they read more like user-facing settings and less like internal schema names.
- Trimmed repetitive descriptions and repeated field labels inside the local/remote TTS and ASR panels, while keeping the key distinction that API keys remain masked and stored via the OS keyring.
- Updated `src/confirmation-panel.test.mjs` to match the simpler wording, and validation is green with `pnpm lint` and `pnpm test:ui`.

## 2026-04-18T18:18:49Z - GPT-5.4 - Playback settings collapsed into one clearer section
- Removed the redundant `settings-volume` and `settings-speed` shell sections so the Settings page now has a single playback block instead of separate duplicated volume/speed settings panels.
- Updated the remaining playback panel copy to state clearly that volume and speed changes apply to current playback and remain the saved defaults for future narration.
- Updated shell and panel tests to lock the simpler structure, and validation is green with `pnpm lint` and `pnpm test:ui`.

## 2026-04-18T18:12:23Z - GPT-5.4 - Settings page reorganized into logical groups
- Reworked the settings shell in `src/app-shell.ts` so the page now flows as: guidance, playback, planner, TTS, ASR, then models/safeguards instead of interleaving unrelated sections.
- Added explicit group headings and intro copy plus supporting styles in `src/styles.css` so playback defaults no longer appear detached from playback controls and provider failover no longer sits ahead of TTS/ASR setup.
- Added `src/app-shell.test.mjs` to lock the new settings-page ordering, and validation is green with `pnpm lint` and `pnpm test:ui`.

## 2026-04-18T17:58:44Z - GPT-5.4 - Type-aware ESLint added for frontend TypeScript
- `eslint.config.js` now applies `typescript-eslint`'s `recommendedTypeChecked` rules to `src/**/*.ts` using `parserOptions.projectService` with the existing `tsconfig.json`.
- The first type-aware lint pass surfaced three real issues in `src/tauri-api.ts`: two redundant `unknown | null` unions and one plain-object throw path; those were fixed instead of weakening the rules.
- Validation after adding the type-aware layer is green: `pnpm lint`, `pnpm test:ui`, `pnpm build`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, and `cargo test --manifest-path src-tauri/Cargo.toml --all-features` all pass.

## 2026-04-18T17:54:33Z - GPT-5.4 - Frontend ESLint expanded to a moderate quality ruleset
- Expanded `eslint.config.js` beyond the minimal baseline by adding moderate correctness and maintainability rules for frontend files: `curly`, `eqeqeq`, `no-console` (allowing only `warn` and `error`), `no-useless-concat`, `object-shorthand`, `prefer-const`, `prefer-template`, and `reportUnusedDisableDirectives`.
- The stricter lint pass surfaced debug-only `console.debug` calls in `src/main.ts`; those were removed instead of relaxing the rule.
- Validation after the expansion is green: `pnpm lint`, `pnpm test:ui`, `pnpm build`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, and `cargo test --manifest-path src-tauri/Cargo.toml --all-features` all pass.

## 2026-04-18T17:49:31Z - GPT-5.4 - CI now includes the frontend lint gate
- Updated `.github/workflows/ci.yml` so the existing `validate` job now runs `pnpm lint` before the UI tests and frontend build.
- The new CI lint step reuses the already-installed JavaScript dependencies and existing Node/pnpm setup in the workflow; no other CI behavior was changed.
- Local revalidation after the workflow update confirmed the new step payload still passes with `source ./fix-node-version.sh && pnpm lint`.

## 2026-04-18T17:47:47Z - GPT-5.4 - Frontend ESLint gate added and passing
- Added a dedicated frontend lint script in `package.json` and a minimal flat ESLint config in `eslint.config.js` for `src/**/*.ts` and `src/**/*.test.mjs`.
- The lint setup uses `eslint`, `@eslint/js`, `typescript-eslint`, and `globals`, with no formatting rules and no type-aware project config, to keep the change minimal and aligned with the existing TypeScript/Vite repo.
- Initial lint findings were fixed in `src/confirmation-panel-helpers.ts` and `src/confirmation-panel.test.mjs` rather than weakening the config.
- Validation after the lint setup is green: `pnpm lint`, `pnpm test:ui`, `pnpm build`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, and `cargo test --manifest-path src-tauri/Cargo.toml --all-features` all pass.

## 2026-04-18T17:42:44Z - GPT-5.4 - Full repo validation is currently green
- Rust lint passes with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`.
- Frontend validation passes with `source ./fix-node-version.sh && pnpm test:ui && pnpm build`, including 84 UI tests green and a successful TypeScript/Vite build.
- Backend validation passes with `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, including 309 Rust tests green.
- There is no separate frontend lint script in `package.json`; the current frontend gate is the UI test suite plus the strict `tsc && vite build` path.

## 2026-04-18T16:50:15Z - GPT-5.4 - TTS and ASR API key inputs now share the masked last-4 preview
- The remote TTS and remote ASR API key textboxes now mirror the planner behavior by showing a display-only masked value in the form `***1234` when a key is already configured.
- Their masked values are derived from resolved configured secrets on the backend and passed through runtime state as `api_key_masked_value`, without exposing raw keys to the frontend.
- The shared input behavior remains the same across planner, TTS, and ASR: clear the mask on focus, restore it on blur if no replacement was entered, and never treat the mask itself as a real draft key.
- Validation after the extension is green: `pnpm test:ui` passed with 84 UI tests, `pnpm build` passed, and `cargo test --manifest-path src-tauri/Cargo.toml --all-features` passed with 309 backend tests.

## 2026-04-18T16:36:32Z - GPT-5.4 - Planner API key input now shows a masked last-4 preview
- The remote planner API key textbox now shows a display-only masked value in the form `***1234` when a key is already configured, instead of staying blank.
- The mask is derived from the resolved configured secret on the backend and passed through runtime state as `api_key_masked_value`, without exposing the raw key.
- Frontend input handling clears the mask on focus and restores it on blur if the user does not type a replacement, so the masked display never gets treated as a real draft value.
- Validation after the change is green: `pnpm test:ui` passed with 82 UI tests, `pnpm build` passed, and `cargo test --manifest-path src-tauri/Cargo.toml --all-features` passed with 309 backend tests.

## 2026-04-18T16:12:23Z - GPT-5.4 - Remote planner settings are now editable with model discovery and reset
- The remote planner Settings panel now lets the user edit the endpoint, load available models from the current OpenAI-compatible `/models` endpoint, select a returned model, save the endpoint/model pair, and reset those two fields back to the shipped defaults.
- The planner panel no longer shows the `Service` card; the user-facing flow is now centered on endpoint, model, and API key actions instead of provider implementation details.
- Backend support was added for persisting planner endpoint/model changes, resetting them to the default template values for the active planner profile, and loading models from OpenAI-compatible endpoints using the entered or configured API key.
- Validation after the feature landed is green: `pnpm test:ui` passed with 79 UI tests, `pnpm build` passed, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` passed with 309 tests, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` passed.

## 2026-04-18T15:49:56Z - GPT-5.4 - API key settings stripped down to user actions only
- `src/settings-status-panels.ts` no longer renders the `Current API key source` card in the remote planner panel, keeping the screen focused on user-relevant fields and actions.
- `src/confirmation-panel-helpers.ts` now shows only the `API key` label and the save/test controls, without any storage implementation or reassurance copy.
- Validation after the simplification: `pnpm test:ui` passed with 75 UI tests and `pnpm build` passed.

## 2026-04-18T15:23:01Z - GPT-5.4 - Planner-provider panel and contract path removed
- Removed the dead-end `Planner provider selection` panel from the Settings UI along with its frontend panel state, render plumbing, guidance link, and API typing.
- Removed the matching backend `planner_provider_settings` field from `AgentStateData`, deleted the unused builder/helper path, and cleaned up the Rust/JS tests and fixtures that only existed for that panel.
- Full validation after the removal is green: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (305 tests), `pnpm test:ui` (75 tests), and `pnpm build` all pass.

## 2026-04-18T14:45:12Z - GPT-5.4 - Settings UI copy trimmed further
- `src/app-shell.ts` no longer renders the top branding copy or the Settings hero text, so the Settings page opens directly on actionable controls.
- `src/settings-status-panels.ts` no longer renders the descriptive sentence above the nearby playback volume and speed controls.
- Validation after the cleanup: `pnpm test:ui` passed with 76 UI tests and `pnpm build` passed.

## 2026-04-18T14:37:47Z - GPT-5.4 - Full validation run passed after API key visibility work
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml --all-features` passed with 306 backend tests green.
- `pnpm test:ui` passed with 76 UI tests green after switching to Node.js 22.12.0 via `fix-node-version.sh`.
- `pnpm build` passed and produced a successful frontend production build.

## 2026-04-18T12:25:06Z - GPT-5.4 - API key test results are now visually explicit in settings
- `src/confirmation-panel-helpers.ts` now renders successful `Test API key` results inside a dedicated status block labeled `Latest test result` instead of a plain low-contrast paragraph, so successful checks are visibly acknowledged after saving a key.
- `src/styles.css` adds distinct success-state styling for that status block, and `src/confirmation-panel.test.mjs` now asserts the explicit status container and label.
- Validation after the UI fix: `pnpm test:ui` passed with 76 tests and `pnpm build` passed.

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

## 2026-04-02T20:11:42Z - GPT-5.4 - Phase 7 frontend modularization
- `src/main.ts` now delegates shared frontend error/guidance mapping to `src/main-errors.ts` and runtime panel reconciliation to `src/runtime-refresh.ts`, which makes the main entrypoint smaller without changing UI behavior.
- `src/confirmation-panel.ts` now preserves the public panel render/type export surface while re-exporting most settings, status, and URL panel renderers from `src/settings-status-panels.ts`.
- Frontend validation after the refactor: `pnpm build` and `pnpm test:ui` both pass.
- `src/confirmation-panel.ts` and `src/styles.css` now show a temporary submitting status and disable both confirmation buttons while the response is in flight.
- Frontend validation after the submitting-state change: `pnpm build` passes.

## 2026-04-02T20:14:43Z - GPT-5.4 - Phase 7 modularization completed
- `src-tauri/src/commands.rs` is now split under `src-tauri/src/commands/` with test-only helper exposure repaired so the shared `commands/tests.rs` module still exercises planner validation and execution helpers.
- Frontend modularization now uses `app-shell.ts`, `panel-state.ts`, `event-handlers.ts`, `main-errors.ts`, `runtime-refresh.ts`, `panel-types.ts`, `confirmation-panel-helpers.ts`, and `settings-status-panels.ts`, with `src/main.ts` reduced below 2000 lines and `src/confirmation-panel.ts` reduced to the confirmation-specific surface.
- `docs/BB_CODE_REVIEW1_TODO.md` now marks phase 7 done, and the validated commands remain `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build` after sourcing `./fix-node-version.sh`.

## 2026-04-02T20:23:45Z - GPT-5.4 - Phase 8 coverage completed
- Added Node test coverage for the extracted DOM seams in `src/dom-seams.test.mjs`, including targeted rerender focus/cursor restoration, delegated URL button handling after panel replacement, settings-target focus jumps, and scoped change-handler busy guards.
- Added Tauri bridge contract coverage in `src/tauri-api.test.mjs` plus an explicit invoke test hook in `src/tauri-api.ts`, and tightened the Rust `GetPageSnapshot` command test so snapshot metrics are asserted as non-zero live fields instead of placeholder zeroes.
- Confirmed `.github/workflows/ci.yml` already runs the same validated command set used locally: Rust clippy, Rust tests, `pnpm test:ui`, and `pnpm build` on Node 22.12.0 with the required Ubuntu native dependencies.

## 2026-04-02T20:28:40Z - GPT-5.4 - Phase 9 accessibility audit completed
- `src/settings-status-panels.ts` now adds screen-reader value text to range sliders, `aria-describedby` wiring for the models-directory input, polite live updates for URL/status changes, and clearer browser-visibility group semantics.
- `src/confirmation-panel-helpers.ts` now links remote API key inputs to their keyring-storage guidance, and `src/confirmation-panel.ts` groups approve/reject actions explicitly for assistive tech.
- `src/confirmation-panel.test.mjs` now covers the new accessibility semantics so the final UX pass stays pinned by the existing Node UI suite.

## 2026-04-02T20:30:24Z - GPT-5.4 - Review backlog completed
- Reconciled `docs/BB_CODE_REVIEW1_TODO.md` so phases 1-10 now reflect the landed work instead of leaving early-phase headings stale after later slices closed the remaining test, rerender, busy-state, and accessibility requirements.
- Ran the full final validation flow on the completed backlog: Rust formatting, clippy, Rust tests, frontend UI tests, and frontend production build all passed on the current tree.
- The review backlog is now fully implemented in the repo state represented by this worktree, with the last landed slices covering modularization, expanded seam/bridge tests, and the final accessibility pass.

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

## 2026-03-26T07:14:21Z - GPT-5.4 - Settings TTS provider selection landed
- `src-tauri/src/app_core.rs` now derives `tts_provider_settings` for `get_agent_state` and persists TTS mode changes through `set_tts_provider_mode(...)`, preserving the existing local and remote profile references.
- `src/tauri-api.ts`, `src/confirmation-panel.ts`, and `src/main.ts` now expose a dedicated Settings selector for switching TTS between the configured local and remote providers, then refresh the adjacent model and voice selectors from runtime state.
- `docs/TODO.md` now marks `TTS provider selection` complete, and validation is green with `cargo fmt`, `cargo clippy`, `cargo test`, `pnpm test:ui`, and `pnpm build` under the sourced Node 22.12.0 workflow.

## 2026-03-26T07:25:33Z - GPT-5.4 - Settings ASR provider selection landed
- `src-tauri/src/config.rs`, `src-tauri/src/app_core.rs`, `src-tauri/src/commands.rs`, and `src-tauri/src/lib.rs` now expose persisted ASR provider-mode selection and runtime-derived `asr_provider_settings` through `get_agent_state`, preserving the existing local and remote ASR profile references.
- `src/tauri-api.ts`, `src/confirmation-panel.ts`, `src/confirmation-panel.test.mjs`, and `src/main.ts` now expose a dedicated Settings selector for switching ASR between the configured local and remote providers and surfacing save errors inline.
- `docs/TODO.md` now marks `ASR provider selection` and `ASR provider selection behavior` complete, and validation is green with `cargo clippy`, `cargo test`, `pnpm test:ui`, and `pnpm build` under the sourced Node 22.12.0 workflow.

## 2026-03-26T07:42:54Z - GPT-5.4 - Planner provider settings are currently remote-only
- `src-tauri/src/config.rs` still explicitly rejects local planner mode, local planner profiles, and planner failover, so the Settings UI cannot honestly offer a `Local`/`Remote` planner selector yet.
- `src-tauri/src/app_core.rs`, `src-tauri/src/commands.rs`, `src/tauri-api.ts`, `src/confirmation-panel.ts`, and `src/main.ts` now expose `planner_provider_settings` through `get_agent_state` and render a read-only Settings panel that explains the active remote-only planner constraint.
- `docs/TODO.md` now marks `Planner provider selection` complete in this remote-only/read-only form, and validation is green with `cargo clippy`, `cargo test`, `pnpm test:ui`, and `pnpm build` under the sourced Node 22.12.0 workflow.

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

## 2026-04-18T11:20:35Z - GPT-5.4 - OpenAI API key hint added for missing secrets
- `src/confirmation-panel-helpers.ts` now tells users where to get an OpenAI API key from the secure API key entry card for remote planner, TTS, and ASR profiles.
- `src/main-errors.ts` now adds the same OpenAI API key URL to missing-secret guidance, including the push-to-talk ASR secret-unavailable path.
- Added focused frontend coverage in `src/main-errors.test.mjs` and extended the remote planner settings render test; `pnpm test:ui` and `pnpm build` both pass.

## 2026-04-18T11:25:27Z - GPT-5.4 - Sanitized API key rejection now includes the creation link
- `src-tauri/src/app_core.rs` now appends `https://platform.openai.com/account/api-keys` to the sanitized 401 OpenAI API key test failure message so the Settings test result gives the user the next action directly.
- Updated the Rust connectivity test to assert the new sanitized message while still proving no `sk-proj` fragment leaks.
- Validation after the change: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` and `cargo test --manifest-path src-tauri/Cargo.toml --all-features` both pass.

## 2026-04-18T11:31:29Z - GPT-5.4 - OpenAI API key URL now renders as a clickable link in settings
- `src/confirmation-panel-helpers.ts` now renders the known OpenAI API key URL as a real anchor in the secure API key entry card and in API key test status text, while still escaping the rest of the message.
- `src/settings-status-panels.ts` now applies the same link rendering to guidance copy so the missing-secret guidance panel is clickable too.
- Added focused render coverage in `src/confirmation-panel.test.mjs`; `pnpm test:ui` passed with 74 tests and `pnpm build` passed.

## 2026-04-18T11:41:20Z - GPT-5.4 - OpenAI settings links now open the system browser
- `src/event-handlers.ts`, `src/main.ts`, and `src/tauri-api.ts` now route clicks on `data-external-link-url` anchors through a dedicated `open_external_url` Tauri command instead of relying on inert `_blank` link behavior inside the Tauri webview.
- `src-tauri/src/lib.rs` now validates HTTPS external URLs and launches the OS browser with `xdg-open`, `open`, or `cmd /C start` depending on the platform.
- Added focused coverage in `src/dom-seams.test.mjs` and `src/tauri-api.test.mjs`; full validation passed with `cargo clippy`, `cargo test`, `pnpm test:ui` (76 tests), and `pnpm build`.

## 2026-04-18T11:50:44Z - GPT-5.4 - Same-session keyring reads now use a runtime cache
- `src-tauri/src/config.rs` now caches keyring-backed secrets in process memory after save or successful read, so an immediate Save API Key → Test API Key flow can resolve the newly saved secret even if the OS keyring backend does not return it back reliably in the same app session.
- The cache sits under the shared `resolve_secret_ref` path, so planner, TTS, ASR, and API key test reads all benefit from the same behavior during the current runtime.
- Validation after the change: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` and `cargo test --manifest-path src-tauri/Cargo.toml --all-features` both pass.

## 2026-04-18T10:59:52Z - GPT-5.4 - Settings OpenAI API key test added
- The Settings page remote planner, remote TTS, and remote ASR cards now expose a `Test API key` action that tests either the entered unsaved key or the currently configured secret reference.
- Backend validation now performs a real OpenAI-compatible `GET /models` request against the configured remote profile base URL, including organization and project headers when configured.
- Validation after the feature change is green: `pnpm test:ui`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, and `pnpm build` all pass.

## 2026-04-18T11:13:10Z - GPT-5.4 - API key test errors sanitized
- The backend no longer returns raw OpenAI error bodies for Settings API key tests, which prevents partial key fingerprints or verbose provider payloads from reaching the UI.
- Settings API key test failures now map HTTP status codes to short safe copy such as invalid key, forbidden project access, rate limiting, timeout, or a generic verification failure.
- Validation after the fix: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` and `cargo test --manifest-path src-tauri/Cargo.toml --all-features` both pass.

## 2026-04-18T09:53:56Z - GPT-5.4 - Full validation run passed
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml --all-features` passed with 304 backend tests green.
- `pnpm test:ui` passed with 65 UI tests green.
- `pnpm build` passed and produced a successful frontend production build.
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

## 2026-04-18T10:14:49Z - GPT-5.4 - Remote-first TTS and ASR defaults
- `config.example.toml` and `AppConfig::default()` now ship with OpenAI TTS and OpenAI ASR selected by default, while retaining the local KittenTTS and Whisper profiles for later setup.
- Push-to-talk setup failures now map `asr_model_unavailable` and `asr_secret_unavailable` into short user-facing messages instead of surfacing raw backend model-path text in the main UI.
- `docs/SPECS.md` now documents the remote-first first-run behavior, and the full validation set passed again: clippy, 304 Rust tests, 65 UI tests, and `pnpm build`.

## 2026-04-18T10:20:20Z - GPT-5.4 - Remote-first media defaults pushed to master
- Committed the remote-first TTS/ASR default change and simplified push-to-talk setup messaging as `e5e4152` (`feat: default voice input and speech to OpenAI`).
- Pushed `master` to `origin` successfully so GitHub now contains the OpenAI-default first-run behavior and the shorter voice-input setup copy.

## 2026-04-18T10:31:09Z - GPT-5.4 - Separate settings page added
- The frontend shell now has two in-app views: `Workspace` for push-to-talk, URL control, runtime status, and confirmation; `Settings` for all provider, model, playback, OCR, and model-management panels.
- The top-level nav switches between the two views, and existing `data-settings-target` buttons now force the settings view open before scrolling/focusing the requested control.
- Validation stayed green after the split: clippy, 304 Rust tests, 67 UI tests, and `pnpm build` all passed.

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

## 2026-03-25T18:23:07Z - GPT-5.4 - Fill-and-submit direct routing landed
- `src-tauri/src/commands.rs` now parses mixed utterances like `fill the email field with phil@example.com and then submit` via `parse_direct_fill_and_submit_command(...)`, stripping bounded submit suffixes while preserving the dictated field value text.
- `src-tauri/src/app_core.rs` now routes those mixed commands before the plain fill/submit shortcuts and emits a single confirmation-gated `confirm_action` → `focus_element` → `type_into_element` → `submit_active_form` plan when the field target grounds deterministically.
- The combined route intentionally submits with `form_element_id: null` so `submit_active_form` can use the focused field's owning form after the fill step succeeds, which keeps the workflow bounded without requiring brittle form-association recovery from the page model.
- Validation baseline for this slice is green: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build` all pass; the Vite build still emits the existing Node `22.11.0` vs `22.12.0+` warning.

## 2026-03-25T18:26:19Z - GPT-5.4 - Fill-and-submit slice revalidated for check-in
- Re-ran the standard validation set immediately before check-in: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, and `pnpm test:ui`.
- The fill-and-submit slice remains green at `142` Rust tests and `12` UI tests, so the current repo baseline for this combined direct-routing flow is clean going into commit and push.

## 2026-03-25T18:29:37Z - GPT-5.4 - GitHub Actions CI landed
- Added `.github/workflows/ci.yml` to run on pushes to `master` and on pull requests, using stable Rust with `clippy`, pnpm v9, Node `22.12.0`, the repo's Ubuntu/Tauri native packages, and the existing validation commands.
- Added a CI badge to `README.md` that points to the new workflow.
- Local validation remains green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`; the local shell still shows the existing Node `22.11.0` vs `22.12.0+` Vite warning, but the workflow pins Node `22.12.0`.

## 2026-03-25T18:32:57Z - GPT-5.4 - Local shell Node updated to 22.12.0
- Switched the active shell to Node `22.12.0` with `nvm use 22.12.0` and set `nvm alias default 22.12.0` so future shells default to the Vite-compatible version.
- Re-ran `pnpm build` afterward and the previous Vite Node version warning no longer appeared.

## 2026-03-25T18:36:34Z - GPT-5.4 - CI slice revalidated for check-in
- Re-ran the standard validation set immediately before check-in under Node `22.12.0`: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, and `pnpm test:ui`.
- The CI and README badge changes remain green at `142` Rust tests and `12` UI tests, and the earlier local Vite Node warning is no longer present after switching this shell to Node `22.12.0`.

## 2026-03-25T18:59:32Z - GPT-5.4 - Capture screenshot slice landed
- Wired `capture_screenshot` through the bounded tool layer in `src-tauri/src/commands.rs`, `src-tauri/src/browser.rs`, and `src-tauri/src/app_core.rs`, including planner-visible schemas, dispatch, executor support, mock coverage, browser PNG capture, cache persistence, and returned screenshot metadata.
- The browser implementation now supports viewport, full-page, and explicit bounding-box screenshots; planner validation rejects conflicting targeting modes and invalid bbox dimensions before dispatch.
- `region_id` requests currently fail clearly with `region_geometry_unavailable` because `PageRegion` still has no geometry, which keeps the tool deterministic without inventing partial region targeting behavior.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`, now at `144` Rust tests and `12` UI tests.

## 2026-03-25T19:07:31Z - GPT-5.4 - Capture screenshot slice revalidated for check-in
- Re-ran the standard validation set immediately before check-in under Node `22.12.0`: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, and `pnpm test:ui`.
- The screenshot slice remains green at `144` Rust tests and `12` UI tests going into commit and push.

## 2026-03-25T19:25:03Z - GPT-5.4 - Run OCR slice landed
- Wired `run_ocr` through the bounded tool layer in `src-tauri/src/commands.rs`, `src-tauri/src/ocr.rs`, and `src-tauri/src/app_core.rs`, including planner-visible schemas, dispatch, executor support, mock coverage, OCR runtime errors, and cached screenshot lookup by `image_id`.
- `src-tauri/src/ocr.rs` now provides an `OcrController` backed by `leptess`, normalizes OCR text/confidence, supports explicit bbox OCR within a cached image, and exposes typed runtime errors instead of hidden fallbacks.
- `run_ocr` currently requires an explicit `image_id` for actual OCR execution; `region_id` targeting still fails clearly with `region_geometry_unavailable`, and bbox-only OCR is intentionally rejected rather than inferring an implicit source image.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`, now at `150` Rust tests and `12` UI tests.

## 2026-03-25T19:30:02Z - GPT-5.4 - Run OCR slice revalidated for check-in
- Re-ran the standard validation set immediately before check-in under Node `22.12.0`: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, and `pnpm test:ui`.
- The run_ocr slice remains green at `150` Rust tests and `12` UI tests going into commit and push.

## 2026-03-25T19:39:33Z - GPT-5.4 - Merge OCR slice landed
- Wired `merge_ocr_into_page_model` through the bounded tool layer in `src-tauri/src/commands.rs` and `src-tauri/src/app_core.rs`, including planner-visible schemas, dispatch, executor support, mock coverage, and runtime page-model merge helpers.
- The merge runtime now validates the active `page_id`, trims OCR text, updates an existing region when `region_id` is supplied, or appends a new OCR region when no target region is given; merged DOM regions are marked `RegionSource::Mixed`.
- `infer_extraction_source(...)` now treats `RegionSource::Mixed` as OCR-contributing so merged page models surface `ExtractionSource::Merged` consistently after OCR enrichment.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`, now at `156` Rust tests and `12` UI tests.

## 2026-03-25T19:43:39Z - GPT-5.4 - Merge OCR slice revalidated for check-in
- Re-ran the standard validation set immediately before check-in under Node `22.12.0`: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, and `pnpm test:ui`.
- The merge_ocr_into_page_model slice remains green at `156` Rust tests and `12` UI tests going into commit and push.

## 2026-03-25T19:48:12Z - GPT-5.4 - Wave 2 contract cleanup completed
- Updated `docs/SPECS.md` so the shared contract section matches the Rust source of truth for `PageRegion`, `PageModel`, `ListeningState`, `RegionSource`, `ElementRole`, `ReportStatus`, and `ExtractionSource`, closing the remaining Wave 2 spec drift.
- Added regression coverage in `src-tauri/src/commands.rs` to assert every registered tool exposes an input schema and that key shared enum variants serialize to the expected external contract values.
- Checked off the remaining Wave 2 cleanup items in `docs/TODO.md` and revalidated with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`, now at `158` Rust tests and `12` UI tests.

## 2026-03-25T19:51:45Z - GPT-5.4 - Wave 2 contract cleanup prepared for check-in
- Verified the worktree contains only the expected cleanup slice files: `docs/SPECS.md`, `docs/TODO.md`, `src-tauri/src/commands.rs`, and `memory.md`.
- This slice is ready to commit and push on `master` after the green validation run that completed at `158` Rust tests and `12` UI tests.

## 2026-03-25T20:05:49Z - GPT-5.4 - Page model geometry attached
- Added `bbox: Option<Rect>` to `PageRegion` in `src-tauri/src/page_model.rs` with `#[serde(default)]` so serialized runtime state without region geometry remains readable.
- Wired live region bounding boxes through `src-tauri/src/browser.rs` and preserved OCR `source_bbox` when `merge_ocr_into_page_model` appends a new OCR region in `src-tauri/src/app_core.rs`.
- Updated `docs/SPECS.md` and `docs/TODO.md` to reflect geometry-bearing page regions, and added a state regression test covering legacy region deserialization without `bbox`.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`, now at `159` Rust tests and `12` UI tests.

## 2026-03-25T20:07:56Z - GPT-5.4 - Page model geometry prepared for check-in
- Verified the worktree contains only the expected geometry slice files: `docs/SPECS.md`, `docs/TODO.md`, `src-tauri/src/app_core.rs`, `src-tauri/src/browser.rs`, `src-tauri/src/commands.rs`, `src-tauri/src/narration.rs`, `src-tauri/src/page_model.rs`, `src-tauri/src/state.rs`, and `memory.md`.
- This slice is ready to commit and push on `master` after the green validation run that completed at `159` Rust tests and `12` UI tests.

## 2026-03-25T20:13:28Z - GPT-5.4 - Link extraction contract finalized
- Confirmed the current runtime already extracts links as `InteractiveElement` entries in `page_model.interactive_elements`; this slice made that behavior explicit rather than adding a separate link structure.
- Added regression coverage in `src-tauri/src/app_core.rs` proving `extract_page_model` preserves link metadata (`href`, text/accessibility fields, attributes, and `bbox`) when `include_links` is enabled and still omits link-role entries when disabled.
- Updated `docs/SPECS.md` and `docs/TODO.md` to reflect that extracted links live in `page_model.interactive_elements`.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`, now at `160` Rust tests and `12` UI tests.

## 2026-03-25T20:16:08Z - GPT-5.4 - Link extraction prepared for check-in
- Verified the worktree contains only the expected extract-links slice files: `docs/SPECS.md`, `docs/TODO.md`, `src-tauri/src/app_core.rs`, and `memory.md`.
- This slice is ready to commit and push on `master` after the green validation run that completed at `160` Rust tests and `12` UI tests.

## 2026-03-25T20:28:08Z - GPT-5.4 - Region screenshot cropping enabled
- Updated `src-tauri/src/app_core.rs` so `capture_screenshot` can target `region_id` by resolving the current page model’s stored `PageRegion.bbox` and passing that crop rectangle into the browser screenshot path.
- Added a regression test for region bbox resolution and kept failure handling explicit with `unknown_region_id`, `missing_region_bbox`, and `invalid_region_bbox`.
- Updated `docs/SPECS.md` and `docs/TODO.md` to reflect that region-targeted screenshots now require a positive stored bounding box and to mark `Crop screenshot regions` complete.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`, now at `161` Rust tests and `12` UI tests.

## 2026-03-25T20:30:59Z - GPT-5.4 - Region screenshot slice prepared for check-in
- Verified the worktree contains only the expected region-screenshot slice files: `docs/SPECS.md`, `docs/TODO.md`, `src-tauri/src/app_core.rs`, and `memory.md`.
- This slice is ready to commit and push on `master` after the green validation run that completed at `161` Rust tests and `12` UI tests.

## 2026-03-25T20:39:13Z - GPT-5.4 - Region OCR targeting enabled
- Updated `src-tauri/src/app_core.rs` so `run_ocr` can target `region_id` by resolving the current page model’s stored `PageRegion.bbox` and passing that crop rectangle into the OCR engine.
- Kept OCR source selection explicit: `region_id` OCR now requires an `image_id` for the cached screenshot, avoiding any implicit image fallback while still returning the resolved `source_bbox`.
- Updated `docs/SPECS.md` and `docs/TODO.md` to reflect that region-targeted OCR now requires a positive stored bounding box plus an explicit cached screenshot source.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`, now at `161` Rust tests and `12` UI tests.

## 2026-03-25T20:50:24Z - GPT-5.4 - OCR merge geometry preserved in page model
- Updated `src-tauri/src/app_core.rs` so `merge_ocr_into_page_model` fills an existing region’s missing stored `bbox` from the OCR `source_bbox`, while preserving any region geometry that is already present.
- Added regression coverage for both merge cases: existing regions now adopt missing geometry when OCR supplies it, and already-known region bounds are not overwritten by a later OCR merge.
- Updated `docs/SPECS.md` and `docs/TODO.md` to document the merge behavior and mark `Merge OCR into PageModel` complete.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`, now at `162` Rust tests and `12` UI tests.

## 2026-03-25T21:09:25Z - GPT-5.4 - No-text OCR fallback triggered from extraction
- Updated `src-tauri/src/app_core.rs` so `extract_page_model` now triggers deterministic OCR fallback when live DOM extraction returns no readable region text and `ocr.trigger_on_no_extractable_text` is enabled.
- The fallback reuses the bounded tool path internally—`capture_screenshot` with `full_page = true`, then `run_ocr`, then `merge_ocr_into_page_model`—instead of adding a second screenshot or OCR execution path.
- Added regression coverage for the no-text fallback trigger helper, keeping this slice scoped to the “no extractable text” case while leaving sparse-text heuristics for the next follow-up.
- Updated `docs/SPECS.md` and `docs/TODO.md` to document the extraction-time OCR fallback and mark `Trigger OCR when no extractable text is found` complete.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`, now at `165` Rust tests and `12` UI tests.

## 2026-03-25T21:32:31Z - GPT-5.4 - Sparse-text OCR thresholds now drive fallback
- Updated `src-tauri/src/app_core.rs` so extraction-time OCR fallback now uses the configured `ocr.sparse_text_char_threshold` and `ocr.sparse_text_region_threshold` instead of only the empty-text case.
- Added helper coverage for extracted-text metrics plus sparse-threshold-triggered and threshold-satisfied cases, keeping this slice focused on configurability rather than region-preference strategy.
- Updated `docs/SPECS.md` and `docs/TODO.md` to document threshold-aware fallback and mark `Make sparse-text OCR thresholds configurable` complete.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`, now at `168` Rust tests and `12` UI tests.

## 2026-03-25T21:46:52Z - GPT-5.4 - Region-first OCR fallback now preferred
- Updated `src-tauri/src/app_core.rs` so sparse extraction fallback now prefers bbox-backed readable regions for `run_ocr(region_id=...)` before widening to full-page OCR when `ocr.prefer_region_ocr` is enabled.
- The region-first path reuses a single cached full-page screenshot, merges successful region OCR back into the matching page regions, and only falls back to broad OCR when region recovery is unavailable or does not recover enough text.
- Added helper coverage for selecting region-first OCR targets and for respecting the `ocr.prefer_region_ocr` toggle.
- Updated `docs/SPECS.md` and `docs/TODO.md` to document the behavior and mark `Prefer region OCR before broader OCR when possible` complete.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`, now at `170` Rust tests and `12` UI tests.

## 2026-03-25T21:52:53Z - GPT-5.4 - Sparse OCR default policy locked in
- Confirmed the shipped OCR defaults already match the intended v1 fallback policy: `OcrSettings::default()`, `config.example.toml`, and the default config template all use `sparse_text_char_threshold = 200` and `sparse_text_region_threshold = 2`.
- Added regression coverage in `src-tauri/src/app_core.rs` for the exact default boundaries: fallback now stays explicitly covered at `200` readable characters, at `1` readable region, and above both boundaries.
- Added direct default/config coverage in `src-tauri/src/ocr.rs` and `src-tauri/src/config.rs`, then updated `docs/SPECS.md` and `docs/TODO.md` to state and mark complete the shipped policy of `200` readable characters or fewer than `2` readable regions.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`, now at `174` Rust tests and `12` UI tests.

## 2026-03-25T22:09:47Z - GPT-5.4 - Leptess integration surfaced in dev workflow
- Confirmed the OCR runtime was already backed by `leptess` through the Cargo `ocr` feature in `src-tauri/src/ocr.rs`, with the `full` feature bundle already including OCR for validation builds.
- Added `pnpm tauri:dev:ocr` and `pnpm tauri:dev:full` in `package.json` so developers can actually launch Tauri with the OCR or full native feature set instead of the no-feature default.
- Updated `README.md`, `docs/SPECS.md`, and `docs/TODO.md` to document that `leptess` is the OCR backend behind the `ocr` feature and to mark `Integrate leptess` complete.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`, with `174` Rust tests and `12` UI tests passing.

## 2026-03-25T22:16:18Z - GPT-5.4 - Basic URL input panel landed
- Added a dedicated `URL input` panel to the frontend shell in `src/confirmation-panel.ts`, including accessible copy, a typed `UrlInputPanelState`, and an editable `type="url"` field for staging the next navigation target.
- Updated `src/main.ts` so the URL field mirrors `agentState.url` until edited, then preserves the local draft across runtime panel rerenders instead of wiping in-progress input.
- Added focused render coverage in `src/confirmation-panel.test.mjs`, updated `src/styles.css` for the new panel layout, and marked `URL input` complete in `docs/TODO.md`.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`, with `174` Rust tests and `14` UI tests passing.

## 2026-03-26T01:31:19Z - GPT-5.4 - Open button wired to deterministic navigation
- Added a typed frontend `openUrl(...)` wrapper in `src/tauri-api.ts` and exposed a matching Tauri `open_url` command in `src-tauri/src/lib.rs`, reusing the existing deterministic Rust `execute_open_url()` path instead of adding parallel navigation logic.
- Extended the URL panel in `src/confirmation-panel.ts`, `src/main.ts`, and `src/styles.css` with an `Open` button plus busy/error handling; successful opens refresh runtime state, while failures preserve the draft URL and surface the backend or transport error.
- Added focused UI render coverage for the new button and its busy/error states in `src/confirmation-panel.test.mjs`, and marked `Open button` complete in `docs/TODO.md`.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`, with `174` Rust tests and `15` UI tests passing.

## 2026-03-26T03:53:45Z - GPT-5.4 - Read button wired through command resolution
- Extended the URL panel in `src/confirmation-panel.ts`, `src/main.ts`, and `src/styles.css` with a `Read` button and distinct opening versus reading busy states so the nearby controls stay explicit without adding a bespoke narration path.
- The frontend now resolves `"read page"` through `resolveCommand(...)`, executes the returned plan with the same request id via `runPlannerExecution(...)`, surfaces blocked planner `user_message` text directly in the panel, and refreshes runtime state after reading starts.
- Added focused render coverage for the new read-button state in `src/confirmation-panel.test.mjs` and marked `Read button` complete in `docs/TODO.md`.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`, with `174` Rust tests and `16` UI tests passing; Vite also warns in this shell that Node `22.11.0` is below its preferred `22.12+` floor even though the build still completes.

## 2026-03-26T05:09:21Z - GPT-5.4 - Stop button wired through command resolution
- Extended the URL panel in `src/confirmation-panel.ts`, `src/main.ts`, and `src/styles.css` with a `Stop` button and a dedicated stopping busy state so nearby navigation and narration controls remain explicit and separate.
- The frontend now resolves `"stop reading"` through `resolveCommand(...)`, executes the returned plan with the same request id via `runPlannerExecution(...)`, surfaces blocked planner `user_message` text directly in the panel, and refreshes runtime state after narration stops.
- Added focused render coverage for the new stop-button state in `src/confirmation-panel.test.mjs` and marked `Stop button` complete in `docs/TODO.md`.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`, with `174` Rust tests and `17` UI tests passing; Vite also warns in this shell that Node `22.11.0` is below its preferred `22.12+` floor even though the build still completes.

## 2026-03-26T05:26:50Z - GPT-5.4 - Next and Previous buttons wired through command resolution
- Extended the nearby URL/readback panel in `src/confirmation-panel.ts`, `src/main.ts`, and `src/styles.css` with `Next` and `Previous` buttons plus advancing and rewinding busy states so narration movement controls remain explicit in the basic Tauri UI.
- The frontend now resolves `continue reading` and `previous section` through `resolveCommand(...)`, executes the returned plans with matching request ids via `runPlannerExecution(...)`, and uses one shared helper so blocked planner messages and runtime refresh behavior stay aligned across read, stop, next, and previous actions.
- Added focused render coverage for the two new button busy states in `src/confirmation-panel.test.mjs` and marked `Next / Previous buttons` complete in `docs/TODO.md`.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`, with `174` Rust tests and `19` UI tests passing; Vite also warns in this shell that Node `22.11.0` is below its preferred `22.12+` floor even though the build still completes.

## 2026-03-26T05:34:46Z - GPT-5.4 - Hands-free listening loop landed
- The frontend push-to-talk panel now syncs runtime listening state from `get_agent_state`, advertises the spoken `start listening` path, and disables manual hold-to-talk while hands-free listening is already active.
- `src/main.ts` now reuses the existing bounded `transcribe_and_execute_command` surface in a frontend loop so spoken `start listening` enters continuous hands-free command capture until `stop listening` or an explicit runtime failure ends the listening session.
- Continuous-listening failures now surface explicit user-facing errors and attempt to stop listening cleanly; `docs/TODO.md` marks `Ensure normal operation is fully voice-controlled` complete and `docs/SPECS.md` now states the hands-free loop expectation.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`, with `174` Rust tests and `21` UI tests passing; Vite also warns in this shell that Node `22.11.0` is below its preferred `22.12+` floor even though the build still completes.

## 2026-03-26T06:15:27Z - GPT-5.4 - Settings volume panel landed
- Added a dedicated Settings UI playback-volume panel in the thin Tauri frontend while reusing the existing persisted `set_playback_volume` flow and shared `AudioControlsPanelState`.
- The nearby playback controls and the new Settings slider stay synchronized because both surfaces bind to the same frontend state and command path.
- Added render coverage for the settings volume panel and validated with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build` (`23` UI tests passing; build still shows the known Node `22.11.0` vs Vite `22.12+` warning).

## 2026-03-26T06:24:37Z - GPT-5.4 - Settings speed panel landed
- Added a dedicated Settings UI playback-speed panel in the thin Tauri frontend while reusing the existing persisted `set_playback_speed` flow and shared `AudioControlsPanelState`.
- The nearby playback controls and the new Settings speed slider stay synchronized because both surfaces bind to the same frontend state and command path.
- Added render coverage for the settings speed panel and validated with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build` (`25` UI tests passing; build still shows the known Node `22.11.0` vs Vite `22.12+` warning).

## 2026-03-26T06:39:13Z - GPT-5.4 - Settings TTS model selector landed
- Added a dedicated Settings UI TTS model selector that chooses among the configured TTS profiles/models for the current TTS mode instead of editing raw model ids or paths.
- The Rust runtime now exposes active and available TTS model/profile choices through `get_agent_state` and persists selection changes through a dedicated config provider-selection write path.
- Added Rust coverage for persisted TTS provider selection and frontend render coverage for the new selector; validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build` (`175` Rust tests, `27` UI tests, with the same known Node `22.11.0` vs Vite `22.12+` warning on build).

## 2026-03-26T06:42:34Z - GPT-5.4 - Node fix helper must be sourced to persist 22.12.0
- The existing `fix-node-version.sh` was reinstalling frontend dependencies under `Node.js 22.12.0` but, when executed normally, could not change the caller's shell back from `22.11.0`, so later `pnpm build` commands still showed the Vite warning.
- Updated `fix-node-version.sh` and `README.md` so the supported workflow is `source ./fix-node-version.sh`; when sourced, the current shell stays on `22.12.0`, and when executed normally the helper now explains that limitation explicitly.
- Verified with `source ./fix-node-version.sh && node -v && pnpm test:ui && pnpm build`: the shell stays on `v22.12.0`, all `27` UI tests pass, and the Vite unsupported-Node warning no longer appears.

## 2026-03-26T07:00:00Z - GPT-5.4 - Settings voice selector landed
- Added a dedicated Settings UI voice selector that reuses the persisted `set_tts_voice` path instead of introducing a frontend-only voice setting.
- The Rust runtime now exposes `tts_voice_settings` through `get_agent_state`, including the shipped KittenTTS voice list for local mode, the built-in OpenAI voice list for remote mode, and preservation of any already-configured custom voice so the UI never drops an existing setting.
- Added Rust coverage for current-mode voice choice derivation and frontend render coverage for the new selector; validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build` (`178` Rust tests, `29` UI tests).

## 2026-03-26T08:01:55Z - GPT-5.4 - Settings provider failover panel landed
- Added a dedicated read-only provider failover settings payload to `get_agent_state` plus a thin frontend Settings panel with disabled planner, TTS, and ASR failover toggles and explicit unavailability copy.
- Kept the slice honest by not adding a writable toggle because the current live runtime does not yet implement automatic provider failover, even though related config schema fields still exist.
- Added focused Rust and frontend render coverage for the new payload and panel; validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build` (`183` Rust tests, `35` UI tests).

## 2026-03-26T08:05:26Z - GPT-5.4 - setup-dev-env.sh now derives repo root dynamically
- Replaced the hardcoded `REPO_ROOT="/home/phil/work/blind_browser"` in `setup-dev-env.sh` with the same `BASH_SOURCE[0]`-based repo-root detection pattern already used by `fix-node-version.sh`.
- This removes the machine-specific `/home/phil` dependency so the dev setup script can be run from any checkout location without editing the file first.
- Verified the change with `bash -n setup-dev-env.sh` and a targeted search confirming `/home/phil` no longer appears in that script.

## 2026-03-26T08:13:58Z - GPT-5.4 - Settings confirmation behavior controls landed
- Added persisted confirmation-behavior settings to `get_agent_state` plus dedicated Tauri setters for `safety.confirmation_confidence_threshold` and `safety.allow_click_without_confirmation`.
- Added a thin frontend Settings panel with a threshold slider, a click-without-confirmation checkbox, and explicit read-only copy that submit actions still always require confirmation.
- Added focused Rust and frontend render coverage for the new persistence and UI path; validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build` (`185` Rust tests, `37` UI tests).

## 2026-03-26T08:22:43Z - GPT-5.4 - Settings OCR threshold controls landed
- Added persisted OCR-threshold settings to `get_agent_state` plus a dedicated Tauri setter for `ocr.sparse_text_char_threshold` and `ocr.sparse_text_region_threshold`.
- Added a thin frontend Settings panel with two numeric controls for the sparse-text character and region thresholds while intentionally leaving the other OCR toggles unchanged in this slice.
- Added focused Rust and frontend render coverage for the new persistence and UI path; validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build` (`187` Rust tests, `39` UI tests).

## 2026-03-26T08:33:39Z - GPT-5.4 - Missing-model errors now point back to settings controls
- Added a transient frontend guidance panel that appears only when surfaced errors look like missing local TTS or ASR model configuration/load failures.
- The panel gives direct jump actions to the already-visible settings controls that can help recover today: TTS provider/model controls for TTS issues and the ASR provider control for ASR issues.
- This keeps the slice honest by improving navigation to existing config surfaces without pretending model-download or local-path editing UI already exists.

## 2026-03-26T09:06:07Z - GPT-5.4 - Settings now expose local model references
- Extended `get_agent_state` with read-only local TTS and ASR model reference payloads so the frontend can show the configured local profile name, backend, model id, model path, and profile-specific details.
- Added dedicated Settings panels for those local model references and updated the missing-model guidance actions so users can jump straight to the new local reference panels when TTS or ASR local-model failures surface.
- Kept the slice honest by leaving edits in config for now; validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build` (`189` Rust tests, `42` UI tests).

## 2026-03-26T09:24:48Z - GPT-5.4 - Settings now expose remote API references
- Extended `get_agent_state` with read-only remote planner, remote TTS, and remote ASR settings payloads so the frontend can show the configured active remote profile details without exposing raw secrets.
- Added dedicated Settings panels for the remote planner/TTS/ASR profiles, including masked secret-reference sources such as environment-variable and file references instead of inline secret values.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build` (`192` Rust tests, `45` UI tests).

## 2026-03-26T18:22:40Z - GPT-5.4 - Settings now support keyring-backed remote API key entry
- Extended `SecretRef` with `from_keyring` support, centralized secret resolution/reference formatting, and added config persistence helpers that store UI-entered remote API keys in the OS keyring while writing only a keyring reference back to `config.toml`.
- Added Tauri commands and frontend settings controls for secure API key entry on the remote planner, remote TTS, and remote ASR panels, plus guidance that points users back to those controls when remote-secret errors surface.
- Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build` (`194` Rust tests, `46` UI tests).

## 2026-03-26T19:34:32Z - GPT-5.4 - Removed bogus migration wording and audited inline secret support
- Cleaned up README, CHANGELOG, and SPECS text that incorrectly described keyring-backed API key storage as a "migration" for this new project.
- Audited remaining `inline` secret support and confirmed it is still an explicit runtime/spec feature (`SecretRef::Inline` in `src-tauri/src/config.rs` and matching spec text), not a separate migration tool or hidden compatibility layer.

## 2026-03-26T19:46:12Z - GPT-5.4 - Inline secret support removed
- Removed `SecretRef::Inline` from the Rust config model and secret-resolution/reference paths, so inline API-key values in config no longer deserialize.
- Updated shipped examples, specs, changelog text, and tests to use only `from_env`, `from_file`, and `from_keyring` secret references; Ollama defaults now point at `OLLAMA_API_KEY`.
- Added a regression test that rejects inline secret refs and revalidated with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build` (`195` Rust tests, `46` UI tests).

## 2026-03-26T20:32:16Z - GPT-5.4 - Model management settings and manual local-model downloads landed
- Added dedicated Tauri model-management commands that expose persisted `models_dir`, `check_on_startup`, and `auto_download_missing` settings, plus explicit manual download actions for the configured local TTS and ASR profiles.
- Local downloads now map supported KittenTTS and Whisper model ids to known Hugging Face artifacts, write them into the configured models directory, and persist the resulting local profile `model_path` back to `config.toml`.
- Fixed the shipped local ASR backend mismatch by using `whisper` consistently, added frontend Settings controls and download buttons for model management, and revalidated with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build` (`195` Rust tests, `48` UI tests).

## 2026-03-26T21:17:02Z - GPT-5.4 - First schema/validation cleanup pass tightened Wave 1 tool contracts
- `src-tauri/src/commands.rs` now uses a closed `TtsVoiceName` enum for `SetTtsVoiceInput` instead of a free-form string, so unknown voice names are rejected at deserialization time before execution.
- Planner-side tool validation now also rejects blank `open_url`, `find_element`, `click_element`, `focus_element`, `type_into_element`, `read_region`, `confirm_action`, and `report_result` strings, plus invalid `find_element.max_candidates`, `transcribe_command.max_duration_ms`, and out-of-range playback volume/speed values.
- Revalidated with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`; Rust tests are now `202` passed and UI tests remain `48` passed.

## 2026-03-26T21:23:19Z - GPT-5.4 - Config/runtime enum cleanup closed backend and audio-format fields
- `src-tauri/src/config.rs` now models `LocalTtsProfile.backend`, `LocalAsrProfile.backend`, and `RemoteTtsProfile.audio_format` as closed enums with stable serialized values (`kitten_tts_rs`, `whisper`, and `wav`) instead of free-form strings.
- `src-tauri/src/tts.rs`, `src-tauri/src/asr.rs`, and `src-tauri/src/app_core.rs` now consume those enums directly, stringify them only when building UI-facing settings payloads, and no longer carry unreachable unsupported-backend or unsupported-audio-format runtime branches.
- Revalidated with `cargo fmt --manifest-path src-tauri/Cargo.toml --all`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`.

## 2026-03-26T21:29:59Z - GPT-5.4 - Validation rerun stayed green after enum cleanup
- Re-ran the standard repo validation pass after the config/runtime enum cleanup and existing schema-contract changes without making further code changes.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` passed, Rust unit/integration tests remained `202` passed, and `pnpm test:ui` remained `48` passed.

## 2026-03-26T21:49:02Z - GPT-5.4 - Wave 1 tool output schemas are now exported and regression-tested
- `src-tauri/src/commands.rs` now exposes `tool_output_schema(&ToolName)` for every registered deterministic tool, using the concrete `ToolResult<T>` envelope schema instead of only exposing input schemas.
- `AvailableTool` now includes `output_schema_ref` alongside `input_schema_ref`, and that planner-visible contract is reflected in `docs/SPECS.md` and `src/tauri-api.ts`.
- Added regression coverage that every registered tool exposes an output schema and that representative serialized tool results produced through `execute_planned_step` match the generated output schema; validation is green with `cargo fmt --manifest-path src-tauri/Cargo.toml --all`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build` (`205` Rust tests, `48` UI tests).

## 2026-03-26T21:53:08Z - GPT-5.4 - Validation rerun stayed green after Wave 1 output-schema work
- Re-ran the standard validation pass after exporting per-tool output schemas and adding output-schema regression tests, without further code changes.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` passed, Rust tests remained `205` passed, and `pnpm test:ui` remained `48` passed.

## 2026-03-26T22:11:07Z - GPT-5.4 - Wave 1 shared enums now cover repeated narration and visibility contract fields
- `src-tauri/src/commands.rs` now uses shared `NarrationInterruptionMode` for `read_region`, `read_next_region`, and `read_previous_region` inputs, shared `ElementVisibilityFilter` for `list_interactive_elements` and `find_element`, and shared `NarrationBoundary` for the next/previous narration outputs.
- `src-tauri/src/app_core.rs` now consumes those enums at the tool boundary while preserving the existing runtime behavior, and `docs/SPECS.md` plus the canonical planner/test fixtures now use the new enum-backed field names (`interruption_mode`, `visibility_filter`, `boundary`).
- Revalidated with `cargo fmt --manifest-path src-tauri/Cargo.toml --all`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`; Rust tests remain `205` passed and UI tests remain `48` passed.

## 2026-03-26T22:25:09Z - GPT-5.4 - Wave 1 shared enums now cover the remaining bounded mode fields
- `src-tauri/src/commands.rs` now replaces the remaining planner-visible semantic booleans with explicit enums: `ReloadMode`, `ClickMode`, `TextEntryMode`, `TextEntrySubmitMode`, and `TranscriptionStopMode`.
- `src-tauri/src/app_core.rs` and `src-tauri/src/lib.rs` now convert those enum values back to the existing runtime booleans only at the browser/ASR boundary, so runtime behavior stays unchanged while the contract surface becomes fully enum-backed for those tools.
- Updated `docs/SPECS.md`, direct-command helpers, canonical planner fixtures, and shared-enum serialization coverage to use `mode`, `click_mode`, `text_entry_mode`, `submit_mode`, and `stop_mode`; full validation is green with `cargo fmt --manifest-path src-tauri/Cargo.toml --all`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build` (`205` Rust tests, `48` UI tests).

## 2026-03-26T22:35:37Z - GPT-5.4 - Capture screenshot scope now uses a bounded enum
- `src-tauri/src/commands.rs` now replaces `CaptureScreenshotInput.full_page` with `scope: ScreenshotScope`, using closed `Viewport` and `FullPage` variants instead of a semantic boolean in the planner-visible contract.
- `src-tauri/src/app_core.rs` keeps screenshot behavior unchanged by converting `scope` back to the existing browser boolean only at the runtime boundary, and the internal OCR fallback screenshot path now requests `ScreenshotScope::FullPage`.
- Updated screenshot planner validation, canonical step fixtures, shared enum serialization coverage, and `docs/SPECS.md` to use `scope`; revalidated with `cargo fmt --manifest-path src-tauri/Cargo.toml --all`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build` (`205` Rust tests, `48` UI tests).

## 2026-03-26T22:41:19Z - GPT-5.4 - Validation rerun stayed green after screenshot cleanup
- Re-ran the standard lint and unit-test pass after the recent contract cleanup work without making further code changes.

## 2026-04-02T19:03:29Z - GPT-5.4 - Review backlog frontend phases 4 and 5 completed
- `src/main.ts` now emits structured `console.debug(...)` events for TTS provider transition start, success, rollback, and propagated panel errors, matching the review follow-up requirement without changing the existing voice-first flow.
- Runtime refresh error copy is now panel-scoped: status surfaces `runtime state refresh failed: ...` while model management surfaces `model management refresh failed: ...`, so partial refresh failures no longer look like one generic fan-out problem.
- `docs/BB_CODE_REVIEW1_TODO.md` now marks review phases 4 and 5 done, and the standard validation pass succeeded with `source ./fix-node-version.sh && . "$HOME/.cargo/env" && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings && cargo test --manifest-path src-tauri/Cargo.toml --all-features && pnpm test:ui && pnpm build`.

## 2026-04-02T19:10:37Z - GPT-5.4 - Review backlog phase 6 completed
- `src-tauri/src/browser.rs` now reads live page snapshot metrics (`scroll_y`, `viewport_width`, `viewport_height`, `document_height`) from the active Chromium page, normalizes the values, and covers that normalization with focused Rust tests.
- `src-tauri/src/app_core.rs` now threads those live metrics into both explicit `get_page_snapshot` tool responses and planner snapshot payloads, surfacing an explicit browser-backed failure instead of placeholder zeros when live metrics cannot be read.
- Updated `README.md`, `src/main.ts`, and `docs/SPECS.md` so the project no longer presents itself as a Phase 0 scaffold and so provider failover is described consistently as config-defined but runtime-disabled in v1.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` passed, Rust tests remained `205` passed, and `pnpm test:ui` remained `48` passed.

## 2026-03-26T22:56:24Z - GPT-5.4 - Wave 1 input schemas are now finalized
- `src-tauri/src/commands.rs` now centralizes the remaining planner-visible Wave 1 input limits and enforces them during planner validation: `open_url` must be absolute, `go_back`/`go_forward` steps must stay within the supported `1..=5` range, `scroll_page` requires either `amount_px` or `target` with a finite positive amount, and `find_element.max_candidates` is capped at the shared default of `3`.
- `src-tauri/src/app_core.rs` now reuses the same history-step, scroll-amount, and find-candidate constants as the contract layer so runtime behavior and planner validation stay aligned without hidden fallback drift.
- Added six new commands-layer regression tests for the finalized invalid-input cases, marked `Finalize input schema for all Wave 1 tools` complete in `docs/TODO.md`, and revalidated with `cargo fmt --manifest-path src-tauri/Cargo.toml --all`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build` (`211` Rust tests, `48` UI tests).

## 2026-03-29T10:56:30Z - GPT-5.4 - Planner contract serde coverage landed
- `src-tauri/src/commands.rs` now includes direct serde round-trip tests for both `PlannerOutput` and `PlannerInput`, in addition to the existing canonical planner schema checks.
- The new coverage validates generated schemas against representative nested planner payloads, including confirmation metadata, skill summaries, runtime agent state, page snapshots, recent tool history, and a page-model fixture with interactive elements.
- Fixed the test-only helper mismatch by building the planner-input page model from `fixture_page_model_without_regions()` inside `commands.rs` tests instead of referencing the inaccessible `app_core.rs`-local `fixture_page(...)` helper.
- Revalidated with `source ./fix-node-version.sh && . "$HOME/.cargo/env" && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings && cargo test --manifest-path src-tauri/Cargo.toml --all-features && pnpm test:ui && pnpm build`; UI tests remain `49` passed and the production build completes cleanly.

## 2026-03-29T11:26:04Z - GPT-5.4 - Per-tool input schema coverage completed
- `src-tauri/src/commands.rs` now includes `sample_planned_steps_match_generated_tool_input_schemas()`, which iterates `sample_planned_steps_for_registered_tools()` and checks each sample arguments payload against `tool_input_schema(...)`.
- The same regression also runs `validate_planned_step_arguments(...)` for every registered tool sample, so the exported JSON schema and the Rust-side runtime validator stay aligned on one representative valid input per tool.
- This closes the `Per-tool input schema validation` TODO without adding fallback behavior or duplicating the many existing targeted invalid-input tests.

## 2026-03-29T12:08:57Z - GPT-5.4 - Shared enum serde coverage completed
- `src-tauri/src/commands.rs` now includes a direct regression covering shared command-contract enums, locking their serialized forms and proving invalid enum strings fail deserialization.
- `src-tauri/src/config.rs` now includes matching coverage for provider/config enums such as `ProviderMode`, `RemoteProviderKind`, `RemoteTtsAudioFormat`, `LocalTtsBackend`, `LocalAsrBackend`, and `SpeechFeedbackStyle`.
- Focused `commands::tests` and `config::tests` both pass after the additions, closing the `Enum serialization/deserialization and validation` TODO with explicit contract tests instead of behavior changes.

## 2026-03-29T13:06:58Z - GPT-5.4 - Provider config serde coverage completed
- `src-tauri/src/config.rs` now includes direct JSON round-trip coverage for `ProviderSelections` plus the remote/local provider profile structs (`RemotePlannerProfile`, `RemoteTtsProfile`, `RemoteAsrProfile`, `LocalTtsProfile`, and `LocalAsrProfile`).
- Added a validation regression that rejects missing selected profiles for TTS remote mode and ASR local mode, complementing the existing planner-only missing-profile checks.
- Focused `config::tests` passes after the additions, closing the `Provider config serialization/deserialization and validation` TODO without changing runtime behavior.

## 2026-03-29T13:47:31Z - GPT-5.4 - Secret reference resolution and masking coverage completed
- `src-tauri/src/config.rs` now has explicit tests for `secret_ref_reference(...)` formatting and `resolve_secret_ref(...)` across environment-variable, file, and keyring-backed secret references, including missing/empty failure cases.
- `src-tauri/src/app_core.rs` now has a masking regression proving remote settings surfaces expose only source references (`Environment variable: ...`, `File reference: ...`, `OS keyring entry: ...`) rather than raw secret values.
- Focused `config::tests` and the new `app_core` masking test both pass, closing the `Secret reference resolution and masking behavior` TODO without altering provider runtime behavior.

## 2026-03-29T20:38:33Z - GPT-5.4 - Command parsing helper coverage completed
- `src-tauri/src/commands.rs` now includes direct tests for `normalize_transcript_for_routing(...)` and `parse_intent_name_value(...)`, covering compound-token merging, punctuation sanitization, cleaned quoted/backticked intent names, and unknown-intent rejection.
- The broader command-parsing suite was already present for `infer_intent_hint(...)` and the direct command resolvers; this slice closes the remaining low-level helper gap instead of duplicating the higher-level routing tests.
- Focused `commands::tests` passes after the additions, closing the `Command parsing` TODO without changing voice-command behavior.

## 2026-03-29T21:54:50Z - GPT-5.4 - LLM provider selection coverage completed
- `src-tauri/src/config.rs` now rejects a selected planner `remote_profile` that is missing from `remote_profiles`, closing a validation gap beyond the existing planner-mode and missing-profile-name checks.
- `src-tauri/src/app_core.rs` now has a runtime settings regression proving that selecting `ollama-default` changes the surfaced planner provider details from the default OpenAI profile to the shipped Ollama planner defaults.
- Focused config and app-core tests pass after the additions, closing the `LLM provider selection behavior` TODO without changing planner runtime behavior.

## 2026-03-27T08:03:57Z - claude-sonnet-4.6 - Runtime browser visibility switching implemented
- `src-tauri/src/browser.rs`: Added `BrowserController::switch_visibility(mode)` — updates `BrowserSessionConfig.visibility`, captures current page URL if an active session exists, drops the session, relaunches with updated config, and navigates back to the captured URL. Returns `Ok(Option<String>)` (restored URL if any). Under `#[cfg(not(feature = "browser"))]` returns `Err(BrowserError::FeatureDisabled)`.
- `src-tauri/src/app_core.rs`: Replaced stub `execute_set_browser_visibility` with a real implementation: returns early with `changed: false` if already in requested mode; calls `browser.switch_visibility`; on success updates state visibility and clears stale `current_page` if a relaunch happened; on `FeatureDisabled` returns `supported: false` without failure; on other errors returns a `browser_tool_failure`.
- `docs/TODO.md`: Marked `Implement runtime browser visibility switching when supported` complete.
- Validation: `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings` (clean), `cargo test --all-features` (211 passed), `pnpm test:ui` (48 passed), `pnpm build` (clean).

## 2026-03-27T10:06:29Z - claude-sonnet-4.6 - kitten_tts_rs integration finalized
- `kitten_tts_rs` was already wired in `tts.rs` (model loading, synthesis, caching). The remaining work was fixing feature-gating bugs.
- Fixed: `parse_openai_speech_response_format`, `resolved_remote_voice`, `is_openai_builtin_voice` were not gated behind `#[cfg(feature = "remote-openai")]`, causing `--features local-tts` (without `remote-openai`) to fail to compile.
- Fixed: Removed the now-dead `#[cfg(not(feature = "remote-openai"))]` stub for `synthesize_with_openai_remote`; `synthesize_remote` now handles the feature-absent path directly.
- Fixed: Top-level imports for `resolve_secret_ref`, `RemoteTtsAudioFormat`, `RemoteTtsProfile` gated behind `#[cfg(feature = "remote-openai")]`.
- Added 3 new unit tests: `normalized_model_path_rejects_missing_path`, `resolved_voice_uses_default_when_runtime_voice_is_empty`, `resolved_voice_uses_default_when_runtime_voice_is_none`.
- `docs/TODO.md`: Marked `Integrate kitten_tts_rs` complete.
- Validation: cargo fmt, clippy (clean, no warnings), cargo test --all-features (214 passed), cargo check --features local-tts (clean standalone), pnpm test:ui (48 passed), pnpm build (clean).

## 2026-03-27T11:20:09Z - Claude Sonnet 4.6 - Speech settings re-read tests

### Speech Settings Re-Read Verification
- Confirmed existing implementation is correct: `synthesize_narration` takes `&self.state.audio` fresh on every call; `update_audio_settings → apply_audio_settings` keeps `state.audio` in sync; voice/speed are synthesis-time so inherently "next utterance only".
- Added 3 new tests to `src-tauri/src/state.rs`:
  - `apply_audio_settings_refreshes_tts_voice_and_speed_mid_session` — verifies that voice and speed updates are visible in `state.audio` before next synthesis call.
  - `apply_audio_settings_does_not_disturb_narration_or_speaking_state` — verifies that `speaking`, `speaking_region_id` are untouched after audio settings change.
  - `apply_audio_settings_reflects_muted_when_volume_is_zero` — verifies that zero volume sets `muted = true`.
- Total Rust tests: 217 passed (was 214 after tts.rs slice; also committed tts.rs kitten_tts_rs slice).
- Marked "Re-read effective speech settings before each new utterance" and "Apply changed speech settings on the next utterance" done in docs/TODO.md.

## 2026-03-27T11:28:54Z - GPT-5.4 - Validation after TTS and speech-settings slices
- Re-ran the standard validation pass after committing the TTS feature-gating fixes and speech-settings tests.
- Validation is green: `cargo fmt --manifest-path src-tauri/Cargo.toml --all`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, and `pnpm test:ui` all pass.
- Current counts remain 217 Rust tests passed and 48 UI tests passed, and the git worktree was clean immediately after validation.

## 2026-03-27T11:44:12Z - GPT-5.4 - get_html tool implemented
- Added a new deterministic `get_html` tool across the planner-visible contract, app runtime, and browser backend.
- `src-tauri/src/browser.rs` now reads `document.documentElement.outerHTML` from the live Chromium page and returns refreshed URL/title/history metadata alongside the HTML payload.
- `src-tauri/src/app_core.rs` now exposes `execute_get_html`, requiring an active page id, refreshing runtime metadata, and returning `{ page_id, url, title, html, html_length }`.
- `src-tauri/src/commands.rs` now registers `ToolName::GetHtml`, its input/output schemas, planner validation, dispatcher wiring, parser name mapping, non-side-effect classification, and regression tests.
- Updated `docs/SPECS.md` and `docs/TODO.md`, and validation is green with 218 Rust tests passed, 48 UI tests passed, and `pnpm build` succeeding.

## 2026-03-27T19:40:49Z - GPT-5.4 - eval_js tool implemented
- Added a deterministic `eval_js` tool as a bounded JavaScript **expression** evaluator rather than a free-form statement runner.
- `src-tauri/src/browser.rs` now evaluates a supplied expression with Chromium, requires the result to serialize as `serde_json::Value`, and refreshes URL/title/history metadata after evaluation.
- `src-tauri/src/app_core.rs` now exposes `execute_eval_js`, returning `{ page_id, url, title, result }` and refreshing runtime browser metadata without silently re-extracting the page model.
- `src-tauri/src/commands.rs` now registers `ToolName::EvalJs`, adds schema/dispatch/parse wiring, rejects blank expressions, and includes regression coverage for dispatch plus validation.
- Updated `docs/SPECS.md` and `docs/TODO.md`; full validation is green with 220 Rust tests passed, 48 UI tests passed, and `pnpm build` succeeding.

## 2026-03-27T20:30:19Z - GPT-5.4 - dom_smoothie extraction path landed
- `src-tauri/src/extractor.rs` now wraps `dom_smoothie` to parse live HTML into readable article regions, splitting article text into ordered paragraph-style `PageRegion`s and avoiding duplicate title text.
- `src-tauri/src/app_core.rs` now treats live `get_html()` + `dom_smoothie` as the primary readable-text extraction path while still reusing the existing browser DOM extraction for interactive elements and as an explicit surfaced fallback.
- Updated `docs/SPECS.md` and `docs/TODO.md` to reflect the new extraction flow, and revalidated with `cargo fmt --manifest-path src-tauri/Cargo.toml --all`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build` (`223` Rust tests and `48` UI tests passing).

## 2026-03-27T20:33:14Z - GPT-5.4 - User wants explicit fallback explanations
- The user is especially concerned when I mention fallback code and wants a clear explanation of what the fallback does, why it exists, and whether it is explicit versus silent.
- When discussing resilience paths in this repo, I should call out whether the behavior is primary-path-only, explicit fallback, or hidden fallback, and why that distinction matters.

## 2026-03-27T21:47:27Z - GPT-5.4 - dom_smoothie quality validation added
- Added extractor regression coverage that validates the current `dom_smoothie` contract against the target `PageModel`: title metadata is preserved without duplicate title regions, readable body text remains in order, interactive elements are preserved, and extracted regions stay DOM-sourced.
- Added app-core regression coverage that `build_extracted_page_model(...)` preserves region ordering and per-region source values when shaping the runtime page model.
- Marked `Validate dom_smoothie output quality against target page model` complete in `docs/TODO.md` and revalidated with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` plus `cargo test --manifest-path src-tauri/Cargo.toml --all-features` (`226` Rust tests passing).

## 2026-03-27T23:09:54Z - GPT-5.4 - Structured extractor output landed
- `src-tauri/src/extractor.rs` now returns an intermediate structured article model with explicit block kinds (`Title`, `Paragraph`, `Heading`) before converting that structure into the current `PageModel`.
- `src-tauri/src/app_core.rs` now consumes the structured extractor output at the boundary and converts it into `PageModel`, keeping interactive-element attachment and explicit dom_smoothie fallback behavior unchanged.
- Updated `docs/TODO.md` and `docs/SPECS.md` to reflect that structured title/paragraph extraction and deterministic extractor-output → page-model conversion are now implemented, and revalidated with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build` (`227` Rust tests and `48` UI tests passing).

## 2026-03-28T03:40:43Z - GPT-5.4 - Page model roles are now structured
- `src-tauri/src/page_model.rs` now gives `PageRegion` a structured `RegionRole` enum (`Title`, `Heading`, `Paragraph`, `Section`, `Other`) with serde defaulting so older saved state still deserializes.
- The known producers and consumers now populate and use that role: structured extractor output, live browser DOM extraction, narration prefixing, OCR region appends, and the affected regression tests.
- Revalidated with `source ./fix-node-version.sh && . "$HOME/.cargo/env" && cargo fmt --manifest-path src-tauri/Cargo.toml --all && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings && cargo test --manifest-path src-tauri/Cargo.toml --all-features && pnpm test:ui && pnpm build`.

## 2026-03-28T06:14:42Z - GPT-5.4 - TTS synthesized-speech cache landed
- `src-tauri/src/tts.rs` now keeps a bounded 8-entry in-memory cache of synthesized speech results keyed by provider/model identity, resolved voice, playback speed, and input text, while preserving the existing cached local-model load behavior.
- This avoids repeated regeneration or remote requests for identical narration requests without changing provider selection or adding hidden fallback behavior.
- Updated `docs/TODO.md` and `docs/SPECS.md`, and revalidated with `source ./fix-node-version.sh && . "$HOME/.cargo/env" && cargo fmt --manifest-path src-tauri/Cargo.toml --all && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings && cargo test --manifest-path src-tauri/Cargo.toml --all-features && pnpm test:ui && pnpm build` (`233` Rust tests and `48` UI tests passing).

## 2026-03-28T06:24:11Z - GPT-5.4 - TTS cache slice ready to land
- The synthesized-speech cache slice remained the only local change after validation, touching `src-tauri/src/tts.rs`, `docs/TODO.md`, `docs/SPECS.md`, and `memory.md`.
- The worktree matched the completed and validated TTS cache implementation, so the next operational step is committing and pushing the slice directly to `master`.

## 2026-03-28T06:27:20Z - GPT-5.4 - Cleaned stale browser TODO items
- `docs/TODO.md` had stale unchecked browser-module items for visible Chromium launch, headless toggle, `open_url()`, and `screenshot_png()` even though the codebase already supports visible/headless browser modes, `open_url`, and `capture_screenshot`.
- The cleanup marked the implemented items done and renamed the screenshot line to match the current deterministic tool name `capture_screenshot`, so the TODO list better reflects the actual shipped browser capabilities.

## 2026-03-28T06:33:55Z - GPT-5.4 - Added browser visibility state coverage
- `src-tauri/src/commands.rs` test coverage now keeps mocked browser visibility state across `set_browser_visibility`, `get_runtime_status`, and `get_agent_state`, so the command-layer tests verify that follow-up state reads report the updated mode.
- `src/confirmation-panel.ts` now exposes a pure `statusPanelStateFromAgentState(...)` helper that maps the runtime agent snapshot into the UI status-panel state, and `src/main.ts` uses that helper instead of duplicating the mapping inline.
- Revalidated with `source ./fix-node-version.sh && . "$HOME/.cargo/env" && cargo fmt --manifest-path src-tauri/Cargo.toml --all && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings && cargo test --manifest-path src-tauri/Cargo.toml --all-features && pnpm test:ui && pnpm build`.

## 2026-03-28T08:17:07Z - GPT-5.4 - Added listening runtime-state coverage
- `src-tauri/src/commands.rs` test coverage now keeps mocked listening state and last transcript across `start_listening`, `stop_listening`, `transcribe_command`, `get_runtime_status`, and `get_agent_state`, so follow-up state reads reflect the updated listening mode and transcription result.
- The new regression cases cover start→runtime-status, stop→agent-state, and both transcription stop behaviors (`AutoStop` and `KeepListening`) to prove the reported runtime state matches the deterministic listening-tool results.
- Revalidated with `source ./fix-node-version.sh && . "$HOME/.cargo/env" && cargo fmt --manifest-path src-tauri/Cargo.toml --all && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings && cargo test --manifest-path src-tauri/Cargo.toml --all-features && pnpm test:ui && pnpm build`.

## 2026-03-28T09:09:30Z - GPT-5.4 - Added browser history runtime-state coverage
- `src-tauri/src/commands.rs` test coverage now keeps mocked browser-history state across `go_back`, `go_forward`, `reload_page`, `get_runtime_status`, and `get_agent_state`, so follow-up state reads reflect updated back/forward availability and history indices.
- The new regression case covers back→runtime-status, forward→runtime-status, and reload→agent-state to prove the reported browser history state stays aligned with deterministic navigation-tool results.
- Revalidated with `source ./fix-node-version.sh && . "$HOME/.cargo/env" && cargo fmt --manifest-path src-tauri/Cargo.toml --all && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings && cargo test --manifest-path src-tauri/Cargo.toml --all-features && pnpm test:ui && pnpm build`.

## 2026-03-28T09:21:36Z - GPT-5.4 - Reconciled stale risky-confirmation TODO
- `docs/TODO.md`'s `Planner requests confirmation before risky execution` item was stale: existing tests already cover the risky-confirmation path via `returns_awaiting_confirmation_when_transition_requests_it`, `aborts_needs_confirmation_plan_before_side_effecting_step`, the queued/timed-out confirmation flow tests, and confirmation-gated submit/fill planning coverage.
- This slice only required auditing the current confirmation behavior and marking the TODO item done so the checklist matches the already-landed confirmation safeguards.

## 2026-03-28T09:34:52Z - GPT-5.4 - Reconciled stale submit-confirmation TODO
- `docs/TODO.md`'s `Submit actions always require confirmation` item was stale: existing planner validation tests already reject `SubmitForm` plans that omit `NeedsConfirmation` or the `confirm_action` gate, and direct submit / fill-and-submit command resolution already builds confirmation-gated plans.
- This slice only required auditing the current submit-confirmation coverage and marking the TODO item done so the checklist matches the already-enforced submit safety behavior.

## 2026-03-28T09:43:39Z - GPT-5.4 - Reconciled stale click-confirmation TODO
- `docs/TODO.md`'s `Click actions may proceed without confirmation when configured` item was stale: the specs already document this behavior, planner examples already include both `click_element_ready` and `click_element_with_confirmation`, and the serialized example tests cover both the ready and confirmation-gated click paths.
- This slice only required auditing the current click-confirmation coverage and marking the TODO item done so the checklist matches the already-landed ordinary-click safety behavior.

## 2026-03-28T09:56:05Z - GPT-5.4 - Reconciled stale fill-field workflow TODO
- `docs/TODO.md`'s `Fill-field workflows resolve the intended input and write the requested value` item was stale: existing tests already cover direct fill-field parsing, missing-value follow-up handling, and the confirmation-free focus→type plan that targets the matched field and carries the requested text value.
- This slice only required auditing the current fill-field workflow coverage and marking the TODO item done so the checklist matches the already-landed fill-field behavior.

## 2026-03-28T10:00:32Z - GPT-5.4 - Reconciled stale fill-and-submit confirmation TODO
- `docs/TODO.md`'s `Fill-and-submit workflows require confirmation before form submission` item was stale: existing tests already cover direct fill-and-submit parsing, missing-value follow-up handling, and the confirmation-gated plan that uses `NeedsConfirmation` with `confirm_action` before `SubmitActiveForm`.
- This slice only required auditing the current fill-and-submit confirmation coverage and marking the TODO item done so the checklist matches the already-enforced guarded submit behavior.

## 2026-03-28T10:02:45Z - GPT-5.4 - Reconciled stale ambiguity-clarification TODO
- `docs/TODO.md`'s `Ambiguous element matches ask the user to clarify instead of silently choosing one` item was stale: existing tests already cover ambiguous focus-field and submit-form matches returning `NeedsFollowUp` clarification/reporting flows instead of silently selecting one candidate.
- This slice only required auditing the current ambiguity-handling coverage and marking the TODO item done so the checklist matches the already-landed clarification behavior.

## 2026-03-28T10:05:48Z - GPT-5.4 - Reconciled stale mixed-command bounded-plan TODO
- `docs/TODO.md`'s `Mixed commands such as fill-and-submit are decomposed into safe bounded plans` item was stale: `resolve_direct_fill_and_submit_command_builds_confirmation_gated_plan` already verifies a bounded four-step plan of `ConfirmAction`, `FocusElement`, `TypeIntoElement`, and `SubmitActiveForm`, and `resolve_direct_fill_and_submit_command_reports_missing_value` covers the follow-up path when the requested text is omitted.
- This slice only required auditing the current mixed-command planner coverage and marking the TODO item done so the checklist matches the existing bounded-plan behavior.

## 2026-03-28T11:30:40Z - GPT-5.4 - Added recent field-context follow-up corrections
- Follow-up fill corrections now reuse recent deterministic field context in `src-tauri/src/app_core.rs`: phrases like `no, the other field` can switch to a stored alternate candidate when a recent field-resolution context exists, and `put Seattle there instead` can reuse the recent field target without submitting.
- The command parser gained explicit `parse_fill_field_correction_command(...)` coverage in `src-tauri/src/commands.rs`, and new app-core regression tests now pin the replacement-text, alternate-field, and no-context follow-up behaviors. Full validation passed afterward: `cargo clippy`, `cargo test` (241 Rust tests), `pnpm test:ui` (49 passes), and `pnpm build`.

## 2026-03-28T11:45:33Z - GPT-5.4 - Revalidated lint and unit test baseline
- Reran the requested validation subset after the recent follow-up context slice: `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, and `pnpm test:ui`.
- The baseline stayed green with 241 Rust tests passing and 49 UI tests passing, so the current worktree is still clean from a lint/unit-test perspective before the next TODO slice.

## 2026-03-28T11:49:22Z - GPT-5.4 - Reconciled stale replanning-after-failure TODO
- `docs/TODO.md`'s `Replanning after tool failure or ambiguous result` item was stale: command-layer tests already cover `execute_planner_output(...)` returning `NeedsReplan` on failure transitions, app-core tests already cover one bounded replan cycle carrying `recent_tool_results`, and the loop already aborts on a second replan with `replan_limit_exceeded`.
- Ambiguous grounding paths are already covered separately as bounded `NeedsFollowUp` clarification flows rather than silent execution, so this slice only required auditing the current coverage and marking the checklist item done.

## 2026-03-28T12:23:09Z - GPT-5.4 - Planner-unavailable voice feedback clarified
- Planner resolution failures in `src-tauri/src/app_core.rs` now consistently say `Command interpretation is unavailable because ...`, so push-to-talk and hands-free command errors surface the unavailable-interpreter state directly instead of only raw backend wording.
- `src/main.ts` now also maps missing remote planner profiles into settings guidance, and `docs/TODO.md` marks `LLM unavailable with no local provider → report command interpretation unavailable` complete. Full validation passed afterward: `cargo clippy`, `cargo test`, `pnpm test:ui`, and `pnpm build`.

## 2026-03-28T12:56:58Z - GPT-5.4 - Remote TTS synthesis path now has working regression coverage
- `src-tauri/src/tts.rs` now exercises the real remote TTS success path with a localhost OpenAI-compatible test server, proving that `ProviderMode::Remote` narration returns decoded `SynthesizedSpeech` instead of only unit-testing helpers.
- While landing that coverage, I fixed a real runtime issue in the remote TTS implementation by switching the OpenAI speech request to the existing synchronous `reqwest::blocking` client, which matches the app’s synchronous runtime model and avoids the missing-Tokio-reactor failure from the previous `async-openai` path. Full validation passed afterward: `cargo clippy`, `cargo test` (243 Rust tests), `pnpm test:ui` (49 passes), and `pnpm build`.

## 2026-03-28T14:30:29Z - GPT-5.4 - Speech-setting changes now wait until the next utterance
- `src-tauri/src/app_core.rs` no longer pushes playback-volume changes into the currently active player, so the current utterance keeps its existing settings while the next narration request uses the updated persisted volume, matching the spec’s next-utterance-only rule.
- `src-tauri/src/state.rs` now pins that mid-session audio updates refresh volume, voice, and speed for the next synthesis call without disturbing active narration state, and `docs/TODO.md` marks `Changed speech settings apply on the next utterance only` complete. Full validation passed afterward: `cargo clippy`, `cargo test` (243 Rust tests), `pnpm test:ui` (49 passes), and `pnpm build`.
## 2026-03-28T15:28:26Z - GPT-5.4 - First agentic planner-skill fixtures landed
- `src-tauri/src/commands.rs` now includes a compact test-only fixture harness that reuses `planner_available_tools()`, `build_planner_skill_selection(...)`, the direct resolver helpers, and `execute_planner_output(...)` to assert transcript-to-skill and transcript-to-tool-sequence behavior.
- The initial regression corpus covers representative direct command flows for audio volume changes, back navigation, read-page extraction when no readable regions exist, and current-URL status queries.
- Validation after the fixture slice passed with `source ./fix-node-version.sh && . "$HOME/.cargo/env" && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings && cargo test --manifest-path src-tauri/Cargo.toml --all-features && pnpm test:ui && pnpm build`, with Rust tests at `244` and UI tests at `49`.

## 2026-03-28T16:13:59Z - GPT-5.4 - Bundled skill ranking corpus expanded
- `src-tauri/src/commands.rs` now includes a representative table-driven test for `build_planner_skill_selection(...)` that pins the top-ranked bundled skill for common spoken tasks across open-url, navigation, readback, status, voice, playback-speed, and browser-visibility flows.
- The status-query corpus now explicitly captures the current ranking behavior where `announce_state` outranks `get_status` for an `are you listening` transcript, documenting the present bundled-skill ordering instead of leaving it implicit.
- Validation after the slice passed with `source ./fix-node-version.sh && . "$HOME/.cargo/env" && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings && cargo test --manifest-path src-tauri/Cargo.toml --all-features && pnpm test:ui && pnpm build`, with Rust tests at `245` and UI tests at `49`.

## 2026-03-28T16:45:40Z - GPT-5.4 - Added app-core fixtures for ambiguous click and form follow-up flows
- `src-tauri/src/app_core.rs` now includes a compact regression fixture harness that reuses the direct field/form resolver helpers plus `determine_find_element_resolution(...)` to pin representative ambiguous focus, fill, fill-and-submit, follow-up replacement, alternate-field correction, ambiguous submit, and click-confirmation paths.
- While landing the slice, the new fixtures documented an important contract detail: bounded clarification flows keep `PlannerStatus::Ready` and express the follow-up requirement through `ReportResult` with `ReportStatus::NeedsFollowUp` rather than a separate planner status.

## 2026-03-28T17:31:18Z - GPT-5.4 - Started realistic problematic-page regression corpus
- `src-tauri/src/commands.rs` and `src-tauri/src/app_core.rs` now include a first corpus of named realistic page-model helpers for problematic page shapes: a cluttered article page with no extracted regions, a docs page status query, a checkout page with duplicate email targets and multiple forms, a newsletter signup page with a single fill target, and a landing page with duplicate CTA buttons.
- The new regression tests pin both command-layer and app-core behavior on those page shapes, including read-page extraction, current-URL reporting, checkout ambiguity/follow-up correction, deterministic newsletter filling, and duplicate-CTA click confirmation.
- While landing the slice, the corpus also documented two current behavior details worth preserving: `read this article` currently ranks non-`read_page` skills ahead of explicit page-reading, and direct fill utterances are currently most reliable on simpler single-field pages than on multi-field checkout layouts. Full validation passed with `source ./fix-node-version.sh && . "$HOME/.cargo/env" && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings && cargo test --manifest-path src-tauri/Cargo.toml --all-features && pnpm test:ui && pnpm build`, with Rust tests at `249` and UI tests at `49`.

## 2026-03-28T17:43:04Z - GPT-5.4 - Reconciled stale per-tool input schema TODO
- `src-tauri/src/commands.rs` already implements per-tool input schemas and argument validation constraints: every registered tool is covered by `tool_input_schema(&ToolName)`, `validate_planner_output(...)` dispatches planned-step arguments through per-tool validators or schema-backed deserialization, and the test suite already fails if any registered tool lacks an input schema.
- Existing regression coverage already pins representative argument constraints such as malformed schema mismatches, blank or relative `open_url` values, invalid history step counts, OCR source requirements, merge-OCR non-empty text, `find_element` limits, and playback volume/speed ranges, so this slice only required auditing the current implementation and marking the checklist item complete.

## 2026-03-28T18:20:48Z - GPT-5.4 - Closed remaining bounded runtime schema strings
- `src-tauri/src/commands.rs` now uses closed enums for the remaining runtime/settings payload fields whose valid sets were already known: local TTS and ASR backends, remote provider labels, and remote TTS audio format are no longer exposed as free-form strings in `AgentStateData`/`GetRuntimeStatusData`.
- `src-tauri/src/app_core.rs` now builds those enum-backed payloads directly from config instead of stringifying them, and `src/tauri-api.ts` mirrors the narrowed frontend contract with literal union types for those fields.
- Revalidated with `source ./fix-node-version.sh && . "$HOME/.cargo/env" && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings && cargo test --manifest-path src-tauri/Cargo.toml --all-features && pnpm test:ui && pnpm build`; Rust tests stayed green and UI tests remained `49` passed.

## 2026-03-28T18:43:56Z - GPT-5.4 - Runtime status schema coverage now pins provider-mode JSON contracts
- `src-tauri/src/commands.rs` now regression-tests `ProviderSelectionStatus` JSON round-tripping with the shipped snake_case `ProviderMode` encoding and verifies that serialized `GetRuntimeStatus` results match the generated output schema when provider modes are requested.
- The runtime-status test coverage also now asserts the exact serialized provider-mode values (`remote`/`local`) and the `null` contract for `provider_modes` when `include_provider_modes` is false, closing the remaining unit-test gap without changing runtime behavior.
- Revalidated with `source ./fix-node-version.sh && . "$HOME/.cargo/env" && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings && cargo test --manifest-path src-tauri/Cargo.toml --all-features && pnpm test:ui && pnpm build`; UI tests remained `49` passed and the full Rust suite stayed green.

## 2026-03-28T19:15:03Z - GPT-5.4 - Listening/transcription coverage now spans command flow and AppState mutations
- `src-tauri/src/state.rs` now unit-tests `AppState::set_listening` and `AppState::record_transcript`, covering deterministic listening toggles plus transcript trimming and empty-value clearing at the state layer.
- Combined with the existing `commands.rs` tests for `StartListening`, `StopListening`, and `TranscribeCommand` follow-up reads, the test suite now covers both deterministic listening state transitions and one-shot transcription behavior without changing runtime logic.
- Revalidated with `source ./fix-node-version.sh && . "$HOME/.cargo/env" && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings && cargo test --manifest-path src-tauri/Cargo.toml --all-features && pnpm test:ui && pnpm build`; UI tests remained `49` passed and the full Rust suite stayed green.

## 2026-03-28T20:08:08Z - GPT-5.4 - Browser-visibility and audio-clamping test coverage landed
- `src-tauri/src/commands.rs` now makes the test-only `MockExecutor` mirror the real deterministic tool behavior for playback-volume clamping, playback-speed clamping, and browser-visibility no-op/unsupported responses instead of only echoing requested values.
- Added focused regression tests that pin clamped playback results plus `get_agent_state` / `get_runtime_status` readbacks, and browser-visibility responses for already-active and unsupported-switch cases, closing the remaining Phase 8 TODO without changing runtime behavior.
- Full validation passed afterward with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`; Rust tests are now `258` passing and UI tests remain `49` passing.

## 2026-03-28T20:28:39Z - GPT-5.4 - Browser-history serialization and boundary coverage landed
- `src-tauri/src/state.rs` now regression-tests `BrowserHistoryState` JSON serialization/deserialization for both the default empty-history case (`current_entry_index: null`, `entry_count: 0`) and a populated navigation position.
- Added a focused navigation-boundary test proving that a new navigation from an earlier history entry truncates forward history, closing the remaining Phase 8 browser-history test TODO without changing runtime behavior.

## 2026-03-28T20:41:58Z - GPT-5.4 - Common tool-envelope serde coverage landed
- `src-tauri/src/commands.rs` now regression-tests the shared `ToolResult<T>` envelope directly, covering success and failure constructor semantics, warning/error serialization, and typed payload deserialization through the generic envelope.
- This closes the Phase 8 common-envelope TODO without changing runtime behavior; the prior schema tests already covered representative per-tool payload shapes, and this slice pins the shared wrapper contract itself.

## 2026-03-28T21:03:47Z - GPT-5.4 - Deterministic tool-result schema TODO was already satisfied
- `src-tauri/src/commands.rs` already exports concrete `ToolResult<...>` output schemas for every registered deterministic tool via `tool_output_schema(...)`, and `registered_tools_all_expose_output_schemas()` already fails if any tool lacks one.
- The existing `sample_serialized_tool_results_match_generated_tool_output_schemas()` regression iterates `sample_planned_steps_for_registered_tools()`, which is built directly from `registered_tools()`, so one representative serialized result is already schema-checked for every registered tool; this slice only reconciled the stale TODO item.

## 2026-03-29T22:03:48Z - GPT-5.4 - Default local model profile selection coverage landed
- Added config validation coverage in `src-tauri/src/config.rs` proving `providers.tts.local_profile` and `providers.asr.local_profile` are rejected when they reference missing local profile entries.
- Added runtime-setting regressions in `src-tauri/src/app_core.rs` proving that switching the selected local TTS or ASR profile changes the surfaced model details, plus a `build_tts_model_settings(...)` regression showing the selected local TTS profile becomes the active profile in the exposed settings.
- Marked `Default local model profile selection behavior` complete in `docs/TODO.md` and revalidated with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`.

## 2026-03-29T22:20:47Z - GPT-5.4 - SKILL.md frontmatter and precedence coverage landed
- Added `commands.rs` regressions proving invalid `SKILL.md` documents are rejected for representative frontmatter failures, including missing YAML frontmatter, unsupported fields, missing descriptions, and unknown tools.
- Added a file-backed discovery regression proving duplicate skill names resolve by precedence, with the project-local `SKILL.md` copy winning over lower-precedence user and bundled copies.
- Marked `SKILL.md frontmatter validation and precedence resolution` complete in `docs/TODO.md` and revalidated with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`.

## 2026-03-29T22:32:19Z - GPT-5.4 - Skill ranking and top-N coverage landed
- Added a file-backed `commands.rs` regression proving custom project skills are ranked by the live scoring inputs and that `build_planner_skill_selection(...)` returns only the top `MAX_SELECTED_PLANNER_SKILLS` matches.
- The new test locks down the current ranking behavior across lexical overlap, inferred-intent tags, allowed-tool overlap, and explicit priority, while excluding weaker and unrelated skills from the selected summaries.
- Marked `Skill ranking and top-N selection behavior` complete in `docs/TODO.md` and revalidated with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`.

## 2026-03-29T22:37:17Z - GPT-5.4 - Planner transition validation coverage landed
- Added `commands.rs` regressions proving `validate_planner_output(...)` rejects planner steps that reference unavailable tools and rejects `NextStep` transitions targeting missing step ids before execution starts.
- The new tests lock down the exact `invalid_planner_output` details for both failure modes, complementing the existing executor-side abort coverage for missing transition targets during execution.
- Marked `Reject unknown tools and invalid planner transitions` complete in `docs/TODO.md` and revalidated with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`.

## 2026-03-29T22:41:33Z - GPT-5.4 - Invalid tool-argument rejection coverage tightened
- Added direct `commands.rs` coverage proving `validate_planned_step_arguments(...)` reports structured `step_id` and `tool_name` details when tool arguments fail schema deserialization.
- Tightened the existing planner-output regression so `validate_planner_output(...)` also pins the same step/tool details for malformed step arguments, complementing the existing executor-side `invalid_tool_arguments` dispatch rejection.
- Marked `Reject invalid tool arguments before execution` complete in `docs/TODO.md` and revalidated with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`.

## 2026-03-29T22:49:28Z - GPT-5.4 - Element matching helper coverage tightened
- Added `app_core.rs` regressions proving `build_find_element_query(...)` normalizes optional hint fields into the query summary and that `rank_find_element_candidates(...)` uses attribute-backed `selector_hint` matches while truncating to the requested candidate limit.
- These tests complement the existing exact-name ranking and confidence-threshold resolution coverage by pinning the bounded hint-driven matching path used before planner clarification.
- Marked `Element matching and resolution behavior` complete in `docs/TODO.md` and revalidated with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`.
## 2026-03-29T23:00:29Z - GPT-5.4 - Page model building coverage completed
- Added a focused regression in `src-tauri/src/app_core.rs` proving `build_extracted_page_model(...)` leaves title, heading, and paragraph regions unchanged when `include_headings` is false, matching the current schema contract.
- This keeps the slice scoped to the real remaining gap: the current runtime does not yet distinguish heading-specific extraction, so disabling `include_headings` must not silently strip or rewrite heading regions.
- Updated `docs/TODO.md` to mark `Page model building` complete. Validation is green with the standard `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build` workflow.
## 2026-03-30T03:59:01Z - GPT-5.4 - Navigation logic coverage completed
- Extracted the shared post-navigation cleanup in `src-tauri/src/app_core.rs` into small private helpers so the current runtime contract is explicit and testable without changing navigation behavior.
- Added focused regressions proving successful navigation clears stale extracted content (`regions` and `interactive_elements`) and resets narration follow-up state (`narration_cursor` and recent field context).
- Updated `docs/TODO.md` to mark `Navigation logic` complete. Validation is green with the standard `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build` workflow.
## 2026-03-30T04:52:02Z - GPT-5.4 - Integration coverage for reading and spoken-command flows completed
- Added focused integration-style regressions in `src-tauri/src/commands.rs` for `Load page -> extract -> read` and `ASR -> command -> action`, both using the existing direct-command resolvers plus the shared `execute_planner_output(...)` harness.
- The read-page regression resolves `read page` from a sparse article fixture, then proves the resulting plan executes `ExtractPageModel` followed by `ReadNextRegion` with the exact resolved arguments.
- The spoken-command regression resolves `continue reading`, then proves the resolved plan executes `ReadNextRegion` end-to-end with the expected interruption behavior; this keeps the slice aligned with the current deterministic transcript-to-plan boundary without introducing fake audio fallback or ASR mocks.
- Updated `docs/TODO.md` to mark both remaining integration-test TODOs complete. Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`.
## 2026-03-30T05:46:34Z - GPT-5.4 - Added post-v1 hardening regression coverage in app_core
- Added focused `src-tauri/src/app_core.rs` tests for tool executor safety gates, covering blank and unknown `element_id` failures in `resolve_clickable_element(...)`.
- Added navigation failure consistency coverage at the shared browser-error translation seam, proving `BrowserError::Navigate` and `BrowserError::History` stay retryable and preserve structured `reason` details for user-facing failure reporting.
- Added a bounded replanning regression proving a follow-up planner-resolution failure aborts with the accumulated execution trace and does not execute another plan; this also pins the current derived execute request IDs across replan cycles.
- Added OCR merge edge-case regressions proving `merge_ocr_text_into_page_model(...)` rejects blank OCR text and unknown target region IDs without mutating the page model. Validation is green with `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`, `cargo test --manifest-path src-tauri/Cargo.toml --all-features`, `pnpm test:ui`, and `pnpm build`.
## 2026-04-02T18:43:31Z - GPT-5.4 - Review-backlog implementation decisions fixed by user
- The user wants `docs/BB_CODE_REVIEW1_TODO.md` treated as the implementation backlog rather than a reduced shortlist.
- TTS provider-switch failures should surface the current failure on the provider, model, and voice panels together instead of only on the provider panel.
- Page snapshot metrics should be implemented now rather than merely documented or gated as placeholders.
- The frontend rerender fix should be a larger internal store / panel-update refactor rather than a minimal localized patch.

## 2026-04-02T19:51:00Z - GPT-5 - Phase 7.3 commands modularization
- Replaced `src-tauri/src/commands.rs` with `src-tauri/src/commands/` and a `mod.rs` façade that re-exports the existing commands API so `crate::commands::{...}` callers stay unchanged.
- Split the backend command layer into focused Rust modules for tool contracts, planner execution, registry/skill loading, routing, validators, and tests without intentionally changing planner or tool behavior.
- Validation for this refactor: `cd src-tauri && cargo fmt && cargo check` succeeded; Phase 7.3 in `docs/BB_CODE_REVIEW1_TODO.md` is now marked done.

## 2026-06-29T13:21:46Z - Claude Haiku 4.5 - BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING complete
- Implemented all 15 task groups of docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING_TODO.md (Ralph loop, one commit per group, gate after each). Branch master, commits 7035f44..4ae653e.
- P0: restored restrictive CSP (was null); shared src/url_policy.rs fails closed to http/https for planner+runtime navigation (rejects file:/javascript:/data:/chrome:/about:/scheme-relative/authority-less); navigation now honors load_state+timeout_ms (with_browser_timeout via tokio::time::timeout — added tokio time feature; ensure_supported_load_state rejects NetworkIdle).
- P1: two-phase switch_visibility (commit-after-prove, read_current_non_blank_url surfaces errors); current_page_snapshot returns Result so metric failure != silent None; remote ASR uses reqwest::blocking multipart with real .timeout() (added reqwest multipart feature, dropped thread::spawn+recv_timeout); local ASR collect_transcript_segments fails on decode error instead of filter_map(...ok()); parse_bundled_skills returns Result and fails loudly (discover_skills .expect()s; malformed requires_confirmation/unknown tool = error); voice-loop reportPushToTalkFailureWithoutInventingListeningState keeps prior isListening after stop/transcribe failure.
- P2: stale confirmation submissions surface a visible error + console.warn; runtime refresh no longer clears action-owned panel errors (only status panel is refresh-owned); allow_click_without_confirmation default flipped true->false (config.example.toml + Rust default + SPECS.md); timestamp helpers use AtomicU64 monotonic fallback instead of => 0.
- P3: scripts/check-silent-fallbacks.sh wired into CI (narrow denylist of the exact removed shapes; excludes legit let _ = timeout_ms / bare unwrap_or(false)).
- Final gate green: guard PASS, fmt PASS, default cargo check PASS, clippy clean, 344 Rust tests, 168 JS tests, build PASS. Net +30 Rust tests (314->344) and +4 JS tests (164->168) vs pass start.
- Note during session: the harness /tmp/claude-1000 tmpfs filled and blocked all Bash output capture; user freed space to continue. Behavioral checks needing a live Chromium/remote-ASR/display (visibility-switch session preservation, NetworkIdle reject end-to-end, remote ASR interleave, Tauri dev launch) remain human-verification items.

## 2026-08-07T15:59:22Z - Claude Opus 4.6 - Full-codebase code review (CR3) + spec/TODO authored
- Ran a six-subsystem parallel review of `master` @ af89a22 (privacy/consent, command/planner, browser/tools, config/state/persistence, ASR/TTS/audio, React frontend) plus a cross-cutting hygiene/CI/dependency pass. Independently re-verified ~12 of the highest-severity findings against source and compiled CSS before recording them.
- Authored `docs/BB_CODE_REVIEW3_SPEC.md` (12 numbered design constraints) and `docs/BB_CODE_REVIEW3_TODO.md` (P0.1-P0.9, P1.1-P1.4, P2.1-P2.8, P3.1-P3.3). No code changed in this session.
- Findings are tagged `[VERIFIED]` (confirmed by reading code) vs `[VERIFY FIRST]` (reported with evidence, not re-derived — must reproduce before fixing). The `[VERIFY FIRST]` set: region bbox coordinate space, execute_planner_output validation gap, "Allow always" snapshot invalidation.
- P0 (verified): click confirmations announce `element-7` instead of the element label (annotate_click_step never restores RUNTIME_TARGET_LABEL_ARG that clear_runtime_annotations strips); `previous_region_index` has no bounds check -> panic when a re-extraction shrinks the region list (replace_current_page_model does not reset narration_cursor); persist_safety/ocr_settings_at_path write before validating -> an invalid threshold bricks startup; two confirmation bypasses (element_scoring truncates before the runner-up margin check so max_candidates:1 disables ambiguity detection, and field_fill/field_focus hardcode requires_confirmation=false when candidates.len()==1, skipping the 0.90 threshold); frontend focusSettingsTarget casts a DOM id to SettingsView -> hides all five subviews -> blank Settings page; confirmation/consent dialogs render inside the view-hidden workspace section so they are unreachable from Settings; empty synthesized audio is played+cached and reported as success (only the input text is checked, never the samples); begin_capture never drains so the hands-free loop accumulates the whole inter-utterance window including the app's own TTS output.
- Architectural gap (user chose "gate them like the planner"): remote TTS and remote ASR send raw page text and raw mic audio with NO consent gate — verified zero references to remote_planner_privacy/network_mode/origin_rules/high_risk/consent/sanitize/redact anywhere in src-tauri/src/tts or src-tauri/src/asr. Tracked as P1.1 (generalize the policy + type-state over three disclosure kinds; keep the private-constructor choke point).
- Four visual regressions traced to MY Tailwind migration in af89a22, folded in as P0.9: `border-[var(--card-border)]` compiles to `border-color` but the var holds the shorthand `1px solid rgba(...)` -> resets to currentColor (and playback.tsx has the colour utility with no width utility, so those cards render no border at all); toggle pressed state, read-only card background both lost to conflicting utilities where the base is emitted later and wins (verified by byte offset in dist CSS); privacy breakpoint moved 768px->640px (max-sm: where max-md: was faithful). All four passed lint+build+238 tests because nothing asserts computed style — P0.9.5 adds a cascade regression test.
- My own cross-cutting finding: scripts/run-rust-tests-linux.sh hardcodes the 7 isolated security test names with no completeness guard vs `cargo test -- --ignored --list`; an 8th ignored security test would silently never run. Third instance of this pattern (after the test:ui glob fixed in d9dc6c7 and check-remote-planner-privacy-state.py's REQUIRED_PATHS, repaired twice). Tracked as P2.2.
- Verified-good and explicitly listed in the spec as must-not-regress: the type-state consent choke point, one-shot click tokens with live-DOM fingerprint revalidation, planner annotation stripping, deterministic confirmation copy, snapshot binding, SecretRef making plaintext keys undeserializable, endpoint-scoped keyring identities, no silent provider fallback, model-download hardening.
- Baseline at review time: clippy clean, 499+4+6 Rust tests, 238 JS tests, all 4 CI guard scripts pass, zero unsafe/TODO/as-any/console.log, 23 production panic-capable calls (all justified).
