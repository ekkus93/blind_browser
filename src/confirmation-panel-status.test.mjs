import assert from "node:assert/strict";
import test from "node:test";

import {
  renderNodeMarkup,
  renderStatusPanel,
  renderVoiceStatusStripNode,
  statusPanelStateFromAgentState,
} from "./confirmation-panel-test-helpers.mjs";

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

  assert.match(html, /disabled="" aria-disabled="true"/);
  assert.match(html, /data-browser-visibility-mode="Visible"[^>]*aria-pressed="true"/);
});

test("voice status strip renders idle state by default", () => {
  const html = renderNodeMarkup(renderVoiceStatusStripNode({ isListening: false, isSpeaking: false, isProcessing: false }));
  assert.match(html, /data-voice-status-strip="true"/);
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
