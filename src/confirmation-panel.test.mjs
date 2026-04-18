import assert from "node:assert/strict";
import test from "node:test";

import {
  renderAudioControlsPanel,
  renderConfirmationPanel,
  renderPushToTalkPanel,
  renderSettingsAsrProviderPanel,
  renderSettingsConfirmationPanel,
  renderSettingsGuidancePanel,
  renderSettingsLocalAsrModelPanel,
  renderSettingsLocalTtsModelPanel,
  renderSettingsModelManagementPanel,
  renderSettingsOcrThresholdPanel,
  renderSettingsProviderFailoverPanel,
  renderSettingsRemoteAsrPanel,
  renderSettingsRemotePlannerPanel,
  renderSettingsRemoteTtsPanel,
  renderSettingsTtsProviderPanel,
  renderSettingsTtsModelPanel,
  renderSettingsTtsVoicePanel,
  renderSettingsSpeedPanel,
  renderSettingsVolumePanel,
  renderStatusPanel,
  statusPanelStateFromAgentState,
  renderUrlInputPanel,
} from "./confirmation-panel.ts";

function renderFixtures() {
  const nonRetryableHtml = renderConfirmationPanel({
    kind: "awaiting-confirmation",
    isSubmitting: false,
    submissionError: {
      kind: "tool-error",
      title: "Runtime cannot complete this request",
      message: "The backend rejected the action.",
      guidance: "Review the planner state before trying again.",
      retryable: false,
      code: "confirmation_denied",
    },
    confirmationId: "confirmation-1",
    promptText: "Submit the form?",
    requestId: "request-1",
    selectedSkills: ["form_submit"],
    nextStepId: "step-2",
    queuedStepIds: ["step-2"],
  });

  const retryableHtml = renderConfirmationPanel({
    kind: "awaiting-confirmation",
    isSubmitting: false,
    submissionError: {
      kind: "tool-error",
      title: "Runtime rejected the request",
      message: "The backend is temporarily unavailable.",
      guidance: "Review the runtime state and try again.",
      retryable: true,
      code: "runtime_busy",
    },
    confirmationId: "confirmation-2",
    promptText: "Submit the form?",
    requestId: "request-2",
    selectedSkills: ["form_submit"],
    nextStepId: "step-2",
    queuedStepIds: ["step-2"],
  });

  const transportHtml = renderConfirmationPanel({
    kind: "awaiting-confirmation",
    isSubmitting: false,
    submissionError: {
      kind: "transport-error",
      title: "Connection problem",
      message: "The app could not reach the confirmation command.",
      guidance: "Check that the runtime is still running, then try again.",
    },
    confirmationId: "confirmation-3",
    promptText: "Submit the form?",
    requestId: "request-3",
    selectedSkills: ["form_submit"],
    nextStepId: "step-2",
    queuedStepIds: ["step-2"],
  });

  return {
    nonRetryableHtml,
    retryableHtml,
    transportHtml,
  };
}

test("renders retry copy only for the matching backend retry state", () => {
  const { nonRetryableHtml, retryableHtml, transportHtml } = renderFixtures();

  assert.match(nonRetryableHtml, /Cannot retry\./);
  assert.doesNotMatch(retryableHtml, /Cannot retry\./);
  assert.doesNotMatch(transportHtml, /Cannot retry\./);

  assert.doesNotMatch(nonRetryableHtml, /Can retry\./);
  assert.match(retryableHtml, /Can retry\./);
  assert.doesNotMatch(transportHtml, /Can retry\./);
});

test("renders the planner-change badge only for non-retryable backend failures", () => {
  const { nonRetryableHtml, retryableHtml, transportHtml } = renderFixtures();

  assert.match(nonRetryableHtml, /Requires planner change/);
  assert.doesNotMatch(retryableHtml, /Requires planner change/);
  assert.doesNotMatch(transportHtml, /Requires planner change/);
});

test("renders the exact backend metadata block for retryable and non-retryable errors", () => {
  const { nonRetryableHtml, retryableHtml, transportHtml } = renderFixtures();

  assert.match(nonRetryableHtml, /<div class="confirmation-error-meta-block">/);
  assert.match(
    nonRetryableHtml,
    /<p class="confirmation-error-meta">\s*Error code: confirmation_denied\. Non-retryable backend failure\.\s*<\/p>/,
  );
  assert.match(
    nonRetryableHtml,
    /<p class="confirmation-error-retry-status">Cannot retry\.<\/p>/,
  );

  assert.match(retryableHtml, /<div class="confirmation-error-meta-block">/);
  assert.match(
    retryableHtml,
    /<p class="confirmation-error-meta">\s*Error code: runtime_busy\. Retryable backend failure\.\s*<\/p>/,
  );
  assert.match(
    retryableHtml,
    /<p class="confirmation-error-retry-status">Can retry\.<\/p>/,
  );

  assert.doesNotMatch(transportHtml, /confirmation-error-meta-block/);
});

test("renders push-to-talk instructions and button label for the idle state", () => {
  const html = renderPushToTalkPanel({
    enabled: true,
    isHolding: false,
    isListening: false,
    isBusy: false,
    lastTranscript: null,
    lastError: null,
  });

  assert.match(
    html,
    /Hold Space or press and hold the button to speak a command\. Say start listening to keep voice input active\./,
  );
  assert.match(html, /Hold to talk/);
  assert.match(html, /data-push-to-talk-button="true"/);
});

