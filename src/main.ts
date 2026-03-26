import "./styles.css";

import {
  renderAudioControlsPanel,
  renderConfirmationPanel,
  renderPushToTalkPanel,
  renderSettingsAsrProviderPanel,
  renderSettingsConfirmationPanel,
  renderSettingsGuidancePanel,
  renderSettingsLocalAsrModelPanel,
  renderSettingsLocalTtsModelPanel,
  renderSettingsOcrThresholdPanel,
  renderSettingsProviderFailoverPanel,
  renderSettingsPlannerProviderPanel,
  renderSettingsTtsProviderPanel,
  renderSettingsTtsModelPanel,
  renderSettingsTtsVoicePanel,
  renderSettingsSpeedPanel,
  renderSettingsVolumePanel,
  renderStatusPanel,
  renderUrlInputPanel,
  type AudioControlsPanelState,
  type AsrProviderPanelState,
  type ConfirmationSettingsPanelState,
  type LocalAsrModelPanelState,
  type LocalTtsModelPanelState,
  type OcrThresholdSettingsPanelState,
  type PlannerProviderPanelState,
  type ProviderFailoverPanelState,
  type PushToTalkPanelState,
  type SettingsGuidancePanelState,
  type StatusPanelState,
  type TtsModelPanelState,
  type TtsProviderPanelState,
  type TtsVoicePanelState,
  type UrlInputPanelState,
} from "./confirmation-panel";
import {
  createExecutionUiStore,
  describeConfirmationSubmissionFailure,
  resolveConfirmationResponse,
  runPlannerExecution,
  type ExecutionUiState,
} from "./planner-orchestration";
import {
  classifyInvokeFailure,
  type AgentStateData,
  getAgentState,
  openUrl,
  resolveCommand,
  setAllowClickWithoutConfirmation,
  setBrowserVisibility,
  setAsrProviderSelection,
  setConfirmationThreshold,
  setOcrThresholds,
  setPlaybackSpeed,
  setPlaybackVolume,
  setTtsModelSelection,
  setTtsProviderSelection,
  setTtsVoice,
  startListening,
  stopListening,
  transcribeAndExecuteCommand,
} from "./tauri-api";

export {
  applyExecutionOutcomeToUiState,
  createExecutionUiStore,
  createInitialExecutionUiState,
  executePlannerOutput,
  isAwaitingConfirmationOutcome,
  runPlannerExecution,
  resolveConfirmationResponse,
  submitConfirmationResponse,
  type ConfirmActionData,
  type ConfirmActionResolution,
  type ConfirmationUiState,
  type ConfirmActionResponseInput,
  type ExecutionUiState,
  type ExecutionOutcome,
  type PlannerOutput,
  type ToolError,
} from "./planner-orchestration";

export {
  executePlannerOutput as invokeExecutePlannerOutput,
  getAgentState as invokeGetAgentState,
  openUrl as invokeOpenUrl,
  resolveCommand as invokeResolveCommand,
  setPlaybackSpeed as invokeSetPlaybackSpeed,
  setPlaybackVolume as invokeSetPlaybackVolume,
  setBrowserVisibility as invokeSetBrowserVisibility,
  startListening as invokeStartListening,
  stopListening as invokeStopListening,
  submitConfirmationResponse as invokeSubmitConfirmationResponse,
  transcribeAndExecuteCommand as invokeTranscribeAndExecuteCommand,
  transcribeCommand as invokeTranscribeCommand,
} from "./tauri-api";

const app = document.querySelector<HTMLDivElement>("#app");
const uiStore = createExecutionUiStore();
const PUSH_TO_TALK_RELEASE_CAPTURE_MS = 1;
const CONTINUOUS_LISTEN_CAPTURE_MS = 3_000;
let currentExecutionUiState = uiStore.getState();
let pushToTalkState: PushToTalkPanelState = createInitialPushToTalkState();
let audioControlsState: AudioControlsPanelState = createInitialAudioControlsState();
let plannerProviderPanelState: PlannerProviderPanelState = createInitialPlannerProviderPanelState();
let providerFailoverPanelState: ProviderFailoverPanelState = createInitialProviderFailoverPanelState();
let confirmationSettingsPanelState: ConfirmationSettingsPanelState = createInitialConfirmationSettingsPanelState();
let ocrThresholdSettingsPanelState: OcrThresholdSettingsPanelState = createInitialOcrThresholdSettingsPanelState();
let asrProviderPanelState: AsrProviderPanelState = createInitialAsrProviderPanelState();
let localAsrModelPanelState: LocalAsrModelPanelState = createInitialLocalAsrModelPanelState();
let ttsProviderPanelState: TtsProviderPanelState = createInitialTtsProviderPanelState();
let ttsModelPanelState: TtsModelPanelState = createInitialTtsModelPanelState();
let localTtsModelPanelState: LocalTtsModelPanelState = createInitialLocalTtsModelPanelState();
let ttsVoicePanelState: TtsVoicePanelState = createInitialTtsVoicePanelState();
let statusPanelState: StatusPanelState = createInitialStatusPanelState();
let urlInputPanelState: UrlInputPanelState = createInitialUrlInputPanelState();
let activePushToTalkSource: "keyboard" | "pointer" | null = null;
let continuousListeningLoopActive = false;

if (!app) {
  throw new Error("App root element was not found.");
}

function createInitialPushToTalkState(): PushToTalkPanelState {
  return {
    enabled: true,
    isHolding: false,
    isListening: false,
    isBusy: false,
    lastTranscript: null,
    lastError: null,
  };
}

function createInitialAudioControlsState(): AudioControlsPanelState {
  return {
    playbackVolume: 1,
    playbackSpeed: 1,
    isBusy: false,
    error: null,
  };
}

function createInitialPlannerProviderPanelState(): PlannerProviderPanelState {
  return {
    activeMode: "Remote",
    availableModes: ["Remote"],
    summary: "Planner currently uses configured remote profiles only.",
  };
}

function createInitialAsrProviderPanelState(): AsrProviderPanelState {
  return {
    activeMode: "Local",
    availableModes: ["Local", "Remote"],
    isBusy: false,
    error: null,
  };
}

function createInitialLocalAsrModelPanelState(): LocalAsrModelPanelState {
  return {
    profileName: null,
    backend: null,
    modelId: null,
    modelPath: null,
    language: null,
    threads: null,
  };
}

