#[cfg(feature = "remote-openai")]
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asr::{
    AsrController, AsrRuntimeError, DEFAULT_TRANSCRIBE_DURATION_MS, MAX_TRANSCRIBE_DURATION_MS,
};
use crate::audio_io::{AudioPlaybackController, AudioPlaybackError, RuntimeAudioState};
use crate::browser::{
    BrowserController, BrowserError, BrowserPageMetrics, BrowserSessionConfig,
    BrowserVisibilityMode, LoadState,
};
use crate::commands::{
    build_planner_skill_selection, execute_planner_output, is_direct_submit_form_command,
    parse_direct_fill_and_submit_command, parse_direct_fill_field_command,
    parse_direct_focus_field_command, parse_fill_field_correction_command, planner_available_tools,
    resolve_direct_audio_command, resolve_direct_browser_visibility_command,
    resolve_direct_navigation_readback_command, resolve_direct_open_url_command,
    resolve_direct_read_page_command, resolve_direct_read_title_command,
    resolve_direct_repeat_command, resolve_direct_status_query_command,
    resolve_direct_voice_input_command, resume_after_confirmation, validate_planner_output,
    AgentStateData, AsrProviderSettings, CaptureScreenshotData, CaptureScreenshotInput,
    ClickElementData, ClickElementInput, ConfirmActionData, ConfirmActionInput,
    ConfirmActionResolution, ConfirmationSettings, DeterministicToolExecutor, EvalJsData,
    EvalJsInput, ExecutionOutcome, ExecutionTrace, ExtractPageModelData, ExtractPageModelInput,
    FillFieldCorrectionCommand, FindElementData, FindElementInput, FocusElementData,
    FocusElementInput, GetAgentStateInput, GetHtmlData, GetHtmlInput, GetPageSnapshotInput,
    GetRuntimeStatusData, GetRuntimeStatusInput, GoBackData, GoBackInput, GoForwardData,
    GoForwardInput, IntentName, IntentSummary, ListInteractiveElementsData,
    ListInteractiveElementsInput, LocalAsrModelSettings, LocalTtsModelSettings,
    MergeOcrIntoPageModelData, MergeOcrIntoPageModelInput, OcrThresholdSettings, OpenUrlData,
    OpenUrlInput, PageSnapshotData, PlannedStep, PlannerInput, PlannerOutput,
    PlannerProviderSettings, PlannerStatus, PlannerToolHistoryEntry, ProviderFailoverSettings,
    ProviderSelectionStatus, ReadNextRegionData, ReadNextRegionInput, ReadPreviousRegionData,
    ReadPreviousRegionInput, ReadRegionData, ReadRegionInput, ReloadPageData, ReloadPageInput,
    RemoteAsrSettings, RemotePlannerSettings, RemoteProviderLabel, RemoteTtsSettings,
    ReportResultData, ReportResultInput, ReportStatus, RunOcrData, RunOcrInput, ScrollPageData,
    ScrollPageInput, SetBrowserVisibilityData, SetBrowserVisibilityInput, SetPlaybackSpeedData,
    SetPlaybackSpeedInput, SetPlaybackVolumeData, SetPlaybackVolumeInput, SetTtsVoiceData,
    SetTtsVoiceInput, StartListeningData, StartListeningInput, StepTransition, StopListeningData,
    StopListeningInput, StopSpeakingData, StopSpeakingInput, SubmitActiveFormData,
    SubmitActiveFormInput, ToolError, ToolName, ToolResult, TranscribeAndExecuteCommandData,
    TranscribeCommandData, TranscribeCommandInput, TtsModelOption, TtsModelSettings,
    TtsProviderSettings, TtsVoiceOption, TtsVoiceSettings, TypeIntoElementData,
    TypeIntoElementInput,
};
#[cfg(feature = "remote-openai")]
use crate::commands::{
    canonical_planner_output_examples, planner_output_schema, tool_input_schema,
};
#[cfg(feature = "remote-openai")]
use crate::config::resolve_secret_ref;
use crate::config::{
    secret_ref_reference, AppConfig, AudioSettings, ConfigError, LocalAsrProfile, LocalTtsProfile,
    ModelManagementSettings, RemotePlannerProfile, RemoteProviderKind,
};
use crate::extractor::extract_structured_article_from_html;
use crate::narration::{
    cursor_for_index, find_region_index, next_region_index, previous_region_index,
    spoken_text_for_region,
};
use crate::ocr::{OcrController, OcrRuntimeError, OcrSettings};
use crate::page_model::PageRegion;
use crate::page_model::{ElementRole, ExtractionSource, PageModel, Rect, RegionRole, RegionSource};
use crate::state::AppState;
use crate::tts::{TtsController, TtsRuntimeError, KITTEN_TTS_VOICES, OPENAI_TTS_VOICES};
use reqwest::blocking::Client;
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingRecentFieldContext {
    target_description: Option<String>,
    active_element_id: Option<String>,
    candidate_element_ids: Vec<String>,
    pending_text: Option<String>,
    submit_after: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecentFieldContext {
    page_id: String,
    target_description: Option<String>,
    active_element_id: Option<String>,
    candidate_element_ids: Vec<String>,
    pending_text: Option<String>,
    submit_after: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct ResolvedDirectFieldCommand {
    planner_output: PlannerOutput,
    recent_field_context: Option<PendingRecentFieldContext>,
}

#[cfg(feature = "remote-openai")]
#[derive(Serialize)]
struct PlannerPromptPayload<'a> {
    planner_input: &'a PlannerInput,
    planner_output_schema: serde_json::Value,
    tool_input_schemas: BTreeMap<String, serde_json::Value>,
    canonical_planner_output_examples: BTreeMap<String, crate::commands::PlannerOutput>,
}

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

trait ReplanningRuntime {
    fn resolve_plan(
        &mut self,
        request_id: String,
        transcript: &str,
        recent_tool_results: &[PlannerToolHistoryEntry],
    ) -> Result<PlannerOutput, ToolError>;

    fn execute_plan(
        &mut self,
        request_id: String,
        planner_output: &PlannerOutput,
    ) -> ExecutionOutcome;
}

fn execution_trace_to_tool_history_entries(trace: &ExecutionTrace) -> Vec<PlannerToolHistoryEntry> {
    trace
        .tool_results
        .iter()
        .map(|result| PlannerToolHistoryEntry {
            tool_name: result.tool_name.clone(),
            ok: result.ok,
            observation_summary: result.observations.clone(),
        })
        .collect()
}

fn append_execution_trace(into: &mut ExecutionTrace, trace: ExecutionTrace) {
    into.executed_step_ids.extend(trace.executed_step_ids);
    into.tool_results.extend(trace.tool_results);
}

fn merge_execution_outcome_trace(
    mut trace: ExecutionTrace,
    outcome: ExecutionOutcome,
) -> ExecutionOutcome {
    match outcome {
        ExecutionOutcome::Complete { trace: next_trace } => {
            append_execution_trace(&mut trace, next_trace);
            ExecutionOutcome::Complete { trace }
        }
        ExecutionOutcome::AwaitingConfirmation {
            trace: next_trace,
            pending_confirmation_id,
            pending_plan_execution,
        } => {
            append_execution_trace(&mut trace, next_trace);
            ExecutionOutcome::AwaitingConfirmation {
                trace,
                pending_confirmation_id,
                pending_plan_execution,
            }
        }
        ExecutionOutcome::NeedsReplan { trace: next_trace } => {
            append_execution_trace(&mut trace, next_trace);
            ExecutionOutcome::NeedsReplan { trace }
        }
        ExecutionOutcome::Aborted {
            trace: next_trace,
            error,
        } => {
            append_execution_trace(&mut trace, next_trace);
            ExecutionOutcome::Aborted { trace, error }
        }
    }
}

fn replanning_request_id(base_request_id: &str, phase: &str, replan_cycle: usize) -> String {
    if replan_cycle == 0 {
        format!("{base_request_id}-{phase}")
    } else {
        format!("{base_request_id}-{phase}-replan-{replan_cycle}")
    }
}

fn execute_bounded_replanning_loop<R: ReplanningRuntime>(
    runtime: &mut R,
    request_id: &str,
    transcript: &str,
) -> Result<ExecutionOutcome, ToolError> {
    let mut replan_cycle = 0usize;
    let mut recent_tool_results = Vec::<PlannerToolHistoryEntry>::new();
    let mut accumulated_trace = ExecutionTrace {
        executed_step_ids: Vec::new(),
        tool_results: Vec::new(),
    };

    loop {
        let planner_output = match runtime.resolve_plan(
            replanning_request_id(request_id, "resolve", replan_cycle),
            transcript,
            &recent_tool_results,
        ) {
            Ok(planner_output) => planner_output,
            Err(error) => {
                if accumulated_trace.executed_step_ids.is_empty()
                    && accumulated_trace.tool_results.is_empty()
                {
                    return Err(error);
                }

                return Ok(ExecutionOutcome::Aborted {
                    trace: accumulated_trace,
                    error,
                });
            }
        };

        let outcome = runtime.execute_plan(
            replanning_request_id(request_id, "execute", replan_cycle),
            &planner_output,
        );
        recent_tool_results.extend(execution_trace_to_tool_history_entries(match &outcome {
            ExecutionOutcome::Complete { trace }
            | ExecutionOutcome::AwaitingConfirmation { trace, .. }
            | ExecutionOutcome::NeedsReplan { trace }
            | ExecutionOutcome::Aborted { trace, .. } => trace,
        }));

        match outcome {
            ExecutionOutcome::NeedsReplan { trace } => {
                append_execution_trace(&mut accumulated_trace, trace);
                if replan_cycle >= MAX_COMMAND_REPLAN_CYCLES {
                    return Ok(ExecutionOutcome::Aborted {
                        trace: accumulated_trace,
                        error: ToolError {
                            code: String::from("replan_limit_exceeded"),
                            message: format!(
                                "planner requested replanning more than {} time(s) for this command",
                                MAX_COMMAND_REPLAN_CYCLES
                            ),
                            retryable: true,
                            details: Some(serde_json::json!({
                                "max_replan_cycles": MAX_COMMAND_REPLAN_CYCLES,
                            })),
                        },
                    });
                }
                replan_cycle += 1;
            }
            other => {
                return Ok(merge_execution_outcome_trace(accumulated_trace, other));
            }
        }
    }
}

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

    pub fn set_playback_volume(&mut self, playback_volume: f32) -> Result<(), ConfigError> {
        let mut audio = self.config.audio.clone();
        audio.playback_volume = playback_volume;
        self.update_audio_settings(audio)
    }

    pub fn set_playback_speed(&mut self, playback_speed: f32) -> Result<(), ConfigError> {
        let mut audio = self.config.audio.clone();
        audio.playback_speed = playback_speed;
        self.update_audio_settings(audio)
    }

    pub fn set_default_tts_voice(
        &mut self,
        default_tts_voice: impl Into<String>,
    ) -> Result<(), ConfigError> {
        let mut audio = self.config.audio.clone();
        audio.default_tts_voice = default_tts_voice.into();
        self.update_audio_settings(audio)
    }

    pub fn set_active_tts_profile(
        &mut self,
        profile_name: impl Into<String>,
    ) -> Result<(), ConfigError> {
        let profile_name = profile_name.into();
        let mut selection = self.config.providers.tts.clone();
        match selection.mode {
            crate::config::ProviderMode::Local => {
                selection.local_profile = Some(profile_name);
            }
            crate::config::ProviderMode::Remote => {
                selection.remote_profile = Some(profile_name);
            }
        }

        let config =
            AppConfig::persist_tts_provider_selection_for_app(&self.app_handle, &selection)?;
        self.config = config;
        Ok(())
    }

    pub fn set_tts_provider_mode(
        &mut self,
        mode: crate::config::ProviderMode,
    ) -> Result<(), ConfigError> {
        let mut selection = self.config.providers.tts.clone();
        selection.mode = mode;

        let config =
            AppConfig::persist_tts_provider_selection_for_app(&self.app_handle, &selection)?;
        self.config = config;
        Ok(())
    }

    pub fn set_asr_provider_mode(
        &mut self,
        mode: crate::config::ProviderMode,
    ) -> Result<(), ConfigError> {
        let mut selection = self.config.providers.asr.clone();
        selection.mode = mode;

        let config =
            AppConfig::persist_asr_provider_selection_for_app(&self.app_handle, &selection)?;
        self.config = config;
        Ok(())
    }

    pub fn set_remote_planner_api_key(
        &mut self,
        profile_name: &str,
        api_key: &str,
    ) -> Result<(), ConfigError> {
        self.config = AppConfig::persist_remote_planner_api_key_for_app(
            &self.app_handle,
            profile_name,
            api_key,
        )?;
        Ok(())
    }

    pub fn set_remote_tts_api_key(
        &mut self,
        profile_name: &str,
        api_key: &str,
    ) -> Result<(), ConfigError> {
        self.config =
            AppConfig::persist_remote_tts_api_key_for_app(&self.app_handle, profile_name, api_key)?;
        Ok(())
    }

    pub fn set_remote_asr_api_key(
        &mut self,
        profile_name: &str,
        api_key: &str,
    ) -> Result<(), ConfigError> {
        self.config =
            AppConfig::persist_remote_asr_api_key_for_app(&self.app_handle, profile_name, api_key)?;
        Ok(())
    }

    pub fn current_model_management_settings(&self) -> ModelManagementSettingsData {
        build_model_management_settings(&self.config)
    }

    pub fn set_model_management_settings(
        &mut self,
        models_dir: &str,
        check_on_startup: bool,
        auto_download_missing: bool,
    ) -> Result<(), ConfigError> {
        let settings = ModelManagementSettings {
            models_dir: models_dir.trim().to_string(),
            check_on_startup,
            auto_download_missing,
        };
        self.config =
            AppConfig::persist_model_management_settings_for_app(&self.app_handle, &settings)?;
        Ok(())
    }

    pub fn download_active_local_tts_model(&mut self) -> Result<DownloadedLocalModelData, String> {
        let (profile_name, profile) = active_local_tts_profile(&self.config)?;
        let model_id = profile.model_id.clone();
        let plan = kitten_download_plan_for_model_id(&model_id)?;
        let models_dir =
            resolved_models_dir_for_app(&self.app_handle, &self.config.models.models_dir)?;
        let target_dir = models_dir.join(plan.directory_name);

        download_hugging_face_directory(&target_dir, plan.repository, plan.files)?;

        let model_path = target_dir
            .to_str()
            .ok_or_else(|| {
                format!(
                    "downloaded model path is not valid UTF-8: {}",
                    target_dir.display()
                )
            })?
            .to_string();
        self.config = AppConfig::persist_local_tts_model_path_for_app(
            &self.app_handle,
            &profile_name,
            &model_path,
        )
        .map_err(|error| error.to_string())?;

        Ok(DownloadedLocalModelData {
            profile_name,
            model_id,
            model_path,
            source_url: format!("https://huggingface.co/{}", plan.repository),
        })
    }

    pub fn download_active_local_asr_model(&mut self) -> Result<DownloadedLocalModelData, String> {
        let (profile_name, profile) = active_local_asr_profile(&self.config)?;
        let model_id = profile.model_id.clone();
        let plan = whisper_download_plan_for_model_id(&model_id)?;
        let models_dir =
            resolved_models_dir_for_app(&self.app_handle, &self.config.models.models_dir)?;
        let target_path = models_dir.join("whisper").join(plan.file_name);

        download_hugging_face_file(&target_path, plan.repository, plan.file_name)?;

        let model_path = target_path
            .to_str()
            .ok_or_else(|| {
                format!(
                    "downloaded model path is not valid UTF-8: {}",
                    target_path.display()
                )
            })?
            .to_string();
        self.config = AppConfig::persist_local_asr_model_path_for_app(
            &self.app_handle,
            &profile_name,
            &model_path,
        )
        .map_err(|error| error.to_string())?;

        Ok(DownloadedLocalModelData {
            profile_name,
            model_id,
            model_path,
            source_url: format!(
                "https://huggingface.co/{}/resolve/main/{}",
                plan.repository, plan.file_name
            ),
        })
    }

    pub fn set_browser_visibility(&mut self, mode: BrowserVisibilityMode) {
        self.state.browser_visibility = mode;
    }

    pub fn execute_open_url(&mut self, input: OpenUrlInput) -> ToolResult<OpenUrlData> {
        let final_url = match normalize_absolute_url(&input.url) {
            Ok(url) => url,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::OpenUrl,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Navigation request was rejected because the URL was not an absolute URL.",
                    )],
                )
            }
        };

        let load_state = input.wait_for_load_state.unwrap_or(LoadState::Load);
        let browser_page = match self
            .browser
            .open_url(&final_url, load_state, input.timeout_ms)
        {
            Ok(browser_page) => browser_page,
            Err(error) => {
                return self.browser_tool_failure(
                    ToolName::OpenUrl,
                    input.request_id,
                    String::from("Browser navigation did not complete successfully."),
                    error,
                )
            }
        };

        let page_id = self.next_page_id(&input.request_id);
        self.stop_narration_playback();
        self.state
            .record_navigation(page_id.clone(), browser_page.url.clone());
        if let Some(current_page) = self.state.current_page.as_mut() {
            current_page.title = browser_page.title.clone();
        }
        self.state.browser_history = browser_page.history.clone();
        self.clear_recent_field_context();

        ToolResult::success(
            ToolName::OpenUrl,
            input.request_id,
            OpenUrlData {
                final_url: browser_page.url,
                title: browser_page.title,
                page_id,
                load_state,
                http_status: None,
                history: browser_page.history,
            },
            vec![
                String::from(
                    "Validated the requested absolute URL and navigated the live Chromium page.",
                ),
                String::from(
                    "Runtime navigation state now reflects the live browser URL and document title.",
                ),
            ],
        )
    }

    pub fn execute_go_back(&mut self, input: GoBackInput) -> ToolResult<GoBackData> {
        if self.state.current_page_id.is_none() {
            return Self::browser_runtime_missing_page(ToolName::GoBack, input.request_id);
        }

        let requested_steps = input.steps.unwrap_or(1).clamp(1, MAX_HISTORY_STEPS);
        let load_state = input.wait_for_load_state.unwrap_or(LoadState::Load);
        let browser_navigation =
            match self
                .browser
                .go_back(requested_steps, load_state, input.timeout_ms)
            {
                Ok(browser_navigation) => browser_navigation,
                Err(error) => {
                    return self.browser_tool_failure(
                        ToolName::GoBack,
                        input.request_id,
                        String::from(
                            "Live browser history navigation did not complete successfully.",
                        ),
                        error,
                    )
                }
            };

        self.state.browser_history = browser_navigation.history.clone();
        if browser_navigation.navigated {
            self.stop_narration_playback();
            refresh_current_page_after_navigation(
                &mut self.state.current_page,
                browser_navigation.url.clone(),
                browser_navigation.title.clone(),
            );
            clear_navigation_follow_up_state(&mut self.state, &mut self.recent_field_context);
        }

        let mut observations = vec![format!(
            "Requested backward history navigation for up to {} step(s).",
            requested_steps
        )];
        if input.steps.is_some_and(|steps| steps > MAX_HISTORY_STEPS) {
            observations.push(format!(
                "Requested steps were clamped to the supported maximum of {}.",
                MAX_HISTORY_STEPS
            ));
        }
        observations.push(if browser_navigation.navigated {
            String::from("The live browser moved backward in history and runtime page metadata was refreshed.")
        } else {
            String::from("The live browser was already at the earliest reachable history entry.")
        });

        ToolResult::success(
            ToolName::GoBack,
            input.request_id,
            GoBackData {
                navigated: browser_navigation.navigated,
                actual_steps: if browser_navigation.navigated {
                    requested_steps
                } else {
                    0
                },
                final_url: browser_navigation.url,
                title: browser_navigation.title,
                load_state: browser_navigation.navigated.then_some(load_state),
                history: browser_navigation.history,
            },
            observations,
        )
    }

    pub fn execute_go_forward(&mut self, input: GoForwardInput) -> ToolResult<GoForwardData> {
        if self.state.current_page_id.is_none() {
            return Self::browser_runtime_missing_page(ToolName::GoForward, input.request_id);
        }

        let requested_steps = input.steps.unwrap_or(1).clamp(1, MAX_HISTORY_STEPS);
        let load_state = input.wait_for_load_state.unwrap_or(LoadState::Load);
        let browser_navigation =
            match self
                .browser
                .go_forward(requested_steps, load_state, input.timeout_ms)
            {
                Ok(browser_navigation) => browser_navigation,
                Err(error) => {
                    return self.browser_tool_failure(
                        ToolName::GoForward,
                        input.request_id,
                        String::from(
                            "Live browser forward navigation did not complete successfully.",
                        ),
                        error,
                    )
                }
            };

        self.state.browser_history = browser_navigation.history.clone();
        if browser_navigation.navigated {
            self.stop_narration_playback();
            refresh_current_page_after_navigation(
                &mut self.state.current_page,
                browser_navigation.url.clone(),
                browser_navigation.title.clone(),
            );
            clear_navigation_follow_up_state(&mut self.state, &mut self.recent_field_context);
        }

        let mut observations = vec![format!(
            "Requested forward history navigation for up to {} step(s).",
            requested_steps
        )];
        if input.steps.is_some_and(|steps| steps > MAX_HISTORY_STEPS) {
            observations.push(format!(
                "Requested steps were clamped to the supported maximum of {}.",
                MAX_HISTORY_STEPS
            ));
        }
        observations.push(if browser_navigation.navigated {
            String::from("The live browser moved forward in history and runtime page metadata was refreshed.")
        } else {
            String::from("The live browser was already at the latest reachable history entry.")
        });

        ToolResult::success(
            ToolName::GoForward,
            input.request_id,
            GoForwardData {
                navigated: browser_navigation.navigated,
                actual_steps: if browser_navigation.navigated {
                    requested_steps
                } else {
                    0
                },
                final_url: browser_navigation.url,
                title: browser_navigation.title,
                load_state: browser_navigation.navigated.then_some(load_state),
                history: browser_navigation.history,
            },
            observations,
        )
    }

    pub fn execute_reload_page(&mut self, input: ReloadPageInput) -> ToolResult<ReloadPageData> {
        if self.state.current_page_id.is_none() {
            return Self::browser_runtime_missing_page(ToolName::ReloadPage, input.request_id);
        }

        let load_state = input.wait_for_load_state.unwrap_or(LoadState::Load);
        let browser_page = match self.browser.reload_page(
            input.mode.uses_cache_bypass(),
            load_state,
            input.timeout_ms,
        ) {
            Ok(browser_page) => browser_page,
            Err(error) => {
                return self.browser_tool_failure(
                    ToolName::ReloadPage,
                    input.request_id,
                    String::from("Live browser reload did not complete successfully."),
                    error,
                )
            }
        };

        self.state.browser_history = browser_page.history.clone();
        self.stop_narration_playback();
        refresh_current_page_after_navigation(
            &mut self.state.current_page,
            Some(browser_page.url.clone()),
            browser_page.title.clone(),
        );
        clear_navigation_follow_up_state(&mut self.state, &mut self.recent_field_context);

        let mut observations = vec![String::from(
            "Reloaded the live browser page and refreshed runtime page metadata.",
        )];
        if input.mode.uses_cache_bypass() {
            observations.push(String::from(
                "The reload ignored browser cache as requested.",
            ));
        }

        ToolResult::success(
            ToolName::ReloadPage,
            input.request_id,
            ReloadPageData {
                reloaded: true,
                final_url: browser_page.url,
                title: browser_page.title,
                load_state,
                http_status: None,
                history: browser_page.history,
            },
            observations,
        )
    }

    pub fn execute_get_html(&mut self, input: GetHtmlInput) -> ToolResult<GetHtmlData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return Self::browser_runtime_missing_page(ToolName::GetHtml, input.request_id);
        };

        let browser_html = match self.browser.get_html(input.timeout_ms) {
            Ok(browser_html) => browser_html,
            Err(error) => {
                return self.browser_tool_failure(
                    ToolName::GetHtml,
                    input.request_id,
                    String::from("Live browser HTML retrieval did not complete successfully."),
                    error,
                )
            }
        };

        if let Some(current_page) = self.state.current_page.as_mut() {
            current_page.url = Some(browser_html.url.clone());
            current_page.title = browser_html.title.clone();
        }
        self.state.browser_history = browser_html.history.clone();

        let html_length = browser_html.html.len();
        ToolResult::success(
            ToolName::GetHtml,
            input.request_id,
            GetHtmlData {
                page_id,
                url: browser_html.url,
                title: browser_html.title,
                html: browser_html.html,
                html_length,
            },
            vec![
                String::from("Read the live browser document HTML from the current active page."),
                String::from(
                    "Runtime page metadata was refreshed from the live browser without altering the current page model.",
                ),
            ],
        )
    }

    pub fn execute_eval_js(&mut self, input: EvalJsInput) -> ToolResult<EvalJsData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return Self::browser_runtime_missing_page(ToolName::EvalJs, input.request_id);
        };

        let browser_eval = match self
            .browser
            .eval_js(input.expression.trim(), input.timeout_ms)
        {
            Ok(browser_eval) => browser_eval,
            Err(error) => {
                return self.browser_tool_failure(
                    ToolName::EvalJs,
                    input.request_id,
                    String::from(
                        "Live browser JavaScript evaluation did not complete successfully.",
                    ),
                    error,
                )
            }
        };

        if let Some(current_page) = self.state.current_page.as_mut() {
            current_page.url = Some(browser_eval.url.clone());
            current_page.title = browser_eval.title.clone();
        }
        self.state.browser_history = browser_eval.history.clone();

        ToolResult::success(
            ToolName::EvalJs,
            input.request_id,
            EvalJsData {
                page_id,
                url: browser_eval.url,
                title: browser_eval.title,
                result: browser_eval.result,
            },
            vec![
                String::from(
                    "Evaluated the requested JavaScript expression against the live browser page.",
                ),
                String::from(
                    "Runtime page metadata was refreshed from the live browser, but page-model regions were not automatically re-extracted.",
                ),
            ],
        )
    }

    pub fn execute_scroll_page(&mut self, input: ScrollPageInput) -> ToolResult<ScrollPageData> {
        if self.state.current_page_id.is_none() {
            return Self::browser_runtime_missing_page(ToolName::ScrollPage, input.request_id);
        }

        if input.amount_px.is_none() && input.target.is_none() {
            return ToolResult::failure(
                ToolName::ScrollPage,
                input.request_id,
                ToolError {
                    code: String::from("invalid_scroll_request"),
                    message: String::from(
                        "scroll_page requires amount_px or target to be provided",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Scroll request was rejected because it did not specify an amount or a target.",
                )],
            );
        }

        let clamped_amount = input
            .amount_px
            .map(|amount| amount.clamp(0.0, MAX_SCROLL_AMOUNT_PX));
        let scroll_state = match self.browser.scroll_page(
            input.direction,
            clamped_amount,
            input.target,
            input.timeout_ms,
        ) {
            Ok(scroll_state) => scroll_state,
            Err(error) => {
                return self.browser_tool_failure(
                    ToolName::ScrollPage,
                    input.request_id,
                    String::from("Live browser scrolling did not complete successfully."),
                    error,
                )
            }
        };

        let mut observations = vec![String::from("Executed a live browser scroll request.")];
        if input
            .amount_px
            .zip(clamped_amount)
            .is_some_and(|(requested, clamped)| (requested - clamped).abs() > f32::EPSILON)
        {
            observations.push(format!(
                "Requested scroll amount was clamped to the supported maximum of {} px.",
                MAX_SCROLL_AMOUNT_PX
            ));
        }
        if scroll_state.reached_boundary {
            observations.push(String::from(
                "The scroll request reached a document boundary.",
            ));
        }

        ToolResult::success(
            ToolName::ScrollPage,
            input.request_id,
            ScrollPageData {
                previous_scroll_y: scroll_state.previous_scroll_y,
                current_scroll_y: scroll_state.current_scroll_y,
                reached_boundary: scroll_state.reached_boundary,
            },
            observations,
        )
    }

    pub fn execute_capture_screenshot(
        &mut self,
        input: CaptureScreenshotInput,
    ) -> ToolResult<CaptureScreenshotData> {
        if self.state.current_page_id.is_none() {
            return Self::browser_runtime_missing_page(
                ToolName::CaptureScreenshot,
                input.request_id,
            );
        }

        let region_id = input
            .region_id
            .as_deref()
            .map(str::trim)
            .filter(|region_id| !region_id.is_empty())
            .map(ToOwned::to_owned);
        let region_id_active = input.region_id.as_deref().is_some();
        let targeting_modes = usize::from(input.scope.captures_full_page())
            + usize::from(region_id_active)
            + usize::from(input.bbox.is_some());
        if targeting_modes > 1 {
            return ToolResult::failure(
                ToolName::CaptureScreenshot,
                input.request_id,
                ToolError {
                    code: String::from("invalid_screenshot_target"),
                    message: String::from(
                        "capture_screenshot accepts at most one targeting mode from scope = FullPage, region_id, or bbox",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Screenshot request was rejected because it combined multiple targeting modes.",
                )],
            );
        }

        if let Some(bbox) = input.bbox.as_ref() {
            if bbox.width <= 0.0 || bbox.height <= 0.0 {
                return ToolResult::failure(
                    ToolName::CaptureScreenshot,
                    input.request_id,
                    ToolError {
                        code: String::from("invalid_screenshot_bbox"),
                        message: String::from(
                            "capture_screenshot bbox requires positive width and height",
                        ),
                        retryable: false,
                        details: Some(serde_json::json!({
                            "x": bbox.x,
                            "y": bbox.y,
                            "width": bbox.width,
                            "height": bbox.height,
                        })),
                    },
                    vec![String::from(
                        "Screenshot request was rejected because the requested bbox was not positive-sized.",
                    )],
                );
            }
        }

        let screenshot_bbox = if let Some(region_id) = region_id.as_deref() {
            let regions = match self.readable_regions() {
                Ok(regions) => regions,
                Err(error) => {
                    return ToolResult::failure(
                        ToolName::CaptureScreenshot,
                        input.request_id,
                        error,
                        vec![String::from(
                            "Region-targeted screenshot capture requires readable regions in the current page model.",
                        )],
                    )
                }
            };

            let bbox = match region_bbox_by_id(regions, region_id) {
                Ok(bbox) => bbox,
                Err(error) => {
                    return ToolResult::failure(
                        ToolName::CaptureScreenshot,
                        input.request_id,
                        error,
                        vec![String::from(
                            "Region-targeted screenshot capture could not resolve a usable bounding box for the requested region.",
                        )],
                    )
                }
            };

            Some(bbox)
        } else {
            input.bbox.clone()
        };

        let browser_screenshot = match self.browser.capture_screenshot(
            input.scope.captures_full_page(),
            screenshot_bbox.clone(),
            input.timeout_ms,
        ) {
            Ok(browser_screenshot) => browser_screenshot,
            Err(error) => {
                return self.browser_tool_failure(
                    ToolName::CaptureScreenshot,
                    input.request_id,
                    String::from("Live browser screenshot capture did not complete successfully."),
                    error,
                )
            }
        };

        if let Some(current_page) = self.state.current_page.as_mut() {
            current_page.url = Some(browser_screenshot.url.clone());
            current_page.title = browser_screenshot.title.clone();
        }
        self.state.browser_history = browser_screenshot.history.clone();

        let image_id = self.next_image_id(&input.request_id);
        let screenshot_path = match self.screenshot_output_path(&image_id) {
            Ok(path) => path,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::CaptureScreenshot,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Screenshot capture completed, but the image could not be persisted to app storage.",
                    )],
                )
            }
        };
        if let Err(error) = fs::write(&screenshot_path, &browser_screenshot.image_bytes) {
            return ToolResult::failure(
                ToolName::CaptureScreenshot,
                input.request_id,
                ToolError {
                    code: String::from("screenshot_write_failed"),
                    message: String::from(
                        "capture_screenshot could not write the PNG file to app storage",
                    ),
                    retryable: true,
                    details: Some(serde_json::json!({
                        "path": screenshot_path.display().to_string(),
                        "reason": error.to_string(),
                    })),
                },
                vec![String::from(
                    "Screenshot capture completed, but writing the PNG file to disk failed.",
                )],
            );
        }

        let mut observations = vec![format!(
            "Captured a deterministic browser screenshot and persisted it as {image_id}.png."
        )];
        if input.scope.captures_full_page() {
            observations.push(String::from(
                "The screenshot targeted the full page rather than only the current viewport.",
            ));
        } else if region_id.is_some() {
            observations.push(String::from(
                "The screenshot was clipped to the requested page region using its stored bounding box.",
            ));
        } else if browser_screenshot.bbox.is_some() {
            observations.push(String::from(
                "The screenshot was clipped to the explicitly requested bounding box.",
            ));
        } else {
            observations.push(String::from(
                "The screenshot used the current viewport because no full-page or bbox target was requested.",
            ));
        }

        ToolResult::success(
            ToolName::CaptureScreenshot,
            input.request_id,
            CaptureScreenshotData {
                image_id,
                path: screenshot_path.display().to_string(),
                bbox: browser_screenshot.bbox,
                width: browser_screenshot.width,
                height: browser_screenshot.height,
            },
            observations,
        )
    }

    pub fn execute_run_ocr(&mut self, input: RunOcrInput) -> ToolResult<RunOcrData> {
        let image_id = input
            .image_id
            .as_deref()
            .map(str::trim)
            .filter(|image_id| !image_id.is_empty())
            .map(ToOwned::to_owned);
        let region_id = input
            .region_id
            .as_deref()
            .map(str::trim)
            .filter(|region_id| !region_id.is_empty())
            .map(ToOwned::to_owned);

        if image_id.is_none() && region_id.is_none() && input.bbox.is_none() {
            return ToolResult::failure(
                ToolName::RunOcr,
                input.request_id,
                ToolError {
                    code: String::from("invalid_ocr_request"),
                    message: String::from(
                        "run_ocr requires at least one source from image_id, region_id, or bbox",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "OCR request was rejected because it did not identify any image or target area.",
                )],
            );
        }

        if let Some(bbox) = input.bbox.as_ref() {
            if bbox.width <= 0.0 || bbox.height <= 0.0 {
                return ToolResult::failure(
                    ToolName::RunOcr,
                    input.request_id,
                    ToolError {
                        code: String::from("invalid_ocr_bbox"),
                        message: String::from("run_ocr bbox requires positive width and height"),
                        retryable: false,
                        details: Some(serde_json::json!({
                            "x": bbox.x,
                            "y": bbox.y,
                            "width": bbox.width,
                            "height": bbox.height,
                        })),
                    },
                    vec![String::from(
                        "OCR request was rejected because the requested bbox was not positive-sized.",
                    )],
                );
            }
        }

        let ocr_bbox = if let Some(region_id) = region_id.as_deref() {
            let regions = match self.readable_regions() {
                Ok(regions) => regions,
                Err(error) => {
                    return ToolResult::failure(
                        ToolName::RunOcr,
                        input.request_id,
                        error,
                        vec![String::from(
                        "Region-targeted OCR requires readable regions in the current page model.",
                    )],
                    )
                }
            };

            match region_bbox_by_id(regions, region_id) {
                Ok(bbox) => Some(bbox),
                Err(error) => {
                    return ToolResult::failure(
                        ToolName::RunOcr,
                        input.request_id,
                        error,
                        vec![String::from(
                            "Region-targeted OCR could not resolve a usable bounding box for the requested region.",
                        )],
                    )
                }
            }
        } else {
            input.bbox.clone()
        };

        let Some(image_id) = image_id else {
            return ToolResult::failure(
                ToolName::RunOcr,
                input.request_id,
                ToolError {
                    code: String::from("missing_ocr_image_id"),
                    message: String::from(
                        "run_ocr currently requires image_id so it can resolve a persisted screenshot",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "OCR needs a persisted screenshot image_id because implicit image selection is not supported.",
                )],
            );
        };

        let image_path = match self.cached_image_path(&image_id) {
            Ok(path) => path,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::RunOcr,
                    input.request_id,
                    error,
                    vec![String::from(
                        "OCR could not resolve the cached screenshot path.",
                    )],
                )
            }
        };

        if !image_path.is_file() {
            return ToolResult::failure(
                ToolName::RunOcr,
                input.request_id,
                ToolError {
                    code: String::from("ocr_image_not_found"),
                    message: String::from(
                        "run_ocr could not find the cached screenshot for the requested image_id",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({
                        "image_id": image_id,
                        "path": image_path.display().to_string(),
                    })),
                },
                vec![String::from(
                    "OCR could not start because the requested cached screenshot does not exist.",
                )],
            );
        }

        let ocr_result = match self.ocr.run_ocr(&image_path, ocr_bbox.as_ref()) {
            Ok(result) => result,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::RunOcr,
                    input.request_id,
                    ocr_runtime_error_to_tool_error(&error),
                    vec![String::from(
                        "OCR could not extract text from the requested screenshot.",
                    )],
                )
            }
        };

        let mut observations = vec![String::from(
            "Ran deterministic OCR on the requested cached screenshot.",
        )];
        if region_id.is_some() {
            observations.push(String::from(
                "OCR was limited to the requested page region using its stored bounding box.",
            ));
        } else if ocr_bbox.is_some() {
            observations.push(String::from(
                "OCR was limited to the explicitly requested bounding box within the cached image.",
            ));
        } else {
            observations.push(String::from(
                "OCR used the full cached screenshot because no bbox override was provided.",
            ));
        }
        if ocr_result.extracted_text.is_empty() {
            observations.push(String::from(
                "OCR completed successfully but did not extract any readable text.",
            ));
        }

        let extracted_text = ocr_result.extracted_text;
        let text_length = extracted_text.len();

        ToolResult::success(
            ToolName::RunOcr,
            input.request_id,
            RunOcrData {
                image_id: Some(image_id),
                extracted_text,
                text_length,
                confidence: ocr_result.confidence,
                source_bbox: ocr_bbox,
            },
            observations,
        )
    }

    pub fn execute_merge_ocr_into_page_model(
        &mut self,
        input: MergeOcrIntoPageModelInput,
    ) -> ToolResult<MergeOcrIntoPageModelData> {
        let requested_page_id = input.page_id.trim().to_string();
        if requested_page_id.is_empty() {
            return ToolResult::failure(
                ToolName::MergeOcrIntoPageModel,
                input.request_id,
                ToolError {
                    code: String::from("invalid_page_id"),
                    message: String::from("merge_ocr_into_page_model requires a non-empty page_id"),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "OCR merge was rejected because it did not identify a target page.",
                )],
            );
        }

        let Some(active_page_id) = self.state.current_page_id.as_deref() else {
            return ToolResult::failure(
                ToolName::MergeOcrIntoPageModel,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from(
                        "merge_ocr_into_page_model requires an active page in runtime state",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({ "page_id": requested_page_id })),
                },
                vec![String::from(
                    "OCR merge could not run because no active page is loaded in runtime state.",
                )],
            );
        };

        if active_page_id != requested_page_id {
            return ToolResult::failure(
                ToolName::MergeOcrIntoPageModel,
                input.request_id,
                ToolError {
                    code: String::from("page_id_mismatch"),
                    message: String::from(
                        "merge_ocr_into_page_model page_id must match the active runtime page",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({
                        "active_page_id": active_page_id,
                        "page_id": requested_page_id,
                    })),
                },
                vec![String::from(
                    "OCR merge was rejected because it targeted a page that is not currently active.",
                )],
            );
        }

        let normalized_ocr_text = input.ocr_text.trim().to_string();
        if normalized_ocr_text.is_empty() {
            return ToolResult::failure(
                ToolName::MergeOcrIntoPageModel,
                input.request_id,
                ToolError {
                    code: String::from("invalid_ocr_text"),
                    message: String::from("merge_ocr_into_page_model requires non-empty ocr_text"),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "OCR merge was rejected because there was no recognized text to merge.",
                )],
            );
        }

        if let Some(bbox) = input.source_bbox.as_ref() {
            if bbox.width <= 0.0 || bbox.height <= 0.0 {
                return ToolResult::failure(
                    ToolName::MergeOcrIntoPageModel,
                    input.request_id,
                    ToolError {
                        code: String::from("invalid_source_bbox"),
                        message: String::from(
                            "merge_ocr_into_page_model source_bbox requires positive width and height",
                        ),
                        retryable: false,
                        details: Some(serde_json::json!({
                            "x": bbox.x,
                            "y": bbox.y,
                            "width": bbox.width,
                            "height": bbox.height,
                        })),
                    },
                    vec![String::from(
                        "OCR merge was rejected because the supplied source bounding box was invalid.",
                    )],
                );
            }
        }

        let next_region_id = self.next_ocr_region_id(&input.request_id);
        let requested_region_id = input
            .region_id
            .as_deref()
            .map(str::trim)
            .filter(|region_id| !region_id.is_empty())
            .map(ToOwned::to_owned);

        let merge_outcome = {
            let Some(current_page) = self.state.current_page.as_mut() else {
                return ToolResult::failure(
                    ToolName::MergeOcrIntoPageModel,
                    input.request_id,
                    ToolError {
                        code: String::from("missing_page_model"),
                        message: String::from(
                            "merge_ocr_into_page_model requires runtime page data for the active page",
                        ),
                        retryable: false,
                        details: Some(serde_json::json!({ "page_id": requested_page_id })),
                    },
                    vec![String::from(
                        "OCR merge could not update the page because the runtime page model is missing.",
                    )],
                );
            };

            merge_ocr_text_into_page_model(
                current_page,
                requested_region_id.as_deref(),
                &normalized_ocr_text,
                input.source_bbox.clone(),
                next_region_id,
            )
        };

        let updated_region_ids = match merge_outcome {
            Ok(updated_region_ids) => updated_region_ids,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::MergeOcrIntoPageModel,
                    input.request_id,
                    error,
                    vec![String::from(
                        "OCR merge could not apply the recognized text to the runtime page model.",
                    )],
                )
            }
        };

        let mut observations = vec![String::from(
            "Merged OCR text into the active runtime page model.",
        )];
        if requested_region_id.is_some() {
            observations.push(String::from(
                "OCR text updated an existing page region and marked it as mixed DOM/OCR content.",
            ));
        } else {
            observations.push(String::from(
                "OCR text was added as a new OCR region because no existing target region was supplied.",
            ));
        }

        ToolResult::success(
            ToolName::MergeOcrIntoPageModel,
            input.request_id,
            MergeOcrIntoPageModelData {
                page_id: requested_page_id,
                updated_region_ids,
                merged_text_length: normalized_ocr_text.len(),
            },
            observations,
        )
    }

    pub fn execute_get_page_snapshot(
        &mut self,
        input: GetPageSnapshotInput,
    ) -> ToolResult<PageSnapshotData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return ToolResult::failure(
                ToolName::GetPageSnapshot,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from(
                        "get_page_snapshot requires an active page in runtime state",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Page snapshot could not be created because no page has been opened yet.",
                )],
            );
        };

        let Some(current_page) = self.state.current_page.as_ref() else {
            return ToolResult::failure(
                ToolName::GetPageSnapshot,
                input.request_id,
                ToolError {
                    code: String::from("missing_page_model"),
                    message: String::from(
                        "get_page_snapshot requires runtime page data for the active page",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({ "page_id": page_id })),
                },
                vec![String::from(
                    "Page snapshot could not be created because the runtime page model is missing.",
                )],
            );
        };

        let Some(url) = current_page.url.clone() else {
            return ToolResult::failure(
                ToolName::GetPageSnapshot,
                input.request_id,
                ToolError {
                    code: String::from("missing_page_url"),
                    message: String::from(
                        "get_page_snapshot requires a current page URL in runtime state",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({ "page_id": page_id })),
                },
                vec![String::from(
                    "Page snapshot could not be created because the runtime page URL is missing.",
                )],
            );
        };

        let title = current_page.title.clone();
        let visible_text_excerpt =
            build_visible_text_excerpt(current_page, input.text_excerpt_max_chars);
        let interactive_elements = if input.include_interactive_elements {
            current_page.interactive_elements.clone()
        } else {
            Vec::new()
        };
        let page_metrics = match self.browser.get_page_metrics() {
            Ok(page_metrics) => page_metrics,
            Err(error) => return self.browser_tool_failure(
                ToolName::GetPageSnapshot,
                input.request_id,
                String::from(
                    "Live page snapshot metrics could not be read from the active browser page.",
                ),
                error,
            ),
        };

        ToolResult::success(
            ToolName::GetPageSnapshot,
            input.request_id,
            PageSnapshotData {
                page_id,
                url,
                title,
                visible_text_excerpt,
                interactive_elements,
                scroll_y: page_metrics.scroll_y,
                viewport_width: page_metrics.viewport_width,
                viewport_height: page_metrics.viewport_height,
                document_height: page_metrics.document_height,
            },
            vec![
                String::from(
                    "Built a deterministic page snapshot from the current runtime page state.",
                ),
                String::from(
                    "Included live scroll and viewport metrics from the active browser page.",
                ),
            ],
        )
    }

    pub fn execute_extract_page_model(
        &mut self,
        input: ExtractPageModelInput,
    ) -> ToolResult<ExtractPageModelData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return ToolResult::failure(
                ToolName::ExtractPageModel,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from(
                        "extract_page_model requires an active page in runtime state",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Page model extraction could not run because no page has been opened yet.",
                )],
            );
        };

        let mut used_dom_smoothie = false;
        let mut dom_smoothie_fallback_reason: Option<String> = None;
        let base_page_model = if input.use_dom_extraction {
            let browser_page_model = match self.browser.extract_page_model() {
                Ok(extracted_page_model) => extracted_page_model,
                Err(error) => {
                    return self.browser_tool_failure(
                        ToolName::ExtractPageModel,
                        input.request_id,
                        String::from(
                            "Live browser page-model extraction did not complete successfully.",
                        ),
                        error,
                    )
                }
            };

            let extracted_page_model = match self.browser.get_html(input.timeout_ms) {
                Ok(browser_html) => match extract_structured_article_from_html(
                    &browser_html.html,
                    browser_page_model.url.as_deref(),
                    browser_page_model.interactive_elements.clone(),
                ) {
                    Ok(extracted_article) => {
                        used_dom_smoothie = true;
                        extracted_article.into_page_model()
                    }
                    Err(error) => {
                        dom_smoothie_fallback_reason = Some(error.to_string());
                        tracing::warn!(error = %error, "dom_smoothie extraction fell back to browser DOM model");
                        browser_page_model
                    }
                },
                Err(error) => {
                    dom_smoothie_fallback_reason = Some(error.to_string());
                    tracing::warn!(error = %error, "HTML retrieval failed during dom_smoothie extraction; falling back to browser DOM model");
                    browser_page_model
                }
            };

            self.state.current_page = Some(extracted_page_model.clone());
            extracted_page_model
        } else {
            let Some(current_page) = self.state.current_page.as_ref() else {
                return ToolResult::failure(
                    ToolName::ExtractPageModel,
                    input.request_id,
                    ToolError {
                        code: String::from("missing_page_model"),
                        message: String::from(
                            "extract_page_model requires runtime page data for the active page",
                        ),
                        retryable: false,
                        details: Some(serde_json::json!({ "page_id": page_id })),
                    },
                    vec![String::from(
                        "Page model extraction could not run because the runtime page model is missing.",
                    )],
                );
            };

            current_page.clone()
        };

        let mut observations = if input.use_dom_extraction {
            if used_dom_smoothie {
                vec![String::from(
                    "Built a deterministic page model from live HTML using dom_smoothie and persisted it into runtime state.",
                )]
            } else {
                let mut observations = vec![String::from(
                    "dom_smoothie extraction fell back to the live Chromium DOM model.",
                )];
                if let Some(reason) = dom_smoothie_fallback_reason.as_deref() {
                    observations.push(format!("dom_smoothie fallback reason: {reason}"));
                }
                observations.push(String::from(
                    "Built a deterministic page model from the live Chromium DOM and persisted it into runtime state.",
                ));
                observations
            }
        } else {
            vec![String::from(
                "Built a deterministic page model from the current runtime page state.",
            )]
        };
        if !input.include_links {
            observations.push(String::from(
                "Link elements were omitted from the extracted page model as requested.",
            ));
        }
        if input.include_headings {
            observations.push(String::from(
                "Heading-specific extraction is not yet distinguished in the current page model schema, so regions were returned unchanged.",
            ));
        }
        if !input.use_dom_extraction {
            observations.push(String::from(
                "A non-DOM extraction request currently reuses runtime page state until OCR-specific extraction is wired.",
            ));
        }

        if should_trigger_extract_page_model_ocr_fallback(
            input.use_dom_extraction,
            &base_page_model,
            &self.config.ocr,
        ) {
            observations.push(String::from(
                "Live DOM extraction did not produce enough readable text, so OCR fallback was triggered.",
            ));

            let screenshot_result = self.execute_capture_screenshot(CaptureScreenshotInput {
                request_id: format!("{}-ocr-fallback-screenshot", input.request_id),
                timeout_ms: input.timeout_ms,
                scope: crate::commands::ScreenshotScope::FullPage,
                region_id: None,
                bbox: None,
            });
            if !screenshot_result.ok {
                return nested_tool_failure_as_extract_page_model(
                    input.request_id,
                    observations,
                    screenshot_result,
                    String::from(
                        "OCR fallback could not capture a screenshot for page-model recovery.",
                    ),
                );
            }
            observations.extend(screenshot_result.observations.clone());
            let Some(screenshot_data) = screenshot_result.data else {
                return extract_page_model_internal_failure(
                    input.request_id,
                    String::from("OCR fallback screenshot result did not include screenshot data"),
                    observations,
                );
            };

            let region_first_targets =
                region_first_ocr_target_ids(&base_page_model, &self.config.ocr);
            let had_region_first_targets = !region_first_targets.is_empty();
            let mut recovered_region_text = false;

            if !region_first_targets.is_empty() {
                observations.push(format!(
                    "OCR fallback is trying {} bbox-backed readable regions before broader OCR.",
                    region_first_targets.len()
                ));

                for region_id in region_first_targets {
                    let ocr_result = self.execute_run_ocr(RunOcrInput {
                        request_id: format!("{}-ocr-fallback-run-{region_id}", input.request_id),
                        timeout_ms: input.timeout_ms,
                        image_id: Some(screenshot_data.image_id.clone()),
                        region_id: Some(region_id.clone()),
                        bbox: None,
                    });
                    observations.extend(ocr_result.observations.clone());

                    if !ocr_result.ok {
                        let failure_code = ocr_result
                            .error
                            .as_ref()
                            .map(|error| error.code.as_str())
                            .unwrap_or("unknown_ocr_error");
                        observations.push(format!(
                            "Region-first OCR for region_id={region_id} did not succeed ({failure_code}), so broader OCR may still be needed."
                        ));
                        continue;
                    }

                    let Some(ocr_data) = ocr_result.data else {
                        return extract_page_model_internal_failure(
                            input.request_id,
                            String::from("Region-first OCR result did not include OCR data"),
                            observations,
                        );
                    };

                    if ocr_data.extracted_text.is_empty() {
                        observations.push(format!(
                            "Region-first OCR completed for region_id={region_id}, but it did not recover readable text."
                        ));
                        continue;
                    }

                    let merge_result =
                        self.execute_merge_ocr_into_page_model(MergeOcrIntoPageModelInput {
                            request_id: format!(
                                "{}-ocr-fallback-merge-{region_id}",
                                input.request_id
                            ),
                            timeout_ms: input.timeout_ms,
                            page_id: page_id.clone(),
                            region_id: Some(region_id.clone()),
                            ocr_text: ocr_data.extracted_text,
                            source_bbox: ocr_data.source_bbox,
                        });
                    if !merge_result.ok {
                        return nested_tool_failure_as_extract_page_model(
                            input.request_id,
                            observations,
                            merge_result,
                            String::from(
                                "Region-first OCR recovered text, but merging it into the page model failed.",
                            ),
                        );
                    }
                    observations.extend(merge_result.observations);
                    observations.push(format!(
                        "Region-first OCR recovered readable text for region_id={region_id}."
                    ));
                    recovered_region_text = true;
                }
            }

            if !recovered_region_text {
                if self.config.ocr.prefer_region_ocr {
                    if !had_region_first_targets {
                        observations.push(String::from(
                            "Region-first OCR was unavailable because no bbox-backed readable regions were present, so fallback widened to the full-page screenshot.",
                        ));
                    } else {
                        observations.push(String::from(
                            "Region-first OCR did not recover enough text, so fallback widened to the full-page screenshot.",
                        ));
                    }
                }

                let ocr_result = self.execute_run_ocr(RunOcrInput {
                    request_id: format!("{}-ocr-fallback-run", input.request_id),
                    timeout_ms: input.timeout_ms,
                    image_id: Some(screenshot_data.image_id.clone()),
                    region_id: None,
                    bbox: None,
                });
                if !ocr_result.ok {
                    return nested_tool_failure_as_extract_page_model(
                        input.request_id,
                        observations,
                        ocr_result,
                        String::from(
                            "OCR fallback could not extract readable text from the recovery screenshot.",
                        ),
                    );
                }
                observations.extend(ocr_result.observations.clone());
                let Some(ocr_data) = ocr_result.data else {
                    return extract_page_model_internal_failure(
                        input.request_id,
                        String::from("OCR fallback result did not include OCR data"),
                        observations,
                    );
                };

                if ocr_data.extracted_text.is_empty() {
                    observations.push(String::from(
                        "OCR fallback completed, but it still did not recover readable text.",
                    ));
                } else {
                    let merge_result =
                        self.execute_merge_ocr_into_page_model(MergeOcrIntoPageModelInput {
                            request_id: format!("{}-ocr-fallback-merge", input.request_id),
                            timeout_ms: input.timeout_ms,
                            page_id: page_id.clone(),
                            region_id: None,
                            ocr_text: ocr_data.extracted_text,
                            source_bbox: ocr_data.source_bbox,
                        });
                    if !merge_result.ok {
                        return nested_tool_failure_as_extract_page_model(
                            input.request_id,
                            observations,
                            merge_result,
                            String::from(
                                "OCR fallback recovered text, but merging it into the page model failed.",
                            ),
                        );
                    }
                    observations.extend(merge_result.observations);
                    observations.push(String::from(
                        "OCR fallback recovered readable text and merged it into the runtime page model.",
                    ));
                }
            }
        }

        let runtime_page_model = self
            .state
            .current_page
            .clone()
            .unwrap_or_else(|| base_page_model.clone());
        let extracted_page_model = build_extracted_page_model(&runtime_page_model, &input);
        let region_count = extracted_page_model.regions.len();
        let readable_region_count = extracted_page_model
            .regions
            .iter()
            .filter(|region| !region.text.trim().is_empty())
            .count();
        let extraction_source = infer_extraction_source(
            &runtime_page_model,
            input.use_dom_extraction,
            used_dom_smoothie,
        );

        ToolResult::success(
            ToolName::ExtractPageModel,
            input.request_id,
            ExtractPageModelData {
                page_model: extracted_page_model,
                region_count,
                readable_region_count,
                extraction_source,
            },
            observations,
        )
    }

    pub fn execute_list_interactive_elements(
        &mut self,
        input: ListInteractiveElementsInput,
    ) -> ToolResult<ListInteractiveElementsData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return ToolResult::failure(
                ToolName::ListInteractiveElements,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from(
                        "list_interactive_elements requires an active page in runtime state",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Interactive elements could not be listed because no page has been opened yet.",
                )],
            );
        };

        let Some(current_page) = self.state.current_page.as_ref() else {
            return ToolResult::failure(
                ToolName::ListInteractiveElements,
                input.request_id,
                ToolError {
                    code: String::from("missing_page_model"),
                    message: String::from(
                        "list_interactive_elements requires runtime page data for the active page",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({ "page_id": page_id })),
                },
                vec![String::from(
                    "Interactive elements could not be listed because the runtime page model is missing.",
                )],
            );
        };

        let elements = filter_interactive_elements(
            &current_page.interactive_elements,
            input.visibility_filter.visible_only(),
            input.roles.as_deref(),
        );
        let visible_count = elements.iter().filter(|element| element.visible).count();

        let mut observations = vec![String::from(
            "Listed deterministic interactive elements from the current runtime page state.",
        )];
        if input.visibility_filter.visible_only() {
            observations.push(String::from(
                "Results were filtered to elements currently marked visible in runtime state.",
            ));
        }
        if let Some(roles) = input.roles.as_ref() {
            observations.push(format!(
                "Results were filtered to {} requested role(s).",
                roles.len()
            ));
        }

        ToolResult::success(
            ToolName::ListInteractiveElements,
            input.request_id,
            ListInteractiveElementsData {
                page_id,
                elements,
                visible_count,
            },
            observations,
        )
    }

    pub fn execute_find_element(&mut self, input: FindElementInput) -> ToolResult<FindElementData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return ToolResult::failure(
                ToolName::FindElement,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from("find_element requires an active page in runtime state"),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Element search could not run because no page has been opened yet.",
                )],
            );
        };

        let Some(current_page) = self.state.current_page.as_ref() else {
            return ToolResult::failure(
                ToolName::FindElement,
                input.request_id,
                ToolError {
                    code: String::from("missing_page_model"),
                    message: String::from(
                        "find_element requires runtime page data for the active page",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({ "page_id": page_id })),
                },
                vec![String::from(
                    "Element search could not run because the runtime page model is missing.",
                )],
            );
        };

        let search_query = match build_find_element_query(&input) {
            Ok(search_query) => search_query,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::FindElement,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Element search was rejected because the search criteria were empty.",
                    )],
                )
            }
        };

        let candidate_limit = input
            .max_candidates
            .unwrap_or(DEFAULT_FIND_ELEMENT_MAX_CANDIDATES)
            .clamp(1, MAX_FIND_ELEMENT_CANDIDATES);
        let elements = filter_interactive_elements(
            &current_page.interactive_elements,
            input.visibility_filter.visible_only(),
            input.role.as_ref().map(std::slice::from_ref),
        );
        let ranked_candidates =
            rank_find_element_candidates(&elements, &search_query, candidate_limit);
        let (chosen_element_id, chosen_confidence, requires_confirmation) =
            determine_find_element_resolution(
                &ranked_candidates,
                self.config.safety.confirmation_confidence_threshold,
            );

        let mut observations = vec![format!(
            "Searched {} interactive element(s) from the current runtime page state.",
            elements.len()
        )];
        if input.visibility_filter.visible_only() {
            observations.push(String::from(
                "Search was limited to elements currently marked visible in runtime state.",
            ));
        }
        if input
            .max_candidates
            .is_some_and(|value| value > MAX_FIND_ELEMENT_CANDIDATES)
        {
            observations.push(format!(
                "Candidate count was clamped to the supported maximum of {}.",
                MAX_FIND_ELEMENT_CANDIDATES
            ));
        }
        if ranked_candidates.is_empty() {
            observations.push(String::from(
                "No interactive elements produced a positive match score for the requested search criteria.",
            ));
        } else if requires_confirmation {
            observations.push(String::from(
                "Top candidates are too close to choose deterministically, so planner clarification is required before any side effect.",
            ));
        } else {
            observations.push(String::from(
                "A single strongest candidate was identified from the filtered interactive elements.",
            ));
        }

        ToolResult::success(
            ToolName::FindElement,
            input.request_id,
            FindElementData {
                query_summary: search_query.summary,
                chosen_element_id,
                chosen_confidence,
                candidates: ranked_candidates,
                requires_confirmation,
            },
            observations,
        )
    }

    pub fn execute_click_element(
        &mut self,
        input: ClickElementInput,
    ) -> ToolResult<ClickElementData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return ToolResult::failure(
                ToolName::ClickElement,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from("click_element requires an active page in runtime state"),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Click could not run because no page has been opened yet.",
                )],
            );
        };

        let element = {
            let Some(current_page) = self.state.current_page.as_ref() else {
                return ToolResult::failure(
                    ToolName::ClickElement,
                    input.request_id,
                    ToolError {
                        code: String::from("missing_page_model"),
                        message: String::from(
                            "click_element requires runtime page data for the active page",
                        ),
                        retryable: false,
                        details: Some(serde_json::json!({ "page_id": page_id })),
                    },
                    vec![String::from(
                        "Click could not run because the runtime page model is missing.",
                    )],
                );
            };

            match resolve_clickable_element(current_page, &input.element_id) {
                Ok(element) => element.clone(),
                Err(error) => {
                    return ToolResult::failure(
                        ToolName::ClickElement,
                        input.request_id,
                        error,
                        vec![String::from(
                            "Click could not run because the requested deterministic element_id was not currently interactable.",
                        )],
                    )
                }
            }
        };

        let browser_click = match self.browser.click_element(
            &element,
            input.click_mode.is_double_click(),
            input.timeout_ms,
        ) {
            Ok(browser_click) => browser_click,
            Err(error) => {
                return self.browser_tool_failure(
                    ToolName::ClickElement,
                    input.request_id,
                    String::from("Live browser click did not complete successfully."),
                    error,
                )
            }
        };

        if browser_click.page_changed {
            let next_page_id = self.next_page_id(&input.request_id);
            self.state
                .record_navigation(next_page_id, browser_click.url.clone());
            if let Some(current_page) = self.state.current_page.as_mut() {
                current_page.title = browser_click.title.clone();
            }
        } else if let Some(current_page) = self.state.current_page.as_mut() {
            current_page.url = Some(browser_click.url.clone());
            current_page.title = browser_click.title.clone();
        }
        self.state.browser_history = browser_click.history.clone();

        let mut observations = vec![format!(
            "Triggered a live Chromium DOM click for element_id={}",
            element.element_id
        )];
        if input.click_mode.is_double_click() {
            observations.push(String::from(
                "The browser backend executed the click with a double-click count.",
            ));
        }
        if browser_click.page_changed {
            observations.push(String::from(
                "The live browser URL changed after the click, so runtime page state advanced to a new page.",
            ));
        } else {
            observations.push(String::from(
                "The click completed without a live browser navigation, so runtime state stayed on the current page.",
            ));
        }

        ToolResult::success(
            ToolName::ClickElement,
            input.request_id,
            ClickElementData {
                element_id: element.element_id.clone(),
                action_performed: true,
                page_changed: browser_click.page_changed,
                navigation_url: browser_click
                    .page_changed
                    .then_some(browser_click.url.clone()),
                resulting_title: browser_click.title,
            },
            observations,
        )
    }

    pub fn execute_focus_element(
        &mut self,
        input: FocusElementInput,
    ) -> ToolResult<FocusElementData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return ToolResult::failure(
                ToolName::FocusElement,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from("focus_element requires an active page in runtime state"),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Field focus could not run because no page has been opened yet.",
                )],
            );
        };

        let element = {
            let Some(current_page) = self.state.current_page.as_ref() else {
                return ToolResult::failure(
                    ToolName::FocusElement,
                    input.request_id,
                    ToolError {
                        code: String::from("missing_page_model"),
                        message: String::from(
                            "focus_element requires runtime page data for the active page",
                        ),
                        retryable: false,
                        details: Some(serde_json::json!({ "page_id": page_id })),
                    },
                    vec![String::from(
                        "Field focus could not run because the runtime page model is missing.",
                    )],
                );
            };

            match resolve_clickable_element(current_page, &input.element_id) {
                Ok(element) => element.clone(),
                Err(error) => {
                    return ToolResult::failure(
                        ToolName::FocusElement,
                        input.request_id,
                        error,
                        vec![String::from(
                            "Field focus could not run because the requested deterministic element_id was not currently interactable.",
                        )],
                    )
                }
            }
        };

        let browser_focus = match self.browser.focus_element(&element, input.timeout_ms) {
            Ok(browser_focus) => browser_focus,
            Err(error) => {
                return self.browser_tool_failure(
                    ToolName::FocusElement,
                    input.request_id,
                    String::from("Live browser field focus did not complete successfully."),
                    error,
                )
            }
        };

        if let Some(current_page) = self.state.current_page.as_mut() {
            current_page.url = Some(browser_focus.url.clone());
            current_page.title = browser_focus.title.clone();
        }
        self.state.browser_history = browser_focus.history.clone();
        self.update_recent_field_target(element.element_id.clone());

        ToolResult::success(
            ToolName::FocusElement,
            input.request_id,
            FocusElementData {
                element_id: element.element_id.clone(),
                focused: browser_focus.focused,
                element_role: Some(element.role),
            },
            vec![
                format!(
                    "Moved live browser focus to element_id={}.",
                    element.element_id
                ),
                String::from(
                    "The runtime page state remained on the current page after the focus change.",
                ),
            ],
        )
    }

    pub fn execute_type_into_element(
        &mut self,
        input: TypeIntoElementInput,
    ) -> ToolResult<TypeIntoElementData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return ToolResult::failure(
                ToolName::TypeIntoElement,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from(
                        "type_into_element requires an active page in runtime state",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Field entry could not run because no page has been opened yet.",
                )],
            );
        };

        let element = {
            let Some(current_page) = self.state.current_page.as_ref() else {
                return ToolResult::failure(
                    ToolName::TypeIntoElement,
                    input.request_id,
                    ToolError {
                        code: String::from("missing_page_model"),
                        message: String::from(
                            "type_into_element requires runtime page data for the active page",
                        ),
                        retryable: false,
                        details: Some(serde_json::json!({ "page_id": page_id })),
                    },
                    vec![String::from(
                        "Field entry could not run because the runtime page model is missing.",
                    )],
                );
            };

            match resolve_typeable_element(current_page, &input.element_id) {
                Ok(element) => element.clone(),
                Err(error) => {
                    return ToolResult::failure(
                        ToolName::TypeIntoElement,
                        input.request_id,
                        error,
                        vec![String::from(
                            "Field entry could not run because the requested deterministic element_id was not currently editable.",
                        )],
                    )
                }
            }
        };

        let browser_type = match self.browser.type_into_element(
            &element,
            &input.text,
            input.text_entry_mode.clears_existing_value(),
            input.submit_mode.submits_after_entry(),
            input.timeout_ms,
        ) {
            Ok(browser_type) => browser_type,
            Err(error) => {
                return self.browser_tool_failure(
                    ToolName::TypeIntoElement,
                    input.request_id,
                    String::from("Live browser text entry did not complete successfully."),
                    error,
                )
            }
        };

        if browser_type.page_changed {
            let next_page_id = self.next_page_id(&input.request_id);
            self.state
                .record_navigation(next_page_id, browser_type.url.clone());
            if let Some(current_page) = self.state.current_page.as_mut() {
                current_page.title = browser_type.title.clone();
            }
        } else if let Some(current_page) = self.state.current_page.as_mut() {
            current_page.url = Some(browser_type.url.clone());
            current_page.title = browser_type.title.clone();
            let requested_element_id = element.element_id.clone();
            if let Some(live_element) = current_page
                .interactive_elements
                .iter_mut()
                .find(|live_element| live_element.element_id == requested_element_id)
            {
                live_element.value = browser_type.value_after.clone();
            }
        }
        self.state.browser_history = browser_type.history.clone();
        if browser_type.page_changed {
            self.clear_recent_field_context();
        } else {
            self.update_recent_field_target(element.element_id.clone());
        }

        let mut observations = vec![format!(
            "Sent text entry to live element_id={}.",
            element.element_id
        )];
        if input.text_entry_mode.clears_existing_value() {
            observations.push(String::from(
                "Existing field contents were cleared before the new text was applied.",
            ));
        }
        if browser_type.page_changed {
            observations.push(String::from(
                "Submitting the field changed the live browser URL, so runtime page state advanced to a new page.",
            ));
        } else {
            observations.push(String::from(
                "The field entry completed without live browser navigation.",
            ));
        }

        ToolResult::success(
            ToolName::TypeIntoElement,
            input.request_id,
            TypeIntoElementData {
                element_id: element.element_id.clone(),
                text_length: input.text.chars().count(),
                value_after: browser_type.value_after,
                accepted_input: browser_type.accepted_input,
            },
            observations,
        )
    }

    pub fn execute_submit_active_form(
        &mut self,
        input: SubmitActiveFormInput,
    ) -> ToolResult<SubmitActiveFormData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return ToolResult::failure(
                ToolName::SubmitActiveForm,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from(
                        "submit_active_form requires an active page in runtime state",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Form submission could not run because no page has been opened yet.",
                )],
            );
        };

        let form = match input.form_element_id.as_deref() {
            Some(form_element_id) => {
                let Some(current_page) = self.state.current_page.as_ref() else {
                    return ToolResult::failure(
                        ToolName::SubmitActiveForm,
                        input.request_id,
                        ToolError {
                            code: String::from("missing_page_model"),
                            message: String::from(
                                "submit_active_form requires runtime page data for the active page",
                            ),
                            retryable: false,
                            details: Some(serde_json::json!({ "page_id": page_id })),
                        },
                        vec![String::from(
                            "Form submission could not run because the runtime page model is missing.",
                        )],
                    );
                };

                match resolve_form_element(current_page, form_element_id) {
                    Ok(form) => Some(form.clone()),
                    Err(error) => {
                        return ToolResult::failure(
                            ToolName::SubmitActiveForm,
                            input.request_id,
                            error,
                            vec![String::from(
                                "Form submission could not run because the requested form target was not currently submittable.",
                            )],
                        )
                    }
                }
            }
            None => None,
        };

        let browser_submit = match self
            .browser
            .submit_active_form(form.as_ref(), input.timeout_ms)
        {
            Ok(browser_submit) => browser_submit,
            Err(error) => {
                return self.browser_tool_failure(
                    ToolName::SubmitActiveForm,
                    input.request_id,
                    String::from("Live browser form submission did not complete successfully."),
                    error,
                )
            }
        };

        if browser_submit.page_changed {
            let next_page_id = self.next_page_id(&input.request_id);
            self.state
                .record_navigation(next_page_id, browser_submit.url.clone());
            if let Some(current_page) = self.state.current_page.as_mut() {
                current_page.title = browser_submit.title.clone();
            }
            self.clear_recent_field_context();
        } else if let Some(current_page) = self.state.current_page.as_mut() {
            current_page.url = Some(browser_submit.url.clone());
            current_page.title = browser_submit.title.clone();
        }
        self.state.browser_history = browser_submit.history.clone();

        let mut observations = vec![String::from(
            "Triggered a live browser form submission request.",
        )];
        if let Some(form) = form.as_ref() {
            observations.push(format!(
                "The submission targeted deterministic form element_id={}.",
                form.element_id
            ));
        } else {
            observations.push(String::from(
                "The browser backend resolved the active or uniquely visible form at submit time.",
            ));
        }
        if browser_submit.page_changed {
            observations.push(String::from(
                "Submitting the form changed the live browser URL, so runtime page state advanced to a new page.",
            ));
        } else {
            observations.push(String::from(
                "The submission request completed without a live browser navigation.",
            ));
        }

        ToolResult::success(
            ToolName::SubmitActiveForm,
            input.request_id,
            SubmitActiveFormData {
                form_element_id: form.as_ref().map(|form| form.element_id.clone()),
                submitted: browser_submit.submitted,
                page_changed: browser_submit.page_changed,
                navigation_url: browser_submit
                    .page_changed
                    .then_some(browser_submit.url.clone()),
            },
            observations,
        )
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

    pub fn execute_set_tts_voice(
        &mut self,
        input: SetTtsVoiceInput,
    ) -> ToolResult<SetTtsVoiceData> {
        let voice = input.voice.to_string();

        let changed = self.config.audio.default_tts_voice != voice;
        match self.set_default_tts_voice(voice.clone()) {
            Ok(()) => ToolResult::success(
                ToolName::SetTtsVoice,
                input.request_id,
                SetTtsVoiceData { voice, changed },
                vec![String::from("Updated the default TTS voice setting.")],
            ),
            Err(error) => self.audio_tool_failure(
                ToolName::SetTtsVoice,
                input.request_id,
                String::from("Failed to persist the requested TTS voice."),
                error,
            ),
        }
    }

    pub fn execute_set_playback_volume(
        &mut self,
        input: SetPlaybackVolumeInput,
    ) -> ToolResult<SetPlaybackVolumeData> {
        let requested_volume = input.volume;
        let clamped_volume = requested_volume.clamp(
            crate::config::MIN_PLAYBACK_VOLUME,
            crate::config::MAX_PLAYBACK_VOLUME,
        );
        let changed = (self.config.audio.playback_volume - clamped_volume).abs() > f32::EPSILON;

        match self.set_playback_volume(clamped_volume) {
            Ok(()) => {
                let mut observations = vec![String::from("Updated the playback volume setting.")];
                observations.push(String::from(
                    "New narration requests will use the updated playback volume.",
                ));
                if (requested_volume - clamped_volume).abs() > f32::EPSILON {
                    observations.push(String::from(
                        "Requested playback volume was clamped to the supported range.",
                    ));
                }

                ToolResult::success(
                    ToolName::SetPlaybackVolume,
                    input.request_id,
                    SetPlaybackVolumeData {
                        playback_volume: self.state.audio.playback_volume,
                        muted: self.state.audio.muted,
                        changed,
                    },
                    observations,
                )
            }
            Err(error) => self.audio_tool_failure(
                ToolName::SetPlaybackVolume,
                input.request_id,
                String::from("Failed to persist the requested playback volume."),
                error,
            ),
        }
    }

    pub fn execute_set_playback_speed(
        &mut self,
        input: SetPlaybackSpeedInput,
    ) -> ToolResult<SetPlaybackSpeedData> {
        let requested_speed = input.speed;
        let clamped_speed = requested_speed.clamp(
            crate::config::MIN_PLAYBACK_SPEED,
            crate::config::MAX_PLAYBACK_SPEED,
        );
        let changed = (self.config.audio.playback_speed - clamped_speed).abs() > f32::EPSILON;

        match self.set_playback_speed(clamped_speed) {
            Ok(()) => {
                let mut observations = vec![String::from("Updated the playback speed setting.")];
                observations.push(String::from(
                    "New narration requests will use the updated native TTS speed.",
                ));
                if (requested_speed - clamped_speed).abs() > f32::EPSILON {
                    observations.push(String::from(
                        "Requested playback speed was clamped to the supported range.",
                    ));
                }

                ToolResult::success(
                    ToolName::SetPlaybackSpeed,
                    input.request_id,
                    SetPlaybackSpeedData {
                        playback_speed: self.state.audio.playback_speed,
                        changed,
                    },
                    observations,
                )
            }
            Err(error) => self.audio_tool_failure(
                ToolName::SetPlaybackSpeed,
                input.request_id,
                String::from("Failed to persist the requested playback speed."),
                error,
            ),
        }
    }

    pub fn execute_set_browser_visibility(
        &mut self,
        input: SetBrowserVisibilityInput,
    ) -> ToolResult<SetBrowserVisibilityData> {
        if self.state.browser_visibility == input.mode {
            return ToolResult::success(
                ToolName::SetBrowserVisibility,
                input.request_id,
                SetBrowserVisibilityData {
                    mode: self.state.browser_visibility,
                    changed: false,
                    supported: true,
                },
                vec![String::from(
                    "Browser visibility mode is already set to the requested value.",
                )],
            );
        }

        match self.browser.switch_visibility(input.mode) {
            Ok(restored_url) => {
                self.state.browser_visibility = input.mode;
                if restored_url.is_some() {
                    // The page model is stale after relaunch; clear it so the
                    // planner knows it must re-extract before relying on any
                    // stored element references.
                    self.state.current_page = None;
                }
                ToolResult::success(
                    ToolName::SetBrowserVisibility,
                    input.request_id,
                    SetBrowserVisibilityData {
                        mode: self.state.browser_visibility,
                        changed: true,
                        supported: true,
                    },
                    vec![String::from("Browser visibility mode was updated.")],
                )
            }
            Err(BrowserError::FeatureDisabled) => ToolResult::success(
                ToolName::SetBrowserVisibility,
                input.request_id,
                SetBrowserVisibilityData {
                    mode: self.state.browser_visibility,
                    changed: false,
                    supported: false,
                },
                vec![String::from(
                    "Browser visibility switching is not supported in this build.",
                )],
            ),
            Err(error) => self.browser_tool_failure(
                ToolName::SetBrowserVisibility,
                input.request_id,
                String::from("Browser could not be relaunched with the requested visibility mode."),
                error,
            ),
        }
    }

    fn current_tts_model_settings(&self) -> TtsModelSettings {
        build_tts_model_settings(&self.config)
    }

    fn current_local_tts_model_settings(&self) -> LocalTtsModelSettings {
        build_local_tts_model_settings(&self.config)
    }

    fn current_tts_voice_settings(&self) -> TtsVoiceSettings {
        build_tts_voice_settings(&self.config, &self.state.audio)
    }

    fn current_tts_provider_settings(&self) -> TtsProviderSettings {
        build_tts_provider_settings(&self.config)
    }

    fn current_asr_provider_settings(&self) -> AsrProviderSettings {
        build_asr_provider_settings(&self.config)
    }

    fn current_local_asr_model_settings(&self) -> LocalAsrModelSettings {
        build_local_asr_model_settings(&self.config)
    }

    fn current_planner_provider_settings(&self) -> PlannerProviderSettings {
        build_planner_provider_settings(&self.config)
    }

    pub fn current_remote_planner_settings(&self) -> RemotePlannerSettings {
        build_remote_planner_settings(&self.config)
    }

    pub fn current_remote_tts_settings(&self) -> RemoteTtsSettings {
        build_remote_tts_settings(&self.config)
    }

    pub fn current_remote_asr_settings(&self) -> RemoteAsrSettings {
        build_remote_asr_settings(&self.config)
    }

    fn current_provider_failover_settings(&self) -> ProviderFailoverSettings {
        build_provider_failover_settings(&self.config)
    }

    fn current_confirmation_settings(&self) -> ConfirmationSettings {
        build_confirmation_settings(&self.config)
    }

    fn current_ocr_threshold_settings(&self) -> OcrThresholdSettings {
        build_ocr_threshold_settings(&self.config)
    }

    pub fn set_confirmation_confidence_threshold(
        &mut self,
        confirmation_confidence_threshold: f32,
    ) -> Result<(), ConfigError> {
        let mut safety = self.config.safety.clone();
        safety.confirmation_confidence_threshold = confirmation_confidence_threshold;
        let next_config = AppConfig::persist_safety_settings_for_app(&self.app_handle, &safety)?;
        self.config = next_config;
        Ok(())
    }

    pub fn set_allow_click_without_confirmation(
        &mut self,
        allow_click_without_confirmation: bool,
    ) -> Result<(), ConfigError> {
        let mut safety = self.config.safety.clone();
        safety.allow_click_without_confirmation = allow_click_without_confirmation;
        let next_config = AppConfig::persist_safety_settings_for_app(&self.app_handle, &safety)?;
        self.config = next_config;
        Ok(())
    }

    pub fn set_ocr_thresholds(
        &mut self,
        sparse_text_char_threshold: u32,
        sparse_text_region_threshold: u32,
    ) -> Result<(), ConfigError> {
        let mut ocr = self.config.ocr.clone();
        ocr.sparse_text_char_threshold = sparse_text_char_threshold;
        ocr.sparse_text_region_threshold = sparse_text_region_threshold;
        let next_config = AppConfig::persist_ocr_settings_for_app(&self.app_handle, &ocr)?;
        self.config = next_config;
        Ok(())
    }

    pub fn execute_read_region(&mut self, input: ReadRegionInput) -> ToolResult<ReadRegionData> {
        self.sync_narration_playback_state();
        let region_id = input.region_id.trim().to_string();
        if region_id.is_empty() {
            return ToolResult::failure(
                ToolName::ReadRegion,
                input.request_id,
                ToolError {
                    code: String::from("invalid_region_id"),
                    message: String::from("read_region requires a non-empty region_id"),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Narration request was rejected because the region_id was empty.",
                )],
            );
        }

        let (region_index, region) = match self.region_by_id(&region_id) {
            Ok(region) => region,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::ReadRegion,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Narration request could not resolve the requested region in the current page model.",
                    )],
                )
            }
        };

        let interrupted_region_id = match self.begin_region_narration(
            region_index,
            &region,
            input.interruption_mode.interrupts_current_playback(),
        ) {
            Ok(interrupted_region_id) => interrupted_region_id,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::ReadRegion,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Narration request could not start playback for the requested region.",
                    )],
                )
            }
        };

        let mut observations = vec![format!(
            "Moved the narration cursor to region_id={} at index {}.",
            region.region_id, region_index
        )];
        observations.push(String::from(
            "Started real narration playback using the configured TTS and audio backends.",
        ));
        if let Some(interrupted_region_id) = interrupted_region_id {
            observations.push(format!(
                "Interrupted the previously active narration region {} before starting the new region.",
                interrupted_region_id
            ));
        }

        ToolResult::success(
            ToolName::ReadRegion,
            input.request_id,
            ReadRegionData {
                region_id: region.region_id,
                region_index,
                text_length: region.text.chars().count(),
                speech_started: true,
            },
            observations,
        )
    }

    pub fn execute_read_next_region(
        &mut self,
        input: ReadNextRegionInput,
    ) -> ToolResult<ReadNextRegionData> {
        self.sync_narration_playback_state();
        let regions = match self.readable_regions() {
            Ok(regions) => regions,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::ReadNextRegion,
                    input.request_id,
                    error,
                    vec![String::from(
                    "Narration could not advance because the current page has no readable regions.",
                )],
                )
            }
        };

        let Some(region_index) = next_region_index(&self.state.narration_cursor, regions.len())
        else {
            return ToolResult::success(
                ToolName::ReadNextRegion,
                input.request_id,
                ReadNextRegionData {
                    cursor: self.state.narration_cursor.clone(),
                    region_id: None,
                    speech_started: false,
                    boundary: crate::commands::NarrationBoundary::End,
                },
                vec![String::from(
                    "Narration is already at the end of the readable region list.",
                )],
            );
        };
        let region = regions[region_index].clone();
        let interrupted_region_id = match self.begin_region_narration(
            region_index,
            &region,
            input.interruption_mode.interrupts_current_playback(),
        ) {
            Ok(interrupted_region_id) => interrupted_region_id,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::ReadNextRegion,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Narration could not advance to the next region for playback.",
                    )],
                )
            }
        };

        let mut observations = vec![format!(
            "Advanced narration to region_id={} at index {}.",
            region.region_id, region_index
        )];
        observations.push(String::from(
            "Started real narration playback using the configured TTS and audio backends.",
        ));
        if let Some(interrupted_region_id) = interrupted_region_id {
            observations.push(format!(
                "Interrupted the previously active narration region {} before reading the next region.",
                interrupted_region_id
            ));
        }

        ToolResult::success(
            ToolName::ReadNextRegion,
            input.request_id,
            ReadNextRegionData {
                cursor: self.state.narration_cursor.clone(),
                region_id: Some(region.region_id),
                speech_started: true,
                boundary: crate::commands::NarrationBoundary::None,
            },
            observations,
        )
    }

    pub fn execute_read_previous_region(
        &mut self,
        input: ReadPreviousRegionInput,
    ) -> ToolResult<ReadPreviousRegionData> {
        self.sync_narration_playback_state();
        let regions = match self.readable_regions() {
            Ok(regions) => regions,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::ReadPreviousRegion,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Narration could not move backward because the current page has no readable regions.",
                    )],
                )
            }
        };

        let Some(region_index) = previous_region_index(&self.state.narration_cursor, regions.len())
        else {
            return ToolResult::success(
                ToolName::ReadPreviousRegion,
                input.request_id,
                ReadPreviousRegionData {
                    cursor: self.state.narration_cursor.clone(),
                    region_id: None,
                    speech_started: false,
                    boundary: crate::commands::NarrationBoundary::Start,
                },
                vec![String::from(
                    "Narration is already at the start of the readable region list.",
                )],
            );
        };
        let region = regions[region_index].clone();
        let interrupted_region_id = match self.begin_region_narration(
            region_index,
            &region,
            input.interruption_mode.interrupts_current_playback(),
        ) {
            Ok(interrupted_region_id) => interrupted_region_id,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::ReadPreviousRegion,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Narration could not move backward to the previous region for playback.",
                    )],
                )
            }
        };

        let mut observations = vec![format!(
            "Moved narration backward to region_id={} at index {}.",
            region.region_id, region_index
        )];
        observations.push(String::from(
            "Started real narration playback using the configured TTS and audio backends.",
        ));
        if let Some(interrupted_region_id) = interrupted_region_id {
            observations.push(format!(
                "Interrupted the previously active narration region {} before reading the previous region.",
                interrupted_region_id
            ));
        }

        ToolResult::success(
            ToolName::ReadPreviousRegion,
            input.request_id,
            ReadPreviousRegionData {
                cursor: self.state.narration_cursor.clone(),
                region_id: Some(region.region_id),
                speech_started: true,
                boundary: crate::commands::NarrationBoundary::None,
            },
            observations,
        )
    }

    pub fn execute_stop_speaking(
        &mut self,
        input: StopSpeakingInput,
    ) -> ToolResult<StopSpeakingData> {
        self.sync_narration_playback_state();
        let interrupted_region_id = self.stop_narration_playback();
        let stopped = interrupted_region_id.is_some();

        ToolResult::success(
            ToolName::StopSpeaking,
            input.request_id,
            StopSpeakingData {
                stopped,
                interrupted_region_id,
            },
            vec![if stopped {
                String::from("Stopped the active narration region.")
            } else {
                String::from("No narration was active, so there was nothing to stop.")
            }],
        )
    }

    pub fn execute_start_listening(
        &mut self,
        input: StartListeningInput,
    ) -> ToolResult<StartListeningData> {
        match self.asr.start_listening() {
            Ok(activated) => {
                self.state.set_listening(self.asr.is_listening());
                ToolResult::success(
                    ToolName::StartListening,
                    input.request_id,
                    StartListeningData {
                        listening_state: self.state.listening.clone(),
                        activated,
                    },
                    vec![if activated {
                        String::from("Started microphone capture for voice input.")
                    } else {
                        String::from(
                            "Listening was already active, so capture continued unchanged.",
                        )
                    }],
                )
            }
            Err(error) => ToolResult::failure(
                ToolName::StartListening,
                input.request_id,
                asr_runtime_error_to_tool_error(&error),
                vec![String::from(
                    "Could not activate voice input listening in the configured ASR backend.",
                )],
            ),
        }
    }

    pub fn execute_stop_listening(
        &mut self,
        input: StopListeningInput,
    ) -> ToolResult<StopListeningData> {
        let deactivated = self.asr.stop_listening();
        self.state.set_listening(self.asr.is_listening());

        ToolResult::success(
            ToolName::StopListening,
            input.request_id,
            StopListeningData {
                listening_state: self.state.listening.clone(),
                deactivated,
            },
            vec![if deactivated {
                String::from("Stopped microphone capture for voice input.")
            } else {
                String::from("Listening was already inactive, so there was nothing to stop.")
            }],
        )
    }

    pub fn execute_transcribe_command(
        &mut self,
        input: TranscribeCommandInput,
    ) -> ToolResult<TranscribeCommandData> {
        let requested_duration_ms = input
            .max_duration_ms
            .unwrap_or(DEFAULT_TRANSCRIBE_DURATION_MS);
        if requested_duration_ms == 0 {
            return ToolResult::failure(
                ToolName::TranscribeCommand,
                input.request_id,
                ToolError {
                    code: String::from("invalid_max_duration_ms"),
                    message: String::from(
                        "transcribe_command requires max_duration_ms to be greater than zero",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Transcription request was rejected because the requested capture duration was zero.",
                )],
            );
        }

        let mut effective_duration_ms = requested_duration_ms.min(MAX_TRANSCRIBE_DURATION_MS);
        if let Some(timeout_ms) = input.timeout_ms {
            effective_duration_ms = effective_duration_ms.min(timeout_ms.max(1));
        }

        match self.asr.transcribe_command(
            &self.config,
            effective_duration_ms,
            input.stop_mode.auto_stops(),
        ) {
            Ok(result) => {
                self.state.set_listening(result.listening_active);
                self.state.record_transcript(result.transcript.clone());

                let mut observations = vec![String::from(
                    "Captured microphone audio and ran deterministic speech transcription.",
                )];
                if requested_duration_ms > MAX_TRANSCRIBE_DURATION_MS {
                    observations.push(format!(
                        "Requested capture duration was clamped to the supported maximum of {} ms.",
                        MAX_TRANSCRIBE_DURATION_MS
                    ));
                }
                if input
                    .timeout_ms
                    .is_some_and(|timeout_ms| timeout_ms < effective_duration_ms)
                {
                    observations.push(String::from(
                        "Capture duration was reduced to respect the requested tool timeout.",
                    ));
                }
                observations.push(if result.transcript.is_some() {
                    String::from("ASR returned a non-empty spoken command transcript.")
                } else {
                    String::from("ASR completed but did not detect a spoken command transcript.")
                });

                ToolResult::success(
                    ToolName::TranscribeCommand,
                    input.request_id,
                    TranscribeCommandData {
                        transcript: result.transcript,
                        confidence: result.confidence,
                        audio_duration_ms: result.audio_duration_ms,
                        listening_state: self.state.listening.clone(),
                    },
                    observations,
                )
            }
            Err(error) => {
                self.state.set_listening(self.asr.is_listening());
                ToolResult::failure(
                    ToolName::TranscribeCommand,
                    input.request_id,
                    asr_runtime_error_to_tool_error(&error),
                    vec![String::from(
                        "Could not complete spoken-command transcription in the configured ASR backend.",
                    )],
                )
            }
        }
    }

    pub fn transcribe_and_execute_command(
        &mut self,
        request_id: String,
        timeout_ms: Option<u64>,
        max_duration_ms: Option<u64>,
        auto_stop: bool,
    ) -> Result<TranscribeAndExecuteCommandData, ToolError> {
        let transcription_result = self.execute_transcribe_command(TranscribeCommandInput {
            request_id: request_id.clone(),
            timeout_ms,
            max_duration_ms,
            stop_mode: if auto_stop {
                crate::commands::TranscriptionStopMode::AutoStop
            } else {
                crate::commands::TranscriptionStopMode::KeepListening
            },
        });

        let Some(transcription) = transcription_result.data else {
            return Err(transcription_result.error.unwrap_or(ToolError {
                code: String::from("missing_transcription_result"),
                message: String::from("transcribe_command did not return transcription data"),
                retryable: false,
                details: Some(serde_json::json!({
                    "request_id": request_id,
                    "tool_name": ToolName::TranscribeCommand,
                })),
            }));
        };

        let (command_error, execution_outcome) =
            if let Some(transcript) = transcription.transcript.clone() {
                match self.execute_command_with_replanning(request_id.clone(), transcript) {
                    Ok(outcome) => (None, Some(outcome)),
                    Err(error) => (Some(error), None),
                }
            } else {
                (None, None)
            };

        Ok(TranscribeAndExecuteCommandData {
            transcription,
            command_error,
            execution_outcome,
        })
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

    fn readable_regions(&self) -> Result<&[PageRegion], ToolError> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return Err(ToolError {
                code: String::from("no_active_page"),
                message: String::from("narration tool requires an active page in runtime state"),
                retryable: false,
                details: None,
            });
        };
        let Some(current_page) = self.state.current_page.as_ref() else {
            return Err(ToolError {
                code: String::from("missing_page_model"),
                message: String::from(
                    "narration tool requires runtime page data for the active page",
                ),
                retryable: false,
                details: Some(serde_json::json!({ "page_id": page_id })),
            });
        };
        if current_page.regions.is_empty() {
            return Err(ToolError {
                code: String::from("no_readable_regions"),
                message: String::from(
                    "narration tool requires at least one readable region in the current page model",
                ),
                retryable: false,
                details: Some(serde_json::json!({ "page_id": page_id })),
            });
        }

        Ok(&current_page.regions)
    }

    fn region_by_id(&self, region_id: &str) -> Result<(usize, PageRegion), ToolError> {
        let regions = self.readable_regions()?;
        let Some(region_index) = find_region_index(regions, region_id) else {
            return Err(ToolError {
                code: String::from("region_not_found"),
                message: String::from(
                    "read_region could not find the requested region_id in the current page model",
                ),
                retryable: false,
                details: Some(serde_json::json!({ "region_id": region_id })),
            });
        };

        Ok((region_index, regions[region_index].clone()))
    }

    fn begin_region_narration(
        &mut self,
        region_index: usize,
        region: &PageRegion,
        interrupt_current: bool,
    ) -> Result<Option<String>, ToolError> {
        self.sync_narration_playback_state();
        if self.state.speaking && !interrupt_current {
            return Err(ToolError {
                code: String::from("speech_in_progress"),
                message: String::from(
                    "a narration region is already active; set interruption_mode to Interrupt to replace it",
                ),
                retryable: false,
                details: Some(serde_json::json!({
                    "active_region_id": self.state.speaking_region_id.clone(),
                })),
            });
        }

        let spoken_text = spoken_text_for_region(region);
        if spoken_text.trim().is_empty() {
            return Err(ToolError {
                code: String::from("empty_region_text"),
                message: String::from(
                    "narration tool requires the selected region to contain readable text",
                ),
                retryable: false,
                details: Some(serde_json::json!({
                    "region_id": region.region_id,
                    "region_index": region_index,
                })),
            });
        }

        let speech = self
            .tts
            .synthesize_narration(&self.config, &self.state.audio, &spoken_text)
            .map_err(tts_runtime_error_to_tool_error)?;

        let interrupted_region_id = if self.state.speaking {
            self.stop_narration_playback()
        } else {
            None
        };

        self.playback
            .play_samples(
                speech.samples,
                speech.channels,
                speech.sample_rate,
                self.state.audio.playback_volume,
            )
            .map_err(audio_playback_error_to_tool_error)?;
        self.state.narration_cursor = cursor_for_index(self.readable_regions()?, region_index);
        self.state.start_speaking_region(region.region_id.clone());

        Ok(interrupted_region_id)
    }

    fn begin_feedback_narration(&mut self, spoken_text: &str) -> Result<(), ToolError> {
        let spoken_text = spoken_text.trim();
        if spoken_text.is_empty() {
            return Err(ToolError {
                code: String::from("empty_report_summary"),
                message: String::from("spoken feedback requires a non-empty summary"),
                retryable: false,
                details: None,
            });
        }

        self.sync_narration_playback_state();
        let speech = self
            .tts
            .synthesize_narration(&self.config, &self.state.audio, spoken_text)
            .map_err(tts_runtime_error_to_tool_error)?;

        if self.state.speaking {
            self.stop_narration_playback();
        }

        self.playback
            .play_samples(
                speech.samples,
                speech.channels,
                speech.sample_rate,
                self.state.audio.playback_volume,
            )
            .map_err(audio_playback_error_to_tool_error)?;
        self.state
            .start_speaking_region(String::from("report-result-feedback"));

        Ok(())
    }

    fn sync_narration_playback_state(&mut self) {
        if !self.playback.is_active() && self.state.speaking {
            self.state.stop_speaking();
        }
    }

    fn stop_narration_playback(&mut self) -> Option<String> {
        let stopped_playback = self.playback.stop();
        let interrupted_region_id = self.state.stop_speaking();

        if interrupted_region_id.is_some() || stopped_playback {
            interrupted_region_id
        } else {
            None
        }
    }

    fn current_agent_state_snapshot(&self, include_last_transcript: bool) -> AgentStateData {
        AgentStateData {
            page_id: self.state.current_page_id.clone(),
            url: self
                .state
                .current_page
                .as_ref()
                .and_then(|page| page.url.clone()),
            title: self
                .state
                .current_page
                .as_ref()
                .and_then(|page| page.title.clone()),
            browser_visibility: self.state.browser_visibility,
            browser_history: self.state.browser_history.clone(),
            narration_cursor: Some(self.state.narration_cursor.clone()),
            speaking: self.state.speaking,
            listening_state: self.state.listening.clone(),
            audio: self.state.audio.clone(),
            last_transcript: if include_last_transcript {
                self.state.last_transcript.clone()
            } else {
                None
            },
            last_tool_call: self.state.last_tool_call.clone(),
            pending_confirmation_id: self.state.pending_confirmation_id.clone(),
            pending_plan_execution: self.state.pending_plan_execution.clone(),
            tts_model_settings: self.current_tts_model_settings(),
            local_tts_model_settings: self.current_local_tts_model_settings(),
            tts_voice_settings: self.current_tts_voice_settings(),
            tts_provider_settings: self.current_tts_provider_settings(),
            asr_provider_settings: self.current_asr_provider_settings(),
            local_asr_model_settings: self.current_local_asr_model_settings(),
            planner_provider_settings: self.current_planner_provider_settings(),
            remote_planner_settings: self.current_remote_planner_settings(),
            remote_tts_settings: self.current_remote_tts_settings(),
            remote_asr_settings: self.current_remote_asr_settings(),
            provider_failover_settings: self.current_provider_failover_settings(),
            confirmation_settings: self.current_confirmation_settings(),
            ocr_threshold_settings: self.current_ocr_threshold_settings(),
        }
    }

    fn current_runtime_status_snapshot(
        &self,
        include_provider_modes: bool,
    ) -> GetRuntimeStatusData {
        GetRuntimeStatusData {
            page_id: self.state.current_page_id.clone(),
            url: self
                .state
                .current_page
                .as_ref()
                .and_then(|page| page.url.clone()),
            title: self
                .state
                .current_page
                .as_ref()
                .and_then(|page| page.title.clone()),
            browser_visibility: self.state.browser_visibility,
            browser_history: self.state.browser_history.clone(),
            listening_state: self.state.listening.clone(),
            speaking: self.state.speaking,
            audio: self.state.audio.clone(),
            pending_confirmation_id: self.state.pending_confirmation_id.clone(),
            pending_plan_execution: self.state.pending_plan_execution.clone(),
            provider_modes: if include_provider_modes {
                Some(ProviderSelectionStatus {
                    planner_mode: self.config.providers.planner.mode.clone(),
                    tts_mode: self.config.providers.tts.mode.clone(),
                    asr_mode: self.config.providers.asr.mode.clone(),
                })
            } else {
                None
            },
        }
    }

    fn current_page_snapshot(
        &mut self,
        text_excerpt_max_chars: Option<usize>,
        include_interactive_elements: bool,
    ) -> Option<PageSnapshotData> {
        let page_id = self.state.current_page_id.clone()?;
        let current_page = self.state.current_page.as_ref()?;
        let url = current_page.url.clone()?;
        let title = current_page.title.clone();
        let visible_text_excerpt = build_visible_text_excerpt(current_page, text_excerpt_max_chars);
        let interactive_elements = if include_interactive_elements {
            current_page.interactive_elements.clone()
        } else {
            Vec::new()
        };
        let _ = current_page;
        let BrowserPageMetrics {
            scroll_y,
            viewport_width,
            viewport_height,
            document_height,
        } = self.browser.get_page_metrics().ok()?;

        Some(PageSnapshotData {
            page_id,
            url,
            title,
            visible_text_excerpt,
            interactive_elements,
            scroll_y,
            viewport_width,
            viewport_height,
            document_height,
        })
    }

    fn resolve_planner_output(
        &self,
        planner_input: &PlannerInput,
    ) -> Result<PlannerOutput, ToolError> {
        self.resolve_remote_planner_output(planner_input)
    }

    fn resolve_remote_planner_output(
        &self,
        planner_input: &PlannerInput,
    ) -> Result<PlannerOutput, ToolError> {
        let Some(profile_name) = self.config.providers.planner.remote_profile.as_deref() else {
            return Err(planner_interpretation_unavailable_error(
                "planner_profile_unavailable",
                "remote planner mode requires a configured planner profile",
                false,
                None,
            ));
        };
        let Some(profile) = self.config.remote_planner_profiles.get(profile_name) else {
            return Err(planner_interpretation_unavailable_error(
                "planner_profile_unavailable",
                format!("configured remote planner profile '{profile_name}' was not found"),
                false,
                None,
            ));
        };

        match profile.provider {
            RemoteProviderKind::OpenAi => self.resolve_with_openai_planner(profile, planner_input),
            RemoteProviderKind::Ollama => self.resolve_with_ollama_planner(profile, planner_input),
        }
    }

    #[cfg(feature = "remote-openai")]
    fn planner_prompt_payload<'a>(
        &self,
        planner_input: &'a PlannerInput,
    ) -> PlannerPromptPayload<'a> {
        let tool_schemas = planner_input
            .available_tools
            .iter()
            .filter_map(|tool| {
                tool_input_schema(&tool.name).map(|schema| (format!("{:?}", tool.name), schema))
            })
            .collect::<BTreeMap<_, _>>();

        PlannerPromptPayload {
            planner_input,
            planner_output_schema: planner_output_schema(),
            tool_input_schemas: tool_schemas,
            canonical_planner_output_examples: canonical_planner_output_examples(),
        }
    }

    #[cfg(feature = "remote-openai")]
    fn resolve_with_openai_planner(
        &self,
        profile: &RemotePlannerProfile,
        planner_input: &PlannerInput,
    ) -> Result<PlannerOutput, ToolError> {
        use async_openai::types::chat::{
            ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
            CreateChatCompletionRequestArgs,
        };
        use async_openai::types::chat::{ResponseFormat, ResponseFormatJsonSchema};
        use async_openai::{config::OpenAIConfig, Client};

        let api_key = resolve_secret_ref(&profile.api_key).map_err(|reason| {
            planner_interpretation_unavailable_error(
                "planner_secret_unavailable",
                "remote planner API key could not be resolved",
                false,
                Some(serde_json::json!({ "reason": reason })),
            )
        })?;

        let mut openai_config = OpenAIConfig::new()
            .with_api_base(profile.base_url.clone())
            .with_api_key(api_key);
        if let Some(organization) = profile.organization.as_ref() {
            openai_config =
                openai_config.with_org_id(resolve_secret_ref(organization).map_err(|reason| {
                    planner_interpretation_unavailable_error(
                        "planner_secret_unavailable",
                        "remote planner organization secret could not be resolved",
                        false,
                        Some(serde_json::json!({ "reason": reason })),
                    )
                })?);
        }
        if let Some(project) = profile.project.as_ref() {
            openai_config = openai_config.with_project_id(project.clone());
        }

        let client = Client::with_config(openai_config);
        let prompt_payload = self.planner_prompt_payload(planner_input);
        let user_content =
            serde_json::to_string_pretty(&prompt_payload).expect("planner prompt should serialize");
        let request = CreateChatCompletionRequestArgs::default()
            .model(profile.model.clone())
            .temperature(profile.temperature_milli as f32 / 1_000.0)
            .max_completion_tokens(profile.max_output_tokens)
            .response_format(ResponseFormat::JsonSchema {
                json_schema: ResponseFormatJsonSchema {
                    description: Some(String::from(
                        "Structured deterministic planner output for blind_browser.",
                    )),
                    name: String::from("planner_output"),
                    schema: Some(planner_output_schema()),
                    strict: Some(true),
                },
            })
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(planner_system_prompt())
                    .build()
                    .map_err(|error| {
                        planner_interpretation_unavailable_error(
                            "planner_request_build_failed",
                            format!(
                                "failed to build planner system message for remote resolution: {error}"
                            ),
                            false,
                            None,
                        )
                    })?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(user_content)
                    .build()
                    .map_err(|error| {
                        planner_interpretation_unavailable_error(
                            "planner_request_build_failed",
                            format!(
                                "failed to build planner user message for remote resolution: {error}"
                            ),
                            false,
                            None,
                        )
                    })?
                    .into(),
            ])
            .build()
            .map_err(|error| {
                planner_interpretation_unavailable_error(
                    "planner_request_build_failed",
                    format!("failed to build remote planner request: {error}"),
                    false,
                    None,
                )
            })?;

        let response =
            futures::executor::block_on(client.chat().create(request)).map_err(|error| {
                planner_interpretation_unavailable_error(
                    "planner_request_failed",
                    format!("remote planner request failed: {error}"),
                    true,
                    Some(serde_json::json!({
                        "provider": "OpenAI",
                        "model": profile.model,
                        "base_url": profile.base_url,
                    })),
                )
            })?;
        let content = response
            .choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .ok_or_else(|| {
                planner_interpretation_unavailable_error(
                    "planner_response_missing",
                    "remote planner returned no structured content",
                    true,
                    None,
                )
            })?;

        serde_json::from_str::<PlannerOutput>(&content).map_err(|error| {
            planner_interpretation_unavailable_error(
                "planner_response_invalid",
                format!("remote planner returned invalid planner JSON: {error}"),
                true,
                Some(serde_json::json!({ "content": content })),
            )
        })
    }

    #[cfg(not(feature = "remote-openai"))]
    fn resolve_with_openai_planner(
        &self,
        _profile: &RemotePlannerProfile,
        _planner_input: &PlannerInput,
    ) -> Result<PlannerOutput, ToolError> {
        Err(planner_interpretation_unavailable_error(
            "planner_backend_unavailable",
            "remote OpenAI planner support is not enabled in this build",
            false,
            None,
        ))
    }

    #[cfg(feature = "remote-openai")]
    fn resolve_with_ollama_planner(
        &self,
        profile: &RemotePlannerProfile,
        planner_input: &PlannerInput,
    ) -> Result<PlannerOutput, ToolError> {
        use async_openai::types::chat::ResponseFormat;
        use async_openai::types::chat::{
            ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
            CreateChatCompletionRequestArgs,
        };
        use async_openai::{config::OpenAIConfig, Client};

        let api_key = resolve_secret_ref(&profile.api_key).map_err(|reason| {
            planner_interpretation_unavailable_error(
                "planner_secret_unavailable",
                "Ollama planner API key placeholder could not be resolved",
                false,
                Some(serde_json::json!({ "reason": reason })),
            )
        })?;

        let client = Client::with_config(
            OpenAIConfig::new()
                .with_api_base(profile.base_url.clone())
                .with_api_key(api_key),
        );
        let prompt_payload = self.planner_prompt_payload(planner_input);
        let user_content =
            serde_json::to_string_pretty(&prompt_payload).expect("planner prompt should serialize");
        let request = CreateChatCompletionRequestArgs::default()
            .model(profile.model.clone())
            .temperature(profile.temperature_milli as f32 / 1_000.0)
            .max_tokens(profile.max_output_tokens)
            .response_format(ResponseFormat::JsonObject)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(planner_system_prompt())
                    .build()
                    .map_err(|error| {
                        planner_interpretation_unavailable_error(
                            "planner_request_build_failed",
                            format!(
                                "failed to build planner system message for Ollama resolution: {error}"
                            ),
                            false,
                            None,
                        )
                    })?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(user_content)
                    .build()
                    .map_err(|error| {
                        planner_interpretation_unavailable_error(
                            "planner_request_build_failed",
                            format!(
                                "failed to build planner user message for Ollama resolution: {error}"
                            ),
                            false,
                            None,
                        )
                    })?
                    .into(),
            ])
            .build()
            .map_err(|error| {
                planner_interpretation_unavailable_error(
                    "planner_request_build_failed",
                    format!("failed to build Ollama planner request: {error}"),
                    false,
                    None,
                )
            })?;

        let response =
            futures::executor::block_on(client.chat().create(request)).map_err(|error| {
                planner_interpretation_unavailable_error(
                    "planner_request_failed",
                    format!("Ollama planner request failed: {error}"),
                    true,
                    Some(serde_json::json!({
                        "provider": "Ollama",
                        "model": profile.model,
                        "base_url": profile.base_url,
                    })),
                )
            })?;
        let content = response
            .choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .ok_or_else(|| {
                planner_interpretation_unavailable_error(
                    "planner_response_missing",
                    "Ollama planner returned no structured content",
                    true,
                    None,
                )
            })?;

        serde_json::from_str::<PlannerOutput>(&content).map_err(|error| {
            planner_interpretation_unavailable_error(
                "planner_response_invalid",
                format!("Ollama planner returned invalid planner JSON: {error}"),
                true,
                Some(serde_json::json!({ "content": content })),
            )
        })
    }

    #[cfg(not(feature = "remote-openai"))]
    fn resolve_with_ollama_planner(
        &self,
        _profile: &RemotePlannerProfile,
        _planner_input: &PlannerInput,
    ) -> Result<PlannerOutput, ToolError> {
        Err(planner_interpretation_unavailable_error(
            "planner_backend_unavailable",
            "remote Ollama planner support is not enabled in this build",
            false,
            None,
        ))
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

impl ReplanningRuntime for AppCore {
    fn resolve_plan(
        &mut self,
        request_id: String,
        transcript: &str,
        recent_tool_results: &[PlannerToolHistoryEntry],
    ) -> Result<PlannerOutput, ToolError> {
        self.resolve_command_with_recent_results(
            request_id,
            transcript,
            recent_tool_results.to_vec(),
        )
    }

    fn execute_plan(
        &mut self,
        request_id: String,
        planner_output: &PlannerOutput,
    ) -> ExecutionOutcome {
        self.execute_planner_output(request_id, planner_output)
    }
}

fn build_tts_model_settings(config: &AppConfig) -> TtsModelSettings {
    let mode = config.providers.tts.mode.clone();
    let (active_profile, available_profiles) = match mode {
        crate::config::ProviderMode::Local => (
            config.providers.tts.local_profile.clone(),
            config
                .local_tts_profiles
                .iter()
                .map(|(profile_name, profile)| TtsModelOption {
                    profile_name: profile_name.clone(),
                    model_label: profile.model_id.clone(),
                })
                .collect(),
        ),
        crate::config::ProviderMode::Remote => (
            config.providers.tts.remote_profile.clone(),
            config
                .remote_tts_profiles
                .iter()
                .map(|(profile_name, profile)| TtsModelOption {
                    profile_name: profile_name.clone(),
                    model_label: profile.model.clone(),
                })
                .collect(),
        ),
    };

    TtsModelSettings {
        mode,
        active_profile,
        available_profiles,
    }
}

fn build_local_tts_model_settings(config: &AppConfig) -> LocalTtsModelSettings {
    let profile_name = config.providers.tts.local_profile.clone();
    let profile = profile_name
        .as_ref()
        .and_then(|configured_profile| config.local_tts_profiles.get(configured_profile));

    LocalTtsModelSettings {
        profile_name,
        backend: profile.map(|configured_profile| configured_profile.backend.clone()),
        model_id: profile.map(|configured_profile| configured_profile.model_id.clone()),
        model_path: profile.map(|configured_profile| configured_profile.model_path.clone()),
        default_voice: profile.map(|configured_profile| configured_profile.default_voice.clone()),
        sample_rate: profile.map(|configured_profile| configured_profile.sample_rate),
    }
}

fn build_tts_provider_settings(config: &AppConfig) -> TtsProviderSettings {
    let mut available_modes = Vec::new();
    if config
        .providers
        .tts
        .local_profile
        .as_ref()
        .and_then(|profile_name| config.local_tts_profiles.get(profile_name))
        .is_some()
    {
        available_modes.push(crate::config::ProviderMode::Local);
    }
    if config
        .providers
        .tts
        .remote_profile
        .as_ref()
        .and_then(|profile_name| config.remote_tts_profiles.get(profile_name))
        .is_some()
    {
        available_modes.push(crate::config::ProviderMode::Remote);
    }
    if available_modes.is_empty() {
        available_modes.push(config.providers.tts.mode.clone());
    }

    TtsProviderSettings {
        active_mode: config.providers.tts.mode.clone(),
        available_modes,
    }
}

fn build_planner_provider_settings(config: &AppConfig) -> PlannerProviderSettings {
    let active_mode = config.providers.planner.mode.clone();
    PlannerProviderSettings {
        active_mode,
        available_modes: vec![crate::config::ProviderMode::Remote],
        summary: String::from("Planner currently uses configured remote profiles only."),
    }
}

fn remote_provider_label(provider: &RemoteProviderKind) -> RemoteProviderLabel {
    match provider {
        RemoteProviderKind::OpenAi => RemoteProviderLabel::OpenAi,
        RemoteProviderKind::Ollama => RemoteProviderLabel::Ollama,
    }
}

fn build_remote_planner_settings(config: &AppConfig) -> RemotePlannerSettings {
    let profile_name = config.providers.planner.remote_profile.clone();
    let profile = profile_name
        .as_ref()
        .and_then(|configured_profile| config.remote_planner_profiles.get(configured_profile));

    RemotePlannerSettings {
        profile_name,
        provider: profile
            .map(|configured_profile| remote_provider_label(&configured_profile.provider)),
        base_url: profile.map(|configured_profile| configured_profile.base_url.clone()),
        model: profile.map(|configured_profile| configured_profile.model.clone()),
        api_key_reference: profile
            .map(|configured_profile| secret_ref_reference(&configured_profile.api_key)),
        organization_reference: profile
            .and_then(|configured_profile| configured_profile.organization.as_ref())
            .map(secret_ref_reference),
        project: profile.and_then(|configured_profile| configured_profile.project.clone()),
        temperature_milli: profile.map(|configured_profile| configured_profile.temperature_milli),
        max_output_tokens: profile.map(|configured_profile| configured_profile.max_output_tokens),
        timeout_ms: profile.map(|configured_profile| configured_profile.timeout_ms),
    }
}

fn build_remote_tts_settings(config: &AppConfig) -> RemoteTtsSettings {
    let profile_name = config.providers.tts.remote_profile.clone();
    let profile = profile_name
        .as_ref()
        .and_then(|configured_profile| config.remote_tts_profiles.get(configured_profile));

    RemoteTtsSettings {
        profile_name,
        provider: profile
            .map(|configured_profile| remote_provider_label(&configured_profile.provider)),
        base_url: profile.map(|configured_profile| configured_profile.base_url.clone()),
        model: profile.map(|configured_profile| configured_profile.model.clone()),
        api_key_reference: profile
            .map(|configured_profile| secret_ref_reference(&configured_profile.api_key)),
        organization_reference: profile
            .and_then(|configured_profile| configured_profile.organization.as_ref())
            .map(secret_ref_reference),
        project: profile.and_then(|configured_profile| configured_profile.project.clone()),
        voice: profile.map(|configured_profile| configured_profile.voice.clone()),
        audio_format: profile.map(|configured_profile| configured_profile.audio_format.clone()),
        timeout_ms: profile.map(|configured_profile| configured_profile.timeout_ms),
    }
}

fn build_remote_asr_settings(config: &AppConfig) -> RemoteAsrSettings {
    let profile_name = config.providers.asr.remote_profile.clone();
    let profile = profile_name
        .as_ref()
        .and_then(|configured_profile| config.remote_asr_profiles.get(configured_profile));

    RemoteAsrSettings {
        profile_name,
        provider: profile
            .map(|configured_profile| remote_provider_label(&configured_profile.provider)),
        base_url: profile.map(|configured_profile| configured_profile.base_url.clone()),
        model: profile.map(|configured_profile| configured_profile.model.clone()),
        api_key_reference: profile
            .map(|configured_profile| secret_ref_reference(&configured_profile.api_key)),
        organization_reference: profile
            .and_then(|configured_profile| configured_profile.organization.as_ref())
            .map(secret_ref_reference),
        project: profile.and_then(|configured_profile| configured_profile.project.clone()),
        language: profile.and_then(|configured_profile| configured_profile.language.clone()),
        temperature_milli: profile.map(|configured_profile| configured_profile.temperature_milli),
        timeout_ms: profile.map(|configured_profile| configured_profile.timeout_ms),
    }
}

fn build_provider_failover_settings(_config: &AppConfig) -> ProviderFailoverSettings {
    ProviderFailoverSettings {
        planner_available: false,
        tts_available: false,
        asr_available: false,
        summary: String::from(
            "Provider failover settings are defined in config, but automatic failover is still disabled in the live runtime.",
        ),
    }
}

fn build_confirmation_settings(config: &AppConfig) -> ConfirmationSettings {
    ConfirmationSettings {
        confirmation_confidence_threshold: config.safety.confirmation_confidence_threshold,
        allow_click_without_confirmation: config.safety.allow_click_without_confirmation,
        always_confirm_submit: config.safety.always_confirm_submit,
    }
}

fn build_ocr_threshold_settings(config: &AppConfig) -> OcrThresholdSettings {
    OcrThresholdSettings {
        sparse_text_char_threshold: config.ocr.sparse_text_char_threshold,
        sparse_text_region_threshold: config.ocr.sparse_text_region_threshold,
    }
}

fn build_asr_provider_settings(config: &AppConfig) -> AsrProviderSettings {
    let mut available_modes = Vec::new();
    if config
        .providers
        .asr
        .local_profile
        .as_ref()
        .and_then(|profile_name| config.local_asr_profiles.get(profile_name))
        .is_some()
    {
        available_modes.push(crate::config::ProviderMode::Local);
    }
    if config
        .providers
        .asr
        .remote_profile
        .as_ref()
        .and_then(|profile_name| config.remote_asr_profiles.get(profile_name))
        .is_some()
    {
        available_modes.push(crate::config::ProviderMode::Remote);
    }
    if available_modes.is_empty() {
        available_modes.push(config.providers.asr.mode.clone());
    }

    AsrProviderSettings {
        active_mode: config.providers.asr.mode.clone(),
        available_modes,
    }
}

fn build_local_asr_model_settings(config: &AppConfig) -> LocalAsrModelSettings {
    let profile_name = config.providers.asr.local_profile.clone();
    let profile = profile_name
        .as_ref()
        .and_then(|configured_profile| config.local_asr_profiles.get(configured_profile));

    LocalAsrModelSettings {
        profile_name,
        backend: profile.map(|configured_profile| configured_profile.backend.clone()),
        model_id: profile.map(|configured_profile| configured_profile.model_id.clone()),
        model_path: profile.map(|configured_profile| configured_profile.model_path.clone()),
        language: profile.and_then(|configured_profile| configured_profile.language.clone()),
        threads: profile.map(|configured_profile| configured_profile.threads),
    }
}

fn build_model_management_settings(config: &AppConfig) -> ModelManagementSettingsData {
    let (local_tts_profile_name, local_tts_profile) =
        match config.providers.tts.local_profile.as_ref() {
            Some(profile_name) => (
                Some(profile_name.clone()),
                config.local_tts_profiles.get(profile_name),
            ),
            None => (None, None),
        };
    let (local_asr_profile_name, local_asr_profile) =
        match config.providers.asr.local_profile.as_ref() {
            Some(profile_name) => (
                Some(profile_name.clone()),
                config.local_asr_profiles.get(profile_name),
            ),
            None => (None, None),
        };

    ModelManagementSettingsData {
        models_dir: config.models.models_dir.clone(),
        check_on_startup: config.models.check_on_startup,
        auto_download_missing: config.models.auto_download_missing,
        local_tts: ManagedLocalModelStatusData {
            profile_name: local_tts_profile_name,
            backend: local_tts_profile.map(|profile| profile.backend.to_string()),
            model_id: local_tts_profile.map(|profile| profile.model_id.clone()),
            model_path: local_tts_profile.map(|profile| profile.model_path.clone()),
            available: local_tts_profile.is_some_and(local_tts_model_is_available),
            download_supported: local_tts_profile.is_some_and(|profile| {
                kitten_download_plan_for_model_id(&profile.model_id).is_ok()
            }),
            download_label: local_tts_profile
                .and_then(|profile| kitten_download_plan_for_model_id(&profile.model_id).ok())
                .map(|plan| format!("Download {}", plan.display_name)),
        },
        local_asr: ManagedLocalModelStatusData {
            profile_name: local_asr_profile_name,
            backend: local_asr_profile.map(|profile| profile.backend.to_string()),
            model_id: local_asr_profile.map(|profile| profile.model_id.clone()),
            model_path: local_asr_profile.map(|profile| profile.model_path.clone()),
            available: local_asr_profile.is_some_and(local_asr_model_is_available),
            download_supported: local_asr_profile.is_some_and(|profile| {
                whisper_download_plan_for_model_id(&profile.model_id).is_ok()
            }),
            download_label: local_asr_profile
                .and_then(|profile| whisper_download_plan_for_model_id(&profile.model_id).ok())
                .map(|plan| format!("Download Whisper {}", plan.display_name)),
        },
    }
}

fn local_tts_model_is_available(profile: &LocalTtsProfile) -> bool {
    let model_path = Path::new(profile.model_path.trim());
    if !model_path.is_dir() {
        return false;
    }

    let has_config = model_path.join("config.json").is_file();
    let has_voices = model_path.join("voices.npz").is_file();
    let has_onnx = fs::read_dir(model_path)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .any(|entry| {
            entry
                .path()
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("onnx"))
        });

    has_config && has_voices && has_onnx
}

