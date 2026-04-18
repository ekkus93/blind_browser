import { createElement, type ReactNode } from "react";

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
} from "./shared-controls.ts";

const h = createElement;

export function renderSettingsLocalTtsModelPanelNode(state: LocalTtsModelPanelState): ReactNode {
  return renderSettingsPanelSection({
    titleId: "settings-local-tts-model-title",
    title: "Local TTS profile",
    description: "Review the local speech profile used when TTS runs in local mode. Edit the app config to change it.",
    children: h(
      "div",
      { className: "settings-grid" },
      renderReadOnlyCard("Profile", state.profileName),
      renderReadOnlyCard("Backend", state.backend),
      renderReadOnlyCard("Model ID", state.modelId),
      renderReadOnlyCard("Model path", state.modelPath),
      renderReadOnlyCard("Default voice", state.defaultVoice),
      renderReadOnlyCard("Sample rate", state.sampleRate),
    ),
  });
}

export function renderSettingsRemoteTtsPanelNode(state: RemoteTtsPanelState): ReactNode {
  return renderSettingsPanelSection({
    titleId: "settings-remote-tts-title",
    title: "Remote TTS profile",
    description: "Review the speech profile used when TTS runs in remote mode. API keys stay masked here, and replacements are stored in the OS keyring instead of the config file.",
    error: state.error,
    children: h(
      "div",
      { className: "settings-grid" },
      renderReadOnlyCard("Profile", state.profileName),
      renderReadOnlyCard("Provider", state.provider),
      renderReadOnlyCard("Base URL", state.baseUrl),
      renderReadOnlyCard("Model", state.model),
      renderReadOnlyCard("API key source", state.apiKeyReference),
      renderReadOnlyCard("Organization source", state.organizationReference),
      renderReadOnlyCard("Project", state.project),
      renderReadOnlyCard("Voice", state.voice),
      renderReadOnlyCard("Audio format", state.audioFormat),
      renderReadOnlyCard("Timeout (ms)", state.timeoutMs),
      renderSecretEntryCard(
        "tts",
        state.profileName,
        state.apiKeyDraft,
        state.apiKeyMaskedValue,
        state.isSavingApiKey,
        state.isTestingApiKey,
        state.apiKeyReference !== null,
        state.apiKeyTestMessage,
      ),
    ),
  });
}

export function renderSettingsTtsProviderPanelNode(state: TtsProviderPanelState): ReactNode {
  return renderSettingsPanelSection({
    titleId: "settings-tts-provider-title",
    title: "TTS provider",
    description: "Choose the local or remote speech output provider. Changes apply to the next utterance.",
    error: state.error,
    children: h(
      "div",
      { className: "settings-grid" },
      renderSelectControlCard({
        id: "settings-tts-provider-control",
        label: "Provider",
        valueText: renderProviderModeLabel(state.activeMode),
        selectedValue: state.activeMode,
        disabled: state.isBusy,
        dataAttributes: { "data-tts-provider-select": "true" },
        options: state.availableModes.map((mode) => ({ value: mode, label: renderProviderModeLabel(mode) })),
      }),
    ),
  });
}

export function renderSettingsTtsModelPanelNode(state: TtsModelPanelState): ReactNode {
  const modeCopy = state.mode === "Remote" ? "remote" : state.mode === "Local" ? "local" : "disabled";
  const activeOption = state.availableProfiles.find((option) => option.profileName === state.activeProfile);

  return renderSettingsPanelSection({
    titleId: "settings-tts-model-title",
    title: "TTS model",
    description: `Choose the ${modeCopy} TTS model for the current mode. Changes apply to the next utterance.`,
    error: state.error,
    children: h(
      "div",
      { className: "settings-grid" },
      renderSelectControlCard({
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
      }),
    ),
  });
}

export function renderSettingsTtsVoicePanelNode(state: TtsVoicePanelState): ReactNode {
  const modeCopy = state.mode === "Remote" ? "remote" : state.mode === "Local" ? "local" : "disabled";
  const activeOption = state.availableVoices.find((option) => option.voiceName === state.activeVoice);

  return renderSettingsPanelSection({
    titleId: "settings-tts-voice-title",
    title: "Voice",
    description: `Choose the ${modeCopy} TTS voice for the current mode. Changes apply to the next utterance.`,
    error: state.error,
    children: h(
      "div",
      { className: "settings-grid" },
      renderSelectControlCard({
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
      }),
    ),
  });
}