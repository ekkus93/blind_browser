# Blind Browser UI/UX Fix 4 TODO

## How to use this file

This TODO is a focused closeout pass for the remaining false-`DONE` items from UIUX Fix 3. Work top-to-bottom by priority. Do not redo working Fix 3 changes unless a task explicitly touches them.

Status key: `PENDING` · `IN PROGRESS` · `DONE` · `BLOCKED`

Priority key:

- `P0`: correctness or quiet-failure bug that can mislead users or hide failures.
- `P1`: accessibility/UX requirement that is still incomplete.
- `P2`: final validation, documentation, or lower-risk cleanup.

Validation gate:

```bash
source ./fix-node-version.sh
pnpm lint
pnpm test:ui
pnpm build
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
```

---

## P0.1 — Make external-link failures visible in settings

**Status:** PENDING  
**Files:**

- `src/panel-types.ts`
- `src/panel-state.ts`
- `src/app-shell-store.ts`
- `src/panel-state-setters.ts`
- `src/app-shell-nav.tsx`
- `src/app-shell.tsx`
- `src/app.tsx`
- `src/settings-panels/runtime.tsx` or a new small renderer file
- `src/styles.css`
- tests: likely `src/app-shell.test.mjs`, `src/dom-seams.test.mjs`, and/or a new focused render test

### Problem

`openExternalLink()` currently catches failures and writes the message into `urlInputPanelState.error`. That panel is part of the workspace and is hidden when the user is in settings. A settings user can click an external API-key/help link, have the OS/browser open fail, and see nothing.

This is still a quiet user-visible failure.

### Required behavior

- Failed external link opens must render a visible settings alert.
- The failed URL must be visible/copyable in the message.
- The alert must be visible on every settings view: overview, planner, tts, asr, runtime.
- The alert must be dismissible.
- Runtime refresh must not clear the alert.
- Do not route this through `urlInputPanelState.error`.

### P0.1.1 — Add settings alert state

In `src/panel-types.ts`, add:

```ts
export interface SettingsAlertPanelState {
  kind: "error" | "warning" | "info";
  message: string | null;
}
```

Add it to `PanelStates` in `src/panel-state.ts`:

```ts
settingsAlertState: SettingsAlertPanelState;
```

Initialize it in `createInitialPanelStates()`:

```ts
settingsAlertState: {
  kind: "info",
  message: null,
},
```

### P0.1.2 — Add Redux setter

In `src/app-shell-store.ts`, add a reducer:

```ts
setSettingsAlertPanelState(state, action: PayloadAction<Partial<PanelStates["settingsAlertState"]>>) {
  Object.assign(state.settingsAlertState, action.payload);
},
```

Export it with the other panel-state actions.

### P0.1.3 — Add setter helper and patch `openExternalLink()`

In `src/panel-state-setters.ts`, import the new store action:

```ts
setSettingsAlertPanelState as setSettingsAlertPanelStoreState,
```

Add:

```ts
export function setSettingsAlertState(nextState: Partial<PanelStates["settingsAlertState"]>) {
  appShellStore.dispatch(setSettingsAlertPanelStoreState(nextState));
}
```

You may need to import `PanelStates` from `./panel-state`.

Replace `openExternalLink()` with this shape:

```ts
export function openExternalLink(url: string) {
  void openExternalUrl({ url }).catch((error) => {
    console.error("Failed to open external link.", error);
    setSettingsAlertState({
      kind: "error",
      message: describeExternalLinkFailure(url, error),
    });
  });
}
```

Important: do **not** call `setUrlInputPanelStoreState()` from `openExternalLink()`.

### P0.1.4 — Add a render root that appears on every settings subpage

In `src/app-shell-nav.tsx`, add a panel root key:

```ts
| "settings-alert"
```

In `src/app-shell.tsx`, render it inside the settings `<section>` before the per-view settings containers:

```tsx
<section
  className={`app-view${settingsActive ? " app-view-active" : ""}`}
  data-app-view-section="settings"
  hidden={!settingsActive}
  aria-hidden={!settingsActive}
>
  {renderPanelContent("settings-alert", panelContent)}

  <div
    className={`settings-view${initialSettingsView === "overview" ? " settings-view-active" : ""}`}
    data-settings-view-section="overview"
    hidden={initialSettingsView !== "overview"}
    aria-hidden={initialSettingsView !== "overview"}
  >
```

This placement is deliberate: it makes the alert visible on `overview`, `planner`, `tts`, `asr`, and `runtime`.

