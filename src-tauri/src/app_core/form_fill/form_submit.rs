use crate::commands::{
    IntentName, IntentSummary, PlannedStep, PlannerOutput, PlannerStatus,
    StepTransition, ToolName, is_direct_submit_form_command,
};
use crate::page_model::PageModel;
use crate::app_core::element_scoring::{
    describe_form_element, submittable_form_elements, summarize_form_candidate_names,
};
use crate::app_core::fill_correction::{build_direct_follow_up_output, DirectFollowUpSpec};

pub(crate) fn resolve_direct_submit_form_command(
    transcript: &str,
    request_id: &str,
    current_page: Option<&PageModel>,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    if !is_direct_submit_form_command(transcript) {
        return None;
    }

    let selected_skills = if active_skill_names
        .iter()
        .any(|active_name| active_name == "submit_form")
    {
        vec![String::from("submit_form")]
    } else {
        Vec::new()
    };

    let Some(current_page) = current_page else {
        return Some(build_direct_follow_up_output(
            request_id,
            DirectFollowUpSpec {
                intent_name: IntentName::SubmitForm,
                goal: String::from("Submit the active form."),
                target_description: Some(String::from("current form")),
                selected_skills,
                summary: String::from("There is no current page to submit a form on yet."),
                next_recommended_action: Some(String::from(
                    "Open a page first, then ask me to submit the form.",
                )),
                step_id: String::from("report-missing-submit-page"),
                purpose: String::from(
                    "Report that there is no active page available for form submission.",
                ),
            },
        ));
    };

    let candidate_forms = submittable_form_elements(current_page);
    let resolved_form = if current_page.interactive_elements.is_empty() {
        None
    } else if candidate_forms.len() == 1 {
        Some(candidate_forms[0].clone())
    } else if candidate_forms.is_empty() {
        return Some(build_direct_follow_up_output(
            request_id,
            DirectFollowUpSpec {
                intent_name: IntentName::SubmitForm,
                goal: String::from("Submit the active form."),
                target_description: Some(String::from("current form")),
                selected_skills,
                summary: String::from(
                    "I could not identify a submittable form on the current page.",
                ),
                next_recommended_action: Some(String::from(
                    "Focus a field in the form or describe which form you want to submit.",
                )),
                step_id: String::from("report-missing-submit-form"),
                purpose: String::from(
                    "Report that no submittable form could be identified on the current page.",
                ),
            },
        ));
    } else {
        let candidate_names = summarize_form_candidate_names(&candidate_forms);
        let summary = if candidate_names.is_empty() {
            String::from(
                "I found multiple forms on the current page. Please tell me which one to submit.",
            )
        } else {
            format!(
                "I found multiple forms on the current page: {}. Please tell me which one to submit.",
                candidate_names.join(", ")
            )
        };
        return Some(build_direct_follow_up_output(
            request_id,
            DirectFollowUpSpec {
                intent_name: IntentName::SubmitForm,
                goal: String::from("Submit the active form."),
                target_description: Some(String::from("current form")),
                selected_skills,
                summary,
                next_recommended_action: Some(String::from(
                    "Name the form or focus a field in it before asking me to submit.",
                )),
                step_id: String::from("report-ambiguous-submit-form"),
                purpose: String::from(
                    "Report that multiple possible forms are available and submission is ambiguous.",
                ),
            },
        ));
    };

    let target_description = resolved_form.as_ref().map(describe_form_element);
    let prompt_text = match target_description.as_deref() {
        Some(description) => format!("Do you want me to submit {description} now?"),
        None => String::from("Do you want me to submit the active form now?"),
    };
    let confirmation_reason = String::from("submitting the form may send data");
    let user_message = String::from("Please confirm before I submit the form.");

    Some(PlannerOutput {
        status: PlannerStatus::NeedsConfirmation,
        intent: IntentSummary {
            name: IntentName::SubmitForm,
            goal: String::from("Submit the active form."),
            target_description,
        },
        selected_skills,
        steps: vec![
            PlannedStep {
                step_id: String::from("confirm-submit-form"),
                tool_name: ToolName::ConfirmAction,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "prompt_text": prompt_text,
                    "reason": confirmation_reason
                }),
                purpose: String::from("Require explicit confirmation before submitting the form."),
                on_success: StepTransition::RequestConfirmation,
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from("submit-active-form"),
                tool_name: ToolName::SubmitActiveForm,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "form_element_id": resolved_form.as_ref().map(|form| form.element_id.clone())
                }),
                purpose: String::from("Submit the confirmed active form in the live browser."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ],
        requires_confirmation: true,
        confirmation_reason: Some(String::from("submitting the form may send data")),
        blocked_reason: None,
        user_message: Some(user_message),
    })
}
