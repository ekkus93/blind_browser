use std::collections::BTreeMap;

use crate::commands::{PlannerInput, PlannerToolHistoryEntry};
use crate::page_model::{InteractiveElement, PageModel, PageRegion};

const MAX_REMOTE_REGIONS: usize = 64;
const MAX_REMOTE_ELEMENTS: usize = 128;
const MAX_REGION_TEXT_CHARS: usize = 2_000;
const MAX_ELEMENT_TEXT_CHARS: usize = 512;
const MAX_ATTRIBUTE_VALUE_CHARS: usize = 256;
const MAX_OBSERVATION_CHARS: usize = 512;

const SAFE_ATTRIBUTES: &[&str] = &[
    "type",
    "role",
    "aria-label",
    "aria-labelledby",
    "placeholder",
    "checked",
    "selected",
    "disabled",
    "name",
    "autocomplete",
];

const SENSITIVE_MARKERS: &[&str] = &[
    "password",
    "passwd",
    "secret",
    "token",
    "csrf",
    "authorization",
    "api_key",
    "apikey",
    "one-time-code",
    "current-password",
    "new-password",
    "credit-card",
    "cc-number",
    "security-answer",
    "social-security",
    "ssn",
];

const SENSITIVE_QUERY_KEYS: &[&str] = &[
    "access_token",
    "auth",
    "authorization",
    "code",
    "csrf",
    "id_token",
    "key",
    "password",
    "session",
    "signature",
    "sig",
    "token",
];

pub(crate) fn sanitize_remote_planner_input(input: &PlannerInput) -> PlannerInput {
    let mut safe = input.clone();

    safe.transcript = sanitize_free_text(&safe.transcript, MAX_REGION_TEXT_CHARS);
    safe.page_model = safe.page_model.as_ref().map(sanitize_page_model);
    safe.page_snapshot = safe.page_snapshot.as_ref().map(|snapshot| {
        let mut snapshot = snapshot.clone();
        snapshot.url = sanitize_url(&snapshot.url);
        snapshot.title = snapshot
            .title
            .as_deref()
            .map(|value| sanitize_free_text(value, MAX_ELEMENT_TEXT_CHARS));
        snapshot.visible_text_excerpt =
            sanitize_free_text(&snapshot.visible_text_excerpt, MAX_REGION_TEXT_CHARS);
        snapshot.interactive_elements = snapshot
            .interactive_elements
            .iter()
            .take(MAX_REMOTE_ELEMENTS)
            .map(sanitize_interactive_element)
            .collect();
        snapshot
    });

    safe.recent_tool_results = safe
        .recent_tool_results
        .iter()
        .map(sanitize_history_entry)
        .collect();
    for skill in &mut safe.relevant_skill_summaries {
        skill.description = sanitize_free_text(&skill.description, MAX_OBSERVATION_CHARS);
        skill.intent_tags = skill
            .intent_tags
            .iter()
            .take(16)
            .map(|tag| truncate_chars(tag, 64))
            .collect();
    }

    safe.agent_state.url = safe.agent_state.url.as_deref().map(sanitize_url);
    safe.agent_state.title = safe
        .agent_state
        .title
        .as_deref()
        .map(|value| sanitize_free_text(value, MAX_ELEMENT_TEXT_CHARS));
    safe.agent_state.last_transcript = safe
        .agent_state
        .last_transcript
        .as_deref()
        .map(|value| sanitize_free_text(value, MAX_REGION_TEXT_CHARS));
    if let Some(last_call) = &mut safe.agent_state.last_tool_call {
        last_call.observation_summary = last_call
            .observation_summary
            .iter()
            .map(|value| sanitize_free_text(value, MAX_OBSERVATION_CHARS))
            .collect();
    }

    // Pending protected actions can contain typed text and exact arguments. They
    // are local authorization state and are never planner context.
    safe.agent_state.pending_confirmation_id = None;
    safe.agent_state.pending_plan_execution = None;

    // Local filesystem paths and credential-reference metadata are operational
    // state, not planning context.
    safe.agent_state.local_tts_model_settings.model_path = None;
    safe.agent_state.local_asr_model_settings.model_path = None;
    clear_remote_planner_secret_metadata(&mut safe);

    safe
}

fn clear_remote_planner_secret_metadata(input: &mut PlannerInput) {
    let planner = &mut input.agent_state.remote_planner_settings;
    planner.base_url = planner.base_url.as_deref().map(sanitize_url);
    planner.api_key_reference = None;
    planner.api_key_masked_value = None;
    planner.api_key_reference_error = None;
    planner.organization_reference = None;

    let tts = &mut input.agent_state.remote_tts_settings;
    tts.base_url = tts.base_url.as_deref().map(sanitize_url);
    tts.api_key_reference = None;
    tts.api_key_masked_value = None;
    tts.api_key_reference_error = None;
    tts.organization_reference = None;

    let asr = &mut input.agent_state.remote_asr_settings;
    asr.base_url = asr.base_url.as_deref().map(sanitize_url);
    asr.api_key_reference = None;
    asr.api_key_masked_value = None;
    asr.api_key_reference_error = None;
    asr.organization_reference = None;
}