function createInitialProviderFailoverPanelState(): ProviderFailoverPanelState {
  return {
    plannerAvailable: false,
    ttsAvailable: false,
    asrAvailable: false,
    summary: "Automatic provider failover is not currently available in the live runtime.",
  };
}

function createInitialConfirmationSettingsPanelState(): ConfirmationSettingsPanelState {
  return {
    confirmationConfidenceThreshold: 0.9,
    allowClickWithoutConfirmation: true,
    alwaysConfirmSubmit: true,
    isBusy: false,
    error: null,
  };
}

function createInitialOcrThresholdSettingsPanelState(): OcrThresholdSettingsPanelState {
  return {
    sparseTextCharThreshold: 200,
    sparseTextRegionThreshold: 2,
    isBusy: false,
    error: null,
  };
}

function createInitialTtsProviderPanelState(): TtsProviderPanelState {
  return {
    activeMode: "Local",
    availableModes: ["Local", "Remote"],
    isBusy: false,
    error: null,
  };
}

function createInitialLocalTtsModelPanelState(): LocalTtsModelPanelState {
  return {
    profileName: null,
    backend: null,
    modelId: null,
    modelPath: null,
    defaultVoice: null,
    sampleRate: null,
  };
}

function createInitialTtsModelPanelState(): TtsModelPanelState {
  return {
    mode: "Local",
    activeProfile: null,
    availableProfiles: [],
    isBusy: false,
    error: null,
  };
}

function createInitialTtsVoicePanelState(): TtsVoicePanelState {
  return {
    mode: "Local",
    activeVoice: null,
    availableVoices: [],
    isBusy: false,
    error: null,
  };
}

function createInitialStatusPanelState(): StatusPanelState {
  return {
    pageTitle: null,
    currentRegionLabel: null,
    lastTranscript: null,
    listening: false,
    speaking: false,
    browserVisibility: "Visible",
    canGoBack: false,
    canGoForward: false,
    isUpdatingVisibility: false,
    error: null,
  };
}

function createInitialUrlInputPanelState(): UrlInputPanelState {
  return {
    draftValue: "",
    currentUrl: null,
    hasUnsubmittedChanges: false,
    isOpening: false,
    isReading: false,
    isStopping: false,
    isAdvancing: false,
    isRewinding: false,
    error: null,
  };
}

const renderApp = (
  uiState: ExecutionUiState,
  pushToTalk: PushToTalkPanelState,
  audioControls: AudioControlsPanelState,
  plannerProviderPanel: PlannerProviderPanelState,
  providerFailoverPanel: ProviderFailoverPanelState,
  confirmationSettingsPanel: ConfirmationSettingsPanelState,
  ocrThresholdSettingsPanel: OcrThresholdSettingsPanelState,
  asrProviderPanel: AsrProviderPanelState,
  localAsrModelPanel: LocalAsrModelPanelState,
  ttsProviderPanel: TtsProviderPanelState,
  ttsModelPanel: TtsModelPanelState,
  localTtsModelPanel: LocalTtsModelPanelState,
  ttsVoicePanel: TtsVoicePanelState,
  statusPanel: StatusPanelState,
  urlInputPanel: UrlInputPanelState,
) => {
  app.innerHTML = `
    <main class="shell">
      <section class="hero">
        <p class="eyebrow">Phase 0 scaffold</p>
        <h1>Voice-first browser workspace</h1>
        <p class="lede">
          Initial Tauri shell for the blind_browser runtime. This scaffold keeps the
          frontend thin while the Rust tool layer, planner contracts, and provider
          configuration are established under src-tauri.
        </p>
      </section>

      <section class="panels" aria-label="Application sections">
        <article class="panel">
          <h2>Current focus</h2>
          <p>Deterministic tool schemas, provider config models, and runtime state live in Rust.</p>
        </article>
        <article class="panel">
          <h2>Next phases</h2>
          <p>Browser control, extraction, narration, ASR, TTS, and planner execution follow this baseline.</p>
        </article>
        <article class="panel">
          <h2>UI stance</h2>
          <p>The UI remains intentionally light so voice-first behavior stays the primary product path.</p>
        </article>
        <article class="panel">
          <h2>Confirmation flow</h2>
          <p>
            Frontend orchestration now opens a dedicated confirmation UI state whenever a planner
            execution returns <strong>AwaitingConfirmation</strong>.
          </p>
        </article>
      </section>

      ${renderPushToTalkPanel(pushToTalk)}
      ${renderUrlInputPanel(urlInputPanel)}
      ${renderStatusPanel(statusPanel)}
      ${renderAudioControlsPanel(audioControls)}
      ${renderSettingsGuidancePanel(currentSettingsGuidanceState())}
      ${renderSettingsPlannerProviderPanel(plannerProviderPanel)}
      ${renderSettingsProviderFailoverPanel(providerFailoverPanel)}
      ${renderSettingsConfirmationPanel(confirmationSettingsPanel)}
      ${renderSettingsOcrThresholdPanel(ocrThresholdSettingsPanel)}
      ${renderSettingsAsrProviderPanel(asrProviderPanel)}
      ${renderSettingsLocalAsrModelPanel(localAsrModelPanel)}
      ${renderSettingsTtsProviderPanel(ttsProviderPanel)}
      ${renderSettingsTtsModelPanel(ttsModelPanel)}
      ${renderSettingsLocalTtsModelPanel(localTtsModelPanel)}
      ${renderSettingsTtsVoicePanel(ttsVoicePanel)}
      ${renderSettingsVolumePanel(audioControls)}
      ${renderSettingsSpeedPanel(audioControls)}
      ${renderConfirmationPanel(uiState.confirmation)}
    </main>
  `;
};

function rerender() {
  renderApp(
    currentExecutionUiState,
    pushToTalkState,
    audioControlsState,
    plannerProviderPanelState,
    providerFailoverPanelState,
    confirmationSettingsPanelState,
    ocrThresholdSettingsPanelState,
    asrProviderPanelState,
    localAsrModelPanelState,
    ttsProviderPanelState,
    ttsModelPanelState,
    localTtsModelPanelState,
    ttsVoicePanelState,
    statusPanelState,
    urlInputPanelState,
  );
}

function setPushToTalkState(nextState: Partial<PushToTalkPanelState>) {
  pushToTalkState = {
    ...pushToTalkState,
    ...nextState,
  };
  rerender();
}

