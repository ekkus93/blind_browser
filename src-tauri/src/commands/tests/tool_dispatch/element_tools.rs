use super::*;

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
