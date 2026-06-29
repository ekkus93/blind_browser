# Blind Browser UI/UX Fix 5 TODO

## How to use this file

This is a focused closeout TODO for the remaining Fix 4 false-completion items. Work top-to-bottom. Do not redo working Fix 4 changes unless this file explicitly names the code.

Status key: `PENDING` · `IN PROGRESS` · `DONE` · `BLOCKED`

Priority key:

- `P0`: quiet-failure/correctness or missing regression coverage for user-visible failure behavior.
- `P1`: incomplete dark-mode/accessibility/static-audit cleanup.
- `P2`: validation, memory, and final closeout.

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

## P0.1 — Add direct regression coverage for external-link failure alert

**Status:** DONE  
**Files:**

- `src/panel-state-setters.ts`
- `src/dom-seams.test.mjs`, `src/app-shell.test.mjs`, or a new focused test file
- possibly `src/app-alert-panel.tsx` if small test hooks are needed

### Problem

Fix 4 implemented a global app alert for external-link failures. That appears correct. However, current coverage mostly verifies that app-shell alert markup can render. It does not strongly prove the failure path:

```text
openExternalLink()
→ openExternalUrl() rejects
→ setAppAlertState(...)
→ alert message includes failed URL
→ dismiss clears the alert
```

This was a P0 quiet-failure bug. It needs direct regression coverage.

### Required behavior

- `openExternalLink(url)` must not throw.
- If `openExternalUrl({ url })` rejects, global app alert state must be set.
- The alert message must include:
  - `Could not open the external link`
  - the failed URL
  - the thrown error message, if present.
- The implementation must not write the failure to `urlInputPanelState.error`.
- Runtime refresh must not clear this alert.

### P0.1.1 — Keep current global app alert routing

Do **not** replace global app alert routing with settings-only routing.

`openExternalLink()` should keep this shape:

```ts
export function openExternalLink(url: string) {
  void openExternalUrl({ url }).catch((error) => {
    console.error("Failed to open external link.", error);
    setAppAlertState({
      kind: "error",
      message: describeExternalLinkFailure(url, error),
    });
  });
}
```

Acceptance search:

```bash
rg -n "setUrlInputPanelStoreState\(|setUrlInputPanelState\(" src/panel-state-setters.ts
```

Expected: no external-link failure path uses URL input panel state.

### P0.1.2 — Add a focused failure-path test

Use whatever test seams already exist in the project. The exact helper names may differ, but the test must simulate `openExternalUrl()` rejection and assert the resulting alert state/message.

Preferred test shape if store/seams are importable:

```js
test("openExternalLink surfaces rejected external open as global app alert", async () => {
  const url = "https://example.test/docs";
  const restore = installOpenExternalUrlMock(async () => {
    throw new Error("portal unavailable");
  });

  try {
    clearAppAlert();
    openExternalLink(url);

    // Wait one microtask because openExternalLink intentionally fire-and-forgets.
    await Promise.resolve();
    await Promise.resolve();

    const state = appShellStore.getState();
    assert.equal(state.appAlertState.kind, "error");
    assert.match(state.appAlertState.message, /Could not open the external link/);
    assert.match(state.appAlertState.message, /https:\/\/example\.test\/docs/);
    assert.match(state.appAlertState.message, /portal unavailable/);
  } finally {
    restore();
    clearAppAlert();
  }
});
```

If the current seams cannot mock `openExternalUrl()`, add a minimal test seam instead of overhauling the architecture. For example:

```ts
// in src/dom-seams.ts or the existing seam file
let openExternalUrlImpl = openExternalUrl;

export function setOpenExternalUrlForTest(next: typeof openExternalUrl) {
  openExternalUrlImpl = next;
  return () => {
    openExternalUrlImpl = openExternalUrl;
  };
}

export function getOpenExternalUrlForRuntime() {
  return openExternalUrlImpl;
}
```

Then `openExternalLink()` calls the seam:

