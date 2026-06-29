import assert from "node:assert/strict";
import test from "node:test";

const tauriApi = await import("./tauri-api.ts");
const {
  persistBrowserVisibility,
  persistPlaybackVolume,
  persistPlaybackSpeed,
  persistAsrProviderSelection,
  persistTtsProviderSelection,
  persistTtsModelSelection,
  persistTtsVoiceSelection,
  persistConfirmationThreshold,
  persistAllowClickWithoutConfirmation,
  persistOcrThresholds,
  persistModelManagementSettings,
} = await import("./settings-actions.ts");
const {
  setAudioControlsState,
  setAsrProviderPanelState,
  setConfirmationSettingsPanelState,
  setModelManagementPanelState,
  setOcrThresholdSettingsPanelState,
  setStatusPanelState,
  setTtsModelPanelState,
  setTtsProviderPanelState,
  setTtsVoicePanelState,
} = await import("./panel-state-setters.ts");
const { appShellStore } = await import("./store.ts");

function getAudioState() {
  return appShellStore.getState().panelStates.audioControlsState;
}
function getAsrState() {
  return appShellStore.getState().panelStates.asrProviderPanelState;
}
function getConfirmState() {
  return appShellStore.getState().panelStates.confirmationSettingsPanelState;
}
function getModelState() {
  return appShellStore.getState().panelStates.modelManagementPanelState;
}
function getOcrState() {
  return appShellStore.getState().panelStates.ocrThresholdSettingsPanelState;
}
function getStatusState() {
  return appShellStore.getState().panelStates.statusPanelState;
}
function getTtsModelState() {
  return appShellStore.getState().panelStates.ttsModelPanelState;
}
function getTtsProviderState() {
  return appShellStore.getState().panelStates.ttsProviderPanelState;
}
function getTtsVoiceState() {
  return appShellStore.getState().panelStates.ttsVoicePanelState;
}

test.beforeEach(() => {
  tauriApi.__setInvokeForTests(async () => {
    throw new Error("invoke called unexpectedly");
  });
});

test.afterEach(() => {
  tauriApi.__resetInvokeForTests();
});

// ─── persistBrowserVisibility ─────────────────────────────────────────────────

test("persistBrowserVisibility updates store on success", async () => {
  setStatusPanelState({ browserVisibility: "Visible", isUpdatingVisibility: false, error: null });
  tauriApi.__setInvokeForTests(async () => ({
    ok: true,
    tool_name: "SetBrowserVisibility",
    request_id: "req-vis",
    timestamp_ms: 0,
    data: { mode: "Headless", changed: true, supported: true },
    error: null,
    warnings: [],
    observations: [],
  }));
  await persistBrowserVisibility("Headless");
  assert.equal(getStatusState().browserVisibility, "Headless");
  assert.equal(getStatusState().isUpdatingVisibility, false);
  assert.equal(getStatusState().error, null);
});

test("persistBrowserVisibility rolls back on failure", async () => {
  setStatusPanelState({ browserVisibility: "Visible", isUpdatingVisibility: false, error: null });
  tauriApi.__setInvokeForTests(async () => {
    throw new Error("not supported");
  });
  await persistBrowserVisibility("Headless");
  assert.equal(getStatusState().browserVisibility, "Visible");
  assert.equal(getStatusState().isUpdatingVisibility, false);
  assert.ok(getStatusState().error !== null);
});

// ─── persistPlaybackVolume ────────────────────────────────────────────────────

test("persistPlaybackVolume is skipped when audio controls are busy", async () => {
  setAudioControlsState({ playbackVolume: 0.5, isBusy: true, error: null });
  let invokeCalled = false;
  tauriApi.__setInvokeForTests(async () => { invokeCalled = true; return {}; });
  await persistPlaybackVolume(0.8);
  assert.equal(invokeCalled, false);
  assert.equal(getAudioState().playbackVolume, 0.5);
});

test("persistPlaybackVolume updates volume on success", async () => {
  setAudioControlsState({ playbackVolume: 0.5, isBusy: false, error: null });
  tauriApi.__setInvokeForTests(async () => ({
    ok: true,
    tool_name: "SetPlaybackVolume",
    request_id: "req-vol",
    timestamp_ms: 0,
    data: { playback_volume: 0.8, muted: false, changed: true },
    error: null,
    warnings: [],
    observations: [],
  }));
  await persistPlaybackVolume(0.8);
  assert.equal(getAudioState().playbackVolume, 0.8);
  assert.equal(getAudioState().isBusy, false);
  assert.equal(getAudioState().error, null);
});

