import ArrowBackRoundedIcon from "@mui/icons-material/ArrowBackRounded";
import SettingsRoundedIcon from "@mui/icons-material/SettingsRounded";
import { Button, CssBaseline, IconButton } from "@mui/material";
import { StyledEngineProvider, ThemeProvider, createTheme } from "@mui/material/styles";
import { createElement, type ComponentProps, type ReactNode } from "react";

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

type DataAttributes = {
  [key: `data-${string}`]: string;
};

type ButtonWithDataProps = ComponentProps<typeof Button> & DataAttributes;
type IconButtonWithDataProps = ComponentProps<typeof IconButton> & DataAttributes;
export type AppShellPanelContent = Partial<Record<PanelRootKey, ReactNode>>;

export interface AppShellNavigationHandlers {
  onAppViewSelect?: (view: AppView) => void;
  onSettingsViewSelect?: (view: SettingsView) => void;
}

function renderPanelRootPlaceholderElement(rootKey: PanelRootKey) {
  return h("div", {
    "data-panel-root": rootKey,
  });
}

function renderPanelContent(rootKey: PanelRootKey, panelContent?: AppShellPanelContent) {
  const content = panelContent?.[rootKey];
  return content !== undefined ? content : renderPanelRootPlaceholderElement(rootKey);
}

function renderAppViewActionButton(initialAppView: AppView, handlers?: AppShellNavigationHandlers) {
  if (initialAppView === "workspace") {
    const buttonProps: IconButtonWithDataProps = {
      type: "button",
      className: "shell-toolbar-action shell-toolbar-action-settings",
      "data-app-view-button": "settings",
      "aria-label": "Open settings",
      title: "Open settings",
      size: "large",
    };

    if (handlers?.onAppViewSelect) {
      buttonProps.onClick = () => {
        handlers.onAppViewSelect?.("settings");
      };
    }

    return h(
      IconButton,
      buttonProps,
      h(SettingsRoundedIcon, {
        className: "shell-toolbar-action-icon",
        fontSize: "small",
        "aria-hidden": true,
      }),
    );
  }

  const buttonProps: IconButtonWithDataProps = {
    type: "button",
    className: "shell-toolbar-action settings-subpage-back",
    "data-app-view-button": "workspace",
    "aria-label": "Back to workspace",
    title: "Back to workspace",
    size: "large",
  };

  if (handlers?.onAppViewSelect) {
    buttonProps.onClick = () => {
      handlers.onAppViewSelect?.("workspace");
    };
  }

  return h(
    IconButton,
    buttonProps,
    h(ArrowBackRoundedIcon, {
      className: "shell-toolbar-action-icon",
      fontSize: "small",
      "aria-hidden": true,
    }),
  );
}

function renderSettingsSubpageBackButton(showBackButton: boolean, handlers?: AppShellNavigationHandlers) {
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

  if (handlers?.onSettingsViewSelect) {
    buttonProps.onClick = () => {
      handlers.onSettingsViewSelect?.("overview");
    };
  }

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

function renderSettingsSubpageLink(
  view: Exclude<SettingsView, "overview">,
  label: string,
  handlers?: AppShellNavigationHandlers,
) {
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

  if (handlers?.onSettingsViewSelect) {
    buttonProps.onClick = () => {
      handlers.onSettingsViewSelect?.(view);
    };
  }

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
  navigationHandlers?: AppShellNavigationHandlers;
}

export function AppShellMarkup({ initialAppView, initialSettingsView, panelContent, navigationHandlers }: AppShellMarkupProps) {
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
    ),
    h(
      "section",
      {
        className: `app-view${workspaceActive ? " app-view-active" : ""}`,
        "data-app-view-section": "workspace",
        hidden: !workspaceActive,
        "aria-hidden": String(!workspaceActive),
      },
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
          renderSettingsSubpageLink("planner", "Open planner setup", navigationHandlers),
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
          renderSettingsSubpageLink("tts", "Open TTS setup", navigationHandlers),
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
          renderSettingsSubpageLink("asr", "Open ASR setup", navigationHandlers),
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
          renderSettingsSubpageLink("runtime", "Open Runtime setup", navigationHandlers),
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
  navigationHandlers?: AppShellNavigationHandlers,
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
      }),
    ),
  );
}

export function AppShellRuntime(props: {
  appView: AppView;
  settingsView: SettingsView;
  panelContent: AppShellPanelContent;
  navigationHandlers?: AppShellNavigationHandlers;
}) {
  return renderShellTree(props.appView, props.settingsView, props.panelContent, props.navigationHandlers);
}

export async function renderAppShell(): Promise<string> {
  const { renderToStaticMarkup } = await import("react-dom/server");
  return renderToStaticMarkup(renderShellTree("workspace", "overview"));
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
