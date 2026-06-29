import { statusPanelStateFromAgentState } from "./confirmation-panel";
import { splitRuntimeRefreshResults } from "./main-behavior";
import type { PanelStates } from "./panel-state";
import {
  getAgentState,
  getModelManagementSettings,
  type AgentStateData,
} from "./tauri-api";

export interface RuntimeRefreshDependencies {
  createRequestId: (prefix: string) => string;
  describeAudioControlFailure: (error: unknown) => string;
  describeScopedRuntimeRefreshFailure: (
    scope: "runtime state" | "model management",
    error: unknown,
  ) => string;
  ensureContinuousListeningLoop: () => Promise<void> | void;
  getPanelStates: () => PanelStates;
  setPushToTalkState: (nextState: Partial<PanelStates["pushToTalkState"]>) => void;
  setAudioControlsState: (nextState: Partial<PanelStates["audioControlsState"]>) => void;
  setRemotePlannerPanelState: (nextState: Partial<PanelStates["remotePlannerPanelState"]>) => void;
  setProviderFailoverPanelState: (nextState: Partial<PanelStates["providerFailoverPanelState"]>) => void;
  setConfirmationSettingsPanelState: (
    nextState: Partial<PanelStates["confirmationSettingsPanelState"]>,
  ) => void;
  setOcrThresholdSettingsPanelState: (
    nextState: Partial<PanelStates["ocrThresholdSettingsPanelState"]>,
  ) => void;
  setAsrProviderPanelState: (nextState: Partial<PanelStates["asrProviderPanelState"]>) => void;
  setLocalAsrModelPanelState: (nextState: Partial<PanelStates["localAsrModelPanelState"]>) => void;
  setModelManagementPanelState: (
    nextState: Partial<PanelStates["modelManagementPanelState"]>,
  ) => void;
  setRemoteAsrPanelState: (nextState: Partial<PanelStates["remoteAsrPanelState"]>) => void;
  setTtsProviderPanelState: (nextState: Partial<PanelStates["ttsProviderPanelState"]>) => void;
  setTtsModelPanelState: (nextState: Partial<PanelStates["ttsModelPanelState"]>) => void;
  setLocalTtsModelPanelState: (nextState: Partial<PanelStates["localTtsModelPanelState"]>) => void;
  setRemoteTtsPanelState: (nextState: Partial<PanelStates["remoteTtsPanelState"]>) => void;
  setTtsVoicePanelState: (nextState: Partial<PanelStates["ttsVoicePanelState"]>) => void;
  setStatusPanelState: (nextState: Partial<PanelStates["statusPanelState"]>) => void;
  setUrlInputPanelState: (nextState: Partial<PanelStates["urlInputPanelState"]>) => void;
}

