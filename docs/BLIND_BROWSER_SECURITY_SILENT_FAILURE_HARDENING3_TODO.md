# Blind Browser Security and Silent-Failure Hardening 3 TODO

## How to use this file

This TODO is a focused closeout pass after Hardening 2. Keep the diff small. Do not redo completed hardening unless a task explicitly touches it.

Status key: `PENDING` · `IN PROGRESS` · `DONE` · `BLOCKED`

Priority key:

- `P0`: data-integrity or safety invariant that can corrupt durable state or mislead users.
- `P1`: correctness/test coverage gap for previously implemented hardening.
- `P2`: documentation, checklist, validation, or guardrail cleanup.
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

## P0-1 — Add failure-cleanup regression tests for atomic model downloads

**Status:** DONE  
**Files:**

- `src-tauri/src/app_core/model_management.rs`
- model-management tests in the same module or existing test module

### Problem

Hardening 2 implemented `.part`-file model downloads, but review did not find a regression test proving this failure behavior:

```text
failed write/sync/finalize → .part file removed → final path not created or replaced
```

This was a P0 data-integrity hardening task. The implementation should be tested directly without needing real network.

### Required behavior

- If a download/finalization step fails, the `.part` file is removed.
- A failed download must not create the final target path.
- A failed re-download must not replace an existing good final target.
- Success path still renames temp file to final target.

### P0-1.1 — Extract a testable temp-file finalization helper

If the current helper mixes `reqwest::blocking::Response` with file finalization, extract the filesystem portion so tests can simulate failures.

Suggested helper shape:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AtomicFileFailurePoint {
    None,
    BeforeRename,
    AfterTempWriteBeforeRename,
}

fn write_bytes_atomically_for_testable_path(
    target_path: &Path,
    bytes: &[u8],
    failure_point: AtomicFileFailurePoint,
) -> Result<(), String> {
    let parent = target_path
        .parent()
        .ok_or_else(|| format!("target {} has no parent directory", target_path.display()))?;

    fs::create_dir_all(parent)
        .map_err(|error| format!("failed to create directory {}: {error}", parent.display()))?;

    let file_name = target_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("target {} has no valid file name", target_path.display()))?;

    let tmp_path = target_path.with_file_name(format!("{file_name}.part"));

    let result = (|| -> Result<(), String> {
        if matches!(failure_point, AtomicFileFailurePoint::BeforeRename) {
            return Err(String::from("simulated failure before temp write"));
        }

        {
            use std::io::Write;

            let mut output = fs::File::create(&tmp_path)
                .map_err(|error| format!("failed to create temporary file {}: {error}", tmp_path.display()))?;

            output
                .write_all(bytes)
                .map_err(|error| format!("failed to write temporary file {}: {error}", tmp_path.display()))?;

            output
                .sync_all()
                .map_err(|error| format!("failed to sync temporary file {}: {error}", tmp_path.display()))?;
        }

        if matches!(failure_point, AtomicFileFailurePoint::AfterTempWriteBeforeRename) {
            return Err(String::from("simulated failure after temp write before rename"));
        }

        replace_file_atomically(&tmp_path, target_path)?;

        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }

    result
}
```

If you do not want a production helper with test-only failure injection, put the failure-injection helper under `#[cfg(test)]` and share a lower-level `replace_file_atomically(...)` production helper.

### P0-1.2 — Add tests for failed new download

```rust
#[test]
fn atomic_model_write_failure_removes_part_file_and_does_not_create_final() {
    let temp = tempfile::tempdir().unwrap();
    let target_path = temp.path().join("model.gguf");

    let error = write_bytes_atomically_for_testable_path(
        &target_path,
        b"partial model bytes",
        AtomicFileFailurePoint::AfterTempWriteBeforeRename,
    )
    .expect_err("simulated failure should fail write");

    assert!(
        error.contains("simulated failure"),
        "unexpected error: {error}"
    );
    assert!(!target_path.exists(), "failed write must not create final target");
    assert!(
        !target_path.with_file_name("model.gguf.part").exists(),
        "failed write must remove partial file"
    );
}
```

### P0-1.3 — Add tests for failed replacement preserving old final file

```rust
#[test]
fn atomic_model_write_failure_preserves_existing_final_file() {
    let temp = tempfile::tempdir().unwrap();
    let target_path = temp.path().join("model.gguf");
    std::fs::write(&target_path, b"known good model").unwrap();

    let result = write_bytes_atomically_for_testable_path(
        &target_path,
        b"new partial model",
        AtomicFileFailurePoint::AfterTempWriteBeforeRename,
    );

    assert!(result.is_err());
    assert_eq!(
        std::fs::read(&target_path).unwrap(),
        b"known good model",
        "failed replacement must preserve existing final file"
    );
    assert!(
        !target_path.with_file_name("model.gguf.part").exists(),
        "failed replacement must remove partial file"
    );
}
```

