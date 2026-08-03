# Blind Browser Post-P8 Fallback and Evidence Hardening Spec

**Date:** 2026-08-03  
**Repository:** `ekkus93/blind_browser`  
**Branch:** `master`  
**Companion TODO:** `docs/BLIND_BROWSER_POST_P8_FALLBACK_EVIDENCE_HARDENING_TODO_2026-08-03.md`  
**Depends on:** `docs/BLIND_BROWSER_POST_BATCH8_SECURITY_HARDENING_TODO_2026-08-02.md`  
**Scope:** Follow-up hardening for accepted fallback behavior, quiet degradation, and evidence-test quality after the bounded post-Batch-8 security-hardening pass.  
**Release boundary:** This is not the full BBCR remediation program and must not be used to declare the whole project release-ready.

---

## 1. Purpose

The post-Batch-8 hardening pass completed its bounded checklist and added important scanner and inventory enforcement. During follow-up review, the remaining risk was not that the P8 checklist was fake or obviously incomplete. The remaining risk is that some fallbacks are now documented and CI-gated but still operationally quiet enough to hide user-visible defects, missing capability, or evidence gaps.

This spec defines a focused hardening pass to convert the most important accepted quiet fallbacks into explicit typed absence, warning, or diagnostic paths, and to strengthen evidence tests that currently depend on source-text assertions rather than semantic contracts.

The goal is to keep the security posture fail-closed while improving diagnosability and audit quality.

---

## 2. Background and current state

The current `master` tree contains:

- `scripts/security-fallback-allowlist.txt`
- `scripts/security-fallback-inventory.json`
- `scripts/check-security-fallbacks.py`
- `scripts/check-security-fallback-inventory.py`
- `docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md`
- `src-tauri/tests/post_batch8_direct_command_policy_evidence.rs`
- permanent CI enforcement for fallback scanners, exact fallback inventory, diagnostic scanners, Rust checks/tests, frontend lint, UI tests, and build.

That foundation is good and must be preserved. This pass should not remove the scanner discipline. It should reduce the number of accepted quiet fallbacks where typed degraded behavior is now practical.

---

## 3. Non-goals

This pass must not attempt to complete the full remaining BBCR program. Specifically, this spec does not include:

- production release-readiness certification;
- full CSP/frontend network-boundary remediation;
- secret-history scanning and push protection;
- dependency/license/SAST gates;
- Windows/macOS packaged CI;
- fuzzing/property/mutation-testing programs;
- full model provenance/update UX;
- directory-level model transaction redesign;
- global Redux secret-draft removal;
- comprehensive persistence locking, fsync, and crash-consistency redesign.

Those should remain separate remediation tracks.

---

## 4. Definitions

### 4.1 Quiet fallback

A quiet fallback is code that converts an error, unavailable input, invalid parse, serialization failure, or missing artifact into a default, omission, skip, or generic result without making the reason observable to the caller, user, diagnostic surface, or test harness.

Examples include, but are not limited to:

- `.ok()` when the original error may be useful;
- `.unwrap_or_default()` when absence and empty value mean different things;
- `filter_map(Result::ok)` over filesystem or parser results;
- `let _ = ...` where the result is security-relevant or correctness-relevant;
- defaulting invalid configuration into unavailable capability without a typed reason;
- optional JSON detail suppression where the public error contract needs the details.

### 4.2 Accepted fallback

An accepted fallback is a quiet-looking construct that is intentionally permitted because it is capability-reducing, presentation-only, defense-in-depth, or irrelevant to authority. Accepted fallbacks must remain exact, documented, justified, and scanner-enforced.

### 4.3 Typed absence

Typed absence means the code represents why information is missing or degraded. This can be an enum, struct field, warning code, diagnostic event, or explicit error. The important property is that `missing`, `invalid`, `unreadable`, `unsupported`, `redacted`, and `not configured` do not all collapse into the same silent empty/default value.

### 4.4 Semantic evidence test

