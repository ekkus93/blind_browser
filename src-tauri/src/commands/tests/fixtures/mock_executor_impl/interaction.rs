use super::*;

pub(super) fn execute_start_listening(
    ex: &mut MockExecutor,
    input: StartListeningInput,
) -> ToolResult<StartListeningData> {
    ex.last_start_listening_request = Some(input.clone());
    ToolResult::success(
        ToolName::StartListening,
        input.request_id,
        StartListeningData {
            listening_state: ListeningState {
                is_listening: true,
                push_to_talk_enabled: true,
            },
            activated: true,
        },
        vec![String::from("started listening for voice input")],
    )
}

pub(super) fn execute_stop_listening(
    ex: &mut MockExecutor,
    input: StopListeningInput,
) -> ToolResult<StopListeningData> {
    ex.last_stop_listening_request = Some(input.clone());
    ToolResult::success(
        ToolName::StopListening,
        input.request_id,
        StopListeningData {
            listening_state: ListeningState {
                is_listening: false,
                push_to_talk_enabled: true,
            },
            deactivated: true,
        },
        vec![String::from("stopped listening for voice input")],
    )
}

pub(super) fn execute_transcribe_command(
    ex: &mut MockExecutor,
    input: TranscribeCommandInput,
) -> ToolResult<TranscribeCommandData> {
    ex.last_transcribe_command_request = Some(input.clone());
    ToolResult::success(
        ToolName::TranscribeCommand,
        input.request_id,
        TranscribeCommandData {
            transcript: Some(String::from("read the next section")),
            confidence: None,
            audio_duration_ms: input.max_duration_ms.or(Some(3_000)),
            listening_state: ListeningState {
                is_listening: !input.stop_mode.auto_stops(),
                push_to_talk_enabled: true,
            },
        },
        vec![String::from("transcribed a spoken command")],
    )
}

pub(super) fn execute_get_page_snapshot(
    ex: &mut MockExecutor,
    input: GetPageSnapshotInput,
) -> ToolResult<PageSnapshotData> {
    ex.last_snapshot_request = Some(input.clone());
    ToolResult::success(
        ToolName::GetPageSnapshot,
        input.request_id,
        PageSnapshotData {
            page_id: String::from("page-1"),
            url: String::from("https://example.com/article"),
            title: Some(String::from("Example article")),
            visible_text_excerpt: String::from("First paragraph"),
            interactive_elements: if input.include_interactive_elements {
                vec![InteractiveElement {
                    element_id: String::from("link-1"),
                    dom_locator: Some(String::from("#link-1")),
                    role: crate::page_model::ElementRole::Link,
                    tag_name: String::from("a"),
                    text: Some(String::from("Read more")),
                    accessible_name: Some(String::from("Read more")),
                    placeholder: None,
                    href: Some(String::from("https://example.com/more")),
                    value: None,
                    bbox: None,
                    visible: true,
                    enabled: true,
                    attributes: std::collections::BTreeMap::new(),
                }]
            } else {
                Vec::new()
            },
            scroll_y: 120.0,
            viewport_width: 1280.0,
            viewport_height: 720.0,
            document_height: 2400.0,
        },
        vec![String::from("captured page snapshot")],
    )
}

