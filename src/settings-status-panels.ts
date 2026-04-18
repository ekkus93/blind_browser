import {
  escapeHtml,
  renderConfirmationThresholdValue,
  renderFailoverAvailabilityLabel,
  renderModelAvailabilityLabel,
  renderOcrThresholdValue,
  renderProviderModeLabel,
  renderReadOnlySettingValue,
  renderSecretEntryCard,
  renderTextWithKnownLinks,
  renderTtsModelOptionLabel,
  renderTtsVoiceOptionLabel,
} from "./confirmation-panel-helpers.ts";
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

export function renderAudioControlsPanel(state: AudioControlsPanelState): string {
  const busyAttribute = state.isBusy ? " disabled aria-disabled=\"true\"" : "";
  const errorCopy = state.error
    ? `<p class="audio-controls-error" role="alert">${escapeHtml(state.error)}</p>`
    : "";

  return `
    <section class="audio-controls-panel" aria-labelledby="audio-controls-title">
      <div class="audio-controls-copy">
        <p class="audio-controls-eyebrow">Speech output</p>
        <h2 id="audio-controls-title">Playback volume and speed</h2>
        <p class="audio-controls-description">
          Adjust narration volume and speed here. Changes apply to the current playback flow and
          remain the saved defaults for future narration.
        </p>
        ${errorCopy}
      </div>
      <div class="audio-controls-grid">
        <label class="audio-control" for="playback-volume-control">
          <span class="audio-control-label">Volume</span>
          <span class="audio-control-value">${Math.round(state.playbackVolume * 100)}%</span>
          <input
            id="playback-volume-control"
            class="audio-control-input"
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
        <label class="audio-control" for="playback-speed-control">
          <span class="audio-control-label">Speed</span>
          <span class="audio-control-value">${state.playbackSpeed.toFixed(2)}x</span>
          <input
            id="playback-speed-control"
            class="audio-control-input"
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

export function renderSettingsRemotePlannerPanel(state: RemotePlannerPanelState): string {
  const errorCopy = state.error
    ? `<p class="settings-panel-error" role="alert">${escapeHtml(state.error)}</p>`
    : "";
  const modelsAreFresh = (state.baseUrl?.trim().length ?? 0) > 0
    && state.loadedModelsEndpoint === state.baseUrl
    && state.availableModels.length > 0;
  const isConnectionBusy = state.isLoadingModels || state.isSavingConnection || state.isResettingConnection;
  const endpointDisabledAttribute = isConnectionBusy ? " disabled aria-disabled=\"true\"" : "";
  const modelDisabledAttribute = isConnectionBusy || state.availableModels.length === 0
    ? " disabled aria-disabled=\"true\""
    : "";
  const loadModelsDisabledAttribute = isConnectionBusy || (state.baseUrl?.trim().length ?? 0) === 0
    ? " disabled aria-disabled=\"true\""
    : "";
  const saveSettingsDisabledAttribute = isConnectionBusy
    || !state.profileName
    || (state.baseUrl?.trim().length ?? 0) === 0
    || (state.model?.trim().length ?? 0) === 0
    || state.loadedModelsEndpoint !== state.baseUrl
      ? " disabled aria-disabled=\"true\""
      : "";
  const resetSettingsDisabledAttribute = isConnectionBusy || !state.profileName
    ? " disabled aria-disabled=\"true\""
    : "";
  const modelOptions = state.availableModels.length > 0
    ? state.availableModels
        .map(
          (model) => `<option value="${escapeHtml(model)}"${model === state.model ? " selected" : ""}>${escapeHtml(model)}</option>`,
        )
        .join("")
    : `<option value="">${escapeHtml(state.loadedModelsEndpoint === state.baseUrl && state.model ? state.model : "Load models for this endpoint")}</option>`;

  return `
    <section class="settings-panel" aria-labelledby="settings-remote-planner-title">
      <div class="settings-panel-copy">
        <p class="settings-panel-eyebrow">Settings</p>
        <h2 id="settings-remote-planner-title">Planner setup</h2>
        <p class="settings-panel-description">
          Set the endpoint, model, and API key used to interpret commands.
        </p>
        ${errorCopy}
      </div>
      <div class="settings-grid settings-grid-single">
        ${renderSecretEntryCard(
          "planner",
          state.profileName,
          state.apiKeyDraft,
          state.apiKeyMaskedValue,
          state.isSavingApiKey,
          state.isTestingApiKey,
          state.apiKeyReference !== null,
          state.apiKeyTestMessage,
        )}
      </div>
      <div class="settings-grid settings-grid-single settings-grid-compact">
        <div class="settings-control-card settings-planner-connection-card">
          <label class="settings-field-group" for="settings-remote-planner-endpoint-input">
            <span class="settings-control-label">Endpoint</span>
            <input
              id="settings-remote-planner-endpoint-input"
              class="settings-control-select"
              data-remote-planner-endpoint-input="true"
              type="text"
              value="${escapeHtml(state.baseUrl ?? "")}"
              placeholder="https://api.openai.com/v1"
              spellcheck="false"
              autocomplete="off"
              ${endpointDisabledAttribute}
            />
          </label>
        </div>
      </div>
      <div class="settings-grid settings-grid-single settings-grid-compact">
        <div class="settings-control-card settings-planner-connection-card">
          <label class="settings-field-group" for="settings-remote-planner-model-select">
            <span class="settings-control-label settings-inline-label-row">
              <span>Model</span>
              <span
                class="settings-status-light ${modelsAreFresh ? "settings-status-light-fresh" : "settings-status-light-stale"}"
                role="img"
                aria-label="${escapeHtml(modelsAreFresh ? "Models are loaded for the current endpoint" : "Models need to be reloaded for the current endpoint")}" 
              ></span>
            </span>
            <div class="settings-inline-control-row settings-inline-control-row-wrap">
              <select
                id="settings-remote-planner-model-select"
                class="settings-control-select settings-inline-control-fill"
                data-remote-planner-model-select="true"
                ${modelDisabledAttribute}
              >
                ${modelOptions}
              </select>
              <button
                type="button"
                class="settings-control-button settings-control-button-secondary"
                data-remote-planner-models-refresh="true"
                ${loadModelsDisabledAttribute}
              >
                ${escapeHtml(state.isLoadingModels ? "Loading models..." : "Load models")}
              </button>
            </div>
          </label>
          <div class="settings-button-row settings-button-row-wrap">
            <button
              type="button"
              class="settings-control-button"
              data-remote-planner-settings-save="true"
              ${saveSettingsDisabledAttribute}
            >
              ${escapeHtml(state.isSavingConnection ? "Saving..." : "Save settings")}
            </button>
            <button
              type="button"
              class="settings-control-button settings-control-button-secondary"
              data-remote-planner-settings-reset="true"
              ${resetSettingsDisabledAttribute}
            >
              ${escapeHtml(state.isResettingConnection ? "Resetting..." : "Reset to defaults")}
            </button>
          </div>
        </div>
      </div>
    </section>
  `;
}

export function renderSettingsProviderFailoverPanel(state: ProviderFailoverPanelState): string {
  const renderFailoverCard = (
    providerKey: "planner" | "tts" | "asr",
    providerLabel: string,
    available: boolean,
  ): string => `
    <label class="settings-control-card" for="settings-provider-failover-${providerKey}">
      <span class="settings-control-label">${escapeHtml(providerLabel)}</span>
      <span class="settings-control-value">${escapeHtml(renderFailoverAvailabilityLabel(available))}</span>
      <input
        id="settings-provider-failover-${providerKey}"
        class="settings-control-input"
        data-provider-failover-toggle="${escapeHtml(providerKey)}"
        type="checkbox"
        disabled
        aria-disabled="true"
      />
    </label>
  `;

  return `
    <section class="settings-panel" aria-labelledby="settings-provider-failover-title">
      <div class="settings-panel-copy">
        <p class="settings-panel-eyebrow">Settings</p>
        <h2 id="settings-provider-failover-title">Failover</h2>
        <p class="settings-panel-description">
          Remote-to-local failover is not available yet. These toggles stay read-only until it is.
        </p>
        <p class="settings-panel-description">${escapeHtml(state.summary)}</p>
      </div>
      <div class="settings-grid">
        ${renderFailoverCard("planner", "Planner", state.plannerAvailable)}
        ${renderFailoverCard("tts", "TTS", state.ttsAvailable)}
        ${renderFailoverCard("asr", "ASR", state.asrAvailable)}
      </div>
    </section>
  `;
}

export function renderSettingsConfirmationPanel(state: ConfirmationSettingsPanelState): string {
  const disabledAttribute = state.isBusy ? " disabled aria-disabled=\"true\"" : "";
  const errorCopy = state.error
    ? `<p class="settings-panel-error" role="alert">${escapeHtml(state.error)}</p>`
    : "";
  const clickWithoutConfirmationChecked = state.allowClickWithoutConfirmation ? " checked" : "";

  return `
    <section class="settings-panel" aria-labelledby="settings-confirmation-title">
      <div class="settings-panel-copy">
        <p class="settings-panel-eyebrow">Settings</p>
        <h2 id="settings-confirmation-title">Confirmation</h2>
        <p class="settings-panel-description">
          Choose how confident a click must be before the app asks for confirmation. Form submits
          still always require confirmation.
        </p>
        ${errorCopy}
      </div>
      <div class="settings-grid">
        <label class="settings-control-card" for="settings-confirmation-threshold-control">
          <span class="settings-control-label">Click threshold</span>
          <span class="settings-control-value">${renderConfirmationThresholdValue(state.confirmationConfidenceThreshold)}</span>
          <input
            id="settings-confirmation-threshold-control"
            class="settings-control-input"
            data-confirmation-threshold-control="true"
            type="range"
            min="0"
            max="1"
            step="0.01"
            value="${state.confirmationConfidenceThreshold.toFixed(2)}"
            aria-valuetext="${escapeHtml(renderConfirmationThresholdValueText(state.confirmationConfidenceThreshold))}"
            ${disabledAttribute}
          />
        </label>
        <label class="settings-control-card" for="settings-click-without-confirmation-toggle">
          <span class="settings-control-label">Skip confirmation for confident clicks</span>
          <span class="settings-control-value">${state.allowClickWithoutConfirmation ? "Enabled" : "Disabled"}</span>
          <input
            id="settings-click-without-confirmation-toggle"
            class="settings-control-input"
            data-click-without-confirmation-toggle="true"
            type="checkbox"
            ${clickWithoutConfirmationChecked}
            ${disabledAttribute}
          />
        </label>
        <div class="settings-control-card" aria-live="polite">
          <span class="settings-control-label">Submit actions</span>
          <span class="settings-control-value">${state.alwaysConfirmSubmit ? "Always require confirmation" : "Confirmation not required"}</span>
        </div>
      </div>
    </section>
  `;
}

export function renderSettingsOcrThresholdPanel(state: OcrThresholdSettingsPanelState): string {
  const disabledAttribute = state.isBusy ? " disabled aria-disabled=\"true\"" : "";
  const errorCopy = state.error
    ? `<p class="settings-panel-error" role="alert">${escapeHtml(state.error)}</p>`
    : "";

  return `
    <section class="settings-panel" aria-labelledby="settings-ocr-thresholds-title">
      <div class="settings-panel-copy">
        <p class="settings-panel-eyebrow">Settings</p>
        <h2 id="settings-ocr-thresholds-title">OCR fallback</h2>
        <p class="settings-panel-description">
          Choose when sparse DOM extraction should fall back to OCR.
        </p>
        ${errorCopy}
      </div>
      <div class="settings-grid">
        <label class="settings-control-card" for="settings-ocr-char-threshold-control">
          <span class="settings-control-label">Character threshold</span>
          <span class="settings-control-value">${renderOcrThresholdValue(state.sparseTextCharThreshold)}</span>
          <input
            id="settings-ocr-char-threshold-control"
            class="settings-control-input"
            data-ocr-threshold-control="char"
            type="number"
            min="1"
            step="1"
            value="${state.sparseTextCharThreshold}"
            ${disabledAttribute}
          />
        </label>
        <label class="settings-control-card" for="settings-ocr-region-threshold-control">
          <span class="settings-control-label">Region threshold</span>
          <span class="settings-control-value">${renderOcrThresholdValue(state.sparseTextRegionThreshold)}</span>
          <input
            id="settings-ocr-region-threshold-control"
            class="settings-control-input"
            data-ocr-threshold-control="region"
            type="number"
            min="1"
            step="1"
            value="${state.sparseTextRegionThreshold}"
            ${disabledAttribute}
          />
        </label>
      </div>
    </section>
  `;
}

export function renderSettingsGuidancePanel(state: SettingsGuidancePanelState | null): string {
  if (!state) {
    return "";
  }

  const actionsCopy = state.actions
    .map(
      (action) => `
        <button
          type="button"
          class="url-open-button"
          data-settings-target="${escapeHtml(action.targetId)}"
        >
          ${escapeHtml(action.label)}
        </button>
      `,
    )
    .join("");

  return `
    <section class="settings-panel" aria-labelledby="settings-guidance-title">
      <div class="settings-panel-copy">
        <p class="settings-panel-eyebrow">Guidance</p>
        <h2 id="settings-guidance-title">${escapeHtml(state.title)}</h2>
        <p class="settings-panel-description">${renderTextWithKnownLinks(state.message)}</p>
      </div>
      <div class="url-input-actions">
        ${actionsCopy}
      </div>
    </section>
  `;
}

export function renderSettingsAsrProviderPanel(state: AsrProviderPanelState): string {
  const disabledAttribute = state.isBusy ? " disabled aria-disabled=\"true\"" : "";
  const errorCopy = state.error
    ? `<p class="settings-panel-error" role="alert">${escapeHtml(state.error)}</p>`
    : "";
  const optionsCopy = state.availableModes
    .map((mode) => {
      const selected = mode === state.activeMode ? " selected" : "";
      return `<option value="${escapeHtml(mode)}"${selected}>${escapeHtml(renderProviderModeLabel(mode))}</option>`;
    })
    .join("");

  return `
    <section class="settings-panel" aria-labelledby="settings-asr-provider-title">
      <div class="settings-panel-copy">
        <p class="settings-panel-eyebrow">Settings</p>
        <h2 id="settings-asr-provider-title">ASR provider</h2>
        <p class="settings-panel-description">
          Choose the local or remote speech-to-text provider. Changes apply to the next listening request.
        </p>
        ${errorCopy}
      </div>
      <div class="settings-grid">
        <label class="settings-control-card" for="settings-asr-provider-control">
          <span class="settings-control-label">Provider</span>
          <span class="settings-control-value">${escapeHtml(renderProviderModeLabel(state.activeMode))}</span>
          <select
            id="settings-asr-provider-control"
            class="settings-control-select"
            data-asr-provider-select="true"
            ${disabledAttribute}
          >
            ${optionsCopy}
          </select>
        </label>
      </div>
    </section>
  `;
}

export function renderSettingsLocalTtsModelPanel(state: LocalTtsModelPanelState): string {
  return `
    <section class="settings-panel" aria-labelledby="settings-local-tts-model-title">
      <div class="settings-panel-copy">
        <p class="settings-panel-eyebrow">Settings</p>
        <h2 id="settings-local-tts-model-title">Local TTS profile</h2>
        <p class="settings-panel-description">
          Review the local speech profile used when TTS runs in local mode. Edit the app config to
          change it.
        </p>
      </div>
      <div class="settings-grid">
        <div class="settings-control-card">
          <span class="settings-control-label">Profile</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.profileName)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Backend</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.backend)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Model ID</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.modelId)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Model path</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.modelPath)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Default voice</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.defaultVoice)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Sample rate</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.sampleRate)}</span>
        </div>
      </div>
    </section>
  `;
}

export function renderSettingsModelManagementPanel(state: ModelManagementPanelState): string {
  const disabledAttribute = state.isSaving ? " disabled aria-disabled=\"true\"" : "";
  const ttsDownloadDisabled =
    state.isDownloadingTts || !state.localTtsDownloadSupported
      ? " disabled aria-disabled=\"true\""
      : "";
  const asrDownloadDisabled =
    state.isDownloadingAsr || !state.localAsrDownloadSupported
      ? " disabled aria-disabled=\"true\""
      : "";
  const errorCopy = state.error
    ? `<p class="settings-panel-error" role="alert">${escapeHtml(state.error)}</p>`
    : "";

  return `
    <section class="settings-panel" aria-labelledby="settings-model-management-title">
      <div class="settings-panel-copy">
        <p class="settings-panel-eyebrow">Settings</p>
        <h2 id="settings-model-management-title">Local models</h2>
        <p class="settings-panel-description">
          Choose where local speech models live, whether startup checks them, and whether missing
          models download automatically.
        </p>
        ${errorCopy}
      </div>
      <div class="settings-grid">
        <label class="settings-control-card" for="settings-models-dir-input">
          <span class="settings-control-label">Model folder</span>
          <span class="settings-control-value">${escapeHtml(state.modelsDir || "Not configured")}</span>
          <input
            id="settings-models-dir-input"
            class="settings-control-select"
            data-model-management-input="models-dir"
            type="text"
            value="${escapeHtml(state.modelsDir)}"
            placeholder="~/.local/share/blind_browser/models"
            spellcheck="false"
            aria-describedby="settings-models-dir-description"
            ${disabledAttribute}
          />
          <span id="settings-models-dir-description" class="settings-panel-description">
            Updates here change where downloads and startup checks look for speech models.
          </span>
        </label>
        <label class="settings-control-card" for="settings-model-check-on-startup-toggle">
          <span class="settings-control-label">Check on startup</span>
          <span class="settings-control-value">${state.checkOnStartup ? "Enabled" : "Disabled"}</span>
          <input
            id="settings-model-check-on-startup-toggle"
            data-model-management-toggle="check-on-startup"
            type="checkbox"
            ${state.checkOnStartup ? "checked" : ""}
            ${disabledAttribute}
          />
        </label>
        <label class="settings-control-card" for="settings-model-auto-download-toggle">
          <span class="settings-control-label">Auto-download missing</span>
          <span class="settings-control-value">${state.autoDownloadMissing ? "Enabled" : "Disabled"}</span>
          <input
            id="settings-model-auto-download-toggle"
            data-model-management-toggle="auto-download-missing"
            type="checkbox"
            ${state.autoDownloadMissing ? "checked" : ""}
            ${disabledAttribute}
          />
        </label>
        <div class="settings-control-card">
          <span class="settings-control-label">Local TTS</span>
          <span class="settings-control-value">${renderModelAvailabilityLabel(state.localTtsAvailable)}</span>
          <button
            type="button"
            class="settings-control-button"
            data-model-download="tts"
            ${ttsDownloadDisabled}
          >
            ${escapeHtml(state.isDownloadingTts ? "Downloading..." : (state.localTtsDownloadLabel ?? "Download unavailable"))}
          </button>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Local ASR</span>
          <span class="settings-control-value">${renderModelAvailabilityLabel(state.localAsrAvailable)}</span>
          <button
            type="button"
            class="settings-control-button"
            data-model-download="asr"
            ${asrDownloadDisabled}
          >
            ${escapeHtml(state.isDownloadingAsr ? "Downloading..." : (state.localAsrDownloadLabel ?? "Download unavailable"))}
          </button>
        </div>
      </div>
    </section>
  `;
}

export function renderSettingsRemoteTtsPanel(state: RemoteTtsPanelState): string {
  const errorCopy = state.error
    ? `<p class="settings-panel-error" role="alert">${escapeHtml(state.error)}</p>`
    : "";

  return `
    <section class="settings-panel" aria-labelledby="settings-remote-tts-title">
      <div class="settings-panel-copy">
        <p class="settings-panel-eyebrow">Settings</p>
        <h2 id="settings-remote-tts-title">Remote TTS profile</h2>
        <p class="settings-panel-description">
          Review the speech profile used when TTS runs in remote mode. API keys stay masked here,
          and replacements are stored in the OS keyring instead of the config file.
        </p>
        ${errorCopy}
      </div>
      <div class="settings-grid">
        <div class="settings-control-card">
          <span class="settings-control-label">Profile</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.profileName)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Provider</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.provider)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Base URL</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.baseUrl)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Model</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.model)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">API key source</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.apiKeyReference)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Organization source</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.organizationReference)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Project</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.project)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Voice</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.voice)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Audio format</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.audioFormat)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Timeout (ms)</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.timeoutMs)}</span>
        </div>
        ${renderSecretEntryCard(
          "tts",
          state.profileName,
          state.apiKeyDraft,
          state.apiKeyMaskedValue,
          state.isSavingApiKey,
          state.isTestingApiKey,
          state.apiKeyReference !== null,
          state.apiKeyTestMessage,
        )}
      </div>
    </section>
  `;
}

export function renderSettingsTtsProviderPanel(state: TtsProviderPanelState): string {
  const disabledAttribute = state.isBusy ? " disabled aria-disabled=\"true\"" : "";
  const errorCopy = state.error
    ? `<p class="settings-panel-error" role="alert">${escapeHtml(state.error)}</p>`
    : "";
  const optionsCopy = state.availableModes
    .map((mode) => {
      const selected = mode === state.activeMode ? " selected" : "";
      return `<option value="${escapeHtml(mode)}"${selected}>${escapeHtml(renderProviderModeLabel(mode))}</option>`;
    })
    .join("");

  return `
    <section class="settings-panel" aria-labelledby="settings-tts-provider-title">
      <div class="settings-panel-copy">
        <p class="settings-panel-eyebrow">Settings</p>
        <h2 id="settings-tts-provider-title">TTS provider</h2>
        <p class="settings-panel-description">
          Choose the local or remote speech output provider. Changes apply to the next utterance.
        </p>
        ${errorCopy}
      </div>
      <div class="settings-grid">
        <label class="settings-control-card" for="settings-tts-provider-control">
          <span class="settings-control-label">Provider</span>
          <span class="settings-control-value">${escapeHtml(renderProviderModeLabel(state.activeMode))}</span>
          <select
            id="settings-tts-provider-control"
            class="settings-control-select"
            data-tts-provider-select="true"
            ${disabledAttribute}
          >
            ${optionsCopy}
          </select>
        </label>
      </div>
    </section>
  `;
}

export function renderSettingsLocalAsrModelPanel(state: LocalAsrModelPanelState): string {
  return `
    <section class="settings-panel" aria-labelledby="settings-local-asr-model-title">
      <div class="settings-panel-copy">
        <p class="settings-panel-eyebrow">Settings</p>
        <h2 id="settings-local-asr-model-title">Local ASR profile</h2>
        <p class="settings-panel-description">
          Review the speech-to-text profile used when ASR runs in local mode. Edit the app config
          to change it.
        </p>
      </div>
      <div class="settings-grid">
        <div class="settings-control-card">
          <span class="settings-control-label">Profile</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.profileName)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Backend</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.backend)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Model ID</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.modelId)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Model path</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.modelPath)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Language</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.language)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Threads</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.threads)}</span>
        </div>
      </div>
    </section>
  `;
}

export function renderSettingsRemoteAsrPanel(state: RemoteAsrPanelState): string {
  const errorCopy = state.error
    ? `<p class="settings-panel-error" role="alert">${escapeHtml(state.error)}</p>`
    : "";

  return `
    <section class="settings-panel" aria-labelledby="settings-remote-asr-title">
      <div class="settings-panel-copy">
        <p class="settings-panel-eyebrow">Settings</p>
        <h2 id="settings-remote-asr-title">Remote ASR profile</h2>
        <p class="settings-panel-description">
          Review the speech-to-text profile used when ASR runs in remote mode. API keys stay
          masked here, and replacements are stored in the OS keyring instead of the config file.
        </p>
        ${errorCopy}
      </div>
      <div class="settings-grid">
        <div class="settings-control-card">
          <span class="settings-control-label">Profile</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.profileName)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Provider</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.provider)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Base URL</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.baseUrl)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Model</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.model)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">API key source</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.apiKeyReference)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Organization source</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.organizationReference)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Project</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.project)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Language</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.language)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Temperature (milli)</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.temperatureMilli)}</span>
        </div>
        <div class="settings-control-card">
          <span class="settings-control-label">Timeout (ms)</span>
          <span class="settings-control-value">${renderReadOnlySettingValue(state.timeoutMs)}</span>
        </div>
        ${renderSecretEntryCard(
          "asr",
          state.profileName,
          state.apiKeyDraft,
          state.apiKeyMaskedValue,
          state.isSavingApiKey,
          state.isTestingApiKey,
          state.apiKeyReference !== null,
          state.apiKeyTestMessage,
        )}
      </div>
    </section>
  `;
}

export function renderSettingsTtsModelPanel(state: TtsModelPanelState): string {
  const disabledAttribute = state.isBusy ? " disabled aria-disabled=\"true\"" : "";
  const errorCopy = state.error
    ? `<p class="settings-panel-error" role="alert">${escapeHtml(state.error)}</p>`
    : "";
  const optionsCopy = state.availableProfiles
    .map((option) => {
      const selected = option.profileName === state.activeProfile ? " selected" : "";
      return `<option value="${escapeHtml(option.profileName)}"${selected}>${escapeHtml(
        renderTtsModelOptionLabel(option.profileName, option.modelLabel),
      )}</option>`;
    })
    .join("");
  const modeCopy =
    state.mode === "Remote" ? "remote" : state.mode === "Local" ? "local" : "disabled";
  const activeOption = state.availableProfiles.find((option) => option.profileName === state.activeProfile);

  return `
    <section class="settings-panel" aria-labelledby="settings-tts-model-title">
      <div class="settings-panel-copy">
        <p class="settings-panel-eyebrow">Settings</p>
        <h2 id="settings-tts-model-title">TTS model</h2>
        <p class="settings-panel-description">
          Choose the ${modeCopy} TTS model for the current mode. Changes apply to the next utterance.
        </p>
        ${errorCopy}
      </div>
      <div class="settings-grid">
        <label class="settings-control-card" for="settings-tts-model-control">
          <span class="settings-control-label">Selected model</span>
          <span class="settings-control-value">${
            activeOption
              ? escapeHtml(renderTtsModelOptionLabel(activeOption.profileName, activeOption.modelLabel))
              : "No configured model"
          }</span>
          <select
            id="settings-tts-model-control"
            class="settings-control-select"
            data-tts-model-select="true"
            ${disabledAttribute}
          >
            ${optionsCopy}
          </select>
        </label>
      </div>
    </section>
  `;
}

export function renderSettingsTtsVoicePanel(state: TtsVoicePanelState): string {
  const disabledAttribute = state.isBusy ? " disabled aria-disabled=\"true\"" : "";
  const errorCopy = state.error
    ? `<p class="settings-panel-error" role="alert">${escapeHtml(state.error)}</p>`
    : "";
  const optionsCopy = state.availableVoices
    .map((option) => {
      const selected = option.voiceName === state.activeVoice ? " selected" : "";
      return `<option value="${escapeHtml(option.voiceName)}"${selected}>${escapeHtml(
        renderTtsVoiceOptionLabel(option.displayLabel, option.voiceName),
      )}</option>`;
    })
    .join("");
  const modeCopy =
    state.mode === "Remote" ? "remote" : state.mode === "Local" ? "local" : "disabled";
  const activeOption = state.availableVoices.find((option) => option.voiceName === state.activeVoice);

  return `
    <section class="settings-panel" aria-labelledby="settings-tts-voice-title">
      <div class="settings-panel-copy">
        <p class="settings-panel-eyebrow">Settings</p>
        <h2 id="settings-tts-voice-title">Voice</h2>
        <p class="settings-panel-description">
          Choose the ${modeCopy} TTS voice for the current mode. Changes apply to the next utterance.
        </p>
        ${errorCopy}
      </div>
      <div class="settings-grid">
        <label class="settings-control-card" for="settings-tts-voice-control">
          <span class="settings-control-label">Selected voice</span>
          <span class="settings-control-value">${
            activeOption
              ? escapeHtml(renderTtsVoiceOptionLabel(activeOption.displayLabel, activeOption.voiceName))
              : state.activeVoice
                ? escapeHtml(state.activeVoice)
                : "No configured voice"
          }</span>
          <select
            id="settings-tts-voice-control"
            class="settings-control-select"
            data-tts-voice-select="true"
            ${disabledAttribute}
          >
            ${optionsCopy}
          </select>
        </label>
      </div>
    </section>
  `;
}

export function renderUrlInputPanel(state: UrlInputPanelState): string {
  const currentUrlCopy = state.currentUrl
    ? `<p class="url-input-current"><strong>Current URL:</strong> ${escapeHtml(state.currentUrl)}</p>`
    : '<p class="url-input-current">No page URL is loaded yet.</p>';
  const draftStatusCopy = state.hasUnsubmittedChanges
    ? '<p class="url-input-status" role="status" aria-live="polite" aria-atomic="true">Draft URL updated. Open controls can use this value next.</p>'
    : '<p class="url-input-status" role="status" aria-live="polite" aria-atomic="true">The field mirrors the current page URL until you edit it.</p>';
  const disabledAttribute =
    state.isOpening || state.isReading || state.isStopping || state.isAdvancing || state.isRewinding
      ? " disabled aria-disabled=\"true\""
      : "";
  const errorCopy = state.error
    ? `<p class="url-input-error" role="alert">${escapeHtml(state.error)}</p>`
    : "";

  return `
    <section class="url-input-panel" aria-labelledby="url-input-title">
      <div class="url-input-copy">
        <p class="url-input-eyebrow">Navigation</p>
        <h2 id="url-input-title">URL input</h2>
        <p class="url-input-description">
          Stage the next destination here. This keeps the nearby UI ready for direct navigation
          controls while voice-first command entry remains the primary path.
        </p>
        ${currentUrlCopy}
        ${draftStatusCopy}
        ${errorCopy}
      </div>
      <div class="url-input-actions">
        <label class="url-input-field" for="url-input-control">
          <span class="url-input-label">Page URL</span>
          <input
            id="url-input-control"
            class="url-input-control"
            data-url-input="true"
            type="url"
            inputmode="url"
            autocomplete="url"
            spellcheck="false"
            placeholder="https://example.com"
            value="${escapeHtml(state.draftValue)}"
            ${disabledAttribute}
          />
        </label>
        <button
          type="button"
          class="url-open-button"
          data-url-open-button="true"
          ${disabledAttribute}
        >
          ${state.isOpening ? "Opening..." : "Open"}
        </button>
        <button
          type="button"
          class="url-open-button url-read-button"
          data-url-read-button="true"
          ${disabledAttribute}
        >
          ${state.isReading ? "Reading..." : "Read"}
        </button>
        <button
          type="button"
          class="url-open-button url-stop-button"
          data-url-stop-button="true"
          ${disabledAttribute}
        >
          ${state.isStopping ? "Stopping..." : "Stop"}
        </button>
        <button
          type="button"
          class="url-open-button url-previous-button"
          data-url-previous-button="true"
          ${disabledAttribute}
        >
          ${state.isRewinding ? "Previous..." : "Previous"}
        </button>
        <button
          type="button"
          class="url-open-button url-next-button"
          data-url-next-button="true"
          ${disabledAttribute}
        >
          ${state.isAdvancing ? "Next..." : "Next"}
        </button>
      </div>
    </section>
  `;
}

export function renderStatusPanel(state: StatusPanelState): string {
  const title = state.pageTitle ?? "No page open yet";
  const region = state.currentRegionLabel ?? "No current region";
  const transcript = state.lastTranscript ?? "No spoken command captured yet";
  const errorCopy = state.error
    ? `<p class="status-panel-error" role="alert">${escapeHtml(state.error)}</p>`
    : "";
  const visiblePressed = state.browserVisibility === "Visible";
  const headlessPressed = state.browserVisibility === "Headless";
  const visibilityDisabled = state.isUpdatingVisibility ? " disabled aria-disabled=\"true\"" : "";

  return `
    <section class="status-panel" aria-labelledby="status-panel-title">
      <div class="status-panel-copy">
        <p class="status-panel-eyebrow">Runtime status</p>
        <h2 id="status-panel-title">Current browser state</h2>
        <p class="status-panel-description">
          This panel mirrors the live runtime so the nearby UI stays aligned with what the browser,
          narration, and listening tools are doing right now.
        </p>
        ${errorCopy}
      </div>
      <dl class="status-panel-grid">
        <div class="status-card status-card-wide">
          <dt>Page title</dt>
          <dd>${escapeHtml(title)}</dd>
        </div>
        <div class="status-card">
          <dt>Current region</dt>
            <dd aria-live="polite" aria-atomic="true">${escapeHtml(region)}</dd>
        </div>
        <div class="status-card status-card-wide status-card-transcript">
          <dt>Last transcript</dt>
            <dd aria-live="polite" aria-atomic="true">${escapeHtml(transcript)}</dd>
        </div>
        <div class="status-card">
          <dt>Listening</dt>
          <dd>
            <span class="status-indicator${state.listening ? " status-indicator-active" : ""}" role="status" aria-live="polite" aria-atomic="true">
              ${state.listening ? "Active" : "Idle"}
            </span>
          </dd>
        </div>
        <div class="status-card">
          <dt>Speaking</dt>
          <dd>
            <span class="status-indicator${state.speaking ? " status-indicator-active" : ""}" role="status" aria-live="polite" aria-atomic="true">
              ${state.speaking ? "Active" : "Idle"}
            </span>
          </dd>
        </div>
        <div class="status-card">
          <dt>Browser mode</dt>
          <dd>
            <span class="status-mode-label" role="status" aria-live="polite" aria-atomic="true">${escapeHtml(state.browserVisibility)}</span>
            <div class="status-toggle-group" role="group" aria-label="Browser visibility mode">
              <button
                type="button"
                class="status-toggle-button${visiblePressed ? " status-toggle-button-active" : ""}"
                data-browser-visibility-mode="Visible"
                aria-label="Browser visibility mode: Visible"
                aria-pressed="${visiblePressed}"
                ${visibilityDisabled}
              >
                Visible
              </button>
              <button
                type="button"
                class="status-toggle-button${headlessPressed ? " status-toggle-button-active" : ""}"
                data-browser-visibility-mode="Headless"
                aria-label="Browser visibility mode: Headless"
                aria-pressed="${headlessPressed}"
                ${visibilityDisabled}
              >
                Headless
              </button>
            </div>
          </dd>
        </div>
        <div class="status-card">
          <dt>History</dt>
          <dd>
            Back: ${state.canGoBack ? "Available" : "Unavailable"}.
            Forward: ${state.canGoForward ? "Available" : "Unavailable"}.
          </dd>
        </div>
      </dl>
    </section>
  `;
}
