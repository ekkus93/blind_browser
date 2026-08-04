use std::sync::{Arc, Mutex};

use super::command_dispatch::PlannerResolution;
use super::remote_data_consent::{
    PendingConsentResolution, PendingRemotePlannerContinuation, RemotePlannerPreparation,
};
use super::remote_planner::resolve_remote_planner;
use super::replanning::{execute_bounded_replanning_loop, ReplanningRuntime, ResolvePlanOutcome};
use super::AppCore;
use crate::commands::{
    validate_planner_output_with_safety, ExecutionOutcome, ExecutionTrace, PlannerOutput,
    PlannerToolHistoryEntry, RemotePlannerConsentDecision, RemotePlannerConsentResponseOutcome,
    ResolveCommandOutcome, ToolError,
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
                    ResolvePhase::Direct(planner_output, planning_snapshot)
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
    let mut guard = lock_app_core(core)?;
    guard.register_planning_snapshot(&planner_output, ready.planning_snapshot)?;
    match ready.continuation {
        PendingRemotePlannerContinuation::ResolveOnly => {
            Ok(RemotePlannerConsentResponseOutcome::Resolved { planner_output })
        }
        PendingRemotePlannerContinuation::Execute => {
            let outcome = guard.execute_planner_output(
                format!("{}-execute-after-remote-data-consent", ready.request_id),
                &planner_output,
            );
            Ok(RemotePlannerConsentResponseOutcome::Executed {
                outcome: Box::new(outcome),
            })
        }
    }
}
