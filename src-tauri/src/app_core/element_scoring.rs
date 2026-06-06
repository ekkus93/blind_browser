use crate::commands::{ElementCandidate, FindElementInput, ToolError};
use crate::narration::find_region_index;
use crate::page_model::{ElementRole, InteractiveElement, PageModel, PageRegion, Rect};

pub(crate) fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub(crate) fn region_bbox_by_id(
    regions: &[PageRegion],
    region_id: &str,
) -> Result<Rect, crate::commands::ToolError> {
    let Some(region_index) = find_region_index(regions, region_id) else {
        return Err(crate::commands::ToolError {
            code: String::from("unknown_region_id"),
            message: String::from(
                "capture_screenshot could not find the requested region_id in the current page model",
            ),
            retryable: false,
            details: Some(serde_json::json!({ "region_id": region_id })),
        });
    };

    let region = &regions[region_index];
    let Some(bbox) = region.bbox.clone() else {
        return Err(crate::commands::ToolError {
            code: String::from("missing_region_bbox"),
            message: String::from(
                "capture_screenshot requires a bounding box for the requested region_id",
            ),
            retryable: false,
            details: Some(serde_json::json!({ "region_id": region_id })),
        });
    };

    if bbox.width <= 0.0 || bbox.height <= 0.0 {
        return Err(crate::commands::ToolError {
            code: String::from("invalid_region_bbox"),
            message: String::from(
                "capture_screenshot requires a positive bounding box for the requested region_id",
            ),
            retryable: false,
            details: Some(serde_json::json!({
                "region_id": region_id,
                "x": bbox.x,
                "y": bbox.y,
                "width": bbox.width,
                "height": bbox.height,
            })),
        });
    }

    Ok(bbox)
}

pub(crate) fn focusable_field_elements(page: &PageModel) -> Vec<InteractiveElement> {
    filter_interactive_elements(
        &page.interactive_elements,
        true,
        Some(&[
            ElementRole::Input,
            ElementRole::TextArea,
            ElementRole::Select,
        ]),
    )
    .into_iter()
    .filter(|element| {
        element.enabled
            && element
                .dom_locator
                .as_deref()
                .map(str::trim)
                .is_some_and(|locator| !locator.is_empty())
    })
    .collect()
}

pub(crate) fn submittable_form_elements(page: &PageModel) -> Vec<InteractiveElement> {
    filter_interactive_elements(&page.interactive_elements, true, Some(&[ElementRole::Form]))
        .into_iter()
        .filter(|element| {
            element
                .dom_locator
                .as_deref()
                .map(str::trim)
                .is_some_and(|locator| !locator.is_empty())
        })
        .collect()
}

pub(crate) fn summarize_candidate_names(
    page: &PageModel,
    candidates: &[ElementCandidate],
) -> Vec<String> {
    candidates
        .iter()
        .filter_map(|candidate| {
            page.interactive_elements
                .iter()
                .find(|element| element.element_id == candidate.element_id)
                .map(describe_field_element)
        })
        .take(super::MAX_DIRECT_FIELD_CANDIDATE_NAMES)
        .collect()
}

pub(crate) fn summarize_form_candidate_names(forms: &[InteractiveElement]) -> Vec<String> {
    forms
        .iter()
        .map(describe_form_element)
        .take(super::MAX_DIRECT_FIELD_CANDIDATE_NAMES)
        .collect()
}

