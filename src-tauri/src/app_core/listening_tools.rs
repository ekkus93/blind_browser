use super::voice_tools::asr_runtime_error_to_tool_error;
use crate::asr::{DEFAULT_TRANSCRIBE_DURATION_MS, MAX_TRANSCRIBE_DURATION_MS};
use crate::commands::{
    StartListeningData, StartListeningInput, StopListeningData, StopListeningInput, ToolError,
    ToolName, ToolResult, TranscribeAndExecuteCommandData, TranscribeCommandData,
    TranscribeCommandInput,
};

impl super::AppCore {
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
                String::from("Listening was already inactive, so capture remained stopped.")
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
