import assert from "node:assert/strict";
import test from "node:test";

import {
  describeRemotePlannerPrivacyRefreshFailure,
  refreshRemotePlannerPrivacyStatus,
} from "./runtime-refresh-with-privacy.ts";

function createPrivacyStatus() {
  return {
    network_mode: "local_only",
    endpoint_scope: null,
    endpoint_display: null,
    endpoint_is_loopback: null,
    current_page_origin: null,
    effective_decision: "local_only",
    reason_code: "network_mode_local_only",
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

test("privacy refresh reports an explicit lifecycle and authoritative status", async () => {
  const events = [];
  const status = createPrivacyStatus();

  await refreshRemotePlannerPrivacyStatus(
    {
      createRequestId: (prefix) => `${prefix}-test`,
      markRemotePlannerPrivacyRefreshStarted: () => events.push(["started"]),
      applyRemotePlannerPrivacyRefreshSuccess: (nextStatus) => {
        events.push(["succeeded", nextStatus]);
      },
      applyRemotePlannerPrivacyRefreshFailure: (message) => {
        events.push(["failed", message]);
      },
    },
    async (input) => {
      assert.deepEqual(input, {
        requestId: "remote-planner-privacy-status-test",
      });
      return status;
    },
  );

  assert.deepEqual(events, [
    ["started"],
    ["succeeded", status],
  ]);
});

test("privacy refresh surfaces structured failures instead of silently retaining defaults", async () => {
  const events = [];

  await refreshRemotePlannerPrivacyStatus(
    {
      createRequestId: () => "privacy-failure-test",
      markRemotePlannerPrivacyRefreshStarted: () => events.push(["started"]),
      applyRemotePlannerPrivacyRefreshSuccess: () => events.push(["succeeded"]),
      applyRemotePlannerPrivacyRefreshFailure: (message) => {
        events.push(["failed", message]);
      },
    },
    async () => {
      throw {
        code: "privacy_status_unavailable",
        message: "The authoritative privacy status was unavailable.",
        retryable: true,
        details: null,
      };
    },
  );

  assert.deepEqual(events, [
    ["started"],
    [
      "failed",
      "Remote planner privacy status could not be refreshed. The authoritative privacy status was unavailable.",
    ],
  ]);
});

test("privacy refresh failure descriptions do not stringify arbitrary error objects", () => {
  assert.equal(
    describeRemotePlannerPrivacyRefreshFailure({
      code: "denied",
      message: "Status access was denied.",
      retryable: false,
      details: { raw_payload: "must not be rendered" },
    }),
    "Remote planner privacy status could not be refreshed. Status access was denied.",
  );
});
