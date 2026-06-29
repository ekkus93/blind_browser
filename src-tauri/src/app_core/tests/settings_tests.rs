use super::*;

#[test]
fn build_remote_planner_settings_reflects_configured_profile_details() {
    let config = AppConfig::default();

    let settings = build_remote_planner_settings(&config);

    assert_eq!(settings.profile_name.as_deref(), Some("openai-default"));
    assert_eq!(
        settings.provider,
        Some(crate::commands::RemoteProviderLabel::OpenAi)
    );
    assert_eq!(
        settings.base_url.as_deref(),
        Some("https://api.openai.com/v1")
    );
    assert_eq!(settings.model.as_deref(), Some("gpt-5.4-mini"));
    assert_eq!(
        settings.api_key_reference.as_deref(),
        Some("Environment variable: OPENAI_API_KEY")
    );
    assert_eq!(settings.organization_reference, None);
    assert_eq!(settings.project, None);
    assert_eq!(settings.temperature_milli, Some(200));
    assert_eq!(settings.max_output_tokens, Some(1024));
    assert_eq!(settings.timeout_ms, Some(30_000));
}

#[test]
fn build_remote_planner_settings_reflects_selected_ollama_profile_details() {
    let mut config = AppConfig::default();
    config.providers.planner.remote_profile = Some(String::from("ollama-default"));

    let settings = build_remote_planner_settings(&config);

    assert_eq!(settings.profile_name.as_deref(), Some("ollama-default"));
    assert_eq!(
        settings.provider,
        Some(crate::commands::RemoteProviderLabel::Ollama)
    );
    assert_eq!(
        settings.base_url.as_deref(),
        Some("http://localhost:11434/v1")
    );
    assert_eq!(settings.model.as_deref(), Some("qwen2.5:3b-instruct"));
    assert_eq!(
        settings.api_key_reference.as_deref(),
        Some("Environment variable: OLLAMA_API_KEY")
    );
    assert_eq!(settings.organization_reference, None);
    assert_eq!(settings.project, None);
    assert_eq!(settings.temperature_milli, Some(200));
    assert_eq!(settings.max_output_tokens, Some(1024));
    assert_eq!(settings.timeout_ms, Some(30_000));
}

#[test]
fn build_remote_tts_settings_reflects_configured_profile_details() {
    let config = AppConfig::default();

    let settings = build_remote_tts_settings(&config);

    assert_eq!(settings.profile_name.as_deref(), Some("openai-tts-default"));
    assert_eq!(
        settings.provider,
        Some(crate::commands::RemoteProviderLabel::OpenAi)
    );
    assert_eq!(
        settings.base_url.as_deref(),
        Some("https://api.openai.com/v1")
    );
    assert_eq!(settings.model.as_deref(), Some("gpt-4o-mini-tts"));
    assert_eq!(
        settings.api_key_reference.as_deref(),
        Some("Environment variable: OPENAI_API_KEY")
    );
    if let Some(masked_value) = settings.api_key_masked_value.as_deref() {
        assert!(masked_value.starts_with("***"));
    }
    assert_eq!(settings.organization_reference, None);
    assert_eq!(settings.project, None);
    assert_eq!(settings.voice.as_deref(), Some("alloy"));
    assert_eq!(
        settings.audio_format,
        Some(crate::config::RemoteTtsAudioFormat::Wav)
    );
    assert_eq!(settings.timeout_ms, Some(30_000));
}

#[test]
fn build_remote_asr_settings_reflects_configured_profile_details() {
    let config = AppConfig::default();

    let settings = build_remote_asr_settings(&config);

    assert_eq!(
        settings.profile_name.as_deref(),
        Some("openai-transcribe-default")
    );
    assert_eq!(
        settings.provider,
        Some(crate::commands::RemoteProviderLabel::OpenAi)
    );
    assert_eq!(
        settings.base_url.as_deref(),
        Some("https://api.openai.com/v1")
    );
    assert_eq!(settings.model.as_deref(), Some("gpt-4o-mini-transcribe"));
    assert_eq!(
        settings.api_key_reference.as_deref(),
        Some("Environment variable: OPENAI_API_KEY")
    );
    if let Some(masked_value) = settings.api_key_masked_value.as_deref() {
        assert!(masked_value.starts_with("***"));
    }
    assert_eq!(settings.organization_reference, None);
    assert_eq!(settings.project, None);
    assert_eq!(settings.language.as_deref(), Some("en"));
    assert_eq!(settings.temperature_milli, Some(0));
    assert_eq!(settings.timeout_ms, Some(30_000));
}

