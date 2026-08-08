//! Evidence that the narration (remote TTS) disclosure kind is gated by the
//! exact same policy engine as the remote planner (CR3 P1.1.6): a high-risk
//! page context blocks remote narration, an origin-block rule blocks remote
//! narration, and a loopback endpoint stays ungated (no consent needed for a
//! local network service). Scoped to `AppCore::prepare_narration_request`'s
//! policy decisions, mirroring `policy_and_disclosure_matrix_tests`'s scope
//! for the planner -- not the full synthesis+playback round trip, which
//! needs a real or fake TTS backend and is exercised separately by
//! `tts::tests`.

use super::*;

use crate::app_core::remote_data_consent::{
    NarrationConsentResolution, NarrationPreparation, NarrationResumeContext,
    RemoteDataDisclosureKind,
};
use crate::commands::RemotePlannerConsentDecision;
use crate::config::{
    PersistedOriginDecision, ProviderMode, RemotePlannerNetworkMode, RemoteProviderKind,
    RemoteTtsAudioFormat, RemoteTtsProfile, SecretRef,
};

const TTS_PROFILE: &str = "openai-tts-narration-evidence";
const TTS_MODEL: &str = "consent-evidence-tts-model";

fn configure_remote_tts(core: &mut crate::app_core::AppCore, base_url: &str) {
    core.config.providers.tts.mode = ProviderMode::Remote;
    core.config.providers.tts.remote_profile = Some(String::from(TTS_PROFILE));
    core.config.remote_tts_profiles.insert(
        String::from(TTS_PROFILE),
        RemoteTtsProfile {
            provider: RemoteProviderKind::OpenAi,
            base_url: base_url.to_string(),
            model: String::from(TTS_MODEL),
            api_key: SecretRef::FromEnv {
                from_env: String::from("BLIND_BROWSER_TEST_UNUSED_TTS_KEY"),
            },
            organization: None,
            project: None,
            voice: String::from("alloy"),
            audio_format: RemoteTtsAudioFormat::Wav,
            timeout_ms: 5_000,
        },
    );
}

fn resume(region_id: &str) -> NarrationResumeContext {
    NarrationResumeContext::Region {
        region_id: region_id.to_string(),
        interrupt_current: false,
    }
}

