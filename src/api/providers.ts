import { invokeCommand } from "./errors.ts";
import type {
  RemotePlannerConnectionSettingsData,
  RemoteSpeechPrivacyNetworkMode,
  SelectableProviderMode,
} from "../tauri-types.ts";


export type RemoteSpeechPrivacyPurpose = "narration" | "microphone";

export async function setRemoteSpeechPrivacyNetworkMode(input: {
  requestId: string;
  timeoutMs?: number;
  purpose: RemoteSpeechPrivacyPurpose;
  networkMode: RemoteSpeechPrivacyNetworkMode;
}): Promise<{
  purpose: RemoteSpeechPrivacyPurpose;
  network_mode: RemoteSpeechPrivacyNetworkMode;
  origin_rule_count: number;
  changed: boolean;
}> {
  return invokeCommand<{
    purpose: RemoteSpeechPrivacyPurpose;
    network_mode: RemoteSpeechPrivacyNetworkMode;
    origin_rule_count: number;
    changed: boolean;
  }>(
    "set_remote_speech_privacy_network_mode",
    {
      requestId: input.requestId,
      timeoutMs: input.timeoutMs,
      purpose: input.purpose,
      networkMode: input.networkMode,
    },
  );
}

export async function setAsrProviderSelection(input: {
  requestId: string;
  timeoutMs?: number;
  mode: SelectableProviderMode;
}): Promise<{ mode: SelectableProviderMode; changed: boolean }> {
  return invokeCommand<{ mode: SelectableProviderMode; changed: boolean }>(
    "set_asr_provider_selection",
    {
      requestId: input.requestId,
      timeoutMs: input.timeoutMs,
      mode: input.mode,
    },
  );
}

export async function setTtsProviderSelection(input: {
  requestId: string;
  timeoutMs?: number;
  mode: SelectableProviderMode;
}): Promise<{ mode: SelectableProviderMode; changed: boolean }> {
  return invokeCommand<{ mode: SelectableProviderMode; changed: boolean }>(
    "set_tts_provider_selection",
    {
      requestId: input.requestId,
      timeoutMs: input.timeoutMs,
      mode: input.mode,
    },
  );
}

export async function setTtsModelSelection(input: {
  requestId: string;
  timeoutMs?: number;
  profileName: string;
}): Promise<{ profile_name: string; changed: boolean }> {
  return invokeCommand<{ profile_name: string; changed: boolean }>("set_tts_model_selection", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
    profileName: input.profileName,
  });
}

export async function setRemotePlannerConnectionSettings(input: {
  requestId: string;
  timeoutMs?: number;
  profileName: string;
  baseUrl: string;
  model: string;
}): Promise<RemotePlannerConnectionSettingsData> {
  return invokeCommand<RemotePlannerConnectionSettingsData>(
    "set_remote_planner_connection_settings",
    {
      requestId: input.requestId,
      timeoutMs: input.timeoutMs,
      profileName: input.profileName,
      baseUrl: input.baseUrl,
      model: input.model,
    },
  );
}

export async function resetRemotePlannerConnectionSettings(input: {
  requestId: string;
  timeoutMs?: number;
  profileName: string;
}): Promise<RemotePlannerConnectionSettingsData> {
  return invokeCommand<RemotePlannerConnectionSettingsData>(
    "reset_remote_planner_connection_settings",
    {
      requestId: input.requestId,
      timeoutMs: input.timeoutMs,
      profileName: input.profileName,
    },
  );
}
