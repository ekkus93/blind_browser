use super::*;

pub fn infer_intent_hint(transcript: &str) -> IntentName {
    let normalized = normalize_transcript_for_routing(transcript);

    if normalized.is_empty() {
        return IntentName::Unknown;
    }

    if is_start_listening_phrase(&normalized) {
        return IntentName::StartListening;
    }
    if is_stop_listening_phrase(&normalized) {
        return IntentName::StopListening;
    }
    if is_transcribe_command_phrase(&normalized) {
        return IntentName::TranscribeCommand;
    }
    if is_back_history_query_phrase(&normalized)
        || is_forward_history_query_phrase(&normalized)
        || is_listening_query_phrase(&normalized)
        || is_speaking_query_phrase(&normalized)
        || is_browser_mode_query_phrase(&normalized)
    {
        return IntentName::GetStatus;
    }
    if is_go_back_phrase(&normalized) {
        return IntentName::GoBack;
    }
    if is_go_forward_phrase(&normalized) {
        return IntentName::GoForward;
    }
    if is_reload_page_phrase(&normalized) {
        return IntentName::ReloadPage;
    }
    if is_current_url_query_phrase(&normalized) || normalized.contains("what page") {
        return IntentName::GetCurrentUrl;
    }
    if is_status_query_phrase(&normalized)
        || is_history_query_phrase(&normalized)
        || is_listening_query_phrase(&normalized)
        || is_speaking_query_phrase(&normalized)
        || is_browser_mode_query_phrase(&normalized)
    {
        return IntentName::GetStatus;
    }
    if is_read_next_phrase(&normalized) {
        return IntentName::ReadNext;
    }
    if is_read_previous_phrase(&normalized) {
        return IntentName::ReadPrevious;
    }
    if is_read_title_phrase(&normalized) {
        return IntentName::ReadTitle;
    }
    if is_repeat_phrase(&normalized) {
        return IntentName::Repeat;
    }
    if is_stop_phrase(&normalized) {
        return IntentName::Stop;
    }
    if is_read_page_phrase(&normalized) {
        return IntentName::ReadPage;
    }
    if is_fill_and_submit_phrase(&normalized) || is_submit_form_phrase(&normalized) {
        return IntentName::SubmitForm;
    }
    if is_fill_input_phrase(&normalized) {
        return IntentName::FillInput;
    }
    if is_set_tts_voice_phrase(&normalized) {
        return IntentName::SetTtsVoice;
    }
    if normalized.contains("open ")
        || normalized.contains("go to ")
        || normalized.contains("visit ")
    {
        return IntentName::OpenUrl;
    }
    if normalized.contains("click ") || normalized.contains("press ") {
        return IntentName::ClickElement;
    }
    if normalized.contains("find ") {
        return IntentName::FindElement;
    }
    if normalized.contains("scroll ") {
        return IntentName::Scroll;
    }
    if is_volume_query_phrase(&normalized) {
        return IntentName::GetPlaybackVolume;
    }
    if is_speed_query_phrase(&normalized) {
        return IntentName::GetPlaybackSpeed;
    }
    if normalized.contains("volume")
        || normalized.contains("mute")
        || normalized.contains("quieter")
        || normalized.contains("louder")
    {
        return IntentName::SetPlaybackVolume;
    }
    if normalized.contains("speed")
        || normalized.contains("faster")
        || normalized.contains("slower")
    {
        return IntentName::SetPlaybackSpeed;
    }
    if is_browser_visibility_phrase(&normalized) {
        return IntentName::SetBrowserVisibility;
    }

    IntentName::Unknown
}

pub(crate) fn normalize_transcript_for_routing(transcript: &str) -> String {
    normalize_command_text(transcript, false)
}

fn normalize_audio_command_text(transcript: &str) -> String {
    normalize_command_text(transcript, true)
}

fn normalize_command_text(transcript: &str, allow_decimal: bool) -> String {
    let sanitized = transcript
        .to_ascii_lowercase()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric()
                || character.is_ascii_whitespace()
                || (allow_decimal && character == '.')
            {
                character
            } else {
                ' '
            }
        })
        .collect::<String>();

    merge_compound_command_tokens(
        sanitized
            .split_whitespace()
            .map(String::from)
            .collect::<Vec<_>>(),
    )
    .into_iter()
    .map(|token| canonicalize_command_token(&token))
    .collect::<Vec<_>>()
    .join(" ")
}

fn merge_compound_command_tokens(tokens: Vec<String>) -> Vec<String> {
    let mut merged = Vec::with_capacity(tokens.len());
    let mut index = 0;

    while index < tokens.len() {
        if let Some(next_token) = tokens.get(index + 1) {
            match (tokens[index].as_str(), next_token.as_str()) {
                ("play", "back") => {
                    merged.push(String::from("playback"));
                    index += 2;
                    continue;
                }
                ("head", "less") => {
                    merged.push(String::from("headless"));
                    index += 2;
                    continue;
                }
                _ => {}
            }
        }

        merged.push(tokens[index].clone());
        index += 1;
    }

    merged
}

fn canonicalize_command_token(token: &str) -> String {
    const FUZZY_COMMAND_KEYWORDS: &[&str] = &[
        "back",
        "browser",
        "current",
        "field",
        "focus",
        "forward",
        "headless",
        "listening",
        "next",
        "playback",
        "previous",
        "reload",
        "refresh",
        "repeat",
        "speed",
        "status",
        "stop",
        "submit",
        "title",
        "transcribe",
        "url",
        "visible",
        "voice",
        "volume",
    ];

    if token.len() < 4
        || token.contains('.')
        || token.chars().any(|character| character.is_ascii_digit())
    {
        return token.to_string();
    }

    let mut matches = FUZZY_COMMAND_KEYWORDS
        .iter()
        .copied()
        .filter(|keyword| is_unambiguous_fuzzy_keyword_match(token, keyword));

    match (matches.next(), matches.next()) {
        (Some(keyword), None) => String::from(keyword),
        _ => token.to_string(),
    }
}

fn is_unambiguous_fuzzy_keyword_match(token: &str, keyword: &str) -> bool {
    if token == keyword {
        return true;
    }

    if token.len().abs_diff(keyword.len()) > 1 {
        return false;
    }

    if token
        .chars()
        .next()
        .zip(keyword.chars().next())
        .is_none_or(|(left, right)| left != right)
    {
        return false;
    }

    is_single_edit_or_transposition(token, keyword)
}

fn is_single_edit_or_transposition(left: &str, right: &str) -> bool {
    let left_chars = left.chars().collect::<Vec<_>>();
    let right_chars = right.chars().collect::<Vec<_>>();

    match left_chars.len().cmp(&right_chars.len()) {
        std::cmp::Ordering::Equal => {
            let mismatches = left_chars
                .iter()
                .zip(right_chars.iter())
                .enumerate()
                .filter_map(|(index, (left_char, right_char))| {
                    (left_char != right_char).then_some(index)
                })
                .collect::<Vec<_>>();

            match mismatches.as_slice() {
                [single_mismatch] => left_chars[*single_mismatch] != right_chars[*single_mismatch],
                [first, second] if *second == *first + 1 => {
                    left_chars[*first] == right_chars[*second]
                        && left_chars[*second] == right_chars[*first]
                }
                _ => false,
            }
        }
        std::cmp::Ordering::Less => is_single_insertion_or_deletion(&left_chars, &right_chars),
        std::cmp::Ordering::Greater => is_single_insertion_or_deletion(&right_chars, &left_chars),
    }
}

