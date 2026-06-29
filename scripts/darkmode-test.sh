#!/usr/bin/env bash
# darkmode-test.sh — Build container, launch blind_browser in dark mode,
# screenshot each major panel, assert no panel regions are bright parchment.
#
# Prerequisites:
#   - Docker running
#   - App already built: pnpm build && cargo build --release --manifest-path src-tauri/Cargo.toml
#
# Usage:
#   bash scripts/darkmode-test.sh
#   KEEP_CONTAINER=1 bash scripts/darkmode-test.sh   # skip cleanup for debugging

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
IMAGE="blind-browser-darkmode-test"
CONTAINER="blind-browser-darkmode-run"
SCREENSHOT_DIR="$REPO_ROOT/darkmode-screenshots"
FAIL=0

# ─── Cleanup ──────────────────────────────────────────────────────────────────

cleanup() {
  if [[ "${KEEP_CONTAINER:-0}" == "1" ]]; then
    echo "[darkmode-test] KEEP_CONTAINER=1 — leaving container $CONTAINER running"
    return
  fi
  echo "[darkmode-test] Stopping and removing container..."
  docker stop "$CONTAINER" 2>/dev/null || true
  docker rm   "$CONTAINER" 2>/dev/null || true
}
trap cleanup EXIT

# ─── Check prerequisites ──────────────────────────────────────────────────────

BINARY="$REPO_ROOT/src-tauri/target/release/blind_browser"
if [[ ! -f "$BINARY" ]]; then
  echo "[darkmode-test] ERROR: Binary not found at $BINARY"
  echo "  Run: cargo build --release --manifest-path src-tauri/Cargo.toml"
  exit 1
fi

if [[ ! -d "$REPO_ROOT/dist" ]]; then
  echo "[darkmode-test] ERROR: Frontend dist not found at $REPO_ROOT/dist"
  echo "  Run: pnpm build"
  exit 1
fi

# ─── Build image ──────────────────────────────────────────────────────────────

echo "[darkmode-test] Building Docker image..."
docker build \
  -f "$REPO_ROOT/Dockerfile.darkmode-test" \
  -t "$IMAGE" \
  "$REPO_ROOT"

# ─── Start container ──────────────────────────────────────────────────────────

echo "[darkmode-test] Starting container..."
docker stop "$CONTAINER" 2>/dev/null || true
docker rm   "$CONTAINER" 2>/dev/null || true

docker run -d \
  --name "$CONTAINER" \
  --shm-size=256m \
  "$IMAGE"

# Give Xvfb + app time to start up
echo "[darkmode-test] Waiting for app to start (5s)..."
sleep 5

# ─── Helper functions ─────────────────────────────────────────────────────────

# exec_display: run a command inside the container with DISPLAY and GTK_THEME set
exec_display() {
  docker exec "$CONTAINER" bash -c "export DISPLAY=:99; export GTK_THEME=Adwaita:dark; $*"
}

# screenshot <name>: take a screenshot and copy it to SCREENSHOT_DIR/<name>.png
screenshot() {
  local name="$1"
  mkdir -p "$SCREENSHOT_DIR"
  docker exec "$CONTAINER" bash -c "export DISPLAY=:99; scrot /tmp/${name}.png"
  docker cp "$CONTAINER:/tmp/${name}.png" "$SCREENSHOT_DIR/${name}.png"
  echo "[darkmode-test] Screenshot: $SCREENSHOT_DIR/${name}.png"
}

# check_region_dark <screenshot_path> <x> <y> <w> <h> <label>
# Crops the given region and measures mean luminance (0.0–1.0).
# Fails if mean > 0.45 (i.e. the region is too bright for dark mode).
check_region_dark() {
  local img="$1" x="$2" y="$3" w="$4" h="$5" label="$6"
  local luminance
  luminance=$(convert "$img" \
    -crop "${w}x${h}+${x}+${y}" +repage \
    -colorspace Gray \
    -format "%[fx:mean]" info: 2>/dev/null)

  # Use awk for float comparison (bash can't do floats)
  local is_dark
  is_dark=$(awk -v lum="$luminance" 'BEGIN { print (lum < 0.45) ? "yes" : "no" }')

  if [[ "$is_dark" == "yes" ]]; then
    echo "  PASS  $label  (luminance=$luminance)"
  else
    echo "  FAIL  $label  (luminance=$luminance — expected < 0.45 for dark mode)"
    FAIL=1
  fi
}

