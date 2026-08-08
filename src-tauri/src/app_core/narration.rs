use super::remote_data_consent::{
    NarrationPreparation, NarrationResumeContext, RemoteNarrationAuthorization,
};
use super::voice_tools::{audio_playback_error_to_tool_error, tts_runtime_error_to_tool_error};
use super::AppCore;
use crate::commands::{RemotePlannerConsentChallenge, ToolError};
use crate::config::ProviderMode;
use crate::narration::{cursor_for_index, find_region_index, spoken_text_for_region};
use crate::page_model::PageRegion;

/// Outcome of a narration attempt that may need to pause for remote-data
/// consent instead of completing: mirrors the planner's own
/// `RemotePlannerPreparation`/`ResolvePlanOutcome::NeedsRemoteDataConsent`
/// split, but at the narration choke point (`begin_region_narration`/
/// `begin_feedback_narration`) rather than the planner-resolution loop, so
/// every caller of either function -- read_region, read_next_region,
/// read_previous_region, report_result's spoken feedback -- is covered by
/// one gate instead of four separate ones.
pub(super) enum NarrationAttempt<T> {
    Completed(T),
    ConsentRequired(Box<RemotePlannerConsentChallenge>),
}

/// Shared error surfaced by every narration call site when a
/// `NarrationAttempt::ConsentRequired` is returned: the caller cannot
/// complete the narration and must not have spoken it, so the tool call
/// fails with a distinct, recognizable code -- the frontend separately polls
/// for the pending challenge (mirroring how remote-planner consent is
/// surfaced) rather than parsing it out of this error.
pub(super) fn narration_consent_required_error(
    challenge: &RemotePlannerConsentChallenge,
) -> ToolError {
    ToolError {
        code: String::from("remote_data_consent_required"),
        message: String::from(
            "Sending this page's text to remote narration requires your permission first. \
             Review the pending privacy decision to continue.",
        ),
        retryable: false,
        // The full challenge (not just its id) travels in the error details:
        // no frontend "fetch the pending challenge" query exists yet for
        // this disclosure kind, so this is the only place the UI can get
        // what it needs to render the consent dialog and call
        // submit_narration_consent_response. See CR3 P1.1's TODO note for
        // the frontend wiring this sets up for.
        details: Some(serde_json::json!({ "challenge": challenge })),
    }
}

