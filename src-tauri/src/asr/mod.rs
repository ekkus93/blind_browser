mod capture;
mod local;
mod processing;
mod remote;
mod wav;

#[cfg(feature = "audio")]
use std::thread;
#[cfg(feature = "audio")]
use std::time::Duration;

#[cfg(feature = "audio")]
use capture::CaptureSession;
use local::transcribe_local;
pub(crate) use processing::CapturedAudio;
#[cfg(test)]
use processing::{interleaved_to_mono, resample_linear};
#[cfg(test)]
use remote::normalized_optional_string;
use remote::transcribe_remote;
#[cfg(test)]
use wav::encode_wav_pcm16;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::{AppConfig, ProviderMode};

pub const DEFAULT_TRANSCRIBE_DURATION_MS: u64 = 3_000;
pub const MAX_TRANSCRIBE_DURATION_MS: u64 = 10_000;
pub const WHISPER_TARGET_SAMPLE_RATE: u32 = 16_000;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum AsrProviderKind {
    Local,
    Remote,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct AsrSettings {
    pub provider: AsrProviderKind,
    pub model: String,
}

impl Default for AsrSettings {
    fn default() -> Self {
        Self {
            provider: AsrProviderKind::Local,
            model: String::from("tiny"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AsrTranscription {
    pub transcript: Option<String>,
    pub confidence: Option<f32>,
    pub audio_duration_ms: Option<u64>,
    pub listening_active: bool,
}

#[derive(Debug, Error)]
pub enum AsrRuntimeError {
    #[error("asr local profile is not configured")]
    MissingLocalProfile,
    #[error("asr local profile '{profile_name}' was not found")]
    MissingLocalProfileDefinition { profile_name: String },
    #[error("asr remote profile is not configured")]
    MissingRemoteProfile,
    #[error("asr remote profile '{profile_name}' was not found")]
    MissingRemoteProfileDefinition { profile_name: String },
    #[error("asr remote profile '{profile_name}' uses unsupported provider '{provider}'")]
    UnsupportedRemoteProvider {
        profile_name: String,
        provider: String,
    },
    #[error("audio capture requires the 'audio' feature to be enabled")]
    AudioFeatureUnavailable,
    #[error("local asr requires the 'local-asr' feature to be enabled")]
    LocalAsrFeatureUnavailable,
    #[error("remote asr requires the 'remote-openai' feature to be enabled")]
    RemoteAsrFeatureUnavailable,
    #[error("could not find a default input audio device")]
    MissingInputDevice,
    #[error("failed to query the default input audio configuration: {reason}")]
    InputConfigUnavailable { reason: String },
    #[error("unsupported input sample format '{sample_format}'")]
    UnsupportedInputSampleFormat { sample_format: String },
    #[error("failed to build the microphone input stream: {reason}")]
    BuildInputStream { reason: String },
    #[error("failed to start the microphone input stream: {reason}")]
    StartInputStream { reason: String },
    #[error("failed to lock the microphone audio buffer")]
    AudioBufferLockFailed,
    #[error("local asr model path must not be empty")]
    EmptyLocalModelPath,
    #[error("local asr model path does not exist: {model_path}")]
    MissingLocalModelPath { model_path: String },
    #[error("failed to load the local asr model from {model_path}: {reason}")]
    LocalModelLoad { model_path: String, reason: String },
    #[error("captured audio buffer was empty")]
    NoAudioCaptured,
    #[error("remote asr secret could not be resolved: {reason}")]
    RemoteSecretUnavailable { reason: String },
    #[error("failed to encode captured audio for remote asr: {reason}")]
    RemoteAudioEncodeFailed { reason: String },
    #[error("failed to build the remote asr request: {reason}")]
    RemoteRequestBuildFailed { reason: String },
    #[error("remote asr request timed out after {timeout_ms}ms")]
    RemoteRequestTimedOut { timeout_ms: u64 },
    #[error("remote asr request failed: {reason}")]
    RemoteRequestFailed { reason: String },
    #[error("failed to transcribe captured audio: {reason}")]
    TranscriptionFailed { reason: String },
}

pub struct AsrController {
    #[cfg(feature = "audio")]
    active_capture: Option<CaptureSession>,
}

impl AsrController {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "audio")]
            active_capture: None,
        }
    }

    pub fn start_listening(&mut self) -> Result<bool, AsrRuntimeError> {
        #[cfg(not(feature = "audio"))]
        {
            Err(AsrRuntimeError::AudioFeatureUnavailable)
        }

        #[cfg(feature = "audio")]
        {
            if self.active_capture.is_some() {
                return Ok(false);
            }

            self.active_capture = Some(CaptureSession::start()?);
            Ok(true)
        }
    }

    pub fn stop_listening(&mut self) -> bool {
        #[cfg(not(feature = "audio"))]
        {
            false
        }

        #[cfg(feature = "audio")]
        {
            self.active_capture.take().is_some()
        }
    }

    pub fn is_listening(&self) -> bool {
        #[cfg(not(feature = "audio"))]
        {
            false
        }

        #[cfg(feature = "audio")]
        {
            self.active_capture.is_some()
        }
    }

    pub fn transcribe_command(
        &mut self,
        config: &AppConfig,
        capture_duration_ms: u64,
        auto_stop: bool,
    ) -> Result<AsrTranscription, AsrRuntimeError> {
        let captured_audio = self.capture_audio(capture_duration_ms, auto_stop)?;
        let audio_duration_ms = Some(captured_audio.duration_ms());
        let transcript = transcribe_captured_audio(config, &captured_audio)?;
        Ok(self.finalize_transcription(transcript, audio_duration_ms))
    }

    fn capture_audio(
        &mut self,
        capture_duration_ms: u64,
        auto_stop: bool,
    ) -> Result<CapturedAudio, AsrRuntimeError> {
        #[cfg(not(feature = "audio"))]
        {
            let _ = (capture_duration_ms, auto_stop);
            Err(AsrRuntimeError::AudioFeatureUnavailable)
        }

        #[cfg(feature = "audio")]
        {
            if let Some(active_capture) = self.active_capture.as_ref() {
                thread::sleep(Duration::from_millis(capture_duration_ms));
                let captured_audio = active_capture.take_captured_audio()?;
                if auto_stop {
                    self.active_capture.take();
                }
                return captured_audio.ensure_non_empty();
            }

            let temporary_capture = CaptureSession::start()?;
            thread::sleep(Duration::from_millis(capture_duration_ms));
            temporary_capture.take_captured_audio()?.ensure_non_empty()
        }
    }

    /// Phase 1 of a lock-released capture: ensure a capture session is recording.
    ///
    /// Returns whether a new session was started for this request (so the caller
    /// knows to stop it again after a one-shot transcription). The cpal stream
    /// keeps filling the shared buffer during the unlocked capture window, so the
    /// `AppCore` lock can be released between this call and [`Self::drain_capture`].
    pub fn begin_capture(&mut self) -> Result<bool, AsrRuntimeError> {
        #[cfg(not(feature = "audio"))]
        {
            Err(AsrRuntimeError::AudioFeatureUnavailable)
        }

        #[cfg(feature = "audio")]
        {
            if self.active_capture.is_some() {
                return Ok(false);
            }
            self.active_capture = Some(CaptureSession::start()?);
            Ok(true)
        }
    }

    /// Phase 2a of a lock-released capture: drain the captured audio and (for a
    /// one-shot / auto-stop request) stop the microphone, so the subsequent
    /// transcription can run with the `AppCore` lock released and the mic idle.
    ///
    /// Returns `Ok(None)` if `stop_listening` dropped the session during the
    /// unlocked capture window, so the caller can report a clean "capture stopped"
    /// result instead of transcribing a stale or partial buffer. The drained audio
    /// is then transcribed (unlocked) via [`transcribe_captured_audio`] and wrapped
    /// with [`Self::finalize_transcription`].
    pub(crate) fn drain_capture(
        &mut self,
        auto_stop: bool,
        started_for_this_request: bool,
    ) -> Result<Option<CapturedAudio>, AsrRuntimeError> {
        #[cfg(not(feature = "audio"))]
        {
            let _ = (auto_stop, started_for_this_request);
            Err(AsrRuntimeError::AudioFeatureUnavailable)
        }

        #[cfg(feature = "audio")]
        {
            let captured_audio = {
                let Some(active_capture) = self.active_capture.as_ref() else {
                    // stop_listening took the session during the capture window.
                    return Ok(None);
                };
                active_capture.take_captured_audio()?.ensure_non_empty()?
            };

            if auto_stop || started_for_this_request {
                self.active_capture.take();
            }

            Ok(Some(captured_audio))
        }
    }

    /// Phase 2b of a lock-released capture: wrap a completed transcript into an
    /// [`AsrTranscription`], normalizing the text and recording the current
    /// listening state. The transcription itself ([`transcribe_captured_audio`])
    /// runs between [`Self::drain_capture`] and this call, with the lock released.
    pub fn finalize_transcription(
        &self,
        transcript: String,
        audio_duration_ms: Option<u64>,
    ) -> AsrTranscription {
        AsrTranscription {
            transcript: normalize_transcript(&transcript),
            confidence: None,
            audio_duration_ms,
            listening_active: self.is_listening(),
        }
    }
}

/// Transcribe drained audio with the configured ASR backend. A free function with
/// no `AsrController` / `AppCore` state, so it can run with the `AppCore` lock
/// released across the (possibly remote, network-bound) transcription.
pub(crate) fn transcribe_captured_audio(
    config: &AppConfig,
    captured_audio: &CapturedAudio,
) -> Result<String, AsrRuntimeError> {
    match config.providers.asr.mode {
        ProviderMode::Local => transcribe_local(config, captured_audio),
        ProviderMode::Remote => transcribe_remote(config, captured_audio),
    }
}

impl Default for AsrController {
    fn default() -> Self {
        Self::new()
    }
}

fn normalize_transcript(transcript: &str) -> Option<String> {
    let trimmed = transcript.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        encode_wav_pcm16, interleaved_to_mono, normalize_transcript, normalized_optional_string,
        resample_linear,
    };

    #[test]
    fn interleaved_to_mono_averages_channels() {
        let mono = interleaved_to_mono(&[1.0, -1.0, 0.5, 0.5], 2);
        assert_eq!(mono, vec![0.0, 0.5]);
    }

    #[test]
    fn resample_linear_returns_original_when_rates_match() {
        let samples = vec![0.0, 0.25, 0.5];
        assert_eq!(resample_linear(&samples, 16_000, 16_000), samples);
    }

    #[test]
    fn normalize_transcript_drops_blank_strings() {
        assert_eq!(normalize_transcript("   "), None);
        assert_eq!(normalize_transcript("hello"), Some(String::from("hello")));
    }

    #[test]
    fn normalized_optional_string_trims_and_drops_empty_values() {
        assert_eq!(
            normalized_optional_string(Some("  en ")),
            Some(String::from("en"))
        );
        assert_eq!(normalized_optional_string(Some("   ")), None);
        assert_eq!(normalized_optional_string(None), None);
    }

    #[test]
    fn encode_wav_pcm16_emits_riff_wave_header() {
        let wav = encode_wav_pcm16(&[0.0, 0.5, -0.5], 16_000, 1).expect("wav should encode");

        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(&wav[36..40], b"data");
        assert_eq!(wav.len(), 44 + (3 * 2));
    }

    // When `stop_listening` drops the capture session during the unlocked capture
    // window, `drain_capture` finds no active session and must report a clean
    // "no audio" result (Ok(None)) instead of erroring or draining a stale buffer.
    // This path returns before touching any audio hardware, so it is exercisable
    // without a real input device.
    #[cfg(feature = "audio")]
    #[test]
    fn drain_capture_reports_none_when_session_stopped_mid_window() {
        use super::AsrController;

        let mut controller = AsrController::new();

        let result = controller
            .drain_capture(false, true)
            .expect("drain_capture should not error when the session was stopped");

        assert!(
            result.is_none(),
            "a session stopped mid-capture should yield no captured audio"
        );
    }
}
