import assert from "node:assert/strict";
import test from "node:test";
import { renderToStaticMarkup } from "react-dom/server";

import { renderSettingsRemotePlannerPanelNode } from "./planner.tsx";
import {
  canPersistentlyAllowCurrentOrigin,
  createConfirmedClearAllRulesOperation,
  createManualOriginRuleOperation,
  createNetworkModeOperation,
  createRevokeOriginRuleOperation,
  findCurrentOriginRule,
  renderRemotePlannerPrivacySettingsCard,
} from "./planner-privacy.tsx";

function plannerState(overrides = {}) {
  return {
    profileName: "openai-default",
    provider: "OpenAI",
    baseUrl: "https://api.example.com/v1",
    model: "model",
    availableModels: [],
    loadedModelsEndpoint: null,
    isLoadingModels: false,
    isSavingConnection: false,
    isResettingConnection: false,
    isConfirmingReset: false,
    apiKeyReference: null,
    apiKeyMaskedValue: null,
    apiKeyReferenceError: null,
    organizationReference: null,
    project: null,
    temperatureMilli: 200,
    maxOutputTokens: 1024,
    timeoutMs: 30000,
    endpointIsLoopback: false,
    consentToRemotePageData: false,
    localOnly: false,
    blockedOriginsDraft: "https://bank.example",
    highRiskOriginPolicy: "block",
    remoteDataNotice: "Legacy privacy notice",
    isSavingPrivacy: false,
    apiKeyDraft: "",
    isSavingApiKey: false,
    isTestingApiKey: false,
    apiKeyTestMessage: null,
    error: null,
    ...overrides,
  };
}

function status(overrides = {}) {
  return {
    network_mode: "ask_per_origin",
    endpoint_scope: "https://api.example.com/v1",
    endpoint_display: "https://api.example.com/v1",
    endpoint_is_loopback: false,
    current_page_origin: "https://example.com",
    effective_decision: "allowed_persistent",
    reason_code: null,
    persistent_rule: "allow",
    session_grant_active: false,
    pending_challenge: null,
    policy_version: 2,
    persistent_rule_count: 3,
    stale_allow_rule_count: 1,
    persistent_rules: [
      {
        page_origin: "https://example.com",
        decision: "allow",
        endpoint_scope: "https://api.example.com/v1",
        endpoint_display: "https://api.example.com/v1",
        policy_version: 2,
        created_at_ms: 1700000000000,
        stale: false,
      },
      {
        page_origin: "https://old.example",
        decision: "allow",
        endpoint_scope: "https://old-api.example/v1",
        endpoint_display: "https://old-api.example/v1",
        policy_version: 1,
        created_at_ms: 1700000001000,
        stale: true,
      },
      {
        page_origin: "https://bank.example",
        decision: "block",
        endpoint_scope: null,
        endpoint_display: null,
        policy_version: 2,
        created_at_ms: 1700000002000,
        stale: false,
      },
    ],
    migration_notice_pending: true,
    ...overrides,
  };
}

function privacyState(statusOverrides = {}, stateOverrides = {}) {
  return {
    status: status(statusOverrides),
    isLoaded: true,
    refreshBusy: false,
    refreshError: null,
    operationBusy: false,
    activeOperation: null,
    operationError: null,
    ...stateOverrides,
  };
}

test("planner settings runtime removes the legacy two-toggle and blocked-origin textarea UI", () => {
  const html = renderToStaticMarkup(renderSettingsRemotePlannerPanelNode(plannerState()));

  assert.match(html, /data-remote-planner-privacy-settings="true"/);
  assert.doesNotMatch(html, /data-remote-planner-consent="true"/);
  assert.doesNotMatch(html, /data-remote-planner-local-only="true"/);
  assert.doesNotMatch(html, /data-remote-planner-blocked-origins="true"/);
  assert.match(html, /data-remote-planner-endpoint-input="true"/);
  assert.match(html, /data-remote-planner-model-input="true"/);
});

test("privacy settings expose one network mode selector and current-origin controls", () => {
  const state = privacyState({
    effective_decision: "consent_required",
    persistent_rule: null,
    persistent_rule_count: 2,
    persistent_rules: status().persistent_rules.slice(1),
  });
  const html = renderToStaticMarkup(renderRemotePlannerPrivacySettingsCard(state));

  assert.match(html, /data-remote-planner-network-mode="local_only"/);
  assert.match(html, /data-remote-planner-network-mode="ask_per_origin"/);
  assert.match(html, /data-remote-planner-network-mode="allow_sanitized_non_high_risk"/);
  assert.match(html, /data-remote-planner-current-origin="true"/);
  assert.match(html, /https:\/\/example\.com/);
  assert.match(html, /data-remote-planner-current-origin-block="true"/);
  assert.match(html, /data-remote-planner-current-origin-allow="true"/);
  assert.match(html, /Allow current site for https:\/\/api\.example\.com\/v1/);
});

