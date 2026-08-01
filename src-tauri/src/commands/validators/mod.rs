use super::*;

mod audio;
use self::audio::{
    validate_set_playback_speed_input, validate_set_playback_volume_input,
    validate_set_tts_voice_input,
};
mod element;
use self::element::{
    validate_click_element_input, validate_find_element_input, validate_focus_element_input,
    validate_submit_active_form_input, validate_type_into_element_input,
};
mod extraction;
use self::extraction::{
    validate_capture_screenshot_input, validate_merge_ocr_into_page_model_input,
    validate_run_ocr_input,
};
mod navigation;
use self::navigation::{
    validate_eval_js_input, validate_go_back_input, validate_go_forward_input,
    validate_open_url_input, validate_scroll_page_input,
};
mod planner;
pub(crate) use self::planner::validate_confirm_action_input;
use self::planner::{validate_report_result_input, validate_step_transition};
mod voice;
use self::voice::{validate_read_region_input, validate_transcribe_command_input};

pub fn validate_planner_output(
    planner_output: &PlannerOutput,
    available_tools: &[AvailableTool],
    active_skill_names: &[String],
) -> Result<(), ToolError> {
    let available_tool_names = available_tools
        .iter()
        .map(|tool| tool.name.clone())
        .collect::<Vec<_>>();
    let active_skill_name_set = active_skill_names.iter().cloned().collect::<HashSet<_>>();

    if planner_output.steps.len() > MAX_INITIAL_PLAN_STEPS {
        return Err(invalid_planner_output(
            format!(
                "planner returned {} steps, exceeding the v1 maximum of {}",
                planner_output.steps.len(),
                MAX_INITIAL_PLAN_STEPS
            ),
            None,
        ));
    }

    match planner_output.status {
        PlannerStatus::Ready | PlannerStatus::NeedsConfirmation => {
            if planner_output.steps.is_empty() {
                return Err(invalid_planner_output(
                    "planner returned no executable steps for an executing status",
                    None,
                ));
            }
        }
        PlannerStatus::Blocked => {
            if !planner_output.steps.is_empty() {
                return Err(invalid_planner_output(
                    "blocked planner output must not include executable steps",
                    None,
                ));
            }
            if planner_output.blocked_reason.is_none() {
                return Err(invalid_planner_output(
                    "blocked planner output must include blocked_reason",
                    None,
                ));
            }
            if planner_output
                .user_message
                .as_ref()
                .is_none_or(|message| message.trim().is_empty())
            {
                return Err(invalid_planner_output(
                    "blocked planner output must include a non-empty user_message",
                    None,
                ));
            }
        }
        PlannerStatus::Complete => {
            if !planner_output.steps.is_empty() {
                return Err(invalid_planner_output(
                    "complete planner output must not include executable steps",
                    None,
                ));
            }
        }
    }

    validate_submit_confirmation_policy(planner_output)?;
    validate_confirmation_policy(planner_output)?;

    let mut seen_step_ids = HashSet::new();
    for step in &planner_output.steps {
        if step.step_id.trim().is_empty() {
            return Err(invalid_planner_output(
                "planner step ids must be non-empty",
                Some(serde_json::json!({ "tool_name": step.tool_name })),
            ));
        }
        if !seen_step_ids.insert(step.step_id.clone()) {
            return Err(invalid_planner_output(
                format!("planner returned duplicate step id '{}'", step.step_id),
                Some(serde_json::json!({ "step_id": step.step_id })),
            ));
        }
        if step.purpose.trim().is_empty() {
            return Err(invalid_planner_output(
                "planner steps must include a non-empty purpose",
                Some(serde_json::json!({ "step_id": step.step_id })),
            ));
        }
        if !available_tool_names
            .iter()
            .any(|tool_name| tool_name == &step.tool_name)
        {
            return Err(invalid_planner_output(
                format!("planner referenced unavailable tool {:?}", step.tool_name),
                Some(serde_json::json!({ "step_id": step.step_id })),
            ));
        }
        validate_planned_step_arguments(step)?;
    }

    for step in &planner_output.steps {
        validate_step_transition(&step.on_success, &seen_step_ids, &step.step_id)?;
        validate_step_transition(&step.on_failure, &seen_step_ids, &step.step_id)?;
    }

    for skill_name in &planner_output.selected_skills {
        if !active_skill_name_set.contains(skill_name) {
            return Err(invalid_planner_output(
                format!("planner selected unknown or ineligible skill '{skill_name}'"),
                None,
            ));
        }
    }

    Ok(())
}

