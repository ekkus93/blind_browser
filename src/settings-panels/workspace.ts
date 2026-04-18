import { createElement, type ReactNode } from "react";

import type {
  StatusPanelAgentStateLike,
  StatusPanelState,
  UrlInputPanelState,
} from "../panel-types.ts";

const h = createElement;

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
      ? `Region ${agentState.narration_cursor.node_index + 1}`
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
    { className: "url-input-panel", "aria-labelledby": "url-input-title" },
    h(
      "div",
      { className: "url-input-copy" },
      h("p", { className: "url-input-eyebrow" }, "Navigation"),
      h("h2", { id: "url-input-title" }, "URL input"),
      h(
        "p",
        { className: "url-input-description" },
        "Stage the next destination here. This keeps the nearby UI ready for direct navigation controls while voice-first command entry remains the primary path.",
      ),
      state.currentUrl
        ? h("p", { className: "url-input-current" }, h("strong", null, "Current URL:"), ` ${state.currentUrl}`)
        : h("p", { className: "url-input-current" }, "No page URL is loaded yet."),
      h(
        "p",
        { className: "url-input-status", role: "status", "aria-live": "polite", "aria-atomic": "true" },
        state.hasUnsubmittedChanges
          ? "Draft URL updated. Open controls can use this value next."
          : "The field mirrors the current page URL until you edit it.",
      ),
      state.error ? h("p", { className: "url-input-error", role: "alert" }, state.error) : null,
    ),
    h(
      "div",
      { className: "url-input-actions" },
      h(
        "label",
        { className: "url-input-field", htmlFor: "url-input-control" },
        h("span", { className: "url-input-label" }, "Page URL"),
        h("input", {
          id: "url-input-control",
          className: "url-input-control",
          "data-url-input": "true",
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
      ),
      h("button", { type: "button", className: "url-open-button", "data-url-open-button": "true", disabled: actionsDisabled || undefined, "aria-disabled": actionsDisabled ? "true" : undefined, onClick: handlers?.onOpen }, state.isOpening ? "Opening..." : "Open"),
      h("button", { type: "button", className: "url-open-button url-read-button", "data-url-read-button": "true", disabled: actionsDisabled || undefined, "aria-disabled": actionsDisabled ? "true" : undefined, onClick: handlers?.onRead }, state.isReading ? "Reading..." : "Read"),
      h("button", { type: "button", className: "url-open-button url-stop-button", "data-url-stop-button": "true", disabled: actionsDisabled || undefined, "aria-disabled": actionsDisabled ? "true" : undefined, onClick: handlers?.onStop }, state.isStopping ? "Stopping..." : "Stop"),
      h("button", { type: "button", className: "url-open-button url-previous-button", "data-url-previous-button": "true", disabled: actionsDisabled || undefined, "aria-disabled": actionsDisabled ? "true" : undefined, onClick: handlers?.onPrevious }, state.isRewinding ? "Previous..." : "Previous"),
      h("button", { type: "button", className: "url-open-button url-next-button", "data-url-next-button": "true", disabled: actionsDisabled || undefined, "aria-disabled": actionsDisabled ? "true" : undefined, onClick: handlers?.onNext }, state.isAdvancing ? "Next..." : "Next"),
    ),
  );
}

export function renderStatusPanelNode(
  state: StatusPanelState,
  handlers?: StatusPanelHandlers,
): ReactNode {
  const title = state.pageTitle ?? "No page open yet";
  const region = state.currentRegionLabel ?? "No current region";
  const transcript = state.lastTranscript ?? "No spoken command captured yet";
  const visiblePressed = state.browserVisibility === "Visible";
  const headlessPressed = state.browserVisibility === "Headless";

  return h(
    "section",
    { className: "status-panel", "aria-labelledby": "status-panel-title" },
    h(
      "div",
      { className: "status-panel-copy" },
      h("p", { className: "status-panel-eyebrow" }, "Runtime status"),
      h("h2", { id: "status-panel-title" }, "Current browser state"),
      h("p", { className: "status-panel-description" }, "This panel mirrors the live runtime so the nearby UI stays aligned with what the browser, narration, and listening tools are doing right now."),
      state.error ? h("p", { className: "status-panel-error", role: "alert" }, state.error) : null,
    ),
    h(
      "dl",
      { className: "status-panel-grid" },
      h("div", { className: "status-card status-card-wide" }, h("dt", null, "Page title"), h("dd", null, title)),
      h("div", { className: "status-card" }, h("dt", null, "Current region"), h("dd", { "aria-live": "polite", "aria-atomic": "true" }, region)),
      h("div", { className: "status-card status-card-wide status-card-transcript" }, h("dt", null, "Last transcript"), h("dd", { "aria-live": "polite", "aria-atomic": "true" }, transcript)),
      h("div", { className: "status-card" }, h("dt", null, "Listening"), h("dd", null, h("span", { className: `status-indicator${state.listening ? " status-indicator-active" : ""}`, role: "status", "aria-live": "polite", "aria-atomic": "true" }, state.listening ? "Active" : "Idle"))),
      h("div", { className: "status-card" }, h("dt", null, "Speaking"), h("dd", null, h("span", { className: `status-indicator${state.speaking ? " status-indicator-active" : ""}`, role: "status", "aria-live": "polite", "aria-atomic": "true" }, state.speaking ? "Active" : "Idle"))),
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