use super::*;

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

#[derive(Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct PlannerInput {
    pub request_id: String,
    #[serde(default)]
    pub runtime_state_token: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RemotePlannerDisclosureClass {
    UserTranscript,
    PageOrigin,
    SelectedPageRegions,
    SelectedElementMetadata,
    OcrDerivedRegions,
    ToolObservationSummaries,
    SkillSummaries,
    TrustedRuntimeContracts,
    // Narration (remote TTS) and microphone-audio (remote ASR) disclosure
    // kinds share this same challenge/manifest shape (see
    // app_core::remote_data_consent) but disclose page text read aloud, or
    // raw microphone audio, respectively -- neither the redacted page
    // context above nor the user's typed command.
    NarrationText,
    MicrophoneAudio,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RemotePlannerDisclosureCounts {
    pub selected_region_count: usize,
    pub selected_element_count: usize,
    pub ocr_derived_region_count: usize,
    pub tool_history_count: usize,
    pub skill_summary_count: usize,
    pub sanitized_serialized_bytes: usize,
    #[serde(default)]
    pub narration_text_bytes: usize,
    #[serde(default)]
    pub microphone_audio_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RemotePlannerConsentChallenge {
    pub challenge_id: String,
    pub challenge_digest: String,
    pub request_id: String,
    pub page_origin: String,
    pub endpoint_display: String,
    pub endpoint_scope: String,
    pub profile_name: String,
    pub model_label: String,
    pub policy_version: u32,
    pub disclosure_classes: Vec<RemotePlannerDisclosureClass>,
    pub disclosure_counts: RemotePlannerDisclosureCounts,
    pub expires_at_ms: u64,
    pub allow_once: bool,
    pub allow_session: bool,
    pub allow_persistent: bool,
    pub block_persistent: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RemotePlannerConsentDecision {
    AllowOnce,
    AllowSession,
    AllowPersistent,
    BlockPersistent,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(untagged)]
pub enum ResolveCommandOutcome {
    Resolved(PlannerOutput),
    NeedsRemoteDataConsent {
        needs_remote_data_consent: RemotePlannerConsentChallenge,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum ExecutionOutcome {
    Complete {
        trace: ExecutionTrace,
    },
    AwaitingConfirmation {
        trace: ExecutionTrace,
        pending_confirmation_id: String,
        pending_plan_execution: Box<PendingPlanExecutionState>,
    },
    NeedsReplan {
        trace: ExecutionTrace,
    },
    NeedsRemoteDataConsent {
        trace: ExecutionTrace,
        challenge: Box<RemotePlannerConsentChallenge>,
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
    pub manifest_digest: String,
    pub manifest: ConfirmationManifest,
    pub prompt_text: String,
    #[serde(skip, default)]
    #[schemars(skip)]
    pub runtime_state_token: String,
    pub next_step_id: Option<String>,
    pub queued_step_ids: Vec<String>,
    #[serde(skip_serializing, default)]
    #[schemars(skip)]
    pub queued_steps: Vec<PlannedStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RemotePlannerConsentResponseOutcome {
    Denied,
    BlockedPersistent,
    Resolved { planner_output: PlannerOutput },
    Executed { outcome: Box<ExecutionOutcome> },
}

/// Response outcome for the narration (remote TTS) disclosure kind's consent
/// dialog (see `app_core::remote_data_consent::narration_consent`). Narration
/// has no "resume planner execution" analog to `Resolved`/`Executed` above --
/// once authorized, the paused narration is simply spoken -- so the caller
/// should refresh runtime status afterward to pick up the resulting
/// narration cursor / speaking state, the same way it already does after any
/// other narration tool call.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum NarrationConsentResponseOutcome {
    Denied,
    BlockedPersistent,
    Spoken,
}

/// Remote-ASR consent never retains pre-consent audio. After authorization the
/// user repeats the voice input, so the only successful non-terminal response
/// is an explicit retry request rather than a silently replayed recording.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum MicrophoneConsentResponseOutcome {
    Denied,
    BlockedPersistent,
    AuthorizedRetryRequired,
}
