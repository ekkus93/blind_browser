from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text()


def write(path: str, content: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content)


def replace_once(path: str, old: str, new: str) -> None:
    content = read(path)
    count = content.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one match, found {count}: {old[:120]!r}")
    write(path, content.replace(old, new, 1))


# ---------------------------------------------------------------------------
# Runtime state: page generation plus non-serializable authorization records.
# ---------------------------------------------------------------------------
replace_once(
    "src-tauri/src/state.rs",
    "use schemars::JsonSchema;\n",
    "use std::collections::BTreeMap;\n\nuse schemars::JsonSchema;\n",
)
replace_once(
    "src-tauri/src/state.rs",
    "use crate::commands::{ExecutionOutcome, LastToolCallSummary, PendingPlanExecutionState};",
    "use crate::commands::{\n    ExecutionOutcome, LastToolCallSummary, PendingPlanExecutionState, PlannerSafetySettings,\n};",
)
replace_once(
    "src-tauri/src/state.rs",
    "impl Default for ListeningState {\n    fn default() -> Self {\n        Self {\n            is_listening: false,\n            push_to_talk_enabled: true,\n        }\n    }\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]",
    "impl Default for ListeningState {\n    fn default() -> Self {\n        Self {\n            is_listening: false,\n            push_to_talk_enabled: true,\n        }\n    }\n}\n\n#[derive(Debug, Clone, PartialEq, Eq)]\npub(crate) struct ClickAuthorizationRecord {\n    pub token: String,\n    pub page_id: String,\n    pub page_generation: u64,\n    pub origin: Option<String>,\n    pub element_id: String,\n    pub dom_locator: String,\n    pub element_fingerprint: String,\n    pub confidence_bps: Option<u16>,\n    pub ambiguous: bool,\n    pub potentially_destructive: bool,\n    pub issued_at_ms: u64,\n    pub expires_at_ms: u64,\n}\n\n#[derive(Debug, Clone, PartialEq)]\npub(crate) struct PlanningStateSnapshot {\n    pub page_id: Option<String>,\n    pub page_generation: u64,\n    pub origin: Option<String>,\n    pub browser_history: BrowserHistoryState,\n    pub safety: PlannerSafetySettings,\n    pub pending_confirmation_id: Option<String>,\n    pub issued_at_ms: u64,\n    pub expires_at_ms: u64,\n}\n\n#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]",
)
replace_once(
    "src-tauri/src/state.rs",
    "pub struct AppState {\n    pub current_page_id: Option<String>,\n    pub current_page: Option<PageModel>,",
    "pub struct AppState {\n    pub current_page_id: Option<String>,\n    pub current_page: Option<PageModel>,\n    pub page_generation: u64,",
)
replace_once(
    "src-tauri/src/state.rs",
    "    pub pending_confirmation_id: Option<String>,\n    pub pending_plan_execution: Option<PendingPlanExecutionState>,\n}",
    "    pub pending_confirmation_id: Option<String>,\n    pub pending_plan_execution: Option<PendingPlanExecutionState>,\n    #[serde(skip, default)]\n    #[schemars(skip)]\n    pub(crate) click_authorizations: BTreeMap<String, ClickAuthorizationRecord>,\n    #[serde(skip, default)]\n    #[schemars(skip)]\n    pub(crate) planning_snapshots: BTreeMap<String, PlanningStateSnapshot>,\n}",
)
replace_once(
    "src-tauri/src/state.rs",
    "            current_page_id: None,\n            current_page: None,\n            browser_visibility:",
    "            current_page_id: None,\n            current_page: None,\n            page_generation: 0,\n            browser_visibility:",
)
replace_once(
    "src-tauri/src/state.rs",
    "            pending_confirmation_id: None,\n            pending_plan_execution: None,\n        }",
    "            pending_confirmation_id: None,\n            pending_plan_execution: None,\n            click_authorizations: BTreeMap::new(),\n            planning_snapshots: BTreeMap::new(),\n        }",
)
replace_once(
    "src-tauri/src/state.rs",
    "    pub fn apply_audio_settings(&mut self, audio: &AudioSettings) {\n        self.audio = RuntimeAudioState::from(audio);\n    }\n\n    pub fn record_navigation",
    "    pub fn apply_audio_settings(&mut self, audio: &AudioSettings) {\n        self.audio = RuntimeAudioState::from(audio);\n    }\n\n    pub fn mark_page_model_changed(&mut self) {\n        self.page_generation = self.page_generation.saturating_add(1).max(1);\n        self.click_authorizations.clear();\n        self.clear_pending_execution();\n    }\n\n    pub fn replace_current_page_model(&mut self, page_model: PageModel) {\n        self.current_page = Some(page_model);\n        self.mark_page_model_changed();\n    }\n\n    pub fn confirmation_page_identity(&self) -> Option<String> {\n        self.current_page_id\n            .as_ref()\n            .map(|page_id| format!(\"{page_id}@generation:{}\", self.page_generation))\n    }\n\n    pub fn record_navigation",
)
replace_once(
    "src-tauri/src/state.rs",
    "        self.current_page = Some(PageModel {\n            title: None,\n            url: Some(page_url),\n            regions: Vec::new(),\n            interactive_elements: Vec::new(),\n        });\n        self.browser_history",
    "        self.current_page = Some(PageModel {\n            title: None,\n            url: Some(page_url),\n            regions: Vec::new(),\n            interactive_elements: Vec::new(),\n        });\n        self.mark_page_model_changed();\n        self.browser_history",
)

# ---------------------------------------------------------------------------
# Action policy: runtime-injected click authorization metadata.
# ---------------------------------------------------------------------------
replace_once(
    "src-tauri/src/commands/action_policy.rs",
    "use super::*;\n",
    "use super::*;\n\npub(crate) const CLICK_AUTH_TOKEN_ARG: &str = \"_runtime_click_authorization\";\npub(crate) const CLICK_AUTH_CONFIDENCE_ARG: &str = \"_runtime_click_confidence_bps\";\npub(crate) const CLICK_AUTH_AMBIGUOUS_ARG: &str = \"_runtime_click_ambiguous\";\npub(crate) const CLICK_AUTH_DESTRUCTIVE_ARG: &str = \"_runtime_click_potentially_destructive\";\npub(crate) const CLICK_AUTH_GENERATION_ARG: &str = \"_runtime_click_page_generation\";\npub(crate) const RUNTIME_TARGET_LABEL_ARG: &str = \"_runtime_target_label\";\npub(crate) const RUNTIME_FORM_LABEL_ARG: &str = \"_runtime_form_label\";\npub(crate) const RUNTIME_FORM_DESTINATION_ARG: &str = \"_runtime_form_destination\";\npub(crate) const RUNTIME_FORM_FIELDS_ARG: &str = \"_runtime_form_fields\";\n",
)
replace_once(
    "src-tauri/src/commands/action_policy.rs",
    "    ClickRequiresConfirmationBySetting,\n    ClickGroundingUnavailable,",
    "    ClickRequiresConfirmationBySetting,\n    ClickGroundingUnavailable,\n    ClickGroundingAuthorized,\n    ClickConfidenceBelowThreshold,\n    ClickGroundingAmbiguous,\n    ClickTargetPotentiallyDestructive,",
)
old_click = """            ToolName::ClickElement => {
                step_requirement = ConfirmationRequirement::ConfirmationRequired;
                reason_code = if safety.allow_click_without_confirmation {
                    // The current planner contract carries only an element id. It has
                    // no page-bound, versioned grounding authorization, so the
                    // configured click exception cannot be exercised safely yet.
                    ActionPolicyReasonCode::ClickGroundingUnavailable
                } else {
                    ActionPolicyReasonCode::ClickRequiresConfirmationBySetting
                };
            }
"""
new_click = """            ToolName::ClickElement => {
                let token_present = step
                    .arguments
                    .get(CLICK_AUTH_TOKEN_ARG)
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|value| !value.trim().is_empty());
                let confidence_bps = step
                    .arguments
                    .get(CLICK_AUTH_CONFIDENCE_ARG)
                    .and_then(serde_json::Value::as_u64)
                    .and_then(|value| u16::try_from(value).ok());
                let ambiguous = step
                    .arguments
                    .get(CLICK_AUTH_AMBIGUOUS_ARG)
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true);
                let potentially_destructive = step
                    .arguments
                    .get(CLICK_AUTH_DESTRUCTIVE_ARG)
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true);
                let threshold_bps =
                    (safety.confirmation_confidence_threshold.clamp(0.0, 1.0) * 10_000.0)
                        .round() as u16;

                let (requirement, reason) = if !safety.allow_click_without_confirmation {
                    (
                        ConfirmationRequirement::ConfirmationRequired,
                        ActionPolicyReasonCode::ClickRequiresConfirmationBySetting,
                    )
                } else if !token_present || confidence_bps.is_none() {
                    (
                        ConfirmationRequirement::ConfirmationRequired,
                        ActionPolicyReasonCode::ClickGroundingUnavailable,
                    )
                } else if ambiguous {
                    (
                        ConfirmationRequirement::ConfirmationRequired,
                        ActionPolicyReasonCode::ClickGroundingAmbiguous,
                    )
                } else if potentially_destructive {
                    (
                        ConfirmationRequirement::ConfirmationRequired,
                        ActionPolicyReasonCode::ClickTargetPotentiallyDestructive,
                    )
                } else if confidence_bps.is_none_or(|value| value < threshold_bps) {
                    (
                        ConfirmationRequirement::ConfirmationRequired,
                        ActionPolicyReasonCode::ClickConfidenceBelowThreshold,
                    )
                } else {
                    (
                        ConfirmationRequirement::NoConfirmation,
                        ActionPolicyReasonCode::ClickGroundingAuthorized,
                    )
                };
                step_requirement = requirement;
                reason_code = reason;
            }
"""
replace_once("src-tauri/src/commands/action_policy.rs", old_click, new_click)

