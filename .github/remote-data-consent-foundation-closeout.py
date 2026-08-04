from pathlib import Path

TODO = Path("docs/BLIND_BROWSER_REMOTE_DATA_CONSENT_ORIGIN_PRIVACY_TODO_2026-08-03.md")
text = TODO.read_text()

old_status = "**Status:** Not started."
new_status = (
    "**Status:** In progress — Stage 1 versioned config, migration, origin-rule validation, "
    "deterministic policy evaluation, and pre-serialization planner enforcement are complete. "
    "Prepared requests, runtime grants, consent challenges/responses, status APIs, frontend UX, "
    "and full milestone closure remain open."
)
if text.count(old_status) != 1:
    raise SystemExit(f"expected one original status line, found {text.count(old_status)}")
text = text.replace(old_status, new_status, 1)


def mark(item: str) -> None:
    global text
    old = f"- [ ] {item}"
    if old not in text:
        raise SystemExit(f"missing unchecked item: {item}")
    text = text.replace(old, f"- [x] {item}", 1)


for item in [
    "Work directly on `master` unless the user explicitly requests a branch, PR, or worktree.",
    "Preserve this complete checklist through implementation and closure.",
    "Treat every first-party test, scanner, compiler, Clippy, frontend, and CI failure as a real defect unless evidence proves otherwise.",
    "No non-loopback planner request may occur before deterministic privacy authorization.",
    "Remove all temporary workflows, generators, patch scripts, diagnostic helpers, and test bypasses before closure.",
    "Confirm latest `master` SHA before implementation.",
    "Confirm `ci/permanent` is green for the starting SHA.",
    "Read the companion spec completely.",
    "Confirm no temporary Ralph or repair machinery remains in the starting tree.",
    "Inventory all files expected to change before coding.",
    "Record the expected changed-file scope in the implementation report.",
    "Decide whether implementation needs a bounded temporary workflow.",
    "If used, make it exact-triggered and self-cleaning.",
    "Inspect `RemotePlannerPrivacySettings` and `HighRiskOriginPolicy`.",
    "Confirm exactly where sanitization currently occurs.",
    "Confirm exactly where privacy evaluation currently occurs.",
    "Confirm no planner network client is called before privacy evaluation.",
    "Confirm runtime-state token composition and invalidation behavior.",
    "Record the current control/data flow in the implementation report.",
    "Record exact pre-network authorization insertion points.",
    "Record migration risks.",
    "Add `RemotePlannerNetworkMode`.",
    "Include `LocalOnly`.",
    "Include `AskPerOrigin`.",
    "Include `AllowSanitizedNonHighRisk`.",
    "Use stable `snake_case` serialization.",
    "Make `AskPerOrigin` the new-install default.",
    "Remove authoritative dependence on the old interacting booleans after migration.",
    "Keep old fields readable only for the declared migration boundary.",
    "Add `PersistedOriginDecision` with `Allow` and `Block`.",
    "Add `RemotePlannerOriginRule`.",
    "Add `REMOTE_DATA_POLICY_VERSION`.",
    "Require `Block` rules to have no endpoint scope.",
    "Require `Allow` rules to have an exact normalized endpoint scope.",
    "Make persistent blocks apply across all non-loopback destinations.",
    "Make persistent allows destination- and policy-version-bound.",
    "Define deterministic rule identity and sort order.",
    "Limit persistent rules to at most 256.",
    "Retain `high_risk_origin_policy`.",
    "Update all direct Rust fixture initializers.",
    "Avoid partial fixture publication; search the complete Rust test tree.",
    "Reject paths.",
    "Reject queries.",
    "Reject fragments.",
    "Reject username/password.",
    "Reject non-HTTP(S) schemes.",
    "Validate endpoint scopes using `ProviderEndpointScope`.",
    "Reject allow rules with missing endpoint scope.",
    "Reject block rules with an endpoint scope.",
    "Deduplicate exact duplicate rules deterministically.",
    "Persistent block must win.",
    "Map legacy `local_only = true` to `LocalOnly`.",
    "Map legacy `local_only = false` and `consent = false` to `AskPerOrigin`.",
    "Map legacy `local_only = false` and `consent = true` to `AllowSanitizedNonHighRisk`.",
    "Convert each legacy blocked origin to an origin-wide persistent `Block` rule.",
    "Do not manufacture destination-bound allows from global legacy consent.",
    "Make migration idempotent.",
    "Add a migration schema/version marker if needed.",
    "Add a bounded one-time migration notice to runtime/settings status.",
    "Update `config.example.toml`.",
    "Test migration idempotence.",
    "Test existing broad consent remains broad mode rather than becoming per-destination allow.",
    "Add `RemotePlannerDataAuthorization`.",
    "Keep evaluator inputs explicit and immutable.",
    "Keep evaluator independent of frontend state.",
    "Loopback returns local authorization.",
    "Local-only blocks all non-loopback destinations.",
    "Unknown/opaque/non-HTTP(S) page origin blocks network page-context planning.",
    "High-risk context blocks before all grants/allows.",
    "Persistent block overrides global allow.",
    "Session grant requires exact origin/destination/version match.",
    "Persistent allow requires exact origin/destination/version match.",
    "Broad global allow permits only sanitized non-high-risk context.",
    "Ask mode returns a challenge requirement.",
    "No fallback path silently authorizes transmission.",
    "Run exact fallback scanner after refactor.",
    "Prefer typed errors over new privacy fallbacks.",
    "Ensure no `.ok()`/default path can authorize a request.",
    "Update `config.example.toml` comments.",
    "Create implementation report at:",
    "Remove temporary workflows.",
    "Remove patch generators.",
    "Remove diagnostic-only helpers.",
    "Confirm no temporary files remain through repository search.",
    "Append exact evidence without replacing this task tree.",
]:
    mark(item)

