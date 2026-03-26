use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[cfg(feature = "remote-openai")]
use std::sync::mpsc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::{
    resolve_secret_ref, AppConfig, LocalAsrProfile, ProviderMode, RemoteAsrProfile,
    RemoteProviderKind,
};

#[cfg(feature = "audio")]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
#[cfg(feature = "audio")]
use cpal::{FromSample, Sample, SampleFormat, SizedSample, Stream, SupportedStreamConfig};

#[cfg(feature = "local-asr")]
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

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

        let transcript = match config.providers.asr.mode {
            ProviderMode::Local => self.transcribe_local(config, &captured_audio)?,
            ProviderMode::Remote => self.transcribe_remote(config, &captured_audio)?,
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

        let model_path = normalized_model_path(&profile.model_path)?;
        let audio = captured_audio.to_whisper_audio();
        transcribe_with_whisper(&model_path, profile, &audio)
    }

    fn transcribe_remote(
        &self,
        config: &AppConfig,
        captured_audio: &CapturedAudio,
    ) -> Result<String, AsrRuntimeError> {
        let profile_name = config
            .providers
            .asr
            .remote_profile
            .as_ref()
            .ok_or(AsrRuntimeError::MissingRemoteProfile)?;
        let profile = config
            .remote_asr_profiles
            .get(profile_name)
            .ok_or_else(|| AsrRuntimeError::MissingRemoteProfileDefinition {
                profile_name: profile_name.clone(),
            })?;

        match &profile.provider {
            RemoteProviderKind::OpenAi => {
                self.transcribe_with_openai_remote(profile, captured_audio)
            }
            other => Err(AsrRuntimeError::UnsupportedRemoteProvider {
                profile_name: profile_name.clone(),
                provider: format!("{other:?}"),
            }),
        }
    }

    #[cfg(feature = "remote-openai")]
    fn transcribe_with_openai_remote(
        &self,
        profile: &RemoteAsrProfile,
        captured_audio: &CapturedAudio,
    ) -> Result<String, AsrRuntimeError> {
        use async_openai::{config::OpenAIConfig, Client};

        let api_key = resolve_secret_ref(&profile.api_key)
            .map_err(|reason| AsrRuntimeError::RemoteSecretUnavailable { reason })?;
        let audio_bytes = captured_audio.to_remote_wav_bytes()?;

        let mut openai_config = OpenAIConfig::new()
            .with_api_base(profile.base_url.clone())
            .with_api_key(api_key);
        if let Some(organization) = profile.organization.as_ref() {
            openai_config = openai_config.with_org_id(
                resolve_secret_ref(organization)
                    .map_err(|reason| AsrRuntimeError::RemoteSecretUnavailable { reason })?,
            );
        }
        if let Some(project) = profile.project.as_ref() {
            openai_config = openai_config.with_project_id(project.clone());
        }

        let request = build_openai_transcription_request(profile, audio_bytes)?;
        let timeout_ms = profile.timeout_ms.max(1);
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let client = Client::with_config(openai_config);
            let result =
                futures::executor::block_on(client.audio().transcription().create(request))
                    .map(|response| response.text)
                    .map_err(|error| AsrRuntimeError::RemoteRequestFailed {
                        reason: error.to_string(),
                    });
            let _ = sender.send(result);
        });

        match receiver.recv_timeout(Duration::from_millis(timeout_ms)) {
            Ok(result) => result,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                Err(AsrRuntimeError::RemoteRequestTimedOut { timeout_ms })
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                Err(AsrRuntimeError::RemoteRequestFailed {
                    reason: String::from("remote transcription task ended without a response"),
                })
            }
        }
    }

    #[cfg(not(feature = "remote-openai"))]
    fn transcribe_with_openai_remote(
        &self,
        _profile: &RemoteAsrProfile,
        _captured_audio: &CapturedAudio,
    ) -> Result<String, AsrRuntimeError> {
        Err(AsrRuntimeError::RemoteAsrFeatureUnavailable)
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

    fn to_remote_wav_bytes(&self) -> Result<Vec<u8>, AsrRuntimeError> {
        let mono = interleaved_to_mono(&self.samples, self.channels);
        let audio = resample_linear(&mono, self.sample_rate, WHISPER_TARGET_SAMPLE_RATE);
        encode_wav_pcm16(&audio, WHISPER_TARGET_SAMPLE_RATE, 1)
            .map_err(|reason| AsrRuntimeError::RemoteAudioEncodeFailed { reason })
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

#[cfg(feature = "remote-openai")]
fn build_openai_transcription_request(
    profile: &RemoteAsrProfile,
    audio_bytes: Vec<u8>,
) -> Result<async_openai::types::audio::CreateTranscriptionRequest, AsrRuntimeError> {
    use async_openai::types::audio::{
        AudioInput, AudioResponseFormat, CreateTranscriptionRequestArgs,
    };

    let mut request = CreateTranscriptionRequestArgs::default();
    request.file(AudioInput::from_vec_u8(
        String::from("command.wav"),
        audio_bytes,
    ));
    request.model(profile.model.clone());
    request.response_format(AudioResponseFormat::Json);
    request.temperature((profile.temperature_milli as f32) / 1000.0);
    if let Some(language) = normalized_optional_string(profile.language.as_deref()) {
        request.language(language);
    }

    request
        .build()
        .map_err(|error| AsrRuntimeError::RemoteRequestBuildFailed {
            reason: error.to_string(),
        })
}

fn normalized_optional_string(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn encode_wav_pcm16(samples: &[f32], sample_rate: u32, channels: u16) -> Result<Vec<u8>, String> {
    if sample_rate == 0 {
        return Err(String::from("sample rate must be greater than zero"));
    }
    if channels == 0 {
        return Err(String::from("channel count must be greater than zero"));
    }

    let bytes_per_sample = 2u16;
    let block_align = channels
        .checked_mul(bytes_per_sample)
        .ok_or_else(|| String::from("wav block alignment overflowed"))?;
    let byte_rate = sample_rate
        .checked_mul(u32::from(block_align))
        .ok_or_else(|| String::from("wav byte rate overflowed"))?;

    let data_size = samples
        .len()
        .checked_mul(2)
        .and_then(|size| u32::try_from(size).ok())
        .ok_or_else(|| String::from("wav data chunk was too large"))?;
    let riff_size = 36u32
        .checked_add(data_size)
        .ok_or_else(|| String::from("wav riff chunk overflowed"))?;

    let mut bytes = Vec::with_capacity(44usize.saturating_add(data_size as usize));
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&riff_size.to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&channels.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());

    for sample in samples {
        let scaled = (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round();
        bytes.extend_from_slice(&(scaled as i16).to_le_bytes());
    }

    Ok(bytes)
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
}
