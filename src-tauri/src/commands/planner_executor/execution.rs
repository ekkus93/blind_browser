use super::super::{
    build_confirmation_manifest, evaluate_action_policy, validate_pending_confirmation_manifest,
    ConfirmationRequirement, ConfirmationRuntimeContext, ExecutionOutcome, ExecutionTrace,
    PendingPlanExecutionState, PlannedStep, PlannerOutput, PlannerSafetySettings, PlannerStatus,
    SerializedToolResult, StepTransition, ToolError,
};
use super::step_helpers::{
    build_step_positions, extract_confirmation_id, queued_step_ids_after, queued_steps_after,
};
use super::tool_dispatch::is_side_effecting_tool;
use super::StepExecutionContext;
use std::collections::HashSet;

fn executor_minimum_safety() -> PlannerSafetySettings {
    PlannerSafetySettings {
        confirmation_confidence_threshold: 1.0,
        allow_click_without_confirmation: false,
        always_confirm_submit: true,
    }
}

fn initial_execution_policy_error(planner_output: &PlannerOutput) -> Option<ToolError> {
    let decision = evaluate_action_policy(&planner_output.steps, &executor_minimum_safety());
    match decision.requirement {
        ConfirmationRequirement::Prohibited => Some(ToolError {
            code: String::from("prohibited_action_at_execution"),
            message: String::from(
                "executor refused an action prohibited by deterministic runtime policy",
            ),
            retryable: false,
            details: serde_json::to_value(decision).ok(),
        }),
        ConfirmationRequirement::ConfirmationRequired
            if planner_output.status != PlannerStatus::NeedsConfirmation =>
        {
            Some(ToolError {
                code: String::from("unconfirmed_side_effect_at_execution"),
                message: String::from(
                    "executor refused a protected action that reached dispatch without confirmation",
                ),
                retryable: false,
                details: serde_json::to_value(decision).ok(),
            })
        }
        ConfirmationRequirement::NoConfirmation
        | ConfirmationRequirement::ConfirmationRequired => None,
    }
}

fn resumed_execution_policy_error(steps: &[PlannedStep]) -> Option<ToolError> {
    let decision = evaluate_action_policy(steps, &executor_minimum_safety());
    if decision.requirement == ConfirmationRequirement::Prohibited {
        return Some(ToolError {
            code: String::from("prohibited_action_at_execution"),
            message: String::from("executor refused a prohibited action even after confirmation"),
            retryable: false,
            details: serde_json::to_value(decision).ok(),
        });
    }
    None
}

#[cfg(test)]
pub(crate) fn execute_planner_output_with_runner<Runner>(
    request_id: String,
    planner_output: &PlannerOutput,
    run_step: Runner,
) -> ExecutionOutcome
where
    Runner: FnMut(&PlannedStep) -> SerializedToolResult,
{
    let context = ConfirmationRuntimeContext::detached();
    execute_planner_output_with_runner_and_context(request_id, planner_output, &context, run_step)
}

pub(crate) fn execute_planner_output_with_runner_and_context<Runner>(
    request_id: String,
    planner_output: &PlannerOutput,
    confirmation_context: &ConfirmationRuntimeContext,
    mut run_step: Runner,
) -> ExecutionOutcome
where
    Runner: FnMut(&PlannedStep) -> SerializedToolResult,
{
    let trace = ExecutionTrace {
        executed_step_ids: Vec::new(),
        tool_results: Vec::new(),
    };

    if let Some(error) = initial_execution_policy_error(planner_output) {
        return ExecutionOutcome::Aborted { trace, error };
    }

    match planner_output.status {
        PlannerStatus::Complete => {
            return ExecutionOutcome::Complete { trace };
        }
        PlannerStatus::Blocked => {
            return ExecutionOutcome::Aborted {
                trace,
                error: ToolError {
                    code: String::from("planner_blocked"),
                    message: planner_output.user_message.clone().unwrap_or_else(|| {
                        String::from("planner blocked execution before any tools could run")
                    }),
                    retryable: false,
                    details: Some(serde_json::json!({
                        "blocked_reason": planner_output.blocked_reason,
                    })),
                },
            };
        }
        PlannerStatus::Ready | PlannerStatus::NeedsConfirmation => {}
    }

    let Some(current_step_id) = planner_output
        .steps
        .first()
        .map(|step| step.step_id.clone())
    else {
        return ExecutionOutcome::Aborted {
            trace,
            error: ToolError {
                code: String::from("empty_plan"),
                message: String::from("planner returned no executable steps"),
                retryable: false,
                details: Some(serde_json::json!({
                    "planner_status": planner_output.status,
                })),
            },
        };
    };

    execute_steps_with_runner(
        StepExecutionContext {
            request_id,
            intent_name: planner_output.intent.name.clone(),
            selected_skills: planner_output.selected_skills.clone(),
            steps: &planner_output.steps,
            initial_step_id: current_step_id,
            block_side_effects_until_confirmation: planner_output.status
                == PlannerStatus::NeedsConfirmation,
            confirmation_context,
        },
        &mut run_step,
        trace,
    )
}

