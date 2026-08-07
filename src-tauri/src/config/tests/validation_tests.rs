use super::*;

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

    let persisted = AppConfig::persist_asr_provider_selection_at_path(&path, &expected_selection)
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

    let persisted = AppConfig::persist_tts_provider_selection_at_path(&path, &expected_selection)
        .expect("tts provider selection should persist successfully");
    let reloaded = AppConfig::load_from_path(&path).expect("persisted config should reload");

    assert_eq!(persisted.providers.tts, expected_selection);
    assert_eq!(reloaded.providers.tts, expected_selection);

    if let Some(parent) = path.parent() {
        let _ = std::fs::remove_dir_all(parent);
    }
}

#[test]
fn persists_remote_planner_connection_settings_and_reloads_them() {
    let path = test_config_path("persist_remote_planner_connection_settings");

    let persisted = AppConfig::persist_remote_planner_connection_settings_at_path(
        &path,
        "openai-default",
        " https://example.invalid/v1/ ",
        "gpt-custom",
    )
    .expect("remote planner connection settings should persist successfully");
    let reloaded =
        AppConfig::load_from_path(&path).expect("persisted planner config should reload");

    assert_eq!(
        persisted
            .remote_planner_profiles
            .get("openai-default")
            .expect("planner profile should remain present")
            .base_url,
        "https://example.invalid/v1"
    );
    assert_eq!(
        persisted
            .remote_planner_profiles
            .get("openai-default")
            .expect("planner profile should remain present")
            .model,
        "gpt-custom"
    );
    assert_eq!(
        reloaded
            .remote_planner_profiles
            .get("openai-default")
            .expect("planner profile should reload")
            .base_url,
        "https://example.invalid/v1"
    );
    assert_eq!(
        reloaded
            .remote_planner_profiles
            .get("openai-default")
            .expect("planner profile should reload")
            .model,
        "gpt-custom"
    );

    if let Some(parent) = path.parent() {
        let _ = std::fs::remove_dir_all(parent);
    }
}

#[test]
fn resets_remote_planner_connection_settings_to_defaults() {
    let path = test_config_path("reset_remote_planner_connection_settings");

    AppConfig::persist_remote_planner_connection_settings_at_path(
        &path,
        "openai-default",
        "https://example.invalid/v1",
        "gpt-custom",
    )
    .expect("custom planner settings should persist");

    let reset = AppConfig::reset_remote_planner_connection_settings_to_defaults_at_path(
        &path,
        "openai-default",
    )
    .expect("planner settings should reset to defaults");

    let profile = reset
        .remote_planner_profiles
        .get("openai-default")
        .expect("planner profile should still exist after reset");
    assert_eq!(profile.base_url, "https://api.openai.com/v1");
    assert_eq!(profile.model, "gpt-5.4-mini");

    if let Some(parent) = path.parent() {
        let _ = std::fs::remove_dir_all(parent);
    }
}

// CR3 P3.1.2: docs/SPECS.md documents these as validation rules; none of
// them were enforced by the loader before this pass. Each test below flips
// exactly one field of the default template invalid and asserts the load
// fails with a message naming that field.

#[test]
fn rejects_always_confirm_submit_false() {
    let invalid = AppConfig::default_template().replace(
        "always_confirm_submit = true",
        "always_confirm_submit = false",
    );

    let error = AppConfig::load_from_str(&invalid).expect_err("config should be invalid");

    match error {
        ConfigError::Validation(message) => {
            assert!(message.contains("safety.always_confirm_submit must remain true"));
        }
        other => panic!("expected validation error, got {other}"),
    }
}

#[test]
fn rejects_a_relative_base_url_for_a_remote_planner_profile() {
    let invalid = AppConfig::default_template().replacen(
        "base_url = \"https://api.openai.com/v1\"",
        "base_url = \"/v1\"",
        1,
    );

    let error = AppConfig::load_from_str(&invalid).expect_err("config should be invalid");

    match error {
        ConfigError::Validation(message) => {
            assert!(message.contains("remote_profiles.openai-default.base_url is invalid"));
        }
        other => panic!("expected validation error, got {other}"),
    }
}

