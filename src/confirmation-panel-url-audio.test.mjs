import assert from "node:assert/strict";
import test from "node:test";

import {
  renderAudioControlsPanel,
  renderSettingsConfirmationPanel,
  renderUrlInputPanel,
} from "./confirmation-panel-test-helpers.mjs";

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
  assert.match(html, /disabled="" aria-disabled="true"/);
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
  assert.match(html, /disabled="" aria-disabled="true"/);
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
  assert.match(html, /disabled="" aria-disabled="true"/);
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
  assert.match(html, /disabled="" aria-disabled="true"/);
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
  assert.match(html, /disabled="" aria-disabled="true"/);
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

test("disables nearby playback controls while audio settings are saving", () => {
  const html = renderAudioControlsPanel({
    playbackVolume: 1,
    playbackSpeed: 1,
    isBusy: true,
    error: null,
  });

  assert.match(html, /disabled="" aria-disabled="true"/);
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
