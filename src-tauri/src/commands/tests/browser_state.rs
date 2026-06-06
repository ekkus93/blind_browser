use super::*;

#[test]
fn set_browser_visibility_reports_no_change_when_mode_is_already_active() {
    let mut executor = MockExecutor {
        browser_visibility: BrowserVisibilityMode::Headless,
        ..Default::default()
    };

    let result = executor.execute_set_browser_visibility(SetBrowserVisibilityInput {
        request_id: String::from("req-visibility-noop"),
        timeout_ms: Some(1_000),
        mode: BrowserVisibilityMode::Headless,
    });

    assert!(result.ok);
    assert_eq!(
        result.data.expect("visibility tool should return data"),
        SetBrowserVisibilityData {
            mode: BrowserVisibilityMode::Headless,
            changed: false,
            supported: true,
        }
    );
    assert_eq!(
        result.observations,
        vec![String::from(
            "Browser visibility mode is already set to the requested value.",
        )]
    );
    assert_eq!(
        executor.current_browser_visibility(),
        BrowserVisibilityMode::Headless
    );
}

#[test]
fn set_browser_visibility_reports_unsupported_when_switching_is_disabled() {
    let mut executor = MockExecutor {
        browser_visibility_switch_supported: false,
        ..Default::default()
    };

    let result = executor.execute_set_browser_visibility(SetBrowserVisibilityInput {
        request_id: String::from("req-visibility-unsupported"),
        timeout_ms: Some(1_000),
        mode: BrowserVisibilityMode::Headless,
    });

    assert!(result.ok);
    assert_eq!(
        result
            .data
            .expect("unsupported visibility tool should return data"),
        SetBrowserVisibilityData {
            mode: BrowserVisibilityMode::Visible,
            changed: false,
            supported: false,
        }
    );
    assert_eq!(
        result.observations,
        vec![String::from(
            "Browser visibility switching is not supported in this build.",
        )]
    );

    let state = executor.execute_get_agent_state(GetAgentStateInput {
        request_id: String::from("req-visibility-state"),
        timeout_ms: Some(1_000),
        include_last_transcript: false,
    });
    assert!(state.ok);
    let state_data = state.data.expect("agent state should return data");
    assert_eq!(
        state_data.browser_visibility,
        BrowserVisibilityMode::Visible
    );
}

