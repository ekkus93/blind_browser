use super::element_scoring::normalize_optional_text;
use super::narration::{narration_consent_required_error, NarrationAttempt};
use crate::commands::{
    AgentStateData, GetAgentStateInput, GetRuntimeStatusData, GetRuntimeStatusInput,
    ReportResultData, ReportResultInput, ToolError, ToolName, ToolResult,
};

impl super::AppCore {
    pub fn execute_get_agent_state(
        &mut self,
        input: GetAgentStateInput,
    ) -> ToolResult<AgentStateData> {
        self.sync_narration_playback_state();
        ToolResult::success(
            ToolName::GetAgentState,
            input.request_id,
            self.current_agent_state_snapshot(input.include_last_transcript),
            vec![String::from("Read the current agent state.")],
        )
    }

    pub fn execute_get_runtime_status(
        &mut self,
        input: GetRuntimeStatusInput,
    ) -> ToolResult<GetRuntimeStatusData> {
        self.sync_narration_playback_state();
        ToolResult::success(
            ToolName::GetRuntimeStatus,
            input.request_id,
            self.current_runtime_status_snapshot(input.include_provider_modes),
            vec![String::from("Read the current runtime status.")],
        )
    }

    pub fn execute_report_result(
        &mut self,
        input: ReportResultInput,
    ) -> ToolResult<ReportResultData> {
        let summary = input.summary.trim().to_string();
        if summary.is_empty() {
            return ToolResult::failure(
                ToolName::ReportResult,
                input.request_id,
                ToolError {
                    code: String::from("invalid_report_summary"),
                    message: String::from("report_result requires a non-empty summary"),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Final result reporting was rejected because the summary was empty.",
                )],
            );
        }

        let next_recommended_action = normalize_optional_text(input.next_recommended_action);
        let user_message = normalize_optional_text(input.user_message);
        let spoken_message = user_message.clone().unwrap_or_else(|| summary.clone());

        match self.begin_feedback_narration(&spoken_message, &input.request_id) {
            Ok(NarrationAttempt::Completed(())) => {}
            Ok(NarrationAttempt::ConsentRequired(challenge)) => {
                return ToolResult::failure(
                    ToolName::ReportResult,
                    input.request_id,
                    narration_consent_required_error(&challenge),
                    vec![String::from(
                        "Spoken feedback was paused because sending this page's text to remote narration requires your permission first.",
                    )],
                );
            }
            Err(error) => {
                return ToolResult::failure(
                    ToolName::ReportResult,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Final result reporting could not start audible feedback with the configured TTS backend.",
                    )],
                );
            }
        }

        ToolResult::success(
            ToolName::ReportResult,
            input.request_id,
            ReportResultData {
                status: input.status,
                summary,
                next_recommended_action,
                user_message,
            },
            vec![
                String::from(
                    "Reported the final planner result in a structured deterministic payload.",
                ),
                String::from("Started spoken feedback for the reported result summary."),
            ],
        )
    }
}
