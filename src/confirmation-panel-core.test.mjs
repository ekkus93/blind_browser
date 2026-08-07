import assert from "node:assert/strict";
import test from "node:test";

import {
  renderFixtures,
  renderPushToTalkPanel,
} from "./confirmation-panel-test-helpers.mjs";

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

  assert.match(nonRetryableHtml, /data-confirmation-error-meta-block="true"/);
  assert.match(
    nonRetryableHtml,
    /<p[^>]*data-confirmation-error-meta="true"[^>]*>\s*Error code: confirmation_denied\. Non-retryable backend failure\.\s*<\/p>/,
  );
  assert.match(
    nonRetryableHtml,
    /<p[^>]*data-confirmation-error-retry-status="true"[^>]*>Cannot retry\.<\/p>/,
  );

  assert.match(retryableHtml, /data-confirmation-error-meta-block="true"/);
  assert.match(
    retryableHtml,
    /<p[^>]*data-confirmation-error-meta="true"[^>]*>\s*Error code: runtime_busy\. Retryable backend failure\.\s*<\/p>/,
  );
  assert.match(
    retryableHtml,
    /<p[^>]*data-confirmation-error-retry-status="true"[^>]*>Can retry\.<\/p>/,
  );

  assert.doesNotMatch(transportHtml, /data-confirmation-error-meta-block="true"/);
});

test("confirmation error container always present in DOM with aria-live assertive", () => {
  const { nonRetryableHtml } = renderFixtures();

  assert.match(nonRetryableHtml, /aria-live="assertive"/);
  assert.match(nonRetryableHtml, /aria-atomic="true"/);
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

  assert.match(html, /aria-label="Hold to talk"/);
  assert.match(html, /data-push-to-talk-button="true"/);
  assert.match(html, /Say a URL or command/);
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

  assert.match(html, /aria-label="Voice input active"/);
  assert.match(html, /disabled="" aria-disabled="true"/);
  assert.match(html, /stop listening.*to end/);
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

  assert.match(html, /aria-label="Voice input active"/);
  assert.match(html, /disabled="" aria-disabled="true"/);
  assert.match(html, /stop listening.*to end/);
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

  assert.match(html, /aria-label="Release to send"/);
  assert.match(html, /data-ptt-holding="true"/);
  assert.match(html, /Listening…/);
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
  assert.match(html, /data-ptt-error="true"/);
  assert.doesNotMatch(html, /data-ptt-error="true"[^>]*sr-only/);
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

  assert.match(disabledHtml, /data-ptt-setup-banner="true"/);
  assert.match(disabledHtml, /Voice input isn&#x27;t set up yet/);
  assert.match(disabledHtml, /data-ptt-open-settings="true"/);
  assert.doesNotMatch(disabledHtml, /data-push-to-talk-button="true"/);
  assert.doesNotMatch(enabledHtml, /data-ptt-setup-banner="true"/);
  assert.match(enabledHtml, /data-push-to-talk-button="true"/);
});