# Allow click-only plans to reach the AppCore runtime, which either injects a
# valid authorization or inserts a deterministic confirmation gate. The generic
# executor remains strict and still rejects an unprepared Ready click.
replace_once(
    "src-tauri/src/commands/validators/mod.rs",
    "        ConfirmationRequirement::ConfirmationRequired => {\n            validate_runtime_confirmation_gate(planner_output, &decision)\n        }",
    "        ConfirmationRequirement::ConfirmationRequired\n            if planner_output.status == PlannerStatus::Ready\n                && decision\n                    .findings\n                    .iter()\n                    .all(|finding| finding.tool_name == ToolName::ClickElement) =>\n        {\n            Ok(())\n        }\n        ConfirmationRequirement::ConfirmationRequired => {\n            validate_runtime_confirmation_gate(planner_output, &decision)\n        }",
)

# ---------------------------------------------------------------------------
# Executor supports actual runtime safety and a raw-step preflight hook.
# ---------------------------------------------------------------------------
replace_once(
    "src-tauri/src/commands/contracts/mod.rs",
    "    fn confirmation_runtime_context(&self) -> ConfirmationRuntimeContext {\n        ConfirmationRuntimeContext::detached()\n    }\n\n    fn execute_open_url",
    "    fn confirmation_runtime_context(&self) -> ConfirmationRuntimeContext {\n        ConfirmationRuntimeContext::detached()\n    }\n\n    fn preflight_planned_step(&mut self, _step: &PlannedStep) -> Result<(), ToolError> {\n        Ok(())\n    }\n\n    fn execute_open_url",
)
replace_once(
    "src-tauri/src/commands/planner_executor/tool_dispatch.rs",
    "pub fn execute_planned_step<E: DeterministicToolExecutor>(\n    executor: &mut E,\n    step: &PlannedStep,\n) -> SerializedToolResult {\n    match step.tool_name {",
    "pub fn execute_planned_step<E: DeterministicToolExecutor>(\n    executor: &mut E,\n    step: &PlannedStep,\n) -> SerializedToolResult {\n    if let Err(error) = executor.preflight_planned_step(step) {\n        return ToolResult::failure(\n            step.tool_name.clone(),\n            inferred_request_id(step),\n            error,\n            vec![String::from(\n                \"Runtime preflight rejected the planned step before dispatch.\",\n            )],\n        );\n    }\n\n    match step.tool_name {",
)
replace_once(
    "src-tauri/src/commands/planner_executor/execution.rs",
    "fn executor_minimum_safety() -> PlannerSafetySettings {",
    "pub(super) fn executor_minimum_safety() -> PlannerSafetySettings {",
)
replace_once(
    "src-tauri/src/commands/planner_executor/execution.rs",
    "fn initial_execution_policy_error(planner_output: &PlannerOutput) -> Option<ToolError> {\n    let decision = evaluate_action_policy(&planner_output.steps, &executor_minimum_safety());",
    "fn initial_execution_policy_error(\n    planner_output: &PlannerOutput,\n    safety: &PlannerSafetySettings,\n) -> Option<ToolError> {\n    let decision = evaluate_action_policy(&planner_output.steps, safety);",
)
replace_once(
    "src-tauri/src/commands/planner_executor/execution.rs",
    "    let context = ConfirmationRuntimeContext::detached();\n    execute_planner_output_with_runner_and_context(request_id, planner_output, &context, run_step)",
    "    let context = ConfirmationRuntimeContext::detached();\n    let safety = executor_minimum_safety();\n    execute_planner_output_with_runner_and_context(\n        request_id,\n        planner_output,\n        &context,\n        &safety,\n        run_step,\n    )",
)
replace_once(
    "src-tauri/src/commands/planner_executor/execution.rs",
    "pub(crate) fn execute_planner_output_with_runner_and_context<Runner>(\n    request_id: String,\n    planner_output: &PlannerOutput,\n    confirmation_context: &ConfirmationRuntimeContext,\n    mut run_step: Runner,",
    "pub(crate) fn execute_planner_output_with_runner_and_context<Runner>(\n    request_id: String,\n    planner_output: &PlannerOutput,\n    confirmation_context: &ConfirmationRuntimeContext,\n    safety: &PlannerSafetySettings,\n    mut run_step: Runner,",
)
replace_once(
    "src-tauri/src/commands/planner_executor/execution.rs",
    "    if let Some(error) = initial_execution_policy_error(planner_output) {",
    "    if let Some(error) = initial_execution_policy_error(planner_output, safety) {",
)
replace_once(
    "src-tauri/src/commands/planner_executor/mod.rs",
    "    execution::execute_planner_output_with_runner_and_context(\n        request_id,\n        planner_output,\n        context,\n        |step| tool_dispatch::execute_planned_step(executor, step),\n    )",
    "    let safety = execution::executor_minimum_safety();\n    execution::execute_planner_output_with_runner_and_context(\n        request_id,\n        planner_output,\n        context,\n        &safety,\n        |step| tool_dispatch::execute_planned_step(executor, step),\n    )",
)
replace_once(
    "src-tauri/src/commands/planner_executor/mod.rs",
    "pub fn resume_after_confirmation<E: DeterministicToolExecutor>(",
    "pub fn execute_planner_output_with_runtime_safety<E: DeterministicToolExecutor>(\n    executor: &mut E,\n    request_id: String,\n    planner_output: &PlannerOutput,\n    safety: &PlannerSafetySettings,\n) -> ExecutionOutcome {\n    let context = executor.confirmation_runtime_context();\n    execution::execute_planner_output_with_runner_and_context(\n        request_id,\n        planner_output,\n        &context,\n        safety,\n        |step| tool_dispatch::execute_planned_step(executor, step),\n    )\n}\n\npub fn resume_after_confirmation<E: DeterministicToolExecutor>(",
)

