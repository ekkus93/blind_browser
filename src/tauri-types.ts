export type ProviderMode = "Local" | "Remote" | "Disabled";
export type SelectableProviderMode = Exclude<ProviderMode, "Disabled">;

export type BrowserVisibilityMode = "Visible" | "Headless";
export type RemoteProviderLabel = "OpenAI" | "Ollama";
export type LocalTtsBackend = "kitten_tts_rs";
export type LocalAsrBackend = "whisper";
export type RemoteTtsAudioFormat = "wav";
export type CapabilityAbsenceReason =
  | "not_configured"
  | "profile_missing"
  | "invalid_endpoint"
  | "unknown_model_id"
  | "manifest_unavailable"
  | "feature_disabled"
  | "credential_reference_missing"
  | "local_binary_unavailable";

export type ToolName =
  | "OpenUrl"
  | "GoBack"
  | "GoForward"
  | "ReloadPage"
  | "GetHtml"
  | "EvalJs"
  | "ScrollPage"
  | "CaptureScreenshot"
  | "SetBrowserVisibility"
  | "GetPageSnapshot"
  | "ExtractPageModel"
  | "ListInteractiveElements"
  | "FindElement"
  | "ClickElement"
  | "FocusElement"
  | "TypeIntoElement"
  | "SubmitActiveForm"
  | "ReadRegion"
  | "ReadNextRegion"
  | "ReadPreviousRegion"
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
  details: unknown;
}

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

export interface SetTtsVoiceData {
  voice: string;
  changed: boolean;
}

export interface SetBrowserVisibilityData {
  mode: BrowserVisibilityMode;
  changed: boolean;
  supported: boolean;
}

