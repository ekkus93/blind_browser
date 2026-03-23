use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct OcrSettings {
    pub trigger_on_no_extractable_text: bool,
    pub sparse_text_char_threshold: u32,
    pub sparse_text_region_threshold: u32,
    pub prefer_region_ocr: bool,
}

impl Default for OcrSettings {
    fn default() -> Self {
        Self {
            trigger_on_no_extractable_text: true,
            sparse_text_char_threshold: 200,
            sparse_text_region_threshold: 2,
            prefer_region_ocr: true,
        }
    }
}