# ---------------------------------------------------------------------------
# New planning snapshot and click authorization modules.
# ---------------------------------------------------------------------------
write(
    "src-tauri/src/app_core/planning_snapshot.rs",
    r'''use sha2::{Digest, Sha256};

use crate::commands::{
    current_timestamp_ms, normalized_origin, PlannedStep, PlannerOutput, PlannerSafetySettings,
    ToolError, ToolName,
};
use crate::state::PlanningStateSnapshot;

const PLANNING_SNAPSHOT_TTL_MS: u64 = 120_000;
const MAX_PLANNING_SNAPSHOTS: usize = 32;

impl super::AppCore {
    pub(crate) fn capture_planning_state_snapshot(&self) -> PlanningStateSnapshot {
        let issued_at_ms = current_timestamp_ms();
        PlanningStateSnapshot {
            page_id: self.state.current_page_id.clone(),
            page_generation: self.state.page_generation,
            origin: normalized_origin(
                self.state
                    .current_page
                    .as_ref()
                    .and_then(|page| page.url.as_deref()),
            ),
            browser_history: self.state.browser_history.clone(),
            safety: PlannerSafetySettings::from(&self.config.safety),
            pending_confirmation_id: self.state.pending_confirmation_id.clone(),
            issued_at_ms,
            expires_at_ms: issued_at_ms.saturating_add(PLANNING_SNAPSHOT_TTL_MS),
        }
    }

    pub(crate) fn register_planning_snapshot(
        &mut self,
        planner_output: &PlannerOutput,
        snapshot: PlanningStateSnapshot,
    ) -> Result<(), ToolError> {
        let digest = planner_output_digest(planner_output)?;
        let now_ms = current_timestamp_ms();
        self.state
            .planning_snapshots
            .retain(|_, stored| stored.expires_at_ms > now_ms);
        self.state.planning_snapshots.insert(digest, snapshot);
        while self.state.planning_snapshots.len() > MAX_PLANNING_SNAPSHOTS {
            let oldest = self
                .state
                .planning_snapshots
                .iter()
                .min_by_key(|(_, stored)| stored.issued_at_ms)
                .map(|(digest, _)| digest.clone());
            let Some(oldest) = oldest else { break };
            self.state.planning_snapshots.remove(&oldest);
        }
        Ok(())
    }

    pub(crate) fn validate_and_consume_planning_snapshot(
        &mut self,
        planner_output: &PlannerOutput,
    ) -> Result<(), ToolError> {
        if !planner_output_requires_snapshot(&planner_output.steps) {
            return Ok(());
        }

        let digest = planner_output_digest(planner_output)?;
        let Some(expected) = self.state.planning_snapshots.remove(&digest) else {
            return Err(planning_error(
                "missing_planning_snapshot",
                "side-effecting planner output was not bound to a runtime planning snapshot",
                None,
            ));
        };
        let now_ms = current_timestamp_ms();
        if now_ms >= expected.expires_at_ms {
            return Err(planning_error(
                "planning_snapshot_expired",
                "the runtime state snapshot used for planning expired before execution",
                Some(serde_json::json!({
                    "expired_at_ms": expected.expires_at_ms,
                    "observed_at_ms": now_ms,
                })),
            ));
        }

        let observed = self.capture_planning_state_snapshot();
        if !planning_snapshots_match(&expected, &observed) {
            return Err(planning_error(
                "stale_planning_snapshot",
                "runtime state changed after the plan was resolved; the plan must be rebuilt",
                Some(serde_json::json!({
                    "expected_page_id": expected.page_id,
                    "observed_page_id": observed.page_id,
                    "expected_page_generation": expected.page_generation,
                    "observed_page_generation": observed.page_generation,
                    "expected_origin": expected.origin,
                    "observed_origin": observed.origin,
                    "expected_history_index": expected.browser_history.current_entry_index,
                    "observed_history_index": observed.browser_history.current_entry_index,
                })),
            ));
        }
        Ok(())
    }
}

fn planner_output_digest(planner_output: &PlannerOutput) -> Result<String, ToolError> {
    let encoded = serde_json::to_vec(planner_output).map_err(|error| {
        planning_error(
            "planning_snapshot_serialization_failed",
            "planner output could not be serialized for snapshot binding",
            Some(serde_json::json!({ "reason": error.to_string() })),
        )
    })?;
    Ok(format!("{:x}", Sha256::digest(encoded)))
}

fn planner_output_requires_snapshot(steps: &[PlannedStep]) -> bool {
    steps.iter().any(|step| {
        matches!(
            step.tool_name,
            ToolName::OpenUrl
                | ToolName::GoBack
                | ToolName::GoForward
                | ToolName::ReloadPage
                | ToolName::EvalJs
                | ToolName::ScrollPage
                | ToolName::SetBrowserVisibility
                | ToolName::ClickElement
                | ToolName::FocusElement
                | ToolName::TypeIntoElement
                | ToolName::SubmitActiveForm
                | ToolName::ReadRegion
                | ToolName::ReadNextRegion
                | ToolName::ReadPreviousRegion
                | ToolName::StopSpeaking
                | ToolName::StartListening
                | ToolName::StopListening
                | ToolName::SetTtsVoice
                | ToolName::SetPlaybackVolume
                | ToolName::SetPlaybackSpeed
                | ToolName::MergeOcrIntoPageModel
        )
    })
}

fn planning_snapshots_match(
    expected: &PlanningStateSnapshot,
    observed: &PlanningStateSnapshot,
) -> bool {
    expected.page_id == observed.page_id
        && expected.page_generation == observed.page_generation
        && expected.origin == observed.origin
        && expected.browser_history == observed.browser_history
        && expected.safety == observed.safety
        && expected.pending_confirmation_id == observed.pending_confirmation_id
}

fn planning_error(code: &str, message: &str, details: Option<serde_json::Value>) -> ToolError {
    ToolError {
        code: code.to_string(),
        message: message.to_string(),
        retryable: false,
        details,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::BrowserHistoryState;

    fn snapshot(generation: u64) -> PlanningStateSnapshot {
        PlanningStateSnapshot {
            page_id: Some(String::from("page-1")),
            page_generation: generation,
            origin: Some(String::from("https://example.com")),
            browser_history: BrowserHistoryState::default(),
            safety: PlannerSafetySettings {
                confirmation_confidence_threshold: 0.85,
                allow_click_without_confirmation: true,
                always_confirm_submit: true,
            },
            pending_confirmation_id: None,
            issued_at_ms: 10,
            expires_at_ms: 20,
        }
    }

    #[test]
    fn page_generation_is_part_of_the_planning_snapshot_contract() {
        assert!(planning_snapshots_match(&snapshot(4), &snapshot(4)));
        assert!(!planning_snapshots_match(&snapshot(4), &snapshot(5)));
    }
}
''',
)

