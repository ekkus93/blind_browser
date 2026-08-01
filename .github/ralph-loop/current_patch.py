from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, content: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8")


def replace_once(content: str, old: str, new: str, path: str) -> str:
    count = content.count(old)
    if count != 1:
        raise RuntimeError(f"{path}: expected one occurrence, found {count}: {old[:80]!r}")
    return content.replace(old, new, 1)


action_policy = r'''use super::*;

/// Deterministic classification of every planner-visible tool. This match is
/// intentionally exhaustive: adding a new `ToolName` fails compilation until
/// its security class and minimum confirmation requirement are chosen.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ActionClass {
    ReadOnly,
    ReversibleLocalStateChange,
    BrowserNavigation,
    PageInteraction,
    DataEntry,
    FormSubmission,
    ArbitraryScriptExecution,
    CredentialOperation,
    ModelDownload,
    OtherSideEffect,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ConfirmationRequirement {
    NoConfirmation,
    ConfirmationRequired,
    Prohibited,
}

impl ConfirmationRequirement {
    fn severity(self) -> u8 {
        match self {
            Self::NoConfirmation => 0,
            Self::ConfirmationRequired => 1,
            Self::Prohibited => 2,
        }
    }

    fn strongest(self, other: Self) -> Self {
        if other.severity() > self.severity() {
            other
        } else {
            self
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ActionPolicyReasonCode {
    ToolClassMinimum,
    SubmitRequiresConfirmation,
    TextEntrySubmitsForm,
    ClickRequiresConfirmationBySetting,
    ClickGroundingUnavailable,
    EvalJsProhibited,
    MalformedProtectedArguments,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ToolPolicy {
    pub class: ActionClass,
    pub minimum_confirmation: ConfirmationRequirement,
    pub reason_code: ActionPolicyReasonCode,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ActionPolicyFinding {
    pub step_id: String,
    pub tool_name: ToolName,
    pub class: ActionClass,
    pub requirement: ConfirmationRequirement,
    pub reason_code: ActionPolicyReasonCode,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ActionPolicyDecision {
    pub requirement: ConfirmationRequirement,
    pub findings: Vec<ActionPolicyFinding>,
}

pub fn tool_policy(tool_name: &ToolName) -> ToolPolicy {
    let (class, minimum_confirmation, reason_code) = match tool_name {
        ToolName::OpenUrl
        | ToolName::GoBack
        | ToolName::GoForward
        | ToolName::ReloadPage => (
            ActionClass::BrowserNavigation,
            ConfirmationRequirement::NoConfirmation,
            ActionPolicyReasonCode::ToolClassMinimum,
        ),
        ToolName::GetHtml
        | ToolName::GetPageSnapshot
        | ToolName::ExtractPageModel
        | ToolName::ListInteractiveElements
        | ToolName::FindElement
        | ToolName::RunOcr
        | ToolName::GetAgentState
        | ToolName::GetRuntimeStatus
        | ToolName::ReportResult => (
            ActionClass::ReadOnly,
            ConfirmationRequirement::NoConfirmation,
            ActionPolicyReasonCode::ToolClassMinimum,
        ),
        ToolName::EvalJs => (
            ActionClass::ArbitraryScriptExecution,
            ConfirmationRequirement::Prohibited,
            ActionPolicyReasonCode::EvalJsProhibited,
        ),
        ToolName::ScrollPage | ToolName::FocusElement => (
            ActionClass::PageInteraction,
            ConfirmationRequirement::NoConfirmation,
            ActionPolicyReasonCode::ToolClassMinimum,
        ),
        ToolName::ClickElement => (
            ActionClass::PageInteraction,
            ConfirmationRequirement::ConfirmationRequired,
            ActionPolicyReasonCode::ClickGroundingUnavailable,
        ),
        ToolName::TypeIntoElement => (
            ActionClass::DataEntry,
            ConfirmationRequirement::NoConfirmation,
            ActionPolicyReasonCode::ToolClassMinimum,
        ),
        ToolName::SubmitActiveForm => (
            ActionClass::FormSubmission,
            ConfirmationRequirement::ConfirmationRequired,
            ActionPolicyReasonCode::SubmitRequiresConfirmation,
        ),
        ToolName::CaptureScreenshot
        | ToolName::SetBrowserVisibility
        | ToolName::ReadRegion
        | ToolName::ReadNextRegion
        | ToolName::ReadPreviousRegion
        | ToolName::StopSpeaking
        | ToolName::StartListening
        | ToolName::StopListening
        | ToolName::TranscribeCommand
        | ToolName::SetTtsVoice
        | ToolName::SetPlaybackVolume
        | ToolName::SetPlaybackSpeed
        | ToolName::MergeOcrIntoPageModel
        | ToolName::ConfirmAction => (
            ActionClass::ReversibleLocalStateChange,
            ConfirmationRequirement::NoConfirmation,
            ActionPolicyReasonCode::ToolClassMinimum,
        ),
    };

    ToolPolicy {
        class,
        minimum_confirmation,
        reason_code,
    }
}

pub fn evaluate_action_policy(
    steps: &[PlannedStep],
    safety: &PlannerSafetySettings,
) -> ActionPolicyDecision {
    let mut requirement = ConfirmationRequirement::NoConfirmation;
    let mut findings = Vec::new();

    for step in steps {
        let policy = tool_policy(&step.tool_name);
        let mut step_requirement = policy.minimum_confirmation;
        let mut reason_code = policy.reason_code;

        match step.tool_name {
            ToolName::ClickElement => {
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
            ToolName::SubmitActiveForm => {
                // Form submission is a runtime minimum. The legacy setting may make
                // the prompt stricter, but it can never weaken this invariant.
                let _legacy_setting = safety.always_confirm_submit;
                step_requirement = ConfirmationRequirement::ConfirmationRequired;
                reason_code = ActionPolicyReasonCode::SubmitRequiresConfirmation;
            }
            ToolName::TypeIntoElement => {
                match serde_json::from_value::<TypeIntoElementInput>(step.arguments.clone()) {
                    Ok(input) if input.submit_mode.submits_after_entry() => {
                        step_requirement = ConfirmationRequirement::ConfirmationRequired;
                        reason_code = ActionPolicyReasonCode::TextEntrySubmitsForm;
                    }
                    Ok(_) => {}
                    Err(_) => {
                        // Argument validation normally rejects this first. Executor
                        // defense in depth still treats malformed data-entry calls as
                        // protected instead of assuming they are harmless.
                        step_requirement = ConfirmationRequirement::ConfirmationRequired;
                        reason_code = ActionPolicyReasonCode::MalformedProtectedArguments;
                    }
                }
            }
            ToolName::EvalJs => {
                step_requirement = ConfirmationRequirement::Prohibited;
                reason_code = ActionPolicyReasonCode::EvalJsProhibited;
            }
            _ => {}
        }

        requirement = requirement.strongest(step_requirement);
        if step_requirement != ConfirmationRequirement::NoConfirmation {
            findings.push(ActionPolicyFinding {
                step_id: step.step_id.clone(),
                tool_name: step.tool_name.clone(),
                class: policy.class,
                requirement: step_requirement,
                reason_code,
            });
        }
    }

    ActionPolicyDecision {
        requirement,
        findings,
    }
}
'''
write("src-tauri/src/commands/action_policy.rs", action_policy)