A semantic evidence test verifies behavior through registry APIs, typed policy objects, test doubles, or runtime-visible outcomes. It does not rely primarily on source-code string matching. Source-text tests may remain as supplemental tripwires, but they should not be the only evidence for an important security invariant.

---

## 5. Global requirements

1. Work directly from current `master` unless the user explicitly asks for a branch or PR.
2. Preserve permanent CI security scanner coverage.
3. Do not remove an accepted fallback from the allowlist unless the source change also removes the exact fallback expression or replaces it with a typed path.
4. Do not add new broad allowlist categories.
5. Every remaining accepted fallback must have a clear disposition:
   - permanent and justified;
   - temporary but acceptable for now;
   - converted to typed warning/error;
   - removed.
6. New warning or diagnostic data must remain privacy-preserving and path-private.
7. Do not leak raw page text, OCR text, transcript text, API keys, request bodies, full provider responses, signed URLs, private absolute paths, or sensitive query strings.
8. Preserve offline/local operation.
9. Do not weaken confirmation, planner, credential, URL, model-download, diagnostic, or hostile-content guards.
10. Preserve the original TODO checklist in future closure; do not replace a detailed task tree with a summary-only checklist.

---

## 6. Workstream A — Accepted fallback inventory triage

### 6.1 Requirements

Audit every entry in `scripts/security-fallback-inventory.json` and assign a disposition:

- `permanent_accepted`
- `temporary_accepted`
- `convert_to_warning`
- `convert_to_error`
- `remove`

Add metadata fields if needed, for example:

```json
{
  "disposition": "temporary_accepted",
  "review_due": "before_release_candidate",
  "owner_note": "Replace with typed settings absence reason once settings diagnostics can render it."
}
```

The inventory scanner must require disposition metadata for every entry.

### 6.2 Acceptance criteria

- The fallback inventory remains exact and synchronized with live source.
- Scanner self-tests cover missing disposition metadata.
- Permanent CI fails when a fallback lacks disposition metadata.
- The human-readable accepted-fallback document summarizes disposition counts.

---

## 7. Workstream B — Skill loading quiet skips

### 7.1 Problem

`load_skills_from_directory` still permits `filter_map(Result::ok)` for directory entries. This is safer than failing open because unreadable skills are omitted, but it is operationally quiet. If a user expects a skill to exist and it is unreadable, the system can silently behave as if the skill was never there.

### 7.2 Requirements

Replace or wrap the quiet skip with bounded path-private diagnostics:

- count unreadable directory entries;
- classify error kind without exposing full paths;
- optionally include a safe leaf name only if already approved by existing path-private diagnostics policy;
- expose the warning through an existing skill discovery diagnostic surface or a new bounded result structure;
- ensure unreadable entries do not grant tools, permissions, or authority.

The code may continue loading other valid skills. The important change is that omitted entries become observable in a safe way.

### 7.3 Acceptance criteria

- Tests prove unreadable entries are omitted but counted or warned.
- Tests prove full private paths are not exposed.
- Tests prove unreadable entries cannot add skill tools or permissions.
- The exact `filter_map(Result::ok)` fallback is removed or reclassified as a narrow internal helper with explicit warning aggregation.

---

## 8. Workstream C — Settings/model/provider typed absence

### 8.1 Problem

Some settings adapters intentionally collapse invalid provider endpoints or unknown model plans into unavailable capability. This is capability-reducing, but it can hide configuration mistakes.

### 8.2 Requirements

Introduce typed absence/degradation metadata for settings capability surfaces, for example:

- `not_configured`
- `invalid_endpoint`
- `unknown_model_id`
- `manifest_unavailable`
- `feature_disabled`
- `credential_reference_missing`
- `local_binary_unavailable`

The UI-facing settings model should be able to distinguish “feature is unavailable because it was not configured” from “feature is unavailable because configured data is invalid.”

### 8.3 Acceptance criteria

