import {
  escapeHtml,
  renderConfirmationThresholdValue,
  renderFailoverAvailabilityLabel,
  renderModelAvailabilityLabel,
  renderOcrThresholdValue,
  renderProviderModeLabel,
  renderSecretEntryCard,
  renderTextWithKnownLinkNodes,
  renderTtsModelOptionLabel,
  renderTtsVoiceOptionLabel,
} from "./confirmation-panel-helpers.ts";
import { createElement, type ReactNode } from "react";
import type {
  AudioControlsPanelState,
  AsrProviderPanelState,
  ConfirmationSettingsPanelState,
  LocalAsrModelPanelState,
  LocalTtsModelPanelState,
  ModelManagementPanelState,
  OcrThresholdSettingsPanelState,
  ProviderFailoverPanelState,
  RemoteAsrPanelState,
  RemotePlannerPanelState,
  RemoteTtsPanelState,
  SettingsGuidancePanelState,
  StatusPanelAgentStateLike,
  StatusPanelState,
  TtsModelPanelState,
  TtsProviderPanelState,
  TtsVoicePanelState,
  UrlInputPanelState,
} from "./panel-types.ts";

const h = createElement;

function renderReadOnlySettingText(value: string | number | null): string {
  if (value === null) {
    return "Not configured";
  }

  return `${value}`;
}

function renderReadOnlyCard(label: string, value: string | number | null) {
  return h(
    "div",
    { className: "settings-control-card" },
    h("span", { className: "settings-control-label" }, label),
    h("span", { className: "settings-control-value" }, renderReadOnlySettingText(value)),
  );
}

function renderPlaybackVolumeValueText(value: number): string {
  return `${Math.round(value * 100)} percent`;
}

function renderPlaybackSpeedValueText(value: number): string {
  return `${value.toFixed(2)} times`;
}

function renderConfirmationThresholdValueText(value: number): string {
  return `${Math.round(value * 100)} percent confidence`;
}

export function statusPanelStateFromAgentState(
  agentState: StatusPanelAgentStateLike,
): StatusPanelState {
  return {
    pageTitle: agentState.title ?? agentState.url,
    currentRegionLabel: agentState.narration_cursor
      ? `Region ${agentState.narration_cursor.node_index + 1}`
      : null,
    lastTranscript: agentState.last_transcript,
    listening: agentState.listening_state.is_listening,
    speaking: agentState.speaking,
    browserVisibility: agentState.browser_visibility,
    canGoBack: agentState.browser_history.can_go_back,
    canGoForward: agentState.browser_history.can_go_forward,
    isUpdatingVisibility: false,
    error: null,
  };
}

export function renderAudioControlsPanelNode(state: AudioControlsPanelState): ReactNode {
  return h(
      "section",
      { className: "audio-controls-panel", "aria-labelledby": "audio-controls-title" },
      h(
        "div",
        { className: "audio-controls-copy" },
        h("p", { className: "audio-controls-eyebrow" }, "Speech output"),
        h("h2", { id: "audio-controls-title" }, "Playback volume and speed"),
        state.error
          ? h("p", { className: "audio-controls-error", role: "alert" }, state.error)
          : null,
      ),
      h(
        "div",
        { className: "audio-controls-grid" },
        h(
          "label",
          { className: "audio-control", htmlFor: "playback-volume-control" },
          h("span", { className: "audio-control-label" }, "Volume"),
          h("span", { className: "audio-control-value" }, `${Math.round(state.playbackVolume * 100)}%`),
          h("input", {
            id: "playback-volume-control",
            className: "audio-control-input",
            "data-audio-control": "volume",
            type: "range",
            min: "0",
            max: "1",
            step: "0.05",
            value: state.playbackVolume.toFixed(2),
            "aria-valuetext": renderPlaybackVolumeValueText(state.playbackVolume),
            disabled: state.isBusy || undefined,
            "aria-disabled": state.isBusy ? "true" : undefined,
            readOnly: true,
          }),
        ),
        h(
          "label",
          { className: "audio-control", htmlFor: "playback-speed-control" },
          h("span", { className: "audio-control-label" }, "Speed"),
          h("span", { className: "audio-control-value" }, `${state.playbackSpeed.toFixed(2)}x`),
          h("input", {
            id: "playback-speed-control",
            className: "audio-control-input",
            "data-audio-control": "speed",
            type: "range",
            min: "0.5",
            max: "5",
            step: "0.05",
            value: state.playbackSpeed.toFixed(2),
            "aria-valuetext": renderPlaybackSpeedValueText(state.playbackSpeed),
            disabled: state.isBusy || undefined,
            "aria-disabled": state.isBusy ? "true" : undefined,
            readOnly: true,
          }),
        ),
      ),
  );
}

