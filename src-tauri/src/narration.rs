use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
pub struct NarrationCursor {
    pub current_region_id: Option<String>,
    pub current_index: Option<usize>,
    pub total_regions: usize,
}
