# Blind Browser UI/UX Fix 6 TODO

## How to use this file

This is a narrow final cleanup pass after Fix 5. Do not redo working Fix 5 changes. Work top-to-bottom and keep the diff small.

Status key: `PENDING` · `IN PROGRESS` · `DONE` · `BLOCKED`

Priority key:

- `P0`: must not regress prior quiet-failure fixes.
- `P1`: code cleanup or dark-mode/accessibility polish.
- `P2`: validation/documentation closeout.

Validation gate:

```bash
source ./fix-node-version.sh
pnpm lint
pnpm test:ui
pnpm build
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
```

Do not mark this TODO complete unless the validation gate actually passes in the developer environment.

---

## P0.1 — Preserve Fix 5 quiet-failure protections

**Status:** PENDING  
**Files:**

- `src/panel-state-setters.ts`
- `src/external-link.test.mjs`
- `src/runtime-refresh.ts`
- `src/styles.css`

### Problem

Fix 5 solved the major quiet-failure path: failed external-link opens are now visible through a global app alert and tested directly. Fix 6 must not accidentally regress this while doing cleanup.

### Required invariants

- `openExternalLink()` still uses the global app alert.
- `openExternalLink()` does not route failures through `urlInputPanelState.error`.
- The failed URL remains visible in the alert message.
- The thrown error detail remains included when available.
- Runtime refresh does not clear the global app alert.
- Planner model-list refresh behavior remains truthful:
  - preserve verified list when endpoint still matches,
  - clear when endpoint changes/no verified list exists,
  - never synthesize a model list from saved backend settings.

### P0.1.1 — Static regression checks

Run:

```bash
rg -n "setUrlInputPanelStoreState\(|setUrlInputPanelState\(" src/panel-state-setters.ts
rg -n "openExternalLink|setAppAlertState|describeExternalLinkFailure" src/panel-state-setters.ts
rg -n "availableModels: \[\]|loadedModelsEndpoint: null" src/runtime-refresh.ts
```

Expected:

- `openExternalLink()` uses `setAppAlertState(...)`.
- No external-link failure path writes to URL input panel state.
- Runtime refresh does not contain unconditional `availableModels: []` / `loadedModelsEndpoint: null` that wipes a verified list after save.

### P0.1.2 — Test regression checks

Run the UI tests after later code edits:

```bash
pnpm test:ui
```

Expected:

- external-link rejection test still passes,
- app-alert dismiss/clear test still passes,
- planner model-list runtime refresh tests still pass,
- focus/CSS render tests still pass.

If any of those tests are missing, do not create a broad test rewrite. Add only the smallest missing regression test.

---

## P1.1 — Use `clearAppAlert` for app-alert dismiss behavior

**Status:** PENDING  
**Files:**

- `src/app.tsx`
- `src/panel-state-setters.ts`
- optional: `src/external-link.test.mjs` or app-alert render test

### Problem

The UI dismiss handler currently hides the alert by setting only:

```ts
setAppAlertState({ message: null })
```

This is user-visible-safe because the alert hides when `message` is null. But it leaves stale internal state such as:

```ts
{
  kind: "error",
  message: null,
}
```

That is not dangerous, but it is sloppy and can confuse debugging/tests.

### Required behavior

Dismissing the app alert should reset the full alert state:

```ts
{
  kind: "info",
  message: null,
}
```

### P1.1.1 — Ensure `clearAppAlert()` exists

In `src/panel-state-setters.ts`, keep or add:

```ts
export function clearAppAlert() {
  setAppAlertState({
    kind: "info",
    message: null,
  });
}
```

### P1.1.2 — Use `clearAppAlert` in the app-alert dismiss handler

In `src/app.tsx`, replace any inline partial dismiss like:

```tsx
"app-alert": renderAppAlertPanelNode(panelStates.appAlertState, {
  onDismiss: () => { setAppAlertState({ message: null }); },
}),
```

with:

```tsx
"app-alert": renderAppAlertPanelNode(panelStates.appAlertState, {
  onDismiss: clearAppAlert,
}),
```

Adjust imports:

```ts
import {
  clearAppAlert,
  // existing imports...
} from "./panel-state-setters.ts";
```

### P1.1.3 — Verify or update tests

If there is already a test for `clearAppAlert()`, make sure it asserts both fields:

```js
assert.equal(state.appAlertState.kind, "info");
assert.equal(state.appAlertState.message, null);
```

If no render-level dismiss test exists, add a small one only if the existing test helpers make it easy. Do not overbuild.

### Acceptance checks

Search:

```bash
rg -n "setAppAlertState\(\{ message: null \}\)|onDismiss: \(\) => \{ setAppAlertState" src
```

Expected: no app-alert dismiss path uses a partial stale-state clear.

---

## P1.2 — Tokenize remaining hardcoded error/danger text colors

**Status:** PENDING  
**Files:**

- `src/styles.css`

### Problem

Fix 5 correctly focused on the previous hardcoded light-mode surface/text audit. A few hardcoded error/danger colors remain outside that audit. They are not the same class of silent bug as the old dark-mode light-surface issue, but they should be tokenized for consistency and dark-mode correctness.

Known examples to check:

```css
color: #6b2820;
color: #7a2018;
color: #54100f;
```

Likely selectors include:

```css
.settings-subpage-card-status-error
.settings-reset-confirm-message
.confirmation-error
.confirmation-error-tool-hard-stop
```

### Required mapping

Use the existing error tokens:

```css
color: var(--color-error-primary);
```

for normal error text, and:

```css
color: var(--color-error-dark);
```

