//! Remote-ASR disclosure gate. Microphone audio is bound to the current page
//! origin because the resulting transcript is used to act on that page, and to
//! the exact configured ASR endpoint/profile. Consent is evaluated before the
//! provider dispatch; a consent challenge stores metadata only, never audio.

use sha2::{Digest, Sha256};

use crate::app_core::planner_redaction::high_risk_page_context_reason;
use crate::app_core::remote_privacy_api::page_origin_from_url;
use crate::asr::{DEFAULT_TRANSCRIBE_DURATION_MS, MAX_TRANSCRIBE_DURATION_MS};
use crate::commands::{
    current_timestamp_ms, RemotePlannerConsentDecision, RemotePlannerConsentResponseOutcome,
    ToolError, TranscribeCommandInput,
};
use crate::config::{PersistedOriginDecision, ProviderMode};
use crate::provider_endpoint::ProviderEndpointScope;

use super::challenge::build_microphone_consent_challenge;
use super::errors::{consent_error, policy_block_error_for};
use super::grants::RemoteDataDisclosureKind;
use super::origin_rules::disclosure_kind_label;
use super::types::{
    MicrophoneConsentResolution, MicrophonePreparation, MicrophoneRequestDraft,
    PendingMicrophoneConsent, RemoteMicrophoneAuthorization,
};
use super::{evaluate_remote_planner_policy, RemotePlannerPolicyResult, SESSION_GRANT_TTL_MS};

const KIND: RemoteDataDisclosureKind = RemoteDataDisclosureKind::MicrophoneAudio;

fn effective_duration_ms(input: &TranscribeCommandInput) -> u64 {
    let requested = input
        .max_duration_ms
        .unwrap_or(DEFAULT_TRANSCRIBE_DURATION_MS);
    let mut effective = requested.min(MAX_TRANSCRIBE_DURATION_MS);
    if let Some(timeout_ms) = input.timeout_ms {
        effective = effective.min(timeout_ms.max(1));
    }
    effective
}

fn request_binding_digest(
    page_origin: &str,
    endpoint: &str,
    profile_name: &str,
    model_label: &str,
    input: &TranscribeCommandInput,
    duration_ms: u64,
) -> String {
    let stop_mode = if input.stop_mode.auto_stops() {
        "auto_stop"
    } else {
        "keep_listening"
    };
    let material = format!(
        "microphone-audio-v1\n{page_origin}\n{endpoint}\n{profile_name}\n{model_label}\n{duration_ms}\n{stop_mode}"
    );
    format!("{:x}", Sha256::digest(material.as_bytes()))
}

