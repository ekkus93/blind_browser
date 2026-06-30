---
description: Run ESLint and all UI/Rust tests for the blind_browser project. Use only when the user explicitly invokes this skill.
model: haiku
effort: low
disable-model-invocation: true
allowed-tools:
  - Bash(source ./fix-node-version.sh*)
  - Bash(pnpm lint*)
  - Bash(pnpm test:ui*)
  - Bash(cargo test*)
---

# Lint and Test

Run the full lint and test suite for the blind_browser project.

## Steps

Run these commands in order:

```bash
source ./fix-node-version.sh && pnpm lint
```

```bash
pnpm test:ui
```

```bash
cargo test --manifest-path src-tauri/Cargo.toml --all-features
```

## Reporting

Report a one-line summary per command: `PASS` or `FAIL`, with the test count for test steps and any error output for failures.

If any command fails, print the relevant error output and stop — do not continue to the next command.
