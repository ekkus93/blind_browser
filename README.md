# blind_browser

[![CI](https://github.com/ekkus93/blind_browser/actions/workflows/ci.yml/badge.svg)](https://github.com/ekkus93/blind_browser/actions/workflows/ci.yml)

Voice-first desktop browser for vision-impaired users, built with Rust and Tauri.

## Workspace Layout

- `src-tauri/`: Rust application shell, tool contracts, config models, and runtime state.
- `src/`: thin frontend shell served by Vite.
- `docs/`: product specs, skill catalog, and implementation plan.

## Current Status

Phase 0 project setup is in place:

- standard Tauri + Vite scaffold
- Rust module boundaries for browser, extraction, narration, ASR, TTS, OCR, config, state, and commands
- initial planner/tool schema layer matching the documented v1 contracts
- feature-gated dependency declarations for browser, OCR, audio, local TTS, local ASR, and remote planner integration via OpenAI-compatible APIs

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

Use these commands after bringing up a new machine or after changing native dependencies:

```bash
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
pnpm test:ui
pnpm build
```

If `cargo clippy --all-features` or `cargo test --all-features` fails in native dependencies, check the Linux prerequisite sections below first.

## Linux Tauri Prerequisites

The Rust crate uses Tauri's standard Linux runtime, which depends on GTK/WebKit development packages. `cargo check` will fail until those system libraries are installed.

Typical Ubuntu or Debian packages:

```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential clang libclang-dev curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev libasound2-dev
```

The exact package names can vary by distribution.

The audio stack depends on ALSA headers via `cpal` and `alsa-sys`. If `pkg-config` reports that it cannot find `alsa.pc`, the missing package is usually `libasound2-dev`.

Rust crates that use `bindgen` may also require `clang` and `libclang-dev`. If a build fails while parsing system headers with errors such as `fatal error: 'stddef.h' file not found`, install those packages and rerun the build.

The frontend build currently requires a Node.js version supported by Vite and its native bindings. If `pnpm build` reports that Vite needs `20.19+` or `22.12+`, switch to a supported version and reinstall JavaScript dependencies before retrying:

```bash
nvm install 22.12.0
nvm use 22.12.0
rm -rf node_modules
pnpm install
```

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

For a fast restart on another machine, the project handoff notes are tracked in `memory.md`. The latest entry should include the current commit, validation status, and where development stopped.
