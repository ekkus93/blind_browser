from pathlib import Path


TODO_PATH = Path("docs/BLIND_BROWSER_COMPREHENSIVE_CODE_REVIEW_FIX_TODO_2026-08-01.md")
START = "## BBCR-005 — Replace Filesystem-Derived Image IDs With Opaque Contained Handles\n"
END = "---\n\n## BBCR-006 — Treat Page Content and OCR as Hostile Prompt-Injection Input\n"

replacement = """## BBCR-005 — Replace Filesystem-Derived Image IDs With Opaque Contained Handles

### Problem

`RunOcrInput.image_id` is caller-controlled and is used to construct a filesystem path without strict validation or canonical containment checks. Absolute paths or traversal components may escape the screenshot cache.

### Required invariant

External/planner-visible image identifiers must be opaque, application-generated handles that resolve only through an internal registry to files beneath the canonical screenshot cache directory.

### Tasks

- [x] Define an opaque image-handle type.
  - [x] Use a UUID, random nonce, or strict application-generated identifier.
  - [x] Implement strict parsing and length limits.
  - [x] Reject path separators, dots-as-components, absolute paths, percent-encoded separators, Unicode separator tricks, control characters, and whitespace.
- [x] Maintain an internal image registry.
  - [x] Map handle to canonical path, owning page ID, creation time, size, and optional content hash.
  - [x] Do not derive arbitrary paths directly from caller-provided strings.
  - [x] Remove entries when files are deleted or expire.
- [x] Canonicalize and verify containment.
  - [x] Canonicalize the screenshot cache root.
  - [x] Canonicalize or safely create the target file.
  - [x] Verify every resolved path remains beneath the root.
  - [x] Reject symlinks or use safe no-follow semantics where available.
- [x] Harden screenshot file creation.
  - [x] Use unique filenames independent of request IDs.
  - [x] Create files with restrictive permissions.
  - [x] Avoid overwriting an existing screenshot silently.
- [x] Bind images to runtime context.
  - [x] Record page ID and origin at capture time.
  - [x] Decide whether OCR may use an image captured from a previous page.
  - [x] Reject stale or cross-page handles unless explicitly supported.
- [x] Add cleanup policy.
  - [x] Set maximum cache count and total bytes.
  - [x] Delete expired screenshots.
  - [x] Remove registry entries transactionally with files.
- [ ] Audit other filesystem-facing identifiers and paths.
  - [ ] Request IDs used in filenames.
  - [ ] Model IDs and model paths.
  - [ ] Skill names/paths.
  - [ ] Configured directories.
  - [ ] Any future export/import path.
  - Batch 6 audited and replaced the screenshot/OCR image path flow only; this broader repository-wide audit remains separate work.

### Required regression tests

- [x] Reject `../` traversal.
- [x] Reject absolute Unix paths.
- [x] Reject Windows drive and UNC paths.
- [x] Reject encoded and Unicode separator variants.
- [x] Reject symlink escape from the cache root.
- [x] Reject unknown, expired, and cross-page handles.
- [ ] Valid captured image handles still resolve and OCR correctly.
  - Same-context registry resolution and the leased OCR routing path are covered, but Batch 6 does not include a real capture-to-Tesseract text assertion.
- [x] Cleanup removes files and registry entries without races.

### Acceptance criteria

- [x] No planner-controlled string is converted directly into a local screenshot path.
- [x] Canonical containment is enforced and tested on supported platforms.

### Evidence

- Validated source commit: `7692336048a6edfba239b863af53fc857d3faca6`.
- Successful exact-trigger source closure: run `30738218767`, job `91470773782`, trigger head `9ef37d4bb15d98d1548b179362037715c58bdeae`.
- The worker passed deterministic transformation assertions, silent-fallback scanning, Rust formatting, default Rust compilation, strict all-target/all-feature Clippy with warnings denied, the complete all-feature Rust test suite under Xvfb, frontend lint, UI tests, and the production frontend build before publishing.
- The worker removed all Batch 6 transformation, patch, trigger, and workflow files before committing the production source.
- Final exact-head permanent-CI evidence is recorded in issue #5 after this documentation closure is validated, avoiding a post-validation repository mutation.

---

## BBCR-006 — Treat Page Content and OCR as Hostile Prompt-Injection Input
"""

text = TODO_PATH.read_text()
if text.count(START) != 1:
    raise SystemExit(f"expected one BBCR-005 heading, found {text.count(START)}")
if text.count(END) != 1:
    raise SystemExit(f"expected one BBCR-005 end marker, found {text.count(END)}")
start = text.index(START)
end = text.index(END, start) + len(END)
updated = text[:start] + replacement + text[end:]
if updated == text:
    raise SystemExit("BBCR-005 documentation transformation made no change")
TODO_PATH.write_text(updated)
print("Closed the evidence-supported BBCR-005 scope conservatively")
