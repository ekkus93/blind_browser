import { createElement, type ReactNode } from "react";

import {
  renderConfirmationErrorBadge,
  renderConfirmationErrorClassName,
} from "../confirmation-panel-helpers.ts";
import type { ConfirmationUiState } from "../planner-orchestration";

const h = createElement;

export function renderConfirmationPanelNode(state: ConfirmationUiState): ReactNode {
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
      h("h2", { id: "confirmation-title" }, "User approval is required before the next action runs."),
      h("p", { className: "confirmation-prompt" }, state.promptText),
      state.isSubmitting
        ? h("p", { className: "confirmation-status", role: "status" }, "Submitting response...")
        : null,
      state.submissionError
        ? h(
          "div",
          { className: renderConfirmationErrorClassName(state), role: "alert" },
          renderConfirmationErrorBadge(state)
            ? h("p", { className: "confirmation-error-badge" }, "Requires planner change")
            : null,
          h("p", { className: "confirmation-error-title" }, state.submissionError.title),
          h("p", { className: "confirmation-error-message" }, state.submissionError.message),
          h("p", { className: "confirmation-error-guidance" }, state.submissionError.guidance),
          state.submissionError.kind === "tool-error"
            ? h(
              "div",
              { className: "confirmation-error-meta-block" },
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
        )
        : null,
    ),
    h(
      "dl",
      { className: "confirmation-meta" },
      h("div", null, h("dt", null, "Confirmation ID"), h("dd", null, state.confirmationId)),
      h("div", null, h("dt", null, "Request ID"), h("dd", null, state.requestId)),
      h("div", null, h("dt", null, "Next step"), h("dd", null, state.nextStepId ?? "No follow-up step queued.")),
    ),
    h(
      "div",
      { className: "confirmation-columns" },
      h(
        "div",
        { className: "confirmation-card" },
        h("h3", null, "Selected skills"),
        h(
          "ul",
          null,
          ...(state.selectedSkills.length
            ? state.selectedSkills.map((skill) => h("li", { key: skill }, skill))
            : [h("li", { key: "empty-skills" }, "No planner skills recorded.")]),
        ),
      ),
      h(
        "div",
        { className: "confirmation-card" },
        h("h3", null, "Queued steps"),
        h(
          "ul",
          null,
          ...(state.queuedStepIds.length
            ? state.queuedStepIds.map((stepId) => h("li", { key: stepId }, stepId))
            : [h("li", { key: "empty-steps" }, "No queued follow-up steps.")]),
        ),
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
        },
        "Reject action",
      ),
    ),
    h(
      "p",
      { className: "confirmation-note" },
      "The frontend can now present approve or reject controls against this state and send the user response back through the Tauri confirmation command.",
    ),
  );
}