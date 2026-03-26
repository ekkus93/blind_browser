import type { ConfirmationUiState } from "./planner-orchestration";

export interface PushToTalkPanelState {
  enabled: boolean;
  isHolding: boolean;
  isListening: boolean;
  isBusy: boolean;
  lastTranscript: string | null;
  lastError: string | null;
}

export interface AudioControlsPanelState {
  playbackVolume: number;
  playbackSpeed: number;
  isBusy: boolean;
  error: string | null;
}

export interface TtsModelPanelState {
  mode: "Local" | "Remote" | "Disabled";
  activeProfile: string | null;
  availableProfiles: Array<{
    profileName: string;
    modelLabel: string;
  }>;
  isBusy: boolean;
  error: string | null;
}

export interface TtsVoicePanelState {
  mode: "Local" | "Remote" | "Disabled";
  activeVoice: string | null;
  availableVoices: Array<{
    voiceName: string;
    displayLabel: string;
  }>;
  isBusy: boolean;
  error: string | null;
}

export interface TtsProviderPanelState {
  activeMode: "Local" | "Remote";
  availableModes: Array<"Local" | "Remote">;
  isBusy: boolean;
  error: string | null;
}

export interface AsrProviderPanelState {
  activeMode: "Local" | "Remote";
  availableModes: Array<"Local" | "Remote">;
  isBusy: boolean;
  error: string | null;
}

export interface PlannerProviderPanelState {
  activeMode: "Remote";
  availableModes: ["Remote"] | "Remote"[];
  summary: string;
}

export interface ProviderFailoverPanelState {
  plannerAvailable: boolean;
  ttsAvailable: boolean;
  asrAvailable: boolean;
  summary: string;
}

export interface ConfirmationSettingsPanelState {
  confirmationConfidenceThreshold: number;
  allowClickWithoutConfirmation: boolean;
  alwaysConfirmSubmit: boolean;
  isBusy: boolean;
  error: string | null;
}

export interface OcrThresholdSettingsPanelState {
  sparseTextCharThreshold: number;
  sparseTextRegionThreshold: number;
  isBusy: boolean;
  error: string | null;
}

export interface UrlInputPanelState {
  draftValue: string;
  currentUrl: string | null;
  hasUnsubmittedChanges: boolean;
  isOpening: boolean;
  isReading: boolean;
  isStopping: boolean;
  isAdvancing: boolean;
  isRewinding: boolean;
  error: string | null;
}

export interface StatusPanelState {
  pageTitle: string | null;
  currentRegionLabel: string | null;
  lastTranscript: string | null;
  listening: boolean;
  speaking: boolean;
  browserVisibility: "Visible" | "Headless";
  canGoBack: boolean;
  canGoForward: boolean;
  isUpdatingVisibility: boolean;
  error: string | null;
}

function renderTtsModelOptionLabel(profileName: string, modelLabel: string): string {
  return `${modelLabel} (${profileName})`;
}

function renderTtsVoiceOptionLabel(displayLabel: string, voiceName: string): string {
  return displayLabel === voiceName ? displayLabel : `${displayLabel} (${voiceName})`;
}

function renderProviderModeLabel(mode: "Local" | "Remote"): string {
  return mode === "Local" ? "Local provider" : "Remote provider";
}

function renderFailoverAvailabilityLabel(available: boolean): string {
  return available ? "Available" : "Unavailable";
}

function renderConfirmationThresholdValue(confidenceThreshold: number): string {
  return `${Math.round(confidenceThreshold * 100)}%`;
}

function renderOcrThresholdValue(value: number): string {
  return `${value}`;
}

