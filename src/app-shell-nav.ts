import ArrowBackRoundedIcon from "@mui/icons-material/ArrowBackRounded";
import SettingsRoundedIcon from "@mui/icons-material/SettingsRounded";
import { IconButton } from "@mui/material";
import { createElement, type ComponentProps, type ReactNode } from "react";

export type PanelRootKey =
  | "push-to-talk"
  | "url-input"
  | "status"
  | "audio-controls"
  | "settings-guidance"
  | "settings-remote-planner"
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
  | "confirmation-panel"
  | "voice-status";

export type AppView = "workspace" | "settings";
export type SettingsView = "overview" | "planner" | "tts" | "asr" | "runtime";
export type SettingsCardStatus = "ok" | "warning" | "error" | "unconfigured";
export type SettingsStatuses = Partial<Record<Exclude<SettingsView, "overview">, SettingsCardStatus>>;
export type AppShellPanelContent = Partial<Record<PanelRootKey, ReactNode>>;

export interface AppShellNavigationHandlers {
  onAppViewSelect?: (view: AppView) => void;
  onSettingsViewSelect?: (view: SettingsView) => void;
}

const h = createElement;

type DataAttributes = {
  [key: `data-${string}`]: string;
};

type IconButtonWithDataProps = ComponentProps<typeof IconButton> & DataAttributes;

const SETTINGS_STATUS_LABEL: Record<SettingsCardStatus, string> = {
  ok: "Configured",
  warning: "Action needed",
  error: "Error",
  unconfigured: "Not configured",
};

export function renderPanelRootPlaceholderElement(rootKey: PanelRootKey) {
  return h("div", {
    "data-panel-root": rootKey,
  });
}

export function renderPanelContent(rootKey: PanelRootKey, panelContent?: AppShellPanelContent) {
  const content = panelContent?.[rootKey];
  return content !== undefined ? content : renderPanelRootPlaceholderElement(rootKey);
}

export function renderAppViewActionButton(
  initialAppView: AppView,
  handlers?: AppShellNavigationHandlers,
) {
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

export function renderSettingsSubpageBackButton(
  showBackButton: boolean,
  handlers?: AppShellNavigationHandlers,
) {
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

export function renderSettingsSubpageLink(
  view: Exclude<SettingsView, "overview">,
  label: string,
  handlers?: AppShellNavigationHandlers,
  status?: SettingsCardStatus,
) {
  const handleClick = handlers?.onSettingsViewSelect
    ? () => {
      handlers.onSettingsViewSelect?.(view);
    }
    : undefined;

  const ariaLabel = status ? `${label} — ${SETTINGS_STATUS_LABEL[status]}` : label;

  return h(
    "button",
    {
      type: "button",
      className: "settings-subpage-card",
      "data-settings-view-button": view,
      onClick: handleClick,
      "aria-label": ariaLabel,
    },
    h("span", { className: "settings-subpage-card-label" }, label),
    status
      ? h(
        "span",
        {
          className: `settings-subpage-card-status settings-subpage-card-status-${status}`,
          "aria-hidden": "true",
          "data-settings-card-status": view,
        },
        SETTINGS_STATUS_LABEL[status],
      )
      : null,
    h("span", { className: "settings-subpage-card-chevron", "aria-hidden": "true" }, "›"),
  );
}
