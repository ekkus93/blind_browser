use super::*;

#[test]
fn dispatches_set_playback_volume_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-1"),
        tool_name: ToolName::SetPlaybackVolume,
        arguments: serde_json::json!({
            "request_id": "req-1",
            "timeout_ms": 1000,
            "volume": 0.4
        }),
        purpose: String::from("update volume"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(executor.last_volume, Some(0.4));
    assert_eq!(result.tool_name, ToolName::SetPlaybackVolume);
    assert_eq!(result.request_id, "req-1");
    let data = result
        .data
        .expect("serialized tool result data should exist");
    assert_eq!(data.get("muted"), Some(&serde_json::Value::Bool(false)));
    assert_eq!(data.get("changed"), Some(&serde_json::Value::Bool(true)));
    let playback_volume = data
        .get("playback_volume")
        .and_then(serde_json::Value::as_f64)
        .expect("playback_volume should be serialized as a number");
    assert!((playback_volume - 0.4).abs() < 0.000_001);
}

#[test]
fn dispatches_open_url_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-open-url"),
        tool_name: ToolName::OpenUrl,
        arguments: serde_json::json!({
            "request_id": "req-open-url",
            "timeout_ms": 1000,
            "url": "https://example.com/article",
            "wait_for_load_state": "NetworkIdle"
        }),
        purpose: String::from("navigate to a page"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor.last_open_url.as_deref(),
        Some("https://example.com/article")
    );
    let data = result.data.expect("open_url should serialize");
    assert_eq!(
        data.get("final_url"),
        Some(&serde_json::Value::String(String::from(
            "https://example.com/article"
        )))
    );
    assert_eq!(
        data.get("load_state"),
        Some(&serde_json::Value::String(String::from("NetworkIdle")))
    );
}

#[test]
fn dispatches_go_back_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-go-back"),
        tool_name: ToolName::GoBack,
        arguments: serde_json::json!({
            "request_id": "req-go-back",
            "timeout_ms": 1000,
            "steps": 2,
            "wait_for_load_state": "Load"
        }),
        purpose: String::from("go back in history"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_go_back_request
            .as_ref()
            .and_then(|input| input.steps),
        Some(2)
    );
}

#[test]
fn dispatches_go_forward_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-go-forward"),
        tool_name: ToolName::GoForward,
        arguments: serde_json::json!({
            "request_id": "req-go-forward",
            "timeout_ms": 1000,
            "steps": 1,
            "wait_for_load_state": "NetworkIdle"
        }),
        purpose: String::from("go forward in history"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_go_forward_request
            .as_ref()
            .and_then(|input| input.steps),
        Some(1)
    );
}

#[test]
fn dispatches_reload_page_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-reload"),
        tool_name: ToolName::ReloadPage,
        arguments: serde_json::json!({
            "request_id": "req-reload",
            "timeout_ms": 1000,
            "mode": "Hard",
            "wait_for_load_state": "Load"
        }),
        purpose: String::from("reload the current page"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_reload_request
            .as_ref()
            .map(|input| input.mode),
        Some(ReloadMode::Hard)
    );
}

#[test]
fn dispatches_get_html_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-get-html"),
        tool_name: ToolName::GetHtml,
        arguments: serde_json::json!({
            "request_id": "req-get-html",
            "timeout_ms": 1000
        }),
        purpose: String::from("read current page HTML"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_get_html_request
            .as_ref()
            .map(|input| input.request_id.as_str()),
        Some("req-get-html")
    );
    let data = result.data.expect("get_html should serialize");
    assert_eq!(
        data.get("page_id"),
        Some(&serde_json::Value::String(String::from("page-1")))
    );
    assert_eq!(
        data.get("html_length").and_then(serde_json::Value::as_u64),
        Some(54)
    );
    assert!(data
        .get("html")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|html| html.contains("<main>Example article</main>")));
}

