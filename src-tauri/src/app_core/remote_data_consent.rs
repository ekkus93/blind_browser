use std::sync::atomic::{AtomicU8, Ordering};

use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::planner_redaction::{
    high_risk_context_reason, high_risk_page_context_reason, planner_page_origin,
    sanitize_remote_planner_input_authorized, RemoteDataMode, RemotePlannerInput,
};
use super::AppCore;
use crate::commands::{
    current_timestamp_ms, PlannerInput, RemotePlannerConsentChallenge,
    RemotePlannerConsentDecision, RemotePlannerConsentResponseOutcome,
    RemotePlannerDisclosureClass, RemotePlannerDisclosureCounts, ToolError,
};
use crate::config::{
    AppConfig, PersistedOriginDecision, RemotePlannerNetworkMode, RemotePlannerOriginRule,
    RemotePlannerPrivacySettings, RemotePlannerProfile, REMOTE_DATA_POLICY_VERSION,
};
use crate::page_model::RegionSource;
use crate::provider_endpoint::ProviderEndpointScope;
use crate::state::PlanningStateSnapshot;

const CONSENT_CHALLENGE_TTL_MS: u64 = 120_000;
const SESSION_GRANT_TTL_MS: u64 = 8 * 60 * 60 * 1_000;
const MAX_EPHEMERAL_GRANTS: usize = 64;

#[derive(Debug, Clone, Serialize)]
struct RemotePlannerConsentManifest {
    challenge_id: String,
    request_id: String,
    page_origin: String,
    endpoint_scope: String,
    profile_name: String,
    model_label: String,
    policy_version: u32,
    disclosure_classes: Vec<RemotePlannerDisclosureClass>,
    disclosure_counts: RemotePlannerDisclosureCounts,
    payload_digest: String,
    runtime_state_token: String,
    expires_at_ms: u64,
}

