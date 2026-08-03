# Blind Browser Post-P8 Fallback Enforcement Hardening Spec

**Date:** 2026-08-03  
**Repository:** `ekkus93/blind_browser`  
**Branch:** `master`  
**Companion TODO:** `docs/BLIND_BROWSER_POST_P8_FALLBACK_ENFORCEMENT_HARDENING_TODO_2026-08-03.md`  
**Depends on:** `docs/BLIND_BROWSER_POST_P8_FALLBACK_EVIDENCE_HARDENING_TODO_2026-08-03.md`  
**Scope:** Follow-up hardening for remaining quiet-fallback risks and fallback-inventory enforcement defects.  
**Release boundary:** This spec does not declare the whole BBCR program complete or the project production-ready.

---

## 1. Purpose

The post-P8 fallback/evidence hardening pass substantially improved the repository, but the review found several remaining risks:

1. The fallback inventory scanner uses `(path, expression)` keys and does not prove per-occurrence exactness for broad expressions like `.ok()`.
2. Some temporary `.ok()` fallbacks still silently remove deterministic command candidates instead of returning typed follow-up diagnostics.
3. Skill-loader warnings are emitted through tracing only, rather than through a first-class status or diagnostics surface.
4. Remote TTS and remote ASR settings do not yet have the same typed absence and sanitized endpoint behavior as remote planner settings.
5. Diagnostic URL redaction handles full-string URLs better than embedded URLs inside prose.
6. Some provider handlers still convert unexpected missing settings into empty strings with `unwrap_or_default()`.
7. Direct-command evidence is stronger, but some behavior remains manually classified rather than runtime-backed.

This spec defines a bounded hardening pass to remove or contain those gaps.

---

## 2. Non-goals

This pass must not expand into the entire remaining BBCR remediation program. Specifically out of scope unless directly touched by the fixes below:

- full production release readiness;
- CSP and packaged app release policy;
- full Windows/macOS packaging validation;
- historical secret scanning beyond the existing diagnostics/fallback contract;
- broad UX redesign;
- unrelated model-download, OCR, ASR, TTS, or planner behavior changes;
- broad refactors that make the security diff hard to review.

---

## 3. Terms

### 3.1 Fallback site

A fallback site is any source expression that intentionally collapses an error, missing optional field, failed parse, unavailable capability, invalid metadata, or diagnostic serialization issue into a default, omission, warning, or reduced-capability path.

Examples include:

- `.ok()`;
- `.unwrap_or_default()`;
- `.unwrap_or(...)`;
- `.map_or(...)` when used to hide missing authority-relevant data;
- `let _ = ...` ignored results;
- `filter_map(Result::ok)`;
- defaulting missing settings to empty strings.

### 3.2 Accepted fallback

An accepted fallback is a reviewed fallback site that is explicitly listed in the allowlist and machine inventory, with exact source binding, disposition, user visibility, side-effect impact, test coverage, replacement plan, review boundary, and owner note.

### 3.3 Temporary accepted fallback

A temporary accepted fallback is a reviewed fallback site that can remain only until the next declared boundary, usually `before_release_candidate`. It must have an actionable replacement plan.

### 3.4 Typed diagnostic

A typed diagnostic is a structured result, warning, or absence reason that identifies why a capability was reduced without exposing secrets, full paths, raw page text, OCR text, credentials, or raw provider responses.

### 3.5 Path-private diagnostic

A path-private diagnostic may include source class, bounded leaf name, count, error kind/category, and broad location class, but must not include full absolute paths, home directory names, project roots, usernames, or private file contents.

---

## 4. Security invariants

All changes in this pass must preserve these invariants:

1. A fallback must never authorize a click, form submission, external launch, credential operation, model activation, network planner request, or other protected action.
2. A fallback must never reduce confirmation requirements.
3. A fallback must never report success for a failed persistence, credential, download, confirmation, protected action, or network operation.
4. Diagnostic paths must not leak secrets, raw page/OCR/transcript text, provider response bodies, authorization headers, cookies, tokens, private local paths, or credential-shaped values.
5. Invalid configuration must be typed as invalid, missing, unavailable, or disabled; it must not be silently treated as a successful empty configuration.
6. A temporary fallback must be exact, counted, documented, tested, and have a review boundary.
7. A source refactor that moves or duplicates an accepted fallback must fail CI until the inventory is updated and re-reviewed.
8. The detailed TODO checklist must be preserved through closure; final evidence must be appended rather than replacing the checklist with a summary.

