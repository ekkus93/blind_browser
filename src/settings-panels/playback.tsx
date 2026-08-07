import { type ReactNode } from "react";

import type { AudioControlsPanelState } from "../panel-types.ts";
import { SETTINGS_PANEL_SECTION_CLASS } from "./shared-controls.tsx";

export interface AudioControlsPanelHandlers {
  onVolumeChange?: (value: number) => void;
  onSpeedChange?: (value: number) => void;
  onDismissError?: () => void;
}

const DISMISS_BUTTON_CLASS = "appearance-none bg-transparent border-none pl-2 [font:inherit] text-[0.84rem] font-bold text-inherit cursor-pointer opacity-70 underline hover:opacity-100";
// Exported so the cascade regression test in tailwind-cascade.test.mjs can
// compile the exact string this module renders, instead of a hand-copied
// duplicate that could drift from it.
export const AUDIO_CONTROL_CLASS = "grid [grid-template-columns:1fr_auto] gap-[8px_12px] p-[16px_18px] rounded-[18px] bg-[var(--color-surface-inner)] [border:var(--inner-card-border)]";
const AUDIO_CONTROL_LABEL_CLASS = "font-bold text-[var(--color-text-primary)]";

function renderPlaybackVolumeValueText(value: number): string {
  return `${Math.round(value * 100)} percent`;
}

function renderPlaybackSpeedValueText(value: number): string {
  return `${value.toFixed(2)} times`;
}

export function renderAudioControlsPanelNode(
  state: AudioControlsPanelState,
  handlers?: AudioControlsPanelHandlers,
): ReactNode {
  return (
    <section className={`${SETTINGS_PANEL_SECTION_CLASS} max-sm:p-5`} aria-labelledby="audio-controls-title">
      <div className="max-w-[60ch]">
        <p className="mb-2 uppercase tracking-[0.18em] text-[0.76rem] text-[var(--eyebrow-color)]">Speech output</p>
        <h2 id="audio-controls-title" className="mb-[10px] [font-family:var(--font-display)] text-[clamp(1.1rem,2vw,1.4rem)] leading-[1.05]">Playback volume and speed</h2>
        {state.error ? (
          <p className="leading-[1.55] mt-[10px] text-[var(--color-error-primary)] font-semibold" role="alert">
            {state.error}
            {handlers?.onDismissError ? (
              <button type="button" className={DISMISS_BUTTON_CLASS} onClick={handlers.onDismissError} aria-label="Dismiss error">Dismiss</button>
            ) : null}
          </p>
        ) : null}
      </div>
      <div className="grid [grid-template-columns:repeat(auto-fit,minmax(220px,1fr))] gap-4 mt-[18px]">
        <label className={AUDIO_CONTROL_CLASS} htmlFor="playback-volume-control">
          <span className={AUDIO_CONTROL_LABEL_CLASS}>Volume</span>
          <span className={`${AUDIO_CONTROL_LABEL_CLASS} justify-self-end`}>{`${Math.round(state.playbackVolume * 100)}%`}</span>
          <input
            id="playback-volume-control"
            className="[grid-column:1/-1] w-full accent-[var(--color-green-primary)] disabled:cursor-progress disabled:opacity-60"
            data-audio-control="volume"
            type="range"
            min="0"
            max="1"
            step="0.05"
            value={state.playbackVolume.toFixed(2)}
            aria-valuetext={renderPlaybackVolumeValueText(state.playbackVolume)}
            disabled={state.isBusy || undefined}
            aria-disabled={state.isBusy ? "true" : undefined}
            onChange={handlers?.onVolumeChange
              ? (event) => { handlers.onVolumeChange?.(Number.parseFloat(event.currentTarget.value)); }
              : undefined}
          />
        </label>
        <label className={AUDIO_CONTROL_CLASS} htmlFor="playback-speed-control">
          <span className={AUDIO_CONTROL_LABEL_CLASS}>Speed</span>
          <span className={`${AUDIO_CONTROL_LABEL_CLASS} justify-self-end`}>{`${Math.min(state.playbackSpeed, 2.5).toFixed(2)}x`}</span>
          <input
            id="playback-speed-control"
            className="[grid-column:1/-1] w-full accent-[var(--color-green-primary)] disabled:cursor-progress disabled:opacity-60"
            data-audio-control="speed"
            type="range"
            min="0.5"
            max="2.5"
            step="0.05"
            value={Math.min(state.playbackSpeed, 2.5).toFixed(2)}
            aria-valuetext={renderPlaybackSpeedValueText(Math.min(state.playbackSpeed, 2.5))}
            disabled={state.isBusy || undefined}
            aria-disabled={state.isBusy ? "true" : undefined}
            onChange={handlers?.onSpeedChange
              ? (event) => { handlers.onSpeedChange?.(Number.parseFloat(event.currentTarget.value)); }
              : undefined}
          />
        </label>
      </div>
    </section>
  );
}
