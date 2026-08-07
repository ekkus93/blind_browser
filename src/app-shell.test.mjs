import test from "node:test";
import assert from "node:assert/strict";

import { renderAppShell } from "./app-shell.tsx";

test("workspace shell keeps the main page focused on live panels", async () => {
  const html = await renderAppShell();

  assert.ok(!html.includes("<h1>Workspace</h1>"));
  assert.match(html, /aria-label="Open settings"/);
  assert.ok(!html.includes('data-app-view-button="workspace"'));
  assert.ok(!html.includes("Voice-first browser"));
  assert.ok(!html.includes("Speak commands here, then keep moving through listening, reading, and confirmation."));
  assert.ok(!html.includes("Page actions"));
  assert.ok(!html.includes("See what the browser, narration, and listening state are doing right now."));
  assert.ok(html.includes('data-workspace-control-bar="true"'));
  assert.ok(html.includes('data-panel-root="push-to-talk"'));
  assert.ok(html.includes('data-panel-root="url-input"'));
  assert.ok(html.includes('data-panel-root="status"'));
});

test("settings shell groups related sections in a logical order", async () => {
  const html = await renderAppShell();

  assert.match(html, /<h1[^>]*>Settings<\/h1>/);
  assert.ok(!html.includes('data-app-view-button="workspace"'));
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

test("app-alert renders above both workspace and settings sections", async () => {
  const alertHtml = '<section class="app-alert app-alert-error" role="alert">Could not open the external link. Copy this URL and open it manually: https://example.test/docs.</section>';

  const workspaceHtml = await renderAppShell({
    initialAppView: "workspace",
    panelContent: { "app-alert": alertHtml },
  });
  assert.match(workspaceHtml, /Could not open the external link/);
  assert.ok(workspaceHtml.indexOf("app-alert") < workspaceHtml.indexOf('data-app-view-section="workspace"'));

  const settingsHtml = await renderAppShell({
    initialAppView: "settings",
    initialSettingsView: "planner",
    panelContent: { "app-alert": alertHtml },
  });
  assert.match(settingsHtml, /Could not open the external link/);
  assert.ok(settingsHtml.indexOf("app-alert") < settingsHtml.indexOf('data-settings-view-section="planner"'));
});

test("confirmation panel stays reachable when the app is in settings view", async () => {
  // Regression test: the confirmation-panel root (action-approval panel +
  // remote-data-consent dialog) used to render inside the workspace-only
  // section, which carries hidden + aria-hidden whenever the app is in
  // settings view. Push-to-talk has no view gate, so a command issued from
  // Settings could raise a confirmation into an invisible, unfocusable,
  // screen-reader-hidden subtree. It must now render as a sibling of both
  // view sections, never nested inside either.
  const confirmationHtml = '<section data-confirmation-panel-marker="true">Approve this action</section>';

  const settingsHtml = await renderAppShell({
    initialAppView: "settings",
    initialSettingsView: "tts",
    panelContent: { "confirmation-panel": confirmationHtml },
  });

  assert.match(settingsHtml, /Approve this action/);
  const confirmationIndex = settingsHtml.indexOf("data-confirmation-panel-marker");
  const workspaceSectionIndex = settingsHtml.indexOf('data-app-view-section="workspace"');
  const settingsSectionIndex = settingsHtml.indexOf('data-app-view-section="settings"');
  assert.ok(confirmationIndex > -1, "confirmation panel content must be present in the settings-view render");
  assert.ok(
    confirmationIndex < workspaceSectionIndex && confirmationIndex < settingsSectionIndex,
    "confirmation panel must render before -- i.e. as a sibling of, not nested inside -- the workspace/settings view sections",
  );

  const workspaceHtml = await renderAppShell({
    initialAppView: "workspace",
    panelContent: { "confirmation-panel": confirmationHtml },
  });
  assert.match(workspaceHtml, /Approve this action/);
});