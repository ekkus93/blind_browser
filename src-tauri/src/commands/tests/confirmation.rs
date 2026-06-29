use super::*;

#[test]
fn dispatches_get_agent_state_without_last_transcript() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-5"),
        tool_name: ToolName::GetAgentState,
        arguments: serde_json::json!({
            "request_id": "req-5",
            "timeout_ms": 1000,
            "include_last_transcript": false
        }),
        purpose: String::from("read agent state"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    let data = result.data.expect("agent state should serialize");
    assert_eq!(data.get("last_transcript"), Some(&serde_json::Value::Null));
    assert_eq!(
        data.get("last_tool_call")
            .and_then(|entry| entry.get("tool_name")),
        Some(&serde_json::Value::String(String::from("GetAgentState")))
    );
}

#[test]
fn dispatches_confirm_action_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-confirm"),
        tool_name: ToolName::ConfirmAction,
        arguments: serde_json::json!({
            "request_id": "req-confirm-dispatch",
            "timeout_ms": 1000,
            "prompt_text": "Do you want me to continue?",
            "reason": "The next step may submit data."
        }),
        purpose: String::from("request confirmation"),
        on_success: StepTransition::RequestConfirmation,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor.last_confirmation_prompt.as_deref(),
        Some("Do you want me to continue?")
    );
    let data = result.data.expect("confirm_action should serialize");
    assert_eq!(
        data.get("confirmation_id"),
        Some(&serde_json::Value::String(String::from("confirm-1")))
    );
}

#[test]
fn dispatches_report_result_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-report"),
        tool_name: ToolName::ReportResult,
        arguments: serde_json::json!({
            "request_id": "req-report",
            "timeout_ms": 1000,
            "status": "Success",
            "summary": "Opened the requested page.",
            "next_recommended_action": null,
            "user_message": "The page is ready."
        }),
        purpose: String::from("report completion"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(result.tool_name, ToolName::ReportResult);
    assert_eq!(
        executor.last_report_result,
        Some(ReportResultData {
            status: ReportStatus::Success,
            summary: String::from("Opened the requested page."),
            next_recommended_action: None,
            user_message: Some(String::from("The page is ready.")),
        })
    );
    let data = result.data.expect("report_result should serialize");
    assert_eq!(
        data.get("status"),
        Some(&serde_json::Value::String(String::from("Success")))
    );
}

#[test]
fn resumes_confirmed_pending_execution_from_stored_steps() {
    let mut executor = MockExecutor::default();
    let pending_plan_execution = PendingPlanExecutionState {
        request_id: String::from("req-resume"),
        intent_name: IntentName::SetBrowserVisibility,
        selected_skills: vec![String::from("confirm_action")],
        confirmation_id: String::from("confirm-1"),
        prompt_text: String::from("Proceed?"),
        next_step_id: Some(String::from("step-2")),
        queued_step_ids: vec![String::from("step-2")],
        queued_steps: vec![PlannedStep {
            step_id: String::from("step-2"),
            tool_name: ToolName::SetBrowserVisibility,
            arguments: serde_json::json!({
                "request_id": "req-resume",
                "timeout_ms": 1000,
                "mode": "Headless"
            }),
            purpose: String::from("apply confirmed action"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
    };

    let outcome =
        resume_after_confirmation(&mut executor, &pending_plan_execution, "confirm-1", true);

    match outcome {
        ExecutionOutcome::Complete { trace } => {
            assert_eq!(trace.executed_step_ids, vec!["step-2"]);
            assert_eq!(
                executor.last_visibility,
                Some(BrowserVisibilityMode::Headless)
            );
        }
        other => panic!("expected complete outcome after resume, got {other:?}"),
    }
}

#[test]
fn resumes_rejected_confirmation_to_replan() {
    let mut executor = MockExecutor::default();
    let pending_plan_execution = PendingPlanExecutionState {
        request_id: String::from("req-resume"),
        intent_name: IntentName::SetBrowserVisibility,
        selected_skills: vec![String::from("confirm_action")],
        confirmation_id: String::from("confirm-1"),
        prompt_text: String::from("Proceed?"),
        next_step_id: Some(String::from("step-2")),
        queued_step_ids: vec![String::from("step-2")],
        queued_steps: vec![PlannedStep {
            step_id: String::from("step-2"),
            tool_name: ToolName::SetBrowserVisibility,
            arguments: serde_json::json!({
                "request_id": "req-resume",
                "timeout_ms": 1000,
                "mode": "Headless"
            }),
            purpose: String::from("apply confirmed action"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
    };

    let outcome =
        resume_after_confirmation(&mut executor, &pending_plan_execution, "confirm-1", false);

    match outcome {
        ExecutionOutcome::NeedsReplan { trace } => {
            assert!(trace.executed_step_ids.is_empty());
            assert_eq!(executor.last_visibility, None);
        }
        other => panic!("expected replan outcome after rejection, got {other:?}"),
    }
}

#[test]
fn rejects_resume_with_mismatched_confirmation_id() {
    let mut executor = MockExecutor::default();
    let pending_plan_execution = PendingPlanExecutionState {
        request_id: String::from("req-resume"),
        intent_name: IntentName::SetBrowserVisibility,
        selected_skills: vec![String::from("confirm_action")],
        confirmation_id: String::from("confirm-1"),
        prompt_text: String::from("Proceed?"),
        next_step_id: Some(String::from("step-2")),
        queued_step_ids: vec![String::from("step-2")],
        queued_steps: vec![PlannedStep {
            step_id: String::from("step-2"),
            tool_name: ToolName::SetBrowserVisibility,
            arguments: serde_json::json!({
                "request_id": "req-resume",
                "timeout_ms": 1000,
                "mode": "Headless"
            }),
            purpose: String::from("apply confirmed action"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
    };

    let outcome = resume_after_confirmation(
        &mut executor,
        &pending_plan_execution,
        "wrong-confirmation-id",
        true,
    );

    match outcome {
        ExecutionOutcome::Aborted { trace, error } => {
            assert!(trace.executed_step_ids.is_empty());
            assert_eq!(error.code, "confirmation_id_mismatch");
            assert_eq!(executor.last_visibility, None);
        }
        other => panic!("expected aborted outcome after mismatch, got {other:?}"),
    }
}
