use super::invalid_planner_output;
use crate::commands::{
    ClickElementInput, FindElementInput, FocusElementInput, SubmitActiveFormInput,
    TypeIntoElementInput, ToolError, DEFAULT_FIND_ELEMENT_MAX_CANDIDATES,
};

pub(super) fn validate_find_element_input(input: &FindElementInput) -> Result<(), ToolError> {
    if input.description.trim().is_empty() {
        return Err(invalid_planner_output(
            "find_element requires a non-empty description",
            None,
        ));
    }

    for (field_name, value) in [
        ("text", input.text.as_deref()),
        ("color_hint", input.color_hint.as_deref()),
        ("nearby_text", input.nearby_text.as_deref()),
        ("selector_hint", input.selector_hint.as_deref()),
    ] {
        if let Some(value) = value {
            if value.trim().is_empty() {
                return Err(invalid_planner_output(
                    format!("find_element {field_name} must be non-empty when provided"),
                    None,
                ));
            }
        }
    }

    if matches!(input.max_candidates, Some(0)) {
        return Err(invalid_planner_output(
            "find_element max_candidates must be greater than 0 when provided",
            None,
        ));
    }

    if let Some(max_candidates) = input.max_candidates {
        if max_candidates > DEFAULT_FIND_ELEMENT_MAX_CANDIDATES {
            return Err(invalid_planner_output(
                format!(
                    "find_element max_candidates must be less than or equal to {DEFAULT_FIND_ELEMENT_MAX_CANDIDATES} when provided"
                ),
                Some(serde_json::json!({ "max_candidates": max_candidates })),
            ));
        }
    }

    Ok(())
}

pub(super) fn validate_click_element_input(input: &ClickElementInput) -> Result<(), ToolError> {
    if input.element_id.trim().is_empty() {
        return Err(invalid_planner_output(
            "click_element requires a non-empty element_id",
            None,
        ));
    }

    Ok(())
}

pub(super) fn validate_focus_element_input(input: &FocusElementInput) -> Result<(), ToolError> {
    if input.element_id.trim().is_empty() {
        return Err(invalid_planner_output(
            "focus_element requires a non-empty element_id",
            None,
        ));
    }

    Ok(())
}

pub(super) fn validate_type_into_element_input(
    input: &TypeIntoElementInput,
) -> Result<(), ToolError> {
    if input.element_id.trim().is_empty() {
        return Err(invalid_planner_output(
            "type_into_element requires a non-empty element_id",
            None,
        ));
    }

    Ok(())
}

pub(super) fn validate_submit_active_form_input(
    input: &SubmitActiveFormInput,
) -> Result<(), ToolError> {
    if let Some(form_element_id) = input.form_element_id.as_deref() {
        if form_element_id.trim().is_empty() {
            return Err(invalid_planner_output(
                "submit_active_form form_element_id must be non-empty when provided",
                None,
            ));
        }
    }

    Ok(())
}
