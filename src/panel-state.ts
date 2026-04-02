import type {
  AudioControlsPanelState,
  AsrProviderPanelState,
  ConfirmationSettingsPanelState,
  LocalAsrModelPanelState,
  LocalTtsModelPanelState,
  ModelManagementPanelState,
  OcrThresholdSettingsPanelState,
  PlannerProviderPanelState,
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

export interface PanelStates {
  pushToTalkState: PushToTalkPanelState;
  audioControlsState: AudioControlsPanelState;
  plannerProviderPanelState: PlannerProviderPanelState;
  remotePlannerPanelState: RemotePlannerPanelState;
  providerFailoverPanelState: ProviderFailoverPanelState;
  confirmationSettingsPanelState: ConfirmationSettingsPanelState;
  ocrThresholdSettingsPanelState: OcrThresholdSettingsPanelState;
  asrProviderPanelState: AsrProviderPanelState;
  localAsrModelPanelState: LocalAsrModelPanelState;
  modelManagementPanelState: ModelManagementPanelState;
  remoteAsrPanelState: RemoteAsrPanelState;
  ttsProviderPanelState: TtsProviderPanelState;
  ttsModelPanelState: TtsModelPanelState;
  localTtsModelPanelState: LocalTtsModelPanelState;
  remoteTtsPanelState: RemoteTtsPanelState;
  ttsVoicePanelState: TtsVoicePanelState;
  statusPanelState: StatusPanelState;
  urlInputPanelState: UrlInputPanelState;
}

export function createInitialPanelStates(): PanelStates {
  return {
    pushToTalkState: {
      enabled: true,
      isHolding: false,
      isListening: false,
      isBusy: false,
      lastTranscript: null,
      lastError: null,
    },
    audioControlsState: {
      playbackVolume: 1,
      playbackSpeed: 1,
      isBusy: false,
      error: null,
    },
    plannerProviderPanelState: {
      activeMode: "Remote",
      availableModes: ["Remote"],
      summary: "Planner currently uses configured remote profiles only.",
    },
    remotePlannerPanelState: {
      profileName: null,
      provider: null,
      baseUrl: null,
      model: null,
      apiKeyReference: null,
      organizationReference: null,
      project: null,
      temperatureMilli: null,
      maxOutputTokens: null,
      timeoutMs: null,
      apiKeyDraft: "",
      isSavingApiKey: false,
      error: null,
    },
    providerFailoverPanelState: {
      plannerAvailable: false,
      ttsAvailable: false,
      asrAvailable: false,
      summary:
        "Provider failover settings are defined in config, but automatic failover is still disabled in the live runtime.",
    },
    confirmationSettingsPanelState: {
      confirmationConfidenceThreshold: 0.9,
      allowClickWithoutConfirmation: true,
      alwaysConfirmSubmit: true,
      isBusy: false,
      error: null,
    },
    ocrThresholdSettingsPanelState: {
      sparseTextCharThreshold: 200,
      sparseTextRegionThreshold: 2,
      isBusy: false,
      error: null,
    },
    asrProviderPanelState: {
      activeMode: "Local",
      availableModes: ["Local", "Remote"],
      isBusy: false,
      error: null,
    },
    localAsrModelPanelState: {
      profileName: null,
      backend: null,
      modelId: null,
      modelPath: null,
      language: null,
      threads: null,
    },
    modelManagementPanelState: {
      modelsDir: "",
      checkOnStartup: true,
      autoDownloadMissing: false,
      localTtsAvailable: false,
      localTtsDownloadSupported: false,
      localTtsDownloadLabel: null,
      localAsrAvailable: false,
      localAsrDownloadSupported: false,
      localAsrDownloadLabel: null,
      isSaving: false,
      isDownloadingTts: false,
      isDownloadingAsr: false,
      error: null,
    },
    remoteAsrPanelState: {
      profileName: null,
      provider: null,
      baseUrl: null,
      model: null,
      apiKeyReference: null,
      organizationReference: null,
      project: null,
      language: null,
      temperatureMilli: null,
      timeoutMs: null,
      apiKeyDraft: "",
      isSavingApiKey: false,
      error: null,
    },
    ttsProviderPanelState: {
      activeMode: "Local",
      availableModes: ["Local", "Remote"],
      isBusy: false,
      error: null,
    },
    ttsModelPanelState: {
      mode: "Local",
      activeProfile: null,
      availableProfiles: [],
      isBusy: false,
      error: null,
    },
    localTtsModelPanelState: {
      profileName: null,
      backend: null,
      modelId: null,
      modelPath: null,
      defaultVoice: null,
      sampleRate: null,
    },
    remoteTtsPanelState: {
      profileName: null,
      provider: null,
      baseUrl: null,
      model: null,
      apiKeyReference: null,
      organizationReference: null,
      project: null,
      voice: null,
      audioFormat: null,
      timeoutMs: null,
      apiKeyDraft: "",
      isSavingApiKey: false,
      error: null,
    },
    ttsVoicePanelState: {
      mode: "Local",
      activeVoice: null,
      availableVoices: [],
      isBusy: false,
      error: null,
    },
    statusPanelState: {
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
    },
    urlInputPanelState: {
      draftValue: "",
      currentUrl: null,
      hasUnsubmittedChanges: false,
      isOpening: false,
      isReading: false,
      isStopping: false,
      isAdvancing: false,
      isRewinding: false,
      error: null,
    },
  };
}
