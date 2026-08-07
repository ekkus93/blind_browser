use std::collections::BTreeMap;
use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ocr::OcrSettings;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to resolve app config directory: {0}")]
    ResolveAppConfigDir(String),
    #[error("failed to create config directory {path}: {source}")]
    CreateDir {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to read config from {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to write config to {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse config TOML: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("failed to serialize config TOML: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("failed to access the system keyring: {0}")]
    Keyring(String),
    #[error("config validation failed:\n{0}")]
    Validation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderMode {
    Local,
    Remote,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct KeyringRef {
    pub service: String,
    pub account: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(untagged)]
pub enum SecretRef {
    FromEnv { from_env: String },
    FromFile { from_file: String },
    FromKeyring { from_keyring: KeyringRef },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProviderSelection {
    pub mode: ProviderMode,
    pub remote_profile: Option<String>,
    pub local_profile: Option<String>,
    pub failover_to_local: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProviderSelections {
    pub planner: ProviderSelection,
    pub tts: ProviderSelection,
    pub asr: ProviderSelection,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct AudioSettings {
    pub playback_volume: f32,
    pub playback_speed: f32,
    pub default_tts_voice: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SafetySettings {
    pub confirmation_confidence_threshold: f32,
    pub allow_click_without_confirmation: bool,
    pub always_confirm_submit: bool,
}

pub const REMOTE_DATA_POLICY_VERSION: u32 = 1;

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RemotePlannerNetworkMode {
    LocalOnly,
    #[default]
    AskPerOrigin,
    AllowSanitizedNonHighRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum PersistedOriginDecision {
    Allow,
    Block,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RemotePlannerOriginRule {
    pub page_origin: String,
    pub decision: PersistedOriginDecision,
    pub endpoint_scope: Option<String>,
    pub policy_version: u32,
    pub created_at_ms: u64,
}

// CR3 P3.2: high-risk page contexts (credit-card fields, password fields --
// see `high_risk_page_context_reason`/`high_risk_context_reason`) are always
// blocked from remote-planner disclosure, unconditionally, with no config
// knob to loosen that. This struct used to carry a `high_risk_origin_policy`
// field for it, but the field was never read by
// `evaluate_remote_planner_policy` (which blocks on `high_risk_reason` alone)
// and its enum had exactly one variant (`Block`) -- it advertised a control
// that did not exist. Removed rather than wired up: making high-risk
// blocking configurable would be a real, unrequested security-behavior
// change, not a cleanup, and the existing invariant (never overridable) is
// deliberate -- the same "runtime minimum, not a config toggle" pattern
// `always_confirm_submit` follows.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RemotePlannerPrivacySettings {
    // Legacy fields. `normalize_remote_planner_privacy_settings` migrates
    // them into `network_mode`/`origin_rules` exactly once, the first time a
    // pre-v1 config (`policy_schema_version == 0`) loads -- but on *every*
    // normalize call after that (every load and persist), it runs the other
    // direction: it overwrites these three fields FROM the current
    // `network_mode`/`origin_rules`, so they read as an always-current
    // mirror for older UI/API consumers, never as a live input. Never
    // authoritative at runtime either way. Dated removal plan (CR3 P3.2, per
    // review request -- "give them a dated removal plan or they become
    // permanent"): drop `consent_to_remote_page_data`/`local_only`/
    // `blocked_origins` from this struct (and their serialization, UI
    // marshalling in `settings_adapters.rs`, and the corresponding fields on
    // `RemotePlannerSettings`/`RemotePlannerPrivacyOperationResult`) the next
    // time `REMOTE_DATA_POLICY_VERSION` bumps past `1` -- that version bump
    // is this schema's existing signal for "every config has been touched by
    // the current normalization logic," so it's the natural removal gate
    // rather than an arbitrary calendar date.
    #[serde(default)]
    pub consent_to_remote_page_data: bool,
    #[serde(default)]
    pub local_only: bool,
    #[serde(default)]
    pub blocked_origins: Vec<String>,
    #[serde(default)]
    pub network_mode: RemotePlannerNetworkMode,
    #[serde(default)]
    pub origin_rules: Vec<RemotePlannerOriginRule>,
    #[serde(default)]
    pub policy_schema_version: u32,
    #[serde(default)]
    pub migration_notice_pending: bool,
}

impl Default for RemotePlannerPrivacySettings {
    fn default() -> Self {
        Self {
            consent_to_remote_page_data: false,
            local_only: false,
            blocked_origins: Vec::new(),
            network_mode: RemotePlannerNetworkMode::AskPerOrigin,
            origin_rules: Vec::new(),
            policy_schema_version: REMOTE_DATA_POLICY_VERSION,
            migration_notice_pending: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ModelManagementSettings {
    pub models_dir: String,
    pub check_on_startup: bool,
    pub auto_download_missing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum SpeechFeedbackStyle {
    Short,
    Detailed,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SpeechFeedbackSettings {
    pub style: SpeechFeedbackStyle,
    pub confirm_setting_changes: bool,
    pub include_previous_value: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum RemoteProviderKind {
    OpenAi,
    Ollama,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum RemoteTtsAudioFormat {
    #[serde(rename = "wav")]
    Wav,
}

impl std::fmt::Display for RemoteTtsAudioFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wav => f.write_str("wav"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum LocalTtsBackend {
    #[serde(rename = "kitten_tts_rs")]
    KittenTtsRs,
}

impl std::fmt::Display for LocalTtsBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::KittenTtsRs => f.write_str("kitten_tts_rs"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum LocalAsrBackend {
    #[serde(rename = "whisper")]
    Whisper,
}

impl std::fmt::Display for LocalAsrBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Whisper => f.write_str("whisper"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RemotePlannerProfile {
    pub provider: RemoteProviderKind,
    pub base_url: String,
    pub model: String,
    pub api_key: SecretRef,
    pub organization: Option<SecretRef>,
    pub project: Option<String>,
    pub temperature_milli: u16,
    pub max_output_tokens: u32,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RemoteTtsProfile {
    pub provider: RemoteProviderKind,
    pub base_url: String,
    pub model: String,
    pub api_key: SecretRef,
    pub organization: Option<SecretRef>,
    pub project: Option<String>,
    pub voice: String,
    pub audio_format: RemoteTtsAudioFormat,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RemoteAsrProfile {
    pub provider: RemoteProviderKind,
    pub base_url: String,
    pub model: String,
    pub api_key: SecretRef,
    pub organization: Option<SecretRef>,
    pub project: Option<String>,
    pub language: Option<String>,
    pub temperature_milli: u16,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct LocalTtsProfile {
    pub backend: LocalTtsBackend,
    pub model_id: String,
    pub model_path: String,
    pub default_voice: String,
    pub sample_rate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct LocalAsrProfile {
    pub backend: LocalAsrBackend,
    pub model_id: String,
    pub model_path: String,
    pub language: Option<String>,
    pub threads: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct AppConfig {
    pub providers: ProviderSelections,
    pub remote_planner_profiles: BTreeMap<String, RemotePlannerProfile>,
    pub remote_tts_profiles: BTreeMap<String, RemoteTtsProfile>,
    pub remote_asr_profiles: BTreeMap<String, RemoteAsrProfile>,
    pub local_tts_profiles: BTreeMap<String, LocalTtsProfile>,
    pub local_asr_profiles: BTreeMap<String, LocalAsrProfile>,
    pub audio: AudioSettings,
    pub safety: SafetySettings,
    pub remote_planner_privacy: RemotePlannerPrivacySettings,
    // Independent privacy policies for remote TTS narration and remote ASR
    // microphone audio: same shape and decision engine as
    // remote_planner_privacy (see remote_data_consent::evaluate_remote_planner_policy),
    // but a separate origin-rules/network-mode instance per disclosure kind so
    // a grant for one (e.g. "always allow the planner on this origin") can
    // never silently authorize another (e.g. narration text or microphone
    // audio leaving the device for that same origin) -- these are materially
    // different user decisions. #[serde(default)] so existing config files
    // written before this field existed load with the same restrictive
    // AskPerOrigin default as a fresh install, not an implicit broad allow.
    #[serde(default)]
    pub remote_narration_privacy: RemotePlannerPrivacySettings,
    #[serde(default)]
    pub remote_microphone_privacy: RemotePlannerPrivacySettings,
    pub ocr: OcrSettings,
    pub models: ModelManagementSettings,
    pub speech_feedback: SpeechFeedbackSettings,
}

#[derive(Debug, Deserialize)]
pub(super) struct RawAppConfig {
    pub(super) providers: ProviderSelections,
    pub(super) audio: AudioSettings,
    pub(super) safety: SafetySettings,
    #[serde(default)]
    pub(super) remote_planner_privacy: RemotePlannerPrivacySettings,
    #[serde(default)]
    pub(super) remote_narration_privacy: RemotePlannerPrivacySettings,
    #[serde(default)]
    pub(super) remote_microphone_privacy: RemotePlannerPrivacySettings,
    pub(super) ocr: OcrSettings,
    pub(super) models: ModelManagementSettings,
    pub(super) speech_feedback: SpeechFeedbackSettings,
    #[serde(default)]
    pub(super) remote_profiles: BTreeMap<String, toml::Table>,
    #[serde(default)]
    pub(super) local_profiles: BTreeMap<String, toml::Table>,
}
