# Blind Browser Security and Silent-Failure Hardening 4 TODO

## How to use this file

This is a narrow closeout pass after Hardening 3. Keep the diff small. Do not redo completed security hardening unless this file explicitly touches it.

Status key: `PENDING` · `IN PROGRESS` · `DONE` · `BLOCKED`

Priority key:

- `P0`: correctness issue that can invalidate validation/dependency hygiene or confuse future audits.
- `P1`: durability/documentation precision or static-check cleanup.
- `P2`: validation, memory, and final closeout.
- `P3`: optional polish after behavior is safe.

Validation gate:

```bash
pnpm install
pnpm test
pnpm build
cd src-tauri
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Do not mark this TODO complete unless the validation gate actually passes.

---

## P0-1 — Move `tempfile` to dev-dependencies

**Status:** PENDING  
**Files:**

- `src-tauri/Cargo.toml`

### Problem

Hardening 3 added `tempfile` for tests, but review found it under `[dependencies]` instead of `[dev-dependencies]`.

If production code does not use `tempfile`, it should not ship in the production dependency graph.

### Required behavior

- `tempfile = "3"` appears under `[dev-dependencies]`.
- `tempfile = "3"` does not appear under `[dependencies]`.
- Rust tests still compile and pass.

### Patch shape

Current bad shape:

```toml
[dependencies]
tempfile = "3"

[dev-dependencies]
```

Replace with:

```toml
[dependencies]
# production dependencies only