fn sanitize_history_entry(entry: &PlannerToolHistoryEntry) -> PlannerToolHistoryEntry {
    let mut safe = entry.clone();
    safe.observation_summary = safe
        .observation_summary
        .iter()
        .take(16)
        .map(|value| sanitize_free_text(value, MAX_OBSERVATION_CHARS))
        .collect();
    safe
}

fn sanitize_page_model(page: &PageModel) -> PageModel {
    PageModel {
        title: page
            .title
            .as_deref()
            .map(|value| sanitize_free_text(value, MAX_ELEMENT_TEXT_CHARS)),
        url: page.url.as_deref().map(sanitize_url),
        regions: page
            .regions
            .iter()
            .take(MAX_REMOTE_REGIONS)
            .map(sanitize_page_region)
            .collect(),
        interactive_elements: page
            .interactive_elements
            .iter()
            .take(MAX_REMOTE_ELEMENTS)
            .map(sanitize_interactive_element)
            .collect(),
    }
}

fn sanitize_page_region(region: &PageRegion) -> PageRegion {
    let mut safe = region.clone();
    safe.region_id = truncate_chars(&safe.region_id, 128);
    safe.label = safe
        .label
        .as_deref()
        .map(|value| sanitize_free_text(value, MAX_ELEMENT_TEXT_CHARS));
    safe.text = sanitize_free_text(&safe.text, MAX_REGION_TEXT_CHARS);
    safe
}

fn sanitize_interactive_element(element: &InteractiveElement) -> InteractiveElement {
    let sensitive = is_sensitive_element(element);
    let attributes = element
        .attributes
        .iter()
        .filter_map(|(name, value)| {
            let normalized = name.to_ascii_lowercase();
            if !SAFE_ATTRIBUTES.contains(&normalized.as_str()) {
                return None;
            }
            if sensitive
                && matches!(
                    normalized.as_str(),
                    "name" | "autocomplete" | "placeholder" | "aria-label"
                )
            {
                return None;
            }
            Some((normalized, truncate_chars(value, MAX_ATTRIBUTE_VALUE_CHARS)))
        })
        .collect::<BTreeMap<_, _>>();

    InteractiveElement {
        element_id: truncate_chars(&element.element_id, 128),
        // CSS locators can embed raw attribute values. The remote planner never
        // needs them; local deterministic resolution retains the raw model.
        dom_locator: None,
        role: element.role.clone(),
        tag_name: truncate_chars(&element.tag_name.to_ascii_lowercase(), 64),
        text: element
            .text
            .as_deref()
            .map(|value| sanitize_free_text(value, MAX_ELEMENT_TEXT_CHARS)),
        accessible_name: (!sensitive)
            .then(|| {
                element
                    .accessible_name
                    .as_deref()
                    .map(|value| sanitize_free_text(value, MAX_ELEMENT_TEXT_CHARS))
            })
            .flatten(),
        placeholder: (!sensitive)
            .then(|| {
                element
                    .placeholder
                    .as_deref()
                    .map(|value| sanitize_free_text(value, MAX_ELEMENT_TEXT_CHARS))
            })
            .flatten(),
        href: element.href.as_deref().map(sanitize_url),
        // Form-control values are local private state. No remote-planner use case
        // is allowed to opt them back in through this type.
        value: None,
        bbox: element.bbox.clone(),
        visible: element.visible,
        enabled: element.enabled,
        attributes,
    }
}

fn is_sensitive_element(element: &InteractiveElement) -> bool {
    let mut descriptors = vec![element.tag_name.to_ascii_lowercase()];
    descriptors.extend(
        ["type", "name", "id", "autocomplete"]
            .into_iter()
            .filter_map(|name| element.attributes.get(name))
            .map(|value| value.to_ascii_lowercase()),
    );
    if let Some(placeholder) = &element.placeholder {
        descriptors.push(placeholder.to_ascii_lowercase());
    }
    let combined = descriptors.join(" ");
    SENSITIVE_MARKERS
        .iter()
        .any(|marker| combined.contains(marker))
        || element
            .attributes
            .get("type")
            .is_some_and(|kind| matches!(kind.to_ascii_lowercase().as_str(), "password" | "hidden"))
}

fn sanitize_free_text(value: &str, max_chars: usize) -> String {
    let lower = value.to_ascii_lowercase();
    let assignment_markers = [
        "password=",
        "password:",
        "passwd=",
        "token=",
        "token:",
        "secret=",
        "api_key=",
        "apikey=",
        "authorization:",
        "bearer ",
    ];
    if assignment_markers
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return String::from("[REDACTED SENSITIVE TEXT]");
    }
    truncate_chars(value, max_chars)
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let truncated = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{truncated}…")
    } else {
        truncated
    }
}

