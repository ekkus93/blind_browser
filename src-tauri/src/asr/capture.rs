#[cfg(feature = "audio")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(feature = "audio")]
use std::sync::Arc;
#[cfg(any(feature = "audio", test))]
use std::sync::Mutex;

#[cfg(feature = "audio")]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
#[cfg(feature = "audio")]
use cpal::{FromSample, Sample, SampleFormat, SizedSample, Stream, SupportedStreamConfig};

use super::processing::CapturedAudio;
use super::AsrRuntimeError;

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

    pub(super) fn take_captured_audio(&self) -> Result<CapturedAudio, AsrRuntimeError> {
        if self.lock_failed.load(Ordering::Relaxed) {
            return Err(AsrRuntimeError::AudioBufferLockFailed);
        }
        let samples = drain_capture_buffer(&self.buffer)?;
        Ok(CapturedAudio {
            samples,
            sample_rate: self.sample_rate,
            channels: self.channels,
        })
    }

    /// Discard whatever audio has accumulated in the buffer so far, without
    /// stopping the stream. Used to reset a hands-free ("keep listening")
    /// capture window before its own listen duration starts, so leftover
    /// audio from between windows — silence, the previous command's own TTS
    /// narration played back over the speaker, planner/browser round-trip
    /// time — never bleeds into the next command. See
    /// [`super::AsrController::begin_capture`] and
    /// [`super::AsrController::capture_audio`] for why this is driven
    /// explicitly by the caller's stop mode rather than inferred here.
    pub(super) fn discard_buffered_audio(&self) -> Result<(), AsrRuntimeError> {
        if self.lock_failed.load(Ordering::Relaxed) {
            return Err(AsrRuntimeError::AudioBufferLockFailed);
        }
        discard_capture_buffer(&self.buffer)
    }
}

#[cfg(any(feature = "audio", test))]
pub(super) fn drain_capture_buffer(buffer: &Mutex<Vec<f32>>) -> Result<Vec<f32>, AsrRuntimeError> {
    let samples = std::mem::take(
        &mut *buffer
            .lock()
            .map_err(|_| AsrRuntimeError::AudioBufferLockFailed)?,
    );
    Ok(samples)
}

/// Like [`drain_capture_buffer`], but the drained samples are discarded
/// rather than returned to the caller.
#[cfg(any(feature = "audio", test))]
pub(super) fn discard_capture_buffer(buffer: &Mutex<Vec<f32>>) -> Result<(), AsrRuntimeError> {
    drain_capture_buffer(buffer).map(|_| ())
}

/// The number of `f32` samples (across all channels) that
/// [`super::MAX_TRANSCRIBE_DURATION_MS`] worth of audio occupies at the
/// given sample rate and channel count. Used as a hard cap on the capture
/// buffer so a session that is never drained or stopped — a stuck
/// hands-free loop, a bug that skips a drain — cannot grow the buffer
/// without bound.
#[cfg(any(feature = "audio", test))]
fn max_buffered_samples(sample_rate: u32, channels: u16) -> usize {
    let total_samples = u64::from(sample_rate)
        .saturating_mul(u64::from(channels))
        .saturating_mul(super::MAX_TRANSCRIBE_DURATION_MS)
        / 1000;
    usize::try_from(total_samples).unwrap_or(usize::MAX)
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
    let max_buffered_samples = max_buffered_samples(config.sample_rate, config.channels);
    device
        .build_input_stream(
            config,
            move |data: &[T], _| {
                capture_input_data::<T>(data, &buffer, &lock_failed, max_buffered_samples)
            },
            err_fn,
            None,
        )
        .map_err(|error| AsrRuntimeError::BuildInputStream {
            reason: error.to_string(),
        })
}

#[cfg(feature = "audio")]
fn capture_input_data<T>(
    input: &[T],
    buffer: &SharedCaptureBuffer,
    lock_failed: &AtomicBool,
    max_buffered_samples: usize,
) where
    T: Sample,
    f32: FromSample<T>,
{
    match buffer.lock() {
        Ok(mut guard) => {
            guard.extend(input.iter().copied().map(f32::from_sample));
            cap_buffered_samples(&mut guard, max_buffered_samples);
        }
        Err(_) => {
            // Only emit one warning per session to avoid flooding the log from every callback.
            if !lock_failed.swap(true, Ordering::Relaxed) {
                tracing::warn!("audio capture buffer lock is poisoned; audio input will be lost");
            }
        }
    }
}

