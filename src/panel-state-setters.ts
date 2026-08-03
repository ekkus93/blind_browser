import { appShellStore } from "./store";
import { classifyInvokeFailure } from "./api/errors";
import {
  setAppView as setAppShellView,
  setSettingsView as setAppShellSettingsView,
  setAppAlertPanelState as setAppAlertPanelStoreState,
  setAsrProviderPanelState as setAsrProviderPanelStoreState,
  setAudioControlsPanelState as setAudioControlsPanelStoreState,
  setConfirmationSettingsPanelState as setConfirmationSettingsPanelStoreState,
  setLocalAsrModelPanelState as setLocalAsrModelPanelStoreState,
  setLocalTtsModelPanelState as setLocalTtsModelPanelStoreState,
  setModelManagementPanelState as setModelManagementPanelStoreState,
  setOcrThresholdSettingsPanelState as setOcrThresholdSettingsPanelStoreState,
  setProviderFailoverPanelState as setProviderFailoverPanelStoreState,
  setPushToTalkPanelState as setPushToTalkPanelStoreState,
  setRemoteAsrPanelState as setRemoteAsrPanelStoreState,
  setRemotePlannerPanelState as setRemotePlannerPanelStoreState,
  setRemoteTtsPanelState as setRemoteTtsPanelStoreState,
  setStatusPanelState as setStatusPanelStoreState,
  setTtsModelPanelState as setTtsModelPanelStoreState,
  setTtsProviderPanelState as setTtsProviderPanelStoreState,
  setTtsVoicePanelState as setTtsVoicePanelStoreState,
  setUrlInputPanelState as setUrlInputPanelStoreState,
} from "./app-shell-store";
import { openExternalUrl as openExternalUrlDefault } from "./tauri-api";

let openExternalUrlImpl: typeof openExternalUrlDefault = openExternalUrlDefault;

export function setOpenExternalUrlForTest(next: typeof openExternalUrlDefault): () => void {
  openExternalUrlImpl = next;
  return () => {
    openExternalUrlImpl = openExternalUrlDefault;
  };
}
import type { AppView, SettingsView } from "./app-shell";
import type {
  AppAlertState,
  AsrProviderPanelState,
  AudioControlsPanelState,
  ConfirmationSettingsPanelState,
  LocalAsrModelPanelState,
  LocalTtsModelPanelState,
  ModelManagementPanelState,
  OcrThresholdSettingsPanelState,
  ProviderFailoverPanelState,
  PushToTalkPanelState,
  RemoteAsrPanelState,
  RemotePlannerPanelState,
  RemoteTtsPanelState,
  StatusPanelState,
  TtsModelPanelState,
  TtsProviderPanelState,
  TtsVoicePanelState,
  UrlInputPanelState,
} from "./confirmation-panel";

export function setAppView(nextView: AppView) {
  appShellStore.dispatch(setAppShellView(nextView));
}

export function setSettingsView(nextView: SettingsView) {
  appShellStore.dispatch(setAppShellSettingsView(nextView));
}

export function focusSettingsTarget(targetId: string) {
  setAppView("settings");
  setSettingsView(targetId as SettingsView);
}

function safeExternalLinkDisplay(url: string): string | null {
  try {
    const parsed = new URL(url);
    if (parsed.protocol !== "https:" || parsed.hostname.length === 0) {
      return null;
    }
    parsed.username = "";
    parsed.password = "";
    parsed.search = "";
    parsed.hash = "";
    return parsed.toString();
  } catch {
    return null;
  }
}

function describeExternalLinkFailure(url: string, error: unknown): string {
  const failure = classifyInvokeFailure(error);
  const failureMessage = failure.kind === "tool-error" ? failure.toolError.message : failure.message;
  const safeUrl = safeExternalLinkDisplay(url);
  if (safeUrl === null) {
    return `Could not open the external link because the URL was invalid. ${failureMessage}`;
  }
  return `Could not open the external link. Copy this URL and open it manually: ${safeUrl}. ${failureMessage}`;
}

