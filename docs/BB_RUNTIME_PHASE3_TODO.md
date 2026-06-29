# Blind Browser Runtime Phase 3 + Tidy-up TODO

## How to use this file

This collects the two leftovers from the async-runtime pass plus the optional
Phase 3 that `BB_ASYNC_RUNTIME_TODO.md` P1.2 deferred. Read
`BB_RUNTIME_PHASE3_SPEC.md` first.

Recommended scope: **do Part A (P0–P1). Part B (Phase 3 proper, P2) is optional and
low-value — only do it if remote-provider responsiveness during network calls is
actually wanted.** Phases 1–2 already delivered the important wins.

Work top-to-bottom. Keep the gate green between tasks.

Status key: `PENDING` · `IN PROGRESS` · `DONE` · `BLOCKED`

Priority key:

- `P0`: trivial doc correctness (the TODO currently contradicts itself).
- `P1`: low-risk de-duplication that prevents future drift.
- `P2`: optional Phase 3 network lock-scoping (structural; defer-friendly).

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

Do not mark a task complete unless the gate passes.

---

## P0.1 — Fix the stale checklist lines in `BB_CODE_REVIEW2_TODO.md`

**Status:** DONE  
**Files:**

- `docs/BB_CODE_REVIEW2_TODO.md`

### Problem

Two final-checklist lines still describe the pre-Phase-2 state and contradict the
reconciliation block above them (which already records P1.1.2 as DONE).

### P0.1.1 — Reword the capture/network lock line

Change the line reading "The AppCore lock is not held across blocking capture /
network calls. (P1.1.2/P1.1.3 BLOCKED — requires CaptureHandle extraction)" to
reflect that capture lock-scoping is DONE (Phase 2) and only the remote-network
half (P1.1.3 / Phase 3) remains deferred.

### P0.1.2 — Reword the stop-interrupt line

Change the line reading "`stop_listening` can interrupt an active capture;
`get_agent_state` returns promptly during an active operation. (BLOCKED — depends
on P1.1.2)" to: capture interruption landed in Phase 2 (code + regression test),
live `--features full` verification still pending; the during-a-network-call half
remains Phase 3 / deferred.

### Acceptance checks

```bash
rg -n "BLOCKED — depends on P1.1.2|requires CaptureHandle extraction" docs/BB_CODE_REVIEW2_TODO.md || echo "stale lines fixed"
```

Expected: no stale lines remain; the checklist agrees with the reconciliation
block.

---

## P1.1 — De-duplicate the two transcription result/observation paths

**Status:** DONE  
**Files:**

- `src-tauri/src/app_core/listening_tools.rs`

### Problem

`execute_transcribe_command` (the planner-dispatched, lock-held path) and
`finish_transcribe_command` (the phased, lock-released path) build the same
observation strings and the same `TranscribeCommandData` success result. The
duplication will drift. The two paths *should* keep their different capture
mechanisms; only the result construction should be shared.

### P1.1.1 — Extract a shared result/observation helper

Add a helper such as `transcribe_success_result(request_id,
requested_duration_ms, effective_duration_ms, timeout_ms, asr_result)` (and/or
`build_transcribe_observations(...)`) and call it from both
`execute_transcribe_command` and `finish_transcribe_command`. Do not change the
observed strings or behavior — this is a pure de-duplication; existing tests should
stay green unchanged.

### P1.1.2 — Cross-reference the two paths

Add a one-line comment on each path noting the other exists, that path 1 (handlers)
releases the lock across capture while path 2 (planner `TranscribeCommand` tool)
runs under the held lock by design, and that they share the result helper.

### Acceptance checks

```bash
rg -n "transcribe_success_result|build_transcribe_observations" src-tauri/src/app_core/listening_tools.rs
```

Expected: one shared helper, used by both paths; no behavioral change; tests green.

---

## P2.1 — (Optional) Phase 3: lock-scope the remote planner network call

**Status:** DONE (implemented at user request). The deterministic resolution +
profile snapshot run under a brief lock (`build_planner_resolution`); the LLM
round-trip (`resolve_remote_planner`, a free function) runs with the guard
released; `execute_plan` re-acquires. Driven by `LockScopedReplanningRuntime`
(`app_core/replanning_orchestrator.rs`) through the existing
`execute_bounded_replanning_loop`. The atomicity tradeoff (resolve runs against a
snapshot a peer could change before execute) is documented at the call site; the
replan bound is unchanged. Gate green under `--all-features`; behavioral
`--features full` + remote-profile verification still pending.  
**Files:**

