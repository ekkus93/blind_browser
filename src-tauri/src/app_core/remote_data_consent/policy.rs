//! Decides whether a remote-planner request may proceed without asking,
//! must be blocked outright, or needs an explicit consent challenge — the
//! single decision point every entry into [`super`] routes through.

use crate::config::{
    PersistedOriginDecision, RemotePlannerNetworkMode, RemotePlannerPrivacySettings,
    REMOTE_DATA_POLICY_VERSION,
};
use crate::provider_endpoint::ProviderEndpointScope;

use super::grants::RemotePlannerEphemeralGrant;
use super::types::RemotePlannerDataAuthorization;

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
