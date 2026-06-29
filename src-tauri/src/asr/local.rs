use std::path::Path;

use crate::config::{AppConfig, LocalAsrProfile};

#[cfg(feature = "local-asr")]
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use super::processing::CapturedAudio;
use super::{AsrController, AsrRuntimeError};

impl AsrController {
    pub(super) fn transcribe_local(
        &self,
        config: &AppConfig,
        captured_audio: &CapturedAudio,
    ) -> Result<String, AsrRuntimeError> {
        let profile_name = config
            .providers
            .asr
            .local_profile
            .as_ref()
            .ok_or(AsrRuntimeError::MissingLocalProfile)?;
        let profile = config.local_asr_profiles.get(profile_name).ok_or_else(|| {
            AsrRuntimeError::MissingLocalProfileDefinition {
                profile_name: profile_name.clone(),
            }
        })?;

        let model_path = normalized_model_path(&profile.model_path)?;
        let audio = captured_audio.to_whisper_audio();
        transcribe_with_whisper(&model_path, profile, &audio)
    }
}

fn normalized_model_path(model_path: &str) -> Result<String, AsrRuntimeError> {
    let trimmed = model_path.trim();
    if trimmed.is_empty() {
        return Err(AsrRuntimeError::EmptyLocalModelPath);
    }
    if !Path::new(trimmed).exists() {
        return Err(AsrRuntimeError::MissingLocalModelPath {
            model_path: trimmed.to_string(),
        });
    }
    Ok(trimmed.to_string())
}

#[cfg(feature = "local-asr")]
fn transcribe_with_whisper(
    model_path: &str,
    profile: &LocalAsrProfile,
    audio: &[f32],
) -> Result<String, AsrRuntimeError> {
    if audio.is_empty() {
        return Err(AsrRuntimeError::NoAudioCaptured);
    }

    let context = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
        .map_err(|error| AsrRuntimeError::LocalModelLoad {
            model_path: model_path.to_string(),
            reason: error.to_string(),
        })?;
    let mut state =
        context
            .create_state()
            .map_err(|error| AsrRuntimeError::TranscriptionFailed {
                reason: error.to_string(),
            })?;
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 0 });
    params.set_n_threads(i32::from(profile.threads.max(1)));
    params.set_translate(false);
    params.set_language(profile.language.as_deref());
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    state
        .full(params, audio)
        .map_err(|error| AsrRuntimeError::TranscriptionFailed {
            reason: error.to_string(),
        })?;

    let transcript = state
        .as_iter()
        .filter_map(|segment| segment.to_str_lossy().ok())
        .map(|segment| segment.trim().to_string())
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    Ok(transcript)
}

#[cfg(not(feature = "local-asr"))]
fn transcribe_with_whisper(
    _model_path: &str,
    _profile: &LocalAsrProfile,
    _audio: &[f32],
) -> Result<String, AsrRuntimeError> {
    Err(AsrRuntimeError::LocalAsrFeatureUnavailable)
}
