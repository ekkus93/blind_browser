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
// docs/SPECS.md: "temperature should be clamped to a supported range such as
// 0.0 to 2.0." `temperature_milli` stores that range in milli-units (0..=2000).
pub const MAX_TEMPERATURE_MILLI: u16 = 2000;

const DEFAULT_CONFIG_TEMPLATE: &str = include_str!("../../../config.example.toml");

mod keyring_store;
mod loading;
mod persistence;
mod types;
mod validation;
pub use keyring_store::{
    keyring_ref_for_remote_api_key, resolve_secret_ref, resolve_secret_ref_for_endpoint,
    secret_ref_reference,
};
use loading::{load_planner_profiles, load_provider_profiles};
pub use types::*;
use validation::{
    normalize_remote_planner_privacy_settings, validate_audio_settings, validate_local_asr_profile,
    validate_local_tts_profile, validate_model_settings, validate_ocr_settings,
    validate_remote_asr_profile, validate_remote_planner_profile, validate_remote_tts_profile,
    validate_safety_settings,
};

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

    fn from_raw(raw: RawAppConfig) -> Result<Self, ConfigError> {
        let mut issues = Vec::new();

        validate_audio_settings(&raw.audio, &mut issues);
        validate_safety_settings(&raw.safety, &mut issues);
        let mut remote_planner_privacy = raw.remote_planner_privacy;
        normalize_remote_planner_privacy_settings(&mut remote_planner_privacy, &mut issues);
        let mut remote_narration_privacy = raw.remote_narration_privacy;
        normalize_remote_planner_privacy_settings(&mut remote_narration_privacy, &mut issues);
        let mut remote_microphone_privacy = raw.remote_microphone_privacy;
        normalize_remote_planner_privacy_settings(&mut remote_microphone_privacy, &mut issues);
        validate_ocr_settings(&raw.ocr, &mut issues);
        validate_model_settings(&raw.models, &mut issues);

        // CR3 P3.1.2: docs/SPECS.md documents per-profile validation rules
        // (positive timeout_ms/max_output_tokens/threads/sample_rate,
        // required model_path, clamped temperature) that nothing previously
        // implemented -- `resolve_profile::<T>` below only structurally
        // deserializes a profile, with no field-level checks. Validated
        // here, after the profiles are resolved, so a hand-edited config.toml
        // (or programmatic persistence, though no current persist path can
        // produce an out-of-range value for these specific fields) fails
        // loudly instead of loading a profile with e.g. `timeout_ms = 0`.
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

        for (name, profile) in &remote_planner_profiles {
            validate_remote_planner_profile(name, profile, &mut issues);
        }
        for (name, profile) in &remote_tts_profiles {
            validate_remote_tts_profile(name, profile, &mut issues);
        }
        for (name, profile) in &remote_asr_profiles {
            validate_remote_asr_profile(name, profile, &mut issues);
        }
        for (name, profile) in &local_tts_profiles {
            validate_local_tts_profile(name, profile, &mut issues);
        }
        for (name, profile) in &local_asr_profiles {
            validate_local_asr_profile(name, profile, &mut issues);
        }

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
            remote_planner_privacy,
            remote_narration_privacy,
            remote_microphone_privacy,
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
                // Conservative default: confirm ordinary clicks (opt into faster
                // unconfirmed clicks via config). Submit stays always-confirmed.
                allow_click_without_confirmation: false,
                always_confirm_submit: true,
            },
            remote_planner_privacy: RemotePlannerPrivacySettings::default(),
            remote_narration_privacy: RemotePlannerPrivacySettings::default(),
            remote_microphone_privacy: RemotePlannerPrivacySettings::default(),
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
