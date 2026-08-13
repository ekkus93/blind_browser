# Blind Browser Async Runtime TODO

## How to use this file

This implements the BLOCKED responsiveness work from `docs/BB_CODE_REVIEW2_TODO.md`
(P1.1.2 / P1.1.3 / P1.1.4): move blocking work off the main thread without the
`block_on` panic, then shrink the lock hold. Read `BB_ASYNC_RUNTIME_SPEC.md` first
— the four runtime facts in its Background section are load-bearing.

Work phase by phase. Each phase ships on its own with a green gate. Do not start a
later phase until the earlier one is gate-green.

Status key: `PENDING` · `IN PROGRESS` · `DONE` · `BLOCKED`

Priority key:

- `P0`: Phase 1 — removes the UI freeze and the `block_on` panic. Highest value.
- `P1`: Phases 2–3 — interruptible capture and interleaved state reads.
- `P2`: validation, status reconciliation, closeout.

Validation gate:

```bash
source ./fix-node-version.sh
pnpm lint
pnpm test:ui
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
```

Do not mark a task complete unless the gate passes in the developer environment.

---

## P0.1 — Phase 1: managed `Arc<Mutex<AppCore>>` + `spawn_blocking` commands

**Status:** DONE (gate green; behavioral `--features full` checks still need human verification)  
**Files:**

- `src-tauri/src/lib.rs`
- `src-tauri/src/command_handlers/*.rs`

### Problem

All command handlers either run on the main thread (sync — freezes the UI) or, if
made `#[tauri::command(async)]`, run on a worker thread where the browser layer's
`tauri::async_runtime::block_on` panics. The safe place for blocking work that
contains a `block_on` is a `spawn_blocking` thread (see spec Background fact 1).

### Required behavior

- Long-running commands do not block the main thread.
- Browser-reaching commands do not panic.
- `std::sync::Mutex` is retained; no guard crosses an `.await`.

### P0.1.1 — Make the managed state `Arc<Mutex<AppCore>>`

In `lib.rs`, change `app.manage(Mutex::new(app_core))` to
`app.manage(Arc::new(Mutex::new(app_core)))`. Update `lock_app_core` and every
handler signature from `tauri::State<'_, Mutex<AppCore>>` to
`tauri::State<'_, Arc<Mutex<AppCore>>>`. Add a `join_error_to_tool_error` helper
for `spawn_blocking` join failures (non-retryable `internal_task_failed`).

### P0.1.2 — Convert long commands to `async fn` + `spawn_blocking`

Apply the spec's pattern to: `transcribe_command`, `transcribe_and_execute_command`,
`resolve_command`, `execute_planner_output`, `submit_confirmation_response`,
`open_url`, `start_listening`, `stop_listening`, `get_agent_state`,
`download_active_local_tts_model`, `download_active_local_asr_model`,
`test_remote_planner_api_key`, `test_remote_tts_api_key`,
`test_remote_asr_api_key`, `list_remote_planner_models`.

Each becomes `async fn`, clones the `Arc`, runs the existing synchronous `AppCore`
method inside `tauri::async_runtime::spawn_blocking`, and awaits the join handle.
The `std::sync::MutexGuard` must be created and dropped inside the closure.

The short config setters (`set_playback_volume`, `set_playback_speed`,
`set_tts_voice`, `set_browser_visibility`, `set_confirmation_threshold`,
`set_allow_click_without_confirmation`, `set_ocr_thresholds`, the provider/model
selection setters, the connection-settings setters) may stay plain sync
`#[tauri::command]`.

### P0.1.3 — Remove the Code Review 2 guardrail comments

Delete the `GUARDRAIL: Keep this a plain #[tauri::command]...` comments on
`transcribe_and_execute_command`, `execute_planner_output`, `resolve_command`,
`submit_confirmation_response`, and `open_url`. Replace with a one-line note that
the blocking section runs in `spawn_blocking` so the inner
`tauri::async_runtime::block_on` is safe.

