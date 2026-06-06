use super::*;

mod intent;
pub use intent::infer_intent_hint;
pub(crate) use intent::{normalize_transcript_for_routing, tokenize_text, likely_tools_for_intent};
use intent::*;

mod audio_commands;
use audio_commands::*;

mod url_commands;
use url_commands::*;

mod planner_outputs;
use planner_outputs::*;

mod status_commands;
use status_commands::*;

pub(crate) fn resolve_direct_audio_command(
    transcript: &str,
    request_id: &str,
    current_volume: f32,
    current_speed: f32,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_audio_command_text(transcript);
    if normalized.is_empty() {
        return None;
    }

    if is_volume_query_phrase(&normalized) {
        let summary = format!(
            "Playback volume is {}.",
            format_playback_volume(current_volume)
        );
        return Some(build_audio_report_planner_output(
            request_id,
            IntentName::GetPlaybackVolume,
            String::from("Report the current playback volume."),
            selected_audio_skill(active_skill_names, "get_volume"),
            Some(format_playback_volume(current_volume)),
            summary,
        ));
    }

    if is_speed_query_phrase(&normalized) {
        let summary = format!(
            "Playback speed is {}.",
            format_playback_speed(current_speed)
        );
        return Some(build_audio_report_planner_output(
            request_id,
            IntentName::GetPlaybackSpeed,
            String::from("Report the current playback speed."),
            selected_audio_skill(active_skill_names, "get_playback_speed"),
            Some(format_playback_speed(current_speed)),
            summary,
        ));
    }

    if let Some(volume) = parse_volume_command(&normalized, current_volume) {
        let summary = format!(
            "Playback volume set to {}.",
            format_playback_volume(volume.value)
        );
        return Some(build_audio_set_planner_output(AudioSetPlanSpec {
            request_id,
            intent_name: IntentName::SetPlaybackVolume,
            goal: volume.goal,
            selected_skills: selected_audio_skill(active_skill_names, volume.skill_name),
            target_description: Some(format_playback_volume(volume.value)),
            set_step_id: "set-playback-volume",
            tool_name: ToolName::SetPlaybackVolume,
            tool_arguments: serde_json::json!({
                "request_id": request_id,
                "timeout_ms": serde_json::Value::Null,
                "volume": volume.value
            }),
            tool_purpose: String::from("Apply and persist the requested playback volume."),
            report_step_id: "report-playback-volume",
            report_summary: summary,
        }));
    }

    if let Some(speed) = parse_speed_command(&normalized, current_speed) {
        let summary = format!(
            "Playback speed set to {}.",
            format_playback_speed(speed.value)
        );
        return Some(build_audio_set_planner_output(AudioSetPlanSpec {
            request_id,
            intent_name: IntentName::SetPlaybackSpeed,
            goal: speed.goal,
            selected_skills: selected_audio_skill(active_skill_names, speed.skill_name),
            target_description: Some(format_playback_speed(speed.value)),
            set_step_id: "set-playback-speed",
            tool_name: ToolName::SetPlaybackSpeed,
            tool_arguments: serde_json::json!({
                "request_id": request_id,
                "timeout_ms": serde_json::Value::Null,
                "speed": speed.value
            }),
            tool_purpose: String::from("Apply and persist the requested playback speed."),
            report_step_id: "report-playback-speed",
            report_summary: summary,
        }));
    }

    None
}

pub(crate) fn resolve_direct_browser_visibility_command(
    transcript: &str,
    request_id: &str,
    current_visibility: BrowserVisibilityMode,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() {
        return None;
    }

    let target_mode = parse_browser_visibility_command(&normalized, current_visibility)?;
    let summary = format!(
        "Browser mode set to {}.",
        format_browser_visibility_mode(target_mode)
    );

    Some(build_browser_visibility_planner_output(
        request_id,
        target_mode,
        selected_skill(active_skill_names, "toggle_browser_visibility"),
        summary,
    ))
}

