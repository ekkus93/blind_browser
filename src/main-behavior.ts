import type {
  AudioControlsPanelState,
  ConfirmationSettingsPanelState,
  ModelManagementPanelState,
  OcrThresholdSettingsPanelState,
  TtsModelPanelState,
  TtsProviderPanelState,
  TtsVoicePanelState,
} from "./confirmation-panel";

export interface SettingsBusyState {
  audioControls: Pick<AudioControlsPanelState, "isBusy">;
  confirmationSettings: Pick<ConfirmationSettingsPanelState, "isBusy">;
  ocrThresholdSettings: Pick<OcrThresholdSettingsPanelState, "isBusy">;
  asrProvider: { isBusy: boolean };
  modelManagement: Pick<ModelManagementPanelState, "isSaving" | "isDownloadingTts" | "isDownloadingAsr">;
  ttsProvider: Pick<TtsProviderPanelState, "isBusy">;
  ttsModel: Pick<TtsModelPanelState, "isBusy">;
  ttsVoice: Pick<TtsVoicePanelState, "isBusy">;
}

export type SettingsControl =
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
  | "tts-voice";

export function isSettingsControlBusy(control: SettingsControl, busyState: SettingsBusyState): boolean {
  switch (control) {
    case "volume":
    case "speed":
      return busyState.audioControls.isBusy;
    case "confirmation-threshold":
    case "click-without-confirmation":
      return busyState.confirmationSettings.isBusy;
    case "ocr-char-threshold":
    case "ocr-region-threshold":
      return busyState.ocrThresholdSettings.isBusy;
    case "model-check-on-startup":
    case "model-auto-download-missing":
    case "models-dir":
      return (
        busyState.modelManagement.isSaving
        || busyState.modelManagement.isDownloadingTts
        || busyState.modelManagement.isDownloadingAsr
      );
    case "asr-provider":
      return busyState.asrProvider.isBusy;
    case "tts-provider":
      return (
        busyState.ttsProvider.isBusy
        || busyState.ttsModel.isBusy
        || busyState.ttsVoice.isBusy
      );
    case "tts-model":
      return busyState.ttsProvider.isBusy || busyState.ttsModel.isBusy;
    case "tts-voice":
      return busyState.ttsProvider.isBusy || busyState.ttsVoice.isBusy;
  }
}

export function buildTtsProviderFailureRollbackState(
  previousProviderState: TtsProviderPanelState,
  previousModelState: TtsModelPanelState,
  previousVoiceState: TtsVoicePanelState,
  errorMessage: string,
): {
  provider: TtsProviderPanelState;
  model: TtsModelPanelState;
  voice: TtsVoicePanelState;
} {
  return {
    provider: {
      ...previousProviderState,
      isBusy: false,
      error: errorMessage,
    },
    model: {
      ...previousModelState,
      isBusy: false,
      error: errorMessage,
    },
    voice: {
      ...previousVoiceState,
      isBusy: false,
      error: errorMessage,
    },
  };
}

export function splitRuntimeRefreshResults<TAgentState, TModelSettings>(
  agentStateResult: PromiseSettledResult<TAgentState>,
  modelSettingsResult: PromiseSettledResult<TModelSettings>,
  describeFailure: (reason: unknown) => string,
): {
  agentState: TAgentState | null;
  agentStateError: string | null;
  modelSettings: TModelSettings | null;
  modelSettingsError: string | null;
} {
  return {
    agentState: agentStateResult.status === "fulfilled" ? agentStateResult.value : null,
    agentStateError:
      agentStateResult.status === "rejected" ? describeFailure(agentStateResult.reason) : null,
    modelSettings: modelSettingsResult.status === "fulfilled" ? modelSettingsResult.value : null,
    modelSettingsError:
      modelSettingsResult.status === "rejected"
        ? describeFailure(modelSettingsResult.reason)
        : null,
  };
}
