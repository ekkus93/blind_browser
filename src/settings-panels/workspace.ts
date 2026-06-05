import { createElement, type ReactNode } from "react";

import type {
  StatusPanelAgentStateLike,
  StatusPanelState,
  UrlInputPanelState,
} from "../panel-types.ts";

const h = createElement;

type UrlActionIcon = "open" | "read" | "stop" | "previous" | "next";

function renderIcon(icon: UrlActionIcon) {
  const pathByIcon = {
    open: "M14 3h7v7a1 1 0 1 1-2 0V6.41l-8.29 8.3a1 1 0 0 1-1.42-1.42L17.59 5H14a1 1 0 1 1 0-2ZM5 5h6a1 1 0 1 1 0 2H6v11h11v-5a1 1 0 1 1 2 0v6a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1Z",
    read: "M8 5.14v13.72a1 1 0 0 0 1.53.85l10.3-6.86a1 1 0 0 0 0-1.7L9.53 4.29A1 1 0 0 0 8 5.14Z",
    stop: "M7 7h10v10H7z",
    previous: "M17.71 6.29a1 1 0 0 1 0 1.42L13.41 12l4.3 4.29a1 1 0 1 1-1.42 1.42l-5-5a1 1 0 0 1 0-1.42l5-5a1 1 0 0 1 1.42 0ZM8 6a1 1 0 0 1 1 1v10a1 1 0 1 1-2 0V7a1 1 0 0 1 1-1Z",
    next: "M6.29 6.29a1 1 0 0 1 1.42 0l5 5a1 1 0 0 1 0 1.42l-5 5a1 1 0 1 1-1.42-1.42L10.59 12l-4.3-4.29a1 1 0 0 1 0-1.42ZM16 6a1 1 0 0 1 1 1v10a1 1 0 1 1-2 0V7a1 1 0 0 1 1-1Z",
  } as const;

  return h(
    "svg",
    {
      className: "icon-button-glyph",
      viewBox: "0 0 24 24",
      "aria-hidden": true,
      focusable: "false",
    },
    h("path", { d: pathByIcon[icon], fill: "currentColor" }),
  );
}

function renderUrlActionButton(
  className: string,
  dataAttribute: string,
  label: string,
  icon: UrlActionIcon,
  isBusy: boolean,
  onClick?: () => void,
) {
  return h(
    "button",
    {
      type: "button",
      className,
      [dataAttribute]: "true",
      "aria-label": label,
      title: label,
      disabled: isBusy || undefined,
      "aria-disabled": isBusy ? "true" : undefined,
      onClick,
    },
    renderIcon(icon),
  );
}

export interface UrlInputPanelHandlers {
  onDraftInput?: (value: string) => void;
  onOpen?: () => void;
  onRead?: () => void;
  onStop?: () => void;
  onPrevious?: () => void;
  onNext?: () => void;
}

export interface StatusPanelHandlers {
  onSetBrowserVisibility?: (mode: "Visible" | "Headless") => void;
}

export function statusPanelStateFromAgentState(
  agentState: StatusPanelAgentStateLike,
): StatusPanelState {
  return {
    pageTitle: agentState.title ?? agentState.url,
    currentRegionLabel: agentState.narration_cursor
      ? `Section ${agentState.narration_cursor.node_index + 1}`
      : null,
    lastTranscript: agentState.last_transcript,
    listening: agentState.listening_state.is_listening,
    speaking: agentState.speaking,
    browserVisibility: agentState.browser_visibility,
    canGoBack: agentState.browser_history.can_go_back,
    canGoForward: agentState.browser_history.can_go_forward,
    isUpdatingVisibility: false,
    error: null,
  };
}

