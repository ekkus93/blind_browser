import { createElement, type ReactNode } from "react";

import type { AudioControlsPanelState } from "../panel-types.ts";

const h = createElement;

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
  return h(
    "section",
    { className: "audio-controls-panel", "aria-labelledby": "audio-controls-title" },
    h(
      "div",
      { className: "audio-controls-copy" },
      h("p", { className: "audio-controls-eyebrow" }, "Speech output"),
      h("h2", { id: "audio-controls-title" }, "Playback volume and speed"),
      state.error ? h("p", { className: "audio-controls-error", role: "alert" }, state.error) : null,
    ),
    h(
      "div",
      { className: "audio-controls-grid" },
      h(
        "label",
        { className: "audio-control", htmlFor: "playback-volume-control" },
        h("span", { className: "audio-control-label" }, "Volume"),
        h("span", { className: "audio-control-value" }, `${Math.round(state.playbackVolume * 100)}%`),
        h("input", {
          id: "playback-volume-control",
          className: "audio-control-input",
          "data-audio-control": "volume",
          type: "range",
          min: "0",
          max: "1",
          step: "0.05",
          value: state.playbackVolume.toFixed(2),
          "aria-valuetext": renderPlaybackVolumeValueText(state.playbackVolume),
          disabled: state.isBusy || undefined,
          "aria-disabled": state.isBusy ? "true" : undefined,
          onChange: handlers?.onVolumeChange
            ? (event: { currentTarget: { value: string } }) => {
              handlers.onVolumeChange?.(Number.parseFloat(event.currentTarget.value));
            }
            : undefined,
        }),
      ),
      h(
        "label",
        { className: "audio-control", htmlFor: "playback-speed-control" },
        h("span", { className: "audio-control-label" }, "Speed"),
        h("span", { className: "audio-control-value" }, `${state.playbackSpeed.toFixed(2)}x`),
        h("input", {
          id: "playback-speed-control",
          className: "audio-control-input",
          "data-audio-control": "speed",
          type: "range",
          min: "0.5",
          max: "5",
          step: "0.05",
          value: state.playbackSpeed.toFixed(2),
          "aria-valuetext": renderPlaybackSpeedValueText(state.playbackSpeed),
          disabled: state.isBusy || undefined,
          "aria-disabled": state.isBusy ? "true" : undefined,
          onChange: handlers?.onSpeedChange
            ? (event: { currentTarget: { value: string } }) => {
              handlers.onSpeedChange?.(Number.parseFloat(event.currentTarget.value));
            }
            : undefined,
        }),
      ),
    ),
  );
}