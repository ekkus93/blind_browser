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
    StartListeningData, StartListeningInput, StopListeningData, StopListeningInput, ToolError,
    ToolResult, TranscribeAndExecuteCommandData, TranscribeCommandData, TranscribeCommandInput,
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
            set_browser_visibility
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
