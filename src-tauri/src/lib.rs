use std::error::Error;
use std::sync::{Arc, Mutex};

pub mod app_core;
pub mod asr;
pub mod atomic_file;
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
pub mod provider_endpoint;
pub mod state;
pub mod tts;
pub mod url_policy;

mod command_handlers;

use command_handlers::api_key_handlers::*;
use command_handlers::audio_handlers::*;
use command_handlers::core_handlers::*;
use command_handlers::model_handlers::*;
use command_handlers::provider_handlers::*;
use command_handlers::safety_handlers::*;
use command_handlers::url_handlers::*;
use command_handlers::voice_handlers::*;

use tauri::Manager;

use crate::app_core::AppCore;
use crate::commands::ToolError;

/// Acquire the global runtime-state guard.
///
/// Takes `&Mutex<AppCore>`, so it works both with a borrowed
/// `tauri::State<'_, Arc<Mutex<AppCore>>>` (deref-coerced) and with an owned
/// `Arc<Mutex<AppCore>>` clone moved into a `spawn_blocking` closure.
fn lock_app_core(
    app_core: &Mutex<AppCore>,
) -> Result<std::sync::MutexGuard<'_, AppCore>, ToolError> {
    app_core.lock().map_err(|_| ToolError {
        code: String::from("app_core_lock_failed"),
        message: String::from("failed to acquire the app runtime state lock"),
        retryable: true,
        details: None,
    })
}

/// Convert a `spawn_blocking` join failure (panic or cancellation in the
/// blocking task) into a non-retryable internal `ToolError`.
fn join_error_to_tool_error(error: tauri::Error) -> ToolError {
    ToolError {
        code: String::from("internal_task_failed"),
        message: format!("a background runtime task failed to complete: {error}"),
        retryable: false,
        details: None,
    }
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
            open_external_url,
            get_agent_state,
            set_playback_volume,
            set_playback_speed,
            set_default_tts_voice,
            set_active_tts_profile,
            set_tts_provider_mode,
            set_asr_provider_mode,
            set_browser_visibility,
            set_confirmation_confidence_threshold,
            set_allow_click_without_confirmation,
            set_ocr_thresholds,
            test_remote_planner_api_key,
            test_remote_tts_api_key,
            test_remote_asr_api_key,
            set_remote_planner_api_key,
            set_remote_tts_api_key,
            set_remote_asr_api_key,
            get_remote_planner_connection_settings,
            set_remote_planner_connection_settings,
            reset_remote_planner_connection_settings_to_defaults,
            list_remote_planner_models,
            get_model_management_settings,
            set_model_management_settings,
            download_active_local_tts_model,
            download_active_local_asr_model,
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            let app_core = AppCore::new(app_handle)?;
            app.manage(Arc::new(Mutex::new(app_core)));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running blind_browser");
}

pub fn error_chain(error: &(dyn Error + 'static)) -> Vec<String> {
    let mut chain = vec![error.to_string()];
    let mut source = error.source();
    while let Some(current) = source {
        chain.push(current.to_string());
        source = current.source();
    }
    chain
}
