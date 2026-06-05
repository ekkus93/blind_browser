import { createElement, type ReactNode } from "react";

import {
  renderConfirmationErrorBadge,
  renderConfirmationErrorClassName,
} from "../confirmation-panel-helpers.ts";
import type { ConfirmationUiState } from "../planner-orchestration";

const h = createElement;

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

  return h(
    "section",
    {
      className: "confirmation-panel",
      "aria-live": "polite",
      "aria-labelledby": "confirmation-title",
      "aria-busy": String(state.isSubmitting),
    },
    h(
      "div",
      { className: "confirmation-copy" },
      h("p", { className: "confirmation-eyebrow" }, "Awaiting confirmation"),
      h("h2", { id: "confirmation-title" }, "Action requires your approval."),
      h("p", { className: "confirmation-prompt" }, state.promptText || "Please approve or reject the pending action."),
      state.isSubmitting
        ? h("p", { className: "confirmation-status", role: "status" }, "Submitting response...")
        : null,
      h(
        "div",
        {
          className: renderConfirmationErrorClassName(state),
          "aria-live": "assertive",
          "aria-atomic": "true",
        },
        state.submissionError
          ? [
            renderConfirmationErrorBadge(state)
              ? h("p", { className: "confirmation-error-badge", key: "badge" }, "Cannot be retried — open Settings to check your AI assistant configuration.")
              : null,
            h("p", { className: "confirmation-error-title", key: "title" }, state.submissionError.title),
            h("p", { className: "confirmation-error-message", key: "message" }, state.submissionError.message),
            h("p", { className: "confirmation-error-guidance", key: "guidance" }, state.submissionError.guidance),
            state.submissionError.kind === "tool-error"
              ? h(
                "div",
                { className: "confirmation-error-meta-block", key: "meta" },
                h(
                  "p",
                  { className: "confirmation-error-meta" },
                  `Error code: ${state.submissionError.code}. ${state.submissionError.retryable ? "Retryable" : "Non-retryable"} backend failure.`,
                ),
                h(
                  "p",
                  { className: "confirmation-error-retry-status" },
                  state.submissionError.retryable ? "Can retry." : "Cannot retry.",
                ),
              )
              : null,
          ]
          : null,
      ),
    ),
    h(
      "div",
      { className: "confirmation-actions", role: "group", "aria-label": "Confirmation actions" },
      h(
        "button",
        {
          type: "button",
          className: "confirmation-button confirmation-button-approve",
          "data-confirmation-action": "approve",
          "data-confirmation-id": state.confirmationId,
          disabled: state.isSubmitting || undefined,
          "aria-disabled": state.isSubmitting ? "true" : undefined,
          onClick: handlers?.onRespond
            ? () => {
              handlers.onRespond?.("approve", state.confirmationId);
            }
            : undefined,
        },
        "Approve action",
      ),
      h(
        "button",
        {
          type: "button",
          className: "confirmation-button confirmation-button-reject",
          "data-confirmation-action": "reject",
          "data-confirmation-id": state.confirmationId,
          disabled: state.isSubmitting || undefined,
          "aria-disabled": state.isSubmitting ? "true" : undefined,
          onClick: handlers?.onRespond
            ? () => {
              handlers.onRespond?.("reject", state.confirmationId);
            }
            : undefined,
        },
        "Reject action",
      ),
    ),
  );
}