export function renderConfirmationPanel(state: ConfirmationUiState): string {
  if (state.kind !== "awaiting-confirmation") {
    return "";
  }

  const disabledAttribute = state.isSubmitting ? " disabled aria-disabled=\"true\"" : "";
  const statusCopy = state.isSubmitting
    ? '<p class="confirmation-status" role="status">Submitting response...</p>'
    : "";
  const errorCopy = state.submissionError
    ? `
      <div class="${renderErrorClassName(state)}" role="alert">
        ${renderErrorBadge(state)}
        <p class="confirmation-error-title">${escapeHtml(state.submissionError.title)}</p>
        <p class="confirmation-error-message">${escapeHtml(state.submissionError.message)}</p>
        <p class="confirmation-error-guidance">${escapeHtml(state.submissionError.guidance)}</p>
        ${renderErrorMeta(state)}
      </div>
    `
    : "";

  const selectedSkills = state.selectedSkills.length
    ? state.selectedSkills.map((skill) => `<li>${escapeHtml(skill)}</li>`).join("")
    : "<li>No planner skills recorded.</li>";

  const queuedSteps = state.queuedStepIds.length
    ? state.queuedStepIds.map((stepId) => `<li>${escapeHtml(stepId)}</li>`).join("")
    : "<li>No queued follow-up steps.</li>";

  const nextStep = state.nextStepId ? escapeHtml(state.nextStepId) : "No follow-up step queued.";

  return `
    <section class="confirmation-panel" aria-live="polite" aria-labelledby="confirmation-title" aria-busy="${state.isSubmitting}">
      <div class="confirmation-copy">
        <p class="confirmation-eyebrow">Awaiting confirmation</p>
        <h2 id="confirmation-title">User approval is required before the next action runs.</h2>
        <p class="confirmation-prompt">${escapeHtml(state.promptText)}</p>
        ${statusCopy}
        ${errorCopy}
      </div>

      <dl class="confirmation-meta">
        <div>
          <dt>Confirmation ID</dt>
          <dd>${escapeHtml(state.confirmationId)}</dd>
        </div>
        <div>
          <dt>Request ID</dt>
          <dd>${escapeHtml(state.requestId)}</dd>
        </div>
        <div>
          <dt>Next step</dt>
          <dd>${nextStep}</dd>
        </div>
      </dl>

      <div class="confirmation-columns">
        <div class="confirmation-card">
          <h3>Selected skills</h3>
          <ul>${selectedSkills}</ul>
        </div>
        <div class="confirmation-card">
          <h3>Queued steps</h3>
          <ul>${queuedSteps}</ul>
        </div>
      </div>

      <div class="confirmation-actions" aria-label="Confirmation actions">
        <button
          type="button"
          class="confirmation-button confirmation-button-approve"
          data-confirmation-action="approve"
          data-confirmation-id="${escapeHtml(state.confirmationId)}"
          ${disabledAttribute}
        >
          Approve action
        </button>
        <button
          type="button"
          class="confirmation-button confirmation-button-reject"
          data-confirmation-action="reject"
          data-confirmation-id="${escapeHtml(state.confirmationId)}"
          ${disabledAttribute}
        >
          Reject action
        </button>
      </div>

      <p class="confirmation-note">
        The frontend can now present approve or reject controls against this state and send the
        user response back through the Tauri confirmation command.
      </p>
    </section>
  `;
}

