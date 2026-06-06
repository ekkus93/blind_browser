use super::*;

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
