use super::*;

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
