import type { AppShellState } from "./app-shell-store";
import type { SettingsCardStatus, SettingsStatuses } from "./app-shell";

export function deriveSettingsStatuses(panelStates: AppShellState["panelStates"]): SettingsStatuses {
  const plannerStatus: SettingsCardStatus =
    panelStates.remotePlannerPanelState.model && panelStates.remotePlannerPanelState.baseUrl
      ? panelStates.remotePlannerPanelState.apiKeyReference
        ? "ok"
        : "warning"
      : "unconfigured";

  const ttsStatus: SettingsCardStatus = panelStates.ttsProviderPanelState.activeMode === "Local"
    ? panelStates.modelManagementPanelState.localTtsAvailable ? "ok" : "warning"
    : panelStates.remoteTtsPanelState.apiKeyReference ? "ok" : "unconfigured";

  const asrStatus: SettingsCardStatus = panelStates.asrProviderPanelState.activeMode === "Local"
    ? panelStates.modelManagementPanelState.localAsrAvailable ? "ok" : "warning"
    : panelStates.remoteAsrPanelState.apiKeyReference ? "ok" : "unconfigured";

  const runtimeStatus: SettingsCardStatus =
    panelStates.modelManagementPanelState.localTtsAvailable && panelStates.modelManagementPanelState.localAsrAvailable
      ? "ok"
      : "warning";

  return { planner: plannerStatus, tts: ttsStatus, asr: asrStatus, runtime: runtimeStatus };
}