impl AppCore {
    pub(super) fn readable_regions(&self) -> Result<&[PageRegion], ToolError> {
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

    pub(super) fn region_by_id(&self, region_id: &str) -> Result<(usize, PageRegion), ToolError> {
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

    pub(super) fn begin_region_narration(
        &mut self,
        region_index: usize,
        region: &PageRegion,
        interrupt_current: bool,
        request_id: &str,
    ) -> Result<NarrationAttempt<Option<String>>, ToolError> {
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

        let remote_authorization: Option<RemoteNarrationAuthorization> =
            if matches!(self.config.providers.tts.mode, ProviderMode::Remote) {
                let resume = NarrationResumeContext::Region {
                    region_id: region.region_id.clone(),
                    interrupt_current,
                };
                match self.prepare_narration_request(
                    &spoken_text,
                    request_id.to_string(),
                    resume,
                )? {
                    NarrationPreparation::ConsentRequired { challenge } => {
                        return Ok(NarrationAttempt::ConsentRequired(challenge));
                    }
                    NarrationPreparation::Authorized(authorization) => Some(authorization),
                }
            } else {
                None
            };

        let speech = self
            .tts
            .synthesize_narration(
                &self.config,
                &self.state.audio,
                &spoken_text,
                remote_authorization.as_ref(),
            )
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

        Ok(NarrationAttempt::Completed(interrupted_region_id))
    }

    pub(super) fn begin_feedback_narration(
        &mut self,
        spoken_text: &str,
        request_id: &str,
    ) -> Result<NarrationAttempt<()>, ToolError> {
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

        // Feedback text (the assistant's own generated summary, e.g. "I
        // couldn't find that button") is not page content the way region
        // text is, but it can still echo page-derived details and describes
        // the current page -- bound to the same page origin as region
        // narration rather than left ungated, per the P1.1.3 spec note
        // requiring this case be handled explicitly.
        let remote_authorization: Option<RemoteNarrationAuthorization> =
            if matches!(self.config.providers.tts.mode, ProviderMode::Remote) {
                let resume = NarrationResumeContext::Feedback {
                    spoken_text: spoken_text.to_string(),
                };
                match self.prepare_narration_request(spoken_text, request_id.to_string(), resume)? {
                    NarrationPreparation::ConsentRequired { challenge } => {
                        return Ok(NarrationAttempt::ConsentRequired(challenge));
                    }
                    NarrationPreparation::Authorized(authorization) => Some(authorization),
                }
            } else {
                None
            };

        let speech = self
            .tts
            .synthesize_narration(
                &self.config,
                &self.state.audio,
                spoken_text,
                remote_authorization.as_ref(),
            )
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

        Ok(NarrationAttempt::Completed(()))
    }

    pub(super) fn sync_narration_playback_state(&mut self) {
        if !self.playback.is_active() && self.state.speaking {
            self.state.stop_speaking();
        }
    }

    pub(super) fn stop_narration_playback(&mut self) -> Option<String> {
        let stopped_playback = self.playback.stop();
        let interrupted_region_id = self.state.stop_speaking();

        if interrupted_region_id.is_some() || stopped_playback {
            interrupted_region_id
        } else {
            None
        }
    }

    /// Resolve a pending narration consent challenge and, if authorized,
    /// redo exactly the paused narration attempt (see `NarrationResumeContext`)
    /// -- re-entering `begin_region_narration`/`begin_feedback_narration`,
    /// which this time proceed because the grant `resolve_narration_consent`
    /// just installed makes the policy re-evaluation pass.
    pub(crate) fn submit_narration_consent_response(
        &mut self,
        challenge_id: &str,
        challenge_digest: &str,
        decision: crate::commands::RemotePlannerConsentDecision,
    ) -> Result<crate::commands::NarrationConsentResponseOutcome, ToolError> {
        use super::remote_data_consent::NarrationConsentResolution;
        use crate::commands::{
            NarrationConsentResponseOutcome, RemotePlannerConsentResponseOutcome,
        };

        match self.resolve_narration_consent(challenge_id, challenge_digest, decision)? {
            NarrationConsentResolution::Terminal(RemotePlannerConsentResponseOutcome::Denied) => {
                Ok(NarrationConsentResponseOutcome::Denied)
            }
            NarrationConsentResolution::Terminal(
                RemotePlannerConsentResponseOutcome::BlockedPersistent,
            ) => Ok(NarrationConsentResponseOutcome::BlockedPersistent),
            NarrationConsentResolution::Terminal(_) => Err(ToolError {
                code: String::from("remote_data_consent_internal_error"),
                message: String::from(
                    "narration consent resolution returned an unexpected terminal outcome",
                ),
                retryable: false,
                details: None,
            }),
            NarrationConsentResolution::Authorized { resume } => {
                let resume_request_id = self.next_id("narration-consent-resume", challenge_id);
                let reevaluation_failed = || ToolError {
                    code: String::from("remote_data_consent_reevaluation_failed"),
                    message: String::from(
                        "narration still required consent immediately after it was just granted",
                    ),
                    retryable: false,
                    details: None,
                };
                match resume {
                    NarrationResumeContext::Region {
                        region_id,
                        interrupt_current,
                    } => {
                        let (region_index, region) = self.region_by_id(&region_id)?;
                        match self.begin_region_narration(
                            region_index,
                            &region,
                            interrupt_current,
                            &resume_request_id,
                        )? {
                            NarrationAttempt::Completed(_) => {
                                Ok(NarrationConsentResponseOutcome::Spoken)
                            }
                            NarrationAttempt::ConsentRequired(_) => Err(reevaluation_failed()),
                        }
                    }
                    NarrationResumeContext::Feedback { spoken_text } => {
                        match self.begin_feedback_narration(&spoken_text, &resume_request_id)? {
                            NarrationAttempt::Completed(()) => {
                                Ok(NarrationConsentResponseOutcome::Spoken)
                            }
                            NarrationAttempt::ConsentRequired(_) => Err(reevaluation_failed()),
                        }
                    }
                }
            }
        }
    }
}
