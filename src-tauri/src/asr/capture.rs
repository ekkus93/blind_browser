#[cfg(feature = "audio")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "audio")]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
#[cfg(feature = "audio")]
use cpal::{FromSample, Sample, SampleFormat, SizedSample, Stream, SupportedStreamConfig};

use super::AsrRuntimeError;
use super::processing::CapturedAudio;

#[cfg(feature = "audio")]
type SharedCaptureBuffer = Arc<Mutex<Vec<f32>>>;

#[cfg(feature = "audio")]
pub(super) struct CaptureSession {
    buffer: SharedCaptureBuffer,
    sample_rate: u32,
    channels: u16,
    _stream: Stream,
}

#[cfg(feature = "audio")]
impl CaptureSession {
    pub(super) fn start() -> Result<Self, AsrRuntimeError> {
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

    pub(super) fn snapshot(&self) -> Result<CapturedAudio, AsrRuntimeError> {
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