function setAudioControlsState(nextState: Partial<AudioControlsPanelState>) {
  audioControlsState = {
    ...audioControlsState,
    ...nextState,
  };
  rerender();
}

function setPlannerProviderPanelState(nextState: Partial<PlannerProviderPanelState>) {
  plannerProviderPanelState = {
    ...plannerProviderPanelState,
    ...nextState,
  };
  rerender();
}

function setProviderFailoverPanelState(nextState: Partial<ProviderFailoverPanelState>) {
  providerFailoverPanelState = {
    ...providerFailoverPanelState,
    ...nextState,
  };
  rerender();
}

function setConfirmationSettingsPanelState(nextState: Partial<ConfirmationSettingsPanelState>) {
  confirmationSettingsPanelState = {
    ...confirmationSettingsPanelState,
    ...nextState,
  };
  rerender();
}

function setOcrThresholdSettingsPanelState(nextState: Partial<OcrThresholdSettingsPanelState>) {
  ocrThresholdSettingsPanelState = {
    ...ocrThresholdSettingsPanelState,
    ...nextState,
  };
  rerender();
}

function setAsrProviderPanelState(nextState: Partial<AsrProviderPanelState>) {
  asrProviderPanelState = {
    ...asrProviderPanelState,
    ...nextState,
  };
  rerender();
}

function setLocalAsrModelPanelState(nextState: Partial<LocalAsrModelPanelState>) {
  localAsrModelPanelState = {
    ...localAsrModelPanelState,
    ...nextState,
  };
  rerender();
}

function setTtsProviderPanelState(nextState: Partial<TtsProviderPanelState>) {
  ttsProviderPanelState = {
    ...ttsProviderPanelState,
    ...nextState,
  };
  rerender();
}

function setTtsModelPanelState(nextState: Partial<TtsModelPanelState>) {
  ttsModelPanelState = {
    ...ttsModelPanelState,
    ...nextState,
  };
  rerender();
}

function setLocalTtsModelPanelState(nextState: Partial<LocalTtsModelPanelState>) {
  localTtsModelPanelState = {
    ...localTtsModelPanelState,
    ...nextState,
  };
  rerender();
}

function setTtsVoicePanelState(nextState: Partial<TtsVoicePanelState>) {
  ttsVoicePanelState = {
    ...ttsVoicePanelState,
    ...nextState,
  };
  rerender();
}

function setStatusPanelState(nextState: Partial<StatusPanelState>) {
  statusPanelState = {
    ...statusPanelState,
    ...nextState,
  };
  rerender();
}

function setUrlInputPanelState(nextState: Partial<UrlInputPanelState>) {
  urlInputPanelState = {
    ...urlInputPanelState,
    ...nextState,
  };
  rerender();
}

