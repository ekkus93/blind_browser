use super::*;

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

fn validate_open_url_input(input: &OpenUrlInput) -> Result<(), ToolError> {
    let trimmed = input.url.trim();
    if trimmed.is_empty() {
        return Err(invalid_planner_output(
            "open_url requires a non-empty url",
            None,
        ));
    }

    let Some(separator_index) = trimmed.find(':') else {
        return Err(invalid_planner_output(
            "open_url requires an absolute URL with a scheme",
            Some(serde_json::json!({ "url": trimmed })),
        ));
    };

    let scheme = &trimmed[..separator_index];
    let remainder = &trimmed[separator_index + 1..];
    let valid_scheme = scheme.chars().enumerate().all(|(index, ch)| match index {
        0 => ch.is_ascii_alphabetic(),
        _ => ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'),
    });

    if !valid_scheme || remainder.is_empty() {
        return Err(invalid_planner_output(
            "open_url requires an absolute URL with a valid scheme",
            Some(serde_json::json!({ "url": trimmed })),
        ));
    }

    Ok(())
}

fn validate_go_back_input(input: &GoBackInput) -> Result<(), ToolError> {
    validate_history_steps("go_back", input.steps)
}

fn validate_go_forward_input(input: &GoForwardInput) -> Result<(), ToolError> {
    validate_history_steps("go_forward", input.steps)
}

fn validate_eval_js_input(input: &EvalJsInput) -> Result<(), ToolError> {
    let trimmed = input.expression.trim();
    if trimmed.is_empty() {
        return Err(invalid_planner_output(
            "eval_js requires a non-empty expression",
            None,
        ));
    }

    Ok(())
}

fn validate_history_steps(tool_name: &str, steps: Option<u8>) -> Result<(), ToolError> {
    if matches!(steps, Some(0)) {
        return Err(invalid_planner_output(
            format!("{tool_name} steps must be greater than 0 when provided"),
            None,
        ));
    }

    if let Some(steps) = steps {
        if steps > MAX_HISTORY_STEPS {
            return Err(invalid_planner_output(
                format!(
                    "{tool_name} steps must be less than or equal to {MAX_HISTORY_STEPS} when provided"
                ),
                Some(serde_json::json!({ "steps": steps })),
            ));
        }
    }

    Ok(())
}

fn validate_scroll_page_input(input: &ScrollPageInput) -> Result<(), ToolError> {
    if input.amount_px.is_none() && input.target.is_none() {
        return Err(invalid_planner_output(
            "scroll_page requires amount_px or target to be provided",
            None,
        ));
    }

    if let Some(amount_px) = input.amount_px {
        if !amount_px.is_finite() {
            return Err(invalid_planner_output(
                "scroll_page amount_px must be a finite number when provided",
                None,
            ));
        }

        if amount_px <= 0.0 {
            return Err(invalid_planner_output(
                "scroll_page amount_px must be greater than 0 when provided",
                Some(serde_json::json!({ "amount_px": amount_px })),
            ));
        }
    }

    Ok(())
}

fn validate_capture_screenshot_input(input: &CaptureScreenshotInput) -> Result<(), ToolError> {
    let region_id_active = input
        .region_id
        .as_deref()
        .map(str::trim)
        .is_some_and(|region_id| !region_id.is_empty());
    let targeting_modes = usize::from(input.scope.captures_full_page())
        + usize::from(region_id_active)
        + usize::from(input.bbox.is_some());
    if targeting_modes > 1 {
        return Err(invalid_planner_output(
            "capture_screenshot supports at most one targeting mode from scope = FullPage, region_id, or bbox",
            None,
        ));
    }

    if let Some(bbox) = input.bbox.as_ref() {
        if bbox.width <= 0.0 || bbox.height <= 0.0 {
            return Err(invalid_planner_output(
                "capture_screenshot bbox requires positive width and height",
                Some(serde_json::json!({
                    "x": bbox.x,
                    "y": bbox.y,
                    "width": bbox.width,
                    "height": bbox.height,
                })),
            ));
        }
    }

    Ok(())
}

