use super::*;

#[test]
fn listening_tools_update_following_runtime_state_reads() {
    let mut executor = MockExecutor::default();
    let start_listening_step = PlannedStep {
        step_id: String::from("step-start-listening"),
        tool_name: ToolName::StartListening,
        arguments: serde_json::json!({
            "request_id": "req-start-listening",
            "timeout_ms": 1500
        }),
        purpose: String::from("start listening"),
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
            step_id: String::from("step-stop-listening"),
        },
        on_failure: StepTransition::Replan,
    };
    let stop_listening_step = PlannedStep {
        step_id: String::from("step-stop-listening"),
        tool_name: ToolName::StopListening,
        arguments: serde_json::json!({
            "request_id": "req-stop-listening"
        }),
        purpose: String::from("stop listening"),
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

    let start_result = execute_planned_step(&mut executor, &start_listening_step);
    let runtime_status_result = execute_planned_step(&mut executor, &runtime_status_step);
    let stop_result = execute_planned_step(&mut executor, &stop_listening_step);
    let agent_state_result = execute_planned_step(&mut executor, &agent_state_step);

    assert!(start_result.ok);
    assert!(runtime_status_result.ok);
    assert!(stop_result.ok);
    assert!(agent_state_result.ok);
    assert_eq!(
        runtime_status_result
            .data
            .as_ref()
            .and_then(|data| data.get("listening_state"))
            .and_then(|state| state.get("is_listening")),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        agent_state_result
            .data
            .as_ref()
            .and_then(|data| data.get("listening_state"))
            .and_then(|state| state.get("is_listening")),
        Some(&serde_json::json!(false))
    );
}

#[test]
fn transcribe_command_updates_following_state_reads_for_auto_stop_and_manual_stop() {
    for (request_id, stop_mode, expected_listening_after_transcribe) in [
        (
            "req-transcribe-auto",
            TranscriptionStopMode::AutoStop,
            false,
        ),
        (
            "req-transcribe-manual",
            TranscriptionStopMode::KeepListening,
            true,
        ),
    ] {
        let mut executor = MockExecutor::default();
        let transcribe_step = PlannedStep {
            step_id: format!("step-{request_id}"),
            tool_name: ToolName::TranscribeCommand,
            arguments: serde_json::json!({
                "request_id": request_id,
                "timeout_ms": 2000,
                "max_duration_ms": 3000,
                "stop_mode": stop_mode
            }),
            purpose: String::from("transcribe a command"),
            on_success: StepTransition::NextStep {
                step_id: String::from("step-runtime"),
            },
            on_failure: StepTransition::Replan,
        };
        let runtime_status_step = PlannedStep {
            step_id: String::from("step-runtime"),
            tool_name: ToolName::GetRuntimeStatus,
            arguments: serde_json::json!({
                "request_id": format!("{request_id}-runtime"),
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
                "request_id": format!("{request_id}-agent"),
                "timeout_ms": 1000,
                "include_last_transcript": true
            }),
            purpose: String::from("read agent state"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let transcribe_result = execute_planned_step(&mut executor, &transcribe_step);
        let runtime_status_result = execute_planned_step(&mut executor, &runtime_status_step);
        let agent_state_result = execute_planned_step(&mut executor, &agent_state_step);

        assert!(transcribe_result.ok);
        assert!(runtime_status_result.ok);
        assert!(agent_state_result.ok);
        assert_eq!(
            runtime_status_result
                .data
                .as_ref()
                .and_then(|data| data.get("listening_state"))
                .and_then(|state| state.get("is_listening")),
            Some(&serde_json::json!(expected_listening_after_transcribe))
        );
        assert_eq!(
            agent_state_result
                .data
                .as_ref()
                .and_then(|data| data.get("listening_state"))
                .and_then(|state| state.get("is_listening")),
            Some(&serde_json::json!(expected_listening_after_transcribe))
        );
        assert_eq!(
            agent_state_result
                .data
                .as_ref()
                .and_then(|data| data.get("last_transcript")),
            Some(&serde_json::json!("read the next section"))
        );
    }
}

