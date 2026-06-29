# Blind Browser Runtime Phase 3 + Tidy-up Spec

## Purpose

Two cheap tidy-ups left over from the async-runtime pass, plus the optional Phase 3
that `BB_ASYNC_RUNTIME_TODO.md` P1.2 deferred (BLOCKED).

Read the priority framing before starting: **the tidy-ups are the recommended
work; Phase 3 proper is optional and low-value.** Phases 1–2 already removed the UI
freeze, the `block_on` panic, and made capture interruptible. Phase 3 only buys
state-read interleaving *during a remote planner / ASR network call*, which matters
only to remote-provider users (the project defaults to local). The team's earlier
decision to defer it was sound. This spec makes it implementable if and when it's
wanted, but does not push it.

## Current known good behavior to preserve

- All async-runtime Phase 1–2 behavior: `Arc<Mutex<AppCore>>` managed state, long
  commands as `async fn` + `spawn_blocking`, `std::sync::Mutex` retained, the
  phased lock-released capture window, and the `Ok(None)` stop-during-capture
  result.
- All earlier fixes (ASR drain, panic-to-error, zeroize, CI fmt) and all
  agent-safety invariants.
- The existing synchronous execution logic is unchanged in behavior; this pass
  relocates lock boundaries and de-duplicates, it does not change what is computed.

## Non-goals

- Do not migrate to `tokio::sync::Mutex`.
- Do not convert any command to `#[tauri::command(async)]` (the worker form). The
  `spawn_blocking` bridge stays.
- Do not change the planner's decision logic, tool contracts, or confirmation flow.

## Part A — Tidy-ups (recommended)

### A1. Fix the stale checklist lines in `BB_CODE_REVIEW2_TODO.md`

Two final-checklist lines still describe the pre-Phase-2 state and now contradict
the reconciliation block above them:

- "The AppCore lock is not held across blocking capture / network calls.
  (P1.1.2/P1.1.3 BLOCKED — requires CaptureHandle extraction)" — P1.1.2 (capture)
  is now DONE; only the network half (P1.1.3) remains deferred. Reword to reflect
  capture = done, network = deferred.
- "`stop_listening` can interrupt an active capture; `get_agent_state` returns
  promptly during an active operation. (BLOCKED — depends on P1.1.2)" — the capture
  half landed in Phase 2; reword to "code landed, live `--features full`
  verification pending" and keep the network half as deferred.

Doc-only. Make the checklist consistent with the reconciliation block.

### A2. De-duplicate the two transcription result/observation paths

There are intentionally two transcription paths: the top-level handlers use the
phased lock-released path (`begin_transcribe_command` / `finish_transcribe_command`),
and the planner can invoke a `TranscribeCommand` tool mid-plan via
`tool_dispatch → execute_transcribe_command → asr.transcribe_command`. The planner
path legitimately stays synchronous and lock-held (it runs inside plan execution,
which is one locked transaction off the main thread). **Keep that capture-mechanism
difference.**

The problem is only the duplicated *result/observation construction*:
`execute_transcribe_command` and `finish_transcribe_command` build the same
observation strings ("Captured microphone audio…", the clamp note, the timeout
note, the transcript-present note) and the same `TranscribeCommandData` success
result. That duplication will drift.

- Extract a shared helper, e.g.
  `fn transcribe_success_result(request_id, requested_duration_ms,
  effective_duration_ms, timeout_ms, asr_result) -> ToolResult<TranscribeCommandData>`
  (and/or a `build_transcribe_observations(...)`), and call it from both paths.
- Add a one-line comment on each path pointing at the other and noting that path 1
  releases the lock across capture while path 2 (planner tool) runs under the held
  lock by design.

Do not unify the capture orchestration — only the result/observation building.

## Part B — Phase 3: lock-scope remote network calls (optional, low-value)

Only do this if state-read responsiveness during remote provider calls is actually
wanted. It is a structural change to the planner-executor control flow, not a
surgical edit.

### Design

