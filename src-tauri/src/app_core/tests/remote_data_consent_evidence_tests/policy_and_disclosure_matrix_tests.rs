//! Evidence for the full policy matrix (session/persistent grants, broad
//! allow, local-only, origin block, high-risk block, opaque origin, loopback)
//! and that a consent challenge discloses only bounded, sanitized metadata —
//! never the hostile page text, OCR content, or query/fragment sentinels
//! that produced it.

use super::*;

use crate::app_core::remote_data_consent::{
    PendingConsentResolution, RemotePlannerDataAuthorization, RemotePlannerPreparation,
};
use crate::commands::{
    current_timestamp_ms, PlannerToolHistoryEntry, RemotePlannerConsentDecision, SkillSummary,
    ToolName,
};
use crate::config::{
    PersistedOriginDecision, RemotePlannerNetworkMode, RemotePlannerOriginRule,
    REMOTE_DATA_POLICY_VERSION,
};
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
fn remote_data_privacy_closure_policy_and_disclosure_matrix_is_bounded() {
    let (app, _config_root) = test_app();
    let (mut core, _secret) = test_core(&app);

    let (challenge, draft) = requirement(&mut core, "session", "analyze this article");
    store(&mut core, challenge.clone(), draft);
    let session = core
        .resolve_pending_remote_planner_consent(
            &challenge.challenge_id,
            &challenge.challenge_digest,
            RemotePlannerConsentDecision::AllowSession,
        )
        .expect("session consent should authorize");
    let PendingConsentResolution::Authorized(session) = session else {
        panic!("session consent returned a terminal outcome");
    };
    assert!(matches!(
        session.prepared.authorization,
        RemotePlannerDataAuthorization::SessionAllow
    ));
    let (profile_name, profile) = core
        .remote_planner_profile_snapshot()
        .expect("test profile should resolve");
    let privacy = core.config.remote_planner_privacy.clone();
    let next = core
        .prepare_remote_planner_request(
            profile_name,
            profile,
            planner_input(&core, "session-next", "analyze this article"),
            &privacy,
        )
        .expect("matching session grant should prepare");
    let RemotePlannerPreparation::Authorized(next) = next else {
        panic!("matching session grant requested consent again");
    };
    assert!(matches!(
        next.authorization,
        RemotePlannerDataAuthorization::SessionAllow
    ));

    core.clear_remote_planner_consent_runtime();
    let (challenge, draft) = requirement(&mut core, "persistent", "analyze this article");
    store(&mut core, challenge.clone(), draft);
    let persistent = core
        .resolve_pending_remote_planner_consent(
            &challenge.challenge_id,
            &challenge.challenge_digest,
            RemotePlannerConsentDecision::AllowPersistent,
        )
        .expect("persistent consent should authorize");
    let PendingConsentResolution::Authorized(persistent) = persistent else {
        panic!("persistent consent returned a terminal outcome");
    };
    assert!(matches!(
        persistent.prepared.authorization,
        RemotePlannerDataAuthorization::PersistentAllow
    ));

    // CR3 P2.6: `AllowPersistent` calls `persist_origin_rule` above, which
    // writes a new rule into `remote_planner_privacy` -- part of
    // `relevant_config_fingerprint` -- *before* this point, but the
    // `planning_snapshot` just captured off `persistent` was captured back
    // when the challenge was drafted, i.e. before that write. If that
    // pre-write snapshot is bound to the eventual `PlannerOutput` and then
    // validated at execution time (as `submit_remote_planner_consent_response`
    // does in production), the config fingerprint captured before the write
    // no longer matches the one observed after it, and the plan spuriously
    // comes back as `NeedsReplan` even though nothing about the *plan
    // itself* went stale -- only `AllowPersistent`'s own bookkeeping did.
    let stop_speaking_after_persistent_allow = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::Stop,
            goal: String::from("stop speaking"),
            target_description: None,
        },
        selected_skills: Vec::new(),
        steps: vec![PlannedStep {
            step_id: String::from("stop-speaking-after-persistent-allow"),
            tool_name: ToolName::StopSpeaking,
            arguments: serde_json::json!({
                "request_id": "stop-speaking-after-persistent-allow",
                "timeout_ms": 1000,
            }),
            purpose: String::from(
                "prove the snapshot AllowPersistent hands back still validates at execution",
            ),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };
    core.register_planning_snapshot(
        &stop_speaking_after_persistent_allow,
        persistent.planning_snapshot,
    )
    .expect("registering the pre-persist snapshot should not fail on its own");
    match core.execute_planner_output(
        String::from("persistent-allow-execute"),
        &stop_speaking_after_persistent_allow,
    ) {
        ExecutionOutcome::Complete { .. } => {}
        ExecutionOutcome::NeedsReplan { trace } => panic!(
            "AllowPersistent's own origin-rule write must not invalidate the \
             planning snapshot it just authorized execution under: {trace:?}"
        ),
        other => panic!("unexpected execution outcome after AllowPersistent: {other:?}"),
    }

    let (profile_name, profile) = core
        .remote_planner_profile_snapshot()
        .expect("test profile should resolve");
    let privacy = core.config.remote_planner_privacy.clone();
    let next = core
        .prepare_remote_planner_request(
            profile_name,
            profile,
            planner_input(&core, "persistent-next", "analyze this article"),
            &privacy,
        )
        .expect("matching persistent allow should prepare");
    let RemotePlannerPreparation::Authorized(next) = next else {
        panic!("matching persistent allow requested consent again");
    };
    assert!(matches!(
        next.authorization,
        RemotePlannerDataAuthorization::PersistentAllow
    ));

    core.config.remote_planner_privacy.origin_rules.clear();
    core.config.remote_planner_privacy.network_mode =
        RemotePlannerNetworkMode::AllowSanitizedNonHighRisk;
    let (profile_name, profile) = core
        .remote_planner_profile_snapshot()
        .expect("test profile should resolve");
    let privacy = core.config.remote_planner_privacy.clone();
    let broad = core
        .prepare_remote_planner_request(
            profile_name,
            profile,
            planner_input(&core, "broad", "analyze this article"),
            &privacy,
        )
        .expect("broad mode should prepare eligible context");
    let RemotePlannerPreparation::Authorized(broad) = broad else {
        panic!("broad mode unexpectedly required consent");
    };
    assert!(matches!(
        broad.authorization,
        RemotePlannerDataAuthorization::GlobalAllow
    ));

    core.config.remote_planner_privacy.network_mode = RemotePlannerNetworkMode::LocalOnly;
    let (profile_name, profile) = core
        .remote_planner_profile_snapshot()
        .expect("test profile should resolve");
    let privacy = core.config.remote_planner_privacy.clone();
    let local_only = core.prepare_remote_planner_request(
        profile_name,
        profile,
        planner_input(&core, "local-only", "analyze this article"),
        &privacy,
    );
    let Err(local_only) = local_only else {
        panic!("local-only unexpectedly prepared non-loopback planning");
    };
    assert_eq!(local_only.code, "remote_data_local_only");

    core.config.remote_planner_privacy.network_mode = RemotePlannerNetworkMode::AskPerOrigin;
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
    let (profile_name, profile) = core
        .remote_planner_profile_snapshot()
        .expect("test profile should resolve");
    let privacy = core.config.remote_planner_privacy.clone();
    let blocked = core.prepare_remote_planner_request(
        profile_name,
        profile,
        planner_input(&core, "blocked", "analyze this article"),
        &privacy,
    );
    let Err(blocked) = blocked else {
        panic!("origin block unexpectedly prepared remote planning");
    };
    assert_eq!(blocked.code, "remote_data_origin_blocked");
    core.config.remote_planner_privacy.origin_rules.clear();

    core.state
        .current_page
        .as_mut()
        .expect("test page should exist")
        .regions
        .push(PageRegion {
            region_id: String::from("payment"),
            role: RegionRole::Paragraph,
            label: None,
            text: String::from("Enter your credit card security code"),
            bbox: None,
            source: RegionSource::Dom,
        });
    let (profile_name, profile) = core
        .remote_planner_profile_snapshot()
        .expect("test profile should resolve");
    let privacy = core.config.remote_planner_privacy.clone();
    let high_risk = core.prepare_remote_planner_request(
        profile_name,
        profile,
        planner_input(&core, "high-risk", "analyze this article"),
        &privacy,
    );
    let Err(high_risk) = high_risk else {
        panic!("high-risk context unexpectedly prepared remote planning");
    };
    assert_eq!(high_risk.code, "remote_data_high_risk_blocked");
    core.state
        .current_page
        .as_mut()
        .expect("test page should exist")
        .regions
        .clear();

    core.state
        .current_page
        .as_mut()
        .expect("test page should exist")
        .url = Some(String::from("file:///tmp/private.html"));
    let (profile_name, profile) = core
        .remote_planner_profile_snapshot()
        .expect("test profile should resolve");
    let privacy = core.config.remote_planner_privacy.clone();
    let opaque = core.prepare_remote_planner_request(
        profile_name,
        profile,
        planner_input(&core, "opaque", "analyze this article"),
        &privacy,
    );
    let Err(opaque) = opaque else {
        panic!("opaque origin unexpectedly prepared remote planning");
    };
    assert_eq!(opaque.code, "remote_data_opaque_origin_blocked");

    core.state
        .current_page
        .as_mut()
        .expect("test page should exist")
        .url = Some(String::from(
        "https://example.com/article?token=secret-query-sentinel#private-fragment",
    ));
    core.state
        .current_page
        .as_mut()
        .expect("test page should exist")
        .regions
        .push(PageRegion {
            region_id: String::from("ocr-private"),
            role: RegionRole::Paragraph,
            label: Some(String::from("ocr-label-sentinel")),
            text: String::from("ocr-content-sentinel"),
            bbox: None,
            source: RegionSource::Ocr,
        });
    let mut hostile_input = planner_input(&core, "hostile-metadata", HOSTILE);
    hostile_input.relevant_skill_summaries.push(SkillSummary {
        name: String::from("skill-name-sentinel"),
        description: String::from("skill-description-sentinel"),
        intent_tags: vec![String::from("skill-intent-sentinel")],
        allowed_tools: None,
        requires_confirmation: false,
        priority: 0,
    });
    hostile_input
        .recent_tool_results
        .push(PlannerToolHistoryEntry {
            tool_name: ToolName::GetPageSnapshot,
            ok: true,
            observation_summary: vec![String::from("tool-observation-sentinel")],
        });
    let (profile_name, profile) = core
        .remote_planner_profile_snapshot()
        .expect("test profile should resolve");
    let privacy = core.config.remote_planner_privacy.clone();
    let prepared = core
        .prepare_remote_planner_request(profile_name, profile, hostile_input, &privacy)
        .expect("hostile but non-high-risk content should be sanitized and challenged");
    let RemotePlannerPreparation::ConsentRequired { challenge, draft } = prepared else {
        panic!("ask mode unexpectedly authorized hostile metadata");
    };
    let challenge_json =
        serde_json::to_string(&challenge).expect("challenge should serialize without excerpts");
    for sentinel in [
        HOSTILE,
        "secret-query-sentinel",
        "private-fragment",
        "ocr-label-sentinel",
        "ocr-content-sentinel",
        "skill-name-sentinel",
        "skill-description-sentinel",
        "skill-intent-sentinel",
        "tool-observation-sentinel",
    ] {
        assert!(!challenge_json.contains(sentinel));
    }
    assert!(challenge_json.contains("ocr_derived_regions"));
    assert!(challenge_json.contains("tool_observation_summaries"));
    assert!(challenge_json.contains("skill_summaries"));
    assert_eq!(challenge.page_origin, ORIGIN);
    assert_eq!(challenge.endpoint_display, "https://api.example.com/v1");
    assert_eq!(draft.disclosure_counts.ocr_derived_region_count, 1);
    assert_eq!(draft.disclosure_counts.tool_history_count, 1);
    assert_eq!(draft.disclosure_counts.skill_summary_count, 1);

    set_profile_base_url(&mut core, "http://localhost:11434/v1");
    core.state
        .current_page
        .as_mut()
        .expect("test page should exist")
        .regions
        .clear();
    let (profile_name, profile) = core
        .remote_planner_profile_snapshot()
        .expect("loopback profile should resolve");
    let privacy = core.config.remote_planner_privacy.clone();
    let loopback = core
        .prepare_remote_planner_request(
            profile_name,
            profile,
            planner_input(&core, "loopback", "analyze this article"),
            &privacy,
        )
        .expect("loopback should prepare without network-remote consent");
    let RemotePlannerPreparation::Authorized(loopback) = loopback else {
        panic!("loopback unexpectedly requested network-remote consent");
    };
    assert!(matches!(
        loopback.authorization,
        RemotePlannerDataAuthorization::Loopback
    ));
}
