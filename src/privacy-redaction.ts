import type { ToolError } from "./tauri-types.ts";

const SENSITIVE_KEY = /api[_-]?key|arguments|authorization|cookie|credential|html|ocr[_-]?text|page[_-]?text|password|response[_-]?body|secret|token|transcript/i;
const SENSITIVE_TEXT = /authorization:|bearer\s+|password\s*[:=]|api[_ ]?key\s*[:=]|access_token\s*=|id_token\s*=|session cookie/i;
const CREDENTIAL = /(?:sk-|ghp_|github_pat_|xox[bp]-|akia)[a-z0-9._-]{12,}/i;

export function redactDiagnosticText(value: string): string {
  if (SENSITIVE_TEXT.test(value) || CREDENTIAL.test(value)) {
    return "[REDACTED SENSITIVE DIAGNOSTIC]";
  }
  try {
    const url = new URL(value);
    url.username = "";
    url.password = "";
    url.search = "";
    url.hash = "";
    return url.toString();
  } catch {
    return value;
  }
}

export function redactDiagnosticValue(value: unknown, key = ""): unknown {
  if (SENSITIVE_KEY.test(key)) {
    return "[REDACTED]";
  }
  if (typeof value === "string") {
    return redactDiagnosticText(value);
  }
  if (Array.isArray(value)) {
    return value.map((entry) => redactDiagnosticValue(entry));
  }
  if (typeof value === "object" && value !== null) {
    return Object.fromEntries(
      Object.entries(value).map(([entryKey, entryValue]) => [
        entryKey,
        redactDiagnosticValue(entryValue, entryKey),
      ]),
    );
  }
  return value;
}

export function sanitizeToolError(error: ToolError): ToolError {
  return {
    code: error.code,
    message: redactDiagnosticText(error.message),
    retryable: error.retryable,
    details: redactDiagnosticValue(error.details),
  };
}
