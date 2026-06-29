use super::*;

#[test]
fn normalize_optional_text_trims_and_drops_empty_values() {
    assert_eq!(normalize_optional_text(None), None);
    assert_eq!(normalize_optional_text(Some(String::from("   "))), None);
    assert_eq!(
        normalize_optional_text(Some(String::from("  next step  "))),
        Some(String::from("next step"))
    );
}

#[test]
fn normalize_absolute_url_accepts_trimmed_web_urls() {
    assert_eq!(
        normalize_absolute_url("  https://example.com/page  ").unwrap(),
        String::from("https://example.com/page")
    );
    assert_eq!(
        normalize_absolute_url("http://localhost:3000").unwrap(),
        String::from("http://localhost:3000")
    );
}

#[test]
fn normalize_absolute_url_rejects_relative_urls() {
    let error = normalize_absolute_url("/relative/path").unwrap_err();
    assert_eq!(error.code, "invalid_url");
}

#[test]
fn normalize_absolute_url_rejects_non_web_schemes() {
    // Planner/user navigation must fail closed to http/https only.
    for raw in [
        "about:blank",
        "file:///etc/passwd",
        "javascript:alert(1)",
        "data:text/html,<h1>x</h1>",
        "chrome://version",
        "//example.com",
        "http:example.com",
        "https:///missing-host",
    ] {
        let error = normalize_absolute_url(raw).unwrap_err();
        assert_eq!(error.code, "invalid_url", "expected rejection for {raw}");
    }
}

#[test]
fn browser_error_to_tool_error_keeps_navigation_failures_retryable_and_structured() {
    let navigate_error = browser_error_to_tool_error(
        String::from("open_url failed to navigate the active page"),
        BrowserError::Navigate(String::from("dns resolution failed")),
    );
    assert_eq!(navigate_error.code, "browser_navigation_failed");
    assert!(navigate_error.retryable);
    assert_eq!(
        navigate_error.details,
        Some(serde_json::json!({
            "reason": "failed to navigate browser page: dns resolution failed"
        }))
    );

    let history_error = browser_error_to_tool_error(
        String::from("go_back failed to update the current page"),
        BrowserError::History(String::from("no previous entry")),
    );
    assert_eq!(history_error.code, "browser_history_failed");
    assert!(history_error.retryable);
    assert_eq!(
        history_error.details,
        Some(serde_json::json!({
            "reason": "failed to read browser navigation history: no previous entry"
        }))
    );
}

#[test]
fn refresh_current_page_after_navigation_replaces_metadata_and_clears_stale_content() {
    let mut current_page = Some(PageModel {
        title: Some(String::from("Old page")),
        url: Some(String::from("https://example.com/old")),
        regions: vec![PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Paragraph,
            label: None,
            text: String::from("Stale extracted text"),
            bbox: None,
            source: RegionSource::Dom,
        }],
        interactive_elements: vec![InteractiveElement {
            element_id: String::from("button-1"),
            dom_locator: Some(String::from("#old-button")),
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
    });

    refresh_current_page_after_navigation(
        &mut current_page,
        Some(String::from("https://example.com/new")),
        Some(String::from("New page")),
    );

    let current_page = current_page.expect("page should still exist");
    assert_eq!(current_page.url.as_deref(), Some("https://example.com/new"));
    assert_eq!(current_page.title.as_deref(), Some("New page"));
    assert!(current_page.regions.is_empty());
    assert!(current_page.interactive_elements.is_empty());
}

#[test]
fn clear_navigation_follow_up_state_resets_cursor_and_recent_field_context() {
    let mut state = AppState::default();
    state.narration_cursor.current_index = Some(3);
    state.narration_cursor.current_region_id = Some(String::from("region-3"));
    state.narration_cursor.total_regions = 8;

    let mut recent_field_context = Some(RecentFieldContext {
        page_id: String::from("page-1"),
        target_description: Some(String::from("email field")),
        active_element_id: Some(String::from("input-email")),
        candidate_element_ids: vec![String::from("input-email"), String::from("input-alt")],
        pending_text: Some(String::from("user@example.com")),
        submit_after: true,
    });

    clear_navigation_follow_up_state(&mut state, &mut recent_field_context);

    assert_eq!(state.narration_cursor, Default::default());
    assert_eq!(recent_field_context, None);
}