fn validate_run_ocr_input(input: &RunOcrInput) -> Result<(), ToolError> {
    let image_id_active = input
        .image_id
        .as_deref()
        .map(str::trim)
        .is_some_and(|image_id| !image_id.is_empty());
    let region_id_active = input
        .region_id
        .as_deref()
        .map(str::trim)
        .is_some_and(|region_id| !region_id.is_empty());

    if !(image_id_active || region_id_active || input.bbox.is_some()) {
        return Err(invalid_planner_output(
            "run_ocr requires at least one source from image_id, region_id, or bbox",
            None,
        ));
    }

    if let Some(bbox) = input.bbox.as_ref() {
        if bbox.width <= 0.0 || bbox.height <= 0.0 {
            return Err(invalid_planner_output(
                "run_ocr bbox requires positive width and height",
                Some(serde_json::json!({
                    "x": bbox.x,
                    "y": bbox.y,
                    "width": bbox.width,
                    "height": bbox.height,
                })),
            ));
        }
    }

    Ok(())
}

fn validate_merge_ocr_into_page_model_input(
    input: &MergeOcrIntoPageModelInput,
) -> Result<(), ToolError> {
    if input.page_id.trim().is_empty() {
        return Err(invalid_planner_output(
            "merge_ocr_into_page_model requires a non-empty page_id",
            None,
        ));
    }

    if input.ocr_text.trim().is_empty() {
        return Err(invalid_planner_output(
            "merge_ocr_into_page_model requires non-empty ocr_text",
            None,
        ));
    }

    if let Some(region_id) = input.region_id.as_deref() {
        if region_id.trim().is_empty() {
            return Err(invalid_planner_output(
                "merge_ocr_into_page_model region_id must be non-empty when provided",
                None,
            ));
        }
    }

    if let Some(bbox) = input.source_bbox.as_ref() {
        if bbox.width <= 0.0 || bbox.height <= 0.0 {
            return Err(invalid_planner_output(
                "merge_ocr_into_page_model source_bbox requires positive width and height",
                Some(serde_json::json!({
                    "x": bbox.x,
                    "y": bbox.y,
                    "width": bbox.width,
                    "height": bbox.height,
                })),
            ));
        }
    }

    Ok(())
}

fn validate_find_element_input(input: &FindElementInput) -> Result<(), ToolError> {
    if input.description.trim().is_empty() {
        return Err(invalid_planner_output(
            "find_element requires a non-empty description",
            None,
        ));
    }

    for (field_name, value) in [
        ("text", input.text.as_deref()),
        ("color_hint", input.color_hint.as_deref()),
        ("nearby_text", input.nearby_text.as_deref()),
        ("selector_hint", input.selector_hint.as_deref()),
    ] {
        if let Some(value) = value {
            if value.trim().is_empty() {
                return Err(invalid_planner_output(
                    format!("find_element {field_name} must be non-empty when provided"),
                    None,
                ));
            }
        }
    }

    if matches!(input.max_candidates, Some(0)) {
        return Err(invalid_planner_output(
            "find_element max_candidates must be greater than 0 when provided",
            None,
        ));
    }

    if let Some(max_candidates) = input.max_candidates {
        if max_candidates > DEFAULT_FIND_ELEMENT_MAX_CANDIDATES {
            return Err(invalid_planner_output(
                format!(
                    "find_element max_candidates must be less than or equal to {DEFAULT_FIND_ELEMENT_MAX_CANDIDATES} when provided"
                ),
                Some(serde_json::json!({ "max_candidates": max_candidates })),
            ));
        }
    }

    Ok(())
}

fn validate_click_element_input(input: &ClickElementInput) -> Result<(), ToolError> {
    if input.element_id.trim().is_empty() {
        return Err(invalid_planner_output(
            "click_element requires a non-empty element_id",
            None,
        ));
    }

    Ok(())
}

fn validate_focus_element_input(input: &FocusElementInput) -> Result<(), ToolError> {
    if input.element_id.trim().is_empty() {
        return Err(invalid_planner_output(
            "focus_element requires a non-empty element_id",
            None,
        ));
    }

    Ok(())
}

fn validate_type_into_element_input(input: &TypeIntoElementInput) -> Result<(), ToolError> {
    if input.element_id.trim().is_empty() {
        return Err(invalid_planner_output(
            "type_into_element requires a non-empty element_id",
            None,
        ));
    }

    Ok(())
}