fn describe_field_element(element: &InteractiveElement) -> String {
    element
        .accessible_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            element
                .placeholder
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .or_else(|| {
            element
                .text
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .map(String::from)
        .unwrap_or_else(|| element.element_id.clone())
}

pub(crate) fn describe_form_element(element: &InteractiveElement) -> String {
    element
        .accessible_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            element
                .text
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .or_else(|| {
            element
                .attributes
                .get("id")
                .map(String::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .map(|description| format!("the {description} form"))
        .unwrap_or_else(|| String::from("the current form"))
}

pub(crate) fn filter_interactive_elements(
    interactive_elements: &[InteractiveElement],
    visible_only: bool,
    roles: Option<&[ElementRole]>,
) -> Vec<InteractiveElement> {
    interactive_elements
        .iter()
        .filter(|element| !visible_only || element.visible)
        .filter(|element| roles.is_none_or(|roles| roles.contains(&element.role)))
        .cloned()
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FindElementQuery {
    pub(crate) summary: String,
    pub(crate) description: Option<String>,
    pub(crate) text: Option<String>,
    pub(crate) role: Option<ElementRole>,
    pub(crate) color_hint: Option<String>,
    pub(crate) nearby_text: Option<String>,
    pub(crate) selector_hint: Option<String>,
}

pub(crate) fn build_find_element_query(
    input: &FindElementInput,
) -> Result<FindElementQuery, ToolError> {
    let description = normalize_optional_text(Some(input.description.clone()));
    let text = normalize_optional_text(input.text.clone());
    let color_hint = normalize_optional_text(input.color_hint.clone());
    let nearby_text = normalize_optional_text(input.nearby_text.clone());
    let selector_hint = normalize_optional_text(input.selector_hint.clone());

    if description.is_none()
        && text.is_none()
        && input.role.is_none()
        && color_hint.is_none()
        && nearby_text.is_none()
        && selector_hint.is_none()
    {
        return Err(ToolError {
            code: String::from("invalid_find_query"),
            message: String::from("find_element requires at least one populated search field"),
            retryable: false,
            details: None,
        });
    }

    let mut summary_parts = Vec::new();
    if let Some(description) = description.as_ref() {
        summary_parts.push(format!("description={description}"));
    }
    if let Some(text) = text.as_ref() {
        summary_parts.push(format!("text={text}"));
    }
    if let Some(role) = input.role.as_ref() {
        summary_parts.push(format!("role={role:?}"));
    }
    if let Some(color_hint) = color_hint.as_ref() {
        summary_parts.push(format!("color_hint={color_hint}"));
    }
    if let Some(nearby_text) = nearby_text.as_ref() {
        summary_parts.push(format!("nearby_text={nearby_text}"));
    }
    if let Some(selector_hint) = selector_hint.as_ref() {
        summary_parts.push(format!("selector_hint={selector_hint}"));
    }

    Ok(FindElementQuery {
        summary: summary_parts.join("; "),
        description,
        text,
        role: input.role.clone(),
        color_hint,
        nearby_text,
        selector_hint,
    })
}

pub(crate) fn rank_find_element_candidates(
    elements: &[InteractiveElement],
    query: &FindElementQuery,
    candidate_limit: usize,
) -> Vec<ElementCandidate> {
    let mut candidates = elements
        .iter()
        .filter_map(|element| score_interactive_element(element, query))
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| {
        right
            .confidence_bps
            .cmp(&left.confidence_bps)
            .then_with(|| left.element_id.cmp(&right.element_id))
    });
    candidates.truncate(candidate_limit);
    candidates
}

pub(crate) fn determine_find_element_resolution(
    candidates: &[ElementCandidate],
    confirmation_confidence_threshold: f32,
) -> (Option<String>, Option<f32>, bool) {
    let Some(top_candidate) = candidates.first() else {
        return (None, None, false);
    };

    let top_confidence = Some(f32::from(top_candidate.confidence_bps) / 10_000.0);
    let required_confidence_bps =
        (confirmation_confidence_threshold.clamp(0.0, 1.0) * 10_000.0).round() as u16;
    let below_threshold = top_candidate.confidence_bps < required_confidence_bps;
    let ambiguous_with_runner_up = candidates.get(1).is_some_and(|second_candidate| {
        top_candidate
            .confidence_bps
            .saturating_sub(second_candidate.confidence_bps)
            <= super::FIND_ELEMENT_AMBIGUITY_MARGIN_BPS
    });

    if below_threshold || ambiguous_with_runner_up {
        (None, top_confidence, true)
    } else {
        (
            Some(top_candidate.element_id.clone()),
            top_confidence,
            false,
        )
    }
}

#[derive(Debug, Default)]
struct FindElementScore {
    score_bps: u16,
    matched_on: Vec<String>,
    rationale_codes: Vec<String>,
}

impl FindElementScore {
    fn push_match(&mut self, match_label: &str, rationale_code: impl Into<String>, score_bps: u16) {
        self.score_bps = self.score_bps.saturating_add(score_bps);
        self.matched_on.push(match_label.to_string());
        self.rationale_codes.push(rationale_code.into());
    }
}

struct AttributeHintSpec<'a> {
    match_label: &'a str,
    exact_score_bps: u16,
    contains_score_bps: u16,
}

fn score_interactive_element(
    element: &InteractiveElement,
    query: &FindElementQuery,
) -> Option<ElementCandidate> {
    let mut score = FindElementScore::default();

    if let Some(role) = query.role.as_ref() {
        if &element.role == role {
            score.push_match("role", "role_match", 1_800);
        } else {
            return None;
        }
    }

    if let Some(description) = query.description.as_ref() {
        let field_match =
            score_text_query_against_element(description, element, "description", &mut score);
        if !field_match && query.role.is_none() {
            return None;
        }
    }

    if let Some(text) = query.text.as_ref() {
        let field_match = score_text_query_against_element(text, element, "text", &mut score);
        if !field_match && query.description.is_none() && query.role.is_none() {
            return None;
        }
    }

    if let Some(nearby_text) = query.nearby_text.as_ref() {
        score_attribute_hint(
            nearby_text,
            element,
            AttributeHintSpec {
                match_label: "nearby_text",
                exact_score_bps: 1_600,
                contains_score_bps: 900,
            },
            &mut score,
        );
    }

    if let Some(selector_hint) = query.selector_hint.as_ref() {
        score_attribute_hint(
            selector_hint,
            element,
            AttributeHintSpec {
                match_label: "selector_hint",
                exact_score_bps: 1_500,
                contains_score_bps: 800,
            },
            &mut score,
        );
    }

    if let Some(color_hint) = query.color_hint.as_ref() {
        score_attribute_hint(
            color_hint,
            element,
            AttributeHintSpec {
                match_label: "color_hint",
                exact_score_bps: 500,
                contains_score_bps: 250,
            },
            &mut score,
        );
    }

    if score.score_bps == 0 {
        return None;
    }

    if element.enabled {
        score.score_bps = score.score_bps.saturating_add(100);
    } else {
        score.rationale_codes.push(String::from("disabled_penalty"));
        score.score_bps = score.score_bps.saturating_sub(300);
    }

    Some(ElementCandidate {
        element_id: element.element_id.clone(),
        confidence_bps: score.score_bps.min(10_000),
        matched_on: score.matched_on,
        rationale_codes: score.rationale_codes,
    })
}

fn score_text_query_against_element(
    query_text: &str,
    element: &InteractiveElement,
    match_label: &str,
    score: &mut FindElementScore,
) -> bool {
    let normalized_query = normalize_search_text(query_text);
    let accessible_name = element
        .accessible_name
        .as_deref()
        .map(normalize_search_text);
    let visible_text = element.text.as_deref().map(normalize_search_text);
    let placeholder = element.placeholder.as_deref().map(normalize_search_text);

    if accessible_name.as_deref() == Some(normalized_query.as_str()) {
        score.push_match(match_label, "accessible_name_exact", 4_200);
        return true;
    }
    if visible_text.as_deref() == Some(normalized_query.as_str()) {
        score.push_match(match_label, "visible_text_exact", 4_000);
        return true;
    }
    if placeholder.as_deref() == Some(normalized_query.as_str()) {
        score.push_match(match_label, "placeholder_exact", 3_400);
        return true;
    }

    let overlap_score = text_overlap_score(&normalized_query, element);
    if overlap_score > 0 {
        score.push_match(match_label, "lexical_overlap", overlap_score);
        return true;
    }

    false
}

fn score_attribute_hint(
    hint: &str,
    element: &InteractiveElement,
    spec: AttributeHintSpec<'_>,
    score: &mut FindElementScore,
) -> bool {
    let normalized_hint = normalize_search_text(hint);
    let attribute_blob = element
        .attributes
        .iter()
        .map(|(key, value)| format!("{key} {value}"))
        .collect::<Vec<_>>()
        .join(" ");
    let normalized_attributes = normalize_search_text(&attribute_blob);

    if normalized_attributes.is_empty() {
        return false;
    }

    if normalized_attributes == normalized_hint {
        score.push_match(
            spec.match_label,
            format!("{}_exact", spec.match_label),
            spec.exact_score_bps,
        );
        true
    } else if normalized_attributes.contains(&normalized_hint) {
        score.push_match(
            spec.match_label,
            format!("{}_contains", spec.match_label),
            spec.contains_score_bps,
        );
        true
    } else {
        false
    }
}

fn text_overlap_score(query_text: &str, element: &InteractiveElement) -> u16 {
    let query_terms = tokenize_search_text(query_text);
    if query_terms.is_empty() {
        return 0;
    }

    let element_blob = [
        element.accessible_name.as_deref().unwrap_or_default(),
        element.text.as_deref().unwrap_or_default(),
        element.placeholder.as_deref().unwrap_or_default(),
        element.href.as_deref().unwrap_or_default(),
        element.value.as_deref().unwrap_or_default(),
    ]
    .join(" ");
    let element_terms = tokenize_search_text(&element_blob);
    if element_terms.is_empty() {
        return 0;
    }

    let overlap = query_terms
        .iter()
        .filter(|term| element_terms.contains(*term))
        .count();
    if overlap == 0 {
        0
    } else {
        let ratio = overlap as f32 / query_terms.len() as f32;
        (900.0 + (ratio * 2_100.0)).round() as u16
    }
}

fn normalize_search_text(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn tokenize_search_text(value: &str) -> Vec<String> {
    normalize_search_text(value)
        .split(' ')
        .filter(|term| !term.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}
