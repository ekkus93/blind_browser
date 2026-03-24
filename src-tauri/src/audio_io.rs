use std::num::{NonZeroU16, NonZeroU32};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::AudioSettings;

#[cfg(feature = "audio")]
use rodio::{buffer::SamplesBuffer, DeviceSinkBuilder, MixerDeviceSink, Player};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct RuntimeAudioState {
    pub playback_volume: f32,
    pub playback_speed: f32,
    pub muted: bool,
    pub tts_voice: Option<String>,
}

impl Default for RuntimeAudioState {
    fn default() -> Self {
        Self {
            playback_volume: 1.0,
            playback_speed: 1.0,
            muted: false,
            tts_voice: Some(String::from("Bruno")),
        }
    }
}

impl From<&AudioSettings> for RuntimeAudioState {
    fn from(audio: &AudioSettings) -> Self {
        Self {
            playback_volume: audio.playback_volume,
            playback_speed: audio.playback_speed,
            muted: audio.playback_volume == 0.0,
            tts_voice: Some(audio.default_tts_voice.clone()),
        }
    }
}

#[derive(Debug, Error)]
pub enum AudioPlaybackError {
    #[error("audio playback requires the 'audio' feature to be enabled")]
    AudioFeatureUnavailable,
    #[error("audio playback requires a non-zero channel count")]
    InvalidChannelCount,
    #[error("audio playback requires a non-zero sample rate")]
    InvalidSampleRate,
    #[error("failed to open the default audio output device: {reason}")]
    OpenDevice { reason: String },
}

pub struct AudioPlaybackController {
    #[cfg(feature = "audio")]
    active_playback: Option<ActivePlayback>,
}

impl AudioPlaybackController {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "audio")]
            active_playback: None,
        }
    }

    pub fn play_samples(
        &mut self,
        samples: Vec<f32>,
        channels: u16,
        sample_rate: u32,
        volume: f32,
    ) -> Result<(), AudioPlaybackError> {
        #[cfg(not(feature = "audio"))]
        {
            let _ = (samples, channels, sample_rate, volume);
            Err(AudioPlaybackError::AudioFeatureUnavailable)
        }

        #[cfg(feature = "audio")]
        {
            self.stop();

            let channels =
                NonZeroU16::new(channels).ok_or(AudioPlaybackError::InvalidChannelCount)?;
            let sample_rate =
                NonZeroU32::new(sample_rate).ok_or(AudioPlaybackError::InvalidSampleRate)?;
            let device = DeviceSinkBuilder::open_default_sink().map_err(|error| {
                AudioPlaybackError::OpenDevice {
                    reason: error.to_string(),
                }
            })?;
            let player = Player::connect_new(device.mixer());
            player.set_volume(volume);
            player.append(SamplesBuffer::new(channels, sample_rate, samples));

            self.active_playback = Some(ActivePlayback {
                _device: device,
                player,
            });
            Ok(())
        }
    }

    pub fn stop(&mut self) -> bool {
        #[cfg(not(feature = "audio"))]
        {
            false
        }

        #[cfg(feature = "audio")]
        {
            if let Some(active_playback) = self.active_playback.take() {
                active_playback.player.stop();
                true
            } else {
                false
            }
        }
    }

    pub fn is_active(&mut self) -> bool {
        #[cfg(not(feature = "audio"))]
        {
            false
        }

        #[cfg(feature = "audio")]
        {
            if self
                .active_playback
                .as_ref()
                .is_some_and(|active_playback| active_playback.player.empty())
            {
                self.active_playback = None;
            }

            self.active_playback.is_some()
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        #[cfg(feature = "audio")]
        if let Some(active_playback) = self.active_playback.as_ref() {
            active_playback.player.set_volume(volume);
        }
    }
}

impl Default for AudioPlaybackController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "audio")]
struct ActivePlayback {
    _device: MixerDeviceSink,
    player: Player,
}
