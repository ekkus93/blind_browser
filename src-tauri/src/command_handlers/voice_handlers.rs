use std::sync::{Arc, Mutex};

use crate::app_core::AppCore;
use crate::commands::{
    StartListeningData, StartListeningInput, StopListeningData, StopListeningInput, ToolError,
    ToolResult, TranscribeAndExecuteCommandData, TranscribeCommandData, TranscribeCommandInput,
    TranscriptionStopMode,
};
use crate::{join_error_to_tool_error, lock_app_core};

#[tauri::command]
pub async fn start_listening(
    request_id: String,
    timeout_ms: Option<u64>,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<ToolResult<StartListeningData>, ToolError> {
    let core = Arc::clone(&app_core);
    tauri::async_runtime::spawn_blocking(move || {
        let mut guard = lock_app_core(&core)?;
        Ok(guard.execute_start_listening(StartListeningInput {
            request_id,
            timeout_ms,
        }))
    })
    .await
    .map_err(join_error_to_tool_error)?
}

#[tauri::command]
pub async fn stop_listening(
    request_id: String,
    timeout_ms: Option<u64>,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<ToolResult<StopListeningData>, ToolError> {
    let core = Arc::clone(&app_core);
    tauri::async_runtime::spawn_blocking(move || {
        let mut guard = lock_app_core(&core)?;
        Ok(guard.execute_stop_listening(StopListeningInput {
            request_id,
            timeout_ms,
        }))
    })
    .await
    .map_err(join_error_to_tool_error)?
}

#[tauri::command]
pub async fn transcribe_command(
    request_id: String,
    timeout_ms: Option<u64>,
    max_duration_ms: Option<u64>,
    auto_stop: bool,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<ToolResult<TranscribeCommandData>, ToolError> {
    let core = Arc::clone(&app_core);
    tauri::async_runtime::spawn_blocking(move || {
        let mut guard = lock_app_core(&core)?;
        Ok(guard.execute_transcribe_command(TranscribeCommandInput {
            request_id,
            timeout_ms,
            max_duration_ms,
            stop_mode: if auto_stop {
                TranscriptionStopMode::AutoStop
            } else {
                TranscriptionStopMode::KeepListening
            },
        }))
    })
    .await
    .map_err(join_error_to_tool_error)?
}

// Runs in `spawn_blocking` so the browser tools reached via
// `execute_command_with_replanning` can call `tauri::async_runtime::block_on`
// safely (blocking-pool threads are not driving the async scheduler).
#[tauri::command]
pub async fn transcribe_and_execute_command(
    request_id: String,
    timeout_ms: Option<u64>,
    max_duration_ms: Option<u64>,
    auto_stop: bool,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<TranscribeAndExecuteCommandData, ToolError> {
    let core = Arc::clone(&app_core);
    tauri::async_runtime::spawn_blocking(move || {
        let mut guard = lock_app_core(&core)?;
        guard.transcribe_and_execute_command(request_id, timeout_ms, max_duration_ms, auto_stop)
    })
    .await
    .map_err(join_error_to_tool_error)?
}
