use std::fs;
use std::path::PathBuf;

fn source(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(root.join(path)).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

#[test]
fn remote_speech_provider_boundaries_require_purpose_specific_authorization() {
    let types = source("src/app_core/remote_data_consent/types.rs");
    assert!(types.contains("pub(crate) struct RemoteNarrationAuthorization"));
    assert!(types.contains("pub(crate) struct RemoteMicrophoneAuthorization"));
    assert!(types.contains("pub(super) fn new() -> Self"));
    assert!(
        !types.contains("impl Clone for RemoteNarrationAuthorization")
            && !types.contains("impl Clone for RemoteMicrophoneAuthorization"),
        "remote speech authorizations must not gain reusable Clone implementations",
    );

    let tts = source("src/tts/mod.rs");
    assert!(tts.contains("authorization: Option<RemoteNarrationAuthorization>"));
    assert!(tts.contains("TtsRuntimeError::RemoteConsentMissing"));
    assert!(tts.contains("authorization.ok_or(TtsRuntimeError::RemoteConsentMissing)?"));

    let asr = source("src/asr/mod.rs");
    assert!(asr.contains("authorization: Option<RemoteMicrophoneAuthorization>"));
    assert!(asr.contains("AsrRuntimeError::RemoteConsentMissing"));
    assert!(asr.contains("authorization.ok_or(AsrRuntimeError::RemoteConsentMissing)?"));
}

#[test]
fn microphone_privacy_gate_is_wired_before_capture_and_pending_state_has_no_audio() {
    let voice = source("src/command_handlers/voice_handlers.rs");
    let prepare = voice
        .find("prepare_microphone_request")
        .expect("top-level transcription must evaluate microphone privacy");
    let capture = voice
        .find("begin_transcribe_command")
        .expect("top-level transcription must have an explicit capture phase");
    assert!(
        prepare < capture,
        "remote microphone privacy evaluation must appear before capture begins",
    );

    let types = source("src/app_core/remote_data_consent/types.rs");
    let pending_start = types
        .find("pub(crate) struct PendingMicrophoneConsent")
        .expect("pending microphone consent type must exist");
    let pending_tail = &types[pending_start..];
    let pending_end = pending_tail
        .find("}\n")
        .expect("pending microphone consent type must terminate");
    let pending = &pending_tail[..=pending_end];
    assert!(
        !pending.contains("audio"),
        "pending microphone consent must store metadata, not audio"
    );
    assert!(!pending.contains("samples"));
    assert!(!pending.contains("bytes"));
}

#[test]
fn speech_allow_once_resolves_to_capability_without_once_grant_retry() {
    let narration = source("src/app_core/remote_data_consent/narration_consent.rs");
    let microphone = source("src/app_core/remote_data_consent/microphone_consent.rs");

    for (label, text) in [("narration", narration), ("microphone", microphone)] {
        let allow_once = text
            .find("RemotePlannerConsentDecision::AllowOnce")
            .unwrap_or_else(|| panic!("{label} consent must handle AllowOnce"));
        let tail = &text[allow_once..];
        let next_arm = tail.find("\n            }").unwrap_or(tail.len());
        let arm = &tail[..next_arm];
        assert!(arm.contains("Authorization::new()") || arm.contains("Authorization::new(),"));
        assert!(
            !arm.contains("install_once_grant") && !arm.contains("consume_once_grant"),
            "{label} AllowOnce must resume with a capability rather than an ambient grant/retry",
        );
    }
}
