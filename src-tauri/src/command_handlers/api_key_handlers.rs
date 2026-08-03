use std::sync::{Arc, Mutex};

use crate::app_core::{AppCore, RemotePlannerModelListData};
use crate::commands::ToolError;
use crate::provider_endpoint::ProviderEndpointScope;
use crate::{join_error_to_tool_error, lock_app_core};

#[derive(serde::Serialize)]
pub struct SetRemoteApiKeyData {
    profile_name: String,
    api_key_reference: String,
}

#[derive(serde::Serialize)]
pub struct TestRemoteApiKeyData {
    profile_name: String,
    message: String,
}

#[derive(Clone, Copy)]
enum RemoteApiKeyKind {
    Planner,
    Tts,
    Asr,
}

impl RemoteApiKeyKind {
    fn reference_error_code(self) -> &'static str {
        match self {
            Self::Planner => "remote_planner_api_key_reference_missing",
            Self::Tts => "remote_tts_api_key_reference_missing",
            Self::Asr => "remote_asr_api_key_reference_missing",
        }
    }

    fn service_label(self) -> &'static str {
        match self {
            Self::Planner => "remote planner",
            Self::Tts => "remote TTS",
            Self::Asr => "remote ASR",
        }
    }
}

fn require_api_key_reference(
    kind: RemoteApiKeyKind,
    reference: Option<String>,
) -> Result<String, ToolError> {
    reference
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ToolError {
            code: String::from(kind.reference_error_code()),
            message: format!(
                "The {} API key was persisted without a non-empty key reference.",
                kind.service_label()
            ),
            retryable: false,
            details: None,
        })
}

#[tauri::command]
pub fn set_remote_planner_api_key(
    request_id: String,
    timeout_ms: Option<u64>,
    profile_name: String,
    api_key: String,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<SetRemoteApiKeyData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let mut app_core = lock_app_core(&app_core)?;
    let profile_name = profile_name.trim().to_string();
    let api_key = api_key.trim().to_string();
    if profile_name.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_remote_planner_profile"),
            message: String::from(
                "Remote planner API key entry requires a configured profile name.",
            ),
            retryable: false,
            details: None,
        });
    }
    if api_key.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_remote_planner_api_key"),
            message: String::from("Remote planner API key entry requires a non-empty API key."),
            retryable: false,
            details: None,
        });
    }

    app_core
        .set_remote_planner_api_key(&profile_name, &api_key)
        .map_err(|error| ToolError {
            code: String::from("remote_planner_api_key_persist_failed"),
            message: format!("Failed to persist the requested remote planner API key: {error}"),
            retryable: false,
            details: None,
        })?;

    let api_key_reference = require_api_key_reference(
        RemoteApiKeyKind::Planner,
        app_core
            .current_remote_planner_settings()
            .api_key_reference,
    )?;

    Ok(SetRemoteApiKeyData {
        profile_name,
        api_key_reference,
    })
}

