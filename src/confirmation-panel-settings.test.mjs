import assert from "node:assert/strict";
import test from "node:test";

import {
  renderConfirmationPanel,
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
  renderSettingsTtsModelPanel,
  renderSettingsTtsProviderPanel,
  renderSettingsTtsVoicePanel,
} from "./confirmation-panel-test-helpers.mjs";

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
    isConfirmingReset: false,
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
  assert.match(html, /btn-spinner/);
  assert.match(html, /disabled aria-disabled="true"/);
  assert.match(html, /Download Whisper tiny model/);
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
    isConfirmingReset: false,
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

  assert.match(html, /AI assistant setup/);
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
  assert.match(html, /Models are loaded for the current endpoint/);
  assert.match(html, /Up to date/);
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

  assert.match(html, /Action confirmation/);
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

  assert.match(html, /Screen reading fallback/);
  assert.match(html, /fall back to image text recognition/i);
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

  assert.match(html, /Voice input provider/);
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
    modelAvailable: null,
  });

  assert.match(html, /Local voice input profile/);
  assert.match(html, /speech-to-text profile used when voice input runs in local mode/i);
  assert.match(html, /whisper-default/);
  assert.match(html, /\/models\/whisper\/tiny\.bin/);
  assert.match(html, /edit the app config/i);
});

test("renders local ASR model missing warning when modelAvailable is false", () => {
  const html = renderSettingsLocalAsrModelPanel({
    profileName: "whisper-default",
    backend: "whisper",
    modelId: "tiny",
    modelPath: "/models/whisper/tiny.bin",
    language: "en",
    threads: 4,
    modelAvailable: false,
  });

  assert.match(html, /Model not downloaded yet/i);
  assert.match(html, /Advanced settings/);
  assert.match(html, /role="alert"/);
  assert.match(html, /data-open-runtime-settings="true"/);
});

test("does not render local ASR model warning when modelAvailable is true", () => {
  const html = renderSettingsLocalAsrModelPanel({
    profileName: "whisper-default",
    backend: "whisper",
    modelId: "tiny",
    modelPath: "/models/whisper/tiny.bin",
    language: "en",
    threads: 4,
    modelAvailable: true,
  });

  assert.doesNotMatch(html, /Model not downloaded yet/i);
  assert.doesNotMatch(html, /data-open-runtime-settings="true"/);
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

  assert.match(html, /Remote voice input profile/);
  assert.match(html, /speech-to-text profile used when voice input runs in remote mode/i);
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

  assert.match(html, /Voice output provider/);
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

  assert.match(html, /Voice model/);
  assert.match(html, /local voice model for the current mode/i);
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
    modelAvailable: null,
  });

  assert.match(html, /Local voice output profile/);
  assert.match(html, /local speech profile used when voice output runs in local mode/i);
  assert.match(html, /kitten-default/);
  assert.match(html, /\/models\/kitten\/default/);
  assert.match(html, /24000/);
});

test("renders local TTS model missing warning when modelAvailable is false", () => {
  const html = renderSettingsLocalTtsModelPanel({
    profileName: "kitten-default",
    backend: "kitten_tts_rs",
    modelId: "default",
    modelPath: "/models/kitten/default",
    defaultVoice: "Bruno",
    sampleRate: 24000,
    modelAvailable: false,
  });

  assert.match(html, /Model not downloaded yet/i);
  assert.match(html, /Advanced settings/);
  assert.match(html, /role="alert"/);
  assert.match(html, /data-open-runtime-settings="true"/);
});

test("does not render local TTS model warning when modelAvailable is true", () => {
  const html = renderSettingsLocalTtsModelPanel({
    profileName: "kitten-default",
    backend: "kitten_tts_rs",
    modelId: "default",
    modelPath: "/models/kitten/default",
    defaultVoice: "Bruno",
    sampleRate: 24000,
    modelAvailable: true,
  });

  assert.doesNotMatch(html, /Model not downloaded yet/i);
  assert.doesNotMatch(html, /data-open-runtime-settings="true"/);
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

  assert.match(html, /Remote voice output profile/);
  assert.match(html, /speech profile used when voice output runs in remote mode/i);
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
    isConfirmingReset: false,
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
    isConfirmingReset: false,
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
  assert.match(html, /btn-spinner/);
  assert.match(html, /Latest test result/);
  assert.match(html, /OpenAI accepted the configured API key\./);
  assert.match(html, /settings-api-key-test-status/);
  assert.match(html, /role="status"/);
  assert.match(html, /Models are loaded for the current endpoint/);
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
    isConfirmingReset: false,
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
    isConfirmingReset: false,
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
    isConfirmingReset: false,
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

  assert.match(html, /Models need to be reloaded for the current endpoint/);
  assert.match(html, /Reload needed/);
});

test("renders spinner on the Load models button while models are loading", () => {
  const html = renderSettingsRemotePlannerPanel({
    profileName: "openai-default",
    provider: "OpenAI",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-5.4-mini",
    availableModels: [],
    loadedModelsEndpoint: null,
    isLoadingModels: true,
    isSavingConnection: false,
    isResettingConnection: false,
    isConfirmingReset: false,
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

  assert.match(html, /Loading models\.\.\./);
  assert.match(html, /btn-spinner/);
});

test("renders spinner on the Save settings button while saving", () => {
  const html = renderSettingsRemotePlannerPanel({
    profileName: "openai-default",
    provider: "OpenAI",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-5.4-mini",
    availableModels: ["gpt-5.4-mini"],
    loadedModelsEndpoint: "https://api.openai.com/v1",
    isLoadingModels: false,
    isSavingConnection: true,
    isResettingConnection: false,
    isConfirmingReset: false,
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

  assert.match(html, /Saving\.\.\./);
  assert.match(html, /btn-spinner/);
});

test("renders settings TTS model errors and disabled state while saving", () => {
  const html = renderSettingsTtsModelPanel({
    mode: "Remote",
    activeProfile: "openai-tts-default",
    availableProfiles: [{ profileName: "openai-tts-default", modelLabel: "gpt-4o-mini-tts" }],
    isBusy: true,
    error: "The TTS model selection could not be saved.",
  });

  assert.match(html, /remote voice model for the current mode/i);
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
  assert.match(html, /local voice for the current mode/i);
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
  assert.match(html, /remote voice for the current mode/i);
  assert.match(html, /disabled aria-disabled="true"/);
});
