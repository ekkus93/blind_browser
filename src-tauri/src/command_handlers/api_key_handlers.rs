use std::sync::Mutex;

use crate::app_core::{AppCore, RemotePlannerModelListData};
use crate::commands::ToolError;
use crate::lock_app_core;

#[derive(serde::Serialize)]
pub struct SetRemoteApiKeyData {
    profile_name: String,
    api_key_reference: String,
}

#[derive(serde::Serialize)]
pub struct TestRemoteApiKeyData {
    profile_name: String,
    message: String,
}

#[tauri::command]
pub fn set_remote_planner_api_key(
    request_id: String,
    timeout_ms: Option<u64>,
    profile_name: String,
    api_key: String,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<SetRemoteApiKeyData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let mut app_core = lock_app_core(&app_core)?;
    let profile_name = profile_name.trim().to_string();
    let api_key = api_key.trim().to_string();
    if profile_name.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_remote_planner_profile"),
            message: String::from(
                "Remote planner API key entry requires a configured profile name.",
            ),
            retryable: false,
            details: None,
        });
    }
    if api_key.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_remote_planner_api_key"),
            message: String::from("Remote planner API key entry requires a non-empty API key."),
            retryable: false,
            details: None,
        });
    }

    app_core
        .set_remote_planner_api_key(&profile_name, &api_key)
        .map_err(|error| ToolError {
            code: String::from("remote_planner_api_key_persist_failed"),
            message: format!("Failed to persist the requested remote planner API key: {error}"),
            retryable: false,
            details: None,
        })?;

    Ok(SetRemoteApiKeyData {
        profile_name: profile_name.clone(),
        api_key_reference: app_core
            .current_remote_planner_settings()
            .api_key_reference
            .unwrap_or_default(),
    })
}

#[tauri::command(async)]
pub fn test_remote_planner_api_key(
    request_id: String,
    timeout_ms: Option<u64>,
    profile_name: String,
    api_key: String,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<TestRemoteApiKeyData, ToolError> {
    let _ = request_id;
    let app_core = lock_app_core(&app_core)?;
    let profile_name = profile_name.trim().to_string();
    if profile_name.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_remote_planner_profile"),
            message: String::from(
                "Remote planner API key test requires a configured profile name.",
            ),
            retryable: false,
            details: None,
        });
    }

    let api_key = api_key.trim().to_string();
    let message = app_core
        .test_remote_planner_api_key(
            &profile_name,
            (!api_key.is_empty()).then_some(api_key.as_str()),
            timeout_ms,
        )
        .map_err(|error| ToolError {
            code: String::from("remote_planner_api_key_test_failed"),
            message: error,
            retryable: false,
            details: None,
        })?;

    Ok(TestRemoteApiKeyData {
        profile_name,
        message,
    })
}

#[tauri::command(async)]
pub fn list_remote_planner_models(
    request_id: String,
    timeout_ms: Option<u64>,
    profile_name: String,
    base_url: String,
    api_key: String,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<RemotePlannerModelListData, ToolError> {
    let _ = request_id;
    let app_core = lock_app_core(&app_core)?;
    let profile_name = profile_name.trim().to_string();
    let base_url = base_url.trim().to_string();
    if profile_name.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_remote_planner_profile"),
            message: String::from(
                "Remote planner model loading requires a configured profile name.",
            ),
            retryable: false,
            details: None,
        });
    }
    if base_url.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_remote_planner_endpoint"),
            message: String::from("Remote planner model loading requires a non-empty endpoint."),
            retryable: false,
            details: None,
        });
    }

    let models = app_core
        .list_remote_planner_models(
            &profile_name,
            Some(&base_url),
            (!api_key.trim().is_empty()).then_some(api_key.as_str()),
            timeout_ms,
        )
        .map_err(|error| ToolError {
            code: String::from("remote_planner_models_load_failed"),
            message: error,
            retryable: false,
            details: None,
        })?;

    Ok(RemotePlannerModelListData {
        profile_name,
        base_url,
        models,
    })
}

