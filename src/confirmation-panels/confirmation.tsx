import { type ReactNode } from "react";

import {
  CONFIRMATION_ERROR_BADGE_CLASS,
  CONFIRMATION_ERROR_MESSAGE_CLASS,
  resolveConfirmationErrorPresentation,
} from "../confirmation-panel-helpers.tsx";
import type { ConfirmationUiState } from "../planner-orchestration";
import { FOCUS_RING, SETTINGS_SECTION_HEADING_CLASS } from "../settings-panels/shared-controls.tsx";

export interface ConfirmationPanelHandlers {
  onRespond?: (action: "approve" | "reject", confirmationId: string) => void;
}

const CONFIRMATION_BUTTON_BASE = `appearance-none border-0 rounded-full p-[12px_18px] [font:inherit] font-semibold cursor-pointer transition-[transform,box-shadow,background-color] duration-[140ms] hover:-translate-y-px hover:shadow-[0_10px_22px_rgba(42,55,66,0.16)] focus-visible:-translate-y-px focus-visible:shadow-[0_10px_22px_rgba(42,55,66,0.16)] ${FOCUS_RING} disabled:cursor-progress disabled:translate-y-0 disabled:shadow-none disabled:opacity-[0.58]`;
const CONFIRMATION_BUTTON_APPROVE_CLASS = `${CONFIRMATION_BUTTON_BASE} bg-[var(--color-green-primary)] text-[#f6fbf8] hover:bg-[var(--color-green-dark)] focus-visible:bg-[var(--color-green-dark)] disabled:bg-[color-mix(in_srgb,var(--color-green-primary)_62%,var(--color-surface-card))]`;
const CONFIRMATION_BUTTON_REJECT_CLASS = `${CONFIRMATION_BUTTON_BASE} bg-[var(--color-surface-inner)] text-[var(--color-amber-active)] border border-[rgba(122,87,39,0.18)] hover:bg-[var(--color-amber-light)] focus-visible:bg-[var(--color-amber-light)] disabled:bg-[var(--color-surface-inner)]`;

export function renderConfirmationPanelNode(
  state: ConfirmationUiState,
  handlers?: ConfirmationPanelHandlers,
): ReactNode {
  if (state.kind !== "awaiting-confirmation") {
    return null;
  }

  const errorPresentation = resolveConfirmationErrorPresentation(state);

  return (
    <section
      className="mt-[26px] p-6 rounded-[22px] bg-gradient-to-br from-[var(--color-amber-light)] to-[var(--color-surface-card)] bg-[var(--color-surface-card)] [border:var(--card-border)] shadow-[0_18px_36px_rgba(49,63,74,0.12)] max-sm:p-5"
      aria-live="polite"
      aria-labelledby="confirmation-title"
      aria-busy={state.isSubmitting}
    >
      <div className="max-w-[62ch]">
        <p className="mb-[10px] uppercase tracking-[0.18em] text-[0.76rem] text-[var(--eyebrow-color)]">Awaiting confirmation</p>
        <h2 id="confirmation-title" className={`${SETTINGS_SECTION_HEADING_CLASS} mb-3`}>Action requires your approval.</h2>
        <p className="text-base leading-[1.6] text-[var(--color-text-secondary)]">
          {state.promptText || "Please approve or reject the pending action."}
        </p>
        {state.isSubmitting
          ? <p className="mt-3 text-[var(--color-green-active)] font-semibold" role="status">Submitting response...</p>
          : null}
        <div
          className={errorPresentation.containerClassName}
          aria-live="assertive"
          aria-atomic="true"
        >
          {state.submissionError ? [
            errorPresentation.showBadge
              ? <p className={CONFIRMATION_ERROR_BADGE_CLASS} key="badge">Cannot be retried — open Settings to check your AI assistant configuration.</p>
              : null,
            <p className={errorPresentation.titleClassName} key="title">{state.submissionError.title}</p>,
            <p className={CONFIRMATION_ERROR_MESSAGE_CLASS} key="message">{state.submissionError.message}</p>,
            <p className={errorPresentation.guidanceClassName} key="guidance">{state.submissionError.guidance}</p>,
            state.submissionError.kind === "tool-error" && errorPresentation.showMeta
              ? (
                <div className={errorPresentation.metaBlockClassName} data-confirmation-error-meta-block="true" key="meta">
                  <p className={errorPresentation.metaClassName} data-confirmation-error-meta="true">
                    {`Error code: ${state.submissionError.code}. ${state.submissionError.retryable ? "Retryable" : "Non-retryable"} backend failure.`}
                  </p>
                  <p className={errorPresentation.retryStatusClassName} data-confirmation-error-retry-status="true">
                    {state.submissionError.retryable ? "Can retry." : "Cannot retry."}
                  </p>
                </div>
              )
              : null,
          ] : null}
        </div>
      </div>
      <div className="flex flex-wrap gap-3 mt-[18px]" role="group" aria-label="Confirmation actions">
        <button
          type="button"
          className={CONFIRMATION_BUTTON_APPROVE_CLASS}
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
          className={CONFIRMATION_BUTTON_REJECT_CLASS}
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
