#!/usr/bin/env bash
set -euo pipefail

verify_chunk() {
  local path="$1"
  local expected_length="$2"
  local expected_sha="$3"
  local cleaned actual_length actual_sha
  cleaned="$(tr -d '\n\r ' < "$path")"
  actual_length="${#cleaned}"
  actual_sha="$(printf '%s' "$cleaned" | sha256sum | cut -d' ' -f1)"
  printf '%s length=%s sha256=%s\n' "$path" "$actual_length" "$actual_sha" >&2
  test "$actual_length" -eq "$expected_length"
  test "$actual_sha" = "$expected_sha"
  printf '%s' "$cleaned"
}

run_logged() {
  local label="$1"
  shift
  local log="/tmp/remote-data-stage1-${label}.log"
  echo "::group::${label}"
  if "$@" >"${log}" 2>&1; then
    tail -n 40 "${log}" || true
    echo "::endgroup::"
    return 0
  fi
  local status=$?
  echo "::error::${label} failed with exit ${status}"
  tail -n 200 "${log}" || true
  echo "::endgroup::"
  return "${status}"
}

{
  verify_chunk .github/remote_data_stage1.part01 2500 04741158419a8377b745c2e9c50546677aa2e182be4dd1f31d9eae7ff6cf9e65
  verify_chunk .github/remote_data_stage1.part02a 1250 be0ccde475c6136d8a738731d29020d8efe8911d8723f94bd425575c997518c9
  verify_chunk .github/remote_data_stage1.part02b 1250 08c4fe02152464ec407a0ddbc7812fcf15191b293adfb8888a7aea22bce601cf
  verify_chunk .github/remote_data_stage1.part03 2500 d3b2e4f798038b0fc678c84cd5515380572b35c6032be6d824b70512819338e9
  verify_chunk .github/remote_data_stage1.part04 2124 c4de313f71a2da069c00cf8075924024d77ee4664a37c9593013a8df5a3ab9fb
} > /tmp/remote_data_stage1.b64

test "$(wc -c < /tmp/remote_data_stage1.b64)" -eq 9624
test "$(sha256sum /tmp/remote_data_stage1.b64 | cut -d' ' -f1)" = 9628593201373b78af534b3295454d24731e24ee78d4cf7be2cb0b5213973df2
base64 -d /tmp/remote_data_stage1.b64 > /tmp/remote_data_stage1.py.gz
test "$(sha256sum /tmp/remote_data_stage1.py.gz | cut -d' ' -f1)" = cc30b1c367c968a5012467b9ec884a8402b21b8ab82e243c485bef05c50a39c8
gzip -d -c /tmp/remote_data_stage1.py.gz > /tmp/remote_data_stage1.py
test "$(wc -c < /tmp/remote_data_stage1.py)" -eq 36790
test "$(sha256sum /tmp/remote_data_stage1.py | cut -d' ' -f1)" = 16b68ddb56abe77920cdbe74d7f49f5be27e373937c2e28933ea54615744817f
python3 -m py_compile /tmp/remote_data_stage1.py

python3 /tmp/remote_data_stage1.py

python3 - <<'PY'
from pathlib import Path

path = Path("src-tauri/src/app_core/planner_redaction.rs")
text = path.read_text()
malformed = '''        },
    
    ..Default::default()
    }
'''
corrected = '''        ..Default::default()
    }
}
'''
count = text.count(malformed)
if count != 1:
    raise SystemExit(
        f"planner_redaction.rs: expected one malformed privacy fixture, found {count}"
    )
text = text.replace(malformed, corrected, 1)

old_import = "    use crate::config::ProviderMode;\n"
new_import = "    use crate::config::{HighRiskOriginPolicy, ProviderMode};\n"
count = text.count(old_import)
if count != 1:
    raise SystemExit(
        f"planner_redaction.rs: expected one ProviderMode test import, found {count}"
    )
path.write_text(text.replace(old_import, new_import, 1))
PY

cargo fmt --manifest-path src-tauri/Cargo.toml --all

git diff --check
git status --short
test -n "$(git status --porcelain)"
test -f src-tauri/src/app_core/remote_data_consent.rs
grep -q 'RemotePlannerNetworkMode' src-tauri/src/config/types.rs
grep -q 'evaluate_remote_planner_policy' src-tauri/src/app_core/remote_data_consent.rs

bash scripts/check-silent-fallbacks.sh
python3 scripts/check-security-fallbacks.py --self-test
python3 scripts/check-security-fallbacks.py
python3 scripts/check-security-fallback-inventory.py --self-test
python3 scripts/check-security-fallback-inventory.py
python3 scripts/check-sensitive-diagnostics.py --self-test
python3 scripts/check-sensitive-diagnostics.py

run_logged cargo-check cargo check --manifest-path src-tauri/Cargo.toml
run_logged cargo-clippy cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
run_logged cargo-test xvfb-run -a cargo test --manifest-path src-tauri/Cargo.toml --all-features
run_logged pnpm-lint pnpm lint
run_logged pnpm-ui-tests pnpm test:ui
run_logged pnpm-build pnpm build

rm -f .github/workflows/remote-data-consent-stage1.yml
rm -f .github/workflows/remote-data-consent-stage1-format-diagnostic.yml
rm -f .github/remote-data-consent-stage1.trigger
rm -f .github/remote-data-consent-stage1-format.trigger
rm -f .github/remote-data-consent-stage1-run.sh
rm -f .github/remote_data_stage1.py.gz.b64
rm -f .github/remote_data_stage1.part01
rm -f .github/remote_data_stage1.part02
rm -f .github/remote_data_stage1.part02a
rm -f .github/remote_data_stage1.part02b
rm -f .github/remote_data_stage1.part03
rm -f .github/remote_data_stage1.part04
rm -f /tmp/remote_data_stage1.py /tmp/remote_data_stage1.py.gz /tmp/remote_data_stage1.b64
rm -f /tmp/remote-data-stage1-*.log

git config user.name github-actions[bot]
git config user.email 41898282+github-actions[bot]@users.noreply.github.com
git add -A
test -n "$(git status --porcelain)"
test ! -e .github/workflows/remote-data-consent-stage1.yml
test ! -e .github/workflows/remote-data-consent-stage1-format-diagnostic.yml
test ! -e .github/remote-data-consent-stage1.trigger
test ! -e .github/remote-data-consent-stage1-format.trigger
test ! -e .github/remote-data-consent-stage1-run.sh
test ! -e .github/remote_data_stage1.py.gz.b64
test ! -e .github/remote_data_stage1.part01
test ! -e .github/remote_data_stage1.part02
test ! -e .github/remote_data_stage1.part02a
test ! -e .github/remote_data_stage1.part02b
test ! -e .github/remote_data_stage1.part03
test ! -e .github/remote_data_stage1.part04

git commit -m "feat: add versioned remote data privacy policy foundation"
git push origin HEAD:master
