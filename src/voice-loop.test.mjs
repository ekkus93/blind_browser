import assert from "node:assert/strict";
import test from "node:test";

const tauriApi = await import("./tauri-api.ts");
const { cancelPushToTalk, stopContinuousListeningAfterFailure, submitConfirmationAction } =
  await import("./voice-loop.ts");
const { setPushToTalkState } = await import("./panel-state-setters.ts");
const { appShellStore } = await import("./store.ts");
const { uiStore } = await import("./ui-store.ts");

function awaitingConfirmationState(confirmationId) {
  return {
    lastOutcome: null,
    confirmation: {
      kind: "awaiting-confirmation",
      isSubmitting: false,
      submissionError: null,
      confirmationId,
      confirmationDigest: "digest-active",
      promptText: "Submit this form?",
      requestId: "req-confirm",
      selectedSkills: [],
      nextStepId: null,
      queuedStepIds: [],
    },
  };
}

function getPushToTalkState() {
  return appShellStore.getState().panelStates.pushToTalkState;
}

test.beforeEach(() => {
  setPushToTalkState({
    enabled: true,
    isHolding: false,
    isListening: true,
    isBusy: false,
    lastError: null,
    lastTranscript: null,
  });
});

test.afterEach(() => {
  tauriApi.__resetInvokeForTests();
});

test("cancelPushToTalk preserves prior listening state when the backend stop fails", async () => {
  tauriApi.__setInvokeForTests(async () => {
    throw new Error("backend unavailable");
  });
  setPushToTalkState({ isHolding: true, isListening: true });

  await cancelPushToTalk();

  const state = getPushToTalkState();
  assert.equal(
    state.isListening,
    true,
    "a failed stop must not invent isListening: false",
  );
  assert.equal(state.isBusy, false);
  assert.match(
    state.lastError ?? "",
    /could not be confirmed|refresh runtime state/,
  );
});

test("stopContinuousListeningAfterFailure preserves listening state when stop fails", async () => {
  tauriApi.__setInvokeForTests(async () => {
    throw new Error("backend unavailable");
  });
  setPushToTalkState({ isListening: true });

  await stopContinuousListeningAfterFailure("Command failed.");

  const state = getPushToTalkState();
  assert.equal(
    state.isListening,
    true,
    "a failed hands-free stop must not invent isListening: false",
  );
  assert.equal(state.isBusy, false);
});

test("submitConfirmationAction surfaces an error for a stale confirmation id", () => {
  uiStore.setState(awaitingConfirmationState("active-id"));

  submitConfirmationAction("approve", "stale-id");

  const confirmation = uiStore.getState().confirmation;
  assert.equal(confirmation.kind, "awaiting-confirmation");
  assert.equal(confirmation.confirmationId, "active-id", "the active confirmation is untouched");
  assert.equal(confirmation.isSubmitting, false, "a stale click must not start a submission");
  assert.ok(confirmation.submissionError, "a stale click must surface a visible error");
  assert.match(
    confirmation.submissionError.message ?? "",
    /no longer active/,
  );
});

function microphoneConsentChallenge(requestId = "req-mic-consent") {
  return {
    challenge_id: "challenge-mic-1",
    challenge_digest: "digest-mic-1",
    request_id: requestId,
    page_origin: "https://example.com",
    endpoint_display: "https://api.example.com/v1",
    endpoint_scope: "https://api.example.com/v1",
    profile_name: "remote-asr",
    model_label: "asr-model",
    policy_version: 1,
    disclosure_classes: ["microphone_audio"],
    disclosure_counts: {
      selected_region_count: 0,
      selected_element_count: 0,
      ocr_derived_region_count: 0,
      tool_history_count: 0,
      skill_summary_count: 0,
      sanitized_serialized_bytes: 0,
      narration_text_bytes: 0,
      microphone_audio_duration_ms: 1500,
    },
    expires_at_ms: Date.now() + 60_000,
    allow_once: true,
    allow_session: true,
    allow_persistent: true,
    block_persistent: true,
  };
}

