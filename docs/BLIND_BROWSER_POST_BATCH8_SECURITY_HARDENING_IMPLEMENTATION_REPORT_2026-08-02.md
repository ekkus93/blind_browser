# Blind Browser Post-Batch-8 Security Hardening Implementation Report

**Date:** 2026-08-02  
**Repository:** `ekkus93/blind_browser`  
**Authoritative TODO:** `docs/BLIND_BROWSER_POST_BATCH8_SECURITY_HARDENING_TODO_2026-08-02.md`  
**Starting `master` SHA:** `00d2e5c5cf5cf42f26dd73872fe58ddc7420ea6a`  
**Working branch:** `post-batch8-hardening-continuation`  
**Draft pull request:** `#12`  
**Pre-report implementation/documentation snapshot:** `7d3e6abd064acac45d6105e5428061b755151eac`  
**Status:** Implementation complete for the bounded source scope; exact final-head and final-`master` permanent CI evidence must still be appended before the authoritative TODO can be closed.

## Scope and corrected completion boundary

This pass completes the residual post-Batch-8 work identified after permanent CI first became green on `00d2e5c5cf5cf42f26dd73872fe58ddc7420ea6a`.

It does **not** declare the repository generally release-ready or the entire comprehensive review complete. The exact remaining BBCR P1/P2/P3 program is recorded in `docs/BLIND_BROWSER_POST_BATCH8_RECONCILIATION_2026-08-02.md`.

## Implemented workstreams

### P8-001 — Model-download integrity reconciliation

The underlying verified-download implementation was already present at the starting SHA. This pass re-audited and reconciled it rather than replacing it.

Confirmed implementation:

- immutable code-pinned Kitten TTS and Whisper ASR file manifests;
- SHA-256, minimum size, and configured maximum size for known files;
- unknown model IDs fail closed;
- explicit connect and request timeouts;
- redirect refusal followed by an explicit HTTPS Hugging Face/CDN host allowlist;
- streaming byte ceilings;
- `.part`-only writes;
- hash/size verification before activation;
- file sync before atomic replacement;
- old-target preservation and partial cleanup on failure;
- typed redirect, timeout, status, size, hash, sync, replacement, and cleanup errors.

Additional evidence added here:

- a regression proving model-download errors expose the manifest file identity without exposing unrelated private local parent paths.

The broader BBCR-007 provenance/update UX and directory-level multi-file transaction program remains open as documented in the reconciliation addendum.

### P8-002 — Direct-command policy and concrete defects

Confirmed the exhaustive `DirectCommandName`/`DirectCommandPolicy` registry and handler-surface parity test.

Implemented:

- strict parsed external HTTPS URL validation;
- required host;
- control-character rejection;
- malformed URL rejection;
- username/password rejection;
- query-string and fragment rejection;
- normalized URL passed to the OS launcher;
- focused URL validation tests;
- explicit planner/TTS/ASR post-persistence API-key-reference invariant errors;
- tests for missing, empty, whitespace-only, and normalized non-empty key references.

Policy evidence remains documented in `docs/BLIND_BROWSER_DIRECT_COMMAND_POLICY_2026-08-02.md`.

### P8-003 — Fail-closed confirmation summaries

Implemented structured `ConfirmationSummaryWarningCode` values for:

- page model unavailable;
- form label unavailable;
- form identity ambiguous;
- destination unavailable;
- field inventory unavailable;
- sensitive/hidden fields may be omitted;
- target label unavailable.

The warning vector is serialized inside each confirmation action manifest and therefore changes the canonical manifest digest.

Behavior implemented and tested:

- full submit metadata includes safe form label, destination, and safe field labels;
- missing page metadata produces an explicit form/destination/inventory warning;
- ambiguous form identity is explicit;
- unknown destination is explicit;
- sensitive/hidden-field omission is explicit;
- typed text is represented only by character count;
- missing type target label is explicit;
- raw typed text is absent from prompt and manifest JSON;
- planner-authored confirmation copy remains ignored;
- tests now use the exact production `__runtime_*` annotation keys.

Missing metadata never reduces the deterministic confirmation requirement.

