use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::audio_io::RuntimeAudioState;
use crate::browser::{BrowserVisibilityMode, LoadState, ScrollDirection, ScrollTarget};
use crate::config::{
    LocalAsrBackend, LocalTtsBackend, ProviderMode, RemoteTtsAudioFormat, MAX_PLAYBACK_SPEED,
    MAX_PLAYBACK_VOLUME, MIN_PLAYBACK_SPEED,
};
use crate::narration::NarrationCursor;
use crate::page_model::{ExtractionSource, InteractiveElement, PageModel, Rect};
use crate::state::{BrowserHistoryState, ListeningState};

mod contracts;
mod planner_executor;
mod registry;
mod routing;
mod validators;

pub use contracts::*;
pub use planner_executor::*;
pub use registry::*;
pub use routing::*;
pub use validators::*;

#[cfg(test)]
pub(crate) use planner_executor::execute_planner_output_with_runner;
#[cfg(test)]
pub(crate) use registry::{
    parse_bundled_skills, parse_intent_name_value, parse_skill_document, schema_json,
    BUNDLED_SKILLS_MARKDOWN, MAX_SELECTED_PLANNER_SKILLS,
};
#[cfg(test)]
pub(crate) use registry::{discover_skills, SkillSource};
#[cfg(test)]
pub(crate) use validators::{validate_confirm_action_input, validate_planned_step_arguments};

const MAX_INITIAL_PLAN_STEPS: usize = 5;
pub(crate) const MAX_HISTORY_STEPS: u8 = 5;
pub(crate) const MAX_SCROLL_AMOUNT_PX: f32 = 4_000.0;
pub(crate) const DEFAULT_FIND_ELEMENT_MAX_CANDIDATES: usize = 3;
const DEFAULT_VOLUME_STEP: f32 = 0.10;
const SMALL_VOLUME_STEP: f32 = 0.05;
const LARGE_VOLUME_STEP: f32 = 0.20;
const DEFAULT_SPEED_STEP: f32 = 0.25;
const SMALL_SPEED_STEP: f32 = 0.10;
const LARGE_SPEED_STEP: f32 = 0.50;

struct StepExecutionContext<'a> {
    request_id: String,
    intent_name: IntentName,
    selected_skills: Vec<String>,
    steps: &'a [PlannedStep],
    initial_step_id: String,
    block_side_effects_until_confirmation: bool,
}

fn current_timestamp_ms() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as u64,
        Err(_) => 0,
    }
}

#[cfg(test)]
mod tests;
