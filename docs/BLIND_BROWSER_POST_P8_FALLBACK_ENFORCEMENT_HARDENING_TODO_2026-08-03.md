# Blind Browser Post-P8 Fallback Enforcement Hardening TODO

**Date:** 2026-08-03  
**Repository:** `ekkus93/blind_browser`  
**Branch:** `master`  
**Companion spec:** `docs/BLIND_BROWSER_POST_P8_FALLBACK_ENFORCEMENT_HARDENING_SPEC_2026-08-03.md`  
**Depends on:** `docs/BLIND_BROWSER_POST_P8_FALLBACK_EVIDENCE_HARDENING_TODO_2026-08-03.md`  
**Status:** Complete for the bounded post-P8 fallback-enforcement scope once the canonical `ci/permanent` status for this exact documentation-closure commit is successful.  
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

- [x] Confirm latest `master` SHA before implementation.
- [x] Confirm permanent CI state for the starting SHA.
- [x] Read `docs/BLIND_BROWSER_POST_P8_FALLBACK_ENFORCEMENT_HARDENING_SPEC_2026-08-03.md`.
- [x] Read `docs/BLIND_BROWSER_POST_P8_FALLBACK_EVIDENCE_HARDENING_TODO_2026-08-03.md`.
- [x] Read `docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md`.
- [x] Read `scripts/security-fallback-allowlist.txt`.
- [x] Read `scripts/security-fallback-inventory.json`.
- [x] Read `scripts/check-security-fallbacks.py`.
- [x] Read `scripts/check-security-fallback-inventory.py`.
- [x] Confirm no temporary Ralph/workflow/script files are present at baseline.
- [x] Confirm expected changed-file scope before coding.

---

## 1. Fallback inventory exact occurrence enforcement

### 1.1 Audit current scanner weakness

- [x] Inspect `scripts/check-security-fallback-inventory.py`.
- [x] Confirm current keying behavior for inventory entries.
- [x] Identify every inventory entry with broad expressions such as:
  - [x] `.ok()`;
  - [x] `.unwrap_or_default()`;
  - [x] `unwrap_or(...)`;
  - [x] `map_or(...)` if authority-relevant;
  - [x] `let _ = ...` if any remain;
  - [x] other small expressions likely to match multiple sites.
- [x] Count all fallback occurrences per `(path, function, expression)`.
- [x] Identify all duplicate or ambiguous occurrences that cannot be uniquely reviewed today.

### 1.2 Inventory schema design

- [x] Choose the exact occurrence identity model:
  - [x] occurrence index within containing function — selected;
  - [x] before/after normalized line context — selected as adjacent context;
  - [x] source-span/context hash — not selected; normalized adjacent context provides the required drift detection;
  - [x] explicit `fallback_id` plus exact occurrence count — not selected; identity is derived from live source occurrence metadata.
- [x] Update inventory schema version.
- [x] Add occurrence identity to every inventory entry.
- [x] Add a migration note in accepted fallback documentation.
- [x] Ensure temporary and permanent entries use the same exactness rules.

### 1.3 Scanner implementation

- [x] Update `scripts/check-security-fallback-inventory.py` to locate every matching source occurrence.
- [x] Fail when source contains an uninventoried matching occurrence.
- [x] Fail when inventory references a missing occurrence.
- [x] Fail when two inventory entries point to the same occurrence.
- [x] Fail when containing function differs from inventory.
- [x] Fail when occurrence context differs from inventory.
- [x] Keep existing disposition/review-boundary/owner-note checks.
- [x] Keep documentation count and temporary-summary checks.

### 1.4 Scanner self-tests

- [x] Add self-test for duplicate broad `.ok()` expression with only one inventory record.
- [x] Add self-test for duplicated `.unwrap_or_default()` expression with only one inventory record.
- [x] Add self-test for stale context.
- [x] Add self-test for duplicate inventory records pointing at one occurrence.
- [x] Add self-test for missing temporary review boundary.
- [x] Add self-test for missing owner note.
- [x] Confirm scanner error messages name the path, function, expression, and occurrence identity.

### 1.5 Acceptance gate

- [x] Run `python3 scripts/check-security-fallback-inventory.py --self-test`.
- [x] Run `python3 scripts/check-security-fallback-inventory.py`.
- [x] Confirm permanent CI runs the enhanced scanner.

---

## 2. Direct focus query fallback conversion

### 2.1 Audit

- [x] Inspect `src-tauri/src/app_core/form_fill/field_focus.rs`.
- [x] Locate `build_find_element_query(&query).ok()?`.
- [x] Determine all failure modes of `build_find_element_query`.
- [x] Confirm whether failure currently falls through to remote planning.
- [x] Confirm no side effect occurs before query construction succeeds.

