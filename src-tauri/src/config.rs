use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use thiserror::Error;
use tracing::info;

use crate::ocr::OcrSettings;

pub const DEFAULT_CONFIG_FILE_NAME: &str = "config.toml";
pub const MIN_PLAYBACK_VOLUME: f32 = 0.0;
pub const MAX_PLAYBACK_VOLUME: f32 = 1.0;
pub const MIN_PLAYBACK_SPEED: f32 = 0.5;
pub const MAX_PLAYBACK_SPEED: f32 = 5.0;

const DEFAULT_CONFIG_TEMPLATE: &str = include_str!("../../config.example.toml");

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
    pub ocr: OcrSettings,
    pub models: ModelManagementSettings,
    pub speech_feedback: SpeechFeedbackSettings,
}

#[derive(Debug, Deserialize)]
struct RawAppConfig {
    providers: ProviderSelections,
    audio: AudioSettings,
    safety: SafetySettings,
    ocr: OcrSettings,
    models: ModelManagementSettings,
    speech_feedback: SpeechFeedbackSettings,
    #[serde(default)]
    remote_profiles: BTreeMap<String, toml::Table>,
    #[serde(default)]
    local_profiles: BTreeMap<String, toml::Table>,
}

impl AppConfig {
    pub fn default_template() -> &'static str {
        DEFAULT_CONFIG_TEMPLATE
    }

    pub fn config_path_for_app(app_handle: &AppHandle) -> Result<PathBuf, ConfigError> {
        let config_dir = app_handle
            .path()
            .app_config_dir()
            .map_err(|error| ConfigError::ResolveAppConfigDir(error.to_string()))?;

        Ok(config_dir.join(DEFAULT_CONFIG_FILE_NAME))
    }

    pub fn load_for_app(app_handle: &AppHandle) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;

        if config_path.exists() {
            return Self::load_from_path(&config_path);
        }

        info!(
            path = %config_path.display(),
            "config file not found; using embedded default template"
        );
        Self::load_from_str(Self::default_template())
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;

        Self::load_from_str(&contents)
    }

    pub fn load_from_str(contents: &str) -> Result<Self, ConfigError> {
        let raw: RawAppConfig = toml::from_str(contents)?;
        Self::from_raw(raw)
    }

    pub fn persist_audio_settings_for_app(
        app_handle: &AppHandle,
        audio: &AudioSettings,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_audio_settings_at_path(&config_path, audio)
    }

    pub fn persist_safety_settings_for_app(
        app_handle: &AppHandle,
        safety: &SafetySettings,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_safety_settings_at_path(&config_path, safety)
    }

    pub fn persist_ocr_settings_for_app(
        app_handle: &AppHandle,
        ocr: &OcrSettings,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_ocr_settings_at_path(&config_path, ocr)
    }

    pub fn persist_model_management_settings_for_app(
        app_handle: &AppHandle,
        models: &ModelManagementSettings,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_model_management_settings_at_path(&config_path, models)
    }

    pub fn persist_remote_planner_api_key_for_app(
        app_handle: &AppHandle,
        profile_name: &str,
        api_key: &str,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_remote_api_key_at_path(&config_path, profile_name, api_key, "planner")
    }

    pub fn persist_remote_tts_api_key_for_app(
        app_handle: &AppHandle,
        profile_name: &str,
        api_key: &str,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_remote_api_key_at_path(&config_path, profile_name, api_key, "tts")
    }

    pub fn persist_remote_asr_api_key_for_app(
        app_handle: &AppHandle,
        profile_name: &str,
        api_key: &str,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_remote_api_key_at_path(&config_path, profile_name, api_key, "asr")
    }

    pub fn persist_local_tts_model_path_for_app(
        app_handle: &AppHandle,
        profile_name: &str,
        model_path: &str,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_local_model_path_at_path(&config_path, profile_name, model_path)
    }

    pub fn persist_local_asr_model_path_for_app(
        app_handle: &AppHandle,
        profile_name: &str,
        model_path: &str,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_local_model_path_at_path(&config_path, profile_name, model_path)
    }

    pub fn persist_audio_settings_at_path(
        path: impl AsRef<Path>,
        audio: &AudioSettings,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let mut issues = Vec::new();
        validate_audio_settings(audio, &mut issues);
        if !issues.is_empty() {
            return Err(ConfigError::Validation(issues.join("\n")));
        }

        let mut document = if path.exists() {
            load_document_table_from_path(path)?
        } else {
            load_document_table_from_str(Self::default_template())?
        };

        document.insert(String::from("audio"), toml::Value::try_from(audio.clone())?);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let serialized = toml::to_string_pretty(&document)?;
        fs::write(path, serialized).map_err(|source| ConfigError::Write {
            path: path.to_path_buf(),
            source,
        })?;

        Self::load_from_path(path)
    }

    pub fn persist_safety_settings_at_path(
        path: impl AsRef<Path>,
        safety: &SafetySettings,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();

        let mut document = if path.exists() {
            load_document_table_from_path(path)?
        } else {
            load_document_table_from_str(Self::default_template())?
        };

        document.insert(
            String::from("safety"),
            toml::Value::try_from(safety.clone())?,
        );

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let serialized = toml::to_string_pretty(&document)?;
        fs::write(path, serialized).map_err(|source| ConfigError::Write {
            path: path.to_path_buf(),
            source,
        })?;

        Self::load_from_path(path)
    }

    pub fn persist_ocr_settings_at_path(
        path: impl AsRef<Path>,
        ocr: &OcrSettings,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();

        let mut document = if path.exists() {
            load_document_table_from_path(path)?
        } else {
            load_document_table_from_str(Self::default_template())?
        };

        document.insert(String::from("ocr"), toml::Value::try_from(ocr.clone())?);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let serialized = toml::to_string_pretty(&document)?;
        fs::write(path, serialized).map_err(|source| ConfigError::Write {
            path: path.to_path_buf(),
            source,
        })?;

        Self::load_from_path(path)
    }

    pub fn persist_model_management_settings_at_path(
        path: impl AsRef<Path>,
        models: &ModelManagementSettings,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let mut issues = Vec::new();
        validate_model_settings(models, &mut issues);
        if !issues.is_empty() {
            return Err(ConfigError::Validation(issues.join("\n")));
        }

        let mut document = if path.exists() {
            load_document_table_from_path(path)?
        } else {
            load_document_table_from_str(Self::default_template())?
        };

        document.insert(
            String::from("models"),
            toml::Value::try_from(models.clone())?,
        );

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let serialized = toml::to_string_pretty(&document)?;
        fs::write(path, serialized).map_err(|source| ConfigError::Write {
            path: path.to_path_buf(),
            source,
        })?;

        Self::load_from_path(path)
    }

    pub fn persist_remote_api_key_at_path(
        path: impl AsRef<Path>,
        profile_name: &str,
        api_key: &str,
        provider_kind: &str,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let normalized_profile_name = profile_name.trim();
        if normalized_profile_name.is_empty() {
            return Err(ConfigError::Validation(String::from(
                "remote API key persistence requires a non-empty configured profile name",
            )));
        }

        let normalized_api_key = api_key.trim();
        if normalized_api_key.is_empty() {
            return Err(ConfigError::Validation(String::from(
                "remote API key persistence requires a non-empty API key value",
            )));
        }

        let keyring_ref = keyring_ref_for_remote_api_key(provider_kind, normalized_profile_name);
        set_keyring_secret(
            &keyring_ref.service,
            &keyring_ref.account,
            normalized_api_key,
        )
        .map_err(ConfigError::Keyring)?;

        let mut document = if path.exists() {
            load_document_table_from_path(path)?
        } else {
            load_document_table_from_str(Self::default_template())?
        };

        let remote_profiles_value = document
            .entry(String::from("remote_profiles"))
            .or_insert_with(|| toml::Value::Table(toml::Table::new()));
        let Some(remote_profiles_table) = remote_profiles_value.as_table_mut() else {
            return Err(ConfigError::Validation(String::from(
                "remote_profiles must remain a TOML table",
            )));
        };

        let Some(profile_value) = remote_profiles_table.get_mut(normalized_profile_name) else {
            return Err(ConfigError::Validation(format!(
                "remote_profiles.{normalized_profile_name} is not configured"
            )));
        };
        let Some(profile_table) = profile_value.as_table_mut() else {
            return Err(ConfigError::Validation(format!(
                "remote_profiles.{normalized_profile_name} must remain a TOML table"
            )));
        };
        profile_table.insert(
            String::from("api_key"),
            toml::Value::try_from(SecretRef::FromKeyring {
                from_keyring: keyring_ref,
            })?,
        );

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let serialized = toml::to_string_pretty(&document)?;
        fs::write(path, serialized).map_err(|source| ConfigError::Write {
            path: path.to_path_buf(),
            source,
        })?;

        Self::load_from_path(path)
    }

    pub fn persist_local_model_path_at_path(
        path: impl AsRef<Path>,
        profile_name: &str,
        model_path: &str,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let normalized_profile_name = profile_name.trim();
        if normalized_profile_name.is_empty() {
            return Err(ConfigError::Validation(String::from(
                "local model path persistence requires a non-empty configured profile name",
            )));
        }

        let normalized_model_path = model_path.trim();
        if normalized_model_path.is_empty() {
            return Err(ConfigError::Validation(String::from(
                "local model path persistence requires a non-empty model path",
            )));
        }

        let mut document = if path.exists() {
            load_document_table_from_path(path)?
        } else {
            load_document_table_from_str(Self::default_template())?
        };

        let local_profiles_value = document
            .entry(String::from("local_profiles"))
            .or_insert_with(|| toml::Value::Table(toml::Table::new()));
        let Some(local_profiles_table) = local_profiles_value.as_table_mut() else {
            return Err(ConfigError::Validation(String::from(
                "local_profiles must remain a TOML table",
            )));
        };

        let Some(profile_value) = local_profiles_table.get_mut(normalized_profile_name) else {
            return Err(ConfigError::Validation(format!(
                "local_profiles.{normalized_profile_name} is not configured"
            )));
        };
        let Some(profile_table) = profile_value.as_table_mut() else {
            return Err(ConfigError::Validation(format!(
                "local_profiles.{normalized_profile_name} must remain a TOML table"
            )));
        };
        profile_table.insert(
            String::from("model_path"),
            toml::Value::String(normalized_model_path.to_string()),
        );

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let serialized = toml::to_string_pretty(&document)?;
        fs::write(path, serialized).map_err(|source| ConfigError::Write {
            path: path.to_path_buf(),
            source,
        })?;

        Self::load_from_path(path)
    }

    pub fn persist_tts_provider_selection_for_app(
        app_handle: &AppHandle,
        selection: &ProviderSelection,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_tts_provider_selection_at_path(&config_path, selection)
    }

    pub fn persist_asr_provider_selection_for_app(
        app_handle: &AppHandle,
        selection: &ProviderSelection,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_asr_provider_selection_at_path(&config_path, selection)
    }

    pub fn persist_asr_provider_selection_at_path(
        path: impl AsRef<Path>,
        selection: &ProviderSelection,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();

        let mut document = if path.exists() {
            load_document_table_from_path(path)?
        } else {
            load_document_table_from_str(Self::default_template())?
        };

        let providers_value = document
            .entry(String::from("providers"))
            .or_insert_with(|| toml::Value::Table(toml::Table::new()));
        let Some(providers_table) = providers_value.as_table_mut() else {
            return Err(ConfigError::Validation(String::from(
                "providers must remain a TOML table",
            )));
        };
        providers_table.insert(
            String::from("asr"),
            toml::Value::try_from(selection.clone())?,
        );

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let serialized = toml::to_string_pretty(&document)?;
        fs::write(path, serialized).map_err(|source| ConfigError::Write {
            path: path.to_path_buf(),
            source,
        })?;

        Self::load_from_path(path)
    }

    pub fn persist_tts_provider_selection_at_path(
        path: impl AsRef<Path>,
        selection: &ProviderSelection,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();

        let mut document = if path.exists() {
            load_document_table_from_path(path)?
        } else {
            load_document_table_from_str(Self::default_template())?
        };

        let providers_value = document
            .entry(String::from("providers"))
            .or_insert_with(|| toml::Value::Table(toml::Table::new()));
        let Some(providers_table) = providers_value.as_table_mut() else {
            return Err(ConfigError::Validation(String::from(
                "providers must remain a TOML table",
            )));
        };
        providers_table.insert(
            String::from("tts"),
            toml::Value::try_from(selection.clone())?,
        );

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let serialized = toml::to_string_pretty(&document)?;
        fs::write(path, serialized).map_err(|source| ConfigError::Write {
            path: path.to_path_buf(),
            source,
        })?;

        Self::load_from_path(path)
    }

    fn from_raw(raw: RawAppConfig) -> Result<Self, ConfigError> {
        let mut issues = Vec::new();

        validate_audio_settings(&raw.audio, &mut issues);
        validate_safety_settings(&raw.safety, &mut issues);
        validate_ocr_settings(&raw.ocr, &mut issues);
        validate_model_settings(&raw.models, &mut issues);

        let mut remote_planner_profiles = BTreeMap::new();
        let mut remote_tts_profiles = BTreeMap::new();
        let mut remote_asr_profiles = BTreeMap::new();
        let mut local_tts_profiles = BTreeMap::new();
        let mut local_asr_profiles = BTreeMap::new();

        load_planner_profiles(
            &raw.providers.planner,
            &raw.remote_profiles,
            &mut remote_planner_profiles,
            &mut issues,
        );
        load_provider_profiles(
            "tts",
            &raw.providers.tts,
            &raw.remote_profiles,
            &raw.local_profiles,
            &mut remote_tts_profiles,
            &mut local_tts_profiles,
            &mut issues,
        );
        load_provider_profiles(
            "asr",
            &raw.providers.asr,
            &raw.remote_profiles,
            &raw.local_profiles,
            &mut remote_asr_profiles,
            &mut local_asr_profiles,
            &mut issues,
        );

        if !issues.is_empty() {
            return Err(ConfigError::Validation(issues.join("\n")));
        }

        Ok(Self {
            providers: raw.providers,
            remote_planner_profiles,
            remote_tts_profiles,
            remote_asr_profiles,
            local_tts_profiles,
            local_asr_profiles,
            audio: raw.audio,
            safety: raw.safety,
            ocr: raw.ocr,
            models: raw.models,
            speech_feedback: raw.speech_feedback,
        })
    }
}

