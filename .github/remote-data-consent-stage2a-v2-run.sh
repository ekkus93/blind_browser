#!/usr/bin/env bash
set -euo pipefail

verify_chunk() {
  local path="$1"
  local size="$2"
  local expected_sha="$3"
  test "$(wc -c < "${path}")" -eq "${size}"
  test "$(sha256sum "${path}" | cut -d' ' -f1)" = "${expected_sha}"
}

verify_chunk .github/remote-data-consent-stage2a.payload00 800 cfabde7cf79d48a089d0588d6262c3c53e4e57e98085102ffc6ef646a0b23bc4
verify_chunk .github/remote-data-consent-stage2a.payload01 1600 56079316cfbced688bcba678a54491d941ff8b3b1d68c875c2cbba0e798b9bcd
verify_chunk .github/remote-data-consent-stage2a.payload02 1600 cb71c3f14b76959bc88ad63aefaf620ad23efe5781e00ad729a3e96082da2570
verify_chunk .github/remote-data-consent-stage2a.payload03 1600 e29d97d4129948b4fe2530d9b12ceb558578047520892bb10dfdd9b06e6ada8c
verify_chunk .github/remote-data-consent-stage2a.payload04 1600 efba2f6aeff76e44f3c3991c2b9c34a47ae6fe885096baa91337b1732fb14df5
verify_chunk .github/remote-data-consent-stage2a.payload05 1600 2d2605fa58fdd1331419cced79396eff51d6a4e555fff20041ff5b041a42fba4
verify_chunk .github/remote-data-consent-stage2a.payload06 1600 617745012c50abcc3ab3c6cfdb89d9ed8f81c918d5acb104418e50ea7a63d4a0
verify_chunk .github/remote-data-consent-stage2a.payload07 1600 beda467f9d47a53cff3257ec8f1bf5b3fff74683a6808b98119f006f534f6ea0
verify_chunk .github/remote-data-consent-stage2a.payload08 1600 dff9e9fdabbcd7af8a45c9350534e70aaca2d8d7d314d268ab0435e641cb79e5
verify_chunk .github/remote-data-consent-stage2a.payload09 1600 5e5c762192dabbff049318f07551e523aab2412352c7c6cb5c72c57088ecc5b8
verify_chunk .github/remote-data-consent-stage2a.payload10 1600 3f7db90a94b40fe8fb5c370a9538f83c5c34883e9d61afc448dda0a8c5bba3e7
verify_chunk .github/remote-data-consent-stage2a.payload11 532 3653edbe4685ead4d7d123cdc0a0f2f99e1eb7429f8ab3253dbc340409d4263d

cat .github/remote-data-consent-stage2a.payload{00..11} > /tmp/remote-data-consent-stage2a.py.gz.b64
test "$(wc -c < /tmp/remote-data-consent-stage2a.py.gz.b64)" -eq 17332
test "$(sha256sum /tmp/remote-data-consent-stage2a.py.gz.b64 | cut -d' ' -f1)" = 8f5f20570d4553c1ccbb0f44e172e54c9554cdcd9d0ff24ad2c34a7c003851f3
base64 -d /tmp/remote-data-consent-stage2a.py.gz.b64 | gzip -d > /tmp/remote-data-consent-stage2a.py
test "$(wc -c < /tmp/remote-data-consent-stage2a.py)" -eq 76782
test "$(sha256sum /tmp/remote-data-consent-stage2a.py | cut -d' ' -f1)" = eb8e80c60322db6807e31c08db52f2471017c1e0bf03f25959e523198fd9e363
python3 -m py_compile /tmp/remote-data-consent-stage2a.py
python3 /tmp/remote-data-consent-stage2a.py

python3 - <<'PY'
import re
from pathlib import Path


def replace_at_most_one(path: str, old: str, new: str, label: str) -> None:
    file_path = Path(path)
    text = file_path.read_text()
    count = text.count(old)
    if count > 1:
        raise SystemExit(f"{label}: expected at most one occurrence, found {count}")
    if count == 1:
        file_path.write_text(text.replace(old, new, 1))


