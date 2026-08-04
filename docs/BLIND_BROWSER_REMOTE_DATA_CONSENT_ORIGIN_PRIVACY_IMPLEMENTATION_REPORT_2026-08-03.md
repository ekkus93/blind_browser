# Blind Browser Remote Data Consent and Origin Privacy Implementation Report

**Report date:** 2026-08-04  
**Repository:** `ekkus93/blind_browser`  
**Branch:** `master`  
**Status:** Stage 1 foundation complete; full milestone remains in progress.

## Bounded conclusion

Stage 1 establishes the versioned configuration, migration, origin-rule validation, deterministic privacy evaluator, and pre-serialization planner enforcement. It does not yet implement the complete just-in-time consent transaction or frontend experience.

## Baseline and exact evidence

- Starting SHA: `043c788346cc9b39942f4198f11c199aaea69ddf`
- Starting permanent CI: run `30886133291`, job `91917696317`, `success`
- Primary foundation implementation: `e6210ae423fb0c5fe08cea9ddc68f463e96b823f`
- Strict-Clippy repair: `14216226b223c092e1a4ff5da5b29c8129f67527`
- Fixture migration repair: `158672218048f4482879232d7ffc0ea779e9bd07`
- Final trigger-free implementation SHA: `ee967b2fb0d23a762bb8316f369d72c987f31df6`
- Guarded fixture repair: run `30900135542`, job `91962264055`, `success`
- Permanent validation trigger: run `30927205924`, job `92052482518`, `success`
- Exact trigger-free permanent CI: run `30928002322`, job `92055223608`, `success`

## Implemented architecture

### Versioned privacy model

`RemotePlannerPrivacySettings` now uses authoritative `network_mode`, `origin_rules`, `policy_schema_version`, and `migration_notice_pending` fields. Legacy consent, local-only, and blocked-origin fields remain readable only across the migration boundary and are synchronized from the new model after normalization. The new-install default is `AskPerOrigin`.

### Persistent decisions

Persistent blocks are origin-wide and contain no endpoint scope. Persistent allows require the exact normalized planner endpoint and current policy version. Rules are normalized with the URL library, sorted and deduplicated deterministically, and bounded to 256 entries.

### Legacy migration

Schema-zero settings map legacy local-only and consent booleans into the new global mode. Legacy blocked origins become origin-wide blocks. Migration is idempotent, sets a bounded notice, and does not manufacture destination-specific allows from broad legacy consent.

### Pure evaluator

The evaluator applies fail-closed precedence:

1. loopback local service;
2. local-only mode;
3. missing or opaque origin;
4. high-risk context;
5. persistent origin block;
6. exact unexpired session grant;
7. exact current-version persistent allow;
8. broad sanitized non-high-risk allow;
9. explicit consent required.

No default or fallback branch authorizes transmission.

### Planner integration

`sanitize_remote_planner_input` runs deterministic privacy evaluation before constructing remote planner input. Privacy settings are also part of the relevant runtime configuration fingerprint used by planning-state validation.

## Changed files

The implementation diff from the planning baseline through the trigger-free implementation contains only:

- `config.example.toml`
- `src-tauri/src/app_core/mod.rs`
- `src-tauri/src/app_core/planner_redaction.rs`
- `src-tauri/src/app_core/planning_snapshot.rs`
- `src-tauri/src/app_core/remote_data_consent.rs`
- `src-tauri/src/app_core/runtime_config.rs`
- `src-tauri/src/config/persistence.rs`
- `src-tauri/src/config/types.rs`
- `src-tauri/src/config/validation.rs`

No permanent workflow or diagnostic helper was added.

## Failed intermediate evidence and repairs

The first Stage 1 validation wrapper was invalidated because it captured `$?` after a shell `if` compound command, allowing a failed validation command to appear successful. Permanent CI—not the wrapper—remained authoritative.

Permanent run `30897812057`, job `91954789324`, then failed strict Clippy on an unnecessary cloned test slice. Commit `14216226b223c092e1a4ff5da5b29c8129f67527` corrected it.

Permanent run `30898762353`, job `91957861759`, passed Clippy and exposed eight stale planner-redaction fixtures. A count-guarded migration updated them to the versioned fields and canonical `remote_data_*` error codes. The repair workflow passed in run `30900135542`, job `91962264055`, and published `158672218048f4482879232d7ffc0ea779e9bd07`.

## Validation

The exact trigger-free implementation passed fallback and sensitive-diagnostic scanners, Rust formatting, default compilation, strict all-target/all-feature Clippy, focused direct-command semantic evidence, all-feature Rust tests, frontend lint, UI tests, and frontend production build in run `30928002322`, job `92055223608`.

## Still open

The following are not claimed complete:

- runtime once/session grant storage;
- prepared-request-only sender API;
- disclosure manifest and digest;
- pending challenge state;
- `NeedsRemoteDataConsent`;
- response command and exact resume;
- complete invalidation;
- status/settings APIs;
- TypeScript contracts;
- accessible consent dialog;
- structured rule management;
- request-count, replay, concurrency, accessibility, and adversarial tests;
- full privacy documentation and final BBCR reconciliation.

## Recommended next stage

Implement the backend consent transaction boundary in this order:

1. runtime grants;
2. `PreparedRemotePlannerRequest`;
3. disclosure manifest and digest;
4. bounded pending challenge;
5. `NeedsRemoteDataConsent`;
6. consent-response command with replay prevention, persistence safety, and unlocked exact network resume.

Frontend consent UX should begin only after those backend contracts and network request-count tests are stable.

## Bounded statement

Stage 1 is complete and green. The full remote-data consent/origin-privacy milestone and broader BBCR program remain open.