export function renderSettingsVolumePanel(state: AudioControlsPanelState): string {
  const busyAttribute = state.isBusy ? " disabled aria-disabled=\"true\"" : "";
  const errorCopy = state.error
    ? `<p class="settings-panel-error" role="alert">${escapeHtml(state.error)}</p>`
    : "";

  return `
    <section class="settings-panel" aria-labelledby="settings-volume-title">
      <div class="settings-panel-copy">
        <p class="settings-panel-eyebrow">Settings</p>
        <h2 id="settings-volume-title">Playback volume</h2>
        <p class="settings-panel-description">
          This dedicated settings control saves the default spoken playback volume for future
          narration. Updates apply to the next utterance and persist across app restarts.
        </p>
        ${errorCopy}
      </div>
      <div class="settings-grid">
        <label class="settings-control-card" for="settings-playback-volume-control">
          <span class="settings-control-label">Default volume</span>
          <span class="settings-control-value">${Math.round(state.playbackVolume * 100)}%</span>
          <input
            id="settings-playback-volume-control"
            class="settings-control-input"
            data-audio-control="volume"
            type="range"
            min="0"
            max="1"
            step="0.05"
            value="${state.playbackVolume.toFixed(2)}"
            aria-valuetext="${escapeHtml(renderPlaybackVolumeValueText(state.playbackVolume))}"
            ${busyAttribute}
          />
        </label>
      </div>
    </section>
  `;
}

export function renderSettingsSpeedPanel(state: AudioControlsPanelState): string {
  const busyAttribute = state.isBusy ? " disabled aria-disabled=\"true\"" : "";
  const errorCopy = state.error
    ? `<p class="settings-panel-error" role="alert">${escapeHtml(state.error)}</p>`
    : "";

  return `
    <section class="settings-panel" aria-labelledby="settings-speed-title">
      <div class="settings-panel-copy">
        <p class="settings-panel-eyebrow">Settings</p>
        <h2 id="settings-speed-title">Playback speed</h2>
        <p class="settings-panel-description">
          This dedicated settings control saves the default narration speed for future speech.
          Updates apply on the next utterance and persist across app restarts.
        </p>
        ${errorCopy}
      </div>
      <div class="settings-grid">
        <label class="settings-control-card" for="settings-playback-speed-control">
          <span class="settings-control-label">Default speed</span>
          <span class="settings-control-value">${state.playbackSpeed.toFixed(2)}x</span>
          <input
            id="settings-playback-speed-control"
            class="settings-control-input"
            data-audio-control="speed"
            type="range"
            min="0.5"
            max="5"
            step="0.05"
            value="${state.playbackSpeed.toFixed(2)}"
            aria-valuetext="${escapeHtml(renderPlaybackSpeedValueText(state.playbackSpeed))}"
            ${busyAttribute}
          />
        </label>
      </div>
    </section>
  `;
}

