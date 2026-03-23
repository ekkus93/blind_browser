use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum BrowserVisibilityMode {
    Visible,
    Headless,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct BrowserSessionConfig {
    pub visibility: BrowserVisibilityMode,
    pub user_agent: Option<String>,
}

impl Default for BrowserSessionConfig {
    fn default() -> Self {
        Self {
            visibility: BrowserVisibilityMode::Visible,
            user_agent: None,
        }
    }
}
