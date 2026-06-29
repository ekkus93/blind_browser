# Blind Browser UI/UX Fix 6 Spec

## Purpose

Fix 6 is a narrow final cleanup pass after the Fix 5 review.

Fix 5 appears effectively complete from a code standpoint: the global external-link alert behavior is implemented and tested, the targeted CSS static audits pass, the planner verified-model-list state remains truthful, and the main dark-mode/focus issues are resolved.

Fix 6 should **not** be a broad redesign pass. Its purpose is to clean up the remaining small issues found in the Fix 5 review:

1. Make app-alert dismissal reset the whole alert state, not only the message.
2. Tokenize remaining hardcoded error/danger text colors.
3. Reconcile TODO/checklist status so documentation matches actual completion.
4. Re-run validation and record a final memory entry only after validation passes.

## Current known good behavior to preserve

Do not regress these behaviors:

- `openExternalLink()` routes failed external-link opens to the global app alert.
- Failed external-link alert text includes the failed URL and error detail.
- External-link failures do **not** route through `urlInputPanelState.error`.
- `clearAppAlert()` clears the alert message and resets the alert kind to `info`.
- Runtime refresh preserves a verified planner model list only when the endpoint still matches.
- Runtime refresh does not synthesize a model list from saved backend settings.
- `--color-surface-inner` has a concrete light-mode value.
- Dark-mode token cascade has the dark block after the base `:root`.
- `.url-action-button:focus-visible` uses `var(--focus-ring)` and `var(--focus-offset)`.
- The Fix 5 CSS static audits pass.

## Non-goals

Do not:

- Replace the global app alert with settings-only alert routing.
- Reintroduce `setUrlInputPanelState()` or `urlInputPanelState.error` for external-link failures.
- Rewrite planner model-list state management.
- Redesign dark mode.
- Add broad new UI features.
- Perform large refactors unrelated to the narrow cleanup tasks.

## Design constraints

### Alert state should be internally consistent

When the app alert is dismissed, both `message` and `kind` should return to the neutral state.

Preferred neutral state:

```ts
{
  kind: "info",
  message: null,
}
```

The UI already hides the alert when `message === null`, so leaving `kind: "error"` is not user-visible. But it is stale state and makes tests/debugging more confusing.

### Error colors should use existing semantic tokens

Hardcoded component-level error colors should use the error token family:

```css
var(--color-error-primary)
var(--color-error-active)
var(--color-error-dark)
var(--color-error-light)
```

Use:

- `var(--color-error-primary)` for ordinary error text.
- `var(--color-error-dark)` for stronger/high-severity error text.
- `var(--color-error-light)` for error backgrounds.
- `var(--color-error-active)` for hover/active danger states if needed.

Do not use raw component-level values like:

```css
color: #6b2820;
color: #7a2018;
color: #54100f;
```

unless there is an explicit nearby comment explaining why the raw color is intentionally mode-invariant.

### Documentation must reflect completion truthfully

If a TODO task is marked `DONE`, the final done checklist should also be checked for the corresponding item. If a checklist item cannot be verified, leave it unchecked and explain why.

Do not mark validation complete unless the validation gate actually passes in the developer environment.

## Expected files touched

Likely files:

- `src/app.tsx`
- `src/panel-state-setters.ts` if `clearAppAlert` needs export/import adjustment
- `src/styles.css`
- `BLIND_BROWSER_UIUX_FIX5_TODO(1).md` or the current working TODO/status file
- `memory.md`

Optional files:

- `src/external-link.test.mjs` if a dismiss test needs to be adjusted to match the `clearAppAlert` handler.
- Any UI render test that asserts app-alert dismiss behavior.

## Acceptance summary

Static checks:

```bash
rg -n "setUrlInputPanelStoreState\(|setUrlInputPanelState\(" src/panel-state-setters.ts
rg -n "openExternalLink" src/panel-state-setters.ts
rg -n "color: #6b2820|color: #7a2018|color: #54100f" src/styles.css
rg -n "\.url-action-button:focus-visible|outline: none" src/styles.css
rg -n "availableModels: \[\]|loadedModelsEndpoint: null" src/runtime-refresh.ts
```

Expected:

- External-link failures still do not route to URL input panel state.
- Remaining raw error colors are tokenized or explicitly justified.
- URL action focus still uses the shared focus ring.
- Runtime refresh still uses guarded planner model-list preservation logic.

Full validation gate:

```bash
source ./fix-node-version.sh
pnpm lint
pnpm test:ui
pnpm build
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
```

Only after those pass:

- update/check the final TODO checklist,
- add the Fix 6 `memory.md` entry with a real UTC timestamp.
