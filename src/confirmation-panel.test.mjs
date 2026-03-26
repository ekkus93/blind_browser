import assert from "node:assert/strict";
import test from "node:test";

import {
  renderAudioControlsPanel,
  renderConfirmationPanel,
  renderPushToTalkPanel,
  renderSettingsAsrProviderPanel,
  renderSettingsConfirmationPanel,
  renderSettingsProviderFailoverPanel,
  renderSettingsPlannerProviderPanel,
  renderSettingsTtsProviderPanel,
  renderSettingsTtsModelPanel,
  renderSettingsTtsVoicePanel,
  renderSettingsSpeedPanel,
  renderSettingsVolumePanel,
  renderStatusPanel,
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

  assert.match(html, /Playback controls/);
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

test("renders settings planner provider as remote-only", () => {
  const html = renderSettingsPlannerProviderPanel({
    activeMode: "Remote",
    availableModes: ["Remote"],
    summary: "Planner currently uses configured remote profiles only.",
  });

  assert.match(html, /Planner provider selection/);
  assert.match(html, /remote profiles only/);
  assert.match(html, /data-planner-provider-select="true"/);
  assert.match(html, /disabled/);
  assert.match(html, /aria-disabled="true"/);
  assert.doesNotMatch(html, /Local provider/);
});

test("renders settings provider failover as read-only unavailable controls", () => {
  const html = renderSettingsProviderFailoverPanel({
    plannerAvailable: false,
    ttsAvailable: false,
    asrAvailable: false,
    summary: "Automatic provider failover is not currently available in the live runtime.",
  });

  assert.match(html, /Provider failover/);
  assert.match(html, /not currently available in the live runtime/);
  assert.match(html, /Planner failover/);
  assert.match(html, /TTS failover/);
  assert.match(html, /ASR failover/);
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

  assert.match(html, /Confirmation behavior/);
  assert.match(html, /Form submission still always requires confirmation/);
  assert.match(html, /Click confirmation threshold/);
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

test("renders settings ASR provider selection for configured modes", () => {
  const html = renderSettingsAsrProviderPanel({
    activeMode: "Local",
    availableModes: ["Local", "Remote"],
    isBusy: false,
    error: null,
  });

  assert.match(html, /ASR provider selection/);
  assert.match(html, /configured local or remote ASR provider/);
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

test("renders settings TTS provider selection for configured modes", () => {
  const html = renderSettingsTtsProviderPanel({
    activeMode: "Local",
    availableModes: ["Local", "Remote"],
    isBusy: false,
    error: null,
  });

  assert.match(html, /TTS provider selection/);
  assert.match(html, /configured local or remote TTS provider/);
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

  assert.match(html, /TTS model selection/);
  assert.match(html, /configured local TTS models/);
  assert.match(html, /default \(kitten-default\)/);
  assert.match(html, /large-v1 \(kitten-large\)/);
  assert.match(html, /data-tts-model-select="true"/);
});

test("renders settings TTS model errors and disabled state while saving", () => {
  const html = renderSettingsTtsModelPanel({
    mode: "Remote",
    activeProfile: "openai-tts-default",
    availableProfiles: [{ profileName: "openai-tts-default", modelLabel: "gpt-4o-mini-tts" }],
    isBusy: true,
    error: "The TTS model selection could not be saved.",
  });

  assert.match(html, /configured remote TTS models/);
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

  assert.match(html, /Voice selection/);
  assert.match(html, /Choose from the configured local TTS voices/);
  assert.match(html, /Configured TTS voice/);
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
  assert.match(html, /Choose from the configured remote TTS voices/);
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
