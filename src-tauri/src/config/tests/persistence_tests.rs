use super::*;

#[test]
fn persist_model_management_settings_rejects_empty_models_dir() {
    let path = test_config_path("persist_model_management_empty_dir");
    let invalid_models = ModelManagementSettings {
        models_dir: String::from("   "),
        check_on_startup: true,
        auto_download_missing: false,
    };
    let error = AppConfig::persist_model_management_settings_at_path(&path, &invalid_models)
        .expect_err("empty models_dir should fail validation");
    match error {
        ConfigError::Validation(message) => {
            assert!(
                message.contains("models_dir must not be empty"),
                "unexpected error: {message}"
            );
        }
        other => panic!("expected validation error, got {other:?}"),
    }
}

#[test]
fn persist_model_management_settings_and_reloads_them() {
    let path = test_config_path("persist_model_management");
    let expected = ModelManagementSettings {
        models_dir: String::from("/data/models"),
        check_on_startup: false,
        auto_download_missing: true,
    };
    let persisted = AppConfig::persist_model_management_settings_at_path(&path, &expected)
        .expect("model management settings should persist successfully");
    let reloaded = AppConfig::load_from_path(&path).expect("persisted config should reload");
    assert_eq!(persisted.models.models_dir, "/data/models");
    assert!(!persisted.models.check_on_startup);
    assert!(persisted.models.auto_download_missing);
    assert_eq!(reloaded.models.models_dir, "/data/models");
    assert!(!reloaded.models.check_on_startup);
    assert!(reloaded.models.auto_download_missing);
    if let Some(parent) = path.parent() {
        let _ = std::fs::remove_dir_all(parent);
    }
}

#[test]
fn persist_remote_planner_connection_settings_rejects_unknown_profile() {
    let path = test_config_path("persist_planner_unknown_profile");
    let error = AppConfig::persist_remote_planner_connection_settings_at_path(
        &path,
        "nonexistent-profile",
        "https://api.example.com/v1",
        "gpt-test",
    )
    .expect_err("unknown profile should fail");
    match error {
        ConfigError::Validation(message) => {
            assert!(
                message.contains("nonexistent-profile"),
                "unexpected error: {message}"
            );
        }
        other => panic!("expected validation error, got {other:?}"),
    }
    if let Some(parent) = path.parent() {
        let _ = std::fs::remove_dir_all(parent);
    }
}

#[test]
fn persist_remote_planner_connection_settings_rejects_empty_profile_name() {
    let path = test_config_path("persist_planner_empty_profile");
    let error = AppConfig::persist_remote_planner_connection_settings_at_path(
        &path,
        "  ",
        "https://api.example.com/v1",
        "gpt-test",
    )
    .expect_err("empty profile name should fail");
    match error {
        ConfigError::Validation(message) => {
            assert!(
                message.contains("non-empty configured profile name"),
                "unexpected error: {message}"
            );
        }
        other => panic!("expected validation error, got {other:?}"),
    }
}

#[test]
fn persist_remote_planner_connection_settings_rejects_empty_model() {
    let path = test_config_path("persist_planner_empty_model");
    let error = AppConfig::persist_remote_planner_connection_settings_at_path(
        &path,
        "openai-default",
        "https://api.example.com/v1",
        "   ",
    )
    .expect_err("empty model should fail");
    match error {
        ConfigError::Validation(message) => {
            assert!(
                message.contains("non-empty model"),
                "unexpected error: {message}"
            );
        }
        other => panic!("expected validation error, got {other:?}"),
    }
    if let Some(parent) = path.parent() {
        let _ = std::fs::remove_dir_all(parent);
    }
}

#[test]
fn persist_remote_planner_connection_settings_rejects_invalid_url() {
    let path = test_config_path("persist_planner_invalid_url");
    let error = AppConfig::persist_remote_planner_connection_settings_at_path(
        &path,
        "openai-default",
        "not-a-url",
        "gpt-test",
    )
    .expect_err("invalid URL should fail");
    match error {
        ConfigError::Validation(message) => {
            assert!(
                message.contains("absolute URL"),
                "unexpected error: {message}"
            );
        }
        other => panic!("expected validation error, got {other:?}"),
    }
}

#[test]
fn persist_local_model_path_and_reloads_it() {
    let path = test_config_path("persist_local_model_path");
    let persisted = AppConfig::persist_local_model_path_at_path(
        &path,
        "kitten-default",
        "/data/models/kitten.onnx",
    )
    .expect("local model path should persist successfully");
    let reloaded = AppConfig::load_from_path(&path).expect("persisted config should reload");
    assert_eq!(
        persisted
            .local_tts_profiles
            .get("kitten-default")
            .expect("kitten-default profile should remain present")
            .model_path,
        "/data/models/kitten.onnx"
    );
    assert_eq!(
        reloaded
            .local_tts_profiles
            .get("kitten-default")
            .expect("kitten-default profile should remain present after reload")
            .model_path,
        "/data/models/kitten.onnx"
    );
    if let Some(parent) = path.parent() {
        let _ = std::fs::remove_dir_all(parent);
    }
}

#[test]
fn persist_local_model_path_rejects_unknown_profile() {
    let path = test_config_path("persist_local_model_path_unknown");
    let error = AppConfig::persist_local_model_path_at_path(
        &path,
        "nonexistent-local-profile",
        "/data/models/something.onnx",
    )
    .expect_err("unknown profile should fail");
    match error {
        ConfigError::Validation(message) => {
            assert!(
                message.contains("nonexistent-local-profile"),
                "unexpected error: {message}"
            );
        }
        other => panic!("expected validation error, got {other:?}"),
    }
    if let Some(parent) = path.parent() {
        let _ = std::fs::remove_dir_all(parent);
    }
}

#[test]
fn persist_audio_settings_rejects_out_of_range_volume() {
    let path = test_config_path("persist_audio_invalid_volume");
    let invalid_audio = AudioSettings {
        playback_volume: 1.5,
        playback_speed: 1.0,
        default_tts_voice: String::from("default"),
    };
    let error = AppConfig::persist_audio_settings_at_path(&path, &invalid_audio)
        .expect_err("out-of-range volume should fail validation");
    match error {
        ConfigError::Validation(message) => {
            assert!(
                message.contains("audio.playback_volume must be between"),
                "unexpected error: {message}"
            );
        }
        other => panic!("expected validation error, got {other:?}"),
    }
}
