# Blind Browser Async Runtime Spec

## Purpose

This is the real fix for the responsiveness limitation tracked as BLOCKED in
`docs/BB_CODE_REVIEW2_TODO.md` (P1.1.2 / P1.1.3 / P1.1.4): long-running commands
run on the main thread and hold the global `AppCore` lock across blocking work, so
the webview freezes during capture / network / browser operations, and `stop` and
state queries cannot interleave. The Code Review 2 follow-up made the current state
*safe* (by reverting `transcribe_and_execute_command` to a sync command to avoid a
`block_on` panic) but left the freeze in place.

The goal here is to move blocking work off the main thread **without** triggering
the "Cannot start a runtime from within a runtime" panic, then progressively
shrink the lock hold so `stop_listening` and `get_agent_state` stay responsive.

This is done in three phases, each independently shippable with a green gate.

## Background: the runtime facts that constrain the design

These four facts drive every decision below. Implementers must keep them in mind.

1. **`tauri::async_runtime::block_on` panics on a worker thread, is safe on the
   main thread, and is safe inside `spawn_blocking`.** It is `Handle::block_on` on
   the global multi-threaded runtime. Calling it from a thread that is driving
   async tasks (a worker — which is where `#[tauri::command(async)]` runs a
   command) panics. Calling it from the main thread (a plain `#[tauri::command]`)
   is fine. Calling it from a `tokio::task::spawn_blocking` closure is fine,
   because blocking-pool threads are not driving the async scheduler. This is the
   documented "bridge sync and async" pattern.

2. **The browser layer uses `tauri::async_runtime::block_on` and is not being
   rewritten.** `browser/navigation.rs`, `element_interaction.rs`,
   `page_inspection.rs`, `dom_extraction.rs`, and `page_metrics.rs` call
   `tauri::async_runtime::block_on` on chromiumoxide futures. The CDP event
   handler is a task spawned on the runtime, so as long as the runtime's worker
   threads are free, those futures make progress while another thread blocks on
   them. Therefore browser-reaching commands must run their blocking section in
   `spawn_blocking` (fact 1), never as `#[tauri::command(async)]`.

3. **Tauri's default runtime is multi-threaded.** Blocking one `spawn_blocking`
   thread on a browser `block_on` does not starve the CDP handler, which runs on
   the worker threads. Do not switch the app to a current-thread runtime; that
   would deadlock this pattern.

4. **`spawn_blocking` needs an owned, `'static`, `Send` handle to the state.**
   `tauri::State<'_, _>` is borrowed and cannot move into a `'static` closure. The
   managed state must become `Arc<Mutex<AppCore>>` so a clone can move in. The
   `std::sync::MutexGuard` is created and dropped *inside* the closure and never
   crosses an `.await`, so `std::sync::Mutex` stays — do **not** migrate to
   `tokio::sync::Mutex`.

## The pattern

Every long-running command becomes an `async fn` that clones the `Arc`, runs the
existing synchronous `AppCore` method inside `spawn_blocking`, and awaits the
result:

```rust
#[tauri::command]
pub async fn transcribe_and_execute_command(
    request_id: String,
    timeout_ms: Option<u64>,
    max_duration_ms: Option<u64>,
    auto_stop: bool,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<TranscribeAndExecuteCommandData, ToolError> {
    let core = Arc::clone(&app_core);
    tauri::async_runtime::spawn_blocking(move || {
        let mut guard = lock_app_core(&core)?;
        guard.transcribe_and_execute_command(request_id, timeout_ms, max_duration_ms, auto_stop)
    })
    .await
    .map_err(join_error_to_tool_error)?
}
```

Note: the command is `async fn` (so Tauri runs it as a task and does not block the
main thread), but the blocking work — including the inner browser `block_on` —
runs on the blocking pool, where `block_on` is safe. The guard lives only inside
the closure. This is what makes browser-reaching commands safe to take off the
main thread; it is the missing piece that the Code Review 2 guardrail comments
were protecting against.

