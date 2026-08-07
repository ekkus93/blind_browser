import { type ReactNode } from "react";

import { ArrowBackIcon, SettingsIcon } from "./icons.tsx";

export type PanelRootKey =
  | "app-alert"
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

const SETTINGS_STATUS_LABEL: Record<SettingsCardStatus, string> = {
  ok: "Configured",
  warning: "Action needed",
  error: "Error",
  unconfigured: "Setup required",
};

const SETTINGS_STATUS_BADGE_CLASS: Record<SettingsCardStatus, string> = {
  ok: "bg-[rgba(41,88,63,0.12)] text-[var(--color-green-dark)]",
  warning: "bg-[rgba(122,87,39,0.14)] text-[var(--color-amber-active)]",
  error: "bg-[rgba(139,52,42,0.14)] text-[var(--color-error-primary)]",
  unconfigured: "bg-[rgba(67,61,55,0.1)] text-[var(--color-text-secondary)]",
};

const FOCUS_RING = "focus-visible:[outline:var(--focus-ring)] focus-visible:[outline-offset:var(--focus-offset)]";

// Shared anatomy of the round 44x44 toolbar buttons (both the settings
// gear/back toggle and the settings-subpage back arrow used the identical
// `.shell-toolbar-action` / `.settings-subpage-back` rule bodies in the old CSS).
const TOOLBAR_ACTION_BUTTON_CLASS = `inline-flex items-center justify-center shrink-0 w-11 h-11 p-0 rounded-full bg-[var(--color-surface-card)] border border-[rgba(41,88,63,0.16)] shadow-[0_10px_20px_rgba(49,63,74,0.08)] hover:-translate-y-px hover:shadow-[0_14px_24px_rgba(31,127,92,0.14)] focus-visible:-translate-y-px focus-visible:shadow-[0_14px_24px_rgba(31,127,92,0.14)] ${FOCUS_RING}`;

export function renderPanelRootPlaceholderElement(rootKey: PanelRootKey) {
  return <div data-panel-root={rootKey} />;
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
    return (
      <button
        type="button"
        className={`${TOOLBAR_ACTION_BUTTON_CLASS} text-[var(--color-green-primary)]`}
        data-app-view-button="settings"
        aria-label="Open settings"
        title="Open settings"
        onClick={handlers?.onAppViewSelect ? () => { handlers.onAppViewSelect?.("settings"); } : undefined}
      >
        <SettingsIcon className="block" />
      </button>
    );
  }

  return (
    <button
      type="button"
      className={TOOLBAR_ACTION_BUTTON_CLASS}
      data-app-view-button="workspace"
      aria-label="Back to workspace"
      title="Back to workspace"
      onClick={handlers?.onAppViewSelect ? () => { handlers.onAppViewSelect?.("workspace"); } : undefined}
    >
      <ArrowBackIcon className="block" />
    </button>
  );
}

export function renderSettingsSubpageBackButton(
  showBackButton: boolean,
  handlers?: AppShellNavigationHandlers,
) {
  return (
    <button
      type="button"
      className={TOOLBAR_ACTION_BUTTON_CLASS}
      data-settings-subpage-back="true"
      data-settings-view-button="overview"
      aria-label="Back to settings"
      title="Back to settings"
      hidden={!showBackButton}
      aria-hidden={!showBackButton}
      onClick={handlers?.onSettingsViewSelect ? () => { handlers.onSettingsViewSelect?.("overview"); } : undefined}
    >
      <ArrowBackIcon className="block" />
    </button>
  );
}

export function renderSettingsSubpageLink(
  view: Exclude<SettingsView, "overview">,
  label: string,
  handlers?: AppShellNavigationHandlers,
  status?: SettingsCardStatus,
) {
  const handleClick = handlers?.onSettingsViewSelect
    ? () => { handlers.onSettingsViewSelect?.(view); }
    : undefined;

  const ariaLabel = status ? `${label} — ${SETTINGS_STATUS_LABEL[status]}` : label;

  return (
    <button
      type="button"
      className={`flex items-center justify-between w-full p-[12px_16px] appearance-none border border-[var(--card-border)] rounded-[10px] bg-[var(--color-surface-card)] text-[var(--color-green-primary)] [font:inherit] font-bold cursor-pointer text-left [transition:background_150ms_ease,box-shadow_150ms_ease,transform_120ms_ease] shadow-[0_2px_8px_rgba(49,63,74,0.06)] hover:-translate-y-px hover:shadow-[0_6px_16px_rgba(41,88,63,0.12)] focus-visible:-translate-y-px focus-visible:shadow-[0_6px_16px_rgba(41,88,63,0.12)] ${FOCUS_RING} max-sm:p-[10px_14px]`}
      data-settings-view-button={view}
      onClick={handleClick}
      aria-label={ariaLabel}
    >
      <span className="flex-1">{label}</span>
      {status ? (
        <span
          className={`text-[0.75rem] font-semibold py-[2px] px-2 rounded-[100px] whitespace-nowrap mr-1 ${SETTINGS_STATUS_BADGE_CLASS[status]}`}
          aria-hidden="true"
          data-settings-card-status={view}
          data-settings-card-status-kind={status}
        >
          {SETTINGS_STATUS_LABEL[status]}
        </span>
      ) : null}
      <span className="text-[1.4rem] leading-none ml-[10px] text-[var(--color-green-primary)] opacity-70" aria-hidden="true">›</span>
    </button>
  );
}
