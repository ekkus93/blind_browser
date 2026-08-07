//! The narration (remote TTS) disclosure kind: gates page text sent to a
//! remote narration provider through the exact same policy engine
//! ([`super::evaluate_remote_planner_policy`]) as the remote planner, with
//! its own independent origin-rules/grants store (see
//! [`crate::config::AppConfig::remote_narration_privacy`]) so a planner
//! grant never silently authorizes narration text leaving the device too.
//!
//! Unlike the planner, narration has no sanitization step -- the whole point
//! is to speak the text -- so there is no separate draft/prepared-request
//! split: [`AppCore::prepare_narration_request`] either authorizes
//! immediately or returns a challenge to answer.

use sha2::{Digest, Sha256};

use crate::app_core::planner_redaction::high_risk_page_context_reason;
use crate::app_core::remote_privacy_api::page_origin_from_url;
use crate::commands::{
    current_timestamp_ms, RemotePlannerConsentDecision, RemotePlannerConsentResponseOutcome,
    ToolError,
};
use crate::config::PersistedOriginDecision;
use crate::provider_endpoint::ProviderEndpointScope;

use super::challenge::build_narration_consent_challenge;
use super::errors::{consent_error, policy_block_error_for};
use super::grants::RemoteDataDisclosureKind;
use super::origin_rules::disclosure_kind_label;
use super::types::{
    NarrationConsentResolution, NarrationPreparation, NarrationRequestDraft,
    NarrationResumeContext, PendingNarrationConsent,
};
use super::{evaluate_remote_planner_policy, RemotePlannerPolicyResult, SESSION_GRANT_TTL_MS};

const KIND: RemoteDataDisclosureKind = RemoteDataDisclosureKind::NarrationText;

impl super::super::AppCore {
    /// Evaluate whether `text` may be sent to the currently configured
    /// remote TTS profile, bound to the current page's origin (narration
    /// text is always page-derived: region text comes directly from the
    /// current page, and feedback text -- execute_report_result -- is bound
    /// to the same page it describes, per the P1.1.3 spec note requiring
    /// this case be handled explicitly rather than left to default). Returns
    /// `Authorized` when playback should proceed immediately, or
    /// `ConsentRequired` with a challenge already stored as pending (see
    /// [`Self::resolve_narration_consent`]) -- the caller only needs to
    /// surface the "consent required" outcome, not build or store anything.
    pub(crate) fn prepare_narration_request(
        &mut self,
        text: &str,
        request_id: String,
        resume: NarrationResumeContext,
    ) -> Result<NarrationPreparation, ToolError> {
        let Some(profile_name) = self.config.providers.tts.remote_profile.clone() else {
            return Err(consent_error(
                "tts_remote_profile_missing",
                "remote tts profile is not configured",
                None,
            ));
        };
        let Some(profile) = self.config.remote_tts_profiles.get(&profile_name).cloned() else {
            return Err(consent_error(
                "tts_remote_profile_missing",
                "the configured remote tts profile was not found",
                None,
            ));
        };
        let endpoint_scope = ProviderEndpointScope::parse(&profile.base_url).map_err(|reason| {
            consent_error(
                "tts_remote_endpoint_invalid",
                "remote tts endpoint is invalid",
                Some(serde_json::json!({ "reason": reason })),
            )
        })?;
        let page_origin = self
            .state
            .current_page
            .as_ref()
            .and_then(|page| page.url.as_deref())
            .and_then(page_origin_from_url);
        let now_ms = current_timestamp_ms();
        self.prune_remote_planner_grants(KIND, now_ms);
        let high_risk_reason = high_risk_page_context_reason(
            self.state
                .current_page
                .as_ref()
                .and_then(|page| page.url.as_deref()),
            self.state.current_page.as_ref(),
            None,
            &[],
        );
        match evaluate_remote_planner_policy(
            self.origin_rules_settings(KIND),
            &endpoint_scope,
            page_origin.as_deref(),
            high_risk_reason,
            self.ephemeral_grants(KIND),
            now_ms,
        ) {
            RemotePlannerPolicyResult::Allowed(_) => Ok(NarrationPreparation::Authorized),
            RemotePlannerPolicyResult::ConsentRequired => {
                let Some(page_origin) = page_origin else {
                    return Err(consent_error(
                        "remote_data_opaque_origin_blocked",
                        "the current page does not have a supported HTTP(S) origin",
                        None,
                    ));
                };
                let payload_digest = format!("{:x}", Sha256::digest(text.as_bytes()));
                let draft = NarrationRequestDraft {
                    text: text.to_string(),
                    endpoint_scope,
                    profile_name,
                    model_label: profile.model,
                    page_origin,
                    payload_digest,
                    runtime_state_token: self.current_runtime_state_token(),
                    resume,
                };
                let challenge = build_narration_consent_challenge(&draft, request_id, now_ms)?;
                self.pending_narration_consent = Some(PendingNarrationConsent {
                    challenge: challenge.clone(),
                    page_origin: draft.page_origin,
                    endpoint_scope: draft.endpoint_scope.normalized_base_url().to_string(),
                    runtime_state_token: draft.runtime_state_token,
                    resume: draft.resume,
                });
                Ok(NarrationPreparation::ConsentRequired {
                    challenge: Box::new(challenge),
                })
            }
            RemotePlannerPolicyResult::Blocked { code, reason_code } => Err(
                policy_block_error_for(disclosure_kind_label(KIND), code, reason_code),
            ),
        }
    }