path = "src-tauri/src/commands/mod.rs"
text = read(path)
text = replace_once(text, "mod contracts;\n", "mod action_policy;\nmod contracts;\n", path)
text = replace_once(text, "pub use contracts::*;\n", "pub use action_policy::*;\npub use contracts::*;\n", path)
write(path, text)

path = "src-tauri/src/commands/validators/mod.rs"
text = read(path)
anchor = "fn validate_submit_confirmation_policy(planner_output: &PlannerOutput) -> Result<(), ToolError> {\n"
addition = r'''pub fn validate_planner_output_with_safety(
    planner_output: &PlannerOutput,
    available_tools: &[AvailableTool],
    active_skill_names: &[String],
    safety: &PlannerSafetySettings,
) -> Result<(), ToolError> {
    // Preserve all schema, transition, availability, and legacy metadata checks,
    // then apply the authoritative runtime policy derived from actual tools.
    validate_planner_output(planner_output, available_tools, active_skill_names)?;
    validate_protected_intent_consistency(planner_output)?;

    let decision = evaluate_action_policy(&planner_output.steps, safety);
    match decision.requirement {
        ConfirmationRequirement::Prohibited => Err(ToolError {
            code: String::from("prohibited_planner_action"),
            message: String::from(
                "planner output contains an action prohibited by deterministic runtime policy",
            ),
            retryable: false,
            details: serde_json::to_value(&decision).ok(),
        }),
        ConfirmationRequirement::ConfirmationRequired => {
            validate_runtime_confirmation_gate(planner_output, &decision)
        }
        ConfirmationRequirement::NoConfirmation => {
            if planner_output.status == PlannerStatus::NeedsConfirmation {
                return Err(invalid_planner_output(
                    "planner requested confirmation for a plan with no runtime-protected action",
                    serde_json::to_value(&decision).ok(),
                ));
            }
            Ok(())
        }
    }
}

fn validate_runtime_confirmation_gate(
    planner_output: &PlannerOutput,
    decision: &ActionPolicyDecision,
) -> Result<(), ToolError> {
    if planner_output.status != PlannerStatus::NeedsConfirmation {
        return Err(ToolError {
            code: String::from("confirmation_required_by_runtime_policy"),
            message: String::from(
                "planner marked a runtime-protected action ready without confirmation",
            ),
            retryable: false,
            details: serde_json::to_value(decision).ok(),
        });
    }

    if !planner_output.requires_confirmation {
        return Err(invalid_planner_output(
            "runtime-protected plan must set requires_confirmation",
            serde_json::to_value(decision).ok(),
        ));
    }

    let confirm_steps = planner_output
        .steps
        .iter()
        .filter(|step| step.tool_name == ToolName::ConfirmAction)
        .collect::<Vec<_>>();
    if confirm_steps.len() != 1 {
        return Err(invalid_planner_output(
            "runtime-protected plan must contain exactly one confirm_action step",
            serde_json::to_value(decision).ok(),
        ));
    }

    let first_step = planner_output.steps.first().ok_or_else(|| {
        invalid_planner_output(
            "runtime-protected plan has no confirmation gate",
            serde_json::to_value(decision).ok(),
        )
    })?;
    if first_step.tool_name != ToolName::ConfirmAction {
        return Err(invalid_planner_output(
            "confirm_action must be the first step of every runtime-protected plan",
            serde_json::to_value(decision).ok(),
        ));
    }
    if !matches!(
        &first_step.on_success,
        StepTransition::RequestConfirmation
    ) {
        return Err(invalid_planner_output(
            "confirm_action success must transition to RequestConfirmation",
            serde_json::to_value(decision).ok(),
        ));
    }
    if !matches!(&first_step.on_failure, StepTransition::Replan) {
        return Err(invalid_planner_output(
            "confirm_action failure must replan and may not route to a protected action",
            serde_json::to_value(decision).ok(),
        ));
    }

    Ok(())
}

fn validate_protected_intent_consistency(
    planner_output: &PlannerOutput,
) -> Result<(), ToolError> {
    let has_explicit_submit = planner_output
        .steps
        .iter()
        .any(|step| step.tool_name == ToolName::SubmitActiveForm);
    let has_submit_via_text_entry = planner_output.steps.iter().any(|step| {
        if step.tool_name != ToolName::TypeIntoElement {
            return false;
        }
        serde_json::from_value::<TypeIntoElementInput>(step.arguments.clone())
            .map(|input| input.submit_mode.submits_after_entry())
            .unwrap_or(true)
    });

    if (has_explicit_submit || has_submit_via_text_entry)
        && planner_output.intent.name != IntentName::SubmitForm
    {
        return Err(invalid_planner_output(
            "planner intent is inconsistent with an actual form-submission action",
            Some(serde_json::json!({
                "intent": planner_output.intent.name,
                "has_submit_active_form": has_explicit_submit,
                "has_submit_via_text_entry": has_submit_via_text_entry,
            })),
        ));
    }

    let has_click = planner_output
        .steps
        .iter()
        .any(|step| step.tool_name == ToolName::ClickElement);
    if has_click
        && !has_explicit_submit
        && !has_submit_via_text_entry
        && planner_output.intent.name != IntentName::ClickElement
    {
        return Err(invalid_planner_output(
            "planner intent is inconsistent with an actual click action",
            Some(serde_json::json!({
                "intent": planner_output.intent.name,
            })),
        ));
    }

    Ok(())
}

'''
text = replace_once(text, anchor, addition + anchor, path)
write(path, text)

