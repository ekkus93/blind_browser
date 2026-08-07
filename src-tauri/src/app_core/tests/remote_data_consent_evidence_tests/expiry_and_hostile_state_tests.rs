//! Evidence that an expired challenge, a config-persistence failure, or a
//! runtime-state change fails closed rather than silently authorizing, and
//! that no serialized runtime state ever leaks the raw sanitized payload or
//! hostile page content.

use super::*;

use crate::commands::{current_timestamp_ms, RemotePlannerConsentDecision};
use crate::config::{
    AppConfig, PersistedOriginDecision, RemotePlannerNetworkMode, RemotePlannerOriginRule,
    REMOTE_DATA_POLICY_VERSION,
};

#[test]
#[cfg_attr(
    any(windows, target_os = "linux"),
    ignore = "real Wry AppCore fixture must run in a process-isolated test invocation"
)]
#[cfg_attr(
    not(any(windows, target_os = "linux")),
    ignore = "real Wry AppCore fixture requires Tauri's any-thread desktop builder"
)]
fn remote_data_consent_expiry_invalidation_persistence_and_hostile_state_are_fail_closed() {
    let (app, _config_root) = test_app();
    let (mut core, _secret) = test_core(&app);

    let (mut challenge, draft) = requirement(&mut core, "expired", "analyze this article");
    challenge.expires_at_ms = current_timestamp_ms();
    store(&mut core, challenge.clone(), draft);
    let expired = error(
        core.resolve_pending_remote_planner_consent(
            &challenge.challenge_id,
            &challenge.challenge_digest,
            RemotePlannerConsentDecision::AllowOnce,
        ),
        "expired consent was accepted",
    );
    assert_eq!(expired.code, "remote_data_consent_expired");
    assert_replay_missing(&mut core, &challenge);

    let (challenge, draft) = requirement(&mut core, "page-change", "analyze this article");
    store(&mut core, challenge.clone(), draft);
    core.state.page_generation += 1;
    let changed = error(
        core.resolve_pending_remote_planner_consent(
            &challenge.challenge_id,
            &challenge.challenge_digest,
            RemotePlannerConsentDecision::AllowOnce,
        ),
        "page change did not invalidate consent",
    );
    assert_eq!(changed.code, "remote_data_consent_state_changed");

    let (challenge, draft) = requirement(&mut core, "destination", "analyze this article");
    store(&mut core, challenge.clone(), draft);
    core.config
        .remote_planner_profiles
        .get_mut(PROFILE)
        .expect("test profile should exist")
        .model = String::from("changed-model");
    let changed = error(
        core.resolve_pending_remote_planner_consent(
            &challenge.challenge_id,
            &challenge.challenge_digest,
            RemotePlannerConsentDecision::AllowOnce,
        ),
        "destination change did not invalidate consent",
    );
    assert_eq!(changed.code, "remote_data_consent_destination_changed");
    core.config
        .remote_planner_profiles
        .get_mut(PROFILE)
        .expect("test profile should exist")
        .model = String::from(MODEL);

    let (challenge, draft) = requirement(&mut core, "mode", "analyze this article");
    store(&mut core, challenge.clone(), draft);
    core.config.remote_planner_privacy.network_mode = RemotePlannerNetworkMode::LocalOnly;
    let changed = error(
        core.resolve_pending_remote_planner_consent(
            &challenge.challenge_id,
            &challenge.challenge_digest,
            RemotePlannerConsentDecision::AllowOnce,
        ),
        "mode change did not invalidate consent",
    );
    assert_eq!(changed.code, "remote_data_consent_state_changed");
    core.config.remote_planner_privacy.network_mode = RemotePlannerNetworkMode::AskPerOrigin;

    let (challenge, draft) = requirement(&mut core, "block", "analyze this article");
    store(&mut core, challenge.clone(), draft);
    core.config
        .remote_planner_privacy
        .origin_rules
        .push(RemotePlannerOriginRule {
            page_origin: String::from(ORIGIN),
            decision: PersistedOriginDecision::Block,
            endpoint_scope: None,
            policy_version: REMOTE_DATA_POLICY_VERSION,
            created_at_ms: current_timestamp_ms(),
        });
    let changed = error(
        core.resolve_pending_remote_planner_consent(
            &challenge.challenge_id,
            &challenge.challenge_digest,
            RemotePlannerConsentDecision::AllowOnce,
        ),
        "block change did not invalidate consent",
    );
    assert_eq!(changed.code, "remote_data_consent_state_changed");
    core.config.remote_planner_privacy.origin_rules.clear();

    let (challenge, draft) = requirement(&mut core, "persist", "analyze this article");
    store(&mut core, challenge.clone(), draft);
    let config_path =
        AppConfig::config_path_for_app(&core.app_handle).expect("test config path should resolve");
    replace_parent_with_file(&config_path);
    let failed = error(
        core.resolve_pending_remote_planner_consent(
            &challenge.challenge_id,
            &challenge.challenge_digest,
            RemotePlannerConsentDecision::AllowPersistent,
        ),
        "persistence failure authorized consent",
    );
    assert_eq!(failed.code, "remote_data_consent_persist_failed");
    assert!(core.config.remote_planner_privacy.origin_rules.is_empty());
    restore_parent(&config_path);
    assert_replay_missing(&mut core, &challenge);
    let _ = requirement(&mut core, "persist-retry", "analyze this article");

    let (challenge, draft) = requirement(&mut core, "hostile", HOSTILE);
    store(&mut core, challenge.clone(), draft);
    let serialized = [
        serde_json::to_string(&core.state).expect("AppState should serialize"),
        serde_json::to_string(&core.current_remote_planner_privacy_status())
            .expect("privacy status should serialize"),
        serde_json::to_string(&core.current_runtime_status_snapshot(true))
            .expect("runtime status should serialize"),
        serde_json::to_string(&core.current_agent_state_snapshot(false))
            .expect("agent state should serialize"),
        serde_json::to_string(&challenge).expect("challenge should serialize"),
    ];
    for value in &serialized {
        assert!(!value.contains(HOSTILE));
        assert!(!value.contains("consent-hostile-sentinel-7b1d"));
        assert!(!value.contains("sanitized_input"));
    }
    for value in &serialized[..4] {
        assert!(!value.contains(&challenge.challenge_digest));
    }
    assert!(serialized[4].contains(&challenge.challenge_digest));
    assert!(!serialized[0].contains("pending_remote_planner_consent"));
    assert!(serialized[1].contains(&challenge.challenge_id));
    assert!(serialized[2].contains(&challenge.challenge_id));
    assert!(serialized[3].contains(&challenge.challenge_id));
}
