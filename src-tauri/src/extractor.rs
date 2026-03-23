use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ExtractionPolicy {
    pub prefer_dom: bool,
    pub enable_sparse_text_checks: bool,
}

impl Default for ExtractionPolicy {
    fn default() -> Self {
        Self {
            prefer_dom: true,
            enable_sparse_text_checks: true,
        }
    }
}
