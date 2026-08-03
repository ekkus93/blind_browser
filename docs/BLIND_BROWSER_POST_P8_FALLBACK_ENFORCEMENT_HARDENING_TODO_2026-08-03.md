# Blind Browser Post-P8 Fallback Enforcement Hardening TODO

**Date:** 2026-08-03  
**Repository:** `ekkus93/blind_browser`  
**Branch:** `master`  
**Companion spec:** `docs/BLIND_BROWSER_POST_P8_FALLBACK_ENFORCEMENT_HARDENING_SPEC_2026-08-03.md`  
**Depends on:** `docs/BLIND_BROWSER_POST_P8_FALLBACK_EVIDENCE_HARDENING_TODO_2026-08-03.md`  
**Status:** Not started.  
**Release boundary:** This TODO is a bounded fallback-enforcement hardening pass. It does not complete the larger BBCR remediation program and must not be used to declare production release readiness.

---

## Completion rules

- Work directly on `master` unless the user explicitly requests a branch, PR, or worktree.
- Do not check a task unless implementation, test, scanner, or documentation evidence exists on `master`.
- Preserve this detailed checklist through closure. Append final evidence instead of replacing the task tree with a summary.
- Do not weaken any post-Batch-8 or post-P8 safety guard.
- Treat first-party CI/test/scanner failures as real defects unless proven otherwise.
- Remove any temporary workflow, script, patch generator, or diagnostic helper before final closure.
- Record exact implementation SHA, cleanup SHA if any, final documentation SHA, permanent CI run ID, and job ID.
- Keep broader BBCR release work out of scope unless this TODO directly touches it.

---

## 0. Baseline and review setup

- [ ] Confirm latest `master` SHA before implementation.
- [ ] Confirm permanent CI state for the starting SHA.
- [ ] Read `docs/BLIND_BROWSER_POST_P8_FALLBACK_ENFORCEMENT_HARDENING_SPEC_2026-08-03.md`.
- [ ] Read `docs/BLIND_BROWSER_POST_P8_FALLBACK_EVIDENCE_HARDENING_TODO_2026-08-03.md`.
- [ ] Read `docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md`.
- [ ] Read `scripts/security-fallback-allowlist.txt`.
- [ ] Read `scripts/security-fallback-inventory.json`.
- [ ] Read `scripts/check-security-fallbacks.py`.
- [ ] Read `scripts/check-security-fallback-inventory.py`.
- [ ] Confirm no temporary Ralph/workflow/script files are present at baseline.
- [ ] Confirm expected changed-file scope before coding.

---

## 1. Fallback inventory exact occurrence enforcement

### 1.1 Audit current scanner weakness

- [ ] Inspect `scripts/check-security-fallback-inventory.py`.
- [ ] Confirm current keying behavior for inventory entries.
- [ ] Identify every inventory entry with broad expressions such as:
  - [ ] `.ok()`;
  - [ ] `.unwrap_or_default()`;
  - [ ] `unwrap_or(...)`;
  - [ ] `map_or(...)` if authority-relevant;
  - [ ] `let _ = ...` if any remain;
  - [ ] other small expressions likely to match multiple sites.
- [ ] Count all fallback occurrences per `(path, function, expression)`.
- [ ] Identify all duplicate or ambiguous occurrences that cannot be uniquely reviewed today.

### 1.2 Inventory schema design

- [ ] Choose the exact occurrence identity model:
  - [ ] occurrence index within containing function; or
  - [ ] before/after normalized line context; or
  - [ ] source-span/context hash; or
  - [ ] explicit `fallback_id` plus exact occurrence count.
- [ ] Update inventory schema version.
- [ ] Add occurrence identity to every inventory entry.
- [ ] Add a migration note in accepted fallback documentation.
- [ ] Ensure temporary and permanent entries use the same exactness rules.

### 1.3 Scanner implementation

- [ ] Update `scripts/check-security-fallback-inventory.py` to locate every matching source occurrence.
- [ ] Fail when source contains an uninventoried matching occurrence.
- [ ] Fail when inventory references a missing occurrence.
- [ ] Fail when two inventory entries point to the same occurrence.
- [ ] Fail when containing function differs from inventory.
- [ ] Fail when occurrence context differs from inventory.
- [ ] Keep existing disposition/review-boundary/owner-note checks.
- [ ] Keep documentation count and temporary-summary checks.

### 1.4 Scanner self-tests

