use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::element_scoring::normalize_optional_text;
use super::narration::{narration_consent_required_error, NarrationAttempt};
use super::remote_data_consent::{MicrophonePreparation, NarrationResumeContext};
use super::voice_tools::tts_runtime_error_to_tool_error;
use super::{AppCore, TranscribeDrainOutcome};
use crate::asr::transcribe_captured_audio;
use crate::commands::{
    execute_planned_step, execute_serialized_step, preflight_rejection, ExecutionOutcome,
    ExecutionTrace, PlannedStep, ReadNextRegionData, ReadNextRegionInput, ReadPreviousRegionData,
    ReadPreviousRegionInput, ReadRegionData, ReadRegionInput, ReportResultData, ReportResultInput,
    SerializedToolResult, ToolError, ToolName, ToolResult, TranscribeCommandData,
    TranscribeCommandInput,
};
use crate::config::ProviderMode;
use crate::lock_app_core;
use crate::narration::{next_region_index, previous_region_index};
use crate::tts::synthesize_prepared_remote_narration;

#[derive(Debug)]
enum RunnerStop {
    Replan,
    Abort(ToolError),
}

/// Runs the existing deterministic planner loop without holding the global
/// `AppCore` mutex for the whole plan. Ordinary deterministic steps still run
/// atomically under one short guard. Speech steps use prepare -> unlocked I/O
/// -> commit phases so microphone waits and remote HTTP never monopolize the
/// global runtime lock.
pub(crate) struct LockScopedStepRunner<'a> {
    core: &'a Arc<Mutex<AppCore>>,
    expected_token_without_listening: String,
    expected_listening: bool,
    stop: Option<RunnerStop>,
}

impl<'a> LockScopedStepRunner<'a> {
    pub(crate) fn new(
        core: &'a Arc<Mutex<AppCore>>,
        expected_token_without_listening: String,
        expected_listening: bool,
    ) -> Self {
        Self {
            core,
            expected_token_without_listening,
            expected_listening,
            stop: None,
        }
    }

    pub(crate) fn run(&mut self, step: &PlannedStep) -> SerializedToolResult {
        if self.stop.is_some() {
            return self.stopped_step_result(step);
        }
        match step.tool_name {
            ToolName::TranscribeCommand => execute_serialized_step(
                step,
                ToolName::TranscribeCommand,
                |input: TranscribeCommandInput| self.run_transcribe(step, input),
            ),
            ToolName::ReadRegion => execute_serialized_step(
                step,
                ToolName::ReadRegion,
                |input: ReadRegionInput| self.run_read_region(step, input),
            ),
            ToolName::ReadNextRegion => execute_serialized_step(
                step,
                ToolName::ReadNextRegion,
                |input: ReadNextRegionInput| self.run_read_next_region(step, input),
            ),
            ToolName::ReadPreviousRegion => execute_serialized_step(
                step,
                ToolName::ReadPreviousRegion,
                |input: ReadPreviousRegionInput| self.run_read_previous_region(step, input),
            ),
            ToolName::ReportResult => execute_serialized_step(
                step,
                ToolName::ReportResult,
                |input: ReportResultInput| self.run_report_result(step, input),
            ),
            _ => self.run_locked(step),
        }
    }

    pub(crate) fn reconcile_outcome(&mut self, outcome: ExecutionOutcome) -> ExecutionOutcome {
        let Some(stop) = self.stop.take() else {
            return outcome;
        };
        let trace = outcome_trace(outcome);
        match stop {
            RunnerStop::Replan => ExecutionOutcome::NeedsReplan { trace },
            RunnerStop::Abort(error) => ExecutionOutcome::Aborted { trace, error },
        }
    }

    fn run_locked(&mut self, step: &PlannedStep) -> SerializedToolResult {
        let mut guard = match lock_app_core(self.core) {
            Ok(guard) => guard,
            Err(error) => return self.abort_step(step, error),
        };
        if !self.runtime_is_compatible(&guard) {
            return self.replan_step(step);
        }
        let result = execute_planned_step(&mut *guard, step);
        self.refresh_expected_state(&guard);
        result
    }

