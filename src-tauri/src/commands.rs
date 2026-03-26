use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::audio_io::RuntimeAudioState;
use crate::browser::{BrowserVisibilityMode, LoadState, ScrollDirection, ScrollTarget};
use crate::config::{ProviderMode, MAX_PLAYBACK_SPEED, MAX_PLAYBACK_VOLUME, MIN_PLAYBACK_SPEED};
use crate::narration::NarrationCursor;
use crate::page_model::{ExtractionSource, InteractiveElement, PageModel, Rect};
use crate::state::{BrowserHistoryState, ListeningState};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ToolName {
    OpenUrl,
    GoBack,
    GoForward,
    ReloadPage,
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
    pub planner_provider_settings: PlannerProviderSettings,
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
    pub backend: Option<String>,
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
    pub backend: Option<String>,
    pub model_id: Option<String>,
    pub model_path: Option<String>,
    pub language: Option<String>,
    pub threads: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PlannerProviderSettings {
    pub active_mode: ProviderMode,
    pub available_modes: Vec<ProviderMode>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RemotePlannerSettings {
    pub profile_name: Option<String>,
    pub provider: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub api_key_reference: Option<String>,
    pub organization_reference: Option<String>,
    pub project: Option<String>,
    pub temperature_milli: Option<u16>,
    pub max_output_tokens: Option<u32>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RemoteTtsSettings {
    pub profile_name: Option<String>,
    pub provider: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub api_key_reference: Option<String>,
    pub organization_reference: Option<String>,
    pub project: Option<String>,
    pub voice: Option<String>,
    pub audio_format: Option<String>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RemoteAsrSettings {
    pub profile_name: Option<String>,
    pub provider: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub api_key_reference: Option<String>,
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

struct StepExecutionContext<'a> {
    request_id: String,
    intent_name: IntentName,
    selected_skills: Vec<String>,
    steps: &'a [PlannedStep],
    initial_step_id: String,
    block_side_effects_until_confirmation: bool,
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
    pub hard_reload: bool,
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
    pub full_page: bool,
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
    pub interrupt_current: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ReadNextRegionInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub interrupt_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ReadNextRegionData {
    pub cursor: NarrationCursor,
    pub region_id: Option<String>,
    pub speech_started: bool,
    pub reached_end: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ReadPreviousRegionInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub interrupt_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ReadPreviousRegionData {
    pub cursor: NarrationCursor,
    pub region_id: Option<String>,
    pub speech_started: bool,
    pub reached_start: bool,
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
    pub auto_stop: bool,
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
    pub visible_only: bool,
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
    pub visible_only: bool,
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
    pub double_click: bool,
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
    pub clear_first: bool,
    pub submit_after: bool,
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

pub fn execute_planned_step<E: DeterministicToolExecutor>(
    executor: &mut E,
    step: &PlannedStep,
) -> SerializedToolResult {
    match step.tool_name {
        ToolName::OpenUrl => {
            execute_serialized_tool(step, ToolName::OpenUrl, executor, |executor, input| {
                executor.execute_open_url(input)
            })
        }
        ToolName::GoBack => {
            execute_serialized_tool(step, ToolName::GoBack, executor, |executor, input| {
                executor.execute_go_back(input)
            })
        }
        ToolName::GoForward => {
            execute_serialized_tool(step, ToolName::GoForward, executor, |executor, input| {
                executor.execute_go_forward(input)
            })
        }
        ToolName::ReloadPage => {
            execute_serialized_tool(step, ToolName::ReloadPage, executor, |executor, input| {
                executor.execute_reload_page(input)
            })
        }
        ToolName::ScrollPage => {
            execute_serialized_tool(step, ToolName::ScrollPage, executor, |executor, input| {
                executor.execute_scroll_page(input)
            })
        }
        ToolName::CaptureScreenshot => execute_serialized_tool(
            step,
            ToolName::CaptureScreenshot,
            executor,
            |executor, input| executor.execute_capture_screenshot(input),
        ),
        ToolName::RunOcr => {
            execute_serialized_tool(step, ToolName::RunOcr, executor, |executor, input| {
                executor.execute_run_ocr(input)
            })
        }
        ToolName::MergeOcrIntoPageModel => execute_serialized_tool(
            step,
            ToolName::MergeOcrIntoPageModel,
            executor,
            |executor, input| executor.execute_merge_ocr_into_page_model(input),
        ),
        ToolName::ReadRegion => {
            execute_serialized_tool(step, ToolName::ReadRegion, executor, |executor, input| {
                executor.execute_read_region(input)
            })
        }
        ToolName::ReadNextRegion => execute_serialized_tool(
            step,
            ToolName::ReadNextRegion,
            executor,
            |executor, input| executor.execute_read_next_region(input),
        ),
        ToolName::ReadPreviousRegion => execute_serialized_tool(
            step,
            ToolName::ReadPreviousRegion,
            executor,
            |executor, input| executor.execute_read_previous_region(input),
        ),
        ToolName::StopSpeaking => {
            execute_serialized_tool(step, ToolName::StopSpeaking, executor, |executor, input| {
                executor.execute_stop_speaking(input)
            })
        }
        ToolName::StartListening => execute_serialized_tool(
            step,
            ToolName::StartListening,
            executor,
            |executor, input| executor.execute_start_listening(input),
        ),
        ToolName::StopListening => execute_serialized_tool(
            step,
            ToolName::StopListening,
            executor,
            |executor, input| executor.execute_stop_listening(input),
        ),
        ToolName::TranscribeCommand => execute_serialized_tool(
            step,
            ToolName::TranscribeCommand,
            executor,
            |executor, input| executor.execute_transcribe_command(input),
        ),
        ToolName::GetPageSnapshot => execute_serialized_tool(
            step,
            ToolName::GetPageSnapshot,
            executor,
            |executor, input| executor.execute_get_page_snapshot(input),
        ),
        ToolName::ListInteractiveElements => execute_serialized_tool(
            step,
            ToolName::ListInteractiveElements,
            executor,
            |executor, input| executor.execute_list_interactive_elements(input),
        ),
        ToolName::FindElement => {
            execute_serialized_tool(step, ToolName::FindElement, executor, |executor, input| {
                executor.execute_find_element(input)
            })
        }
        ToolName::ClickElement => {
            execute_serialized_tool(step, ToolName::ClickElement, executor, |executor, input| {
                executor.execute_click_element(input)
            })
        }
        ToolName::FocusElement => {
            execute_serialized_tool(step, ToolName::FocusElement, executor, |executor, input| {
                executor.execute_focus_element(input)
            })
        }
        ToolName::TypeIntoElement => execute_serialized_tool(
            step,
            ToolName::TypeIntoElement,
            executor,
            |executor, input| executor.execute_type_into_element(input),
        ),
        ToolName::SubmitActiveForm => execute_serialized_tool(
            step,
            ToolName::SubmitActiveForm,
            executor,
            |executor, input| executor.execute_submit_active_form(input),
        ),
        ToolName::ExtractPageModel => execute_serialized_tool(
            step,
            ToolName::ExtractPageModel,
            executor,
            |executor, input| executor.execute_extract_page_model(input),
        ),
        ToolName::SetTtsVoice => {
            execute_serialized_tool(step, ToolName::SetTtsVoice, executor, |executor, input| {
                executor.execute_set_tts_voice(input)
            })
        }
        ToolName::SetPlaybackVolume => execute_serialized_tool(
            step,
            ToolName::SetPlaybackVolume,
            executor,
            |executor, input| executor.execute_set_playback_volume(input),
        ),
        ToolName::SetPlaybackSpeed => execute_serialized_tool(
            step,
            ToolName::SetPlaybackSpeed,
            executor,
            |executor, input| executor.execute_set_playback_speed(input),
        ),
        ToolName::SetBrowserVisibility => execute_serialized_tool(
            step,
            ToolName::SetBrowserVisibility,
            executor,
            |executor, input| executor.execute_set_browser_visibility(input),
        ),
        ToolName::GetAgentState => execute_serialized_tool(
            step,
            ToolName::GetAgentState,
            executor,
            |executor, input| executor.execute_get_agent_state(input),
        ),
        ToolName::GetRuntimeStatus => execute_serialized_tool(
            step,
            ToolName::GetRuntimeStatus,
            executor,
            |executor, input| executor.execute_get_runtime_status(input),
        ),
        ToolName::ConfirmAction => execute_serialized_tool(
            step,
            ToolName::ConfirmAction,
            executor,
            |executor, input| executor.execute_confirm_action(input),
        ),
        ToolName::ReportResult => {
            execute_serialized_tool(step, ToolName::ReportResult, executor, |executor, input| {
                executor.execute_report_result(input)
            })
        }
    }
}

pub fn execute_planner_output<E: DeterministicToolExecutor>(
    executor: &mut E,
    request_id: String,
    planner_output: &PlannerOutput,
) -> ExecutionOutcome {
    execute_planner_output_with_runner(request_id, planner_output, |step| {
        execute_planned_step(executor, step)
    })
}

pub fn resume_after_confirmation<E: DeterministicToolExecutor>(
    executor: &mut E,
    pending_plan_execution: &PendingPlanExecutionState,
    confirmation_id: &str,
    confirmed: bool,
) -> ExecutionOutcome {
    resume_after_confirmation_with_runner(
        pending_plan_execution,
        confirmation_id,
        confirmed,
        |step| execute_planned_step(executor, step),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SkillFrontmatter {
    pub name: String,
    pub description: String,
    pub allowed_tools: Option<Vec<ToolName>>,
    pub intent_tags: Vec<String>,
    pub requires_confirmation: bool,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ElementSearchResult {
    pub query: String,
    pub matches: Vec<ElementCandidate>,
    pub elements: Vec<InteractiveElement>,
}

const MAX_SELECTED_PLANNER_SKILLS: usize = 3;
const MAX_INITIAL_PLAN_STEPS: usize = 5;
const BUNDLED_SKILLS_MARKDOWN: &str = include_str!("../../docs/SKILLS.md");
const DEFAULT_VOLUME_STEP: f32 = 0.10;
const SMALL_VOLUME_STEP: f32 = 0.05;
const LARGE_VOLUME_STEP: f32 = 0.20;
const DEFAULT_SPEED_STEP: f32 = 0.25;
const SMALL_SPEED_STEP: f32 = 0.10;
const LARGE_SPEED_STEP: f32 = 0.50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannerSkillSelection {
    pub active_skill_names: Vec<String>,
    pub relevant_skill_summaries: Vec<SkillSummary>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SkillSource {
    Project,
    User,
    Bundled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LoadedSkill {
    summary: SkillSummary,
    body: String,
    source: SkillSource,
}

pub fn registered_tools() -> Vec<AvailableTool> {
    use ToolName::*;

    [
        OpenUrl,
        GoBack,
        GoForward,
        ReloadPage,
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
    ]
    .into_iter()
    .map(|name| AvailableTool {
        input_schema_ref: format!("schema://tool-input/{name:?}"),
        output_schema_ref: format!("schema://tool-output/{name:?}"),
        description: format!("Deterministic tool contract for {name:?}."),
        name,
    })
    .collect()
}

pub fn planner_available_tools() -> Vec<AvailableTool> {
    registered_tools()
        .into_iter()
        .filter(|tool| is_plannable_tool(&tool.name))
        .collect()
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
                            "double_click": false
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
                            "double_click": false
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

pub fn build_planner_skill_selection(
    project_root: Option<&Path>,
    user_skill_root: Option<&Path>,
    transcript: &str,
    available_tools: &[AvailableTool],
) -> PlannerSkillSelection {
    let loaded_skills = discover_skills(project_root, user_skill_root, available_tools);
    let mut active_skill_names = loaded_skills
        .iter()
        .map(|skill| skill.summary.name.clone())
        .collect::<Vec<_>>();
    active_skill_names.sort();

    let inferred_intent = infer_intent_hint(transcript);
    let likely_tools = likely_tools_for_intent(&inferred_intent);
    let transcript_tokens = tokenize_text(transcript);

    let mut ranked_skills = loaded_skills
        .into_iter()
        .filter_map(|skill| {
            score_skill(&skill, &transcript_tokens, &inferred_intent, &likely_tools)
                .map(|score| (score, skill.summary))
        })
        .collect::<Vec<_>>();

    ranked_skills.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| left.1.name.cmp(&right.1.name))
    });

    let relevant_skill_summaries = ranked_skills
        .into_iter()
        .take(MAX_SELECTED_PLANNER_SKILLS)
        .map(|(_, summary)| summary)
        .collect();

    PlannerSkillSelection {
        active_skill_names,
        relevant_skill_summaries,
    }
}

pub fn infer_intent_hint(transcript: &str) -> IntentName {
    let normalized = normalize_transcript_for_routing(transcript);

    if normalized.is_empty() {
        return IntentName::Unknown;
    }

    if is_start_listening_phrase(&normalized) {
        return IntentName::StartListening;
    }
    if is_stop_listening_phrase(&normalized) {
        return IntentName::StopListening;
    }
    if is_transcribe_command_phrase(&normalized) {
        return IntentName::TranscribeCommand;
    }
    if is_back_history_query_phrase(&normalized)
        || is_forward_history_query_phrase(&normalized)
        || is_listening_query_phrase(&normalized)
        || is_speaking_query_phrase(&normalized)
        || is_browser_mode_query_phrase(&normalized)
    {
        return IntentName::GetStatus;
    }
    if is_go_back_phrase(&normalized) {
        return IntentName::GoBack;
    }
    if is_go_forward_phrase(&normalized) {
        return IntentName::GoForward;
    }
    if is_reload_page_phrase(&normalized) {
        return IntentName::ReloadPage;
    }
    if is_current_url_query_phrase(&normalized) || normalized.contains("what page") {
        return IntentName::GetCurrentUrl;
    }
    if is_status_query_phrase(&normalized)
        || is_history_query_phrase(&normalized)
        || is_listening_query_phrase(&normalized)
        || is_speaking_query_phrase(&normalized)
        || is_browser_mode_query_phrase(&normalized)
    {
        return IntentName::GetStatus;
    }
    if is_read_next_phrase(&normalized) {
        return IntentName::ReadNext;
    }
    if is_read_previous_phrase(&normalized) {
        return IntentName::ReadPrevious;
    }
    if is_read_title_phrase(&normalized) {
        return IntentName::ReadTitle;
    }
    if is_repeat_phrase(&normalized) {
        return IntentName::Repeat;
    }
    if is_stop_phrase(&normalized) {
        return IntentName::Stop;
    }
    if is_read_page_phrase(&normalized) {
        return IntentName::ReadPage;
    }
    if is_fill_and_submit_phrase(&normalized) || is_submit_form_phrase(&normalized) {
        return IntentName::SubmitForm;
    }
    if is_fill_input_phrase(&normalized) {
        return IntentName::FillInput;
    }
    if is_set_tts_voice_phrase(&normalized) {
        return IntentName::SetTtsVoice;
    }
    if normalized.contains("open ")
        || normalized.contains("go to ")
        || normalized.contains("visit ")
    {
        return IntentName::OpenUrl;
    }
    if normalized.contains("click ") || normalized.contains("press ") {
        return IntentName::ClickElement;
    }
    if normalized.contains("find ") {
        return IntentName::FindElement;
    }
    if normalized.contains("scroll ") {
        return IntentName::Scroll;
    }
    if is_volume_query_phrase(&normalized) {
        return IntentName::GetPlaybackVolume;
    }
    if is_speed_query_phrase(&normalized) {
        return IntentName::GetPlaybackSpeed;
    }
    if normalized.contains("volume")
        || normalized.contains("mute")
        || normalized.contains("quieter")
        || normalized.contains("louder")
    {
        return IntentName::SetPlaybackVolume;
    }
    if normalized.contains("speed")
        || normalized.contains("faster")
        || normalized.contains("slower")
    {
        return IntentName::SetPlaybackSpeed;
    }
    if is_browser_visibility_phrase(&normalized) {
        return IntentName::SetBrowserVisibility;
    }

    IntentName::Unknown
}

pub fn validate_planner_output(
    planner_output: &PlannerOutput,
    available_tools: &[AvailableTool],
    active_skill_names: &[String],
) -> Result<(), ToolError> {
    let available_tool_names = available_tools
        .iter()
        .map(|tool| tool.name.clone())
        .collect::<Vec<_>>();
    let active_skill_name_set = active_skill_names.iter().cloned().collect::<HashSet<_>>();

    if planner_output.steps.len() > MAX_INITIAL_PLAN_STEPS {
        return Err(invalid_planner_output(
            format!(
                "planner returned {} steps, exceeding the v1 maximum of {}",
                planner_output.steps.len(),
                MAX_INITIAL_PLAN_STEPS
            ),
            None,
        ));
    }

    match planner_output.status {
        PlannerStatus::Ready | PlannerStatus::NeedsConfirmation => {
            if planner_output.steps.is_empty() {
                return Err(invalid_planner_output(
                    "planner returned no executable steps for an executing status",
                    None,
                ));
            }
        }
        PlannerStatus::Blocked => {
            if !planner_output.steps.is_empty() {
                return Err(invalid_planner_output(
                    "blocked planner output must not include executable steps",
                    None,
                ));
            }
            if planner_output.blocked_reason.is_none() {
                return Err(invalid_planner_output(
                    "blocked planner output must include blocked_reason",
                    None,
                ));
            }
            if planner_output
                .user_message
                .as_ref()
                .is_none_or(|message| message.trim().is_empty())
            {
                return Err(invalid_planner_output(
                    "blocked planner output must include a non-empty user_message",
                    None,
                ));
            }
        }
        PlannerStatus::Complete => {
            if !planner_output.steps.is_empty() {
                return Err(invalid_planner_output(
                    "complete planner output must not include executable steps",
                    None,
                ));
            }
        }
    }

    validate_submit_confirmation_policy(planner_output)?;
    validate_confirmation_policy(planner_output)?;

    let mut seen_step_ids = HashSet::new();
    for step in &planner_output.steps {
        if step.step_id.trim().is_empty() {
            return Err(invalid_planner_output(
                "planner step ids must be non-empty",
                Some(serde_json::json!({ "tool_name": step.tool_name })),
            ));
        }
        if !seen_step_ids.insert(step.step_id.clone()) {
            return Err(invalid_planner_output(
                format!("planner returned duplicate step id '{}'", step.step_id),
                Some(serde_json::json!({ "step_id": step.step_id })),
            ));
        }
        if step.purpose.trim().is_empty() {
            return Err(invalid_planner_output(
                "planner steps must include a non-empty purpose",
                Some(serde_json::json!({ "step_id": step.step_id })),
            ));
        }
        if !available_tool_names
            .iter()
            .any(|tool_name| tool_name == &step.tool_name)
        {
            return Err(invalid_planner_output(
                format!("planner referenced unavailable tool {:?}", step.tool_name),
                Some(serde_json::json!({ "step_id": step.step_id })),
            ));
        }
        validate_planned_step_arguments(step)?;
    }

    for step in &planner_output.steps {
        validate_step_transition(&step.on_success, &seen_step_ids, &step.step_id)?;
        validate_step_transition(&step.on_failure, &seen_step_ids, &step.step_id)?;
    }

    for skill_name in &planner_output.selected_skills {
        if !active_skill_name_set.contains(skill_name) {
            return Err(invalid_planner_output(
                format!("planner selected unknown or ineligible skill '{skill_name}'"),
                None,
            ));
        }
    }

    Ok(())
}

fn validate_submit_confirmation_policy(planner_output: &PlannerOutput) -> Result<(), ToolError> {
    if planner_output.intent.name != IntentName::SubmitForm {
        return Ok(());
    }

    if planner_output.status != PlannerStatus::NeedsConfirmation {
        return Err(invalid_planner_output(
            "submit-form planner output must use NeedsConfirmation status",
            None,
        ));
    }

    if !planner_output.requires_confirmation {
        return Err(invalid_planner_output(
            "submit-form planner output must set requires_confirmation",
            None,
        ));
    }

    if planner_output
        .confirmation_reason
        .as_ref()
        .is_none_or(|reason| reason.trim().is_empty())
    {
        return Err(invalid_planner_output(
            "submit-form planner output must include confirmation_reason",
            None,
        ));
    }

    if planner_output
        .user_message
        .as_ref()
        .is_none_or(|message| message.trim().is_empty())
    {
        return Err(invalid_planner_output(
            "submit-form planner output must include user_message",
            None,
        ));
    }

    let has_confirm_action_step = planner_output
        .steps
        .iter()
        .any(|step| step.tool_name == ToolName::ConfirmAction);
    if !has_confirm_action_step {
        return Err(invalid_planner_output(
            "submit-form planner output must include a confirm_action step",
            None,
        ));
    }

    let requests_confirmation = planner_output.steps.iter().any(|step| {
        step.tool_name == ToolName::ConfirmAction
            && matches!(step.on_success, StepTransition::RequestConfirmation)
    });
    if !requests_confirmation {
        return Err(invalid_planner_output(
            "submit-form planner output must request confirmation from a confirm_action step",
            None,
        ));
    }

    Ok(())
}

fn validate_confirmation_policy(planner_output: &PlannerOutput) -> Result<(), ToolError> {
    let has_confirm_action_step = planner_output
        .steps
        .iter()
        .any(|step| step.tool_name == ToolName::ConfirmAction);
    let requests_confirmation = planner_output.steps.iter().any(|step| {
        step.tool_name == ToolName::ConfirmAction
            && matches!(step.on_success, StepTransition::RequestConfirmation)
    });
    let has_confirmation_reason = planner_output
        .confirmation_reason
        .as_ref()
        .is_some_and(|reason| !reason.trim().is_empty());

    if planner_output.status == PlannerStatus::NeedsConfirmation {
        if !planner_output.requires_confirmation {
            return Err(invalid_planner_output(
                "needs-confirmation planner output must set requires_confirmation",
                None,
            ));
        }
        if !has_confirmation_reason {
            return Err(invalid_planner_output(
                "needs-confirmation planner output must include confirmation_reason",
                None,
            ));
        }
        if planner_output
            .user_message
            .as_ref()
            .is_none_or(|message| message.trim().is_empty())
        {
            return Err(invalid_planner_output(
                "needs-confirmation planner output must include user_message",
                None,
            ));
        }
        if !has_confirm_action_step {
            return Err(invalid_planner_output(
                "needs-confirmation planner output must include a confirm_action step",
                None,
            ));
        }
        if !requests_confirmation {
            return Err(invalid_planner_output(
                "needs-confirmation planner output must request confirmation from a confirm_action step",
                None,
            ));
        }
        return Ok(());
    }

    if planner_output.requires_confirmation {
        return Err(invalid_planner_output(
            "non-needs-confirmation planner output must not set requires_confirmation",
            None,
        ));
    }
    if has_confirmation_reason {
        return Err(invalid_planner_output(
            "non-needs-confirmation planner output must not include confirmation_reason",
            None,
        ));
    }
    if has_confirm_action_step {
        return Err(invalid_planner_output(
            "non-needs-confirmation planner output must not include a confirm_action step",
            None,
        ));
    }
    if requests_confirmation {
        return Err(invalid_planner_output(
            "non-needs-confirmation planner output must not request confirmation",
            None,
        ));
    }

    Ok(())
}

fn is_plannable_tool(tool_name: &ToolName) -> bool {
    matches!(
        tool_name,
        ToolName::OpenUrl
            | ToolName::GoBack
            | ToolName::GoForward
            | ToolName::ReloadPage
            | ToolName::ScrollPage
            | ToolName::CaptureScreenshot
            | ToolName::RunOcr
            | ToolName::MergeOcrIntoPageModel
            | ToolName::SetBrowserVisibility
            | ToolName::GetPageSnapshot
            | ToolName::ExtractPageModel
            | ToolName::ListInteractiveElements
            | ToolName::FindElement
            | ToolName::ClickElement
            | ToolName::FocusElement
            | ToolName::TypeIntoElement
            | ToolName::SubmitActiveForm
            | ToolName::ReadRegion
            | ToolName::ReadNextRegion
            | ToolName::ReadPreviousRegion
            | ToolName::StopSpeaking
            | ToolName::StartListening
            | ToolName::StopListening
            | ToolName::TranscribeCommand
            | ToolName::SetTtsVoice
            | ToolName::SetPlaybackVolume
            | ToolName::SetPlaybackSpeed
            | ToolName::GetAgentState
            | ToolName::GetRuntimeStatus
            | ToolName::ConfirmAction
            | ToolName::ReportResult
    )
}

fn schema_json<T: JsonSchema>() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(T)).expect("schema generation should serialize")
}

fn validate_planned_step_arguments(step: &PlannedStep) -> Result<(), ToolError> {
    match step.tool_name {
        ToolName::OpenUrl => validate_open_url_input(&deserialize_tool_arguments(step)?),
        ToolName::GoBack => validate_tool_arguments::<GoBackInput>(step),
        ToolName::GoForward => validate_tool_arguments::<GoForwardInput>(step),
        ToolName::ReloadPage => validate_tool_arguments::<ReloadPageInput>(step),
        ToolName::ScrollPage => validate_tool_arguments::<ScrollPageInput>(step),
        ToolName::CaptureScreenshot => {
            let input = deserialize_tool_arguments::<CaptureScreenshotInput>(step)?;
            validate_capture_screenshot_input(&input)
        }
        ToolName::RunOcr => {
            let input = deserialize_tool_arguments::<RunOcrInput>(step)?;
            validate_run_ocr_input(&input)
        }
        ToolName::MergeOcrIntoPageModel => {
            let input = deserialize_tool_arguments::<MergeOcrIntoPageModelInput>(step)?;
            validate_merge_ocr_into_page_model_input(&input)
        }
        ToolName::SetBrowserVisibility => {
            validate_tool_arguments::<SetBrowserVisibilityInput>(step)
        }
        ToolName::GetPageSnapshot => validate_tool_arguments::<GetPageSnapshotInput>(step),
        ToolName::ExtractPageModel => validate_tool_arguments::<ExtractPageModelInput>(step),
        ToolName::ListInteractiveElements => {
            validate_tool_arguments::<ListInteractiveElementsInput>(step)
        }
        ToolName::FindElement => validate_find_element_input(&deserialize_tool_arguments(step)?),
        ToolName::ClickElement => validate_click_element_input(&deserialize_tool_arguments(step)?),
        ToolName::FocusElement => validate_focus_element_input(&deserialize_tool_arguments(step)?),
        ToolName::TypeIntoElement => {
            validate_type_into_element_input(&deserialize_tool_arguments(step)?)
        }
        ToolName::SubmitActiveForm => {
            validate_submit_active_form_input(&deserialize_tool_arguments(step)?)
        }
        ToolName::ReadRegion => validate_read_region_input(&deserialize_tool_arguments(step)?),
        ToolName::ReadNextRegion => validate_tool_arguments::<ReadNextRegionInput>(step),
        ToolName::ReadPreviousRegion => validate_tool_arguments::<ReadPreviousRegionInput>(step),
        ToolName::StopSpeaking => validate_tool_arguments::<StopSpeakingInput>(step),
        ToolName::StartListening => validate_tool_arguments::<StartListeningInput>(step),
        ToolName::StopListening => validate_tool_arguments::<StopListeningInput>(step),
        ToolName::TranscribeCommand => {
            validate_transcribe_command_input(&deserialize_tool_arguments(step)?)
        }
        ToolName::SetTtsVoice => validate_set_tts_voice_input(&deserialize_tool_arguments(step)?),
        ToolName::SetPlaybackVolume => {
            validate_set_playback_volume_input(&deserialize_tool_arguments(step)?)
        }
        ToolName::SetPlaybackSpeed => {
            validate_set_playback_speed_input(&deserialize_tool_arguments(step)?)
        }
        ToolName::GetAgentState => validate_tool_arguments::<GetAgentStateInput>(step),
        ToolName::GetRuntimeStatus => validate_tool_arguments::<GetRuntimeStatusInput>(step),
        ToolName::ConfirmAction => {
            validate_confirm_action_input(&deserialize_tool_arguments(step)?)
        }
        ToolName::ReportResult => validate_report_result_input(&deserialize_tool_arguments(step)?),
    }
}

fn deserialize_tool_arguments<T: serde::de::DeserializeOwned>(
    step: &PlannedStep,
) -> Result<T, ToolError> {
    serde_json::from_value::<T>(step.arguments.clone()).map_err(|error| {
        invalid_planner_output(
            format!("tool arguments did not match the expected schema: {error}"),
            Some(serde_json::json!({
                "step_id": step.step_id,
                "tool_name": step.tool_name,
            })),
        )
    })
}

fn validate_open_url_input(input: &OpenUrlInput) -> Result<(), ToolError> {
    if input.url.trim().is_empty() {
        return Err(invalid_planner_output(
            "open_url requires a non-empty url",
            None,
        ));
    }

    Ok(())
}

fn validate_capture_screenshot_input(input: &CaptureScreenshotInput) -> Result<(), ToolError> {
    let region_id_active = input
        .region_id
        .as_deref()
        .map(str::trim)
        .is_some_and(|region_id| !region_id.is_empty());
    let targeting_modes = usize::from(input.full_page)
        + usize::from(region_id_active)
        + usize::from(input.bbox.is_some());
    if targeting_modes > 1 {
        return Err(invalid_planner_output(
            "capture_screenshot supports at most one targeting mode from full_page, region_id, or bbox",
            None,
        ));
    }

    if let Some(bbox) = input.bbox.as_ref() {
        if bbox.width <= 0.0 || bbox.height <= 0.0 {
            return Err(invalid_planner_output(
                "capture_screenshot bbox requires positive width and height",
                Some(serde_json::json!({
                    "x": bbox.x,
                    "y": bbox.y,
                    "width": bbox.width,
                    "height": bbox.height,
                })),
            ));
        }
    }

    Ok(())
}

fn validate_run_ocr_input(input: &RunOcrInput) -> Result<(), ToolError> {
    let image_id_active = input
        .image_id
        .as_deref()
        .map(str::trim)
        .is_some_and(|image_id| !image_id.is_empty());
    let region_id_active = input
        .region_id
        .as_deref()
        .map(str::trim)
        .is_some_and(|region_id| !region_id.is_empty());

    if !(image_id_active || region_id_active || input.bbox.is_some()) {
        return Err(invalid_planner_output(
            "run_ocr requires at least one source from image_id, region_id, or bbox",
            None,
        ));
    }

    if let Some(bbox) = input.bbox.as_ref() {
        if bbox.width <= 0.0 || bbox.height <= 0.0 {
            return Err(invalid_planner_output(
                "run_ocr bbox requires positive width and height",
                Some(serde_json::json!({
                    "x": bbox.x,
                    "y": bbox.y,
                    "width": bbox.width,
                    "height": bbox.height,
                })),
            ));
        }
    }

    Ok(())
}

fn validate_merge_ocr_into_page_model_input(
    input: &MergeOcrIntoPageModelInput,
) -> Result<(), ToolError> {
    if input.page_id.trim().is_empty() {
        return Err(invalid_planner_output(
            "merge_ocr_into_page_model requires a non-empty page_id",
            None,
        ));
    }

    if input.ocr_text.trim().is_empty() {
        return Err(invalid_planner_output(
            "merge_ocr_into_page_model requires non-empty ocr_text",
            None,
        ));
    }

    if let Some(region_id) = input.region_id.as_deref() {
        if region_id.trim().is_empty() {
            return Err(invalid_planner_output(
                "merge_ocr_into_page_model region_id must be non-empty when provided",
                None,
            ));
        }
    }

    if let Some(bbox) = input.source_bbox.as_ref() {
        if bbox.width <= 0.0 || bbox.height <= 0.0 {
            return Err(invalid_planner_output(
                "merge_ocr_into_page_model source_bbox requires positive width and height",
                Some(serde_json::json!({
                    "x": bbox.x,
                    "y": bbox.y,
                    "width": bbox.width,
                    "height": bbox.height,
                })),
            ));
        }
    }

    Ok(())
}

fn validate_find_element_input(input: &FindElementInput) -> Result<(), ToolError> {
    if input.description.trim().is_empty() {
        return Err(invalid_planner_output(
            "find_element requires a non-empty description",
            None,
        ));
    }

    for (field_name, value) in [
        ("text", input.text.as_deref()),
        ("color_hint", input.color_hint.as_deref()),
        ("nearby_text", input.nearby_text.as_deref()),
        ("selector_hint", input.selector_hint.as_deref()),
    ] {
        if let Some(value) = value {
            if value.trim().is_empty() {
                return Err(invalid_planner_output(
                    format!("find_element {field_name} must be non-empty when provided"),
                    None,
                ));
            }
        }
    }

    if matches!(input.max_candidates, Some(0)) {
        return Err(invalid_planner_output(
            "find_element max_candidates must be greater than 0 when provided",
            None,
        ));
    }

    Ok(())
}

fn validate_click_element_input(input: &ClickElementInput) -> Result<(), ToolError> {
    if input.element_id.trim().is_empty() {
        return Err(invalid_planner_output(
            "click_element requires a non-empty element_id",
            None,
        ));
    }

    Ok(())
}

fn validate_focus_element_input(input: &FocusElementInput) -> Result<(), ToolError> {
    if input.element_id.trim().is_empty() {
        return Err(invalid_planner_output(
            "focus_element requires a non-empty element_id",
            None,
        ));
    }

    Ok(())
}

fn validate_type_into_element_input(input: &TypeIntoElementInput) -> Result<(), ToolError> {
    if input.element_id.trim().is_empty() {
        return Err(invalid_planner_output(
            "type_into_element requires a non-empty element_id",
            None,
        ));
    }

    Ok(())
}

fn validate_submit_active_form_input(input: &SubmitActiveFormInput) -> Result<(), ToolError> {
    if let Some(form_element_id) = input.form_element_id.as_deref() {
        if form_element_id.trim().is_empty() {
            return Err(invalid_planner_output(
                "submit_active_form form_element_id must be non-empty when provided",
                None,
            ));
        }
    }

    Ok(())
}

fn validate_read_region_input(input: &ReadRegionInput) -> Result<(), ToolError> {
    if input.region_id.trim().is_empty() {
        return Err(invalid_planner_output(
            "read_region requires a non-empty region_id",
            None,
        ));
    }

    Ok(())
}

fn validate_transcribe_command_input(input: &TranscribeCommandInput) -> Result<(), ToolError> {
    if matches!(input.max_duration_ms, Some(0)) {
        return Err(invalid_planner_output(
            "transcribe_command max_duration_ms must be greater than 0 when provided",
            None,
        ));
    }

    Ok(())
}

fn validate_set_tts_voice_input(_input: &SetTtsVoiceInput) -> Result<(), ToolError> {
    Ok(())
}

fn validate_set_playback_volume_input(input: &SetPlaybackVolumeInput) -> Result<(), ToolError> {
    if !input.volume.is_finite() {
        return Err(invalid_planner_output(
            "set_playback_volume requires a finite numeric volume value",
            None,
        ));
    }

    if !(0.0..=MAX_PLAYBACK_VOLUME).contains(&input.volume) {
        return Err(invalid_planner_output(
            format!("set_playback_volume volume must be between 0.0 and {MAX_PLAYBACK_VOLUME}"),
            Some(serde_json::json!({ "volume": input.volume })),
        ));
    }

    Ok(())
}

fn validate_set_playback_speed_input(input: &SetPlaybackSpeedInput) -> Result<(), ToolError> {
    if !input.speed.is_finite() {
        return Err(invalid_planner_output(
            "set_playback_speed requires a finite numeric speed value",
            None,
        ));
    }

    if !(MIN_PLAYBACK_SPEED..=MAX_PLAYBACK_SPEED).contains(&input.speed) {
        return Err(invalid_planner_output(
            format!(
                "set_playback_speed speed must be between {MIN_PLAYBACK_SPEED} and {MAX_PLAYBACK_SPEED}"
            ),
            Some(serde_json::json!({ "speed": input.speed })),
        ));
    }

    Ok(())
}

fn validate_confirm_action_input(input: &ConfirmActionInput) -> Result<(), ToolError> {
    if input.prompt_text.trim().is_empty() {
        return Err(invalid_planner_output(
            "confirm_action requires a non-empty prompt_text",
            None,
        ));
    }

    if input.reason.trim().is_empty() {
        return Err(invalid_planner_output(
            "confirm_action requires a non-empty reason",
            None,
        ));
    }

    Ok(())
}

fn validate_report_result_input(input: &ReportResultInput) -> Result<(), ToolError> {
    if input.summary.trim().is_empty() {
        return Err(invalid_planner_output(
            "report_result requires a non-empty summary",
            None,
        ));
    }

    for (field_name, value) in [
        (
            "next_recommended_action",
            input.next_recommended_action.as_deref(),
        ),
        ("user_message", input.user_message.as_deref()),
    ] {
        if let Some(value) = value {
            if value.trim().is_empty() {
                return Err(invalid_planner_output(
                    format!("report_result {field_name} must be non-empty when provided"),
                    None,
                ));
            }
        }
    }

    Ok(())
}

fn validate_step_transition(
    transition: &StepTransition,
    step_ids: &HashSet<String>,
    source_step_id: &str,
) -> Result<(), ToolError> {
    if let StepTransition::NextStep { step_id } = transition {
        if !step_ids.contains(step_id) {
            return Err(invalid_planner_output(
                format!(
                    "planner referenced missing next step '{}' from '{}'",
                    step_id, source_step_id
                ),
                Some(serde_json::json!({
                    "source_step_id": source_step_id,
                    "next_step_id": step_id,
                })),
            ));
        }
    }

    Ok(())
}

fn invalid_planner_output(
    message: impl Into<String>,
    details: Option<serde_json::Value>,
) -> ToolError {
    ToolError {
        code: String::from("invalid_planner_output"),
        message: message.into(),
        retryable: false,
        details,
    }
}

fn discover_skills(
    project_root: Option<&Path>,
    user_skill_root: Option<&Path>,
    available_tools: &[AvailableTool],
) -> Vec<LoadedSkill> {
    let available_tool_names = available_tools
        .iter()
        .map(|tool| tool.name.clone())
        .collect::<Vec<_>>();
    let mut discovered = HashMap::<String, LoadedSkill>::new();

    if let Some(project_root) = project_root {
        load_skills_from_directory(
            &project_root.join(".pi").join("skills"),
            SkillSource::Project,
            &available_tool_names,
            &mut discovered,
        );
    }

    if let Some(user_skill_root) = user_skill_root {
        load_skills_from_directory(
            user_skill_root,
            SkillSource::User,
            &available_tool_names,
            &mut discovered,
        );
    }

    for skill in parse_bundled_skills(BUNDLED_SKILLS_MARKDOWN, &available_tool_names) {
        discovered
            .entry(skill.summary.name.clone())
            .or_insert(skill);
    }

    discovered.into_values().collect()
}

fn load_skills_from_directory(
    skill_root: &Path,
    source: SkillSource,
    available_tool_names: &[ToolName],
    discovered: &mut HashMap<String, LoadedSkill>,
) {
    let entries = match fs::read_dir(skill_root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => {
            tracing::warn!(
                path = %skill_root.display(),
                error = %error,
                "failed to read skill directory"
            );
            return;
        }
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let skill_file_path = path.join("SKILL.md");
        let content = match fs::read_to_string(&skill_file_path) {
            Ok(content) => content,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                tracing::warn!(
                    path = %skill_file_path.display(),
                    error = %error,
                    "failed to read SKILL.md"
                );
                continue;
            }
        };

        match parse_skill_document(&content, source, available_tool_names) {
            Ok(skill) => {
                let directory_name = path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default();
                if directory_name != skill.summary.name {
                    tracing::warn!(
                        path = %skill_file_path.display(),
                        expected = directory_name,
                        actual = %skill.summary.name,
                        "skipping skill because directory name does not match frontmatter name"
                    );
                    continue;
                }
                discovered
                    .entry(skill.summary.name.clone())
                    .or_insert(skill);
            }
            Err(error) => {
                tracing::warn!(
                    path = %skill_file_path.display(),
                    error = %error,
                    "skipping invalid skill document"
                );
            }
        }
    }
}

fn parse_skill_document(
    content: &str,
    source: SkillSource,
    available_tool_names: &[ToolName],
) -> Result<LoadedSkill, String> {
    let normalized = content.replace("\r\n", "\n");
    let Some(frontmatter_body) = normalized.strip_prefix("---\n") else {
        return Err(String::from("SKILL.md is missing a YAML frontmatter block"));
    };
    let Some(split_index) = frontmatter_body.find("\n---\n") else {
        return Err(String::from("SKILL.md frontmatter block is not terminated"));
    };

    let frontmatter_block = &frontmatter_body[..split_index];
    let body = frontmatter_body[(split_index + 5)..].trim().to_string();
    let frontmatter = parse_skill_frontmatter(frontmatter_block, available_tool_names)?;
    Ok(LoadedSkill {
        summary: skill_summary_from_frontmatter(frontmatter),
        body,
        source,
    })
}

fn parse_skill_frontmatter(
    block: &str,
    available_tool_names: &[ToolName],
) -> Result<SkillFrontmatter, String> {
    let mut scalar_fields = HashMap::<String, String>::new();
    let mut list_fields = HashMap::<String, Vec<String>>::new();
    let mut active_list_key: Option<String> = None;

    for raw_line in block.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(list_key) = active_list_key.as_ref() {
            if let Some(item) = trimmed.strip_prefix("- ") {
                list_fields
                    .entry(list_key.clone())
                    .or_default()
                    .push(clean_skill_value(item));
                continue;
            }
            active_list_key = None;
        }

        let Some((key, value)) = trimmed.split_once(':') else {
            return Err(format!("invalid frontmatter line '{trimmed}'"));
        };
        let key = normalize_skill_key(key.trim());
        let value = value.trim();

        match key.as_str() {
            "name" | "description" | "requires_confirmation" | "priority" => {
                scalar_fields.insert(key, clean_skill_value(value));
            }
            "allowed_tools" | "intent_tags" => {
                let list = list_fields.entry(key.clone()).or_default();
                list.extend(parse_inline_list(value));
                active_list_key = Some(key);
            }
            _ => return Err(format!("unsupported frontmatter field '{key}'")),
        }
    }

    skill_frontmatter_from_parts(scalar_fields, list_fields, available_tool_names)
}

fn parse_bundled_skills(markdown: &str, available_tool_names: &[ToolName]) -> Vec<LoadedSkill> {
    let mut current_name: Option<String> = None;
    let mut description = String::new();
    let mut intent_tags = Vec::new();
    let mut allowed_tools = Vec::new();
    let mut skills = Vec::new();

    let flush_skill = |skills: &mut Vec<LoadedSkill>,
                       current_name: &mut Option<String>,
                       description: &mut String,
                       intent_tags: &mut Vec<String>,
                       allowed_tools: &mut Vec<String>,
                       requires_confirmation: bool| {
        let Some(name) = current_name.take() else {
            return;
        };

        let mut scalar_fields = HashMap::new();
        scalar_fields.insert(String::from("name"), name);
        scalar_fields.insert(String::from("description"), description.trim().to_string());
        scalar_fields.insert(
            String::from("requires_confirmation"),
            requires_confirmation.to_string(),
        );
        let mut list_fields = HashMap::new();
        list_fields.insert(String::from("intent_tags"), intent_tags.clone());
        list_fields.insert(String::from("allowed_tools"), allowed_tools.clone());

        match skill_frontmatter_from_parts(scalar_fields, list_fields, available_tool_names) {
            Ok(frontmatter) => skills.push(LoadedSkill {
                summary: skill_summary_from_frontmatter(frontmatter),
                body: description.trim().to_string(),
                source: SkillSource::Bundled,
            }),
            Err(error) => {
                tracing::warn!(skill_name = %skills.last().map(|skill| skill.summary.name.as_str()).unwrap_or("unknown"), error = %error, "skipping invalid bundled skill");
            }
        }

        description.clear();
        intent_tags.clear();
        allowed_tools.clear();
    };

    let mut requires_confirmation_value = false;
    for raw_line in markdown.lines() {
        let trimmed = raw_line.trim();
        if let Some(name) = trimmed.strip_prefix("#### ") {
            flush_skill(
                &mut skills,
                &mut current_name,
                &mut description,
                &mut intent_tags,
                &mut allowed_tools,
                requires_confirmation_value,
            );
            current_name = Some(name.trim().to_string());
            requires_confirmation_value = false;
            continue;
        }

        if current_name.is_none() {
            continue;
        }

        if let Some(value) = trimmed.strip_prefix("- intent_tags:") {
            intent_tags = parse_backticked_list(value);
        } else if let Some(value) = trimmed.strip_prefix("- allowed_tools:") {
            allowed_tools = parse_backticked_list(value);
        } else if let Some(value) = trimmed.strip_prefix("- requires_confirmation:") {
            requires_confirmation_value = parse_bool_value(value).unwrap_or(false);
        } else if let Some(value) = trimmed.strip_prefix("- description:") {
            description = clean_skill_value(value);
        }
    }

    flush_skill(
        &mut skills,
        &mut current_name,
        &mut description,
        &mut intent_tags,
        &mut allowed_tools,
        requires_confirmation_value,
    );

    skills
}

fn skill_frontmatter_from_parts(
    scalar_fields: HashMap<String, String>,
    list_fields: HashMap<String, Vec<String>>,
    available_tool_names: &[ToolName],
) -> Result<SkillFrontmatter, String> {
    let name = scalar_fields
        .get("name")
        .ok_or_else(|| String::from("skill frontmatter is missing name"))?
        .trim()
        .to_string();
    if !is_valid_skill_name(&name) {
        return Err(format!("invalid skill name '{name}'"));
    }

    let description = scalar_fields
        .get("description")
        .ok_or_else(|| String::from("skill frontmatter is missing description"))?
        .trim()
        .to_string();
    if description.is_empty() {
        return Err(String::from("skill description must not be empty"));
    }

    let mut intent_tags = list_fields
        .get("intent_tags")
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();
    intent_tags.sort();
    intent_tags.dedup();
    for tag in &intent_tags {
        if let Some(intent_name) = tag.strip_prefix("intent:") {
            parse_intent_name_value(intent_name)?;
        }
    }

    let allowed_tools = match list_fields.get("allowed_tools") {
        Some(tool_names) if !tool_names.is_empty() => {
            let mut resolved_tools = Vec::new();
            for tool_name in tool_names {
                let tool = parse_tool_name_value(tool_name)?;
                if !available_tool_names
                    .iter()
                    .any(|available| available == &tool)
                {
                    return Err(format!("skill references unavailable tool '{tool_name}'"));
                }
                resolved_tools.push(tool);
            }
            Some(resolved_tools)
        }
        _ => None,
    };

    let requires_confirmation = scalar_fields
        .get("requires_confirmation")
        .map(|value| parse_bool_value(value))
        .transpose()?
        .unwrap_or(false);

    let priority = scalar_fields
        .get("priority")
        .map(|value| {
            value
                .parse::<i32>()
                .map_err(|error| format!("invalid priority value '{value}': {error}"))
        })
        .transpose()?
        .unwrap_or(0);

    Ok(SkillFrontmatter {
        name,
        description,
        allowed_tools,
        intent_tags,
        requires_confirmation,
        priority,
    })
}

fn skill_summary_from_frontmatter(frontmatter: SkillFrontmatter) -> SkillSummary {
    SkillSummary {
        name: frontmatter.name,
        description: frontmatter.description,
        intent_tags: frontmatter.intent_tags,
        allowed_tools: frontmatter.allowed_tools,
        requires_confirmation: frontmatter.requires_confirmation,
        priority: frontmatter.priority,
    }
}

fn normalize_skill_key(key: &str) -> String {
    key.trim().replace('-', "_")
}

fn parse_inline_list(value: &str) -> Vec<String> {
    let cleaned = clean_skill_value(value);
    if cleaned.is_empty() {
        return Vec::new();
    }

    let trimmed = cleaned.trim_matches(['[', ']']);
    trimmed
        .split(',')
        .map(clean_skill_value)
        .filter(|item| !item.is_empty())
        .collect()
}

fn parse_backticked_list(value: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current = String::new();
    let mut in_tick = false;
    for character in value.chars() {
        match character {
            '`' => {
                if in_tick {
                    items.push(current.trim().to_string());
                    current.clear();
                }
                in_tick = !in_tick;
            }
            _ if in_tick => current.push(character),
            _ => {}
        }
    }

    if items.is_empty() {
        parse_inline_list(value)
    } else {
        items
    }
}

fn parse_bool_value(value: &str) -> Result<bool, String> {
    match clean_skill_value(value).as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(format!("invalid boolean value '{other}'")),
    }
}

fn clean_skill_value(value: &str) -> String {
    value
        .trim()
        .trim_matches('`')
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

fn is_valid_skill_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || character == '-'
                || character == '_'
        })
}

fn parse_tool_name_value(value: &str) -> Result<ToolName, String> {
    match clean_skill_value(value).as_str() {
        "open_url" => Ok(ToolName::OpenUrl),
        "go_back" => Ok(ToolName::GoBack),
        "go_forward" => Ok(ToolName::GoForward),
        "reload_page" => Ok(ToolName::ReloadPage),
        "scroll_page" => Ok(ToolName::ScrollPage),
        "capture_screenshot" => Ok(ToolName::CaptureScreenshot),
        "set_browser_visibility" => Ok(ToolName::SetBrowserVisibility),
        "get_page_snapshot" => Ok(ToolName::GetPageSnapshot),
        "extract_page_model" => Ok(ToolName::ExtractPageModel),
        "list_interactive_elements" => Ok(ToolName::ListInteractiveElements),
        "find_element" => Ok(ToolName::FindElement),
        "click_element" => Ok(ToolName::ClickElement),
        "focus_element" => Ok(ToolName::FocusElement),
        "type_into_element" => Ok(ToolName::TypeIntoElement),
        "submit_active_form" => Ok(ToolName::SubmitActiveForm),
        "read_region" => Ok(ToolName::ReadRegion),
        "read_next_region" => Ok(ToolName::ReadNextRegion),
        "read_previous_region" => Ok(ToolName::ReadPreviousRegion),
        "stop_speaking" => Ok(ToolName::StopSpeaking),
        "start_listening" => Ok(ToolName::StartListening),
        "stop_listening" => Ok(ToolName::StopListening),
        "transcribe_command" => Ok(ToolName::TranscribeCommand),
        "set_tts_voice" => Ok(ToolName::SetTtsVoice),
        "set_playback_volume" => Ok(ToolName::SetPlaybackVolume),
        "set_playback_speed" => Ok(ToolName::SetPlaybackSpeed),
        "run_ocr" => Ok(ToolName::RunOcr),
        "merge_ocr_into_page_model" => Ok(ToolName::MergeOcrIntoPageModel),
        "get_agent_state" => Ok(ToolName::GetAgentState),
        "get_runtime_status" => Ok(ToolName::GetRuntimeStatus),
        "confirm_action" => Ok(ToolName::ConfirmAction),
        "report_result" => Ok(ToolName::ReportResult),
        other => Err(format!("unknown tool '{other}'")),
    }
}

fn parse_intent_name_value(value: &str) -> Result<IntentName, String> {
    match clean_skill_value(value).as_str() {
        "OpenUrl" => Ok(IntentName::OpenUrl),
        "GoBack" => Ok(IntentName::GoBack),
        "GoForward" => Ok(IntentName::GoForward),
        "ReloadPage" => Ok(IntentName::ReloadPage),
        "GetCurrentUrl" => Ok(IntentName::GetCurrentUrl),
        "ReadPage" => Ok(IntentName::ReadPage),
        "ReadTitle" => Ok(IntentName::ReadTitle),
        "ReadNext" => Ok(IntentName::ReadNext),
        "ReadPrevious" => Ok(IntentName::ReadPrevious),
        "Repeat" => Ok(IntentName::Repeat),
        "Stop" => Ok(IntentName::Stop),
        "StartListening" => Ok(IntentName::StartListening),
        "StopListening" => Ok(IntentName::StopListening),
        "TranscribeCommand" => Ok(IntentName::TranscribeCommand),
        "SetTtsVoice" => Ok(IntentName::SetTtsVoice),
        "SetPlaybackVolume" => Ok(IntentName::SetPlaybackVolume),
        "GetPlaybackVolume" => Ok(IntentName::GetPlaybackVolume),
        "SetPlaybackSpeed" => Ok(IntentName::SetPlaybackSpeed),
        "GetPlaybackSpeed" => Ok(IntentName::GetPlaybackSpeed),
        "SetBrowserVisibility" => Ok(IntentName::SetBrowserVisibility),
        "GetStatus" => Ok(IntentName::GetStatus),
        "FindElement" => Ok(IntentName::FindElement),
        "ClickElement" => Ok(IntentName::ClickElement),
        "FillInput" => Ok(IntentName::FillInput),
        "SubmitForm" => Ok(IntentName::SubmitForm),
        "Scroll" => Ok(IntentName::Scroll),
        "OcrRecovery" => Ok(IntentName::OcrRecovery),
        "Unknown" => Ok(IntentName::Unknown),
        other => Err(format!("unknown intent tag '{other}'")),
    }
}

pub(crate) fn normalize_transcript_for_routing(transcript: &str) -> String {
    normalize_command_text(transcript, false)
}

fn normalize_audio_command_text(transcript: &str) -> String {
    normalize_command_text(transcript, true)
}

fn normalize_command_text(transcript: &str, allow_decimal: bool) -> String {
    let sanitized = transcript
        .to_ascii_lowercase()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric()
                || character.is_ascii_whitespace()
                || (allow_decimal && character == '.')
            {
                character
            } else {
                ' '
            }
        })
        .collect::<String>();

    merge_compound_command_tokens(
        sanitized
            .split_whitespace()
            .map(String::from)
            .collect::<Vec<_>>(),
    )
    .into_iter()
    .map(|token| canonicalize_command_token(&token))
    .collect::<Vec<_>>()
    .join(" ")
}

fn merge_compound_command_tokens(tokens: Vec<String>) -> Vec<String> {
    let mut merged = Vec::with_capacity(tokens.len());
    let mut index = 0;

    while index < tokens.len() {
        if let Some(next_token) = tokens.get(index + 1) {
            match (tokens[index].as_str(), next_token.as_str()) {
                ("play", "back") => {
                    merged.push(String::from("playback"));
                    index += 2;
                    continue;
                }
                ("head", "less") => {
                    merged.push(String::from("headless"));
                    index += 2;
                    continue;
                }
                _ => {}
            }
        }

        merged.push(tokens[index].clone());
        index += 1;
    }

    merged
}

fn canonicalize_command_token(token: &str) -> String {
    const FUZZY_COMMAND_KEYWORDS: &[&str] = &[
        "back",
        "browser",
        "current",
        "field",
        "focus",
        "forward",
        "headless",
        "listening",
        "next",
        "playback",
        "previous",
        "reload",
        "refresh",
        "repeat",
        "speed",
        "status",
        "stop",
        "submit",
        "title",
        "transcribe",
        "url",
        "visible",
        "voice",
        "volume",
    ];

    if token.len() < 4
        || token.contains('.')
        || token.chars().any(|character| character.is_ascii_digit())
    {
        return token.to_string();
    }

    let mut matches = FUZZY_COMMAND_KEYWORDS
        .iter()
        .copied()
        .filter(|keyword| is_unambiguous_fuzzy_keyword_match(token, keyword));

    match (matches.next(), matches.next()) {
        (Some(keyword), None) => String::from(keyword),
        _ => token.to_string(),
    }
}

fn is_unambiguous_fuzzy_keyword_match(token: &str, keyword: &str) -> bool {
    if token == keyword {
        return true;
    }

    if token.len().abs_diff(keyword.len()) > 1 {
        return false;
    }

    if token
        .chars()
        .next()
        .zip(keyword.chars().next())
        .is_none_or(|(left, right)| left != right)
    {
        return false;
    }

    is_single_edit_or_transposition(token, keyword)
}

fn is_single_edit_or_transposition(left: &str, right: &str) -> bool {
    let left_chars = left.chars().collect::<Vec<_>>();
    let right_chars = right.chars().collect::<Vec<_>>();

    match left_chars.len().cmp(&right_chars.len()) {
        std::cmp::Ordering::Equal => {
            let mismatches = left_chars
                .iter()
                .zip(right_chars.iter())
                .enumerate()
                .filter_map(|(index, (left_char, right_char))| {
                    (left_char != right_char).then_some(index)
                })
                .collect::<Vec<_>>();

            match mismatches.as_slice() {
                [single_mismatch] => left_chars[*single_mismatch] != right_chars[*single_mismatch],
                [first, second] if *second == *first + 1 => {
                    left_chars[*first] == right_chars[*second]
                        && left_chars[*second] == right_chars[*first]
                }
                _ => false,
            }
        }
        std::cmp::Ordering::Less => is_single_insertion_or_deletion(&left_chars, &right_chars),
        std::cmp::Ordering::Greater => is_single_insertion_or_deletion(&right_chars, &left_chars),
    }
}

fn is_single_insertion_or_deletion(shorter: &[char], longer: &[char]) -> bool {
    let mut shorter_index = 0;
    let mut longer_index = 0;
    let mut skipped = false;

    while shorter_index < shorter.len() && longer_index < longer.len() {
        if shorter[shorter_index] == longer[longer_index] {
            shorter_index += 1;
            longer_index += 1;
            continue;
        }

        if skipped {
            return false;
        }

        skipped = true;
        longer_index += 1;
    }

    true
}

fn tokenize_text(text: &str) -> HashSet<String> {
    normalize_transcript_for_routing(text)
        .split_whitespace()
        .filter(|token| token.len() > 1)
        .map(String::from)
        .collect()
}

fn likely_tools_for_intent(intent: &IntentName) -> Vec<ToolName> {
    match intent {
        IntentName::OpenUrl => vec![ToolName::OpenUrl],
        IntentName::GoBack => vec![ToolName::GoBack],
        IntentName::GoForward => vec![ToolName::GoForward],
        IntentName::ReloadPage => vec![ToolName::ReloadPage],
        IntentName::GetCurrentUrl => vec![ToolName::GetAgentState, ToolName::ReportResult],
        IntentName::ReadPage => vec![ToolName::ExtractPageModel, ToolName::ReadRegion],
        IntentName::ReadTitle => vec![ToolName::ReportResult],
        IntentName::ReadNext => vec![ToolName::ReadNextRegion],
        IntentName::ReadPrevious => vec![ToolName::ReadPreviousRegion],
        IntentName::Repeat => vec![ToolName::GetAgentState, ToolName::ReadRegion],
        IntentName::Stop => vec![ToolName::StopSpeaking],
        IntentName::StartListening => vec![ToolName::StartListening],
        IntentName::StopListening => vec![ToolName::StopListening],
        IntentName::TranscribeCommand => vec![ToolName::TranscribeCommand],
        IntentName::SetTtsVoice => vec![ToolName::SetTtsVoice, ToolName::ReportResult],
        IntentName::SetPlaybackVolume => vec![ToolName::SetPlaybackVolume],
        IntentName::SetPlaybackSpeed => vec![ToolName::SetPlaybackSpeed],
        IntentName::GetPlaybackVolume | IntentName::GetPlaybackSpeed => {
            vec![ToolName::GetRuntimeStatus, ToolName::ReportResult]
        }
        IntentName::SetBrowserVisibility => vec![ToolName::SetBrowserVisibility],
        IntentName::GetStatus => vec![ToolName::GetRuntimeStatus, ToolName::ReportResult],
        IntentName::FindElement => vec![ToolName::FindElement],
        IntentName::ClickElement => vec![ToolName::FindElement, ToolName::ClickElement],
        IntentName::FillInput => vec![ToolName::FocusElement, ToolName::TypeIntoElement],
        IntentName::SubmitForm => vec![ToolName::ConfirmAction, ToolName::SubmitActiveForm],
        IntentName::Scroll => vec![ToolName::ScrollPage],
        IntentName::OcrRecovery => vec![
            ToolName::CaptureScreenshot,
            ToolName::RunOcr,
            ToolName::MergeOcrIntoPageModel,
            ToolName::GetPageSnapshot,
            ToolName::ReportResult,
        ],
        IntentName::Unknown => Vec::new(),
    }
}

pub(crate) fn resolve_direct_audio_command(
    transcript: &str,
    request_id: &str,
    current_volume: f32,
    current_speed: f32,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_audio_command_text(transcript);
    if normalized.is_empty() {
        return None;
    }

    if is_volume_query_phrase(&normalized) {
        let summary = format!(
            "Playback volume is {}.",
            format_playback_volume(current_volume)
        );
        return Some(build_audio_report_planner_output(
            request_id,
            IntentName::GetPlaybackVolume,
            String::from("Report the current playback volume."),
            selected_audio_skill(active_skill_names, "get_volume"),
            Some(format_playback_volume(current_volume)),
            summary,
        ));
    }

    if is_speed_query_phrase(&normalized) {
        let summary = format!(
            "Playback speed is {}.",
            format_playback_speed(current_speed)
        );
        return Some(build_audio_report_planner_output(
            request_id,
            IntentName::GetPlaybackSpeed,
            String::from("Report the current playback speed."),
            selected_audio_skill(active_skill_names, "get_playback_speed"),
            Some(format_playback_speed(current_speed)),
            summary,
        ));
    }

    if let Some(volume) = parse_volume_command(&normalized, current_volume) {
        let summary = format!(
            "Playback volume set to {}.",
            format_playback_volume(volume.value)
        );
        return Some(build_audio_set_planner_output(AudioSetPlanSpec {
            request_id,
            intent_name: IntentName::SetPlaybackVolume,
            goal: volume.goal,
            selected_skills: selected_audio_skill(active_skill_names, volume.skill_name),
            target_description: Some(format_playback_volume(volume.value)),
            set_step_id: "set-playback-volume",
            tool_name: ToolName::SetPlaybackVolume,
            tool_arguments: serde_json::json!({
                "request_id": request_id,
                "timeout_ms": serde_json::Value::Null,
                "volume": volume.value
            }),
            tool_purpose: String::from("Apply and persist the requested playback volume."),
            report_step_id: "report-playback-volume",
            report_summary: summary,
        }));
    }

    if let Some(speed) = parse_speed_command(&normalized, current_speed) {
        let summary = format!(
            "Playback speed set to {}.",
            format_playback_speed(speed.value)
        );
        return Some(build_audio_set_planner_output(AudioSetPlanSpec {
            request_id,
            intent_name: IntentName::SetPlaybackSpeed,
            goal: speed.goal,
            selected_skills: selected_audio_skill(active_skill_names, speed.skill_name),
            target_description: Some(format_playback_speed(speed.value)),
            set_step_id: "set-playback-speed",
            tool_name: ToolName::SetPlaybackSpeed,
            tool_arguments: serde_json::json!({
                "request_id": request_id,
                "timeout_ms": serde_json::Value::Null,
                "speed": speed.value
            }),
            tool_purpose: String::from("Apply and persist the requested playback speed."),
            report_step_id: "report-playback-speed",
            report_summary: summary,
        }));
    }

    None
}

pub(crate) fn resolve_direct_browser_visibility_command(
    transcript: &str,
    request_id: &str,
    current_visibility: BrowserVisibilityMode,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() {
        return None;
    }

    let target_mode = parse_browser_visibility_command(&normalized, current_visibility)?;
    let summary = format!(
        "Browser mode set to {}.",
        format_browser_visibility_mode(target_mode)
    );

    Some(build_browser_visibility_planner_output(
        request_id,
        target_mode,
        selected_skill(active_skill_names, "toggle_browser_visibility"),
        summary,
    ))
}

pub(crate) fn resolve_direct_navigation_readback_command(
    transcript: &str,
    request_id: &str,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() {
        return None;
    }

    if is_go_back_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::GoBack,
                goal: String::from("Navigate back one entry in browser history."),
                target_description: None,
            },
            selected_skill(active_skill_names, "go_back"),
            PlannedStep {
                step_id: String::from("go-back"),
                tool_name: ToolName::GoBack,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "steps": 1,
                    "wait_for_load_state": LoadState::Load
                }),
                purpose: String::from("Move back to the previous history entry."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    if is_go_forward_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::GoForward,
                goal: String::from("Navigate forward one entry in browser history."),
                target_description: None,
            },
            selected_skill(active_skill_names, "go_forward"),
            PlannedStep {
                step_id: String::from("go-forward"),
                tool_name: ToolName::GoForward,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "steps": 1,
                    "wait_for_load_state": LoadState::Load
                }),
                purpose: String::from("Move forward to the next history entry."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    if is_reload_page_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::ReloadPage,
                goal: String::from("Reload the current page."),
                target_description: Some(String::from("current page")),
            },
            selected_skill(active_skill_names, "reload_page"),
            PlannedStep {
                step_id: String::from("reload-page"),
                tool_name: ToolName::ReloadPage,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "hard_reload": false,
                    "wait_for_load_state": LoadState::Load
                }),
                purpose: String::from("Reload the current page and wait for it to finish loading."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    if is_read_next_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::ReadNext,
                goal: String::from("Read the next narration region."),
                target_description: Some(String::from("next narration region")),
            },
            selected_skill(active_skill_names, "read_next"),
            PlannedStep {
                step_id: String::from("read-next-region"),
                tool_name: ToolName::ReadNextRegion,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "interrupt_current": true
                }),
                purpose: String::from("Move narration to the next region and start reading it."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    if is_read_previous_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::ReadPrevious,
                goal: String::from("Read the previous narration region."),
                target_description: Some(String::from("previous narration region")),
            },
            selected_skill(active_skill_names, "read_previous"),
            PlannedStep {
                step_id: String::from("read-previous-region"),
                tool_name: ToolName::ReadPreviousRegion,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "interrupt_current": true
                }),
                purpose: String::from(
                    "Move narration to the previous region and start reading it.",
                ),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    if is_stop_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::Stop,
                goal: String::from("Stop current speech output."),
                target_description: Some(String::from("speech output")),
            },
            selected_stop_skill(active_skill_names),
            PlannedStep {
                step_id: String::from("stop-speaking"),
                tool_name: ToolName::StopSpeaking,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null
                }),
                purpose: String::from("Stop any current spoken narration or playback."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    None
}

