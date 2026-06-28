import "./styles.css";

import { createElement } from "react";
import { createRoot } from "react-dom/client";
import { Provider } from "react-redux";
import { appShellStore } from "./store";
import {
  createRequestId,
  setAsrProviderPanelState,
  setAudioControlsState,
  setConfirmationSettingsPanelState,
  setLocalAsrModelPanelState,
  setLocalTtsModelPanelState,
  setModelManagementPanelState,
  setOcrThresholdSettingsPanelState,
  setProviderFailoverPanelState,
  setPushToTalkState,
  setRemoteAsrPanelState,
  setRemotePlannerPanelState,
  setRemoteTtsPanelState,
  setStatusPanelState,
  setTtsModelPanelState,
  setTtsProviderPanelState,
  setTtsVoicePanelState,
  setUrlInputPanelState,
} from "./panel-state-setters";
import {
  describeAudioControlFailure,
  describeScopedRuntimeRefreshFailure,
} from "./main-errors";
import { registerShellEventHandlers } from "./shell-event-handlers";
import { createRuntimeRefreshHandlers } from "./runtime-refresh";
import { setRuntimeRefreshHandle } from "./refresh-handle";
import { ensureContinuousListeningLoop } from "./voice-loop";
import { BlindBrowserApp } from "./app.tsx";

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

if (!app) {
  throw new Error("App root element was not found.");
}
const appRoot: HTMLDivElement = app;
const h = createElement;
const runtimeRoot = createRoot(appRoot);

const { refreshRuntimePanelsFromRuntime } = createRuntimeRefreshHandlers({
  createRequestId,
  describeAudioControlFailure,
  describeScopedRuntimeRefreshFailure,
  ensureContinuousListeningLoop,
  getPanelStates: () => appShellStore.getState().panelStates,
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

setRuntimeRefreshHandle(refreshRuntimePanelsFromRuntime);

runtimeRoot.render(h(Provider, { store: appShellStore, children: h(BlindBrowserApp) }));

void refreshRuntimePanelsFromRuntime();
registerShellEventHandlers(appRoot);

export { renderConfirmationPanelNode } from "./confirmation-panel";