function createRequestId(prefix: string): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return `${prefix}-${crypto.randomUUID()}`;
  }

  return `${prefix}-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }

  return (
    target.isContentEditable ||
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement
  );
}

function isPushToTalkKeyEvent(event: KeyboardEvent): boolean {
  return (
    event.code === "Space" &&
    !event.repeat &&
    !event.altKey &&
    !event.ctrlKey &&
    !event.metaKey &&
    !isEditableTarget(event.target)
  );
}

function describePushToTalkFailure(error: unknown): string {
  const failure = classifyInvokeFailure(error);
  if (failure.kind === "tool-error") {
    return failure.toolError.message;
  }

  return failure.message;
}

function describeAudioControlFailure(error: unknown): string {
  const failure = classifyInvokeFailure(error);
  if (failure.kind === "tool-error") {
    return failure.toolError.message;
  }

  return failure.message;
}

function describeUrlInputFailure(error: unknown): string {
  const failure = classifyInvokeFailure(error);
  if (failure.kind === "tool-error") {
    return failure.toolError.message;
  }

  return failure.message;
}

function describePlannerBlockedMessage(defaultMessage: string, userMessage: string | null): string {
  const trimmed = userMessage?.trim();
  return trimmed && trimmed.length > 0 ? trimmed : defaultMessage;
}

function guidanceStateForErrorMessage(message: string | null): SettingsGuidancePanelState | null {
  const normalized = message?.trim().toLowerCase();
  if (!normalized) {
    return null;
  }

  if (
    normalized.includes("local tts model path")
    || normalized.includes("failed to load the local tts model")
    || normalized.includes("tts local profile")
  ) {
    return {
      title: "Model setup needs attention",
      message: "The current local TTS setup is unavailable. Review the TTS provider or TTS model settings below.",
      actions: [
        { label: "Review TTS provider", targetId: "settings-tts-provider-control" },
        { label: "Review TTS model", targetId: "settings-tts-model-control" },
        { label: "Review local TTS reference", targetId: "settings-local-tts-model-title" },
      ],
    };
  }

  if (
    normalized.includes("local asr model path")
    || normalized.includes("failed to load the local asr model")
    || normalized.includes("asr local profile")
  ) {
    return {
      title: "Model setup needs attention",
      message: "The current local ASR setup is unavailable. Review the ASR provider and local model settings below.",
      actions: [
        { label: "Review ASR provider", targetId: "settings-asr-provider-control" },
        { label: "Review local ASR reference", targetId: "settings-local-asr-model-title" },
      ],
    };
  }

  return null;
}

function currentSettingsGuidanceState(): SettingsGuidancePanelState | null {
  return (
    guidanceStateForErrorMessage(pushToTalkState.lastError)
    ?? guidanceStateForErrorMessage(urlInputPanelState.error)
    ?? guidanceStateForErrorMessage(statusPanelState.error)
    ?? guidanceStateForErrorMessage(ttsProviderPanelState.error)
    ?? guidanceStateForErrorMessage(ttsModelPanelState.error)
    ?? guidanceStateForErrorMessage(ttsVoicePanelState.error)
    ?? guidanceStateForErrorMessage(asrProviderPanelState.error)
  );
}

function currentRegionLabelForAgentState(agentState: AgentStateData): string | null {
  if (!agentState.narration_cursor) {
    return null;
  }

  return `Region ${agentState.narration_cursor.node_index + 1}`;
}

function applyAgentStateToPanels(agentState: AgentStateData) {
  setPushToTalkState({
    enabled: agentState.listening_state.push_to_talk_enabled,
    isListening: agentState.listening_state.is_listening,
    lastTranscript: agentState.last_transcript,
  });
  setAudioControlsState({
    playbackVolume: agentState.audio.playback_volume,
    playbackSpeed: agentState.audio.playback_speed,
    error: null,
  });
  setPlannerProviderPanelState({
    activeMode: agentState.planner_provider_settings.active_mode,
    availableModes: agentState.planner_provider_settings.available_modes,
    summary: agentState.planner_provider_settings.summary,
  });
  setProviderFailoverPanelState({
    plannerAvailable: agentState.provider_failover_settings.planner_available,
    ttsAvailable: agentState.provider_failover_settings.tts_available,
    asrAvailable: agentState.provider_failover_settings.asr_available,
    summary: agentState.provider_failover_settings.summary,
  });
  setConfirmationSettingsPanelState({
    confirmationConfidenceThreshold: agentState.confirmation_settings.confirmation_confidence_threshold,
    allowClickWithoutConfirmation: agentState.confirmation_settings.allow_click_without_confirmation,
    alwaysConfirmSubmit: agentState.confirmation_settings.always_confirm_submit,
    isBusy: false,
    error: null,
  });
  setOcrThresholdSettingsPanelState({
    sparseTextCharThreshold: agentState.ocr_threshold_settings.sparse_text_char_threshold,
    sparseTextRegionThreshold: agentState.ocr_threshold_settings.sparse_text_region_threshold,
    isBusy: false,
    error: null,
  });
  setAsrProviderPanelState({
    activeMode: agentState.asr_provider_settings.active_mode,
    availableModes: agentState.asr_provider_settings.available_modes,
    isBusy: false,
    error: null,
  });
  setLocalAsrModelPanelState({
    profileName: agentState.local_asr_model_settings.profile_name,
    backend: agentState.local_asr_model_settings.backend,
    modelId: agentState.local_asr_model_settings.model_id,
    modelPath: agentState.local_asr_model_settings.model_path,
    language: agentState.local_asr_model_settings.language,
    threads: agentState.local_asr_model_settings.threads,
  });
  setTtsProviderPanelState({
    activeMode: agentState.tts_provider_settings.active_mode,
    availableModes: agentState.tts_provider_settings.available_modes,
    isBusy: false,
    error: null,
  });
  setTtsModelPanelState({
    mode: agentState.tts_model_settings.mode,
    activeProfile: agentState.tts_model_settings.active_profile,
    availableProfiles: agentState.tts_model_settings.available_profiles.map((option) => ({
      profileName: option.profile_name,
      modelLabel: option.model_label,
    })),
    isBusy: false,
    error: null,
  });
  setLocalTtsModelPanelState({
    profileName: agentState.local_tts_model_settings.profile_name,
    backend: agentState.local_tts_model_settings.backend,
    modelId: agentState.local_tts_model_settings.model_id,
    modelPath: agentState.local_tts_model_settings.model_path,
    defaultVoice: agentState.local_tts_model_settings.default_voice,
    sampleRate: agentState.local_tts_model_settings.sample_rate,
  });
  setTtsVoicePanelState({
    mode: agentState.tts_voice_settings.mode,
    activeVoice: agentState.tts_voice_settings.active_voice,
    availableVoices: agentState.tts_voice_settings.available_voices.map((option) => ({
      voiceName: option.voice_name,
      displayLabel: option.display_label,
    })),
    isBusy: false,
    error: null,
  });
  setStatusPanelState({
    pageTitle: agentState.title ?? agentState.url,
    currentRegionLabel: currentRegionLabelForAgentState(agentState),
    lastTranscript: agentState.last_transcript,
    listening: agentState.listening_state.is_listening,
    speaking: agentState.speaking,
    browserVisibility: agentState.browser_visibility,
    canGoBack: agentState.browser_history.can_go_back,
    canGoForward: agentState.browser_history.can_go_forward,
    error: null,
  });
  setUrlInputPanelState({
    currentUrl: agentState.url,
    draftValue: urlInputPanelState.hasUnsubmittedChanges ? urlInputPanelState.draftValue : (agentState.url ?? ""),
    isOpening: false,
    isReading: false,
    isStopping: false,
    isAdvancing: false,
    isRewinding: false,
    error: null,
  });
  if (agentState.listening_state.is_listening && !pushToTalkState.isHolding) {
    void ensureContinuousListeningLoop();
  }
}

function isUrlInputActionBusy(): boolean {
  return (
    urlInputPanelState.isOpening ||
    urlInputPanelState.isReading ||
    urlInputPanelState.isStopping ||
    urlInputPanelState.isAdvancing ||
    urlInputPanelState.isRewinding
  );
}

async function executeUrlPanelPlannerCommand(input: {
  busyState: Partial<UrlInputPanelState>;
  clearState: Partial<UrlInputPanelState>;
  requestPrefix: string;
  transcript: string;
  blockedMessage: string;
  completeMessage: string;
}) {
  if (isUrlInputActionBusy()) {
    return;
  }

  const previousState = urlInputPanelState;
  setUrlInputPanelState({
    ...input.busyState,
    error: null,
  });

  try {
    const requestId = createRequestId(input.requestPrefix);
    const plannerOutput = await resolveCommand(requestId, input.transcript);
    if (plannerOutput.status === "Blocked") {
      setUrlInputPanelState({
        ...previousState,
        ...input.clearState,
        error: describePlannerBlockedMessage(input.blockedMessage, plannerOutput.user_message),
      });
      return;
    }

    if (plannerOutput.status === "Complete") {
      setUrlInputPanelState({
        ...previousState,
        ...input.clearState,
        error: describePlannerBlockedMessage(input.completeMessage, plannerOutput.user_message),
      });
      return;
    }

    await runPlannerExecution(requestId, plannerOutput, uiStore);
    setUrlInputPanelState({
      ...previousState,
      ...input.clearState,
      error: null,
    });
    await refreshRuntimePanelsFromRuntime();
  } catch (error: unknown) {
    setUrlInputPanelState({
      ...previousState,
      ...input.clearState,
      error: describeUrlInputFailure(error),
    });
  }
}

async function refreshRuntimePanelsFromRuntime() {
  try {
    const agentState = await getAgentState({
      requestId: createRequestId("runtime-panels-state"),
      includeLastTranscript: true,
    });
    applyAgentStateToPanels(agentState);
  } catch (error: unknown) {
    const message = describeAudioControlFailure(error);
    setAudioControlsState({
      error: message,
    });
    setAsrProviderPanelState({
      error: message,
      isBusy: false,
    });
    setTtsProviderPanelState({
      error: message,
      isBusy: false,
    });
    setTtsModelPanelState({
      error: message,
      isBusy: false,
    });
    setTtsVoicePanelState({
      error: message,
      isBusy: false,
    });
    setStatusPanelState({
      error: message,
    });
  }
}

async function stopContinuousListeningAfterFailure(message: string) {
  if (!pushToTalkState.isListening) {
    setPushToTalkState({
      isBusy: false,
      lastError: message,
    });
    return;
  }

  try {
    const result = await stopListening({
      requestId: createRequestId("continuous-listen-stop"),
    });
    setPushToTalkState({
      enabled: result.listening_state.push_to_talk_enabled,
      isListening: result.listening_state.is_listening,
      isBusy: false,
      lastError: message,
    });
  } catch (error: unknown) {
    const stopFailure = describePushToTalkFailure(error);
    setPushToTalkState({
      isListening: false,
      isBusy: false,
      lastError: `${message} The runtime also failed to stop hands-free listening: ${stopFailure}`,
    });
  }
}

async function ensureContinuousListeningLoop() {
  if (continuousListeningLoopActive || !pushToTalkState.isListening || pushToTalkState.isHolding) {
    return;
  }

  continuousListeningLoopActive = true;

  try {
    while (pushToTalkState.isListening && !pushToTalkState.isHolding) {
      setPushToTalkState({
        isBusy: true,
        lastError: null,
      });

      const result = await transcribeAndExecuteCommand({
        requestId: createRequestId("continuous-listen"),
        maxDurationMs: CONTINUOUS_LISTEN_CAPTURE_MS,
        autoStop: false,
      });

      setPushToTalkState({
        enabled: result.transcription.listening_state.push_to_talk_enabled,
        isListening: result.transcription.listening_state.is_listening,
        isBusy: false,
        lastTranscript: result.transcription.transcript ?? pushToTalkState.lastTranscript,
        lastError: result.command_error?.message ?? null,
      });

      if (result.execution_outcome) {
        uiStore.applyOutcome(result.execution_outcome);
      }

      await refreshRuntimePanelsFromRuntime();

      if (!result.transcription.listening_state.is_listening) {
        break;
      }
    }
  } catch (error: unknown) {
    const message = describePushToTalkFailure(error);
    await stopContinuousListeningAfterFailure(message);
    await refreshRuntimePanelsFromRuntime();
  } finally {
    continuousListeningLoopActive = false;
  }
}

async function persistBrowserVisibility(nextMode: "Visible" | "Headless") {
  const previousMode = statusPanelState.browserVisibility;
  setStatusPanelState({
    browserVisibility: nextMode,
    isUpdatingVisibility: true,
    error: null,
  });

  try {
    const result = await setBrowserVisibility({
      requestId: createRequestId("browser-visibility"),
      mode: nextMode,
    });
    setStatusPanelState({
      browserVisibility: result.mode,
      isUpdatingVisibility: false,
      error: null,
    });
    await refreshRuntimePanelsFromRuntime();
  } catch (error: unknown) {
    setStatusPanelState({
      browserVisibility: previousMode,
      isUpdatingVisibility: false,
      error: describeAudioControlFailure(error),
    });
  }
}

async function persistPlaybackVolume(nextVolume: number) {
  const previousState = audioControlsState;
  setAudioControlsState({
    playbackVolume: nextVolume,
    isBusy: true,
    error: null,
  });

  try {
    const result = await setPlaybackVolume({
      requestId: createRequestId("audio-volume"),
      volume: nextVolume,
    });
    setAudioControlsState({
      playbackVolume: result.playback_volume,
      isBusy: false,
    });
  } catch (error: unknown) {
    setAudioControlsState({
      playbackVolume: previousState.playbackVolume,
      isBusy: false,
      error: describeAudioControlFailure(error),
    });
  }
}

async function persistPlaybackSpeed(nextSpeed: number) {
  const previousState = audioControlsState;
  setAudioControlsState({
    playbackSpeed: nextSpeed,
    isBusy: true,
    error: null,
  });

  try {
    const result = await setPlaybackSpeed({
      requestId: createRequestId("audio-speed"),
      speed: nextSpeed,
    });
    setAudioControlsState({
      playbackSpeed: result.playback_speed,
      isBusy: false,
    });
  } catch (error: unknown) {
    setAudioControlsState({
      playbackSpeed: previousState.playbackSpeed,
      isBusy: false,
      error: describeAudioControlFailure(error),
    });
  }
}

async function persistAsrProviderSelection(nextMode: "Local" | "Remote") {
  const previousState = asrProviderPanelState;
  setAsrProviderPanelState({
    activeMode: nextMode,
    isBusy: true,
    error: null,
  });

  try {
    const result = await setAsrProviderSelection({
      requestId: createRequestId("asr-provider"),
      mode: nextMode,
    });
    setAsrProviderPanelState({
      activeMode: result.mode,
      isBusy: false,
      error: null,
    });
    await refreshRuntimePanelsFromRuntime();
  } catch (error: unknown) {
    setAsrProviderPanelState({
      activeMode: previousState.activeMode,
      isBusy: false,
      error: describeAudioControlFailure(error),
    });
  }
}

async function persistTtsProviderSelection(nextMode: "Local" | "Remote") {
  const previousProviderState = ttsProviderPanelState;
  const previousModelState = ttsModelPanelState;
  const previousVoiceState = ttsVoicePanelState;
  setTtsProviderPanelState({
    activeMode: nextMode,
    isBusy: true,
    error: null,
  });
  setTtsModelPanelState({ isBusy: true, error: null });
  setTtsVoicePanelState({ isBusy: true, error: null });

  try {
    const result = await setTtsProviderSelection({
      requestId: createRequestId("tts-provider"),
      mode: nextMode,
    });
    setTtsProviderPanelState({
      activeMode: result.mode,
      isBusy: false,
      error: null,
    });
    await refreshRuntimePanelsFromRuntime();
  } catch (error: unknown) {
    const message = describeAudioControlFailure(error);
    setTtsProviderPanelState({
      activeMode: previousProviderState.activeMode,
      isBusy: false,
      error: message,
    });
    setTtsModelPanelState({
      activeProfile: previousModelState.activeProfile,
      isBusy: false,
      error: previousModelState.error,
    });
    setTtsVoicePanelState({
      activeVoice: previousVoiceState.activeVoice,
      isBusy: false,
      error: previousVoiceState.error,
    });
  }
}

async function persistTtsModelSelection(nextProfileName: string) {
  const previousState = ttsModelPanelState;
  setTtsModelPanelState({
    activeProfile: nextProfileName,
    isBusy: true,
    error: null,
  });

  try {
    const result = await setTtsModelSelection({
      requestId: createRequestId("tts-model"),
      profileName: nextProfileName,
    });
    setTtsModelPanelState({
      activeProfile: result.profile_name,
      isBusy: false,
      error: null,
    });
    await refreshRuntimePanelsFromRuntime();
  } catch (error: unknown) {
    setTtsModelPanelState({
      activeProfile: previousState.activeProfile,
      isBusy: false,
      error: describeAudioControlFailure(error),
    });
  }
}

async function persistTtsVoiceSelection(nextVoice: string) {
  const previousState = ttsVoicePanelState;
  setTtsVoicePanelState({
    activeVoice: nextVoice,
    isBusy: true,
    error: null,
  });

  try {
    const result = await setTtsVoice({
      requestId: createRequestId("tts-voice"),
      voice: nextVoice,
    });
    setTtsVoicePanelState({
      activeVoice: result.voice,
      isBusy: false,
      error: null,
    });
    await refreshRuntimePanelsFromRuntime();
  } catch (error: unknown) {
    setTtsVoicePanelState({
      activeVoice: previousState.activeVoice,
      isBusy: false,
      error: describeAudioControlFailure(error),
    });
  }
}

async function persistConfirmationThreshold(nextThreshold: number) {
  const previousState = confirmationSettingsPanelState;
  setConfirmationSettingsPanelState({
    confirmationConfidenceThreshold: nextThreshold,
    isBusy: true,
    error: null,
  });

  try {
    const result = await setConfirmationThreshold({
      requestId: createRequestId("confirmation-threshold"),
      confirmationConfidenceThreshold: nextThreshold,
    });
    setConfirmationSettingsPanelState({
      confirmationConfidenceThreshold: result.confirmation_confidence_threshold,
      isBusy: false,
      error: null,
    });
    await refreshRuntimePanelsFromRuntime();
  } catch (error: unknown) {
    setConfirmationSettingsPanelState({
      confirmationConfidenceThreshold: previousState.confirmationConfidenceThreshold,
      allowClickWithoutConfirmation: previousState.allowClickWithoutConfirmation,
      alwaysConfirmSubmit: previousState.alwaysConfirmSubmit,
      isBusy: false,
      error: describeAudioControlFailure(error),
    });
  }
}

async function persistAllowClickWithoutConfirmation(nextValue: boolean) {
  const previousState = confirmationSettingsPanelState;
  setConfirmationSettingsPanelState({
    allowClickWithoutConfirmation: nextValue,
    isBusy: true,
    error: null,
  });

  try {
    const result = await setAllowClickWithoutConfirmation({
      requestId: createRequestId("click-without-confirmation"),
      allowClickWithoutConfirmation: nextValue,
    });
    setConfirmationSettingsPanelState({
      allowClickWithoutConfirmation: result.allow_click_without_confirmation,
      isBusy: false,
      error: null,
    });
    await refreshRuntimePanelsFromRuntime();
  } catch (error: unknown) {
    setConfirmationSettingsPanelState({
      confirmationConfidenceThreshold: previousState.confirmationConfidenceThreshold,
      allowClickWithoutConfirmation: previousState.allowClickWithoutConfirmation,
      alwaysConfirmSubmit: previousState.alwaysConfirmSubmit,
      isBusy: false,
      error: describeAudioControlFailure(error),
    });
  }
}

async function persistOcrThresholds(nextCharThreshold: number, nextRegionThreshold: number) {
  const previousState = ocrThresholdSettingsPanelState;
  setOcrThresholdSettingsPanelState({
    sparseTextCharThreshold: nextCharThreshold,
    sparseTextRegionThreshold: nextRegionThreshold,
    isBusy: true,
    error: null,
  });

  try {
    const result = await setOcrThresholds({
      requestId: createRequestId("ocr-thresholds"),
      sparseTextCharThreshold: nextCharThreshold,
      sparseTextRegionThreshold: nextRegionThreshold,
    });
    setOcrThresholdSettingsPanelState({
      sparseTextCharThreshold: result.sparse_text_char_threshold,
      sparseTextRegionThreshold: result.sparse_text_region_threshold,
      isBusy: false,
      error: null,
    });
    await refreshRuntimePanelsFromRuntime();
  } catch (error: unknown) {
    setOcrThresholdSettingsPanelState({
      sparseTextCharThreshold: previousState.sparseTextCharThreshold,
      sparseTextRegionThreshold: previousState.sparseTextRegionThreshold,
      isBusy: false,
      error: describeAudioControlFailure(error),
    });
  }
}

async function openDraftUrl() {
  if (isUrlInputActionBusy()) {
    return;
  }

  const nextUrl = urlInputPanelState.draftValue.trim();
  if (nextUrl.length === 0) {
    setUrlInputPanelState({
      error: "Enter a URL before opening a page.",
    });
    return;
  }

  const previousState = urlInputPanelState;
  setUrlInputPanelState({
    isOpening: true,
    error: null,
  });

  try {
    const result = await openUrl({
      requestId: createRequestId("open-url"),
      url: nextUrl,
    });
    setUrlInputPanelState({
      draftValue: result.final_url,
      currentUrl: result.final_url,
      hasUnsubmittedChanges: false,
      isOpening: false,
      error: null,
    });
    await refreshRuntimePanelsFromRuntime();
  } catch (error: unknown) {
    setUrlInputPanelState({
      ...previousState,
      isOpening: false,
      error: describeUrlInputFailure(error),
    });
  }
}

async function readCurrentPage() {
  await executeUrlPanelPlannerCommand({
    busyState: {
      isReading: true,
    },
    clearState: {
      isReading: false,
    },
    requestPrefix: "read-page",
    transcript: "read page",
    blockedMessage: "The runtime could not start reading the current page.",
    completeMessage: "The runtime did not need to run any reading steps.",
  });
}

async function stopCurrentReading() {
  await executeUrlPanelPlannerCommand({
    busyState: {
      isStopping: true,
    },
    clearState: {
      isStopping: false,
    },
    requestPrefix: "stop-reading",
    transcript: "stop reading",
    blockedMessage: "The runtime could not stop the current reading session.",
    completeMessage: "The runtime did not need to stop any current reading.",
  });
}

async function readNextRegion() {
  await executeUrlPanelPlannerCommand({
    busyState: {
      isAdvancing: true,
    },
    clearState: {
      isAdvancing: false,
    },
    requestPrefix: "read-next",
    transcript: "continue reading",
    blockedMessage: "The runtime could not move to the next reading region.",
    completeMessage: "The runtime did not need to move to another reading region.",
  });
}

async function readPreviousRegion() {
  await executeUrlPanelPlannerCommand({
    busyState: {
      isRewinding: true,
    },
    clearState: {
      isRewinding: false,
    },
    requestPrefix: "read-previous",
    transcript: "previous section",
    blockedMessage: "The runtime could not move to the previous reading region.",
    completeMessage: "The runtime did not need to move to a previous reading region.",
  });
}

async function beginPushToTalk(source: "keyboard" | "pointer") {
  if (
    !pushToTalkState.enabled ||
    pushToTalkState.isHolding ||
    pushToTalkState.isBusy ||
    pushToTalkState.isListening
  ) {
    return;
  }

  activePushToTalkSource = source;
  setPushToTalkState({
    isHolding: true,
    isBusy: true,
    lastError: null,
    lastTranscript: null,
  });

  try {
    const result = await startListening({
      requestId: createRequestId("push-to-talk-start"),
    });
    setPushToTalkState({
      enabled: result.listening_state.push_to_talk_enabled,
      isHolding: true,
      isListening: result.listening_state.is_listening,
      isBusy: false,
    });
    await refreshRuntimePanelsFromRuntime();
  } catch (error: unknown) {
    activePushToTalkSource = null;
    setPushToTalkState({
      isHolding: false,
      isListening: false,
      isBusy: false,
      lastError: describePushToTalkFailure(error),
    });
  }
}

async function cancelPushToTalk() {
  activePushToTalkSource = null;

  if (!pushToTalkState.isHolding && !pushToTalkState.isListening) {
    return;
  }

  setPushToTalkState({
    isHolding: false,
    isBusy: true,
    lastError: null,
  });

  try {
    const result = await stopListening({
      requestId: createRequestId("push-to-talk-cancel"),
    });
    setPushToTalkState({
      enabled: result.listening_state.push_to_talk_enabled,
      isListening: result.listening_state.is_listening,
      isBusy: false,
    });
    await refreshRuntimePanelsFromRuntime();
  } catch (error: unknown) {
    setPushToTalkState({
      isListening: false,
      isBusy: false,
      lastError: describePushToTalkFailure(error),
    });
  }
}

async function releasePushToTalk(source: "keyboard" | "pointer") {
  if (!pushToTalkState.isHolding || activePushToTalkSource !== source) {
    return;
  }

  activePushToTalkSource = null;
  setPushToTalkState({
    isHolding: false,
    isBusy: true,
    lastError: null,
  });

  try {
    const result = await transcribeAndExecuteCommand({
      requestId: createRequestId("push-to-talk-transcribe"),
      maxDurationMs: PUSH_TO_TALK_RELEASE_CAPTURE_MS,
      autoStop: true,
    });
    const {
      transcription,
      command_error: commandError,
      execution_outcome: executionOutcome,
    } = result;
    setPushToTalkState({
      enabled: transcription.listening_state.push_to_talk_enabled,
      isListening: transcription.listening_state.is_listening,
      isBusy: false,
      lastTranscript: transcription.transcript,
      lastError: commandError?.message ?? null,
    });

    if (!transcription.transcript) {
      await refreshRuntimePanelsFromRuntime();
      return;
    }

    if (commandError) {
      await refreshRuntimePanelsFromRuntime();
      return;
    }

    if (executionOutcome) {
      uiStore.applyOutcome(executionOutcome);
    }
    await refreshRuntimePanelsFromRuntime();
    if (pushToTalkState.isListening && !pushToTalkState.isHolding) {
      void ensureContinuousListeningLoop();
    }
  } catch (error: unknown) {
    setPushToTalkState({
      isListening: false,
      isBusy: false,
      lastError: describePushToTalkFailure(error),
    });
  }
}

rerender();
void refreshRuntimePanelsFromRuntime();
uiStore.subscribe((uiState) => {
  currentExecutionUiState = uiState;
  rerender();
});

app.addEventListener("click", (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }

  const settingsTargetButton = target.closest<HTMLButtonElement>("[data-settings-target]");
  if (settingsTargetButton) {
    const targetId = settingsTargetButton.dataset.settingsTarget;
    if (!targetId) {
      return;
    }

    const targetElement = document.getElementById(targetId);
    if (!targetElement) {
      return;
    }

    targetElement.scrollIntoView({ behavior: "smooth", block: "center" });
    if (
      targetElement instanceof HTMLInputElement
      || targetElement instanceof HTMLSelectElement
      || targetElement instanceof HTMLButtonElement
      || targetElement instanceof HTMLTextAreaElement
    ) {
      targetElement.focus({ preventScroll: true });
    }
    return;
  }

  const visibilityButton = target.closest<HTMLButtonElement>("[data-browser-visibility-mode]");
  if (visibilityButton) {
    if (statusPanelState.isUpdatingVisibility || visibilityButton.disabled) {
      return;
    }

    const mode = visibilityButton.dataset.browserVisibilityMode;
    if (mode === "Visible" || mode === "Headless") {
      void persistBrowserVisibility(mode);
    }
    return;
  }

  const urlOpenButton = target.closest<HTMLButtonElement>("[data-url-open-button]");
  if (urlOpenButton) {
    if (
      isUrlInputActionBusy() ||
      urlOpenButton.disabled
    ) {
      return;
    }

    void openDraftUrl();
    return;
  }

  const urlReadButton = target.closest<HTMLButtonElement>("[data-url-read-button]");
  if (urlReadButton) {
    if (
      isUrlInputActionBusy() ||
      urlReadButton.disabled
    ) {
      return;
    }

    void readCurrentPage();
    return;
  }

  const urlStopButton = target.closest<HTMLButtonElement>("[data-url-stop-button]");
  if (urlStopButton) {
    if (
      isUrlInputActionBusy() ||
      urlStopButton.disabled
    ) {
      return;
    }

    void stopCurrentReading();
    return;
  }

  const urlPreviousButton = target.closest<HTMLButtonElement>("[data-url-previous-button]");
  if (urlPreviousButton) {
    if (isUrlInputActionBusy() || urlPreviousButton.disabled) {
      return;
    }

    void readPreviousRegion();
    return;
  }

  const urlNextButton = target.closest<HTMLButtonElement>("[data-url-next-button]");
  if (urlNextButton) {
    if (isUrlInputActionBusy() || urlNextButton.disabled) {
      return;
    }

    void readNextRegion();
    return;
  }

  const actionButton = target.closest<HTMLButtonElement>("[data-confirmation-action]");
  if (!actionButton) {
    return;
  }

  const confirmationState = uiStore.getState().confirmation;
  if (confirmationState.kind !== "awaiting-confirmation" || confirmationState.isSubmitting) {
    return;
  }

  const action = actionButton.dataset.confirmationAction;
  const confirmationId = actionButton.dataset.confirmationId;
  if (!confirmationId || confirmationId !== confirmationState.confirmationId) {
    return;
  }

  if (action !== "approve" && action !== "reject") {
    return;
  }

  const confirmed = action === "approve";
  uiStore.setConfirmationSubmitting(confirmationId, true);

  void resolveConfirmationResponse(
    {
      confirmationId,
      confirmed,
      timedOut: false,
    },
    uiStore,
  )
    .then(async () => {
      await refreshRuntimePanelsFromRuntime();
    })
    .catch((error: unknown) => {
      uiStore.setConfirmationError(confirmationId, describeConfirmationSubmissionFailure(error));
      console.error("Failed to submit confirmation response.", error);
    });
});

app.addEventListener("input", (event) => {
  const target = event.target;
  if (!(target instanceof HTMLInputElement)) {
    return;
  }

  if (target.dataset.audioControl === "volume") {
    setAudioControlsState({
      playbackVolume: Number.parseFloat(target.value),
      error: null,
    });
    return;
  }

  if (target.dataset.audioControl === "speed") {
    setAudioControlsState({
      playbackSpeed: Number.parseFloat(target.value),
      error: null,
    });
    return;
  }

  if (target.dataset.confirmationThresholdControl === "true") {
    setConfirmationSettingsPanelState({
      confirmationConfidenceThreshold: Number.parseFloat(target.value),
      error: null,
    });
    return;
  }

  if (target.dataset.ocrThresholdControl === "char") {
    setOcrThresholdSettingsPanelState({
      sparseTextCharThreshold: Number.parseInt(target.value, 10),
      error: null,
    });
    return;
  }

  if (target.dataset.ocrThresholdControl === "region") {
    setOcrThresholdSettingsPanelState({
      sparseTextRegionThreshold: Number.parseInt(target.value, 10),
      error: null,
    });
    return;
  }

  if (target.dataset.urlInput === "true") {
    setUrlInputPanelState({
      draftValue: target.value,
      hasUnsubmittedChanges: true,
      error: null,
    });
  }
});

app.addEventListener("change", (event) => {
  const target = event.target;
  if (!(target instanceof HTMLInputElement) && !(target instanceof HTMLSelectElement)) {
    return;
  }

  if (
    audioControlsState.isBusy ||
    confirmationSettingsPanelState.isBusy ||
    ocrThresholdSettingsPanelState.isBusy ||
    asrProviderPanelState.isBusy ||
    ttsProviderPanelState.isBusy ||
    ttsModelPanelState.isBusy ||
    ttsVoicePanelState.isBusy
  ) {
    return;
  }

  if (target instanceof HTMLInputElement && target.dataset.audioControl === "volume") {
    void persistPlaybackVolume(Number.parseFloat(target.value));
    return;
  }

  if (target instanceof HTMLInputElement && target.dataset.audioControl === "speed") {
    void persistPlaybackSpeed(Number.parseFloat(target.value));
    return;
  }

  if (target instanceof HTMLInputElement && target.dataset.confirmationThresholdControl === "true") {
    void persistConfirmationThreshold(Number.parseFloat(target.value));
    return;
  }

  if (target instanceof HTMLInputElement && target.dataset.clickWithoutConfirmationToggle === "true") {
    void persistAllowClickWithoutConfirmation(target.checked);
    return;
  }

  if (target instanceof HTMLInputElement && target.dataset.ocrThresholdControl === "char") {
    void persistOcrThresholds(
      Number.parseInt(target.value, 10),
      ocrThresholdSettingsPanelState.sparseTextRegionThreshold,
    );
    return;
  }

  if (target instanceof HTMLInputElement && target.dataset.ocrThresholdControl === "region") {
    void persistOcrThresholds(
      ocrThresholdSettingsPanelState.sparseTextCharThreshold,
      Number.parseInt(target.value, 10),
    );
    return;
  }

  if (target instanceof HTMLSelectElement && target.dataset.asrProviderSelect === "true") {
    void persistAsrProviderSelection(target.value as "Local" | "Remote");
    return;
  }

  if (target instanceof HTMLSelectElement && target.dataset.ttsProviderSelect === "true") {
    void persistTtsProviderSelection(target.value as "Local" | "Remote");
    return;
  }

  if (target instanceof HTMLSelectElement && target.dataset.ttsModelSelect === "true") {
    void persistTtsModelSelection(target.value);
    return;
  }

  if (target instanceof HTMLSelectElement && target.dataset.ttsVoiceSelect === "true") {
    void persistTtsVoiceSelection(target.value);
  }
});

app.addEventListener("pointerdown", (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }

  const button = target.closest<HTMLButtonElement>("[data-push-to-talk-button]");
  if (!button || button.disabled || event.button !== 0) {
    return;
  }

  event.preventDefault();
  void beginPushToTalk("pointer");
});

window.addEventListener("pointerup", () => {
  void releasePushToTalk("pointer");
});

window.addEventListener("pointercancel", () => {
  void cancelPushToTalk();
});

window.addEventListener("blur", () => {
  void cancelPushToTalk();
});

window.addEventListener("keydown", (event) => {
  if (!isPushToTalkKeyEvent(event)) {
    return;
  }

  event.preventDefault();
  void beginPushToTalk("keyboard");
});

window.addEventListener("keyup", (event) => {
  if (!isPushToTalkKeyEvent(event)) {
    return;
  }

  event.preventDefault();
  void releasePushToTalk("keyboard");
});

export { renderConfirmationPanel } from "./confirmation-panel";
