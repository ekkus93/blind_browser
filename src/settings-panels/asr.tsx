import { type ReactNode } from "react";

import {
  renderProviderModeLabel,
  renderSecretEntryCard,
} from "../confirmation-panel-helpers.ts";
import type {
  AsrProviderPanelState,
  LocalAsrModelPanelState,
  RemoteAsrPanelState,
} from "../panel-types.ts";
import {
  renderReadOnlyCard,
  renderSelectControlCard,
  renderSettingsPanelSection,
} from "./shared-controls.tsx";

export interface AsrProviderPanelHandlers {
  onProviderSelect?: (mode: "Local" | "Remote") => void;
}

export interface RemoteAsrPanelHandlers {
  onApiKeyInput?: (value: string) => void;
  onSaveApiKey?: () => void;
  onTestApiKey?: () => void;
  onOpenExternalLink?: (url: string) => void;
}

export function renderSettingsAsrProviderPanelNode(
  state: AsrProviderPanelState,
  handlers?: AsrProviderPanelHandlers,
): ReactNode {
  return renderSettingsPanelSection({
    titleId: "settings-asr-provider-title",
    title: "Voice input provider",
    description: "Choose the local or remote speech-to-text provider. Changes apply to the next listening request.",
    error: state.error,
    children: (
      <div className="settings-grid">
        {renderSelectControlCard({
          id: "settings-asr-provider-control",
          label: "Provider",
          valueText: renderProviderModeLabel(state.activeMode),
          selectedValue: state.activeMode,
          disabled: state.isBusy,
          dataAttributes: { "data-asr-provider-select": "true" },
          options: state.availableModes.map((mode) => ({ value: mode, label: renderProviderModeLabel(mode) })),
          onChange: handlers?.onProviderSelect
            ? (value) => { handlers.onProviderSelect?.(value as "Local" | "Remote"); }
            : undefined,
        })}
      </div>
    ),
  });
}

export interface LocalAsrModelPanelHandlers {
  onOpenRuntimeSettings?: () => void;
}

export function renderSettingsLocalAsrModelPanelNode(state: LocalAsrModelPanelState, handlers?: LocalAsrModelPanelHandlers): ReactNode {
  return renderSettingsPanelSection({
    titleId: "settings-local-asr-model-title",
    title: "Local voice input profile",
    description: "Review the speech-to-text profile used when voice input runs in local mode. Edit the app config to change it.",
    children: [
      <div className="settings-grid" key="asr-local-grid">
        {renderReadOnlyCard("Profile", state.profileName)}
        {renderReadOnlyCard("Backend", state.backend)}
        {renderReadOnlyCard("Model ID", state.modelId)}
        {renderReadOnlyCard("Model path", state.modelPath)}
        {renderReadOnlyCard("Language", state.language)}
      </div>,
      state.modelAvailable === false
        ? (
          <div className="settings-model-missing-warning" role="alert" key="asr-local-warning">
            <p className="settings-model-missing-message">Model not downloaded yet. Go to Advanced settings to download it.</p>
            <button
              type="button"
              className="settings-model-missing-button"
              data-open-runtime-settings="true"
              onClick={handlers?.onOpenRuntimeSettings}
            >
              Open Advanced settings
            </button>
          </div>
        )
        : null,
    ],
  });
}

export function renderSettingsRemoteAsrPanelNode(
  state: RemoteAsrPanelState,
  handlers?: RemoteAsrPanelHandlers,
): ReactNode {
  return renderSettingsPanelSection({
    titleId: "settings-remote-asr-title",
    title: "Remote voice input profile",
    description: "Review the speech-to-text profile used when voice input runs in remote mode. API keys stay masked here and are stored securely on your device instead of the config file.",
    error: state.error,
    children: (
      <div className="settings-grid">
        {renderReadOnlyCard("Profile", state.profileName)}
        {renderReadOnlyCard("Provider", state.provider)}
        {renderReadOnlyCard("Base URL", state.baseUrl)}
        {renderReadOnlyCard("Model", state.model)}
        {renderReadOnlyCard("API key source", state.apiKeyReference)}
        {renderReadOnlyCard("Organization source", state.organizationReference)}
        {renderReadOnlyCard("Project", state.project)}
        {renderReadOnlyCard("Language", state.language)}
        {renderReadOnlyCard("Creativity", state.temperatureMilli != null ? (state.temperatureMilli / 1000).toFixed(2) : null)}
        {renderReadOnlyCard("Timeout", state.timeoutMs != null ? `${(state.timeoutMs / 1000).toFixed(0)} seconds` : null)}
        {renderSecretEntryCard(
          "asr",
          state.profileName,
          state.apiKeyDraft,
          state.apiKeyMaskedValue,
          state.isSavingApiKey,
          state.isTestingApiKey,
          state.apiKeyReference !== null,
          state.apiKeyTestMessage,
          {
            onInput: handlers?.onApiKeyInput,
            onSave: handlers?.onSaveApiKey,
            onTest: handlers?.onTestApiKey,
            onOpenExternalLink: handlers?.onOpenExternalLink,
          },
        )}
      </div>
    ),
  });
}
