import { type ReactNode } from "react";

import {
  renderProviderModeLabel,
  renderSecretEntryCard,
  renderTtsModelOptionLabel,
  renderTtsVoiceOptionLabel,
} from "../confirmation-panel-helpers.ts";
import type {
  LocalTtsModelPanelState,
  RemoteTtsPanelState,
  TtsModelPanelState,
  TtsProviderPanelState,
  TtsVoicePanelState,
} from "../panel-types.ts";
import {
  renderReadOnlyCard,
  renderSelectControlCard,
  renderSettingsPanelSection,
} from "./shared-controls.tsx";

export interface TtsProviderPanelHandlers {
  onProviderSelect?: (mode: "Local" | "Remote") => void;
  onDismissError?: () => void;
}

export interface TtsModelPanelHandlers {
  onModelSelect?: (profileName: string) => void;
  onDismissError?: () => void;
}

export interface TtsVoicePanelHandlers {
  onVoiceSelect?: (voice: string) => void;
  onDismissError?: () => void;
}

export interface RemoteTtsPanelHandlers {
  onApiKeyInput?: (value: string) => void;
  onSaveApiKey?: () => void;
  onTestApiKey?: () => void;
  onOpenExternalLink?: (url: string) => void;
  onDismissError?: () => void;
  onRetry?: () => void;
}

export interface LocalTtsModelPanelHandlers {
  onOpenRuntimeSettings?: () => void;
}

