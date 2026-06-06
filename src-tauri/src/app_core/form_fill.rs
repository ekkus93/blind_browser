use crate::commands::{
    FillFieldCorrectionCommand, FindElementInput, IntentName, IntentSummary, PlannedStep,
    PlannerOutput, PlannerStatus, ReportStatus, StepTransition, ToolName,
    is_direct_submit_form_command, parse_direct_fill_and_submit_command,
    parse_direct_fill_field_command, parse_direct_focus_field_command,
    parse_fill_field_correction_command, DEFAULT_FIND_ELEMENT_MAX_CANDIDATES,
};
use crate::page_model::PageModel;
use super::interaction_tools::{
    build_find_element_query, describe_form_element, determine_find_element_resolution,
    focusable_field_elements, rank_find_element_candidates, resolve_typeable_element,
    submittable_form_elements, summarize_candidate_names, summarize_form_candidate_names,
};

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

struct DirectFollowUpSpec {
    intent_name: IntentName,
    goal: String,
    target_description: Option<String>,
    selected_skills: Vec<String>,
    summary: String,
    next_recommended_action: Option<String>,
    step_id: String,
    purpose: String,
}

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
        visibility_filter: crate::commands::ElementVisibilityFilter::VisibleOnly,
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

fn selected_skills_for_fill_command(
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

fn build_direct_fill_ready_output(
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

fn build_direct_fill_and_submit_ready_output(
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
                .find(|candidate_id| candidate_id.as_str() != active_element_id)
                .and_then(|candidate_id| {
                    resolve_typeable_element(current_page, candidate_id)
                        .ok()
                        .map(|_| candidate_id.clone())
                });
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

pub(crate) fn resolve_direct_fill_command_internal(
    transcript: &str,
    request_id: &str,
    current_page_id: Option<&str>,
    current_page: Option<&PageModel>,
    active_skill_names: &[String],
    confirmation_confidence_threshold: f32,
    submit_after: bool,
) -> Option<ResolvedDirectFieldCommand> {
    let command = if submit_after {
        parse_direct_fill_and_submit_command(transcript)?
    } else {
        parse_direct_fill_field_command(transcript)?
    };
    let selected_skills = selected_skills_for_fill_command(active_skill_names, submit_after);
    let goal = if submit_after {
        "Fill the requested field and submit the form."
    } else {
        "Fill the requested field."
    };
    let intent_name = if submit_after {
        IntentName::SubmitForm
    } else {
        IntentName::FillInput
    };

    let Some(description) = command.description else {
        let planner_output = build_direct_follow_up_output(
            request_id,
            DirectFollowUpSpec {
                intent_name,
                goal: goal.to_string(),
                target_description: Some(String::from("field fill target")),
                selected_skills,
                summary: if submit_after {
                    String::from("Please tell me which field to fill before I submit.")
                } else {
                    String::from("Please tell me which field to fill.")
                },
                next_recommended_action: Some(if submit_after {
                    String::from(
                        "Say the field name and value, like fill the email field with phil@example.com and submit.",
                    )
                } else {
                    String::from(
                        "Say the field name and value, like fill the email field with phil@example.com.",
                    )
                }),
                step_id: if submit_after {
                    String::from("report-missing-fill-submit-field-description")
                } else {
                    String::from("report-missing-fill-field-description")
                },
                purpose: if submit_after {
                    String::from(
                        "Report that the field name is required before filling and submitting.",
                    )
                } else {
                    String::from("Report that the field name is required before filling.")
                },
            },
        );
        return Some(ResolvedDirectFieldCommand {
            planner_output,
            recent_field_context: None,
        });
    };

    let Some(text) = command.text else {
        let planner_output = build_direct_follow_up_output(
            request_id,
            DirectFollowUpSpec {
                intent_name,
                goal: goal.to_string(),
                target_description: Some(description),
                selected_skills,
                summary: if submit_after {
                    String::from("Please tell me what text to enter before I submit.")
                } else {
                    String::from("Please tell me what text to enter.")
                },
                next_recommended_action: Some(if submit_after {
                    String::from(
                        "Say the value after the field name, like fill the email field with phil@example.com and submit.",
                    )
                } else {
                    String::from(
                        "Say the value after the field name, like fill the email field with phil@example.com.",
                    )
                }),
                step_id: if submit_after {
                    String::from("report-missing-fill-submit-text")
                } else {
                    String::from("report-missing-fill-text")
                },
                purpose: if submit_after {
                    String::from(
                        "Report that the requested field value is required before filling and submitting.",
                    )
                } else {
                    String::from(
                        "Report that the requested field value is required before filling.",
                    )
                },
            },
        );
        return Some(ResolvedDirectFieldCommand {
            planner_output,
            recent_field_context: None,
        });
    };

    let Some(current_page) = current_page else {
        let planner_output = build_direct_follow_up_output(
            request_id,
            DirectFollowUpSpec {
                intent_name,
                goal: goal.to_string(),
                target_description: None,
                selected_skills,
                summary: if submit_after {
                    String::from("There is no current page to fill and submit a form on yet.")
                } else {
                    String::from("There is no current page to fill a field on yet.")
                },
                next_recommended_action: Some(if submit_after {
                    String::from(
                        "Open a page first, then ask me to fill a field and submit the form.",
                    )
                } else {
                    String::from("Open a page first, then ask me to fill a field.")
                }),
                step_id: if submit_after {
                    String::from("report-missing-fill-submit-page")
                } else {
                    String::from("report-missing-fill-page")
                },
                purpose: if submit_after {
                    String::from(
                        "Report that there is no active page available for filling and submitting.",
                    )
                } else {
                    String::from("Report that there is no active page available for field entry.")
                },
            },
        );
        return Some(ResolvedDirectFieldCommand {
            planner_output,
            recent_field_context: None,
        });
    };

    let field_elements = focusable_field_elements(current_page);
    if field_elements.is_empty() {
        let planner_output = build_direct_follow_up_output(
            request_id,
            DirectFollowUpSpec {
                intent_name,
                goal: goal.to_string(),
                target_description: Some(description.clone()),
                selected_skills,
                summary: String::from("I could not find any fillable fields on the current page."),
                next_recommended_action: Some(String::from(
                    "Try again after the page finishes loading or becomes interactive.",
                )),
                step_id: if submit_after {
                    String::from("report-missing-fill-submit-fields")
                } else {
                    String::from("report-missing-fillable-fields")
                },
                purpose: if submit_after {
                    String::from(
                        "Report that no editable fields are available for filling and submitting.",
                    )
                } else {
                    String::from(
                        "Report that no editable fields are available on the current page.",
                    )
                },
            },
        );
        return Some(ResolvedDirectFieldCommand {
            planner_output,
            recent_field_context: None,
        });
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
        visibility_filter: crate::commands::ElementVisibilityFilter::VisibleOnly,
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
    let recent_field_context = current_page_id.map(|_| PendingRecentFieldContext {
        target_description: Some(description.clone()),
        active_element_id: chosen_element_id.clone(),
        candidate_element_ids: candidates
            .iter()
            .map(|candidate| candidate.element_id.clone())
            .collect(),
        pending_text: Some(text.clone()),
        submit_after,
    });

    if let Some(element_id) = chosen_element_id {
        let planner_output = if submit_after {
            build_direct_fill_and_submit_ready_output(
                request_id,
                selected_skills,
                Some(description),
                element_id,
                text,
            )
        } else {
            build_direct_fill_ready_output(
                request_id,
                selected_skills,
                Some(description),
                element_id,
                text,
            )
        };
        return Some(ResolvedDirectFieldCommand {
            planner_output,
            recent_field_context,
        });
    }

    let summary = if requires_confirmation {
        let candidate_names = summarize_candidate_names(current_page, &candidates);
        if candidate_names.is_empty() {
            if submit_after {
                format!(
                    "I found multiple possible fields for {description}. Please be more specific before I submit."
                )
            } else {
                format!(
                    "I found multiple possible fields for {description}. Please be more specific."
                )
            }
        } else if submit_after {
            format!(
                "I found multiple possible fields for {description}: {}. Please be more specific before I submit.",
                candidate_names.join(", ")
            )
        } else {
            format!(
                "I found multiple possible fields for {description}: {}. Please be more specific.",
                candidate_names.join(", ")
            )
        }
    } else {
        format!("I could not find a visible field matching {description}.")
    };

    let planner_output = build_direct_follow_up_output(
        request_id,
        DirectFollowUpSpec {
            intent_name,
            goal: goal.to_string(),
            target_description: Some(description),
            selected_skills,
            summary,
            next_recommended_action: Some(String::from(
                "Try naming the field label or placeholder more specifically.",
            )),
            step_id: if submit_after {
                String::from("report-fill-submit-follow-up")
            } else {
                String::from("report-fill-field-follow-up")
            },
            purpose: if submit_after {
                String::from(
                    "Report that the requested field could not be filled and submitted deterministically.",
                )
            } else {
                String::from(
                    "Report that the requested field could not be filled deterministically.",
                )
            },
        },
    );
    Some(ResolvedDirectFieldCommand {
        planner_output,
        recent_field_context,
    })
}

#[cfg(test)]
pub(crate) fn resolve_direct_fill_field_command(
    transcript: &str,
    request_id: &str,
    current_page: Option<&PageModel>,
    active_skill_names: &[String],
    confirmation_confidence_threshold: f32,
) -> Option<PlannerOutput> {
    resolve_direct_fill_command_internal(
        transcript,
        request_id,
        None,
        current_page,
        active_skill_names,
        confirmation_confidence_threshold,
        false,
    )
    .map(|resolved| resolved.planner_output)
}

#[cfg(test)]
pub(crate) fn resolve_direct_fill_and_submit_command(
    transcript: &str,
    request_id: &str,
    current_page: Option<&PageModel>,
    active_skill_names: &[String],
    confirmation_confidence_threshold: f32,
) -> Option<PlannerOutput> {
    resolve_direct_fill_command_internal(
        transcript,
        request_id,
        None,
        current_page,
        active_skill_names,
        confirmation_confidence_threshold,
        true,
    )
    .map(|resolved| resolved.planner_output)
}

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

fn build_direct_follow_up_output(request_id: &str, spec: DirectFollowUpSpec) -> PlannerOutput {
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
