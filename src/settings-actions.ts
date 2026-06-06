import { appShellStore } from "./store";
import {
  buildTtsProviderFailureRollbackState,
  isSettingsControlBusy,
  type SettingsControl,
} from "./main-behavior";
import { describeAudioControlFailure } from "./main-errors";
import {
  createRequestId,
  setAsrProviderPanelState,
  setAudioControlsState,
  setConfirmationSettingsPanelState,
  setModelManagementPanelState,
  setOcrThresholdSettingsPanelState,
  setStatusPanelState,
  setTtsModelPanelState,
  setTtsProviderPanelState,
  setTtsVoicePanelState,
} from "./panel-state-setters";
import { refreshRuntimePanels } from "./refresh-handle";
import {
  setAllowClickWithoutConfirmation,
  setAsrProviderSelection,
  setBrowserVisibility,
  setConfirmationThreshold,
  setModelManagementSettings,
  setOcrThresholds,
  setPlaybackSpeed,
  setPlaybackVolume,
  setTtsModelSelection,
  setTtsProviderSelection,
  setTtsVoice,
} from "./tauri-api";

function getPanelStates() {
  return appShellStore.getState().panelStates;
}

function isSettingsActionBusy(control: SettingsControl): boolean {
  const ps = getPanelStates();
  return isSettingsControlBusy(control, {
    audioControls: ps.audioControlsState,
    confirmationSettings: ps.confirmationSettingsPanelState,
    ocrThresholdSettings: ps.ocrThresholdSettingsPanelState,
    asrProvider: ps.asrProviderPanelState,
    modelManagement: ps.modelManagementPanelState,
    ttsProvider: ps.ttsProviderPanelState,
    ttsModel: ps.ttsModelPanelState,
    ttsVoice: ps.ttsVoicePanelState,
  });
}

export async function persistBrowserVisibility(nextMode: "Visible" | "Headless") {
  const previousMode = getPanelStates().statusPanelState.browserVisibility;
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
    await refreshRuntimePanels();
  } catch (error: unknown) {
    setStatusPanelState({
      browserVisibility: previousMode,
      isUpdatingVisibility: false,
      error: describeAudioControlFailure(error),
    });
  }
}

export async function persistPlaybackVolume(nextVolume: number) {
  if (isSettingsActionBusy("volume")) {
    return;
  }

  const previousState = getPanelStates().audioControlsState;
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

export async function persistPlaybackSpeed(nextSpeed: number) {
  if (isSettingsActionBusy("speed")) {
    return;
  }

  const previousState = getPanelStates().audioControlsState;
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

export async function persistAsrProviderSelection(nextMode: "Local" | "Remote") {
  if (isSettingsActionBusy("asr-provider")) {
    return;
  }

  const previousState = getPanelStates().asrProviderPanelState;
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
    await refreshRuntimePanels();
  } catch (error: unknown) {
    setAsrProviderPanelState({
      activeMode: previousState.activeMode,
      isBusy: false,
      error: describeAudioControlFailure(error),
    });
  }
}

export async function persistTtsProviderSelection(nextMode: "Local" | "Remote") {
  if (isSettingsActionBusy("tts-provider")) {
    return;
  }

  const previousProviderState = getPanelStates().ttsProviderPanelState;
  const previousModelState = getPanelStates().ttsModelPanelState;
  const previousVoiceState = getPanelStates().ttsVoicePanelState;
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
    await refreshRuntimePanels();
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

export async function persistTtsModelSelection(nextProfileName: string) {
  if (isSettingsActionBusy("tts-model")) {
    return;
  }

  const previousState = getPanelStates().ttsModelPanelState;
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
    await refreshRuntimePanels();
  } catch (error: unknown) {
    setTtsModelPanelState({
      activeProfile: previousState.activeProfile,
      isBusy: false,
      error: describeAudioControlFailure(error),
    });
  }
}

export async function persistTtsVoiceSelection(nextVoice: string) {
  if (isSettingsActionBusy("tts-voice")) {
    return;
  }

  const previousState = getPanelStates().ttsVoicePanelState;
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
    await refreshRuntimePanels();
  } catch (error: unknown) {
    setTtsVoicePanelState({
      activeVoice: previousState.activeVoice,
      isBusy: false,
      error: describeAudioControlFailure(error),
    });
  }
}

export async function persistConfirmationThreshold(nextThreshold: number) {
  if (isSettingsActionBusy("confirmation-threshold")) {
    return;
  }

  const previousState = getPanelStates().confirmationSettingsPanelState;
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
    await refreshRuntimePanels();
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

export async function persistAllowClickWithoutConfirmation(nextValue: boolean) {
  if (isSettingsActionBusy("click-without-confirmation")) {
    return;
  }

  const previousState = getPanelStates().confirmationSettingsPanelState;
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
    await refreshRuntimePanels();
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

export async function persistOcrThresholds(nextCharThreshold: number, nextRegionThreshold: number) {
  if (isSettingsActionBusy("ocr-char-threshold")) {
    return;
  }

  const previousState = getPanelStates().ocrThresholdSettingsPanelState;
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
    await refreshRuntimePanels();
  } catch (error: unknown) {
    setOcrThresholdSettingsPanelState({
      sparseTextCharThreshold: previousState.sparseTextCharThreshold,
      sparseTextRegionThreshold: previousState.sparseTextRegionThreshold,
      isBusy: false,
      error: describeAudioControlFailure(error),
    });
  }
}

export async function persistModelManagementSettings() {
  if (isSettingsActionBusy("models-dir")) {
    return;
  }

  const panelState = getPanelStates().modelManagementPanelState;
  const modelsDir = panelState.modelsDir.trim();
  if (modelsDir.length === 0) {
    setModelManagementPanelState({
      error: "Enter a models directory before saving model management settings.",
    });
    return;
  }

  const previousState = panelState;
  setModelManagementPanelState({
    isSaving: true,
    error: null,
  });

  try {
    const result = await setModelManagementSettings({
      requestId: createRequestId("model-management-settings"),
      modelsDir,
      checkOnStartup: panelState.checkOnStartup,
      autoDownloadMissing: panelState.autoDownloadMissing,
    });
    setModelManagementPanelState({
      modelsDir: result.models_dir,
      checkOnStartup: result.check_on_startup,
      autoDownloadMissing: result.auto_download_missing,
      isSaving: false,
      error: null,
    });
    await refreshRuntimePanels();
  } catch (error: unknown) {
    setModelManagementPanelState({
      ...previousState,
      isSaving: false,
      error: describeAudioControlFailure(error),
    });
  }
}
