#!/usr/bin/env bash
# Guard against regressions fixed in the security & silent-failure hardening pass
# (docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING_SPEC.md).
#
# Each entry is the EXACT shape that was removed, deliberately narrow to avoid false
# positives on legitimate code. Notably this does NOT ban `let _ = timeout_ms` (still
# valid in feature-disabled stubs and non-navigation handlers) or a bare
# `unwrap_or(false)` (legitimate elsewhere) — only the precise reintroductions.
set -euo pipefail

cd "$(dirname "$0")/.."

# Roots scanned for each pattern. One pattern (the CSP) lives in the Tauri config,
# the rest in Rust source.
declare -A pattern_roots=(
  ['let _ = load_state']='src-tauri/src'
  ['"csp": null']='src-tauri/tauri.conf.json'
  ['filter_map(|segment| segment.to_str_lossy().ok())']='src-tauri/src'
  ['parse_bool_value(value).unwrap_or(false)']='src-tauri/src'
  ['.get_page_metrics().ok()?']='src-tauri/src'
)

status=0
for pattern in "${!pattern_roots[@]}"; do
  root="${pattern_roots[$pattern]}"
  if grep -rIn -F -- "$pattern" "$root" 2>/dev/null; then
    echo "ERROR: forbidden silent-fallback pattern reintroduced in $root: $pattern" >&2
    status=1
  fi
done

if [ "$status" -ne 0 ]; then
  echo "" >&2
  echo "Silent-fallback guard failed. These patterns were removed in the security and" >&2
  echo "silent-failure hardening pass and must not return. See" >&2
  echo "docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING_SPEC.md." >&2
fi
exit "$status"
