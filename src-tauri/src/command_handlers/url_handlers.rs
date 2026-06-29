use std::process::Command;
use std::sync::{Arc, Mutex};

use crate::app_core::AppCore;
use crate::commands::{OpenUrlData, OpenUrlInput, ToolError, ToolResult};
use crate::{join_error_to_tool_error, lock_app_core};

// Runs in `spawn_blocking` so the browser navigation's
// `tauri::async_runtime::block_on` calls are safe off the async worker threads.
#[tauri::command]
pub async fn open_url(
    request_id: String,
    timeout_ms: Option<u64>,
    url: String,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<ToolResult<OpenUrlData>, ToolError> {
    let core = Arc::clone(&app_core);
    tauri::async_runtime::spawn_blocking(move || {
        let mut guard = lock_app_core(&core)?;
        Ok(guard.execute_open_url(OpenUrlInput {
            request_id,
            timeout_ms,
            url,
            wait_for_load_state: None,
        }))
    })
    .await
    .map_err(join_error_to_tool_error)?
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