write(
    "src-tauri/src/app_core/click_authorization.rs",
    r'''use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};
use url::Url;

use super::interaction_tools::resolve_clickable_element;
use crate::commands::{
    current_timestamp_ms, evaluate_action_policy, normalized_origin, ConfirmationRequirement,
    PlannedStep, PlannerOutput, PlannerSafetySettings, PlannerStatus, StepTransition, ToolError,
    ToolName, CLICK_AUTH_AMBIGUOUS_ARG, CLICK_AUTH_CONFIDENCE_ARG,
    CLICK_AUTH_DESTRUCTIVE_ARG, CLICK_AUTH_GENERATION_ARG, CLICK_AUTH_TOKEN_ARG,
    RUNTIME_FORM_DESTINATION_ARG, RUNTIME_FORM_FIELDS_ARG, RUNTIME_FORM_LABEL_ARG,
    RUNTIME_TARGET_LABEL_ARG,
};
use crate::page_model::{ElementRole, InteractiveElement};
use crate::state::ClickAuthorizationRecord;

const CLICK_AUTHORIZATION_TTL_MS: u64 = 30_000;
const MAX_CLICK_AUTHORIZATIONS: usize = 32;
static CLICK_AUTHORIZATION_COUNTER: AtomicU64 = AtomicU64::new(1);

impl super::AppCore {
    pub(crate) fn issue_find_element_click_authorization(
        &mut self,
        request_id: &str,
        element_id: &str,
        confidence_bps: u16,
    ) -> Result<String, ToolError> {
        self.mint_click_authorization(
            request_id,
            element_id,
            Some(confidence_bps),
            false,
        )
        .map(|record| record.token)
    }

    pub(crate) fn prepare_planner_output_for_execution(
        &mut self,
        planner_output: &PlannerOutput,
    ) -> Result<PlannerOutput, ToolError> {
        self.prune_click_authorizations();
        let mut prepared = planner_output.clone();
        let mut used_tokens = HashSet::new();

        for step in &mut prepared.steps {
            clear_runtime_annotations(step);
            match step.tool_name {
                ToolName::ClickElement => {
                    let element_id = required_string_argument(step, "element_id")?;
                    let provided_token = step
                        .arguments
                        .get(CLICK_AUTH_TOKEN_ARG)
                        .and_then(serde_json::Value::as_str)
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(ToOwned::to_owned);
                    let record = if let Some(token) = provided_token {
                        self.validate_stored_click_authorization(&token, &element_id, false)?
                    } else if let Some(existing) =
                        self.latest_click_authorization_for_element(&element_id)
                    {
                        self.mint_click_authorization(
                            &step.step_id,
                            &element_id,
                            existing.confidence_bps,
                            existing.ambiguous,
                        )?
                    } else {
                        self.mint_click_authorization(
                            &step.step_id,
                            &element_id,
                            None,
                            true,
                        )?
                    };
                    if !used_tokens.insert(record.token.clone()) {
                        return Err(click_error(
                            "duplicate_click_authorization",
                            "one click authorization token cannot authorize multiple planned clicks",
                            None,
                        ));
                    }
                    annotate_click_step(step, &record)?;
                }
                ToolName::TypeIntoElement => self.annotate_target_step(step)?,
                ToolName::SubmitActiveForm => self.annotate_submit_step(step)?,
                _ => {}
            }
        }

        let safety = PlannerSafetySettings::from(&self.config.safety);
        let decision = evaluate_action_policy(&prepared.steps, &safety);
        if prepared.status == PlannerStatus::Ready
            && decision.requirement == ConfirmationRequirement::ConfirmationRequired
            && decision
                .findings
                .iter()
                .all(|finding| finding.tool_name == ToolName::ClickElement)
        {
            insert_deterministic_click_confirmation_gate(&mut prepared);
        }

        Ok(prepared)
    }

    pub(crate) fn preflight_pending_click_authorizations(
        &mut self,
        steps: &[PlannedStep],
    ) -> Result<(), ToolError> {
        let mut seen = HashSet::new();
        for step in steps {
            if step.tool_name != ToolName::ClickElement {
                continue;
            }
            let token = required_string_argument(step, CLICK_AUTH_TOKEN_ARG)?;
            if !seen.insert(token.clone()) {
                return Err(click_error(
                    "duplicate_click_authorization",
                    "pending protected actions reused one click authorization token",
                    None,
                ));
            }
            let element_id = required_string_argument(step, "element_id")?;
            self.validate_stored_click_authorization(&token, &element_id, true)?;
        }
        Ok(())
    }

    pub(crate) fn preflight_planned_step_runtime(
        &mut self,
        step: &PlannedStep,
    ) -> Result<(), ToolError> {
        if step.tool_name != ToolName::ClickElement {
            return Ok(());
        }
        let token = required_string_argument(step, CLICK_AUTH_TOKEN_ARG)?;
        let element_id = required_string_argument(step, "element_id")?;
        self.validate_stored_click_authorization(&token, &element_id, true)?;
        self.state.click_authorizations.remove(&token);
        Ok(())
    }

    fn latest_click_authorization_for_element(
        &self,
        element_id: &str,
    ) -> Option<ClickAuthorizationRecord> {
        let now_ms = current_timestamp_ms();
        self.state
            .click_authorizations
            .values()
            .filter(|record| {
                record.element_id == element_id
                    && record.page_id == self.state.current_page_id.as_deref().unwrap_or_default()
                    && record.page_generation == self.state.page_generation
                    && record.expires_at_ms > now_ms
            })
            .max_by_key(|record| record.issued_at_ms)
            .cloned()
    }

    fn mint_click_authorization(
        &mut self,
        request_id: &str,
        element_id: &str,
        confidence_bps: Option<u16>,
        ambiguous: bool,
    ) -> Result<ClickAuthorizationRecord, ToolError> {
        let page_id = self.state.current_page_id.clone().ok_or_else(|| {
            click_error(
                "no_active_page",
                "click authorization requires an active page",
                None,
            )
        })?;
        let page = self.state.current_page.as_ref().ok_or_else(|| {
            click_error(
                "missing_page_model",
                "click authorization requires a current page model",
                Some(serde_json::json!({ "page_id": page_id })),
            )
        })?;
        let element = resolve_clickable_element(page, element_id)?.clone();
        let dom_locator = element
            .dom_locator
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .expect("resolve_clickable_element requires a locator")
            .to_string();
        let now_ms = current_timestamp_ms();
        let counter = CLICK_AUTHORIZATION_COUNTER.fetch_add(1, Ordering::Relaxed);
        let token = format!("click-auth-{request_id}-{now_ms}-{counter}");
        let record = ClickAuthorizationRecord {
            token: token.clone(),
            page_id,
            page_generation: self.state.page_generation,
            origin: normalized_origin(page.url.as_deref()),
            element_id: element.element_id.clone(),
            dom_locator,
            element_fingerprint: element_fingerprint(&element),
            confidence_bps,
            ambiguous,
            potentially_destructive: is_potentially_destructive_click(&element),
            issued_at_ms: now_ms,
            expires_at_ms: now_ms.saturating_add(CLICK_AUTHORIZATION_TTL_MS),
        };
        self.state.click_authorizations.insert(token, record.clone());
        self.prune_click_authorizations();
        Ok(record)
    }

    fn validate_stored_click_authorization(
        &mut self,
        token: &str,
        element_id: &str,
        validate_live_dom: bool,
    ) -> Result<ClickAuthorizationRecord, ToolError> {
        let record = self
            .state
            .click_authorizations
            .get(token)
            .cloned()
            .ok_or_else(|| {
                click_error(
                    "unknown_click_authorization",
                    "click authorization token is unknown, consumed, or expired",
                    None,
                )
            })?;
        let now_ms = current_timestamp_ms();
        if now_ms >= record.expires_at_ms {
            self.state.click_authorizations.remove(token);
            return Err(click_error(
                "click_authorization_expired",
                "click authorization expired before dispatch",
                Some(serde_json::json!({
                    "expired_at_ms": record.expires_at_ms,
                    "observed_at_ms": now_ms,
                })),
            ));
        }
        if record.element_id != element_id
            || self.state.current_page_id.as_deref() != Some(record.page_id.as_str())
            || self.state.page_generation != record.page_generation
            || normalized_origin(
                self.state
                    .current_page
                    .as_ref()
                    .and_then(|page| page.url.as_deref()),
            ) != record.origin
        {
            return Err(click_error(
                "stale_click_authorization",
                "click authorization no longer matches the active page generation and target",
                Some(serde_json::json!({
                    "expected_page_id": record.page_id,
                    "observed_page_id": self.state.current_page_id,
                    "expected_page_generation": record.page_generation,
                    "observed_page_generation": self.state.page_generation,
                    "expected_element_id": record.element_id,
                    "observed_element_id": element_id,
                })),
            ));
        }

        let page = self.state.current_page.as_ref().ok_or_else(|| {
            click_error(
                "missing_page_model",
                "click authorization cannot be checked without a current page model",
                None,
            )
        })?;
        let current_element = resolve_clickable_element(page, element_id)?;
        verify_element_matches_record(current_element, &record)?;

        if validate_live_dom {
            let live_page = self.browser.extract_page_model().map_err(|error| {
                click_error(
                    "click_live_revalidation_failed",
                    "live DOM could not be re-extracted before click dispatch",
                    Some(serde_json::json!({ "reason": error.to_string() })),
                )
            })?;
            let live_element = resolve_clickable_element(&live_page, element_id).map_err(|error| {
                click_error(
                    "click_target_changed",
                    "the authorized click target no longer resolves in the live DOM",
                    Some(serde_json::json!({
                        "element_id": element_id,
                        "reason_code": error.code,
                    })),
                )
            })?;
            verify_element_matches_record(live_element, &record)?;
        }

        Ok(record)
    }

    fn prune_click_authorizations(&mut self) {
        let now_ms = current_timestamp_ms();
        self.state
            .click_authorizations
            .retain(|_, record| record.expires_at_ms > now_ms);
        while self.state.click_authorizations.len() > MAX_CLICK_AUTHORIZATIONS {
            let oldest = self
                .state
                .click_authorizations
                .iter()
                .min_by_key(|(_, record)| record.issued_at_ms)
                .map(|(token, _)| token.clone());
            let Some(oldest) = oldest else { break };
            self.state.click_authorizations.remove(&oldest);
        }
    }

    fn annotate_target_step(&self, step: &mut PlannedStep) -> Result<(), ToolError> {
        let element_id = required_string_argument(step, "element_id")?;
        let page = self.state.current_page.as_ref().ok_or_else(|| {
            click_error(
                "missing_page_model",
                "target summary requires a current page model",
                None,
            )
        })?;
        let element = page
            .interactive_elements
            .iter()
            .find(|element| element.element_id == element_id)
            .ok_or_else(|| {
                click_error(
                    "unknown_element_id",
                    "target summary requires an element from the current page model",
                    Some(serde_json::json!({ "element_id": element_id })),
                )
            })?;
        insert_runtime_value(
            step,
            RUNTIME_TARGET_LABEL_ARG,
            serde_json::Value::String(safe_element_label(element)),
        )
    }

    fn annotate_submit_step(&self, step: &mut PlannedStep) -> Result<(), ToolError> {
        let Some(page) = self.state.current_page.as_ref() else {
            return Ok(());
        };
        let requested_form_id = step
            .arguments
            .get("form_element_id")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let form = requested_form_id
            .and_then(|form_id| {
                page.interactive_elements
                    .iter()
                    .find(|element| element.element_id == form_id && element.role == ElementRole::Form)
            })
            .or_else(|| {
                let mut forms = page
                    .interactive_elements
                    .iter()
                    .filter(|element| element.role == ElementRole::Form && element.visible);
                let first = forms.next()?;
                forms.next().is_none().then_some(first)
            });
        if let Some(form) = form {
            insert_runtime_value(
                step,
                RUNTIME_FORM_LABEL_ARG,
                serde_json::Value::String(safe_element_label(form)),
            )?;
        }
        if let Some(destination) = form_destination(page.url.as_deref(), form) {
            insert_runtime_value(
                step,
                RUNTIME_FORM_DESTINATION_ARG,
                serde_json::Value::String(destination),
            )?;
        }
        let fields = page
            .interactive_elements
            .iter()
            .filter(|element| {
                matches!(
                    element.role,
                    ElementRole::Input | ElementRole::TextArea | ElementRole::Select
                ) && !sensitive_field_label(element)
            })
            .map(safe_element_label)
            .filter(|label| !label.is_empty())
            .take(8)
            .map(serde_json::Value::String)
            .collect::<Vec<_>>();
        insert_runtime_value(
            step,
            RUNTIME_FORM_FIELDS_ARG,
            serde_json::Value::Array(fields),
        )
    }
}

fn annotate_click_step(
    step: &mut PlannedStep,
    record: &ClickAuthorizationRecord,
) -> Result<(), ToolError> {
    insert_runtime_value(
        step,
        CLICK_AUTH_TOKEN_ARG,
        serde_json::Value::String(record.token.clone()),
    )?;
    insert_runtime_value(
        step,
        CLICK_AUTH_CONFIDENCE_ARG,
        record
            .confidence_bps
            .map(|value| serde_json::Value::from(u64::from(value)))
            .unwrap_or(serde_json::Value::Null),
    )?;
    insert_runtime_value(
        step,
        CLICK_AUTH_AMBIGUOUS_ARG,
        serde_json::Value::Bool(record.ambiguous),
    )?;
    insert_runtime_value(
        step,
        CLICK_AUTH_DESTRUCTIVE_ARG,
        serde_json::Value::Bool(record.potentially_destructive),
    )?;
    insert_runtime_value(
        step,
        CLICK_AUTH_GENERATION_ARG,
        serde_json::Value::from(record.page_generation),
    )
}

fn clear_runtime_annotations(step: &mut PlannedStep) {
    let Some(arguments) = step.arguments.as_object_mut() else {
        return;
    };
    for key in [
        CLICK_AUTH_CONFIDENCE_ARG,
        CLICK_AUTH_AMBIGUOUS_ARG,
        CLICK_AUTH_DESTRUCTIVE_ARG,
        CLICK_AUTH_GENERATION_ARG,
        RUNTIME_TARGET_LABEL_ARG,
        RUNTIME_FORM_LABEL_ARG,
        RUNTIME_FORM_DESTINATION_ARG,
        RUNTIME_FORM_FIELDS_ARG,
    ] {
        arguments.remove(key);
    }
}

fn insert_runtime_value(
    step: &mut PlannedStep,
    name: &str,
    value: serde_json::Value,
) -> Result<(), ToolError> {
    let arguments = step.arguments.as_object_mut().ok_or_else(|| {
        click_error(
            "invalid_tool_arguments",
            "runtime authorization requires object-shaped tool arguments",
            Some(serde_json::json!({ "step_id": step.step_id })),
        )
    })?;
    arguments.insert(name.to_string(), value);
    Ok(())
}

fn required_string_argument(step: &PlannedStep, name: &str) -> Result<String, ToolError> {
    step.arguments
        .get(name)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            click_error(
                "missing_runtime_argument",
                &format!("planned step is missing required runtime argument '{name}'"),
                Some(serde_json::json!({ "step_id": step.step_id })),
            )
        })
}

fn verify_element_matches_record(
    element: &InteractiveElement,
    record: &ClickAuthorizationRecord,
) -> Result<(), ToolError> {
    let locator = element
        .dom_locator
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if locator != Some(record.dom_locator.as_str())
        || element_fingerprint(element) != record.element_fingerprint
        || !element.visible
        || !element.enabled
    {
        return Err(click_error(
            "click_target_changed",
            "the click target changed after authorization was issued",
            Some(serde_json::json!({ "element_id": record.element_id })),
        ));
    }
    Ok(())
}

fn element_fingerprint(element: &InteractiveElement) -> String {
    let value = serde_json::json!({
        "element_id": element.element_id,
        "dom_locator": element.dom_locator,
        "role": element.role,
        "tag_name": element.tag_name,
        "text": element.text,
        "accessible_name": element.accessible_name,
        "placeholder": element.placeholder,
        "href": element.href,
        "visible": element.visible,
        "enabled": element.enabled,
    });
    let encoded = serde_json::to_vec(&value).expect("element fingerprint should serialize");
    format!("{:x}", Sha256::digest(encoded))
}

fn safe_element_label(element: &InteractiveElement) -> String {
    element
        .accessible_name
        .as_deref()
        .or(element.text.as_deref())
        .or(element.placeholder.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&element.element_id)
        .chars()
        .filter(|character| !character.is_control())
        .take(80)
        .collect()
}

fn is_potentially_destructive_click(element: &InteractiveElement) -> bool {
    let text = format!(
        "{} {} {}",
        element.accessible_name.as_deref().unwrap_or_default(),
        element.text.as_deref().unwrap_or_default(),
        element.href.as_deref().unwrap_or_default(),
    )
    .to_ascii_lowercase();
    [
        "delete",
        "remove",
        "purchase",
        "buy now",
        "pay",
        "place order",
        "transfer",
        "send money",
        "confirm order",
        "close account",
        "sign out",
        "log out",
        "publish",
        "post",
    ]
    .iter()
    .any(|keyword| text.contains(keyword))
}

fn sensitive_field_label(element: &InteractiveElement) -> bool {
    let label = format!(
        "{} {} {} {}",
        element.element_id,
        element.accessible_name.as_deref().unwrap_or_default(),
        element.placeholder.as_deref().unwrap_or_default(),
        element.attributes.get("type").map(String::as_str).unwrap_or_default(),
    )
    .to_ascii_lowercase();
    [
        "password",
        "passwd",
        "token",
        "secret",
        "one-time",
        "otp",
        "credit card",
        "card number",
        "cvv",
        "cvc",
        "ssn",
        "social security",
        "security answer",
    ]
    .iter()
    .any(|keyword| label.contains(keyword))
}

fn form_destination(
    page_url: Option<&str>,
    form: Option<&InteractiveElement>,
) -> Option<String> {
    let page_url = page_url?;
    let action = form.and_then(|form| form.attributes.get("action"));
    let destination = match action.map(String::as_str).map(str::trim) {
        Some(action) if !action.is_empty() => Url::parse(page_url).ok()?.join(action).ok()?,
        _ => Url::parse(page_url).ok()?,
    };
    normalized_origin(Some(destination.as_str()))
}

fn insert_deterministic_click_confirmation_gate(planner_output: &mut PlannerOutput) {
    let mut step_id = String::from("runtime-confirm-click");
    let existing = planner_output
        .steps
        .iter()
        .map(|step| step.step_id.as_str())
        .collect::<HashSet<_>>();
    let mut suffix = 1_u32;
    while existing.contains(step_id.as_str()) {
        step_id = format!("runtime-confirm-click-{suffix}");
        suffix += 1;
    }
    planner_output.steps.insert(
        0,
        PlannedStep {
            step_id,
            tool_name: ToolName::ConfirmAction,
            arguments: serde_json::json!({
                "request_id": "runtime-click-confirmation",
                "timeout_ms": 120000,
                "prompt_text": "Runtime-generated click confirmation",
                "reason": "deterministic click policy requires confirmation"
            }),
            purpose: String::from("request deterministic click confirmation"),
            on_success: StepTransition::RequestConfirmation,
            on_failure: StepTransition::Replan,
        },
    );
    planner_output.status = PlannerStatus::NeedsConfirmation;
    planner_output.requires_confirmation = true;
    planner_output.confirmation_reason = Some(String::from(
        "Deterministic runtime click policy requires confirmation.",
    ));
    planner_output.user_message = Some(String::from(
        "Please confirm the selected page interaction.",
    ));
}

fn click_error(code: &str, message: &str, details: Option<serde_json::Value>) -> ToolError {
    ToolError {
        code: code.to_string(),
        message: message.to_string(),
        retryable: false,
        details,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn element(label: &str) -> InteractiveElement {
        InteractiveElement {
            element_id: String::from("button-1"),
            dom_locator: Some(String::from("#button-1")),
            role: ElementRole::Button,
            tag_name: String::from("button"),
            text: Some(label.to_string()),
            accessible_name: Some(label.to_string()),
            placeholder: None,
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: BTreeMap::new(),
        }
    }

    #[test]
    fn element_fingerprint_changes_with_locator_or_label() {
        let first = element("Continue");
        let mut changed = first.clone();
        changed.dom_locator = Some(String::from("#replacement"));
        assert_ne!(element_fingerprint(&first), element_fingerprint(&changed));
        changed = first.clone();
        changed.accessible_name = Some(String::from("Delete account"));
        assert_ne!(element_fingerprint(&first), element_fingerprint(&changed));
    }

    #[test]
    fn destructive_click_labels_are_detected_deterministically() {
        assert!(!is_potentially_destructive_click(&element("Continue")));
        assert!(is_potentially_destructive_click(&element("Delete account")));
    }
}
''',
)

