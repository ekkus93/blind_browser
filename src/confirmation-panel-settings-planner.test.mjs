import assert from "node:assert/strict";
import test from "node:test";

import {
  renderConfirmationPanel,
  renderSettingsConfirmationPanel,
  renderSettingsGuidancePanel,
  renderSettingsModelManagementPanel,
  renderSettingsOcrThresholdPanel,
  renderSettingsProviderFailoverPanel,
  renderSettingsRemotePlannerPanel,
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
  assert.match(html, /disabled="" aria-disabled="true"/);
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
  assert.match(html, /endpoint, model, API key, and fail-closed site privacy rules used to interpret commands/i);
  assert.match(html, /Model/);
  assert.match(html, /Endpoint/);
  assert.match(html, /data-remote-planner-endpoint-input="true"/);
  assert.match(html, /data-remote-planner-model-select="true"/);
  assert.match(html, /data-remote-planner-models-refresh="true"/);
  assert.match(html, /data-remote-planner-settings-save="true"/);
  assert.match(html, /data-remote-planner-settings-reset="true"/);
  assert.match(html, /Refresh model list/);
  assert.match(html, /Reset to defaults/);
  assert.match(html, /Model list is loaded for the current endpoint/);
  assert.match(html, /Model list up to date/);
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
  assert.match(html, /disabled="" aria-disabled="true"/);
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
  assert.match(html, /disabled="" aria-disabled="true"/);
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
  assert.match(html, /disabled="" aria-disabled="true"/);
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
  assert.match(html, /Model list is loaded for the current endpoint/);
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

  assert.match(html, /Model list has not been loaded for the current endpoint/);
  assert.match(html, /Model list may be outdated/);
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
  assert.match(html, /settings-inline-loading/);
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

test("manual planner model does not render as a verified loaded model list", () => {
  const html = renderSettingsRemotePlannerPanel({
    profileName: "openai-default",
    provider: "OpenAI",
    baseUrl: "https://api.example.com/v1",
    model: "gpt-manual",
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

  assert.match(html, /value="gpt-manual"/);
  assert.match(html, /Model list may be outdated/);
  assert.doesNotMatch(html, /Model list up to date/);
  assert.doesNotMatch(html, /<select[^>]*data-remote-planner-model-select/);
});

test("verified model list shows fresh indicator after successful load", () => {
  const html = renderSettingsRemotePlannerPanel({
    profileName: "openai-default",
    provider: "OpenAI",
    baseUrl: "https://api.example.com/v1",
    model: "gpt-test",
    availableModels: ["gpt-test", "gpt-other"],
    loadedModelsEndpoint: "https://api.example.com/v1",
    isLoadingModels: false,
    isSavingConnection: false,
    isResettingConnection: false,
    isConfirmingReset: false,
    apiKeyReference: null,
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

  assert.match(html, /Model list up to date/);
  assert.match(html, /Model list is loaded for the current endpoint/);
  assert.match(html, /<select[^>]*data-remote-planner-model-select/);
  assert.doesNotMatch(html, /Model list may be outdated/);
});

test("renders api key reference error warning in remote planner panel", () => {
  const html = renderSettingsRemotePlannerPanel({
    profileName: "openai-default",
    provider: "OpenAI",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-4o",
    availableModels: [],
    loadedModelsEndpoint: null,
    isLoadingModels: false,
    isSavingConnection: false,
    isResettingConnection: false,
    isConfirmingReset: false,
    apiKeyReference: "OS keyring entry: blind_browser / remote_planner:openai-default:api_key",
    apiKeyMaskedValue: null,
    apiKeyReferenceError: "Configured secret could not be inspected: keyring service unavailable",
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

  assert.match(html, /role="alert"/);
  assert.match(html, /keyring service unavailable/);
});
