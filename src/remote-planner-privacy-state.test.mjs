import assert from "node:assert/strict";
import test from "node:test";

import {
  createInitialRemotePlannerPrivacyState,
  dismissRemotePlannerPrivacyOperationError,
  projectRemotePlannerPrivacyStatus,
  remotePlannerPrivacyOperationFailed,
  remotePlannerPrivacyOperationStarted,
  remotePlannerPrivacyOperationSucceeded,
  remotePlannerPrivacyReducer,
  remotePlannerPrivacyRefreshStarted,
  remotePlannerPrivacyRefreshSucceeded,
} from "./remote-planner-privacy-state.ts";

function createPrivacyStatus() {
  return {
    network_mode: "ask_per_origin",
    endpoint_scope: "https://api.example.com:443/v1",
    endpoint_display: "https://api.example.com/v1",
    endpoint_is_loopback: false,
    current_page_origin: "https://example.com",
    effective_decision: "consent_required",
    reason_code: "origin_consent_required",
    persistent_rule: null,
    session_grant_active: false,
    pending_challenge: {
      challenge_id: "challenge-1",
      request_id: "request-1",
      page_origin: "https://example.com",
      endpoint_display: "https://api.example.com/v1",
      profile_name: "openai-default",
      model_label: "gpt-test",
      policy_version: 1,
      disclosure_classes: [
        "user_transcript",
        "page_origin",
        "selected_page_regions",
      ],
      disclosure_counts: {
        selected_region_count: 2,
        selected_element_count: 1,
        ocr_derived_region_count: 0,
        tool_history_count: 1,
        skill_summary_count: 2,
        sanitized_serialized_bytes: 512,
        narration_text_bytes: 0,
        microphone_audio_duration_ms: 0,
      },
      expires_at_ms: 123456789,
      allow_once: true,
      allow_session: true,
      allow_persistent: true,
      block_persistent: true,
    },
    policy_version: 1,
    persistent_rule_count: 1,
    stale_allow_rule_count: 0,
    persistent_rules: [
      {
        page_origin: "https://blocked.example",
        decision: "block",
        endpoint_scope: null,
        endpoint_display: null,
        policy_version: 1,
        created_at_ms: 123456000,
        stale: false,
      },
    ],
    migration_notice_pending: false,
  };
}

test("privacy status projection keeps only the approved sanitized contract", () => {
  const source = createPrivacyStatus();
  source.raw_transcript = "sensitive transcript";
  source.pending_challenge.raw_payload = { page_html: "secret" };
  source.pending_challenge.disclosure_counts.hidden_byte_count = 999;
  source.persistent_rules[0].credential = "secret";

  const projected = projectRemotePlannerPrivacyStatus(source);

  assert.equal("raw_transcript" in projected, false);
  assert.equal("raw_payload" in projected.pending_challenge, false);
  assert.equal("hidden_byte_count" in projected.pending_challenge.disclosure_counts, false);
  assert.equal("credential" in projected.persistent_rules[0], false);
  assert.deepEqual(projected.pending_challenge.disclosure_classes, [
    "user_transcript",
    "page_origin",
    "selected_page_regions",
  ]);

  source.pending_challenge.disclosure_classes.push("skill_summaries");
  source.persistent_rules[0].page_origin = "https://mutated.example";
  assert.deepEqual(projected.pending_challenge.disclosure_classes, [
    "user_transcript",
    "page_origin",
    "selected_page_regions",
  ]);
  assert.equal(projected.persistent_rules[0].page_origin, "https://blocked.example");
});

test("privacy status projection rejects unsupported decisions instead of guessing", () => {
  const source = createPrivacyStatus();
  source.effective_decision = "allowed_by_legacy_fallback";

  assert.throws(
    () => projectRemotePlannerPrivacyStatus(source),
    /effective_decision contained an unsupported value/,
  );
});

test("background refresh cannot clear operation-owned busy or error state", () => {
  let state = createInitialRemotePlannerPrivacyState();

  state = remotePlannerPrivacyReducer(
    state,
    remotePlannerPrivacyOperationStarted("upsert_current_origin_rule"),
  );
  assert.equal(state.operationBusy, true);

  state = remotePlannerPrivacyReducer(
    state,
    remotePlannerPrivacyOperationFailed({
      operation: "upsert_current_origin_rule",
      error: "The origin rule could not be persisted.",
    }),
  );
  state = remotePlannerPrivacyReducer(state, remotePlannerPrivacyRefreshStarted());
  state = remotePlannerPrivacyReducer(
    state,
    remotePlannerPrivacyRefreshSucceeded(createPrivacyStatus()),
  );

  assert.equal(state.isLoaded, true);
  assert.equal(state.refreshBusy, false);
  assert.equal(state.refreshError, null);
  assert.equal(state.operationBusy, false);
  assert.equal(state.activeOperation, "upsert_current_origin_rule");
  assert.equal(state.operationError, "The origin rule could not be persisted.");
});

test("operation success owns clearing its own failure and updates validated status", () => {
  let state = createInitialRemotePlannerPrivacyState();
  state = remotePlannerPrivacyReducer(
    state,
    remotePlannerPrivacyOperationFailed({
      operation: "clear_persistent_allows",
      error: "Clear failed.",
    }),
  );
  state = remotePlannerPrivacyReducer(
    state,
    remotePlannerPrivacyOperationSucceeded(createPrivacyStatus()),
  );

  assert.equal(state.operationBusy, false);
  assert.equal(state.activeOperation, null);
  assert.equal(state.operationError, null);
  assert.equal(state.status.effective_decision, "consent_required");
});

test("operation errors can only be dismissed when no operation is active", () => {
  let state = createInitialRemotePlannerPrivacyState();
  state = remotePlannerPrivacyReducer(
    state,
    remotePlannerPrivacyOperationStarted("clear_all_persistent_rules"),
  );
  state = remotePlannerPrivacyReducer(
    state,
    dismissRemotePlannerPrivacyOperationError(),
  );
  assert.equal(state.operationBusy, true);
  assert.equal(state.activeOperation, "clear_all_persistent_rules");

  state = remotePlannerPrivacyReducer(
    state,
    remotePlannerPrivacyOperationFailed({
      operation: "clear_all_persistent_rules",
      error: "Confirmation was rejected.",
    }),
  );
  state = remotePlannerPrivacyReducer(
    state,
    dismissRemotePlannerPrivacyOperationError(),
  );
  assert.equal(state.activeOperation, null);
  assert.equal(state.operationError, null);
});