path = "src-tauri/src/app_core/command_dispatch.rs"
text = read(path)
text = replace_once(
    text,
    "    resolve_direct_voice_input_command, validate_planner_output, AvailableTool, ExecutionOutcome,\n    PlannerInput, PlannerOutput, PlannerToolHistoryEntry, ToolError,\n",
    "    resolve_direct_voice_input_command, validate_planner_output_with_safety, AvailableTool,\n    ExecutionOutcome, PlannerInput, PlannerOutput, PlannerSafetySettings,\n    PlannerToolHistoryEntry, ToolError,\n",
    path,
)
text = replace_once(
    text,
    "        let available_tools = planner_available_tools();\n",
    "        let available_tools = planner_available_tools();\n        let planner_safety = PlannerSafetySettings::from(&self.config.safety);\n",
    path,
)
text = text.replace("validate_planner_output(\n", "validate_planner_output_with_safety(\n")
needle = "                &skill_selection.active_skill_names,\n            )?;"
replacement = "                &skill_selection.active_skill_names,\n                &planner_safety,\n            )?;"
count = text.count(needle)
if count < 10:
    raise RuntimeError(f"{path}: expected many validator call sites, found {count}")
text = text.replace(needle, replacement)
if "validate_planner_output(" in text:
    raise RuntimeError(f"{path}: legacy validator call remained")
