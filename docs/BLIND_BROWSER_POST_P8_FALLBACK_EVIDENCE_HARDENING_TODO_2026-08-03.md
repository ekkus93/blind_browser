# Blind Browser Post-P8 Fallback and Evidence Hardening TODO

**Date:** 2026-08-03  
**Repository:** `ekkus93/blind_browser`  
**Branch:** `master`  
**Companion spec:** `docs/BLIND_BROWSER_POST_P8_FALLBACK_EVIDENCE_HARDENING_SPEC_2026-08-03.md`  
**Implementation report:** `docs/BLIND_BROWSER_POST_P8_FALLBACK_EVIDENCE_HARDENING_IMPLEMENTATION_REPORT_2026-08-03.md`  
**Depends on:** `docs/BLIND_BROWSER_POST_BATCH8_SECURITY_HARDENING_TODO_2026-08-02.md`  
**Status:** Implementation complete; final permanent CI evidence pending on the reconciled TODO candidate.  
**Release boundary:** This TODO is a bounded follow-up pass for accepted fallback behavior, quiet degradation, and evidence-test quality. It is not the full BBCR remediation program and must not be used to declare general release readiness.

---

## Completion rules

- Do not check a task unless the implementation, test, scanner, or documentation evidence is present on `master`.
- Preserve this detailed checklist. At closure, append final evidence instead of replacing this task tree with a summary-only checklist.
- Record exact implementation SHA, cleanup SHA if any, final documentation SHA, permanent CI run ID, and job ID.
- Do not add branch/PR/worktree language unless the user explicitly requests that workflow.
- Do not weaken existing P8 security guards.
- Do not broaden this TODO into the entire remaining BBCR program.

---

## 0. Baseline and operating rules

- [x] Confirm latest `master` SHA before starting implementation.
- [x] Confirm permanent CI state for the starting SHA.
- [x] Read `docs/BLIND_BROWSER_POST_P8_FALLBACK_EVIDENCE_HARDENING_SPEC_2026-08-03.md`.
- [x] Read `docs/BLIND_BROWSER_POST_BATCH8_SECURITY_HARDENING_TODO_2026-08-02.md`.
- [x] Read `docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md`.
- [x] Read `scripts/security-fallback-allowlist.txt`.
- [x] Read `scripts/security-fallback-inventory.json`.
- [x] Read `scripts/check-security-fallbacks.py`.
- [x] Read `scripts/check-security-fallback-inventory.py`.
- [x] Confirm no temporary workflow/script from the prior Ralph run remains in the production tree.
- [x] Confirm this pass will work directly from `master` unless the user gives different instructions.

---

## 1. Accepted fallback inventory triage

### 1.1 Inventory review

- [x] Enumerate every entry in `scripts/security-fallback-inventory.json`.
- [x] Group entries by source category:
  - [x] click authorization / element scoring;
  - [x] command dispatch / fill correction / field focus;
  - [x] confirmation workflow;
  - [x] planner redaction / URL sanitization;
  - [x] settings/model/provider capability discovery;
  - [x] numeric conversion;
  - [x] policy-detail serialization;
  - [x] skill loader / skill parser;
  - [x] frontend redaction / UI helper fallbacks, if present.
- [x] For every entry, determine whether the current fallback is:
  - [x] permanently justified;
  - [x] temporarily justified;
  - [x] convertible to typed warning;
  - [x] convertible to typed error;
  - [x] removable.

### 1.2 Inventory metadata

- [x] Add a `disposition` or equivalent field to every machine-readable inventory entry.
- [x] Add a `review_due` or equivalent field for temporary accepted fallbacks.
- [x] Add a short `owner_note` or equivalent field for every temporary accepted fallback.
- [x] Update `scripts/check-security-fallback-inventory.py` to require the new metadata.
- [x] Add scanner self-test coverage for:
  - [x] missing disposition;
  - [x] invalid disposition;
  - [x] temporary fallback missing review boundary;
  - [x] stale source expression;
  - [x] stale containing function.
- [x] Update `docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md` with disposition counts and policy.

### 1.3 Acceptance gate

