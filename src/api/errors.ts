import { invoke } from "@tauri-apps/api/core";
import type { ToolError, ToolResult } from "../tauri-types.ts";

const tauriInvoker = {
  invoke,
};

export function __setInvokeForTests(nextInvoke: typeof invoke) {
  tauriInvoker.invoke = nextInvoke;
}

export function __resetInvokeForTests() {
  tauriInvoker.invoke = invoke;
}

export function invokeCommand<T>(command: string, args: Record<string, unknown>): Promise<T> {
  return tauriInvoker.invoke<T>(command, args);
}

export interface BackendToolErrorFailure {
  kind: "tool-error";
  toolError: ToolError;
}

export interface TransportFailure {
  kind: "transport-error";
  message: string;
}

export type InvokeFailure = BackendToolErrorFailure | TransportFailure;

export function classifyInvokeFailure(error: unknown): InvokeFailure {
  const toolError = parseToolError(error);
  if (toolError) {
    return {
      kind: "tool-error",
      toolError,
    };
  }

  if (error instanceof Error && error.message.trim().length > 0) {
    return {
      kind: "transport-error",
      message: error.message,
    };
  }

  if (typeof error === "string" && error.trim().length > 0) {
    return {
      kind: "transport-error",
      message: error,
    };
  }

  return {
    kind: "transport-error",
    message: "The app could not reach the requested Tauri command.",
  };
}

export function parseToolError(error: unknown): ToolError | null {
  if (!isRecord(error)) {
    return null;
  }

  const { code, message, retryable, details } = error;
  if (typeof code !== "string" || typeof message !== "string" || typeof retryable !== "boolean") {
    return null;
  }

  return {
    code,
    message,
    retryable,
    details: details ?? null,
  };
}

export function unwrapToolResult<T>(result: ToolResult<T>): T {
  if (result.ok && result.data !== null) {
    return result.data;
  }

  if (result.error) {
    throw new Error(result.error.message);
  }

  throw new Error("The runtime returned an invalid tool result.");
}

export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
