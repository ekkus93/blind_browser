use super::*;

pub(super) fn execute_capture_screenshot(
    ex: &mut MockExecutor,
    input: CaptureScreenshotInput,
) -> ToolResult<CaptureScreenshotData> {
    ex.last_capture_screenshot_request = Some(input.clone());
    ToolResult::success(
        ToolName::CaptureScreenshot,
        input.request_id,
        CaptureScreenshotData {
            image_id: String::from("image-1"),
            path: String::from("/tmp/image-1.png"),
            bbox: input.bbox,
            width: 640,
            height: 480,
        },
        vec![String::from("captured a screenshot")],
    )
}

pub(super) fn execute_run_ocr(
    ex: &mut MockExecutor,
    input: RunOcrInput,
) -> ToolResult<RunOcrData> {
    ex.last_run_ocr_request = Some(input.clone());
    ToolResult::success(
        ToolName::RunOcr,
        input.request_id,
        RunOcrData {
            image_id: input.image_id,
            extracted_text: String::from("recognized text"),
            text_length: 15,
            confidence: Some(0.82),
            source_bbox: input.bbox,
        },
        vec![String::from("ran OCR on the requested image")],
    )
}

pub(super) fn execute_merge_ocr_into_page_model(
    ex: &mut MockExecutor,
    input: MergeOcrIntoPageModelInput,
) -> ToolResult<MergeOcrIntoPageModelData> {
    ex.last_merge_ocr_request = Some(input.clone());
    ToolResult::success(
        ToolName::MergeOcrIntoPageModel,
        input.request_id,
        MergeOcrIntoPageModelData {
            page_id: input.page_id,
            updated_region_ids: vec![input
                .region_id
                .unwrap_or_else(|| String::from("ocr-region-1"))],
            merged_text_length: input.ocr_text.trim().len(),
        },
        vec![String::from("merged OCR text into the page model")],
    )
}

pub(super) fn execute_read_region(
    ex: &mut MockExecutor,
    input: ReadRegionInput,
) -> ToolResult<ReadRegionData> {
    ex.last_read_region_request = Some(input.clone());
    ToolResult::success(
        ToolName::ReadRegion,
        input.request_id,
        ReadRegionData {
            region_id: input.region_id,
            region_index: 1,
            text_length: 128,
            speech_started: true,
        },
        vec![String::from("started reading the requested region")],
    )
}

pub(super) fn execute_read_next_region(
    ex: &mut MockExecutor,
    input: ReadNextRegionInput,
) -> ToolResult<ReadNextRegionData> {
    ex.last_read_next_region_request = Some(input.clone());
    ToolResult::success(
        ToolName::ReadNextRegion,
        input.request_id,
        ReadNextRegionData {
            cursor: NarrationCursor {
                current_region_id: Some(String::from("region-2")),
                current_index: Some(1),
                total_regions: 3,
            },
            region_id: Some(String::from("region-2")),
            speech_started: true,
            boundary: NarrationBoundary::None,
        },
        vec![String::from("advanced narration to the next region")],
    )
}

pub(super) fn execute_read_previous_region(
    ex: &mut MockExecutor,
    input: ReadPreviousRegionInput,
) -> ToolResult<ReadPreviousRegionData> {
    ex.last_read_previous_region_request = Some(input.clone());
    ToolResult::success(
        ToolName::ReadPreviousRegion,
        input.request_id,
        ReadPreviousRegionData {
            cursor: NarrationCursor {
                current_region_id: Some(String::from("region-1")),
                current_index: Some(0),
                total_regions: 3,
            },
            region_id: Some(String::from("region-1")),
            speech_started: true,
            boundary: NarrationBoundary::None,
        },
        vec![String::from("moved narration to the previous region")],
    )
}

pub(super) fn execute_stop_speaking(
    ex: &mut MockExecutor,
    input: StopSpeakingInput,
) -> ToolResult<StopSpeakingData> {
    ex.last_stop_speaking_request = Some(input.clone());
    ToolResult::success(
        ToolName::StopSpeaking,
        input.request_id,
        StopSpeakingData {
            stopped: true,
            interrupted_region_id: Some(String::from("region-2")),
        },
        vec![String::from("stopped current narration playback")],
    )
}
