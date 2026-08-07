import { type ReactNode } from "react";

import type {
  StatusPanelAgentStateLike,
  StatusPanelState,
  UrlInputPanelState,
} from "../panel-types.ts";
import { DISMISS_BUTTON_CLASS, FOCUS_RING, SETTINGS_PANEL_SECTION_CLASS } from "./shared-controls.tsx";

const URL_ACTION_BUTTON_BASE_CLASS = `appearance-none w-11 h-11 p-0 inline-flex items-center justify-center border-none rounded-[14px] text-[#fffdf8] cursor-pointer enabled:hover:-translate-y-px focus-visible:-translate-y-px ${FOCUS_RING} disabled:cursor-progress disabled:opacity-[0.62] disabled:shadow-none`;
const URL_ACTION_BUTTON_GREEN_CLASS = "bg-gradient-to-br from-[var(--color-green-primary)] to-[var(--color-green-active)] shadow-[0_12px_24px_rgba(31,127,92,0.18)] enabled:hover:shadow-[0_16px_28px_rgba(31,127,92,0.24)] focus-visible:shadow-[0_16px_28px_rgba(31,127,92,0.24)]";
const URL_ACTION_BUTTON_STOP_CLASS = "bg-gradient-to-br from-[var(--color-error-primary)] to-[var(--color-error-active)] shadow-[0_12px_24px_rgba(176,71,54,0.18)] enabled:hover:shadow-[0_16px_28px_rgba(176,71,54,0.24)] focus-visible:shadow-[0_16px_28px_rgba(176,71,54,0.24)]";
const URL_ACTION_BUTTON_NAV_CLASS = "bg-gradient-to-br from-[var(--btn-nav-start)] to-[var(--btn-nav-end)] shadow-[0_12px_24px_var(--btn-nav-shadow)] enabled:hover:shadow-[0_16px_28px_var(--btn-nav-shadow)] focus-visible:shadow-[0_16px_28px_var(--btn-nav-shadow)]";

// The browser-visibility toggle's pressed/unpressed states are complete,
// self-contained alternatives (not a base string plus a conditional delta):
// pressed and unpressed both set their own border/background/text, so
// Tailwind's utility-emission order can never leave one state's colors
// losing to the other's on the same element.
const TOGGLE_BUTTON_BASE_CLASS = `appearance-none rounded-full py-2 px-3 [font:inherit] font-semibold cursor-pointer transition-[transform,box-shadow,background-color] duration-[140ms] hover:-translate-y-px hover:shadow-[0_10px_22px_rgba(42,55,66,0.12)] focus-visible:-translate-y-px focus-visible:shadow-[0_10px_22px_rgba(42,55,66,0.12)] ${FOCUS_RING} disabled:cursor-progress disabled:translate-y-0 disabled:shadow-none disabled:opacity-60`;
// Exported (rather than kept module-private) so the cascade regression test
// in tailwind-cascade.test.mjs can compile the exact strings this module
// renders, instead of a hand-copied duplicate that could drift from them.
export const TOGGLE_BUTTON_UNPRESSED_CLASS = `${TOGGLE_BUTTON_BASE_CLASS} border border-[rgba(123,98,70,0.2)] bg-[var(--color-surface-inner)] text-[var(--color-text-secondary)]`;
export const TOGGLE_BUTTON_PRESSED_CLASS = `${TOGGLE_BUTTON_BASE_CLASS} border border-[rgba(41,88,63,0.28)] bg-[rgba(41,88,63,0.14)] text-[var(--color-green-active)]`;

type UrlActionIcon = "open" | "read" | "stop" | "previous" | "next";

