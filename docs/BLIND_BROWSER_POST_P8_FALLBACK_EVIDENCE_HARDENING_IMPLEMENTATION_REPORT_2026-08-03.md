# Blind Browser Post-P8 Fallback and Evidence Hardening Implementation Report

**Date:** 2026-08-03  
**Repository:** `ekkus93/blind_browser`  
**Branch:** `master`  
**Authoritative TODO:** `docs/BLIND_BROWSER_POST_P8_FALLBACK_EVIDENCE_HARDENING_TODO_2026-08-03.md`  
**Companion spec:** `docs/BLIND_BROWSER_POST_P8_FALLBACK_EVIDENCE_HARDENING_SPEC_2026-08-03.md`  
**Starting SHA:** `b333a0578a324fc7e1bde738ebee5a5257cdd581`  
**Published implementation SHA:** `7c72d4282db4952eb94f1d0152eb0c3f48cc6a88`  
**Cleaned implementation SHA:** `e04524f0184230d5564f5ecdb2a167d5fbd7c791`  
**Documentation/scanner/CI reconciliation candidate:** `64b8a0fa843a1bc2764f59f80787d5d28578a9c8`  
**Ralph validation:** run `30848401463`, job `91802215500` — success  
**Status:** Implementation and bounded Ralph validation complete. Permanent CI on the final TODO-closure SHA remains the authoritative final gate.  
**Release boundary:** This report closes only the post-P8 fallback/evidence hardening scope. It does not declare the full BBCR program complete or the repository production release-ready.

## 1. Summary

This pass converted the highest-priority accepted quiet fallbacks into explicit typed or diagnostic behavior and strengthened direct-command evidence from source-string-only checks toward semantic policy mappings.

The implementation:

- replaced quiet unreadable skill-entry skipping with bounded, path-private warning aggregation;
- added typed settings capability absence reasons for not configured, missing profile, invalid endpoint, and unknown model IDs;
- reconstructed display-safe URLs from approved origin/path components instead of ignoring username/password mutation results;
- added semantic mappings for direct-command network, credential, page-context, and verified-artifact policies;
- retained source-text tests only as supplemental drift detectors;
- replaced optional policy-detail serialization suppression with an explicit `detail_serialization: failed` marker;
- classified every remaining accepted fallback as permanent or temporary;
- removed thirteen converted expressions from the exact allowlist/inventory;
- added machine enforcement for disposition metadata, review boundaries, owner notes, human-readable counts, and temporary-entry documentation;
- added the focused direct-command semantic evidence test to permanent CI.

## 2. Publication and validation history

The source patch was generated, tested, and published directly to `master`; no branch, pull request, or worktree is part of the delivery.

- Source patch commit: `7c72d4282db4952eb94f1d0152eb0c3f48cc6a88`
- Successful bounded Ralph run: `30848401463`
- Successful Ralph job: `91802215500`
- Temporary workflow removal: `cfbbe1b89392eaca2dc1d50ee4995ffc61fd7190`
- Temporary generator removal / cleaned implementation: `e04524f0184230d5564f5ecdb2a167d5fbd7c791`
- Human/machine documentation parity and focused-CI candidate: `64b8a0fa843a1bc2764f59f80787d5d28578a9c8`

The successful Ralph job ran every fallback/diagnostic scanner, Rust formatting, `cargo check`, deny-warning Clippy, the full Rust test suite, frontend lint, UI tests, and the production frontend build before publishing the implementation commit.

## 3. Implemented workstreams

### 3.1 Accepted fallback disposition metadata

`scripts/security-fallback-inventory.json` is now version 2 and requires:

- `disposition`;
- `review_due`;
- `owner_note`;
- all prior path/function/expression, justification, visibility, side-effect, test, and replacement fields.

Current disposition counts:

- `permanent_accepted`: 21
- `temporary_accepted`: 5
- converted or removed in this pass: 13

All five temporary entries are due before the release-candidate gate and carry actionable replacement notes.

### 3.2 Skill loading

`src-tauri/src/commands/skill_loader.rs` no longer uses `filter_map(Result::ok)` for directory entries. It now:

- retains readable neighboring entries;
- counts skipped entries;
- aggregates bounded error categories;
- logs only source class, count, and error categories;
- does not include full paths;
- continues to log unreadable manifests, parse failures, and directory-name mismatches through existing path-private diagnostics.

Unreadable entries are absent from the candidate set and therefore cannot contribute tools or permissions.

### 3.3 Typed settings absence

A shared `CapabilityAbsenceReason` contract was added to Rust and TypeScript. Settings surfaces now distinguish:

- `not_configured`;
- `profile_missing`;
- `invalid_endpoint`;
- `unknown_model_id`;
- reserved explicit reasons for manifest, feature, credential-reference, and local-binary unavailability.

Invalid endpoint display uses sanitized origin/path output. Credentials, query strings, and fragments are not exposed. Actual provider/model operations retain their existing fail-closed behavior.

### 3.4 Semantic direct-command evidence

`src-tauri/src/direct_command_policy.rs` now exposes typed mappings for:

- network policy class;
- endpoint-bound credential requirements;
- remote-planner page-context sanitization;
- verified atomic model activation.

Runtime registry validation and unit tests require the semantic mappings to match `DirectCommandPolicy` flags. The integration test keeps handler/source wiring checks under explicit `source_drift_*` names so they are supplemental rather than the primary semantic proof.

Permanent CI now runs the direct-command integration test by itself before the full Rust suite.

### 3.5 URL sanitization

`src-tauri/src/diagnostic_redaction.rs` provides a shared URL reconstruction helper. It:

- parses the URL;
- requires a host and non-opaque origin;
- rebuilds output from origin and path only;
- never copies username, password, query, or fragment;
- returns generic redaction for malformed URL-like input.

Planner redaction and settings diagnostics use this helper. The prior ignored `set_username` and `set_password` results were removed from the accepted inventory.

### 3.6 Confirmation and scoring defaults

The pass retained conservative empty-text scoring fallbacks where absence only lowers information or confidence. Existing digest-bound confirmation warnings remain authoritative where missing labels affect user-facing protected-action summaries:

- target label unavailable;
- page model unavailable;
- form label/identity unavailable;
- destination unavailable;
- field inventory unavailable;
- sensitive fields may be omitted.

Missing metadata cannot create click authorization, lower confirmation, or mark destructive actions safe. The remaining exact scoring/default expressions are permanently accepted and CI-enforced.

### 3.7 Policy-detail serialization

Executor and validator policy refusal paths no longer silently drop supplemental JSON details through `.ok()`. Serialization failure produces:

```json
{ "detail_serialization": "failed" }
```

The primary refusal code remains visible, and execution remains blocked. Helper-level tests cover the explicit marker.

## 4. Scanner and CI changes

### `scripts/check-security-fallback-inventory.py`

The scanner now verifies:

- allowlist/inventory key equality;
- exact source expression and containing function;
- complete disposition metadata;
- valid disposition values;
- actionable temporary review boundaries and owner notes;
- human-readable permanent/temporary counts;
- presence of every temporary entry in the accepted-fallback document;
- absence of the deprecated duplicated Markdown exact-expression table.

Self-tests cover missing and invalid disposition, missing temporary review boundary, insufficient owner note, stale expression/function lookup, human-document count/entry mismatch, and deprecated table markers.

### Permanent CI

Permanent CI continues to run all scanners and validation commands and now separately executes:

```text
cargo test --manifest-path src-tauri/Cargo.toml --all-features --test post_batch8_direct_command_policy_evidence
```

This focused step is followed by the full Rust suite.

## 5. Changed-file inventory

The following implementation/reconciliation files differ from starting SHA `b333a0578a324fc7e1bde738ebee5a5257cdd581` through candidate `64b8a0fa843a1bc2764f59f80787d5d28578a9c8`:

- `.github/workflows/ci.yml`
- `docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md`
- `scripts/check-security-fallback-inventory.py`
- `scripts/security-fallback-allowlist.txt`
- `scripts/security-fallback-inventory.json`
- `src-tauri/src/app_core/mod.rs`
- `src-tauri/src/app_core/planner_redaction.rs`
- `src-tauri/src/app_core/settings_adapters.rs`
- `src-tauri/src/app_core/tests/settings_tests.rs`
- `src-tauri/src/commands/contracts/providers.rs`
- `src-tauri/src/commands/planner_executor/execution.rs`
- `src-tauri/src/commands/skill_loader.rs`
- `src-tauri/src/commands/validators/mod.rs`
- `src-tauri/src/diagnostic_redaction.rs`
- `src-tauri/src/direct_command_policy.rs`
- `src-tauri/tests/post_batch8_direct_command_policy_evidence.rs`
- `src/tauri-types.ts`

This implementation report and the authoritative TODO are additional final documentation files.

## 6. Tests added or modified

- URL reconstruction and malformed URL redaction tests.
- Planner URL sanitization tests for credentials, query, fragment, port, safe path, and malformed input.
- Skill-entry warning aggregation, adjacent-valid-entry, and path-privacy tests.
- Settings tests for invalid endpoint, sanitized endpoint display, not-configured, profile-missing, unknown Kitten model, and unknown Whisper model.
- Direct-command semantic mapping parity test.
- Renamed supplemental `source_drift_*` direct-command wiring tests.
- Executor and validator policy-detail serialization marker tests.
- Inventory scanner hostile self-tests for disposition and documentation parity.

The pre-existing confirmation/scoring test suite continues to cover missing labels, degraded warning metadata, digest binding, planner-authored text rejection, conservative destructive-action handling, and absence of raw typed values.

## 7. Removed and remaining fallbacks

Thirteen exact expressions were removed from the accepted inventory:

- two ignored planner URL userinfo mutation results;
- three settings/model lookup `.ok()` expressions;
- one settings voice `.unwrap_or_default()` expression;
- one skill directory `filter_map(Result::ok)` expression;
- one executor policy-detail serialization `.ok()` expression;
- five validator policy-detail serialization `.ok()` expressions.

Five temporary accepted fallbacks remain:

1. optional planner candidate discovery `.ok()`;
2. optional current-directory discovery `.ok()`;
3. optional fill-correction discovery `.ok()`;
4. optional field-focus query construction `.ok()?`;
5. optional skill-frontmatter text `.unwrap_or_default()`.

They reduce capability only, are exact-expression enforced, and must be reconsidered before the release-candidate gate.

## 8. Validation commands

The successful Ralph job executed:

```text
bash scripts/check-silent-fallbacks.sh
python3 scripts/check-security-fallbacks.py --self-test
python3 scripts/check-security-fallbacks.py
python3 scripts/check-security-fallback-inventory.py --self-test
python3 scripts/check-security-fallback-inventory.py
python3 scripts/check-sensitive-diagnostics.py --self-test
python3 scripts/check-sensitive-diagnostics.py
cargo fmt --manifest-path src-tauri/Cargo.toml --all
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
xvfb-run -a cargo test --manifest-path src-tauri/Cargo.toml --all-features
pnpm lint
pnpm test:ui
pnpm build
```

Permanent CI additionally runs formatting in check mode and the focused direct-command evidence test.

## 9. Remaining risks and out-of-scope work

This pass does not close the larger BBCR program. Remaining work includes, among other items:

- converting the five temporary accepted fallbacks before release-candidate review;
- full remote-data consent and per-origin/local-only product UX;
- model provenance/update UX and directory-level transactions;
- planner cancellation and bounded response bodies;
- config/keyring rollback and crash/concurrency durability;
- centralized resource budgets and stress testing;
- removal of raw API-key drafts from global frontend state;
- production CSP/frontend network-boundary proof;
- secret-history scanning and push protection;
- dependency/license/SAST gates;
- packaged Windows/macOS validation;
- fuzzing, property, and mutation testing;
- primary architecture, threat-model, privacy, and operations documentation.

## 10. Final evidence rule

A commit cannot embed its own SHA or the workflow run created after it is pushed. The authoritative TODO records the exact implementation, cleanup, and documentation-candidate evidence available before closure. The exact final TODO-closure SHA and its `ci/permanent` result are canonical GitHub commit metadata.

> The post-P8 fallback and evidence hardening implementation is complete for its bounded scope once permanent CI passes on the final TODO-closure SHA. This is not a general release-readiness declaration.
