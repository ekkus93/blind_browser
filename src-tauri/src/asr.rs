use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::{AppConfig, LocalAsrProfile, ProviderMode};

#[cfg(feature = "audio")]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
#[cfg(feature = "audio")]
use cpal::{FromSample, Sample, SampleFormat, SizedSample, Stream, SupportedStreamConfig};

#[cfg(feature = "local-asr")]
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub const DEFAULT_TRANSCRIBE_DURATION_MS: u64 = 3_000;
pub const MAX_TRANSCRIBE_DURATION_MS: u64 = 10_000;
pub const WHISPER_TARGET_SAMPLE_RATE: u32 = 16_000;
pub const WHISPER_BACKEND: &str = "whisper_rs";

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
    #[error("remote asr profile '{profile_name}' is not implemented yet")]
    RemoteProviderUnimplemented { profile_name: String },
    #[error("unsupported local asr backend '{backend}'")]
    UnsupportedLocalBackend { backend: String },
    #[error("audio capture requires the 'audio' feature to be enabled")]
    AudioFeatureUnavailable,
    #[error("local asr requires the 'local-asr' feature to be enabled")]
    LocalAsrFeatureUnavailable,
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

        let transcript = match config.providers.asr.mode {
            ProviderMode::Local => self.transcribe_local(config, &captured_audio)?,
            ProviderMode::Remote => {
                let profile_name = config
                    .providers
                    .asr
                    .remote_profile
                    .clone()
                    .ok_or(AsrRuntimeError::MissingRemoteProfile)?;
                return Err(AsrRuntimeError::RemoteProviderUnimplemented { profile_name });
            }
        };

        Ok(AsrTranscription {
            transcript: normalize_transcript(&transcript),
            confidence: None,
            audio_duration_ms,
            listening_active: self.is_listening(),
        })
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
                let captured_audio = active_capture.snapshot()?;
                if auto_stop {
                    self.active_capture.take();
                }
                return captured_audio.ensure_non_empty();
            }

            let temporary_capture = CaptureSession::start()?;
            thread::sleep(Duration::from_millis(capture_duration_ms));
            temporary_capture.snapshot()?.ensure_non_empty()
        }
    }

    fn transcribe_local(
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

        if profile.backend != WHISPER_BACKEND {
            return Err(AsrRuntimeError::UnsupportedLocalBackend {
                backend: profile.backend.clone(),
            });
        }

        let model_path = normalized_model_path(&profile.model_path)?;
        let audio = captured_audio.to_whisper_audio();
        transcribe_with_whisper(&model_path, profile, &audio)
    }
}

impl Default for AsrController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "audio")]
type SharedCaptureBuffer = Arc<Mutex<Vec<f32>>>;

#[cfg(feature = "audio")]
struct CaptureSession {
    buffer: SharedCaptureBuffer,
    sample_rate: u32,
    channels: u16,
    _stream: Stream,
}

#[cfg(feature = "audio")]
impl CaptureSession {
    fn start() -> Result<Self, AsrRuntimeError> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(AsrRuntimeError::MissingInputDevice)?;
        let config = device.default_input_config().map_err(|error| {
            AsrRuntimeError::InputConfigUnavailable {
                reason: error.to_string(),
            }
        })?;
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let stream = build_input_stream(&device, &config, Arc::clone(&buffer))?;
        stream
            .play()
            .map_err(|error| AsrRuntimeError::StartInputStream {
                reason: error.to_string(),
            })?;

        Ok(Self {
            buffer,
            sample_rate: config.sample_rate(),
            channels: config.channels(),
            _stream: stream,
        })
    }

    fn snapshot(&self) -> Result<CapturedAudio, AsrRuntimeError> {
        let samples = self
            .buffer
            .lock()
            .map_err(|_| AsrRuntimeError::AudioBufferLockFailed)?
            .clone();
        Ok(CapturedAudio {
            samples,
            sample_rate: self.sample_rate,
            channels: self.channels,
        })
    }
}

