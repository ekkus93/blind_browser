use std::path::{Path, PathBuf};

use crate::audio_io::RuntimeAudioState;
use crate::config::AppConfig;

#[cfg(feature = "local-tts")]
use kitten_tts::model::KittenTTS;

use super::{
    CachedLocalTtsModel, CachedSpeechKey, SynthesizedSpeech, TtsController, TtsProviderKind,
    TtsRuntimeError, KITTEN_TTS_CHANNELS, KITTEN_TTS_SAMPLE_RATE,
};

impl TtsController {
    pub(super) fn synthesize_local(
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
}

pub(super) fn resolved_voice(runtime_audio: &RuntimeAudioState, default_voice: &str) -> String {
    runtime_audio
        .tts_voice
        .as_deref()
        .map(str::trim)
        .filter(|voice| !voice.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| default_voice.to_string())
}

pub(super) fn normalized_model_path(model_path: &str) -> Result<PathBuf, TtsRuntimeError> {
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
