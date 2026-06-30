# Replies: BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING3

Covers: docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING3_SPEC.md
        docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING3_TODO.md

---

1. Q: Implementation order for P0-1/P0-2. The `write_bytes_atomically_for_testable_path` helper in P0-1 calls `replace_file_atomically`, which is defined in P0-2. Should I implement P0-2 (the shared `replace_file_atomically` helper) first, then P0-1 (tests calling it)? Or would you prefer P0-1 to use a simple inline `fs::rename` — and P0-2 wraps it into the shared helper afterward?
   A:

2. Q: Windows support target. Is Blind Browser intended to run on Windows, or is it Linux/macOS-only for now? This determines whether the `#[cfg(windows)]` stub is enough documentation, or whether a portable crate like `atomic-write-file` is preferred.
   A:

3. Q: New `atomic_file` module or local helpers? P0-2.3 references `crate::atomic_file::replace_file_atomically` — a new top-level module. The TODO also says "If you do not want a new `atomic_file` module, keep a local helper in each file." Which do you prefer? (A new top-level module requires ask-first approval per CLAUDE.md, but I'm asking now.)
   A:

4. Q: HARDENING2 final checklist. The 10 checklist items are all unchecked even though Hardening 2 passed validation (362 Rust + 171 JS tests). Should I mark all 10 as `[x]`, or leave any unchecked with a note?
   A:

5. Q: P1-2 guardrail style. The `File::create(target_path)` check requires regex (`-E`). Should it go in a second standalone `grep -E` block (matching the `masked_secret_value` precedent from Hardening 2), or is there another preferred placement?
   A:

---

Fill in each `A:` line above, then share this file back (or paste your answers).
