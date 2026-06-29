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

// GUARDRAIL: Keep this a plain `#[tauri::command]` (main-thread), NOT
// `#[tauri::command(async)]`. It reaches browser tools via
// `execute_command_with_replanning`, and those tools call
// `tauri::async_runtime::block_on`, which panics ("Cannot start a runtime from
// within a runtime") when invoked from a tokio worker thread. Until browser ops
// stop calling `block_on` from a worker (see BB_CODE_REVIEW2_TODO.md P1.1.2 /
// P1.1.4), converting this to `(async)` reintroduces that crash.
#[tauri::command]
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
