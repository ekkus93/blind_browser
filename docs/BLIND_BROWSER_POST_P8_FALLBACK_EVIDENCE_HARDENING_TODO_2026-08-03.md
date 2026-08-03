# Blind Browser Post-P8 Fallback and Evidence Hardening TODO

**Date:** 2026-08-03  
**Repository:** `ekkus93/blind_browser`  
**Branch:** `master`  
**Companion spec:** `docs/BLIND_BROWSER_POST_P8_FALLBACK_EVIDENCE_HARDENING_SPEC_2026-08-03.md`  
**Depends on:** `docs/BLIND_BROWSER_POST_BATCH8_SECURITY_HARDENING_TODO_2026-08-02.md`  
**Status:** Not started  
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

- [ ] Confirm latest `master` SHA before starting implementation.
- [ ] Confirm permanent CI state for the starting SHA.
- [ ] Read `docs/BLIND_BROWSER_POST_P8_FALLBACK_EVIDENCE_HARDENING_SPEC_2026-08-03.md`.
- [ ] Read `docs/BLIND_BROWSER_POST_BATCH8_SECURITY_HARDENING_TODO_2026-08-02.md`.
- [ ] Read `docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md`.
- [ ] Read `scripts/security-fallback-allowlist.txt`.
- [ ] Read `scripts/security-fallback-inventory.json`.
- [ ] Read `scripts/check-security-fallbacks.py`.
- [ ] Read `scripts/check-security-fallback-inventory.py`.
- [ ] Confirm no temporary workflow/script from the prior Ralph run remains in the production tree.
- [ ] Confirm this pass will work directly from `master` unless the user gives different instructions.

---

## 1. Accepted fallback inventory triage

### 1.1 Inventory review

- [ ] Enumerate every entry in `scripts/security-fallback-inventory.json`.
- [ ] Group entries by source category:
  - [ ] click authorization / element scoring;
  - [ ] command dispatch / fill correction / field focus;
  - [ ] confirmation workflow;
  - [ ] planner redaction / URL sanitization;
  - [ ] settings/model/provider capability discovery;
  - [ ] numeric conversion;
  - [ ] policy-detail serialization;
  - [ ] skill loader / skill parser;
  - [ ] frontend redaction / UI helper fallbacks, if present.
- [ ] For every entry, determine whether the current fallback is:
  - [ ] permanently justified;
  - [ ] temporarily justified;
  - [ ] convertible to typed warning;
  - [ ] convertible to typed error;
  - [ ] removable.

### 1.2 Inventory metadata

- [ ] Add a `disposition` or equivalent field to every machine-readable inventory entry.
- [ ] Add a `review_due` or equivalent field for temporary accepted fallbacks.
- [ ] Add a short `owner_note` or equivalent field for every temporary accepted fallback.
- [ ] Update `scripts/check-security-fallback-inventory.py` to require the new metadata.
- [ ] Add scanner self-test coverage for:
  - [ ] missing disposition;
  - [ ] invalid disposition;
  - [ ] temporary fallback missing review boundary;
  - [ ] stale source expression;
  - [ ] stale containing function.
- [ ] Update `docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md` with disposition counts and policy.

### 1.3 Acceptance gate

- [ ] Run `python3 scripts/check-security-fallback-inventory.py --self-test`.
- [ ] Run `python3 scripts/check-security-fallback-inventory.py`.
- [ ] Confirm permanent CI includes the enhanced inventory check.

---

## 2. Skill loading quiet-skip hardening

### 2.1 Audit

- [ ] Inspect `src-tauri/src/commands/skill_loader.rs`.
- [ ] Locate every `filter_map(Result::ok)` or equivalent skip in skill loading.
- [ ] Identify what errors can be skipped:
  - [ ] unreadable directory entry;
  - [ ] invalid file type;
  - [ ] unreadable skill manifest;
  - [ ] parse failure;
  - [ ] permission denied;
  - [ ] symlink/path policy rejection.
- [ ] Determine which errors are already surfaced elsewhere and which are currently quiet.

### 2.2 Implementation

- [ ] Replace direct quiet skipping of unreadable directory entries with explicit warning aggregation.
- [ ] Preserve ability to load other valid skills when one optional entry is unreadable.
- [ ] Record bounded warning metadata:
  - [ ] count of skipped entries;
  - [ ] error kind or category;
  - [ ] source class;
  - [ ] safe leaf identifier only if allowed by path-privacy rules.