pub fn validate_planner_output_with_safety(
    planner_output: &PlannerOutput,
    available_tools: &[AvailableTool],
    active_skill_names: &[String],
    safety: &PlannerSafetySettings,
) -> Result<(), ToolError> {
    // Prohibited actions are security policy violations, not ordinary schema
    // mistakes. Reject them before less-specific structural validation so the
    // caller and audit trail retain the authoritative reason code.
    let preliminary_decision = evaluate_action_policy(&planner_output.steps, safety);
    if preliminary_decision.requirement == ConfirmationRequirement::Prohibited {
        return Err(ToolError {
            code: String::from("prohibited_planner_action"),
            message: String::from(
                "planner output contains an action prohibited by deterministic runtime policy",
            ),
            retryable: false,
            details: serde_json::to_value(preliminary_decision).ok(),
        });
    }

    // Preserve all schema, transition, availability, and legacy metadata checks,
    // then apply the authoritative runtime policy derived from actual tools.
    validate_planner_output(planner_output, available_tools, active_skill_names)?;
    validate_protected_intent_consistency(planner_output)?;

    let decision = evaluate_action_policy(&planner_output.steps, safety);
    match decision.requirement {
        ConfirmationRequirement::Prohibited => Err(ToolError {
            code: String::from("prohibited_planner_action"),
            message: String::from(
                "planner output contains an action prohibited by deterministic runtime policy",
            ),
            retryable: false,
            details: serde_json::to_value(&decision).ok(),
        }),
        ConfirmationRequirement::ConfirmationRequired => {
            validate_runtime_confirmation_gate(planner_output, &decision)
        }
        ConfirmationRequirement::NoConfirmation => {
            if planner_output.status == PlannerStatus::NeedsConfirmation {
                return Err(invalid_planner_output(
                    "planner requested confirmation for a plan with no runtime-protected action",
                    serde_json::to_value(&decision).ok(),
                ));
            }
            Ok(())
        }
    }
}

fn validate_runtime_confirmation_gate(
    planner_output: &PlannerOutput,
    decision: &ActionPolicyDecision,
) -> Result<(), ToolError> {
    if planner_output.status != PlannerStatus::NeedsConfirmation {
        return Err(ToolError {
            code: String::from("confirmation_required_by_runtime_policy"),
            message: String::from(
                "planner marked a runtime-protected action ready without confirmation",
            ),
            retryable: false,
            details: serde_json::to_value(decision).ok(),
        });
    }

    if !planner_output.requires_confirmation {
        return Err(invalid_planner_output(
            "runtime-protected plan must set requires_confirmation",
            serde_json::to_value(decision).ok(),
        ));
    }

    let confirm_steps = planner_output
        .steps
        .iter()
        .filter(|step| step.tool_name == ToolName::ConfirmAction)
        .collect::<Vec<_>>();
    if confirm_steps.len() != 1 {
        return Err(invalid_planner_output(
            "runtime-protected plan must contain exactly one confirm_action step",
            serde_json::to_value(decision).ok(),
        ));
    }

    let first_step = planner_output.steps.first().ok_or_else(|| {
        invalid_planner_output(
            "runtime-protected plan has no confirmation gate",
            serde_json::to_value(decision).ok(),
        )
    })?;
    if first_step.tool_name != ToolName::ConfirmAction {
        return Err(invalid_planner_output(
            "confirm_action must be the first step of every runtime-protected plan",
            serde_json::to_value(decision).ok(),
        ));
    }
    if !matches!(&first_step.on_success, StepTransition::RequestConfirmation) {
        return Err(invalid_planner_output(
            "confirm_action success must transition to RequestConfirmation",
            serde_json::to_value(decision).ok(),
        ));
    }
    if !matches!(&first_step.on_failure, StepTransition::Replan) {
        return Err(invalid_planner_output(
            "confirm_action failure must replan and may not route to a protected action",
            serde_json::to_value(decision).ok(),
        ));
    }

    Ok(())
}

fn validate_protected_intent_consistency(planner_output: &PlannerOutput) -> Result<(), ToolError> {
    let has_explicit_submit = planner_output
        .steps
        .iter()
        .any(|step| step.tool_name == ToolName::SubmitActiveForm);
    let has_submit_via_text_entry = planner_output.steps.iter().any(|step| {
        if step.tool_name != ToolName::TypeIntoElement {
            return false;
        }
        serde_json::from_value::<TypeIntoElementInput>(step.arguments.clone())
            .map(|input| input.submit_mode.submits_after_entry())
            .unwrap_or(true)
    });

    if (has_explicit_submit || has_submit_via_text_entry)
        && planner_output.intent.name != IntentName::SubmitForm
    {
        return Err(invalid_planner_output(
            "planner intent is inconsistent with an actual form-submission action",
            Some(serde_json::json!({
                "intent": planner_output.intent.name,
                "has_submit_active_form": has_explicit_submit,
                "has_submit_via_text_entry": has_submit_via_text_entry,
            })),
        ));
    }

    let has_click = planner_output
        .steps
        .iter()
        .any(|step| step.tool_name == ToolName::ClickElement);
    if has_click
        && !has_explicit_submit
        && !has_submit_via_text_entry
        && planner_output.intent.name != IntentName::ClickElement
    {
        return Err(invalid_planner_output(
            "planner intent is inconsistent with an actual click action",
            Some(serde_json::json!({
                "intent": planner_output.intent.name,
            })),
        ));
    }

    Ok(())
}

