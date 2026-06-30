# Responses — BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING4 Spec + TODO Review

1. Q: **Stale-path scope**: The acceptance check in TODO P2-1 is scoped to just `HARDENING2_TODO.md` + `HARDENING3_TODO.md`. Should we also fix H2 SPEC (line 72), H3 SPEC (line 93), and H2 RESPONSES (line 93) for completeness, or leave historical docs alone and only touch what the acceptance check requires?
   A:

2. Q: **P1-2 parent-directory fsync**: Implement the Unix parent-directory sync in `atomic_file.rs`, or document the limitation only? (This determines which doc-comment variant P1-1 uses.)
   A:

3. Q: **`pnpm test` vs `pnpm test:ui`**: The TODO validation gate says `pnpm test`. Is this the correct command for this project, or should it be `pnpm test:ui`?
   A:

4. Q: **In-progress state**: `tempfile` was already removed from `[dependencies]` in Cargo.toml before `/spec-todo` was invoked, leaving Cargo.toml inconsistent (tests that use `tempfile` will fail to compile). Should I complete P0-1 now (add `tempfile` to `[dev-dependencies]`), or revert first so we start clean?
   A:

5. Q: **P1-2 / P1-3 ordering**: P1-1's doc-comment must be chosen *after* deciding P1-2, since the two comment variants differ. If you want parent-directory fsync implemented, confirm that P1-2 should be done before P1-1's comment is written.
   A:

6. Q: **H3 SPEC acceptance command (cross-cutting)**: The H3 SPEC acceptance summary (line 93) uses the broad `rg docs` scan, which will continue to fail if run literally even after H3 TODO is fixed. The H4 TODO does not ask us to fix the H3 SPEC. Should we also fix H3 SPEC line 93 (narrow its scan to just the TODO files), or leave the H3 SPEC untouched since the H4 acceptance check does not cover it?
   A:

---

Fill in each `A:` line above, then share this file back (or paste your answers) to continue with implementation.
