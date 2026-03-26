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

export interface UrlInputPanelState {
  draftValue: string;
  currentUrl: string | null;
  hasUnsubmittedChanges: boolean;
  isOpening: boolean;
  isReading: boolean;
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
    : state.isBusy
      ? "Processing the captured speech command."
      : state.isListening
        ? "Listening is active."
        : state.enabled
          ? "Hold Space or press and hold the button to speak a command."
          : "Push-to-talk is unavailable in the current runtime state.";
  const transcriptCopy = state.lastTranscript
    ? `<p class="push-to-talk-transcript"><strong>Last transcript:</strong> ${escapeHtml(state.lastTranscript)}</p>`
    : "";
  const errorCopy = state.lastError
    ? `<p class="push-to-talk-error" role="alert">${escapeHtml(state.lastError)}</p>`
    : "";
  const disabledAttribute = !state.enabled || state.isBusy ? " disabled aria-disabled=\"true\"" : "";
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

export function renderUrlInputPanel(state: UrlInputPanelState): string {
  const currentUrlCopy = state.currentUrl
    ? `<p class="url-input-current"><strong>Current URL:</strong> ${escapeHtml(state.currentUrl)}</p>`
    : '<p class="url-input-current">No page URL is loaded yet.</p>';
  const draftStatusCopy = state.hasUnsubmittedChanges
    ? '<p class="url-input-status" role="status">Draft URL updated. Open controls can use this value next.</p>'
    : '<p class="url-input-status" role="status">The field mirrors the current page URL until you edit it.</p>';
  const disabledAttribute =
    state.isOpening || state.isReading ? " disabled aria-disabled=\"true\"" : "";
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
