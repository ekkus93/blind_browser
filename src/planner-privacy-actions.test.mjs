import assert from "node:assert/strict";
import test from "node:test";

import { parseBlockedOriginsDraft } from "./planner-actions.ts";

test("blocked origin drafts are trimmed deduplicated and deterministic", () => {
  assert.deepEqual(
    parseBlockedOriginsDraft(" https://b.example\nhttps://a.example, https://b.example "),
    ["https://a.example", "https://b.example"],
  );
});
