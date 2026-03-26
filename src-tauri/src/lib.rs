use std::error::Error;
use std::sync::Mutex;

pub mod app_core;
pub mod asr;
pub mod audio_io;
pub mod browser;
pub mod commands;
pub mod config;
pub mod dom_inspector;
pub mod extractor;
pub mod logging;
pub mod narration;
pub mod ocr;
pub mod page_model;
pub mod state;
pub mod tts;

use tauri::Manager;

use crate::app_core::{AppCore, DownloadedLocalModelData, ModelManagementSettingsData};
use crate::browser::BrowserVisibilityMode;
use crate::commands::{
    AgentStateData, ConfirmActionResolution, ExecutionOutcome, GetAgentStateInput, OpenUrlData,
    OpenUrlInput, PlannerOutput, SetBrowserVisibilityData, SetBrowserVisibilityInput,
    SetPlaybackSpeedData, SetPlaybackSpeedInput, SetPlaybackVolumeData, SetPlaybackVolumeInput,
    SetTtsVoiceData, SetTtsVoiceInput, StartListeningData, StartListeningInput, StopListeningData,
    StopListeningInput, ToolError, ToolResult, TranscribeAndExecuteCommandData,
    TranscribeCommandData, TranscribeCommandInput, TtsVoiceName,
};
use crate::config::ProviderMode;

fn lock_app_core<'a>(
    app_core: &'a tauri::State<'a, Mutex<AppCore>>,
) -> Result<std::sync::MutexGuard<'a, AppCore>, ToolError> {
    app_core.lock().map_err(|_| ToolError {
        code: String::from("app_core_lock_failed"),
        message: String::from("failed to acquire the app runtime state lock"),
        retryable: true,
        details: None,
    })
}

