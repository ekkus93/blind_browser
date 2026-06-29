use super::*;

mod execution;
mod step_helpers;
mod tool_dispatch;

#[cfg(test)]
pub(crate) use execution::execute_planner_output_with_runner;
pub use tool_dispatch::execute_planned_step;

struct StepExecutionContext<'a> {
    request_id: String,
    intent_name: IntentName,
    selected_skills: Vec<String>,
    steps: &'a [PlannedStep],
    initial_step_id: String,
    block_side_effects_until_confirmation: bool,
}

pub fn execute_planner_output<E: DeterministicToolExecutor>(
    executor: &mut E,
    request_id: String,
    planner_output: &PlannerOutput,
) -> ExecutionOutcome {
    execution::execute_planner_output_with_runner(request_id, planner_output, |step| {
        tool_dispatch::execute_planned_step(executor, step)
    })
}

pub fn resume_after_confirmation<E: DeterministicToolExecutor>(
    executor: &mut E,
    pending_plan_execution: &PendingPlanExecutionState,
    confirmation_id: &str,
    confirmed: bool,
) -> ExecutionOutcome {
    execution::resume_after_confirmation_with_runner(
        pending_plan_execution,
        confirmation_id,
        confirmed,
        |step| tool_dispatch::execute_planned_step(executor, step),
    )
}
