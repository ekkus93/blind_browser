use super::*;

#[test]
fn resolve_recent_fill_correction_command_reuses_recent_target_for_replacement() {
    let page = PageModel {
        title: Some(String::from("Example form")),
        url: Some(String::from("https://example.com/form")),
        regions: Vec::new(),
        interactive_elements: vec![InteractiveElement {
            element_id: String::from("input-city"),
            dom_locator: Some(String::from("#city")),
            role: ElementRole::Input,
            tag_name: String::from("input"),
            text: None,
            accessible_name: Some(String::from("City")),
            placeholder: Some(String::from("City")),
            href: None,
            value: Some(String::from("Portland")),
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::new(),
        }],
    };

    let (planner_output, next_context) = resolve_recent_fill_correction_command(
        "put Seattle there instead",
        "req-fill-correction",
        Some("page-1"),
        Some(&page),
        &[String::from("fill_field_by_label")],
        Some(&RecentFieldContext {
            page_id: String::from("page-1"),
            target_description: Some(String::from("city")),
            active_element_id: Some(String::from("input-city")),
            candidate_element_ids: vec![String::from("input-city")],
            pending_text: Some(String::from("Portland")),
            submit_after: false,
        }),
    )
    .expect("follow-up correction should resolve");

    assert_eq!(planner_output.intent.name, IntentName::FillInput);
    assert_eq!(planner_output.status, PlannerStatus::Ready);
    assert_eq!(planner_output.steps[0].tool_name, ToolName::FocusElement);
    assert_eq!(planner_output.steps[1].tool_name, ToolName::TypeIntoElement);
    assert_eq!(
        planner_output.steps[1].arguments.get("text"),
        Some(&serde_json::json!("Seattle"))
    );
    assert_eq!(
        next_context.and_then(|context| context.pending_text),
        Some(String::from("Seattle"))
    );
}