pub fn keyring_ref_for_remote_api_key(provider_kind: &str, profile_name: &str) -> KeyringRef {
    KeyringRef {
        service: String::from("blind_browser"),
        account: format!("remote_{provider_kind}:{profile_name}:api_key"),
    }
}

pub fn secret_ref_reference(secret_ref: &SecretRef) -> String {
    match secret_ref {
        SecretRef::FromEnv { from_env } => format!("Environment variable: {from_env}"),
        SecretRef::FromFile { from_file } => format!("File reference: {from_file}"),
        SecretRef::FromKeyring { from_keyring } => {
            format!(
                "OS keyring entry: {} / {}",
                from_keyring.service, from_keyring.account
            )
        }
    }
}

pub fn resolve_secret_ref(secret_ref: &SecretRef) -> Result<String, String> {
    match secret_ref {
        SecretRef::FromEnv { from_env } => std::env::var(from_env)
            .map(|value| value.trim().to_string())
            .map_err(|error| format!("failed to read environment variable '{from_env}': {error}")),
        SecretRef::FromFile { from_file } => fs::read_to_string(from_file)
            .map(|value| value.trim().to_string())
            .map_err(|error| format!("failed to read secret file '{from_file}': {error}")),
        SecretRef::FromKeyring { from_keyring } => {
            get_keyring_secret(&from_keyring.service, &from_keyring.account)
        }
    }
    .and_then(|value| {
        if value.is_empty() {
            Err(String::from("resolved secret value was empty"))
        } else {
            Ok(value)
        }
    })
}

