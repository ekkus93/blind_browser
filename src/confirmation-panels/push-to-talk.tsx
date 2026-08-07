import { type ReactNode } from "react";

import type { PushToTalkPanelState } from "../panel-types.ts";

const FOCUS_RING = "focus-visible:[outline:var(--focus-ring)] focus-visible:[outline-offset:var(--focus-offset)]";

function MicrophoneIcon() {
  return (
    <svg
      className="w-12 h-12 block"
      viewBox="0 0 24 24"
      aria-hidden={true}
      focusable="false"
    >
      <path
        d="M12 15a3 3 0 0 0 3-3V7a3 3 0 1 0-6 0v5a3 3 0 0 0 3 3Zm5-3a1 1 0 1 1 2 0 7 7 0 0 1-6 6.93V21h3a1 1 0 1 1 0 2H8a1 1 0 1 1 0-2h3v-2.07A7 7 0 0 1 5 12a1 1 0 1 1 2 0 5 5 0 0 0 10 0Z"
        fill="currentColor"
      />
    </svg>
  );
}

export interface VoiceStatusStripState {
  isListening: boolean;
  isSpeaking: boolean;
  isProcessing: boolean;
}

type VoiceState = "idle" | "listening" | "speaking" | "processing";

function deriveVoiceState(state: VoiceStatusStripState): VoiceState {
  if (state.isListening) {
    return "listening";
  }
  if (state.isSpeaking) {
    return "speaking";
  }
  if (state.isProcessing) {
    return "processing";
  }
  return "idle";
}

const VOICE_STATE_LABEL: Record<VoiceState, string> = {
  idle: "Ready",
  listening: "Listening",
  speaking: "Speaking",
  processing: "Processing",
};

// The dot color depended on a `.voice-status-strip[data-voice-state="x"] .voice-status-dot`
// attribute selector in the old CSS; since the state is already known here, it's
// simpler and equally faithful to pick the color directly.
const VOICE_STATE_DOT_COLOR: Record<VoiceState, string> = {
  idle: "bg-[rgba(123,98,70,0.35)]",
  listening: "bg-[var(--color-green-primary)]",
  speaking: "bg-[#9a6c2d]",
  processing: "bg-[var(--color-amber-primary)]",
};

export function renderVoiceStatusStripNode(state: VoiceStatusStripState): ReactNode {
  const voiceState = deriveVoiceState(state);
  const label = VOICE_STATE_LABEL[voiceState];
  return (
    <div
      className="inline-flex items-center gap-[7px] py-[5px] px-3 rounded-full bg-[var(--color-surface-card)] [border:var(--card-border)] shadow-[0_4px_10px_rgba(49,63,74,0.06)] text-[0.8rem] text-[var(--color-text-secondary)] ml-auto select-none max-sm:text-[0.74rem] max-sm:py-1 max-sm:px-[10px]"
      data-voice-status-strip="true"
      data-voice-state={voiceState}
      role="status"
      aria-live="polite"
      aria-atomic="true"
      aria-label={`Voice state: ${label}`}
    >
      <span className={`w-2 h-2 rounded-full shrink-0 transition-colors duration-200 ${VOICE_STATE_DOT_COLOR[voiceState]}`} aria-hidden="true" />
      <span className="font-semibold tracking-[0.02em]">{label}</span>
    </div>
  );
}

export interface PushToTalkPanelHandlers {
  onPointerDown?: () => void;
  onOpenSettings?: () => void;
}

// [aria-label] { border-radius: 22px; } in the old CSS applies unconditionally,
// since every render of this panel sets an aria-label.
const PUSH_TO_TALK_PANEL_CLASS = "flex flex-col items-center gap-[14px] m-0 py-7 pt-7 px-0 pb-2 rounded-[22px]";

