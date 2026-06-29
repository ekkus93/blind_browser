import assert from "node:assert/strict";
import test from "node:test";

const tauriApi = await import("./tauri-api.ts");
const {
  persistRemotePlannerApiKey,
  loadRemotePlannerModels,
  persistRemotePlannerConnection,
  resetRemotePlannerConnectionToDefaults,
  persistRemoteTtsApiKey,
  persistRemoteAsrApiKey,
  testConfiguredRemotePlannerApiKey,
  testConfiguredRemoteTtsApiKey,
  testConfiguredRemoteAsrApiKey,
} = await import("./planner-actions.ts");
const {
  setRemoteAsrPanelState,
  setRemotePlannerPanelState,
  setRemoteTtsPanelState,
} = await import("./panel-state-setters.ts");
const { appShellStore } = await import("./store.ts");

function getPlannerState() {
  return appShellStore.getState().panelStates.remotePlannerPanelState;
}
function getTtsState() {
  return appShellStore.getState().panelStates.remoteTtsPanelState;
}
function getAsrState() {
  return appShellStore.getState().panelStates.remoteAsrPanelState;
}

test.beforeEach(() => {
  tauriApi.__setInvokeForTests(async () => {
    throw new Error("invoke called unexpectedly");
  });
  setRemotePlannerPanelState({
    error: null,
    isSavingApiKey: false,
    isTestingApiKey: false,
    apiKeyTestMessage: null,
  });
  setRemoteTtsPanelState({
    error: null,
    isSavingApiKey: false,
    isTestingApiKey: false,
    apiKeyTestMessage: null,
  });
  setRemoteAsrPanelState({
    error: null,
    isSavingApiKey: false,
    isTestingApiKey: false,
    apiKeyTestMessage: null,
  });
});

test.afterEach(() => {
  tauriApi.__resetInvokeForTests();
});

// ─── persistRemotePlannerApiKey ──────────────────────────────────────────────

test("persistRemotePlannerApiKey sets error when profileName is null", async () => {
  setRemotePlannerPanelState({ profileName: null, apiKeyDraft: "sk-test" });
  await persistRemotePlannerApiKey();
  assert.match(
    getPlannerState().error,
    /No remote planner profile is configured/,
  );
  assert.equal(getPlannerState().isSavingApiKey, false);
});

test("persistRemotePlannerApiKey sets error when apiKeyDraft is blank", async () => {
  setRemotePlannerPanelState({ profileName: "openai-default", apiKeyDraft: "   " });
  await persistRemotePlannerApiKey();
  assert.match(getPlannerState().error, /Enter a remote planner API key/);
  assert.equal(getPlannerState().isSavingApiKey, false);
});

test("persistRemotePlannerApiKey clears draft and updates reference on success", async () => {
  setRemotePlannerPanelState({ profileName: "openai-default", apiKeyDraft: "sk-test" });
  tauriApi.__setInvokeForTests(async () => ({
    profile_name: "openai-default",
    api_key_reference: "keyring:planner",
  }));
  await persistRemotePlannerApiKey();
  const state = getPlannerState();
  assert.equal(state.profileName, "openai-default");
  assert.equal(state.apiKeyReference, "keyring:planner");
  assert.equal(state.apiKeyDraft, "");
  assert.equal(state.error, null);
  assert.equal(state.isSavingApiKey, false);
});

test("persistRemotePlannerApiKey sets error on invoke failure", async () => {
  setRemotePlannerPanelState({ profileName: "openai-default", apiKeyDraft: "sk-test" });
  tauriApi.__setInvokeForTests(async () => {
    throw new Error("Network unreachable");
  });
  await persistRemotePlannerApiKey();
  assert.ok(getPlannerState().error !== null);
  assert.equal(getPlannerState().isSavingApiKey, false);
});

// ─── loadRemotePlannerModels ──────────────────────────────────────────────────

test("loadRemotePlannerModels sets error when profileName is null", async () => {
  setRemotePlannerPanelState({ profileName: null, baseUrl: "https://api.example.com/v1" });
  await loadRemotePlannerModels();
  assert.match(getPlannerState().error, /No remote planner profile/);
});

test("loadRemotePlannerModels sets error when baseUrl is blank", async () => {
  setRemotePlannerPanelState({ profileName: "openai-default", baseUrl: "" });
  await loadRemotePlannerModels();
  assert.match(getPlannerState().error, /Enter an endpoint/);
});

