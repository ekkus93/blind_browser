use std::path::Path;

use crate::config::{AppConfig, LocalAsrProfile};

#[cfg(feature = "local-asr")]
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use super::processing::CapturedAudio;
use super::AsrRuntimeError;

// Free function (no `AsrController` state): pure over `(config, captured_audio)`, so
// it can run with the `AppCore` lock released. See [`super::transcribe_captured_audio`].
pub(super) fn transcribe_local(
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

    collect_transcript_segments(state.as_iter().map(|segment| segment.to_str_lossy()))
}

/// Join whisper transcript segments, failing the whole transcription if any segment
/// fails to decode rather than silently dropping it (a dropped segment can turn a
/// spoken command into a partial command). Empty/whitespace-only segments are
/// skipped. Generic over the segment string/error types so it is testable without a
/// real whisper model.
#[cfg(any(feature = "local-asr", test))]
fn collect_transcript_segments<S, E>(
    segments: impl Iterator<Item = Result<S, E>>,
) -> Result<String, AsrRuntimeError>
where
    S: AsRef<str>,
    E: std::fmt::Display,
{
    let mut collected = Vec::new();
    for segment in segments {
        let text = segment.map_err(|error| AsrRuntimeError::TranscriptionFailed {
            reason: format!("failed to decode whisper transcript segment: {error}"),
        })?;
        let trimmed = text.as_ref().trim();
        if !trimmed.is_empty() {
            collected.push(trimmed.to_string());
        }
    }
    Ok(collected.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_transcript_segments_joins_and_trims_ok_segments() {
        let segments = vec![
            Ok::<&str, String>("  open  "),
            Ok("the"),
            Ok("   "),
            Ok("page  "),
        ];
        let transcript = collect_transcript_segments(segments.into_iter()).unwrap();
        assert_eq!(transcript, "open the page");
    }

    #[test]
    fn collect_transcript_segments_fails_on_a_decode_error() {
        let segments = vec![
            Ok::<&str, String>("open"),
            Err(String::from("bad utf-8")),
            Ok("page"),
        ];
        let error = collect_transcript_segments(segments.into_iter())
            .expect_err("a segment decode failure must fail the whole transcription");
        match error {
            AsrRuntimeError::TranscriptionFailed { reason } => {
                assert!(reason.contains("bad utf-8"), "unexpected reason: {reason}");
            }
            other => panic!("expected TranscriptionFailed, got {other:?}"),
        }
    }
}

#[cfg(not(feature = "local-asr"))]
fn transcribe_with_whisper(
    _model_path: &str,
    _profile: &LocalAsrProfile,
    _audio: &[f32],
) -> Result<String, AsrRuntimeError> {
    Err(AsrRuntimeError::LocalAsrFeatureUnavailable)
}
