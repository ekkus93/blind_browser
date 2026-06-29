# Responses: BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING2

Spec: `docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING2_SPEC.md`
TODO: `docs/BLIND_BROWSER_SECURITY_SILENT_FAILURE_HARDENING2_TODO.md`

Fill in each `A:` line, then share the file or paste the answers back.

---

1. Q: **`tempfile` dev-dependency (P0-3)**: The suggested tests for P0-3 use
   `tempfile::tempdir()`, which is not in `Cargo.toml` (there is no
   `[dev-dependencies]` section at all). Should I add `tempfile = "3"` under
   `[dev-dependencies]`? Or would you prefer tests written using
   `std::env::temp_dir()` with manual cleanup (no new dep, but more verbose)?

   A:

---

2. Q: **P2-1 DTO approach**: To distinguish "secret missing" from "secret
   unreadable" in settings, the two options are:
   - **Flat error field** (smaller diff, backward-compatible shape): add
     `api_key_reference_error: Option<String>` (Rust) /
     `apiKeyReferenceError: string | null` (TypeScript) alongside the existing
     `api_key_masked_value`/`apiKeyMaskedValue` on the three existing DTOs in
     `src-tauri/src/commands/contracts/providers.rs` and `src/panel-types.ts`.
   - **`SecretReferenceStatus` struct** (cleaner semantics): replace the
     existing `api_key_masked_value: Option<String>` field with a struct that
     carries both the label and an error — a breaking change to the TS type
     shape.

   Which approach do you prefer?

   A:

---

3. Q: **Guardrail script style (P2-2)**: The existing `check-silent-fallbacks.sh`
   uses a `declare -A` associative array to map patterns to scan roots. The
   TODO suggests adding new patterns using individual `if grep` blocks — a
   different style. Three options:
   - **Extend `declare -A` style** — add new entries to the existing array
     (one consistent style throughout).
   - **Add `if grep` blocks after the loop** — two styles in one file, but
     avoids touching working code.
   - **Refactor all checks to `if grep` style** — consistent and slightly more
     readable per-entry, but a small non-functional change to working code.

   Which do you prefer?

   A:

---

4. Q: **`SecretRefError::NotConfigured` ghost (P0-1)**: The spec's suggested
   code for `resolve_optional_remote_planner_api_key` includes a match arm for
   `Err(SecretRefError::NotConfigured) => Ok(None)`. This type does not exist —
   `resolve_secret_ref` returns `Result<String, String>`, not a typed error enum,
   and `SecretRef` (the `api_key` field type) is never `Option<SecretRef>` so
   there is no "not configured" sentinel value. I plan to implement the simpler
   two-arm form:
   ```rust
   match resolve_secret_ref(&profile.api_key) {
       Ok(value) => Ok(Some(value)),
       Err(reason) => Err(format!(
           "Remote planner model list could not read the configured API key: {reason}"
       )),
   }
   ```
   Is this the correct interpretation, or do you want a typed `SecretRefError`
   added to `keyring_store.rs` first?

   A:

---

5. Q: **`url` crate return-value normalization (P1-2)**: The current
   `normalize_browser_navigation_url` returns the raw trimmed input string
   unchanged. After switching to the `url` crate, `parsed.to_string()` may
   differ from the raw input — for example, the `url` crate adds a trailing `/`
   for bare-host URLs (`https://example.com` → `https://example.com/`), and
   may normalize percent-encoding. This will break existing test assertions that
   compare exact strings (e.g., `assert_eq!(result, "http://localhost:3000")`).
   Should I update those tests to match the `url` crate's normalized output, or
   return `trimmed.to_string()` after validation (skip normalization, keep raw
   input)?

   A:

---

6. Q: **Wrong file path in P2-1 acceptance check**: The acceptance `rg` command
   in the TODO points at `src-tauri/src/commands/settings_adapters.rs`, which
   does not exist. The actual file is
   `src-tauri/src/app_core/settings_adapters.rs`. I plan to silently correct
   this in the acceptance check when implementing. Is that fine, or do you want
   it flagged separately?

   A:

---

7. Q: **Frontend rendering for P2-1**: Adding the error field to the Rust DTOs
   and TypeScript panel types is clear. The spec also mentions updating
   "settings panel renderers for remote planner/ASR/TTS key references" to
   display the error as a warning near the key reference. Should I implement the
   frontend warning rendering as part of P2-1, or is surfacing the field in the
   TS types (without wiring it to UI) enough for now?

   A:
