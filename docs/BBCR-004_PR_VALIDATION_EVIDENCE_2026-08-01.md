# BBCR-004 Pull Request Validation Evidence

**Date:** 2026-08-01
**Pull request:** `#4`
**Branch:** `agent/bbcr-004-origin-bound-credentials`
**State:** Draft and unmerged

## Proven implementation evidence

- Cleaned BBCR-004 implementation commit: `4d38b71363a83dc343dfd555a9e3a353ed6801b1`
- Successful bounded finalizer run: `30722003167`
- Successful bounded finalizer job: `91427197740`
- Master-synchronization merge commit: `3f91c2d716f83adfdc807ea3fa0eb7ad1da63296`
- Successful bounded synchronization run: `30722995411`
- Successful bounded synchronization job: `91429691369`
- Synchronized master commit: `7c3c4e49ee9a6c99fbf958ef17c92e4b1f9f5369`
- Prior owner-authored exact-head validation: commit `27dda7c43f2015cc33c051120dd1e721cc49c0b0`, run `30723051745`, job `91429862875` — success
- Clean TODO-closure implementation commit: `3fa91ef970f5836318e72d9e54b0436ad120c245`
- Successful self-cleaning closure run: `30723663277`

The master-synchronization commit retained the validated BBCR-004 product tree, incorporated the complete BBCR-002 post-merge evidence from `master`, removed the one-shot synchronization workflow, and left the branch ahead of `master` with no missing base commits.

The TODO-closure commit registered independent same-origin redirect-refusal coverage, documented legacy-keyring cleanup, marked the authoritative BBCR-004 checklist from implementation evidence, and removed its temporary workflow and transformation script before pushing.

## Legacy keyring migration and cleanup policy

Legacy unbound keyring accounts are detected before their secret is read. The application does not guess a destination, copy the old value, or automatically delete the legacy account. The user must re-enter the credential after reviewing and saving the destination; this writes a new account whose identity includes provider kind, profile name, and the normalized endpoint-scope digest.

After the newly scoped credential has been saved and tested successfully, the obsolete legacy account may be removed manually through the operating system's credential manager. Automatic deletion is intentionally avoided because the application cannot prove that an old account is unused by another installed version, backup, or recovery workflow.

## Redirect policy

Every credential-bearing HTTP client uses `Policy::none()`. Redirects are rejected even when the target remains on the same origin, and a separate cross-origin regression verifies that the destination server receives no follow-up connection. This fail-closed rule is simpler and stronger than attempting to selectively preserve authorization across redirects.

## Exact-head validation contract

The commit containing this document is the final owner-authored PR head that must pass the permanent `.github/workflows/ci.yml` workflow before any merge-readiness claim. The resulting run and job identifiers are recorded in PR #4 and issue #5 rather than by mutating this exact validated head.

The permanent gate must pass:

```text
bash scripts/check-silent-fallbacks.sh
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
pnpm lint
pnpm test:ui
pnpm build
```

No BBCR-004 merge, release-readiness, comprehensive-TODO completion, or full security-signoff claim is made by this document.
