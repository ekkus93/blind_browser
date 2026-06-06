use super::*;

#[test]
fn app_core_form_regression_fixtures_cover_ambiguous_fill_submit_and_follow_up_cases() {
    let fixtures = vec![
        AppCorePlannerFixture {
            name: "ambiguous-focus-field",
            kind: AppCorePlannerFixtureKind::FocusField,
            transcript: "focus the email field",
            current_page_id: None,
            page: Some(fixture_page(vec![
                fixture_field("input-email", "#email", "Email", "Email address"),
                fixture_field(
                    "input-email-confirm",
                    "#email-confirm",
                    "Email confirmation",
                    "Confirm email",
                ),
            ])),
            active_skills: vec!["focus_field"],
            recent_context: None,
            confirmation_threshold: 0.95,
            expected_intent: IntentName::FillInput,
            expected_status: PlannerStatus::Ready,
            expected_selected_skills: vec!["focus_field"],
            expected_tool_sequence: vec![ToolName::ReportResult],
            expected_focus_element_id: None,
            expected_typed_text: None,
            expected_next_active_element_id: None,
            expected_next_pending_text: None,
        },
        AppCorePlannerFixture {
            name: "fill-field-success",
            kind: AppCorePlannerFixtureKind::FillField,
            transcript: "fill the email field with phil@example.com",
            current_page_id: None,
            page: Some(fixture_page(vec![
                fixture_field("input-email", "#email", "Email", "Email address"),
                fixture_field("input-password", "#password", "Password", "Password"),
            ])),
            active_skills: vec!["fill_field_by_label"],
            recent_context: None,
            confirmation_threshold: 0.9,
            expected_intent: IntentName::FillInput,
            expected_status: PlannerStatus::Ready,
            expected_selected_skills: vec!["fill_field_by_label"],
            expected_tool_sequence: vec![ToolName::FocusElement, ToolName::TypeIntoElement],
            expected_focus_element_id: Some("input-email"),
            expected_typed_text: Some("phil@example.com"),
            expected_next_active_element_id: None,
            expected_next_pending_text: None,
        },
        AppCorePlannerFixture {
            name: "fill-and-submit-confirmation",
            kind: AppCorePlannerFixtureKind::FillAndSubmit,
            transcript: "fill the email field with phil@example.com and then submit",
            current_page_id: None,
            page: Some(fixture_page(vec![fixture_field(
                "input-email",
                "#email",
                "Email",
                "Email address",
            )])),
            active_skills: vec!["fill_and_submit_form"],
            recent_context: None,
            confirmation_threshold: 0.9,
            expected_intent: IntentName::SubmitForm,
            expected_status: PlannerStatus::NeedsConfirmation,
            expected_selected_skills: vec!["fill_and_submit_form"],
            expected_tool_sequence: vec![
                ToolName::ConfirmAction,
                ToolName::FocusElement,
                ToolName::TypeIntoElement,
                ToolName::SubmitActiveForm,
            ],
            expected_focus_element_id: Some("input-email"),
            expected_typed_text: Some("phil@example.com"),
            expected_next_active_element_id: None,
            expected_next_pending_text: None,
        },
        AppCorePlannerFixture {
            name: "follow-up-replacement",
            kind: AppCorePlannerFixtureKind::FollowUpCorrection,
            transcript: "put Seattle there instead",
            current_page_id: Some("page-1"),
            page: Some(fixture_page(vec![InteractiveElement {
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
            }])),
            active_skills: vec!["fill_field_by_label"],
            recent_context: Some(RecentFieldContext {
                page_id: String::from("page-1"),
                target_description: Some(String::from("city")),
                active_element_id: Some(String::from("input-city")),
                candidate_element_ids: vec![String::from("input-city")],
                pending_text: Some(String::from("Portland")),
                submit_after: false,
            }),
            confirmation_threshold: 0.9,
            expected_intent: IntentName::FillInput,
            expected_status: PlannerStatus::Ready,
            expected_selected_skills: vec!["fill_field_by_label"],
            expected_tool_sequence: vec![ToolName::FocusElement, ToolName::TypeIntoElement],
            expected_focus_element_id: Some("input-city"),
            expected_typed_text: Some("Seattle"),
            expected_next_active_element_id: Some("input-city"),
            expected_next_pending_text: Some("Seattle"),
        },
        AppCorePlannerFixture {
            name: "follow-up-other-field",
            kind: AppCorePlannerFixtureKind::FollowUpCorrection,
            transcript: "no, the other field",
            current_page_id: Some("page-1"),
            page: Some(fixture_page(vec![
                fixture_field("input-email", "#email", "Email", "Email"),
                fixture_field(
                    "input-billing-email",
                    "#billing-email",
                    "Billing email",
                    "Billing email",
                ),
            ])),
            active_skills: vec!["fill_and_submit_form"],
            recent_context: Some(RecentFieldContext {
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
            confirmation_threshold: 0.9,
            expected_intent: IntentName::SubmitForm,
            expected_status: PlannerStatus::NeedsConfirmation,
            expected_selected_skills: vec!["fill_and_submit_form"],
            expected_tool_sequence: vec![
                ToolName::ConfirmAction,
                ToolName::FocusElement,
                ToolName::TypeIntoElement,
                ToolName::SubmitActiveForm,
            ],
            expected_focus_element_id: Some("input-billing-email"),
            expected_typed_text: Some("phil@example.com"),
            expected_next_active_element_id: Some("input-billing-email"),
            expected_next_pending_text: Some("phil@example.com"),
        },
        AppCorePlannerFixture {
            name: "ambiguous-submit-form",
            kind: AppCorePlannerFixtureKind::SubmitForm,
            transcript: "submit form",
            current_page_id: None,
            page: Some(fixture_page(vec![
                fixture_form("form-shipping", "#shipping-form", "Shipping"),
                fixture_form("form-billing", "#billing-form", "Billing"),
            ])),
            active_skills: vec!["submit_form"],
            recent_context: None,
            confirmation_threshold: 0.9,
            expected_intent: IntentName::SubmitForm,
            expected_status: PlannerStatus::Ready,
            expected_selected_skills: vec!["submit_form"],
            expected_tool_sequence: vec![ToolName::ReportResult],
            expected_focus_element_id: None,
            expected_typed_text: None,
            expected_next_active_element_id: None,
            expected_next_pending_text: None,
        },
    ];

    for fixture in fixtures {
        assert_app_core_planner_fixture(fixture);
    }
}

#[test]
fn ambiguous_click_regression_fixtures_pin_confirmation_threshold_behavior() {
    struct AmbiguousClickFixture {
        name: &'static str,
        candidates: Vec<crate::commands::ElementCandidate>,
        confirmation_threshold: f32,
        expected_element_id: Option<&'static str>,
        expected_confidence: Option<f32>,
        expected_requires_confirmation: bool,
    }

    let fixtures = vec![
        AmbiguousClickFixture {
            name: "close-candidates-trigger-follow-up",
            candidates: vec![
                crate::commands::ElementCandidate {
                    element_id: String::from("button-1"),
                    confidence_bps: 8_900,
                    matched_on: vec![String::from("description")],
                    rationale_codes: vec![String::from("accessible_name_exact")],
                },
                crate::commands::ElementCandidate {
                    element_id: String::from("button-2"),
                    confidence_bps: 8_400,
                    matched_on: vec![String::from("description")],
                    rationale_codes: vec![String::from("accessible_name_exact")],
                },
            ],
            confirmation_threshold: 0.9,
            expected_element_id: None,
            expected_confidence: Some(0.89),
            expected_requires_confirmation: true,
        },
        AmbiguousClickFixture {
            name: "threshold-crossing-allows-direct-click",
            candidates: vec![crate::commands::ElementCandidate {
                element_id: String::from("link-help"),
                confidence_bps: 8_800,
                matched_on: vec![String::from("accessible_name")],
                rationale_codes: vec![String::from("accessible_name_exact")],
            }],
            confirmation_threshold: 0.85,
            expected_element_id: Some("link-help"),
            expected_confidence: Some(0.88),
            expected_requires_confirmation: false,
        },
    ];

    for fixture in fixtures {
        let (chosen_element_id, chosen_confidence, requires_confirmation) =
            determine_find_element_resolution(
                &fixture.candidates,
                fixture.confirmation_threshold,
            );

        assert_eq!(
            chosen_element_id.as_deref(),
            fixture.expected_element_id,
            "fixture {} chose the wrong element",
            fixture.name
        );
        assert_eq!(
            chosen_confidence, fixture.expected_confidence,
            "fixture {} produced unexpected confidence",
            fixture.name
        );
        assert_eq!(
            requires_confirmation, fixture.expected_requires_confirmation,
            "fixture {} produced unexpected confirmation behavior",
            fixture.name
        );
    }
}

#[test]
fn problematic_page_regression_fixtures_cover_checkout_and_duplicate_cta_shapes() {
    let checkout_page = fixture_problematic_checkout_page();
    let newsletter_page = fixture_problematic_newsletter_page();
    let fixtures = vec![
        AppCorePlannerFixture {
            name: "problematic-checkout-ambiguous-email-focus",
            kind: AppCorePlannerFixtureKind::FocusField,
            transcript: "focus the email field",
            current_page_id: None,
            page: Some(checkout_page.clone()),
            active_skills: vec!["focus_field"],
            recent_context: None,
            confirmation_threshold: 0.95,
            expected_intent: IntentName::FillInput,
            expected_status: PlannerStatus::Ready,
            expected_selected_skills: vec!["focus_field"],
            expected_tool_sequence: vec![ToolName::ReportResult],
            expected_focus_element_id: None,
            expected_typed_text: None,
            expected_next_active_element_id: None,
            expected_next_pending_text: None,
        },
        AppCorePlannerFixture {
            name: "problematic-newsletter-fill-email",
            kind: AppCorePlannerFixtureKind::FillField,
            transcript: "fill the email field with phil@example.com",
            current_page_id: None,
            page: Some(newsletter_page),
            active_skills: vec!["fill_field_by_label"],
            recent_context: None,
            confirmation_threshold: 0.9,
            expected_intent: IntentName::FillInput,
            expected_status: PlannerStatus::Ready,
            expected_selected_skills: vec!["fill_field_by_label"],
            expected_tool_sequence: vec![ToolName::FocusElement, ToolName::TypeIntoElement],
            expected_focus_element_id: Some("input-newsletter-email"),
            expected_typed_text: Some("phil@example.com"),
            expected_next_active_element_id: None,
            expected_next_pending_text: None,
        },
        AppCorePlannerFixture {
            name: "problematic-checkout-other-field-correction",
            kind: AppCorePlannerFixtureKind::FollowUpCorrection,
            transcript: "no, the other field",
            current_page_id: Some("checkout-page"),
            page: Some(checkout_page.clone()),
            active_skills: vec!["fill_and_submit_form"],
            recent_context: Some(RecentFieldContext {
                page_id: String::from("checkout-page"),
                target_description: Some(String::from("email")),
                active_element_id: Some(String::from("input-shipping-email")),
                candidate_element_ids: vec![
                    String::from("input-shipping-email"),
                    String::from("input-billing-email"),
                ],
                pending_text: Some(String::from("phil@example.com")),
                submit_after: true,
            }),
            confirmation_threshold: 0.9,
            expected_intent: IntentName::SubmitForm,
            expected_status: PlannerStatus::NeedsConfirmation,
            expected_selected_skills: vec!["fill_and_submit_form"],
            expected_tool_sequence: vec![
                ToolName::ConfirmAction,
                ToolName::FocusElement,
                ToolName::TypeIntoElement,
                ToolName::SubmitActiveForm,
            ],
            expected_focus_element_id: Some("input-billing-email"),
            expected_typed_text: Some("phil@example.com"),
            expected_next_active_element_id: Some("input-billing-email"),
            expected_next_pending_text: Some("phil@example.com"),
        },
        AppCorePlannerFixture {
            name: "problematic-checkout-ambiguous-submit",
            kind: AppCorePlannerFixtureKind::SubmitForm,
            transcript: "submit form",
            current_page_id: None,
            page: Some(checkout_page.clone()),
            active_skills: vec!["submit_form"],
            recent_context: None,
            confirmation_threshold: 0.9,
            expected_intent: IntentName::SubmitForm,
            expected_status: PlannerStatus::Ready,
            expected_selected_skills: vec!["submit_form"],
            expected_tool_sequence: vec![ToolName::ReportResult],
            expected_focus_element_id: None,
            expected_typed_text: None,
            expected_next_active_element_id: None,
            expected_next_pending_text: None,
        },
    ];

    for fixture in fixtures {
        assert_app_core_planner_fixture(fixture);
    }

    let landing_page = fixture_problematic_landing_page();
    let query = build_find_element_query(&FindElementInput {
        request_id: String::from("req-problematic-cta"),
        timeout_ms: None,
        description: String::from("Get started"),
        text: None,
        role: Some(ElementRole::Button),
        color_hint: None,
        nearby_text: None,
        selector_hint: None,
        visibility_filter: crate::commands::ElementVisibilityFilter::VisibleOnly,
        max_candidates: Some(3),
    })
    .expect("landing-page query should be valid");
    let candidates =
        rank_find_element_candidates(&landing_page.interactive_elements, &query, 3);
    let (chosen_element_id, _, requires_confirmation) =
        determine_find_element_resolution(&candidates, 0.9);

    assert_eq!(candidates.len(), 2);
    assert_eq!(chosen_element_id, None);
    assert!(requires_confirmation);
}