fn is_single_insertion_or_deletion(shorter: &[char], longer: &[char]) -> bool {
    let mut shorter_index = 0;
    let mut longer_index = 0;
    let mut skipped = false;

    while shorter_index < shorter.len() && longer_index < longer.len() {
        if shorter[shorter_index] == longer[longer_index] {
            shorter_index += 1;
            longer_index += 1;
            continue;
        }

        if skipped {
            return false;
        }

        skipped = true;
        longer_index += 1;
    }

    true
}

pub(crate) fn tokenize_text(text: &str) -> HashSet<String> {
    normalize_transcript_for_routing(text)
        .split_whitespace()
        .filter(|token| token.len() > 1)
        .map(String::from)
        .collect()
}

pub(crate) fn likely_tools_for_intent(intent: &IntentName) -> Vec<ToolName> {
    match intent {
        IntentName::OpenUrl => vec![ToolName::OpenUrl],
        IntentName::GoBack => vec![ToolName::GoBack],
        IntentName::GoForward => vec![ToolName::GoForward],
        IntentName::ReloadPage => vec![ToolName::ReloadPage],
        IntentName::GetCurrentUrl => vec![ToolName::GetAgentState, ToolName::ReportResult],
        IntentName::ReadPage => vec![ToolName::ExtractPageModel, ToolName::ReadRegion],
        IntentName::ReadTitle => vec![ToolName::ReportResult],
        IntentName::ReadNext => vec![ToolName::ReadNextRegion],
        IntentName::ReadPrevious => vec![ToolName::ReadPreviousRegion],
        IntentName::Repeat => vec![ToolName::GetAgentState, ToolName::ReadRegion],
        IntentName::Stop => vec![ToolName::StopSpeaking],
        IntentName::StartListening => vec![ToolName::StartListening],
        IntentName::StopListening => vec![ToolName::StopListening],
        IntentName::TranscribeCommand => vec![ToolName::TranscribeCommand],
        IntentName::SetTtsVoice => vec![ToolName::SetTtsVoice, ToolName::ReportResult],
        IntentName::SetPlaybackVolume => vec![ToolName::SetPlaybackVolume],
        IntentName::SetPlaybackSpeed => vec![ToolName::SetPlaybackSpeed],
        IntentName::GetPlaybackVolume | IntentName::GetPlaybackSpeed => {
            vec![ToolName::GetRuntimeStatus, ToolName::ReportResult]
        }
        IntentName::SetBrowserVisibility => vec![ToolName::SetBrowserVisibility],
        IntentName::GetStatus => vec![ToolName::GetRuntimeStatus, ToolName::ReportResult],
        IntentName::FindElement => vec![ToolName::FindElement],
        IntentName::ClickElement => vec![ToolName::FindElement, ToolName::ClickElement],
        IntentName::FillInput => vec![ToolName::FocusElement, ToolName::TypeIntoElement],
        IntentName::SubmitForm => vec![ToolName::ConfirmAction, ToolName::SubmitActiveForm],
        IntentName::Scroll => vec![ToolName::ScrollPage],
        IntentName::OcrRecovery => vec![
            ToolName::CaptureScreenshot,
            ToolName::RunOcr,
            ToolName::MergeOcrIntoPageModel,
            ToolName::GetPageSnapshot,
            ToolName::ReportResult,
        ],
        IntentName::Unknown => Vec::new(),
    }
}

pub(crate) fn resolve_direct_audio_command(
    transcript: &str,
    request_id: &str,
    current_volume: f32,
    current_speed: f32,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_audio_command_text(transcript);
    if normalized.is_empty() {
        return None;
    }

    if is_volume_query_phrase(&normalized) {
        let summary = format!(
            "Playback volume is {}.",
            format_playback_volume(current_volume)
        );
        return Some(build_audio_report_planner_output(
            request_id,
            IntentName::GetPlaybackVolume,
            String::from("Report the current playback volume."),
            selected_audio_skill(active_skill_names, "get_volume"),
            Some(format_playback_volume(current_volume)),
            summary,
        ));
    }

    if is_speed_query_phrase(&normalized) {
        let summary = format!(
            "Playback speed is {}.",
            format_playback_speed(current_speed)
        );
        return Some(build_audio_report_planner_output(
            request_id,
            IntentName::GetPlaybackSpeed,
            String::from("Report the current playback speed."),
            selected_audio_skill(active_skill_names, "get_playback_speed"),
            Some(format_playback_speed(current_speed)),
            summary,
        ));
    }

    if let Some(volume) = parse_volume_command(&normalized, current_volume) {
        let summary = format!(
            "Playback volume set to {}.",
            format_playback_volume(volume.value)
        );
        return Some(build_audio_set_planner_output(AudioSetPlanSpec {
            request_id,
            intent_name: IntentName::SetPlaybackVolume,
            goal: volume.goal,
            selected_skills: selected_audio_skill(active_skill_names, volume.skill_name),
            target_description: Some(format_playback_volume(volume.value)),
            set_step_id: "set-playback-volume",
            tool_name: ToolName::SetPlaybackVolume,
            tool_arguments: serde_json::json!({
                "request_id": request_id,
                "timeout_ms": serde_json::Value::Null,
                "volume": volume.value
            }),
            tool_purpose: String::from("Apply and persist the requested playback volume."),
            report_step_id: "report-playback-volume",
            report_summary: summary,
        }));
    }

    if let Some(speed) = parse_speed_command(&normalized, current_speed) {
        let summary = format!(
            "Playback speed set to {}.",
            format_playback_speed(speed.value)
        );
        return Some(build_audio_set_planner_output(AudioSetPlanSpec {
            request_id,
            intent_name: IntentName::SetPlaybackSpeed,
            goal: speed.goal,
            selected_skills: selected_audio_skill(active_skill_names, speed.skill_name),
            target_description: Some(format_playback_speed(speed.value)),
            set_step_id: "set-playback-speed",
            tool_name: ToolName::SetPlaybackSpeed,
            tool_arguments: serde_json::json!({
                "request_id": request_id,
                "timeout_ms": serde_json::Value::Null,
                "speed": speed.value
            }),
            tool_purpose: String::from("Apply and persist the requested playback speed."),
            report_step_id: "report-playback-speed",
            report_summary: summary,
        }));
    }

    None
}

pub(crate) fn resolve_direct_browser_visibility_command(
    transcript: &str,
    request_id: &str,
    current_visibility: BrowserVisibilityMode,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() {
        return None;
    }

    let target_mode = parse_browser_visibility_command(&normalized, current_visibility)?;
    let summary = format!(
        "Browser mode set to {}.",
        format_browser_visibility_mode(target_mode)
    );

    Some(build_browser_visibility_planner_output(
        request_id,
        target_mode,
        selected_skill(active_skill_names, "toggle_browser_visibility"),
        summary,
    ))
}