replace_at_most_one(
    "src-tauri/src/app_core/planner_redaction.rs",
    "pub(crate) pub(crate) fn high_risk_context_reason",
    "pub(crate) fn high_risk_context_reason",
    "duplicate high-risk helper visibility",
)
replace_at_most_one(
    "src-tauri/src/commands/contracts/planner.rs",
    "use crate::app_core::remote_data_consent::RemotePlannerConsentChallenge;\n",
    "",
    "duplicate private consent challenge import",
)

consent_path = Path("src-tauri/src/app_core/remote_data_consent.rs")
consent = consent_path.read_text()

bad_type = consent.count("PersistedRemotePlannerPrivacySettings")
if bad_type > 1:
    raise SystemExit(
        f"privacy settings type repair: expected at most one bad type, found {bad_type}"
    )
if bad_type == 1:
    consent = consent.replace(
        "PersistedRemotePlannerPrivacySettings",
        "RemotePlannerPrivacySettings",
        1,
    )

lifetime_pattern = re.compile(
    r"fn matching_grant\(\s*"
    r"grants: &\[RemotePlannerEphemeralGrant\],\s*"
    r"draft: &RemotePlannerRequestDraft,\s*"
    r"challenge_digest: Option<&str>,\s*"
    r"now_ms: u64,\s*"
    r"\) -> Option<&RemotePlannerEphemeralGrant> \{"
)
lifetime_matches = list(lifetime_pattern.finditer(consent))
if len(lifetime_matches) > 1:
    raise SystemExit(
        "matching grant lifetime repair: expected at most one signature, "
        f"found {len(lifetime_matches)}"
    )
if len(lifetime_matches) == 1:
    consent = lifetime_pattern.sub(
        "fn matching_grant<'a>(\n"
        "    grants: &'a [RemotePlannerEphemeralGrant],\n"
        "    draft: &RemotePlannerRequestDraft,\n"
        "    challenge_digest: Option<&str>,\n"
        "    now_ms: u64,\n"
        ") -> Option<&'a RemotePlannerEphemeralGrant> {",
        consent,
        count=1,
    )

tool_name_pattern = re.compile(
    r"(use crate::commands::\{[^}]*?ToolError),\s*ToolName,([^}]*?\};)",
    re.DOTALL,
)
import_matches = list(tool_name_pattern.finditer(consent))
if len(import_matches) > 1:
    raise SystemExit(
        "unused ToolName import repair: expected at most one import block, "
        f"found {len(import_matches)}"
    )
if len(import_matches) == 1:
    consent = tool_name_pattern.sub(r"\1,\2", consent, count=1)

endpoint_display_old = "endpoint_display: sanitize_url_for_display(endpoint),"
endpoint_display_count = consent.count(endpoint_display_old)
if endpoint_display_count > 1:
    raise SystemExit(
        "endpoint display repair: expected at most one constructor, "
        f"found {endpoint_display_count}"
    )
if endpoint_display_count == 1:
    consent = consent.replace(
        endpoint_display_old,
        "endpoint_display: crate::provider_endpoint::ProviderEndpointScope::parse(endpoint)\n"
        "            .map(|scope| scope.normalized_base_url().to_string())\n"
        "            .unwrap_or_else(|_| String::from(\"invalid remote endpoint\")),",
        1,
    )

consent_path.write_text(consent)

remote_planner_path = Path("src-tauri/src/app_core/remote_planner.rs")
remote_planner = remote_planner_path.read_text()
profile_helper_signature = "pub(crate) fn remote_planner_profile_snapshot("
profile_helper_count = remote_planner.count(profile_helper_signature)
if profile_helper_count > 1:
    raise SystemExit(
        "remote planner profile snapshot repair: expected at most one helper, "
        f"found {profile_helper_count}"
    )
