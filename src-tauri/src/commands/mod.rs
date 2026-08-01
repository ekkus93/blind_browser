use std::collections::{BTreeMap, HashSet};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
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

mod action_policy;
mod confirmation_manifest;
mod contracts;
mod planner_executor;
mod registry;
mod routing;
mod schemas;
mod skill_loader;
mod skill_parser;
mod validators;

pub use action_policy::*;
pub use confirmation_manifest::*;
pub use contracts::*;
pub use planner_executor::*;
pub use registry::*;
pub use routing::*;
pub use schemas::*;
pub use validators::*;

#[cfg(test)]
pub(crate) use planner_executor::execute_planner_output_with_runner;
#[cfg(test)]
pub(crate) use registry::MAX_SELECTED_PLANNER_SKILLS;
#[cfg(test)]
pub(crate) use schemas::schema_json;
#[cfg(test)]
pub(crate) use skill_loader::{discover_skills, BUNDLED_SKILLS_MARKDOWN};
#[cfg(test)]
pub(crate) use skill_parser::{
    parse_bundled_skills, parse_intent_name_value, parse_skill_document,
};
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

/// Monotonic fallback used only when the system clock is before the UNIX epoch, so
/// a clock fault produces distinct increasing values rather than a stream of `0`
/// (which would collapse to duplicate, non-informative ids).
static FALLBACK_TIMESTAMP_COUNTER: AtomicU64 = AtomicU64::new(1);

pub(crate) fn current_timestamp_ms() -> u64 {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_millis() as u64);
    timestamp_ms_or_fallback(now_ms)
}

/// Resolve a millisecond timestamp, falling back to a monotonic counter when the
/// clock read failed (`None`) instead of returning `0`.
fn timestamp_ms_or_fallback(now_ms: Option<u64>) -> u64 {
    now_ms.unwrap_or_else(|| FALLBACK_TIMESTAMP_COUNTER.fetch_add(1, Ordering::Relaxed))
}

#[cfg(test)]
mod timestamp_tests {
    use super::*;

    #[test]
    fn timestamp_ms_or_fallback_uses_the_clock_value_when_present() {
        assert_eq!(timestamp_ms_or_fallback(Some(1_234)), 1_234);
    }

    #[test]
    fn timestamp_ms_or_fallback_yields_distinct_increasing_values_without_a_clock() {
        let first = timestamp_ms_or_fallback(None);
        let second = timestamp_ms_or_fallback(None);
        assert_ne!(first, 0, "fallback must not silently return zero");
        assert!(second > first, "fallback ids must be strictly increasing");
    }
}

#[cfg(test)]
mod tests;
