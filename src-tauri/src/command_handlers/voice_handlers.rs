use std::sync::Mutex;

use crate::app_core::AppCore;
use crate::commands::{
    StartListeningData, StartListeningInput, StopListeningData, StopListeningInput, ToolError,
    ToolResult, TranscribeAndExecuteCommandData, TranscribeCommandData, TranscribeCommandInput,
    TranscriptionStopMode,
};
use crate::lock_app_core;

#[tauri::command]
pub fn start_listening(
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
pub fn stop_listening(
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

#[tauri::command(async)]
pub fn transcribe_command(
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
            TranscriptionStopMode::AutoStop
        } else {
            TranscriptionStopMode::KeepListening
        },
    }))
}

#[tauri::command(async)]
pub fn transcribe_and_execute_command(
    request_id: String,
    timeout_ms: Option<u64>,
    max_duration_ms: Option<u64>,
    auto_stop: bool,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<TranscribeAndExecuteCommandData, ToolError> {
    let mut app_core = lock_app_core(&app_core)?;
    app_core.transcribe_and_execute_command(request_id, timeout_ms, max_duration_ms, auto_stop)
}
