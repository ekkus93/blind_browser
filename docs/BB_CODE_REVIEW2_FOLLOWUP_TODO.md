# Blind Browser Code Review 2 Follow-up TODO

## How to use this file

This is a small corrective pass after verifying the Code Review 2 implementation.
Work top-to-bottom. Keep the validation gate green between tasks. Do not restart
the BLOCKED async / lock-release work (P1.1.2 / P1.1.3 / P1.1.4 in
`BB_CODE_REVIEW2_TODO.md`) — this pass only makes the current state safe and
honest.

Status key: `PENDING` · `IN PROGRESS` · `DONE` · `BLOCKED`

Priority key:

- `P0`: prevents a runtime crash on the core feature. Fix first.
- `P1`: security hardening that the prior pass marked done but did not deliver.
- `P2`: status/doc correction, validation, and closeout.

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

## P0.1 — Stop `transcribe_and_execute_command` from panicking on browser actions

**Status:** DONE  
**Files:**

- `src-tauri/src/command_handlers/voice_handlers.rs`

### Problem

`transcribe_and_execute_command` was changed to `#[tauri::command(async)]`, so it
runs on a tokio worker thread. It calls `execute_command_with_replanning`, which
runs planner steps that reach browser tools. Those tools call
`tauri::async_runtime::block_on`, which panics — *"Cannot start a runtime from
within a runtime"* — when invoked from a runtime worker. This is reachable on the
core voice → browser path (open URL, click, scroll, read) via push-to-talk and the
continuous-listening loop, and `cargo test` does not catch it (no real Chromium in
unit tests).

This is a regression: when the command ran on the main thread (plain
`#[tauri::command]`), the same `block_on` calls were safe.

### Required behavior

- A voice command that navigates / clicks / reads the page does not panic.
- `transcribe_command` stays `#[tauri::command(async)]` (it never reaches a
  browser op — capture + ASR only).

### P0.1.1 — Revert the one command

In `voice_handlers.rs`, change `transcribe_and_execute_command` from
`#[tauri::command(async)]` back to `#[tauri::command]`. Leave `transcribe_command`
as `#[tauri::command(async)]`.

This re-introduces the pre-existing UI freeze on the execute path. That freeze is
the known limitation P1.1.2 is meant to fix later; a freeze is acceptable, a crash
is not.

### P0.1.2 — Add a guardrail comment on browser-reaching commands

Add a short comment above `transcribe_and_execute_command`, `execute_planner_output`,
`resolve_command`, `submit_confirmation_response`, and `open_url` noting that they
must stay `#[tauri::command]` (not `(async)`) until browser ops stop calling
`tauri::async_runtime::block_on` from a worker thread (see
`BB_CODE_REVIEW2_TODO.md` P1.1.2 / P1.1.4). This prevents a future "finish P1.1.1"
pass from reintroducing the panic.

### Acceptance checks

```bash
rg -n "command\(async\)" src-tauri/src/command_handlers
```

Expected: the only `(async)` commands are `transcribe_command`,
`test_remote_planner_api_key`, `test_remote_tts_api_key`,
`test_remote_asr_api_key`, `list_remote_planner_models`,
`download_active_local_tts_model`, and `download_active_local_asr_model`. None of
these reach a browser op.

Behavioral: under `--features full`, a voice command that opens a URL or clicks an
element completes without a worker-thread panic.

---

## P1.1 — Zeroize cached secrets with the `zeroize` crate

**Status:** DONE  
**Files:**

- `src-tauri/src/config/keyring_store.rs`
- `src-tauri/Cargo.toml`

### Problem

P2.1.2 in the prior pass used a hand-rolled `unsafe { old.as_mut_vec().fill(0) }`
that (a) only scrubs on same-key overwrite, so the common cache-once case is never
cleared, (b) is a dead store the optimizer may elide in release builds, and (c)
adds `unsafe` to a security module for a weaker guarantee than the suggested crate.

### Required behavior

- Every cached secret value is reliably zeroized on drop and on replacement.
- No `unsafe` remains in `keyring_store.rs`.

### P1.1.1 — Add the dependency

Add to `src-tauri/Cargo.toml`:

```toml
zeroize = "1"
```

### P1.1.2 — Store cached secrets as `Zeroizing<String>`

In `keyring_store.rs`:

- Change the session store value type to
  `BTreeMap<(String, String), zeroize::Zeroizing<String>>`.
