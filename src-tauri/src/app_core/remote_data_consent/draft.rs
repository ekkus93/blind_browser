//! Builds a [`RemotePlannerRequestDraft`] from a raw [`PlannerInput`]:
//! sanitizes it via [`crate::app_core::planner_redaction`], computes its
//! payload digest, and summarizes what classes of data it discloses.

use sha2::{Digest, Sha256};

use crate::app_core::planner_redaction::{
    planner_page_origin, sanitize_remote_planner_input_authorized, RemoteDataMode,
    RemotePlannerInput,
};
use crate::commands::{
    PlannerInput, RemotePlannerDisclosureClass, RemotePlannerDisclosureCounts, ToolError,
};
use crate::config::RemotePlannerProfile;
use crate::page_model::RegionSource;
use crate::provider_endpoint::ProviderEndpointScope;

use super::errors::consent_error;
use super::types::{RemotePlannerDataAuthorization, RemotePlannerRequestDraft};

pub(super) fn build_request_draft(
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
        narration_text_bytes: 0,
        microphone_audio_duration_ms: 0,
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

pub(super) fn remote_data_mode(authorization: RemotePlannerDataAuthorization) -> RemoteDataMode {
    if matches!(authorization, RemotePlannerDataAuthorization::Loopback) {
        RemoteDataMode::LoopbackLocalService
    } else {
        RemoteDataMode::NetworkRemoteWithExplicitConsent
    }
}
