# Blind Browser Accepted Fallback Inventory

**Date:** 2026-08-02  
**Scope:** Production fallback expressions in security-sensitive Rust and frontend paths  
**Machine-readable allowlist:** `scripts/security-fallback-allowlist.txt`  
**Enforcement:** `scripts/check-security-fallbacks.py`  
**Status:** Reconciled for the post-Batch-8 hardening branch; final exact-SHA CI evidence remains pending.

## Rules

An allowlisted expression is not automatically safe. It is accepted only when its failure mode cannot authorize a stronger action, leak a secret, report false success for a protected side effect, or replace a verified artifact. New expressions fail CI until reviewed.

Every accepted fallback must satisfy all of the following:

1. It can only reduce capability, omit optional presentation/diagnostic detail, or select an explicitly documented conservative default.
2. It cannot turn a failed credential, confirmation, download, persistence, navigation, or protected-action operation into success.
3. It cannot reduce a deterministic confirmation requirement or create an authorization token.
4. It cannot expose raw page/OCR/form/transcript/credential data.
5. Its source line is present in the exact machine allowlist or excluded because it is test-only code.
6. Refactoring the source line requires renewed review because the allowlist uses normalized exact expressions.

## Reviewed inventory

| Category | Files/functions | Expression/fallback | Justification and side-effect impact | User visibility | Tests / enforcement | Future replacement |
|---|---|---|---|---|---|---|
| Optional element display/scoring text | `src-tauri/src/app_core/element_scoring.rs`; destructive/sensitive label builders in `click_authorization.rs` | Missing optional label, text, placeholder, href, or value becomes empty comparison text | Used only for deterministic scoring/classification. It cannot execute an action. Unresolved, ambiguous, low-confidence, or risky targets remain confirmation-required or rejected. | Confirmation text now emits an explicit target-label warning when required metadata is unavailable. | Click authorization, destructive-click, ambiguity, stale-target, and confirmation tests; fallback scanner. | Replace with richer typed absence reasons only when the page-model contract benefits from the extra distinction. |
| Optional URL-derived metadata | `normalized_origin`; form destination extraction | Parse failure becomes `None` | Invalid/missing origin metadata cannot authorize an action. Confirmation remains required and the manifest now includes digest-bound destination/page-metadata warnings. | Explicit warning in degraded confirmation summaries. | Confirmation manifest degradation and digest tests. | Retain `Option`; add more granular typed parse reasons only if the UI needs them. |
| Optional runtime presentation state | confirmation response prompt lookup; skill display descriptions | Missing non-authoritative display text becomes empty text | Confirmation ID/digest/runtime validation remains authoritative. Skill descriptions cannot grant tools or bypass policy. | Missing protected target/form metadata is explicitly warned; ordinary optional descriptive text may remain absent. | Confirmation mismatch/replay tests; skill parser/policy tests; fallback scanner. | No security replacement required. |
| Bounded numeric parsing | audio routing, ASR WAV sizing, click confidence decoding, direct-command parsing | Failed checked conversion/parse returns `None` or enters validation error branch | Invalid protected arguments are rejected or require confirmation; no unchecked wrap or authority expansion occurs. | Typed validation failure or unavailable capability state. | Boundary tests plus scanner allowlist. | Retain checked conversions. |
| Optional policy diagnostics | policy decision detail serialization | `serde_json::to_value(...).ok()` may omit optional detail | The error code, refusal, retryability, and action classification are already fixed before optional detail serialization. Failure removes diagnostics only. | User still receives the typed failure without optional detail. | Centralized `ToolError` serialization/redaction tests; fallback scanner. | Replace only if policy detail becomes contractually mandatory. |
| Capability discovery | current directory, app config directory, user skill root, optional model plan | Unavailable optional source may be omitted | Omission cannot increase authority. Project/user skill discovery failures are now logged without full local paths; configured required model/profile operations still fail explicitly. | Skill directory read failures are logged with source class/leaf name/error kind. Settings/runtime panels expose unavailable configured capabilities. | Skill diagnostic privacy test; model availability tests; scanner. | Continue converting user-actionable configured capability failures to explicit UI warnings as the UI gains dedicated surfaces. |
| Best-effort non-authoritative cleanup | temporary config/image-cache cleanup after a primary failure or cache eviction | Cleanup failure may be secondary | Cleanup does not report the protected operation as successful and cannot activate unverified model bytes. Verified model `.part` cleanup is explicitly not allowlisted: failure returns `ModelDownloadError::CleanupFailed`. | Primary failure remains visible; secondary cache cleanup may be diagnostic-only. | Security fallback scanner synthetic tests; model cleanup tests. | Retain only for disposable cache/config artifacts. |
| Feature-disabled stubs | remote TTS feature-disabled implementation | Parameters are consumed before typed unavailable error | No request or fallback operation succeeds. | Explicit feature-unavailable error. | Feature-matrix compile/tests; scanner. | Remove stub only if feature becomes mandatory. |
| URL sanitization setters | provider/diagnostic URL redaction helpers | Results of `Url::set_username`/`set_password` are ignored after successful parsing | The URL is subsequently stripped of query and fragment data. Failure to clear credentials is covered by output assertions; the value is never used to authorize navigation or credentials. | Sanitized endpoint/path or generic redacted URL. | Planner payload, diagnostic redaction, and external-link tests. | Replace with a reconstruction-from-components helper if URL crate semantics change. |