test("loadRemotePlannerModels preserves current model when it is in the returned list", async () => {
  setRemotePlannerPanelState({
    profileName: "openai-default",
    baseUrl: "https://api.example.com/v1",
    model: "gpt-current",
    apiKeyDraft: "",
  });
  tauriApi.__setInvokeForTests(async () => ({
    profile_name: "openai-default",
    base_url: "https://api.example.com/v1",
    models: ["gpt-current", "gpt-other"],
  }));
  await loadRemotePlannerModels();
  const state = getPlannerState();
  assert.equal(state.model, "gpt-current");
  assert.deepEqual(state.availableModels, ["gpt-current", "gpt-other"]);
  assert.equal(state.error, null);
  assert.equal(state.isLoadingModels, false);
});

test("loadRemotePlannerModels selects first model when current model is not in returned list", async () => {
  setRemotePlannerPanelState({
    profileName: "openai-default",
    baseUrl: "https://api.example.com/v1",
    model: "gpt-stale",
    apiKeyDraft: "",
  });
  tauriApi.__setInvokeForTests(async () => ({
    profile_name: "openai-default",
    base_url: "https://api.example.com/v1",
    models: ["gpt-new", "gpt-other"],
  }));
  await loadRemotePlannerModels();
  assert.equal(getPlannerState().model, "gpt-new");
});

test("loadRemotePlannerModels sets error on invoke failure", async () => {
  setRemotePlannerPanelState({
    profileName: "openai-default",
    baseUrl: "https://api.example.com/v1",
    apiKeyDraft: "",
  });
  tauriApi.__setInvokeForTests(async () => {
    throw new Error("timeout");
  });
  await loadRemotePlannerModels();
  assert.ok(getPlannerState().error !== null);
  assert.equal(getPlannerState().isLoadingModels, false);
});

// ─── persistRemotePlannerConnection ──────────────────────────────────────────

test("persistRemotePlannerConnection sets error when profileName is null", async () => {
  setRemotePlannerPanelState({
    profileName: null,
    baseUrl: "https://api.example.com/v1",
    model: "gpt-test",
  });
  await persistRemotePlannerConnection();
  assert.match(getPlannerState().error, /No remote planner profile/);
});

test("persistRemotePlannerConnection sets error when baseUrl is blank", async () => {
  setRemotePlannerPanelState({
    profileName: "openai-default",
    baseUrl: "",
    model: "gpt-test",
  });
  await persistRemotePlannerConnection();
  assert.match(getPlannerState().error, /Enter an endpoint/);
});

test("persistRemotePlannerConnection sets error when model is blank", async () => {
  setRemotePlannerPanelState({
    profileName: "openai-default",
    baseUrl: "https://api.example.com/v1",
    model: "   ",
  });
  await persistRemotePlannerConnection();
  assert.match(getPlannerState().error, /Choose a model/);
});

test("persistRemotePlannerConnection clears model list when endpoint changed", async () => {
  setRemotePlannerPanelState({
    profileName: "openai-default",
    baseUrl: "https://new-endpoint.example.com/v1",
    model: "gpt-test",
    availableModels: ["gpt-old"],
    loadedModelsEndpoint: "https://old-endpoint.example.com/v1",
    isSavingConnection: false,
  });
  tauriApi.__setInvokeForTests(async () => ({
    profile_name: "openai-default",
    base_url: "https://new-endpoint.example.com/v1",
    model: "gpt-test",
  }));
  await persistRemotePlannerConnection();
  const state = getPlannerState();
  assert.deepEqual(state.availableModels, []);
  assert.equal(state.loadedModelsEndpoint, null);
  assert.equal(state.error, null);
  assert.equal(state.isSavingConnection, false);
});

// ─── resetRemotePlannerConnectionToDefaults ───────────────────────────────────

test("resetRemotePlannerConnectionToDefaults sets error when profileName is null", async () => {
  setRemotePlannerPanelState({ profileName: null });
  await resetRemotePlannerConnectionToDefaults();
  assert.match(getPlannerState().error, /No remote planner profile/);
});

// ─── testConfiguredRemotePlannerApiKey ────────────────────────────────────────

test("testConfiguredRemotePlannerApiKey sets error when profileName is null", async () => {
  setRemotePlannerPanelState({ profileName: null, apiKeyDraft: "", apiKeyReference: null });
  await testConfiguredRemotePlannerApiKey();
  assert.match(getPlannerState().error, /No remote planner profile/);
});

test("testConfiguredRemotePlannerApiKey sets error when no draft or saved key", async () => {
  setRemotePlannerPanelState({
    profileName: "openai-default",
    apiKeyDraft: "",
    apiKeyReference: null,
  });
  await testConfiguredRemotePlannerApiKey();
  assert.match(getPlannerState().error, /Enter a remote planner API key/);
});

test("testConfiguredRemotePlannerApiKey accepts saved reference without draft", async () => {
  setRemotePlannerPanelState({
    profileName: "openai-default",
    apiKeyDraft: "",
    apiKeyReference: "keyring:planner",
  });
  tauriApi.__setInvokeForTests(async () => ({
    profile_name: "openai-default",
    message: "API key is valid.",
  }));
  await testConfiguredRemotePlannerApiKey();
  const state = getPlannerState();
  assert.equal(state.apiKeyTestMessage, "API key is valid.");
  assert.equal(state.error, null);
  assert.equal(state.isTestingApiKey, false);
});

