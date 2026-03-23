use std::time::{SystemTime, UNIX_EPOCH};

use tauri::AppHandle;

use crate::browser::BrowserVisibilityMode;
use crate::commands::{
    execute_planner_output, resume_after_confirmation, AgentStateData, ConfirmActionData,
    ConfirmActionInput, ConfirmActionResolution, DeterministicToolExecutor, ExecutionOutcome,
    GetAgentStateInput, GetRuntimeStatusData, GetRuntimeStatusInput, PlannerOutput,
    ProviderSelectionStatus, SetBrowserVisibilityData, SetBrowserVisibilityInput,
    SetPlaybackSpeedData, SetPlaybackSpeedInput, SetPlaybackVolumeData, SetPlaybackVolumeInput,
    SetTtsVoiceData, SetTtsVoiceInput, ToolError, ToolName, ToolResult,
};
use crate::config::{AppConfig, AudioSettings, ConfigError};
use crate::state::AppState;

#[derive(Clone)]
pub struct AppCore {
    pub app_handle: AppHandle,
    pub config: AppConfig,
    pub state: AppState,
}

impl AppCore {
    pub fn new(app_handle: AppHandle) -> Result<Self, ConfigError> {
        let config = AppConfig::load_for_app(&app_handle)?;
        let state = AppState::from_config(&config);

        Ok(Self {
            app_handle,
            config,
            state,
        })
    }

    pub fn update_audio_settings(&mut self, audio: AudioSettings) -> Result<(), ConfigError> {
        let config = AppConfig::persist_audio_settings_for_app(&self.app_handle, &audio)?;
        self.state.apply_audio_settings(&config.audio);
        self.config = config;
        Ok(())
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
        let voice = input.voice.trim().to_string();
        if voice.is_empty() {
            return ToolResult::failure(
                ToolName::SetTtsVoice,
                input.request_id,
                ToolError {
                    code: String::from("invalid_voice"),
                    message: String::from(
                        "voice must be a non-empty provider-supported voice name",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Voice update rejected because the voice name was empty.",
                )],
            );
        }

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
        let changed = self.state.browser_visibility != input.mode;
        self.set_browser_visibility(input.mode);

        ToolResult::success(
            ToolName::SetBrowserVisibility,
            input.request_id,
            SetBrowserVisibilityData {
                mode: self.state.browser_visibility,
                changed,
                supported: true,
            },
            vec![String::from("Updated the browser visibility mode.")],
        )
    }

    pub fn execute_get_agent_state(
        &mut self,
        input: GetAgentStateInput,
    ) -> ToolResult<AgentStateData> {
        ToolResult::success(
            ToolName::GetAgentState,
            input.request_id,
            self.current_agent_state(input.include_last_transcript),
            vec![String::from("Read the current agent state.")],
        )
    }

    pub fn execute_get_runtime_status(
        &mut self,
        input: GetRuntimeStatusInput,
    ) -> ToolResult<GetRuntimeStatusData> {
        ToolResult::success(
            ToolName::GetRuntimeStatus,
            input.request_id,
            self.current_runtime_status(input.include_provider_modes),
            vec![String::from("Read the current runtime status.")],
        )
    }

    fn current_agent_state(&self, include_last_transcript: bool) -> AgentStateData {
        AgentStateData {
            page_id: None,
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
            speaking: false,
            listening_state: self.state.listening.clone(),
            audio: self.state.audio.clone(),
            last_transcript: if include_last_transcript {
                Some(String::new())
            } else {
                None
            },
            last_action: None,
            pending_confirmation_id: self.state.pending_confirmation_id.clone(),
            pending_plan_execution: self.state.pending_plan_execution.clone(),
        }
    }

    fn current_runtime_status(&self, include_provider_modes: bool) -> GetRuntimeStatusData {
        GetRuntimeStatusData {
            page_id: None,
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
            speaking: false,
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

    fn next_confirmation_id(&self, request_id: &str) -> String {
        let timestamp_ms = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(_) => 0,
        };
        format!("confirm-{request_id}-{timestamp_ms}")
    }
}

impl DeterministicToolExecutor for AppCore {
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
}
