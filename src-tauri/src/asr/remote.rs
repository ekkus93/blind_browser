#[cfg(feature = "remote-openai")]
use std::sync::mpsc;
#[cfg(feature = "remote-openai")]
use std::thread;
#[cfg(feature = "remote-openai")]
use std::time::Duration;

use crate::config::{AppConfig, RemoteAsrProfile, RemoteProviderKind};

#[cfg(feature = "remote-openai")]
use crate::config::resolve_secret_ref;

use super::processing::CapturedAudio;
use super::AsrRuntimeError;

// Free function (no `AsrController` state): pure over `(config, captured_audio)`, so
// it can run with the `AppCore` lock released. See [`super::transcribe_captured_audio`].
pub(super) fn transcribe_remote(
    config: &AppConfig,
    captured_audio: &CapturedAudio,
) -> Result<String, AsrRuntimeError> {
    let profile_name = config
        .providers
        .asr
        .remote_profile
        .as_ref()
        .ok_or(AsrRuntimeError::MissingRemoteProfile)?;
    let profile = config
        .remote_asr_profiles
        .get(profile_name)
        .ok_or_else(|| AsrRuntimeError::MissingRemoteProfileDefinition {
            profile_name: profile_name.clone(),
        })?;

    match &profile.provider {
        RemoteProviderKind::OpenAi => transcribe_with_openai_remote(profile, captured_audio),
        other => Err(AsrRuntimeError::UnsupportedRemoteProvider {
            profile_name: profile_name.clone(),
            provider: format!("{other:?}"),
        }),
    }
}

#[cfg(feature = "remote-openai")]
fn transcribe_with_openai_remote(
    profile: &RemoteAsrProfile,
    captured_audio: &CapturedAudio,
) -> Result<String, AsrRuntimeError> {
    use async_openai::{config::OpenAIConfig, Client};

    let api_key = resolve_secret_ref(&profile.api_key)
        .map_err(|reason| AsrRuntimeError::RemoteSecretUnavailable { reason })?;
    let audio_bytes = captured_audio.to_remote_wav_bytes()?;

    let mut openai_config = OpenAIConfig::new()
        .with_api_base(profile.base_url.clone())
        .with_api_key(api_key);
    if let Some(organization) = profile.organization.as_ref() {
        openai_config = openai_config.with_org_id(
            resolve_secret_ref(organization)
                .map_err(|reason| AsrRuntimeError::RemoteSecretUnavailable { reason })?,
        );
    }
    if let Some(project) = profile.project.as_ref() {
        openai_config = openai_config.with_project_id(project.clone());
    }

    let request = build_openai_transcription_request(profile, audio_bytes)?;
    let timeout_ms = profile.timeout_ms.max(1);
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let client = Client::with_config(openai_config);
        let result = futures::executor::block_on(client.audio().transcription().create(request))
            .map(|response| response.text)
            .map_err(|error| AsrRuntimeError::RemoteRequestFailed {
                reason: error.to_string(),
            });
        let _ = sender.send(result);
    });

    match receiver.recv_timeout(Duration::from_millis(timeout_ms)) {
        Ok(result) => result,
        Err(mpsc::RecvTimeoutError::Timeout) => {
            Err(AsrRuntimeError::RemoteRequestTimedOut { timeout_ms })
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(AsrRuntimeError::RemoteRequestFailed {
            reason: String::from("remote transcription task ended without a response"),
        }),
    }
}

#[cfg(not(feature = "remote-openai"))]
fn transcribe_with_openai_remote(
    _profile: &RemoteAsrProfile,
    _captured_audio: &CapturedAudio,
) -> Result<String, AsrRuntimeError> {
    Err(AsrRuntimeError::RemoteAsrFeatureUnavailable)
}

#[cfg(feature = "remote-openai")]
fn build_openai_transcription_request(
    profile: &RemoteAsrProfile,
    audio_bytes: Vec<u8>,
) -> Result<async_openai::types::audio::CreateTranscriptionRequest, AsrRuntimeError> {
    use async_openai::types::audio::{
        AudioInput, AudioResponseFormat, CreateTranscriptionRequestArgs,
    };

    let mut request = CreateTranscriptionRequestArgs::default();
    request.file(AudioInput::from_vec_u8(
        String::from("command.wav"),
        audio_bytes,
    ));
    request.model(profile.model.clone());
    request.response_format(AudioResponseFormat::Json);
    request.temperature((profile.temperature_milli as f32) / 1000.0);
    if let Some(language) = normalized_optional_string(profile.language.as_deref()) {
        request.language(language);
    }

    request
        .build()
        .map_err(|error| AsrRuntimeError::RemoteRequestBuildFailed {
            reason: error.to_string(),
        })
}

#[cfg(any(feature = "remote-openai", test))]
pub(super) fn normalized_optional_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}
