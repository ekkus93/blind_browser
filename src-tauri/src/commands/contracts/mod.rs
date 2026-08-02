use super::*;

mod interaction;
mod planner;
mod providers;
mod tools;

pub use interaction::*;
pub use planner::*;
pub use providers::*;
pub use tools::*;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ToolName {
    OpenUrl,
    GoBack,
    GoForward,
    ReloadPage,
    GetHtml,
    EvalJs,
    ScrollPage,
    CaptureScreenshot,
    SetBrowserVisibility,
    GetPageSnapshot,
    ExtractPageModel,
    ListInteractiveElements,
    FindElement,
    ClickElement,
    FocusElement,
    TypeIntoElement,
    SubmitActiveForm,
    ReadRegion,
    ReadNextRegion,
    ReadPreviousRegion,
    StopSpeaking,
    StartListening,
    StopListening,
    TranscribeCommand,
    SetTtsVoice,
    SetPlaybackVolume,
    SetPlaybackSpeed,
    RunOcr,
    MergeOcrIntoPageModel,
    GetAgentState,
    GetRuntimeStatus,
    ConfirmAction,
    ReportResult,
}

#[derive(Clone, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ToolError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
    pub details: Option<serde_json::Value>,
}

impl std::fmt::Debug for ToolError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ToolError")
            .field("code", &self.code)
            .field(
                "message",
                &crate::diagnostic_redaction::redact_diagnostic_text(&self.message),
            )
            .field("retryable", &self.retryable)
            .field(
                "details",
                &self
                    .details
                    .as_ref()
                    .map(crate::diagnostic_redaction::redact_json_value),
            )
            .finish()
    }
}

