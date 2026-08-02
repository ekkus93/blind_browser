use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Monotonic fallback for [`AppCore::next_id`] when the system clock is before the
/// UNIX epoch, so generated ids stay distinct instead of all becoming `0`.
static FALLBACK_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

use crate::asr::AsrController;
use crate::audio_io::AudioPlaybackController;
use crate::browser::{BrowserController, BrowserError, BrowserSessionConfig};
use crate::commands::{ToolError, ToolName, ToolResult};
use crate::config::{AppConfig, AudioSettings, ConfigError};
use crate::ocr::OcrController;
use crate::state::AppState;
use crate::tts::TtsController;
use serde::Serialize;
use tauri::{AppHandle, Manager};

const DEFAULT_FIND_ELEMENT_MAX_CANDIDATES: usize =
    crate::commands::DEFAULT_FIND_ELEMENT_MAX_CANDIDATES;
const MAX_FIND_ELEMENT_CANDIDATES: usize = 5;
const FIND_ELEMENT_AMBIGUITY_MARGIN_BPS: u16 = 800;
const MAX_HISTORY_STEPS: u8 = crate::commands::MAX_HISTORY_STEPS;
const MAX_SCROLL_AMOUNT_PX: f32 = crate::commands::MAX_SCROLL_AMOUNT_PX;
const MAX_COMMAND_REPLAN_CYCLES: usize = 1;
const MAX_DIRECT_FIELD_CANDIDATE_NAMES: usize = 2;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ManagedLocalModelStatusData {
    pub profile_name: Option<String>,
    pub backend: Option<String>,
    pub model_id: Option<String>,
    pub model_path: Option<String>,
    pub available: bool,
    pub download_supported: bool,
    pub download_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ModelManagementSettingsData {
    pub models_dir: String,
    pub check_on_startup: bool,
    pub auto_download_missing: bool,
    pub local_tts: ManagedLocalModelStatusData,
    pub local_asr: ManagedLocalModelStatusData,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DownloadedLocalModelData {
    pub profile_name: String,
    pub model_id: String,
    pub model_path: String,
    pub source_url: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RemotePlannerConnectionSettingsData {
    pub profile_name: String,
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RemotePlannerModelListData {
    pub profile_name: String,
    pub base_url: String,
    pub models: Vec<String>,
}

pub struct AppCore {
    pub app_handle: AppHandle,
    pub config: AppConfig,
    pub state: AppState,
    pub browser: BrowserController,
    recent_field_context: Option<RecentFieldContext>,
    ocr: OcrController,
    tts: TtsController,
    playback: AudioPlaybackController,
    asr: AsrController,
}

mod api_key_tools;
#[cfg(test)]
mod api_key_tools_redirect_tests;

mod content_tools;

mod extraction_tools;
mod ocr_merge;
mod page_model_builder;
mod planner_redaction;

mod fill_correction;
use fill_correction::{PendingRecentFieldContext, RecentFieldContext};

mod form_fill;

mod element_scoring;
use element_scoring::region_bbox_by_id;

mod interaction_tools;

mod listening_tools;
pub use listening_tools::{TranscribeCapturePlan, TranscribeDrainOutcome, TranscribePending};
mod reading_tools;
mod voice_tools;

mod planner_prompt;

mod navigation_tools;
use navigation_tools::browser_error_to_tool_error;

mod click_authorization;
mod command_dispatch;
mod confirmation_workflow;
mod result_reporting;

mod model_management;

mod planning_snapshot;
mod replanning;
mod replanning_orchestrator;
pub(crate) use replanning_orchestrator::{
    resolve_command_lock_scoped, run_command_with_lock_scoped_replanning,
};

mod tool_executor;

mod narration;
mod remote_planner;
mod runtime_config;
mod state_snapshots;

mod settings_adapters;

impl AppCore {
    pub fn new(app_handle: AppHandle) -> Result<Self, ConfigError> {
        let config = AppConfig::load_for_app(&app_handle)?;
        let state = AppState::from_config(&config);
        let browser = BrowserController::new(BrowserSessionConfig {
            visibility: state.browser_visibility,
            user_agent: None,
        });

        Ok(Self {
            app_handle,
            config,
            state,
            browser,
            recent_field_context: None,
            ocr: OcrController::new(),
            tts: TtsController::new(),
            playback: AudioPlaybackController::new(),
            asr: AsrController::new(),
        })
    }

    pub fn update_audio_settings(&mut self, audio: AudioSettings) -> Result<(), ConfigError> {
        let config = AppConfig::persist_audio_settings_for_app(&self.app_handle, &audio)?;
        self.state.apply_audio_settings(&config.audio);
        self.config = config;
        Ok(())
    }

    fn store_recent_field_context(&mut self, context: Option<PendingRecentFieldContext>) {
        self.recent_field_context = context.and_then(|context| {
            self.state
                .current_page_id
                .clone()
                .map(|page_id| RecentFieldContext {
                    page_id,
                    target_description: context.target_description,
                    active_element_id: context.active_element_id,
                    candidate_element_ids: context.candidate_element_ids,
                    pending_text: context.pending_text,
                    submit_after: context.submit_after,
                })
        });
    }

    fn update_recent_field_target(&mut self, element_id: String) {
        let Some(page_id) = self.state.current_page_id.clone() else {
            self.recent_field_context = None;
            return;
        };

        match self.recent_field_context.as_mut() {
            Some(context) if context.page_id == page_id => {
                context.active_element_id = Some(element_id);
            }
            _ => {
                self.recent_field_context = Some(RecentFieldContext {
                    page_id,
                    target_description: None,
                    active_element_id: Some(element_id),
                    candidate_element_ids: Vec::new(),
                    pending_text: None,
                    submit_after: false,
                });
            }
        }
    }

    fn clear_recent_field_context(&mut self) {
        self.recent_field_context = None;
    }

    fn audio_tool_failure<T>(
        &self,
        tool_name: ToolName,
        request_id: String,
        message: String,
        error: ConfigError,
    ) -> ToolResult<T> {
        ToolResult::failure(
            tool_name,
            request_id,
            ToolError {
                code: String::from("config_update_failed"),
                message,
                retryable: false,
                details: Some(serde_json::json!({ "reason": error.to_string() })),
            },
            vec![String::from(
                "Audio setting update did not complete successfully.",
            )],
        )
    }

    fn browser_runtime_missing_page<T>(tool_name: ToolName, request_id: String) -> ToolResult<T> {
        ToolResult::failure(
            tool_name,
            request_id,
            ToolError {
                code: String::from("no_active_page"),
                message: String::from("browser tool requires an active page in runtime state"),
                retryable: false,
                details: None,
            },
            vec![String::from(
                "Browser action could not run because no page has been opened yet.",
            )],
        )
    }

    fn browser_tool_failure<T>(
        &self,
        tool_name: ToolName,
        request_id: String,
        message: String,
        error: BrowserError,
    ) -> ToolResult<T> {
        ToolResult::failure(
            tool_name,
            request_id,
            browser_error_to_tool_error(message, error),
            vec![String::from(
                "Browser backend action did not complete successfully.",
            )],
        )
    }

    fn next_id(&self, prefix: &str, request_id: &str) -> String {
        let timestamp_ms = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            // Monotonic fallback on a pre-epoch clock fault, so ids stay distinct
            // instead of all collapsing to `0`.
            Err(_) => u128::from(FALLBACK_ID_COUNTER.fetch_add(1, Ordering::Relaxed)),
        };
        format!("{prefix}-{request_id}-{timestamp_ms}")
    }

    fn next_confirmation_id(&self, request_id: &str) -> String {
        self.next_id("confirm", request_id)
    }

    fn next_page_id(&self, request_id: &str) -> String {
        self.next_id("page", request_id)
    }

    fn next_image_id(&self, request_id: &str) -> String {
        self.next_id("image", request_id)
    }

    fn next_ocr_region_id(&self, request_id: &str) -> String {
        self.next_id("ocr-region", request_id)
    }

    fn cached_image_dir(&self) -> Result<PathBuf, ToolError> {
        let cache_dir = self
            .app_handle
            .path()
            .app_cache_dir()
            .map_err(|error| ToolError {
                code: String::from("resolve_app_cache_dir_failed"),
                message: String::from(
                    "capture_screenshot could not resolve the app cache directory",
                ),
                retryable: true,
                details: Some(serde_json::json!({ "reason": error.to_string() })),
            })?;
        let image_dir = cache_dir.join("screenshots");
        fs::create_dir_all(&image_dir).map_err(|error| ToolError {
            code: String::from("create_screenshot_dir_failed"),
            message: String::from(
                "capture_screenshot could not create the screenshot cache directory",
            ),
            retryable: true,
            details: Some(serde_json::json!({
                "path": image_dir.display().to_string(),
                "reason": error.to_string(),
            })),
        })?;
        Ok(image_dir)
    }

    fn screenshot_output_path(&self, image_id: &str) -> Result<PathBuf, ToolError> {
        Ok(self.cached_image_dir()?.join(format!("{image_id}.png")))
    }

    fn cached_image_path(&self, image_id: &str) -> Result<PathBuf, ToolError> {
        Ok(self.cached_image_dir()?.join(format!("{image_id}.png")))
    }
}

#[cfg(test)]
mod tests;
