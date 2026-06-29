use std::collections::VecDeque;
use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "local-tts")]
use kitten_tts::model::KittenTTS;

use crate::audio_io::RuntimeAudioState;
use crate::config::{AppConfig, ProviderMode};

mod local;
mod remote;
mod wav;

#[cfg(test)]
use local::{normalized_model_path, resolved_voice};
#[cfg(all(feature = "remote-openai", test))]
use remote::{openai_speech_response_format_value, resolved_remote_voice};
#[cfg(test)]
use wav::decode_wav_samples;

pub const KITTEN_TTS_SAMPLE_RATE: u32 = 24_000;
pub const KITTEN_TTS_CHANNELS: u16 = 1;
pub const KITTEN_TTS_VOICES: &[&str] = &[
    "Bella", "Jasper", "Luna", "Bruno", "Rosie", "Hugo", "Kiki", "Leo",
];
pub const OPENAI_TTS_VOICES: &[&str] = &[
    "alloy", "ash", "ballad", "coral", "echo", "fable", "onyx", "nova", "sage", "shimmer", "verse",
    "marin", "cedar",
];
#[cfg(any(feature = "remote-openai", test))]
const OPENAI_REMOTE_TTS_MIN_SPEED: f32 = 0.25;
#[cfg(any(feature = "remote-openai", test))]
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
