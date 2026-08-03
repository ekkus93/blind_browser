# Blind Browser Accepted Fallback Inventory

**Date:** 2026-08-02  
**Scope:** Production fallback expressions in security-sensitive Rust and frontend paths  
**Machine-readable allowlist:** `scripts/security-fallback-allowlist.txt`  
**Enforcement:** `scripts/check-security-fallbacks.py`

## Rules

An allowlisted expression is not automatically safe. It is accepted only because its failure mode cannot authorize a stronger action, leak a secret, report false success for a protected side effect, or replace a verified artifact. New expressions fail CI until reviewed. Security-sensitive cleanup that fails is either returned as an error or explicitly classified as best-effort cleanup.

## Accepted categories

### Optional display and scoring text

`unwrap_or_default()` calls in `element_scoring.rs` and the destructive-click/sensitive-label builders in `click_authorization.rs` convert absent optional labels, text, placeholders, hrefs, or values into empty strings for deterministic comparison. They do not execute an action, weaken confirmation, or convert an error into success. Existing policy tests cover destructive-click classification and click authorization still fails closed when the target itself cannot be resolved.

### Optional URL-derived summaries

`form_destination` and `normalized_origin` return `None` when a page or form URL cannot be parsed. Post-Batch-8 confirmation code now binds an explicit degraded-summary warning into the confirmation manifest whenever destination information is unavailable. Failure therefore produces a more cautious prompt rather than authorization.

### Optional runtime presentation state

The confirmation-response prompt lookup defaults to an empty display string only after confirmation ID and digest matching; resume validation remains authoritative and fail-closed. Skill descriptions and parser display fields may default to empty text because they cannot grant tools or bypass deterministic action policy.

### Bounded numeric parsing

Checked conversions and parse attempts in audio routing, ASR WAV sizing, click confidence decoding, and direct command parsing return `None` or trigger the existing validation branch. Invalid protected arguments remain confirmation-required or are rejected.

### Diagnostic serialization

`serde_json::to_value(...).ok()` is used only to attach optional policy-decision details to an already constructed failure. Serialization failure removes diagnostics; it does not change the error code, retryability, action classification, or refusal.

### Capability discovery

Current-directory, app-config-directory, user-skill, and model-plan discovery can omit an unavailable optional source. These paths do not enable a stronger action. They are retained for compatibility but should eventually emit a user-visible capability warning. They are specifically tracked because capability loss must not become invisible expansion of authority.

### Best-effort cleanup

Temporary config and image-cache file removal may be best effort after a primary failure or cache eviction. Verified model-download `.part` cleanup is not in this category: cleanup failure is returned as `ModelDownloadError::CleanupFailed`. Test-only cleanup is excluded from the production scanner.

### Feature-disabled stubs

Remote TTS parameters are explicitly consumed in the feature-disabled implementation before returning the typed feature-unavailable error. No operation or fallback succeeds.

### URL sanitization setters

`Url::set_username` and `Url::set_password` return results that are ignored only after a valid URL is parsed. The sanitized URL is subsequently stripped of query and fragment data. Regression tests verify credentials are absent from remote planner and diagnostic payloads.

## Converted fallbacks in this pass

- API-key persistence no longer returns an empty reference through `unwrap_or_default()`; it returns `persisted_api_key_reference_missing`.
- Model availability directory iteration no longer silently skips unreadable entries.
- Verified model download cleanup failure is no longer ignored.
- Missing form/field metadata now produces digest-bound confirmation warnings.
- External URL validation no longer relies on a string prefix.

## Scanner limitations and maintenance

The scanner is a regression tripwire, not a substitute for review. It examines production code before each file’s `#[cfg(test)]` section and ignores test-only cleanup. The allowlist uses exact normalized source lines, so refactoring an accepted fallback requires renewed review. The broader code-review rule remains: an accepted fallback may only reduce capability or diagnostic detail; it must never increase authority or report a protected side effect as successful.
