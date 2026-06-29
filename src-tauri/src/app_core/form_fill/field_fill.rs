use crate::app_core::element_scoring::{
    build_find_element_query, determine_find_element_resolution, focusable_field_elements,
    rank_find_element_candidates, summarize_candidate_names,
};
use crate::app_core::fill_correction::{
    build_direct_fill_and_submit_ready_output, build_direct_fill_ready_output,
    build_direct_follow_up_output, selected_skills_for_fill_command, DirectFollowUpSpec,
    PendingRecentFieldContext, ResolvedDirectFieldCommand,
};
#[cfg(test)]
use crate::commands::PlannerOutput;
use crate::commands::{
    parse_direct_fill_and_submit_command, parse_direct_fill_field_command, ElementVisibilityFilter,
    FindElementInput, IntentName, DEFAULT_FIND_ELEMENT_MAX_CANDIDATES,
};
use crate::page_model::PageModel;

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
