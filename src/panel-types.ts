export interface PushToTalkPanelState {
  enabled: boolean;
  isHolding: boolean;
  isListening: boolean;
  isBusy: boolean;
  lastTranscript: string | null;
  lastError: string | null;
}

export interface AudioControlsPanelState {
  playbackVolume: number;
  playbackSpeed: number;
  isBusy: boolean;
  error: string | null;
}

export interface TtsModelPanelState {
  mode: "Local" | "Remote" | "Disabled";
  activeProfile: string | null;
  availableProfiles: Array<{
    profileName: string;
    modelLabel: string;
  }>;
  isBusy: boolean;
  error: string | null;
}

export interface TtsVoicePanelState {
  mode: "Local" | "Remote" | "Disabled";
  activeVoice: string | null;
  availableVoices: Array<{
    voiceName: string;
    displayLabel: string;
  }>;
  isBusy: boolean;
  error: string | null;
}

export interface TtsProviderPanelState {
  activeMode: "Local" | "Remote";
  availableModes: Array<"Local" | "Remote">;
  isBusy: boolean;
  error: string | null;
}

export interface AsrProviderPanelState {
  activeMode: "Local" | "Remote";
  availableModes: Array<"Local" | "Remote">;
  isBusy: boolean;
  error: string | null;
}

export interface LocalTtsModelPanelState {
  profileName: string | null;
  backend: string | null;
  modelId: string | null;
  modelPath: string | null;
  defaultVoice: string | null;
  sampleRate: number | null;
}

export interface LocalAsrModelPanelState {
  profileName: string | null;
  backend: string | null;
  modelId: string | null;
  modelPath: string | null;
  language: string | null;
  threads: number | null;
}

export interface ModelManagementPanelState {
  modelsDir: string;
  checkOnStartup: boolean;
  autoDownloadMissing: boolean;
  localTtsAvailable: boolean;
  localTtsDownloadSupported: boolean;
  localTtsDownloadLabel: string | null;
  localAsrAvailable: boolean;
  localAsrDownloadSupported: boolean;
  localAsrDownloadLabel: string | null;
  isSaving: boolean;
  isDownloadingTts: boolean;
  isDownloadingAsr: boolean;
  error: string | null;
}

export interface PlannerProviderPanelState {
  activeMode: "Remote";
  availableModes: ["Remote"] | "Remote"[];
  summary: string;
}

export interface RemotePlannerPanelState {
  profileName: string | null;
  provider: string | null;
  baseUrl: string | null;
  model: string | null;
  apiKeyReference: string | null;
  organizationReference: string | null;
  project: string | null;
  temperatureMilli: number | null;
  maxOutputTokens: number | null;
  timeoutMs: number | null;
  apiKeyDraft: string;
  isSavingApiKey: boolean;
  isTestingApiKey: boolean;
  apiKeyTestMessage: string | null;
  error: string | null;
}

export interface RemoteTtsPanelState {
  profileName: string | null;
  provider: string | null;
  baseUrl: string | null;
  model: string | null;
  apiKeyReference: string | null;
  organizationReference: string | null;
  project: string | null;
  voice: string | null;
  audioFormat: string | null;
  timeoutMs: number | null;
  apiKeyDraft: string;
  isSavingApiKey: boolean;
  isTestingApiKey: boolean;
  apiKeyTestMessage: string | null;
  error: string | null;
}

export interface RemoteAsrPanelState {
  profileName: string | null;
  provider: string | null;
  baseUrl: string | null;
  model: string | null;
  apiKeyReference: string | null;
  organizationReference: string | null;
  project: string | null;
  language: string | null;
  temperatureMilli: number | null;
  timeoutMs: number | null;
  apiKeyDraft: string;
  isSavingApiKey: boolean;
  isTestingApiKey: boolean;
  apiKeyTestMessage: string | null;
  error: string | null;
}

export interface ProviderFailoverPanelState {
  plannerAvailable: boolean;
  ttsAvailable: boolean;
  asrAvailable: boolean;
  summary: string;
}

export interface ConfirmationSettingsPanelState {
  confirmationConfidenceThreshold: number;
  allowClickWithoutConfirmation: boolean;
  alwaysConfirmSubmit: boolean;
  isBusy: boolean;
  error: string | null;
}

export interface OcrThresholdSettingsPanelState {
  sparseTextCharThreshold: number;
  sparseTextRegionThreshold: number;
  isBusy: boolean;
  error: string | null;
}

export interface SettingsGuidancePanelAction {
  label: string;
  targetId: string;
}

export interface SettingsGuidancePanelState {
  title: string;
  message: string;
  actions: SettingsGuidancePanelAction[];
}

export interface UrlInputPanelState {
  draftValue: string;
  currentUrl: string | null;
  hasUnsubmittedChanges: boolean;
  isOpening: boolean;
  isReading: boolean;
  isStopping: boolean;
  isAdvancing: boolean;
  isRewinding: boolean;
  error: string | null;
}

export interface StatusPanelState {
  pageTitle: string | null;
  currentRegionLabel: string | null;
  lastTranscript: string | null;
  listening: boolean;
  speaking: boolean;
  browserVisibility: "Visible" | "Headless";
  canGoBack: boolean;
  canGoForward: boolean;
  isUpdatingVisibility: boolean;
  error: string | null;
}

export interface StatusPanelAgentStateLike {
  title: string | null;
  url: string | null;
  narration_cursor: {
    node_index: number;
  } | null;
  last_transcript: string | null;
  listening_state: {
    is_listening: boolean;
  };
  speaking: boolean;
  browser_visibility: "Visible" | "Headless";
  browser_history: {
    can_go_back: boolean;
    can_go_forward: boolean;
  };
}
