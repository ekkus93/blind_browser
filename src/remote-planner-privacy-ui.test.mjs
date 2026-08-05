import assert from "node:assert/strict";
import test from "node:test";
import { renderToStaticMarkup } from "react-dom/server";

import { renderRemotePlannerPrivacyWorkspaceNode } from "./remote-planner-privacy-ui.tsx";

function privacyState(overrides = {}) {
  return {
    status: {
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
    },
    isLoaded: true,
    refreshBusy: false,
    refreshError: null,
    operationBusy: false,
    activeOperation: null,
    operationError: null,
    ...overrides,
  };
}

function consentState(overrides = {}) {
  return {
    kind: "awaiting-remote-data-consent",
    isSubmitting: false,
    submissionError: null,
    challenge: {
      challenge_id: "challenge-1",
      challenge_digest: "digest-must-not-render",
      request_id: "request-must-not-render",
      page_origin: "https://example.com",
      endpoint_display: "https://api.example.com/v1",
      endpoint_scope: "scope-must-not-render",
      profile_name: "openai-default",
      model_label: "gpt-test",
      policy_version: 1,
      disclosure_classes: [
        "user_transcript",
        "page_origin",
        "selected_page_regions",
        "selected_element_metadata",
        "ocr_derived_regions",
        "tool_observation_summaries",
        "skill_summaries",
        "trusted_runtime_contracts",
      ],
      disclosure_counts: {
        selected_region_count: 2,
        selected_element_count: 1,
        ocr_derived_region_count: 1,
        tool_history_count: 1,
        skill_summary_count: 2,
        sanitized_serialized_bytes: 512,
      },
      expires_at_ms: Date.UTC(2030, 0, 1),
      allow_once: true,
      allow_session: true,
      allow_persistent: true,
      block_persistent: true,
    },
    ...overrides,
  };
}

test("workspace privacy status renders the typed effective decision and safe displays", () => {
  const html = renderToStaticMarkup(
    renderRemotePlannerPrivacyWorkspaceNode(
      privacyState(),
      { kind: "idle" },
    ),
  );

  assert.match(html, /Remote data: ask for this site/);
  assert.match(html, /https:\/\/example\.com/);
  assert.match(html, /https:\/\/api\.example\.com\/v1/);
});

test("consent dialog renders categories and counts without challenge secrets or content previews", () => {
  const html = renderToStaticMarkup(
    renderRemotePlannerPrivacyWorkspaceNode(
      privacyState(),
      consentState(),
    ),
  );

  assert.match(html, /role="dialog"/);
  assert.match(html, /No planner request has been sent yet/);
  assert.match(html, /Your command transcript/);
  assert.match(html, /Locally selected page text regions/);
  assert.match(html, /Trusted runtime safety and tool contracts/);
  assert.match(html, /Sanitized request size/);
  assert.match(html, />512 bytes</);
  assert.doesNotMatch(html, /digest-must-not-render/);
  assert.doesNotMatch(html, /request-must-not-render/);
  assert.doesNotMatch(html, /scope-must-not-render/);
});

test("allow choices have no implicit default and cancel is always present", () => {
  const html = renderToStaticMarkup(
    renderRemotePlannerPrivacyWorkspaceNode(
      privacyState(),
      consentState(),
    ),
  );

  assert.match(html, /data-remote-consent-decision="allow_once"/);
  assert.match(html, /data-remote-consent-decision="allow_session"/);
  assert.match(html, /data-remote-consent-decision="allow_persistent"/);
  assert.match(html, /data-remote-consent-decision="block_persistent"/);
  assert.match(html, /data-remote-consent-decision="deny"/);
  assert.doesNotMatch(html, /autofocus/);
  assert.doesNotMatch(html, /<form/);
});

test("high-risk status exposes guidance but no network override", () => {
  const state = privacyState();
  state.status.effective_decision = "high_risk_blocked";
  state.status.reason_code = "remote_data_high_risk_blocked";

  const html = renderToStaticMarkup(
    renderRemotePlannerPrivacyWorkspaceNode(state, { kind: "idle" }),
  );

  assert.match(html, /High-risk page: network planner blocked/);
  assert.match(html, /Use a local planner or continue with direct commands/);
  assert.doesNotMatch(html, /data-remote-consent-decision=/);
});

test("submitting consent disables every decision button", () => {
  const html = renderToStaticMarkup(
    renderRemotePlannerPrivacyWorkspaceNode(
      privacyState(),
      consentState({ isSubmitting: true }),
    ),
  );

  const buttons = html.match(/data-remote-consent-decision=/g) ?? [];
  const disabled = html.match(/ disabled=""/g) ?? [];
  assert.equal(buttons.length, 5);
  assert.equal(disabled.length, 5);
  assert.match(html, /Processing privacy choice/);
});
