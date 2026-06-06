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