test("persistPlaybackVolume rolls back on failure", async () => {
  setAudioControlsState({ playbackVolume: 0.5, isBusy: false, error: null });
  tauriApi.__setInvokeForTests(async () => {
    throw new Error("backend unavailable");
  });
  await persistPlaybackVolume(0.8);
  assert.equal(getAudioState().playbackVolume, 0.5);
  assert.equal(getAudioState().isBusy, false);
  assert.ok(getAudioState().error !== null);
});

// ─── persistPlaybackSpeed ─────────────────────────────────────────────────────

test("persistPlaybackSpeed updates speed on success", async () => {
  setAudioControlsState({ playbackSpeed: 1.0, isBusy: false, error: null });
  tauriApi.__setInvokeForTests(async () => ({
    ok: true,
    tool_name: "SetPlaybackSpeed",
    request_id: "req-speed",
    timestamp_ms: 0,
    data: { playback_speed: 1.5, changed: true },
    error: null,
    warnings: [],
    observations: [],
  }));
  await persistPlaybackSpeed(1.5);
  assert.equal(getAudioState().playbackSpeed, 1.5);
  assert.equal(getAudioState().isBusy, false);
});

test("persistPlaybackSpeed rolls back on failure", async () => {
  setAudioControlsState({ playbackSpeed: 1.0, isBusy: false, error: null });
  tauriApi.__setInvokeForTests(async () => {
    throw new Error("TTS unavailable");
  });
  await persistPlaybackSpeed(1.5);
  assert.equal(getAudioState().playbackSpeed, 1.0);
  assert.ok(getAudioState().error !== null);
});

// ─── persistAsrProviderSelection ─────────────────────────────────────────────

test("persistAsrProviderSelection is skipped when asr provider is busy", async () => {
  setAsrProviderPanelState({ activeMode: "Local", isBusy: true, error: null });
  let invokeCalled = false;
  tauriApi.__setInvokeForTests(async () => { invokeCalled = true; return {}; });
  await persistAsrProviderSelection("Remote");
  assert.equal(invokeCalled, false);
  assert.equal(getAsrState().activeMode, "Local");
});

test("persistAsrProviderSelection updates mode on success", async () => {
  setAsrProviderPanelState({ activeMode: "Local", isBusy: false, error: null });
  tauriApi.__setInvokeForTests(async () => ({ mode: "Remote", changed: true }));
  await persistAsrProviderSelection("Remote");
  assert.equal(getAsrState().activeMode, "Remote");
  assert.equal(getAsrState().isBusy, false);
  assert.equal(getAsrState().error, null);
});

test("persistAsrProviderSelection rolls back on failure", async () => {
  setAsrProviderPanelState({ activeMode: "Local", isBusy: false, error: null });
  tauriApi.__setInvokeForTests(async () => {
    throw new Error("ASR unavailable");
  });
  await persistAsrProviderSelection("Remote");
  assert.equal(getAsrState().activeMode, "Local");
  assert.ok(getAsrState().error !== null);
});

// ─── persistTtsProviderSelection ─────────────────────────────────────────────

test("persistTtsProviderSelection is skipped when tts provider is busy", async () => {
  setTtsProviderPanelState({ activeMode: "Local", isBusy: true, error: null });
  let invokeCalled = false;
  tauriApi.__setInvokeForTests(async () => { invokeCalled = true; return {}; });
  await persistTtsProviderSelection("Remote");
  assert.equal(invokeCalled, false);
  assert.equal(getTtsProviderState().activeMode, "Local");
});

test("persistTtsProviderSelection updates mode on success", async () => {
  setTtsProviderPanelState({ activeMode: "Local", isBusy: false, error: null });
  setTtsModelPanelState({ isBusy: false, error: null });
  setTtsVoicePanelState({ isBusy: false, error: null });
  tauriApi.__setInvokeForTests(async () => ({ mode: "Remote", changed: true }));
  await persistTtsProviderSelection("Remote");
  assert.equal(getTtsProviderState().activeMode, "Remote");
  assert.equal(getTtsProviderState().isBusy, false);
  assert.equal(getTtsProviderState().error, null);
});

