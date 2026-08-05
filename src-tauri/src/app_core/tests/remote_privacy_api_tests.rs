use super::*;

use super::super::remote_data_consent::{
    evaluate_remote_planner_policy, RemotePlannerDataAuthorization, RemotePlannerPolicyResult,
};
use crate::commands::{
    RemotePlannerConsentChallengeSummary, RemotePlannerDisclosureClass,
    RemotePlannerDisclosureCounts, RemotePlannerEffectiveDecision, RemotePlannerPrivacyOperation,
};
use crate::config::{
    HighRiskOriginPolicy, PersistedOriginDecision, RemotePlannerNetworkMode,
    RemotePlannerOriginRule, RemotePlannerPrivacySettings, REMOTE_DATA_POLICY_VERSION,
};
use crate::provider_endpoint::ProviderEndpointScope;

fn endpoint(raw: &str) -> ProviderEndpointScope {
    ProviderEndpointScope::parse(raw).expect("test endpoint must be valid")
}

fn privacy_settings(mode: RemotePlannerNetworkMode) -> RemotePlannerPrivacySettings {
    RemotePlannerPrivacySettings {
        network_mode: mode,
        high_risk_origin_policy: HighRiskOriginPolicy::Block,
        ..RemotePlannerPrivacySettings::default()
    }
}

fn allow_rule(origin: &str, endpoint_scope: &str, policy_version: u32) -> RemotePlannerOriginRule {
    RemotePlannerOriginRule {
        page_origin: origin.to_string(),
        decision: PersistedOriginDecision::Allow,
        endpoint_scope: Some(endpoint_scope.to_string()),
        policy_version,
        created_at_ms: 1,
    }
}

fn block_rule(origin: &str) -> RemotePlannerOriginRule {
    RemotePlannerOriginRule {
        page_origin: origin.to_string(),
        decision: PersistedOriginDecision::Block,
        endpoint_scope: None,
        policy_version: REMOTE_DATA_POLICY_VERSION,
        created_at_ms: 1,
    }
}

#[test]
fn remote_privacy_policy_matrix_is_fail_closed_and_scope_bound() {
    let remote = endpoint("https://api.example.com/v1");
    let ask = privacy_settings(RemotePlannerNetworkMode::AskPerOrigin);

    assert_eq!(
        evaluate_remote_planner_policy(&ask, &remote, None, None, &[], 10),
        RemotePlannerPolicyResult::Blocked {
            code: "remote_data_opaque_origin_blocked",
            reason_code: "origin_unavailable",
        }
    );
    assert_eq!(
        evaluate_remote_planner_policy(
            &ask,
            &remote,
            Some("https://example.com"),
            None,
            &[],
            10,
        ),
        RemotePlannerPolicyResult::ConsentRequired
    );

    let global = privacy_settings(RemotePlannerNetworkMode::AllowSanitizedNonHighRisk);
    assert_eq!(
        evaluate_remote_planner_policy(
            &global,
            &remote,
            Some("https://example.com"),
            None,
            &[],
            10,
        ),
        RemotePlannerPolicyResult::Allowed(RemotePlannerDataAuthorization::GlobalAllow)
    );
    assert_eq!(
        evaluate_remote_planner_policy(
            &global,
            &remote,
            Some("https://example.com"),
            Some("payment_context"),
            &[],
            10,
        ),
        RemotePlannerPolicyResult::Blocked {
            code: "remote_data_high_risk_blocked",
            reason_code: "payment_context",
        }
    );

    let mut blocked = global.clone();
    blocked.origin_rules.push(block_rule("https://example.com"));
    assert_eq!(
        evaluate_remote_planner_policy(
            &blocked,
            &remote,
            Some("https://example.com"),
            None,
            &[],
            10,
        ),
        RemotePlannerPolicyResult::Blocked {
            code: "remote_data_origin_blocked",
            reason_code: "origin_block",
        }
    );

    let mut exact_allow = ask.clone();
    exact_allow.origin_rules.push(allow_rule(
        "https://example.com",
        "https://api.example.com/v1",
        REMOTE_DATA_POLICY_VERSION,
    ));
    assert_eq!(
        evaluate_remote_planner_policy(
            &exact_allow,
            &remote,
            Some("https://example.com"),
            None,
            &[],
            10,
        ),
        RemotePlannerPolicyResult::Allowed(RemotePlannerDataAuthorization::PersistentAllow)
    );

    for mismatched_endpoint in [
        "https://api.example.com/v2",
        "https://api.example.com:444/v1",
        "https://other.example.com/v1",
        "http://api.example.com/v1",
    ] {
        assert_eq!(
            evaluate_remote_planner_policy(
                &exact_allow,
                &endpoint(mismatched_endpoint),
                Some("https://example.com"),
                None,
                &[],
                10,
            ),
            RemotePlannerPolicyResult::ConsentRequired,
            "endpoint mismatch must not authorize: {mismatched_endpoint}"
        );
    }
    for mismatched_origin in [
        "http://example.com",
        "https://other.example.com",
        "https://example.com:444",
    ] {
        assert_eq!(
            evaluate_remote_planner_policy(
                &exact_allow,
                &remote,
                Some(mismatched_origin),
                None,
                &[],
                10,
            ),
            RemotePlannerPolicyResult::ConsentRequired,
            "origin mismatch must not authorize: {mismatched_origin}"
        );
    }

    let mut stale_allow = ask;
    stale_allow.origin_rules.push(allow_rule(
        "https://example.com",
        "https://api.example.com/v1",
        REMOTE_DATA_POLICY_VERSION + 1,
    ));
    assert_eq!(
        evaluate_remote_planner_policy(
            &stale_allow,
            &remote,
            Some("https://example.com"),
            None,
            &[],
            10,
        ),
        RemotePlannerPolicyResult::ConsentRequired
    );
}

