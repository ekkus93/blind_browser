use super::model_management::{
    kitten_download_plan_for_model_id, local_asr_model_is_available, local_tts_model_is_available,
    whisper_download_plan_for_model_id,
};
use super::{ManagedLocalModelStatusData, ModelManagementSettingsData};
use crate::audio_io::RuntimeAudioState;
use crate::commands::{
    AsrProviderSettings, ConfirmationSettings, LocalAsrModelSettings, LocalTtsModelSettings,
    OcrThresholdSettings, ProviderFailoverSettings, RemoteAsrSettings, RemotePlannerSettings,
    RemoteProviderLabel, RemoteTtsSettings, TtsModelOption, TtsModelSettings, TtsProviderSettings,
    TtsVoiceOption, TtsVoiceSettings,
};
use crate::config::{
    secret_ref_reference, AppConfig, LocalAsrProfile, LocalTtsProfile, RemoteProviderKind,
};
use crate::tts::{KITTEN_TTS_VOICES, OPENAI_TTS_VOICES};

fn remote_provider_label(provider: &RemoteProviderKind) -> RemoteProviderLabel {
    match provider {
        RemoteProviderKind::OpenAi => RemoteProviderLabel::OpenAi,
        RemoteProviderKind::Ollama => RemoteProviderLabel::Ollama,
    }
}

fn masked_secret_value(secret_ref: &crate::config::SecretRef) -> Option<String> {
    let secret = crate::config::resolve_secret_ref(secret_ref).ok()?;
    let trimmed_secret = secret.trim();
    if trimmed_secret.is_empty() {
        return None;
    }

    let suffix = trimmed_secret
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();
    Some(format!("***{suffix}"))
}

pub(crate) fn build_tts_model_settings(config: &AppConfig) -> TtsModelSettings {
    let mode = config.providers.tts.mode.clone();
    let (active_profile, available_profiles) = match mode {
        crate::config::ProviderMode::Local => (
            config.providers.tts.local_profile.clone(),
            config
                .local_tts_profiles
                .iter()
                .map(|(profile_name, profile)| TtsModelOption {
                    profile_name: profile_name.clone(),
                    model_label: profile.model_id.clone(),
                })
                .collect(),
        ),
        crate::config::ProviderMode::Remote => (
            config.providers.tts.remote_profile.clone(),
            config
                .remote_tts_profiles
                .iter()
                .map(|(profile_name, profile)| TtsModelOption {
                    profile_name: profile_name.clone(),
                    model_label: profile.model.clone(),
                })
                .collect(),
        ),
    };

    TtsModelSettings {
        mode,
        active_profile,
        available_profiles,
    }
}

pub(crate) fn build_local_tts_model_settings(config: &AppConfig) -> LocalTtsModelSettings {
    let profile_name = config.providers.tts.local_profile.clone();
    let profile = profile_name
        .as_ref()
        .and_then(|configured_profile| config.local_tts_profiles.get(configured_profile));

    LocalTtsModelSettings {
        profile_name,
        backend: profile.map(|configured_profile| configured_profile.backend.clone()),
        model_id: profile.map(|configured_profile| configured_profile.model_id.clone()),
        model_path: profile.map(|configured_profile| configured_profile.model_path.clone()),
        default_voice: profile.map(|configured_profile| configured_profile.default_voice.clone()),
        sample_rate: profile.map(|configured_profile| configured_profile.sample_rate),
    }
}

pub(crate) fn build_tts_provider_settings(config: &AppConfig) -> TtsProviderSettings {
    let mut available_modes = Vec::new();
    if config
        .providers
        .tts
        .local_profile
        .as_ref()
        .and_then(|profile_name| config.local_tts_profiles.get(profile_name))
        .is_some()
    {
        available_modes.push(crate::config::ProviderMode::Local);
    }
    if config
        .providers
        .tts
        .remote_profile
        .as_ref()
        .and_then(|profile_name| config.remote_tts_profiles.get(profile_name))
        .is_some()
    {
        available_modes.push(crate::config::ProviderMode::Remote);
    }
    if available_modes.is_empty() {
        available_modes.push(config.providers.tts.mode.clone());
    }

    TtsProviderSettings {
        active_mode: config.providers.tts.mode.clone(),
        available_modes,
    }
}

