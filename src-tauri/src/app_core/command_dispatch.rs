use tauri::Manager;

use super::fill_correction::resolve_recent_fill_correction_command;
use super::form_fill::{
    resolve_direct_fill_command_internal, resolve_direct_focus_field_command,
    resolve_direct_submit_form_command,
};
use crate::commands::{
    build_planner_skill_selection, execute_planner_output_with_runtime_safety,
    planner_available_tools, resolve_direct_audio_command,
    resolve_direct_browser_visibility_command, resolve_direct_navigation_readback_command,
    resolve_direct_open_url_command, resolve_direct_read_page_command,
    resolve_direct_read_title_command, resolve_direct_repeat_command,
    resolve_direct_status_query_command, resolve_direct_voice_input_command,
    validate_planner_output_with_safety, AvailableTool, ExecutionOutcome, PlannerInput,
    PlannerOutput, PlannerSafetySettings, PlannerToolHistoryEntry, SerializedToolResult, ToolError,
    ToolName,
};
use crate::config::{RemotePlannerPrivacySettings, RemotePlannerProfile};

/// Outcome of the deterministic resolution phase, which runs under the `AppCore`
/// lock. A `Direct` match is already validated and needs no network. A `Remote`
/// result carries everything the (unlocked) LLM round-trip needs, so the lock can
/// be released before [`super::remote_planner::resolve_remote_planner`] runs.
pub(crate) enum PlannerResolution {
    Direct(PlannerOutput),
    Remote {
        // Boxed: `PlannerInput` is large, so an unboxed variant bloats every
        // `PlannerResolution` (clippy `large_enum_variant`).
        planner_input: Box<PlannerInput>,
        profile_name: String,
        profile: RemotePlannerProfile,
        privacy: RemotePlannerPrivacySettings,
        available_tools: Vec<AvailableTool>,
        active_skill_names: Vec<String>,
    },
}

impl super::AppCore {
    pub fn execute_planner_output(
        &mut self,
        request_id: String,
        planner_output: &PlannerOutput,
    ) -> ExecutionOutcome {
        if let Err(error) = self.validate_and_consume_planning_snapshot(planner_output) {
            let outcome = planner_snapshot_validation_outcome(error);
            self.state.apply_execution_outcome(&outcome);
            return outcome;
        }
        let prepared = match self.prepare_planner_output_for_execution(planner_output) {
            Ok(prepared) => prepared,
            Err(error) => {
                let outcome = planner_execution_abort(error);
                self.state.apply_execution_outcome(&outcome);
                return outcome;
            }
        };
        let safety = PlannerSafetySettings::from(&self.config.safety);
        let mut outcome =
            execute_planner_output_with_runtime_safety(self, request_id, &prepared, &safety);
        self.state.apply_execution_outcome(&outcome);
        if let ExecutionOutcome::AwaitingConfirmation {
            pending_plan_execution,
            ..
        } = &mut outcome
        {
            let runtime_state_token = self.current_runtime_state_token();
            pending_plan_execution.runtime_state_token = runtime_state_token.clone();
            if let Some(stored_pending) = self.state.pending_plan_execution.as_mut() {
                stored_pending.runtime_state_token = runtime_state_token;
            }
        }
        outcome
    }

    /// Deterministic resolution phase: try every direct-command resolver, and if
    /// none match, assemble the `PlannerInput` and snapshot the remote planner
    /// profile for an unlocked LLM round-trip. Runs under the `AppCore` lock; the
    /// caller (the lock-scoped replanning orchestrator) drops the guard before the
    /// network step. Direct results are already validated.
    pub(crate) fn build_planner_resolution(
        &mut self,
        request_id: String,
        transcript: &str,
        recent_tool_results: Vec<PlannerToolHistoryEntry>,
    ) -> Result<PlannerResolution, ToolError> {
        let transcript = transcript.trim();
        if transcript.is_empty() {
            return Err(ToolError {
                code: String::from("empty_transcript"),
                message: String::from("resolve_command requires a non-empty transcript"),
                retryable: false,
                details: None,
            });
        }

        let available_tools = planner_available_tools();
        let planner_safety = PlannerSafetySettings::from(&self.config.safety);
        let current_dir = std::env::current_dir().ok();
        let user_skill_root = self
            .app_handle
            .path()
            .app_config_dir()
            .ok()
            .map(|path| path.join("skills"));
        let skill_selection = build_planner_skill_selection(
            current_dir.as_deref(),
            user_skill_root.as_deref(),
            transcript,
            &available_tools,
        );

        if let Some(planner_output) = resolve_direct_browser_visibility_command(
            transcript,
            &request_id,
            self.state.browser_visibility,
            &skill_selection.active_skill_names,
        ) {
            validate_planner_output_with_safety(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
                &planner_safety,
            )?;
            return Ok(PlannerResolution::Direct(planner_output));
        }

        let current_agent_state = self.current_agent_state_snapshot(true);

        if let Some(planner_output) = resolve_direct_navigation_readback_command(
            transcript,
            &request_id,
            &skill_selection.active_skill_names,
        ) {
            validate_planner_output_with_safety(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
                &planner_safety,
            )?;
            return Ok(PlannerResolution::Direct(planner_output));
        }

        if let Some(planner_output) = resolve_direct_voice_input_command(
            transcript,
            &request_id,
            &skill_selection.active_skill_names,
        ) {
            validate_planner_output_with_safety(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
                &planner_safety,
            )?;
            return Ok(PlannerResolution::Direct(planner_output));
        }

        if let Some(planner_output) = resolve_direct_open_url_command(
            transcript,
            &request_id,
            &skill_selection.active_skill_names,
        ) {
            validate_planner_output_with_safety(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
                &planner_safety,
            )?;
            return Ok(PlannerResolution::Direct(planner_output));
        }

        if let Some(planner_output) = resolve_direct_read_page_command(
            transcript,
            &request_id,
            self.state.current_page.as_ref(),
            &current_agent_state,
            &skill_selection.active_skill_names,
        ) {
            validate_planner_output_with_safety(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
                &planner_safety,
            )?;
            return Ok(PlannerResolution::Direct(planner_output));
        }

        if let Some((planner_output, next_recent_field_context)) =
            resolve_recent_fill_correction_command(
                transcript,
                &request_id,
                self.state.current_page_id.as_deref(),
                self.state.current_page.as_ref(),
                &skill_selection.active_skill_names,
                self.recent_field_context.as_ref(),
            )
        {
            self.recent_field_context = next_recent_field_context;
            validate_planner_output_with_safety(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
                &planner_safety,
            )?;
            return Ok(PlannerResolution::Direct(planner_output));
        }

        if let Some(resolved) = resolve_direct_fill_command_internal(
            transcript,
            &request_id,
            self.state.current_page_id.as_deref(),
            self.state.current_page.as_ref(),
            &skill_selection.active_skill_names,
            self.config.safety.confirmation_confidence_threshold,
            true,
        ) {
            self.store_recent_field_context(resolved.recent_field_context);
            let planner_output = resolved.planner_output;
            validate_planner_output_with_safety(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
                &planner_safety,
            )?;
            return Ok(PlannerResolution::Direct(planner_output));
        }

        if let Some(resolved) = resolve_direct_fill_command_internal(
            transcript,
            &request_id,
            self.state.current_page_id.as_deref(),
            self.state.current_page.as_ref(),
            &skill_selection.active_skill_names,
            self.config.safety.confirmation_confidence_threshold,
            false,
        ) {
            self.store_recent_field_context(resolved.recent_field_context);
            let planner_output = resolved.planner_output;
            validate_planner_output_with_safety(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
                &planner_safety,
            )?;
            return Ok(PlannerResolution::Direct(planner_output));
        }

        if let Some(planner_output) = resolve_direct_submit_form_command(
            transcript,
            &request_id,
            self.state.current_page.as_ref(),
            &skill_selection.active_skill_names,
        ) {
            validate_planner_output_with_safety(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
                &planner_safety,
            )?;
            return Ok(PlannerResolution::Direct(planner_output));
        }

        if let Some(planner_output) = resolve_direct_focus_field_command(
            transcript,
            &request_id,
            self.state.current_page.as_ref(),
            &skill_selection.active_skill_names,
            self.config.safety.confirmation_confidence_threshold,
        ) {
            validate_planner_output_with_safety(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
                &planner_safety,
            )?;
            return Ok(PlannerResolution::Direct(planner_output));
        }

        if let Some(planner_output) = resolve_direct_repeat_command(
            transcript,
            &request_id,
            &current_agent_state,
            &skill_selection.active_skill_names,
        ) {
            validate_planner_output_with_safety(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
                &planner_safety,
            )?;
            return Ok(PlannerResolution::Direct(planner_output));
        }

        if let Some(planner_output) = resolve_direct_read_title_command(
            transcript,
            &request_id,
            &current_agent_state,
            &skill_selection.active_skill_names,
        ) {
            validate_planner_output_with_safety(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
                &planner_safety,
            )?;
            return Ok(PlannerResolution::Direct(planner_output));
        }

        let current_runtime_status = self.current_runtime_status_snapshot(false);

        if let Some(planner_output) = resolve_direct_status_query_command(
            transcript,
            &request_id,
            &current_agent_state,
            &current_runtime_status,
            &skill_selection.active_skill_names,
        ) {
            validate_planner_output_with_safety(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
                &planner_safety,
            )?;
            return Ok(PlannerResolution::Direct(planner_output));
        }

        if let Some(planner_output) = resolve_direct_audio_command(
            transcript,
            &request_id,
            self.state.audio.playback_volume,
            self.state.audio.playback_speed,
            &skill_selection.active_skill_names,
        ) {
            validate_planner_output_with_safety(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
                &planner_safety,
            )?;
            return Ok(PlannerResolution::Direct(planner_output));
        }

        // No direct command matched: snapshot the remote planner profile under the
        // lock so the LLM round-trip can run with the guard released.
        let (profile_name, profile) = self.remote_planner_profile_snapshot()?;
        let privacy = self.config.remote_planner_privacy.clone();

        let planner_input = PlannerInput {
            request_id: request_id.clone(),
            runtime_state_token: self.current_runtime_state_token(),
            transcript: transcript.to_string(),
            agent_state: current_agent_state,
            safety: (&self.config.safety).into(),
            available_tools: available_tools.clone(),
            active_skill_names: skill_selection.active_skill_names.clone(),
            relevant_skill_summaries: skill_selection.relevant_skill_summaries.clone(),
            page_snapshot: self.current_page_snapshot(Some(1_200), true)?,
            page_model: self.state.current_page.clone(),
            recent_tool_results,
        };

        Ok(PlannerResolution::Remote {
            active_skill_names: planner_input.active_skill_names.clone(),
            planner_input: Box::new(planner_input),
            profile_name,
            profile,
            privacy,
            available_tools,
        })
    }
}

