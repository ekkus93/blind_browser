# Blind Browser Accepted Fallback Inventory

**Original date:** 2026-08-02  
**Post-P8 reconciliation:** 2026-08-03  
**Scope:** Production fallback expressions in security-sensitive Rust and frontend paths  
**Machine-readable allowlist:** `scripts/security-fallback-allowlist.txt`  
**Machine-readable inventory:** `scripts/security-fallback-inventory.json`  
**Enforcement:** `scripts/check-security-fallbacks.py` and `scripts/check-security-fallback-inventory.py`  
**Status:** Reconciled and disposition-classified for the post-P8 fallback/evidence hardening pass.

## Acceptance rules

An allowlisted expression is not automatically safe. It is accepted only when its failure mode cannot authorize a stronger action, leak a secret, report false success for a protected side effect, or replace a verified artifact.

Every accepted fallback must satisfy all of these rules:

1. It only reduces capability, omits optional presentation/diagnostic detail, or selects a documented conservative default.
2. It cannot convert a failed credential, confirmation, download, persistence, navigation, or protected-action operation into success.
3. It cannot reduce confirmation requirements or create authorization.
4. It cannot expose raw page, OCR, form, transcript, credential, provider-response, or private-path data.
5. It is listed as an exact normalized expression in the allowlist and inventory.
6. Its disposition, review boundary, justification, visibility, side-effect impact, tests, and replacement plan are machine-readable.
7. Refactoring an accepted expression requires renewed review because CI matches exact source expressions and containing functions.

## Disposition policy and counts

- `permanent_accepted`: **21**
- `temporary_accepted`: **5**
- converted or removed in this pass: **13**

`permanent_accepted` is reserved for capability-reducing, presentation-only, checked-conversion, or feature-disabled behavior that does not hide protected-operation failure. These entries must be reconsidered if they begin affecting authority, persistence success, or a public error contract.

`temporary_accepted` requires an actionable `owner_note` and a concrete `review_due` boundary. All five temporary entries are due **before the release-candidate gate**.

Permanent CI verifies the live source expression, containing function, complete metadata, valid disposition, and temporary-review requirements. It also verifies the counts and temporary-entry summary in this document.

## Remaining temporary accepted fallbacks

- `src-tauri/src/app_core/command_dispatch.rs` — `.ok()` in `build_planner_resolution`. Optional candidate discovery failure reduces capability. Replace with a typed candidate-rejection reason before the release-candidate gate when the UI can display it safely.
- `src-tauri/src/app_core/command_dispatch.rs` — `let current_dir = std::env::current_dir().ok();` in `build_planner_resolution`. Missing optional project context reduces skill discovery only. Replace with a bounded typed project-context diagnostic before the release-candidate gate.
- `src-tauri/src/app_core/fill_correction.rs` — `.ok()` in `resolve_recent_fill_correction_command`. Invalid optional correction discovery is omitted. Replace with a typed correction-rejection reason before the release-candidate gate.
- `src-tauri/src/app_core/form_fill/field_focus.rs` — `let search_query = build_find_element_query(&query).ok()?;` in `resolve_direct_focus_field_command`. Failed optional focus-query construction aborts that candidate. Replace with a typed query-construction reason before the release-candidate gate.
- `src-tauri/src/commands/skill_parser.rs` — `.unwrap_or_default()` in `skill_frontmatter_from_parts`. Missing optional frontmatter text reduces descriptive capability. Replace with typed per-entry parser diagnostics before the release-candidate gate if the settings/status UI can expose them without path leakage.

None of these temporary entries can grant tools, authorize an action, lower confirmation, report a protected side effect as successful, or leak sensitive content. Their exact source and metadata remain CI-enforced.

## Permanent accepted categories

| Category | Current behavior | Why it is accepted | User/diagnostic effect | Replacement boundary |
|---|---|---|---|---|
| Optional element scoring and click classification | Missing accessible name, placeholder, text, href, or value contributes empty comparison text | Absence lowers information and confidence; it cannot mint click authorization or mark a destructive target safe | Protected summaries emit target/degradation warnings where user-facing identity matters | Introduce richer typed page-model absence only if diagnostics need the distinction |
| Optional form destination/origin parsing | Invalid optional URL metadata becomes `None` | Missing destination/origin cannot authorize submission and keeps confirmation conservative | Confirmation warnings identify unavailable destination/page metadata | Retain `Option` unless UI requires typed URL-parse reasons |
| Confirmation display-only text | Missing non-authoritative display text becomes empty | Confirmation ID, digest, expiry, runtime binding, and runtime-authored text remain authoritative | Protected confirmation remains visible and validated | Change only if display text becomes contractual |
| Checked numeric conversion/parsing | Overflow or invalid numeric input becomes validation absence/failure | Checked conversion cannot widen bounds or increase authority | Invalid input is rejected or reported unavailable | Retain checked conversions |
| Planner page-origin metadata | Invalid optional origin is omitted after privacy controls remain authoritative | Cannot bypass remote-data consent, high-risk blocking, or credential scoping | Origin is unavailable/redacted | Add typed reason only if privacy UI needs it |
| Feature-disabled remote TTS stub | Parameters are consumed before a typed unavailable error | No network, playback, or successful fallback occurs | Caller receives explicit feature-unavailable failure | Remove only if remote TTS becomes mandatory in every build |

## Converted or removed fallbacks

This pass removed thirteen exact accepted expressions:

- two ignored URL userinfo mutation results in planner sanitization;
- three settings/model lookup `.ok()` paths;
- one settings voice `.unwrap_or_default()` path;
- one quiet skill-directory `filter_map(Result::ok)` skip;
- one executor policy-detail serialization `.ok()` expression;
- five validator policy-detail serialization `.ok()` expressions.

They were replaced by:

- URL reconstruction from approved origin/path components;
- typed settings capability absence reasons;
- bounded path-private skill-entry warning aggregation;
- explicit policy-detail serialization failure markers.

## Authoritative exact inventory

The authoritative per-expression records are in `scripts/security-fallback-inventory.json`. Each record contains:

- source path and containing function(s);
- exact normalized expression;
- justification and user visibility;
- side-effect impact and tests;
- future replacement plan;
- disposition, review boundary, and owner note.

The JSON inventory is intentionally authoritative rather than duplicating a large generated Markdown table that can become stale. `scripts/check-security-fallback-inventory.py` verifies source/allowlist/inventory parity and the human-readable counts and temporary-entry summary above.
