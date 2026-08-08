//! Entry points for the remote-planner consent workflow: prepares a request
//! (deciding via [`policy`] whether it's allowed, blocked, or needs a
//! challenge), stores a pending challenge, and resolves the user's response.
//! See `docs/BLIND_BROWSER_REMOTE_DATA_CONSENT_ORIGIN_PRIVACY_*` for the
//! consent policy this module implements.
//!
//! Module layout:
//! - [`types`] — workflow shapes (draft, prepared request, pending consent, ...)
//! - [`policy`] — the allow/block/consent-required decision (shared verbatim,
//!   with no planner-specific types in its signature, by every disclosure
//!   kind below)
//! - [`grants`] — ephemeral session/one-shot consent grants, and the
//!   [`RemoteDataDisclosureKind`] selector each kind's state is keyed by
//! - [`origin_rules`] — durable per-origin allow/block persistence
//! - [`draft`] — sanitizes a `PlannerInput` into a `RemotePlannerRequestDraft`
//! - [`challenge`] — builds the tamper-evident consent challenge shown to the user
//! - [`errors`] — shared `ToolError` constructors
//! - [`narration_consent`] — the narration (remote TTS) disclosure kind
//! - [`microphone_consent`] — microphone audio sent to remote ASR
//!
//! All network speech disclosure is fail-closed here before provider dispatch.
//! Local TTS/ASR never enters this policy path.

use crate::app_core::planner_redaction::{
    high_risk_context_reason, high_risk_page_context_reason, planner_page_origin, RemoteDataMode,
};
use crate::commands::{
    current_timestamp_ms, PlannerInput, RemotePlannerConsentChallenge,
    RemotePlannerConsentDecision, RemotePlannerConsentResponseOutcome, ToolError,
};
#[cfg(test)]
use crate::commands::{RemotePlannerDisclosureClass, RemotePlannerDisclosureCounts};
use crate::config::{PersistedOriginDecision, RemotePlannerPrivacySettings, RemotePlannerProfile};
#[cfg(test)]
use crate::config::{RemotePlannerNetworkMode, REMOTE_DATA_POLICY_VERSION};
use crate::provider_endpoint::ProviderEndpointScope;
use crate::state::PlanningStateSnapshot;

use super::AppCore;

mod challenge;
mod draft;
mod errors;
mod grants;
mod microphone_consent;
mod narration_consent;
mod origin_rules;
mod policy;
mod types;

pub(crate) use grants::{RemoteDataDisclosureKind, RemotePlannerEphemeralGrant};
pub(crate) use policy::{evaluate_remote_planner_policy, RemotePlannerPolicyResult};
pub(crate) use types::*;

use challenge::build_consent_challenge;
use draft::{build_request_draft, remote_data_mode};
use errors::{consent_error, policy_block_error};
use grants::SESSION_GRANT_TTL_MS;

#[cfg(test)]
use challenge::{remote_planner_consent_manifest_digest, RemotePlannerConsentManifest};

impl AppCore {
    pub(crate) fn prepare_remote_planner_request(
        &mut self,
        profile_name: String,
        profile: RemotePlannerProfile,
        planner_input: PlannerInput,
        privacy: &RemotePlannerPrivacySettings,
    ) -> Result<RemotePlannerPreparation, ToolError> {
        let endpoint_scope = ProviderEndpointScope::parse(&profile.base_url).map_err(|reason| {
            consent_error(
                "planner_endpoint_invalid",
                "remote planner endpoint is invalid",
                Some(serde_json::json!({ "reason": reason })),
            )
        })?;
        let now_ms = current_timestamp_ms();
        self.prune_remote_planner_grants(RemoteDataDisclosureKind::PlannerPayload, now_ms);
        let page_origin = planner_page_origin(&planner_input);
        let high_risk_reason = high_risk_context_reason(&planner_input);
        match evaluate_remote_planner_policy(
            privacy,
            &endpoint_scope,
            page_origin.as_deref(),
            high_risk_reason,
            self.ephemeral_grants(RemoteDataDisclosureKind::PlannerPayload),
            now_ms,
        ) {
            RemotePlannerPolicyResult::Allowed(authorization) => {
                let mode = remote_data_mode(authorization);
                let draft = build_request_draft(
                    profile_name,
                    profile,
                    endpoint_scope,
                    planner_input,
                    mode,
                )?;
                Ok(RemotePlannerPreparation::Authorized(Box::new(
                    draft.authorize(authorization),
                )))
            }
            RemotePlannerPolicyResult::ConsentRequired => {
                let draft = build_request_draft(
                    profile_name,
                    profile,
                    endpoint_scope,
                    planner_input,
                    RemoteDataMode::NetworkRemoteWithExplicitConsent,
                )?;
                let challenge = build_consent_challenge(&draft, now_ms)?;
                Ok(RemotePlannerPreparation::ConsentRequired {
                    challenge: Box::new(challenge),
                    draft: Box::new(draft),
                })
            }
            RemotePlannerPolicyResult::Blocked { code, reason_code } => {
                Err(policy_block_error(code, reason_code))
            }
        }
    }