- [ ] Ensure full absolute paths are not exposed.
- [ ] Ensure skipped entries cannot add skills, tools, or permissions.
- [ ] Ensure warnings are visible through an existing diagnostic/status path or a new bounded return field.

### 2.3 Tests

- [ ] Test unreadable entry is omitted but produces a bounded warning.
- [ ] Test multiple unreadable entries aggregate without leaking full paths.
- [ ] Test valid adjacent skills still load.
- [ ] Test skipped entries cannot grant tools or permissions.
- [ ] Test diagnostics remain path-private.
- [ ] Remove or reclassify the old skill-loader fallback entry from the allowlist/inventory.

---

## 3. Settings/model/provider typed absence hardening

### 3.1 Audit

- [ ] Inspect `src-tauri/src/app_core/settings_adapters.rs`.
- [ ] Identify fallbacks that collapse invalid settings into generic unavailable capability.
- [ ] Audit remote planner base URL parsing fallbacks.
- [ ] Audit Kitten TTS model-plan lookup fallbacks.
- [ ] Audit Whisper ASR model-plan lookup fallbacks.
- [ ] Audit voice/settings defaults that use `.unwrap_or_default()`.

### 3.2 Typed absence design

- [ ] Define typed absence/degradation reasons, such as:
  - [ ] `not_configured`;
  - [ ] `invalid_endpoint`;
  - [ ] `unknown_model_id`;
  - [ ] `manifest_unavailable`;
  - [ ] `feature_disabled`;
  - [ ] `credential_reference_missing`;
  - [ ] `local_binary_unavailable`.
- [ ] Decide the Rust type location for the absence reasons.
- [ ] Decide the frontend shape if settings UI receives these reasons.
- [ ] Ensure reason fields do not include raw credentials, query strings, fragments, full local paths, or raw provider responses.

### 3.3 Implementation

- [ ] Surface invalid remote planner endpoint distinctly from not configured.
- [ ] Surface unknown Kitten TTS model ID distinctly from not configured.
- [ ] Surface unknown Whisper ASR model ID distinctly from not configured.
- [ ] Preserve explicit feature-disabled behavior.
- [ ] Preserve fail-closed behavior for actual operations.
- [ ] Remove or narrow fallback inventory entries that are converted to typed absence.

### 3.4 Tests

- [ ] Test invalid endpoint reason.
- [ ] Test unknown TTS model reason.
- [ ] Test unknown ASR model reason.
- [ ] Test not-configured reason.
- [ ] Test feature-disabled reason if applicable.
- [ ] Test sanitized output for endpoint strings containing credentials/query/fragment.
- [ ] Update frontend tests if settings output shape changes.

---

## 4. Direct-command semantic evidence hardening

### 4.1 Audit current evidence test

- [ ] Inspect `src-tauri/tests/post_batch8_direct_command_policy_evidence.rs`.
- [ ] Identify assertions that depend primarily on source-code string search.
- [ ] Identify which invariants already have semantic coverage elsewhere.
- [ ] Identify which invariants have only text-search coverage.

### 4.2 Semantic registry/API mapping

- [ ] Ensure every command in `tauri::generate_handler!` maps to `DirectCommandPolicy` or equivalent metadata.
- [ ] Ensure every direct command policy can be inspected by tests without parsing source text.
- [ ] Add or expose a typed mapping for network client/policy classes where practical.
- [ ] Add or expose a typed mapping for credential-scope requirement where practical.
- [ ] Add or expose a typed mapping for verified model-download activation where practical.
- [ ] Add or expose a typed mapping for page-context sanitizer requirement where practical.

### 4.3 Tests

- [ ] Add semantic test: every handler has direct-command policy metadata.
- [ ] Add semantic test: every networked command has a network policy/client mapping.
- [ ] Add semantic test: every credential-bearing command has endpoint-bound credential mapping where applicable.
- [ ] Add semantic test: every model-download command maps to verified activation.
- [ ] Add semantic test: every page-context-transmitting command maps to privacy sanitization.
- [ ] Retain source-text checks only as supplemental drift detectors.
- [ ] Rename or split tests so semantic checks and source-text drift checks are clearly distinguished.

### 4.4 Acceptance gate

- [ ] Run the direct-command evidence test by itself.
- [ ] Run the full Rust test suite.
- [ ] Confirm a new networked command without metadata would fail tests.

---

