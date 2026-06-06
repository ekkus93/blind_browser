use super::*;

pub fn sample_planned_step(tool_name: ToolName) -> PlannedStep {
    match tool_name {
        ToolName::OpenUrl => PlannedStep {
            step_id: String::from("step-open-url"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-open-url",
                "timeout_ms": 1000,
                "url": "https://example.com/article",
                "wait_for_load_state": "NetworkIdle"
            }),
            purpose: String::from("navigate to a page"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::GoBack => PlannedStep {
            step_id: String::from("step-go-back"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-go-back",
                "timeout_ms": 1000,
                "steps": 2,
                "wait_for_load_state": "Load"
            }),
            purpose: String::from("go back in history"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::GoForward => PlannedStep {
            step_id: String::from("step-go-forward"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-go-forward",
                "timeout_ms": 1000,
                "steps": 1,
                "wait_for_load_state": "NetworkIdle"
            }),
            purpose: String::from("go forward in history"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::ReloadPage => PlannedStep {
            step_id: String::from("step-reload"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-reload",
                "timeout_ms": 1000,
                "mode": "Hard",
                "wait_for_load_state": "Load"
            }),
            purpose: String::from("reload the current page"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::GetHtml => PlannedStep {
            step_id: String::from("step-get-html"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-get-html",
                "timeout_ms": 1000
            }),
            purpose: String::from("read current page HTML"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::EvalJs => PlannedStep {
            step_id: String::from("step-eval-js"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-eval-js",
                "timeout_ms": 1000,
                "expression": "({ headline: document.title, regionCount: document.querySelectorAll('main, article, section').length })"
            }),
            purpose: String::from("evaluate a bounded JavaScript expression"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::ScrollPage => PlannedStep {
            step_id: String::from("step-scroll"),
            tool_name,
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
        },
        ToolName::CaptureScreenshot => PlannedStep {
            step_id: String::from("step-capture-screenshot"),
            tool_name,
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
        },
        ToolName::SetBrowserVisibility => PlannedStep {
            step_id: String::from("step-set-browser-visibility"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-visibility",
                "timeout_ms": 1000,
                "mode": "Headless"
            }),
            purpose: String::from("toggle browser visibility"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::GetPageSnapshot => PlannedStep {
            step_id: String::from("step-snapshot"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-snapshot",
                "timeout_ms": 1000,
                "include_interactive_elements": true,
                "text_excerpt_max_chars": 120
            }),
            purpose: String::from("read current page snapshot"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::ExtractPageModel => PlannedStep {
            step_id: String::from("step-extract"),
            tool_name,
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
        },
        ToolName::ListInteractiveElements => PlannedStep {
            step_id: String::from("step-list"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-list",
                "timeout_ms": 1000,
                "visibility_filter": "VisibleOnly",
                "roles": ["Button"]
            }),
            purpose: String::from("list visible buttons"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::FindElement => PlannedStep {
            step_id: String::from("step-find"),
            tool_name,
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
        },
        ToolName::ClickElement => PlannedStep {
            step_id: String::from("step-click"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-click",
                "timeout_ms": 1000,
                "element_id": "button-1",
                "click_mode": "Single"
            }),
            purpose: String::from("click the resolved button"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::FocusElement => PlannedStep {
            step_id: String::from("step-focus"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-focus",
                "timeout_ms": 1000,
                "element_id": "input-1"
            }),
            purpose: String::from("focus the resolved field"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::TypeIntoElement => PlannedStep {
            step_id: String::from("step-type"),
            tool_name,
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
        },
        ToolName::SubmitActiveForm => PlannedStep {
            step_id: String::from("step-submit"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-submit",
                "timeout_ms": 1000,
                "form_element_id": "form-login"
            }),
            purpose: String::from("submit the active form"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::ReadRegion => PlannedStep {
            step_id: String::from("step-read-region"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-read-region",
                "timeout_ms": 1000,
                "region_id": "region-2",
                "interruption_mode": "Interrupt"
            }),
            purpose: String::from("read a specific region"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::ReadNextRegion => PlannedStep {
            step_id: String::from("step-read-next"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-read-next",
                "timeout_ms": 1000,
                "interruption_mode": "Queue"
            }),
            purpose: String::from("read the next region"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::ReadPreviousRegion => PlannedStep {
            step_id: String::from("step-read-previous"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-read-previous",
                "timeout_ms": 1000,
                "interruption_mode": "Interrupt"
            }),
            purpose: String::from("read the previous region"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::StopSpeaking => PlannedStep {
            step_id: String::from("step-stop-speaking"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-stop-speaking",
                "timeout_ms": 1000
            }),
            purpose: String::from("stop current narration"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::StartListening => PlannedStep {
            step_id: String::from("step-start-listening"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-start-listening",
                "timeout_ms": 1500
            }),
            purpose: String::from("start listening"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::StopListening => PlannedStep {
            step_id: String::from("step-stop-listening"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-stop-listening"
            }),
            purpose: String::from("stop listening"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::TranscribeCommand => PlannedStep {
            step_id: String::from("step-transcribe-command"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-transcribe-command",
                "timeout_ms": 2000,
                "max_duration_ms": 3000,
                "stop_mode": "AutoStop"
            }),
            purpose: String::from("transcribe a command"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::SetTtsVoice => PlannedStep {
            step_id: String::from("step-set-tts-voice"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-set-tts-voice",
                "timeout_ms": 1000,
                "voice": "Bruno"
            }),
            purpose: String::from("change the TTS voice"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::SetPlaybackVolume => PlannedStep {
            step_id: String::from("step-set-playback-volume"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-set-playback-volume",
                "timeout_ms": 1000,
                "volume": 0.4
            }),
            purpose: String::from("update volume"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::SetPlaybackSpeed => PlannedStep {
            step_id: String::from("step-set-playback-speed"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-set-playback-speed",
                "timeout_ms": 1000,
                "speed": 1.2
            }),
            purpose: String::from("update speed"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::RunOcr => PlannedStep {
            step_id: String::from("step-run-ocr"),
            tool_name,
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
        },
        ToolName::MergeOcrIntoPageModel => PlannedStep {
            step_id: String::from("step-merge-ocr"),
            tool_name,
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
        },
        ToolName::GetAgentState => PlannedStep {
            step_id: String::from("step-get-agent-state"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-agent-state",
                "timeout_ms": 1000,
                "include_last_transcript": false
            }),
            purpose: String::from("read agent state"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::GetRuntimeStatus => PlannedStep {
            step_id: String::from("step-get-runtime-status"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-runtime-status",
                "timeout_ms": 1000,
                "include_provider_modes": true
            }),
            purpose: String::from("read runtime status"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::ConfirmAction => PlannedStep {
            step_id: String::from("step-confirm-action"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-confirm-action",
                "timeout_ms": 1000,
                "prompt_text": "Do you want me to continue?",
                "reason": "The next step may submit data."
            }),
            purpose: String::from("request confirmation"),
            on_success: StepTransition::RequestConfirmation,
            on_failure: StepTransition::Replan,
        },
        ToolName::ReportResult => PlannedStep {
            step_id: String::from("step-report-result"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-report-result",
                "timeout_ms": 1000,
                "status": "Success",
                "summary": "Opened the requested page.",
                "next_recommended_action": null,
                "user_message": "The page is ready."
            }),
            purpose: String::from("report completion"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
    }
}

pub fn sample_planned_steps_for_registered_tools() -> Vec<PlannedStep> {
    registered_tools()
        .into_iter()
        .map(|tool| sample_planned_step(tool.name))
        .collect()
}