export function renderUrlInputPanelNode(
  state: UrlInputPanelState,
  handlers?: UrlInputPanelHandlers,
): ReactNode {
  const actionsDisabled = state.isOpening || state.isReading || state.isStopping || state.isAdvancing || state.isRewinding;

  return h(
    "section",
    { className: "url-input-panel", "aria-label": "Page navigation" },
    h(
      "div",
      { className: "url-input-actions" },
      h(
        "div",
        { className: "url-input-row url-input-row-primary" },
        h("input", {
          id: "url-input-control",
          className: "url-input-control",
          "data-url-input": "true",
          "aria-label": "Page URL",
          type: "url",
          inputMode: "url",
          autoComplete: "url",
          spellCheck: false,
          placeholder: "https://example.com",
          value: state.draftValue,
          disabled: actionsDisabled || undefined,
          "aria-disabled": actionsDisabled ? "true" : undefined,
          onChange: handlers?.onDraftInput
            ? (event: { currentTarget: { value: string } }) => {
              handlers.onDraftInput?.(event.currentTarget.value);
            }
            : undefined,
        }),
        renderUrlActionButton(
          "url-action-button url-open-button",
          "data-url-open-button",
          state.isOpening ? "Opening" : "Open",
          "open",
          actionsDisabled,
          handlers?.onOpen,
        ),
      ),
      h(
        "div",
        { className: "url-input-row url-input-row-secondary", role: "group", "aria-label": "Page reading controls" },
        renderUrlActionButton(
          "url-action-button url-read-button",
          "data-url-read-button",
          state.isReading ? "Reading" : "Read",
          "read",
          actionsDisabled,
          handlers?.onRead,
        ),
        renderUrlActionButton(
          "url-action-button url-stop-button",
          "data-url-stop-button",
          state.isStopping ? "Stopping" : "Stop",
          "stop",
          actionsDisabled,
          handlers?.onStop,
        ),
        renderUrlActionButton(
          "url-action-button url-previous-button",
          "data-url-previous-button",
          state.isRewinding ? "Moving to previous section" : "Previous",
          "previous",
          actionsDisabled,
          handlers?.onPrevious,
        ),
        renderUrlActionButton(
          "url-action-button url-next-button",
          "data-url-next-button",
          state.isAdvancing ? "Moving to next section" : "Next",
          "next",
          actionsDisabled,
          handlers?.onNext,
        ),
      ),
      state.error ? h("p", { className: "url-input-error", role: "alert" }, state.error) : null,
    ),
  );
}

export function renderStatusPanelNode(
  state: StatusPanelState,
  handlers?: StatusPanelHandlers,
): ReactNode {
  const title = state.pageTitle ?? "No page open yet";
  const region = state.currentRegionLabel ?? "No current section";
  const transcript = state.lastTranscript ?? "No spoken command captured yet";
  const visiblePressed = state.browserVisibility === "Visible";
  const headlessPressed = state.browserVisibility === "Headless";
  const isFirstLoad = state.pageTitle == null;

  return h(
    "section",
    { className: "status-panel", "aria-labelledby": "status-panel-title" },
    h(
      "div",
      { className: "status-panel-copy" },
      h("p", { className: "status-panel-eyebrow" }, "Runtime status"),
      h("h2", { id: "status-panel-title" }, "Current browser state"),
      state.error ? h("p", { className: "status-panel-error", role: "alert" }, state.error) : null,
    ),
    isFirstLoad
      ? h(
        "p",
        { className: "status-panel-empty", "aria-live": "polite" },
        "Hold the Talk button and say a URL or command to get started.",
      )
      : h(
        "dl",
        { className: "status-panel-grid" },
        h("div", { className: "status-card status-card-wide" }, h("dt", null, "Page title"), h("dd", null, title)),
        h("div", { className: "status-card" }, h("dt", null, "Current section"), h("dd", { "aria-live": "polite", "aria-atomic": "true" }, region)),
        h("div", { className: "status-card status-card-wide status-card-transcript" }, h("dt", null, "Last transcript"), h("dd", { "aria-live": "polite", "aria-atomic": "true" }, transcript)),
        h(
          "div",
          { className: "status-card" },
          h("dt", null, "Browser mode"),
          h(
            "dd",
            null,
            h("span", { className: "status-mode-label", role: "status", "aria-live": "polite", "aria-atomic": "true" }, state.browserVisibility),
            h(
              "div",
              { className: "status-toggle-group", role: "group", "aria-label": "Browser visibility mode" },
              h("button", { type: "button", className: `status-toggle-button${visiblePressed ? " status-toggle-button-active" : ""}`, "data-browser-visibility-mode": "Visible", "aria-label": "Browser visibility mode: Visible", "aria-pressed": String(visiblePressed), disabled: state.isUpdatingVisibility || undefined, "aria-disabled": state.isUpdatingVisibility ? "true" : undefined, onClick: handlers?.onSetBrowserVisibility ? () => { handlers.onSetBrowserVisibility?.("Visible"); } : undefined }, "Visible"),
              h("button", { type: "button", className: `status-toggle-button${headlessPressed ? " status-toggle-button-active" : ""}`, "data-browser-visibility-mode": "Headless", "aria-label": "Browser visibility mode: Headless", "aria-pressed": String(headlessPressed), disabled: state.isUpdatingVisibility || undefined, "aria-disabled": state.isUpdatingVisibility ? "true" : undefined, onClick: handlers?.onSetBrowserVisibility ? () => { handlers.onSetBrowserVisibility?.("Headless"); } : undefined }, "Headless"),
            ),
          ),
        ),
        h("div", { className: "status-card" }, h("dt", null, "History"), h("dd", null, `Back: ${state.canGoBack ? "Available" : "Unavailable"}. Forward: ${state.canGoForward ? "Available" : "Unavailable"}.`)),
      ),
  );
}