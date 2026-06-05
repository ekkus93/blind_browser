import assert from "node:assert/strict";
import test from "node:test";
import { isValidElement } from "react";

import {
  renderAudioControlsPanelNode,
  renderConfirmationPanelNode,
  renderPushToTalkPanelNode,
  renderSettingsAsrProviderPanelNode,
  renderSettingsConfirmationPanelNode,
  renderSettingsGuidancePanelNode,
  renderSettingsLocalAsrModelPanelNode,
  renderSettingsLocalTtsModelPanelNode,
  renderSettingsModelManagementPanelNode,
  renderSettingsOcrThresholdPanelNode,
  renderSettingsProviderFailoverPanelNode,
  renderSettingsRemoteAsrPanelNode,
  renderSettingsRemotePlannerPanelNode,
  renderSettingsRemoteTtsPanelNode,
  renderSettingsTtsProviderPanelNode,
  renderSettingsTtsModelPanelNode,
  renderSettingsTtsVoicePanelNode,
  renderStatusPanelNode,
  statusPanelStateFromAgentState,
  renderUrlInputPanelNode,
  renderVoiceStatusStripNode,
} from "./confirmation-panel.ts";

const VOID_ELEMENTS = new Set(["area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr"]);

function escapeHtml(value) {
  return String(value)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function mapAttributeName(name) {
  switch (name) {
    case "className":
      return "class";
    case "htmlFor":
      return "for";
    case "inputMode":
      return "inputmode";
    case "autoComplete":
      return "autocomplete";
    case "spellCheck":
      return "spellcheck";
    default:
      return name;
  }
}

function renderNodeMarkup(node, parentContext = {}) {
  if (node === null || node === undefined || typeof node === "boolean") {
    return "";
  }

  if (typeof node === "string" || typeof node === "number") {
    return escapeHtml(node);
  }

  if (Array.isArray(node)) {
    return node.map((child) => renderNodeMarkup(child, parentContext)).join("");
  }

  if (!isValidElement(node)) {
    return "";
  }

  if (typeof node.type !== "string") {
    return renderNodeMarkup(node.props.children, parentContext);
  }

  const { children, ...rawProps } = node.props ?? {};
  const props = { ...rawProps };
  if (
    node.type === "option"
    && props.selected === undefined
    && parentContext.selectedValue !== undefined
    && props.value === parentContext.selectedValue
  ) {
    props.selected = true;
  }

  const attributes = Object.entries(props)
    .filter(([name, value]) => name !== "key" && name !== "ref" && name !== "children" && name !== "dangerouslySetInnerHTML" && name !== "onChange" && name !== "readOnly" && value !== undefined && value !== null && value !== false)
    .map(([name, value]) => {
      const attributeName = mapAttributeName(name);
      if (value === true) {
        return ` ${attributeName}`;
      }

      return ` ${attributeName}="${escapeHtml(value)}"`;
    })
    .join("");

  if (VOID_ELEMENTS.has(node.type)) {
    return `<${node.type}${attributes}>`;
  }

  const nextContext = node.type === "select"
    ? { ...parentContext, selectedValue: props.value }
    : parentContext;

  return `<${node.type}${attributes}>${renderNodeMarkup(children, nextContext)}</${node.type}>`;
}

function renderConfirmationPanel(state) {
  return renderNodeMarkup(renderConfirmationPanelNode(state));
}

function renderPushToTalkPanel(state) {
  return renderNodeMarkup(renderPushToTalkPanelNode(state));
}

function renderAudioControlsPanel(state) {
  return renderNodeMarkup(renderAudioControlsPanelNode(state));
}

function renderSettingsAsrProviderPanel(state) {
  return renderNodeMarkup(renderSettingsAsrProviderPanelNode(state));
}

function renderSettingsConfirmationPanel(state) {
  return renderNodeMarkup(renderSettingsConfirmationPanelNode(state));
}

function renderSettingsGuidancePanel(state) {
  return renderNodeMarkup(renderSettingsGuidancePanelNode(state));
}

function renderSettingsLocalAsrModelPanel(state) {
  return renderNodeMarkup(renderSettingsLocalAsrModelPanelNode(state));
}

function renderSettingsLocalTtsModelPanel(state) {
  return renderNodeMarkup(renderSettingsLocalTtsModelPanelNode(state));
}

function renderSettingsModelManagementPanel(state) {
  return renderNodeMarkup(renderSettingsModelManagementPanelNode(state));
}

function renderSettingsOcrThresholdPanel(state) {
  return renderNodeMarkup(renderSettingsOcrThresholdPanelNode(state));
}

function renderSettingsProviderFailoverPanel(state) {
  return renderNodeMarkup(renderSettingsProviderFailoverPanelNode(state));
}

function renderSettingsRemoteAsrPanel(state) {
  return renderNodeMarkup(renderSettingsRemoteAsrPanelNode(state));
}

function renderSettingsRemotePlannerPanel(state) {
  return renderNodeMarkup(renderSettingsRemotePlannerPanelNode(state));
}

function renderSettingsRemoteTtsPanel(state) {
  return renderNodeMarkup(renderSettingsRemoteTtsPanelNode(state));
}

function renderSettingsTtsProviderPanel(state) {
  return renderNodeMarkup(renderSettingsTtsProviderPanelNode(state));
}

function renderSettingsTtsModelPanel(state) {
  return renderNodeMarkup(renderSettingsTtsModelPanelNode(state));
}

function renderSettingsTtsVoicePanel(state) {
  return renderNodeMarkup(renderSettingsTtsVoicePanelNode(state));
}

function renderStatusPanel(state) {
  return renderNodeMarkup(renderStatusPanelNode(state));
}

function renderUrlInputPanel(state) {
  return renderNodeMarkup(renderUrlInputPanelNode(state));
}

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

  assert.match(nonRetryableHtml, /Cannot be retried/);
  assert.doesNotMatch(retryableHtml, /Cannot be retried/);
  assert.doesNotMatch(transportHtml, /Cannot be retried/);
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

test("renders a compact talk icon button for the idle state", () => {
  const html = renderPushToTalkPanel({
    enabled: true,
    isHolding: false,
    isListening: false,
    isBusy: false,
    lastTranscript: null,
    lastError: null,
  });

  assert.match(html, /aria-label="Talk"/);
  assert.match(html, /data-push-to-talk-button="true"/);
  assert.doesNotMatch(html, /Voice input/);
  assert.doesNotMatch(html, /Push to talk/);
});

test("disables the talk button during hands-free listening", () => {
  const html = renderPushToTalkPanel({
    enabled: true,
    isHolding: false,
    isListening: true,
    isBusy: false,
    lastTranscript: "start listening",
    lastError: null,
  });

  assert.match(html, /aria-label="Hands-free listening active"/);
  assert.match(html, /disabled aria-disabled="true"/);
});

test("disables the talk button while processing the next spoken command", () => {
  const html = renderPushToTalkPanel({
    enabled: true,
    isHolding: false,
    isListening: true,
    isBusy: true,
    lastTranscript: null,
    lastError: null,
  });

  assert.match(html, /aria-label="Listening busy"/);
  assert.match(html, /disabled aria-disabled="true"/);
});

test("renders the talk button active while holding", () => {
  const html = renderPushToTalkPanel({
    enabled: true,
    isHolding: true,
    isListening: true,
    isBusy: false,
    lastTranscript: "open example dot com",
    lastError: null,
  });

  assert.match(html, /aria-label="Release to transcribe"/);
  assert.match(html, /push-to-talk-button-active/);
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
  assert.match(html, /push-to-talk-error/);
  assert.doesNotMatch(html, /push-to-talk-error[^>]*sr-only/);
});

test("renders setup banner when push-to-talk is disabled, hides it when enabled", () => {
  const disabledHtml = renderPushToTalkPanel({
    enabled: false,
    isHolding: false,
    isListening: false,
    isBusy: false,
    lastTranscript: null,
    lastError: null,
  });
  const enabledHtml = renderPushToTalkPanel({
    enabled: true,
    isHolding: false,
    isListening: false,
    isBusy: false,
    lastTranscript: null,
    lastError: null,
  });

  assert.match(disabledHtml, /ptt-setup-banner/);
  assert.match(disabledHtml, /Voice input isn&#39;t set up yet/);
  assert.match(disabledHtml, /data-ptt-open-settings="true"/);
  assert.doesNotMatch(enabledHtml, /ptt-setup-banner/);
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

  assert.match(html, /data-url-input="true"/);
  assert.match(html, /data-url-open-button="true"/);
  assert.match(html, /data-url-read-button="true"/);
  assert.match(html, /data-url-stop-button="true"/);
  assert.match(html, /data-url-previous-button="true"/);
  assert.match(html, /data-url-next-button="true"/);
  assert.match(html, /aria-label="Open"/);
  assert.match(html, /aria-label="Read"/);
  assert.match(html, /aria-label="Stop"/);
  assert.match(html, /aria-label="Previous"/);
  assert.match(html, /aria-label="Next"/);
  assert.match(html, /value="https:\/\/staged\.example\.com"/);
  assert.doesNotMatch(html, /URL input/);
  assert.doesNotMatch(html, /No page URL is loaded yet\./);
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

  assert.match(html, /placeholder="https:\/\/example\.com"/);
  assert.match(html, /aria-label="Page URL"/);
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

  assert.match(html, /aria-label="Opening"/);
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

  assert.match(html, /aria-label="Reading"/);
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

  assert.match(html, /aria-label="Stopping"/);
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

  assert.match(html, /aria-label="Moving to next section"/);
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

  assert.match(html, /aria-label="Moving to previous section"/);
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
  assert.match(html, /Volume/);
  assert.match(html, /70%/);
  assert.match(html, /Speed/);
  assert.match(html, /1\.25x/);
  assert.match(html, /data-audio-control="volume"/);
  assert.match(html, /data-audio-control="speed"/);
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
    currentRegionLabel: "Section 3",
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

  assert.match(html, /Hold the Talk button/);
  assert.ok(!html.includes("No page open yet"), "first-load state hides status grid");
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

test("voice status strip renders idle state by default", () => {
  const html = renderNodeMarkup(renderVoiceStatusStripNode({ isListening: false, isSpeaking: false, isProcessing: false }));
  assert.match(html, /voice-status-strip/);
  assert.match(html, /data-voice-state="idle"/);
  assert.match(html, /Ready/);
  assert.match(html, /role="status"/);
  assert.match(html, /aria-live="polite"/);
});

test("voice status strip shows Listening state", () => {
  const html = renderNodeMarkup(renderVoiceStatusStripNode({ isListening: true, isSpeaking: false, isProcessing: false }));
  assert.match(html, /data-voice-state="listening"/);
  assert.match(html, /Listening/);
});

test("voice status strip shows Speaking state", () => {
  const html = renderNodeMarkup(renderVoiceStatusStripNode({ isListening: false, isSpeaking: true, isProcessing: false }));
  assert.match(html, /data-voice-state="speaking"/);
  assert.match(html, /Speaking/);
});

test("voice status strip shows Processing state", () => {
  const html = renderNodeMarkup(renderVoiceStatusStripNode({ isListening: false, isSpeaking: false, isProcessing: true }));
  assert.match(html, /data-voice-state="processing"/);
  assert.match(html, /Processing/);
});

test("voice status strip prioritises Listening over Speaking when both active", () => {
  const html = renderNodeMarkup(renderVoiceStatusStripNode({ isListening: true, isSpeaking: true, isProcessing: false }));
  assert.match(html, /data-voice-state="listening"/);
});

