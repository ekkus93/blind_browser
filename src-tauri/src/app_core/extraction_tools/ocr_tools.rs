use crate::app_core::element_scoring::region_bbox_by_id;
use crate::app_core::ocr_merge::{extracted_text_metrics, merge_ocr_text_into_page_model};
use crate::app_core::page_model_builder::ocr_runtime_error_to_tool_error;
use crate::commands::{
    MergeOcrIntoPageModelData, MergeOcrIntoPageModelInput, RunOcrData, RunOcrInput, ToolError,
    ToolName, ToolResult,
};
use crate::ocr::OcrSettings;
use crate::page_model::PageModel;

impl super::super::AppCore {
    pub fn execute_run_ocr(&mut self, input: RunOcrInput) -> ToolResult<RunOcrData> {
        let image_id = input.image_id;
        let region_id = input
            .region_id
            .as_deref()
            .map(str::trim)
            .filter(|region_id| !region_id.is_empty())
            .map(ToOwned::to_owned);

        if image_id.is_none() && region_id.is_none() && input.bbox.is_none() {
            return ToolResult::failure(
                ToolName::RunOcr,
                input.request_id,
                ToolError {
                    code: String::from("invalid_ocr_request"),
                    message: String::from(
                        "run_ocr requires at least one source from image_id, region_id, or bbox",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "OCR request was rejected because it did not identify any image or target area.",
                )],
            );
        }

        if let Some(bbox) = input.bbox.as_ref() {
            if bbox.width <= 0.0 || bbox.height <= 0.0 {
                return ToolResult::failure(
                    ToolName::RunOcr,
                    input.request_id,
                    ToolError {
                        code: String::from("invalid_ocr_bbox"),
                        message: String::from("run_ocr bbox requires positive width and height"),
                        retryable: false,
                        details: Some(serde_json::json!({
                            "x": bbox.x,
                            "y": bbox.y,
                            "width": bbox.width,
                            "height": bbox.height,
                        })),
                    },
                    vec![String::from(
                        "OCR request was rejected because the requested bbox was not positive-sized.",
                    )],
                );
            }
        }

        let ocr_bbox = if let Some(region_id) = region_id.as_deref() {
            let regions = match self.readable_regions() {
                Ok(regions) => regions,
                Err(error) => {
                    return ToolResult::failure(
                        ToolName::RunOcr,
                        input.request_id,
                        error,
                        vec![String::from(
                        "Region-targeted OCR requires readable regions in the current page model.",
                    )],
                    )
                }
            };

            match region_bbox_by_id(regions, region_id) {
                Ok(bbox) => Some(bbox),
                Err(error) => {
                    return ToolResult::failure(
                        ToolName::RunOcr,
                        input.request_id,
                        error,
                        vec![String::from(
                            "Region-targeted OCR could not resolve a usable bounding box for the requested region.",
                        )],
                    )
                }
            }
        } else {
            input.bbox.clone()
        };

        let Some(image_id) = image_id else {
            return ToolResult::failure(
                ToolName::RunOcr,
                input.request_id,
                ToolError {
                    code: String::from("missing_ocr_image_id"),
                    message: String::from(
                        "run_ocr currently requires image_id so it can resolve a persisted screenshot",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "OCR needs a persisted screenshot image_id because implicit image selection is not supported.",
                )],
            );
        };

        let image_path = match self.resolve_screenshot_image(&image_id) {
            Ok(path) => path,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::RunOcr,
                    input.request_id,
                    error,
                    vec![String::from(
                        "OCR could not resolve the opaque screenshot handle for the current page state.",
                    )],
                )
            }
        };

        let ocr_result = match self.ocr.run_ocr(&image_path, ocr_bbox.as_ref()) {
            Ok(result) => result,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::RunOcr,
                    input.request_id,
                    ocr_runtime_error_to_tool_error(&error),
                    vec![String::from(
                        "OCR could not extract text from the requested screenshot.",
                    )],
                )
            }
        };

        let mut observations = vec![String::from(
            "Ran deterministic OCR on the requested cached screenshot.",
        )];
        if region_id.is_some() {
            observations.push(String::from(
                "OCR was limited to the requested page region using its stored bounding box.",
            ));
        } else if ocr_bbox.is_some() {
            observations.push(String::from(
                "OCR was limited to the explicitly requested bounding box within the cached image.",
            ));
        } else {
            observations.push(String::from(
                "OCR used the full cached screenshot because no bbox override was provided.",
            ));
        }
        if ocr_result.extracted_text.is_empty() {
            observations.push(String::from(
                "OCR completed successfully but did not extract any readable text.",
            ));
        }
        if ocr_result.truncated {
            observations.push(format!(
                "OCR output was deterministically truncated from {} bytes to the configured output limit.",
                ocr_result.original_text_bytes
            ));
        }

        let original_text_length = ocr_result.original_text_bytes;
        let truncated = ocr_result.truncated;
        let extracted_text = ocr_result.extracted_text;
        let text_length = extracted_text.len();

        ToolResult::success(
            ToolName::RunOcr,
            input.request_id,
            RunOcrData {
                image_id: Some(image_id),
                extracted_text,
                text_length,
                original_text_length,
                truncated,
                confidence: ocr_result.confidence,
                source_bbox: ocr_bbox,
            },
            observations,
        )
    }

    pub fn execute_merge_ocr_into_page_model(
        &mut self,
        input: MergeOcrIntoPageModelInput,
    ) -> ToolResult<MergeOcrIntoPageModelData> {
        let requested_page_id = input.page_id.trim().to_string();
        if requested_page_id.is_empty() {
            return ToolResult::failure(
                ToolName::MergeOcrIntoPageModel,
                input.request_id,
                ToolError {
                    code: String::from("invalid_page_id"),
                    message: String::from("merge_ocr_into_page_model requires a non-empty page_id"),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "OCR merge was rejected because it did not identify a target page.",
                )],
            );
        }

        let Some(active_page_id) = self.state.current_page_id.as_deref() else {
            return ToolResult::failure(
                ToolName::MergeOcrIntoPageModel,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from(
                        "merge_ocr_into_page_model requires an active page in runtime state",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({ "page_id": requested_page_id })),
                },
                vec![String::from(
                    "OCR merge could not run because no active page is loaded in runtime state.",
                )],
            );
        };

        if active_page_id != requested_page_id {
            return ToolResult::failure(
                ToolName::MergeOcrIntoPageModel,
                input.request_id,
                ToolError {
                    code: String::from("page_id_mismatch"),
                    message: String::from(
                        "merge_ocr_into_page_model page_id must match the active runtime page",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({
                        "active_page_id": active_page_id,
                        "page_id": requested_page_id,
                    })),
                },
                vec![String::from(
                    "OCR merge was rejected because it targeted a page that is not currently active.",
                )],
            );
        }

        let normalized_ocr_text = input.ocr_text.trim().to_string();
        if normalized_ocr_text.is_empty() {
            return ToolResult::failure(
                ToolName::MergeOcrIntoPageModel,
                input.request_id,
                ToolError {
                    code: String::from("invalid_ocr_text"),
                    message: String::from("merge_ocr_into_page_model requires non-empty ocr_text"),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "OCR merge was rejected because there was no recognized text to merge.",
                )],
            );
        }

        if let Some(bbox) = input.source_bbox.as_ref() {
            if bbox.width <= 0.0 || bbox.height <= 0.0 {
                return ToolResult::failure(
                    ToolName::MergeOcrIntoPageModel,
                    input.request_id,
                    ToolError {
                        code: String::from("invalid_source_bbox"),
                        message: String::from(
                            "merge_ocr_into_page_model source_bbox requires positive width and height",
                        ),
                        retryable: false,
                        details: Some(serde_json::json!({
                            "x": bbox.x,
                            "y": bbox.y,
                            "width": bbox.width,
                            "height": bbox.height,
                        })),
                    },
                    vec![String::from(
                        "OCR merge was rejected because the supplied source bounding box was invalid.",
                    )],
                );
            }
        }

        let next_region_id = self.next_ocr_region_id(&input.request_id);
        let requested_region_id = input
            .region_id
            .as_deref()
            .map(str::trim)
            .filter(|region_id| !region_id.is_empty())
            .map(ToOwned::to_owned);

        let merge_outcome = {
            let Some(current_page) = self.state.current_page.as_mut() else {
                return ToolResult::failure(
                    ToolName::MergeOcrIntoPageModel,
                    input.request_id,
                    ToolError {
                        code: String::from("missing_page_model"),
                        message: String::from(
                            "merge_ocr_into_page_model requires runtime page data for the active page",
                        ),
                        retryable: false,
                        details: Some(serde_json::json!({ "page_id": requested_page_id })),
                    },
                    vec![String::from(
                        "OCR merge could not update the page because the runtime page model is missing.",
                    )],
                );
            };

            merge_ocr_text_into_page_model(
                current_page,
                requested_region_id.as_deref(),
                &normalized_ocr_text,
                input.source_bbox.clone(),
                next_region_id,
            )
        };

        let updated_region_ids = match merge_outcome {
            Ok(updated_region_ids) => updated_region_ids,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::MergeOcrIntoPageModel,
                    input.request_id,
                    error,
                    vec![String::from(
                        "OCR merge could not apply the recognized text to the runtime page model.",
                    )],
                )
            }
        };

        self.state.mark_page_model_changed();

        let mut observations = vec![String::from(
            "Merged OCR text into the active runtime page model.",
        )];
        if requested_region_id.is_some() {
            observations.push(String::from(
                "OCR text updated an existing page region and marked it as mixed DOM/OCR content.",
            ));
        } else {
            observations.push(String::from(
                "OCR text was added as a new OCR region because no existing target region was supplied.",
            ));
        }

        ToolResult::success(
            ToolName::MergeOcrIntoPageModel,
            input.request_id,
            MergeOcrIntoPageModelData {
                page_id: requested_page_id,
                updated_region_ids,
                merged_text_length: normalized_ocr_text.len(),
            },
            observations,
        )
    }
}

pub(crate) fn should_trigger_extract_page_model_ocr_fallback(
    use_dom_extraction: bool,
    page: &PageModel,
    ocr_settings: &OcrSettings,
) -> bool {
    if !use_dom_extraction || !ocr_settings.trigger_on_no_extractable_text {
        return false;
    }

    let (readable_char_count, readable_region_count) = extracted_text_metrics(page);

    readable_region_count == 0
        || readable_char_count <= ocr_settings.sparse_text_char_threshold as usize
        || readable_region_count < ocr_settings.sparse_text_region_threshold as usize
}
