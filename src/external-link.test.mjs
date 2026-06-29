import assert from "node:assert/strict";
import test from "node:test";

const {
  openExternalLink,
  clearAppAlert,
  setAppAlertState,
  setOpenExternalUrlForTest,
} = await import("./panel-state-setters.ts");
const { appShellStore } = await import("./store.ts");

function getAlertState() {
  return appShellStore.getState().panelStates.appAlertState;
}

test.afterEach(() => {
  clearAppAlert();
});

// ─── P0.1.2 — failure path regression ────────────────────────────────────────

test("openExternalLink surfaces rejected external open as global app alert", async () => {
  const url = "https://example.test/docs";
  const restore = setOpenExternalUrlForTest(async () => {
    throw new Error("portal unavailable");
  });

  try {
    clearAppAlert();
    openExternalLink(url);

    await Promise.resolve();
    await Promise.resolve();

    const state = getAlertState();
    assert.equal(state.kind, "error");
    assert.match(state.message, /Could not open the external link/);
    assert.match(state.message, /https:\/\/example\.test\/docs/);
    assert.match(state.message, /portal unavailable/);
  } finally {
    restore();
    clearAppAlert();
  }
});

test("openExternalLink does not throw when openExternalUrl rejects", async () => {
  const restore = setOpenExternalUrlForTest(async () => {
    throw new Error("connection refused");
  });

  try {
    let threw = false;
    try {
      openExternalLink("https://example.test/page");
      await Promise.resolve();
      await Promise.resolve();
    } catch {
      threw = true;
    }
    assert.equal(threw, false);
  } finally {
    restore();
  }
});

test("openExternalLink does not route failure to urlInputPanelState", async () => {
  const restore = setOpenExternalUrlForTest(async () => {
    throw new Error("blocked");
  });

  try {
    clearAppAlert();
    openExternalLink("https://blocked.test/");
    await Promise.resolve();
    await Promise.resolve();

    const panelStates = appShellStore.getState().panelStates;
    assert.equal(panelStates.urlInputPanelState.error, null);
    assert.equal(getAlertState().kind, "error");
  } finally {
    restore();
  }
});

// ─── P0.1.3 — dismiss behavior ───────────────────────────────────────────────

test("clearAppAlert clears global alert message", () => {
  setAppAlertState({
    kind: "error",
    message: "Could not open the external link. Copy this URL and open it manually: https://example.test/docs. portal unavailable",
  });

  clearAppAlert();

  assert.equal(getAlertState().message, null);
});

test("clearAppAlert sets kind back to info", () => {
  setAppAlertState({ kind: "error", message: "something went wrong" });
  clearAppAlert();
  assert.equal(getAlertState().kind, "info");
});