#[test]
fn dispatches_eval_js_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-eval-js"),
        tool_name: ToolName::EvalJs,
        arguments: serde_json::json!({
            "request_id": "req-eval-js",
            "timeout_ms": 1000,
            "expression": "({ headline: document.title, regionCount: document.querySelectorAll('main, article, section').length })"
        }),
        purpose: String::from("evaluate a bounded JavaScript expression"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_eval_js_request
            .as_ref()
            .map(|input| input.request_id.as_str()),
        Some("req-eval-js")
    );
    let data = result.data.expect("eval_js should serialize");
    assert_eq!(
        data.get("page_id"),
        Some(&serde_json::Value::String(String::from("page-1")))
    );
    assert_eq!(
        data.get("result")
            .and_then(|value| value.get("regionCount"))
            .and_then(serde_json::Value::as_u64),
        Some(3)
    );
}

#[test]
fn dispatches_scroll_page_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-scroll"),
        tool_name: ToolName::ScrollPage,
        arguments: serde_json::json!({
            "request_id": "req-scroll",
            "timeout_ms": 1000,
            "direction": "Down",
            "amount_px": 480.0,
            "target": null
        }),
        purpose: String::from("scroll the page"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_scroll_request
            .as_ref()
            .and_then(|input| input.amount_px),
        Some(480.0)
    );
}

#[test]
fn dispatches_read_region_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-read-region"),
        tool_name: ToolName::ReadRegion,
        arguments: serde_json::json!({
            "request_id": "req-read-region",
            "timeout_ms": 1000,
            "region_id": "region-2",
            "interruption_mode": "Interrupt"
        }),
        purpose: String::from("read a specific region"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_read_region_request
            .as_ref()
            .map(|input| input.region_id.as_str()),
        Some("region-2")
    );
}

#[test]
fn dispatches_read_next_region_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-read-next"),
        tool_name: ToolName::ReadNextRegion,
        arguments: serde_json::json!({
            "request_id": "req-read-next",
            "timeout_ms": 1000,
            "interruption_mode": "Queue"
        }),
        purpose: String::from("read the next region"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_read_next_region_request
            .as_ref()
            .map(|input| input.interruption_mode),
        Some(NarrationInterruptionMode::Queue)
    );
}

#[test]
fn dispatches_read_previous_region_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-read-previous"),
        tool_name: ToolName::ReadPreviousRegion,
        arguments: serde_json::json!({
            "request_id": "req-read-previous",
            "timeout_ms": 1000,
            "interruption_mode": "Interrupt"
        }),
        purpose: String::from("read the previous region"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_read_previous_region_request
            .as_ref()
            .map(|input| input.interruption_mode),
        Some(NarrationInterruptionMode::Interrupt)
    );
}

#[test]
fn dispatches_stop_speaking_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-stop-speaking"),
        tool_name: ToolName::StopSpeaking,
        arguments: serde_json::json!({
            "request_id": "req-stop-speaking",
            "timeout_ms": 1000
        }),
        purpose: String::from("stop current narration"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_stop_speaking_request
            .as_ref()
            .map(|input| input.request_id.as_str()),
        Some("req-stop-speaking")
    );
}

#[test]
fn dispatches_start_listening_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-start-listening"),
        tool_name: ToolName::StartListening,
        arguments: serde_json::json!({
            "request_id": "req-start-listening",
            "timeout_ms": 1500
        }),
        purpose: String::from("start listening"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_start_listening_request
            .as_ref()
            .map(|input| input.request_id.as_str()),
        Some("req-start-listening")
    );
}

#[test]
fn dispatches_stop_listening_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-stop-listening"),
        tool_name: ToolName::StopListening,
        arguments: serde_json::json!({
            "request_id": "req-stop-listening"
        }),
        purpose: String::from("stop listening"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_stop_listening_request
            .as_ref()
            .map(|input| input.request_id.as_str()),
        Some("req-stop-listening")
    );
}

#[test]
fn dispatches_transcribe_command_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-transcribe-command"),
        tool_name: ToolName::TranscribeCommand,
        arguments: serde_json::json!({
            "request_id": "req-transcribe-command",
            "timeout_ms": 2000,
            "max_duration_ms": 3000,
            "stop_mode": "AutoStop"
        }),
        purpose: String::from("transcribe a command"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_transcribe_command_request
            .as_ref()
            .map(|input| input.request_id.as_str()),
        Some("req-transcribe-command")
    );
    assert_eq!(
        executor
            .last_transcribe_command_request
            .as_ref()
            .and_then(|input| input.max_duration_ms),
        Some(3000)
    );
}

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

