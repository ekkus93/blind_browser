use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asr::AsrController;
use crate::audio_io::AudioPlaybackController;
use crate::browser::{BrowserController, BrowserError, BrowserSessionConfig};
use crate::commands::{
    build_planner_skill_selection, execute_planner_output, planner_available_tools,
    resolve_direct_audio_command, resolve_direct_browser_visibility_command,
    resolve_direct_navigation_readback_command, resolve_direct_open_url_command,
    resolve_direct_read_page_command, resolve_direct_read_title_command,
    resolve_direct_repeat_command, resolve_direct_status_query_command,
    resolve_direct_voice_input_command, resume_after_confirmation, validate_planner_output,
    AgentStateData, ConfirmActionData, ConfirmActionInput, ConfirmActionResolution, ExecutionOutcome,
    GetAgentStateInput, GetRuntimeStatusData, GetRuntimeStatusInput, PlannerInput, PlannerOutput,
    PlannerToolHistoryEntry, ReportResultData, ReportResultInput, ToolError, ToolName, ToolResult,
};
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

mod content_tools;

mod extraction_tools;
mod ocr_merge;
mod page_model_builder;

mod fill_correction;
use fill_correction::{PendingRecentFieldContext, RecentFieldContext, resolve_recent_fill_correction_command};

mod form_fill;
use form_fill::{
    resolve_direct_fill_command_internal, resolve_direct_focus_field_command,
    resolve_direct_submit_form_command,
};

mod element_scoring;
use element_scoring::{normalize_optional_text, region_bbox_by_id};

mod interaction_tools;

mod voice_tools;
mod reading_tools;
mod listening_tools;

mod planner_prompt;

mod navigation_tools;
use navigation_tools::browser_error_to_tool_error;

mod model_management;