pub(crate) fn resolve_direct_navigation_readback_command(
    transcript: &str,
    request_id: &str,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() {
        return None;
    }

    if is_go_back_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::GoBack,
                goal: String::from("Navigate back one entry in browser history."),
                target_description: None,
            },
            selected_skill(active_skill_names, "go_back"),
            PlannedStep {
                step_id: String::from("go-back"),
                tool_name: ToolName::GoBack,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "steps": 1,
                    "wait_for_load_state": LoadState::Load
                }),
                purpose: String::from("Move back to the previous history entry."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    if is_go_forward_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::GoForward,
                goal: String::from("Navigate forward one entry in browser history."),
                target_description: None,
            },
            selected_skill(active_skill_names, "go_forward"),
            PlannedStep {
                step_id: String::from("go-forward"),
                tool_name: ToolName::GoForward,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "steps": 1,
                    "wait_for_load_state": LoadState::Load
                }),
                purpose: String::from("Move forward to the next history entry."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    if is_reload_page_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::ReloadPage,
                goal: String::from("Reload the current page."),
                target_description: Some(String::from("current page")),
            },
            selected_skill(active_skill_names, "reload_page"),
            PlannedStep {
                step_id: String::from("reload-page"),
                tool_name: ToolName::ReloadPage,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "mode": "Standard",
                    "wait_for_load_state": LoadState::Load
                }),
                purpose: String::from("Reload the current page and wait for it to finish loading."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    if is_read_next_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::ReadNext,
                goal: String::from("Read the next narration region."),
                target_description: Some(String::from("next narration region")),
            },
            selected_skill(active_skill_names, "read_next"),
            PlannedStep {
                step_id: String::from("read-next-region"),
                tool_name: ToolName::ReadNextRegion,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "interruption_mode": "Interrupt"
                }),
                purpose: String::from("Move narration to the next region and start reading it."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    if is_read_previous_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::ReadPrevious,
                goal: String::from("Read the previous narration region."),
                target_description: Some(String::from("previous narration region")),
            },
            selected_skill(active_skill_names, "read_previous"),
            PlannedStep {
                step_id: String::from("read-previous-region"),
                tool_name: ToolName::ReadPreviousRegion,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "interruption_mode": "Interrupt"
                }),
                purpose: String::from(
                    "Move narration to the previous region and start reading it.",
                ),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    if is_stop_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::Stop,
                goal: String::from("Stop current speech output."),
                target_description: Some(String::from("speech output")),
            },
            selected_stop_skill(active_skill_names),
            PlannedStep {
                step_id: String::from("stop-speaking"),
                tool_name: ToolName::StopSpeaking,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null
                }),
                purpose: String::from("Stop any current spoken narration or playback."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    None
}

pub(crate) fn resolve_direct_voice_input_command(
    transcript: &str,
    request_id: &str,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() {
        return None;
    }

    if is_start_listening_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::StartListening,
                goal: String::from("Start listening for voice input."),
                target_description: Some(String::from("voice input")),
            },
            selected_skill(active_skill_names, "start_listening"),
            PlannedStep {
                step_id: String::from("start-listening"),
                tool_name: ToolName::StartListening,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null
                }),
                purpose: String::from("Start listening for the next spoken command."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    if is_stop_listening_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::StopListening,
                goal: String::from("Stop listening for voice input."),
                target_description: Some(String::from("voice input")),
            },
            selected_skill(active_skill_names, "stop_listening"),
            PlannedStep {
                step_id: String::from("stop-listening"),
                tool_name: ToolName::StopListening,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null
                }),
                purpose: String::from("Stop active voice listening."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    if is_transcribe_command_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::TranscribeCommand,
                goal: String::from("Capture and transcribe a short spoken command."),
                target_description: Some(String::from("spoken command")),
            },
            selected_skill(active_skill_names, "transcribe_command"),
            PlannedStep {
                step_id: String::from("transcribe-command"),
                tool_name: ToolName::TranscribeCommand,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "max_duration_ms": serde_json::Value::Null,
                    "stop_mode": "AutoStop"
                }),
                purpose: String::from("Capture and transcribe a bounded spoken command."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    None
}

pub(crate) fn resolve_direct_open_url_command(
    transcript: &str,
    request_id: &str,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let url = parse_direct_open_url_target(transcript)?;

    Some(build_single_step_planner_output(
        IntentSummary {
            name: IntentName::OpenUrl,
            goal: String::from("Open the requested URL."),
            target_description: Some(url.clone()),
        },
        selected_skill(active_skill_names, "open_url"),
        PlannedStep {
            step_id: String::from("open-url"),
            tool_name: ToolName::OpenUrl,
            arguments: serde_json::json!({
                "request_id": request_id,
                "timeout_ms": serde_json::Value::Null,
                "url": url,
                "wait_for_load_state": LoadState::Load
            }),
            purpose: String::from("Open the requested URL and wait for the page to load."),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
    ))
}

pub(crate) fn resolve_direct_read_page_command(
    transcript: &str,
    request_id: &str,
    current_page: Option<&PageModel>,
    agent_state: &AgentStateData,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() || !is_read_page_phrase(&normalized) {
        return None;
    }

    let selected_skills = selected_skill(active_skill_names, "read_page");

    if agent_state.page_id.is_none() {
        let summary = String::from("There is no current page to read yet.");
        return Some(PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::ReadPage,
                goal: String::from("Read the current page from the beginning."),
                target_description: Some(String::from("current page")),
            },
            selected_skills,
            steps: vec![PlannedStep {
                step_id: String::from("report-missing-page"),
                tool_name: ToolName::ReportResult,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "status": ReportStatus::NeedsFollowUp,
                    "summary": summary.clone(),
                    "next_recommended_action": "Open a page first, then ask me to read it.",
                    "user_message": summary
                }),
                purpose: String::from("Report that there is no active page available to read."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        });
    }

    if let Some(region_id) = current_page.and_then(first_readable_region_id) {
        return Some(PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::ReadPage,
                goal: String::from("Read the current page from the beginning."),
                target_description: Some(String::from("current page")),
            },
            selected_skills,
            steps: vec![PlannedStep {
                step_id: String::from("read-page-from-start"),
                tool_name: ToolName::ReadRegion,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "region_id": region_id,
                    "interruption_mode": "Interrupt"
                }),
                purpose: String::from(
                    "Restart narration from the first readable region of the current page.",
                ),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        });
    }

    Some(PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::ReadPage,
            goal: String::from("Read the current page from the beginning."),
            target_description: Some(String::from("current page")),
        },
        selected_skills,
        steps: vec![
            PlannedStep {
                step_id: String::from("extract-page-for-reading"),
                tool_name: ToolName::ExtractPageModel,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "use_dom_extraction": true,
                    "include_headings": true,
                    "include_links": true
                }),
                purpose: String::from("Refresh the readable page model before starting narration."),
                on_success: StepTransition::NextStep {
                    step_id: String::from("read-first-region"),
                },
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from("read-first-region"),
                tool_name: ToolName::ReadNextRegion,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "interruption_mode": "Interrupt"
                }),
                purpose: String::from(
                    "Start narration from the first readable region of the refreshed page.",
                ),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    })
}

pub(crate) fn resolve_direct_status_query_command(
    transcript: &str,
    request_id: &str,
    agent_state: &AgentStateData,
    runtime_status: &GetRuntimeStatusData,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() {
        return None;
    }

    if is_current_url_query_phrase(&normalized) {
        let summary = format_current_url_summary(agent_state);
        return Some(build_status_query_planner_output(StatusQueryPlanSpec {
            request_id,
            intent_name: IntentName::GetCurrentUrl,
            goal: String::from("Report the current page URL and title."),
            selected_skills: selected_skill(active_skill_names, "get_current_url"),
            target_description: Some(current_page_label(agent_state)),
            read_step_id: "get-current-url",
            read_tool_name: ToolName::GetAgentState,
            read_tool_arguments: serde_json::json!({
                "request_id": request_id,
                "include_last_transcript": false
            }),
            read_tool_purpose: String::from("Read the current agent page state."),
            report_step_id: "report-current-url",
            report_summary: summary,
        }));
    }

    if is_status_query_phrase(&normalized)
        || is_history_query_phrase(&normalized)
        || is_listening_query_phrase(&normalized)
        || is_speaking_query_phrase(&normalized)
        || is_browser_mode_query_phrase(&normalized)
    {
        let summary = if is_back_history_query_phrase(&normalized) {
            format_back_history_summary(runtime_status)
        } else if is_forward_history_query_phrase(&normalized) {
            format_forward_history_summary(runtime_status)
        } else if is_listening_query_phrase(&normalized) {
            format_listening_summary(runtime_status)
        } else if is_speaking_query_phrase(&normalized) {
            format_speaking_summary(runtime_status)
        } else if is_browser_mode_query_phrase(&normalized) {
            format_browser_mode_summary(runtime_status)
        } else {
            format_runtime_status_summary(runtime_status)
        };

        return Some(build_status_query_planner_output(StatusQueryPlanSpec {
            request_id,
            intent_name: IntentName::GetStatus,
            goal: String::from("Report the current runtime status relevant to the user's query."),
            selected_skills: selected_status_skill(active_skill_names),
            target_description: Some(String::from("runtime status")),
            read_step_id: "get-runtime-status",
            read_tool_name: ToolName::GetRuntimeStatus,
            read_tool_arguments: serde_json::json!({
                "request_id": request_id,
                "include_provider_modes": false
            }),
            read_tool_purpose: String::from("Read the current runtime status."),
            report_step_id: "report-runtime-status",
            report_summary: summary,
        }));
    }

    None
}

