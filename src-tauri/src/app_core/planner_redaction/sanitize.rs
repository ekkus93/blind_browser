//! Field-by-field redaction of a [`PlannerInput`] into the [`super::types`]
//! shapes actually sent to a remote planner: truncates, drops sensitive
//! fields outright, and strips URLs down to origin + path.

use crate::commands::{PageSnapshotData, PlannerInput, PlannerToolHistoryEntry, SkillSummary};
use crate::diagnostic_redaction::sanitize_url_for_display;
use crate::page_model::{InteractiveElement, PageModel, PageRegion};
use crate::resource_limits::{
    MAX_ELEMENT_TEXT_CHARS, MAX_IDENTIFIER_CHARS, MAX_INTENT_TAGS, MAX_OBSERVATION_CHARS,
    MAX_REGION_TEXT_CHARS, MAX_REMOTE_ELEMENTS, MAX_REMOTE_HISTORY_BYTES,
    MAX_REMOTE_HISTORY_ENTRIES, MAX_REMOTE_REGIONS, MAX_REMOTE_SKILLS, MAX_URL_CHARS,
};

use super::relevance::{select_relevant_elements, select_relevant_regions};
use super::sensitive::{contains_sensitive_material, is_sensitive_element};
use super::types::{
    PlannerSafeElementAttributes, PlannerSafeUrl, RemoteAgentState, RemoteInteractiveElement,
    RemotePageModel, RemotePageRegion, RemotePageSnapshot, RemoteSkillSummary,
    RemoteToolObservation, SanitizationMetadata, UntrustedText,
};

const SAFE_INPUT_TYPES: &[&str] = &[
    "button", "checkbox", "email", "number", "radio", "range", "reset", "search", "submit", "tel",
    "text", "url",
];

const SAFE_AUTOCOMPLETE_HINTS: &[&str] = &[
    "additional-name",
    "address-level1",
    "address-level2",
    "address-line1",
    "address-line2",
    "bday",
    "country",
    "country-name",
    "email",
    "family-name",
    "given-name",
    "honorific-prefix",
    "honorific-suffix",
    "language",
    "name",
    "nickname",
    "organization",
    "organization-title",
    "postal-code",
    "sex",
    "street-address",
    "tel",
    "url",
    "username",
];

pub(super) fn sanitize_agent_state(
    input: &PlannerInput,
    metadata: &mut SanitizationMetadata,
) -> RemoteAgentState {
    RemoteAgentState {
        page_id: input
            .agent_state
            .page_id
            .as_deref()
            .map(truncate_identifier),
        url: input
            .agent_state
            .url
            .as_deref()
            .map(|value| sanitize_url(value, metadata)),
        title: input
            .agent_state
            .title
            .as_deref()
            .map(|value| sanitize_text(value, MAX_ELEMENT_TEXT_CHARS, metadata)),
        browser_visibility: input.agent_state.browser_visibility,
        browser_history: input.agent_state.browser_history.clone(),
        narration_cursor: input.agent_state.narration_cursor.clone(),
        speaking: input.agent_state.speaking,
        listening_state: input.agent_state.listening_state.clone(),
        audio: input.agent_state.audio.clone(),
    }
}

pub(super) fn sanitize_page_snapshot(
    snapshot: &PageSnapshotData,
    transcript: &str,
    metadata: &mut SanitizationMetadata,
) -> RemotePageSnapshot {
    let selected_elements = select_relevant_elements(
        &snapshot.interactive_elements,
        transcript,
        MAX_REMOTE_ELEMENTS,
        metadata,
    );

    RemotePageSnapshot {
        page_id: truncate_identifier(&snapshot.page_id),
        url: sanitize_url(&snapshot.url, metadata),
        title: snapshot
            .title
            .as_deref()
            .map(|value| sanitize_text(value, MAX_ELEMENT_TEXT_CHARS, metadata)),
        visible_text_excerpt: sanitize_text(
            &snapshot.visible_text_excerpt,
            MAX_REGION_TEXT_CHARS,
            metadata,
        ),
        interactive_elements: selected_elements
            .into_iter()
            .map(|element| sanitize_interactive_element(element, metadata))
            .collect(),
        scroll_y: snapshot.scroll_y,
        viewport_width: snapshot.viewport_width,
        viewport_height: snapshot.viewport_height,
        document_height: snapshot.document_height,
    }
}

