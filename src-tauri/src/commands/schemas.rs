use super::*;

pub(crate) fn schema_json<T: JsonSchema>() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(T)).expect("schema generation should serialize")
}

pub fn planner_output_schema() -> serde_json::Value {
    schema_json::<PlannerOutput>()
}

pub fn canonical_planner_output_examples() -> BTreeMap<String, PlannerOutput> {
    BTreeMap::from([
        (
            String::from("get_status"),
            PlannerOutput {
                status: PlannerStatus::Ready,
                intent: IntentSummary {
                    name: IntentName::GetStatus,
                    goal: String::from("Report the current runtime status."),
                    target_description: None,
                },
                selected_skills: vec![String::from("get_status")],
                steps: vec![
                    PlannedStep {
                        step_id: String::from("fetch-runtime-status"),
                        tool_name: ToolName::GetRuntimeStatus,
                        arguments: serde_json::json!({
                            "request_id": "example-get-status",
                            "timeout_ms": null,
                            "include_provider_modes": true
                        }),
                        purpose: String::from("Read the current runtime status before speaking."),
                        on_success: StepTransition::NextStep {
                            step_id: String::from("report-runtime-status"),
                        },
                        on_failure: StepTransition::Replan,
                    },
                    PlannedStep {
                        step_id: String::from("report-runtime-status"),
                        tool_name: ToolName::ReportResult,
                        arguments: serde_json::json!({
                            "request_id": "example-get-status",
                            "timeout_ms": null,
                            "status": "Success",
                            "summary": "Browser is visible, listening is idle, and nothing is currently speaking.",
                            "next_recommended_action": null,
                            "user_message": "Browser visible. Listening idle. Not speaking."
                        }),
                        purpose: String::from("Speak a short status summary to the user."),
                        on_success: StepTransition::Complete,
                        on_failure: StepTransition::Replan,
                    },
                ],
                requires_confirmation: false,
                confirmation_reason: None,
                blocked_reason: None,
                user_message: None,
            },
        ),
        (
            String::from("read_title"),
            PlannerOutput {
                status: PlannerStatus::Ready,
                intent: IntentSummary {
                    name: IntentName::ReadTitle,
                    goal: String::from("Read the current page title."),
                    target_description: None,
                },
                selected_skills: vec![String::from("read_title")],
                steps: vec![PlannedStep {
                    step_id: String::from("report-page-title"),
                    tool_name: ToolName::ReportResult,
                    arguments: serde_json::json!({
                        "request_id": "example-read-title",
                        "timeout_ms": null,
                        "status": "Success",
                        "summary": "Page title is Example article.",
                        "next_recommended_action": null,
                        "user_message": "Page title is Example article."
                    }),
                    purpose: String::from("Speak the current page title."),
                    on_success: StepTransition::Complete,
                    on_failure: StepTransition::Replan,
                }],
                requires_confirmation: false,
                confirmation_reason: None,
                blocked_reason: None,
                user_message: None,
            },
        ),
        (
            String::from("set_playback_volume"),
            PlannerOutput {
                status: PlannerStatus::Ready,
                intent: IntentSummary {
                    name: IntentName::SetPlaybackVolume,
                    goal: String::from("Set playback volume to 70%."),
                    target_description: Some(String::from("70%")),
                },
                selected_skills: vec![String::from("set_volume")],
                steps: vec![
                    PlannedStep {
                        step_id: String::from("set-playback-volume"),
                        tool_name: ToolName::SetPlaybackVolume,
                        arguments: serde_json::json!({
                            "request_id": "example-set-volume",
                            "timeout_ms": null,
                            "volume": 0.7
                        }),
                        purpose: String::from("Apply and persist the requested playback volume."),
                        on_success: StepTransition::NextStep {
                            step_id: String::from("report-playback-volume"),
                        },
                        on_failure: StepTransition::Replan,
                    },
                    PlannedStep {
                        step_id: String::from("report-playback-volume"),
                        tool_name: ToolName::ReportResult,
                        arguments: serde_json::json!({
                            "request_id": "example-set-volume",
                            "timeout_ms": null,
                            "status": "Success",
                            "summary": "Playback volume set to 70%.",
                            "next_recommended_action": null,
                            "user_message": "Playback volume set to 70%."
                        }),
                        purpose: String::from("Confirm the updated playback volume."),
                        on_success: StepTransition::Complete,
                        on_failure: StepTransition::Replan,
                    },
                ],
                requires_confirmation: false,
                confirmation_reason: None,
                blocked_reason: None,
                user_message: None,
            },
        ),
        (
            String::from("click_element_ready"),
            PlannerOutput {
                status: PlannerStatus::Ready,
                intent: IntentSummary {
                    name: IntentName::ClickElement,
                    goal: String::from("Open the help link."),
                    target_description: Some(String::from("help link")),
                },
                selected_skills: vec![String::from("open_link_by_text")],
                steps: vec![
                    PlannedStep {
                        step_id: String::from("click-help-link"),
                        tool_name: ToolName::ClickElement,
                        arguments: serde_json::json!({
                            "request_id": "example-click-link",
                            "timeout_ms": null,
                            "element_id": "link-help",
                            "click_mode": "Single"
                        }),
                        purpose: String::from(
                            "Activate the requested link without an extra confirmation step.",
                        ),
                        on_success: StepTransition::NextStep {
                            step_id: String::from("report-click-link"),
                        },
                        on_failure: StepTransition::Replan,
                    },
                    PlannedStep {
                        step_id: String::from("report-click-link"),
                        tool_name: ToolName::ReportResult,
                        arguments: serde_json::json!({
                            "request_id": "example-click-link",
                            "timeout_ms": null,
                            "status": "Success",
                            "summary": "Activated the help link.",
                            "next_recommended_action": null,
                            "user_message": "Opened the help link."
                        }),
                        purpose: String::from("Confirm the ordinary click action to the user."),
                        on_success: StepTransition::Complete,
                        on_failure: StepTransition::Replan,
                    },
                ],
                requires_confirmation: false,
                confirmation_reason: None,
                blocked_reason: None,
                user_message: None,
            },
        ),
        (
            String::from("click_element_with_confirmation"),
            PlannerOutput {
                status: PlannerStatus::NeedsConfirmation,
                intent: IntentSummary {
                    name: IntentName::ClickElement,
                    goal: String::from("Open the submit button after confirmation."),
                    target_description: Some(String::from("submit button")),
                },
                selected_skills: vec![
                    String::from("open_link_by_text"),
                    String::from("confirm_action"),
                ],
                steps: vec![
                    PlannedStep {
                        step_id: String::from("confirm-click-target"),
                        tool_name: ToolName::ConfirmAction,
                        arguments: serde_json::json!({
                            "request_id": "example-confirm-click",
                            "timeout_ms": null,
                            "prompt_text": "Do you want me to activate the submit button?",
                            "reason": "The requested click may submit data or navigate away."
                        }),
                        purpose: String::from("Ask for confirmation before the protected click."),
                        on_success: StepTransition::RequestConfirmation,
                        on_failure: StepTransition::Replan,
                    },
                    PlannedStep {
                        step_id: String::from("click-submit-button"),
                        tool_name: ToolName::ClickElement,
                        arguments: serde_json::json!({
                            "request_id": "example-confirm-click",
                            "timeout_ms": null,
                            "element_id": "button-submit",
                            "click_mode": "Single"
                        }),
                        purpose: String::from("Activate the confirmed target element."),
                        on_success: StepTransition::Complete,
                        on_failure: StepTransition::Replan,
                    },
                ],
                requires_confirmation: true,
                confirmation_reason: Some(String::from(
                    "Clicking the submit button may send data or change page context.",
                )),
                blocked_reason: None,
                user_message: Some(String::from(
                    "Please confirm before I activate the submit button.",
                )),
            },
        ),
    ])
}

