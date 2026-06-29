use super::invalid_planner_output;
use crate::commands::{
    EvalJsInput, GoBackInput, GoForwardInput, OpenUrlInput, ScrollPageInput, ToolError,
    MAX_HISTORY_STEPS,
};

pub(super) fn validate_open_url_input(input: &OpenUrlInput) -> Result<(), ToolError> {
    // Fail closed to http/https via the same shared policy the runtime uses, so a
    // dangerous scheme (file:, javascript:, data:, chrome:, about:, …) is rejected
    // before the plan ever reaches browser execution.
    crate::url_policy::normalize_browser_navigation_url(&input.url)
        .map_err(|error| invalid_planner_output(error.user_message(), Some(error.details())))?;
    Ok(())
}

pub(super) fn validate_go_back_input(input: &GoBackInput) -> Result<(), ToolError> {
    validate_history_steps("go_back", input.steps)
}

pub(super) fn validate_go_forward_input(input: &GoForwardInput) -> Result<(), ToolError> {
    validate_history_steps("go_forward", input.steps)
}

pub(super) fn validate_eval_js_input(input: &EvalJsInput) -> Result<(), ToolError> {
    let trimmed = input.expression.trim();
    if trimmed.is_empty() {
        return Err(invalid_planner_output(
            "eval_js requires a non-empty expression",
            None,
        ));
    }

    Ok(())
}

fn validate_history_steps(tool_name: &str, steps: Option<u8>) -> Result<(), ToolError> {
    if matches!(steps, Some(0)) {
        return Err(invalid_planner_output(
            format!("{tool_name} steps must be greater than 0 when provided"),
            None,
        ));
    }

    if let Some(steps) = steps {
        if steps > MAX_HISTORY_STEPS {
            return Err(invalid_planner_output(
                format!(
                    "{tool_name} steps must be less than or equal to {MAX_HISTORY_STEPS} when provided"
                ),
                Some(serde_json::json!({ "steps": steps })),
            ));
        }
    }

    Ok(())
}

pub(super) fn validate_scroll_page_input(input: &ScrollPageInput) -> Result<(), ToolError> {
    if input.amount_px.is_none() && input.target.is_none() {
        return Err(invalid_planner_output(
            "scroll_page requires amount_px or target to be provided",
            None,
        ));
    }

    if let Some(amount_px) = input.amount_px {
        if !amount_px.is_finite() {
            return Err(invalid_planner_output(
                "scroll_page amount_px must be a finite number when provided",
                None,
            ));
        }

        if amount_px <= 0.0 {
            return Err(invalid_planner_output(
                "scroll_page amount_px must be greater than 0 when provided",
                Some(serde_json::json!({ "amount_px": amount_px })),
            ));
        }
    }

    Ok(())
}
