use super::*;

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