fn local_asr_model_is_available(profile: &LocalAsrProfile) -> bool {
    Path::new(profile.model_path.trim()).is_file()
}

struct KittenDownloadPlan {
    repository: &'static str,
    directory_name: &'static str,
    display_name: &'static str,
    files: &'static [&'static str],
}

fn kitten_download_plan_for_model_id(model_id: &str) -> Result<KittenDownloadPlan, String> {
    let normalized = model_id.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "default" | "mini" | "kitten-tts-mini" => Ok(KittenDownloadPlan {
            repository: "KittenML/kitten-tts-mini-0.8",
            directory_name: "kitten-tts-mini",
            display_name: "KittenTTS mini model",
            files: &["config.json", "kitten_tts_mini_v0_8.onnx", "voices.npz"],
        }),
        "micro" | "kitten-tts-micro" => Ok(KittenDownloadPlan {
            repository: "KittenML/kitten-tts-micro-0.8",
            directory_name: "kitten-tts-micro",
            display_name: "KittenTTS micro model",
            files: &["config.json", "kitten_tts_micro_v0_8.onnx", "voices.npz"],
        }),
        "nano" | "kitten-tts-nano" => Ok(KittenDownloadPlan {
            repository: "KittenML/kitten-tts-nano-0.8-fp32",
            directory_name: "kitten-tts-nano",
            display_name: "KittenTTS nano model",
            files: &["config.json", "kitten_tts_nano_v0_8.onnx", "voices.npz"],
        }),
        "nano-int8" | "kitten-tts-nano-int8" => Ok(KittenDownloadPlan {
            repository: "KittenML/kitten-tts-nano-0.8-int8",
            directory_name: "kitten-tts-nano-int8",
            display_name: "KittenTTS nano int8 model",
            files: &[
                "config.json",
                "kitten_tts_nano_v0_8_int8.onnx",
                "voices.npz",
            ],
        }),
        _ => Err(format!(
            "local TTS model_id '{}' does not have a known Hugging Face download mapping",
            model_id.trim()
        )),
    }
}

