# Security/Silent-Failure Hardening 4 Closure

Hardening 4 is complete.

## Reconciliation result

The source-side fixes requested by Hardening 4 were already present on the green baseline rather than still awaiting implementation:

- `tempfile = "3"` is under `[dev-dependencies]` only.
- `replace_file_atomically` documents the actual durability contract and, on Unix, fsyncs the containing directory after rename. Directory-open/fsync failures are returned instead of silently degrading durability.

The remaining work was documentation and validation correctness. The closeout corrected stale `settings_adapters.rs` paths, replaced nonexistent `pnpm test` instructions with `pnpm test:ui`, and changed the config-write static check to inspect production code only so the intentional `#[cfg(test)]` fixture write is not misclassified as a production fallback.

## Validation evidence

The documentation/static-check reconciliation candidate was:

- SHA: `fc9afbb651adf9b7b175bc322997527accf3f902`
- permanent CI run: `31282795594`
- job: `93166691767`
- conclusion: **SUCCESS**

That exact run passed the silent-fallback/security/privacy scanners, rustfmt, default-feature cargo check, strict all-target/all-feature Clippy with `-D warnings`, focused direct-command semantic evidence, the complete Rust/Wry test suite, frontend lint, UI tests, and production build.

The final closeout documentation commit is still required to pass permanent CI on its own exact SHA before promotion to `master`; the successful reconciliation run above is not used to bypass that final exact-SHA gate.