# Register the modules and make the click resolver available to authorization.
replace_once(
    "src-tauri/src/app_core/mod.rs",
    "mod command_dispatch;\nmod confirmation_workflow;",
    "mod click_authorization;\nmod command_dispatch;\nmod confirmation_workflow;",
)
replace_once(
    "src-tauri/src/app_core/mod.rs",
    "mod replanning;\nmod replanning_orchestrator;",
    "mod planning_snapshot;\nmod replanning;\nmod replanning_orchestrator;",
)
replace_once(
    "src-tauri/src/app_core/interaction_tools/mod.rs",
    "#[cfg(test)]\npub(crate) use click_focus::resolve_clickable_element;",
    "pub(crate) use click_focus::resolve_clickable_element;",
)

# FindElement issues a short-lived authorization only for a unique, sufficiently
# confident deterministic resolution. The opaque token is carried in observations
# for a bounded replan or a direct caller.
replace_once(
    "src-tauri/src/app_core/interaction_tools/element_queries.rs",
    "        let mut observations = vec![format!(\n            \"Searched {} interactive element(s) from the current runtime page state.\",\n            elements.len()\n        )];",
    "        let click_authorization_token = if let Some(element_id) = chosen_element_id.as_deref() {\n            let confidence_bps = ranked_candidates\n                .first()\n                .map(|candidate| candidate.confidence_bps)\n                .ok_or_else(|| ToolError {\n                    code: String::from(\"missing_click_confidence\"),\n                    message: String::from(\n                        \"deterministic element resolution did not retain its confidence score\",\n                    ),\n                    retryable: false,\n                    details: None,\n                });\n            match confidence_bps.and_then(|confidence_bps| {\n                self.issue_find_element_click_authorization(\n                    &input.request_id,\n                    element_id,\n                    confidence_bps,\n                )\n            }) {\n                Ok(token) => Some(token),\n                Err(error) => {\n                    return ToolResult::failure(\n                        ToolName::FindElement,\n                        input.request_id,\n                        error,\n                        vec![String::from(\n                            \"Element resolution succeeded, but runtime click authorization could not be issued.\",\n                        )],\n                    )\n                }\n            }\n        } else {\n            None\n        };\n\n        let mut observations = vec![format!(\n            \"Searched {} interactive element(s) from the current runtime page state.\",\n            elements.len()\n        )];",
)
replace_once(
    "src-tauri/src/app_core/interaction_tools/element_queries.rs",
    "        } else {\n            observations.push(String::from(\n                \"A single strongest candidate was identified from the filtered interactive elements.\",\n            ));\n        }\n\n        ToolResult::success(",
    "        } else {\n            observations.push(String::from(\n                \"A single strongest candidate was identified from the filtered interactive elements.\",\n            ));\n        }\n        if let Some(token) = click_authorization_token.as_deref() {\n            observations.push(format!(\n                \"Opaque click authorization issued: {token}\"\n            ));\n        }\n\n        ToolResult::success(",
)