---

## 5. Requirements

### FEH-001: Exact fallback occurrence enforcement

`script/security-fallback-inventory.json` and `scripts/check-security-fallback-inventory.py` must move beyond `(path, expression)` set matching.

The implementation must make every accepted fallback site uniquely auditable. Acceptable designs include one of:

- `path + function + normalized expression + occurrence_index`;
- `path + function + normalized expression + before_context + after_context`;
- `path + function + normalized expression + source_span_hash`;
- stable `fallback_id` records that are checked against exact occurrence counts and context.

The scanner must fail when:

- a broad expression such as `.ok()` appears more times than inventoried;
- an inventoried expression disappears;
- an inventoried expression moves to an unexpected containing function;
- two inventory entries point to the same occurrence;
- a temporary fallback lacks a review boundary or owner note;
- documentation counts or temporary summaries drift from JSON inventory.

Scanner self-tests must include duplicate broad-expression fixtures that fail unless every occurrence is separately inventoried.

### FEH-002: Replace direct focus query silent fallback

`resolve_direct_focus_field_command` currently may silently abandon a deterministic local command when `build_find_element_query(&query).ok()?` fails.

Replace this with a typed follow-up path. The resolver must return a deterministic `ReportResult`-based follow-up describing that the field query could not be constructed, without falling through silently to remote planning.

The follow-up must:

- avoid raw page/OCR text;
- include a bounded reason code;
- preserve local deterministic handling;
- not authorize focus, typing, or submission;
- not reduce confirmation.

### FEH-003: Promote remaining temporary command-discovery fallbacks

The remaining temporary fallbacks in command discovery and fill correction must be converted where practical:

- `std::env::current_dir().ok()` should become a typed project-context diagnostic.
- `app_config_dir().ok()` should become a typed user-skill-root diagnostic.
- command/fill candidate `.ok()` omissions should become typed candidate-rejection reasons or deterministic follow-up outputs.
- fill-correction stale target resolution should identify stale/invalid candidate reasons instead of silently skipping them.

If a fallback remains temporary after this pass, the spec must explain why it cannot be converted now and CI must continue enforcing it as a temporary exact occurrence.

### FEH-004: First-class skill discovery diagnostics

Skill discovery must expose path-private diagnostics through a typed surface, not only tracing.

The implementation may introduce one of:

- `SkillDiscoveryResult { skills, diagnostics }`;
- `SkillDiscoveryDiagnostics` stored in runtime status;
- `SkillLoadWarning` entries included in agent/runtime state;
- an equivalent bounded diagnostics surface.

Diagnostics must include:

- source class: project, user, bundled;
- count of skipped unreadable entries;
- bounded error categories;
- safe leaf identifiers only where allowed;
- parse/read/name-mismatch categories where already known;
- no full absolute paths or private directory names.

Existing skill loading behavior may still skip invalid optional skills, but the skip must be visible to diagnostics/status without granting tools or permissions.

### FEH-005: Remote TTS/ASR typed absence parity

Remote TTS and remote ASR settings must gain typed absence/degradation parity with remote planner settings.

Add or extend output contracts so these settings can represent:

- not configured;
- profile missing;
- invalid endpoint;
- credential reference missing or unreadable where applicable;
- feature disabled where applicable;
- sanitized display endpoint.

Remote TTS/ASR endpoint display must not expose:

- username;
- password;
- query strings;
- fragments;
- token-shaped path/query material if detected;
- raw provider responses.

Frontend TypeScript types and tests must be updated if the runtime status shape changes.

### FEH-006: Embedded diagnostic URL redaction

Diagnostic text redaction must sanitize URL-like substrings inside prose, not only whole-string URLs.

The redactor must remove or redact:

- URL userinfo;
- query strings;
- fragments;
- common OAuth/query secret names such as `code`, `state`, `token`, `access_token`, `refresh_token`, `id_token`, `sig`, `signature`, `client_secret`, `api_key`, `key`, `session`, and similar values;
- malformed embedded URLs conservatively.