function refreshAgentStateToolResult() {
  return {
    ok: true,
    tool_name: "GetAgentState",
    request_id: "refresh-agent",
    timestamp_ms: 1,
    data: {
      listening_state: { push_to_talk_enabled: true, is_listening: false },
      last_transcript: null,
      audio: { playback_volume: 1, playback_speed: 1 },
      remote_planner_settings: {
        profile_name: null,
        provider: null,
        base_url: null,
        model: null,
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
        remote_data_notice: "",
      },
      provider_failover_settings: {
        planner_available: false,
        tts_available: false,
        asr_available: false,
        summary: "",
      },
      confirmation_settings: {
        confirmation_confidence_threshold: 0.9,
        allow_click_without_confirmation: true,
        always_confirm_submit: true,
      },
      ocr_threshold_settings: {
        sparse_text_char_threshold: 200,
        sparse_text_region_threshold: 2,
      },
      asr_provider_settings: { active_mode: "Remote", available_modes: ["Local", "Remote"] },
      local_asr_model_settings: {
        profile_name: null, backend: null, model_id: null, model_path: null, language: null, threads: null,
      },
      remote_asr_settings: {
        profile_name: "remote-asr",
        provider: "OpenAI",
        base_url: "https://api.example.com/v1",
        model: "asr-model",
        api_key_reference: null,
        api_key_masked_value: null,
        api_key_reference_error: null,
        organization_reference: null,
        project: null,
        language: "en",
        temperature_milli: 0,
        timeout_ms: 30000,
        endpoint_is_loopback: false,
        availability_reason: null,
        privacy_network_mode: "ask_per_origin",
        privacy_origin_rule_count: 0,
        privacy_notice: "Remote microphone audio requires consent.",
      },
      tts_provider_settings: { active_mode: "Local", available_modes: ["Local", "Remote"] },
      tts_model_settings: { mode: "Local", active_profile: null, available_profiles: [] },
      local_tts_model_settings: {
        profile_name: null, backend: null, model_id: null, model_path: null, default_voice: null, sample_rate: null,
      },
      remote_tts_settings: {
        profile_name: null,
        provider: null,
        base_url: null,
        model: null,
        api_key_reference: null,
        api_key_masked_value: null,
        api_key_reference_error: null,
        organization_reference: null,
        project: null,
        voice: null,
        audio_format: null,
        timeout_ms: null,
        endpoint_is_loopback: null,
        availability_reason: null,
        privacy_network_mode: "ask_per_origin",
        privacy_origin_rule_count: 0,
        privacy_notice: "Remote narration requires consent.",
      },
      tts_voice_settings: { mode: "Local", active_voice: null, available_voices: [] },
      title: null,
      url: "https://example.com/page",
      narration_cursor: null,
      speaking: false,
      browser_visibility: "Visible",
      browser_history: { can_go_back: false, can_go_forward: false },
    },
    error: null,
    warnings: [],
    observations: [],
  };
}

function modelManagementState() {
  return {
    models_dir: "/tmp/models",
    check_on_startup: true,
    auto_download_missing: false,
    local_tts: {
      profile_name: null,
      backend: null,
      model_id: null,
      model_path: null,
      available: false,
      download_supported: false,
      download_label: null,
      download_absence_reason: null,
    },
    local_asr: {
      profile_name: null,
      backend: null,
      model_id: null,
      model_path: null,
      available: false,
      download_supported: false,
      download_label: null,
      download_absence_reason: null,
    },
  };
}

test("beginPushToTalk stages remote microphone consent and does not invent capture", async () => {
  const { beginPushToTalk } = await import("./voice-loop.ts");
  const { createInitialExecutionUiState } = await import("./planner-orchestration.ts");
  uiStore.setState(createInitialExecutionUiState());
  setPushToTalkState({
    enabled: true,
    isHolding: false,
    isListening: false,
    isBusy: false,
    lastError: null,
    lastTranscript: null,
  });
  const challenge = microphoneConsentChallenge();
  tauriApi.__setInvokeForTests(async (command) => {
    if (command === "start_listening") {
      throw {
        code: "remote_data_consent_required",
        message: "Remote microphone consent is required.",
        retryable: false,
        details: { challenge },
      };
    }
    if (command === "get_agent_state") {
      return refreshAgentStateToolResult();
    }
    if (command === "get_model_management_settings") {
      return modelManagementState();
    }
    throw new Error(`unexpected command: ${command}`);
  });

  await beginPushToTalk("keyboard");

  const ptt = getPushToTalkState();
  assert.equal(ptt.isHolding, false);
  assert.equal(ptt.isListening, false, "privacy challenge must not invent microphone capture");
  assert.match(ptt.lastError ?? "", /Capture did not start/);
  const consent = uiStore.getState().remoteDataConsent;
  assert.equal(consent.kind, "awaiting-remote-data-consent");
  assert.deepEqual(consent.challenge.disclosure_classes, ["microphone_audio"]);
});