[dev-dependencies]
tempfile = "3"
```

If `[dev-dependencies]` already contains other entries, add `tempfile = "3"` there.

### Acceptance checks

Run:

```bash
python3 - <<'PY'
from pathlib import Path
text = Path("src-tauri/Cargo.toml").read_text()
dep = text.split("[dependencies]", 1)[1].split("[dev-dependencies]", 1)[0]
dev = text.split("[dev-dependencies]", 1)[1] if "[dev-dependencies]" in text else ""
assert 'tempfile = "3"' not in dep, "tempfile must not be a production dependency"
assert 'tempfile = "3"' in dev, "tempfile must be a dev-dependency"
PY
```

Then:

```bash
cd src-tauri
cargo test atomic_model_write
```

Expected: tests compile and pass.

---

## P0-2 — Fix stale wrong `settings_adapters.rs` path references in docs

**Status:** PENDING  
**Files:**

- `docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING2_SPEC.md`
- `docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING3_SPEC.md`
- `docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING3_TODO.md`
- any other docs returned by the search below

### Problem

Hardening 3 was supposed to correct the stale wrong path:

```text
src-tauri/src/commands/settings_adapters.rs
```

The real file is:

```text
src-tauri/src/app_core/settings_adapters.rs
```

Review found the wrong path still appears in docs, including active Hardening 3 acceptance checks.

### Required behavior

- Active acceptance checks use `src-tauri/src/app_core/settings_adapters.rs`.
- No active docs command points at `src-tauri/src/commands/settings_adapters.rs`.
- If the old path remains in a historical note, the note must explicitly say it was the old wrong path and must not be used as an acceptance command.

### Patch shape

Replace active commands like:

```bash
rg -n "masked_secret_value.*\.ok\(\)\?" src-tauri/src/commands/settings_adapters.rs
```

with:

```bash
rg -n "masked_secret_value.*\.ok\(\)\?" src-tauri/src/app_core/settings_adapters.rs
```

For broad stale-path checks, prefer a scoped command that checks the file that matters:

```bash
rg -n "src-tauri/src/commands/settings_adapters.rs" docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING2_TODO.md
```

rather than scanning every historical doc if those docs intentionally discuss the old path.

### Acceptance checks

Run:

```bash
rg -n "src-tauri/src/commands/settings_adapters.rs" docs
```

Expected: no matches, unless every match explicitly labels it as the old wrong path.

Preferred stricter check:

```bash
rg -n "src-tauri/src/commands/settings_adapters.rs" docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING2_TODO.md docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING3_TODO.md
```

Expected: no matches.

---

## P1-1 — Clarify atomic replacement durability guarantees

**Status:** PENDING  
**Files:**

- `src-tauri/src/atomic_file.rs`

### Problem

The shared atomic file helper is a major improvement over direct writes, but the exact durability guarantee needs to be documented precisely.

If the helper syncs the temp file and then renames/replaces it, but does not sync the parent directory after the rename, it should not be described as fully crash-durable across power loss.

### Required behavior

- Add a clear doc comment to the public helper.
- State what is guaranteed.
- State what is not guaranteed if parent-directory fsync is not implemented.
- Do not overclaim “fully crash-proof” persistence.

### Suggested doc comment without parent-directory fsync

Use this if Hardening 4 does **not** implement parent-directory sync:

```rust
/// Replaces `target_path` with `tmp_path` using same-directory replacement semantics.
///
/// Callers are expected to write and sync `tmp_path` before calling this helper.
/// This avoids direct truncation of the final target and gives atomic name
/// replacement on supported platforms.
///
/// Durability note: this helper does not currently fsync the containing
/// directory after replacement. A sudden power loss immediately after rename may
/// still lose the directory entry on some filesystems. The old file is not
/// truncated in place, but this is not a full crash-consistency guarantee.
pub fn replace_file_atomically(tmp_path: &Path, target_path: &Path) -> Result<(), String> {
    // existing implementation
}
```

### Suggested doc comment with parent-directory fsync

Use this if Hardening 4 **does** implement parent-directory sync:

```rust
/// Replaces `target_path` with `tmp_path` using same-directory replacement semantics.
///
/// Callers are expected to write and sync `tmp_path` before calling this helper.
/// After replacement, this helper attempts to sync the parent directory on
/// platforms where directory fsync is supported. That improves crash durability
/// for the rename itself while still returning explicit errors on replacement
/// failure.
pub fn replace_file_atomically(tmp_path: &Path, target_path: &Path) -> Result<(), String> {
    // existing implementation
}
```

### Acceptance checks

Run:

```bash
rg -n "Durability note|fsync the containing directory|sync the parent directory" src-tauri/src/atomic_file.rs
```

Expected: a clear comment exists.

---

## P1-2 — Optionally sync parent directory after atomic replacement

**Status:** PENDING  
**Files:**

- `src-tauri/src/atomic_file.rs`
- tests if a helper is added

### Problem

For stronger crash durability on Unix-like systems, syncing the temp file before rename is not always enough. The parent directory should also be synced after the rename so the directory entry update is durable.

This is optional if it becomes platform-risky. If not implemented, P1-1 must document the limitation.

### Required behavior if implemented

- After successful replace/rename, attempt to sync the containing directory on Unix-like platforms.
- Do not break Windows builds.
- If directory sync fails, decide whether to return an error or log/document a best-effort warning. For config/model integrity, returning an error is more conservative.

### Suggested helper

```rust
fn sync_parent_directory(path: &Path) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };

    #[cfg(unix)]
    {
        let dir = std::fs::File::open(parent).map_err(|error| {
            format!(
                "failed to open parent directory {} for sync: {error}",
                parent.display()
            )
        })?;
        dir.sync_all().map_err(|error| {
            format!(
                "failed to sync parent directory {} after atomic replace: {error}",
                parent.display()
            )
        })?;
    }

    #[cfg(not(unix))]
    {
        // Directory fsync is not uniformly available through std on all targets.
        // The replacement itself still returns explicit errors; this branch does
        // not claim full crash-durable directory-entry persistence.
        let _ = parent;
    }

    Ok(())
}
```

Then in `replace_file_atomically(...)`:

```rust
std::fs::rename(tmp_path, target_path).map_err(|error| {
    format!(
        "failed to replace {} with {}: {error}",
        target_path.display(),
        tmp_path.display()
    )
})?;

sync_parent_directory(target_path)?;