pub(crate) fn resolve_direct_read_title_command(
    transcript: &str,
    request_id: &str,
    agent_state: &AgentStateData,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() || !is_read_title_phrase(&normalized) {
        return None;
    }

    let summary = match normalized_optional_text(agent_state.title.as_deref()) {
        Some(title) => format!("Page title is {title}."),
        None => String::from("This page does not have a readable title yet."),
    };

    Some(PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::ReadTitle,
            goal: String::from("Read the current page title."),
            target_description: Some(String::from("current page title")),
        },
        selected_skills: selected_skill(active_skill_names, "read_title"),
        steps: vec![PlannedStep {
            step_id: String::from("report-page-title"),
            tool_name: ToolName::ReportResult,
            arguments: serde_json::json!({
                "request_id": request_id,
                "timeout_ms": serde_json::Value::Null,
                "status": ReportStatus::Success,
                "summary": summary.clone(),
                "next_recommended_action": serde_json::Value::Null,
                "user_message": summary
            }),
            purpose: String::from("Speak the current page title."),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    })
}

pub(crate) fn resolve_direct_repeat_command(
    transcript: &str,
    request_id: &str,
    agent_state: &AgentStateData,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() || !is_repeat_phrase(&normalized) {
        return None;
    }

    let selected_skills = selected_skill(active_skill_names, "repeat");
    let Some(region_id) = agent_state
        .narration_cursor
        .as_ref()
        .and_then(|cursor| cursor.current_region_id.as_deref())
    else {
        let summary = String::from("There is no current region to repeat yet.");
        return Some(PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::Repeat,
                goal: String::from("Repeat the current narration region."),
                target_description: Some(String::from("current narration region")),
            },
            selected_skills,
            steps: vec![PlannedStep {
                step_id: String::from("report-missing-repeat-region"),
                tool_name: ToolName::ReportResult,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "status": ReportStatus::NeedsFollowUp,
                    "summary": summary.clone(),
                    "next_recommended_action": "Read the page or move to a region first.",
                    "user_message": summary
                }),
                purpose: String::from(
                    "Report that no current narration region is available to repeat.",
                ),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        });
    };

    Some(PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::Repeat,
            goal: String::from("Repeat the current narration region."),
            target_description: Some(String::from("current narration region")),
        },
        selected_skills,
        steps: vec![PlannedStep {
            step_id: String::from("repeat-current-region"),
            tool_name: ToolName::ReadRegion,
            arguments: serde_json::json!({
                "request_id": request_id,
                "timeout_ms": serde_json::Value::Null,
                "region_id": region_id,
                "interruption_mode": "Interrupt"
            }),
            purpose: String::from(
                "Repeat the current narration region from the stored narration cursor.",
            ),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    })
}

#[derive(Debug, Clone, PartialEq)]
struct NormalizedAudioSetting {
    value: f32,
    goal: String,
    skill_name: &'static str,
}

fn parse_volume_command(normalized: &str, current_volume: f32) -> Option<NormalizedAudioSetting> {
    if normalized == "mute" || normalized.contains("mute volume") {
        return Some(NormalizedAudioSetting {
            value: 0.0,
            goal: String::from("Set playback volume to muted."),
            skill_name: "mute_volume",
        });
    }

    if let Some(step) = volume_relative_step(normalized) {
        let target = (current_volume + step).clamp(0.0, MAX_PLAYBACK_VOLUME);
        let goal = if step.is_sign_positive() {
            String::from("Increase playback volume by the requested normalized step.")
        } else {
            String::from("Decrease playback volume by the requested normalized step.")
        };
        let skill_name = if step.is_sign_positive() {
            "increase_volume"
        } else {
            "decrease_volume"
        };
        return Some(NormalizedAudioSetting {
            value: round_audio_setting_value(target),
            goal,
            skill_name,
        });
    }

    if !normalized.contains("volume") {
        return None;
    }

    parse_absolute_volume_value(normalized).map(|value| NormalizedAudioSetting {
        value: round_audio_setting_value(value.clamp(0.0, MAX_PLAYBACK_VOLUME)),
        goal: String::from("Set playback volume to the requested normalized value."),
        skill_name: "set_volume",
    })
}

fn parse_speed_command(normalized: &str, current_speed: f32) -> Option<NormalizedAudioSetting> {
    if let Some(step) = speed_relative_step(normalized) {
        let target = (current_speed + step).clamp(MIN_PLAYBACK_SPEED, MAX_PLAYBACK_SPEED);
        let goal = if step.is_sign_positive() {
            String::from("Increase playback speed by the requested normalized step.")
        } else {
            String::from("Decrease playback speed by the requested normalized step.")
        };
        let skill_name = if step.is_sign_positive() {
            "increase_playback_speed"
        } else {
            "decrease_playback_speed"
        };
        return Some(NormalizedAudioSetting {
            value: round_audio_setting_value(target),
            goal,
            skill_name,
        });
    }

    if !normalized.contains("speed") {
        return None;
    }

    parse_absolute_speed_value(normalized).map(|value| NormalizedAudioSetting {
        value: round_audio_setting_value(value.clamp(MIN_PLAYBACK_SPEED, MAX_PLAYBACK_SPEED)),
        goal: String::from("Set playback speed to the requested normalized value."),
        skill_name: "set_playback_speed",
    })
}

fn parse_absolute_volume_value(normalized: &str) -> Option<f32> {
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();

    for (index, token) in tokens.iter().enumerate() {
        let Ok(value) = token.parse::<f32>() else {
            continue;
        };

        if tokens.get(index + 1).copied() == Some("percent") {
            return Some(value / 100.0);
        }
        if value.fract() == 0.0 && (0.0..=100.0).contains(&value) {
            return Some(value / 100.0);
        }
        return Some(value);
    }

    None
}

fn parse_absolute_speed_value(normalized: &str) -> Option<f32> {
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();

    for (index, token) in tokens.iter().enumerate() {
        if let Some(multiplier) = parse_multiplier_token(token) {
            return Some(multiplier);
        }

        let Ok(value) = token.parse::<f32>() else {
            continue;
        };

        if matches!(tokens.get(index + 1).copied(), Some("times") | Some("time")) {
            return Some(value);
        }
        if tokens.get(index + 1).copied() == Some("percent") {
            return Some(value / 100.0);
        }
        return Some(value);
    }

    None
}

fn parse_multiplier_token(token: &str) -> Option<f32> {
    token
        .strip_suffix('x')
        .and_then(|value| (!value.is_empty()).then_some(value))
        .and_then(|value| value.parse::<f32>().ok())
}