test("renders hands-free listening copy when continuous voice input is active", () => {
  const html = renderPushToTalkPanel({
    enabled: true,
    isHolding: false,
    isListening: true,
    isBusy: false,
    lastTranscript: "start listening",
    lastError: null,
  });

  assert.match(html, /Hands-free listening is active\. Say a command, or say stop listening to leave hands-free mode\./);
  assert.match(html, /Last transcript:<\/strong> start listening/);
  assert.match(html, /disabled aria-disabled="true"/);
});

test("renders hands-free listening busy copy while processing the next spoken command", () => {
  const html = renderPushToTalkPanel({
    enabled: true,
    isHolding: false,
    isListening: true,
    isBusy: true,
    lastTranscript: null,
    lastError: null,
  });

  assert.match(html, /Hands-free listening is active and processing the next spoken command\./);
  assert.match(html, /disabled aria-disabled="true"/);
});

test("renders push-to-talk transcript and active button state while holding", () => {
  const html = renderPushToTalkPanel({
    enabled: true,
    isHolding: true,
    isListening: true,
    isBusy: false,
    lastTranscript: "open example dot com",
    lastError: null,
  });

  assert.match(html, /Listening now\. Release to transcribe and run the spoken command\./);
  assert.match(html, /Release to transcribe/);
  assert.match(html, /push-to-talk-button-active/);
  assert.match(html, /Last transcript:<\/strong> open example dot com/);
});

test("renders push-to-talk errors when voice input fails", () => {
  const html = renderPushToTalkPanel({
    enabled: true,
    isHolding: false,
    isListening: false,
    isBusy: false,
    lastTranscript: null,
    lastError: "The microphone is unavailable.",
  });

  assert.match(html, /The microphone is unavailable\./);
  assert.match(html, /role="alert"/);
});

test("renders slider controls with screen-reader value text", () => {
  const audioHtml = renderAudioControlsPanel({
    playbackVolume: 0.67,
    playbackSpeed: 1.25,
    isBusy: false,
    error: null,
  });
  const confirmationHtml = renderSettingsConfirmationPanel({
    confirmationConfidenceThreshold: 0.82,
    allowClickWithoutConfirmation: false,
    alwaysConfirmSubmit: true,
    isBusy: false,
    error: null,
  });

  assert.match(audioHtml, /aria-valuetext="67 percent"/);
  assert.match(audioHtml, /aria-valuetext="1\.25 times"/);
  assert.match(confirmationHtml, /aria-valuetext="82 percent confidence"/);
});

test("renders described settings inputs and grouped confirmation actions", () => {
  renderSettingsRemotePlannerPanel({
    profileName: "remote-default",
    provider: "OpenAI",
    baseUrl: "https://api.example.com",
    model: "gpt-test",
    availableModels: ["gpt-test", "gpt-next"],
    loadedModelsEndpoint: "https://api.example.com",
    isLoadingModels: false,
    isSavingConnection: false,
    isResettingConnection: false,
    apiKeyReference: "keyring:planner",
    organizationReference: null,
    project: null,
    temperatureMilli: 250,
    maxOutputTokens: 1024,
    timeoutMs: 30000,
    apiKeyDraft: "secret",
    isSavingApiKey: false,
    isTestingApiKey: false,
    apiKeyTestMessage: null,
    error: null,
  });
  const modelManagementHtml = renderSettingsModelManagementPanel({
    modelsDir: "/tmp/models",
    checkOnStartup: true,
    autoDownloadMissing: false,
    localTtsAvailable: true,
    localTtsDownloadSupported: true,
    localTtsDownloadLabel: "Download TTS",
    localAsrAvailable: true,
    localAsrDownloadSupported: true,
    localAsrDownloadLabel: "Download ASR",
    isSaving: false,
    isDownloadingTts: false,
    isDownloadingAsr: false,
    error: null,
  });
  const confirmationHtml = renderConfirmationPanel({
    kind: "awaiting-confirmation",
    isSubmitting: false,
    submissionError: null,
    confirmationId: "confirmation-1",
    promptText: "Submit the form?",
    requestId: "request-1",
    selectedSkills: ["form_submit"],
    nextStepId: "step-2",
    queuedStepIds: ["step-2"],
  });

  assert.match(modelManagementHtml, /aria-describedby="settings-models-dir-description"/);
  assert.match(confirmationHtml, /role="group" aria-label="Confirmation actions"/);
});

test("renders URL input with current URL and staged draft value", () => {
  const html = renderUrlInputPanel({
    draftValue: "https://staged.example.com",
    currentUrl: "https://current.example.com",
    hasUnsubmittedChanges: true,
    isOpening: false,
    isReading: false,
    isStopping: false,
    isAdvancing: false,
    isRewinding: false,
    error: null,
  });

  assert.match(html, /URL input/);
  assert.match(html, /Current URL:<\/strong> https:\/\/current\.example\.com/);
  assert.match(html, /Draft URL updated\. Open controls can use this value next\./);
  assert.match(html, /data-url-input="true"/);
  assert.match(html, /data-url-open-button="true"/);
  assert.match(html, /data-url-read-button="true"/);
  assert.match(html, /data-url-stop-button="true"/);
  assert.match(html, /data-url-previous-button="true"/);
  assert.match(html, /data-url-next-button="true"/);
  assert.match(html, />\s*Open\s*<\/button>/);
  assert.match(html, />\s*Read\s*<\/button>/);
  assert.match(html, />\s*Stop\s*<\/button>/);
  assert.match(html, />\s*Previous\s*<\/button>/);
  assert.match(html, />\s*Next\s*<\/button>/);
  assert.match(html, /value="https:\/\/staged\.example\.com"/);
  assert.match(html, /role="status" aria-live="polite" aria-atomic="true"/);
});

