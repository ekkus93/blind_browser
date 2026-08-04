use super::interaction_tools::resolve_typeable_element;
use crate::commands::{
    parse_fill_field_correction_command, FillFieldCorrectionCommand, IntentName, IntentSummary,
    PlannedStep, PlannerOutput, PlannerStatus, ReportStatus, StepTransition, ToolName,
};
use crate::page_model::PageModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingRecentFieldContext {
    pub(crate) target_description: Option<String>,
    pub(crate) active_element_id: Option<String>,
    pub(crate) candidate_element_ids: Vec<String>,
    pub(crate) pending_text: Option<String>,
    pub(crate) submit_after: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecentFieldContext {
    pub(crate) page_id: String,
    pub(crate) target_description: Option<String>,
    pub(crate) active_element_id: Option<String>,
    pub(crate) candidate_element_ids: Vec<String>,
    pub(crate) pending_text: Option<String>,
    pub(crate) submit_after: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ResolvedDirectFieldCommand {
    pub(crate) planner_output: PlannerOutput,
    pub(crate) recent_field_context: Option<PendingRecentFieldContext>,
}

pub(crate) struct DirectFollowUpSpec {
    pub(crate) intent_name: IntentName,
    pub(crate) goal: String,
    pub(crate) target_description: Option<String>,
    pub(crate) selected_skills: Vec<String>,
    pub(crate) summary: String,
    pub(crate) next_recommended_action: Option<String>,
    pub(crate) step_id: String,
    pub(crate) purpose: String,
}

pub(crate) fn selected_skills_for_fill_command(
    active_skill_names: &[String],
    submit_after: bool,
) -> Vec<String> {
    let expected_skill_name = if submit_after {
        "fill_and_submit_form"
    } else {
        "fill_field_by_label"
    };

    if active_skill_names
        .iter()
        .any(|active_name| active_name == expected_skill_name)
    {
        vec![expected_skill_name.to_string()]
    } else {
        Vec::new()
    }
}

pub(crate) fn build_direct_fill_ready_output(
    request_id: &str,
    selected_skills: Vec<String>,
    description: Option<String>,
    element_id: String,
    text: String,
) -> PlannerOutput {
    PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::FillInput,
            goal: String::from("Fill the requested field."),
            target_description: description,
        },
        selected_skills,
        steps: vec![
            PlannedStep {
                step_id: String::from("focus-fill-field"),
                tool_name: ToolName::FocusElement,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "element_id": element_id
                }),
                purpose: String::from("Move focus to the requested field before typing."),
                on_success: StepTransition::NextStep {
                    step_id: String::from("type-into-fill-field"),
                },
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from("type-into-fill-field"),
                tool_name: ToolName::TypeIntoElement,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "element_id": element_id,
                    "text": text,
                    "text_entry_mode": "Replace",
                    "submit_mode": "KeepEditing"
                }),
                purpose: String::from(
                    "Replace the requested field contents with the spoken value.",
                ),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    }
}

pub(crate) fn build_direct_fill_and_submit_ready_output(
    request_id: &str,
    selected_skills: Vec<String>,
    description: Option<String>,
    element_id: String,
    text: String,
) -> PlannerOutput {
    let description_text = description
        .clone()
        .unwrap_or_else(|| String::from("requested"));
    let prompt_text = format!(
        "Do you want me to fill the {description_text} field with {text} and then submit that form?"
    );
    let confirmation_reason =
        String::from("filling the field and submitting the form may change or send data");

    PlannerOutput {
        status: PlannerStatus::NeedsConfirmation,
        intent: IntentSummary {
            name: IntentName::SubmitForm,
            goal: String::from("Fill the requested field and submit the form."),
            target_description: description,
        },
        selected_skills,
        steps: vec![
            PlannedStep {
                step_id: String::from("confirm-fill-and-submit-form"),
                tool_name: ToolName::ConfirmAction,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "prompt_text": prompt_text,
                    "reason": confirmation_reason
                }),
                purpose: String::from(
                    "Require explicit confirmation before filling the field and submitting the form.",
                ),
                on_success: StepTransition::RequestConfirmation,
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from("focus-fill-submit-field"),
                tool_name: ToolName::FocusElement,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "element_id": element_id
                }),
                purpose: String::from("Move focus to the requested field before typing."),
                on_success: StepTransition::NextStep {
                    step_id: String::from("type-fill-submit-field"),
                },
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from("type-fill-submit-field"),
                tool_name: ToolName::TypeIntoElement,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "element_id": element_id,
                    "text": text,
                    "text_entry_mode": "Replace",
                    "submit_mode": "KeepEditing"
                }),
                purpose: String::from(
                    "Replace the requested field contents with the spoken value before submission.",
                ),
                on_success: StepTransition::NextStep {
                    step_id: String::from("submit-fill-submit-form"),
                },
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from("submit-fill-submit-form"),
                tool_name: ToolName::SubmitActiveForm,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "form_element_id": serde_json::Value::Null
                }),
                purpose: String::from(
                    "Submit the form that owns the focused field after the fill step succeeds.",
                ),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ],
        requires_confirmation: true,
        confirmation_reason: Some(String::from(
            "filling the field and submitting the form may change or send data",
        )),
        blocked_reason: None,
        user_message: Some(String::from(
            "Please confirm before I fill the field and submit the form.",
        )),
    }
}

