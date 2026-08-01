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

fn visibility_step(step_id: &str, mode: &str) -> PlannedStep {
    PlannedStep {
        step_id: step_id.to_string(),
        tool_name: ToolName::SetBrowserVisibility,
        arguments: serde_json::json!({
            "request_id": "req-resume",
            "timeout_ms": 1000,
            "mode": mode
        }),
        purpose: String::from("apply confirmed action"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    }
}

fn pending_visibility_confirmation(
    context: &ConfirmationRuntimeContext,
) -> PendingPlanExecutionState {
    let queued_steps = vec![visibility_step("step-2", "Headless")];
    let built = build_confirmation_manifest("req-resume", &queued_steps, context)
        .expect("manifest should build");
    PendingPlanExecutionState {
        request_id: String::from("req-resume"),
        intent_name: IntentName::SetBrowserVisibility,
        selected_skills: vec![String::from("confirm_action")],
        confirmation_id: String::from("confirm-1"),
        manifest_digest: built.digest,
        manifest: built.manifest,
        prompt_text: built.prompt_text,
        next_step_id: Some(String::from("step-2")),
        queued_step_ids: vec![String::from("step-2")],
        queued_steps,
    }
}

fn bound_context(now_ms: u64) -> ConfirmationRuntimeContext {
    ConfirmationRuntimeContext::at(
        Some("page-1"),
        Some("https://example.com/form?token=secret"),
        now_ms,
    )
}

#[test]
fn resumes_confirmed_pending_execution_from_stored_steps() {
    let mut executor = MockExecutor::default();
    let issued_context = bound_context(10_000);
    let pending = pending_visibility_confirmation(&issued_context);
    let resume_context = bound_context(10_001);

    let outcome = resume_after_confirmation_with_context(
        &mut executor,
        &pending,
        "confirm-1",
        &pending.manifest_digest,
        true,
        &resume_context,
    );

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
fn rejected_confirmation_does_not_execute_actions() {
    let mut executor = MockExecutor::default();
    let context = bound_context(10_000);
    let pending = pending_visibility_confirmation(&context);

    let outcome = resume_after_confirmation_with_context(
        &mut executor,
        &pending,
        "confirm-1",
        &pending.manifest_digest,
        false,
        &bound_context(10_001),
    );

    assert!(matches!(outcome, ExecutionOutcome::NeedsReplan { .. }));
    assert_eq!(executor.last_visibility, None);
}

#[test]
fn rejects_resume_with_mismatched_confirmation_id() {
    let mut executor = MockExecutor::default();
    let context = bound_context(10_000);
    let pending = pending_visibility_confirmation(&context);

    let outcome = resume_after_confirmation_with_context(
        &mut executor,
        &pending,
        "wrong-confirmation-id",
        &pending.manifest_digest,
        true,
        &bound_context(10_001),
    );

    match outcome {
        ExecutionOutcome::Aborted { error, .. } => {
            assert_eq!(error.code, "confirmation_id_mismatch");
            assert_eq!(executor.last_visibility, None);
        }
        other => panic!("expected aborted outcome after mismatch, got {other:?}"),
    }
}

#[test]
fn rejects_resume_with_mismatched_manifest_digest() {
    let mut executor = MockExecutor::default();
    let pending = pending_visibility_confirmation(&bound_context(10_000));

    let outcome = resume_after_confirmation_with_context(
        &mut executor,
        &pending,
        "confirm-1",
        "wrong-digest",
        true,
        &bound_context(10_001),
    );

    match outcome {
        ExecutionOutcome::Aborted { error, .. } => {
            assert_eq!(error.code, "confirmation_digest_mismatch");
            assert_eq!(executor.last_visibility, None);
        }
        other => panic!("expected digest mismatch, got {other:?}"),
    }
}

#[test]
fn changing_a_queued_argument_invalidates_confirmation() {
    let mut executor = MockExecutor::default();
    let mut pending = pending_visibility_confirmation(&bound_context(10_000));
    pending.queued_steps[0].arguments["mode"] = serde_json::json!("Visible");

    let outcome = resume_after_confirmation_with_context(
        &mut executor,
        &pending,
        "confirm-1",
        &pending.manifest_digest,
        true,
        &bound_context(10_001),
    );

    match outcome {
        ExecutionOutcome::Aborted { error, .. } => {
            assert_eq!(error.code, "confirmation_manifest_mismatch");
            assert_eq!(executor.last_visibility, None);
        }
        other => panic!("expected manifest mismatch, got {other:?}"),
    }
}

#[test]
fn changing_a_queued_transition_invalidates_confirmation() {
    let mut executor = MockExecutor::default();
    let mut pending = pending_visibility_confirmation(&bound_context(10_000));
    pending.queued_steps[0].on_success = StepTransition::Replan;

    let outcome = resume_after_confirmation_with_context(
        &mut executor,
        &pending,
        "confirm-1",
        &pending.manifest_digest,
        true,
        &bound_context(10_001),
    );

    match outcome {
        ExecutionOutcome::Aborted { error, .. } => {
            assert_eq!(error.code, "confirmation_manifest_mismatch");
            assert_eq!(executor.last_visibility, None);
        }
        other => panic!("expected transition-bound manifest mismatch, got {other:?}"),
    }
}

#[test]
fn reordering_queued_actions_invalidates_confirmation() {
    let context = bound_context(10_000);
    let queued_steps = vec![
        visibility_step("step-2", "Headless"),
        visibility_step("step-3", "Visible"),
    ];
    let built = build_confirmation_manifest("req-resume", &queued_steps, &context)
        .expect("manifest should build");
    let mut pending = PendingPlanExecutionState {
        request_id: String::from("req-resume"),
        intent_name: IntentName::SetBrowserVisibility,
        selected_skills: vec![String::from("confirm_action")],
        confirmation_id: String::from("confirm-1"),
        manifest_digest: built.digest,
        manifest: built.manifest,
        prompt_text: built.prompt_text,
        next_step_id: Some(String::from("step-2")),
        queued_step_ids: vec![String::from("step-2"), String::from("step-3")],
        queued_steps,
    };
    pending.queued_steps.reverse();
    let mut executor = MockExecutor::default();

    let outcome = resume_after_confirmation_with_context(
        &mut executor,
        &pending,
        "confirm-1",
        &pending.manifest_digest,
        true,
        &bound_context(10_001),
    );

    match outcome {
        ExecutionOutcome::Aborted { error, .. } => {
            assert_eq!(error.code, "confirmation_queue_mismatch");
            assert_eq!(executor.last_visibility, None);
        }
        other => panic!("expected queue mismatch, got {other:?}"),
    }
}

#[test]
fn page_or_origin_change_invalidates_confirmation() {
    let pending = pending_visibility_confirmation(&bound_context(10_000));
    let cases = [
        ConfirmationRuntimeContext::at(Some("page-2"), Some("https://example.com/form"), 10_001),
        ConfirmationRuntimeContext::at(
            Some("page-1"),
            Some("https://attacker.example/form"),
            10_001,
        ),
    ];
    let expected_codes = ["confirmation_page_changed", "confirmation_origin_changed"];

    for (context, expected_code) in cases.into_iter().zip(expected_codes) {
        let mut executor = MockExecutor::default();
        let outcome = resume_after_confirmation_with_context(
            &mut executor,
            &pending,
            "confirm-1",
            &pending.manifest_digest,
            true,
            &context,
        );
        match outcome {
            ExecutionOutcome::Aborted { error, .. } => {
                assert_eq!(error.code, expected_code);
                assert_eq!(executor.last_visibility, None);
            }
            other => panic!("expected state-bound rejection, got {other:?}"),
        }
    }
}

#[test]
fn expired_confirmation_is_rejected() {
    let pending = pending_visibility_confirmation(&bound_context(10_000));
    let mut executor = MockExecutor::default();
    let expired = bound_context(pending.manifest.expires_at_ms);

    let outcome = resume_after_confirmation_with_context(
        &mut executor,
        &pending,
        "confirm-1",
        &pending.manifest_digest,
        true,
        &expired,
    );

    match outcome {
        ExecutionOutcome::Aborted { error, .. } => {
            assert_eq!(error.code, "confirmation_expired");
            assert_eq!(executor.last_visibility, None);
        }
        other => panic!("expected expired confirmation rejection, got {other:?}"),
    }
}

#[test]
fn serialized_pending_state_hides_raw_queued_arguments_and_secrets() {
    let context = bound_context(10_000);
    let queued_steps = vec![PlannedStep {
        step_id: String::from("step-secret"),
        tool_name: ToolName::TypeIntoElement,
        arguments: serde_json::json!({
            "request_id": "req-secret",
            "timeout_ms": 1000,
            "element_id": "password-field",
            "text": "super-secret-password",
            "mode": "Replace",
            "submit": "KeepEditing"
        }),
        purpose: String::from("type a password"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    }];
    let built = build_confirmation_manifest("req-secret", &queued_steps, &context)
        .expect("manifest should build");
    let pending = PendingPlanExecutionState {
        request_id: String::from("req-secret"),
        intent_name: IntentName::FillInput,
        selected_skills: vec![String::from("confirm_action")],
        confirmation_id: String::from("confirm-secret"),
        manifest_digest: built.digest,
        manifest: built.manifest,
        prompt_text: built.prompt_text,
        next_step_id: Some(String::from("step-secret")),
        queued_step_ids: vec![String::from("step-secret")],
        queued_steps,
    };

    let value = serde_json::to_value(&pending).expect("pending state should serialize");
    let encoded = value.to_string();
    assert!(value.get("queued_steps").is_none());
    assert!(!encoded.contains("super-secret-password"));
    assert!(pending.prompt_text.contains("21 characters"));
    assert!(!pending.prompt_text.contains("super-secret-password"));
}
