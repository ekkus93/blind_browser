use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::audio_io::RuntimeAudioState;
use crate::browser::BrowserVisibilityMode;
use crate::config::ProviderMode;
use crate::narration::NarrationCursor;
use crate::page_model::{InteractiveElement, PageModel};
use crate::state::{BrowserHistoryState, ListeningState};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ToolName {
    OpenUrl,
    GoBack,
    GoForward,
    ReloadPage,
    ScrollPage,
    CaptureScreenshot,
    SetBrowserVisibility,
    GetPageSnapshot,
    ExtractPageModel,
    ListInteractiveElements,
    StopSpeaking,
    StartListening,
    StopListening,
    TranscribeCommand,
    SetTtsVoice,
    SetPlaybackVolume,
    SetPlaybackSpeed,
    RunOcr,
    MergeOcrIntoPageModel,
    GetAgentState,
    GetRuntimeStatus,
    ConfirmAction,
    ReportResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ToolError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ToolWarning {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ToolResult<T> {
    pub ok: bool,
    pub tool_name: ToolName,
    pub request_id: String,
    pub timestamp_ms: u64,
    pub data: Option<T>,
    pub error: Option<ToolError>,
    pub warnings: Vec<ToolWarning>,
    pub observations: Vec<String>,
}

pub type SerializedToolResult = ToolResult<serde_json::Value>;

impl<T> ToolResult<T> {
    pub fn success(
        tool_name: ToolName,
        request_id: String,
        data: T,
        observations: Vec<String>,
    ) -> Self {
        Self {
            ok: true,
            tool_name,
            request_id,
            timestamp_ms: current_timestamp_ms(),
            data: Some(data),
            error: None,
            warnings: Vec::new(),
            observations,
        }
    }

    pub fn failure(
        tool_name: ToolName,
        request_id: String,
        error: ToolError,
        observations: Vec<String>,
    ) -> Self {
        Self {
            ok: false,
            tool_name,
            request_id,
            timestamp_ms: current_timestamp_ms(),
            data: None,
            error: Some(error),
            warnings: Vec::new(),
            observations,
        }
    }
}

pub trait DeterministicToolExecutor {
    fn execute_set_tts_voice(&mut self, input: SetTtsVoiceInput) -> ToolResult<SetTtsVoiceData>;
    fn execute_set_playback_volume(
        &mut self,
        input: SetPlaybackVolumeInput,
    ) -> ToolResult<SetPlaybackVolumeData>;
    fn execute_set_playback_speed(
        &mut self,
        input: SetPlaybackSpeedInput,
    ) -> ToolResult<SetPlaybackSpeedData>;
    fn execute_set_browser_visibility(
        &mut self,
        input: SetBrowserVisibilityInput,
    ) -> ToolResult<SetBrowserVisibilityData>;
    fn execute_get_agent_state(&mut self, input: GetAgentStateInput) -> ToolResult<AgentStateData>;
    fn execute_get_runtime_status(
        &mut self,
        input: GetRuntimeStatusInput,
    ) -> ToolResult<GetRuntimeStatusData>;
    fn execute_confirm_action(
        &mut self,
        input: ConfirmActionInput,
    ) -> ToolResult<ConfirmActionData>;
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct AvailableTool {
    pub name: ToolName,
    pub description: String,
    pub input_schema_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SkillSummary {
    pub name: String,
    pub description: String,
    pub intent_tags: Vec<String>,
    pub allowed_tools: Option<Vec<ToolName>>,
    pub requires_confirmation: bool,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PlannerToolHistoryEntry {
    pub tool_name: ToolName,
    pub ok: bool,
    pub observation_summary: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct AgentStateData {
    pub page_id: Option<String>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub browser_visibility: BrowserVisibilityMode,
    pub browser_history: BrowserHistoryState,
    pub narration_cursor: Option<NarrationCursor>,
    pub speaking: bool,
    pub listening_state: ListeningState,
    pub audio: RuntimeAudioState,
    pub last_transcript: Option<String>,
    pub last_action: Option<String>,
    pub pending_confirmation_id: Option<String>,
    pub pending_plan_execution: Option<PendingPlanExecutionState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct PageSnapshotData {
    pub url: String,
    pub title: Option<String>,
    pub visible_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ProviderSelectionStatus {
    pub planner_mode: ProviderMode,
    pub tts_mode: ProviderMode,
    pub asr_mode: ProviderMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ElementCandidate {
    pub element_id: String,
    pub confidence_bps: u16,
    pub matched_on: Vec<String>,
    pub rationale_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct GetRuntimeStatusData {
    pub page_id: Option<String>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub browser_visibility: BrowserVisibilityMode,
    pub browser_history: BrowserHistoryState,
    pub listening_state: ListeningState,
    pub speaking: bool,
    pub audio: RuntimeAudioState,
    pub pending_confirmation_id: Option<String>,
    pub pending_plan_execution: Option<PendingPlanExecutionState>,
    pub provider_modes: Option<ProviderSelectionStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct PlannerInput {
    pub request_id: String,
    pub transcript: String,
    pub agent_state: AgentStateData,
    pub available_tools: Vec<AvailableTool>,
    pub active_skill_names: Vec<String>,
    pub relevant_skill_summaries: Vec<SkillSummary>,
    pub page_snapshot: Option<PageSnapshotData>,
    pub page_model: Option<PageModel>,
    pub recent_tool_results: Vec<PlannerToolHistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum PlannerStatus {
    Ready,
    NeedsConfirmation,
    Blocked,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum BlockedReason {
    Gatekept,
    MissingContext,
    UnsupportedCapability,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum IntentName {
    OpenUrl,
    GoBack,
    GoForward,
    ReloadPage,
    GetCurrentUrl,
    ReadPage,
    ReadNext,
    ReadPrevious,
    Repeat,
    Stop,
    StartListening,
    StopListening,
    TranscribeCommand,
    SetTtsVoice,
    SetPlaybackVolume,
    GetPlaybackVolume,
    SetPlaybackSpeed,
    GetPlaybackSpeed,
    SetBrowserVisibility,
    GetStatus,
    FindElement,
    ClickElement,
    FillInput,
    SubmitForm,
    Scroll,
    OcrRecovery,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct IntentSummary {
    pub name: IntentName,
    pub goal: String,
    pub target_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum StepTransition {
    NextStep { step_id: String },
    Complete,
    RequestConfirmation,
    Replan,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ExecutionTrace {
    pub executed_step_ids: Vec<String>,
    pub tool_results: Vec<SerializedToolResult>,
}

struct StepExecutionContext<'a> {
    request_id: String,
    intent_name: IntentName,
    selected_skills: Vec<String>,
    steps: &'a [PlannedStep],
    initial_step_id: String,
    block_side_effects_until_confirmation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum ExecutionOutcome {
    Complete {
        trace: ExecutionTrace,
    },
    AwaitingConfirmation {
        trace: ExecutionTrace,
        pending_confirmation_id: String,
        pending_plan_execution: PendingPlanExecutionState,
    },
    NeedsReplan {
        trace: ExecutionTrace,
    },
    Aborted {
        trace: ExecutionTrace,
        error: ToolError,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct PlannedStep {
    pub step_id: String,
    pub tool_name: ToolName,
    pub arguments: serde_json::Value,
    pub purpose: String,
    pub on_success: StepTransition,
    pub on_failure: StepTransition,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct PlannerOutput {
    pub status: PlannerStatus,
    pub intent: IntentSummary,
    pub selected_skills: Vec<String>,
    pub steps: Vec<PlannedStep>,
    pub requires_confirmation: bool,
    pub confirmation_reason: Option<String>,
    pub blocked_reason: Option<BlockedReason>,
    pub user_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct PendingPlanExecutionState {
    pub request_id: String,
    pub intent_name: IntentName,
    pub selected_skills: Vec<String>,
    pub confirmation_id: String,
    pub prompt_text: String,
    pub next_step_id: Option<String>,
    pub queued_step_ids: Vec<String>,
    pub queued_steps: Vec<PlannedStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ConfirmActionData {
    pub confirmation_id: String,
    pub prompt_text: String,
    pub confirmed: Option<bool>,
    pub timed_out: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ConfirmActionInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub prompt_text: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ConfirmActionResolution {
    pub tool_result: ToolResult<ConfirmActionData>,
    pub resume_outcome: ExecutionOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ReportStatus {
    Success,
    NeedsFollowUp,
    Blocked,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ReportResultData {
    pub status: ReportStatus,
    pub summary: String,
    pub next_recommended_action: Option<String>,
    pub user_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SetTtsVoiceInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub voice: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SetTtsVoiceData {
    pub voice: String,
    pub changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SetPlaybackVolumeInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub volume: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SetPlaybackVolumeData {
    pub playback_volume: f32,
    pub muted: bool,
    pub changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SetPlaybackSpeedInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct SetPlaybackSpeedData {
    pub playback_speed: f32,
    pub changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SetBrowserVisibilityInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub mode: BrowserVisibilityMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SetBrowserVisibilityData {
    pub mode: BrowserVisibilityMode,
    pub changed: bool,
    pub supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct GetAgentStateInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub include_last_transcript: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct GetRuntimeStatusInput {
    pub request_id: String,
    pub timeout_ms: Option<u64>,
    pub include_provider_modes: bool,
}

pub fn execute_planned_step<E: DeterministicToolExecutor>(
    executor: &mut E,
    step: &PlannedStep,
) -> SerializedToolResult {
    match step.tool_name {
        ToolName::SetTtsVoice => {
            execute_serialized_tool(step, ToolName::SetTtsVoice, executor, |executor, input| {
                executor.execute_set_tts_voice(input)
            })
        }
        ToolName::SetPlaybackVolume => execute_serialized_tool(
            step,
            ToolName::SetPlaybackVolume,
            executor,
            |executor, input| executor.execute_set_playback_volume(input),
        ),
        ToolName::SetPlaybackSpeed => execute_serialized_tool(
            step,
            ToolName::SetPlaybackSpeed,
            executor,
            |executor, input| executor.execute_set_playback_speed(input),
        ),
        ToolName::SetBrowserVisibility => execute_serialized_tool(
            step,
            ToolName::SetBrowserVisibility,
            executor,
            |executor, input| executor.execute_set_browser_visibility(input),
        ),
        ToolName::GetAgentState => execute_serialized_tool(
            step,
            ToolName::GetAgentState,
            executor,
            |executor, input| executor.execute_get_agent_state(input),
        ),
        ToolName::GetRuntimeStatus => execute_serialized_tool(
            step,
            ToolName::GetRuntimeStatus,
            executor,
            |executor, input| executor.execute_get_runtime_status(input),
        ),
        ToolName::ConfirmAction => execute_serialized_tool(
            step,
            ToolName::ConfirmAction,
            executor,
            |executor, input| executor.execute_confirm_action(input),
        ),
        _ => ToolResult::failure(
            step.tool_name.clone(),
            inferred_request_id(step),
            ToolError {
                code: String::from("unsupported_tool"),
                message: format!(
                    "planner/executor dispatch for {:?} is not implemented yet",
                    step.tool_name
                ),
                retryable: false,
                details: Some(serde_json::json!({ "step_id": step.step_id })),
            },
            vec![String::from(
                "Executor could not dispatch the requested tool because it is not wired yet.",
            )],
        ),
    }
}

pub fn execute_planner_output<E: DeterministicToolExecutor>(
    executor: &mut E,
    request_id: String,
    planner_output: &PlannerOutput,
) -> ExecutionOutcome {
    execute_planner_output_with_runner(request_id, planner_output, |step| {
        execute_planned_step(executor, step)
    })
}

pub fn resume_after_confirmation<E: DeterministicToolExecutor>(
    executor: &mut E,
    pending_plan_execution: &PendingPlanExecutionState,
    confirmation_id: &str,
    confirmed: bool,
) -> ExecutionOutcome {
    resume_after_confirmation_with_runner(
        pending_plan_execution,
        confirmation_id,
        confirmed,
        |step| execute_planned_step(executor, step),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SkillFrontmatter {
    pub name: String,
    pub description: String,
    pub allowed_tools: Option<Vec<ToolName>>,
    pub intent_tags: Vec<String>,
    pub requires_confirmation: bool,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ElementSearchResult {
    pub query: String,
    pub matches: Vec<ElementCandidate>,
    pub elements: Vec<InteractiveElement>,
}

pub fn registered_tools() -> Vec<AvailableTool> {
    use ToolName::*;

    [
        OpenUrl,
        GoBack,
        GoForward,
        ReloadPage,
        ScrollPage,
        CaptureScreenshot,
        SetBrowserVisibility,
        GetPageSnapshot,
        ExtractPageModel,
        ListInteractiveElements,
        StopSpeaking,
        StartListening,
        StopListening,
        TranscribeCommand,
        SetTtsVoice,
        SetPlaybackVolume,
        SetPlaybackSpeed,
        RunOcr,
        MergeOcrIntoPageModel,
        GetAgentState,
        GetRuntimeStatus,
        ConfirmAction,
        ReportResult,
    ]
    .into_iter()
    .map(|name| AvailableTool {
        input_schema_ref: format!("schema://tool-input/{name:?}"),
        description: format!("Deterministic tool contract for {name:?}."),
        name,
    })
    .collect()
}

fn execute_planner_output_with_runner<Runner>(
    request_id: String,
    planner_output: &PlannerOutput,
    mut run_step: Runner,
) -> ExecutionOutcome
where
    Runner: FnMut(&PlannedStep) -> SerializedToolResult,
{
    let trace = ExecutionTrace {
        executed_step_ids: Vec::new(),
        tool_results: Vec::new(),
    };

    match planner_output.status {
        PlannerStatus::Complete => {
            return ExecutionOutcome::Complete { trace };
        }
        PlannerStatus::Blocked => {
            return ExecutionOutcome::Aborted {
                trace,
                error: ToolError {
                    code: String::from("planner_blocked"),
                    message: planner_output.user_message.clone().unwrap_or_else(|| {
                        String::from("planner blocked execution before any tools could run")
                    }),
                    retryable: false,
                    details: Some(serde_json::json!({
                        "blocked_reason": planner_output.blocked_reason,
                    })),
                },
            };
        }
        PlannerStatus::Ready | PlannerStatus::NeedsConfirmation => {}
    }

    let Some(current_step_id) = planner_output
        .steps
        .first()
        .map(|step| step.step_id.clone())
    else {
        return ExecutionOutcome::Aborted {
            trace,
            error: ToolError {
                code: String::from("empty_plan"),
                message: String::from("planner returned no executable steps"),
                retryable: false,
                details: Some(serde_json::json!({
                    "planner_status": planner_output.status,
                })),
            },
        };
    };

    execute_steps_with_runner(
        StepExecutionContext {
            request_id,
            intent_name: planner_output.intent.name.clone(),
            selected_skills: planner_output.selected_skills.clone(),
            steps: &planner_output.steps,
            initial_step_id: current_step_id,
            block_side_effects_until_confirmation: planner_output.status
                == PlannerStatus::NeedsConfirmation,
        },
        &mut run_step,
        trace,
    )
}

fn resume_after_confirmation_with_runner<Runner>(
    pending_plan_execution: &PendingPlanExecutionState,
    confirmation_id: &str,
    confirmed: bool,
    mut run_step: Runner,
) -> ExecutionOutcome
where
    Runner: FnMut(&PlannedStep) -> SerializedToolResult,
{
    let trace = ExecutionTrace {
        executed_step_ids: Vec::new(),
        tool_results: Vec::new(),
    };

    if pending_plan_execution.confirmation_id != confirmation_id {
        return ExecutionOutcome::Aborted {
            trace,
            error: ToolError {
                code: String::from("confirmation_id_mismatch"),
                message: String::from(
                    "confirmation response did not match the pending confirmation id",
                ),
                retryable: false,
                details: Some(serde_json::json!({
                    "expected_confirmation_id": pending_plan_execution.confirmation_id,
                    "received_confirmation_id": confirmation_id,
                })),
            },
        };
    }

    if !confirmed {
        return ExecutionOutcome::NeedsReplan { trace };
    }

    let Some(next_step_id) = pending_plan_execution.next_step_id.clone() else {
        return ExecutionOutcome::Complete { trace };
    };

    execute_steps_with_runner(
        StepExecutionContext {
            request_id: pending_plan_execution.request_id.clone(),
            intent_name: pending_plan_execution.intent_name.clone(),
            selected_skills: pending_plan_execution.selected_skills.clone(),
            steps: &pending_plan_execution.queued_steps,
            initial_step_id: next_step_id,
            block_side_effects_until_confirmation: false,
        },
        &mut run_step,
        trace,
    )
}

fn execute_steps_with_runner<Runner>(
    context: StepExecutionContext<'_>,
    run_step: &mut Runner,
    mut trace: ExecutionTrace,
) -> ExecutionOutcome
where
    Runner: FnMut(&PlannedStep) -> SerializedToolResult,
{
    let StepExecutionContext {
        request_id,
        intent_name,
        selected_skills,
        steps,
        initial_step_id,
        block_side_effects_until_confirmation,
    } = context;

    let step_positions = match build_step_positions(steps) {
        Ok(positions) => positions,
        Err(error) => {
            return ExecutionOutcome::Aborted { trace, error };
        }
    };

    if !step_positions.contains_key(&initial_step_id) {
        return ExecutionOutcome::Aborted {
            trace,
            error: ToolError {
                code: String::from("missing_resume_step"),
                message: format!(
                    "pending execution referenced missing step '{}'",
                    initial_step_id
                ),
                retryable: false,
                details: None,
            },
        };
    }

    let mut current_step_id = initial_step_id;
    let mut visited_step_ids = HashSet::new();
    loop {
        if !visited_step_ids.insert(current_step_id.clone()) {
            return ExecutionOutcome::Aborted {
                trace,
                error: ToolError {
                    code: String::from("planner_step_cycle"),
                    message: format!(
                        "planner attempted to execute step '{}' more than once",
                        current_step_id
                    ),
                    retryable: false,
                    details: None,
                },
            };
        }

        let step = &steps[*step_positions
            .get(&current_step_id)
            .expect("step positions should contain the current step")];

        if block_side_effects_until_confirmation && is_side_effecting_tool(&step.tool_name) {
            return ExecutionOutcome::Aborted {
                trace,
                error: ToolError {
                    code: String::from("side_effect_before_confirmation"),
                    message: format!(
                        "planner attempted to execute side-effecting tool {:?} before confirmation",
                        step.tool_name
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({
                        "step_id": step.step_id,
                        "tool_name": step.tool_name,
                    })),
                },
            };
        }

        let result = run_step(step);
        trace.executed_step_ids.push(step.step_id.clone());
        trace.tool_results.push(result.clone());

        let transition = if result.ok {
            &step.on_success
        } else {
            &step.on_failure
        };

        match transition {
            StepTransition::NextStep { step_id } => {
                if !step_positions.contains_key(step_id) {
                    return ExecutionOutcome::Aborted {
                        trace,
                        error: ToolError {
                            code: String::from("missing_transition_step"),
                            message: format!(
                                "planner referenced missing next step '{}' from '{}'",
                                step_id, step.step_id
                            ),
                            retryable: false,
                            details: None,
                        },
                    };
                }
                current_step_id = step_id.clone();
            }
            StepTransition::Complete => {
                return ExecutionOutcome::Complete { trace };
            }
            StepTransition::RequestConfirmation => {
                let confirmation_id = match extract_confirmation_id(&result) {
                    Ok(confirmation_id) => confirmation_id,
                    Err(error) => {
                        return ExecutionOutcome::Aborted { trace, error };
                    }
                };
                let prompt_text = match extract_confirmation_prompt_text(&result) {
                    Ok(prompt_text) => prompt_text,
                    Err(error) => {
                        return ExecutionOutcome::Aborted { trace, error };
                    }
                };

                let queued_step_ids = queued_step_ids_after(steps, step, &step_positions);
                let queued_steps = queued_steps_after(steps, step, &step_positions);
                let pending_plan_execution = PendingPlanExecutionState {
                    request_id,
                    intent_name: intent_name.clone(),
                    selected_skills: selected_skills.clone(),
                    confirmation_id: confirmation_id.clone(),
                    prompt_text,
                    next_step_id: queued_step_ids.first().cloned(),
                    queued_step_ids,
                    queued_steps,
                };

                return ExecutionOutcome::AwaitingConfirmation {
                    trace,
                    pending_confirmation_id: confirmation_id,
                    pending_plan_execution,
                };
            }
            StepTransition::Replan => {
                return ExecutionOutcome::NeedsReplan { trace };
            }
        }
    }
}

fn execute_serialized_tool<E, Input, Output, Handler>(
    step: &PlannedStep,
    tool_name: ToolName,
    executor: &mut E,
    handler: Handler,
) -> SerializedToolResult
where
    E: DeterministicToolExecutor,
    Input: for<'de> Deserialize<'de>,
    Output: Serialize,
    Handler: FnOnce(&mut E, Input) -> ToolResult<Output>,
{
    match serde_json::from_value::<Input>(step.arguments.clone()) {
        Ok(input) => serialize_tool_result(handler(executor, input)),
        Err(error) => ToolResult::failure(
            tool_name,
            inferred_request_id(step),
            ToolError {
                code: String::from("invalid_tool_arguments"),
                message: format!("tool arguments did not match the expected schema: {error}"),
                retryable: false,
                details: Some(serde_json::json!({
                    "step_id": step.step_id,
                    "arguments": step.arguments,
                })),
            },
            vec![String::from(
                "Executor rejected the tool call because the arguments were invalid.",
            )],
        ),
    }
}

fn build_step_positions(steps: &[PlannedStep]) -> Result<HashMap<String, usize>, ToolError> {
    let mut positions = HashMap::with_capacity(steps.len());

    for (index, step) in steps.iter().enumerate() {
        if positions.insert(step.step_id.clone(), index).is_some() {
            return Err(ToolError {
                code: String::from("duplicate_step_id"),
                message: format!("planner returned duplicate step id '{}'", step.step_id),
                retryable: false,
                details: None,
            });
        }
    }

    Ok(positions)
}

fn queued_step_ids_after(
    steps: &[PlannedStep],
    current_step: &PlannedStep,
    step_positions: &HashMap<String, usize>,
) -> Vec<String> {
    let Some(current_index) = step_positions.get(&current_step.step_id).copied() else {
        return Vec::new();
    };

    steps
        .iter()
        .skip(current_index + 1)
        .map(|step| step.step_id.clone())
        .collect()
}

fn queued_steps_after(
    steps: &[PlannedStep],
    current_step: &PlannedStep,
    step_positions: &HashMap<String, usize>,
) -> Vec<PlannedStep> {
    let Some(current_index) = step_positions.get(&current_step.step_id).copied() else {
        return Vec::new();
    };

    steps.iter().skip(current_index + 1).cloned().collect()
}

fn extract_confirmation_id(result: &SerializedToolResult) -> Result<String, ToolError> {
    let confirmation_id = result
        .data
        .as_ref()
        .and_then(|data| data.get("confirmation_id"))
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    confirmation_id.ok_or_else(|| ToolError {
        code: String::from("missing_confirmation_id"),
        message: String::from(
            "step requested confirmation but the tool result did not include confirmation_id",
        ),
        retryable: false,
        details: Some(serde_json::json!({
            "tool_name": result.tool_name,
            "request_id": result.request_id,
        })),
    })
}

fn extract_confirmation_prompt_text(result: &SerializedToolResult) -> Result<String, ToolError> {
    let prompt_text = result
        .data
        .as_ref()
        .and_then(|data| data.get("prompt_text"))
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    prompt_text.ok_or_else(|| ToolError {
        code: String::from("missing_confirmation_prompt"),
        message: String::from(
            "step requested confirmation but the tool result did not include prompt_text",
        ),
        retryable: false,
        details: Some(serde_json::json!({
            "tool_name": result.tool_name,
            "request_id": result.request_id,
        })),
    })
}

fn is_side_effecting_tool(tool_name: &ToolName) -> bool {
    !matches!(
        tool_name,
        ToolName::CaptureScreenshot
            | ToolName::GetPageSnapshot
            | ToolName::ExtractPageModel
            | ToolName::ListInteractiveElements
            | ToolName::TranscribeCommand
            | ToolName::GetAgentState
            | ToolName::GetRuntimeStatus
            | ToolName::ConfirmAction
            | ToolName::ReportResult
    )
}

fn serialize_tool_result<T>(result: ToolResult<T>) -> SerializedToolResult
where
    T: Serialize,
{
    let ToolResult {
        ok,
        tool_name,
        request_id,
        timestamp_ms,
        data,
        error,
        warnings,
        observations,
    } = result;

    let serialized_data = match data {
        Some(data) => match serde_json::to_value(data) {
            Ok(value) => Some(value),
            Err(error) => {
                return ToolResult::failure(
                    tool_name,
                    request_id,
                    ToolError {
                        code: String::from("tool_result_serialization_failed"),
                        message: format!("failed to serialize tool result payload: {error}"),
                        retryable: false,
                        details: None,
                    },
                    vec![String::from(
                        "Executor could not serialize the tool result payload.",
                    )],
                );
            }
        },
        None => None,
    };

    ToolResult {
        ok,
        tool_name,
        request_id,
        timestamp_ms,
        data: serialized_data,
        error,
        warnings,
        observations,
    }
}

fn inferred_request_id(step: &PlannedStep) -> String {
    step.arguments
        .get("request_id")
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| step.step_id.clone())
}

fn current_timestamp_ms() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as u64,
        Err(_) => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockExecutor {
        last_voice: Option<String>,
        last_volume: Option<f32>,
        last_speed: Option<f32>,
        last_visibility: Option<BrowserVisibilityMode>,
        last_confirmation_prompt: Option<String>,
    }

    impl DeterministicToolExecutor for MockExecutor {
        fn execute_set_tts_voice(
            &mut self,
            input: SetTtsVoiceInput,
        ) -> ToolResult<SetTtsVoiceData> {
            self.last_voice = Some(input.voice.clone());
            ToolResult::success(
                ToolName::SetTtsVoice,
                input.request_id,
                SetTtsVoiceData {
                    voice: input.voice,
                    changed: true,
                },
                vec![String::from("voice updated")],
            )
        }

        fn execute_set_playback_volume(
            &mut self,
            input: SetPlaybackVolumeInput,
        ) -> ToolResult<SetPlaybackVolumeData> {
            self.last_volume = Some(input.volume);
            ToolResult::success(
                ToolName::SetPlaybackVolume,
                input.request_id,
                SetPlaybackVolumeData {
                    playback_volume: input.volume,
                    muted: input.volume == 0.0,
                    changed: true,
                },
                vec![String::from("volume updated")],
            )
        }

        fn execute_set_playback_speed(
            &mut self,
            input: SetPlaybackSpeedInput,
        ) -> ToolResult<SetPlaybackSpeedData> {
            self.last_speed = Some(input.speed);
            ToolResult::success(
                ToolName::SetPlaybackSpeed,
                input.request_id,
                SetPlaybackSpeedData {
                    playback_speed: input.speed,
                    changed: true,
                },
                vec![String::from("speed updated")],
            )
        }

        fn execute_set_browser_visibility(
            &mut self,
            input: SetBrowserVisibilityInput,
        ) -> ToolResult<SetBrowserVisibilityData> {
            self.last_visibility = Some(input.mode);
            ToolResult::success(
                ToolName::SetBrowserVisibility,
                input.request_id,
                SetBrowserVisibilityData {
                    mode: input.mode,
                    changed: true,
                    supported: true,
                },
                vec![String::from("visibility updated")],
            )
        }

        fn execute_get_agent_state(
            &mut self,
            input: GetAgentStateInput,
        ) -> ToolResult<AgentStateData> {
            ToolResult::success(
                ToolName::GetAgentState,
                input.request_id,
                AgentStateData {
                    page_id: None,
                    url: Some(String::from("https://example.com")),
                    title: Some(String::from("Example")),
                    browser_visibility: BrowserVisibilityMode::Visible,
                    browser_history: BrowserHistoryState::default(),
                    narration_cursor: Some(NarrationCursor::default()),
                    speaking: false,
                    listening_state: ListeningState::default(),
                    audio: RuntimeAudioState::default(),
                    last_transcript: if input.include_last_transcript {
                        Some(String::from("read next"))
                    } else {
                        None
                    },
                    last_action: Some(String::from("get_agent_state")),
                    pending_confirmation_id: None,
                    pending_plan_execution: None,
                },
                vec![String::from("agent state read")],
            )
        }

        fn execute_get_runtime_status(
            &mut self,
            input: GetRuntimeStatusInput,
        ) -> ToolResult<GetRuntimeStatusData> {
            ToolResult::success(
                ToolName::GetRuntimeStatus,
                input.request_id,
                GetRuntimeStatusData {
                    page_id: None,
                    url: Some(String::from("https://example.com")),
                    title: Some(String::from("Example")),
                    browser_visibility: BrowserVisibilityMode::Visible,
                    browser_history: BrowserHistoryState::default(),
                    listening_state: ListeningState::default(),
                    speaking: false,
                    audio: RuntimeAudioState::default(),
                    pending_confirmation_id: None,
                    pending_plan_execution: None,
                    provider_modes: if input.include_provider_modes {
                        Some(ProviderSelectionStatus {
                            planner_mode: ProviderMode::Remote,
                            tts_mode: ProviderMode::Local,
                            asr_mode: ProviderMode::Local,
                        })
                    } else {
                        None
                    },
                },
                vec![String::from("runtime status read")],
            )
        }

        fn execute_confirm_action(
            &mut self,
            input: ConfirmActionInput,
        ) -> ToolResult<ConfirmActionData> {
            self.last_confirmation_prompt = Some(input.prompt_text.clone());
            ToolResult::success(
                ToolName::ConfirmAction,
                input.request_id,
                ConfirmActionData {
                    confirmation_id: String::from("confirm-1"),
                    prompt_text: input.prompt_text,
                    confirmed: None,
                    timed_out: false,
                },
                vec![input.reason],
            )
        }
    }

    #[test]
    fn dispatches_set_playback_volume_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-1"),
            tool_name: ToolName::SetPlaybackVolume,
            arguments: serde_json::json!({
                "request_id": "req-1",
                "timeout_ms": 1000,
                "volume": 0.4
            }),
            purpose: String::from("update volume"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(executor.last_volume, Some(0.4));
        assert_eq!(result.tool_name, ToolName::SetPlaybackVolume);
        assert_eq!(result.request_id, "req-1");
        let data = result
            .data
            .expect("serialized tool result data should exist");
        assert_eq!(data.get("muted"), Some(&serde_json::Value::Bool(false)));
        assert_eq!(data.get("changed"), Some(&serde_json::Value::Bool(true)));
        let playback_volume = data
            .get("playback_volume")
            .and_then(serde_json::Value::as_f64)
            .expect("playback_volume should be serialized as a number");
        assert!((playback_volume - 0.4).abs() < 0.000_001);
    }

    #[test]
    fn rejects_invalid_tool_arguments_before_dispatch() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-2"),
            tool_name: ToolName::SetPlaybackSpeed,
            arguments: serde_json::json!({
                "request_id": "req-2",
                "timeout_ms": 1000,
                "speed": "fast"
            }),
            purpose: String::from("update speed"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(!result.ok);
        assert_eq!(result.tool_name, ToolName::SetPlaybackSpeed);
        assert_eq!(result.request_id, "req-2");
        assert_eq!(
            result.error.expect("error should be present").code,
            "invalid_tool_arguments"
        );
        assert_eq!(executor.last_speed, None);
    }

    #[test]
    fn dispatches_set_browser_visibility_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-3"),
            tool_name: ToolName::SetBrowserVisibility,
            arguments: serde_json::json!({
                "request_id": "req-3",
                "timeout_ms": 1000,
                "mode": "Headless"
            }),
            purpose: String::from("toggle browser visibility"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor.last_visibility,
            Some(BrowserVisibilityMode::Headless)
        );
        assert_eq!(result.tool_name, ToolName::SetBrowserVisibility);
    }

    #[test]
    fn dispatches_get_runtime_status_with_provider_modes() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-4"),
            tool_name: ToolName::GetRuntimeStatus,
            arguments: serde_json::json!({
                "request_id": "req-4",
                "timeout_ms": 1000,
                "include_provider_modes": true
            }),
            purpose: String::from("read runtime status"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        let data = result.data.expect("runtime status should serialize");
        assert!(data.get("provider_modes").is_some());
    }

    #[test]
    fn dispatches_get_agent_state_without_last_transcript() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-5"),
            tool_name: ToolName::GetAgentState,
            arguments: serde_json::json!({
                "request_id": "req-5",
                "timeout_ms": 1000,
                "include_last_transcript": false
            }),
            purpose: String::from("read agent state"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        let data = result.data.expect("agent state should serialize");
        assert_eq!(data.get("last_transcript"), Some(&serde_json::Value::Null));
    }

    #[test]
    fn dispatches_confirm_action_from_planned_step() {
        let mut executor = MockExecutor::default();
        let step = PlannedStep {
            step_id: String::from("step-confirm"),
            tool_name: ToolName::ConfirmAction,
            arguments: serde_json::json!({
                "request_id": "req-confirm-dispatch",
                "timeout_ms": 1000,
                "prompt_text": "Do you want me to continue?",
                "reason": "The next step may submit data."
            }),
            purpose: String::from("request confirmation"),
            on_success: StepTransition::RequestConfirmation,
            on_failure: StepTransition::Replan,
        };

        let result = execute_planned_step(&mut executor, &step);

        assert!(result.ok);
        assert_eq!(
            executor.last_confirmation_prompt.as_deref(),
            Some("Do you want me to continue?")
        );
        let data = result.data.expect("confirm_action should serialize");
        assert_eq!(
            data.get("confirmation_id"),
            Some(&serde_json::Value::String(String::from("confirm-1")))
        );
    }

    #[test]
    fn executes_next_step_chain_until_complete() {
        let mut executor = MockExecutor::default();
        let planner_output = PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::SetPlaybackVolume,
                goal: String::from("adjust audio"),
                target_description: None,
            },
            selected_skills: vec![String::from("audio_controls")],
            steps: vec![
                PlannedStep {
                    step_id: String::from("step-1"),
                    tool_name: ToolName::SetPlaybackVolume,
                    arguments: serde_json::json!({
                        "request_id": "req-plan",
                        "timeout_ms": 1000,
                        "volume": 0.4
                    }),
                    purpose: String::from("set the volume"),
                    on_success: StepTransition::NextStep {
                        step_id: String::from("step-2"),
                    },
                    on_failure: StepTransition::Replan,
                },
                PlannedStep {
                    step_id: String::from("step-2"),
                    tool_name: ToolName::GetRuntimeStatus,
                    arguments: serde_json::json!({
                        "request_id": "req-plan",
                        "timeout_ms": 1000,
                        "include_provider_modes": false
                    }),
                    purpose: String::from("read back the runtime state"),
                    on_success: StepTransition::Complete,
                    on_failure: StepTransition::Replan,
                },
            ],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        };

        let outcome =
            execute_planner_output(&mut executor, String::from("req-plan"), &planner_output);

        match outcome {
            ExecutionOutcome::Complete { trace } => {
                assert_eq!(trace.executed_step_ids, vec!["step-1", "step-2"]);
                assert_eq!(trace.tool_results.len(), 2);
            }
            other => panic!("expected complete outcome, got {other:?}"),
        }
    }

    #[test]
    fn follows_failure_transition_to_replan() {
        let mut executor = MockExecutor::default();
        let planner_output = PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::SetPlaybackSpeed,
                goal: String::from("adjust playback speed"),
                target_description: None,
            },
            selected_skills: vec![String::from("audio_controls")],
            steps: vec![PlannedStep {
                step_id: String::from("step-1"),
                tool_name: ToolName::SetPlaybackSpeed,
                arguments: serde_json::json!({
                    "request_id": "req-replan",
                    "timeout_ms": 1000,
                    "speed": "fast"
                }),
                purpose: String::from("set invalid speed"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        };

        let outcome =
            execute_planner_output(&mut executor, String::from("req-replan"), &planner_output);

        match outcome {
            ExecutionOutcome::NeedsReplan { trace } => {
                assert_eq!(trace.executed_step_ids, vec!["step-1"]);
                assert_eq!(trace.tool_results.len(), 1);
                assert!(!trace.tool_results[0].ok);
            }
            other => panic!("expected replan outcome, got {other:?}"),
        }
    }

    #[test]
    fn returns_awaiting_confirmation_when_transition_requests_it() {
        let planner_output = PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::ClickElement,
                goal: String::from("confirm button choice"),
                target_description: Some(String::from("submit button")),
            },
            selected_skills: vec![String::from("confirm_action")],
            steps: vec![
                PlannedStep {
                    step_id: String::from("step-1"),
                    tool_name: ToolName::ConfirmAction,
                    arguments: serde_json::json!({
                        "request_id": "req-confirm"
                    }),
                    purpose: String::from("ask for confirmation"),
                    on_success: StepTransition::RequestConfirmation,
                    on_failure: StepTransition::Replan,
                },
                PlannedStep {
                    step_id: String::from("step-2"),
                    tool_name: ToolName::SetBrowserVisibility,
                    arguments: serde_json::json!({
                        "request_id": "req-confirm",
                        "timeout_ms": 1000,
                        "mode": "Visible"
                    }),
                    purpose: String::from("placeholder protected step"),
                    on_success: StepTransition::Complete,
                    on_failure: StepTransition::Replan,
                },
            ],
            requires_confirmation: true,
            confirmation_reason: Some(String::from("protected action")),
            blocked_reason: None,
            user_message: Some(String::from("Please confirm.")),
        };

        let outcome = execute_planner_output_with_runner(
            String::from("req-confirm"),
            &planner_output,
            |step| {
                assert_eq!(step.step_id, "step-1");
                ToolResult::success(
                    ToolName::ConfirmAction,
                    String::from("req-confirm"),
                    serde_json::json!({
                        "confirmation_id": "confirm-1",
                        "prompt_text": "Proceed?",
                        "confirmed": serde_json::Value::Null,
                        "timed_out": false
                    }),
                    vec![String::from("confirmation requested")],
                )
            },
        );

        match outcome {
            ExecutionOutcome::AwaitingConfirmation {
                trace,
                pending_confirmation_id,
                pending_plan_execution,
            } => {
                assert_eq!(trace.executed_step_ids, vec!["step-1"]);
                assert_eq!(pending_confirmation_id, "confirm-1");
                assert_eq!(pending_plan_execution.request_id, "req-confirm");
                assert_eq!(pending_plan_execution.intent_name, IntentName::ClickElement);
                assert_eq!(pending_plan_execution.prompt_text, "Proceed?");
                assert_eq!(
                    pending_plan_execution.next_step_id,
                    Some(String::from("step-2"))
                );
                assert_eq!(pending_plan_execution.queued_step_ids, vec!["step-2"]);
                assert_eq!(pending_plan_execution.queued_steps.len(), 1);
                assert_eq!(pending_plan_execution.queued_steps[0].step_id, "step-2");
            }
            other => panic!("expected awaiting confirmation outcome, got {other:?}"),
        }
    }

    #[test]
    fn resumes_confirmed_pending_execution_from_stored_steps() {
        let mut executor = MockExecutor::default();
        let pending_plan_execution = PendingPlanExecutionState {
            request_id: String::from("req-resume"),
            intent_name: IntentName::SetBrowserVisibility,
            selected_skills: vec![String::from("confirm_action")],
            confirmation_id: String::from("confirm-1"),
            prompt_text: String::from("Proceed?"),
            next_step_id: Some(String::from("step-2")),
            queued_step_ids: vec![String::from("step-2")],
            queued_steps: vec![PlannedStep {
                step_id: String::from("step-2"),
                tool_name: ToolName::SetBrowserVisibility,
                arguments: serde_json::json!({
                    "request_id": "req-resume",
                    "timeout_ms": 1000,
                    "mode": "Headless"
                }),
                purpose: String::from("apply confirmed action"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
        };

        let outcome =
            resume_after_confirmation(&mut executor, &pending_plan_execution, "confirm-1", true);

        match outcome {
            ExecutionOutcome::Complete { trace } => {
                assert_eq!(trace.executed_step_ids, vec!["step-2"]);
                assert_eq!(
                    executor.last_visibility,
                    Some(BrowserVisibilityMode::Headless)
                );
            }
            other => panic!("expected complete outcome after resume, got {other:?}"),
        }
    }

    #[test]
    fn resumes_rejected_confirmation_to_replan() {
        let mut executor = MockExecutor::default();
        let pending_plan_execution = PendingPlanExecutionState {
            request_id: String::from("req-resume"),
            intent_name: IntentName::SetBrowserVisibility,
            selected_skills: vec![String::from("confirm_action")],
            confirmation_id: String::from("confirm-1"),
            prompt_text: String::from("Proceed?"),
            next_step_id: Some(String::from("step-2")),
            queued_step_ids: vec![String::from("step-2")],
            queued_steps: vec![PlannedStep {
                step_id: String::from("step-2"),
                tool_name: ToolName::SetBrowserVisibility,
                arguments: serde_json::json!({
                    "request_id": "req-resume",
                    "timeout_ms": 1000,
                    "mode": "Headless"
                }),
                purpose: String::from("apply confirmed action"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
        };

        let outcome =
            resume_after_confirmation(&mut executor, &pending_plan_execution, "confirm-1", false);

        match outcome {
            ExecutionOutcome::NeedsReplan { trace } => {
                assert!(trace.executed_step_ids.is_empty());
                assert_eq!(executor.last_visibility, None);
            }
            other => panic!("expected replan outcome after rejection, got {other:?}"),
        }
    }

    #[test]
    fn rejects_resume_with_mismatched_confirmation_id() {
        let mut executor = MockExecutor::default();
        let pending_plan_execution = PendingPlanExecutionState {
            request_id: String::from("req-resume"),
            intent_name: IntentName::SetBrowserVisibility,
            selected_skills: vec![String::from("confirm_action")],
            confirmation_id: String::from("confirm-1"),
            prompt_text: String::from("Proceed?"),
            next_step_id: Some(String::from("step-2")),
            queued_step_ids: vec![String::from("step-2")],
            queued_steps: vec![PlannedStep {
                step_id: String::from("step-2"),
                tool_name: ToolName::SetBrowserVisibility,
                arguments: serde_json::json!({
                    "request_id": "req-resume",
                    "timeout_ms": 1000,
                    "mode": "Headless"
                }),
                purpose: String::from("apply confirmed action"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
        };

        let outcome = resume_after_confirmation(
            &mut executor,
            &pending_plan_execution,
            "wrong-confirmation-id",
            true,
        );

        match outcome {
            ExecutionOutcome::Aborted { trace, error } => {
                assert!(trace.executed_step_ids.is_empty());
                assert_eq!(error.code, "confirmation_id_mismatch");
                assert_eq!(executor.last_visibility, None);
            }
            other => panic!("expected aborted outcome after mismatch, got {other:?}"),
        }
    }

    #[test]
    fn aborts_when_next_step_transition_is_missing() {
        let mut executor = MockExecutor::default();
        let planner_output = PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::SetPlaybackVolume,
                goal: String::from("adjust audio"),
                target_description: None,
            },
            selected_skills: vec![String::from("audio_controls")],
            steps: vec![PlannedStep {
                step_id: String::from("step-1"),
                tool_name: ToolName::SetPlaybackVolume,
                arguments: serde_json::json!({
                    "request_id": "req-bad-transition",
                    "timeout_ms": 1000,
                    "volume": 0.4
                }),
                purpose: String::from("set the volume"),
                on_success: StepTransition::NextStep {
                    step_id: String::from("missing-step"),
                },
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        };

        let outcome = execute_planner_output(
            &mut executor,
            String::from("req-bad-transition"),
            &planner_output,
        );

        match outcome {
            ExecutionOutcome::Aborted { trace, error } => {
                assert_eq!(trace.executed_step_ids, vec!["step-1"]);
                assert_eq!(error.code, "missing_transition_step");
            }
            other => panic!("expected aborted outcome, got {other:?}"),
        }
    }

    #[test]
    fn aborts_needs_confirmation_plan_before_side_effecting_step() {
        let mut executor = MockExecutor::default();
        let planner_output = PlannerOutput {
            status: PlannerStatus::NeedsConfirmation,
            intent: IntentSummary {
                name: IntentName::SetBrowserVisibility,
                goal: String::from("toggle browser visibility"),
                target_description: None,
            },
            selected_skills: vec![String::from("confirm_action")],
            steps: vec![PlannedStep {
                step_id: String::from("step-1"),
                tool_name: ToolName::SetBrowserVisibility,
                arguments: serde_json::json!({
                    "request_id": "req-needs-confirm",
                    "timeout_ms": 1000,
                    "mode": "Visible"
                }),
                purpose: String::from("protected action before confirmation"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: true,
            confirmation_reason: Some(String::from("protected action")),
            blocked_reason: None,
            user_message: Some(String::from("Please confirm.")),
        };

        let outcome = execute_planner_output(
            &mut executor,
            String::from("req-needs-confirm"),
            &planner_output,
        );

        match outcome {
            ExecutionOutcome::Aborted { trace, error } => {
                assert!(trace.executed_step_ids.is_empty());
                assert_eq!(error.code, "side_effect_before_confirmation");
            }
            other => panic!("expected aborted outcome, got {other:?}"),
        }
    }
}
