#!/usr/bin/env bash

set -euo pipefail

REQUIRED_NODE_VERSION="22.12.0"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCRIPT_NAME="$(basename "${BASH_SOURCE[0]}")"
IS_SOURCED=0

if [[ "${BASH_SOURCE[0]}" != "${0}" ]]; then
  IS_SOURCED=1
fi

fail() {
  echo "error: $*" >&2
  if [ "$IS_SOURCED" -eq 1 ]; then
    return 1
  fi
  exit 1
}

echo "==> Switching to Node.js ${REQUIRED_NODE_VERSION}"

export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
if [ ! -s "$NVM_DIR/nvm.sh" ]; then
  fail "nvm was not found at $NVM_DIR/nvm.sh"
fi

# shellcheck disable=SC1090
source "$NVM_DIR/nvm.sh"

nvm install "$REQUIRED_NODE_VERSION"
nvm use "$REQUIRED_NODE_VERSION"
hash -r

echo "Using Node.js version: $(node -p 'process.versions.node')"

cd "$REPO_ROOT"

echo "==> Reinstalling frontend dependencies"
rm -rf node_modules
pnpm install

echo "==> Done"
if [ "$IS_SOURCED" -eq 1 ]; then
  echo "Your current shell now uses Node.js $(node -p 'process.versions.node')."
  echo "You can now rerun:"
  echo "  pnpm build"
else
  echo "Dependencies were reinstalled with Node.js ${REQUIRED_NODE_VERSION},"
  echo "but this script cannot change the parent shell when executed normally."
  echo "To switch your current shell too, run:"
  echo "  source ./${SCRIPT_NAME}"
fi