## Current known good behavior to preserve

- All Code Review 2 + follow-up fixes stay intact: ASR buffer drain, the three
  panic-to-error conversions, the collapsed ID helper, CI `cargo fmt --check`, and
  the `zeroize` secret cache.
- All agent-safety invariants: confirmation id matching, side-effect-before-
  confirmation block, step-cycle detection, planner output validation.
- The existing synchronous `AppCore` execution logic is **unchanged** — this work
  relocates *where* it runs and *how long* the lock is held, not what it computes.
  Existing Rust tests must stay green without behavioral edits.
- Command ordering: the frontend already sequences dependent calls with `await`
  (e.g. `startListening` then `transcribe`, the continuous loop, push-to-talk), so
  correctness does not depend on backend main-thread serialization. Preserve that
  assumption; do not add new cross-command ordering requirements.

## Non-goals

- Do not rewrite the browser layer to be natively async / drop
  `tauri::async_runtime::block_on`. The `spawn_blocking` bridge is the chosen
  approach.
- Do not migrate to `tokio::sync::Mutex`.
- Do not attempt to make planner execution itself concurrent. The global lock
  still serializes execution; the goal is responsiveness, not parallelism.
- Do not add voice-activity detection.
- Do not scope the lock around every individual browser op inside a plan
  (out of scope; the plan executes as one locked transaction off the main thread).

## Design constraints (phased plan)

### Phase 1 — Managed `Arc<Mutex<AppCore>>` + `spawn_blocking` for long commands

This phase delivers the headline win: no main-thread freeze and no `block_on`
panic, which also lets `transcribe_and_execute_command` be async again safely.

- Change the managed state from `Mutex<AppCore>` to `Arc<Mutex<AppCore>>`
  (`app.manage(Arc::new(Mutex::new(app_core)))`), and update `lock_app_core` and
  every handler signature to `tauri::State<'_, Arc<Mutex<AppCore>>>`.
- Convert these commands to `async fn` + `spawn_blocking` per the pattern above:
  `transcribe_command`, `transcribe_and_execute_command`, `resolve_command`,
  `execute_planner_output`, `submit_confirmation_response`, `open_url`,
  `start_listening`, `stop_listening`, the `download_active_local_*_model`
  commands, the `test_remote_*_api_key` commands, and `list_remote_planner_models`.
  (The `test_*`, `list_*`, and `download_*` commands were already
  `#[tauri::command(async)]`; switch them to the `async fn` + `spawn_blocking`
  form so they too run on the blocking pool and never on a worker.)
- Convert `get_agent_state` to the same form, so a state query during an in-flight
  long command waits on the blocking pool rather than blocking the main thread.
- The short config setters (`set_playback_volume`, `set_tts_voice`,
  `set_confirmation_threshold`, etc.) may stay plain sync `#[tauri::command]`;
  their lock hold is microscopic. Converting them is optional and harmless.
- **Remove the Code Review 2 follow-up guardrail comments** on the browser-
  reaching commands. They were correct for the sync-vs-`(async)` distinction, but
  the `spawn_blocking` form makes async safe, so the comments now describe a
  constraint that no longer applies. Replace them with a one-line note that
  blocking work runs in `spawn_blocking` precisely so the inner `block_on` is safe.
- Add a `join_error_to_tool_error` helper for `spawn_blocking` `JoinError`
  (treat as a non-retryable internal error).

After Phase 1: the webview no longer freezes during capture / planner / browser
work, and voice→browser commands no longer risk the runtime panic. `stop_listening`
still cannot *interrupt* an in-flight capture (Phase 2), but it no longer freezes
the UI while it waits.

### Phase 2 — Scope the lock around the capture window

So `stop_listening` can interrupt an active capture, the `AppCore` lock must not be
held during the audio capture sleep.