if profile_helper_count == 0:
    marker = "/// Resolve a planner output via the remote LLM"
    marker_count = remote_planner.count(marker)
    if marker_count != 1:
        raise SystemExit(
            "remote planner profile snapshot repair: expected one insertion marker, "
            f"found {marker_count}"
        )
    helper = '''impl AppCore {
    /// Snapshot the configured remote planner profile under the `AppCore` lock so
    /// network preparation can run against an owned, immutable copy.
    pub(crate) fn remote_planner_profile_snapshot(
        &self,
    ) -> Result<(String, RemotePlannerProfile), ToolError> {
        let Some(profile_name) = self.config.providers.planner.remote_profile.as_deref() else {
            return Err(planner_interpretation_unavailable_error(
                "planner_profile_unavailable",
                "remote planner mode requires a configured planner profile",
                false,
                None,
            ));
        };
        let Some(profile) = self.config.remote_planner_profiles.get(profile_name) else {
            return Err(planner_interpretation_unavailable_error(
                "planner_profile_unavailable",
                format!("configured remote planner profile '{profile_name}' was not found"),
                false,
                None,
            ));
        };
        Ok((profile_name.to_string(), profile.clone()))
    }
}

'''
    remote_planner = remote_planner.replace(marker, helper + marker, 1)
    remote_planner_path.write_text(remote_planner)
PY

cargo fmt --manifest-path src-tauri/Cargo.toml --all
git diff --check

bash scripts/check-silent-fallbacks.sh
python3 scripts/check-security-fallbacks.py --self-test
python3 scripts/check-security-fallbacks.py
python3 scripts/check-security-fallback-inventory.py --self-test
python3 scripts/check-security-fallback-inventory.py
python3 scripts/check-sensitive-diagnostics.py --self-test
python3 scripts/check-sensitive-diagnostics.py

cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
xvfb-run -a cargo test --manifest-path src-tauri/Cargo.toml --all-features remote_data_consent -- --nocapture
xvfb-run -a cargo test --manifest-path src-tauri/Cargo.toml --all-features
pnpm lint
pnpm test:ui
pnpm build

rm -f .github/workflows/remote-data-consent-stage2a.yml
rm -f .github/workflows/remote-data-consent-stage2a-repair.yml
rm -f .github/workflows/remote-data-consent-stage2a-repair-patcher.yml
rm -f .github/workflows/remote-data-consent-stage2a-repair-v2.yml
rm -f .github/workflows/remote-data-consent-stage2a-v2-guard-fix.yml
rm -f .github/workflows/remote-data-consent-stage2a-v2-guard-fix-line.yml
rm -f .github/remote-data-consent-stage2a.trigger
rm -f .github/remote-data-consent-stage2a-repair.trigger
rm -f .github/remote-data-consent-stage2a-repair-patcher.trigger
rm -f .github/remote-data-consent-stage2a-repair-v2.trigger
rm -f .github/remote-data-consent-stage2a-v2-guard-fix.trigger
rm -f .github/remote-data-consent-stage2a-v2-guard-fix-line.trigger
rm -f .github/remote-data-consent-stage2a-v2-run.sh
rm -f .github/remote-data-consent-stage2a.payload{00..11}
rm -f /tmp/remote-data-consent-stage2a.py /tmp/remote-data-consent-stage2a.py.gz.b64

git config user.name github-actions[bot]
git config user.email 41898282+github-actions[bot]@users.noreply.github.com
git add -A
test -n "$(git status --porcelain)"
for path in \
  .github/workflows/remote-data-consent-stage2a.yml \
  .github/workflows/remote-data-consent-stage2a-repair.yml \
  .github/workflows/remote-data-consent-stage2a-repair-patcher.yml \
  .github/workflows/remote-data-consent-stage2a-repair-v2.yml \
  .github/workflows/remote-data-consent-stage2a-v2-guard-fix.yml \
  .github/workflows/remote-data-consent-stage2a-v2-guard-fix-line.yml \
  .github/remote-data-consent-stage2a.trigger \
  .github/remote-data-consent-stage2a-repair.trigger \
  .github/remote-data-consent-stage2a-repair-patcher.trigger \
  .github/remote-data-consent-stage2a-repair-v2.trigger \
  .github/remote-data-consent-stage2a-v2-guard-fix.trigger \
  .github/remote-data-consent-stage2a-v2-guard-fix-line.trigger \
  .github/remote-data-consent-stage2a-v2-run.sh; do
  test ! -e "${path}"
done
for path in .github/remote-data-consent-stage2a.payload{00..11}; do
  test ! -e "${path}"
done

git commit -m "feat: add remote data consent transaction boundary"
git push origin HEAD:master