pub(crate) fn build_remote_planner_settings(config: &AppConfig) -> RemotePlannerSettings {
    let profile_name = config.providers.planner.remote_profile.clone();
    let profile = profile_name
        .as_ref()
        .and_then(|configured_profile| config.remote_planner_profiles.get(configured_profile));

    RemotePlannerSettings {
        profile_name,
        provider: profile
            .map(|configured_profile| remote_provider_label(&configured_profile.provider)),
        base_url: profile.map(|configured_profile| configured_profile.base_url.clone()),
        model: profile.map(|configured_profile| configured_profile.model.clone()),
        api_key_reference: profile
            .map(|configured_profile| secret_ref_reference(&configured_profile.api_key)),
        api_key_masked_value: profile
            .and_then(|configured_profile| masked_secret_value(&configured_profile.api_key)),
        organization_reference: profile
            .and_then(|configured_profile| configured_profile.organization.as_ref())
            .map(secret_ref_reference),
        project: profile.and_then(|configured_profile| configured_profile.project.clone()),
        temperature_milli: profile.map(|configured_profile| configured_profile.temperature_milli),
        max_output_tokens: profile.map(|configured_profile| configured_profile.max_output_tokens),
        timeout_ms: profile.map(|configured_profile| configured_profile.timeout_ms),
    }
}

pub(crate) fn build_remote_tts_settings(config: &AppConfig) -> RemoteTtsSettings {
    let profile_name = config.providers.tts.remote_profile.clone();
    let profile = profile_name
        .as_ref()
        .and_then(|configured_profile| config.remote_tts_profiles.get(configured_profile));

    RemoteTtsSettings {
        profile_name,
        provider: profile
            .map(|configured_profile| remote_provider_label(&configured_profile.provider)),
        base_url: profile.map(|configured_profile| configured_profile.base_url.clone()),
        model: profile.map(|configured_profile| configured_profile.model.clone()),
        api_key_reference: profile
            .map(|configured_profile| secret_ref_reference(&configured_profile.api_key)),
        api_key_masked_value: profile
            .and_then(|configured_profile| masked_secret_value(&configured_profile.api_key)),
        organization_reference: profile
            .and_then(|configured_profile| configured_profile.organization.as_ref())
            .map(secret_ref_reference),
        project: profile.and_then(|configured_profile| configured_profile.project.clone()),
        voice: profile.map(|configured_profile| configured_profile.voice.clone()),
        audio_format: profile.map(|configured_profile| configured_profile.audio_format.clone()),
        timeout_ms: profile.map(|configured_profile| configured_profile.timeout_ms),
    }
}

pub(crate) fn build_remote_asr_settings(config: &AppConfig) -> RemoteAsrSettings {
    let profile_name = config.providers.asr.remote_profile.clone();
    let profile = profile_name
        .as_ref()
        .and_then(|configured_profile| config.remote_asr_profiles.get(configured_profile));

    RemoteAsrSettings {
        profile_name,
        provider: profile
            .map(|configured_profile| remote_provider_label(&configured_profile.provider)),
        base_url: profile.map(|configured_profile| configured_profile.base_url.clone()),
        model: profile.map(|configured_profile| configured_profile.model.clone()),
        api_key_reference: profile
            .map(|configured_profile| secret_ref_reference(&configured_profile.api_key)),
        api_key_masked_value: profile
            .and_then(|configured_profile| masked_secret_value(&configured_profile.api_key)),
        organization_reference: profile
            .and_then(|configured_profile| configured_profile.organization.as_ref())
            .map(secret_ref_reference),
        project: profile.and_then(|configured_profile| configured_profile.project.clone()),
        language: profile.and_then(|configured_profile| configured_profile.language.clone()),
        temperature_milli: profile.map(|configured_profile| configured_profile.temperature_milli),
        timeout_ms: profile.map(|configured_profile| configured_profile.timeout_ms),
    }
}

