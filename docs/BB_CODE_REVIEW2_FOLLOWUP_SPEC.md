# Blind Browser Code Review 2 Follow-up Spec

## Purpose

This follow-up corrects issues found while verifying the Code Review 2
implementation. It is small and surgical. It does **not** restart the larger
async / lock-release refactor (still tracked as BLOCKED in
`BB_CODE_REVIEW2_TODO.md`).

Three findings drive this pass:

1. **A likely runtime panic was introduced.** `transcribe_and_execute_command`
   was changed to `#[tauri::command(async)]`, so it now runs on a tokio worker
   thread. It reaches browser tools that call `tauri::async_runtime::block_on`,
   which panics with *"Cannot start a runtime from within a runtime"* when called
   from a runtime worker. This is reachable on the core voice → browser path
   (open URL, click, scroll, read) via push-to-talk and the continuous-listening
   loop, and it is not caught by `cargo test` because the unit tests do not drive
   a real Chromium session.
2. **The secret-zeroize change is largely a no-op.** P2.1.2 was implemented as a
   hand-rolled `unsafe { old.as_mut_vec().fill(0) }` that only runs on same-key
   overwrite and is subject to dead-store elimination. It should use the `zeroize`
   crate.
3. **The P1.1.1 status and the "no freeze" checklist item are overstated.** The
   lock is still held across the blocking work, and several long-running commands
   were not moved off the main thread, so the UI still freezes. The status needs
   to reflect reality, and a guardrail is needed so the remaining
   browser-touching commands are **not** "finished" into `(async)` form, which
   would spread the same panic.

## Background: why `(async)` + `block_on` panics

`#[tauri::command(async)]` (even on a sync-bodied function) runs the command via
`async_runtime::spawn` on a tokio worker thread. `tauri::async_runtime::block_on`
calls the multi-threaded runtime's `block_on`, which panics when invoked from a
thread that is already driving the runtime. The browser layer
(`browser/navigation.rs`, `element_interaction.rs`, `page_inspection.rs`,
`dom_extraction.rs`, `page_metrics.rs`) calls `tauri::async_runtime::block_on`
directly. Therefore:

- A command that is `#[tauri::command(async)]` **must not** transitively reach a
  browser op until those ops stop calling `tauri::async_runtime::block_on` from a
  worker (the real fix, tracked as P1.1.2 / P1.1.4 and still BLOCKED).
- The remote-planner path uses `futures::executor::block_on`, which does **not**
  panic in this situation (it blocks the worker instead), so planner-only `(async)`
  commands are safe.

## Current known good behavior to preserve

- All Code Review 2 fixes that verified correct stay intact: the ASR buffer drain
  (P0.1), the three panic-to-error conversions (P1.2), the collapsed ID helper
  (P2.1.1), and the CI `cargo fmt --check` step (P2.1.3).
- `transcribe_command` may remain `#[tauri::command(async)]`: it only captures and
  runs ASR and never reaches a browser op.
- The planner-only and download/key-test commands that were made `(async)` may
  remain `(async)`: they use `reqwest::blocking` / `futures::executor::block_on`,
  not `tauri::async_runtime::block_on`.
- All agent-safety invariants (confirmation gating, side-effect block, step-cycle
  detection, planner validation) still hold.

## Non-goals

- Do not start P1.1.2 / P1.1.3 / P1.1.4 (lock release, browser-op async
  conversion). They remain BLOCKED and out of scope here.
- Do not convert any additional commands to `(async)`.
- Do not add voice-activity detection.

## Design constraints

### 1. Browser-touching commands must run on the main thread (for now)

Until browser ops no longer call `tauri::async_runtime::block_on` from a worker,
every command that can reach a browser op must stay a plain `#[tauri::command]`
(main-thread). The known browser-reaching commands are:

- `transcribe_and_execute_command` (via `execute_command_with_replanning`)
- `execute_planner_output`, `resolve_command` (planner execution can run browser
  tools)
- `submit_confirmation_response` (resume-after-confirmation can run side-effecting
  browser tools)
- `open_url`

`transcribe_and_execute_command` is the one that regressed: revert it to
`#[tauri::command]`. The others are already plain `#[tauri::command]`; leave them
that way and add a short guard comment so a future "finish P1.1.1" pass does not
convert them and reintroduce the panic.

Accept that this re-introduces the pre-existing UI freeze on those paths. That
freeze is the known limitation P1.1.2 is meant to fix; a freeze is preferable to a
crash on the core feature.

### 2. Cached secrets must be zeroized with the `zeroize` crate

Replace the `unsafe { old.as_mut_vec().fill(0) }` approach with the `zeroize`
crate so scrubbing is reliable (volatile writes the optimizer cannot elide) and
applies to every cache entry, not just same-key overwrites.

Preferred shape: store cached values as `zeroize::Zeroizing<String>` so each entry
is zeroed on drop and on replacement automatically:

```rust
use zeroize::Zeroizing;

// session store value type:
BTreeMap<(String, String), Zeroizing<String>>

// insert:
store.insert(key, Zeroizing::new(secret.to_string()));
```

Remove the `unsafe` block entirely. Returning the secret to callers as a plain
`String` for immediate use is acceptable; the durable cached copy is what must be
zeroized.

### 3. The Code Review 2 status must be truthful

Correct `BB_CODE_REVIEW2_TODO.md`:

- P1.1.1 is not fully DONE — several listed commands (`start_listening`,
  `stop_listening`, `resolve_command`, `execute_planner_output`, `open_url`,
  `submit_confirmation_response`) were intentionally not converted, and per
  constraint 1 they must stay sync for now. Re-state P1.1.1 as PARTIAL with that
  reason.
- The final-checklist item "Long-running commands run off the main thread; the
  webview does not freeze" must be unchecked: the lock is still held across the
  blocking work, so converted commands do not actually keep the UI responsive
  while a peer command contends on the lock.

## Expected files touched

- `src-tauri/src/command_handlers/voice_handlers.rs` (revert the one command)
- `src-tauri/src/command_handlers/*.rs` (guard comments on browser-reaching
  commands)
- `src-tauri/src/config/keyring_store.rs` (zeroize)
- `src-tauri/Cargo.toml` (add `zeroize`)
- `docs/BB_CODE_REVIEW2_TODO.md` (status correction)
- `memory.md`

## Acceptance summary

Static checks:

```bash
# transcribe_and_execute_command is back to a plain command; transcribe_command stays async.
rg -n "tauri::command(\(async\))?\]" src-tauri/src/command_handlers/voice_handlers.rs -A1

# No browser-reaching command is (async).
rg -n "command\(async\)" src-tauri/src/command_handlers

# zeroize is used; the unsafe scrub is gone.
rg -n "Zeroizing|zeroize" src-tauri/src/config/keyring_store.rs src-tauri/Cargo.toml
rg -n "as_mut_vec\(\)\.fill\(0\)|unsafe" src-tauri/src/config/keyring_store.rs
```

Expected:

- `transcribe_and_execute_command` is `#[tauri::command]`; `transcribe_command`
  remains `#[tauri::command(async)]`.
- The only remaining `(async)` commands are `transcribe_command`,
  `test_remote_*_api_key`, `list_remote_planner_models`, and the two
  `download_active_local_*_model` commands — none of which reach a browser op.
- `keyring_store.rs` contains no `unsafe` and uses `Zeroizing`.

Behavioral acceptance:

- A voice command that navigates / clicks / reads the page no longer panics the
  worker thread. (Verify under `--features full` with a real page, since
  `cargo test` does not exercise this path.)

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

Only after those pass: update the checklist and add the follow-up `memory.md`
entry with a real UTC timestamp.
