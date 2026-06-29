# Blind Browser UI/UX Fix 4 Spec

## Purpose

This spec defines the next hardening pass after the UIUX Fix 3 review. Fix 3 implemented several useful changes, but some tasks were marked `DONE` while their behavior was still incomplete or misleading. Fix 4 must close those remaining gaps without reworking the already-successful pieces.

The core theme of this pass is **truthful user-visible state**:

- Do not route errors into hidden panels.
- Do not let CSS silently fall back because a variable is invalid or overridden by cascade order.
- Do not remove a verified planner model-list state immediately after saving it.
- Do not suppress keyboard focus outlines.
- Do not mark validation/memory complete unless the commands really ran.

## Current known-good Fix 3 behavior to preserve

Do not regress these items:

1. `FrontendToolError` preserves structured backend `ToolError` metadata.
2. Push-to-talk setup-required state renders setup guidance instead of the large disabled talk button.
3. Planner manual model entry is allowed without requiring model-list loading.
4. Audio capture lock failures are observable instead of silently dropped.
5. Google Fonts dependency is explicitly documented when fonts are not locally bundled.
6. The large Rust test fixture has an explicit documented exemption from the 600-line target.

## Remaining problems to fix

### 1. External-link failures are not visible in the settings context

`openExternalLink()` currently catches `openExternalUrl()` failures and stores the message in `urlInputPanelState.error`. That is not acceptable because external links are often clicked from settings subpages. The workspace URL input panel is hidden while settings are visible, so the user may never see the error.

The app needs a **settings/global alert channel** that is rendered inside the settings view regardless of the active settings subpage.

Required end state:

- Failed external-link opens render visible text in settings.
- The failed URL is visible/copyable in the rendered message.
- The alert is visible on `overview`, `planner`, `tts`, `asr`, and `runtime` settings views.
- The alert has an explicit dismiss button.
- Runtime refresh must not clear this alert incidentally.
- `urlInputPanelState.error` must only be used for URL/workspace navigation errors, not settings/global failures.

### 2. CSS token and dark-mode cascade is still incomplete

`src/styles.css` currently contains a self-referential custom property:

```css
--color-surface-inner: var(--color-surface-inner);
```

This is a bug. Any component using `var(--color-surface-inner)` can silently render without the intended light-mode surface.

The dark-mode override block is also ordered before the later base `:root` block. The later `:root` block resets `color-scheme`, border tokens, and eyebrow tokens, defeating part of the dark-mode override.

Required end state:

- `--color-surface-inner` has a concrete light-mode value.
- Base `:root` tokens are declared before the `@media (prefers-color-scheme: dark)` override.
- Dark mode overrides all surface, text, border, amber, and error tokens that are mode-sensitive.
- Component CSS uses tokens for remaining light-mode hardcoded surfaces/text.
- No component-level `background: rgba(255, 255, 255, ...)` remains unless there is a comment explaining why it is intentionally mode-invariant.
- No component-level `color: #1f2527`, `color: #433d37`, or `color: #6f675c` remains where a text token should be used.

### 3. URL action buttons still suppress focus outline

The generic `.url-action-button:focus-visible` rule currently sets `outline: none`. Variant-specific focus rules adjust shadows, but none restores the shared focus ring.

Required end state:

- `.url-action-button:focus-visible` uses `outline: var(--focus-ring)` and `outline-offset: var(--focus-offset)`.
- Hover transforms and focus transforms may remain, but the outline must not be suppressed.
- Static audit of all `:focus-visible` blocks confirms every visible focus outline uses the shared focus tokens.

### 4. Runtime refresh wipes a legitimately loaded remote-planner model list

Fix 3 correctly stopped synthesizing a verified model list from saved settings. However, `persistRemotePlannerConnection()` then calls `refreshRuntimePanels()`, and `runtime-refresh.ts` always applies:

```ts
availableModels: [],
loadedModelsEndpoint: null,
```

This does not recreate the original false-success bug, but it erases a real model-list state that was loaded successfully immediately before save.

Required end state:

- Runtime refresh must **not synthesize** a model list from saved settings.
- Runtime refresh may **preserve** an already-loaded model list if all of these are true:
  - the current panel state already has `availableModels.length > 0`,
  - `loadedModelsEndpoint` is non-null,
  - the refreshed backend `remote_planner_settings.base_url` equals the current `loadedModelsEndpoint`.
- If the endpoint differs, the loaded list must be cleared.
- Manual model-only saves must still not mark the list as loaded.

### 5. Final validation and memory entry were not actually completed

The previous TODO marks major tasks `DONE`, but the final checklist was left unchecked and `memory.md` does not contain a real UIUX Fix 3/Fix 4 completion entry.

Required end state:

- Validation commands run in the developer environment.
- Static audit commands run and produce expected results or documented intentional exceptions.
- `memory.md` is updated with a real UTC timestamp from `date -u`, never a fabricated timestamp.

## Non-goals

Do not re-implement the entire UI.
Do not remove the existing successful PTT setup-only layout.
Do not remove manual planner model entry.
Do not replace the structured frontend error work.
Do not bundle local fonts unless there is time; documented network-font dependency is acceptable for this pass.

## Implementation constraints

- Prefer narrow patches over large refactors.
- Every new user-visible error path must be accessible: use `role="alert"` for errors or `role="status"` for non-error notices as appropriate.
- Do not use `console.error()` as the only failure path for any user-triggered action.
- Do not add broad fallback behavior that makes an unverified state look verified.
- Preserve typed backend error details where already implemented.

## Acceptance summary

Fix 4 is complete only when all of these are true:

1. External-link open failure is visible on every settings subpage and can be dismissed.
2. `rg -n "setUrlInputPanelStoreState\(|setUrlInputPanelState\(" src/panel-state-setters.ts` does not show external-link error routing to URL input state.
3. `--color-surface-inner` has a concrete light-mode value.
4. The dark-mode block appears after the base `:root` block.
5. The known hardcoded light text/surface colors are tokenized or explicitly justified.
6. `.url-action-button:focus-visible` uses the shared focus ring.
7. Runtime refresh preserves a genuinely loaded planner model list when the endpoint still matches, but does not synthesize a list from saved settings.
8. Full validation gate passes in a real dev environment.
9. `memory.md` has a real UTC completion entry.