// Runs in `spawn_blocking` so the `futures::executor::block_on` network round-trip
// runs off the main thread and off the async worker threads.
#[tauri::command]
pub async fn test_remote_planner_api_key(
    request_id: String,
    timeout_ms: Option<u64>,
    profile_name: String,
    api_key: String,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<TestRemoteApiKeyData, ToolError> {
    let _ = request_id;
    let core = Arc::clone(&app_core);
    tauri::async_runtime::spawn_blocking(move || {
        let app_core = lock_app_core(&core)?;
        let profile_name = profile_name.trim().to_string();
        if profile_name.is_empty() {
            return Err(ToolError {
                code: String::from("invalid_remote_planner_profile"),
                message: String::from(
                    "Remote planner API key test requires a configured profile name.",
                ),
                retryable: false,
                details: None,
            });
        }

        let api_key = api_key.trim().to_string();
        let message = app_core
            .test_remote_planner_api_key(
                &profile_name,
                (!api_key.is_empty()).then_some(api_key.as_str()),
                timeout_ms,
            )
            .map_err(|error| ToolError {
                code: String::from("remote_planner_api_key_test_failed"),
                message: error,
                retryable: false,
                details: None,
            })?;

        Ok(TestRemoteApiKeyData {
            profile_name,
            message,
        })
    })
    .await
    .map_err(join_error_to_tool_error)?
}

// Runs in `spawn_blocking` so the `futures::executor::block_on` network round-trip
// runs off the main thread and off the async worker threads.
#[tauri::command]
pub async fn list_remote_planner_models(
    request_id: String,
    timeout_ms: Option<u64>,
    profile_name: String,
    base_url: String,
    api_key: String,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<RemotePlannerModelListData, ToolError> {
    let _ = request_id;
    let core = Arc::clone(&app_core);
    tauri::async_runtime::spawn_blocking(move || {
        let app_core = lock_app_core(&core)?;
        let profile_name = profile_name.trim().to_string();
        let base_url = base_url.trim().to_string();
        if profile_name.is_empty() {
            return Err(ToolError {
                code: String::from("invalid_remote_planner_profile"),
                message: String::from(
                    "Remote planner model loading requires a configured profile name.",
                ),
                retryable: false,
                details: None,
            });
        }
        if base_url.is_empty() {
            return Err(ToolError {
                code: String::from("invalid_remote_planner_endpoint"),
                message: String::from(
                    "Remote planner model loading requires a non-empty endpoint.",
                ),
                retryable: false,
                details: None,
            });
        }

        let endpoint_scope =
            ProviderEndpointScope::parse(&base_url).map_err(|reason| ToolError {
                code: String::from("invalid_remote_planner_endpoint"),
                message: format!("Remote planner model endpoint is invalid: {reason}"),
                retryable: false,
                details: None,
            })?;
        let normalized_base_url = endpoint_scope.normalized_base_url().to_string();
        let models = app_core
            .list_remote_planner_models(
                &profile_name,
                Some(&normalized_base_url),
                (!api_key.trim().is_empty()).then_some(api_key.as_str()),
                timeout_ms,
            )
            .map_err(|error| ToolError {
                code: String::from("remote_planner_models_load_failed"),
                message: error,
                retryable: false,
                details: None,
            })?;

        Ok(RemotePlannerModelListData {
            profile_name,
            base_url: normalized_base_url,
            models,
        })
    })
    .await
    .map_err(join_error_to_tool_error)?
}

#[tauri::command]
pub fn set_remote_tts_api_key(
    request_id: String,
    timeout_ms: Option<u64>,
    profile_name: String,
    api_key: String,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<SetRemoteApiKeyData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let mut app_core = lock_app_core(&app_core)?;
    let profile_name = profile_name.trim().to_string();
    let api_key = api_key.trim().to_string();
    if profile_name.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_remote_tts_profile"),
            message: String::from("Remote TTS API key entry requires a configured profile name."),
            retryable: false,
            details: None,
        });
    }
    if api_key.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_remote_tts_api_key"),
            message: String::from("Remote TTS API key entry requires a non-empty API key."),
            retryable: false,
            details: None,
        });
    }

    app_core
        .set_remote_tts_api_key(&profile_name, &api_key)
        .map_err(|error| ToolError {
            code: String::from("remote_tts_api_key_persist_failed"),
            message: format!("Failed to persist the requested remote TTS API key: {error}"),
            retryable: false,
            details: None,
        })?;

    let api_key_reference = require_api_key_reference(
        RemoteApiKeyKind::Tts,
        app_core.current_remote_tts_settings().api_key_reference,
    )?;

    Ok(SetRemoteApiKeyData {
        profile_name,
        api_key_reference,
    })
}