fn volume_relative_step(normalized: &str) -> Option<f32> {
    if normalized.contains("increase volume")
        || normalized.contains("turn it up")
        || normalized.contains("volume up")
        || normalized.contains("louder")
    {
        return Some(volume_step_size(normalized));
    }

    if normalized.contains("decrease volume")
        || normalized.contains("turn it down")
        || normalized.contains("volume down")
        || normalized.contains("quieter")
    {
        return Some(-volume_step_size(normalized));
    }

    None
}

fn speed_relative_step(normalized: &str) -> Option<f32> {
    if normalized.contains("increase playback speed")
        || normalized.contains("speed up")
        || normalized.contains("go faster")
        || normalized == "faster"
    {
        return Some(speed_step_size(normalized));
    }

    if normalized.contains("decrease playback speed")
        || normalized.contains("slow down")
        || normalized.contains("go slower")
        || normalized == "slower"
    {
        return Some(-speed_step_size(normalized));
    }

    None
}

fn volume_step_size(normalized: &str) -> f32 {
    if normalized.contains("a little") || normalized.contains("slightly") {
        SMALL_VOLUME_STEP
    } else if normalized.contains("a lot") || normalized.contains("much") {
        LARGE_VOLUME_STEP
    } else {
        DEFAULT_VOLUME_STEP
    }
}

fn speed_step_size(normalized: &str) -> f32 {
    if normalized.contains("a little") || normalized.contains("slightly") {
        SMALL_SPEED_STEP
    } else if normalized.contains("a lot") || normalized.contains("much") {
        LARGE_SPEED_STEP
    } else {
        DEFAULT_SPEED_STEP
    }
}

fn is_volume_query_phrase(normalized: &str) -> bool {
    normalized.contains("what is the volume")
        || normalized.contains("what s the volume")
        || normalized.contains("current volume")
        || normalized.contains("tell me the volume")
}

fn is_speed_query_phrase(normalized: &str) -> bool {
    normalized.contains("what is the playback speed")
        || normalized.contains("what s the playback speed")
        || normalized.contains("current playback speed")
        || normalized.contains("what speed am i on")
        || normalized.contains("tell me the speed")
}

fn is_current_url_query_phrase(normalized: &str) -> bool {
    normalized.contains("current url")
        || normalized.contains("what page am i on")
        || normalized.contains("what page is this")
        || normalized.contains("what site am i on")
        || normalized.contains("where is this page")
}

fn is_go_back_phrase(normalized: &str) -> bool {
    normalized == "back" || normalized.contains("go back")
}

fn is_start_listening_phrase(normalized: &str) -> bool {
    normalized.contains("start listening")
        || normalized.contains("listen now")
        || normalized.contains("begin listening")
}

fn is_stop_listening_phrase(normalized: &str) -> bool {
    normalized.contains("stop listening")
        || normalized.contains("stop listen")
        || normalized.contains("quit listening")
}

fn is_transcribe_command_phrase(normalized: &str) -> bool {
    normalized.contains("transcribe")
        || normalized.contains("what did i say")
        || normalized.contains("what did i just say")
}

fn is_read_page_phrase(normalized: &str) -> bool {
    normalized == "read page"
        || normalized == "read this page"
        || normalized == "read current page"
        || normalized == "start reading page"
        || normalized == "start reading this page"
}

fn parse_direct_open_url_target(transcript: &str) -> Option<String> {
    let trimmed = transcript.trim();
    let lowercase = trimmed.to_ascii_lowercase();

    let raw_target = ["open ", "go to ", "visit "]
        .iter()
        .find_map(|prefix| {
            lowercase
                .strip_prefix(prefix)
                .map(|_| &trimmed[prefix.len()..])
        })?
        .trim()
        .trim_matches(|character: char| matches!(character, '.' | ',' | ';' | ':' | '"' | '\''));

    normalize_spoken_url_target(raw_target)
}

fn normalize_spoken_url_target(target: &str) -> Option<String> {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.eq_ignore_ascii_case("about blank") || trimmed.eq_ignore_ascii_case("about:blank") {
        return Some(String::from("about:blank"));
    }

    if trimmed.contains("://") || trimmed.to_ascii_lowercase().starts_with("about:") {
        return Some(trimmed.split_whitespace().collect());
    }

    if looks_like_host_without_scheme(trimmed) {
        return Some(prepend_default_scheme(trimmed));
    }

    let normalized = normalize_transcript_for_routing(trimmed);
    if normalized.is_empty() {
        return None;
    }

    let mut rebuilt = String::new();
    for token in normalized.split_whitespace() {
        match token {
            "dot" => rebuilt.push('.'),
            "slash" => rebuilt.push('/'),
            "colon" => rebuilt.push(':'),
            "dash" | "hyphen" => rebuilt.push('-'),
            "underscore" => rebuilt.push('_'),
            other => rebuilt.push_str(other),
        }
    }

    looks_like_host_without_scheme(&rebuilt).then(|| prepend_default_scheme(&rebuilt))
}

fn looks_like_host_without_scheme(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.contains(' ') {
        return false;
    }

    trimmed == "localhost" || trimmed.starts_with("localhost:") || trimmed.contains('.')
}

fn prepend_default_scheme(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed == "localhost"
        || trimmed.starts_with("localhost:")
        || trimmed.starts_with("127.0.0.1")
        || trimmed.starts_with("0.0.0.0")
    {
        format!("http://{trimmed}")
    } else {
        format!("https://{trimmed}")
    }
}

fn first_readable_region_id(page_model: &PageModel) -> Option<String> {
    page_model
        .regions
        .iter()
        .find(|region| !region.text.trim().is_empty())
        .map(|region| region.region_id.clone())
}

pub(crate) fn parse_direct_focus_field_command(transcript: &str) -> Option<FocusFieldCommand> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() || !is_focus_field_phrase(&normalized) {
        return None;
    }

    let mut remainder = normalized.as_str();
    if let Some(stripped) = remainder.strip_prefix("focus ") {
        remainder = stripped;
    }
    if let Some(stripped) = remainder.strip_prefix("on ") {
        remainder = stripped;
    }
    if let Some(stripped) = remainder.strip_prefix("the ") {
        remainder = stripped;
    }
    if let Some(stripped) = remainder.strip_prefix("my ") {
        remainder = stripped;
    }
    if let Some(stripped) = remainder.strip_prefix("field ") {
        remainder = stripped;
    }
    if let Some(stripped) = remainder.strip_suffix(" field") {
        remainder = stripped;
    }

    let description = remainder.trim();
    Some(FocusFieldCommand {
        description: (!description.is_empty() && description != "field")
            .then(|| description.to_string()),
    })
}

pub(crate) fn parse_direct_fill_field_command(transcript: &str) -> Option<FillFieldCommand> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty()
        || !is_fill_input_phrase(&normalized)
        || is_focus_field_phrase(&normalized)
        || is_fill_and_submit_phrase(&normalized)
    {
        return None;
    }

    let collapsed = collapse_transcript_whitespace(transcript);
    if collapsed.is_empty() {
        return Some(FillFieldCommand {
            description: None,
            text: None,
        });
    }

    if let Some((description, text)) = parse_fill_with_pattern(&collapsed) {
        return Some(FillFieldCommand {
            description,
            text: Some(text),
        });
    }

    if let Some((text, description)) = parse_into_field_pattern(&collapsed) {
        return Some(FillFieldCommand {
            description,
            text: Some(text),
        });
    }

    Some(FillFieldCommand {
        description: parse_fill_field_description_only(&collapsed),
        text: None,
    })
}

