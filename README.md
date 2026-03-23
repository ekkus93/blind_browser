# blind_browser

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
- feature-gated dependency declarations for browser, OCR, audio, local TTS, local ASR, and remote OpenAI integration

## Local Development

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

## Linux Tauri Prerequisites

The Rust crate uses Tauri's standard Linux runtime, which depends on GTK/WebKit development packages. `cargo check` will fail until those system libraries are installed.

Typical Ubuntu or Debian packages:

```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev libasound2-dev
```

The exact package names can vary by distribution.

The audio stack depends on ALSA headers via `cpal` and `alsa-sys`. If `pkg-config` reports that it cannot find `alsa.pc`, the missing package is usually `libasound2-dev`.

## Linux OCR Prerequisites

The OCR stack uses `leptess`, which depends on native Tesseract and Leptonica development libraries. Rust builds that enable OCR features, and lint commands such as `cargo clippy --all-features`, can fail until those system packages are installed.

Typical Ubuntu or Debian packages:

```bash
sudo apt install libleptonica-dev libtesseract-dev tesseract-ocr
```

If `pkg-config` reports that it cannot find `lept.pc`, the missing package is usually `libleptonica-dev`.

## Config

See `config.example.toml` for the initial shipped defaults and provider profile names.