- [ ] Add self-test for duplicate broad `.ok()` expression with only one inventory record.
- [ ] Add self-test for duplicated `.unwrap_or_default()` expression with only one inventory record.
- [ ] Add self-test for stale context.
- [ ] Add self-test for duplicate inventory records pointing at one occurrence.
- [ ] Add self-test for missing temporary review boundary.
- [ ] Add self-test for missing owner note.
- [ ] Confirm scanner error messages name the path, function, expression, and occurrence identity.

### 1.5 Acceptance gate

- [ ] Run `python3 scripts/check-security-fallback-inventory.py --self-test`.
- [ ] Run `python3 scripts/check-security-fallback-inventory.py`.
- [ ] Confirm permanent CI runs the enhanced scanner.

---

## 2. Direct focus query fallback conversion

### 2.1 Audit

- [ ] Inspect `src-tauri/src/app_core/form_fill/field_focus.rs`.
- [ ] Locate `build_find_element_query(&query).ok()?`.
- [ ] Determine all failure modes of `build_find_element_query`.
- [ ] Confirm whether failure currently falls through to remote planning.
- [ ] Confirm no side effect occurs before query construction succeeds.

### 2.2 Implementation

- [ ] Replace `build_find_element_query(&query).ok()?` with explicit handling.
- [ ] Return deterministic `ReportResult` follow-up when query construction fails.
- [ ] Add a bounded reason code, such as `focus_query_construction_failed`.
- [ ] Do not expose raw page text, OCR text, private selectors, or full DOM details.
- [ ] Do not authorize focus, typing, form submission, or click on query construction failure.
- [ ] Ensure the direct resolver does not silently disappear for this failure.
- [ ] Remove or update the corresponding temporary fallback inventory entry.

### 2.3 Tests

- [ ] Add unit test for query-construction failure returning `ReportResult` follow-up.
- [ ] Test follow-up has `ReportStatus::NeedsFollowUp`.
- [ ] Test follow-up contains a bounded user message.
- [ ] Test no `FocusElement`, `TypeIntoElement`, or `SubmitActiveForm` step is emitted.
- [ ] Test the resolver does not return `None` for this failure mode.
- [ ] Run focused form-fill/field-focus tests.

---

## 3. Remaining temporary command/fill fallback conversion

### 3.1 Command dispatch project context diagnostics

- [ ] Inspect `src-tauri/src/app_core/command_dispatch.rs`.
- [ ] Locate `let current_dir = std::env::current_dir().ok();`.
- [ ] Locate `app_config_dir().ok()` or equivalent user-skill-root fallback.
- [ ] Design typed diagnostics for missing project context and user skill root.
- [ ] Ensure diagnostics are path-private.
- [ ] Ensure diagnostics reduce skill discovery only and cannot grant tools.
- [ ] Decide where diagnostics are stored or returned.
- [ ] Implement typed diagnostics.
- [ ] Remove or update related temporary fallback inventory entries.

### 3.2 Candidate discovery `.ok()` fallbacks

- [ ] Identify all temporary `.ok()` entries in command discovery.
- [ ] Convert each practical site to typed candidate-rejection reason.
- [ ] Ensure candidate rejection cannot execute a protected action.
- [ ] Ensure candidate rejection cannot report success.
- [ ] Add tests for candidate rejection diagnostics.
- [ ] Keep any non-converted fallback explicitly temporary with exact occurrence identity.

### 3.3 Fill correction stale-target diagnostics

- [ ] Inspect `src-tauri/src/app_core/fill_correction.rs`.
- [ ] Locate `resolve_typeable_element(...).ok()` in recent fill correction.
- [ ] Replace silent candidate omission with typed stale-candidate/follow-up reason if practical.
- [ ] Preserve deterministic follow-up behavior when recent context is missing or stale.
- [ ] Ensure stale candidates cannot type into a wrong element.
- [ ] Add tests for stale active target and stale alternate target.
- [ ] Remove or update related temporary fallback inventory entries.

### 3.4 Skill parser optional-frontmatter fallback

- [ ] Inspect `src-tauri/src/commands/skill_parser.rs`.
- [ ] Locate optional frontmatter `.unwrap_or_default()` fallback.
- [ ] Decide whether missing optional frontmatter should remain permanent or become typed parser diagnostic.
- [ ] If converted, add bounded per-skill parser diagnostic without full path leakage.
- [ ] If retained, enforce exact occurrence identity and temporary/permanent disposition explicitly.
- [ ] Add parser tests for missing optional frontmatter diagnostics.