#[test]
fn dispatches_list_interactive_elements_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-list"),
        tool_name: ToolName::ListInteractiveElements,
        arguments: serde_json::json!({
            "request_id": "req-list",
            "timeout_ms": 1000,
            "visibility_filter": "VisibleOnly",
            "roles": ["Button"]
        }),
        purpose: String::from("list visible buttons"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_list_request
            .as_ref()
            .map(|input| input.visibility_filter),
        Some(ElementVisibilityFilter::VisibleOnly)
    );
    let data = result
        .data
        .expect("list_interactive_elements should serialize");
    assert_eq!(
        data.get("page_id"),
        Some(&serde_json::Value::String(String::from("page-1")))
    );
    assert_eq!(
        data.get("visible_count"),
        Some(&serde_json::Value::Number(serde_json::Number::from(1)))
    );
    assert!(data
        .get("elements")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|elements| elements.len() == 1));
}

#[test]
fn dispatches_find_element_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-find"),
        tool_name: ToolName::FindElement,
        arguments: serde_json::json!({
            "request_id": "req-find",
            "timeout_ms": 1000,
            "description": "continue",
            "text": null,
            "role": "Button",
            "color_hint": null,
            "nearby_text": null,
            "selector_hint": null,
            "visibility_filter": "VisibleOnly",
            "max_candidates": 3
        }),
        purpose: String::from("find the continue button"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_find_request
            .as_ref()
            .and_then(|input| input.role.as_ref()),
        Some(&crate::page_model::ElementRole::Button)
    );
    let data = result.data.expect("find_element should serialize");
    assert_eq!(
        data.get("chosen_element_id"),
        Some(&serde_json::Value::String(String::from("button-1")))
    );
    assert_eq!(
        data.get("requires_confirmation"),
        Some(&serde_json::Value::Bool(false))
    );
}

#[test]
fn dispatches_click_element_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-click"),
        tool_name: ToolName::ClickElement,
        arguments: serde_json::json!({
            "request_id": "req-click",
            "timeout_ms": 1000,
            "element_id": "button-1",
            "click_mode": "Single"
        }),
        purpose: String::from("click the resolved button"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_click_request
            .as_ref()
            .map(|input| input.element_id.as_str()),
        Some("button-1")
    );
    let data = result.data.expect("click_element should serialize");
    assert_eq!(
        data.get("element_id"),
        Some(&serde_json::Value::String(String::from("button-1")))
    );
    assert_eq!(
        data.get("action_performed"),
        Some(&serde_json::Value::Bool(true))
    );
    assert_eq!(
        data.get("page_changed"),
        Some(&serde_json::Value::Bool(false))
    );
}

#[test]
fn dispatches_focus_element_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-focus"),
        tool_name: ToolName::FocusElement,
        arguments: serde_json::json!({
            "request_id": "req-focus",
            "timeout_ms": 1000,
            "element_id": "input-1"
        }),
        purpose: String::from("focus the resolved field"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_focus_request
            .as_ref()
            .map(|input| input.element_id.as_str()),
        Some("input-1")
    );
    let data = result.data.expect("focus_element should serialize");
    assert_eq!(data.get("focused"), Some(&serde_json::Value::Bool(true)));
}

#[test]
fn dispatches_type_into_element_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-type"),
        tool_name: ToolName::TypeIntoElement,
        arguments: serde_json::json!({
            "request_id": "req-type",
            "timeout_ms": 1000,
            "element_id": "input-1",
            "text": "phil@example.com",
            "text_entry_mode": "Replace",
            "submit_mode": "KeepEditing"
        }),
        purpose: String::from("type into the resolved field"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_type_request
            .as_ref()
            .map(|input| input.text.as_str()),
        Some("phil@example.com")
    );
    let data = result.data.expect("type_into_element should serialize");
    assert_eq!(
        data.get("accepted_input"),
        Some(&serde_json::Value::Bool(true))
    );
}

