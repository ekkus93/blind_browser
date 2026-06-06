#[cfg(feature = "remote-openai")]
use std::collections::BTreeMap;

use crate::commands::ToolError;
#[cfg(feature = "remote-openai")]
use crate::commands::{PlannerInput, PlannerOutput};
#[cfg(feature = "remote-openai")]
use serde::Serialize;

#[cfg(feature = "remote-openai")]
#[derive(Serialize)]
pub(super) struct PlannerPromptPayload<'a> {
    pub(super) planner_input: &'a PlannerInput,
    pub(super) planner_output_schema: serde_json::Value,
    pub(super) tool_input_schemas: BTreeMap<String, serde_json::Value>,
    pub(super) canonical_planner_output_examples: BTreeMap<String, PlannerOutput>,
}

#[cfg(any(feature = "remote-openai", test))]
pub(super) fn planner_system_prompt() -> &'static str {
    "You are the bounded planner for blind_browser, a voice-first desktop browser for vision-impaired users.
Return only JSON that matches the provided planner_output_schema.
Use only tool names that appear in planner_input.available_tools and only selected_skills that appear in planner_input.active_skill_names.
Every step arguments object must match the corresponding tool_input_schemas entry exactly, including snake_case field names.
Use canonical_planner_output_examples only as shape references; adapt the returned tools, skills, and arguments to the current planner_input.
Keep plans linear and short: at most five steps, with at most one NextStep edge from any step.
When planner_input.safety.allow_click_without_confirmation is true, ordinary ClickElement plans may use Ready without confirm_action; reserve NeedsConfirmation for clicks whose grounded confidence falls below planner_input.safety.confirmation_confidence_threshold or remains ambiguous/risky.
Use NeedsConfirmation plus a confirm_action step when the request is risky or ambiguous before side effects, and do not use confirm_action or confirmation metadata on Ready, Blocked, or Complete plans.
SubmitForm plans must always use NeedsConfirmation with confirm_action before any submit side effect.
Use Blocked only when the request cannot be grounded safely or is outside the supported tool set.
Do not invent tools, skills, statuses, transition kinds, or argument fields."
}

pub(super) fn planner_interpretation_unavailable_error(
    code: &str,
    reason: impl Into<String>,
    retryable: bool,
    details: Option<serde_json::Value>,
) -> ToolError {
    let reason = reason.into();
    let reason = reason.trim().trim_end_matches('.').to_string();

    ToolError {
        code: String::from(code),
        message: format!("Command interpretation is unavailable because {reason}."),
        retryable,
        details,
    }
}