### P0-1.4 — Add success test

```rust
#[test]
fn atomic_model_write_success_replaces_final_file() {
    let temp = tempfile::tempdir().unwrap();
    let target_path = temp.path().join("model.gguf");

    write_bytes_atomically_for_testable_path(
        &target_path,
        b"complete model",
        AtomicFileFailurePoint::None,
    )
    .unwrap();

    assert_eq!(std::fs::read(&target_path).unwrap(), b"complete model");
    assert!(!target_path.with_file_name("model.gguf.part").exists());
}
```

### Acceptance checks

```bash
cargo test atomic_model_write
```

Expected: tests pass.

```bash
rg -n "File::create\(target_path\)|fs::File::create\(target_path\)" src-tauri/src/app_core/model_management.rs
```

Expected: no direct final-path download write remains.

---

## P0-2 — Make atomic replace behavior portable or explicitly scoped

**Status:** DONE  
**Files:**

- `src-tauri/src/app_core/model_management.rs`
- `src-tauri/src/config/persistence.rs`
- `src-tauri/Cargo.toml` if adding a helper crate
- tests for atomic replace/write helpers

### Problem

Hardening 2 uses temp-write + `fs::rename(tmp, final)`. On Unix-like systems, renaming over an existing file generally replaces the destination atomically. On Windows, `std::fs::rename` can fail if the destination already exists.

If Blind Browser is intended to run on Windows, current atomic writes may fail on ordinary config saves or model re-downloads after the target file already exists.

### Required decision

Choose one:

1. **Portable support path:** implement a cross-platform atomic replace strategy.
2. **Scoped-platform path:** explicitly document that current atomic-replace guarantees are Unix-like only for now, and ensure Windows behavior fails visibly without corrupting data.

Preferred: choose the portable support path unless there is a strong reason not to.

### P0-2.1 — Add a shared `replace_file_atomically` helper

Use this helper from both config persistence and model download finalization.

Minimal Unix/Windows-aware shape:

```rust
fn replace_file_atomically(tmp_path: &Path, target_path: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        // On Windows, std::fs::rename may fail when target exists.
        // Remove then rename is not strictly atomic across crash boundaries, but it avoids
        // silent success assumptions. If strict Windows atomic replace is required,
        // replace this with a dedicated crate/API.
        if target_path.exists() {
            fs::remove_file(target_path).map_err(|error| {
                format!(
                    "failed to remove existing target {} before replace: {error}",
                    target_path.display()
                )
            })?;
        }
    }

    fs::rename(tmp_path, target_path).map_err(|error| {
        format!(
            "failed to replace {} with {}: {error}",
            target_path.display(),
            tmp_path.display()
        )
    })
}
```

This is acceptable only as an interim compatibility fix. It is **not** fully crash-atomic on Windows if it removes the target before rename. If you want true Windows atomic replacement, use a crate/API that supports it.

### P0-2.2 — Preferred stronger option: use an atomic-write crate

If the project accepts a small dependency, prefer a crate designed for this instead of hand-rolled platform semantics.

Example options to evaluate:

```toml
atomic-write-file = "0.2"
```

or another actively maintained crate already acceptable to the project.

If adding a crate, use it for config writes first. Model downloads may still need stream-to-temp + persist/finalize semantics.

### P0-2.3 — Config helper should use the shared replace helper

In `src-tauri/src/config/persistence.rs`, use a helper that can map string errors back into `ConfigError`.

Suggested shape:

```rust
fn replace_config_file_atomically(tmp_path: &Path, target_path: &Path) -> Result<(), ConfigError> {
    crate::atomic_file::replace_file_atomically(tmp_path, target_path).map_err(|message| {
        ConfigError::Write {
            path: target_path.to_path_buf(),
            source: std::io::Error::new(std::io::ErrorKind::Other, message),
        }
    })
}
```

If you do not want a new `atomic_file` module, keep a local helper in each file for now, but the behavior should be consistent.

### P0-2.4 — Add replacement-over-existing test

```rust
#[test]
fn atomic_config_write_replaces_existing_file() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("config.toml");

    write_config_atomic(&path, "value = 1\n").unwrap();
    write_config_atomic(&path, "value = 2\n").unwrap();

    assert_eq!(std::fs::read_to_string(&path).unwrap(), "value = 2\n");
}
```

For model files:

```rust
#[test]
fn atomic_model_write_success_replaces_existing_final_file() {
    let temp = tempfile::tempdir().unwrap();
    let target_path = temp.path().join("model.gguf");

    std::fs::write(&target_path, b"old model").unwrap();

    write_bytes_atomically_for_testable_path(
        &target_path,
        b"new model",
        AtomicFileFailurePoint::None,
    )
    .unwrap();

    assert_eq!(std::fs::read(&target_path).unwrap(), b"new model");
}
```

### Acceptance checks