### Acceptance checks

```bash
rg -n "Arc<Mutex<AppCore>>|manage\(Arc::new" src-tauri/src/lib.rs
rg -n "spawn_blocking" src-tauri/src/command_handlers
rg -n "command\(async\)" src-tauri/src/command_handlers
rg -n "tokio::sync::Mutex" src-tauri/src || echo "good: no tokio mutex"
rg -n "GUARDRAIL" src-tauri/src/command_handlers || echo "guardrails removed"
```

Behavioral (under `--features full`, real page):

- Webview stays responsive during a 10s capture, a slow planner call, and a
  navigation.
- A voice command that opens a URL / clicks / reads completes without a
  worker-thread panic.

---

## P1.1 — Phase 2: scope the lock around the capture window

**Status:** DONE (gate green; behavioral `--features full` interrupt check still needs human verification)  
**Files:**

- `src-tauri/src/app_core/listening_tools.rs`
- `src-tauri/src/asr/mod.rs`
- `src-tauri/src/asr/capture.rs`

### Problem

The `AppCore` lock is held across the capture `thread::sleep`, so `stop_listening`
cannot take the `CaptureSession` and end an active capture until the sleep
finishes.

### Required behavior

- The blocking capture sleep runs without the `AppCore` guard held.
- `stop_listening` can end an active capture mid-window.
- The Code Review 2 buffer-drain semantics (no re-transcription) are preserved.

### P1.1.1 — Split transcribe into locked / unlocked / locked phases

Restructure so a brief locked phase starts or locates the `CaptureSession` and
takes an owned handle to its shared buffer (the cpal callback keeps filling the
buffer regardless of the `AppCore` lock); drop the guard; run the blocking sleep
unlocked; re-acquire to drain, run ASR, and record transcript + listening state.

### P1.1.2 — Handle stop-during-capture cleanly

If `stop_listening` drops the `CaptureSession` during the unlocked sleep, the
in-flight transcribe must return a clean "capture stopped" outcome (or transcribe
what was captured up to the stop) rather than erroring opaquely or transcribing a
stale buffer. Pick one behavior and add a regression test for it.

### Acceptance checks

```bash
rg -n "thread::sleep" src-tauri/src/asr/mod.rs
```

Behavioral: `stop_listening` ends an active capture instead of waiting for the
full window; `get_agent_state` returns during a capture.

---

## P1.2 — Phase 3: scope the lock around remote network calls

**Status:** DONE. Remote planner lock-scoping landed in
`BB_RUNTIME_PHASE3_TODO.md` P2.1 and remote ASR lock-scoping subsequently landed
in P2.2. The later CR3 P1.2 pass also closed the separate planner-embedded speech
lock window.

> The structural change anticipated here was carried out in the dedicated
> `BB_RUNTIME_PHASE3` pass: `execute_command_with_replanning` was split into
> `build_planner_resolution` (deterministic resolution + profile snapshot, under a
> brief lock) and a free `resolve_remote_planner` (unlocked LLM round-trip), driven
> by a handler-level `LockScopedReplanningRuntime` through the existing
> `execute_bounded_replanning_loop`. The atomicity tradeoff is documented at the
> call site. Remote ASR was initially skipped as the lowest-value remote-only
> item, then implemented in `BB_RUNTIME_PHASE3_TODO.md` P2.2 with the same
> drain/transcribe-unlocked/record discipline.
**Files:**

- `src-tauri/src/app_core/remote_planner.rs`
- `src-tauri/src/asr/remote.rs`

### Problem

The `AppCore` lock is held across remote planner / ASR network round-trips, so
state reads block on the network.

### P1.2.1 — Resolve-drop-call-reacquire

In the remote planner and remote ASR paths: resolve inputs (profile, prompt,
secret) under the lock, drop the guard, run the
`futures::executor::block_on(...)` network call unlocked, then re-acquire to apply
results. Re-resolving the secret outside the lock is fine.