pub fn tool_input_schema(tool_name: &ToolName) -> Option<serde_json::Value> {
    match tool_name {
        ToolName::OpenUrl => Some(schema_json::<OpenUrlInput>()),
        ToolName::GoBack => Some(schema_json::<GoBackInput>()),
        ToolName::GoForward => Some(schema_json::<GoForwardInput>()),
        ToolName::ReloadPage => Some(schema_json::<ReloadPageInput>()),
        ToolName::GetHtml => Some(schema_json::<GetHtmlInput>()),
        ToolName::EvalJs => Some(schema_json::<EvalJsInput>()),
        ToolName::ScrollPage => Some(schema_json::<ScrollPageInput>()),
        ToolName::CaptureScreenshot => Some(schema_json::<CaptureScreenshotInput>()),
        ToolName::RunOcr => Some(schema_json::<RunOcrInput>()),
        ToolName::MergeOcrIntoPageModel => Some(schema_json::<MergeOcrIntoPageModelInput>()),
        ToolName::SetBrowserVisibility => Some(schema_json::<SetBrowserVisibilityInput>()),
        ToolName::GetPageSnapshot => Some(schema_json::<GetPageSnapshotInput>()),
        ToolName::ExtractPageModel => Some(schema_json::<ExtractPageModelInput>()),
        ToolName::ListInteractiveElements => Some(schema_json::<ListInteractiveElementsInput>()),
        ToolName::FindElement => Some(schema_json::<FindElementInput>()),
        ToolName::ClickElement => Some(schema_json::<ClickElementInput>()),
        ToolName::FocusElement => Some(schema_json::<FocusElementInput>()),
        ToolName::TypeIntoElement => Some(schema_json::<TypeIntoElementInput>()),
        ToolName::SubmitActiveForm => Some(schema_json::<SubmitActiveFormInput>()),
        ToolName::ReadRegion => Some(schema_json::<ReadRegionInput>()),
        ToolName::ReadNextRegion => Some(schema_json::<ReadNextRegionInput>()),
        ToolName::ReadPreviousRegion => Some(schema_json::<ReadPreviousRegionInput>()),
        ToolName::StopSpeaking => Some(schema_json::<StopSpeakingInput>()),
        ToolName::StartListening => Some(schema_json::<StartListeningInput>()),
        ToolName::StopListening => Some(schema_json::<StopListeningInput>()),
        ToolName::TranscribeCommand => Some(schema_json::<TranscribeCommandInput>()),
        ToolName::SetTtsVoice => Some(schema_json::<SetTtsVoiceInput>()),
        ToolName::SetPlaybackVolume => Some(schema_json::<SetPlaybackVolumeInput>()),
        ToolName::SetPlaybackSpeed => Some(schema_json::<SetPlaybackSpeedInput>()),
        ToolName::GetAgentState => Some(schema_json::<GetAgentStateInput>()),
        ToolName::GetRuntimeStatus => Some(schema_json::<GetRuntimeStatusInput>()),
        ToolName::ConfirmAction => Some(schema_json::<ConfirmActionInput>()),
        ToolName::ReportResult => Some(schema_json::<ReportResultInput>()),
    }
}

