use super::super::{PlannedStep, SerializedToolResult, ToolError, ToolResult};
use serde::Serialize;
use std::collections::HashMap;

pub(in crate::commands::planner_executor) fn build_step_positions(
    steps: &[PlannedStep],
) -> Result<HashMap<String, usize>, ToolError> {
    let mut positions = HashMap::with_capacity(steps.len());

    for (index, step) in steps.iter().enumerate() {
        if positions.insert(step.step_id.clone(), index).is_some() {
            return Err(ToolError {
                code: String::from("duplicate_step_id"),
                message: format!("planner returned duplicate step id '{}'", step.step_id),
                retryable: false,
                details: None,
            });
        }
    }

    Ok(positions)
}

pub(in crate::commands::planner_executor) fn queued_step_ids_after(
    steps: &[PlannedStep],
    current_step: &PlannedStep,
    step_positions: &HashMap<String, usize>,
) -> Vec<String> {
    let Some(current_index) = step_positions.get(&current_step.step_id).copied() else {
        return Vec::new();
    };

    steps
        .iter()
        .skip(current_index + 1)
        .map(|step| step.step_id.clone())
        .collect()
}

pub(in crate::commands::planner_executor) fn queued_steps_after(
    steps: &[PlannedStep],
    current_step: &PlannedStep,
    step_positions: &HashMap<String, usize>,
) -> Vec<PlannedStep> {
    let Some(current_index) = step_positions.get(&current_step.step_id).copied() else {
        return Vec::new();
    };

    steps.iter().skip(current_index + 1).cloned().collect()
}

pub(in crate::commands::planner_executor) fn extract_confirmation_id(
    result: &SerializedToolResult,
) -> Result<String, ToolError> {
    let confirmation_id = result
        .data
        .as_ref()
        .and_then(|data| data.get("confirmation_id"))
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    confirmation_id.ok_or_else(|| ToolError {
        code: String::from("missing_confirmation_id"),
        message: String::from(
            "step requested confirmation but the tool result did not include confirmation_id",
        ),
        retryable: false,
        details: Some(serde_json::json!({
            "tool_name": result.tool_name,
            "request_id": result.request_id,
        })),
    })
}

pub(in crate::commands::planner_executor) fn extract_confirmation_prompt_text(
    result: &SerializedToolResult,
) -> Result<String, ToolError> {
    let prompt_text = result
        .data
        .as_ref()
        .and_then(|data| data.get("prompt_text"))
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    prompt_text.ok_or_else(|| ToolError {
        code: String::from("missing_confirmation_prompt"),
        message: String::from(
            "step requested confirmation but the tool result did not include prompt_text",
        ),
        retryable: false,
        details: Some(serde_json::json!({
            "tool_name": result.tool_name,
            "request_id": result.request_id,
        })),
    })
}

pub(in crate::commands::planner_executor) fn serialize_tool_result<T>(
    result: ToolResult<T>,
) -> SerializedToolResult
where
    T: Serialize,
{
    let ToolResult {
        ok,
        tool_name,
        request_id,
        timestamp_ms,
        data,
        error,
        warnings,
        observations,
    } = result;

    let serialized_data = match data {
        Some(data) => match serde_json::to_value(data) {
            Ok(value) => Some(value),
            Err(error) => {
                return ToolResult::failure(
                    tool_name,
                    request_id,
                    ToolError {
                        code: String::from("tool_result_serialization_failed"),
                        message: format!("failed to serialize tool result payload: {error}"),
                        retryable: false,
                        details: None,
                    },
                    vec![String::from(
                        "Executor could not serialize the tool result payload.",
                    )],
                );
            }
        },
        None => None,
    };

    ToolResult {
        ok,
        tool_name,
        request_id,
        timestamp_ms,
        data: serialized_data,
        error,
        warnings,
        observations,
    }
}

pub(in crate::commands::planner_executor) fn inferred_request_id(step: &PlannedStep) -> String {
    step.arguments
        .get("request_id")
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| step.step_id.clone())
}
