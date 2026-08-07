import { type ReactNode } from "react";

import {
  renderConfirmationThresholdValue,
  renderFailoverAvailabilityLabel,
  renderModelAvailabilityLabel,
  renderOcrThresholdValue,
  renderTextWithKnownLinkNodes,
} from "../confirmation-panel-helpers.tsx";
import type {
  ConfirmationSettingsPanelState,
  ModelManagementPanelState,
  OcrThresholdSettingsPanelState,
  ProviderFailoverPanelState,
  SettingsGuidancePanelState,
} from "../panel-types.ts";
import {
  BTN_SPINNER_CLASS,
  CONTROL_CARD,
  CONTROL_LABEL,
  CONTROL_VALUE,
  SETTINGS_CONTROL_BUTTON_CLASS,
  SETTINGS_CONTROL_INPUT_CLASS,
  SETTINGS_CONTROL_SELECT_CLASS,
  SETTINGS_GRID_CLASS,
  SETTINGS_PANEL_DESCRIPTION_CLASS,
  renderCheckboxControlCard,
  renderSettingsPanelSection,
} from "./shared-controls.tsx";

export interface SettingsGuidancePanelHandlers {
  onSelectTarget?: (targetId: string) => void;
  onOpenExternalLink?: (url: string) => void;
}

export interface ConfirmationSettingsPanelHandlers {
  onThresholdChange?: (value: number) => void;
  onClickWithoutConfirmationChange?: (checked: boolean) => void;
  onDismissError?: () => void;
}

export interface OcrThresholdPanelHandlers {
  onCharThresholdChange?: (value: number) => void;
  onRegionThresholdChange?: (value: number) => void;
  onDismissError?: () => void;
}

export interface ModelManagementPanelHandlers {
  onModelsDirInput?: (value: string) => void;
  onPersistModelsDir?: () => void;
  onCheckOnStartupChange?: (checked: boolean) => void;
  onAutoDownloadMissingChange?: (checked: boolean) => void;
  onDownloadModel?: (kind: "tts" | "asr") => void;
  onDismissError?: () => void;
  onRetry?: () => void;
}

function renderConfirmationThresholdValueText(value: number): string {
  return `${Math.round(value * 100)} percent confidence`;
}

// `.url-open-button` alone (without the `.url-action-button` base it's paired
// with in workspace.tsx's icon buttons) only ever contributed its gradient
// background and shadow in the old CSS — no dimensions, no focus outline.
// These text-label guidance buttons faithfully inherit just that: no custom
// focus ring override, matching the original scoped rule.
//
// CR3 P3.3.2: that inherited-from-the-old-CSS anatomy never set a text
// color either, so the label fell back to inherited body text
// (`--color-text-primary`, near-black) against this dark-green gradient —
// well below 4.5:1 on what is the primary remediation CTA. Added
// `text-[#fffdf8]` (the same near-white `shared-controls.tsx`'s
// `SETTINGS_CONTROL_BUTTON_DANGER_CLASS` already uses). Computed against
// the *lighter* end of this gradient (`--color-green-active` #1f7f5c, the
// worse case -- its higher luminance than `--color-green-primary` #29583f
// means less contrast, not more) via the WCAG relative-luminance formula:
// ~4.93:1. `shared-controls.tsx`'s slightly warmer `#f6f2eb` was
// considered first (it's the established convention for
// `SETTINGS_CONTROL_BUTTON_CLASS`) but only clears ~4.43:1 here -- under
// 4.5:1 -- because that convention was tuned against a different, lighter
// gradient end color (`#347f55`), not this one; `#fffdf8` clears the
// target with real margin instead of copying a convention that happens to
// fall just short against this particular background.
// Exported so tailwind-cascade.test.mjs can compile this exact string and
// pin that a `color` declaration is actually present, guarding against the
// contrast bug (a missing `text-*` utility) recurring silently.
export const GUIDANCE_ACTION_BUTTON_CLASS = "bg-gradient-to-br from-[var(--color-green-primary)] to-[var(--color-green-active)] text-[#fffdf8] shadow-[0_12px_24px_rgba(31,127,92,0.18)] enabled:hover:shadow-[0_16px_28px_rgba(31,127,92,0.24)] focus-visible:shadow-[0_16px_28px_rgba(31,127,92,0.24)]";