#[test]
fn rejects_zero_timeout_ms_for_a_remote_planner_profile() {
    // `timeout_ms = 30000` appears on multiple remote profiles in the
    // template; `replacen(.., 1)` targets only the first (planner
    // openai-default), matching the profile name asserted below.
    let invalid = AppConfig::default_template().replacen("timeout_ms = 30000", "timeout_ms = 0", 1);

    let error = AppConfig::load_from_str(&invalid).expect_err("config should be invalid");

    match error {
        ConfigError::Validation(message) => {
            assert!(message.contains("remote_profiles.openai-default.timeout_ms must be positive"));
        }
        other => panic!("expected validation error, got {other}"),
    }
}

#[test]
fn rejects_zero_max_output_tokens_for_a_remote_planner_profile() {
    let invalid = AppConfig::default_template().replacen(
        "max_output_tokens = 1024",
        "max_output_tokens = 0",
        1,
    );

    let error = AppConfig::load_from_str(&invalid).expect_err("config should be invalid");

    match error {
        ConfigError::Validation(message) => {
            assert!(message
                .contains("remote_profiles.openai-default.max_output_tokens must be positive"));
        }
        other => panic!("expected validation error, got {other}"),
    }
}

#[test]
fn rejects_out_of_range_temperature_for_a_remote_planner_profile() {
    let invalid = AppConfig::default_template().replacen(
        "temperature_milli = 200",
        "temperature_milli = 2001",
        1,
    );

    let error = AppConfig::load_from_str(&invalid).expect_err("config should be invalid");

    match error {
        ConfigError::Validation(message) => {
            assert!(message.contains(
                "remote_profiles.openai-default.temperature_milli must be between 0 and 2000"
            ));
        }
        other => panic!("expected validation error, got {other}"),
    }
}

#[test]
fn rejects_empty_model_path_for_a_local_tts_profile() {
    let invalid = AppConfig::default_template().replace(
        "model_path = \"/path/to/kitten/model\"",
        "model_path = \"\"",
    );

    let error = AppConfig::load_from_str(&invalid).expect_err("config should be invalid");

    match error {
        ConfigError::Validation(message) => {
            assert!(message.contains("local_profiles.kitten-default.model_path must not be empty"));
        }
        other => panic!("expected validation error, got {other}"),
    }
}

#[test]
fn rejects_zero_sample_rate_for_a_local_tts_profile() {
    let invalid = AppConfig::default_template().replace("sample_rate = 24000", "sample_rate = 0");

    let error = AppConfig::load_from_str(&invalid).expect_err("config should be invalid");

    match error {
        ConfigError::Validation(message) => {
            assert!(message.contains("local_profiles.kitten-default.sample_rate must be positive"));
        }
        other => panic!("expected validation error, got {other}"),
    }
}

#[test]
fn rejects_empty_model_path_for_a_local_asr_profile() {
    let invalid = AppConfig::default_template().replace(
        "model_path = \"/path/to/whisper/model\"",
        "model_path = \"\"",
    );

    let error = AppConfig::load_from_str(&invalid).expect_err("config should be invalid");

    match error {
        ConfigError::Validation(message) => {
            assert!(message.contains("local_profiles.whisper-default.model_path must not be empty"));
        }
        other => panic!("expected validation error, got {other}"),
    }
}

#[test]
fn rejects_zero_threads_for_a_local_asr_profile() {
    let invalid = AppConfig::default_template().replace("threads = 4", "threads = 0");

    let error = AppConfig::load_from_str(&invalid).expect_err("config should be invalid");

    match error {
        ConfigError::Validation(message) => {
            assert!(message.contains("local_profiles.whisper-default.threads must be positive"));
        }
        other => panic!("expected validation error, got {other}"),
    }
}
