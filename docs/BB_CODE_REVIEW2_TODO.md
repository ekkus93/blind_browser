# Blind Browser Code Review 2 TODO

## How to use this file

This is a targeted correctness / robustness / architecture pass driven by Code
Review 2. Work top-to-bottom. Keep diffs scoped to the task at hand and keep the
validation gate green between tasks. Do not redo working prior-pass changes and do
not start the non-goals listed in the spec.

Status key: `PENDING` · `IN PROGRESS` · `DONE` · `BLOCKED`

Priority key:

- `P0`: correctness / safety bug reachable in normal use. Fix first.
- `P1`: architecture / robustness change that prevents UI freezes, hangs, or
  process crashes.
- `P2`: minor cleanup, hardening, and validation / documentation closeout.

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

Do not mark this TODO complete unless the validation gate actually passes in the
developer environment.

---

## P0.1 — Drain the audio capture buffer on each snapshot

**Status:** DONE  
**Files:**

- `src-tauri/src/asr/capture.rs`
- `src-tauri/src/asr/mod.rs`
- `src-tauri/src/asr/processing.rs` (if extracting a testable helper)

### Problem

`CaptureSession::snapshot()` clones the shared sample buffer without clearing it,
and the cpal callback only ever appends. In continuous listening
(`auto_stop = false`) each `transcribe_command` snapshots the entire accumulated
buffer, so each window re-transcribes all prior audio. This causes duplicate
command execution (repeated clicks / scrolls / navigations from one utterance),
unbounded memory growth, and linearly growing latency.

### Required behavior

- Capturing audio consumes the samples it returns.
- Each continuous-listening window transcribes only audio captured since the
  previous window.
- Audio arriving during transcription is retained for the next window, not lost.
- Push-to-talk still returns the full held utterance.

### P0.1.1 — Drain instead of clone

In the snapshot path, under the buffer lock, replace the clone with a drain:

```rust
let samples = std::mem::take(&mut *guard);
// or, to preserve capacity:
// let samples = std::mem::replace(&mut *guard, Vec::with_capacity(prior_len));
```

Keep the existing `lock_failed` poison check and the existing
`sample_rate` / `channels` propagation. Consider renaming the method to reflect
its now-destructive semantics (e.g. `take_captured_audio`) if it reads more
clearly, but a rename is optional.

### P0.1.2 — Make the drain unit-testable

Extract the buffer drain into a small free function over the shared buffer so it
can be tested without a real input device, e.g.:

```rust
fn drain_capture_buffer(buffer: &std::sync::Mutex<Vec<f32>>) -> Result<Vec<f32>, AsrRuntimeError>
```

Add a unit test asserting that two consecutive drains do not return overlapping
samples (append A, drain → A and buffer empty; append B, drain → B only).

### P0.1.3 — Verify push-to-talk is unaffected

Confirm (by reading the call path and, if practical, a test) that the
press-to-release accumulation still returns the full utterance, since PTT drops
the session on release.

### Acceptance checks

```bash
rg -n "mem::take|mem::replace|drain_capture_buffer" src-tauri/src/asr
rg -n "\.clone\(\)" src-tauri/src/asr/capture.rs
```

Expected: the snapshot/drain path no longer clones-without-clearing; a drain
helper exists and is unit-tested; continuous-listening windows do not re-transcribe
prior audio.

---

## P1.1 — Run long commands off the main thread and release the lock across blocking work

**Status:** PARTIAL (P1.1.1 PARTIAL; P1.1.2 and P1.1.3 BLOCKED)

> Correction (follow-up pass, see `BB_CODE_REVIEW2_FOLLOWUP_TODO.md`): P1.1.1 is
> **PARTIAL**, not DONE. `start_listening`, `stop_listening`, `resolve_command`,
> `execute_planner_output`, `open_url`, and `submit_confirmation_response` were
> intentionally left as plain `#[tauri::command]` and must stay that way until the
> browser ops stop calling `tauri::async_runtime::block_on` from a tokio worker
> (P1.1.2 / P1.1.4). `transcribe_and_execute_command` was briefly converted to
> `(async)` and reverted because it reaches browser ops and panicked
> ("runtime within a runtime"). Converting a command to `(async)` does not by
> itself keep the UI responsive: the `AppCore` lock is still held for the whole
> blocking duration, so a peer command still contends. The real fix is the
> lock-release work (P1.1.2 / P1.1.3), still BLOCKED.

**Files:**

- `src-tauri/src/lib.rs`
- `src-tauri/src/command_handlers/*.rs`
- `src-tauri/src/app_core/listening_tools.rs`
- `src-tauri/src/app_core/remote_planner.rs`
- `src-tauri/src/app_core/model_management.rs`
- `src-tauri/src/app_core/api_key_tools.rs`
- `src-tauri/src/asr/mod.rs`
- `src-tauri/src/browser/*`