The remote planner round-trip (`resolve_with_openai_planner` /
`resolve_with_ollama_planner`, a `futures::executor::block_on` on the LLM client)
currently runs inside `execute_command_with_replanning` while the `AppCore` lock is
held (in `spawn_blocking`). To release the lock across only the network call, hoist
the resolve/execute alternation out of the `&mut self` method and into a handler-
level orchestrator that owns the `Arc<Mutex<AppCore>>` (the same shape as
`run_phased_transcribe`):

1. **Build planner input under the lock**, then drop the guard. Snapshot whatever
   the planner request needs (the planner input, the resolved
   `RemotePlannerProfile`, and the resolved secret) so the network step needs no
   `AppCore` access.
2. **Run the network resolve unlocked** — `futures::executor::block_on(...)` on the
   blocking-pool thread, with no guard held.
3. **Re-acquire the lock and execute the resolved plan.** If the outcome is
   `NeedsReplan`, repeat from step 1 for the next cycle (respecting the existing
   `MAX_COMMAND_REPLAN_CYCLES` bound).

This requires splitting `execute_command_with_replanning` into composable pieces:
a `build_planner_input(&self, ...)`, a free/unlocked `resolve_remote(profile,
secret, input)`, and an `execute_resolved_plan(&mut self, ...) -> ExecutionOutcome`
that returns control (and any replan input) to the orchestrator between cycles.

### Important design consideration — atomicity

Today, resolve + execute are atomic under one lock. Releasing the lock between them
means the plan is resolved against a state snapshot that another command could, in
principle, change before execution. For this app (single user, frontend-serialized
voice commands) that is acceptable, but it **is** a semantic change. Make it a
conscious choice: keep each resolve→execute cycle's snapshot self-consistent, and
do not widen the replan bound. If this tradeoff is unwanted, do not do Part B —
leave the planner call under the lock (off the main thread is already enough).

### B-optional — remote ASR

The remote ASR round-trip runs under the lock inside `finish_transcribe_command`.
Scoping it would add a drain(lock) → transcribe(unlocked) → record(lock) split — a
five-phase dance for a remote-only, default-off path. **This is the lowest-value
item in the whole backlog; skip it unless there is a concrete reason.** Do not
gold-plate it.

## Expected files touched

Part A:

- `docs/BB_CODE_REVIEW2_TODO.md` (A1)
- `src-tauri/src/app_core/listening_tools.rs` (A2 shared helper + comments)

Part B (only if undertaken):

- `src-tauri/src/command_handlers/voice_handlers.rs` and/or `core_handlers.rs`
  (orchestrator)
- `src-tauri/src/app_core/remote_planner.rs`,
  `src-tauri/src/commands/planner_executor/*` (split resolve/execute)
- `src-tauri/src/app_core/listening_tools.rs`, `asr/remote.rs` (B-optional)
- `docs/BB_ASYNC_RUNTIME_TODO.md` (mark P1.2 resolved if Part B lands)
- `memory.md`

## Acceptance summary

Part A:

```bash
# No self-contradiction left in the CR2 checklist.
rg -n "BLOCKED — depends on P1.1.2|requires CaptureHandle extraction" docs/BB_CODE_REVIEW2_TODO.md || echo "stale lines fixed"
# Shared transcribe-result helper exists and both paths use it.
rg -n "transcribe_success_result|build_transcribe_observations" src-tauri/src/app_core/listening_tools.rs
```

Part B (if undertaken) — behavioral, under `--features full` with a remote planner
profile:

- `get_agent_state` returns promptly while a remote planner call is in flight.
- A voice command that resolves remotely and then navigates still produces the same
  plan and outcome as before (no behavioral regression).

Full validation gate (both parts):

```bash
source ./fix-node-version.sh
pnpm lint
pnpm test:ui
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
```

Only after the gate (and any Part B behavioral checks) pass: update statuses and add
a `memory.md` entry with a real UTC timestamp, noting what was verified live vs.
left pending.