test("persistTtsProviderSelection rolls back all three panels on failure", async () => {
  setTtsProviderPanelState({ activeMode: "Local", isBusy: false, error: null });
  setTtsModelPanelState({ isBusy: false, error: null });
  setTtsVoicePanelState({ isBusy: false, error: null });
  tauriApi.__setInvokeForTests(async () => {
    throw new Error("TTS provider unreachable");
  });
  await persistTtsProviderSelection("Remote");
  assert.equal(getTtsProviderState().activeMode, "Local");
  assert.ok(getTtsProviderState().error !== null);
  assert.ok(getTtsModelState().error !== null);
  assert.ok(getTtsVoiceState().error !== null);
});

// ─── persistTtsModelSelection ─────────────────────────────────────────────────

test("persistTtsModelSelection updates activeProfile on success", async () => {
  setTtsModelPanelState({ activeProfile: "kitten-default", isBusy: false, error: null });
  tauriApi.__setInvokeForTests(async () => ({
    profile_name: "kitten-other",
    changed: true,
  }));
  await persistTtsModelSelection("kitten-other");
  assert.equal(getTtsModelState().activeProfile, "kitten-other");
  assert.equal(getTtsModelState().isBusy, false);
});

test("persistTtsModelSelection rolls back on failure", async () => {
  setTtsModelPanelState({ activeProfile: "kitten-default", isBusy: false, error: null });
  tauriApi.__setInvokeForTests(async () => {
    throw new Error("model unavailable");
  });
  await persistTtsModelSelection("kitten-other");
  assert.equal(getTtsModelState().activeProfile, "kitten-default");
  assert.ok(getTtsModelState().error !== null);
});

// ─── persistTtsVoiceSelection ─────────────────────────────────────────────────

test("persistTtsVoiceSelection updates activeVoice on success", async () => {
  setTtsVoicePanelState({ activeVoice: "Bruno", isBusy: false, error: null });
  setTtsProviderPanelState({ isBusy: false, error: null });
  tauriApi.__setInvokeForTests(async () => ({
    ok: true,
    tool_name: "SetTtsVoice",
    request_id: "req-voice",
    timestamp_ms: 0,
    data: { voice: "Bella", changed: true },
    error: null,
    warnings: [],
    observations: [],
  }));
  await persistTtsVoiceSelection("Bella");
  assert.equal(getTtsVoiceState().activeVoice, "Bella");
  assert.equal(getTtsVoiceState().isBusy, false);
  assert.equal(getTtsVoiceState().error, null);
});

test("persistTtsVoiceSelection rolls back on failure", async () => {
  setTtsVoicePanelState({ activeVoice: "Bruno", isBusy: false, error: null });
  setTtsProviderPanelState({ isBusy: false, error: null });
  tauriApi.__setInvokeForTests(async () => {
    throw new Error("voice unavailable");
  });
  await persistTtsVoiceSelection("Bella");
  assert.equal(getTtsVoiceState().activeVoice, "Bruno");
  assert.ok(getTtsVoiceState().error !== null);
});

// ─── persistConfirmationThreshold ────────────────────────────────────────────

test("persistConfirmationThreshold is skipped when confirmation settings are busy", async () => {
  setConfirmationSettingsPanelState({ confirmationConfidenceThreshold: 0.8, isBusy: true, error: null });
  let invokeCalled = false;
  tauriApi.__setInvokeForTests(async () => { invokeCalled = true; return {}; });
  await persistConfirmationThreshold(0.9);
  assert.equal(invokeCalled, false);
  assert.equal(getConfirmState().confirmationConfidenceThreshold, 0.8);
});

test("persistConfirmationThreshold updates threshold on success", async () => {
  setConfirmationSettingsPanelState({ confirmationConfidenceThreshold: 0.8, isBusy: false, error: null });
  tauriApi.__setInvokeForTests(async () => ({
    confirmation_confidence_threshold: 0.9,
    changed: true,
  }));
  await persistConfirmationThreshold(0.9);
  assert.equal(getConfirmState().confirmationConfidenceThreshold, 0.9);
  assert.equal(getConfirmState().isBusy, false);
  assert.equal(getConfirmState().error, null);
});

test("persistConfirmationThreshold rolls back on failure", async () => {
  setConfirmationSettingsPanelState({ confirmationConfidenceThreshold: 0.8, isBusy: false, error: null });
  tauriApi.__setInvokeForTests(async () => {
    throw new Error("save failed");
  });
  await persistConfirmationThreshold(0.9);
  assert.equal(getConfirmState().confirmationConfidenceThreshold, 0.8);
  assert.ok(getConfirmState().error !== null);
});