### P0.1.5 — Add renderer

Either put this in `src/settings-panels/runtime.tsx` near `renderSettingsGuidancePanelNode()`, or create `src/settings-panels/settings-alert.tsx`.

Suggested renderer:

```tsx
import { type ReactNode } from "react";
import type { SettingsAlertPanelState } from "../panel-types.ts";

export interface SettingsAlertPanelHandlers {
  onDismiss?: () => void;
}

export function renderSettingsAlertPanelNode(
  state: SettingsAlertPanelState,
  handlers?: SettingsAlertPanelHandlers,
): ReactNode {
  if (!state.message) {
    return null;
  }

  const title = state.kind === "error"
    ? "Settings action failed"
    : state.kind === "warning"
      ? "Settings warning"
      : "Settings notice";

  return (
    <section
      className={`settings-alert settings-alert-${state.kind}`}
      role={state.kind === "error" ? "alert" : "status"}
      aria-live={state.kind === "error" ? "assertive" : "polite"}
      aria-labelledby="settings-alert-title"
    >
      <div className="settings-alert-copy">
        <p className="settings-alert-eyebrow">{state.kind}</p>
        <h2 id="settings-alert-title">{title}</h2>
        <p className="settings-alert-message">{state.message}</p>
      </div>
      <button
        type="button"
        className="panel-error-dismiss settings-alert-dismiss"
        data-settings-alert-dismiss="true"
        onClick={handlers?.onDismiss}
      >
        Dismiss
      </button>
    </section>
  );
}
```

### P0.1.6 — Wire renderer in `app.tsx`

Import the renderer and setter, then add this to `panelContent`:

```tsx
"settings-alert": renderSettingsAlertPanelNode(panelStates.settingsAlertState, {
  onDismiss: () => { setSettingsAlertState({ message: null }); },
}),
```

### P0.1.7 — Add CSS

In `src/styles.css`:

```css
.settings-alert {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  margin: 0 0 18px;
  padding: 16px 18px;
  border-radius: 18px;
  background: var(--color-error-light, rgba(139, 52, 42, 0.09));
  border: 1px solid rgba(139, 52, 42, 0.24);
  color: var(--color-text-primary);
}

.settings-alert-copy {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}

.settings-alert-eyebrow {
  margin: 0;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  font-size: 0.76rem;
  font-weight: 700;
  color: var(--color-error-primary);
}

.settings-alert h2,
.settings-alert-message {
  margin: 0;
}

.settings-alert-message {
  overflow-wrap: anywhere;
  line-height: 1.5;
}

.settings-alert-dismiss {
  flex: 0 0 auto;
}
```

If `--color-error-light` does not exist yet, add it in the token cleanup task below.

### P0.1.8 — Tests

Add a render test that verifies the alert is visible on a settings subpage.

Suggested shape for `src/app-shell.test.mjs`:

```js
test("settings alert renders above every settings subpage", () => {
  const html = renderAppShell({
    initialAppView: "settings",
    initialSettingsView: "planner",
    panelContent: {
      "settings-alert": '<section class="settings-alert" role="alert">Could not open the external link. Copy this URL and open it manually: https://example.test/docs.</section>',
    },
  });

  assert.match(html, /settings-alert/);
  assert.match(html, /Could not open the external link/);
  assert.match(html, /data-settings-view-section="planner"/);
});
```

Adapt to the project’s actual helper signatures.

### Acceptance checks

```bash
rg -n "openExternalLink|setUrlInputPanelStoreState|Could not open the external link" src/panel-state-setters.ts src/app.tsx src/app-shell.tsx src/app-shell-nav.tsx src/settings-panels
```

Expected:

- `openExternalLink()` routes to settings alert state.
- No external-link failure path writes to `urlInputPanelState.error`.
- Settings alert root is rendered inside the settings section, outside the individual overview/subpage containers.

---

## P0.2 — Preserve verified planner model-list state across runtime refresh

**Status:** PENDING  
**Files:**

- `src/runtime-refresh.ts`
- `src/main-behavior.test.mjs` or whichever test file covers runtime refresh dependencies

### Problem

Fix 3 stopped synthesizing a verified model list from saved settings. Good. But `runtime-refresh.ts` now always clears `availableModels` and `loadedModelsEndpoint`, so a genuinely loaded model list is wiped immediately after save because `persistRemotePlannerConnection()` calls `refreshRuntimePanels()`.

### Required invariant