pub(super) fn sanitize_page_model(
    page: &PageModel,
    transcript: &str,
    metadata: &mut SanitizationMetadata,
) -> RemotePageModel {
    let selected_regions =
        select_relevant_regions(&page.regions, transcript, MAX_REMOTE_REGIONS, metadata);
    let selected_elements = select_relevant_elements(
        &page.interactive_elements,
        transcript,
        MAX_REMOTE_ELEMENTS,
        metadata,
    );

    RemotePageModel {
        title: page
            .title
            .as_deref()
            .map(|value| sanitize_text(value, MAX_ELEMENT_TEXT_CHARS, metadata)),
        url: page
            .url
            .as_deref()
            .map(|value| sanitize_url(value, metadata)),
        regions: selected_regions
            .into_iter()
            .map(|region| sanitize_page_region(region, metadata))
            .collect(),
        interactive_elements: selected_elements
            .into_iter()
            .map(|element| sanitize_interactive_element(element, metadata))
            .collect(),
    }
}

fn sanitize_page_region(
    region: &PageRegion,
    metadata: &mut SanitizationMetadata,
) -> RemotePageRegion {
    RemotePageRegion {
        region_id: truncate_identifier(&region.region_id),
        role: region.role.clone(),
        label: region
            .label
            .as_deref()
            .map(|value| sanitize_text(value, MAX_ELEMENT_TEXT_CHARS, metadata)),
        text: sanitize_text(&region.text, MAX_REGION_TEXT_CHARS, metadata),
        bbox: region.bbox.clone(),
        source: region.source.clone(),
    }
}

pub(super) fn sanitize_interactive_element(
    element: &InteractiveElement,
    metadata: &mut SanitizationMetadata,
) -> RemoteInteractiveElement {
    let sensitive = is_sensitive_element(element);

    RemoteInteractiveElement {
        element_id: truncate_identifier(&element.element_id),
        role: element.role.clone(),
        tag_name: truncate_chars(&element.tag_name.to_ascii_lowercase(), 64, metadata),
        text: (!sensitive)
            .then(|| {
                element
                    .text
                    .as_deref()
                    .map(|value| sanitize_text(value, MAX_ELEMENT_TEXT_CHARS, metadata))
            })
            .flatten(),
        accessible_name: (!sensitive)
            .then(|| {
                element
                    .accessible_name
                    .as_deref()
                    .map(|value| sanitize_text(value, MAX_ELEMENT_TEXT_CHARS, metadata))
            })
            .flatten(),
        placeholder: (!sensitive)
            .then(|| {
                element
                    .placeholder
                    .as_deref()
                    .map(|value| sanitize_text(value, MAX_ELEMENT_TEXT_CHARS, metadata))
            })
            .flatten(),
        href: (!sensitive)
            .then(|| {
                element
                    .href
                    .as_deref()
                    .map(|value| sanitize_url(value, metadata))
            })
            .flatten(),
        bbox: element.bbox.clone(),
        visible: element.visible,
        enabled: element.enabled,
        sensitive,
        safe_attributes: sanitize_element_attributes(element, sensitive),
    }
}

fn sanitize_element_attributes(
    element: &InteractiveElement,
    sensitive: bool,
) -> PlannerSafeElementAttributes {
    if sensitive {
        return PlannerSafeElementAttributes::default();
    }

    PlannerSafeElementAttributes {
        input_type: element
            .attributes
            .get("type")
            .map(|value| value.trim().to_ascii_lowercase())
            .filter(|value| SAFE_INPUT_TYPES.contains(&value.as_str())),
        checked: element
            .attributes
            .get("checked")
            .and_then(|value| parse_boolean_attribute(value)),
        selected: element
            .attributes
            .get("selected")
            .and_then(|value| parse_boolean_attribute(value)),
        disabled: element
            .attributes
            .get("disabled")
            .and_then(|value| parse_boolean_attribute(value)),
        autocomplete: element
            .attributes
            .get("autocomplete")
            .map(|value| value.trim().to_ascii_lowercase())
            .filter(|value| SAFE_AUTOCOMPLETE_HINTS.contains(&value.as_str())),
    }
}