#[tauri::command]
pub fn set_remote_tts_api_key(
    request_id: String,
    timeout_ms: Option<u64>,
    profile_name: String,
    api_key: String,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<SetRemoteApiKeyData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let mut app_core = lock_app_core(&app_core)?;
    let profile_name = profile_name.trim().to_string();
    let api_key = api_key.trim().to_string();
    if profile_name.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_remote_tts_profile"),
            message: String::from("Remote TTS API key entry requires a configured profile name."),
            retryable: false,
            details: None,
        });
    }
    if api_key.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_remote_tts_api_key"),
            message: String::from("Remote TTS API key entry requires a non-empty API key."),
            retryable: false,
            details: None,
        });
    }

    app_core
        .set_remote_tts_api_key(&profile_name, &api_key)
        .map_err(|error| ToolError {
            code: String::from("remote_tts_api_key_persist_failed"),
            message: format!("Failed to persist the requested remote TTS API key: {error}"),
            retryable: false,
            details: None,
        })?;

    Ok(SetRemoteApiKeyData {
        profile_name: profile_name.clone(),
        api_key_reference: app_core
            .current_remote_tts_settings()
            .api_key_reference
            .unwrap_or_default(),
    })
}

#[tauri::command(async)]
pub fn test_remote_tts_api_key(
    request_id: String,
    timeout_ms: Option<u64>,
    profile_name: String,
    api_key: String,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<TestRemoteApiKeyData, ToolError> {
    let _ = request_id;
    let app_core = lock_app_core(&app_core)?;
    let profile_name = profile_name.trim().to_string();
    if profile_name.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_remote_tts_profile"),
            message: String::from("Remote TTS API key test requires a configured profile name."),
            retryable: false,
            details: None,
        });
    }

    let api_key = api_key.trim().to_string();
    let message = app_core
        .test_remote_tts_api_key(
            &profile_name,
            (!api_key.is_empty()).then_some(api_key.as_str()),
            timeout_ms,
        )
        .map_err(|error| ToolError {
            code: String::from("remote_tts_api_key_test_failed"),
            message: error,
            retryable: false,
            details: None,
        })?;

    Ok(TestRemoteApiKeyData {
        profile_name,
        message,
    })
}

#[tauri::command]
pub fn set_remote_asr_api_key(
    request_id: String,
    timeout_ms: Option<u64>,
    profile_name: String,
    api_key: String,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<SetRemoteApiKeyData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let mut app_core = lock_app_core(&app_core)?;
    let profile_name = profile_name.trim().to_string();
    let api_key = api_key.trim().to_string();
    if profile_name.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_remote_asr_profile"),
            message: String::from("Remote ASR API key entry requires a configured profile name."),
            retryable: false,
            details: None,
        });
    }
    if api_key.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_remote_asr_api_key"),
            message: String::from("Remote ASR API key entry requires a non-empty API key."),
            retryable: false,
            details: None,
        });
    }

    app_core
        .set_remote_asr_api_key(&profile_name, &api_key)
        .map_err(|error| ToolError {
            code: String::from("remote_asr_api_key_persist_failed"),
            message: format!("Failed to persist the requested remote ASR API key: {error}"),
            retryable: false,
            details: None,
        })?;

    Ok(SetRemoteApiKeyData {
        profile_name: profile_name.clone(),
        api_key_reference: app_core
            .current_remote_asr_settings()
            .api_key_reference
            .unwrap_or_default(),
    })
}

#[tauri::command(async)]
pub fn test_remote_asr_api_key(
    request_id: String,
    timeout_ms: Option<u64>,
    profile_name: String,
    api_key: String,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<TestRemoteApiKeyData, ToolError> {
    let _ = request_id;
    let app_core = lock_app_core(&app_core)?;
    let profile_name = profile_name.trim().to_string();
    if profile_name.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_remote_asr_profile"),
            message: String::from("Remote ASR API key test requires a configured profile name."),
            retryable: false,
            details: None,
        });
    }

    let api_key = api_key.trim().to_string();
    let message = app_core
        .test_remote_asr_api_key(
            &profile_name,
            (!api_key.is_empty()).then_some(api_key.as_str()),
            timeout_ms,
        )
        .map_err(|error| ToolError {
            code: String::from("remote_asr_api_key_test_failed"),
            message: error,
            retryable: false,
            details: None,
        })?;

    Ok(TestRemoteApiKeyData {
        profile_name,
        message,
    })
}