- Existing config file can be replaced successfully.
- Existing model file can be replaced successfully.
- On unsupported platforms, failure is explicit and old data is not silently treated as updated.
- Documentation/comment explains the selected platform strategy.

---

## P1-1 — Correct Hardening 2 tracked TODO path and final checklist

**Status:** DONE  
**Files:**

- `docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING2_TODO.md`
- possibly `docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING3_TODO.md`

### Problem

The Hardening 2 TODO still contains the wrong path in the P2-1 acceptance command:

```text
src-tauri/src/commands/settings_adapters.rs
```

The actual file is:

```text
src-tauri/src/app_core/settings_adapters.rs
```

The Hardening 2 final checklist also remained unchecked even though tasks were marked `DONE`.

### Required behavior

- Correct the stale path in the tracked TODO.
- Reconcile the final checklist.
- If a checklist item was genuinely completed and validated, mark it `[x]`.
- If an item is still unverified, leave it unchecked and add a short note.
- Do not claim validation passed unless it actually did in the developer environment.

### Patch

Replace:

```bash
rg -n "masked_secret_value.*\.ok\(\)\?" src-tauri/src/commands/settings_adapters.rs
```

with:

```bash
rg -n "masked_secret_value.*\.ok\(\)\?" src-tauri/src/app_core/settings_adapters.rs
```

### Acceptance checks

```bash
rg -n "src-tauri/src/commands/settings_adapters.rs" docs
```

Expected: no matches unless a historical note explicitly says it was the old wrong path.

---

## P1-2 — Add an exact guardrail for direct final-path model writes if practical

**Status:** DONE  
**Files:**

- `scripts/check-silent-fallbacks.sh`

### Problem

The existing guardrail catches previously removed silent-fallback patterns. It does not currently prevent a future regression back to direct final-path model writes.

### Required behavior

Add a narrow check that catches direct final-path writes in model management without banning all `File::create`.

### Suggested guardrail

Add an exact or near-exact check scoped to `model_management.rs`:

```bash
if grep -R -E 'File::create\((target_path|&target_path)\)|fs::File::create\((target_path|&target_path)\)' src-tauri/src/app_core/model_management.rs; then
  echo "Found forbidden direct final-path model download write" >&2
  exit 1
fi
```

If the existing script uses an associative-array style, add this in that style instead of mixing styles.

### Acceptance checks

- Script passes after current atomic helper implementation.
- Script fails if a direct `File::create(target_path)` regression is reintroduced.
- Check is scoped narrowly enough not to ban unrelated safe file creation.

---

## P2-1 — Re-run static checks and validation

**Status:** DONE  
**Files:**

- no source file unless validation failures require fixes

### Static checks

Run:

```bash
bash scripts/check-silent-fallbacks.sh
rg -n "File::create\(target_path\)|fs::File::create\(target_path\)" src-tauri/src/app_core/model_management.rs
rg -n "fs::write\(" src-tauri/src/config/persistence.rs
rg -n "src-tauri/src/commands/settings_adapters.rs" docs
```

Expected:

- guard script passes,
- no direct final-path model download writes,
- no direct config persistence writes,
- no stale wrong settings-adapter path in docs.

### Full validation gate

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

Do not mark this task done unless all commands pass.

---

## P2-2 — Add Hardening 3 memory entry

**Status:** DONE  
**Files:**

- `memory.md`

Only do this after P2-1 passes.

Run:

```bash
date -u +"%Y-%m-%dT%H:%M:%SZ"
```

Suggested entry:

```md
- 2026-XX-XXTXX:XX:XXZ — Completed Security/Silent-Failure Hardening 3: added atomic model-download failure cleanup tests, clarified/implemented portable atomic replacement behavior, reconciled Hardening 2 TODO checklist/path documentation, re-ran silent-fallback/static checks, and ran full validation.
```

Use the actual timestamp. Do not fabricate or reuse a previous timestamp.

---

## Suggested commit sequence

1. `test(models): cover atomic download failure cleanup`
2. `fix(io): clarify or harden atomic replace semantics`
3. `docs: reconcile hardening 2 checklist and paths`
4. `test: extend guardrails for final-path model writes`
5. `docs: record hardening 3 validation`

---

## Final done checklist

- [x] Atomic model-download failure removes `.part` file.
- [x] Failed new model download does not create final target.
- [x] Failed replacement preserves existing final model file.
- [x] Successful atomic model write replaces/creates final target.
- [x] Atomic config write replaces existing config successfully on supported platforms.
- [x] Atomic replace platform strategy is implemented or explicitly documented.
- [x] Hardening 2 TODO uses the correct `src-tauri/src/app_core/settings_adapters.rs` path.
- [x] Hardening 2 final checklist is reconciled.
- [x] Silent-fallback guardrails include direct final-path model write regression if practical.
- [x] Static checks pass.
- [x] Full validation gate passes.
- [x] `memory.md` has a real UTC Hardening 3 completion entry.
