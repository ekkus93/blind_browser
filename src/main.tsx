import "./styles.css";

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
import {
  remotePlannerPrivacyRefreshFailed,
  remotePlannerPrivacyRefreshStarted,
  remotePlannerPrivacyRefreshSucceeded,
} from "./remote-planner-privacy-state";
import { registerShellEventHandlers } from "./shell-event-handlers";
import { createRuntimeRefreshHandlersWithPrivacy } from "./runtime-refresh-with-privacy";
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
const runtimeRoot = createRoot(appRoot);

const { refreshRuntimePanelsFromRuntime } = createRuntimeRefreshHandlersWithPrivacy({
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
  markRemotePlannerPrivacyRefreshStarted: () => {
    appShellStore.dispatch(remotePlannerPrivacyRefreshStarted());
  },
  applyRemotePlannerPrivacyRefreshSuccess: (status) => {
    appShellStore.dispatch(remotePlannerPrivacyRefreshSucceeded(status));
  },
  applyRemotePlannerPrivacyRefreshFailure: (message) => {
    appShellStore.dispatch(remotePlannerPrivacyRefreshFailed(message));
  },
});

setRuntimeRefreshHandle(refreshRuntimePanelsFromRuntime);

runtimeRoot.render(
  <Provider store={appShellStore}>
    <BlindBrowserApp />
  </Provider>,
);

void refreshRuntimePanelsFromRuntime();
registerShellEventHandlers(appRoot);

export { renderConfirmationPanelNode } from "./confirmation-panel";