#[cfg(feature = "audio")]
fn build_input_stream(
    device: &cpal::Device,
    config: &SupportedStreamConfig,
    buffer: SharedCaptureBuffer,
) -> Result<Stream, AsrRuntimeError> {
    let err_fn = |error| tracing::warn!("audio input stream error: {error}");

    let stream_config = config.config();
    match config.sample_format() {
        SampleFormat::I8 => build_typed_input_stream::<i8>(device, &stream_config, buffer, err_fn),
        SampleFormat::I16 => {
            build_typed_input_stream::<i16>(device, &stream_config, buffer, err_fn)
        }
        SampleFormat::I32 => {
            build_typed_input_stream::<i32>(device, &stream_config, buffer, err_fn)
        }
        SampleFormat::I64 => {
            build_typed_input_stream::<i64>(device, &stream_config, buffer, err_fn)
        }
        SampleFormat::U8 => build_typed_input_stream::<u8>(device, &stream_config, buffer, err_fn),
        SampleFormat::U16 => {
            build_typed_input_stream::<u16>(device, &stream_config, buffer, err_fn)
        }
        SampleFormat::U32 => {
            build_typed_input_stream::<u32>(device, &stream_config, buffer, err_fn)
        }
        SampleFormat::U64 => {
            build_typed_input_stream::<u64>(device, &stream_config, buffer, err_fn)
        }
        SampleFormat::F32 => {
            build_typed_input_stream::<f32>(device, &stream_config, buffer, err_fn)
        }
        SampleFormat::F64 => {
            build_typed_input_stream::<f64>(device, &stream_config, buffer, err_fn)
        }
        other => Err(AsrRuntimeError::UnsupportedInputSampleFormat {
            sample_format: format!("{other:?}"),
        }),
    }
}

#[cfg(feature = "audio")]
fn build_typed_input_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    buffer: SharedCaptureBuffer,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<Stream, AsrRuntimeError>
where
    T: Sample + SizedSample,
    f32: FromSample<T>,
{
    device
        .build_input_stream(
            config,
            move |data: &[T], _| capture_input_data::<T>(data, &buffer),
            err_fn,
            None,
        )
        .map_err(|error| AsrRuntimeError::BuildInputStream {
            reason: error.to_string(),
        })
}

#[cfg(feature = "audio")]
fn capture_input_data<T>(input: &[T], buffer: &SharedCaptureBuffer)
where
    T: Sample,
    f32: FromSample<T>,
{
    if let Ok(mut guard) = buffer.lock() {
        guard.extend(input.iter().copied().map(f32::from_sample));
    }
}

#[derive(Debug, Clone, PartialEq)]
struct CapturedAudio {
    samples: Vec<f32>,
    sample_rate: u32,
    channels: u16,
}

impl CapturedAudio {
    fn ensure_non_empty(self) -> Result<Self, AsrRuntimeError> {
        if self.samples.is_empty() {
            Err(AsrRuntimeError::NoAudioCaptured)
        } else {
            Ok(self)
        }
    }

    fn duration_ms(&self) -> u64 {
        if self.sample_rate == 0 || self.channels == 0 {
            return 0;
        }

        let frame_count = self.samples.len() / usize::from(self.channels);
        ((frame_count as f64 / f64::from(self.sample_rate)) * 1000.0).round() as u64
    }

    fn to_whisper_audio(&self) -> Vec<f32> {
        let mono = interleaved_to_mono(&self.samples, self.channels);
        resample_linear(&mono, self.sample_rate, WHISPER_TARGET_SAMPLE_RATE)
    }
}

fn interleaved_to_mono(samples: &[f32], channels: u16) -> Vec<f32> {
    if channels <= 1 {
        return samples.to_vec();
    }

    let channel_count = usize::from(channels);
    let mut mono = Vec::with_capacity(samples.len() / channel_count);
    for frame in samples.chunks_exact(channel_count) {
        let sum: f32 = frame.iter().copied().sum();
        mono.push(sum / channels as f32);
    }
    mono
}

fn resample_linear(samples: &[f32], input_sample_rate: u32, output_sample_rate: u32) -> Vec<f32> {
    if samples.is_empty() || input_sample_rate == output_sample_rate || input_sample_rate == 0 {
        return samples.to_vec();
    }

    let ratio = output_sample_rate as f64 / input_sample_rate as f64;
    let output_len = ((samples.len() as f64) * ratio).round().max(1.0) as usize;
    let mut output = Vec::with_capacity(output_len);

    for output_index in 0..output_len {
        let source_position = output_index as f64 / ratio;
        let lower_index = source_position.floor() as usize;
        let upper_index = lower_index
            .saturating_add(1)
            .min(samples.len().saturating_sub(1));
        let fraction = (source_position - lower_index as f64) as f32;
        let lower = samples[lower_index];
        let upper = samples[upper_index];
        output.push(lower + ((upper - lower) * fraction));
    }

    output
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
    use super::{interleaved_to_mono, normalize_transcript, resample_linear};

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
}
