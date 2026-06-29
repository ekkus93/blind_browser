use std::collections::HashMap;

use super::*;

pub(crate) fn parse_skill_document(
    content: &str,
    source: SkillSource,
    available_tool_names: &[ToolName],
) -> Result<LoadedSkill, String> {
    let normalized = content.replace("\r\n", "\n");
    let Some(frontmatter_body) = normalized.strip_prefix("---\n") else {
        return Err(String::from("SKILL.md is missing a YAML frontmatter block"));
    };
    let Some(split_index) = frontmatter_body.find("\n---\n") else {
        return Err(String::from("SKILL.md frontmatter block is not terminated"));
    };

    let frontmatter_block = &frontmatter_body[..split_index];
    let body = frontmatter_body[(split_index + 5)..].trim().to_string();
    let frontmatter = parse_skill_frontmatter(frontmatter_block, available_tool_names)?;
    Ok(LoadedSkill {
        summary: skill_summary_from_frontmatter(frontmatter),
        body,
        source,
    })
}

fn parse_skill_frontmatter(
    block: &str,
    available_tool_names: &[ToolName],
) -> Result<SkillFrontmatter, String> {
    let mut scalar_fields = HashMap::<String, String>::new();
    let mut list_fields = HashMap::<String, Vec<String>>::new();
    let mut active_list_key: Option<String> = None;

    for raw_line in block.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(list_key) = active_list_key.as_ref() {
            if let Some(item) = trimmed.strip_prefix("- ") {
                list_fields
                    .entry(list_key.clone())
                    .or_default()
                    .push(clean_skill_value(item));
                continue;
            }
            active_list_key = None;
        }

        let Some((key, value)) = trimmed.split_once(':') else {
            return Err(format!("invalid frontmatter line '{trimmed}'"));
        };
        let key = normalize_skill_key(key.trim());
        let value = value.trim();

        match key.as_str() {
            "name" | "description" | "requires_confirmation" | "priority" => {
                scalar_fields.insert(key, clean_skill_value(value));
            }
            "allowed_tools" | "intent_tags" => {
                let list = list_fields.entry(key.clone()).or_default();
                list.extend(parse_inline_list(value));
                active_list_key = Some(key);
            }
            _ => return Err(format!("unsupported frontmatter field '{key}'")),
        }
    }

    skill_frontmatter_from_parts(scalar_fields, list_fields, available_tool_names)
}

