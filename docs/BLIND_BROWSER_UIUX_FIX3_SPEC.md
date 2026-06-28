# Blind Browser UI/UX Fix 3 Specification

## Purpose

This spec defines the next hardening pass for `blind_browser` after the static review of the current implementation against `UIUX_IMPROVEMENTS2_TODO.md`.

The goal is not to add new user-facing scope. The goal is to make the previous UI/UX pass truthful, complete, and maintainable:

1. Remove false-success UI states.
2. Finish the incomplete accessibility and visual-system work.
3. Fix quiet/silent failure paths.
4. Preserve structured backend error data in the frontend.
5. Add tests that prevent Claude Code from reintroducing these regressions.

This pass should be treated as a stabilization and correctness pass, not a redesign.

---

## Current known state

The current code already has several good pieces:

- Settings subpage lede paragraphs exist in `src/app-shell.tsx`.
- Settings subpage breadcrumbs exist.
- Settings subpage toolbar behavior mostly follows Option A: subpage back goes to settings overview; workspace back is not also shown.
- Settings card `unconfigured` wording has been changed to `Setup required` in `src/app-shell-nav.tsx`.
- Planner settings now allow manual model entry.
- Endpoint `onBlur` can trigger model loading.
- Inline error dismiss/retry affordances exist in shared settings controls.
- Status panel has coarse `Loading page…` and `Working…` states.

The current code still has important defects:

- Saving a manually typed planner model falsely marks the endpoint's model list as loaded/up to date.
- Runtime refresh synthesizes `availableModels` from a saved model and marks `loadedModelsEndpoint`, even when no model list was loaded.
- Dark mode is partial and still leaves hardcoded light-mode colors.
- The page background still contains the blue radial gradient that the TODO required removing.
- Teal/blue colors still appear in UI states where the TODO wanted the warm green/amber palette.
- Focus rings are not unified.
- The disabled push-to-talk setup state still renders the large disabled talk button before the setup guidance.
- `unwrapToolResult()` throws away structured backend `ToolError` metadata.
- External link failures are console-only.
- At least one non-CSS source/test fixture file exceeds the 600-line file-size target.

---

## Non-negotiable implementation principles

### 1. No false verification states

The UI must never imply that a remote model list has been loaded or verified unless `listRemotePlannerModels()` actually succeeded for the current endpoint.

A saved model name is not the same thing as a loaded model list.

Bad behavior to remove:

```ts
availableModels: [result.model, ...currentAvailable]
loadedModelsEndpoint: result.base_url
```

when this occurs after saving a manually typed model rather than after model-list loading.

### 2. No quiet or silent failures

Do not convert backend errors into generic frontend errors if structured metadata exists.

Do not hide user-action failures in `console.error()` only.

Do not silently drop runtime/capture failures unless the failure is provably non-actionable and separately observable through diagnostics.

### 3. Accessibility fixes are functional requirements

Focus-ring consistency, dark-mode contrast, disabled-state hierarchy, and visible error feedback are not cosmetic in this app. This is a blind/low-vision browser. Keyboard focus, status feedback, and readable contrast are product-critical.

### 4. Prefer small, testable changes

Do not rewrite the app shell or settings architecture. Patch the existing files with focused state, rendering, CSS, and test updates.

### 5. Do not mark TODO items done based on intent

A TODO item is done only when:

- the target behavior exists in code,
- the old broken behavior is removed,
- tests cover the regression where practical,
- the validation gate passes in the developer environment.

---

## Target behavior by area

## A. Remote planner setup truthfulness

### Required user behavior

The AI assistant settings panel must support three distinct states:

1. **No endpoint/model saved yet**
   - User can enter endpoint.
   - User can manually enter a model.
   - User can load/refresh model list.
   - Save is allowed once endpoint and model are non-empty.

2. **Manual model saved, model list not loaded**
   - Saved model remains visible in the text input.
   - UI must not say model list is up to date.
   - UI must warn: `Model list may be outdated — refresh to verify available models.`
   - `availableModels` should be empty unless a real model-list call succeeded.
   - `loadedModelsEndpoint` should be `null` unless a real model-list call succeeded.

3. **Model list loaded for current endpoint**
   - Dropdown is populated from `listRemotePlannerModels()` result.
   - Freshness label can say `Model list up to date`.
   - Manual text input remains available below the dropdown.
   - Refresh button remains available and is labeled `Refresh model list`.

### State invariant

`loadedModelsEndpoint !== null` means: "The model list in `availableModels` came from a successful `listRemotePlannerModels()` call for this exact endpoint."

`loadedModelsEndpoint` must not be set by:

- `persistRemotePlannerConnection()` unless the list was already loaded for that endpoint before saving.
- `refreshRuntimePanels()` / `runtime-refresh.ts`.
- any path that only reads saved settings.

### Acceptance criteria

- Saving a manually typed model with no loaded models leaves `availableModels: []` and `loadedModelsEndpoint: null`.
- Refreshing runtime state from a saved manual model leaves `availableModels: []` and `loadedModelsEndpoint: null`.
- Loading models for an endpoint is the only normal path that sets `availableModels` and `loadedModelsEndpoint` together.
- Stale/fresh labels use explicit language, not only a colored dot.
- The manual save path never blocks on model loading.

---

## B. CSS design tokens and color cleanup

### Required behavior

The visual system should use a coherent warm parchment/green/amber/red palette.

The following must be removed from component CSS unless explicitly justified in a comment:

- Blue radial page-background gradient.
- Teal UI accents such as `#1c5871` and `#24404f`.
- Hardcoded green variants where a semantic token exists.
- Hardcoded white/light surfaces that break dark mode.
- Component-local focus colors that diverge from the shared focus token.

### Token strategy

Keep the current Tailwind `@theme` approach if desired, but add explicit application-level aliases in `:root` so component CSS can use stable semantic names:

```css
:root {
  color-scheme: light;

  --surface-base: var(--color-surface-base);
  --surface-mid: var(--color-surface-mid);
  --surface-card: var(--color-surface-card);
  --surface-card-inner: rgba(255, 255, 255, 0.68);

  --text-primary: var(--color-text-primary);
  --text-secondary: var(--color-text-secondary);
  --text-muted: var(--color-text-muted);
  --text-label: #6f675c;

  --green-primary: var(--color-green-primary);
  --green-active: var(--color-green-active);
  --green-dark: var(--color-green-dark);
  --green-fresh: var(--color-green-fresh);

  --amber-primary: var(--color-amber-primary);
  --amber-active: var(--color-amber-active);
  --amber-light: rgba(122, 87, 39, 0.12);

  --error-primary: var(--color-error-primary);
  --error-active: var(--color-error-active);
  --error-dark: var(--color-error-dark);
  --error-light: rgba(139, 52, 42, 0.09);

  --focus-ring: 2px solid var(--green-active);
  --focus-offset: 2px;

  --card-border: 1px solid rgba(123, 98, 70, 0.16);
  --inner-card-border: 1px solid rgba(123, 98, 70, 0.12);
  --eyebrow-color: #7b6246;
  --btn-nav-start: #5a5048;
  --btn-nav-end: #7a6860;
  --btn-nav-shadow: rgba(90, 80, 72, 0.18);

  background: linear-gradient(180deg, var(--surface-base) 0%, var(--surface-mid) 100%);
  color: var(--text-primary);
}
```

### Acceptance criteria

- `src/styles.css` no longer contains `rgba(35, 103, 161`.
- `src/styles.css` no longer contains `#1c5871`.
- `src/styles.css` no longer contains `#24404f`.
- Interactive green buttons use `var(--green-primary)` / `var(--green-active)` / `var(--green-dark)`.
- Slider `accent-color` values are green, not teal.
- Confirmation transport errors use a warm neutral/amber treatment, not blue-teal.

---

## C. Focus-ring unification

### Required behavior

Every keyboard-focusable element in workspace and settings must use the same focus outline:

```css
outline: var(--focus-ring);
outline-offset: var(--focus-offset);
```

Elements may keep component-specific hover shadows/transforms, but the visible outline must be consistent.

### Required targets

Audit and fix at minimum:

- `.shell-toolbar-action:focus-visible`
- `.settings-subpage-card:focus-visible`
- `.settings-subpage-back:focus-visible`
- `.push-to-talk-button:focus-visible`
- `.url-input-control:focus-visible`
- `.url-action-button:focus-visible`
- `.settings-control-select:focus-visible`
- `.settings-control-button:focus-visible`
- `.confirmation-button:focus-visible`
- `.status-toggle-button:focus-visible`
- `.ptt-setup-banner-button:focus-visible`
- `.settings-model-missing-button:focus-visible`

### Acceptance criteria

- `rg -n ":focus-visible" src/styles.css` shows all focus-visible blocks using the shared token unless a comment explicitly explains an exception.
- URL input no longer has amber focus outline.
- PTT focus ring no longer uses a larger 3px/4px variant.
- Setup banner button has an explicit focus-visible rule.

---

## D. Dark mode completion

### Required behavior

Dark mode must be token-driven and readable across all major panels.

The app must not leave white input islands or dark-on-dark text inside dark surfaces.

### Required token overrides

Use a full dark-mode override, not just three surface variables:

```css
@media (prefers-color-scheme: dark) {
  :root {
    color-scheme: dark;

    --surface-base: #1a1712;
    --surface-mid: #141210;
    --surface-card: rgba(36, 32, 26, 0.92);
    --surface-card-inner: rgba(48, 43, 36, 0.80);

    --text-primary: #f0ead8;
    --text-secondary: #c8bfae;
    --text-muted: #a99b86;
    --text-label: #b4a48e;

    --card-border: 1px solid rgba(180, 155, 110, 0.14);
    --inner-card-border: 1px solid rgba(180, 155, 110, 0.10);
    --eyebrow-color: #b4a48e;

    --amber-primary: #c49a50;
    --amber-active: #e0b86a;
    --amber-light: rgba(196, 154, 80, 0.14);

    --error-primary: #e17b6f;
    --error-active: #ff9588;
    --error-dark: #ffb4aa;
    --error-light: rgba(225, 123, 111, 0.14);

    background: linear-gradient(180deg, var(--surface-base) 0%, var(--surface-mid) 100%);
  }
}
```

### Component-level requirements

Use tokens for:

- `.voice-status-strip`
- `.status-card dt` / `dd`
- `.audio-control-label` / `.audio-control-value`
- `.url-input-control`
- `.settings-control-select`
- `.settings-control-button-secondary`
- `.confirmation-button-reject`
- warning/error banners

### Acceptance criteria

- Dark mode has no unreadable dark text on dark backgrounds.
- Dark mode has no glaring white cards or inputs unless intentionally high-contrast and still tokenized.
- `color-scheme: light` is not left as the only root color scheme.
- `pnpm test:ui` passes.
- Manual visual pass covers workspace, settings overview, all settings subpages, confirmation/error states, and setup-required PTT state.

---

## E. Push-to-talk setup-required layout

### Required behavior

When voice input is not configured, the setup guidance is the primary UI.

Current bad hierarchy:

1. Large disabled PTT circle.
2. Hint text.
3. Setup banner below.

Required hierarchy:

1. Setup banner with explanation and `Open settings` action.
2. Optional muted/smaller disabled PTT placeholder below, or no PTT button at all.

Prefer the simplest accessible implementation: when `!state.enabled`, render only the setup banner and a subdued explanatory hint. Do not render the large disabled circular talk button.

### Acceptance criteria

- Disabled setup state HTML renders `.ptt-setup-banner` before any `.push-to-talk-button`, or does not render `.push-to-talk-button` at all.
- `data-push-to-talk-button="true"` is absent in setup-required state if using the preferred implementation.
- Enabled idle state still renders the normal PTT button.
- Tests cover both setup-required and enabled idle states.

---

## F. Preserve structured frontend errors

### Required behavior

When a backend command returns `ToolResult<T>` with `error`, the frontend must preserve the full `ToolError` object:

- `code`
- `message`
- `retryable`
- `details`

`classifyInvokeFailure()` should classify this as `kind: "tool-error"`, not a generic transport error.

### Required behavior change

Current problem:

```ts
if (result.error) {
  throw new Error(result.error.message);
}
```

Required replacement:

```ts
export class FrontendToolError extends Error {
  constructor(public readonly toolError: ToolError) {
    super(toolError.message);
    this.name = "FrontendToolError";
  }
}
```

Then `unwrapToolResult()` must throw `new FrontendToolError(result.error)`.

`parseToolError()` must recognize `FrontendToolError`.

### Acceptance criteria

- Unit test proves `unwrapToolResult()` preserves structured errors.
- Unit test proves `classifyInvokeFailure(errorFromUnwrap)` returns `kind: "tool-error"`.
- Existing confirmation error metadata still renders backend error code and retryability.

---

## G. Visible external-link failures

### Required behavior

If opening an external URL fails, the user must see a visible error message in the current UI context.

Current problem:

```ts
void openExternalUrl({ url }).catch((error) => {
  console.error("Failed to open external link.", error);
});
```

This is silent to the user.

### Preferred implementation

Create a small user-visible global or settings guidance error state. The minimal acceptable implementation is to set the current settings guidance panel error if that panel owns the link, or add a global app-shell status/error if the link can appear in multiple settings panels.

Do not block the first fix pass on a new toast system. A visible panel error is enough.

### Acceptance criteria

- Failure to open a settings external link renders visible copy such as: `Could not open the external link. Copy the URL and open it manually.`
- The error includes either the URL or a way for the user to recover.
- Test covers a rejected `openExternalUrl()` call.

---

## H. Audio capture lock failure observability

### Required behavior

Do not silently ignore persistent audio capture buffer lock failures.

Because this code is in an audio callback, do not log on every callback. Instead use a one-shot flag or deferred error observation.

### Acceptable approaches

- Use an `AtomicBool` to record that a buffer lock failed, and after capture/session completion surface a warning or error.
- Emit one throttled tracing warning if the codebase already has tracing available.
- If mutex poisoning is impossible by construction, replace the `if let Ok` with `expect()` plus a comment explaining why crashing is preferable to silently losing audio input. Use this only if truly safe.

### Acceptance criteria

- No bare `if let Ok(...) { ... }` lock write in the audio callback without an `else` path or comment explaining why loss is impossible/non-actionable.
- Tests or code comments document the intended failure behavior.

---

## I. File-size target

The previous TODO says no file over 600 lines except CSS. Current static audit found `src-tauri/src/commands/tests/fixtures/mock_executor_impl.rs` over the target.

### Required behavior

Decide one of these explicitly:

1. Split the fixture into smaller modules.
2. Exempt test fixtures by updating the rule in docs and documenting why.

Do not leave the target silently violated.

### Acceptance criteria

- Either no non-CSS tracked source/test file exceeds 600 lines, or docs explicitly say generated/test fixture files are exempt.
- If exempted, update the final audit checklist so future reviewers do not repeatedly flag the same intentional exception.

---

## Validation gate

Run after each P0 group and before final handoff:

```bash
source ./fix-node-version.sh
pnpm lint
pnpm test:ui
pnpm build
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
```

If any command cannot run in the current environment, document the exact command, failure reason, and what still needs to be run by the user.

---

## Manual visual audit

After validation passes, manually inspect:

- Workspace view in light mode.
- Workspace view in dark mode.
- PTT enabled idle state.
- PTT setup-required state.
- URL loading state.
- Status panel `Working…` state.
- Settings overview.
- AI assistant setup subpage with no endpoint.
- AI assistant setup subpage with manual saved model but no loaded list.
- AI assistant setup subpage after successful model-list load.
- Voice output setup.
- Voice input setup.
- Advanced settings.
- Confirmation panel with approve/reject buttons.
- Transport error state.
- Tool/backend error state.
- Keyboard tab order and focus ring consistency across all interactive elements.

---

## Done definition

This pass is complete only when:

- P0 and P1 TODOs are complete.
- Tests cover the high-risk regressions.
- Full validation gate passes.
- `memory.md` has a UTC timestamped entry summarizing the pass.
- No TODO item is marked done solely because a UI string changed while the state invariant remains wrong.
