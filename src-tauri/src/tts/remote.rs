#[cfg(feature = "remote-openai")]
use std::time::Duration;

use crate::audio_io::RuntimeAudioState;
use crate::config::AppConfig;

#[cfg(feature = "remote-openai")]
use crate::config::{resolve_secret_ref, RemoteProviderKind, RemoteTtsAudioFormat, RemoteTtsProfile};

#[cfg(feature = "remote-openai")]
use super::wav::decode_wav_samples;

use super::{
    CachedSpeechKey, SynthesizedSpeech, TtsController, TtsProviderKind, TtsRuntimeError,
};

#[cfg(feature = "remote-openai")]
use super::{OPENAI_REMOTE_TTS_MAX_SPEED, OPENAI_REMOTE_TTS_MIN_SPEED, OPENAI_TTS_VOICES};

impl TtsController {
    pub(super) fn synthesize_remote(
        &mut self,
        config: &AppConfig,
        runtime_audio: &RuntimeAudioState,
        text: &str,
    ) -> Result<SynthesizedSpeech, TtsRuntimeError> {
        #[cfg(feature = "remote-openai")]
        {
            let profile_name = config
                .providers
                .tts
                .remote_profile
                .as_ref()
                .ok_or(TtsRuntimeError::MissingRemoteProfile)?;
            let profile = config
                .remote_tts_profiles
                .get(profile_name)
                .ok_or_else(|| TtsRuntimeError::MissingRemoteProfileDefinition {
                    profile_name: profile_name.clone(),
                })?;

            match &profile.provider {
                RemoteProviderKind::OpenAi => {
                    self.synthesize_with_openai_remote(profile, runtime_audio, text)
                }
                other => Err(TtsRuntimeError::UnsupportedRemoteProvider {
                    profile_name: profile_name.clone(),
                    provider: format!("{other:?}"),
                }),
            }
        }

        #[cfg(not(feature = "remote-openai"))]
        {
            let _ = config;
            let _ = runtime_audio;
            let _ = text;
            Err(TtsRuntimeError::RemoteTtsFeatureUnavailable)
        }
    }

    #[cfg(feature = "remote-openai")]
    fn synthesize_with_openai_remote(
        &mut self,
        profile: &RemoteTtsProfile,
        runtime_audio: &RuntimeAudioState,
        text: &str,
    ) -> Result<SynthesizedSpeech, TtsRuntimeError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_millis(profile.timeout_ms.max(1)))
            .build()
            .map_err(|error| TtsRuntimeError::RemoteRequestBuildFailed {
                reason: error.to_string(),
            })?;

        let api_key = resolve_secret_ref(&profile.api_key)
            .map_err(|reason| TtsRuntimeError::RemoteSecretUnavailable { reason })?;

        let voice = resolved_remote_voice(runtime_audio, profile)?;
        let cache_key = CachedSpeechKey {
            provider: TtsProviderKind::Remote,
            model_identity: format!(
                "{}|{}|{:?}",
                profile.base_url, profile.model, profile.audio_format
            ),
            voice: voice.clone(),
            playback_speed_bits: runtime_audio.playback_speed.to_bits(),
            text: text.to_string(),
        };
        if let Some(cached) = self.cached_speech(&cache_key) {
            return Ok(cached);
        }
        let response_format = openai_speech_response_format_value(profile.audio_format.clone());
        let endpoint = format!("{}/audio/speech", profile.base_url.trim_end_matches('/'));
        let mut request = client
            .post(endpoint)
            .bearer_auth(api_key)
            .json(&serde_json::json!({
                "model": profile.model,
                "input": text,
                "voice": voice,
                "response_format": response_format,
                "speed": runtime_audio
                    .playback_speed
                    .clamp(OPENAI_REMOTE_TTS_MIN_SPEED, OPENAI_REMOTE_TTS_MAX_SPEED),
            }));
        if let Some(organization) = profile.organization.as_ref() {
            request = request.header(
                "OpenAI-Organization",
                resolve_secret_ref(organization)
                    .map_err(|reason| TtsRuntimeError::RemoteSecretUnavailable { reason })?,
            );
        }
        if let Some(project) = profile.project.as_ref() {
            request = request.header("OpenAI-Project", project);
        }

        let response = request
            .send()
            .and_then(reqwest::blocking::Response::error_for_status)
            .map_err(|error| TtsRuntimeError::RemoteRequestFailed {
                reason: error.to_string(),
            })?;
        let response_bytes =
            response
                .bytes()
                .map_err(|error| TtsRuntimeError::RemoteRequestFailed {
                    reason: error.to_string(),
                })?;

        match profile.audio_format {
            RemoteTtsAudioFormat::Wav => {
                let decoded = decode_wav_samples(response_bytes.as_ref())
                    .map_err(|reason| TtsRuntimeError::RemoteResponseDecodeFailed { reason })?;
                let speech = SynthesizedSpeech {
                    provider: TtsProviderKind::Remote,
                    voice,
                    sample_rate: decoded.sample_rate,
                    channels: decoded.channels,
                    samples: decoded.samples,
                };
                self.store_cached_speech(cache_key, speech.clone());
                Ok(speech)
            }
        }
    }
}

#[cfg(feature = "remote-openai")]
pub(super) fn resolved_remote_voice(
    runtime_audio: &RuntimeAudioState,
    profile: &RemoteTtsProfile,
) -> Result<String, TtsRuntimeError> {
    let configured_voice = profile.voice.trim();
    if configured_voice.is_empty() {
        return Err(TtsRuntimeError::EmptyRemoteVoice);
    }

    let runtime_voice = runtime_audio
        .tts_voice
        .as_deref()
        .map(str::trim)
        .filter(|voice| !voice.is_empty());

    Ok(match runtime_voice {
        Some(voice) if is_openai_builtin_voice(voice) => voice.to_ascii_lowercase(),
        _ => configured_voice.to_string(),
    })
}

#[cfg(feature = "remote-openai")]
fn is_openai_builtin_voice(voice: &str) -> bool {
    let normalized = voice.trim().to_ascii_lowercase();
    OPENAI_TTS_VOICES
        .iter()
        .any(|candidate| *candidate == normalized)
}

#[cfg(feature = "remote-openai")]
pub(super) fn openai_speech_response_format_value(audio_format: RemoteTtsAudioFormat) -> &'static str {
    match audio_format {
        RemoteTtsAudioFormat::Wav => "wav",
    }
}