- Insert with `store.insert(key, Zeroizing::new(secret.to_string()));`.
- Remove the `unsafe { old.as_mut_vec().fill(0) }` block — `Zeroizing` zeroes the
  prior value on replacement automatically.
- Adjust `cached_keyring_secret` to return the secret for immediate use (cloning
  the inner `String` is fine; the durable cached copy is what must be zeroized).

### Acceptance checks

```bash
rg -n "Zeroizing|zeroize" src-tauri/src/config/keyring_store.rs src-tauri/Cargo.toml
rg -n "as_mut_vec\(\)\.fill\(0\)|unsafe" src-tauri/src/config/keyring_store.rs
```

Expected: `Zeroizing` is used; no `unsafe` and no `fill(0)` remain.

---

## P2.1 — Correct the Code Review 2 status and checklist

**Status:** DONE  
**Files:**

- `docs/BB_CODE_REVIEW2_TODO.md`

### Problem

The prior TODO overstates progress: P1.1.1 is marked DONE but several listed
commands were not converted, and per P0.1 they must stay sync for now. The
"no freeze" checklist item is also not met, because the lock is still held across
the blocking work.

### P2.1.1 — Restate P1.1.1 as PARTIAL

In `BB_CODE_REVIEW2_TODO.md`, change the P1.1.1 status from DONE to PARTIAL and
note that `start_listening`, `stop_listening`, `resolve_command`,
`execute_planner_output`, `open_url`, and `submit_confirmation_response` were
intentionally left as plain `#[tauri::command]` and must stay that way until the
browser ops are converted (P1.1.2 / P1.1.4).

### P2.1.2 — Uncheck the "no freeze" item

In the final checklist of `BB_CODE_REVIEW2_TODO.md`, uncheck:

```md
- [ ] Long-running commands run off the main thread; the webview does not freeze.
```

Add a one-line note that converting a command to `(async)` does not deliver this
on its own, because the `AppCore` lock is still held for the full blocking
duration; the real fix is the lock-release work (P1.1.2 / P1.1.3, BLOCKED).

---

## P2.2 — Run the full validation gate

**Status:** DONE (gate green; P0.1 behavioral check under `--features full` still requires manual verification on a real page)  
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

Do not mark this task done unless every command completes successfully.

Note: the gate cannot prove the P0.1 panic is gone, because the browser path is
not exercised by `cargo test`. Confirm P0.1 behaviorally under `--features full`
with a real page in addition to the gate.

---

## P2.3 — Add follow-up memory entry with real UTC timestamp

**Status:** DONE  
**Files:**

- `memory.md`

Only after P2.2 passes. Run:

```bash
date -u +"%Y-%m-%dT%H:%M:%SZ"
```

Add an entry like:

```md
- 2026-XX-XXTXX:XX:XXZ — Code Review 2 follow-up: reverted transcribe_and_execute_command to a sync command to stop a "runtime within a runtime" panic on voice-driven browser actions, added guard comments on browser-reaching commands, replaced the unsafe secret-scrub with zeroize::Zeroizing, and corrected the Code Review 2 status/checklist. Validation gate passed.
```

Replace the timestamp with the actual command output. Do not fabricate or reuse an
old timestamp.

---

## Suggested commit sequence

1. `fix(voice): revert transcribe_and_execute_command to sync to avoid runtime panic`
2. `fix(config): zeroize cached secrets with the zeroize crate`
3. `docs: correct Code Review 2 P1.1.1 status and no-freeze checklist`
4. `docs: record Code Review 2 follow-up validation`

---

## Final done checklist

- [x] `transcribe_and_execute_command` is `#[tauri::command]` again;
      `transcribe_command` stays `(async)`.
- [x] No browser-reaching command is `(async)`; guard comments are in place.
- [ ] A voice command that navigates / clicks / reads no longer panics
      (verified under `--features full`). — code fix landed (revert to sync); the
      live `--features full` behavioral check still needs a human on a real page,
      since `cargo test` does not drive Chromium.
- [x] Cached secrets use `zeroize::Zeroizing`; no `unsafe` remains in
      `keyring_store.rs`.
- [x] `BB_CODE_REVIEW2_TODO.md` P1.1.1 is restated as PARTIAL with the reason.
- [x] The "no freeze" checklist item in `BB_CODE_REVIEW2_TODO.md` is unchecked
      with a note.
- [x] Preserved Code Review 2 fixes (P0.1 drain, P1.2 panics, P2.1.1 id helper,
      P2.1.3 CI fmt) still hold.
- [x] Full validation gate passes.
- [x] `memory.md` has a real UTC follow-up entry.