# ─── Navigate and screenshot each panel ──────────────────────────────────────

# Window is 1280x900. We'll click through panels using xdotool.
# All coordinates are approximate center-points for a 1280x900 window.

echo ""
echo "[darkmode-test] === Workspace (home) ==="
sleep 1
screenshot "01-workspace"

echo ""
echo "[darkmode-test] === Settings (click settings gear button, top-left) ==="
# Settings gear is the round button at top-left of the toolbar (~197, 79)
exec_display "xdotool mousemove 197 79 click 1"
sleep 2
screenshot "02-settings-overview"

echo ""
echo "[darkmode-test] === Planner settings (AI assistant) ==="
# Click the "Open AI assistant setup" button (right-side nav button, x≈920, y≈562)
exec_display "xdotool mousemove 920 562 click 1"
sleep 2
screenshot "03-settings-planner"

# Go back to settings overview, then re-open settings for next subpage
exec_display "xdotool mousemove 197 79 click 1"
sleep 1

echo ""
echo "[darkmode-test] === TTS settings (Voice output) ==="
# "Open voice output setup" button at x≈920, y≈644
exec_display "xdotool mousemove 920 644 click 1"
sleep 2
screenshot "04-settings-tts"

exec_display "xdotool mousemove 197 79 click 1"
sleep 1

echo ""
echo "[darkmode-test] === ASR settings (Voice input) ==="
# "Open voice input setup" button at x≈920, y≈726
exec_display "xdotool mousemove 920 726 click 1"
sleep 2
screenshot "05-settings-asr"

exec_display "xdotool mousemove 197 79 click 1"
sleep 1

echo ""
echo "[darkmode-test] === Advanced/Runtime settings ==="
# "Open advanced settings" button at x≈920, y≈808
exec_display "xdotool mousemove 920 808 click 1"
sleep 2
screenshot "06-settings-runtime"

# Back out of subpage, then back out of settings to workspace
exec_display "xdotool mousemove 197 79 click 1"
sleep 1
exec_display "xdotool mousemove 197 79 click 1"
sleep 1
screenshot "07-workspace-return"

# ─── Pixel brightness checks ─────────────────────────────────────────────────
# Regions to check (x, y, width, height) at 1280x900.
# These target the main content area, avoiding the OS window border.
# Adjust coordinates if the app layout changes.

echo ""
echo "[darkmode-test] === Brightness checks (luminance must be < 0.45) ==="

# 01-workspace: main panel grid area
check_region_dark "$SCREENSHOT_DIR/01-workspace.png"    100 150 1080 600 "workspace panels"
check_region_dark "$SCREENSHOT_DIR/01-workspace.png"    100  40 1080  80 "toolbar"

# 02-settings-overview: settings card list
check_region_dark "$SCREENSHOT_DIR/02-settings-overview.png" 200 200  880 550 "settings overview cards"

# 03-settings-planner: planner settings content
check_region_dark "$SCREENSHOT_DIR/03-settings-planner.png"  200 200  880 550 "planner settings"

# 04-settings-tts
check_region_dark "$SCREENSHOT_DIR/04-settings-tts.png"      200 200  880 550 "TTS settings"

# 05-settings-asr
check_region_dark "$SCREENSHOT_DIR/05-settings-asr.png"      200 200  880 550 "ASR settings"

# 06-settings-runtime
check_region_dark "$SCREENSHOT_DIR/06-settings-runtime.png"  200 200  880 550 "Runtime settings"

# ─── Report ───────────────────────────────────────────────────────────────────

echo ""
if [[ "$FAIL" -eq 0 ]]; then
  echo "[darkmode-test] ALL CHECKS PASSED — no bright panel regions found in dark mode"
else
  echo "[darkmode-test] SOME CHECKS FAILED — see luminance values above"
  echo "  Screenshots saved to: $SCREENSHOT_DIR"
  echo "  Rerun with KEEP_CONTAINER=1 to inspect the running container"
  exit 1
fi

echo "  Screenshots saved to: $SCREENSHOT_DIR"