#[tauri::command]
fn execute_planner_output(
    request_id: String,
    planner_output: PlannerOutput,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<ExecutionOutcome, ToolError> {
    let mut app_core = lock_app_core(&app_core)?;

    Ok(app_core.execute_planner_output(request_id, &planner_output))
}

#[tauri::command]
fn resolve_command(
    request_id: String,
    transcript: String,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<PlannerOutput, ToolError> {
    let mut app_core = lock_app_core(&app_core)?;

    app_core.resolve_command(request_id, transcript)
}

#[tauri::command]
fn submit_confirmation_response(
    confirmation_id: String,
    confirmed: bool,
    timed_out: bool,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<ConfirmActionResolution, ToolError> {
    let mut app_core = lock_app_core(&app_core)?;

    Ok(app_core.submit_confirmation_response(&confirmation_id, confirmed, timed_out))
}

#[tauri::command]
fn start_listening(
    request_id: String,
    timeout_ms: Option<u64>,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<ToolResult<StartListeningData>, ToolError> {
    let mut app_core = lock_app_core(&app_core)?;

    Ok(app_core.execute_start_listening(StartListeningInput {
        request_id,
        timeout_ms,
    }))
}

#[tauri::command]
fn stop_listening(
    request_id: String,
    timeout_ms: Option<u64>,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<ToolResult<StopListeningData>, ToolError> {
    let mut app_core = lock_app_core(&app_core)?;

    Ok(app_core.execute_stop_listening(StopListeningInput {
        request_id,
        timeout_ms,
    }))
}

#[tauri::command]
fn transcribe_command(
    request_id: String,
    timeout_ms: Option<u64>,
    max_duration_ms: Option<u64>,
    auto_stop: bool,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<ToolResult<TranscribeCommandData>, ToolError> {
    let mut app_core = lock_app_core(&app_core)?;

    Ok(app_core.execute_transcribe_command(TranscribeCommandInput {
        request_id,
        timeout_ms,
        max_duration_ms,
        stop_mode: if auto_stop {
            crate::commands::TranscriptionStopMode::AutoStop
        } else {
            crate::commands::TranscriptionStopMode::KeepListening
        },
    }))
}

#[tauri::command]
fn transcribe_and_execute_command(
    request_id: String,
    timeout_ms: Option<u64>,
    max_duration_ms: Option<u64>,
    auto_stop: bool,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<TranscribeAndExecuteCommandData, ToolError> {
    let mut app_core = lock_app_core(&app_core)?;

    app_core.transcribe_and_execute_command(request_id, timeout_ms, max_duration_ms, auto_stop)
}

#[tauri::command]
fn open_url(
    request_id: String,
    timeout_ms: Option<u64>,
    url: String,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<ToolResult<OpenUrlData>, ToolError> {
    let mut app_core = lock_app_core(&app_core)?;

    Ok(app_core.execute_open_url(OpenUrlInput {
        request_id,
        timeout_ms,
        url,
        wait_for_load_state: None,
    }))
}

#[tauri::command]
fn get_agent_state(
    request_id: String,
    timeout_ms: Option<u64>,
    include_last_transcript: bool,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<ToolResult<AgentStateData>, ToolError> {
    let mut app_core = lock_app_core(&app_core)?;

    Ok(app_core.execute_get_agent_state(GetAgentStateInput {
        request_id,
        timeout_ms,
        include_last_transcript,
    }))
}

#[tauri::command]
fn set_playback_volume(
    request_id: String,
    timeout_ms: Option<u64>,
    volume: f32,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<ToolResult<SetPlaybackVolumeData>, ToolError> {
    let mut app_core = lock_app_core(&app_core)?;

    Ok(
        app_core.execute_set_playback_volume(SetPlaybackVolumeInput {
            request_id,
            timeout_ms,
            volume,
        }),
    )
}

#[tauri::command]
fn set_playback_speed(
    request_id: String,
    timeout_ms: Option<u64>,
    speed: f32,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<ToolResult<SetPlaybackSpeedData>, ToolError> {
    let mut app_core = lock_app_core(&app_core)?;

    Ok(app_core.execute_set_playback_speed(SetPlaybackSpeedInput {
        request_id,
        timeout_ms,
        speed,
    }))
}

#[tauri::command]
fn set_browser_visibility(
    request_id: String,
    timeout_ms: Option<u64>,
    mode: BrowserVisibilityMode,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<ToolResult<SetBrowserVisibilityData>, ToolError> {
    let mut app_core = lock_app_core(&app_core)?;

    Ok(
        app_core.execute_set_browser_visibility(SetBrowserVisibilityInput {
            request_id,
            timeout_ms,
            mode,
        }),
    )
}

#[tauri::command]
fn set_tts_voice(
    request_id: String,
    timeout_ms: Option<u64>,
    voice: TtsVoiceName,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<ToolResult<SetTtsVoiceData>, ToolError> {
    let mut app_core = lock_app_core(&app_core)?;

    Ok(app_core.execute_set_tts_voice(SetTtsVoiceInput {
        request_id,
        timeout_ms,
        voice,
    }))
}

#[derive(serde::Serialize)]
struct SetAsrProviderSelectionData {
    mode: ProviderMode,
    changed: bool,
}

#[derive(serde::Serialize)]
struct SetModelManagementSettingsData {
    models_dir: String,
    check_on_startup: bool,
    auto_download_missing: bool,
}

#[tauri::command]
fn set_asr_provider_selection(
    request_id: String,
    timeout_ms: Option<u64>,
    mode: ProviderMode,
    app_core: tauri::State<'_, Mutex<AppCore>>,
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

#[derive(serde::Serialize)]
struct SetTtsProviderSelectionData {
    mode: ProviderMode,
    changed: bool,
}

#[tauri::command]
fn set_tts_provider_selection(
    request_id: String,
    timeout_ms: Option<u64>,
    mode: ProviderMode,
    app_core: tauri::State<'_, Mutex<AppCore>>,
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

#[derive(serde::Serialize)]
struct SetTtsModelSelectionData {
    profile_name: String,
    changed: bool,
}

#[derive(serde::Serialize)]
struct SetConfirmationThresholdData {
    confirmation_confidence_threshold: f32,
    changed: bool,
}

#[tauri::command]
fn set_confirmation_threshold(
    request_id: String,
    timeout_ms: Option<u64>,
    confirmation_confidence_threshold: f32,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<SetConfirmationThresholdData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let mut app_core = lock_app_core(&app_core)?;
    let changed = app_core.config.safety.confirmation_confidence_threshold
        != confirmation_confidence_threshold;

    app_core
        .set_confirmation_confidence_threshold(confirmation_confidence_threshold)
        .map_err(|error| ToolError {
            code: String::from("confirmation_threshold_persist_failed"),
            message: format!("Failed to persist the requested confirmation threshold: {error}"),
            retryable: false,
            details: None,
        })?;

    Ok(SetConfirmationThresholdData {
        confirmation_confidence_threshold,
        changed,
    })
}

#[derive(serde::Serialize)]
struct SetAllowClickWithoutConfirmationData {
    allow_click_without_confirmation: bool,
    changed: bool,
}

#[derive(serde::Serialize)]
struct SetOcrThresholdsData {
    sparse_text_char_threshold: u32,
    sparse_text_region_threshold: u32,
    changed: bool,
}

#[derive(serde::Serialize)]
struct SetRemoteApiKeyData {
    profile_name: String,
    api_key_reference: String,
}

#[tauri::command]
fn set_allow_click_without_confirmation(
    request_id: String,
    timeout_ms: Option<u64>,
    allow_click_without_confirmation: bool,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<SetAllowClickWithoutConfirmationData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let mut app_core = lock_app_core(&app_core)?;
    let changed =
        app_core.config.safety.allow_click_without_confirmation != allow_click_without_confirmation;

    app_core
        .set_allow_click_without_confirmation(allow_click_without_confirmation)
        .map_err(|error| ToolError {
            code: String::from("allow_click_without_confirmation_persist_failed"),
            message: format!(
                "Failed to persist the requested click-without-confirmation setting: {error}"
            ),
            retryable: false,
            details: None,
        })?;

    Ok(SetAllowClickWithoutConfirmationData {
        allow_click_without_confirmation,
        changed,
    })
}

#[tauri::command]
fn set_ocr_thresholds(
    request_id: String,
    timeout_ms: Option<u64>,
    sparse_text_char_threshold: u32,
    sparse_text_region_threshold: u32,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<SetOcrThresholdsData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let mut app_core = lock_app_core(&app_core)?;
    let changed = app_core.config.ocr.sparse_text_char_threshold != sparse_text_char_threshold
        || app_core.config.ocr.sparse_text_region_threshold != sparse_text_region_threshold;

    app_core
        .set_ocr_thresholds(sparse_text_char_threshold, sparse_text_region_threshold)
        .map_err(|error| ToolError {
            code: String::from("ocr_thresholds_persist_failed"),
            message: format!("Failed to persist the requested OCR thresholds: {error}"),
            retryable: false,
            details: None,
        })?;

    Ok(SetOcrThresholdsData {
        sparse_text_char_threshold,
        sparse_text_region_threshold,
        changed,
    })
}

#[tauri::command]
fn set_remote_planner_api_key(
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

#[tauri::command]
fn set_remote_tts_api_key(
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

#[tauri::command]
fn set_remote_asr_api_key(
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

#[tauri::command]
fn get_model_management_settings(
    request_id: String,
    timeout_ms: Option<u64>,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<ModelManagementSettingsData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let app_core = lock_app_core(&app_core)?;
    Ok(app_core.current_model_management_settings())
}

#[tauri::command]
fn set_model_management_settings(
    request_id: String,
    timeout_ms: Option<u64>,
    models_dir: String,
    check_on_startup: bool,
    auto_download_missing: bool,
    app_core: tauri::State<'_, Mutex<AppCore>>,
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
fn download_active_local_tts_model(
    request_id: String,
    timeout_ms: Option<u64>,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<DownloadedLocalModelData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let mut app_core = lock_app_core(&app_core)?;
    app_core
        .download_active_local_tts_model()
        .map_err(|message| ToolError {
            code: String::from("local_tts_model_download_failed"),
            message,
            retryable: false,
            details: None,
        })
}

#[tauri::command]
fn download_active_local_asr_model(
    request_id: String,
    timeout_ms: Option<u64>,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<DownloadedLocalModelData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let mut app_core = lock_app_core(&app_core)?;
    app_core
        .download_active_local_asr_model()
        .map_err(|message| ToolError {
            code: String::from("local_asr_model_download_failed"),
            message,
            retryable: false,
            details: None,
        })
}

#[tauri::command]
fn set_tts_model_selection(
    request_id: String,
    timeout_ms: Option<u64>,
    profile_name: String,
    app_core: tauri::State<'_, Mutex<AppCore>>,
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
        crate::config::ProviderMode::Local => app_core.config.providers.tts.local_profile.clone(),
        crate::config::ProviderMode::Remote => app_core.config.providers.tts.remote_profile.clone(),
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

pub fn run() {
    logging::init_logging();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            resolve_command,
            execute_planner_output,
            submit_confirmation_response,
            start_listening,
            stop_listening,
            transcribe_command,
            transcribe_and_execute_command,
            open_url,
            get_agent_state,
            set_playback_volume,
            set_playback_speed,
            set_browser_visibility,
            set_tts_voice,
            set_confirmation_threshold,
            set_allow_click_without_confirmation,
            set_ocr_thresholds,
            set_remote_planner_api_key,
            set_remote_tts_api_key,
            set_remote_asr_api_key,
            get_model_management_settings,
            set_model_management_settings,
            download_active_local_tts_model,
            download_active_local_asr_model,
            set_tts_provider_selection,
            set_asr_provider_selection,
            set_tts_model_selection
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            let app_core = app_core::AppCore::new(app_handle)
                .map_err(|error| -> Box<dyn Error> { Box::new(error) })?;
            app.manage(Mutex::new(app_core));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run blind_browser application");
}