pub(crate) fn resolve_direct_voice_input_command(
    transcript: &str,
    request_id: &str,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() {
        return None;
    }

    if is_start_listening_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::StartListening,
                goal: String::from("Start listening for voice input."),
                target_description: Some(String::from("voice input")),
            },
            selected_skill(active_skill_names, "start_listening"),
            PlannedStep {
                step_id: String::from("start-listening"),
                tool_name: ToolName::StartListening,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null
                }),
                purpose: String::from("Start listening for the next spoken command."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    if is_stop_listening_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::StopListening,
                goal: String::from("Stop listening for voice input."),
                target_description: Some(String::from("voice input")),
            },
            selected_skill(active_skill_names, "stop_listening"),
            PlannedStep {
                step_id: String::from("stop-listening"),
                tool_name: ToolName::StopListening,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null
                }),
                purpose: String::from("Stop active voice listening."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    if is_transcribe_command_phrase(&normalized) {
        return Some(build_single_step_planner_output(
            IntentSummary {
                name: IntentName::TranscribeCommand,
                goal: String::from("Capture and transcribe a short spoken command."),
                target_description: Some(String::from("spoken command")),
            },
            selected_skill(active_skill_names, "transcribe_command"),
            PlannedStep {
                step_id: String::from("transcribe-command"),
                tool_name: ToolName::TranscribeCommand,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "max_duration_ms": serde_json::Value::Null,
                    "auto_stop": true
                }),
                purpose: String::from("Capture and transcribe a bounded spoken command."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ));
    }

    None
}

pub(crate) fn resolve_direct_open_url_command(
    transcript: &str,
    request_id: &str,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let url = parse_direct_open_url_target(transcript)?;

    Some(build_single_step_planner_output(
        IntentSummary {
            name: IntentName::OpenUrl,
            goal: String::from("Open the requested URL."),
            target_description: Some(url.clone()),
        },
        selected_skill(active_skill_names, "open_url"),
        PlannedStep {
            step_id: String::from("open-url"),
            tool_name: ToolName::OpenUrl,
            arguments: serde_json::json!({
                "request_id": request_id,
                "timeout_ms": serde_json::Value::Null,
                "url": url,
                "wait_for_load_state": LoadState::Load
            }),
            purpose: String::from("Open the requested URL and wait for the page to load."),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
    ))
}

pub(crate) fn resolve_direct_read_page_command(
    transcript: &str,
    request_id: &str,
    current_page: Option<&PageModel>,
    agent_state: &AgentStateData,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() || !is_read_page_phrase(&normalized) {
        return None;
    }

    let selected_skills = selected_skill(active_skill_names, "read_page");

    if agent_state.page_id.is_none() {
        let summary = String::from("There is no current page to read yet.");
        return Some(PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::ReadPage,
                goal: String::from("Read the current page from the beginning."),
                target_description: Some(String::from("current page")),
            },
            selected_skills,
            steps: vec![PlannedStep {
                step_id: String::from("report-missing-page"),
                tool_name: ToolName::ReportResult,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "status": ReportStatus::NeedsFollowUp,
                    "summary": summary.clone(),
                    "next_recommended_action": "Open a page first, then ask me to read it.",
                    "user_message": summary
                }),
                purpose: String::from("Report that there is no active page available to read."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        });
    }

    if let Some(region_id) = current_page.and_then(first_readable_region_id) {
        return Some(PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::ReadPage,
                goal: String::from("Read the current page from the beginning."),
                target_description: Some(String::from("current page")),
            },
            selected_skills,
            steps: vec![PlannedStep {
                step_id: String::from("read-page-from-start"),
                tool_name: ToolName::ReadRegion,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "region_id": region_id,
                    "interrupt_current": true
                }),
                purpose: String::from(
                    "Restart narration from the first readable region of the current page.",
                ),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        });
    }

    Some(PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::ReadPage,
            goal: String::from("Read the current page from the beginning."),
            target_description: Some(String::from("current page")),
        },
        selected_skills,
        steps: vec![
            PlannedStep {
                step_id: String::from("extract-page-for-reading"),
                tool_name: ToolName::ExtractPageModel,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "use_dom_extraction": true,
                    "include_headings": true,
                    "include_links": true
                }),
                purpose: String::from("Refresh the readable page model before starting narration."),
                on_success: StepTransition::NextStep {
                    step_id: String::from("read-first-region"),
                },
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from("read-first-region"),
                tool_name: ToolName::ReadNextRegion,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "interrupt_current": true
                }),
                purpose: String::from(
                    "Start narration from the first readable region of the refreshed page.",
                ),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    })
}

pub(crate) fn resolve_direct_status_query_command(
    transcript: &str,
    request_id: &str,
    agent_state: &AgentStateData,
    runtime_status: &GetRuntimeStatusData,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() {
        return None;
    }

    if is_current_url_query_phrase(&normalized) {
        let summary = format_current_url_summary(agent_state);
        return Some(build_status_query_planner_output(StatusQueryPlanSpec {
            request_id,
            intent_name: IntentName::GetCurrentUrl,
            goal: String::from("Report the current page URL and title."),
            selected_skills: selected_skill(active_skill_names, "get_current_url"),
            target_description: Some(current_page_label(agent_state)),
            read_step_id: "get-current-url",
            read_tool_name: ToolName::GetAgentState,
            read_tool_arguments: serde_json::json!({
                "request_id": request_id,
                "include_last_transcript": false
            }),
            read_tool_purpose: String::from("Read the current agent page state."),
            report_step_id: "report-current-url",
            report_summary: summary,
        }));
    }

    if is_status_query_phrase(&normalized)
        || is_history_query_phrase(&normalized)
        || is_listening_query_phrase(&normalized)
        || is_speaking_query_phrase(&normalized)
        || is_browser_mode_query_phrase(&normalized)
    {
        let summary = if is_back_history_query_phrase(&normalized) {
            format_back_history_summary(runtime_status)
        } else if is_forward_history_query_phrase(&normalized) {
            format_forward_history_summary(runtime_status)
        } else if is_listening_query_phrase(&normalized) {
            format_listening_summary(runtime_status)
        } else if is_speaking_query_phrase(&normalized) {
            format_speaking_summary(runtime_status)
        } else if is_browser_mode_query_phrase(&normalized) {
            format_browser_mode_summary(runtime_status)
        } else {
            format_runtime_status_summary(runtime_status)
        };

        return Some(build_status_query_planner_output(StatusQueryPlanSpec {
            request_id,
            intent_name: IntentName::GetStatus,
            goal: String::from("Report the current runtime status relevant to the user's query."),
            selected_skills: selected_status_skill(active_skill_names),
            target_description: Some(String::from("runtime status")),
            read_step_id: "get-runtime-status",
            read_tool_name: ToolName::GetRuntimeStatus,
            read_tool_arguments: serde_json::json!({
                "request_id": request_id,
                "include_provider_modes": false
            }),
            read_tool_purpose: String::from("Read the current runtime status."),
            report_step_id: "report-runtime-status",
            report_summary: summary,
        }));
    }

    None
}

