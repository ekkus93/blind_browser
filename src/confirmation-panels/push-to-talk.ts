import { createElement, type ReactNode } from "react";

import type { PushToTalkPanelState } from "../panel-types.ts";

const h = createElement;

function renderMicrophoneIcon() {
  return h(
    "svg",
    {
      className: "icon-button-glyph",
      viewBox: "0 0 24 24",
      "aria-hidden": true,
      focusable: "false",
    },
    h("path", {
      d: "M12 15a3 3 0 0 0 3-3V7a3 3 0 1 0-6 0v5a3 3 0 0 0 3 3Zm5-3a1 1 0 1 1 2 0 7 7 0 0 1-6 6.93V21h3a1 1 0 1 1 0 2H8a1 1 0 1 1 0-2h3v-2.07A7 7 0 0 1 5 12a1 1 0 1 1 2 0 5 5 0 0 0 10 0Z",
      fill: "currentColor",
    }),
  );
}

export interface PushToTalkPanelHandlers {
  onPointerDown?: () => void;
}

export function renderPushToTalkPanelNode(
  state: PushToTalkPanelState,
  handlers?: PushToTalkPanelHandlers,
): ReactNode {
  const buttonLabel = state.isHolding
    ? "Release to transcribe"
    : state.isListening && state.isBusy
      ? "Listening busy"
      : state.isListening
        ? "Hands-free listening active"
        : state.isBusy
          ? "Processing speech"
          : state.enabled
            ? "Talk"
            : "Talk unavailable";

  return h(
    "section",
    { className: "push-to-talk-panel", "aria-label": "Talk control" },
    h(
      "button",
      {
        type: "button",
        className: `push-to-talk-button${state.isHolding ? " push-to-talk-button-active" : ""}`,
        "data-push-to-talk-button": "true",
        "aria-label": buttonLabel,
        "aria-pressed": String(state.isHolding),
        disabled: (!state.enabled || state.isBusy || state.isListening) || undefined,
        "aria-disabled": (!state.enabled || state.isBusy || state.isListening) ? "true" : undefined,
        onPointerDown: handlers?.onPointerDown
          ? (event: { button: number; preventDefault: () => void }) => {
            if (event.button !== 0) {
              return;
            }

            event.preventDefault();
            handlers.onPointerDown?.();
          }
          : undefined,
      },
      renderMicrophoneIcon(),
    ),
    h(
      "p",
      {
        className: "push-to-talk-hint",
        "aria-hidden": "true",
      },
      state.isHolding
        ? "Release to transcribe"
        : state.isListening
          ? "Listening…"
          : state.isBusy
            ? "Processing…"
            : "Hold to talk",
    ),
    state.lastError
      ? h("span", { className: "sr-only", role: "alert" }, state.lastError)
      : null,
  );
}