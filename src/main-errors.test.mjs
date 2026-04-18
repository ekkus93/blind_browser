import assert from "node:assert/strict";
import test from "node:test";

const { describePushToTalkFailure, guidanceStateForErrorMessage } = await import("./main-errors.ts");

test("describePushToTalkFailure adds the OpenAI API key hint for missing ASR secrets", () => {
  const message = describePushToTalkFailure({
    code: "asr_secret_unavailable",
    message: "backend raw message",
    retryable: false,
    details: null,
  });

  assert.match(message, /Voice input needs an OpenAI API key/i);
  assert.match(message, /platform\.openai\.com\/account\/api-keys/);
});

test("guidanceStateForErrorMessage includes the OpenAI API key hint for missing remote secrets", () => {
  const guidance = guidanceStateForErrorMessage("The current remote ASR secret is unavailable.");

  assert.ok(guidance);
  assert.match(guidance.message, /platform\.openai\.com\/account\/api-keys/);
  assert.equal(guidance.title, "Remote ASR secret needs attention");
});