pub(super) fn execute_list_interactive_elements(
    ex: &mut MockExecutor,
    input: ListInteractiveElementsInput,
) -> ToolResult<ListInteractiveElementsData> {
    ex.last_list_request = Some(input.clone());
    ToolResult::success(
        ToolName::ListInteractiveElements,
        input.request_id,
        ListInteractiveElementsData {
            page_id: String::from("page-1"),
            elements: vec![InteractiveElement {
                element_id: String::from("button-1"),
                dom_locator: Some(String::from("#button-1")),
                role: crate::page_model::ElementRole::Button,
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
            visible_count: 1,
        },
        vec![String::from("listed interactive elements")],
    )
}

pub(super) fn execute_find_element(
    ex: &mut MockExecutor,
    input: FindElementInput,
) -> ToolResult<FindElementData> {
    ex.last_find_request = Some(input.clone());
    ToolResult::success(
        ToolName::FindElement,
        input.request_id,
        FindElementData {
            query_summary: String::from("role=Button; description=continue"),
            chosen_element_id: Some(String::from("button-1")),
            chosen_confidence: Some(0.94),
            candidates: vec![ElementCandidate {
                element_id: String::from("button-1"),
                confidence_bps: 9400,
                matched_on: vec![String::from("description"), String::from("role")],
                rationale_codes: vec![
                    String::from("accessible_name_exact"),
                    String::from("role_match"),
                ],
            }],
            requires_confirmation: false,
        },
        vec![String::from("found a matching element")],
    )
}

pub(super) fn execute_click_element(
    ex: &mut MockExecutor,
    input: ClickElementInput,
) -> ToolResult<ClickElementData> {
    ex.last_click_request = Some(input.clone());
    ToolResult::success(
        ToolName::ClickElement,
        input.request_id,
        ClickElementData {
            element_id: input.element_id,
            action_performed: true,
            page_changed: false,
            navigation_url: None,
            resulting_title: Some(String::from("Example article")),
        },
        vec![String::from("clicked the requested element")],
    )
}

pub(super) fn execute_focus_element(
    ex: &mut MockExecutor,
    input: FocusElementInput,
) -> ToolResult<FocusElementData> {
    ex.last_focus_request = Some(input.clone());
    ToolResult::success(
        ToolName::FocusElement,
        input.request_id,
        FocusElementData {
            element_id: input.element_id,
            focused: true,
            element_role: Some(crate::page_model::ElementRole::Input),
        },
        vec![String::from("focused the requested element")],
    )
}

pub(super) fn execute_type_into_element(
    ex: &mut MockExecutor,
    input: TypeIntoElementInput,
) -> ToolResult<TypeIntoElementData> {
    ex.last_type_request = Some(input.clone());
    ToolResult::success(
        ToolName::TypeIntoElement,
        input.request_id,
        TypeIntoElementData {
            element_id: input.element_id,
            text_length: input.text.chars().count(),
            value_after: Some(input.text),
            accepted_input: true,
        },
        vec![String::from("typed into the requested element")],
    )
}

pub(super) fn execute_submit_active_form(
    ex: &mut MockExecutor,
    input: SubmitActiveFormInput,
) -> ToolResult<SubmitActiveFormData> {
    ex.last_submit_request = Some(input.clone());
    ToolResult::success(
        ToolName::SubmitActiveForm,
        input.request_id,
        SubmitActiveFormData {
            form_element_id: input.form_element_id,
            submitted: true,
            page_changed: true,
            navigation_url: Some(String::from("https://example.com/submitted")),
        },
        vec![String::from("submitted the active form")],
    )
}

pub(super) fn execute_extract_page_model(
    ex: &mut MockExecutor,
    input: ExtractPageModelInput,
) -> ToolResult<ExtractPageModelData> {
    ex.last_extract_request = Some(input.clone());
    ToolResult::success(
        ToolName::ExtractPageModel,
        input.request_id,
        ExtractPageModelData {
            page_model: PageModel {
                title: Some(String::from("Example article")),
                url: Some(String::from("https://example.com/article")),
                regions: Vec::new(),
                interactive_elements: if input.include_links {
                    vec![InteractiveElement {
                        element_id: String::from("link-1"),
                        dom_locator: Some(String::from("#link-1")),
                        role: crate::page_model::ElementRole::Link,
                        tag_name: String::from("a"),
                        text: Some(String::from("Read more")),
                        accessible_name: Some(String::from("Read more")),
                        placeholder: None,
                        href: Some(String::from("https://example.com/more")),
                        value: None,
                        bbox: None,
                        visible: true,
                        enabled: true,
                        attributes: std::collections::BTreeMap::new(),
                    }]
                } else {
                    Vec::new()
                },
            },
            region_count: 0,
            readable_region_count: 0,
            extraction_source: ExtractionSource::DomFallback,
        },
        vec![String::from("extracted page model")],
    )
}
