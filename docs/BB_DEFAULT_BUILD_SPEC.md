# Blind Browser Default Build Spec

## Purpose

The crate's default feature set (`default = []`) does not compile, and CI only ever
builds with `--all-features`, so the breakage is invisible to the gate. This is a
pre-existing condition in the feature-gated `browser` / `tts` / `asr` / `audio_io`
modules (per-item `#[cfg(feature = …)]` gating that is incomplete for the
no-feature configuration); it was not introduced by the async-runtime / Phase 3
work. This pass fixes the contributor-facing papercut.

Symptoms today:

- `cargo build` / `cargo check` with no `--features` fails to compile.
- rust-analyzer (which uses the default feature set unless configured otherwise)
  shows errors across the project for anyone who hasn't set `--features full`.
- `scripts/darkmode-test.sh` invokes a plain `cargo build --release` with no
  `--features`, which currently cannot succeed.

## Decision

This is an application crate, not a library — there are no downstream consumers
that need a minimal build, and no script or doc relies on `default = []` being
minimal (the only no-feature `cargo build`, in `darkmode-test.sh`, expects it to
*work*). The supported, CI-tested configuration is the full feature set.

**Primary fix: set `default = ["full"]`** so the default configuration matches the
only supported one. Then `cargo build`, `cargo check`, and rust-analyzer work with
no flags, and `darkmode-test.sh`'s plain `cargo build` succeeds.

Do **not** pursue completing the per-item gating to make `default = []` compile
unless a minimal build becomes an actual product goal. It currently buys nothing
(nothing builds or ships the minimal config) and is a much larger audit of the
`browser` / `tts` / `asr` / `audio_io` modules.

## Current known good behavior to preserve

- The `--all-features` gate must stay green exactly as today (clippy, fmt, tests,
  build). Setting `default = ["full"]` does not change `--all-features` behavior:
  `--all-features` already enables every real feature, so the compiled code is the
  same.
- The dev scripts (`tauri:dev:ocr`, `tauri:dev:full`) and the explicit-feature
  build requirement noted in `memory.md` are unaffected; they pass explicit
  `--features` and continue to work.
- No feature-gating logic inside the modules needs to change for the primary fix.

## Non-goals

- Do not refactor or "complete" the per-item `#[cfg]` gating in `browser` / `tts` /
  `asr` / `audio_io`.
- Do not change any feature definitions other than `default`.
- Do not alter the `--all-features` CI steps' semantics (a small additional check
  step is fine; see below).

## Design

1. In `src-tauri/Cargo.toml`, change `default = []` to `default = ["full"]`.
2. Confirm a plain `cargo build` / `cargo check` (no `--features`) now compiles
   (system libraries for the full set must be present — CI already installs them).
3. Add a lightweight CI guard so the default configuration cannot silently rot
   again: a `cargo check --manifest-path src-tauri/Cargo.toml` step (no feature
   flags — exercises the new default). This is cheap and catches default-config
   breakage directly, distinct from the existing `--all-features` steps.

If, and only if, a minimal build is later wanted, that is a separate effort:
complete the gating so `cargo check --no-default-features` compiles, then add that
invocation to CI. Out of scope here.

## Expected files touched

- `src-tauri/Cargo.toml` (`default = ["full"]`)
- `.github/workflows/ci.yml` (add the default-config `cargo check` step)
- `memory.md`

## Acceptance summary

```bash
# Default now resolves to the full feature set.
rg -n "^default = \[\"full\"\]" src-tauri/Cargo.toml

# Plain (no-feature) build/check compiles.
cargo check --manifest-path src-tauri/Cargo.toml

# CI guards the default configuration.
rg -n "cargo check --manifest-path src-tauri/Cargo.toml" .github/workflows/ci.yml
```

Expected: `default = ["full"]`; a no-flag `cargo check` compiles; CI has a default-
config check step.

Full validation gate (unchanged):

```bash
source ./fix-node-version.sh
pnpm lint
pnpm test:ui
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
```

Only after the gate passes: add a `memory.md` entry with a real UTC timestamp.
