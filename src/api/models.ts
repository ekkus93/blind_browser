import { invokeCommand } from "./errors.ts";
import type { DownloadedLocalModelData, ModelManagementSettingsData } from "../tauri-types.ts";

export async function getModelManagementSettings(input: {
  requestId: string;
  timeoutMs?: number;
}): Promise<ModelManagementSettingsData> {
  return invokeCommand<ModelManagementSettingsData>("get_model_management_settings", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
  });
}

export async function setModelManagementSettings(input: {
  requestId: string;
  timeoutMs?: number;
  modelsDir: string;
  checkOnStartup: boolean;
  autoDownloadMissing: boolean;
}): Promise<{
  models_dir: string;
  check_on_startup: boolean;
  auto_download_missing: boolean;
}> {
  return invokeCommand("set_model_management_settings", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
    modelsDir: input.modelsDir,
    checkOnStartup: input.checkOnStartup,
    autoDownloadMissing: input.autoDownloadMissing,
  });
}

export async function downloadActiveLocalTtsModel(input: {
  requestId: string;
  timeoutMs?: number;
}): Promise<DownloadedLocalModelData> {
  return invokeCommand<DownloadedLocalModelData>("download_active_local_tts_model", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
  });
}

export async function downloadActiveLocalAsrModel(input: {
  requestId: string;
  timeoutMs?: number;
}): Promise<DownloadedLocalModelData> {
  return invokeCommand<DownloadedLocalModelData>("download_active_local_asr_model", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
  });
}
