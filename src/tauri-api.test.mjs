import assert from "node:assert/strict";
import test from "node:test";

const invokeCalls = [];
let invokeImplementation = async () => {
  throw new Error("invokeImplementation was not configured");
};

const tauriApi = await import("./tauri-api.ts");

test.beforeEach(() => {
  invokeCalls.length = 0;
  tauriApi.__setInvokeForTests(async (...args) => {
    invokeCalls.push(args);
    return invokeImplementation(...args);
  });
});

test.afterEach(() => {
  tauriApi.__resetInvokeForTests();
});

test("setPlaybackVolume unwraps tool results and forwards request arguments", async () => {
  invokeImplementation = async () => ({
    ok: true,
    tool_name: "SetPlaybackVolume",
    request_id: "req-volume",
    timestamp_ms: 1,
    data: {
      playback_volume: 0.35,
      muted: false,
      changed: true,
    },
    error: null,
    warnings: [],
    observations: [],
  });

  const result = await tauriApi.setPlaybackVolume({
    requestId: "req-volume",
    timeoutMs: 250,
    volume: 0.35,
  });

  assert.deepEqual(result, {
    playback_volume: 0.35,
    muted: false,
    changed: true,
  });
  assert.deepEqual(invokeCalls, [[
    "set_playback_volume",
    {
      requestId: "req-volume",
      timeoutMs: 250,
      volume: 0.35,
    },
  ]]);
});

test("getAgentState requests includeLastTranscript and unwraps the backend tool result", async () => {
  invokeImplementation = async () => ({
    ok: true,
    tool_name: "GetAgentState",
    request_id: "req-agent",
    timestamp_ms: 2,
    data: {
      title: "Example",
      url: "https://example.com",
      speaking: false,
      last_transcript: "read page",
      browser_visibility: "Visible",
      browser_history: {
        can_go_back: false,
        can_go_forward: false,
        current_entry_index: null,
        entry_count: 0,
      },
      listening_state: {
        is_listening: false,
        push_to_talk_enabled: true,
      },
      audio: {
        default_tts_voice: "alloy",
        playback_volume: 1,
        playback_speed: 1,
        muted: false,
      },
      planner_provider_settings: {
        active_mode: "Remote",
        available_modes: ["Remote"],
        summary: "remote only",
      },
      remote_planner_settings: {
        profile_name: null,
        provider: null,
        base_url: null,
        model: null,
        api_key_reference: null,
        organization_reference: null,
        project: null,
        temperature_milli: null,
        max_output_tokens: null,
        timeout_ms: null,
      },
      provider_failover_settings: {
        planner_available: false,
        tts_available: false,
        asr_available: false,
        summary: "disabled",
      },
      confirmation_settings: {
        confirmation_confidence_threshold: 0.8,
        allow_click_without_confirmation: false,
        always_confirm_submit: true,
      },
      ocr_threshold_settings: {
        sparse_text_char_threshold: 200,
        sparse_text_region_threshold: 2,
      },
      asr_provider_settings: {
        active_mode: "Local",
        available_modes: ["Local", "Remote"],
      },
      local_asr_model_settings: {
        profile_name: null,
        backend: null,
        model_id: null,
        model_path: null,
        language: null,
        threads: null,
      },
      remote_asr_settings: {
        profile_name: null,
        provider: null,
        base_url: null,
        model: null,
        api_key_reference: null,
        organization_reference: null,
        project: null,
        language: null,
        temperature_milli: null,
        timeout_ms: null,
      },
      tts_provider_settings: {
        active_mode: "Local",
        available_modes: ["Local", "Remote"],
      },
      tts_model_settings: {
        mode: "Local",
        active_profile: null,
        available_profiles: [],
      },
      local_tts_model_settings: {
        profile_name: null,
        backend: null,
        model_id: null,
        model_path: null,
        default_voice: null,
        sample_rate: null,
      },
      remote_tts_settings: {
        profile_name: null,
        provider: null,
        base_url: null,
        model: null,
        api_key_reference: null,
        organization_reference: null,
        project: null,
        voice: null,
        audio_format: null,
        timeout_ms: null,
      },
      tts_voice_settings: {
        mode: "Local",
        active_voice: null,
        available_voices: [],
      },
      narration_cursor: null,
      pending_confirmation_id: null,
      pending_confirmation_prompt: null,
      pending_plan_execution: null,
    },
    error: null,
    warnings: [],
    observations: [],
  });

  const result = await tauriApi.getAgentState({
    requestId: "req-agent",
    includeLastTranscript: true,
  });

  assert.equal(result.last_transcript, "read page");
  assert.deepEqual(invokeCalls, [[
    "get_agent_state",
    {
      requestId: "req-agent",
      timeoutMs: undefined,
      includeLastTranscript: true,
    },
  ]]);
});

test("setModelManagementSettings forwards camelCase frontend fields to the Tauri command", async () => {
  invokeImplementation = async () => ({
    models_dir: "/tmp/models",
    check_on_startup: true,
    auto_download_missing: false,
  });

  const result = await tauriApi.setModelManagementSettings({
    requestId: "req-models",
    timeoutMs: 500,
    modelsDir: "/tmp/models",
    checkOnStartup: true,
    autoDownloadMissing: false,
  });

  assert.deepEqual(result, {
    models_dir: "/tmp/models",
    check_on_startup: true,
    auto_download_missing: false,
  });
  assert.deepEqual(invokeCalls, [[
    "set_model_management_settings",
    {
      requestId: "req-models",
      timeoutMs: 500,
      modelsDir: "/tmp/models",
      checkOnStartup: true,
      autoDownloadMissing: false,
    },
  ]]);
});

test("classifyInvokeFailure prefers structured tool errors and falls back to transport messages", () => {
  assert.deepEqual(
    tauriApi.classifyInvokeFailure({
      code: "denied",
      message: "No",
      retryable: false,
      details: { field: "voice" },
    }),
    {
      kind: "tool-error",
      toolError: {
        code: "denied",
        message: "No",
        retryable: false,
        details: { field: "voice" },
      },
    },
  );

  assert.deepEqual(tauriApi.classifyInvokeFailure("transport down"), {
    kind: "transport-error",
    message: "transport down",
  });
});