```ts
export function openExternalLink(url: string) {
  void getOpenExternalUrlForRuntime()({ url }).catch((error) => {
    console.error("Failed to open external link.", error);
    setAppAlertState({
      kind: "error",
      message: describeExternalLinkFailure(url, error),
    });
  });
}
```

Do not add a broad dependency-injection framework. Keep this test seam minimal.

### P0.1.3 — Add or verify dismiss behavior test

There should be a test that proves dismiss clears the alert.

Suggested state-level test:

```js
test("app alert dismiss clears global alert message", () => {
  setAppAlertState({
    kind: "error",
    message: "Could not open the external link. Copy this URL and open it manually: https://example.test/docs.",
  });

  clearAppAlert();

  const state = appShellStore.getState();
  assert.equal(state.appAlertState.message, null);
});
```

If `clearAppAlert()` does not exist, add it:

```ts
export function clearAppAlert() {
  setAppAlertState({
    kind: "info",
    message: null,
  });
}
```

### Acceptance checks

- A rejected external-link open sets global alert state.
- Failed URL is present in alert message.
- Dismiss clears the alert.
- Static search confirms external-link failure is not routed through URL input panel state.

---

## P1.1 — Fix the known failing CSS static audit match

**Status:** DONE  
**Files:**

- `src/styles.css`

### Problem

The Fix 4 static audit still fails because this hardcoded light-mode text color remains:

```css
.settings-api-key-test-status-message {
  margin: 0;
  line-height: 1.55;
  color: #1f2527;
  font-weight: 600;
}
```

This violates the required audit:

```bash
rg -n "color: #433d37|color: #1f2527|color: #6f675c|background: rgba\(255, 255, 255" src/styles.css
```

### P1.1.1 — Replace the hardcoded text color

Patch:

```css
.settings-api-key-test-status-message {
  margin: 0;
  line-height: 1.55;
  color: var(--color-text-primary);
  font-weight: 600;
}
```

### Acceptance check

Run:

```bash
rg -n "color: #433d37|color: #1f2527|color: #6f675c|background: rgba\(255, 255, 255" src/styles.css
```

Expected: no component-level matches. If there is a remaining match, either replace it with a token or add a nearby comment explaining why it is mode-invariant and safe.

---

## P1.2 — Tokenize remaining hardcoded parchment/light surfaces

**Status:** DONE  
**Files:**

- `src/styles.css`

### Problem

Fix 4 resolved the narrow `rgba(255, 255, 255, ...)` audit but left multiple `rgba(255, 252, 247, ...)` component backgrounds. These are warm light surfaces. In dark mode, they can become bright islands.

Known examples from the latest static review:

```text
background: rgba(255, 252, 247, 0.82);
background: rgba(255, 252, 247, 0.6);
background: rgba(255, 252, 247, 0.9);
```

### Required rule

Component surfaces should generally use one of:

```css
background: var(--color-surface-card);
background: var(--color-surface-inner);
background: color-mix(in srgb, var(--color-surface-card) 72%, transparent);
```

Do not blindly replace decorative shadows/borders. This task is for component backgrounds.

### P1.2.1 — Replace shell/toolbar/panel light surfaces

Look for rules like these:

```css
background: rgba(255, 252, 247, 0.82);
background: rgba(255, 252, 247, 0.6);
background: rgba(255, 252, 247, 0.9);
```

Suggested replacements by intent:

```css
.shell-toolbar-action,
.settings-subpage-back,
.panel {
  background: var(--color-surface-card);
}
```

For hover/focus surfaces where translucency is desired:

```css
.settings-subpage-card:hover,
.settings-subpage-card:focus-visible {
  background: var(--color-surface-card);
}
```

For intentionally subtle surfaces:

```css
.some-subtle-surface {
  background: color-mix(in srgb, var(--color-surface-card) 72%, transparent);
}
```

Use the actual selectors already present in `src/styles.css`. Do not create duplicate selectors if an existing rule can be edited directly.