#[test]
#[cfg_attr(
    any(windows, target_os = "linux"),
    ignore = "real Wry AppCore fixture must run in a process-isolated test invocation"
)]
#[cfg_attr(
    not(any(windows, target_os = "linux")),
    ignore = "real Wry AppCore fixture requires Tauri's any-thread desktop builder"
)]
fn remote_narration_consent_policy_matrix_is_fail_closed() {
    let (app, _config_root) = test_app();
    let (mut core, _secret) = test_core(&app);
    configure_remote_tts(&mut core, "https://api.example.com/v1");

    // A high-risk page context blocks remote narration, exactly like it
    // blocks the remote planner -- "read this page" must not ship raw text
    // off-device on a page classified high-risk.
    core.state
        .current_page
        .as_mut()
        .expect("test page should exist")
        .regions
        .push(crate::page_model::PageRegion {
            region_id: String::from("payment"),
            role: crate::page_model::RegionRole::Paragraph,
            label: None,
            text: String::from("Enter your credit card security code"),
            bbox: None,
            source: crate::page_model::RegionSource::Dom,
        });
    let high_risk = core
        .prepare_narration_request(
            "read this region aloud",
            String::from("narration-high-risk"),
            resume("payment"),
        )
        .expect_err("high-risk page context must block remote narration");
    assert_eq!(high_risk.code, "remote_data_high_risk_blocked");
    core.state
        .current_page
        .as_mut()
        .expect("test page should exist")
        .regions
        .clear();

    // A persistent origin-block rule blocks remote narration for that
    // origin, independent of whatever the planner's own origin rules say
    // (narration has its own, separate rule store).
    core.config.remote_narration_privacy.origin_rules.push(
        crate::config::RemotePlannerOriginRule {
            page_origin: String::from(ORIGIN),
            decision: PersistedOriginDecision::Block,
            endpoint_scope: None,
            policy_version: crate::config::REMOTE_DATA_POLICY_VERSION,
            created_at_ms: crate::commands::current_timestamp_ms(),
        },
    );
    let blocked = core
        .prepare_narration_request(
            "read this region aloud",
            String::from("narration-blocked"),
            resume("intro"),
        )
        .expect_err("an origin-block rule must block remote narration");
    assert_eq!(blocked.code, "remote_data_origin_blocked");
    core.config.remote_narration_privacy.origin_rules.clear();

    // A planner-scoped persistent allow for this same origin must NOT
    // silently authorize narration too -- the two disclosure kinds keep
    // independent grant/origin-rule stores by design (see
    // AppConfig::remote_narration_privacy's doc comment).
    core.config
        .remote_planner_privacy
        .origin_rules
        .push(crate::config::RemotePlannerOriginRule {
            page_origin: String::from(ORIGIN),
            decision: PersistedOriginDecision::Allow,
            endpoint_scope: Some(String::from("https://api.example.com/v1")),
            policy_version: crate::config::REMOTE_DATA_POLICY_VERSION,
            created_at_ms: crate::commands::current_timestamp_ms(),
        });
    let challenge = match core
        .prepare_narration_request(
            "read this region aloud",
            String::from("narration-cross-kind"),
            resume("intro"),
        )
        .expect("ask-per-origin mode should still evaluate, not error, for narration")
    {
        NarrationPreparation::ConsentRequired { challenge } => *challenge,
        NarrationPreparation::Authorized(_) => {
            panic!("a planner-only origin allow must not silently authorize narration")
        }
    };
    core.config.remote_planner_privacy.origin_rules.clear();

    let once = core
        .resolve_narration_consent(
            &challenge.challenge_id,
            &challenge.challenge_digest,
            RemotePlannerConsentDecision::AllowOnce,
        )
        .expect("allow-once should authorize the exact pending narration request");
    assert!(matches!(
        once,
        NarrationConsentResolution::Authorized { .. }
    ));
    assert!(
        core.remote_narration_ephemeral_grants.is_empty(),
        "allow-once must not install an ambient narration grant"
    );
    let replay = core
        .resolve_narration_consent(
            &challenge.challenge_id,
            &challenge.challenge_digest,
            RemotePlannerConsentDecision::AllowOnce,
        )
        .expect_err("a consumed narration challenge must not authorize twice");
    assert_eq!(replay.code, "remote_data_consent_missing");

    let fresh = core
        .prepare_narration_request(
            "read this region aloud again",
            String::from("narration-after-once"),
            resume("intro"),
        )
        .expect("a later narration request should evaluate normally");
    assert!(matches!(
        fresh,
        NarrationPreparation::ConsentRequired { .. }
    ));

    let policy_changed = core
        .set_remote_speech_privacy_network_mode(
            RemoteDataDisclosureKind::NarrationText,
            RemotePlannerNetworkMode::LocalOnly,
        )
        .expect("narration privacy mode should persist through the typed speech operation");
    assert!(policy_changed);
    assert!(
        core.pending_narration_consent.is_none(),
        "changing narration privacy policy must invalidate the pending challenge"
    );
    assert!(core.remote_narration_ephemeral_grants.is_empty());

    // A loopback narration endpoint stays ungated -- no data leaves the
    // device, so no consent is required, the same as the planner's own
    // loopback exemption.
    configure_remote_tts(&mut core, "http://localhost:11434/v1");
    let loopback = core
        .prepare_narration_request(
            "read this region aloud",
            String::from("narration-loopback"),
            resume("intro"),
        )
        .expect("loopback narration endpoint should not require consent");
    assert!(
        matches!(loopback, NarrationPreparation::Authorized(_)),
        "loopback narration endpoint unexpectedly required consent"
    );
}