## 5. URL sanitization ignored-result hardening

### 5.1 Audit

- [ ] Inspect `src-tauri/src/app_core/planner_redaction.rs`.
- [ ] Locate `let _ = parsed.set_username("")`.
- [ ] Locate `let _ = parsed.set_password(None)`.
- [ ] Search for other ignored URL mutation results in Rust and frontend code.

### 5.2 Implementation

- [ ] Add a helper that reconstructs sanitized URLs from approved components, or handles URL mutation failure explicitly.
- [ ] Ensure the helper strips:
  - [ ] username;
  - [ ] password;
  - [ ] query;
  - [ ] fragment;
  - [ ] known token-like material.
- [ ] Ensure malformed input returns a generic redacted URL or typed invalid result.
- [ ] Remove broad ignored-result fallback entries if no longer needed.

### 5.3 Tests

- [ ] Test URL with username.
- [ ] Test URL with password.
- [ ] Test URL with query token.
- [ ] Test URL with fragment.
- [ ] Test URL with port and safe path.
- [ ] Test malformed URL.
- [ ] Test output never includes credentials, query, or fragment.

---

## 6. Optional label/default behavior in confirmation and scoring

### 6.1 Audit

- [ ] Inspect `src-tauri/src/app_core/click_authorization.rs`.
- [ ] Inspect `src-tauri/src/app_core/element_scoring.rs`.
- [ ] Inspect `src-tauri/src/app_core/confirmation_workflow.rs`.
- [ ] Inspect `src-tauri/src/commands/confirmation_manifest.rs`.
- [ ] Identify `.unwrap_or_default()` uses that collapse missing labels/text/hrefs/values to empty strings.

### 6.2 Design

- [ ] Decide which missing fields should remain pure scoring defaults.
- [ ] Decide which missing fields should produce degraded-summary warning metadata.
- [ ] Decide whether existing warning codes are sufficient or new warning codes are needed.
- [ ] Ensure distinction between missing, empty, unavailable, and redacted values where user-facing confirmation depends on them.

### 6.3 Implementation

- [ ] Add typed absence metadata where missing target labels affect user-facing confirmation.
- [ ] Preserve conservative scoring defaults where absence only reduces confidence.
- [ ] Ensure missing metadata cannot lower confirmation.
- [ ] Ensure missing metadata cannot authorize clicks.
- [ ] Ensure missing metadata cannot mark destructive targets safe.
- [ ] Remove or narrow fallback entries that are converted to typed absence.

### 6.4 Tests

- [ ] Test missing accessible name.
- [ ] Test missing placeholder.
- [ ] Test missing text.
- [ ] Test missing href.
- [ ] Test missing value/redacted value.
- [ ] Test confirmation warning metadata when user-facing summaries need missing labels.
- [ ] Test confirmation digest changes when new warning metadata is added.
- [ ] Test planner-authored text remains non-authoritative.

---

## 7. Policy-detail serialization hardening

### 7.1 Audit

- [ ] Inspect `src-tauri/src/commands/planner_executor/execution.rs`.
- [ ] Locate `serde_json::to_value(decision).ok()` or equivalent optional policy-detail serialization.
- [ ] Identify all public error paths that may lose supplemental details.
- [ ] Decide whether each detail field is optional or part of the diagnostic contract.

### 7.2 Implementation

- [ ] If details are mandatory, replace `.ok()` with typed fallback detail on serialization failure.
- [ ] If details are optional, strengthen inventory justification with why typed refusal is sufficient.
- [ ] Ensure serialization failure cannot convert refusal into success.
- [ ] Ensure serialization failure cannot suppress the primary error code.

### 7.3 Tests

- [ ] Add injection point or helper-level unit test for policy-detail serialization failure if practical.
- [ ] Test primary error code remains visible when detail serialization fails.
- [ ] Test action is not executed when policy details fail to serialize.
- [ ] Update inventory disposition accordingly.

---

## 8. Scanner and allowlist maintenance

### 8.1 Scanner updates

- [ ] Update `scripts/check-security-fallback-inventory.py` for new disposition metadata.
- [ ] Update `scripts/check-security-fallbacks.py` only if new exact patterns require scanner recognition.
- [ ] Avoid broad regexes that flag harmless code across unrelated modules.
- [ ] Keep source/function exactness checks.
- [ ] Keep scanner self-tests realistic and hostile.

### 8.2 Allowlist updates

