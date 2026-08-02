use std::sync::{Arc, Mutex};

use crate::app_core::AppCore;
use crate::commands::ToolError;
use crate::lock_app_core;

#[derive(serde::Serialize)]
pub struct SetConfirmationThresholdData {
    confirmation_confidence_threshold: f32,
    changed: bool,
}

#[derive(serde::Serialize)]
pub struct SetAllowClickWithoutConfirmationData {
    allow_click_without_confirmation: bool,
    changed: bool,
}

#[derive(serde::Serialize)]
pub struct SetRemotePlannerPrivacyData {
    consent_to_remote_page_data: bool,
    local_only: bool,
    blocked_origins: Vec<String>,
    high_risk_origin_policy: String,
    changed: bool,
}

#[derive(serde::Serialize)]
pub struct SetOcrThresholdsData {
    sparse_text_char_threshold: u32,
    sparse_text_region_threshold: u32,
    changed: bool,
}

#[tauri::command]
pub fn set_confirmation_threshold(
    request_id: String,
    timeout_ms: Option<u64>,
    confirmation_confidence_threshold: f32,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<SetConfirmationThresholdData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let mut app_core = lock_app_core(&app_core)?;
    let changed = app_core.config.safety.confirmation_confidence_threshold
        != confirmation_confidence_threshold;

    app_core
        .set_confirmation_confidence_threshold(confirmation_confidence_threshold)
        .map_err(|error| ToolError {
            code: String::from("confirmation_threshold_persist_failed"),
            message: format!("Failed to persist the requested confirmation threshold: {error}"),
            retryable: false,
            details: None,
        })?;

    Ok(SetConfirmationThresholdData {
        confirmation_confidence_threshold,
        changed,
    })
}

#[tauri::command]
pub fn set_allow_click_without_confirmation(
    request_id: String,
    timeout_ms: Option<u64>,
    allow_click_without_confirmation: bool,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<SetAllowClickWithoutConfirmationData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let mut app_core = lock_app_core(&app_core)?;
    let changed =
        app_core.config.safety.allow_click_without_confirmation != allow_click_without_confirmation;

    app_core
        .set_allow_click_without_confirmation(allow_click_without_confirmation)
        .map_err(|error| ToolError {
            code: String::from("allow_click_without_confirmation_persist_failed"),
            message: format!(
                "Failed to persist the requested click-without-confirmation setting: {error}"
            ),
            retryable: false,
            details: None,
        })?;

    Ok(SetAllowClickWithoutConfirmationData {
        allow_click_without_confirmation,
        changed,
    })
}

#[tauri::command]
pub fn set_remote_planner_privacy_settings(
    request_id: String,
    timeout_ms: Option<u64>,
    consent_to_remote_page_data: bool,
    local_only: bool,
    blocked_origins: Vec<String>,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<SetRemotePlannerPrivacyData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let mut app_core = lock_app_core(&app_core)?;
    let previous = app_core.config.remote_planner_privacy.clone();
    app_core
        .set_remote_planner_privacy_settings(
            consent_to_remote_page_data,
            local_only,
            blocked_origins,
        )
        .map_err(|error| ToolError {
            code: String::from("remote_planner_privacy_persist_failed"),
            message: format!("Failed to persist the remote planner privacy policy: {error}"),
            retryable: false,
            details: None,
        })?;
    let current = app_core.config.remote_planner_privacy.clone();
    Ok(SetRemotePlannerPrivacyData {
        consent_to_remote_page_data: current.consent_to_remote_page_data,
        local_only: current.local_only,
        blocked_origins: current.blocked_origins.clone(),
        high_risk_origin_policy: String::from("block"),
        changed: current != previous,
    })
}

#[tauri::command]
pub fn set_ocr_thresholds(
    request_id: String,
    timeout_ms: Option<u64>,
    sparse_text_char_threshold: u32,
    sparse_text_region_threshold: u32,
    app_core: tauri::State<'_, Arc<Mutex<AppCore>>>,
) -> Result<SetOcrThresholdsData, ToolError> {
    let _ = request_id;
    let _ = timeout_ms;
    let mut app_core = lock_app_core(&app_core)?;
    let changed = app_core.config.ocr.sparse_text_char_threshold != sparse_text_char_threshold
        || app_core.config.ocr.sparse_text_region_threshold != sparse_text_region_threshold;

    app_core
        .set_ocr_thresholds(sparse_text_char_threshold, sparse_text_region_threshold)
        .map_err(|error| ToolError {
            code: String::from("ocr_thresholds_persist_failed"),
            message: format!("Failed to persist the requested OCR thresholds: {error}"),
            retryable: false,
            details: None,
        })?;

    Ok(SetOcrThresholdsData {
        sparse_text_char_threshold,
        sparse_text_region_threshold,
        changed,
    })
}
