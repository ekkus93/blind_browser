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

pub fn previous_region_index(cursor: &NarrationCursor, region_count: usize) -> Option<usize> {
    if region_count == 0 {
        return None;
    }

    match cursor.current_index {
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
