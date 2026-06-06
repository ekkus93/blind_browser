use super::*;

mod intent;
pub use intent::infer_intent_hint;
pub(crate) use intent::{normalize_transcript_for_routing, tokenize_text, likely_tools_for_intent};
use intent::*;

mod audio_commands;
pub(crate) use audio_commands::{
    resolve_direct_audio_command, resolve_direct_browser_visibility_command,
};

mod url_commands;
use url_commands::*;
pub(crate) use url_commands::resolve_direct_open_url_command;

mod planner_outputs;
use planner_outputs::*;

mod status_commands;
use status_commands::*;
pub(crate) use status_commands::{
    resolve_direct_read_title_command, resolve_direct_repeat_command,
    resolve_direct_status_query_command,
};

mod navigation_routing;
pub(crate) use navigation_routing::resolve_direct_navigation_readback_command;

mod voice_routing;
pub(crate) use voice_routing::resolve_direct_voice_input_command;

mod reading_routing;
pub(crate) use reading_routing::resolve_direct_read_page_command;

mod field_routing;
pub(crate) use field_routing::{
    is_direct_submit_form_command, parse_direct_fill_and_submit_command,
    parse_direct_fill_field_command, parse_direct_focus_field_command,
    parse_fill_field_correction_command,
};

fn normalized_optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(String::from)
}
