import assert from "node:assert/strict";
import test from "node:test";

import { classifyInvokeFailure } from "./api/errors.ts";
import { redactDiagnosticText, redactDiagnosticValue, sanitizeToolError } from "./privacy-redaction.ts";

const REDACTED = "[REDACTED SENSITIVE DIAGNOSTIC]";

test("diagnostic text redacts authorization and provider token shapes", () => {
  for (const value of [
    "authorization: Bearer abcdefghijklmnop",
    "request failed with sk-abcdefghijklmnopqrstuv",
    "request failed with ghp_abcdefghijklmnopqrstuv",
    "request failed with xoxb-abcdefghijklmnopqrstuv",
    "request failed with AKIAABCDEFGHIJKLMNOP",
  ]) {
    assert.equal(redactDiagnosticText(value), REDACTED);
  }
});

test("diagnostic text redacts JWT-shaped strings", () => {
  assert.equal(
    redactDiagnosticText(
      "transport failed for eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ1c2VyLTEyMyJ9.abcdefghijklmnop",
    ),
    REDACTED,
  );
});

test("diagnostic URLs drop credentials, query strings, and fragments", () => {
  assert.equal(
    redactDiagnosticText("https://user:password@example.com/private?token=secret#fragment"),
    "https://example.com/private",
  );
  assert.equal(
    redactDiagnosticText(
      "request failed at https://user:password@example.com/private?token=secret#fragment",
    ),
    "request failed at https://example.com/private",
  );
});

test("nested diagnostic values redact sensitive keys and response bodies", () => {
  const safe = redactDiagnosticValue({
    nested: {
      api_key: "sk-private-secret-value",
      response_body: { html: "private page", token: "private token" },
      request: {
        endpoint: "https://user:pass@example.com/v1?api_key=private#debug",
        count: 2,
      },
    },
    transcript: "private words",
  });
  const serialized = JSON.stringify(safe);

  assert.doesNotMatch(serialized, /private-secret|private page|private token|private words|user:pass|api_key=|#debug/);
  assert.match(serialized, /REDACTED/);
  assert.deepEqual(safe, {
    nested: {
      api_key: "[REDACTED]",
      response_body: "[REDACTED]",
      request: {
        endpoint: "https://example.com/v1",
        count: 2,
      },
    },
    transcript: "[REDACTED]",
  });
});

test("tool errors redact messages and arbitrary nested details", () => {
  const safe = sanitizeToolError({
    code: "failed",
    message: "request failed at https://user:pass@example.com/v1?token=secret#debug",
    retryable: false,
    details: {
      payload: [
        { credential: "private credential" },
        "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ1c2VyLTEyMyJ9.abcdefghijklmnop",
      ],
    },
  });
  const serialized = JSON.stringify(safe);

  assert.doesNotMatch(serialized, /user:pass|token=secret|#debug|private credential|eyJhbGci/);
  assert.equal(safe.message, "request failed at https://example.com/v1");
  assert.deepEqual(safe.details, {
    payload: ["[REDACTED]", REDACTED],
  });
});

test("frontend Error messages are redacted before classification", () => {
  const failure = classifyInvokeFailure(
    new Error("transport failed with sk-abcdefghijklmnopqrstuv"),
  );

  assert.deepEqual(failure, {
    kind: "transport-error",
    message: REDACTED,
  });
});