- Invalid remote planner base URL is surfaced as invalid configuration, not generic unavailable.
- Unknown Kitten/Whisper model ID is surfaced as unknown model, not generic unavailable.
- Feature-disabled state remains explicit.
- No raw provider URLs with credentials, query strings, or fragments are exposed.
- Existing settings tests are updated or extended.
- Any removed `.ok()`/`.unwrap_or_default()` entries are removed from the fallback allowlist and inventory.

---

## 9. Workstream D — Direct-command evidence quality

### 9.1 Problem

`src-tauri/tests/post_batch8_direct_command_policy_evidence.rs` provides useful exhaustive evidence, but parts of it are source-text based. That makes it brittle and weaker than semantic validation.

### 9.2 Requirements

Keep the exhaustive inventory and parity test, but move important invariants toward semantic tests:

- every direct command in the Tauri handler surface maps to `DirectCommandPolicy` metadata;
- every `performs_network_io` command maps to a tested network client/policy wrapper or typed network plan;
- every `credential_bearing_network_io` command maps to endpoint-bound credential scope resolution;
- every `downloads_executable_or_model_artifact` command maps to verified activation;
- every `transmits_page_context` command maps to privacy sanitization before network transmission.

Source-text checks may remain as supplemental drift detectors, but each important invariant should have a semantic proof where practical.

### 9.3 Acceptance criteria

- Tests fail when a new direct command lacks policy metadata.
- Tests fail when a networked command lacks a semantic network policy mapping.
- Tests fail when a credential-bearing command lacks endpoint-scope mapping.
- Tests fail when a page-context command lacks a sanitizer mapping.
- The test names should distinguish semantic tests from source-text drift checks.

---

## 10. Workstream E — URL sanitization ignored results

### 10.1 Problem

Some URL sanitization code intentionally ignores the return values of `Url::set_username` and `Url::set_password` after successful parsing. The output tests currently justify this. The risk is that future URL crate behavior or refactors could make mutation failure harder to reason about.

### 10.2 Requirements

Add a small helper that either:

- reconstructs sanitized URLs from approved components; or
- handles username/password mutation failures explicitly and falls back to a generic redacted URL.

### 10.3 Acceptance criteria

- No `let _ = parsed.set_username(...)` or `let _ = parsed.set_password(...)` remains in security-sensitive sanitization code unless it is isolated in one helper with explicit tests.
- Tests cover URLs with username, password, query, fragment, path, port, and malformed input.
- Sanitized output never contains credentials, query secrets, or fragments.

---

## 11. Workstream F — Optional label/default behavior in confirmation and scoring

### 11.1 Problem

Missing labels and optional element text currently default to empty strings in some scoring and confirmation-adjacent code. This is generally conservative, but the absence reason can be useful for debugging and for degraded-summary UX.

### 11.2 Requirements

Where practical, distinguish missing optional text from genuinely empty text:

- page model label missing;
- accessible name missing;
- placeholder missing;
- text missing;
- href missing;
- value intentionally redacted;
- value unavailable.

Do not make optional labels mandatory for execution if that would break legitimate pages. Instead, propagate bounded absence metadata into warning or scoring diagnostics where useful.

### 11.3 Acceptance criteria

- Missing protected-action labels produce explicit degraded metadata where user-facing confirmation depends on them.
- Scoring remains conservative.
- Missing labels cannot authorize clicks, lower confirmation, or mark destructive actions safe.
- Existing confirmation digest behavior remains stable except where new warning metadata intentionally changes it.

---

## 12. Workstream G — Optional policy-detail serialization

### 12.1 Problem

Some policy refusal paths serialize supplemental details with `.ok()`. The typed refusal remains visible, so this is not a fail-open bug. However, if policy details are important for debugging or audit, silent omission can hide useful evidence.

### 12.2 Requirements

Evaluate whether policy details should be mandatory for each refusal class:

- If details are optional, keep the fallback but document why omission is harmless.
- If details are part of the public diagnostic contract, replace `.ok()` with typed fallback detail such as `{ "detail_serialization": "failed" }`.

### 12.3 Acceptance criteria

- Policy refusal cannot become success if details fail to serialize.
- Any serialization failure is either explicitly represented or permanently justified in the inventory.
- Tests cover policy-detail serialization failure if a practical injection point exists.

---

## 13. Workstream H — TODO closure auditability

### 13.1 Problem

The post-Batch-8 TODO ended in a consolidated closure format. That is valid as a final status summary but weaker as an audit artifact because the original granular checklist is no longer visible in the current file.

### 13.2 Requirements

For this and future hardening passes:

- preserve the original task tree and individual checkboxes;
- append a final evidence section instead of replacing detailed tasks with summary sections;
- record exact source implementation SHA, cleanup SHA, final documentation SHA, CI run IDs, and job IDs;
- explicitly separate bounded-scope completion from broader release-readiness.

### 13.3 Acceptance criteria

- The TODO file remains useful as a line-by-line audit artifact after closure.
- Completion does not depend on reconstructing old checklists from Git history.
- Final evidence is appended, not substituted for the task tree.

---

## 14. Workstream I — Scanner and CI enforcement

### 14.1 Requirements

Enhance scanners only where necessary. Avoid writing brittle scanners that reject harmless code patterns broadly. Prefer exact path/expression enforcement for accepted fallbacks.

Permanent CI must continue running:

```text
bash scripts/check-silent-fallbacks.sh
python3 scripts/check-security-fallbacks.py --self-test
python3 scripts/check-security-fallbacks.py
python3 scripts/check-security-fallback-inventory.py --self-test
python3 scripts/check-security-fallback-inventory.py
python3 scripts/check-sensitive-diagnostics.py --self-test
python3 scripts/check-sensitive-diagnostics.py
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
xvfb-run -a cargo test --manifest-path src-tauri/Cargo.toml --all-features
pnpm lint
pnpm test:ui
pnpm build
```

Add focused tests for new behavior rather than relying only on scanners.

---

## 15. Expected deliverables

Implementation should update or add some subset of:

- `scripts/security-fallback-inventory.json`
- `scripts/security-fallback-allowlist.txt`
- `scripts/check-security-fallback-inventory.py`
- `docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md`
- `src-tauri/src/commands/skill_loader.rs`
- `src-tauri/src/commands/skill_parser.rs`
- `src-tauri/src/app_core/settings_adapters.rs`
- `src-tauri/src/app_core/planner_redaction.rs`
- `src-tauri/src/app_core/click_authorization.rs`
- `src-tauri/src/app_core/element_scoring.rs`
- `src-tauri/src/app_core/confirmation_workflow.rs`
- `src-tauri/src/commands/planner_executor/execution.rs`
- `src-tauri/tests/post_batch8_direct_command_policy_evidence.rs`
- focused new Rust tests if needed
- focused frontend tests if settings surfaces change
- this spec/TODO pair with final evidence appended on completion

The exact changed-file set may differ, but every changed source path must have corresponding tests or scanner evidence.

---

## 16. Completion definition

This pass is complete only when:

1. every task and subtask in the companion TODO is checked in place;
2. every remaining accepted fallback has disposition metadata;
3. the highest-value quiet fallbacks identified here are converted to typed warning/error/absence paths or explicitly retained with stronger justification;
4. semantic direct-command evidence exists for the major P8 direct-command invariants;
5. permanent CI passes on the exact final `master` SHA;
6. temporary workflows/scripts are removed or never introduced;
7. final documentation states what remains outside this bounded pass.

---

## 17. Final bounded statement template

When complete, the final report may say:

> The post-P8 fallback and evidence hardening pass is complete for its bounded scope. It reduces accepted quiet fallback behavior, improves typed diagnostics, and strengthens direct-command evidence quality. The broader BBCR remediation remains open, so this is not a general production release-readiness declaration.
