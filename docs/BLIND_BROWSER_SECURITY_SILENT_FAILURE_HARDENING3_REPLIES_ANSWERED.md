# Replies: BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING3

Covers: docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING3_SPEC.md
        docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING3_TODO.md

---

1. Q: Implementation order for P0-1/P0-2. The `write_bytes_atomically_for_testable_path` helper in P0-1 calls `replace_file_atomically`, which is defined in P0-2. Should I implement P0-2 (the shared `replace_file_atomically` helper) first, then P0-1 (tests calling it)? Or would you prefer P0-1 to use a simple inline `fs::rename` — and P0-2 wraps it into the shared helper afterward?
   A: Implement P0-2 first, then P0-1.

   Create the shared replacement/finalization helper first, then make the model-download tests call the same helper the production path uses. Do **not** write the P0-1 tests around an inline `fs::rename` and then refactor afterward, because that risks testing behavior that is not actually the final production behavior.

   Preferred order:

   1. Add the shared atomic/finalize helper.
   2. Update model download finalization to call it.
   3. Update config atomic write to call it or a thin config-specific wrapper around it.
   4. Add the P0-1 model-download failure/success tests against that final helper path.
   5. Run the static guardrails.

   The test helper may still use a `#[cfg(test)]` failure-injection enum, but the actual replace/finalize call should be shared with production.

---

2. Q: Windows support target. Is Blind Browser intended to run on Windows, or is it Linux/macOS-only for now? This determines whether the `#[cfg(windows)]` stub is enough documentation, or whether a portable crate like `atomic-write-file` is preferred.
   A: Treat Windows as a supported target for this Tauri app unless the project explicitly documents otherwise. Prefer a portable approach.

   Do **not** rely on “Unix rename over existing destination” semantics as the only behavior. If the implementation keeps a `#[cfg(windows)]` remove-then-rename fallback, document clearly that this is not crash-atomic during the small replace window. Better is to use a small well-scoped crate or API that gives safer cross-platform atomic write/replace semantics.

   Preference:

   1. Use a small crate designed for atomic writes/replaces if it fits the project cleanly.
   2. If using a crate is too invasive, implement the best practical cross-platform helper and document the Windows limitation directly in the helper comment.
   3. In either case, tests must show that replacing an existing config/model file works on the current platform.

   Do not silently choose Linux/macOS-only behavior unless `README`, specs, or packaging docs are also updated to say Windows is not currently supported.

---

3. Q: New `atomic_file` module or local helpers? P0-2.3 references `crate::atomic_file::replace_file_atomically` — a new top-level module. The TODO also says "If you do not want a new `atomic_file` module, keep a local helper in each file." Which do you prefer? (A new top-level module requires ask-first approval per CLAUDE.md, but I'm asking now.)
   A: Create a small shared module. Approval granted.

   Use a focused module rather than duplicating local helpers in model management and config persistence. The behavior needs to be consistent across config writes and model download finalization.

   Preferred file:

   ```text
   src-tauri/src/atomic_file.rs
   ```

   Export it from `src-tauri/src/lib.rs`:

   ```rust
   pub mod atomic_file;
   ```

   Keep the module narrow. It should not become a general filesystem abstraction. Suggested public API:

   ```rust
   pub fn replace_file_atomically(tmp_path: &Path, target_path: &Path) -> Result<(), String>
   ```

   Optionally add a higher-level helper if it naturally fits both callers:

   ```rust
   pub fn write_bytes_atomically(target_path: &Path, bytes: &[u8]) -> Result<(), String>
   ```

   But do not overbuild. The minimum useful shared piece is finalizing/replacing a temp file safely and consistently.

---

4. Q: HARDENING2 final checklist. The 10 checklist items are all unchecked even though Hardening 2 passed validation (362 Rust + 171 JS tests). Should I mark all 10 as `[x]`, or leave any unchecked with a note?
   A: Mark the checklist items `[x]` only if they are actually satisfied in the repo after the Hardening 3 cleanup changes.

   Based on the last review, most Hardening 2 behavior was implemented, but two items needed follow-up cleanup:

   - model-download atomic behavior needed failure-cleanup regression coverage,
   - final documentation/checklist state needed reconciliation.

   Since Hardening 3 is specifically closing those gaps, the correct sequence is:

   1. Implement the Hardening 3 test/documentation fixes.
   2. Re-run the full validation gate.
   3. Then mark the Hardening 2 final checklist items `[x]`.

   If validation truly passed with 362 Rust + 171 JS tests after these fixes, it is fine to mark all 10 Hardening 2 checklist items `[x]`.

   Add a short note near the checklist or in `memory.md` such as:

   ```md
   Hardening 2 final checklist reconciled after Hardening 3 added atomic model-download failure cleanup coverage and reran validation.
   ```

   Do not leave the checklist unchecked if the work is now verified. Unchecked boxes contradict `DONE` statuses and create future review confusion.

---

5. Q: P1-2 guardrail style. The `File::create(target_path)` check requires regex (`-E`). Should it go in a second standalone `grep -E` block (matching the `masked_secret_value` precedent from Hardening 2), or is there another preferred placement?
   A: Use a standalone `grep -E` block if the current associative-array style is fixed-string oriented or awkward for regex.

   Keep it narrow and readable. The important thing is that the check is scoped to `src-tauri/src/app_core/model_management.rs` and only catches direct final-path model writes, not all file creation.

   Suggested block:

   ```bash
   if grep -R -E 'File::create\((target_path|&target_path)\)|fs::File::create\((target_path|&target_path)\)' src-tauri/src/app_core/model_management.rs; then
     echo "Found forbidden direct final-path model download write" >&2
     exit 1
   fi
   ```

   If the existing script already has a clean function/helper for regex checks, use that instead. Do not refactor the whole script just to avoid one standalone regex block.

   Also add or document a negative check: temporarily reintroducing `File::create(target_path)` in `model_management.rs` should make `scripts/check-silent-fallbacks.sh` fail.

---

Fill in each `A:` line above, then share this file back (or paste your answers).
