#!/usr/bin/env bash

set -euo pipefail

REQUIRED_NODE_VERSION="22.12.0"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "==> Switching to Node.js ${REQUIRED_NODE_VERSION}"

export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
if [ ! -s "$NVM_DIR/nvm.sh" ]; then
  echo "error: nvm was not found at $NVM_DIR/nvm.sh" >&2
  echo "error: install nvm first, then rerun this script" >&2
  exit 1
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
echo "You can now rerun:"
echo "  pnpm build"