/// Parse the bundled skills markdown, failing loudly on any defect.
///
/// Confirmation metadata is a safety contract: a malformed `requires_confirmation`
/// value (or an invalid frontmatter / unknown tool name) is a build defect, not a
/// best-effort parse condition, so it returns `Err` instead of defaulting to `false`
/// or skipping the skill with a warning. Bundled skills come from a compile-time
/// `include_str!`, so the caller treats any `Err` as fatal.
pub(crate) fn parse_bundled_skills(
    markdown: &str,
    available_tool_names: &[ToolName],
) -> Result<Vec<LoadedSkill>, String> {
    let mut current_name: Option<String> = None;
    let mut description = String::new();
    let mut intent_tags = Vec::new();
    let mut allowed_tools = Vec::new();
    let mut skills = Vec::new();

    let flush_skill = |skills: &mut Vec<LoadedSkill>,
                       current_name: &mut Option<String>,
                       description: &mut String,
                       intent_tags: &mut Vec<String>,
                       allowed_tools: &mut Vec<String>,
                       requires_confirmation: bool|
     -> Result<(), String> {
        let Some(name) = current_name.take() else {
            return Ok(());
        };

        let mut scalar_fields = HashMap::new();
        scalar_fields.insert(String::from("name"), name.clone());
        scalar_fields.insert(String::from("description"), description.trim().to_string());
        scalar_fields.insert(
            String::from("requires_confirmation"),
            requires_confirmation.to_string(),
        );
        let mut list_fields = HashMap::new();
        list_fields.insert(String::from("intent_tags"), intent_tags.clone());
        list_fields.insert(String::from("allowed_tools"), allowed_tools.clone());

        let frontmatter =
            skill_frontmatter_from_parts(scalar_fields, list_fields, available_tool_names)
                .map_err(|error| format!("invalid bundled skill {name:?}: {error}"))?;
        skills.push(LoadedSkill {
            summary: skill_summary_from_frontmatter(frontmatter),
            body: description.trim().to_string(),
            source: SkillSource::Bundled,
        });

        description.clear();
        intent_tags.clear();
        allowed_tools.clear();
        Ok(())
    };

    let mut requires_confirmation_value = false;
    for raw_line in markdown.lines() {
        let trimmed = raw_line.trim();
        if let Some(name) = trimmed.strip_prefix("#### ") {
            flush_skill(
                &mut skills,
                &mut current_name,
                &mut description,
                &mut intent_tags,
                &mut allowed_tools,
                requires_confirmation_value,
            )?;
            current_name = Some(name.trim().to_string());
            requires_confirmation_value = false;
            continue;
        }

        if current_name.is_none() {
            continue;
        }

        if let Some(value) = trimmed.strip_prefix("- intent_tags:") {
            intent_tags = parse_backticked_list(value);
        } else if let Some(value) = trimmed.strip_prefix("- allowed_tools:") {
            allowed_tools = parse_backticked_list(value);
        } else if let Some(value) = trimmed.strip_prefix("- requires_confirmation:") {
            requires_confirmation_value = parse_bool_value(value).map_err(|error| {
                format!(
                    "invalid requires_confirmation value for bundled skill {}: {error}",
                    current_name.as_deref().unwrap_or("<unknown>")
                )
            })?;
        } else if let Some(value) = trimmed.strip_prefix("- description:") {
            description = clean_skill_value(value);
        }
    }

    flush_skill(
        &mut skills,
        &mut current_name,
        &mut description,
        &mut intent_tags,
        &mut allowed_tools,
        requires_confirmation_value,
    )?;

    Ok(skills)
}

fn skill_frontmatter_from_parts(
    scalar_fields: HashMap<String, String>,
    list_fields: HashMap<String, Vec<String>>,
    available_tool_names: &[ToolName],
) -> Result<SkillFrontmatter, String> {
    let name = scalar_fields
        .get("name")
        .ok_or_else(|| String::from("skill frontmatter is missing name"))?
        .trim()
        .to_string();
    if !is_valid_skill_name(&name) {
        return Err(format!("invalid skill name '{name}'"));
    }

    let description = scalar_fields
        .get("description")
        .ok_or_else(|| String::from("skill frontmatter is missing description"))?
        .trim()
        .to_string();
    if description.is_empty() {
        return Err(String::from("skill description must not be empty"));
    }

    let mut intent_tags = list_fields
        .get("intent_tags")
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();
    intent_tags.sort();
    intent_tags.dedup();
    for tag in &intent_tags {
        if let Some(intent_name) = tag.strip_prefix("intent:") {
            parse_intent_name_value(intent_name)?;
        }
    }

    let allowed_tools = match list_fields.get("allowed_tools") {
        Some(tool_names) if !tool_names.is_empty() => {
            let mut resolved_tools = Vec::new();
            for tool_name in tool_names {
                let tool = parse_tool_name_value(tool_name)?;
                if !available_tool_names
                    .iter()
                    .any(|available| available == &tool)
                {
                    return Err(format!("skill references unavailable tool '{tool_name}'"));
                }
                resolved_tools.push(tool);
            }
            Some(resolved_tools)
        }
        _ => None,
    };

    let requires_confirmation = scalar_fields
        .get("requires_confirmation")
        .map(|value| parse_bool_value(value))
        .transpose()?
        .unwrap_or(false);

    let priority = scalar_fields
        .get("priority")
        .map(|value| {
            value
                .parse::<i32>()
                .map_err(|error| format!("invalid priority value '{value}': {error}"))
        })
        .transpose()?
        .unwrap_or(0);

    Ok(SkillFrontmatter {
        name,
        description,
        allowed_tools,
        intent_tags,
        requires_confirmation,
        priority,
    })
}

