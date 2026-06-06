import { invokeCommand, unwrapToolResult } from "./errors.ts";
import type {
  DirectToolRequestInput,
  DirectTranscribeCommandInput,
  StartListeningData,
  StopListeningData,
  TranscribeAndExecuteCommandData,
  TranscribeCommandData,
  ToolResult,
} from "../tauri-types.ts";

export async function startListening(input: DirectToolRequestInput): Promise<StartListeningData> {
  const result = await invokeCommand<ToolResult<StartListeningData>>("start_listening", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
  });
  return unwrapToolResult(result);
}

export async function stopListening(input: DirectToolRequestInput): Promise<StopListeningData> {
  const result = await invokeCommand<ToolResult<StopListeningData>>("stop_listening", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
  });
  return unwrapToolResult(result);
}

export async function transcribeCommand(
  input: DirectTranscribeCommandInput,
): Promise<TranscribeCommandData> {
  const result = await invokeCommand<ToolResult<TranscribeCommandData>>("transcribe_command", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
    maxDurationMs: input.maxDurationMs,
    autoStop: input.autoStop,
  });
  return unwrapToolResult(result);
}

export async function transcribeAndExecuteCommand(
  input: DirectTranscribeCommandInput,
): Promise<TranscribeAndExecuteCommandData> {
  return invokeCommand<TranscribeAndExecuteCommandData>("transcribe_and_execute_command", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
    maxDurationMs: input.maxDurationMs,
    autoStop: input.autoStop,
  });
}
