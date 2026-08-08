import assert from "node:assert/strict";
import test from "node:test";

import { createRemotePlannerPrivacyController } from "./remote-planner-privacy-controller.ts";
import { createInitialExecutionUiState } from "./planner-orchestration.ts";
import { createInitialRemotePlannerPrivacyState } from "./remote-planner-privacy-state.ts";

function privacyStatus() {
  return {
    network_mode: "ask_per_origin",
    endpoint_scope: "https://api.example.com:443/v1",
    endpoint_display: "https://api.example.com/v1",
    endpoint_is_loopback: false,
    current_page_origin: "https://example.com",
    effective_decision: "consent_required",
    reason_code: "remote_data_consent_required",
    persistent_rule: null,
    session_grant_active: false,
    pending_challenge: null,
    policy_version: 1,
    persistent_rule_count: 0,
    stale_allow_rule_count: 0,
    persistent_rules: [],
    migration_notice_pending: false,
  };
}

function challenge() {
  return {
    challenge_id: "challenge-1",
    challenge_digest: "digest-1",
    request_id: "request-1",
    page_origin: "https://example.com",
    endpoint_display: "https://api.example.com/v1",
    endpoint_scope: "https://api.example.com:443/v1",
    profile_name: "openai-default",
    model_label: "gpt-test",
    policy_version: 1,
    disclosure_classes: ["user_transcript", "page_origin"],
    disclosure_counts: {
      selected_region_count: 0,
      selected_element_count: 0,
      ocr_derived_region_count: 0,
      tool_history_count: 0,
      skill_summary_count: 0,
      sanitized_serialized_bytes: 64,
      narration_text_bytes: 0,
      microphone_audio_duration_ms: 0,
    },
    expires_at_ms: 123456789,
    allow_once: true,
    allow_session: true,
    allow_persistent: true,
    block_persistent: true,
  };
}

function createHarness(overrides = {}) {
  const calls = [];
  const privacyState = createInitialRemotePlannerPrivacyState();
  const executionState = createInitialExecutionUiState();
  executionState.remoteDataConsent = {
    kind: "awaiting-remote-data-consent",
    isSubmitting: false,
    submissionError: null,
    challenge: challenge(),
  };

  const dependencies = {
    getPrivacyState: () => privacyState,
    getExecutionUiState: () => executionState,
    applyPrivacyOperation: async (input) => {
      calls.push(["applyPrivacyOperation", input]);
      return {
        status: privacyStatus(),
        changed: true,
        network_mode: "ask_per_origin",
        consent_to_remote_page_data: false,
        local_only: false,
        blocked_origins: [],
      };
    },
    submitConsentResponse: async (input) => {
      calls.push(["submitConsentResponse", input]);
      return { status: "denied" };
    },
    submitNarrationConsentResponse: async (input) => {
      calls.push(["submitNarrationConsentResponse", input]);
      return { status: "denied" };
    },
    submitMicrophoneConsentResponse: async (input) => {
      calls.push(["submitMicrophoneConsentResponse", input]);
      return { status: "denied" };
    },
    executePlannerOutput: async (requestId, plannerOutput) => {
      calls.push(["executePlannerOutput", { requestId, plannerOutput }]);
      return { Complete: { trace: { executed_step_ids: [], tool_results: [] } } };
    },
    createRequestId: () => "privacy-operation-1",
    markOperationStarted: (operation) => {
      calls.push(["markOperationStarted", operation]);
      privacyState.operationBusy = true;
      privacyState.activeOperation = operation;
    },
    applyOperationSuccess: (status) => {
      calls.push(["applyOperationSuccess", status]);
      privacyState.operationBusy = false;
      privacyState.status = status;
    },
    applyOperationFailure: (operation, message) => {
      calls.push(["applyOperationFailure", { operation, message }]);
      privacyState.operationBusy = false;
      privacyState.operationError = message;
    },
    setConsentSubmitting: (challengeId, isSubmitting) => {
      calls.push(["setConsentSubmitting", { challengeId, isSubmitting }]);
      executionState.remoteDataConsent.isSubmitting = isSubmitting;
    },
    setConsentError: (challengeId, failure) => {
      calls.push(["setConsentError", { challengeId, failure }]);
      executionState.remoteDataConsent.isSubmitting = false;
      executionState.remoteDataConsent.submissionError = failure;
    },
    clearConsent: (challengeId) => {
      calls.push(["clearConsent", challengeId]);
      executionState.remoteDataConsent = { kind: "idle" };
    },
    applyExecutionOutcome: (outcome) => {
      calls.push(["applyExecutionOutcome", outcome]);
    },
    refreshRuntime: async () => {
      calls.push(["refreshRuntime"]);
    },
    reportGlobalError: (message) => {
      calls.push(["reportGlobalError", message]);
    },
    reportGlobalInfo: (message) => {
      calls.push(["reportGlobalInfo", message]);
    },
    warn: (message, details) => {
      calls.push(["warn", { message, details }]);
    },
    ...overrides,
  };

  return {
    calls,
    privacyState,
    executionState,
    controller: createRemotePlannerPrivacyController(dependencies),
  };
}

