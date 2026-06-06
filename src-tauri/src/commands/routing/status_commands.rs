use super::*;

pub(super) fn format_browser_visibility_mode(mode: BrowserVisibilityMode) -> String {
    match mode {
        BrowserVisibilityMode::Visible => String::from("visible"),
        BrowserVisibilityMode::Headless => String::from("headless"),
    }
}
pub(super) fn current_page_label(agent_state: &AgentStateData) -> String {
    normalized_optional_text(agent_state.title.as_deref())
        .or_else(|| normalized_optional_text(agent_state.url.as_deref()))
        .unwrap_or_else(|| String::from("no page open"))
}

pub(super) fn format_runtime_status_summary(runtime_status: &GetRuntimeStatusData) -> String {
    let page_summary = current_page_label_from_runtime_status(runtime_status);
    let browser_mode = format_browser_visibility_mode(runtime_status.browser_visibility);
    let listening = if runtime_status.listening_state.is_listening {
        "on"
    } else {
        "off"
    };
    let speaking = if runtime_status.speaking {
        "active"
    } else {
        "idle"
    };
    let back = if runtime_status.browser_history.can_go_back {
        "available"
    } else {
        "unavailable"
    };
    let forward = if runtime_status.browser_history.can_go_forward {
        "available"
    } else {
        "unavailable"
    };

    format!(
        "Current page is {page_summary}. Browser mode is {browser_mode}. Listening is {listening}. Speech output is {speaking}. Back is {back}. Forward is {forward}."
    )
}

pub(super) fn current_page_label_from_runtime_status(runtime_status: &GetRuntimeStatusData) -> String {
    normalized_optional_text(runtime_status.title.as_deref())
        .or_else(|| normalized_optional_text(runtime_status.url.as_deref()))
        .unwrap_or_else(|| String::from("no page open"))
}

pub(super) fn format_back_history_summary(runtime_status: &GetRuntimeStatusData) -> String {
    if runtime_status.browser_history.can_go_back {
        String::from("Back navigation is available.")
    } else {
        String::from("Back navigation is not available.")
    }
}

pub(super) fn format_forward_history_summary(runtime_status: &GetRuntimeStatusData) -> String {
    if runtime_status.browser_history.can_go_forward {
        String::from("Forward navigation is available.")
    } else {
        String::from("Forward navigation is not available.")
    }
}

pub(super) fn format_listening_summary(runtime_status: &GetRuntimeStatusData) -> String {
    if runtime_status.listening_state.is_listening {
        String::from("Listening is on.")
    } else {
        String::from("Listening is off.")
    }
}

pub(super) fn format_speaking_summary(runtime_status: &GetRuntimeStatusData) -> String {
    if runtime_status.speaking {
        String::from("Speech output is active.")
    } else {
        String::from("Speech output is idle.")
    }
}

pub(super) fn format_browser_mode_summary(runtime_status: &GetRuntimeStatusData) -> String {
    format!(
        "Browser mode is {}.",
        format_browser_visibility_mode(runtime_status.browser_visibility)
    )
}

fn selected_status_skill(active_skill_names: &[String]) -> Vec<String> {
    if active_skill_names
        .iter()
        .any(|active_name| active_name == "get_status")
    {
        vec![String::from("get_status")]
    } else if active_skill_names
        .iter()
        .any(|active_name| active_name == "announce_state")
    {
        vec![String::from("announce_state")]
    } else {
        Vec::new()
    }
}