# AppCore executor wrapper: consume the planning snapshot, inject runtime click
# proof/confirmation metadata, and use the user's actual safety settings.
replace_once(
    "src-tauri/src/app_core/command_dispatch.rs",
    "    build_planner_skill_selection, execute_planner_output, planner_available_tools,",
    "    build_planner_skill_selection, execute_planner_output_with_runtime_safety,\n    planner_available_tools,",
)
replace_once(
    "src-tauri/src/app_core/command_dispatch.rs",
    "    pub fn execute_planner_output(\n        &mut self,\n        request_id: String,\n        planner_output: &PlannerOutput,\n    ) -> ExecutionOutcome {\n        let outcome = execute_planner_output(self, request_id, planner_output);\n        self.state.apply_execution_outcome(&outcome);\n        outcome\n    }",
    "    pub fn execute_planner_output(\n        &mut self,\n        request_id: String,\n        planner_output: &PlannerOutput,\n    ) -> ExecutionOutcome {\n        if let Err(error) = self.validate_and_consume_planning_snapshot(planner_output) {\n            let outcome = planner_execution_abort(error);\n            self.state.apply_execution_outcome(&outcome);\n            return outcome;\n        }\n        let prepared = match self.prepare_planner_output_for_execution(planner_output) {\n            Ok(prepared) => prepared,\n            Err(error) => {\n                let outcome = planner_execution_abort(error);\n                self.state.apply_execution_outcome(&outcome);\n                return outcome;\n            }\n        };\n        let safety = PlannerSafetySettings::from(&self.config.safety);\n        let outcome = execute_planner_output_with_runtime_safety(\n            self,\n            request_id,\n            &prepared,\n            &safety,\n        );\n        self.state.apply_execution_outcome(&outcome);\n        outcome\n    }",
)
write_path = "src-tauri/src/app_core/command_dispatch.rs"
content = read(write_path)
content += """

fn planner_execution_abort(error: ToolError) -> ExecutionOutcome {
    ExecutionOutcome::Aborted {
        trace: crate::commands::ExecutionTrace {
            executed_step_ids: Vec::new(),
            tool_results: Vec::new(),
        },
        error,
    }
}
"""
write(write_path, content)