struct WhisperDownloadPlan {
    repository: &'static str,
    display_name: &'static str,
    file_name: &'static str,
}

fn whisper_download_plan_for_model_id(model_id: &str) -> Result<WhisperDownloadPlan, String> {
    let normalized = model_id.trim().to_ascii_lowercase();
    let file_name = match normalized.as_str() {
        "tiny" => "ggml-tiny.bin",
        "base" => "ggml-base.bin",
        "small" => "ggml-small.bin",
        "medium" => "ggml-medium.bin",
        "large-v3" => "ggml-large-v3.bin",
        "large-v3-turbo" => "ggml-large-v3-turbo.bin",
        _ => {
            return Err(format!(
                "local ASR model_id '{}' does not have a known Hugging Face download mapping",
                model_id.trim()
            ))
        }
    };

    Ok(WhisperDownloadPlan {
        repository: "ggerganov/whisper.cpp",
        display_name: match normalized.as_str() {
            "tiny" => "tiny model",
            "base" => "base model",
            "small" => "small model",
            "medium" => "medium model",
            "large-v3" => "large-v3 model",
            "large-v3-turbo" => "large-v3-turbo model",
            _ => unreachable!(),
        },
        file_name,
    })
}

fn active_local_tts_profile(config: &AppConfig) -> Result<(String, &LocalTtsProfile), String> {
    let profile_name = config
        .providers
        .tts
        .local_profile
        .clone()
        .ok_or_else(|| String::from("No local TTS profile is configured."))?;
    let profile = config
        .local_tts_profiles
        .get(&profile_name)
        .ok_or_else(|| format!("Configured local TTS profile '{profile_name}' was not found."))?;
    Ok((profile_name, profile))
}

