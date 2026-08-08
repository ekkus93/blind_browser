import assert from "node:assert/strict";
import test from "node:test";

const tauriApi = await import("./tauri-api.ts");
const {
  persistRemoteAsrPrivacyNetworkMode,
  persistRemoteTtsPrivacyNetworkMode,
} = await import("./settings-actions.ts");
const {
  setRemoteAsrPanelState,
  setRemoteTtsPanelState,
} = await import("./panel-state-setters.ts");
const { appShellStore } = await import("./store.ts");
const {
  renderSettingsRemoteAsrPanel,
  renderSettingsRemoteTtsPanel,
} = await import("./confirmation-panel-test-helpers.mjs");

function remoteTtsState() {
  return appShellStore.getState().panelStates.remoteTtsPanelState;
}

function remoteAsrState() {
  return appShellStore.getState().panelStates.remoteAsrPanelState;
}

test.beforeEach(() => {
  setRemoteTtsPanelState({
    privacyNetworkMode: "ask_per_origin",
    privacyOriginRuleCount: 2,
    privacyNotice: "Narration policy is independent.",
    isSavingPrivacy: false,
    error: null,
  });
  setRemoteAsrPanelState({
    privacyNetworkMode: "ask_per_origin",
    privacyOriginRuleCount: 3,
    privacyNotice: "Microphone policy is independent.",
    isSavingPrivacy: false,
    error: null,
  });
  tauriApi.__setInvokeForTests(async () => {
    throw new Error("invoke called unexpectedly");
  });
});

test.afterEach(() => {
  tauriApi.__resetInvokeForTests();
});

test("remote narration privacy mode uses the typed narration operation", async () => {
  let captured = null;
  tauriApi.__setInvokeForTests(async (command, args) => {
    captured = { command, args };
    return {
      purpose: "narration",
      network_mode: "local_only",
      origin_rule_count: 2,
      changed: true,
    };
  });

  await persistRemoteTtsPrivacyNetworkMode("local_only");

  assert.equal(captured.command, "set_remote_speech_privacy_network_mode");
  assert.equal(captured.args.purpose, "narration");
  assert.equal(captured.args.networkMode, "local_only");
  assert.equal(remoteTtsState().privacyNetworkMode, "local_only");
  assert.equal(remoteTtsState().isSavingPrivacy, false);
  assert.equal(remoteTtsState().error, null);
});

test("remote microphone privacy mode uses the typed microphone operation", async () => {
  let captured = null;
  tauriApi.__setInvokeForTests(async (command, args) => {
    captured = { command, args };
    return {
      purpose: "microphone",
      network_mode: "local_only",
      origin_rule_count: 3,
      changed: true,
    };
  });

  await persistRemoteAsrPrivacyNetworkMode("local_only");

  assert.equal(captured.command, "set_remote_speech_privacy_network_mode");
  assert.equal(captured.args.purpose, "microphone");
  assert.equal(captured.args.networkMode, "local_only");
  assert.equal(remoteAsrState().privacyNetworkMode, "local_only");
  assert.equal(remoteAsrState().isSavingPrivacy, false);
  assert.equal(remoteAsrState().error, null);
});

test("speech privacy save failure rolls back instead of pretending success", async () => {
  tauriApi.__setInvokeForTests(async () => {
    throw new Error("privacy persistence unavailable");
  });

  await persistRemoteAsrPrivacyNetworkMode("local_only");
  await persistRemoteTtsPrivacyNetworkMode("local_only");

  assert.equal(remoteAsrState().privacyNetworkMode, "ask_per_origin");
  assert.equal(remoteTtsState().privacyNetworkMode, "ask_per_origin");
  assert.match(remoteAsrState().error ?? "", /privacy persistence unavailable/);
  assert.match(remoteTtsState().error ?? "", /privacy persistence unavailable/);
});

test("remote speech settings expose independent editable privacy controls", () => {
  const common = {
    profileName: "remote-profile",
    provider: "OpenAI",
    baseUrl: "https://example.invalid/v1",
    model: "model",
    apiKeyReference: null,
    apiKeyMaskedValue: null,
    apiKeyReferenceError: null,
    organizationReference: null,
    project: null,
    timeoutMs: 30_000,
    privacyNetworkMode: "ask_per_origin",
    privacyOriginRuleCount: 1,
    isSavingPrivacy: false,
    apiKeyDraft: "",
    isSavingApiKey: false,
    isTestingApiKey: false,
    apiKeyTestMessage: null,
    error: null,
  };
  const ttsHtml = renderSettingsRemoteTtsPanel({
    ...common,
    voice: "alloy",
    audioFormat: "wav",
    privacyNotice: "Narration privacy policy.",
  });
  const asrHtml = renderSettingsRemoteAsrPanel({
    ...common,
    language: "en",
    temperatureMilli: 0,
    privacyNotice: "Microphone privacy policy.",
  });

  assert.match(ttsHtml, /data-remote-tts-privacy-select="true"/);
  assert.match(ttsHtml, /Narration privacy policy\./);
  assert.match(asrHtml, /data-remote-asr-privacy-select="true"/);
  assert.match(asrHtml, /Microphone privacy policy\./);
  assert.match(ttsHtml, /Local only — block network speech/);
  assert.match(asrHtml, /Ask before network speech/);
});