write(path, text)

path = "src-tauri/src/app_core/replanning_orchestrator.rs"
text = read(path)
text = replace_once(
    text,
    "    validate_planner_output, ExecutionOutcome, ExecutionTrace, PlannerOutput,\n    PlannerToolHistoryEntry, ToolError,\n",
    "    validate_planner_output_with_safety, ExecutionOutcome, ExecutionTrace, PlannerOutput,\n    PlannerToolHistoryEntry, ToolError,\n",
    path,
)
text = replace_once(
    text,
    "                validate_planner_output(&planner_output, &available_tools, &active_skill_names)?;\n",
    "                validate_planner_output_with_safety(\n                    &planner_output,\n                    &available_tools,\n                    &active_skill_names,\n                    &planner_input.safety,\n                )?;\n",
    path,
)
write(path, text)

path = "src-tauri/src/commands/planner_executor/execution.rs"
text = read(path)
text = replace_once(
    text,
    "    ExecutionOutcome, ExecutionTrace, PendingPlanExecutionState, PlannedStep, PlannerOutput,\n    PlannerStatus, SerializedToolResult, StepTransition, ToolError,\n",
    "    evaluate_action_policy, ConfirmationRequirement, ExecutionOutcome, ExecutionTrace,\n    PendingPlanExecutionState, PlannedStep, PlannerOutput, PlannerSafetySettings, PlannerStatus,\n    SerializedToolResult, StepTransition, ToolError,\n",
    path,
)
insert_anchor = "pub(crate) fn execute_planner_output_with_runner<Runner>(\n"
helper = r'''fn executor_minimum_safety() -> PlannerSafetySettings {
    PlannerSafetySettings {
        confirmation_confidence_threshold: 1.0,
        allow_click_without_confirmation: false,
        always_confirm_submit: true,
    }
}

fn initial_execution_policy_error(planner_output: &PlannerOutput) -> Option<ToolError> {
    let decision = evaluate_action_policy(&planner_output.steps, &executor_minimum_safety());
    match decision.requirement {
        ConfirmationRequirement::Prohibited => Some(ToolError {
            code: String::from("prohibited_action_at_execution"),
            message: String::from(
                "executor refused an action prohibited by deterministic runtime policy",
            ),
            retryable: false,
            details: serde_json::to_value(decision).ok(),
        }),
        ConfirmationRequirement::ConfirmationRequired
            if planner_output.status != PlannerStatus::NeedsConfirmation =>
        {
            Some(ToolError {
                code: String::from("unconfirmed_side_effect_at_execution"),
                message: String::from(
                    "executor refused a protected action that reached dispatch without confirmation",
                ),
                retryable: false,
                details: serde_json::to_value(decision).ok(),
            })
        }
        ConfirmationRequirement::NoConfirmation
        | ConfirmationRequirement::ConfirmationRequired => None,
    }
}

fn resumed_execution_policy_error(steps: &[PlannedStep]) -> Option<ToolError> {
    let decision = evaluate_action_policy(steps, &executor_minimum_safety());
    if decision.requirement == ConfirmationRequirement::Prohibited {
        return Some(ToolError {
            code: String::from("prohibited_action_at_execution"),
            message: String::from(
                "executor refused a prohibited action even after confirmation",
            ),
            retryable: false,
            details: serde_json::to_value(decision).ok(),
        });
    }
    None
}

'''
text = replace_once(text, insert_anchor, helper + insert_anchor, path)
trace_anchor = "    let trace = ExecutionTrace {\n        executed_step_ids: Vec::new(),\n        tool_results: Vec::new(),\n    };\n\n    match planner_output.status {\n"
trace_replacement = "    let trace = ExecutionTrace {\n        executed_step_ids: Vec::new(),\n        tool_results: Vec::new(),\n    };\n\n    if let Some(error) = initial_execution_policy_error(planner_output) {\n        return ExecutionOutcome::Aborted { trace, error };\n    }\n\n    match planner_output.status {\n"
text = replace_once(text, trace_anchor, trace_replacement, path)
resume_anchor = "    let trace = ExecutionTrace {\n        executed_step_ids: Vec::new(),\n        tool_results: Vec::new(),\n    };\n\n    if pending_plan_execution.confirmation_id != confirmation_id {\n"
resume_replacement = "    let trace = ExecutionTrace {\n        executed_step_ids: Vec::new(),\n        tool_results: Vec::new(),\n    };\n\n    if let Some(error) = resumed_execution_policy_error(&pending_plan_execution.queued_steps) {\n        return ExecutionOutcome::Aborted { trace, error };\n    }\n\n    if pending_plan_execution.confirmation_id != confirmation_id {\n"
text = replace_once(text, resume_anchor, resume_replacement, path)
write(path, text)