pub(crate) fn resolve_recent_fill_correction_command(
    transcript: &str,
    request_id: &str,
    current_page_id: Option<&str>,
    current_page: Option<&PageModel>,
    active_skill_names: &[String],
    recent_field_context: Option<&RecentFieldContext>,
) -> Option<(PlannerOutput, Option<RecentFieldContext>)> {
    let correction = parse_fill_field_correction_command(transcript)?;
    let matching_context = recent_field_context.filter(|context| {
        current_page_id.is_some_and(|page_id| context.page_id == page_id) && current_page.is_some()
    });

    match correction {
        FillFieldCorrectionCommand::AlternateField => {
            let Some(context) = matching_context else {
                let planner_output = build_direct_follow_up_output(
                    request_id,
                    DirectFollowUpSpec {
                        intent_name: IntentName::FillInput,
                        goal: String::from("Fill the requested field."),
                        target_description: Some(String::from("field fill target")),
                        selected_skills: selected_skills_for_fill_command(active_skill_names, false),
                        summary: String::from(
                            "Please tell me which field you want me to use instead.",
                        ),
                        next_recommended_action: Some(String::from(
                            "Name the field label or placeholder, like use the billing email field instead.",
                        )),
                        step_id: String::from("report-missing-alternate-field-context"),
                        purpose: String::from(
                            "Report that the alternate field cannot be resolved without recent context.",
                        ),
                    },
                );
                return Some((planner_output, None));
            };

            let current_page = current_page?;
            let Some(active_element_id) = context.active_element_id.as_deref() else {
                let planner_output = build_direct_follow_up_output(
                    request_id,
                    DirectFollowUpSpec {
                        intent_name: IntentName::FillInput,
                        goal: String::from("Fill the requested field."),
                        target_description: context.target_description.clone(),
                        selected_skills: selected_skills_for_fill_command(
                            active_skill_names,
                            context.submit_after,
                        ),
                        summary: String::from(
                            "Please tell me which field you mean before I switch to another one.",
                        ),
                        next_recommended_action: Some(String::from(
                            "Name the specific field label or placeholder you want me to use.",
                        )),
                        step_id: String::from("report-missing-active-field-context"),
                        purpose: String::from(
                            "Report that there is no recent resolved field target to swap away from.",
                        ),
                    },
                );
                return Some((planner_output, Some(context.clone())));
            };

            let alternate_element_id = context
                .candidate_element_ids
                .iter()
                .find(|candidate_id| {
                    candidate_id.as_str() != active_element_id
                        && resolve_typeable_element(current_page, candidate_id.as_str()).is_ok()
                })
                .cloned();
            let Some(alternate_element_id) = alternate_element_id else {
                let planner_output = build_direct_follow_up_output(
                    request_id,
                    DirectFollowUpSpec {
                        intent_name: IntentName::FillInput,
                        goal: String::from("Fill the requested field."),
                        target_description: context.target_description.clone(),
                        selected_skills: selected_skills_for_fill_command(
                            active_skill_names,
                            context.submit_after,
                        ),
                        summary: String::from(
                            "Please tell me which field you want after all.",
                        ),
                        next_recommended_action: Some(String::from(
                            "Name the specific field label or placeholder so I can target it deterministically.",
                        )),
                        step_id: String::from("report-missing-alternate-field-target"),
                        purpose: String::from(
                            "Report that no alternate recent field target is available anymore.",
                        ),
                    },
                );
                return Some((planner_output, Some(context.clone())));
            };

            let Some(text) = context.pending_text.clone() else {
                let planner_output = build_direct_follow_up_output(
                    request_id,
                    DirectFollowUpSpec {
                        intent_name: IntentName::FillInput,
                        goal: String::from("Fill the requested field."),
                        target_description: context.target_description.clone(),
                        selected_skills: selected_skills_for_fill_command(
                            active_skill_names,
                            context.submit_after,
                        ),
                        summary: String::from(
                            "Please tell me what text to enter before I switch fields.",
                        ),
                        next_recommended_action: Some(String::from(
                            "Say the value you want me to type after naming the field.",
                        )),
                        step_id: String::from("report-missing-alternate-field-text"),
                        purpose: String::from(
                            "Report that the original field value is no longer available for the alternate target.",
                        ),
                    },
                );
                return Some((planner_output, Some(context.clone())));
            };

            let planner_output = if context.submit_after {
                build_direct_fill_and_submit_ready_output(
                    request_id,
                    selected_skills_for_fill_command(active_skill_names, true),
                    context.target_description.clone(),
                    alternate_element_id.clone(),
                    text.clone(),
                )
            } else {
                build_direct_fill_ready_output(
                    request_id,
                    selected_skills_for_fill_command(active_skill_names, false),
                    context.target_description.clone(),
                    alternate_element_id.clone(),
                    text.clone(),
                )
            };
            let mut next_context = context.clone();
            next_context.active_element_id = Some(alternate_element_id);
            next_context.pending_text = Some(text);
            Some((planner_output, Some(next_context)))
        }
        FillFieldCorrectionCommand::ReplaceValue { text } => {
            let Some(context) = matching_context else {
                let planner_output = build_direct_follow_up_output(
                    request_id,
                    DirectFollowUpSpec {
                        intent_name: IntentName::FillInput,
                        goal: String::from("Fill the requested field."),
                        target_description: Some(String::from("field fill target")),
                        selected_skills: selected_skills_for_fill_command(active_skill_names, false),
                        summary: String::from("Please tell me which field to update."),
                        next_recommended_action: Some(String::from(
                            "Say the field name and value, like fill the city field with Seattle.",
                        )),
                        step_id: String::from("report-missing-recent-fill-target"),
                        purpose: String::from(
                            "Report that there is no recent field target available for replacement text.",
                        ),
                    },
                );
                return Some((planner_output, None));
            };

            let current_page = current_page?;
            let active_element_id = context
                .active_element_id
                .as_ref()
                .filter(|element_id| resolve_typeable_element(current_page, element_id).is_ok())
                .cloned();
            let Some(active_element_id) = active_element_id else {
                let planner_output = build_direct_follow_up_output(
                    request_id,
                    DirectFollowUpSpec {
                        intent_name: IntentName::FillInput,
                        goal: String::from("Fill the requested field."),
                        target_description: context.target_description.clone(),
                        selected_skills: selected_skills_for_fill_command(active_skill_names, false),
                        summary: String::from(
                            "Please tell me which field to update because the recent target is no longer available.",
                        ),
                        next_recommended_action: Some(String::from(
                            "Say the field label or placeholder together with the new value.",
                        )),
                        step_id: String::from("report-stale-recent-fill-target"),
                        purpose: String::from(
                            "Report that the stored recent field target cannot be reused on the current page.",
                        ),
                    },
                );
                return Some((planner_output, Some(context.clone())));
            };

            let planner_output = build_direct_fill_ready_output(
                request_id,
                selected_skills_for_fill_command(active_skill_names, false),
                context.target_description.clone(),
                active_element_id.clone(),
                text.clone(),
            );
            let mut next_context = context.clone();
            next_context.active_element_id = Some(active_element_id);
            next_context.pending_text = Some(text);
            next_context.submit_after = false;
            Some((planner_output, Some(next_context)))
        }
    }
}

pub(crate) fn build_direct_follow_up_output(
    request_id: &str,
    spec: DirectFollowUpSpec,
) -> PlannerOutput {
    PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: spec.intent_name,
            goal: spec.goal,
            target_description: spec.target_description,
        },
        selected_skills: spec.selected_skills,
        steps: vec![PlannedStep {
            step_id: spec.step_id,
            tool_name: ToolName::ReportResult,
            arguments: serde_json::json!({
                "request_id": request_id,
                "timeout_ms": serde_json::Value::Null,
                "status": ReportStatus::NeedsFollowUp,
                "summary": spec.summary.clone(),
                "next_recommended_action": spec.next_recommended_action,
                "user_message": spec.summary
            }),
            purpose: spec.purpose,
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    }
}
