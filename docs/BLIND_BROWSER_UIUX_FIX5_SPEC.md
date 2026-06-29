# Blind Browser UI/UX Fix 5 Spec

## Purpose

Fix 5 is a focused closeout pass after the Fix 4 implementation. Fix 4 corrected the major P0 behaviors, but the final static audit still did not pass because `src/styles.css` retained hardcoded light-mode component text/surface values. Fix 5 must finish the remaining CSS/dark-mode cleanup, add missing regression coverage for the external-link failure path, and only then update final validation documentation.

This is **not** a broad redesign pass. Do not redo working Fix 3/Fix 4 changes unless a task explicitly touches the relevant code.

## Current known state

The following Fix 4 work appears correct and should be preserved:

- External-link failures now route through a global app alert instead of the hidden workspace URL input panel.
- Runtime refresh preserves a genuinely verified remote planner model list when the endpoint still matches.
- Runtime refresh does not synthesize model lists from saved backend settings.
- `--color-surface-inner` no longer self-references.
- The dark-mode block now appears after the base `:root` declaration.
- `.url-action-button:focus-visible` uses the shared focus ring.
- `memory.md` has a Fix 4 entry.

The remaining problem is that the Fix 4 checklist was marked complete even though the CSS static audit still matched hardcoded light-mode component values. Fix 5 exists to prevent that class of false completion.

## Required outcome

After Fix 5:

1. The CSS static audit for hardcoded light-mode component text/surfaces must pass.
2. Any remaining hardcoded light-mode color or surface must have an explicit nearby comment explaining why it is mode-invariant and safe.
3. Dark mode must not contain bright/light component islands from hardcoded `rgba(255, 252, 247, ...)` or `rgba(255, 255, 255, ...)` component backgrounds.
4. Keyboard focus behavior from Fix 4 must not regress.
5. External-link failure handling must have a direct regression test for the actual failure path, not only an app-shell placeholder render test.
6. `memory.md` must only receive a new Fix 5 entry after the static audit and full validation gate pass.

## Non-goals

Do not:

- Rename or redesign the panel architecture.
- Replace the global app alert with a settings-only alert.
- Reintroduce `urlInputPanelState.error` as the external-link failure path.
- Rework the remote planner model-list logic unless a regression is found.
- Add broad new UI features.
- Mark TODO items `DONE` just because code was edited.

## Design principles

### 1. Treat dark mode as token-driven UI, not ad hoc overrides

Component CSS should use semantic/tokenized values:

```css
background: var(--color-surface-card);
background: var(--color-surface-inner);
color: var(--color-text-primary);
color: var(--color-text-secondary);
color: var(--color-text-muted);
color: var(--color-text-label);
color: var(--eyebrow-color);
border: var(--card-border);
border: var(--inner-card-border);
```

Hardcoded component colors like `#1f2527`, `#7b6246`, `#3a342e`, `#5d584e`, and `#2c3233` are not acceptable unless intentionally mode-invariant and documented.

### 2. Prefer fixing component rules over adding dark-only patches

If a component has:

```css
.some-card {
  background: rgba(255, 252, 247, 0.82);
  color: #1f2527;
}
```

prefer:

```css
.some-card {
  background: var(--color-surface-card);
  color: var(--color-text-primary);
}
```

instead of adding a separate dark-mode override for `.some-card`. The base component should inherit correct theme behavior.

### 3. Preserve visible alert behavior across app views

`openExternalLink()` is a generic helper. It must remain safe even if a future call site is added outside settings. The global app alert is the preferred implementation. Do not downgrade it to settings-only routing.

### 4. Tests should verify behavior, not just placeholder rendering

A test that passes pre-rendered alert markup into `renderAppShell()` is useful for layout, but it does not prove that a failed external-link open produces a visible alert. Fix 5 needs a direct test of the failure path or the state setter path.

### 5. Completion requires evidence

The final TODO checkbox must not be marked done unless:

- static audits pass,
- the full validation gate passes in the developer environment,
- `memory.md` has a real UTC Fix 5 completion entry,
- and any intentional static-audit exceptions are documented inline.

## Expected files touched

Likely files:

- `src/styles.css`
- `src/panel-state-setters.ts`
- `src/app-alert-panel.tsx` if alert accessibility/test hooks need small adjustments
- `src/dom-seams.test.mjs`, `src/app-shell.test.mjs`, or a new focused test file for external-link failure behavior
- `memory.md`
- optionally the Fix 5 TODO status file after implementation

Avoid touching Rust, planner logic, or unrelated settings panels unless a validation failure requires it.

## Acceptance summary

At the end of implementation, the following must be true:

```bash
rg -n "color: #433d37|color: #1f2527|color: #6f675c|background: rgba\(255, 255, 255" src/styles.css
```

No unjustified component-level matches.

```bash
rg -n "background: rgba\(255, 252, 247|color: #7b6246|color: #3a342e|color: #5d584e|color: #2c3233" src/styles.css
```

No unjustified component-level matches.

```bash
rg -n "setUrlInputPanelStoreState\(|setUrlInputPanelState\(" src/panel-state-setters.ts
```

No external-link failure routing through the workspace URL input error.

```bash
rg -n "\.url-action-button:focus-visible|outline: none" src/styles.css
```

`.url-action-button:focus-visible` uses `var(--focus-ring)` and `var(--focus-offset)`, and no URL action focus path suppresses the outline.

Full validation gate:

```bash
source ./fix-node-version.sh
pnpm lint
pnpm test:ui
pnpm build
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
```
