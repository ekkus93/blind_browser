use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::page_model::{PageRegion, RegionRole};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
pub struct NarrationCursor {
    pub current_region_id: Option<String>,
    pub current_index: Option<usize>,
    pub total_regions: usize,
}

pub fn find_region_index(regions: &[PageRegion], region_id: &str) -> Option<usize> {
    let normalized_region_id = region_id.trim();
    regions
        .iter()
        .position(|region| region.region_id == normalized_region_id)
}

pub fn cursor_for_index(regions: &[PageRegion], index: usize) -> NarrationCursor {
    NarrationCursor {
        current_region_id: regions.get(index).map(|region| region.region_id.clone()),
        current_index: Some(index),
        total_regions: regions.len(),
    }
}

/// Returns the index narration should advance to, or `None` if there is no
/// next region to read.
///
/// If `cursor.current_index` is stale relative to `region_count` (e.g. the
/// page re-extracted with fewer regions than when the cursor was set), this
/// never subtracts and only ever compares, so an out-of-range cursor safely
/// falls through to "no next region" rather than indexing past the end.
pub fn next_region_index(cursor: &NarrationCursor, region_count: usize) -> Option<usize> {
    if region_count == 0 {
        return None;
    }

    match cursor.current_index {
        Some(index) if index + 1 < region_count => Some(index + 1),
        Some(_) => None,
        None => Some(0),
    }
}

/// Returns the index narration should move to when reading backward, or
/// `None` if there is no previous region to read.
///
/// If `cursor.current_index` is stale relative to `region_count` (e.g. the
/// page re-extracted with fewer regions than when the cursor was set — a
/// SPA re-render, a cookie banner replacing content, lazy content
/// collapsing), the position is clamped to the last valid index before
/// stepping backward, rather than indexing past the end of the now-shorter
/// region list. The returned index, if any, is always `< region_count`.
pub fn previous_region_index(cursor: &NarrationCursor, region_count: usize) -> Option<usize> {
    if region_count == 0 {
        return None;
    }

    match cursor.current_index {
        Some(index) if index >= region_count => Some(region_count - 1),
        Some(index) if index > 0 => Some(index - 1),
        _ => None,
    }
}

pub fn spoken_text_for_region(region: &PageRegion) -> String {
    let label = region
        .label
        .as_deref()
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| match region.role {
            RegionRole::Title => Some(String::from("Title")),
            RegionRole::Heading => Some(String::from("Heading")),
            RegionRole::Section => Some(String::from("Section")),
            RegionRole::Paragraph | RegionRole::Other => None,
        });
    let text = region.text.trim();

    match (label.as_deref(), text.is_empty()) {
        (Some(label), true) => label.to_string(),
        (Some(label), false) if !text.starts_with(label) => format!("{label}. {text}"),
        _ => text.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        cursor_for_index, find_region_index, next_region_index, previous_region_index,
        spoken_text_for_region,
    };
    use crate::narration::NarrationCursor;
    use crate::page_model::{PageRegion, RegionRole, RegionSource};

    fn sample_regions() -> Vec<PageRegion> {
        vec![
            PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Title,
                label: Some(String::from("Title")),
                text: String::from("Page title"),
                bbox: None,
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("region-2"),
                role: RegionRole::Paragraph,
                label: Some(String::from("Body")),
                text: String::from("Body text"),
                bbox: None,
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("region-3"),
                role: RegionRole::Section,
                label: Some(String::from("Footer")),
                text: String::from("Footer text"),
                bbox: None,
                source: RegionSource::Dom,
            },
        ]
    }

    #[test]
    fn find_region_index_matches_trimmed_region_id() {
        let regions = sample_regions();
        assert_eq!(find_region_index(&regions, " region-2 "), Some(1));
        assert_eq!(find_region_index(&regions, "missing"), None);
    }

    #[test]
    fn cursor_for_index_sets_region_identity_and_total() {
        let regions = sample_regions();
        let cursor = cursor_for_index(&regions, 1);

        assert_eq!(cursor.current_region_id.as_deref(), Some("region-2"));
        assert_eq!(cursor.current_index, Some(1));
        assert_eq!(cursor.total_regions, 3);
    }

    #[test]
    fn next_region_index_starts_from_first_region_when_cursor_is_unset() {
        assert_eq!(next_region_index(&NarrationCursor::default(), 3), Some(0));
    }

    #[test]
    fn previous_region_index_stops_at_start_when_cursor_is_unset() {
        assert_eq!(previous_region_index(&NarrationCursor::default(), 3), None);
        assert_eq!(
            previous_region_index(
                &NarrationCursor {
                    current_region_id: Some(String::from("region-1")),
                    current_index: Some(0),
                    total_regions: 3,
                },
                3,
            ),
            None
        );
    }

    #[test]
    fn previous_region_index_clamps_a_stale_cursor_instead_of_underflowing() {
        // Regression test: the region list can shrink out from under the
        // cursor (e.g. a re-extraction after an SPA re-render) without a
        // navigation event resetting narration_cursor. A cursor pointing past
        // the end of the new, shorter list must clamp rather than compute an
        // out-of-bounds index.
        let stale_cursor = NarrationCursor {
            current_region_id: Some(String::from("region-8")),
            current_index: Some(7),
            total_regions: 10,
        };

        // region_count shrank to 3; a naive `index - 1` would compute 6, which
        // is out of bounds for a 3-element list.
        assert_eq!(previous_region_index(&stale_cursor, 3), Some(2));

        // The clamped index is always a valid position, never region_count
        // itself or higher.
        for region_count in 1..=10 {
            let index = previous_region_index(&stale_cursor, region_count);
            if let Some(index) = index {
                assert!(
                    index < region_count,
                    "previous_region_index returned {index} for region_count {region_count}"
                );
            }
        }
    }

    #[test]
    fn next_region_index_does_not_advance_past_a_shrunk_region_list() {
        // Same stale-cursor scenario for the "next" direction: it must never
        // panic or return an out-of-range index, even though this direction
        // was already bounds-safe before the previous_region_index fix.
        let stale_cursor = NarrationCursor {
            current_region_id: Some(String::from("region-8")),
            current_index: Some(7),
            total_regions: 10,
        };
        assert_eq!(next_region_index(&stale_cursor, 3), None);
    }

    #[test]
    fn spoken_text_for_region_prefixes_label_when_needed() {
        let region = PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Heading,
            label: Some(String::from("Heading")),
            text: String::from("Welcome to the page"),
            bbox: None,
            source: RegionSource::Dom,
        };

        assert_eq!(
            spoken_text_for_region(&region),
            "Heading. Welcome to the page"
        );
    }

    #[test]
    fn spoken_text_for_region_avoids_repeating_existing_label_prefix() {
        let region = PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Heading,
            label: Some(String::from("Heading")),
            text: String::from("Heading one overview"),
            bbox: None,
            source: RegionSource::Dom,
        };

        assert_eq!(spoken_text_for_region(&region), "Heading one overview");
    }

    #[test]
    fn spoken_text_for_region_falls_back_to_structured_role_when_label_missing() {
        let region = PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Heading,
            label: None,
            text: String::from("Welcome to the page"),
            bbox: None,
            source: RegionSource::Dom,
        };

        assert_eq!(
            spoken_text_for_region(&region),
            "Heading. Welcome to the page"
        );
    }
}
