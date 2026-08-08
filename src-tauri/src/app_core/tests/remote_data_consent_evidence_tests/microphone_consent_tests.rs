//! Evidence that remote ASR is authorized before capture and that one-shot
//! microphone consent resumes exactly one pending operation without installing
//! a reusable grant.

use super::*;

use crate::app_core::remote_data_consent::{
    MicrophoneConsentResolution, MicrophonePreparation, MicrophoneResumeContext,
    RemoteDataDisclosureKind,
};
use crate::commands::{
    RemotePlannerConsentDecision, TranscribeCommandInput, TranscriptionStopMode,
};
use crate::config::{
    ProviderMode, RemoteAsrProfile, RemotePlannerNetworkMode, RemoteProviderKind, SecretRef,
};

const ASR_PROFILE: &str = "openai-asr-consent-evidence";
const ASR_MODEL: &str = "consent-evidence-asr-model";

fn configure_remote_asr(core: &mut crate::app_core::AppCore, base_url: &str) {
    core.config.providers.asr.mode = ProviderMode::Remote;
    core.config.providers.asr.remote_profile = Some(String::from(ASR_PROFILE));
    core.config.remote_asr_profiles.insert(
        String::from(ASR_PROFILE),
        RemoteAsrProfile {
            provider: RemoteProviderKind::OpenAi,
            base_url: base_url.to_string(),
            model: String::from(ASR_MODEL),
            api_key: SecretRef::FromEnv {
                from_env: String::from("BLIND_BROWSER_TEST_UNUSED_ASR_KEY"),
            },
            organization: None,
            project: None,
            language: Some(String::from("en")),
            temperature_milli: 0,
            timeout_ms: 5_000,
        },
    );
    core.config.remote_microphone_privacy.network_mode = RemotePlannerNetworkMode::AskPerOrigin;
    core.config.remote_microphone_privacy.origin_rules.clear();
}

fn transcribe(request_id: &str) -> MicrophoneResumeContext {
    MicrophoneResumeContext::Transcribe {
        input: TranscribeCommandInput {
            request_id: request_id.to_string(),
            timeout_ms: Some(5_000),
            max_duration_ms: Some(1_500),
            stop_mode: TranscriptionStopMode::AutoStop,
        },
        execute_after: false,
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
fn remote_microphone_consent_is_pre_capture_one_shot_and_fail_closed() {
    let (app, _config_root) = test_app();
    let (mut core, _secret) = test_core(&app);
    configure_remote_asr(&mut core, "https://api.example.com/v1");

    assert!(!core.state.listening.is_listening);
    let challenge = match core
        .prepare_microphone_request(transcribe("microphone-once"))
        .expect("ask-per-origin should create a microphone consent challenge")
    {
        MicrophonePreparation::ConsentRequired { challenge } => *challenge,
        MicrophonePreparation::Authorized(_) => {
            panic!("remote microphone was authorized without consent")
        }
        MicrophonePreparation::Local => panic!("remote ASR was misclassified as local"),
    };

    assert!(
        !core.state.listening.is_listening,
        "creating a remote microphone challenge must not start capture"
    );
    assert_eq!(
        challenge.disclosure_counts.microphone_audio_duration_ms,
        1_500
    );

    let resolution = core
        .resolve_microphone_consent(
            &challenge.challenge_id,
            &challenge.challenge_digest,
            RemotePlannerConsentDecision::AllowOnce,
        )
        .expect("allow-once should authorize the exact pending microphone request");
    assert!(matches!(
        resolution,
        MicrophoneConsentResolution::Authorized { .. }
    ));
    assert!(
        core.remote_microphone_ephemeral_grants.is_empty(),
        "allow-once must not install a reusable microphone grant"
    );
    assert!(
        !core.state.listening.is_listening,
        "resolving consent alone must not capture before the authorized resume path runs"
    );

    let replay = core
        .resolve_microphone_consent(
            &challenge.challenge_id,
            &challenge.challenge_digest,
            RemotePlannerConsentDecision::AllowOnce,
        )
        .expect_err("a consumed microphone challenge must not authorize twice");
    assert_eq!(replay.code, "remote_data_consent_missing");

    // A fresh request still needs consent: AllowOnce authorized only the
    // operation returned above and did not leak into ambient policy state.
    let next = core
        .prepare_microphone_request(transcribe("microphone-next"))
        .expect("a later request should still evaluate normally");
    assert!(matches!(
        next,
        MicrophonePreparation::ConsentRequired { .. }
    ));

    let policy_changed = core
        .set_remote_speech_privacy_network_mode(
            RemoteDataDisclosureKind::MicrophoneAudio,
            RemotePlannerNetworkMode::LocalOnly,
        )
        .expect("microphone privacy mode should persist through the typed speech operation");
    assert!(policy_changed);
    assert!(
        core.pending_microphone_consent.is_none(),
        "changing microphone privacy policy must invalidate the pending challenge"
    );
    assert!(core.remote_microphone_ephemeral_grants.is_empty());
    assert!(core.active_remote_microphone_authorization.is_none());

    let blocked = core
        .prepare_microphone_request(transcribe("microphone-local-only"))
        .expect_err("local-only must reject a non-loopback remote ASR endpoint");
    assert_eq!(blocked.code, "remote_data_local_only");

    // Loopback is local transport even when the provider mode is Remote and
    // remains usable under local-only privacy policy.
    configure_remote_asr(&mut core, "http://127.0.0.1:11434/v1");
    core.config.remote_microphone_privacy.network_mode = RemotePlannerNetworkMode::LocalOnly;
    let loopback = core
        .prepare_microphone_request(transcribe("microphone-loopback"))
        .expect("loopback ASR should remain local-only compatible");
    assert!(matches!(loopback, MicrophonePreparation::Authorized(_)));
}