pub(crate) fn resolve_direct_read_title_command(
    transcript: &str,
    request_id: &str,
    agent_state: &AgentStateData,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() || !is_read_title_phrase(&normalized) {
        return None;
    }

    let summary = match normalized_optional_text(agent_state.title.as_deref()) {
        Some(title) => format!("Page title is {title}."),
        None => String::from("This page does not have a readable title yet."),
    };

    Some(PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::ReadTitle,
            goal: String::from("Read the current page title."),
            target_description: Some(String::from("current page title")),
        },
        selected_skills: selected_skill(active_skill_names, "read_title"),
        steps: vec![PlannedStep {
            step_id: String::from("report-page-title"),
            tool_name: ToolName::ReportResult,
            arguments: serde_json::json!({
                "request_id": request_id,
                "timeout_ms": serde_json::Value::Null,
                "status": ReportStatus::Success,
                "summary": summary.clone(),
                "next_recommended_action": serde_json::Value::Null,
                "user_message": summary
            }),
            purpose: String::from("Speak the current page title."),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    })
}

pub(crate) fn resolve_direct_repeat_command(
    transcript: &str,
    request_id: &str,
    agent_state: &AgentStateData,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() || !is_repeat_phrase(&normalized) {
        return None;
    }

    let selected_skills = selected_skill(active_skill_names, "repeat");
    let Some(region_id) = agent_state
        .narration_cursor
        .as_ref()
        .and_then(|cursor| cursor.current_region_id.as_deref())
    else {
        let summary = String::from("There is no current region to repeat yet.");
        return Some(PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::Repeat,
                goal: String::from("Repeat the current narration region."),
                target_description: Some(String::from("current narration region")),
            },
            selected_skills,
            steps: vec![PlannedStep {
                step_id: String::from("report-missing-repeat-region"),
                tool_name: ToolName::ReportResult,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "status": ReportStatus::NeedsFollowUp,
                    "summary": summary.clone(),
                    "next_recommended_action": "Read the page or move to a region first.",
                    "user_message": summary
                }),
                purpose: String::from(
                    "Report that no current narration region is available to repeat.",
                ),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        });
    };

    Some(PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::Repeat,
            goal: String::from("Repeat the current narration region."),
            target_description: Some(String::from("current narration region")),
        },
        selected_skills,
        steps: vec![PlannedStep {
            step_id: String::from("repeat-current-region"),
            tool_name: ToolName::ReadRegion,
            arguments: serde_json::json!({
                "request_id": request_id,
                "timeout_ms": serde_json::Value::Null,
                "region_id": region_id,
                "interrupt_current": true
            }),
            purpose: String::from(
                "Repeat the current narration region from the stored narration cursor.",
            ),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    })
}

#[derive(Debug, Clone, PartialEq)]
struct NormalizedAudioSetting {
    value: f32,
    goal: String,
    skill_name: &'static str,
}

fn parse_volume_command(normalized: &str, current_volume: f32) -> Option<NormalizedAudioSetting> {
    if normalized == "mute" || normalized.contains("mute volume") {
        return Some(NormalizedAudioSetting {
            value: 0.0,
            goal: String::from("Set playback volume to muted."),
            skill_name: "mute_volume",
        });
    }

    if let Some(step) = volume_relative_step(normalized) {
        let target = (current_volume + step).clamp(0.0, MAX_PLAYBACK_VOLUME);
        let goal = if step.is_sign_positive() {
            String::from("Increase playback volume by the requested normalized step.")
        } else {
            String::from("Decrease playback volume by the requested normalized step.")
        };
        let skill_name = if step.is_sign_positive() {
            "increase_volume"
        } else {
            "decrease_volume"
        };
        return Some(NormalizedAudioSetting {
            value: round_audio_setting_value(target),
            goal,
            skill_name,
        });
    }

    if !normalized.contains("volume") {
        return None;
    }

    parse_absolute_volume_value(normalized).map(|value| NormalizedAudioSetting {
        value: round_audio_setting_value(value.clamp(0.0, MAX_PLAYBACK_VOLUME)),
        goal: String::from("Set playback volume to the requested normalized value."),
        skill_name: "set_volume",
    })
}

fn parse_speed_command(normalized: &str, current_speed: f32) -> Option<NormalizedAudioSetting> {
    if let Some(step) = speed_relative_step(normalized) {
        let target = (current_speed + step).clamp(MIN_PLAYBACK_SPEED, MAX_PLAYBACK_SPEED);
        let goal = if step.is_sign_positive() {
            String::from("Increase playback speed by the requested normalized step.")
        } else {
            String::from("Decrease playback speed by the requested normalized step.")
        };
        let skill_name = if step.is_sign_positive() {
            "increase_playback_speed"
        } else {
            "decrease_playback_speed"
        };
        return Some(NormalizedAudioSetting {
            value: round_audio_setting_value(target),
            goal,
            skill_name,
        });
    }

    if !normalized.contains("speed") {
        return None;
    }

    parse_absolute_speed_value(normalized).map(|value| NormalizedAudioSetting {
        value: round_audio_setting_value(value.clamp(MIN_PLAYBACK_SPEED, MAX_PLAYBACK_SPEED)),
        goal: String::from("Set playback speed to the requested normalized value."),
        skill_name: "set_playback_speed",
    })
}

fn parse_absolute_volume_value(normalized: &str) -> Option<f32> {
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();

    for (index, token) in tokens.iter().enumerate() {
        let Ok(value) = token.parse::<f32>() else {
            continue;
        };

        if tokens.get(index + 1).copied() == Some("percent") {
            return Some(value / 100.0);
        }
        if value.fract() == 0.0 && (0.0..=100.0).contains(&value) {
            return Some(value / 100.0);
        }
        return Some(value);
    }

    None
}

fn parse_absolute_speed_value(normalized: &str) -> Option<f32> {
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();

    for (index, token) in tokens.iter().enumerate() {
        if let Some(multiplier) = parse_multiplier_token(token) {
            return Some(multiplier);
        }

        let Ok(value) = token.parse::<f32>() else {
            continue;
        };

        if matches!(tokens.get(index + 1).copied(), Some("times") | Some("time")) {
            return Some(value);
        }
        if tokens.get(index + 1).copied() == Some("percent") {
            return Some(value / 100.0);
        }
        return Some(value);
    }

    None
}

fn parse_multiplier_token(token: &str) -> Option<f32> {
    token
        .strip_suffix('x')
        .and_then(|value| (!value.is_empty()).then_some(value))
        .and_then(|value| value.parse::<f32>().ok())
}

fn volume_relative_step(normalized: &str) -> Option<f32> {
    if normalized.contains("increase volume")
        || normalized.contains("turn it up")
        || normalized.contains("volume up")
        || normalized.contains("louder")
    {
        return Some(volume_step_size(normalized));
    }

    if normalized.contains("decrease volume")
        || normalized.contains("turn it down")
        || normalized.contains("volume down")
        || normalized.contains("quieter")
    {
        return Some(-volume_step_size(normalized));
    }

    None
}

fn speed_relative_step(normalized: &str) -> Option<f32> {
    if normalized.contains("increase playback speed")
        || normalized.contains("speed up")
        || normalized.contains("go faster")
        || normalized == "faster"
    {
        return Some(speed_step_size(normalized));
    }

    if normalized.contains("decrease playback speed")
        || normalized.contains("slow down")
        || normalized.contains("go slower")
        || normalized == "slower"
    {
        return Some(-speed_step_size(normalized));
    }

    None
}

fn volume_step_size(normalized: &str) -> f32 {
    if normalized.contains("a little") || normalized.contains("slightly") {
        SMALL_VOLUME_STEP
    } else if normalized.contains("a lot") || normalized.contains("much") {
        LARGE_VOLUME_STEP
    } else {
        DEFAULT_VOLUME_STEP
    }
}

fn speed_step_size(normalized: &str) -> f32 {
    if normalized.contains("a little") || normalized.contains("slightly") {
        SMALL_SPEED_STEP
    } else if normalized.contains("a lot") || normalized.contains("much") {
        LARGE_SPEED_STEP
    } else {
        DEFAULT_SPEED_STEP
    }
}

fn is_volume_query_phrase(normalized: &str) -> bool {
    normalized.contains("what is the volume")
        || normalized.contains("what s the volume")
        || normalized.contains("current volume")
        || normalized.contains("tell me the volume")
}

fn is_speed_query_phrase(normalized: &str) -> bool {
    normalized.contains("what is the playback speed")
        || normalized.contains("what s the playback speed")
        || normalized.contains("current playback speed")
        || normalized.contains("what speed am i on")
        || normalized.contains("tell me the speed")
}

fn is_current_url_query_phrase(normalized: &str) -> bool {
    normalized.contains("current url")
        || normalized.contains("what page am i on")
        || normalized.contains("what page is this")
        || normalized.contains("what site am i on")
        || normalized.contains("where is this page")
}

fn is_go_back_phrase(normalized: &str) -> bool {
    normalized == "back" || normalized.contains("go back")
}

fn is_start_listening_phrase(normalized: &str) -> bool {
    normalized.contains("start listening")
        || normalized.contains("listen now")
        || normalized.contains("begin listening")
}

fn is_stop_listening_phrase(normalized: &str) -> bool {
    normalized.contains("stop listening")
        || normalized.contains("stop listen")
        || normalized.contains("quit listening")
}

fn is_transcribe_command_phrase(normalized: &str) -> bool {
    normalized.contains("transcribe")
        || normalized.contains("what did i say")
        || normalized.contains("what did i just say")
}

fn is_read_page_phrase(normalized: &str) -> bool {
    normalized == "read page"
        || normalized == "read this page"
        || normalized == "read current page"
        || normalized == "start reading page"
        || normalized == "start reading this page"
}

fn parse_direct_open_url_target(transcript: &str) -> Option<String> {
    let trimmed = transcript.trim();
    let lowercase = trimmed.to_ascii_lowercase();

    let raw_target = ["open ", "go to ", "visit "]
        .iter()
        .find_map(|prefix| {
            lowercase
                .strip_prefix(prefix)
                .map(|_| &trimmed[prefix.len()..])
        })?
        .trim()
        .trim_matches(|character: char| matches!(character, '.' | ',' | ';' | ':' | '"' | '\''));

    normalize_spoken_url_target(raw_target)
}

fn normalize_spoken_url_target(target: &str) -> Option<String> {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.eq_ignore_ascii_case("about blank") || trimmed.eq_ignore_ascii_case("about:blank") {
        return Some(String::from("about:blank"));
    }

    if trimmed.contains("://") || trimmed.to_ascii_lowercase().starts_with("about:") {
        return Some(trimmed.split_whitespace().collect());
    }

    if looks_like_host_without_scheme(trimmed) {
        return Some(prepend_default_scheme(trimmed));
    }

    let normalized = normalize_transcript_for_routing(trimmed);
    if normalized.is_empty() {
        return None;
    }

    let mut rebuilt = String::new();
    for token in normalized.split_whitespace() {
        match token {
            "dot" => rebuilt.push('.'),
            "slash" => rebuilt.push('/'),
            "colon" => rebuilt.push(':'),
            "dash" | "hyphen" => rebuilt.push('-'),
            "underscore" => rebuilt.push('_'),
            other => rebuilt.push_str(other),
        }
    }

    looks_like_host_without_scheme(&rebuilt).then(|| prepend_default_scheme(&rebuilt))
}

fn looks_like_host_without_scheme(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.contains(' ') {
        return false;
    }

    trimmed == "localhost" || trimmed.starts_with("localhost:") || trimmed.contains('.')
}

fn prepend_default_scheme(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed == "localhost"
        || trimmed.starts_with("localhost:")
        || trimmed.starts_with("127.0.0.1")
        || trimmed.starts_with("0.0.0.0")
    {
        format!("http://{trimmed}")
    } else {
        format!("https://{trimmed}")
    }
}

fn first_readable_region_id(page_model: &PageModel) -> Option<String> {
    page_model
        .regions
        .iter()
        .find(|region| !region.text.trim().is_empty())
        .map(|region| region.region_id.clone())
}

pub(crate) fn parse_direct_focus_field_command(transcript: &str) -> Option<FocusFieldCommand> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() || !is_focus_field_phrase(&normalized) {
        return None;
    }

    let mut remainder = normalized.as_str();
    if let Some(stripped) = remainder.strip_prefix("focus ") {
        remainder = stripped;
    }
    if let Some(stripped) = remainder.strip_prefix("on ") {
        remainder = stripped;
    }
    if let Some(stripped) = remainder.strip_prefix("the ") {
        remainder = stripped;
    }
    if let Some(stripped) = remainder.strip_prefix("my ") {
        remainder = stripped;
    }
    if let Some(stripped) = remainder.strip_prefix("field ") {
        remainder = stripped;
    }
    if let Some(stripped) = remainder.strip_suffix(" field") {
        remainder = stripped;
    }

    let description = remainder.trim();
    Some(FocusFieldCommand {
        description: (!description.is_empty() && description != "field")
            .then(|| description.to_string()),
    })
}

pub(crate) fn parse_direct_fill_field_command(transcript: &str) -> Option<FillFieldCommand> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty()
        || !is_fill_input_phrase(&normalized)
        || is_focus_field_phrase(&normalized)
        || is_fill_and_submit_phrase(&normalized)
    {
        return None;
    }

    let collapsed = collapse_transcript_whitespace(transcript);
    if collapsed.is_empty() {
        return Some(FillFieldCommand {
            description: None,
            text: None,
        });
    }

    if let Some((description, text)) = parse_fill_with_pattern(&collapsed) {
        return Some(FillFieldCommand {
            description,
            text: Some(text),
        });
    }

    if let Some((text, description)) = parse_into_field_pattern(&collapsed) {
        return Some(FillFieldCommand {
            description,
            text: Some(text),
        });
    }

    Some(FillFieldCommand {
        description: parse_fill_field_description_only(&collapsed),
        text: None,
    })
}

pub(crate) fn parse_direct_fill_and_submit_command(transcript: &str) -> Option<FillFieldCommand> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() || !is_fill_and_submit_phrase(&normalized) {
        return None;
    }

    let collapsed = collapse_transcript_whitespace(transcript);
    let fill_portion = strip_fill_and_submit_suffix(&collapsed)?;
    parse_direct_fill_field_command(&fill_portion)
}

pub(crate) fn is_direct_submit_form_command(transcript: &str) -> bool {
    let normalized = normalize_transcript_for_routing(transcript);
    !normalized.is_empty() && is_submit_form_phrase(&normalized)
}

fn is_go_forward_phrase(normalized: &str) -> bool {
    normalized == "forward" || normalized.contains("go forward")
}

fn is_reload_page_phrase(normalized: &str) -> bool {
    normalized == "reload"
        || normalized == "refresh"
        || normalized.contains("reload page")
        || normalized.contains("refresh page")
}

fn is_status_query_phrase(normalized: &str) -> bool {
    normalized.contains("what is the status")
        || normalized.contains("what s the status")
        || normalized.contains("current status")
        || normalized.contains("status please")
        || normalized.contains("where am i")
}

fn is_history_query_phrase(normalized: &str) -> bool {
    is_back_history_query_phrase(normalized) || is_forward_history_query_phrase(normalized)
}

fn is_back_history_query_phrase(normalized: &str) -> bool {
    normalized.contains("can i go back")
        || normalized.contains("can we go back")
        || normalized.contains("is back available")
        || normalized.contains("can go back")
        || normalized.contains("back available")
}

fn is_forward_history_query_phrase(normalized: &str) -> bool {
    normalized.contains("can i go forward")
        || normalized.contains("can we go forward")
        || normalized.contains("is forward available")
        || normalized.contains("can go forward")
        || normalized.contains("forward available")
}

fn is_listening_query_phrase(normalized: &str) -> bool {
    normalized.contains("are you listening")
        || normalized.contains("listening status")
        || normalized.contains("is listening on")
        || normalized.contains("am i listening")
}

fn is_speaking_query_phrase(normalized: &str) -> bool {
    normalized.contains("are you speaking")
        || normalized.contains("are you reading")
        || normalized.contains("is speech active")
        || normalized.contains("are you talking")
}

fn is_browser_mode_query_phrase(normalized: &str) -> bool {
    normalized.contains("browser mode")
        || normalized.contains("is the browser visible")
        || normalized.contains("is browser visible")
        || normalized.contains("is it headless")
        || normalized.contains("are we headless")
}

fn is_repeat_phrase(normalized: &str) -> bool {
    normalized == "repeat"
        || normalized.contains("repeat that")
        || normalized.contains("repeat this")
        || normalized.contains("repeat region")
        || normalized.contains("read that again")
        || normalized.contains("say that again")
        || normalized.contains("say this again")
}

fn is_read_next_phrase(normalized: &str) -> bool {
    normalized == "next"
        || normalized.contains("read next")
        || normalized.contains("next region")
        || normalized.contains("next section")
        || normalized.contains("continue reading")
        || normalized.contains("keep reading")
}

fn is_read_previous_phrase(normalized: &str) -> bool {
    normalized == "previous"
        || normalized.contains("read previous")
        || normalized.contains("previous region")
        || normalized.contains("previous section")
}

fn is_stop_phrase(normalized: &str) -> bool {
    normalized == "stop"
        || normalized.contains("stop reading")
        || normalized.contains("stop speaking")
        || normalized.contains("pause reading")
}

fn is_read_title_phrase(normalized: &str) -> bool {
    normalized == "read title"
        || normalized.contains("read the title")
        || normalized.contains("read page title")
        || normalized.contains("read the page title")
        || normalized.contains("what is the title")
        || normalized.contains("what s the title")
        || normalized.contains("tell me the title")
}

fn is_set_tts_voice_phrase(normalized: &str) -> bool {
    normalized.contains("voice")
        && (normalized.contains("set ")
            || normalized.contains("change ")
            || normalized.contains("switch ")
            || normalized.contains("use "))
}

fn is_fill_and_submit_phrase(normalized: &str) -> bool {
    (normalized.contains("fill ") || normalized.contains("enter ") || normalized.contains("type "))
        && normalized.contains("submit")
}

fn is_focus_field_phrase(normalized: &str) -> bool {
    normalized.contains("focus field")
        || (normalized.contains("focus ") && normalized.contains(" field"))
}

fn is_submit_form_phrase(normalized: &str) -> bool {
    normalized == "submit"
        || normalized.contains("submit form")
        || normalized.contains("submit this form")
        || normalized.contains("send form")
        || normalized.contains("send this form")
        || normalized.contains("press submit")
        || normalized.contains("hit submit")
}

fn is_fill_input_phrase(normalized: &str) -> bool {
    is_focus_field_phrase(normalized)
        || normalized.contains("fill in ")
        || (normalized.contains("fill ") && normalized.contains(" field"))
        || normalized.contains("type into ")
        || (normalized.contains("type ")
            && normalized.contains(" into ")
            && normalized.contains(" field"))
        || (normalized.contains("enter ") && normalized.contains(" field"))
        || (normalized.contains("put ") && normalized.contains(" field"))
        || (normalized.contains("choose ") && normalized.contains(" list"))
        || (normalized.contains("select ") && normalized.contains(" field"))
        || normalized.contains("the other field")
        || normalized.contains("there instead")
}

fn collapse_transcript_whitespace(transcript: &str) -> String {
    transcript.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn parse_fill_with_pattern(transcript: &str) -> Option<(Option<String>, String)> {
    let lowered = transcript.to_ascii_lowercase();
    let prefix = if lowered.starts_with("fill in ") {
        "fill in "
    } else if lowered.starts_with("fill ") {
        "fill "
    } else {
        return None;
    };

    let remainder = transcript.get(prefix.len()..)?.trim();
    let (field_target, text) = split_case_insensitive_once(remainder, " with ")?;
    let text = normalize_fill_value(text)?;
    Some((normalize_field_target(field_target), text))
}

fn parse_into_field_pattern(transcript: &str) -> Option<(String, Option<String>)> {
    let lowered = transcript.to_ascii_lowercase();
    let (prefix, separator) = if lowered.starts_with("type ") && lowered.contains(" into ") {
        ("type ", " into ")
    } else if lowered.starts_with("enter ") && lowered.contains(" into ") {
        ("enter ", " into ")
    } else if lowered.starts_with("enter ") && lowered.contains(" in ") {
        ("enter ", " in ")
    } else if lowered.starts_with("put ") && lowered.contains(" in ") {
        ("put ", " in ")
    } else {
        return None;
    };

    let remainder = transcript.get(prefix.len()..)?.trim();
    let (text, field_target) = split_case_insensitive_once(remainder, separator)?;
    let text = normalize_fill_value(text)?;
    Some((text, normalize_field_target(field_target)))
}

fn parse_fill_field_description_only(transcript: &str) -> Option<String> {
    let lowered = transcript.to_ascii_lowercase();
    let remainder = if lowered.starts_with("fill in ") {
        transcript.get("fill in ".len()..)?
    } else if lowered.starts_with("fill ") {
        transcript.get("fill ".len()..)?
    } else if lowered.starts_with("type into ") {
        transcript.get("type into ".len()..)?
    } else if lowered.starts_with("enter into ") {
        transcript.get("enter into ".len()..)?
    } else if lowered.starts_with("enter in ") {
        transcript.get("enter in ".len()..)?
    } else if lowered.starts_with("put in ") {
        transcript.get("put in ".len()..)?
    } else {
        return None;
    };

    normalize_field_target(remainder)
}

fn strip_fill_and_submit_suffix(transcript: &str) -> Option<String> {
    let normalized = collapse_transcript_whitespace(transcript);
    if normalized.is_empty() {
        return None;
    }

    let lowered = normalized.to_ascii_lowercase();
    for suffix in [
        " and then submit this form",
        " and then submit form",
        " and then send this form",
        " and then send form",
        " and then press submit",
        " and then hit submit",
        " and then submit",
        " then submit this form",
        " then submit form",
        " then send this form",
        " then send form",
        " then press submit",
        " then hit submit",
        " then submit",
        " and submit this form",
        " and submit form",
        " and send this form",
        " and send form",
        " and press submit",
        " and hit submit",
        " and submit",
    ] {
        if lowered.ends_with(suffix) {
            let trimmed = normalized
                .get(..normalized.len().saturating_sub(suffix.len()))?
                .trim();
            return (!trimmed.is_empty()).then(|| trimmed.to_string());
        }
    }

    None
}

fn split_case_insensitive_once<'a>(text: &'a str, separator: &str) -> Option<(&'a str, &'a str)> {
    let lowered = text.to_ascii_lowercase();
    let index = lowered.find(separator)?;
    let before = text.get(..index)?.trim();
    let after = text.get(index + separator.len()..)?.trim();
    Some((before, after))
}

fn normalize_field_target(target: &str) -> Option<String> {
    let mut normalized = collapse_transcript_whitespace(target);
    if normalized.is_empty() {
        return None;
    }

    let lowered = normalized.to_ascii_lowercase();
    for prefix in ["the ", "my ", "a ", "an "] {
        if lowered.starts_with(prefix) {
            normalized = normalized.get(prefix.len()..)?.trim().to_string();
            break;
        }
    }

    for suffix in [" field", " textbox", " text box", " input", " input box"] {
        if normalized.to_ascii_lowercase().ends_with(suffix) {
            let end = normalized.len().saturating_sub(suffix.len());
            normalized = normalized.get(..end)?.trim().to_string();
            break;
        }
    }

    let normalized = normalized.trim();
    (!normalized.is_empty()).then(|| normalized.to_string())
}

fn normalize_fill_value(value: &str) -> Option<String> {
    let collapsed = collapse_transcript_whitespace(value);
    let trimmed = collapsed.trim();
    if trimmed.is_empty() {
        return None;
    }

    let unquoted = if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        trimmed
            .get(1..trimmed.len().saturating_sub(1))
            .unwrap_or(trimmed)
    } else {
        trimmed
    };

    let cleaned = unquoted.trim();
    (!cleaned.is_empty()).then(|| cleaned.to_string())
}

fn is_browser_visibility_phrase(normalized: &str) -> bool {
    normalized.contains("show browser")
        || normalized.contains("hide browser")
        || normalized.contains("show the browser")
        || normalized.contains("hide the browser")
        || normalized.contains("make browser visible")
        || normalized.contains("make the browser visible")
        || normalized.contains("make it visible")
        || normalized.contains("switch to visible")
        || normalized.contains("switch browser to visible")
        || normalized.contains("visible mode")
        || normalized.contains("show the window")
        || normalized.contains("go headless")
        || normalized.contains("make browser headless")
        || normalized.contains("make the browser headless")
        || normalized.contains("make it headless")
        || normalized.contains("switch to headless")
        || normalized.contains("switch browser to headless")
        || normalized.contains("headless mode")
}

fn selected_skill(active_skill_names: &[String], skill_name: &'static str) -> Vec<String> {
    if active_skill_names
        .iter()
        .any(|active_name| active_name == skill_name)
    {
        vec![String::from(skill_name)]
    } else {
        Vec::new()
    }
}

fn selected_audio_skill(active_skill_names: &[String], skill_name: &'static str) -> Vec<String> {
    selected_skill(active_skill_names, skill_name)
}

fn selected_stop_skill(active_skill_names: &[String]) -> Vec<String> {
    if active_skill_names
        .iter()
        .any(|active_name| active_name == "stop_reading")
    {
        vec![String::from("stop_reading")]
    } else if active_skill_names
        .iter()
        .any(|active_name| active_name == "pause_reading")
    {
        vec![String::from("pause_reading")]
    } else {
        Vec::new()
    }
}

fn build_single_step_planner_output(
    intent: IntentSummary,
    selected_skills: Vec<String>,
    step: PlannedStep,
) -> PlannerOutput {
    PlannerOutput {
        status: PlannerStatus::Ready,
        intent,
        selected_skills,
        steps: vec![step],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    }
}

fn build_audio_set_planner_output(spec: AudioSetPlanSpec<'_>) -> PlannerOutput {
    PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: spec.intent_name,
            goal: spec.goal,
            target_description: spec.target_description,
        },
        selected_skills: spec.selected_skills,
        steps: vec![
            PlannedStep {
                step_id: String::from(spec.set_step_id),
                tool_name: spec.tool_name,
                arguments: spec.tool_arguments,
                purpose: spec.tool_purpose,
                on_success: StepTransition::NextStep {
                    step_id: String::from(spec.report_step_id),
                },
                on_failure: StepTransition::Replan,
            },
            build_report_result_step(spec.request_id, spec.report_step_id, spec.report_summary),
        ],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    }
}

fn build_audio_report_planner_output(
    request_id: &str,
    intent_name: IntentName,
    goal: String,
    selected_skills: Vec<String>,
    target_description: Option<String>,
    report_summary: String,
) -> PlannerOutput {
    PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: intent_name,
            goal,
            target_description,
        },
        selected_skills,
        steps: vec![build_report_result_step(
            request_id,
            "report-audio-setting",
            report_summary,
        )],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    }
}

fn build_browser_visibility_planner_output(
    request_id: &str,
    target_mode: BrowserVisibilityMode,
    selected_skills: Vec<String>,
    report_summary: String,
) -> PlannerOutput {
    PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::SetBrowserVisibility,
            goal: String::from("Set the browser visibility mode to the requested target."),
            target_description: Some(format_browser_visibility_mode(target_mode)),
        },
        selected_skills,
        steps: vec![
            PlannedStep {
                step_id: String::from("set-browser-visibility"),
                tool_name: ToolName::SetBrowserVisibility,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "mode": target_mode
                }),
                purpose: String::from("Apply the requested browser visibility mode."),
                on_success: StepTransition::NextStep {
                    step_id: String::from("report-browser-visibility"),
                },
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from("report-browser-visibility"),
                tool_name: ToolName::ReportResult,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "status": ReportStatus::Success,
                    "summary": report_summary.clone(),
                    "next_recommended_action": serde_json::Value::Null,
                    "user_message": report_summary
                }),
                purpose: String::from("Report the resulting browser visibility mode."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    }
}

fn build_status_query_planner_output(spec: StatusQueryPlanSpec<'_>) -> PlannerOutput {
    PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: spec.intent_name,
            goal: spec.goal,
            target_description: spec.target_description,
        },
        selected_skills: spec.selected_skills,
        steps: vec![
            PlannedStep {
                step_id: String::from(spec.read_step_id),
                tool_name: spec.read_tool_name,
                arguments: spec.read_tool_arguments,
                purpose: spec.read_tool_purpose,
                on_success: StepTransition::NextStep {
                    step_id: String::from(spec.report_step_id),
                },
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from(spec.report_step_id),
                tool_name: ToolName::ReportResult,
                arguments: serde_json::json!({
                    "request_id": spec.request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "status": ReportStatus::Success,
                    "summary": spec.report_summary.clone(),
                    "next_recommended_action": serde_json::Value::Null,
                    "user_message": spec.report_summary
                }),
                purpose: String::from("Report the resulting status query answer."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    }
}

fn build_report_result_step(request_id: &str, step_id: &str, summary: String) -> PlannedStep {
    PlannedStep {
        step_id: String::from(step_id),
        tool_name: ToolName::ReportResult,
        arguments: serde_json::json!({
            "request_id": request_id,
            "timeout_ms": serde_json::Value::Null,
            "status": ReportStatus::Success,
            "summary": summary.clone(),
            "next_recommended_action": serde_json::Value::Null,
            "user_message": summary
        }),
        purpose: String::from("Report the resulting playback setting."),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    }
}

fn format_playback_volume(volume: f32) -> String {
    format!("{}%", (volume * 100.0).round() as i32)
}

fn format_playback_speed(speed: f32) -> String {
    let formatted = format!("{speed:.2}");
    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
    format!("{trimmed}x")
}

fn format_browser_visibility_mode(mode: BrowserVisibilityMode) -> String {
    match mode {
        BrowserVisibilityMode::Visible => String::from("visible"),
        BrowserVisibilityMode::Headless => String::from("headless"),
    }
}

fn format_current_url_summary(agent_state: &AgentStateData) -> String {
    match (
        normalized_optional_text(agent_state.title.as_deref()),
        agent_state.url.as_deref(),
    ) {
        (Some(title), Some(url)) => format!("Current page is {title} at {url}."),
        (None, Some(url)) => format!("Current page URL is {url}."),
        (Some(title), None) => format!("Current page is {title}."),
        (None, None) => String::from("No page is open yet."),
    }
}

fn current_page_label(agent_state: &AgentStateData) -> String {
    normalized_optional_text(agent_state.title.as_deref())
        .or_else(|| normalized_optional_text(agent_state.url.as_deref()))
        .unwrap_or_else(|| String::from("no page open"))
}

fn format_runtime_status_summary(runtime_status: &GetRuntimeStatusData) -> String {
    let page_summary = current_page_label_from_runtime_status(runtime_status);
    let browser_mode = format_browser_visibility_mode(runtime_status.browser_visibility);
    let listening = if runtime_status.listening_state.is_listening {
        "on"
    } else {
        "off"
    };
    let speaking = if runtime_status.speaking {
        "active"
    } else {
        "idle"
    };
    let back = if runtime_status.browser_history.can_go_back {
        "available"
    } else {
        "unavailable"
    };
    let forward = if runtime_status.browser_history.can_go_forward {
        "available"
    } else {
        "unavailable"
    };

    format!(
        "Current page is {page_summary}. Browser mode is {browser_mode}. Listening is {listening}. Speech output is {speaking}. Back is {back}. Forward is {forward}."
    )
}

fn current_page_label_from_runtime_status(runtime_status: &GetRuntimeStatusData) -> String {
    normalized_optional_text(runtime_status.title.as_deref())
        .or_else(|| normalized_optional_text(runtime_status.url.as_deref()))
        .unwrap_or_else(|| String::from("no page open"))
}

fn format_back_history_summary(runtime_status: &GetRuntimeStatusData) -> String {
    if runtime_status.browser_history.can_go_back {
        String::from("Back navigation is available.")
    } else {
        String::from("Back navigation is not available.")
    }
}

fn format_forward_history_summary(runtime_status: &GetRuntimeStatusData) -> String {
    if runtime_status.browser_history.can_go_forward {
        String::from("Forward navigation is available.")
    } else {
        String::from("Forward navigation is not available.")
    }
}

fn format_listening_summary(runtime_status: &GetRuntimeStatusData) -> String {
    if runtime_status.listening_state.is_listening {
        String::from("Listening is on.")
    } else {
        String::from("Listening is off.")
    }
}

fn format_speaking_summary(runtime_status: &GetRuntimeStatusData) -> String {
    if runtime_status.speaking {
        String::from("Speech output is active.")
    } else {
        String::from("Speech output is idle.")
    }
}

fn format_browser_mode_summary(runtime_status: &GetRuntimeStatusData) -> String {
    format!(
        "Browser mode is {}.",
        format_browser_visibility_mode(runtime_status.browser_visibility)
    )
}

fn parse_browser_visibility_command(
    normalized: &str,
    current_visibility: BrowserVisibilityMode,
) -> Option<BrowserVisibilityMode> {
    if normalized.contains("hide browser")
        || normalized.contains("hide the browser")
        || normalized.contains("go headless")
        || normalized.contains("make browser headless")
        || normalized.contains("make the browser headless")
        || normalized.contains("make it headless")
        || normalized.contains("switch to headless")
        || normalized.contains("switch browser to headless")
        || normalized.contains("headless mode")
    {
        return Some(BrowserVisibilityMode::Headless);
    }

    if normalized.contains("show browser")
        || normalized.contains("show the browser")
        || normalized.contains("make browser visible")
        || normalized.contains("make the browser visible")
        || normalized.contains("make it visible")
        || normalized.contains("switch to visible")
        || normalized.contains("switch browser to visible")
        || normalized.contains("visible mode")
        || normalized.contains("show the window")
    {
        return Some(BrowserVisibilityMode::Visible);
    }

    if normalized.contains("toggle browser visibility") || normalized.contains("toggle visibility")
    {
        return Some(match current_visibility {
            BrowserVisibilityMode::Visible => BrowserVisibilityMode::Headless,
            BrowserVisibilityMode::Headless => BrowserVisibilityMode::Visible,
        });
    }

    None
}

fn normalized_optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(String::from)
}

fn selected_status_skill(active_skill_names: &[String]) -> Vec<String> {
    if active_skill_names
        .iter()
        .any(|active_name| active_name == "get_status")
    {
        vec![String::from("get_status")]
    } else if active_skill_names
        .iter()
        .any(|active_name| active_name == "announce_state")
    {
        vec![String::from("announce_state")]
    } else {
        Vec::new()
    }
}

struct StatusQueryPlanSpec<'a> {
    request_id: &'a str,
    intent_name: IntentName,
    goal: String,
    selected_skills: Vec<String>,
    target_description: Option<String>,
    read_step_id: &'a str,
    read_tool_name: ToolName,
    read_tool_arguments: serde_json::Value,
    read_tool_purpose: String,
    report_step_id: &'a str,
    report_summary: String,
}