pub fn tool_output_schema(tool_name: &ToolName) -> Option<serde_json::Value> {
    match tool_name {
        ToolName::OpenUrl => Some(schema_json::<ToolResult<OpenUrlData>>()),
        ToolName::GoBack => Some(schema_json::<ToolResult<GoBackData>>()),
        ToolName::GoForward => Some(schema_json::<ToolResult<GoForwardData>>()),
        ToolName::ReloadPage => Some(schema_json::<ToolResult<ReloadPageData>>()),
        ToolName::GetHtml => Some(schema_json::<ToolResult<GetHtmlData>>()),
        ToolName::EvalJs => Some(schema_json::<ToolResult<EvalJsData>>()),
        ToolName::ScrollPage => Some(schema_json::<ToolResult<ScrollPageData>>()),
        ToolName::CaptureScreenshot => Some(schema_json::<ToolResult<CaptureScreenshotData>>()),
        ToolName::RunOcr => Some(schema_json::<ToolResult<RunOcrData>>()),
        ToolName::MergeOcrIntoPageModel => {
            Some(schema_json::<ToolResult<MergeOcrIntoPageModelData>>())
        }
        ToolName::SetBrowserVisibility => {
            Some(schema_json::<ToolResult<SetBrowserVisibilityData>>())
        }
        ToolName::GetPageSnapshot => Some(schema_json::<ToolResult<PageSnapshotData>>()),
        ToolName::ExtractPageModel => Some(schema_json::<ToolResult<ExtractPageModelData>>()),
        ToolName::ListInteractiveElements => {
            Some(schema_json::<ToolResult<ListInteractiveElementsData>>())
        }
        ToolName::FindElement => Some(schema_json::<ToolResult<FindElementData>>()),
        ToolName::ClickElement => Some(schema_json::<ToolResult<ClickElementData>>()),
        ToolName::FocusElement => Some(schema_json::<ToolResult<FocusElementData>>()),
        ToolName::TypeIntoElement => Some(schema_json::<ToolResult<TypeIntoElementData>>()),
        ToolName::SubmitActiveForm => Some(schema_json::<ToolResult<SubmitActiveFormData>>()),
        ToolName::ReadRegion => Some(schema_json::<ToolResult<ReadRegionData>>()),
        ToolName::ReadNextRegion => Some(schema_json::<ToolResult<ReadNextRegionData>>()),
        ToolName::ReadPreviousRegion => Some(schema_json::<ToolResult<ReadPreviousRegionData>>()),
        ToolName::StopSpeaking => Some(schema_json::<ToolResult<StopSpeakingData>>()),
        ToolName::StartListening => Some(schema_json::<ToolResult<StartListeningData>>()),
        ToolName::StopListening => Some(schema_json::<ToolResult<StopListeningData>>()),
        ToolName::TranscribeCommand => Some(schema_json::<ToolResult<TranscribeCommandData>>()),
        ToolName::SetTtsVoice => Some(schema_json::<ToolResult<SetTtsVoiceData>>()),
        ToolName::SetPlaybackVolume => Some(schema_json::<ToolResult<SetPlaybackVolumeData>>()),
        ToolName::SetPlaybackSpeed => Some(schema_json::<ToolResult<SetPlaybackSpeedData>>()),
        ToolName::GetAgentState => Some(schema_json::<ToolResult<AgentStateData>>()),
        ToolName::GetRuntimeStatus => Some(schema_json::<ToolResult<GetRuntimeStatusData>>()),
        ToolName::ConfirmAction => Some(schema_json::<ToolResult<ConfirmActionData>>()),
        ToolName::ReportResult => Some(schema_json::<ToolResult<ReportResultData>>()),
    }
}