---

## 4. First-class skill discovery diagnostics

### 4.1 Audit

- [ ] Inspect `src-tauri/src/commands/skill_loader.rs`.
- [ ] Identify all current `tracing::warn!` skill discovery diagnostics.
- [ ] Identify current public/runtime surfaces that could expose skill diagnostics.
- [ ] Confirm path-private data requirements.

### 4.2 Type design

- [ ] Add or select a type such as `SkillDiscoveryDiagnostics`.
- [ ] Add or select warning entries such as `SkillLoadWarning`.
- [ ] Include source class: project, user, bundled.
- [ ] Include skipped entry counts.
- [ ] Include bounded error categories.
- [ ] Include safe leaf identifiers only where allowed.
- [ ] Include parse/read/name-mismatch categories where already known.
- [ ] Exclude full paths, project roots, home directories, usernames, raw file contents, and raw manifest text.

### 4.3 Implementation

- [ ] Return or store typed skill discovery diagnostics.
- [ ] Surface diagnostics through runtime status, agent state, settings status, or equivalent typed path.
- [ ] Preserve existing skill loading behavior for valid skills.
- [ ] Preserve omission of invalid optional skills.
- [ ] Ensure skipped skills cannot add tools or permissions.
- [ ] Preserve tracing warnings as supplemental diagnostics if useful.

### 4.4 Tests

- [ ] Test unreadable directory entries produce typed diagnostics.
- [ ] Test unreadable `SKILL.md` produces typed diagnostics.
- [ ] Test invalid frontmatter produces typed diagnostics.
- [ ] Test directory/frontmatter name mismatch produces typed diagnostics.
- [ ] Test valid adjacent skill still loads.
- [ ] Test diagnostic output does not contain full absolute paths.
- [ ] Test skipped invalid skill grants no tools.
- [ ] Update TypeScript types/tests if runtime status changes.

---

## 5. Remote TTS/ASR typed absence parity

### 5.1 Audit

- [ ] Inspect `src-tauri/src/app_core/settings_adapters.rs`.
- [ ] Inspect `src-tauri/src/commands/contracts/providers.rs`.
- [ ] Inspect `src/tauri-types.ts`.
- [ ] Compare `RemotePlannerSettings` with `RemoteTtsSettings` and `RemoteAsrSettings`.
- [ ] Identify raw `base_url` clones in TTS/ASR settings.
- [ ] Identify missing typed absence fields.

### 5.2 Contract design

- [ ] Add `availability_reason` or equivalent to `RemoteTtsSettings`.
- [ ] Add `availability_reason` or equivalent to `RemoteAsrSettings`.
- [ ] Reuse `CapabilityAbsenceReason` where possible.
- [ ] Add sanitized endpoint display for remote TTS.
- [ ] Add sanitized endpoint display for remote ASR.
- [ ] Decide how to represent invalid endpoint vs profile missing vs not configured.
- [ ] Decide how to represent credential-reference missing or unreadable when visible from settings.
- [ ] Decide how to represent feature-disabled states.

### 5.3 Implementation

- [ ] Surface remote TTS not-configured reason.
- [ ] Surface remote TTS profile-missing reason.
- [ ] Surface remote TTS invalid-endpoint reason.
- [ ] Surface remote ASR not-configured reason.
- [ ] Surface remote ASR profile-missing reason.
- [ ] Surface remote ASR invalid-endpoint reason.
- [ ] Sanitize TTS endpoint userinfo/query/fragment.
- [ ] Sanitize ASR endpoint userinfo/query/fragment.
- [ ] Ensure raw credentials and provider responses are never surfaced.
- [ ] Update frontend type definitions.

### 5.4 Tests

- [ ] Test remote TTS invalid endpoint reason.
- [ ] Test remote TTS profile missing reason.
- [ ] Test remote TTS not configured reason.
- [ ] Test remote TTS sanitized endpoint output.
- [ ] Test remote ASR invalid endpoint reason.
- [ ] Test remote ASR profile missing reason.
- [ ] Test remote ASR not configured reason.
- [ ] Test remote ASR sanitized endpoint output.
- [ ] Test endpoint containing username/password/query/fragment leaks none of those values.
- [ ] Update frontend tests if needed.

---

## 6. Embedded diagnostic URL redaction

### 6.1 Audit