fn cache_keyring_secret(service: &str, account: &str, secret: &str) -> Result<(), String> {
    let store = session_keyring_store();
    let mut store = store
        .lock()
        .map_err(|_| String::from("failed to acquire the session keyring cache lock"))?;
    store.insert(
        (String::from(service), String::from(account)),
        String::from(secret),
    );
    Ok(())
}

fn cached_keyring_secret(service: &str, account: &str) -> Result<Option<String>, String> {
    let store = session_keyring_store();
    let store = store
        .lock()
        .map_err(|_| String::from("failed to acquire the session keyring cache lock"))?;
    Ok(store
        .get(&(String::from(service), String::from(account)))
        .cloned())
}

fn session_keyring_store() -> &'static Mutex<BTreeMap<(String, String), String>> {
    static STORE: OnceLock<Mutex<BTreeMap<(String, String), String>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

#[cfg(not(test))]
fn set_keyring_secret(service: &str, account: &str, secret: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(service, account)
        .map_err(|error| format!("failed to open keyring entry '{service}/{account}': {error}"))?;
    entry
        .set_password(secret)
        .map_err(|error| format!("failed to store keyring secret '{service}/{account}': {error}"))?;

    cache_keyring_secret(service, account, secret)
}

#[cfg(not(test))]
fn get_keyring_secret(service: &str, account: &str) -> Result<String, String> {
    if let Some(cached_secret) = cached_keyring_secret(service, account)? {
        return Ok(cached_secret);
    }

    let entry = keyring::Entry::new(service, account)
        .map_err(|error| format!("failed to open keyring entry '{service}/{account}': {error}"))?;
    let secret = entry
        .get_password()
        .map_err(|error| format!("failed to read keyring secret '{service}/{account}': {error}"))?;

    cache_keyring_secret(service, account, &secret)?;
    Ok(secret)
}

#[cfg(test)]
fn set_keyring_secret(service: &str, account: &str, secret: &str) -> Result<(), String> {
    cache_keyring_secret(service, account, secret)
}

#[cfg(test)]
fn get_keyring_secret(service: &str, account: &str) -> Result<String, String> {
    cached_keyring_secret(service, account)?
        .ok_or_else(|| format!("failed to read keyring secret '{service}/{account}': no entry"))
}

fn load_document_table_from_path(path: &Path) -> Result<toml::Table, ConfigError> {
    let contents = fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path.to_path_buf(),
        source,
    })?;

    load_document_table_from_str(&contents)
}

fn load_document_table_from_str(contents: &str) -> Result<toml::Table, ConfigError> {
    toml::from_str(contents).map_err(ConfigError::Parse)
}

fn load_planner_profiles(
    selection: &ProviderSelection,
    remote_profiles: &BTreeMap<String, toml::Table>,
    resolved_remote_profiles: &mut BTreeMap<String, RemotePlannerProfile>,
    issues: &mut Vec<String>,
) {
    if selection.mode != ProviderMode::Remote {
        issues.push(String::from(
            "providers.planner.mode must be \"remote\"; local planner support has been removed",
        ));
    }

    if selection.local_profile.is_some() {
        issues.push(String::from(
            "providers.planner.local_profile is not supported; use a remote planner profile such as openai-default or ollama-default",
        ));
    }

    if selection.failover_to_local.is_some() {
        issues.push(String::from(
            "providers.planner.failover_to_local is not supported; use the selected remote planner directly",
        ));
    }

    if let Some(profile_name) = selection.remote_profile.as_deref() {
        if let Some(profile) = resolve_profile::<RemotePlannerProfile>(
            "remote_profiles",
            "planner",
            profile_name,
            remote_profiles,
            issues,
        ) {
            resolved_remote_profiles.insert(profile_name.to_owned(), profile);
        }
    } else {
        issues.push(String::from(
            "providers.planner.remote_profile is required when mode = \"remote\"",
        ));
    }
}

fn load_provider_profiles<RemoteProfile, LocalProfile>(
    category: &str,
    selection: &ProviderSelection,
    remote_profiles: &BTreeMap<String, toml::Table>,
    local_profiles: &BTreeMap<String, toml::Table>,
    resolved_remote_profiles: &mut BTreeMap<String, RemoteProfile>,
    resolved_local_profiles: &mut BTreeMap<String, LocalProfile>,
    issues: &mut Vec<String>,
) where
    RemoteProfile: DeserializeOwned,
    LocalProfile: DeserializeOwned,
{
    if let Some(profile_name) = selection.remote_profile.as_deref() {
        if let Some(profile) = resolve_profile::<RemoteProfile>(
            "remote_profiles",
            category,
            profile_name,
            remote_profiles,
            issues,
        ) {
            resolved_remote_profiles.insert(profile_name.to_owned(), profile);
        }
    } else if selection.mode == ProviderMode::Remote {
        issues.push(format!(
            "providers.{category}.remote_profile is required when mode = \"remote\""
        ));
    }

    if let Some(profile_name) = selection.local_profile.as_deref() {
        if let Some(profile) = resolve_profile::<LocalProfile>(
            "local_profiles",
            category,
            profile_name,
            local_profiles,
            issues,
        ) {
            resolved_local_profiles.insert(profile_name.to_owned(), profile);
        }
    } else if selection.mode == ProviderMode::Local {
        issues.push(format!(
            "providers.{category}.local_profile is required when mode = \"local\""
        ));
    }
}

fn resolve_profile<T>(
    profile_group: &str,
    category: &str,
    profile_name: &str,
    profiles: &BTreeMap<String, toml::Table>,
    issues: &mut Vec<String>,
) -> Option<T>
where
    T: DeserializeOwned,
{
    let Some(table) = profiles.get(profile_name) else {
        issues.push(format!(
            "providers.{category} references missing {profile_group}.{profile_name}"
        ));
        return None;
    };

    match toml::Value::Table(table.clone()).try_into::<T>() {
        Ok(profile) => Some(profile),
        Err(error) => {
            issues.push(format!(
                "{profile_group}.{profile_name} is not valid for {category}: {error}"
            ));
            None
        }
    }
}

