import ArrowBackRoundedIcon from "@mui/icons-material/ArrowBackRounded";
import { Button, CssBaseline, IconButton } from "@mui/material";
import { StyledEngineProvider, ThemeProvider, createTheme } from "@mui/material/styles";
import { createElement, type ComponentProps, type ReactNode } from "react";
import { flushSync } from "react-dom";
import { createRoot, type Root } from "react-dom/client";

export type PanelRootKey =
  | "push-to-talk"
  | "url-input"
  | "status"
  | "audio-controls"
  | "settings-guidance"
  | "settings-remote-planner"
  | "settings-provider-failover"
  | "settings-confirmation"
  | "settings-ocr-threshold"
  | "settings-asr-provider"
  | "settings-local-asr-model"
  | "settings-model-management"
  | "settings-remote-asr"
  | "settings-tts-provider"
  | "settings-tts-model"
  | "settings-local-tts-model"
  | "settings-remote-tts"
  | "settings-tts-voice"
  | "confirmation-panel";

export type AppView = "workspace" | "settings";
export type SettingsView = "overview" | "planner" | "tts" | "asr" | "runtime";

export type PanelRootMap = Record<PanelRootKey, HTMLDivElement>;

const h = createElement;

const appShellTheme = createTheme({
  palette: {
    mode: "light",
    primary: {
      main: "#29583f",
      dark: "#1f7f5c",
      contrastText: "#fffdf8",
    },
    secondary: {
      main: "#7a5727",
    },
    background: {
      default: "#f7f4ec",
      paper: "rgba(255, 252, 247, 0.9)",
    },
    text: {
      primary: "#1d1a16",
      secondary: "#433d37",
    },
  },
  shape: {
    borderRadius: 18,
  },
  typography: {
    fontFamily: '"IBM Plex Sans", "Segoe UI", sans-serif',
    button: {
      textTransform: "none",
      fontWeight: 700,
    },
  },
});

const mountedShellRoots = new WeakMap<HTMLDivElement, Root>();
const mountedPanelRoots = new WeakMap<HTMLDivElement, Root>();

type DataAttributes = {
  [key: `data-${string}`]: string;
};

type ButtonWithDataProps = ComponentProps<typeof Button> & DataAttributes;
type IconButtonWithDataProps = ComponentProps<typeof IconButton> & DataAttributes;
export type AppShellPanelContent = Partial<Record<PanelRootKey, ReactNode>>;

function renderPanelRootPlaceholderElement(rootKey: PanelRootKey) {
  return h("div", {
    "data-panel-root": rootKey,
  });
}

function renderPanelContent(rootKey: PanelRootKey, panelContent?: AppShellPanelContent) {
  const content = panelContent?.[rootKey];
  return content !== undefined ? content : renderPanelRootPlaceholderElement(rootKey);
}

function renderShellNavButton(view: AppView, label: string, isActive: boolean) {
  const buttonProps: ButtonWithDataProps = {
    type: "button",
    className: `shell-nav-button${isActive ? " shell-nav-button-active" : ""}`,
    disableElevation: true,
    variant: isActive ? "contained" : "text",
    "data-app-view-button": view,
    "aria-pressed": isActive,
    sx: {
      minWidth: 0,
      px: 2.25,
      py: 1.4,
    },
  };

  return h(
    Button,
    buttonProps,
    label,
  );
}

function renderSettingsSubpageBackButton(showBackButton: boolean) {
  const buttonProps: IconButtonWithDataProps = {
    type: "button",
    className: "settings-subpage-back",
    "data-settings-subpage-back": "true",
    "data-settings-view-button": "overview",
    "aria-label": "Back to settings",
    title: "Back to settings",
    hidden: !showBackButton,
    "aria-hidden": !showBackButton,
    size: "large",
  };

  return h(
    IconButton,
    buttonProps,
    h(ArrowBackRoundedIcon, {
      className: "settings-subpage-back-icon",
      fontSize: "small",
      "aria-hidden": true,
    }),
  );
}

function renderWorkspaceOverviewCard(title: string, copy: string) {
  return h(
    "article",
    { className: "panel" },
    h("h2", null, title),
    h("p", null, copy),
  );
}

function renderSettingsSubpageLink(view: Exclude<SettingsView, "overview">, label: string) {
  const buttonProps: ButtonWithDataProps = {
    type: "button",
    className: "settings-subpage-link",
    variant: "text",
    disableRipple: true,
    "data-settings-view-button": view,
    sx: {
      justifyContent: "flex-start",
      minWidth: 0,
      p: 0,
    },
  };

  return h(
    "div",
    { className: "settings-subpage-card" },
    h(
      Button,
      buttonProps,
      label,
    ),
  );
}

interface AppShellMarkupProps {
  initialAppView: AppView;
  initialSettingsView: SettingsView;
  panelContent?: AppShellPanelContent;
}