- [ ] Inspect `src-tauri/src/diagnostic_redaction.rs`.
- [ ] Identify whole-string URL redaction behavior.
- [ ] Identify embedded URL cases that are not currently sanitized.
- [ ] Review sensitive marker list for query-secret names.

### 6.2 Implementation

- [ ] Add embedded URL detection for prose strings.
- [ ] Sanitize each URL-like substring using the approved origin/path reconstruction helper.
- [ ] Redact malformed embedded URL-like substrings conservatively.
- [ ] Remove username and password from embedded URLs.
- [ ] Remove query strings from embedded URLs.
- [ ] Remove fragments from embedded URLs.
- [ ] Add detection for OAuth/signed URL query-secret names:
  - [ ] `code`;
  - [ ] `state`;
  - [ ] `token`;
  - [ ] `access_token`;
  - [ ] `refresh_token`;
  - [ ] `id_token`;
  - [ ] `sig`;
  - [ ] `signature`;
  - [ ] `client_secret`;
  - [ ] `api_key`;
  - [ ] `key`;
  - [ ] `session`.
- [ ] Preserve non-URL prose that contains no secret marker.

### 6.3 Tests

- [ ] Test bare URL redaction.
- [ ] Test prose with one embedded URL.
- [ ] Test prose with multiple embedded URLs.
- [ ] Test embedded URL with username/password.
- [ ] Test OAuth callback URL with `code` and `state`.
- [ ] Test signed URL with `sig` or `signature`.
- [ ] Test malformed embedded URL-like substring.
- [ ] Test non-URL prose remains unchanged.
- [ ] Test JSON diagnostic redaction applies embedded URL redaction recursively.

---

## 7. Provider handler empty-default removal

### 7.1 Audit

- [ ] Inspect `src-tauri/src/command_handlers/provider_handlers.rs`.
- [ ] Locate response-side `settings.base_url.unwrap_or_default()`.
- [ ] Locate response-side `settings.model.unwrap_or_default()`.
- [ ] Search for similar provider handler empty defaults.
- [ ] Decide whether empty value is ever a valid post-persist state.

### 7.2 Implementation

- [ ] Replace post-persist empty defaults with typed internal consistency errors.
- [ ] Preserve explicit persistence failures as failures.
- [ ] Do not return successful empty endpoint/model unless explicitly valid.
- [ ] Sanitize returned endpoint strings.
- [ ] Include bounded error codes for inconsistent post-persist settings.
- [ ] Remove or inventory any remaining accepted defaults.

### 7.3 Tests

- [ ] Test set remote planner settings success returns non-empty sanitized base URL and model.
- [ ] Test reset remote planner settings success returns non-empty sanitized base URL and model.
- [ ] Test inconsistent post-persist settings returns typed error.
- [ ] Test no raw credentials/query/fragment in handler response.
- [ ] Test persistence failure remains a failure.

---

## 8. Direct-command behavioral evidence strengthening

### 8.1 Audit

- [ ] Inspect `src-tauri/src/direct_command_policy.rs`.
- [ ] Inspect `src-tauri/tests/post_batch8_direct_command_policy_evidence.rs`.
- [ ] Identify remaining source-string-only checks.
- [ ] Identify available typed helpers or runtime paths for behavioral tests.

### 8.2 Behavioral test coverage

- [ ] Add or strengthen test proving networked commands use timeout and redirect policy wrappers.
- [ ] Add or strengthen test proving credential-bearing commands use endpoint-bound secret resolution.
- [ ] Add or strengthen test proving page-context commands pass through remote planner privacy sanitization.
- [ ] Add or strengthen test proving model downloads use verified atomic activation.
- [ ] Add or strengthen test proving external launch commands require user gesture and validated URL policy.
- [ ] Keep source-drift tests only as supplemental checks.

### 8.3 Registry safeguards

- [ ] Ensure a new Tauri handler without a direct-command registry entry fails tests.
- [ ] Ensure a networked registry entry without network policy mapping fails tests.
- [ ] Ensure a credential-bearing entry without endpoint-bound mapping fails tests.
- [ ] Ensure a page-context entry without sanitizer mapping fails tests.
- [ ] Ensure a model-download entry without verified activation mapping fails tests.

### 8.4 CI gate

- [ ] Confirm permanent CI runs focused direct-command evidence.
- [ ] Confirm permanent CI runs the full Rust suite after focused evidence.

---

## 9. Allowlist and accepted fallback documentation

