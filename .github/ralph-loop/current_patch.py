from pathlib import Path


def replace_exact(path: str, old: str, new: str) -> None:
    target = Path(path)
    content = target.read_text(encoding="utf-8")
    count = content.count(old)
    if count != 1:
        raise RuntimeError(f"expected exactly one match in {path}, found {count}")
    target.write_text(content.replace(old, new, 1), encoding="utf-8")


def replace_count(path: str, old: str, new: str, expected: int) -> None:
    target = Path(path)
    content = target.read_text(encoding="utf-8")
    count = content.count(old)
    if count != expected:
        raise RuntimeError(f"expected {expected} matches in {path}, found {count}")
    target.write_text(content.replace(old, new), encoding="utf-8")


def create_new(path: str, content: str) -> None:
    target = Path(path)
    if target.exists():
        raise RuntimeError(f"refusing to overwrite existing file: {path}")
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8")


old_wry_test_attribute = """#[test]
#[cfg_attr(
    not(any(windows, target_os = \"linux\")),
    ignore = \"real Wry AppCore fixture requires Tauri's any-thread desktop builder\"
)]
"""
new_wry_test_attribute = """#[test]
#[cfg_attr(
    any(windows, target_os = \"linux\"),
    ignore = \"real Wry AppCore fixture must run in a process-isolated test invocation\"
)]
#[cfg_attr(
    not(any(windows, target_os = \"linux\")),
    ignore = \"real Wry AppCore fixture requires Tauri's any-thread desktop builder\"
)]
"""

replace_exact(
    "src-tauri/src/app_core/tests/confirmation_replay_tests.rs",
    old_wry_test_attribute,
    new_wry_test_attribute,
)
replace_count(
    "src-tauri/src/app_core/tests/remote_privacy_api_tests.rs",
    old_wry_test_attribute,
    new_wry_test_attribute,
    2,
)
replace_exact(
    "src-tauri/src/app_core/tests/remote_privacy_api_tests.rs",
    '        "http://api.example.com/v1",\n',
    "",
)

create_new(
    "scripts/run-rust-tests-linux.sh",
    """#!/usr/bin/env bash
set -euo pipefail

export CARGO_TERM_COLOR=never
export RUST_BACKTRACE=1

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${repo_root}"

if ! command -v xvfb-run >/dev/null 2>&1; then
  echo "xvfb-run is required for Linux desktop Rust tests" >&2
  exit 1
fi

run_isolated_wry_test() {
  local test_name="$1"
  local case_log
  case_log="$(mktemp)"

  echo "BEGIN isolated Wry test: ${test_name}"
  if ! xvfb-run -a cargo test \\
    --manifest-path src-tauri/Cargo.toml \\
    --all-features \\
    --lib \\
    "${test_name}" \\
    -- \\
    --ignored \\
    --exact \\
    --test-threads=1 \\
    2>&1 | tee "${case_log}"; then
    echo "Isolated Wry test command failed: ${test_name}" >&2
    return 1
  fi

  if ! grep -Eq '^running 1 test\\r?$' "${case_log}"; then
    echo "Isolated Wry invocation did not execute one test: ${test_name}" >&2
    return 1
  fi
  if ! grep -Fq "test result: ok. 1 passed; 0 failed; 0 ignored;" "${case_log}"; then
    echo "Isolated Wry invocation did not execute exactly one passing test: ${test_name}" >&2
    return 1
  fi

  rm -f -- "${case_log}"
  echo "PASS isolated Wry test: ${test_name}"
}

xvfb-run -a cargo test \\
  --manifest-path src-tauri/Cargo.toml \\
  --all-features

run_isolated_wry_test \\
  app_core::tests::confirmation_replay_tests::app_core_confirmation_replay_and_runtime_state_binding_are_enforced
run_isolated_wry_test \\
  app_core::tests::remote_privacy_api_tests::remote_privacy_status_reports_every_non_ephemeral_decision_and_stale_rules
run_isolated_wry_test \\
  app_core::tests::remote_privacy_api_tests::remote_privacy_operations_fail_closed_without_unnecessary_persistence
""",
)

replace_exact(
    ".github/workflows/ci.yml",
    """      - name: Run Rust tests
        run: xvfb-run -a cargo test --manifest-path src-tauri/Cargo.toml --all-features
""",
    """      - name: Run Rust tests
        run: bash scripts/run-rust-tests-linux.sh
""",
)