// Runs in `spawn_blocking` so the `futures::executor::block_on` network round-trip
// runs off the main thread and off the async worker threads.
#[tauri::command]
pub async fn test_remote_tts_api_key(
    request_id: String,
    timeout_ms: Option<u64>,
    profile_name: String,
    api_key: String,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<TestRemoteApiKeyData, ToolError> {
    let _ = request_id;
    let core = Arc::clone(&app_core);
    tauri::async_runtime::spawn_blocking(move || {
        let app_core = lock_app_core(&core)?;
        let profile_name = profile_name.trim().to_string();
        if profile_name.is_empty() {
            return Err(ToolError {
                code: String::from("invalid_remote_tts_profile"),
                message: String::from(
                    "Remote TTS API key test requires a configured profile name.",
                ),
                retryable: false,
                details: None,
            });
        }

        let api_key = api_key.trim().to_string();
        let message = app_core
            .test_remote_tts_api_key(
                &profile_name,
                (!api_key.is_empty()).then_some(api_key.as_str()),
                timeout_ms,
            )
            .map_err(|error| ToolError {
                code: String::from("remote_tts_api_key_test_failed"),
                message: error,
                retryable: false,
                details: None,
            })?;

        Ok(TestRemoteApiKeyData {
            profile_name,
            message,
        })
    })
    .await
    .map_err(join_error_to_tool_error)?
}

#[tauri::command]
pub fn set_remote_asr_api_key(
    request_id: String,
    timeout_ms: Option<u64>,
    profile_name: String,
    api_key: String,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<SetRemoteApiKeyData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let mut app_core = lock_app_core(&app_core)?;
    let profile_name = profile_name.trim().to_string();
    let api_key = api_key.trim().to_string();
    if profile_name.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_remote_asr_profile"),
            message: String::from("Remote ASR API key entry requires a configured profile name."),
            retryable: false,
            details: None,
        });
    }
    if api_key.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_remote_asr_api_key"),
            message: String::from("Remote ASR API key entry requires a non-empty API key."),
            retryable: false,
            details: None,
        });
    }

    app_core
        .set_remote_asr_api_key(&profile_name, &api_key)
        .map_err(|error| ToolError {
            code: String::from("remote_asr_api_key_persist_failed"),
            message: format!("Failed to persist the requested remote ASR API key: {error}"),
            retryable: false,
            details: None,
        })?;

    let api_key_reference = require_api_key_reference(
        RemoteApiKeyKind::Asr,
        app_core.current_remote_asr_settings().api_key_reference,
    )?;

    Ok(SetRemoteApiKeyData {
        profile_name,
        api_key_reference,
    })
}

// Runs in `spawn_blocking` so the `futures::executor::block_on` network round-trip
// runs off the main thread and off the async worker threads.
#[tauri::command]
pub async fn test_remote_asr_api_key(
    request_id: String,
    timeout_ms: Option<u64>,
    profile_name: String,
    api_key: String,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<TestRemoteApiKeyData, ToolError> {
    let _ = request_id;
    let core = Arc::clone(&app_core);
    tauri::async_runtime::spawn_blocking(move || {
        let app_core = lock_app_core(&core)?;
        let profile_name = profile_name.trim().to_string();
        if profile_name.is_empty() {
            return Err(ToolError {
                code: String::from("invalid_remote_asr_profile"),
                message: String::from(
                    "Remote ASR API key test requires a configured profile name.",
                ),
                retryable: false,
                details: None,
            });
        }

        let api_key = api_key.trim().to_string();
        let message = app_core
            .test_remote_asr_api_key(
                &profile_name,
                (!api_key.is_empty()).then_some(api_key.as_str()),
                timeout_ms,
            )
            .map_err(|error| ToolError {
                code: String::from("remote_asr_api_key_test_failed"),
                message: error,
                retryable: false,
                details: None,
            })?;

        Ok(TestRemoteApiKeyData {
            profile_name,
            message,
        })
    })
    .await
    .map_err(join_error_to_tool_error)?
}

#[cfg(test)]
mod tests {
    use super::{require_api_key_reference, RemoteApiKeyKind};

    fn assert_missing_reference(kind: RemoteApiKeyKind, expected_code: &str) {
        for reference in [None, Some(String::new()), Some(String::from("   "))] {
            let error = require_api_key_reference(kind, reference)
                .expect_err("missing API key reference should fail closed");
            assert_eq!(error.code, expected_code);
            assert!(!error.retryable);
            assert!(error.details.is_none());
        }
    }

    #[test]
    fn planner_api_key_reference_is_required() {
        assert_missing_reference(
            RemoteApiKeyKind::Planner,
            "remote_planner_api_key_reference_missing",
        );
    }

    #[test]
    fn tts_api_key_reference_is_required() {
        assert_missing_reference(
            RemoteApiKeyKind::Tts,
            "remote_tts_api_key_reference_missing",
        );
    }

    #[test]
    fn asr_api_key_reference_is_required() {
        assert_missing_reference(
            RemoteApiKeyKind::Asr,
            "remote_asr_api_key_reference_missing",
        );
    }

    #[test]
    fn non_empty_api_key_reference_is_normalized() {
        let reference = require_api_key_reference(
            RemoteApiKeyKind::Planner,
            Some(String::from("  keyring://remote-planner/profile  ")),
        )
        .expect("non-empty API key reference should be accepted");
        assert_eq!(reference, "keyring://remote-planner/profile");
    }
}
