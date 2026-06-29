import assert from "node:assert/strict";
import test from "node:test";

const tauriApi = await import("./tauri-api.ts");
const { cancelPushToTalk, stopContinuousListeningAfterFailure } = await import(
  "./voice-loop.ts"
);
const { setPushToTalkState } = await import("./panel-state-setters.ts");
const { appShellStore } = await import("./store.ts");

function getPushToTalkState() {
  return appShellStore.getState().panelStates.pushToTalkState;
}

test.beforeEach(() => {
  setPushToTalkState({
    enabled: true,
    isHolding: false,
    isListening: true,
    isBusy: false,
    lastError: null,
    lastTranscript: null,
  });
});

test.afterEach(() => {
  tauriApi.__resetInvokeForTests();
});

test("cancelPushToTalk preserves prior listening state when the backend stop fails", async () => {
  tauriApi.__setInvokeForTests(async () => {
    throw new Error("backend unavailable");
  });
  setPushToTalkState({ isHolding: true, isListening: true });

  await cancelPushToTalk();

  const state = getPushToTalkState();
  assert.equal(
    state.isListening,
    true,
    "a failed stop must not invent isListening: false",
  );
  assert.equal(state.isBusy, false);
  assert.match(
    state.lastError ?? "",
    /could not be confirmed|refresh runtime state/,
  );
});

test("stopContinuousListeningAfterFailure preserves listening state when stop fails", async () => {
  tauriApi.__setInvokeForTests(async () => {
    throw new Error("backend unavailable");
  });
  setPushToTalkState({ isListening: true });

  await stopContinuousListeningAfterFailure("Command failed.");

  const state = getPushToTalkState();
  assert.equal(
    state.isListening,
    true,
    "a failed hands-free stop must not invent isListening: false",
  );
  assert.equal(state.isBusy, false);
});