pub(crate) fn build_provider_failover_settings(_config: &AppConfig) -> ProviderFailoverSettings {
    ProviderFailoverSettings {
        planner_available: false,
        tts_available: false,
        asr_available: false,
        summary: String::from(
            "Provider failover settings are defined in config, but automatic failover is still disabled in the live runtime.",
        ),
    }
}

pub(crate) fn build_confirmation_settings(config: &AppConfig) -> ConfirmationSettings {
    ConfirmationSettings {
        confirmation_confidence_threshold: config.safety.confirmation_confidence_threshold,
        allow_click_without_confirmation: config.safety.allow_click_without_confirmation,
        always_confirm_submit: config.safety.always_confirm_submit,
    }
}

pub(crate) fn build_ocr_threshold_settings(config: &AppConfig) -> OcrThresholdSettings {
    OcrThresholdSettings {
        sparse_text_char_threshold: config.ocr.sparse_text_char_threshold,
        sparse_text_region_threshold: config.ocr.sparse_text_region_threshold,
    }
}

pub(crate) fn build_asr_provider_settings(config: &AppConfig) -> AsrProviderSettings {
    let mut available_modes = Vec::new();
    if config
        .providers
        .asr
        .local_profile
        .as_ref()
        .and_then(|profile_name| config.local_asr_profiles.get(profile_name))
        .is_some()
    {
        available_modes.push(crate::config::ProviderMode::Local);
    }
    if config
        .providers
        .asr
        .remote_profile
        .as_ref()
        .and_then(|profile_name| config.remote_asr_profiles.get(profile_name))
        .is_some()
    {
        available_modes.push(crate::config::ProviderMode::Remote);
    }
    if available_modes.is_empty() {
        available_modes.push(config.providers.asr.mode.clone());
    }

    AsrProviderSettings {
        active_mode: config.providers.asr.mode.clone(),
        available_modes,
    }
}

pub(crate) fn build_local_asr_model_settings(config: &AppConfig) -> LocalAsrModelSettings {
    let profile_name = config.providers.asr.local_profile.clone();
    let profile = profile_name
        .as_ref()
        .and_then(|configured_profile| config.local_asr_profiles.get(configured_profile));

    LocalAsrModelSettings {
        profile_name,
        backend: profile.map(|configured_profile| configured_profile.backend.clone()),
        model_id: profile.map(|configured_profile| configured_profile.model_id.clone()),
        model_path: profile.map(|configured_profile| configured_profile.model_path.clone()),
        language: profile.and_then(|configured_profile| configured_profile.language.clone()),
        threads: profile.map(|configured_profile| configured_profile.threads),
    }
}

pub(crate) fn build_model_management_settings(config: &AppConfig) -> ModelManagementSettingsData {
    let (local_tts_profile_name, local_tts_profile) =
        match config.providers.tts.local_profile.as_ref() {
            Some(profile_name) => (
                Some(profile_name.clone()),
                config.local_tts_profiles.get(profile_name),
            ),
            None => (None, None),
        };
    let (local_asr_profile_name, local_asr_profile) =
        match config.providers.asr.local_profile.as_ref() {
            Some(profile_name) => (
                Some(profile_name.clone()),
                config.local_asr_profiles.get(profile_name),
            ),
            None => (None, None),
        };

    ModelManagementSettingsData {
        models_dir: config.models.models_dir.clone(),
        check_on_startup: config.models.check_on_startup,
        auto_download_missing: config.models.auto_download_missing,
        local_tts: ManagedLocalModelStatusData {
            profile_name: local_tts_profile_name,
            backend: local_tts_profile.map(|profile| profile.backend.to_string()),
            model_id: local_tts_profile.map(|profile| profile.model_id.clone()),
            model_path: local_tts_profile.map(|profile| profile.model_path.clone()),
            available: local_tts_profile.is_some_and(local_tts_model_is_available),
            download_supported: local_tts_profile.is_some_and(|profile| {
                kitten_download_plan_for_model_id(&profile.model_id).is_ok()
            }),
            download_label: local_tts_profile
                .and_then(|profile| kitten_download_plan_for_model_id(&profile.model_id).ok())
                .map(|plan| format!("Download {}", plan.display_name)),
        },
        local_asr: ManagedLocalModelStatusData {
            profile_name: local_asr_profile_name,
            backend: local_asr_profile.map(|profile| profile.backend.to_string()),
            model_id: local_asr_profile.map(|profile| profile.model_id.clone()),
            model_path: local_asr_profile.map(|profile| profile.model_path.clone()),
            available: local_asr_profile.is_some_and(local_asr_model_is_available),
            download_supported: local_asr_profile.is_some_and(|profile| {
                whisper_download_plan_for_model_id(&profile.model_id).is_ok()
            }),
            download_label: local_asr_profile
                .and_then(|profile| whisper_download_plan_for_model_id(&profile.model_id).ok())
                .map(|plan| format!("Download Whisper {}", plan.display_name)),
        },
    }
}