for stronger/hard-stop error text.

### P1.2.1 — Patch settings error text

Suggested replacements:

```css
.settings-subpage-card-status-error {
  color: var(--color-error-primary);
}
```

```css
.settings-reset-confirm-message {
  color: var(--color-error-primary);
}
```

If either rule also has backgrounds/borders, keep them unless they use hardcoded colors that should already map to `--color-error-light` or tokenized borders.

### P1.2.2 — Patch confirmation error text

Suggested replacements:

```css
.confirmation-error {
  color: var(--color-error-primary);
}
```

```css
.confirmation-error-tool-hard-stop {
  color: var(--color-error-dark);
}
```

If `.confirmation-error-tool-hard-stop` needs stronger emphasis, use font weight or background/border tokens instead of raw hardcoded text color.

### P1.2.3 — Static acceptance check

Run:

```bash
rg -n "color: #6b2820|color: #7a2018|color: #54100f" src/styles.css
```

Expected: no component-level matches.

If a raw color remains intentionally, add a nearby comment:

```css
/* Intentional mode-invariant brand/error color; verified for light and dark contrast. */
```

Do not use this comment for ordinary error text unless there is a concrete reason.

---

## P1.3 — Re-run dark-mode and focus static audits

**Status:** PENDING  
**Files:**

- `src/styles.css`

### P1.3.1 — Dark-mode hardcoded value audit

Run:

```bash
rg -n "background: rgba\(255, 252, 247|background: rgba\(255, 255, 255|color: #433d37|color: #1f2527|color: #6f675c|color: #7b6246|color: #3a342e|color: #5d584e|color: #2c3233" src/styles.css
```

Expected:

- no component-level matches,
- root token definitions are acceptable,
- any intentional component-level exception has a nearby comment.

### P1.3.2 — Focus audit

Run:

```bash
rg -n ":focus-visible" src/styles.css
```

Manually inspect all focus-visible rules. Every visible outline should use:

```css
outline: var(--focus-ring);
outline-offset: var(--focus-offset);
```

unless a nearby comment explains an intentional accessible exception.

### P1.3.3 — URL action specific check

Run:

```bash
rg -n "\.url-action-button:focus-visible|outline: none" src/styles.css
```

Expected:

- `.url-action-button:focus-visible` exists,
- it uses `outline: var(--focus-ring)`,
- it uses `outline-offset: var(--focus-offset)`,
- it does not use `outline: none`.

---

## P2.1 — Reconcile final TODO checklist/status

**Status:** PENDING  
**Files:**

- current working TODO file, likely `BLIND_BROWSER_UIUX_FIX5_TODO(1).md` or successor
- optional project docs

### Problem

The Fix 5 task statuses were marked `DONE`, but the final checklist remained unchecked. That is process sloppiness and makes future review harder.

### Required behavior

For the working TODO/status file:

- If a task is complete and verified, check the corresponding final checklist item.
- If a checklist item cannot be verified, leave it unchecked and add a short note explaining why.
- Do not mark validation complete unless the full validation gate actually passed.

### P2.1.1 — Update checklist after validation, not before

Do not check final checklist items until after P2.2 validation passes.

Once validation passes, check items like:

```md
- [x] Rejected external-link open has direct regression coverage.
- [x] Alert message includes failed URL and error detail.
- [x] Alert dismiss behavior is tested or clearly covered.
```

If this repo does not track uploaded TODO files, update the local project status document that Claude Code is using instead.

---

## P2.2 — Run final validation gate

**Status:** PENDING  
**Files:**

- no source file unless failures require fixes

Run:

```bash
source ./fix-node-version.sh
pnpm lint
pnpm test:ui
pnpm build
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
```

Do not mark this task done unless every command completes successfully.

If a command fails:

1. Fix the failure.
2. Re-run the full gate.
3. Only then update the checklist and memory entry.

---

## P2.3 — Add Fix 6 memory entry with real UTC timestamp

**Status:** PENDING  
**Files:**

- `memory.md`

Only do this after P2.2 passes.

Run:

```bash
date -u +"%Y-%m-%dT%H:%M:%SZ"
```

Add an entry like:

```md
- 2026-XX-XXTXX:XX:XXZ — Completed UIUX Fix 6 cleanup: used full app-alert clear on dismiss, tokenized remaining hardcoded error colors, re-ran dark-mode/focus/static audits, reconciled TODO checklist status, and ran the full validation gate.
```

Replace the timestamp with the actual command output. Do not fabricate or reuse an old timestamp.

---

## Suggested commit sequence

1. `fix(ui): clear app alert state on dismiss`
2. `style(ui): tokenize remaining error text colors`
3. `test(ui): verify alert clear and focus regressions`
4. `docs: reconcile Fix 5 checklist and record Fix 6 validation`

---

## Final done checklist

- [ ] `openExternalLink()` still routes rejected opens to global app alert.
- [ ] External-link failures do not route to `urlInputPanelState.error`.
- [ ] App-alert dismiss uses `clearAppAlert` or equivalent full-state reset.
- [ ] App-alert clear behavior is tested or already covered.
- [ ] Remaining hardcoded error text colors are tokenized or explicitly justified.
- [ ] Dark-mode hardcoded light-surface/text audit passes.
- [ ] URL action focus ring still uses `--focus-ring` and `--focus-offset`.
- [ ] Runtime refresh still preserves verified planner model list only when endpoint matches.
- [ ] Final TODO/checklist status is reconciled.
- [ ] Full validation gate passes.
- [ ] `memory.md` has a real UTC Fix 6 completion entry.
