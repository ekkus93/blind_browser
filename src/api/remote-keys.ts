import { invokeCommand } from "./errors.ts";
import type {
  RemotePlannerModelListData,
  SetRemoteApiKeyData,
  TestRemoteApiKeyData,
} from "../tauri-types.ts";

export async function setRemotePlannerApiKey(input: {
  requestId: string;
  timeoutMs?: number;
  profileName: string;
  apiKey: string;
}): Promise<SetRemoteApiKeyData> {
  return invokeCommand<SetRemoteApiKeyData>("set_remote_planner_api_key", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
    profileName: input.profileName,
    apiKey: input.apiKey,
  });
}

export async function setRemoteTtsApiKey(input: {
  requestId: string;
  timeoutMs?: number;
  profileName: string;
  apiKey: string;
}): Promise<SetRemoteApiKeyData> {
  return invokeCommand<SetRemoteApiKeyData>("set_remote_tts_api_key", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
    profileName: input.profileName,
    apiKey: input.apiKey,
  });
}

export async function setRemoteAsrApiKey(input: {
  requestId: string;
  timeoutMs?: number;
  profileName: string;
  apiKey: string;
}): Promise<SetRemoteApiKeyData> {
  return invokeCommand<SetRemoteApiKeyData>("set_remote_asr_api_key", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
    profileName: input.profileName,
    apiKey: input.apiKey,
  });
}

export async function testRemotePlannerApiKey(input: {
  requestId: string;
  timeoutMs?: number;
  profileName: string;
  apiKey: string;
}): Promise<TestRemoteApiKeyData> {
  return invokeCommand<TestRemoteApiKeyData>("test_remote_planner_api_key", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
    profileName: input.profileName,
    apiKey: input.apiKey,
  });
}

export async function testRemoteTtsApiKey(input: {
  requestId: string;
  timeoutMs?: number;
  profileName: string;
  apiKey: string;
}): Promise<TestRemoteApiKeyData> {
  return invokeCommand<TestRemoteApiKeyData>("test_remote_tts_api_key", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
    profileName: input.profileName,
    apiKey: input.apiKey,
  });
}

export async function testRemoteAsrApiKey(input: {
  requestId: string;
  timeoutMs?: number;
  profileName: string;
  apiKey: string;
}): Promise<TestRemoteApiKeyData> {
  return invokeCommand<TestRemoteApiKeyData>("test_remote_asr_api_key", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
    profileName: input.profileName,
    apiKey: input.apiKey,
  });
}

export async function listRemotePlannerModels(input: {
  requestId: string;
  timeoutMs?: number;
  profileName: string;
  baseUrl: string;
  apiKey: string;
}): Promise<RemotePlannerModelListData> {
  return invokeCommand<RemotePlannerModelListData>("list_remote_planner_models", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
    profileName: input.profileName,
    baseUrl: input.baseUrl,
    apiKey: input.apiKey,
  });
}