fn sanitize_url(raw: &str) -> String {
    if let Ok(mut parsed) = url::Url::parse(raw) {
        let _ = parsed.set_username("");
        let _ = parsed.set_password(None);
        parsed.set_fragment(None);
        let retained = parsed
            .query_pairs()
            .filter(|(name, _)| !is_sensitive_query_key(name))
            .map(|(name, value)| (name.into_owned(), value.into_owned()))
            .collect::<Vec<_>>();
        parsed.set_query(None);
        if !retained.is_empty() {
            parsed.query_pairs_mut().extend_pairs(retained);
        }
        return parsed.to_string();
    }

    let without_fragment = raw.split('#').next().unwrap_or(raw);
    let Some((base, query)) = without_fragment.split_once('?') else {
        return truncate_chars(without_fragment, 2_048);
    };
    let retained = query
        .split('&')
        .filter(|pair| {
            let name = pair.split('=').next().unwrap_or_default();
            !is_sensitive_query_key(name)
        })
        .collect::<Vec<_>>();
    if retained.is_empty() {
        truncate_chars(base, 2_048)
    } else {
        truncate_chars(&format!("{base}?{}", retained.join("&")), 2_048)
    }
}

fn is_sensitive_query_key(name: &str) -> bool {
    let normalized = name.to_ascii_lowercase();
    SENSITIVE_QUERY_KEYS
        .iter()
        .any(|key| normalized == *key || normalized.ends_with(&format!("_{key}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page_model::{ElementRole, RegionRole, RegionSource};

    fn element(attributes: &[(&str, &str)], value: &str) -> InteractiveElement {
        InteractiveElement {
            element_id: String::from("element-1"),
            dom_locator: Some(String::from("input[value='do-not-leak']")),
            role: ElementRole::Input,
            tag_name: String::from("input"),
            text: None,
            accessible_name: Some(String::from("Account field")),
            placeholder: Some(String::from("Enter value")),
            href: Some(String::from(
                "https://user:pass@example.com/form?token=secret&safe=ok#private",
            )),
            value: Some(value.to_string()),
            bbox: None,
            visible: false,
            enabled: true,
            attributes: attributes
                .iter()
                .map(|(name, value)| (name.to_string(), value.to_string()))
                .collect(),
        }
    }

    #[test]
    fn password_and_hidden_values_never_enter_planner_page_json() {
        for (kind, secret) in [("password", "hunter2"), ("hidden", "csrf-secret")] {
            let page = PageModel {
                title: Some(String::from("Private form")),
                url: Some(String::from("https://example.com/?access_token=url-secret")),
                regions: Vec::new(),
                interactive_elements: vec![element(&[("type", kind), ("name", "token")], secret)],
            };
            let json = serde_json::to_string(&sanitize_page_model(&page)).unwrap();
            assert!(!json.contains(secret));
            assert!(!json.contains("do-not-leak"));
            assert!(!json.contains("url-secret"));
            assert!(!json.contains("csrf-secret"));
        }
    }

    #[test]
    fn remote_elements_use_an_attribute_allowlist_and_omit_all_values() {
        let safe = sanitize_interactive_element(&element(
            &[
                ("type", "text"),
                ("aria-label", "Search"),
                ("data-session", "private-data"),
                ("onclick", "exfiltrate()"),
                ("style", "background:url(secret)"),
            ],
            "private draft",
        ));
        let json = serde_json::to_string(&safe).unwrap();
        assert_eq!(safe.value, None);
        assert_eq!(safe.dom_locator, None);
        assert_eq!(
            safe.attributes.get("aria-label"),
            Some(&String::from("Search"))
        );
        assert!(!json.contains("private draft"));
        assert!(!json.contains("private-data"));
        assert!(!json.contains("onclick"));
        assert!(!json.contains("background:url"));
    }

    #[test]
    fn secret_bearing_url_parts_are_removed() {
        let sanitized = sanitize_url(
            "https://user:pass@example.com/path?token=abc&safe=ok&redirect=yes#fragment",
        );
        assert!(!sanitized.contains("user"));
        assert!(!sanitized.contains("pass"));
        assert!(!sanitized.contains("abc"));
        assert!(!sanitized.contains("fragment"));
        assert!(sanitized.contains("safe=ok"));
        assert!(sanitized.contains("redirect=yes"));
    }

    #[test]
    fn page_payload_is_bounded_before_remote_serialization() {
        let region = PageRegion {
            region_id: String::from("region"),
            role: RegionRole::Paragraph,
            label: None,
            text: "x".repeat(MAX_REGION_TEXT_CHARS + 100),
            bbox: None,
            source: RegionSource::Dom,
        };
        let page = PageModel {
            title: None,
            url: None,
            regions: vec![region; MAX_REMOTE_REGIONS + 10],
            interactive_elements: vec![
                element(&[("type", "text")], "draft");
                MAX_REMOTE_ELEMENTS + 10
            ],
        };
        let safe = sanitize_page_model(&page);
        assert_eq!(safe.regions.len(), MAX_REMOTE_REGIONS);
        assert_eq!(safe.interactive_elements.len(), MAX_REMOTE_ELEMENTS);
        assert!(safe.regions[0].text.chars().count() <= MAX_REGION_TEXT_CHARS + 1);
    }
}
