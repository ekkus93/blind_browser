use std::process::Command;
use std::sync::{Arc, Mutex};

use url::Url;

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

fn external_url_error(message: impl Into<String>) -> ToolError {
    ToolError {
        code: String::from("external_url_invalid"),
        message: message.into(),
        retryable: false,
        details: None,
    }
}

fn validate_external_url(url: &str) -> Result<String, ToolError> {
    if url.chars().any(char::is_control) {
        return Err(external_url_error(
            "External links cannot contain control characters.",
        ));
    }

    let parsed = Url::parse(url)
        .map_err(|_| external_url_error("The external link is not a valid absolute URL."))?;

    if parsed.scheme() != "https" {
        return Err(external_url_error(
            "Only HTTPS external links can be opened.",
        ));
    }
    if parsed.host_str().is_none() {
        return Err(external_url_error(
            "External HTTPS links must include a host.",
        ));
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(external_url_error(
            "External links cannot contain embedded credentials.",
        ));
    }
    if parsed.query().is_some() {
        return Err(external_url_error(
            "External links cannot contain a query string.",
        ));
    }
    if parsed.fragment().is_some() {
        return Err(external_url_error(
            "External links cannot contain a fragment.",
        ));
    }

    Ok(parsed.to_string())
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
    let normalized_url = validate_external_url(&url)?;
    launch_external_url(&normalized_url)
}

#[cfg(test)]
mod tests {
    use super::validate_external_url;

    fn assert_invalid(url: &str) {
        let error = validate_external_url(url).expect_err("URL should be rejected");
        assert_eq!(error.code, "external_url_invalid");
        assert!(!error.retryable);
        assert!(error.details.is_none());
    }

    #[test]
    fn rejects_non_https_url() {
        assert_invalid("http://example.com/path");
        assert_invalid("file:///tmp/example");
    }

    #[test]
    fn rejects_missing_host() {
        assert_invalid("https://");
    }

    #[test]
    fn rejects_control_characters() {
        assert_invalid("https://example.com/path\nnext");
        assert_invalid("https://example.com/\u{0000}");
    }

    #[test]
    fn rejects_malformed_url() {
        assert_invalid("https://[::1");
        assert_invalid("not a url");
    }

    #[test]
    fn rejects_embedded_credentials() {
        assert_invalid("https://user@example.com/path");
        assert_invalid("https://user:password@example.com/path");
    }

    #[test]
    fn rejects_query_strings_and_fragments() {
        assert_invalid("https://example.com/path?token=secret");
        assert_invalid("https://example.com/path#section");
    }

    #[test]
    fn accepts_and_normalizes_valid_https_url() {
        let normalized = validate_external_url("HTTPS://Example.COM/a/../help")
            .expect("valid HTTPS URL should be accepted");
        assert_eq!(normalized, "https://example.com/help");
    }
}