function AppShellMarkup({ initialAppView, initialSettingsView, panelContent }: AppShellMarkupProps) {
  const workspaceActive = initialAppView === "workspace";
  const settingsActive = initialAppView === "settings";
  const showBackButton = settingsActive && initialSettingsView !== "overview";

  return h(
    "main",
    { className: "shell" },
    h(
      "header",
      { className: "shell-toolbar" },
      h(
        "nav",
        {
          className: "shell-nav",
          "aria-label": "App pages",
        },
        renderShellNavButton("workspace", "Workspace", workspaceActive),
        renderShellNavButton("settings", "Settings", settingsActive),
      ),
      renderSettingsSubpageBackButton(showBackButton),
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
        "section",
        { className: "hero" },
        h("p", { className: "eyebrow" }, "Voice-first browser"),
        h("h1", null, "Workspace"),
        h(
          "p",
          { className: "lede" },
          "Open pages, speak commands, control reading, and check the current state here. Settings stay on a separate page so this workflow stays focused.",
        ),
      ),
      h(
        "section",
        { className: "panels", "aria-label": "Workspace sections" },
        renderWorkspaceOverviewCard(
          "Voice input",
          "Speak commands here, then keep moving through listening, reading, and confirmation.",
        ),
        renderWorkspaceOverviewCard(
          "Page actions",
          "Open a page, start reading, move forward or back, and stop without leaving the workspace.",
        ),
        renderWorkspaceOverviewCard(
          "Status",
          "See what the browser, narration, and listening state are doing right now.",
        ),
      ),
      renderPanelContent("push-to-talk", panelContent),
      renderPanelContent("url-input", panelContent),
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
            h("h2", { id: "settings-group-planner-title" }, "Planner"),
          ),
          renderSettingsSubpageLink("planner", "Open planner setup"),
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
            h("h2", { id: "settings-group-tts-title" }, "Text to speech"),
          ),
          renderSettingsSubpageLink("tts", "Open TTS setup"),
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
            h("h2", { id: "settings-group-asr-title" }, "Automatic speech recognition"),
          ),
          renderSettingsSubpageLink("asr", "Open ASR setup"),
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
            h("p", { className: "settings-group-eyebrow" }, "Runtime behavior"),
            h("h2", { id: "settings-group-runtime-title" }, "Runtime"),
          ),
          renderSettingsSubpageLink("runtime", "Open Runtime setup"),
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
          h("p", { className: "settings-group-eyebrow" }, "Command interpretation"),
          h("h2", null, "Planner setup"),
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
          h("p", { className: "settings-group-eyebrow" }, "Speech output"),
          h("h2", null, "TTS setup"),
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
          h("p", { className: "settings-group-eyebrow" }, "Speech input"),
          h("h2", null, "ASR setup"),
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
          h("p", { className: "settings-group-eyebrow" }, "Runtime behavior"),
          h("h2", null, "Runtime setup"),
        ),
        renderPanelContent("settings-model-management", panelContent),
        renderPanelContent("settings-provider-failover", panelContent),
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
      }),
    ),
  );
}

export function AppShellRuntime(props: {
  appView: AppView;
  settingsView: SettingsView;
  panelContent: AppShellPanelContent;
}) {
  return renderShellTree(props.appView, props.settingsView, props.panelContent);
}

export async function renderAppShell(): Promise<string> {
  const { renderToStaticMarkup } = await import("react-dom/server");
  return renderToStaticMarkup(renderShellTree("workspace", "overview"));
}

export function setActiveAppView(appRoot: HTMLDivElement, nextView: AppView) {
  const sections = appRoot.querySelectorAll<HTMLElement>("[data-app-view-section]");
  sections.forEach((section) => {
    const isActive = section.dataset.appViewSection === nextView;
    section.hidden = !isActive;
    section.setAttribute("aria-hidden", String(!isActive));
    section.classList.toggle("app-view-active", isActive);
  });

  const buttons = appRoot.querySelectorAll<HTMLButtonElement>("[data-app-view-button]");
  buttons.forEach((button) => {
    const isActive = button.dataset.appViewButton === nextView;
    button.setAttribute("aria-pressed", String(isActive));
    button.classList.toggle("shell-nav-button-active", isActive);
  });

  const subpageBackButton = appRoot.querySelector<HTMLButtonElement>("[data-settings-subpage-back]");
  if (subpageBackButton && nextView !== "settings") {
    subpageBackButton.hidden = true;
    subpageBackButton.setAttribute("aria-hidden", "true");
  }
}

export function setActiveSettingsView(appRoot: HTMLDivElement, nextView: SettingsView) {
  const sections = appRoot.querySelectorAll<HTMLElement>("[data-settings-view-section]");
  sections.forEach((section) => {
    const isActive = section.dataset.settingsViewSection === nextView;
    section.hidden = !isActive;
    section.setAttribute("aria-hidden", String(!isActive));
    section.classList.toggle("settings-view-active", isActive);
  });

  const subpageBackButton = appRoot.querySelector<HTMLButtonElement>("[data-settings-subpage-back]");
  if (subpageBackButton) {
    const showBackButton = nextView !== "overview";
    subpageBackButton.hidden = !showBackButton;
    subpageBackButton.setAttribute("aria-hidden", String(!showBackButton));
  }
}

