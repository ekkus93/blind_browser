# blind_browser

[![CI](https://github.com/ekkus93/blind_browser/actions/workflows/ci.yml/badge.svg)](https://github.com/ekkus93/blind_browser/actions/workflows/ci.yml)

Voice-first desktop browser for vision-impaired users, built with Rust and Tauri.

## Workspace Layout

- `src-tauri/`: Rust application shell, tool contracts, config models, and runtime state.
- `src/`: React 19 + Redux frontend shell served by Vite.
- `docs/`: product specs, skill catalog, and implementation plan.

## Current Status

The project is well past scaffold stage. The current repo includes:

- live Rust runtime support for browser control, extraction, narration, OCR, ASR, TTS, deterministic tool execution, and planner orchestration
- a React + Redux Tauri frontend for voice capture, confirmation flows, URL/navigation actions, runtime status, and settings panels
- persisted local/remote provider configuration plus model-management controls for local TTS and ASR profiles

Still intentionally incomplete:

- automatic provider failover is configured in schema but remains disabled in the live runtime

## Frontend Architecture

- `src/main.ts` mounts a single React app and keeps async runtime effects separate from presentational components.
- `src/app-shell-store.ts` is the frontend source of truth for shell view routing, settings subpages, panel state, and confirmation UI state.
- Panel renderers live under `src/settings-panels/` and `src/confirmation-panels/`, with stable barrel exports kept in `src/settings-status-panels.ts` and `src/confirmation-panel.ts`.
- React-owned handlers now drive live shell, URL, settings, confirmation, and nearby-control interactions. The remaining imperative DOM listeners are limited to masked API-key focus behavior and global push-to-talk release/cancel handling.
- Tauri commands stay behind the explicit async functions in `src/main.ts` and `src/tauri-api.ts`; presentational components do not call backend APIs directly.

## Local Development

Quick start on a new Linux machine:

1. Install Rust stable with `rustup`.
2. Install Node.js `20.19+` or `22.12+` and enable `pnpm` with `corepack enable`.
3. Install the Linux native packages from the Tauri and OCR prerequisite sections below.
4. Run `pnpm install` at the workspace root.
5. Use the validation commands in the `Validation` section to confirm the environment is complete.

Install frontend dependencies:

```bash
pnpm install
```

Run the frontend shell only:

```bash
pnpm dev
```

Build the frontend shell:

```bash
pnpm build
```

Format the Rust code:

```bash
cd src-tauri
cargo fmt
```

Run the Tauri app in development with the full native backend set:

```bash
pnpm tauri:dev:full
```

If you only need the OCR backend without the other optional native integrations:

```bash
pnpm tauri:dev:ocr
```

## Validation

Use these commands after bringing up a new machine or after changing native dependencies. They match what CI runs:

```bash
bash scripts/check-silent-fallbacks.sh
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
pnpm lint
pnpm test:ui
pnpm build
```

`scripts/check-silent-fallbacks.sh` is a local guardrail that catches reintroduced silent-fallback patterns (direct final-path model writes, missing masked-secret status, and similar regressions). Run it first to get a fast signal before the longer Rust build.

`cargo check` without `--all-features` catches breakage in the default feature set. `cargo clippy --all-features` and `cargo test --all-features` then verify the full feature set including native OCR and audio backends.

If `cargo clippy --all-features` or `cargo test --all-features` fails on native dependencies, check the Linux prerequisite sections below first.

## Linux Tauri Prerequisites

The Rust crate uses Tauri's standard Linux runtime, which depends on GTK/WebKit development packages. `cargo check` will fail until those system libraries are installed.

Typical Ubuntu or Debian packages:

```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential clang libclang-dev curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev libasound2-dev
```

The exact package names can vary by distribution.

The audio stack depends on ALSA headers via `cpal` and `alsa-sys`. If `pkg-config` reports that it cannot find `alsa.pc`, the missing package is usually `libasound2-dev`.

Rust crates that use `bindgen` may also require `clang` and `libclang-dev`. If a build fails while parsing system headers with errors such as `fatal error: 'stddef.h' file not found`, install those packages and rerun the build.

The frontend build currently requires a Node.js version supported by Vite and its native bindings. If `pnpm build` reports that Vite needs `20.19+` or `22.12+`, switch to a supported version in your current shell and reinstall JavaScript dependencies before retrying:

```bash
source ./fix-node-version.sh
```

If you execute `./fix-node-version.sh` normally, it will reinstall dependencies under `22.12.0` but cannot change the caller's shell. Use `source ./fix-node-version.sh` when you want the following `pnpm` commands in that shell to run under `22.12.0`.

## Linux OCR Prerequisites

The OCR stack uses `leptess`, which depends on native Tesseract and Leptonica development libraries. Rust builds that enable OCR features, and lint commands such as `cargo clippy --all-features`, can fail until those system packages are installed.

The desktop app only includes the OCR backend when the Rust `ocr` feature is enabled. Use `pnpm tauri:dev:ocr` for an OCR-only desktop run or `pnpm tauri:dev:full` for the full native stack.

Typical Ubuntu or Debian packages:

```bash
sudo apt install libleptonica-dev libtesseract-dev tesseract-ocr
```

If `pkg-config` reports that it cannot find `lept.pc`, the missing package is usually `libleptonica-dev`.

## Config

See `config.example.toml` for the initial shipped defaults and provider profile names.

Keyring-backed API keys

- The project now stores UI-entered remote API keys in the operating system keyring. When saving a remote profile API key from the Settings UI, the app writes the secret to the OS keyring and updates `config.toml` to store only a keyring reference (`from_keyring`).
- Remote API keys can also be sourced from environment variables or files via `SecretRef`.
- Saving an API key in the Settings UI always writes the secret to the OS keyring rather than storing plaintext in `config.toml`.
- CI and unit tests use an in-memory test keyring; production uses the OS keyring on supported platforms.

For a fast restart on another machine, the project handoff notes are tracked in `memory.md`. The latest entry should include the current commit, validation status, and where development stopped.