### Problem

All Tauri command handlers are synchronous `fn`, so they run on the main thread and
freeze the webview during blocking work. They also hold the global
`std::sync::Mutex<AppCore>` across multi-second operations (audio `thread::sleep`,
blocking `reqwest`, `block_on` for planner / remote ASR / browser), so no other
command — including `stop_listening`, `get_agent_state`, and confirmation
submission — can run during a long operation. The user cannot cancel or query
state mid-operation.

### Required behavior

- Long-running commands do not block the main thread (UI stays responsive).
- The `AppCore` lock is not held across the blocking capture / network /
  download call.
- During an active capture or network call, `stop_listening` and
  `get_agent_state` are still dispatchable and return promptly.
- No `MutexGuard` is held across an `.await`.

### P1.1.1 — Move blocking commands off the main thread

Convert the long-running command handlers to async (`async fn` or
`#[tauri::command(async)]`) and run the blocking section inside
`tauri::async_runtime::spawn_blocking`. Scope the lock acquisition inside the
blocking closure so the guard is dropped before returning. Affected commands at
minimum:

- `transcribe_command`, `transcribe_and_execute_command`, `start_listening`,
  `stop_listening`
- `resolve_command` / `execute_planner_output` paths that reach the remote planner
- `list_remote_planner_models`, `test_remote_*_api_key`
- `download_active_local_tts_model`, `download_active_local_asr_model`
- `open_url` and other browser-backed commands

Short, non-blocking state-only commands (e.g. `get_agent_state`, volume/speed
setters) can stay as they are, but verify they do not contend on a held lock.

### P1.1.2 — Release the lock across the audio capture window

**Status:** BLOCKED (requires CaptureHandle extraction and multi-phase transcribe restructuring; see spec section 2)

Restructure the capture path so the `AppCore` guard is not held during the
`thread::sleep` capture window. Acquire briefly to start/inspect the session,
release, run the blocking capture, then re-acquire to record the transcript and
listening state. This is what lets `stop_listening` take the session and interrupt
an in-flight capture.

### P1.1.3 — Release the lock across remote network calls

**Status:** BLOCKED (requires CaptureHandle extraction and multi-phase transcribe restructuring; see spec section 2)

Apply the same scoping to the remote planner, remote ASR, and model-download
paths: resolve the inputs under the lock, drop the guard, perform the network call
on a blocking thread, then re-acquire to apply results.

### P1.1.4 — Confirm the CDP handler keeps progressing

After the browser-op commands stop blocking the main thread, confirm navigation /
extraction still work and the spawned `chromiumoxide` handler future is not
starved.

### Acceptance checks

```bash
rg -n "async fn|command\(async\)" src-tauri/src/command_handlers
rg -n "thread::sleep" src-tauri/src/asr/mod.rs
```

Manual / behavioral acceptance:

- The webview does not freeze during a 10s capture or a slow planner call.
- `stop_listening` interrupts an active capture instead of blocking until it
  finishes.
- `get_agent_state` returns promptly during an active operation.

Note: this task may be split across multiple commits (capture path, then network
paths, then browser paths). Keep the validation gate green at each step. Mark
`BLOCKED` with a note if a Tauri / Send constraint forces a `tokio::sync::Mutex`
migration that needs its own review.

---

## P1.2 — Replace main-thread panics with error returns

**Status:** DONE  
**Files:**

- `src-tauri/src/commands/planner_executor/execution.rs`
- `src-tauri/src/app_core/model_management.rs`
- `src-tauri/src/tts/wav.rs`

### Problem

`.expect()` / `unreachable!()` in production paths crash the whole process because
the handlers run on the main thread. Three reachable-or-fragile sites should
return errors.

### P1.2.1 — Planner execution step lookup

In `execution.rs`, replace:

```rust
let step = &steps[*step_positions
    .get(&current_step_id)
    .expect("step positions should contain the current step")];
```

with a guarded lookup that returns an `Aborted` `ExecutionOutcome` (with a clear
`code`, e.g. `missing_step_position`) when the id is absent, instead of panicking.
The validator should already prevent this, so the error path is defensive.

### P1.2.2 — Model-management display name

In `model_management.rs`, remove the `unreachable!()` in the `display_name` match.
Either return an error for an unmapped id, or derive the display name from the
already-validated `normalized` value without a second hand-synced `match`.

### P1.2.3 — WAV header parsing

In `tts/wav.rs`, replace the `try_into().expect(...)` calls with checked parsing
that returns a `TtsRuntimeError` (or the local error type) on short / malformed
input, since the bytes can come from a remote TTS provider.

### Acceptance checks

```bash
rg -n "unreachable!\(\)|\.expect\(" src-tauri/src/commands/planner_executor/execution.rs
rg -n "unreachable!\(\)" src-tauri/src/app_core/model_management.rs
rg -n "\.expect\(" src-tauri/src/tts/wav.rs
```