export function renderPushToTalkPanelNode(
  state: PushToTalkPanelState,
  handlers?: PushToTalkPanelHandlers,
): ReactNode {
  if (!state.enabled) {
    return (
      <section className={`${PUSH_TO_TALK_PANEL_CLASS} items-stretch`} aria-label="Talk control setup required">
        <div className="m-0 p-[12px_16px] bg-[rgba(41,88,63,0.08)] border border-[rgba(41,88,63,0.22)] rounded-xl flex flex-col items-start gap-2" data-ptt-setup-banner="true" role="status" aria-live="polite">
          <p className="m-0 text-[0.9rem] font-medium text-[var(--color-green-dark)] leading-[1.4]">
            Voice input isn't set up yet. Open settings to configure your microphone and speech providers.
          </p>
          <button
            type="button"
            className={`py-[6px] px-[14px] rounded-lg border border-[rgba(41,88,63,0.35)] bg-[rgba(41,88,63,0.12)] text-[var(--color-green-dark)] text-[0.875rem] font-semibold cursor-pointer font-[inherit] transition-colors duration-150 hover:bg-[rgba(41,88,63,0.2)] ${FOCUS_RING}`}
            data-ptt-open-settings="true"
            onClick={handlers?.onOpenSettings}
          >
            Open settings
          </button>
        </div>
        {state.lastError
          ? <span className="sr-only" role="alert">{state.lastError}</span>
          : null}
        {state.lastError
          ? <p className="mt-2 mb-0 text-[0.9rem] font-semibold text-[var(--color-error-primary)] text-center" data-ptt-error="true" aria-hidden="true">{state.lastError}</p>
          : null}
      </section>
    );
  }

  const buttonLabel = state.isHolding
    ? "Release to send"
    : state.isListening && state.isBusy
      ? "Voice input active"
      : state.isListening
        ? "Voice input active"
        : state.isBusy
          ? "Processing"
          : "Hold to talk";

  return (
    <section className={PUSH_TO_TALK_PANEL_CLASS} aria-label="Talk control">
      <button
        type="button"
        className={`w-40 h-40 min-w-0 min-h-0 aspect-square p-0 inline-flex items-center justify-center border-none rounded-full text-[#fffdf8] cursor-pointer transition-[transform,box-shadow,background] duration-[120ms,120ms,200ms] max-sm:w-[130px] max-sm:h-[130px] ${FOCUS_RING} focus-visible:-translate-y-px focus-visible:shadow-[0_18px_30px_rgba(31,127,92,0.3)] disabled:cursor-not-allowed disabled:shadow-none disabled:opacity-[0.72] ${
          state.isHolding
            ? "bg-gradient-to-br from-[var(--color-error-primary)] to-[var(--color-error-active)] shadow-[0_18px_30px_rgba(176,71,54,0.28)] [animation:pulse-ring_1.6s_ease-out_infinite] motion-reduce:[animation:none]"
            : "bg-gradient-to-br from-[var(--color-green-primary)] to-[var(--color-green-active)] shadow-[0_14px_28px_rgba(31,127,92,0.24)] enabled:hover:-translate-y-px enabled:hover:shadow-[0_18px_30px_rgba(31,127,92,0.3)] disabled:bg-gradient-to-br disabled:from-[#8a8070] disabled:to-[#a0947e]"
        }`}
        data-push-to-talk-button="true"
        data-ptt-holding={state.isHolding ? "true" : undefined}
        aria-label={buttonLabel}
        aria-pressed={state.isHolding}
        disabled={(state.isBusy || state.isListening) || undefined}
        aria-disabled={(state.isBusy || state.isListening) ? "true" : undefined}
        onPointerDown={handlers?.onPointerDown
          ? (event) => {
            if (event.button !== 0) {
              return;
            }
            event.preventDefault();
            handlers.onPointerDown?.();
          }
          : undefined}
      >
        <MicrophoneIcon />
      </button>
      <p className="m-0 text-[0.9rem] text-[var(--color-text-muted)] tracking-[0.04em]" aria-hidden="true">
        {state.isHolding
          ? "Listening…"
          : state.isListening
            ? "Say 'stop listening' to end"
            : state.isBusy
              ? "Working on your command"
              : "Say a URL or command"}
      </p>
      {state.lastError
        ? <span className="sr-only" role="alert">{state.lastError}</span>
        : null}
      {state.lastError
        ? <p className="mt-2 mb-0 text-[0.9rem] font-semibold text-[var(--color-error-primary)] text-center" data-ptt-error="true" aria-hidden="true">{state.lastError}</p>
        : null}
    </section>
  );
}