pub(crate) fn parse_fill_field_correction_command(
    transcript: &str,
) -> Option<FillFieldCorrectionCommand> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() || !is_fill_input_phrase(&normalized) {
        return None;
    }

    if normalized.contains("the other field") {
        return Some(FillFieldCorrectionCommand::AlternateField);
    }

    let collapsed = collapse_transcript_whitespace(transcript);
    if collapsed.is_empty() {
        return None;
    }

    let lowered = collapsed.to_ascii_lowercase();
    for prefix in ["put ", "type ", "enter "] {
        let suffix = " there instead";
        if lowered.starts_with(prefix) && lowered.ends_with(suffix) {
            let end_index = collapsed.len().saturating_sub(suffix.len());
            let value = collapsed.get(prefix.len()..end_index)?.trim();
            let text = normalize_fill_value(value)?;
            return Some(FillFieldCorrectionCommand::ReplaceValue { text });
        }
    }

    None
}

pub(crate) fn parse_direct_fill_and_submit_command(transcript: &str) -> Option<FillFieldCommand> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() || !is_fill_and_submit_phrase(&normalized) {
        return None;
    }

    let collapsed = collapse_transcript_whitespace(transcript);
    let fill_portion = strip_fill_and_submit_suffix(&collapsed)?;
    parse_direct_fill_field_command(&fill_portion)
}

pub(crate) fn is_direct_submit_form_command(transcript: &str) -> bool {
    let normalized = normalize_transcript_for_routing(transcript);
    !normalized.is_empty() && is_submit_form_phrase(&normalized)
}

fn is_go_forward_phrase(normalized: &str) -> bool {
    normalized == "forward" || normalized.contains("go forward")
}

fn is_reload_page_phrase(normalized: &str) -> bool {
    normalized == "reload"
        || normalized == "refresh"
        || normalized.contains("reload page")
        || normalized.contains("refresh page")
}

fn is_status_query_phrase(normalized: &str) -> bool {
    normalized.contains("what is the status")
        || normalized.contains("what s the status")
        || normalized.contains("current status")
        || normalized.contains("status please")
        || normalized.contains("where am i")
}

fn is_history_query_phrase(normalized: &str) -> bool {
    is_back_history_query_phrase(normalized) || is_forward_history_query_phrase(normalized)
}

fn is_back_history_query_phrase(normalized: &str) -> bool {
    normalized.contains("can i go back")
        || normalized.contains("can we go back")
        || normalized.contains("is back available")
        || normalized.contains("can go back")
        || normalized.contains("back available")
}

fn is_forward_history_query_phrase(normalized: &str) -> bool {
    normalized.contains("can i go forward")
        || normalized.contains("can we go forward")
        || normalized.contains("is forward available")
        || normalized.contains("can go forward")
        || normalized.contains("forward available")
}

fn is_listening_query_phrase(normalized: &str) -> bool {
    normalized.contains("are you listening")
        || normalized.contains("listening status")
        || normalized.contains("is listening on")
        || normalized.contains("am i listening")
}

fn is_speaking_query_phrase(normalized: &str) -> bool {
    normalized.contains("are you speaking")
        || normalized.contains("are you reading")
        || normalized.contains("is speech active")
        || normalized.contains("are you talking")
}

fn is_browser_mode_query_phrase(normalized: &str) -> bool {
    normalized.contains("browser mode")
        || normalized.contains("is the browser visible")
        || normalized.contains("is browser visible")
        || normalized.contains("is it headless")
        || normalized.contains("are we headless")
}

fn is_repeat_phrase(normalized: &str) -> bool {
    normalized == "repeat"
        || normalized.contains("repeat that")
        || normalized.contains("repeat this")
        || normalized.contains("repeat region")
        || normalized.contains("read that again")
        || normalized.contains("say that again")
        || normalized.contains("say this again")
}

fn is_read_next_phrase(normalized: &str) -> bool {
    normalized == "next"
        || normalized.contains("read next")
        || normalized.contains("next region")
        || normalized.contains("next section")
        || normalized.contains("continue reading")
        || normalized.contains("keep reading")
}

fn is_read_previous_phrase(normalized: &str) -> bool {
    normalized == "previous"
        || normalized.contains("read previous")
        || normalized.contains("previous region")
        || normalized.contains("previous section")
}

fn is_stop_phrase(normalized: &str) -> bool {
    normalized == "stop"
        || normalized.contains("stop reading")
        || normalized.contains("stop speaking")
        || normalized.contains("pause reading")
}

fn is_read_title_phrase(normalized: &str) -> bool {
    normalized == "read title"
        || normalized.contains("read the title")
        || normalized.contains("read page title")
        || normalized.contains("read the page title")
        || normalized.contains("what is the title")
        || normalized.contains("what s the title")
        || normalized.contains("tell me the title")
}

fn is_set_tts_voice_phrase(normalized: &str) -> bool {
    normalized.contains("voice")
        && (normalized.contains("set ")
            || normalized.contains("change ")
            || normalized.contains("switch ")
            || normalized.contains("use "))
}

fn is_fill_and_submit_phrase(normalized: &str) -> bool {
    (normalized.contains("fill ") || normalized.contains("enter ") || normalized.contains("type "))
        && normalized.contains("submit")
}

fn is_focus_field_phrase(normalized: &str) -> bool {
    normalized.contains("focus field")
        || (normalized.contains("focus ") && normalized.contains(" field"))
}

fn is_submit_form_phrase(normalized: &str) -> bool {
    normalized == "submit"
        || normalized.contains("submit form")
        || normalized.contains("submit this form")
        || normalized.contains("send form")
        || normalized.contains("send this form")
        || normalized.contains("press submit")
        || normalized.contains("hit submit")
}

fn is_fill_input_phrase(normalized: &str) -> bool {
    is_focus_field_phrase(normalized)
        || normalized.contains("fill in ")
        || (normalized.contains("fill ") && normalized.contains(" field"))
        || normalized.contains("type into ")
        || (normalized.contains("type ")
            && normalized.contains(" into ")
            && normalized.contains(" field"))
        || (normalized.contains("enter ") && normalized.contains(" field"))
        || (normalized.contains("put ") && normalized.contains(" field"))
        || (normalized.contains("choose ") && normalized.contains(" list"))
        || (normalized.contains("select ") && normalized.contains(" field"))
        || normalized.contains("the other field")
        || normalized.contains("there instead")
}

