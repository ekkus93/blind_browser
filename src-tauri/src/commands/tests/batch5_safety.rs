use super::*;
use crate::state::{AppState, ClickAuthorizationRecord};

fn safety(allow_click_without_confirmation: bool) -> PlannerSafetySettings {
    PlannerSafetySettings {
        confirmation_confidence_threshold: 0.85,
        allow_click_without_confirmation,
        always_confirm_submit: true,
    }
}

fn authorized_click_step(confidence_bps: Option<u16>) -> PlannedStep {
    let mut arguments = serde_json::json!({
        "request_id": "req-batch5-click",
        "timeout_ms": 1000,
        "element_id": "button-1",
        "click_mode": "Single"
    });
    let object = arguments
        .as_object_mut()
        .expect("click arguments should be an object");
    object.insert(
        CLICK_AUTH_TOKEN_ARG.to_string(),
        serde_json::json!("opaque-runtime-token"),
    );
    object.insert(
        CLICK_AUTH_AMBIGUOUS_ARG.to_string(),
        serde_json::json!(false),
    );
    object.insert(
        CLICK_AUTH_DESTRUCTIVE_ARG.to_string(),
        serde_json::json!(false),
    );
    object.insert(CLICK_AUTH_GENERATION_ARG.to_string(), serde_json::json!(7));
    object.insert(
        CLICK_AUTH_CONFIDENCE_ARG.to_string(),
        confidence_bps
            .map(|value| serde_json::json!(value))
            .unwrap_or(serde_json::Value::Null),
    );

    PlannedStep {
        step_id: String::from("click"),
        tool_name: ToolName::ClickElement,
        arguments,
        purpose: String::from("click the runtime-authorized target"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    }
}

fn planner_output(steps: Vec<PlannedStep>) -> PlannerOutput {
    PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::ClickElement,
            goal: String::from("activate the selected page target"),
            target_description: Some(String::from("button-1")),
        },
        selected_skills: Vec::new(),
        steps,
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    }
}

#[test]
fn missing_click_confidence_fails_closed() {
    let decision = evaluate_action_policy(&[authorized_click_step(None)], &safety(true));

    assert_eq!(
        decision.requirement,
        ConfirmationRequirement::ConfirmationRequired
    );
    assert_eq!(
        decision.findings[0].reason_code,
        ActionPolicyReasonCode::ClickGroundingUnavailable
    );
}

#[test]
fn click_confidence_below_threshold_requires_confirmation() {
    let decision = evaluate_action_policy(&[authorized_click_step(Some(8_499))], &safety(true));

    assert_eq!(
        decision.requirement,
        ConfirmationRequirement::ConfirmationRequired
    );
    assert_eq!(
        decision.findings[0].reason_code,
        ActionPolicyReasonCode::ClickConfidenceBelowThreshold
    );
}

#[test]
fn disabled_click_exception_requires_confirmation_even_with_valid_grounding() {
    let decision = evaluate_action_policy(&[authorized_click_step(Some(9_500))], &safety(false));

    assert_eq!(
        decision.requirement,
        ConfirmationRequirement::ConfirmationRequired
    );
    assert_eq!(
        decision.findings[0].reason_code,
        ActionPolicyReasonCode::ClickRequiresConfirmationBySetting
    );
}

#[test]
fn protected_click_cannot_hide_inside_a_cycle() {
    let status_step = PlannedStep {
        step_id: String::from("status"),
        tool_name: ToolName::GetRuntimeStatus,
        arguments: serde_json::json!({
            "request_id": "req-batch5-cycle",
            "timeout_ms": 1000,
            "include_provider_modes": true
        }),
        purpose: String::from("read status before interaction"),
        on_success: StepTransition::NextStep {
            step_id: String::from("click"),
        },
        on_failure: StepTransition::Replan,
    };
    let mut click = authorized_click_step(Some(9_500));
    click.on_success = StepTransition::NextStep {
        step_id: String::from("status"),
    };
    let output = planner_output(vec![status_step, click]);

    let error = validate_planner_output_with_safety(
        &output,
        &planner_available_tools(),
        &[],
        &safety(true),
    )
    .expect_err("a cyclic action graph must be rejected before execution");

    assert_eq!(error.code, "invalid_planner_output");
    assert!(error.message.to_ascii_lowercase().contains("cycle"));
}

#[test]
fn page_model_change_invalidates_click_authorizations_and_pending_confirmation() {
    let mut state = AppState::default();
    state.current_page_id = Some(String::from("page-1"));
    state.page_generation = 7;
    state.pending_confirmation_id = Some(String::from("confirm-1"));
    state.click_authorizations.insert(
        String::from("token-1"),
        ClickAuthorizationRecord {
            token: String::from("token-1"),
            page_id: String::from("page-1"),
            page_generation: 7,
            origin: Some(String::from("https://example.com")),
            element_id: String::from("button-1"),
            dom_locator: String::from("#button-1"),
            element_fingerprint: String::from("fingerprint"),
            confidence_bps: Some(9_500),
            ambiguous: false,
            potentially_destructive: false,
            issued_at_ms: 1,
            expires_at_ms: u64::MAX,
        },
    );

    state.mark_page_model_changed();

    assert_eq!(state.page_generation, 8);
    assert!(state.click_authorizations.is_empty());
    assert_eq!(state.pending_confirmation_id, None);
}
