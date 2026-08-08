# Blind Browser Security and Silent-Failure Hardening 4 Spec

## Purpose

Hardening 4 is a narrow closeout pass after `BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING3`.

Hardening 3 mostly succeeded: atomic model-download failure tests were added, model replacement behavior is tested, a shared atomic file helper exists, guardrails catch direct final-path model writes, and Hardening 2 checklist state was largely reconciled.

Hardening 4 should **not** reopen the broader security hardening work. Its purpose is to close the remaining small issues from review:

1. Move `tempfile` from production dependencies to dev-dependencies.
2. Fix stale/wrong `settings_adapters.rs` path references in docs/static checks.
3. Clarify the actual durability guarantee of `atomic_file.rs`.
4. Optionally add parent-directory fsync on Unix-like platforms if it can be done cleanly.
5. Re-run static checks and validation.
6. Record completion truthfully in `memory.md`.

## Current known good behavior to preserve

Do not regress any of these behaviors:

- Remote planner model listing surfaces configured API-key resolution failures.
- Model downloads use `.part` temp files and shared replacement finalization.
- Atomic model-write tests cover failed new writes, failed replacements, successful writes, and replacement over existing files.
- Model availability checks reject empty/obviously partial files.
- Config persistence uses atomic temp-write/finalize helper and no direct `fs::write`.
- Remote ASR rejects missing/non-string `text` in success JSON.
- URL policy uses parser-backed validation and rejects dangerous schemes/malformed authorities.
- Masked-secret inspection failures are visible in settings.
- Silent-fallback guardrails pass and include direct final-path model write regression checks.

## Non-goals

Do not:

- Redesign the atomic file module.
- Replace the existing model-management or config-persistence architecture.
- Add broad static bans on all filesystem writes.
- Add checksums or full cryptographic model verification.
- Rework URL policy, ASR, planner, skill loading, or voice-loop behavior unless a regression is found.
- Claim full crash-proof persistence unless parent-directory sync is actually implemented and documented.
- Mark validation complete without running the full gate.

## Design guidance

### 1. `tempfile` belongs in dev-dependencies

`tempfile` is used by tests. It should not be part of the production dependency graph unless production code uses it. Move it to `[dev-dependencies]`.

### 2. Stale docs paths must not make acceptance checks fail

An earlier document used a stale non-existent `commands/` copy of
`settings_adapters.rs`. The actual path is:

```text
src-tauri/src/app_core/settings_adapters.rs
```

Hardening 4 should remove or correct the stale path everywhere it appears in active acceptance checks. If it remains in a historical note, the note must explicitly say it was the old wrong path.

### 3. Atomic replacement must be described precisely

The current helper appears to use same-directory temp-file replacement. That is a major improvement over direct writes. But unless the parent directory is fsynced after rename where supported, it should not be described as fully crash-durable.

Document the guarantee accurately:

- temp file contents are synced before replacement,
- same-directory rename/replace is used,
- direct final-path truncation is avoided,
- parent-directory fsync may or may not be implemented depending on this pass.

### 4. Parent-directory fsync is optional but valuable

If Claude Code can add Unix parent-directory sync cleanly without cross-platform breakage, do it. If it cannot, add a clear TODO/comment documenting that this is a future crash-durability enhancement.

Do not add risky platform-specific code that breaks Windows builds.

## Expected files touched

Likely:

- `src-tauri/Cargo.toml`
- `src-tauri/src/atomic_file.rs`
- `docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING2_SPEC.md`
- `docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING3_SPEC.md`
- `docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING3_TODO.md`
- `scripts/check-silent-fallbacks.sh` only if static checks need narrowing/clarification
- `memory.md`

Possibly:

- `src-tauri/src/app_core/model_management.rs` only if tests or imports need tiny cleanup.
- `src-tauri/src/config/persistence.rs` only if atomic helper signature changes.

## Acceptance summary

Static checks:

```bash
bash scripts/check-silent-fallbacks.sh
rg -n '^tempfile\s*=\s*"3"' src-tauri/Cargo.toml
python3 - <<'PY'
from pathlib import Path
needle = "src-tauri/src/commands/" + "settings_adapters.rs"
hits = [str(path) for path in Path("docs").glob("*.md") if needle in path.read_text()]
assert not hits, f"stale settings-adapter path remains in: {hits}"
PY
rg -n "File::create\(target_path\)|fs::File::create\(target_path\)" src-tauri/src/app_core/model_management.rs
python3 - <<'PY'
from pathlib import Path
text = Path("src-tauri/src/config/persistence.rs").read_text()
production = text.split("#[cfg(test)]", 1)[0]
assert "fs::write(" not in production
assert "std::fs::write(" not in production
PY
```

Expected:

- silent-fallback guard passes,
- `tempfile` appears only under `[dev-dependencies]`,
- no active docs acceptance check points at the wrong settings adapter path,
- no direct final-path model write remains,
- no direct config persistence write remains.

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

Completion docs:

- Hardening 4 TODO checklist is checked only after validation passes.
- `memory.md` has a real UTC Hardening 4 completion entry.