### Acceptance checks

Behavioral: `get_agent_state` returns promptly while a remote planner call is in
flight.

---

## P2.1 — Reconcile Code Review 2 status

**Status:** DONE (P1.1.2, P1.1.3, and P1.1.4 reconciled; Phase 3 landed)
**Files:**

- `docs/BB_CODE_REVIEW2_TODO.md`

After the relevant phases land, update `BB_CODE_REVIEW2_TODO.md`:

- Mark P1.1.2 resolved by Phase 2, P1.1.3 resolved by Phase 3, P1.1.4 satisfied by
  Phase 1's `spawn_blocking` bridge (CDP handler keeps progressing on the
  multi-thread runtime).
- Re-check the "Long-running commands run off the main thread; the webview does
  not freeze" item once Phase 1 is verified behaviorally, with a note that it is
  delivered by the `spawn_blocking` form, not by `#[tauri::command(async)]`.

---

## P2.2 — Run the full validation gate

**Status:** DONE for Phases 1–3 (gates green; live provider checks remain useful behavioral acceptance coverage)
**Files:**

- no source file unless failures require fixes

Run the gate (top of file) after each phase. Do not mark done unless every command
passes. Remember the gate cannot prove the behavioral acceptance — run the
`--features full` checks on a real page in addition.

---

## P2.3 — Add memory entries with real UTC timestamps

**Status:** DONE (Phase 3 and later CR3 P1.2 outcomes are also recorded in `memory.md`)
**Files:**

- `memory.md`

Add one entry per phase (or one combined entry if shipped together) after that
phase's gate + behavioral checks pass. Use:

```bash
date -u +"%Y-%m-%dT%H:%M:%SZ"
```

Note honestly which behavioral checks were run on a real page and which still need
human verification. Do not fabricate timestamps.

---

## Suggested commit sequence

1. `refactor(runtime): manage Arc<Mutex<AppCore>>; run long commands in spawn_blocking`
2. `refactor(asr): scope the AppCore lock around the capture window`
3. `refactor(net): scope the AppCore lock around remote planner/ASR calls`
4. `docs: resolve Code Review 2 P1.1.2/P1.1.3/P1.1.4 and record validation`

---

## Final done checklist

- [x] Managed state is `Arc<Mutex<AppCore>>`; `std::sync::Mutex` retained.
- [x] Long commands are `async fn` running their work in `spawn_blocking`.
- [x] No `#[tauri::command(async)]` remains.
- [x] Code Review 2 guardrail comments removed / replaced.
- [ ] Webview stays responsive during capture / planner / navigation (verified
      under `--features full`). — code landed (Phase 1); live verification pending.
- [ ] Voice → browser commands complete without a worker-thread panic. — code
      landed (spawn_blocking bridge); live `--features full` verification pending.
- [x] `stop_listening` ends an active capture (Phase 2). — code + regression test
      landed; live verification pending.
- [x] `get_agent_state` can acquire the runtime during capture/planner network windows.
      — capture lock-scoping landed in Phase 2; remote planner/ASR Phase 3 and the later
      planner-embedded speech P1.2 pass release the relevant blocking windows. Live
      provider verification remains useful acceptance coverage, not an implementation blocker.
- [x] Buffer-drain semantics (no re-transcription) still hold.
- [x] All preserved Code Review 2 + follow-up fixes and safety invariants hold.
- [x] `BB_CODE_REVIEW2_TODO.md` P1.1.2 / P1.1.4 reconciled (P1.1.3 still BLOCKED).
- [x] Full validation gate passes for Phases 1–2.
- [x] `memory.md` has real UTC entries noting what was behaviorally verified.
- [x] Phase 3 remote planner/ASR lock-scoping landed in `BB_RUNTIME_PHASE3`; the
      remaining planner-embedded speech lock scope was subsequently closed by CR3 P1.2.
