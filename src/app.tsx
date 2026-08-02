import { useSelector } from "react-redux";

import { AppShellRuntime } from "./app-shell.tsx";
import { type AppShellState } from "./app-shell-store";
import { renderAppAlertPanelNode } from "./app-alert-panel.tsx";
import {
  clearAppAlert,
  focusSettingsTarget,
  openExternalLink,
  setAppView,
  setAsrProviderPanelState,
  setAudioControlsState,
  setConfirmationSettingsPanelState,
  setModelManagementPanelState,
  setOcrThresholdSettingsPanelState,
  setRemoteAsrPanelState,
  setRemotePlannerPanelState,
  setRemoteTtsPanelState,
  setSettingsView,
  setStatusPanelState,
  setTtsModelPanelState,
  setTtsProviderPanelState,
  setTtsVoicePanelState,
  setUrlInputPanelState,
} from "./panel-state-setters";
import {
  renderAudioControlsPanelNode,
  renderConfirmationPanelNode,
  renderPushToTalkPanelNode,
  renderVoiceStatusStripNode,
  renderSettingsAsrProviderPanelNode,
  renderSettingsConfirmationPanelNode,
  renderSettingsGuidancePanelNode,
  renderSettingsLocalAsrModelPanelNode,
  renderSettingsLocalTtsModelPanelNode,
  renderSettingsModelManagementPanelNode,
  renderSettingsOcrThresholdPanelNode,
  renderSettingsRemoteAsrPanelNode,
  renderSettingsRemotePlannerPanelNode,
  renderSettingsRemoteTtsPanelNode,
  renderSettingsTtsProviderPanelNode,
  renderSettingsTtsModelPanelNode,
  renderSettingsTtsVoicePanelNode,
  renderStatusPanelNode,
  renderUrlInputPanelNode,
} from "./confirmation-panel";
import { deriveSettingsStatuses } from "./settings-statuses";
import { currentSettingsGuidanceState } from "./main-errors";
import {
  openDraftUrl,
  readCurrentPage,
  readNextRegion,
  readPreviousRegion,
  stopCurrentReading,
} from "./browser-actions";
import { beginPushToTalk, submitConfirmationAction } from "./voice-loop";
import {
  persistAllowClickWithoutConfirmation,
  persistAsrProviderSelection,
  persistBrowserVisibility,
  persistConfirmationThreshold,
  persistModelManagementSettings,
  persistOcrThresholds,
  persistPlaybackSpeed,
  persistPlaybackVolume,
  persistTtsModelSelection,
  persistTtsProviderSelection,
  persistTtsVoiceSelection,
} from "./settings-actions";
import {
  downloadManagedLocalAsrModel,
  downloadManagedLocalTtsModel,
  loadRemotePlannerModels,
  persistRemoteAsrApiKey,
  persistRemotePlannerApiKey,
  persistRemotePlannerConnection,
  persistRemotePlannerPrivacyPolicy,
  persistRemoteTtsApiKey,
  resetRemotePlannerConnectionToDefaults,
  testConfiguredRemoteAsrApiKey,
  testConfiguredRemotePlannerApiKey,
  testConfiguredRemoteTtsApiKey,
} from "./planner-actions";

