use super::*;

#[test]
fn planner_available_tools_include_all_wave_two_tools() {
    let available_tools = planner_available_tools();

    assert!(available_tools
        .iter()
        .any(|tool| tool.name == ToolName::OpenUrl));
    assert!(available_tools
        .iter()
        .any(|tool| tool.name == ToolName::TranscribeCommand));
    assert!(available_tools
        .iter()
        .any(|tool| tool.name == ToolName::FocusElement));
    assert!(available_tools
        .iter()
        .any(|tool| tool.name == ToolName::TypeIntoElement));
    assert!(available_tools
        .iter()
        .any(|tool| tool.name == ToolName::SubmitActiveForm));
    assert!(available_tools
        .iter()
        .any(|tool| tool.name == ToolName::CaptureScreenshot));
    assert!(available_tools
        .iter()
        .any(|tool| tool.name == ToolName::GetHtml));
    assert!(available_tools
        .iter()
        .any(|tool| tool.name == ToolName::RunOcr));
    assert!(available_tools
        .iter()
        .any(|tool| tool.name == ToolName::MergeOcrIntoPageModel));
}

#[test]
fn validate_planner_output_rejects_unknown_selected_skill() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::GetStatus,
            goal: String::from("report the current status"),
            target_description: None,
        },
        selected_skills: vec![String::from("not-a-real-skill")],
        steps: vec![PlannedStep {
            step_id: String::from("step-status"),
            tool_name: ToolName::GetRuntimeStatus,
            arguments: serde_json::json!({
                "request_id": "req-status",
                "include_provider_modes": true
            }),
            purpose: String::from("read runtime status"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("get_status")],
    )
    .expect_err("validation should reject unknown selected skills");
    assert_eq!(error.code, "invalid_planner_output");
    assert!(error.message.contains("unknown or ineligible skill"));
}

#[test]
fn validate_planner_output_rejects_unavailable_tool_reference() {
    let mut available_tools = planner_available_tools();
    available_tools.retain(|tool| tool.name != ToolName::SetPlaybackVolume);
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::SetPlaybackVolume,
            goal: String::from("adjust audio"),
            target_description: None,
        },
        selected_skills: vec![String::from("audio_controls")],
        steps: vec![PlannedStep {
            step_id: String::from("step-volume"),
            tool_name: ToolName::SetPlaybackVolume,
            arguments: serde_json::json!({
                "request_id": "req-volume",
                "volume": 0.4
            }),
            purpose: String::from("set the volume"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("audio_controls")],
    )
    .expect_err("validation should reject unavailable tool references");
    assert_eq!(error.code, "invalid_planner_output");
    assert!(error
        .message
        .contains("planner referenced unavailable tool SetPlaybackVolume"));
    assert_eq!(
        error
            .details
            .as_ref()
            .and_then(|details| details.get("step_id")),
        Some(&serde_json::json!("step-volume"))
    );
}

#[test]
fn validate_planner_output_rejects_missing_next_step_transition() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::SetPlaybackVolume,
            goal: String::from("adjust audio"),
            target_description: None,
        },
        selected_skills: vec![String::from("audio_controls")],
        steps: vec![PlannedStep {
            step_id: String::from("step-volume"),
            tool_name: ToolName::SetPlaybackVolume,
            arguments: serde_json::json!({
                "request_id": "req-volume",
                "volume": 0.4
            }),
            purpose: String::from("set the volume"),
            on_success: StepTransition::NextStep {
                step_id: String::from("missing-step"),
            },
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("audio_controls")],
    )
    .expect_err("validation should reject missing next-step transitions");
    assert_eq!(error.code, "invalid_planner_output");
    assert!(error
        .message
        .contains("planner referenced missing next step 'missing-step' from 'step-volume'"));
    assert_eq!(
        error
            .details
            .as_ref()
            .and_then(|details| details.get("source_step_id")),
        Some(&serde_json::json!("step-volume"))
    );
    assert_eq!(
        error
            .details
            .as_ref()
            .and_then(|details| details.get("next_step_id")),
        Some(&serde_json::json!("missing-step"))
    );
}

#[test]
fn validate_planner_output_rejects_submit_form_without_needs_confirmation() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::SubmitForm,
            goal: String::from("submit the active form"),
            target_description: Some(String::from("login form")),
        },
        selected_skills: vec![String::from("confirm_action")],
        steps: vec![PlannedStep {
            step_id: String::from("confirm-submit"),
            tool_name: ToolName::ConfirmAction,
            arguments: serde_json::json!({
                "request_id": "req-submit",
                "timeout_ms": 1000,
                "prompt_text": "Submit the form now?",
                "reason": "Submitting the form may send data."
            }),
            purpose: String::from("ask for confirmation"),
            on_success: StepTransition::RequestConfirmation,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: true,
        confirmation_reason: Some(String::from("submitting the form may send data")),
        blocked_reason: None,
        user_message: Some(String::from("Please confirm before I submit the form.")),
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("confirm_action")],
    )
    .expect_err("submit-form plans should require NeedsConfirmation status");
    assert_eq!(error.code, "invalid_planner_output");
    assert!(error
        .message
        .contains("submit-form planner output must use NeedsConfirmation"));
}