fn collapse_transcript_whitespace(transcript: &str) -> String {
    transcript.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn parse_fill_with_pattern(transcript: &str) -> Option<(Option<String>, String)> {
    let lowered = transcript.to_ascii_lowercase();
    let prefix = if lowered.starts_with("fill in ") {
        "fill in "
    } else if lowered.starts_with("fill ") {
        "fill "
    } else {
        return None;
    };

    let remainder = transcript.get(prefix.len()..)?.trim();
    let (field_target, text) = split_case_insensitive_once(remainder, " with ")?;
    let text = normalize_fill_value(text)?;
    Some((normalize_field_target(field_target), text))
}

fn parse_into_field_pattern(transcript: &str) -> Option<(String, Option<String>)> {
    let lowered = transcript.to_ascii_lowercase();
    let (prefix, separator) = if lowered.starts_with("type ") && lowered.contains(" into ") {
        ("type ", " into ")
    } else if lowered.starts_with("enter ") && lowered.contains(" into ") {
        ("enter ", " into ")
    } else if lowered.starts_with("enter ") && lowered.contains(" in ") {
        ("enter ", " in ")
    } else if lowered.starts_with("put ") && lowered.contains(" in ") {
        ("put ", " in ")
    } else {
        return None;
    };

    let remainder = transcript.get(prefix.len()..)?.trim();
    let (text, field_target) = split_case_insensitive_once(remainder, separator)?;
    let text = normalize_fill_value(text)?;
    Some((text, normalize_field_target(field_target)))
}

fn parse_fill_field_description_only(transcript: &str) -> Option<String> {
    let lowered = transcript.to_ascii_lowercase();
    let remainder = if lowered.starts_with("fill in ") {
        transcript.get("fill in ".len()..)?
    } else if lowered.starts_with("fill ") {
        transcript.get("fill ".len()..)?
    } else if lowered.starts_with("type into ") {
        transcript.get("type into ".len()..)?
    } else if lowered.starts_with("enter into ") {
        transcript.get("enter into ".len()..)?
    } else if lowered.starts_with("enter in ") {
        transcript.get("enter in ".len()..)?
    } else if lowered.starts_with("put in ") {
        transcript.get("put in ".len()..)?
    } else {
        return None;
    };

    normalize_field_target(remainder)
}

fn strip_fill_and_submit_suffix(transcript: &str) -> Option<String> {
    let normalized = collapse_transcript_whitespace(transcript);
    if normalized.is_empty() {
        return None;
    }

    let lowered = normalized.to_ascii_lowercase();
    for suffix in [
        " and then submit this form",
        " and then submit form",
        " and then send this form",
        " and then send form",
        " and then press submit",
        " and then hit submit",
        " and then submit",
        " then submit this form",
        " then submit form",
        " then send this form",
        " then send form",
        " then press submit",
        " then hit submit",
        " then submit",
        " and submit this form",
        " and submit form",
        " and send this form",
        " and send form",
        " and press submit",
        " and hit submit",
        " and submit",
    ] {
        if lowered.ends_with(suffix) {
            let trimmed = normalized
                .get(..normalized.len().saturating_sub(suffix.len()))?
                .trim();
            return (!trimmed.is_empty()).then(|| trimmed.to_string());
        }
    }

    None
}

fn split_case_insensitive_once<'a>(text: &'a str, separator: &str) -> Option<(&'a str, &'a str)> {
    let lowered = text.to_ascii_lowercase();
    let index = lowered.find(separator)?;
    let before = text.get(..index)?.trim();
    let after = text.get(index + separator.len()..)?.trim();
    Some((before, after))
}

fn normalize_field_target(target: &str) -> Option<String> {
    let mut normalized = collapse_transcript_whitespace(target);
    if normalized.is_empty() {
        return None;
    }

    let lowered = normalized.to_ascii_lowercase();
    for prefix in ["the ", "my ", "a ", "an "] {
        if lowered.starts_with(prefix) {
            normalized = normalized.get(prefix.len()..)?.trim().to_string();
            break;
        }
    }

    for suffix in [" field", " textbox", " text box", " input", " input box"] {
        if normalized.to_ascii_lowercase().ends_with(suffix) {
            let end = normalized.len().saturating_sub(suffix.len());
            normalized = normalized.get(..end)?.trim().to_string();
            break;
        }
    }

    let normalized = normalized.trim();
    (!normalized.is_empty()).then(|| normalized.to_string())
}

fn normalize_fill_value(value: &str) -> Option<String> {
    let collapsed = collapse_transcript_whitespace(value);
    let trimmed = collapsed.trim();
    if trimmed.is_empty() {
        return None;
    }

    let unquoted = if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        trimmed
            .get(1..trimmed.len().saturating_sub(1))
            .unwrap_or(trimmed)
    } else {
        trimmed
    };

    let cleaned = unquoted.trim();
    (!cleaned.is_empty()).then(|| cleaned.to_string())
}

fn is_browser_visibility_phrase(normalized: &str) -> bool {
    normalized.contains("show browser")
        || normalized.contains("hide browser")
        || normalized.contains("show the browser")
        || normalized.contains("hide the browser")
        || normalized.contains("make browser visible")
        || normalized.contains("make the browser visible")
        || normalized.contains("make it visible")
        || normalized.contains("switch to visible")
        || normalized.contains("switch browser to visible")
        || normalized.contains("visible mode")
        || normalized.contains("show the window")
        || normalized.contains("go headless")
        || normalized.contains("make browser headless")
        || normalized.contains("make the browser headless")
        || normalized.contains("make it headless")
        || normalized.contains("switch to headless")
        || normalized.contains("switch browser to headless")
        || normalized.contains("headless mode")
}

fn selected_skill(active_skill_names: &[String], skill_name: &'static str) -> Vec<String> {
    if active_skill_names
        .iter()
        .any(|active_name| active_name == skill_name)
    {
        vec![String::from(skill_name)]
    } else {
        Vec::new()
    }
}

fn selected_audio_skill(active_skill_names: &[String], skill_name: &'static str) -> Vec<String> {
    selected_skill(active_skill_names, skill_name)
}

fn selected_stop_skill(active_skill_names: &[String]) -> Vec<String> {
    if active_skill_names
        .iter()
        .any(|active_name| active_name == "stop_reading")
    {
        vec![String::from("stop_reading")]
    } else if active_skill_names
        .iter()
        .any(|active_name| active_name == "pause_reading")
    {
        vec![String::from("pause_reading")]
    } else {
        Vec::new()
    }
}

fn build_single_step_planner_output(
    intent: IntentSummary,
    selected_skills: Vec<String>,
    step: PlannedStep,
) -> PlannerOutput {
    PlannerOutput {
        status: PlannerStatus::Ready,
        intent,
        selected_skills,
        steps: vec![step],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    }
}

fn build_audio_set_planner_output(spec: AudioSetPlanSpec<'_>) -> PlannerOutput {
    PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: spec.intent_name,
            goal: spec.goal,
            target_description: spec.target_description,
        },
        selected_skills: spec.selected_skills,
        steps: vec![
            PlannedStep {
                step_id: String::from(spec.set_step_id),
                tool_name: spec.tool_name,
                arguments: spec.tool_arguments,
                purpose: spec.tool_purpose,
                on_success: StepTransition::NextStep {
                    step_id: String::from(spec.report_step_id),
                },
                on_failure: StepTransition::Replan,
            },
            build_report_result_step(spec.request_id, spec.report_step_id, spec.report_summary),
        ],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    }
}

fn build_audio_report_planner_output(
    request_id: &str,
    intent_name: IntentName,
    goal: String,
    selected_skills: Vec<String>,
    target_description: Option<String>,
    report_summary: String,
) -> PlannerOutput {
    PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: intent_name,
            goal,
            target_description,
        },
        selected_skills,
        steps: vec![build_report_result_step(
            request_id,
            "report-audio-setting",
            report_summary,
        )],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    }
}

fn build_browser_visibility_planner_output(
    request_id: &str,
    target_mode: BrowserVisibilityMode,
    selected_skills: Vec<String>,
    report_summary: String,
) -> PlannerOutput {
    PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::SetBrowserVisibility,
            goal: String::from("Set the browser visibility mode to the requested target."),
            target_description: Some(format_browser_visibility_mode(target_mode)),
        },
        selected_skills,
        steps: vec![
            PlannedStep {
                step_id: String::from("set-browser-visibility"),
                tool_name: ToolName::SetBrowserVisibility,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "mode": target_mode
                }),
                purpose: String::from("Apply the requested browser visibility mode."),
                on_success: StepTransition::NextStep {
                    step_id: String::from("report-browser-visibility"),
                },
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from("report-browser-visibility"),
                tool_name: ToolName::ReportResult,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "status": ReportStatus::Success,
                    "summary": report_summary.clone(),
                    "next_recommended_action": serde_json::Value::Null,
                    "user_message": report_summary
                }),
                purpose: String::from("Report the resulting browser visibility mode."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    }
}