    pub(crate) fn store_pending_remote_planner_consent(
        &mut self,
        challenge: RemotePlannerConsentChallenge,
        draft: RemotePlannerRequestDraft,
        planning_snapshot: PlanningStateSnapshot,
        continuation: PendingRemotePlannerContinuation,
    ) {
        self.pending_remote_planner_consent = Some(PendingRemotePlannerConsent {
            challenge,
            draft,
            planning_snapshot,
            continuation,
        });
    }

    pub(crate) fn resolve_pending_remote_planner_consent(
        &mut self,
        challenge_id: &str,
        challenge_digest: &str,
        decision: RemotePlannerConsentDecision,
    ) -> Result<PendingConsentResolution, ToolError> {
        let pending = self.pending_remote_planner_consent.take().ok_or_else(|| {
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
        if pending.draft.runtime_state_token != self.current_runtime_state_token() {
            return Err(consent_error(
                "remote_data_consent_state_changed",
                "runtime state changed after the remote-data consent challenge was created",
                None,
            ));
        }
        let (current_profile_name, current_profile) = self.remote_planner_profile_snapshot()?;
        if current_profile_name != pending.draft.profile_name
            || current_profile.base_url != pending.draft.profile.base_url
            || current_profile.model != pending.draft.profile.model
        {
            return Err(consent_error(
                "remote_data_consent_destination_changed",
                "remote planner destination changed after the consent challenge was created",
                None,
            ));
        }
        let current_high_risk_reason = high_risk_page_context_reason(
            self.state
                .current_page
                .as_ref()
                .and_then(|page| page.url.as_deref()),
            self.state.current_page.as_ref(),
            None,
            &[],
        );
        match evaluate_remote_planner_policy(
            &self.config.remote_planner_privacy,
            &pending.draft.endpoint_scope,
            Some(&pending.draft.page_origin),
            current_high_risk_reason,
            self.ephemeral_grants(RemoteDataDisclosureKind::PlannerPayload),
            now_ms,
        ) {
            RemotePlannerPolicyResult::Blocked { code, reason_code } => {
                return Err(policy_block_error(code, reason_code));
            }
            RemotePlannerPolicyResult::Allowed(_) | RemotePlannerPolicyResult::ConsentRequired => {}
        }

        match decision {
            RemotePlannerConsentDecision::Deny => Ok(PendingConsentResolution::Terminal(
                RemotePlannerConsentResponseOutcome::Denied,
            )),
            RemotePlannerConsentDecision::BlockPersistent => {
                self.persist_origin_rule(
                    RemoteDataDisclosureKind::PlannerPayload,
                    &pending.draft.page_origin,
                    PersistedOriginDecision::Block,
                    None,
                )?;
                self.remote_planner_ephemeral_grants
                    .retain(|grant| grant.page_origin != pending.draft.page_origin);
                Ok(PendingConsentResolution::Terminal(
                    RemotePlannerConsentResponseOutcome::BlockedPersistent,
                ))
            }
            RemotePlannerConsentDecision::AllowPersistent => {
                self.persist_origin_rule(
                    RemoteDataDisclosureKind::PlannerPayload,
                    &pending.draft.page_origin,
                    PersistedOriginDecision::Allow,
                    Some(
                        pending
                            .draft
                            .endpoint_scope
                            .normalized_base_url()
                            .to_string(),
                    ),
                )?;
                // CR3 P2.6: re-capture the planning snapshot *after*
                // `persist_origin_rule` above, not before. `persist_origin_rule`
                // writes a new rule into `remote_planner_privacy`, which is part
                // of `relevant_config_fingerprint` (see `planning_snapshot.rs`
                // for why that field must stay in the fingerprint). Handing back
                // `pending.planning_snapshot` -- captured when the challenge was
                // first drafted, before this write -- meant the fingerprint
                // bound to the eventual `PlannerOutput` could never match the
                // one observed when `execute_planner_output` re-validated it,
                // so the plan spuriously came back `NeedsReplan` on every
                // persistent-allow. Re-capturing here binds the snapshot to the
                // runtime state as it actually is at the moment this
                // authorization is granted.
                Ok(PendingConsentResolution::Authorized(Box::new(
                    AuthorizedPendingRemotePlannerRequest {
                        request_id: pending.challenge.request_id,
                        prepared: pending
                            .draft
                            .authorize(RemotePlannerDataAuthorization::PersistentAllow),
                        planning_snapshot: self.capture_planning_state_snapshot(),
                        continuation: pending.continuation,
                    },
                )))
            }
            RemotePlannerConsentDecision::AllowSession => {
                self.install_session_grant(
                    RemoteDataDisclosureKind::PlannerPayload,
                    pending.draft.page_origin.clone(),
                    pending
                        .draft
                        .endpoint_scope
                        .normalized_base_url()
                        .to_string(),
                    now_ms.saturating_add(SESSION_GRANT_TTL_MS),
                );
                Ok(PendingConsentResolution::Authorized(Box::new(
                    AuthorizedPendingRemotePlannerRequest {
                        request_id: pending.challenge.request_id,
                        prepared: pending
                            .draft
                            .authorize(RemotePlannerDataAuthorization::SessionAllow),
                        planning_snapshot: pending.planning_snapshot,
                        continuation: pending.continuation,
                    },
                )))
            }
            RemotePlannerConsentDecision::AllowOnce => {
                let endpoint = pending
                    .draft
                    .endpoint_scope
                    .normalized_base_url()
                    .to_string();
                self.install_once_grant(
                    RemoteDataDisclosureKind::PlannerPayload,
                    pending.draft.page_origin.clone(),
                    endpoint.clone(),
                    pending.challenge.challenge_digest.clone(),
                    pending.challenge.expires_at_ms,
                );
                if !self.consume_once_grant(
                    RemoteDataDisclosureKind::PlannerPayload,
                    &pending.draft.page_origin,
                    &endpoint,
                    &pending.challenge.challenge_digest,
                    now_ms,
                ) {
                    return Err(consent_error(
                        "remote_data_consent_replay",
                        "one-shot remote-data consent was already consumed or expired",
                        None,
                    ));
                }
                Ok(PendingConsentResolution::Authorized(Box::new(
                    AuthorizedPendingRemotePlannerRequest {
                        request_id: pending.challenge.request_id,
                        prepared: pending
                            .draft
                            .authorize(RemotePlannerDataAuthorization::AllowOnce),
                        planning_snapshot: pending.planning_snapshot,
                        continuation: pending.continuation,
                    },
                )))
            }
        }
    }

    pub(crate) fn clear_remote_planner_consent_runtime(&mut self) {
        self.pending_remote_planner_consent = None;
        self.remote_planner_ephemeral_grants.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RemotePlannerOriginRule;

    fn endpoint(raw: &str) -> ProviderEndpointScope {
        ProviderEndpointScope::parse(raw).expect("test endpoint must be valid")
    }

    fn settings(mode: RemotePlannerNetworkMode) -> RemotePlannerPrivacySettings {
        RemotePlannerPrivacySettings {
            network_mode: mode,
            ..Default::default()
        }
    }

    #[test]
    fn loopback_precedes_network_consent() {
        assert_eq!(
            evaluate_remote_planner_policy(
                &settings(RemotePlannerNetworkMode::LocalOnly),
                &endpoint("http://localhost:11434/v1"),
                None,
                Some("authentication_context"),
                &[],
                1,
            ),
            RemotePlannerPolicyResult::Allowed(RemotePlannerDataAuthorization::Loopback)
        );
    }

    #[test]
    fn local_only_and_high_risk_override_allows() {
        let mut local = settings(RemotePlannerNetworkMode::LocalOnly);
        local.origin_rules.push(RemotePlannerOriginRule {
            page_origin: String::from("https://example.com"),
            decision: PersistedOriginDecision::Allow,
            endpoint_scope: Some(String::from("https://api.example.com/v1")),
            policy_version: REMOTE_DATA_POLICY_VERSION,
            created_at_ms: 1,
        });
        assert!(matches!(
            evaluate_remote_planner_policy(
                &local,
                &endpoint("https://api.example.com/v1"),
                Some("https://example.com"),
                None,
                &[],
                1,
            ),
            RemotePlannerPolicyResult::Blocked {
                code: "remote_data_local_only",
                ..
            }
        ));
        local.network_mode = RemotePlannerNetworkMode::AllowSanitizedNonHighRisk;
        assert!(matches!(
            evaluate_remote_planner_policy(
                &local,
                &endpoint("https://api.example.com/v1"),
                Some("https://example.com"),
                Some("payment_context"),
                &[],
                1,
            ),
            RemotePlannerPolicyResult::Blocked {
                code: "remote_data_high_risk_blocked",
                ..
            }
        ));
    }

    #[test]
    fn origin_block_overrides_broad_allow() {
        let mut privacy = settings(RemotePlannerNetworkMode::AllowSanitizedNonHighRisk);
        privacy.origin_rules.push(RemotePlannerOriginRule {
            page_origin: String::from("https://private.example"),
            decision: PersistedOriginDecision::Block,
            endpoint_scope: None,
            policy_version: REMOTE_DATA_POLICY_VERSION,
            created_at_ms: 1,
        });
        assert!(matches!(
            evaluate_remote_planner_policy(
                &privacy,
                &endpoint("https://api.example.com/v1"),
                Some("https://private.example"),
                None,
                &[],
                1,
            ),
            RemotePlannerPolicyResult::Blocked {
                code: "remote_data_origin_blocked",
                ..
            }
        ));
    }

    #[test]
    fn persistent_allow_is_destination_and_version_bound() {
        let mut privacy = settings(RemotePlannerNetworkMode::AskPerOrigin);
        privacy.origin_rules.push(RemotePlannerOriginRule {
            page_origin: String::from("https://example.com"),
            decision: PersistedOriginDecision::Allow,
            endpoint_scope: Some(String::from("https://api.example.com/v1")),
            policy_version: REMOTE_DATA_POLICY_VERSION,
            created_at_ms: 1,
        });
        assert_eq!(
            evaluate_remote_planner_policy(
                &privacy,
                &endpoint("https://api.example.com/v1"),
                Some("https://example.com"),
                None,
                &[],
                1,
            ),
            RemotePlannerPolicyResult::Allowed(RemotePlannerDataAuthorization::PersistentAllow)
        );
        assert_eq!(
            evaluate_remote_planner_policy(
                &privacy,
                &endpoint("https://api.example.com/v2"),
                Some("https://example.com"),
                None,
                &[],
                1,
            ),
            RemotePlannerPolicyResult::ConsentRequired
        );
    }

    #[test]
    fn session_and_one_shot_grants_are_scoped_and_bounded() {
        let session = RemotePlannerEphemeralGrant::session(
            String::from("https://example.com"),
            String::from("https://api.example.com/v1"),
            REMOTE_DATA_POLICY_VERSION,
            100,
        );
        assert!(session.is_matching_session(
            "https://example.com",
            "https://api.example.com/v1",
            99
        ));
        assert!(!session.is_matching_session(
            "https://other.example",
            "https://api.example.com/v1",
            99
        ));
        let once = RemotePlannerEphemeralGrant::once(
            String::from("https://example.com"),
            String::from("https://api.example.com/v1"),
            REMOTE_DATA_POLICY_VERSION,
            String::from("digest"),
            100,
        );
        assert!(once.consume_matching_once(
            "https://example.com",
            "https://api.example.com/v1",
            "digest",
            99
        ));
        assert!(!once.consume_matching_once(
            "https://example.com",
            "https://api.example.com/v1",
            "digest",
            99
        ));
    }

    fn manifest() -> RemotePlannerConsentManifest {
        RemotePlannerConsentManifest {
            challenge_id: String::from("challenge-1"),
            request_id: String::from("request-1"),
            page_origin: String::from("https://example.com"),
            endpoint_scope: String::from("https://api.example.com:443/v1"),
            profile_name: String::from("profile"),
            model_label: String::from("model"),
            policy_version: REMOTE_DATA_POLICY_VERSION,
            disclosure_classes: vec![
                RemotePlannerDisclosureClass::UserTranscript,
                RemotePlannerDisclosureClass::PageOrigin,
            ],
            disclosure_counts: RemotePlannerDisclosureCounts {
                selected_region_count: 1,
                selected_element_count: 2,
                ocr_derived_region_count: 3,
                tool_history_count: 4,
                skill_summary_count: 5,
                sanitized_serialized_bytes: 512,
                narration_text_bytes: 6,
                microphone_audio_duration_ms: 7,
            },
            payload_digest: String::from("payload-digest"),
            runtime_state_token: String::from("runtime-state"),
            expires_at_ms: 123_456,
        }
    }

    #[test]
    fn consent_manifest_digest_binds_every_semantic_field() {
        let baseline = manifest();
        let baseline_digest = remote_planner_consent_manifest_digest(&baseline)
            .expect("baseline manifest should hash");
        assert_eq!(
            baseline_digest,
            remote_planner_consent_manifest_digest(&baseline.clone())
                .expect("equivalent manifest should hash identically")
        );

        type ManifestMutation = Box<dyn Fn(&mut RemotePlannerConsentManifest)>;

        let mutations: Vec<ManifestMutation> = vec![
            Box::new(|value| value.challenge_id.push_str("-changed")),
            Box::new(|value| value.request_id.push_str("-changed")),
            Box::new(|value| value.page_origin = String::from("https://other.example")),
            Box::new(|value| value.endpoint_scope = String::from("https://api.other.example/v1")),
            Box::new(|value| value.profile_name.push_str("-changed")),
            Box::new(|value| value.model_label.push_str("-changed")),
            Box::new(|value| value.policy_version = value.policy_version.saturating_add(1)),
            Box::new(|value| {
                value
                    .disclosure_classes
                    .push(RemotePlannerDisclosureClass::OcrDerivedRegions)
            }),
            Box::new(|value| value.disclosure_counts.selected_region_count += 1),
            Box::new(|value| value.disclosure_counts.selected_element_count += 1),
            Box::new(|value| value.disclosure_counts.ocr_derived_region_count += 1),
            Box::new(|value| value.disclosure_counts.tool_history_count += 1),
            Box::new(|value| value.disclosure_counts.skill_summary_count += 1),
            Box::new(|value| value.disclosure_counts.sanitized_serialized_bytes += 1),
            Box::new(|value| value.disclosure_counts.narration_text_bytes += 1),
            Box::new(|value| value.disclosure_counts.microphone_audio_duration_ms += 1),
            Box::new(|value| value.payload_digest.push_str("-changed")),
            Box::new(|value| value.runtime_state_token.push_str("-changed")),
            Box::new(|value| value.expires_at_ms += 1),
        ];

        for mutate in mutations {
            let mut changed = baseline.clone();
            mutate(&mut changed);
            let changed_digest = remote_planner_consent_manifest_digest(&changed)
                .expect("mutated manifest should hash");
            assert_ne!(baseline_digest, changed_digest);
        }
    }
}
