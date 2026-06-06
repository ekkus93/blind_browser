use super::*;

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
