//! Builds the [`RemotePlannerConsentChallenge`] shown to the user when
//! [`super::policy`] returns `ConsentRequired`: a tamper-evident digest over
//! every semantic field the user is asked to approve, so a stale or altered
//! challenge can never be replayed against a changed request.

use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::commands::{
    RemotePlannerConsentChallenge, RemotePlannerDisclosureClass, RemotePlannerDisclosureCounts,
    ToolError,
};
use crate::config::REMOTE_DATA_POLICY_VERSION;

use super::errors::consent_error;
use super::types::RemotePlannerRequestDraft;

const CONSENT_CHALLENGE_TTL_MS: u64 = 120_000;

#[derive(Debug, Clone, Serialize)]
pub(super) struct RemotePlannerConsentManifest {
    pub(super) challenge_id: String,
    pub(super) request_id: String,
    pub(super) page_origin: String,
    pub(super) endpoint_scope: String,
    pub(super) profile_name: String,
    pub(super) model_label: String,
    pub(super) policy_version: u32,
    pub(super) disclosure_classes: Vec<RemotePlannerDisclosureClass>,
    pub(super) disclosure_counts: RemotePlannerDisclosureCounts,
    pub(super) payload_digest: String,
    pub(super) runtime_state_token: String,
    pub(super) expires_at_ms: u64,
}

pub(super) fn remote_planner_consent_manifest_digest(
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

pub(super) fn build_consent_challenge(
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
