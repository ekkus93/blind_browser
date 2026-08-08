use super::voice_tools::asr_runtime_error_to_tool_error;
use crate::asr::{
    AsrTranscription, CapturedAudio, DEFAULT_TRANSCRIBE_DURATION_MS, MAX_TRANSCRIBE_DURATION_MS,
};
use crate::commands::{
    RemotePlannerConsentChallenge, StartListeningData, StartListeningInput, StopListeningData,
    StopListeningInput, ToolError, ToolName, ToolResult, TranscribeCommandData,
    TranscribeCommandInput,
};
use crate::config::AppConfig;
use crate::state::ListeningState;

/// Build the observation strings shared by both transcription success paths.
pub(crate) fn microphone_consent_required_error(
    challenge: &RemotePlannerConsentChallenge,
) -> ToolError {
    ToolError {
        code: String::from("remote_data_consent_required"),
        message: String::from(
            "Sending microphone audio to remote transcription requires your permission first. Review the pending privacy decision, then repeat the voice input.",
        ),
        retryable: false,
        details: Some(serde_json::json!({ "challenge": challenge })),
    }
}

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
/// ([`super::AppCore::record_transcribe_command`]) and the planner-dispatched
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
/// [`super::AppCore::drain_transcribe_command`]. Holds only plain owned data so
/// the `AppCore` lock can be released while the microphone captures audio.
pub struct TranscribeCapturePlan {
    input: TranscribeCommandInput,
    request_id: String,
    requested_duration_ms: u64,
    pub effective_duration_ms: u64,
    timeout_ms: Option<u64>,
    auto_stop: bool,
    started_for_this_request: bool,
}

/// Drained capture carried across the unlocked ASR-transcription window between
/// [`super::AppCore::drain_transcribe_command`] and
/// [`super::AppCore::record_transcribe_command`]. Holds owned audio plus a config
/// snapshot so the (possibly remote) transcription needs no `AppCore` access.
pub struct TranscribePending {
    plan: TranscribeCapturePlan,
    captured_audio: CapturedAudio,
    config: AppConfig,
    audio_duration_ms: Option<u64>,
}

impl TranscribePending {
    /// Borrow the exact request that this capture belongs to. The remote
    /// microphone consent gate binds authorization to these request semantics.
    pub(crate) fn input(&self) -> &TranscribeCommandInput {
        &self.plan.input
    }

    /// Borrow the snapshot the unlocked transcription step needs.
    pub(crate) fn transcription_inputs(&self) -> (&AppConfig, &CapturedAudio) {
        (&self.config, &self.captured_audio)
    }
}

/// Result of the lock-held drain phase: either a terminal result (capture stopped
/// mid-window or drain failed) or a pending capture to transcribe unlocked.
pub enum TranscribeDrainOutcome {
    Terminal(Box<ToolResult<TranscribeCommandData>>),
    Pending(Box<TranscribePending>),
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
    /// ([`Self::begin_transcribe_command`] → [`Self::drain_transcribe_command`] →
    /// [`Self::record_transcribe_command`]) instead releases the lock across both
    /// the capture and transcription windows. Both share
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

        let remote_authorization = match self.prepare_microphone_transcription(&input) {
            Ok(super::remote_data_consent::MicrophonePreparation::Authorized(authorization)) => {
                authorization
            }
            Ok(super::remote_data_consent::MicrophonePreparation::ConsentRequired {
                challenge,
            }) => {
                return ToolResult::failure(
                    ToolName::TranscribeCommand,
                    input.request_id,
                    microphone_consent_required_error(&challenge),
                    vec![String::from(
                        "Remote transcription paused before microphone audio was sent.",
                    )],
                );
            }
            Err(error) => {
                return ToolResult::failure(
                    ToolName::TranscribeCommand,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Remote transcription was blocked by the microphone privacy policy.",
                    )],
                );
            }
        };

        match self.asr.transcribe_command(
            &self.config,
            effective_duration_ms,
            input.stop_mode.auto_stops(),
            remote_authorization.as_ref(),
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
    /// `plan.effective_duration_ms`, then call [`Self::drain_transcribe_command`].
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

        match self.asr.begin_capture(input.stop_mode.auto_stops()) {
            Ok(started_for_this_request) => {
                self.state.set_listening(self.asr.is_listening());
                Ok(TranscribeCapturePlan {
                    input: input.clone(),
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

    /// Phase 2 of a lock-released transcription: drain the captured audio under the
    /// re-acquired lock and snapshot what the (possibly remote, network-bound)
    /// transcription needs, so the caller can release the lock again across the ASR
    /// round-trip. Returns a terminal `ToolResult` when capture was stopped
    /// mid-window or draining failed; otherwise a [`TranscribePending`] to feed to
    /// [`crate::asr::transcribe_captured_audio`] (unlocked) and then
    /// [`Self::record_transcribe_command`].
    pub fn drain_transcribe_command(
        &mut self,
        plan: TranscribeCapturePlan,
    ) -> TranscribeDrainOutcome {
        match self
            .asr
            .drain_capture(plan.auto_stop, plan.started_for_this_request)
        {
            Ok(Some(captured_audio)) => {
                let audio_duration_ms = Some(captured_audio.duration_ms());
                TranscribeDrainOutcome::Pending(Box::new(TranscribePending {
                    plan,
                    captured_audio,
                    config: self.config.clone(),
                    audio_duration_ms,
                }))
            }
            Ok(None) => {
                // stop_listening interrupted the capture during the unlocked window.
                self.state.set_listening(self.asr.is_listening());
                TranscribeDrainOutcome::Terminal(Box::new(ToolResult::success(
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
                )))
            }
            Err(error) => {
                self.state.set_listening(self.asr.is_listening());
                TranscribeDrainOutcome::Terminal(Box::new(ToolResult::failure(
                    ToolName::TranscribeCommand,
                    plan.request_id,
                    asr_runtime_error_to_tool_error(&error),
                    vec![String::from(
                        "Could not complete spoken-command transcription in the configured ASR backend.",
                    )],
                )))
            }
        }
    }

    /// Phase 3 of a lock-released transcription: record the transcript + listening
    /// state under the re-acquired lock and build the result. `transcript_result`
    /// is the outcome of [`crate::asr::transcribe_captured_audio`], run unlocked
    /// between [`Self::drain_transcribe_command`] and this call.
    ///
    /// Shares [`transcribe_success_result`] with the planner-dispatched
    /// [`Self::execute_transcribe_command`] (which runs under the held lock).
    pub fn record_transcribe_command(
        &mut self,
        pending: TranscribePending,
        transcript_result: Result<String, crate::asr::AsrRuntimeError>,
    ) -> ToolResult<TranscribeCommandData> {
        let TranscribePending {
            plan,
            audio_duration_ms,
            ..
        } = pending;
        match transcript_result {
            Ok(transcript) => {
                let asr_result = self
                    .asr
                    .finalize_transcription(transcript, audio_duration_ms);
                self.state.set_listening(asr_result.listening_active);
                self.state.record_transcript(asr_result.transcript.clone());
                transcribe_success_result(
                    plan.request_id,
                    plan.requested_duration_ms,
                    plan.effective_duration_ms,
                    plan.timeout_ms,
                    asr_result,
                    self.state.listening.clone(),
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