- [ ] Remove exact fallback entries for converted code.
- [ ] Add no new broad-category exceptions.
- [ ] Add new exact entries only if they are intentionally accepted and fully documented.
- [ ] Ensure every allowlist entry has matching inventory metadata.
- [ ] Ensure every inventory entry resolves to live source.

### 8.3 CI updates

- [ ] Confirm permanent CI runs the updated scanner self-tests.
- [ ] Confirm permanent CI runs the updated repository scanner checks.
- [ ] Confirm CI fails on missing metadata with a useful error message.

---

## 9. Documentation and auditability

### 9.1 Spec/TODO preservation

- [ ] Preserve this TODO's original checklist structure through closure.
- [ ] Append final evidence instead of replacing the task tree.
- [ ] Record all unchecked or intentionally deferred items explicitly.
- [ ] Keep broader BBCR work outside this TODO unless directly affected by this pass.

### 9.2 Accepted fallback documentation

- [ ] Update `docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md` with:
  - [ ] disposition categories;
  - [ ] count by disposition;
  - [ ] converted fallback summary;
  - [ ] remaining temporary fallback summary;
  - [ ] future replacement notes.
- [ ] Ensure human-readable documentation agrees with machine-readable inventory.

### 9.3 Implementation report

- [ ] Create or update a post-P8 implementation report if the change set is non-trivial.
- [ ] Include exact changed-file inventory.
- [ ] Include tests added or modified.
- [ ] Include scanner changes.
- [ ] Include accepted fallback entries removed, added, or reclassified.
- [ ] Include unresolved risks and out-of-scope BBCR items.

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
xvfb-run -a cargo test --manifest-path src-tauri/Cargo.toml --all-features
pnpm lint
pnpm test:ui
pnpm build
```

- [ ] Silent-fallback shell scanner passes.
- [ ] Security-fallback scanner self-test passes.
- [ ] Security-fallback repository scan passes.
- [ ] Exact fallback inventory scanner self-test passes.
- [ ] Exact fallback inventory repository scan passes.
- [ ] Sensitive diagnostics scanner self-test passes.
- [ ] Sensitive diagnostics repository scan passes.
- [ ] Rust formatting passes.
- [ ] Rust check passes.
- [ ] Rust Clippy passes with `-D warnings`.
- [ ] Full Rust test suite passes.
- [ ] Frontend lint passes.
- [ ] UI tests pass.
- [ ] Frontend production build passes.

---

## 11. Permanent CI and exact evidence

- [ ] Push implementation to `master`.
- [ ] Confirm permanent CI starts on the exact implementation SHA.
- [ ] Record permanent CI run ID.
- [ ] Record permanent CI job ID.
- [ ] Repair any CI failure as a real source/test/scanner/doc bug.
- [ ] Repeat until permanent CI passes on the exact candidate SHA.
- [ ] Remove any temporary workflow or helper script before final closure.
- [ ] Run permanent CI again if cleanup/documentation changes are made after implementation validation.

---

## 12. Completion checklist

- [ ] Every fallback inventory entry has disposition metadata.
- [ ] Skill-loader unreadable-entry behavior is no longer silently invisible.
- [ ] Settings/model/provider invalid configuration has typed absence/degradation where required.
- [ ] Direct-command evidence contains semantic tests for the major invariants.
- [ ] URL sanitization no longer relies on ignored mutation results except inside a tested helper, if retained.
- [ ] Confirmation/scoring missing metadata is typed where user-facing degradation depends on it.
- [ ] Policy-detail serialization fallback is either explicit or permanently justified.
- [ ] Scanner self-tests cover the new inventory rules.
- [ ] Human-readable fallback documentation matches machine-readable inventory.
- [ ] This TODO retains its detailed task tree after closure.
- [ ] Permanent CI passes on the exact final `master` SHA.
- [ ] Final documentation states that broader BBCR remediation remains open.

---

## Final evidence

Append final evidence here at closure. Do not replace the checklist above.

- **Starting SHA:** TBD
- **Implementation SHA:** TBD
- **Cleanup/documentation SHA:** TBD
- **Final SHA:** TBD
- **Permanent CI run:** TBD
- **Permanent CI job:** TBD
- **Result:** TBD

## Final bounded statement

> Pending. When complete, this pass may state that the post-P8 fallback and evidence hardening scope is complete. It must not state that the entire Blind Browser repository is production release-ready or that the full BBCR remediation program is complete.