- Do not synthesize a model list from saved backend settings.
- Do preserve an already-verified loaded list if the refreshed endpoint still matches it.

### P0.2.1 — Patch `applyAgentStateToPanels()`

In `src/runtime-refresh.ts`, compute the existing planner state before setting the remote planner panel state.

Suggested snippet:

```ts
const currentPanelStates = dependencies.getPanelStates();
const currentPlannerState = currentPanelStates.remotePlannerPanelState;
const refreshedPlannerBaseUrl = agentState.remote_planner_settings.base_url;
const keepVerifiedPlannerModels = currentPlannerState.availableModels.length > 0
  && currentPlannerState.loadedModelsEndpoint !== null
  && currentPlannerState.loadedModelsEndpoint === refreshedPlannerBaseUrl;
```

Then replace the current unconditional clear:

```ts
availableModels: [],
loadedModelsEndpoint: null,
```

with:

```ts
availableModels: keepVerifiedPlannerModels ? currentPlannerState.availableModels : [],
loadedModelsEndpoint: keepVerifiedPlannerModels ? currentPlannerState.loadedModelsEndpoint : null,
```

Keep this field untouched:

```ts
model: agentState.remote_planner_settings.model,
```

Do not reconstruct `availableModels` from `agentState.remote_planner_settings.model`.

### P0.2.2 — Avoid clearing unrelated URL errors during refresh if possible

This task is lower priority than preserving model lists, but while you are in `runtime-refresh.ts`, check the `setUrlInputPanelState({ error: null })` call. Runtime refresh should not clear unrelated user-visible errors unless the refresh action directly fixed that error.

Minimum acceptable behavior for this pass:

- Settings/global alert from P0.1 must not be affected by runtime refresh.
- URL input errors may stay as-is unless a successful URL action specifically clears them.

If you leave URL input clearing as-is, add a comment explaining that URL input refresh is intentionally scoped to workspace navigation state and does not control settings/global alerts.

### P0.2.3 — Add regression tests

Add tests covering both cases:

1. Preserve verified list when endpoint matches.
2. Clear verified list when endpoint changes.

Suggested test logic:

```js
test("runtime refresh preserves verified planner model list when endpoint still matches", async () => {
  const calls = [];
  const deps = createRuntimeRefreshTestDependencies({
    panelStates: {
      remotePlannerPanelState: {
        baseUrl: "https://api.example.com/v1",
        model: "gpt-test",
        availableModels: ["gpt-test", "gpt-other"],
        loadedModelsEndpoint: "https://api.example.com/v1",
      },
    },
    onSetRemotePlannerPanelState: (next) => calls.push(next),
  });

  applyAgentStateToPanelsForTest(deps, {
    remote_planner_settings: {
      base_url: "https://api.example.com/v1",
      model: "gpt-test",
      // include other required fields
    },
  });

  assert.deepEqual(calls.at(-1).availableModels, ["gpt-test", "gpt-other"]);
  assert.equal(calls.at(-1).loadedModelsEndpoint, "https://api.example.com/v1");
});
```

Adapt to the actual runtime-refresh test helpers. The important assertion is the state update, not the exact helper shape.

### Acceptance checks

```bash
rg -n "availableModels: \[\]|loadedModelsEndpoint: null" src/runtime-refresh.ts
```

Expected: any remaining clear must be guarded by endpoint mismatch/no verified list logic.

Manual flow:

1. Open AI assistant settings.
2. Enter endpoint.
3. Load model list successfully.
4. Select a model.
5. Save settings.
6. Confirm the UI still says `Model list up to date` if the endpoint did not change.

---

## P1.1 — Fix CSS token cascade and `--color-surface-inner`

**Status:** PENDING  
**Files:**

- `src/styles.css`

### Problem

The stylesheet currently has:

```css
--color-surface-inner: var(--color-surface-inner);
```

That is self-referential and can silently break light-mode component surfaces.

The dark-mode block also appears before the later base `:root` block. The later base root can override dark-mode values like `color-scheme`, border tokens, and eyebrow tokens.

### P1.1.1 — Replace self-referential token

In the `@theme` block, replace:

```css
--color-surface-inner: var(--color-surface-inner);
```

with:

```css
--color-surface-inner: rgba(255, 255, 255, 0.68);
```

Also add an error-light token if it is missing:

```css
--color-error-light: rgba(139, 52, 42, 0.09);
```

### P1.1.2 — Reorder dark mode after base `:root`

The file should have this high-level order:

```css
@import "tailwindcss";

@theme {
  /* concrete light-mode token values */
}

:root {
  color-scheme: light;
  font-family: var(--font-sans);
  background: linear-gradient(180deg, var(--color-surface-base) 0%, var(--color-surface-mid) 100%);
  color: var(--color-text-primary);

  --card-border: 1px solid rgba(123, 98, 70, 0.16);
  --inner-card-border: 1px solid rgba(123, 98, 70, 0.12);
  --eyebrow-color: #7b6246;
  --btn-nav-start: #5a5048;
  --btn-nav-end: #7a6860;
  --btn-nav-shadow: rgba(90, 80, 72, 0.18);
}

@media (prefers-color-scheme: dark) {
  :root {
    color-scheme: dark;

    --color-surface-base: #1a1712;
    --color-surface-mid: #141210;
    --color-surface-card: rgba(36, 32, 26, 0.92);
    --color-surface-inner: rgba(48, 43, 36, 0.80);

    --color-text-primary: #f0ead8;
    --color-text-secondary: #c8bfae;
    --color-text-muted: #a99b86;
    --color-text-label: #b4a48e;

    --card-border: 1px solid rgba(180, 155, 110, 0.14);
    --inner-card-border: 1px solid rgba(180, 155, 110, 0.10);
    --eyebrow-color: #b4a48e;

    --color-amber-primary: #c49a50;
    --color-amber-active: #e0b86a;
    --color-amber-light: rgba(196, 154, 80, 0.14);

    --color-error-primary: #e17b6f;
    --color-error-active: #ff9588;
    --color-error-dark: #ffb4aa;
    --color-error-light: rgba(225, 123, 111, 0.14);

    background: linear-gradient(180deg, var(--color-surface-base) 0%, var(--color-surface-mid) 100%);
  }
}
```

Important: the dark block must come **after** the base `:root`, not before it.

### P1.1.3 — Static acceptance checks

```bash
rg -n "--color-surface-inner: var\(--color-surface-inner\)" src/styles.css
```

Expected: no matches.

```bash
python3 - <<'PY'
from pathlib import Path
css = Path('src/styles.css').read_text()
root = css.find(':root')
dark = css.find('@media (prefers-color-scheme: dark)')
assert root != -1 and dark != -1 and dark > root, 'dark-mode block must appear after base :root block'
PY
```

---

## P1.2 — Tokenize remaining hardcoded light-mode surfaces and text

**Status:** PENDING  
**Files:**

- `src/styles.css`

### Problem

Dark mode is still incomplete because several component-level rules keep hardcoded light surfaces or dark text values.

### P1.2.1 — Apply these replacements

Use these exact replacements unless visual testing proves a better token is needed.

```css
.voice-status-strip {
  background: var(--color-surface-card);
  color: var(--color-text-secondary);
}
```

```css
.url-input-label,
.settings-control-label,
.confirmation-meta dt,
.confirmation-card h3 {
  color: var(--color-text-label);
}
```

```css
.status-toggle-button,
.settings-control-card {
  background: var(--color-surface-inner);
}
```

```css
.settings-control-card-readonly {
  background: color-mix(in srgb, var(--color-surface-card) 72%, transparent);
  border-color: var(--inner-card-border);
}
```

If `color-mix()` is not acceptable for the supported webview, use a static tokenized fallback and comment it.

```css
.settings-control-value,
.settings-api-key-test-status-message,
.confirmation-meta dd {
  color: var(--color-text-primary);
}
```

```css
.confirmation-panel {
  background:
    linear-gradient(135deg, var(--color-amber-light), var(--color-surface-card)),
    var(--color-surface-card);
}
```

### P1.2.2 — Do not blindly remove shadows/borders

Do not remove every `rgba(255, 255, 255, ...)` in shadows if it is intentional. This task is about component surfaces and text, not all decorative effects.

### P1.2.3 — Static acceptance checks

```bash
rg -n "color: #433d37|color: #1f2527|color: #6f675c|background: rgba\(255, 255, 255" src/styles.css
```

Expected: no component-level matches. If a remaining match is intentional, add a nearby comment explaining why it is mode-invariant.

Manual check:

- Switch OS/browser to dark mode.
- Settings cards must not remain white/light.
- Status toggle buttons must not remain white/light.
- Settings values and confirmation metadata must remain readable.

---

## P1.3 — Restore URL action button focus ring

**Status:** PENDING  
**Files:**

- `src/styles.css`

### Problem

