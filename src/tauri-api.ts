import { invoke } from "@tauri-apps/api/core";

export type ProviderMode = "Local" | "Remote" | "Disabled";

export type BrowserVisibilityMode = "Visible" | "Headless";

export type ToolName =
  | "OpenUrl"
  | "GoBack"
  | "GoForward"
  | "ReloadPage"
  | "ScrollPage"
  | "CaptureScreenshot"
  | "SetBrowserVisibility"
  | "GetPageSnapshot"
  | "ExtractPageModel"
  | "ListInteractiveElements"
  | "StopSpeaking"
  | "StartListening"
  | "StopListening"
  | "TranscribeCommand"
  | "SetTtsVoice"
  | "SetPlaybackVolume"
  | "SetPlaybackSpeed"
  | "RunOcr"
  | "MergeOcrIntoPageModel"
  | "GetAgentState"
  | "GetRuntimeStatus"
  | "ConfirmAction"
  | "ReportResult";

export type IntentName =
  | "OpenUrl"
  | "GoBack"
  | "GoForward"
  | "ReloadPage"
  | "GetCurrentUrl"
  | "ReadPage"
  | "ReadNext"
  | "ReadPrevious"
  | "Repeat"
  | "Stop"
  | "StartListening"
  | "StopListening"
  | "TranscribeCommand"
  | "SetTtsVoice"
  | "SetPlaybackVolume"
  | "GetPlaybackVolume"
  | "SetPlaybackSpeed"
  | "GetPlaybackSpeed"
  | "SetBrowserVisibility"
  | "GetStatus"
  | "FindElement"
  | "ClickElement"
  | "FillInput"
  | "SubmitForm"
  | "Scroll"
  | "OcrRecovery"
  | "Unknown";

export type PlannerStatus = "Ready" | "NeedsConfirmation" | "Blocked" | "Complete";

export type BlockedReason = "Gatekept" | "MissingContext" | "UnsupportedCapability";

export interface ToolError {
  code: string;
  message: string;
  retryable: boolean;
  details: unknown | null;
}

export interface BackendToolErrorFailure {
  kind: "tool-error";
  toolError: ToolError;
}

export interface TransportFailure {
  kind: "transport-error";
  message: string;
}

export type InvokeFailure = BackendToolErrorFailure | TransportFailure;

export interface ToolWarning {
  code: string;
  message: string;
}

export interface ToolResult<T> {
  ok: boolean;
  tool_name: ToolName;
  request_id: string;
  timestamp_ms: number;
  data: T | null;
  error: ToolError | null;
  warnings: ToolWarning[];
  observations: string[];
}

export interface StartListeningData {
  listening_state: ListeningState;
  activated: boolean;
}

export interface StopListeningData {
  listening_state: ListeningState;
  deactivated: boolean;
}

export interface TranscribeCommandData {
  transcript: string | null;
  confidence: number | null;
  audio_duration_ms: number | null;
  listening_state: ListeningState;
}

export interface TranscribeAndExecuteCommandData {
  transcription: TranscribeCommandData;
  command_error: ToolError | null;
  execution_outcome: ExecutionOutcome | null;
}

export interface SetPlaybackVolumeData {
  playback_volume: number;
  muted: boolean;
  changed: boolean;
}

export interface SetPlaybackSpeedData {
  playback_speed: number;
  changed: boolean;
}

export interface SetBrowserVisibilityData {
  mode: BrowserVisibilityMode;
  changed: boolean;
  supported: boolean;
}

export interface BrowserHistoryState {
  can_go_back: boolean;
  can_go_forward: boolean;
  current_entry_index: number | null;
  entry_count: number;
}

export interface NarrationCursor {
  node_index: number;
  char_offset: number;
}

export interface ListeningState {
  is_listening: boolean;
  push_to_talk_enabled: boolean;
}

export interface RuntimeAudioState {
  default_tts_voice: string;
  playback_volume: number;
  playback_speed: number;
  muted: boolean;
}