fn validate_submit_active_form_input(input: &SubmitActiveFormInput) -> Result<(), ToolError> {
    if let Some(form_element_id) = input.form_element_id.as_deref() {
        if form_element_id.trim().is_empty() {
            return Err(invalid_planner_output(
                "submit_active_form form_element_id must be non-empty when provided",
                None,
            ));
        }
    }

    Ok(())
}

fn validate_read_region_input(input: &ReadRegionInput) -> Result<(), ToolError> {
    if input.region_id.trim().is_empty() {
        return Err(invalid_planner_output(
            "read_region requires a non-empty region_id",
            None,
        ));
    }

    Ok(())
}

fn validate_transcribe_command_input(input: &TranscribeCommandInput) -> Result<(), ToolError> {
    if matches!(input.max_duration_ms, Some(0)) {
        return Err(invalid_planner_output(
            "transcribe_command max_duration_ms must be greater than 0 when provided",
            None,
        ));
    }

    Ok(())
}

fn validate_set_tts_voice_input(_input: &SetTtsVoiceInput) -> Result<(), ToolError> {
    Ok(())
}

fn validate_set_playback_volume_input(input: &SetPlaybackVolumeInput) -> Result<(), ToolError> {
    if !input.volume.is_finite() {
        return Err(invalid_planner_output(
            "set_playback_volume requires a finite numeric volume value",
            None,
        ));
    }

    if !(0.0..=MAX_PLAYBACK_VOLUME).contains(&input.volume) {
        return Err(invalid_planner_output(
            format!("set_playback_volume volume must be between 0.0 and {MAX_PLAYBACK_VOLUME}"),
            Some(serde_json::json!({ "volume": input.volume })),
        ));
    }

    Ok(())
}

fn validate_set_playback_speed_input(input: &SetPlaybackSpeedInput) -> Result<(), ToolError> {
    if !input.speed.is_finite() {
        return Err(invalid_planner_output(
            "set_playback_speed requires a finite numeric speed value",
            None,
        ));
    }

    if !(MIN_PLAYBACK_SPEED..=MAX_PLAYBACK_SPEED).contains(&input.speed) {
        return Err(invalid_planner_output(
            format!(
                "set_playback_speed speed must be between {MIN_PLAYBACK_SPEED} and {MAX_PLAYBACK_SPEED}"
            ),
            Some(serde_json::json!({ "speed": input.speed })),
        ));
    }

    Ok(())
}

pub(crate) fn validate_confirm_action_input(input: &ConfirmActionInput) -> Result<(), ToolError> {
    if input.prompt_text.trim().is_empty() {
        return Err(invalid_planner_output(
            "confirm_action requires a non-empty prompt_text",
            None,
        ));
    }

    if input.reason.trim().is_empty() {
        return Err(invalid_planner_output(
            "confirm_action requires a non-empty reason",
            None,
        ));
    }

    Ok(())
}

fn validate_report_result_input(input: &ReportResultInput) -> Result<(), ToolError> {
    if input.summary.trim().is_empty() {
        return Err(invalid_planner_output(
            "report_result requires a non-empty summary",
            None,
        ));
    }

    for (field_name, value) in [
        (
            "next_recommended_action",
            input.next_recommended_action.as_deref(),
        ),
        ("user_message", input.user_message.as_deref()),
    ] {
        if let Some(value) = value {
            if value.trim().is_empty() {
                return Err(invalid_planner_output(
                    format!("report_result {field_name} must be non-empty when provided"),
                    None,
                ));
            }
        }
    }

    Ok(())
}

fn validate_step_transition(
    transition: &StepTransition,
    step_ids: &HashSet<String>,
    source_step_id: &str,
) -> Result<(), ToolError> {
    if let StepTransition::NextStep { step_id } = transition {
        if !step_ids.contains(step_id) {
            return Err(invalid_planner_output(
                format!(
                    "planner referenced missing next step '{}' from '{}'",
                    step_id, source_step_id
                ),
                Some(serde_json::json!({
                    "source_step_id": source_step_id,
                    "next_step_id": step_id,
                })),
            ));
        }
    }

    Ok(())
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
