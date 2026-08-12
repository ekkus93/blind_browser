use std::sync::{Arc, Mutex};

use crate::app_core::AppCore;
use crate::commands::{
    AgentStateData, ConfirmActionResolution, ExecutionOutcome, GetAgentStateInput,
    MicrophoneConsentResponseOutcome, NarrationConsentResponseOutcome, PlannerOutput,
    RemotePlannerConsentDecision, RemotePlannerConsentResponseOutcome, ResolveCommandOutcome,
    ToolError, ToolResult,
};
use crate::{join_error_to_tool_error, lock_app_core};

// The blocking section runs in `spawn_blocking` so the inner browser
// `tauri::async_runtime::block_on` calls are safe (a blocking-pool thread is not
// driving the async scheduler). See `docs/BB_ASYNC_RUNTIME_SPEC.md`.
#[tauri::command]
pub async fn execute_planner_output(
    request_id: String,
    planner_output: PlannerOutput,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<ExecutionOutcome, ToolError> {
    let core = Arc::clone(&app_core);
    tauri::async_runtime::spawn_blocking(move || {
        Ok(crate::app_core::execute_planner_output_lock_scoped(
            &core,
            request_id,
            &planner_output,
        ))
    })
    .await
    .map_err(join_error_to_tool_error)?
}

// Runs in `spawn_blocking`. Resolution releases the `AppCore` lock across the
// remote planner round-trip via `resolve_command_lock_scoped`; any browser
// `block_on` reached by a direct command runs safely off the async worker threads.
#[tauri::command]
pub async fn resolve_command(
    request_id: String,
    transcript: String,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<ResolveCommandOutcome, ToolError> {
    let core = Arc::clone(&app_core);
    tauri::async_runtime::spawn_blocking(move || {
        crate::app_core::resolve_command_lock_scoped(&core, request_id, transcript)
    })
    .await
    .map_err(join_error_to_tool_error)?
}

// Runs in `spawn_blocking` so resume-after-confirmation's side-effecting browser
// `block_on` calls are safe off the async worker threads.
#[tauri::command]
pub async fn submit_remote_planner_consent_response(
    challenge_id: String,
    challenge_digest: String,
    decision: RemotePlannerConsentDecision,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<RemotePlannerConsentResponseOutcome, ToolError> {
    let core = Arc::clone(&app_core);
    tauri::async_runtime::spawn_blocking(move || {
        crate::app_core::submit_remote_planner_consent_response_lock_scoped(
            &core,
            challenge_id,
            challenge_digest,
            decision,
        )
    })
    .await
    .map_err(join_error_to_tool_error)?
}

// Runs in `spawn_blocking`; remote narration synthesis uses the same
// lock-scoped prepare -> unlocked HTTP -> commit path as planner narration.
#[tauri::command]
pub async fn submit_narration_consent_response(
    challenge_id: String,
    challenge_digest: String,
    decision: RemotePlannerConsentDecision,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<NarrationConsentResponseOutcome, ToolError> {
    let core = Arc::clone(&app_core);
    tauri::async_runtime::spawn_blocking(move || {
        crate::app_core::submit_narration_consent_response_lock_scoped(
            &core,
            challenge_id,
            challenge_digest,
            decision,
        )
    })
    .await
    .map_err(join_error_to_tool_error)?
}

#[tauri::command]
pub async fn submit_microphone_consent_response(
    challenge_id: String,
    challenge_digest: String,
    decision: RemotePlannerConsentDecision,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<MicrophoneConsentResponseOutcome, ToolError> {
    let core = Arc::clone(&app_core);
    tauri::async_runtime::spawn_blocking(move || {
        let mut guard = lock_app_core(&core)?;
        match guard.resolve_microphone_consent(&challenge_id, &challenge_digest, decision)? {
            crate::app_core::remote_data_consent::MicrophoneConsentResolution::Terminal(
                RemotePlannerConsentResponseOutcome::Denied,
            ) => Ok(MicrophoneConsentResponseOutcome::Denied),
            crate::app_core::remote_data_consent::MicrophoneConsentResolution::Terminal(
                RemotePlannerConsentResponseOutcome::BlockedPersistent,
            ) => Ok(MicrophoneConsentResponseOutcome::BlockedPersistent),
            crate::app_core::remote_data_consent::MicrophoneConsentResolution::Terminal(_) => {
                Err(ToolError {
                    code: String::from("remote_data_consent_internal_error"),
                    message: String::from(
                        "microphone consent resolution returned an unexpected terminal outcome",
                    ),
                    retryable: false,
                    details: None,
                })
            }
            crate::app_core::remote_data_consent::MicrophoneConsentResolution::AuthorizedRetryRequired => {
                Ok(MicrophoneConsentResponseOutcome::AuthorizedRetryRequired)
            }
        }
    })
    .await
    .map_err(join_error_to_tool_error)?
}

#[tauri::command]
pub async fn submit_confirmation_response(
    confirmation_id: String,
    confirmation_digest: String,
    confirmed: bool,
    timed_out: bool,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<ConfirmActionResolution, ToolError> {
    let core = Arc::clone(&app_core);
    tauri::async_runtime::spawn_blocking(move || {
        crate::app_core::submit_confirmation_response_lock_scoped(
            &core,
            confirmation_id,
            confirmation_digest,
            confirmed,
            timed_out,
        )
    })
    .await
    .map_err(join_error_to_tool_error)?
}

// Runs in `spawn_blocking` so a state query waits on the blocking pool rather
// than the main thread while a long command is in flight.
#[tauri::command]
pub async fn get_agent_state(
    request_id: String,
    timeout_ms: Option<u64>,
    include_last_transcript: bool,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<ToolResult<AgentStateData>, ToolError> {
    let core = Arc::clone(&app_core);
    tauri::async_runtime::spawn_blocking(move || {
        let mut guard = lock_app_core(&core)?;
        Ok(guard.execute_get_agent_state(GetAgentStateInput {
            request_id,
            timeout_ms,
            include_last_transcript,
        }))
    })
    .await
    .map_err(join_error_to_tool_error)?
}