#[test]
fn validate_planner_output_rejects_submit_form_without_confirm_action_step() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::NeedsConfirmation,
        intent: IntentSummary {
            name: IntentName::SubmitForm,
            goal: String::from("submit the active form"),
            target_description: Some(String::from("login form")),
        },
        selected_skills: vec![String::from("confirm_action")],
        steps: vec![PlannedStep {
            step_id: String::from("report-submit"),
            tool_name: ToolName::ReportResult,
            arguments: serde_json::json!({
                "request_id": "req-submit",
                "timeout_ms": 1000,
                "status": "NeedsFollowUp",
                "summary": "The form is ready to submit.",
                "next_recommended_action": "Confirm the submission.",
                "user_message": "The form is ready to submit."
            }),
            purpose: String::from("report submit readiness"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: true,
        confirmation_reason: Some(String::from("submitting the form may send data")),
        blocked_reason: None,
        user_message: Some(String::from("Please confirm before I submit the form.")),
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("confirm_action")],
    )
    .expect_err("submit-form plans should require a confirm_action step");
    assert_eq!(error.code, "invalid_planner_output");
    assert!(error.message.contains("must include a confirm_action step"));
}

#[test]
fn validate_planner_output_rejects_needs_confirmation_without_confirm_action_step() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::NeedsConfirmation,
        intent: IntentSummary {
            name: IntentName::ClickElement,
            goal: String::from("activate the selected button"),
            target_description: Some(String::from("submit button")),
        },
        selected_skills: vec![String::from("open_link_by_text")],
        steps: vec![PlannedStep {
            step_id: String::from("click-button"),
            tool_name: ToolName::ClickElement,
            arguments: serde_json::json!({
                "request_id": "req-click",
                "timeout_ms": 1000,
                "element_id": "button-submit",
                "click_mode": "Single"
            }),
            purpose: String::from("activate the chosen button"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: true,
        confirmation_reason: Some(String::from("clicking may trigger a protected action")),
        blocked_reason: None,
        user_message: Some(String::from("Please confirm before I activate the button.")),
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("open_link_by_text")],
    )
    .expect_err("needs-confirmation plans should require a confirm_action step");
    assert_eq!(error.code, "invalid_planner_output");
    assert!(error.message.contains("must include a confirm_action step"));
}

#[test]
fn validate_planner_output_rejects_ready_output_with_confirmation_metadata() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::ClickElement,
            goal: String::from("activate the selected button"),
            target_description: Some(String::from("submit button")),
        },
        selected_skills: vec![String::from("confirm_action")],
        steps: vec![PlannedStep {
            step_id: String::from("confirm-click"),
            tool_name: ToolName::ConfirmAction,
            arguments: serde_json::json!({
                "request_id": "req-click",
                "timeout_ms": 1000,
                "prompt_text": "Do you want me to activate the submit button?",
                "reason": "Activating it may send data."
            }),
            purpose: String::from("ask for confirmation"),
            on_success: StepTransition::RequestConfirmation,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: true,
        confirmation_reason: Some(String::from("activating the button may send data")),
        blocked_reason: None,
        user_message: Some(String::from("Please confirm before I activate the button.")),
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("confirm_action")],
    )
    .expect_err("ready plans should not carry confirmation-only metadata");
    assert_eq!(error.code, "invalid_planner_output");
    assert!(error.message.contains("must not set requires_confirmation"));
}

#[test]
fn validate_planner_output_accepts_submit_form_with_confirmation_gate() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::NeedsConfirmation,
        intent: IntentSummary {
            name: IntentName::SubmitForm,
            goal: String::from("submit the active form"),
            target_description: Some(String::from("login form")),
        },
        selected_skills: vec![String::from("confirm_action")],
        steps: vec![
            PlannedStep {
                step_id: String::from("confirm-submit"),
                tool_name: ToolName::ConfirmAction,
                arguments: serde_json::json!({
                    "request_id": "req-submit",
                    "timeout_ms": 1000,
                    "prompt_text": "The form is filled. Do you want me to submit it now?",
                    "reason": "Submitting the form may send data."
                }),
                purpose: String::from("require explicit confirmation before submission"),
                on_success: StepTransition::RequestConfirmation,
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from("report-submit-ready"),
                tool_name: ToolName::ReportResult,
                arguments: serde_json::json!({
                    "request_id": "req-submit",
                    "timeout_ms": 1000,
                    "status": "NeedsFollowUp",
                    "summary": "The form is ready to submit after you confirm.",
                    "next_recommended_action": "Confirm the submission.",
                    "user_message": "Please confirm before I submit the form."
                }),
                purpose: String::from("keep the user informed while awaiting confirmation"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ],
        requires_confirmation: true,
        confirmation_reason: Some(String::from("submitting the form may send data")),
        blocked_reason: None,
        user_message: Some(String::from("Please confirm before I submit the form.")),
    };

    validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("confirm_action")],
    )
    .expect("submit-form plans should validate when confirmation is required");
}