fn round_audio_setting_value(value: f32) -> f32 {
    (value * 100.0).round() / 100.0
}

struct AudioSetPlanSpec<'a> {
    request_id: &'a str,
    intent_name: IntentName,
    goal: String,
    selected_skills: Vec<String>,
    target_description: Option<String>,
    set_step_id: &'a str,
    tool_name: ToolName,
    tool_arguments: serde_json::Value,
    tool_purpose: String,
    report_step_id: &'a str,
    report_summary: String,
}

fn score_skill(
    skill: &LoadedSkill,
    transcript_tokens: &HashSet<String>,
    inferred_intent: &IntentName,
    likely_tools: &[ToolName],
) -> Option<i32> {
    let skill_tokens = tokenize_text(&format!(
        "{} {} {} {}",
        skill.summary.name,
        skill.summary.description,
        skill.summary.intent_tags.join(" "),
        skill.body
    ));
    let lexical_overlap = transcript_tokens.intersection(&skill_tokens).count() as i32;
    let intent_match = skill
        .summary
        .intent_tags
        .iter()
        .any(|tag| tag == &format!("intent:{inferred_intent:?}"));
    let tool_overlap = skill
        .summary
        .allowed_tools
        .as_ref()
        .map(|tools| {
            tools
                .iter()
                .filter(|tool| likely_tools.iter().any(|candidate| candidate == *tool))
                .count() as i32
        })
        .unwrap_or(0);

    if lexical_overlap == 0 && !intent_match && tool_overlap == 0 {
        return None;
    }

    let precedence_score = match skill.source {
        SkillSource::Project => 3_000,
        SkillSource::User => 2_000,
        SkillSource::Bundled => 1_000,
    };

    Some(
        precedence_score
            + skill.summary.priority
            + (lexical_overlap * 75)
            + (tool_overlap * 100)
            + if intent_match { 500 } else { 0 },
    )
}

fn validate_tool_arguments<Input>(step: &PlannedStep) -> Result<(), ToolError>
where
    Input: serde::de::DeserializeOwned,
{
    serde_json::from_value::<Input>(step.arguments.clone())
        .map(|_| ())
        .map_err(|error| {
            invalid_planner_output(
                format!("tool arguments did not match the expected schema: {error}"),
                Some(serde_json::json!({
                    "step_id": step.step_id,
                    "tool_name": step.tool_name,
                })),
            )
        })
}

fn execute_planner_output_with_runner<Runner>(
    request_id: String,
    planner_output: &PlannerOutput,
    mut run_step: Runner,
) -> ExecutionOutcome
where
    Runner: FnMut(&PlannedStep) -> SerializedToolResult,
{
    let trace = ExecutionTrace {
        executed_step_ids: Vec::new(),
        tool_results: Vec::new(),
    };

    match planner_output.status {
        PlannerStatus::Complete => {
            return ExecutionOutcome::Complete { trace };
        }
        PlannerStatus::Blocked => {
            return ExecutionOutcome::Aborted {
                trace,
                error: ToolError {
                    code: String::from("planner_blocked"),
                    message: planner_output.user_message.clone().unwrap_or_else(|| {
                        String::from("planner blocked execution before any tools could run")
                    }),
                    retryable: false,
                    details: Some(serde_json::json!({
                        "blocked_reason": planner_output.blocked_reason,
                    })),
                },
            };
        }
        PlannerStatus::Ready | PlannerStatus::NeedsConfirmation => {}
    }

    let Some(current_step_id) = planner_output
        .steps
        .first()
        .map(|step| step.step_id.clone())
    else {
        return ExecutionOutcome::Aborted {
            trace,
            error: ToolError {
                code: String::from("empty_plan"),
                message: String::from("planner returned no executable steps"),
                retryable: false,
                details: Some(serde_json::json!({
                    "planner_status": planner_output.status,
                })),
            },
        };
    };

    execute_steps_with_runner(
        StepExecutionContext {
            request_id,
            intent_name: planner_output.intent.name.clone(),
            selected_skills: planner_output.selected_skills.clone(),
            steps: &planner_output.steps,
            initial_step_id: current_step_id,
            block_side_effects_until_confirmation: planner_output.status
                == PlannerStatus::NeedsConfirmation,
        },
        &mut run_step,
        trace,
    )
}

fn resume_after_confirmation_with_runner<Runner>(
    pending_plan_execution: &PendingPlanExecutionState,
    confirmation_id: &str,
    confirmed: bool,
    mut run_step: Runner,
) -> ExecutionOutcome
where
    Runner: FnMut(&PlannedStep) -> SerializedToolResult,
{
    let trace = ExecutionTrace {
        executed_step_ids: Vec::new(),
        tool_results: Vec::new(),
    };

    if pending_plan_execution.confirmation_id != confirmation_id {
        return ExecutionOutcome::Aborted {
            trace,
            error: ToolError {
                code: String::from("confirmation_id_mismatch"),
                message: String::from(
                    "confirmation response did not match the pending confirmation id",
                ),
                retryable: false,
                details: Some(serde_json::json!({
                    "expected_confirmation_id": pending_plan_execution.confirmation_id,
                    "received_confirmation_id": confirmation_id,
                })),
            },
        };
    }

    if !confirmed {
        return ExecutionOutcome::NeedsReplan { trace };
    }

    let Some(next_step_id) = pending_plan_execution.next_step_id.clone() else {
        return ExecutionOutcome::Complete { trace };
    };

    execute_steps_with_runner(
        StepExecutionContext {
            request_id: pending_plan_execution.request_id.clone(),
            intent_name: pending_plan_execution.intent_name.clone(),
            selected_skills: pending_plan_execution.selected_skills.clone(),
            steps: &pending_plan_execution.queued_steps,
            initial_step_id: next_step_id,
            block_side_effects_until_confirmation: false,
        },
        &mut run_step,
        trace,
    )
}

fn execute_steps_with_runner<Runner>(
    context: StepExecutionContext<'_>,
    run_step: &mut Runner,
    mut trace: ExecutionTrace,
) -> ExecutionOutcome
where
    Runner: FnMut(&PlannedStep) -> SerializedToolResult,
{
    let StepExecutionContext {
        request_id,
        intent_name,
        selected_skills,
        steps,
        initial_step_id,
        block_side_effects_until_confirmation,
    } = context;

    let step_positions = match build_step_positions(steps) {
        Ok(positions) => positions,
        Err(error) => {
            return ExecutionOutcome::Aborted { trace, error };
        }
    };

    if !step_positions.contains_key(&initial_step_id) {
        return ExecutionOutcome::Aborted {
            trace,
            error: ToolError {
                code: String::from("missing_resume_step"),
                message: format!(
                    "pending execution referenced missing step '{}'",
                    initial_step_id
                ),
                retryable: false,
                details: None,
            },
        };
    }

    let mut current_step_id = initial_step_id;
    let mut visited_step_ids = HashSet::new();
    loop {
        if !visited_step_ids.insert(current_step_id.clone()) {
            return ExecutionOutcome::Aborted {
                trace,
                error: ToolError {
                    code: String::from("planner_step_cycle"),
                    message: format!(
                        "planner attempted to execute step '{}' more than once",
                        current_step_id
                    ),
                    retryable: false,
                    details: None,
                },
            };
        }

        let step = &steps[*step_positions
            .get(&current_step_id)
            .expect("step positions should contain the current step")];

        if block_side_effects_until_confirmation && is_side_effecting_tool(&step.tool_name) {
            return ExecutionOutcome::Aborted {
                trace,
                error: ToolError {
                    code: String::from("side_effect_before_confirmation"),
                    message: format!(
                        "planner attempted to execute side-effecting tool {:?} before confirmation",
                        step.tool_name
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({
                        "step_id": step.step_id,
                        "tool_name": step.tool_name,
                    })),
                },
            };
        }

        let result = run_step(step);
        trace.executed_step_ids.push(step.step_id.clone());
        trace.tool_results.push(result.clone());

        let transition = if result.ok {
            &step.on_success
        } else {
            &step.on_failure
        };

        match transition {
            StepTransition::NextStep { step_id } => {
                if !step_positions.contains_key(step_id) {
                    return ExecutionOutcome::Aborted {
                        trace,
                        error: ToolError {
                            code: String::from("missing_transition_step"),
                            message: format!(
                                "planner referenced missing next step '{}' from '{}'",
                                step_id, step.step_id
                            ),
                            retryable: false,
                            details: None,
                        },
                    };
                }
                current_step_id = step_id.clone();
            }
            StepTransition::Complete => {
                return ExecutionOutcome::Complete { trace };
            }
            StepTransition::RequestConfirmation => {
                let confirmation_id = match extract_confirmation_id(&result) {
                    Ok(confirmation_id) => confirmation_id,
                    Err(error) => {
                        return ExecutionOutcome::Aborted { trace, error };
                    }
                };
                let prompt_text = match extract_confirmation_prompt_text(&result) {
                    Ok(prompt_text) => prompt_text,
                    Err(error) => {
                        return ExecutionOutcome::Aborted { trace, error };
                    }
                };

                let queued_step_ids = queued_step_ids_after(steps, step, &step_positions);
                let queued_steps = queued_steps_after(steps, step, &step_positions);
                let pending_plan_execution = PendingPlanExecutionState {
                    request_id,
                    intent_name: intent_name.clone(),
                    selected_skills: selected_skills.clone(),
                    confirmation_id: confirmation_id.clone(),
                    prompt_text,
                    next_step_id: queued_step_ids.first().cloned(),
                    queued_step_ids,
                    queued_steps,
                };

                return ExecutionOutcome::AwaitingConfirmation {
                    trace,
                    pending_confirmation_id: confirmation_id,
                    pending_plan_execution,
                };
            }
            StepTransition::Replan => {
                return ExecutionOutcome::NeedsReplan { trace };
            }
        }
    }
}

fn execute_serialized_tool<E, Input, Output, Handler>(
    step: &PlannedStep,
    tool_name: ToolName,
    executor: &mut E,
    handler: Handler,
) -> SerializedToolResult
where
    E: DeterministicToolExecutor,
    Input: for<'de> Deserialize<'de>,
    Output: Serialize,
    Handler: FnOnce(&mut E, Input) -> ToolResult<Output>,
{
    match serde_json::from_value::<Input>(step.arguments.clone()) {
        Ok(input) => serialize_tool_result(handler(executor, input)),
        Err(error) => ToolResult::failure(
            tool_name,
            inferred_request_id(step),
            ToolError {
                code: String::from("invalid_tool_arguments"),
                message: format!("tool arguments did not match the expected schema: {error}"),
                retryable: false,
                details: Some(serde_json::json!({
                    "step_id": step.step_id,
                    "arguments": step.arguments,
                })),
            },
            vec![String::from(
                "Executor rejected the tool call because the arguments were invalid.",
            )],
        ),
    }
}

fn build_step_positions(steps: &[PlannedStep]) -> Result<HashMap<String, usize>, ToolError> {
    let mut positions = HashMap::with_capacity(steps.len());

    for (index, step) in steps.iter().enumerate() {
        if positions.insert(step.step_id.clone(), index).is_some() {
            return Err(ToolError {
                code: String::from("duplicate_step_id"),
                message: format!("planner returned duplicate step id '{}'", step.step_id),
                retryable: false,
                details: None,
            });
        }
    }

    Ok(positions)
}

fn queued_step_ids_after(
    steps: &[PlannedStep],
    current_step: &PlannedStep,
    step_positions: &HashMap<String, usize>,
) -> Vec<String> {
    let Some(current_index) = step_positions.get(&current_step.step_id).copied() else {
        return Vec::new();
    };

    steps
        .iter()
        .skip(current_index + 1)
        .map(|step| step.step_id.clone())
        .collect()
}

fn queued_steps_after(
    steps: &[PlannedStep],
    current_step: &PlannedStep,
    step_positions: &HashMap<String, usize>,
) -> Vec<PlannedStep> {
    let Some(current_index) = step_positions.get(&current_step.step_id).copied() else {
        return Vec::new();
    };

    steps.iter().skip(current_index + 1).cloned().collect()
}

fn extract_confirmation_id(result: &SerializedToolResult) -> Result<String, ToolError> {
    let confirmation_id = result
        .data
        .as_ref()
        .and_then(|data| data.get("confirmation_id"))
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    confirmation_id.ok_or_else(|| ToolError {
        code: String::from("missing_confirmation_id"),
        message: String::from(
            "step requested confirmation but the tool result did not include confirmation_id",
        ),
        retryable: false,
        details: Some(serde_json::json!({
            "tool_name": result.tool_name,
            "request_id": result.request_id,
        })),
    })
}

fn extract_confirmation_prompt_text(result: &SerializedToolResult) -> Result<String, ToolError> {
    let prompt_text = result
        .data
        .as_ref()
        .and_then(|data| data.get("prompt_text"))
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    prompt_text.ok_or_else(|| ToolError {
        code: String::from("missing_confirmation_prompt"),
        message: String::from(
            "step requested confirmation but the tool result did not include prompt_text",
        ),
        retryable: false,
        details: Some(serde_json::json!({
            "tool_name": result.tool_name,
            "request_id": result.request_id,
        })),
    })
}

fn is_side_effecting_tool(tool_name: &ToolName) -> bool {
    !matches!(
        tool_name,
        ToolName::CaptureScreenshot
            | ToolName::GetPageSnapshot
            | ToolName::ExtractPageModel
            | ToolName::ListInteractiveElements
            | ToolName::FindElement
            | ToolName::TranscribeCommand
            | ToolName::GetAgentState
            | ToolName::GetRuntimeStatus
            | ToolName::ConfirmAction
            | ToolName::ReportResult
    )
}

fn serialize_tool_result<T>(result: ToolResult<T>) -> SerializedToolResult
where
    T: Serialize,
{
    let ToolResult {
        ok,
        tool_name,
        request_id,
        timestamp_ms,
        data,
        error,
        warnings,
        observations,
    } = result;

    let serialized_data = match data {
        Some(data) => match serde_json::to_value(data) {
            Ok(value) => Some(value),
            Err(error) => {
                return ToolResult::failure(
                    tool_name,
                    request_id,
                    ToolError {
                        code: String::from("tool_result_serialization_failed"),
                        message: format!("failed to serialize tool result payload: {error}"),
                        retryable: false,
                        details: None,
                    },
                    vec![String::from(
                        "Executor could not serialize the tool result payload.",
                    )],
                );
            }
        },
        None => None,
    };

    ToolResult {
        ok,
        tool_name,
        request_id,
        timestamp_ms,
        data: serialized_data,
        error,
        warnings,
        observations,
    }
}

fn inferred_request_id(step: &PlannedStep) -> String {
    step.arguments
        .get("request_id")
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| step.step_id.clone())
}

