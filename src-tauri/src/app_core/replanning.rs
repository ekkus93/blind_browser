use crate::commands::{
    ExecutionOutcome, ExecutionTrace, PlannerOutput, PlannerToolHistoryEntry,
    RemotePlannerConsentChallenge, ToolError,
};

pub(crate) enum ResolvePlanOutcome {
    Resolved(PlannerOutput),
    NeedsRemoteDataConsent(RemotePlannerConsentChallenge),
}

pub(crate) trait ReplanningRuntime {
    fn resolve_plan(
        &mut self,
        request_id: String,
        transcript: &str,
        recent_tool_results: &[PlannerToolHistoryEntry],
    ) -> Result<ResolvePlanOutcome, ToolError>;

    fn execute_plan(
        &mut self,
        request_id: String,
        planner_output: &PlannerOutput,
    ) -> ExecutionOutcome;
}

fn execution_trace_to_tool_history_entries(trace: &ExecutionTrace) -> Vec<PlannerToolHistoryEntry> {
    trace
        .tool_results
        .iter()
        .map(|result| PlannerToolHistoryEntry {
            tool_name: result.tool_name.clone(),
            ok: result.ok,
            observation_summary: result.observations.clone(),
        })
        .collect()
}

fn append_execution_trace(into: &mut ExecutionTrace, trace: ExecutionTrace) {
    into.executed_step_ids.extend(trace.executed_step_ids);
    into.tool_results.extend(trace.tool_results);
}

fn merge_execution_outcome_trace(
    mut trace: ExecutionTrace,
    outcome: ExecutionOutcome,
) -> ExecutionOutcome {
    match outcome {
        ExecutionOutcome::Complete { trace: next_trace } => {
            append_execution_trace(&mut trace, next_trace);
            ExecutionOutcome::Complete { trace }
        }
        ExecutionOutcome::AwaitingConfirmation {
            trace: next_trace,
            pending_confirmation_id,
            pending_plan_execution,
        } => {
            append_execution_trace(&mut trace, next_trace);
            ExecutionOutcome::AwaitingConfirmation {
                trace,
                pending_confirmation_id,
                pending_plan_execution,
            }
        }
        ExecutionOutcome::NeedsReplan { trace: next_trace } => {
            append_execution_trace(&mut trace, next_trace);
            ExecutionOutcome::NeedsReplan { trace }
        }
        ExecutionOutcome::NeedsRemoteDataConsent {
            trace: next_trace,
            challenge,
        } => {
            append_execution_trace(&mut trace, next_trace);
            ExecutionOutcome::NeedsRemoteDataConsent { trace, challenge }
        }
        ExecutionOutcome::Aborted {
            trace: next_trace,
            error,
        } => {
            append_execution_trace(&mut trace, next_trace);
            ExecutionOutcome::Aborted { trace, error }
        }
    }
}

fn replanning_request_id(base_request_id: &str, phase: &str, replan_cycle: usize) -> String {
    if replan_cycle == 0 {
        format!("{base_request_id}-{phase}")
    } else {
        format!("{base_request_id}-{phase}-replan-{replan_cycle}")
    }
}

pub(crate) fn execute_bounded_replanning_loop<R: ReplanningRuntime>(
    runtime: &mut R,
    request_id: &str,
    transcript: &str,
) -> Result<ExecutionOutcome, ToolError> {
    let mut replan_cycle = 0usize;
    let mut recent_tool_results = Vec::<PlannerToolHistoryEntry>::new();
    let mut accumulated_trace = ExecutionTrace {
        executed_step_ids: Vec::new(),
        tool_results: Vec::new(),
    };

    loop {
        let planner_output = match runtime.resolve_plan(
            replanning_request_id(request_id, "resolve", replan_cycle),
            transcript,
            &recent_tool_results,
        ) {
            Ok(ResolvePlanOutcome::Resolved(planner_output)) => planner_output,
            Ok(ResolvePlanOutcome::NeedsRemoteDataConsent(challenge)) => {
                return Ok(ExecutionOutcome::NeedsRemoteDataConsent {
                    trace: accumulated_trace,
                    challenge,
                });
            }
            Err(error) => {
                if accumulated_trace.executed_step_ids.is_empty()
                    && accumulated_trace.tool_results.is_empty()
                {
                    return Err(error);
                }
                return Ok(ExecutionOutcome::Aborted {
                    trace: accumulated_trace,
                    error,
                });
            }
        };

        let outcome = runtime.execute_plan(
            replanning_request_id(request_id, "execute", replan_cycle),
            &planner_output,
        );
        recent_tool_results.extend(execution_trace_to_tool_history_entries(match &outcome {
            ExecutionOutcome::Complete { trace }
            | ExecutionOutcome::AwaitingConfirmation { trace, .. }
            | ExecutionOutcome::NeedsReplan { trace }
            | ExecutionOutcome::NeedsRemoteDataConsent { trace, .. }
            | ExecutionOutcome::Aborted { trace, .. } => trace,
        }));

        match outcome {
            ExecutionOutcome::NeedsReplan { trace } => {
                append_execution_trace(&mut accumulated_trace, trace);
                if replan_cycle >= super::MAX_COMMAND_REPLAN_CYCLES {
                    return Ok(ExecutionOutcome::Aborted {
                        trace: accumulated_trace,
                        error: ToolError {
                            code: String::from("replan_limit_exceeded"),
                            message: format!(
                                "planner requested replanning more than {} time(s) for this command",
                                super::MAX_COMMAND_REPLAN_CYCLES
                            ),
                            retryable: true,
                            details: Some(serde_json::json!({
                                "max_replan_cycles": super::MAX_COMMAND_REPLAN_CYCLES,
                            })),
                        },
                    });
                }
                replan_cycle += 1;
            }
            other => return Ok(merge_execution_outcome_trace(accumulated_trace, other)),
        }
    }
}
