use super::*;

pub(super) fn spawn_openai_models_test_server(
    status_line: &str,
    response_body: &str,
) -> (String, std::thread::JoinHandle<()>) {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    let listener =
        TcpListener::bind("127.0.0.1:0").expect("test server should bind an ephemeral port");
    let address = listener
        .local_addr()
        .expect("test server should expose its bound address");
    let base_url = format!("http://{address}/v1");
    let body = response_body.to_string();
    let status_line = status_line.to_string();

    let handle = std::thread::spawn(move || {
        let (mut stream, _) = listener
            .accept()
            .expect("test server should accept one request");
        let mut buffer = [0_u8; 8192];
        let bytes_read = stream
            .read(&mut buffer)
            .expect("test server should read request bytes");
        let request = String::from_utf8_lossy(&buffer[..bytes_read]);

        assert!(
            request.starts_with("GET /v1/models HTTP/1.1\r\n"),
            "expected GET /v1/models request: {request}"
        );
        assert!(
            request.contains("authorization: Bearer blind-browser-test-key")
                || request.contains("Authorization: Bearer blind-browser-test-key"),
            "expected bearer auth header in request: {request}"
        );
        assert!(
            request.contains("openai-organization: org_test")
                || request.contains("OpenAI-Organization: org_test"),
            "expected organization header in request: {request}"
        );
        assert!(
            request.contains("openai-project: proj_test")
                || request.contains("OpenAI-Project: proj_test"),
            "expected project header in request: {request}"
        );

        let headers = format!(
            "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        stream
            .write_all(headers.as_bytes())
            .expect("test server should write response headers");
        stream
            .write_all(body.as_bytes())
            .expect("test server should write response body");
        stream.flush().expect("test server should flush response");
    });

    (base_url, handle)
}

pub(super) fn fixture_page(interactive_elements: Vec<InteractiveElement>) -> PageModel {
    PageModel {
        title: Some(String::from("Example form")),
        url: Some(String::from("https://example.com/form")),
        regions: Vec::new(),
        interactive_elements,
    }
}

pub(super) fn fixture_page_with_metadata(
    title: &str,
    url: &str,
    interactive_elements: Vec<InteractiveElement>,
) -> PageModel {
    PageModel {
        title: Some(String::from(title)),
        url: Some(String::from(url)),
        regions: Vec::new(),
        interactive_elements,
    }
}

pub(super) fn fixture_field(
    element_id: &str,
    dom_locator: &str,
    accessible_name: &str,
    placeholder: &str,
) -> InteractiveElement {
    InteractiveElement {
        element_id: String::from(element_id),
        dom_locator: Some(String::from(dom_locator)),
        role: ElementRole::Input,
        tag_name: String::from("input"),
        text: None,
        accessible_name: Some(String::from(accessible_name)),
        placeholder: Some(String::from(placeholder)),
        href: None,
        value: None,
        bbox: None,
        visible: true,
        enabled: true,
        attributes: std::collections::BTreeMap::new(),
    }
}

pub(super) fn fixture_form(
    element_id: &str,
    dom_locator: &str,
    accessible_name: &str,
) -> InteractiveElement {
    InteractiveElement {
        element_id: String::from(element_id),
        dom_locator: Some(String::from(dom_locator)),
        role: ElementRole::Form,
        tag_name: String::from("form"),
        text: Some(String::from(accessible_name)),
        accessible_name: Some(String::from(accessible_name)),
        placeholder: None,
        href: None,
        value: None,
        bbox: None,
        visible: true,
        enabled: true,
        attributes: std::collections::BTreeMap::new(),
    }
}

pub(super) fn fixture_problematic_checkout_page() -> PageModel {
    fixture_page_with_metadata(
        "Example Shop | Checkout",
        "https://shop.example.com/checkout",
        vec![
            fixture_form("form-shipping", "#shipping-form", "Shipping address"),
            fixture_field(
                "input-shipping-email",
                "#shipping-email",
                "Shipping email",
                "Email for shipping updates",
            ),
            fixture_field(
                "input-shipping-name",
                "#shipping-name",
                "Full name",
                "Full name",
            ),
            fixture_form("form-billing", "#billing-form", "Billing address"),
            fixture_field(
                "input-billing-email",
                "#billing-email",
                "Billing email",
                "Billing email for receipts",
            ),
            fixture_field(
                "input-card-name",
                "#card-name",
                "Name on card",
                "Name on card",
            ),
        ],
    )
}

pub(super) fn fixture_problematic_landing_page() -> PageModel {
    fixture_page_with_metadata(
        "Example Cloud | Start free trial",
        "https://www.example.com/start",
        vec![
            InteractiveElement {
                element_id: String::from("button-hero-get-started"),
                dom_locator: Some(String::from("#hero-get-started")),
                role: ElementRole::Button,
                tag_name: String::from("button"),
                text: Some(String::from("Get started")),
                accessible_name: Some(String::from("Get started")),
                placeholder: None,
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
            InteractiveElement {
                element_id: String::from("button-footer-get-started"),
                dom_locator: Some(String::from("#footer-get-started")),
                role: ElementRole::Button,
                tag_name: String::from("button"),
                text: Some(String::from("Get started")),
                accessible_name: Some(String::from("Get started")),
                placeholder: None,
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
        ],
    )
}

pub(super) fn fixture_problematic_newsletter_page() -> PageModel {
    fixture_page_with_metadata(
        "Metro news | Sign up for morning headlines",
        "https://news.example.com/newsletters/morning-headlines",
        vec![fixture_field(
            "input-newsletter-email",
            "#newsletter-email",
            "Email",
            "Email address",
        )],
    )
}

pub(super) fn planner_tool_sequence(planner_output: &PlannerOutput) -> Vec<ToolName> {
    planner_output
        .steps
        .iter()
        .map(|step| step.tool_name.clone())
        .collect()
}

#[derive(Clone, Copy)]
pub(super) enum AppCorePlannerFixtureKind {
    FocusField,
    FillField,
    FillAndSubmit,
    FollowUpCorrection,
    SubmitForm,
}

pub(super) struct AppCorePlannerFixture {
    pub(super) name: &'static str,
    pub(super) kind: AppCorePlannerFixtureKind,
    pub(super) transcript: &'static str,
    pub(super) current_page_id: Option<&'static str>,
    pub(super) page: Option<PageModel>,
    pub(super) active_skills: Vec<&'static str>,
    pub(super) recent_context: Option<RecentFieldContext>,
    pub(super) confirmation_threshold: f32,
    pub(super) expected_intent: IntentName,
    pub(super) expected_status: PlannerStatus,
    pub(super) expected_selected_skills: Vec<&'static str>,
    pub(super) expected_tool_sequence: Vec<ToolName>,
    pub(super) expected_focus_element_id: Option<&'static str>,
    pub(super) expected_typed_text: Option<&'static str>,
    pub(super) expected_next_active_element_id: Option<&'static str>,
    pub(super) expected_next_pending_text: Option<&'static str>,
}

pub(super) fn resolve_app_core_planner_fixture(
    fixture: &AppCorePlannerFixture,
) -> (PlannerOutput, Option<RecentFieldContext>) {
    match fixture.kind {
        AppCorePlannerFixtureKind::FocusField => (
            resolve_direct_focus_field_command(
                fixture.transcript,
                fixture.name,
                fixture.page.as_ref(),
                &fixture
                    .active_skills
                    .iter()
                    .map(|skill| String::from(*skill))
                    .collect::<Vec<_>>(),
                fixture.confirmation_threshold,
            )
            .unwrap_or_else(|| panic!("fixture {} should resolve", fixture.name)),
            None,
        ),
        AppCorePlannerFixtureKind::FillField => (
            resolve_direct_fill_field_command(
                fixture.transcript,
                fixture.name,
                fixture.page.as_ref(),
                &fixture
                    .active_skills
                    .iter()
                    .map(|skill| String::from(*skill))
                    .collect::<Vec<_>>(),
                fixture.confirmation_threshold,
            )
            .unwrap_or_else(|| panic!("fixture {} should resolve", fixture.name)),
            None,
        ),
        AppCorePlannerFixtureKind::FillAndSubmit => (
            resolve_direct_fill_and_submit_command(
                fixture.transcript,
                fixture.name,
                fixture.page.as_ref(),
                &fixture
                    .active_skills
                    .iter()
                    .map(|skill| String::from(*skill))
                    .collect::<Vec<_>>(),
                fixture.confirmation_threshold,
            )
            .unwrap_or_else(|| panic!("fixture {} should resolve", fixture.name)),
            None,
        ),
        AppCorePlannerFixtureKind::FollowUpCorrection => resolve_recent_fill_correction_command(
            fixture.transcript,
            fixture.name,
            fixture.current_page_id,
            fixture.page.as_ref(),
            &fixture
                .active_skills
                .iter()
                .map(|skill| String::from(*skill))
                .collect::<Vec<_>>(),
            fixture.recent_context.as_ref(),
        )
        .unwrap_or_else(|| panic!("fixture {} should resolve", fixture.name)),
        AppCorePlannerFixtureKind::SubmitForm => (
            resolve_direct_submit_form_command(
                fixture.transcript,
                fixture.name,
                fixture.page.as_ref(),
                &fixture
                    .active_skills
                    .iter()
                    .map(|skill| String::from(*skill))
                    .collect::<Vec<_>>(),
            )
            .unwrap_or_else(|| panic!("fixture {} should resolve", fixture.name)),
            None,
        ),
    }
}

pub(super) fn assert_app_core_planner_fixture(fixture: AppCorePlannerFixture) {
    let (planner_output, next_context) = resolve_app_core_planner_fixture(&fixture);
    let expected_selected_skills = fixture
        .expected_selected_skills
        .iter()
        .map(|skill| String::from(*skill))
        .collect::<Vec<_>>();

    assert_eq!(
        planner_output.intent.name, fixture.expected_intent,
        "fixture {} resolved unexpected intent",
        fixture.name
    );
    assert_eq!(
        planner_output.status, fixture.expected_status,
        "fixture {} resolved unexpected planner status",
        fixture.name
    );
    assert_eq!(
        planner_output.selected_skills, expected_selected_skills,
        "fixture {} selected unexpected skills",
        fixture.name
    );
    assert_eq!(
        planner_tool_sequence(&planner_output),
        fixture.expected_tool_sequence,
        "fixture {} produced unexpected tool sequence",
        fixture.name
    );

    if let Some(expected_focus_element_id) = fixture.expected_focus_element_id {
        let focus_step = planner_output
            .steps
            .iter()
            .find(|step| step.tool_name == ToolName::FocusElement)
            .unwrap_or_else(|| panic!("fixture {} should include a focus step", fixture.name));
        assert_eq!(
            focus_step.arguments.get("element_id"),
            Some(&serde_json::json!(expected_focus_element_id)),
            "fixture {} focused the wrong element",
            fixture.name
        );
    }

    if let Some(expected_typed_text) = fixture.expected_typed_text {
        let type_step = planner_output
            .steps
            .iter()
            .find(|step| step.tool_name == ToolName::TypeIntoElement)
            .unwrap_or_else(|| panic!("fixture {} should include a type step", fixture.name));
        assert_eq!(
            type_step.arguments.get("text"),
            Some(&serde_json::json!(expected_typed_text)),
            "fixture {} typed unexpected text",
            fixture.name
        );
    }

    assert_eq!(
        next_context
            .as_ref()
            .and_then(|context| context.active_element_id.as_deref()),
        fixture.expected_next_active_element_id,
        "fixture {} produced unexpected next active element",
        fixture.name
    );
    assert_eq!(
        next_context
            .as_ref()
            .and_then(|context| context.pending_text.as_deref()),
        fixture.expected_next_pending_text,
        "fixture {} produced unexpected next pending text",
        fixture.name
    );
}