- [x] Run `python3 scripts/check-security-fallback-inventory.py --self-test`.
- [x] Run `python3 scripts/check-security-fallback-inventory.py`.
- [x] Confirm permanent CI includes the enhanced inventory check.

---

## 2. Skill loading quiet-skip hardening

### 2.1 Audit

- [x] Inspect `src-tauri/src/commands/skill_loader.rs`.
- [x] Locate every `filter_map(Result::ok)` or equivalent skip in skill loading.
- [x] Identify what errors can be skipped:
  - [x] unreadable directory entry;
  - [x] invalid file type;
  - [x] unreadable skill manifest;
  - [x] parse failure;
  - [x] permission denied;
  - [x] symlink/path policy rejection.
- [x] Determine which errors are already surfaced elsewhere and which are currently quiet.

Audit result: directory-entry I/O errors were the quiet path and are now aggregated. Manifest read errors, parse errors, and name mismatches were already path-private warnings. Non-directory entries are non-skill candidates rather than failures. The loader does not grant authority from an omitted or rejected entry; broader filesystem/symlink policy remains part of the larger BBCR filesystem review.

### 2.2 Implementation

- [x] Replace direct quiet skipping of unreadable directory entries with explicit warning aggregation.
- [x] Preserve ability to load other valid skills when one optional entry is unreadable.
- [x] Record bounded warning metadata:
  - [x] count of skipped entries;
  - [x] error kind or category;
  - [x] source class;
  - [x] safe leaf identifier only if allowed by path-privacy rules.
- [x] Ensure full absolute paths are not exposed.
- [x] Ensure skipped entries cannot add skills, tools, or permissions.
- [x] Ensure warnings are visible through an existing diagnostic/status path or a new bounded return field.

The implementation uses the existing structured tracing diagnostic path and records source class, aggregate count, and bounded error categories only.

### 2.3 Tests

- [x] Test unreadable entry is omitted but produces a bounded warning.
- [x] Test multiple unreadable entries aggregate without leaking full paths.
- [x] Test valid adjacent skills still load.
- [x] Test skipped entries cannot grant tools or permissions.
- [x] Test diagnostics remain path-private.
- [x] Remove or reclassify the old skill-loader fallback entry from the allowlist/inventory.

---

## 3. Settings/model/provider typed absence hardening

### 3.1 Audit

- [x] Inspect `src-tauri/src/app_core/settings_adapters.rs`.
- [x] Identify fallbacks that collapse invalid settings into generic unavailable capability.
- [x] Audit remote planner base URL parsing fallbacks.
- [x] Audit Kitten TTS model-plan lookup fallbacks.
- [x] Audit Whisper ASR model-plan lookup fallbacks.
- [x] Audit voice/settings defaults that use `.unwrap_or_default()`.

### 3.2 Typed absence design

- [x] Define typed absence/degradation reasons, such as:
  - [x] `not_configured`;
  - [x] `invalid_endpoint`;
  - [x] `unknown_model_id`;
  - [x] `manifest_unavailable`;
  - [x] `feature_disabled`;
  - [x] `credential_reference_missing`;
  - [x] `local_binary_unavailable`.
- [x] Decide the Rust type location for the absence reasons.
- [x] Decide the frontend shape if settings UI receives these reasons.
- [x] Ensure reason fields do not include raw credentials, query strings, fragments, full local paths, or raw provider responses.

### 3.3 Implementation

- [x] Surface invalid remote planner endpoint distinctly from not configured.
- [x] Surface unknown Kitten TTS model ID distinctly from not configured.
- [x] Surface unknown Whisper ASR model ID distinctly from not configured.
- [x] Preserve explicit feature-disabled behavior.
- [x] Preserve fail-closed behavior for actual operations.
- [x] Remove or narrow fallback inventory entries that are converted to typed absence.

### 3.4 Tests

- [x] Test invalid endpoint reason.
- [x] Test unknown TTS model reason.
- [x] Test unknown ASR model reason.
- [x] Test not-configured reason.
- [x] Test feature-disabled reason if applicable.
- [x] Test sanitized output for endpoint strings containing credentials/query/fragment.
- [x] Update frontend tests if settings output shape changes.

