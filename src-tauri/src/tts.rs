use std::fs;
use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::audio_io::RuntimeAudioState;
use crate::config::{AppConfig, ProviderMode, RemoteProviderKind, RemoteTtsProfile, SecretRef};

#[cfg(feature = "local-tts")]
use kitten_tts::model::KittenTTS;

pub const KITTEN_TTS_BACKEND: &str = "kitten_tts_rs";
pub const KITTEN_TTS_SAMPLE_RATE: u32 = 24_000;
pub const KITTEN_TTS_CHANNELS: u16 = 1;
pub const KITTEN_TTS_VOICES: &[&str] = &[
    "Bella", "Jasper", "Luna", "Bruno", "Rosie", "Hugo", "Kiki", "Leo",
];
pub const OPENAI_TTS_VOICES: &[&str] = &[
    "alloy", "ash", "ballad", "coral", "echo", "fable", "onyx", "nova", "sage", "shimmer", "verse",
    "marin", "cedar",
];
const OPENAI_REMOTE_TTS_MIN_SPEED: f32 = 0.25;
const OPENAI_REMOTE_TTS_MAX_SPEED: f32 = 4.0;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum TtsProviderKind {
    Local,
    Remote,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct TtsSettings {
    pub provider: TtsProviderKind,
    pub voice: Option<String>,
    pub playback_speed: f32,
}

impl Default for TtsSettings {
    fn default() -> Self {
        Self {
            provider: TtsProviderKind::Local,
            voice: Some(String::from("Bruno")),
            playback_speed: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SynthesizedSpeech {
    pub provider: TtsProviderKind,
    pub voice: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: Vec<f32>,
}

#[derive(Debug, Error)]
pub enum TtsRuntimeError {
    #[error("narration text was empty after normalization")]
    EmptyNarrationText,
    #[error("tts local profile is not configured")]
    MissingLocalProfile,
    #[error("tts local profile '{profile_name}' was not found")]
    MissingLocalProfileDefinition { profile_name: String },
    #[error("tts remote profile is not configured")]
    MissingRemoteProfile,
    #[error("tts remote profile '{profile_name}' was not found")]
    MissingRemoteProfileDefinition { profile_name: String },
    #[error("tts remote profile '{profile_name}' uses unsupported provider '{provider}'")]
    UnsupportedRemoteProvider {
        profile_name: String,
        provider: String,
    },
    #[error("remote tts profile requires a non-empty voice")]
    EmptyRemoteVoice,
    #[error("remote tts secret could not be resolved: {reason}")]
    RemoteSecretUnavailable { reason: String },
    #[error("remote tts requires the 'remote-openai' feature to be enabled")]
    RemoteTtsFeatureUnavailable,
    #[error("failed to build the remote tts request: {reason}")]
    RemoteRequestBuildFailed { reason: String },
    #[error("remote tts request failed: {reason}")]
    RemoteRequestFailed { reason: String },
    #[error("remote tts audio format '{audio_format}' is not supported")]
    UnsupportedRemoteAudioFormat { audio_format: String },
    #[error("failed to decode the remote tts audio response: {reason}")]
    RemoteResponseDecodeFailed { reason: String },
    #[error("unsupported local tts backend '{backend}'")]
    UnsupportedLocalBackend { backend: String },
    #[error("local tts requires the 'local-tts' feature to be enabled")]
    LocalTtsFeatureUnavailable,
    #[error("local tts model path must not be empty")]
    EmptyLocalModelPath,
    #[error("local tts model path does not exist: {model_path}")]
    MissingLocalModelPath { model_path: String },
    #[error(
        "local tts sample_rate {sample_rate} is not supported; kitten_tts_rs outputs {KITTEN_TTS_SAMPLE_RATE} Hz audio"
    )]
    UnsupportedLocalSampleRate { sample_rate: u32 },
    #[error("failed to load the local tts model from {model_path}: {reason}")]
    LocalModelLoad { model_path: String, reason: String },
    #[error("failed to synthesize narration audio: {reason}")]
    SynthesisFailed { reason: String },
}

pub struct TtsController {
    #[cfg(feature = "local-tts")]
    local_model: Option<CachedLocalTtsModel>,
}

impl TtsController {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "local-tts")]
            local_model: None,
        }
    }

    pub fn synthesize_narration(
        &mut self,
        config: &AppConfig,
        runtime_audio: &RuntimeAudioState,
        text: &str,
    ) -> Result<SynthesizedSpeech, TtsRuntimeError> {
        let normalized_text = text.trim();
        if normalized_text.is_empty() {
            return Err(TtsRuntimeError::EmptyNarrationText);
        }

        match config.providers.tts.mode {
            ProviderMode::Local => self.synthesize_local(config, runtime_audio, normalized_text),
            ProviderMode::Remote => self.synthesize_remote(config, runtime_audio, normalized_text),
        }
    }

    fn synthesize_remote(
        &mut self,
        config: &AppConfig,
        runtime_audio: &RuntimeAudioState,
        text: &str,
    ) -> Result<SynthesizedSpeech, TtsRuntimeError> {
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

    #[cfg(feature = "remote-openai")]
    fn synthesize_with_openai_remote(
        &mut self,
        profile: &RemoteTtsProfile,
        runtime_audio: &RuntimeAudioState,
        text: &str,
    ) -> Result<SynthesizedSpeech, TtsRuntimeError> {
        use async_openai::types::audio::{
            CreateSpeechRequestArgs, SpeechModel, SpeechResponseFormat, Voice,
        };
        use async_openai::{config::OpenAIConfig, Client};

        let api_key = resolve_secret_ref(&profile.api_key)
            .map_err(|reason| TtsRuntimeError::RemoteSecretUnavailable { reason })?;

        let mut openai_config = OpenAIConfig::new()
            .with_api_base(profile.base_url.clone())
            .with_api_key(api_key);
        if let Some(organization) = profile.organization.as_ref() {
            openai_config = openai_config.with_org_id(
                resolve_secret_ref(organization)
                    .map_err(|reason| TtsRuntimeError::RemoteSecretUnavailable { reason })?,
            );
        }
        if let Some(project) = profile.project.as_ref() {
            openai_config = openai_config.with_project_id(project.clone());
        }

        let voice = resolved_remote_voice(runtime_audio, profile)?;
        let response_format = parse_openai_speech_response_format(&profile.audio_format)?;
        let request = CreateSpeechRequestArgs::default()
            .input(text.to_string())
            .model(SpeechModel::Other(profile.model.clone()))
            .voice(Voice::Other(voice.clone()))
            .response_format(response_format)
            .speed(
                runtime_audio
                    .playback_speed
                    .clamp(OPENAI_REMOTE_TTS_MIN_SPEED, OPENAI_REMOTE_TTS_MAX_SPEED),
            )
            .build()
            .map_err(|error| TtsRuntimeError::RemoteRequestBuildFailed {
                reason: error.to_string(),
            })?;

        let client = Client::with_config(openai_config);
        let response = futures::executor::block_on(client.audio().speech().create(request))
            .map_err(|error| TtsRuntimeError::RemoteRequestFailed {
                reason: error.to_string(),
            })?;

        match response_format {
            SpeechResponseFormat::Wav => {
                let decoded = decode_wav_samples(response.bytes.as_ref())
                    .map_err(|reason| TtsRuntimeError::RemoteResponseDecodeFailed { reason })?;
                Ok(SynthesizedSpeech {
                    provider: TtsProviderKind::Remote,
                    voice,
                    sample_rate: decoded.sample_rate,
                    channels: decoded.channels,
                    samples: decoded.samples,
                })
            }
            _ => Err(TtsRuntimeError::UnsupportedRemoteAudioFormat {
                audio_format: profile.audio_format.clone(),
            }),
        }
    }

    #[cfg(not(feature = "remote-openai"))]
    fn synthesize_with_openai_remote(
        &mut self,
        _profile: &RemoteTtsProfile,
        _runtime_audio: &RuntimeAudioState,
        _text: &str,
    ) -> Result<SynthesizedSpeech, TtsRuntimeError> {
        Err(TtsRuntimeError::RemoteTtsFeatureUnavailable)
    }

    fn synthesize_local(
        &mut self,
        config: &AppConfig,
        runtime_audio: &RuntimeAudioState,
        text: &str,
    ) -> Result<SynthesizedSpeech, TtsRuntimeError> {
        let profile_name = config
            .providers
            .tts
            .local_profile
            .as_ref()
            .ok_or(TtsRuntimeError::MissingLocalProfile)?;
        let profile = config.local_tts_profiles.get(profile_name).ok_or_else(|| {
            TtsRuntimeError::MissingLocalProfileDefinition {
                profile_name: profile_name.clone(),
            }
        })?;

        if profile.backend != KITTEN_TTS_BACKEND {
            return Err(TtsRuntimeError::UnsupportedLocalBackend {
                backend: profile.backend.clone(),
            });
        }

        if profile.sample_rate != KITTEN_TTS_SAMPLE_RATE {
            return Err(TtsRuntimeError::UnsupportedLocalSampleRate {
                sample_rate: profile.sample_rate,
            });
        }

        let voice = resolved_voice(runtime_audio, &profile.default_voice);
        let model_dir = normalized_model_path(&profile.model_path)?;
        let samples = self.generate_local_samples(&model_dir, text, &voice, runtime_audio)?;

        Ok(SynthesizedSpeech {
            provider: TtsProviderKind::Local,
            voice,
            sample_rate: profile.sample_rate,
            channels: KITTEN_TTS_CHANNELS,
            samples,
        })
    }

    #[cfg(feature = "local-tts")]
    fn generate_local_samples(
        &mut self,
        model_dir: &Path,
        text: &str,
        voice: &str,
        runtime_audio: &RuntimeAudioState,
    ) -> Result<Vec<f32>, TtsRuntimeError> {
        let model = self.local_model(model_dir)?;
        model
            .generate(text, voice, runtime_audio.playback_speed, true)
            .map_err(|error| TtsRuntimeError::SynthesisFailed {
                reason: error.to_string(),
            })
    }

    #[cfg(not(feature = "local-tts"))]
    fn generate_local_samples(
        &mut self,
        _model_dir: &Path,
        _text: &str,
        _voice: &str,
        _runtime_audio: &RuntimeAudioState,
    ) -> Result<Vec<f32>, TtsRuntimeError> {
        Err(TtsRuntimeError::LocalTtsFeatureUnavailable)
    }

    #[cfg(feature = "local-tts")]
    fn local_model(&mut self, model_dir: &Path) -> Result<&mut KittenTTS, TtsRuntimeError> {
        let needs_reload = self
            .local_model
            .as_ref()
            .is_none_or(|cached| cached.model_dir != model_dir);

        if needs_reload {
            let model = KittenTTS::from_dir(model_dir).map_err(|error| {
                TtsRuntimeError::LocalModelLoad {
                    model_path: model_dir.display().to_string(),
                    reason: error.to_string(),
                }
            })?;
            self.local_model = Some(CachedLocalTtsModel {
                model_dir: model_dir.to_path_buf(),
                model,
            });
        }

        Ok(&mut self
            .local_model
            .as_mut()
            .expect("local model should be present after load")
            .model)
    }
}

