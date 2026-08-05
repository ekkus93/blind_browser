import assert from "node:assert/strict";
import test from "node:test";

import {
  applyExecutionOutcomeToUiState,
  createExecutionUiStore,
  isNeedsRemoteDataConsentOutcome,
} from "./planner-orchestration.ts";

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
    },
    expires_at_ms: 123456789,
    allow_once: true,
    allow_session: true,
    allow_persistent: true,
    block_persistent: true,
  };
}

function consentOutcome() {
  return {
    NeedsRemoteDataConsent: {
      trace: { executed_step_ids: [], tool_results: [] },
      challenge: challenge(),
    },
  };
}

test("consent outcome creates a dedicated consent state and clears action confirmation", () => {
  const outcome = consentOutcome();
  assert.equal(isNeedsRemoteDataConsentOutcome(outcome), true);

  const state = applyExecutionOutcomeToUiState(outcome);

  assert.equal(state.confirmation.kind, "idle");
  assert.equal(state.remoteDataConsent.kind, "awaiting-remote-data-consent");
  assert.equal(state.remoteDataConsent.challenge.challenge_digest, "digest-1");
  assert.equal(state.remoteDataConsent.isSubmitting, false);
  assert.equal(state.remoteDataConsent.submissionError, null);
});

test("a later non-consent outcome clears the pending consent UI", () => {
  const store = createExecutionUiStore();
  store.applyOutcome(consentOutcome());

  store.applyOutcome({ Complete: { trace: { executed_step_ids: [], tool_results: [] } } });

  assert.equal(store.getState().remoteDataConsent.kind, "idle");
});

test("consent state methods reject stale challenge identifiers", () => {
  const store = createExecutionUiStore();
  store.applyOutcome(consentOutcome());

  store.setRemoteDataConsentSubmitting("old-challenge", true);
  store.setRemoteDataConsentError("old-challenge", {
    kind: "transport-error",
    title: "Old",
    message: "Old",
    guidance: "Old",
  });
  store.clearRemoteDataConsent("old-challenge");

  const state = store.getState().remoteDataConsent;
  assert.equal(state.kind, "awaiting-remote-data-consent");
  assert.equal(state.isSubmitting, false);
  assert.equal(state.submissionError, null);
});

test("consent submission is challenge-bound and preserves visible failures", () => {
  const store = createExecutionUiStore();
  store.applyOutcome(consentOutcome());

  store.setRemoteDataConsentSubmitting("challenge-1", true);
  assert.equal(store.getState().remoteDataConsent.isSubmitting, true);

  store.setRemoteDataConsentError("challenge-1", {
    kind: "tool-error",
    title: "State changed",
    message: "The page changed.",
    guidance: "Review the page.",
    retryable: false,
    code: "remote_data_consent_state_changed",
  });

  const state = store.getState().remoteDataConsent;
  assert.equal(state.isSubmitting, false);
  assert.equal(state.submissionError.code, "remote_data_consent_state_changed");
});
