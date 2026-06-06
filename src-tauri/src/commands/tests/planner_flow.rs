use super::*;

#[test]
fn executes_next_step_chain_until_complete() {
    let mut executor = MockExecutor::default();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::SetPlaybackVolume,
            goal: String::from("adjust audio"),
            target_description: None,
        },
        selected_skills: vec![String::from("audio_controls")],
        steps: vec![
            PlannedStep {
                step_id: String::from("step-1"),
                tool_name: ToolName::SetPlaybackVolume,
                arguments: serde_json::json!({
                    "request_id": "req-plan",
                    "timeout_ms": 1000,
                    "volume": 0.4
                }),
                purpose: String::from("set the volume"),
                on_success: StepTransition::NextStep {
                    step_id: String::from("step-2"),
                },
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from("step-2"),
                tool_name: ToolName::GetRuntimeStatus,
                arguments: serde_json::json!({
                    "request_id": "req-plan",
                    "timeout_ms": 1000,
                    "include_provider_modes": false
                }),
                purpose: String::from("read back the runtime state"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let outcome = execute_planner_output(&mut executor, String::from("req-plan"), &planner_output);

    match outcome {
        ExecutionOutcome::Complete { trace } => {
            assert_eq!(trace.executed_step_ids, vec!["step-1", "step-2"]);
            assert_eq!(trace.tool_results.len(), 2);
        }
        other => panic!("expected complete outcome, got {other:?}"),
    }
}

#[test]
fn executes_load_page_extract_and_read_flow_from_resolved_read_page_plan() {
    let mut executor = MockExecutor::default();
    let page_model = fixture_problematic_article_page_without_regions();
    let agent_state = fixture_agent_state_for_page(
        "Metro news | Night trains finally return",
        "https://news.example.com/city/night-trains-return",
    );
    let planner_output = resolve_direct_read_page_command(
        "read page",
        "req-load-extract-read",
        Some(&page_model),
        &agent_state,
        &[String::from("read_page")],
    )
    .expect("read-page command should resolve");
    let expected_step_ids = planner_output
        .steps
        .iter()
        .map(|step| step.step_id.clone())
        .collect::<Vec<_>>();
    let expected_extract_input: ExtractPageModelInput =
        serde_json::from_value(planner_output.steps[0].arguments.clone())
            .expect("extract step should deserialize");
    let expected_read_next_input: ReadNextRegionInput =
        serde_json::from_value(planner_output.steps[1].arguments.clone())
            .expect("read-next step should deserialize");

    let outcome = execute_planner_output(
        &mut executor,
        String::from("req-load-extract-read"),
        &planner_output,
    );

    match outcome {
        ExecutionOutcome::Complete { trace } => {
            assert_eq!(trace.executed_step_ids, expected_step_ids);
            assert_eq!(
                trace
                    .tool_results
                    .iter()
                    .map(|result| result.tool_name.clone())
                    .collect::<Vec<_>>(),
                vec![ToolName::ExtractPageModel, ToolName::ReadNextRegion]
            );
            assert_eq!(executor.last_extract_request, Some(expected_extract_input));
            assert_eq!(
                executor.last_read_next_region_request,
                Some(expected_read_next_input)
            );
        }
        other => panic!("expected complete outcome, got {other:?}"),
    }
}

#[test]
fn executes_resolved_spoken_command_action_flow_for_continue_reading() {
    let mut executor = MockExecutor::default();
    let planner_output = resolve_direct_navigation_readback_command(
        "continue reading",
        "req-asr-command-action",
        &[String::from("read_next")],
    )
    .expect("continue-reading command should resolve");
    let expected_step_ids = planner_output
        .steps
        .iter()
        .map(|step| step.step_id.clone())
        .collect::<Vec<_>>();

    let outcome = execute_planner_output(
        &mut executor,
        String::from("req-asr-command-action"),
        &planner_output,
    );

    match outcome {
        ExecutionOutcome::Complete { trace } => {
            assert_eq!(planner_output.intent.name, IntentName::ReadNext);
            assert_eq!(trace.executed_step_ids, expected_step_ids);
            assert_eq!(trace.tool_results.len(), 1);
            assert_eq!(trace.tool_results[0].tool_name, ToolName::ReadNextRegion);
            assert_eq!(
                executor.last_read_next_region_request,
                Some(ReadNextRegionInput {
                    request_id: String::from("req-asr-command-action"),
                    timeout_ms: None,
                    interruption_mode: NarrationInterruptionMode::Interrupt,
                })
            );
        }
        other => panic!("expected complete outcome, got {other:?}"),
    }
}

#[test]
fn follows_failure_transition_to_replan() {
    let mut executor = MockExecutor::default();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::SetPlaybackSpeed,
            goal: String::from("adjust playback speed"),
            target_description: None,
        },
        selected_skills: vec![String::from("audio_controls")],
        steps: vec![PlannedStep {
            step_id: String::from("step-1"),
            tool_name: ToolName::SetPlaybackSpeed,
            arguments: serde_json::json!({
                "request_id": "req-replan",
                "timeout_ms": 1000,
                "speed": "fast"
            }),
            purpose: String::from("set invalid speed"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let outcome =
        execute_planner_output(&mut executor, String::from("req-replan"), &planner_output);

    match outcome {
        ExecutionOutcome::NeedsReplan { trace } => {
            assert_eq!(trace.executed_step_ids, vec!["step-1"]);
            assert_eq!(trace.tool_results.len(), 1);
            assert!(!trace.tool_results[0].ok);
        }
        other => panic!("expected replan outcome, got {other:?}"),
    }
}

#[test]
fn returns_awaiting_confirmation_when_transition_requests_it() {
    let planner_output = PlannerOutput {
        status: PlannerStatus::NeedsConfirmation,
        intent: IntentSummary {
            name: IntentName::ClickElement,
            goal: String::from("confirm button choice"),
            target_description: Some(String::from("submit button")),
        },
        selected_skills: vec![String::from("confirm_action")],
        steps: vec![
            PlannedStep {
                step_id: String::from("step-1"),
                tool_name: ToolName::ConfirmAction,
                arguments: serde_json::json!({
                    "request_id": "req-confirm"
                }),
                purpose: String::from("ask for confirmation"),
                on_success: StepTransition::RequestConfirmation,
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from("step-2"),
                tool_name: ToolName::SetBrowserVisibility,
                arguments: serde_json::json!({
                    "request_id": "req-confirm",
                    "timeout_ms": 1000,
                    "mode": "Visible"
                }),
                purpose: String::from("placeholder protected step"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ],
        requires_confirmation: true,
        confirmation_reason: Some(String::from("protected action")),
        blocked_reason: None,
        user_message: Some(String::from("Please confirm.")),
    };

    let outcome =
        execute_planner_output_with_runner(String::from("req-confirm"), &planner_output, |step| {
            assert_eq!(step.step_id, "step-1");
            ToolResult::success(
                ToolName::ConfirmAction,
                String::from("req-confirm"),
                serde_json::json!({
                    "confirmation_id": "confirm-1",
                    "prompt_text": "Proceed?",
                    "confirmed": serde_json::Value::Null,
                    "timed_out": false
                }),
                vec![String::from("confirmation requested")],
            )
        });

    match outcome {
        ExecutionOutcome::AwaitingConfirmation {
            trace,
            pending_confirmation_id,
            pending_plan_execution,
        } => {
            assert_eq!(trace.executed_step_ids, vec!["step-1"]);
            assert_eq!(pending_confirmation_id, "confirm-1");
            assert_eq!(pending_plan_execution.request_id, "req-confirm");
            assert_eq!(pending_plan_execution.intent_name, IntentName::ClickElement);
            assert_eq!(pending_plan_execution.prompt_text, "Proceed?");
            assert_eq!(
                pending_plan_execution.next_step_id,
                Some(String::from("step-2"))
            );
            assert_eq!(pending_plan_execution.queued_step_ids, vec!["step-2"]);
            assert_eq!(pending_plan_execution.queued_steps.len(), 1);
            assert_eq!(pending_plan_execution.queued_steps[0].step_id, "step-2");
        }
        other => panic!("expected awaiting confirmation outcome, got {other:?}"),
    }
}

#[test]
fn aborts_when_next_step_transition_is_missing() {
    let mut executor = MockExecutor::default();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::SetPlaybackVolume,
            goal: String::from("adjust audio"),
            target_description: None,
        },
        selected_skills: vec![String::from("audio_controls")],
        steps: vec![PlannedStep {
            step_id: String::from("step-1"),
            tool_name: ToolName::SetPlaybackVolume,
            arguments: serde_json::json!({
                "request_id": "req-bad-transition",
                "timeout_ms": 1000,
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

    let outcome = execute_planner_output(
        &mut executor,
        String::from("req-bad-transition"),
        &planner_output,
    );

    match outcome {
        ExecutionOutcome::Aborted { trace, error } => {
            assert_eq!(trace.executed_step_ids, vec!["step-1"]);
            assert_eq!(error.code, "missing_transition_step");
        }
        other => panic!("expected aborted outcome, got {other:?}"),
    }
}

#[test]
fn aborts_needs_confirmation_plan_before_side_effecting_step() {
    let mut executor = MockExecutor::default();
    let planner_output = PlannerOutput {
        status: PlannerStatus::NeedsConfirmation,
        intent: IntentSummary {
            name: IntentName::SetBrowserVisibility,
            goal: String::from("toggle browser visibility"),
            target_description: None,
        },
        selected_skills: vec![String::from("confirm_action")],
        steps: vec![PlannedStep {
            step_id: String::from("step-1"),
            tool_name: ToolName::SetBrowserVisibility,
            arguments: serde_json::json!({
                "request_id": "req-needs-confirm",
                "timeout_ms": 1000,
                "mode": "Visible"
            }),
            purpose: String::from("protected action before confirmation"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: true,
        confirmation_reason: Some(String::from("protected action")),
        blocked_reason: None,
        user_message: Some(String::from("Please confirm.")),
    };

    let outcome = execute_planner_output(
        &mut executor,
        String::from("req-needs-confirm"),
        &planner_output,
    );

    match outcome {
        ExecutionOutcome::Aborted { trace, error } => {
            assert!(trace.executed_step_ids.is_empty());
            assert_eq!(error.code, "side_effect_before_confirmation");
        }
        other => panic!("expected aborted outcome, got {other:?}"),
    }
}

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

#[test]
fn set_tts_voice_input_accepts_known_voice_names_only() {
    let local_voice: SetTtsVoiceInput = serde_json::from_value(serde_json::json!({
        "request_id": "req-local-voice",
        "voice": "Bruno"
    }))
    .expect("known local voice should deserialize");
    assert_eq!(local_voice.voice, TtsVoiceName::Bruno);

    let remote_voice: SetTtsVoiceInput = serde_json::from_value(serde_json::json!({
        "request_id": "req-remote-voice",
        "voice": "alloy"
    }))
    .expect("known remote voice should deserialize");
    assert_eq!(remote_voice.voice, TtsVoiceName::Alloy);

    let invalid_voice = serde_json::from_value::<SetTtsVoiceInput>(serde_json::json!({
        "request_id": "req-invalid-voice",
        "voice": "not-a-real-voice"
    }));
    assert!(invalid_voice.is_err());
}

#[test]
fn validate_planner_output_rejects_open_url_with_blank_url() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::OpenUrl,
            goal: String::from("open a page"),
            target_description: None,
        },
        selected_skills: vec![String::from("open_url_direct")],
        steps: vec![PlannedStep {
            step_id: String::from("step-open-url"),
            tool_name: ToolName::OpenUrl,
            arguments: serde_json::json!({
                "request_id": "req-open-url",
                "url": "   ",
                "wait_for_load_state": "NetworkIdle"
            }),
            purpose: String::from("open a page"),
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
        &[String::from("open_url_direct")],
    )
    .expect_err("validation should reject blank open_url values");
    assert!(error.message.contains("open_url requires a non-empty url"));
}

#[test]
fn validate_eval_js_input_rejects_blank_expression() {
    let step = PlannedStep {
        step_id: String::from("step-eval-js"),
        tool_name: ToolName::EvalJs,
        arguments: serde_json::json!({
            "request_id": "req-eval-js",
            "expression": "   "
        }),
        purpose: String::from("evaluate a bounded JavaScript expression"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let error = validate_planned_step_arguments(&step)
        .expect_err("validation should reject blank eval_js expressions");
    assert!(error
        .message
        .contains("eval_js requires a non-empty expression"));
}

#[test]
fn validate_planner_output_rejects_open_url_with_relative_url() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::OpenUrl,
            goal: String::from("open a page"),
            target_description: None,
        },
        selected_skills: vec![String::from("open_url_direct")],
        steps: vec![PlannedStep {
            step_id: String::from("step-open-url"),
            tool_name: ToolName::OpenUrl,
            arguments: serde_json::json!({
                "request_id": "req-open-url",
                "url": "/relative/path",
                "wait_for_load_state": "Load"
            }),
            purpose: String::from("open a page"),
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
        &[String::from("open_url_direct")],
    )
    .expect_err("validation should reject relative open_url values");
    assert!(error
        .message
        .contains("open_url requires an absolute URL with a scheme"));
}

#[test]
fn validate_planner_output_rejects_go_back_with_too_many_steps() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::GoBack,
            goal: String::from("go back"),
            target_description: None,
        },
        selected_skills: vec![String::from("go_back")],
        steps: vec![PlannedStep {
            step_id: String::from("step-go-back"),
            tool_name: ToolName::GoBack,
            arguments: serde_json::json!({
                "request_id": "req-go-back",
                "steps": MAX_HISTORY_STEPS + 1,
                "wait_for_load_state": "Load"
            }),
            purpose: String::from("go back"),
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
        &[String::from("go_back")],
    )
    .expect_err("validation should reject go_back steps above the supported maximum");
    assert!(error
        .message
        .contains("go_back steps must be less than or equal to"));
}

#[test]
fn validate_planner_output_rejects_go_forward_with_zero_steps() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::GoForward,
            goal: String::from("go forward"),
            target_description: None,
        },
        selected_skills: vec![String::from("go_forward")],
        steps: vec![PlannedStep {
            step_id: String::from("step-go-forward"),
            tool_name: ToolName::GoForward,
            arguments: serde_json::json!({
                "request_id": "req-go-forward",
                "steps": 0,
                "wait_for_load_state": "Load"
            }),
            purpose: String::from("go forward"),
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
        &[String::from("go_forward")],
    )
    .expect_err("validation should reject go_forward steps below the supported minimum");
    assert!(error
        .message
        .contains("go_forward steps must be greater than 0"));
}

#[test]
fn validate_planner_output_rejects_scroll_page_without_amount_or_target() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::ReadNext,
            goal: String::from("scroll the page"),
            target_description: None,
        },
        selected_skills: vec![String::from("scroll_page")],
        steps: vec![PlannedStep {
            step_id: String::from("step-scroll"),
            tool_name: ToolName::ScrollPage,
            arguments: serde_json::json!({
                "request_id": "req-scroll",
                "direction": "Down",
                "amount_px": serde_json::Value::Null,
                "target": serde_json::Value::Null
            }),
            purpose: String::from("scroll the page"),
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
        &[String::from("scroll_page")],
    )
    .expect_err("validation should reject scroll_page requests without amount or target");
    assert!(error
        .message
        .contains("scroll_page requires amount_px or target to be provided"));
}

#[test]
fn validate_planner_output_rejects_scroll_page_with_non_positive_amount() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::ReadNext,
            goal: String::from("scroll the page"),
            target_description: None,
        },
        selected_skills: vec![String::from("scroll_page")],
        steps: vec![PlannedStep {
            step_id: String::from("step-scroll"),
            tool_name: ToolName::ScrollPage,
            arguments: serde_json::json!({
                "request_id": "req-scroll",
                "direction": "Down",
                "amount_px": 0.0,
                "target": serde_json::Value::Null
            }),
            purpose: String::from("scroll the page"),
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
        &[String::from("scroll_page")],
    )
    .expect_err("validation should reject non-positive scroll amounts");
    assert!(error
        .message
        .contains("scroll_page amount_px must be greater than 0"));
}

#[test]
fn validate_planner_output_rejects_find_element_with_blank_description() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::FindElement,
            goal: String::from("find an element"),
            target_description: None,
        },
        selected_skills: vec![String::from("find_element")],
        steps: vec![PlannedStep {
            step_id: String::from("step-find-element"),
            tool_name: ToolName::FindElement,
            arguments: serde_json::json!({
                "request_id": "req-find-element",
                "description": "   ",
                "text": null,
                "role": null,
                "color_hint": null,
                "nearby_text": null,
                "selector_hint": null,
                "visibility_filter": "VisibleOnly",
                "max_candidates": 3
            }),
            purpose: String::from("find an element"),
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
        &[String::from("find_element")],
    )
    .expect_err("validation should reject blank find_element descriptions");
    assert!(error
        .message
        .contains("find_element requires a non-empty description"));
}

#[test]
fn validate_planner_output_rejects_find_element_with_zero_max_candidates() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::FindElement,
            goal: String::from("find an element"),
            target_description: None,
        },
        selected_skills: vec![String::from("find_element")],
        steps: vec![PlannedStep {
            step_id: String::from("step-find-element"),
            tool_name: ToolName::FindElement,
            arguments: serde_json::json!({
                "request_id": "req-find-element",
                "description": "search field",
                "text": null,
                "role": null,
                "color_hint": null,
                "nearby_text": null,
                "selector_hint": null,
                "visibility_filter": "VisibleOnly",
                "max_candidates": 0
            }),
            purpose: String::from("find an element"),
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
        &[String::from("find_element")],
    )
    .expect_err("validation should reject zero max_candidates");
    assert!(error
        .message
        .contains("find_element max_candidates must be greater than 0"));
}

#[test]
fn validate_planner_output_rejects_find_element_with_too_many_max_candidates() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::FindElement,
            goal: String::from("find an element"),
            target_description: None,
        },
        selected_skills: vec![String::from("find_element")],
        steps: vec![PlannedStep {
            step_id: String::from("step-find-element"),
            tool_name: ToolName::FindElement,
            arguments: serde_json::json!({
                "request_id": "req-find-element",
                "description": "search field",
                "text": null,
                "role": null,
                "color_hint": null,
                "nearby_text": null,
                "selector_hint": null,
                "visibility_filter": "VisibleOnly",
                "max_candidates": DEFAULT_FIND_ELEMENT_MAX_CANDIDATES + 1
            }),
            purpose: String::from("find an element"),
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
        &[String::from("find_element")],
    )
    .expect_err("validation should reject max_candidates above the supported maximum");
    assert!(error
        .message
        .contains("find_element max_candidates must be less than or equal to"));
}

#[test]
fn validate_planner_output_rejects_set_playback_volume_out_of_range() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::SetPlaybackVolume,
            goal: String::from("set playback volume"),
            target_description: None,
        },
        selected_skills: vec![String::from("audio_controls")],
        steps: vec![PlannedStep {
            step_id: String::from("step-volume"),
            tool_name: ToolName::SetPlaybackVolume,
            arguments: serde_json::json!({
                "request_id": "req-volume",
                "volume": 1.5
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
    .expect_err("validation should reject out-of-range playback volume");
    assert!(error
        .message
        .contains("set_playback_volume volume must be between 0.0"));
}

#[test]
fn validate_planner_output_rejects_set_playback_speed_out_of_range() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::SetPlaybackSpeed,
            goal: String::from("set playback speed"),
            target_description: None,
        },
        selected_skills: vec![String::from("audio_controls")],
        steps: vec![PlannedStep {
            step_id: String::from("step-speed"),
            tool_name: ToolName::SetPlaybackSpeed,
            arguments: serde_json::json!({
                "request_id": "req-speed",
                "speed": 10.0
            }),
            purpose: String::from("set the speed"),
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
    .expect_err("validation should reject out-of-range playback speed");
    assert!(error
        .message
        .contains("set_playback_speed speed must be between"));
}

#[test]
fn validate_confirm_action_input_rejects_blank_prompt() {
    let error = validate_confirm_action_input(&ConfirmActionInput {
        request_id: String::from("req-confirm"),
        timeout_ms: None,
        prompt_text: String::from("   "),
        reason: String::from("Submission changes remote state."),
    })
    .expect_err("validation should reject blank confirm_action prompt_text");
    assert!(error
        .message
        .contains("confirm_action requires a non-empty prompt_text"));
}

