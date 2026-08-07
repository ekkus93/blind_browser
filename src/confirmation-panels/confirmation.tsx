import { type ReactNode } from "react";

import {
  renderConfirmationErrorBadge,
  renderConfirmationErrorClassName,
} from "../confirmation-panel-helpers.tsx";
import type { ConfirmationUiState } from "../planner-orchestration";

export interface ConfirmationPanelHandlers {
  onRespond?: (action: "approve" | "reject", confirmationId: string) => void;
}

export function renderConfirmationPanelNode(
  state: ConfirmationUiState,
  handlers?: ConfirmationPanelHandlers,
): ReactNode {
  if (state.kind !== "awaiting-confirmation") {
    return null;
  }

  return (
    <section
      className="confirmation-panel"
      aria-live="polite"
      aria-labelledby="confirmation-title"
      aria-busy={state.isSubmitting}
    >
      <div className="confirmation-copy">
        <p className="confirmation-eyebrow">Awaiting confirmation</p>
        <h2 id="confirmation-title">Action requires your approval.</h2>
        <p className="confirmation-prompt">
          {state.promptText || "Please approve or reject the pending action."}
        </p>
        {state.isSubmitting
          ? <p className="confirmation-status" role="status">Submitting response...</p>
          : null}
        <div
          className={renderConfirmationErrorClassName(state)}
          aria-live="assertive"
          aria-atomic="true"
        >
          {state.submissionError ? [
            renderConfirmationErrorBadge(state)
              ? <p className="confirmation-error-badge" key="badge">Cannot be retried — open Settings to check your AI assistant configuration.</p>
              : null,
            <p className="confirmation-error-title" key="title">{state.submissionError.title}</p>,
            <p className="confirmation-error-message" key="message">{state.submissionError.message}</p>,
            <p className="confirmation-error-guidance" key="guidance">{state.submissionError.guidance}</p>,
            state.submissionError.kind === "tool-error"
              ? (
                <div className="confirmation-error-meta-block" key="meta">
                  <p className="confirmation-error-meta">
                    {`Error code: ${state.submissionError.code}. ${state.submissionError.retryable ? "Retryable" : "Non-retryable"} backend failure.`}
                  </p>
                  <p className="confirmation-error-retry-status">
                    {state.submissionError.retryable ? "Can retry." : "Cannot retry."}
                  </p>
                </div>
              )
              : null,
          ] : null}
        </div>
      </div>
      <div className="confirmation-actions" role="group" aria-label="Confirmation actions">
        <button
          type="button"
          className="confirmation-button confirmation-button-approve"
          data-confirmation-action="approve"
          data-confirmation-id={state.confirmationId}
          disabled={state.isSubmitting || undefined}
          aria-disabled={state.isSubmitting ? "true" : undefined}
          onClick={handlers?.onRespond ? () => { handlers.onRespond?.("approve", state.confirmationId); } : undefined}
        >
          Approve action
        </button>
        <button
          type="button"
          className="confirmation-button confirmation-button-reject"
          data-confirmation-action="reject"
          data-confirmation-id={state.confirmationId}
          disabled={state.isSubmitting || undefined}
          aria-disabled={state.isSubmitting ? "true" : undefined}
          onClick={handlers?.onRespond ? () => { handlers.onRespond?.("reject", state.confirmationId); } : undefined}
        >
          Reject action
        </button>
      </div>
    </section>
  );
}
