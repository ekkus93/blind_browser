#[cfg(feature = "audio")]
use std::sync::atomic::{AtomicBool, Ordering};
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
    lock_failed: Arc<AtomicBool>,
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
        let lock_failed = Arc::new(AtomicBool::new(false));
        let stream = build_input_stream(
            &device,
            &config,
            Arc::clone(&buffer),
            Arc::clone(&lock_failed),
        )?;
        stream
            .play()
            .map_err(|error| AsrRuntimeError::StartInputStream {
                reason: error.to_string(),
            })?;

        Ok(Self {
            buffer,
            lock_failed,
            sample_rate: config.sample_rate(),
            channels: config.channels(),
            _stream: stream,
        })
    }

    pub(super) fn snapshot(&self) -> Result<CapturedAudio, AsrRuntimeError> {
        if self.lock_failed.load(Ordering::Relaxed) {
            return Err(AsrRuntimeError::AudioBufferLockFailed);
        }
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
    lock_failed: Arc<AtomicBool>,
) -> Result<Stream, AsrRuntimeError> {
    let err_fn = |error| tracing::warn!("audio input stream error: {error}");

    let stream_config = config.config();
    macro_rules! typed_stream {
        ($ty:ty) => {
            build_typed_input_stream::<$ty>(
                device,
                &stream_config,
                Arc::clone(&buffer),
                Arc::clone(&lock_failed),
                err_fn,
            )
        };
    }
    match config.sample_format() {
        SampleFormat::I8 => typed_stream!(i8),
        SampleFormat::I16 => typed_stream!(i16),
        SampleFormat::I32 => typed_stream!(i32),
        SampleFormat::I64 => typed_stream!(i64),
        SampleFormat::U8 => typed_stream!(u8),
        SampleFormat::U16 => typed_stream!(u16),
        SampleFormat::U32 => typed_stream!(u32),
        SampleFormat::U64 => typed_stream!(u64),
        SampleFormat::F32 => typed_stream!(f32),
        SampleFormat::F64 => typed_stream!(f64),
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
    lock_failed: Arc<AtomicBool>,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<Stream, AsrRuntimeError>
where
    T: Sample + SizedSample,
    f32: FromSample<T>,
{
    device
        .build_input_stream(
            config,
            move |data: &[T], _| capture_input_data::<T>(data, &buffer, &lock_failed),
            err_fn,
            None,
        )
        .map_err(|error| AsrRuntimeError::BuildInputStream {
            reason: error.to_string(),
        })
}

#[cfg(feature = "audio")]
fn capture_input_data<T>(input: &[T], buffer: &SharedCaptureBuffer, lock_failed: &AtomicBool)
where
    T: Sample,
    f32: FromSample<T>,
{
    match buffer.lock() {
        Ok(mut guard) => guard.extend(input.iter().copied().map(f32::from_sample)),
        Err(_) => {
            // Only emit one warning per session to avoid flooding the log from every callback.
            if !lock_failed.swap(true, Ordering::Relaxed) {
                tracing::warn!("audio capture buffer lock is poisoned; audio input will be lost");
            }
        }
    }
}
