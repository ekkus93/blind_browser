use super::*;

pub(super) fn parse_direct_open_url_target(transcript: &str) -> Option<String> {
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

pub(super) fn normalize_spoken_url_target(target: &str) -> Option<String> {
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

pub(super) fn looks_like_host_without_scheme(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.contains(' ') {
        return false;
    }

    trimmed == "localhost" || trimmed.starts_with("localhost:") || trimmed.contains('.')
}

pub(super) fn prepend_default_scheme(value: &str) -> String {
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

pub(super) fn first_readable_region_id(page_model: &PageModel) -> Option<String> {
    page_model
        .regions
        .iter()
        .find(|region| !region.text.trim().is_empty())
        .map(|region| region.region_id.clone())
}

pub(super) fn format_current_url_summary(agent_state: &AgentStateData) -> String {
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