fn validate_audio_settings(audio: &AudioSettings, issues: &mut Vec<String>) {
    if !(MIN_PLAYBACK_VOLUME..=MAX_PLAYBACK_VOLUME).contains(&audio.playback_volume) {
        issues.push(String::from(
            "audio.playback_volume must be between 0.0 and 1.0",
        ));
    }

    if !(MIN_PLAYBACK_SPEED..=MAX_PLAYBACK_SPEED).contains(&audio.playback_speed) {
        issues.push(format!(
            "audio.playback_speed must be between {MIN_PLAYBACK_SPEED} and {MAX_PLAYBACK_SPEED}"
        ));
    }

    if audio.default_tts_voice.trim().is_empty() {
        issues.push(String::from("audio.default_tts_voice must not be empty"));
    }
}

fn validate_safety_settings(safety: &SafetySettings, issues: &mut Vec<String>) {
    if !(0.0..=1.0).contains(&safety.confirmation_confidence_threshold) {
        issues.push(String::from(
            "safety.confirmation_confidence_threshold must be between 0.0 and 1.0",
        ));
    }
}

fn validate_ocr_settings(ocr: &OcrSettings, issues: &mut Vec<String>) {
    if ocr.sparse_text_char_threshold == 0 {
        issues.push(String::from(
            "ocr.sparse_text_char_threshold must be greater than 0",
        ));
    }

    if ocr.sparse_text_region_threshold == 0 {
        issues.push(String::from(
            "ocr.sparse_text_region_threshold must be greater than 0",
        ));
    }
}

