# BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING2_REPLIES.md

## 1. `tempfile` dev-dependency for P0-3

A: Add `tempfile = "3"` under `[dev-dependencies]`.

Use `tempfile::tempdir()` for these tests. It is the right tradeoff here: less brittle than `std::env::temp_dir()` with manual cleanup, easier to read, and already a common Rust testing dependency. The hardening tests are specifically about filesystem safety, partial files, and atomic writes, so reliable temporary directories are worth the small dev-only dependency.

Suggested `Cargo.toml` addition:

```toml
[dev-dependencies]
tempfile = "3"
```

If `[dev-dependencies]` already gets added by another task before you touch it, append `tempfile = "3"` there instead of creating a duplicate section.

---

## 2. P2-1 DTO approach for secret reference errors

A: Use the **flat error field** approach for this pass.

Add fields alongside the existing masked-value/reference fields:

Rust DTO shape, using the existing DTO names:

```rust
pub api_key_reference_error: Option<String>,
```

TypeScript shape:

```ts
apiKeyReferenceError: string | null;
```

Do this for the remote planner, remote ASR, and remote TTS DTOs/panel states where masked API-key reference data is already surfaced.

Reasoning:

- This is a hardening follow-up, not a DTO redesign.
- It is a smaller diff.
- It avoids breaking every existing consumer of `api_key_masked_value` / `apiKeyMaskedValue`.
- It still gives the UI enough information to distinguish “no key configured” from “configured key could not be inspected.”

Do **not** replace the existing `api_key_masked_value` field with a new `SecretReferenceStatus` struct in this pass.

A future cleanup can unify secret reference presentation into a proper struct once the safety behavior is already correct and tested.

---

## 3. Guardrail script style for P2-2

A: Extend the existing `declare -A` associative-array style.

Keep the script internally consistent. Do not refactor the whole script just for style, and do not add separate `if grep` blocks after the loop unless the current array style cannot express the needed check.

Preferred approach:

```bash
declare -A CHECKS=(
  # existing entries...

  ["src-tauri/src/app_core/runtime_config.rs"]='resolve_secret_ref\(&profile\.api_key\)\.ok\(\)'
  ["src-tauri/src/app_core/settings_adapters.rs"]='masked_secret_value.*\.ok\(\)\?'
  ["src-tauri/src/asr/remote.rs"]='unwrap_or_default\(\)\.to_string\(\)'
)
```

If the script currently maps patterns to roots rather than roots to patterns, follow the existing direction exactly. The important part is consistency and narrowness.

Do **not** ban all `.ok()`, all `unwrap_or_default()`, or all `fs::write()` globally. Only guard against the exact silent-failure shapes fixed in this pass.

Also include a negative test or documented manual check showing that the script fails when one of the forbidden exact patterns is reintroduced.

---

## 4. `SecretRefError::NotConfigured` does not exist

A: Use the simpler two-arm form. Do **not** add a new typed `SecretRefError` just for this pass.

Your interpretation is correct. Since `resolve_secret_ref` returns `Result<String, String>` and `RemotePlannerProfile.api_key` is not `Option<SecretRef>`, there is no “not configured” sentinel to preserve here.

Implement:

```rust
match resolve_secret_ref(&profile.api_key) {
    Ok(value) => Ok(Some(value)),
    Err(reason) => Err(format!(
        "Remote planner model list could not read the configured API key: {reason}"
    )),
}
```

Keep override behavior ahead of this:

```rust
if let Some(override_value) = api_key_override.map(str::trim) {
    if !override_value.is_empty() {
        return Ok(Some(override_value.to_string()));
    }
}
```

Do not create a typed secret error enum unless the implementation naturally requires it for broader reasons. The goal is to remove the silent `.ok()` fallback, not to redesign keyring error handling.

---

## 5. `url` crate normalization behavior

A: Return the `url` crate’s normalized output and update tests accordingly.

Use `parsed.to_string()` after validation.

That means tests should expect normalized URLs such as:

```rust
assert_eq!(
    normalize_browser_navigation_url("https://example.com").unwrap(),
    "https://example.com/"
);
```

and:

```rust
assert_eq!(
    normalize_browser_navigation_url("http://localhost:3000").unwrap(),
    "http://localhost:3000/"
);
```

Reasoning:

- The function is named `normalize_browser_navigation_url`; normalization is acceptable and desirable.
- Returning parser-normalized output avoids carrying odd but technically valid formatting forward.
- Tests should assert the policy’s behavior, not preserve pre-parser string formatting.
- The browser will effectively normalize many URLs anyway; doing it explicitly makes state and tests more predictable.

Do still reject whitespace/control characters before parsing, as specified. Do not accept a raw input merely because a downstream browser might tolerate or reinterpret it.

---

## 6. Wrong file path in P2-1 acceptance check

A: Silently correct the path in the implementation and update the TODO/docs if they are tracked in the repo.

Use the actual file:

```text
src-tauri/src/app_core/settings_adapters.rs
```

The TODO path was simply wrong. You do not need to create a separate issue for that.

If the TODO file is checked into the repo, fix the acceptance command there too:

```bash
rg -n "masked_secret_value.*\.ok\(\)\?" src-tauri/src/app_core/settings_adapters.rs
```

---

## 7. Frontend rendering for P2-1

A: Implement the frontend warning rendering as part of P2-1. Surfacing the field in TypeScript types without rendering it is **not enough**.

The whole point of P2-1 is that users should be able to distinguish:

- no secret configured,
- secret configured and readable/maskable,
- secret configured but unreadable/uninspectable.

A hidden DTO field does not solve the user-visible silent-failure problem.

Add warning rendering near the relevant API-key/reference UI in:

- remote planner settings,
- remote ASR settings,
- remote TTS settings.

Use existing warning/error styling if available. If no suitable shared warning class exists, add one small reusable class.

Suggested TS/React shape:

```tsx
{state.apiKeyReferenceError ? (
  <p className="settings-warning" role="alert">
    {state.apiKeyReferenceError}
  </p>
) : null}
```

Suggested CSS if needed:

```css
.settings-warning {
  margin: 8px 0 0;
  padding: 10px 12px;
  border-radius: 12px;
  border: 1px solid color-mix(in srgb, var(--color-amber-primary) 32%, transparent);
  background: var(--color-amber-light);
  color: var(--color-amber-active);
  line-height: 1.45;
  font-weight: 600;
}

.settings-warning:focus-visible {
  outline: var(--focus-ring);
  outline-offset: var(--focus-offset);
}
```

If `color-mix()` is not acceptable for the target webview, use an existing tokenized border style instead.

Add/adjust UI tests to verify that an `apiKeyReferenceError` appears in rendered settings HTML.

Do not block P2-1 on a larger secret-reference component refactor.
