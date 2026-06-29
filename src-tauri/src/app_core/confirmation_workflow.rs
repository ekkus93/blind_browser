use crate::commands::{
    resume_after_confirmation, ConfirmActionData, ConfirmActionInput, ConfirmActionResolution,
    ExecutionOutcome, ExecutionTrace, ToolError, ToolName, ToolResult,
};

impl super::AppCore {
    pub fn resume_after_confirmation(
        &mut self,
        confirmation_id: &str,
        confirmed: bool,
    ) -> ExecutionOutcome {
        let Some(pending_plan_execution) = self.state.pending_plan_execution.clone() else {
            return ExecutionOutcome::Aborted {
                trace: ExecutionTrace {
                    executed_step_ids: Vec::new(),
                    tool_results: Vec::new(),
                },
                error: ToolError {
                    code: String::from("missing_pending_execution"),
                    message: String::from(
                        "there is no pending plan execution to resume for confirmation",
                    ),
                    retryable: false,
                    details: None,
                },
            };
        };

        if self.state.pending_confirmation_id.as_deref() != Some(confirmation_id) {
            return ExecutionOutcome::Aborted {
                trace: ExecutionTrace {
                    executed_step_ids: Vec::new(),
                    tool_results: Vec::new(),
                },
                error: ToolError {
                    code: String::from("confirmation_id_mismatch"),
                    message: String::from(
                        "confirmation response did not match the stored pending confirmation id",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({
                        "expected_confirmation_id": self.state.pending_confirmation_id,
                        "received_confirmation_id": confirmation_id,
                    })),
                },
            };
        }

        let outcome =
            resume_after_confirmation(self, &pending_plan_execution, confirmation_id, confirmed);
        self.state.apply_execution_outcome(&outcome);
        outcome
    }

    pub fn submit_confirmation_response(
        &mut self,
        confirmation_id: &str,
        confirmed: bool,
        timed_out: bool,
    ) -> ConfirmActionResolution {
        let prompt_text = self
            .state
            .pending_plan_execution
            .as_ref()
            .filter(|pending| pending.confirmation_id == confirmation_id)
            .map(|pending| pending.prompt_text.clone())
            .unwrap_or_default();

        let should_resume = confirmed && !timed_out;
        let resume_outcome = self.resume_after_confirmation(confirmation_id, should_resume);
        let tool_result = match &resume_outcome {
            ExecutionOutcome::Aborted { error, .. } => ToolResult::failure(
                ToolName::ConfirmAction,
                confirmation_id.to_string(),
                error.clone(),
                vec![String::from(
                    "Confirmation response could not be applied to the pending plan.",
                )],
            ),
            _ => ToolResult::success(
                ToolName::ConfirmAction,
                confirmation_id.to_string(),
                ConfirmActionData {
                    confirmation_id: confirmation_id.to_string(),
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

    pub fn execute_confirm_action(
        &mut self,
        input: ConfirmActionInput,
    ) -> ToolResult<ConfirmActionData> {
        let prompt_text = input.prompt_text.trim().to_string();
        if prompt_text.is_empty() {
            return ToolResult::failure(
                ToolName::ConfirmAction,
                input.request_id,
                ToolError {
                    code: String::from("invalid_confirmation_prompt"),
                    message: String::from("confirm_action requires a non-empty prompt_text"),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Confirmation request was rejected because the prompt text was empty.",
                )],
            );
        }

        let mut observations = vec![String::from("Prepared a confirmation prompt for the user.")];
        let reason = input.reason.trim();
        if !reason.is_empty() {
            observations.push(reason.to_string());
        }

        ToolResult::success(
            ToolName::ConfirmAction,
            input.request_id.clone(),
            ConfirmActionData {
                confirmation_id: self.next_confirmation_id(&input.request_id),
                prompt_text,
                confirmed: None,
                timed_out: false,
            },
            observations,
        )
    }
}