// Not wired into the app shell until the backend failover feature ships.
// Re-add "settings-provider-failover" to PanelRootKey in app-shell-nav.tsx and wire
// the panel in main.ts once automatic failover is implemented in the Rust runtime.
export function renderSettingsProviderFailoverPanelNode(state: ProviderFailoverPanelState): ReactNode {
  const renderFailoverCard = (
    providerKey: "planner" | "tts" | "asr",
    providerLabel: string,
    available: boolean,
  ) => (
    <label
      className={CONTROL_CARD}
      htmlFor={`settings-provider-failover-${providerKey}`}
      key={providerKey}
    >
      <span className={CONTROL_LABEL}>{providerLabel}</span>
      <span className={CONTROL_VALUE}>{renderFailoverAvailabilityLabel(available)}</span>
      <input
        id={`settings-provider-failover-${providerKey}`}
        className={SETTINGS_CONTROL_INPUT_CLASS}
        data-provider-failover-toggle={providerKey}
        type="checkbox"
        disabled={true}
        aria-disabled="true"
        readOnly={true}
      />
    </label>
  );

  return renderSettingsPanelSection({
    titleId: "settings-provider-failover-title",
    title: "Failover",
    description: "Remote-to-local failover is not available yet. These toggles stay read-only until it is.",
    children: (
      <div className={SETTINGS_GRID_CLASS}>
        {renderFailoverCard("planner", "Planner", state.plannerAvailable)}
        {renderFailoverCard("tts", "TTS", state.ttsAvailable)}
        {renderFailoverCard("asr", "ASR", state.asrAvailable)}
      </div>
    ),
    error: null,
  });
}

export function renderSettingsConfirmationPanelNode(
  state: ConfirmationSettingsPanelState,
  handlers?: ConfirmationSettingsPanelHandlers,
): ReactNode {
  return renderSettingsPanelSection({
    titleId: "settings-confirmation-title",
    title: "Action confirmation",
    description: "Choose how confident a click must be before the app asks for confirmation. Form submits still always require confirmation.",
    error: state.error,
    onDismissError: handlers?.onDismissError,
    children: (
      <div className={SETTINGS_GRID_CLASS}>
        <label className={CONTROL_CARD} htmlFor="settings-confirmation-threshold-control">
          <span className={CONTROL_LABEL}>Click threshold</span>
          <span className={CONTROL_VALUE}>
            {renderConfirmationThresholdValue(state.confirmationConfidenceThreshold)}
          </span>
          <input
            id="settings-confirmation-threshold-control"
            className={SETTINGS_CONTROL_INPUT_CLASS}
            data-confirmation-threshold-control="true"
            type="range"
            min="0"
            max="1"
            step="0.01"
            value={state.confirmationConfidenceThreshold.toFixed(2)}
            aria-valuetext={renderConfirmationThresholdValueText(state.confirmationConfidenceThreshold)}
            disabled={state.isBusy || undefined}
            aria-disabled={state.isBusy ? "true" : undefined}
            onChange={handlers?.onThresholdChange
              ? (event) => { handlers.onThresholdChange?.(Number.parseFloat(event.currentTarget.value)); }
              : undefined}
          />
        </label>
        {renderCheckboxControlCard({
          id: "settings-click-without-confirmation-toggle",
          label: "Skip confirmation for confident clicks",
          valueText: state.allowClickWithoutConfirmation ? "Enabled" : "Disabled",
          checked: state.allowClickWithoutConfirmation,
          disabled: state.isBusy,
          dataAttributes: { "data-click-without-confirmation-toggle": "true" },
          onChange: handlers?.onClickWithoutConfirmationChange,
        })}
        <div className={CONTROL_CARD} aria-live="polite">
          <span className={CONTROL_LABEL}>Submit actions</span>
          <span className={CONTROL_VALUE}>
            {state.alwaysConfirmSubmit ? "Always require confirmation" : "Confirmation not required"}
          </span>
        </div>
      </div>
    ),
  });
}