### P8-004 — Accepted fallback inventory and scanner

Reconciled `docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md` into a per-category evidence table covering source/function category, expression class, justification, side-effect impact, user visibility, tests, and replacement plan.

Confirmed permanent machine enforcement:

- `scripts/security-fallback-allowlist.txt`;
- `scripts/check-security-fallbacks.py`;
- synthetic scanner self-test;
- permanent CI invocation.

Removed from the accepted-fallback category:

- API-key-reference defaults;
- prefix-only external URL validation;
- implicit confirmation degradation;
- verified model cleanup failure suppression;
- raw frontend caught-error logging;
- full skill-path diagnostics.

### P8-005 — Diagnostics and privacy

Created `docs/BLIND_BROWSER_POST_BATCH8_DIAGNOSTIC_PRIVACY_AUDIT_2026-08-02.md`.

Backend changes/evidence:

- remote planner request failures now retain only provider, model, and normalized base URL;
- regression verifies no response body, authorization data, or token-shaped content is serialized;
- remote ASR parse failure regression verifies hostile provider fields are not echoed;
- remote TTS status/transport paths were audited and retain no provider response body;
- skill-loading diagnostics now expose only source class, leaf skill directory, and error kind;
- model-download error regression excludes private target parent paths;
- API-key result structures are protected from accidental sensitive `Debug` derivation;
- centralized redacting `ToolError` serialization remains authoritative.

Frontend changes/evidence:

- JWT and common provider-token detection;
- recursive sensitive-key redaction;
- URL credentials/query/fragment stripping;
- nested response-body redaction;
- generic `Error.message` redaction before classification;
- raw confirmation-submission console error replaced by classified output;
- external-link logs and UI guidance use sanitized URLs and classified failures;
- regression tests cover token shapes, JWTs, nested objects, response bodies, URLs, serialized tool errors, generic errors, and alerts.

Scanner improvements:

- multiline Rust and TypeScript diagnostic parsing;
- sensitive planner/page/OCR/transcript/credential/response/tool-argument references;
- raw frontend error-object logging;
- named sensitive `Debug` derives;
- centralized serializer/redactor presence;
- hostile and benign `--self-test` fixtures;
- permanent CI runs the self-test and repository scan.

### P8-006 — Hostile DOM/OCR corpus

Added deterministic corpus data at:

- `src-tauri/tests/fixtures/post_batch8_hostile_content_corpus.json`

Added completeness/invariant tests at:

- `src-tauri/tests/hostile_content_corpus_manifest.rs`

The corpus explicitly enumerates:

- hidden input injection;
- off-screen CSS text;
- malicious ARIA labels;
- malicious title/alt text;
- malicious `data-*` content;
- script/style/comment text;
- invisible overlay text;
- malicious form labels;
- confirmation bypass near buttons;
- credential-exfiltration instructions near inputs;
- OCR instruction override;
- OCR authority impersonation;
- OCR confirmation bypass;
- OCR credential exfiltration;
- OCR high-risk payment/receipt context;
- mixed benign and hostile OCR regions;
- OCR attempts to authorize click/submit.

The manifest enforces:

- caution-only telemetry;
- no lowering confirmation;
- no creation of click authorization;
- no marking destructive clicks safe;
- no high-risk-origin bypass;
- no entry into trusted runtime prompt sections;
- only monotonic effects: increase caution, redact, block, require confirmation, abort, or replan;
- continued presence of the real `hostile_prompt_injection.png` fixture.

Existing planner-redaction/action-policy tests provide behavioral enforcement behind the corpus.

### P8-007 — Reconciliation

Created `docs/BLIND_BROWSER_POST_BATCH8_RECONCILIATION_2026-08-02.md`.

It records a distinct status for every `BBCR-001` through `BBCR-021` item and states exactly which residual gaps this pass closes. It also retains the still-open comprehensive work, notably remote-data consent, broader filesystem/persistence/resource limits, Redux secret-draft redesign, CSP, secret scanning, dependency/SAST, cross-platform packaged CI, fuzzing/mutation, and primary architecture/security documentation.

## Changed-file inventory at pre-report snapshot

