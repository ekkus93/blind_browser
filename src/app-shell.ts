import { CssBaseline } from "@mui/material";
import { StyledEngineProvider, ThemeProvider } from "@mui/material/styles";
import { createElement } from "react";

import { appShellTheme } from "./app-shell-theme.ts";
import {
  type AppShellNavigationHandlers,
  type AppShellPanelContent,
  type AppView,
  type SettingsStatuses,
  type SettingsView,
  renderAppViewActionButton,
  renderPanelContent,
  renderSettingsSubpageBackButton,
  renderSettingsSubpageLink,
} from "./app-shell-nav.ts";

export type {
  AppShellNavigationHandlers,
  AppShellPanelContent,
  AppView,
  PanelRootKey,
  SettingsCardStatus,
  SettingsStatuses,
  SettingsView,
} from "./app-shell-nav.ts";

export { preserveActivePanelControl } from "./app-shell-controls.ts";

const h = createElement;

interface AppShellMarkupProps {
  initialAppView: AppView;
  initialSettingsView: SettingsView;
  panelContent?: AppShellPanelContent;
  navigationHandlers?: AppShellNavigationHandlers;
  settingsStatuses?: SettingsStatuses;
}

export function AppShellMarkup({
  initialAppView,
  initialSettingsView,
  panelContent,
  navigationHandlers,
  settingsStatuses,
}: AppShellMarkupProps) {
  const workspaceActive = initialAppView === "workspace";
  const settingsActive = initialAppView === "settings";
  const showBackButton = settingsActive && initialSettingsView !== "overview";
  const showAppViewAction = workspaceActive || (settingsActive && !showBackButton);

  return h(
    "main",
    { className: "shell" },
    h(
      "header",
      { className: "shell-toolbar" },
      showAppViewAction ? renderAppViewActionButton(initialAppView, navigationHandlers) : null,
      renderSettingsSubpageBackButton(showBackButton, navigationHandlers),
      renderPanelContent("voice-status", panelContent),
    ),
    h(
      "section",
      {
        className: `app-view${workspaceActive ? " app-view-active" : ""}`,
        "data-app-view-section": "workspace",
        hidden: !workspaceActive,
        "aria-hidden": String(!workspaceActive),
      },
      h(
        "div",
        { className: "workspace-control-bar", "data-workspace-control-bar": "true" },
        renderPanelContent("push-to-talk", panelContent),
        renderPanelContent("url-input", panelContent),
      ),
      renderPanelContent("status", panelContent),
      renderPanelContent("confirmation-panel", panelContent),
    ),
    h(
      "section",
      {
        className: `app-view${settingsActive ? " app-view-active" : ""}`,
        "data-app-view-section": "settings",
        hidden: !settingsActive,
        "aria-hidden": String(!settingsActive),
      },
      h(
        "div",
        {
          className: `settings-view${initialSettingsView === "overview" ? " settings-view-active" : ""}`,
          "data-settings-view-section": "overview",
          hidden: initialSettingsView !== "overview",
          "aria-hidden": String(initialSettingsView !== "overview"),
        },
        h(
          "section",
          { className: "hero hero-settings" },
          h("h1", null, "Settings"),
        ),
        renderPanelContent("settings-guidance", panelContent),
        h(
          "section",
          {
            className: "settings-group",
            "aria-labelledby": "settings-group-playback-title",
          },
          h(
            "div",
            { className: "settings-group-copy" },
            h("p", { className: "settings-group-eyebrow" }, "Listening"),
            h("h2", { id: "settings-group-playback-title" }, "Playback"),
          ),
          renderPanelContent("audio-controls", panelContent),
        ),
        h(
          "section",
          {
            className: "settings-group settings-group-link",
            "aria-labelledby": "settings-group-planner-title",
          },
          h(
            "div",
            { className: "settings-group-copy" },
            h("p", { className: "settings-group-eyebrow" }, "Command interpretation"),
            h("h2", { id: "settings-group-planner-title" }, "AI assistant"),
          ),
          renderSettingsSubpageLink(
            "planner",
            "Open AI assistant setup",
            navigationHandlers,
            settingsStatuses?.planner,
          ),
        ),
        h(
          "section",
          {
            className: "settings-group settings-group-link",
            "aria-labelledby": "settings-group-tts-title",
          },
          h(
            "div",
            { className: "settings-group-copy" },
            h("p", { className: "settings-group-eyebrow" }, "Speech output"),
            h("h2", { id: "settings-group-tts-title" }, "Voice output"),
          ),
          renderSettingsSubpageLink(
            "tts",
            "Open voice output setup",
            navigationHandlers,
            settingsStatuses?.tts,
          ),
        ),
        h(
          "section",
          {
            className: "settings-group settings-group-link",
            "aria-labelledby": "settings-group-asr-title",
          },
          h(
            "div",
            { className: "settings-group-copy" },
            h("p", { className: "settings-group-eyebrow" }, "Speech input"),
            h("h2", { id: "settings-group-asr-title" }, "Voice input"),
          ),
          renderSettingsSubpageLink(
            "asr",
            "Open voice input setup",
            navigationHandlers,
            settingsStatuses?.asr,
          ),
        ),
        h(
          "section",
          {
            className: "settings-group settings-group-link",
            "aria-labelledby": "settings-group-runtime-title",
          },
          h(
            "div",
            { className: "settings-group-copy" },
            h("p", { className: "settings-group-eyebrow" }, "Advanced"),
            h("h2", { id: "settings-group-runtime-title" }, "Advanced settings"),
          ),
          renderSettingsSubpageLink(
            "runtime",
            "Open advanced settings",
            navigationHandlers,
            settingsStatuses?.runtime,
          ),
        ),
      ),
      h(
        "div",
        {
          className: `settings-view${initialSettingsView === "planner" ? " settings-view-active" : ""}`,
          "data-settings-view-section": "planner",
          hidden: initialSettingsView !== "planner",
          "aria-hidden": String(initialSettingsView !== "planner"),
        },
        h(
          "section",
          { className: "hero hero-settings hero-settings-subpage" },
          h("h2", null, "AI assistant setup"),
        ),
        renderPanelContent("settings-remote-planner", panelContent),
      ),
      h(
        "div",
        {
          className: `settings-view${initialSettingsView === "tts" ? " settings-view-active" : ""}`,
          "data-settings-view-section": "tts",
          hidden: initialSettingsView !== "tts",
          "aria-hidden": String(initialSettingsView !== "tts"),
        },
        h(
          "section",
          { className: "hero hero-settings hero-settings-subpage" },
          h("h2", null, "Voice output setup"),
        ),
        renderPanelContent("settings-tts-provider", panelContent),
        renderPanelContent("settings-tts-model", panelContent),
        renderPanelContent("settings-local-tts-model", panelContent),
        renderPanelContent("settings-remote-tts", panelContent),
        renderPanelContent("settings-tts-voice", panelContent),
      ),
      h(
        "div",
        {
          className: `settings-view${initialSettingsView === "asr" ? " settings-view-active" : ""}`,
          "data-settings-view-section": "asr",
          hidden: initialSettingsView !== "asr",
          "aria-hidden": String(initialSettingsView !== "asr"),
        },
        h(
          "section",
          { className: "hero hero-settings hero-settings-subpage" },
          h("h2", null, "Voice input setup"),
        ),
        renderPanelContent("settings-asr-provider", panelContent),
        renderPanelContent("settings-local-asr-model", panelContent),
        renderPanelContent("settings-remote-asr", panelContent),
      ),
      h(
        "div",
        {
          className: `settings-view${initialSettingsView === "runtime" ? " settings-view-active" : ""}`,
          "data-settings-view-section": "runtime",
          hidden: initialSettingsView !== "runtime",
          "aria-hidden": String(initialSettingsView !== "runtime"),
        },
        h(
          "section",
          { className: "hero hero-settings hero-settings-subpage" },
          h("h2", null, "Advanced settings"),
        ),
        renderPanelContent("settings-model-management", panelContent),
        renderPanelContent("settings-confirmation", panelContent),
        renderPanelContent("settings-ocr-threshold", panelContent),
      ),
    ),
  );
}

function renderShellTree(
  initialAppView: AppView,
  initialSettingsView: SettingsView,
  panelContent?: AppShellPanelContent,
  navigationHandlers?: AppShellNavigationHandlers,
  settingsStatuses?: SettingsStatuses,
) {
  return h(
    StyledEngineProvider,
    { injectFirst: true },
    h(
      ThemeProvider,
      { theme: appShellTheme },
      h(CssBaseline, null),
      h(AppShellMarkup, {
        initialAppView,
        initialSettingsView,
        panelContent,
        navigationHandlers,
        settingsStatuses,
      }),
    ),
  );
}

export function AppShellRuntime(props: {
  appView: AppView;
  settingsView: SettingsView;
  panelContent: AppShellPanelContent;
  navigationHandlers?: AppShellNavigationHandlers;
  settingsStatuses?: SettingsStatuses;
}) {
  return renderShellTree(
    props.appView,
    props.settingsView,
    props.panelContent,
    props.navigationHandlers,
    props.settingsStatuses,
  );
}

export async function renderAppShell(): Promise<string> {
  const { renderToStaticMarkup } = await import("react-dom/server");
  return renderToStaticMarkup(renderShellTree("workspace", "overview"));
}
