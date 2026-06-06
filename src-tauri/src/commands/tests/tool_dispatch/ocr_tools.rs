use super::*;

#[test]
fn dispatches_capture_screenshot_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-capture-screenshot"),
        tool_name: ToolName::CaptureScreenshot,
        arguments: serde_json::json!({
            "request_id": "req-capture-screenshot",
            "timeout_ms": 1000,
            "scope": "Viewport",
            "region_id": serde_json::Value::Null,
            "bbox": {
                "x": 10.0,
                "y": 20.0,
                "width": 300.0,
                "height": 120.0
            }
        }),
        purpose: String::from("capture a deterministic screenshot"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_capture_screenshot_request
            .as_ref()
            .map(|input| input.scope),
        Some(ScreenshotScope::Viewport)
    );
    assert_eq!(
        executor
            .last_capture_screenshot_request
            .as_ref()
            .map(|input| input.request_id.as_str()),
        Some("req-capture-screenshot")
    );
    assert_eq!(
        executor
            .last_capture_screenshot_request
            .as_ref()
            .and_then(|input| input.bbox.as_ref())
            .map(|bbox| (bbox.x, bbox.y, bbox.width, bbox.height)),
        Some((10.0, 20.0, 300.0, 120.0))
    );
}

#[test]
fn dispatches_run_ocr_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-run-ocr"),
        tool_name: ToolName::RunOcr,
        arguments: serde_json::json!({
            "request_id": "req-run-ocr",
            "timeout_ms": 1000,
            "image_id": "image-1",
            "region_id": serde_json::Value::Null,
            "bbox": {
                "x": 4.0,
                "y": 8.0,
                "width": 120.0,
                "height": 48.0
            }
        }),
        purpose: String::from("run OCR on a cached screenshot"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_run_ocr_request
            .as_ref()
            .and_then(|input| input.image_id.as_deref()),
        Some("image-1")
    );
    assert_eq!(
        executor
            .last_run_ocr_request
            .as_ref()
            .and_then(|input| input.bbox.as_ref())
            .map(|bbox| (bbox.x, bbox.y, bbox.width, bbox.height)),
        Some((4.0, 8.0, 120.0, 48.0))
    );
}

#[test]
fn dispatches_merge_ocr_into_page_model_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-merge-ocr"),
        tool_name: ToolName::MergeOcrIntoPageModel,
        arguments: serde_json::json!({
            "request_id": "req-merge-ocr",
            "timeout_ms": 1000,
            "page_id": "page-1",
            "region_id": "region-2",
            "ocr_text": "Recovered readable text",
            "source_bbox": {
                "x": 10.0,
                "y": 12.0,
                "width": 200.0,
                "height": 80.0
            }
        }),
        purpose: String::from("merge OCR text into the runtime page model"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_merge_ocr_request
            .as_ref()
            .map(|input| input.page_id.as_str()),
        Some("page-1")
    );
    assert_eq!(
        executor
            .last_merge_ocr_request
            .as_ref()
            .and_then(|input| input.region_id.as_deref()),
        Some("region-2")
    );
}

#[test]
fn dispatches_get_page_snapshot_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-snapshot"),
        tool_name: ToolName::GetPageSnapshot,
        arguments: serde_json::json!({
            "request_id": "req-snapshot",
            "timeout_ms": 1000,
            "include_interactive_elements": true,
            "text_excerpt_max_chars": 120
        }),
        purpose: String::from("read current page snapshot"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_snapshot_request
            .as_ref()
            .map(|input| input.include_interactive_elements),
        Some(true)
    );
    let data = result.data.expect("get_page_snapshot should serialize");
    assert_eq!(
        data.get("page_id"),
        Some(&serde_json::Value::String(String::from("page-1")))
    );
    assert!(data
        .get("interactive_elements")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|elements| !elements.is_empty()));
    assert_eq!(data.get("scroll_y"), Some(&serde_json::json!(120.0)));
    assert_eq!(data.get("viewport_width"), Some(&serde_json::json!(1280.0)));
    assert_eq!(data.get("viewport_height"), Some(&serde_json::json!(720.0)));
    assert_eq!(
        data.get("document_height"),
        Some(&serde_json::json!(2400.0))
    );
}