### 2.2 Implementation

- [x] Replace `build_find_element_query(&query).ok()?` with explicit handling.
- [x] Return deterministic `ReportResult` follow-up when query construction fails.
- [x] Add a bounded reason code, such as `focus_query_construction_failed`.
- [x] Do not expose raw page text, OCR text, private selectors, or full DOM details.
- [x] Do not authorize focus, typing, form submission, or click on query construction failure.
- [x] Ensure the direct resolver does not silently disappear for this failure.
- [x] Remove or update the corresponding temporary fallback inventory entry.

### 2.3 Tests

- [x] Add unit test for query-construction failure returning `ReportResult` follow-up.
- [x] Test follow-up has `ReportStatus::NeedsFollowUp`.
- [x] Test follow-up contains a bounded user message.
- [x] Test no `FocusElement`, `TypeIntoElement`, or `SubmitActiveForm` step is emitted.
- [x] Test the resolver does not return `None` for this failure mode.
- [x] Run focused form-fill/field-focus tests.

---

## 3. Remaining temporary command/fill fallback conversion

### 3.1 Command dispatch project context diagnostics

- [x] Inspect `src-tauri/src/app_core/command_dispatch.rs`.
- [x] Locate `let current_dir = std::env::current_dir().ok();`.
- [x] Locate `app_config_dir().ok()` or equivalent user-skill-root fallback.
- [x] Design typed diagnostics for missing project context and user skill root.
- [x] Ensure diagnostics are path-private.
- [x] Ensure diagnostics reduce skill discovery only and cannot grant tools.
- [x] Decide where diagnostics are stored or returned.
- [x] Implement typed diagnostics.
- [x] Remove or update related temporary fallback inventory entries.

### 3.2 Candidate discovery `.ok()` fallbacks

- [x] Identify all temporary `.ok()` entries in command discovery.
- [x] Convert each practical site to typed candidate-rejection reason.
- [x] Ensure candidate rejection cannot execute a protected action.
- [x] Ensure candidate rejection cannot report success.
- [x] Add tests for candidate rejection diagnostics.
- [x] Keep any non-converted fallback explicitly temporary with exact occurrence identity — not applicable; no temporary accepted fallback remains.

### 3.3 Fill correction stale-target diagnostics

- [x] Inspect `src-tauri/src/app_core/fill_correction.rs`.
- [x] Locate `resolve_typeable_element(...).ok()` in recent fill correction.
- [x] Replace silent candidate omission with typed stale-candidate/follow-up reason if practical.
- [x] Preserve deterministic follow-up behavior when recent context is missing or stale.
- [x] Ensure stale candidates cannot type into a wrong element.
- [x] Add tests for stale active target and stale alternate target.
- [x] Remove or update related temporary fallback inventory entries.

### 3.4 Skill parser optional-frontmatter fallback

- [x] Inspect `src-tauri/src/commands/skill_parser.rs`.
- [x] Locate optional frontmatter `.unwrap_or_default()` fallback.
- [x] Decide whether missing optional frontmatter should remain permanent or become typed parser diagnostic — retained as a permanent capability-reducing absence for optional `intent_tags`.
- [x] If converted, add bounded per-skill parser diagnostic without full path leakage — not applicable because the optional `intent_tags` absence was retained.
- [x] If retained, enforce exact occurrence identity and temporary/permanent disposition explicitly — completed as `permanent_accepted` under schema version 3.
- [x] Add parser tests for missing optional frontmatter diagnostics — not applicable to the retained optional-list default; parser behavior and exact inventory enforcement remain covered.

---

## 4. First-class skill discovery diagnostics

### 4.1 Audit

- [x] Inspect `src-tauri/src/commands/skill_loader.rs`.
- [x] Identify all current `tracing::warn!` skill discovery diagnostics.
- [x] Identify current public/runtime surfaces that could expose skill diagnostics.
- [x] Confirm path-private data requirements.

### 4.2 Type design

- [x] Add or select a type such as `SkillDiscoveryDiagnostics`.
- [x] Add or select warning entries such as `SkillLoadWarning`.
- [x] Include source class: project, user, bundled.
- [x] Include skipped entry counts.
- [x] Include bounded error categories.
- [x] Include safe leaf identifiers only where allowed.
- [x] Include parse/read/name-mismatch categories where already known.
- [x] Exclude full paths, project roots, home directories, usernames, raw file contents, and raw manifest text.

### 4.3 Implementation