#[test]
fn validate_planner_output_accepts_click_element_with_confirmation_gate() {
    let available_tools = planner_available_tools();
    let mut examples = canonical_planner_output_examples();
    let planner_output = examples
        .remove("click_element_with_confirmation")
        .expect("click confirmation example should exist");

    validate_planner_output(
        &planner_output,
        &available_tools,
        &[
            String::from("open_link_by_text"),
            String::from("confirm_action"),
        ],
    )
    .expect("click plans should validate when they use the bounded confirmation flow");
}

#[test]
fn validate_planner_output_rejects_invalid_step_arguments() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::SetPlaybackVolume,
            goal: String::from("adjust playback volume"),
            target_description: None,
        },
        selected_skills: vec![String::from("set_volume")],
        steps: vec![PlannedStep {
            step_id: String::from("step-volume"),
            tool_name: ToolName::SetPlaybackVolume,
            arguments: serde_json::json!({
                "request_id": "req-volume",
                "volume": "loud"
            }),
            purpose: String::from("set playback volume"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("set_volume")],
    )
    .expect_err("validation should reject malformed step arguments");
    assert_eq!(error.code, "invalid_planner_output");
    assert!(error.message.contains("expected schema"));
    assert_eq!(
        error
            .details
            .as_ref()
            .and_then(|details| details.get("step_id")),
        Some(&serde_json::json!("step-volume"))
    );
    assert_eq!(
        error
            .details
            .as_ref()
            .and_then(|details| details.get("tool_name")),
        Some(&serde_json::json!("SetPlaybackVolume"))
    );
}

#[test]
fn validate_planner_output_rejects_capture_screenshot_with_multiple_targets() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::OcrRecovery,
            goal: String::from("capture a screenshot for OCR"),
            target_description: None,
        },
        selected_skills: vec![String::from("ocr_current_region")],
        steps: vec![PlannedStep {
            step_id: String::from("step-capture"),
            tool_name: ToolName::CaptureScreenshot,
            arguments: serde_json::json!({
                "request_id": "req-capture",
                "scope": "FullPage",
                "region_id": "region-1",
                "bbox": serde_json::Value::Null
            }),
            purpose: String::from("capture an image for OCR"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("ocr_current_region")],
    )
    .expect_err("validation should reject conflicting screenshot target modes");
    assert_eq!(error.code, "invalid_planner_output");
    assert!(error
        .message
        .contains("capture_screenshot supports at most one targeting mode"));
}

#[test]
fn validate_planner_output_rejects_run_ocr_without_any_source() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::OcrRecovery,
            goal: String::from("read text from an image"),
            target_description: None,
        },
        selected_skills: vec![String::from("ocr_current_region")],
        steps: vec![PlannedStep {
            step_id: String::from("step-run-ocr"),
            tool_name: ToolName::RunOcr,
            arguments: serde_json::json!({
                "request_id": "req-run-ocr",
                "image_id": serde_json::Value::Null,
                "region_id": serde_json::Value::Null,
                "bbox": serde_json::Value::Null
            }),
            purpose: String::from("run OCR"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("ocr_current_region")],
    )
    .expect_err("validation should reject run_ocr without any source image or target");
    assert_eq!(error.code, "invalid_planner_output");
    assert!(error
        .message
        .contains("run_ocr requires at least one source"));
}

#[test]
fn validate_planner_output_rejects_merge_ocr_with_empty_text() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::OcrRecovery,
            goal: String::from("merge OCR text"),
            target_description: None,
        },
        selected_skills: vec![String::from("read_visible_text")],
        steps: vec![PlannedStep {
            step_id: String::from("step-merge-ocr"),
            tool_name: ToolName::MergeOcrIntoPageModel,
            arguments: serde_json::json!({
                "request_id": "req-merge-ocr",
                "page_id": "page-1",
                "region_id": serde_json::Value::Null,
                "ocr_text": "   ",
                "source_bbox": serde_json::Value::Null
            }),
            purpose: String::from("merge OCR text"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("read_visible_text")],
    )
    .expect_err("validation should reject merge_ocr_into_page_model without OCR text");
    assert_eq!(error.code, "invalid_planner_output");
    assert!(error
        .message
        .contains("merge_ocr_into_page_model requires non-empty ocr_text"));
}