- `src-tauri/src/command_handlers/voice_handlers.rs` / `core_handlers.rs`
- `src-tauri/src/app_core/remote_planner.rs`
- `src-tauri/src/commands/planner_executor/*`

### Problem

The remote planner round-trip runs under the `AppCore` lock, so state reads can't
interleave during a remote plan resolution. Releasing the lock around only the
network call needs the resolve/execute alternation hoisted out of the `&mut self`
executor into a handler-level orchestrator.

### P2.1.1 — Split resolve / execute

Split `execute_command_with_replanning` into: `build_planner_input(&self, ...)`, a
free/unlocked `resolve_remote(profile, secret, input)`, and
`execute_resolved_plan(&mut self, ...) -> ExecutionOutcome` that returns control to
the orchestrator between replan cycles.

### P2.1.2 — Add the lock-scoped orchestrator

Mirror `run_phased_transcribe`: lock → build input + snapshot profile/secret → drop
→ network resolve unlocked → re-lock → execute → on `NeedsReplan`, repeat (within
`MAX_COMMAND_REPLAN_CYCLES`).

### P2.1.3 — Make the atomicity tradeoff explicit

Releasing the lock between resolve and execute means resolution runs against a state
snapshot that could change before execution. Document this in a comment, keep each
cycle's snapshot self-consistent, and do not widen the replan bound. If this
tradeoff is unwanted, do not do P2.1 — leave the planner call under the lock.

### Acceptance checks

Behavioral, under `--features full` with a remote planner profile: `get_agent_state`
returns promptly during an in-flight remote resolve; a remote-resolved voice command
produces the same plan/outcome as before.

---

## P2.2 — (Optional, lowest priority) Phase 3 remote ASR

**Status:** SKIPPED (consciously, per spec). Remote ASR runs under the lock inside
`finish_transcribe_command`; scoping it out is a five-phase dance for a remote-only,
default-off path that the spec flags as the lowest-value item ("skip unless there is
a concrete reason"). No concrete reason exists, so it is deliberately left as-is.  
**Files:**

- `src-tauri/src/app_core/listening_tools.rs`, `src-tauri/src/asr/remote.rs`

Scope the remote ASR round-trip out of the held lock (drain→transcribe-unlocked→
record). This is remote-only and default-off; it is the lowest-value item in the
backlog. Do not gold-plate it.

---

## P2.3 — Validation, status reconciliation, memory

**Status:** DONE  
**Files:**

- `docs/BB_ASYNC_RUNTIME_TODO.md`, `memory.md`

- Run the full gate after each landed task.
- If Part B (P2.1) lands, mark `BB_ASYNC_RUNTIME_TODO.md` P1.2 (and CR2 P1.1.3)
  resolved; otherwise leave them deferred.
- Add a `memory.md` entry with a real UTC timestamp noting what landed and what was
  verified live vs. pending. Do not fabricate timestamps.

---

## Suggested commit sequence

1. `docs: fix stale Code Review 2 checklist lines`
2. `refactor(asr): share transcribe result/observation builder across both paths`
3. `refactor(planner): lock-scope the remote resolve (optional Phase 3)`
4. `docs: reconcile async-runtime Phase 3 status and record validation`

---

## Final done checklist

- [x] `BB_CODE_REVIEW2_TODO.md` checklist no longer contradicts its reconciliation
      block.
- [x] A shared transcribe result/observation helper is used by both transcription
      paths; behavior unchanged; tests green.
- [x] The two transcription paths cross-reference each other.
- [x] (Optional) Remote planner resolve is lock-scoped with the atomicity tradeoff
      documented (implemented at user request).
- [x] (Optional) Remote ASR lock-scoping consciously skipped (lowest-value,
      remote-only; no concrete reason).
- [x] Statuses in `BB_ASYNC_RUNTIME_TODO.md` reconciled with what actually landed.
- [x] Full validation gate passes (`--all-features`). Note: the default-feature
      build is broken by pre-existing issues in `browser/`/`tts/`/`asr` modules
      (present before this pass, unrelated to it); the project's gate is
      `--all-features`.
- [x] `memory.md` has a real UTC entry noting live-verified vs. pending.
