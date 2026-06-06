import { invokeCommand, unwrapToolResult } from "./errors.ts";
import type {
  BrowserVisibilityMode,
  SetBrowserVisibilityData,
  SetPlaybackSpeedData,
  SetPlaybackVolumeData,
  SetTtsVoiceData,
  ToolResult,
} from "../tauri-types.ts";

export async function setPlaybackVolume(input: {
  requestId: string;
  timeoutMs?: number;
  volume: number;
}): Promise<SetPlaybackVolumeData> {
  const result = await invokeCommand<ToolResult<SetPlaybackVolumeData>>("set_playback_volume", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
    volume: input.volume,
  });
  return unwrapToolResult(result);
}

export async function setPlaybackSpeed(input: {
  requestId: string;
  timeoutMs?: number;
  speed: number;
}): Promise<SetPlaybackSpeedData> {
  const result = await invokeCommand<ToolResult<SetPlaybackSpeedData>>("set_playback_speed", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
    speed: input.speed,
  });
  return unwrapToolResult(result);
}

export async function setBrowserVisibility(input: {
  requestId: string;
  timeoutMs?: number;
  mode: BrowserVisibilityMode;
}): Promise<SetBrowserVisibilityData> {
  const result = await invokeCommand<ToolResult<SetBrowserVisibilityData>>(
    "set_browser_visibility",
    {
      requestId: input.requestId,
      timeoutMs: input.timeoutMs,
      mode: input.mode,
    },
  );
  return unwrapToolResult(result);
}

export async function setTtsVoice(input: {
  requestId: string;
  timeoutMs?: number;
  voice: string;
}): Promise<SetTtsVoiceData> {
  const result = await invokeCommand<ToolResult<SetTtsVoiceData>>("set_tts_voice", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
    voice: input.voice,
  });
  return unwrapToolResult(result);
}