impl Default for TtsController {
    fn default() -> Self {
        Self::new()
    }
}

fn resolved_voice(runtime_audio: &RuntimeAudioState, default_voice: &str) -> String {
    runtime_audio
        .tts_voice
        .as_deref()
        .map(str::trim)
        .filter(|voice| !voice.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| default_voice.to_string())
}

fn resolved_remote_voice(
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

fn is_openai_builtin_voice(voice: &str) -> bool {
    let normalized = voice.trim().to_ascii_lowercase();
    OPENAI_TTS_VOICES
        .iter()
        .any(|candidate| *candidate == normalized)
}

fn parse_openai_speech_response_format(
    audio_format: &str,
) -> Result<async_openai::types::audio::SpeechResponseFormat, TtsRuntimeError> {
    use async_openai::types::audio::SpeechResponseFormat;

    match audio_format.trim().to_ascii_lowercase().as_str() {
        "wav" => Ok(SpeechResponseFormat::Wav),
        _ => Err(TtsRuntimeError::UnsupportedRemoteAudioFormat {
            audio_format: audio_format.trim().to_string(),
        }),
    }
}

fn resolve_secret_ref(secret_ref: &SecretRef) -> Result<String, String> {
    match secret_ref {
        SecretRef::FromEnv { from_env } => std::env::var(from_env)
            .map(|value| value.trim().to_string())
            .map_err(|error| format!("failed to read environment variable '{from_env}': {error}")),
        SecretRef::FromFile { from_file } => fs::read_to_string(from_file)
            .map(|value| value.trim().to_string())
            .map_err(|error| format!("failed to read secret file '{from_file}': {error}")),
        SecretRef::Inline { inline } => Ok(inline.trim().to_string()),
    }
    .and_then(|value| {
        if value.is_empty() {
            Err(String::from("resolved secret value was empty"))
        } else {
            Ok(value)
        }
    })
}

fn normalized_model_path(model_path: &str) -> Result<PathBuf, TtsRuntimeError> {
    let trimmed = model_path.trim();
    if trimmed.is_empty() {
        return Err(TtsRuntimeError::EmptyLocalModelPath);
    }

    let path = PathBuf::from(trimmed);
    if !path.exists() {
        return Err(TtsRuntimeError::MissingLocalModelPath {
            model_path: trimmed.to_string(),
        });
    }

    Ok(path)
}

struct DecodedWav {
    sample_rate: u32,
    channels: u16,
    samples: Vec<f32>,
}

fn decode_wav_samples(bytes: &[u8]) -> Result<DecodedWav, String> {
    if bytes.len() < 12 {
        return Err(String::from("response was too short to be a WAV file"));
    }
    if &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(String::from("response was not a RIFF/WAVE file"));
    }

    let mut cursor = 12usize;
    let mut fmt_chunk = None;
    let mut data_chunk = None;
    while cursor + 8 <= bytes.len() {
        let chunk_id = &bytes[cursor..cursor + 4];
        let chunk_size = u32::from_le_bytes(
            bytes[cursor + 4..cursor + 8]
                .try_into()
                .expect("chunk size should be four bytes"),
        ) as usize;
        let chunk_start = cursor + 8;
        let chunk_end = chunk_start.saturating_add(chunk_size);
        if chunk_end > bytes.len() {
            return Err(String::from("WAV chunk length exceeded the response size"));
        }

        match chunk_id {
            b"fmt " => fmt_chunk = Some(&bytes[chunk_start..chunk_end]),
            b"data" => data_chunk = Some(&bytes[chunk_start..chunk_end]),
            _ => {}
        }

        cursor = chunk_end;
        if chunk_size % 2 == 1 {
            cursor = cursor.saturating_add(1);
        }
    }

    let fmt_chunk =
        fmt_chunk.ok_or_else(|| String::from("WAV response did not include a fmt chunk"))?;
    let data_chunk =
        data_chunk.ok_or_else(|| String::from("WAV response did not include a data chunk"))?;
    if fmt_chunk.len() < 16 {
        return Err(String::from("WAV fmt chunk was too short"));
    }

    let format_tag = u16::from_le_bytes(fmt_chunk[0..2].try_into().expect("format tag size"));
    let channels = u16::from_le_bytes(fmt_chunk[2..4].try_into().expect("channel count size"));
    let sample_rate = u32::from_le_bytes(fmt_chunk[4..8].try_into().expect("sample rate size"));
    let bits_per_sample = u16::from_le_bytes(fmt_chunk[14..16].try_into().expect("bit depth size"));

    if channels == 0 {
        return Err(String::from("WAV response reported zero channels"));
    }
    if sample_rate == 0 {
        return Err(String::from("WAV response reported zero sample rate"));
    }

    let samples = match (format_tag, bits_per_sample) {
        (1, 8) => data_chunk
            .iter()
            .map(|sample| (*sample as f32 - 128.0) / 128.0)
            .collect(),
        (1, 16) => data_chunk
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]) as f32 / i16::MAX as f32)
            .collect(),
        (1, 24) => data_chunk
            .chunks_exact(3)
            .map(|chunk| {
                let signed = i32::from_le_bytes([
                    chunk[0],
                    chunk[1],
                    chunk[2],
                    if chunk[2] & 0x80 != 0 { 0xFF } else { 0x00 },
                ]);
                signed as f32 / 8_388_607.0
            })
            .collect(),
        (1, 32) => data_chunk
            .chunks_exact(4)
            .map(|chunk| {
                i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) as f32
                    / i32::MAX as f32
            })
            .collect(),
        (3, 32) => data_chunk
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect(),
        _ => {
            return Err(format!(
                "WAV response used unsupported encoding format_tag={format_tag}, bits_per_sample={bits_per_sample}"
            ))
        }
    };

    Ok(DecodedWav {
        sample_rate,
        channels,
        samples,
    })
}