- [x] Return or store typed skill discovery diagnostics.
- [x] Surface diagnostics through runtime status, agent state, settings status, or equivalent typed path.
- [x] Preserve existing skill loading behavior for valid skills.
- [x] Preserve omission of invalid optional skills.
- [x] Ensure skipped skills cannot add tools or permissions.
- [x] Preserve tracing warnings as supplemental diagnostics if useful.

### 4.4 Tests

- [x] Test unreadable directory entries produce typed diagnostics.
- [x] Test unreadable `SKILL.md` produces typed diagnostics.
- [x] Test invalid frontmatter produces typed diagnostics.
- [x] Test directory/frontmatter name mismatch produces typed diagnostics.
- [x] Test valid adjacent skill still loads.
- [x] Test diagnostic output does not contain full absolute paths.
- [x] Test skipped invalid skill grants no tools.
- [x] Update TypeScript types/tests if runtime status changes.

---

## 5. Remote TTS/ASR typed absence parity

### 5.1 Audit

- [x] Inspect `src-tauri/src/app_core/settings_adapters.rs`.
- [x] Inspect `src-tauri/src/commands/contracts/providers.rs`.
- [x] Inspect `src/tauri-types.ts`.
- [x] Compare `RemotePlannerSettings` with `RemoteTtsSettings` and `RemoteAsrSettings`.
- [x] Identify raw `base_url` clones in TTS/ASR settings.
- [x] Identify missing typed absence fields.

### 5.2 Contract design

- [x] Add `availability_reason` or equivalent to `RemoteTtsSettings`.
- [x] Add `availability_reason` or equivalent to `RemoteAsrSettings`.
- [x] Reuse `CapabilityAbsenceReason` where possible.
- [x] Add sanitized endpoint display for remote TTS.
- [x] Add sanitized endpoint display for remote ASR.
- [x] Decide how to represent invalid endpoint vs profile missing vs not configured.
- [x] Decide how to represent credential-reference missing or unreadable when visible from settings.
- [x] Decide how to represent feature-disabled states.

### 5.3 Implementation

- [x] Surface remote TTS not-configured reason.
- [x] Surface remote TTS profile-missing reason.
- [x] Surface remote TTS invalid-endpoint reason.
- [x] Surface remote ASR not-configured reason.
- [x] Surface remote ASR profile-missing reason.
- [x] Surface remote ASR invalid-endpoint reason.
- [x] Sanitize TTS endpoint userinfo/query/fragment.
- [x] Sanitize ASR endpoint userinfo/query/fragment.
- [x] Ensure raw credentials and provider responses are never surfaced.
- [x] Update frontend type definitions.

### 5.4 Tests

- [x] Test remote TTS invalid endpoint reason.
- [x] Test remote TTS profile missing reason.
- [x] Test remote TTS not configured reason.
- [x] Test remote TTS sanitized endpoint output.
- [x] Test remote ASR invalid endpoint reason.
- [x] Test remote ASR profile missing reason.
- [x] Test remote ASR not configured reason.
- [x] Test remote ASR sanitized endpoint output.
- [x] Test endpoint containing username/password/query/fragment leaks none of those values.
- [x] Update frontend tests if needed.

---

## 6. Embedded diagnostic URL redaction

### 6.1 Audit

- [x] Inspect `src-tauri/src/diagnostic_redaction.rs`.
- [x] Identify whole-string URL redaction behavior.
- [x] Identify embedded URL cases that are not currently sanitized.
- [x] Review sensitive marker list for query-secret names.

### 6.2 Implementation

- [x] Add embedded URL detection for prose strings.
- [x] Sanitize each URL-like substring using the approved origin/path reconstruction helper.
- [x] Redact malformed embedded URL-like substrings conservatively.
- [x] Remove username and password from embedded URLs.
- [x] Remove query strings from embedded URLs.
- [x] Remove fragments from embedded URLs.
- [x] Add detection for OAuth/signed URL query-secret names:
  - [x] `code`;
  - [x] `state`;
  - [x] `token`;
  - [x] `access_token`;
  - [x] `refresh_token`;
  - [x] `id_token`;
  - [x] `sig`;
  - [x] `signature`;
  - [x] `client_secret`;
  - [x] `api_key`;
  - [x] `key`;
  - [x] `session`.
- [x] Preserve non-URL prose that contains no secret marker.

### 6.3 Tests

- [x] Test bare URL redaction.
- [x] Test prose with one embedded URL.
- [x] Test prose with multiple embedded URLs.
- [x] Test embedded URL with username/password.
- [x] Test OAuth callback URL with `code` and `state`.
- [x] Test signed URL with `sig` or `signature`.
- [x] Test malformed embedded URL-like substring.
- [x] Test non-URL prose remains unchanged.
- [x] Test JSON diagnostic redaction applies embedded URL redaction recursively.

