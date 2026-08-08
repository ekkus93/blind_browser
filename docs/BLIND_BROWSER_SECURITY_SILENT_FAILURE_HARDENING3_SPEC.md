# Blind Browser Security and Silent-Failure Hardening 3 Spec

## Purpose

Hardening 3 is a focused closeout pass after `BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING2`.

Hardening 2 substantially fixed the dangerous silent-failure patterns it targeted: configured remote-planner API-key failures now surface, model downloads are written through temporary files, model availability checks reject obvious corrupt files, config writes use an atomic-write helper, remote ASR rejects malformed success JSON, URL policy uses robust parsing, masked-secret inspection errors surface in settings, and guardrails were extended.

Hardening 3 should not redo that work. It exists to close the remaining review issues:

1. Add regression coverage for failed atomic model downloads cleaning up `.part` files and not creating/replacing final files.
2. Decide and implement a portable atomic replace strategy, or explicitly document and test the supported platform behavior.
3. Fix tracked TODO/documentation inconsistencies from Hardening 2.
4. Re-run validation and record completion truthfully.

## Current known good behavior to preserve

Do not regress these behaviors:

- Remote planner model listing does not swallow configured API-key resolution failures.
- Model downloads write to a temporary `.part` file before finalization.
- Empty/tiny ASR and TTS model files are not considered available.
- Config persistence no longer uses direct `fs::write`.
- Remote ASR requires a string `text` field in successful JSON responses.
- Browser navigation policy uses parser-backed URL validation and rejects dangerous schemes/malformed authorities.
- Settings UI renders secret-inspection warnings.
- Silent-fallback guardrails check the exact removed bad patterns.
- Prior CSP, navigation timeout, visibility-switch, page metrics, local ASR, bundled skill, voice-loop, stale confirmation, and runtime-refresh hardening remain intact.

## Non-goals

Do not:

- Redesign model management.
- Rewrite the configuration system.
- Add checksum support unless model metadata already provides checksums.
- Add broad CI grep bans for all `.ok()`, all `unwrap_or_default()`, or all filesystem writes.
- Reopen already-completed UI/UX work.
- Add large new dependencies unless they directly solve portable atomic replace.
- Mark validation complete unless it actually passed in the developer environment.

## Design principles

### 1. Atomic-download behavior must be tested, not only inspected

A `.part`-file implementation can look correct but still fail to clean up, replace the wrong file, or overwrite an existing good file too early. Hardening 3 must add helper-level tests that simulate failure before finalization.

The test does not need real network. Prefer extracting a small file-finalization helper so the cleanup behavior can be tested without mocking `reqwest::blocking::Response`.

### 2. Atomic replace semantics must match supported platforms

On Unix-like systems, `std::fs::rename(temp, existing_final)` generally replaces the destination atomically. On Windows, `std::fs::rename` can fail when the destination exists. If the app supports Windows, use a cross-platform atomic replace strategy.

Preferred options:

- Use a small crate such as `tempfile::NamedTempFile::persist` / `persist_noclobber` where appropriate.
- Use a crate designed for atomic writes/replaces if already acceptable for the project.
- Implement platform-specific replace logic carefully.
- If the app is intentionally Linux/macOS-only for now, document that explicitly and add a test/comment explaining the supported semantics.

### 3. Documentation/checklists are part of the trust boundary

The Hardening 2 TODO marked tasks as done but left the final checklist unchecked and kept one stale path in an acceptance command. That undermines reviewability. Hardening 3 must reconcile those documents after validation.

### 4. Keep guardrails exact and low-noise

Extend guardrails only for exact regressions if needed. Avoid broad patterns that will create false positives and get disabled later.

## Expected files touched

Likely:

- `src-tauri/src/app_core/model_management.rs`
- `src-tauri/src/config/persistence.rs`
- `src-tauri/Cargo.toml` if adding a small atomic-write dependency
- `docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING2_TODO.md`
- `docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING3_TODO.md`
- `memory.md`

Optional:

- `scripts/check-silent-fallbacks.sh` if adding exact checks for final-path download writes or direct config writes.
- CI config if the script invocation needs updating.

## Acceptance summary

Static checks:

```bash
bash scripts/check-silent-fallbacks.sh
rg -n "File::create\(target_path\)|fs::File::create\(target_path\)" src-tauri/src/app_core/model_management.rs
python3 - <<'PY'
from pathlib import Path
text = Path("src-tauri/src/config/persistence.rs").read_text()
production = text.split("#[cfg(test)]", 1)[0]
assert "fs::write(" not in production
assert "std::fs::write(" not in production
PY
python3 - <<'PY'
from pathlib import Path
needle = "src-tauri/src/commands/" + "settings_adapters.rs"
hits = [str(path) for path in Path("docs").glob("*.md") if needle in path.read_text()]
assert not hits, f"stale settings-adapter path remains in: {hits}"
PY
```

The first two `rg` checks should have no unsafe production matches. Any remaining match must be in a helper/test with a comment explaining why it is safe.

Validation gate:

```bash
pnpm install
pnpm test:ui
pnpm build
cd src-tauri
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Documentation completion:

- Hardening 2 final checklist is reconciled.
- Hardening 3 final checklist is checked only after validation passes.
- `memory.md` has a real UTC Hardening 3 entry.
