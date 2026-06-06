#[cfg(feature = "remote-openai")]
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asr::AsrController;
use crate::audio_io::AudioPlaybackController;
use crate::browser::{
    BrowserController, BrowserError, BrowserPageMetrics, BrowserSessionConfig, BrowserVisibilityMode,
};
use crate::commands::{
    build_planner_skill_selection, execute_planner_output, planner_available_tools,
    resolve_direct_audio_command, resolve_direct_browser_visibility_command,
    resolve_direct_navigation_readback_command, resolve_direct_open_url_command,
    resolve_direct_read_page_command, resolve_direct_read_title_command,
    resolve_direct_repeat_command, resolve_direct_status_query_command,
    resolve_direct_voice_input_command, resume_after_confirmation, validate_planner_output,
    AgentStateData, AsrProviderSettings, ConfirmActionData, ConfirmActionInput,
    ConfirmActionResolution, ConfirmationSettings, ExecutionOutcome, GetAgentStateInput,
    GetRuntimeStatusData, GetRuntimeStatusInput, LocalAsrModelSettings, LocalTtsModelSettings,
    OcrThresholdSettings, PageSnapshotData, PlannerInput, PlannerOutput, PlannerToolHistoryEntry,
    ProviderFailoverSettings, ProviderSelectionStatus, RemoteAsrSettings,
    RemotePlannerSettings, RemoteTtsSettings, ReportResultData, ReportResultInput, ToolError,
    ToolName, ToolResult, TtsModelSettings, TtsProviderSettings, TtsVoiceSettings,
};
#[cfg(feature = "remote-openai")]
use crate::commands::{
    canonical_planner_output_examples, planner_output_schema, tool_input_schema,
};
#[cfg(feature = "remote-openai")]
use crate::config::resolve_secret_ref;
use crate::config::{
    AppConfig, AudioSettings, ConfigError, ModelManagementSettings, RemotePlannerProfile,
    RemoteProviderKind,
};
use crate::narration::{cursor_for_index, find_region_index, spoken_text_for_region};
use crate::ocr::OcrController;
use crate::page_model::PageRegion;
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
use api_key_tools::{
    fetch_openai_compatible_models, test_remote_openai_profile_api_key, RemoteApiKeyTarget,
    RemoteOpenAiApiKeyTestProfile,
};

mod content_tools;

mod extraction_tools;

mod form_fill;
use form_fill::{
    PendingRecentFieldContext, RecentFieldContext,
    resolve_direct_fill_command_internal, resolve_direct_focus_field_command,
    resolve_direct_submit_form_command, resolve_recent_fill_correction_command,
};
#[cfg(test)]
use form_fill::{resolve_direct_fill_and_submit_command, resolve_direct_fill_field_command};

mod interaction_tools;
use interaction_tools::{normalize_optional_text, region_bbox_by_id};
use extraction_tools::build_visible_text_excerpt;

mod voice_tools;
use voice_tools::{audio_playback_error_to_tool_error, tts_runtime_error_to_tool_error};

mod planner_prompt;
use planner_prompt::planner_interpretation_unavailable_error;
#[cfg(any(feature = "remote-openai", test))]
use planner_prompt::planner_system_prompt;
#[cfg(feature = "remote-openai")]
use planner_prompt::PlannerPromptPayload;

mod navigation_tools;
use navigation_tools::browser_error_to_tool_error;

mod model_management;
use model_management::{
    download_hugging_face_directory, download_hugging_face_file, kitten_download_plan_for_model_id,
    resolved_models_dir_for_app, whisper_download_plan_for_model_id,
};

mod replanning;
use replanning::execute_bounded_replanning_loop;

mod tool_executor;

