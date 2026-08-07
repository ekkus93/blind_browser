#!/usr/bin/env bash
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
  if ! xvfb-run -a cargo test \
    --manifest-path src-tauri/Cargo.toml \
    --all-features \
    --lib \
    "${test_name}" \
    -- \
    --ignored \
    --exact \
    --test-threads=1 \
    2>&1 | tee "${case_log}"; then
    echo "Isolated Wry test command failed: ${test_name}" >&2
    return 1
  fi

  if ! grep -Eq '^running 1 test\r?$' "${case_log}"; then
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

xvfb-run -a cargo test \
  --manifest-path src-tauri/Cargo.toml \
  --all-features

run_isolated_wry_test \
  app_core::tests::confirmation_replay_tests::app_core_confirmation_replay_and_runtime_state_binding_are_enforced
run_isolated_wry_test \
  app_core::tests::remote_privacy_api_tests::remote_privacy_status_reports_every_non_ephemeral_decision_and_stale_rules
run_isolated_wry_test \
  app_core::tests::remote_privacy_api_tests::remote_privacy_operations_fail_closed_without_unnecessary_persistence
run_isolated_wry_test \
  app_core::tests::remote_data_consent_evidence_tests::replay_and_concurrency_tests::remote_data_consent_request_counts_replay_and_concurrency_are_enforced
run_isolated_wry_test \
  app_core::tests::remote_data_consent_evidence_tests::expiry_and_hostile_state_tests::remote_data_consent_expiry_invalidation_persistence_and_hostile_state_are_fail_closed
run_isolated_wry_test \
  app_core::tests::remote_data_consent_evidence_tests::identity_scope_and_restart_tests::remote_data_privacy_closure_identity_scope_and_restart_are_fail_closed
run_isolated_wry_test \
  app_core::tests::remote_data_consent_evidence_tests::policy_and_disclosure_matrix_tests::remote_data_privacy_closure_policy_and_disclosure_matrix_is_bounded
