#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="/home/phil/work/blind_browser"
REQUIRED_NODE_VERSION="22.12.0"

echo "==> blind_browser dev environment setup"

version_ge() {
  [ "$(printf '%s\n' "$2" "$1" | sort -V | head -n 1)" = "$2" ]
}

if ! command -v sudo >/dev/null 2>&1; then
  echo "error: sudo is required for apt package installation" >&2
  exit 1
fi

if ! command -v curl >/dev/null 2>&1; then
  echo "error: curl is required to install rustup" >&2
  exit 1
fi

echo "==> Installing Rust with rustup if needed"
if ! command -v rustup >/dev/null 2>&1; then
  curl https://sh.rustup.rs -sSf | sh -s -- -y
fi

if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck disable=SC1090
  source "$HOME/.cargo/env"
fi

echo "==> Verifying Rust toolchain"
rustup default stable
rustc --version
cargo --version

echo "==> Verifying Node.js"
if ! command -v node >/dev/null 2>&1; then
  echo "error: Node.js 20.19+ or 22.12+ is required but node was not found" >&2
  exit 1
fi

node_version="$(node -p 'process.versions.node')"
echo "Current Node.js version: $node_version"

if ! version_ge "$node_version" "$REQUIRED_NODE_VERSION"; then
  echo "==> Upgrading Node.js with nvm to $REQUIRED_NODE_VERSION"

  export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
  if [ ! -s "$NVM_DIR/nvm.sh" ]; then
    echo "error: Node.js $node_version is too old and nvm was not found at $NVM_DIR/nvm.sh" >&2
    echo "error: install Node.js >= $REQUIRED_NODE_VERSION or install nvm and rerun this script" >&2
    exit 1
  fi

  # shellcheck disable=SC1090
  source "$NVM_DIR/nvm.sh"
  nvm install "$REQUIRED_NODE_VERSION"
  nvm use "$REQUIRED_NODE_VERSION"
  hash -r

  node_version="$(node -p 'process.versions.node')"
  echo "Using Node.js version: $node_version"
fi

echo "==> Ensuring pnpm is available"
npm_global_prefix="$(npm prefix -g)"
npm_global_bin="$npm_global_prefix/bin"
pnpm_bin="$npm_global_bin/pnpm"

if [ -L "$pnpm_bin" ] && [ "$(readlink "$pnpm_bin")" = "../lib/node_modules/corepack/dist/pnpm.js" ]; then
  rm -f "$pnpm_bin"
fi

npm install -g --force pnpm

if [ -d "$npm_global_bin" ] && [[ ":$PATH:" != *":$npm_global_bin:"* ]]; then
  export PATH="$npm_global_bin:$PATH"
fi
"$pnpm_bin" --version

echo "==> Installing Ubuntu native dependencies"
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  clang \
  libclang-dev \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libasound2-dev \
  libleptonica-dev \
  libtesseract-dev \
  tesseract-ocr

echo "==> Verifying native libraries"
pkg-config --modversion webkit2gtk-4.1
pkg-config --modversion alsa
pkg-config --modversion lept
pkg-config --modversion tesseract
clang --version

echo "==> Installing JavaScript dependencies"
cd "$REPO_ROOT"
if [ -d node_modules ]; then
  echo "==> Removing node_modules to refresh optional native bindings"
  rm -rf node_modules
fi
"$pnpm_bin" install

echo "==> Running project validation"
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
"$pnpm_bin" test:ui
"$pnpm_bin" build

echo "==> Setup complete"