pub(crate) fn resolve_direct_navigation_readback_command(
    transcript: &str,
    request_id: &str,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() {
        return None;
    }

    if is_go_back_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::GoBack,
                goal: String::from("Navigate back one entry in browser history."),
                target_description: None,
            },
            selected_skill(active_skill_names, "go_back"),
            PlannedStep {
                step_id: String::from("go-back"),
                tool_name: ToolName::GoBack,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "steps": 1,
                    "wait_for_load_state": LoadState::Load
                }),
                purpose: String::from("Move back to the previous history entry."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    if is_go_forward_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::GoForward,
                goal: String::from("Navigate forward one entry in browser history."),
                target_description: None,
            },
            selected_skill(active_skill_names, "go_forward"),
            PlannedStep {
                step_id: String::from("go-forward"),
                tool_name: ToolName::GoForward,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "steps": 1,
                    "wait_for_load_state": LoadState::Load
                }),
                purpose: String::from("Move forward to the next history entry."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    if is_reload_page_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::ReloadPage,
                goal: String::from("Reload the current page."),
                target_description: Some(String::from("current page")),
            },
            selected_skill(active_skill_names, "reload_page"),
            PlannedStep {
                step_id: String::from("reload-page"),
                tool_name: ToolName::ReloadPage,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "mode": "Standard",
                    "wait_for_load_state": LoadState::Load
                }),
                purpose: String::from("Reload the current page and wait for it to finish loading."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    if is_read_next_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::ReadNext,
                goal: String::from("Read the next narration region."),
                target_description: Some(String::from("next narration region")),
            },
            selected_skill(active_skill_names, "read_next"),
            PlannedStep {
                step_id: String::from("read-next-region"),
                tool_name: ToolName::ReadNextRegion,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "interruption_mode": "Interrupt"
                }),
                purpose: String::from("Move narration to the next region and start reading it."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    if is_read_previous_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::ReadPrevious,
                goal: String::from("Read the previous narration region."),
                target_description: Some(String::from("previous narration region")),
            },
            selected_skill(active_skill_names, "read_previous"),
            PlannedStep {
                step_id: String::from("read-previous-region"),
                tool_name: ToolName::ReadPreviousRegion,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "interruption_mode": "Interrupt"
                }),
                purpose: String::from(
                    "Move narration to the previous region and start reading it.",
                ),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    if is_stop_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::Stop,
                goal: String::from("Stop current speech output."),
                target_description: Some(String::from("speech output")),
            },
            selected_stop_skill(active_skill_names),
            PlannedStep {
                step_id: String::from("stop-speaking"),
                tool_name: ToolName::StopSpeaking,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null
                }),
                purpose: String::from("Stop any current spoken narration or playback."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    None
}

pub(crate) fn resolve_direct_voice_input_command(
    transcript: &str,
    request_id: &str,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() {
        return None;
    }

    if is_start_listening_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::StartListening,
                goal: String::from("Start listening for voice input."),
                target_description: Some(String::from("voice input")),
            },
            selected_skill(active_skill_names, "start_listening"),
            PlannedStep {
                step_id: String::from("start-listening"),
                tool_name: ToolName::StartListening,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null
                }),
                purpose: String::from("Start listening for the next spoken command."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    if is_stop_listening_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::StopListening,
                goal: String::from("Stop listening for voice input."),
                target_description: Some(String::from("voice input")),
            },
            selected_skill(active_skill_names, "stop_listening"),
            PlannedStep {
                step_id: String::from("stop-listening"),
                tool_name: ToolName::StopListening,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null
                }),
                purpose: String::from("Stop active voice listening."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    if is_transcribe_command_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::TranscribeCommand,
                goal: String::from("Capture and transcribe a short spoken command."),
                target_description: Some(String::from("spoken command")),
            },
            selected_skill(active_skill_names, "transcribe_command"),
            PlannedStep {
                step_id: String::from("transcribe-command"),
                tool_name: ToolName::TranscribeCommand,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "max_duration_ms": serde_json::Value::Null,
                    "stop_mode": "AutoStop"
                }),
                purpose: String::from("Capture and transcribe a bounded spoken command."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    None
}

