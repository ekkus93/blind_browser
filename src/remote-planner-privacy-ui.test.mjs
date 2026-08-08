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
        narration_text_bytes: 0,
        microphone_audio_duration_ms: 0,
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

test("every effective privacy decision renders a distinct textual status", () => {
  const cases = [
    ["loopback_local", "On-device planner"],
    ["local_only", "Local-only mode"],
    ["high_risk_blocked", "High-risk page: network planner blocked"],
    ["origin_blocked", "This site stays local"],
    ["allowed_global", "Remote data allowed by the global setting"],
    ["allowed_persistent", "Remote data always allowed for this site and destination"],
    ["allowed_session", "Remote data allowed for this session"],
    ["consent_required", "Remote data: ask for this site"],
    ["origin_unavailable", "Current page cannot use a network planner"],
    ["planner_unavailable", "Remote planner unavailable"],
  ];

  for (const [decision, expected] of cases) {
    const state = privacyState();
    state.status.effective_decision = decision;
    const html = renderToStaticMarkup(
      renderRemotePlannerPrivacyWorkspaceNode(state, { kind: "idle" }),
    );
    assert.ok(html.includes(expected), `${decision} should render ${expected}`);
  }
});

test("consent choices expose distinct scope labels in deterministic keyboard order", () => {
  const html = renderToStaticMarkup(
    renderRemotePlannerPrivacyWorkspaceNode(privacyState(), consentState()),
  );
  const ordered = [
    ["allow_once", "Allow sanitized data for this request only"],
    ["allow_session", "Allow sanitized data for this site and planner for this application session"],
    ["allow_persistent", "Always allow sanitized data for this site and exact planner destination"],
    ["block_persistent", "Keep this site local for every network planner"],
    ["deny", "Cancel and do not send data"],
  ];
  let previousIndex = -1;
  for (const [decision, label] of ordered) {
    const decisionIndex = html.indexOf(`data-remote-consent-decision="${decision}"`);
    assert.ok(decisionIndex > previousIndex, `${decision} should follow the prior choice`);
    assert.match(html, new RegExp(`aria-label="${label}"`));
    previousIndex = decisionIndex;
  }
});

test("expired challenge and persistence failures remain explicit and accessible", () => {
  const expiredAt = Date.UTC(2020, 0, 1);
  const state = consentState({
    challenge: {
      ...consentState().challenge,
      expires_at_ms: expiredAt,
    },
    submissionError: {
      title: "Privacy rule was not saved",
      message: "The privacy rule could not be written.",
      guidance: "No data was sent. Review storage access and try again.",
    },
  });
  const html = renderToStaticMarkup(
    renderRemotePlannerPrivacyWorkspaceNode(privacyState(), state),
  );

  assert.match(html, /<time dateTime="2020-01-01T00:00:00.000Z">/);
  assert.match(html, /data-remote-consent-error="true" role="alert"/);
  assert.match(html, /No data was sent/);
});

test("stale allow warnings use a textual status region rather than color alone", () => {
  const state = privacyState();
  state.status.stale_allow_rule_count = 1;
  const html = renderToStaticMarkup(
    renderRemotePlannerPrivacyWorkspaceNode(state, { kind: "idle" }),
  );

  assert.match(html, /role="status"/);
  assert.match(html, /saved allow rule is inactive because the destination or privacy policy changed/);
});

test("narration consent uses speech-specific disclosure copy instead of planner sanitization copy", () => {
  const state = consentState();
  state.challenge.disclosure_classes = ["narration_text"];
  state.challenge.disclosure_counts.narration_text_bytes = 321;

  const html = renderToStaticMarkup(
    renderRemotePlannerPrivacyWorkspaceNode(privacyState(), state),
  );

  assert.match(html, /Send this page narration text to the remote speech provider/);
  assert.match(html, /No narration text has been sent yet/);
  assert.match(html, /Page narration text sent to the remote speech provider/);
  assert.match(html, /Narration text size/);
  assert.match(html, />321 bytes</);
  assert.doesNotMatch(html, /Sanitized request size/);
});

test("microphone consent says audio is discarded until the user retries", () => {
  const state = consentState();
  state.challenge.disclosure_classes = ["microphone_audio"];
  state.challenge.disclosure_counts.microphone_audio_duration_ms = 3000;

  const html = renderToStaticMarkup(
    renderRemotePlannerPrivacyWorkspaceNode(privacyState(), state),
  );

  assert.match(html, /Send microphone audio to the remote transcription provider/);
  assert.match(html, /Captured audio from the paused attempt is not retained or sent/);
  assert.match(html, /Captured microphone audio sent to the remote transcription provider/);
  assert.match(html, /Requested microphone capture/);
  assert.match(html, />3000 ms</);
  assert.match(html, /Allow one new microphone upload with the same request settings/);
});
