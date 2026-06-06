use std::collections::HashSet;

use super::invalid_planner_output;
use crate::commands::{ConfirmActionInput, ReportResultInput, StepTransition, ToolError};

pub(crate) fn validate_confirm_action_input(
    input: &ConfirmActionInput,
) -> Result<(), ToolError> {
    if input.prompt_text.trim().is_empty() {
        return Err(invalid_planner_output(
            "confirm_action requires a non-empty prompt_text",
            None,
        ));
    }

    if input.reason.trim().is_empty() {
        return Err(invalid_planner_output(
            "confirm_action requires a non-empty reason",
            None,
        ));
    }

    Ok(())
}

pub(super) fn validate_report_result_input(
    input: &ReportResultInput,
) -> Result<(), ToolError> {
    if input.summary.trim().is_empty() {
        return Err(invalid_planner_output(
            "report_result requires a non-empty summary",
            None,
        ));
    }

    for (field_name, value) in [
        (
            "next_recommended_action",
            input.next_recommended_action.as_deref(),
        ),
        ("user_message", input.user_message.as_deref()),
    ] {
        if let Some(value) = value {
            if value.trim().is_empty() {
                return Err(invalid_planner_output(
                    format!("report_result {field_name} must be non-empty when provided"),
                    None,
                ));
            }
        }
    }

    Ok(())
}

pub(super) fn validate_step_transition(
    transition: &StepTransition,
    step_ids: &HashSet<String>,
    source_step_id: &str,
) -> Result<(), ToolError> {
    if let StepTransition::NextStep { step_id } = transition {
        if !step_ids.contains(step_id) {
            return Err(invalid_planner_output(
                format!(
                    "planner referenced missing next step '{}' from '{}'",
                    step_id, source_step_id
                ),
                Some(serde_json::json!({
                    "source_step_id": source_step_id,
                    "next_step_id": step_id,
                })),
            ));
        }
    }

    Ok(())
}
