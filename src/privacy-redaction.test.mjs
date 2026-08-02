import assert from "node:assert/strict";
import test from "node:test";

import { redactDiagnosticText, redactDiagnosticValue, sanitizeToolError } from "./privacy-redaction.ts";

test("diagnostic text and nested error values redact credentials", () => {
  assert.equal(
    redactDiagnosticText("authorization: Bearer abcdefghijklmnop"),
    "[REDACTED SENSITIVE DIAGNOSTIC]",
  );
  const safe = sanitizeToolError({
    code: "failed",
    message: "request failed",
    retryable: false,
    details: { api_key: "sk-private-secret-value", nested: { count: 2 } },
  });
  const serialized = JSON.stringify(safe);
  assert.doesNotMatch(serialized, /private-secret/);
  assert.match(serialized, /REDACTED/);
  assert.deepEqual(redactDiagnosticValue({ transcript: "private words", count: 2 }), {
    transcript: "[REDACTED]",
    count: 2,
  });
});