export function BlindBrowserApp() {
  const shellView = useSelector((state: AppShellState) => state.shellView);
  const panelStates = useSelector((state: AppShellState) => state.panelStates);
  const confirmationState = useSelector((state: AppShellState) => state.executionUi.confirmation);

  return (
    <AppShellRuntime
      appView={shellView.appView}
      settingsView={shellView.settingsView}
      navigationHandlers={{
        onAppViewSelect: setAppView,
        onSettingsViewSelect: setSettingsView,
      }}
      settingsStatuses={deriveSettingsStatuses(panelStates)}
      panelContent={{
        "app-alert": renderAppAlertPanelNode(panelStates.appAlertState, {
          onDismiss: clearAppAlert,
        }),
        "voice-status": renderVoiceStatusStripNode({
          isListening: panelStates.pushToTalkState.isListening,
          isSpeaking: panelStates.statusPanelState.speaking,
          isProcessing: panelStates.pushToTalkState.isBusy,
        }),
        "push-to-talk": renderPushToTalkPanelNode(panelStates.pushToTalkState, {
          onPointerDown: () => { void beginPushToTalk("pointer"); },
          onOpenSettings: () => { setAppView("settings"); },
        }),
        "url-input": renderUrlInputPanelNode(panelStates.urlInputPanelState, {
          onDraftInput: (value) => {
            setUrlInputPanelState({ draftValue: value, hasUnsubmittedChanges: true, error: null });
          },
          onOpen: () => { void openDraftUrl(); },
          onRead: () => { void readCurrentPage(); },
          onStop: () => { void stopCurrentReading(); },
          onPrevious: () => { void readPreviousRegion(); },
          onNext: () => { void readNextRegion(); },
          onDismissError: () => { setUrlInputPanelState({ error: null }); },
        }),
        status: renderStatusPanelNode({
          ...panelStates.statusPanelState,
          isPageLoading: panelStates.urlInputPanelState.isOpening,
          plannerBusy: panelStates.urlInputPanelState.isOpening
            || panelStates.urlInputPanelState.isReading
            || panelStates.statusPanelState.speaking,
        }, {
          onSetBrowserVisibility: (mode) => { void persistBrowserVisibility(mode); },
          onDismissError: () => { setStatusPanelState({ error: null }); },
        }),
        "audio-controls": renderAudioControlsPanelNode(panelStates.audioControlsState, {
          onVolumeChange: (value) => {
            setAudioControlsState({ playbackVolume: value, error: null });
            void persistPlaybackVolume(value);
          },
          onSpeedChange: (value) => {
            setAudioControlsState({ playbackSpeed: value, error: null });
            void persistPlaybackSpeed(value);
          },
          onDismissError: () => { setAudioControlsState({ error: null }); },
        }),
        "settings-guidance": renderSettingsGuidancePanelNode(currentSettingsGuidanceState(panelStates), {
          onSelectTarget: focusSettingsTarget,
          onOpenExternalLink: openExternalLink,
        }),
        "settings-remote-planner": renderSettingsRemotePlannerPanelNode(panelStates.remotePlannerPanelState, {
          onConsentChange: (checked) => {
            setRemotePlannerPanelState({ consentToRemotePageData: checked, error: null });
          },
          onLocalOnlyChange: (checked) => {
            setRemotePlannerPanelState({ localOnly: checked, error: null });
          },
          onBlockedOriginsInput: (value) => {
            setRemotePlannerPanelState({ blockedOriginsDraft: value, error: null });
          },
          onSavePrivacy: () => { void persistRemotePlannerPrivacyPolicy(); },
          onApiKeyInput: (value) => {
            setRemotePlannerPanelState({ apiKeyDraft: value, apiKeyTestMessage: null, error: null });
          },
          onSaveApiKey: () => { void persistRemotePlannerApiKey(); },
          onTestApiKey: () => { void testConfiguredRemotePlannerApiKey(); },
          onOpenExternalLink: openExternalLink,
          onEndpointInput: (value) => {
            setRemotePlannerPanelState({ baseUrl: value, availableModels: [], loadedModelsEndpoint: null, error: null });
          },
          onEndpointBlur: () => {
            // Endpoint editing is intentionally side-effect free. Credential-bearing
            // model discovery requires the explicit Load models action.
          },
          onModelSelect: (value) => { setRemotePlannerPanelState({ model: value, error: null }); },
          onModelInput: (value) => { setRemotePlannerPanelState({ model: value, error: null }); },
          onLoadModels: () => { void loadRemotePlannerModels(); },
          onSaveSettings: () => { void persistRemotePlannerConnection(); },
          onBeginReset: () => { setRemotePlannerPanelState({ isConfirmingReset: true }); },
          onConfirmReset: () => {
            setRemotePlannerPanelState({ isConfirmingReset: false });
            void resetRemotePlannerConnectionToDefaults();
          },
          onCancelReset: () => { setRemotePlannerPanelState({ isConfirmingReset: false }); },
          onDismissError: () => { setRemotePlannerPanelState({ error: null }); },
          onRetry: () => { void loadRemotePlannerModels(); },
        }),
        "settings-confirmation": renderSettingsConfirmationPanelNode(panelStates.confirmationSettingsPanelState, {
          onThresholdChange: (value) => {
            setConfirmationSettingsPanelState({ confirmationConfidenceThreshold: value, error: null });
            void persistConfirmationThreshold(value);
          },
          onClickWithoutConfirmationChange: (checked) => { void persistAllowClickWithoutConfirmation(checked); },
          onDismissError: () => { setConfirmationSettingsPanelState({ error: null }); },
        }),
        "settings-ocr-threshold": renderSettingsOcrThresholdPanelNode(panelStates.ocrThresholdSettingsPanelState, {
          onCharThresholdChange: (value) => {
            setOcrThresholdSettingsPanelState({ sparseTextCharThreshold: value, error: null });
            void persistOcrThresholds(value, panelStates.ocrThresholdSettingsPanelState.sparseTextRegionThreshold);
          },
          onRegionThresholdChange: (value) => {
            setOcrThresholdSettingsPanelState({ sparseTextRegionThreshold: value, error: null });
            void persistOcrThresholds(panelStates.ocrThresholdSettingsPanelState.sparseTextCharThreshold, value);
          },
          onDismissError: () => { setOcrThresholdSettingsPanelState({ error: null }); },
        }),
        "settings-asr-provider": renderSettingsAsrProviderPanelNode(panelStates.asrProviderPanelState, {
          onProviderSelect: (mode) => { void persistAsrProviderSelection(mode); },
          onDismissError: () => { setAsrProviderPanelState({ error: null }); },
        }),
        "settings-local-asr-model": renderSettingsLocalAsrModelPanelNode(
          { ...panelStates.localAsrModelPanelState, modelAvailable: panelStates.modelManagementPanelState.localAsrAvailable },
          { onOpenRuntimeSettings: () => { setSettingsView("runtime"); setAppView("settings"); } },
        ),
        "settings-model-management": renderSettingsModelManagementPanelNode(panelStates.modelManagementPanelState, {
          onModelsDirInput: (value) => { setModelManagementPanelState({ modelsDir: value, error: null }); },
          onPersistModelsDir: () => { void persistModelManagementSettings(); },
          onCheckOnStartupChange: (checked) => {
            setModelManagementPanelState({ checkOnStartup: checked, error: null });
            void persistModelManagementSettings();
          },
          onAutoDownloadMissingChange: (checked) => {
            setModelManagementPanelState({ autoDownloadMissing: checked, error: null });
            void persistModelManagementSettings();
          },
          onDownloadModel: (kind) => {
            if (kind === "tts") {
              void downloadManagedLocalTtsModel();
              return;
            }
            void downloadManagedLocalAsrModel();
          },
          onDismissError: () => { setModelManagementPanelState({ error: null }); },
          onRetry: () => { void persistModelManagementSettings(); },
        }),
        "settings-remote-asr": renderSettingsRemoteAsrPanelNode(panelStates.remoteAsrPanelState, {
          onApiKeyInput: (value) => {
            setRemoteAsrPanelState({ apiKeyDraft: value, apiKeyTestMessage: null, error: null });
          },
          onSaveApiKey: () => { void persistRemoteAsrApiKey(); },
          onTestApiKey: () => { void testConfiguredRemoteAsrApiKey(); },
          onOpenExternalLink: openExternalLink,
          onDismissError: () => { setRemoteAsrPanelState({ error: null }); },
          onRetry: () => { void testConfiguredRemoteAsrApiKey(); },
        }),
        "settings-tts-provider": renderSettingsTtsProviderPanelNode(panelStates.ttsProviderPanelState, {
          onProviderSelect: (mode) => { void persistTtsProviderSelection(mode); },
          onDismissError: () => { setTtsProviderPanelState({ error: null }); },
        }),
        "settings-tts-model": renderSettingsTtsModelPanelNode(panelStates.ttsModelPanelState, {
          onModelSelect: (profileName) => { void persistTtsModelSelection(profileName); },
          onDismissError: () => { setTtsModelPanelState({ error: null }); },
        }),
        "settings-local-tts-model": renderSettingsLocalTtsModelPanelNode(
          { ...panelStates.localTtsModelPanelState, modelAvailable: panelStates.modelManagementPanelState.localTtsAvailable },
          { onOpenRuntimeSettings: () => { setSettingsView("runtime"); setAppView("settings"); } },
        ),
        "settings-remote-tts": renderSettingsRemoteTtsPanelNode(panelStates.remoteTtsPanelState, {
          onApiKeyInput: (value) => {
            setRemoteTtsPanelState({ apiKeyDraft: value, apiKeyTestMessage: null, error: null });
          },
          onSaveApiKey: () => { void persistRemoteTtsApiKey(); },
          onTestApiKey: () => { void testConfiguredRemoteTtsApiKey(); },
          onOpenExternalLink: openExternalLink,
          onDismissError: () => { setRemoteTtsPanelState({ error: null }); },
          onRetry: () => { void testConfiguredRemoteTtsApiKey(); },
        }),
        "settings-tts-voice": renderSettingsTtsVoicePanelNode(panelStates.ttsVoicePanelState, {
          onVoiceSelect: (voice) => { void persistTtsVoiceSelection(voice); },
          onDismissError: () => { setTtsVoicePanelState({ error: null }); },
        }),
        "confirmation-panel": renderConfirmationPanelNode(confirmationState, {
          onRespond: submitConfirmationAction,
        }),
      }}
    />
  );
}
