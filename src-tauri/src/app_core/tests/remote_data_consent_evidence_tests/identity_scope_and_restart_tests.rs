//! Evidence that a consent response is bound to the exact challenge that
//! issued it (id, digest, page, destination, safety settings), that
//! unrelated runtime changes don't invalidate it, and that ephemeral grants
//! and pending challenges never survive an `AppCore` restart.

use super::*;

use crate::app_core::remote_data_consent::PendingConsentResolution;
use crate::app_core::AppCore;
use crate::commands::RemotePlannerConsentDecision;
use crate::config::{AppConfig, PersistedOriginDecision};
use crate::page_model::{PageRegion, RegionRole, RegionSource};

#[test]
#[cfg_attr(
    any(windows, target_os = "linux"),
    ignore = "real Wry AppCore fixture must run in a process-isolated test invocation"
)]
#[cfg_attr(
    not(any(windows, target_os = "linux")),
    ignore = "real Wry AppCore fixture requires Tauri's any-thread desktop builder"
)]
fn remote_data_privacy_closure_identity_scope_and_restart_are_fail_closed() {
    let (app, _config_root) = test_app();
    let (mut core, _secret) = test_core(&app);

    let (challenge, draft) = requirement(&mut core, "wrong-id", "analyze this article");
    store(&mut core, challenge.clone(), draft);
    assert_consumed_failure(
        &mut core,
        &challenge,
        "wrong-challenge-id",
        &challenge.challenge_digest,
        "remote_data_consent_mismatch",
    );

    let (challenge, draft) = requirement(&mut core, "wrong-digest", "analyze this article");
    store(&mut core, challenge.clone(), draft);
    assert_consumed_failure(
        &mut core,
        &challenge,
        &challenge.challenge_id,
        "wrong-challenge-digest",
        "remote_data_consent_mismatch",
    );

    let (old_challenge, old_draft) = requirement(&mut core, "old", "analyze this article");
    let (new_challenge, new_draft) = requirement(&mut core, "new", "analyze this article");
    store(&mut core, old_challenge.clone(), old_draft);
    store(&mut core, new_challenge.clone(), new_draft);
    assert_consumed_failure(
        &mut core,
        &new_challenge,
        &old_challenge.challenge_id,
        &old_challenge.challenge_digest,
        "remote_data_consent_mismatch",
    );

    let (challenge, draft) = requirement(&mut core, "page-id", "analyze this article");
    store(&mut core, challenge.clone(), draft);
    core.state.current_page_id = Some(String::from("different-page"));
    assert_consumed_failure(
        &mut core,
        &challenge,
        &challenge.challenge_id,
        &challenge.challenge_digest,
        "remote_data_consent_state_changed",
    );
    core.state.current_page_id = Some(String::from("page-consent-evidence"));

    let (challenge, draft) = requirement(&mut core, "origin", "analyze this article");
    store(&mut core, challenge.clone(), draft);
    core.state
        .current_page
        .as_mut()
        .expect("test page should exist")
        .url = Some(String::from("https://other.example/article"));
    assert_consumed_failure(
        &mut core,
        &challenge,
        &challenge.challenge_id,
        &challenge.challenge_digest,
        "remote_data_consent_state_changed",
    );
    core.state
        .current_page
        .as_mut()
        .expect("test page should exist")
        .url = Some(String::from("https://example.com/article"));

    for (name, base_url) in [
        ("scheme", "http://api.example.com/v1"),
        ("host", "https://other.example.com/v1"),
        ("port", "https://api.example.com:8443/v1"),
        ("path", "https://api.example.com/v2"),
    ] {
        let (challenge, draft) = requirement(
            &mut core,
            &format!("destination-{name}"),
            "analyze this article",
        );
        store(&mut core, challenge.clone(), draft);
        set_profile_base_url(&mut core, base_url);
        assert_consumed_failure(
            &mut core,
            &challenge,
            &challenge.challenge_id,
            &challenge.challenge_digest,
            "remote_data_consent_destination_changed",
        );
        set_profile_base_url(&mut core, "https://api.example.com/v1");
    }

    let (challenge, draft) = requirement(&mut core, "safety", "analyze this article");
    store(&mut core, challenge.clone(), draft);
    core.config.safety.always_confirm_submit = !core.config.safety.always_confirm_submit;
    assert_consumed_failure(
        &mut core,
        &challenge,
        &challenge.challenge_id,
        &challenge.challenge_digest,
        "remote_data_consent_state_changed",
    );
    core.config.safety.always_confirm_submit = !core.config.safety.always_confirm_submit;

    let (challenge, draft) = requirement(&mut core, "unrelated", "analyze this article");
    let token_before = core.current_runtime_state_token();
    store(&mut core, challenge.clone(), draft);
    core.state.speaking = !core.state.speaking;
    assert_eq!(token_before, core.current_runtime_state_token());
    let unrelated = core
        .resolve_pending_remote_planner_consent(
            &challenge.challenge_id,
            &challenge.challenge_digest,
            RemotePlannerConsentDecision::AllowOnce,
        )
        .expect("unrelated speaking status should not invalidate consent");
    assert!(matches!(unrelated, PendingConsentResolution::Authorized(_)));
    core.clear_remote_planner_consent_runtime();

    let (challenge, draft) = requirement(&mut core, "high-risk", "analyze this article");
    store(&mut core, challenge.clone(), draft);
    core.state
        .current_page
        .as_mut()
        .expect("test page should exist")
        .regions
        .push(PageRegion {
            region_id: String::from("new-payment-region"),
            role: RegionRole::Paragraph,
            label: None,
            text: String::from("Enter credit card security code"),
            bbox: None,
            source: RegionSource::Dom,
        });
    assert_consumed_failure(
        &mut core,
        &challenge,
        &challenge.challenge_id,
        &challenge.challenge_digest,
        "remote_data_high_risk_blocked",
    );
    core.state
        .current_page
        .as_mut()
        .expect("test page should exist")
        .regions
        .clear();

    let (session_challenge, session_draft) =
        requirement(&mut core, "restart-session", "analyze this article");
    let (pending_challenge, pending_draft) =
        requirement(&mut core, "restart-pending", "analyze this article");
    store(&mut core, session_challenge.clone(), session_draft);
    let session = core
        .resolve_pending_remote_planner_consent(
            &session_challenge.challenge_id,
            &session_challenge.challenge_digest,
            RemotePlannerConsentDecision::AllowSession,
        )
        .expect("session consent should authorize");
    assert!(matches!(session, PendingConsentResolution::Authorized(_)));
    assert_eq!(core.remote_planner_ephemeral_grants.len(), 1);
    store(&mut core, pending_challenge.clone(), pending_draft);
    assert!(core.pending_remote_planner_consent.is_some());

    let reconstructed = AppCore::new(app.handle().clone())
        .expect("AppCore should reconstruct from persistent config");
    assert!(reconstructed.remote_planner_ephemeral_grants.is_empty());
    assert!(reconstructed.pending_remote_planner_consent.is_none());

    core.clear_remote_planner_consent_runtime();
    let (challenge, draft) = requirement(&mut core, "persist-restart", "analyze this article");
    store(&mut core, challenge.clone(), draft);
    let persistent = core
        .resolve_pending_remote_planner_consent(
            &challenge.challenge_id,
            &challenge.challenge_digest,
            RemotePlannerConsentDecision::AllowPersistent,
        )
        .expect("persistent consent should authorize");
    assert!(matches!(
        persistent,
        PendingConsentResolution::Authorized(_)
    ));
    let reconstructed =
        AppCore::new(app.handle().clone()).expect("AppCore should reload persisted privacy rules");
    assert!(reconstructed
        .config
        .remote_planner_privacy
        .origin_rules
        .iter()
        .any(|rule| {
            rule.page_origin == ORIGIN
                && matches!(rule.decision, PersistedOriginDecision::Allow)
                && rule.endpoint_scope.as_deref() == Some("https://api.example.com/v1")
        }));
    assert!(reconstructed.remote_planner_ephemeral_grants.is_empty());
    assert!(reconstructed.pending_remote_planner_consent.is_none());

    let config_path =
        AppConfig::config_path_for_app(&core.app_handle).expect("test config path should resolve");
    let config_bytes =
        std::fs::read_to_string(config_path).expect("persisted config should be readable");
    assert!(!config_bytes.contains(&challenge.challenge_id));
    assert!(!config_bytes.contains(&challenge.challenge_digest));
    assert!(!config_bytes.contains("sanitized_input"));
}