Feature-disabled remote TTS remains an existing explicit typed failure and feature-matrix contract; the new settings reason enum reserves the matching UI state without weakening the operation path.

---

## 4. Direct-command semantic evidence hardening

### 4.1 Audit current evidence test

- [x] Inspect `src-tauri/tests/post_batch8_direct_command_policy_evidence.rs`.
- [x] Identify assertions that depend primarily on source-code string search.
- [x] Identify which invariants already have semantic coverage elsewhere.
- [x] Identify which invariants have only text-search coverage.

### 4.2 Semantic registry/API mapping

- [x] Ensure every command in `tauri::generate_handler!` maps to `DirectCommandPolicy` or equivalent metadata.
- [x] Ensure every direct command policy can be inspected by tests without parsing source text.
- [x] Add or expose a typed mapping for network client/policy classes where practical.
- [x] Add or expose a typed mapping for credential-scope requirement where practical.
- [x] Add or expose a typed mapping for verified model-download activation where practical.
- [x] Add or expose a typed mapping for page-context sanitizer requirement where practical.

### 4.3 Tests

- [x] Add semantic test: every handler has direct-command policy metadata.
- [x] Add semantic test: every networked command has a network policy/client mapping.
- [x] Add semantic test: every credential-bearing command has endpoint-bound credential mapping where applicable.
- [x] Add semantic test: every model-download command maps to verified activation.
- [x] Add semantic test: every page-context-transmitting command maps to privacy sanitization.
- [x] Retain source-text checks only as supplemental drift detectors.
- [x] Rename or split tests so semantic checks and source-text drift checks are clearly distinguished.

### 4.4 Acceptance gate

- [x] Run the direct-command evidence test by itself.
- [x] Run the full Rust test suite.
- [x] Confirm a new networked command without metadata would fail tests.

Permanent CI contains a focused direct-command evidence step in addition to the full Rust suite.

---

## 5. URL sanitization ignored-result hardening

### 5.1 Audit

- [x] Inspect `src-tauri/src/app_core/planner_redaction.rs`.
- [x] Locate `let _ = parsed.set_username("")`.
- [x] Locate `let _ = parsed.set_password(None)`.
- [x] Search for other ignored URL mutation results in Rust and frontend code.

### 5.2 Implementation

- [x] Add a helper that reconstructs sanitized URLs from approved components, or handles URL mutation failure explicitly.
- [x] Ensure the helper strips:
  - [x] username;
  - [x] password;
  - [x] query;
  - [x] fragment;
  - [x] known token-like material.
- [x] Ensure malformed input returns a generic redacted URL or typed invalid result.
- [x] Remove broad ignored-result fallback entries if no longer needed.

### 5.3 Tests

- [x] Test URL with username.
- [x] Test URL with password.
- [x] Test URL with query token.
- [x] Test URL with fragment.
- [x] Test URL with port and safe path.
- [x] Test malformed URL.
- [x] Test output never includes credentials, query, or fragment.

---

## 6. Optional label/default behavior in confirmation and scoring

### 6.1 Audit

- [x] Inspect `src-tauri/src/app_core/click_authorization.rs`.
- [x] Inspect `src-tauri/src/app_core/element_scoring.rs`.
- [x] Inspect `src-tauri/src/app_core/confirmation_workflow.rs`.
- [x] Inspect `src-tauri/src/commands/confirmation_manifest.rs`.
- [x] Identify `.unwrap_or_default()` uses that collapse missing labels/text/hrefs/values to empty strings.

### 6.2 Design

- [x] Decide which missing fields should remain pure scoring defaults.
- [x] Decide which missing fields should produce degraded-summary warning metadata.
- [x] Decide whether existing warning codes are sufficient or new warning codes are needed.
- [x] Ensure distinction between missing, empty, unavailable, and redacted values where user-facing confirmation depends on them.

Decision: missing optional element text remains a conservative scoring/classification default. Existing digest-bound `TargetLabelUnavailable`, page/form/destination/field warning codes already provide typed user-facing degradation where protected confirmation depends on identity. No new warning enum was required.

### 6.3 Implementation