pub(crate) fn resolve_direct_open_url_command(
    transcript: &str,
    request_id: &str,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let url = parse_direct_open_url_target(transcript)?;

    Some(build_single_step_planner_output(
        IntentSummary {
            name: IntentName::OpenUrl,
            goal: String::from("Open the requested URL."),
            target_description: Some(url.clone()),
        },
        selected_skill(active_skill_names, "open_url"),
        PlannedStep {
            step_id: String::from("open-url"),
            tool_name: ToolName::OpenUrl,
            arguments: serde_json::json!({
                "request_id": request_id,
                "timeout_ms": serde_json::Value::Null,
                "url": url,
                "wait_for_load_state": LoadState::Load
            }),
            purpose: String::from("Open the requested URL and wait for the page to load."),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
    ))
}

pub(crate) fn resolve_direct_read_page_command(
    transcript: &str,
    request_id: &str,
    current_page: Option<&PageModel>,
    agent_state: &AgentStateData,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() || !is_read_page_phrase(&normalized) {
        return None;
    }

    let selected_skills = selected_skill(active_skill_names, "read_page");

    if agent_state.page_id.is_none() {
        let summary = String::from("There is no current page to read yet.");
        return Some(PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::ReadPage,
                goal: String::from("Read the current page from the beginning."),
                target_description: Some(String::from("current page")),
            },
            selected_skills,
            steps: vec![PlannedStep {
                step_id: String::from("report-missing-page"),
                tool_name: ToolName::ReportResult,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "status": ReportStatus::NeedsFollowUp,
                    "summary": summary.clone(),
                    "next_recommended_action": "Open a page first, then ask me to read it.",
                    "user_message": summary
                }),
                purpose: String::from("Report that there is no active page available to read."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        });
    }

    if let Some(region_id) = current_page.and_then(first_readable_region_id) {
        return Some(PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::ReadPage,
                goal: String::from("Read the current page from the beginning."),
                target_description: Some(String::from("current page")),
            },
            selected_skills,
            steps: vec![PlannedStep {
                step_id: String::from("read-page-from-start"),
                tool_name: ToolName::ReadRegion,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "region_id": region_id,
                    "interruption_mode": "Interrupt"
                }),
                purpose: String::from(
                    "Restart narration from the first readable region of the current page.",
                ),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        });
    }

    Some(PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::ReadPage,
            goal: String::from("Read the current page from the beginning."),
            target_description: Some(String::from("current page")),
        },
        selected_skills,
        steps: vec![
            PlannedStep {
                step_id: String::from("extract-page-for-reading"),
                tool_name: ToolName::ExtractPageModel,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "use_dom_extraction": true,
                    "include_headings": true,
                    "include_links": true
                }),
                purpose: String::from("Refresh the readable page model before starting narration."),
                on_success: StepTransition::NextStep {
                    step_id: String::from("read-first-region"),
                },
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from("read-first-region"),
                tool_name: ToolName::ReadNextRegion,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "interruption_mode": "Interrupt"
                }),
                purpose: String::from(
                    "Start narration from the first readable region of the refreshed page.",
                ),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    })
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

pub(crate) fn parse_direct_focus_field_command(transcript: &str) -> Option<FocusFieldCommand> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() || !is_focus_field_phrase(&normalized) {
        return None;
    }

    let mut remainder = normalized.as_str();
    if let Some(stripped) = remainder.strip_prefix("focus ") {
        remainder = stripped;
    }
    if let Some(stripped) = remainder.strip_prefix("on ") {
        remainder = stripped;
    }
    if let Some(stripped) = remainder.strip_prefix("the ") {
        remainder = stripped;
    }
    if let Some(stripped) = remainder.strip_prefix("my ") {
        remainder = stripped;
    }
    if let Some(stripped) = remainder.strip_prefix("field ") {
        remainder = stripped;
    }
    if let Some(stripped) = remainder.strip_suffix(" field") {
        remainder = stripped;
    }

    let description = remainder.trim();
    Some(FocusFieldCommand {
        description: (!description.is_empty() && description != "field")
            .then(|| description.to_string()),
    })
}

