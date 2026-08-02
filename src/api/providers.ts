import { invokeCommand } from "./errors.ts";
import type {
  RemotePlannerConnectionSettingsData,
  RemotePlannerPrivacySettingsData,
  SelectableProviderMode,
} from "../tauri-types.ts";

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

export async function setRemotePlannerPrivacySettings(input: {
  requestId: string;
  timeoutMs?: number;
  consentToRemotePageData: boolean;
  localOnly: boolean;
  blockedOrigins: string[];
}): Promise<RemotePlannerPrivacySettingsData> {
  return invokeCommand<RemotePlannerPrivacySettingsData>(
    "set_remote_planner_privacy_settings",
    {
      requestId: input.requestId,
      timeoutMs: input.timeoutMs,
      consentToRemotePageData: input.consentToRemotePageData,
      localOnly: input.localOnly,
      blockedOrigins: input.blockedOrigins,
    },
  );
}
