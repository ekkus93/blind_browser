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

# CR3 P2.2: this list documents intent and pins run order, but a hardcoded
# list that nothing checks against reality silently under-runs the moment it
# drifts -- an eighth #[ignore]d test added without a matching entry here
# would simply never run, with a green build (this is the exact shape of bug
# check-remote-planner-privacy-state.py's REQUIRED_PATHS and the old
# `test:ui` glob both had; both needed manual repair after silently
# under-running). verify_isolated_wry_test_list_is_complete below derives
# the real #[ignore]d set from `cargo test -- --ignored --list` and fails
# loudly on any divergence, in either direction, before any test runs.
ISOLATED_WRY_TESTS=(
  app_core::tests::confirmation_replay_tests::app_core_confirmation_replay_and_runtime_state_binding_are_enforced
  app_core::tests::remote_privacy_api_tests::remote_privacy_status_reports_every_non_ephemeral_decision_and_stale_rules
  app_core::tests::remote_privacy_api_tests::remote_privacy_operations_fail_closed_without_unnecessary_persistence
  app_core::tests::remote_data_consent_evidence_tests::replay_and_concurrency_tests::remote_data_consent_request_counts_replay_and_concurrency_are_enforced
  app_core::tests::remote_data_consent_evidence_tests::expiry_and_hostile_state_tests::remote_data_consent_expiry_invalidation_persistence_and_hostile_state_are_fail_closed
  app_core::tests::remote_data_consent_evidence_tests::identity_scope_and_restart_tests::remote_data_privacy_closure_identity_scope_and_restart_are_fail_closed
  app_core::tests::remote_data_consent_evidence_tests::policy_and_disclosure_matrix_tests::remote_data_privacy_closure_policy_and_disclosure_matrix_is_bounded
  app_core::tests::remote_data_consent_evidence_tests::narration_consent_tests::remote_narration_consent_policy_matrix_is_fail_closed
  app_core::tests::remote_data_consent_evidence_tests::narration_consent_tests::narration_allow_once_authorizes_exact_retry_once
  app_core::tests::remote_data_consent_evidence_tests::narration_consent_tests::remote_microphone_allow_once_authorizes_exact_retry_once
  app_core::tests::remote_data_consent_evidence_tests::narration_consent_tests::remote_microphone_consent_local_only_blocks_network_but_loopback_is_ungated
  app_core::click_authorization::tests::app_core_evidence_tests::click_authorization_subsystem_is_fail_closed
)

verify_isolated_wry_test_list_is_complete() {
  local listing
  listing="$(mktemp)"

  if ! cargo test \
    --manifest-path src-tauri/Cargo.toml \
    --all-features \
    --lib \
    -- \
    --ignored \
    --list \
    | sed -n 's/^\([a-zA-Z0-9_:]*\): test$/\1/p' \
    | sort -u \
    > "${listing}"; then
    echo "Could not list #[ignore]d tests to verify ISOLATED_WRY_TESTS completeness" >&2
    rm -f -- "${listing}"
    return 1
  fi

  local expected
  expected="$(mktemp)"
  printf '%s\n' "${ISOLATED_WRY_TESTS[@]}" | sort -u > "${expected}"

  local diff_output
  if ! diff_output="$(diff -u "${expected}" "${listing}")"; then
    echo "${diff_output}"
    echo "ISOLATED_WRY_TESTS in $(basename "${BASH_SOURCE[0]}") has drifted from the real" >&2
    echo "#[ignore]d test set (diff above: '-' = only in the hardcoded list, '+' = only" >&2
    echo "actually #[ignore]d). Update ISOLATED_WRY_TESTS to match -- every #[ignore]d" >&2
    echo "test must run somewhere, and this list must not claim tests that don't exist." >&2
    rm -f -- "${listing}" "${expected}"
    return 1
  fi

  rm -f -- "${listing}" "${expected}"
  echo "PASS: ISOLATED_WRY_TESTS matches the real #[ignore]d test set (${#ISOLATED_WRY_TESTS[@]} tests)"
}

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

verify_isolated_wry_test_list_is_complete

for isolated_wry_test in "${ISOLATED_WRY_TESTS[@]}"; do
  run_isolated_wry_test "${isolated_wry_test}"
done
