use super::*;

#[test]
fn rejects_invalid_tool_arguments_before_dispatch() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-2"),
        tool_name: ToolName::SetPlaybackSpeed,
        arguments: serde_json::json!({
            "request_id": "req-2",
            "timeout_ms": 1000,
            "speed": "fast"
        }),
        purpose: String::from("update speed"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(!result.ok);
    assert_eq!(result.tool_name, ToolName::SetPlaybackSpeed);
    assert_eq!(result.request_id, "req-2");
    assert_eq!(
        result.error.expect("error should be present").code,
        "invalid_tool_arguments"
    );
    assert_eq!(executor.last_speed, None);
}

#[test]
fn validate_planned_step_arguments_reports_schema_mismatch_details() {
    let step = PlannedStep {
        step_id: String::from("step-speed"),
        tool_name: ToolName::SetPlaybackSpeed,
        arguments: serde_json::json!({
            "request_id": "req-speed",
            "speed": "fast"
        }),
        purpose: String::from("set playback speed"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let error = validate_planned_step_arguments(&step)
        .expect_err("validation should reject malformed step arguments");

    assert_eq!(error.code, "invalid_planner_output");
    assert!(error.message.contains("expected schema"));
    assert_eq!(
        error
            .details
            .as_ref()
            .and_then(|details| details.get("step_id")),
        Some(&serde_json::json!("step-speed"))
    );
    assert_eq!(
        error
            .details
            .as_ref()
            .and_then(|details| details.get("tool_name")),
        Some(&serde_json::json!("SetPlaybackSpeed"))
    );
}

#[test]
fn dispatches_set_browser_visibility_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-3"),
        tool_name: ToolName::SetBrowserVisibility,
        arguments: serde_json::json!({
            "request_id": "req-3",
            "timeout_ms": 1000,
            "mode": "Headless"
        }),
        purpose: String::from("toggle browser visibility"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor.last_visibility,
        Some(BrowserVisibilityMode::Headless)
    );
    assert_eq!(result.tool_name, ToolName::SetBrowserVisibility);
}

#[test]
fn dispatches_get_runtime_status_with_provider_modes() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-4"),
        tool_name: ToolName::GetRuntimeStatus,
        arguments: serde_json::json!({
            "request_id": "req-4",
            "timeout_ms": 1000,
            "include_provider_modes": true
        }),
        purpose: String::from("read runtime status"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    let data = result.data.expect("runtime status should serialize");
    assert!(data.get("provider_modes").is_some());
}