    pub(crate) fn resolve_narration_consent(
        &mut self,
        challenge_id: &str,
        challenge_digest: &str,
        decision: RemotePlannerConsentDecision,
    ) -> Result<NarrationConsentResolution, ToolError> {
        let pending = self.pending_narration_consent.take().ok_or_else(|| {
            consent_error(
                "remote_data_consent_missing",
                "no remote-data consent request is pending",
                None,
            )
        })?;
        let now_ms = current_timestamp_ms();
        if pending.challenge.challenge_id != challenge_id
            || pending.challenge.challenge_digest != challenge_digest
        {
            return Err(consent_error(
                "remote_data_consent_mismatch",
                "remote-data consent response did not match the pending challenge",
                None,
            ));
        }
        if now_ms >= pending.challenge.expires_at_ms {
            return Err(consent_error(
                "remote_data_consent_expired",
                "remote-data consent challenge expired before it was answered",
                Some(serde_json::json!({
                    "expired_at_ms": pending.challenge.expires_at_ms,
                    "observed_at_ms": now_ms,
                })),
            ));
        }
        if pending.runtime_state_token != self.current_runtime_state_token() {
            return Err(consent_error(
                "remote_data_consent_state_changed",
                "runtime state changed after the remote-data consent challenge was created",
                None,
            ));
        }

        match decision {
            RemotePlannerConsentDecision::Deny => Ok(NarrationConsentResolution::Terminal(
                RemotePlannerConsentResponseOutcome::Denied,
            )),
            RemotePlannerConsentDecision::BlockPersistent => {
                self.persist_origin_rule(
                    KIND,
                    &pending.page_origin,
                    PersistedOriginDecision::Block,
                    None,
                )?;
                self.remote_narration_ephemeral_grants
                    .retain(|grant| grant.page_origin != pending.page_origin);
                Ok(NarrationConsentResolution::Terminal(
                    RemotePlannerConsentResponseOutcome::BlockedPersistent,
                ))
            }
            RemotePlannerConsentDecision::AllowPersistent => {
                self.persist_origin_rule(
                    KIND,
                    &pending.page_origin,
                    PersistedOriginDecision::Allow,
                    Some(pending.endpoint_scope.clone()),
                )?;
                Ok(NarrationConsentResolution::Authorized {
                    resume: pending.resume,
                })
            }
            RemotePlannerConsentDecision::AllowSession => {
                self.install_session_grant(
                    KIND,
                    pending.page_origin.clone(),
                    pending.endpoint_scope.clone(),
                    now_ms.saturating_add(SESSION_GRANT_TTL_MS),
                );
                Ok(NarrationConsentResolution::Authorized {
                    resume: pending.resume,
                })
            }
            RemotePlannerConsentDecision::AllowOnce => {
                self.install_once_grant(
                    KIND,
                    pending.page_origin.clone(),
                    pending.endpoint_scope.clone(),
                    pending.challenge.challenge_digest.clone(),
                    pending.challenge.expires_at_ms,
                );
                if !self.consume_once_grant(
                    KIND,
                    &pending.page_origin,
                    &pending.endpoint_scope,
                    &pending.challenge.challenge_digest,
                    now_ms,
                ) {
                    return Err(consent_error(
                        "remote_data_consent_replay",
                        "one-shot remote-data consent was already consumed or expired",
                        None,
                    ));
                }
                Ok(NarrationConsentResolution::Authorized {
                    resume: pending.resume,
                })
            }
        }
    }
}
