use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::config::AudioSettings;

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
