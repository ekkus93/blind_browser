#[cfg(feature = "remote-openai")]
use std::time::Duration;

use crate::config::{AppConfig, RemoteAsrProfile, RemoteProviderKind};

#[cfg(feature = "remote-openai")]
use crate::config::resolve_secret_ref_for_endpoint;
#[cfg(feature = "remote-openai")]
use crate::provider_endpoint::ProviderEndpointScope;

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
        RemoteProviderKind::OpenAi => {
            transcribe_with_openai_remote(profile_name, profile, captured_audio)
        }
        other => Err(AsrRuntimeError::UnsupportedRemoteProvider {
            profile_name: profile_name.clone(),
            provider: format!("{other:?}"),
        }),
    }
}

// Blocking multipart upload with a real request-level timeout, mirroring the remote
// TTS path. The previous async-openai + thread::spawn + recv_timeout approach left
// the spawned request running after a caller-side timeout, leaking in-flight
// requests/threads; `reqwest::blocking` with `.timeout(...)` bounds the actual HTTP
// request and surfaces `RemoteRequestTimedOut` via `error.is_timeout()`.
#[cfg(feature = "remote-openai")]
fn transcribe_with_openai_remote(
    profile_name: &str,
    profile: &RemoteAsrProfile,
    captured_audio: &CapturedAudio,
) -> Result<String, AsrRuntimeError> {
    let timeout_ms = profile.timeout_ms.max(1);
    let endpoint_scope = ProviderEndpointScope::parse(&profile.base_url)
        .map_err(|reason| AsrRuntimeError::RemoteRequestBuildFailed { reason })?;
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|error| AsrRuntimeError::RemoteRequestBuildFailed {
            reason: error.to_string(),
        })?;

    let api_key =
        resolve_secret_ref_for_endpoint(&profile.api_key, "asr", profile_name, &endpoint_scope)
            .map_err(|reason| AsrRuntimeError::RemoteSecretUnavailable { reason })?;
    let audio_bytes = captured_audio.to_remote_wav_bytes()?;

    let part = reqwest::blocking::multipart::Part::bytes(audio_bytes)
        .file_name("command.wav")
        .mime_str("audio/wav")
        .map_err(|error| AsrRuntimeError::RemoteRequestBuildFailed {
            reason: error.to_string(),
        })?;

    let mut form = reqwest::blocking::multipart::Form::new()
        .text("model", profile.model.clone())
        .text("response_format", "json")
        .text(
            "temperature",
            format!("{:.3}", (profile.temperature_milli as f32) / 1000.0),
        )
        .part("file", part);
    if let Some(language) = normalized_optional_string(profile.language.as_deref()) {
        form = form.text("language", language);
    }

    let endpoint = endpoint_scope
        .endpoint_url("audio/transcriptions")
        .map_err(|reason| AsrRuntimeError::RemoteRequestBuildFailed { reason })?;
    let mut request = client.post(endpoint).bearer_auth(api_key).multipart(form);
    if let Some(organization) = profile.organization.as_ref() {
        request = request.header(
            "OpenAI-Organization",
            resolve_secret_ref_for_endpoint(organization, "asr", profile_name, &endpoint_scope)
                .map_err(|reason| AsrRuntimeError::RemoteSecretUnavailable { reason })?,
        );
    }
    if let Some(project) = profile.project.as_ref() {
        request = request.header("OpenAI-Project", project);
    }

    let response = request
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|error| {
            if error.is_timeout() {
                AsrRuntimeError::RemoteRequestTimedOut { timeout_ms }
            } else {
                AsrRuntimeError::RemoteRequestFailed {
                    reason: error.to_string(),
                }
            }
        })?;

    let body = response
        .text()
        .map_err(|error| AsrRuntimeError::RemoteRequestFailed {
            reason: error.to_string(),
        })?;
    let parsed: serde_json::Value =
        serde_json::from_str(&body).map_err(|error| AsrRuntimeError::RemoteRequestFailed {
            reason: format!("failed to parse remote transcription response: {error}"),
        })?;

    parse_remote_transcription_text(&parsed)
}

#[cfg(not(feature = "remote-openai"))]
fn transcribe_with_openai_remote(
    _profile_name: &str,
    _profile: &RemoteAsrProfile,
    _captured_audio: &CapturedAudio,
) -> Result<String, AsrRuntimeError> {
    Err(AsrRuntimeError::RemoteAsrFeatureUnavailable)
}

#[cfg(any(feature = "remote-openai", test))]
pub(super) fn normalized_optional_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

#[cfg(any(feature = "remote-openai", test))]
pub(super) fn parse_remote_transcription_text(
    parsed: &serde_json::Value,
) -> Result<String, AsrRuntimeError> {
    let text = parsed
        .get("text")
        .and_then(|value| value.as_str())
        .ok_or_else(|| AsrRuntimeError::RemoteRequestFailed {
            reason: String::from(
                "remote transcription response did not contain a string 'text' field",
            ),
        })?;
    Ok(text.to_string())
}

#[cfg(test)]
mod tests {
    use super::parse_remote_transcription_text;

    #[test]
    fn parse_remote_transcription_text_requires_text_field() {
        let parsed = serde_json::json!({ "duration": 1.0 });

        let error =
            parse_remote_transcription_text(&parsed).expect_err("missing text must be an error");

        assert!(
            error.to_string().contains("text"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn parse_remote_transcription_text_rejects_non_string_text() {
        let parsed = serde_json::json!({ "text": 42 });

        assert!(parse_remote_transcription_text(&parsed).is_err());
    }

    #[test]
    fn parse_remote_transcription_text_allows_empty_string_text() {
        let parsed = serde_json::json!({ "text": "" });

        assert_eq!(parse_remote_transcription_text(&parsed).unwrap(), "");
    }

    #[test]
    fn parse_remote_transcription_text_returns_the_text_value() {
        let parsed = serde_json::json!({ "text": "hello world" });

        assert_eq!(
            parse_remote_transcription_text(&parsed).unwrap(),
            "hello world"
        );
    }
}