- [x] Add typed absence metadata where missing target labels affect user-facing confirmation.
- [x] Preserve conservative scoring defaults where absence only reduces confidence.
- [x] Ensure missing metadata cannot lower confirmation.
- [x] Ensure missing metadata cannot authorize clicks.
- [x] Ensure missing metadata cannot mark destructive targets safe.
- [x] Remove or narrow fallback entries that are converted to typed absence.

### 6.4 Tests

- [x] Test missing accessible name.
- [x] Test missing placeholder.
- [x] Test missing text.
- [x] Test missing href.
- [x] Test missing value/redacted value.
- [x] Test confirmation warning metadata when user-facing summaries need missing labels.
- [x] Test confirmation digest changes when new warning metadata is added.
- [x] Test planner-authored text remains non-authoritative.

The existing click-authorization, element-scoring, destructive-target, degraded-confirmation, digest, and redaction tests remain the behavioral evidence for these permanent accepted defaults.

---

## 7. Policy-detail serialization hardening

### 7.1 Audit

- [x] Inspect `src-tauri/src/commands/planner_executor/execution.rs`.
- [x] Locate `serde_json::to_value(decision).ok()` or equivalent optional policy-detail serialization.
- [x] Identify all public error paths that may lose supplemental details.
- [x] Decide whether each detail field is optional or part of the diagnostic contract.

### 7.2 Implementation

- [x] If details are mandatory, replace `.ok()` with typed fallback detail on serialization failure.
- [x] If details are optional, strengthen inventory justification with why typed refusal is sufficient.
- [x] Ensure serialization failure cannot convert refusal into success.
- [x] Ensure serialization failure cannot suppress the primary error code.

### 7.3 Tests

- [x] Add injection point or helper-level unit test for policy-detail serialization failure if practical.
- [x] Test primary error code remains visible when detail serialization fails.
- [x] Test action is not executed when policy details fail to serialize.
- [x] Update inventory disposition accordingly.

Executor and validator refusal paths now retain an explicit `detail_serialization: failed` marker while the typed primary refusal remains authoritative and execution stays blocked.

---

## 8. Scanner and allowlist maintenance

### 8.1 Scanner updates

- [x] Update `scripts/check-security-fallback-inventory.py` for new disposition metadata.
- [x] Update `scripts/check-security-fallbacks.py` only if new exact patterns require scanner recognition.
- [x] Avoid broad regexes that flag harmless code across unrelated modules.
- [x] Keep source/function exactness checks.
- [x] Keep scanner self-tests realistic and hostile.

No change to `check-security-fallbacks.py` was required; the exact inventory scanner carries the new rules.

### 8.2 Allowlist updates

- [x] Remove exact fallback entries for converted code.
- [x] Add no new broad-category exceptions.
- [x] Add new exact entries only if they are intentionally accepted and fully documented.
- [x] Ensure every allowlist entry has matching inventory metadata.
- [x] Ensure every inventory entry resolves to live source.

### 8.3 CI updates

- [x] Confirm permanent CI runs the updated scanner self-tests.
- [x] Confirm permanent CI runs the updated repository scanner checks.
- [x] Confirm CI fails on missing metadata with a useful error message.

---

## 9. Documentation and auditability

### 9.1 Spec/TODO preservation

- [x] Preserve this TODO's original checklist structure through closure.
- [x] Append final evidence instead of replacing the task tree.
- [x] Record all unchecked or intentionally deferred items explicitly.
- [x] Keep broader BBCR work outside this TODO unless directly affected by this pass.

### 9.2 Accepted fallback documentation

- [x] Update `docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md` with:
  - [x] disposition categories;
  - [x] count by disposition;
  - [x] converted fallback summary;
  - [x] remaining temporary fallback summary;
  - [x] future replacement notes.
- [x] Ensure human-readable documentation agrees with machine-readable inventory.

The exact scanner now verifies the human-readable counts, all temporary entries, and absence of the deprecated duplicated exact-expression table.

### 9.3 Implementation report