#[test]
fn build_remote_settings_expose_secret_references_without_raw_values() {
    let mut config = AppConfig::default();
    let planner_profile = config
        .remote_planner_profiles
        .get_mut("openai-default")
        .expect("planner profile should exist");
    planner_profile.api_key = SecretRef::FromFile {
        from_file: String::from("/secure/planner.key"),
    };
    planner_profile.organization = Some(SecretRef::FromKeyring {
        from_keyring: KeyringRef {
            service: String::from("blind-browser"),
            account: String::from("planner/openai-default"),
        },
    });

    let settings = build_remote_planner_settings(&config);

    assert_eq!(
        settings.api_key_reference.as_deref(),
        Some("File reference: /secure/planner.key")
    );
    assert_eq!(
        settings.organization_reference.as_deref(),
        Some("OS keyring entry: blind-browser / planner/openai-default")
    );
    assert!(!settings
        .api_key_reference
        .as_deref()
        .unwrap_or_default()
        .contains("super-secret"));
    assert!(!settings
        .organization_reference
        .as_deref()
        .unwrap_or_default()
        .contains("super-secret"));
}

#[test]
fn build_provider_failover_settings_reports_unavailable_runtime_support() {
    let config = AppConfig::default();

    let settings = build_provider_failover_settings(&config);

    assert!(!settings.planner_available);
    assert!(!settings.tts_available);
    assert!(!settings.asr_available);
    assert_eq!(
        settings.summary,
        String::from(
            "Provider failover settings are defined in config, but automatic failover is still disabled in the live runtime."
        )
    );
}

#[test]
fn build_confirmation_settings_reflects_configured_safety_values() {
    let config = AppConfig::default();

    let settings = build_confirmation_settings(&config);

    assert_eq!(settings.confirmation_confidence_threshold, 0.9);
    // Conservative default: ordinary clicks are confirmed unless explicitly opted out.
    assert!(!settings.allow_click_without_confirmation);
    assert!(settings.always_confirm_submit);
}

#[test]
fn build_local_tts_model_settings_reflects_configured_profile_details() {
    let config = AppConfig::default();

    let settings = build_local_tts_model_settings(&config);

    assert_eq!(settings.profile_name.as_deref(), Some("kitten-default"));
    assert_eq!(
        settings.backend,
        Some(crate::config::LocalTtsBackend::KittenTtsRs)
    );
    assert_eq!(settings.model_id.as_deref(), Some("default"));
    assert_eq!(
        settings.model_path.as_deref(),
        Some("/path/to/kitten/model")
    );
    assert_eq!(settings.default_voice.as_deref(), Some("Bruno"));
    assert_eq!(settings.sample_rate, Some(24_000));
}

#[test]
fn build_tts_model_settings_uses_selected_local_profile() {
    let mut config = AppConfig::default();
    config.providers.tts.mode = ProviderMode::Local;
    config.local_tts_profiles.insert(
        String::from("kitten-alt"),
        crate::config::LocalTtsProfile {
            backend: crate::config::LocalTtsBackend::KittenTtsRs,
            model_id: String::from("expressive"),
            model_path: String::from("/path/to/kitten/expressive"),
            default_voice: String::from("Bella"),
            sample_rate: 22_050,
        },
    );
    config.providers.tts.local_profile = Some(String::from("kitten-alt"));

    let settings = build_tts_model_settings(&config);

    assert_eq!(settings.mode, ProviderMode::Local);
    assert_eq!(settings.active_profile.as_deref(), Some("kitten-alt"));
    assert!(settings
        .available_profiles
        .iter()
        .any(|option| option.profile_name == "kitten-default" && option.model_label == "default"));
    assert!(settings
        .available_profiles
        .iter()
        .any(|option| option.profile_name == "kitten-alt" && option.model_label == "expressive"));
}

#[test]
fn build_local_tts_model_settings_reflects_selected_profile_details() {
    let mut config = AppConfig::default();
    config.local_tts_profiles.insert(
        String::from("kitten-alt"),
        crate::config::LocalTtsProfile {
            backend: crate::config::LocalTtsBackend::KittenTtsRs,
            model_id: String::from("expressive"),
            model_path: String::from("/path/to/kitten/expressive"),
            default_voice: String::from("Bella"),
            sample_rate: 22_050,
        },
    );
    config.providers.tts.local_profile = Some(String::from("kitten-alt"));

    let settings = build_local_tts_model_settings(&config);

    assert_eq!(settings.profile_name.as_deref(), Some("kitten-alt"));
    assert_eq!(
        settings.backend,
        Some(crate::config::LocalTtsBackend::KittenTtsRs)
    );
    assert_eq!(settings.model_id.as_deref(), Some("expressive"));
    assert_eq!(
        settings.model_path.as_deref(),
        Some("/path/to/kitten/expressive")
    );
    assert_eq!(settings.default_voice.as_deref(), Some("Bella"));
    assert_eq!(settings.sample_rate, Some(22_050));
}