// ─── persistAllowClickWithoutConfirmation ─────────────────────────────────────

test("persistAllowClickWithoutConfirmation updates setting on success", async () => {
  setConfirmationSettingsPanelState({ allowClickWithoutConfirmation: true, isBusy: false, error: null });
  tauriApi.__setInvokeForTests(async () => ({
    allow_click_without_confirmation: false,
    changed: true,
  }));
  await persistAllowClickWithoutConfirmation(false);
  assert.equal(getConfirmState().allowClickWithoutConfirmation, false);
  assert.equal(getConfirmState().isBusy, false);
});

// ─── persistOcrThresholds ────────────────────────────────────────────────────

test("persistOcrThresholds is skipped when ocr settings are busy", async () => {
  setOcrThresholdSettingsPanelState({ sparseTextCharThreshold: 200, sparseTextRegionThreshold: 2, isBusy: true, error: null });
  let invokeCalled = false;
  tauriApi.__setInvokeForTests(async () => { invokeCalled = true; return {}; });
  await persistOcrThresholds(150, 3);
  assert.equal(invokeCalled, false);
  assert.equal(getOcrState().sparseTextCharThreshold, 200);
});

test("persistOcrThresholds updates thresholds on success", async () => {
  setOcrThresholdSettingsPanelState({ sparseTextCharThreshold: 200, sparseTextRegionThreshold: 2, isBusy: false, error: null });
  tauriApi.__setInvokeForTests(async () => ({
    sparse_text_char_threshold: 150,
    sparse_text_region_threshold: 3,
    changed: true,
  }));
  await persistOcrThresholds(150, 3);
  assert.equal(getOcrState().sparseTextCharThreshold, 150);
  assert.equal(getOcrState().sparseTextRegionThreshold, 3);
  assert.equal(getOcrState().isBusy, false);
  assert.equal(getOcrState().error, null);
});

test("persistOcrThresholds rolls back on failure", async () => {
  setOcrThresholdSettingsPanelState({ sparseTextCharThreshold: 200, sparseTextRegionThreshold: 2, isBusy: false, error: null });
  tauriApi.__setInvokeForTests(async () => {
    throw new Error("OCR settings unavailable");
  });
  await persistOcrThresholds(150, 3);
  assert.equal(getOcrState().sparseTextCharThreshold, 200);
  assert.equal(getOcrState().sparseTextRegionThreshold, 2);
  assert.ok(getOcrState().error !== null);
});

// ─── persistModelManagementSettings ──────────────────────────────────────────

test("persistModelManagementSettings sets error when modelsDir is blank", async () => {
  setModelManagementPanelState({ modelsDir: "   ", isSaving: false, error: null });
  let invokeCalled = false;
  tauriApi.__setInvokeForTests(async () => { invokeCalled = true; return {}; });
  await persistModelManagementSettings();
  assert.equal(invokeCalled, false);
  assert.match(getModelState().error, /Enter a models directory/);
  assert.equal(getModelState().isSaving, false);
});

test("persistModelManagementSettings is skipped when saving is in progress", async () => {
  setModelManagementPanelState({ modelsDir: "/data/models", isSaving: true, error: null });
  let invokeCalled = false;
  tauriApi.__setInvokeForTests(async () => { invokeCalled = true; return {}; });
  await persistModelManagementSettings();
  assert.equal(invokeCalled, false);
  assert.equal(getModelState().modelsDir, "/data/models");
});

test("persistModelManagementSettings updates settings on success", async () => {
  setModelManagementPanelState({
    modelsDir: "/data/models",
    checkOnStartup: true,
    autoDownloadMissing: false,
    isSaving: false,
    error: null,
  });
  tauriApi.__setInvokeForTests(async () => ({
    models_dir: "/data/models",
    check_on_startup: true,
    auto_download_missing: false,
  }));
  await persistModelManagementSettings();
  assert.equal(getModelState().modelsDir, "/data/models");
  assert.equal(getModelState().isSaving, false);
  assert.equal(getModelState().error, null);
});

test("persistModelManagementSettings rolls back on failure", async () => {
  setModelManagementPanelState({
    modelsDir: "/data/models",
    isSaving: false,
    error: null,
  });
  tauriApi.__setInvokeForTests(async () => {
    throw new Error("write error");
  });
  await persistModelManagementSettings();
  assert.ok(getModelState().error !== null);
  assert.equal(getModelState().isSaving, false);
});