fn active_local_asr_profile(config: &AppConfig) -> Result<(String, &LocalAsrProfile), String> {
    let profile_name = config
        .providers
        .asr
        .local_profile
        .clone()
        .ok_or_else(|| String::from("No local ASR profile is configured."))?;
    let profile = config
        .local_asr_profiles
        .get(&profile_name)
        .ok_or_else(|| format!("Configured local ASR profile '{profile_name}' was not found."))?;
    Ok((profile_name, profile))
}

fn resolved_models_dir_for_app(
    app_handle: &AppHandle,
    configured_models_dir: &str,
) -> Result<PathBuf, String> {
    let trimmed = configured_models_dir.trim();
    if trimmed.is_empty() {
        return Err(String::from("Configured models_dir must not be empty."));
    }

    if let Some(relative_to_home) = trimmed.strip_prefix("~/") {
        let Some(home_dir) = app_handle.path().home_dir().ok() else {
            return Err(String::from(
                "Failed to resolve the current user's home directory.",
            ));
        };
        return Ok(home_dir.join(relative_to_home));
    }

    let candidate = PathBuf::from(trimmed);
    if candidate.is_absolute() {
        return Ok(candidate);
    }

    let config_path =
        AppConfig::config_path_for_app(app_handle).map_err(|error| error.to_string())?;
    let config_dir = config_path.parent().ok_or_else(|| {
        format!(
            "Failed to resolve the parent config directory for {}",
            config_path.display()
        )
    })?;
    Ok(config_dir.join(candidate))
}

fn download_hugging_face_directory(
    target_dir: &Path,
    repository: &str,
    files: &[&str],
) -> Result<(), String> {
    fs::create_dir_all(target_dir).map_err(|error| {
        format!(
            "Failed to create model directory {}: {error}",
            target_dir.display()
        )
    })?;
    for file_name in files {
        let target_path = target_dir.join(file_name);
        download_hugging_face_file(&target_path, repository, file_name)?;
    }
    Ok(())
}

fn download_hugging_face_file(
    target_path: &Path,
    repository: &str,
    file_name: &str,
) -> Result<(), String> {
    let parent = target_path.parent().ok_or_else(|| {
        format!(
            "Failed to resolve the parent directory for download target {}",
            target_path.display()
        )
    })?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Failed to create directory {}: {error}", parent.display()))?;

    let url = format!("https://huggingface.co/{repository}/resolve/main/{file_name}");
    let client = Client::builder()
        .build()
        .map_err(|error| format!("Failed to create the download client: {error}"))?;
    let mut response = client
        .get(&url)
        .send()
        .map_err(|error| format!("Failed to download {url}: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Hugging Face returned {} while downloading {}",
            response.status(),
            url
        ));
    }

    let mut output = fs::File::create(target_path)
        .map_err(|error| format!("Failed to create {}: {error}", target_path.display()))?;
    response
        .copy_to(&mut output)
        .map_err(|error| format!("Failed to write {}: {error}", target_path.display()))?;
    Ok(())
}

fn build_tts_voice_settings(
    config: &AppConfig,
    runtime_audio: &RuntimeAudioState,
) -> TtsVoiceSettings {
    let mode = config.providers.tts.mode.clone();
    let mut available_voices = match mode {
        crate::config::ProviderMode::Local => KITTEN_TTS_VOICES
            .iter()
            .map(|voice| TtsVoiceOption {
                voice_name: (*voice).to_string(),
                display_label: (*voice).to_string(),
            })
            .collect(),
        crate::config::ProviderMode::Remote => {
            let active_remote_profile = config
                .providers
                .tts
                .remote_profile
                .as_ref()
                .and_then(|profile_name| config.remote_tts_profiles.get(profile_name));
            match active_remote_profile.map(|profile| &profile.provider) {
                Some(RemoteProviderKind::OpenAi) => OPENAI_TTS_VOICES
                    .iter()
                    .map(|voice| TtsVoiceOption {
                        voice_name: (*voice).to_string(),
                        display_label: (*voice).to_string(),
                    })
                    .collect(),
                Some(_) => active_remote_profile
                    .map(|profile| {
                        vec![TtsVoiceOption {
                            voice_name: profile.voice.clone(),
                            display_label: profile.voice.clone(),
                        }]
                    })
                    .unwrap_or_default(),
                None => Vec::new(),
            }
        }
    };
    let mut active_voice = runtime_audio
        .tts_voice
        .as_deref()
        .map(str::trim)
        .filter(|voice| !voice.is_empty())
        .map(ToOwned::to_owned);

    if let Some(current_voice) = active_voice.clone() {
        if let Some(option) = available_voices
            .iter()
            .find(|option| option.voice_name.eq_ignore_ascii_case(&current_voice))
        {
            active_voice = Some(option.voice_name.clone());
        } else {
            available_voices.insert(
                0,
                TtsVoiceOption {
                    voice_name: current_voice.clone(),
                    display_label: current_voice,
                },
            );
        }
    }

    TtsVoiceSettings {
        mode,
        active_voice,
        available_voices,
    }
}

impl DeterministicToolExecutor for AppCore {
    fn execute_open_url(&mut self, input: OpenUrlInput) -> ToolResult<OpenUrlData> {
        AppCore::execute_open_url(self, input)
    }

    fn execute_read_region(&mut self, input: ReadRegionInput) -> ToolResult<ReadRegionData> {
        AppCore::execute_read_region(self, input)
    }

    fn execute_read_next_region(
        &mut self,
        input: ReadNextRegionInput,
    ) -> ToolResult<ReadNextRegionData> {
        AppCore::execute_read_next_region(self, input)
    }

    fn execute_read_previous_region(
        &mut self,
        input: ReadPreviousRegionInput,
    ) -> ToolResult<ReadPreviousRegionData> {
        AppCore::execute_read_previous_region(self, input)
    }

    fn execute_stop_speaking(&mut self, input: StopSpeakingInput) -> ToolResult<StopSpeakingData> {
        AppCore::execute_stop_speaking(self, input)
    }

    fn execute_start_listening(
        &mut self,
        input: StartListeningInput,
    ) -> ToolResult<StartListeningData> {
        AppCore::execute_start_listening(self, input)
    }

    fn execute_stop_listening(
        &mut self,
        input: StopListeningInput,
    ) -> ToolResult<StopListeningData> {
        AppCore::execute_stop_listening(self, input)
    }

    fn execute_transcribe_command(
        &mut self,
        input: TranscribeCommandInput,
    ) -> ToolResult<TranscribeCommandData> {
        AppCore::execute_transcribe_command(self, input)
    }

    fn execute_go_back(&mut self, input: GoBackInput) -> ToolResult<GoBackData> {
        AppCore::execute_go_back(self, input)
    }

    fn execute_go_forward(&mut self, input: GoForwardInput) -> ToolResult<GoForwardData> {
        AppCore::execute_go_forward(self, input)
    }

    fn execute_reload_page(&mut self, input: ReloadPageInput) -> ToolResult<ReloadPageData> {
        AppCore::execute_reload_page(self, input)
    }

    fn execute_get_html(&mut self, input: GetHtmlInput) -> ToolResult<GetHtmlData> {
        AppCore::execute_get_html(self, input)
    }

    fn execute_eval_js(&mut self, input: EvalJsInput) -> ToolResult<EvalJsData> {
        AppCore::execute_eval_js(self, input)
    }

    fn execute_scroll_page(&mut self, input: ScrollPageInput) -> ToolResult<ScrollPageData> {
        AppCore::execute_scroll_page(self, input)
    }

    fn execute_capture_screenshot(
        &mut self,
        input: CaptureScreenshotInput,
    ) -> ToolResult<CaptureScreenshotData> {
        AppCore::execute_capture_screenshot(self, input)
    }

    fn execute_run_ocr(&mut self, input: RunOcrInput) -> ToolResult<RunOcrData> {
        AppCore::execute_run_ocr(self, input)
    }

    fn execute_merge_ocr_into_page_model(
        &mut self,
        input: MergeOcrIntoPageModelInput,
    ) -> ToolResult<MergeOcrIntoPageModelData> {
        AppCore::execute_merge_ocr_into_page_model(self, input)
    }

    fn execute_list_interactive_elements(
        &mut self,
        input: ListInteractiveElementsInput,
    ) -> ToolResult<ListInteractiveElementsData> {
        AppCore::execute_list_interactive_elements(self, input)
    }

    fn execute_find_element(&mut self, input: FindElementInput) -> ToolResult<FindElementData> {
        AppCore::execute_find_element(self, input)
    }

    fn execute_click_element(&mut self, input: ClickElementInput) -> ToolResult<ClickElementData> {
        AppCore::execute_click_element(self, input)
    }

    fn execute_focus_element(&mut self, input: FocusElementInput) -> ToolResult<FocusElementData> {
        AppCore::execute_focus_element(self, input)
    }

    fn execute_type_into_element(
        &mut self,
        input: TypeIntoElementInput,
    ) -> ToolResult<TypeIntoElementData> {
        AppCore::execute_type_into_element(self, input)
    }

    fn execute_submit_active_form(
        &mut self,
        input: SubmitActiveFormInput,
    ) -> ToolResult<SubmitActiveFormData> {
        AppCore::execute_submit_active_form(self, input)
    }

    fn execute_extract_page_model(
        &mut self,
        input: ExtractPageModelInput,
    ) -> ToolResult<ExtractPageModelData> {
        AppCore::execute_extract_page_model(self, input)
    }

    fn execute_get_page_snapshot(
        &mut self,
        input: GetPageSnapshotInput,
    ) -> ToolResult<PageSnapshotData> {
        AppCore::execute_get_page_snapshot(self, input)
    }

    fn execute_set_tts_voice(&mut self, input: SetTtsVoiceInput) -> ToolResult<SetTtsVoiceData> {
        AppCore::execute_set_tts_voice(self, input)
    }

    fn execute_set_playback_volume(
        &mut self,
        input: SetPlaybackVolumeInput,
    ) -> ToolResult<SetPlaybackVolumeData> {
        AppCore::execute_set_playback_volume(self, input)
    }

    fn execute_set_playback_speed(
        &mut self,
        input: SetPlaybackSpeedInput,
    ) -> ToolResult<SetPlaybackSpeedData> {
        AppCore::execute_set_playback_speed(self, input)
    }

    fn execute_set_browser_visibility(
        &mut self,
        input: SetBrowserVisibilityInput,
    ) -> ToolResult<SetBrowserVisibilityData> {
        AppCore::execute_set_browser_visibility(self, input)
    }

    fn execute_get_agent_state(&mut self, input: GetAgentStateInput) -> ToolResult<AgentStateData> {
        AppCore::execute_get_agent_state(self, input)
    }

    fn execute_get_runtime_status(
        &mut self,
        input: GetRuntimeStatusInput,
    ) -> ToolResult<GetRuntimeStatusData> {
        AppCore::execute_get_runtime_status(self, input)
    }

    fn execute_confirm_action(
        &mut self,
        input: ConfirmActionInput,
    ) -> ToolResult<ConfirmActionData> {
        AppCore::execute_confirm_action(self, input)
    }

    fn execute_report_result(&mut self, input: ReportResultInput) -> ToolResult<ReportResultData> {
        AppCore::execute_report_result(self, input)
    }
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn region_bbox_by_id(regions: &[PageRegion], region_id: &str) -> Result<Rect, ToolError> {
    let Some(region_index) = find_region_index(regions, region_id) else {
        return Err(ToolError {
            code: String::from("unknown_region_id"),
            message: String::from(
                "capture_screenshot could not find the requested region_id in the current page model",
            ),
            retryable: false,
            details: Some(serde_json::json!({ "region_id": region_id })),
        });
    };

    let region = &regions[region_index];
    let Some(bbox) = region.bbox.clone() else {
        return Err(ToolError {
            code: String::from("missing_region_bbox"),
            message: String::from(
                "capture_screenshot requires a bounding box for the requested region_id",
            ),
            retryable: false,
            details: Some(serde_json::json!({ "region_id": region_id })),
        });
    };

    if bbox.width <= 0.0 || bbox.height <= 0.0 {
        return Err(ToolError {
            code: String::from("invalid_region_bbox"),
            message: String::from(
                "capture_screenshot requires a positive bounding box for the requested region_id",
            ),
            retryable: false,
            details: Some(serde_json::json!({
                "region_id": region_id,
                "x": bbox.x,
                "y": bbox.y,
                "width": bbox.width,
                "height": bbox.height,
            })),
        });
    }

    Ok(bbox)
}

fn merge_ocr_text_into_page_model(
    page: &mut PageModel,
    region_id: Option<&str>,
    ocr_text: &str,
    source_bbox: Option<Rect>,
    next_region_id: String,
) -> Result<Vec<String>, ToolError> {
    let normalized_text = ocr_text.trim();
    if normalized_text.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_ocr_text"),
            message: String::from("merge_ocr_into_page_model requires non-empty ocr_text"),
            retryable: false,
            details: None,
        });
    }

    if let Some(region_id) = region_id {
        let Some(region) = page
            .regions
            .iter_mut()
            .find(|region| region.region_id == region_id)
        else {
            return Err(ToolError {
                code: String::from("unknown_region_id"),
                message: String::from(
                    "merge_ocr_into_page_model requires a region_id that exists in the current page model",
                ),
                retryable: false,
                details: Some(serde_json::json!({ "region_id": region_id })),
            });
        };

        region.text = merged_region_text(&region.text, normalized_text);
        region.source = match region.source {
            RegionSource::Dom | RegionSource::Mixed => RegionSource::Mixed,
            RegionSource::Ocr => RegionSource::Ocr,
        };
        if region.bbox.is_none() {
            region.bbox = source_bbox;
        }

        Ok(vec![region.region_id.clone()])
    } else {
        let region_id = next_region_id;
        page.regions.push(PageRegion {
            region_id: region_id.clone(),
            role: RegionRole::Other,
            label: None,
            text: normalized_text.to_string(),
            bbox: source_bbox,
            source: RegionSource::Ocr,
        });
        Ok(vec![region_id])
    }
}

fn extracted_text_metrics(page: &PageModel) -> (usize, usize) {
    page.regions
        .iter()
        .fold((0usize, 0usize), |(chars, regions), region| {
            let trimmed = region.text.trim();
            if trimmed.is_empty() {
                (chars, regions)
            } else {
                (chars + trimmed.chars().count(), regions + 1)
            }
        })
}

fn has_positive_bbox(region: &PageRegion) -> bool {
    matches!(
        region.bbox,
        Some(Rect {
            width,
            height,
            ..
        }) if width > 0.0 && height > 0.0
    )
}

fn region_first_ocr_target_ids(page: &PageModel, ocr_settings: &OcrSettings) -> Vec<String> {
    if !ocr_settings.prefer_region_ocr {
        return Vec::new();
    }

    page.regions
        .iter()
        .filter(|region| !region.text.trim().is_empty() && has_positive_bbox(region))
        .map(|region| region.region_id.clone())
        .collect()
}

fn should_trigger_extract_page_model_ocr_fallback(
    use_dom_extraction: bool,
    page: &PageModel,
    ocr_settings: &OcrSettings,
) -> bool {
    if !use_dom_extraction || !ocr_settings.trigger_on_no_extractable_text {
        return false;
    }

    let (readable_char_count, readable_region_count) = extracted_text_metrics(page);

    readable_region_count == 0
        || readable_char_count <= ocr_settings.sparse_text_char_threshold as usize
        || readable_region_count < ocr_settings.sparse_text_region_threshold as usize
}

fn merged_region_text(existing_text: &str, ocr_text: &str) -> String {
    let existing_text = existing_text.trim();
    let ocr_text = ocr_text.trim();

    if existing_text.is_empty() {
        return ocr_text.to_string();
    }
    if ocr_text.is_empty() {
        return existing_text.to_string();
    }
    if existing_text == ocr_text {
        return existing_text.to_string();
    }
    if existing_text.contains(ocr_text) {
        return existing_text.to_string();
    }
    if ocr_text.contains(existing_text) {
        return ocr_text.to_string();
    }

    format!("{existing_text}\n\n{ocr_text}")
}

fn ocr_runtime_error_to_tool_error(error: &OcrRuntimeError) -> ToolError {
    let code = match error {
        OcrRuntimeError::FeatureUnavailable => "ocr_backend_unavailable",
        OcrRuntimeError::EngineInitFailed { .. } => "ocr_engine_init_failed",
        OcrRuntimeError::ImageLoadFailed { .. } => "ocr_image_load_failed",
        OcrRuntimeError::InvalidBbox => "invalid_ocr_bbox",
        OcrRuntimeError::TextExtractionFailed { .. } => "ocr_text_extraction_failed",
    };

    ToolError {
        code: String::from(code),
        message: error.to_string(),
        retryable: matches!(
            error,
            OcrRuntimeError::EngineInitFailed { .. } | OcrRuntimeError::TextExtractionFailed { .. }
        ),
        details: None,
    }
}

fn extract_page_model_internal_failure(
    request_id: String,
    message: String,
    observations: Vec<String>,
) -> ToolResult<ExtractPageModelData> {
    ToolResult::failure(
        ToolName::ExtractPageModel,
        request_id,
        ToolError {
            code: String::from("extract_page_model_internal_error"),
            message,
            retryable: false,
            details: None,
        },
        observations,
    )
}

fn nested_tool_failure_as_extract_page_model<T>(
    request_id: String,
    mut observations: Vec<String>,
    nested_result: ToolResult<T>,
    failure_observation: String,
) -> ToolResult<ExtractPageModelData> {
    observations.extend(nested_result.observations);
    observations.push(failure_observation);

    let error = nested_result.error.unwrap_or(ToolError {
        code: String::from("extract_page_model_internal_error"),
        message: String::from("nested OCR fallback tool failed without returning a ToolError"),
        retryable: false,
        details: None,
    });

    ToolResult::failure(ToolName::ExtractPageModel, request_id, error, observations)
}