fn skill_summary_from_frontmatter(frontmatter: SkillFrontmatter) -> SkillSummary {
    SkillSummary {
        name: frontmatter.name,
        description: frontmatter.description,
        intent_tags: frontmatter.intent_tags,
        allowed_tools: frontmatter.allowed_tools,
        requires_confirmation: frontmatter.requires_confirmation,
        priority: frontmatter.priority,
    }
}

fn normalize_skill_key(key: &str) -> String {
    key.trim().replace('-', "_")
}

fn parse_inline_list(value: &str) -> Vec<String> {
    let cleaned = clean_skill_value(value);
    if cleaned.is_empty() {
        return Vec::new();
    }

    let trimmed = cleaned.trim_matches(['[', ']']);
    trimmed
        .split(',')
        .map(clean_skill_value)
        .filter(|item| !item.is_empty())
        .collect()
}

fn parse_backticked_list(value: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current = String::new();
    let mut in_tick = false;
    for character in value.chars() {
        match character {
            '`' => {
                if in_tick {
                    items.push(current.trim().to_string());
                    current.clear();
                }
                in_tick = !in_tick;
            }
            _ if in_tick => current.push(character),
            _ => {}
        }
    }

    if items.is_empty() {
        parse_inline_list(value)
    } else {
        items
    }
}

fn parse_bool_value(value: &str) -> Result<bool, String> {
    match clean_skill_value(value).as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(format!("invalid boolean value '{other}'")),
    }
}

fn clean_skill_value(value: &str) -> String {
    value
        .trim()
        .trim_matches('`')
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

fn is_valid_skill_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || character == '-'
                || character == '_'
        })
}

fn parse_tool_name_value(value: &str) -> Result<ToolName, String> {
    match clean_skill_value(value).as_str() {
        "open_url" => Ok(ToolName::OpenUrl),
        "go_back" => Ok(ToolName::GoBack),
        "go_forward" => Ok(ToolName::GoForward),
        "reload_page" => Ok(ToolName::ReloadPage),
        "get_html" => Ok(ToolName::GetHtml),
        "eval_js" => Ok(ToolName::EvalJs),
        "scroll_page" => Ok(ToolName::ScrollPage),
        "capture_screenshot" => Ok(ToolName::CaptureScreenshot),
        "set_browser_visibility" => Ok(ToolName::SetBrowserVisibility),
        "get_page_snapshot" => Ok(ToolName::GetPageSnapshot),
        "extract_page_model" => Ok(ToolName::ExtractPageModel),
        "list_interactive_elements" => Ok(ToolName::ListInteractiveElements),
        "find_element" => Ok(ToolName::FindElement),
        "click_element" => Ok(ToolName::ClickElement),
        "focus_element" => Ok(ToolName::FocusElement),
        "type_into_element" => Ok(ToolName::TypeIntoElement),
        "submit_active_form" => Ok(ToolName::SubmitActiveForm),
        "read_region" => Ok(ToolName::ReadRegion),
        "read_next_region" => Ok(ToolName::ReadNextRegion),
        "read_previous_region" => Ok(ToolName::ReadPreviousRegion),
        "stop_speaking" => Ok(ToolName::StopSpeaking),
        "start_listening" => Ok(ToolName::StartListening),
        "stop_listening" => Ok(ToolName::StopListening),
        "transcribe_command" => Ok(ToolName::TranscribeCommand),
        "set_tts_voice" => Ok(ToolName::SetTtsVoice),
        "set_playback_volume" => Ok(ToolName::SetPlaybackVolume),
        "set_playback_speed" => Ok(ToolName::SetPlaybackSpeed),
        "run_ocr" => Ok(ToolName::RunOcr),
        "merge_ocr_into_page_model" => Ok(ToolName::MergeOcrIntoPageModel),
        "get_agent_state" => Ok(ToolName::GetAgentState),
        "get_runtime_status" => Ok(ToolName::GetRuntimeStatus),
        "confirm_action" => Ok(ToolName::ConfirmAction),
        "report_result" => Ok(ToolName::ReportResult),
        other => Err(format!("unknown tool '{other}'")),
    }
}

