import assert from "node:assert/strict";
import test from "node:test";

import {
  activateConsentDialogFocus,
  handleConsentDialogKeyboard,
} from "./remote-planner-consent-dialog-interactions.ts";

function target(name, calls, overrides = {}) {
  return {
    name,
    isConnected: true,
    focus: () => { calls.push(["focus", name]); },
    ...overrides,
  };
}

function keyboardEvent(key, shiftKey = false) {
  const calls = [];
  return {
    calls,
    event: {
      key,
      shiftKey,
      preventDefault: () => { calls.push("preventDefault"); },
    },
  };
}

test("dialog activation focuses cancel and restores the invoking control", () => {
  const calls = [];
  const invoking = target("invoking", calls);
  const cancel = target("cancel", calls);

  const restore = activateConsentDialogFocus(invoking, cancel);
  restore();

  assert.deepEqual(calls, [
    ["focus", "cancel"],
    ["focus", "invoking"],
  ]);
});

test("focus restoration is skipped when the invoking control no longer exists", () => {
  const calls = [];
  const invoking = target("invoking", calls, { isConnected: false });
  const cancel = target("cancel", calls);

  const restore = activateConsentDialogFocus(invoking, cancel);
  restore();

  assert.deepEqual(calls, [["focus", "cancel"]]);
});

test("Escape prevents default and submits deny exactly once", () => {
  const calls = [];
  const { event, calls: eventCalls } = keyboardEvent("Escape");
  handleConsentDialogKeyboard({
    event,
    activeElement: null,
    focusableElements: [],
    dialogRoot: target("root", calls),
    submitDecision: (decision) => { calls.push(["decision", decision]); },
  });

  assert.deepEqual(eventCalls, ["preventDefault"]);
  assert.deepEqual(calls, [["decision", "deny"]]);
});

test("Tab wraps forward from the final control", () => {
  const calls = [];
  const first = target("first", calls);
  const last = target("last", calls);
  const { event, calls: eventCalls } = keyboardEvent("Tab");

  handleConsentDialogKeyboard({
    event,
    activeElement: last,
    focusableElements: [first, last],
    dialogRoot: target("root", calls),
    submitDecision: () => { throw new Error("Tab must not submit"); },
  });

  assert.deepEqual(eventCalls, ["preventDefault"]);
  assert.deepEqual(calls, [["focus", "first"]]);
});

test("Shift+Tab wraps backward from the first control", () => {
  const calls = [];
  const first = target("first", calls);
  const last = target("last", calls);
  const { event, calls: eventCalls } = keyboardEvent("Tab", true);

  handleConsentDialogKeyboard({
    event,
    activeElement: first,
    focusableElements: [first, last],
    dialogRoot: target("root", calls),
    submitDecision: () => { throw new Error("Tab must not submit"); },
  });

  assert.deepEqual(eventCalls, ["preventDefault"]);
  assert.deepEqual(calls, [["focus", "last"]]);
});

test("an empty focus set keeps focus on the dialog root", () => {
  const calls = [];
  const { event, calls: eventCalls } = keyboardEvent("Tab");

  handleConsentDialogKeyboard({
    event,
    activeElement: null,
    focusableElements: [],
    dialogRoot: target("root", calls),
    submitDecision: () => { throw new Error("Tab must not submit"); },
  });

  assert.deepEqual(eventCalls, ["preventDefault"]);
  assert.deepEqual(calls, [["focus", "root"]]);
});

test("unhandled keys neither move focus nor submit a decision", () => {
  const calls = [];
  const { event, calls: eventCalls } = keyboardEvent("Enter");

  handleConsentDialogKeyboard({
    event,
    activeElement: null,
    focusableElements: [target("button", calls)],
    dialogRoot: target("root", calls),
    submitDecision: (decision) => { calls.push(["decision", decision]); },
  });

  assert.deepEqual(eventCalls, []);
  assert.deepEqual(calls, []);
});