pub(super) fn resume_after_confirmation_with_runner_and_context<Runner>(
    pending_plan_execution: &PendingPlanExecutionState,
    confirmation_id: &str,
    confirmation_digest: &str,
    confirmed: bool,
    confirmation_context: &ConfirmationRuntimeContext,
    mut run_step: Runner,
) -> ExecutionOutcome
where
    Runner: FnMut(&PlannedStep) -> SerializedToolResult,
{
    let trace = ExecutionTrace {
        executed_step_ids: Vec::new(),
        tool_results: Vec::new(),
    };

    if pending_plan_execution.confirmation_id != confirmation_id {
        return ExecutionOutcome::Aborted {
            trace,
            error: ToolError {
                code: String::from("confirmation_id_mismatch"),
                message: String::from(
                    "confirmation response did not match the pending confirmation id",
                ),
                retryable: false,
                details: Some(serde_json::json!({
                    "expected_confirmation_id": pending_plan_execution.confirmation_id,
                    "received_confirmation_id": confirmation_id,
                })),
            },
        };
    }

    if let Err(error) = validate_pending_confirmation_manifest(
        pending_plan_execution,
        confirmation_digest,
        confirmation_context,
    ) {
        return ExecutionOutcome::Aborted { trace, error };
    }

    if let Some(error) = resumed_execution_policy_error(&pending_plan_execution.queued_steps) {
        return ExecutionOutcome::Aborted { trace, error };
    }

    if !confirmed {
        return ExecutionOutcome::NeedsReplan { trace };
    }

    let Some(next_step_id) = pending_plan_execution.next_step_id.clone() else {
        return ExecutionOutcome::Complete { trace };
    };

    execute_steps_with_runner(
        StepExecutionContext {
            request_id: pending_plan_execution.request_id.clone(),
            intent_name: pending_plan_execution.intent_name.clone(),
            selected_skills: pending_plan_execution.selected_skills.clone(),
            steps: &pending_plan_execution.queued_steps,
            initial_step_id: next_step_id,
            block_side_effects_until_confirmation: false,
            confirmation_context,
        },
        &mut run_step,
        trace,
    )
}

fn execute_steps_with_runner<Runner>(
    context: StepExecutionContext<'_>,
    run_step: &mut Runner,
    mut trace: ExecutionTrace,
) -> ExecutionOutcome
where
    Runner: FnMut(&PlannedStep) -> SerializedToolResult,
{
    let StepExecutionContext {
        request_id,
        intent_name,
        selected_skills,
        steps,
        initial_step_id,
        block_side_effects_until_confirmation,
        confirmation_context,
    } = context;

    let step_positions = match build_step_positions(steps) {
        Ok(positions) => positions,
        Err(error) => {
            return ExecutionOutcome::Aborted { trace, error };
        }
    };

    if !step_positions.contains_key(&initial_step_id) {
        return ExecutionOutcome::Aborted {
            trace,
            error: ToolError {
                code: String::from("missing_resume_step"),
                message: format!(
                    "pending execution referenced missing step '{}'",
                    initial_step_id
                ),
                retryable: false,
                details: None,
            },
        };
    }

    let mut current_step_id = initial_step_id;
    let mut visited_step_ids = HashSet::new();
    loop {
        if !visited_step_ids.insert(current_step_id.clone()) {
            return ExecutionOutcome::Aborted {
                trace,
                error: ToolError {
                    code: String::from("planner_step_cycle"),
                    message: format!(
                        "planner attempted to execute step '{}' more than once",
                        current_step_id
                    ),
                    retryable: false,
                    details: None,
                },
            };
        }

        let Some(step_position) = step_positions.get(&current_step_id) else {
            return ExecutionOutcome::Aborted {
                trace,
                error: ToolError {
                    code: String::from("missing_step_position"),
                    message: format!(
                        "step '{}' was not found in the step position map",
                        current_step_id
                    ),
                    retryable: false,
                    details: None,
                },
            };
        };
        let step = &steps[*step_position];

        if block_side_effects_until_confirmation && is_side_effecting_tool(&step.tool_name) {
            return ExecutionOutcome::Aborted {
                trace,
                error: ToolError {
                    code: String::from("side_effect_before_confirmation"),
                    message: format!(
                        "planner attempted to execute side-effecting tool {:?} before confirmation",
                        step.tool_name
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({
                        "step_id": step.step_id,
                        "tool_name": step.tool_name,
                    })),
                },
            };
        }

        let mut result = run_step(step);
        let transition = if result.ok {
            step.on_success.clone()
        } else {
            step.on_failure.clone()
        };

        if transition == StepTransition::RequestConfirmation {
            let confirmation_id = match extract_confirmation_id(&result) {
                Ok(confirmation_id) => confirmation_id,
                Err(error) => {
                    return ExecutionOutcome::Aborted { trace, error };
                }
            };
            let queued_step_ids = queued_step_ids_after(steps, step, &step_positions);
            let queued_steps = queued_steps_after(steps, step, &step_positions);
            let built_manifest =
                match build_confirmation_manifest(&request_id, &queued_steps, confirmation_context)
                {
                    Ok(manifest) => manifest,
                    Err(error) => {
                        return ExecutionOutcome::Aborted { trace, error };
                    }
                };

            if let Some(serde_json::Value::Object(data)) = result.data.as_mut() {
                data.insert(
                    String::from("prompt_text"),
                    serde_json::Value::String(built_manifest.prompt_text.clone()),
                );
                data.insert(
                    String::from("manifest_digest"),
                    serde_json::Value::String(built_manifest.digest.clone()),
                );
            }

            trace.executed_step_ids.push(step.step_id.clone());
            trace.tool_results.push(result);

            let pending_plan_execution = PendingPlanExecutionState {
                request_id,
                intent_name: intent_name.clone(),
                selected_skills: selected_skills.clone(),
                confirmation_id: confirmation_id.clone(),
                manifest_digest: built_manifest.digest,
                manifest: built_manifest.manifest,
                prompt_text: built_manifest.prompt_text,
                next_step_id: queued_step_ids.first().cloned(),
                queued_step_ids,
                queued_steps,
            };

            return ExecutionOutcome::AwaitingConfirmation {
                trace,
                pending_confirmation_id: confirmation_id,
                pending_plan_execution: Box::new(pending_plan_execution),
            };
        }

        trace.executed_step_ids.push(step.step_id.clone());
        trace.tool_results.push(result);

        match transition {
            StepTransition::NextStep { step_id } => {
                if !step_positions.contains_key(step_id.as_str()) {
                    return ExecutionOutcome::Aborted {
                        trace,
                        error: ToolError {
                            code: String::from("missing_transition_step"),
                            message: format!(
                                "planner referenced missing next step '{}' from '{}'",
                                step_id, step.step_id
                            ),
                            retryable: false,
                            details: None,
                        },
                    };
                }
                current_step_id = step_id;
            }
            StepTransition::Complete => {
                return ExecutionOutcome::Complete { trace };
            }
            StepTransition::RequestConfirmation => unreachable!("handled before trace commit"),
            StepTransition::Replan => {
                return ExecutionOutcome::NeedsReplan { trace };
            }
        }
    }
}