test("renders URL input fallback copy when no page is loaded", () => {
  const html = renderUrlInputPanel({
    draftValue: "",
    currentUrl: null,
    hasUnsubmittedChanges: false,
    isOpening: false,
    isReading: false,
    isStopping: false,
    isAdvancing: false,
    isRewinding: false,
    error: null,
  });

  assert.match(html, /No page URL is loaded yet\./);
  assert.match(html, /The field mirrors the current page URL until you edit it\./);
  assert.match(html, /placeholder="https:\/\/example\.com"/);
});

test("renders URL input busy and error states while opening", () => {
  const html = renderUrlInputPanel({
    draftValue: "https://example.com",
    currentUrl: "https://example.com",
    hasUnsubmittedChanges: false,
    isOpening: true,
    isReading: false,
    isStopping: false,
    isAdvancing: false,
    isRewinding: false,
    error: "The browser could not open that URL.",
  });

  assert.match(html, /Opening\.\.\./);
  assert.match(html, /The browser could not open that URL\./);
  assert.match(html, /disabled aria-disabled="true"/);
  assert.match(html, /role="alert"/);
});

test("renders URL input busy state while starting page reading", () => {
  const html = renderUrlInputPanel({
    draftValue: "https://example.com",
    currentUrl: "https://example.com",
    hasUnsubmittedChanges: false,
    isOpening: false,
    isReading: true,
    isStopping: false,
    isAdvancing: false,
    isRewinding: false,
    error: null,
  });

  assert.match(html, /Reading\.\.\./);
  assert.match(html, /disabled aria-disabled="true"/);
});

test("renders URL input busy state while stopping page reading", () => {
  const html = renderUrlInputPanel({
    draftValue: "https://example.com",
    currentUrl: "https://example.com",
    hasUnsubmittedChanges: false,
    isOpening: false,
    isReading: false,
    isStopping: true,
    isAdvancing: false,
    isRewinding: false,
    error: null,
  });

  assert.match(html, /Stopping\.\.\./);
  assert.match(html, /disabled aria-disabled="true"/);
});

test("renders URL input busy state while moving to the next reading region", () => {
  const html = renderUrlInputPanel({
    draftValue: "https://example.com",
    currentUrl: "https://example.com",
    hasUnsubmittedChanges: false,
    isOpening: false,
    isReading: false,
    isStopping: false,
    isAdvancing: true,
    isRewinding: false,
    error: null,
  });

  assert.match(html, /Next\.\.\./);
  assert.match(html, /disabled aria-disabled="true"/);
});

test("renders model management controls and download actions", () => {
  const html = renderSettingsModelManagementPanel({
    modelsDir: "~/.local/share/blind_browser/models",
    checkOnStartup: true,
    autoDownloadMissing: false,
    localTtsAvailable: false,
    localTtsDownloadSupported: true,
    localTtsDownloadLabel: "Download KittenTTS mini model",
    localAsrAvailable: true,
    localAsrDownloadSupported: true,
    localAsrDownloadLabel: "Download Whisper tiny model",
    isSaving: false,
    isDownloadingTts: false,
    isDownloadingAsr: false,
    error: null,
  });

  assert.match(html, /Local models/);
  assert.match(html, /whether startup checks them/i);
  assert.match(html, /Model folder/);
  assert.match(html, /data-model-management-input="models-dir"/);
  assert.match(html, /data-model-management-toggle="check-on-startup"/);
  assert.match(html, /data-model-management-toggle="auto-download-missing"/);
  assert.match(html, /data-model-download="tts"/);
  assert.match(html, /data-model-download="asr"/);
  assert.match(html, /Missing/);
  assert.match(html, /Downloaded/);
  assert.match(html, /Download KittenTTS mini model/);
  assert.match(html, /Download Whisper tiny model/);
});

test("renders model management busy and error states", () => {
  const html = renderSettingsModelManagementPanel({
    modelsDir: "/tmp/models",
    checkOnStartup: false,
    autoDownloadMissing: true,
    localTtsAvailable: false,
    localTtsDownloadSupported: false,
    localTtsDownloadLabel: null,
    localAsrAvailable: false,
    localAsrDownloadSupported: true,
    localAsrDownloadLabel: "Download Whisper tiny model",
    isSaving: true,
    isDownloadingTts: true,
    isDownloadingAsr: false,
    error: "Download failed.",
  });

  assert.match(html, /Download failed\./);
  assert.match(html, /Downloading\.\.\./);
  assert.match(html, /disabled aria-disabled="true"/);
  assert.match(html, /Download Whisper tiny model/);
});

test("renders URL input busy state while moving to the previous reading region", () => {
  const html = renderUrlInputPanel({
    draftValue: "https://example.com",
    currentUrl: "https://example.com",
    hasUnsubmittedChanges: false,
    isOpening: false,
    isReading: false,
    isStopping: false,
    isAdvancing: false,
    isRewinding: true,
    error: null,
  });

  assert.match(html, /Previous\.\.\./);
  assert.match(html, /disabled aria-disabled="true"/);
});