mod settings_adapters;
use settings_adapters::{
    active_local_asr_profile, active_local_tts_profile, build_asr_provider_settings,
    build_confirmation_settings, build_local_asr_model_settings, build_local_tts_model_settings,
    build_model_management_settings, build_ocr_threshold_settings, build_provider_failover_settings,
    build_remote_asr_settings, build_remote_planner_settings, build_remote_tts_settings,
    build_tts_model_settings, build_tts_provider_settings, build_tts_voice_settings,
};

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

    pub fn set_remote_planner_connection_settings(
        &mut self,
        profile_name: &str,
        base_url: &str,
        model: &str,
    ) -> Result<(), ConfigError> {
        self.config = AppConfig::persist_remote_planner_connection_settings_for_app(
            &self.app_handle,
            profile_name,
            base_url,
            model,
        )?;
        Ok(())
    }

    pub fn reset_remote_planner_connection_settings_to_defaults(
        &mut self,
        profile_name: &str,
    ) -> Result<(), ConfigError> {
        self.config = AppConfig::reset_remote_planner_connection_settings_to_defaults_for_app(
            &self.app_handle,
            profile_name,
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

    pub fn test_remote_planner_api_key(
        &self,
        profile_name: &str,
        api_key_override: Option<&str>,
        timeout_ms_override: Option<u64>,
    ) -> Result<String, String> {
        let profile = self
            .config
            .remote_planner_profiles
            .get(profile_name)
            .ok_or_else(|| format!("unknown remote planner profile '{profile_name}'"))?;
        test_remote_openai_profile_api_key(
            RemoteApiKeyTarget::Planner,
            RemoteOpenAiApiKeyTestProfile {
                profile_name,
                provider: &profile.provider,
                base_url: &profile.base_url,
                configured_api_key: &profile.api_key,
                organization: profile.organization.as_ref(),
                project: profile.project.as_deref(),
                timeout_ms: timeout_ms_override.unwrap_or(profile.timeout_ms),
            },
            api_key_override,
        )
    }

    pub fn list_remote_planner_models(
        &self,
        profile_name: &str,
        base_url_override: Option<&str>,
        api_key_override: Option<&str>,
        timeout_ms_override: Option<u64>,
    ) -> Result<Vec<String>, String> {
        let profile = self
            .config
            .remote_planner_profiles
            .get(profile_name)
            .ok_or_else(|| format!("unknown remote planner profile '{profile_name}'"))?;

        let api_key = match api_key_override.map(str::trim) {
            Some(override_value) if !override_value.is_empty() => Some(override_value.to_string()),
            _ => resolve_secret_ref(&profile.api_key).ok(),
        };
        let organization = profile
            .organization
            .as_ref()
            .map(resolve_secret_ref)
            .transpose()
            .map_err(|reason| {
                format!(
                    "Remote planner model list could not read the configured organization secret: {reason}"
                )
            })?;

        fetch_openai_compatible_models(
            base_url_override.unwrap_or(&profile.base_url),
            api_key.as_deref(),
            organization.as_deref(),
            profile.project.as_deref(),
            timeout_ms_override.unwrap_or(profile.timeout_ms),
        )
    }

    pub fn test_remote_tts_api_key(
        &self,
        profile_name: &str,
        api_key_override: Option<&str>,
        timeout_ms_override: Option<u64>,
    ) -> Result<String, String> {
        let profile = self
            .config
            .remote_tts_profiles
            .get(profile_name)
            .ok_or_else(|| format!("unknown remote TTS profile '{profile_name}'"))?;
        test_remote_openai_profile_api_key(
            RemoteApiKeyTarget::Tts,
            RemoteOpenAiApiKeyTestProfile {
                profile_name,
                provider: &profile.provider,
                base_url: &profile.base_url,
                configured_api_key: &profile.api_key,
                organization: profile.organization.as_ref(),
                project: profile.project.as_deref(),
                timeout_ms: timeout_ms_override.unwrap_or(profile.timeout_ms),
            },
            api_key_override,
        )
    }

    pub fn test_remote_asr_api_key(
        &self,
        profile_name: &str,
        api_key_override: Option<&str>,
        timeout_ms_override: Option<u64>,
    ) -> Result<String, String> {
        let profile = self
            .config
            .remote_asr_profiles
            .get(profile_name)
            .ok_or_else(|| format!("unknown remote ASR profile '{profile_name}'"))?;
        test_remote_openai_profile_api_key(
            RemoteApiKeyTarget::Asr,
            RemoteOpenAiApiKeyTestProfile {
                profile_name,
                provider: &profile.provider,
                base_url: &profile.base_url,
                configured_api_key: &profile.api_key,
                organization: profile.organization.as_ref(),
                project: profile.project.as_deref(),
                timeout_ms: timeout_ms_override.unwrap_or(profile.timeout_ms),
            },
            api_key_override,
        )
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


#[cfg(test)]
mod tests;