# Register each resolved output against the state snapshot captured while the
# AppCore lock was held. This covers direct and remote resolve_command paths and
# every replan cycle.
replace_once(
    "src-tauri/src/app_core/replanning_orchestrator.rs",
    "        let resolution = {\n            let mut guard = lock_app_core(self.core)?;\n            guard.build_planner_resolution(request_id, transcript, recent_tool_results)?\n        };\n\n        match resolution {",
    "        let (resolution, planning_snapshot) = {\n            let mut guard = lock_app_core(self.core)?;\n            let planning_snapshot = guard.capture_planning_state_snapshot();\n            let resolution =\n                guard.build_planner_resolution(request_id, transcript, recent_tool_results)?;\n            (resolution, planning_snapshot)\n        };\n\n        let planner_output = match resolution {",
)
replace_once(
    "src-tauri/src/app_core/replanning_orchestrator.rs",
    "            PlannerResolution::Direct(planner_output) => Ok(planner_output),",
    "            PlannerResolution::Direct(planner_output) => planner_output,",
)
replace_once(
    "src-tauri/src/app_core/replanning_orchestrator.rs",
    "                validate_planner_output_with_safety(\n                    &planner_output,\n                    &available_tools,\n                    &active_skill_names,\n                    &planner_input.safety,\n                )?;\n                Ok(planner_output)\n            }\n        }",
    "                validate_planner_output_with_safety(\n                    &planner_output,\n                    &available_tools,\n                    &active_skill_names,\n                    &planner_input.safety,\n                )?;\n                planner_output\n            }\n        };\n\n        {\n            let mut guard = lock_app_core(self.core)?;\n            guard.register_planning_snapshot(&planner_output, planning_snapshot)?;\n        }\n        Ok(planner_output)",
)

# Confirmation page identity includes page generation. Revalidate every pending
# click before consuming the single-use confirmation state.
replace_once(
    "src-tauri/src/app_core/tool_executor.rs",
    "            self.state.current_page_id.clone(),",
    "            self.state.confirmation_page_identity(),",
)
replace_once(
    "src-tauri/src/app_core/tool_executor.rs",
    "    DeterministicToolExecutor, EvalJsData, EvalJsInput, ExtractPageModelData,",
    "    DeterministicToolExecutor, EvalJsData, EvalJsInput, ExtractPageModelData,",
)
replace_once(
    "src-tauri/src/app_core/tool_executor.rs",
    "    StopSpeakingInput, SubmitActiveFormData, SubmitActiveFormInput, ToolResult,",
    "    StopSpeakingInput, SubmitActiveFormData, SubmitActiveFormInput, ToolError, ToolResult,\n    PlannedStep,",
)
replace_once(
    "src-tauri/src/app_core/tool_executor.rs",
    "impl DeterministicToolExecutor for super::AppCore {\n    fn confirmation_runtime_context",
    "impl DeterministicToolExecutor for super::AppCore {\n    fn preflight_planned_step(&mut self, step: &PlannedStep) -> Result<(), ToolError> {\n        self.preflight_planned_step_runtime(step)\n    }\n\n    fn confirmation_runtime_context",
)
replace_once(
    "src-tauri/src/app_core/confirmation_workflow.rs",
    "            self.state.current_page_id.clone(),",
    "            self.state.confirmation_page_identity(),",
)
replace_once(
    "src-tauri/src/app_core/confirmation_workflow.rs",
    "        // Matching challenges are consumed before dispatch so duplicate responses,\n        // re-entrant UI events, and retries cannot execute the protected action twice.\n        self.state.clear_pending_execution();",
    "        if confirmed {\n            if let Err(error) =\n                self.preflight_pending_click_authorizations(&pending_plan_execution.queued_steps)\n            {\n                return ExecutionOutcome::Aborted {\n                    trace: ExecutionTrace {\n                        executed_step_ids: Vec::new(),\n                        tool_results: Vec::new(),\n                    },\n                    error,\n                };\n            }\n        }\n\n        // Matching challenges are consumed before dispatch so duplicate responses,\n        // re-entrant UI events, and retries cannot execute the protected action twice.\n        self.state.clear_pending_execution();",
)

# Deterministic confirmation copy uses runtime-resolved safe target/form metadata.
replace_once(
    "src-tauri/src/commands/confirmation_manifest.rs",
    "        ToolName::SubmitActiveForm => String::from(\"Submit the active form.\"),\n        ToolName::ClickElement => string_argument(step, \"element_id\")",
    "        ToolName::SubmitActiveForm => {\n            let form = string_argument(step, RUNTIME_FORM_LABEL_ARG)\n                .map(safe_label)\n                .unwrap_or_else(|| String::from(\"the active form\"));\n            let destination = string_argument(step, RUNTIME_FORM_DESTINATION_ARG)\n                .map(|origin| format!(\" to {origin}\"))\n                .unwrap_or_default();\n            let fields = string_array_argument(step, RUNTIME_FORM_FIELDS_ARG);\n            if fields.is_empty() {\n                format!(\"Submit {form}{destination}.\")\n            } else {\n                format!(\n                    \"Submit {form}{destination} with fields: {}.\",\n                    fields.join(\", \")\n                )\n            }\n        }\n        ToolName::ClickElement => string_argument(step, RUNTIME_TARGET_LABEL_ARG)\n            .or_else(|| string_argument(step, \"element_id\"))",
)
replace_once(
    "src-tauri/src/commands/confirmation_manifest.rs",
    "            let target = string_argument(step, \"element_id\")\n                .map(safe_label)",
    "            let target = string_argument(step, RUNTIME_TARGET_LABEL_ARG)\n                .or_else(|| string_argument(step, \"element_id\"))\n                .map(safe_label)",
)
replace_once(
    "src-tauri/src/commands/confirmation_manifest.rs",
    "fn safe_label(value: &str) -> String {",
    "fn string_array_argument(step: &PlannedStep, name: &str) -> Vec<String> {\n    step.arguments\n        .get(name)\n        .and_then(serde_json::Value::as_array)\n        .into_iter()\n        .flatten()\n        .filter_map(serde_json::Value::as_str)\n        .map(safe_label)\n        .filter(|value| !value.is_empty())\n        .collect()\n}\n\nfn safe_label(value: &str) -> String {",
)
replace_once(
    "src-tauri/src/commands/confirmation_manifest.rs",
    "use super::{PendingPlanExecutionState, PlannedStep, ToolError, ToolName};",
    "use super::{\n    PendingPlanExecutionState, PlannedStep, ToolError, ToolName,\n    RUNTIME_FORM_DESTINATION_ARG, RUNTIME_FORM_FIELDS_ARG, RUNTIME_FORM_LABEL_ARG,\n    RUNTIME_TARGET_LABEL_ARG,\n};",
)

# Page generation advances whenever the page model is replaced or a browser
# action can make the retained model stale.
replace_once(
    "src-tauri/src/app_core/extraction_tools/page_extraction.rs",
    "            self.state.current_page = Some(extracted_page_model.clone());",
    "            self.state\n                .replace_current_page_model(extracted_page_model.clone());",
)
replace_once(
    "src-tauri/src/app_core/extraction_tools/ocr_tools.rs",
    "        let mut observations = vec![String::from(\n            \"Merged OCR text into the active runtime page model.\",\n        )];",
    "        self.state.mark_page_model_changed();\n\n        let mut observations = vec![String::from(\n            \"Merged OCR text into the active runtime page model.\",\n        )];",
)
for needle in [
    "            refresh_current_page_after_navigation(\n                &mut self.state.current_page,\n                browser_navigation.url.clone(),\n                browser_navigation.title.clone(),\n            );\n            clear_navigation_follow_up_state",
    "            refresh_current_page_after_navigation(\n                &mut self.state.current_page,\n                browser_navigation.url.clone(),\n                browser_navigation.title.clone(),\n            );\n            clear_navigation_follow_up_state",
]:
    # Two identical occurrences (back and forward), replace one at a time.
    content = read("src-tauri/src/app_core/navigation_tools.rs")
    if needle not in content:
        raise SystemExit("navigation_tools.rs: expected back/forward refresh occurrence")
    content = content.replace(
        needle,
        needle.replace("            clear_navigation_follow_up_state", "            self.state.mark_page_model_changed();\n            clear_navigation_follow_up_state"),
        1,
    )
    write("src-tauri/src/app_core/navigation_tools.rs", content)