fn current_timestamp_ms() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as u64,
        Err(_) => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockExecutor {
        last_open_url: Option<String>,
        last_go_back_request: Option<GoBackInput>,
        last_go_forward_request: Option<GoForwardInput>,
        last_reload_request: Option<ReloadPageInput>,
        last_scroll_request: Option<ScrollPageInput>,
        last_capture_screenshot_request: Option<CaptureScreenshotInput>,
        last_run_ocr_request: Option<RunOcrInput>,
        last_merge_ocr_request: Option<MergeOcrIntoPageModelInput>,
        last_read_region_request: Option<ReadRegionInput>,
        last_read_next_region_request: Option<ReadNextRegionInput>,
        last_read_previous_region_request: Option<ReadPreviousRegionInput>,
        last_stop_speaking_request: Option<StopSpeakingInput>,
        last_start_listening_request: Option<StartListeningInput>,
        last_stop_listening_request: Option<StopListeningInput>,
        last_transcribe_command_request: Option<TranscribeCommandInput>,
        last_snapshot_request: Option<GetPageSnapshotInput>,
        last_list_request: Option<ListInteractiveElementsInput>,
        last_find_request: Option<FindElementInput>,
        last_click_request: Option<ClickElementInput>,
        last_focus_request: Option<FocusElementInput>,
        last_type_request: Option<TypeIntoElementInput>,
        last_submit_request: Option<SubmitActiveFormInput>,
        last_extract_request: Option<ExtractPageModelInput>,
        last_voice: Option<String>,
        last_volume: Option<f32>,
        last_speed: Option<f32>,
        last_visibility: Option<BrowserVisibilityMode>,
        last_confirmation_prompt: Option<String>,
        last_report_result: Option<ReportResultData>,
    }

    impl DeterministicToolExecutor for MockExecutor {
        fn execute_open_url(&mut self, input: OpenUrlInput) -> ToolResult<OpenUrlData> {
            self.last_open_url = Some(input.url.clone());
            ToolResult::success(
                ToolName::OpenUrl,
                input.request_id,
                OpenUrlData {
                    final_url: input.url,
                    title: None,
                    page_id: String::from("page-1"),
                    load_state: input.wait_for_load_state.unwrap_or(LoadState::Load),
                    http_status: None,
                    history: BrowserHistoryState {
                        can_go_back: false,
                        can_go_forward: false,
                        current_entry_index: Some(0),
                        entry_count: 1,
                    },
                },
                vec![String::from("opened url")],
            )
        }

        fn execute_go_back(&mut self, input: GoBackInput) -> ToolResult<GoBackData> {
            self.last_go_back_request = Some(input.clone());
            ToolResult::success(
                ToolName::GoBack,
                input.request_id,
                GoBackData {
                    navigated: true,
                    actual_steps: input.steps.unwrap_or(1),
                    final_url: Some(String::from("https://example.com/previous")),
                    title: Some(String::from("Previous page")),
                    load_state: Some(input.wait_for_load_state.unwrap_or(LoadState::Load)),
                    history: BrowserHistoryState {
                        can_go_back: false,
                        can_go_forward: true,
                        current_entry_index: Some(0),
                        entry_count: 2,
                    },
                },
                vec![String::from("went back in history")],
            )
        }

        fn execute_go_forward(&mut self, input: GoForwardInput) -> ToolResult<GoForwardData> {
            self.last_go_forward_request = Some(input.clone());
            ToolResult::success(
                ToolName::GoForward,
                input.request_id,
                GoForwardData {
                    navigated: true,
                    actual_steps: input.steps.unwrap_or(1),
                    final_url: Some(String::from("https://example.com/next")),
                    title: Some(String::from("Next page")),
                    load_state: Some(input.wait_for_load_state.unwrap_or(LoadState::Load)),
                    history: BrowserHistoryState {
                        can_go_back: true,
                        can_go_forward: false,
                        current_entry_index: Some(1),
                        entry_count: 2,
                    },
                },
                vec![String::from("went forward in history")],
            )
        }

        fn execute_reload_page(&mut self, input: ReloadPageInput) -> ToolResult<ReloadPageData> {
            self.last_reload_request = Some(input.clone());
            ToolResult::success(
                ToolName::ReloadPage,
                input.request_id,
                ReloadPageData {
                    reloaded: true,
                    final_url: String::from("https://example.com/current"),
                    title: Some(String::from("Current page")),
                    load_state: input.wait_for_load_state.unwrap_or(LoadState::Load),
                    http_status: None,
                    history: BrowserHistoryState {
                        can_go_back: true,
                        can_go_forward: false,
                        current_entry_index: Some(1),
                        entry_count: 2,
                    },
                },
                vec![String::from("reloaded the page")],
            )
        }

        fn execute_scroll_page(&mut self, input: ScrollPageInput) -> ToolResult<ScrollPageData> {
            self.last_scroll_request = Some(input.clone());
            ToolResult::success(
                ToolName::ScrollPage,
                input.request_id,
                ScrollPageData {
                    previous_scroll_y: 120.0,
                    current_scroll_y: 640.0,
                    reached_boundary: false,
                },
                vec![String::from("scrolled the page")],
            )
        }

        fn execute_capture_screenshot(
            &mut self,
            input: CaptureScreenshotInput,
        ) -> ToolResult<CaptureScreenshotData> {
            self.last_capture_screenshot_request = Some(input.clone());
            ToolResult::success(
                ToolName::CaptureScreenshot,
                input.request_id,
                CaptureScreenshotData {
                    image_id: String::from("image-1"),
                    path: String::from("/tmp/image-1.png"),
                    bbox: input.bbox,
                    width: 640,
                    height: 480,
                },
                vec![String::from("captured a screenshot")],
            )
        }

        fn execute_run_ocr(&mut self, input: RunOcrInput) -> ToolResult<RunOcrData> {
            self.last_run_ocr_request = Some(input.clone());
            ToolResult::success(
                ToolName::RunOcr,
                input.request_id,
                RunOcrData {
                    image_id: input.image_id,
                    extracted_text: String::from("recognized text"),
                    text_length: 15,
                    confidence: Some(0.82),
                    source_bbox: input.bbox,
                },
                vec![String::from("ran OCR on the requested image")],
            )
        }

        fn execute_merge_ocr_into_page_model(
            &mut self,
            input: MergeOcrIntoPageModelInput,
        ) -> ToolResult<MergeOcrIntoPageModelData> {
            self.last_merge_ocr_request = Some(input.clone());
            ToolResult::success(
                ToolName::MergeOcrIntoPageModel,
                input.request_id,
                MergeOcrIntoPageModelData {
                    page_id: input.page_id,
                    updated_region_ids: vec![input
                        .region_id
                        .unwrap_or_else(|| String::from("ocr-region-1"))],
                    merged_text_length: input.ocr_text.trim().len(),
                },
                vec![String::from("merged OCR text into the page model")],
            )
        }

        fn execute_read_region(&mut self, input: ReadRegionInput) -> ToolResult<ReadRegionData> {
            self.last_read_region_request = Some(input.clone());
            ToolResult::success(
                ToolName::ReadRegion,
                input.request_id,
                ReadRegionData {
                    region_id: input.region_id,
                    region_index: 1,
                    text_length: 128,
                    speech_started: true,
                },
                vec![String::from("started reading the requested region")],
            )
        }

        fn execute_read_next_region(
            &mut self,
            input: ReadNextRegionInput,
        ) -> ToolResult<ReadNextRegionData> {
            self.last_read_next_region_request = Some(input.clone());
            ToolResult::success(
                ToolName::ReadNextRegion,
                input.request_id,
                ReadNextRegionData {
                    cursor: NarrationCursor {
                        current_region_id: Some(String::from("region-2")),
                        current_index: Some(1),
                        total_regions: 3,
                    },
                    region_id: Some(String::from("region-2")),
                    speech_started: true,
                    reached_end: false,
                },
                vec![String::from("advanced narration to the next region")],
            )
        }

        fn execute_read_previous_region(
            &mut self,
            input: ReadPreviousRegionInput,
        ) -> ToolResult<ReadPreviousRegionData> {
            self.last_read_previous_region_request = Some(input.clone());
            ToolResult::success(
                ToolName::ReadPreviousRegion,
                input.request_id,
                ReadPreviousRegionData {
                    cursor: NarrationCursor {
                        current_region_id: Some(String::from("region-1")),
                        current_index: Some(0),
                        total_regions: 3,
                    },
                    region_id: Some(String::from("region-1")),
                    speech_started: true,
                    reached_start: false,
                },
                vec![String::from("moved narration to the previous region")],
            )
        }

        fn execute_stop_speaking(
            &mut self,
            input: StopSpeakingInput,
        ) -> ToolResult<StopSpeakingData> {
            self.last_stop_speaking_request = Some(input.clone());
            ToolResult::success(
                ToolName::StopSpeaking,
                input.request_id,
                StopSpeakingData {
                    stopped: true,
                    interrupted_region_id: Some(String::from("region-2")),
                },
                vec![String::from("stopped current narration playback")],
            )
        }

        fn execute_start_listening(
            &mut self,
            input: StartListeningInput,
        ) -> ToolResult<StartListeningData> {
            self.last_start_listening_request = Some(input.clone());
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

        fn execute_stop_listening(
            &mut self,
            input: StopListeningInput,
        ) -> ToolResult<StopListeningData> {
            self.last_stop_listening_request = Some(input.clone());
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

        fn execute_transcribe_command(
            &mut self,
            input: TranscribeCommandInput,
        ) -> ToolResult<TranscribeCommandData> {
            self.last_transcribe_command_request = Some(input.clone());
            ToolResult::success(
                ToolName::TranscribeCommand,
                input.request_id,
                TranscribeCommandData {
                    transcript: Some(String::from("read the next section")),
                    confidence: None,
                    audio_duration_ms: input.max_duration_ms.or(Some(3_000)),
                    listening_state: ListeningState {
                        is_listening: !input.auto_stop,
                        push_to_talk_enabled: true,
                    },
                },
                vec![String::from("transcribed a spoken command")],
            )
        }

        fn execute_get_page_snapshot(
            &mut self,
            input: GetPageSnapshotInput,
        ) -> ToolResult<PageSnapshotData> {
            self.last_snapshot_request = Some(input.clone());
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
                    scroll_y: 0.0,
                    viewport_width: 0.0,
                    viewport_height: 0.0,
                    document_height: 0.0,
                },
                vec![String::from("captured page snapshot")],
            )
        }

        fn execute_list_interactive_elements(
            &mut self,
            input: ListInteractiveElementsInput,
        ) -> ToolResult<ListInteractiveElementsData> {
            self.last_list_request = Some(input.clone());
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

        fn execute_find_element(&mut self, input: FindElementInput) -> ToolResult<FindElementData> {
            self.last_find_request = Some(input.clone());
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

        fn execute_click_element(
            &mut self,
            input: ClickElementInput,
        ) -> ToolResult<ClickElementData> {
            self.last_click_request = Some(input.clone());
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

        fn execute_focus_element(
            &mut self,
            input: FocusElementInput,
        ) -> ToolResult<FocusElementData> {
            self.last_focus_request = Some(input.clone());
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

        fn execute_type_into_element(
            &mut self,
            input: TypeIntoElementInput,
        ) -> ToolResult<TypeIntoElementData> {
            self.last_type_request = Some(input.clone());
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

        fn execute_submit_active_form(
            &mut self,
            input: SubmitActiveFormInput,
        ) -> ToolResult<SubmitActiveFormData> {
            self.last_submit_request = Some(input.clone());
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

        fn execute_extract_page_model(
            &mut self,
            input: ExtractPageModelInput,
        ) -> ToolResult<ExtractPageModelData> {
            self.last_extract_request = Some(input.clone());
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

        fn execute_set_tts_voice(
            &mut self,
            input: SetTtsVoiceInput,
        ) -> ToolResult<SetTtsVoiceData> {
            self.last_voice = Some(input.voice.to_string());
            ToolResult::success(
                ToolName::SetTtsVoice,
                input.request_id,
                SetTtsVoiceData {
                    voice: input.voice.to_string(),
                    changed: true,
                },
                vec![String::from("voice updated")],
            )
        }

        fn execute_set_playback_volume(
            &mut self,
            input: SetPlaybackVolumeInput,
        ) -> ToolResult<SetPlaybackVolumeData> {
            self.last_volume = Some(input.volume);
            ToolResult::success(
                ToolName::SetPlaybackVolume,
                input.request_id,
                SetPlaybackVolumeData {
                    playback_volume: input.volume,
                    muted: input.volume == 0.0,
                    changed: true,
                },
                vec![String::from("volume updated")],
            )
        }

        fn execute_set_playback_speed(
            &mut self,
            input: SetPlaybackSpeedInput,
        ) -> ToolResult<SetPlaybackSpeedData> {
            self.last_speed = Some(input.speed);
            ToolResult::success(
                ToolName::SetPlaybackSpeed,
                input.request_id,
                SetPlaybackSpeedData {
                    playback_speed: input.speed,
                    changed: true,
                },
                vec![String::from("speed updated")],
            )
        }

        fn execute_set_browser_visibility(
            &mut self,
            input: SetBrowserVisibilityInput,
        ) -> ToolResult<SetBrowserVisibilityData> {
            self.last_visibility = Some(input.mode);
            ToolResult::success(
                ToolName::SetBrowserVisibility,
                input.request_id,
                SetBrowserVisibilityData {
                    mode: input.mode,
                    changed: true,
                    supported: true,
                },
                vec![String::from("visibility updated")],
            )
        }

        fn execute_get_agent_state(
            &mut self,
            input: GetAgentStateInput,
        ) -> ToolResult<AgentStateData> {
            ToolResult::success(
                ToolName::GetAgentState,
                input.request_id,
                AgentStateData {
                    page_id: None,
                    url: Some(String::from("https://example.com")),
                    title: Some(String::from("Example")),
                    browser_visibility: BrowserVisibilityMode::Visible,
                    browser_history: BrowserHistoryState::default(),
                    narration_cursor: Some(NarrationCursor::default()),
                    speaking: false,
                    listening_state: ListeningState::default(),
                    audio: RuntimeAudioState::default(),
                    last_transcript: if input.include_last_transcript {
                        Some(String::from("read next"))
                    } else {
                        None
                    },
                    last_tool_call: Some(LastToolCallSummary {
                        request_id: String::from("req-5"),
                        tool_name: ToolName::GetAgentState,
                        ok: true,
                        observation_summary: vec![String::from("agent state read")],
                    }),
                    pending_confirmation_id: None,
                    pending_plan_execution: None,
                    tts_model_settings: TtsModelSettings {
                        mode: ProviderMode::Local,
                        active_profile: Some(String::from("kitten-default")),
                        available_profiles: vec![TtsModelOption {
                            profile_name: String::from("kitten-default"),
                            model_label: String::from("default"),
                        }],
                    },
                    local_tts_model_settings: LocalTtsModelSettings {
                        profile_name: Some(String::from("kitten-default")),
                        backend: Some(String::from("kitten_tts_rs")),
                        model_id: Some(String::from("default")),
                        model_path: Some(String::from("/path/to/kitten/model")),
                        default_voice: Some(String::from("Bruno")),
                        sample_rate: Some(24_000),
                    },
                tts_voice_settings: TtsVoiceSettings {
                        mode: ProviderMode::Local,
                        active_voice: Some(String::from("Bruno")),
                        available_voices: vec![
                            TtsVoiceOption {
                                voice_name: String::from("Bella"),
                                display_label: String::from("Bella"),
                            },
                            TtsVoiceOption {
                                voice_name: String::from("Bruno"),
                                display_label: String::from("Bruno"),
                            },
                        ],
                    },
                    tts_provider_settings: TtsProviderSettings {
                        active_mode: ProviderMode::Local,
                        available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
                    },
                    asr_provider_settings: AsrProviderSettings {
                        active_mode: ProviderMode::Local,
                        available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
                    },
                    local_asr_model_settings: LocalAsrModelSettings {
                        profile_name: Some(String::from("whisper-default")),
                        backend: Some(String::from("whisper")),
                        model_id: Some(String::from("tiny")),
                        model_path: Some(String::from("/path/to/whisper/model")),
                        language: Some(String::from("en")),
                        threads: Some(4),
                    },
                    planner_provider_settings: PlannerProviderSettings {
                        active_mode: ProviderMode::Remote,
                        available_modes: vec![ProviderMode::Remote],
                        summary: String::from("Planner currently uses configured remote profiles only."),
                    },
                    remote_planner_settings: RemotePlannerSettings {
                        profile_name: Some(String::from("openai-default")),
                        provider: Some(String::from("OpenAI")),
                        base_url: Some(String::from("https://api.openai.com/v1")),
                        model: Some(String::from("gpt-5.4-mini")),
                        api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                        organization_reference: None,
                        project: None,
                        temperature_milli: Some(200),
                        max_output_tokens: Some(1024),
                        timeout_ms: Some(30_000),
                    },
                    remote_tts_settings: RemoteTtsSettings {
                        profile_name: Some(String::from("openai-tts-default")),
                        provider: Some(String::from("OpenAI")),
                        base_url: Some(String::from("https://api.openai.com/v1")),
                        model: Some(String::from("gpt-4o-mini-tts")),
                        api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                        organization_reference: None,
                        project: None,
                        voice: Some(String::from("alloy")),
                        audio_format: Some(String::from("wav")),
                        timeout_ms: Some(30_000),
                    },
                    remote_asr_settings: RemoteAsrSettings {
                        profile_name: Some(String::from("openai-transcribe-default")),
                        provider: Some(String::from("OpenAI")),
                        base_url: Some(String::from("https://api.openai.com/v1")),
                        model: Some(String::from("gpt-4o-mini-transcribe")),
                        api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                        organization_reference: None,
                        project: None,
                        language: Some(String::from("en")),
                        temperature_milli: Some(0),
                        timeout_ms: Some(30_000),
                    },
                    provider_failover_settings: ProviderFailoverSettings {
                        planner_available: false,
                        tts_available: false,
                        asr_available: false,
                        summary: String::from("Automatic provider failover is not currently available in the live runtime."),
                    },
                    confirmation_settings: ConfirmationSettings {
                        confirmation_confidence_threshold: 0.9,
                        allow_click_without_confirmation: true,
                        always_confirm_submit: true,
                    },
                    ocr_threshold_settings: OcrThresholdSettings {
                        sparse_text_char_threshold: 200,
                        sparse_text_region_threshold: 2,
                    },
                },
                vec![String::from("agent state read")],
            )
        }

        fn execute_get_runtime_status(
            &mut self,
            input: GetRuntimeStatusInput,
        ) -> ToolResult<GetRuntimeStatusData> {
            ToolResult::success(
                ToolName::GetRuntimeStatus,
                input.request_id,
                GetRuntimeStatusData {
                    page_id: None,
                    url: Some(String::from("https://example.com")),
                    title: Some(String::from("Example")),
                    browser_visibility: BrowserVisibilityMode::Visible,
                    browser_history: BrowserHistoryState::default(),
                    listening_state: ListeningState::default(),
                    speaking: false,
                    audio: RuntimeAudioState::default(),
                    pending_confirmation_id: None,
                    pending_plan_execution: None,
                    provider_modes: if input.include_provider_modes {
                        Some(ProviderSelectionStatus {
                            planner_mode: ProviderMode::Remote,
                            tts_mode: ProviderMode::Local,
                            asr_mode: ProviderMode::Local,
                        })
                    } else {
                        None
                    },
                },
                vec![String::from("runtime status read")],
            )
        }

        fn execute_confirm_action(
            &mut self,
            input: ConfirmActionInput,
        ) -> ToolResult<ConfirmActionData> {
            self.last_confirmation_prompt = Some(input.prompt_text.clone());
            ToolResult::success(
                ToolName::ConfirmAction,
                input.request_id,
                ConfirmActionData {
                    confirmation_id: String::from("confirm-1"),
                    prompt_text: input.prompt_text,
                    confirmed: None,
                    timed_out: false,
                },
                vec![input.reason],
            )
        }

        fn execute_report_result(
            &mut self,
            input: ReportResultInput,
        ) -> ToolResult<ReportResultData> {
            let data = ReportResultData {
                status: input.status,
                summary: input.summary,
                next_recommended_action: input.next_recommended_action,
                user_message: input.user_message,
            };
            self.last_report_result = Some(data.clone());

            ToolResult::success(
                ToolName::ReportResult,
                input.request_id,
                data,
                vec![String::from("reported final result")],
            )
        }
    }

    fn sample_planned_step(tool_name: ToolName) -> PlannedStep {
        match tool_name {
            ToolName::OpenUrl => PlannedStep {
                step_id: String::from("step-open-url"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-open-url",
                    "timeout_ms": 1000,
                    "url": "https://example.com/article",
                    "wait_for_load_state": "NetworkIdle"
                }),
                purpose: String::from("navigate to a page"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::GoBack => PlannedStep {
                step_id: String::from("step-go-back"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-go-back",
                    "timeout_ms": 1000,
                    "steps": 2,
                    "wait_for_load_state": "Load"
                }),
                purpose: String::from("go back in history"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::GoForward => PlannedStep {
                step_id: String::from("step-go-forward"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-go-forward",
                    "timeout_ms": 1000,
                    "steps": 1,
                    "wait_for_load_state": "NetworkIdle"
                }),
                purpose: String::from("go forward in history"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::ReloadPage => PlannedStep {
                step_id: String::from("step-reload"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-reload",
                    "timeout_ms": 1000,
                    "hard_reload": true,
                    "wait_for_load_state": "Load"
                }),
                purpose: String::from("reload the current page"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::ScrollPage => PlannedStep {
                step_id: String::from("step-scroll"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-scroll",
                    "timeout_ms": 1000,
                    "direction": "Down",
                    "amount_px": 480.0,
                    "target": null
                }),
                purpose: String::from("scroll the page"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::CaptureScreenshot => PlannedStep {
                step_id: String::from("step-capture-screenshot"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-capture-screenshot",
                    "timeout_ms": 1000,
                    "full_page": false,
                    "region_id": serde_json::Value::Null,
                    "bbox": {
                        "x": 10.0,
                        "y": 20.0,
                        "width": 300.0,
                        "height": 120.0
                    }
                }),
                purpose: String::from("capture a deterministic screenshot"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::SetBrowserVisibility => PlannedStep {
                step_id: String::from("step-set-browser-visibility"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-visibility",
                    "timeout_ms": 1000,
                    "mode": "Headless"
                }),
                purpose: String::from("toggle browser visibility"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::GetPageSnapshot => PlannedStep {
                step_id: String::from("step-snapshot"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-snapshot",
                    "timeout_ms": 1000,
                    "include_interactive_elements": true,
                    "text_excerpt_max_chars": 120
                }),
                purpose: String::from("read current page snapshot"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::ExtractPageModel => PlannedStep {
                step_id: String::from("step-extract"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-extract",
                    "timeout_ms": 1000,
                    "use_dom_extraction": true,
                    "include_headings": true,
                    "include_links": false
                }),
                purpose: String::from("extract a page model"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::ListInteractiveElements => PlannedStep {
                step_id: String::from("step-list"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-list",
                    "timeout_ms": 1000,
                    "visible_only": true,
                    "roles": ["Button"]
                }),
                purpose: String::from("list visible buttons"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::FindElement => PlannedStep {
                step_id: String::from("step-find"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-find",
                    "timeout_ms": 1000,
                    "description": "continue",
                    "text": null,
                    "role": "Button",
                    "color_hint": null,
                    "nearby_text": null,
                    "selector_hint": null,
                    "visible_only": true,
                    "max_candidates": 3
                }),
                purpose: String::from("find the continue button"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::ClickElement => PlannedStep {
                step_id: String::from("step-click"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-click",
                    "timeout_ms": 1000,
                    "element_id": "button-1",
                    "double_click": false
                }),
                purpose: String::from("click the resolved button"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::FocusElement => PlannedStep {
                step_id: String::from("step-focus"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-focus",
                    "timeout_ms": 1000,
                    "element_id": "input-1"
                }),
                purpose: String::from("focus the resolved field"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::TypeIntoElement => PlannedStep {
                step_id: String::from("step-type"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-type",
                    "timeout_ms": 1000,
                    "element_id": "input-1",
                    "text": "phil@example.com",
                    "clear_first": true,
                    "submit_after": false
                }),
                purpose: String::from("type into the resolved field"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::SubmitActiveForm => PlannedStep {
                step_id: String::from("step-submit"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-submit",
                    "timeout_ms": 1000,
                    "form_element_id": "form-login"
                }),
                purpose: String::from("submit the active form"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::ReadRegion => PlannedStep {
                step_id: String::from("step-read-region"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-read-region",
                    "timeout_ms": 1000,
                    "region_id": "region-2",
                    "interrupt_current": true
                }),
                purpose: String::from("read a specific region"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::ReadNextRegion => PlannedStep {
                step_id: String::from("step-read-next"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-read-next",
                    "timeout_ms": 1000,
                    "interrupt_current": false
                }),
                purpose: String::from("read the next region"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::ReadPreviousRegion => PlannedStep {
                step_id: String::from("step-read-previous"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-read-previous",
                    "timeout_ms": 1000,
                    "interrupt_current": true
                }),
                purpose: String::from("read the previous region"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::StopSpeaking => PlannedStep {
                step_id: String::from("step-stop-speaking"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-stop-speaking",
                    "timeout_ms": 1000
                }),
                purpose: String::from("stop current narration"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::StartListening => PlannedStep {
                step_id: String::from("step-start-listening"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-start-listening",
                    "timeout_ms": 1500
                }),
                purpose: String::from("start listening"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::StopListening => PlannedStep {
                step_id: String::from("step-stop-listening"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-stop-listening"
                }),
                purpose: String::from("stop listening"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::TranscribeCommand => PlannedStep {
                step_id: String::from("step-transcribe-command"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-transcribe-command",
                    "timeout_ms": 2000,
                    "max_duration_ms": 3000,
                    "auto_stop": true
                }),
                purpose: String::from("transcribe a command"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::SetTtsVoice => PlannedStep {
                step_id: String::from("step-set-tts-voice"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-set-tts-voice",
                    "timeout_ms": 1000,
                    "voice": "Bruno"
                }),
                purpose: String::from("change the TTS voice"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::SetPlaybackVolume => PlannedStep {
                step_id: String::from("step-set-playback-volume"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-set-playback-volume",
                    "timeout_ms": 1000,
                    "volume": 0.4
                }),
                purpose: String::from("update volume"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::SetPlaybackSpeed => PlannedStep {
                step_id: String::from("step-set-playback-speed"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-set-playback-speed",
                    "timeout_ms": 1000,
                    "speed": 1.2
                }),
                purpose: String::from("update speed"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::RunOcr => PlannedStep {
                step_id: String::from("step-run-ocr"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-run-ocr",
                    "timeout_ms": 1000,
                    "image_id": "image-1",
                    "region_id": serde_json::Value::Null,
                    "bbox": {
                        "x": 4.0,
                        "y": 8.0,
                        "width": 120.0,
                        "height": 48.0
                    }
                }),
                purpose: String::from("run OCR on a cached screenshot"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::MergeOcrIntoPageModel => PlannedStep {
                step_id: String::from("step-merge-ocr"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-merge-ocr",
                    "timeout_ms": 1000,
                    "page_id": "page-1",
                    "region_id": "region-2",
                    "ocr_text": "Recovered readable text",
                    "source_bbox": {
                        "x": 10.0,
                        "y": 12.0,
                        "width": 200.0,
                        "height": 80.0
                    }
                }),
                purpose: String::from("merge OCR text into the runtime page model"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::GetAgentState => PlannedStep {
                step_id: String::from("step-get-agent-state"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-agent-state",
                    "timeout_ms": 1000,
                    "include_last_transcript": false
                }),
                purpose: String::from("read agent state"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::GetRuntimeStatus => PlannedStep {
                step_id: String::from("step-get-runtime-status"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-runtime-status",
                    "timeout_ms": 1000,
                    "include_provider_modes": true
                }),
                purpose: String::from("read runtime status"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
            ToolName::ConfirmAction => PlannedStep {
                step_id: String::from("step-confirm-action"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-confirm-action",
                    "timeout_ms": 1000,
                    "prompt_text": "Do you want me to continue?",
                    "reason": "The next step may submit data."
                }),
                purpose: String::from("request confirmation"),
                on_success: StepTransition::RequestConfirmation,
                on_failure: StepTransition::Replan,
            },
            ToolName::ReportResult => PlannedStep {
                step_id: String::from("step-report-result"),
                tool_name,
                arguments: serde_json::json!({
                    "request_id": "req-report-result",
                    "timeout_ms": 1000,
                    "status": "Success",
                    "summary": "Opened the requested page.",
                    "next_recommended_action": null,
                    "user_message": "The page is ready."
                }),
                purpose: String::from("report completion"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        }
    }

    fn sample_planned_steps_for_registered_tools() -> Vec<PlannedStep> {
        registered_tools()
            .into_iter()
            .map(|tool| sample_planned_step(tool.name))
            .collect()
    }

    #[test]
    fn dispatches_set_playback_volume_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-1"),
            tool_name: ToolName::SetPlaybackVolume,
            arguments: serde_json::json!({
                "request_id": "req-1",
                "timeout_ms": 1000,
                "volume": 0.4
            }),
            purpose: String::from("update volume"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(executor.last_volume, Some(0.4));
        assert_eq!(result.tool_name, ToolName::SetPlaybackVolume);
        assert_eq!(result.request_id, "req-1");
        let data = result
            .data
            .expect("serialized tool result data should exist");
        assert_eq!(data.get("muted"), Some(&serde_json::Value::Bool(false)));
        assert_eq!(data.get("changed"), Some(&serde_json::Value::Bool(true)));
        let playback_volume = data
            .get("playback_volume")
            .and_then(serde_json::Value::as_f64)
            .expect("playback_volume should be serialized as a number");
        assert!((playback_volume - 0.4).abs() < 0.000_001);
    }

    #[test]
    fn dispatches_open_url_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-open-url"),
            tool_name: ToolName::OpenUrl,
            arguments: serde_json::json!({
                "request_id": "req-open-url",
                "timeout_ms": 1000,
                "url": "https://example.com/article",
                "wait_for_load_state": "NetworkIdle"
            }),
            purpose: String::from("navigate to a page"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor.last_open_url.as_deref(),
            Some("https://example.com/article")
        );
        let data = result.data.expect("open_url should serialize");
        assert_eq!(
            data.get("final_url"),
            Some(&serde_json::Value::String(String::from(
                "https://example.com/article"
            )))
        );
        assert_eq!(
            data.get("load_state"),
            Some(&serde_json::Value::String(String::from("NetworkIdle")))
        );
    }

    #[test]
    fn dispatches_go_back_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-go-back"),
            tool_name: ToolName::GoBack,
            arguments: serde_json::json!({
                "request_id": "req-go-back",
                "timeout_ms": 1000,
                "steps": 2,
                "wait_for_load_state": "Load"
            }),
            purpose: String::from("go back in history"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor
                .last_go_back_request
                .as_ref()
                .and_then(|input| input.steps),
            Some(2)
        );
    }

    #[test]
    fn dispatches_go_forward_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-go-forward"),
            tool_name: ToolName::GoForward,
            arguments: serde_json::json!({
                "request_id": "req-go-forward",
                "timeout_ms": 1000,
                "steps": 1,
                "wait_for_load_state": "NetworkIdle"
            }),
            purpose: String::from("go forward in history"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor
                .last_go_forward_request
                .as_ref()
                .and_then(|input| input.steps),
            Some(1)
        );
    }

    #[test]
    fn dispatches_reload_page_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-reload"),
            tool_name: ToolName::ReloadPage,
            arguments: serde_json::json!({
                "request_id": "req-reload",
                "timeout_ms": 1000,
                "hard_reload": true,
                "wait_for_load_state": "Load"
            }),
            purpose: String::from("reload the current page"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor
                .last_reload_request
                .as_ref()
                .map(|input| input.hard_reload),
            Some(true)
        );
    }

    #[test]
    fn dispatches_scroll_page_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-scroll"),
            tool_name: ToolName::ScrollPage,
            arguments: serde_json::json!({
                "request_id": "req-scroll",
                "timeout_ms": 1000,
                "direction": "Down",
                "amount_px": 480.0,
                "target": null
            }),
            purpose: String::from("scroll the page"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor
                .last_scroll_request
                .as_ref()
                .and_then(|input| input.amount_px),
            Some(480.0)
        );
    }

    #[test]
    fn dispatches_read_region_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-read-region"),
            tool_name: ToolName::ReadRegion,
            arguments: serde_json::json!({
                "request_id": "req-read-region",
                "timeout_ms": 1000,
                "region_id": "region-2",
                "interrupt_current": true
            }),
            purpose: String::from("read a specific region"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor
                .last_read_region_request
                .as_ref()
                .map(|input| input.region_id.as_str()),
            Some("region-2")
        );
    }

    #[test]
    fn dispatches_read_next_region_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-read-next"),
            tool_name: ToolName::ReadNextRegion,
            arguments: serde_json::json!({
                "request_id": "req-read-next",
                "timeout_ms": 1000,
                "interrupt_current": false
            }),
            purpose: String::from("read the next region"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor
                .last_read_next_region_request
                .as_ref()
                .map(|input| input.interrupt_current),
            Some(false)
        );
    }

    #[test]
    fn dispatches_read_previous_region_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-read-previous"),
            tool_name: ToolName::ReadPreviousRegion,
            arguments: serde_json::json!({
                "request_id": "req-read-previous",
                "timeout_ms": 1000,
                "interrupt_current": true
            }),
            purpose: String::from("read the previous region"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor
                .last_read_previous_region_request
                .as_ref()
                .map(|input| input.interrupt_current),
            Some(true)
        );
    }

    #[test]
    fn dispatches_stop_speaking_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-stop-speaking"),
            tool_name: ToolName::StopSpeaking,
            arguments: serde_json::json!({
                "request_id": "req-stop-speaking",
                "timeout_ms": 1000
            }),
            purpose: String::from("stop current narration"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor
                .last_stop_speaking_request
                .as_ref()
                .map(|input| input.request_id.as_str()),
            Some("req-stop-speaking")
        );
    }

    #[test]
    fn dispatches_start_listening_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-start-listening"),
            tool_name: ToolName::StartListening,
            arguments: serde_json::json!({
                "request_id": "req-start-listening",
                "timeout_ms": 1500
            }),
            purpose: String::from("start listening"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor
                .last_start_listening_request
                .as_ref()
                .map(|input| input.request_id.as_str()),
            Some("req-start-listening")
        );
    }

    #[test]
    fn dispatches_stop_listening_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-stop-listening"),
            tool_name: ToolName::StopListening,
            arguments: serde_json::json!({
                "request_id": "req-stop-listening"
            }),
            purpose: String::from("stop listening"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor
                .last_stop_listening_request
                .as_ref()
                .map(|input| input.request_id.as_str()),
            Some("req-stop-listening")
        );
    }

    #[test]
    fn dispatches_transcribe_command_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-transcribe-command"),
            tool_name: ToolName::TranscribeCommand,
            arguments: serde_json::json!({
                "request_id": "req-transcribe-command",
                "timeout_ms": 2000,
                "max_duration_ms": 3000,
                "auto_stop": true
            }),
            purpose: String::from("transcribe a command"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor
                .last_transcribe_command_request
                .as_ref()
                .map(|input| input.request_id.as_str()),
            Some("req-transcribe-command")
        );
        assert_eq!(
            executor
                .last_transcribe_command_request
                .as_ref()
                .and_then(|input| input.max_duration_ms),
            Some(3000)
        );
    }

    #[test]
    fn dispatches_capture_screenshot_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-capture-screenshot"),
            tool_name: ToolName::CaptureScreenshot,
            arguments: serde_json::json!({
                "request_id": "req-capture-screenshot",
                "timeout_ms": 1000,
                "full_page": false,
                "region_id": serde_json::Value::Null,
                "bbox": {
                    "x": 10.0,
                    "y": 20.0,
                    "width": 300.0,
                    "height": 120.0
                }
            }),
            purpose: String::from("capture a deterministic screenshot"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor
                .last_capture_screenshot_request
                .as_ref()
                .map(|input| input.request_id.as_str()),
            Some("req-capture-screenshot")
        );
        assert_eq!(
            executor
                .last_capture_screenshot_request
                .as_ref()
                .and_then(|input| input.bbox.as_ref())
                .map(|bbox| (bbox.x, bbox.y, bbox.width, bbox.height)),
            Some((10.0, 20.0, 300.0, 120.0))
        );
    }

    #[test]
    fn dispatches_run_ocr_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-run-ocr"),
            tool_name: ToolName::RunOcr,
            arguments: serde_json::json!({
                "request_id": "req-run-ocr",
                "timeout_ms": 1000,
                "image_id": "image-1",
                "region_id": serde_json::Value::Null,
                "bbox": {
                    "x": 4.0,
                    "y": 8.0,
                    "width": 120.0,
                    "height": 48.0
                }
            }),
            purpose: String::from("run OCR on a cached screenshot"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor
                .last_run_ocr_request
                .as_ref()
                .and_then(|input| input.image_id.as_deref()),
            Some("image-1")
        );
        assert_eq!(
            executor
                .last_run_ocr_request
                .as_ref()
                .and_then(|input| input.bbox.as_ref())
                .map(|bbox| (bbox.x, bbox.y, bbox.width, bbox.height)),
            Some((4.0, 8.0, 120.0, 48.0))
        );
    }

    #[test]
    fn dispatches_merge_ocr_into_page_model_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-merge-ocr"),
            tool_name: ToolName::MergeOcrIntoPageModel,
            arguments: serde_json::json!({
                "request_id": "req-merge-ocr",
                "timeout_ms": 1000,
                "page_id": "page-1",
                "region_id": "region-2",
                "ocr_text": "Recovered readable text",
                "source_bbox": {
                    "x": 10.0,
                    "y": 12.0,
                    "width": 200.0,
                    "height": 80.0
                }
            }),
            purpose: String::from("merge OCR text into the runtime page model"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor
                .last_merge_ocr_request
                .as_ref()
                .map(|input| input.page_id.as_str()),
            Some("page-1")
        );
        assert_eq!(
            executor
                .last_merge_ocr_request
                .as_ref()
                .and_then(|input| input.region_id.as_deref()),
            Some("region-2")
        );
    }

    #[test]
    fn dispatches_get_page_snapshot_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-snapshot"),
            tool_name: ToolName::GetPageSnapshot,
            arguments: serde_json::json!({
                "request_id": "req-snapshot",
                "timeout_ms": 1000,
                "include_interactive_elements": true,
                "text_excerpt_max_chars": 120
            }),
            purpose: String::from("read current page snapshot"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor
                .last_snapshot_request
                .as_ref()
                .map(|input| input.include_interactive_elements),
            Some(true)
        );
        let data = result.data.expect("get_page_snapshot should serialize");
        assert_eq!(
            data.get("page_id"),
            Some(&serde_json::Value::String(String::from("page-1")))
        );
        assert!(data
            .get("interactive_elements")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|elements| !elements.is_empty()));
    }

    #[test]
    fn dispatches_list_interactive_elements_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-list"),
            tool_name: ToolName::ListInteractiveElements,
            arguments: serde_json::json!({
                "request_id": "req-list",
                "timeout_ms": 1000,
                "visible_only": true,
                "roles": ["Button"]
            }),
            purpose: String::from("list visible buttons"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor
                .last_list_request
                .as_ref()
                .map(|input| input.visible_only),
            Some(true)
        );
        let data = result
            .data
            .expect("list_interactive_elements should serialize");
        assert_eq!(
            data.get("page_id"),
            Some(&serde_json::Value::String(String::from("page-1")))
        );
        assert_eq!(
            data.get("visible_count"),
            Some(&serde_json::Value::Number(serde_json::Number::from(1)))
        );
        assert!(data
            .get("elements")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|elements| elements.len() == 1));
    }

    #[test]
    fn dispatches_find_element_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-find"),
            tool_name: ToolName::FindElement,
            arguments: serde_json::json!({
                "request_id": "req-find",
                "timeout_ms": 1000,
                "description": "continue",
                "text": null,
                "role": "Button",
                "color_hint": null,
                "nearby_text": null,
                "selector_hint": null,
                "visible_only": true,
                "max_candidates": 3
            }),
            purpose: String::from("find the continue button"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor
                .last_find_request
                .as_ref()
                .and_then(|input| input.role.as_ref()),
            Some(&crate::page_model::ElementRole::Button)
        );
        let data = result.data.expect("find_element should serialize");
        assert_eq!(
            data.get("chosen_element_id"),
            Some(&serde_json::Value::String(String::from("button-1")))
        );
        assert_eq!(
            data.get("requires_confirmation"),
            Some(&serde_json::Value::Bool(false))
        );
    }

    #[test]
    fn dispatches_click_element_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-click"),
            tool_name: ToolName::ClickElement,
            arguments: serde_json::json!({
                "request_id": "req-click",
                "timeout_ms": 1000,
                "element_id": "button-1",
                "double_click": false
            }),
            purpose: String::from("click the resolved button"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor
                .last_click_request
                .as_ref()
                .map(|input| input.element_id.as_str()),
            Some("button-1")
        );
        let data = result.data.expect("click_element should serialize");
        assert_eq!(
            data.get("element_id"),
            Some(&serde_json::Value::String(String::from("button-1")))
        );
        assert_eq!(
            data.get("action_performed"),
            Some(&serde_json::Value::Bool(true))
        );
        assert_eq!(
            data.get("page_changed"),
            Some(&serde_json::Value::Bool(false))
        );
    }

    #[test]
    fn dispatches_focus_element_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-focus"),
            tool_name: ToolName::FocusElement,
            arguments: serde_json::json!({
                "request_id": "req-focus",
                "timeout_ms": 1000,
                "element_id": "input-1"
            }),
            purpose: String::from("focus the resolved field"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor
                .last_focus_request
                .as_ref()
                .map(|input| input.element_id.as_str()),
            Some("input-1")
        );
        let data = result.data.expect("focus_element should serialize");
        assert_eq!(data.get("focused"), Some(&serde_json::Value::Bool(true)));
    }

    #[test]
    fn dispatches_type_into_element_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-type"),
            tool_name: ToolName::TypeIntoElement,
            arguments: serde_json::json!({
                "request_id": "req-type",
                "timeout_ms": 1000,
                "element_id": "input-1",
                "text": "phil@example.com",
                "clear_first": true,
                "submit_after": false
            }),
            purpose: String::from("type into the resolved field"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor
                .last_type_request
                .as_ref()
                .map(|input| input.text.as_str()),
            Some("phil@example.com")
        );
        let data = result.data.expect("type_into_element should serialize");
        assert_eq!(
            data.get("accepted_input"),
            Some(&serde_json::Value::Bool(true))
        );
    }

    #[test]
    fn dispatches_submit_active_form_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-submit"),
            tool_name: ToolName::SubmitActiveForm,
            arguments: serde_json::json!({
                "request_id": "req-submit",
                "timeout_ms": 1000,
                "form_element_id": "form-login"
            }),
            purpose: String::from("submit the active form"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor
                .last_submit_request
                .as_ref()
                .and_then(|input| input.form_element_id.as_deref()),
            Some("form-login")
        );
        let data = result.data.expect("submit_active_form should serialize");
        assert_eq!(data.get("submitted"), Some(&serde_json::Value::Bool(true)));
        assert_eq!(
            data.get("page_changed"),
            Some(&serde_json::Value::Bool(true))
        );
    }

    #[test]
    fn dispatches_extract_page_model_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-extract"),
            tool_name: ToolName::ExtractPageModel,
            arguments: serde_json::json!({
                "request_id": "req-extract",
                "timeout_ms": 1000,
                "use_dom_extraction": true,
                "include_headings": true,
                "include_links": false
            }),
            purpose: String::from("extract a page model"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor
                .last_extract_request
                .as_ref()
                .map(|input| input.include_links),
            Some(false)
        );
        let data = result.data.expect("extract_page_model should serialize");
        assert_eq!(
            data.get("extraction_source"),
            Some(&serde_json::Value::String(String::from("DomFallback")))
        );
        assert!(data
            .get("page_model")
            .and_then(|model| model.get("interactive_elements"))
            .and_then(serde_json::Value::as_array)
            .is_some_and(|elements| elements.is_empty()));
    }

    #[test]
    fn rejects_invalid_tool_arguments_before_dispatch() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-2"),
            tool_name: ToolName::SetPlaybackSpeed,
            arguments: serde_json::json!({
                "request_id": "req-2",
                "timeout_ms": 1000,
                "speed": "fast"
            }),
            purpose: String::from("update speed"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(!result.ok);
        assert_eq!(result.tool_name, ToolName::SetPlaybackSpeed);
        assert_eq!(result.request_id, "req-2");
        assert_eq!(
            result.error.expect("error should be present").code,
            "invalid_tool_arguments"
        );
        assert_eq!(executor.last_speed, None);
    }

    #[test]
    fn dispatches_set_browser_visibility_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-3"),
            tool_name: ToolName::SetBrowserVisibility,
            arguments: serde_json::json!({
                "request_id": "req-3",
                "timeout_ms": 1000,
                "mode": "Headless"
            }),
            purpose: String::from("toggle browser visibility"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor.last_visibility,
            Some(BrowserVisibilityMode::Headless)
        );
        assert_eq!(result.tool_name, ToolName::SetBrowserVisibility);
    }

    #[test]
    fn dispatches_get_runtime_status_with_provider_modes() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-4"),
            tool_name: ToolName::GetRuntimeStatus,
            arguments: serde_json::json!({
                "request_id": "req-4",
                "timeout_ms": 1000,
                "include_provider_modes": true
            }),
            purpose: String::from("read runtime status"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        let data = result.data.expect("runtime status should serialize");
        assert!(data.get("provider_modes").is_some());
    }

    #[test]
    fn dispatches_get_agent_state_without_last_transcript() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-5"),
            tool_name: ToolName::GetAgentState,
            arguments: serde_json::json!({
                "request_id": "req-5",
                "timeout_ms": 1000,
                "include_last_transcript": false
            }),
            purpose: String::from("read agent state"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        let data = result.data.expect("agent state should serialize");
        assert_eq!(data.get("last_transcript"), Some(&serde_json::Value::Null));
        assert_eq!(
            data.get("last_tool_call")
                .and_then(|entry| entry.get("tool_name")),
            Some(&serde_json::Value::String(String::from("GetAgentState")))
        );
    }

    #[test]
    fn dispatches_confirm_action_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-confirm"),
            tool_name: ToolName::ConfirmAction,
            arguments: serde_json::json!({
                "request_id": "req-confirm-dispatch",
                "timeout_ms": 1000,
                "prompt_text": "Do you want me to continue?",
                "reason": "The next step may submit data."
            }),
            purpose: String::from("request confirmation"),
            on_success: StepTransition::RequestConfirmation,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor.last_confirmation_prompt.as_deref(),
            Some("Do you want me to continue?")
        );
        let data = result.data.expect("confirm_action should serialize");
        assert_eq!(
            data.get("confirmation_id"),
            Some(&serde_json::Value::String(String::from("confirm-1")))
        );
    }

    #[test]
    fn dispatches_report_result_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-report"),
            tool_name: ToolName::ReportResult,
            arguments: serde_json::json!({
                "request_id": "req-report",
                "timeout_ms": 1000,
                "status": "Success",
                "summary": "Opened the requested page.",
                "next_recommended_action": null,
                "user_message": "The page is ready."
            }),
            purpose: String::from("report completion"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(result.tool_name, ToolName::ReportResult);
        assert_eq!(
            executor.last_report_result,
            Some(ReportResultData {
                status: ReportStatus::Success,
                summary: String::from("Opened the requested page."),
                next_recommended_action: None,
                user_message: Some(String::from("The page is ready.")),
            })
        );
        let data = result.data.expect("report_result should serialize");
        assert_eq!(
            data.get("status"),
            Some(&serde_json::Value::String(String::from("Success")))
        );
    }

    #[test]
    fn executes_next_step_chain_until_complete() {
        let mut executor = MockExecutor::default();
        let planner_output = PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::SetPlaybackVolume,
                goal: String::from("adjust audio"),
                target_description: None,
            },
            selected_skills: vec![String::from("audio_controls")],
            steps: vec![
                PlannedStep {
                    step_id: String::from("step-1"),
                    tool_name: ToolName::SetPlaybackVolume,
                    arguments: serde_json::json!({
                        "request_id": "req-plan",
                        "timeout_ms": 1000,
                        "volume": 0.4
                    }),
                    purpose: String::from("set the volume"),
                    on_success: StepTransition::NextStep {
                        step_id: String::from("step-2"),
                    },
                    on_failure: StepTransition::Replan,
                },
                PlannedStep {
                    step_id: String::from("step-2"),
                    tool_name: ToolName::GetRuntimeStatus,
                    arguments: serde_json::json!({
                        "request_id": "req-plan",
                        "timeout_ms": 1000,
                        "include_provider_modes": false
                    }),
                    purpose: String::from("read back the runtime state"),
                    on_success: StepTransition::Complete,
                    on_failure: StepTransition::Replan,
                },
            ],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        };

        let outcome =
            execute_planner_output(&mut executor, String::from("req-plan"), &planner_output);

        match outcome {
            ExecutionOutcome::Complete { trace } => {
                assert_eq!(trace.executed_step_ids, vec!["step-1", "step-2"]);
                assert_eq!(trace.tool_results.len(), 2);
            }
            other => panic!("expected complete outcome, got {other:?}"),
        }
    }

    #[test]
    fn follows_failure_transition_to_replan() {
        let mut executor = MockExecutor::default();
        let planner_output = PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::SetPlaybackSpeed,
                goal: String::from("adjust playback speed"),
                target_description: None,
            },
            selected_skills: vec![String::from("audio_controls")],
            steps: vec![PlannedStep {
                step_id: String::from("step-1"),
                tool_name: ToolName::SetPlaybackSpeed,
                arguments: serde_json::json!({
                    "request_id": "req-replan",
                    "timeout_ms": 1000,
                    "speed": "fast"
                }),
                purpose: String::from("set invalid speed"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        };

        let outcome =
            execute_planner_output(&mut executor, String::from("req-replan"), &planner_output);

        match outcome {
            ExecutionOutcome::NeedsReplan { trace } => {
                assert_eq!(trace.executed_step_ids, vec!["step-1"]);
                assert_eq!(trace.tool_results.len(), 1);
                assert!(!trace.tool_results[0].ok);
            }
            other => panic!("expected replan outcome, got {other:?}"),
        }
    }

    #[test]
    fn returns_awaiting_confirmation_when_transition_requests_it() {
        let planner_output = PlannerOutput {
            status: PlannerStatus::NeedsConfirmation,
            intent: IntentSummary {
                name: IntentName::ClickElement,
                goal: String::from("confirm button choice"),
                target_description: Some(String::from("submit button")),
            },
            selected_skills: vec![String::from("confirm_action")],
            steps: vec![
                PlannedStep {
                    step_id: String::from("step-1"),
                    tool_name: ToolName::ConfirmAction,
                    arguments: serde_json::json!({
                        "request_id": "req-confirm"
                    }),
                    purpose: String::from("ask for confirmation"),
                    on_success: StepTransition::RequestConfirmation,
                    on_failure: StepTransition::Replan,
                },
                PlannedStep {
                    step_id: String::from("step-2"),
                    tool_name: ToolName::SetBrowserVisibility,
                    arguments: serde_json::json!({
                        "request_id": "req-confirm",
                        "timeout_ms": 1000,
                        "mode": "Visible"
                    }),
                    purpose: String::from("placeholder protected step"),
                    on_success: StepTransition::Complete,
                    on_failure: StepTransition::Replan,
                },
            ],
            requires_confirmation: true,
            confirmation_reason: Some(String::from("protected action")),
            blocked_reason: None,
            user_message: Some(String::from("Please confirm.")),
        };

        let outcome = execute_planner_output_with_runner(
            String::from("req-confirm"),
            &planner_output,
            |step| {
                assert_eq!(step.step_id, "step-1");
                ToolResult::success(
                    ToolName::ConfirmAction,
                    String::from("req-confirm"),
                    serde_json::json!({
                        "confirmation_id": "confirm-1",
                        "prompt_text": "Proceed?",
                        "confirmed": serde_json::Value::Null,
                        "timed_out": false
                    }),
                    vec![String::from("confirmation requested")],
                )
            },
        );

        match outcome {
            ExecutionOutcome::AwaitingConfirmation {
                trace,
                pending_confirmation_id,
                pending_plan_execution,
            } => {
                assert_eq!(trace.executed_step_ids, vec!["step-1"]);
                assert_eq!(pending_confirmation_id, "confirm-1");
                assert_eq!(pending_plan_execution.request_id, "req-confirm");
                assert_eq!(pending_plan_execution.intent_name, IntentName::ClickElement);
                assert_eq!(pending_plan_execution.prompt_text, "Proceed?");
                assert_eq!(
                    pending_plan_execution.next_step_id,
                    Some(String::from("step-2"))
                );
                assert_eq!(pending_plan_execution.queued_step_ids, vec!["step-2"]);
                assert_eq!(pending_plan_execution.queued_steps.len(), 1);
                assert_eq!(pending_plan_execution.queued_steps[0].step_id, "step-2");
            }
            other => panic!("expected awaiting confirmation outcome, got {other:?}"),
        }
    }

    #[test]
    fn resumes_confirmed_pending_execution_from_stored_steps() {
        let mut executor = MockExecutor::default();
        let pending_plan_execution = PendingPlanExecutionState {
            request_id: String::from("req-resume"),
            intent_name: IntentName::SetBrowserVisibility,
            selected_skills: vec![String::from("confirm_action")],
            confirmation_id: String::from("confirm-1"),
            prompt_text: String::from("Proceed?"),
            next_step_id: Some(String::from("step-2")),
            queued_step_ids: vec![String::from("step-2")],
            queued_steps: vec![PlannedStep {
                step_id: String::from("step-2"),
                tool_name: ToolName::SetBrowserVisibility,
                arguments: serde_json::json!({
                    "request_id": "req-resume",
                    "timeout_ms": 1000,
                    "mode": "Headless"
                }),
                purpose: String::from("apply confirmed action"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
        };

        let outcome =
            resume_after_confirmation(&mut executor, &pending_plan_execution, "confirm-1", true);

        match outcome {
            ExecutionOutcome::Complete { trace } => {
                assert_eq!(trace.executed_step_ids, vec!["step-2"]);
                assert_eq!(
                    executor.last_visibility,
                    Some(BrowserVisibilityMode::Headless)
                );
            }
            other => panic!("expected complete outcome after resume, got {other:?}"),
        }
    }

    #[test]
    fn resumes_rejected_confirmation_to_replan() {
        let mut executor = MockExecutor::default();
        let pending_plan_execution = PendingPlanExecutionState {
            request_id: String::from("req-resume"),
            intent_name: IntentName::SetBrowserVisibility,
            selected_skills: vec![String::from("confirm_action")],
            confirmation_id: String::from("confirm-1"),
            prompt_text: String::from("Proceed?"),
            next_step_id: Some(String::from("step-2")),
            queued_step_ids: vec![String::from("step-2")],
            queued_steps: vec![PlannedStep {
                step_id: String::from("step-2"),
                tool_name: ToolName::SetBrowserVisibility,
                arguments: serde_json::json!({
                    "request_id": "req-resume",
                    "timeout_ms": 1000,
                    "mode": "Headless"
                }),
                purpose: String::from("apply confirmed action"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
        };

        let outcome =
            resume_after_confirmation(&mut executor, &pending_plan_execution, "confirm-1", false);

        match outcome {
            ExecutionOutcome::NeedsReplan { trace } => {
                assert!(trace.executed_step_ids.is_empty());
                assert_eq!(executor.last_visibility, None);
            }
            other => panic!("expected replan outcome after rejection, got {other:?}"),
        }
    }

    #[test]
    fn rejects_resume_with_mismatched_confirmation_id() {
        let mut executor = MockExecutor::default();
        let pending_plan_execution = PendingPlanExecutionState {
            request_id: String::from("req-resume"),
            intent_name: IntentName::SetBrowserVisibility,
            selected_skills: vec![String::from("confirm_action")],
            confirmation_id: String::from("confirm-1"),
            prompt_text: String::from("Proceed?"),
            next_step_id: Some(String::from("step-2")),
            queued_step_ids: vec![String::from("step-2")],
            queued_steps: vec![PlannedStep {
                step_id: String::from("step-2"),
                tool_name: ToolName::SetBrowserVisibility,
                arguments: serde_json::json!({
                    "request_id": "req-resume",
                    "timeout_ms": 1000,
                    "mode": "Headless"
                }),
                purpose: String::from("apply confirmed action"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
        };

        let outcome = resume_after_confirmation(
            &mut executor,
            &pending_plan_execution,
            "wrong-confirmation-id",
            true,
        );

        match outcome {
            ExecutionOutcome::Aborted { trace, error } => {
                assert!(trace.executed_step_ids.is_empty());
                assert_eq!(error.code, "confirmation_id_mismatch");
                assert_eq!(executor.last_visibility, None);
            }
            other => panic!("expected aborted outcome after mismatch, got {other:?}"),
        }
    }

    #[test]
    fn aborts_when_next_step_transition_is_missing() {
        let mut executor = MockExecutor::default();
        let planner_output = PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::SetPlaybackVolume,
                goal: String::from("adjust audio"),
                target_description: None,
            },
            selected_skills: vec![String::from("audio_controls")],
            steps: vec![PlannedStep {
                step_id: String::from("step-1"),
                tool_name: ToolName::SetPlaybackVolume,
                arguments: serde_json::json!({
                    "request_id": "req-bad-transition",
                    "timeout_ms": 1000,
                    "volume": 0.4
                }),
                purpose: String::from("set the volume"),
                on_success: StepTransition::NextStep {
                    step_id: String::from("missing-step"),
                },
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        };

        let outcome = execute_planner_output(
            &mut executor,
            String::from("req-bad-transition"),
            &planner_output,
        );

        match outcome {
            ExecutionOutcome::Aborted { trace, error } => {
                assert_eq!(trace.executed_step_ids, vec!["step-1"]);
                assert_eq!(error.code, "missing_transition_step");
            }
            other => panic!("expected aborted outcome, got {other:?}"),
        }
    }

    #[test]
    fn aborts_needs_confirmation_plan_before_side_effecting_step() {
        let mut executor = MockExecutor::default();
        let planner_output = PlannerOutput {
            status: PlannerStatus::NeedsConfirmation,
            intent: IntentSummary {
                name: IntentName::SetBrowserVisibility,
                goal: String::from("toggle browser visibility"),
                target_description: None,
            },
            selected_skills: vec![String::from("confirm_action")],
            steps: vec![PlannedStep {
                step_id: String::from("step-1"),
                tool_name: ToolName::SetBrowserVisibility,
                arguments: serde_json::json!({
                    "request_id": "req-needs-confirm",
                    "timeout_ms": 1000,
                    "mode": "Visible"
                }),
                purpose: String::from("protected action before confirmation"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: true,
            confirmation_reason: Some(String::from("protected action")),
            blocked_reason: None,
            user_message: Some(String::from("Please confirm.")),
        };

        let outcome = execute_planner_output(
            &mut executor,
            String::from("req-needs-confirm"),
            &planner_output,
        );

        match outcome {
            ExecutionOutcome::Aborted { trace, error } => {
                assert!(trace.executed_step_ids.is_empty());
                assert_eq!(error.code, "side_effect_before_confirmation");
            }
            other => panic!("expected aborted outcome, got {other:?}"),
        }
    }

    #[test]
    fn planner_available_tools_include_all_wave_two_tools() {
        let available_tools = planner_available_tools();

        assert!(available_tools
            .iter()
            .any(|tool| tool.name == ToolName::OpenUrl));
        assert!(available_tools
            .iter()
            .any(|tool| tool.name == ToolName::TranscribeCommand));
        assert!(available_tools
            .iter()
            .any(|tool| tool.name == ToolName::FocusElement));
        assert!(available_tools
            .iter()
            .any(|tool| tool.name == ToolName::TypeIntoElement));
        assert!(available_tools
            .iter()
            .any(|tool| tool.name == ToolName::SubmitActiveForm));
        assert!(available_tools
            .iter()
            .any(|tool| tool.name == ToolName::CaptureScreenshot));
        assert!(available_tools
            .iter()
            .any(|tool| tool.name == ToolName::RunOcr));
        assert!(available_tools
            .iter()
            .any(|tool| tool.name == ToolName::MergeOcrIntoPageModel));
    }

    #[test]
    fn build_planner_skill_selection_prefers_matching_bundled_skill() {
        let available_tools = planner_available_tools();
        let selection = build_planner_skill_selection(
            None,
            None,
            "please go back to the previous page",
            &available_tools,
        );

        assert!(selection
            .active_skill_names
            .iter()
            .any(|name| name == "go_back"));
        assert_eq!(
            selection
                .relevant_skill_summaries
                .first()
                .map(|skill| skill.name.as_str()),
            Some("go_back")
        );
    }

    #[test]
    fn build_planner_skill_selection_prefers_set_tts_voice_skill_for_voice_commands() {
        let available_tools = planner_available_tools();
        let selection = build_planner_skill_selection(
            None,
            None,
            "change the voice to Bruno",
            &available_tools,
        );

        assert!(selection
            .active_skill_names
            .iter()
            .any(|name| name == "set_tts_voice"));
        assert_eq!(
            selection
                .relevant_skill_summaries
                .first()
                .map(|skill| skill.name.as_str()),
            Some("set_tts_voice")
        );
    }

    #[test]
    fn bundled_skills_cover_planner_visible_command_family_intents() {
        let available_tool_names = registered_tools()
            .into_iter()
            .map(|tool| tool.name)
            .collect::<Vec<_>>();
        let bundled_skills = parse_bundled_skills(BUNDLED_SKILLS_MARKDOWN, &available_tool_names);
        let bundled_intents = bundled_skills
            .iter()
            .flat_map(|skill| skill.summary.intent_tags.iter())
            .filter_map(|tag| tag.strip_prefix("intent:"))
            .map(parse_intent_name_value)
            .collect::<Result<HashSet<_>, _>>()
            .expect("bundled intent tags should parse");

        let required_intents = [
            IntentName::OpenUrl,
            IntentName::GoBack,
            IntentName::GoForward,
            IntentName::ReloadPage,
            IntentName::GetCurrentUrl,
            IntentName::ReadPage,
            IntentName::ReadTitle,
            IntentName::ReadNext,
            IntentName::ReadPrevious,
            IntentName::Repeat,
            IntentName::Stop,
            IntentName::StartListening,
            IntentName::StopListening,
            IntentName::TranscribeCommand,
            IntentName::SetTtsVoice,
            IntentName::SetPlaybackVolume,
            IntentName::GetPlaybackVolume,
            IntentName::SetPlaybackSpeed,
            IntentName::GetPlaybackSpeed,
            IntentName::SetBrowserVisibility,
            IntentName::GetStatus,
            IntentName::FindElement,
            IntentName::ClickElement,
            IntentName::Scroll,
            IntentName::OcrRecovery,
        ];

        let missing = required_intents
            .into_iter()
            .filter(|intent| !bundled_intents.contains(intent))
            .collect::<Vec<_>>();
        assert!(
            missing.is_empty(),
            "bundled skills are missing explicit intent coverage for {missing:?}"
        );
    }

    #[test]
    fn canonical_planner_output_examples_validate_against_current_contract() {
        let available_tools = planner_available_tools();
        let active_skill_names =
            build_planner_skill_selection(None, None, "", &available_tools).active_skill_names;

        for (example_name, planner_output) in canonical_planner_output_examples() {
            validate_planner_output(&planner_output, &available_tools, &active_skill_names)
                .unwrap_or_else(|error| {
                    panic!("canonical planner example '{example_name}' should validate: {error:?}")
                });
        }
    }

    #[test]
    fn registered_tools_all_expose_input_schemas() {
        let missing = registered_tools()
            .into_iter()
            .filter(|tool| tool_input_schema(&tool.name).is_none())
            .map(|tool| tool.name)
            .collect::<Vec<_>>();

        assert!(
            missing.is_empty(),
            "registered tools missing input schemas: {missing:?}"
        );
    }

    #[test]
    fn registered_tools_all_expose_output_schemas() {
        let missing = registered_tools()
            .into_iter()
            .filter(|tool| tool_output_schema(&tool.name).is_none())
            .map(|tool| tool.name)
            .collect::<Vec<_>>();

        assert!(
            missing.is_empty(),
            "registered tools missing output schemas: {missing:?}"
        );
    }

    #[test]
    fn registered_tools_include_output_schema_refs() {
        for tool in registered_tools() {
            assert_eq!(
                tool.output_schema_ref,
                format!("schema://tool-output/{:?}", tool.name)
            );
        }
    }

    #[test]
    fn sample_serialized_tool_results_match_generated_tool_output_schemas() {
        let mut executor = MockExecutor::default();

        for step in sample_planned_steps_for_registered_tools() {
            let result = execute_planned_step(&mut executor, &step);
            let serialized =
                serde_json::to_value(&result).expect("serialized tool result should serialize");
            let schema = tool_output_schema(&step.tool_name).unwrap_or_else(|| {
                panic!(
                    "sample tool result uses tool {:?} without an output schema",
                    step.tool_name
                )
            });
            assert_json_matches_schema(&serialized, &schema).unwrap_or_else(|error| {
                panic!(
                    "sample serialized {:?} result should match generated output schema: {error}",
                    step.tool_name
                )
            });
        }
    }

    #[test]
    fn shared_contract_enums_serialize_expected_variants() {
        assert_eq!(
            serde_json::json!(crate::page_model::ElementRole::Landmark),
            "Landmark"
        );
        assert_eq!(
            serde_json::json!(crate::page_model::ElementRole::Other),
            "Other"
        );
        assert_eq!(
            serde_json::json!(crate::page_model::RegionSource::Mixed),
            "Mixed"
        );
        assert_eq!(
            serde_json::json!(crate::page_model::ExtractionSource::Merged),
            "Merged"
        );
        assert_eq!(
            serde_json::json!(ReportStatus::NeedsFollowUp),
            "NeedsFollowUp"
        );
    }

    #[test]
    fn canonical_planner_output_examples_serialize_expected_strings_and_fields() {
        let examples = canonical_planner_output_examples();

        let set_volume = serde_json::to_value(
            examples
                .get("set_playback_volume")
                .expect("set_playback_volume example should exist"),
        )
        .expect("planner example should serialize");
        assert_eq!(
            set_volume.pointer("/intent/name"),
            Some(&serde_json::json!("SetPlaybackVolume"))
        );
        assert_eq!(
            set_volume.pointer("/steps/0/arguments/request_id"),
            Some(&serde_json::json!("example-set-volume"))
        );
        assert_eq!(
            set_volume.pointer("/steps/0/arguments/volume"),
            Some(&serde_json::json!(0.7))
        );
        assert_eq!(
            set_volume.pointer("/steps/0/on_success"),
            Some(&serde_json::json!({
                "NextStep": {
                    "step_id": "report-playback-volume"
                }
            }))
        );
        assert_eq!(
            set_volume.pointer("/steps/1/on_success"),
            Some(&serde_json::json!("Complete"))
        );

        let ready_click = serde_json::to_value(
            examples
                .get("click_element_ready")
                .expect("click_element_ready example should exist"),
        )
        .expect("planner example should serialize");
        assert_eq!(
            ready_click.pointer("/status"),
            Some(&serde_json::json!("Ready"))
        );
        assert_eq!(
            ready_click.pointer("/steps/0/tool_name"),
            Some(&serde_json::json!("ClickElement"))
        );
        assert_eq!(
            ready_click.pointer("/steps/0/arguments/element_id"),
            Some(&serde_json::json!("link-help"))
        );

        let needs_confirmation = serde_json::to_value(
            examples
                .get("click_element_with_confirmation")
                .expect("click_element_with_confirmation example should exist"),
        )
        .expect("planner example should serialize");
        assert_eq!(
            needs_confirmation.pointer("/status"),
            Some(&serde_json::json!("NeedsConfirmation"))
        );
        assert_eq!(
            needs_confirmation.pointer("/steps/0/on_success"),
            Some(&serde_json::json!("RequestConfirmation"))
        );
        assert_eq!(
            needs_confirmation.pointer("/steps/1/arguments/element_id"),
            Some(&serde_json::json!("button-submit"))
        );
    }

    #[test]
    fn canonical_planner_output_examples_match_generated_planner_output_schema() {
        let schema = planner_output_schema();

        for (example_name, planner_output) in canonical_planner_output_examples() {
            let serialized = serde_json::to_value(&planner_output)
                .expect("planner example should serialize to JSON");
            assert_json_matches_schema(&serialized, &schema).unwrap_or_else(|error| {
                panic!(
                    "canonical planner example '{example_name}' should match generated planner schema: {error}"
                )
            });
        }
    }

    #[test]
    fn canonical_planner_output_step_arguments_match_generated_tool_input_schemas() {
        for (example_name, planner_output) in canonical_planner_output_examples() {
            for step in &planner_output.steps {
                let schema = tool_input_schema(&step.tool_name).unwrap_or_else(|| {
                    panic!(
                        "canonical planner example '{example_name}' uses tool {:?} without an input schema",
                        step.tool_name
                    )
                });
                assert_json_matches_schema(&step.arguments, &schema).unwrap_or_else(|error| {
                    panic!(
                        "canonical planner example '{example_name}' step '{}' arguments should match generated {:?} schema: {error}",
                        step.step_id,
                        step.tool_name
                    )
                });
            }
        }
    }

    #[test]
    fn planner_input_serializes_safety_settings_for_click_policy() {
        let planner_input = PlannerInput {
            request_id: String::from("req-planner"),
            transcript: String::from("click the help link"),
            agent_state: AgentStateData {
                page_id: Some(String::from("page-1")),
                url: Some(String::from("https://example.com")),
                title: Some(String::from("Example")),
                browser_visibility: BrowserVisibilityMode::Visible,
                browser_history: BrowserHistoryState::default(),
                narration_cursor: None,
                speaking: false,
                listening_state: ListeningState::default(),
                audio: RuntimeAudioState::default(),
                last_transcript: None,
                last_tool_call: None,
                pending_confirmation_id: None,
                pending_plan_execution: None,
                tts_model_settings: TtsModelSettings {
                    mode: ProviderMode::Local,
                    active_profile: Some(String::from("kitten-default")),
                    available_profiles: vec![TtsModelOption {
                        profile_name: String::from("kitten-default"),
                        model_label: String::from("default"),
                    }],
                },
                local_tts_model_settings: LocalTtsModelSettings {
                    profile_name: Some(String::from("kitten-default")),
                    backend: Some(String::from("kitten_tts_rs")),
                    model_id: Some(String::from("default")),
                    model_path: Some(String::from("/path/to/kitten/model")),
                    default_voice: Some(String::from("Bruno")),
                    sample_rate: Some(24_000),
                },
                tts_voice_settings: TtsVoiceSettings {
                    mode: ProviderMode::Local,
                    active_voice: Some(String::from("Bruno")),
                    available_voices: vec![
                        TtsVoiceOption {
                            voice_name: String::from("Bella"),
                            display_label: String::from("Bella"),
                        },
                        TtsVoiceOption {
                            voice_name: String::from("Bruno"),
                            display_label: String::from("Bruno"),
                        },
                    ],
                },
                tts_provider_settings: TtsProviderSettings {
                    active_mode: ProviderMode::Local,
                    available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
                },
                asr_provider_settings: AsrProviderSettings {
                    active_mode: ProviderMode::Local,
                    available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
                },
                local_asr_model_settings: LocalAsrModelSettings {
                    profile_name: Some(String::from("whisper-default")),
                    backend: Some(String::from("whisper")),
                    model_id: Some(String::from("tiny")),
                    model_path: Some(String::from("/path/to/whisper/model")),
                    language: Some(String::from("en")),
                    threads: Some(4),
                },
                planner_provider_settings: PlannerProviderSettings {
                    active_mode: ProviderMode::Remote,
                    available_modes: vec![ProviderMode::Remote],
                    summary: String::from("Planner currently uses configured remote profiles only."),
                },
                remote_planner_settings: RemotePlannerSettings {
                    profile_name: Some(String::from("openai-default")),
                    provider: Some(String::from("OpenAI")),
                    base_url: Some(String::from("https://api.openai.com/v1")),
                    model: Some(String::from("gpt-5.4-mini")),
                    api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                    organization_reference: None,
                    project: None,
                    temperature_milli: Some(200),
                    max_output_tokens: Some(1024),
                    timeout_ms: Some(30_000),
                },
                remote_tts_settings: RemoteTtsSettings {
                    profile_name: Some(String::from("openai-tts-default")),
                    provider: Some(String::from("OpenAI")),
                    base_url: Some(String::from("https://api.openai.com/v1")),
                    model: Some(String::from("gpt-4o-mini-tts")),
                    api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                    organization_reference: None,
                    project: None,
                    voice: Some(String::from("alloy")),
                    audio_format: Some(String::from("wav")),
                    timeout_ms: Some(30_000),
                },
                remote_asr_settings: RemoteAsrSettings {
                    profile_name: Some(String::from("openai-transcribe-default")),
                    provider: Some(String::from("OpenAI")),
                    base_url: Some(String::from("https://api.openai.com/v1")),
                    model: Some(String::from("gpt-4o-mini-transcribe")),
                    api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                    organization_reference: None,
                    project: None,
                    language: Some(String::from("en")),
                    temperature_milli: Some(0),
                    timeout_ms: Some(30_000),
                },
                provider_failover_settings: ProviderFailoverSettings {
                    planner_available: false,
                    tts_available: false,
                    asr_available: false,
                    summary: String::from("Automatic provider failover is not currently available in the live runtime."),
                },
                confirmation_settings: ConfirmationSettings {
                    confirmation_confidence_threshold: 0.9,
                    allow_click_without_confirmation: true,
                    always_confirm_submit: true,
                },
                ocr_threshold_settings: OcrThresholdSettings {
                    sparse_text_char_threshold: 200,
                    sparse_text_region_threshold: 2,
                },
            },
            safety: PlannerSafetySettings {
                confirmation_confidence_threshold: 0.9,
                allow_click_without_confirmation: true,
                always_confirm_submit: true,
            },
            available_tools: Vec::new(),
            active_skill_names: vec![String::from("open_link_by_text")],
            relevant_skill_summaries: Vec::new(),
            page_snapshot: None,
            page_model: None,
            recent_tool_results: Vec::new(),
        };

        let serialized =
            serde_json::to_value(&planner_input).expect("planner input should serialize");
        assert_eq!(
            serialized.pointer("/safety/allow_click_without_confirmation"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            serialized.pointer("/safety/always_confirm_submit"),
            Some(&serde_json::json!(true))
        );
    }

    #[test]
    fn validate_planner_output_rejects_unknown_selected_skill() {
        let available_tools = planner_available_tools();
        let planner_output = PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::GetStatus,
                goal: String::from("report the current status"),
                target_description: None,
            },
            selected_skills: vec![String::from("not-a-real-skill")],
            steps: vec![PlannedStep {
                step_id: String::from("step-status"),
                tool_name: ToolName::GetRuntimeStatus,
                arguments: serde_json::json!({
                    "request_id": "req-status",
                    "include_provider_modes": true
                }),
                purpose: String::from("read runtime status"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        };

        let error = validate_planner_output(
            &planner_output,
            &available_tools,
            &[String::from("get_status")],
        )
        .expect_err("validation should reject unknown selected skills");
        assert_eq!(error.code, "invalid_planner_output");
        assert!(error.message.contains("unknown or ineligible skill"));
    }

    #[test]
    fn validate_planner_output_rejects_submit_form_without_needs_confirmation() {
        let available_tools = planner_available_tools();
        let planner_output = PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::SubmitForm,
                goal: String::from("submit the active form"),
                target_description: Some(String::from("login form")),
            },
            selected_skills: vec![String::from("confirm_action")],
            steps: vec![PlannedStep {
                step_id: String::from("confirm-submit"),
                tool_name: ToolName::ConfirmAction,
                arguments: serde_json::json!({
                    "request_id": "req-submit",
                    "timeout_ms": 1000,
                    "prompt_text": "Submit the form now?",
                    "reason": "Submitting the form may send data."
                }),
                purpose: String::from("ask for confirmation"),
                on_success: StepTransition::RequestConfirmation,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: true,
            confirmation_reason: Some(String::from("submitting the form may send data")),
            blocked_reason: None,
            user_message: Some(String::from("Please confirm before I submit the form.")),
        };

        let error = validate_planner_output(
            &planner_output,
            &available_tools,
            &[String::from("confirm_action")],
        )
        .expect_err("submit-form plans should require NeedsConfirmation status");
        assert_eq!(error.code, "invalid_planner_output");
        assert!(error
            .message
            .contains("submit-form planner output must use NeedsConfirmation"));
    }

    #[test]
    fn validate_planner_output_rejects_submit_form_without_confirm_action_step() {
        let available_tools = planner_available_tools();
        let planner_output = PlannerOutput {
            status: PlannerStatus::NeedsConfirmation,
            intent: IntentSummary {
                name: IntentName::SubmitForm,
                goal: String::from("submit the active form"),
                target_description: Some(String::from("login form")),
            },
            selected_skills: vec![String::from("confirm_action")],
            steps: vec![PlannedStep {
                step_id: String::from("report-submit"),
                tool_name: ToolName::ReportResult,
                arguments: serde_json::json!({
                    "request_id": "req-submit",
                    "timeout_ms": 1000,
                    "status": "NeedsFollowUp",
                    "summary": "The form is ready to submit.",
                    "next_recommended_action": "Confirm the submission.",
                    "user_message": "The form is ready to submit."
                }),
                purpose: String::from("report submit readiness"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: true,
            confirmation_reason: Some(String::from("submitting the form may send data")),
            blocked_reason: None,
            user_message: Some(String::from("Please confirm before I submit the form.")),
        };

        let error = validate_planner_output(
            &planner_output,
            &available_tools,
            &[String::from("confirm_action")],
        )
        .expect_err("submit-form plans should require a confirm_action step");
        assert_eq!(error.code, "invalid_planner_output");
        assert!(error.message.contains("must include a confirm_action step"));
    }

    #[test]
    fn validate_planner_output_rejects_needs_confirmation_without_confirm_action_step() {
        let available_tools = planner_available_tools();
        let planner_output = PlannerOutput {
            status: PlannerStatus::NeedsConfirmation,
            intent: IntentSummary {
                name: IntentName::ClickElement,
                goal: String::from("activate the selected button"),
                target_description: Some(String::from("submit button")),
            },
            selected_skills: vec![String::from("open_link_by_text")],
            steps: vec![PlannedStep {
                step_id: String::from("click-button"),
                tool_name: ToolName::ClickElement,
                arguments: serde_json::json!({
                    "request_id": "req-click",
                    "timeout_ms": 1000,
                    "element_id": "button-submit",
                    "double_click": false
                }),
                purpose: String::from("activate the chosen button"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: true,
            confirmation_reason: Some(String::from("clicking may trigger a protected action")),
            blocked_reason: None,
            user_message: Some(String::from("Please confirm before I activate the button.")),
        };

        let error = validate_planner_output(
            &planner_output,
            &available_tools,
            &[String::from("open_link_by_text")],
        )
        .expect_err("needs-confirmation plans should require a confirm_action step");
        assert_eq!(error.code, "invalid_planner_output");
        assert!(error.message.contains("must include a confirm_action step"));
    }

    #[test]
    fn validate_planner_output_rejects_ready_output_with_confirmation_metadata() {
        let available_tools = planner_available_tools();
        let planner_output = PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::ClickElement,
                goal: String::from("activate the selected button"),
                target_description: Some(String::from("submit button")),
            },
            selected_skills: vec![String::from("confirm_action")],
            steps: vec![PlannedStep {
                step_id: String::from("confirm-click"),
                tool_name: ToolName::ConfirmAction,
                arguments: serde_json::json!({
                    "request_id": "req-click",
                    "timeout_ms": 1000,
                    "prompt_text": "Do you want me to activate the submit button?",
                    "reason": "Activating it may send data."
                }),
                purpose: String::from("ask for confirmation"),
                on_success: StepTransition::RequestConfirmation,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: true,
            confirmation_reason: Some(String::from("activating the button may send data")),
            blocked_reason: None,
            user_message: Some(String::from("Please confirm before I activate the button.")),
        };

        let error = validate_planner_output(
            &planner_output,
            &available_tools,
            &[String::from("confirm_action")],
        )
        .expect_err("ready plans should not carry confirmation-only metadata");
        assert_eq!(error.code, "invalid_planner_output");
        assert!(error.message.contains("must not set requires_confirmation"));
    }

    #[test]
    fn validate_planner_output_accepts_submit_form_with_confirmation_gate() {
        let available_tools = planner_available_tools();
        let planner_output = PlannerOutput {
            status: PlannerStatus::NeedsConfirmation,
            intent: IntentSummary {
                name: IntentName::SubmitForm,
                goal: String::from("submit the active form"),
                target_description: Some(String::from("login form")),
            },
            selected_skills: vec![String::from("confirm_action")],
            steps: vec![
                PlannedStep {
                    step_id: String::from("confirm-submit"),
                    tool_name: ToolName::ConfirmAction,
                    arguments: serde_json::json!({
                        "request_id": "req-submit",
                        "timeout_ms": 1000,
                        "prompt_text": "The form is filled. Do you want me to submit it now?",
                        "reason": "Submitting the form may send data."
                    }),
                    purpose: String::from("require explicit confirmation before submission"),
                    on_success: StepTransition::RequestConfirmation,
                    on_failure: StepTransition::Replan,
                },
                PlannedStep {
                    step_id: String::from("report-submit-ready"),
                    tool_name: ToolName::ReportResult,
                    arguments: serde_json::json!({
                        "request_id": "req-submit",
                        "timeout_ms": 1000,
                        "status": "NeedsFollowUp",
                        "summary": "The form is ready to submit after you confirm.",
                        "next_recommended_action": "Confirm the submission.",
                        "user_message": "Please confirm before I submit the form."
                    }),
                    purpose: String::from("keep the user informed while awaiting confirmation"),
                    on_success: StepTransition::Complete,
                    on_failure: StepTransition::Replan,
                },
            ],
            requires_confirmation: true,
            confirmation_reason: Some(String::from("submitting the form may send data")),
            blocked_reason: None,
            user_message: Some(String::from("Please confirm before I submit the form.")),
        };

        validate_planner_output(
            &planner_output,
            &available_tools,
            &[String::from("confirm_action")],
        )
        .expect("submit-form plans should validate when confirmation is required");
    }

    #[test]
    fn validate_planner_output_accepts_click_element_with_confirmation_gate() {
        let available_tools = planner_available_tools();
        let mut examples = canonical_planner_output_examples();
        let planner_output = examples
            .remove("click_element_with_confirmation")
            .expect("click confirmation example should exist");

        validate_planner_output(
            &planner_output,
            &available_tools,
            &[
                String::from("open_link_by_text"),
                String::from("confirm_action"),
            ],
        )
        .expect("click plans should validate when they use the bounded confirmation flow");
    }

    #[test]
    fn validate_planner_output_rejects_invalid_step_arguments() {
        let available_tools = planner_available_tools();
        let planner_output = PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::SetPlaybackVolume,
                goal: String::from("adjust playback volume"),
                target_description: None,
            },
            selected_skills: vec![String::from("set_volume")],
            steps: vec![PlannedStep {
                step_id: String::from("step-volume"),
                tool_name: ToolName::SetPlaybackVolume,
                arguments: serde_json::json!({
                    "request_id": "req-volume",
                    "volume": "loud"
                }),
                purpose: String::from("set playback volume"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        };

        let error = validate_planner_output(
            &planner_output,
            &available_tools,
            &[String::from("set_volume")],
        )
        .expect_err("validation should reject malformed step arguments");
        assert_eq!(error.code, "invalid_planner_output");
        assert!(error.message.contains("expected schema"));
    }

    #[test]
    fn validate_planner_output_rejects_capture_screenshot_with_multiple_targets() {
        let available_tools = planner_available_tools();
        let planner_output = PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::OcrRecovery,
                goal: String::from("capture a screenshot for OCR"),
                target_description: None,
            },
            selected_skills: vec![String::from("ocr_current_region")],
            steps: vec![PlannedStep {
                step_id: String::from("step-capture"),
                tool_name: ToolName::CaptureScreenshot,
                arguments: serde_json::json!({
                    "request_id": "req-capture",
                    "full_page": true,
                    "region_id": "region-1",
                    "bbox": serde_json::Value::Null
                }),
                purpose: String::from("capture an image for OCR"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        };

        let error = validate_planner_output(
            &planner_output,
            &available_tools,
            &[String::from("ocr_current_region")],
        )
        .expect_err("validation should reject conflicting screenshot target modes");
        assert_eq!(error.code, "invalid_planner_output");
        assert!(error
            .message
            .contains("capture_screenshot supports at most one targeting mode"));
    }

    #[test]
    fn validate_planner_output_rejects_run_ocr_without_any_source() {
        let available_tools = planner_available_tools();
        let planner_output = PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::OcrRecovery,
                goal: String::from("read text from an image"),
                target_description: None,
            },
            selected_skills: vec![String::from("ocr_current_region")],
            steps: vec![PlannedStep {
                step_id: String::from("step-run-ocr"),
                tool_name: ToolName::RunOcr,
                arguments: serde_json::json!({
                    "request_id": "req-run-ocr",
                    "image_id": serde_json::Value::Null,
                    "region_id": serde_json::Value::Null,
                    "bbox": serde_json::Value::Null
                }),
                purpose: String::from("run OCR"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        };

        let error = validate_planner_output(
            &planner_output,
            &available_tools,
            &[String::from("ocr_current_region")],
        )
        .expect_err("validation should reject run_ocr without any source image or target");
        assert_eq!(error.code, "invalid_planner_output");
        assert!(error
            .message
            .contains("run_ocr requires at least one source"));
    }

    #[test]
    fn validate_planner_output_rejects_merge_ocr_with_empty_text() {
        let available_tools = planner_available_tools();
        let planner_output = PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::OcrRecovery,
                goal: String::from("merge OCR text"),
                target_description: None,
            },
            selected_skills: vec![String::from("read_visible_text")],
            steps: vec![PlannedStep {
                step_id: String::from("step-merge-ocr"),
                tool_name: ToolName::MergeOcrIntoPageModel,
                arguments: serde_json::json!({
                    "request_id": "req-merge-ocr",
                    "page_id": "page-1",
                    "region_id": serde_json::Value::Null,
                    "ocr_text": "   ",
                    "source_bbox": serde_json::Value::Null
                }),
                purpose: String::from("merge OCR text"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        };

        let error = validate_planner_output(
            &planner_output,
            &available_tools,
            &[String::from("read_visible_text")],
        )
        .expect_err("validation should reject merge_ocr_into_page_model without OCR text");
        assert_eq!(error.code, "invalid_planner_output");
        assert!(error
            .message
            .contains("merge_ocr_into_page_model requires non-empty ocr_text"));
    }

    #[test]
    fn set_tts_voice_input_accepts_known_voice_names_only() {
        let local_voice: SetTtsVoiceInput = serde_json::from_value(serde_json::json!({
            "request_id": "req-local-voice",
            "voice": "Bruno"
        }))
        .expect("known local voice should deserialize");
        assert_eq!(local_voice.voice, TtsVoiceName::Bruno);

        let remote_voice: SetTtsVoiceInput = serde_json::from_value(serde_json::json!({
            "request_id": "req-remote-voice",
            "voice": "alloy"
        }))
        .expect("known remote voice should deserialize");
        assert_eq!(remote_voice.voice, TtsVoiceName::Alloy);

        let invalid_voice = serde_json::from_value::<SetTtsVoiceInput>(serde_json::json!({
            "request_id": "req-invalid-voice",
            "voice": "not-a-real-voice"
        }));
        assert!(invalid_voice.is_err());
    }

    #[test]
    fn validate_planner_output_rejects_open_url_with_blank_url() {
        let available_tools = planner_available_tools();
        let planner_output = PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::OpenUrl,
                goal: String::from("open a page"),
                target_description: None,
            },
            selected_skills: vec![String::from("open_url_direct")],
            steps: vec![PlannedStep {
                step_id: String::from("step-open-url"),
                tool_name: ToolName::OpenUrl,
                arguments: serde_json::json!({
                    "request_id": "req-open-url",
                    "url": "   ",
                    "wait_for_load_state": "NetworkIdle"
                }),
                purpose: String::from("open a page"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        };

        let error = validate_planner_output(
            &planner_output,
            &available_tools,
            &[String::from("open_url_direct")],
        )
        .expect_err("validation should reject blank open_url values");
        assert!(error.message.contains("open_url requires a non-empty url"));
    }

    #[test]
    fn validate_planner_output_rejects_find_element_with_blank_description() {
        let available_tools = planner_available_tools();
        let planner_output = PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::FindElement,
                goal: String::from("find an element"),
                target_description: None,
            },
            selected_skills: vec![String::from("find_element")],
            steps: vec![PlannedStep {
                step_id: String::from("step-find-element"),
                tool_name: ToolName::FindElement,
                arguments: serde_json::json!({
                    "request_id": "req-find-element",
                    "description": "   ",
                    "text": null,
                    "role": null,
                    "color_hint": null,
                    "nearby_text": null,
                    "selector_hint": null,
                    "visible_only": true,
                    "max_candidates": 3
                }),
                purpose: String::from("find an element"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        };

        let error = validate_planner_output(
            &planner_output,
            &available_tools,
            &[String::from("find_element")],
        )
        .expect_err("validation should reject blank find_element descriptions");
        assert!(error
            .message
            .contains("find_element requires a non-empty description"));
    }

    #[test]
    fn validate_planner_output_rejects_find_element_with_zero_max_candidates() {
        let available_tools = planner_available_tools();
        let planner_output = PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::FindElement,
                goal: String::from("find an element"),
                target_description: None,
            },
            selected_skills: vec![String::from("find_element")],
            steps: vec![PlannedStep {
                step_id: String::from("step-find-element"),
                tool_name: ToolName::FindElement,
                arguments: serde_json::json!({
                    "request_id": "req-find-element",
                    "description": "search field",
                    "text": null,
                    "role": null,
                    "color_hint": null,
                    "nearby_text": null,
                    "selector_hint": null,
                    "visible_only": true,
                    "max_candidates": 0
                }),
                purpose: String::from("find an element"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        };

        let error = validate_planner_output(
            &planner_output,
            &available_tools,
            &[String::from("find_element")],
        )
        .expect_err("validation should reject zero max_candidates");
        assert!(error
            .message
            .contains("find_element max_candidates must be greater than 0"));
    }

    #[test]
    fn validate_planner_output_rejects_set_playback_volume_out_of_range() {
        let available_tools = planner_available_tools();
        let planner_output = PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::SetPlaybackVolume,
                goal: String::from("set playback volume"),
                target_description: None,
            },
            selected_skills: vec![String::from("audio_controls")],
            steps: vec![PlannedStep {
                step_id: String::from("step-volume"),
                tool_name: ToolName::SetPlaybackVolume,
                arguments: serde_json::json!({
                    "request_id": "req-volume",
                    "volume": 1.5
                }),
                purpose: String::from("set the volume"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        };

        let error = validate_planner_output(
            &planner_output,
            &available_tools,
            &[String::from("audio_controls")],
        )
        .expect_err("validation should reject out-of-range playback volume");
        assert!(error
            .message
            .contains("set_playback_volume volume must be between 0.0"));
    }

    #[test]
    fn validate_planner_output_rejects_set_playback_speed_out_of_range() {
        let available_tools = planner_available_tools();
        let planner_output = PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::SetPlaybackSpeed,
                goal: String::from("set playback speed"),
                target_description: None,
            },
            selected_skills: vec![String::from("audio_controls")],
            steps: vec![PlannedStep {
                step_id: String::from("step-speed"),
                tool_name: ToolName::SetPlaybackSpeed,
                arguments: serde_json::json!({
                    "request_id": "req-speed",
                    "speed": 10.0
                }),
                purpose: String::from("set the speed"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        };

        let error = validate_planner_output(
            &planner_output,
            &available_tools,
            &[String::from("audio_controls")],
        )
        .expect_err("validation should reject out-of-range playback speed");
        assert!(error
            .message
            .contains("set_playback_speed speed must be between"));
    }

    #[test]
    fn validate_confirm_action_input_rejects_blank_prompt() {
        let error = validate_confirm_action_input(&ConfirmActionInput {
            request_id: String::from("req-confirm"),
            timeout_ms: None,
            prompt_text: String::from("   "),
            reason: String::from("Submission changes remote state."),
        })
        .expect_err("validation should reject blank confirm_action prompt_text");
        assert!(error
            .message
            .contains("confirm_action requires a non-empty prompt_text"));
    }

    #[test]
    fn infer_intent_hint_prefers_audio_queries_over_setters() {
        assert_eq!(
            infer_intent_hint("what is the volume"),
            IntentName::GetPlaybackVolume
        );
        assert_eq!(
            infer_intent_hint("what's the playback speed"),
            IntentName::GetPlaybackSpeed
        );
        assert_eq!(
            infer_intent_hint("what is the volum"),
            IntentName::GetPlaybackVolume
        );
        assert_eq!(
            infer_intent_hint("what s the play back spead"),
            IntentName::GetPlaybackSpeed
        );
    }

    #[test]
    fn infer_intent_hint_recognizes_browser_visibility_phrases() {
        assert_eq!(
            infer_intent_hint("go headless"),
            IntentName::SetBrowserVisibility
        );
        assert_eq!(
            infer_intent_hint("make the browser visible"),
            IntentName::SetBrowserVisibility
        );
        assert_eq!(
            infer_intent_hint("show the browsr"),
            IntentName::SetBrowserVisibility
        );
        assert_eq!(
            infer_intent_hint("go head less"),
            IntentName::SetBrowserVisibility
        );
    }

    #[test]
    fn infer_intent_hint_recognizes_status_and_history_queries() {
        assert_eq!(infer_intent_hint("can i go back"), IntentName::GetStatus);
        assert_eq!(
            infer_intent_hint("are you listening"),
            IntentName::GetStatus
        );
        assert_eq!(
            infer_intent_hint("what page am i on"),
            IntentName::GetCurrentUrl
        );
        assert_eq!(
            infer_intent_hint("what is the statuz"),
            IntentName::GetStatus
        );
        assert_eq!(infer_intent_hint("are you listenin"), IntentName::GetStatus);
        assert_eq!(
            infer_intent_hint("what is the curent url"),
            IntentName::GetCurrentUrl
        );
    }

    #[test]
    fn infer_intent_hint_recognizes_navigation_readback_action_phrases() {
        assert_eq!(infer_intent_hint("back"), IntentName::GoBack);
        assert_eq!(infer_intent_hint("go forward"), IntentName::GoForward);
        assert_eq!(infer_intent_hint("refesh page"), IntentName::ReloadPage);
        assert_eq!(infer_intent_hint("next"), IntentName::ReadNext);
        assert_eq!(
            infer_intent_hint("prevous region"),
            IntentName::ReadPrevious
        );
        assert_eq!(infer_intent_hint("stpo reading"), IntentName::Stop);
    }

    #[test]
    fn infer_intent_hint_recognizes_voice_input_phrases() {
        assert_eq!(
            infer_intent_hint("start listening"),
            IntentName::StartListening
        );
        assert_eq!(
            infer_intent_hint("stop listenin"),
            IntentName::StopListening
        );
        assert_eq!(
            infer_intent_hint("what did i just say"),
            IntentName::TranscribeCommand
        );
        assert_eq!(
            infer_intent_hint("transcribe this"),
            IntentName::TranscribeCommand
        );
    }

    #[test]
    fn infer_intent_hint_recognizes_open_url_phrases() {
        assert_eq!(
            infer_intent_hint("open github dot com"),
            IntentName::OpenUrl
        );
        assert_eq!(
            infer_intent_hint("go to https://example.com"),
            IntentName::OpenUrl
        );
        assert_eq!(
            infer_intent_hint("visit localhost colon 3000"),
            IntentName::OpenUrl
        );
    }

    #[test]
    fn infer_intent_hint_recognizes_read_page_phrases() {
        assert_eq!(infer_intent_hint("read page"), IntentName::ReadPage);
        assert_eq!(infer_intent_hint("read this page"), IntentName::ReadPage);
        assert_eq!(infer_intent_hint("read current page"), IntentName::ReadPage);
    }

    #[test]
    fn infer_intent_hint_recognizes_form_filling_and_submission_phrases() {
        assert_eq!(
            infer_intent_hint("focus the email field"),
            IntentName::FillInput
        );
        assert_eq!(
            infer_intent_hint("fill the password field"),
            IntentName::FillInput
        );
        assert_eq!(
            infer_intent_hint("type hello into the search field"),
            IntentName::FillInput
        );
        assert_eq!(
            infer_intent_hint("submit this form"),
            IntentName::SubmitForm
        );
        assert_eq!(
            infer_intent_hint("fill the email field and then submit"),
            IntentName::SubmitForm
        );
        assert_eq!(
            infer_intent_hint("no, the other field"),
            IntentName::FillInput
        );
        assert_eq!(
            infer_intent_hint("put Seattle there instead"),
            IntentName::FillInput
        );
        assert_eq!(
            infer_intent_hint("choose California from the state list"),
            IntentName::FillInput
        );
        assert_eq!(
            infer_intent_hint("foccus the email feild"),
            IntentName::FillInput
        );
        assert_eq!(
            infer_intent_hint("submitt this form"),
            IntentName::SubmitForm
        );
    }

    #[test]
    fn parse_direct_focus_field_command_extracts_field_description() {
        assert_eq!(
            parse_direct_focus_field_command("focus the email field"),
            Some(FocusFieldCommand {
                description: Some(String::from("email"))
            })
        );
        assert_eq!(
            parse_direct_focus_field_command("foccus the password feild"),
            Some(FocusFieldCommand {
                description: Some(String::from("password"))
            })
        );
        assert_eq!(
            parse_direct_focus_field_command("focus field"),
            Some(FocusFieldCommand { description: None })
        );
        assert_eq!(parse_direct_focus_field_command("read page"), None);
    }

    #[test]
    fn parse_direct_fill_field_command_extracts_description_and_text() {
        assert_eq!(
            parse_direct_fill_field_command("fill the email field with phil@example.com"),
            Some(FillFieldCommand {
                description: Some(String::from("email")),
                text: Some(String::from("phil@example.com"))
            })
        );
        assert_eq!(
            parse_direct_fill_field_command("type \"hello world\" into the search field"),
            Some(FillFieldCommand {
                description: Some(String::from("search")),
                text: Some(String::from("hello world"))
            })
        );
        assert_eq!(
            parse_direct_fill_field_command("enter secret in the password field"),
            Some(FillFieldCommand {
                description: Some(String::from("password")),
                text: Some(String::from("secret"))
            })
        );
        assert_eq!(
            parse_direct_fill_field_command("fill the email field"),
            Some(FillFieldCommand {
                description: Some(String::from("email")),
                text: None
            })
        );
        assert_eq!(
            parse_direct_fill_field_command("focus the email field"),
            None
        );
    }

    #[test]
    fn parse_direct_fill_and_submit_command_extracts_description_and_text() {
        assert_eq!(
            parse_direct_fill_and_submit_command(
                "fill the email field with phil@example.com and then submit"
            ),
            Some(FillFieldCommand {
                description: Some(String::from("email")),
                text: Some(String::from("phil@example.com"))
            })
        );
        assert_eq!(
            parse_direct_fill_and_submit_command(
                "type hello world into the search field and submit form"
            ),
            Some(FillFieldCommand {
                description: Some(String::from("search")),
                text: Some(String::from("hello world"))
            })
        );
        assert_eq!(
            parse_direct_fill_and_submit_command("fill the email field and submit"),
            Some(FillFieldCommand {
                description: Some(String::from("email")),
                text: None
            })
        );
        assert_eq!(parse_direct_fill_and_submit_command("submit form"), None);
    }

    #[test]
    fn infer_intent_hint_recognizes_repeat_phrases() {
        assert_eq!(infer_intent_hint("repeat"), IntentName::Repeat);
        assert_eq!(infer_intent_hint("repeat that"), IntentName::Repeat);
        assert_eq!(infer_intent_hint("read that again"), IntentName::Repeat);
        assert_eq!(infer_intent_hint("say that again"), IntentName::Repeat);
    }

    #[test]
    fn infer_intent_hint_recognizes_read_title_phrases() {
        assert_eq!(infer_intent_hint("read title"), IntentName::ReadTitle);
        assert_eq!(
            infer_intent_hint("read the page title"),
            IntentName::ReadTitle
        );
        assert_eq!(
            infer_intent_hint("what is the title"),
            IntentName::ReadTitle
        );
    }

    #[test]
    fn infer_intent_hint_recognizes_tts_voice_phrases() {
        assert_eq!(
            infer_intent_hint("change the voice to Bruno"),
            IntentName::SetTtsVoice
        );
        assert_eq!(
            infer_intent_hint("switch to the Bella voice"),
            IntentName::SetTtsVoice
        );
        assert_eq!(
            infer_intent_hint("use the Hugo voice"),
            IntentName::SetTtsVoice
        );
        assert_eq!(
            infer_intent_hint("set the voise to Luna"),
            IntentName::SetTtsVoice
        );
    }

    #[test]
    fn resolve_direct_audio_command_normalizes_absolute_volume_percent() {
        let planner_output = resolve_direct_audio_command(
            "set volume to 70 percent",
            "req-volume",
            1.0,
            1.0,
            &[String::from("set_volume")],
        )
        .expect("volume command should normalize");

        assert_eq!(planner_output.intent.name, IntentName::SetPlaybackVolume);
        assert_eq!(
            planner_output.selected_skills,
            vec![String::from("set_volume")]
        );
        assert_eq!(planner_output.steps.len(), 2);
        assert_eq!(
            planner_output.steps[0].tool_name,
            ToolName::SetPlaybackVolume
        );
        let volume = planner_output.steps[0]
            .arguments
            .get("volume")
            .and_then(serde_json::Value::as_f64)
            .expect("volume should be numeric");
        assert!((volume - 0.7).abs() < 0.000_001);
        assert_eq!(planner_output.steps[1].tool_name, ToolName::ReportResult);
        assert_eq!(
            planner_output.steps[1].arguments.get("summary"),
            Some(&serde_json::json!("Playback volume set to 70%."))
        );

        let fuzzy_planner_output = resolve_direct_audio_command(
            "set volum to 70 percent",
            "req-volume-fuzzy",
            1.0,
            1.0,
            &[String::from("set_volume")],
        )
        .expect("fuzzy volume command should normalize");

        let fuzzy_volume = fuzzy_planner_output.steps[0]
            .arguments
            .get("volume")
            .and_then(serde_json::Value::as_f64)
            .expect("fuzzy volume should be numeric");
        assert!((fuzzy_volume - 0.7).abs() < 0.000_001);
    }

    #[test]
    fn resolve_direct_audio_command_applies_large_relative_speed_step() {
        let planner_output = resolve_direct_audio_command(
            "go faster a lot",
            "req-speed",
            1.0,
            1.0,
            &[String::from("increase_playback_speed")],
        )
        .expect("speed command should normalize");

        assert_eq!(planner_output.intent.name, IntentName::SetPlaybackSpeed);
        assert_eq!(
            planner_output.selected_skills,
            vec![String::from("increase_playback_speed")]
        );
        assert_eq!(
            planner_output.steps[0].arguments.get("speed"),
            Some(&serde_json::json!(1.5))
        );
        assert_eq!(
            planner_output.steps[1].arguments.get("summary"),
            Some(&serde_json::json!("Playback speed set to 1.5x."))
        );
    }

    #[test]
    fn resolve_direct_audio_command_reports_current_speed_for_queries() {
        let planner_output = resolve_direct_audio_command(
            "tell me the speed",
            "req-speed-query",
            0.8,
            1.25,
            &[String::from("get_playback_speed")],
        )
        .expect("speed query should normalize");

        assert_eq!(planner_output.intent.name, IntentName::GetPlaybackSpeed);
        assert_eq!(
            planner_output.selected_skills,
            vec![String::from("get_playback_speed")]
        );
        assert_eq!(planner_output.steps.len(), 1);
        assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
        assert_eq!(
            planner_output.steps[0].arguments.get("summary"),
            Some(&serde_json::json!("Playback speed is 1.25x."))
        );
    }

    #[test]
    fn resolve_direct_browser_visibility_command_normalizes_headless_phrase() {
        let planner_output = resolve_direct_browser_visibility_command(
            "go headless",
            "req-headless",
            BrowserVisibilityMode::Visible,
            &[String::from("toggle_browser_visibility")],
        )
        .expect("visibility command should normalize");

        assert_eq!(planner_output.intent.name, IntentName::SetBrowserVisibility);
        assert_eq!(
            planner_output.selected_skills,
            vec![String::from("toggle_browser_visibility")]
        );
        assert_eq!(planner_output.steps.len(), 2);
        assert_eq!(
            planner_output.steps[0].arguments.get("mode"),
            Some(&serde_json::json!(BrowserVisibilityMode::Headless))
        );
        assert_eq!(
            planner_output.steps[1].arguments.get("summary"),
            Some(&serde_json::json!("Browser mode set to headless."))
        );

        let fuzzy_planner_output = resolve_direct_browser_visibility_command(
            "show the browsr",
            "req-visible-fuzzy",
            BrowserVisibilityMode::Headless,
            &[String::from("toggle_browser_visibility")],
        )
        .expect("fuzzy visibility command should normalize");

        assert_eq!(
            fuzzy_planner_output.steps[0].arguments.get("mode"),
            Some(&serde_json::json!(BrowserVisibilityMode::Visible))
        );
    }

    #[test]
    fn resolve_direct_browser_visibility_command_toggles_when_requested() {
        let planner_output = resolve_direct_browser_visibility_command(
            "toggle browser visibility",
            "req-toggle",
            BrowserVisibilityMode::Headless,
            &[String::from("toggle_browser_visibility")],
        )
        .expect("toggle visibility command should normalize");

        assert_eq!(
            planner_output.steps[0].arguments.get("mode"),
            Some(&serde_json::json!(BrowserVisibilityMode::Visible))
        );
        assert_eq!(
            planner_output.steps[1].arguments.get("summary"),
            Some(&serde_json::json!("Browser mode set to visible."))
        );
    }

    #[test]
    fn resolve_direct_navigation_readback_command_builds_history_and_reload_plans() {
        let go_back_plan = resolve_direct_navigation_readback_command(
            "back",
            "req-back",
            &[String::from("go_back")],
        )
        .expect("back command should normalize");

        assert_eq!(go_back_plan.intent.name, IntentName::GoBack);
        assert_eq!(go_back_plan.selected_skills, vec![String::from("go_back")]);
        assert_eq!(go_back_plan.steps.len(), 1);
        assert_eq!(go_back_plan.steps[0].tool_name, ToolName::GoBack);
        assert_eq!(
            go_back_plan.steps[0].arguments.get("steps"),
            Some(&serde_json::json!(1))
        );
        assert_eq!(
            go_back_plan.steps[0].arguments.get("wait_for_load_state"),
            Some(&serde_json::json!(LoadState::Load))
        );

        let reload_plan = resolve_direct_navigation_readback_command(
            "refesh page",
            "req-reload",
            &[String::from("reload_page")],
        )
        .expect("reload command should normalize");

        assert_eq!(reload_plan.intent.name, IntentName::ReloadPage);
        assert_eq!(
            reload_plan.selected_skills,
            vec![String::from("reload_page")]
        );
        assert_eq!(reload_plan.steps[0].tool_name, ToolName::ReloadPage);
        assert_eq!(
            reload_plan.steps[0].arguments.get("hard_reload"),
            Some(&serde_json::json!(false))
        );
    }

    #[test]
    fn resolve_direct_navigation_readback_command_builds_reading_and_stop_plans() {
        let next_plan = resolve_direct_navigation_readback_command(
            "continue reading",
            "req-next",
            &[String::from("read_next")],
        )
        .expect("next command should normalize");

        assert_eq!(next_plan.intent.name, IntentName::ReadNext);
        assert_eq!(next_plan.selected_skills, vec![String::from("read_next")]);
        assert_eq!(next_plan.steps[0].tool_name, ToolName::ReadNextRegion);
        assert_eq!(
            next_plan.steps[0].arguments.get("interrupt_current"),
            Some(&serde_json::json!(true))
        );

        let previous_plan = resolve_direct_navigation_readback_command(
            "prevous section",
            "req-previous",
            &[String::from("read_previous")],
        )
        .expect("previous command should normalize");

        assert_eq!(previous_plan.intent.name, IntentName::ReadPrevious);
        assert_eq!(
            previous_plan.selected_skills,
            vec![String::from("read_previous")]
        );
        assert_eq!(
            previous_plan.steps[0].tool_name,
            ToolName::ReadPreviousRegion
        );

        let stop_plan = resolve_direct_navigation_readback_command(
            "stpo reading",
            "req-stop",
            &[String::from("stop_reading")],
        )
        .expect("stop command should normalize");

        assert_eq!(stop_plan.intent.name, IntentName::Stop);
        assert_eq!(
            stop_plan.selected_skills,
            vec![String::from("stop_reading")]
        );
        assert_eq!(stop_plan.steps[0].tool_name, ToolName::StopSpeaking);
    }

    #[test]
    fn resolve_direct_voice_input_command_builds_start_and_stop_listening_plans() {
        let start_plan = resolve_direct_voice_input_command(
            "start listening",
            "req-start-listening",
            &[String::from("start_listening")],
        )
        .expect("start listening command should normalize");

        assert_eq!(start_plan.intent.name, IntentName::StartListening);
        assert_eq!(
            start_plan.selected_skills,
            vec![String::from("start_listening")]
        );
        assert_eq!(start_plan.steps[0].tool_name, ToolName::StartListening);

        let stop_plan = resolve_direct_voice_input_command(
            "stop listenin",
            "req-stop-listening",
            &[String::from("stop_listening")],
        )
        .expect("stop listening command should normalize");

        assert_eq!(stop_plan.intent.name, IntentName::StopListening);
        assert_eq!(
            stop_plan.selected_skills,
            vec![String::from("stop_listening")]
        );
        assert_eq!(stop_plan.steps[0].tool_name, ToolName::StopListening);
    }

    #[test]
    fn resolve_direct_voice_input_command_builds_transcribe_plan() {
        let planner_output = resolve_direct_voice_input_command(
            "what did i just say",
            "req-transcribe",
            &[String::from("transcribe_command")],
        )
        .expect("transcribe command should normalize");

        assert_eq!(planner_output.intent.name, IntentName::TranscribeCommand);
        assert_eq!(
            planner_output.selected_skills,
            vec![String::from("transcribe_command")]
        );
        assert_eq!(planner_output.steps.len(), 1);
        assert_eq!(
            planner_output.steps[0].tool_name,
            ToolName::TranscribeCommand
        );
        assert_eq!(
            planner_output.steps[0].arguments.get("auto_stop"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            planner_output.steps[0].arguments.get("max_duration_ms"),
            Some(&serde_json::Value::Null)
        );
    }

    #[test]
    fn resolve_direct_open_url_command_normalizes_spoken_and_absolute_urls() {
        let spoken_plan = resolve_direct_open_url_command(
            "open github dot com slash features",
            "req-open-spoken",
            &[String::from("open_url")],
        )
        .expect("spoken open-url command should normalize");

        assert_eq!(spoken_plan.intent.name, IntentName::OpenUrl);
        assert_eq!(spoken_plan.selected_skills, vec![String::from("open_url")]);
        assert_eq!(spoken_plan.steps.len(), 1);
        assert_eq!(spoken_plan.steps[0].tool_name, ToolName::OpenUrl);
        assert_eq!(
            spoken_plan.steps[0].arguments.get("url"),
            Some(&serde_json::json!("https://github.com/features"))
        );
        assert_eq!(
            spoken_plan.steps[0].arguments.get("wait_for_load_state"),
            Some(&serde_json::json!(LoadState::Load))
        );

        let localhost_plan = resolve_direct_open_url_command(
            "visit localhost colon 3000",
            "req-open-localhost",
            &[String::from("open_url")],
        )
        .expect("localhost command should normalize");

        assert_eq!(
            localhost_plan.steps[0].arguments.get("url"),
            Some(&serde_json::json!("http://localhost:3000"))
        );

        let absolute_plan = resolve_direct_open_url_command(
            "go to https://example.com/docs",
            "req-open-absolute",
            &[String::from("open_url")],
        )
        .expect("absolute open-url command should normalize");

        assert_eq!(
            absolute_plan.steps[0].arguments.get("url"),
            Some(&serde_json::json!("https://example.com/docs"))
        );
    }

    #[test]
    fn resolve_direct_read_page_command_reads_from_first_region_when_available() {
        let page_model = PageModel {
            title: Some(String::from("Example page")),
            url: Some(String::from("https://example.com/article")),
            regions: vec![
                crate::page_model::PageRegion {
                    region_id: String::from("region-1"),
                    label: Some(String::from("Main")),
                    text: String::from("Welcome to the article."),
                    bbox: None,
                    source: crate::page_model::RegionSource::Dom,
                },
                crate::page_model::PageRegion {
                    region_id: String::from("region-2"),
                    label: Some(String::from("Details")),
                    text: String::from("More details."),
                    bbox: None,
                    source: crate::page_model::RegionSource::Dom,
                },
            ],
            interactive_elements: Vec::new(),
        };
        let agent_state = AgentStateData {
            page_id: Some(String::from("page-1")),
            url: page_model.url.clone(),
            title: page_model.title.clone(),
            browser_visibility: BrowserVisibilityMode::Visible,
            browser_history: BrowserHistoryState::default(),
            narration_cursor: None,
            speaking: false,
            listening_state: ListeningState::default(),
            audio: RuntimeAudioState::default(),
            last_transcript: None,
            last_tool_call: None,
            pending_confirmation_id: None,
            pending_plan_execution: None,
            tts_model_settings: TtsModelSettings {
                mode: ProviderMode::Local,
                active_profile: Some(String::from("kitten-default")),
                available_profiles: vec![TtsModelOption {
                    profile_name: String::from("kitten-default"),
                    model_label: String::from("default"),
                }],
            },
            local_tts_model_settings: LocalTtsModelSettings {
                profile_name: Some(String::from("kitten-default")),
                backend: Some(String::from("kitten_tts_rs")),
                model_id: Some(String::from("default")),
                model_path: Some(String::from("/path/to/kitten/model")),
                default_voice: Some(String::from("Bruno")),
                sample_rate: Some(24_000),
            },
            tts_voice_settings: TtsVoiceSettings {
                mode: ProviderMode::Local,
                active_voice: Some(String::from("Bruno")),
                available_voices: vec![
                    TtsVoiceOption {
                        voice_name: String::from("Bella"),
                        display_label: String::from("Bella"),
                    },
                    TtsVoiceOption {
                        voice_name: String::from("Bruno"),
                        display_label: String::from("Bruno"),
                    },
                ],
            },
            tts_provider_settings: TtsProviderSettings {
                active_mode: ProviderMode::Local,
                available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
            },
            asr_provider_settings: AsrProviderSettings {
                active_mode: ProviderMode::Local,
                available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
            },
            local_asr_model_settings: LocalAsrModelSettings {
                profile_name: Some(String::from("whisper-default")),
                backend: Some(String::from("whisper")),
                model_id: Some(String::from("tiny")),
                model_path: Some(String::from("/path/to/whisper/model")),
                language: Some(String::from("en")),
                threads: Some(4),
            },
            planner_provider_settings: PlannerProviderSettings {
                active_mode: ProviderMode::Remote,
                available_modes: vec![ProviderMode::Remote],
                summary: String::from("Planner currently uses configured remote profiles only."),
            },
            remote_planner_settings: RemotePlannerSettings {
                profile_name: Some(String::from("openai-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-5.4-mini")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                temperature_milli: Some(200),
                max_output_tokens: Some(1024),
                timeout_ms: Some(30_000),
            },
            remote_tts_settings: RemoteTtsSettings {
                profile_name: Some(String::from("openai-tts-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-4o-mini-tts")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                voice: Some(String::from("alloy")),
                audio_format: Some(String::from("wav")),
                timeout_ms: Some(30_000),
            },
            remote_asr_settings: RemoteAsrSettings {
                profile_name: Some(String::from("openai-transcribe-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-4o-mini-transcribe")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                language: Some(String::from("en")),
                temperature_milli: Some(0),
                timeout_ms: Some(30_000),
            },
            provider_failover_settings: ProviderFailoverSettings {
                planner_available: false,
                tts_available: false,
                asr_available: false,
                summary: String::from(
                    "Automatic provider failover is not currently available in the live runtime.",
                ),
            },
            confirmation_settings: ConfirmationSettings {
                confirmation_confidence_threshold: 0.9,
                allow_click_without_confirmation: true,
                always_confirm_submit: true,
            },
            ocr_threshold_settings: OcrThresholdSettings {
                sparse_text_char_threshold: 200,
                sparse_text_region_threshold: 2,
            },
        };

        let planner_output = resolve_direct_read_page_command(
            "read this page",
            "req-read-page",
            Some(&page_model),
            &agent_state,
            &[String::from("read_page")],
        )
        .expect("read-page command should resolve");

        assert_eq!(planner_output.intent.name, IntentName::ReadPage);
        assert_eq!(
            planner_output.selected_skills,
            vec![String::from("read_page")]
        );
        assert_eq!(planner_output.steps.len(), 1);
        assert_eq!(planner_output.steps[0].tool_name, ToolName::ReadRegion);
        assert_eq!(
            planner_output.steps[0].arguments.get("region_id"),
            Some(&serde_json::json!("region-1"))
        );
        assert_eq!(
            planner_output.steps[0].arguments.get("interrupt_current"),
            Some(&serde_json::json!(true))
        );
    }

    #[test]
    fn resolve_direct_read_page_command_extracts_then_reads_when_regions_missing() {
        let page_model = PageModel {
            title: Some(String::from("Example page")),
            url: Some(String::from("https://example.com/article")),
            regions: Vec::new(),
            interactive_elements: Vec::new(),
        };
        let agent_state = AgentStateData {
            page_id: Some(String::from("page-1")),
            url: page_model.url.clone(),
            title: page_model.title.clone(),
            browser_visibility: BrowserVisibilityMode::Visible,
            browser_history: BrowserHistoryState::default(),
            narration_cursor: None,
            speaking: false,
            listening_state: ListeningState::default(),
            audio: RuntimeAudioState::default(),
            last_transcript: None,
            last_tool_call: None,
            pending_confirmation_id: None,
            pending_plan_execution: None,
            tts_model_settings: TtsModelSettings {
                mode: ProviderMode::Local,
                active_profile: Some(String::from("kitten-default")),
                available_profiles: vec![TtsModelOption {
                    profile_name: String::from("kitten-default"),
                    model_label: String::from("default"),
                }],
            },
            local_tts_model_settings: LocalTtsModelSettings {
                profile_name: Some(String::from("kitten-default")),
                backend: Some(String::from("kitten_tts_rs")),
                model_id: Some(String::from("default")),
                model_path: Some(String::from("/path/to/kitten/model")),
                default_voice: Some(String::from("Bruno")),
                sample_rate: Some(24_000),
            },
            tts_voice_settings: TtsVoiceSettings {
                mode: ProviderMode::Local,
                active_voice: Some(String::from("Bruno")),
                available_voices: vec![
                    TtsVoiceOption {
                        voice_name: String::from("Bella"),
                        display_label: String::from("Bella"),
                    },
                    TtsVoiceOption {
                        voice_name: String::from("Bruno"),
                        display_label: String::from("Bruno"),
                    },
                ],
            },
            tts_provider_settings: TtsProviderSettings {
                active_mode: ProviderMode::Local,
                available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
            },
            asr_provider_settings: AsrProviderSettings {
                active_mode: ProviderMode::Local,
                available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
            },
            local_asr_model_settings: LocalAsrModelSettings {
                profile_name: Some(String::from("whisper-default")),
                backend: Some(String::from("whisper")),
                model_id: Some(String::from("tiny")),
                model_path: Some(String::from("/path/to/whisper/model")),
                language: Some(String::from("en")),
                threads: Some(4),
            },
            planner_provider_settings: PlannerProviderSettings {
                active_mode: ProviderMode::Remote,
                available_modes: vec![ProviderMode::Remote],
                summary: String::from("Planner currently uses configured remote profiles only."),
            },
            remote_planner_settings: RemotePlannerSettings {
                profile_name: Some(String::from("openai-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-5.4-mini")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                temperature_milli: Some(200),
                max_output_tokens: Some(1024),
                timeout_ms: Some(30_000),
            },
            remote_tts_settings: RemoteTtsSettings {
                profile_name: Some(String::from("openai-tts-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-4o-mini-tts")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                voice: Some(String::from("alloy")),
                audio_format: Some(String::from("wav")),
                timeout_ms: Some(30_000),
            },
            remote_asr_settings: RemoteAsrSettings {
                profile_name: Some(String::from("openai-transcribe-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-4o-mini-transcribe")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                language: Some(String::from("en")),
                temperature_milli: Some(0),
                timeout_ms: Some(30_000),
            },
            provider_failover_settings: ProviderFailoverSettings {
                planner_available: false,
                tts_available: false,
                asr_available: false,
                summary: String::from(
                    "Automatic provider failover is not currently available in the live runtime.",
                ),
            },
            confirmation_settings: ConfirmationSettings {
                confirmation_confidence_threshold: 0.9,
                allow_click_without_confirmation: true,
                always_confirm_submit: true,
            },
            ocr_threshold_settings: OcrThresholdSettings {
                sparse_text_char_threshold: 200,
                sparse_text_region_threshold: 2,
            },
        };

        let planner_output = resolve_direct_read_page_command(
            "read page",
            "req-read-page-extract",
            Some(&page_model),
            &agent_state,
            &[String::from("read_page")],
        )
        .expect("read-page command should resolve");

        assert_eq!(planner_output.steps.len(), 2);
        assert_eq!(
            planner_output.steps[0].tool_name,
            ToolName::ExtractPageModel
        );
        assert_eq!(
            planner_output.steps[0].arguments.get("use_dom_extraction"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(planner_output.steps[1].tool_name, ToolName::ReadNextRegion);
        assert_eq!(
            planner_output.steps[1].arguments.get("interrupt_current"),
            Some(&serde_json::json!(true))
        );
    }

    #[test]
    fn resolve_direct_read_page_command_reports_missing_active_page() {
        let agent_state = AgentStateData {
            page_id: None,
            url: None,
            title: None,
            browser_visibility: BrowserVisibilityMode::Visible,
            browser_history: BrowserHistoryState::default(),
            narration_cursor: None,
            speaking: false,
            listening_state: ListeningState::default(),
            audio: RuntimeAudioState::default(),
            last_transcript: None,
            last_tool_call: None,
            pending_confirmation_id: None,
            pending_plan_execution: None,
            tts_model_settings: TtsModelSettings {
                mode: ProviderMode::Local,
                active_profile: Some(String::from("kitten-default")),
                available_profiles: vec![TtsModelOption {
                    profile_name: String::from("kitten-default"),
                    model_label: String::from("default"),
                }],
            },
            local_tts_model_settings: LocalTtsModelSettings {
                profile_name: Some(String::from("kitten-default")),
                backend: Some(String::from("kitten_tts_rs")),
                model_id: Some(String::from("default")),
                model_path: Some(String::from("/path/to/kitten/model")),
                default_voice: Some(String::from("Bruno")),
                sample_rate: Some(24_000),
            },
            tts_voice_settings: TtsVoiceSettings {
                mode: ProviderMode::Local,
                active_voice: Some(String::from("Bruno")),
                available_voices: vec![
                    TtsVoiceOption {
                        voice_name: String::from("Bella"),
                        display_label: String::from("Bella"),
                    },
                    TtsVoiceOption {
                        voice_name: String::from("Bruno"),
                        display_label: String::from("Bruno"),
                    },
                ],
            },
            tts_provider_settings: TtsProviderSettings {
                active_mode: ProviderMode::Local,
                available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
            },
            asr_provider_settings: AsrProviderSettings {
                active_mode: ProviderMode::Local,
                available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
            },
            local_asr_model_settings: LocalAsrModelSettings {
                profile_name: Some(String::from("whisper-default")),
                backend: Some(String::from("whisper")),
                model_id: Some(String::from("tiny")),
                model_path: Some(String::from("/path/to/whisper/model")),
                language: Some(String::from("en")),
                threads: Some(4),
            },
            planner_provider_settings: PlannerProviderSettings {
                active_mode: ProviderMode::Remote,
                available_modes: vec![ProviderMode::Remote],
                summary: String::from("Planner currently uses configured remote profiles only."),
            },
            remote_planner_settings: RemotePlannerSettings {
                profile_name: Some(String::from("openai-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-5.4-mini")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                temperature_milli: Some(200),
                max_output_tokens: Some(1024),
                timeout_ms: Some(30_000),
            },
            remote_tts_settings: RemoteTtsSettings {
                profile_name: Some(String::from("openai-tts-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-4o-mini-tts")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                voice: Some(String::from("alloy")),
                audio_format: Some(String::from("wav")),
                timeout_ms: Some(30_000),
            },
            remote_asr_settings: RemoteAsrSettings {
                profile_name: Some(String::from("openai-transcribe-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-4o-mini-transcribe")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                language: Some(String::from("en")),
                temperature_milli: Some(0),
                timeout_ms: Some(30_000),
            },
            provider_failover_settings: ProviderFailoverSettings {
                planner_available: false,
                tts_available: false,
                asr_available: false,
                summary: String::from(
                    "Automatic provider failover is not currently available in the live runtime.",
                ),
            },
            confirmation_settings: ConfirmationSettings {
                confirmation_confidence_threshold: 0.9,
                allow_click_without_confirmation: true,
                always_confirm_submit: true,
            },
            ocr_threshold_settings: OcrThresholdSettings {
                sparse_text_char_threshold: 200,
                sparse_text_region_threshold: 2,
            },
        };

        let planner_output = resolve_direct_read_page_command(
            "read current page",
            "req-read-page-missing",
            None,
            &agent_state,
            &[String::from("read_page")],
        )
        .expect("read-page command should resolve");

        assert_eq!(planner_output.steps.len(), 1);
        assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
        assert_eq!(
            planner_output.steps[0].arguments.get("status"),
            Some(&serde_json::json!(ReportStatus::NeedsFollowUp))
        );
    }

    #[test]
    fn resolve_direct_status_query_command_reports_current_url() {
        let agent_state = AgentStateData {
            page_id: Some(String::from("page-1")),
            url: Some(String::from("https://example.com/article")),
            title: Some(String::from("Example article")),
            browser_visibility: BrowserVisibilityMode::Visible,
            browser_history: BrowserHistoryState::default(),
            narration_cursor: Some(NarrationCursor::default()),
            speaking: false,
            listening_state: ListeningState::default(),
            audio: RuntimeAudioState::default(),
            last_transcript: None,
            last_tool_call: None,
            pending_confirmation_id: None,
            pending_plan_execution: None,
            tts_model_settings: TtsModelSettings {
                mode: ProviderMode::Local,
                active_profile: Some(String::from("kitten-default")),
                available_profiles: vec![TtsModelOption {
                    profile_name: String::from("kitten-default"),
                    model_label: String::from("default"),
                }],
            },
            local_tts_model_settings: LocalTtsModelSettings {
                profile_name: Some(String::from("kitten-default")),
                backend: Some(String::from("kitten_tts_rs")),
                model_id: Some(String::from("default")),
                model_path: Some(String::from("/path/to/kitten/model")),
                default_voice: Some(String::from("Bruno")),
                sample_rate: Some(24_000),
            },
            tts_voice_settings: TtsVoiceSettings {
                mode: ProviderMode::Local,
                active_voice: Some(String::from("Bruno")),
                available_voices: vec![
                    TtsVoiceOption {
                        voice_name: String::from("Bella"),
                        display_label: String::from("Bella"),
                    },
                    TtsVoiceOption {
                        voice_name: String::from("Bruno"),
                        display_label: String::from("Bruno"),
                    },
                ],
            },
            tts_provider_settings: TtsProviderSettings {
                active_mode: ProviderMode::Local,
                available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
            },
            asr_provider_settings: AsrProviderSettings {
                active_mode: ProviderMode::Local,
                available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
            },
            local_asr_model_settings: LocalAsrModelSettings {
                profile_name: Some(String::from("whisper-default")),
                backend: Some(String::from("whisper")),
                model_id: Some(String::from("tiny")),
                model_path: Some(String::from("/path/to/whisper/model")),
                language: Some(String::from("en")),
                threads: Some(4),
            },
            planner_provider_settings: PlannerProviderSettings {
                active_mode: ProviderMode::Remote,
                available_modes: vec![ProviderMode::Remote],
                summary: String::from("Planner currently uses configured remote profiles only."),
            },
            remote_planner_settings: RemotePlannerSettings {
                profile_name: Some(String::from("openai-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-5.4-mini")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                temperature_milli: Some(200),
                max_output_tokens: Some(1024),
                timeout_ms: Some(30_000),
            },
            remote_tts_settings: RemoteTtsSettings {
                profile_name: Some(String::from("openai-tts-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-4o-mini-tts")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                voice: Some(String::from("alloy")),
                audio_format: Some(String::from("wav")),
                timeout_ms: Some(30_000),
            },
            remote_asr_settings: RemoteAsrSettings {
                profile_name: Some(String::from("openai-transcribe-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-4o-mini-transcribe")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                language: Some(String::from("en")),
                temperature_milli: Some(0),
                timeout_ms: Some(30_000),
            },
            provider_failover_settings: ProviderFailoverSettings {
                planner_available: false,
                tts_available: false,
                asr_available: false,
                summary: String::from(
                    "Automatic provider failover is not currently available in the live runtime.",
                ),
            },
            confirmation_settings: ConfirmationSettings {
                confirmation_confidence_threshold: 0.9,
                allow_click_without_confirmation: true,
                always_confirm_submit: true,
            },
            ocr_threshold_settings: OcrThresholdSettings {
                sparse_text_char_threshold: 200,
                sparse_text_region_threshold: 2,
            },
        };
        let runtime_status = GetRuntimeStatusData {
            page_id: agent_state.page_id.clone(),
            url: agent_state.url.clone(),
            title: agent_state.title.clone(),
            browser_visibility: agent_state.browser_visibility,
            browser_history: agent_state.browser_history.clone(),
            listening_state: agent_state.listening_state.clone(),
            speaking: agent_state.speaking,
            audio: agent_state.audio.clone(),
            pending_confirmation_id: None,
            pending_plan_execution: None,
            provider_modes: None,
        };

        let planner_output = resolve_direct_status_query_command(
            "what page am i on",
            "req-current-url",
            &agent_state,
            &runtime_status,
            &[String::from("get_current_url")],
        )
        .expect("current url query should normalize");

        assert_eq!(planner_output.intent.name, IntentName::GetCurrentUrl);
        assert_eq!(
            planner_output.selected_skills,
            vec![String::from("get_current_url")]
        );
        assert_eq!(planner_output.steps[0].tool_name, ToolName::GetAgentState);
        assert_eq!(
            planner_output.steps[1].arguments.get("summary"),
            Some(&serde_json::json!(
                "Current page is Example article at https://example.com/article."
            ))
        );
    }

    #[test]
    fn resolve_direct_status_query_command_reports_back_history_availability() {
        let agent_state = AgentStateData {
            page_id: Some(String::from("page-1")),
            url: Some(String::from("https://example.com/article")),
            title: Some(String::from("Example article")),
            browser_visibility: BrowserVisibilityMode::Visible,
            browser_history: BrowserHistoryState {
                can_go_back: true,
                can_go_forward: false,
                current_entry_index: Some(1),
                entry_count: 2,
            },
            narration_cursor: Some(NarrationCursor::default()),
            speaking: false,
            listening_state: ListeningState::default(),
            audio: RuntimeAudioState::default(),
            last_transcript: None,
            last_tool_call: None,
            pending_confirmation_id: None,
            pending_plan_execution: None,
            tts_model_settings: TtsModelSettings {
                mode: ProviderMode::Local,
                active_profile: Some(String::from("kitten-default")),
                available_profiles: vec![TtsModelOption {
                    profile_name: String::from("kitten-default"),
                    model_label: String::from("default"),
                }],
            },
            local_tts_model_settings: LocalTtsModelSettings {
                profile_name: Some(String::from("kitten-default")),
                backend: Some(String::from("kitten_tts_rs")),
                model_id: Some(String::from("default")),
                model_path: Some(String::from("/path/to/kitten/model")),
                default_voice: Some(String::from("Bruno")),
                sample_rate: Some(24_000),
            },
            tts_voice_settings: TtsVoiceSettings {
                mode: ProviderMode::Local,
                active_voice: Some(String::from("Bruno")),
                available_voices: vec![
                    TtsVoiceOption {
                        voice_name: String::from("Bella"),
                        display_label: String::from("Bella"),
                    },
                    TtsVoiceOption {
                        voice_name: String::from("Bruno"),
                        display_label: String::from("Bruno"),
                    },
                ],
            },
            tts_provider_settings: TtsProviderSettings {
                active_mode: ProviderMode::Local,
                available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
            },
            asr_provider_settings: AsrProviderSettings {
                active_mode: ProviderMode::Local,
                available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
            },
            local_asr_model_settings: LocalAsrModelSettings {
                profile_name: Some(String::from("whisper-default")),
                backend: Some(String::from("whisper")),
                model_id: Some(String::from("tiny")),
                model_path: Some(String::from("/path/to/whisper/model")),
                language: Some(String::from("en")),
                threads: Some(4),
            },
            planner_provider_settings: PlannerProviderSettings {
                active_mode: ProviderMode::Remote,
                available_modes: vec![ProviderMode::Remote],
                summary: String::from("Planner currently uses configured remote profiles only."),
            },
            remote_planner_settings: RemotePlannerSettings {
                profile_name: Some(String::from("openai-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-5.4-mini")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                temperature_milli: Some(200),
                max_output_tokens: Some(1024),
                timeout_ms: Some(30_000),
            },
            remote_tts_settings: RemoteTtsSettings {
                profile_name: Some(String::from("openai-tts-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-4o-mini-tts")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                voice: Some(String::from("alloy")),
                audio_format: Some(String::from("wav")),
                timeout_ms: Some(30_000),
            },
            remote_asr_settings: RemoteAsrSettings {
                profile_name: Some(String::from("openai-transcribe-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-4o-mini-transcribe")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                language: Some(String::from("en")),
                temperature_milli: Some(0),
                timeout_ms: Some(30_000),
            },
            provider_failover_settings: ProviderFailoverSettings {
                planner_available: false,
                tts_available: false,
                asr_available: false,
                summary: String::from(
                    "Automatic provider failover is not currently available in the live runtime.",
                ),
            },
            confirmation_settings: ConfirmationSettings {
                confirmation_confidence_threshold: 0.9,
                allow_click_without_confirmation: true,
                always_confirm_submit: true,
            },
            ocr_threshold_settings: OcrThresholdSettings {
                sparse_text_char_threshold: 200,
                sparse_text_region_threshold: 2,
            },
        };
        let runtime_status = GetRuntimeStatusData {
            page_id: agent_state.page_id.clone(),
            url: agent_state.url.clone(),
            title: agent_state.title.clone(),
            browser_visibility: agent_state.browser_visibility,
            browser_history: agent_state.browser_history.clone(),
            listening_state: agent_state.listening_state.clone(),
            speaking: agent_state.speaking,
            audio: agent_state.audio.clone(),
            pending_confirmation_id: None,
            pending_plan_execution: None,
            provider_modes: None,
        };

        let planner_output = resolve_direct_status_query_command(
            "can i go back",
            "req-back-status",
            &agent_state,
            &runtime_status,
            &[String::from("get_status")],
        )
        .expect("back history query should normalize");

        assert_eq!(planner_output.intent.name, IntentName::GetStatus);
        assert_eq!(
            planner_output.steps[0].tool_name,
            ToolName::GetRuntimeStatus
        );
        assert_eq!(
            planner_output.steps[1].arguments.get("summary"),
            Some(&serde_json::json!("Back navigation is available."))
        );
    }

    #[test]
    fn resolve_direct_status_query_command_reports_listening_state() {
        let agent_state = AgentStateData {
            page_id: None,
            url: None,
            title: None,
            browser_visibility: BrowserVisibilityMode::Headless,
            browser_history: BrowserHistoryState::default(),
            narration_cursor: Some(NarrationCursor::default()),
            speaking: false,
            listening_state: ListeningState {
                is_listening: true,
                push_to_talk_enabled: true,
            },
            audio: RuntimeAudioState::default(),
            last_transcript: None,
            last_tool_call: None,
            pending_confirmation_id: None,
            pending_plan_execution: None,
            tts_model_settings: TtsModelSettings {
                mode: ProviderMode::Local,
                active_profile: Some(String::from("kitten-default")),
                available_profiles: vec![TtsModelOption {
                    profile_name: String::from("kitten-default"),
                    model_label: String::from("default"),
                }],
            },
            local_tts_model_settings: LocalTtsModelSettings {
                profile_name: Some(String::from("kitten-default")),
                backend: Some(String::from("kitten_tts_rs")),
                model_id: Some(String::from("default")),
                model_path: Some(String::from("/path/to/kitten/model")),
                default_voice: Some(String::from("Bruno")),
                sample_rate: Some(24_000),
            },
            tts_voice_settings: TtsVoiceSettings {
                mode: ProviderMode::Local,
                active_voice: Some(String::from("Bruno")),
                available_voices: vec![
                    TtsVoiceOption {
                        voice_name: String::from("Bella"),
                        display_label: String::from("Bella"),
                    },
                    TtsVoiceOption {
                        voice_name: String::from("Bruno"),
                        display_label: String::from("Bruno"),
                    },
                ],
            },
            tts_provider_settings: TtsProviderSettings {
                active_mode: ProviderMode::Local,
                available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
            },
            asr_provider_settings: AsrProviderSettings {
                active_mode: ProviderMode::Local,
                available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
            },
            local_asr_model_settings: LocalAsrModelSettings {
                profile_name: Some(String::from("whisper-default")),
                backend: Some(String::from("whisper")),
                model_id: Some(String::from("tiny")),
                model_path: Some(String::from("/path/to/whisper/model")),
                language: Some(String::from("en")),
                threads: Some(4),
            },
            planner_provider_settings: PlannerProviderSettings {
                active_mode: ProviderMode::Remote,
                available_modes: vec![ProviderMode::Remote],
                summary: String::from("Planner currently uses configured remote profiles only."),
            },
            remote_planner_settings: RemotePlannerSettings {
                profile_name: Some(String::from("openai-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-5.4-mini")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                temperature_milli: Some(200),
                max_output_tokens: Some(1024),
                timeout_ms: Some(30_000),
            },
            remote_tts_settings: RemoteTtsSettings {
                profile_name: Some(String::from("openai-tts-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-4o-mini-tts")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                voice: Some(String::from("alloy")),
                audio_format: Some(String::from("wav")),
                timeout_ms: Some(30_000),
            },
            remote_asr_settings: RemoteAsrSettings {
                profile_name: Some(String::from("openai-transcribe-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-4o-mini-transcribe")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                language: Some(String::from("en")),
                temperature_milli: Some(0),
                timeout_ms: Some(30_000),
            },
            provider_failover_settings: ProviderFailoverSettings {
                planner_available: false,
                tts_available: false,
                asr_available: false,
                summary: String::from(
                    "Automatic provider failover is not currently available in the live runtime.",
                ),
            },
            confirmation_settings: ConfirmationSettings {
                confirmation_confidence_threshold: 0.9,
                allow_click_without_confirmation: true,
                always_confirm_submit: true,
            },
            ocr_threshold_settings: OcrThresholdSettings {
                sparse_text_char_threshold: 200,
                sparse_text_region_threshold: 2,
            },
        };
        let runtime_status = GetRuntimeStatusData {
            page_id: None,
            url: None,
            title: None,
            browser_visibility: BrowserVisibilityMode::Headless,
            browser_history: BrowserHistoryState::default(),
            listening_state: agent_state.listening_state.clone(),
            speaking: false,
            audio: agent_state.audio.clone(),
            pending_confirmation_id: None,
            pending_plan_execution: None,
            provider_modes: None,
        };

        let planner_output = resolve_direct_status_query_command(
            "are you listening",
            "req-listening-status",
            &agent_state,
            &runtime_status,
            &[String::from("get_status")],
        )
        .expect("listening query should normalize");

        assert_eq!(planner_output.intent.name, IntentName::GetStatus);
        assert_eq!(
            planner_output.steps[1].arguments.get("summary"),
            Some(&serde_json::json!("Listening is on."))
        );

        let fuzzy_planner_output = resolve_direct_status_query_command(
            "are you listenin",
            "req-listening-status-fuzzy",
            &agent_state,
            &runtime_status,
            &[String::from("get_status")],
        )
        .expect("fuzzy listening query should normalize");

        assert_eq!(
            fuzzy_planner_output.steps[1].arguments.get("summary"),
            Some(&serde_json::json!("Listening is on."))
        );
    }

    #[test]
    fn resolve_direct_repeat_command_replays_current_region() {
        let agent_state = AgentStateData {
            page_id: Some(String::from("page-1")),
            url: Some(String::from("https://example.com/article")),
            title: Some(String::from("Example article")),
            browser_visibility: BrowserVisibilityMode::Visible,
            browser_history: BrowserHistoryState::default(),
            narration_cursor: Some(NarrationCursor {
                current_region_id: Some(String::from("region-2")),
                current_index: Some(1),
                total_regions: 3,
            }),
            speaking: false,
            listening_state: ListeningState::default(),
            audio: RuntimeAudioState::default(),
            last_transcript: None,
            last_tool_call: None,
            pending_confirmation_id: None,
            pending_plan_execution: None,
            tts_model_settings: TtsModelSettings {
                mode: ProviderMode::Local,
                active_profile: Some(String::from("kitten-default")),
                available_profiles: vec![TtsModelOption {
                    profile_name: String::from("kitten-default"),
                    model_label: String::from("default"),
                }],
            },
            local_tts_model_settings: LocalTtsModelSettings {
                profile_name: Some(String::from("kitten-default")),
                backend: Some(String::from("kitten_tts_rs")),
                model_id: Some(String::from("default")),
                model_path: Some(String::from("/path/to/kitten/model")),
                default_voice: Some(String::from("Bruno")),
                sample_rate: Some(24_000),
            },
            tts_voice_settings: TtsVoiceSettings {
                mode: ProviderMode::Local,
                active_voice: Some(String::from("Bruno")),
                available_voices: vec![
                    TtsVoiceOption {
                        voice_name: String::from("Bella"),
                        display_label: String::from("Bella"),
                    },
                    TtsVoiceOption {
                        voice_name: String::from("Bruno"),
                        display_label: String::from("Bruno"),
                    },
                ],
            },
            tts_provider_settings: TtsProviderSettings {
                active_mode: ProviderMode::Local,
                available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
            },
            asr_provider_settings: AsrProviderSettings {
                active_mode: ProviderMode::Local,
                available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
            },
            local_asr_model_settings: LocalAsrModelSettings {
                profile_name: Some(String::from("whisper-default")),
                backend: Some(String::from("whisper")),
                model_id: Some(String::from("tiny")),
                model_path: Some(String::from("/path/to/whisper/model")),
                language: Some(String::from("en")),
                threads: Some(4),
            },
            planner_provider_settings: PlannerProviderSettings {
                active_mode: ProviderMode::Remote,
                available_modes: vec![ProviderMode::Remote],
                summary: String::from("Planner currently uses configured remote profiles only."),
            },
            remote_planner_settings: RemotePlannerSettings {
                profile_name: Some(String::from("openai-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-5.4-mini")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                temperature_milli: Some(200),
                max_output_tokens: Some(1024),
                timeout_ms: Some(30_000),
            },
            remote_tts_settings: RemoteTtsSettings {
                profile_name: Some(String::from("openai-tts-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-4o-mini-tts")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                voice: Some(String::from("alloy")),
                audio_format: Some(String::from("wav")),
                timeout_ms: Some(30_000),
            },
            remote_asr_settings: RemoteAsrSettings {
                profile_name: Some(String::from("openai-transcribe-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-4o-mini-transcribe")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                language: Some(String::from("en")),
                temperature_milli: Some(0),
                timeout_ms: Some(30_000),
            },
            provider_failover_settings: ProviderFailoverSettings {
                planner_available: false,
                tts_available: false,
                asr_available: false,
                summary: String::from(
                    "Automatic provider failover is not currently available in the live runtime.",
                ),
            },
            confirmation_settings: ConfirmationSettings {
                confirmation_confidence_threshold: 0.9,
                allow_click_without_confirmation: true,
                always_confirm_submit: true,
            },
            ocr_threshold_settings: OcrThresholdSettings {
                sparse_text_char_threshold: 200,
                sparse_text_region_threshold: 2,
            },
        };

        let planner_output = resolve_direct_repeat_command(
            "say that again",
            "req-repeat",
            &agent_state,
            &[String::from("repeat")],
        )
        .expect("repeat command should normalize");

        assert_eq!(planner_output.intent.name, IntentName::Repeat);
        assert_eq!(planner_output.selected_skills, vec![String::from("repeat")]);
        assert_eq!(planner_output.steps.len(), 1);
        assert_eq!(planner_output.steps[0].tool_name, ToolName::ReadRegion);
        assert_eq!(
            planner_output.steps[0].arguments.get("region_id"),
            Some(&serde_json::json!("region-2"))
        );
        assert_eq!(
            planner_output.steps[0].arguments.get("interrupt_current"),
            Some(&serde_json::json!(true))
        );
    }

    #[test]
    fn resolve_direct_repeat_command_reports_missing_current_region() {
        let agent_state = AgentStateData {
            page_id: Some(String::from("page-1")),
            url: Some(String::from("https://example.com/article")),
            title: Some(String::from("Example article")),
            browser_visibility: BrowserVisibilityMode::Visible,
            browser_history: BrowserHistoryState::default(),
            narration_cursor: Some(NarrationCursor::default()),
            speaking: false,
            listening_state: ListeningState::default(),
            audio: RuntimeAudioState::default(),
            last_transcript: None,
            last_tool_call: None,
            pending_confirmation_id: None,
            pending_plan_execution: None,
            tts_model_settings: TtsModelSettings {
                mode: ProviderMode::Local,
                active_profile: Some(String::from("kitten-default")),
                available_profiles: vec![TtsModelOption {
                    profile_name: String::from("kitten-default"),
                    model_label: String::from("default"),
                }],
            },
            local_tts_model_settings: LocalTtsModelSettings {
                profile_name: Some(String::from("kitten-default")),
                backend: Some(String::from("kitten_tts_rs")),
                model_id: Some(String::from("default")),
                model_path: Some(String::from("/path/to/kitten/model")),
                default_voice: Some(String::from("Bruno")),
                sample_rate: Some(24_000),
            },
            tts_voice_settings: TtsVoiceSettings {
                mode: ProviderMode::Local,
                active_voice: Some(String::from("Bruno")),
                available_voices: vec![
                    TtsVoiceOption {
                        voice_name: String::from("Bella"),
                        display_label: String::from("Bella"),
                    },
                    TtsVoiceOption {
                        voice_name: String::from("Bruno"),
                        display_label: String::from("Bruno"),
                    },
                ],
            },
            tts_provider_settings: TtsProviderSettings {
                active_mode: ProviderMode::Local,
                available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
            },
            asr_provider_settings: AsrProviderSettings {
                active_mode: ProviderMode::Local,
                available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
            },
            local_asr_model_settings: LocalAsrModelSettings {
                profile_name: Some(String::from("whisper-default")),
                backend: Some(String::from("whisper")),
                model_id: Some(String::from("tiny")),
                model_path: Some(String::from("/path/to/whisper/model")),
                language: Some(String::from("en")),
                threads: Some(4),
            },
            planner_provider_settings: PlannerProviderSettings {
                active_mode: ProviderMode::Remote,
                available_modes: vec![ProviderMode::Remote],
                summary: String::from("Planner currently uses configured remote profiles only."),
            },
            remote_planner_settings: RemotePlannerSettings {
                profile_name: Some(String::from("openai-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-5.4-mini")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                temperature_milli: Some(200),
                max_output_tokens: Some(1024),
                timeout_ms: Some(30_000),
            },
            remote_tts_settings: RemoteTtsSettings {
                profile_name: Some(String::from("openai-tts-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-4o-mini-tts")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                voice: Some(String::from("alloy")),
                audio_format: Some(String::from("wav")),
                timeout_ms: Some(30_000),
            },
            remote_asr_settings: RemoteAsrSettings {
                profile_name: Some(String::from("openai-transcribe-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-4o-mini-transcribe")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                language: Some(String::from("en")),
                temperature_milli: Some(0),
                timeout_ms: Some(30_000),
            },
            provider_failover_settings: ProviderFailoverSettings {
                planner_available: false,
                tts_available: false,
                asr_available: false,
                summary: String::from(
                    "Automatic provider failover is not currently available in the live runtime.",
                ),
            },
            confirmation_settings: ConfirmationSettings {
                confirmation_confidence_threshold: 0.9,
                allow_click_without_confirmation: true,
                always_confirm_submit: true,
            },
            ocr_threshold_settings: OcrThresholdSettings {
                sparse_text_char_threshold: 200,
                sparse_text_region_threshold: 2,
            },
        };

        let planner_output =
            resolve_direct_repeat_command("repeat that", "req-repeat-missing", &agent_state, &[])
                .expect("repeat command should still produce a bounded response");

        assert_eq!(planner_output.intent.name, IntentName::Repeat);
        assert_eq!(planner_output.steps.len(), 1);
        assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
        assert_eq!(
            planner_output.steps[0].arguments.get("status"),
            Some(&serde_json::json!(ReportStatus::NeedsFollowUp))
        );
        assert_eq!(
            planner_output.steps[0].arguments.get("summary"),
            Some(&serde_json::json!(
                "There is no current region to repeat yet."
            ))
        );
        assert_eq!(
            planner_output.steps[0]
                .arguments
                .get("next_recommended_action"),
            Some(&serde_json::json!(
                "Read the page or move to a region first."
            ))
        );
    }

    #[test]
    fn resolve_direct_read_title_command_reports_current_title() {
        let agent_state = AgentStateData {
            page_id: Some(String::from("page-1")),
            url: Some(String::from("https://example.com/article")),
            title: Some(String::from("Example article")),
            browser_visibility: BrowserVisibilityMode::Visible,
            browser_history: BrowserHistoryState::default(),
            narration_cursor: Some(NarrationCursor::default()),
            speaking: false,
            listening_state: ListeningState::default(),
            audio: RuntimeAudioState::default(),
            last_transcript: None,
            last_tool_call: None,
            pending_confirmation_id: None,
            pending_plan_execution: None,
            tts_model_settings: TtsModelSettings {
                mode: ProviderMode::Local,
                active_profile: Some(String::from("kitten-default")),
                available_profiles: vec![TtsModelOption {
                    profile_name: String::from("kitten-default"),
                    model_label: String::from("default"),
                }],
            },
            local_tts_model_settings: LocalTtsModelSettings {
                profile_name: Some(String::from("kitten-default")),
                backend: Some(String::from("kitten_tts_rs")),
                model_id: Some(String::from("default")),
                model_path: Some(String::from("/path/to/kitten/model")),
                default_voice: Some(String::from("Bruno")),
                sample_rate: Some(24_000),
            },
            tts_voice_settings: TtsVoiceSettings {
                mode: ProviderMode::Local,
                active_voice: Some(String::from("Bruno")),
                available_voices: vec![
                    TtsVoiceOption {
                        voice_name: String::from("Bella"),
                        display_label: String::from("Bella"),
                    },
                    TtsVoiceOption {
                        voice_name: String::from("Bruno"),
                        display_label: String::from("Bruno"),
                    },
                ],
            },
            tts_provider_settings: TtsProviderSettings {
                active_mode: ProviderMode::Local,
                available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
            },
            asr_provider_settings: AsrProviderSettings {
                active_mode: ProviderMode::Local,
                available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
            },
            local_asr_model_settings: LocalAsrModelSettings {
                profile_name: Some(String::from("whisper-default")),
                backend: Some(String::from("whisper")),
                model_id: Some(String::from("tiny")),
                model_path: Some(String::from("/path/to/whisper/model")),
                language: Some(String::from("en")),
                threads: Some(4),
            },
            planner_provider_settings: PlannerProviderSettings {
                active_mode: ProviderMode::Remote,
                available_modes: vec![ProviderMode::Remote],
                summary: String::from("Planner currently uses configured remote profiles only."),
            },
            remote_planner_settings: RemotePlannerSettings {
                profile_name: Some(String::from("openai-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-5.4-mini")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                temperature_milli: Some(200),
                max_output_tokens: Some(1024),
                timeout_ms: Some(30_000),
            },
            remote_tts_settings: RemoteTtsSettings {
                profile_name: Some(String::from("openai-tts-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-4o-mini-tts")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                voice: Some(String::from("alloy")),
                audio_format: Some(String::from("wav")),
                timeout_ms: Some(30_000),
            },
            remote_asr_settings: RemoteAsrSettings {
                profile_name: Some(String::from("openai-transcribe-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-4o-mini-transcribe")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                language: Some(String::from("en")),
                temperature_milli: Some(0),
                timeout_ms: Some(30_000),
            },
            provider_failover_settings: ProviderFailoverSettings {
                planner_available: false,
                tts_available: false,
                asr_available: false,
                summary: String::from(
                    "Automatic provider failover is not currently available in the live runtime.",
                ),
            },
            confirmation_settings: ConfirmationSettings {
                confirmation_confidence_threshold: 0.9,
                allow_click_without_confirmation: true,
                always_confirm_submit: true,
            },
            ocr_threshold_settings: OcrThresholdSettings {
                sparse_text_char_threshold: 200,
                sparse_text_region_threshold: 2,
            },
        };

        let planner_output = resolve_direct_read_title_command(
            "read the page title",
            "req-read-title",
            &agent_state,
            &[String::from("read_title")],
        )
        .expect("read title command should normalize");

        assert_eq!(planner_output.intent.name, IntentName::ReadTitle);
        assert_eq!(
            planner_output.selected_skills,
            vec![String::from("read_title")]
        );
        assert_eq!(planner_output.steps.len(), 1);
        assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
        assert_eq!(
            planner_output.steps[0].arguments.get("summary"),
            Some(&serde_json::json!("Page title is Example article."))
        );
    }

    #[test]
    fn resolve_direct_read_title_command_reports_missing_title() {
        let agent_state = AgentStateData {
            page_id: Some(String::from("page-1")),
            url: Some(String::from("https://example.com/article")),
            title: None,
            browser_visibility: BrowserVisibilityMode::Visible,
            browser_history: BrowserHistoryState::default(),
            narration_cursor: Some(NarrationCursor::default()),
            speaking: false,
            listening_state: ListeningState::default(),
            audio: RuntimeAudioState::default(),
            last_transcript: None,
            last_tool_call: None,
            pending_confirmation_id: None,
            pending_plan_execution: None,
            tts_model_settings: TtsModelSettings {
                mode: ProviderMode::Local,
                active_profile: Some(String::from("kitten-default")),
                available_profiles: vec![TtsModelOption {
                    profile_name: String::from("kitten-default"),
                    model_label: String::from("default"),
                }],
            },
            local_tts_model_settings: LocalTtsModelSettings {
                profile_name: Some(String::from("kitten-default")),
                backend: Some(String::from("kitten_tts_rs")),
                model_id: Some(String::from("default")),
                model_path: Some(String::from("/path/to/kitten/model")),
                default_voice: Some(String::from("Bruno")),
                sample_rate: Some(24_000),
            },
            tts_voice_settings: TtsVoiceSettings {
                mode: ProviderMode::Local,
                active_voice: Some(String::from("Bruno")),
                available_voices: vec![
                    TtsVoiceOption {
                        voice_name: String::from("Bella"),
                        display_label: String::from("Bella"),
                    },
                    TtsVoiceOption {
                        voice_name: String::from("Bruno"),
                        display_label: String::from("Bruno"),
                    },
                ],
            },
            tts_provider_settings: TtsProviderSettings {
                active_mode: ProviderMode::Local,
                available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
            },
            asr_provider_settings: AsrProviderSettings {
                active_mode: ProviderMode::Local,
                available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
            },
            local_asr_model_settings: LocalAsrModelSettings {
                profile_name: Some(String::from("whisper-default")),
                backend: Some(String::from("whisper")),
                model_id: Some(String::from("tiny")),
                model_path: Some(String::from("/path/to/whisper/model")),
                language: Some(String::from("en")),
                threads: Some(4),
            },
            planner_provider_settings: PlannerProviderSettings {
                active_mode: ProviderMode::Remote,
                available_modes: vec![ProviderMode::Remote],
                summary: String::from("Planner currently uses configured remote profiles only."),
            },
            remote_planner_settings: RemotePlannerSettings {
                profile_name: Some(String::from("openai-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-5.4-mini")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                temperature_milli: Some(200),
                max_output_tokens: Some(1024),
                timeout_ms: Some(30_000),
            },
            remote_tts_settings: RemoteTtsSettings {
                profile_name: Some(String::from("openai-tts-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-4o-mini-tts")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                voice: Some(String::from("alloy")),
                audio_format: Some(String::from("wav")),
                timeout_ms: Some(30_000),
            },
            remote_asr_settings: RemoteAsrSettings {
                profile_name: Some(String::from("openai-transcribe-default")),
                provider: Some(String::from("OpenAI")),
                base_url: Some(String::from("https://api.openai.com/v1")),
                model: Some(String::from("gpt-4o-mini-transcribe")),
                api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
                organization_reference: None,
                project: None,
                language: Some(String::from("en")),
                temperature_milli: Some(0),
                timeout_ms: Some(30_000),
            },
            provider_failover_settings: ProviderFailoverSettings {
                planner_available: false,
                tts_available: false,
                asr_available: false,
                summary: String::from(
                    "Automatic provider failover is not currently available in the live runtime.",
                ),
            },
            confirmation_settings: ConfirmationSettings {
                confirmation_confidence_threshold: 0.9,
                allow_click_without_confirmation: true,
                always_confirm_submit: true,
            },
            ocr_threshold_settings: OcrThresholdSettings {
                sparse_text_char_threshold: 200,
                sparse_text_region_threshold: 2,
            },
        };

        let planner_output = resolve_direct_read_title_command(
            "what is the title",
            "req-read-title-missing",
            &agent_state,
            &[],
        )
        .expect("missing-title command should still produce a bounded response");

        assert_eq!(planner_output.intent.name, IntentName::ReadTitle);
        assert_eq!(planner_output.steps.len(), 1);
        assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
        assert_eq!(
            planner_output.steps[0].arguments.get("summary"),
            Some(&serde_json::json!(
                "This page does not have a readable title yet."
            ))
        );
    }
    fn assert_json_matches_schema(
        instance: &serde_json::Value,
        schema: &serde_json::Value,
    ) -> Result<(), String> {
        assert_json_matches_schema_at(instance, schema, schema, "$")
    }

    fn assert_json_matches_schema_at(
        instance: &serde_json::Value,
        schema: &serde_json::Value,
        root_schema: &serde_json::Value,
        path: &str,
    ) -> Result<(), String> {
        let schema = resolve_schema_reference(schema, root_schema)?;

        if let Some(all_of) = schema.get("allOf").and_then(serde_json::Value::as_array) {
            for subschema in all_of {
                assert_json_matches_schema_at(instance, subschema, root_schema, path)?;
            }
        }

        if let Some(any_of) = schema.get("anyOf").and_then(serde_json::Value::as_array) {
            let mut errors = Vec::new();
            for subschema in any_of {
                match assert_json_matches_schema_at(instance, subschema, root_schema, path) {
                    Ok(()) => {
                        errors.clear();
                        break;
                    }
                    Err(error) => errors.push(error),
                }
            }
            if !errors.is_empty() {
                return Err(format!(
                    "{path}: value did not satisfy anyOf alternatives: {}",
                    errors.join(" | ")
                ));
            }
        }

        if let Some(one_of) = schema.get("oneOf").and_then(serde_json::Value::as_array) {
            let mut match_count = 0;
            let mut errors = Vec::new();
            for subschema in one_of {
                match assert_json_matches_schema_at(instance, subschema, root_schema, path) {
                    Ok(()) => match_count += 1,
                    Err(error) => errors.push(error),
                }
            }
            if match_count != 1 {
                return Err(format!(
                    "{path}: value matched {match_count} oneOf alternatives (expected exactly 1): {}",
                    errors.join(" | ")
                ));
            }
        }

        if let Some(expected_const) = schema.get("const") {
            if instance != expected_const {
                return Err(format!(
                    "{path}: expected const {expected_const:?}, got {instance:?}"
                ));
            }
        }

        if let Some(enum_values) = schema.get("enum").and_then(serde_json::Value::as_array) {
            if !enum_values.iter().any(|candidate| candidate == instance) {
                return Err(format!(
                    "{path}: expected one of {enum_values:?}, got {instance:?}"
                ));
            }
        }

        if let Some(type_schema) = schema.get("type") {
            if !json_matches_type(instance, type_schema) {
                return Err(format!(
                    "{path}: value {instance:?} did not match schema type {type_schema:?}"
                ));
            }
        }

        if let Some(required) = schema.get("required").and_then(serde_json::Value::as_array) {
            let Some(object) = instance.as_object() else {
                return Err(format!("{path}: required fields only apply to objects"));
            };
            for field_name in required.iter().filter_map(serde_json::Value::as_str) {
                if !object.contains_key(field_name) {
                    return Err(format!("{path}: missing required field '{field_name}'"));
                }
            }
        }

        if let Some(properties) = schema
            .get("properties")
            .and_then(serde_json::Value::as_object)
        {
            let Some(object) = instance.as_object() else {
                return Err(format!("{path}: properties only apply to objects"));
            };
            let additional_properties_allowed = schema
                .get("additionalProperties")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true);

            for (key, value) in object {
                if let Some(property_schema) = properties.get(key) {
                    assert_json_matches_schema_at(
                        value,
                        property_schema,
                        root_schema,
                        &format!("{path}/{key}"),
                    )?;
                } else if !additional_properties_allowed {
                    return Err(format!(
                        "{path}: unexpected property '{key}' is not allowed by the schema"
                    ));
                }
            }
        }

        if let Some(items_schema) = schema.get("items") {
            let Some(array) = instance.as_array() else {
                return Err(format!("{path}: items only apply to arrays"));
            };
            for (index, item) in array.iter().enumerate() {
                assert_json_matches_schema_at(
                    item,
                    items_schema,
                    root_schema,
                    &format!("{path}/{index}"),
                )?;
            }
        }

        Ok(())
    }

    fn resolve_schema_reference<'a>(
        schema: &'a serde_json::Value,
        root_schema: &'a serde_json::Value,
    ) -> Result<&'a serde_json::Value, String> {
        let Some(reference) = schema.get("$ref").and_then(serde_json::Value::as_str) else {
            return Ok(schema);
        };
        let pointer = reference
            .strip_prefix('#')
            .ok_or_else(|| format!("unsupported non-local schema reference '{reference}'"))?;
        let resolved = root_schema
            .pointer(pointer)
            .ok_or_else(|| format!("failed to resolve schema reference '{reference}'"))?;
        if std::ptr::eq(resolved, schema) {
            return Ok(resolved);
        }
        resolve_schema_reference(resolved, root_schema)
    }

    fn json_matches_type(instance: &serde_json::Value, type_schema: &serde_json::Value) -> bool {
        match type_schema {
            serde_json::Value::String(kind) => json_matches_single_type(instance, kind),
            serde_json::Value::Array(kinds) => kinds.iter().any(|kind| {
                kind.as_str()
                    .is_some_and(|kind| json_matches_single_type(instance, kind))
            }),
            _ => true,
        }
    }

    fn json_matches_single_type(instance: &serde_json::Value, kind: &str) -> bool {
        match kind {
            "null" => instance.is_null(),
            "boolean" => instance.is_boolean(),
            "object" => instance.is_object(),
            "array" => instance.is_array(),
            "string" => instance.is_string(),
            "number" => instance.is_number(),
            "integer" => instance
                .as_f64()
                .is_some_and(|number| number.fract().abs() < f64::EPSILON),
            _ => false,
        }
    }
}
