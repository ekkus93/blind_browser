import assert from "node:assert/strict";
import test from "node:test";

const invokeCalls = [];
let invokeImplementation = async () => {
  throw new Error("invokeImplementation was not configured");
};

const tauriApi = await import("./tauri-api.ts");

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

test("submitRemotePlannerConsentResponse forwards the exact challenge binding and decision", async () => {
  const response = { status: "denied" };
  invokeImplementation = async () => response;

  const result = await tauriApi.submitRemotePlannerConsentResponse({
    challengeId: "challenge-1",
    challengeDigest: "digest-1",
    decision: "deny",
  });

  assert.deepEqual(result, response);
  assert.deepEqual(invokeCalls, [[
    "submit_remote_planner_consent_response",
    {
      challengeId: "challenge-1",
      challengeDigest: "digest-1",
      decision: "deny",
    },
  ]]);
});
