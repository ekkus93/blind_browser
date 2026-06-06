import { invokeCommand, unwrapToolResult } from "./errors.ts";
import type { OpenUrlData, ToolResult } from "../tauri-types.ts";

export async function openUrl(input: {
  requestId: string;
  timeoutMs?: number;
  url: string;
}): Promise<OpenUrlData> {
  const result = await invokeCommand<ToolResult<OpenUrlData>>("open_url", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
    url: input.url,
  });
  return unwrapToolResult(result);
}

export async function openExternalUrl(input: { url: string }): Promise<void> {
  await invokeCommand<void>("open_external_url", {
    url: input.url,
  });
}
