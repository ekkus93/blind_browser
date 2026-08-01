import assert from "node:assert/strict";
import test from "node:test";

const tauriApi = await import("./tauri-api.ts");
const { cancelPushToTalk, stopContinuousListeningAfterFailure, submitConfirmationAction } =
  await import("./voice-loop.ts");
const { setPushToTalkState } = await import("./panel-state-setters.ts");
const { appShellStore } = await import("./store.ts");
const { uiStore } = await import("./ui-store.ts");

function awaitingConfirmationState(confirmationId) {
  return {
    lastOutcome: null,
    confirmation: {
      kind: "awaiting-confirmation",
      isSubmitting: false,
      submissionError: null,
      confirmationId,
      confirmationDigest: "digest-active",
      promptText: "Submit this form?",
      requestId: "req-confirm",
      selectedSkills: [],
      nextStepId: null,
      queuedStepIds: [],
    },
  };
}

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

test("submitConfirmationAction surfaces an error for a stale confirmation id", () => {
  uiStore.setState(awaitingConfirmationState("active-id"));

  submitConfirmationAction("approve", "stale-id");

  const confirmation = uiStore.getState().confirmation;
  assert.equal(confirmation.kind, "awaiting-confirmation");
  assert.equal(confirmation.confirmationId, "active-id", "the active confirmation is untouched");
  assert.equal(confirmation.isSubmitting, false, "a stale click must not start a submission");
  assert.ok(confirmation.submissionError, "a stale click must surface a visible error");
  assert.match(
    confirmation.submissionError.message ?? "",
    /no longer active/,
  );
});