### 9.1 Allowlist updates

- [ ] Remove fallback expressions converted to typed diagnostics/errors.
- [ ] Add no broad-category exceptions.
- [ ] If a fallback remains, bind it to exact occurrence identity.
- [ ] Ensure every allowlist line has a matching inventory entry.
- [ ] Ensure every inventory entry resolves to live source.

### 9.2 Documentation updates

- [ ] Update `docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md`.
- [ ] Update disposition counts.
- [ ] Update temporary fallback summary.
- [ ] Update converted/removed fallback summary.
- [ ] Document the new exact occurrence identity rules.
- [ ] Ensure human-readable docs do not duplicate a stale per-expression table.
- [ ] Ensure documentation parity is enforced by scanner.

### 9.3 Implementation report

- [ ] Create `docs/BLIND_BROWSER_POST_P8_FALLBACK_ENFORCEMENT_HARDENING_IMPLEMENTATION_REPORT_2026-08-03.md`.
- [ ] Include exact changed-file inventory.
- [ ] Include fallbacks removed, converted, retained, and newly inventoried.
- [ ] Include tests added or modified.
- [ ] Include scanner changes.
- [ ] Include CI run/job evidence.
- [ ] Include unresolved risks and out-of-scope BBCR items.

---

## 10. Validation commands

Run before claiming completion:

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
- [ ] Focused direct-command evidence test passes.
- [ ] Full Rust test suite passes.
- [ ] Frontend lint passes.
- [ ] UI tests pass.
- [ ] Frontend production build passes.

---

## 11. Permanent CI and exact evidence

- [ ] Push implementation to `master`.
- [ ] Confirm permanent CI starts on the exact implementation candidate SHA.
- [ ] Record implementation SHA.
- [ ] Record permanent CI run ID.
- [ ] Record permanent CI job ID.
- [ ] Repair any CI failure as a real source/test/scanner/doc bug.
- [ ] Repeat until permanent CI passes on the exact candidate SHA.
- [ ] Remove any temporary workflow/helper script before final closure.
- [ ] Push documentation/report/TODO reconciliation to `master`.
- [ ] Run permanent CI again if cleanup or documentation changes occur after implementation validation.
- [ ] Record final evidence SHA.
- [ ] Record final permanent CI run ID.
- [ ] Record final permanent CI job ID.
- [ ] Record final result.

---

## 12. Completion checklist

- [ ] Broad fallback expressions cannot hide duplicate source occurrences.
- [ ] Every accepted fallback has unique occurrence identity.
- [ ] Direct focus query construction failure returns typed follow-up, not silent `None`.
- [ ] Command-discovery temporary fallbacks are converted or exactly inventoried with occurrence identity.
- [ ] Fill-correction stale target fallback is typed or exactly inventoried with occurrence identity.
- [ ] Skill parser optional-frontmatter fallback is typed or exactly inventoried with occurrence identity.
- [ ] Skill discovery diagnostics are first-class and path-private.
- [ ] Remote TTS settings expose typed absence and sanitized endpoint display.
- [ ] Remote ASR settings expose typed absence and sanitized endpoint display.
- [ ] Diagnostic redaction handles embedded URLs in prose.
- [ ] Provider handlers no longer return silent empty defaults for inconsistent post-persist settings.
- [ ] Direct-command evidence includes stronger behavioral/runtime-path tests.
- [ ] Allowlist and inventory remain exact, synchronized, and CI-enforced.
- [ ] Human-readable fallback documentation matches machine-readable inventory.
- [ ] This TODO retains its detailed task tree after closure.
- [ ] Permanent CI passes on the exact final `master` SHA.
- [ ] Final documentation states broader BBCR remediation remains open.

---

## Final evidence

Populate at closure:

- **Starting SHA:** pending
- **Starting permanent CI:** pending
- **Implementation SHA:** pending
- **Implementation permanent CI run/job:** pending
- **Cleanup SHA, if any:** pending
- **Implementation report SHA:** pending
- **Final TODO/documentation SHA:** pending
- **Final permanent CI run:** pending
- **Final permanent CI job:** pending
- **Final result:** pending

## Final bounded statement

> Pending implementation. This TODO will be complete only after the fallback-enforcement changes are implemented, all checklist items remain visible and checked, temporary files are absent, and permanent CI passes on the exact final `master` SHA. Completion of this TODO will not declare the broader BBCR remediation program complete or the project production release-ready.
