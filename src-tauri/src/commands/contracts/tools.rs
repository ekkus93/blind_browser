use super::*;

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
