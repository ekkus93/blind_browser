# BBCR-005 Final Validation Evidence — 2026-08-02

## Scope

This record covers Batch 6 implementation of BBCR-005: replacing filesystem-derived screenshot and OCR image identifiers with opaque, application-generated, context-bound handles resolved through a private contained cache registry.

It does not close the broader repository-wide audit of every filesystem-facing identifier, and it does not claim a real capture-to-Tesseract text assertion that is not present in this batch.

## Validated source closure

- Trigger head: `9ef37d4bb15d98d1548b179362037715c58bdeae`
- Source commit: `7692336048a6edfba239b863af53fc857d3faca6`
- Workflow run: `30738218767`
- Workflow job: `91470773782`

The source worker passed, in order:

1. exact-head checkout and repository-state refusal;
2. deterministic transformation and invariant assertions;
3. silent-fallback scanning;
4. Rust formatting;
5. default Rust compilation;
6. strict all-target/all-feature Clippy with warnings denied;
7. the complete all-feature Rust test suite under Xvfb;
8. frontend lint;
9. UI tests;
10. production frontend build;
11. one-shot workflow, trigger, transformation, and patch cleanup before publication.

## Implemented security properties

- Public capture and OCR contracts expose an opaque `image_id`, not a caller-supplied path.
- Handles use a strict application-generated format and reject path-like, encoded, Unicode-separator, malformed, and tampered values.
- An internal registry binds each handle to an independently generated filename, owning page ID, page generation, origin, creation time, size, and content hash.
- The cache root and resolved candidates are canonicalized and checked for containment.
- Symlink, type, size, and content-hash substitutions fail closed.
- Files are uniquely created without overwrite; Unix cache and file permissions are restricted to the current user.
- Unknown, expired, stale-generation, stale-origin, and cross-page handles fail closed.
- TTL, count, and byte ceilings bound the cache.
- Active OCR resolution uses a lease so cleanup cannot remove an image while it is in use.
- Cleanup tests cover deferred deletion and registry/file rollback behavior.

## Conservative documentation closure

- Documentation trigger head: `451d23c08dd4d56accca8fc42c3d7de5ed09af1b`
- Documentation commit: `445513ca8e863bf170466a38f61f649b36c990b0`
- Workflow run: `30738674244`
- Workflow job: `91472005280`

The documentation worker asserted the validated source commit was an ancestor, updated only the evidence-supported BBCR-005 checklist scope, verified the expected checked and intentionally open items, removed its one-shot machinery, and then published the documentation commit.

## Intentionally open work

- Audit every other filesystem-facing identifier and path in the repository, including request-derived filenames, model paths, skill paths, configured directories, and future import/export paths.
- Add a real end-to-end test that captures image bytes, resolves the returned opaque handle, runs the production OCR engine, and asserts expected extracted text.

## Exact-final-SHA policy

The owner-authored commit adding this record is the final repository mutation for Batch 6 and is intended to trigger the permanent `ci/permanent` workflow. The exact commit SHA and terminal permanent-CI run/job evidence are recorded in issue #5 after completion so the validated repository SHA is not changed merely to document its own validation result.