### P1.2.2 — Replace settings card light surfaces

If a settings-related card uses a hardcoded parchment surface, prefer:

```css
.settings-subpage-card,
.settings-control-card,
.settings-control-card-readonly {
  background: var(--color-surface-card);
}
```

or for inner cards:

```css
.settings-control-card,
.settings-control-card-readonly {
  background: var(--color-surface-inner);
}
```

Use `var(--color-surface-inner)` for nested form controls/cards; use `var(--color-surface-card)` for primary cards.

### P1.2.3 — Static acceptance check

Run:

```bash
rg -n "background: rgba\(255, 252, 247" src/styles.css
```

Expected: no component-level matches. If a match is intentionally mode-invariant, add a nearby comment:

```css
/* Intentional mode-invariant decorative highlight; not a component surface. */
```

Do not use this comment to justify ordinary cards, panels, controls, or buttons.

---

## P1.3 — Tokenize remaining hardcoded light-mode text colors

**Status:** DONE  
**Files:**

- `src/styles.css`

### Problem

The latest review found additional hardcoded text colors outside the narrow Fix 4 audit:

```text
color: #7b6246;
color: #3a342e;
color: #5d584e;
color: #2c3233;
```

These may be low-contrast or visually inconsistent in dark mode.

### Required mapping

Use this mapping unless visual testing proves a better token is needed:

```text
#7b6246  → var(--eyebrow-color) or var(--color-text-label)
#3a342e  → var(--color-text-secondary)
#5d584e  → var(--color-text-secondary) or var(--color-text-muted)
#2c3233  → var(--color-text-secondary)
```

### P1.3.1 — Replace eyebrow/status label hardcodes

For eyebrow-style labels:

```css
.eyebrow,
.status-panel-eyebrow,
.settings-model-freshness-label {
  color: var(--eyebrow-color);
}
```

If a selector is not an eyebrow but still a label, use:

```css
color: var(--color-text-label);
```

### P1.3.2 — Replace body/secondary copy hardcodes

For ledes, indicators, muted list text, and confirmation copy:

```css
.lede,
.status-indicator,
.confirmation-card ul {
  color: var(--color-text-secondary);
}
```

For less important secondary text:

```css
color: var(--color-text-muted);
```

### P1.3.3 — Static acceptance check

Run:

```bash
rg -n "color: #7b6246|color: #3a342e|color: #5d584e|color: #2c3233" src/styles.css
```

Expected: no component-level matches. Any intentional exception needs a nearby comment explaining why it is safe in both light and dark mode.

---

## P1.4 — Re-check focus ring did not regress

**Status:** DONE  
**Files:**

- `src/styles.css`

### Problem

Fix 4 restored `.url-action-button:focus-visible`. Fix 5 CSS edits must not regress focus visibility.

### P1.4.1 — Static focus audit

Run:

```bash
rg -n ":focus-visible" src/styles.css
```

Manually inspect every result. Every visible focus outline must use:

```css
outline: var(--focus-ring);
outline-offset: var(--focus-offset);
```

unless there is an explicit nearby comment explaining why a different focus style is accessible and intentional.

### P1.4.2 — URL action specific audit

Run:

```bash
rg -n "\.url-action-button:focus-visible|outline: none" src/styles.css
```

Expected:

- `.url-action-button:focus-visible` exists.
- It uses `outline: var(--focus-ring)`.
- It uses `outline-offset: var(--focus-offset)`.
- It does not use `outline: none`.

---

## P1.5 — Manual dark-mode walkthrough checklist

**Status:** DONE (automated via Docker + Xvfb + GTK_THEME=Adwaita:dark + scrot + ImageMagick; see scripts/darkmode-test.sh)  
**Files:**

- no code file unless issues are found
- optionally `memory.md` final entry summarizes walkthrough

### Required manual checks

Switch OS/browser/webview to dark mode and inspect:

- Workspace shell
- Toolbar buttons
- URL input panel
- Status panel
- Voice/PTT panel
- Settings overview
- Planner settings
- TTS settings
- ASR settings
- Runtime/advanced settings
- Confirmation panels
- Global app alert

### Expected result

- No card/panel/control remains bright parchment or white unless intentionally highlighted and documented.
- Text remains readable.
- Error/warning/alert states remain distinct.
- Keyboard focus rings remain visible.
- Global app alert remains visible and dismissible.

If any visual issue is found, fix it with tokens rather than dark-only one-off overrides when possible.

---

## P2.1 — Run final static audits

**Status:** DONE  
**Files:**

- `src/styles.css`
- `src/panel-state-setters.ts`
- `src/runtime-refresh.ts`

Run all commands:

```bash
rg -n "setUrlInputPanelStoreState\(|setUrlInputPanelState\(" src/panel-state-setters.ts
rg -n "--color-surface-inner: var\(--color-surface-inner\)" src/styles.css
rg -n "color: #433d37|color: #1f2527|color: #6f675c|background: rgba\(255, 255, 255" src/styles.css
rg -n "background: rgba\(255, 252, 247" src/styles.css
rg -n "color: #7b6246|color: #3a342e|color: #5d584e|color: #2c3233" src/styles.css
rg -n "\.url-action-button:focus-visible|outline: none" src/styles.css
rg -n "availableModels: \[\]|loadedModelsEndpoint: null" src/runtime-refresh.ts
```

Expected:

- No external-link failure routing through URL input panel state.
- No self-referential surface-inner token.
- No unjustified hardcoded light-mode component colors/surfaces.
- URL action focus uses the shared ring and does not suppress outline.
- Runtime refresh only clears planner model list through guarded endpoint mismatch/no verified list logic.

If a static-audit match remains intentionally, document it with a nearby comment and mention it in the final memory entry.

---

## P2.2 — Run full validation gate

**Status:** DONE  
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

If any command fails:

1. Fix the failure.
2. Re-run the full gate.
3. Record the final successful run in `memory.md`.

---

## P2.3 — Update `memory.md` with a real Fix 5 closeout entry

**Status:** DONE  
**Files:**

- `memory.md`

Only do this after P2.1 and P2.2 pass.

Run:

```bash
date -u +"%Y-%m-%dT%H:%M:%SZ"
```

Add an entry like:

```md
- 2026-XX-XXTXX:XX:XXZ — Completed UIUX Fix 5 closeout: added direct external-link failure regression coverage, finished tokenizing remaining hardcoded light-mode CSS surfaces/text, verified URL action focus rings, passed static audits, and ran the full validation gate.
```

Replace the timestamp with the actual command output. Do not fabricate or reuse an old timestamp.

---

## Suggested commit sequence

1. `test(ui): cover external-link failure alert path`
2. `style(ui): finish hardcoded light-mode CSS token cleanup`
3. `style(ui): verify focus and dark-mode closeout`
4. `test: run Fix 5 static audits and validation`
5. `docs: record Fix 5 completion in memory`

---

## Final done checklist

- [x] Rejected external-link open has direct regression coverage.
- [x] Alert message includes failed URL and error detail.
- [x] Alert dismiss behavior is tested or clearly covered.
- [x] External-link failures do not route to `urlInputPanelState.error`.
- [x] `settings-api-key-test-status-message` uses `var(--color-text-primary)`.
- [x] No unjustified hardcoded `rgba(255, 252, 247, ...)` component backgrounds remain.
- [x] No unjustified hardcoded `#7b6246`, `#3a342e`, `#5d584e`, or `#2c3233` component text colors remain.
- [x] `.url-action-button:focus-visible` still uses `--focus-ring` and `--focus-offset`.
- [x] Manual dark-mode walkthrough completed.
- [x] Static audits pass or intentional exceptions are documented inline.
- [x] Full validation gate passes.
- [x] `memory.md` has a real UTC Fix 5 completion entry.
