import assert from "node:assert/strict";
import test from "node:test";

import { applyAgentStateToPanels } from "./runtime-refresh.ts";

function createMinimalAgentState(remotePlannerOverrides = {}) {
  return {
    listening_state: { push_to_talk_enabled: true, is_listening: false },
    last_transcript: null,
    audio: { playback_volume: 1, playback_speed: 1 },
    remote_planner_settings: {
      profile_name: null,
      provider: null,
      base_url: "https://api.example.com/v1",
      model: "gpt-test",
      api_key_reference: null,
      api_key_masked_value: null,
      api_key_reference_error: null,
      organization_reference: null,
      project: null,
      temperature_milli: null,
      max_output_tokens: null,
      timeout_ms: null,
      endpoint_is_loopback: null,
      availability_reason: null,
      consent_to_remote_page_data: false,
      local_only: false,
      blocked_origins: [],
      high_risk_origin_policy: "block",
      remote_data_notice: "",
      ...remotePlannerOverrides,
    },
    provider_failover_settings: { planner_available: false, tts_available: false, asr_available: false, summary: "" },
    confirmation_settings: { confirmation_confidence_threshold: 0.9, allow_click_without_confirmation: true, always_confirm_submit: true },
    ocr_threshold_settings: { sparse_text_char_threshold: 200, sparse_text_region_threshold: 2 },
    asr_provider_settings: { active_mode: "Local", available_modes: ["Local"] },
    local_asr_model_settings: { profile_name: null, backend: null, model_id: null, model_path: null, language: null, threads: null },
    remote_asr_settings: { profile_name: null, provider: null, base_url: null, model: null, api_key_reference: null, api_key_masked_value: null, organization_reference: null, project: null, language: null, temperature_milli: null, timeout_ms: null },
    tts_provider_settings: { active_mode: "Local", available_modes: ["Local"] },
    tts_model_settings: { mode: "Local", active_profile: null, available_profiles: [] },
    local_tts_model_settings: { profile_name: null, backend: null, model_id: null, model_path: null, default_voice: null, sample_rate: null },
    remote_tts_settings: { profile_name: null, provider: null, base_url: null, model: null, api_key_reference: null, api_key_masked_value: null, organization_reference: null, project: null, voice: null, audio_format: null, timeout_ms: null },
    tts_voice_settings: { mode: "Local", active_voice: null, available_voices: [] },
    title: null,
    url: null,
    narration_cursor: null,
    speaking: false,
    browser_visibility: "Visible",
    browser_history: { can_go_back: false, can_go_forward: false },
  };
}

function createMinimalDeps(plannerState, onSetRemotePlanner) {
  const currentPlannerState = { ...plannerState };
  return {
    createRequestId: (prefix) => `${prefix}-test`,
    describeAudioControlFailure: (err) => String(err),
    describeScopedRuntimeRefreshFailure: (_scope, err) => String(err),
    ensureContinuousListeningLoop: () => {},
    getPanelStates: () => ({
      remotePlannerPanelState: currentPlannerState,
      urlInputPanelState: { hasUnsubmittedChanges: false, draftValue: "" },
      pushToTalkState: { isHolding: false },
    }),
    setPushToTalkState: () => {},
    setAudioControlsState: () => {},
    setRemotePlannerPanelState: (next) => {
      Object.assign(currentPlannerState, next);
      onSetRemotePlanner?.(next);
    },
    setProviderFailoverPanelState: () => {},
    setConfirmationSettingsPanelState: () => {},
    setOcrThresholdSettingsPanelState: () => {},
    setAsrProviderPanelState: () => {},
    setLocalAsrModelPanelState: () => {},
    setModelManagementPanelState: () => {},
    setRemoteAsrPanelState: () => {},
    setTtsProviderPanelState: () => {},
    setTtsModelPanelState: () => {},
    setLocalTtsModelPanelState: () => {},
    setRemoteTtsPanelState: () => {},
    setTtsVoicePanelState: () => {},
    setStatusPanelState: () => {},
    setUrlInputPanelState: () => {},
  };
}

test("runtime refresh preserves verified planner model list when endpoint still matches", () => {
  const calls = [];
  const deps = createMinimalDeps(
    {
      availableModels: ["gpt-test", "gpt-other"],
      loadedModelsEndpoint: "https://api.example.com/v1",
    },
    (next) => calls.push(next),
  );

  applyAgentStateToPanels(deps, createMinimalAgentState());

  const last = calls.at(-1);
  assert.deepEqual(last.availableModels, ["gpt-test", "gpt-other"]);
  assert.equal(last.loadedModelsEndpoint, "https://api.example.com/v1");
});

test("runtime refresh clears verified planner model list when endpoint changes", () => {
  const calls = [];
  const deps = createMinimalDeps(
    {
      availableModels: ["gpt-test", "gpt-other"],
      loadedModelsEndpoint: "https://old-endpoint.example.com/v1",
    },
    (next) => calls.push(next),
  );

  applyAgentStateToPanels(deps, createMinimalAgentState({
    base_url: "https://api.example.com/v1",
  }));

  const last = calls.at(-1);
  assert.deepEqual(last.availableModels, []);
  assert.equal(last.loadedModelsEndpoint, null);
});

test("runtime refresh clears planner model list when no verified list exists", () => {
  const calls = [];
  const deps = createMinimalDeps(
    {
      availableModels: [],
      loadedModelsEndpoint: null,
    },
    (next) => calls.push(next),
  );

  applyAgentStateToPanels(deps, createMinimalAgentState());

  const last = calls.at(-1);
  assert.deepEqual(last.availableModels, []);
  assert.equal(last.loadedModelsEndpoint, null);
});

test("runtime refresh does not clear action-owned panel errors", () => {
  const captured = {};
  const deps = createMinimalDeps(
    { availableModels: [], loadedModelsEndpoint: null },
    () => {},
  );
  // Record what generic refresh writes to each action-owned panel.
  deps.setAudioControlsState = (next) => {
    captured.audio = next;
  };
  deps.setConfirmationSettingsPanelState = (next) => {
    captured.confirmation = next;
  };
  deps.setOcrThresholdSettingsPanelState = (next) => {
    captured.ocr = next;
  };
  deps.setAsrProviderPanelState = (next) => {
    captured.asrProvider = next;
  };
  deps.setTtsProviderPanelState = (next) => {
    captured.ttsProvider = next;
  };
  deps.setTtsModelPanelState = (next) => {
    captured.ttsModel = next;
  };
  deps.setTtsVoicePanelState = (next) => {
    captured.ttsVoice = next;
  };
  deps.setUrlInputPanelState = (next) => {
    captured.urlInput = next;
  };

  applyAgentStateToPanels(deps, createMinimalAgentState());

  for (const [panel, next] of Object.entries(captured)) {
    assert.ok(
      !("error" in next),
      `generic runtime refresh must not clear the ${panel} action error`,
    );
  }
});
