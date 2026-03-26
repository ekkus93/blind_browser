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

use crate::app_core::AppCore;
use crate::commands::{
    AgentStateData, ConfirmActionResolution, ExecutionOutcome, GetAgentStateInput, OpenUrlData,
    OpenUrlInput, PlannerOutput, SetBrowserVisibilityData, SetBrowserVisibilityInput,
    SetPlaybackSpeedData, SetPlaybackSpeedInput, SetPlaybackVolumeData, SetPlaybackVolumeInput,
    SetTtsVoiceData, SetTtsVoiceInput, StartListeningData, StartListeningInput,
    StopListeningData, StopListeningInput, ToolError, ToolResult,
    TranscribeAndExecuteCommandData, TranscribeCommandData, TranscribeCommandInput,
};
use crate::browser::BrowserVisibilityMode;

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
        auto_stop,
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

    Ok(app_core.execute_set_playback_volume(SetPlaybackVolumeInput {
        request_id,
        timeout_ms,
        volume,
    }))
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

    Ok(app_core.execute_set_browser_visibility(SetBrowserVisibilityInput {
        request_id,
        timeout_ms,
        mode,
    }))
}

#[tauri::command]
fn set_tts_voice(
    request_id: String,
    timeout_ms: Option<u64>,
    voice: String,
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
struct SetTtsModelSelectionData {
    profile_name: String,
    changed: bool,
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
            message: String::from("TTS model selection requires a non-empty configured profile name."),
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