export function applyAgentStateToPanels(
  dependencies: RuntimeRefreshDependencies,
  agentState: AgentStateData,
): void {
  dependencies.setPushToTalkState({
    enabled: agentState.listening_state.push_to_talk_enabled,
    isListening: agentState.listening_state.is_listening,
    lastTranscript: agentState.last_transcript,
  });
  dependencies.setAudioControlsState({
    playbackVolume: agentState.audio.playback_volume,
    playbackSpeed: agentState.audio.playback_speed,
    error: null,
  });

  const currentPlannerState = dependencies.getPanelStates().remotePlannerPanelState;
  const refreshedBaseUrl = agentState.remote_planner_settings.base_url;
  const keepVerifiedPlannerModels = currentPlannerState.availableModels.length > 0
    && currentPlannerState.loadedModelsEndpoint !== null
    && currentPlannerState.loadedModelsEndpoint === refreshedBaseUrl;

  dependencies.setRemotePlannerPanelState({
    profileName: agentState.remote_planner_settings.profile_name,
    provider: agentState.remote_planner_settings.provider,
    baseUrl: agentState.remote_planner_settings.base_url,
    model: agentState.remote_planner_settings.model,
    availableModels: keepVerifiedPlannerModels ? currentPlannerState.availableModels : [],
    loadedModelsEndpoint: keepVerifiedPlannerModels ? currentPlannerState.loadedModelsEndpoint : null,
    isLoadingModels: false,
    isSavingConnection: false,
    isResettingConnection: false,
    apiKeyReference: agentState.remote_planner_settings.api_key_reference,
    apiKeyMaskedValue: agentState.remote_planner_settings.api_key_masked_value,
    organizationReference: agentState.remote_planner_settings.organization_reference,
    project: agentState.remote_planner_settings.project,
    temperatureMilli: agentState.remote_planner_settings.temperature_milli,
    maxOutputTokens: agentState.remote_planner_settings.max_output_tokens,
    timeoutMs: agentState.remote_planner_settings.timeout_ms,
  });
  dependencies.setProviderFailoverPanelState({
    plannerAvailable: agentState.provider_failover_settings.planner_available,
    ttsAvailable: agentState.provider_failover_settings.tts_available,
    asrAvailable: agentState.provider_failover_settings.asr_available,
    summary: agentState.provider_failover_settings.summary,
  });
  dependencies.setConfirmationSettingsPanelState({
    confirmationConfidenceThreshold: agentState.confirmation_settings.confirmation_confidence_threshold,
    allowClickWithoutConfirmation: agentState.confirmation_settings.allow_click_without_confirmation,
    alwaysConfirmSubmit: agentState.confirmation_settings.always_confirm_submit,
    isBusy: false,
    error: null,
  });
  dependencies.setOcrThresholdSettingsPanelState({
    sparseTextCharThreshold: agentState.ocr_threshold_settings.sparse_text_char_threshold,
    sparseTextRegionThreshold: agentState.ocr_threshold_settings.sparse_text_region_threshold,
    isBusy: false,
    error: null,
  });
  dependencies.setAsrProviderPanelState({
    activeMode: agentState.asr_provider_settings.active_mode,
    availableModes: agentState.asr_provider_settings.available_modes,
    isBusy: false,
    error: null,
  });
  dependencies.setLocalAsrModelPanelState({
    profileName: agentState.local_asr_model_settings.profile_name,
    backend: agentState.local_asr_model_settings.backend,
    modelId: agentState.local_asr_model_settings.model_id,
    modelPath: agentState.local_asr_model_settings.model_path,
    language: agentState.local_asr_model_settings.language,
    threads: agentState.local_asr_model_settings.threads,
  });
  dependencies.setRemoteAsrPanelState({
    profileName: agentState.remote_asr_settings.profile_name,
    provider: agentState.remote_asr_settings.provider,
    baseUrl: agentState.remote_asr_settings.base_url,
    model: agentState.remote_asr_settings.model,
    apiKeyReference: agentState.remote_asr_settings.api_key_reference,
    apiKeyMaskedValue: agentState.remote_asr_settings.api_key_masked_value,
    organizationReference: agentState.remote_asr_settings.organization_reference,
    project: agentState.remote_asr_settings.project,
    language: agentState.remote_asr_settings.language,
    temperatureMilli: agentState.remote_asr_settings.temperature_milli,
    timeoutMs: agentState.remote_asr_settings.timeout_ms,
  });
  dependencies.setTtsProviderPanelState({
    activeMode: agentState.tts_provider_settings.active_mode,
    availableModes: agentState.tts_provider_settings.available_modes,
    isBusy: false,
    error: null,
  });
  dependencies.setTtsModelPanelState({
    mode: agentState.tts_model_settings.mode,
    activeProfile: agentState.tts_model_settings.active_profile,
    availableProfiles: agentState.tts_model_settings.available_profiles.map((option) => ({
      profileName: option.profile_name,
      modelLabel: option.model_label,
    })),
    isBusy: false,
    error: null,
  });
  dependencies.setLocalTtsModelPanelState({
    profileName: agentState.local_tts_model_settings.profile_name,
    backend: agentState.local_tts_model_settings.backend,
    modelId: agentState.local_tts_model_settings.model_id,
    modelPath: agentState.local_tts_model_settings.model_path,
    defaultVoice: agentState.local_tts_model_settings.default_voice,
    sampleRate: agentState.local_tts_model_settings.sample_rate,
  });
  dependencies.setRemoteTtsPanelState({
    profileName: agentState.remote_tts_settings.profile_name,
    provider: agentState.remote_tts_settings.provider,
    baseUrl: agentState.remote_tts_settings.base_url,
    model: agentState.remote_tts_settings.model,
    apiKeyReference: agentState.remote_tts_settings.api_key_reference,
    apiKeyMaskedValue: agentState.remote_tts_settings.api_key_masked_value,
    organizationReference: agentState.remote_tts_settings.organization_reference,
    project: agentState.remote_tts_settings.project,
    voice: agentState.remote_tts_settings.voice,
    audioFormat: agentState.remote_tts_settings.audio_format,
    timeoutMs: agentState.remote_tts_settings.timeout_ms,
  });
  dependencies.setTtsVoicePanelState({
    mode: agentState.tts_voice_settings.mode,
    activeVoice: agentState.tts_voice_settings.active_voice,
    availableVoices: agentState.tts_voice_settings.available_voices.map((option) => ({
      voiceName: option.voice_name,
      displayLabel: option.display_label,
    })),
    isBusy: false,
    error: null,
  });
  dependencies.setStatusPanelState(statusPanelStateFromAgentState(agentState));

  // Re-read panel states for URL input — the URL input draft is preserved across refreshes
  // when the user has uncommitted changes. URL input errors are cleared here because they
  // reflect workspace navigation state; settings/global alerts are unaffected by this path.
  const panelStates = dependencies.getPanelStates();
  dependencies.setUrlInputPanelState({
    currentUrl: agentState.url,
    draftValue: panelStates.urlInputPanelState.hasUnsubmittedChanges
      ? panelStates.urlInputPanelState.draftValue
      : (agentState.url ?? ""),
    isOpening: false,
    isReading: false,
    isStopping: false,
    isAdvancing: false,
    isRewinding: false,
    error: null,
  });

  if (agentState.listening_state.is_listening && !dependencies.getPanelStates().pushToTalkState.isHolding) {
    void dependencies.ensureContinuousListeningLoop();
  }
}

