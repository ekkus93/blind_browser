use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::audio_io::RuntimeAudioState;
use crate::config::{AppConfig, LocalTtsProfile, ProviderMode};

#[cfg(feature = "local-tts")]
use kitten_tts::model::KittenTTS;

pub const KITTEN_TTS_BACKEND: &str = "kitten_tts_rs";
pub const KITTEN_TTS_SAMPLE_RATE: u32 = 24_000;
pub const KITTEN_TTS_CHANNELS: u16 = 1;

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
    #[error("remote tts profile '{profile_name}' is not implemented yet")]
    RemoteProviderUnimplemented { profile_name: String },
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
    LocalModelLoad {
        model_path: String,
        reason: String,
    },
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
            ProviderMode::Remote => {
                let profile_name = config
                    .providers
                    .tts
                    .remote_profile
                    .clone()
                    .ok_or(TtsRuntimeError::MissingRemoteProfile)?;
                Err(TtsRuntimeError::RemoteProviderUnimplemented { profile_name })
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

        let voice = resolved_voice(runtime_audio, profile);
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

fn resolved_voice(runtime_audio: &RuntimeAudioState, profile: &LocalTtsProfile) -> String {
    runtime_audio
        .tts_voice
        .as_deref()
        .map(str::trim)
        .filter(|voice| !voice.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| profile.default_voice.clone())
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

#[cfg(feature = "local-tts")]
struct CachedLocalTtsModel {
    model_dir: PathBuf,
    model: KittenTTS,
}

#[cfg(test)]
mod tests {
    use super::{normalized_model_path, resolved_voice, KITTEN_TTS_SAMPLE_RATE};
    use crate::audio_io::RuntimeAudioState;
    use crate::config::LocalTtsProfile;

    #[test]
    fn resolved_voice_prefers_runtime_voice_over_profile_default() {
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

        assert_eq!(resolved_voice(&runtime_audio, &profile), "Rosie");
    }

    #[test]
    fn normalized_model_path_rejects_empty_values() {
        let error = normalized_model_path("   ").expect_err("empty paths should fail");
        assert_eq!(error.to_string(), "local tts model path must not be empty");
    }
}
