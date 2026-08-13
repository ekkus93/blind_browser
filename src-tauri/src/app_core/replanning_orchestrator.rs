use std::sync::{Arc, Mutex};

use super::command_dispatch::PlannerResolution;
use super::lock_scoped_tools::LockScopedStepRunner;
use super::remote_data_consent::{
    NarrationConsentResolution, PendingConsentResolution, PendingRemotePlannerContinuation,
    RemotePlannerPreparation,
};
use super::remote_planner::resolve_remote_planner;
use super::replanning::{execute_bounded_replanning_loop, ReplanningRuntime, ResolvePlanOutcome};
use super::AppCore;
use crate::commands::{
    execute_planner_output_with_runtime_safety_and_runner, resume_after_confirmation_with_runner,
    validate_planner_output_with_safety, ConfirmActionData, ConfirmActionResolution,
    ExecutionOutcome, ExecutionTrace, NarrationConsentResponseOutcome, PlannerOutput,
    PlannerToolHistoryEntry, RemotePlannerConsentDecision, RemotePlannerConsentResponseOutcome,
    ResolveCommandOutcome, ToolError, ToolName, ToolResult,
};
use crate::lock_app_core;

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
        continuation: PendingRemotePlannerContinuation,
    ) -> Result<ResolvePlanOutcome, ToolError> {
        let phase = {
            let mut guard = lock_app_core(self.core)?;
            let planning_snapshot = guard.capture_planning_state_snapshot();
            match guard.build_planner_resolution(request_id, transcript, recent_tool_results)? {
                PlannerResolution::Direct(planner_output) => {
                    ResolvePhase::Direct(planner_output.into_inner(), planning_snapshot)
                }
                PlannerResolution::Remote {
                    planner_input,
                    profile_name,
                    profile,
                    privacy,
                    available_tools,
                    active_skill_names,
                } => match guard.prepare_remote_planner_request(
                    profile_name,
                    profile,
                    *planner_input,
                    &privacy,
                )? {
                    RemotePlannerPreparation::Authorized(prepared) => ResolvePhase::Remote {
                        prepared,
                        planning_snapshot,
                        available_tools,
                        active_skill_names,
                    },
                    RemotePlannerPreparation::ConsentRequired { challenge, draft } => {
                        let challenge = *challenge;
                        let draft = *draft;
                        guard.store_pending_remote_planner_consent(
                            challenge.clone(),
                            draft,
                            planning_snapshot,
                            continuation,
                        );
                        return Ok(ResolvePlanOutcome::NeedsRemoteDataConsent(challenge));
                    }
                },
            }
        };

        let planner_output = match phase {
            ResolvePhase::Direct(planner_output, planning_snapshot) => {
                let mut guard = lock_app_core(self.core)?;
                guard.register_planning_snapshot(&planner_output, planning_snapshot)?;
                return Ok(ResolvePlanOutcome::Resolved(planner_output));
            }
            ResolvePhase::Remote {
                prepared,
                planning_snapshot,
                available_tools,
                active_skill_names,
            } => {
                let safety = prepared.sanitized_input.trusted_runtime.safety.clone();
                let planner_output = resolve_remote_planner(&prepared)?;
                validate_planner_output_with_safety(
                    &planner_output,
                    &available_tools,
                    &active_skill_names,
                    &safety,
                )?;
                let mut guard = lock_app_core(self.core)?;
                guard.register_planning_snapshot(&planner_output, planning_snapshot)?;
                planner_output
            }
        };
        Ok(ResolvePlanOutcome::Resolved(planner_output))
    }
}

enum ResolvePhase {
    Direct(PlannerOutput, crate::state::PlanningStateSnapshot),
    Remote {
        prepared: Box<super::remote_data_consent::PreparedRemotePlannerRequest>,
        planning_snapshot: crate::state::PlanningStateSnapshot,
        available_tools: Vec<crate::commands::AvailableTool>,
        active_skill_names: Vec<String>,
    },
}

impl ReplanningRuntime for LockScopedReplanningRuntime<'_> {
    fn resolve_plan(
        &mut self,
        request_id: String,
        transcript: &str,
        recent_tool_results: &[PlannerToolHistoryEntry],
    ) -> Result<ResolvePlanOutcome, ToolError> {
        self.resolve(
            request_id,
            transcript,
            recent_tool_results.to_vec(),
            PendingRemotePlannerContinuation::Execute,
        )
    }

    fn execute_plan(
        &mut self,
        request_id: String,
        planner_output: &PlannerOutput,
    ) -> ExecutionOutcome {
        execute_planner_output_lock_scoped(self.core, request_id, planner_output)
    }
}