replace_once(
    "src-tauri/src/app_core/navigation_tools.rs",
    "        refresh_current_page_after_navigation(\n            &mut self.state.current_page,\n            Some(browser_page.url.clone()),\n            browser_page.title.clone(),\n        );\n        clear_navigation_follow_up_state",
    "        refresh_current_page_after_navigation(\n            &mut self.state.current_page,\n            Some(browser_page.url.clone()),\n            browser_page.title.clone(),\n        );\n        self.state.mark_page_model_changed();\n        clear_navigation_follow_up_state",
)
replace_once(
    "src-tauri/src/app_core/interaction_tools/click_focus.rs",
    "        self.state.browser_history = browser_click.history.clone();",
    "        if !browser_click.page_changed {\n            self.state.mark_page_model_changed();\n        }\n        self.state.browser_history = browser_click.history.clone();",
)
replace_once(
    "src-tauri/src/app_core/interaction_tools/text_entry.rs",
    "        self.state.browser_history = browser_type.history.clone();",
    "        if !browser_type.page_changed {\n            self.state.mark_page_model_changed();\n        }\n        self.state.browser_history = browser_type.history.clone();",
)
replace_once(
    "src-tauri/src/app_core/interaction_tools/text_entry.rs",
    "        self.state.browser_history = browser_submit.history.clone();",
    "        if !browser_submit.page_changed {\n            self.state.mark_page_model_changed();\n        }\n        self.state.browser_history = browser_submit.history.clone();",
)

# State tests prove generation and confirmation identity change on same-page model
# replacement, closing the same-page DOM replacement hole.
state_path = "src-tauri/src/state.rs"
state_content = read(state_path)
insert_before = "    #[test]\n    fn stop_speaking_clears_runtime_speaking_state()"
state_test = '''    #[test]
    fn page_generation_advances_for_navigation_and_model_replacement() {
        let mut state = AppState::default();
        state.record_navigation(
            String::from("page-1"),
            String::from("https://example.com"),
        );
        let first_identity = state.confirmation_page_identity();
        assert_eq!(state.page_generation, 1);

        state.replace_current_page_model(PageModel {
            title: Some(String::from("Replacement")),
            url: Some(String::from("https://example.com")),
            regions: Vec::new(),
            interactive_elements: Vec::new(),
        });
        assert_eq!(state.page_generation, 2);
        assert_ne!(state.confirmation_page_identity(), first_identity);
    }

'''
if state_content.count(insert_before) != 1:
    raise SystemExit("state.rs: test insertion anchor mismatch")
write(state_path, state_content.replace(insert_before, state_test + insert_before, 1))

# Security policy regressions: runtime-authorized high-confidence ordinary clicks
# may honor the user setting; missing, ambiguous, destructive, and low-confidence
# grounding remains protected.
security_path = "src-tauri/src/commands/tests/security_policy.rs"
security_content = read(security_path)
old_test = '''#[test]
fn click_setting_cannot_bypass_missing_grounding_authorization() {
    let plan = output(
        PlannerStatus::Ready,
        IntentName::ClickElement,
        vec![click_step()],
    );
    let error =
        validate_planner_output_with_safety(&plan, &planner_available_tools(), &[], &safety(true))
            .expect_err("element id alone is not deterministic click authorization");

    assert_eq!(error.code, "confirmation_required_by_runtime_policy");
}
'''
new_test = '''#[test]
fn click_without_runtime_grounding_is_deferred_to_the_strict_executor() {
    let plan = output(
        PlannerStatus::Ready,
        IntentName::ClickElement,
        vec![click_step()],
    );
    validate_planner_output_with_safety(&plan, &planner_available_tools(), &[], &safety(true))
        .expect("click-only policy is finalized against runtime authorization under AppCore");

    let outcome = execute_planner_output_with_runner(String::from("req-security"), &plan, |_| {
        panic!("unprepared click must not reach the runner")
    });
    let ExecutionOutcome::Aborted { error, .. } = outcome else {
        panic!("strict generic executor must reject an unprepared click");
    };
    assert_eq!(error.code, "unconfirmed_side_effect_at_execution");
}

#[test]
fn high_confidence_runtime_authorization_can_honor_click_setting() {
    let mut step = click_step();
    let arguments = step.arguments.as_object_mut().unwrap();
    arguments.insert(
        CLICK_AUTH_TOKEN_ARG.to_string(),
        serde_json::json!("opaque-runtime-token"),
    );
    arguments.insert(CLICK_AUTH_CONFIDENCE_ARG.to_string(), serde_json::json!(9500));
    arguments.insert(CLICK_AUTH_AMBIGUOUS_ARG.to_string(), serde_json::json!(false));
    arguments.insert(CLICK_AUTH_DESTRUCTIVE_ARG.to_string(), serde_json::json!(false));

    let decision = evaluate_action_policy(&[step], &safety(true));
    assert_eq!(decision.requirement, ConfirmationRequirement::NoConfirmation);
    assert_eq!(
        decision.findings,
        Vec::<ActionPolicyFinding>::new(),
        "authorized ordinary click should not create a protected finding"
    );
}

#[test]
fn ambiguous_or_destructive_runtime_clicks_still_require_confirmation() {
    for (ambiguous, destructive) in [(true, false), (false, true)] {
        let mut step = click_step();
        let arguments = step.arguments.as_object_mut().unwrap();
        arguments.insert(
            CLICK_AUTH_TOKEN_ARG.to_string(),
            serde_json::json!("opaque-runtime-token"),
        );
        arguments.insert(CLICK_AUTH_CONFIDENCE_ARG.to_string(), serde_json::json!(9500));
        arguments.insert(CLICK_AUTH_AMBIGUOUS_ARG.to_string(), serde_json::json!(ambiguous));
        arguments.insert(
            CLICK_AUTH_DESTRUCTIVE_ARG.to_string(),
            serde_json::json!(destructive),
        );

        let decision = evaluate_action_policy(&[step], &safety(true));
        assert_eq!(
            decision.requirement,
            ConfirmationRequirement::ConfirmationRequired
        );
    }
}
'''
if security_content.count(old_test) != 1:
    raise SystemExit("security_policy.rs: old click test mismatch")
write(security_path, security_content.replace(old_test, new_test, 1))

# Implementation report: remove obsolete unmerged-PR language now that BBCR-004
# is on master; Batch 5 evidence is appended after the worker passes all gates.
report_path = "docs/BLIND_BROWSER_COMPREHENSIVE_CODE_REVIEW_FIX_IMPLEMENTATION_REPORT_2026-08-01.md"
report = read(report_path)
report = report.replace(
    "**Draft PR:** `#4`<br>\n**Result:** implementation and TODO closure complete; final exact-head CI and the human merge decision are the remaining PR gates",
    "**Merged PR:** `#4`<br>\n**Master squash commit:** `30a3d0b2cb7d24cddba35304fff3051062815e81`<br>\n**Result:** implementation, TODO closure, exact-head validation, merge, and stale-branch cleanup complete",
)
report = report.replace(
    "- The final TODO-closure head must pass the permanent repository CI workflow; its run and job are recorded in PR #4 and issue #5.\n- PR #4 remains draft and unmerged for the human merge decision.\n- Credential-origin binding does not replace the still-open remote-data consent and high-risk-origin controls in BBCR-003.",
    "- PR #4 was squash-merged to `master` as `30a3d0b2cb7d24cddba35304fff3051062815e81`.\n- All non-master working branches were deleted after merge.\n- Credential-origin binding does not replace the still-open remote-data consent and high-risk-origin controls in BBCR-003.",
)
report = report.replace(
    "- **BBCR-004:** Implementation and TODO closure complete on draft PR #4; final exact-head CI is recorded externally, and the human merge decision remains open.",
    "- **BBCR-004:** Complete, validated, merged to `master`, and branch cleanup complete.",
)
write(report_path, report)

print("BBCR-005 direct-master transformations applied")
