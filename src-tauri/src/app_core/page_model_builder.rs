use crate::commands::{ExtractPageModelData, ExtractPageModelInput, ToolError, ToolName, ToolResult};
use crate::ocr::OcrRuntimeError;
use crate::page_model::{ExtractionSource, PageModel, RegionSource};

pub(crate) fn ocr_runtime_error_to_tool_error(error: &OcrRuntimeError) -> ToolError {
    let code = match error {
        OcrRuntimeError::FeatureUnavailable => "ocr_backend_unavailable",
        OcrRuntimeError::EngineInitFailed { .. } => "ocr_engine_init_failed",
        OcrRuntimeError::ImageLoadFailed { .. } => "ocr_image_load_failed",
        OcrRuntimeError::InvalidBbox => "invalid_ocr_bbox",
        OcrRuntimeError::TextExtractionFailed { .. } => "ocr_text_extraction_failed",
    };

    ToolError {
        code: String::from(code),
        message: error.to_string(),
        retryable: matches!(
            error,
            OcrRuntimeError::EngineInitFailed { .. } | OcrRuntimeError::TextExtractionFailed { .. }
        ),
        details: None,
    }
}

pub(crate) fn extract_page_model_internal_failure(
    request_id: String,
    message: String,
    observations: Vec<String>,
) -> ToolResult<ExtractPageModelData> {
    ToolResult::failure(
        ToolName::ExtractPageModel,
        request_id,
        ToolError {
            code: String::from("extract_page_model_internal_error"),
            message,
            retryable: false,
            details: None,
        },
        observations,
    )
}

pub(crate) fn nested_tool_failure_as_extract_page_model<T>(
    request_id: String,
    mut observations: Vec<String>,
    nested_result: ToolResult<T>,
    failure_observation: String,
) -> ToolResult<ExtractPageModelData> {
    observations.extend(nested_result.observations);
    observations.push(failure_observation);

    let error = nested_result.error.unwrap_or(ToolError {
        code: String::from("extract_page_model_internal_error"),
        message: String::from("nested OCR fallback tool failed without returning a ToolError"),
        retryable: false,
        details: None,
    });

    ToolResult::failure(ToolName::ExtractPageModel, request_id, error, observations)
}

pub(crate) fn build_visible_text_excerpt(page: &PageModel, max_chars: Option<usize>) -> String {
    let joined_text = page
        .regions
        .iter()
        .map(|region| region.text.trim())
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");

    match max_chars {
        Some(limit) => joined_text.chars().take(limit).collect(),
        None => joined_text,
    }
}

pub(crate) fn build_extracted_page_model(page: &PageModel, input: &ExtractPageModelInput) -> PageModel {
    use crate::page_model::ElementRole;

    let interactive_elements = if input.include_links {
        page.interactive_elements.clone()
    } else {
        page.interactive_elements
            .iter()
            .filter(|element| element.role != ElementRole::Link)
            .cloned()
            .collect()
    };

    PageModel {
        title: page.title.clone(),
        url: page.url.clone(),
        regions: page.regions.clone(),
        interactive_elements,
    }
}

pub(crate) fn infer_extraction_source(
    page: &PageModel,
    use_dom_extraction: bool,
    used_dom_smoothie: bool,
) -> ExtractionSource {
    let has_ocr = page
        .regions
        .iter()
        .any(|region| matches!(region.source, RegionSource::Ocr | RegionSource::Mixed));
    let has_dom_like = page
        .regions
        .iter()
        .any(|region| matches!(region.source, RegionSource::Dom | RegionSource::Mixed));

    if has_ocr && has_dom_like {
        ExtractionSource::Merged
    } else if has_ocr {
        ExtractionSource::Ocr
    } else if use_dom_extraction && used_dom_smoothie {
        ExtractionSource::DomSmoothie
    } else {
        ExtractionSource::DomFallback
    }
}
