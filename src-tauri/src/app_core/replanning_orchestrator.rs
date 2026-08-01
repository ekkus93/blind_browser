use std::sync::{Arc, Mutex};

use super::command_dispatch::PlannerResolution;
use super::remote_planner::resolve_remote_planner;
use super::replanning::{execute_bounded_replanning_loop, ReplanningRuntime};
use super::AppCore;
use crate::commands::{
    validate_planner_output_with_safety, ExecutionOutcome, ExecutionTrace, PlannerOutput,
    PlannerToolHistoryEntry, ToolError,
};
use crate::lock_app_core;

/// A [`ReplanningRuntime`] that owns a handle to the managed `Arc<Mutex<AppCore>>`
/// and releases the `AppCore` lock across the remote planner network round-trip.
///
/// Each `resolve_plan` runs the deterministic resolution + profile snapshot under a
/// brief lock, drops the guard, performs the LLM round-trip unlocked, then returns
/// the validated plan. `execute_plan` re-acquires the lock to run the plan. This is
/// the same lock-release shape as `run_phased_transcribe`, applied to the
/// resolve/execute alternation of the bounded replanning loop.
pub(crate) struct LockScopedReplanningRuntime<'a> {
    core: &'a Arc<Mutex<AppCore>>,
}

impl<'a> LockScopedReplanningRuntime<'a> {
    pub(crate) fn new(core: &'a Arc<Mutex<AppCore>>) -> Self {
        Self { core }
    }

    fn resolve(
        &self,
        request_id: String,
        transcript: &str,
        recent_tool_results: Vec<PlannerToolHistoryEntry>,
    ) -> Result<PlannerOutput, ToolError> {
        // Phase 1 (locked): deterministic resolution + remote profile snapshot.
        let resolution = {
            let mut guard = lock_app_core(self.core)?;
            guard.build_planner_resolution(request_id, transcript, recent_tool_results)?
        };

        match resolution {
            PlannerResolution::Direct(planner_output) => Ok(planner_output),
            PlannerResolution::Remote {
                planner_input,
                profile_name,
                profile,
                available_tools,
                active_skill_names,
            } => {
                // Phase 2 (unlocked): the LLM round-trip runs with the guard dropped.
                //
                // ATOMICITY TRADEOFF: resolve and execute are no longer one locked
                // transaction. The plan is resolved against the state snapshot taken
                // in phase 1, which a peer command could in principle change before
                // `execute_plan` re-acquires the lock. This is acceptable for this
                // single-user app whose frontend serializes dependent voice commands
                // with `await`; each resolve→execute cycle keeps its own snapshot
                // self-consistent and the replan bound is unchanged.
                let planner_output =
                    resolve_remote_planner(&profile_name, &profile, &planner_input)?;
                validate_planner_output_with_safety(
                    &planner_output,
                    &available_tools,
                    &active_skill_names,
                    &planner_input.safety,
                )?;
                Ok(planner_output)
            }
        }
    }
}

impl ReplanningRuntime for LockScopedReplanningRuntime<'_> {
    fn resolve_plan(
        &mut self,
        request_id: String,
        transcript: &str,
        recent_tool_results: &[PlannerToolHistoryEntry],
    ) -> Result<PlannerOutput, ToolError> {
        self.resolve(request_id, transcript, recent_tool_results.to_vec())
    }

    fn execute_plan(
        &mut self,
        request_id: String,
        planner_output: &PlannerOutput,
    ) -> ExecutionOutcome {
        match lock_app_core(self.core) {
            Ok(mut guard) => guard.execute_planner_output(request_id, planner_output),
            Err(error) => ExecutionOutcome::Aborted {
                trace: ExecutionTrace {
                    executed_step_ids: Vec::new(),
                    tool_results: Vec::new(),
                },
                error,
            },
        }
    }
}

/// Run a voice command through the bounded replanning loop with the `AppCore` lock
/// released across each remote planner round-trip. Used by the
/// `transcribe_and_execute_command` handler.
pub(crate) fn run_command_with_lock_scoped_replanning(
    core: &Arc<Mutex<AppCore>>,
    request_id: &str,
    transcript: &str,
) -> Result<ExecutionOutcome, ToolError> {
    let mut runtime = LockScopedReplanningRuntime::new(core);
    execute_bounded_replanning_loop(&mut runtime, request_id, transcript)
}

/// Resolve a single command to a plan with the `AppCore` lock released across the
/// remote planner round-trip. Used by the `resolve_command` handler.
pub(crate) fn resolve_command_lock_scoped(
    core: &Arc<Mutex<AppCore>>,
    request_id: String,
    transcript: String,
) -> Result<PlannerOutput, ToolError> {
    LockScopedReplanningRuntime::new(core).resolve(request_id, &transcript, Vec::new())
}
