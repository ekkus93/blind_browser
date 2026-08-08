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

test("submitRemotePlannerConsentResponse forwards the exact challenge binding and decision", async () => {
  const response = { status: "denied" };
  invokeImplementation = async () => response;

  const result = await tauriApi.submitRemotePlannerConsentResponse({
    challengeId: "challenge-1",
    challengeDigest: "digest-1",
    decision: "deny",
  });

  assert.deepEqual(result, response);
  assert.deepEqual(invokeCalls, [[
    "submit_remote_planner_consent_response",
    {
      challengeId: "challenge-1",
      challengeDigest: "digest-1",
      decision: "deny",
    },
  ]]);
});


test("speech consent commands forward the same exact challenge binding", async () => {
  invokeImplementation = async () => ({ status: "denied" });

  await tauriApi.submitNarrationConsentResponse({
    challengeId: "tts-challenge",
    challengeDigest: "tts-digest",
    decision: "allow_once",
  });
  await tauriApi.submitMicrophoneConsentResponse({
    challengeId: "asr-challenge",
    challengeDigest: "asr-digest",
    decision: "deny",
  });

  assert.deepEqual(invokeCalls, [
    ["submit_narration_consent_response", {
      challengeId: "tts-challenge",
      challengeDigest: "tts-digest",
      decision: "allow_once",
    }],
    ["submit_microphone_consent_response", {
      challengeId: "asr-challenge",
      challengeDigest: "asr-digest",
      decision: "deny",
    }],
  ]);
});

test("consent challenge parser accepts speech metadata and rejects malformed disclosure classes", () => {
  const challenge = {
    challenge_id: "challenge-1",
    challenge_digest: "digest-1",
    request_id: "request-1",
    page_origin: "https://example.com",
    endpoint_display: "https://api.example.com/v1",
    endpoint_scope: "https://api.example.com:443/v1",
    profile_name: "remote-asr",
    model_label: "whisper",
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
      microphone_audio_duration_ms: 500,
    },
    expires_at_ms: 2_000_000_000_000,
    allow_once: true,
    allow_session: true,
    allow_persistent: true,
    block_persistent: true,
  };

  assert.deepEqual(tauriApi.parseRemoteDataConsentChallenge(challenge), challenge);
  assert.equal(
    tauriApi.parseRemoteDataConsentChallenge({ ...challenge, disclosure_classes: ["unknown_audio"] }),
    null,
  );
});

test("invoke error challenge extraction requires the exact privacy error code and valid shape", () => {
  const challenge = {
    challenge_id: "challenge-1",
    challenge_digest: "digest-1",
    request_id: "request-1",
    page_origin: "https://example.com",
    endpoint_display: "https://api.example.com/v1",
    endpoint_scope: "https://api.example.com:443/v1",
    profile_name: "remote-tts",
    model_label: "tts-model",
    policy_version: 1,
    disclosure_classes: ["narration_text"],
    disclosure_counts: {
      selected_region_count: 0,
      selected_element_count: 0,
      ocr_derived_region_count: 0,
      tool_history_count: 0,
      skill_summary_count: 0,
      sanitized_serialized_bytes: 0,
      narration_text_bytes: 42,
      microphone_audio_duration_ms: 0,
    },
    expires_at_ms: 2_000_000_000_000,
    allow_once: true,
    allow_session: true,
    allow_persistent: true,
    block_persistent: true,
  };

  assert.equal(
    tauriApi.remoteDataConsentChallengeFromInvokeError({
      code: "some_other_error",
      message: "not consent",
      retryable: false,
      details: { challenge },
    }),
    null,
  );
  assert.equal(
    tauriApi.remoteDataConsentChallengeFromInvokeError({
      code: "remote_data_consent_required",
      message: "permission required",
      retryable: false,
      details: { challenge: { ...challenge, allow_once: "yes" } },
    }),
    null,
  );
  assert.deepEqual(
    tauriApi.remoteDataConsentChallengeFromInvokeError({
      code: "remote_data_consent_required",
      message: "permission required",
      retryable: false,
      details: { challenge },
    }),
    challenge,
  );
});
