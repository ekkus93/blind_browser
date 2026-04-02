import assert from "node:assert/strict";
import test from "node:test";

import {
  buildTtsProviderFailureRollbackState,
  isSettingsControlBusy,
  splitRuntimeRefreshResults,
} from "./main-behavior.ts";

function createBusyState(overrides = {}) {
  return {
    audioControls: { isBusy: false },
    confirmationSettings: { isBusy: false },
    ocrThresholdSettings: { isBusy: false },
    asrProvider: { isBusy: false },
    modelManagement: {
      isSaving: false,
      isDownloadingTts: false,
      isDownloadingAsr: false,
    },
    ttsProvider: { isBusy: false },
    ttsModel: { isBusy: false },
    ttsVoice: { isBusy: false },
    ...overrides,
  };
}

test("busy volume saves do not block unrelated controls", () => {
  const busyState = createBusyState({
    audioControls: { isBusy: true },
  });

  assert.equal(isSettingsControlBusy("volume", busyState), true);
  assert.equal(isSettingsControlBusy("speed", busyState), true);
  assert.equal(isSettingsControlBusy("asr-provider", busyState), false);
  assert.equal(isSettingsControlBusy("tts-provider", busyState), false);
  assert.equal(isSettingsControlBusy("models-dir", busyState), false);
});

test("busy TTS controls do not block ASR interactions", () => {
  const busyState = createBusyState({
    ttsProvider: { isBusy: true },
    ttsModel: { isBusy: true },
    ttsVoice: { isBusy: true },
  });

  assert.equal(isSettingsControlBusy("tts-provider", busyState), true);
  assert.equal(isSettingsControlBusy("tts-model", busyState), true);
  assert.equal(isSettingsControlBusy("tts-voice", busyState), true);
  assert.equal(isSettingsControlBusy("asr-provider", busyState), false);
  assert.equal(isSettingsControlBusy("confirmation-threshold", busyState), false);
});

test("same-control duplicate submissions stay blocked by their own busy state", () => {
  const busyState = createBusyState({
    confirmationSettings: { isBusy: true },
  });

  assert.equal(isSettingsControlBusy("confirmation-threshold", busyState), true);
  assert.equal(isSettingsControlBusy("click-without-confirmation", busyState), true);
  assert.equal(isSettingsControlBusy("ocr-char-threshold", busyState), false);
});

test("TTS provider rollback restores prior selections and surfaces the current error everywhere", () => {
  const rollback = buildTtsProviderFailureRollbackState(
    {
      activeMode: "Local",
      availableModes: ["Local", "Remote"],
      isBusy: true,
      error: null,
    },
    {
      mode: "Local",
      activeProfile: "local-default",
      availableProfiles: [{ profileName: "local-default", modelLabel: "Kitten" }],
      isBusy: true,
      error: "stale model error",
    },
    {
      mode: "Local",
      activeVoice: "alloy",
      availableVoices: [{ voiceName: "alloy", displayLabel: "Alloy" }],
      isBusy: true,
      error: "stale voice error",
    },
    "Provider switch failed.",
  );

  assert.deepEqual(rollback.provider, {
    activeMode: "Local",
    availableModes: ["Local", "Remote"],
    isBusy: false,
    error: "Provider switch failed.",
  });
  assert.equal(rollback.model.activeProfile, "local-default");
  assert.equal(rollback.model.isBusy, false);
  assert.equal(rollback.model.error, "Provider switch failed.");
  assert.equal(rollback.voice.activeVoice, "alloy");
  assert.equal(rollback.voice.isBusy, false);
  assert.equal(rollback.voice.error, "Provider switch failed.");
});

test("runtime refresh keeps successful data when only model-management refresh fails", () => {
  const result = splitRuntimeRefreshResults(
    { status: "fulfilled", value: { title: "Example" } },
    { status: "rejected", reason: new Error("models unavailable") },
    (reason) => (reason instanceof Error ? reason.message : String(reason)),
  );

  assert.deepEqual(result.agentState, { title: "Example" });
  assert.equal(result.agentStateError, null);
  assert.equal(result.modelSettings, null);
  assert.equal(result.modelSettingsError, "models unavailable");
});

test("runtime refresh keeps successful model settings when agent-state refresh fails", () => {
  const result = splitRuntimeRefreshResults(
    { status: "rejected", reason: new Error("runtime unavailable") },
    { status: "fulfilled", value: { models_dir: "/tmp/models" } },
    (reason) => (reason instanceof Error ? reason.message : String(reason)),
  );

  assert.equal(result.agentState, null);
  assert.equal(result.agentStateError, "runtime unavailable");
  assert.deepEqual(result.modelSettings, { models_dir: "/tmp/models" });
  assert.equal(result.modelSettingsError, null);
});