fn tts_runtime_error_to_tool_error(error: TtsRuntimeError) -> ToolError {
    match error {
        TtsRuntimeError::EmptyNarrationText => ToolError {
            code: String::from("empty_narration_text"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        TtsRuntimeError::MissingLocalProfile
        | TtsRuntimeError::MissingLocalProfileDefinition { .. }
        | TtsRuntimeError::MissingRemoteProfile
        | TtsRuntimeError::MissingRemoteProfileDefinition { .. } => ToolError {
            code: String::from("tts_profile_unavailable"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        TtsRuntimeError::UnsupportedRemoteProvider { .. } => ToolError {
            code: String::from("unsupported_tts_provider"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        TtsRuntimeError::LocalTtsFeatureUnavailable
        | TtsRuntimeError::RemoteTtsFeatureUnavailable => ToolError {
            code: String::from("tts_backend_unavailable"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        TtsRuntimeError::EmptyRemoteVoice => ToolError {
            code: String::from("tts_voice_unavailable"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        TtsRuntimeError::RemoteSecretUnavailable { .. } => ToolError {
            code: String::from("tts_secret_unavailable"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        TtsRuntimeError::RemoteRequestBuildFailed { .. } => ToolError {
            code: String::from("tts_request_build_failed"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        TtsRuntimeError::RemoteRequestFailed { .. } => ToolError {
            code: String::from("tts_request_failed"),
            message: error.to_string(),
            retryable: true,
            details: None,
        },
        TtsRuntimeError::RemoteResponseDecodeFailed { .. } => ToolError {
            code: String::from("tts_response_invalid"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        TtsRuntimeError::EmptyLocalModelPath | TtsRuntimeError::MissingLocalModelPath { .. } => {
            ToolError {
                code: String::from("tts_model_unavailable"),
                message: error.to_string(),
                retryable: false,
                details: None,
            }
        }
        TtsRuntimeError::UnsupportedLocalSampleRate { .. } => ToolError {
            code: String::from("unsupported_tts_sample_rate"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        TtsRuntimeError::LocalModelLoad { .. } => ToolError {
            code: String::from("tts_model_load_failed"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        TtsRuntimeError::SynthesisFailed { .. } => ToolError {
            code: String::from("tts_synthesis_failed"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
    }
}

fn audio_playback_error_to_tool_error(error: AudioPlaybackError) -> ToolError {
    match error {
        AudioPlaybackError::AudioFeatureUnavailable => ToolError {
            code: String::from("audio_backend_unavailable"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        AudioPlaybackError::InvalidChannelCount | AudioPlaybackError::InvalidSampleRate => {
            ToolError {
                code: String::from("invalid_audio_output"),
                message: error.to_string(),
                retryable: false,
                details: None,
            }
        }
        AudioPlaybackError::OpenDevice { .. } => ToolError {
            code: String::from("audio_output_unavailable"),
            message: error.to_string(),
            retryable: true,
            details: None,
        },
    }
}

fn asr_runtime_error_to_tool_error(error: &AsrRuntimeError) -> ToolError {
    let code = match error {
        AsrRuntimeError::MissingLocalProfile
        | AsrRuntimeError::MissingLocalProfileDefinition { .. }
        | AsrRuntimeError::MissingRemoteProfile
        | AsrRuntimeError::MissingRemoteProfileDefinition { .. } => "asr_profile_unavailable",
        AsrRuntimeError::UnsupportedRemoteProvider { .. } => "unsupported_asr_provider",
        AsrRuntimeError::AudioFeatureUnavailable => "audio_backend_unavailable",
        AsrRuntimeError::LocalAsrFeatureUnavailable
        | AsrRuntimeError::RemoteAsrFeatureUnavailable => "asr_backend_unavailable",
        AsrRuntimeError::MissingInputDevice => "audio_input_unavailable",
        AsrRuntimeError::InputConfigUnavailable { .. } => "audio_input_config_unavailable",
        AsrRuntimeError::UnsupportedInputSampleFormat { .. } => "unsupported_audio_input_format",
        AsrRuntimeError::BuildInputStream { .. } => "audio_input_stream_build_failed",
        AsrRuntimeError::StartInputStream { .. } => "audio_input_stream_start_failed",
        AsrRuntimeError::AudioBufferLockFailed => "audio_buffer_lock_failed",
        AsrRuntimeError::EmptyLocalModelPath | AsrRuntimeError::MissingLocalModelPath { .. } => {
            "asr_model_unavailable"
        }
        AsrRuntimeError::RemoteSecretUnavailable { .. } => "asr_secret_unavailable",
        AsrRuntimeError::RemoteAudioEncodeFailed { .. } => "invalid_audio_input",
        AsrRuntimeError::RemoteRequestBuildFailed { .. } => "asr_request_build_failed",
        AsrRuntimeError::RemoteRequestTimedOut { .. } => "asr_request_timed_out",
        AsrRuntimeError::RemoteRequestFailed { .. } => "asr_request_failed",
        AsrRuntimeError::LocalModelLoad { .. } => "asr_model_load_failed",
        AsrRuntimeError::NoAudioCaptured => "no_audio_captured",
        AsrRuntimeError::TranscriptionFailed { .. } => "asr_transcription_failed",
    };

    ToolError {
        code: String::from(code),
        message: error.to_string(),
        retryable: matches!(
            error,
            AsrRuntimeError::MissingInputDevice
                | AsrRuntimeError::InputConfigUnavailable { .. }
                | AsrRuntimeError::BuildInputStream { .. }
                | AsrRuntimeError::StartInputStream { .. }
                | AsrRuntimeError::AudioBufferLockFailed
                | AsrRuntimeError::NoAudioCaptured
                | AsrRuntimeError::RemoteRequestTimedOut { .. }
                | AsrRuntimeError::RemoteRequestFailed { .. }
        ),
        details: None,
    }
}

#[cfg(any(feature = "remote-openai", test))]
fn planner_system_prompt() -> &'static str {
    "You are the bounded planner for blind_browser, a voice-first desktop browser for vision-impaired users.
Return only JSON that matches the provided planner_output_schema.
Use only tool names that appear in planner_input.available_tools and only selected_skills that appear in planner_input.active_skill_names.
Every step arguments object must match the corresponding tool_input_schemas entry exactly, including snake_case field names.
Use canonical_planner_output_examples only as shape references; adapt the returned tools, skills, and arguments to the current planner_input.
Keep plans linear and short: at most five steps, with at most one NextStep edge from any step.
When planner_input.safety.allow_click_without_confirmation is true, ordinary ClickElement plans may use Ready without confirm_action; reserve NeedsConfirmation for clicks whose grounded confidence falls below planner_input.safety.confirmation_confidence_threshold or remains ambiguous/risky.
Use NeedsConfirmation plus a confirm_action step when the request is risky or ambiguous before side effects, and do not use confirm_action or confirmation metadata on Ready, Blocked, or Complete plans.
SubmitForm plans must always use NeedsConfirmation with confirm_action before any submit side effect.
Use Blocked only when the request cannot be grounded safely or is outside the supported tool set.
Do not invent tools, skills, statuses, transition kinds, or argument fields."
}

fn normalize_absolute_url(url: &str) -> Result<String, ToolError> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_url"),
            message: String::from("open_url requires a non-empty absolute URL"),
            retryable: false,
            details: None,
        });
    }

    let Some(separator_index) = trimmed.find(':') else {
        return Err(ToolError {
            code: String::from("invalid_url"),
            message: String::from("open_url requires an absolute URL with a scheme"),
            retryable: false,
            details: Some(serde_json::json!({ "url": trimmed })),
        });
    };

    let scheme = &trimmed[..separator_index];
    let remainder = &trimmed[separator_index + 1..];
    let valid_scheme = scheme.chars().enumerate().all(|(index, ch)| match index {
        0 => ch.is_ascii_alphabetic(),
        _ => ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'),
    });

    if !valid_scheme || remainder.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_url"),
            message: String::from("open_url requires an absolute URL with a valid scheme"),
            retryable: false,
            details: Some(serde_json::json!({ "url": trimmed })),
        });
    }

    Ok(trimmed.to_string())
}

fn refresh_current_page_after_navigation(
    current_page: &mut Option<PageModel>,
    url: Option<String>,
    title: Option<String>,
) {
    if let Some(current_page) = current_page.as_mut() {
        current_page.url = url;
        current_page.title = title;
        current_page.regions.clear();
        current_page.interactive_elements.clear();
    }
}

fn clear_navigation_follow_up_state(
    state: &mut AppState,
    recent_field_context: &mut Option<RecentFieldContext>,
) {
    state.narration_cursor = Default::default();
    *recent_field_context = None;
}

fn resolve_direct_focus_field_command(
    transcript: &str,
    request_id: &str,
    current_page: Option<&PageModel>,
    active_skill_names: &[String],
    confirmation_confidence_threshold: f32,
) -> Option<PlannerOutput> {
    let command = parse_direct_focus_field_command(transcript)?;
    let selected_skills = if active_skill_names
        .iter()
        .any(|active_name| active_name == "focus_field")
    {
        vec![String::from("focus_field")]
    } else {
        Vec::new()
    };

    let Some(description) = command.description else {
        let summary = String::from("Please tell me which field to focus.");
        return Some(build_direct_follow_up_output(
            request_id,
            DirectFollowUpSpec {
                intent_name: IntentName::FillInput,
                goal: String::from("Focus the requested field."),
                target_description: Some(String::from("field focus target")),
                selected_skills,
                summary,
                next_recommended_action: Some(String::from(
                    "Say the field name, like focus the email field.",
                )),
                step_id: String::from("report-missing-focus-field-description"),
                purpose: String::from("Report that the field name is required before focusing."),
            },
        ));
    };

    let Some(current_page) = current_page else {
        let summary = String::from("There is no current page to focus a field on yet.");
        return Some(build_direct_follow_up_output(
            request_id,
            DirectFollowUpSpec {
                intent_name: IntentName::FillInput,
                goal: String::from("Focus the requested field."),
                target_description: Some(String::from("current page field")),
                selected_skills,
                summary,
                next_recommended_action: Some(String::from(
                    "Open a page first, then ask me to focus a field.",
                )),
                step_id: String::from("report-missing-focus-page"),
                purpose: String::from(
                    "Report that there is no active page available for field focus.",
                ),
            },
        ));
    };

    let field_elements = focusable_field_elements(current_page);
    if field_elements.is_empty() {
        let summary = String::from("I could not find any focusable fields on the current page.");
        return Some(build_direct_follow_up_output(
            request_id,
            DirectFollowUpSpec {
                intent_name: IntentName::FillInput,
                goal: String::from("Focus the requested field."),
                target_description: Some(description.clone()),
                selected_skills,
                summary,
                next_recommended_action: Some(String::from(
                    "Try again after the page finishes loading or becomes interactive.",
                )),
                step_id: String::from("report-missing-focusable-fields"),
                purpose: String::from(
                    "Report that no focusable fields are available on the current page.",
                ),
            },
        ));
    }

    let query = FindElementInput {
        request_id: request_id.to_string(),
        timeout_ms: None,
        description: description.clone(),
        text: None,
        role: None,
        color_hint: None,
        nearby_text: None,
        selector_hint: None,
        visibility_filter: crate::commands::ElementVisibilityFilter::VisibleOnly,
        max_candidates: Some(DEFAULT_FIND_ELEMENT_MAX_CANDIDATES),
    };
    let search_query = build_find_element_query(&query).ok()?;
    let candidates = rank_find_element_candidates(
        &field_elements,
        &search_query,
        DEFAULT_FIND_ELEMENT_MAX_CANDIDATES,
    );
    let (chosen_element_id, _, requires_confirmation) = if candidates.len() == 1 {
        (Some(candidates[0].element_id.clone()), None, false)
    } else {
        determine_find_element_resolution(&candidates, confirmation_confidence_threshold)
    };

    if let Some(element_id) = chosen_element_id {
        return Some(PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::FillInput,
                goal: String::from("Focus the requested field."),
                target_description: Some(description),
            },
            selected_skills,
            steps: vec![PlannedStep {
                step_id: String::from("focus-field"),
                tool_name: ToolName::FocusElement,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "element_id": element_id
                }),
                purpose: String::from("Move focus to the requested field."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        });
    }

    let summary = if requires_confirmation {
        let candidate_names = summarize_candidate_names(current_page, &candidates);
        if candidate_names.is_empty() {
            format!("I found multiple possible fields for {description}. Please be more specific.")
        } else {
            format!(
                "I found multiple possible fields for {description}: {}. Please be more specific.",
                candidate_names.join(", ")
            )
        }
    } else {
        format!("I could not find a visible field matching {description}.")
    };

    Some(build_direct_follow_up_output(
        request_id,
        DirectFollowUpSpec {
            intent_name: IntentName::FillInput,
            goal: String::from("Focus the requested field."),
            target_description: Some(description),
            selected_skills,
            summary,
            next_recommended_action: Some(String::from(
                "Try naming the field label or placeholder more specifically.",
            )),
            step_id: String::from("report-focus-field-follow-up"),
            purpose: String::from(
                "Report that the requested field could not be focused deterministically.",
            ),
        },
    ))
}

fn selected_skills_for_fill_command(
    active_skill_names: &[String],
    submit_after: bool,
) -> Vec<String> {
    let expected_skill_name = if submit_after {
        "fill_and_submit_form"
    } else {
        "fill_field_by_label"
    };

    if active_skill_names
        .iter()
        .any(|active_name| active_name == expected_skill_name)
    {
        vec![expected_skill_name.to_string()]
    } else {
        Vec::new()
    }
}

fn build_direct_fill_ready_output(
    request_id: &str,
    selected_skills: Vec<String>,
    description: Option<String>,
    element_id: String,
    text: String,
) -> PlannerOutput {
    PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::FillInput,
            goal: String::from("Fill the requested field."),
            target_description: description,
        },
        selected_skills,
        steps: vec![
            PlannedStep {
                step_id: String::from("focus-fill-field"),
                tool_name: ToolName::FocusElement,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "element_id": element_id
                }),
                purpose: String::from("Move focus to the requested field before typing."),
                on_success: StepTransition::NextStep {
                    step_id: String::from("type-into-fill-field"),
                },
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from("type-into-fill-field"),
                tool_name: ToolName::TypeIntoElement,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "element_id": element_id,
                    "text": text,
                    "text_entry_mode": "Replace",
                    "submit_mode": "KeepEditing"
                }),
                purpose: String::from(
                    "Replace the requested field contents with the spoken value.",
                ),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    }
}

fn build_direct_fill_and_submit_ready_output(
    request_id: &str,
    selected_skills: Vec<String>,
    description: Option<String>,
    element_id: String,
    text: String,
) -> PlannerOutput {
    let description_text = description
        .clone()
        .unwrap_or_else(|| String::from("requested"));
    let prompt_text = format!(
        "Do you want me to fill the {description_text} field with {text} and then submit that form?"
    );
    let confirmation_reason =
        String::from("filling the field and submitting the form may change or send data");

    PlannerOutput {
        status: PlannerStatus::NeedsConfirmation,
        intent: IntentSummary {
            name: IntentName::SubmitForm,
            goal: String::from("Fill the requested field and submit the form."),
            target_description: description,
        },
        selected_skills,
        steps: vec![
            PlannedStep {
                step_id: String::from("confirm-fill-and-submit-form"),
                tool_name: ToolName::ConfirmAction,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "prompt_text": prompt_text,
                    "reason": confirmation_reason
                }),
                purpose: String::from(
                    "Require explicit confirmation before filling the field and submitting the form.",
                ),
                on_success: StepTransition::RequestConfirmation,
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from("focus-fill-submit-field"),
                tool_name: ToolName::FocusElement,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "element_id": element_id
                }),
                purpose: String::from("Move focus to the requested field before typing."),
                on_success: StepTransition::NextStep {
                    step_id: String::from("type-fill-submit-field"),
                },
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from("type-fill-submit-field"),
                tool_name: ToolName::TypeIntoElement,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "element_id": element_id,
                    "text": text,
                    "text_entry_mode": "Replace",
                    "submit_mode": "KeepEditing"
                }),
                purpose: String::from(
                    "Replace the requested field contents with the spoken value before submission.",
                ),
                on_success: StepTransition::NextStep {
                    step_id: String::from("submit-fill-submit-form"),
                },
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from("submit-fill-submit-form"),
                tool_name: ToolName::SubmitActiveForm,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "form_element_id": serde_json::Value::Null
                }),
                purpose: String::from(
                    "Submit the form that owns the focused field after the fill step succeeds.",
                ),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ],
        requires_confirmation: true,
        confirmation_reason: Some(String::from(
            "filling the field and submitting the form may change or send data",
        )),
        blocked_reason: None,
        user_message: Some(String::from(
            "Please confirm before I fill the field and submit the form.",
        )),
    }
}

fn resolve_recent_fill_correction_command(
    transcript: &str,
    request_id: &str,
    current_page_id: Option<&str>,
    current_page: Option<&PageModel>,
    active_skill_names: &[String],
    recent_field_context: Option<&RecentFieldContext>,
) -> Option<(PlannerOutput, Option<RecentFieldContext>)> {
    let correction = parse_fill_field_correction_command(transcript)?;
    let matching_context = recent_field_context.filter(|context| {
        current_page_id.is_some_and(|page_id| context.page_id == page_id) && current_page.is_some()
    });

    match correction {
        FillFieldCorrectionCommand::AlternateField => {
            let Some(context) = matching_context else {
                let planner_output = build_direct_follow_up_output(
                    request_id,
                    DirectFollowUpSpec {
                        intent_name: IntentName::FillInput,
                        goal: String::from("Fill the requested field."),
                        target_description: Some(String::from("field fill target")),
                        selected_skills: selected_skills_for_fill_command(active_skill_names, false),
                        summary: String::from(
                            "Please tell me which field you want me to use instead.",
                        ),
                        next_recommended_action: Some(String::from(
                            "Name the field label or placeholder, like use the billing email field instead.",
                        )),
                        step_id: String::from("report-missing-alternate-field-context"),
                        purpose: String::from(
                            "Report that the alternate field cannot be resolved without recent context.",
                        ),
                    },
                );
                return Some((planner_output, None));
            };

            let current_page = current_page?;
            let Some(active_element_id) = context.active_element_id.as_deref() else {
                let planner_output = build_direct_follow_up_output(
                    request_id,
                    DirectFollowUpSpec {
                        intent_name: IntentName::FillInput,
                        goal: String::from("Fill the requested field."),
                        target_description: context.target_description.clone(),
                        selected_skills: selected_skills_for_fill_command(
                            active_skill_names,
                            context.submit_after,
                        ),
                        summary: String::from(
                            "Please tell me which field you mean before I switch to another one.",
                        ),
                        next_recommended_action: Some(String::from(
                            "Name the specific field label or placeholder you want me to use.",
                        )),
                        step_id: String::from("report-missing-active-field-context"),
                        purpose: String::from(
                            "Report that there is no recent resolved field target to swap away from.",
                        ),
                    },
                );
                return Some((planner_output, Some(context.clone())));
            };

            let alternate_element_id = context
                .candidate_element_ids
                .iter()
                .find(|candidate_id| candidate_id.as_str() != active_element_id)
                .and_then(|candidate_id| {
                    resolve_typeable_element(current_page, candidate_id)
                        .ok()
                        .map(|_| candidate_id.clone())
                });
            let Some(alternate_element_id) = alternate_element_id else {
                let planner_output = build_direct_follow_up_output(
                    request_id,
                    DirectFollowUpSpec {
                        intent_name: IntentName::FillInput,
                        goal: String::from("Fill the requested field."),
                        target_description: context.target_description.clone(),
                        selected_skills: selected_skills_for_fill_command(
                            active_skill_names,
                            context.submit_after,
                        ),
                        summary: String::from(
                            "Please tell me which field you want after all.",
                        ),
                        next_recommended_action: Some(String::from(
                            "Name the specific field label or placeholder so I can target it deterministically.",
                        )),
                        step_id: String::from("report-missing-alternate-field-target"),
                        purpose: String::from(
                            "Report that no alternate recent field target is available anymore.",
                        ),
                    },
                );
                return Some((planner_output, Some(context.clone())));
            };

            let Some(text) = context.pending_text.clone() else {
                let planner_output = build_direct_follow_up_output(
                    request_id,
                    DirectFollowUpSpec {
                        intent_name: IntentName::FillInput,
                        goal: String::from("Fill the requested field."),
                        target_description: context.target_description.clone(),
                        selected_skills: selected_skills_for_fill_command(
                            active_skill_names,
                            context.submit_after,
                        ),
                        summary: String::from(
                            "Please tell me what text to enter before I switch fields.",
                        ),
                        next_recommended_action: Some(String::from(
                            "Say the value you want me to type after naming the field.",
                        )),
                        step_id: String::from("report-missing-alternate-field-text"),
                        purpose: String::from(
                            "Report that the original field value is no longer available for the alternate target.",
                        ),
                    },
                );
                return Some((planner_output, Some(context.clone())));
            };

            let planner_output = if context.submit_after {
                build_direct_fill_and_submit_ready_output(
                    request_id,
                    selected_skills_for_fill_command(active_skill_names, true),
                    context.target_description.clone(),
                    alternate_element_id.clone(),
                    text.clone(),
                )
            } else {
                build_direct_fill_ready_output(
                    request_id,
                    selected_skills_for_fill_command(active_skill_names, false),
                    context.target_description.clone(),
                    alternate_element_id.clone(),
                    text.clone(),
                )
            };
            let mut next_context = context.clone();
            next_context.active_element_id = Some(alternate_element_id);
            next_context.pending_text = Some(text);
            Some((planner_output, Some(next_context)))
        }
        FillFieldCorrectionCommand::ReplaceValue { text } => {
            let Some(context) = matching_context else {
                let planner_output = build_direct_follow_up_output(
                    request_id,
                    DirectFollowUpSpec {
                        intent_name: IntentName::FillInput,
                        goal: String::from("Fill the requested field."),
                        target_description: Some(String::from("field fill target")),
                        selected_skills: selected_skills_for_fill_command(active_skill_names, false),
                        summary: String::from("Please tell me which field to update."),
                        next_recommended_action: Some(String::from(
                            "Say the field name and value, like fill the city field with Seattle.",
                        )),
                        step_id: String::from("report-missing-recent-fill-target"),
                        purpose: String::from(
                            "Report that there is no recent field target available for replacement text.",
                        ),
                    },
                );
                return Some((planner_output, None));
            };

            let current_page = current_page?;
            let active_element_id = context
                .active_element_id
                .as_ref()
                .filter(|element_id| resolve_typeable_element(current_page, element_id).is_ok())
                .cloned();
            let Some(active_element_id) = active_element_id else {
                let planner_output = build_direct_follow_up_output(
                    request_id,
                    DirectFollowUpSpec {
                        intent_name: IntentName::FillInput,
                        goal: String::from("Fill the requested field."),
                        target_description: context.target_description.clone(),
                        selected_skills: selected_skills_for_fill_command(active_skill_names, false),
                        summary: String::from(
                            "Please tell me which field to update because the recent target is no longer available.",
                        ),
                        next_recommended_action: Some(String::from(
                            "Say the field label or placeholder together with the new value.",
                        )),
                        step_id: String::from("report-stale-recent-fill-target"),
                        purpose: String::from(
                            "Report that the stored recent field target cannot be reused on the current page.",
                        ),
                    },
                );
                return Some((planner_output, Some(context.clone())));
            };

            let planner_output = build_direct_fill_ready_output(
                request_id,
                selected_skills_for_fill_command(active_skill_names, false),
                context.target_description.clone(),
                active_element_id.clone(),
                text.clone(),
            );
            let mut next_context = context.clone();
            next_context.active_element_id = Some(active_element_id);
            next_context.pending_text = Some(text);
            next_context.submit_after = false;
            Some((planner_output, Some(next_context)))
        }
    }
}

fn resolve_direct_fill_command_internal(
    transcript: &str,
    request_id: &str,
    current_page_id: Option<&str>,
    current_page: Option<&PageModel>,
    active_skill_names: &[String],
    confirmation_confidence_threshold: f32,
    submit_after: bool,
) -> Option<ResolvedDirectFieldCommand> {
    let command = if submit_after {
        parse_direct_fill_and_submit_command(transcript)?
    } else {
        parse_direct_fill_field_command(transcript)?
    };
    let selected_skills = selected_skills_for_fill_command(active_skill_names, submit_after);
    let goal = if submit_after {
        "Fill the requested field and submit the form."
    } else {
        "Fill the requested field."
    };
    let intent_name = if submit_after {
        IntentName::SubmitForm
    } else {
        IntentName::FillInput
    };

    let Some(description) = command.description else {
        let planner_output = build_direct_follow_up_output(
            request_id,
            DirectFollowUpSpec {
                intent_name,
                goal: goal.to_string(),
                target_description: Some(String::from("field fill target")),
                selected_skills,
                summary: if submit_after {
                    String::from("Please tell me which field to fill before I submit.")
                } else {
                    String::from("Please tell me which field to fill.")
                },
                next_recommended_action: Some(if submit_after {
                    String::from(
                        "Say the field name and value, like fill the email field with phil@example.com and submit.",
                    )
                } else {
                    String::from(
                        "Say the field name and value, like fill the email field with phil@example.com.",
                    )
                }),
                step_id: if submit_after {
                    String::from("report-missing-fill-submit-field-description")
                } else {
                    String::from("report-missing-fill-field-description")
                },
                purpose: if submit_after {
                    String::from(
                        "Report that the field name is required before filling and submitting.",
                    )
                } else {
                    String::from("Report that the field name is required before filling.")
                },
            },
        );
        return Some(ResolvedDirectFieldCommand {
            planner_output,
            recent_field_context: None,
        });
    };

    let Some(text) = command.text else {
        let planner_output = build_direct_follow_up_output(
            request_id,
            DirectFollowUpSpec {
                intent_name,
                goal: goal.to_string(),
                target_description: Some(description),
                selected_skills,
                summary: if submit_after {
                    String::from("Please tell me what text to enter before I submit.")
                } else {
                    String::from("Please tell me what text to enter.")
                },
                next_recommended_action: Some(if submit_after {
                    String::from(
                        "Say the value after the field name, like fill the email field with phil@example.com and submit.",
                    )
                } else {
                    String::from(
                        "Say the value after the field name, like fill the email field with phil@example.com.",
                    )
                }),
                step_id: if submit_after {
                    String::from("report-missing-fill-submit-text")
                } else {
                    String::from("report-missing-fill-text")
                },
                purpose: if submit_after {
                    String::from(
                        "Report that the requested field value is required before filling and submitting.",
                    )
                } else {
                    String::from(
                        "Report that the requested field value is required before filling.",
                    )
                },
            },
        );
        return Some(ResolvedDirectFieldCommand {
            planner_output,
            recent_field_context: None,
        });
    };

    let Some(current_page) = current_page else {
        let planner_output = build_direct_follow_up_output(
            request_id,
            DirectFollowUpSpec {
                intent_name,
                goal: goal.to_string(),
                target_description: None,
                selected_skills,
                summary: if submit_after {
                    String::from("There is no current page to fill and submit a form on yet.")
                } else {
                    String::from("There is no current page to fill a field on yet.")
                },
                next_recommended_action: Some(if submit_after {
                    String::from(
                        "Open a page first, then ask me to fill a field and submit the form.",
                    )
                } else {
                    String::from("Open a page first, then ask me to fill a field.")
                }),
                step_id: if submit_after {
                    String::from("report-missing-fill-submit-page")
                } else {
                    String::from("report-missing-fill-page")
                },
                purpose: if submit_after {
                    String::from(
                        "Report that there is no active page available for filling and submitting.",
                    )
                } else {
                    String::from("Report that there is no active page available for field entry.")
                },
            },
        );
        return Some(ResolvedDirectFieldCommand {
            planner_output,
            recent_field_context: None,
        });
    };

    let field_elements = focusable_field_elements(current_page);
    if field_elements.is_empty() {
        let planner_output = build_direct_follow_up_output(
            request_id,
            DirectFollowUpSpec {
                intent_name,
                goal: goal.to_string(),
                target_description: Some(description.clone()),
                selected_skills,
                summary: String::from("I could not find any fillable fields on the current page."),
                next_recommended_action: Some(String::from(
                    "Try again after the page finishes loading or becomes interactive.",
                )),
                step_id: if submit_after {
                    String::from("report-missing-fill-submit-fields")
                } else {
                    String::from("report-missing-fillable-fields")
                },
                purpose: if submit_after {
                    String::from(
                        "Report that no editable fields are available for filling and submitting.",
                    )
                } else {
                    String::from(
                        "Report that no editable fields are available on the current page.",
                    )
                },
            },
        );
        return Some(ResolvedDirectFieldCommand {
            planner_output,
            recent_field_context: None,
        });
    }

    let query = FindElementInput {
        request_id: request_id.to_string(),
        timeout_ms: None,
        description: description.clone(),
        text: None,
        role: None,
        color_hint: None,
        nearby_text: None,
        selector_hint: None,
        visibility_filter: crate::commands::ElementVisibilityFilter::VisibleOnly,
        max_candidates: Some(DEFAULT_FIND_ELEMENT_MAX_CANDIDATES),
    };
    let search_query = build_find_element_query(&query).ok()?;
    let candidates = rank_find_element_candidates(
        &field_elements,
        &search_query,
        DEFAULT_FIND_ELEMENT_MAX_CANDIDATES,
    );
    let (chosen_element_id, _, requires_confirmation) = if candidates.len() == 1 {
        (Some(candidates[0].element_id.clone()), None, false)
    } else {
        determine_find_element_resolution(&candidates, confirmation_confidence_threshold)
    };
    let recent_field_context = current_page_id.map(|_| PendingRecentFieldContext {
        target_description: Some(description.clone()),
        active_element_id: chosen_element_id.clone(),
        candidate_element_ids: candidates
            .iter()
            .map(|candidate| candidate.element_id.clone())
            .collect(),
        pending_text: Some(text.clone()),
        submit_after,
    });

    if let Some(element_id) = chosen_element_id {
        let planner_output = if submit_after {
            build_direct_fill_and_submit_ready_output(
                request_id,
                selected_skills,
                Some(description),
                element_id,
                text,
            )
        } else {
            build_direct_fill_ready_output(
                request_id,
                selected_skills,
                Some(description),
                element_id,
                text,
            )
        };
        return Some(ResolvedDirectFieldCommand {
            planner_output,
            recent_field_context,
        });
    }

    let summary = if requires_confirmation {
        let candidate_names = summarize_candidate_names(current_page, &candidates);
        if candidate_names.is_empty() {
            if submit_after {
                format!(
                    "I found multiple possible fields for {description}. Please be more specific before I submit."
                )
            } else {
                format!(
                    "I found multiple possible fields for {description}. Please be more specific."
                )
            }
        } else if submit_after {
            format!(
                "I found multiple possible fields for {description}: {}. Please be more specific before I submit.",
                candidate_names.join(", ")
            )
        } else {
            format!(
                "I found multiple possible fields for {description}: {}. Please be more specific.",
                candidate_names.join(", ")
            )
        }
    } else {
        format!("I could not find a visible field matching {description}.")
    };

    let planner_output = build_direct_follow_up_output(
        request_id,
        DirectFollowUpSpec {
            intent_name,
            goal: goal.to_string(),
            target_description: Some(description),
            selected_skills,
            summary,
            next_recommended_action: Some(String::from(
                "Try naming the field label or placeholder more specifically.",
            )),
            step_id: if submit_after {
                String::from("report-fill-submit-follow-up")
            } else {
                String::from("report-fill-field-follow-up")
            },
            purpose: if submit_after {
                String::from(
                    "Report that the requested field could not be filled and submitted deterministically.",
                )
            } else {
                String::from(
                    "Report that the requested field could not be filled deterministically.",
                )
            },
        },
    );
    Some(ResolvedDirectFieldCommand {
        planner_output,
        recent_field_context,
    })
}

#[cfg(test)]
fn resolve_direct_fill_field_command(
    transcript: &str,
    request_id: &str,
    current_page: Option<&PageModel>,
    active_skill_names: &[String],
    confirmation_confidence_threshold: f32,
) -> Option<PlannerOutput> {
    resolve_direct_fill_command_internal(
        transcript,
        request_id,
        None,
        current_page,
        active_skill_names,
        confirmation_confidence_threshold,
        false,
    )
    .map(|resolved| resolved.planner_output)
}

#[cfg(test)]
fn resolve_direct_fill_and_submit_command(
    transcript: &str,
    request_id: &str,
    current_page: Option<&PageModel>,
    active_skill_names: &[String],
    confirmation_confidence_threshold: f32,
) -> Option<PlannerOutput> {
    resolve_direct_fill_command_internal(
        transcript,
        request_id,
        None,
        current_page,
        active_skill_names,
        confirmation_confidence_threshold,
        true,
    )
    .map(|resolved| resolved.planner_output)
}

fn resolve_direct_submit_form_command(
    transcript: &str,
    request_id: &str,
    current_page: Option<&PageModel>,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    if !is_direct_submit_form_command(transcript) {
        return None;
    }

    let selected_skills = if active_skill_names
        .iter()
        .any(|active_name| active_name == "submit_form")
    {
        vec![String::from("submit_form")]
    } else {
        Vec::new()
    };

    let Some(current_page) = current_page else {
        return Some(build_direct_follow_up_output(
            request_id,
            DirectFollowUpSpec {
                intent_name: IntentName::SubmitForm,
                goal: String::from("Submit the active form."),
                target_description: Some(String::from("current form")),
                selected_skills,
                summary: String::from("There is no current page to submit a form on yet."),
                next_recommended_action: Some(String::from(
                    "Open a page first, then ask me to submit the form.",
                )),
                step_id: String::from("report-missing-submit-page"),
                purpose: String::from(
                    "Report that there is no active page available for form submission.",
                ),
            },
        ));
    };

    let candidate_forms = submittable_form_elements(current_page);
    let resolved_form = if current_page.interactive_elements.is_empty() {
        None
    } else if candidate_forms.len() == 1 {
        Some(candidate_forms[0].clone())
    } else if candidate_forms.is_empty() {
        return Some(build_direct_follow_up_output(
            request_id,
            DirectFollowUpSpec {
                intent_name: IntentName::SubmitForm,
                goal: String::from("Submit the active form."),
                target_description: Some(String::from("current form")),
                selected_skills,
                summary: String::from(
                    "I could not identify a submittable form on the current page.",
                ),
                next_recommended_action: Some(String::from(
                    "Focus a field in the form or describe which form you want to submit.",
                )),
                step_id: String::from("report-missing-submit-form"),
                purpose: String::from(
                    "Report that no submittable form could be identified on the current page.",
                ),
            },
        ));
    } else {
        let candidate_names = summarize_form_candidate_names(&candidate_forms);
        let summary = if candidate_names.is_empty() {
            String::from(
                "I found multiple forms on the current page. Please tell me which one to submit.",
            )
        } else {
            format!(
                "I found multiple forms on the current page: {}. Please tell me which one to submit.",
                candidate_names.join(", ")
            )
        };
        return Some(build_direct_follow_up_output(
            request_id,
            DirectFollowUpSpec {
                intent_name: IntentName::SubmitForm,
                goal: String::from("Submit the active form."),
                target_description: Some(String::from("current form")),
                selected_skills,
                summary,
                next_recommended_action: Some(String::from(
                    "Name the form or focus a field in it before asking me to submit.",
                )),
                step_id: String::from("report-ambiguous-submit-form"),
                purpose: String::from(
                    "Report that multiple possible forms are available and submission is ambiguous.",
                ),
            },
        ));
    };

    let target_description = resolved_form.as_ref().map(describe_form_element);
    let prompt_text = match target_description.as_deref() {
        Some(description) => format!("Do you want me to submit {description} now?"),
        None => String::from("Do you want me to submit the active form now?"),
    };
    let confirmation_reason = String::from("submitting the form may send data");
    let user_message = String::from("Please confirm before I submit the form.");

    Some(PlannerOutput {
        status: PlannerStatus::NeedsConfirmation,
        intent: IntentSummary {
            name: IntentName::SubmitForm,
            goal: String::from("Submit the active form."),
            target_description,
        },
        selected_skills,
        steps: vec![
            PlannedStep {
                step_id: String::from("confirm-submit-form"),
                tool_name: ToolName::ConfirmAction,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "prompt_text": prompt_text,
                    "reason": confirmation_reason
                }),
                purpose: String::from("Require explicit confirmation before submitting the form."),
                on_success: StepTransition::RequestConfirmation,
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from("submit-active-form"),
                tool_name: ToolName::SubmitActiveForm,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "form_element_id": resolved_form.as_ref().map(|form| form.element_id.clone())
                }),
                purpose: String::from("Submit the confirmed active form in the live browser."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ],
        requires_confirmation: true,
        confirmation_reason: Some(String::from("submitting the form may send data")),
        blocked_reason: None,
        user_message: Some(user_message),
    })
}

fn build_visible_text_excerpt(page: &PageModel, max_chars: Option<usize>) -> String {
    let joined_text = page
        .regions
        .iter()
        .map(|region| region.text.trim())
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");

    match max_chars {
        Some(limit) => joined_text.chars().take(limit).collect(),
        None => joined_text,
    }
}

fn build_extracted_page_model(page: &PageModel, input: &ExtractPageModelInput) -> PageModel {
    let interactive_elements = if input.include_links {
        page.interactive_elements.clone()
    } else {
        page.interactive_elements
            .iter()
            .filter(|element| element.role != ElementRole::Link)
            .cloned()
            .collect()
    };

    PageModel {
        title: page.title.clone(),
        url: page.url.clone(),
        regions: page.regions.clone(),
        interactive_elements,
    }
}

struct DirectFollowUpSpec {
    intent_name: IntentName,
    goal: String,
    target_description: Option<String>,
    selected_skills: Vec<String>,
    summary: String,
    next_recommended_action: Option<String>,
    step_id: String,
    purpose: String,
}

fn build_direct_follow_up_output(request_id: &str, spec: DirectFollowUpSpec) -> PlannerOutput {
    PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: spec.intent_name,
            goal: spec.goal,
            target_description: spec.target_description,
        },
        selected_skills: spec.selected_skills,
        steps: vec![PlannedStep {
            step_id: spec.step_id,
            tool_name: ToolName::ReportResult,
            arguments: serde_json::json!({
                "request_id": request_id,
                "timeout_ms": serde_json::Value::Null,
                "status": ReportStatus::NeedsFollowUp,
                "summary": spec.summary.clone(),
                "next_recommended_action": spec.next_recommended_action,
                "user_message": spec.summary
            }),
            purpose: spec.purpose,
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    }
}

fn focusable_field_elements(page: &PageModel) -> Vec<crate::page_model::InteractiveElement> {
    filter_interactive_elements(
        &page.interactive_elements,
        true,
        Some(&[
            ElementRole::Input,
            ElementRole::TextArea,
            ElementRole::Select,
        ]),
    )
    .into_iter()
    .filter(|element| {
        element.enabled
            && element
                .dom_locator
                .as_deref()
                .map(str::trim)
                .is_some_and(|locator| !locator.is_empty())
    })
    .collect()
}

fn submittable_form_elements(page: &PageModel) -> Vec<crate::page_model::InteractiveElement> {
    filter_interactive_elements(&page.interactive_elements, true, Some(&[ElementRole::Form]))
        .into_iter()
        .filter(|element| {
            element
                .dom_locator
                .as_deref()
                .map(str::trim)
                .is_some_and(|locator| !locator.is_empty())
        })
        .collect()
}

fn summarize_candidate_names(
    page: &PageModel,
    candidates: &[crate::commands::ElementCandidate],
) -> Vec<String> {
    candidates
        .iter()
        .filter_map(|candidate| {
            page.interactive_elements
                .iter()
                .find(|element| element.element_id == candidate.element_id)
                .map(describe_field_element)
        })
        .take(MAX_DIRECT_FIELD_CANDIDATE_NAMES)
        .collect()
}

fn summarize_form_candidate_names(forms: &[crate::page_model::InteractiveElement]) -> Vec<String> {
    forms
        .iter()
        .map(describe_form_element)
        .take(MAX_DIRECT_FIELD_CANDIDATE_NAMES)
        .collect()
}

fn describe_field_element(element: &crate::page_model::InteractiveElement) -> String {
    element
        .accessible_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            element
                .placeholder
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .or_else(|| {
            element
                .text
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .map(String::from)
        .unwrap_or_else(|| element.element_id.clone())
}

fn describe_form_element(element: &crate::page_model::InteractiveElement) -> String {
    element
        .accessible_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            element
                .text
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .or_else(|| {
            element
                .attributes
                .get("id")
                .map(String::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .map(|description| format!("the {description} form"))
        .unwrap_or_else(|| String::from("the current form"))
}

fn filter_interactive_elements(
    interactive_elements: &[crate::page_model::InteractiveElement],
    visible_only: bool,
    roles: Option<&[ElementRole]>,
) -> Vec<crate::page_model::InteractiveElement> {
    interactive_elements
        .iter()
        .filter(|element| !visible_only || element.visible)
        .filter(|element| roles.is_none_or(|roles| roles.contains(&element.role)))
        .cloned()
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FindElementQuery {
    summary: String,
    description: Option<String>,
    text: Option<String>,
    role: Option<ElementRole>,
    color_hint: Option<String>,
    nearby_text: Option<String>,
    selector_hint: Option<String>,
}

fn build_find_element_query(input: &FindElementInput) -> Result<FindElementQuery, ToolError> {
    let description = normalize_optional_text(Some(input.description.clone()));
    let text = normalize_optional_text(input.text.clone());
    let color_hint = normalize_optional_text(input.color_hint.clone());
    let nearby_text = normalize_optional_text(input.nearby_text.clone());
    let selector_hint = normalize_optional_text(input.selector_hint.clone());

    if description.is_none()
        && text.is_none()
        && input.role.is_none()
        && color_hint.is_none()
        && nearby_text.is_none()
        && selector_hint.is_none()
    {
        return Err(ToolError {
            code: String::from("invalid_find_query"),
            message: String::from("find_element requires at least one populated search field"),
            retryable: false,
            details: None,
        });
    }

    let mut summary_parts = Vec::new();
    if let Some(description) = description.as_ref() {
        summary_parts.push(format!("description={description}"));
    }
    if let Some(text) = text.as_ref() {
        summary_parts.push(format!("text={text}"));
    }
    if let Some(role) = input.role.as_ref() {
        summary_parts.push(format!("role={role:?}"));
    }
    if let Some(color_hint) = color_hint.as_ref() {
        summary_parts.push(format!("color_hint={color_hint}"));
    }
    if let Some(nearby_text) = nearby_text.as_ref() {
        summary_parts.push(format!("nearby_text={nearby_text}"));
    }
    if let Some(selector_hint) = selector_hint.as_ref() {
        summary_parts.push(format!("selector_hint={selector_hint}"));
    }

    Ok(FindElementQuery {
        summary: summary_parts.join("; "),
        description,
        text,
        role: input.role.clone(),
        color_hint,
        nearby_text,
        selector_hint,
    })
}

fn rank_find_element_candidates(
    elements: &[crate::page_model::InteractiveElement],
    query: &FindElementQuery,
    candidate_limit: usize,
) -> Vec<crate::commands::ElementCandidate> {
    let mut candidates = elements
        .iter()
        .filter_map(|element| score_interactive_element(element, query))
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| {
        right
            .confidence_bps
            .cmp(&left.confidence_bps)
            .then_with(|| left.element_id.cmp(&right.element_id))
    });
    candidates.truncate(candidate_limit);
    candidates
}

fn determine_find_element_resolution(
    candidates: &[crate::commands::ElementCandidate],
    confirmation_confidence_threshold: f32,
) -> (Option<String>, Option<f32>, bool) {
    let Some(top_candidate) = candidates.first() else {
        return (None, None, false);
    };

    let top_confidence = Some(f32::from(top_candidate.confidence_bps) / 10_000.0);
    let required_confidence_bps =
        (confirmation_confidence_threshold.clamp(0.0, 1.0) * 10_000.0).round() as u16;
    let below_threshold = top_candidate.confidence_bps < required_confidence_bps;
    let ambiguous_with_runner_up = candidates.get(1).is_some_and(|second_candidate| {
        top_candidate
            .confidence_bps
            .saturating_sub(second_candidate.confidence_bps)
            <= FIND_ELEMENT_AMBIGUITY_MARGIN_BPS
    });

    if below_threshold || ambiguous_with_runner_up {
        (None, top_confidence, true)
    } else {
        (
            Some(top_candidate.element_id.clone()),
            top_confidence,
            false,
        )
    }
}

fn resolve_clickable_element<'a>(
    page: &'a PageModel,
    element_id: &str,
) -> Result<&'a crate::page_model::InteractiveElement, ToolError> {
    let normalized_element_id = element_id.trim();
    if normalized_element_id.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_element_id"),
            message: String::from("click_element requires a non-empty deterministic element_id"),
            retryable: false,
            details: None,
        });
    }

    let Some(element) = page
        .interactive_elements
        .iter()
        .find(|element| element.element_id == normalized_element_id)
    else {
        return Err(ToolError {
            code: String::from("unknown_element_id"),
            message: String::from(
                "click_element requires an element_id that exists in the current page model",
            ),
            retryable: false,
            details: Some(serde_json::json!({ "element_id": normalized_element_id })),
        });
    };

    if !element.visible {
        return Err(ToolError {
            code: String::from("element_not_visible"),
            message: String::from("click_element cannot act on an element marked not visible"),
            retryable: false,
            details: Some(serde_json::json!({ "element_id": normalized_element_id })),
        });
    }

    if !element.enabled {
        return Err(ToolError {
            code: String::from("element_disabled"),
            message: String::from("click_element cannot act on a disabled element"),
            retryable: false,
            details: Some(serde_json::json!({ "element_id": normalized_element_id })),
        });
    }

    if element
        .dom_locator
        .as_deref()
        .map(str::trim)
        .is_none_or(str::is_empty)
    {
        return Err(ToolError {
            code: String::from("missing_dom_locator"),
            message: String::from(
                "click_element requires the current page model to carry a stable dom_locator",
            ),
            retryable: false,
            details: Some(serde_json::json!({ "element_id": normalized_element_id })),
        });
    }

    Ok(element)
}

fn resolve_typeable_element<'a>(
    page: &'a PageModel,
    element_id: &str,
) -> Result<&'a crate::page_model::InteractiveElement, ToolError> {
    let element = resolve_clickable_element(page, element_id)?;
    if !matches!(
        element.role,
        ElementRole::Input | ElementRole::TextArea | ElementRole::Select
    ) {
        return Err(ToolError {
            code: String::from("element_not_editable"),
            message: String::from(
                "type_into_element requires an input, textarea, or select element",
            ),
            retryable: false,
            details: Some(serde_json::json!({ "element_id": element.element_id })),
        });
    }

    Ok(element)
}