pub(super) fn sanitize_history(
    history: &[PlannerToolHistoryEntry],
    metadata: &mut SanitizationMetadata,
) -> Vec<RemoteToolObservation> {
    let mut sanitized = Vec::new();
    let mut serialized_bytes = 2_usize;
    for entry in history.iter().take(MAX_REMOTE_HISTORY_ENTRIES) {
        let candidate = RemoteToolObservation {
            tool_name: entry.tool_name.clone(),
            ok: entry.ok,
            observation_summary: entry
                .observation_summary
                .iter()
                .take(16)
                .map(|value| sanitize_text(value, MAX_OBSERVATION_CHARS, metadata))
                .collect(),
        };
        let candidate_bytes = match serde_json::to_vec(&candidate) {
            Ok(bytes) => bytes.len().saturating_add(1),
            Err(_) => break,
        };
        if serialized_bytes.saturating_add(candidate_bytes) > MAX_REMOTE_HISTORY_BYTES {
            break;
        }
        serialized_bytes = serialized_bytes.saturating_add(candidate_bytes);
        sanitized.push(candidate);
    }
    metadata.omitted_history_entries += history.len().saturating_sub(sanitized.len());
    sanitized
}

pub(super) fn sanitize_skills(
    skills: &[SkillSummary],
    metadata: &mut SanitizationMetadata,
) -> Vec<RemoteSkillSummary> {
    metadata.omitted_skill_summaries += skills.len().saturating_sub(MAX_REMOTE_SKILLS);

    skills
        .iter()
        .take(MAX_REMOTE_SKILLS)
        .map(|skill| RemoteSkillSummary {
            name: truncate_identifier(&skill.name),
            description: sanitize_text(&skill.description, MAX_OBSERVATION_CHARS, metadata),
            intent_tags: skill
                .intent_tags
                .iter()
                .take(MAX_INTENT_TAGS)
                .map(|tag| truncate_chars(tag, 64, metadata))
                .collect(),
            allowed_tools: skill.allowed_tools.clone(),
            requires_confirmation: skill.requires_confirmation,
            priority: skill.priority,
        })
        .collect()
}

pub(super) fn sanitize_text(
    value: &str,
    max_chars: usize,
    metadata: &mut SanitizationMetadata,
) -> UntrustedText {
    if contains_sensitive_material(value) {
        metadata.redacted_text_fields += 1;
        return UntrustedText(String::from("[REDACTED SENSITIVE TEXT]"));
    }

    UntrustedText(truncate_chars(value, max_chars, metadata))
}

pub(super) fn sanitize_url(raw: &str, metadata: &mut SanitizationMetadata) -> PlannerSafeUrl {
    let Some(safe) = sanitize_url_for_display(raw) else {
        metadata.redacted_text_fields += 1;
        return PlannerSafeUrl(String::from("[REDACTED INVALID URL]"));
    };
    if safe.removed_query {
        metadata.query_values_removed += 1;
    }
    PlannerSafeUrl(truncate_chars(&safe.value, MAX_URL_CHARS, metadata))
}

pub(super) fn truncate_identifier(value: &str) -> String {
    value.chars().take(MAX_IDENTIFIER_CHARS).collect()
}

fn truncate_chars(value: &str, max_chars: usize, metadata: &mut SanitizationMetadata) -> String {
    let mut chars = value.chars();
    let truncated = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        metadata.truncated_text_fields += 1;
        format!("{truncated}…")
    } else {
        truncated
    }
}

fn parse_boolean_attribute(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "" | "1" | "true" | "checked" | "selected" | "disabled" => Some(true),
        "0" | "false" => Some(false),
        _ => None,
    }
}
