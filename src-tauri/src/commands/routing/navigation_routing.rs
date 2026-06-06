use super::*;

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