export function renderSettingsOcrThresholdPanelNode(
  state: OcrThresholdSettingsPanelState,
  handlers?: OcrThresholdPanelHandlers,
): ReactNode {
  return renderSettingsPanelSection({
    titleId: "settings-ocr-thresholds-title",
    title: "Screen reading fallback",
    description: "Choose when sparse DOM extraction should fall back to image text recognition.",
    error: state.error,
    onDismissError: handlers?.onDismissError,
    children: (
      <div className={SETTINGS_GRID_CLASS}>
        <label className={CONTROL_CARD} htmlFor="settings-ocr-char-threshold-control">
          <span className={CONTROL_LABEL}>Character threshold</span>
          <span className={CONTROL_VALUE}>{renderOcrThresholdValue(state.sparseTextCharThreshold)}</span>
          <input
            id="settings-ocr-char-threshold-control"
            className={SETTINGS_CONTROL_INPUT_CLASS}
            data-ocr-threshold-control="char"
            type="number"
            min="1"
            step="1"
            value={`${state.sparseTextCharThreshold}`}
            disabled={state.isBusy || undefined}
            aria-disabled={state.isBusy ? "true" : undefined}
            onChange={handlers?.onCharThresholdChange
              ? (event) => { handlers.onCharThresholdChange?.(Number.parseInt(event.currentTarget.value, 10)); }
              : undefined}
          />
        </label>
        <label className={CONTROL_CARD} htmlFor="settings-ocr-region-threshold-control">
          <span className={CONTROL_LABEL}>Region threshold</span>
          <span className={CONTROL_VALUE}>{renderOcrThresholdValue(state.sparseTextRegionThreshold)}</span>
          <input
            id="settings-ocr-region-threshold-control"
            className={SETTINGS_CONTROL_INPUT_CLASS}
            data-ocr-threshold-control="region"
            type="number"
            min="1"
            step="1"
            value={`${state.sparseTextRegionThreshold}`}
            disabled={state.isBusy || undefined}
            aria-disabled={state.isBusy ? "true" : undefined}
            onChange={handlers?.onRegionThresholdChange
              ? (event) => { handlers.onRegionThresholdChange?.(Number.parseInt(event.currentTarget.value, 10)); }
              : undefined}
          />
        </label>
      </div>
    ),
  });
}

export function renderSettingsGuidancePanelNode(
  state: SettingsGuidancePanelState | null,
  handlers?: SettingsGuidancePanelHandlers,
): ReactNode {
  if (!state) {
    return null;
  }

  return renderSettingsPanelSection({
    titleId: "settings-guidance-title",
    title: state.title,
    description: renderTextWithKnownLinkNodes(state.message, handlers?.onOpenExternalLink),
    eyebrow: "Guidance",
    children: (
      <div className="grid gap-3">
        {state.actions.map((action) => (
          <button
            key={action.targetId}
            type="button"
            className={GUIDANCE_ACTION_BUTTON_CLASS}
            data-settings-target={action.targetId}
            onClick={handlers?.onSelectTarget ? () => { handlers.onSelectTarget?.(action.targetId); } : undefined}
          >
            {action.label}
          </button>
        ))}
      </div>
    ),
  });
}