test("renders nearby playback controls with volume and speed values", () => {
  const html = renderAudioControlsPanel({
    playbackVolume: 0.7,
    playbackSpeed: 1.25,
    isBusy: false,
    error: null,
  });

  assert.match(html, /Playback volume and speed/);
  assert.match(html, /saved defaults for future narration/);
  assert.match(html, /Volume/);
  assert.match(html, /70%/);
  assert.match(html, /Speed/);
  assert.match(html, /1\.25x/);
  assert.match(html, /data-audio-control="volume"/);
  assert.match(html, /data-audio-control="speed"/);
});

test("renders settings volume control with the persisted default value", () => {
  const html = renderSettingsVolumePanel({
    playbackVolume: 0.65,
    playbackSpeed: 1.25,
    isBusy: false,
    error: null,
  });

  assert.match(html, /Playback volume/);
  assert.match(html, /Default volume/);
  assert.match(html, /65%/);
  assert.match(html, /persist across app restarts/);
  assert.match(html, /id="settings-playback-volume-control"/);
  assert.match(html, /data-audio-control="volume"/);
});

test("renders settings volume errors and disabled state while saving", () => {
  const html = renderSettingsVolumePanel({
    playbackVolume: 1,
    playbackSpeed: 1,
    isBusy: true,
    error: "The playback volume could not be saved.",
  });

  assert.match(html, /The playback volume could not be saved\./);
  assert.match(html, /disabled aria-disabled="true"/);
  assert.match(html, /role="alert"/);
});

test("renders settings speed control with the persisted default value", () => {
  const html = renderSettingsSpeedPanel({
    playbackVolume: 0.65,
    playbackSpeed: 1.4,
    isBusy: false,
    error: null,
  });

  assert.match(html, /Playback speed/);
  assert.match(html, /Default speed/);
  assert.match(html, /1\.40x/);
  assert.match(html, /persist across app restarts/);
  assert.match(html, /id="settings-playback-speed-control"/);
  assert.match(html, /data-audio-control="speed"/);
});

test("renders settings speed errors and disabled state while saving", () => {
  const html = renderSettingsSpeedPanel({
    playbackVolume: 1,
    playbackSpeed: 2,
    isBusy: true,
    error: "The playback speed could not be saved.",
  });

  assert.match(html, /The playback speed could not be saved\./);
  assert.match(html, /disabled aria-disabled="true"/);
  assert.match(html, /role="alert"/);
});

test("renders remote planner API reference details", () => {
  const html = renderSettingsRemotePlannerPanel({
    profileName: "openai-default",
    provider: "OpenAI",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-5.4-mini",
    availableModels: ["gpt-5.4-mini", "gpt-5.4"],
    loadedModelsEndpoint: "https://api.openai.com/v1",
    isLoadingModels: false,
    isSavingConnection: false,
    isResettingConnection: false,
    apiKeyReference: "Environment variable: OPENAI_API_KEY",
    organizationReference: null,
    project: null,
    temperatureMilli: 200,
    maxOutputTokens: 1024,
    timeoutMs: 30000,
    apiKeyDraft: "",
    isSavingApiKey: false,
    isTestingApiKey: false,
    apiKeyTestMessage: null,
    error: null,
  });

  assert.match(html, /Planner setup/);
  assert.match(html, /endpoint, model, and API key used to interpret commands/i);
  assert.match(html, /Model/);
  assert.match(html, /Endpoint/);
  assert.match(html, /data-remote-planner-endpoint-input="true"/);
  assert.match(html, /data-remote-planner-model-select="true"/);
  assert.match(html, /data-remote-planner-models-refresh="true"/);
  assert.match(html, /data-remote-planner-settings-save="true"/);
  assert.match(html, /data-remote-planner-settings-reset="true"/);
  assert.match(html, /Load models/);
  assert.match(html, /Reset to defaults/);
  assert.match(html, /aria-label="Models are loaded for the current endpoint"/);
  assert.doesNotMatch(html, /Planner remote profile/);
  assert.doesNotMatch(html, /Service/);
  assert.doesNotMatch(html, /Temperature \(milli\)/);
  assert.doesNotMatch(html, /Max output tokens/);
  assert.doesNotMatch(html, /Timeout \(ms\)/);
  assert.match(html, /data-remote-api-key-input="planner"/);
  assert.match(html, /data-remote-api-key-save="planner"/);
  assert.match(html, /data-remote-api-key-test="planner"/);
  assert.match(html, /<a href="https:\/\/platform\.openai\.com\/account\/api-keys" target="_blank" rel="noreferrer" data-external-link-url="https:\/\/platform\.openai\.com\/account\/api-keys">https:\/\/platform\.openai\.com\/account\/api-keys<\/a>/);
});

test("renders settings provider failover as read-only unavailable controls", () => {
  const html = renderSettingsProviderFailoverPanel({
    plannerAvailable: false,
    ttsAvailable: false,
    asrAvailable: false,
    summary: "Automatic provider failover is not currently available in the live runtime.",
  });

  assert.match(html, /Failover/);
  assert.match(html, /not available yet/i);
  assert.match(html, />Planner</);
  assert.match(html, />TTS</);
  assert.match(html, />ASR</);
  assert.match(html, /data-provider-failover-toggle="planner"/);
  assert.match(html, /data-provider-failover-toggle="tts"/);
  assert.match(html, /data-provider-failover-toggle="asr"/);
  assert.match(html, /Unavailable/);
  assert.match(html, /aria-disabled="true"/);
});

