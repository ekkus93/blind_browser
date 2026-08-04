use std::sync::{Arc, Mutex};

use crate::app_core::{AppCore, RemotePlannerConnectionSettingsData};
use crate::commands::ToolError;
use crate::config::ProviderMode;
use crate::lock_app_core;

#[derive(serde::Serialize)]
pub struct SetAsrProviderSelectionData {
    mode: ProviderMode,
    changed: bool,
}

#[derive(serde::Serialize)]
pub struct SetTtsProviderSelectionData {
    mode: ProviderMode,
    changed: bool,
}

#[derive(serde::Serialize)]
pub struct SetTtsModelSelectionData {
    profile_name: String,
    changed: bool,
}

fn completed_remote_planner_connection_settings(
    profile_name: String,
    settings: crate::commands::RemotePlannerSettings,
) -> Result<RemotePlannerConnectionSettingsData, ToolError> {
    let base_url = settings
        .base_url
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| ToolError {
            code: String::from("remote_planner_settings_inconsistent"),
            message: String::from(
                "Persisted remote planner settings did not produce a usable sanitized endpoint.",
            ),
            retryable: false,
            details: Some(
                serde_json::json!({ "availability_reason": settings.availability_reason }),
            ),
        })?;
    let model = settings
        .model
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| ToolError {
            code: String::from("remote_planner_settings_inconsistent"),
            message: String::from(
                "Persisted remote planner settings did not produce a configured model.",
            ),
            retryable: false,
            details: None,
        })?;
    Ok(RemotePlannerConnectionSettingsData {
        profile_name,
        base_url,
        model,
    })
}

#[tauri::command]
pub fn set_asr_provider_selection(
    request_id: String,
    timeout_ms: Option<u64>,
    mode: ProviderMode,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<SetAsrProviderSelectionData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let mut app_core = lock_app_core(&app_core)?;
    let changed = app_core.config.providers.asr.mode != mode;

    app_core
        .set_asr_provider_mode(mode.clone())
        .map_err(|error| ToolError {
            code: String::from("asr_provider_selection_persist_failed"),
            message: format!("Failed to persist the requested ASR provider selection: {error}"),
            retryable: false,
            details: None,
        })?;

    Ok(SetAsrProviderSelectionData { mode, changed })
}

#[tauri::command]
pub fn set_tts_provider_selection(
    request_id: String,
    timeout_ms: Option<u64>,
    mode: ProviderMode,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<SetTtsProviderSelectionData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let mut app_core = lock_app_core(&app_core)?;
    let changed = app_core.config.providers.tts.mode != mode;

    app_core
        .set_tts_provider_mode(mode.clone())
        .map_err(|error| ToolError {
            code: String::from("tts_provider_selection_persist_failed"),
            message: format!("Failed to persist the requested TTS provider selection: {error}"),
            retryable: false,
            details: None,
        })?;

    Ok(SetTtsProviderSelectionData { mode, changed })
}

#[tauri::command]
pub fn set_tts_model_selection(
    request_id: String,
    timeout_ms: Option<u64>,
    profile_name: String,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<SetTtsModelSelectionData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let mut app_core = lock_app_core(&app_core)?;
    let profile_name = profile_name.trim().to_string();
    if profile_name.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_tts_model_profile"),
            message: String::from(
                "TTS model selection requires a non-empty configured profile name.",
            ),
            retryable: false,
            details: None,
        });
    }

    let current_profile = match app_core.config.providers.tts.mode {
        ProviderMode::Local => app_core.config.providers.tts.local_profile.clone(),
        ProviderMode::Remote => app_core.config.providers.tts.remote_profile.clone(),
    };
    let changed = current_profile.as_deref() != Some(profile_name.as_str());

    app_core
        .set_active_tts_profile(profile_name.clone())
        .map_err(|error| ToolError {
            code: String::from("tts_model_selection_persist_failed"),
            message: format!("Failed to persist the requested TTS model selection: {error}"),
            retryable: false,
            details: None,
        })?;

    Ok(SetTtsModelSelectionData {
        profile_name,
        changed,
    })
}

#[tauri::command]
pub fn set_remote_planner_connection_settings(
    request_id: String,
    timeout_ms: Option<u64>,
    profile_name: String,
    base_url: String,
    model: String,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<RemotePlannerConnectionSettingsData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let mut app_core = lock_app_core(&app_core)?;
    let profile_name = profile_name.trim().to_string();
    let base_url = base_url.trim().to_string();
    let model = model.trim().to_string();
    if profile_name.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_remote_planner_profile"),
            message: String::from("Remote planner settings require a configured profile name."),
            retryable: false,
            details: None,
        });
    }

    app_core
        .set_remote_planner_connection_settings(&profile_name, &base_url, &model)
        .map_err(|error| ToolError {
            code: String::from("remote_planner_settings_persist_failed"),
            message: format!("Failed to persist the requested remote planner settings: {error}"),
            retryable: false,
            details: None,
        })?;

    let settings = app_core.current_remote_planner_settings();
    completed_remote_planner_connection_settings(profile_name, settings)
}

#[tauri::command]
pub fn reset_remote_planner_connection_settings(
    request_id: String,
    timeout_ms: Option<u64>,
    profile_name: String,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<RemotePlannerConnectionSettingsData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let mut app_core = lock_app_core(&app_core)?;
    let profile_name = profile_name.trim().to_string();
    if profile_name.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_remote_planner_profile"),
            message: String::from(
                "Remote planner settings reset requires a configured profile name.",
            ),
            retryable: false,
            details: None,
        });
    }

    app_core
        .reset_remote_planner_connection_settings_to_defaults(&profile_name)
        .map_err(|error| ToolError {
            code: String::from("remote_planner_settings_reset_failed"),
            message: format!("Failed to reset the remote planner settings: {error}"),
            retryable: false,
            details: None,
        })?;

    let settings = app_core.current_remote_planner_settings();
    completed_remote_planner_connection_settings(profile_name, settings)
}

#[cfg(test)]
mod post_p8_enforcement_tests {
    use super::*;
    use crate::commands::{CapabilityAbsenceReason, RemotePlannerSettings};

    #[test]
    fn inconsistent_post_persist_settings_are_typed_failures() {
        let error = completed_remote_planner_connection_settings(
            String::from("profile"),
            RemotePlannerSettings {
                profile_name: Some(String::from("profile")),
                availability_reason: Some(CapabilityAbsenceReason::InvalidEndpoint),
                ..RemotePlannerSettings::default()
            },
        )
        .expect_err("missing endpoint/model must fail");
        assert_eq!(error.code, "remote_planner_settings_inconsistent");
    }

    #[test]
    fn complete_post_persist_settings_are_returned() {
        let result = completed_remote_planner_connection_settings(
            String::from("profile"),
            RemotePlannerSettings {
                profile_name: Some(String::from("profile")),
                base_url: Some(String::from("https://example.com/v1")),
                model: Some(String::from("model")),
                ..RemotePlannerSettings::default()
            },
        )
        .expect("complete settings");
        assert_eq!(result.base_url, "https://example.com/v1");
        assert_eq!(result.model, "model");
    }
}
