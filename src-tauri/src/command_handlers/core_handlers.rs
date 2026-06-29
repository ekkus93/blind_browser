use std::sync::{Arc, Mutex};

use crate::app_core::AppCore;
use crate::commands::{
    AgentStateData, ConfirmActionResolution, ExecutionOutcome, GetAgentStateInput, PlannerOutput,
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
        let mut guard = lock_app_core(&core)?;
        Ok(guard.execute_planner_output(request_id, &planner_output))
    })
    .await
    .map_err(join_error_to_tool_error)?
}

// Runs in `spawn_blocking` so command resolution's browser `block_on` calls are
// safe off the async worker threads.
#[tauri::command]
pub async fn resolve_command(
    request_id: String,
    transcript: String,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<PlannerOutput, ToolError> {
    let core = Arc::clone(&app_core);
    tauri::async_runtime::spawn_blocking(move || {
        let mut guard = lock_app_core(&core)?;
        guard.resolve_command(request_id, transcript)
    })
    .await
    .map_err(join_error_to_tool_error)?
}

// Runs in `spawn_blocking` so resume-after-confirmation's side-effecting browser
// `block_on` calls are safe off the async worker threads.
#[tauri::command]
pub async fn submit_confirmation_response(
    confirmation_id: String,
    confirmed: bool,
    timed_out: bool,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<ConfirmActionResolution, ToolError> {
    let core = Arc::clone(&app_core);
    tauri::async_runtime::spawn_blocking(move || {
        let mut guard = lock_app_core(&core)?;
        Ok(guard.submit_confirmation_response(&confirmation_id, confirmed, timed_out))
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