test("renders settings confirmation behavior controls", () => {
  const html = renderSettingsConfirmationPanel({
    confirmationConfidenceThreshold: 0.82,
    allowClickWithoutConfirmation: true,
    alwaysConfirmSubmit: true,
    isBusy: false,
    error: null,
  });

  assert.match(html, /Confirmation/);
  assert.match(html, /Form submits\s+still always require confirmation/i);
  assert.match(html, /Click threshold/);
  assert.match(html, /82%/);
  assert.match(html, /data-confirmation-threshold-control="true"/);
  assert.match(html, /data-click-without-confirmation-toggle="true"/);
  assert.match(html, /Always require confirmation/);
});

test("renders confirmation settings errors and disabled state while saving", () => {
  const html = renderSettingsConfirmationPanel({
    confirmationConfidenceThreshold: 0.9,
    allowClickWithoutConfirmation: false,
    alwaysConfirmSubmit: true,
    isBusy: true,
    error: "The confirmation settings could not be saved.",
  });

  assert.match(html, /The confirmation settings could not be saved\./);
  assert.match(html, /disabled aria-disabled="true"/);
  assert.match(html, /role="alert"/);
});

test("renders OCR threshold settings controls", () => {
  const html = renderSettingsOcrThresholdPanel({
    sparseTextCharThreshold: 120,
    sparseTextRegionThreshold: 3,
    isBusy: false,
    error: null,
  });

  assert.match(html, /OCR fallback/);
  assert.match(html, /fall back to OCR/i);
  assert.match(html, /Character threshold/);
  assert.match(html, /Region threshold/);
  assert.match(html, /data-ocr-threshold-control="char"/);
  assert.match(html, /data-ocr-threshold-control="region"/);
  assert.match(html, /value="120"/);
  assert.match(html, /value="3"/);
});

test("renders OCR threshold settings errors and disabled state while saving", () => {
  const html = renderSettingsOcrThresholdPanel({
    sparseTextCharThreshold: 200,
    sparseTextRegionThreshold: 2,
    isBusy: true,
    error: "The OCR thresholds could not be saved.",
  });

  assert.match(html, /The OCR thresholds could not be saved\./);
  assert.match(html, /disabled aria-disabled="true"/);
  assert.match(html, /role="alert"/);
});

test("renders settings guidance panel for model-related errors", () => {
  const html = renderSettingsGuidancePanel({
    title: "Model setup needs attention",
    message: "The current local TTS model could not be loaded. Review the TTS settings below.",
    actions: [
      { label: "Review TTS provider", targetId: "settings-tts-provider-control" },
      { label: "Review TTS model", targetId: "settings-tts-model-control" },
    ],
  });

  assert.match(html, /Model setup needs attention/);
  assert.match(html, /could not be loaded/);
  assert.match(html, /data-settings-target="settings-tts-provider-control"/);
  assert.match(html, /data-settings-target="settings-tts-model-control"/);
});

test("renders settings guidance panel with a clickable OpenAI API key link", () => {
  const html = renderSettingsGuidancePanel({
    title: "Remote ASR secret needs attention",
    message: "Get an OpenAI API key at https://platform.openai.com/account/api-keys if needed.",
    actions: [{ label: "Enter remote ASR API key", targetId: "settings-remote-asr-api-key-input" }],
  });

  assert.match(html, /<a href="https:\/\/platform\.openai\.com\/account\/api-keys" target="_blank" rel="noreferrer" data-external-link-url="https:\/\/platform\.openai\.com\/account\/api-keys">https:\/\/platform\.openai\.com\/account\/api-keys<\/a>/);
});

test("renders settings ASR provider selection for configured modes", () => {
  const html = renderSettingsAsrProviderPanel({
    activeMode: "Local",
    availableModes: ["Local", "Remote"],
    isBusy: false,
    error: null,
  });

  assert.match(html, /ASR provider/);
  assert.match(html, /local or remote speech-to-text provider/i);
  assert.match(html, /Local provider/);
  assert.match(html, /Remote provider/);
  assert.match(html, /data-asr-provider-select="true"/);
});

test("renders settings ASR provider errors and disabled state while saving", () => {
  const html = renderSettingsAsrProviderPanel({
    activeMode: "Remote",
    availableModes: ["Local", "Remote"],
    isBusy: true,
    error: "The ASR provider selection could not be saved.",
  });

  assert.match(html, /The ASR provider selection could not be saved\./);
  assert.match(html, /disabled aria-disabled="true"/);
  assert.match(html, /role="alert"/);
});

test("renders local ASR model reference details", () => {
  const html = renderSettingsLocalAsrModelPanel({
    profileName: "whisper-default",
    backend: "whisper",
    modelId: "tiny",
    modelPath: "/models/whisper/tiny.bin",
    language: "en",
    threads: 4,
  });

  assert.match(html, /Local ASR profile/);
  assert.match(html, /speech-to-text profile used when ASR runs in local mode/i);
  assert.match(html, /whisper-default/);
  assert.match(html, /\/models\/whisper\/tiny\.bin/);
  assert.match(html, /edit the app config/i);
});