    fn run_transcribe(
        &mut self,
        step: &PlannedStep,
        input: TranscribeCommandInput,
    ) -> ToolResult<TranscribeCommandData> {
        let (_, effective_duration_ms) = match super::transcribe_capture_durations(&input) {
            Ok(durations) => durations,
            Err(result) => return *result,
        };

        let (plan, remote_authorization) = {
            let mut guard = match lock_app_core(self.core) {
                Ok(guard) => guard,
                Err(error) => return self.abort_typed(ToolName::TranscribeCommand, &input.request_id, error),
            };
            if !self.runtime_is_compatible(&guard) {
                return self.replan_typed(ToolName::TranscribeCommand, &input.request_id);
            }
            if let Err(error) = guard.preflight_planned_step_runtime(step) {
                return typed_from_serialized(preflight_rejection(step, error));
            }
            let authorization = match guard.prepare_microphone_transcription(&input) {
                Ok(MicrophonePreparation::Authorized(authorization)) => authorization,
                Ok(MicrophonePreparation::ConsentRequired { challenge }) => {
                    guard.discard_microphone_capture_after_privacy_rejection();
                    self.refresh_expected_state(&guard);
                    return ToolResult::failure(
                        ToolName::TranscribeCommand,
                        input.request_id,
                        super::microphone_consent_required_error(&challenge),
                        vec![String::from(
                            "Remote transcription paused before microphone capture began.",
                        )],
                    );
                }
                Err(error) => {
                    guard.discard_microphone_capture_after_privacy_rejection();
                    self.refresh_expected_state(&guard);
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
            let plan = match guard.begin_transcribe_command(&input) {
                Ok(plan) => plan,
                Err(result) => {
                    self.refresh_expected_state(&guard);
                    return *result;
                }
            };
            self.refresh_expected_state(&guard);
            (plan, authorization)
        };

        // P1.2: this capture window is deliberately outside the AppCore guard.
        thread::sleep(Duration::from_millis(effective_duration_ms));

        let pending = {
            let mut guard = match lock_app_core(self.core) {
                Ok(guard) => guard,
                Err(error) => return self.abort_typed(ToolName::TranscribeCommand, &input.request_id, error),
            };
            if !self.runtime_is_compatible(&guard) {
                guard.discard_microphone_capture_after_privacy_rejection();
                self.refresh_expected_state(&guard);
                return self.replan_typed(ToolName::TranscribeCommand, &input.request_id);
            }
            match guard.drain_transcribe_command(plan) {
                TranscribeDrainOutcome::Terminal(result) => {
                    self.refresh_expected_state(&guard);
                    return *result;
                }
                TranscribeDrainOutcome::Pending(pending) => {
                    self.refresh_expected_state(&guard);
                    pending
                }
            }
        };

        let transcript_result = {
            let (config, captured_audio) = pending.transcription_inputs();
            transcribe_captured_audio(config, captured_audio, remote_authorization)
        };

        let mut guard = match lock_app_core(self.core) {
            Ok(guard) => guard,
            Err(error) => return self.abort_typed(ToolName::TranscribeCommand, &input.request_id, error),
        };
        if !self.runtime_is_compatible(&guard) {
            self.refresh_expected_state(&guard);
            return self.replan_typed(ToolName::TranscribeCommand, &input.request_id);
        }
        let result = guard.record_transcribe_command(pending, transcript_result);
        self.refresh_expected_state(&guard);
        result
    }

    fn run_read_region(
        &mut self,
        step: &PlannedStep,
        input: ReadRegionInput,
    ) -> ToolResult<ReadRegionData> {
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

        let (region_index, region, synthesis) = {
            let mut guard = match lock_app_core(self.core) {
                Ok(guard) => guard,
                Err(error) => return self.abort_typed(ToolName::ReadRegion, &input.request_id, error),
            };
            if !self.runtime_is_compatible(&guard) {
                return self.replan_typed(ToolName::ReadRegion, &input.request_id);
            }
            if !matches!(guard.config.providers.tts.mode, ProviderMode::Remote) {
                let result = guard.execute_read_region(input);
                self.refresh_expected_state(&guard);
                return result;
            }
            if let Err(error) = guard.preflight_planned_step_runtime(step) {
                return typed_from_serialized(preflight_rejection(step, error));
            }
            let (region_index, region) = match guard.region_by_id(&region_id) {
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
            let synthesis = match guard.prepare_remote_region_narration(
                region_index,
                &region,
                input.interruption_mode.interrupts_current_playback(),
                &input.request_id,
            ) {
                Ok(NarrationAttempt::Completed(synthesis)) => synthesis,
                Ok(NarrationAttempt::ConsentRequired(challenge)) => {
                    self.refresh_expected_state(&guard);
                    return ToolResult::failure(
                        ToolName::ReadRegion,
                        input.request_id,
                        narration_consent_required_error(&challenge),
                        vec![String::from(
                            "Narration was paused because sending this page's text to remote narration requires your permission first.",
                        )],
                    );
                }
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
            self.refresh_expected_state(&guard);
            (region_index, region, synthesis)
        };

        let completed = match synthesize_prepared_remote_narration(synthesis) {
            Ok(completed) => completed,
            Err(error) => return self.remote_tts_failure(ToolName::ReadRegion, &input.request_id, error),
        };
        let mut guard = match lock_app_core(self.core) {
            Ok(guard) => guard,
            Err(error) => return self.abort_typed(ToolName::ReadRegion, &input.request_id, error),
        };
        if !self.runtime_is_compatible(&guard) {
            return self.replan_typed(ToolName::ReadRegion, &input.request_id);
        }
        let interrupted_region_id = match guard.finish_remote_region_narration(region_index, &region, completed) {
            Ok(value) => value,
            Err(error) => {
                self.refresh_expected_state(&guard);
                return ToolResult::failure(
                    ToolName::ReadRegion,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Narration request could not start playback for the requested region.",
                    )],
                );
            }
        };
        self.refresh_expected_state(&guard);

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

    fn run_read_next_region(
        &mut self,
        step: &PlannedStep,
        input: ReadNextRegionInput,
    ) -> ToolResult<ReadNextRegionData> {
        let (region_index, region, synthesis) = {
            let mut guard = match lock_app_core(self.core) {
                Ok(guard) => guard,
                Err(error) => return self.abort_typed(ToolName::ReadNextRegion, &input.request_id, error),
            };
            if !self.runtime_is_compatible(&guard) {
                return self.replan_typed(ToolName::ReadNextRegion, &input.request_id);
            }
            if !matches!(guard.config.providers.tts.mode, ProviderMode::Remote) {
                let result = guard.execute_read_next_region(input);
                self.refresh_expected_state(&guard);
                return result;
            }
            if let Err(error) = guard.preflight_planned_step_runtime(step) {
                return typed_from_serialized(preflight_rejection(step, error));
            }
            guard.sync_narration_playback_state();
            let regions = match guard.readable_regions() {
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
            let Some((region_index, region)) =
                next_region_index(&guard.state.narration_cursor, regions.len())
                    .and_then(|index| regions.get(index).map(|region| (index, region.clone())))
            else {
                return ToolResult::success(
                    ToolName::ReadNextRegion,
                    input.request_id,
                    ReadNextRegionData {
                        cursor: guard.state.narration_cursor.clone(),
                        region_id: None,
                        speech_started: false,
                        boundary: crate::commands::NarrationBoundary::End,
                    },
                    vec![String::from(
                        "Narration is already at the end of the readable region list.",
                    )],
                );
            };
            let synthesis = match guard.prepare_remote_region_narration(
                region_index,
                &region,
                input.interruption_mode.interrupts_current_playback(),
                &input.request_id,
            ) {
                Ok(NarrationAttempt::Completed(synthesis)) => synthesis,
                Ok(NarrationAttempt::ConsentRequired(challenge)) => {
                    self.refresh_expected_state(&guard);
                    return ToolResult::failure(
                        ToolName::ReadNextRegion,
                        input.request_id,
                        narration_consent_required_error(&challenge),
                        vec![String::from(
                            "Narration was paused because sending this page's text to remote narration requires your permission first.",
                        )],
                    );
                }
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
            self.refresh_expected_state(&guard);
            (region_index, region, synthesis)
        };

        let completed = match synthesize_prepared_remote_narration(synthesis) {
            Ok(completed) => completed,
            Err(error) => return self.remote_tts_failure(ToolName::ReadNextRegion, &input.request_id, error),
        };
        let mut guard = match lock_app_core(self.core) {
            Ok(guard) => guard,
            Err(error) => return self.abort_typed(ToolName::ReadNextRegion, &input.request_id, error),
        };
        if !self.runtime_is_compatible(&guard) {
            return self.replan_typed(ToolName::ReadNextRegion, &input.request_id);
        }
        let interrupted_region_id = match guard.finish_remote_region_narration(region_index, &region, completed) {
            Ok(value) => value,
            Err(error) => {
                self.refresh_expected_state(&guard);
                return ToolResult::failure(
                    ToolName::ReadNextRegion,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Narration could not advance to the next region for playback.",
                    )],
                );
            }
        };
        self.refresh_expected_state(&guard);
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
                cursor: guard.state.narration_cursor.clone(),
                region_id: Some(region.region_id),
                speech_started: true,
                boundary: crate::commands::NarrationBoundary::None,
            },
            observations,
        )
    }

    fn run_read_previous_region(
        &mut self,
        step: &PlannedStep,
        input: ReadPreviousRegionInput,
    ) -> ToolResult<ReadPreviousRegionData> {
        let (region_index, region, synthesis) = {
            let mut guard = match lock_app_core(self.core) {
                Ok(guard) => guard,
                Err(error) => return self.abort_typed(ToolName::ReadPreviousRegion, &input.request_id, error),
            };
            if !self.runtime_is_compatible(&guard) {
                return self.replan_typed(ToolName::ReadPreviousRegion, &input.request_id);
            }
            if !matches!(guard.config.providers.tts.mode, ProviderMode::Remote) {
                let result = guard.execute_read_previous_region(input);
                self.refresh_expected_state(&guard);
                return result;
            }
            if let Err(error) = guard.preflight_planned_step_runtime(step) {
                return typed_from_serialized(preflight_rejection(step, error));
            }
            guard.sync_narration_playback_state();
            let regions = match guard.readable_regions() {
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
            let Some((region_index, region)) =
                previous_region_index(&guard.state.narration_cursor, regions.len())
                    .and_then(|index| regions.get(index).map(|region| (index, region.clone())))
            else {
                return ToolResult::success(
                    ToolName::ReadPreviousRegion,
                    input.request_id,
                    ReadPreviousRegionData {
                        cursor: guard.state.narration_cursor.clone(),
                        region_id: None,
                        speech_started: false,
                        boundary: crate::commands::NarrationBoundary::Start,
                    },
                    vec![String::from(
                        "Narration is already at the start of the readable region list.",
                    )],
                );
            };
            let synthesis = match guard.prepare_remote_region_narration(
                region_index,
                &region,
                input.interruption_mode.interrupts_current_playback(),
                &input.request_id,
            ) {
                Ok(NarrationAttempt::Completed(synthesis)) => synthesis,
                Ok(NarrationAttempt::ConsentRequired(challenge)) => {
                    self.refresh_expected_state(&guard);
                    return ToolResult::failure(
                        ToolName::ReadPreviousRegion,
                        input.request_id,
                        narration_consent_required_error(&challenge),
                        vec![String::from(
                            "Narration was paused because sending this page's text to remote narration requires your permission first.",
                        )],
                    );
                }
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
            self.refresh_expected_state(&guard);
            (region_index, region, synthesis)
        };

        let completed = match synthesize_prepared_remote_narration(synthesis) {
            Ok(completed) => completed,
            Err(error) => return self.remote_tts_failure(ToolName::ReadPreviousRegion, &input.request_id, error),
        };
        let mut guard = match lock_app_core(self.core) {
            Ok(guard) => guard,
            Err(error) => return self.abort_typed(ToolName::ReadPreviousRegion, &input.request_id, error),
        };
        if !self.runtime_is_compatible(&guard) {
            return self.replan_typed(ToolName::ReadPreviousRegion, &input.request_id);
        }
        let interrupted_region_id = match guard.finish_remote_region_narration(region_index, &region, completed) {
            Ok(value) => value,
            Err(error) => {
                self.refresh_expected_state(&guard);
                return ToolResult::failure(
                    ToolName::ReadPreviousRegion,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Narration could not move backward to the previous region for playback.",
                    )],
                );
            }
        };
        self.refresh_expected_state(&guard);
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
                cursor: guard.state.narration_cursor.clone(),
                region_id: Some(region.region_id),
                speech_started: true,
                boundary: crate::commands::NarrationBoundary::None,
            },
            observations,
        )
    }

    fn run_report_result(
        &mut self,
        step: &PlannedStep,
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

        let synthesis = {
            let mut guard = match lock_app_core(self.core) {
                Ok(guard) => guard,
                Err(error) => return self.abort_typed(ToolName::ReportResult, &input.request_id, error),
            };
            if !self.runtime_is_compatible(&guard) {
                return self.replan_typed(ToolName::ReportResult, &input.request_id);
            }
            if !matches!(guard.config.providers.tts.mode, ProviderMode::Remote) {
                let result = guard.execute_report_result(ReportResultInput {
                    request_id: input.request_id,
                    timeout_ms: input.timeout_ms,
                    status: input.status,
                    summary,
                    next_recommended_action,
                    user_message,
                });
                self.refresh_expected_state(&guard);
                return result;
            }
            if let Err(error) = guard.preflight_planned_step_runtime(step) {
                return typed_from_serialized(preflight_rejection(step, error));
            }
            let synthesis = match guard.prepare_remote_feedback_narration(
                &spoken_message,
                &input.request_id,
            ) {
                Ok(NarrationAttempt::Completed(synthesis)) => synthesis,
                Ok(NarrationAttempt::ConsentRequired(challenge)) => {
                    self.refresh_expected_state(&guard);
                    return ToolResult::failure(
                        ToolName::ReportResult,
                        input.request_id,
                        narration_consent_required_error(&challenge),
                        vec![String::from(
                            "Spoken feedback was paused because sending this page's text to remote narration requires your permission first.",
                        )],
                    );
                }
                Err(error) => {
                    return ToolResult::failure(
                        ToolName::ReportResult,
                        input.request_id,
                        error,
                        vec![String::from(
                            "Final result reporting could not start audible feedback with the configured TTS backend.",
                        )],
                    )
                }
            };
            self.refresh_expected_state(&guard);
            synthesis
        };

        let completed = match synthesize_prepared_remote_narration(synthesis) {
            Ok(completed) => completed,
            Err(error) => return self.remote_tts_failure(ToolName::ReportResult, &input.request_id, error),
        };
        let mut guard = match lock_app_core(self.core) {
            Ok(guard) => guard,
            Err(error) => return self.abort_typed(ToolName::ReportResult, &input.request_id, error),
        };
        if !self.runtime_is_compatible(&guard) {
            return self.replan_typed(ToolName::ReportResult, &input.request_id);
        }
        if let Err(error) = guard.finish_remote_feedback_narration(completed) {
            self.refresh_expected_state(&guard);
            return ToolResult::failure(
                ToolName::ReportResult,
                input.request_id,
                error,
                vec![String::from(
                    "Final result reporting could not start audible feedback with the configured TTS backend.",
                )],
            );
        }
        self.refresh_expected_state(&guard);
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

    pub(crate) fn resume_narration_after_consent(
        &mut self,
        resume: NarrationResumeContext,
        request_id: &str,
    ) -> Result<(), ToolError> {
        match resume {
            NarrationResumeContext::Region {
                region_id,
                interrupt_current,
            } => {
                let (region_index, region, synthesis) = {
                    let mut guard = lock_app_core(self.core)?;
                    if !self.runtime_is_compatible(&guard) {
                        return Err(stale_execution_error());
                    }
                    let (region_index, region) = guard.region_by_id(&region_id)?;
                    let synthesis = match guard.prepare_remote_region_narration(
                        region_index,
                        &region,
                        interrupt_current,
                        request_id,
                    )? {
                        NarrationAttempt::Completed(synthesis) => synthesis,
                        NarrationAttempt::ConsentRequired(_) => {
                            return Err(ToolError {
                                code: String::from("remote_data_consent_reevaluation_failed"),
                                message: String::from(
                                    "narration still required consent immediately after it was just granted",
                                ),
                                retryable: false,
                                details: None,
                            });
                        }
                    };
                    self.refresh_expected_state(&guard);
                    (region_index, region, synthesis)
                };
                let completed = synthesize_prepared_remote_narration(synthesis)
                    .map_err(|error| tts_runtime_error_to_tool_error(&error))?;
                let mut guard = lock_app_core(self.core)?;
                if !self.runtime_is_compatible(&guard) {
                    return Err(stale_execution_error());
                }
                guard.finish_remote_region_narration(region_index, &region, completed)?;
                self.refresh_expected_state(&guard);
                Ok(())
            }
            NarrationResumeContext::Feedback { spoken_text } => {
                let synthesis = {
                    let mut guard = lock_app_core(self.core)?;
                    if !self.runtime_is_compatible(&guard) {
                        return Err(stale_execution_error());
                    }
                    let synthesis = match guard.prepare_remote_feedback_narration(
                        &spoken_text,
                        request_id,
                    )? {
                        NarrationAttempt::Completed(synthesis) => synthesis,
                        NarrationAttempt::ConsentRequired(_) => {
                            return Err(ToolError {
                                code: String::from("remote_data_consent_reevaluation_failed"),
                                message: String::from(
                                    "narration still required consent immediately after it was just granted",
                                ),
                                retryable: false,
                                details: None,
                            });
                        }
                    };
                    self.refresh_expected_state(&guard);
                    synthesis
                };
                let completed = synthesize_prepared_remote_narration(synthesis)
                    .map_err(|error| tts_runtime_error_to_tool_error(&error))?;
                let mut guard = lock_app_core(self.core)?;
                if !self.runtime_is_compatible(&guard) {
                    return Err(stale_execution_error());
                }
                guard.finish_remote_feedback_narration(completed)?;
                self.refresh_expected_state(&guard);
                Ok(())
            }
        }
    }

    fn remote_tts_failure<T>(
        &mut self,
        tool_name: ToolName,
        request_id: &str,
        error: crate::tts::TtsRuntimeError,
    ) -> ToolResult<T> {
        // Even a failed network call ran during an unlocked window. Re-check
        // state before surfacing the provider failure so a concurrent runtime
        // mutation cannot be accidentally ignored.
        match lock_app_core(self.core) {
            Ok(guard) => {
                if !self.runtime_is_compatible(&guard) {
                    return self.replan_typed(tool_name, request_id);
                }
            }
            Err(lock_error) => return self.abort_typed(tool_name, request_id, lock_error),
        }
        ToolResult::failure(
            tool_name,
            request_id.to_string(),
            tts_runtime_error_to_tool_error(&error),
            vec![String::from(
                "Remote narration synthesis did not complete successfully.",
            )],
        )
    }

    fn runtime_is_compatible(&mut self, guard: &AppCore) -> bool {
        let observed_token = guard.current_lock_scoped_execution_token_without_listening();
        if observed_token != self.expected_token_without_listening {
            self.stop = Some(RunnerStop::Replan);
            return false;
        }
        let observed_listening = guard.current_lock_scoped_listening_state();
        if observed_listening == self.expected_listening {
            return true;
        }
        if self.expected_listening && !observed_listening {
            // The one permitted interleaving: an explicit stop request while a
            // capture/network window is unlocked.
            self.expected_listening = false;
            return true;
        }
        self.stop = Some(RunnerStop::Replan);
        false
    }

    fn refresh_expected_state(&mut self, guard: &AppCore) {
        self.expected_token_without_listening =
            guard.current_lock_scoped_execution_token_without_listening();
        self.expected_listening = guard.current_lock_scoped_listening_state();
    }

    fn replan_step(&mut self, step: &PlannedStep) -> SerializedToolResult {
        self.stop = Some(RunnerStop::Replan);
        ToolResult::failure(
            step.tool_name.clone(),
            step.arguments
                .get("request_id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(&step.step_id)
                .to_string(),
            stale_execution_error(),
            vec![String::from(
                "Runtime state changed during lock-released execution; bounded replanning is required.",
            )],
        )
    }

    fn replan_typed<T>(&mut self, tool_name: ToolName, request_id: &str) -> ToolResult<T> {
        self.stop = Some(RunnerStop::Replan);
        ToolResult::failure(
            tool_name,
            request_id.to_string(),
            stale_execution_error(),
            vec![String::from(
                "Runtime state changed during lock-released execution; bounded replanning is required.",
            )],
        )
    }

    fn abort_step(&mut self, step: &PlannedStep, error: ToolError) -> SerializedToolResult {
        self.stop = Some(RunnerStop::Abort(error.clone()));
        ToolResult::failure(
            step.tool_name.clone(),
            step.arguments
                .get("request_id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(&step.step_id)
                .to_string(),
            error,
            vec![String::from("AppCore became unavailable during plan execution.")],
        )
    }

    fn abort_typed<T>(
        &mut self,
        tool_name: ToolName,
        request_id: &str,
        error: ToolError,
    ) -> ToolResult<T> {
        self.stop = Some(RunnerStop::Abort(error.clone()));
        ToolResult::failure(
            tool_name,
            request_id.to_string(),
            error,
            vec![String::from("AppCore became unavailable during plan execution.")],
        )
    }

    fn stopped_step_result(&self, step: &PlannedStep) -> SerializedToolResult {
        let error = match self.stop.as_ref() {
            Some(RunnerStop::Abort(error)) => error.clone(),
            Some(RunnerStop::Replan) | None => stale_execution_error(),
        };
        ToolResult::failure(
            step.tool_name.clone(),
            step.arguments
                .get("request_id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(&step.step_id)
                .to_string(),
            error,
            vec![String::from("Planner execution was already stopped.")],
        )
    }
}

fn stale_execution_error() -> ToolError {
    ToolError {
        code: String::from("stale_lock_scoped_execution"),
        message: String::from(
            "runtime state changed while blocking speech work ran without the AppCore lock",
        ),
        retryable: true,
        details: None,
    }
}

fn outcome_trace(outcome: ExecutionOutcome) -> ExecutionTrace {
    match outcome {
        ExecutionOutcome::Complete { trace }
        | ExecutionOutcome::AwaitingConfirmation { trace, .. }
        | ExecutionOutcome::NeedsReplan { trace }
        | ExecutionOutcome::NeedsRemoteDataConsent { trace, .. }
        | ExecutionOutcome::Aborted { trace, .. } => trace,
    }
}

/// This conversion is only used after `preflight_rejection`, whose payload is
/// intentionally data-less. Keeping it local avoids exposing planner-executor
/// serialization internals more broadly.
fn typed_from_serialized<T>(result: SerializedToolResult) -> ToolResult<T> {
    ToolResult {
        ok: result.ok,
        tool_name: result.tool_name,
        request_id: result.request_id,
        timestamp_ms: result.timestamp_ms,
        data: None,
        error: result.error,
        warnings: result.warnings,
        observations: result.observations,
    }
}