#[test]
#[cfg_attr(
    not(any(windows, target_os = "linux")),
    ignore = "real Wry AppCore fixture requires Tauri's any-thread desktop builder"
)]
fn remote_privacy_status_reports_every_non_ephemeral_decision_and_stale_rules() {
    let builder = tauri::Builder::<tauri::Wry>::default();
    #[cfg(any(windows, target_os = "linux"))]
    let builder = builder.any_thread();
    let app = builder
        .build(tauri::generate_context!())
        .expect("test Tauri application should build");
    let mut core = super::super::AppCore::new(app.handle().clone())
        .expect("AppCore should initialize for privacy status tests");
    core.config = AppConfig::default();
    core.state = AppState::from_config(&core.config);
    core.state.current_page = Some(fixture_page_with_metadata(
        "Example",
        "https://EXAMPLE.com:443/private?q=secret#fragment",
        Vec::new(),
    ));

    core.config.remote_planner_privacy =
        privacy_settings(RemotePlannerNetworkMode::AskPerOrigin);
    let status = core.current_remote_planner_privacy_status();
    assert_eq!(status.current_page_origin.as_deref(), Some("https://example.com"));
    assert_eq!(status.effective_decision, RemotePlannerEffectiveDecision::ConsentRequired);
    assert_eq!(status.reason_code.as_deref(), Some("consent_required"));

    core.config.remote_planner_privacy.network_mode = RemotePlannerNetworkMode::LocalOnly;
    let status = core.current_remote_planner_privacy_status();
    assert_eq!(status.effective_decision, RemotePlannerEffectiveDecision::LocalOnly);
    assert_eq!(status.reason_code.as_deref(), Some("local_only"));

    core.config.remote_planner_privacy =
        privacy_settings(RemotePlannerNetworkMode::AllowSanitizedNonHighRisk);
    let status = core.current_remote_planner_privacy_status();
    assert_eq!(status.effective_decision, RemotePlannerEffectiveDecision::AllowedGlobal);

    core.config.remote_planner_privacy =
        privacy_settings(RemotePlannerNetworkMode::AskPerOrigin);
    core.config
        .remote_planner_privacy
        .origin_rules
        .push(block_rule("https://example.com"));
    let status = core.current_remote_planner_privacy_status();
    assert_eq!(status.effective_decision, RemotePlannerEffectiveDecision::OriginBlocked);
    assert_eq!(status.reason_code.as_deref(), Some("origin_block"));
    assert_eq!(status.persistent_rule, Some(PersistedOriginDecision::Block));

    core.config.remote_planner_privacy =
        privacy_settings(RemotePlannerNetworkMode::AskPerOrigin);
    core.config.remote_planner_privacy.origin_rules.push(allow_rule(
        "https://example.com",
        "https://api.openai.com/v1",
        REMOTE_DATA_POLICY_VERSION,
    ));
    let status = core.current_remote_planner_privacy_status();
    assert_eq!(status.effective_decision, RemotePlannerEffectiveDecision::AllowedPersistent);
    assert_eq!(status.persistent_rule, Some(PersistedOriginDecision::Allow));
    assert_eq!(status.stale_allow_rule_count, 0);

    core.config.remote_planner_privacy.origin_rules[0].policy_version =
        REMOTE_DATA_POLICY_VERSION + 1;
    let status = core.current_remote_planner_privacy_status();
    assert_eq!(status.effective_decision, RemotePlannerEffectiveDecision::ConsentRequired);
    assert_eq!(status.persistent_rule, Some(PersistedOriginDecision::Allow));
    assert_eq!(status.stale_allow_rule_count, 1);
    assert!(status.persistent_rules[0].stale);

    core.config.remote_planner_privacy =
        privacy_settings(RemotePlannerNetworkMode::AskPerOrigin);
    core.state.current_page = Some(fixture_page_with_metadata(
        "Local file",
        "file:///tmp/private.html",
        Vec::new(),
    ));
    let status = core.current_remote_planner_privacy_status();
    assert_eq!(status.effective_decision, RemotePlannerEffectiveDecision::OriginUnavailable);
    assert_eq!(status.reason_code.as_deref(), Some("origin_unavailable"));
    assert_eq!(status.current_page_origin, None);

    core.state.current_page = Some(fixture_problematic_checkout_page());
    core.config.remote_planner_privacy =
        privacy_settings(RemotePlannerNetworkMode::AllowSanitizedNonHighRisk);
    let status = core.current_remote_planner_privacy_status();
    assert_eq!(status.effective_decision, RemotePlannerEffectiveDecision::HighRiskBlocked);
    assert!(status.reason_code.is_some());

    core.config.providers.planner.remote_profile = Some(String::from("ollama-default"));
    let status = core.current_remote_planner_privacy_status();
    assert_eq!(status.effective_decision, RemotePlannerEffectiveDecision::LoopbackLocal);
    assert_eq!(status.endpoint_is_loopback, Some(true));
}