export interface OpenUrlData {
  final_url: string;
  title: string | null;
  page_id: string;
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

export interface SkillLoadWarning {
  source: string;
  code: string;
  count: number;
  skill: string | null;
}

export interface SkillDiscoveryDiagnostics {
  warnings: SkillLoadWarning[];
}

export interface TtsModelOption {
  profile_name: string;
  model_label: string;
}

export interface TtsModelSettings {
  mode: ProviderMode;
  active_profile: string | null;
  available_profiles: TtsModelOption[];
}

export interface LocalTtsModelSettings {
  profile_name: string | null;
  backend: LocalTtsBackend | null;
  model_id: string | null;
  model_path: string | null;
  default_voice: string | null;
  sample_rate: number | null;
}

export interface TtsVoiceOption {
  voice_name: string;
  display_label: string;
}

export interface TtsVoiceSettings {
  mode: ProviderMode;
  active_voice: string | null;
  available_voices: TtsVoiceOption[];
}

export interface TtsProviderSettings {
  active_mode: SelectableProviderMode;
  available_modes: SelectableProviderMode[];
}

export interface AsrProviderSettings {
  active_mode: SelectableProviderMode;
  available_modes: SelectableProviderMode[];
}

export interface LocalAsrModelSettings {
  profile_name: string | null;
  backend: LocalAsrBackend | null;
  model_id: string | null;
  model_path: string | null;
  language: string | null;
  threads: number | null;
}

export interface ManagedLocalModelStatusData {
  profile_name: string | null;
  backend: string | null;
  model_id: string | null;
  model_path: string | null;
  available: boolean;
  download_supported: boolean;
  download_label: string | null;
  download_absence_reason: CapabilityAbsenceReason | null;
}

export interface ModelManagementSettingsData {
  models_dir: string;
  check_on_startup: boolean;
  auto_download_missing: boolean;
  local_tts: ManagedLocalModelStatusData;
  local_asr: ManagedLocalModelStatusData;
}

export interface DownloadedLocalModelData {
  profile_name: string;
  model_id: string;
  model_path: string;
  source_url: string;
}

export interface RemotePlannerSettings {
  profile_name: string | null;
  provider: RemoteProviderLabel | null;
  base_url: string | null;
  model: string | null;
  api_key_reference: string | null;
  api_key_masked_value: string | null;
  api_key_reference_error: string | null;
  organization_reference: string | null;
  project: string | null;
  temperature_milli: number | null;
  max_output_tokens: number | null;
  timeout_ms: number | null;
  endpoint_is_loopback: boolean | null;
  availability_reason: CapabilityAbsenceReason | null;
  consent_to_remote_page_data: boolean;
  local_only: boolean;
  blocked_origins: string[];
  remote_data_notice: string;
}

export interface RemoteTtsSettings {
  profile_name: string | null;
  provider: RemoteProviderLabel | null;
  base_url: string | null;
  model: string | null;
  api_key_reference: string | null;
  api_key_masked_value: string | null;
  api_key_reference_error: string | null;
  organization_reference: string | null;
  project: string | null;
  voice: string | null;
  audio_format: RemoteTtsAudioFormat | null;
  timeout_ms: number | null;
  endpoint_is_loopback: boolean | null;
  availability_reason: CapabilityAbsenceReason | null;
}

export interface RemoteAsrSettings {
  profile_name: string | null;
  provider: RemoteProviderLabel | null;
  base_url: string | null;
  model: string | null;
  api_key_reference: string | null;
  api_key_masked_value: string | null;
  api_key_reference_error: string | null;
  organization_reference: string | null;
  project: string | null;
  language: string | null;
  temperature_milli: number | null;
  timeout_ms: number | null;
  endpoint_is_loopback: boolean | null;
  availability_reason: CapabilityAbsenceReason | null;
}

export interface SetRemoteApiKeyData {
  profile_name: string;
  api_key_reference: string;
}

export interface RemotePlannerPrivacySettingsData {
  consent_to_remote_page_data: boolean;
  local_only: boolean;
  blocked_origins: string[];
  changed: boolean;
}

export interface RemotePlannerConnectionSettingsData {
  profile_name: string;
  base_url: string;
  model: string;
}

export interface RemotePlannerModelListData {
  profile_name: string;
  base_url: string;
  models: string[];
}

export interface TestRemoteApiKeyData {
  profile_name: string;
  message: string;
}

export interface ProviderFailoverSettings {
  planner_available: boolean;
  tts_available: boolean;
  asr_available: boolean;
  summary: string;
}

export interface ConfirmationSettings {
  confirmation_confidence_threshold: number;
  allow_click_without_confirmation: boolean;
  always_confirm_submit: boolean;
}

export interface OcrThresholdSettings {
  sparse_text_char_threshold: number;
  sparse_text_region_threshold: number;
}

export interface ToolHistoryEntry {
  tool_name: ToolName;
  ok: boolean;
  observation_summary: string[];
}

export interface LastToolCallSummary {
  request_id: string;
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

export interface ConfirmationActionManifest {
  sequence: number;
  step_id: string;
  tool_name: ToolName;
  argument_digest: string;
  transition_digest: string;
  safe_summary: string;
}

export interface ConfirmationManifest {
  request_id: string;
  page_id: string | null;
  origin: string | null;
  issued_at_ms: number;
  expires_at_ms: number;
  actions: ConfirmationActionManifest[];
}

export interface PendingPlanExecutionState {
  request_id: string;
  intent_name: IntentName;
  selected_skills: string[];
  confirmation_id: string;
  manifest_digest: string;
  manifest: ConfirmationManifest;
  prompt_text: string;
  next_step_id: string | null;
  queued_step_ids: string[];
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
  tts_model_settings: TtsModelSettings;
  local_tts_model_settings: LocalTtsModelSettings;
  tts_voice_settings: TtsVoiceSettings;
  tts_provider_settings: TtsProviderSettings;
  asr_provider_settings: AsrProviderSettings;
  local_asr_model_settings: LocalAsrModelSettings;
  remote_planner_settings: RemotePlannerSettings;
  remote_tts_settings: RemoteTtsSettings;
  remote_asr_settings: RemoteAsrSettings;
  provider_failover_settings: ProviderFailoverSettings;
  confirmation_settings: ConfirmationSettings;
  ocr_threshold_settings: OcrThresholdSettings;
  last_transcript: string | null;
  last_tool_call: LastToolCallSummary | null;
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
  output_schema_ref: string;
}

export interface PlannerInput {
  request_id: string;
  transcript: string;
  agent_state: AgentStateData;
  available_tools: AvailableTool[];
  active_skill_names: string[];
  relevant_skill_summaries: SkillSummary[];
  page_snapshot: PageSnapshotData | null;
  page_model: unknown;
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
  confirmationDigest: string;
  confirmed: boolean;
  timedOut: boolean;
}

export interface DirectToolRequestInput {
  requestId: string;
  timeoutMs?: number;
  includeLastTranscript?: boolean;
}

export interface DirectTranscribeCommandInput extends DirectToolRequestInput {
  maxDurationMs?: number;
  autoStop: boolean;
}
