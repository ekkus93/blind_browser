import assert from "node:assert/strict";
import test from "node:test";

const invokeCalls = [];
let invokeImplementation = async () => {
  throw new Error("invokeImplementation was not configured");
};

const tauriApi = await import("./tauri-api.ts");

const privacyStatus = {
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
    request_id: "planner-request-1",
    page_origin: "https://example.com",
    endpoint_display: "https://api.example.com/v1",
    profile_name: "openai-default",
    model_label: "gpt-test",
    policy_version: 1,
    disclosure_classes: [
      "user_transcript",
      "page_origin",
      "selected_page_regions",
      "trusted_runtime_contracts",
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

const operationResult = {
  status: privacyStatus,
  changed: false,
  network_mode: "ask_per_origin",
  consent_to_remote_page_data: false,
  local_only: false,
  blocked_origins: ["https://blocked.example"],
};

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

test("applyRemotePlannerPrivacyOperation forwards every tagged operation without legacy fields", async () => {
  const operations = [
    { operation: "get_status" },
    {
      operation: "set_network_mode",
      network_mode: "allow_sanitized_non_high_risk",
    },
    {
      operation: "upsert_origin_rule",
      page_origin: "https://example.com",
      decision: "allow",
    },
    {
      operation: "upsert_current_origin_rule",
      decision: "block",
    },
    {
      operation: "revoke_origin_rule",
      page_origin: "https://example.com",
      decision: "allow",
      endpoint_scope: "https://api.example.com:443/v1",
    },
    { operation: "clear_session_grants" },
    { operation: "clear_persistent_allows" },
    {
      operation: "clear_all_persistent_rules",
      confirmed: false,
    },
    { operation: "acknowledge_migration_notice" },
  ];

  invokeImplementation = async () => operationResult;

  for (const [index, operation] of operations.entries()) {
    invokeCalls.length = 0;
    const requestId = `privacy-operation-${index}`;
    const result = await tauriApi.applyRemotePlannerPrivacyOperation({
      requestId,
      timeoutMs: 500,
      operation,
    });

    assert.deepEqual(result, operationResult);
    assert.deepEqual(invokeCalls, [[
      "set_remote_planner_privacy_settings",
      {
        requestId,
        timeoutMs: 500,
        operation,
      },
    ]]);
    assert.equal("consentToRemotePageData" in invokeCalls[0][1], false);
    assert.equal("localOnly" in invokeCalls[0][1], false);
    assert.equal("blockedOrigins" in invokeCalls[0][1], false);
  }
});

test("getRemotePlannerPrivacyStatus uses the typed get_status operation and returns authoritative status", async () => {
  invokeImplementation = async () => operationResult;

  const result = await tauriApi.getRemotePlannerPrivacyStatus({
    requestId: "privacy-status",
  });

  assert.deepEqual(result, privacyStatus);
  assert.deepEqual(invokeCalls, [[
    "set_remote_planner_privacy_settings",
    {
      requestId: "privacy-status",
      timeoutMs: undefined,
      operation: { operation: "get_status" },
    },
  ]]);
});

test("clear-all remains explicitly unconfirmed unless the caller supplies confirmation", async () => {
  invokeImplementation = async () => operationResult;

  await tauriApi.applyRemotePlannerPrivacyOperation({
    requestId: "privacy-clear-all",
    operation: {
      operation: "clear_all_persistent_rules",
      confirmed: false,
    },
  });

  assert.equal(invokeCalls[0][1].operation.confirmed, false);
});
