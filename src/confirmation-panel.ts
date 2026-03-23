import type { ConfirmationUiState } from "./planner-orchestration";

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