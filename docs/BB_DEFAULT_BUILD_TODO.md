# Blind Browser Default Build TODO

## How to use this file

Small, self-contained fix: make the default (`cargo build` with no `--features`)
configuration compile, so contributors and rust-analyzer aren't broken. Read
`BB_DEFAULT_BUILD_SPEC.md` first. The breakage is pre-existing and invisible to CI
(which only builds `--all-features`).

Status key: `PENDING` · `IN PROGRESS` · `DONE` · `BLOCKED`

Priority key:

- `P0`: make the default configuration compile.
- `P1`: guard it in CI so it can't rot again.
- `P2`: validation + memory.

Validation gate:

```bash
source ./fix-node-version.sh
pnpm lint
pnpm test:ui
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
```

Do not mark complete unless the gate passes.

---

## P0.1 — Set `default = ["full"]`

**Status:** PENDING  
**Files:**

- `src-tauri/Cargo.toml`

### Problem

`default = []` does not compile (incomplete per-item `#[cfg]` gating in
`browser` / `tts` / `asr` / `audio_io`). This is an application crate with no
consumer of a minimal build, and the only no-feature `cargo build`
(`scripts/darkmode-test.sh`) expects the default config to work.

### P0.1.1 — Change the default feature set

In `src-tauri/Cargo.toml`, change `default = []` to `default = ["full"]`. Do not
touch any other feature definition.

### P0.1.2 — Confirm the default config compiles

Run a no-flag check and confirm it builds (CI already installs the system libs the
full set needs):

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Do **not** attempt to make `--no-default-features` compile — that minimal
configuration is out of scope (see spec). Leave the per-item gating untouched.

### Acceptance checks

```bash
rg -n "^default = \[\"full\"\]" src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: `default = ["full"]`; the no-flag check compiles.

---

## P1.1 — Guard the default configuration in CI

**Status:** PENDING  
**Files:**

- `.github/workflows/ci.yml`

### P1.1.1 — Add a default-config check step

Add a step (alongside the existing `--all-features` steps) that builds the default
configuration so it can't silently rot again:

```yaml
- name: Check default feature configuration
  run: cargo check --manifest-path src-tauri/Cargo.toml
```

This is distinct from the `--all-features` clippy/test steps: it exercises exactly
what a plain `cargo build` / rust-analyzer uses.

### Acceptance checks

```bash
rg -n "cargo check --manifest-path src-tauri/Cargo.toml" .github/workflows/ci.yml
```

Expected: a default-config `cargo check` step exists in CI.

---

## P2.1 — Validation and memory

**Status:** PENDING  
**Files:**

- `memory.md`

- Run the full gate (top of file); it must stay green (`--all-features` behavior is
  unchanged by this fix).
- Add a `memory.md` entry with a real UTC timestamp:

```bash
date -u +"%Y-%m-%dT%H:%M:%SZ"
```

Note that `default = ["full"]` fixes the contributor/rust-analyzer default build and
unbreaks `scripts/darkmode-test.sh`'s plain `cargo build`, that `--no-default-features`
remains intentionally unsupported, and what was verified. Do not fabricate timestamps.

---

## Suggested commit sequence

1. `build: default to the full feature set so the default config compiles`
2. `ci: guard the default feature configuration with cargo check`
3. `docs: record default-build fix`

---

## Final done checklist

- [ ] `default = ["full"]` in `src-tauri/Cargo.toml`.
- [ ] A no-flag `cargo check` compiles.
- [ ] `--no-default-features` left intentionally unsupported (no gating changes).
- [ ] CI has a default-config `cargo check` step.
- [ ] `--all-features` gate still green.
- [ ] `memory.md` has a real UTC entry.