pub(crate) fn execute_planner_output_lock_scoped(
    core: &Arc<Mutex<AppCore>>,
    request_id: String,
    planner_output: &PlannerOutput,
) -> ExecutionOutcome {
    let prepared = {
        let mut guard = match lock_app_core(core) {
            Ok(guard) => guard,
            Err(error) => return execution_abort(error),
        };
        if let Err(error) = guard.begin_lock_scoped_plan_execution() {
            return execution_abort(error);
        }
        match guard.prepare_planner_execution(planner_output) {
            Ok(prepared) => prepared,
            Err(outcome) => {
                guard.end_lock_scoped_plan_execution();
                return outcome;
            }
        }
    };

    let mut runner = LockScopedStepRunner::new(
        core,
        prepared.lock_scoped_execution_token,
        prepared.listening_state,
    );
    let outcome = execute_planner_output_with_runtime_safety_and_runner(
        request_id,
        &prepared.planner_output,
        &prepared.confirmation_context,
        &prepared.safety,
        |step| runner.run(step),
    );
    let outcome = runner.reconcile_outcome(outcome);

    match lock_app_core(core) {
        Ok(mut guard) => {
            guard.end_lock_scoped_plan_execution();
            guard.finish_planner_execution(outcome)
        }
        Err(error) => {
            let trace = execution_trace(outcome);
            ExecutionOutcome::Aborted { trace, error }
        }
    }
}

pub(crate) fn submit_confirmation_response_lock_scoped(
    core: &Arc<Mutex<AppCore>>,
    confirmation_id: String,
    confirmation_digest: String,
    confirmed: bool,
    timed_out: bool,
) -> Result<ConfirmActionResolution, ToolError> {
    let should_resume = confirmed && !timed_out;
    let (prompt_text, prepared) = {
        let mut guard = lock_app_core(core)?;
        let prompt_text = match guard
            .state
            .pending_plan_execution
            .as_ref()
            .filter(|pending| {
                pending.confirmation_id == confirmation_id
                    && pending.manifest_digest == confirmation_digest
            }) {
            Some(pending) => pending.prompt_text.clone(),
            None => String::new(),
        };
        guard.begin_lock_scoped_plan_execution()?;
        match guard.prepare_confirmation_resume(
            &confirmation_id,
            &confirmation_digest,
            should_resume,
        ) {
            Ok(prepared) => (prompt_text, prepared),
            Err(outcome) => {
                guard.end_lock_scoped_plan_execution();
                return Ok(confirmation_resolution_from_outcome(
                    confirmation_id,
                    prompt_text,
                    confirmed,
                    timed_out,
                    outcome,
                ));
            }
        }
    };

    let mut runner = LockScopedStepRunner::new(
        core,
        prepared.lock_scoped_execution_token,
        prepared.listening_state,
    );
    let outcome = resume_after_confirmation_with_runner(
        &prepared.pending_plan_execution,
        &confirmation_id,
        &confirmation_digest,
        should_resume,
        &prepared.confirmation_context,
        |step| runner.run(step),
    );
    let outcome = runner.reconcile_outcome(outcome);
    let outcome = {
        let mut guard = lock_app_core(core)?;
        guard.end_lock_scoped_plan_execution();
        guard.finish_planner_execution(outcome)
    };
    Ok(confirmation_resolution_from_outcome(
        confirmation_id,
        prompt_text,
        confirmed,
        timed_out,
        outcome,
    ))
}

pub(crate) fn submit_narration_consent_response_lock_scoped(
    core: &Arc<Mutex<AppCore>>,
    challenge_id: String,
    challenge_digest: String,
    decision: RemotePlannerConsentDecision,
) -> Result<NarrationConsentResponseOutcome, ToolError> {
    let (resume, request_id, token, listening_state) = {
        let mut guard = lock_app_core(core)?;
        match guard.resolve_narration_consent(&challenge_id, &challenge_digest, decision)? {
            NarrationConsentResolution::Terminal(RemotePlannerConsentResponseOutcome::Denied) => {
                return Ok(NarrationConsentResponseOutcome::Denied);
            }
            NarrationConsentResolution::Terminal(
                RemotePlannerConsentResponseOutcome::BlockedPersistent,
            ) => return Ok(NarrationConsentResponseOutcome::BlockedPersistent),
            NarrationConsentResolution::Terminal(_) => {
                return Err(ToolError {
                    code: String::from("remote_data_consent_internal_error"),
                    message: String::from(
                        "narration consent resolution returned an unexpected terminal outcome",
                    ),
                    retryable: false,
                    details: None,
                });
            }
            NarrationConsentResolution::Authorized { resume } => {
                let request_id = guard.next_id("narration-consent-resume", &challenge_id);
                (
                    resume,
                    request_id,
                    guard.current_lock_scoped_execution_token_without_listening(),
                    guard.current_lock_scoped_listening_state(),
                )
            }
        }
    };

    let mut runner = LockScopedStepRunner::new(core, token, listening_state);
    runner.resume_narration_after_consent(resume, &request_id)?;
    Ok(NarrationConsentResponseOutcome::Spoken)
}