export function createRuntimeRefreshHandlers(dependencies: RuntimeRefreshDependencies) {
  const refreshRuntimePanelsFromRuntime = async () => {
    const refreshResults = splitRuntimeRefreshResults(
      ...(await Promise.allSettled([
        getAgentState({
          requestId: dependencies.createRequestId("runtime-panels-state"),
          includeLastTranscript: true,
        }),
        getModelManagementSettings({
          requestId: dependencies.createRequestId("model-management-state"),
        }),
      ])),
      dependencies.describeAudioControlFailure,
    );

    if (refreshResults.agentState) {
      applyAgentStateToPanels(dependencies, refreshResults.agentState);
    } else if (refreshResults.agentStateError) {
      dependencies.setStatusPanelState({
        error: dependencies.describeScopedRuntimeRefreshFailure(
          "runtime state",
          refreshResults.agentStateError,
        ),
        isUpdatingVisibility: false,
      });
      dependencies.setPushToTalkState({
        isBusy: false,
        lastError: dependencies.getPanelStates().pushToTalkState.lastError,
      });
    }

    if (refreshResults.modelSettings) {
      const modelManagementSettings = refreshResults.modelSettings;
      dependencies.setModelManagementPanelState({
        modelsDir: modelManagementSettings.models_dir,
        checkOnStartup: modelManagementSettings.check_on_startup,
        autoDownloadMissing: modelManagementSettings.auto_download_missing,
        localTtsAvailable: modelManagementSettings.local_tts.available,
        localTtsDownloadSupported: modelManagementSettings.local_tts.download_supported,
        localTtsDownloadLabel: modelManagementSettings.local_tts.download_label,
        localAsrAvailable: modelManagementSettings.local_asr.available,
        localAsrDownloadSupported: modelManagementSettings.local_asr.download_supported,
        localAsrDownloadLabel: modelManagementSettings.local_asr.download_label,
        isSaving: false,
        isDownloadingTts: false,
        isDownloadingAsr: false,
        error: null,
      });
    } else if (refreshResults.modelSettingsError) {
      dependencies.setModelManagementPanelState({
        isSaving: false,
        isDownloadingTts: false,
        isDownloadingAsr: false,
        error: dependencies.describeScopedRuntimeRefreshFailure(
          "model management",
          refreshResults.modelSettingsError,
        ),
      });
    }
  };

  return { refreshRuntimePanelsFromRuntime };
}
