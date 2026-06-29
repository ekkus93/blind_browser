# Blind Browser Code Review 2 Spec

## Purpose

Code Review 2 is a targeted correctness, robustness, and architecture pass driven
by a fresh review of the current `master` snapshot. It is **not** a redesign or a
feature pass.

The review found one concrete functional bug (an audio capture buffer that is
never drained, which causes duplicate command execution in hands-free mode), one
high-impact architectural issue (synchronous Tauri commands doing multi-second
blocking work on the main thread while holding the global runtime lock), and a
small set of latent panics and minor cleanups.

Code Review 2 fixes those, in priority order, without regressing the existing
agent-safety guarantees or the passing test suite.

## What was validated during review

Confirmed in the review environment against this snapshot:

- `pnpm test:ui` passed: 164 / 164 tests.
- `pnpm lint` passed with no findings.
- `tsc --noEmit` passed with no type errors.

The Rust suite was read but not executed in the review environment (native
toolchain / system libraries for `cargo test --all-features` were not provisioned
there). The Rust validation gate below must still pass in the developer
environment before this pass is considered complete.

## Current known good behavior to preserve

Do not regress these behaviors:

- The confirmation workflow matches `confirmation_id` before resuming a pending
  plan, and aborts on mismatch or missing pending execution.
- Side-effecting tools are blocked before confirmation
  (`block_side_effects_until_confirmation` + `is_side_effecting_tool`).
- Planner step-cycle detection (`planner_step_cycle`) still aborts on repeated
  steps.
- The planner validator still rejects a `next_step_id` that does not reference a
  real step (`validate_step_transition`).
- Secret handling stays env / file / OS keyring only, with no secrets written to
  disk in plaintext and no secrets logged.
- Optional native integrations stay feature-gated (`browser`, `ocr`, `audio`,
  `local-tts`, `local-asr`, `remote-openai`); the crate still builds and tests
  with default features off.
- Push-to-talk capture still transcribes the full held utterance (the buffer
  accumulates from press to release).
- All currently passing UI and Rust tests stay green.

## Non-goals

Do not:

- Add voice-activity detection / silence detection (note it as a follow-up only).
- Redesign the planner, the tool contracts, or the confirmation UI.
- Enable provider failover (still intentionally disabled).
- Rewrite the browser/CDP integration or swap `chromiumoxide`.
- Migrate remaining hand-written CSS to Tailwind (tracked separately).
- Perform broad refactors unrelated to the tasks below.

## Design constraints

### 1. The audio capture buffer must be drained on each snapshot

`asr/capture.rs` accumulates microphone samples into a shared
`Arc<Mutex<Vec<f32>>>`. The cpal callback only ever `extend`s the buffer, and
`CaptureSession::snapshot()` **clones** the buffer without clearing it. The buffer
is only freed when the whole `CaptureSession` is dropped (stop / auto-stop).

In continuous (hands-free) listening — `ensureContinuousListeningLoop` calls
`transcribe_and_execute_command` with `auto_stop = false` — each iteration
snapshots the entire accumulated buffer. Iteration 2 re-transcribes everything
from iteration 1, iteration 3 re-transcribes 1 + 2, and so on. Consequences:

- The same utterance is re-transcribed and can be **re-executed**, producing
  repeated scrolls / clicks / navigations from a single spoken command. This is a
  correctness and safety problem for a tool driving a live browser.
- Memory grows without bound until the user stops listening.
- Per-iteration transcription latency grows roughly linearly.

Required behavior: capturing audio must consume the samples it returns, so each
continuous-listening window transcribes only newly captured audio.

Preferred implementation: drain under the lock instead of cloning. Replace the
clone in the snapshot path with a `std::mem::take` (or `std::mem::replace` with a
fresh `Vec` that preserves prior capacity) so accumulated samples are removed
atomically while the cpal callback continues appending to the now-empty buffer:

```rust
// in CaptureSession, under the buffer lock:
let samples = std::mem::take(&mut *guard); // returns accumulated samples, leaves empty
```

Audio that arrives between the drain and the next window is retained for the next
window (it is appended fresh), so no speech is lost across iterations.

Testability: extract the drain into a small free function over the shared buffer
(e.g. `fn drain_capture_buffer(buffer: &Mutex<Vec<f32>>) -> Result<Vec<f32>, _>`)
so it can be unit-tested without a real input device. The test must assert that
two consecutive drains do not return overlapping samples.

Push-to-talk must be unaffected: it drops the session on release, so drain and
clone are equivalent there; verify the full held utterance is still returned.

### 2. Long-running commands must not block the main thread or hold the runtime lock across blocking work

Every `#[tauri::command]` handler is a synchronous `fn`. Per the Tauri v2 docs,
commands without the `async` keyword run on the **main thread**, and blocking work
on the main thread freezes the webview UI. These handlers perform multi-second
blocking work — `thread::sleep` up to 10s for audio capture, `reqwest::blocking`
for model downloads and key tests, and `block_on` for the planner, remote ASR, and
all browser operations — while holding a single global `std::sync::Mutex<AppCore>`
for the entire duration.

Two consequences:

- The UI freezes during capture, network calls, and downloads.
- Because the lock is held the whole time, no other command can run during a
  long operation: `stop_listening`, `get_agent_state`, and confirmation
  submission all block on the same mutex. The user cannot barge-in / cancel an
  in-flight capture or network call — a real accessibility problem for a
  voice-first tool.

Required behavior, in two layers:

1. **Move blocking commands off the main thread.** Convert the long-running
   command handlers to async (`async fn` or `#[tauri::command(async)]`) and run
   the blocking section inside `tauri::async_runtime::spawn_blocking`, so the main
   thread and the webview stay responsive and the chromiumoxide handler keeps
   making progress.
2. **Do not hold the `AppCore` lock across the blocking operation.** Restructure
   the affected flows so the lock is acquired briefly to read/mutate state, then
   released before the blocking capture / network round-trip, then re-acquired to
   record results. Concretely, the audio `thread::sleep` capture window and the
   remote planner / remote ASR / model-download network calls must not run while
   the `AppCore` guard is held.

Acceptance for this constraint is behavioral: during an active capture or network
call, `stop_listening` and `get_agent_state` must still be dispatchable and must
return promptly.

This is the largest task in the pass and may be staged across more than one commit
(for example: audio-capture path first, then remote-network paths, then
browser-op paths). Each stage must keep the validation gate green.

Note: `std::sync::MutexGuard` is not `Send` and cannot be held across an `.await`.
Either keep the lock acquisition fully inside the `spawn_blocking` closure (scoped
so the guard is dropped before the blocking call), or move shared state to
`tokio::sync::Mutex`. Do not introduce a held-guard-across-await pattern.

### 3. Production code paths must not panic on recoverable conditions

Because the handlers run on the main thread, an `.expect()` / `unreachable!()` /
`panic!` in a production path takes down the whole app. Replace these with error
returns:

- `commands/planner_executor/execution.rs` — the
  `.expect("step positions should contain the current step")` is currently
  guarded by `validate_step_transition`, but a validation gap would panic the
  process. Return an `Aborted` `ExecutionOutcome` with a clear `code` instead.
- `app_core/model_management.rs` — the `unreachable!()` in the display-name match
  is only safe because two parallel `match` arms are kept in sync by hand. Make it
  return an error (or derive the display name without a second match) so adding a
  model in one place cannot panic.
- `tts/wav.rs` — the `try_into().expect(...)` calls parse WAV bytes that can come
  from a remote TTS provider. A malformed or truncated response would panic.
  Return a `TtsRuntimeError` (or equivalent) on short / malformed input.

Do not change the test-only `.expect("... should serialize/deserialize")` helpers;
those are fine.

### 4. Minor cleanups

- **Drop the redundant timestamp ID boilerplate.** `next_confirmation_id`,
  `next_page_id`, `next_image_id`, and `next_ocr_region_id` in `app_core/mod.rs`
  are four near-identical functions whose millisecond timestamp adds nothing —
  the frontend already issues UUID request IDs (`crypto.randomUUID()`). Collapse
  them into one helper, or drop the timestamp suffix. Keep the public ID shapes
  stable if anything asserts on them.
- **Zeroize cached secrets.** The session keyring cache in `config/keyring_store.rs`
  holds resolved secrets in plaintext in a process-global `static` for the whole
  session and never clears them. Add a `zeroize` pass (or clear the cache entry
  when a key is updated). This is a hardening step, not a behavior change.
- **Enforce formatting in CI.** `cargo fmt --check` is documented as a manual step
  but not enforced in `.github/workflows/ci.yml`. Add it to the gate.

## Expected files touched

Likely:

- `src-tauri/src/asr/capture.rs`
- `src-tauri/src/asr/mod.rs`
- `src-tauri/src/asr/processing.rs` (if a testable drain helper is extracted)
- `src-tauri/src/lib.rs` (command signatures)
- `src-tauri/src/command_handlers/*.rs`
- `src-tauri/src/app_core/listening_tools.rs`
- `src-tauri/src/app_core/remote_planner.rs`
- `src-tauri/src/app_core/model_management.rs`
- `src-tauri/src/app_core/api_key_tools.rs`
- `src-tauri/src/app_core/mod.rs` (ID helpers)
- `src-tauri/src/browser/*` (block_on / main-thread paths)
- `src-tauri/src/commands/planner_executor/execution.rs`
- `src-tauri/src/tts/wav.rs`
- `src-tauri/src/config/keyring_store.rs`
- `src-tauri/Cargo.toml` (if `zeroize` is added)
- `.github/workflows/ci.yml`
- `memory.md`

## Acceptance summary

Static checks:

```bash
# Buffer is drained, not cloned-without-clear, on the snapshot path.
rg -n "mem::take|mem::replace|drain_capture_buffer" src-tauri/src/asr
rg -n "\.clone\(\)" src-tauri/src/asr/capture.rs

# No production panics remain in the targeted sites.
rg -n "unreachable!\(\)|\.expect\(" src-tauri/src/commands/planner_executor/execution.rs
rg -n "unreachable!\(\)" src-tauri/src/app_core/model_management.rs
rg -n "\.expect\(" src-tauri/src/tts/wav.rs

# Long-running commands are async / run off the main thread.
rg -n "async fn|command\(async\)" src-tauri/src/command_handlers

# CI enforces formatting.
rg -n "cargo fmt" .github/workflows/ci.yml
```

Expected:

- The snapshot path drains the buffer; no continuous-listening window
  re-transcribes prior audio.
- The three targeted panic sites return errors instead of panicking.
- Capture / network / download commands no longer block the main thread or hold
  the `AppCore` lock across the blocking call; `stop_listening` and
  `get_agent_state` remain dispatchable during an active operation.
- All preserved safety invariants above still hold.

Full validation gate:

```bash
source ./fix-node-version.sh
pnpm lint
pnpm test:ui
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
```

Only after those pass:

- check the final TODO checklist,
- add the Code Review 2 `memory.md` entry with a real UTC timestamp.