#[cfg(feature = "local-tts")]
struct CachedLocalTtsModel {
    model_dir: PathBuf,
    model: KittenTTS,
}

#[cfg(test)]
mod tests {
    use super::{
        decode_wav_samples, normalized_model_path, parse_openai_speech_response_format,
        resolved_remote_voice, resolved_voice, KITTEN_TTS_SAMPLE_RATE,
    };
    use crate::audio_io::RuntimeAudioState;
    use crate::config::{LocalTtsProfile, RemoteProviderKind, RemoteTtsProfile, SecretRef};

    #[test]
    fn resolved_voice_prefers_runtime_voice_over_profile_default() {
        let runtime_audio = RuntimeAudioState {
            tts_voice: Some(String::from("Rosie")),
            ..RuntimeAudioState::default()
        };

        assert_eq!(resolved_voice(&runtime_audio, "Bruno"), "Rosie");
    }

    #[test]
    fn resolved_remote_voice_falls_back_to_profile_voice_for_local_only_defaults() {
        let runtime_audio = RuntimeAudioState {
            tts_voice: Some(String::from("Bruno")),
            ..RuntimeAudioState::default()
        };
        let profile = RemoteTtsProfile {
            provider: RemoteProviderKind::OpenAi,
            base_url: String::from("https://api.openai.com/v1"),
            model: String::from("gpt-4o-mini-tts"),
            api_key: SecretRef::Inline {
                inline: String::from("test"),
            },
            organization: None,
            project: None,
            voice: String::from("alloy"),
            audio_format: String::from("wav"),
            timeout_ms: 30_000,
        };

        assert_eq!(
            resolved_remote_voice(&runtime_audio, &profile).expect("voice resolution should work"),
            "alloy"
        );
    }