test("privacy operation controller applies only the authoritative returned status", async () => {
  const harness = createHarness();

  const result = await harness.controller.runOperation({
    operation: "set_network_mode",
    network_mode: "local_only",
  });

  assert.equal(result.changed, true);
  assert.deepEqual(harness.calls.slice(0, 3), [
    ["markOperationStarted", "set_network_mode"],
    ["applyPrivacyOperation", {
      requestId: "privacy-operation-1",
      operation: {
        operation: "set_network_mode",
        network_mode: "local_only",
      },
    }],
    ["applyOperationSuccess", privacyStatus()],
  ]);
});

test("privacy operation controller refuses a duplicate operation while busy", async () => {
  const harness = createHarness();
  harness.privacyState.operationBusy = true;
  harness.privacyState.activeOperation = "clear_session_grants";

  const result = await harness.controller.runOperation({ operation: "clear_persistent_allows" });

  assert.equal(result, null);
  assert.equal(harness.calls.some(([name]) => name === "applyPrivacyOperation"), false);
  assert.equal(harness.calls[0][0], "warn");
});

test("consent controller binds the exact active challenge digest", async () => {
  const plannerOutput = {
    status: "Complete",
    intent: { name: "ReadPage", goal: "Read", target_description: null },
    selected_skills: [],
    steps: [],
    requires_confirmation: false,
    confirmation_reason: null,
    blocked_reason: null,
    user_message: null,
  };
  const harness = createHarness({
    submitConsentResponse: async (input) => {
      harness.calls.push(["submitConsentResponse", input]);
      return { status: "resolved", planner_output: plannerOutput };
    },
  });

  const result = await harness.controller.submitConsentDecision("allow_once", "challenge-1");

  assert.equal(result.status, "resolved");
  assert.deepEqual(
    harness.calls.find(([name]) => name === "submitConsentResponse")[1],
    {
      challengeId: "challenge-1",
      challengeDigest: "digest-1",
      decision: "allow_once",
    },
  );
  assert.deepEqual(
    harness.calls.find(([name]) => name === "executePlannerOutput")[1],
    { requestId: "request-1", plannerOutput },
  );
  assert.equal(harness.executionState.remoteDataConsent.kind, "idle");
});

test("consent controller disables duplicate submission at the state boundary", async () => {
  const harness = createHarness();
  harness.executionState.remoteDataConsent.isSubmitting = true;

  const result = await harness.controller.submitConsentDecision("allow_session", "challenge-1");

  assert.equal(result, null);
  assert.equal(harness.calls.some(([name]) => name === "submitConsentResponse"), false);
  assert.equal(harness.calls[0][0], "warn");
});

test("stale consent button produces a visible error on the active challenge", async () => {
  const harness = createHarness();

  const result = await harness.controller.submitConsentDecision("deny", "old-challenge");

  assert.equal(result, null);
  const call = harness.calls.find(([name]) => name === "setConsentError");
  assert.equal(call[1].challengeId, "challenge-1");
  assert.match(call[1].failure.message, /no longer the active request/);
});

test("authoritative backend rejection clears stale controls and refreshes status", async () => {
  const harness = createHarness({
    submitConsentResponse: async () => {
      throw {
        code: "remote_data_consent_state_changed",
        message: "The page changed before consent was submitted.",
        retryable: false,
        details: null,
      };
    },
  });

  const result = await harness.controller.submitConsentDecision("allow_once", "challenge-1");

  assert.equal(result, null);
  assert.equal(harness.executionState.remoteDataConsent.kind, "idle");
  assert.equal(harness.calls.some(([name]) => name === "setConsentError"), false);
  assert.equal(harness.calls.some(([name]) => name === "clearConsent"), true);
  assert.equal(harness.calls.some(([name]) => name === "refreshRuntime"), true);
  const error = harness.calls.find(([name]) => name === "reportGlobalError")[1];
  assert.match(error, /page changed/);
});

