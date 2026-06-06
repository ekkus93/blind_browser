use super::*;

#[test]
fn dispatches_set_playback_volume_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-1"),
        tool_name: ToolName::SetPlaybackVolume,
        arguments: serde_json::json!({
            "request_id": "req-1",
            "timeout_ms": 1000,
            "volume": 0.4
        }),
        purpose: String::from("update volume"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(executor.last_volume, Some(0.4));
    assert_eq!(result.tool_name, ToolName::SetPlaybackVolume);
    assert_eq!(result.request_id, "req-1");
    let data = result
        .data
        .expect("serialized tool result data should exist");
    assert_eq!(data.get("muted"), Some(&serde_json::Value::Bool(false)));
    assert_eq!(data.get("changed"), Some(&serde_json::Value::Bool(true)));
    let playback_volume = data
        .get("playback_volume")
        .and_then(serde_json::Value::as_f64)
        .expect("playback_volume should be serialized as a number");
    assert!((playback_volume - 0.4).abs() < 0.000_001);
}

#[test]
fn dispatches_open_url_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-open-url"),
        tool_name: ToolName::OpenUrl,
        arguments: serde_json::json!({
            "request_id": "req-open-url",
            "timeout_ms": 1000,
            "url": "https://example.com/article",
            "wait_for_load_state": "NetworkIdle"
        }),
        purpose: String::from("navigate to a page"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor.last_open_url.as_deref(),
        Some("https://example.com/article")
    );
    let data = result.data.expect("open_url should serialize");
    assert_eq!(
        data.get("final_url"),
        Some(&serde_json::Value::String(String::from(
            "https://example.com/article"
        )))
    );
    assert_eq!(
        data.get("load_state"),
        Some(&serde_json::Value::String(String::from("NetworkIdle")))
    );
}

#[test]
fn dispatches_go_back_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-go-back"),
        tool_name: ToolName::GoBack,
        arguments: serde_json::json!({
            "request_id": "req-go-back",
            "timeout_ms": 1000,
            "steps": 2,
            "wait_for_load_state": "Load"
        }),
        purpose: String::from("go back in history"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_go_back_request
            .as_ref()
            .and_then(|input| input.steps),
        Some(2)
    );
}

#[test]
fn dispatches_go_forward_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-go-forward"),
        tool_name: ToolName::GoForward,
        arguments: serde_json::json!({
            "request_id": "req-go-forward",
            "timeout_ms": 1000,
            "steps": 1,
            "wait_for_load_state": "NetworkIdle"
        }),
        purpose: String::from("go forward in history"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_go_forward_request
            .as_ref()
            .and_then(|input| input.steps),
        Some(1)
    );
}

#[test]
fn dispatches_reload_page_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-reload"),
        tool_name: ToolName::ReloadPage,
        arguments: serde_json::json!({
            "request_id": "req-reload",
            "timeout_ms": 1000,
            "mode": "Hard",
            "wait_for_load_state": "Load"
        }),
        purpose: String::from("reload the current page"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_reload_request
            .as_ref()
            .map(|input| input.mode),
        Some(ReloadMode::Hard)
    );
}