Ok(())
```

If current code uses a crate for atomic replace, adapt this only if the crate does not already document parent-directory syncing.

### Tests

This is hard to test meaningfully without filesystem-specific fault injection. Do not overbuild. A small unit test can at least ensure the helper returns `Ok(())` for a real temp directory on Unix:

```rust
#[test]
fn sync_parent_directory_accepts_existing_parent() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("file.txt");

    sync_parent_directory(&path).unwrap();
}
```

If this test is too platform-specific, skip it and rely on full `cargo test` plus documentation.

### Acceptance checks

If implemented:

```bash
rg -n "sync_parent_directory|sync_all\(\).*parent|parent directory" src-tauri/src/atomic_file.rs
```

Expected: helper exists and is called after replacement.

If not implemented:

```bash
rg -n "does not currently fsync the containing directory|not a full crash-consistency guarantee" src-tauri/src/atomic_file.rs
```

Expected: limitation is documented.

---

## P1-3 — Verify Hardening 3 static checks are internally consistent

**Status:** PENDING  
**Files:**

- `docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING3_TODO.md`
- `scripts/check-silent-fallbacks.sh`

### Problem

The Hardening 3 TODO final checklist says static checks pass, but review found the docs stale-path check still failed.

### Required behavior

- Hardening 3 TODO static checks must be commands that actually pass.
- If a command intentionally ignores historical docs, scope it to active TODO files.
- Guardrail script remains narrow and passes.

### Patch shape

In Hardening 3 TODO, replace broad stale-path check if needed:

```bash
rg -n "src-tauri/src/commands/settings_adapters.rs" docs
```

with:

```bash
rg -n "src-tauri/src/commands/settings_adapters.rs" docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING2_TODO.md docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING3_TODO.md
```

or remove the stale path entirely from active docs.

### Acceptance checks

Run the exact static commands listed in Hardening 3 TODO and ensure they pass.

---

## P2-1 — Re-run final static checks

**Status:** PENDING  
**Files:**

- no source files unless checks fail

Run:

```bash
bash scripts/check-silent-fallbacks.sh

python3 - <<'PY'
from pathlib import Path
text = Path("src-tauri/Cargo.toml").read_text()
dep = text.split("[dependencies]", 1)[1].split("[dev-dependencies]", 1)[0]
dev = text.split("[dev-dependencies]", 1)[1] if "[dev-dependencies]" in text else ""
assert 'tempfile = "3"' not in dep, "tempfile must not be a production dependency"
assert 'tempfile = "3"' in dev, "tempfile must be a dev-dependency"
PY

rg -n "File::create\(target_path\)|fs::File::create\(target_path\)" src-tauri/src/app_core/model_management.rs
rg -n "fs::write\(" src-tauri/src/config/persistence.rs
rg -n "src-tauri/src/commands/settings_adapters.rs" docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING2_TODO.md docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING3_TODO.md
```

Expected:

- guard script passes,
- dependency placement is correct,
- no direct final-path model write,
- no direct config persistence write,
- no stale wrong settings-adapter path in active TODO docs.

For the `rg` commands, “no matches” is success.

---

## P2-2 — Run full validation gate

**Status:** PENDING  
**Files:**

- no source files unless validation failures require fixes

Run:

```bash
pnpm install
pnpm test
pnpm build
cd src-tauri
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Do not mark this task done unless every command passes.

---

## P2-3 — Add Hardening 4 memory entry

**Status:** PENDING  
**Files:**

- `memory.md`

Only do this after P2-1 and P2-2 pass.

Run:

```bash
date -u +"%Y-%m-%dT%H:%M:%SZ"
```

Suggested entry:

```md
- 2026-XX-XXTXX:XX:XXZ — Completed Security/Silent-Failure Hardening 4: moved test-only tempfile dependency to dev-dependencies, corrected stale settings-adapter docs/static checks, clarified atomic replacement durability guarantees, re-ran silent-fallback/static checks, and ran full validation.
```

If parent-directory fsync is implemented, use this instead:

```md
- 2026-XX-XXTXX:XX:XXZ — Completed Security/Silent-Failure Hardening 4: moved test-only tempfile dependency to dev-dependencies, corrected stale settings-adapter docs/static checks, added parent-directory sync to atomic replacement where supported, re-ran silent-fallback/static checks, and ran full validation.
```

Use the actual timestamp. Do not fabricate or reuse a previous timestamp.

---

## Suggested commit sequence

1. `chore(deps): move tempfile to dev-dependencies`
2. `docs: fix stale hardening settings adapter paths`
3. `docs(io): clarify atomic replacement durability`
4. Optional: `fix(io): sync parent directory after atomic replace`
5. `docs: record hardening 4 validation`

---

## Final done checklist

- [ ] `tempfile` is only in `[dev-dependencies]`.
- [ ] Active docs/TODO checks use `src-tauri/src/app_core/settings_adapters.rs`.
- [ ] No active docs/TODO acceptance command points at `src-tauri/src/commands/settings_adapters.rs`.
- [ ] `atomic_file.rs` documents exact replacement/durability semantics.
- [ ] Parent-directory fsync is implemented where supported, or the limitation is explicitly documented.
- [ ] Silent-fallback guard script passes.
- [ ] Direct final-path model write check passes.
- [ ] Direct config `fs::write` check passes.
- [ ] Full validation gate passes.
- [ ] `memory.md` has a real UTC Hardening 4 completion entry.