#[test]
#[cfg_attr(
    not(any(windows, target_os = "linux")),
    ignore = "real Wry AppCore fixture requires Tauri's any-thread desktop builder"
)]
fn remote_privacy_operations_fail_closed_without_unnecessary_persistence() {
    let builder = tauri::Builder::<tauri::Wry>::default();
    #[cfg(any(windows, target_os = "linux"))]
    let builder = builder.any_thread();
    let app = builder
        .build(tauri::generate_context!())
        .expect("test Tauri application should build");
    let mut core = super::super::AppCore::new(app.handle().clone())
        .expect("AppCore should initialize for privacy operation tests");
    core.config = AppConfig::default();
    core.state = AppState::from_config(&core.config);

    assert!(!core
        .apply_remote_planner_privacy_operation(RemotePlannerPrivacyOperation::GetStatus)
        .expect("status lookup must be a no-op"));
    assert!(!core
        .apply_remote_planner_privacy_operation(RemotePlannerPrivacyOperation::ClearSessionGrants)
        .expect("clearing an empty session-grant set must be a no-op"));

    let error = core
        .apply_remote_planner_privacy_operation(
            RemotePlannerPrivacyOperation::ClearAllPersistentRules { confirmed: false },
        )
        .expect_err("clear-all must require explicit confirmation");
    assert_eq!(error.code, "remote_data_clear_all_confirmation_required");

    let error = core
        .apply_remote_planner_privacy_operation(RemotePlannerPrivacyOperation::UpsertOriginRule {
            page_origin: String::from("https://example.com/private?secret=value"),
            decision: PersistedOriginDecision::Block,
        })
        .expect_err("rule input with path/query must be rejected");
    assert_eq!(error.code, "remote_data_rule_invalid");

    let error = core
        .apply_remote_planner_privacy_operation(RemotePlannerPrivacyOperation::RevokeOriginRule {
            page_origin: String::from("https://example.com"),
            decision: PersistedOriginDecision::Block,
            endpoint_scope: Some(String::from("https://api.openai.com/v1")),
        })
        .expect_err("origin-wide blocks must not accept endpoint scope");
    assert_eq!(error.code, "remote_data_rule_invalid");

    core.state.current_page = Some(fixture_page_with_metadata(
        "Local file",
        "file:///tmp/private.html",
        Vec::new(),
    ));
    let error = core
        .apply_remote_planner_privacy_operation(
            RemotePlannerPrivacyOperation::UpsertCurrentOriginRule {
                decision: PersistedOriginDecision::Block,
            },
        )
        .expect_err("opaque current origins must not create rules");
    assert_eq!(error.code, "remote_data_opaque_origin_blocked");

    core.state.current_page = Some(fixture_page_with_metadata(
        "Example",
        "https://example.com/private",
        Vec::new(),
    ));
    core.config.remote_planner_privacy.origin_rules = vec![block_rule("https://example.com")];
    assert!(!core
        .apply_remote_planner_privacy_operation(
            RemotePlannerPrivacyOperation::UpsertCurrentOriginRule {
                decision: PersistedOriginDecision::Block,
            },
        )
        .expect("an exact current-origin block must be a no-op"));
    assert!(!core
        .apply_remote_planner_privacy_operation(
            RemotePlannerPrivacyOperation::ClearPersistentAllows,
        )
        .expect("clearing allows must retain an all-block rule set"));
    assert_eq!(
        core.config.remote_planner_privacy.origin_rules,
        vec![block_rule("https://example.com")]
    );

    core.config.remote_planner_privacy.origin_rules = vec![allow_rule(
        "https://example.com",
        "https://api.openai.com/v1",
        REMOTE_DATA_POLICY_VERSION,
    )];
    assert!(!core
        .apply_remote_planner_privacy_operation(RemotePlannerPrivacyOperation::UpsertOriginRule {
            page_origin: String::from("https://EXAMPLE.com:443/"),
            decision: PersistedOriginDecision::Allow,
        })
        .expect("an exact authoritative allow must be a no-op"));
    assert_eq!(
        core.config.remote_planner_privacy.origin_rules[0]
            .endpoint_scope
            .as_deref(),
        Some("https://api.openai.com/v1")
    );

    core.state.current_page = Some(fixture_problematic_checkout_page());
    let error = core
        .apply_remote_planner_privacy_operation(
            RemotePlannerPrivacyOperation::UpsertCurrentOriginRule {
                decision: PersistedOriginDecision::Allow,
            },
        )
        .expect_err("high-risk current origins must not gain persistent allows");
    assert_eq!(error.code, "remote_data_high_risk_blocked");
}

