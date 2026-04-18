import test from "node:test";
import assert from "node:assert/strict";

import { renderAppShell } from "./app-shell.ts";

test("settings shell groups related sections in a logical order", () => {
  const html = renderAppShell();

  assert.ok(html.includes("<h1>Settings</h1>"));
  assert.ok(html.includes("settings-group-playback-title"));
  assert.ok(html.includes("settings-group-planner-title"));
  assert.ok(html.includes("settings-group-tts-title"));
  assert.ok(html.includes("settings-group-asr-title"));
  assert.ok(html.includes("settings-group-runtime-title"));

  assert.ok(!html.includes('data-panel-root="settings-volume"'));
  assert.ok(!html.includes('data-panel-root="settings-speed"'));
  assert.ok(html.indexOf('data-panel-root="settings-remote-planner"') < html.indexOf('data-panel-root="settings-tts-provider"'));
  assert.ok(html.indexOf('data-panel-root="settings-tts-voice"') < html.indexOf('data-panel-root="settings-asr-provider"'));
  assert.ok(html.indexOf('data-panel-root="settings-remote-asr"') < html.indexOf('data-panel-root="settings-provider-failover"'));
});