import assert from "node:assert/strict";
import test from "node:test";

import {
  renderSettingsAsrProviderPanel,
  renderSettingsLocalAsrModelPanel,
  renderSettingsLocalTtsModelPanel,
  renderSettingsRemoteAsrPanel,
  renderSettingsRemoteTtsPanel,
  renderSettingsTtsModelPanel,
  renderSettingsTtsProviderPanel,
  renderSettingsTtsVoicePanel,
} from "./confirmation-panel-test-helpers.mjs";

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
  assert.match(html, /disabled="" aria-disabled="true"/);
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
  assert.match(html, /disabled="" aria-disabled="true"/);
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
  assert.match(html, /disabled="" aria-disabled="true"/);
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
  assert.match(html, /<option value="Bruno" selected="">Bruno<\/option>/);
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
  assert.match(html, /disabled="" aria-disabled="true"/);
});

test("renders api key reference error warning in remote ASR panel", () => {
  const html = renderSettingsRemoteAsrPanel({
    profileName: "openai-transcribe-default",
    provider: "OpenAI",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-4o-mini-transcribe",
    apiKeyReference: "OS keyring entry: blind_browser / remote_asr:openai-transcribe-default:api_key",
    apiKeyMaskedValue: null,
    apiKeyReferenceError: "Configured secret could not be inspected: keyring service unavailable",
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

  assert.match(html, /role="alert"/);
  assert.match(html, /keyring service unavailable/);
});

test("renders api key reference error warning in remote TTS panel", () => {
  const html = renderSettingsRemoteTtsPanel({
    profileName: "openai-tts-default",
    provider: "OpenAI",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-4o-mini-tts",
    apiKeyReference: "OS keyring entry: blind_browser / remote_tts:openai-tts-default:api_key",
    apiKeyMaskedValue: null,
    apiKeyReferenceError: "Configured secret could not be inspected: keyring service unavailable",
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

  assert.match(html, /role="alert"/);
  assert.match(html, /keyring service unavailable/);
});