fn confirmation_resolution_from_outcome(
    confirmation_id: String,
    prompt_text: String,
    confirmed: bool,
    timed_out: bool,
    resume_outcome: ExecutionOutcome,
) -> ConfirmActionResolution {
    let tool_result = match &resume_outcome {
        ExecutionOutcome::Aborted { error, .. } => ToolResult::failure(
            ToolName::ConfirmAction,
            confirmation_id.clone(),
            error.clone(),
            vec![String::from(
                "Confirmation response could not be applied to the pending plan.",
            )],
        ),
        _ => ToolResult::success(
            ToolName::ConfirmAction,
            confirmation_id.clone(),
            ConfirmActionData {
                confirmation_id: confirmation_id.clone(),
                prompt_text,
                confirmed: Some(confirmed),
                timed_out,
            },
            vec![String::from(
                "Confirmation response was applied to the pending plan execution.",
            )],
        ),
    };
    ConfirmActionResolution {
        tool_result,
        resume_outcome,
    }
}

fn execution_abort(error: ToolError) -> ExecutionOutcome {
    ExecutionOutcome::Aborted {
        trace: ExecutionTrace {
            executed_step_ids: Vec::new(),
            tool_results: Vec::new(),
        },
        error,
    }
}

fn execution_trace(outcome: ExecutionOutcome) -> ExecutionTrace {
    match outcome {
        ExecutionOutcome::Complete { trace }
        | ExecutionOutcome::AwaitingConfirmation { trace, .. }
        | ExecutionOutcome::NeedsReplan { trace }
        | ExecutionOutcome::NeedsRemoteDataConsent { trace, .. }
        | ExecutionOutcome::Aborted { trace, .. } => trace,
    }
}

pub(crate) fn run_command_with_lock_scoped_replanning(
    core: &Arc<Mutex<AppCore>>,
    request_id: &str,
    transcript: &str,
) -> Result<ExecutionOutcome, ToolError> {
    let mut runtime = LockScopedReplanningRuntime::new(core);
    execute_bounded_replanning_loop(&mut runtime, request_id, transcript)
}

pub(crate) fn resolve_command_lock_scoped(
    core: &Arc<Mutex<AppCore>>,
    request_id: String,
    transcript: String,
) -> Result<ResolveCommandOutcome, ToolError> {
    match LockScopedReplanningRuntime::new(core).resolve(
        request_id,
        &transcript,
        Vec::new(),
        PendingRemotePlannerContinuation::ResolveOnly,
    )? {
        ResolvePlanOutcome::Resolved(planner_output) => {
            Ok(ResolveCommandOutcome::Resolved(planner_output))
        }
        ResolvePlanOutcome::NeedsRemoteDataConsent(challenge) => {
            Ok(ResolveCommandOutcome::NeedsRemoteDataConsent {
                needs_remote_data_consent: challenge,
            })
        }
    }
}

pub(crate) fn submit_remote_planner_consent_response_lock_scoped(
    core: &Arc<Mutex<AppCore>>,
    challenge_id: String,
    challenge_digest: String,
    decision: RemotePlannerConsentDecision,
) -> Result<RemotePlannerConsentResponseOutcome, ToolError> {
    let ready = {
        let mut guard = lock_app_core(core)?;
        guard.resolve_pending_remote_planner_consent(&challenge_id, &challenge_digest, decision)?
    };
    let ready = match ready {
        PendingConsentResolution::Terminal(outcome) => return Ok(outcome),
        PendingConsentResolution::Authorized(ready) => *ready,
    };

    let safety = ready
        .prepared
        .sanitized_input
        .trusted_runtime
        .safety
        .clone();
    let available_tools = ready
        .prepared
        .sanitized_input
        .trusted_runtime
        .available_tools
        .clone();
    let active_skill_names = ready
        .prepared
        .sanitized_input
        .trusted_runtime
        .active_skill_names
        .clone();
    let planner_output = resolve_remote_planner(&ready.prepared)?;
    validate_planner_output_with_safety(
        &planner_output,
        &available_tools,
        &active_skill_names,
        &safety,
    )?;
    {
        let mut guard = lock_app_core(core)?;
        guard.register_planning_snapshot(&planner_output, ready.planning_snapshot)?;
    }
    match ready.continuation {
        PendingRemotePlannerContinuation::ResolveOnly => {
            Ok(RemotePlannerConsentResponseOutcome::Resolved { planner_output })
        }
        PendingRemotePlannerContinuation::Execute => {
            let outcome = execute_planner_output_lock_scoped(
                core,
                format!("{}-execute-after-remote-data-consent", ready.request_id),
                &planner_output,
            );
            Ok(RemotePlannerConsentResponseOutcome::Executed {
                outcome: Box::new(outcome),
            })
        }
    }
}
