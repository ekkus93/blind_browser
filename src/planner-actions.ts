import { appShellStore } from "./store";
import { describeAudioControlFailure } from "./main-errors";
import {
  createRequestId,
  setModelManagementPanelState,
  setRemoteAsrPanelState,
  setRemotePlannerPanelState,
  setRemoteTtsPanelState,
} from "./panel-state-setters";
import { refreshRuntimePanels } from "./refresh-handle";
import {
  downloadActiveLocalAsrModel,
  downloadActiveLocalTtsModel,
  listRemotePlannerModels,
  resetRemotePlannerConnectionSettings,
  setRemoteAsrApiKey,
  setRemotePlannerApiKey,
  setRemotePlannerConnectionSettings,
  setRemoteTtsApiKey,
  testRemoteAsrApiKey,
  testRemotePlannerApiKey,
  testRemoteTtsApiKey,
} from "./tauri-api";

function getPanelStates() {
  return appShellStore.getState().panelStates;
}

export async function downloadManagedLocalTtsModel() {
  if (getPanelStates().modelManagementPanelState.isDownloadingTts) {
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
    await refreshRuntimePanels();
  } catch (error: unknown) {
    setModelManagementPanelState({
      isDownloadingTts: false,
      error: describeAudioControlFailure(error),
    });
  }
}

export async function downloadManagedLocalAsrModel() {
  if (getPanelStates().modelManagementPanelState.isDownloadingAsr) {
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
    await refreshRuntimePanels();
  } catch (error: unknown) {
    setModelManagementPanelState({
      isDownloadingAsr: false,
      error: describeAudioControlFailure(error),
    });
  }
}

export async function persistRemotePlannerApiKey() {
  const state = getPanelStates().remotePlannerPanelState;
  const profileName = state.profileName;
  const apiKey = state.apiKeyDraft.trim();
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
    await refreshRuntimePanels();
  } catch (error: unknown) {
    setRemotePlannerPanelState({
      isSavingApiKey: false,
      error: describeAudioControlFailure(error),
    });
  }
}

export async function loadRemotePlannerModels() {
  const state = getPanelStates().remotePlannerPanelState;
  const profileName = state.profileName;
  const baseUrl = state.baseUrl?.trim() ?? "";
  const apiKey = state.apiKeyDraft.trim();
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
    const currentModel = getPanelStates().remotePlannerPanelState.model;
    const nextModel = result.models.includes(currentModel ?? "")
      ? currentModel
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

export async function persistRemotePlannerConnection() {
  const state = getPanelStates().remotePlannerPanelState;
  const profileName = state.profileName;
  const baseUrl = state.baseUrl?.trim() ?? "";
  const model = state.model?.trim() ?? "";
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
    const currentAvailable = getPanelStates().remotePlannerPanelState.availableModels;
    setRemotePlannerPanelState({
      profileName: result.profile_name,
      baseUrl: result.base_url,
      model: result.model,
      availableModels: currentAvailable.includes(result.model)
        ? currentAvailable
        : [result.model, ...currentAvailable],
      loadedModelsEndpoint: result.base_url,
      isSavingConnection: false,
      error: null,
    });
    await refreshRuntimePanels();
  } catch (error: unknown) {
    setRemotePlannerPanelState({
      isSavingConnection: false,
      error: describeAudioControlFailure(error),
    });
  }
}

export async function resetRemotePlannerConnectionToDefaults() {
  const profileName = getPanelStates().remotePlannerPanelState.profileName;
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
    await refreshRuntimePanels();
  } catch (error: unknown) {
    setRemotePlannerPanelState({
      isResettingConnection: false,
      error: describeAudioControlFailure(error),
    });
  }
}

export async function persistRemoteTtsApiKey() {
  const state = getPanelStates().remoteTtsPanelState;
  const profileName = state.profileName;
  const apiKey = state.apiKeyDraft.trim();
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
    await refreshRuntimePanels();
  } catch (error: unknown) {
    setRemoteTtsPanelState({
      isSavingApiKey: false,
      error: describeAudioControlFailure(error),
    });
  }
}

export async function persistRemoteAsrApiKey() {
  const state = getPanelStates().remoteAsrPanelState;
  const profileName = state.profileName;
  const apiKey = state.apiKeyDraft.trim();
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
    await refreshRuntimePanels();
  } catch (error: unknown) {
    setRemoteAsrPanelState({
      isSavingApiKey: false,
      error: describeAudioControlFailure(error),
    });
  }
}

export async function testConfiguredRemotePlannerApiKey() {
  const state = getPanelStates().remotePlannerPanelState;
  const profileName = state.profileName;
  const apiKey = state.apiKeyDraft.trim();
  if (!profileName) {
    setRemotePlannerPanelState({
      error: "No remote planner profile is configured for API key testing.",
      apiKeyTestMessage: null,
    });
    return;
  }
  if (apiKey.length === 0 && !state.apiKeyReference) {
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

export async function testConfiguredRemoteTtsApiKey() {
  const state = getPanelStates().remoteTtsPanelState;
  const profileName = state.profileName;
  const apiKey = state.apiKeyDraft.trim();
  if (!profileName) {
    setRemoteTtsPanelState({
      error: "No remote TTS profile is configured for API key testing.",
      apiKeyTestMessage: null,
    });
    return;
  }
  if (apiKey.length === 0 && !state.apiKeyReference) {
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

export async function testConfiguredRemoteAsrApiKey() {
  const state = getPanelStates().remoteAsrPanelState;
  const profileName = state.profileName;
  const apiKey = state.apiKeyDraft.trim();
  if (!profileName) {
    setRemoteAsrPanelState({
      error: "No remote ASR profile is configured for API key testing.",
      apiKeyTestMessage: null,
    });
    return;
  }
  if (apiKey.length === 0 && !state.apiKeyReference) {
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