`.url-action-button:focus-visible` currently sets `outline: none`. This violates the shared focus-ring requirement and makes keyboard focus harder to see.

### P1.3.1 — Split hover and focus rules

Replace the current combined rule:

```css
.url-action-button:hover:not(:disabled),
.url-action-button:focus-visible {
  outline: none;
  transform: translateY(-1px);
}
```

with:

```css
.url-action-button:hover:not(:disabled) {
  transform: translateY(-1px);
}

.url-action-button:focus-visible {
  outline: var(--focus-ring);
  outline-offset: var(--focus-offset);
  transform: translateY(-1px);
}
```

Keep existing variant-specific focus/hover box-shadow rules for `.url-open-button`, `.url-read-button`, `.url-stop-button`, `.url-previous-button`, and `.url-next-button`.

### P1.3.2 — Static acceptance checks

```bash
rg -n "url-action-button:focus-visible|outline: none|outline: var\(--focus-ring\)" src/styles.css
```

Expected:

- `.url-action-button:focus-visible` exists.
- It uses `outline: var(--focus-ring)` and `outline-offset: var(--focus-offset)`.
- It does not use `outline: none`.

Also manually inspect every `:focus-visible` rule:

```bash
rg -n ":focus-visible" src/styles.css
```

Every visible focus outline should use shared focus tokens unless an explicit comment justifies an exception.

---

## P2.1 — Add final closeout tests/static audit

**Status:** PENDING  
**Files:**

- relevant `*.test.mjs` files
- `memory.md`
- optional docs/TODO status file

### P2.1.1 — Run static searches

```bash
rg -n "setUrlInputPanelStoreState\(|setUrlInputPanelState\(" src/panel-state-setters.ts
rg -n "--color-surface-inner: var\(--color-surface-inner\)" src/styles.css
rg -n "color: #433d37|color: #1f2527|color: #6f675c|background: rgba\(255, 255, 255" src/styles.css
rg -n "\.url-action-button:focus-visible|outline: none" src/styles.css
rg -n "availableModels: \[\]|loadedModelsEndpoint: null" src/runtime-refresh.ts
```

Expected:

- External-link failure is not routed to URL input panel state.
- No self-referential surface-inner token.
- No unjustified hardcoded light-mode component colors/surfaces.
- URL action focus uses shared ring.
- Runtime refresh only clears planner model lists when the existing verified list is absent or endpoint mismatches.

### P2.1.2 — Run validation gate

```bash
source ./fix-node-version.sh
pnpm lint
pnpm test:ui
pnpm build
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
```

Do not mark this task done unless every command completes successfully.

### P2.1.3 — Update `memory.md`

Run:

```bash
date -u +"%Y-%m-%dT%H:%M:%SZ"
```

Add a real entry like:

```md
- 2026-XX-XXTXX:XX:XXZ — Completed UIUX Fix 4 closeout: made external-link failures visible in settings, fixed CSS dark-mode token cascade and surface-inner token, restored URL action focus rings, preserved verified planner model-list state across runtime refresh, and ran the full validation gate.
```

Do not fabricate the timestamp. Use the actual command output.

---

## Suggested commit sequence

1. `fix(settings): surface external-link failures in visible alert`
2. `fix(planner): preserve verified model list across refresh`
3. `style(ui): fix surface token cascade and dark mode remnants`
4. `style(ui): restore URL action focus outline`
5. `test(ui): add Fix 4 regression coverage`
6. `docs: record Fix 4 validation in memory`

---

## Final done checklist

- [ ] External-link failures render visible settings alert on every settings subpage.
- [ ] External-link failure message includes copyable failed URL.
- [ ] External-link alert can be dismissed.
- [ ] `openExternalLink()` does not route failures into `urlInputPanelState.error`.
- [ ] Runtime refresh does not clear the new settings/global alert.
- [ ] Runtime refresh preserves an already verified planner model list when endpoint still matches.
- [ ] Runtime refresh clears planner model list when endpoint changes or no verified list exists.
- [ ] Runtime refresh does not synthesize model list from saved settings.
- [ ] `--color-surface-inner` has a concrete light-mode value.
- [ ] Dark-mode block appears after base `:root` block.
- [ ] Remaining component-level hardcoded light text/surface values are tokenized or justified.
- [ ] `.url-action-button:focus-visible` uses `--focus-ring` and `--focus-offset`.
- [ ] Full validation gate passes.
- [ ] Static audit commands pass or have documented intentional exceptions.
- [ ] `memory.md` has a real UTC completion entry.
