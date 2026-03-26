# CHANGELOG

## 2026-03-26 - Keyring-backed remote API keys

- Added OS keyring-backed storage for UI-entered remote API keys. When saved via the Settings UI or Tauri commands, secrets are stored in the OS keyring and `config.toml` is updated to a `from_keyring` reference.
- Runtime supports `from_env`, `from_file`, and `from_keyring` secret references.
- Saving a remote API key through the Settings UI stores it in the OS keyring instead of writing plaintext to `config.toml`.
- CI and unit tests use an in-memory test keyring for determinism; production uses the platform keyring.
- Related commits: d3ccdd5 (feature), cc100b3 (docs)
