use super::{AsrRuntimeError, WHISPER_TARGET_SAMPLE_RATE};

#[cfg(feature = "remote-openai")]
use super::wav::encode_wav_pcm16;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CapturedAudio {
    pub(crate) samples: Vec<f32>,
    pub(crate) sample_rate: u32,
    pub(crate) channels: u16,
}

impl CapturedAudio {
    #[cfg(feature = "audio")]
    pub(crate) fn ensure_non_empty(self) -> Result<Self, AsrRuntimeError> {
        if self.samples.is_empty() {
            Err(AsrRuntimeError::NoAudioCaptured)
        } else {
            Ok(self)
        }
    }

    pub(crate) fn duration_ms(&self) -> u64 {
        if self.sample_rate == 0 || self.channels == 0 {
            return 0;
        }

        let frame_count = self.samples.len() / usize::from(self.channels);
        ((frame_count as f64 / f64::from(self.sample_rate)) * 1000.0).round() as u64
    }

    pub(crate) fn to_whisper_audio(&self) -> Vec<f32> {
        let mono = interleaved_to_mono(&self.samples, self.channels);
        resample_linear(&mono, self.sample_rate, WHISPER_TARGET_SAMPLE_RATE)
    }

    #[cfg(feature = "remote-openai")]
    pub(crate) fn to_remote_wav_bytes(&self) -> Result<Vec<u8>, AsrRuntimeError> {
        let mono = interleaved_to_mono(&self.samples, self.channels);
        let audio = resample_linear(&mono, self.sample_rate, WHISPER_TARGET_SAMPLE_RATE);
        encode_wav_pcm16(&audio, WHISPER_TARGET_SAMPLE_RATE, 1)
            .map_err(|reason| AsrRuntimeError::RemoteAudioEncodeFailed { reason })
    }
}

pub(super) fn interleaved_to_mono(samples: &[f32], channels: u16) -> Vec<f32> {
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

pub(super) fn resample_linear(
    samples: &[f32],
    input_sample_rate: u32,
    output_sample_rate: u32,
) -> Vec<f32> {
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
