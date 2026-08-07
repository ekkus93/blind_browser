//! Redacts a [`PlannerInput`] into the trust-boundary-safe shape defined in
//! [`types`] before it may leave the device for a network remote planner.
//! See `docs/BLIND_BROWSER_REMOTE_DATA_CONSENT_ORIGIN_PRIVACY_*` for the
//! consent policy this sanitizer sits behind (enforced in
//! [`super::remote_data_consent`], not here).
//!
//! Module layout:
//! - [`types`] — the sanitized data shapes themselves
//! - [`sanitize`] — field-by-field redaction into those shapes
//! - [`relevance`] — transcript-relevance ranking used to bound payload size
//! - [`sensitive`] — credential/PII-shaped content detection
//! - [`high_risk`] — page-context checks that block network planning outright
//! - [`prompt_injection`] — advisory (never authorizing) injection detection

use crate::commands::PlannerInput;
#[cfg(test)]
use crate::config::RemotePlannerPrivacySettings;
#[cfg(test)]
use crate::provider_endpoint::ProviderEndpointScope;
use crate::resource_limits::{MAX_REGION_TEXT_CHARS, MAX_REMOTE_SKILLS};
#[cfg(test)]
use crate::resource_limits::{MAX_REMOTE_ELEMENTS, MAX_REMOTE_REGIONS};

#[cfg(test)]
use super::remote_data_consent::{evaluate_remote_planner_policy, RemotePlannerPolicyResult};

mod high_risk;
mod prompt_injection;
mod relevance;
mod sanitize;
mod sensitive;
mod types;

pub(crate) use high_risk::{high_risk_context_reason, high_risk_page_context_reason};
pub(crate) use types::*;

use prompt_injection::detect_prompt_injection;
use sanitize::{
    sanitize_agent_state, sanitize_history, sanitize_page_model, sanitize_page_snapshot,
    sanitize_skills, sanitize_text, truncate_identifier,
};
#[cfg(test)]
use sanitize::{sanitize_interactive_element, sanitize_url};

#[cfg(test)]
pub(crate) fn sanitize_remote_planner_input(
    input: &PlannerInput,
    privacy: &RemotePlannerPrivacySettings,
    endpoint_scope: &ProviderEndpointScope,
) -> Result<RemotePlannerInput, crate::commands::ToolError> {
    let remote_data_mode = enforce_remote_planner_privacy(input, privacy, endpoint_scope)?;
    sanitize_remote_planner_input_authorized(input, remote_data_mode)
}

pub(crate) fn sanitize_remote_planner_input_authorized(
    input: &PlannerInput,
    remote_data_mode: RemoteDataMode,
) -> Result<RemotePlannerInput, crate::commands::ToolError> {
    let mut metadata = SanitizationMetadata::default();
    let prompt_injection_indicators = detect_prompt_injection(input);

    let safe = RemotePlannerInput {
        trust_boundary_version: String::from("remote-planner-boundary-v2"),
        trusted_runtime: RemoteTrustedRuntime {
            request_id: truncate_identifier(&input.request_id),
            safety: input.safety.clone(),
            available_tools: input.available_tools.clone(),
            active_skill_names: input
                .active_skill_names
                .iter()
                .take(MAX_REMOTE_SKILLS)
                .map(|name| truncate_identifier(name))
                .collect(),
            remote_data_mode,
        },
        user_request: RemoteUserRequest {
            transcript: sanitize_text(&input.transcript, MAX_REGION_TEXT_CHARS, &mut metadata),
        },
        untrusted_data: RemoteUntrustedData {
            agent_state: sanitize_agent_state(input, &mut metadata),
            page_snapshot: input
                .page_snapshot
                .as_ref()
                .map(|snapshot| sanitize_page_snapshot(snapshot, &input.transcript, &mut metadata)),
            page_model: input
                .page_model
                .as_ref()
                .map(|page| sanitize_page_model(page, &input.transcript, &mut metadata)),
            recent_tool_results: sanitize_history(&input.recent_tool_results, &mut metadata),
            relevant_skill_summaries: sanitize_skills(
                &input.relevant_skill_summaries,
                &mut metadata,
            ),
            prompt_injection_indicators,
            sanitization: metadata,
        },
    };

    Ok(safe)
}

#[cfg(test)]
fn enforce_remote_planner_privacy(
    input: &PlannerInput,
    privacy: &RemotePlannerPrivacySettings,
    endpoint_scope: &ProviderEndpointScope,
) -> Result<RemoteDataMode, crate::commands::ToolError> {
    let page_origin = planner_page_origin(input);
    let high_risk_reason = high_risk_context_reason(input);
    match evaluate_remote_planner_policy(
        privacy,
        endpoint_scope,
        page_origin.as_deref(),
        high_risk_reason,
        &[],
        crate::commands::current_timestamp_ms(),
    ) {
        RemotePlannerPolicyResult::Allowed(authorization) => {
            if matches!(
                authorization,
                super::remote_data_consent::RemotePlannerDataAuthorization::Loopback
            ) {
                Ok(RemoteDataMode::LoopbackLocalService)
            } else {
                Ok(RemoteDataMode::NetworkRemoteWithExplicitConsent)
            }
        }
        RemotePlannerPolicyResult::ConsentRequired => Err(privacy_error(
            "remote_data_consent_required",
            "This site requires an explicit remote-data decision before sanitized planner context can leave the device.",
            "consent_required",
        )),
        RemotePlannerPolicyResult::Blocked { code, reason_code } => Err(crate::commands::ToolError {
            code: code.to_string(),
            message: match code {
                "remote_data_local_only" => String::from(
                    "Local-only planner mode blocks non-loopback planner endpoints.",
                ),
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
        }),
    }
}

#[cfg(test)]
fn privacy_error(code: &str, message: &str, policy: &str) -> crate::commands::ToolError {
    crate::commands::ToolError {
        code: code.to_string(),
        message: message.to_string(),
        retryable: false,
        details: Some(serde_json::json!({ "policy": policy })),
    }
}

pub(crate) fn planner_page_origin(input: &PlannerInput) -> Option<String> {
    let raw = input
        .agent_state
        .url
        .as_deref()
        .or_else(|| {
            input
                .page_model
                .as_ref()
                .and_then(|page| page.url.as_deref())
        })
        .or_else(|| {
            input
                .page_snapshot
                .as_ref()
                .map(|snapshot| snapshot.url.as_str())
        })?;
    let url = match url::Url::parse(raw) {
        Ok(url) => url,
        Err(_) => return None,
    };
    let origin = url.origin();
    origin.is_tuple().then(|| origin.ascii_serialization())
}

#[cfg(test)]
mod post_p8_url_sanitization_tests;
#[cfg(test)]
mod tests;