#[test]
fn browser_visibility_changes_are_reflected_in_following_state_reads() {
    let mut executor = MockExecutor::default();
    let set_visibility_step = PlannedStep {
        step_id: String::from("step-visibility"),
        tool_name: ToolName::SetBrowserVisibility,
        arguments: serde_json::json!({
            "request_id": "req-visibility",
            "timeout_ms": 1000,
            "mode": "Headless"
        }),
        purpose: String::from("toggle browser visibility"),
        on_success: StepTransition::NextStep {
            step_id: String::from("step-runtime"),
        },
        on_failure: StepTransition::Replan,
    };
    let runtime_status_step = PlannedStep {
        step_id: String::from("step-runtime"),
        tool_name: ToolName::GetRuntimeStatus,
        arguments: serde_json::json!({
            "request_id": "req-runtime",
            "timeout_ms": 1000,
            "include_provider_modes": false
        }),
        purpose: String::from("read runtime status"),
        on_success: StepTransition::NextStep {
            step_id: String::from("step-agent"),
        },
        on_failure: StepTransition::Replan,
    };
    let agent_state_step = PlannedStep {
        step_id: String::from("step-agent"),
        tool_name: ToolName::GetAgentState,
        arguments: serde_json::json!({
            "request_id": "req-agent",
            "timeout_ms": 1000,
            "include_last_transcript": true
        }),
        purpose: String::from("read agent state"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let set_visibility_result = execute_planned_step(&mut executor, &set_visibility_step);
    let runtime_status_result = execute_planned_step(&mut executor, &runtime_status_step);
    let agent_state_result = execute_planned_step(&mut executor, &agent_state_step);

    assert!(set_visibility_result.ok);
    assert!(runtime_status_result.ok);
    assert!(agent_state_result.ok);
    assert_eq!(
        runtime_status_result
            .data
            .as_ref()
            .and_then(|data| data.get("browser_visibility")),
        Some(&serde_json::json!("Headless"))
    );
    assert_eq!(
        agent_state_result
            .data
            .as_ref()
            .and_then(|data| data.get("browser_visibility")),
        Some(&serde_json::json!("Headless"))
    );
}

#[test]
fn browser_history_navigation_updates_following_state_reads() {
    let mut executor = MockExecutor::default();
    let go_back_step = PlannedStep {
        step_id: String::from("step-go-back"),
        tool_name: ToolName::GoBack,
        arguments: serde_json::json!({
            "request_id": "req-go-back",
            "timeout_ms": 1000,
            "steps": 1,
            "wait_for_load_state": "Load"
        }),
        purpose: String::from("go back in history"),
        on_success: StepTransition::NextStep {
            step_id: String::from("step-runtime-after-back"),
        },
        on_failure: StepTransition::Replan,
    };
    let runtime_after_back_step = PlannedStep {
        step_id: String::from("step-runtime-after-back"),
        tool_name: ToolName::GetRuntimeStatus,
        arguments: serde_json::json!({
            "request_id": "req-runtime-after-back",
            "timeout_ms": 1000,
            "include_provider_modes": false
        }),
        purpose: String::from("read runtime status"),
        on_success: StepTransition::NextStep {
            step_id: String::from("step-go-forward"),
        },
        on_failure: StepTransition::Replan,
    };
    let go_forward_step = PlannedStep {
        step_id: String::from("step-go-forward"),
        tool_name: ToolName::GoForward,
        arguments: serde_json::json!({
            "request_id": "req-go-forward",
            "timeout_ms": 1000,
            "steps": 1,
            "wait_for_load_state": "Load"
        }),
        purpose: String::from("go forward in history"),
        on_success: StepTransition::NextStep {
            step_id: String::from("step-runtime-after-forward"),
        },
        on_failure: StepTransition::Replan,
    };
    let runtime_after_forward_step = PlannedStep {
        step_id: String::from("step-runtime-after-forward"),
        tool_name: ToolName::GetRuntimeStatus,
        arguments: serde_json::json!({
            "request_id": "req-runtime-after-forward",
            "timeout_ms": 1000,
            "include_provider_modes": false
        }),
        purpose: String::from("read runtime status"),
        on_success: StepTransition::NextStep {
            step_id: String::from("step-reload"),
        },
        on_failure: StepTransition::Replan,
    };
    let reload_step = PlannedStep {
        step_id: String::from("step-reload"),
        tool_name: ToolName::ReloadPage,
        arguments: serde_json::json!({
            "request_id": "req-reload",
            "timeout_ms": 1000,
            "mode": "Hard",
            "wait_for_load_state": "Load"
        }),
        purpose: String::from("reload the current page"),
        on_success: StepTransition::NextStep {
            step_id: String::from("step-agent-after-reload"),
        },
        on_failure: StepTransition::Replan,
    };
    let agent_after_reload_step = PlannedStep {
        step_id: String::from("step-agent-after-reload"),
        tool_name: ToolName::GetAgentState,
        arguments: serde_json::json!({
            "request_id": "req-agent-after-reload",
            "timeout_ms": 1000,
            "include_last_transcript": false
        }),
        purpose: String::from("read agent state"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let go_back_result = execute_planned_step(&mut executor, &go_back_step);
    let runtime_after_back_result = execute_planned_step(&mut executor, &runtime_after_back_step);
    let go_forward_result = execute_planned_step(&mut executor, &go_forward_step);
    let runtime_after_forward_result =
        execute_planned_step(&mut executor, &runtime_after_forward_step);
    let reload_result = execute_planned_step(&mut executor, &reload_step);
    let agent_after_reload_result = execute_planned_step(&mut executor, &agent_after_reload_step);

    assert!(go_back_result.ok);
    assert!(runtime_after_back_result.ok);
    assert!(go_forward_result.ok);
    assert!(runtime_after_forward_result.ok);
    assert!(reload_result.ok);
    assert!(agent_after_reload_result.ok);
    assert_eq!(
        runtime_after_back_result
            .data
            .as_ref()
            .and_then(|data| data.get("browser_history"))
            .and_then(|history| history.get("can_go_back")),
        Some(&serde_json::json!(false))
    );
    assert_eq!(
        runtime_after_back_result
            .data
            .as_ref()
            .and_then(|data| data.get("browser_history"))
            .and_then(|history| history.get("can_go_forward")),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        runtime_after_forward_result
            .data
            .as_ref()
            .and_then(|data| data.get("browser_history"))
            .and_then(|history| history.get("can_go_back")),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        runtime_after_forward_result
            .data
            .as_ref()
            .and_then(|data| data.get("browser_history"))
            .and_then(|history| history.get("can_go_forward")),
        Some(&serde_json::json!(false))
    );
    assert_eq!(
        agent_after_reload_result
            .data
            .as_ref()
            .and_then(|data| data.get("browser_history"))
            .and_then(|history| history.get("current_entry_index")),
        Some(&serde_json::json!(1))
    );
    assert_eq!(
        agent_after_reload_result
            .data
            .as_ref()
            .and_then(|data| data.get("browser_history"))
            .and_then(|history| history.get("entry_count")),
        Some(&serde_json::json!(2))
    );
}