path = "src-tauri/src/commands/tests/mod.rs"
text = read(path)
text = replace_once(text, "mod runtime_status;\n", "mod runtime_status;\nmod security_policy;\n", path)
write(path, text)

security_tests = r'''use super::*;

fn safety(allow_click_without_confirmation: bool) -> PlannerSafetySettings {
    PlannerSafetySettings {
        confirmation_confidence_threshold: 0.85,
        allow_click_without_confirmation,
        always_confirm_submit: true,
    }
}

fn output(status: PlannerStatus, intent: IntentName, steps: Vec<PlannedStep>) -> PlannerOutput {
    let needs_confirmation = status == PlannerStatus::NeedsConfirmation;
    PlannerOutput {
        status,
        intent: IntentSummary {
            name: intent,
            goal: String::from("security policy regression test"),
            target_description: None,
        },
        selected_skills: Vec::new(),
        steps,
        requires_confirmation: needs_confirmation,
        confirmation_reason: needs_confirmation
            .then(|| String::from("deterministic runtime policy requires confirmation")),
        blocked_reason: None,
        user_message: needs_confirmation
            .then(|| String::from("Please confirm the protected action.")),
    }
}

fn confirm_step(on_failure: StepTransition) -> PlannedStep {
    PlannedStep {
        step_id: String::from("confirm"),
        tool_name: ToolName::ConfirmAction,
        arguments: serde_json::json!({
            "request_id": "req-security",
            "timeout_ms": 1000,
            "prompt_text": "untrusted planner wording",
            "reason": "untrusted planner reason"
        }),
        purpose: String::from("request confirmation"),
        on_success: StepTransition::RequestConfirmation,
        on_failure,
    }
}

fn submit_step() -> PlannedStep {
    PlannedStep {
        step_id: String::from("submit"),
        tool_name: ToolName::SubmitActiveForm,
        arguments: serde_json::json!({
            "request_id": "req-security",
            "timeout_ms": 1000,
            "form_element_id": null
        }),
        purpose: String::from("submit the active form"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    }
}

fn click_step() -> PlannedStep {
    PlannedStep {
        step_id: String::from("click"),
        tool_name: ToolName::ClickElement,
        arguments: serde_json::json!({
            "request_id": "req-security",
            "timeout_ms": 1000,
            "element_id": "button-1",
            "click_mode": "Single"
        }),
        purpose: String::from("click the selected element"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    }
}

#[test]
fn actual_submit_tool_cannot_hide_under_read_page_intent() {
    let plan = output(PlannerStatus::Ready, IntentName::ReadPage, vec![submit_step()]);
    let error = validate_planner_output_with_safety(
        &plan,
        &planner_available_tools(),
        &[],
        &safety(false),
    )
    .expect_err("actual submit tool must be rejected regardless of declared intent");

    assert!(matches!(
        error.code.as_str(),
        "confirmation_required_by_runtime_policy" | "invalid_planner_output"
    ));
}

#[test]
fn ready_submit_after_read_only_step_is_rejected() {
    let read_step = PlannedStep {
        step_id: String::from("read"),
        tool_name: ToolName::GetRuntimeStatus,
        arguments: serde_json::json!({
            "request_id": "req-security",
            "timeout_ms": 1000,
            "include_provider_modes": true
        }),
        purpose: String::from("read status first"),
        on_success: StepTransition::NextStep {
            step_id: String::from("submit"),
        },
        on_failure: StepTransition::Replan,
    };
    let plan = output(
        PlannerStatus::Ready,
        IntentName::SubmitForm,
        vec![read_step, submit_step()],
    );

    validate_planner_output_with_safety(
        &plan,
        &planner_available_tools(),
        &[],
        &safety(false),
    )
    .expect_err("a protected later step must not bypass confirmation");
}

#[test]
fn click_setting_cannot_bypass_missing_grounding_authorization() {
    let plan = output(
        PlannerStatus::Ready,
        IntentName::ClickElement,
        vec![click_step()],
    );
    let error = validate_planner_output_with_safety(
        &plan,
        &planner_available_tools(),
        &[],
        &safety(true),
    )
    .expect_err("element id alone is not deterministic click authorization");

    assert_eq!(error.code, "confirmation_required_by_runtime_policy");
}

#[test]
fn eval_js_is_prohibited_even_when_planner_requests_confirmation() {
    let eval = PlannedStep {
        step_id: String::from("eval"),
        tool_name: ToolName::EvalJs,
        arguments: serde_json::json!({
            "request_id": "req-security",
            "timeout_ms": 1000,
            "expression": "document.body.innerHTML = 'owned'"
        }),
        purpose: String::from("execute planner supplied JavaScript"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };
    let plan = output(
        PlannerStatus::NeedsConfirmation,
        IntentName::Unknown,
        vec![confirm_step(StepTransition::Replan), eval],
    );

    let error = validate_planner_output_with_safety(
        &plan,
        &planner_available_tools(),
        &[],
        &safety(false),
    )
    .expect_err("arbitrary planner JavaScript is prohibited");
    assert_eq!(error.code, "prohibited_planner_action");
}

#[test]
fn protected_plan_requires_confirmation_as_first_step() {
    let plan = output(
        PlannerStatus::NeedsConfirmation,
        IntentName::SubmitForm,
        vec![submit_step(), confirm_step(StepTransition::Replan)],
    );
    let error = validate_planner_output_with_safety(
        &plan,
        &planner_available_tools(),
        &[],
        &safety(false),
    )
    .expect_err("protected action before confirmation must be rejected");
    assert!(error.message.contains("confirm_action must be the first step"));
}

#[test]
fn confirmation_failure_cannot_route_to_protected_action() {
    let plan = output(
        PlannerStatus::NeedsConfirmation,
        IntentName::SubmitForm,
        vec![
            confirm_step(StepTransition::NextStep {
                step_id: String::from("submit"),
            }),
            submit_step(),
        ],
    );
    let error = validate_planner_output_with_safety(
        &plan,
        &planner_available_tools(),
        &[],
        &safety(false),
    )
    .expect_err("confirmation failure must fail closed");
    assert!(error.message.contains("failure must replan"));
}

#[test]
fn correctly_gated_submit_plan_is_accepted() {
    let plan = output(
        PlannerStatus::NeedsConfirmation,
        IntentName::SubmitForm,
        vec![confirm_step(StepTransition::Replan), submit_step()],
    );

    validate_planner_output_with_safety(
        &plan,
        &planner_available_tools(),
        &[],
        &safety(false),
    )
    .expect("deterministically gated submit plan should validate");
}

#[test]
fn executor_rejects_unconfirmed_submit_when_validation_is_skipped() {
    let plan = output(PlannerStatus::Ready, IntentName::ReadPage, vec![submit_step()]);
    let outcome = execute_planner_output_with_runner(
        String::from("req-security"),
        &plan,
        |_| panic!("protected step must not reach the runner"),
    );

    let ExecutionOutcome::Aborted { error, .. } = outcome else {
        panic!("executor must abort an unconfirmed protected action");
    };
    assert_eq!(error.code, "unconfirmed_side_effect_at_execution");
}

#[test]
fn executor_allows_safe_read_only_plan_without_confirmation() {
    let step = PlannedStep {
        step_id: String::from("status"),
        tool_name: ToolName::GetRuntimeStatus,
        arguments: serde_json::json!({
            "request_id": "req-security",
            "timeout_ms": 1000,
            "include_provider_modes": true
        }),
        purpose: String::from("read runtime status"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };
    let plan = output(PlannerStatus::Ready, IntentName::GetStatus, vec![step]);
    let outcome = execute_planner_output_with_runner(
        String::from("req-security"),
        &plan,
        |executed| {
            ToolResult::success(
                executed.tool_name.clone(),
                String::from("req-security"),
                serde_json::json!({"status": "ok"}),
                Vec::new(),
            )
        },
    );

    assert!(matches!(outcome, ExecutionOutcome::Complete { .. }));
}

#[test]
fn every_tool_has_an_explicit_policy_classification() {
    let tools = [
        ToolName::OpenUrl,
        ToolName::GoBack,
        ToolName::GoForward,
        ToolName::ReloadPage,
        ToolName::GetHtml,
        ToolName::EvalJs,
        ToolName::ScrollPage,
        ToolName::CaptureScreenshot,
        ToolName::SetBrowserVisibility,
        ToolName::GetPageSnapshot,
        ToolName::ExtractPageModel,
        ToolName::ListInteractiveElements,
        ToolName::FindElement,
        ToolName::ClickElement,
        ToolName::FocusElement,
        ToolName::TypeIntoElement,
        ToolName::SubmitActiveForm,
        ToolName::ReadRegion,
        ToolName::ReadNextRegion,
        ToolName::ReadPreviousRegion,
        ToolName::StopSpeaking,
        ToolName::StartListening,
        ToolName::StopListening,
        ToolName::TranscribeCommand,
        ToolName::SetTtsVoice,
        ToolName::SetPlaybackVolume,
        ToolName::SetPlaybackSpeed,
        ToolName::RunOcr,
        ToolName::MergeOcrIntoPageModel,
        ToolName::GetAgentState,
        ToolName::GetRuntimeStatus,
        ToolName::ConfirmAction,
        ToolName::ReportResult,
    ];

    for tool in tools {
        let policy = tool_policy(&tool);
        assert_ne!(
            policy.class,
            ActionClass::CredentialOperation,
            "current tool unexpectedly fell through to a placeholder class"
        );
        assert_ne!(
            policy.class,
            ActionClass::ModelDownload,
            "current tool unexpectedly fell through to a placeholder class"
        );
    }
}
'''
write("src-tauri/src/commands/tests/security_policy.rs", security_tests)
