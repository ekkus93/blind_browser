use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::app_core::{microphone_consent_required_error, AppCore, TranscribeDrainOutcome};
use crate::commands::{
    StartListeningData, StartListeningInput, StopListeningData, StopListeningInput, ToolError,
    ToolName, ToolResult, TranscribeAndExecuteCommandData, TranscribeCommandData,
    TranscribeCommandInput, TranscriptionStopMode,
};
use crate::{join_error_to_tool_error, lock_app_core};

/// Run a transcription with the `AppCore` lock released across both blocking
/// windows — the audio capture *and* the (possibly remote) ASR transcription — so
/// `stop_listening` / `get_agent_state` can interleave throughout. Runs inside
/// `spawn_blocking`. Five phases:
///
/// 1. lock → `begin_transcribe_command` (start capture) → unlock
/// 2. unlocked capture sleep (the cpal stream keeps filling its buffer)
/// 3. lock → `drain_transcribe_command` (take audio + snapshot config) → unlock
/// 4. unlocked `transcribe_captured_audio` (the ASR round-trip, network for remote)
/// 5. lock → `record_transcribe_command` (record transcript + build result)
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

    let (pending, remote_authorization) = {
        let mut guard = lock_app_core(core)?;
        let pending = match guard.drain_transcribe_command(plan) {
            TranscribeDrainOutcome::Terminal(result) => return Ok(*result),
            TranscribeDrainOutcome::Pending(pending) => pending,
        };
        let authorization = match guard.prepare_microphone_transcription(pending.input())? {
            crate::app_core::remote_data_consent::MicrophonePreparation::Authorized(
                authorization,
            ) => authorization,
            crate::app_core::remote_data_consent::MicrophonePreparation::ConsentRequired {
                challenge,
            } => {
                return Ok(ToolResult::failure(
                    ToolName::TranscribeCommand,
                    pending.input().request_id.clone(),
                    microphone_consent_required_error(&challenge),
                    vec![String::from(
                        "Remote transcription paused before microphone audio was sent; captured audio was discarded.",
                    )],
                ));
            }
        };
        (pending, authorization)
    };

    // Unlocked transcription window: the ASR backend (CPU for local whisper, a
    // network round-trip for remote) runs with the guard dropped, against the
    // drained audio + config snapshot the pending capture carries. A remote
    // provider cannot dispatch without the unforgeable authorization token
    // returned by the shared privacy gate above.
    let (config, captured_audio) = pending.transcription_inputs();
    let transcript_result = crate::asr::transcribe_captured_audio(
        config,
        captured_audio,
        remote_authorization.as_ref(),
    );

    let mut guard = lock_app_core(core)?;
    Ok(guard.record_transcribe_command(*pending, transcript_result))
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

// Runs in `spawn_blocking` so the browser tools reached during plan execution can
// call `tauri::async_runtime::block_on` safely (blocking-pool threads are not
// driving the async scheduler). The capture window releases the lock via
// `run_phased_transcribe`, and the resolve/execute loop releases it across each
// remote planner round-trip via `run_command_with_lock_scoped_replanning`.
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
                match crate::app_core::run_command_with_lock_scoped_replanning(
                    &core,
                    &request_id,
                    &transcript,
                ) {
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
