import { createElement, type ReactNode } from "react";

import type { PushToTalkPanelState } from "../panel-types.ts";

const h = createElement;

export function renderPushToTalkPanelNode(state: PushToTalkPanelState): ReactNode {
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
  const buttonLabel = state.isHolding ? "Release to transcribe" : "Hold to talk";

  return h(
    "section",
    { className: "push-to-talk-panel", "aria-labelledby": "push-to-talk-title" },
    h(
      "div",
      { className: "push-to-talk-copy" },
      h("p", { className: "push-to-talk-eyebrow" }, "Voice input"),
      h("h2", { id: "push-to-talk-title" }, "Push to talk"),
      h("p", { className: "push-to-talk-status", role: "status" }, statusCopy),
      state.lastTranscript
        ? h(
          "p",
          { className: "push-to-talk-transcript" },
          h("strong", null, "Last transcript:"),
          ` ${state.lastTranscript}`,
        )
        : null,
      state.lastError
        ? h("p", { className: "push-to-talk-error", role: "alert" }, state.lastError)
        : null,
    ),
    h(
      "button",
      {
        type: "button",
        className: `push-to-talk-button${state.isHolding ? " push-to-talk-button-active" : ""}`,
        "data-push-to-talk-button": "true",
        "aria-pressed": String(state.isHolding),
        disabled: (!state.enabled || state.isBusy || state.isListening) || undefined,
        "aria-disabled": (!state.enabled || state.isBusy || state.isListening) ? "true" : undefined,
      },
      buttonLabel,
    ),
  );
}