use super::api_key_tools::{fetch_openai_compatible_models, test_openai_api_key_connectivity};
use super::element_scoring::{
    build_find_element_query, determine_find_element_resolution, filter_interactive_elements,
    normalize_optional_text, rank_find_element_candidates, region_bbox_by_id,
};
use super::extraction_tools::should_trigger_extract_page_model_ocr_fallback;
use super::fill_correction::{resolve_recent_fill_correction_command, RecentFieldContext};
use super::form_fill::{
    resolve_direct_fill_and_submit_command, resolve_direct_fill_field_command,
    resolve_direct_focus_field_command, resolve_direct_submit_form_command,
};
use super::interaction_tools::{
    resolve_clickable_element, resolve_form_element, resolve_typeable_element,
};
use super::navigation_tools::{
    browser_error_to_tool_error, clear_navigation_follow_up_state, normalize_absolute_url,
    refresh_current_page_after_navigation,
};
use super::ocr_merge::{
    extracted_text_metrics, merge_ocr_text_into_page_model, merged_region_text,
    region_first_ocr_target_ids,
};
use super::page_model_builder::{
    build_extracted_page_model, build_visible_text_excerpt, infer_extraction_source,
};
use super::planner_prompt::{planner_interpretation_unavailable_error, planner_system_prompt};
use super::replanning::execute_bounded_replanning_loop;
use super::replanning::ReplanningRuntime;
use super::settings_adapters::{
    build_asr_provider_settings, build_confirmation_settings, build_local_asr_model_settings,
    build_local_tts_model_settings, build_ocr_threshold_settings, build_provider_failover_settings,
    build_remote_asr_settings, build_remote_planner_settings, build_remote_tts_settings,
    build_tts_model_settings, build_tts_provider_settings, build_tts_voice_settings,
};
use crate::audio_io::RuntimeAudioState;
use crate::browser::BrowserError;
use crate::commands::{
    ExecutionOutcome, ExecutionTrace, ExtractPageModelInput, FindElementInput, IntentName,
    IntentSummary, PlannedStep, PlannerOutput, PlannerStatus, PlannerToolHistoryEntry,
    ReportStatus, StepTransition, ToolName, ToolResult,
};
use crate::config::{AppConfig, KeyringRef, ProviderMode, SecretRef};
use crate::ocr::OcrSettings;
use crate::page_model::{
    ElementRole, ExtractionSource, InteractiveElement, PageModel, PageRegion, Rect, RegionRole,
    RegionSource,
};
use crate::state::AppState;

mod helpers;
use helpers::*;

mod browser_tests;
mod element_scoring_tests;
mod extraction_tests;
mod fill_correction_tests;
mod focus_fill_tests;
mod ocr_merge_tests;
mod ocr_threshold_tests;
mod planner_tests;
mod regression_tests;
mod settings_tests;