pub(crate) fn parse_intent_name_value(value: &str) -> Result<IntentName, String> {
    match clean_skill_value(value).as_str() {
        "OpenUrl" => Ok(IntentName::OpenUrl),
        "GoBack" => Ok(IntentName::GoBack),
        "GoForward" => Ok(IntentName::GoForward),
        "ReloadPage" => Ok(IntentName::ReloadPage),
        "GetCurrentUrl" => Ok(IntentName::GetCurrentUrl),
        "ReadPage" => Ok(IntentName::ReadPage),
        "ReadTitle" => Ok(IntentName::ReadTitle),
        "ReadNext" => Ok(IntentName::ReadNext),
        "ReadPrevious" => Ok(IntentName::ReadPrevious),
        "Repeat" => Ok(IntentName::Repeat),
        "Stop" => Ok(IntentName::Stop),
        "StartListening" => Ok(IntentName::StartListening),
        "StopListening" => Ok(IntentName::StopListening),
        "TranscribeCommand" => Ok(IntentName::TranscribeCommand),
        "SetTtsVoice" => Ok(IntentName::SetTtsVoice),
        "SetPlaybackVolume" => Ok(IntentName::SetPlaybackVolume),
        "GetPlaybackVolume" => Ok(IntentName::GetPlaybackVolume),
        "SetPlaybackSpeed" => Ok(IntentName::SetPlaybackSpeed),
        "GetPlaybackSpeed" => Ok(IntentName::GetPlaybackSpeed),
        "SetBrowserVisibility" => Ok(IntentName::SetBrowserVisibility),
        "GetStatus" => Ok(IntentName::GetStatus),
        "FindElement" => Ok(IntentName::FindElement),
        "ClickElement" => Ok(IntentName::ClickElement),
        "FillInput" => Ok(IntentName::FillInput),
        "SubmitForm" => Ok(IntentName::SubmitForm),
        "Scroll" => Ok(IntentName::Scroll),
        "OcrRecovery" => Ok(IntentName::OcrRecovery),
        "Unknown" => Ok(IntentName::Unknown),
        other => Err(format!("unknown intent tag '{other}'")),
    }
}

pub(crate) fn score_skill(
    skill: &LoadedSkill,
    transcript_tokens: &HashSet<String>,
    inferred_intent: &IntentName,
    likely_tools: &[ToolName],
) -> Option<i32> {
    let skill_tokens = tokenize_text(&format!(
        "{} {} {} {}",
        skill.summary.name,
        skill.summary.description,
        skill.summary.intent_tags.join(" "),
        skill.body
    ));
    let lexical_overlap = transcript_tokens.intersection(&skill_tokens).count() as i32;
    let intent_match = skill
        .summary
        .intent_tags
        .iter()
        .any(|tag| tag == &format!("intent:{inferred_intent:?}"));
    let tool_overlap = skill
        .summary
        .allowed_tools
        .as_ref()
        .map(|tools| {
            tools
                .iter()
                .filter(|tool| likely_tools.iter().any(|candidate| candidate == *tool))
                .count() as i32
        })
        .unwrap_or(0);

    if lexical_overlap == 0 && !intent_match && tool_overlap == 0 {
        return None;
    }

    let precedence_score = match skill.source {
        SkillSource::Project => 3_000,
        SkillSource::User => 2_000,
        SkillSource::Bundled => 1_000,
    };

    Some(
        precedence_score
            + skill.summary.priority
            + (lexical_overlap * 75)
            + (tool_overlap * 100)
            + if intent_match { 500 } else { 0 },
    )
}