#[test]
fn dispatches_submit_active_form_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-submit"),
        tool_name: ToolName::SubmitActiveForm,
        arguments: serde_json::json!({
            "request_id": "req-submit",
            "timeout_ms": 1000,
            "form_element_id": "form-login"
        }),
        purpose: String::from("submit the active form"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_submit_request
            .as_ref()
            .and_then(|input| input.form_element_id.as_deref()),
        Some("form-login")
    );
    let data = result.data.expect("submit_active_form should serialize");
    assert_eq!(data.get("submitted"), Some(&serde_json::Value::Bool(true)));
    assert_eq!(
        data.get("page_changed"),
        Some(&serde_json::Value::Bool(true))
    );
}

#[test]
fn dispatches_extract_page_model_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-extract"),
        tool_name: ToolName::ExtractPageModel,
        arguments: serde_json::json!({
            "request_id": "req-extract",
            "timeout_ms": 1000,
            "use_dom_extraction": true,
            "include_headings": true,
            "include_links": false
        }),
        purpose: String::from("extract a page model"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_extract_request
            .as_ref()
            .map(|input| input.include_links),
        Some(false)
    );
    let data = result.data.expect("extract_page_model should serialize");
    assert_eq!(
        data.get("extraction_source"),
        Some(&serde_json::Value::String(String::from("DomFallback")))
    );
    assert!(data
        .get("page_model")
        .and_then(|model| model.get("interactive_elements"))
        .and_then(serde_json::Value::as_array)
        .is_some_and(|elements| elements.is_empty()));
}

#[test]
fn rejects_invalid_tool_arguments_before_dispatch() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-2"),
        tool_name: ToolName::SetPlaybackSpeed,
        arguments: serde_json::json!({
            "request_id": "req-2",
            "timeout_ms": 1000,
            "speed": "fast"
        }),
        purpose: String::from("update speed"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(!result.ok);
    assert_eq!(result.tool_name, ToolName::SetPlaybackSpeed);
    assert_eq!(result.request_id, "req-2");
    assert_eq!(
        result.error.expect("error should be present").code,
        "invalid_tool_arguments"
    );
    assert_eq!(executor.last_speed, None);
}

#[test]
fn validate_planned_step_arguments_reports_schema_mismatch_details() {
    let step = PlannedStep {
        step_id: String::from("step-speed"),
        tool_name: ToolName::SetPlaybackSpeed,
        arguments: serde_json::json!({
            "request_id": "req-speed",
            "speed": "fast"
        }),
        purpose: String::from("set playback speed"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let error = validate_planned_step_arguments(&step)
        .expect_err("validation should reject malformed step arguments");

    assert_eq!(error.code, "invalid_planner_output");
    assert!(error.message.contains("expected schema"));
    assert_eq!(
        error
            .details
            .as_ref()
            .and_then(|details| details.get("step_id")),
        Some(&serde_json::json!("step-speed"))
    );
    assert_eq!(
        error
            .details
            .as_ref()
            .and_then(|details| details.get("tool_name")),
        Some(&serde_json::json!("SetPlaybackSpeed"))
    );
}

#[test]
fn dispatches_set_browser_visibility_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-3"),
        tool_name: ToolName::SetBrowserVisibility,
        arguments: serde_json::json!({
            "request_id": "req-3",
            "timeout_ms": 1000,
            "mode": "Headless"
        }),
        purpose: String::from("toggle browser visibility"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor.last_visibility,
        Some(BrowserVisibilityMode::Headless)
    );
    assert_eq!(result.tool_name, ToolName::SetBrowserVisibility);
}

#[test]
fn dispatches_get_runtime_status_with_provider_modes() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-4"),
        tool_name: ToolName::GetRuntimeStatus,
        arguments: serde_json::json!({
            "request_id": "req-4",
            "timeout_ms": 1000,
            "include_provider_modes": true
        }),
        purpose: String::from("read runtime status"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    let data = result.data.expect("runtime status should serialize");
    assert!(data.get("provider_modes").is_some());
}