Tests must cover:

- a bare URL;
- a prose string containing a URL;
- multiple URLs in one string;
- URL with username/password;
- OAuth callback URL with `code` and `state`;
- signed URL with `sig` or `signature`;
- non-URL prose that should remain unchanged.

### FEH-007: Provider handler empty-default removal

Provider handlers must not return empty strings after successful persistence unless empty strings are a valid, explicit configuration state.

Replace response-side `unwrap_or_default()` patterns after setting/resetting remote planner settings with typed internal consistency errors or explicit typed absence fields.

The fix must ensure:

- persistence failure remains a failure;
- missing settings after success is not reported as successful empty data;
- returned endpoint/model values are sanitized where applicable;
- tests cover inconsistent post-persist settings.

### FEH-008: Direct-command behavioral evidence strengthening

The direct-command registry is useful, but manual classification is not enough on its own.

Add targeted behavioral/source-bound tests that prove key command classes use approved runtime paths:

- networked commands use timeout and redirect policy wrappers;
- credential-bearing commands use endpoint-bound secret resolution;
- page-context commands pass through remote planner privacy sanitization;
- model-download commands activate artifacts only through verified atomic activation;
- external launch commands require explicit user gesture and validated URL policy.

The existing source-drift checks may remain supplemental, but at least one test per class must assert behavior through a typed helper, public wrapper, or runtime path rather than only searching strings.

### FEH-009: CI and documentation enforcement

Permanent CI must run:

- silent-fallback shell scanner;
- reviewed security fallback scanner;
- enhanced exact fallback inventory scanner with self-tests;
- sensitive diagnostics scanner;
- focused direct-command evidence test;
- Rust formatting/check/Clippy/tests;
- frontend lint/UI/build if runtime status shapes change.

The accepted fallback documentation must remain machine-verifiable against the inventory.

### FEH-010: Closure and evidence

Completion requires:

- implementation committed directly to `master` unless user explicitly asks otherwise;
- no temporary workflow/script files left in the final tree;
- every TODO task checked only after source/test/scanner/doc evidence exists;
- permanent CI success on the final evidence SHA;
- final documentation stating that the broader BBCR remediation program remains open.

---

## 6. Expected changed files

Likely changed files include, but are not limited to:

- `scripts/check-security-fallback-inventory.py`;
- `scripts/security-fallback-inventory.json`;
- `scripts/security-fallback-allowlist.txt`;
- `docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md`;
- `src-tauri/src/app_core/command_dispatch.rs`;
- `src-tauri/src/app_core/fill_correction.rs`;
- `src-tauri/src/app_core/form_fill/field_focus.rs`;
- `src-tauri/src/commands/skill_loader.rs`;
- `src-tauri/src/commands/skill_parser.rs`;
- `src-tauri/src/app_core/settings_adapters.rs`;
- `src-tauri/src/commands/contracts/providers.rs`;
- `src-tauri/src/command_handlers/provider_handlers.rs`;
- `src-tauri/src/diagnostic_redaction.rs`;
- `src-tauri/src/direct_command_policy.rs`;
- `src-tauri/tests/post_batch8_direct_command_policy_evidence.rs`;
- `src/tauri-types.ts`.

---

## 7. Acceptance criteria

This pass is complete only when all of the following are true:

1. Broad fallback expressions cannot hide duplicate fallback sites.
2. Every accepted fallback occurrence has unique machine-checkable identity.
3. The direct focus query construction fallback is no longer silent.
4. Temporary command/fill/skill-parser fallbacks are either converted or remain explicitly temporary with exact occurrence enforcement.
5. Skill discovery diagnostics are first-class and path-private.
6. Remote TTS/ASR settings expose typed absence and sanitized endpoints.
7. Embedded diagnostic URLs are redacted.
8. Provider handler post-persist empty defaults are removed or typed.
9. Direct-command evidence includes stronger behavioral/runtime-path tests.
10. Permanent CI validates the final `master` SHA.
11. The final report explicitly says this is bounded fallback enforcement hardening, not general release readiness.
