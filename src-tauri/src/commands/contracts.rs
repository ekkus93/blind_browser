use super::*;

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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ToolError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
    pub details: Option<serde_json::Value>,
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct AgentStateData {
    pub page_id: Option<String>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub browser_visibility: BrowserVisibilityMode,
    pub browser_history: BrowserHistoryState,
    pub narration_cursor: Option<NarrationCursor>,
    pub speaking: bool,
    pub listening_state: ListeningState,
    pub audio: RuntimeAudioState,
    pub last_transcript: Option<String>,
    pub last_tool_call: Option<LastToolCallSummary>,
    pub pending_confirmation_id: Option<String>,
    pub pending_plan_execution: Option<PendingPlanExecutionState>,
    pub tts_model_settings: TtsModelSettings,
    pub local_tts_model_settings: LocalTtsModelSettings,
    pub tts_voice_settings: TtsVoiceSettings,
    pub tts_provider_settings: TtsProviderSettings,
    pub asr_provider_settings: AsrProviderSettings,
    pub local_asr_model_settings: LocalAsrModelSettings,
    pub remote_planner_settings: RemotePlannerSettings,
    pub remote_tts_settings: RemoteTtsSettings,
    pub remote_asr_settings: RemoteAsrSettings,
    pub provider_failover_settings: ProviderFailoverSettings,
    pub confirmation_settings: ConfirmationSettings,
    pub ocr_threshold_settings: OcrThresholdSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct PageSnapshotData {
    pub page_id: String,
    pub url: String,
    pub title: Option<String>,
    pub visible_text_excerpt: String,
    pub interactive_elements: Vec<InteractiveElement>,
    pub scroll_y: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub document_height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProviderSelectionStatus {
    pub planner_mode: ProviderMode,
    pub tts_mode: ProviderMode,
    pub asr_mode: ProviderMode,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum RemoteProviderLabel {
    #[serde(rename = "OpenAI")]
    OpenAi,
    #[serde(rename = "Ollama")]
    Ollama,
}

impl From<&crate::config::RemoteProviderKind> for RemoteProviderLabel {
    fn from(value: &crate::config::RemoteProviderKind) -> Self {
        match value {
            crate::config::RemoteProviderKind::OpenAi => Self::OpenAi,
            crate::config::RemoteProviderKind::Ollama => Self::Ollama,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TtsModelOption {
    pub profile_name: String,
    pub model_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TtsModelSettings {
    pub mode: ProviderMode,
    pub active_profile: Option<String>,
    pub available_profiles: Vec<TtsModelOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct LocalTtsModelSettings {
    pub profile_name: Option<String>,
    pub backend: Option<LocalTtsBackend>,
    pub model_id: Option<String>,
    pub model_path: Option<String>,
    pub default_voice: Option<String>,
    pub sample_rate: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TtsVoiceOption {
    pub voice_name: String,
    pub display_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TtsVoiceSettings {
    pub mode: ProviderMode,
    pub active_voice: Option<String>,
    pub available_voices: Vec<TtsVoiceOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TtsProviderSettings {
    pub active_mode: ProviderMode,
    pub available_modes: Vec<ProviderMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct AsrProviderSettings {
    pub active_mode: ProviderMode,
    pub available_modes: Vec<ProviderMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct LocalAsrModelSettings {
    pub profile_name: Option<String>,
    pub backend: Option<LocalAsrBackend>,
    pub model_id: Option<String>,
    pub model_path: Option<String>,
    pub language: Option<String>,
    pub threads: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RemotePlannerSettings {
    pub profile_name: Option<String>,
    pub provider: Option<RemoteProviderLabel>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub api_key_reference: Option<String>,
    pub api_key_masked_value: Option<String>,
    pub organization_reference: Option<String>,
    pub project: Option<String>,
    pub temperature_milli: Option<u16>,
    pub max_output_tokens: Option<u32>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RemoteTtsSettings {
    pub profile_name: Option<String>,
    pub provider: Option<RemoteProviderLabel>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub api_key_reference: Option<String>,
    pub api_key_masked_value: Option<String>,
    pub organization_reference: Option<String>,
    pub project: Option<String>,
    pub voice: Option<String>,
    pub audio_format: Option<RemoteTtsAudioFormat>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RemoteAsrSettings {
    pub profile_name: Option<String>,
    pub provider: Option<RemoteProviderLabel>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub api_key_reference: Option<String>,
    pub api_key_masked_value: Option<String>,
    pub organization_reference: Option<String>,
    pub project: Option<String>,
    pub language: Option<String>,
    pub temperature_milli: Option<u16>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProviderFailoverSettings {
    pub planner_available: bool,
    pub tts_available: bool,
    pub asr_available: bool,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ConfirmationSettings {
    pub confirmation_confidence_threshold: f32,
    pub allow_click_without_confirmation: bool,
    pub always_confirm_submit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct OcrThresholdSettings {
    pub sparse_text_char_threshold: u32,
    pub sparse_text_region_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ElementCandidate {
    pub element_id: String,
    pub confidence_bps: u16,
    pub matched_on: Vec<String>,
    pub rationale_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct GetRuntimeStatusData {
    pub page_id: Option<String>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub browser_visibility: BrowserVisibilityMode,
    pub browser_history: BrowserHistoryState,
    pub listening_state: ListeningState,
    pub speaking: bool,
    pub audio: RuntimeAudioState,
    pub pending_confirmation_id: Option<String>,
    pub pending_plan_execution: Option<PendingPlanExecutionState>,
    pub provider_modes: Option<ProviderSelectionStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct PlannerSafetySettings {
    pub confirmation_confidence_threshold: f32,
    pub allow_click_without_confirmation: bool,
    pub always_confirm_submit: bool,
}

impl From<&crate::config::SafetySettings> for PlannerSafetySettings {
    fn from(safety: &crate::config::SafetySettings) -> Self {
        Self {
            confirmation_confidence_threshold: safety.confirmation_confidence_threshold,
            allow_click_without_confirmation: safety.allow_click_without_confirmation,
            always_confirm_submit: safety.always_confirm_submit,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct PlannerInput {
    pub request_id: String,
    pub transcript: String,
    pub agent_state: AgentStateData,
    pub safety: PlannerSafetySettings,
    pub available_tools: Vec<AvailableTool>,
    pub active_skill_names: Vec<String>,
    pub relevant_skill_summaries: Vec<SkillSummary>,
    pub page_snapshot: Option<PageSnapshotData>,
    pub page_model: Option<PageModel>,
    pub recent_tool_results: Vec<PlannerToolHistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum PlannerStatus {
    Ready,
    NeedsConfirmation,
    Blocked,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum BlockedReason {
    Gatekept,
    MissingContext,
    UnsupportedCapability,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub enum IntentName {
    OpenUrl,
    GoBack,
    GoForward,
    ReloadPage,
    GetCurrentUrl,
    ReadPage,
    ReadTitle,
    ReadNext,
    ReadPrevious,
    Repeat,
    Stop,
    StartListening,
    StopListening,
    TranscribeCommand,
    SetTtsVoice,
    SetPlaybackVolume,
    GetPlaybackVolume,
    SetPlaybackSpeed,
    GetPlaybackSpeed,
    SetBrowserVisibility,
    GetStatus,
    FindElement,
    ClickElement,
    FillInput,
    SubmitForm,
    Scroll,
    OcrRecovery,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct IntentSummary {
    pub name: IntentName,
    pub goal: String,
    pub target_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum StepTransition {
    NextStep { step_id: String },
    Complete,
    RequestConfirmation,
    Replan,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ExecutionTrace {
    pub executed_step_ids: Vec<String>,
    pub tool_results: Vec<SerializedToolResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum ExecutionOutcome {
    Complete {
        trace: ExecutionTrace,
    },
    AwaitingConfirmation {
        trace: ExecutionTrace,
        pending_confirmation_id: String,
        pending_plan_execution: PendingPlanExecutionState,
    },
    NeedsReplan {
        trace: ExecutionTrace,
    },
    Aborted {
        trace: ExecutionTrace,
        error: ToolError,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct PlannedStep {
    pub step_id: String,
    pub tool_name: ToolName,
    pub arguments: serde_json::Value,
    pub purpose: String,
    pub on_success: StepTransition,
    pub on_failure: StepTransition,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct PlannerOutput {
    pub status: PlannerStatus,
    pub intent: IntentSummary,
    pub selected_skills: Vec<String>,
    pub steps: Vec<PlannedStep>,
    pub requires_confirmation: bool,
    pub confirmation_reason: Option<String>,
    pub blocked_reason: Option<BlockedReason>,
    pub user_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct PendingPlanExecutionState {
    pub request_id: String,
    pub intent_name: IntentName,
    pub selected_skills: Vec<String>,
    pub confirmation_id: String,
    pub prompt_text: String,
    pub next_step_id: Option<String>,
    pub queued_step_ids: Vec<String>,
    pub queued_steps: Vec<PlannedStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ConfirmActionData {
    pub confirmation_id: String,
    pub prompt_text: String,
    pub confirmed: Option<bool>,
    pub timed_out: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ConfirmActionInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub prompt_text: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ConfirmActionResolution {
    pub tool_result: ToolResult<ConfirmActionData>,
    pub resume_outcome: ExecutionOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub enum TtsVoiceName {
    #[serde(rename = "Bella")]
    Bella,
    #[serde(rename = "Jasper")]
    Jasper,
    #[serde(rename = "Luna")]
    Luna,
    #[serde(rename = "Bruno")]
    Bruno,
    #[serde(rename = "Rosie")]
    Rosie,
    #[serde(rename = "Hugo")]
    Hugo,
    #[serde(rename = "Kiki")]
    Kiki,
    #[serde(rename = "Leo")]
    Leo,
    #[serde(rename = "alloy")]
    Alloy,
    #[serde(rename = "ash")]
    Ash,
    #[serde(rename = "ballad")]
    Ballad,
    #[serde(rename = "coral")]
    Coral,
    #[serde(rename = "echo")]
    Echo,
    #[serde(rename = "fable")]
    Fable,
    #[serde(rename = "onyx")]
    Onyx,
    #[serde(rename = "nova")]
    Nova,
    #[serde(rename = "sage")]
    Sage,
    #[serde(rename = "shimmer")]
    Shimmer,
    #[serde(rename = "verse")]
    Verse,
    #[serde(rename = "marin")]
    Marin,
    #[serde(rename = "cedar")]
    Cedar,
}

impl TtsVoiceName {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bella => "Bella",
            Self::Jasper => "Jasper",
            Self::Luna => "Luna",
            Self::Bruno => "Bruno",
            Self::Rosie => "Rosie",
            Self::Hugo => "Hugo",
            Self::Kiki => "Kiki",
            Self::Leo => "Leo",
            Self::Alloy => "alloy",
            Self::Ash => "ash",
            Self::Ballad => "ballad",
            Self::Coral => "coral",
            Self::Echo => "echo",
            Self::Fable => "fable",
            Self::Onyx => "onyx",
            Self::Nova => "nova",
            Self::Sage => "sage",
            Self::Shimmer => "shimmer",
            Self::Verse => "verse",
            Self::Marin => "marin",
            Self::Cedar => "cedar",
        }
    }
}

impl std::fmt::Display for TtsVoiceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub enum NarrationInterruptionMode {
    Queue,
    Interrupt,
}

impl NarrationInterruptionMode {
    pub fn interrupts_current_playback(self) -> bool {
        matches!(self, Self::Interrupt)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub enum NarrationBoundary {
    None,
    Start,
    End,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub enum ElementVisibilityFilter {
    All,
    VisibleOnly,
}

impl ElementVisibilityFilter {
    pub fn visible_only(self) -> bool {
        matches!(self, Self::VisibleOnly)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub enum ReloadMode {
    Standard,
    Hard,
}

impl ReloadMode {
    pub fn uses_cache_bypass(self) -> bool {
        matches!(self, Self::Hard)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub enum ClickMode {
    Single,
    Double,
}

impl ClickMode {
    pub fn is_double_click(self) -> bool {
        matches!(self, Self::Double)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub enum TextEntryMode {
    Append,
    Replace,
}

impl TextEntryMode {
    pub fn clears_existing_value(self) -> bool {
        matches!(self, Self::Replace)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub enum TextEntrySubmitMode {
    KeepEditing,
    Submit,
}

impl TextEntrySubmitMode {
    pub fn submits_after_entry(self) -> bool {
        matches!(self, Self::Submit)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub enum TranscriptionStopMode {
    KeepListening,
    AutoStop,
}

impl TranscriptionStopMode {
    pub fn auto_stops(self) -> bool {
        matches!(self, Self::AutoStop)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub enum ScreenshotScope {
    Viewport,
    FullPage,
}

impl ScreenshotScope {
    pub fn captures_full_page(self) -> bool {
        matches!(self, Self::FullPage)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ReportStatus {
    Success,
    NeedsFollowUp,
    Blocked,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ReportResultData {
    pub status: ReportStatus,
    pub summary: String,
    pub next_recommended_action: Option<String>,
    pub user_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ReportResultInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub status: ReportStatus,
    pub summary: String,
    pub next_recommended_action: Option<String>,
    pub user_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct OpenUrlInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub url: String,
    pub wait_for_load_state: Option<LoadState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct OpenUrlData {
    pub final_url: String,
    pub title: Option<String>,
    pub page_id: String,
    pub load_state: LoadState,
    pub http_status: Option<u16>,
    pub history: BrowserHistoryState,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct GoBackInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub steps: Option<u8>,
    pub wait_for_load_state: Option<LoadState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct GoBackData {
    pub navigated: bool,
    pub actual_steps: u8,
    pub final_url: Option<String>,
    pub title: Option<String>,
    pub load_state: Option<LoadState>,
    pub history: BrowserHistoryState,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct GoForwardInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub steps: Option<u8>,
    pub wait_for_load_state: Option<LoadState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct GoForwardData {
    pub navigated: bool,
    pub actual_steps: u8,
    pub final_url: Option<String>,
    pub title: Option<String>,
    pub load_state: Option<LoadState>,
    pub history: BrowserHistoryState,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ReloadPageInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub mode: ReloadMode,
    pub wait_for_load_state: Option<LoadState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ReloadPageData {
    pub reloaded: bool,
    pub final_url: String,
    pub title: Option<String>,
    pub load_state: LoadState,
    pub http_status: Option<u16>,
    pub history: BrowserHistoryState,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct GetHtmlInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct GetHtmlData {
    pub page_id: String,
    pub url: String,
    pub title: Option<String>,
    pub html: String,
    pub html_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct EvalJsInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub expression: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct EvalJsData {
    pub page_id: String,
    pub url: String,
    pub title: Option<String>,
    pub result: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ScrollPageInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub direction: ScrollDirection,
    pub amount_px: Option<f32>,
    pub target: Option<ScrollTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ScrollPageData {
    pub previous_scroll_y: f32,
    pub current_scroll_y: f32,
    pub reached_boundary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct CaptureScreenshotInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub scope: ScreenshotScope,
    pub region_id: Option<String>,
    pub bbox: Option<Rect>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct CaptureScreenshotData {
    pub image_id: String,
    pub path: String,
    pub bbox: Option<Rect>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct RunOcrInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub image_id: Option<String>,
    pub region_id: Option<String>,
    pub bbox: Option<Rect>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct RunOcrData {
    pub image_id: Option<String>,
    pub extracted_text: String,
    pub text_length: usize,
    pub confidence: Option<f32>,
    pub source_bbox: Option<Rect>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct MergeOcrIntoPageModelInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub page_id: String,
    pub region_id: Option<String>,
    pub ocr_text: String,
    pub source_bbox: Option<Rect>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct MergeOcrIntoPageModelData {
    pub page_id: String,
    pub updated_region_ids: Vec<String>,
    pub merged_text_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ReadRegionInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub region_id: String,
    pub interruption_mode: NarrationInterruptionMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ReadRegionData {
    pub region_id: String,
    pub region_index: usize,
    pub text_length: usize,
    pub speech_started: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FocusFieldCommand {
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FillFieldCommand {
    pub description: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FillFieldCorrectionCommand {
    AlternateField,
    ReplaceValue { text: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ReadNextRegionInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub interruption_mode: NarrationInterruptionMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ReadNextRegionData {
    pub cursor: NarrationCursor,
    pub region_id: Option<String>,
    pub speech_started: bool,
    pub boundary: NarrationBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ReadPreviousRegionInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub interruption_mode: NarrationInterruptionMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ReadPreviousRegionData {
    pub cursor: NarrationCursor,
    pub region_id: Option<String>,
    pub speech_started: bool,
    pub boundary: NarrationBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct StopSpeakingInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct StopSpeakingData {
    pub stopped: bool,
    pub interrupted_region_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct StartListeningInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct StartListeningData {
    pub listening_state: ListeningState,
    pub activated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct StopListeningInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct StopListeningData {
    pub listening_state: ListeningState,
    pub deactivated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TranscribeCommandInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub max_duration_ms: Option<u64>,
    pub stop_mode: TranscriptionStopMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct TranscribeCommandData {
    pub transcript: Option<String>,
    pub confidence: Option<f32>,
    pub audio_duration_ms: Option<u64>,
    pub listening_state: ListeningState,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct TranscribeAndExecuteCommandData {
    pub transcription: TranscribeCommandData,
    pub command_error: Option<ToolError>,
    pub execution_outcome: Option<ExecutionOutcome>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct GetPageSnapshotInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub include_interactive_elements: bool,
    pub text_excerpt_max_chars: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ExtractPageModelInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub use_dom_extraction: bool,
    pub include_headings: bool,
    pub include_links: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ListInteractiveElementsInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub visibility_filter: ElementVisibilityFilter,
    pub roles: Option<Vec<crate::page_model::ElementRole>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ListInteractiveElementsData {
    pub page_id: String,
    pub elements: Vec<InteractiveElement>,
    pub visible_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct FindElementInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub description: String,
    pub text: Option<String>,
    pub role: Option<crate::page_model::ElementRole>,
    pub color_hint: Option<String>,
    pub nearby_text: Option<String>,
    pub selector_hint: Option<String>,
    pub visibility_filter: ElementVisibilityFilter,
    pub max_candidates: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct FindElementData {
    pub query_summary: String,
    pub chosen_element_id: Option<String>,
    pub chosen_confidence: Option<f32>,
    pub candidates: Vec<ElementCandidate>,
    pub requires_confirmation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ClickElementInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub element_id: String,
    pub click_mode: ClickMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ClickElementData {
    pub element_id: String,
    pub action_performed: bool,
    pub page_changed: bool,
    pub navigation_url: Option<String>,
    pub resulting_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct FocusElementInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub element_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct FocusElementData {
    pub element_id: String,
    pub focused: bool,
    pub element_role: Option<crate::page_model::ElementRole>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TypeIntoElementInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub element_id: String,
    pub text: String,
    pub text_entry_mode: TextEntryMode,
    pub submit_mode: TextEntrySubmitMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TypeIntoElementData {
    pub element_id: String,
    pub text_length: usize,
    pub value_after: Option<String>,
    pub accepted_input: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SubmitActiveFormInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub form_element_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SubmitActiveFormData {
    pub form_element_id: Option<String>,
    pub submitted: bool,
    pub page_changed: bool,
    pub navigation_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ExtractPageModelData {
    pub page_model: PageModel,
    pub region_count: usize,
    pub readable_region_count: usize,
    pub extraction_source: ExtractionSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SetTtsVoiceInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub voice: TtsVoiceName,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SetTtsVoiceData {
    pub voice: String,
    pub changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SetPlaybackVolumeInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub volume: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SetPlaybackVolumeData {
    pub playback_volume: f32,
    pub muted: bool,
    pub changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SetPlaybackSpeedInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SetPlaybackSpeedData {
    pub playback_speed: f32,
    pub changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SetBrowserVisibilityInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub mode: BrowserVisibilityMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SetBrowserVisibilityData {
    pub mode: BrowserVisibilityMode,
    pub changed: bool,
    pub supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct GetAgentStateInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub include_last_transcript: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct GetRuntimeStatusInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub include_provider_modes: bool,
}
