use std::process::Command;
use std::sync::Mutex;

use crate::app_core::AppCore;
use crate::commands::{OpenUrlData, OpenUrlInput, ToolError, ToolResult};
use crate::lock_app_core;

// GUARDRAIL: Keep this a plain `#[tauri::command]` (main-thread). Opening a URL
// drives the browser, which calls `tauri::async_runtime::block_on` and panics
// when driven from a tokio worker. See BB_CODE_REVIEW2_TODO.md P1.1.2 / P1.1.4.
#[tauri::command]
pub fn open_url(
    request_id: String,
    timeout_ms: Option<u64>,
    url: String,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<ToolResult<OpenUrlData>, ToolError> {
    let mut app_core = lock_app_core(&app_core)?;
    Ok(app_core.execute_open_url(OpenUrlInput {
        request_id,
        timeout_ms,
        url,
        wait_for_load_state: None,
    }))
}

fn validate_external_url(url: &str) -> Result<(), ToolError> {
    if url.starts_with("https://") {
        return Ok(());
    }

    Err(ToolError {
        code: String::from("external_url_invalid"),
        message: String::from("Only HTTPS external links can be opened."),
        retryable: false,
        details: None,
    })
}

fn launch_external_url(url: &str) -> Result<(), ToolError> {
    #[cfg(target_os = "macos")]
    let mut command = {
        let mut command = Command::new("open");
        command.arg(url);
        command
    };

    #[cfg(target_os = "windows")]
    let mut command = {
        let mut command = Command::new("cmd");
        command.args(["/C", "start", "", url]);
        command
    };

    #[cfg(all(unix, not(target_os = "macos")))]
    let mut command = {
        let mut command = Command::new("xdg-open");
        command.arg(url);
        command
    };

    command.spawn().map_err(|error| ToolError {
        code: String::from("external_url_open_failed"),
        message: format!("Failed to open the external link: {error}"),
        retryable: true,
        details: None,
    })?;

    Ok(())
}

#[tauri::command]
pub fn open_external_url(url: String) -> Result<(), ToolError> {
    validate_external_url(&url)?;
    launch_external_url(&url)
}