pub(crate) fn build_tts_voice_settings(
    config: &AppConfig,
    runtime_audio: &RuntimeAudioState,
) -> TtsVoiceSettings {
    let mode = config.providers.tts.mode.clone();
    let mut available_voices = match mode {
        crate::config::ProviderMode::Local => KITTEN_TTS_VOICES
            .iter()
            .map(|voice| TtsVoiceOption {
                voice_name: (*voice).to_string(),
                display_label: (*voice).to_string(),
            })
            .collect(),
        crate::config::ProviderMode::Remote => {
            let active_remote_profile = config
                .providers
                .tts
                .remote_profile
                .as_ref()
                .and_then(|profile_name| config.remote_tts_profiles.get(profile_name));
            match active_remote_profile.map(|profile| &profile.provider) {
                Some(RemoteProviderKind::OpenAi) => OPENAI_TTS_VOICES
                    .iter()
                    .map(|voice| TtsVoiceOption {
                        voice_name: (*voice).to_string(),
                        display_label: (*voice).to_string(),
                    })
                    .collect(),
                Some(_) => active_remote_profile
                    .map(|profile| {
                        vec![TtsVoiceOption {
                            voice_name: profile.voice.clone(),
                            display_label: profile.voice.clone(),
                        }]
                    })
                    .unwrap_or_default(),
                None => Vec::new(),
            }
        }
    };
    let mut active_voice = runtime_audio
        .tts_voice
        .as_deref()
        .map(str::trim)
        .filter(|voice| !voice.is_empty())
        .map(ToOwned::to_owned);

    if let Some(current_voice) = active_voice.clone() {
        if let Some(option) = available_voices
            .iter()
            .find(|option| option.voice_name.eq_ignore_ascii_case(&current_voice))
        {
            active_voice = Some(option.voice_name.clone());
        } else {
            available_voices.insert(
                0,
                TtsVoiceOption {
                    voice_name: current_voice.clone(),
                    display_label: current_voice,
                },
            );
        }
    }

    TtsVoiceSettings {
        mode,
        active_voice,
        available_voices,
    }
}

pub(crate) fn active_local_tts_profile(
    config: &AppConfig,
) -> Result<(String, &LocalTtsProfile), String> {
    let profile_name = config
        .providers
        .tts
        .local_profile
        .clone()
        .ok_or_else(|| String::from("No local TTS profile is configured."))?;
    let profile = config
        .local_tts_profiles
        .get(&profile_name)
        .ok_or_else(|| format!("Configured local TTS profile '{profile_name}' was not found."))?;
    Ok((profile_name, profile))
}

pub(crate) fn active_local_asr_profile(
    config: &AppConfig,
) -> Result<(String, &LocalAsrProfile), String> {
    let profile_name = config
        .providers
        .asr
        .local_profile
        .clone()
        .ok_or_else(|| String::from("No local ASR profile is configured."))?;
    let profile = config
        .local_asr_profiles
        .get(&profile_name)
        .ok_or_else(|| format!("Configured local ASR profile '{profile_name}' was not found."))?;
    Ok((profile_name, profile))
}