Expected: no production panic remains at these sites; add a small unit test for
the WAV short-input error path if the helper is testable.

---

## P2.1 — Minor cleanups and hardening

**Status:** DONE  
**Files:**

- `src-tauri/src/app_core/mod.rs`
- `src-tauri/src/config/keyring_store.rs`
- `src-tauri/Cargo.toml` (if `zeroize` is added)
- `.github/workflows/ci.yml`

### P2.1.1 — Collapse the ID helpers

`next_confirmation_id`, `next_page_id`, `next_image_id`, and `next_ocr_region_id`
are near-identical and their millisecond timestamp is redundant (request IDs are
already UUIDs from the frontend). Collapse into one helper or drop the timestamp
suffix. Keep ID prefixes stable if anything asserts on them.

### P2.1.2 — Zeroize cached secrets

The session keyring cache holds resolved secrets in plaintext in a process-global
`static` and never clears them. Add a `zeroize` pass (or clear the cached entry on
key update). No behavior change beyond memory hygiene.

### P2.1.3 — Enforce formatting in CI

Add a `cargo fmt --manifest-path src-tauri/Cargo.toml --check` step to
`.github/workflows/ci.yml` so formatting is gated, not just documented.

### Acceptance checks

```bash
rg -n "next_confirmation_id|next_page_id|next_image_id|next_ocr_region_id" src-tauri/src/app_core/mod.rs
rg -n "zeroize" src-tauri/src/config/keyring_store.rs
rg -n "cargo fmt" .github/workflows/ci.yml
```

Expected: one shared ID helper (or no redundant timestamp); cached secrets are
zeroized / cleared; CI runs `cargo fmt --check`.

---

## P2.2 — Run the full validation gate

**Status:** DONE  
**Files:**

- no source file unless failures require fixes

Run:

```bash
source ./fix-node-version.sh
pnpm lint
pnpm test:ui
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
```

Do not mark this task done unless every command completes successfully. If a
command fails: fix it, re-run the full gate, and only then update the checklist and
memory entry.

---

## P2.3 — Add Code Review 2 memory entry with real UTC timestamp

**Status:** DONE  
**Files:**

- `memory.md`

Only after P2.2 passes. Run:

```bash
date -u +"%Y-%m-%dT%H:%M:%SZ"
```

Add an entry like:

```md
- 2026-XX-XXTXX:XX:XXZ — Completed Code Review 2: drained the ASR capture buffer per snapshot (fixes duplicate command execution in continuous listening), moved long-running commands off the main thread and stopped holding the AppCore lock across blocking capture/network work, replaced three main-thread panics with error returns, collapsed the ID helpers, zeroized cached secrets, and added cargo fmt to CI. Full validation gate passed.
```

Replace the timestamp with the actual command output. Do not fabricate or reuse an
old timestamp.

---

## Suggested commit sequence

1. `fix(asr): drain capture buffer per snapshot to stop re-transcription`
2. `refactor(runtime): run blocking commands off-thread without holding the lock`
3. `fix(core): return errors instead of panicking in execution/model/wav paths`
4. `chore(core): collapse id helpers and zeroize cached secrets`
5. `ci: enforce cargo fmt --check`
6. `docs: record Code Review 2 validation`

---

## Final done checklist

- [x] ASR capture buffer is drained per snapshot; a drain helper is unit-tested.
- [x] Continuous listening no longer re-transcribes prior audio.
- [x] Push-to-talk still returns the full held utterance.
- [ ] Long-running commands run off the main thread; the webview does not freeze.
      (NOT delivered: converting a command to `(async)` alone does not achieve this
      because the `AppCore` lock is still held for the full blocking duration; the
      real fix is the lock-release work P1.1.2 / P1.1.3, still BLOCKED.)
- [ ] The AppCore lock is not held across blocking capture / network calls. (P1.1.2/P1.1.3 BLOCKED — requires CaptureHandle extraction)
- [ ] `stop_listening` can interrupt an active capture; `get_agent_state` returns
      promptly during an active operation. (BLOCKED — depends on P1.1.2)
- [x] No `MutexGuard` is held across an `.await`.
- [x] Planner execution returns an Aborted outcome instead of panicking on a
      missing step position.
- [x] `model_management` display name no longer uses `unreachable!()`.
- [x] `tts/wav.rs` returns an error on short / malformed input.
- [x] ID helpers are collapsed / de-duplicated.
- [x] Cached secrets are zeroized or cleared on update.
- [x] CI runs `cargo fmt --check`.
- [x] Preserved safety invariants (confirmation gating, side-effect block,
      step-cycle detection, planner validation) still hold.
- [x] Full validation gate passes.
- [x] `memory.md` has a real UTC Code Review 2 completion entry.
