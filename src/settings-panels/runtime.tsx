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
      className="settings-control-card"
      htmlFor={`settings-provider-failover-${providerKey}`}
      key={providerKey}
    >
      <span className="settings-control-label">{providerLabel}</span>
      <span className="settings-control-value">{renderFailoverAvailabilityLabel(available)}</span>
      <input
        id={`settings-provider-failover-${providerKey}`}
        className="settings-control-input"
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
      <div className="settings-grid">
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
      <div className="settings-grid">
        <label className="settings-control-card" htmlFor="settings-confirmation-threshold-control">
          <span className="settings-control-label">Click threshold</span>
          <span className="settings-control-value">
            {renderConfirmationThresholdValue(state.confirmationConfidenceThreshold)}
          </span>
          <input
            id="settings-confirmation-threshold-control"
            className="settings-control-input"
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
        <div className="settings-control-card" aria-live="polite">
          <span className="settings-control-label">Submit actions</span>
          <span className="settings-control-value">
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
      <div className="settings-grid">
        <label className="settings-control-card" htmlFor="settings-ocr-char-threshold-control">
          <span className="settings-control-label">Character threshold</span>
          <span className="settings-control-value">{renderOcrThresholdValue(state.sparseTextCharThreshold)}</span>
          <input
            id="settings-ocr-char-threshold-control"
            className="settings-control-input"
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
        <label className="settings-control-card" htmlFor="settings-ocr-region-threshold-control">
          <span className="settings-control-label">Region threshold</span>
          <span className="settings-control-value">{renderOcrThresholdValue(state.sparseTextRegionThreshold)}</span>
          <input
            id="settings-ocr-region-threshold-control"
            className="settings-control-input"
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
      <div className="url-input-actions">
        {state.actions.map((action) => (
          <button
            key={action.targetId}
            type="button"
            className="url-open-button"
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
      <div className="settings-grid">
        <label className="settings-control-card" htmlFor="settings-models-dir-input">
          <span className="settings-control-label">Model folder</span>
          <span className="settings-control-value">{state.modelsDir || "Not configured"}</span>
          <input
            id="settings-models-dir-input"
            className="settings-control-select"
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
          <span id="settings-models-dir-description" className="settings-panel-description">
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
        <div className="settings-control-card">
          <span className="settings-control-label">Local TTS</span>
          <span className="settings-control-value">{renderModelAvailabilityLabel(state.localTtsAvailable)}</span>
          <button
            type="button"
            className="settings-control-button"
            data-model-download="tts"
            disabled={ttsDownloadDisabled || undefined}
            aria-disabled={ttsDownloadDisabled ? "true" : undefined}
            onClick={handlers?.onDownloadModel ? () => { handlers.onDownloadModel?.("tts"); } : undefined}
          >
            {state.isDownloadingTts
              ? <><span className="btn-spinner" aria-hidden="true" />Downloading...</>
              : (state.localTtsDownloadLabel ?? "Download unavailable")}
          </button>
        </div>
        <div className="settings-control-card">
          <span className="settings-control-label">Local ASR</span>
          <span className="settings-control-value">{renderModelAvailabilityLabel(state.localAsrAvailable)}</span>
          <button
            type="button"
            className="settings-control-button"
            data-model-download="asr"
            disabled={asrDownloadDisabled || undefined}
            aria-disabled={asrDownloadDisabled ? "true" : undefined}
            onClick={handlers?.onDownloadModel ? () => { handlers.onDownloadModel?.("asr"); } : undefined}
          >
            {state.isDownloadingAsr
              ? <><span className="btn-spinner" aria-hidden="true" />Downloading...</>
              : (state.localAsrDownloadLabel ?? "Download unavailable")}
          </button>
        </div>
      </div>
    ),
  });
}