test("persistence failure remains visible without leaving stale allow controls", async () => {
  const harness = createHarness({
    submitConsentResponse: async () => {
      throw {
        code: "remote_data_consent_persist_failed",
        message: "The privacy rule could not be written.",
        retryable: false,
        details: null,
      };
    },
  });

  const result = await harness.controller.submitConsentDecision(
    "allow_persistent",
    "challenge-1",
  );

  assert.equal(result, null);
  assert.equal(harness.executionState.remoteDataConsent.kind, "idle");
  const error = harness.calls.find(([name]) => name === "reportGlobalError")[1];
  assert.match(error, /could not be written/);
  assert.equal(harness.calls.some(([name]) => name === "refreshRuntime"), true);
});

test("transport failure keeps the bounded challenge visible for explicit retry", async () => {
  const harness = createHarness({
    submitConsentResponse: async () => {
      throw new Error("Tauri transport disconnected");
    },
  });

  const result = await harness.controller.submitConsentDecision("allow_once", "challenge-1");

  assert.equal(result, null);
  assert.equal(harness.executionState.remoteDataConsent.kind, "awaiting-remote-data-consent");
  assert.equal(harness.executionState.remoteDataConsent.isSubmitting, false);
  assert.match(
    harness.executionState.remoteDataConsent.submissionError.message,
    /transport disconnected/,
  );
  assert.equal(harness.calls.some(([name]) => name === "clearConsent"), false);
});

test("backend rejection reports a refresh failure without restoring the challenge", async () => {
  const harness = createHarness({
    submitConsentResponse: async () => {
      throw {
        code: "remote_data_consent_expired",
        message: "The privacy request expired.",
        retryable: false,
        details: null,
      };
    },
    refreshRuntime: async () => {
      harness.calls.push(["refreshRuntime"]);
      throw new Error("refresh unavailable");
    },
  });

  const result = await harness.controller.submitConsentDecision("deny", "challenge-1");

  assert.equal(result, null);
  assert.equal(harness.executionState.remoteDataConsent.kind, "idle");
  const errors = harness.calls
    .filter(([name]) => name === "reportGlobalError")
    .map(([, message]) => message);
  assert.equal(errors.length, 2);
  assert.match(errors[1], /runtime status could not be refreshed/);
});

test("narration consent is submitted to the narration handler instead of the planner handler", async () => {
  const harness = createHarness({
    submitNarrationConsentResponse: async (input) => {
      harness.calls.push(["submitNarrationConsentResponse", input]);
      return { status: "spoken" };
    },
  });
  harness.executionState.remoteDataConsent.challenge.disclosure_classes = ["narration_text"];
  harness.executionState.remoteDataConsent.challenge.disclosure_counts.narration_text_bytes = 42;

  const result = await harness.controller.submitConsentDecision("allow_once", "challenge-1");

  assert.equal(result.status, "spoken");
  assert.equal(harness.executionState.remoteDataConsent.kind, "idle");
  assert.equal(harness.calls.some(([name]) => name === "submitConsentResponse"), false);
  assert.equal(harness.calls.some(([name]) => name === "submitNarrationConsentResponse"), true);
});

test("microphone authorization is routed separately and tells the user to repeat input", async () => {
  const calls = [];
  const harness = createHarness({
    submitMicrophoneConsentResponse: async (input) => {
      calls.push(["submitMicrophoneConsentResponse", input]);
      return { status: "authorized_retry_required" };
    },
    reportGlobalInfo: (message) => {
      calls.push(["reportGlobalInfo", message]);
    },
  });
  harness.executionState.remoteDataConsent.challenge.disclosure_classes = ["microphone_audio"];
  harness.executionState.remoteDataConsent.challenge.disclosure_counts.microphone_audio_duration_ms = 3000;

  const result = await harness.controller.submitConsentDecision("allow_session", "challenge-1");

  assert.equal(result.status, "authorized_retry_required");
  assert.equal(harness.executionState.remoteDataConsent.kind, "idle");
  assert.equal(calls[0][0], "submitMicrophoneConsentResponse");
  assert.match(calls.find(([name]) => name === "reportGlobalInfo")[1], /Repeat the voice input/);
});