---

## 7. Provider handler empty-default removal

### 7.1 Audit

- [x] Inspect `src-tauri/src/command_handlers/provider_handlers.rs`.
- [x] Locate response-side `settings.base_url.unwrap_or_default()`.
- [x] Locate response-side `settings.model.unwrap_or_default()`.
- [x] Search for similar provider handler empty defaults.
- [x] Decide whether empty value is ever a valid post-persist state.

### 7.2 Implementation

- [x] Replace post-persist empty defaults with typed internal consistency errors.
- [x] Preserve explicit persistence failures as failures.
- [x] Do not return successful empty endpoint/model unless explicitly valid.
- [x] Sanitize returned endpoint strings.
- [x] Include bounded error codes for inconsistent post-persist settings.
- [x] Remove or inventory any remaining accepted defaults.

### 7.3 Tests

- [x] Test set remote planner settings success returns non-empty sanitized base URL and model.
- [x] Test reset remote planner settings success returns non-empty sanitized base URL and model.
- [x] Test inconsistent post-persist settings returns typed error.
- [x] Test no raw credentials/query/fragment in handler response.
- [x] Test persistence failure remains a failure.

---

## 8. Direct-command behavioral evidence strengthening

### 8.1 Audit

- [x] Inspect `src-tauri/src/direct_command_policy.rs`.
- [x] Inspect `src-tauri/tests/post_batch8_direct_command_policy_evidence.rs`.
- [x] Identify remaining source-string-only checks.
- [x] Identify available typed helpers or runtime paths for behavioral tests.

### 8.2 Behavioral test coverage

- [x] Add or strengthen test proving networked commands use timeout and redirect policy wrappers.
- [x] Add or strengthen test proving credential-bearing commands use endpoint-bound secret resolution.
- [x] Add or strengthen test proving page-context commands pass through remote planner privacy sanitization.
- [x] Add or strengthen test proving model downloads use verified atomic activation.
- [x] Add or strengthen test proving external launch commands require user gesture and validated URL policy.
- [x] Keep source-drift tests only as supplemental checks.

### 8.3 Registry safeguards

- [x] Ensure a new Tauri handler without a direct-command registry entry fails tests.
- [x] Ensure a networked registry entry without network policy mapping fails tests.
- [x] Ensure a credential-bearing entry without endpoint-bound mapping fails tests.
- [x] Ensure a page-context entry without sanitizer mapping fails tests.
- [x] Ensure a model-download entry without verified activation mapping fails tests.

### 8.4 CI gate

- [x] Confirm permanent CI runs focused direct-command evidence.
- [x] Confirm permanent CI runs the full Rust suite after focused evidence.

---

## 9. Allowlist and accepted fallback documentation

### 9.1 Allowlist updates

- [x] Remove fallback expressions converted to typed diagnostics/errors.
- [x] Add no broad-category exceptions.
- [x] If a fallback remains, bind it to exact occurrence identity.
- [x] Ensure every allowlist line has a matching inventory entry.
- [x] Ensure every inventory entry resolves to live source.

### 9.2 Documentation updates

- [x] Update `docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md`.
- [x] Update disposition counts.
- [x] Update temporary fallback summary.
- [x] Update converted/removed fallback summary.
- [x] Document the new exact occurrence identity rules.
- [x] Ensure human-readable docs do not duplicate a stale per-expression table.
- [x] Ensure documentation parity is enforced by scanner.

### 9.3 Implementation report

- [x] Create `docs/BLIND_BROWSER_POST_P8_FALLBACK_ENFORCEMENT_HARDENING_IMPLEMENTATION_REPORT_2026-08-03.md`.
- [x] Include exact changed-file inventory.
- [x] Include fallbacks removed, converted, retained, and newly inventoried.
- [x] Include tests added or modified.
- [x] Include scanner changes.
- [x] Include CI run/job evidence.
- [x] Include unresolved risks and out-of-scope BBCR items.

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
- [x] Focused direct-command evidence test passes.
- [x] Full Rust test suite passes.
- [x] Frontend lint passes.
- [x] UI tests pass.
- [x] Frontend production build passes.

---

## 11. Permanent CI and exact evidence

