import { type ReactNode } from "react";

import type { PushToTalkPanelState } from "../panel-types.ts";

function MicrophoneIcon() {
  return (
    <svg
      className="icon-button-glyph"
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

export function renderVoiceStatusStripNode(state: VoiceStatusStripState): ReactNode {
  const voiceState = deriveVoiceState(state);
  const label = VOICE_STATE_LABEL[voiceState];
  return (
    <div
      className="voice-status-strip"
      data-voice-state={voiceState}
      role="status"
      aria-live="polite"
      aria-atomic="true"
      aria-label={`Voice state: ${label}`}
    >
      <span className="voice-status-dot" aria-hidden="true" />
      <span className="voice-status-label">{label}</span>
    </div>
  );
}

export interface PushToTalkPanelHandlers {
  onPointerDown?: () => void;
  onOpenSettings?: () => void;
}

export function renderPushToTalkPanelNode(
  state: PushToTalkPanelState,
  handlers?: PushToTalkPanelHandlers,
): ReactNode {
  if (!state.enabled) {
    return (
      <section className="push-to-talk-panel push-to-talk-panel-setup-required" aria-label="Talk control setup required">
        <div className="ptt-setup-banner" role="status" aria-live="polite">
          <p className="ptt-setup-banner-message">
            Voice input isn't set up yet. Open settings to configure your microphone and speech providers.
          </p>
          <button
            type="button"
            className="ptt-setup-banner-button"
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
          ? <p className="push-to-talk-error" aria-hidden="true">{state.lastError}</p>
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
    <section className="push-to-talk-panel" aria-label="Talk control">
      <button
        type="button"
        className={`push-to-talk-button${state.isHolding ? " push-to-talk-button-active" : ""}`}
        data-push-to-talk-button="true"
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
      <p className="push-to-talk-hint" aria-hidden="true">
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
        ? <p className="push-to-talk-error" aria-hidden="true">{state.lastError}</p>
        : null}
    </section>
  );
}
