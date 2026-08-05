import assert from "node:assert/strict";
import test from "node:test";

const tauriApi = await import("./tauri-api.ts");
const plannerActions = await import("./planner-actions.ts");

test("legacy planner privacy adapter exports are removed", () => {
  assert.equal("setRemotePlannerPrivacySettings" in tauriApi, false);
  assert.equal("persistRemotePlannerPrivacyPolicy" in plannerActions, false);
  assert.equal("parseBlockedOriginsDraft" in plannerActions, false);
});
