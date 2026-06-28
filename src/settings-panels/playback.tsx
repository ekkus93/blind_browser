import { type ReactNode } from "react";

import type { AudioControlsPanelState } from "../panel-types.ts";

export interface AudioControlsPanelHandlers {
  onVolumeChange?: (value: number) => void;
  onSpeedChange?: (value: number) => void;
}

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
    <section className="audio-controls-panel" aria-labelledby="audio-controls-title">
      <div className="audio-controls-copy">
        <p className="audio-controls-eyebrow">Speech output</p>
        <h2 id="audio-controls-title">Playback volume and speed</h2>
        {state.error ? <p className="audio-controls-error" role="alert">{state.error}</p> : null}
      </div>
      <div className="audio-controls-grid">
        <label className="audio-control" htmlFor="playback-volume-control">
          <span className="audio-control-label">Volume</span>
          <span className="audio-control-value">{`${Math.round(state.playbackVolume * 100)}%`}</span>
          <input
            id="playback-volume-control"
            className="audio-control-input"
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
        <label className="audio-control" htmlFor="playback-speed-control">
          <span className="audio-control-label">Speed</span>
          <span className="audio-control-value">{`${Math.min(state.playbackSpeed, 2.5).toFixed(2)}x`}</span>
          <input
            id="playback-speed-control"
            className="audio-control-input"
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