test("renders remote ASR API reference details", () => {
  const html = renderSettingsRemoteAsrPanel({
    profileName: "openai-transcribe-default",
    provider: "OpenAI",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-4o-mini-transcribe",
    apiKeyReference: "Environment variable: OPENAI_API_KEY",
    apiKeyMaskedValue: null,
    organizationReference: null,
    project: null,
    language: "en",
    temperatureMilli: 0,
    timeoutMs: 30000,
    apiKeyDraft: "",
    isSavingApiKey: false,
    isTestingApiKey: false,
    apiKeyTestMessage: null,
    error: null,
  });

  assert.match(html, /Remote ASR profile/);
  assert.match(html, /speech-to-text profile used when ASR runs in remote mode/i);
  assert.match(html, /openai-transcribe-default/);
  assert.match(html, /OPENAI_API_KEY/);
  assert.match(html, /gpt-4o-mini-transcribe/);
  assert.match(html, /data-remote-api-key-input="asr"/);
  assert.match(html, /Save API key/);
  assert.match(html, /Test API key/);
});

test("renders a masked remote ASR API key value when a key is already configured", () => {
  const html = renderSettingsRemoteAsrPanel({
    profileName: "openai-transcribe-default",
    provider: "OpenAI",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-4o-mini-transcribe",
    apiKeyReference: "OS keyring entry: blind_browser / remote_asr:openai-transcribe-default:api_key",
    apiKeyMaskedValue: "***2468",
    organizationReference: null,
    project: null,
    language: "en",
    temperatureMilli: 0,
    timeoutMs: 30000,
    apiKeyDraft: "",
    isSavingApiKey: false,
    isTestingApiKey: false,
    apiKeyTestMessage: null,
    error: null,
  });

  assert.match(html, /type="text"/);
  assert.match(html, /value="\*\*\*2468"/);
  assert.match(html, /data-masked-api-key-display="\*\*\*2468"/);
});

test("renders settings TTS provider selection for configured modes", () => {
  const html = renderSettingsTtsProviderPanel({
    activeMode: "Local",
    availableModes: ["Local", "Remote"],
    isBusy: false,
    error: null,
  });

  assert.match(html, /TTS provider/);
  assert.match(html, /local or remote speech output provider/i);
  assert.match(html, /Local provider/);
  assert.match(html, /Remote provider/);
  assert.match(html, /data-tts-provider-select="true"/);
});

test("renders settings TTS provider errors and disabled state while saving", () => {
  const html = renderSettingsTtsProviderPanel({
    activeMode: "Remote",
    availableModes: ["Local", "Remote"],
    isBusy: true,
    error: "The TTS provider selection could not be saved.",
  });

  assert.match(html, /The TTS provider selection could not be saved\./);
  assert.match(html, /disabled aria-disabled="true"/);
  assert.match(html, /role="alert"/);
});

test("renders settings TTS model selection for configured profiles", () => {
  const html = renderSettingsTtsModelPanel({
    mode: "Local",
    activeProfile: "kitten-default",
    availableProfiles: [
      { profileName: "kitten-default", modelLabel: "default" },
      { profileName: "kitten-large", modelLabel: "large-v1" },
    ],
    isBusy: false,
    error: null,
  });

  assert.match(html, /TTS model/);
  assert.match(html, /local TTS model for the current mode/i);
  assert.match(html, /default \(kitten-default\)/);
  assert.match(html, /large-v1 \(kitten-large\)/);
  assert.match(html, /data-tts-model-select="true"/);
});

test("renders local TTS model reference details", () => {
  const html = renderSettingsLocalTtsModelPanel({
    profileName: "kitten-default",
    backend: "kitten_tts_rs",
    modelId: "default",
    modelPath: "/models/kitten/default",
    defaultVoice: "Bruno",
    sampleRate: 24000,
  });

  assert.match(html, /Local TTS profile/);
  assert.match(html, /local speech profile used when TTS runs in local mode/i);
  assert.match(html, /kitten-default/);
  assert.match(html, /\/models\/kitten\/default/);
  assert.match(html, /24000/);
});

test("renders remote TTS API reference details", () => {
  const html = renderSettingsRemoteTtsPanel({
    profileName: "openai-tts-default",
    provider: "OpenAI",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-4o-mini-tts",
    apiKeyReference: "Environment variable: OPENAI_API_KEY",
    apiKeyMaskedValue: null,
    organizationReference: null,
    project: null,
    voice: "alloy",
    audioFormat: "wav",
    timeoutMs: 30000,
    apiKeyDraft: "",
    isSavingApiKey: false,
    isTestingApiKey: false,
    apiKeyTestMessage: null,
    error: null,
  });

  assert.match(html, /Remote TTS profile/);
  assert.match(html, /speech profile used when TTS runs in remote mode/i);
  assert.match(html, /openai-tts-default/);
  assert.match(html, /OPENAI_API_KEY/);
  assert.match(html, /alloy/);
  assert.match(html, /data-remote-api-key-input="tts"/);
  assert.match(html, /Test API key/);
});

test("renders a masked remote TTS API key value when a key is already configured", () => {
  const html = renderSettingsRemoteTtsPanel({
    profileName: "openai-tts-default",
    provider: "OpenAI",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-4o-mini-tts",
    apiKeyReference: "OS keyring entry: blind_browser / remote_tts:openai-tts-default:api_key",
    apiKeyMaskedValue: "***1357",
    organizationReference: null,
    project: null,
    voice: "alloy",
    audioFormat: "wav",
    timeoutMs: 30000,
    apiKeyDraft: "",
    isSavingApiKey: false,
    isTestingApiKey: false,
    apiKeyTestMessage: null,
    error: null,
  });

  assert.match(html, /type="text"/);
  assert.match(html, /value="\*\*\*1357"/);
  assert.match(html, /data-masked-api-key-display="\*\*\*1357"/);
});