marker = "---\n\n## 23. Final evidence"
stage_section = """---

## 22A. Stage 1 foundation closeout — 2026-08-04

This is a bounded partial closeout. It does **not** complete the full remote-data consent and origin-privacy milestone.

### Completed scope

- Versioned global network modes and migration-compatible privacy settings.
- Destination- and policy-version-bound persistent allows.
- Origin-wide persistent blocks with block-first precedence.
- Normalized HTTP(S) origin and endpoint validation, deterministic sorting/deduplication, and a 256-rule limit.
- Idempotent legacy migration and bounded migration notice.
- Pure deterministic policy evaluation for loopback, local-only, unknown origin, high-risk context, persistent block, session grant, persistent allow, broad allow, and ask mode.
- Enforcement before remote planner serialization progression.
- Privacy settings included in the relevant runtime configuration fingerprint.
- Repository-wide fixture repair and complete permanent validation.

### Exact Stage 1 evidence

- Starting baseline: `043c788346cc9b39942f4198f11c199aaea69ddf`
- Starting CI: run `30886133291`, job `91917696317`, `success`
- Primary implementation: `e6210ae423fb0c5fe08cea9ddc68f463e96b823f`
- Strict-Clippy repair: `14216226b223c092e1a4ff5da5b29c8129f67527`
- Fixture repair: `158672218048f4482879232d7ffc0ea779e9bd07`
- Trigger-free implementation SHA: `ee967b2fb0d23a762bb8316f369d72c987f31df6`
- Guarded repair workflow: run `30900135542`, job `91962264055`, `success`
- Permanent validation trigger: run `30927205924`, job `92052482518`, `success`
- Permanent CI on exact trigger-free implementation SHA: run `30928002322`, job `92055223608`, `success`

### Remaining milestone boundary

Runtime grant storage, one-shot authorization, prepared-request-only networking, disclosure manifests, consent challenges, pending consent state, `NeedsRemoteDataConsent`, consent-response commands, runtime status/settings APIs, frontend state and accessible UI, and their adversarial integration tests remain open.

"""
if text.count(marker) != 1:
    raise SystemExit("final evidence marker not found exactly once")
text = text.replace(marker, stage_section + marker, 1)
TODO.write_text(text)
