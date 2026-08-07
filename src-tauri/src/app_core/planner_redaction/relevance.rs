//! Transcript-relevance ranking used to keep the sanitized page payload sent
//! to a remote planner within [`crate::resource_limits`] bounds while still
//! surfacing the regions/elements most likely to matter for the user's request.

use std::collections::BTreeSet;

use crate::page_model::{InteractiveElement, PageRegion};

use super::types::SanitizationMetadata;

fn relevance_terms(transcript: &str) -> BTreeSet<String> {
    const STOP_WORDS: &[&str] = &[
        "and", "are", "for", "from", "into", "open", "page", "please", "that", "the", "this",
        "with", "you", "your",
    ];
    transcript
        .split(|character: char| !character.is_ascii_alphanumeric())
        .map(str::to_ascii_lowercase)
        .filter(|term| term.len() >= 3 && !STOP_WORDS.contains(&term.as_str()))
        .collect()
}

fn relevance_score(text: &str, terms: &BTreeSet<String>) -> usize {
    let lower = text.to_ascii_lowercase();
    terms
        .iter()
        .map(|term| lower.match_indices(term).count())
        .sum()
}

pub(super) fn select_relevant_regions<'a>(
    regions: &'a [PageRegion],
    transcript: &str,
    limit: usize,
    metadata: &mut SanitizationMetadata,
) -> Vec<&'a PageRegion> {
    let terms = relevance_terms(transcript);
    let mut ranked = regions
        .iter()
        .enumerate()
        .map(|(index, region)| {
            let mut score = relevance_score(&region.text, &terms);
            if let Some(label) = &region.label {
                score += relevance_score(label, &terms) * 2;
            }
            (score, index, region)
        })
        .collect::<Vec<_>>();
    ranked.sort_by_key(|(score, index, _)| (std::cmp::Reverse(*score), *index));
    let selected = ranked
        .into_iter()
        .take(limit)
        .map(|(_, _, region)| region)
        .collect::<Vec<_>>();
    // `regions.len() - selected.len()` and `regions.len() - limit` are the same
    // quantity for every input: `selected.len() == regions.len().min(limit)`
    // always holds given the `.take(limit)` above, and there is no separate
    // visibility-filtering stage for regions (unlike elements) that could make
    // them diverge. CR3 P2.5 collapsed the once-duplicate
    // `relevance_filtered_regions` field into this one.
    metadata.omitted_regions += regions.len().saturating_sub(selected.len());
    selected
}

fn element_relevance_text(element: &InteractiveElement) -> String {
    [
        Some(element.tag_name.as_str()),
        element.text.as_deref(),
        element.accessible_name.as_deref(),
        element.placeholder.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ")
}

pub(super) fn select_relevant_elements<'a>(
    elements: &'a [InteractiveElement],
    transcript: &str,
    limit: usize,
    metadata: &mut SanitizationMetadata,
) -> Vec<&'a InteractiveElement> {
    let terms = relevance_terms(transcript);
    let visible = elements
        .iter()
        .filter(|element| element.visible)
        .collect::<Vec<_>>();
    let visible_count = visible.len();
    metadata.omitted_hidden_elements += elements.len().saturating_sub(visible_count);
    let mut ranked = visible
        .into_iter()
        .enumerate()
        .map(|(index, element)| {
            (
                relevance_score(&element_relevance_text(element), &terms),
                index,
                element,
            )
        })
        .collect::<Vec<_>>();
    ranked.sort_by_key(|(score, index, _)| (std::cmp::Reverse(*score), *index));
    let selected = ranked
        .into_iter()
        .take(limit)
        .map(|(_, _, element)| element)
        .collect::<Vec<_>>();
    // Count against `visible_count`, not `elements.len()`: hidden elements are
    // already accounted for above via `omitted_hidden_elements`, and counting
    // them again here would double-count them as if the relevance/limit filter
    // had also dropped them. This also makes the count equal
    // `visible_count.saturating_sub(limit)` for every input (since
    // `selected.len() == visible_count.min(limit)` always holds), so the
    // once-duplicate `relevance_filtered_elements` field (CR3 P2.5) has been
    // collapsed into this one.
    metadata.omitted_elements += visible_count.saturating_sub(selected.len());
    selected
}