## Converted unsafe or ambiguous fallbacks

The following are no longer accepted fallbacks:

- API-key setters no longer use `.api_key_reference.unwrap_or_default()`. Planner, TTS, and ASR persistence report typed invariant failures when a non-empty reference is absent.
- External URL opening no longer trusts `starts_with("https://")`. URLs are parsed and normalized; non-HTTPS schemes, missing hosts, control characters, embedded credentials, query strings, fragments, and malformed inputs are rejected.
- Confirmation summaries no longer silently degrade to generic form/destination/field wording. Structured warning codes and user-visible text are part of the manifest digest.
- Model availability directory iteration no longer silently skips unreadable entries.
- Verified model-download cleanup failure is not ignored.
- Verified model bytes are written only to `.part`, bounded, SHA-256 verified, synced, and atomically activated.
- Unknown model IDs fail closed unless a pinned integrity manifest exists.
- Raw frontend caught errors are no longer written directly to `console.error`; they pass through the diagnostic classifier/redactor.
- Skill-loading diagnostics no longer reveal full project or user-home paths.

## Scanner coverage

`scripts/check-security-fallbacks.py` and its machine-readable allowlist detect reviewed security-sensitive fallback expressions, including:

- suspicious `.ok()`;
- suspicious `.unwrap_or_default()`;
- ignored `Result` patterns;
- direct model final-path writes;
- API-key-reference defaults;
- unchecked diagnostic/serialization fallbacks.

Permanent CI runs:

```text
python3 scripts/check-security-fallbacks.py --self-test
python3 scripts/check-security-fallbacks.py
```

The self-test proves that synthetic unsafe `.ok()`, direct final-path model writes, and API-key-reference `.unwrap_or_default()` patterns fail, while a documented allowlisted expression is accepted.

## Limitations and maintenance

The scanner is a regression tripwire, not a substitute for semantic review. It scans production code before each file's `#[cfg(test)]` section and excludes test-only cleanup. Aliases, macro expansion, generated code, and semantically equivalent new patterns may evade a regex-based scanner. Reviewers must therefore apply the governing rule directly: an accepted fallback may only reduce capability or optional detail; it must never increase authority, disclose sensitive data, activate an unverified artifact, or report a protected side effect as successful.

No fallback-audit completion or release-readiness claim is valid until permanent CI passes on the exact final repository SHA and the authoritative TODO is reconciled.