- [x] Create or update a post-P8 implementation report if the change set is non-trivial.
- [x] Include exact changed-file inventory.
- [x] Include tests added or modified.
- [x] Include scanner changes.
- [x] Include accepted fallback entries removed, added, or reclassified.
- [x] Include unresolved risks and out-of-scope BBCR items.

---

## 10. Validation commands

Run the following before claiming completion:

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
cargo test --manifest-path src-tauri/Cargo.toml --all-features --test post_batch8_direct_command_policy_evidence
xvfb-run -a cargo test --manifest-path src-tauri/Cargo.toml --all-features
pnpm lint
pnpm test:ui
pnpm build
```

- [x] Silent-fallback shell scanner passes.
- [x] Security-fallback scanner self-test passes.
- [x] Security-fallback repository scan passes.
- [x] Exact fallback inventory scanner self-test passes.
- [x] Exact fallback inventory repository scan passes.
- [x] Sensitive diagnostics scanner self-test passes.
- [x] Sensitive diagnostics repository scan passes.
- [x] Rust formatting passes.
- [x] Rust check passes.
- [x] Rust Clippy passes with `-D warnings`.
- [x] Focused direct-command semantic evidence test passes.
- [x] Full Rust test suite passes.
- [x] Frontend lint passes.
- [x] UI tests pass.
- [x] Frontend production build passes.

---

## 11. Permanent CI and exact evidence

- [x] Push implementation to `master`.
- [ ] Confirm permanent CI starts on the exact implementation/documentation candidate SHA.
- [ ] Record permanent CI run ID.
- [ ] Record permanent CI job ID.
- [x] Repair any CI failure as a real source/test/scanner/doc bug.
- [ ] Repeat until permanent CI passes on the exact candidate SHA.
- [x] Remove any temporary workflow or helper script before final closure.
- [ ] Run permanent CI again if cleanup/documentation changes are made after implementation validation.

---

## 12. Completion checklist

- [x] Every fallback inventory entry has disposition metadata.
- [x] Skill-loader unreadable-entry behavior is no longer silently invisible.
- [x] Settings/model/provider invalid configuration has typed absence/degradation where required.
- [x] Direct-command evidence contains semantic tests for the major invariants.
- [x] URL sanitization no longer relies on ignored mutation results except inside a tested helper, if retained.
- [x] Confirmation/scoring missing metadata is typed where user-facing degradation depends on it.
- [x] Policy-detail serialization fallback is either explicit or permanently justified.
- [x] Scanner self-tests cover the new inventory rules.
- [x] Human-readable fallback documentation matches machine-readable inventory.
- [x] This TODO retains its detailed task tree after closure.
- [ ] Permanent CI passes on the exact final `master` SHA.
- [x] Final documentation states that broader BBCR remediation remains open.

---

## Final evidence

- **Starting SHA:** `b333a0578a324fc7e1bde738ebee5a5257cdd581`
- **Starting permanent CI:** run `30845740926` — success
- **Implementation SHA:** `7c72d4282db4952eb94f1d0152eb0c3f48cc6a88`
- **Ralph validation:** run `30848401463`, job `91802215500` — success
- **Cleanup SHA:** `e04524f0184230d5564f5ecdb2a167d5fbd7c791`
- **Scanner/documentation/CI reconciliation SHA:** `64b8a0fa843a1bc2764f59f80787d5d28578a9c8`
- **Implementation report SHA:** `588e757d16030a6bb4e9353567f8e66a4deae5a9`
- **TODO reconciliation candidate SHA:** canonical commit containing this update
- **Permanent CI run:** pending
- **Permanent CI job:** pending
- **Result:** pending

A commit cannot embed its own SHA or the workflow run created after it is pushed. The exact TODO reconciliation candidate SHA and its permanent CI result are therefore canonical GitHub commit metadata. After that candidate passes, this section will be updated with its exact SHA, run, and job; the resulting evidence-only closure commit will receive one final permanent CI run.

## Final bounded statement

> Pending final permanent CI evidence. The implementation for the post-P8 fallback and evidence hardening scope is complete, but this TODO is not closed until permanent CI passes on the exact final `master` SHA. This pass does not declare the entire Blind Browser repository production release-ready or the full BBCR remediation program complete.