impl super::super::AppCore {
    /// Gate a transcription before microphone audio can reach a remote ASR
    /// endpoint. Local ASR returns `Authorized` immediately and never creates
    /// consent state.
    pub(crate) fn prepare_microphone_transcription(
        &mut self,
        input: &TranscribeCommandInput,
    ) -> Result<MicrophonePreparation, ToolError> {
        if !matches!(self.config.providers.asr.mode, ProviderMode::Remote) {
            return Ok(MicrophonePreparation::Authorized(None));
        }
        let Some(profile_name) = self.config.providers.asr.remote_profile.clone() else {
            return Err(consent_error(
                "asr_remote_profile_missing",
                "remote asr profile is not configured",
                None,
            ));
        };
        let Some(profile) = self.config.remote_asr_profiles.get(&profile_name).cloned() else {
            return Err(consent_error(
                "asr_remote_profile_missing",
                "the configured remote asr profile was not found",
                None,
            ));
        };
        let endpoint_scope = ProviderEndpointScope::parse(&profile.base_url).map_err(|reason| {
            consent_error(
                "asr_remote_endpoint_invalid",
                "remote asr endpoint is invalid",
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
        let duration_ms = effective_duration_ms(input);
        let endpoint = endpoint_scope.normalized_base_url().to_string();
        let binding_digest = page_origin.as_deref().map(|origin| {
            request_binding_digest(
                origin,
                &endpoint,
                &profile_name,
                &profile.model,
                input,
                duration_ms,
            )
        });

        match evaluate_remote_planner_policy(
            self.origin_rules_settings(KIND),
            &endpoint_scope,
            page_origin.as_deref(),
            high_risk_reason,
            self.ephemeral_grants(KIND),
            now_ms,
        ) {
            RemotePlannerPolicyResult::Allowed(_) => Ok(MicrophonePreparation::Authorized(Some(
                RemoteMicrophoneAuthorization::new(),
            ))),
            RemotePlannerPolicyResult::ConsentRequired => {
                let Some(page_origin) = page_origin else {
                    return Err(consent_error(
                        "remote_data_opaque_origin_blocked",
                        "the current page does not have a supported HTTP(S) origin",
                        None,
                    ));
                };
                let Some(binding_digest) = binding_digest else {
                    return Err(consent_error(
                        "remote_data_consent_internal_error",
                        "microphone request binding could not be created for a valid page origin",
                        None,
                    ));
                };
                if self.consume_once_grant(KIND, &page_origin, &endpoint, &binding_digest, now_ms) {
                    return Ok(MicrophonePreparation::Authorized(Some(
                        RemoteMicrophoneAuthorization::new(),
                    )));
                }
                let draft = MicrophoneRequestDraft {
                    endpoint_scope,
                    profile_name,
                    model_label: profile.model,
                    page_origin,
                    request_binding_digest: binding_digest,
                    runtime_state_token: self.current_runtime_state_token(),
                    effective_duration_ms: duration_ms,
                    input: input.clone(),
                };
                let challenge = build_microphone_consent_challenge(&draft, now_ms)?;
                self.pending_microphone_consent = Some(PendingMicrophoneConsent {
                    challenge: challenge.clone(),
                    page_origin: draft.page_origin,
                    endpoint_scope: draft.endpoint_scope.normalized_base_url().to_string(),
                    request_binding_digest: draft.request_binding_digest,
                    runtime_state_token: draft.runtime_state_token,
                });
                Ok(MicrophonePreparation::ConsentRequired {
                    challenge: Box::new(challenge),
                })
            }
            RemotePlannerPolicyResult::Blocked { code, reason_code } => Err(
                policy_block_error_for(disclosure_kind_label(KIND), code, reason_code),
            ),
        }
    }

    pub(crate) fn resolve_microphone_consent(
        &mut self,
        challenge_id: &str,
        challenge_digest: &str,
        decision: RemotePlannerConsentDecision,
    ) -> Result<MicrophoneConsentResolution, ToolError> {
        let pending = self.pending_microphone_consent.take().ok_or_else(|| {
            consent_error(
                "remote_data_consent_missing",
                "no microphone remote-data consent request is pending",
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
            RemotePlannerConsentDecision::Deny => Ok(MicrophoneConsentResolution::Terminal(
                RemotePlannerConsentResponseOutcome::Denied,
            )),
            RemotePlannerConsentDecision::BlockPersistent => {
                self.persist_origin_rule(
                    KIND,
                    &pending.page_origin,
                    PersistedOriginDecision::Block,
                    None,
                )?;
                self.remote_microphone_ephemeral_grants
                    .retain(|grant| grant.page_origin != pending.page_origin);
                Ok(MicrophoneConsentResolution::Terminal(
                    RemotePlannerConsentResponseOutcome::BlockedPersistent,
                ))
            }
            RemotePlannerConsentDecision::AllowPersistent => {
                self.persist_origin_rule(
                    KIND,
                    &pending.page_origin,
                    PersistedOriginDecision::Allow,
                    Some(pending.endpoint_scope),
                )?;
                Ok(MicrophoneConsentResolution::AuthorizedRetryRequired)
            }
            RemotePlannerConsentDecision::AllowSession => {
                self.install_session_grant(
                    KIND,
                    pending.page_origin,
                    pending.endpoint_scope,
                    now_ms.saturating_add(SESSION_GRANT_TTL_MS),
                );
                Ok(MicrophoneConsentResolution::AuthorizedRetryRequired)
            }
            RemotePlannerConsentDecision::AllowOnce => {
                self.install_once_grant(
                    KIND,
                    pending.page_origin,
                    pending.endpoint_scope,
                    pending.request_binding_digest,
                    pending.challenge.expires_at_ms,
                );
                Ok(MicrophoneConsentResolution::AuthorizedRetryRequired)
            }
        }
    }
}