fn remote_planner_consent_manifest_digest(
    manifest: &RemotePlannerConsentManifest,
) -> Result<String, ToolError> {
    let encoded = serde_json::to_vec(manifest).map_err(|error| {
        consent_error(
            "remote_data_challenge_serialization_failed",
            "remote-data consent challenge could not be bound",
            Some(serde_json::json!({ "reason": error.to_string() })),
        )
    })?;
    Ok(format!("{:x}", Sha256::digest(encoded)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RemotePlannerDataAuthorization {
    Loopback,
    GlobalAllow,
    PersistentAllow,
    SessionAllow,
    AllowOnce,
}

pub(crate) enum EphemeralConsentKind {
    Once {
        challenge_digest: String,
        remaining_uses: AtomicU8,
    },
    Session,
}

pub(crate) struct RemotePlannerEphemeralGrant {
    page_origin: String,
    endpoint_scope: String,
    policy_version: u32,
    expires_at_ms: u64,
    kind: EphemeralConsentKind,
}

impl RemotePlannerEphemeralGrant {
    fn session(
        page_origin: String,
        endpoint_scope: String,
        policy_version: u32,
        expires_at_ms: u64,
    ) -> Self {
        Self {
            page_origin,
            endpoint_scope,
            policy_version,
            expires_at_ms,
            kind: EphemeralConsentKind::Session,
        }
    }

    fn once(
        page_origin: String,
        endpoint_scope: String,
        policy_version: u32,
        challenge_digest: String,
        expires_at_ms: u64,
    ) -> Self {
        Self {
            page_origin,
            endpoint_scope,
            policy_version,
            expires_at_ms,
            kind: EphemeralConsentKind::Once {
                challenge_digest,
                remaining_uses: AtomicU8::new(1),
            },
        }
    }

    fn is_matching_session(&self, page_origin: &str, endpoint_scope: &str, now_ms: u64) -> bool {
        self.expires_at_ms > now_ms
            && self.page_origin == page_origin
            && self.endpoint_scope == endpoint_scope
            && self.policy_version == REMOTE_DATA_POLICY_VERSION
            && matches!(&self.kind, EphemeralConsentKind::Session)
    }

    fn consume_matching_once(
        &self,
        page_origin: &str,
        endpoint_scope: &str,
        challenge_digest: &str,
        now_ms: u64,
    ) -> bool {
        if self.expires_at_ms <= now_ms
            || self.page_origin != page_origin
            || self.endpoint_scope != endpoint_scope
            || self.policy_version != REMOTE_DATA_POLICY_VERSION
        {
            return false;
        }
        let EphemeralConsentKind::Once {
            challenge_digest: expected_digest,
            remaining_uses,
        } = &self.kind
        else {
            return false;
        };
        if expected_digest != challenge_digest {
            return false;
        }
        remaining_uses
            .compare_exchange(1, 0, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RemotePlannerPolicyResult {
    Allowed(RemotePlannerDataAuthorization),
    ConsentRequired,
    Blocked {
        code: &'static str,
        reason_code: &'static str,
    },
}

pub(crate) fn evaluate_remote_planner_policy(
    privacy: &RemotePlannerPrivacySettings,
    endpoint_scope: &ProviderEndpointScope,
    page_origin: Option<&str>,
    high_risk_reason: Option<&'static str>,
    grants: &[RemotePlannerEphemeralGrant],
    now_ms: u64,
) -> RemotePlannerPolicyResult {
    if endpoint_scope.is_loopback() {
        return RemotePlannerPolicyResult::Allowed(RemotePlannerDataAuthorization::Loopback);
    }
    if matches!(privacy.network_mode, RemotePlannerNetworkMode::LocalOnly) {
        return RemotePlannerPolicyResult::Blocked {
            code: "remote_data_local_only",
            reason_code: "local_only",
        };
    }
    let Some(page_origin) = page_origin else {
        return RemotePlannerPolicyResult::Blocked {
            code: "remote_data_opaque_origin_blocked",
            reason_code: "origin_unavailable",
        };
    };
    if let Some(reason_code) = high_risk_reason {
        return RemotePlannerPolicyResult::Blocked {
            code: "remote_data_high_risk_blocked",
            reason_code,
        };
    }
    if privacy.origin_rules.iter().any(|rule| {
        rule.page_origin == page_origin && matches!(rule.decision, PersistedOriginDecision::Block)
    }) {
        return RemotePlannerPolicyResult::Blocked {
            code: "remote_data_origin_blocked",
            reason_code: "origin_block",
        };
    }
    let endpoint = endpoint_scope.normalized_base_url();
    if grants
        .iter()
        .any(|grant| grant.is_matching_session(page_origin, endpoint, now_ms))
    {
        return RemotePlannerPolicyResult::Allowed(RemotePlannerDataAuthorization::SessionAllow);
    }
    if privacy.origin_rules.iter().any(|rule| {
        rule.page_origin == page_origin
            && matches!(rule.decision, PersistedOriginDecision::Allow)
            && rule.endpoint_scope.as_deref() == Some(endpoint)
            && rule.policy_version == REMOTE_DATA_POLICY_VERSION
    }) {
        return RemotePlannerPolicyResult::Allowed(RemotePlannerDataAuthorization::PersistentAllow);
    }
    if matches!(
        privacy.network_mode,
        RemotePlannerNetworkMode::AllowSanitizedNonHighRisk
    ) {
        return RemotePlannerPolicyResult::Allowed(RemotePlannerDataAuthorization::GlobalAllow);
    }
    RemotePlannerPolicyResult::ConsentRequired
}

pub(crate) struct RemotePlannerRequestDraft {
    pub(crate) profile_name: String,
    pub(crate) profile: RemotePlannerProfile,
    pub(crate) endpoint_scope: ProviderEndpointScope,
    pub(crate) sanitized_input: Box<RemotePlannerInput>,
    pub(crate) payload_digest: String,
    pub(crate) page_origin: String,
    pub(crate) runtime_state_token: String,
    pub(crate) disclosure_classes: Vec<RemotePlannerDisclosureClass>,
    pub(crate) disclosure_counts: RemotePlannerDisclosureCounts,
}

pub(crate) struct PreparedRemotePlannerRequest {
    pub(crate) profile_name: String,
    pub(crate) profile: RemotePlannerProfile,
    pub(crate) endpoint_scope: ProviderEndpointScope,
    pub(crate) sanitized_input: RemotePlannerInput,
    pub(crate) authorization: RemotePlannerDataAuthorization,
    pub(crate) page_origin: String,
    pub(crate) runtime_state_token: String,
}

impl RemotePlannerRequestDraft {
    fn authorize(
        self,
        authorization: RemotePlannerDataAuthorization,
    ) -> PreparedRemotePlannerRequest {
        PreparedRemotePlannerRequest {
            profile_name: self.profile_name,
            profile: self.profile,
            endpoint_scope: self.endpoint_scope,
            sanitized_input: *self.sanitized_input,
            authorization,
            page_origin: self.page_origin,
            runtime_state_token: self.runtime_state_token,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PendingRemotePlannerContinuation {
    ResolveOnly,
    Execute,
}

pub(crate) struct PendingRemotePlannerConsent {
    pub(crate) challenge: RemotePlannerConsentChallenge,
    draft: RemotePlannerRequestDraft,
    planning_snapshot: PlanningStateSnapshot,
    continuation: PendingRemotePlannerContinuation,
}

pub(crate) enum RemotePlannerPreparation {
    Authorized(Box<PreparedRemotePlannerRequest>),
    ConsentRequired {
        challenge: Box<RemotePlannerConsentChallenge>,
        draft: Box<RemotePlannerRequestDraft>,
    },
}

pub(crate) struct AuthorizedPendingRemotePlannerRequest {
    pub(crate) prepared: PreparedRemotePlannerRequest,
    pub(crate) planning_snapshot: PlanningStateSnapshot,
    pub(crate) continuation: PendingRemotePlannerContinuation,
    pub(crate) request_id: String,
}

pub(crate) enum PendingConsentResolution {
    Terminal(RemotePlannerConsentResponseOutcome),
    Authorized(Box<AuthorizedPendingRemotePlannerRequest>),
}

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
        self.prune_remote_planner_grants(now_ms);
        let page_origin = planner_page_origin(&planner_input);
        let high_risk_reason = high_risk_context_reason(&planner_input);
        match evaluate_remote_planner_policy(
            privacy,
            &endpoint_scope,
            page_origin.as_deref(),
            high_risk_reason,
            &self.remote_planner_ephemeral_grants,
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
            &self.remote_planner_ephemeral_grants,
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
                Ok(PendingConsentResolution::Authorized(Box::new(
                    AuthorizedPendingRemotePlannerRequest {
                        request_id: pending.challenge.request_id,
                        prepared: pending
                            .draft
                            .authorize(RemotePlannerDataAuthorization::PersistentAllow),
                        planning_snapshot: pending.planning_snapshot,
                        continuation: pending.continuation,
                    },
                )))
            }
            RemotePlannerConsentDecision::AllowSession => {
                self.install_session_grant(
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
                    pending.draft.page_origin.clone(),
                    endpoint.clone(),
                    pending.challenge.challenge_digest.clone(),
                    pending.challenge.expires_at_ms,
                );
                if !self.consume_once_grant(
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

    fn prune_remote_planner_grants(&mut self, now_ms: u64) {
        self.remote_planner_ephemeral_grants
            .retain(|grant| grant.expires_at_ms > now_ms);
    }

    fn install_session_grant(
        &mut self,
        page_origin: String,
        endpoint_scope: String,
        expires_at_ms: u64,
    ) {
        self.remote_planner_ephemeral_grants.retain(|grant| {
            !(grant.page_origin == page_origin
                && grant.endpoint_scope == endpoint_scope
                && matches!(&grant.kind, EphemeralConsentKind::Session))
        });
        self.remote_planner_ephemeral_grants
            .push(RemotePlannerEphemeralGrant::session(
                page_origin,
                endpoint_scope,
                REMOTE_DATA_POLICY_VERSION,
                expires_at_ms,
            ));
        self.bound_remote_planner_grants();
    }

    fn install_once_grant(
        &mut self,
        page_origin: String,
        endpoint_scope: String,
        challenge_digest: String,
        expires_at_ms: u64,
    ) {
        self.remote_planner_ephemeral_grants
            .push(RemotePlannerEphemeralGrant::once(
                page_origin,
                endpoint_scope,
                REMOTE_DATA_POLICY_VERSION,
                challenge_digest,
                expires_at_ms,
            ));
        self.bound_remote_planner_grants();
    }

    fn consume_once_grant(
        &self,
        page_origin: &str,
        endpoint_scope: &str,
        challenge_digest: &str,
        now_ms: u64,
    ) -> bool {
        self.remote_planner_ephemeral_grants.iter().any(|grant| {
            grant.consume_matching_once(page_origin, endpoint_scope, challenge_digest, now_ms)
        })
    }

    fn bound_remote_planner_grants(&mut self) {
        if self.remote_planner_ephemeral_grants.len() > MAX_EPHEMERAL_GRANTS {
            let remove = self.remote_planner_ephemeral_grants.len() - MAX_EPHEMERAL_GRANTS;
            self.remote_planner_ephemeral_grants.drain(0..remove);
        }
    }

    fn persist_origin_rule(
        &mut self,
        page_origin: &str,
        decision: PersistedOriginDecision,
        endpoint_scope: Option<String>,
    ) -> Result<(), ToolError> {
        let mut settings = self.config.remote_planner_privacy.clone();
        if matches!(decision, PersistedOriginDecision::Allow)
            && settings.origin_rules.iter().any(|rule| {
                rule.page_origin == page_origin
                    && matches!(rule.decision, PersistedOriginDecision::Block)
            })
        {
            return Err(policy_block_error(
                "remote_data_origin_blocked",
                "origin_block",
            ));
        }
        if matches!(decision, PersistedOriginDecision::Block) {
            settings
                .origin_rules
                .retain(|rule| rule.page_origin != page_origin);
        } else {
            settings.origin_rules.retain(|rule| {
                !(rule.page_origin == page_origin
                    && rule.decision == decision
                    && rule.endpoint_scope == endpoint_scope)
            });
        }
        settings.origin_rules.push(RemotePlannerOriginRule {
            page_origin: page_origin.to_string(),
            decision,
            endpoint_scope,
            policy_version: REMOTE_DATA_POLICY_VERSION,
            created_at_ms: current_timestamp_ms(),
        });
        settings.policy_schema_version = REMOTE_DATA_POLICY_VERSION;
        self.config =
            AppConfig::persist_remote_planner_privacy_settings_for_app(&self.app_handle, &settings)
                .map_err(|error| {
                    consent_error(
                        "remote_data_consent_persist_failed",
                        "remote-data consent decision could not be persisted",
                        Some(serde_json::json!({ "reason": error.to_string() })),
                    )
                })?;
        Ok(())
    }
}

fn build_request_draft(
    profile_name: String,
    profile: RemotePlannerProfile,
    endpoint_scope: ProviderEndpointScope,
    planner_input: PlannerInput,
    mode: RemoteDataMode,
) -> Result<RemotePlannerRequestDraft, ToolError> {
    let page_origin = planner_page_origin(&planner_input).ok_or_else(|| {
        consent_error(
            "remote_data_opaque_origin_blocked",
            "the current page does not have a supported HTTP(S) origin",
            None,
        )
    })?;
    let runtime_state_token = planner_input.runtime_state_token.clone();
    let sanitized_input = sanitize_remote_planner_input_authorized(&planner_input, mode)?;
    let encoded = serde_json::to_vec(&sanitized_input).map_err(|error| {
        consent_error(
            "remote_data_payload_serialization_failed",
            "sanitized remote planner payload could not be serialized",
            Some(serde_json::json!({ "reason": error.to_string() })),
        )
    })?;
    let payload_digest = format!("{:x}", Sha256::digest(&encoded));
    let (disclosure_classes, disclosure_counts) =
        disclosure_summary(&sanitized_input, encoded.len());
    Ok(RemotePlannerRequestDraft {
        profile_name,
        profile,
        endpoint_scope,
        sanitized_input: Box::new(sanitized_input),
        payload_digest,
        page_origin,
        runtime_state_token,
        disclosure_classes,
        disclosure_counts,
    })
}

fn disclosure_summary(
    input: &RemotePlannerInput,
    sanitized_serialized_bytes: usize,
) -> (
    Vec<RemotePlannerDisclosureClass>,
    RemotePlannerDisclosureCounts,
) {
    let selected_region_count = input
        .untrusted_data
        .page_model
        .as_ref()
        .map(|page| page.regions.len())
        .unwrap_or(0);
    let model_elements = input
        .untrusted_data
        .page_model
        .as_ref()
        .map(|page| page.interactive_elements.len())
        .unwrap_or(0);
    let snapshot_elements = input
        .untrusted_data
        .page_snapshot
        .as_ref()
        .map(|page| page.interactive_elements.len())
        .unwrap_or(0);
    let selected_element_count = model_elements.saturating_add(snapshot_elements);
    let ocr_derived_region_count = input
        .untrusted_data
        .page_model
        .as_ref()
        .map(|page| {
            page.regions
                .iter()
                .filter(|region| matches!(region.source, RegionSource::Ocr | RegionSource::Mixed))
                .count()
        })
        .unwrap_or(0);
    let tool_history_count = input.untrusted_data.recent_tool_results.len();
    let skill_summary_count = input.untrusted_data.relevant_skill_summaries.len();
    let counts = RemotePlannerDisclosureCounts {
        selected_region_count,
        selected_element_count,
        ocr_derived_region_count,
        tool_history_count,
        skill_summary_count,
        sanitized_serialized_bytes,
    };
    let mut classes = vec![
        RemotePlannerDisclosureClass::UserTranscript,
        RemotePlannerDisclosureClass::PageOrigin,
        RemotePlannerDisclosureClass::TrustedRuntimeContracts,
    ];
    if selected_region_count > 0 {
        classes.push(RemotePlannerDisclosureClass::SelectedPageRegions);
    }
    if selected_element_count > 0 {
        classes.push(RemotePlannerDisclosureClass::SelectedElementMetadata);
    }
    if ocr_derived_region_count > 0 {
        classes.push(RemotePlannerDisclosureClass::OcrDerivedRegions);
    }
    if tool_history_count > 0 {
        classes.push(RemotePlannerDisclosureClass::ToolObservationSummaries);
    }
    if skill_summary_count > 0 {
        classes.push(RemotePlannerDisclosureClass::SkillSummaries);
    }
    classes.sort();
    (classes, counts)
}

fn build_consent_challenge(
    draft: &RemotePlannerRequestDraft,
    now_ms: u64,
) -> Result<RemotePlannerConsentChallenge, ToolError> {
    let challenge_id = Uuid::new_v4().to_string();
    let expires_at_ms = now_ms.saturating_add(CONSENT_CHALLENGE_TTL_MS);
    let endpoint = draft.endpoint_scope.normalized_base_url().to_string();
    let manifest = RemotePlannerConsentManifest {
        challenge_id: challenge_id.clone(),
        request_id: draft.sanitized_input.trusted_runtime.request_id.clone(),
        page_origin: draft.page_origin.clone(),
        endpoint_scope: endpoint.clone(),
        profile_name: draft.profile_name.clone(),
        model_label: draft.profile.model.clone(),
        policy_version: REMOTE_DATA_POLICY_VERSION,
        disclosure_classes: draft.disclosure_classes.clone(),
        disclosure_counts: draft.disclosure_counts.clone(),
        payload_digest: draft.payload_digest.clone(),
        runtime_state_token: draft.runtime_state_token.clone(),
        expires_at_ms,
    };
    let challenge_digest = remote_planner_consent_manifest_digest(&manifest)?;
    Ok(RemotePlannerConsentChallenge {
        challenge_id,
        challenge_digest,
        request_id: manifest.request_id,
        page_origin: manifest.page_origin,
        endpoint_display: endpoint.clone(),
        endpoint_scope: endpoint,
        profile_name: manifest.profile_name,
        model_label: manifest.model_label,
        policy_version: manifest.policy_version,
        disclosure_classes: manifest.disclosure_classes,
        disclosure_counts: manifest.disclosure_counts,
        expires_at_ms,
        allow_once: true,
        allow_session: true,
        allow_persistent: true,
        block_persistent: true,
    })
}

fn remote_data_mode(authorization: RemotePlannerDataAuthorization) -> RemoteDataMode {
    if matches!(authorization, RemotePlannerDataAuthorization::Loopback) {
        RemoteDataMode::LoopbackLocalService
    } else {
        RemoteDataMode::NetworkRemoteWithExplicitConsent
    }
}

fn policy_block_error(code: &str, reason_code: &str) -> ToolError {
    ToolError {
        code: code.to_string(),
        message: match code {
            "remote_data_local_only" => {
                String::from("Local-only planner mode blocks non-loopback planner endpoints.")
            }
            "remote_data_high_risk_blocked" => String::from(
                "Network planning is blocked for this high-risk page context. Use direct commands or a loopback local planner.",
            ),
            "remote_data_origin_blocked" => String::from(
                "This page origin is configured to remain local for every network planner.",
            ),
            _ => String::from(
                "The current page origin cannot be safely authorized for network planning.",
            ),
        },
        retryable: false,
        details: Some(serde_json::json!({
            "policy": code,
            "reason_code": reason_code,
        })),
    }
}

fn consent_error(code: &str, message: &str, details: Option<serde_json::Value>) -> ToolError {
    ToolError {
        code: code.to_string(),
        message: message.to_string(),
        retryable: false,
        details,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{HighRiskOriginPolicy, RemotePlannerOriginRule};

    fn endpoint(raw: &str) -> ProviderEndpointScope {
        ProviderEndpointScope::parse(raw).expect("test endpoint must be valid")
    }

    fn settings(mode: RemotePlannerNetworkMode) -> RemotePlannerPrivacySettings {
        RemotePlannerPrivacySettings {
            network_mode: mode,
            high_risk_origin_policy: HighRiskOriginPolicy::Block,
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