test("renders remote planner API key save errors and disabled state while saving", () => {
  const html = renderSettingsRemotePlannerPanel({
    profileName: "openai-default",
    provider: "OpenAI",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-5.4-mini",
    availableModels: ["gpt-5.4-mini"],
    loadedModelsEndpoint: "https://api.openai.com/v1",
    isLoadingModels: false,
    isSavingConnection: false,
    isResettingConnection: false,
    apiKeyReference: "OS keyring entry: blind_browser / remote_planner:openai-default:api_key",
    organizationReference: null,
    project: null,
    temperatureMilli: 200,
    maxOutputTokens: 1024,
    timeoutMs: 30000,
    apiKeyDraft: "secret-value",
    isSavingApiKey: true,
    isTestingApiKey: false,
    apiKeyTestMessage: null,
    error: "The remote planner API key could not be saved.",
  });

  assert.match(html, /The remote planner API key could not be saved\./);
  assert.match(html, /disabled aria-disabled="true"/);
  assert.match(html, /role="alert"/);
});

test("renders remote planner API key test status while testing", () => {
  const html = renderSettingsRemotePlannerPanel({
    profileName: "openai-default",
    provider: "OpenAI",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-5.4-mini",
    availableModels: ["gpt-5.4-mini"],
    loadedModelsEndpoint: "https://api.openai.com/v1",
    isLoadingModels: false,
    isSavingConnection: false,
    isResettingConnection: false,
    apiKeyReference: "Environment variable: OPENAI_API_KEY",
    apiKeyMaskedValue: null,
    organizationReference: null,
    project: null,
    temperatureMilli: 200,
    maxOutputTokens: 1024,
    timeoutMs: 30000,
    apiKeyDraft: "",
    isSavingApiKey: false,
    isTestingApiKey: true,
    apiKeyTestMessage: "OpenAI accepted the configured API key.",
    error: null,
  });

  assert.match(html, /Testing\.\.\./);
  assert.match(html, /Latest test result/);
  assert.match(html, /OpenAI accepted the configured API key\./);
  assert.match(html, /settings-api-key-test-status/);
  assert.match(html, /role="status"/);
  assert.match(html, /aria-label="Models are loaded for the current endpoint"/);
});

test("renders a masked planner API key value when a key is already configured", () => {
  const html = renderSettingsRemotePlannerPanel({
    profileName: "openai-default",
    provider: "OpenAI",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-5.4-mini",
    availableModels: ["gpt-5.4-mini"],
    loadedModelsEndpoint: "https://api.openai.com/v1",
    isLoadingModels: false,
    isSavingConnection: false,
    isResettingConnection: false,
    apiKeyReference: "OS keyring entry: blind_browser / remote_planner:openai-default:api_key",
    apiKeyMaskedValue: "***7890",
    organizationReference: null,
    project: null,
    temperatureMilli: 200,
    maxOutputTokens: 1024,
    timeoutMs: 30000,
    apiKeyDraft: "",
    isSavingApiKey: false,
    isTestingApiKey: false,
    apiKeyTestMessage: null,
    error: null,
  });

  assert.match(html, /type="text"/);
  assert.match(html, /value="\*\*\*7890"/);
  assert.match(html, /data-masked-api-key-display="\*\*\*7890"/);
});

test("renders remote planner API key test failures with a clickable OpenAI API key link", () => {
  const html = renderSettingsRemotePlannerPanel({
    profileName: "openai-default",
    provider: "OpenAI",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-5.4-mini",
    availableModels: ["gpt-5.4-mini"],
    loadedModelsEndpoint: "https://api.openai.com/v1",
    isLoadingModels: false,
    isSavingConnection: false,
    isResettingConnection: false,
    apiKeyReference: "Environment variable: OPENAI_API_KEY",
    organizationReference: null,
    project: null,
    temperatureMilli: 200,
    maxOutputTokens: 1024,
    timeoutMs: 30000,
    apiKeyDraft: "",
    isSavingApiKey: false,
    isTestingApiKey: false,
    apiKeyTestMessage: "OpenAI rejected that API key. Check the key and try again, or create one at https://platform.openai.com/account/api-keys.",
    error: null,
  });

  assert.match(html, /role="status"/);
  assert.match(html, /Latest test result/);
  assert.match(html, /<a href="https:\/\/platform\.openai\.com\/account\/api-keys" target="_blank" rel="noreferrer" data-external-link-url="https:\/\/platform\.openai\.com\/account\/api-keys">https:\/\/platform\.openai\.com\/account\/api-keys<\/a>/);
});

test("renders stale planner model indicator when endpoint models need reload", () => {
  const html = renderSettingsRemotePlannerPanel({
    profileName: "openai-default",
    provider: "OpenAI",
    baseUrl: "https://api.example.com/v1",
    model: "gpt-5.4-mini",
    availableModels: [],
    loadedModelsEndpoint: null,
    isLoadingModels: false,
    isSavingConnection: false,
    isResettingConnection: false,
    apiKeyReference: "Environment variable: OPENAI_API_KEY",
    organizationReference: null,
    project: null,
    temperatureMilli: 200,
    maxOutputTokens: 1024,
    timeoutMs: 30000,
    apiKeyDraft: "",
    isSavingApiKey: false,
    isTestingApiKey: false,
    apiKeyTestMessage: null,
    error: null,
  });

  assert.match(html, /aria-label="Models need to be reloaded for the current endpoint"/);
});

