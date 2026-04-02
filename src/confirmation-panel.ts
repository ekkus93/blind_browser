import {
  escapeHtml,
  renderConfirmationErrorBadge,
  renderConfirmationErrorClassName,
  renderConfirmationErrorMeta,
} from "./confirmation-panel-helpers.ts";
export type {
  AudioControlsPanelState,
  AsrProviderPanelState,
  ConfirmationSettingsPanelState,
  LocalAsrModelPanelState,
  LocalTtsModelPanelState,
  ModelManagementPanelState,
  OcrThresholdSettingsPanelState,
  PlannerProviderPanelState,
  ProviderFailoverPanelState,
  PushToTalkPanelState,
  RemoteAsrPanelState,
  RemotePlannerPanelState,
  RemoteTtsPanelState,
  SettingsGuidancePanelAction,
  SettingsGuidancePanelState,
  StatusPanelAgentStateLike,
  StatusPanelState,
  TtsModelPanelState,
  TtsProviderPanelState,
  TtsVoicePanelState,
  UrlInputPanelState,
} from "./panel-types.ts";
export {
  renderAudioControlsPanel,
  renderSettingsAsrProviderPanel,
  renderSettingsConfirmationPanel,
  renderSettingsGuidancePanel,
  renderSettingsLocalAsrModelPanel,
  renderSettingsLocalTtsModelPanel,
  renderSettingsModelManagementPanel,
  renderSettingsOcrThresholdPanel,
  renderSettingsProviderFailoverPanel,
  renderSettingsPlannerProviderPanel,
  renderSettingsRemoteAsrPanel,
  renderSettingsRemotePlannerPanel,
  renderSettingsRemoteTtsPanel,
  renderSettingsTtsProviderPanel,
  renderSettingsTtsModelPanel,
  renderSettingsTtsVoicePanel,
  renderSettingsSpeedPanel,
  renderSettingsVolumePanel,
  renderStatusPanel,
  renderUrlInputPanel,
  statusPanelStateFromAgentState,
} from "./settings-status-panels.ts";
import type { ConfirmationUiState } from "./planner-orchestration";
import type { PushToTalkPanelState } from "./panel-types.ts";

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
      <div class="${renderConfirmationErrorClassName(state)}" role="alert">
        ${renderConfirmationErrorBadge(state)}
        <p class="confirmation-error-title">${escapeHtml(state.submissionError.title)}</p>
        <p class="confirmation-error-message">${escapeHtml(state.submissionError.message)}</p>
        <p class="confirmation-error-guidance">${escapeHtml(state.submissionError.guidance)}</p>
        ${renderConfirmationErrorMeta(state)}
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