#[test]
fn privacy_status_challenge_summary_serialization_excludes_digest_and_payload_content() {
    let summary = RemotePlannerConsentChallengeSummary {
        challenge_id: String::from("challenge-public-id"),
        request_id: String::from("request-public-id"),
        page_origin: String::from("https://example.com"),
        endpoint_display: String::from("https://api.example.com/v1"),
        profile_name: String::from("test-profile"),
        model_label: String::from("test-model"),
        policy_version: REMOTE_DATA_POLICY_VERSION,
        disclosure_classes: vec![
            RemotePlannerDisclosureClass::UserTranscript,
            RemotePlannerDisclosureClass::PageOrigin,
        ],
        disclosure_counts: RemotePlannerDisclosureCounts {
            selected_region_count: 1,
            selected_element_count: 2,
            ocr_derived_region_count: 0,
            tool_history_count: 0,
            skill_summary_count: 0,
            sanitized_serialized_bytes: 128,
        },
        expires_at_ms: 123_456,
        allow_once: true,
        allow_session: true,
        allow_persistent: true,
        block_persistent: true,
    };

    let serialized = serde_json::to_string(&summary).expect("summary should serialize");
    assert!(serialized.contains("challenge-public-id"));
    assert!(!serialized.contains("challenge_digest"));
    assert!(!serialized.contains("payload_digest"));
    assert!(!serialized.contains("sanitized_input"));
    assert!(!serialized.contains("transcript"));
    assert!(!serialized.contains("page_model"));
    assert!(!serialized.contains("ocr_text"));
}
