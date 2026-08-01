use super::*;

fn safety(allow_click_without_confirmation: bool) -> PlannerSafetySettings {
    PlannerSafetySettings {
        confirmation_confidence_threshold: 0.85,
        allow_click_without_confirmation,
        always_confirm_submit: true,
    }
}

fn output(status: PlannerStatus, intent: IntentName, steps: Vec<PlannedStep>) -> PlannerOutput {
    let needs_confirmation = status == PlannerStatus::NeedsConfirmation;
    PlannerOutput {
        status,
        intent: IntentSummary {
            name: intent,
            goal: String::from("security policy regression test"),
            target_description: None,
        },
        selected_skills: Vec::new(),
        steps,
        requires_confirmation: needs_confirmation,
        confirmation_reason: needs_confirmation
            .then(|| String::from("deterministic runtime policy requires confirmation")),
        blocked_reason: None,
        user_message: needs_confirmation
            .then(|| String::from("Please confirm the protected action.")),
    }
}

fn confirm_step(on_failure: StepTransition) -> PlannedStep {
    PlannedStep {
        step_id: String::from("confirm"),
        tool_name: ToolName::ConfirmAction,
        arguments: serde_json::json!({
            "request_id": "req-security",
            "timeout_ms": 1000,
            "prompt_text": "untrusted planner wording",
            "reason": "untrusted planner reason"
        }),
        purpose: String::from("request confirmation"),
        on_success: StepTransition::RequestConfirmation,
        on_failure,
    }
}

