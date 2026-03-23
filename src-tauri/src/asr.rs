use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