    #[test]
    fn parse_openai_speech_response_format_rejects_non_wav() {
        let error =
            parse_openai_speech_response_format("mp3").expect_err("mp3 should be unsupported");
        assert_eq!(
            error.to_string(),
            "remote tts audio format 'mp3' is not supported"
        );
    }

    #[test]
    fn decode_wav_samples_parses_pcm16_mono_audio() {
        let bytes = [
            b'R', b'I', b'F', b'F', 40, 0, 0, 0, b'W', b'A', b'V', b'E', b'f', b'm', b't', b' ',
            16, 0, 0, 0, 1, 0, 1, 0, 0x80, 0x3E, 0, 0, 0, 0x7D, 0, 0, 2, 0, 16, 0, b'd', b'a',
            b't', b'a', 4, 0, 0, 0, 0, 0, 0xFF, 0x7F,
        ];

        let decoded = decode_wav_samples(&bytes).expect("wav bytes should decode");
        assert_eq!(decoded.channels, 1);
        assert_eq!(decoded.sample_rate, 16_000);
        assert_eq!(decoded.samples.len(), 2);
        assert!((decoded.samples[0] - 0.0).abs() < 0.0001);
        assert!(decoded.samples[1] > 0.99);
    }

    #[test]
    fn normalized_model_path_rejects_empty_values() {
        let error = normalized_model_path("   ").expect_err("empty paths should fail");
        assert_eq!(error.to_string(), "local tts model path must not be empty");
    }

    #[test]
    fn resolved_voice_prefers_runtime_voice_over_profile_default_struct() {
        let runtime_audio = RuntimeAudioState {
            tts_voice: Some(String::from("Rosie")),
            ..RuntimeAudioState::default()
        };
        let profile = LocalTtsProfile {
            backend: String::from("kitten_tts_rs"),
            model_id: String::from("default"),
            model_path: String::from("/tmp/model"),
            default_voice: String::from("Bruno"),
            sample_rate: KITTEN_TTS_SAMPLE_RATE,
        };

        assert_eq!(
            resolved_voice(&runtime_audio, &profile.default_voice),
            "Rosie"
        );
    }
}