mod replanning;
use replanning::execute_bounded_replanning_loop;

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









    pub fn execute_planner_output(
        &mut self,
        request_id: String,
        planner_output: &PlannerOutput,
    ) -> ExecutionOutcome {
        let outcome = execute_planner_output(self, request_id, planner_output);
        self.state.apply_execution_outcome(&outcome);
        outcome
    }

    fn execute_command_with_replanning(
        &mut self,
        request_id: String,
        transcript: String,
    ) -> Result<ExecutionOutcome, ToolError> {
        execute_bounded_replanning_loop(self, &request_id, &transcript)
    }

    pub fn resolve_command(
        &mut self,
        request_id: String,
        transcript: String,
    ) -> Result<PlannerOutput, ToolError> {
        self.resolve_command_with_recent_results(request_id, &transcript, Vec::new())
    }

    fn resolve_command_with_recent_results(
        &mut self,
        request_id: String,
        transcript: &str,
        recent_tool_results: Vec<PlannerToolHistoryEntry>,
    ) -> Result<PlannerOutput, ToolError> {
        let transcript = transcript.trim();
        if transcript.is_empty() {
            return Err(ToolError {
                code: String::from("empty_transcript"),
                message: String::from("resolve_command requires a non-empty transcript"),
                retryable: false,
                details: None,
            });
        }

        let available_tools = planner_available_tools();
        let current_dir = std::env::current_dir().ok();
        let user_skill_root = self
            .app_handle
            .path()
            .app_config_dir()
            .ok()
            .map(|path| path.join("skills"));
        let skill_selection = build_planner_skill_selection(
            current_dir.as_deref(),
            user_skill_root.as_deref(),
            transcript,
            &available_tools,
        );

        if let Some(planner_output) = resolve_direct_browser_visibility_command(
            transcript,
            &request_id,
            self.state.browser_visibility,
            &skill_selection.active_skill_names,
        ) {
            validate_planner_output(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
            )?;
            return Ok(planner_output);
        }

        let current_agent_state = self.current_agent_state_snapshot(true);

        if let Some(planner_output) = resolve_direct_navigation_readback_command(
            transcript,
            &request_id,
            &skill_selection.active_skill_names,
        ) {
            validate_planner_output(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
            )?;
            return Ok(planner_output);
        }

        if let Some(planner_output) = resolve_direct_voice_input_command(
            transcript,
            &request_id,
            &skill_selection.active_skill_names,
        ) {
            validate_planner_output(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
            )?;
            return Ok(planner_output);
        }

        if let Some(planner_output) = resolve_direct_open_url_command(
            transcript,
            &request_id,
            &skill_selection.active_skill_names,
        ) {
            validate_planner_output(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
            )?;
            return Ok(planner_output);
        }

        if let Some(planner_output) = resolve_direct_read_page_command(
            transcript,
            &request_id,
            self.state.current_page.as_ref(),
            &current_agent_state,
            &skill_selection.active_skill_names,
        ) {
            validate_planner_output(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
            )?;
            return Ok(planner_output);
        }

        if let Some((planner_output, next_recent_field_context)) =
            resolve_recent_fill_correction_command(
                transcript,
                &request_id,
                self.state.current_page_id.as_deref(),
                self.state.current_page.as_ref(),
                &skill_selection.active_skill_names,
                self.recent_field_context.as_ref(),
            )
        {
            self.recent_field_context = next_recent_field_context;
            validate_planner_output(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
            )?;
            return Ok(planner_output);
        }

        if let Some(resolved) = resolve_direct_fill_command_internal(
            transcript,
            &request_id,
            self.state.current_page_id.as_deref(),
            self.state.current_page.as_ref(),
            &skill_selection.active_skill_names,
            self.config.safety.confirmation_confidence_threshold,
            true,
        ) {
            self.store_recent_field_context(resolved.recent_field_context);
            let planner_output = resolved.planner_output;
            validate_planner_output(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
            )?;
            return Ok(planner_output);
        }

        if let Some(resolved) = resolve_direct_fill_command_internal(
            transcript,
            &request_id,
            self.state.current_page_id.as_deref(),
            self.state.current_page.as_ref(),
            &skill_selection.active_skill_names,
            self.config.safety.confirmation_confidence_threshold,
            false,
        ) {
            self.store_recent_field_context(resolved.recent_field_context);
            let planner_output = resolved.planner_output;
            validate_planner_output(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
            )?;
            return Ok(planner_output);
        }

        if let Some(planner_output) = resolve_direct_submit_form_command(
            transcript,
            &request_id,
            self.state.current_page.as_ref(),
            &skill_selection.active_skill_names,
        ) {
            validate_planner_output(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
            )?;
            return Ok(planner_output);
        }

        if let Some(planner_output) = resolve_direct_focus_field_command(
            transcript,
            &request_id,
            self.state.current_page.as_ref(),
            &skill_selection.active_skill_names,
            self.config.safety.confirmation_confidence_threshold,
        ) {
            validate_planner_output(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
            )?;
            return Ok(planner_output);
        }

        if let Some(planner_output) = resolve_direct_repeat_command(
            transcript,
            &request_id,
            &current_agent_state,
            &skill_selection.active_skill_names,
        ) {
            validate_planner_output(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
            )?;
            return Ok(planner_output);
        }

        if let Some(planner_output) = resolve_direct_read_title_command(
            transcript,
            &request_id,
            &current_agent_state,
            &skill_selection.active_skill_names,
        ) {
            validate_planner_output(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
            )?;
            return Ok(planner_output);
        }

        let current_runtime_status = self.current_runtime_status_snapshot(false);

        if let Some(planner_output) = resolve_direct_status_query_command(
            transcript,
            &request_id,
            &current_agent_state,
            &current_runtime_status,
            &skill_selection.active_skill_names,
        ) {
            validate_planner_output(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
            )?;
            return Ok(planner_output);
        }

        if let Some(planner_output) = resolve_direct_audio_command(
            transcript,
            &request_id,
            self.state.audio.playback_volume,
            self.state.audio.playback_speed,
            &skill_selection.active_skill_names,
        ) {
            validate_planner_output(
                &planner_output,
                &available_tools,
                &skill_selection.active_skill_names,
            )?;
            return Ok(planner_output);
        }

        let planner_input = PlannerInput {
            request_id: request_id.clone(),
            transcript: transcript.to_string(),
            agent_state: current_agent_state,
            safety: (&self.config.safety).into(),
            available_tools: available_tools.clone(),
            active_skill_names: skill_selection.active_skill_names.clone(),
            relevant_skill_summaries: skill_selection.relevant_skill_summaries.clone(),
            page_snapshot: self.current_page_snapshot(Some(1_200), true),
            page_model: self.state.current_page.clone(),
            recent_tool_results,
        };

        let planner_output = self.resolve_planner_output(&planner_input)?;
        validate_planner_output(
            &planner_output,
            &available_tools,
            &planner_input.active_skill_names,
        )?;
        Ok(planner_output)
    }

    pub fn resume_after_confirmation(
        &mut self,
        confirmation_id: &str,
        confirmed: bool,
    ) -> ExecutionOutcome {
        let Some(pending_plan_execution) = self.state.pending_plan_execution.clone() else {
            return ExecutionOutcome::Aborted {
                trace: crate::commands::ExecutionTrace {
                    executed_step_ids: Vec::new(),
                    tool_results: Vec::new(),
                },
                error: ToolError {
                    code: String::from("missing_pending_execution"),
                    message: String::from(
                        "there is no pending plan execution to resume for confirmation",
                    ),
                    retryable: false,
                    details: None,
                },
            };
        };

        if self.state.pending_confirmation_id.as_deref() != Some(confirmation_id) {
            return ExecutionOutcome::Aborted {
                trace: crate::commands::ExecutionTrace {
                    executed_step_ids: Vec::new(),
                    tool_results: Vec::new(),
                },
                error: ToolError {
                    code: String::from("confirmation_id_mismatch"),
                    message: String::from(
                        "confirmation response did not match the stored pending confirmation id",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({
                        "expected_confirmation_id": self.state.pending_confirmation_id,
                        "received_confirmation_id": confirmation_id,
                    })),
                },
            };
        }

        let outcome =
            resume_after_confirmation(self, &pending_plan_execution, confirmation_id, confirmed);
        self.state.apply_execution_outcome(&outcome);
        outcome
    }

    pub fn submit_confirmation_response(
        &mut self,
        confirmation_id: &str,
        confirmed: bool,
        timed_out: bool,
    ) -> ConfirmActionResolution {
        let prompt_text = self
            .state
            .pending_plan_execution
            .as_ref()
            .filter(|pending| pending.confirmation_id == confirmation_id)
            .map(|pending| pending.prompt_text.clone())
            .unwrap_or_default();

        let should_resume = confirmed && !timed_out;
        let resume_outcome = self.resume_after_confirmation(confirmation_id, should_resume);
        let tool_result = match &resume_outcome {
            ExecutionOutcome::Aborted { error, .. } => ToolResult::failure(
                ToolName::ConfirmAction,
                confirmation_id.to_string(),
                error.clone(),
                vec![String::from(
                    "Confirmation response could not be applied to the pending plan.",
                )],
            ),
            _ => ToolResult::success(
                ToolName::ConfirmAction,
                confirmation_id.to_string(),
                ConfirmActionData {
                    confirmation_id: confirmation_id.to_string(),
                    prompt_text,
                    confirmed: Some(confirmed),
                    timed_out,
                },
                vec![String::from(
                    "Confirmation response was applied to the pending plan execution.",
                )],
            ),
        };

        ConfirmActionResolution {
            tool_result,
            resume_outcome,
        }
    }

    pub fn execute_confirm_action(
        &mut self,
        input: ConfirmActionInput,
    ) -> ToolResult<ConfirmActionData> {
        let prompt_text = input.prompt_text.trim().to_string();
        if prompt_text.is_empty() {
            return ToolResult::failure(
                ToolName::ConfirmAction,
                input.request_id,
                ToolError {
                    code: String::from("invalid_confirmation_prompt"),
                    message: String::from("confirm_action requires a non-empty prompt_text"),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Confirmation request was rejected because the prompt text was empty.",
                )],
            );
        }

        let mut observations = vec![String::from("Prepared a confirmation prompt for the user.")];
        let reason = input.reason.trim();
        if !reason.is_empty() {
            observations.push(reason.to_string());
        }

        ToolResult::success(
            ToolName::ConfirmAction,
            input.request_id.clone(),
            ConfirmActionData {
                confirmation_id: self.next_confirmation_id(&input.request_id),
                prompt_text,
                confirmed: None,
                timed_out: false,
            },
            observations,
        )
    }
    pub fn execute_get_agent_state(
        &mut self,
        input: GetAgentStateInput,
    ) -> ToolResult<AgentStateData> {
        self.sync_narration_playback_state();
        ToolResult::success(
            ToolName::GetAgentState,
            input.request_id,
            self.current_agent_state_snapshot(input.include_last_transcript),
            vec![String::from("Read the current agent state.")],
        )
    }

    pub fn execute_get_runtime_status(
        &mut self,
        input: GetRuntimeStatusInput,
    ) -> ToolResult<GetRuntimeStatusData> {
        self.sync_narration_playback_state();
        ToolResult::success(
            ToolName::GetRuntimeStatus,
            input.request_id,
            self.current_runtime_status_snapshot(input.include_provider_modes),
            vec![String::from("Read the current runtime status.")],
        )
    }

    pub fn execute_report_result(
        &mut self,
        input: ReportResultInput,
    ) -> ToolResult<ReportResultData> {
        let summary = input.summary.trim().to_string();
        if summary.is_empty() {
            return ToolResult::failure(
                ToolName::ReportResult,
                input.request_id,
                ToolError {
                    code: String::from("invalid_report_summary"),
                    message: String::from("report_result requires a non-empty summary"),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Final result reporting was rejected because the summary was empty.",
                )],
            );
        }

        let next_recommended_action = normalize_optional_text(input.next_recommended_action);
        let user_message = normalize_optional_text(input.user_message);
        let spoken_message = user_message.clone().unwrap_or_else(|| summary.clone());

        if let Err(error) = self.begin_feedback_narration(&spoken_message) {
            return ToolResult::failure(
                ToolName::ReportResult,
                input.request_id,
                error,
                vec![String::from(
                    "Final result reporting could not start audible feedback with the configured TTS backend.",
                )],
            );
        }

        ToolResult::success(
            ToolName::ReportResult,
            input.request_id,
            ReportResultData {
                status: input.status,
                summary,
                next_recommended_action,
                user_message,
            },
            vec![
                String::from(
                    "Reported the final planner result in a structured deterministic payload.",
                ),
                String::from("Started spoken feedback for the reported result summary."),
            ],
        )
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

    fn next_confirmation_id(&self, request_id: &str) -> String {
        let timestamp_ms = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(_) => 0,
        };
        format!("confirm-{request_id}-{timestamp_ms}")
    }

    fn next_page_id(&self, request_id: &str) -> String {
        let timestamp_ms = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(_) => 0,
        };
        format!("page-{request_id}-{timestamp_ms}")
    }

    fn next_image_id(&self, request_id: &str) -> String {
        let timestamp_ms = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(_) => 0,
        };
        format!("image-{request_id}-{timestamp_ms}")
    }

    fn next_ocr_region_id(&self, request_id: &str) -> String {
        let timestamp_ms = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(_) => 0,
        };
        format!("ocr-region-{request_id}-{timestamp_ms}")
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
