use crate::commands::ToolError;
use crate::ocr::OcrSettings;
use crate::page_model::{PageModel, PageRegion, RegionRole, RegionSource};

pub(crate) fn merge_ocr_text_into_page_model(
    page: &mut PageModel,
    region_id: Option<&str>,
    ocr_text: &str,
    source_bbox: Option<crate::page_model::Rect>,
    next_region_id: String,
) -> Result<Vec<String>, ToolError> {
    let normalized_text = ocr_text.trim();
    if normalized_text.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_ocr_text"),
            message: String::from("merge_ocr_into_page_model requires non-empty ocr_text"),
            retryable: false,
            details: None,
        });
    }

    if let Some(region_id) = region_id {
        let Some(region) = page
            .regions
            .iter_mut()
            .find(|region| region.region_id == region_id)
        else {
            return Err(ToolError {
                code: String::from("unknown_region_id"),
                message: String::from(
                    "merge_ocr_into_page_model requires a region_id that exists in the current page model",
                ),
                retryable: false,
                details: Some(serde_json::json!({ "region_id": region_id })),
            });
        };

        region.text = merged_region_text(&region.text, normalized_text);
        region.source = match region.source {
            RegionSource::Dom | RegionSource::Mixed => RegionSource::Mixed,
            RegionSource::Ocr => RegionSource::Ocr,
        };
        if region.bbox.is_none() {
            region.bbox = source_bbox;
        }

        Ok(vec![region.region_id.clone()])
    } else {
        let region_id = next_region_id;
        page.regions.push(PageRegion {
            region_id: region_id.clone(),
            role: RegionRole::Other,
            label: None,
            text: normalized_text.to_string(),
            bbox: source_bbox,
            source: RegionSource::Ocr,
        });
        Ok(vec![region_id])
    }
}

pub(crate) fn extracted_text_metrics(page: &PageModel) -> (usize, usize) {
    page.regions
        .iter()
        .fold((0usize, 0usize), |(chars, regions), region| {
            let trimmed = region.text.trim();
            if trimmed.is_empty() {
                (chars, regions)
            } else {
                (chars + trimmed.chars().count(), regions + 1)
            }
        })
}

fn has_positive_bbox(region: &PageRegion) -> bool {
    matches!(
        region.bbox,
        Some(crate::page_model::Rect {
            width,
            height,
            ..
        }) if width > 0.0 && height > 0.0
    )
}

pub(crate) fn region_first_ocr_target_ids(
    page: &PageModel,
    ocr_settings: &OcrSettings,
) -> Vec<String> {
    if !ocr_settings.prefer_region_ocr {
        return Vec::new();
    }

    page.regions
        .iter()
        .filter(|region| !region.text.trim().is_empty() && has_positive_bbox(region))
        .map(|region| region.region_id.clone())
        .collect()
}

pub(crate) fn merged_region_text(existing_text: &str, ocr_text: &str) -> String {
    let existing_text = existing_text.trim();
    let ocr_text = ocr_text.trim();

    if existing_text.is_empty() {
        return ocr_text.to_string();
    }
    if ocr_text.is_empty() {
        return existing_text.to_string();
    }
    if existing_text == ocr_text {
        return existing_text.to_string();
    }
    if existing_text.contains(ocr_text) {
        return existing_text.to_string();
    }
    if ocr_text.contains(existing_text) {
        return ocr_text.to_string();
    }

    format!("{existing_text}\n\n{ocr_text}")
}
