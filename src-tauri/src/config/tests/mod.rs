use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    keyring_ref_for_remote_api_key, resolve_secret_ref, secret_ref_reference,
    AppConfig, AudioSettings, ConfigError, KeyringRef, LocalAsrBackend,
    LocalAsrProfile, LocalTtsBackend, LocalTtsProfile, ModelManagementSettings,
    ProviderMode, ProviderSelection, ProviderSelections, RemoteAsrProfile, RemotePlannerProfile,
    RemoteProviderKind, RemoteTtsAudioFormat, RemoteTtsProfile, SafetySettings, SecretRef,
    SpeechFeedbackStyle,
};
use super::keyring_store::set_keyring_secret;
use crate::ocr::OcrSettings;

mod load_tests;
mod keyring_tests;
mod persistence_tests;
mod validation_tests;

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
