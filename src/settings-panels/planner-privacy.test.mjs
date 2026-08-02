import assert from "node:assert/strict";
import test from "node:test";
import { renderToStaticMarkup } from "react-dom/server";

import { renderSettingsRemotePlannerPanelNode } from "./planner.tsx";

function state(overrides = {}) {
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
    remoteDataNotice: "Network planner endpoints receive sanitized context after explicit consent.",
    isSavingPrivacy: false,
    apiKeyDraft: "",
    isSavingApiKey: false,
    isTestingApiKey: false,
    apiKeyTestMessage: null,
    error: null,
    ...overrides,
  };
}

test("remote planner panel prominently exposes consent local-only and origin controls", () => {
  const html = renderToStaticMarkup(renderSettingsRemotePlannerPanelNode(state()));
  assert.match(html, /data-remote-planner-privacy="true"/);
  assert.match(html, /data-remote-planner-consent="true"/);
  assert.match(html, /data-remote-planner-local-only="true"/);
  assert.match(html, /data-remote-planner-blocked-origins="true"/);
  assert.match(html, /High-risk authentication, payment, identity, health, wallet/);
  assert.match(html, /explicit consent/);
});

test("loopback endpoint indication states that context stays on device", () => {
  const html = renderToStaticMarkup(renderSettingsRemotePlannerPanelNode(state({ endpointIsLoopback: true })));
  assert.match(html, /Planner context stays on this device/);
});