fn submit_step() -> PlannedStep {
    PlannedStep {
        step_id: String::from("submit"),
        tool_name: ToolName::SubmitActiveForm,
        arguments: serde_json::json!({
            "request_id": "req-security",
            "timeout_ms": 1000,
            "form_element_id": null
        }),
        purpose: String::from("submit the active form"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    }
}

fn click_step() -> PlannedStep {
    PlannedStep {
        step_id: String::from("click"),
        tool_name: ToolName::ClickElement,
        arguments: serde_json::json!({
            "request_id": "req-security",
            "timeout_ms": 1000,
            "element_id": "button-1",
            "click_mode": "Single"
        }),
        purpose: String::from("click the selected element"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    }
}

#[test]
fn actual_submit_tool_cannot_hide_under_read_page_intent() {
    let plan = output(
        PlannerStatus::Ready,
        IntentName::ReadPage,
        vec![submit_step()],
    );
    let error =
        validate_planner_output_with_safety(&plan, &planner_available_tools(), &[], &safety(false))
            .expect_err("actual submit tool must be rejected regardless of declared intent");

    assert!(matches!(
        error.code.as_str(),
        "confirmation_required_by_runtime_policy" | "invalid_planner_output"
    ));
}

#[test]
fn ready_submit_after_read_only_step_is_rejected() {
    let read_step = PlannedStep {
        step_id: String::from("read"),
        tool_name: ToolName::GetRuntimeStatus,
        arguments: serde_json::json!({
            "request_id": "req-security",
            "timeout_ms": 1000,
            "include_provider_modes": true
        }),
        purpose: String::from("read status first"),
        on_success: StepTransition::NextStep {
            step_id: String::from("submit"),
        },
        on_failure: StepTransition::Replan,
    };
    let plan = output(
        PlannerStatus::Ready,
        IntentName::SubmitForm,
        vec![read_step, submit_step()],
    );

    validate_planner_output_with_safety(&plan, &planner_available_tools(), &[], &safety(false))
        .expect_err("a protected later step must not bypass confirmation");
}

#[test]
fn click_setting_cannot_bypass_missing_grounding_authorization() {
    let plan = output(
        PlannerStatus::Ready,
        IntentName::ClickElement,
        vec![click_step()],
    );
    let error =
        validate_planner_output_with_safety(&plan, &planner_available_tools(), &[], &safety(true))
            .expect_err("element id alone is not deterministic click authorization");

    assert_eq!(error.code, "confirmation_required_by_runtime_policy");
}

#[test]
fn eval_js_is_prohibited_even_when_planner_requests_confirmation() {
    let eval = PlannedStep {
        step_id: String::from("eval"),
        tool_name: ToolName::EvalJs,
        arguments: serde_json::json!({
            "request_id": "req-security",
            "timeout_ms": 1000,
            "expression": "document.body.innerHTML = 'owned'"
        }),
        purpose: String::from("execute planner supplied JavaScript"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };
    let plan = output(
        PlannerStatus::NeedsConfirmation,
        IntentName::Unknown,
        vec![confirm_step(StepTransition::Replan), eval],
    );

    let error =
        validate_planner_output_with_safety(&plan, &planner_available_tools(), &[], &safety(false))
            .expect_err("arbitrary planner JavaScript is prohibited");
    assert_eq!(error.code, "prohibited_planner_action");
}

#[test]
fn protected_plan_requires_confirmation_as_first_step() {
    let plan = output(
        PlannerStatus::NeedsConfirmation,
        IntentName::SubmitForm,
        vec![submit_step(), confirm_step(StepTransition::Replan)],
    );
    let error =
        validate_planner_output_with_safety(&plan, &planner_available_tools(), &[], &safety(false))
            .expect_err("protected action before confirmation must be rejected");
    assert!(error
        .message
        .contains("confirm_action must be the first step"));
}

#[test]
fn confirmation_failure_cannot_route_to_protected_action() {
    let plan = output(
        PlannerStatus::NeedsConfirmation,
        IntentName::SubmitForm,
        vec![
            confirm_step(StepTransition::NextStep {
                step_id: String::from("submit"),
            }),
            submit_step(),
        ],
    );
    let error =
        validate_planner_output_with_safety(&plan, &planner_available_tools(), &[], &safety(false))
            .expect_err("confirmation failure must fail closed");
    assert!(error.message.contains("failure must replan"));
}

#[test]
fn correctly_gated_submit_plan_is_accepted() {
    let plan = output(
        PlannerStatus::NeedsConfirmation,
        IntentName::SubmitForm,
        vec![confirm_step(StepTransition::Replan), submit_step()],
    );

    validate_planner_output_with_safety(&plan, &planner_available_tools(), &[], &safety(false))
        .expect("deterministically gated submit plan should validate");
}

#[test]
fn executor_rejects_unconfirmed_submit_when_validation_is_skipped() {
    let plan = output(
        PlannerStatus::Ready,
        IntentName::ReadPage,
        vec![submit_step()],
    );
    let outcome = execute_planner_output_with_runner(String::from("req-security"), &plan, |_| {
        panic!("protected step must not reach the runner")
    });

    let ExecutionOutcome::Aborted { error, .. } = outcome else {
        panic!("executor must abort an unconfirmed protected action");
    };
    assert_eq!(error.code, "unconfirmed_side_effect_at_execution");
}

#[test]
fn executor_allows_safe_read_only_plan_without_confirmation() {
    let step = PlannedStep {
        step_id: String::from("status"),
        tool_name: ToolName::GetRuntimeStatus,
        arguments: serde_json::json!({
            "request_id": "req-security",
            "timeout_ms": 1000,
            "include_provider_modes": true
        }),
        purpose: String::from("read runtime status"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };
    let plan = output(PlannerStatus::Ready, IntentName::GetStatus, vec![step]);
    let outcome =
        execute_planner_output_with_runner(String::from("req-security"), &plan, |executed| {
            ToolResult::success(
                executed.tool_name.clone(),
                String::from("req-security"),
                serde_json::json!({"status": "ok"}),
                Vec::new(),
            )
        });

    assert!(matches!(outcome, ExecutionOutcome::Complete { .. }));
}

#[test]
fn every_tool_has_an_explicit_policy_classification() {
    let tools = [
        ToolName::OpenUrl,
        ToolName::GoBack,
        ToolName::GoForward,
        ToolName::ReloadPage,
        ToolName::GetHtml,
        ToolName::EvalJs,
        ToolName::ScrollPage,
        ToolName::CaptureScreenshot,
        ToolName::SetBrowserVisibility,
        ToolName::GetPageSnapshot,
        ToolName::ExtractPageModel,
        ToolName::ListInteractiveElements,
        ToolName::FindElement,
        ToolName::ClickElement,
        ToolName::FocusElement,
        ToolName::TypeIntoElement,
        ToolName::SubmitActiveForm,
        ToolName::ReadRegion,
        ToolName::ReadNextRegion,
        ToolName::ReadPreviousRegion,
        ToolName::StopSpeaking,
        ToolName::StartListening,
        ToolName::StopListening,
        ToolName::TranscribeCommand,
        ToolName::SetTtsVoice,
        ToolName::SetPlaybackVolume,
        ToolName::SetPlaybackSpeed,
        ToolName::RunOcr,
        ToolName::MergeOcrIntoPageModel,
        ToolName::GetAgentState,
        ToolName::GetRuntimeStatus,
        ToolName::ConfirmAction,
        ToolName::ReportResult,
    ];

    for tool in tools {
        let policy = tool_policy(&tool);
        assert_ne!(
            policy.class,
            ActionClass::CredentialOperation,
            "current tool unexpectedly fell through to a placeholder class"
        );
        assert_ne!(
            policy.class,
            ActionClass::ModelDownload,
            "current tool unexpectedly fell through to a placeholder class"
        );
    }
}