export interface ProviderSelectionStatus {
  planner_mode: ProviderMode;
  tts_mode: ProviderMode;
  asr_mode: ProviderMode;
}

export interface ToolHistoryEntry {
  tool_name: ToolName;
  ok: boolean;
  observation_summary: string[];
}

export interface IntentSummary {
  name: IntentName;
  goal: string;
  target_description: string | null;
}

export type StepTransition =
  | "Complete"
  | "RequestConfirmation"
  | "Replan"
  | { NextStep: { step_id: string } };

export interface PlannedStep {
  step_id: string;
  tool_name: ToolName;
  arguments: unknown;
  purpose: string;
  on_success: StepTransition;
  on_failure: StepTransition;
}

export interface PendingPlanExecutionState {
  request_id: string;
  intent_name: IntentName;
  selected_skills: string[];
  confirmation_id: string;
  prompt_text: string;
  next_step_id: string | null;
  queued_step_ids: string[];
  queued_steps: PlannedStep[];
}

export interface AgentStateData {
  page_id: string | null;
  url: string | null;
  title: string | null;
  browser_visibility: BrowserVisibilityMode;
  browser_history: BrowserHistoryState;
  narration_cursor: NarrationCursor | null;
  speaking: boolean;
  listening_state: ListeningState;
  audio: RuntimeAudioState;
  last_transcript: string | null;
  last_action: string | null;
  pending_confirmation_id: string | null;
  pending_plan_execution: PendingPlanExecutionState | null;
}

export interface PageSnapshotData {
  url: string;
  title: string | null;
  visible_text: string;
}

export interface SkillSummary {
  name: string;
  description: string;
  intent_tags: string[];
  allowed_tools: ToolName[] | null;
  requires_confirmation: boolean;
  priority: number;
}

export interface AvailableTool {
  name: ToolName;
  description: string;
  input_schema_ref: string;
}

export interface PlannerInput {
  request_id: string;
  transcript: string;
  agent_state: AgentStateData;
  available_tools: AvailableTool[];
  active_skill_names: string[];
  relevant_skill_summaries: SkillSummary[];
  page_snapshot: PageSnapshotData | null;
  page_model: unknown | null;
  recent_tool_results: ToolHistoryEntry[];
}

export interface PlannerOutput {
  status: PlannerStatus;
  intent: IntentSummary;
  selected_skills: string[];
  steps: PlannedStep[];
  requires_confirmation: boolean;
  confirmation_reason: string | null;
  blocked_reason: BlockedReason | null;
  user_message: string | null;
}

export interface ExecutionTrace {
  executed_step_ids: string[];
  tool_results: ToolResult<unknown>[];
}

export type ExecutionOutcome =
  | { Complete: { trace: ExecutionTrace } }
  | {
      AwaitingConfirmation: {
        trace: ExecutionTrace;
        pending_confirmation_id: string;
        pending_plan_execution: PendingPlanExecutionState;
      };
    }
  | { NeedsReplan: { trace: ExecutionTrace } }
  | { Aborted: { trace: ExecutionTrace; error: ToolError } };

export interface ConfirmActionData {
  confirmation_id: string;
  prompt_text: string;
  confirmed: boolean | null;
  timed_out: boolean;
}

export interface ConfirmActionResolution {
  tool_result: ToolResult<ConfirmActionData>;
  resume_outcome: ExecutionOutcome;
}

export interface ConfirmActionResponseInput {
  confirmationId: string;
  confirmed: boolean;
  timedOut: boolean;
}

export interface DirectToolRequestInput {
  requestId: string;
  timeoutMs?: number;
}

export interface DirectTranscribeCommandInput extends DirectToolRequestInput {
  maxDurationMs?: number;
  autoStop: boolean;
}

export async function executePlannerOutput(
  requestId: string,
  plannerOutput: PlannerOutput,
): Promise<ExecutionOutcome> {
  return invoke<ExecutionOutcome>("execute_planner_output", {
    requestId,
    plannerOutput,
  });
}