fn resolve_form_element<'a>(
    page: &'a PageModel,
    element_id: &str,
) -> Result<&'a crate::page_model::InteractiveElement, ToolError> {
    let element = resolve_clickable_element(page, element_id)?;
    if element.role != ElementRole::Form {
        return Err(ToolError {
            code: String::from("element_not_form"),
            message: String::from(
                "submit_active_form requires a form element from the current page model",
            ),
            retryable: false,
            details: Some(serde_json::json!({ "element_id": element.element_id })),
        });
    }

    Ok(element)
}

#[derive(Debug, Default)]
struct FindElementScore {
    score_bps: u16,
    matched_on: Vec<String>,
    rationale_codes: Vec<String>,
}

impl FindElementScore {
    fn push_match(&mut self, match_label: &str, rationale_code: impl Into<String>, score_bps: u16) {
        self.score_bps = self.score_bps.saturating_add(score_bps);
        self.matched_on.push(match_label.to_string());
        self.rationale_codes.push(rationale_code.into());
    }
}

struct AttributeHintSpec<'a> {
    match_label: &'a str,
    exact_score_bps: u16,
    contains_score_bps: u16,
}

fn score_interactive_element(
    element: &crate::page_model::InteractiveElement,
    query: &FindElementQuery,
) -> Option<crate::commands::ElementCandidate> {
    let mut score = FindElementScore::default();

    if let Some(role) = query.role.as_ref() {
        if &element.role == role {
            score.push_match("role", "role_match", 1_800);
        } else {
            return None;
        }
    }

    if let Some(description) = query.description.as_ref() {
        let field_match =
            score_text_query_against_element(description, element, "description", &mut score);
        if !field_match && query.role.is_none() {
            return None;
        }
    }

    if let Some(text) = query.text.as_ref() {
        let field_match = score_text_query_against_element(text, element, "text", &mut score);
        if !field_match && query.description.is_none() && query.role.is_none() {
            return None;
        }
    }

    if let Some(nearby_text) = query.nearby_text.as_ref() {
        score_attribute_hint(
            nearby_text,
            element,
            AttributeHintSpec {
                match_label: "nearby_text",
                exact_score_bps: 1_600,
                contains_score_bps: 900,
            },
            &mut score,
        );
    }

    if let Some(selector_hint) = query.selector_hint.as_ref() {
        score_attribute_hint(
            selector_hint,
            element,
            AttributeHintSpec {
                match_label: "selector_hint",
                exact_score_bps: 1_500,
                contains_score_bps: 800,
            },
            &mut score,
        );
    }

    if let Some(color_hint) = query.color_hint.as_ref() {
        score_attribute_hint(
            color_hint,
            element,
            AttributeHintSpec {
                match_label: "color_hint",
                exact_score_bps: 500,
                contains_score_bps: 250,
            },
            &mut score,
        );
    }

    if score.score_bps == 0 {
        return None;
    }

    if element.enabled {
        score.score_bps = score.score_bps.saturating_add(100);
    } else {
        score.rationale_codes.push(String::from("disabled_penalty"));
        score.score_bps = score.score_bps.saturating_sub(300);
    }

    Some(crate::commands::ElementCandidate {
        element_id: element.element_id.clone(),
        confidence_bps: score.score_bps.min(10_000),
        matched_on: score.matched_on,
        rationale_codes: score.rationale_codes,
    })
}

fn score_text_query_against_element(
    query_text: &str,
    element: &crate::page_model::InteractiveElement,
    match_label: &str,
    score: &mut FindElementScore,
) -> bool {
    let normalized_query = normalize_search_text(query_text);
    let accessible_name = element
        .accessible_name
        .as_deref()
        .map(normalize_search_text);
    let visible_text = element.text.as_deref().map(normalize_search_text);
    let placeholder = element.placeholder.as_deref().map(normalize_search_text);

    if accessible_name.as_deref() == Some(normalized_query.as_str()) {
        score.push_match(match_label, "accessible_name_exact", 4_200);
        return true;
    }
    if visible_text.as_deref() == Some(normalized_query.as_str()) {
        score.push_match(match_label, "visible_text_exact", 4_000);
        return true;
    }
    if placeholder.as_deref() == Some(normalized_query.as_str()) {
        score.push_match(match_label, "placeholder_exact", 3_400);
        return true;
    }

    let overlap_score = text_overlap_score(&normalized_query, element);
    if overlap_score > 0 {
        score.push_match(match_label, "lexical_overlap", overlap_score);
        return true;
    }

    false
}

fn score_attribute_hint(
    hint: &str,
    element: &crate::page_model::InteractiveElement,
    spec: AttributeHintSpec<'_>,
    score: &mut FindElementScore,
) -> bool {
    let normalized_hint = normalize_search_text(hint);
    let attribute_blob = element
        .attributes
        .iter()
        .map(|(key, value)| format!("{key} {value}"))
        .collect::<Vec<_>>()
        .join(" ");
    let normalized_attributes = normalize_search_text(&attribute_blob);

    if normalized_attributes.is_empty() {
        return false;
    }

    if normalized_attributes == normalized_hint {
        score.push_match(
            spec.match_label,
            format!("{}_exact", spec.match_label),
            spec.exact_score_bps,
        );
        true
    } else if normalized_attributes.contains(&normalized_hint) {
        score.push_match(
            spec.match_label,
            format!("{}_contains", spec.match_label),
            spec.contains_score_bps,
        );
        true
    } else {
        false
    }
}

fn text_overlap_score(query_text: &str, element: &crate::page_model::InteractiveElement) -> u16 {
    let query_terms = tokenize_search_text(query_text);
    if query_terms.is_empty() {
        return 0;
    }

    let element_blob = [
        element.accessible_name.as_deref().unwrap_or_default(),
        element.text.as_deref().unwrap_or_default(),
        element.placeholder.as_deref().unwrap_or_default(),
        element.href.as_deref().unwrap_or_default(),
        element.value.as_deref().unwrap_or_default(),
    ]
    .join(" ");
    let element_terms = tokenize_search_text(&element_blob);
    if element_terms.is_empty() {
        return 0;
    }

    let overlap = query_terms
        .iter()
        .filter(|term| element_terms.contains(*term))
        .count();
    if overlap == 0 {
        0
    } else {
        let ratio = overlap as f32 / query_terms.len() as f32;
        (900.0 + (ratio * 2_100.0)).round() as u16
    }
}

