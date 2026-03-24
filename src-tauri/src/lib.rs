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
use crate::commands::{ConfirmActionResolution, ExecutionOutcome, PlannerOutput, ToolError};

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

pub fn run() {
    logging::init_logging();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            resolve_command,
            execute_planner_output,
            submit_confirmation_response
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
