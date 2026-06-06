use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};
use tracing::info;

use crate::ocr::OcrSettings;

pub const DEFAULT_CONFIG_FILE_NAME: &str = "config.toml";
pub const MIN_PLAYBACK_VOLUME: f32 = 0.0;
pub const MAX_PLAYBACK_VOLUME: f32 = 1.0;
pub const MIN_PLAYBACK_SPEED: f32 = 0.5;
pub const MAX_PLAYBACK_SPEED: f32 = 5.0;

const DEFAULT_CONFIG_TEMPLATE: &str = include_str!("../../../config.example.toml");

mod types;
mod loading;
mod validation;
mod keyring_store;
pub use keyring_store::{keyring_ref_for_remote_api_key, resolve_secret_ref, secret_ref_reference};
use keyring_store::set_keyring_secret;
use validation::{normalize_remote_endpoint, validate_audio_settings, validate_model_settings, validate_ocr_settings, validate_safety_settings};
use loading::{load_document_table_from_path, load_document_table_from_str, load_planner_profiles, load_provider_profiles};
pub use types::*;

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

    pub fn persist_remote_planner_connection_settings_for_app(
        app_handle: &AppHandle,
        profile_name: &str,
        base_url: &str,
        model: &str,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_remote_planner_connection_settings_at_path(
            &config_path,
            profile_name,
            base_url,
            model,
        )
    }

    pub fn reset_remote_planner_connection_settings_to_defaults_for_app(
        app_handle: &AppHandle,
        profile_name: &str,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::reset_remote_planner_connection_settings_to_defaults_at_path(&config_path, profile_name)
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

    pub fn persist_remote_planner_connection_settings_at_path(
        path: impl AsRef<Path>,
        profile_name: &str,
        base_url: &str,
        model: &str,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let normalized_profile_name = profile_name.trim();
        if normalized_profile_name.is_empty() {
            return Err(ConfigError::Validation(String::from(
                "remote planner settings persistence requires a non-empty configured profile name",
            )));
        }

        let normalized_base_url = normalize_remote_endpoint(base_url)?;
        let normalized_model = model.trim();
        if normalized_model.is_empty() {
            return Err(ConfigError::Validation(String::from(
                "remote planner settings persistence requires a non-empty model",
            )));
        }

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
            String::from("base_url"),
            toml::Value::String(normalized_base_url),
        );
        profile_table.insert(
            String::from("model"),
            toml::Value::String(String::from(normalized_model)),
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

    pub fn reset_remote_planner_connection_settings_to_defaults_at_path(
        path: impl AsRef<Path>,
        profile_name: &str,
    ) -> Result<Self, ConfigError> {
        let normalized_profile_name = profile_name.trim();
        if normalized_profile_name.is_empty() {
            return Err(ConfigError::Validation(String::from(
                "remote planner defaults reset requires a non-empty configured profile name",
            )));
        }

        let default_config = Self::load_from_str(Self::default_template())?;
        let Some(default_profile) = default_config.remote_planner_profiles.get(normalized_profile_name) else {
            return Err(ConfigError::Validation(format!(
                "remote planner defaults are not defined for profile '{normalized_profile_name}'"
            )));
        };

        Self::persist_remote_planner_connection_settings_at_path(
            path,
            normalized_profile_name,
            &default_profile.base_url,
            &default_profile.model,
        )
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
mod tests;
