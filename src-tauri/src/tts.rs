use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum TtsProviderKind {
    Local,
    Remote,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct TtsSettings {
    pub provider: TtsProviderKind,
    pub voice: Option<String>,
    pub playback_speed: f32,
}

impl Default for TtsSettings {
    fn default() -> Self {
        Self {
            provider: TtsProviderKind::Local,
            voice: Some(String::from("Bruno")),
            playback_speed: 1.0,
        }
    }
}
