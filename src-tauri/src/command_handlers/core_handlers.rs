use std::sync::Mutex;

use crate::app_core::AppCore;
use crate::commands::{
    AgentStateData, ConfirmActionResolution, ExecutionOutcome, GetAgentStateInput, PlannerOutput,
    ToolError, ToolResult,
};
use crate::lock_app_core;

#[tauri::command]
pub fn execute_planner_output(
    request_id: String,
    planner_output: PlannerOutput,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<ExecutionOutcome, ToolError> {
    let mut app_core = lock_app_core(&app_core)?;
    Ok(app_core.execute_planner_output(request_id, &planner_output))
}

#[tauri::command]
pub fn resolve_command(
    request_id: String,
    transcript: String,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<PlannerOutput, ToolError> {
    let mut app_core = lock_app_core(&app_core)?;
    app_core.resolve_command(request_id, transcript)
}

#[tauri::command]
pub fn submit_confirmation_response(
    confirmation_id: String,
    confirmed: bool,
    timed_out: bool,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<ConfirmActionResolution, ToolError> {
    let mut app_core = lock_app_core(&app_core)?;
    Ok(app_core.submit_confirmation_response(&confirmation_id, confirmed, timed_out))
}

#[tauri::command]
pub fn get_agent_state(
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