test("structured rule management shows destination-bound allows stale rules blocks and migration notice", () => {
  const html = renderToStaticMarkup(renderRemotePlannerPrivacySettingsCard(privacyState()));

  assert.match(html, /data-remote-planner-rule-management="true"/);
  assert.match(html, /data-remote-planner-rule="allow"/);
  assert.match(html, /data-remote-planner-rule="block"/);
  assert.match(html, /data-remote-planner-rule-stale="true"/);
  assert.match(html, /https:\/\/old-api\.example\/v1/);
  assert.match(html, /cannot authorize transmission/);
  assert.match(html, /Privacy settings were migrated/);
  assert.match(html, /Acknowledge migration notice/);
  assert.match(html, /data-remote-planner-clear-session-grants="true"/);
  assert.match(html, /data-remote-planner-clear-persistent-allows="true"/);
  assert.match(html, /data-remote-planner-clear-all-rules="true"/);
  assert.match(html, /data-remote-planner-manual-rule="true"/);
});

test("high-risk current pages never render a persistent allow control", () => {
  const highRiskState = privacyState({
    effective_decision: "high_risk_blocked",
    reason_code: "payment_context",
    persistent_rule: null,
    persistent_rule_count: 2,
    persistent_rules: status().persistent_rules.slice(1),
  });
  const html = renderToStaticMarkup(renderRemotePlannerPrivacySettingsCard(highRiskState));

  assert.match(html, /High-risk page blocking is non-overridable/);
  assert.match(html, /data-remote-planner-current-origin-block="true"/);
  assert.doesNotMatch(html, /data-remote-planner-current-origin-allow="true"/);
  assert.equal(canPersistentlyAllowCurrentOrigin(highRiskState.status), false);
});

test("loopback destination is presented separately as on-device behavior", () => {
  const loopback = privacyState({
    endpoint_scope: "http://127.0.0.1:11434/v1",
    endpoint_display: "http://127.0.0.1:11434/v1",
    endpoint_is_loopback: true,
    effective_decision: "loopback_local",
    persistent_rule: null,
    persistent_rule_count: 0,
    stale_allow_rule_count: 0,
    persistent_rules: [],
  });
  const html = renderToStaticMarkup(renderRemotePlannerPrivacySettingsCard(loopback));

  assert.match(html, /data-remote-planner-loopback-status="true"/);
  assert.match(html, /On device/);
  assert.match(html, /Context stays on this device/);
  assert.doesNotMatch(html, /data-remote-planner-current-origin-allow="true"/);
  assert.equal(canPersistentlyAllowCurrentOrigin(loopback.status), false);
});

test("operation builders preserve fail-closed scope and explicit clear-all confirmation", () => {
  assert.deepEqual(createNetworkModeOperation("local_only"), {
    operation: "set_network_mode",
    network_mode: "local_only",
  });

  const manualAllow = createManualOriginRuleOperation("  https://example.com  ", "allow");
  assert.deepEqual(manualAllow, {
    operation: "upsert_origin_rule",
    page_origin: "https://example.com",
    decision: "allow",
  });
  assert.equal("endpoint_scope" in manualAllow, false);

  const currentRule = status().persistent_rules[0];
  assert.deepEqual(createRevokeOriginRuleOperation(currentRule), {
    operation: "revoke_origin_rule",
    page_origin: "https://example.com",
    decision: "allow",
    endpoint_scope: "https://api.example.com/v1",
  });
  assert.deepEqual(createConfirmedClearAllRulesOperation(), {
    operation: "clear_all_persistent_rules",
    confirmed: true,
  });
  assert.deepEqual(findCurrentOriginRule(status()), currentRule);
});

test("opaque origins disable persistent current-site controls", () => {
  const opaque = privacyState({
    current_page_origin: null,
    effective_decision: "origin_unavailable",
    persistent_rule: null,
  });
  const html = renderToStaticMarkup(renderRemotePlannerPrivacySettingsCard(opaque));

  assert.match(html, /does not expose a supported normalized HTTP\(S\) origin/);
  assert.doesNotMatch(html, /data-remote-planner-current-origin-block="true"/);
  assert.doesNotMatch(html, /data-remote-planner-current-origin-allow="true"/);
  assert.equal(canPersistentlyAllowCurrentOrigin(opaque.status), false);
});