export function renderSettingsRemotePlannerPanelNode(state: RemotePlannerPanelState): ReactNode {
  const modelsAreFresh = (state.baseUrl?.trim().length ?? 0) > 0
    && state.loadedModelsEndpoint === state.baseUrl
    && state.availableModels.length > 0;
  const isConnectionBusy = state.isLoadingModels || state.isSavingConnection || state.isResettingConnection;
  const modelDisabled = isConnectionBusy || state.availableModels.length === 0;
  const loadModelsDisabled = isConnectionBusy || (state.baseUrl?.trim().length ?? 0) === 0;
  const saveSettingsDisabled = isConnectionBusy
    || !state.profileName
    || (state.baseUrl?.trim().length ?? 0) === 0
    || (state.model?.trim().length ?? 0) === 0
    || state.loadedModelsEndpoint !== state.baseUrl
  const resetSettingsDisabled = isConnectionBusy || !state.profileName;
  const modelOptions = state.availableModels.length > 0
    ? state.availableModels
    : [state.loadedModelsEndpoint === state.baseUrl && state.model ? state.model : "Load models for this endpoint"];

  return h(
      "section",
      { className: "settings-panel", "aria-labelledby": "settings-remote-planner-title" },
      h(
        "div",
        { className: "settings-panel-copy" },
        h("p", { className: "settings-panel-eyebrow" }, "Settings"),
        h("h2", { id: "settings-remote-planner-title" }, "Planner setup"),
        h("p", { className: "settings-panel-description" }, "Set the endpoint, model, and API key used to interpret commands."),
        state.error ? h("p", { className: "settings-panel-error", role: "alert" }, state.error) : null,
      ),
      h(
        "div",
        { className: "settings-grid settings-grid-single" },
        renderSecretEntryCard(
          "planner",
          state.profileName,
          state.apiKeyDraft,
          state.apiKeyMaskedValue,
          state.isSavingApiKey,
          state.isTestingApiKey,
          state.apiKeyReference !== null,
          state.apiKeyTestMessage,
        ),
      ),
      h(
        "div",
        { className: "settings-grid settings-grid-single settings-grid-compact" },
        h(
          "div",
          { className: "settings-control-card settings-planner-connection-card" },
          h(
            "label",
            { className: "settings-field-group", htmlFor: "settings-remote-planner-endpoint-input" },
            h("span", { className: "settings-control-label" }, "Endpoint"),
            h("input", {
              id: "settings-remote-planner-endpoint-input",
              className: "settings-control-select",
              "data-remote-planner-endpoint-input": "true",
              type: "text",
              value: state.baseUrl ?? "",
              placeholder: "https://api.openai.com/v1",
              spellCheck: false,
              autoComplete: "off",
              disabled: isConnectionBusy || undefined,
              "aria-disabled": isConnectionBusy ? "true" : undefined,
              readOnly: true,
            }),
          ),
        ),
      ),
      h(
        "div",
        { className: "settings-grid settings-grid-single settings-grid-compact" },
        h(
          "div",
          { className: "settings-control-card settings-planner-connection-card" },
          h(
            "label",
            { className: "settings-field-group", htmlFor: "settings-remote-planner-model-select" },
            h(
              "span",
              { className: "settings-control-label settings-inline-label-row" },
              h("span", null, "Model"),
              h("span", {
                className: `settings-status-light ${modelsAreFresh ? "settings-status-light-fresh" : "settings-status-light-stale"}`,
                role: "img",
                "aria-label": modelsAreFresh
                  ? "Models are loaded for the current endpoint"
                  : "Models need to be reloaded for the current endpoint",
              }),
            ),
            h(
              "div",
              { className: "settings-inline-control-row settings-inline-control-row-wrap" },
              h(
                "select",
                {
                  id: "settings-remote-planner-model-select",
                  className: "settings-control-select settings-inline-control-fill",
                  "data-remote-planner-model-select": "true",
                  value: state.model ?? "",
                  disabled: modelDisabled || undefined,
                  "aria-disabled": modelDisabled ? "true" : undefined,
                  onChange: () => undefined,
                },
                ...modelOptions.map((model) => h("option", { value: model, key: model }, model)),
              ),
              h(
                "button",
                {
                  type: "button",
                  className: "settings-control-button settings-control-button-secondary",
                  "data-remote-planner-models-refresh": "true",
                  disabled: loadModelsDisabled || undefined,
                  "aria-disabled": loadModelsDisabled ? "true" : undefined,
                },
                state.isLoadingModels ? "Loading models..." : "Load models",
              ),
            ),
          ),
          h(
            "div",
            { className: "settings-button-row settings-button-row-wrap" },
            h(
              "button",
              {
                type: "button",
                className: "settings-control-button",
                "data-remote-planner-settings-save": "true",
                disabled: saveSettingsDisabled || undefined,
                "aria-disabled": saveSettingsDisabled ? "true" : undefined,
              },
              state.isSavingConnection ? "Saving..." : "Save settings",
            ),
            h(
              "button",
              {
                type: "button",
                className: "settings-control-button settings-control-button-secondary",
                "data-remote-planner-settings-reset": "true",
                disabled: resetSettingsDisabled || undefined,
                "aria-disabled": resetSettingsDisabled ? "true" : undefined,
              },
              state.isResettingConnection ? "Resetting..." : "Reset to defaults",
            ),
          ),
        ),
      ),
  );
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

  return h(
      "section",
      { className: "settings-panel", "aria-labelledby": "settings-provider-failover-title" },
      h(
        "div",
        { className: "settings-panel-copy" },
        h("p", { className: "settings-panel-eyebrow" }, "Settings"),
        h("h2", { id: "settings-provider-failover-title" }, "Failover"),
        h(
          "p",
          { className: "settings-panel-description" },
          "Remote-to-local failover is not available yet. These toggles stay read-only until it is.",
        ),
        h("p", { className: "settings-panel-description" }, state.summary),
      ),
      h(
        "div",
        { className: "settings-grid" },
        renderFailoverCard("planner", "Planner", state.plannerAvailable),
        renderFailoverCard("tts", "TTS", state.ttsAvailable),
        renderFailoverCard("asr", "ASR", state.asrAvailable),
      ),
  );
}