export function renderPushToTalkPanel(state: PushToTalkPanelState): string {
  const statusCopy = state.isHolding
    ? "Listening now. Release to transcribe and run the spoken command."
    : state.isListening && state.isBusy
      ? "Hands-free listening is active and processing the next spoken command."
      : state.isListening
        ? "Hands-free listening is active. Say a command, or say stop listening to leave hands-free mode."
        : state.isBusy
          ? "Processing the captured speech command."
          : state.enabled
            ? "Hold Space or press and hold the button to speak a command. Say start listening to keep voice input active."
            : "Push-to-talk is unavailable in the current runtime state.";
  const transcriptCopy = state.lastTranscript
    ? `<p class="push-to-talk-transcript"><strong>Last transcript:</strong> ${escapeHtml(state.lastTranscript)}</p>`
    : "";
  const errorCopy = state.lastError
    ? `<p class="push-to-talk-error" role="alert">${escapeHtml(state.lastError)}</p>`
    : "";
  const disabledAttribute = !state.enabled || state.isBusy || state.isListening
    ? " disabled aria-disabled=\"true\""
    : "";
  const buttonLabel = state.isHolding ? "Release to transcribe" : "Hold to talk";

  return `
    <section class="push-to-talk-panel" aria-labelledby="push-to-talk-title">
      <div class="push-to-talk-copy">
        <p class="push-to-talk-eyebrow">Voice input</p>
        <h2 id="push-to-talk-title">Push to talk</h2>
        <p class="push-to-talk-status" role="status">${escapeHtml(statusCopy)}</p>
        ${transcriptCopy}
        ${errorCopy}
      </div>
      <button
        type="button"
        class="push-to-talk-button${state.isHolding ? " push-to-talk-button-active" : ""}"
        data-push-to-talk-button="true"
        aria-pressed="${state.isHolding}"
        ${disabledAttribute}
      >
        ${escapeHtml(buttonLabel)}
      </button>
    </section>
  `;
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
        <h2 id="audio-controls-title">Playback controls</h2>
        <p class="audio-controls-description">
          Adjust the nearby volume and speed controls when you want spoken feedback louder, quieter,
          faster, or slower.
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
            ${busyAttribute}
          />
        </label>
      </div>
    </section>
  `;
}

export function renderSettingsPlannerProviderPanel(state: PlannerProviderPanelState): string {
  const optionsCopy = state.availableModes
    .map((mode) => `<option value="${escapeHtml(mode)}" selected>${escapeHtml(renderProviderModeLabel(mode))}</option>`)
    .join("");

  return `
    <section class="settings-panel" aria-labelledby="settings-planner-provider-title">
      <div class="settings-panel-copy">
        <p class="settings-panel-eyebrow">Settings</p>
        <h2 id="settings-planner-provider-title">Planner provider selection</h2>
        <p class="settings-panel-description">
          The planner currently runs through configured remote profiles only. Local planner mode is
          not available in the current runtime, so this panel reflects the active remote-only configuration.
        </p>
        <p class="settings-panel-description">${escapeHtml(state.summary)}</p>
      </div>
      <div class="settings-grid">
        <label class="settings-control-card" for="settings-planner-provider-control">
          <span class="settings-control-label">Planner provider</span>
          <span class="settings-control-value">${escapeHtml(renderProviderModeLabel(state.activeMode))}</span>
          <select
            id="settings-planner-provider-control"
            class="settings-control-select"
            data-planner-provider-select="true"
            disabled
            aria-disabled="true"
          >
            ${optionsCopy}
          </select>
        </label>
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
        <h2 id="settings-provider-failover-title">Provider failover</h2>
        <p class="settings-panel-description">
          Automatic remote-to-local provider failover is not currently available in the live runtime.
          These toggles stay read-only until real failover support is implemented.
        </p>
        <p class="settings-panel-description">${escapeHtml(state.summary)}</p>
      </div>
      <div class="settings-grid">
        ${renderFailoverCard("planner", "Planner failover", state.plannerAvailable)}
        ${renderFailoverCard("tts", "TTS failover", state.ttsAvailable)}
        ${renderFailoverCard("asr", "ASR failover", state.asrAvailable)}
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
        <h2 id="settings-confirmation-title">Confirmation behavior</h2>
        <p class="settings-panel-description">
          Adjust how confidently the runtime can resolve a click before it asks for confirmation.
          Form submission still always requires confirmation.
        </p>
        ${errorCopy}
      </div>
      <div class="settings-grid">
        <label class="settings-control-card" for="settings-confirmation-threshold-control">
          <span class="settings-control-label">Click confirmation threshold</span>
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
            ${disabledAttribute}
          />
        </label>
        <label class="settings-control-card" for="settings-click-without-confirmation-toggle">
          <span class="settings-control-label">Allow confident clicks without confirmation</span>
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
        <h2 id="settings-ocr-thresholds-title">OCR thresholds</h2>
        <p class="settings-panel-description">
          Adjust when sparse DOM extraction should trigger OCR fallback. Existing OCR fallback
          toggles remain unchanged by this panel.
        </p>
        ${errorCopy}
      </div>
      <div class="settings-grid">
        <label class="settings-control-card" for="settings-ocr-char-threshold-control">
          <span class="settings-control-label">Sparse text character threshold</span>
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
          <span class="settings-control-label">Sparse text region threshold</span>
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
        <h2 id="settings-asr-provider-title">ASR provider selection</h2>
        <p class="settings-panel-description">
          Choose whether spoken command transcription uses the configured local or remote ASR provider.
          Changes apply to the next listening request and preserve the current provider profiles.
        </p>
        ${errorCopy}
      </div>
      <div class="settings-grid">
        <label class="settings-control-card" for="settings-asr-provider-control">
          <span class="settings-control-label">ASR provider</span>
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
        <h2 id="settings-tts-provider-title">TTS provider selection</h2>
        <p class="settings-panel-description">
          Choose whether spoken output uses the configured local or remote TTS provider. Changes
          apply to the next utterance and preserve the current provider profiles.
        </p>
        ${errorCopy}
      </div>
      <div class="settings-grid">
        <label class="settings-control-card" for="settings-tts-provider-control">
          <span class="settings-control-label">TTS provider</span>
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
        <h2 id="settings-tts-model-title">TTS model selection</h2>
        <p class="settings-panel-description">
          Choose from the configured ${modeCopy} TTS models for the current TTS mode. Changes apply
          to the next utterance and persist across app restarts.
        </p>
        ${errorCopy}
      </div>
      <div class="settings-grid">
        <label class="settings-control-card" for="settings-tts-model-control">
          <span class="settings-control-label">Configured TTS model</span>
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
        <h2 id="settings-tts-voice-title">Voice selection</h2>
        <p class="settings-panel-description">
          Choose from the configured ${modeCopy} TTS voices for the current TTS mode. Changes apply
          to the next utterance and persist across app restarts.
        </p>
        ${errorCopy}
      </div>
      <div class="settings-grid">
        <label class="settings-control-card" for="settings-tts-voice-control">
          <span class="settings-control-label">Configured TTS voice</span>
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
    ? '<p class="url-input-status" role="status">Draft URL updated. Open controls can use this value next.</p>'
    : '<p class="url-input-status" role="status">The field mirrors the current page URL until you edit it.</p>';
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
          <dd>${escapeHtml(region)}</dd>
        </div>
        <div class="status-card status-card-wide status-card-transcript">
          <dt>Last transcript</dt>
          <dd>${escapeHtml(transcript)}</dd>
        </div>
        <div class="status-card">
          <dt>Listening</dt>
          <dd>
            <span class="status-indicator${state.listening ? " status-indicator-active" : ""}">
              ${state.listening ? "Active" : "Idle"}
            </span>
          </dd>
        </div>
        <div class="status-card">
          <dt>Speaking</dt>
          <dd>
            <span class="status-indicator${state.speaking ? " status-indicator-active" : ""}">
              ${state.speaking ? "Active" : "Idle"}
            </span>
          </dd>
        </div>
        <div class="status-card">
          <dt>Browser mode</dt>
          <dd>
            <span class="status-mode-label">${escapeHtml(state.browserVisibility)}</span>
            <div class="status-toggle-group" aria-label="Browser visibility mode">
              <button
                type="button"
                class="status-toggle-button${visiblePressed ? " status-toggle-button-active" : ""}"
                data-browser-visibility-mode="Visible"
                aria-pressed="${visiblePressed}"
                ${visibilityDisabled}
              >
                Visible
              </button>
              <button
                type="button"
                class="status-toggle-button${headlessPressed ? " status-toggle-button-active" : ""}"
                data-browser-visibility-mode="Headless"
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

function renderErrorClassName(
  state: Extract<ConfirmationUiState, { kind: "awaiting-confirmation" }>,
): string {
  const classNames = ["confirmation-error"];

  if (!state.submissionError) {
    return classNames.join(" ");
  }

  if (state.submissionError.kind === "transport-error") {
    classNames.push("confirmation-error-transport");
    return classNames.join(" ");
  }

  classNames.push("confirmation-error-tool");
  classNames.push(
    state.submissionError.retryable
      ? "confirmation-error-tool-retryable"
      : "confirmation-error-tool-hard-stop",
  );

  return classNames.join(" ");
}

function renderErrorBadge(
  state: Extract<ConfirmationUiState, { kind: "awaiting-confirmation" }>,
): string {
  if (
    !state.submissionError ||
    state.submissionError.kind !== "tool-error" ||
    state.submissionError.retryable
  ) {
    return "";
  }

  return '<p class="confirmation-error-badge">Requires planner change</p>';
}

function renderErrorMeta(state: Extract<ConfirmationUiState, { kind: "awaiting-confirmation" }>): string {
  if (!state.submissionError || state.submissionError.kind !== "tool-error") {
    return "";
  }

  const retryableLabel = state.submissionError.retryable ? "Retryable" : "Non-retryable";
  const retryStatus = state.submissionError.retryable ? "Can retry." : "Cannot retry.";
  return `
    <div class="confirmation-error-meta-block">
      <p class="confirmation-error-meta">
        Error code: ${escapeHtml(state.submissionError.code)}. ${retryableLabel} backend failure.
      </p>
      <p class="confirmation-error-retry-status">${retryStatus}</p>
    </div>
  `;
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/\"/g, "&quot;")
    .replace(/'/g, "&#39;");
}