test("testConfiguredRemotePlannerApiKey sets error on invoke failure", async () => {
  setRemotePlannerPanelState({
    profileName: "openai-default",
    apiKeyDraft: "sk-test",
    apiKeyReference: null,
  });
  tauriApi.__setInvokeForTests(async () => {
    throw new Error("invalid API key");
  });
  await testConfiguredRemotePlannerApiKey();
  assert.ok(getPlannerState().error !== null);
  assert.equal(getPlannerState().isTestingApiKey, false);
});

// ─── persistRemoteTtsApiKey ───────────────────────────────────────────────────

test("persistRemoteTtsApiKey sets error when profileName is null", async () => {
  setRemoteTtsPanelState({ profileName: null, apiKeyDraft: "sk-test" });
  await persistRemoteTtsApiKey();
  assert.match(getTtsState().error, /No remote TTS profile/);
});

test("persistRemoteTtsApiKey sets error when apiKeyDraft is blank", async () => {
  setRemoteTtsPanelState({ profileName: "openai-tts-default", apiKeyDraft: "   " });
  await persistRemoteTtsApiKey();
  assert.match(getTtsState().error, /Enter a remote TTS API key/);
});

test("persistRemoteTtsApiKey clears draft on success", async () => {
  setRemoteTtsPanelState({ profileName: "openai-tts-default", apiKeyDraft: "sk-tts" });
  tauriApi.__setInvokeForTests(async () => ({
    profile_name: "openai-tts-default",
    api_key_reference: "keyring:tts",
  }));
  await persistRemoteTtsApiKey();
  const state = getTtsState();
  assert.equal(state.apiKeyDraft, "");
  assert.equal(state.apiKeyReference, "keyring:tts");
  assert.equal(state.error, null);
});

// ─── persistRemoteAsrApiKey ───────────────────────────────────────────────────

test("persistRemoteAsrApiKey sets error when profileName is null", async () => {
  setRemoteAsrPanelState({ profileName: null, apiKeyDraft: "sk-test" });
  await persistRemoteAsrApiKey();
  assert.match(getAsrState().error, /No remote ASR profile/);
});

test("persistRemoteAsrApiKey sets error when apiKeyDraft is blank", async () => {
  setRemoteAsrPanelState({ profileName: "openai-transcribe-default", apiKeyDraft: "  " });
  await persistRemoteAsrApiKey();
  assert.match(getAsrState().error, /Enter a remote ASR API key/);
});

// ─── testConfiguredRemoteTtsApiKey ───────────────────────────────────────────

test("testConfiguredRemoteTtsApiKey sets error when profileName is null", async () => {
  setRemoteTtsPanelState({ profileName: null, apiKeyDraft: "", apiKeyReference: null });
  await testConfiguredRemoteTtsApiKey();
  assert.match(getTtsState().error, /No remote TTS profile/);
});

test("testConfiguredRemoteTtsApiKey sets message on success", async () => {
  setRemoteTtsPanelState({
    profileName: "openai-tts-default",
    apiKeyDraft: "sk-tts",
    apiKeyReference: null,
  });
  tauriApi.__setInvokeForTests(async () => ({
    profile_name: "openai-tts-default",
    message: "TTS key accepted.",
  }));
  await testConfiguredRemoteTtsApiKey();
  const state = getTtsState();
  assert.equal(state.apiKeyTestMessage, "TTS key accepted.");
  assert.equal(state.error, null);
  assert.equal(state.isTestingApiKey, false);
});

// ─── testConfiguredRemoteAsrApiKey ───────────────────────────────────────────

test("testConfiguredRemoteAsrApiKey sets error when profileName is null", async () => {
  setRemoteAsrPanelState({ profileName: null, apiKeyDraft: "", apiKeyReference: null });
  await testConfiguredRemoteAsrApiKey();
  assert.match(getAsrState().error, /No remote ASR profile/);
});

test("testConfiguredRemoteAsrApiKey sets message on success", async () => {
  setRemoteAsrPanelState({
    profileName: "openai-transcribe-default",
    apiKeyDraft: "sk-asr",
    apiKeyReference: null,
  });
  tauriApi.__setInvokeForTests(async () => ({
    profile_name: "openai-transcribe-default",
    message: "ASR key accepted.",
  }));
  await testConfiguredRemoteAsrApiKey();
  const state = getAsrState();
  assert.equal(state.apiKeyTestMessage, "ASR key accepted.");
  assert.equal(state.error, null);
  assert.equal(state.isTestingApiKey, false);
});