test("renders settings TTS model errors and disabled state while saving", () => {
  const html = renderSettingsTtsModelPanel({
    mode: "Remote",
    activeProfile: "openai-tts-default",
    availableProfiles: [{ profileName: "openai-tts-default", modelLabel: "gpt-4o-mini-tts" }],
    isBusy: true,
    error: "The TTS model selection could not be saved.",
  });

  assert.match(html, /remote TTS model for the current mode/i);
  assert.match(html, /The TTS model selection could not be saved\./);
  assert.match(html, /disabled aria-disabled="true"/);
  assert.match(html, /role="alert"/);
});

test("renders settings voice selection for configured voices", () => {
  const html = renderSettingsTtsVoicePanel({
    mode: "Local",
    activeVoice: "Bruno",
    availableVoices: [
      { voiceName: "Bella", displayLabel: "Bella" },
      { voiceName: "Bruno", displayLabel: "Bruno" },
    ],
    isBusy: false,
    error: null,
  });

  assert.match(html, /Voice/);
  assert.match(html, /local TTS voice for the current mode/i);
  assert.match(html, /Selected voice/);
  assert.match(html, /Bruno/);
  assert.match(html, /data-tts-voice-select="true"/);
  assert.match(html, /<option value="Bruno" selected>Bruno<\/option>/);
});

test("renders settings voice errors and disabled state while saving", () => {
  const html = renderSettingsTtsVoicePanel({
    mode: "Remote",
    activeVoice: "alloy",
    availableVoices: [{ voiceName: "alloy", displayLabel: "alloy" }],
    isBusy: true,
    error: "The runtime could not save that voice.",
  });

  assert.match(html, /The runtime could not save that voice\./);
  assert.match(html, /remote TTS voice for the current mode/i);
  assert.match(html, /disabled aria-disabled="true"/);
});

test("disables nearby playback controls while audio settings are saving", () => {
  const html = renderAudioControlsPanel({
    playbackVolume: 1,
    playbackSpeed: 1,
    isBusy: true,
    error: null,
  });

  assert.match(html, /disabled aria-disabled="true"/);
});

test("renders nearby playback control errors when syncing fails", () => {
  const html = renderAudioControlsPanel({
    playbackVolume: 1,
    playbackSpeed: 1,
    isBusy: false,
    error: "The audio settings could not be loaded.",
  });

  assert.match(html, /The audio settings could not be loaded\./);
  assert.match(html, /role="alert"/);
});

test("renders runtime status details from agent state", () => {
  const html = renderStatusPanel({
    pageTitle: "Example Domain",
    currentRegionLabel: "Region 3",
    lastTranscript: "open example dot com",
    listening: true,
    speaking: false,
    browserVisibility: "Headless",
    canGoBack: true,
    canGoForward: false,
    isUpdatingVisibility: false,
    error: null,
  });

  assert.match(html, /Current browser state/);
  assert.match(html, /Example Domain/);
  assert.match(html, /Region 3/);
  assert.match(html, /Last transcript/);
  assert.match(html, /open example dot com/);
  assert.match(html, /Listening/);
  assert.match(html, /Active/);
  assert.match(html, /Browser mode/);
  assert.match(html, /Headless/);
  assert.match(html, /data-browser-visibility-mode="Visible"/);
  assert.match(html, /data-browser-visibility-mode="Headless"/);
  assert.match(html, /Back: Available\./);
  assert.match(html, /Forward: Unavailable\./);
});

test("maps agent state browser visibility into status panel state", () => {
  const statusState = statusPanelStateFromAgentState({
    title: null,
    url: "https://example.com/docs",
    narration_cursor: { node_index: 2 },
    last_transcript: "go headless",
    listening_state: { is_listening: true },
    speaking: false,
    browser_visibility: "Headless",
    browser_history: {
      can_go_back: true,
      can_go_forward: false,
    },
  });

  assert.deepEqual(statusState, {
    pageTitle: "https://example.com/docs",
    currentRegionLabel: "Region 3",
    lastTranscript: "go headless",
    listening: true,
    speaking: false,
    browserVisibility: "Headless",
    canGoBack: true,
    canGoForward: false,
    isUpdatingVisibility: false,
    error: null,
  });
});

test("renders status panel fallbacks and errors when runtime sync fails", () => {
  const html = renderStatusPanel({
    pageTitle: null,
    currentRegionLabel: null,
    lastTranscript: null,
    listening: false,
    speaking: false,
    browserVisibility: "Visible",
    canGoBack: false,
    canGoForward: false,
    isUpdatingVisibility: false,
    error: "The runtime state could not be loaded.",
  });

  assert.match(html, /No page open yet/);
  assert.match(html, /No current region/);
  assert.match(html, /No spoken command captured yet/);
  assert.match(html, /The runtime state could not be loaded\./);
  assert.match(html, /role="alert"/);
});

test("disables browser visibility toggle buttons while visibility changes are in flight", () => {
  const html = renderStatusPanel({
    pageTitle: "Example Domain",
    currentRegionLabel: "Region 1",
    lastTranscript: "show browser",
    listening: false,
    speaking: false,
    browserVisibility: "Visible",
    canGoBack: false,
    canGoForward: false,
    isUpdatingVisibility: true,
    error: null,
  });

  assert.match(html, /disabled aria-disabled="true"/);
  assert.match(html, /status-toggle-button-active/);
});