#[test]
fn build_local_asr_model_settings_reflects_configured_profile_details() {
    let config = AppConfig::default();

    let settings = build_local_asr_model_settings(&config);

    assert_eq!(settings.profile_name.as_deref(), Some("whisper-default"));
    assert_eq!(
        settings.backend,
        Some(crate::config::LocalAsrBackend::Whisper)
    );
    assert_eq!(settings.model_id.as_deref(), Some("tiny"));
    assert_eq!(
        settings.model_path.as_deref(),
        Some("/path/to/whisper/model")
    );
    assert_eq!(settings.language.as_deref(), Some("en"));
    assert_eq!(settings.threads, Some(4));
}

#[test]
fn build_local_asr_model_settings_reflects_selected_profile_details() {
    let mut config = AppConfig::default();
    config.local_asr_profiles.insert(
        String::from("whisper-alt"),
        crate::config::LocalAsrProfile {
            backend: crate::config::LocalAsrBackend::Whisper,
            model_id: String::from("base"),
            model_path: String::from("/path/to/whisper/base"),
            language: Some(String::from("fr")),
            threads: 6,
        },
    );
    config.providers.asr.local_profile = Some(String::from("whisper-alt"));

    let settings = build_local_asr_model_settings(&config);

    assert_eq!(settings.profile_name.as_deref(), Some("whisper-alt"));
    assert_eq!(
        settings.backend,
        Some(crate::config::LocalAsrBackend::Whisper)
    );
    assert_eq!(settings.model_id.as_deref(), Some("base"));
    assert_eq!(
        settings.model_path.as_deref(),
        Some("/path/to/whisper/base")
    );
    assert_eq!(settings.language.as_deref(), Some("fr"));
    assert_eq!(settings.threads, Some(6));
}

#[test]
fn build_ocr_threshold_settings_reflects_configured_ocr_values() {
    let config = AppConfig::default();

    let settings = build_ocr_threshold_settings(&config);

    assert_eq!(settings.sparse_text_char_threshold, 200);
    assert_eq!(settings.sparse_text_region_threshold, 2);
}

#[test]
fn build_asr_provider_settings_returns_available_modes() {
    let config = AppConfig::default();

    let settings = build_asr_provider_settings(&config);

    assert_eq!(settings.active_mode, ProviderMode::Remote);
    assert_eq!(
        settings.available_modes,
        vec![ProviderMode::Local, ProviderMode::Remote]
    );
}

#[test]
fn build_tts_provider_settings_returns_available_modes() {
    let config = AppConfig::default();

    let settings = build_tts_provider_settings(&config);

    assert_eq!(settings.active_mode, ProviderMode::Remote);
    assert_eq!(
        settings.available_modes,
        vec![ProviderMode::Local, ProviderMode::Remote]
    );
}

#[test]
fn build_tts_voice_settings_returns_kitten_voice_choices_for_local_mode() {
    let mut config = AppConfig::default();
    config.providers.tts.mode = ProviderMode::Local;
    let runtime_audio = RuntimeAudioState::from(&config.audio);

    let settings = build_tts_voice_settings(&config, &runtime_audio);

    assert_eq!(settings.mode, ProviderMode::Local);
    assert_eq!(settings.active_voice.as_deref(), Some("Bruno"));
    assert_eq!(settings.available_voices.len(), 8);
    assert!(settings
        .available_voices
        .iter()
        .any(|option| option.voice_name == "Bella"));
    assert!(settings
        .available_voices
        .iter()
        .any(|option| option.voice_name == "Leo"));
}

#[test]
fn build_tts_voice_settings_preserves_custom_active_voice() {
    let config = AppConfig::default();
    let runtime_audio = RuntimeAudioState {
        tts_voice: Some(String::from("CustomVoice")),
        ..RuntimeAudioState::from(&config.audio)
    };

    let settings = build_tts_voice_settings(&config, &runtime_audio);

    assert_eq!(settings.active_voice.as_deref(), Some("CustomVoice"));
    assert_eq!(settings.available_voices[0].voice_name, "CustomVoice");
}

#[test]
fn build_tts_voice_settings_returns_openai_builtin_voices_for_remote_mode() {
    let mut config = AppConfig::default();
    config.providers.tts.mode = ProviderMode::Remote;
    let runtime_audio = RuntimeAudioState {
        tts_voice: Some(String::from("Alloy")),
        ..RuntimeAudioState::from(&config.audio)
    };

    let settings = build_tts_voice_settings(&config, &runtime_audio);

    assert_eq!(settings.mode, ProviderMode::Remote);
    assert_eq!(settings.active_voice.as_deref(), Some("alloy"));
    assert!(settings
        .available_voices
        .iter()
        .any(|option| option.voice_name == "cedar"));
}
