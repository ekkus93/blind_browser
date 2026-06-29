use std::sync::Mutex;

use crate::app_core::AppCore;
use crate::browser::BrowserVisibilityMode;
use crate::commands::{
    SetBrowserVisibilityData, SetBrowserVisibilityInput, SetPlaybackSpeedData,
    SetPlaybackSpeedInput, SetPlaybackVolumeData, SetPlaybackVolumeInput, SetTtsVoiceData,
    SetTtsVoiceInput, ToolError, ToolResult, TtsVoiceName,
};
use crate::lock_app_core;

#[tauri::command]
pub fn set_playback_volume(
    request_id: String,
    timeout_ms: Option<u64>,
    volume: f32,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<ToolResult<SetPlaybackVolumeData>, ToolError> {
    let mut app_core = lock_app_core(&app_core)?;
    Ok(
        app_core.execute_set_playback_volume(SetPlaybackVolumeInput {
            request_id,
            timeout_ms,
            volume,
        }),
    )
}

#[tauri::command]
pub fn set_playback_speed(
    request_id: String,
    timeout_ms: Option<u64>,
    speed: f32,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<ToolResult<SetPlaybackSpeedData>, ToolError> {
    let mut app_core = lock_app_core(&app_core)?;
    Ok(app_core.execute_set_playback_speed(SetPlaybackSpeedInput {
        request_id,
        timeout_ms,
        speed,
    }))
}

#[tauri::command]
pub fn set_browser_visibility(
    request_id: String,
    timeout_ms: Option<u64>,
    mode: BrowserVisibilityMode,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<ToolResult<SetBrowserVisibilityData>, ToolError> {
    let mut app_core = lock_app_core(&app_core)?;
    Ok(
        app_core.execute_set_browser_visibility(SetBrowserVisibilityInput {
            request_id,
            timeout_ms,
            mode,
        }),
    )
}

#[tauri::command]
pub fn set_tts_voice(
    request_id: String,
    timeout_ms: Option<u64>,
    voice: TtsVoiceName,
    app_core: tauri::State<'_, Mutex<AppCore>>,
) -> Result<ToolResult<SetTtsVoiceData>, ToolError> {
    let mut app_core = lock_app_core(&app_core)?;
    Ok(app_core.execute_set_tts_voice(SetTtsVoiceInput {
        request_id,
        timeout_ms,
        voice,
    }))
}
