use super::*;

#[test]
fn filter_interactive_elements_applies_visibility_and_role_filters() {
    let elements = vec![
        InteractiveElement {
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
        },
        InteractiveElement {
            element_id: String::from("link-1"),
            dom_locator: Some(String::from("#link-1")),
            role: ElementRole::Link,
            tag_name: String::from("a"),
            text: Some(String::from("Read more")),
            accessible_name: Some(String::from("Read more")),
            placeholder: None,
            href: Some(String::from("https://example.com/more")),
            value: None,
            bbox: None,
            visible: false,
            enabled: true,
            attributes: std::collections::BTreeMap::new(),
        },
    ];

    let filtered = filter_interactive_elements(&elements, true, Some(&[ElementRole::Button]));

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].element_id, "button-1");
}

#[test]
fn resolve_direct_focus_field_command_focuses_single_matching_field() {
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
                placeholder: Some(String::from("Email address")),
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
            InteractiveElement {
                element_id: String::from("input-password"),
                dom_locator: Some(String::from("#password")),
                role: ElementRole::Input,
                tag_name: String::from("input"),
                text: None,
                accessible_name: Some(String::from("Password")),
                placeholder: Some(String::from("Password")),
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
        ],
    };

    let planner_output = resolve_direct_focus_field_command(
        "focus the email field",
        "req-focus-field",
        Some(&page),
        &[String::from("focus_field")],
        0.9,
    )
    .expect("focus-field command should resolve");

    assert_eq!(planner_output.intent.name, IntentName::FillInput);
    assert_eq!(
        planner_output.selected_skills,
        vec![String::from("focus_field")]
    );
    assert_eq!(planner_output.steps.len(), 1);
    assert_eq!(planner_output.steps[0].tool_name, ToolName::FocusElement);
    assert_eq!(
        planner_output.steps[0].arguments.get("element_id"),
        Some(&serde_json::json!("input-email"))
    );
}

#[test]
fn resolve_direct_focus_field_command_reports_missing_description() {
    let page = PageModel {
        title: Some(String::from("Example form")),
        url: Some(String::from("https://example.com/form")),
        regions: Vec::new(),
        interactive_elements: Vec::new(),
    };

    let planner_output = resolve_direct_focus_field_command(
        "focus field",
        "req-focus-field-missing",
        Some(&page),
        &[String::from("focus_field")],
        0.9,
    )
    .expect("focus-field command should resolve");

    assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
    assert_eq!(
        planner_output.steps[0].arguments.get("status"),
        Some(&serde_json::json!(ReportStatus::NeedsFollowUp))
    );
}

#[test]
fn resolve_direct_focus_field_command_reports_ambiguous_match() {
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
                placeholder: Some(String::from("Email address")),
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
            InteractiveElement {
                element_id: String::from("input-email-confirm"),
                dom_locator: Some(String::from("#email-confirm")),
                role: ElementRole::Input,
                tag_name: String::from("input"),
                text: None,
                accessible_name: Some(String::from("Email confirmation")),
                placeholder: Some(String::from("Confirm email")),
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
        ],
    };

    let planner_output = resolve_direct_focus_field_command(
        "focus the email field",
        "req-focus-field-ambiguous",
        Some(&page),
        &[String::from("focus_field")],
        0.95,
    )
    .expect("focus-field command should resolve");

    assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
    assert_eq!(
        planner_output.steps[0].arguments.get("status"),
        Some(&serde_json::json!(ReportStatus::NeedsFollowUp))
    );
}

