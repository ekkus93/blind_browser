use super::voice_tools::asr_runtime_error_to_tool_error;
use crate::asr::{AsrTranscription, DEFAULT_TRANSCRIBE_DURATION_MS, MAX_TRANSCRIBE_DURATION_MS};
use crate::commands::{
    StartListeningData, StartListeningInput, StopListeningData, StopListeningInput, ToolError,
    ToolName, ToolResult, TranscribeCommandData, TranscribeCommandInput,
};
use crate::state::ListeningState;

/// Build the observation strings shared by both transcription success paths.
fn build_transcribe_observations(
    requested_duration_ms: u64,
    effective_duration_ms: u64,
    timeout_ms: Option<u64>,
    transcript_present: bool,
) -> Vec<String> {
    let mut observations = vec![String::from(
        "Captured microphone audio and ran deterministic speech transcription.",
    )];
    if requested_duration_ms > MAX_TRANSCRIBE_DURATION_MS {
        observations.push(format!(
            "Requested capture duration was clamped to the supported maximum of {} ms.",
            MAX_TRANSCRIBE_DURATION_MS
        ));
    }
    if timeout_ms.is_some_and(|timeout_ms| timeout_ms < effective_duration_ms) {
        observations.push(String::from(
            "Capture duration was reduced to respect the requested tool timeout.",
        ));
    }
    observations.push(if transcript_present {
        String::from("ASR returned a non-empty spoken command transcript.")
    } else {
        String::from("ASR completed but did not detect a spoken command transcript.")
    });
    observations
}

/// Build the shared `TranscribeCommand` success `ToolResult` from a completed ASR
/// transcription. Used by both transcription paths — the lock-released handler path
/// ([`super::AppCore::finish_transcribe_command`]) and the planner-dispatched
/// lock-held tool path ([`super::AppCore::execute_transcribe_command`]) — so the
/// observation wording and result shape cannot drift apart.
fn transcribe_success_result(
    request_id: String,
    requested_duration_ms: u64,
    effective_duration_ms: u64,
    timeout_ms: Option<u64>,
    asr_result: AsrTranscription,
    listening_state: ListeningState,
) -> ToolResult<TranscribeCommandData> {
    let observations = build_transcribe_observations(
        requested_duration_ms,
        effective_duration_ms,
        timeout_ms,
        asr_result.transcript.is_some(),
    );
    ToolResult::success(
        ToolName::TranscribeCommand,
        request_id,
        TranscribeCommandData {
            transcript: asr_result.transcript,
            confidence: asr_result.confidence,
            audio_duration_ms: asr_result.audio_duration_ms,
            listening_state,
        },
        observations,
    )
}

/// Plan carried across the unlocked audio-capture window between
/// [`super::AppCore::begin_transcribe_command`] and
/// [`super::AppCore::finish_transcribe_command`]. Holds only plain owned data so
/// the `AppCore` lock can be released while the microphone captures audio.
pub struct TranscribeCapturePlan {
    request_id: String,
    requested_duration_ms: u64,
    pub effective_duration_ms: u64,
    timeout_ms: Option<u64>,
    auto_stop: bool,
    started_for_this_request: bool,
}

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

    /// Planner-dispatched `TranscribeCommand` tool path. Runs synchronously under
    /// the held `AppCore` lock by design — it executes inside a plan as one locked
    /// transaction (already off the main thread). The top-level handler path
    /// ([`Self::begin_transcribe_command`] / [`Self::finish_transcribe_command`])
    /// instead releases the lock across the capture window. Both share
    /// [`transcribe_success_result`] for result/observation construction.
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
                transcribe_success_result(
                    input.request_id,
                    requested_duration_ms,
                    effective_duration_ms,
                    input.timeout_ms,
                    result,
                    self.state.listening.clone(),
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

    /// Phase 1 of a lock-released transcription: validate the request and start
    /// (or locate) the capture session. On success returns a plan to carry across
    /// the unlocked capture window; on rejection returns the `ToolResult` to
    /// surface as-is. The caller must release the `AppCore` lock, sleep for
    /// `plan.effective_duration_ms`, then call [`Self::finish_transcribe_command`].
    pub fn begin_transcribe_command(
        &mut self,
        input: &TranscribeCommandInput,
    ) -> Result<TranscribeCapturePlan, Box<ToolResult<TranscribeCommandData>>> {
        let requested_duration_ms = input
            .max_duration_ms
            .unwrap_or(DEFAULT_TRANSCRIBE_DURATION_MS);
        if requested_duration_ms == 0 {
            return Err(Box::new(ToolResult::failure(
                ToolName::TranscribeCommand,
                input.request_id.clone(),
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
            )));
        }

        let mut effective_duration_ms = requested_duration_ms.min(MAX_TRANSCRIBE_DURATION_MS);
        if let Some(timeout_ms) = input.timeout_ms {
            effective_duration_ms = effective_duration_ms.min(timeout_ms.max(1));
        }

        match self.asr.begin_capture() {
            Ok(started_for_this_request) => {
                self.state.set_listening(self.asr.is_listening());
                Ok(TranscribeCapturePlan {
                    request_id: input.request_id.clone(),
                    requested_duration_ms,
                    effective_duration_ms,
                    timeout_ms: input.timeout_ms,
                    auto_stop: input.stop_mode.auto_stops(),
                    started_for_this_request,
                })
            }
            Err(error) => {
                self.state.set_listening(self.asr.is_listening());
                Err(Box::new(ToolResult::failure(
                    ToolName::TranscribeCommand,
                    input.request_id.clone(),
                    asr_runtime_error_to_tool_error(&error),
                    vec![String::from(
                        "Could not activate voice input listening in the configured ASR backend.",
                    )],
                )))
            }
        }
    }

    /// Phase 2 of a lock-released transcription: drain the captured audio, run the
    /// configured ASR backend, and record the transcript + listening state. Called
    /// after the unlocked capture window, with the `AppCore` lock re-acquired.
    ///
    /// Counterpart to the planner-dispatched [`Self::execute_transcribe_command`],
    /// which runs under the held lock; both share [`transcribe_success_result`] for
    /// the success result/observation construction.
    pub fn finish_transcribe_command(
        &mut self,
        plan: TranscribeCapturePlan,
    ) -> ToolResult<TranscribeCommandData> {
        match self
            .asr
            .finish_capture(&self.config, plan.auto_stop, plan.started_for_this_request)
        {
            Ok(Some(result)) => {
                self.state.set_listening(result.listening_active);
                self.state.record_transcript(result.transcript.clone());
                transcribe_success_result(
                    plan.request_id,
                    plan.requested_duration_ms,
                    plan.effective_duration_ms,
                    plan.timeout_ms,
                    result,
                    self.state.listening.clone(),
                )
            }
            Ok(None) => {
                // stop_listening interrupted the capture during the unlocked window.
                self.state.set_listening(self.asr.is_listening());
                ToolResult::success(
                    ToolName::TranscribeCommand,
                    plan.request_id,
                    TranscribeCommandData {
                        transcript: None,
                        confidence: None,
                        audio_duration_ms: None,
                        listening_state: self.state.listening.clone(),
                    },
                    vec![String::from(
                        "Listening was stopped during capture, so no spoken command was transcribed.",
                    )],
                )
            }
            Err(error) => {
                self.state.set_listening(self.asr.is_listening());
                ToolResult::failure(
                    ToolName::TranscribeCommand,
                    plan.request_id,
                    asr_runtime_error_to_tool_error(&error),
                    vec![String::from(
                        "Could not complete spoken-command transcription in the configured ASR backend.",
                    )],
                )
            }
        }
    }
}