fn validate_model_settings(models: &ModelManagementSettings, issues: &mut Vec<String>) {
    if models.models_dir.trim().is_empty() {
        issues.push(String::from("models.models_dir must not be empty"));
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        let planner_remote = RemotePlannerProfile {
            provider: RemoteProviderKind::OpenAi,
            base_url: String::from("https://api.openai.com/v1"),
            model: String::from("gpt-5.4-mini"),
            api_key: SecretRef::FromEnv {
                from_env: String::from("OPENAI_API_KEY"),
            },
            organization: None,
            project: None,
            temperature_milli: 200,
            max_output_tokens: 1024,
            timeout_ms: 30_000,
        };
        let planner_ollama = RemotePlannerProfile {
            provider: RemoteProviderKind::Ollama,
            base_url: String::from("http://localhost:11434/v1"),
            model: String::from("qwen2.5:3b-instruct"),
            api_key: SecretRef::FromEnv {
                from_env: String::from("OLLAMA_API_KEY"),
            },
            organization: None,
            project: None,
            temperature_milli: 200,
            max_output_tokens: 1024,
            timeout_ms: 30_000,
        };

        let tts_remote = RemoteTtsProfile {
            provider: RemoteProviderKind::OpenAi,
            base_url: String::from("https://api.openai.com/v1"),
            model: String::from("gpt-4o-mini-tts"),
            api_key: SecretRef::FromEnv {
                from_env: String::from("OPENAI_API_KEY"),
            },
            organization: None,
            project: None,
            voice: String::from("alloy"),
            audio_format: RemoteTtsAudioFormat::Wav,
            timeout_ms: 30_000,
        };

        let asr_remote = RemoteAsrProfile {
            provider: RemoteProviderKind::OpenAi,
            base_url: String::from("https://api.openai.com/v1"),
            model: String::from("gpt-4o-mini-transcribe"),
            api_key: SecretRef::FromEnv {
                from_env: String::from("OPENAI_API_KEY"),
            },
            organization: None,
            project: None,
            language: Some(String::from("en")),
            temperature_milli: 0,
            timeout_ms: 30_000,
        };

        let mut remote_planner_profiles = std::collections::BTreeMap::new();
        remote_planner_profiles.insert(String::from("openai-default"), planner_remote);
        remote_planner_profiles.insert(String::from("ollama-default"), planner_ollama);

        let mut remote_tts_profiles = std::collections::BTreeMap::new();
        remote_tts_profiles.insert(String::from("openai-tts-default"), tts_remote);

        let mut remote_asr_profiles = std::collections::BTreeMap::new();
        remote_asr_profiles.insert(String::from("openai-transcribe-default"), asr_remote);

        let mut local_tts_profiles = std::collections::BTreeMap::new();
        local_tts_profiles.insert(
            String::from("kitten-default"),
            LocalTtsProfile {
                backend: LocalTtsBackend::KittenTtsRs,
                model_id: String::from("default"),
                model_path: String::from("/path/to/kitten/model"),
                default_voice: String::from("Bruno"),
                sample_rate: 24_000,
            },
        );

        let mut local_asr_profiles = std::collections::BTreeMap::new();
        local_asr_profiles.insert(
            String::from("whisper-default"),
            LocalAsrProfile {
                backend: LocalAsrBackend::Whisper,
                model_id: String::from("tiny"),
                model_path: String::from("/path/to/whisper/model"),
                language: Some(String::from("en")),
                threads: 4,
            },
        );

        Self {
            providers: ProviderSelections {
                planner: ProviderSelection {
                    mode: ProviderMode::Remote,
                    remote_profile: Some(String::from("openai-default")),
                    local_profile: None,
                    failover_to_local: None,
                },
                tts: ProviderSelection {
                    mode: ProviderMode::Remote,
                    remote_profile: Some(String::from("openai-tts-default")),
                    local_profile: Some(String::from("kitten-default")),
                    failover_to_local: None,
                },
                asr: ProviderSelection {
                    mode: ProviderMode::Remote,
                    remote_profile: Some(String::from("openai-transcribe-default")),
                    local_profile: Some(String::from("whisper-default")),
                    failover_to_local: None,
                },
            },
            remote_planner_profiles,
            remote_tts_profiles,
            remote_asr_profiles,
            local_tts_profiles,
            local_asr_profiles,
            audio: AudioSettings {
                playback_volume: 1.0,
                playback_speed: 1.0,
                default_tts_voice: String::from("Bruno"),
            },
            safety: SafetySettings {
                confirmation_confidence_threshold: 0.90,
                allow_click_without_confirmation: true,
                always_confirm_submit: true,
            },
            ocr: OcrSettings::default(),
            models: ModelManagementSettings {
                models_dir: String::from("~/.config/blind_browser/models"),
                check_on_startup: true,
                auto_download_missing: false,
            },
            speech_feedback: SpeechFeedbackSettings {
                style: SpeechFeedbackStyle::Short,
                confirm_setting_changes: true,
                include_previous_value: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        keyring_ref_for_remote_api_key, resolve_secret_ref, secret_ref_reference,
        set_keyring_secret, AppConfig, AudioSettings, ConfigError, KeyringRef, LocalAsrBackend,
        LocalAsrProfile, LocalTtsBackend, LocalTtsProfile, ProviderMode, ProviderSelection,
        ProviderSelections, RemoteAsrProfile, RemotePlannerProfile, RemoteProviderKind,
        RemoteTtsAudioFormat, RemoteTtsProfile, SafetySettings, SecretRef, SpeechFeedbackStyle,
    };
    use crate::ocr::OcrSettings;

    fn test_config_path(label: &str) -> PathBuf {
        let unique_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after UNIX_EPOCH")
            .as_nanos();

        std::env::temp_dir()
            .join(format!(
                "blind_browser_{label}_{}_{}",
                std::process::id(),
                unique_id
            ))
            .join("config.toml")
    }

    fn test_temp_path(label: &str, file_name: &str) -> PathBuf {
        let unique_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after UNIX_EPOCH")
            .as_nanos();

        std::env::temp_dir().join(format!(
            "blind_browser_{label}_{}_{}_{}",
            std::process::id(),
            unique_id,
            file_name
        ))
    }

    #[test]
    fn config_enums_round_trip_and_reject_invalid_variants() {
        fn assert_enum_round_trip<T>(
            value: T,
            expected: serde_json::Value,
            invalid: serde_json::Value,
        ) where
            T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
        {
            let serialized = serde_json::to_value(&value).expect("enum should serialize");
            assert_eq!(serialized, expected);

            let round_tripped: T =
                serde_json::from_value(serialized).expect("enum should deserialize");
            assert_eq!(round_tripped, value);
            assert!(serde_json::from_value::<T>(invalid).is_err());
        }

        assert_enum_round_trip(
            ProviderMode::Remote,
            serde_json::json!("remote"),
            serde_json::json!("REMOTE"),
        );
        assert_enum_round_trip(
            RemoteProviderKind::OpenAi,
            serde_json::json!("OpenAi"),
            serde_json::json!("Anthropic"),
        );
        assert_enum_round_trip(
            RemoteTtsAudioFormat::Wav,
            serde_json::json!("wav"),
            serde_json::json!("mp3"),
        );
        assert_enum_round_trip(
            LocalTtsBackend::KittenTtsRs,
            serde_json::json!("kitten_tts_rs"),
            serde_json::json!("kitten"),
        );
        assert_enum_round_trip(
            LocalAsrBackend::Whisper,
            serde_json::json!("whisper"),
            serde_json::json!("deepgram"),
        );
        assert_enum_round_trip(
            SpeechFeedbackStyle::Detailed,
            serde_json::json!("Detailed"),
            serde_json::json!("Verbose"),
        );
    }

    #[test]
    fn provider_configs_round_trip_through_json() {
        let providers = ProviderSelections {
            planner: ProviderSelection {
                mode: ProviderMode::Remote,
                remote_profile: Some(String::from("openai-default")),
                local_profile: None,
                failover_to_local: None,
            },
            tts: ProviderSelection {
                mode: ProviderMode::Local,
                remote_profile: Some(String::from("openai-tts-default")),
                local_profile: Some(String::from("kitten-default")),
                failover_to_local: Some(false),
            },
            asr: ProviderSelection {
                mode: ProviderMode::Remote,
                remote_profile: Some(String::from("openai-asr-default")),
                local_profile: Some(String::from("whisper-default")),
                failover_to_local: Some(true),
            },
        };
        let planner_profile = RemotePlannerProfile {
            provider: RemoteProviderKind::OpenAi,
            base_url: String::from("https://api.openai.com/v1"),
            model: String::from("gpt-4.1"),
            api_key: SecretRef::FromEnv {
                from_env: String::from("OPENAI_API_KEY"),
            },
            organization: Some(SecretRef::FromEnv {
                from_env: String::from("OPENAI_ORG_ID"),
            }),
            project: Some(String::from("blind-browser")),
            temperature_milli: 250,
            max_output_tokens: 1024,
            timeout_ms: 30_000,
        };
        let remote_tts_profile = RemoteTtsProfile {
            provider: RemoteProviderKind::OpenAi,
            base_url: String::from("https://api.openai.com/v1"),
            model: String::from("gpt-4o-mini-tts"),
            api_key: SecretRef::FromKeyring {
                from_keyring: KeyringRef {
                    service: String::from("blind-browser"),
                    account: String::from("tts/openai-tts-default"),
                },
            },
            organization: None,
            project: Some(String::from("blind-browser")),
            voice: String::from("alloy"),
            audio_format: RemoteTtsAudioFormat::Wav,
            timeout_ms: 20_000,
        };
        let remote_asr_profile = RemoteAsrProfile {
            provider: RemoteProviderKind::OpenAi,
            base_url: String::from("https://api.openai.com/v1"),
            model: String::from("gpt-4o-mini-transcribe"),
            api_key: SecretRef::FromFile {
                from_file: String::from("/tmp/openai-asr.key"),
            },
            organization: None,
            project: Some(String::from("blind-browser")),
            language: Some(String::from("en")),
            temperature_milli: 0,
            timeout_ms: 20_000,
        };
        let local_tts_profile = LocalTtsProfile {
            backend: LocalTtsBackend::KittenTtsRs,
            model_id: String::from("default"),
            model_path: String::from("/models/kitten/default.onnx"),
            default_voice: String::from("Bruno"),
            sample_rate: 24_000,
        };
        let local_asr_profile = LocalAsrProfile {
            backend: LocalAsrBackend::Whisper,
            model_id: String::from("tiny"),
            model_path: String::from("/models/whisper/tiny.bin"),
            language: Some(String::from("en")),
            threads: 4,
        };

        let serialized = serde_json::json!({
            "providers": providers,
            "planner_profile": planner_profile,
            "remote_tts_profile": remote_tts_profile,
            "remote_asr_profile": remote_asr_profile,
            "local_tts_profile": local_tts_profile,
            "local_asr_profile": local_asr_profile
        });

        let round_tripped = serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(
            serialized.clone(),
        )
        .expect("provider config payload should deserialize as JSON object");

        let decoded_providers: ProviderSelections =
            serde_json::from_value(serialized.get("providers").cloned().unwrap())
                .expect("provider selections should deserialize");
        let decoded_planner_profile: RemotePlannerProfile =
            serde_json::from_value(serialized.get("planner_profile").cloned().unwrap())
                .expect("planner profile should deserialize");
        let decoded_remote_tts_profile: RemoteTtsProfile =
            serde_json::from_value(serialized.get("remote_tts_profile").cloned().unwrap())
                .expect("remote tts profile should deserialize");
        let decoded_remote_asr_profile: RemoteAsrProfile =
            serde_json::from_value(serialized.get("remote_asr_profile").cloned().unwrap())
                .expect("remote asr profile should deserialize");
        let decoded_local_tts_profile: LocalTtsProfile =
            serde_json::from_value(serialized.get("local_tts_profile").cloned().unwrap())
                .expect("local tts profile should deserialize");
        let decoded_local_asr_profile: LocalAsrProfile =
            serde_json::from_value(serialized.get("local_asr_profile").cloned().unwrap())
                .expect("local asr profile should deserialize");

        assert_eq!(decoded_providers, providers);
        assert_eq!(decoded_planner_profile, planner_profile);
        assert_eq!(decoded_remote_tts_profile, remote_tts_profile);
        assert_eq!(decoded_remote_asr_profile, remote_asr_profile);
        assert_eq!(decoded_local_tts_profile, local_tts_profile);
        assert_eq!(decoded_local_asr_profile, local_asr_profile);
        assert!(round_tripped.contains_key("providers"));
    }

    #[test]
    fn secret_ref_reference_formats_sources_without_secret_values() {
        let env_reference = secret_ref_reference(&SecretRef::FromEnv {
            from_env: String::from("OPENAI_API_KEY"),
        });
        let file_reference = secret_ref_reference(&SecretRef::FromFile {
            from_file: String::from("/secure/openai.key"),
        });
        let keyring_reference = secret_ref_reference(&SecretRef::FromKeyring {
            from_keyring: KeyringRef {
                service: String::from("blind-browser"),
                account: String::from("planner/openai-default"),
            },
        });

        assert_eq!(env_reference, "Environment variable: OPENAI_API_KEY");
        assert_eq!(file_reference, "File reference: /secure/openai.key");
        assert_eq!(
            keyring_reference,
            "OS keyring entry: blind-browser / planner/openai-default"
        );
        assert!(!env_reference.contains("super-secret"));
        assert!(!file_reference.contains("super-secret"));
        assert!(!keyring_reference.contains("super-secret"));
    }

    #[test]
    fn resolve_secret_ref_reads_all_supported_reference_types() {
        let env_var_name = format!(
            "BLIND_BROWSER_TEST_SECRET_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after UNIX_EPOCH")
                .as_nanos()
        );
        let file_path = test_temp_path("resolve_secret_file", "secret.txt");

        std::env::set_var(&env_var_name, "env-secret");
        fs::write(&file_path, "file-secret").expect("secret file should write");
        set_keyring_secret("blind-browser", "tests/keyring-secret", "keyring-secret")
            .expect("keyring secret should store");

        let env_secret = resolve_secret_ref(&SecretRef::FromEnv {
            from_env: env_var_name.clone(),
        })
        .expect("env secret should resolve");
        let file_secret = resolve_secret_ref(&SecretRef::FromFile {
            from_file: file_path.display().to_string(),
        })
        .expect("file secret should resolve");
        let keyring_secret = resolve_secret_ref(&SecretRef::FromKeyring {
            from_keyring: KeyringRef {
                service: String::from("blind-browser"),
                account: String::from("tests/keyring-secret"),
            },
        })
        .expect("keyring secret should resolve");

        assert_eq!(env_secret, "env-secret");
        assert_eq!(file_secret, "file-secret");
        assert_eq!(keyring_secret, "keyring-secret");

        std::env::remove_var(&env_var_name);
        let _ = fs::remove_file(&file_path);
    }

    #[test]
    fn resolve_secret_ref_rejects_missing_or_empty_values() {
        let missing_env_name = format!(
            "BLIND_BROWSER_TEST_MISSING_SECRET_{}_{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after UNIX_EPOCH")
                .as_nanos()
        );
        let empty_file_path = test_temp_path("resolve_empty_secret_file", "empty-secret.txt");

        fs::write(&empty_file_path, "").expect("empty secret file should write");
        set_keyring_secret("blind-browser", "tests/empty-keyring-secret", "")
            .expect("empty keyring secret should store");

        let missing_env_error = resolve_secret_ref(&SecretRef::FromEnv {
            from_env: missing_env_name,
        })
        .expect_err("missing env secret should fail");
        let empty_file_error = resolve_secret_ref(&SecretRef::FromFile {
            from_file: empty_file_path.display().to_string(),
        })
        .expect_err("empty file secret should fail");
        let empty_keyring_error = resolve_secret_ref(&SecretRef::FromKeyring {
            from_keyring: KeyringRef {
                service: String::from("blind-browser"),
                account: String::from("tests/empty-keyring-secret"),
            },
        })
        .expect_err("empty keyring secret should fail");

        assert!(missing_env_error.contains("failed to read environment variable"));
        assert_eq!(empty_file_error, "resolved secret value was empty");
        assert_eq!(empty_keyring_error, "resolved secret value was empty");

        let _ = fs::remove_file(&empty_file_path);
    }

    #[test]
    fn parses_default_template() {
        let config = AppConfig::load_from_str(AppConfig::default_template())
            .expect("default template should parse and validate");

        assert_eq!(
            config.providers.planner.remote_profile.as_deref(),
            Some("openai-default")
        );
        assert_eq!(config.providers.tts.mode, ProviderMode::Remote);
        assert_eq!(config.providers.asr.mode, ProviderMode::Remote);
        assert!(config
            .remote_planner_profiles
            .contains_key("openai-default"));
        assert!(config.local_tts_profiles.contains_key("kitten-default"));
        assert!(config.local_asr_profiles.contains_key("whisper-default"));
        assert!(config.ocr.trigger_on_no_extractable_text);
        assert_eq!(config.ocr.sparse_text_char_threshold, 200);
        assert_eq!(config.ocr.sparse_text_region_threshold, 2);
        assert!(config.ocr.prefer_region_ocr);
    }

    #[test]
    fn parses_ollama_planner_profile_when_selected() {
        let config = AppConfig::load_from_str(
            r#"
[providers.planner]
mode = "remote"
remote_profile = "ollama-default"

[providers.tts]
mode = "local"
local_profile = "kitten-default"

[providers.asr]
mode = "local"
local_profile = "whisper-default"

[audio]
playback_volume = 1.0
playback_speed = 1.0
default_tts_voice = "Bruno"

[safety]
confirmation_confidence_threshold = 0.9
allow_click_without_confirmation = true
always_confirm_submit = true

[ocr]
trigger_on_no_extractable_text = true
sparse_text_char_threshold = 200
sparse_text_region_threshold = 2
prefer_region_ocr = true

[models]
models_dir = "~/.config/blind_browser/models"
check_on_startup = true
auto_download_missing = false

[speech_feedback]
style = "Short"
confirm_setting_changes = true
include_previous_value = false

[remote_profiles.ollama-default]
provider = "Ollama"
base_url = "http://localhost:11434/v1"
model = "qwen2.5:3b-instruct"
api_key = { from_env = "OLLAMA_API_KEY" }
temperature_milli = 200
max_output_tokens = 1024
timeout_ms = 30000

[local_profiles.kitten-default]
backend = "kitten_tts_rs"
model_id = "default"
model_path = "/path/to/kitten/model"
default_voice = "Bruno"
sample_rate = 24000

[local_profiles.whisper-default]
backend = "whisper"
model_id = "tiny"
model_path = "/path/to/whisper/model"
language = "en"
threads = 4
"#,
        )
        .expect("Ollama planner config should parse and validate");

        let profile = config
            .remote_planner_profiles
            .get("ollama-default")
            .expect("selected Ollama profile should be loaded");
        assert_eq!(profile.provider, RemoteProviderKind::Ollama);
        assert_eq!(profile.base_url, "http://localhost:11434/v1");
        assert_eq!(profile.model, "qwen2.5:3b-instruct");
        assert_eq!(
            profile.api_key,
            SecretRef::FromEnv {
                from_env: String::from("OLLAMA_API_KEY"),
            }
        );
    }

    #[test]
    fn rejects_missing_selected_remote_planner_profile_reference() {
        let invalid = r#"
[providers.planner]
mode = "remote"
remote_profile = "missing-planner-profile"

[providers.tts]
mode = "local"
local_profile = "kitten-default"

[providers.asr]
mode = "local"
local_profile = "whisper-default"

[audio]
playback_volume = 1.0
playback_speed = 1.0
default_tts_voice = "Bruno"

[safety]
confirmation_confidence_threshold = 0.9
allow_click_without_confirmation = true
always_confirm_submit = true

[ocr]
trigger_on_no_extractable_text = true
sparse_text_char_threshold = 200
sparse_text_region_threshold = 2
prefer_region_ocr = true

[models]
models_dir = "~/.config/blind_browser/models"
check_on_startup = true
auto_download_missing = false

[speech_feedback]
style = "Short"
confirm_setting_changes = true
include_previous_value = false

[local_profiles.kitten-default]
backend = "kitten_tts_rs"
model_id = "default"
model_path = "/path/to/kitten/model"
default_voice = "Bruno"
sample_rate = 24000

[local_profiles.whisper-default]
backend = "whisper"
model_id = "tiny"
model_path = "/path/to/whisper/model"
language = "en"
threads = 4
"#;

        let error = AppConfig::load_from_str(invalid).expect_err("config should be invalid");

        match error {
            ConfigError::Validation(message) => {
                assert!(message.contains(
                    "providers.planner references missing remote_profiles.missing-planner-profile"
                ));
            }
            other => panic!("expected validation error, got {other}"),
        }
    }

    #[test]
    fn rejects_inline_secret_refs() {
        let invalid = r#"
[providers.planner]
mode = "remote"
remote_profile = "ollama-default"

[providers.tts]
mode = "local"
local_profile = "kitten-default"

[providers.asr]
mode = "local"
local_profile = "whisper-default"

[audio]
playback_volume = 1.0
playback_speed = 1.0
default_tts_voice = "Bruno"

[safety]
confirmation_confidence_threshold = 0.9
allow_click_without_confirmation = true
always_confirm_submit = true

[ocr]
trigger_on_no_extractable_text = true
sparse_text_char_threshold = 200
sparse_text_region_threshold = 2
prefer_region_ocr = true

[models]
models_dir = "~/.config/blind_browser/models"
check_on_startup = true
auto_download_missing = false

[speech_feedback]
style = "Short"
confirm_setting_changes = true
include_previous_value = false

[remote_profiles.ollama-default]
provider = "Ollama"
base_url = "http://localhost:11434/v1"
model = "qwen2.5:3b-instruct"
api_key = { inline = "ollama" }
temperature_milli = 200
max_output_tokens = 1024
timeout_ms = 30000

[local_profiles.kitten-default]
backend = "kitten_tts_rs"
model_id = "default"
model_path = "/path/to/kitten/model"
default_voice = "Bruno"
sample_rate = 24000

[local_profiles.whisper-default]
backend = "whisper"
model_id = "tiny"
model_path = "/path/to/whisper/model"
language = "en"
threads = 4
"#;

        let error = AppConfig::load_from_str(invalid).expect_err("config should be invalid");
        assert!(
            matches!(error, ConfigError::Validation(ref message) if message.contains("data did not match any variant of untagged enum SecretRef")),
            "expected inline secret refs to fail validation, got {error}"
        );
    }

    #[test]
    fn rejects_local_planner_configuration() {
        let invalid = r#"
[providers.planner]
mode = "local"
local_profile = "ollama-default"
failover_to_local = true

[providers.tts]
mode = "local"
local_profile = "kitten-default"

[providers.asr]
mode = "local"
local_profile = "whisper-default"

[audio]
playback_volume = 1.0
playback_speed = 1.0
default_tts_voice = "Bruno"

[safety]
confirmation_confidence_threshold = 0.9
allow_click_without_confirmation = true
always_confirm_submit = true

[ocr]
trigger_on_no_extractable_text = true
sparse_text_char_threshold = 200
sparse_text_region_threshold = 2
prefer_region_ocr = true

[models]
models_dir = "~/.config/blind_browser/models"
check_on_startup = true
auto_download_missing = false

[speech_feedback]
style = "Short"
confirm_setting_changes = true
include_previous_value = false

[remote_profiles.ollama-default]
provider = "Ollama"
base_url = "http://localhost:11434/v1"
model = "qwen2.5:3b-instruct"
api_key = { from_env = "OLLAMA_API_KEY" }
temperature_milli = 200
max_output_tokens = 1024
timeout_ms = 30000

[local_profiles.kitten-default]
backend = "kitten_tts_rs"
model_id = "default"
model_path = "/path/to/kitten/model"
default_voice = "Bruno"
sample_rate = 24000

[local_profiles.whisper-default]
backend = "whisper"
model_id = "tiny"
model_path = "/path/to/whisper/model"
language = "en"
threads = 4
"#;

        let error = AppConfig::load_from_str(invalid).expect_err("config should be invalid");

        match error {
            ConfigError::Validation(message) => {
                assert!(message.contains("providers.planner.mode must be \"remote\""));
                assert!(message.contains("providers.planner.local_profile is not supported"));
                assert!(message.contains("providers.planner.failover_to_local is not supported"));
            }
            other => panic!("expected validation error, got {other}"),
        }
    }

    #[test]
    fn rejects_missing_remote_profile_for_remote_mode() {
        let invalid = r#"
[providers.planner]
mode = "remote"

[providers.tts]
mode = "local"
local_profile = "kitten-default"

[providers.asr]
mode = "local"
local_profile = "whisper-default"

[audio]
playback_volume = 1.0
playback_speed = 1.0
default_tts_voice = "Bruno"

[safety]
confirmation_confidence_threshold = 0.9
allow_click_without_confirmation = true
always_confirm_submit = true

[ocr]
trigger_on_no_extractable_text = true
sparse_text_char_threshold = 200
sparse_text_region_threshold = 2
prefer_region_ocr = true

[models]
models_dir = "~/.config/blind_browser/models"
check_on_startup = true
auto_download_missing = false

[speech_feedback]
style = "Short"
confirm_setting_changes = true
include_previous_value = false

[local_profiles.kitten-default]
backend = "kitten_tts_rs"
model_id = "default"
model_path = "/path/to/kitten/model"
default_voice = "Bruno"
sample_rate = 24000

[local_profiles.whisper-default]
backend = "whisper"
model_id = "tiny"
model_path = "/path/to/whisper/model"
language = "en"
threads = 4
"#;

        let error = AppConfig::load_from_str(invalid).expect_err("config should be invalid");

        match error {
            ConfigError::Validation(message) => {
                assert!(message.contains("providers.planner.remote_profile is required"));
            }
            other => panic!("expected validation error, got {other}"),
        }
    }

    #[test]
    fn rejects_missing_selected_profiles_for_tts_and_asr_modes() {
        let invalid = r#"
[providers.planner]
mode = "remote"
remote_profile = "openai-default"

[providers.tts]
mode = "remote"

[providers.asr]
mode = "local"

[audio]
playback_volume = 1.0
playback_speed = 1.0
default_tts_voice = "Bruno"

[safety]
confirmation_confidence_threshold = 0.9
allow_click_without_confirmation = true
always_confirm_submit = true

[ocr]
trigger_on_no_extractable_text = true
sparse_text_char_threshold = 200
sparse_text_region_threshold = 2
prefer_region_ocr = true

[models]
models_dir = "~/.config/blind_browser/models"
check_on_startup = true
auto_download_missing = false

[speech_feedback]
style = "Short"
confirm_setting_changes = true
include_previous_value = false

[remote_profiles.openai-default]
provider = "OpenAi"
base_url = "https://api.openai.com/v1"
model = "gpt-4.1"
api_key = { from_env = "OPENAI_API_KEY" }
temperature_milli = 200
max_output_tokens = 1024
timeout_ms = 30000

[local_profiles.kitten-default]
backend = "kitten_tts_rs"
model_id = "default"
model_path = "/path/to/kitten/model"
default_voice = "Bruno"
sample_rate = 24000

[local_profiles.whisper-default]
backend = "whisper"
model_id = "tiny"
model_path = "/path/to/whisper/model"
language = "en"
threads = 4
"#;

        let error = AppConfig::load_from_str(invalid).expect_err("config should be invalid");

        match error {
            ConfigError::Validation(message) => {
                assert!(message.contains("providers.tts.remote_profile is required"));
                assert!(message.contains("providers.asr.local_profile is required"));
            }
            other => panic!("expected validation error, got {other}"),
        }
    }

    #[test]
    fn rejects_missing_selected_local_profile_references_for_tts_and_asr() {
        let invalid = r#"
[providers.planner]
mode = "remote"
remote_profile = "openai-default"

[providers.tts]
mode = "local"
local_profile = "missing-kitten-profile"

[providers.asr]
mode = "local"
local_profile = "missing-whisper-profile"

[audio]
playback_volume = 1.0
playback_speed = 1.0
default_tts_voice = "Bruno"

[safety]
confirmation_confidence_threshold = 0.9
allow_click_without_confirmation = true
always_confirm_submit = true

[ocr]
trigger_on_no_extractable_text = true
sparse_text_char_threshold = 200
sparse_text_region_threshold = 2
prefer_region_ocr = true

[models]
models_dir = "~/.config/blind_browser/models"
check_on_startup = true
auto_download_missing = false

[speech_feedback]
style = "Short"
confirm_setting_changes = true
include_previous_value = false

[remote_profiles.openai-default]
provider = "OpenAi"
base_url = "https://api.openai.com/v1"
model = "gpt-4.1"
api_key = { from_env = "OPENAI_API_KEY" }
temperature_milli = 200
max_output_tokens = 1024
timeout_ms = 30000
"#;

        let error = AppConfig::load_from_str(invalid).expect_err("config should be invalid");

        match error {
            ConfigError::Validation(message) => {
                assert!(message.contains(
                    "providers.tts references missing local_profiles.missing-kitten-profile"
                ));
                assert!(message.contains(
                    "providers.asr references missing local_profiles.missing-whisper-profile"
                ));
            }
            other => panic!("expected validation error, got {other}"),
        }
    }

    #[test]
    fn rejects_out_of_range_audio_settings() {
        let invalid =
            AppConfig::default_template().replace("playback_speed = 1.0", "playback_speed = 7.0");

        let error = AppConfig::load_from_str(&invalid).expect_err("config should be invalid");

        match error {
            ConfigError::Validation(message) => {
                assert!(message.contains("audio.playback_speed must be between"));
            }
            other => panic!("expected validation error, got {other}"),
        }
    }

    #[test]
    fn persists_audio_settings_and_reloads_them() {
        let path = test_config_path("persist_audio");
        let expected_audio = AudioSettings {
            playback_volume: 0.35,
            playback_speed: 1.4,
            default_tts_voice: String::from("Rosie"),
        };

        let persisted = AppConfig::persist_audio_settings_at_path(&path, &expected_audio)
            .expect("audio settings should persist successfully");
        let reloaded = AppConfig::load_from_path(&path).expect("persisted config should reload");

        assert_eq!(persisted.audio, expected_audio);
        assert_eq!(reloaded.audio, expected_audio);

        if let Some(parent) = path.parent() {
            let _ = std::fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn persists_safety_settings_and_reloads_them() {
        let path = test_config_path("persist_safety");
        let expected_safety = SafetySettings {
            confirmation_confidence_threshold: 0.82,
            allow_click_without_confirmation: false,
            always_confirm_submit: true,
        };

        let persisted = AppConfig::persist_safety_settings_at_path(&path, &expected_safety)
            .expect("safety settings should persist successfully");
        let reloaded = AppConfig::load_from_path(&path).expect("persisted config should reload");

        assert_eq!(persisted.safety, expected_safety);
        assert_eq!(reloaded.safety, expected_safety);

        if let Some(parent) = path.parent() {
            let _ = std::fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn persists_ocr_settings_and_reloads_them() {
        let path = test_config_path("persist_ocr");
        let expected_ocr = OcrSettings {
            trigger_on_no_extractable_text: true,
            sparse_text_char_threshold: 120,
            sparse_text_region_threshold: 3,
            prefer_region_ocr: true,
        };

        let persisted = AppConfig::persist_ocr_settings_at_path(&path, &expected_ocr)
            .expect("ocr settings should persist successfully");
        let reloaded = AppConfig::load_from_path(&path).expect("persisted config should reload");

        assert_eq!(persisted.ocr, expected_ocr);
        assert_eq!(reloaded.ocr, expected_ocr);

        if let Some(parent) = path.parent() {
            let _ = std::fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn persists_asr_provider_selection_and_reloads_it() {
        let path = test_config_path("persist_asr_provider");
        let expected_selection = ProviderSelection {
            mode: ProviderMode::Remote,
            remote_profile: Some(String::from("openai-transcribe-default")),
            local_profile: Some(String::from("whisper-default")),
            failover_to_local: None,
        };

        let persisted =
            AppConfig::persist_asr_provider_selection_at_path(&path, &expected_selection)
                .expect("asr provider selection should persist successfully");
        let reloaded = AppConfig::load_from_path(&path).expect("persisted config should reload");

        assert_eq!(persisted.providers.asr, expected_selection);
        assert_eq!(reloaded.providers.asr, expected_selection);
    }

    #[test]
    fn persists_tts_provider_selection_and_reloads_it() {
        let path = test_config_path("persist_tts_provider");
        let expected_selection = ProviderSelection {
            mode: ProviderMode::Remote,
            remote_profile: Some(String::from("openai-tts-default")),
            local_profile: Some(String::from("kitten-default")),
            failover_to_local: None,
        };

        let persisted =
            AppConfig::persist_tts_provider_selection_at_path(&path, &expected_selection)
                .expect("tts provider selection should persist successfully");
        let reloaded = AppConfig::load_from_path(&path).expect("persisted config should reload");

        assert_eq!(persisted.providers.tts, expected_selection);
        assert_eq!(reloaded.providers.tts, expected_selection);

        if let Some(parent) = path.parent() {
            let _ = std::fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn persists_remote_planner_api_key_to_keyring_reference_and_reloads_it() {
        let path = test_config_path("persist_remote_planner_api_key");

        let persisted = AppConfig::persist_remote_api_key_at_path(
            &path,
            "openai-default",
            "super-secret",
            "planner",
        )
        .expect("remote planner API key should persist successfully");
        let reloaded = AppConfig::load_from_path(&path).expect("persisted config should reload");

        let expected_keyring_ref = keyring_ref_for_remote_api_key("planner", "openai-default");

        let expected_secret_ref = SecretRef::FromKeyring {
            from_keyring: KeyringRef {
                service: expected_keyring_ref.service.clone(),
                account: expected_keyring_ref.account.clone(),
            },
        };

        assert_eq!(
            persisted
                .remote_planner_profiles
                .get("openai-default")
                .expect("planner profile should remain present")
                .api_key,
            expected_secret_ref
        );
        assert_eq!(
            reloaded
                .remote_planner_profiles
                .get("openai-default")
                .expect("planner profile should reload")
                .api_key,
            expected_secret_ref
        );
        assert_eq!(
            resolve_secret_ref(&expected_secret_ref).expect("keyring secret should resolve"),
            "super-secret"
        );

        if let Some(parent) = path.parent() {
            let _ = std::fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn rejects_empty_remote_api_key_persistence_input() {
        let path = test_config_path("reject_empty_remote_api_key");

        let error =
            AppConfig::persist_remote_api_key_at_path(&path, "openai-default", "   ", "planner")
                .expect_err("empty API key should be rejected");

        match error {
            ConfigError::Validation(message) => {
                assert!(message.contains("non-empty API key value"));
            }
            other => panic!("expected validation error, got {other}"),
        }
    }
}
