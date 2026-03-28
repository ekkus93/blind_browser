use std::collections::VecDeque;
use std::path::{Path, PathBuf};
#[cfg(feature = "remote-openai")]
use std::time::Duration;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::audio_io::RuntimeAudioState;
use crate::config::{AppConfig, ProviderMode};

#[cfg(feature = "remote-openai")]
use crate::config::{resolve_secret_ref, RemoteTtsAudioFormat, RemoteTtsProfile};

#[cfg(feature = "local-tts")]
use kitten_tts::model::KittenTTS;

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
const SYNTHESIZED_SPEECH_CACHE_LIMIT: usize = 8;

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
    #[error("failed to decode the remote tts audio response: {reason}")]
    RemoteResponseDecodeFailed { reason: String },
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
    synthesized_speech_cache: VecDeque<CachedSynthesizedSpeech>,
}

impl TtsController {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "local-tts")]
            local_model: None,
            synthesized_speech_cache: VecDeque::new(),
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
        #[cfg(feature = "remote-openai")]
        {
            use crate::config::RemoteProviderKind;

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

        if profile.sample_rate != KITTEN_TTS_SAMPLE_RATE {
            return Err(TtsRuntimeError::UnsupportedLocalSampleRate {
                sample_rate: profile.sample_rate,
            });
        }

        let voice = resolved_voice(runtime_audio, &profile.default_voice);
        let model_dir = normalized_model_path(&profile.model_path)?;
        let cache_key = CachedSpeechKey {
            provider: TtsProviderKind::Local,
            model_identity: format!(
                "{}|{}|{}",
                profile.model_id,
                model_dir.display(),
                profile.sample_rate
            ),
            voice: voice.clone(),
            playback_speed_bits: runtime_audio.playback_speed.to_bits(),
            text: text.to_string(),
        };
        if let Some(cached) = self.cached_speech(&cache_key) {
            return Ok(cached);
        }
        let samples = self.generate_local_samples(&model_dir, text, &voice, runtime_audio)?;

        let speech = SynthesizedSpeech {
            provider: TtsProviderKind::Local,
            voice,
            sample_rate: profile.sample_rate,
            channels: KITTEN_TTS_CHANNELS,
            samples,
        };
        self.store_cached_speech(cache_key, speech.clone());
        Ok(speech)
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

    fn cached_speech(&mut self, key: &CachedSpeechKey) -> Option<SynthesizedSpeech> {
        let index = self
            .synthesized_speech_cache
            .iter()
            .position(|entry| &entry.key == key)?;
        let entry = self
            .synthesized_speech_cache
            .remove(index)
            .expect("cache entry should exist at located index");
        let speech = entry.speech.clone();
        self.synthesized_speech_cache.push_front(entry);
        Some(speech)
    }

    fn store_cached_speech(&mut self, key: CachedSpeechKey, speech: SynthesizedSpeech) {
        if let Some(index) = self
            .synthesized_speech_cache
            .iter()
            .position(|entry| entry.key == key)
        {
            self.synthesized_speech_cache.remove(index);
        }
        self.synthesized_speech_cache
            .push_front(CachedSynthesizedSpeech { key, speech });
        while self.synthesized_speech_cache.len() > SYNTHESIZED_SPEECH_CACHE_LIMIT {
            self.synthesized_speech_cache.pop_back();
        }
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

#[cfg(feature = "remote-openai")]
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

#[cfg(feature = "remote-openai")]
fn is_openai_builtin_voice(voice: &str) -> bool {
    let normalized = voice.trim().to_ascii_lowercase();
    OPENAI_TTS_VOICES
        .iter()
        .any(|candidate| *candidate == normalized)
}

#[cfg(feature = "remote-openai")]
fn openai_speech_response_format_value(audio_format: RemoteTtsAudioFormat) -> &'static str {
    match audio_format {
        RemoteTtsAudioFormat::Wav => "wav",
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct CachedSpeechKey {
    provider: TtsProviderKind,
    model_identity: String,
    voice: String,
    playback_speed_bits: u32,
    text: String,
}

#[derive(Debug, Clone, PartialEq)]
struct CachedSynthesizedSpeech {
    key: CachedSpeechKey,
    speech: SynthesizedSpeech,
}

#[cfg(test)]
mod tests {
    use std::fs;
    #[cfg(feature = "remote-openai")]
    use std::io::{Read, Write};
    #[cfg(feature = "remote-openai")]
    use std::net::TcpListener;
    use std::path::PathBuf;
    #[cfg(feature = "remote-openai")]
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        decode_wav_samples, normalized_model_path, resolved_voice, CachedSpeechKey,
        SynthesizedSpeech, TtsController, TtsProviderKind, KITTEN_TTS_CHANNELS,
        KITTEN_TTS_SAMPLE_RATE, SYNTHESIZED_SPEECH_CACHE_LIMIT,
    };
    use crate::audio_io::RuntimeAudioState;
    use crate::config::{AppConfig, LocalTtsBackend, LocalTtsProfile, ProviderMode};

    #[cfg(feature = "remote-openai")]
    use super::{openai_speech_response_format_value, resolved_remote_voice};
    #[cfg(feature = "remote-openai")]
    use crate::config::{RemoteProviderKind, RemoteTtsAudioFormat, RemoteTtsProfile, SecretRef};

    fn test_wav_bytes() -> Vec<u8> {
        vec![
            b'R', b'I', b'F', b'F', 40, 0, 0, 0, b'W', b'A', b'V', b'E', b'f', b'm', b't', b' ',
            16, 0, 0, 0, 1, 0, 1, 0, 0x80, 0x3E, 0, 0, 0, 0x7D, 0, 0, 2, 0, 16, 0, b'd', b'a',
            b't', b'a', 4, 0, 0, 0, 0, 0, 0xFF, 0x7F,
        ]
    }

    fn unique_test_path(label: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "blind-browser-{label}-{}-{timestamp}",
            std::process::id()
        ))
    }

    #[cfg(feature = "remote-openai")]
    fn spawn_remote_tts_test_server(response_body: Vec<u8>) -> (String, thread::JoinHandle<()>) {
        let listener =
            TcpListener::bind("127.0.0.1:0").expect("test server should bind an ephemeral port");
        let address = listener
            .local_addr()
            .expect("test server should expose its bound address");
        let base_url = format!("http://{address}/v1");

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener
                .accept()
                .expect("test server should accept one request");

            let mut request = Vec::new();
            let mut header_end = None;
            loop {
                let mut buffer = [0_u8; 1024];
                let bytes_read = stream
                    .read(&mut buffer)
                    .expect("test server should read request bytes");
                if bytes_read == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..bytes_read]);
                if let Some(position) = request.windows(4).position(|window| window == b"\r\n\r\n")
                {
                    header_end = Some(position + 4);
                    break;
                }
            }

            let header_end = header_end.expect("request should include headers");
            let headers = String::from_utf8_lossy(&request[..header_end]).into_owned();
            assert!(
                headers.starts_with("POST /v1/audio/speech HTTP/1.1\r\n"),
                "unexpected request line: {headers}"
            );
            assert!(
                headers.contains("authorization: Bearer blind-browser-test-key")
                    || headers.contains("Authorization: Bearer blind-browser-test-key"),
                "expected bearer auth header in request: {headers}"
            );

            let content_length = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.trim()
                        .eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())
                        .flatten()
                })
                .expect("request should include a content-length header");

            while request.len() < header_end + content_length {
                let mut buffer = [0_u8; 1024];
                let bytes_read = stream
                    .read(&mut buffer)
                    .expect("test server should read request body bytes");
                if bytes_read == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..bytes_read]);
            }

            let body = String::from_utf8_lossy(&request[header_end..]).into_owned();
            assert!(
                body.contains("\"input\":\"Hello remote world\""),
                "unexpected body: {body}"
            );
            assert!(
                body.contains("\"voice\":\"alloy\""),
                "unexpected body: {body}"
            );

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: audio/wav\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                response_body.len()
            );
            stream
                .write_all(response.as_bytes())
                .expect("test server should write response headers");
            stream
                .write_all(&response_body)
                .expect("test server should write response body");
            stream.flush().expect("test server should flush response");
        });

        (base_url, handle)
    }

    #[test]
    fn resolved_voice_prefers_runtime_voice_over_profile_default() {
        let runtime_audio = RuntimeAudioState {
            tts_voice: Some(String::from("Rosie")),
            ..RuntimeAudioState::default()
        };

        assert_eq!(resolved_voice(&runtime_audio, "Bruno"), "Rosie");
    }

    #[cfg(feature = "remote-openai")]
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
            api_key: SecretRef::FromEnv {
                from_env: String::from("OPENAI_API_KEY"),
            },
            organization: None,
            project: None,
            voice: String::from("alloy"),
            audio_format: RemoteTtsAudioFormat::Wav,
            timeout_ms: 30_000,
        };

        assert_eq!(
            resolved_remote_voice(&runtime_audio, &profile).expect("voice resolution should work"),
            "alloy"
        );
    }

    #[cfg(feature = "remote-openai")]
    #[test]
    fn openai_speech_response_format_value_returns_wav() {
        assert_eq!(
            openai_speech_response_format_value(RemoteTtsAudioFormat::Wav),
            "wav"
        );
    }

    #[test]
    fn decode_wav_samples_parses_pcm16_mono_audio() {
        let decoded = decode_wav_samples(&test_wav_bytes()).expect("wav bytes should decode");
        assert_eq!(decoded.channels, 1);
        assert_eq!(decoded.sample_rate, 16_000);
        assert_eq!(decoded.samples.len(), 2);
        assert!((decoded.samples[0] - 0.0).abs() < 0.0001);
        assert!(decoded.samples[1] > 0.99);
    }

    #[cfg(feature = "remote-openai")]
    #[test]
    fn synthesize_narration_returns_remote_audio_when_remote_tts_is_selected() {
        let secret_path = unique_test_path("remote-tts-secret");
        fs::write(&secret_path, "blind-browser-test-key\n")
            .expect("test should write a temporary secret file");

        let (base_url, server) = spawn_remote_tts_test_server(test_wav_bytes());

        let mut config = AppConfig::default();
        config.providers.tts.mode = ProviderMode::Remote;
        config.providers.tts.remote_profile = Some(String::from("openai-tts-default"));
        let profile = config
            .remote_tts_profiles
            .get_mut("openai-tts-default")
            .expect("default config should include the remote tts profile");
        profile.base_url = base_url;
        profile.api_key = SecretRef::FromFile {
            from_file: secret_path.display().to_string(),
        };
        profile.voice = String::from("alloy");

        let runtime_audio = RuntimeAudioState::default();
        let mut controller = TtsController::new();

        let speech = controller
            .synthesize_narration(&config, &runtime_audio, "Hello remote world")
            .expect("remote synthesis should succeed");

        assert_eq!(speech.provider, TtsProviderKind::Remote);
        assert_eq!(speech.voice, "alloy");
        assert_eq!(speech.sample_rate, 16_000);
        assert_eq!(speech.channels, 1);
        assert_eq!(speech.samples.len(), 2);
        assert!((speech.samples[0] - 0.0).abs() < 0.0001);
        assert!(speech.samples[1] > 0.99);

        server.join().expect("test server should exit cleanly");
        fs::remove_file(secret_path).expect("test should clean up the temporary secret file");
    }

    #[test]
    fn normalized_model_path_rejects_empty_values() {
        let error = normalized_model_path("   ").expect_err("empty paths should fail");
        assert_eq!(error.to_string(), "local tts model path must not be empty");
    }

    #[test]
    fn normalized_model_path_rejects_missing_path() {
        let error = normalized_model_path("/tmp/definitely-does-not-exist-blind-browser-tts")
            .expect_err("missing path should fail");
        assert!(
            error.to_string().contains("does not exist"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn resolved_voice_uses_default_when_runtime_voice_is_empty() {
        let runtime_audio = RuntimeAudioState {
            tts_voice: Some(String::from("   ")),
            ..RuntimeAudioState::default()
        };

        assert_eq!(resolved_voice(&runtime_audio, "Bruno"), "Bruno");
    }

    #[test]
    fn resolved_voice_uses_default_when_runtime_voice_is_none() {
        let runtime_audio = RuntimeAudioState {
            tts_voice: None,
            ..RuntimeAudioState::default()
        };

        assert_eq!(resolved_voice(&runtime_audio, "Luna"), "Luna");
    }

    #[test]
    fn resolved_voice_prefers_runtime_voice_over_profile_default_struct() {
        let runtime_audio = RuntimeAudioState {
            tts_voice: Some(String::from("Rosie")),
            ..RuntimeAudioState::default()
        };
        let profile = LocalTtsProfile {
            backend: LocalTtsBackend::KittenTtsRs,
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

    #[test]
    fn synthesized_speech_cache_returns_cached_entry() {
        let mut controller = TtsController::new();
        let key = CachedSpeechKey {
            provider: TtsProviderKind::Local,
            model_identity: String::from("local|model"),
            voice: String::from("Bruno"),
            playback_speed_bits: 1.0f32.to_bits(),
            text: String::from("Hello world"),
        };
        let speech = SynthesizedSpeech {
            provider: TtsProviderKind::Local,
            voice: String::from("Bruno"),
            sample_rate: KITTEN_TTS_SAMPLE_RATE,
            channels: KITTEN_TTS_CHANNELS,
            samples: vec![0.1, 0.2],
        };

        controller.store_cached_speech(key.clone(), speech.clone());

        assert_eq!(controller.cached_speech(&key), Some(speech));
    }

    #[test]
    fn synthesized_speech_cache_evicts_oldest_entries() {
        let mut controller = TtsController::new();

        for index in 0..=SYNTHESIZED_SPEECH_CACHE_LIMIT {
            controller.store_cached_speech(
                CachedSpeechKey {
                    provider: TtsProviderKind::Local,
                    model_identity: format!("local|model-{index}"),
                    voice: String::from("Bruno"),
                    playback_speed_bits: 1.0f32.to_bits(),
                    text: format!("text-{index}"),
                },
                SynthesizedSpeech {
                    provider: TtsProviderKind::Local,
                    voice: String::from("Bruno"),
                    sample_rate: KITTEN_TTS_SAMPLE_RATE,
                    channels: KITTEN_TTS_CHANNELS,
                    samples: vec![index as f32],
                },
            );
        }

        assert_eq!(
            controller.synthesized_speech_cache.len(),
            SYNTHESIZED_SPEECH_CACHE_LIMIT
        );
        let oldest_key = CachedSpeechKey {
            provider: TtsProviderKind::Local,
            model_identity: String::from("local|model-0"),
            voice: String::from("Bruno"),
            playback_speed_bits: 1.0f32.to_bits(),
            text: String::from("text-0"),
        };
        assert_eq!(controller.cached_speech(&oldest_key), None);
    }
}