fn validate_submit_confirmation_policy(planner_output: &PlannerOutput) -> Result<(), ToolError> {
    if planner_output.intent.name != IntentName::SubmitForm {
        return Ok(());
    }

    if planner_output.status != PlannerStatus::NeedsConfirmation {
        return Err(invalid_planner_output(
            "submit-form planner output must use NeedsConfirmation status",
            None,
        ));
    }

    if !planner_output.requires_confirmation {
        return Err(invalid_planner_output(
            "submit-form planner output must set requires_confirmation",
            None,
        ));
    }

    if planner_output
        .confirmation_reason
        .as_ref()
        .is_none_or(|reason| reason.trim().is_empty())
    {
        return Err(invalid_planner_output(
            "submit-form planner output must include confirmation_reason",
            None,
        ));
    }

    if planner_output
        .user_message
        .as_ref()
        .is_none_or(|message| message.trim().is_empty())
    {
        return Err(invalid_planner_output(
            "submit-form planner output must include user_message",
            None,
        ));
    }

    let has_confirm_action_step = planner_output
        .steps
        .iter()
        .any(|step| step.tool_name == ToolName::ConfirmAction);
    if !has_confirm_action_step {
        return Err(invalid_planner_output(
            "submit-form planner output must include a confirm_action step",
            None,
        ));
    }

    let requests_confirmation = planner_output.steps.iter().any(|step| {
        step.tool_name == ToolName::ConfirmAction
            && matches!(step.on_success, StepTransition::RequestConfirmation)
    });
    if !requests_confirmation {
        return Err(invalid_planner_output(
            "submit-form planner output must request confirmation from a confirm_action step",
            None,
        ));
    }

    Ok(())
}

fn validate_confirmation_policy(planner_output: &PlannerOutput) -> Result<(), ToolError> {
    let has_confirm_action_step = planner_output
        .steps
        .iter()
        .any(|step| step.tool_name == ToolName::ConfirmAction);
    let requests_confirmation = planner_output.steps.iter().any(|step| {
        step.tool_name == ToolName::ConfirmAction
            && matches!(step.on_success, StepTransition::RequestConfirmation)
    });
    let has_confirmation_reason = planner_output
        .confirmation_reason
        .as_ref()
        .is_some_and(|reason| !reason.trim().is_empty());

    if planner_output.status == PlannerStatus::NeedsConfirmation {
        if !planner_output.requires_confirmation {
            return Err(invalid_planner_output(
                "needs-confirmation planner output must set requires_confirmation",
                None,
            ));
        }
        if !has_confirmation_reason {
            return Err(invalid_planner_output(
                "needs-confirmation planner output must include confirmation_reason",
                None,
            ));
        }
        if planner_output
            .user_message
            .as_ref()
            .is_none_or(|message| message.trim().is_empty())
        {
            return Err(invalid_planner_output(
                "needs-confirmation planner output must include user_message",
                None,
            ));
        }
        if !has_confirm_action_step {
            return Err(invalid_planner_output(
                "needs-confirmation planner output must include a confirm_action step",
                None,
            ));
        }
        if !requests_confirmation {
            return Err(invalid_planner_output(
                "needs-confirmation planner output must request confirmation from a confirm_action step",
                None,
            ));
        }
        return Ok(());
    }

    if planner_output.requires_confirmation {
        return Err(invalid_planner_output(
            "non-needs-confirmation planner output must not set requires_confirmation",
            None,
        ));
    }
    if has_confirmation_reason {
        return Err(invalid_planner_output(
            "non-needs-confirmation planner output must not include confirmation_reason",
            None,
        ));
    }
    if has_confirm_action_step {
        return Err(invalid_planner_output(
            "non-needs-confirmation planner output must not include a confirm_action step",
            None,
        ));
    }
    if requests_confirmation {
        return Err(invalid_planner_output(
            "non-needs-confirmation planner output must not request confirmation",
            None,
        ));
    }

    Ok(())
}

