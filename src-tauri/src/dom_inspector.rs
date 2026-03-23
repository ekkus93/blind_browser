use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct DomInspectionPolicy {
    pub include_bounding_boxes: bool,
    pub include_accessibility_names: bool,
}

impl Default for DomInspectionPolicy {
    fn default() -> Self {
        Self {
            include_bounding_boxes: true,
            include_accessibility_names: true,
        }
    }
}
