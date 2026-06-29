use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::app_core::AppCore;
use crate::commands::{
    StartListeningData, StartListeningInput, StopListeningData, StopListeningInput, ToolError,
    ToolName, ToolResult, TranscribeAndExecuteCommandData, TranscribeCommandData,
    TranscribeCommandInput, TranscriptionStopMode,
};
use crate::{join_error_to_tool_error, lock_app_core};

/// Run a transcription with the `AppCore` lock released across the audio-capture
/// window: lock briefly to start capture, drop the guard during the blocking
/// capture sleep (so `stop_listening` / `get_agent_state` can interleave), then
/// re-lock to drain and transcribe. Runs inside `spawn_blocking`.
fn run_phased_transcribe(
    core: &Arc<Mutex<AppCore>>,
    input: TranscribeCommandInput,
) -> Result<ToolResult<TranscribeCommandData>, ToolError> {
    let plan = {
        let mut guard = lock_app_core(core)?;
        guard.begin_transcribe_command(&input)
    };
    let plan = match plan {
        Ok(plan) => plan,
        Err(rejected) => return Ok(*rejected),
    };

    // Unlocked capture window: the cpal stream keeps filling its buffer while the
    // guard is dropped, so other commands can run against the same AppCore.
    thread::sleep(Duration::from_millis(plan.effective_duration_ms));

    let mut guard = lock_app_core(core)?;
    Ok(guard.finish_transcribe_command(plan))
}

fn transcription_stop_mode(auto_stop: bool) -> TranscriptionStopMode {
    if auto_stop {
        TranscriptionStopMode::AutoStop
    } else {
        TranscriptionStopMode::KeepListening
    }
}

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
        run_phased_transcribe(
            &core,
            TranscribeCommandInput {
                request_id,
                timeout_ms,
                max_duration_ms,
                stop_mode: transcription_stop_mode(auto_stop),
            },
        )
    })
    .await
    .map_err(join_error_to_tool_error)?
}

// Runs in `spawn_blocking` so the browser tools reached via
// `execute_transcribed_command` can call `tauri::async_runtime::block_on` safely
// (blocking-pool threads are not driving the async scheduler). The capture window
// itself releases the lock via `run_phased_transcribe`.
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
        let transcription_result = run_phased_transcribe(
            &core,
            TranscribeCommandInput {
                request_id: request_id.clone(),
                timeout_ms,
                max_duration_ms,
                stop_mode: transcription_stop_mode(auto_stop),
            },
        )?;

        let Some(transcription) = transcription_result.data else {
            return Err(transcription_result.error.unwrap_or(ToolError {
                code: String::from("missing_transcription_result"),
                message: String::from("transcribe_command did not return transcription data"),
                retryable: false,
                details: Some(serde_json::json!({
                    "request_id": request_id,
                    "tool_name": ToolName::TranscribeCommand,
                })),
            }));
        };

        let (command_error, execution_outcome) =
            if let Some(transcript) = transcription.transcript.clone() {
                let mut guard = lock_app_core(&core)?;
                match guard.execute_transcribed_command(request_id.clone(), transcript) {
                    Ok(outcome) => (None, Some(outcome)),
                    Err(error) => (Some(error), None),
                }
            } else {
                (None, None)
            };

        Ok(TranscribeAndExecuteCommandData {
            transcription,
            command_error,
            execution_outcome,
        })
    })
    .await
    .map_err(join_error_to_tool_error)?
}
