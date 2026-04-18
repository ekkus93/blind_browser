import { createElement, type ReactNode } from "react";

import {
  renderConfirmationThresholdValue,
  renderFailoverAvailabilityLabel,
  renderModelAvailabilityLabel,
  renderOcrThresholdValue,
  renderTextWithKnownLinkNodes,
} from "../confirmation-panel-helpers.ts";
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
} from "./shared-controls.ts";

const h = createElement;

function renderConfirmationThresholdValueText(value: number): string {
  return `${Math.round(value * 100)} percent confidence`;
}

export function renderSettingsProviderFailoverPanelNode(state: ProviderFailoverPanelState): ReactNode {
  const renderFailoverCard = (
    providerKey: "planner" | "tts" | "asr",
    providerLabel: string,
    available: boolean,
  ) => h(
    "label",
    {
      className: "settings-control-card",
      htmlFor: `settings-provider-failover-${providerKey}`,
      key: providerKey,
    },
    h("span", { className: "settings-control-label" }, providerLabel),
    h("span", { className: "settings-control-value" }, renderFailoverAvailabilityLabel(available)),
    h("input", {
      id: `settings-provider-failover-${providerKey}`,
      className: "settings-control-input",
      "data-provider-failover-toggle": providerKey,
      type: "checkbox",
      disabled: true,
      "aria-disabled": "true",
      readOnly: true,
    }),
  );

  return renderSettingsPanelSection({
    titleId: "settings-provider-failover-title",
    title: "Failover",
    description: "Remote-to-local failover is not available yet. These toggles stay read-only until it is.",
    children: h(
      "div",
      { className: "settings-grid" },
      renderFailoverCard("planner", "Planner", state.plannerAvailable),
      renderFailoverCard("tts", "TTS", state.ttsAvailable),
      renderFailoverCard("asr", "ASR", state.asrAvailable),
    ),
    error: null,
  });
}

export function renderSettingsConfirmationPanelNode(state: ConfirmationSettingsPanelState): ReactNode {
  return renderSettingsPanelSection({
    titleId: "settings-confirmation-title",
    title: "Confirmation",
    description: "Choose how confident a click must be before the app asks for confirmation. Form submits still always require confirmation.",
    error: state.error,
    children: h(
      "div",
      { className: "settings-grid" },
      h(
        "label",
        { className: "settings-control-card", htmlFor: "settings-confirmation-threshold-control" },
        h("span", { className: "settings-control-label" }, "Click threshold"),
        h(
          "span",
          { className: "settings-control-value" },
          renderConfirmationThresholdValue(state.confirmationConfidenceThreshold),
        ),
        h("input", {
          id: "settings-confirmation-threshold-control",
          className: "settings-control-input",
          "data-confirmation-threshold-control": "true",
          type: "range",
          min: "0",
          max: "1",
          step: "0.01",
          value: state.confirmationConfidenceThreshold.toFixed(2),
          "aria-valuetext": renderConfirmationThresholdValueText(state.confirmationConfidenceThreshold),
          disabled: state.isBusy || undefined,
          "aria-disabled": state.isBusy ? "true" : undefined,
          readOnly: true,
        }),
      ),
      renderCheckboxControlCard({
        id: "settings-click-without-confirmation-toggle",
        label: "Skip confirmation for confident clicks",
        valueText: state.allowClickWithoutConfirmation ? "Enabled" : "Disabled",
        checked: state.allowClickWithoutConfirmation,
        disabled: state.isBusy,
        dataAttributes: { "data-click-without-confirmation-toggle": "true" },
      }),
      h(
        "div",
        { className: "settings-control-card", "aria-live": "polite" },
        h("span", { className: "settings-control-label" }, "Submit actions"),
        h(
          "span",
          { className: "settings-control-value" },
          state.alwaysConfirmSubmit ? "Always require confirmation" : "Confirmation not required",
        ),
      ),
    ),
  });
}

export function renderSettingsOcrThresholdPanelNode(state: OcrThresholdSettingsPanelState): ReactNode {
  return renderSettingsPanelSection({
    titleId: "settings-ocr-thresholds-title",
    title: "OCR fallback",
    description: "Choose when sparse DOM extraction should fall back to OCR.",
    error: state.error,
    children: h(
      "div",
      { className: "settings-grid" },
      h(
        "label",
        { className: "settings-control-card", htmlFor: "settings-ocr-char-threshold-control" },
        h("span", { className: "settings-control-label" }, "Character threshold"),
        h("span", { className: "settings-control-value" }, renderOcrThresholdValue(state.sparseTextCharThreshold)),
        h("input", {
          id: "settings-ocr-char-threshold-control",
          className: "settings-control-input",
          "data-ocr-threshold-control": "char",
          type: "number",
          min: "1",
          step: "1",
          value: `${state.sparseTextCharThreshold}`,
          disabled: state.isBusy || undefined,
          "aria-disabled": state.isBusy ? "true" : undefined,
          readOnly: true,
        }),
      ),
      h(
        "label",
        { className: "settings-control-card", htmlFor: "settings-ocr-region-threshold-control" },
        h("span", { className: "settings-control-label" }, "Region threshold"),
        h("span", { className: "settings-control-value" }, renderOcrThresholdValue(state.sparseTextRegionThreshold)),
        h("input", {
          id: "settings-ocr-region-threshold-control",
          className: "settings-control-input",
          "data-ocr-threshold-control": "region",
          type: "number",
          min: "1",
          step: "1",
          value: `${state.sparseTextRegionThreshold}`,
          disabled: state.isBusy || undefined,
          "aria-disabled": state.isBusy ? "true" : undefined,
          readOnly: true,
        }),
      ),
    ),
  });
}