function requirePanelRoot(appRoot: HTMLDivElement, rootKey: PanelRootKey): HTMLDivElement {
  const root = appRoot.querySelector<HTMLDivElement>(`[data-panel-root="${rootKey}"]`);
  if (!root) {
    throw new Error(`Panel root ${rootKey} was not found.`);
  }

  return root;
}

export function createPanelRoots(appRoot: HTMLDivElement): PanelRootMap {
  let root = mountedShellRoots.get(appRoot);
  if (!root) {
    root = createRoot(appRoot);
    mountedShellRoots.set(appRoot, root);
  }

  flushSync(() => {
    root.render(renderShellTree("workspace", "overview"));
  });

  return {
    "push-to-talk": requirePanelRoot(appRoot, "push-to-talk"),
    "url-input": requirePanelRoot(appRoot, "url-input"),
    status: requirePanelRoot(appRoot, "status"),
    "audio-controls": requirePanelRoot(appRoot, "audio-controls"),
    "settings-guidance": requirePanelRoot(appRoot, "settings-guidance"),
    "settings-remote-planner": requirePanelRoot(appRoot, "settings-remote-planner"),
    "settings-provider-failover": requirePanelRoot(appRoot, "settings-provider-failover"),
    "settings-confirmation": requirePanelRoot(appRoot, "settings-confirmation"),
    "settings-ocr-threshold": requirePanelRoot(appRoot, "settings-ocr-threshold"),
    "settings-asr-provider": requirePanelRoot(appRoot, "settings-asr-provider"),
    "settings-local-asr-model": requirePanelRoot(appRoot, "settings-local-asr-model"),
    "settings-model-management": requirePanelRoot(appRoot, "settings-model-management"),
    "settings-remote-asr": requirePanelRoot(appRoot, "settings-remote-asr"),
    "settings-tts-provider": requirePanelRoot(appRoot, "settings-tts-provider"),
    "settings-tts-model": requirePanelRoot(appRoot, "settings-tts-model"),
    "settings-local-tts-model": requirePanelRoot(appRoot, "settings-local-tts-model"),
    "settings-remote-tts": requirePanelRoot(appRoot, "settings-remote-tts"),
    "settings-tts-voice": requirePanelRoot(appRoot, "settings-tts-voice"),
    "confirmation-panel": requirePanelRoot(appRoot, "confirmation-panel"),
  };
}

interface PreservedPanelControlState {
  elementId: string;
  value: string;
  selectionStart: number | null;
  selectionEnd: number | null;
  selectionDirection: "forward" | "backward" | "none" | null;
}

function captureActivePanelControl(root: HTMLDivElement): PreservedPanelControlState | null {
  const activeElement = document.activeElement;
  if (
    !activeElement
    || !root.contains(activeElement)
    || (
      !(activeElement instanceof HTMLInputElement)
      && !(activeElement instanceof HTMLTextAreaElement)
      && !(activeElement instanceof HTMLSelectElement)
    )
    || !activeElement.id
  ) {
    return null;
  }

  return {
    elementId: activeElement.id,
    value: activeElement.value,
    selectionStart:
      activeElement instanceof HTMLSelectElement ? null : activeElement.selectionStart,
    selectionEnd:
      activeElement instanceof HTMLSelectElement ? null : activeElement.selectionEnd,
    selectionDirection:
      activeElement instanceof HTMLSelectElement ? null : activeElement.selectionDirection,
  };
}

function restoreActivePanelControl(
  root: HTMLDivElement,
  controlState: PreservedPanelControlState | null,
) {
  if (!controlState) {
    return;
  }

  const nextElement = document.getElementById(controlState.elementId);
  if (
    !nextElement
    || !root.contains(nextElement)
    || (
      !(nextElement instanceof HTMLInputElement)
      && !(nextElement instanceof HTMLTextAreaElement)
      && !(nextElement instanceof HTMLSelectElement)
    )
  ) {
    return;
  }

  nextElement.focus({ preventScroll: true });
  if (
    nextElement instanceof HTMLInputElement
    || nextElement instanceof HTMLTextAreaElement
  ) {
    if (nextElement.value === controlState.value) {
      nextElement.setSelectionRange(
        controlState.selectionStart,
        controlState.selectionEnd,
        controlState.selectionDirection ?? undefined,
      );
    }
  }
}

export function preserveActivePanelControl(root: HTMLDivElement, renderPanel: () => void) {
  const controlState = captureActivePanelControl(root);
  renderPanel();
  restoreActivePanelControl(root, controlState);
}

export function renderPanelRootNode(
  panelRoots: PanelRootMap,
  rootKey: PanelRootKey,
  node: ReactNode,
) {
  const root = panelRoots[rootKey];
  preserveActivePanelControl(root, () => {
    let panelRoot = mountedPanelRoots.get(root);
    if (!panelRoot) {
      panelRoot = createRoot(root);
      mountedPanelRoots.set(root, panelRoot);
    }

    flushSync(() => {
      panelRoot.render(node);
    });
  });
}
