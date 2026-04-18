import "./styles.css";

import {
  createPanelRoots,
  renderPanelRootNode,
  setActiveAppView,
  setActiveSettingsView,
  type AppView,
  type PanelRootKey,
  type PanelRootMap,
  type SettingsView,
} from "./app-shell";
import type { ReactNode } from "react";
import {
  createAppShellStore,
  setAppView as setAppShellView,
  setSettingsView as setAppShellSettingsView,
} from "./app-shell-store";
import { createInitialPanelStates } from "./panel-state";
import {
  renderAudioControlsPanelNode,
  renderConfirmationPanelNode,
  renderPushToTalkPanelNode,
  renderSettingsAsrProviderPanelNode,
  renderSettingsConfirmationPanelNode,
  renderSettingsGuidancePanelNode,
  renderSettingsLocalAsrModelPanelNode,
  renderSettingsLocalTtsModelPanelNode,
  renderSettingsModelManagementPanelNode,
  renderSettingsOcrThresholdPanelNode,
  renderSettingsProviderFailoverPanelNode,
  renderSettingsRemoteAsrPanelNode,
  renderSettingsRemotePlannerPanelNode,
  renderSettingsRemoteTtsPanelNode,
  renderSettingsTtsProviderPanelNode,
  renderSettingsTtsModelPanelNode,
  renderSettingsTtsVoicePanelNode,
  renderStatusPanelNode,
  renderUrlInputPanelNode,
  type AudioControlsPanelState,
  type AsrProviderPanelState,
  type ConfirmationSettingsPanelState,
  type LocalAsrModelPanelState,
  type LocalTtsModelPanelState,
  type ModelManagementPanelState,
  type OcrThresholdSettingsPanelState,
  type ProviderFailoverPanelState,
  type RemoteAsrPanelState,
  type RemotePlannerPanelState,
  type RemoteTtsPanelState,
  type PushToTalkPanelState,
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
} from "./planner-orchestration";
import {
  buildTtsProviderFailureRollbackState,
  isSettingsControlBusy,
} from "./main-behavior";
import {
  currentSettingsGuidanceState,
  describeAudioControlFailure,
  describePlannerBlockedMessage,
  describePushToTalkFailure,
  describeScopedRuntimeRefreshFailure,
  describeUrlInputFailure,
} from "./main-errors";
import { registerAppEventHandlers } from "./event-handlers";
import {
  createRuntimeRefreshHandlers,
} from "./runtime-refresh";
import {
  openExternalUrl,
  openUrl,
  resolveCommand,
  downloadActiveLocalAsrModel,
  downloadActiveLocalTtsModel,
  setAllowClickWithoutConfirmation,
  setModelManagementSettings,
  setRemoteAsrApiKey,
  setRemotePlannerApiKey,
  setRemotePlannerConnectionSettings,
  setRemoteTtsApiKey,
  testRemoteAsrApiKey,
  listRemotePlannerModels,
  resetRemotePlannerConnectionSettings,
  testRemotePlannerApiKey,
  testRemoteTtsApiKey,
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
const initialPanelStates = createInitialPanelStates();
const appShellStore = createAppShellStore();
let currentExecutionUiState = uiStore.getState();
let pushToTalkState: PushToTalkPanelState = initialPanelStates.pushToTalkState;
let audioControlsState: AudioControlsPanelState = initialPanelStates.audioControlsState;
let remotePlannerPanelState: RemotePlannerPanelState = initialPanelStates.remotePlannerPanelState;
let providerFailoverPanelState: ProviderFailoverPanelState = initialPanelStates.providerFailoverPanelState;
let confirmationSettingsPanelState: ConfirmationSettingsPanelState =
  initialPanelStates.confirmationSettingsPanelState;
let ocrThresholdSettingsPanelState: OcrThresholdSettingsPanelState =
  initialPanelStates.ocrThresholdSettingsPanelState;
let asrProviderPanelState: AsrProviderPanelState = initialPanelStates.asrProviderPanelState;
let localAsrModelPanelState: LocalAsrModelPanelState = initialPanelStates.localAsrModelPanelState;
let modelManagementPanelState: ModelManagementPanelState =
  initialPanelStates.modelManagementPanelState;
let remoteAsrPanelState: RemoteAsrPanelState = initialPanelStates.remoteAsrPanelState;
let ttsProviderPanelState: TtsProviderPanelState = initialPanelStates.ttsProviderPanelState;
let ttsModelPanelState: TtsModelPanelState = initialPanelStates.ttsModelPanelState;
let localTtsModelPanelState: LocalTtsModelPanelState = initialPanelStates.localTtsModelPanelState;
let remoteTtsPanelState: RemoteTtsPanelState = initialPanelStates.remoteTtsPanelState;
let ttsVoicePanelState: TtsVoicePanelState = initialPanelStates.ttsVoicePanelState;
let statusPanelState: StatusPanelState = initialPanelStates.statusPanelState;
let urlInputPanelState: UrlInputPanelState = initialPanelStates.urlInputPanelState;
let activePushToTalkSource: "keyboard" | "pointer" | null = null;
let continuousListeningLoopActive = false;
let currentShellViewState = appShellStore.getState();

if (!app) {
  throw new Error("App root element was not found.");
}
const appRoot: HTMLDivElement = app;

let panelRoots: PanelRootMap | null = null;
function ensurePanelRoots(): PanelRootMap {
  if (panelRoots) {
    return panelRoots;
  }

  panelRoots = createPanelRoots(appRoot);
  setActiveAppView(appRoot, currentShellViewState.appView);
  setActiveSettingsView(appRoot, currentShellViewState.settingsView);

  return panelRoots;
}

function setAppView(nextView: AppView) {
  appShellStore.dispatch(setAppShellView(nextView));
}

function setSettingsView(nextView: SettingsView) {
  appShellStore.dispatch(setAppShellSettingsView(nextView));
}

appShellStore.subscribe(() => {
  const nextShellViewState = appShellStore.getState();
  if (
    nextShellViewState.appView === currentShellViewState.appView
    && nextShellViewState.settingsView === currentShellViewState.settingsView
  ) {
    return;
  }

  currentShellViewState = nextShellViewState;
  if (!panelRoots) {
    return;
  }

  setActiveAppView(appRoot, currentShellViewState.appView);
  setActiveSettingsView(appRoot, currentShellViewState.settingsView);
});

function renderPanelRoot(rootKey: PanelRootKey, node: ReactNode) {
  renderPanelRootNode(ensurePanelRoots(), rootKey, node);
}

function rerenderSettingsGuidancePanel() {
  renderPanelRoot(
    "settings-guidance",
    renderSettingsGuidancePanelNode(
      currentSettingsGuidanceState({
        pushToTalkState,
        audioControlsState,
        remotePlannerPanelState,
        providerFailoverPanelState,
        confirmationSettingsPanelState,
        ocrThresholdSettingsPanelState,
        asrProviderPanelState,
        localAsrModelPanelState,
        modelManagementPanelState,
        remoteAsrPanelState,
        ttsProviderPanelState,
        ttsModelPanelState,
        localTtsModelPanelState,
        remoteTtsPanelState,
        ttsVoicePanelState,
        statusPanelState,
        urlInputPanelState,
      }),
    ),
  );
}

function rerenderPushToTalkPanel() {
  renderPanelRoot("push-to-talk", renderPushToTalkPanelNode(pushToTalkState));
  rerenderSettingsGuidancePanel();
}

function rerenderAudioPanels() {
  renderPanelRoot("audio-controls", renderAudioControlsPanelNode(audioControlsState));
}

function rerenderRemotePlannerPanel() {
  renderPanelRoot("settings-remote-planner", renderSettingsRemotePlannerPanelNode(remotePlannerPanelState));
  rerenderSettingsGuidancePanel();
}

function rerenderProviderFailoverPanel() {
  renderPanelRoot(
    "settings-provider-failover",
    renderSettingsProviderFailoverPanelNode(providerFailoverPanelState),
  );
}

function rerenderConfirmationSettingsPanel() {
  renderPanelRoot(
    "settings-confirmation",
    renderSettingsConfirmationPanelNode(confirmationSettingsPanelState),
  );
}

function rerenderOcrThresholdPanel() {
  renderPanelRoot(
    "settings-ocr-threshold",
    renderSettingsOcrThresholdPanelNode(ocrThresholdSettingsPanelState),
  );
}

function rerenderAsrPanels() {
  renderPanelRoot("settings-asr-provider", renderSettingsAsrProviderPanelNode(asrProviderPanelState));
  renderPanelRoot(
    "settings-local-asr-model",
    renderSettingsLocalAsrModelPanelNode(localAsrModelPanelState),
  );
  renderPanelRoot("settings-remote-asr", renderSettingsRemoteAsrPanelNode(remoteAsrPanelState));
  rerenderSettingsGuidancePanel();
}

function rerenderModelManagementPanel() {
  renderPanelRoot(
    "settings-model-management",
    renderSettingsModelManagementPanelNode(modelManagementPanelState),
  );
  rerenderSettingsGuidancePanel();
}

function rerenderTtsPanels() {
  renderPanelRoot("settings-tts-provider", renderSettingsTtsProviderPanelNode(ttsProviderPanelState));
  renderPanelRoot("settings-tts-model", renderSettingsTtsModelPanelNode(ttsModelPanelState));
  renderPanelRoot(
    "settings-local-tts-model",
    renderSettingsLocalTtsModelPanelNode(localTtsModelPanelState),
  );
  renderPanelRoot("settings-remote-tts", renderSettingsRemoteTtsPanelNode(remoteTtsPanelState));
  renderPanelRoot("settings-tts-voice", renderSettingsTtsVoicePanelNode(ttsVoicePanelState));
  rerenderSettingsGuidancePanel();
}

function rerenderStatusPanel() {
  renderPanelRoot("status", renderStatusPanelNode(statusPanelState));
  rerenderSettingsGuidancePanel();
}

function rerenderUrlInputPanel() {
  renderPanelRoot("url-input", renderUrlInputPanelNode(urlInputPanelState));
  rerenderSettingsGuidancePanel();
}

function rerenderConfirmationPanel() {
  renderPanelRoot("confirmation-panel", renderConfirmationPanelNode(currentExecutionUiState.confirmation));
}

function rerender() {
  ensurePanelRoots();
  rerenderPushToTalkPanel();
  rerenderUrlInputPanel();
  rerenderStatusPanel();
  rerenderAudioPanels();
  rerenderRemotePlannerPanel();
  rerenderProviderFailoverPanel();
  rerenderConfirmationSettingsPanel();
  rerenderOcrThresholdPanel();
  rerenderAsrPanels();
  rerenderModelManagementPanel();
  rerenderTtsPanels();
  rerenderConfirmationPanel();
}

function setPushToTalkState(nextState: Partial<PushToTalkPanelState>) {
  pushToTalkState = {
    ...pushToTalkState,
    ...nextState,
  };
  rerenderPushToTalkPanel();
}

function setAudioControlsState(nextState: Partial<AudioControlsPanelState>) {
  audioControlsState = {
    ...audioControlsState,
    ...nextState,
  };
  rerenderAudioPanels();
}

function setRemotePlannerPanelState(nextState: Partial<RemotePlannerPanelState>) {
  remotePlannerPanelState = {
    ...remotePlannerPanelState,
    ...nextState,
  };
  rerenderRemotePlannerPanel();
}

function setProviderFailoverPanelState(nextState: Partial<ProviderFailoverPanelState>) {
  providerFailoverPanelState = {
    ...providerFailoverPanelState,
    ...nextState,
  };
  rerenderProviderFailoverPanel();
}

function setConfirmationSettingsPanelState(nextState: Partial<ConfirmationSettingsPanelState>) {
  confirmationSettingsPanelState = {
    ...confirmationSettingsPanelState,
    ...nextState,
  };
  rerenderConfirmationSettingsPanel();
}

function setOcrThresholdSettingsPanelState(nextState: Partial<OcrThresholdSettingsPanelState>) {
  ocrThresholdSettingsPanelState = {
    ...ocrThresholdSettingsPanelState,
    ...nextState,
  };
  rerenderOcrThresholdPanel();
}

function setAsrProviderPanelState(nextState: Partial<AsrProviderPanelState>) {
  asrProviderPanelState = {
    ...asrProviderPanelState,
    ...nextState,
  };
  rerenderAsrPanels();
}

function setLocalAsrModelPanelState(nextState: Partial<LocalAsrModelPanelState>) {
  localAsrModelPanelState = {
    ...localAsrModelPanelState,
    ...nextState,
  };
  rerenderAsrPanels();
}

function setRemoteAsrPanelState(nextState: Partial<RemoteAsrPanelState>) {
  remoteAsrPanelState = {
    ...remoteAsrPanelState,
    ...nextState,
  };
  rerenderAsrPanels();
}

function setModelManagementPanelState(nextState: Partial<ModelManagementPanelState>) {
  modelManagementPanelState = {
    ...modelManagementPanelState,
    ...nextState,
  };
  rerenderModelManagementPanel();
}

function setTtsProviderPanelState(nextState: Partial<TtsProviderPanelState>) {
  ttsProviderPanelState = {
    ...ttsProviderPanelState,
    ...nextState,
  };
  rerenderTtsPanels();
}

function setTtsModelPanelState(nextState: Partial<TtsModelPanelState>) {
  ttsModelPanelState = {
    ...ttsModelPanelState,
    ...nextState,
  };
  rerenderTtsPanels();
}

function setLocalTtsModelPanelState(nextState: Partial<LocalTtsModelPanelState>) {
  localTtsModelPanelState = {
    ...localTtsModelPanelState,
    ...nextState,
  };
  rerenderTtsPanels();
}

function setRemoteTtsPanelState(nextState: Partial<RemoteTtsPanelState>) {
  remoteTtsPanelState = {
    ...remoteTtsPanelState,
    ...nextState,
  };
  rerenderTtsPanels();
}

function setTtsVoicePanelState(nextState: Partial<TtsVoicePanelState>) {
  ttsVoicePanelState = {
    ...ttsVoicePanelState,
    ...nextState,
  };
  rerenderTtsPanels();
}

function setStatusPanelState(nextState: Partial<StatusPanelState>) {
  statusPanelState = {
    ...statusPanelState,
    ...nextState,
  };
  rerenderStatusPanel();
}

function setUrlInputPanelState(nextState: Partial<UrlInputPanelState>) {
  urlInputPanelState = {
    ...urlInputPanelState,
    ...nextState,
  };
  rerenderUrlInputPanel();
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

function isSettingsActionBusy(
  control:
    | "volume"
    | "speed"
    | "confirmation-threshold"
    | "click-without-confirmation"
    | "ocr-char-threshold"
    | "ocr-region-threshold"
    | "model-check-on-startup"
    | "model-auto-download-missing"
    | "models-dir"
    | "asr-provider"
    | "tts-provider"
    | "tts-model"
    | "tts-voice",
): boolean {
  return isSettingsControlBusy(control, {
    audioControls: audioControlsState,
    confirmationSettings: confirmationSettingsPanelState,
    ocrThresholdSettings: ocrThresholdSettingsPanelState,
    asrProvider: asrProviderPanelState,
    modelManagement: modelManagementPanelState,
    ttsProvider: ttsProviderPanelState,
    ttsModel: ttsModelPanelState,
    ttsVoice: ttsVoicePanelState,
  });
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

const { refreshRuntimePanelsFromRuntime } = createRuntimeRefreshHandlers({
  createRequestId,
  describeAudioControlFailure,
  describeScopedRuntimeRefreshFailure,
  ensureContinuousListeningLoop,
  getPanelStates: () => ({
    pushToTalkState,
    audioControlsState,
    remotePlannerPanelState,
    providerFailoverPanelState,
    confirmationSettingsPanelState,
    ocrThresholdSettingsPanelState,
    asrProviderPanelState,
    localAsrModelPanelState,
    modelManagementPanelState,
    remoteAsrPanelState,
    ttsProviderPanelState,
    ttsModelPanelState,
    localTtsModelPanelState,
    remoteTtsPanelState,
    ttsVoicePanelState,
    statusPanelState,
    urlInputPanelState,
  }),
  setPushToTalkState,
  setAudioControlsState,
  setRemotePlannerPanelState,
  setProviderFailoverPanelState,
  setConfirmationSettingsPanelState,
  setOcrThresholdSettingsPanelState,
  setAsrProviderPanelState,
  setLocalAsrModelPanelState,
  setModelManagementPanelState,
  setRemoteAsrPanelState,
  setTtsProviderPanelState,
  setTtsModelPanelState,
  setLocalTtsModelPanelState,
  setRemoteTtsPanelState,
  setTtsVoicePanelState,
  setStatusPanelState,
  setUrlInputPanelState,
});

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
  if (isSettingsActionBusy("volume")) {
    return;
  }

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
  if (isSettingsActionBusy("speed")) {
    return;
  }

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
  if (isSettingsActionBusy("asr-provider")) {
    return;
  }

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
  if (isSettingsActionBusy("tts-provider")) {
    return;
  }

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
    const rollbackState = buildTtsProviderFailureRollbackState(
      previousProviderState,
      previousModelState,
      previousVoiceState,
      message,
    );
    setTtsProviderPanelState(rollbackState.provider);
    setTtsModelPanelState(rollbackState.model);
    setTtsVoicePanelState(rollbackState.voice);
  }
}

async function persistTtsModelSelection(nextProfileName: string) {
  if (isSettingsActionBusy("tts-model")) {
    return;
  }

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
  if (isSettingsActionBusy("tts-voice")) {
    return;
  }

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
  if (isSettingsActionBusy("confirmation-threshold")) {
    return;
  }

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
  if (isSettingsActionBusy("click-without-confirmation")) {
    return;
  }

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
  if (isSettingsActionBusy("ocr-char-threshold")) {
    return;
  }

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

async function persistModelManagementSettings() {
  if (isSettingsActionBusy("models-dir")) {
    return;
  }

  const modelsDir = modelManagementPanelState.modelsDir.trim();
  if (modelsDir.length === 0) {
    setModelManagementPanelState({
      error: "Enter a models directory before saving model management settings.",
    });
    return;
  }

  const previousState = modelManagementPanelState;
  setModelManagementPanelState({
    isSaving: true,
    error: null,
  });

  try {
    const result = await setModelManagementSettings({
      requestId: createRequestId("model-management-settings"),
      modelsDir,
      checkOnStartup: modelManagementPanelState.checkOnStartup,
      autoDownloadMissing: modelManagementPanelState.autoDownloadMissing,
    });
    setModelManagementPanelState({
      modelsDir: result.models_dir,
      checkOnStartup: result.check_on_startup,
      autoDownloadMissing: result.auto_download_missing,
      isSaving: false,
      error: null,
    });
    await refreshRuntimePanelsFromRuntime();
  } catch (error: unknown) {
    setModelManagementPanelState({
      ...previousState,
      isSaving: false,
      error: describeAudioControlFailure(error),
    });
  }
}

async function downloadManagedLocalTtsModel() {
  if (modelManagementPanelState.isDownloadingTts) {
    return;
  }

  setModelManagementPanelState({
    isDownloadingTts: true,
    error: null,
  });

  try {
    await downloadActiveLocalTtsModel({
      requestId: createRequestId("download-local-tts-model"),
    });
    setModelManagementPanelState({
      isDownloadingTts: false,
      error: null,
    });
    await refreshRuntimePanelsFromRuntime();
  } catch (error: unknown) {
    setModelManagementPanelState({
      isDownloadingTts: false,
      error: describeAudioControlFailure(error),
    });
  }
}

async function downloadManagedLocalAsrModel() {
  if (modelManagementPanelState.isDownloadingAsr) {
    return;
  }

  setModelManagementPanelState({
    isDownloadingAsr: true,
    error: null,
  });

  try {
    await downloadActiveLocalAsrModel({
      requestId: createRequestId("download-local-asr-model"),
    });
    setModelManagementPanelState({
      isDownloadingAsr: false,
      error: null,
    });
    await refreshRuntimePanelsFromRuntime();
  } catch (error: unknown) {
    setModelManagementPanelState({
      isDownloadingAsr: false,
      error: describeAudioControlFailure(error),
    });
  }
}

async function persistRemotePlannerApiKey() {
  const profileName = remotePlannerPanelState.profileName;
  const apiKey = remotePlannerPanelState.apiKeyDraft.trim();
  if (!profileName) {
    setRemotePlannerPanelState({
      error: "No remote planner profile is configured for secure API key entry.",
    });
    return;
  }
  if (apiKey.length === 0) {
    setRemotePlannerPanelState({
      error: "Enter a remote planner API key before saving.",
    });
    return;
  }

  setRemotePlannerPanelState({
    isSavingApiKey: true,
    apiKeyTestMessage: null,
    error: null,
  });

  try {
    const result = await setRemotePlannerApiKey({
      requestId: createRequestId("remote-planner-api-key"),
      profileName,
      apiKey,
    });
    setRemotePlannerPanelState({
      profileName: result.profile_name,
      apiKeyReference: result.api_key_reference,
      apiKeyDraft: "",
      isSavingApiKey: false,
      apiKeyTestMessage: null,
      error: null,
    });
    await refreshRuntimePanelsFromRuntime();
  } catch (error: unknown) {
    setRemotePlannerPanelState({
      isSavingApiKey: false,
      error: describeAudioControlFailure(error),
    });
  }
}

async function loadRemotePlannerModels() {
  const profileName = remotePlannerPanelState.profileName;
  const baseUrl = remotePlannerPanelState.baseUrl?.trim() ?? "";
  const apiKey = remotePlannerPanelState.apiKeyDraft.trim();
  if (!profileName) {
    setRemotePlannerPanelState({
      error: "No remote planner profile is configured for model loading.",
    });
    return;
  }
  if (baseUrl.length === 0) {
    setRemotePlannerPanelState({
      error: "Enter an endpoint before loading models.",
    });
    return;
  }

  setRemotePlannerPanelState({
    isLoadingModels: true,
    error: null,
  });

  try {
    const result = await listRemotePlannerModels({
      requestId: createRequestId("list-remote-planner-models"),
      profileName,
      baseUrl,
      apiKey,
    });
    const nextModel = result.models.includes(remotePlannerPanelState.model ?? "")
      ? remotePlannerPanelState.model
      : (result.models[0] ?? null);
    setRemotePlannerPanelState({
      baseUrl: result.base_url,
      availableModels: result.models,
      loadedModelsEndpoint: result.base_url,
      model: nextModel,
      isLoadingModels: false,
      error: null,
    });
  } catch (error: unknown) {
    setRemotePlannerPanelState({
      isLoadingModels: false,
      error: describeAudioControlFailure(error),
    });
  }
}

async function persistRemotePlannerConnection() {
  const profileName = remotePlannerPanelState.profileName;
  const baseUrl = remotePlannerPanelState.baseUrl?.trim() ?? "";
  const model = remotePlannerPanelState.model?.trim() ?? "";
  if (!profileName) {
    setRemotePlannerPanelState({
      error: "No remote planner profile is configured for endpoint and model changes.",
    });
    return;
  }
  if (baseUrl.length === 0) {
    setRemotePlannerPanelState({
      error: "Enter an endpoint before saving the planner settings.",
    });
    return;
  }
  if (model.length === 0) {
    setRemotePlannerPanelState({
      error: "Choose a model before saving the planner settings.",
    });
    return;
  }
  if (remotePlannerPanelState.loadedModelsEndpoint !== baseUrl) {
    setRemotePlannerPanelState({
      error: "Load models for the current endpoint before saving the planner settings.",
    });
    return;
  }

  setRemotePlannerPanelState({
    isSavingConnection: true,
    error: null,
  });

  try {
    const result = await setRemotePlannerConnectionSettings({
      requestId: createRequestId("save-remote-planner-settings"),
      profileName,
      baseUrl,
      model,
    });
    setRemotePlannerPanelState({
      profileName: result.profile_name,
      baseUrl: result.base_url,
      model: result.model,
      availableModels: remotePlannerPanelState.availableModels.includes(result.model)
        ? remotePlannerPanelState.availableModels
        : [result.model, ...remotePlannerPanelState.availableModels],
      loadedModelsEndpoint: result.base_url,
      isSavingConnection: false,
      error: null,
    });
    await refreshRuntimePanelsFromRuntime();
  } catch (error: unknown) {
    setRemotePlannerPanelState({
      isSavingConnection: false,
      error: describeAudioControlFailure(error),
    });
  }
}

async function resetRemotePlannerConnectionToDefaults() {
  const profileName = remotePlannerPanelState.profileName;
  if (!profileName) {
    setRemotePlannerPanelState({
      error: "No remote planner profile is configured for reset.",
    });
    return;
  }

  setRemotePlannerPanelState({
    isResettingConnection: true,
    error: null,
  });

  try {
    const result = await resetRemotePlannerConnectionSettings({
      requestId: createRequestId("reset-remote-planner-settings"),
      profileName,
    });
    setRemotePlannerPanelState({
      profileName: result.profile_name,
      baseUrl: result.base_url,
      model: result.model,
      availableModels: [result.model],
      loadedModelsEndpoint: result.base_url,
      isResettingConnection: false,
      error: null,
    });
    await refreshRuntimePanelsFromRuntime();
  } catch (error: unknown) {
    setRemotePlannerPanelState({
      isResettingConnection: false,
      error: describeAudioControlFailure(error),
    });
  }
}

async function persistRemoteTtsApiKey() {
  const profileName = remoteTtsPanelState.profileName;
  const apiKey = remoteTtsPanelState.apiKeyDraft.trim();
  if (!profileName) {
    setRemoteTtsPanelState({
      error: "No remote TTS profile is configured for secure API key entry.",
    });
    return;
  }
  if (apiKey.length === 0) {
    setRemoteTtsPanelState({
      error: "Enter a remote TTS API key before saving.",
    });
    return;
  }

  setRemoteTtsPanelState({
    isSavingApiKey: true,
    apiKeyTestMessage: null,
    error: null,
  });

  try {
    const result = await setRemoteTtsApiKey({
      requestId: createRequestId("remote-tts-api-key"),
      profileName,
      apiKey,
    });
    setRemoteTtsPanelState({
      profileName: result.profile_name,
      apiKeyReference: result.api_key_reference,
      apiKeyDraft: "",
      isSavingApiKey: false,
      apiKeyTestMessage: null,
      error: null,
    });
    await refreshRuntimePanelsFromRuntime();
  } catch (error: unknown) {
    setRemoteTtsPanelState({
      isSavingApiKey: false,
      error: describeAudioControlFailure(error),
    });
  }
}

async function persistRemoteAsrApiKey() {
  const profileName = remoteAsrPanelState.profileName;
  const apiKey = remoteAsrPanelState.apiKeyDraft.trim();
  if (!profileName) {
    setRemoteAsrPanelState({
      error: "No remote ASR profile is configured for secure API key entry.",
    });
    return;
  }
  if (apiKey.length === 0) {
    setRemoteAsrPanelState({
      error: "Enter a remote ASR API key before saving.",
    });
    return;
  }

  setRemoteAsrPanelState({
    isSavingApiKey: true,
    apiKeyTestMessage: null,
    error: null,
  });

  try {
    const result = await setRemoteAsrApiKey({
      requestId: createRequestId("remote-asr-api-key"),
      profileName,
      apiKey,
    });
    setRemoteAsrPanelState({
      profileName: result.profile_name,
      apiKeyReference: result.api_key_reference,
      apiKeyDraft: "",
      isSavingApiKey: false,
      apiKeyTestMessage: null,
      error: null,
    });
    await refreshRuntimePanelsFromRuntime();
  } catch (error: unknown) {
    setRemoteAsrPanelState({
      isSavingApiKey: false,
      error: describeAudioControlFailure(error),
    });
  }
}

async function testConfiguredRemotePlannerApiKey() {
  const profileName = remotePlannerPanelState.profileName;
  const apiKey = remotePlannerPanelState.apiKeyDraft.trim();
  if (!profileName) {
    setRemotePlannerPanelState({
      error: "No remote planner profile is configured for API key testing.",
      apiKeyTestMessage: null,
    });
    return;
  }
  if (apiKey.length === 0 && !remotePlannerPanelState.apiKeyReference) {
    setRemotePlannerPanelState({
      error: "Enter a remote planner API key or save one before testing.",
      apiKeyTestMessage: null,
    });
    return;
  }

  setRemotePlannerPanelState({
    isTestingApiKey: true,
    apiKeyTestMessage: null,
    error: null,
  });

  try {
    const result = await testRemotePlannerApiKey({
      requestId: createRequestId("test-remote-planner-api-key"),
      profileName,
      apiKey,
    });
    setRemotePlannerPanelState({
      profileName: result.profile_name,
      isTestingApiKey: false,
      apiKeyTestMessage: result.message,
      error: null,
    });
  } catch (error: unknown) {
    setRemotePlannerPanelState({
      isTestingApiKey: false,
      apiKeyTestMessage: null,
      error: describeAudioControlFailure(error),
    });
  }
}

async function testConfiguredRemoteTtsApiKey() {
  const profileName = remoteTtsPanelState.profileName;
  const apiKey = remoteTtsPanelState.apiKeyDraft.trim();
  if (!profileName) {
    setRemoteTtsPanelState({
      error: "No remote TTS profile is configured for API key testing.",
      apiKeyTestMessage: null,
    });
    return;
  }
  if (apiKey.length === 0 && !remoteTtsPanelState.apiKeyReference) {
    setRemoteTtsPanelState({
      error: "Enter a remote TTS API key or save one before testing.",
      apiKeyTestMessage: null,
    });
    return;
  }

  setRemoteTtsPanelState({
    isTestingApiKey: true,
    apiKeyTestMessage: null,
    error: null,
  });

  try {
    const result = await testRemoteTtsApiKey({
      requestId: createRequestId("test-remote-tts-api-key"),
      profileName,
      apiKey,
    });
    setRemoteTtsPanelState({
      profileName: result.profile_name,
      isTestingApiKey: false,
      apiKeyTestMessage: result.message,
      error: null,
    });
  } catch (error: unknown) {
    setRemoteTtsPanelState({
      isTestingApiKey: false,
      apiKeyTestMessage: null,
      error: describeAudioControlFailure(error),
    });
  }
}

async function testConfiguredRemoteAsrApiKey() {
  const profileName = remoteAsrPanelState.profileName;
  const apiKey = remoteAsrPanelState.apiKeyDraft.trim();
  if (!profileName) {
    setRemoteAsrPanelState({
      error: "No remote ASR profile is configured for API key testing.",
      apiKeyTestMessage: null,
    });
    return;
  }
  if (apiKey.length === 0 && !remoteAsrPanelState.apiKeyReference) {
    setRemoteAsrPanelState({
      error: "Enter a remote ASR API key or save one before testing.",
      apiKeyTestMessage: null,
    });
    return;
  }

  setRemoteAsrPanelState({
    isTestingApiKey: true,
    apiKeyTestMessage: null,
    error: null,
  });

  try {
    const result = await testRemoteAsrApiKey({
      requestId: createRequestId("test-remote-asr-api-key"),
      profileName,
      apiKey,
    });
    setRemoteAsrPanelState({
      profileName: result.profile_name,
      isTestingApiKey: false,
      apiKeyTestMessage: result.message,
      error: null,
    });
  } catch (error: unknown) {
    setRemoteAsrPanelState({
      isTestingApiKey: false,
      apiKeyTestMessage: null,
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

function submitConfirmationAction(action: "approve" | "reject", confirmationId: string) {
  const confirmationState = uiStore.getState().confirmation;
  if (confirmationState.kind !== "awaiting-confirmation" || confirmationState.isSubmitting) {
    return;
  }

  if (confirmationId !== confirmationState.confirmationId) {
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
}

rerender();
void refreshRuntimePanelsFromRuntime();
uiStore.subscribe((uiState) => {
  currentExecutionUiState = uiState;
  rerenderConfirmationPanel();
});
registerAppEventHandlers({
  appRoot,
  document,
  window,
  isUrlInputActionBusy,
  isBrowserVisibilityUpdating: () => statusPanelState.isUpdatingVisibility,
  isSettingsActionBusy,
  isPushToTalkKeyEvent,
  saveRemoteApiKey: (kind) => {
    if (kind === "planner") {
      void persistRemotePlannerApiKey();
      return;
    }
    if (kind === "tts") {
      void persistRemoteTtsApiKey();
      return;
    }
    void persistRemoteAsrApiKey();
  },
  testRemoteApiKey: (kind) => {
    if (kind === "planner") {
      void testConfiguredRemotePlannerApiKey();
      return;
    }
    if (kind === "tts") {
      void testConfiguredRemoteTtsApiKey();
      return;
    }
    void testConfiguredRemoteAsrApiKey();
  },
  downloadModel: (kind) => {
    if (kind === "tts") {
      void downloadManagedLocalTtsModel();
      return;
    }
    void downloadManagedLocalAsrModel();
  },
  setBrowserVisibility: (mode) => {
    void persistBrowserVisibility(mode);
  },
  runUrlAction: (action) => {
    if (action === "open") {
      void openDraftUrl();
      return;
    }
    if (action === "read") {
      void readCurrentPage();
      return;
    }
    if (action === "stop") {
      void stopCurrentReading();
      return;
    }
    if (action === "previous") {
      void readPreviousRegion();
      return;
    }
    void readNextRegion();
  },
  submitConfirmationAction,
  updateAudioInput: (kind, value) => {
    if (kind === "volume") {
      setAudioControlsState({
        playbackVolume: value,
        error: null,
      });
      return;
    }
    setAudioControlsState({
      playbackSpeed: value,
      error: null,
    });
  },
  updateConfirmationThresholdInput: (value) => {
    setConfirmationSettingsPanelState({
      confirmationConfidenceThreshold: value,
      error: null,
    });
  },
  updateOcrThresholdInput: (kind, value) => {
    if (kind === "char") {
      setOcrThresholdSettingsPanelState({
        sparseTextCharThreshold: value,
        error: null,
      });
      return;
    }
    setOcrThresholdSettingsPanelState({
      sparseTextRegionThreshold: value,
      error: null,
    });
  },
  updateRemoteApiKeyInput: (kind, value) => {
    if (kind === "planner") {
      setRemotePlannerPanelState({
        apiKeyDraft: value,
        apiKeyTestMessage: null,
        error: null,
      });
      return;
    }
    if (kind === "tts") {
      setRemoteTtsPanelState({
        apiKeyDraft: value,
        apiKeyTestMessage: null,
        error: null,
      });
      return;
    }
    setRemoteAsrPanelState({
      apiKeyDraft: value,
      apiKeyTestMessage: null,
      error: null,
    });
  },
  updateRemotePlannerEndpointInput: (value) => {
    setRemotePlannerPanelState({
      baseUrl: value,
      availableModels: [],
      loadedModelsEndpoint: null,
      error: null,
    });
  },
  updateRemotePlannerModelSelection: (value) => {
    setRemotePlannerPanelState({
      model: value,
      error: null,
    });
  },
  updateModelManagementInput: (value) => {
    setModelManagementPanelState({
      modelsDir: value,
      error: null,
    });
  },
  updateUrlInput: (value) => {
    setUrlInputPanelState({
      draftValue: value,
      hasUnsubmittedChanges: true,
      error: null,
    });
  },
  persistAudioChange: (kind, value) => {
    if (kind === "volume") {
      void persistPlaybackVolume(value);
      return;
    }
    void persistPlaybackSpeed(value);
  },
  persistConfirmationThreshold: (value) => {
    void persistConfirmationThreshold(value);
  },
  persistClickWithoutConfirmation: (checked) => {
    void persistAllowClickWithoutConfirmation(checked);
  },
  persistOcrThresholdChange: (kind, value) => {
    if (kind === "char") {
      void persistOcrThresholds(value, ocrThresholdSettingsPanelState.sparseTextRegionThreshold);
      return;
    }
    void persistOcrThresholds(ocrThresholdSettingsPanelState.sparseTextCharThreshold, value);
  },
  persistModelManagementToggle: (kind, checked) => {
    if (kind === "check-on-startup") {
      setModelManagementPanelState({
        checkOnStartup: checked,
        error: null,
      });
    } else {
      setModelManagementPanelState({
        autoDownloadMissing: checked,
        error: null,
      });
    }
    void persistModelManagementSettings();
  },
  persistModelsDir: () => {
    void persistModelManagementSettings();
  },
  persistAsrProvider: (mode) => {
    void persistAsrProviderSelection(mode);
  },
  persistTtsProvider: (mode) => {
    void persistTtsProviderSelection(mode);
  },
  persistTtsModel: (profileName) => {
    void persistTtsModelSelection(profileName);
  },
  persistTtsVoice: (voice) => {
    void persistTtsVoiceSelection(voice);
  },
  loadRemotePlannerModels: () => {
    void loadRemotePlannerModels();
  },
  persistRemotePlannerConnectionSettings: () => {
    void persistRemotePlannerConnection();
  },
  resetRemotePlannerConnectionSettings: () => {
    void resetRemotePlannerConnectionToDefaults();
  },
  openExternalLink: (url) => {
    void openExternalUrl({ url }).catch((error) => {
      console.error("Failed to open external link.", error);
    });
  },
  setAppView,
  setSettingsView,
  beginPushToTalk: (source) => {
    void beginPushToTalk(source);
  },
  releasePushToTalk: (source) => {
    void releasePushToTalk(source);
  },
  cancelPushToTalk: () => {
    void cancelPushToTalk();
  },
});

export { renderConfirmationPanelNode } from "./confirmation-panel";