pub(crate) fn validate_planned_step_arguments(step: &PlannedStep) -> Result<(), ToolError> {
    match step.tool_name {
        ToolName::OpenUrl => validate_open_url_input(&deserialize_tool_arguments(step)?),
        ToolName::GoBack => validate_go_back_input(&deserialize_tool_arguments(step)?),
        ToolName::GoForward => validate_go_forward_input(&deserialize_tool_arguments(step)?),
        ToolName::ReloadPage => validate_tool_arguments::<ReloadPageInput>(step),
        ToolName::GetHtml => validate_tool_arguments::<GetHtmlInput>(step),
        ToolName::EvalJs => validate_eval_js_input(&deserialize_tool_arguments(step)?),
        ToolName::ScrollPage => validate_scroll_page_input(&deserialize_tool_arguments(step)?),
        ToolName::CaptureScreenshot => {
            let input = deserialize_tool_arguments::<CaptureScreenshotInput>(step)?;
            validate_capture_screenshot_input(&input)
        }
        ToolName::RunOcr => {
            let input = deserialize_tool_arguments::<RunOcrInput>(step)?;
            validate_run_ocr_input(&input)
        }
        ToolName::MergeOcrIntoPageModel => {
            let input = deserialize_tool_arguments::<MergeOcrIntoPageModelInput>(step)?;
            validate_merge_ocr_into_page_model_input(&input)
        }
        ToolName::SetBrowserVisibility => {
            validate_tool_arguments::<SetBrowserVisibilityInput>(step)
        }
        ToolName::GetPageSnapshot => validate_tool_arguments::<GetPageSnapshotInput>(step),
        ToolName::ExtractPageModel => validate_tool_arguments::<ExtractPageModelInput>(step),
        ToolName::ListInteractiveElements => {
            validate_tool_arguments::<ListInteractiveElementsInput>(step)
        }
        ToolName::FindElement => validate_find_element_input(&deserialize_tool_arguments(step)?),
        ToolName::ClickElement => validate_click_element_input(&deserialize_tool_arguments(step)?),
        ToolName::FocusElement => validate_focus_element_input(&deserialize_tool_arguments(step)?),
        ToolName::TypeIntoElement => {
            validate_type_into_element_input(&deserialize_tool_arguments(step)?)
        }
        ToolName::SubmitActiveForm => {
            validate_submit_active_form_input(&deserialize_tool_arguments(step)?)
        }
        ToolName::ReadRegion => validate_read_region_input(&deserialize_tool_arguments(step)?),
        ToolName::ReadNextRegion => validate_tool_arguments::<ReadNextRegionInput>(step),
        ToolName::ReadPreviousRegion => validate_tool_arguments::<ReadPreviousRegionInput>(step),
        ToolName::StopSpeaking => validate_tool_arguments::<StopSpeakingInput>(step),
        ToolName::StartListening => validate_tool_arguments::<StartListeningInput>(step),
        ToolName::StopListening => validate_tool_arguments::<StopListeningInput>(step),
        ToolName::TranscribeCommand => {
            validate_transcribe_command_input(&deserialize_tool_arguments(step)?)
        }
        ToolName::SetTtsVoice => validate_set_tts_voice_input(&deserialize_tool_arguments(step)?),
        ToolName::SetPlaybackVolume => {
            validate_set_playback_volume_input(&deserialize_tool_arguments(step)?)
        }
        ToolName::SetPlaybackSpeed => {
            validate_set_playback_speed_input(&deserialize_tool_arguments(step)?)
        }
        ToolName::GetAgentState => validate_tool_arguments::<GetAgentStateInput>(step),
        ToolName::GetRuntimeStatus => validate_tool_arguments::<GetRuntimeStatusInput>(step),
        ToolName::ConfirmAction => {
            validate_confirm_action_input(&deserialize_tool_arguments(step)?)
        }
        ToolName::ReportResult => validate_report_result_input(&deserialize_tool_arguments(step)?),
    }
}

fn deserialize_tool_arguments<T: serde::de::DeserializeOwned>(
    step: &PlannedStep,
) -> Result<T, ToolError> {
    serde_json::from_value::<T>(step.arguments.clone()).map_err(|error| {
        invalid_planner_output(
            format!("tool arguments did not match the expected schema: {error}"),
            Some(serde_json::json!({
                "step_id": step.step_id,
                "tool_name": step.tool_name,
            })),
        )
    })
}

fn invalid_planner_output(
    message: impl Into<String>,
    details: Option<serde_json::Value>,
) -> ToolError {
    ToolError {
        code: String::from("invalid_planner_output"),
        message: message.into(),
        retryable: false,
        details,
    }
}

fn validate_tool_arguments<Input>(step: &PlannedStep) -> Result<(), ToolError>
where
    Input: serde::de::DeserializeOwned,
{
    serde_json::from_value::<Input>(step.arguments.clone())
        .map(|_| ())
        .map_err(|error| {
            invalid_planner_output(
                format!("tool arguments did not match the expected schema: {error}"),
                Some(serde_json::json!({
                    "step_id": step.step_id,
                    "tool_name": step.tool_name,
                })),
            )
        })
}