fn planner_snapshot_validation_outcome(error: ToolError) -> ExecutionOutcome {
    if matches!(
        error.code.as_str(),
        "missing_planning_snapshot" | "planning_snapshot_expired" | "stale_planning_snapshot"
    ) {
        return ExecutionOutcome::NeedsReplan {
            trace: crate::commands::ExecutionTrace {
                executed_step_ids: Vec::new(),
                tool_results: vec![SerializedToolResult::failure(
                    ToolName::GetAgentState,
                    String::from("runtime-state-revalidation"),
                    error,
                    vec![String::from(
                        "Runtime state changed after planning; bounded replanning is required.",
                    )],
                )],
            },
        };
    }
    planner_execution_abort(error)
}

fn planner_execution_abort(error: ToolError) -> ExecutionOutcome {
    ExecutionOutcome::Aborted {
        trace: crate::commands::ExecutionTrace {
            executed_step_ids: Vec::new(),
            tool_results: Vec::new(),
        },
        error,
    }
}

#[cfg(test)]
mod planning_snapshot_outcome_tests {
    use super::*;

    fn snapshot_error(code: &str) -> ToolError {
        ToolError {
            code: code.to_string(),
            message: String::from("runtime planning snapshot rejected"),
            retryable: false,
            details: None,
        }
    }

    #[test]
    fn stale_snapshot_failures_request_bounded_replanning() {
        for code in [
            "missing_planning_snapshot",
            "planning_snapshot_expired",
            "stale_planning_snapshot",
        ] {
            assert!(matches!(
                planner_snapshot_validation_outcome(snapshot_error(code)),
                ExecutionOutcome::NeedsReplan { .. }
            ));
        }
    }

    #[test]
    fn non_snapshot_validation_failures_remain_terminal() {
        assert!(matches!(
            planner_snapshot_validation_outcome(snapshot_error("invalid_planner_output")),
            ExecutionOutcome::Aborted { .. }
        ));
    }
}
