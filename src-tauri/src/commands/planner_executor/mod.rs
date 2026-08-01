use super::*;

mod execution;
mod step_helpers;
mod tool_dispatch;

#[cfg(test)]
pub(crate) use execution::{
    execute_planner_output_with_runner, execute_planner_output_with_runner_and_context,
};
pub use tool_dispatch::execute_planned_step;

struct StepExecutionContext<'a> {
    request_id: String,
    intent_name: IntentName,
    selected_skills: Vec<String>,
    steps: &'a [PlannedStep],
    initial_step_id: String,
    block_side_effects_until_confirmation: bool,
    confirmation_context: &'a ConfirmationRuntimeContext,
}

pub fn execute_planner_output<E: DeterministicToolExecutor>(
    executor: &mut E,
    request_id: String,
    planner_output: &PlannerOutput,
) -> ExecutionOutcome {
    let context = ConfirmationRuntimeContext::detached();
    execute_planner_output_with_context(executor, request_id, planner_output, &context)
}

pub fn execute_planner_output_with_context<E: DeterministicToolExecutor>(
    executor: &mut E,
    request_id: String,
    planner_output: &PlannerOutput,
    context: &ConfirmationRuntimeContext,
) -> ExecutionOutcome {
    execution::execute_planner_output_with_runner_and_context(
        request_id,
        planner_output,
        context,
        |step| tool_dispatch::execute_planned_step(executor, step),
    )
}

pub fn resume_after_confirmation<E: DeterministicToolExecutor>(
    executor: &mut E,
    pending_plan_execution: &PendingPlanExecutionState,
    confirmation_id: &str,
    confirmed: bool,
) -> ExecutionOutcome {
    let context = ConfirmationRuntimeContext {
        page_id: pending_plan_execution.manifest.page_id.clone(),
        page_url: pending_plan_execution.manifest.origin.clone(),
        now_ms: pending_plan_execution.manifest.issued_at_ms,
    };
    resume_after_confirmation_with_context(
        executor,
        pending_plan_execution,
        confirmation_id,
        &pending_plan_execution.manifest_digest,
        confirmed,
        &context,
    )
}

pub fn resume_after_confirmation_with_context<E: DeterministicToolExecutor>(
    executor: &mut E,
    pending_plan_execution: &PendingPlanExecutionState,
    confirmation_id: &str,
    confirmation_digest: &str,
    confirmed: bool,
    context: &ConfirmationRuntimeContext,
) -> ExecutionOutcome {
    execution::resume_after_confirmation_with_runner_and_context(
        pending_plan_execution,
        confirmation_id,
        confirmation_digest,
        confirmed,
        context,
        |step| tool_dispatch::execute_planned_step(executor, step),
    )
}