#[test]
fn resolve_recent_fill_correction_command_switches_to_alternate_candidate() {
    let page = PageModel {
        title: Some(String::from("Example form")),
        url: Some(String::from("https://example.com/form")),
        regions: Vec::new(),
        interactive_elements: vec![
            InteractiveElement {
                element_id: String::from("input-email"),
                dom_locator: Some(String::from("#email")),
                role: ElementRole::Input,
                tag_name: String::from("input"),
                text: None,
                accessible_name: Some(String::from("Email")),
                placeholder: Some(String::from("Email")),
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
            InteractiveElement {
                element_id: String::from("input-billing-email"),
                dom_locator: Some(String::from("#billing-email")),
                role: ElementRole::Input,
                tag_name: String::from("input"),
                text: None,
                accessible_name: Some(String::from("Billing email")),
                placeholder: Some(String::from("Billing email")),
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
        ],
    };

    let (planner_output, next_context) = resolve_recent_fill_correction_command(
        "no, the other field",
        "req-fill-other-field",
        Some("page-1"),
        Some(&page),
        &[String::from("fill_and_submit_form")],
        Some(&RecentFieldContext {
            page_id: String::from("page-1"),
            target_description: Some(String::from("email")),
            active_element_id: Some(String::from("input-email")),
            candidate_element_ids: vec![
                String::from("input-email"),
                String::from("input-billing-email"),
            ],
            pending_text: Some(String::from("phil@example.com")),
            submit_after: true,
        }),
    )
    .expect("alternate-field correction should resolve");

    assert_eq!(planner_output.intent.name, IntentName::SubmitForm);
    assert_eq!(planner_output.status, PlannerStatus::NeedsConfirmation);
    assert_eq!(planner_output.steps[0].tool_name, ToolName::ConfirmAction);
    assert_eq!(planner_output.steps[1].tool_name, ToolName::FocusElement);
    assert_eq!(
        planner_output.steps[1].arguments.get("element_id"),
        Some(&serde_json::json!("input-billing-email"))
    );
    assert_eq!(
        next_context.and_then(|context| context.active_element_id),
        Some(String::from("input-billing-email"))
    );
}

#[test]
fn resolve_recent_fill_correction_command_asks_for_target_without_recent_context() {
    let (planner_output, next_context) = resolve_recent_fill_correction_command(
        "put Seattle there instead",
        "req-fill-no-context",
        None,
        None,
        &[String::from("fill_field_by_label")],
        None,
    )
    .expect("correction phrase should still produce a bounded follow-up");

    assert_eq!(planner_output.intent.name, IntentName::FillInput);
    assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
    assert_eq!(
        planner_output.steps[0].arguments.get("status"),
        Some(&serde_json::json!(ReportStatus::NeedsFollowUp))
    );
    assert!(next_context.is_none());
}

#[test]
fn resolve_typeable_element_rejects_non_field_roles() {
    let page = PageModel {
        title: Some(String::from("Example page")),
        url: Some(String::from("https://example.com")),
        regions: Vec::new(),
        interactive_elements: vec![InteractiveElement {
            element_id: String::from("button-1"),
            dom_locator: Some(String::from("#button-1")),
            role: ElementRole::Button,
            tag_name: String::from("button"),
            text: Some(String::from("Continue")),
            accessible_name: Some(String::from("Continue")),
            placeholder: None,
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::new(),
        }],
    };

    let error = resolve_typeable_element(&page, "button-1")
        .expect_err("non-field roles should be rejected");
    assert_eq!(error.code, "element_not_editable");
}

#[test]
fn resolve_direct_submit_form_command_builds_confirmation_gated_submit_plan() {
    let page = PageModel {
        title: Some(String::from("Login")),
        url: Some(String::from("https://example.com/login")),
        regions: Vec::new(),
        interactive_elements: vec![InteractiveElement {
            element_id: String::from("form-login"),
            dom_locator: Some(String::from("#login-form")),
            role: ElementRole::Form,
            tag_name: String::from("form"),
            text: Some(String::from("Sign in")),
            accessible_name: Some(String::from("Login")),
            placeholder: None,
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::new(),
        }],
    };

    let planner_output = resolve_direct_submit_form_command(
        "submit form",
        "req-submit-form",
        Some(&page),
        &[String::from("submit_form")],
    )
    .expect("submit-form command should resolve");

    assert_eq!(planner_output.status, PlannerStatus::NeedsConfirmation);
    assert_eq!(planner_output.intent.name, IntentName::SubmitForm);
    assert_eq!(
        planner_output.selected_skills,
        vec![String::from("submit_form")]
    );
    assert_eq!(planner_output.steps.len(), 2);
    assert_eq!(planner_output.steps[0].tool_name, ToolName::ConfirmAction);
    assert_eq!(
        planner_output.steps[1].tool_name,
        ToolName::SubmitActiveForm
    );
    assert_eq!(
        planner_output.steps[1].arguments.get("form_element_id"),
        Some(&serde_json::json!("form-login"))
    );
    assert!(planner_output.requires_confirmation);
}

#[test]
fn resolve_direct_submit_form_command_reports_ambiguous_forms() {
    let page = PageModel {
        title: Some(String::from("Checkout")),
        url: Some(String::from("https://example.com/checkout")),
        regions: Vec::new(),
        interactive_elements: vec![
            InteractiveElement {
                element_id: String::from("form-shipping"),
                dom_locator: Some(String::from("#shipping-form")),
                role: ElementRole::Form,
                tag_name: String::from("form"),
                text: Some(String::from("Shipping")),
                accessible_name: Some(String::from("Shipping")),
                placeholder: None,
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
            InteractiveElement {
                element_id: String::from("form-billing"),
                dom_locator: Some(String::from("#billing-form")),
                role: ElementRole::Form,
                tag_name: String::from("form"),
                text: Some(String::from("Billing")),
                accessible_name: Some(String::from("Billing")),
                placeholder: None,
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
        ],
    };

    let planner_output = resolve_direct_submit_form_command(
        "submit form",
        "req-submit-form-ambiguous",
        Some(&page),
        &[String::from("submit_form")],
    )
    .expect("submit-form command should resolve");

    assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
    assert_eq!(
        planner_output.steps[0].arguments.get("status"),
        Some(&serde_json::json!(ReportStatus::NeedsFollowUp))
    );
}