export function renderSettingsGuidancePanelNode(state: SettingsGuidancePanelState | null): ReactNode {
  if (!state) {
    return null;
  }

  return renderSettingsPanelSection({
    titleId: "settings-guidance-title",
    title: state.title,
    description: renderTextWithKnownLinkNodes(state.message),
    eyebrow: "Guidance",
    children: h(
      "div",
      { className: "url-input-actions" },
      ...state.actions.map((action) => h(
        "button",
        {
          type: "button",
          className: "url-open-button",
          "data-settings-target": action.targetId,
          key: action.targetId,
        },
        action.label,
      )),
    ),
  });
}

export function renderSettingsModelManagementPanelNode(state: ModelManagementPanelState): ReactNode {
  const ttsDownloadDisabled = state.isDownloadingTts || !state.localTtsDownloadSupported;
  const asrDownloadDisabled = state.isDownloadingAsr || !state.localAsrDownloadSupported;

  return renderSettingsPanelSection({
    titleId: "settings-model-management-title",
    title: "Local models",
    description: "Choose where local speech models live, whether startup checks them, and whether missing models download automatically.",
    error: state.error,
    children: h(
      "div",
      { className: "settings-grid" },
      h(
        "label",
        { className: "settings-control-card", htmlFor: "settings-models-dir-input" },
        h("span", { className: "settings-control-label" }, "Model folder"),
        h("span", { className: "settings-control-value" }, state.modelsDir || "Not configured"),
        h("input", {
          id: "settings-models-dir-input",
          className: "settings-control-select",
          "data-model-management-input": "models-dir",
          type: "text",
          value: state.modelsDir,
          placeholder: "~/.local/share/blind_browser/models",
          spellCheck: false,
          "aria-describedby": "settings-models-dir-description",
          disabled: state.isSaving || undefined,
          "aria-disabled": state.isSaving ? "true" : undefined,
          readOnly: true,
        }),
        h(
          "span",
          { id: "settings-models-dir-description", className: "settings-panel-description" },
          "Updates here change where downloads and startup checks look for speech models.",
        ),
      ),
      renderCheckboxControlCard({
        id: "settings-model-check-on-startup-toggle",
        label: "Check on startup",
        valueText: state.checkOnStartup ? "Enabled" : "Disabled",
        checked: state.checkOnStartup,
        disabled: state.isSaving,
        dataAttributes: { "data-model-management-toggle": "check-on-startup" },
      }),
      renderCheckboxControlCard({
        id: "settings-model-auto-download-toggle",
        label: "Auto-download missing",
        valueText: state.autoDownloadMissing ? "Enabled" : "Disabled",
        checked: state.autoDownloadMissing,
        disabled: state.isSaving,
        dataAttributes: { "data-model-management-toggle": "auto-download-missing" },
      }),
      h(
        "div",
        { className: "settings-control-card" },
        h("span", { className: "settings-control-label" }, "Local TTS"),
        h("span", { className: "settings-control-value" }, renderModelAvailabilityLabel(state.localTtsAvailable)),
        h(
          "button",
          {
            type: "button",
            className: "settings-control-button",
            "data-model-download": "tts",
            disabled: ttsDownloadDisabled || undefined,
            "aria-disabled": ttsDownloadDisabled ? "true" : undefined,
          },
          state.isDownloadingTts ? "Downloading..." : (state.localTtsDownloadLabel ?? "Download unavailable"),
        ),
      ),
      h(
        "div",
        { className: "settings-control-card" },
        h("span", { className: "settings-control-label" }, "Local ASR"),
        h("span", { className: "settings-control-value" }, renderModelAvailabilityLabel(state.localAsrAvailable)),
        h(
          "button",
          {
            type: "button",
            className: "settings-control-button",
            "data-model-download": "asr",
            disabled: asrDownloadDisabled || undefined,
            "aria-disabled": asrDownloadDisabled ? "true" : undefined,
          },
          state.isDownloadingAsr ? "Downloading..." : (state.localAsrDownloadLabel ?? "Download unavailable"),
        ),
      ),
    ),
  });
}