export function renderSettingsConfirmationPanelNode(state: ConfirmationSettingsPanelState): ReactNode {
  return h(
      "section",
      { className: "settings-panel", "aria-labelledby": "settings-confirmation-title" },
      h(
        "div",
        { className: "settings-panel-copy" },
        h("p", { className: "settings-panel-eyebrow" }, "Settings"),
        h("h2", { id: "settings-confirmation-title" }, "Confirmation"),
        h(
          "p",
          { className: "settings-panel-description" },
          "Choose how confident a click must be before the app asks for confirmation. Form submits still always require confirmation.",
        ),
        state.error
          ? h("p", { className: "settings-panel-error", role: "alert" }, state.error)
          : null,
      ),
      h(
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
        h(
          "label",
          { className: "settings-control-card", htmlFor: "settings-click-without-confirmation-toggle" },
          h("span", { className: "settings-control-label" }, "Skip confirmation for confident clicks"),
          h(
            "span",
            { className: "settings-control-value" },
            state.allowClickWithoutConfirmation ? "Enabled" : "Disabled",
          ),
          h("input", {
            id: "settings-click-without-confirmation-toggle",
            className: "settings-control-input",
            "data-click-without-confirmation-toggle": "true",
            type: "checkbox",
            checked: state.allowClickWithoutConfirmation || undefined,
            disabled: state.isBusy || undefined,
            "aria-disabled": state.isBusy ? "true" : undefined,
            readOnly: true,
          }),
        ),
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
  );
}

export function renderSettingsOcrThresholdPanelNode(state: OcrThresholdSettingsPanelState): ReactNode {
  return h(
      "section",
      { className: "settings-panel", "aria-labelledby": "settings-ocr-thresholds-title" },
      h(
        "div",
        { className: "settings-panel-copy" },
        h("p", { className: "settings-panel-eyebrow" }, "Settings"),
        h("h2", { id: "settings-ocr-thresholds-title" }, "OCR fallback"),
        h(
          "p",
          { className: "settings-panel-description" },
          "Choose when sparse DOM extraction should fall back to OCR.",
        ),
        state.error
          ? h("p", { className: "settings-panel-error", role: "alert" }, state.error)
          : null,
      ),
      h(
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
  );
}

export function renderSettingsGuidancePanelNode(state: SettingsGuidancePanelState | null): ReactNode {
  if (!state) {
    return null;
  }

  return h(
      "section",
      { className: "settings-panel", "aria-labelledby": "settings-guidance-title" },
      h(
        "div",
        { className: "settings-panel-copy" },
        h("p", { className: "settings-panel-eyebrow" }, "Guidance"),
        h("h2", { id: "settings-guidance-title" }, state.title),
        h(
          "p",
          { className: "settings-panel-description" },
          ...renderTextWithKnownLinkNodes(state.message),
        ),
      ),
      h(
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
  );
}

export function renderSettingsAsrProviderPanelNode(state: AsrProviderPanelState): ReactNode {
  return h(
      "section",
      { className: "settings-panel", "aria-labelledby": "settings-asr-provider-title" },
      h(
        "div",
        { className: "settings-panel-copy" },
        h("p", { className: "settings-panel-eyebrow" }, "Settings"),
        h("h2", { id: "settings-asr-provider-title" }, "ASR provider"),
        h(
          "p",
          { className: "settings-panel-description" },
          "Choose the local or remote speech-to-text provider. Changes apply to the next listening request.",
        ),
        state.error
          ? h("p", { className: "settings-panel-error", role: "alert" }, state.error)
          : null,
      ),
      h(
        "div",
        { className: "settings-grid" },
        h(
          "label",
          { className: "settings-control-card", htmlFor: "settings-asr-provider-control" },
          h("span", { className: "settings-control-label" }, "Provider"),
          h("span", { className: "settings-control-value" }, renderProviderModeLabel(state.activeMode)),
          h(
            "select",
            {
              id: "settings-asr-provider-control",
              className: "settings-control-select",
              "data-asr-provider-select": "true",
              value: state.activeMode,
              disabled: state.isBusy || undefined,
              "aria-disabled": state.isBusy ? "true" : undefined,
              onChange: () => undefined,
            },
            ...state.availableModes.map((mode) => h(
              "option",
              { value: mode, key: mode },
              renderProviderModeLabel(mode),
            )),
          ),
        ),
      ),
  );
}

export function renderSettingsLocalTtsModelPanelNode(state: LocalTtsModelPanelState): ReactNode {
  return h(
      "section",
      { className: "settings-panel", "aria-labelledby": "settings-local-tts-model-title" },
      h(
        "div",
        { className: "settings-panel-copy" },
        h("p", { className: "settings-panel-eyebrow" }, "Settings"),
        h("h2", { id: "settings-local-tts-model-title" }, "Local TTS profile"),
        h(
          "p",
          { className: "settings-panel-description" },
          "Review the local speech profile used when TTS runs in local mode. Edit the app config to change it.",
        ),
      ),
      h(
        "div",
        { className: "settings-grid" },
        renderReadOnlyCard("Profile", state.profileName),
        renderReadOnlyCard("Backend", state.backend),
        renderReadOnlyCard("Model ID", state.modelId),
        renderReadOnlyCard("Model path", state.modelPath),
        renderReadOnlyCard("Default voice", state.defaultVoice),
        renderReadOnlyCard("Sample rate", state.sampleRate),
      ),
  );
}

export function renderSettingsModelManagementPanelNode(state: ModelManagementPanelState): ReactNode {
  const ttsDownloadDisabled = state.isDownloadingTts || !state.localTtsDownloadSupported;
  const asrDownloadDisabled = state.isDownloadingAsr || !state.localAsrDownloadSupported;

  return h(
      "section",
      { className: "settings-panel", "aria-labelledby": "settings-model-management-title" },
      h(
        "div",
        { className: "settings-panel-copy" },
        h("p", { className: "settings-panel-eyebrow" }, "Settings"),
        h("h2", { id: "settings-model-management-title" }, "Local models"),
        h(
          "p",
          { className: "settings-panel-description" },
          "Choose where local speech models live, whether startup checks them, and whether missing models download automatically.",
        ),
        state.error ? h("p", { className: "settings-panel-error", role: "alert" }, state.error) : null,
      ),
      h(
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
        h(
          "label",
          { className: "settings-control-card", htmlFor: "settings-model-check-on-startup-toggle" },
          h("span", { className: "settings-control-label" }, "Check on startup"),
          h("span", { className: "settings-control-value" }, state.checkOnStartup ? "Enabled" : "Disabled"),
          h("input", {
            id: "settings-model-check-on-startup-toggle",
            "data-model-management-toggle": "check-on-startup",
            type: "checkbox",
            checked: state.checkOnStartup || undefined,
            disabled: state.isSaving || undefined,
            "aria-disabled": state.isSaving ? "true" : undefined,
            readOnly: true,
          }),
        ),
        h(
          "label",
          { className: "settings-control-card", htmlFor: "settings-model-auto-download-toggle" },
          h("span", { className: "settings-control-label" }, "Auto-download missing"),
          h("span", { className: "settings-control-value" }, state.autoDownloadMissing ? "Enabled" : "Disabled"),
          h("input", {
            id: "settings-model-auto-download-toggle",
            "data-model-management-toggle": "auto-download-missing",
            type: "checkbox",
            checked: state.autoDownloadMissing || undefined,
            disabled: state.isSaving || undefined,
            "aria-disabled": state.isSaving ? "true" : undefined,
            readOnly: true,
          }),
        ),
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
  );
}

export function renderSettingsRemoteTtsPanelNode(state: RemoteTtsPanelState): ReactNode {
  return h(
      "section",
      { className: "settings-panel", "aria-labelledby": "settings-remote-tts-title" },
      h(
        "div",
        { className: "settings-panel-copy" },
        h("p", { className: "settings-panel-eyebrow" }, "Settings"),
        h("h2", { id: "settings-remote-tts-title" }, "Remote TTS profile"),
        h(
          "p",
          { className: "settings-panel-description" },
          "Review the speech profile used when TTS runs in remote mode. API keys stay masked here, and replacements are stored in the OS keyring instead of the config file.",
        ),
        state.error ? h("p", { className: "settings-panel-error", role: "alert" }, state.error) : null,
      ),
      h(
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
  );
}

export function renderSettingsTtsProviderPanelNode(state: TtsProviderPanelState): ReactNode {
  return h(
      "section",
      { className: "settings-panel", "aria-labelledby": "settings-tts-provider-title" },
      h(
        "div",
        { className: "settings-panel-copy" },
        h("p", { className: "settings-panel-eyebrow" }, "Settings"),
        h("h2", { id: "settings-tts-provider-title" }, "TTS provider"),
        h(
          "p",
          { className: "settings-panel-description" },
          "Choose the local or remote speech output provider. Changes apply to the next utterance.",
        ),
        state.error ? h("p", { className: "settings-panel-error", role: "alert" }, state.error) : null,
      ),
      h(
        "div",
        { className: "settings-grid" },
        h(
          "label",
          { className: "settings-control-card", htmlFor: "settings-tts-provider-control" },
          h("span", { className: "settings-control-label" }, "Provider"),
          h("span", { className: "settings-control-value" }, renderProviderModeLabel(state.activeMode)),
          h(
            "select",
            {
              id: "settings-tts-provider-control",
              className: "settings-control-select",
              "data-tts-provider-select": "true",
              value: state.activeMode,
              disabled: state.isBusy || undefined,
              "aria-disabled": state.isBusy ? "true" : undefined,
              onChange: () => undefined,
            },
            ...state.availableModes.map((mode) => h(
              "option",
              { value: mode, key: mode },
              renderProviderModeLabel(mode),
            )),
          ),
        ),
      ),
  );
}

export function renderSettingsLocalAsrModelPanelNode(state: LocalAsrModelPanelState): ReactNode {
  return h(
      "section",
      { className: "settings-panel", "aria-labelledby": "settings-local-asr-model-title" },
      h(
        "div",
        { className: "settings-panel-copy" },
        h("p", { className: "settings-panel-eyebrow" }, "Settings"),
        h("h2", { id: "settings-local-asr-model-title" }, "Local ASR profile"),
        h(
          "p",
          { className: "settings-panel-description" },
          "Review the speech-to-text profile used when ASR runs in local mode. Edit the app config to change it.",
        ),
      ),
      h(
        "div",
        { className: "settings-grid" },
        renderReadOnlyCard("Profile", state.profileName),
        renderReadOnlyCard("Backend", state.backend),
        renderReadOnlyCard("Model ID", state.modelId),
        renderReadOnlyCard("Model path", state.modelPath),
        renderReadOnlyCard("Language", state.language),
        renderReadOnlyCard("Threads", state.threads),
      ),
  );
}

export function renderSettingsRemoteAsrPanelNode(state: RemoteAsrPanelState): ReactNode {
  return h(
      "section",
      { className: "settings-panel", "aria-labelledby": "settings-remote-asr-title" },
      h(
        "div",
        { className: "settings-panel-copy" },
        h("p", { className: "settings-panel-eyebrow" }, "Settings"),
        h("h2", { id: "settings-remote-asr-title" }, "Remote ASR profile"),
        h(
          "p",
          { className: "settings-panel-description" },
          "Review the speech-to-text profile used when ASR runs in remote mode. API keys stay masked here, and replacements are stored in the OS keyring instead of the config file.",
        ),
        state.error ? h("p", { className: "settings-panel-error", role: "alert" }, state.error) : null,
      ),
      h(
        "div",
        { className: "settings-grid" },
        renderReadOnlyCard("Profile", state.profileName),
        renderReadOnlyCard("Provider", state.provider),
        renderReadOnlyCard("Base URL", state.baseUrl),
        renderReadOnlyCard("Model", state.model),
        renderReadOnlyCard("API key source", state.apiKeyReference),
        renderReadOnlyCard("Organization source", state.organizationReference),
        renderReadOnlyCard("Project", state.project),
        renderReadOnlyCard("Language", state.language),
        renderReadOnlyCard("Temperature (milli)", state.temperatureMilli),
        renderReadOnlyCard("Timeout (ms)", state.timeoutMs),
        renderSecretEntryCard(
          "asr",
          state.profileName,
          state.apiKeyDraft,
          state.apiKeyMaskedValue,
          state.isSavingApiKey,
          state.isTestingApiKey,
          state.apiKeyReference !== null,
          state.apiKeyTestMessage,
        ),
      ),
  );
}

export function renderSettingsTtsModelPanelNode(state: TtsModelPanelState): ReactNode {
  const modeCopy =
    state.mode === "Remote" ? "remote" : state.mode === "Local" ? "local" : "disabled";
  const activeOption = state.availableProfiles.find((option) => option.profileName === state.activeProfile);

  return h(
      "section",
      { className: "settings-panel", "aria-labelledby": "settings-tts-model-title" },
      h(
        "div",
        { className: "settings-panel-copy" },
        h("p", { className: "settings-panel-eyebrow" }, "Settings"),
        h("h2", { id: "settings-tts-model-title" }, "TTS model"),
        h(
          "p",
          { className: "settings-panel-description" },
          `Choose the ${modeCopy} TTS model for the current mode. Changes apply to the next utterance.`,
        ),
        state.error ? h("p", { className: "settings-panel-error", role: "alert" }, state.error) : null,
      ),
      h(
        "div",
        { className: "settings-grid" },
        h(
          "label",
          { className: "settings-control-card", htmlFor: "settings-tts-model-control" },
          h("span", { className: "settings-control-label" }, "Selected model"),
          h(
            "span",
            { className: "settings-control-value" },
            activeOption
              ? renderTtsModelOptionLabel(activeOption.profileName, activeOption.modelLabel)
              : "No configured model",
          ),
          h(
            "select",
            {
              id: "settings-tts-model-control",
              className: "settings-control-select",
              "data-tts-model-select": "true",
              value: state.activeProfile ?? "",
              disabled: state.isBusy || undefined,
              "aria-disabled": state.isBusy ? "true" : undefined,
              onChange: () => undefined,
            },
            ...state.availableProfiles.map((option) => h(
              "option",
              { value: option.profileName, key: option.profileName },
              renderTtsModelOptionLabel(option.profileName, option.modelLabel),
            )),
          ),
        ),
      ),
  );
}

export function renderSettingsTtsVoicePanelNode(state: TtsVoicePanelState): ReactNode {
  const modeCopy =
    state.mode === "Remote" ? "remote" : state.mode === "Local" ? "local" : "disabled";
  const activeOption = state.availableVoices.find((option) => option.voiceName === state.activeVoice);

  return h(
      "section",
      { className: "settings-panel", "aria-labelledby": "settings-tts-voice-title" },
      h(
        "div",
        { className: "settings-panel-copy" },
        h("p", { className: "settings-panel-eyebrow" }, "Settings"),
        h("h2", { id: "settings-tts-voice-title" }, "Voice"),
        h(
          "p",
          { className: "settings-panel-description" },
          `Choose the ${modeCopy} TTS voice for the current mode. Changes apply to the next utterance.`,
        ),
        state.error ? h("p", { className: "settings-panel-error", role: "alert" }, state.error) : null,
      ),
      h(
        "div",
        { className: "settings-grid" },
        h(
          "label",
          { className: "settings-control-card", htmlFor: "settings-tts-voice-control" },
          h("span", { className: "settings-control-label" }, "Selected voice"),
          h(
            "span",
            { className: "settings-control-value" },
            activeOption
              ? renderTtsVoiceOptionLabel(activeOption.displayLabel, activeOption.voiceName)
              : state.activeVoice
                ? state.activeVoice
                : "No configured voice",
          ),
          h(
            "select",
            {
              id: "settings-tts-voice-control",
              className: "settings-control-select",
              "data-tts-voice-select": "true",
              value: state.activeVoice ?? "",
              disabled: state.isBusy || undefined,
              "aria-disabled": state.isBusy ? "true" : undefined,
              onChange: () => undefined,
            },
            ...state.availableVoices.map((option) => h(
              "option",
              { value: option.voiceName, key: option.voiceName },
              renderTtsVoiceOptionLabel(option.displayLabel, option.voiceName),
            )),
          ),
        ),
      ),
  );
}

export function renderUrlInputPanelNode(state: UrlInputPanelState): ReactNode {
  const actionsDisabled = state.isOpening || state.isReading || state.isStopping || state.isAdvancing || state.isRewinding;

  return h(
    "section",
    { className: "url-input-panel", "aria-labelledby": "url-input-title" },
    h(
      "div",
      { className: "url-input-copy" },
      h("p", { className: "url-input-eyebrow" }, "Navigation"),
      h("h2", { id: "url-input-title" }, "URL input"),
      h(
        "p",
        { className: "url-input-description" },
        "Stage the next destination here. This keeps the nearby UI ready for direct navigation controls while voice-first command entry remains the primary path.",
      ),
      state.currentUrl
        ? h("p", { className: "url-input-current" }, h("strong", null, "Current URL:"), ` ${state.currentUrl}`)
        : h("p", { className: "url-input-current" }, "No page URL is loaded yet."),
      h(
        "p",
        { className: "url-input-status", role: "status", "aria-live": "polite", "aria-atomic": "true" },
        state.hasUnsubmittedChanges
          ? "Draft URL updated. Open controls can use this value next."
          : "The field mirrors the current page URL until you edit it.",
      ),
      state.error ? h("p", { className: "url-input-error", role: "alert" }, state.error) : null,
    ),
    h(
      "div",
      { className: "url-input-actions" },
      h(
        "label",
        { className: "url-input-field", htmlFor: "url-input-control" },
        h("span", { className: "url-input-label" }, "Page URL"),
        h("input", {
          id: "url-input-control",
          className: "url-input-control",
          "data-url-input": "true",
          type: "url",
          inputMode: "url",
          autoComplete: "url",
          spellCheck: false,
          placeholder: "https://example.com",
          value: state.draftValue,
          disabled: actionsDisabled || undefined,
          "aria-disabled": actionsDisabled ? "true" : undefined,
          readOnly: true,
        }),
      ),
      h("button", { type: "button", className: "url-open-button", "data-url-open-button": "true", disabled: actionsDisabled || undefined, "aria-disabled": actionsDisabled ? "true" : undefined }, state.isOpening ? "Opening..." : "Open"),
      h("button", { type: "button", className: "url-open-button url-read-button", "data-url-read-button": "true", disabled: actionsDisabled || undefined, "aria-disabled": actionsDisabled ? "true" : undefined }, state.isReading ? "Reading..." : "Read"),
      h("button", { type: "button", className: "url-open-button url-stop-button", "data-url-stop-button": "true", disabled: actionsDisabled || undefined, "aria-disabled": actionsDisabled ? "true" : undefined }, state.isStopping ? "Stopping..." : "Stop"),
      h("button", { type: "button", className: "url-open-button url-previous-button", "data-url-previous-button": "true", disabled: actionsDisabled || undefined, "aria-disabled": actionsDisabled ? "true" : undefined }, state.isRewinding ? "Previous..." : "Previous"),
      h("button", { type: "button", className: "url-open-button url-next-button", "data-url-next-button": "true", disabled: actionsDisabled || undefined, "aria-disabled": actionsDisabled ? "true" : undefined }, state.isAdvancing ? "Next..." : "Next"),
    ),
  );
}

export function renderStatusPanelNode(state: StatusPanelState): ReactNode {
  const title = state.pageTitle ?? "No page open yet";
  const region = state.currentRegionLabel ?? "No current region";
  const transcript = state.lastTranscript ?? "No spoken command captured yet";
  const visiblePressed = state.browserVisibility === "Visible";
  const headlessPressed = state.browserVisibility === "Headless";
  return h(
    "section",
    { className: "status-panel", "aria-labelledby": "status-panel-title" },
    h(
      "div",
      { className: "status-panel-copy" },
      h("p", { className: "status-panel-eyebrow" }, "Runtime status"),
      h("h2", { id: "status-panel-title" }, "Current browser state"),
      h("p", { className: "status-panel-description" }, "This panel mirrors the live runtime so the nearby UI stays aligned with what the browser, narration, and listening tools are doing right now."),
      state.error ? h("p", { className: "status-panel-error", role: "alert" }, state.error) : null,
    ),
    h(
      "dl",
      { className: "status-panel-grid" },
      h("div", { className: "status-card status-card-wide" }, h("dt", null, "Page title"), h("dd", null, title)),
      h("div", { className: "status-card" }, h("dt", null, "Current region"), h("dd", { "aria-live": "polite", "aria-atomic": "true" }, region)),
      h("div", { className: "status-card status-card-wide status-card-transcript" }, h("dt", null, "Last transcript"), h("dd", { "aria-live": "polite", "aria-atomic": "true" }, transcript)),
      h("div", { className: "status-card" }, h("dt", null, "Listening"), h("dd", null, h("span", { className: `status-indicator${state.listening ? " status-indicator-active" : ""}`, role: "status", "aria-live": "polite", "aria-atomic": "true" }, state.listening ? "Active" : "Idle"))),
      h("div", { className: "status-card" }, h("dt", null, "Speaking"), h("dd", null, h("span", { className: `status-indicator${state.speaking ? " status-indicator-active" : ""}`, role: "status", "aria-live": "polite", "aria-atomic": "true" }, state.speaking ? "Active" : "Idle"))),
      h(
        "div",
        { className: "status-card" },
        h("dt", null, "Browser mode"),
        h(
          "dd",
          null,
          h("span", { className: "status-mode-label", role: "status", "aria-live": "polite", "aria-atomic": "true" }, state.browserVisibility),
          h(
            "div",
            { className: "status-toggle-group", role: "group", "aria-label": "Browser visibility mode" },
            h("button", { type: "button", className: `status-toggle-button${visiblePressed ? " status-toggle-button-active" : ""}`, "data-browser-visibility-mode": "Visible", "aria-label": "Browser visibility mode: Visible", "aria-pressed": String(visiblePressed), disabled: state.isUpdatingVisibility || undefined, "aria-disabled": state.isUpdatingVisibility ? "true" : undefined }, "Visible"),
            h("button", { type: "button", className: `status-toggle-button${headlessPressed ? " status-toggle-button-active" : ""}`, "data-browser-visibility-mode": "Headless", "aria-label": "Browser visibility mode: Headless", "aria-pressed": String(headlessPressed), disabled: state.isUpdatingVisibility || undefined, "aria-disabled": state.isUpdatingVisibility ? "true" : undefined }, "Headless"),
          ),
        ),
      ),
      h("div", { className: "status-card" }, h("dt", null, "History"), h("dd", null, `Back: ${state.canGoBack ? "Available" : "Unavailable"}. Forward: ${state.canGoForward ? "Available" : "Unavailable"}.`)),
    ),
  );
}