fn build_status_query_planner_output(spec: StatusQueryPlanSpec<'_>) -> PlannerOutput {
    PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: spec.intent_name,
            goal: spec.goal,
            target_description: spec.target_description,
        },
        selected_skills: spec.selected_skills,
        steps: vec![
            PlannedStep {
                step_id: String::from(spec.read_step_id),
                tool_name: spec.read_tool_name,
                arguments: spec.read_tool_arguments,
                purpose: spec.read_tool_purpose,
                on_success: StepTransition::NextStep {
                    step_id: String::from(spec.report_step_id),
                },
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from(spec.report_step_id),
                tool_name: ToolName::ReportResult,
                arguments: serde_json::json!({
                    "request_id": spec.request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "status": ReportStatus::Success,
                    "summary": spec.report_summary.clone(),
                    "next_recommended_action": serde_json::Value::Null,
                    "user_message": spec.report_summary
                }),
                purpose: String::from("Report the resulting status query answer."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    }
}

fn build_report_result_step(request_id: &str, step_id: &str, summary: String) -> PlannedStep {
    PlannedStep {
        step_id: String::from(step_id),
        tool_name: ToolName::ReportResult,
        arguments: serde_json::json!({
            "request_id": request_id,
            "timeout_ms": serde_json::Value::Null,
            "status": ReportStatus::Success,
            "summary": summary.clone(),
            "next_recommended_action": serde_json::Value::Null,
            "user_message": summary
        }),
        purpose: String::from("Report the resulting playback setting."),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    }
}

fn format_playback_volume(volume: f32) -> String {
    format!("{}%", (volume * 100.0).round() as i32)
}

fn format_playback_speed(speed: f32) -> String {
    let formatted = format!("{speed:.2}");
    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
    format!("{trimmed}x")
}

fn format_browser_visibility_mode(mode: BrowserVisibilityMode) -> String {
    match mode {
        BrowserVisibilityMode::Visible => String::from("visible"),
        BrowserVisibilityMode::Headless => String::from("headless"),
    }
}

fn format_current_url_summary(agent_state: &AgentStateData) -> String {
    match (
        normalized_optional_text(agent_state.title.as_deref()),
        agent_state.url.as_deref(),
    ) {
        (Some(title), Some(url)) => format!("Current page is {title} at {url}."),
        (None, Some(url)) => format!("Current page URL is {url}."),
        (Some(title), None) => format!("Current page is {title}."),
        (None, None) => String::from("No page is open yet."),
    }
}

fn current_page_label(agent_state: &AgentStateData) -> String {
    normalized_optional_text(agent_state.title.as_deref())
        .or_else(|| normalized_optional_text(agent_state.url.as_deref()))
        .unwrap_or_else(|| String::from("no page open"))
}

fn format_runtime_status_summary(runtime_status: &GetRuntimeStatusData) -> String {
    let page_summary = current_page_label_from_runtime_status(runtime_status);
    let browser_mode = format_browser_visibility_mode(runtime_status.browser_visibility);
    let listening = if runtime_status.listening_state.is_listening {
        "on"
    } else {
        "off"
    };
    let speaking = if runtime_status.speaking {
        "active"
    } else {
        "idle"
    };
    let back = if runtime_status.browser_history.can_go_back {
        "available"
    } else {
        "unavailable"
    };
    let forward = if runtime_status.browser_history.can_go_forward {
        "available"
    } else {
        "unavailable"
    };

    format!(
        "Current page is {page_summary}. Browser mode is {browser_mode}. Listening is {listening}. Speech output is {speaking}. Back is {back}. Forward is {forward}."
    )
}

fn current_page_label_from_runtime_status(runtime_status: &GetRuntimeStatusData) -> String {
    normalized_optional_text(runtime_status.title.as_deref())
        .or_else(|| normalized_optional_text(runtime_status.url.as_deref()))
        .unwrap_or_else(|| String::from("no page open"))
}

fn format_back_history_summary(runtime_status: &GetRuntimeStatusData) -> String {
    if runtime_status.browser_history.can_go_back {
        String::from("Back navigation is available.")
    } else {
        String::from("Back navigation is not available.")
    }
}

fn format_forward_history_summary(runtime_status: &GetRuntimeStatusData) -> String {
    if runtime_status.browser_history.can_go_forward {
        String::from("Forward navigation is available.")
    } else {
        String::from("Forward navigation is not available.")
    }
}

fn format_listening_summary(runtime_status: &GetRuntimeStatusData) -> String {
    if runtime_status.listening_state.is_listening {
        String::from("Listening is on.")
    } else {
        String::from("Listening is off.")
    }
}

fn format_speaking_summary(runtime_status: &GetRuntimeStatusData) -> String {
    if runtime_status.speaking {
        String::from("Speech output is active.")
    } else {
        String::from("Speech output is idle.")
    }
}

fn format_browser_mode_summary(runtime_status: &GetRuntimeStatusData) -> String {
    format!(
        "Browser mode is {}.",
        format_browser_visibility_mode(runtime_status.browser_visibility)
    )
}

fn parse_browser_visibility_command(
    normalized: &str,
    current_visibility: BrowserVisibilityMode,
) -> Option<BrowserVisibilityMode> {
    if normalized.contains("hide browser")
        || normalized.contains("hide the browser")
        || normalized.contains("go headless")
        || normalized.contains("make browser headless")
        || normalized.contains("make the browser headless")
        || normalized.contains("make it headless")
        || normalized.contains("switch to headless")
        || normalized.contains("switch browser to headless")
        || normalized.contains("headless mode")
    {
        return Some(BrowserVisibilityMode::Headless);
    }

    if normalized.contains("show browser")
        || normalized.contains("show the browser")
        || normalized.contains("make browser visible")
        || normalized.contains("make the browser visible")
        || normalized.contains("make it visible")
        || normalized.contains("switch to visible")
        || normalized.contains("switch browser to visible")
        || normalized.contains("visible mode")
        || normalized.contains("show the window")
    {
        return Some(BrowserVisibilityMode::Visible);
    }

    if normalized.contains("toggle browser visibility") || normalized.contains("toggle visibility")
    {
        return Some(match current_visibility {
            BrowserVisibilityMode::Visible => BrowserVisibilityMode::Headless,
            BrowserVisibilityMode::Headless => BrowserVisibilityMode::Visible,
        });
    }

    None
}

fn normalized_optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(String::from)
}

fn selected_status_skill(active_skill_names: &[String]) -> Vec<String> {
    if active_skill_names
        .iter()
        .any(|active_name| active_name == "get_status")
    {
        vec![String::from("get_status")]
    } else if active_skill_names
        .iter()
        .any(|active_name| active_name == "announce_state")
    {
        vec![String::from("announce_state")]
    } else {
        Vec::new()
    }
}

struct StatusQueryPlanSpec<'a> {
    request_id: &'a str,
    intent_name: IntentName,
    goal: String,
    selected_skills: Vec<String>,
    target_description: Option<String>,
    read_step_id: &'a str,
    read_tool_name: ToolName,
    read_tool_arguments: serde_json::Value,
    read_tool_purpose: String,
    report_step_id: &'a str,
    report_summary: String,
}

fn round_audio_setting_value(value: f32) -> f32 {
    (value * 100.0).round() / 100.0
}

struct AudioSetPlanSpec<'a> {
    request_id: &'a str,
    intent_name: IntentName,
    goal: String,
    selected_skills: Vec<String>,
    target_description: Option<String>,
    set_step_id: &'a str,
    tool_name: ToolName,
    tool_arguments: serde_json::Value,
    tool_purpose: String,
    report_step_id: &'a str,
    report_summary: String,
}
