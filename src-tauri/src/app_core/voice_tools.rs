use crate::asr::AsrRuntimeError;
use crate::audio_io::AudioPlaybackError;
use crate::browser::BrowserError;
use crate::commands::{
    SetBrowserVisibilityData, SetBrowserVisibilityInput, SetPlaybackSpeedData,
    SetPlaybackSpeedInput, SetPlaybackVolumeData, SetPlaybackVolumeInput, SetTtsVoiceData,
    SetTtsVoiceInput, ToolError, ToolName, ToolResult,
};
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
}

pub(crate) fn tts_runtime_error_to_tool_error(error: TtsRuntimeError) -> ToolError {
    match error {
        TtsRuntimeError::EmptyNarrationText => ToolError {
            code: String::from("empty_narration_text"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        TtsRuntimeError::NarrationTextTooLarge { .. } => ToolError {
            code: String::from("tts_narration_text_too_large"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        TtsRuntimeError::OperationLimited { .. } => ToolError {
            code: String::from("tts_operation_limited"),
            message: error.to_string(),
            retryable: true,
            details: None,
        },
        TtsRuntimeError::RemoteConsentMissing => ToolError {
            code: String::from("tts_remote_consent_missing"),
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
        TtsRuntimeError::RemoteRequestTimedOut { .. } => ToolError {
            code: String::from("tts_request_timed_out"),
            message: error.to_string(),
            retryable: true,
            details: None,
        },
        TtsRuntimeError::RemoteHttpStatus { status } => ToolError {
            code: String::from("tts_http_status"),
            message: error.to_string(),
            retryable: status == 408 || status == 429 || status >= 500,
            details: None,
        },
        TtsRuntimeError::RemoteResponseTooLarge { .. } => ToolError {
            code: String::from("tts_response_too_large"),
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
        TtsRuntimeError::EmptySynthesizedAudio => ToolError {
            code: String::from("tts_empty_synthesized_audio"),
            message: error.to_string(),
            // Not retryable: this is most commonly punctuation-only or
            // otherwise degenerate narration text (e.g. "...", "?!"), which
            // will deterministically produce the same empty result again.
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

pub(crate) fn asr_runtime_error_to_tool_error(error: &AsrRuntimeError) -> ToolError {
    let code = match error {
        AsrRuntimeError::MissingLocalProfile
        | AsrRuntimeError::MissingLocalProfileDefinition { .. }
        | AsrRuntimeError::MissingRemoteProfile
        | AsrRuntimeError::MissingRemoteProfileDefinition { .. } => "asr_profile_unavailable",
        AsrRuntimeError::RemoteConsentMissing => "asr_remote_consent_missing",
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
        AsrRuntimeError::RemoteHttpStatus { .. } => "asr_http_status",
        AsrRuntimeError::RemoteResponseTooLarge { .. } => "asr_response_too_large",
        AsrRuntimeError::RemoteAudioTooLarge { .. } => "asr_audio_too_large",
        AsrRuntimeError::OperationLimited { .. } => "asr_operation_limited",
        AsrRuntimeError::RemoteRequestFailed { .. } => "asr_request_failed",
        AsrRuntimeError::LocalModelLoad { .. } => "asr_model_load_failed",
        AsrRuntimeError::WhisperContextCacheLockFailed => "asr_model_cache_lock_failed",
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
                | AsrRuntimeError::OperationLimited { .. }
        ) || matches!(
            error,
            AsrRuntimeError::RemoteHttpStatus { status }
                if *status == 408 || *status == 429 || *status >= 500
        ),
        details: None,
    }
}
