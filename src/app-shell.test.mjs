import test from "node:test";
import assert from "node:assert/strict";

import { renderAppShell } from "./app-shell.ts";

test("workspace shell uses plain-language overview copy", async () => {
  const html = await renderAppShell();

  assert.ok(html.includes("<h1>Workspace</h1>"));
  assert.match(html, /Open pages, speak commands, control reading, and check the current state here\./);
  assert.match(html, /Speak commands here, then keep moving through listening, reading, and confirmation\./);
  assert.match(html, /<h2>Page actions<\/h2>/);
  assert.match(html, /without leaving the workspace\./);
  assert.match(html, /<h2>Status<\/h2>/);
});

test("settings shell groups related sections in a logical order", async () => {
  const html = await renderAppShell();

  assert.ok(html.includes("<h1>Settings</h1>"));
  assert.ok(html.includes('data-settings-view-section="overview"'));
  assert.ok(html.includes('data-settings-view-section="planner"'));
  assert.ok(html.includes('data-settings-view-section="tts"'));
  assert.ok(html.includes('data-settings-view-section="asr"'));
  assert.ok(html.includes('data-settings-view-section="runtime"'));
  assert.ok(html.includes("settings-group-playback-title"));
  assert.ok(html.includes("settings-group-planner-title"));
  assert.ok(html.includes("settings-group-tts-title"));
  assert.ok(html.includes("settings-group-asr-title"));
  assert.ok(html.includes("settings-group-runtime-title"));
  assert.ok(html.includes('data-settings-view-button="planner"'));
  assert.ok(html.includes('data-settings-view-button="tts"'));
  assert.ok(html.includes('data-settings-view-button="asr"'));
  assert.ok(html.includes('data-settings-view-button="runtime"'));
  assert.ok(html.includes('data-settings-view-button="overview"'));
  assert.match(html, /aria-label="Back to settings"/);
  assert.ok(html.includes('data-settings-subpage-back="true"'));

  assert.ok(!html.includes('data-panel-root="settings-volume"'));
  assert.ok(!html.includes('data-panel-root="settings-speed"'));
  assert.ok(html.indexOf('data-panel-root="settings-remote-planner"') < html.indexOf('data-panel-root="settings-asr-provider"'));
  assert.ok(html.indexOf('data-panel-root="settings-tts-provider"') < html.indexOf('data-panel-root="settings-asr-provider"'));
  assert.ok(html.indexOf('data-panel-root="settings-remote-planner"') < html.indexOf('data-panel-root="settings-tts-provider"'));
  assert.ok(html.indexOf('data-panel-root="settings-remote-asr"') < html.indexOf('data-panel-root="settings-model-management"'));
});