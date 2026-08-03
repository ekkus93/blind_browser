use std::sync::{Arc, Mutex};

use crate::app_core::{AppCore, DownloadedLocalModelData, ModelManagementSettingsData};
use crate::commands::ToolError;
use crate::{join_error_to_tool_error, lock_app_core};

#[derive(serde::Serialize)]
pub struct SetModelManagementSettingsData {
    models_dir: String,
    check_on_startup: bool,
    auto_download_missing: bool,
}

#[tauri::command]
pub fn get_model_management_settings(
    request_id: String,
    timeout_ms: Option<u64>,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<ModelManagementSettingsData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let app_core = lock_app_core(&app_core)?;
    Ok(app_core.current_model_management_settings())
}

#[tauri::command]
pub fn set_model_management_settings(
    request_id: String,
    timeout_ms: Option<u64>,
    models_dir: String,
    check_on_startup: bool,
    auto_download_missing: bool,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<SetModelManagementSettingsData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let mut app_core = lock_app_core(&app_core)?;
    let models_dir = models_dir.trim().to_string();
    if models_dir.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_models_dir"),
            message: String::from(
                "Model management settings require a non-empty models directory.",
            ),
            retryable: false,
            details: None,
        });
    }

    app_core
        .set_model_management_settings(&models_dir, check_on_startup, auto_download_missing)
        .map_err(|error| ToolError {
            code: String::from("model_management_settings_persist_failed"),
            message: format!("Failed to persist the requested model management settings: {error}"),
            retryable: false,
            details: None,
        })?;

    Ok(SetModelManagementSettingsData {
        models_dir,
        check_on_startup,
        auto_download_missing,
    })
}

#[tauri::command]
pub async fn download_active_local_tts_model(
    request_id: String,
    timeout_ms: Option<u64>,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<DownloadedLocalModelData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let core = Arc::clone(&app_core);
    tauri::async_runtime::spawn_blocking(move || {
        let mut guard = lock_app_core(&core)?;
        let download_result = guard.download_active_local_tts_model();
        download_result.map_err(|message| ToolError {
            code: String::from("local_tts_model_download_failed"),
            message,
            retryable: false,
            details: None,
        })
    })
    .await
    .map_err(join_error_to_tool_error)?
}

#[tauri::command]
pub async fn download_active_local_asr_model(
    request_id: String,
    timeout_ms: Option<u64>,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<DownloadedLocalModelData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let core = Arc::clone(&app_core);
    tauri::async_runtime::spawn_blocking(move || {
        let mut guard = lock_app_core(&core)?;
        let download_result = guard.download_active_local_asr_model();
        download_result.map_err(|message| ToolError {
            code: String::from("local_asr_model_download_failed"),
            message,
            retryable: false,
            details: None,
        })
    })
    .await
    .map_err(join_error_to_tool_error)?
}