- Extract the capture so the long `thread::sleep` does not run under the `AppCore`
  guard. Shape: a brief locked phase starts/locates the `CaptureSession` and takes
  an owned handle to its shared buffer (the cpal stream keeps filling the buffer
  regardless of the `AppCore` lock); the guard is dropped; the blocking sleep runs
  unlocked; then a second brief locked phase drains the buffer, runs ASR, and
  records the transcript + listening state.
- `stop_listening` (already `spawn_blocking` from Phase 1) can then acquire the
  lock during the unlocked sleep window and drop the `CaptureSession`, ending the
  capture. The in-flight transcribe must detect that the session was stopped and
  return a clean "capture stopped" result rather than transcribing a partial
  buffer, or transcribe what was captured up to the stop — pick one and cover it
  with a test.
- Keep the buffer-drain semantics from Code Review 2 P0.1 intact (no
  re-transcription of prior audio).

### Phase 3 — Scope the lock around remote network calls

So `get_agent_state` and other reads return promptly while a remote planner / ASR
call is in flight, the lock must not be held across the network round-trip.

- In the remote planner path (`remote_planner.rs`) and remote ASR path: resolve
  inputs (profile, prompt payload, secret) under the lock, drop the guard, perform
  the `futures::executor::block_on(...)` network call unlocked, then re-acquire to
  apply results to state. The model-download path may be similarly scoped, though
  it already runs off the main thread after Phase 1.
- Re-resolving the secret outside the lock is fine; it does not need `AppCore`.

After Phase 3: a state read during an in-flight planner call no longer blocks on
the network round-trip.

## Expected files touched

- `src-tauri/src/lib.rs` (managed state type, `lock_app_core`, optional helper)
- `src-tauri/src/command_handlers/*.rs` (async + spawn_blocking conversions,
  remove guardrail comments)
- `src-tauri/src/app_core/listening_tools.rs`, `asr/mod.rs`, `asr/capture.rs`
  (Phase 2 capture lock-scoping)
- `src-tauri/src/app_core/remote_planner.rs`, `asr/remote.rs` (Phase 3 network
  lock-scoping)
- `docs/BB_CODE_REVIEW2_TODO.md` (mark P1.1.2 / P1.1.3 / P1.1.4 resolved, re-check
  the "no freeze" item)
- `memory.md`

## Acceptance summary

Static checks:

```bash
# Managed state is Arc<Mutex<AppCore>>.
rg -n "Arc<Mutex<AppCore>>|manage\(Arc::new" src-tauri/src/lib.rs

# Long commands run their work in spawn_blocking and are async fn.
rg -n "spawn_blocking" src-tauri/src/command_handlers

# No browser-reaching command is #[tauri::command(async)] (worker form).
rg -n "command\(async\)" src-tauri/src/command_handlers

# Still std::sync::Mutex, not tokio::sync::Mutex.
rg -n "tokio::sync::Mutex" src-tauri/src || echo "good: no tokio mutex"
```

Expected: managed state is `Arc<Mutex<AppCore>>`; long commands use
`spawn_blocking`; no `#[tauri::command(async)]` remains; `std::sync::Mutex` is
retained.

Behavioral acceptance (requires `--features full` and a real page, since
`cargo test` does not drive Chromium or audio hardware):

- The webview stays responsive (animations/scroll) during a 10s capture, a slow
  planner call, and a page navigation.
- A voice command that opens a URL / clicks / reads the page completes without a
  worker-thread panic.
- `get_agent_state` returns promptly during an in-flight capture and during an
  in-flight planner call (Phase 3).
- `stop_listening` ends an active capture instead of waiting for it to finish
  (Phase 2).

Full validation gate (per phase):

```bash
source ./fix-node-version.sh
pnpm lint
pnpm test:ui
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
```

Only after a phase passes the gate and its behavioral checks: update
`BB_CODE_REVIEW2_TODO.md` status and add a `memory.md` entry with a real UTC
timestamp.
