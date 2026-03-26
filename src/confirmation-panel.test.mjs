import assert from "node:assert/strict";
import test from "node:test";

import {
  renderAudioControlsPanel,
  renderConfirmationPanel,
  renderPushToTalkPanel,
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

  assert.match(html, /Hold Space or press and hold the button to speak a command\./);
  assert.match(html, /Hold to talk/);
  assert.match(html, /data-push-to-talk-button="true"/);
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
    isBusy: false,
    error: null,
  });

  assert.match(html, /URL input/);
  assert.match(html, /Current URL:<\/strong> https:\/\/current\.example\.com/);
  assert.match(html, /Draft URL updated\. Open controls can use this value next\./);
  assert.match(html, /data-url-input="true"/);
  assert.match(html, /data-url-open-button="true"/);
  assert.match(html, />\s*Open\s*<\/button>/);
  assert.match(html, /value="https:\/\/staged\.example\.com"/);
});

test("renders URL input fallback copy when no page is loaded", () => {
  const html = renderUrlInputPanel({
    draftValue: "",
    currentUrl: null,
    hasUnsubmittedChanges: false,
    isBusy: false,
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
    isBusy: true,
    error: "The browser could not open that URL.",
  });

  assert.match(html, /Opening\.\.\./);
  assert.match(html, /The browser could not open that URL\./);
  assert.match(html, /disabled aria-disabled="true"/);
  assert.match(html, /role="alert"/);
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