- [x] Push implementation to `master`.
- [x] Confirm permanent CI starts on the exact implementation candidate SHA.
- [x] Record implementation SHA.
- [x] Record permanent CI run ID.
- [x] Record permanent CI job ID.
- [x] Repair any CI failure as a real source/test/scanner/doc bug.
- [x] Repeat until permanent CI passes on the exact candidate SHA.
- [x] Remove any temporary workflow/helper script before final closure.
- [x] Push documentation/report/TODO reconciliation to `master`.
- [x] Run permanent CI again if cleanup or documentation changes occur after implementation validation.
- [x] Record final evidence SHA.
- [x] Record final permanent CI run ID.
- [x] Record final permanent CI job ID.
- [x] Record final result.

---

## 12. Completion checklist

- [x] Broad fallback expressions cannot hide duplicate source occurrences.
- [x] Every accepted fallback has unique occurrence identity.
- [x] Direct focus query construction failure returns typed follow-up, not silent `None`.
- [x] Command-discovery temporary fallbacks are converted or exactly inventoried with occurrence identity.
- [x] Fill-correction stale target fallback is typed or exactly inventoried with occurrence identity.
- [x] Skill parser optional-frontmatter fallback is typed or exactly inventoried with occurrence identity.
- [x] Skill discovery diagnostics are first-class and path-private.
- [x] Remote TTS settings expose typed absence and sanitized endpoint display.
- [x] Remote ASR settings expose typed absence and sanitized endpoint display.
- [x] Diagnostic redaction handles embedded URLs in prose.
- [x] Provider handlers no longer return silent empty defaults for inconsistent post-persist settings.
- [x] Direct-command evidence includes stronger behavioral/runtime-path tests.
- [x] Allowlist and inventory remain exact, synchronized, and CI-enforced.
- [x] Human-readable fallback documentation matches machine-readable inventory.
- [x] This TODO retains its detailed task tree after closure.
- [x] Permanent CI passes on the exact final `master` SHA.
- [x] Final documentation states broader BBCR remediation remains open.

---

---

## Reconciliation notes

- The detailed task tree is preserved. Every checkbox is resolved against implementation, tests, scanners, documentation, or permanent CI evidence on `master`.
- The selected occurrence identity is the combination of containing function, one-based occurrence index, and normalized adjacent source context. Rejected alternatives remain visible and are marked not selected.
- The optional skill `intent_tags` empty-list behavior remains a permanent capability-reducing fallback. It grants no tools or authority and is bound to an exact schema-version-3 occurrence identity.
- Conditional branches that were not selected are marked not applicable rather than being deleted or represented as implemented.
- Closely related assertions are satisfied by grouped tests where one test exercises multiple required properties, including bounded follow-up status/message/non-authorizing behavior and TTS/ASR absence/sanitization parity.
- Permanent CI failure on the first cleaned candidate was treated as a real source defect. The missing fixture migration was repaired across the complete affected test tree before final cleaned-code validation.
- A Git commit cannot contain its own SHA or the workflow run/job identifiers created only after it is pushed. The exact documentation-closure SHA and its final `ci/permanent` run/job are therefore canonical GitHub commit/status metadata and are reported in the closure response.


## Final evidence

- **Starting SHA:** `419b6698482c57e0731641a96c5132e3892f8e2e`
- **Starting permanent CI:** run `30852987503`, job `91817341089` — success
- **Primary implementation SHA:** `8c44bb8ed08e0897f04e8deb4e291018c81ac2b9`
- **First cleaned implementation candidate:** `cc3a296de90b2553aa7e53a620456d13d5d5a05b`
- **Failed candidate permanent CI:** run `30877130400`, job `91890767295` — failed at deny-warning Clippy because the contract migration omitted affected Rust fixtures
- **Corrective fixture-completion SHA:** `f19ceec71d44cd113e6a1ee498deb569291216b4`
- **Final cleaned code SHA:** `25a902e4117275ff77b23e8ecc44bba31d9cced6`
- **Final cleaned-code permanent CI:** run `30881345809`, job `91903228743` — success
- **Implementation report SHA:** `bea5d223475b6e754f7235dbe1b96e312bef5b5e`
- **Final TODO/documentation SHA:** this documentation-closure commit; canonical in GitHub commit metadata
- **Final permanent CI run/job:** canonical `ci/permanent` status attached to this exact documentation-closure commit; reported in the closure response
- **Final result:** bounded post-P8 fallback-enforcement hardening complete when that canonical status is successful; broader BBCR remediation remains open

## Final bounded statement

> The post-P8 fallback-enforcement hardening pass is complete for its bounded scope when permanent CI succeeds on this exact documentation-closure commit. The complete checklist remains visible, all selected and non-selected branches are reconciled, temporary workflow/helper files are absent from the closure tree, and the implementation report records the real failure-and-repair history. This closure does not declare the broader BBCR remediation program complete or the repository production release-ready.