pub(crate) fn parse_direct_fill_field_command(transcript: &str) -> Option<FillFieldCommand> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty()
        || !is_fill_input_phrase(&normalized)
        || is_focus_field_phrase(&normalized)
        || is_fill_and_submit_phrase(&normalized)
    {
        return None;
    }

    let collapsed = collapse_transcript_whitespace(transcript);
    if collapsed.is_empty() {
        return Some(FillFieldCommand {
            description: None,
            text: None,
        });
    }

    if let Some((description, text)) = parse_fill_with_pattern(&collapsed) {
        return Some(FillFieldCommand {
            description,
            text: Some(text),
        });
    }

    if let Some((text, description)) = parse_into_field_pattern(&collapsed) {
        return Some(FillFieldCommand {
            description,
            text: Some(text),
        });
    }

    Some(FillFieldCommand {
        description: parse_fill_field_description_only(&collapsed),
        text: None,
    })
}

pub(crate) fn parse_fill_field_correction_command(
    transcript: &str,
) -> Option<FillFieldCorrectionCommand> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() || !is_fill_input_phrase(&normalized) {
        return None;
    }

    if normalized.contains("the other field") {
        return Some(FillFieldCorrectionCommand::AlternateField);
    }

    let collapsed = collapse_transcript_whitespace(transcript);
    if collapsed.is_empty() {
        return None;
    }

    let lowered = collapsed.to_ascii_lowercase();
    for prefix in ["put ", "type ", "enter "] {
        let suffix = " there instead";
        if lowered.starts_with(prefix) && lowered.ends_with(suffix) {
            let end_index = collapsed.len().saturating_sub(suffix.len());
            let value = collapsed.get(prefix.len()..end_index)?.trim();
            let text = normalize_fill_value(value)?;
            return Some(FillFieldCorrectionCommand::ReplaceValue { text });
        }
    }

    None
}

pub(crate) fn parse_direct_fill_and_submit_command(transcript: &str) -> Option<FillFieldCommand> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() || !is_fill_and_submit_phrase(&normalized) {
        return None;
    }

    let collapsed = collapse_transcript_whitespace(transcript);
    let fill_portion = strip_fill_and_submit_suffix(&collapsed)?;
    parse_direct_fill_field_command(&fill_portion)
}

pub(crate) fn is_direct_submit_form_command(transcript: &str) -> bool {
    let normalized = normalize_transcript_for_routing(transcript);
    !normalized.is_empty() && is_submit_form_phrase(&normalized)
}

fn parse_fill_with_pattern(transcript: &str) -> Option<(Option<String>, String)> {
    let lowered = transcript.to_ascii_lowercase();
    let prefix = if lowered.starts_with("fill in ") {
        "fill in "
    } else if lowered.starts_with("fill ") {
        "fill "
    } else {
        return None;
    };

    let remainder = transcript.get(prefix.len()..)?.trim();
    let (field_target, text) = split_case_insensitive_once(remainder, " with ")?;
    let text = normalize_fill_value(text)?;
    Some((normalize_field_target(field_target), text))
}

fn parse_into_field_pattern(transcript: &str) -> Option<(String, Option<String>)> {
    let lowered = transcript.to_ascii_lowercase();
    let (prefix, separator) = if lowered.starts_with("type ") && lowered.contains(" into ") {
        ("type ", " into ")
    } else if lowered.starts_with("enter ") && lowered.contains(" into ") {
        ("enter ", " into ")
    } else if lowered.starts_with("enter ") && lowered.contains(" in ") {
        ("enter ", " in ")
    } else if lowered.starts_with("put ") && lowered.contains(" in ") {
        ("put ", " in ")
    } else {
        return None;
    };

    let remainder = transcript.get(prefix.len()..)?.trim();
    let (text, field_target) = split_case_insensitive_once(remainder, separator)?;
    let text = normalize_fill_value(text)?;
    Some((text, normalize_field_target(field_target)))
}

fn parse_fill_field_description_only(transcript: &str) -> Option<String> {
    let lowered = transcript.to_ascii_lowercase();
    let remainder = if lowered.starts_with("fill in ") {
        transcript.get("fill in ".len()..)?
    } else if lowered.starts_with("fill ") {
        transcript.get("fill ".len()..)?
    } else if lowered.starts_with("type into ") {
        transcript.get("type into ".len()..)?
    } else if lowered.starts_with("enter into ") {
        transcript.get("enter into ".len()..)?
    } else if lowered.starts_with("enter in ") {
        transcript.get("enter in ".len()..)?
    } else if lowered.starts_with("put in ") {
        transcript.get("put in ".len()..)?
    } else {
        return None;
    };

    normalize_field_target(remainder)
}