export function setAppAlertState(nextState: Partial<AppAlertState>) {
  appShellStore.dispatch(setAppAlertPanelStoreState(nextState));
}

export function clearAppAlert() {
  setAppAlertState({ kind: "info", message: null });
}

export function openExternalLink(url: string) {
  void openExternalUrlImpl({ url }).catch((error) => {
    const safeFailure = classifyInvokeFailure(error);
    console.error("Failed to open external link.", safeFailure);
    setAppAlertState({
      kind: "error",
      message: describeExternalLinkFailure(url, error),
    });
  });
}

export function setPushToTalkState(nextState: Partial<PushToTalkPanelState>) {
  appShellStore.dispatch(setPushToTalkPanelStoreState(nextState));
}

export function setAudioControlsState(nextState: Partial<AudioControlsPanelState>) {
  appShellStore.dispatch(setAudioControlsPanelStoreState(nextState));
}

export function setRemotePlannerPanelState(nextState: Partial<RemotePlannerPanelState>) {
  appShellStore.dispatch(setRemotePlannerPanelStoreState(nextState));
}

export function setProviderFailoverPanelState(nextState: Partial<ProviderFailoverPanelState>) {
  appShellStore.dispatch(setProviderFailoverPanelStoreState(nextState));
}

export function setConfirmationSettingsPanelState(nextState: Partial<ConfirmationSettingsPanelState>) {
  appShellStore.dispatch(setConfirmationSettingsPanelStoreState(nextState));
}

export function setOcrThresholdSettingsPanelState(nextState: Partial<OcrThresholdSettingsPanelState>) {
  appShellStore.dispatch(setOcrThresholdSettingsPanelStoreState(nextState));
}

export function setAsrProviderPanelState(nextState: Partial<AsrProviderPanelState>) {
  appShellStore.dispatch(setAsrProviderPanelStoreState(nextState));
}

export function setLocalAsrModelPanelState(nextState: Partial<LocalAsrModelPanelState>) {
  appShellStore.dispatch(setLocalAsrModelPanelStoreState(nextState));
}

export function setRemoteAsrPanelState(nextState: Partial<RemoteAsrPanelState>) {
  appShellStore.dispatch(setRemoteAsrPanelStoreState(nextState));
}

export function setModelManagementPanelState(nextState: Partial<ModelManagementPanelState>) {
  appShellStore.dispatch(setModelManagementPanelStoreState(nextState));
}

export function setTtsProviderPanelState(nextState: Partial<TtsProviderPanelState>) {
  appShellStore.dispatch(setTtsProviderPanelStoreState(nextState));
}

export function setTtsModelPanelState(nextState: Partial<TtsModelPanelState>) {
  appShellStore.dispatch(setTtsModelPanelStoreState(nextState));
}

export function setLocalTtsModelPanelState(nextState: Partial<LocalTtsModelPanelState>) {
  appShellStore.dispatch(setLocalTtsModelPanelStoreState(nextState));
}

export function setRemoteTtsPanelState(nextState: Partial<RemoteTtsPanelState>) {
  appShellStore.dispatch(setRemoteTtsPanelStoreState(nextState));
}

export function setTtsVoicePanelState(nextState: Partial<TtsVoicePanelState>) {
  appShellStore.dispatch(setTtsVoicePanelStoreState(nextState));
}

export function setStatusPanelState(nextState: Partial<StatusPanelState>) {
  appShellStore.dispatch(setStatusPanelStoreState(nextState));
}

export function setUrlInputPanelState(nextState: Partial<UrlInputPanelState>) {
  appShellStore.dispatch(setUrlInputPanelStoreState(nextState));
}

export function createRequestId(prefix: string): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return `${prefix}-${crypto.randomUUID()}`;
  }

  return `${prefix}-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}