export function renderSettingsLocalTtsModelPanelNode(state: LocalTtsModelPanelState, handlers?: LocalTtsModelPanelHandlers): ReactNode {
  return renderSettingsPanelSection({
    titleId: "settings-local-tts-model-title",
    title: "Local voice output profile",
    description: "Review the local speech profile used when voice output runs in local mode. Edit the app config to change it.",
    children: [
      <div className="settings-grid" key="tts-local-grid">
        {renderReadOnlyCard("Profile", state.profileName)}
        {renderReadOnlyCard("Backend", state.backend)}
        {renderReadOnlyCard("Model ID", state.modelId)}
        {renderReadOnlyCard("Model path", state.modelPath)}
        {renderReadOnlyCard("Default voice", state.defaultVoice)}
        {renderReadOnlyCard("Sample rate", state.sampleRate)}
      </div>,
      state.modelAvailable === false
        ? (
          <div className="settings-model-missing-warning" role="alert" key="tts-local-warning">
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

export function renderSettingsRemoteTtsPanelNode(
  state: RemoteTtsPanelState,
  handlers?: RemoteTtsPanelHandlers,
): ReactNode {
  return renderSettingsPanelSection({
    titleId: "settings-remote-tts-title",
    title: "Remote voice output profile",
    description: "Review the speech profile used when voice output runs in remote mode. API keys stay masked here and are stored securely on your device instead of the config file.",
    error: state.error,
    onDismissError: handlers?.onDismissError,
    onRetry: handlers?.onRetry,
    children: (
      <div className="settings-grid">
        {renderReadOnlyCard("Profile", state.profileName)}
        {renderReadOnlyCard("Provider", state.provider)}
        {renderReadOnlyCard("Base URL", state.baseUrl)}
        {renderReadOnlyCard("Model", state.model)}
        {renderReadOnlyCard("API key source", state.apiKeyReference)}
        {state.apiKeyReferenceError ? (
          <p className="settings-panel-description settings-panel-warning" role="alert">
            {state.apiKeyReferenceError}
          </p>
        ) : null}
        {renderReadOnlyCard("Organization source", state.organizationReference)}
        {renderReadOnlyCard("Project", state.project)}
        {renderReadOnlyCard("Voice", state.voice)}
        {renderReadOnlyCard("Audio format", state.audioFormat)}
        {renderReadOnlyCard("Timeout", state.timeoutMs != null ? `${(state.timeoutMs / 1000).toFixed(0)} seconds` : null)}
        {renderSecretEntryCard(
          "tts",
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

export function renderSettingsTtsProviderPanelNode(
  state: TtsProviderPanelState,
  handlers?: TtsProviderPanelHandlers,
): ReactNode {
  return renderSettingsPanelSection({
    titleId: "settings-tts-provider-title",
    title: "Voice output provider",
    description: "Choose the local or remote speech output provider. Changes apply to the next utterance.",
    error: state.error,
    onDismissError: handlers?.onDismissError,
    children: (
      <div className="settings-grid">
        {renderSelectControlCard({
          id: "settings-tts-provider-control",
          label: "Provider",
          valueText: renderProviderModeLabel(state.activeMode),
          selectedValue: state.activeMode,
          disabled: state.isBusy,
          dataAttributes: { "data-tts-provider-select": "true" },
          options: state.availableModes.map((mode) => ({ value: mode, label: renderProviderModeLabel(mode) })),
          onChange: handlers?.onProviderSelect
            ? (value) => { handlers.onProviderSelect?.(value as "Local" | "Remote"); }
            : undefined,
        })}
      </div>
    ),
  });
}

export function renderSettingsTtsModelPanelNode(
  state: TtsModelPanelState,
  handlers?: TtsModelPanelHandlers,
): ReactNode {
  const modeCopy = state.mode === "Remote" ? "remote" : state.mode === "Local" ? "local" : "disabled";
  const activeOption = state.availableProfiles.find((option) => option.profileName === state.activeProfile);

  return renderSettingsPanelSection({
    titleId: "settings-tts-model-title",
    title: "Voice model",
    description: `Choose the ${modeCopy} voice model for the current mode. Changes apply to the next utterance.`,
    error: state.error,
    onDismissError: handlers?.onDismissError,
    children: (
      <div className="settings-grid">
        {renderSelectControlCard({
          id: "settings-tts-model-control",
          label: "Selected model",
          valueText: activeOption
            ? renderTtsModelOptionLabel(activeOption.profileName, activeOption.modelLabel)
            : "No configured model",
          selectedValue: state.activeProfile ?? "",
          disabled: state.isBusy,
          dataAttributes: { "data-tts-model-select": "true" },
          options: state.availableProfiles.map((option) => ({
            value: option.profileName,
            label: renderTtsModelOptionLabel(option.profileName, option.modelLabel),
          })),
          onChange: handlers?.onModelSelect,
        })}
      </div>
    ),
  });
}

export function renderSettingsTtsVoicePanelNode(
  state: TtsVoicePanelState,
  handlers?: TtsVoicePanelHandlers,
): ReactNode {
  const modeCopy = state.mode === "Remote" ? "remote" : state.mode === "Local" ? "local" : "disabled";
  const activeOption = state.availableVoices.find((option) => option.voiceName === state.activeVoice);

  return renderSettingsPanelSection({
    titleId: "settings-tts-voice-title",
    title: "Voice",
    description: `Choose the ${modeCopy} voice for the current mode. Changes apply to the next utterance.`,
    error: state.error,
    onDismissError: handlers?.onDismissError,
    children: (
      <div className="settings-grid">
        {renderSelectControlCard({
          id: "settings-tts-voice-control",
          label: "Selected voice",
          valueText: activeOption
            ? renderTtsVoiceOptionLabel(activeOption.displayLabel, activeOption.voiceName)
            : state.activeVoice
              ? state.activeVoice
              : "No configured voice",
          selectedValue: state.activeVoice ?? "",
          disabled: state.isBusy,
          dataAttributes: { "data-tts-voice-select": "true" },
          options: state.availableVoices.map((option) => ({
            value: option.voiceName,
            label: renderTtsVoiceOptionLabel(option.displayLabel, option.voiceName),
          })),
          onChange: handlers?.onVoiceSelect,
        })}
      </div>
    ),
  });
}
