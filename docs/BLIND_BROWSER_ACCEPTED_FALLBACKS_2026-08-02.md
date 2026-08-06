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

- `permanent_accepted`: **22**
- `temporary_accepted`: **0**
- converted or removed across post-P8 passes: **18**

`permanent_accepted` is reserved for capability-reducing, presentation-only, checked-conversion, or feature-disabled behavior that does not hide protected-operation failure. These entries must be reconsidered if they begin affecting authority, persistence success, or a public error contract.

`temporary_accepted` requires an actionable `owner_note` and a concrete `review_due` boundary. This enforcement pass converted the four remaining command/fill temporary fallbacks and reclassified optional skill intent tags as permanently capability-reducing, leaving no temporary accepted fallback entries.

Every accepted occurrence is now identified by **path + function + normalized expression + occurrence index + adjacent normalized context**. A new duplicate `.ok()` or `.unwrap_or_default()` in an already allowlisted function therefore fails CI instead of inheriting approval.

Permanent CI verifies the live source expression, containing function, complete metadata, valid disposition, and temporary-review requirements. It also verifies the counts and temporary-entry summary in this document.

## Remaining temporary accepted fallbacks

None. Any future temporary fallback must carry a unique occurrence identity, actionable owner note, and concrete review boundary.

## Permanent accepted categories

| Category | Current behavior | Why it is accepted | User/diagnostic effect | Replacement boundary |
|---|---|---|---|---|
| Optional element scoring and click classification | Missing accessible name, placeholder, text, href, or value contributes empty comparison text | Absence lowers information and confidence; it cannot mint click authorization or mark a destructive target safe | Protected summaries emit target/degradation warnings where user-facing identity matters | Introduce richer typed page-model absence only if diagnostics need the distinction |
| Optional form destination parsing | Invalid optional form-action URL metadata becomes `None` | Missing destination cannot authorize submission and keeps confirmation conservative | Confirmation warnings identify unavailable destination metadata | Retain `Option` unless UI requires typed URL-parse reasons |
| Confirmation display-only text | Missing non-authoritative display text becomes empty | Confirmation ID, digest, expiry, runtime binding, and runtime-authored text remain authoritative | Protected confirmation remains visible and validated | Change only if display text becomes contractual |
| Checked numeric conversion/parsing | Overflow or invalid numeric input becomes validation absence/failure | Checked conversion cannot widen bounds or increase authority | Invalid input is rejected or reported unavailable | Retain checked conversions |
| Feature-disabled remote TTS stub | Parameters are consumed before a typed unavailable error | No network, playback, or successful fallback occurs | Caller receives explicit feature-unavailable failure | Remove only if remote TTS becomes mandatory in every build |

## Converted or removed fallbacks

The post-P8 passes removed or converted eighteen exact accepted expressions:

- two ignored URL userinfo mutation results in planner sanitization;
- three settings/model lookup `.ok()` paths;
- one settings voice `.unwrap_or_default()` path;
- one quiet skill-directory `filter_map(Result::ok)` skip;
- one executor policy-detail serialization `.ok()` expression;
- five validator policy-detail serialization `.ok()` expressions;
- project-root and user-skill-root discovery `.ok()` fallbacks;
- direct focus-query construction `.ok()?`;
- recent fill-correction candidate `.ok()` omission;
- planner page-origin parse `.ok()` fallback and lower-priority URL fallthrough.

They were replaced by:

- URL reconstruction from approved origin/path components;
- typed settings capability absence reasons;
- bounded path-private skill-entry warning aggregation;
- explicit policy-detail serialization failure markers;
- authoritative first-present page URL parsing with explicit rejection of parse failures and non-tuple origins.

## Authoritative exact inventory

The authoritative per-expression records are in `scripts/security-fallback-inventory.json`. Each record contains:

- source path and containing function(s);
- exact normalized expression;
- justification and user visibility;
- side-effect impact and tests;
- future replacement plan;
- disposition, review boundary, and owner note.

The JSON inventory is intentionally authoritative rather than duplicating a large generated Markdown table that can become stale. `scripts/check-security-fallback-inventory.py` verifies source/allowlist/inventory parity and the human-readable counts and temporary-entry summary above.
