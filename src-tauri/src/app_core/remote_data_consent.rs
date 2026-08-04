use crate::config::{
    PersistedOriginDecision, RemotePlannerNetworkMode, RemotePlannerPrivacySettings,
    REMOTE_DATA_POLICY_VERSION,
};
use crate::provider_endpoint::ProviderEndpointScope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RemotePlannerDataAuthorization {
    Loopback,
    GlobalAllow,
    PersistentAllow,
    SessionAllow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RemotePlannerSessionGrant {
    pub(crate) page_origin: String,
    pub(crate) endpoint_scope: String,
    pub(crate) policy_version: u32,
    pub(crate) expires_at_ms: u64,
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
    session_grants: &[RemotePlannerSessionGrant],
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
    if session_grants.iter().any(|grant| {
        grant.expires_at_ms > now_ms
            && grant.page_origin == page_origin
            && grant.endpoint_scope == endpoint
            && grant.policy_version == REMOTE_DATA_POLICY_VERSION
    }) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        HighRiskOriginPolicy, RemotePlannerOriginRule, RemotePlannerPrivacySettings,
    };

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
    fn persistent_allow_is_exactly_destination_and_version_bound() {
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
        privacy.origin_rules[0].policy_version = 0;
        assert_eq!(
            evaluate_remote_planner_policy(
                &privacy,
                &endpoint("https://api.example.com/v1"),
                Some("https://example.com"),
                None,
                &[],
                1,
            ),
            RemotePlannerPolicyResult::ConsentRequired
        );
    }

    #[test]
    fn session_grant_is_scoped_and_expires() {
        let privacy = settings(RemotePlannerNetworkMode::AskPerOrigin);
        let grant = RemotePlannerSessionGrant {
            page_origin: String::from("https://example.com"),
            endpoint_scope: String::from("https://api.example.com/v1"),
            policy_version: REMOTE_DATA_POLICY_VERSION,
            expires_at_ms: 100,
        };
        assert_eq!(
            evaluate_remote_planner_policy(
                &privacy,
                &endpoint("https://api.example.com/v1"),
                Some("https://example.com"),
                None,
                &[grant.clone()],
                99,
            ),
            RemotePlannerPolicyResult::Allowed(RemotePlannerDataAuthorization::SessionAllow)
        );
        assert_eq!(
            evaluate_remote_planner_policy(
                &privacy,
                &endpoint("https://api.example.com/v1"),
                Some("https://example.com"),
                None,
                &[grant],
                100,
            ),
            RemotePlannerPolicyResult::ConsentRequired
        );
    }

    #[test]
    fn ask_mode_requires_consent_and_unknown_origin_fails_closed() {
        let privacy = settings(RemotePlannerNetworkMode::AskPerOrigin);
        assert_eq!(
            evaluate_remote_planner_policy(
                &privacy,
                &endpoint("https://api.example.com/v1"),
                Some("https://example.com"),
                None,
                &[],
                1,
            ),
            RemotePlannerPolicyResult::ConsentRequired
        );
        assert!(matches!(
            evaluate_remote_planner_policy(
                &privacy,
                &endpoint("https://api.example.com/v1"),
                None,
                None,
                &[],
                1,
            ),
            RemotePlannerPolicyResult::Blocked {
                code: "remote_data_opaque_origin_blocked",
                ..
            }
        ));
    }
}
