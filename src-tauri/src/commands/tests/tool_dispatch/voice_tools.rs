use super::*;

#[test]
fn dispatches_start_listening_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-start-listening"),
        tool_name: ToolName::StartListening,
        arguments: serde_json::json!({
            "request_id": "req-start-listening",
            "timeout_ms": 1500
        }),
        purpose: String::from("start listening"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_start_listening_request
            .as_ref()
            .map(|input| input.request_id.as_str()),
        Some("req-start-listening")
    );
}

#[test]
fn dispatches_stop_listening_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-stop-listening"),
        tool_name: ToolName::StopListening,
        arguments: serde_json::json!({
            "request_id": "req-stop-listening"
        }),
        purpose: String::from("stop listening"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_stop_listening_request
            .as_ref()
            .map(|input| input.request_id.as_str()),
        Some("req-stop-listening")
    );
}

#[test]
fn dispatches_transcribe_command_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-transcribe-command"),
        tool_name: ToolName::TranscribeCommand,
        arguments: serde_json::json!({
            "request_id": "req-transcribe-command",
            "timeout_ms": 2000,
            "max_duration_ms": 3000,
            "stop_mode": "AutoStop"
        }),
        purpose: String::from("transcribe a command"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_transcribe_command_request
            .as_ref()
            .map(|input| input.request_id.as_str()),
        Some("req-transcribe-command")
    );
    assert_eq!(
        executor
            .last_transcribe_command_request
            .as_ref()
            .and_then(|input| input.max_duration_ms),
        Some(3000)
    );
}