export async function resolveCommand(
  requestId: string,
  transcript: string,
): Promise<PlannerOutput> {
  return invoke<PlannerOutput>("resolve_command", {
    requestId,
    transcript,
  });
}

export async function submitConfirmationResponse(
  input: ConfirmActionResponseInput,
): Promise<ConfirmActionResolution> {
  return invoke<ConfirmActionResolution>("submit_confirmation_response", {
    confirmationId: input.confirmationId,
    confirmed: input.confirmed,
    timedOut: input.timedOut,
  });
}

export async function startListening(input: DirectToolRequestInput): Promise<StartListeningData> {
  const result = await invoke<ToolResult<StartListeningData>>("start_listening", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
  });
  return unwrapToolResult(result);
}

export async function stopListening(input: DirectToolRequestInput): Promise<StopListeningData> {
  const result = await invoke<ToolResult<StopListeningData>>("stop_listening", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
  });
  return unwrapToolResult(result);
}

export async function transcribeCommand(
  input: DirectTranscribeCommandInput,
): Promise<TranscribeCommandData> {
  const result = await invoke<ToolResult<TranscribeCommandData>>("transcribe_command", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
    maxDurationMs: input.maxDurationMs,
    autoStop: input.autoStop,
  });
  return unwrapToolResult(result);
}

export async function transcribeAndExecuteCommand(
  input: DirectTranscribeCommandInput,
): Promise<TranscribeAndExecuteCommandData> {
  return invoke<TranscribeAndExecuteCommandData>("transcribe_and_execute_command", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
    maxDurationMs: input.maxDurationMs,
    autoStop: input.autoStop,
  });
}

export async function getAgentState(input: DirectToolRequestInput): Promise<AgentStateData> {
  const result = await invoke<ToolResult<AgentStateData>>("get_agent_state", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
    includeLastTranscript: false,
  });
  return unwrapToolResult(result);
}

export async function setPlaybackVolume(input: {
  requestId: string;
  timeoutMs?: number;
  volume: number;
}): Promise<SetPlaybackVolumeData> {
  const result = await invoke<ToolResult<SetPlaybackVolumeData>>("set_playback_volume", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
    volume: input.volume,
  });
  return unwrapToolResult(result);
}

export async function setPlaybackSpeed(input: {
  requestId: string;
  timeoutMs?: number;
  speed: number;
}): Promise<SetPlaybackSpeedData> {
  const result = await invoke<ToolResult<SetPlaybackSpeedData>>("set_playback_speed", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
    speed: input.speed,
  });
  return unwrapToolResult(result);
}

export async function setBrowserVisibility(input: {
  requestId: string;
  timeoutMs?: number;
  mode: BrowserVisibilityMode;
}): Promise<SetBrowserVisibilityData> {
  const result = await invoke<ToolResult<SetBrowserVisibilityData>>("set_browser_visibility", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
    mode: input.mode,
  });
  return unwrapToolResult(result);
}

export function classifyInvokeFailure(error: unknown): InvokeFailure {
  const toolError = parseToolError(error);
  if (toolError) {
    return {
      kind: "tool-error",
      toolError,
    };
  }

  if (error instanceof Error && error.message.trim().length > 0) {
    return {
      kind: "transport-error",
      message: error.message,
    };
  }

  if (typeof error === "string" && error.trim().length > 0) {
    return {
      kind: "transport-error",
      message: error,
    };
  }

  return {
    kind: "transport-error",
    message: "The app could not reach the requested Tauri command.",
  };
}

function parseToolError(error: unknown): ToolError | null {
  if (!isRecord(error)) {
    return null;
  }

  const { code, message, retryable, details } = error;
  if (typeof code !== "string" || typeof message !== "string" || typeof retryable !== "boolean") {
    return null;
  }

  return {
    code,
    message,
    retryable,
    details: details ?? null,
  };
}

function unwrapToolResult<T>(result: ToolResult<T>): T {
  if (result.ok && result.data !== null) {
    return result.data;
  }

  throw result.error ?? new Error("The runtime returned an invalid tool result.");
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