export function renderSettingsModelManagementPanelNode(
  state: ModelManagementPanelState,
  handlers?: ModelManagementPanelHandlers,
): ReactNode {
  const ttsDownloadDisabled = state.isDownloadingTts || !state.localTtsDownloadSupported;
  const asrDownloadDisabled = state.isDownloadingAsr || !state.localAsrDownloadSupported;

  return renderSettingsPanelSection({
    titleId: "settings-model-management-title",
    title: "Local models",
    description: "Choose where local speech models live, whether startup checks them, and whether missing models download automatically.",
    error: state.error,
    onDismissError: handlers?.onDismissError,
    onRetry: handlers?.onRetry,
    children: (
      <div className={SETTINGS_GRID_CLASS}>
        <label className={CONTROL_CARD} htmlFor="settings-models-dir-input">
          <span className={CONTROL_LABEL}>Model folder</span>
          <span className={CONTROL_VALUE}>{state.modelsDir || "Not configured"}</span>
          <input
            id="settings-models-dir-input"
            className={SETTINGS_CONTROL_SELECT_CLASS}
            data-model-management-input="models-dir"
            type="text"
            value={state.modelsDir}
            placeholder="~/.local/share/blind_browser/models"
            spellCheck={false}
            aria-describedby="settings-models-dir-description"
            disabled={state.isSaving || undefined}
            aria-disabled={state.isSaving ? "true" : undefined}
            onChange={handlers?.onModelsDirInput
              ? (event) => { handlers.onModelsDirInput?.(event.currentTarget.value); }
              : undefined}
            onBlur={handlers?.onPersistModelsDir}
          />
          <span id="settings-models-dir-description" className={SETTINGS_PANEL_DESCRIPTION_CLASS}>
            Updates here change where downloads and startup checks look for speech models.
          </span>
        </label>
        {renderCheckboxControlCard({
          id: "settings-model-check-on-startup-toggle",
          label: "Check on startup",
          valueText: state.checkOnStartup ? "Enabled" : "Disabled",
          checked: state.checkOnStartup,
          disabled: state.isSaving,
          dataAttributes: { "data-model-management-toggle": "check-on-startup" },
          onChange: handlers?.onCheckOnStartupChange,
        })}
        {renderCheckboxControlCard({
          id: "settings-model-auto-download-toggle",
          label: "Auto-download missing",
          valueText: state.autoDownloadMissing ? "Enabled" : "Disabled",
          checked: state.autoDownloadMissing,
          disabled: state.isSaving,
          dataAttributes: { "data-model-management-toggle": "auto-download-missing" },
          onChange: handlers?.onAutoDownloadMissingChange,
        })}
        <div className={CONTROL_CARD}>
          <span className={CONTROL_LABEL}>Local TTS</span>
          <span className={CONTROL_VALUE}>{renderModelAvailabilityLabel(state.localTtsAvailable)}</span>
          <button
            type="button"
            className={SETTINGS_CONTROL_BUTTON_CLASS}
            data-model-download="tts"
            disabled={ttsDownloadDisabled || undefined}
            aria-disabled={ttsDownloadDisabled ? "true" : undefined}
            onClick={handlers?.onDownloadModel ? () => { handlers.onDownloadModel?.("tts"); } : undefined}
          >
            {state.isDownloadingTts
              ? <><span className={BTN_SPINNER_CLASS} data-btn-spinner="true" aria-hidden="true" />Downloading...</>
              : (state.localTtsDownloadLabel ?? "Download unavailable")}
          </button>
        </div>
        <div className={CONTROL_CARD}>
          <span className={CONTROL_LABEL}>Local ASR</span>
          <span className={CONTROL_VALUE}>{renderModelAvailabilityLabel(state.localAsrAvailable)}</span>
          <button
            type="button"
            className={SETTINGS_CONTROL_BUTTON_CLASS}
            data-model-download="asr"
            disabled={asrDownloadDisabled || undefined}
            aria-disabled={asrDownloadDisabled ? "true" : undefined}
            onClick={handlers?.onDownloadModel ? () => { handlers.onDownloadModel?.("asr"); } : undefined}
          >
            {state.isDownloadingAsr
              ? <><span className={BTN_SPINNER_CLASS} data-btn-spinner="true" aria-hidden="true" />Downloading...</>
              : (state.localAsrDownloadLabel ?? "Download unavailable")}
          </button>
        </div>
      </div>
    ),
  });
}