| File | Purpose |
|---|---|
| `.github/workflows/ci.yml` | Run diagnostic scanner self-test before repository audit. |
| `docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md` | Reconciled accepted fallback inventory. |
| `docs/BLIND_BROWSER_POST_BATCH8_DIAGNOSTIC_PRIVACY_AUDIT_2026-08-02.md` | Backend/frontend privacy audit and residual risks. |
| `docs/BLIND_BROWSER_POST_BATCH8_RECONCILIATION_2026-08-02.md` | BBCR-001 through BBCR-021 status reconciliation. |
| `scripts/check-sensitive-diagnostics.py` | Multiline, raw-error, sensitive-name, and `Debug` scanner plus self-test. |
| `src-tauri/src/app_core/model_management/tests.rs` | Model error local-path privacy regression. |
| `src-tauri/src/app_core/remote_planner.rs` | Safe planner request-failure helper and regression. |
| `src-tauri/src/asr/remote.rs` | Remote ASR response-body omission regression. |
| `src-tauri/src/command_handlers/api_key_handlers.rs` | Planner/TTS/ASR key-reference invariants and tests. |
| `src-tauri/src/command_handlers/url_handlers.rs` | Strict external URL parsing/normalization and tests. |
| `src-tauri/src/commands/confirmation_manifest.rs` | Structured degraded-summary metadata, visible warnings, digest binding, tests. |
| `src-tauri/src/commands/skill_loader.rs` | Path-private diagnostics and regression. |
| `src-tauri/tests/fixtures/post_batch8_hostile_content_corpus.json` | Deterministic hostile DOM/OCR corpus. |
| `src-tauri/tests/hostile_content_corpus_manifest.rs` | Corpus completeness and monotonic-security invariant tests. |
| `src/external-link.test.mjs` | External-link UI/log redaction regressions. |
| `src/panel-state-setters.ts` | Sanitized external-link error display/logging. |
| `src/privacy-redaction.test.mjs` | Token/JWT/URL/nested/error redaction matrix. |
| `src/privacy-redaction.ts` | Expanded diagnostic redaction. |
| `src/voice-loop.ts` | Classified confirmation-submission console error. |

This report itself is an additional documentation file after the listed snapshot.

## Permanent validation gate

The permanent workflow runs:

```text
bash scripts/check-silent-fallbacks.sh
python3 scripts/check-security-fallbacks.py --self-test
python3 scripts/check-security-fallbacks.py
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

## Validation evidence to finalize

The following fields must be replaced after the report-containing head passes and after merge to `master`:

- **Final branch SHA:** pending
- **Permanent branch CI run:** pending
- **Permanent branch CI job:** pending
- **Branch result:** pending
- **Merged PR:** pending
- **Final `master` SHA:** pending
- **Permanent final-`master` CI run:** pending
- **Permanent final-`master` CI job:** pending
- **Final result:** pending

Because a Git commit cannot contain its own SHA or a workflow run that starts only after that commit exists, exact evidence for the report-publication commit and final `master` commit should also be recorded in PR #12 without mutating the already validated tree.

## Remaining risks

- The broader comprehensive BBCR remediation remains open as listed in the reconciliation document.
- Regex/static scanners are regression tripwires, not semantic proofs.
- Third-party libraries may emit diagnostics outside application wrappers; production logging must remain conservative.
- The current external URL launcher still relies on platform OS opener behavior after strict normalization; future launcher changes require command-injection review on every platform.
- Model files are verified individually; the broader directory-level transactional update/provenance workflow remains a separate BBCR-007/011 concern.
- The hostile corpus proves deterministic boundaries and fixture coverage; it does not prove that every remote model will ignore malicious content.
- Explicit remote page-data consent/per-origin policy remains outside this bounded pass.

## Provisional completion statement

After exact final-`master` permanent CI is green, the permitted statement is:

> The post-Batch-8 security-hardening TODO is complete for its bounded scope. The broader comprehensive remediation remains open, so this is not a general release-readiness or comprehensive-security-completion declaration.

Until the evidence fields above are finalized, even that bounded statement is provisional.