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
            set_browser_visibility,
            set_tts_voice,
            set_confirmation_threshold,
            set_allow_click_without_confirmation,
            set_ocr_thresholds,
            set_remote_planner_connection_settings,
            reset_remote_planner_connection_settings,
            list_remote_planner_models,
            set_remote_planner_api_key,
            set_remote_tts_api_key,
            set_remote_asr_api_key,
            test_remote_planner_api_key,
            test_remote_tts_api_key,
            test_remote_asr_api_key,
            get_model_management_settings,
            set_model_management_settings,
            download_active_local_tts_model,
            download_active_local_asr_model,
            set_tts_provider_selection,
            set_asr_provider_selection,
            set_tts_model_selection
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