function UrlActionSvgIcon({ icon }: { icon: UrlActionIcon }) {
  const pathByIcon = {
    open: "M14 3h7v7a1 1 0 1 1-2 0V6.41l-8.29 8.3a1 1 0 0 1-1.42-1.42L17.59 5H14a1 1 0 1 1 0-2ZM5 5h6a1 1 0 1 1 0 2H6v11h11v-5a1 1 0 1 1 2 0v6a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1Z",
    read: "M8 5.14v13.72a1 1 0 0 0 1.53.85l10.3-6.86a1 1 0 0 0 0-1.7L9.53 4.29A1 1 0 0 0 8 5.14Z",
    stop: "M7 7h10v10H7z",
    previous: "M17.71 6.29a1 1 0 0 1 0 1.42L13.41 12l4.3 4.29a1 1 0 1 1-1.42 1.42l-5-5a1 1 0 0 1 0-1.42l5-5a1 1 0 0 1 1.42 0ZM8 6a1 1 0 0 1 1 1v10a1 1 0 1 1-2 0V7a1 1 0 0 1 1-1Z",
    next: "M6.29 6.29a1 1 0 0 1 1.42 0l5 5a1 1 0 0 1 0 1.42l-5 5a1 1 0 1 1-1.42-1.42L10.59 12l-4.3-4.29a1 1 0 0 1 0-1.42ZM16 6a1 1 0 0 1 1 1v10a1 1 0 1 1-2 0V7a1 1 0 0 1 1-1Z",
  } as const;

  return (
    <svg className="w-6 h-6 block" viewBox="0 0 24 24" aria-hidden={true} focusable="false">
      <path d={pathByIcon[icon]} fill="currentColor" />
    </svg>
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
  return (
    <button
      type="button"
      className={className}
      {...{ [dataAttribute]: "true" }}
      aria-label={label}
      title={label}
      disabled={isBusy || undefined}
      aria-disabled={isBusy ? "true" : undefined}
      onClick={onClick}
    >
      <UrlActionSvgIcon icon={icon} />
    </button>
  );
}

export interface UrlInputPanelHandlers {
  onDraftInput?: (value: string) => void;
  onOpen?: () => void;
  onRead?: () => void;
  onStop?: () => void;
  onPrevious?: () => void;
  onNext?: () => void;
  onDismissError?: () => void;
}

export interface StatusPanelHandlers {
  onSetBrowserVisibility?: (mode: "Visible" | "Headless") => void;
  onDismissError?: () => void;
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

  return (
    <section
      className="block m-0 p-[22px_24px] rounded-[22px] bg-[var(--color-surface-card)] [border:var(--card-border)] shadow-[0_18px_36px_rgba(49,63,74,0.08)]"
      aria-label="Page navigation"
    >
      <div className="grid gap-3">
        <div className="grid gap-3 items-center [grid-template-columns:minmax(0,1fr)_auto] max-sm:[grid-template-columns:1fr]">
          <input
            id="url-input-control"
            className={`w-full p-[14px_16px] rounded-[16px] border border-[rgba(123,98,70,0.18)] bg-[var(--color-surface-inner)] text-[var(--color-text-primary)] [font:inherit] shadow-[inset_0_1px_0_rgba(255,255,255,0.5)] focus-visible:border-[rgba(122,87,39,0.32)] disabled:cursor-progress disabled:opacity-[0.72] ${FOCUS_RING}`}
            data-url-input="true"
            aria-label="Page URL"
            type="url"
            inputMode="url"
            autoComplete="url"
            spellCheck={false}
            placeholder="https://example.com"
            value={state.draftValue}
            disabled={actionsDisabled || undefined}
            aria-disabled={actionsDisabled ? "true" : undefined}
            onChange={handlers?.onDraftInput
              ? (event) => { handlers.onDraftInput?.(event.currentTarget.value); }
              : undefined}
          />
          {renderUrlActionButton(
            `${URL_ACTION_BUTTON_BASE_CLASS} ${URL_ACTION_BUTTON_GREEN_CLASS}`,
            "data-url-open-button",
            state.isOpening ? "Opening" : "Open",
            "open",
            actionsDisabled,
            handlers?.onOpen,
          )}
        </div>
        <div
          className="grid gap-3 items-center [grid-template-columns:repeat(4,minmax(0,44px))] max-sm:[grid-template-columns:repeat(4,minmax(0,1fr))]"
          role="group"
          aria-label="Page reading controls"
        >
          {renderUrlActionButton(
            `${URL_ACTION_BUTTON_BASE_CLASS} ${URL_ACTION_BUTTON_GREEN_CLASS}`,
            "data-url-read-button",
            state.isReading ? "Reading" : "Read",
            "read",
            actionsDisabled,
            handlers?.onRead,
          )}
          {renderUrlActionButton(
            `${URL_ACTION_BUTTON_BASE_CLASS} ${URL_ACTION_BUTTON_STOP_CLASS}`,
            "data-url-stop-button",
            state.isStopping ? "Stopping" : "Stop",
            "stop",
            actionsDisabled,
            handlers?.onStop,
          )}
          {renderUrlActionButton(
            `${URL_ACTION_BUTTON_BASE_CLASS} ${URL_ACTION_BUTTON_NAV_CLASS}`,
            "data-url-previous-button",
            state.isRewinding ? "Moving to previous section" : "Previous",
            "previous",
            actionsDisabled,
            handlers?.onPrevious,
          )}
          {renderUrlActionButton(
            `${URL_ACTION_BUTTON_BASE_CLASS} ${URL_ACTION_BUTTON_NAV_CLASS}`,
            "data-url-next-button",
            state.isAdvancing ? "Moving to next section" : "Next",
            "next",
            actionsDisabled,
            handlers?.onNext,
          )}
        </div>
        {state.error ? (
          <p className="mt-[10px] text-[var(--color-error-primary)] font-semibold" role="alert">
            {state.error}
            {handlers?.onDismissError ? (
              <button type="button" className={DISMISS_BUTTON_CLASS} onClick={handlers.onDismissError} aria-label="Dismiss error">Dismiss</button>
            ) : null}
          </p>
        ) : null}
      </div>
    </section>
  );
}

export function renderStatusPanelNode(
  state: StatusPanelState,
  handlers?: StatusPanelHandlers,
): ReactNode {
  const isFirstLoad = state.pageTitle == null && !state.isPageLoading;
  const title = state.isPageLoading ? "Loading page…" : (state.pageTitle ?? "No page open yet");
  const region = state.isPageLoading ? "—" : (state.currentRegionLabel ?? "No current section");
  const transcript = state.lastTranscript ?? "No spoken command captured yet";
  const visiblePressed = state.browserVisibility === "Visible";
  const headlessPressed = state.browserVisibility === "Headless";

  return (
    <section className={`${SETTINGS_PANEL_SECTION_CLASS} max-sm:p-5`} aria-labelledby="status-panel-title">
      <div className="max-w-[60ch]">
        <p className="mb-2 uppercase tracking-[0.18em] text-[0.76rem] text-[var(--eyebrow-color)]">
          Runtime status
          {state.plannerBusy ? (
            <span
              className="ml-2 text-[0.78rem] font-semibold text-[var(--color-text-muted)] [animation:status-panel-busy-pulse_1.4s_ease-in-out_infinite] motion-reduce:[animation:none] motion-reduce:opacity-[0.75]"
              aria-live="polite"
              aria-label="Working"
            >
              Working…
            </span>
          ) : null}
        </p>
        <h2 id="status-panel-title" className="mb-[10px] [font-family:var(--font-display)] text-[clamp(1.1rem,2vw,1.4rem)] leading-[1.05]">Current browser state</h2>
        {state.error ? (
          <p className="mt-[10px] leading-[1.55] text-[var(--color-error-primary)] font-semibold" role="alert">
            {state.error}
            {handlers?.onDismissError ? (
              <button type="button" className={DISMISS_BUTTON_CLASS} onClick={handlers.onDismissError} aria-label="Dismiss error">Dismiss</button>
            ) : null}
          </p>
        ) : null}
      </div>
      {isFirstLoad
        ? <p className="mt-[18px] leading-[1.55] text-[var(--color-text-muted)] italic" aria-live="polite">Hold the Talk button and say a URL or command to get started.</p>
        : (
          <dl className="grid [grid-template-columns:repeat(auto-fit,minmax(180px,1fr))] gap-4 mt-[18px]">
            <div className="m-0 p-[16px_18px] rounded-[18px] bg-[var(--color-surface-inner)] border border-[rgba(123,98,70,0.12)] [grid-column:span_2] max-sm:[grid-column:span_1]">
              <dt className="m-[0_0_6px] text-[0.84rem] uppercase tracking-[0.08em] text-[var(--color-text-label)]">Page title</dt>
              <dd className="m-0 text-[var(--color-text-primary)] font-semibold leading-[1.45]">{title}</dd>
            </div>
            <div className="m-0 p-[16px_18px] rounded-[18px] bg-[var(--color-surface-inner)] border border-[rgba(123,98,70,0.12)]">
              <dt className="m-[0_0_6px] text-[0.84rem] uppercase tracking-[0.08em] text-[var(--color-text-label)]">Current section</dt>
              <dd className="m-0 text-[var(--color-text-primary)] font-semibold leading-[1.45]" aria-live="polite" aria-atomic="true">{region}</dd>
            </div>
            <div className="m-0 p-[16px_18px] rounded-[18px] bg-[var(--color-surface-inner)] border border-[rgba(123,98,70,0.12)] [grid-column:span_2] max-sm:[grid-column:span_1]">
              <dt className="m-[0_0_6px] text-[0.84rem] uppercase tracking-[0.08em] text-[var(--color-text-label)]">Last transcript</dt>
              <dd className="m-0 text-[var(--color-text-primary)] font-medium leading-[1.45] [overflow-wrap:anywhere]" aria-live="polite" aria-atomic="true">{transcript}</dd>
            </div>
            <div className="m-0 p-[16px_18px] rounded-[18px] bg-[var(--color-surface-inner)] border border-[rgba(123,98,70,0.12)]">
              <dt className="m-[0_0_6px] text-[0.84rem] uppercase tracking-[0.08em] text-[var(--color-text-label)]">Browser mode</dt>
              <dd className="m-0 text-[var(--color-text-primary)] font-semibold leading-[1.45]">
                <span className="block mb-[10px]" role="status" aria-live="polite" aria-atomic="true">
                  {state.browserVisibility}
                </span>
                <div className="inline-flex gap-2 flex-wrap" role="group" aria-label="Browser visibility mode">
                  <button
                    type="button"
                    className={visiblePressed ? TOGGLE_BUTTON_PRESSED_CLASS : TOGGLE_BUTTON_UNPRESSED_CLASS}
                    data-browser-visibility-mode="Visible"
                    aria-label="Browser visibility mode: Visible"
                    aria-pressed={visiblePressed}
                    disabled={state.isUpdatingVisibility || undefined}
                    aria-disabled={state.isUpdatingVisibility ? "true" : undefined}
                    onClick={handlers?.onSetBrowserVisibility ? () => { handlers.onSetBrowserVisibility?.("Visible"); } : undefined}
                  >
                    Visible
                  </button>
                  <button
                    type="button"
                    className={headlessPressed ? TOGGLE_BUTTON_PRESSED_CLASS : TOGGLE_BUTTON_UNPRESSED_CLASS}
                    data-browser-visibility-mode="Headless"
                    aria-label="Browser visibility mode: Headless"
                    aria-pressed={headlessPressed}
                    disabled={state.isUpdatingVisibility || undefined}
                    aria-disabled={state.isUpdatingVisibility ? "true" : undefined}
                    onClick={handlers?.onSetBrowserVisibility ? () => { handlers.onSetBrowserVisibility?.("Headless"); } : undefined}
                  >
                    Headless
                  </button>
                </div>
              </dd>
            </div>
            <div className="m-0 p-[16px_18px] rounded-[18px] bg-[var(--color-surface-inner)] border border-[rgba(123,98,70,0.12)]">
              <dt className="m-[0_0_6px] text-[0.84rem] uppercase tracking-[0.08em] text-[var(--color-text-label)]">History</dt>
              <dd className="m-0 text-[var(--color-text-primary)] font-semibold leading-[1.45]">{`Back: ${state.canGoBack ? "Available" : "Unavailable"}. Forward: ${state.canGoForward ? "Available" : "Unavailable"}.`}</dd>
            </div>
          </dl>
        )}
    </section>
  );
}
