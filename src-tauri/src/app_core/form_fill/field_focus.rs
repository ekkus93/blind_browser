use crate::commands::{
    ElementVisibilityFilter, FindElementInput, IntentName, IntentSummary, PlannedStep,
    PlannerOutput, PlannerStatus, StepTransition, ToolName,
    parse_direct_focus_field_command, DEFAULT_FIND_ELEMENT_MAX_CANDIDATES,
};
use crate::page_model::PageModel;
use crate::app_core::element_scoring::{
    build_find_element_query, determine_find_element_resolution, focusable_field_elements,
    rank_find_element_candidates, summarize_candidate_names,
};
use crate::app_core::fill_correction::{build_direct_follow_up_output, DirectFollowUpSpec};

pub(crate) fn resolve_direct_focus_field_command(
    transcript: &str,
    request_id: &str,
    current_page: Option<&PageModel>,
    active_skill_names: &[String],
    confirmation_confidence_threshold: f32,
) -> Option<PlannerOutput> {
    let command = parse_direct_focus_field_command(transcript)?;
    let selected_skills = if active_skill_names
        .iter()
        .any(|active_name| active_name == "focus_field")
    {
        vec![String::from("focus_field")]
    } else {
        Vec::new()
    };

    let Some(description) = command.description else {
        let summary = String::from("Please tell me which field to focus.");
        return Some(build_direct_follow_up_output(
            request_id,
            DirectFollowUpSpec {
                intent_name: IntentName::FillInput,
                goal: String::from("Focus the requested field."),
                target_description: Some(String::from("field focus target")),
                selected_skills,
                summary,
                next_recommended_action: Some(String::from(
                    "Say the field name, like focus the email field.",
                )),
                step_id: String::from("report-missing-focus-field-description"),
                purpose: String::from("Report that the field name is required before focusing."),
            },
        ));
    };

    let Some(current_page) = current_page else {
        let summary = String::from("There is no current page to focus a field on yet.");
        return Some(build_direct_follow_up_output(
            request_id,
            DirectFollowUpSpec {
                intent_name: IntentName::FillInput,
                goal: String::from("Focus the requested field."),
                target_description: Some(String::from("current page field")),
                selected_skills,
                summary,
                next_recommended_action: Some(String::from(
                    "Open a page first, then ask me to focus a field.",
                )),
                step_id: String::from("report-missing-focus-page"),
                purpose: String::from(
                    "Report that there is no active page available for field focus.",
                ),
            },
        ));
    };

    let field_elements = focusable_field_elements(current_page);
    if field_elements.is_empty() {
        let summary = String::from("I could not find any focusable fields on the current page.");
        return Some(build_direct_follow_up_output(
            request_id,
            DirectFollowUpSpec {
                intent_name: IntentName::FillInput,
                goal: String::from("Focus the requested field."),
                target_description: Some(description.clone()),
                selected_skills,
                summary,
                next_recommended_action: Some(String::from(
                    "Try again after the page finishes loading or becomes interactive.",
                )),
                step_id: String::from("report-missing-focusable-fields"),
                purpose: String::from(
                    "Report that no focusable fields are available on the current page.",
                ),
            },
        ));
    }

    let query = FindElementInput {
        request_id: request_id.to_string(),
        timeout_ms: None,
        description: description.clone(),
        text: None,
        role: None,
        color_hint: None,
        nearby_text: None,
        selector_hint: None,
        visibility_filter: ElementVisibilityFilter::VisibleOnly,
        max_candidates: Some(DEFAULT_FIND_ELEMENT_MAX_CANDIDATES),
    };
    let search_query = build_find_element_query(&query).ok()?;
    let candidates = rank_find_element_candidates(
        &field_elements,
        &search_query,
        DEFAULT_FIND_ELEMENT_MAX_CANDIDATES,
    );
    let (chosen_element_id, _, requires_confirmation) = if candidates.len() == 1 {
        (Some(candidates[0].element_id.clone()), None, false)
    } else {
        determine_find_element_resolution(&candidates, confirmation_confidence_threshold)
    };

    if let Some(element_id) = chosen_element_id {
        return Some(PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::FillInput,
                goal: String::from("Focus the requested field."),
                target_description: Some(description),
            },
            selected_skills,
            steps: vec![PlannedStep {
                step_id: String::from("focus-field"),
                tool_name: ToolName::FocusElement,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "element_id": element_id
                }),
                purpose: String::from("Move focus to the requested field."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        });
    }

    let summary = if requires_confirmation {
        let candidate_names = summarize_candidate_names(current_page, &candidates);
        if candidate_names.is_empty() {
            format!("I found multiple possible fields for {description}. Please be more specific.")
        } else {
            format!(
                "I found multiple possible fields for {description}: {}. Please be more specific.",
                candidate_names.join(", ")
            )
        }
    } else {
        format!("I could not find a visible field matching {description}.")
    };

    Some(build_direct_follow_up_output(
        request_id,
        DirectFollowUpSpec {
            intent_name: IntentName::FillInput,
            goal: String::from("Focus the requested field."),
            target_description: Some(description),
            selected_skills,
            summary,
            next_recommended_action: Some(String::from(
                "Try naming the field label or placeholder more specifically.",
            )),
            step_id: String::from("report-focus-field-follow-up"),
            purpose: String::from(
                "Report that the requested field could not be focused deterministically.",
            ),
        },
    ))
}
