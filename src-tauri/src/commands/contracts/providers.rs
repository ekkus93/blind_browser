use super::*;
use crate::config::{PersistedOriginDecision, RemotePlannerNetworkMode};

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityAbsenceReason {
    NotConfigured,
    ProfileMissing,
    InvalidEndpoint,
    UnknownModelId,
    ManifestUnavailable,
    FeatureDisabled,
    CredentialReferenceMissing,
    LocalBinaryUnavailable,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RemotePlannerSettings {
    pub profile_name: Option<String>,
    pub provider: Option<RemoteProviderLabel>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub api_key_reference: Option<String>,
    pub api_key_masked_value: Option<String>,
    pub api_key_reference_error: Option<String>,
    pub organization_reference: Option<String>,
    pub project: Option<String>,
    pub temperature_milli: Option<u16>,
    pub max_output_tokens: Option<u32>,
    pub timeout_ms: Option<u64>,
    pub endpoint_is_loopback: Option<bool>,
    pub availability_reason: Option<CapabilityAbsenceReason>,
    pub consent_to_remote_page_data: bool,
    pub local_only: bool,
    pub blocked_origins: Vec<String>,
    pub high_risk_origin_policy: String,
    pub remote_data_notice: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RemotePlannerEffectiveDecision {
    LoopbackLocal,
    LocalOnly,
    HighRiskBlocked,
    OriginBlocked,
    AllowedGlobal,
    AllowedPersistent,
    AllowedSession,
    ConsentRequired,
    OriginUnavailable,
    PlannerUnavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RemotePlannerConsentChallengeSummary {
    pub challenge_id: String,
    pub request_id: String,
    pub page_origin: String,
    pub endpoint_display: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RemotePlannerOriginRuleStatus {
    pub page_origin: String,
    pub decision: PersistedOriginDecision,
    pub endpoint_scope: Option<String>,
    pub endpoint_display: Option<String>,
    pub policy_version: u32,
    pub created_at_ms: u64,
    pub stale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RemotePlannerPrivacyStatus {
    pub network_mode: RemotePlannerNetworkMode,
    pub endpoint_scope: Option<String>,
    pub endpoint_display: Option<String>,
    pub endpoint_is_loopback: Option<bool>,
    pub current_page_origin: Option<String>,
    pub effective_decision: RemotePlannerEffectiveDecision,
    pub reason_code: Option<String>,
    pub persistent_rule: Option<PersistedOriginDecision>,
    pub session_grant_active: bool,
    pub pending_challenge: Option<RemotePlannerConsentChallengeSummary>,
    pub policy_version: u32,
    pub persistent_rule_count: usize,
    pub stale_allow_rule_count: usize,
    pub persistent_rules: Vec<RemotePlannerOriginRuleStatus>,
    pub migration_notice_pending: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum RemotePlannerPrivacyOperation {
    GetStatus,
    SetNetworkMode {
        network_mode: RemotePlannerNetworkMode,
    },
    UpsertOriginRule {
        page_origin: String,
        decision: PersistedOriginDecision,
    },
    UpsertCurrentOriginRule {
        decision: PersistedOriginDecision,
    },
    RevokeOriginRule {
        page_origin: String,
        decision: PersistedOriginDecision,
        endpoint_scope: Option<String>,
    },
    ClearSessionGrants,
    ClearPersistentAllows,
    ClearAllPersistentRules {
        confirmed: bool,
    },
    AcknowledgeMigrationNotice,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RemotePlannerPrivacyOperationResult {
    pub status: RemotePlannerPrivacyStatus,
    pub changed: bool,
    pub network_mode: RemotePlannerNetworkMode,
    pub consent_to_remote_page_data: bool,
    pub local_only: bool,
    pub blocked_origins: Vec<String>,
    pub high_risk_origin_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RemoteTtsSettings {
    pub profile_name: Option<String>,
    pub provider: Option<RemoteProviderLabel>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub api_key_reference: Option<String>,
    pub api_key_masked_value: Option<String>,
    pub api_key_reference_error: Option<String>,
    pub organization_reference: Option<String>,
    pub project: Option<String>,
    pub voice: Option<String>,
    pub audio_format: Option<RemoteTtsAudioFormat>,
    pub timeout_ms: Option<u64>,
    pub endpoint_is_loopback: Option<bool>,
    pub availability_reason: Option<CapabilityAbsenceReason>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RemoteAsrSettings {
    pub profile_name: Option<String>,
    pub provider: Option<RemoteProviderLabel>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub api_key_reference: Option<String>,
    pub api_key_masked_value: Option<String>,
    pub api_key_reference_error: Option<String>,
    pub organization_reference: Option<String>,
    pub project: Option<String>,
    pub language: Option<String>,
    pub temperature_milli: Option<u16>,
    pub timeout_ms: Option<u64>,
    pub endpoint_is_loopback: Option<bool>,
    pub availability_reason: Option<CapabilityAbsenceReason>,
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
    pub skill_discovery_diagnostics: SkillDiscoveryDiagnostics,
}