fn normalize_search_text(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn tokenize_search_text(value: &str) -> Vec<String> {
    normalize_search_text(value)
        .split(' ')
        .filter(|term| !term.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn infer_extraction_source(
    page: &PageModel,
    use_dom_extraction: bool,
    used_dom_smoothie: bool,
) -> ExtractionSource {
    let has_ocr = page
        .regions
        .iter()
        .any(|region| matches!(region.source, RegionSource::Ocr | RegionSource::Mixed));
    let has_dom_like = page
        .regions
        .iter()
        .any(|region| matches!(region.source, RegionSource::Dom | RegionSource::Mixed));

    if has_ocr && has_dom_like {
        ExtractionSource::Merged
    } else if has_ocr {
        ExtractionSource::Ocr
    } else if use_dom_extraction && used_dom_smoothie {
        ExtractionSource::DomSmoothie
    } else {
        ExtractionSource::DomFallback
    }
}

fn browser_error_to_tool_error(message: String, error: BrowserError) -> ToolError {
    let code = match &error {
        BrowserError::FeatureDisabled => "browser_feature_disabled",
        BrowserError::Launch(_) => "browser_launch_failed",
        BrowserError::CreatePage(_) => "browser_page_creation_failed",
        BrowserError::Navigate(_) => "browser_navigation_failed",
        BrowserError::Inspect(_) => "browser_state_read_failed",
        BrowserError::NoActivePage => "browser_no_active_page",
        BrowserError::MissingDomLocator { .. } => "missing_dom_locator",
        BrowserError::Resolve(_) => "browser_element_resolution_failed",
        BrowserError::ElementNotFound { .. } => "browser_element_not_found",
        BrowserError::Click(_) => "browser_click_failed",
        BrowserError::Focus(_) => "browser_focus_failed",
        BrowserError::Type(_) => "browser_type_failed",
        BrowserError::Submit(_) => "browser_submit_failed",
        BrowserError::History(_) => "browser_history_failed",
        BrowserError::Reload(_) => "browser_reload_failed",
        BrowserError::Eval(_) => "browser_eval_failed",
        BrowserError::Scroll(_) => "browser_scroll_failed",
        BrowserError::Screenshot(_) => "browser_screenshot_failed",
    };

    ToolError {
        code: String::from(code),
        message,
        retryable: matches!(
            error,
            BrowserError::Launch(_)
                | BrowserError::CreatePage(_)
                | BrowserError::Navigate(_)
                | BrowserError::Inspect(_)
                | BrowserError::History(_)
                | BrowserError::Reload(_)
                | BrowserError::Eval(_)
                | BrowserError::Scroll(_)
                | BrowserError::Screenshot(_)
        ),
        details: Some(serde_json::json!({ "reason": error.to_string() })),
    }
}

fn planner_interpretation_unavailable_error(
    code: &str,
    reason: impl Into<String>,
    retryable: bool,
    details: Option<serde_json::Value>,
) -> ToolError {
    let reason = reason.into();
    let reason = reason.trim().trim_end_matches('.').to_string();

    ToolError {
        code: String::from(code),
        message: format!("Command interpretation is unavailable because {reason}."),
        retryable,
        details,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        browser_error_to_tool_error, build_asr_provider_settings, build_confirmation_settings,
        build_extracted_page_model, build_find_element_query, build_local_asr_model_settings,
        build_local_tts_model_settings, build_ocr_threshold_settings,
        build_planner_provider_settings, build_provider_failover_settings,
        build_remote_asr_settings, build_remote_planner_settings, build_remote_tts_settings,
        build_tts_model_settings, build_tts_provider_settings, build_tts_voice_settings,
        build_visible_text_excerpt, clear_navigation_follow_up_state,
        determine_find_element_resolution, execute_bounded_replanning_loop, extracted_text_metrics,
        filter_interactive_elements, infer_extraction_source, merge_ocr_text_into_page_model,
        merged_region_text, normalize_absolute_url, normalize_optional_text,
        planner_interpretation_unavailable_error, planner_system_prompt,
        rank_find_element_candidates, refresh_current_page_after_navigation, region_bbox_by_id,
        region_first_ocr_target_ids, resolve_clickable_element,
        resolve_direct_fill_and_submit_command, resolve_direct_fill_field_command,
        resolve_direct_focus_field_command, resolve_direct_submit_form_command,
        resolve_form_element, resolve_recent_fill_correction_command, resolve_typeable_element,
        should_trigger_extract_page_model_ocr_fallback, RecentFieldContext, ReplanningRuntime,
    };
    use crate::audio_io::RuntimeAudioState;
    use crate::browser::BrowserError;
    use crate::commands::{
        ExecutionOutcome, ExecutionTrace, ExtractPageModelInput, FindElementInput, IntentName,
        IntentSummary, PlannedStep, PlannerOutput, PlannerStatus, PlannerToolHistoryEntry,
        ReportStatus, StepTransition, ToolName, ToolResult,
    };
    use crate::config::{AppConfig, KeyringRef, ProviderMode, SecretRef};
    use crate::ocr::OcrSettings;
    use crate::page_model::{
        ElementRole, ExtractionSource, InteractiveElement, PageModel, PageRegion, Rect, RegionRole,
        RegionSource,
    };
    use crate::state::AppState;

    fn fixture_page(interactive_elements: Vec<InteractiveElement>) -> PageModel {
        PageModel {
            title: Some(String::from("Example form")),
            url: Some(String::from("https://example.com/form")),
            regions: Vec::new(),
            interactive_elements,
        }
    }

    fn fixture_page_with_metadata(
        title: &str,
        url: &str,
        interactive_elements: Vec<InteractiveElement>,
    ) -> PageModel {
        PageModel {
            title: Some(String::from(title)),
            url: Some(String::from(url)),
            regions: Vec::new(),
            interactive_elements,
        }
    }

    fn fixture_field(
        element_id: &str,
        dom_locator: &str,
        accessible_name: &str,
        placeholder: &str,
    ) -> InteractiveElement {
        InteractiveElement {
            element_id: String::from(element_id),
            dom_locator: Some(String::from(dom_locator)),
            role: ElementRole::Input,
            tag_name: String::from("input"),
            text: None,
            accessible_name: Some(String::from(accessible_name)),
            placeholder: Some(String::from(placeholder)),
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::new(),
        }
    }

    fn fixture_form(
        element_id: &str,
        dom_locator: &str,
        accessible_name: &str,
    ) -> InteractiveElement {
        InteractiveElement {
            element_id: String::from(element_id),
            dom_locator: Some(String::from(dom_locator)),
            role: ElementRole::Form,
            tag_name: String::from("form"),
            text: Some(String::from(accessible_name)),
            accessible_name: Some(String::from(accessible_name)),
            placeholder: None,
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::new(),
        }
    }

    fn fixture_problematic_checkout_page() -> PageModel {
        fixture_page_with_metadata(
            "Example Shop | Checkout",
            "https://shop.example.com/checkout",
            vec![
                fixture_form("form-shipping", "#shipping-form", "Shipping address"),
                fixture_field(
                    "input-shipping-email",
                    "#shipping-email",
                    "Shipping email",
                    "Email for shipping updates",
                ),
                fixture_field(
                    "input-shipping-name",
                    "#shipping-name",
                    "Full name",
                    "Full name",
                ),
                fixture_form("form-billing", "#billing-form", "Billing address"),
                fixture_field(
                    "input-billing-email",
                    "#billing-email",
                    "Billing email",
                    "Billing email for receipts",
                ),
                fixture_field(
                    "input-card-name",
                    "#card-name",
                    "Name on card",
                    "Name on card",
                ),
            ],
        )
    }

    fn fixture_problematic_landing_page() -> PageModel {
        fixture_page_with_metadata(
            "Example Cloud | Start free trial",
            "https://www.example.com/start",
            vec![
                InteractiveElement {
                    element_id: String::from("button-hero-get-started"),
                    dom_locator: Some(String::from("#hero-get-started")),
                    role: ElementRole::Button,
                    tag_name: String::from("button"),
                    text: Some(String::from("Get started")),
                    accessible_name: Some(String::from("Get started")),
                    placeholder: None,
                    href: None,
                    value: None,
                    bbox: None,
                    visible: true,
                    enabled: true,
                    attributes: std::collections::BTreeMap::new(),
                },
                InteractiveElement {
                    element_id: String::from("button-footer-get-started"),
                    dom_locator: Some(String::from("#footer-get-started")),
                    role: ElementRole::Button,
                    tag_name: String::from("button"),
                    text: Some(String::from("Get started")),
                    accessible_name: Some(String::from("Get started")),
                    placeholder: None,
                    href: None,
                    value: None,
                    bbox: None,
                    visible: true,
                    enabled: true,
                    attributes: std::collections::BTreeMap::new(),
                },
            ],
        )
    }

    fn fixture_problematic_newsletter_page() -> PageModel {
        fixture_page_with_metadata(
            "Metro news | Sign up for morning headlines",
            "https://news.example.com/newsletters/morning-headlines",
            vec![fixture_field(
                "input-newsletter-email",
                "#newsletter-email",
                "Email",
                "Email address",
            )],
        )
    }

    fn planner_tool_sequence(planner_output: &PlannerOutput) -> Vec<ToolName> {
        planner_output
            .steps
            .iter()
            .map(|step| step.tool_name.clone())
            .collect()
    }

    #[derive(Clone, Copy)]
    enum AppCorePlannerFixtureKind {
        FocusField,
        FillField,
        FillAndSubmit,
        FollowUpCorrection,
        SubmitForm,
    }

    struct AppCorePlannerFixture {
        name: &'static str,
        kind: AppCorePlannerFixtureKind,
        transcript: &'static str,
        current_page_id: Option<&'static str>,
        page: Option<PageModel>,
        active_skills: Vec<&'static str>,
        recent_context: Option<RecentFieldContext>,
        confirmation_threshold: f32,
        expected_intent: IntentName,
        expected_status: PlannerStatus,
        expected_selected_skills: Vec<&'static str>,
        expected_tool_sequence: Vec<ToolName>,
        expected_focus_element_id: Option<&'static str>,
        expected_typed_text: Option<&'static str>,
        expected_next_active_element_id: Option<&'static str>,
        expected_next_pending_text: Option<&'static str>,
    }

    fn resolve_app_core_planner_fixture(
        fixture: &AppCorePlannerFixture,
    ) -> (PlannerOutput, Option<RecentFieldContext>) {
        match fixture.kind {
            AppCorePlannerFixtureKind::FocusField => (
                resolve_direct_focus_field_command(
                    fixture.transcript,
                    fixture.name,
                    fixture.page.as_ref(),
                    &fixture
                        .active_skills
                        .iter()
                        .map(|skill| String::from(*skill))
                        .collect::<Vec<_>>(),
                    fixture.confirmation_threshold,
                )
                .unwrap_or_else(|| panic!("fixture {} should resolve", fixture.name)),
                None,
            ),
            AppCorePlannerFixtureKind::FillField => (
                resolve_direct_fill_field_command(
                    fixture.transcript,
                    fixture.name,
                    fixture.page.as_ref(),
                    &fixture
                        .active_skills
                        .iter()
                        .map(|skill| String::from(*skill))
                        .collect::<Vec<_>>(),
                    fixture.confirmation_threshold,
                )
                .unwrap_or_else(|| panic!("fixture {} should resolve", fixture.name)),
                None,
            ),
            AppCorePlannerFixtureKind::FillAndSubmit => (
                resolve_direct_fill_and_submit_command(
                    fixture.transcript,
                    fixture.name,
                    fixture.page.as_ref(),
                    &fixture
                        .active_skills
                        .iter()
                        .map(|skill| String::from(*skill))
                        .collect::<Vec<_>>(),
                    fixture.confirmation_threshold,
                )
                .unwrap_or_else(|| panic!("fixture {} should resolve", fixture.name)),
                None,
            ),
            AppCorePlannerFixtureKind::FollowUpCorrection => {
                resolve_recent_fill_correction_command(
                    fixture.transcript,
                    fixture.name,
                    fixture.current_page_id,
                    fixture.page.as_ref(),
                    &fixture
                        .active_skills
                        .iter()
                        .map(|skill| String::from(*skill))
                        .collect::<Vec<_>>(),
                    fixture.recent_context.as_ref(),
                )
                .unwrap_or_else(|| panic!("fixture {} should resolve", fixture.name))
            }
            AppCorePlannerFixtureKind::SubmitForm => (
                resolve_direct_submit_form_command(
                    fixture.transcript,
                    fixture.name,
                    fixture.page.as_ref(),
                    &fixture
                        .active_skills
                        .iter()
                        .map(|skill| String::from(*skill))
                        .collect::<Vec<_>>(),
                )
                .unwrap_or_else(|| panic!("fixture {} should resolve", fixture.name)),
                None,
            ),
        }
    }

    fn assert_app_core_planner_fixture(fixture: AppCorePlannerFixture) {
        let (planner_output, next_context) = resolve_app_core_planner_fixture(&fixture);
        let expected_selected_skills = fixture
            .expected_selected_skills
            .iter()
            .map(|skill| String::from(*skill))
            .collect::<Vec<_>>();

        assert_eq!(
            planner_output.intent.name, fixture.expected_intent,
            "fixture {} resolved unexpected intent",
            fixture.name
        );
        assert_eq!(
            planner_output.status, fixture.expected_status,
            "fixture {} resolved unexpected planner status",
            fixture.name
        );
        assert_eq!(
            planner_output.selected_skills, expected_selected_skills,
            "fixture {} selected unexpected skills",
            fixture.name
        );
        assert_eq!(
            planner_tool_sequence(&planner_output),
            fixture.expected_tool_sequence,
            "fixture {} produced unexpected tool sequence",
            fixture.name
        );

        if let Some(expected_focus_element_id) = fixture.expected_focus_element_id {
            let focus_step = planner_output
                .steps
                .iter()
                .find(|step| step.tool_name == ToolName::FocusElement)
                .unwrap_or_else(|| panic!("fixture {} should include a focus step", fixture.name));
            assert_eq!(
                focus_step.arguments.get("element_id"),
                Some(&serde_json::json!(expected_focus_element_id)),
                "fixture {} focused the wrong element",
                fixture.name
            );
        }

        if let Some(expected_typed_text) = fixture.expected_typed_text {
            let type_step = planner_output
                .steps
                .iter()
                .find(|step| step.tool_name == ToolName::TypeIntoElement)
                .unwrap_or_else(|| panic!("fixture {} should include a type step", fixture.name));
            assert_eq!(
                type_step.arguments.get("text"),
                Some(&serde_json::json!(expected_typed_text)),
                "fixture {} typed unexpected text",
                fixture.name
            );
        }

        assert_eq!(
            next_context
                .as_ref()
                .and_then(|context| context.active_element_id.as_deref()),
            fixture.expected_next_active_element_id,
            "fixture {} produced unexpected next active element",
            fixture.name
        );
        assert_eq!(
            next_context
                .as_ref()
                .and_then(|context| context.pending_text.as_deref()),
            fixture.expected_next_pending_text,
            "fixture {} produced unexpected next pending text",
            fixture.name
        );
    }
    #[test]
    fn build_planner_provider_settings_reports_remote_only_mode() {
        let config = AppConfig::default();

        let settings = build_planner_provider_settings(&config);

        assert_eq!(settings.active_mode, ProviderMode::Remote);
        assert_eq!(settings.available_modes, vec![ProviderMode::Remote]);
        assert_eq!(
            settings.summary,
            String::from("Planner currently uses configured remote profiles only.")
        );
    }

    #[test]
    fn planner_interpretation_unavailable_error_wraps_reason_for_voice_feedback() {
        let error = planner_interpretation_unavailable_error(
            "planner_profile_unavailable",
            "remote planner mode requires a configured planner profile",
            false,
            None,
        );

        assert_eq!(error.code, "planner_profile_unavailable");
        assert_eq!(
            error.message,
            "Command interpretation is unavailable because remote planner mode requires a configured planner profile."
        );
        assert!(!error.retryable);
        assert_eq!(error.details, None);
    }

    #[test]
    fn build_remote_planner_settings_reflects_configured_profile_details() {
        let config = AppConfig::default();

        let settings = build_remote_planner_settings(&config);

        assert_eq!(settings.profile_name.as_deref(), Some("openai-default"));
        assert_eq!(
            settings.provider,
            Some(crate::commands::RemoteProviderLabel::OpenAi)
        );
        assert_eq!(
            settings.base_url.as_deref(),
            Some("https://api.openai.com/v1")
        );
        assert_eq!(settings.model.as_deref(), Some("gpt-5.4-mini"));
        assert_eq!(
            settings.api_key_reference.as_deref(),
            Some("Environment variable: OPENAI_API_KEY")
        );
        assert_eq!(settings.organization_reference, None);
        assert_eq!(settings.project, None);
        assert_eq!(settings.temperature_milli, Some(200));
        assert_eq!(settings.max_output_tokens, Some(1024));
        assert_eq!(settings.timeout_ms, Some(30_000));
    }

    #[test]
    fn build_remote_planner_settings_reflects_selected_ollama_profile_details() {
        let mut config = AppConfig::default();
        config.providers.planner.remote_profile = Some(String::from("ollama-default"));

        let settings = build_remote_planner_settings(&config);

        assert_eq!(settings.profile_name.as_deref(), Some("ollama-default"));
        assert_eq!(
            settings.provider,
            Some(crate::commands::RemoteProviderLabel::Ollama)
        );
        assert_eq!(
            settings.base_url.as_deref(),
            Some("http://localhost:11434/v1")
        );
        assert_eq!(settings.model.as_deref(), Some("qwen2.5:3b-instruct"));
        assert_eq!(
            settings.api_key_reference.as_deref(),
            Some("Environment variable: OLLAMA_API_KEY")
        );
        assert_eq!(settings.organization_reference, None);
        assert_eq!(settings.project, None);
        assert_eq!(settings.temperature_milli, Some(200));
        assert_eq!(settings.max_output_tokens, Some(1024));
        assert_eq!(settings.timeout_ms, Some(30_000));
    }

    #[test]
    fn build_remote_tts_settings_reflects_configured_profile_details() {
        let config = AppConfig::default();

        let settings = build_remote_tts_settings(&config);

        assert_eq!(settings.profile_name.as_deref(), Some("openai-tts-default"));
        assert_eq!(
            settings.provider,
            Some(crate::commands::RemoteProviderLabel::OpenAi)
        );
        assert_eq!(
            settings.base_url.as_deref(),
            Some("https://api.openai.com/v1")
        );
        assert_eq!(settings.model.as_deref(), Some("gpt-4o-mini-tts"));
        assert_eq!(
            settings.api_key_reference.as_deref(),
            Some("Environment variable: OPENAI_API_KEY")
        );
        assert_eq!(settings.organization_reference, None);
        assert_eq!(settings.project, None);
        assert_eq!(settings.voice.as_deref(), Some("alloy"));
        assert_eq!(
            settings.audio_format,
            Some(crate::config::RemoteTtsAudioFormat::Wav)
        );
        assert_eq!(settings.timeout_ms, Some(30_000));
    }

    #[test]
    fn build_remote_asr_settings_reflects_configured_profile_details() {
        let config = AppConfig::default();

        let settings = build_remote_asr_settings(&config);

        assert_eq!(
            settings.profile_name.as_deref(),
            Some("openai-transcribe-default")
        );
        assert_eq!(
            settings.provider,
            Some(crate::commands::RemoteProviderLabel::OpenAi)
        );
        assert_eq!(
            settings.base_url.as_deref(),
            Some("https://api.openai.com/v1")
        );
        assert_eq!(settings.model.as_deref(), Some("gpt-4o-mini-transcribe"));
        assert_eq!(
            settings.api_key_reference.as_deref(),
            Some("Environment variable: OPENAI_API_KEY")
        );
        assert_eq!(settings.organization_reference, None);
        assert_eq!(settings.project, None);
        assert_eq!(settings.language.as_deref(), Some("en"));
        assert_eq!(settings.temperature_milli, Some(0));
        assert_eq!(settings.timeout_ms, Some(30_000));
    }

    #[test]
    fn build_remote_settings_expose_secret_references_without_raw_values() {
        let mut config = AppConfig::default();
        let planner_profile = config
            .remote_planner_profiles
            .get_mut("openai-default")
            .expect("planner profile should exist");
        planner_profile.api_key = SecretRef::FromFile {
            from_file: String::from("/secure/planner.key"),
        };
        planner_profile.organization = Some(SecretRef::FromKeyring {
            from_keyring: KeyringRef {
                service: String::from("blind-browser"),
                account: String::from("planner/openai-default"),
            },
        });

        let settings = build_remote_planner_settings(&config);

        assert_eq!(
            settings.api_key_reference.as_deref(),
            Some("File reference: /secure/planner.key")
        );
        assert_eq!(
            settings.organization_reference.as_deref(),
            Some("OS keyring entry: blind-browser / planner/openai-default")
        );
        assert!(!settings
            .api_key_reference
            .as_deref()
            .unwrap_or_default()
            .contains("super-secret"));
        assert!(!settings
            .organization_reference
            .as_deref()
            .unwrap_or_default()
            .contains("super-secret"));
    }

    #[test]
    fn build_provider_failover_settings_reports_unavailable_runtime_support() {
        let config = AppConfig::default();

        let settings = build_provider_failover_settings(&config);

        assert!(!settings.planner_available);
        assert!(!settings.tts_available);
        assert!(!settings.asr_available);
        assert_eq!(
            settings.summary,
            String::from(
                "Provider failover settings are defined in config, but automatic failover is still disabled in the live runtime."
            )
        );
    }

    #[test]
    fn build_confirmation_settings_reflects_configured_safety_values() {
        let config = AppConfig::default();

        let settings = build_confirmation_settings(&config);

        assert_eq!(settings.confirmation_confidence_threshold, 0.9);
        assert!(settings.allow_click_without_confirmation);
        assert!(settings.always_confirm_submit);
    }

    #[test]
    fn build_local_tts_model_settings_reflects_configured_profile_details() {
        let config = AppConfig::default();

        let settings = build_local_tts_model_settings(&config);

        assert_eq!(settings.profile_name.as_deref(), Some("kitten-default"));
        assert_eq!(
            settings.backend,
            Some(crate::config::LocalTtsBackend::KittenTtsRs)
        );
        assert_eq!(settings.model_id.as_deref(), Some("default"));
        assert_eq!(
            settings.model_path.as_deref(),
            Some("/path/to/kitten/model")
        );
        assert_eq!(settings.default_voice.as_deref(), Some("Bruno"));
        assert_eq!(settings.sample_rate, Some(24_000));
    }

    #[test]
    fn build_tts_model_settings_uses_selected_local_profile() {
        let mut config = AppConfig::default();
        config.providers.tts.mode = ProviderMode::Local;
        config.local_tts_profiles.insert(
            String::from("kitten-alt"),
            crate::config::LocalTtsProfile {
                backend: crate::config::LocalTtsBackend::KittenTtsRs,
                model_id: String::from("expressive"),
                model_path: String::from("/path/to/kitten/expressive"),
                default_voice: String::from("Bella"),
                sample_rate: 22_050,
            },
        );
        config.providers.tts.local_profile = Some(String::from("kitten-alt"));

        let settings = build_tts_model_settings(&config);

        assert_eq!(settings.mode, ProviderMode::Local);
        assert_eq!(settings.active_profile.as_deref(), Some("kitten-alt"));
        assert!(settings
            .available_profiles
            .iter()
            .any(
                |option| option.profile_name == "kitten-default" && option.model_label == "default"
            ));
        assert!(settings
            .available_profiles
            .iter()
            .any(
                |option| option.profile_name == "kitten-alt" && option.model_label == "expressive"
            ));
    }

    #[test]
    fn build_local_tts_model_settings_reflects_selected_profile_details() {
        let mut config = AppConfig::default();
        config.local_tts_profiles.insert(
            String::from("kitten-alt"),
            crate::config::LocalTtsProfile {
                backend: crate::config::LocalTtsBackend::KittenTtsRs,
                model_id: String::from("expressive"),
                model_path: String::from("/path/to/kitten/expressive"),
                default_voice: String::from("Bella"),
                sample_rate: 22_050,
            },
        );
        config.providers.tts.local_profile = Some(String::from("kitten-alt"));

        let settings = build_local_tts_model_settings(&config);

        assert_eq!(settings.profile_name.as_deref(), Some("kitten-alt"));
        assert_eq!(
            settings.backend,
            Some(crate::config::LocalTtsBackend::KittenTtsRs)
        );
        assert_eq!(settings.model_id.as_deref(), Some("expressive"));
        assert_eq!(
            settings.model_path.as_deref(),
            Some("/path/to/kitten/expressive")
        );
        assert_eq!(settings.default_voice.as_deref(), Some("Bella"));
        assert_eq!(settings.sample_rate, Some(22_050));
    }

    #[test]
    fn build_local_asr_model_settings_reflects_configured_profile_details() {
        let config = AppConfig::default();

        let settings = build_local_asr_model_settings(&config);

        assert_eq!(settings.profile_name.as_deref(), Some("whisper-default"));
        assert_eq!(
            settings.backend,
            Some(crate::config::LocalAsrBackend::Whisper)
        );
        assert_eq!(settings.model_id.as_deref(), Some("tiny"));
        assert_eq!(
            settings.model_path.as_deref(),
            Some("/path/to/whisper/model")
        );
        assert_eq!(settings.language.as_deref(), Some("en"));
        assert_eq!(settings.threads, Some(4));
    }

    #[test]
    fn build_local_asr_model_settings_reflects_selected_profile_details() {
        let mut config = AppConfig::default();
        config.local_asr_profiles.insert(
            String::from("whisper-alt"),
            crate::config::LocalAsrProfile {
                backend: crate::config::LocalAsrBackend::Whisper,
                model_id: String::from("base"),
                model_path: String::from("/path/to/whisper/base"),
                language: Some(String::from("fr")),
                threads: 6,
            },
        );
        config.providers.asr.local_profile = Some(String::from("whisper-alt"));

        let settings = build_local_asr_model_settings(&config);

        assert_eq!(settings.profile_name.as_deref(), Some("whisper-alt"));
        assert_eq!(
            settings.backend,
            Some(crate::config::LocalAsrBackend::Whisper)
        );
        assert_eq!(settings.model_id.as_deref(), Some("base"));
        assert_eq!(
            settings.model_path.as_deref(),
            Some("/path/to/whisper/base")
        );
        assert_eq!(settings.language.as_deref(), Some("fr"));
        assert_eq!(settings.threads, Some(6));
    }

    #[test]
    fn build_ocr_threshold_settings_reflects_configured_ocr_values() {
        let config = AppConfig::default();

        let settings = build_ocr_threshold_settings(&config);

        assert_eq!(settings.sparse_text_char_threshold, 200);
        assert_eq!(settings.sparse_text_region_threshold, 2);
    }

    #[test]
    fn build_asr_provider_settings_returns_available_modes() {
        let config = AppConfig::default();

        let settings = build_asr_provider_settings(&config);

        assert_eq!(settings.active_mode, ProviderMode::Remote);
        assert_eq!(
            settings.available_modes,
            vec![ProviderMode::Local, ProviderMode::Remote]
        );
    }

    #[test]
    fn build_tts_provider_settings_returns_available_modes() {
        let config = AppConfig::default();

        let settings = build_tts_provider_settings(&config);

        assert_eq!(settings.active_mode, ProviderMode::Remote);
        assert_eq!(
            settings.available_modes,
            vec![ProviderMode::Local, ProviderMode::Remote]
        );
    }

    #[test]
    fn build_tts_voice_settings_returns_kitten_voice_choices_for_local_mode() {
        let mut config = AppConfig::default();
        config.providers.tts.mode = ProviderMode::Local;
        let runtime_audio = RuntimeAudioState::from(&config.audio);

        let settings = build_tts_voice_settings(&config, &runtime_audio);

        assert_eq!(settings.mode, ProviderMode::Local);
        assert_eq!(settings.active_voice.as_deref(), Some("Bruno"));
        assert_eq!(settings.available_voices.len(), 8);
        assert!(settings
            .available_voices
            .iter()
            .any(|option| option.voice_name == "Bella"));
        assert!(settings
            .available_voices
            .iter()
            .any(|option| option.voice_name == "Leo"));
    }

    #[test]
    fn build_tts_voice_settings_preserves_custom_active_voice() {
        let config = AppConfig::default();
        let runtime_audio = RuntimeAudioState {
            tts_voice: Some(String::from("CustomVoice")),
            ..RuntimeAudioState::from(&config.audio)
        };

        let settings = build_tts_voice_settings(&config, &runtime_audio);

        assert_eq!(settings.active_voice.as_deref(), Some("CustomVoice"));
        assert_eq!(settings.available_voices[0].voice_name, "CustomVoice");
    }

    #[test]
    fn build_tts_voice_settings_returns_openai_builtin_voices_for_remote_mode() {
        let mut config = AppConfig::default();
        config.providers.tts.mode = ProviderMode::Remote;
        let runtime_audio = RuntimeAudioState {
            tts_voice: Some(String::from("Alloy")),
            ..RuntimeAudioState::from(&config.audio)
        };

        let settings = build_tts_voice_settings(&config, &runtime_audio);

        assert_eq!(settings.mode, ProviderMode::Remote);
        assert_eq!(settings.active_voice.as_deref(), Some("alloy"));
        assert!(settings
            .available_voices
            .iter()
            .any(|option| option.voice_name == "cedar"));
    }

    #[test]
    fn normalize_optional_text_trims_and_drops_empty_values() {
        assert_eq!(normalize_optional_text(None), None);
        assert_eq!(normalize_optional_text(Some(String::from("   "))), None);
        assert_eq!(
            normalize_optional_text(Some(String::from("  next step  "))),
            Some(String::from("next step"))
        );
    }

    #[test]
    fn normalize_absolute_url_accepts_trimmed_absolute_urls() {
        assert_eq!(
            normalize_absolute_url("  https://example.com/page  ").unwrap(),
            String::from("https://example.com/page")
        );
        assert_eq!(
            normalize_absolute_url("about:blank").unwrap(),
            String::from("about:blank")
        );
    }

    #[test]
    fn normalize_absolute_url_rejects_relative_urls() {
        let error = normalize_absolute_url("/relative/path").unwrap_err();
        assert_eq!(error.code, "invalid_url");
    }

    #[test]
    fn browser_error_to_tool_error_keeps_navigation_failures_retryable_and_structured() {
        let navigate_error = browser_error_to_tool_error(
            String::from("open_url failed to navigate the active page"),
            BrowserError::Navigate(String::from("dns resolution failed")),
        );
        assert_eq!(navigate_error.code, "browser_navigation_failed");
        assert!(navigate_error.retryable);
        assert_eq!(
            navigate_error.details,
            Some(serde_json::json!({
                "reason": "failed to navigate browser page: dns resolution failed"
            }))
        );

        let history_error = browser_error_to_tool_error(
            String::from("go_back failed to update the current page"),
            BrowserError::History(String::from("no previous entry")),
        );
        assert_eq!(history_error.code, "browser_history_failed");
        assert!(history_error.retryable);
        assert_eq!(
            history_error.details,
            Some(serde_json::json!({
                "reason": "failed to read browser navigation history: no previous entry"
            }))
        );
    }

    #[test]
    fn refresh_current_page_after_navigation_replaces_metadata_and_clears_stale_content() {
        let mut current_page = Some(PageModel {
            title: Some(String::from("Old page")),
            url: Some(String::from("https://example.com/old")),
            regions: vec![PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Paragraph,
                label: None,
                text: String::from("Stale extracted text"),
                bbox: None,
                source: RegionSource::Dom,
            }],
            interactive_elements: vec![InteractiveElement {
                element_id: String::from("button-1"),
                dom_locator: Some(String::from("#old-button")),
                role: ElementRole::Button,
                tag_name: String::from("button"),
                text: Some(String::from("Continue")),
                accessible_name: Some(String::from("Continue")),
                placeholder: None,
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            }],
        });

        refresh_current_page_after_navigation(
            &mut current_page,
            Some(String::from("https://example.com/new")),
            Some(String::from("New page")),
        );

        let current_page = current_page.expect("page should still exist");
        assert_eq!(current_page.url.as_deref(), Some("https://example.com/new"));
        assert_eq!(current_page.title.as_deref(), Some("New page"));
        assert!(current_page.regions.is_empty());
        assert!(current_page.interactive_elements.is_empty());
    }

    #[test]
    fn clear_navigation_follow_up_state_resets_cursor_and_recent_field_context() {
        let mut state = AppState::default();
        state.narration_cursor.current_index = Some(3);
        state.narration_cursor.current_region_id = Some(String::from("region-3"));
        state.narration_cursor.total_regions = 8;

        let mut recent_field_context = Some(RecentFieldContext {
            page_id: String::from("page-1"),
            target_description: Some(String::from("email field")),
            active_element_id: Some(String::from("input-email")),
            candidate_element_ids: vec![String::from("input-email"), String::from("input-alt")],
            pending_text: Some(String::from("user@example.com")),
            submit_after: true,
        });

        clear_navigation_follow_up_state(&mut state, &mut recent_field_context);

        assert_eq!(state.narration_cursor, Default::default());
        assert_eq!(recent_field_context, None);
    }

    #[test]
    fn build_visible_text_excerpt_joins_regions_and_applies_limit() {
        let page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: vec![
                PageRegion {
                    region_id: String::from("region-1"),
                    role: RegionRole::Other,
                    label: None,
                    text: String::from("First paragraph"),
                    bbox: None,
                    source: RegionSource::Dom,
                },
                PageRegion {
                    region_id: String::from("region-2"),
                    role: RegionRole::Other,
                    label: None,
                    text: String::from("Second paragraph"),
                    bbox: None,
                    source: RegionSource::Dom,
                },
            ],
            interactive_elements: Vec::new(),
        };

        assert_eq!(
            build_visible_text_excerpt(&page, None),
            String::from("First paragraph\n\nSecond paragraph")
        );
        assert_eq!(
            build_visible_text_excerpt(&page, Some(5)),
            String::from("First")
        );
    }

    #[test]
    fn region_bbox_by_id_returns_region_geometry_when_available() {
        let regions = vec![PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Section,
            label: Some(String::from("Main")),
            text: String::from("Text"),
            bbox: Some(Rect {
                x: 10.0,
                y: 20.0,
                width: 30.0,
                height: 40.0,
            }),
            source: RegionSource::Dom,
        }];

        assert_eq!(
            region_bbox_by_id(&regions, "region-1").expect("region bbox should resolve"),
            Rect {
                x: 10.0,
                y: 20.0,
                width: 30.0,
                height: 40.0,
            }
        );
    }

    #[test]
    fn build_extracted_page_model_can_omit_links() {
        let page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: Vec::new(),
            interactive_elements: vec![
                InteractiveElement {
                    element_id: String::from("link-1"),
                    dom_locator: Some(String::from("#link-1")),
                    role: ElementRole::Link,
                    tag_name: String::from("a"),
                    text: Some(String::from("Read more")),
                    accessible_name: Some(String::from("Read more")),
                    placeholder: None,
                    href: Some(String::from("https://example.com/more")),
                    value: None,
                    bbox: None,
                    visible: true,
                    enabled: true,
                    attributes: std::collections::BTreeMap::new(),
                },
                InteractiveElement {
                    element_id: String::from("button-1"),
                    dom_locator: Some(String::from("#button-1")),
                    role: ElementRole::Button,
                    tag_name: String::from("button"),
                    text: Some(String::from("Continue")),
                    accessible_name: Some(String::from("Continue")),
                    placeholder: None,
                    href: None,
                    value: None,
                    bbox: None,
                    visible: true,
                    enabled: true,
                    attributes: std::collections::BTreeMap::new(),
                },
            ],
        };
        let input = ExtractPageModelInput {
            request_id: String::from("req-extract"),
            timeout_ms: None,
            use_dom_extraction: true,
            include_headings: true,
            include_links: false,
        };

        let extracted = build_extracted_page_model(&page, &input);

        assert_eq!(extracted.interactive_elements.len(), 1);
        assert_eq!(extracted.interactive_elements[0].role, ElementRole::Button);
    }

    #[test]
    fn build_extracted_page_model_preserves_link_metadata_when_requested() {
        let page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: Vec::new(),
            interactive_elements: vec![InteractiveElement {
                element_id: String::from("link-1"),
                dom_locator: Some(String::from("#link-1")),
                role: ElementRole::Link,
                tag_name: String::from("a"),
                text: Some(String::from("Read more")),
                accessible_name: Some(String::from("Read more about examples")),
                placeholder: None,
                href: Some(String::from("https://example.com/more")),
                value: None,
                bbox: Some(Rect {
                    x: 10.0,
                    y: 20.0,
                    width: 30.0,
                    height: 12.0,
                }),
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::from([(
                    String::from("rel"),
                    String::from("noopener"),
                )]),
            }],
        };
        let input = ExtractPageModelInput {
            request_id: String::from("req-extract"),
            timeout_ms: None,
            use_dom_extraction: true,
            include_headings: true,
            include_links: true,
        };

        let extracted = build_extracted_page_model(&page, &input);

        assert_eq!(extracted.interactive_elements.len(), 1);
        let link = &extracted.interactive_elements[0];
        assert_eq!(link.role, ElementRole::Link);
        assert_eq!(link.href.as_deref(), Some("https://example.com/more"));
        assert_eq!(link.text.as_deref(), Some("Read more"));
        assert_eq!(
            link.accessible_name.as_deref(),
            Some("Read more about examples")
        );
        assert_eq!(
            link.bbox,
            Some(Rect {
                x: 10.0,
                y: 20.0,
                width: 30.0,
                height: 12.0,
            })
        );
        assert_eq!(
            link.attributes.get("rel").map(String::as_str),
            Some("noopener")
        );
    }

    #[test]
    fn build_extracted_page_model_preserves_region_order_and_sources() {
        let page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: vec![
                PageRegion {
                    region_id: String::from("dom-region-title"),
                    role: RegionRole::Title,
                    label: Some(String::from("Title")),
                    text: String::from("Example"),
                    bbox: None,
                    source: RegionSource::Dom,
                },
                PageRegion {
                    region_id: String::from("dom-region-1"),
                    role: RegionRole::Other,
                    label: None,
                    text: String::from("First paragraph."),
                    bbox: None,
                    source: RegionSource::Dom,
                },
                PageRegion {
                    region_id: String::from("ocr-region-1"),
                    role: RegionRole::Other,
                    label: None,
                    text: String::from("Recovered OCR text."),
                    bbox: None,
                    source: RegionSource::Ocr,
                },
            ],
            interactive_elements: Vec::new(),
        };
        let input = ExtractPageModelInput {
            request_id: String::from("req-extract"),
            timeout_ms: None,
            use_dom_extraction: true,
            include_headings: true,
            include_links: true,
        };

        let extracted = build_extracted_page_model(&page, &input);

        let ordered_region_ids = extracted
            .regions
            .iter()
            .map(|region| region.region_id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            ordered_region_ids,
            vec!["dom-region-title", "dom-region-1", "ocr-region-1"]
        );
        assert_eq!(
            extracted
                .regions
                .iter()
                .map(|region| region.source.clone())
                .collect::<Vec<_>>(),
            vec![RegionSource::Dom, RegionSource::Dom, RegionSource::Ocr]
        );
    }

    #[test]
    fn build_extracted_page_model_leaves_heading_regions_unchanged_when_disabled() {
        let page = PageModel {
            title: Some(String::from("Example article")),
            url: Some(String::from("https://example.com/article")),
            regions: vec![
                PageRegion {
                    region_id: String::from("region-title"),
                    role: RegionRole::Title,
                    label: Some(String::from("Title")),
                    text: String::from("Example article"),
                    bbox: None,
                    source: RegionSource::Dom,
                },
                PageRegion {
                    region_id: String::from("region-heading"),
                    role: RegionRole::Heading,
                    label: Some(String::from("Heading")),
                    text: String::from("Section one"),
                    bbox: None,
                    source: RegionSource::Dom,
                },
                PageRegion {
                    region_id: String::from("region-paragraph"),
                    role: RegionRole::Paragraph,
                    label: None,
                    text: String::from("First paragraph."),
                    bbox: None,
                    source: RegionSource::Dom,
                },
            ],
            interactive_elements: Vec::new(),
        };
        let input = ExtractPageModelInput {
            request_id: String::from("req-extract"),
            timeout_ms: None,
            use_dom_extraction: false,
            include_headings: false,
            include_links: true,
        };

        let extracted = build_extracted_page_model(&page, &input);

        assert_eq!(extracted.title, page.title);
        assert_eq!(extracted.url, page.url);
        assert_eq!(extracted.regions, page.regions);
    }

    #[test]
    fn infer_extraction_source_detects_merged_models() {
        let page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: vec![
                PageRegion {
                    region_id: String::from("dom-region"),
                    role: RegionRole::Other,
                    label: None,
                    text: String::from("DOM text"),
                    bbox: None,
                    source: RegionSource::Dom,
                },
                PageRegion {
                    region_id: String::from("ocr-region"),
                    role: RegionRole::Other,
                    label: None,
                    text: String::from("OCR text"),
                    bbox: None,
                    source: RegionSource::Ocr,
                },
            ],
            interactive_elements: Vec::new(),
        };

        assert_eq!(
            infer_extraction_source(&page, true, false),
            ExtractionSource::Merged
        );
    }

    #[test]
    fn infer_extraction_source_treats_mixed_regions_as_merged() {
        let page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: vec![PageRegion {
                region_id: String::from("mixed-region"),
                role: RegionRole::Other,
                label: None,
                text: String::from("DOM text\n\nOCR text"),
                bbox: None,
                source: RegionSource::Mixed,
            }],
            interactive_elements: Vec::new(),
        };

        assert_eq!(
            infer_extraction_source(&page, true, false),
            ExtractionSource::Merged
        );
    }

    #[test]
    fn infer_extraction_source_reports_dom_smoothie_when_dom_only() {
        let page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: vec![PageRegion {
                region_id: String::from("dom-region"),
                role: RegionRole::Other,
                label: None,
                text: String::from("Readable text"),
                bbox: None,
                source: RegionSource::Dom,
            }],
            interactive_elements: Vec::new(),
        };

        assert_eq!(
            infer_extraction_source(&page, true, true),
            ExtractionSource::DomSmoothie
        );
        assert_eq!(
            infer_extraction_source(&page, true, false),
            ExtractionSource::DomFallback
        );
    }

    #[test]
    fn should_trigger_no_extractable_text_ocr_fallback_when_dom_regions_are_empty() {
        let page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: vec![PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Other,
                label: None,
                text: String::from("   "),
                bbox: None,
                source: RegionSource::Dom,
            }],
            interactive_elements: Vec::new(),
        };

        assert!(should_trigger_extract_page_model_ocr_fallback(
            true,
            &page,
            &OcrSettings::default()
        ));
    }

    #[test]
    fn extracted_text_metrics_counts_trimmed_text_and_regions() {
        let page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: vec![
                PageRegion {
                    region_id: String::from("region-1"),
                    role: RegionRole::Other,
                    label: None,
                    text: String::from("  Visible DOM text  "),
                    bbox: None,
                    source: RegionSource::Dom,
                },
                PageRegion {
                    region_id: String::from("region-2"),
                    role: RegionRole::Other,
                    label: None,
                    text: String::from(" "),
                    bbox: None,
                    source: RegionSource::Dom,
                },
            ],
            interactive_elements: Vec::new(),
        };

        assert_eq!(extracted_text_metrics(&page), (16, 1));
    }

    #[test]
    fn should_trigger_extract_page_model_ocr_fallback_when_text_is_below_char_threshold() {
        let page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: vec![PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Other,
                label: None,
                text: String::from("Short text"),
                bbox: None,
                source: RegionSource::Dom,
            }],
            interactive_elements: Vec::new(),
        };
        let settings = OcrSettings {
            sparse_text_char_threshold: 20,
            sparse_text_region_threshold: 1,
            ..OcrSettings::default()
        };

        assert!(should_trigger_extract_page_model_ocr_fallback(
            true, &page, &settings
        ));
    }

    #[test]
    fn should_trigger_extract_page_model_ocr_fallback_when_region_count_is_below_threshold() {
        let page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: vec![PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Other,
                label: None,
                text: String::from("This region has enough text to pass the char threshold alone."),
                bbox: None,
                source: RegionSource::Dom,
            }],
            interactive_elements: Vec::new(),
        };
        let settings = OcrSettings {
            sparse_text_char_threshold: 10,
            sparse_text_region_threshold: 2,
            ..OcrSettings::default()
        };

        assert!(should_trigger_extract_page_model_ocr_fallback(
            true, &page, &settings
        ));
    }

    #[test]
    fn should_trigger_extract_page_model_ocr_fallback_at_default_char_boundary() {
        let page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: vec![
                PageRegion {
                    region_id: String::from("region-1"),
                    role: RegionRole::Other,
                    label: None,
                    text: "a".repeat(100),
                    bbox: None,
                    source: RegionSource::Dom,
                },
                PageRegion {
                    region_id: String::from("region-2"),
                    role: RegionRole::Other,
                    label: None,
                    text: "b".repeat(100),
                    bbox: None,
                    source: RegionSource::Dom,
                },
            ],
            interactive_elements: Vec::new(),
        };

        assert!(should_trigger_extract_page_model_ocr_fallback(
            true,
            &page,
            &OcrSettings::default()
        ));
    }

    #[test]
    fn should_trigger_extract_page_model_ocr_fallback_at_default_region_boundary() {
        let page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: vec![PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Other,
                label: None,
                text: "a".repeat(201),
                bbox: None,
                source: RegionSource::Dom,
            }],
            interactive_elements: Vec::new(),
        };

        assert!(should_trigger_extract_page_model_ocr_fallback(
            true,
            &page,
            &OcrSettings::default()
        ));
    }

    #[test]
    fn should_not_trigger_extract_page_model_ocr_fallback_above_default_boundaries() {
        let page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: vec![
                PageRegion {
                    region_id: String::from("region-1"),
                    role: RegionRole::Other,
                    label: None,
                    text: "a".repeat(101),
                    bbox: None,
                    source: RegionSource::Dom,
                },
                PageRegion {
                    region_id: String::from("region-2"),
                    role: RegionRole::Other,
                    label: None,
                    text: "b".repeat(100),
                    bbox: None,
                    source: RegionSource::Dom,
                },
            ],
            interactive_elements: Vec::new(),
        };

        assert!(!should_trigger_extract_page_model_ocr_fallback(
            true,
            &page,
            &OcrSettings::default()
        ));
    }

    #[test]
    fn should_not_trigger_extract_page_model_ocr_fallback_when_thresholds_are_satisfied() {
        let page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: vec![
                PageRegion {
                    region_id: String::from("region-1"),
                    role: RegionRole::Other,
                    label: None,
                    text: String::from(
                        "This first region contains comfortably more than twenty characters.",
                    ),
                    bbox: None,
                    source: RegionSource::Dom,
                },
                PageRegion {
                    region_id: String::from("region-2"),
                    role: RegionRole::Other,
                    label: None,
                    text: String::from("This second region also contains enough text."),
                    bbox: None,
                    source: RegionSource::Dom,
                },
            ],
            interactive_elements: Vec::new(),
        };
        let settings = OcrSettings {
            sparse_text_char_threshold: 20,
            sparse_text_region_threshold: 2,
            ..OcrSettings::default()
        };

        assert!(!should_trigger_extract_page_model_ocr_fallback(
            true, &page, &settings
        ));
    }

    #[test]
    fn should_not_trigger_extract_page_model_ocr_fallback_when_disabled_or_non_dom() {
        let page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: vec![PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Other,
                label: None,
                text: String::new(),
                bbox: None,
                source: RegionSource::Dom,
            }],
            interactive_elements: Vec::new(),
        };
        let disabled_settings = OcrSettings {
            trigger_on_no_extractable_text: false,
            ..OcrSettings::default()
        };

        assert!(!should_trigger_extract_page_model_ocr_fallback(
            true,
            &page,
            &disabled_settings
        ));
        assert!(!should_trigger_extract_page_model_ocr_fallback(
            false,
            &page,
            &OcrSettings::default()
        ));
    }

    #[test]
    fn region_first_ocr_target_ids_prefers_bbox_backed_readable_regions() {
        let page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: vec![
                PageRegion {
                    region_id: String::from("region-1"),
                    role: RegionRole::Other,
                    label: None,
                    text: String::from("Readable text"),
                    bbox: Some(Rect {
                        x: 1.0,
                        y: 2.0,
                        width: 30.0,
                        height: 40.0,
                    }),
                    source: RegionSource::Dom,
                },
                PageRegion {
                    region_id: String::from("region-2"),
                    role: RegionRole::Other,
                    label: None,
                    text: String::from("Readable but no bbox"),
                    bbox: None,
                    source: RegionSource::Dom,
                },
                PageRegion {
                    region_id: String::from("region-3"),
                    role: RegionRole::Other,
                    label: None,
                    text: String::from(""),
                    bbox: Some(Rect {
                        x: 5.0,
                        y: 6.0,
                        width: 50.0,
                        height: 60.0,
                    }),
                    source: RegionSource::Dom,
                },
                PageRegion {
                    region_id: String::from("region-4"),
                    role: RegionRole::Other,
                    label: None,
                    text: String::from("Readable but invalid bbox"),
                    bbox: Some(Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 0.0,
                        height: 10.0,
                    }),
                    source: RegionSource::Dom,
                },
            ],
            interactive_elements: Vec::new(),
        };

        assert_eq!(
            region_first_ocr_target_ids(&page, &OcrSettings::default()),
            vec![String::from("region-1")]
        );
    }

    #[test]
    fn region_first_ocr_target_ids_respects_preference_toggle() {
        let page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: vec![PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Other,
                label: None,
                text: String::from("Readable text"),
                bbox: Some(Rect {
                    x: 1.0,
                    y: 2.0,
                    width: 30.0,
                    height: 40.0,
                }),
                source: RegionSource::Dom,
            }],
            interactive_elements: Vec::new(),
        };
        let settings = OcrSettings {
            prefer_region_ocr: false,
            ..OcrSettings::default()
        };

        assert!(region_first_ocr_target_ids(&page, &settings).is_empty());
    }

    #[test]
    fn merged_region_text_prefers_more_complete_or_combined_text() {
        assert_eq!(
            merged_region_text("Short label", "Short label with extra detail"),
            String::from("Short label with extra detail")
        );
        assert_eq!(
            merged_region_text("DOM text", "OCR text"),
            String::from("DOM text\n\nOCR text")
        );
    }

    #[test]
    fn merge_ocr_text_into_page_model_updates_existing_region_as_mixed_and_adopts_bbox() {
        let mut page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: vec![PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Section,
                label: Some(String::from("Main")),
                text: String::from("DOM summary"),
                bbox: None,
                source: RegionSource::Dom,
            }],
            interactive_elements: Vec::new(),
        };

        let updated_region_ids = merge_ocr_text_into_page_model(
            &mut page,
            Some("region-1"),
            "OCR detail",
            Some(Rect {
                x: 10.0,
                y: 20.0,
                width: 30.0,
                height: 40.0,
            }),
            String::from("unused"),
        )
        .expect("merge should update the requested region");

        assert_eq!(updated_region_ids, vec![String::from("region-1")]);
        assert_eq!(page.regions[0].source, RegionSource::Mixed);
        assert_eq!(
            page.regions[0].text,
            String::from("DOM summary\n\nOCR detail")
        );
        assert_eq!(
            page.regions[0].bbox,
            Some(Rect {
                x: 10.0,
                y: 20.0,
                width: 30.0,
                height: 40.0,
            })
        );
    }

    #[test]
    fn merge_ocr_text_into_page_model_preserves_existing_region_bbox() {
        let mut page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: vec![PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Section,
                label: Some(String::from("Main")),
                text: String::from("DOM summary"),
                bbox: Some(Rect {
                    x: 1.0,
                    y: 2.0,
                    width: 3.0,
                    height: 4.0,
                }),
                source: RegionSource::Dom,
            }],
            interactive_elements: Vec::new(),
        };

        let updated_region_ids = merge_ocr_text_into_page_model(
            &mut page,
            Some("region-1"),
            "OCR detail",
            Some(Rect {
                x: 10.0,
                y: 20.0,
                width: 30.0,
                height: 40.0,
            }),
            String::from("unused"),
        )
        .expect("merge should update the requested region");

        assert_eq!(updated_region_ids, vec![String::from("region-1")]);
        assert_eq!(
            page.regions[0].bbox,
            Some(Rect {
                x: 1.0,
                y: 2.0,
                width: 3.0,
                height: 4.0,
            })
        );
    }

    #[test]
    fn merge_ocr_text_into_page_model_appends_new_ocr_region_when_target_missing() {
        let mut page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: Vec::new(),
            interactive_elements: Vec::new(),
        };

        let updated_region_ids = merge_ocr_text_into_page_model(
            &mut page,
            None,
            "Recovered OCR text",
            Some(Rect {
                x: 5.0,
                y: 6.0,
                width: 70.0,
                height: 80.0,
            }),
            String::from("ocr-region-generated"),
        )
        .expect("merge should create a new OCR region when no target region_id is supplied");

        assert_eq!(
            updated_region_ids,
            vec![String::from("ocr-region-generated")]
        );
        assert_eq!(page.regions.len(), 1);
        assert_eq!(page.regions[0].region_id, "ocr-region-generated");
        assert_eq!(page.regions[0].source, RegionSource::Ocr);
        assert_eq!(page.regions[0].text, "Recovered OCR text");
        assert_eq!(
            page.regions[0].bbox,
            Some(Rect {
                x: 5.0,
                y: 6.0,
                width: 70.0,
                height: 80.0,
            })
        );
    }

    #[test]
    fn merge_ocr_text_into_page_model_rejects_blank_ocr_text() {
        let mut page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: vec![PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Paragraph,
                label: None,
                text: String::from("Existing text"),
                bbox: None,
                source: RegionSource::Dom,
            }],
            interactive_elements: Vec::new(),
        };

        let error = merge_ocr_text_into_page_model(
            &mut page,
            Some("region-1"),
            "   ",
            None,
            String::from("ocr-region-1"),
        )
        .unwrap_err();

        assert_eq!(error.code, "invalid_ocr_text");
        assert_eq!(page.regions[0].text, "Existing text");
        assert_eq!(page.regions[0].source, RegionSource::Dom);
    }

    #[test]
    fn merge_ocr_text_into_page_model_rejects_unknown_target_region() {
        let mut page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: vec![PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Paragraph,
                label: None,
                text: String::from("Existing text"),
                bbox: None,
                source: RegionSource::Dom,
            }],
            interactive_elements: Vec::new(),
        };

        let error = merge_ocr_text_into_page_model(
            &mut page,
            Some("missing-region"),
            "Scanned text",
            Some(Rect {
                x: 10.0,
                y: 20.0,
                width: 100.0,
                height: 40.0,
            }),
            String::from("ocr-region-1"),
        )
        .unwrap_err();

        assert_eq!(error.code, "unknown_region_id");
        assert_eq!(
            error.details,
            Some(serde_json::json!({ "region_id": "missing-region" }))
        );
        assert_eq!(page.regions.len(), 1);
        assert_eq!(page.regions[0].text, "Existing text");
    }

    #[test]
    fn filter_interactive_elements_applies_visibility_and_role_filters() {
        let elements = vec![
            InteractiveElement {
                element_id: String::from("button-1"),
                dom_locator: Some(String::from("#button-1")),
                role: ElementRole::Button,
                tag_name: String::from("button"),
                text: Some(String::from("Continue")),
                accessible_name: Some(String::from("Continue")),
                placeholder: None,
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
            InteractiveElement {
                element_id: String::from("link-1"),
                dom_locator: Some(String::from("#link-1")),
                role: ElementRole::Link,
                tag_name: String::from("a"),
                text: Some(String::from("Read more")),
                accessible_name: Some(String::from("Read more")),
                placeholder: None,
                href: Some(String::from("https://example.com/more")),
                value: None,
                bbox: None,
                visible: false,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
        ];

        let filtered = filter_interactive_elements(&elements, true, Some(&[ElementRole::Button]));

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].element_id, "button-1");
    }

    #[test]
    fn resolve_direct_focus_field_command_focuses_single_matching_field() {
        let page = PageModel {
            title: Some(String::from("Example form")),
            url: Some(String::from("https://example.com/form")),
            regions: Vec::new(),
            interactive_elements: vec![
                InteractiveElement {
                    element_id: String::from("input-email"),
                    dom_locator: Some(String::from("#email")),
                    role: ElementRole::Input,
                    tag_name: String::from("input"),
                    text: None,
                    accessible_name: Some(String::from("Email")),
                    placeholder: Some(String::from("Email address")),
                    href: None,
                    value: None,
                    bbox: None,
                    visible: true,
                    enabled: true,
                    attributes: std::collections::BTreeMap::new(),
                },
                InteractiveElement {
                    element_id: String::from("input-password"),
                    dom_locator: Some(String::from("#password")),
                    role: ElementRole::Input,
                    tag_name: String::from("input"),
                    text: None,
                    accessible_name: Some(String::from("Password")),
                    placeholder: Some(String::from("Password")),
                    href: None,
                    value: None,
                    bbox: None,
                    visible: true,
                    enabled: true,
                    attributes: std::collections::BTreeMap::new(),
                },
            ],
        };

        let planner_output = resolve_direct_focus_field_command(
            "focus the email field",
            "req-focus-field",
            Some(&page),
            &[String::from("focus_field")],
            0.9,
        )
        .expect("focus-field command should resolve");

        assert_eq!(planner_output.intent.name, IntentName::FillInput);
        assert_eq!(
            planner_output.selected_skills,
            vec![String::from("focus_field")]
        );
        assert_eq!(planner_output.steps.len(), 1);
        assert_eq!(planner_output.steps[0].tool_name, ToolName::FocusElement);
        assert_eq!(
            planner_output.steps[0].arguments.get("element_id"),
            Some(&serde_json::json!("input-email"))
        );
    }

    #[test]
    fn resolve_direct_focus_field_command_reports_missing_description() {
        let page = PageModel {
            title: Some(String::from("Example form")),
            url: Some(String::from("https://example.com/form")),
            regions: Vec::new(),
            interactive_elements: Vec::new(),
        };

        let planner_output = resolve_direct_focus_field_command(
            "focus field",
            "req-focus-field-missing",
            Some(&page),
            &[String::from("focus_field")],
            0.9,
        )
        .expect("focus-field command should resolve");

        assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
        assert_eq!(
            planner_output.steps[0].arguments.get("status"),
            Some(&serde_json::json!(ReportStatus::NeedsFollowUp))
        );
    }

    #[test]
    fn resolve_direct_focus_field_command_reports_ambiguous_match() {
        let page = PageModel {
            title: Some(String::from("Example form")),
            url: Some(String::from("https://example.com/form")),
            regions: Vec::new(),
            interactive_elements: vec![
                InteractiveElement {
                    element_id: String::from("input-email"),
                    dom_locator: Some(String::from("#email")),
                    role: ElementRole::Input,
                    tag_name: String::from("input"),
                    text: None,
                    accessible_name: Some(String::from("Email")),
                    placeholder: Some(String::from("Email address")),
                    href: None,
                    value: None,
                    bbox: None,
                    visible: true,
                    enabled: true,
                    attributes: std::collections::BTreeMap::new(),
                },
                InteractiveElement {
                    element_id: String::from("input-email-confirm"),
                    dom_locator: Some(String::from("#email-confirm")),
                    role: ElementRole::Input,
                    tag_name: String::from("input"),
                    text: None,
                    accessible_name: Some(String::from("Email confirmation")),
                    placeholder: Some(String::from("Confirm email")),
                    href: None,
                    value: None,
                    bbox: None,
                    visible: true,
                    enabled: true,
                    attributes: std::collections::BTreeMap::new(),
                },
            ],
        };

        let planner_output = resolve_direct_focus_field_command(
            "focus the email field",
            "req-focus-field-ambiguous",
            Some(&page),
            &[String::from("focus_field")],
            0.95,
        )
        .expect("focus-field command should resolve");

        assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
        assert_eq!(
            planner_output.steps[0].arguments.get("status"),
            Some(&serde_json::json!(ReportStatus::NeedsFollowUp))
        );
    }

    #[test]
    fn resolve_direct_fill_field_command_focuses_then_types_into_matching_field() {
        let page = PageModel {
            title: Some(String::from("Example form")),
            url: Some(String::from("https://example.com/form")),
            regions: Vec::new(),
            interactive_elements: vec![
                InteractiveElement {
                    element_id: String::from("input-email"),
                    dom_locator: Some(String::from("#email")),
                    role: ElementRole::Input,
                    tag_name: String::from("input"),
                    text: None,
                    accessible_name: Some(String::from("Email")),
                    placeholder: Some(String::from("Email address")),
                    href: None,
                    value: None,
                    bbox: None,
                    visible: true,
                    enabled: true,
                    attributes: std::collections::BTreeMap::new(),
                },
                InteractiveElement {
                    element_id: String::from("input-password"),
                    dom_locator: Some(String::from("#password")),
                    role: ElementRole::Input,
                    tag_name: String::from("input"),
                    text: None,
                    accessible_name: Some(String::from("Password")),
                    placeholder: Some(String::from("Password")),
                    href: None,
                    value: None,
                    bbox: None,
                    visible: true,
                    enabled: true,
                    attributes: std::collections::BTreeMap::new(),
                },
            ],
        };

        let planner_output = resolve_direct_fill_field_command(
            "fill the email field with phil@example.com",
            "req-fill-field",
            Some(&page),
            &[String::from("fill_field_by_label")],
            0.9,
        )
        .expect("fill-field command should resolve");

        assert_eq!(planner_output.intent.name, IntentName::FillInput);
        assert_eq!(
            planner_output.selected_skills,
            vec![String::from("fill_field_by_label")]
        );
        assert_eq!(planner_output.steps.len(), 2);
        assert_eq!(planner_output.steps[0].tool_name, ToolName::FocusElement);
        assert_eq!(planner_output.steps[1].tool_name, ToolName::TypeIntoElement);
        assert_eq!(
            planner_output.steps[1].arguments.get("text"),
            Some(&serde_json::json!("phil@example.com"))
        );
    }

    #[test]
    fn resolve_direct_fill_field_command_reports_missing_value() {
        let page = PageModel {
            title: Some(String::from("Example form")),
            url: Some(String::from("https://example.com/form")),
            regions: Vec::new(),
            interactive_elements: Vec::new(),
        };

        let planner_output = resolve_direct_fill_field_command(
            "fill the email field",
            "req-fill-field-missing-value",
            Some(&page),
            &[String::from("fill_field_by_label")],
            0.9,
        )
        .expect("fill-field command should resolve");

        assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
        assert_eq!(
            planner_output.steps[0].arguments.get("status"),
            Some(&serde_json::json!(ReportStatus::NeedsFollowUp))
        );
    }

    #[test]
    fn resolve_direct_fill_and_submit_command_builds_confirmation_gated_plan() {
        let page = PageModel {
            title: Some(String::from("Example form")),
            url: Some(String::from("https://example.com/form")),
            regions: Vec::new(),
            interactive_elements: vec![InteractiveElement {
                element_id: String::from("input-email"),
                dom_locator: Some(String::from("#email")),
                role: ElementRole::Input,
                tag_name: String::from("input"),
                text: None,
                accessible_name: Some(String::from("Email")),
                placeholder: Some(String::from("Email address")),
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            }],
        };

        let planner_output = resolve_direct_fill_and_submit_command(
            "fill the email field with phil@example.com and then submit",
            "req-fill-submit",
            Some(&page),
            &[String::from("fill_and_submit_form")],
            0.9,
        )
        .expect("fill-and-submit command should resolve");

        assert_eq!(planner_output.intent.name, IntentName::SubmitForm);
        assert_eq!(planner_output.status, PlannerStatus::NeedsConfirmation);
        assert_eq!(
            planner_output.selected_skills,
            vec![String::from("fill_and_submit_form")]
        );
        assert_eq!(planner_output.steps.len(), 4);
        assert_eq!(planner_output.steps[0].tool_name, ToolName::ConfirmAction);
        assert_eq!(planner_output.steps[1].tool_name, ToolName::FocusElement);
        assert_eq!(planner_output.steps[2].tool_name, ToolName::TypeIntoElement);
        assert_eq!(
            planner_output.steps[3].tool_name,
            ToolName::SubmitActiveForm
        );
        assert_eq!(
            planner_output.steps[2].arguments.get("text"),
            Some(&serde_json::json!("phil@example.com"))
        );
        assert_eq!(
            planner_output.steps[3].arguments.get("form_element_id"),
            Some(&serde_json::Value::Null)
        );
        assert!(planner_output.requires_confirmation);
    }

    #[test]
    fn resolve_direct_fill_and_submit_command_reports_missing_value() {
        let page = PageModel {
            title: Some(String::from("Example form")),
            url: Some(String::from("https://example.com/form")),
            regions: Vec::new(),
            interactive_elements: Vec::new(),
        };

        let planner_output = resolve_direct_fill_and_submit_command(
            "fill the email field and submit",
            "req-fill-submit-missing-value",
            Some(&page),
            &[String::from("fill_and_submit_form")],
            0.9,
        )
        .expect("fill-and-submit command should resolve");

        assert_eq!(planner_output.intent.name, IntentName::SubmitForm);
        assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
        assert_eq!(
            planner_output.steps[0].arguments.get("status"),
            Some(&serde_json::json!(ReportStatus::NeedsFollowUp))
        );
    }

    #[test]
    fn resolve_recent_fill_correction_command_reuses_recent_target_for_replacement() {
        let page = PageModel {
            title: Some(String::from("Example form")),
            url: Some(String::from("https://example.com/form")),
            regions: Vec::new(),
            interactive_elements: vec![InteractiveElement {
                element_id: String::from("input-city"),
                dom_locator: Some(String::from("#city")),
                role: ElementRole::Input,
                tag_name: String::from("input"),
                text: None,
                accessible_name: Some(String::from("City")),
                placeholder: Some(String::from("City")),
                href: None,
                value: Some(String::from("Portland")),
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            }],
        };

        let (planner_output, next_context) = resolve_recent_fill_correction_command(
            "put Seattle there instead",
            "req-fill-correction",
            Some("page-1"),
            Some(&page),
            &[String::from("fill_field_by_label")],
            Some(&RecentFieldContext {
                page_id: String::from("page-1"),
                target_description: Some(String::from("city")),
                active_element_id: Some(String::from("input-city")),
                candidate_element_ids: vec![String::from("input-city")],
                pending_text: Some(String::from("Portland")),
                submit_after: false,
            }),
        )
        .expect("follow-up correction should resolve");

        assert_eq!(planner_output.intent.name, IntentName::FillInput);
        assert_eq!(planner_output.status, PlannerStatus::Ready);
        assert_eq!(planner_output.steps[0].tool_name, ToolName::FocusElement);
        assert_eq!(planner_output.steps[1].tool_name, ToolName::TypeIntoElement);
        assert_eq!(
            planner_output.steps[1].arguments.get("text"),
            Some(&serde_json::json!("Seattle"))
        );
        assert_eq!(
            next_context.and_then(|context| context.pending_text),
            Some(String::from("Seattle"))
        );
    }

    #[test]
    fn resolve_recent_fill_correction_command_switches_to_alternate_candidate() {
        let page = PageModel {
            title: Some(String::from("Example form")),
            url: Some(String::from("https://example.com/form")),
            regions: Vec::new(),
            interactive_elements: vec![
                InteractiveElement {
                    element_id: String::from("input-email"),
                    dom_locator: Some(String::from("#email")),
                    role: ElementRole::Input,
                    tag_name: String::from("input"),
                    text: None,
                    accessible_name: Some(String::from("Email")),
                    placeholder: Some(String::from("Email")),
                    href: None,
                    value: None,
                    bbox: None,
                    visible: true,
                    enabled: true,
                    attributes: std::collections::BTreeMap::new(),
                },
                InteractiveElement {
                    element_id: String::from("input-billing-email"),
                    dom_locator: Some(String::from("#billing-email")),
                    role: ElementRole::Input,
                    tag_name: String::from("input"),
                    text: None,
                    accessible_name: Some(String::from("Billing email")),
                    placeholder: Some(String::from("Billing email")),
                    href: None,
                    value: None,
                    bbox: None,
                    visible: true,
                    enabled: true,
                    attributes: std::collections::BTreeMap::new(),
                },
            ],
        };

        let (planner_output, next_context) = resolve_recent_fill_correction_command(
            "no, the other field",
            "req-fill-other-field",
            Some("page-1"),
            Some(&page),
            &[String::from("fill_and_submit_form")],
            Some(&RecentFieldContext {
                page_id: String::from("page-1"),
                target_description: Some(String::from("email")),
                active_element_id: Some(String::from("input-email")),
                candidate_element_ids: vec![
                    String::from("input-email"),
                    String::from("input-billing-email"),
                ],
                pending_text: Some(String::from("phil@example.com")),
                submit_after: true,
            }),
        )
        .expect("alternate-field correction should resolve");

        assert_eq!(planner_output.intent.name, IntentName::SubmitForm);
        assert_eq!(planner_output.status, PlannerStatus::NeedsConfirmation);
        assert_eq!(planner_output.steps[0].tool_name, ToolName::ConfirmAction);
        assert_eq!(planner_output.steps[1].tool_name, ToolName::FocusElement);
        assert_eq!(
            planner_output.steps[1].arguments.get("element_id"),
            Some(&serde_json::json!("input-billing-email"))
        );
        assert_eq!(
            next_context.and_then(|context| context.active_element_id),
            Some(String::from("input-billing-email"))
        );
    }

    #[test]
    fn resolve_recent_fill_correction_command_asks_for_target_without_recent_context() {
        let (planner_output, next_context) = resolve_recent_fill_correction_command(
            "put Seattle there instead",
            "req-fill-no-context",
            None,
            None,
            &[String::from("fill_field_by_label")],
            None,
        )
        .expect("correction phrase should still produce a bounded follow-up");

        assert_eq!(planner_output.intent.name, IntentName::FillInput);
        assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
        assert_eq!(
            planner_output.steps[0].arguments.get("status"),
            Some(&serde_json::json!(ReportStatus::NeedsFollowUp))
        );
        assert!(next_context.is_none());
    }

    #[test]
    fn resolve_typeable_element_rejects_non_field_roles() {
        let page = PageModel {
            title: Some(String::from("Example page")),
            url: Some(String::from("https://example.com")),
            regions: Vec::new(),
            interactive_elements: vec![InteractiveElement {
                element_id: String::from("button-1"),
                dom_locator: Some(String::from("#button-1")),
                role: ElementRole::Button,
                tag_name: String::from("button"),
                text: Some(String::from("Continue")),
                accessible_name: Some(String::from("Continue")),
                placeholder: None,
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            }],
        };

        let error = resolve_typeable_element(&page, "button-1")
            .expect_err("non-field roles should be rejected");
        assert_eq!(error.code, "element_not_editable");
    }

    #[test]
    fn resolve_direct_submit_form_command_builds_confirmation_gated_submit_plan() {
        let page = PageModel {
            title: Some(String::from("Login")),
            url: Some(String::from("https://example.com/login")),
            regions: Vec::new(),
            interactive_elements: vec![InteractiveElement {
                element_id: String::from("form-login"),
                dom_locator: Some(String::from("#login-form")),
                role: ElementRole::Form,
                tag_name: String::from("form"),
                text: Some(String::from("Sign in")),
                accessible_name: Some(String::from("Login")),
                placeholder: None,
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            }],
        };

        let planner_output = resolve_direct_submit_form_command(
            "submit form",
            "req-submit-form",
            Some(&page),
            &[String::from("submit_form")],
        )
        .expect("submit-form command should resolve");

        assert_eq!(planner_output.status, PlannerStatus::NeedsConfirmation);
        assert_eq!(planner_output.intent.name, IntentName::SubmitForm);
        assert_eq!(
            planner_output.selected_skills,
            vec![String::from("submit_form")]
        );
        assert_eq!(planner_output.steps.len(), 2);
        assert_eq!(planner_output.steps[0].tool_name, ToolName::ConfirmAction);
        assert_eq!(
            planner_output.steps[1].tool_name,
            ToolName::SubmitActiveForm
        );
        assert_eq!(
            planner_output.steps[1].arguments.get("form_element_id"),
            Some(&serde_json::json!("form-login"))
        );
        assert!(planner_output.requires_confirmation);
    }

    #[test]
    fn resolve_direct_submit_form_command_reports_ambiguous_forms() {
        let page = PageModel {
            title: Some(String::from("Checkout")),
            url: Some(String::from("https://example.com/checkout")),
            regions: Vec::new(),
            interactive_elements: vec![
                InteractiveElement {
                    element_id: String::from("form-shipping"),
                    dom_locator: Some(String::from("#shipping-form")),
                    role: ElementRole::Form,
                    tag_name: String::from("form"),
                    text: Some(String::from("Shipping")),
                    accessible_name: Some(String::from("Shipping")),
                    placeholder: None,
                    href: None,
                    value: None,
                    bbox: None,
                    visible: true,
                    enabled: true,
                    attributes: std::collections::BTreeMap::new(),
                },
                InteractiveElement {
                    element_id: String::from("form-billing"),
                    dom_locator: Some(String::from("#billing-form")),
                    role: ElementRole::Form,
                    tag_name: String::from("form"),
                    text: Some(String::from("Billing")),
                    accessible_name: Some(String::from("Billing")),
                    placeholder: None,
                    href: None,
                    value: None,
                    bbox: None,
                    visible: true,
                    enabled: true,
                    attributes: std::collections::BTreeMap::new(),
                },
            ],
        };

        let planner_output = resolve_direct_submit_form_command(
            "submit form",
            "req-submit-form-ambiguous",
            Some(&page),
            &[String::from("submit_form")],
        )
        .expect("submit-form command should resolve");

        assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
        assert_eq!(
            planner_output.steps[0].arguments.get("status"),
            Some(&serde_json::json!(ReportStatus::NeedsFollowUp))
        );
    }

    #[test]
    fn app_core_form_regression_fixtures_cover_ambiguous_fill_submit_and_follow_up_cases() {
        let fixtures = vec![
            AppCorePlannerFixture {
                name: "ambiguous-focus-field",
                kind: AppCorePlannerFixtureKind::FocusField,
                transcript: "focus the email field",
                current_page_id: None,
                page: Some(fixture_page(vec![
                    fixture_field("input-email", "#email", "Email", "Email address"),
                    fixture_field(
                        "input-email-confirm",
                        "#email-confirm",
                        "Email confirmation",
                        "Confirm email",
                    ),
                ])),
                active_skills: vec!["focus_field"],
                recent_context: None,
                confirmation_threshold: 0.95,
                expected_intent: IntentName::FillInput,
                expected_status: PlannerStatus::Ready,
                expected_selected_skills: vec!["focus_field"],
                expected_tool_sequence: vec![ToolName::ReportResult],
                expected_focus_element_id: None,
                expected_typed_text: None,
                expected_next_active_element_id: None,
                expected_next_pending_text: None,
            },
            AppCorePlannerFixture {
                name: "fill-field-success",
                kind: AppCorePlannerFixtureKind::FillField,
                transcript: "fill the email field with phil@example.com",
                current_page_id: None,
                page: Some(fixture_page(vec![
                    fixture_field("input-email", "#email", "Email", "Email address"),
                    fixture_field("input-password", "#password", "Password", "Password"),
                ])),
                active_skills: vec!["fill_field_by_label"],
                recent_context: None,
                confirmation_threshold: 0.9,
                expected_intent: IntentName::FillInput,
                expected_status: PlannerStatus::Ready,
                expected_selected_skills: vec!["fill_field_by_label"],
                expected_tool_sequence: vec![ToolName::FocusElement, ToolName::TypeIntoElement],
                expected_focus_element_id: Some("input-email"),
                expected_typed_text: Some("phil@example.com"),
                expected_next_active_element_id: None,
                expected_next_pending_text: None,
            },
            AppCorePlannerFixture {
                name: "fill-and-submit-confirmation",
                kind: AppCorePlannerFixtureKind::FillAndSubmit,
                transcript: "fill the email field with phil@example.com and then submit",
                current_page_id: None,
                page: Some(fixture_page(vec![fixture_field(
                    "input-email",
                    "#email",
                    "Email",
                    "Email address",
                )])),
                active_skills: vec!["fill_and_submit_form"],
                recent_context: None,
                confirmation_threshold: 0.9,
                expected_intent: IntentName::SubmitForm,
                expected_status: PlannerStatus::NeedsConfirmation,
                expected_selected_skills: vec!["fill_and_submit_form"],
                expected_tool_sequence: vec![
                    ToolName::ConfirmAction,
                    ToolName::FocusElement,
                    ToolName::TypeIntoElement,
                    ToolName::SubmitActiveForm,
                ],
                expected_focus_element_id: Some("input-email"),
                expected_typed_text: Some("phil@example.com"),
                expected_next_active_element_id: None,
                expected_next_pending_text: None,
            },
            AppCorePlannerFixture {
                name: "follow-up-replacement",
                kind: AppCorePlannerFixtureKind::FollowUpCorrection,
                transcript: "put Seattle there instead",
                current_page_id: Some("page-1"),
                page: Some(fixture_page(vec![InteractiveElement {
                    element_id: String::from("input-city"),
                    dom_locator: Some(String::from("#city")),
                    role: ElementRole::Input,
                    tag_name: String::from("input"),
                    text: None,
                    accessible_name: Some(String::from("City")),
                    placeholder: Some(String::from("City")),
                    href: None,
                    value: Some(String::from("Portland")),
                    bbox: None,
                    visible: true,
                    enabled: true,
                    attributes: std::collections::BTreeMap::new(),
                }])),
                active_skills: vec!["fill_field_by_label"],
                recent_context: Some(RecentFieldContext {
                    page_id: String::from("page-1"),
                    target_description: Some(String::from("city")),
                    active_element_id: Some(String::from("input-city")),
                    candidate_element_ids: vec![String::from("input-city")],
                    pending_text: Some(String::from("Portland")),
                    submit_after: false,
                }),
                confirmation_threshold: 0.9,
                expected_intent: IntentName::FillInput,
                expected_status: PlannerStatus::Ready,
                expected_selected_skills: vec!["fill_field_by_label"],
                expected_tool_sequence: vec![ToolName::FocusElement, ToolName::TypeIntoElement],
                expected_focus_element_id: Some("input-city"),
                expected_typed_text: Some("Seattle"),
                expected_next_active_element_id: Some("input-city"),
                expected_next_pending_text: Some("Seattle"),
            },
            AppCorePlannerFixture {
                name: "follow-up-other-field",
                kind: AppCorePlannerFixtureKind::FollowUpCorrection,
                transcript: "no, the other field",
                current_page_id: Some("page-1"),
                page: Some(fixture_page(vec![
                    fixture_field("input-email", "#email", "Email", "Email"),
                    fixture_field(
                        "input-billing-email",
                        "#billing-email",
                        "Billing email",
                        "Billing email",
                    ),
                ])),
                active_skills: vec!["fill_and_submit_form"],
                recent_context: Some(RecentFieldContext {
                    page_id: String::from("page-1"),
                    target_description: Some(String::from("email")),
                    active_element_id: Some(String::from("input-email")),
                    candidate_element_ids: vec![
                        String::from("input-email"),
                        String::from("input-billing-email"),
                    ],
                    pending_text: Some(String::from("phil@example.com")),
                    submit_after: true,
                }),
                confirmation_threshold: 0.9,
                expected_intent: IntentName::SubmitForm,
                expected_status: PlannerStatus::NeedsConfirmation,
                expected_selected_skills: vec!["fill_and_submit_form"],
                expected_tool_sequence: vec![
                    ToolName::ConfirmAction,
                    ToolName::FocusElement,
                    ToolName::TypeIntoElement,
                    ToolName::SubmitActiveForm,
                ],
                expected_focus_element_id: Some("input-billing-email"),
                expected_typed_text: Some("phil@example.com"),
                expected_next_active_element_id: Some("input-billing-email"),
                expected_next_pending_text: Some("phil@example.com"),
            },
            AppCorePlannerFixture {
                name: "ambiguous-submit-form",
                kind: AppCorePlannerFixtureKind::SubmitForm,
                transcript: "submit form",
                current_page_id: None,
                page: Some(fixture_page(vec![
                    fixture_form("form-shipping", "#shipping-form", "Shipping"),
                    fixture_form("form-billing", "#billing-form", "Billing"),
                ])),
                active_skills: vec!["submit_form"],
                recent_context: None,
                confirmation_threshold: 0.9,
                expected_intent: IntentName::SubmitForm,
                expected_status: PlannerStatus::Ready,
                expected_selected_skills: vec!["submit_form"],
                expected_tool_sequence: vec![ToolName::ReportResult],
                expected_focus_element_id: None,
                expected_typed_text: None,
                expected_next_active_element_id: None,
                expected_next_pending_text: None,
            },
        ];

        for fixture in fixtures {
            assert_app_core_planner_fixture(fixture);
        }
    }

    #[test]
    fn ambiguous_click_regression_fixtures_pin_confirmation_threshold_behavior() {
        struct AmbiguousClickFixture {
            name: &'static str,
            candidates: Vec<crate::commands::ElementCandidate>,
            confirmation_threshold: f32,
            expected_element_id: Option<&'static str>,
            expected_confidence: Option<f32>,
            expected_requires_confirmation: bool,
        }

        let fixtures = vec![
            AmbiguousClickFixture {
                name: "close-candidates-trigger-follow-up",
                candidates: vec![
                    crate::commands::ElementCandidate {
                        element_id: String::from("button-1"),
                        confidence_bps: 8_900,
                        matched_on: vec![String::from("description")],
                        rationale_codes: vec![String::from("accessible_name_exact")],
                    },
                    crate::commands::ElementCandidate {
                        element_id: String::from("button-2"),
                        confidence_bps: 8_400,
                        matched_on: vec![String::from("description")],
                        rationale_codes: vec![String::from("accessible_name_exact")],
                    },
                ],
                confirmation_threshold: 0.9,
                expected_element_id: None,
                expected_confidence: Some(0.89),
                expected_requires_confirmation: true,
            },
            AmbiguousClickFixture {
                name: "threshold-crossing-allows-direct-click",
                candidates: vec![crate::commands::ElementCandidate {
                    element_id: String::from("link-help"),
                    confidence_bps: 8_800,
                    matched_on: vec![String::from("accessible_name")],
                    rationale_codes: vec![String::from("accessible_name_exact")],
                }],
                confirmation_threshold: 0.85,
                expected_element_id: Some("link-help"),
                expected_confidence: Some(0.88),
                expected_requires_confirmation: false,
            },
        ];

        for fixture in fixtures {
            let (chosen_element_id, chosen_confidence, requires_confirmation) =
                determine_find_element_resolution(
                    &fixture.candidates,
                    fixture.confirmation_threshold,
                );

            assert_eq!(
                chosen_element_id.as_deref(),
                fixture.expected_element_id,
                "fixture {} chose the wrong element",
                fixture.name
            );
            assert_eq!(
                chosen_confidence, fixture.expected_confidence,
                "fixture {} produced unexpected confidence",
                fixture.name
            );
            assert_eq!(
                requires_confirmation, fixture.expected_requires_confirmation,
                "fixture {} produced unexpected confirmation behavior",
                fixture.name
            );
        }
    }

    #[test]
    fn problematic_page_regression_fixtures_cover_checkout_and_duplicate_cta_shapes() {
        let checkout_page = fixture_problematic_checkout_page();
        let newsletter_page = fixture_problematic_newsletter_page();
        let fixtures = vec![
            AppCorePlannerFixture {
                name: "problematic-checkout-ambiguous-email-focus",
                kind: AppCorePlannerFixtureKind::FocusField,
                transcript: "focus the email field",
                current_page_id: None,
                page: Some(checkout_page.clone()),
                active_skills: vec!["focus_field"],
                recent_context: None,
                confirmation_threshold: 0.95,
                expected_intent: IntentName::FillInput,
                expected_status: PlannerStatus::Ready,
                expected_selected_skills: vec!["focus_field"],
                expected_tool_sequence: vec![ToolName::ReportResult],
                expected_focus_element_id: None,
                expected_typed_text: None,
                expected_next_active_element_id: None,
                expected_next_pending_text: None,
            },
            AppCorePlannerFixture {
                name: "problematic-newsletter-fill-email",
                kind: AppCorePlannerFixtureKind::FillField,
                transcript: "fill the email field with phil@example.com",
                current_page_id: None,
                page: Some(newsletter_page),
                active_skills: vec!["fill_field_by_label"],
                recent_context: None,
                confirmation_threshold: 0.9,
                expected_intent: IntentName::FillInput,
                expected_status: PlannerStatus::Ready,
                expected_selected_skills: vec!["fill_field_by_label"],
                expected_tool_sequence: vec![ToolName::FocusElement, ToolName::TypeIntoElement],
                expected_focus_element_id: Some("input-newsletter-email"),
                expected_typed_text: Some("phil@example.com"),
                expected_next_active_element_id: None,
                expected_next_pending_text: None,
            },
            AppCorePlannerFixture {
                name: "problematic-checkout-other-field-correction",
                kind: AppCorePlannerFixtureKind::FollowUpCorrection,
                transcript: "no, the other field",
                current_page_id: Some("checkout-page"),
                page: Some(checkout_page.clone()),
                active_skills: vec!["fill_and_submit_form"],
                recent_context: Some(RecentFieldContext {
                    page_id: String::from("checkout-page"),
                    target_description: Some(String::from("email")),
                    active_element_id: Some(String::from("input-shipping-email")),
                    candidate_element_ids: vec![
                        String::from("input-shipping-email"),
                        String::from("input-billing-email"),
                    ],
                    pending_text: Some(String::from("phil@example.com")),
                    submit_after: true,
                }),
                confirmation_threshold: 0.9,
                expected_intent: IntentName::SubmitForm,
                expected_status: PlannerStatus::NeedsConfirmation,
                expected_selected_skills: vec!["fill_and_submit_form"],
                expected_tool_sequence: vec![
                    ToolName::ConfirmAction,
                    ToolName::FocusElement,
                    ToolName::TypeIntoElement,
                    ToolName::SubmitActiveForm,
                ],
                expected_focus_element_id: Some("input-billing-email"),
                expected_typed_text: Some("phil@example.com"),
                expected_next_active_element_id: Some("input-billing-email"),
                expected_next_pending_text: Some("phil@example.com"),
            },
            AppCorePlannerFixture {
                name: "problematic-checkout-ambiguous-submit",
                kind: AppCorePlannerFixtureKind::SubmitForm,
                transcript: "submit form",
                current_page_id: None,
                page: Some(checkout_page),
                active_skills: vec!["submit_form"],
                recent_context: None,
                confirmation_threshold: 0.9,
                expected_intent: IntentName::SubmitForm,
                expected_status: PlannerStatus::Ready,
                expected_selected_skills: vec!["submit_form"],
                expected_tool_sequence: vec![ToolName::ReportResult],
                expected_focus_element_id: None,
                expected_typed_text: None,
                expected_next_active_element_id: None,
                expected_next_pending_text: None,
            },
        ];

        for fixture in fixtures {
            assert_app_core_planner_fixture(fixture);
        }

        let landing_page = fixture_problematic_landing_page();
        let query = build_find_element_query(&FindElementInput {
            request_id: String::from("req-problematic-cta"),
            timeout_ms: None,
            description: String::from("Get started"),
            text: None,
            role: Some(ElementRole::Button),
            color_hint: None,
            nearby_text: None,
            selector_hint: None,
            visibility_filter: crate::commands::ElementVisibilityFilter::VisibleOnly,
            max_candidates: Some(3),
        })
        .expect("landing-page query should be valid");
        let candidates =
            rank_find_element_candidates(&landing_page.interactive_elements, &query, 3);
        let (chosen_element_id, _, requires_confirmation) =
            determine_find_element_resolution(&candidates, 0.9);

        assert_eq!(candidates.len(), 2);
        assert_eq!(chosen_element_id, None);
        assert!(requires_confirmation);
    }

    #[test]
    fn resolve_form_element_rejects_non_form_roles() {
        let page = PageModel {
            title: Some(String::from("Example page")),
            url: Some(String::from("https://example.com")),
            regions: Vec::new(),
            interactive_elements: vec![InteractiveElement {
                element_id: String::from("button-1"),
                dom_locator: Some(String::from("#button-1")),
                role: ElementRole::Button,
                tag_name: String::from("button"),
                text: Some(String::from("Submit")),
                accessible_name: Some(String::from("Submit")),
                placeholder: None,
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            }],
        };

        let error =
            resolve_form_element(&page, "button-1").expect_err("non-form roles should be rejected");
        assert_eq!(error.code, "element_not_form");
    }

    #[test]
    fn rank_find_element_candidates_prefers_exact_accessible_name_matches() {
        let elements = vec![
            InteractiveElement {
                element_id: String::from("button-1"),
                dom_locator: Some(String::from("#button-1")),
                role: ElementRole::Button,
                tag_name: String::from("button"),
                text: Some(String::from("Continue")),
                accessible_name: Some(String::from("Continue")),
                placeholder: None,
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
            InteractiveElement {
                element_id: String::from("button-2"),
                dom_locator: Some(String::from("#button-2")),
                role: ElementRole::Button,
                tag_name: String::from("button"),
                text: Some(String::from("Continue reading")),
                accessible_name: Some(String::from("Continue reading")),
                placeholder: None,
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
        ];
        let query = build_find_element_query(&FindElementInput {
            request_id: String::from("req-find"),
            timeout_ms: None,
            description: String::from("Continue"),
            text: None,
            role: Some(ElementRole::Button),
            color_hint: None,
            nearby_text: None,
            selector_hint: None,
            visibility_filter: crate::commands::ElementVisibilityFilter::VisibleOnly,
            max_candidates: Some(3),
        })
        .expect("query should be valid");

        let candidates = rank_find_element_candidates(&elements, &query, 3);

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].element_id, "button-1");
        assert!(candidates[0].confidence_bps > candidates[1].confidence_bps);
    }

    #[test]
    fn rank_find_element_candidates_uses_selector_hint_and_respects_candidate_limit() {
        let elements = vec![
            InteractiveElement {
                element_id: String::from("button-primary"),
                dom_locator: Some(String::from("#checkout-submit")),
                role: ElementRole::Button,
                tag_name: String::from("button"),
                text: Some(String::from("Continue")),
                accessible_name: Some(String::from("Continue")),
                placeholder: None,
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::from([
                    (String::from("data-testid"), String::from("checkout-submit")),
                    (String::from("class"), String::from("cta primary")),
                ]),
            },
            InteractiveElement {
                element_id: String::from("button-secondary"),
                dom_locator: Some(String::from("#continue-reading")),
                role: ElementRole::Button,
                tag_name: String::from("button"),
                text: Some(String::from("Continue")),
                accessible_name: Some(String::from("Continue")),
                placeholder: None,
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::from([(
                    String::from("data-testid"),
                    String::from("continue-reading"),
                )]),
            },
            InteractiveElement {
                element_id: String::from("button-tertiary"),
                dom_locator: Some(String::from("#continue-later")),
                role: ElementRole::Button,
                tag_name: String::from("button"),
                text: Some(String::from("Continue later")),
                accessible_name: Some(String::from("Continue later")),
                placeholder: None,
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::from([(
                    String::from("data-testid"),
                    String::from("continue-later"),
                )]),
            },
        ];
        let query = build_find_element_query(&FindElementInput {
            request_id: String::from("req-find"),
            timeout_ms: None,
            description: String::from("Continue"),
            text: None,
            role: Some(ElementRole::Button),
            color_hint: None,
            nearby_text: None,
            selector_hint: Some(String::from("checkout-submit")),
            visibility_filter: crate::commands::ElementVisibilityFilter::VisibleOnly,
            max_candidates: Some(2),
        })
        .expect("query should be valid");

        let candidates = rank_find_element_candidates(&elements, &query, 2);

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].element_id, "button-primary");
        assert!(candidates[0]
            .matched_on
            .iter()
            .any(|matched_on| matched_on == "selector_hint"));
        assert!(candidates[0].confidence_bps > candidates[1].confidence_bps);
        assert!(!candidates
            .iter()
            .any(|candidate| candidate.element_id == "button-tertiary"));
    }

    #[test]
    fn build_find_element_query_normalizes_optional_hints_into_summary() {
        let query = build_find_element_query(&FindElementInput {
            request_id: String::from("req-find"),
            timeout_ms: None,
            description: String::from("  Continue  "),
            text: Some(String::from("  Start now  ")),
            role: Some(ElementRole::Button),
            color_hint: Some(String::from("  primary blue  ")),
            nearby_text: Some(String::from("  pricing  ")),
            selector_hint: Some(String::from("  cta-primary  ")),
            visibility_filter: crate::commands::ElementVisibilityFilter::VisibleOnly,
            max_candidates: Some(3),
        })
        .expect("query should be valid");

        assert_eq!(query.description.as_deref(), Some("Continue"));
        assert_eq!(query.text.as_deref(), Some("Start now"));
        assert_eq!(query.color_hint.as_deref(), Some("primary blue"));
        assert_eq!(query.nearby_text.as_deref(), Some("pricing"));
        assert_eq!(query.selector_hint.as_deref(), Some("cta-primary"));
        assert_eq!(
            query.summary,
            "description=Continue; text=Start now; role=Button; color_hint=primary blue; nearby_text=pricing; selector_hint=cta-primary"
        );
    }

    #[test]
    fn determine_find_element_resolution_flags_close_candidates_for_confirmation() {
        let candidates = vec![
            crate::commands::ElementCandidate {
                element_id: String::from("button-1"),
                confidence_bps: 8_900,
                matched_on: vec![String::from("description")],
                rationale_codes: vec![String::from("accessible_name_exact")],
            },
            crate::commands::ElementCandidate {
                element_id: String::from("button-2"),
                confidence_bps: 8_400,
                matched_on: vec![String::from("description")],
                rationale_codes: vec![String::from("accessible_name_exact")],
            },
        ];

        let (chosen_element_id, chosen_confidence, requires_confirmation) =
            determine_find_element_resolution(&candidates, 0.9);

        assert_eq!(chosen_element_id, None);
        assert_eq!(chosen_confidence, Some(0.89));
        assert!(requires_confirmation);
    }

    #[test]
    fn determine_find_element_resolution_uses_configured_confidence_threshold() {
        let candidates = vec![crate::commands::ElementCandidate {
            element_id: String::from("link-help"),
            confidence_bps: 8_800,
            matched_on: vec![String::from("accessible_name")],
            rationale_codes: vec![String::from("accessible_name_exact")],
        }];

        let (chosen_element_id, chosen_confidence, requires_confirmation) =
            determine_find_element_resolution(&candidates, 0.9);
        assert_eq!(chosen_element_id, None);
        assert_eq!(chosen_confidence, Some(0.88));
        assert!(requires_confirmation);

        let (chosen_element_id, chosen_confidence, requires_confirmation) =
            determine_find_element_resolution(&candidates, 0.85);
        assert_eq!(chosen_element_id, Some(String::from("link-help")));
        assert_eq!(chosen_confidence, Some(0.88));
        assert!(!requires_confirmation);
    }

    #[test]
    fn planner_system_prompt_mentions_click_confirmation_config() {
        let prompt = planner_system_prompt();

        assert!(prompt.contains("planner_input.safety.allow_click_without_confirmation"));
        assert!(prompt.contains("ordinary ClickElement plans may use Ready"));
        assert!(prompt.contains("planner_input.safety.confirmation_confidence_threshold"));
    }

    struct MockReplanningRuntime {
        resolve_results: Vec<Result<PlannerOutput, crate::commands::ToolError>>,
        execute_results: Vec<ExecutionOutcome>,
        resolve_recent_tool_results: Vec<Vec<PlannerToolHistoryEntry>>,
        execute_request_ids: Vec<String>,
    }

    impl ReplanningRuntime for MockReplanningRuntime {
        fn resolve_plan(
            &mut self,
            _request_id: String,
            _transcript: &str,
            recent_tool_results: &[PlannerToolHistoryEntry],
        ) -> Result<PlannerOutput, crate::commands::ToolError> {
            self.resolve_recent_tool_results
                .push(recent_tool_results.to_vec());
            self.resolve_results.remove(0)
        }

        fn execute_plan(
            &mut self,
            request_id: String,
            _planner_output: &PlannerOutput,
        ) -> ExecutionOutcome {
            self.execute_request_ids.push(request_id);
            self.execute_results.remove(0)
        }
    }

    fn mock_planner_output(step_id: &str) -> PlannerOutput {
        PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::GetStatus,
                goal: String::from("report runtime status"),
                target_description: None,
            },
            selected_skills: vec![String::from("get_status")],
            steps: vec![PlannedStep {
                step_id: step_id.to_string(),
                tool_name: ToolName::GetRuntimeStatus,
                arguments: serde_json::json!({
                    "request_id": format!("req-{step_id}"),
                    "timeout_ms": null,
                    "include_provider_modes": false
                }),
                purpose: String::from("read runtime status"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        }
    }

    fn mock_trace(step_id: &str, tool_name: ToolName, observation: &str) -> ExecutionTrace {
        ExecutionTrace {
            executed_step_ids: vec![step_id.to_string()],
            tool_results: vec![ToolResult::success(
                tool_name,
                format!("req-{step_id}"),
                serde_json::json!({}),
                vec![observation.to_string()],
            )],
        }
    }

    #[test]
    fn bounded_replanning_loop_replans_once_with_recent_tool_history() {
        let mut runtime = MockReplanningRuntime {
            resolve_results: vec![
                Ok(mock_planner_output("step-1")),
                Ok(mock_planner_output("step-2")),
            ],
            execute_results: vec![
                ExecutionOutcome::NeedsReplan {
                    trace: mock_trace("step-1", ToolName::GetRuntimeStatus, "first plan failed"),
                },
                ExecutionOutcome::Complete {
                    trace: mock_trace("step-2", ToolName::ReportResult, "second plan succeeded"),
                },
            ],
            resolve_recent_tool_results: Vec::new(),
            execute_request_ids: Vec::new(),
        };

        let outcome = execute_bounded_replanning_loop(&mut runtime, "req", "what is the status")
            .expect("bounded replanning should succeed");

        match outcome {
            ExecutionOutcome::Complete { trace } => {
                assert_eq!(trace.executed_step_ids, vec!["step-1", "step-2"]);
                assert_eq!(trace.tool_results.len(), 2);
            }
            other => panic!("expected complete outcome, got {other:?}"),
        }

        assert_eq!(runtime.resolve_recent_tool_results.len(), 2);
        assert!(runtime.resolve_recent_tool_results[0].is_empty());
        assert_eq!(runtime.resolve_recent_tool_results[1].len(), 1);
        assert_eq!(
            runtime.resolve_recent_tool_results[1][0].observation_summary,
            vec![String::from("first plan failed")]
        );
        assert_eq!(
            runtime.execute_request_ids,
            vec![
                String::from("req-execute"),
                String::from("req-execute-replan-1")
            ]
        );
    }

    #[test]
    fn bounded_replanning_loop_stops_after_replan_limit() {
        let mut runtime = MockReplanningRuntime {
            resolve_results: vec![
                Ok(mock_planner_output("step-1")),
                Ok(mock_planner_output("step-2")),
            ],
            execute_results: vec![
                ExecutionOutcome::NeedsReplan {
                    trace: mock_trace(
                        "step-1",
                        ToolName::GetRuntimeStatus,
                        "first replan requested",
                    ),
                },
                ExecutionOutcome::NeedsReplan {
                    trace: mock_trace(
                        "step-2",
                        ToolName::GetRuntimeStatus,
                        "second replan requested",
                    ),
                },
            ],
            resolve_recent_tool_results: Vec::new(),
            execute_request_ids: Vec::new(),
        };

        let outcome = execute_bounded_replanning_loop(&mut runtime, "req", "what is the status")
            .expect("bounded replanning should return an execution outcome");

        match outcome {
            ExecutionOutcome::Aborted { trace, error } => {
                assert_eq!(error.code, "replan_limit_exceeded");
                assert_eq!(trace.executed_step_ids, vec!["step-1", "step-2"]);
                assert_eq!(trace.tool_results.len(), 2);
            }
            other => panic!("expected aborted outcome, got {other:?}"),
        }
    }

    #[test]
    fn bounded_replanning_loop_aborts_with_accumulated_trace_when_follow_up_resolution_fails() {
        let mut runtime = MockReplanningRuntime {
            resolve_results: vec![
                Ok(mock_planner_output("step-1")),
                Err(crate::commands::ToolError {
                    code: String::from("planner_backend_unavailable"),
                    message: String::from("planner could not resolve a follow-up plan"),
                    retryable: true,
                    details: Some(serde_json::json!({
                        "attempt": 2
                    })),
                }),
            ],
            execute_results: vec![ExecutionOutcome::NeedsReplan {
                trace: mock_trace("step-1", ToolName::GetRuntimeStatus, "first plan failed"),
            }],
            resolve_recent_tool_results: Vec::new(),
            execute_request_ids: Vec::new(),
        };

        let outcome = execute_bounded_replanning_loop(&mut runtime, "req", "what is the status")
            .expect("bounded replanning should surface an aborted execution outcome");

        match outcome {
            ExecutionOutcome::Aborted { trace, error } => {
                assert_eq!(error.code, "planner_backend_unavailable");
                assert_eq!(trace.executed_step_ids, vec![String::from("step-1")]);
                assert_eq!(trace.tool_results.len(), 1);
                assert_eq!(
                    trace.tool_results[0].observations,
                    vec![String::from("first plan failed")]
                );
            }
            other => panic!("expected aborted outcome, got {other:?}"),
        }

        assert_eq!(runtime.resolve_recent_tool_results.len(), 2);
        assert!(runtime.resolve_recent_tool_results[0].is_empty());
        assert_eq!(runtime.resolve_recent_tool_results[1].len(), 1);
        assert_eq!(
            runtime.execute_request_ids,
            vec![String::from("req-execute")]
        );
    }

    #[test]
    fn resolve_clickable_element_requires_an_enabled_visible_exact_match() {
        let page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: Vec::new(),
            interactive_elements: vec![InteractiveElement {
                element_id: String::from("button-disabled"),
                dom_locator: Some(String::from("#button-disabled")),
                role: ElementRole::Button,
                tag_name: String::from("button"),
                text: Some(String::from("Continue")),
                accessible_name: Some(String::from("Continue")),
                placeholder: None,
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: false,
                attributes: std::collections::BTreeMap::new(),
            }],
        };

        let error = resolve_clickable_element(&page, "button-disabled").unwrap_err();

        assert_eq!(error.code, "element_disabled");
    }

    #[test]
    fn resolve_clickable_element_requires_a_stable_dom_locator() {
        let page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: Vec::new(),
            interactive_elements: vec![InteractiveElement {
                element_id: String::from("button-1"),
                dom_locator: None,
                role: ElementRole::Button,
                tag_name: String::from("button"),
                text: Some(String::from("Continue")),
                accessible_name: Some(String::from("Continue")),
                placeholder: None,
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            }],
        };

        let error = resolve_clickable_element(&page, "button-1").unwrap_err();

        assert_eq!(error.code, "missing_dom_locator");
    }

    #[test]
    fn resolve_clickable_element_rejects_blank_and_unknown_ids() {
        let page = fixture_page(vec![InteractiveElement {
            element_id: String::from("button-1"),
            dom_locator: Some(String::from("#button-1")),
            role: ElementRole::Button,
            tag_name: String::from("button"),
            text: Some(String::from("Continue")),
            accessible_name: Some(String::from("Continue")),
            placeholder: None,
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::new(),
        }]);

        let blank_error = resolve_clickable_element(&page, "   ").unwrap_err();
        assert_eq!(blank_error.code, "invalid_element_id");

        let unknown_error = resolve_clickable_element(&page, "missing-button").unwrap_err();
        assert_eq!(unknown_error.code, "unknown_element_id");
        assert_eq!(
            unknown_error.details,
            Some(serde_json::json!({ "element_id": "missing-button" }))
        );
    }
}
