use super::invalid_planner_output;
use crate::commands::{CaptureScreenshotInput, MergeOcrIntoPageModelInput, RunOcrInput, ToolError};

pub(super) fn validate_capture_screenshot_input(
    input: &CaptureScreenshotInput,
) -> Result<(), ToolError> {
    let region_id_active = input
        .region_id
        .as_deref()
        .map(str::trim)
        .is_some_and(|region_id| !region_id.is_empty());
    let targeting_modes = usize::from(input.scope.captures_full_page())
        + usize::from(region_id_active)
        + usize::from(input.bbox.is_some());
    if targeting_modes > 1 {
        return Err(invalid_planner_output(
            "capture_screenshot supports at most one targeting mode from scope = FullPage, region_id, or bbox",
            None,
        ));
    }

    if let Some(bbox) = input.bbox.as_ref() {
        if bbox.width <= 0.0 || bbox.height <= 0.0 {
            return Err(invalid_planner_output(
                "capture_screenshot bbox requires positive width and height",
                Some(serde_json::json!({
                    "x": bbox.x,
                    "y": bbox.y,
                    "width": bbox.width,
                    "height": bbox.height,
                })),
            ));
        }
    }

    Ok(())
}

pub(super) fn validate_run_ocr_input(input: &RunOcrInput) -> Result<(), ToolError> {
    let image_id_active = input
        .image_id
        .as_deref()
        .map(str::trim)
        .is_some_and(|image_id| !image_id.is_empty());
    let region_id_active = input
        .region_id
        .as_deref()
        .map(str::trim)
        .is_some_and(|region_id| !region_id.is_empty());

    if !(image_id_active || region_id_active || input.bbox.is_some()) {
        return Err(invalid_planner_output(
            "run_ocr requires at least one source from image_id, region_id, or bbox",
            None,
        ));
    }

    if let Some(bbox) = input.bbox.as_ref() {
        if bbox.width <= 0.0 || bbox.height <= 0.0 {
            return Err(invalid_planner_output(
                "run_ocr bbox requires positive width and height",
                Some(serde_json::json!({
                    "x": bbox.x,
                    "y": bbox.y,
                    "width": bbox.width,
                    "height": bbox.height,
                })),
            ));
        }
    }

    Ok(())
}

pub(super) fn validate_merge_ocr_into_page_model_input(
    input: &MergeOcrIntoPageModelInput,
) -> Result<(), ToolError> {
    if input.page_id.trim().is_empty() {
        return Err(invalid_planner_output(
            "merge_ocr_into_page_model requires a non-empty page_id",
            None,
        ));
    }

    if input.ocr_text.trim().is_empty() {
        return Err(invalid_planner_output(
            "merge_ocr_into_page_model requires non-empty ocr_text",
            None,
        ));
    }

    if let Some(region_id) = input.region_id.as_deref() {
        if region_id.trim().is_empty() {
            return Err(invalid_planner_output(
                "merge_ocr_into_page_model region_id must be non-empty when provided",
                None,
            ));
        }
    }

    if let Some(bbox) = input.source_bbox.as_ref() {
        if bbox.width <= 0.0 || bbox.height <= 0.0 {
            return Err(invalid_planner_output(
                "merge_ocr_into_page_model source_bbox requires positive width and height",
                Some(serde_json::json!({
                    "x": bbox.x,
                    "y": bbox.y,
                    "width": bbox.width,
                    "height": bbox.height,
                })),
            ));
        }
    }

    Ok(())
}
