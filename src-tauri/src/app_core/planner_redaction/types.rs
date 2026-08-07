//! Data model sent to a remote planner after sanitization: every field here is
//! the redacted, size-bounded shape produced by [`super::sanitize`] — never the
//! raw [`crate::commands::PlannerInput`].

use serde::Serialize;

use crate::audio_io::RuntimeAudioState;
use crate::browser::BrowserVisibilityMode;
use crate::commands::{AvailableTool, PlannerSafetySettings, ToolName};
use crate::narration::NarrationCursor;
use crate::page_model::{ElementRole, Rect, RegionRole, RegionSource};
use crate::state::{BrowserHistoryState, ListeningState};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(transparent)]
pub(crate) struct UntrustedText(pub(super) String);

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(transparent)]
pub(crate) struct PlannerSafeUrl(pub(super) String);

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct RemotePlannerInput {
    pub(crate) trust_boundary_version: String,
    pub(crate) trusted_runtime: RemoteTrustedRuntime,
    pub(crate) user_request: RemoteUserRequest,
    pub(crate) untrusted_data: RemoteUntrustedData,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct RemoteTrustedRuntime {
    pub(crate) request_id: String,
    pub(crate) safety: PlannerSafetySettings,
    pub(crate) available_tools: Vec<AvailableTool>,
    pub(crate) active_skill_names: Vec<String>,
    pub(crate) remote_data_mode: RemoteDataMode,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub(crate) enum RemoteDataMode {
    LoopbackLocalService,
    NetworkRemoteWithExplicitConsent,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct RemoteUserRequest {
    pub(crate) transcript: UntrustedText,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct RemoteUntrustedData {
    pub(crate) agent_state: RemoteAgentState,
    pub(crate) page_snapshot: Option<RemotePageSnapshot>,
    pub(crate) page_model: Option<RemotePageModel>,
    pub(crate) recent_tool_results: Vec<RemoteToolObservation>,
    pub(crate) relevant_skill_summaries: Vec<RemoteSkillSummary>,
    pub(crate) prompt_injection_indicators: PromptInjectionIndicators,
    pub(crate) sanitization: SanitizationMetadata,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct RemoteAgentState {
    pub(crate) page_id: Option<String>,
    pub(crate) url: Option<PlannerSafeUrl>,
    pub(crate) title: Option<UntrustedText>,
    pub(crate) browser_visibility: BrowserVisibilityMode,
    pub(crate) browser_history: BrowserHistoryState,
    pub(crate) narration_cursor: Option<NarrationCursor>,
    pub(crate) speaking: bool,
    pub(crate) listening_state: ListeningState,
    pub(crate) audio: RuntimeAudioState,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct RemotePageSnapshot {
    pub(crate) page_id: String,
    pub(crate) url: PlannerSafeUrl,
    pub(crate) title: Option<UntrustedText>,
    pub(crate) visible_text_excerpt: UntrustedText,
    pub(crate) interactive_elements: Vec<RemoteInteractiveElement>,
    pub(crate) scroll_y: f32,
    pub(crate) viewport_width: f32,
    pub(crate) viewport_height: f32,
    pub(crate) document_height: f32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct RemotePageModel {
    pub(crate) title: Option<UntrustedText>,
    pub(crate) url: Option<PlannerSafeUrl>,
    pub(crate) regions: Vec<RemotePageRegion>,
    pub(crate) interactive_elements: Vec<RemoteInteractiveElement>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct RemotePageRegion {
    pub(crate) region_id: String,
    pub(crate) role: RegionRole,
    pub(crate) label: Option<UntrustedText>,
    pub(crate) text: UntrustedText,
    pub(crate) bbox: Option<Rect>,
    pub(crate) source: RegionSource,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct RemoteInteractiveElement {
    pub(crate) element_id: String,
    pub(crate) role: ElementRole,
    pub(crate) tag_name: String,
    pub(crate) text: Option<UntrustedText>,
    pub(crate) accessible_name: Option<UntrustedText>,
    pub(crate) placeholder: Option<UntrustedText>,
    pub(crate) href: Option<PlannerSafeUrl>,
    pub(crate) bbox: Option<Rect>,
    pub(crate) visible: bool,
    pub(crate) enabled: bool,
    pub(crate) sensitive: bool,
    pub(crate) safe_attributes: PlannerSafeElementAttributes,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub(crate) struct PlannerSafeElementAttributes {
    pub(crate) input_type: Option<String>,
    pub(crate) checked: Option<bool>,
    pub(crate) selected: Option<bool>,
    pub(crate) disabled: Option<bool>,
    pub(crate) autocomplete: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct RemoteToolObservation {
    pub(crate) tool_name: ToolName,
    pub(crate) ok: bool,
    pub(crate) observation_summary: Vec<UntrustedText>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct RemoteSkillSummary {
    pub(crate) name: String,
    pub(crate) description: UntrustedText,
    pub(crate) intent_tags: Vec<String>,
    pub(crate) allowed_tools: Option<Vec<ToolName>>,
    pub(crate) requires_confirmation: bool,
    pub(crate) priority: i32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct PromptInjectionIndicators {
    pub(crate) caution_only: bool,
    pub(crate) detected: bool,
    pub(crate) reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub(crate) struct SanitizationMetadata {
    pub(crate) redacted_text_fields: usize,
    pub(crate) truncated_text_fields: usize,
    pub(crate) omitted_elements: usize,
    pub(crate) omitted_hidden_elements: usize,
    pub(crate) relevance_filtered_elements: usize,
    pub(crate) omitted_regions: usize,
    pub(crate) relevance_filtered_regions: usize,
    pub(crate) omitted_history_entries: usize,
    pub(crate) omitted_skill_summaries: usize,
    pub(crate) query_values_removed: usize,
}
