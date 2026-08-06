import assert from "node:assert/strict";
import test from "node:test";

import {
  activatePrivacyConfirmationFocus,
  handlePrivacyConfirmationKeyboard,
} from "./planner-privacy-confirmation-interactions.ts";

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

test("confirmation activation focuses cancel and restores the invoking control", () => {
  const calls = [];
  const invoking = target("invoking", calls);
  const cancel = target("cancel", calls);

  const restore = activatePrivacyConfirmationFocus(invoking, cancel);
  restore();

  assert.deepEqual(calls, [
    ["focus", "cancel"],
    ["focus", "invoking"],
  ]);
});

test("confirmation focus restoration skips a disconnected invoking control", () => {
  const calls = [];
  const invoking = target("invoking", calls, { isConnected: false });
  const cancel = target("cancel", calls);

  const restore = activatePrivacyConfirmationFocus(invoking, cancel);
  restore();

  assert.deepEqual(calls, [["focus", "cancel"]]);
});

test("Escape cancels once unless the confirmation is busy", () => {
  const calls = [];
  const first = keyboardEvent("Escape");
  handlePrivacyConfirmationKeyboard({
    event: first.event,
    busy: false,
    activeElement: null,
    focusableElements: [],
    dialogRoot: target("root", calls),
    cancel: () => { calls.push("cancel"); },
  });
  assert.deepEqual(first.calls, ["preventDefault"]);
  assert.deepEqual(calls, ["cancel"]);

  const busy = keyboardEvent("Escape");
  handlePrivacyConfirmationKeyboard({
    event: busy.event,
    busy: true,
    activeElement: null,
    focusableElements: [],
    dialogRoot: target("root", calls),
    cancel: () => { calls.push("unexpected-cancel"); },
  });
  assert.deepEqual(busy.calls, []);
  assert.deepEqual(calls, ["cancel"]);
});

test("Tab and Shift+Tab wrap within the confirmation", () => {
  const calls = [];
  const first = target("first", calls);
  const last = target("last", calls);

  const forward = keyboardEvent("Tab");
  handlePrivacyConfirmationKeyboard({
    event: forward.event,
    busy: false,
    activeElement: last,
    focusableElements: [first, last],
    dialogRoot: target("root", calls),
    cancel: () => { throw new Error("Tab must not cancel"); },
  });

  const reverse = keyboardEvent("Tab", true);
  handlePrivacyConfirmationKeyboard({
    event: reverse.event,
    busy: false,
    activeElement: first,
    focusableElements: [first, last],
    dialogRoot: target("root", calls),
    cancel: () => { throw new Error("Tab must not cancel"); },
  });

  assert.deepEqual(forward.calls, ["preventDefault"]);
  assert.deepEqual(reverse.calls, ["preventDefault"]);
  assert.deepEqual(calls, [
    ["focus", "first"],
    ["focus", "last"],
  ]);
});

test("an empty confirmation focus set keeps focus on the dialog root", () => {
  const calls = [];
  const { event, calls: eventCalls } = keyboardEvent("Tab");
  handlePrivacyConfirmationKeyboard({
    event,
    busy: false,
    activeElement: null,
    focusableElements: [],
    dialogRoot: target("root", calls),
    cancel: () => { throw new Error("Tab must not cancel"); },
  });

  assert.deepEqual(eventCalls, ["preventDefault"]);
  assert.deepEqual(calls, [["focus", "root"]]);
});