fn strip_fill_and_submit_suffix(transcript: &str) -> Option<String> {
    let normalized = collapse_transcript_whitespace(transcript);
    if normalized.is_empty() {
        return None;
    }

    let lowered = normalized.to_ascii_lowercase();
    for suffix in [
        " and then submit this form",
        " and then submit form",
        " and then send this form",
        " and then send form",
        " and then press submit",
        " and then hit submit",
        " and then submit",
        " then submit this form",
        " then submit form",
        " then send this form",
        " then send form",
        " then press submit",
        " then hit submit",
        " then submit",
        " and submit this form",
        " and submit form",
        " and send this form",
        " and send form",
        " and press submit",
        " and hit submit",
        " and submit",
    ] {
        if lowered.ends_with(suffix) {
            let trimmed = normalized
                .get(..normalized.len().saturating_sub(suffix.len()))?
                .trim();
            return (!trimmed.is_empty()).then(|| trimmed.to_string());
        }
    }

    None
}

fn split_case_insensitive_once<'a>(text: &'a str, separator: &str) -> Option<(&'a str, &'a str)> {
    let lowered = text.to_ascii_lowercase();
    let index = lowered.find(separator)?;
    let before = text.get(..index)?.trim();
    let after = text.get(index + separator.len()..)?.trim();
    Some((before, after))
}

fn normalize_field_target(target: &str) -> Option<String> {
    let mut normalized = collapse_transcript_whitespace(target);
    if normalized.is_empty() {
        return None;
    }

    let lowered = normalized.to_ascii_lowercase();
    for prefix in ["the ", "my ", "a ", "an "] {
        if lowered.starts_with(prefix) {
            normalized = normalized.get(prefix.len()..)?.trim().to_string();
            break;
        }
    }

    for suffix in [" field", " textbox", " text box", " input", " input box"] {
        if normalized.to_ascii_lowercase().ends_with(suffix) {
            let end = normalized.len().saturating_sub(suffix.len());
            normalized = normalized.get(..end)?.trim().to_string();
            break;
        }
    }

    let normalized = normalized.trim();
    (!normalized.is_empty()).then(|| normalized.to_string())
}

fn normalize_fill_value(value: &str) -> Option<String> {
    let collapsed = collapse_transcript_whitespace(value);
    let trimmed = collapsed.trim();
    if trimmed.is_empty() {
        return None;
    }

    let unquoted = if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        trimmed
            .get(1..trimmed.len().saturating_sub(1))
            .unwrap_or(trimmed)
    } else {
        trimmed
    };

    let cleaned = unquoted.trim();
    (!cleaned.is_empty()).then(|| cleaned.to_string())
}

fn parse_browser_visibility_command(
    normalized: &str,
    current_visibility: BrowserVisibilityMode,
) -> Option<BrowserVisibilityMode> {
    if normalized.contains("hide browser")
        || normalized.contains("hide the browser")
        || normalized.contains("go headless")
        || normalized.contains("make browser headless")
        || normalized.contains("make the browser headless")
        || normalized.contains("make it headless")
        || normalized.contains("switch to headless")
        || normalized.contains("switch browser to headless")
        || normalized.contains("headless mode")
    {
        return Some(BrowserVisibilityMode::Headless);
    }

    if normalized.contains("show browser")
        || normalized.contains("show the browser")
        || normalized.contains("make browser visible")
        || normalized.contains("make the browser visible")
        || normalized.contains("make it visible")
        || normalized.contains("switch to visible")
        || normalized.contains("switch browser to visible")
        || normalized.contains("visible mode")
        || normalized.contains("show the window")
    {
        return Some(BrowserVisibilityMode::Visible);
    }

    if normalized.contains("toggle browser visibility") || normalized.contains("toggle visibility")
    {
        return Some(match current_visibility {
            BrowserVisibilityMode::Visible => BrowserVisibilityMode::Headless,
            BrowserVisibilityMode::Headless => BrowserVisibilityMode::Visible,
        });
    }

    None
}

fn normalized_optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(String::from)
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