#[test]
fn resolve_direct_fill_field_command_focuses_then_types_into_matching_field() {
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
                placeholder: Some(String::from("Email address")),
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
            InteractiveElement {
                element_id: String::from("input-password"),
                dom_locator: Some(String::from("#password")),
                role: ElementRole::Input,
                tag_name: String::from("input"),
                text: None,
                accessible_name: Some(String::from("Password")),
                placeholder: Some(String::from("Password")),
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
        ],
    };

    let planner_output = resolve_direct_fill_field_command(
        "fill the email field with phil@example.com",
        "req-fill-field",
        Some(&page),
        &[String::from("fill_field_by_label")],
        0.9,
    )
    .expect("fill-field command should resolve");

    assert_eq!(planner_output.intent.name, IntentName::FillInput);
    assert_eq!(
        planner_output.selected_skills,
        vec![String::from("fill_field_by_label")]
    );
    assert_eq!(planner_output.steps.len(), 2);
    assert_eq!(planner_output.steps[0].tool_name, ToolName::FocusElement);
    assert_eq!(planner_output.steps[1].tool_name, ToolName::TypeIntoElement);
    assert_eq!(
        planner_output.steps[1].arguments.get("text"),
        Some(&serde_json::json!("phil@example.com"))
    );
}

#[test]
fn resolve_direct_fill_field_command_reports_missing_value() {
    let page = PageModel {
        title: Some(String::from("Example form")),
        url: Some(String::from("https://example.com/form")),
        regions: Vec::new(),
        interactive_elements: Vec::new(),
    };

    let planner_output = resolve_direct_fill_field_command(
        "fill the email field",
        "req-fill-field-missing-value",
        Some(&page),
        &[String::from("fill_field_by_label")],
        0.9,
    )
    .expect("fill-field command should resolve");

    assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
    assert_eq!(
        planner_output.steps[0].arguments.get("status"),
        Some(&serde_json::json!(ReportStatus::NeedsFollowUp))
    );
}

#[test]
fn resolve_direct_fill_and_submit_command_builds_confirmation_gated_plan() {
    let page = PageModel {
        title: Some(String::from("Example form")),
        url: Some(String::from("https://example.com/form")),
        regions: Vec::new(),
        interactive_elements: vec![InteractiveElement {
            element_id: String::from("input-email"),
            dom_locator: Some(String::from("#email")),
            role: ElementRole::Input,
            tag_name: String::from("input"),
            text: None,
            accessible_name: Some(String::from("Email")),
            placeholder: Some(String::from("Email address")),
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::new(),
        }],
    };

    let planner_output = resolve_direct_fill_and_submit_command(
        "fill the email field with phil@example.com and then submit",
        "req-fill-submit",
        Some(&page),
        &[String::from("fill_and_submit_form")],
        0.9,
    )
    .expect("fill-and-submit command should resolve");

    assert_eq!(planner_output.intent.name, IntentName::SubmitForm);
    assert_eq!(planner_output.status, PlannerStatus::NeedsConfirmation);
    assert_eq!(
        planner_output.selected_skills,
        vec![String::from("fill_and_submit_form")]
    );
    assert_eq!(planner_output.steps.len(), 4);
    assert_eq!(planner_output.steps[0].tool_name, ToolName::ConfirmAction);
    assert_eq!(planner_output.steps[1].tool_name, ToolName::FocusElement);
    assert_eq!(planner_output.steps[2].tool_name, ToolName::TypeIntoElement);
    assert_eq!(
        planner_output.steps[3].tool_name,
        ToolName::SubmitActiveForm
    );
    assert_eq!(
        planner_output.steps[2].arguments.get("text"),
        Some(&serde_json::json!("phil@example.com"))
    );
    assert_eq!(
        planner_output.steps[3].arguments.get("form_element_id"),
        Some(&serde_json::Value::Null)
    );
    assert!(planner_output.requires_confirmation);
}

#[test]
fn resolve_direct_fill_and_submit_command_reports_missing_value() {
    let page = PageModel {
        title: Some(String::from("Example form")),
        url: Some(String::from("https://example.com/form")),
        regions: Vec::new(),
        interactive_elements: Vec::new(),
    };

    let planner_output = resolve_direct_fill_and_submit_command(
        "fill the email field and submit",
        "req-fill-submit-missing-value",
        Some(&page),
        &[String::from("fill_and_submit_form")],
        0.9,
    )
    .expect("fill-and-submit command should resolve");

    assert_eq!(planner_output.intent.name, IntentName::SubmitForm);
    assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
    assert_eq!(
        planner_output.steps[0].arguments.get("status"),
        Some(&serde_json::json!(ReportStatus::NeedsFollowUp))
    );
}
