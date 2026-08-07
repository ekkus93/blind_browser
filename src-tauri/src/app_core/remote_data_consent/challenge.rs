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
use super::types::{NarrationRequestDraft, RemotePlannerRequestDraft};

const CONSENT_CHALLENGE_TTL_MS: u64 = 120_000;

/// The fields every disclosure kind's draft can supply to build a challenge.
/// None of these are planner-specific in type -- extracted so
/// [`build_consent_challenge_from_fields`] is the single shared construction
/// point for all three kinds, rather than three near-identical copies of the
/// digest/manifest-building logic.
pub(super) struct ChallengeFields {
    pub(super) request_id: String,
    pub(super) page_origin: String,
    pub(super) endpoint: String,
    pub(super) profile_name: String,
    pub(super) model_label: String,
    pub(super) disclosure_classes: Vec<RemotePlannerDisclosureClass>,
    pub(super) disclosure_counts: RemotePlannerDisclosureCounts,
    pub(super) payload_digest: String,
    pub(super) runtime_state_token: String,
}

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
    build_consent_challenge_from_fields(
        ChallengeFields {
            request_id: draft.sanitized_input.trusted_runtime.request_id.clone(),
            page_origin: draft.page_origin.clone(),
            endpoint: draft.endpoint_scope.normalized_base_url().to_string(),
            profile_name: draft.profile_name.clone(),
            model_label: draft.profile.model.clone(),
            disclosure_classes: draft.disclosure_classes.clone(),
            disclosure_counts: draft.disclosure_counts.clone(),
            payload_digest: draft.payload_digest.clone(),
            runtime_state_token: draft.runtime_state_token.clone(),
        },
        now_ms,
    )
}

pub(super) fn build_narration_consent_challenge(
    draft: &NarrationRequestDraft,
    request_id: String,
    now_ms: u64,
) -> Result<RemotePlannerConsentChallenge, ToolError> {
    build_consent_challenge_from_fields(
        ChallengeFields {
            request_id,
            page_origin: draft.page_origin.clone(),
            endpoint: draft.endpoint_scope.normalized_base_url().to_string(),
            profile_name: draft.profile_name.clone(),
            model_label: draft.model_label.clone(),
            disclosure_classes: vec![RemotePlannerDisclosureClass::NarrationText],
            disclosure_counts: RemotePlannerDisclosureCounts {
                narration_text_bytes: draft.text.len(),
                ..RemotePlannerDisclosureCounts::default()
            },
            payload_digest: draft.payload_digest.clone(),
            runtime_state_token: draft.runtime_state_token.clone(),
        },
        now_ms,
    )
}

fn build_consent_challenge_from_fields(
    fields: ChallengeFields,
    now_ms: u64,
) -> Result<RemotePlannerConsentChallenge, ToolError> {
    let challenge_id = Uuid::new_v4().to_string();
    let expires_at_ms = now_ms.saturating_add(CONSENT_CHALLENGE_TTL_MS);
    let manifest = RemotePlannerConsentManifest {
        challenge_id: challenge_id.clone(),
        request_id: fields.request_id,
        page_origin: fields.page_origin,
        endpoint_scope: fields.endpoint.clone(),
        profile_name: fields.profile_name,
        model_label: fields.model_label,
        policy_version: REMOTE_DATA_POLICY_VERSION,
        disclosure_classes: fields.disclosure_classes,
        disclosure_counts: fields.disclosure_counts,
        payload_digest: fields.payload_digest,
        runtime_state_token: fields.runtime_state_token,
        expires_at_ms,
    };
    let challenge_digest = remote_planner_consent_manifest_digest(&manifest)?;
    Ok(RemotePlannerConsentChallenge {
        challenge_id,
        challenge_digest,
        request_id: manifest.request_id,
        page_origin: manifest.page_origin,
        endpoint_display: fields.endpoint.clone(),
        endpoint_scope: fields.endpoint,
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