/// Drop the oldest samples in `buffer` so its length never exceeds
/// `max_buffered_samples`. Extracted from [`capture_input_data`] so the cap
/// itself can be unit-tested without a real audio device.
#[cfg(any(feature = "audio", test))]
fn cap_buffered_samples(buffer: &mut Vec<f32>, max_buffered_samples: usize) {
    if buffer.len() > max_buffered_samples {
        let excess = buffer.len() - max_buffered_samples;
        buffer.drain(0..excess);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn consecutive_drains_do_not_return_overlapping_samples() {
        let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
        buffer.lock().unwrap().extend([1.0f32, 2.0, 3.0]);

        let first = drain_capture_buffer(&buffer).unwrap();
        assert_eq!(first, vec![1.0f32, 2.0, 3.0]);
        assert!(buffer.lock().unwrap().is_empty());

        buffer.lock().unwrap().extend([4.0f32, 5.0]);

        let second = drain_capture_buffer(&buffer).unwrap();
        assert_eq!(second, vec![4.0f32, 5.0]);
    }

    #[test]
    fn discard_capture_buffer_clears_the_buffer_without_returning_it() {
        let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
        buffer.lock().unwrap().extend([1.0f32, 2.0, 3.0]);

        discard_capture_buffer(&buffer).unwrap();

        assert!(buffer.lock().unwrap().is_empty());
    }

    #[test]
    fn two_consecutive_hands_free_windows_do_not_return_overlapping_samples() {
        // Simulates the production sequence for a "keep listening" window:
        // discard whatever accumulated between windows (stale silence, the
        // previous command's own narration, round-trip time), then only what
        // the stream appends during this window's own listen duration is
        // drained. This is what distinguishes a hands-free window from a PTT
        // hold, which deliberately skips the discard step.
        let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));

        // Audio that accumulated before this window began (e.g. the
        // assistant's own narration bleeding into an open mic) -- must not
        // survive into either drained window.
        buffer.lock().unwrap().extend([9.0f32, 9.0, 9.0]);

        discard_capture_buffer(&buffer).unwrap();
        buffer.lock().unwrap().extend([1.0f32, 2.0]);
        let first_window = drain_capture_buffer(&buffer).unwrap();
        assert_eq!(first_window, vec![1.0f32, 2.0]);

        // More audio drifts in between windows (round-trip time) before the
        // next window's reset runs.
        buffer.lock().unwrap().extend([9.0f32]);

        discard_capture_buffer(&buffer).unwrap();
        buffer.lock().unwrap().extend([3.0f32, 4.0, 5.0]);
        let second_window = drain_capture_buffer(&buffer).unwrap();
        assert_eq!(second_window, vec![3.0f32, 4.0, 5.0]);
    }

    #[test]
    fn max_buffered_samples_covers_max_transcribe_duration_at_a_given_rate() {
        // 16 kHz mono, MAX_TRANSCRIBE_DURATION_MS = 10_000ms -> 160_000 samples.
        assert_eq!(max_buffered_samples(16_000, 1), 160_000);
        // Stereo doubles the per-ms sample count.
        assert_eq!(max_buffered_samples(16_000, 2), 320_000);
    }

    #[test]
    fn cap_buffered_samples_drops_the_oldest_samples_past_the_cap() {
        let mut buffer = vec![1.0f32, 2.0, 3.0, 4.0, 5.0];

        cap_buffered_samples(&mut buffer, 3);

        assert_eq!(buffer, vec![3.0f32, 4.0, 5.0]);
    }

    #[test]
    fn cap_buffered_samples_is_a_no_op_under_the_cap() {
        let mut buffer = vec![1.0f32, 2.0];

        cap_buffered_samples(&mut buffer, 3);

        assert_eq!(buffer, vec![1.0f32, 2.0]);
    }

    #[test]
    fn cap_buffered_samples_holds_even_under_sustained_appends() {
        // A session that is never stopped or drained (a stuck hands-free
        // loop, a bug that skips a drain) must never grow the buffer past
        // the cap, no matter how many callback invocations accumulate.
        let mut buffer = Vec::<f32>::new();
        let cap = 100;

        for _ in 0..1_000 {
            buffer.extend([0.0f32; 7]);
            cap_buffered_samples(&mut buffer, cap);
        }

        assert_eq!(buffer.len(), cap);
    }
}