pub(crate) fn resolve_direct_status_query_command(
    transcript: &str,
    request_id: &str,
    agent_state: &AgentStateData,
    runtime_status: &GetRuntimeStatusData,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() {
        return None;
    }

    if is_current_url_query_phrase(&normalized) {
        let summary = format_current_url_summary(agent_state);
        return Some(build_status_query_planner_output(StatusQueryPlanSpec {
            request_id,
            intent_name: IntentName::GetCurrentUrl,
            goal: String::from("Report the current page URL and title."),
            selected_skills: selected_skill(active_skill_names, "get_current_url"),
            target_description: Some(current_page_label(agent_state)),
            read_step_id: "get-current-url",
            read_tool_name: ToolName::GetAgentState,
            read_tool_arguments: serde_json::json!({
                "request_id": request_id,
                "include_last_transcript": false
            }),
            read_tool_purpose: String::from("Read the current agent page state."),
            report_step_id: "report-current-url",
            report_summary: summary,
        }));
    }

    if is_status_query_phrase(&normalized)
        || is_history_query_phrase(&normalized)
        || is_listening_query_phrase(&normalized)
        || is_speaking_query_phrase(&normalized)
        || is_browser_mode_query_phrase(&normalized)
    {
        let summary = if is_back_history_query_phrase(&normalized) {
            format_back_history_summary(runtime_status)
        } else if is_forward_history_query_phrase(&normalized) {
            format_forward_history_summary(runtime_status)
        } else if is_listening_query_phrase(&normalized) {
            format_listening_summary(runtime_status)
        } else if is_speaking_query_phrase(&normalized) {
            format_speaking_summary(runtime_status)
        } else if is_browser_mode_query_phrase(&normalized) {
            format_browser_mode_summary(runtime_status)
        } else {
            format_runtime_status_summary(runtime_status)
        };

        return Some(build_status_query_planner_output(StatusQueryPlanSpec {
            request_id,
            intent_name: IntentName::GetStatus,
            goal: String::from("Report the current runtime status relevant to the user's query."),
            selected_skills: selected_status_skill(active_skill_names),
            target_description: Some(String::from("runtime status")),
            read_step_id: "get-runtime-status",
            read_tool_name: ToolName::GetRuntimeStatus,
            read_tool_arguments: serde_json::json!({
                "request_id": request_id,
                "include_provider_modes": false
            }),
            read_tool_purpose: String::from("Read the current runtime status."),
            report_step_id: "report-runtime-status",
            report_summary: summary,
        }));
    }

    None
}

pub(crate) fn resolve_direct_read_title_command(
    transcript: &str,
    request_id: &str,
    agent_state: &AgentStateData,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() || !is_read_title_phrase(&normalized) {
        return None;
    }

    let summary = match normalized_optional_text(agent_state.title.as_deref()) {
        Some(title) => format!("Page title is {title}."),
        None => String::from("This page does not have a readable title yet."),
    };

    Some(PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::ReadTitle,
            goal: String::from("Read the current page title."),
            target_description: Some(String::from("current page title")),
        },
        selected_skills: selected_skill(active_skill_names, "read_title"),
        steps: vec![PlannedStep {
            step_id: String::from("report-page-title"),
            tool_name: ToolName::ReportResult,
            arguments: serde_json::json!({
                "request_id": request_id,
                "timeout_ms": serde_json::Value::Null,
                "status": ReportStatus::Success,
                "summary": summary.clone(),
                "next_recommended_action": serde_json::Value::Null,
                "user_message": summary
            }),
            purpose: String::from("Speak the current page title."),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    })
}

pub(crate) fn resolve_direct_repeat_command(
    transcript: &str,
    request_id: &str,
    agent_state: &AgentStateData,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() || !is_repeat_phrase(&normalized) {
        return None;
    }

    let selected_skills = selected_skill(active_skill_names, "repeat");
    let Some(region_id) = agent_state
        .narration_cursor
        .as_ref()
        .and_then(|cursor| cursor.current_region_id.as_deref())
    else {
        let summary = String::from("There is no current region to repeat yet.");
        return Some(PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::Repeat,
                goal: String::from("Repeat the current narration region."),
                target_description: Some(String::from("current narration region")),
            },
            selected_skills,
            steps: vec![PlannedStep {
                step_id: String::from("report-missing-repeat-region"),
                tool_name: ToolName::ReportResult,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "status": ReportStatus::NeedsFollowUp,
                    "summary": summary.clone(),
                    "next_recommended_action": "Read the page or move to a region first.",
                    "user_message": summary
                }),
                purpose: String::from(
                    "Report that no current narration region is available to repeat.",
                ),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        });
    };

    Some(PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::Repeat,
            goal: String::from("Repeat the current narration region."),
            target_description: Some(String::from("current narration region")),
        },
        selected_skills,
        steps: vec![PlannedStep {
            step_id: String::from("repeat-current-region"),
            tool_name: ToolName::ReadRegion,
            arguments: serde_json::json!({
                "request_id": request_id,
                "timeout_ms": serde_json::Value::Null,
                "region_id": region_id,
                "interruption_mode": "Interrupt"
            }),
            purpose: String::from(
                "Repeat the current narration region from the stored narration cursor.",
            ),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    })
}
