# CHANGELOG

## 2026-03-26 - Keyring-backed remote API keys

- Added OS keyring-backed storage for UI-entered remote API keys. When saved via the Settings UI or Tauri commands, secrets are stored in the OS keyring and `config.toml` is updated to a `from_keyring` reference.
- Runtime continues to support `from_env`, `from_file`, and `inline` secret references; no automatic bulk migration is performed.
- To migrate an inline API key: open Settings → Remote profile, enter the API key into the masked API key field, and click Save. This will store the secret in the OS keyring and update the config reference.
- CI and unit tests use an in-memory test keyring for determinism; production uses the platform keyring.
- Related commits: d3ccdd5 (feature), cc100b3 (docs)
