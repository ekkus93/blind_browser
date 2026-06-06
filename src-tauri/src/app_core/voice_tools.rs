use crate::asr::{AsrRuntimeError, DEFAULT_TRANSCRIBE_DURATION_MS, MAX_TRANSCRIBE_DURATION_MS};
use crate::audio_io::AudioPlaybackError;
use crate::browser::BrowserError;
use crate::commands::{
    ReadNextRegionInput, ReadNextRegionData, ReadPreviousRegionData, ReadPreviousRegionInput,
    ReadRegionData, ReadRegionInput, SetBrowserVisibilityData, SetBrowserVisibilityInput,
    SetPlaybackSpeedData, SetPlaybackSpeedInput, SetPlaybackVolumeData, SetPlaybackVolumeInput,
    SetTtsVoiceData, SetTtsVoiceInput, StartListeningData, StartListeningInput, StopListeningData,
    StopListeningInput, StopSpeakingData, StopSpeakingInput, ToolError, ToolName, ToolResult,
    TranscribeAndExecuteCommandData, TranscribeCommandData, TranscribeCommandInput,
};
use crate::narration::{next_region_index, previous_region_index};
use crate::tts::TtsRuntimeError;

impl super::AppCore {
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

}

pub(crate) fn tts_runtime_error_to_tool_error(error: TtsRuntimeError) -> ToolError {
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

pub(crate) fn audio_playback_error_to_tool_error(error: AudioPlaybackError) -> ToolError {
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
