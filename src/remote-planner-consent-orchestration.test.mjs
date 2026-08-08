import assert from "node:assert/strict";
import test from "node:test";

import {
  applyExecutionOutcomeToUiState,
  isNeedsRemoteDataConsentOutcome,
} from "./planner-orchestration.ts";
import {
  applyExecutionOutcome,
  clearRemoteDataConsent,
  createAppShellStore,
  setRemoteDataConsentError,
  setRemoteDataConsentSubmitting,
} from "./app-shell-store.ts";

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

// CR3 P2.8.4: these three tests used to drive `createExecutionUiStore` (a
// standalone in-memory `ExecutionUiStore` implementation in
// planner-orchestration.ts) rather than the real production store. That
// factory had no production caller -- the live app's `ExecutionUiStore` is
// `ui-store.ts`'s Redux-backed adapter around `app-shell-store.ts`, whose
// `executionUi` slice reducers independently reimplement this exact
// challenge-id-bound guard logic -- so the factory was deleted as dead code
// and these tests now dispatch the real Redux actions against a real store
// instead, preserving coverage of the guard behavior itself (a stale
// challenge response must not perturb a newer one) while testing the code
// that actually runs.
test("a later non-consent outcome clears the pending consent UI", () => {
  const store = createAppShellStore();
  store.dispatch(applyExecutionOutcome(consentOutcome()));

  store.dispatch(
    applyExecutionOutcome({ Complete: { trace: { executed_step_ids: [], tool_results: [] } } }),
  );

  assert.equal(store.getState().executionUi.remoteDataConsent.kind, "idle");
});

test("consent state methods reject stale challenge identifiers", () => {
  const store = createAppShellStore();
  store.dispatch(applyExecutionOutcome(consentOutcome()));

  store.dispatch(setRemoteDataConsentSubmitting({ challengeId: "old-challenge", isSubmitting: true }));
  store.dispatch(
    setRemoteDataConsentError({
      challengeId: "old-challenge",
      submissionError: {
        kind: "transport-error",
        title: "Old",
        message: "Old",
        guidance: "Old",
      },
    }),
  );
  store.dispatch(clearRemoteDataConsent({ challengeId: "old-challenge" }));

  const state = store.getState().executionUi.remoteDataConsent;
  assert.equal(state.kind, "awaiting-remote-data-consent");
  assert.equal(state.isSubmitting, false);
  assert.equal(state.submissionError, null);
});

test("consent submission is challenge-bound and preserves visible failures", () => {
  const store = createAppShellStore();
  store.dispatch(applyExecutionOutcome(consentOutcome()));

  store.dispatch(setRemoteDataConsentSubmitting({ challengeId: "challenge-1", isSubmitting: true }));
  assert.equal(store.getState().executionUi.remoteDataConsent.isSubmitting, true);

  store.dispatch(
    setRemoteDataConsentError({
      challengeId: "challenge-1",
      submissionError: {
        kind: "tool-error",
        title: "State changed",
        message: "The page changed.",
        guidance: "Review the page.",
        retryable: false,
        code: "remote_data_consent_state_changed",
      },
    }),
  );

  const state = store.getState().executionUi.remoteDataConsent;
  assert.equal(state.isSubmitting, false);
  assert.equal(state.submissionError.code, "remote_data_consent_state_changed");
});


test("aborted speech tool error promotes its consent challenge instead of clearing it", () => {
  const speechChallenge = {
    ...challenge(),
    disclosure_classes: ["narration_text"],
    disclosure_counts: {
      ...challenge().disclosure_counts,
      sanitized_serialized_bytes: 0,
      narration_text_bytes: 48,
    },
  };
  const outcome = {
    Aborted: {
      trace: { executed_step_ids: [], tool_results: [] },
      error: {
        code: "remote_data_consent_required",
        message: "Remote narration requires permission.",
        retryable: false,
        details: { challenge: speechChallenge },
      },
    },
  };

  const state = applyExecutionOutcomeToUiState(outcome);

  assert.equal(state.remoteDataConsent.kind, "awaiting-remote-data-consent");
  assert.deepEqual(state.remoteDataConsent.challenge.disclosure_classes, ["narration_text"]);
});

test("malformed consent details do not become actionable UI", () => {
  const outcome = {
    Aborted: {
      trace: { executed_step_ids: [], tool_results: [] },
      error: {
        code: "remote_data_consent_required",
        message: "Malformed privacy request.",
        retryable: false,
        details: { challenge: { challenge_id: "partial" } },
      },
    },
  };

  const state = applyExecutionOutcomeToUiState(outcome);

  assert.equal(state.remoteDataConsent.kind, "idle");
});
