use super::*;

#[test]
fn resolve_form_element_rejects_non_form_roles() {
    let page = PageModel {
        title: Some(String::from("Example page")),
        url: Some(String::from("https://example.com")),
        regions: Vec::new(),
        interactive_elements: vec![InteractiveElement {
            element_id: String::from("button-1"),
            dom_locator: Some(String::from("#button-1")),
            role: ElementRole::Button,
            tag_name: String::from("button"),
            text: Some(String::from("Submit")),
            accessible_name: Some(String::from("Submit")),
            placeholder: None,
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::new(),
        }],
    };

    let error =
        resolve_form_element(&page, "button-1").expect_err("non-form roles should be rejected");
    assert_eq!(error.code, "element_not_form");
}

#[test]
fn rank_find_element_candidates_prefers_exact_accessible_name_matches() {
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
            element_id: String::from("button-2"),
            dom_locator: Some(String::from("#button-2")),
            role: ElementRole::Button,
            tag_name: String::from("button"),
            text: Some(String::from("Continue reading")),
            accessible_name: Some(String::from("Continue reading")),
            placeholder: None,
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::new(),
        },
    ];
    let query = build_find_element_query(&FindElementInput {
        request_id: String::from("req-find"),
        timeout_ms: None,
        description: String::from("Continue"),
        text: None,
        role: Some(ElementRole::Button),
        color_hint: None,
        nearby_text: None,
        selector_hint: None,
        visibility_filter: crate::commands::ElementVisibilityFilter::VisibleOnly,
        max_candidates: Some(3),
    })
    .expect("query should be valid");

    let candidates = rank_find_element_candidates(&elements, &query, 3);

    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].element_id, "button-1");
    assert!(candidates[0].confidence_bps > candidates[1].confidence_bps);
}

#[test]
fn rank_find_element_candidates_uses_selector_hint_and_respects_candidate_limit() {
    let elements = vec![
        InteractiveElement {
            element_id: String::from("button-primary"),
            dom_locator: Some(String::from("#checkout-submit")),
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
            attributes: std::collections::BTreeMap::from([
                (String::from("data-testid"), String::from("checkout-submit")),
                (String::from("class"), String::from("cta primary")),
            ]),
        },
        InteractiveElement {
            element_id: String::from("button-secondary"),
            dom_locator: Some(String::from("#continue-reading")),
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
            attributes: std::collections::BTreeMap::from([(
                String::from("data-testid"),
                String::from("continue-reading"),
            )]),
        },
        InteractiveElement {
            element_id: String::from("button-tertiary"),
            dom_locator: Some(String::from("#continue-later")),
            role: ElementRole::Button,
            tag_name: String::from("button"),
            text: Some(String::from("Continue later")),
            accessible_name: Some(String::from("Continue later")),
            placeholder: None,
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::from([(
                String::from("data-testid"),
                String::from("continue-later"),
            )]),
        },
    ];
    let query = build_find_element_query(&FindElementInput {
        request_id: String::from("req-find"),
        timeout_ms: None,
        description: String::from("Continue"),
        text: None,
        role: Some(ElementRole::Button),
        color_hint: None,
        nearby_text: None,
        selector_hint: Some(String::from("checkout-submit")),
        visibility_filter: crate::commands::ElementVisibilityFilter::VisibleOnly,
        max_candidates: Some(2),
    })
    .expect("query should be valid");

    let candidates = rank_find_element_candidates(&elements, &query, 2);

    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].element_id, "button-primary");
    assert!(candidates[0]
        .matched_on
        .iter()
        .any(|matched_on| matched_on == "selector_hint"));
    assert!(candidates[0].confidence_bps > candidates[1].confidence_bps);
    assert!(!candidates
        .iter()
        .any(|candidate| candidate.element_id == "button-tertiary"));
}

#[test]
fn build_find_element_query_normalizes_optional_hints_into_summary() {
    let query = build_find_element_query(&FindElementInput {
        request_id: String::from("req-find"),
        timeout_ms: None,
        description: String::from("  Continue  "),
        text: Some(String::from("  Start now  ")),
        role: Some(ElementRole::Button),
        color_hint: Some(String::from("  primary blue  ")),
        nearby_text: Some(String::from("  pricing  ")),
        selector_hint: Some(String::from("  cta-primary  ")),
        visibility_filter: crate::commands::ElementVisibilityFilter::VisibleOnly,
        max_candidates: Some(3),
    })
    .expect("query should be valid");

    assert_eq!(query.description.as_deref(), Some("Continue"));
    assert_eq!(query.text.as_deref(), Some("Start now"));
    assert_eq!(query.color_hint.as_deref(), Some("primary blue"));
    assert_eq!(query.nearby_text.as_deref(), Some("pricing"));
    assert_eq!(query.selector_hint.as_deref(), Some("cta-primary"));
    assert_eq!(
        query.summary,
        "description=Continue; text=Start now; role=Button; color_hint=primary blue; nearby_text=pricing; selector_hint=cta-primary"
    );
}

#[test]
fn determine_find_element_resolution_flags_close_candidates_for_confirmation() {
    let candidates = vec![
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
    ];

    let (chosen_element_id, chosen_confidence, requires_confirmation) =
        determine_find_element_resolution(&candidates, 0.9);

    assert_eq!(chosen_element_id, None);
    assert_eq!(chosen_confidence, Some(0.89));
    assert!(requires_confirmation);
}

#[test]
fn determine_find_element_resolution_uses_configured_confidence_threshold() {
    let candidates = vec![crate::commands::ElementCandidate {
        element_id: String::from("link-help"),
        confidence_bps: 8_800,
        matched_on: vec![String::from("accessible_name")],
        rationale_codes: vec![String::from("accessible_name_exact")],
    }];

    let (chosen_element_id, chosen_confidence, requires_confirmation) =
        determine_find_element_resolution(&candidates, 0.9);
    assert_eq!(chosen_element_id, None);
    assert_eq!(chosen_confidence, Some(0.88));
    assert!(requires_confirmation);

    let (chosen_element_id, chosen_confidence, requires_confirmation) =
        determine_find_element_resolution(&candidates, 0.85);
    assert_eq!(chosen_element_id, Some(String::from("link-help")));
    assert_eq!(chosen_confidence, Some(0.88));
    assert!(!requires_confirmation);
}
