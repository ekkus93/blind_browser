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

pub(super) fn normalize_audio_command_text(transcript: &str) -> String {
    normalize_command_text(transcript, true)
}

pub(super) fn normalize_command_text(transcript: &str, allow_decimal: bool) -> String {
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

pub(super) fn merge_compound_command_tokens(tokens: Vec<String>) -> Vec<String> {
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

pub(super) fn canonicalize_command_token(token: &str) -> String {
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

pub(super) fn is_unambiguous_fuzzy_keyword_match(token: &str, keyword: &str) -> bool {
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

pub(super) fn is_single_edit_or_transposition(left: &str, right: &str) -> bool {
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

pub(super) fn is_single_insertion_or_deletion(shorter: &[char], longer: &[char]) -> bool {
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
pub(super) fn is_volume_query_phrase(normalized: &str) -> bool {
    normalized.contains("what is the volume")
        || normalized.contains("what s the volume")
        || normalized.contains("current volume")
        || normalized.contains("tell me the volume")
}

pub(super) fn is_speed_query_phrase(normalized: &str) -> bool {
    normalized.contains("what is the playback speed")
        || normalized.contains("what s the playback speed")
        || normalized.contains("current playback speed")
        || normalized.contains("what speed am i on")
        || normalized.contains("tell me the speed")
}

pub(super) fn is_current_url_query_phrase(normalized: &str) -> bool {
    normalized.contains("current url")
        || normalized.contains("what page am i on")
        || normalized.contains("what page is this")
        || normalized.contains("what site am i on")
        || normalized.contains("where is this page")
}

pub(super) fn is_go_back_phrase(normalized: &str) -> bool {
    normalized == "back" || normalized.contains("go back")
}

pub(super) fn is_start_listening_phrase(normalized: &str) -> bool {
    normalized.contains("start listening")
        || normalized.contains("listen now")
        || normalized.contains("begin listening")
}

pub(super) fn is_stop_listening_phrase(normalized: &str) -> bool {
    normalized.contains("stop listening")
        || normalized.contains("stop listen")
        || normalized.contains("quit listening")
}

pub(super) fn is_transcribe_command_phrase(normalized: &str) -> bool {
    normalized.contains("transcribe")
        || normalized.contains("what did i say")
        || normalized.contains("what did i just say")
}

pub(super) fn is_read_page_phrase(normalized: &str) -> bool {
    normalized == "read page"
        || normalized == "read this page"
        || normalized == "read current page"
        || normalized == "start reading page"
        || normalized == "start reading this page"
}

pub(super) fn is_go_forward_phrase(normalized: &str) -> bool {
    normalized == "forward" || normalized.contains("go forward")
}

pub(super) fn is_reload_page_phrase(normalized: &str) -> bool {
    normalized == "reload"
        || normalized == "refresh"
        || normalized.contains("reload page")
        || normalized.contains("refresh page")
}

pub(super) fn is_status_query_phrase(normalized: &str) -> bool {
    normalized.contains("what is the status")
        || normalized.contains("what s the status")
        || normalized.contains("current status")
        || normalized.contains("status please")
        || normalized.contains("where am i")
}

pub(super) fn is_history_query_phrase(normalized: &str) -> bool {
    is_back_history_query_phrase(normalized) || is_forward_history_query_phrase(normalized)
}

pub(super) fn is_back_history_query_phrase(normalized: &str) -> bool {
    normalized.contains("can i go back")
        || normalized.contains("can we go back")
        || normalized.contains("is back available")
        || normalized.contains("can go back")
        || normalized.contains("back available")
}

pub(super) fn is_forward_history_query_phrase(normalized: &str) -> bool {
    normalized.contains("can i go forward")
        || normalized.contains("can we go forward")
        || normalized.contains("is forward available")
        || normalized.contains("can go forward")
        || normalized.contains("forward available")
}

pub(super) fn is_listening_query_phrase(normalized: &str) -> bool {
    normalized.contains("are you listening")
        || normalized.contains("listening status")
        || normalized.contains("is listening on")
        || normalized.contains("am i listening")
}

pub(super) fn is_speaking_query_phrase(normalized: &str) -> bool {
    normalized.contains("are you speaking")
        || normalized.contains("are you reading")
        || normalized.contains("is speech active")
        || normalized.contains("are you talking")
}

pub(super) fn is_browser_mode_query_phrase(normalized: &str) -> bool {
    normalized.contains("browser mode")
        || normalized.contains("is the browser visible")
        || normalized.contains("is browser visible")
        || normalized.contains("is it headless")
        || normalized.contains("are we headless")
}

pub(super) fn is_repeat_phrase(normalized: &str) -> bool {
    normalized == "repeat"
        || normalized.contains("repeat that")
        || normalized.contains("repeat this")
        || normalized.contains("repeat region")
        || normalized.contains("read that again")
        || normalized.contains("say that again")
        || normalized.contains("say this again")
}

pub(super) fn is_read_next_phrase(normalized: &str) -> bool {
    normalized == "next"
        || normalized.contains("read next")
        || normalized.contains("next region")
        || normalized.contains("next section")
        || normalized.contains("continue reading")
        || normalized.contains("keep reading")
}

pub(super) fn is_read_previous_phrase(normalized: &str) -> bool {
    normalized == "previous"
        || normalized.contains("read previous")
        || normalized.contains("previous region")
        || normalized.contains("previous section")
}

pub(super) fn is_stop_phrase(normalized: &str) -> bool {
    normalized == "stop"
        || normalized.contains("stop reading")
        || normalized.contains("stop speaking")
        || normalized.contains("pause reading")
}

pub(super) fn is_read_title_phrase(normalized: &str) -> bool {
    normalized == "read title"
        || normalized.contains("read the title")
        || normalized.contains("read page title")
        || normalized.contains("read the page title")
        || normalized.contains("what is the title")
        || normalized.contains("what s the title")
        || normalized.contains("tell me the title")
}

pub(super) fn is_set_tts_voice_phrase(normalized: &str) -> bool {
    normalized.contains("voice")
        && (normalized.contains("set ")
            || normalized.contains("change ")
            || normalized.contains("switch ")
            || normalized.contains("use "))
}

pub(super) fn is_fill_and_submit_phrase(normalized: &str) -> bool {
    (normalized.contains("fill ") || normalized.contains("enter ") || normalized.contains("type "))
        && normalized.contains("submit")
}

pub(super) fn is_focus_field_phrase(normalized: &str) -> bool {
    normalized.contains("focus field")
        || (normalized.contains("focus ") && normalized.contains(" field"))
}

pub(super) fn is_submit_form_phrase(normalized: &str) -> bool {
    normalized == "submit"
        || normalized.contains("submit form")
        || normalized.contains("submit this form")
        || normalized.contains("send form")
        || normalized.contains("send this form")
        || normalized.contains("press submit")
        || normalized.contains("hit submit")
}

pub(super) fn is_fill_input_phrase(normalized: &str) -> bool {
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

pub(super) fn collapse_transcript_whitespace(transcript: &str) -> String {
    transcript.split_whitespace().collect::<Vec<_>>().join(" ")
}
pub(super) fn is_browser_visibility_phrase(normalized: &str) -> bool {
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

pub(super) fn selected_skill(active_skill_names: &[String], skill_name: &'static str) -> Vec<String> {
    if active_skill_names
        .iter()
        .any(|active_name| active_name == skill_name)
    {
        vec![String::from(skill_name)]
    } else {
        Vec::new()
    }
}

pub(super) fn selected_audio_skill(active_skill_names: &[String], skill_name: &'static str) -> Vec<String> {
    selected_skill(active_skill_names, skill_name)
}

pub(super) fn selected_stop_skill(active_skill_names: &[String]) -> Vec<String> {
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
