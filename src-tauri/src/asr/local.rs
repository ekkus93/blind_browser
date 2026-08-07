use std::path::Path;
#[cfg(feature = "local-asr")]
use std::sync::{Mutex, OnceLock};

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

/// Whether the cached whisper context (if any) must be rebuilt for
/// `requested_path`. Factored out of `transcribe_with_whisper` so the reload
/// decision is unit-testable without a real ggml model file, the same reason
/// `collect_transcript_segments` above is generic over its segment type.
#[cfg(any(feature = "local-asr", test))]
fn whisper_cache_needs_reload(cached_path: Option<&str>, requested_path: &str) -> bool {
    cached_path.is_none_or(|cached| cached != requested_path)
}

// CR3 P2.7: `WhisperContext` construction loads the whole ggml model file
// into memory (78 MB for the tiny model, up to 3.09 GB for large-v3), so
// rebuilding it on every utterance -- as this used to do -- meant every
// spoken command paid a full model load before transcription even started.
// This process-level cache mirrors `tts::local::TtsController::local_model`'s
// reload-when-the-path-changes pattern, but as a free-standing `OnceLock`
// rather than a field on a controller struct: the CR2 lock-scoping refactor
// deliberately made `transcribe_local` a free function that holds no
// controller state so it can run with the `AppCore` lock released (see its
// doc comment above), and a cache field would reintroduce exactly that
// coupling. The `Mutex` here is local-asr-only and distinct from the
// `AppCore` lock, held only for the duration of one transcription, so it
// does not reintroduce the lock-held-across-model-load problem P1.2 fixed.
#[cfg(feature = "local-asr")]
struct CachedWhisperContext {
    model_path: String,
    context: WhisperContext,
}

#[cfg(feature = "local-asr")]
fn whisper_context_cache() -> &'static Mutex<Option<CachedWhisperContext>> {
    static CACHE: OnceLock<Mutex<Option<CachedWhisperContext>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(None))
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

    let mut cache = whisper_context_cache()
        .lock()
        .map_err(|_| AsrRuntimeError::WhisperContextCacheLockFailed)?;
    if whisper_cache_needs_reload(
        cache.as_ref().map(|cached| cached.model_path.as_str()),
        model_path,
    ) {
        let context =
            WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
                .map_err(|error| AsrRuntimeError::LocalModelLoad {
                    model_path: model_path.to_string(),
                    reason: error.to_string(),
                })?;
        *cache = Some(CachedWhisperContext {
            model_path: model_path.to_string(),
            context,
        });
    }
    let context = &cache
        .as_ref()
        .expect("whisper context should be present after load")
        .context;

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

    // CR3 P2.7: pin the whisper-context cache's reload decision -- no cached
    // entry, or a cached entry for a different model path, must reload; a
    // cached entry for the same path must not.
    #[test]
    fn whisper_cache_needs_reload_on_empty_cache() {
        assert!(whisper_cache_needs_reload(None, "/models/ggml-tiny.bin"));
    }

    #[test]
    fn whisper_cache_needs_reload_on_model_path_change() {
        assert!(whisper_cache_needs_reload(
            Some("/models/ggml-tiny.bin"),
            "/models/ggml-large-v3.bin"
        ));
    }

    #[test]
    fn whisper_cache_reuses_context_for_the_same_model_path() {
        assert!(!whisper_cache_needs_reload(
            Some("/models/ggml-tiny.bin"),
            "/models/ggml-tiny.bin"
        ));
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