impl Serialize for ToolError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ToolError", 4)?;
        state.serialize_field("code", &self.code)?;
        state.serialize_field(
            "message",
            &crate::diagnostic_redaction::redact_diagnostic_text(&self.message),
        )?;
        state.serialize_field("retryable", &self.retryable)?;
        state.serialize_field(
            "details",
            &self
                .details
                .as_ref()
                .map(crate::diagnostic_redaction::redact_json_value),
        )?;
        state.end()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ToolWarning {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ToolResult<T> {
    pub ok: bool,
    pub tool_name: ToolName,
    pub request_id: String,
    pub timestamp_ms: u64,
    pub data: Option<T>,
    pub error: Option<ToolError>,
    pub warnings: Vec<ToolWarning>,
    pub observations: Vec<String>,
}

pub type SerializedToolResult = ToolResult<serde_json::Value>;

impl<T> ToolResult<T> {
    pub fn success(
        tool_name: ToolName,
        request_id: String,
        data: T,
        observations: Vec<String>,
    ) -> Self {
        Self {
            ok: true,
            tool_name,
            request_id,
            timestamp_ms: current_timestamp_ms(),
            data: Some(data),
            error: None,
            warnings: Vec::new(),
            observations,
        }
    }

    pub fn failure(
        tool_name: ToolName,
        request_id: String,
        error: ToolError,
        observations: Vec<String>,
    ) -> Self {
        Self {
            ok: false,
            tool_name,
            request_id,
            timestamp_ms: current_timestamp_ms(),
            data: None,
            error: Some(error),
            warnings: Vec::new(),
            observations,
        }
    }
}

pub trait DeterministicToolExecutor {
    fn confirmation_runtime_context(&self) -> ConfirmationRuntimeContext {
        ConfirmationRuntimeContext::detached()
    }

    fn preflight_planned_step(&mut self, _step: &PlannedStep) -> Result<(), ToolError> {
        Ok(())
    }

    fn execute_open_url(&mut self, input: OpenUrlInput) -> ToolResult<OpenUrlData>;
    fn execute_go_back(&mut self, input: GoBackInput) -> ToolResult<GoBackData>;
    fn execute_go_forward(&mut self, input: GoForwardInput) -> ToolResult<GoForwardData>;
    fn execute_reload_page(&mut self, input: ReloadPageInput) -> ToolResult<ReloadPageData>;
    fn execute_get_html(&mut self, input: GetHtmlInput) -> ToolResult<GetHtmlData>;
    fn execute_eval_js(&mut self, input: EvalJsInput) -> ToolResult<EvalJsData>;
    fn execute_scroll_page(&mut self, input: ScrollPageInput) -> ToolResult<ScrollPageData>;
    fn execute_capture_screenshot(
        &mut self,
        input: CaptureScreenshotInput,
    ) -> ToolResult<CaptureScreenshotData>;
    fn execute_run_ocr(&mut self, input: RunOcrInput) -> ToolResult<RunOcrData>;
    fn execute_merge_ocr_into_page_model(
        &mut self,
        input: MergeOcrIntoPageModelInput,
    ) -> ToolResult<MergeOcrIntoPageModelData>;
    fn execute_get_page_snapshot(
        &mut self,
        input: GetPageSnapshotInput,
    ) -> ToolResult<PageSnapshotData>;
    fn execute_list_interactive_elements(
        &mut self,
        input: ListInteractiveElementsInput,
    ) -> ToolResult<ListInteractiveElementsData>;
    fn execute_find_element(&mut self, input: FindElementInput) -> ToolResult<FindElementData>;
    fn execute_click_element(&mut self, input: ClickElementInput) -> ToolResult<ClickElementData>;
    fn execute_focus_element(&mut self, input: FocusElementInput) -> ToolResult<FocusElementData>;
    fn execute_type_into_element(
        &mut self,
        input: TypeIntoElementInput,
    ) -> ToolResult<TypeIntoElementData>;
    fn execute_submit_active_form(
        &mut self,
        input: SubmitActiveFormInput,
    ) -> ToolResult<SubmitActiveFormData>;
    fn execute_extract_page_model(
        &mut self,
        input: ExtractPageModelInput,
    ) -> ToolResult<ExtractPageModelData>;
    fn execute_read_region(&mut self, input: ReadRegionInput) -> ToolResult<ReadRegionData>;
    fn execute_read_next_region(
        &mut self,
        input: ReadNextRegionInput,
    ) -> ToolResult<ReadNextRegionData>;
    fn execute_read_previous_region(
        &mut self,
        input: ReadPreviousRegionInput,
    ) -> ToolResult<ReadPreviousRegionData>;
    fn execute_stop_speaking(&mut self, input: StopSpeakingInput) -> ToolResult<StopSpeakingData>;
    fn execute_start_listening(
        &mut self,
        input: StartListeningInput,
    ) -> ToolResult<StartListeningData>;
    fn execute_stop_listening(
        &mut self,
        input: StopListeningInput,
    ) -> ToolResult<StopListeningData>;
    fn execute_transcribe_command(
        &mut self,
        input: TranscribeCommandInput,
    ) -> ToolResult<TranscribeCommandData>;
    fn execute_set_tts_voice(&mut self, input: SetTtsVoiceInput) -> ToolResult<SetTtsVoiceData>;
    fn execute_set_playback_volume(
        &mut self,
        input: SetPlaybackVolumeInput,
    ) -> ToolResult<SetPlaybackVolumeData>;
    fn execute_set_playback_speed(
        &mut self,
        input: SetPlaybackSpeedInput,
    ) -> ToolResult<SetPlaybackSpeedData>;
    fn execute_set_browser_visibility(
        &mut self,
        input: SetBrowserVisibilityInput,
    ) -> ToolResult<SetBrowserVisibilityData>;
    fn execute_get_agent_state(&mut self, input: GetAgentStateInput) -> ToolResult<AgentStateData>;
    fn execute_get_runtime_status(
        &mut self,
        input: GetRuntimeStatusInput,
    ) -> ToolResult<GetRuntimeStatusData>;
    fn execute_confirm_action(
        &mut self,
        input: ConfirmActionInput,
    ) -> ToolResult<ConfirmActionData>;
    fn execute_report_result(&mut self, input: ReportResultInput) -> ToolResult<ReportResultData>;
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct AvailableTool {
    pub name: ToolName,
    pub description: String,
    pub input_schema_ref: String,
    pub output_schema_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SkillSummary {
    pub name: String,
    pub description: String,
    pub intent_tags: Vec<String>,
    pub allowed_tools: Option<Vec<ToolName>>,
    pub requires_confirmation: bool,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PlannerToolHistoryEntry {
    pub tool_name: ToolName,
    pub ok: bool,
    pub observation_summary: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct LastToolCallSummary {
    pub request_id: String,
    pub tool_name: ToolName,
    pub ok: bool,
    pub observation_summary: Vec<String>,
}

#[cfg(test)]
mod diagnostic_contract_tests {
    use super::*;

    #[test]
    fn tool_error_debug_and_serialization_redact_secrets() {
        let error = ToolError {
            code: String::from("test"),
            message: String::from("authorization: Bearer abcdefghijklmnop"),
            retryable: false,
            details: Some(serde_json::json!({
                "api_key": "sk-private-secret-value",
                "safe_count": 2,
            })),
        };
        let debug = format!("{error:?}");
        let json = serde_json::to_string(&error).unwrap();
        for output in [debug, json] {
            assert!(!output.contains("abcdefghijklmnop"));
            assert!(!output.contains("private-secret"));
            assert!(output.contains("REDACTED"));
        }
    }
}
