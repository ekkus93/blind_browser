use crate::commands::{
    resume_after_confirmation_with_context, ConfirmActionData, ConfirmActionInput,
    ConfirmActionResolution, ConfirmationRuntimeContext, ExecutionOutcome, ExecutionTrace,
    ToolError, ToolName, ToolResult,
};

impl super::AppCore {
    pub fn resume_after_confirmation(
        &mut self,
        confirmation_id: &str,
        confirmation_digest: &str,
        confirmed: bool,
    ) -> ExecutionOutcome {
        let Some(pending_plan_execution) = self.state.pending_plan_execution.clone() else {
            return confirmation_abort(
                "missing_pending_execution",
                "there is no pending plan execution to resume for confirmation",
                None,
            );
        };

        if self.state.pending_confirmation_id.as_deref() != Some(confirmation_id)
            || pending_plan_execution.confirmation_id != confirmation_id
        {
            return confirmation_abort(
                "confirmation_id_mismatch",
                "confirmation response did not match the stored pending confirmation id",
                Some(serde_json::json!({
                    "expected_confirmation_id": self.state.pending_confirmation_id,
                    "received_confirmation_id": confirmation_id,
                })),
            );
        }

        if pending_plan_execution.manifest_digest != confirmation_digest {
            return confirmation_abort(
                "confirmation_digest_mismatch",
                "confirmation response did not match the stored pending action manifest",
                Some(serde_json::json!({
                    "received_digest_present": !confirmation_digest.trim().is_empty(),
                })),
            );
        }

        let confirmation_context = ConfirmationRuntimeContext::current(
            self.state.confirmation_page_identity(),
            self.state
                .current_page
                .as_ref()
                .and_then(|page| page.url.clone()),
        );

        if confirmed {
            if let Err(error) =
                self.preflight_pending_click_authorizations(&pending_plan_execution.queued_steps)
            {
                return ExecutionOutcome::Aborted {
                    trace: ExecutionTrace {
                        executed_step_ids: Vec::new(),
                        tool_results: Vec::new(),
                    },
                    error,
                };
            }

            let observed_runtime_state_token = self.current_runtime_state_token();
            if pending_plan_execution.runtime_state_token.is_empty()
                || pending_plan_execution.runtime_state_token != observed_runtime_state_token
            {
                let expected_runtime_state_token =
                    pending_plan_execution.runtime_state_token.clone();
                self.state.clear_pending_execution();
                return confirmation_abort(
                    "stale_confirmation_runtime_state",
                    "runtime or relevant configuration state changed while confirmation was pending",
                    Some(serde_json::json!({
                        "expected_runtime_state_token": expected_runtime_state_token,
                        "observed_runtime_state_token": observed_runtime_state_token,
                    })),
                );
            }
        }

        // Matching challenges are consumed before dispatch so duplicate responses,
        // re-entrant UI events, and retries cannot execute the protected action twice.
        self.state.clear_pending_execution();
        let outcome = resume_after_confirmation_with_context(
            self,
            &pending_plan_execution,
            confirmation_id,
            confirmation_digest,
            confirmed,
            &confirmation_context,
        );
        self.state.apply_execution_outcome(&outcome);
        outcome
    }

    pub fn submit_confirmation_response(
        &mut self,
        confirmation_id: &str,
        confirmation_digest: &str,
        confirmed: bool,
        timed_out: bool,
    ) -> ConfirmActionResolution {
        let prompt_text = self
            .state
            .pending_plan_execution
            .as_ref()
            .filter(|pending| {
                pending.confirmation_id == confirmation_id
                    && pending.manifest_digest == confirmation_digest
            })
            .map(|pending| pending.prompt_text.clone())
            .unwrap_or_default();

        let should_resume = confirmed && !timed_out;
        let resume_outcome =
            self.resume_after_confirmation(confirmation_id, confirmation_digest, should_resume);
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
        if input.prompt_text.trim().is_empty() {
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

        ToolResult::success(
            ToolName::ConfirmAction,
            input.request_id.clone(),
            ConfirmActionData {
                confirmation_id: self.next_confirmation_id(&input.request_id),
                // Planner-authored wording is deliberately not surfaced. The
                // executor replaces this placeholder with its deterministic
                // action-manifest summary before returning the challenge.
                prompt_text: String::from("Protected action confirmation pending."),
                confirmed: None,
                timed_out: false,
            },
            vec![String::from(
                "Prepared a deterministic confirmation challenge for protected actions.",
            )],
        )
    }
}

fn confirmation_abort(
    code: &str,
    message: &str,
    details: Option<serde_json::Value>,
) -> ExecutionOutcome {
    ExecutionOutcome::Aborted {
        trace: ExecutionTrace {
            executed_step_ids: Vec::new(),
            tool_results: Vec::new(),
        },
        error: ToolError {
            code: code.to_string(),
            message: message.to_string(),
            retryable: false,
            details,
        },
    }
}
