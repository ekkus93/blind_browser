# FIX4 Spec/TODO Review — Responses and Questions

## Confirmed bugs (all accurate, ready to implement)

### 1. Self-referential `--color-surface-inner` (P1.1.1)

Confirmed. `src/styles.css` line 32 has:

```css
--color-surface-inner: var(--color-surface-inner);
```

This was introduced in FIX3 when a global replace of `rgba(255, 255, 255, 0.68)` with
`var(--color-surface-inner)` also hit the definition line in `@theme`. All eight component
rules that use `var(--color-surface-inner)` are currently broken in light mode.

### 2. Dark-mode block before base `:root` (P1.1.2)

Confirmed. The dark-mode `@media` block starts at line 50; the base `:root` block starts
at line 80. The base `:root`'s `color-scheme: light`, `--card-border`, `--inner-card-border`,
and `--eyebrow-color` override the dark block's versions of the same variables.

### 3. URL action button suppresses focus outline (P1.3)

Confirmed. Lines 719–723 of `src/styles.css`:

```css
.url-action-button:hover:not(:disabled),
.url-action-button:focus-visible {
  outline: none;
  transform: translateY(-1px);
}
```

This was missed in the FIX3 focus-ring unification pass because hover and focus are
combined in one rule. The `outline: none` kills keyboard focus visibility.

### 4. External-link failures route to hidden `urlInputPanelState` (P0.1)

Confirmed. `src/panel-state-setters.ts` line 68 dispatches
`setUrlInputPanelStoreState({ error: ... })`. That panel is not visible while the user
is in settings, so the failure is silent.

---

## One clarification on the spec

### P2.1.3 — `memory.md` "not updated" note

The spec says `memory.md` does not contain a real UIUX Fix 3 completion entry. This was
true when the spec was written, but `memory.md` was updated at the end of the FIX3
implementation session (timestamp `2026-06-28T23:40:27Z`). P2.1.3 only needs a new FIX4
entry — no retroactive FIX3 entry is required.

---

## One open question

### P0.1 — Should the settings alert also cover workspace external-link failures?

The TODO routes external-link failures exclusively to `settingsAlertState`, which is
rendered inside the settings `<section>`. Looking at the current codebase, all
`openExternalLink()` call sites appear to be in settings panels (API-key help links,
documentation links). If that is intentional and no external link will ever be triggered
from the workspace view, routing only to `settingsAlertState` is correct.

**Question**: Is it safe to assume that `openExternalLink()` will only ever be called
from settings contexts? Or should there be a fallback for workspace use — for example,
routing to `urlInputPanelState.error` when the settings view is not active, and to
`settingsAlertState` when it is?

If the answer is "settings only is fine for now," the implementation is straightforward.
If the answer is "we need to handle both," the implementation needs a small routing
decision based on which view is active, which adds some complexity.
