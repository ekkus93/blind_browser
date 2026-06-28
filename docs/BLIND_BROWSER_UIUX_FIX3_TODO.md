# Blind Browser UI/UX Fix 3 TODO

## How to use this file

Work top-to-bottom by priority. Do not skip P0 tasks. Commit after each completed task group if validation passes.

Status key: `PENDING` · `IN PROGRESS` · `DONE` · `BLOCKED`

Priority key:

- `P0`: correctness or quiet-failure bug that can mislead users or hide failures.
- `P1`: accessibility/UX requirement from the previous TODO that is incomplete.
- `P2`: cleanup, documentation, or lower-risk hardening.

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

## P0.1 — Fix false “model list loaded/up to date” planner state

**Status:** PENDING  
**Files:**

- `src/planner-actions.ts`
- `src/runtime-refresh.ts`
- `src/settings-panels/planner.tsx`
- `src/confirmation-panel-settings-planner.test.mjs`
- possibly `src/panel-types.ts` / `src/panel-state.ts` only if you decide to add a new explicit state field

### Problem

The current code treats a manually saved model as if it came from a loaded endpoint model list.

Bad current behavior:

- `persistRemotePlannerConnection()` inserts `result.model` into `availableModels`.
- `persistRemotePlannerConnection()` sets `loadedModelsEndpoint` to `result.base_url` after save.
- `runtime-refresh.ts` reconstructs `availableModels` from the saved model and sets `loadedModelsEndpoint`.

This creates a quiet false-success state. The UI can say the model list is up to date even when no model-list request succeeded.

### Required invariant

`loadedModelsEndpoint` means: a model list was successfully loaded by `loadRemotePlannerModels()` for this exact endpoint.

Only a successful `listRemotePlannerModels()` result should create a fresh model-list state.

### Subtasks

#### P0.1.1 — Patch `persistRemotePlannerConnection()`

Replace the post-save state update in `src/planner-actions.ts` with logic that preserves the loaded list only if it was already loaded for the saved endpoint.

Suggested snippet:

```ts
const currentPlannerState = getPanelStates().remotePlannerPanelState;
const currentAvailable = currentPlannerState.availableModels;
const modelsWereLoadedForSavedEndpoint = currentPlannerState.loadedModelsEndpoint === result.base_url
  && currentAvailable.length > 0;

setRemotePlannerPanelState({
  profileName: result.profile_name,
  baseUrl: result.base_url,
  model: result.model,
  availableModels: modelsWereLoadedForSavedEndpoint ? currentAvailable : [],
  loadedModelsEndpoint: modelsWereLoadedForSavedEndpoint ? result.base_url : null,
  isSavingConnection: false,
  error: null,
});
```

Important: do **not** synthesize `[result.model, ...currentAvailable]` unless the list was already real and loaded for the same endpoint.

#### P0.1.2 — Patch `runtime-refresh.ts`

In `applyAgentStateToPanels`, do not reconstruct a loaded model list from saved settings.

Suggested replacement:

```ts
availableModels: [],
loadedModelsEndpoint: null,
```

Keep the saved model itself:

```ts
model: agentState.remote_planner_settings.model,
```

The UI should show the saved model in the manual text input, but it should not claim the endpoint model list is verified.

#### P0.1.3 — Improve freshness copy in `planner.tsx`

Use explicit state labels:

- Fresh: `Model list up to date`
- Stale/unverified: `Model list may be outdated — reload to refresh`
- Button: `Refresh model list`

Suggested local variables:

```ts
const endpointMatchesLoadedModels = baseUrlTrimmed.length > 0
  && state.loadedModelsEndpoint === state.baseUrl
  && state.availableModels.length > 0;
const modelsAreFresh = endpointMatchesLoadedModels;
const hasLoadedModels = state.availableModels.length > 0;
const modelsNotLoadedForEndpoint = baseUrlTrimmed.length > 0 && !modelsAreFresh;
```

Suggested label block:

```tsx
<span className="settings-model-freshness-indicator" aria-hidden="true">
  <span className={`settings-status-light ${modelsAreFresh ? "settings-status-light-fresh" : "settings-status-light-stale"}`} />
  <span className="settings-model-freshness-label">
    {modelsAreFresh ? "Model list up to date" : "Model list may be outdated — reload to refresh"}
  </span>
</span>
<span className="sr-only">
  {modelsAreFresh
    ? "Model list is loaded for the current endpoint"
    : "Model list has not been loaded for the current endpoint"}
</span>
```

Suggested button copy:

```tsx
{state.isLoadingModels
  ? <><span className="btn-spinner" aria-hidden="true" />Loading models...</>
  : "Refresh model list"}
```

#### P0.1.4 — Keep manual model save enabled

Do not reintroduce the old save gate requiring models to be loaded.

Save should be disabled only when:

- connection is busy,
- profile is missing,
- endpoint is empty,
- model text is empty.

#### P0.1.5 — Add regression tests

Add tests in `src/confirmation-panel-settings-planner.test.mjs` that render these states:

1. Manual saved model, no loaded list:
   - `model: "gpt-manual"`
   - `availableModels: []`
   - `loadedModelsEndpoint: null`
   - assert manual input value exists,
   - assert stale/unverified copy exists,
   - assert no dropdown option implies verified list.

2. Loaded list for endpoint:
   - `availableModels: ["gpt-test"]`
   - `loadedModelsEndpoint` equals `baseUrl`
   - assert `Model list up to date`.

Suggested test skeleton:

```js
test("manual planner model does not render as a verified loaded model list", () => {
  const html = renderSettingsRemotePlannerPanel({
    profileName: "openai-default",
    provider: "OpenAI",
    baseUrl: "https://api.example.com/v1",
    model: "gpt-manual",
    availableModels: [],
    loadedModelsEndpoint: null,
    isLoadingModels: false,
    isSavingConnection: false,
    isResettingConnection: false,
    isConfirmingReset: false,
    apiKeyReference: "Environment variable: OPENAI_API_KEY",
    organizationReference: null,
    project: null,
    temperatureMilli: 200,
    maxOutputTokens: 1024,
    timeoutMs: 30000,
    apiKeyDraft: "",
    isSavingApiKey: false,
    isTestingApiKey: false,
    apiKeyTestMessage: null,
    error: null,
  });

  assert.match(html, /value="gpt-manual"/);
  assert.match(html, /Model list may be outdated/);
  assert.doesNotMatch(html, /Model list up to date/);
  assert.doesNotMatch(html, /<select[^>]*data-remote-planner-model-select/);
});
```

### Acceptance checks

- `rg -n "availableModels: \[result\.model|loadedModelsEndpoint: result\.base_url" src/planner-actions.ts src/runtime-refresh.ts` returns no false verification path.
- Manual save still works without loaded models.
- Fresh model-list state only appears after successful `loadRemotePlannerModels()`.

---

## P0.2 — Preserve structured backend `ToolError` in frontend API wrappers

**Status:** PENDING  
**Files:**

- `src/api/errors.ts`
- `src/tauri-api.test.mjs`
- any tests that currently assume generic `Error` from `unwrapToolResult()`

### Problem

`unwrapToolResult()` currently throws `new Error(result.error.message)`, which discards `code`, `retryable`, and `details`.

This makes structured backend errors look like generic transport errors.

### Subtasks

#### P0.2.1 — Add a typed frontend error wrapper

Patch `src/api/errors.ts`.

Suggested snippet:

```ts
export class FrontendToolError extends Error {
  constructor(public readonly toolError: ToolError) {
    super(toolError.message);
    this.name = "FrontendToolError";
  }
}
```

#### P0.2.2 — Teach `parseToolError()` to recognize the wrapper

Suggested patch at the top of `parseToolError()`:

```ts
export function parseToolError(error: unknown): ToolError | null {
  if (error instanceof FrontendToolError) {
    return error.toolError;
  }

  if (!isRecord(error)) {
    return null;
  }

  // existing structured-object parsing follows...
}
```

#### P0.2.3 — Update `unwrapToolResult()`

Suggested replacement:

```ts
export function unwrapToolResult<T>(result: ToolResult<T>): T {
  if (result.ok && result.data !== null) {
    return result.data;
  }

  if (result.error) {
    throw new FrontendToolError(result.error);
  }

  throw new Error("The runtime returned an invalid tool result.");
}
```

#### P0.2.4 — Add tests

In `src/tauri-api.test.mjs`, add a test that proves structured errors survive unwrap/classify.

Suggested test:

```js
test("unwrapToolResult preserves structured backend tool errors", () => {
  const result = {
    ok: false,
    tool_name: "GetAgentState",
    request_id: "req-test",
    timestamp_ms: 0,
    data: null,
    error: {
      code: "runtime_busy",
      message: "Runtime is busy.",
      retryable: true,
      details: { phase: "planner" },
    },
    warnings: [],
    observations: [],
  };

  assert.throws(
    () => tauriApi.unwrapToolResult(result),
    tauriApi.FrontendToolError,
  );

  try {
    tauriApi.unwrapToolResult(result);
    assert.fail("expected unwrapToolResult to throw");
  } catch (error) {
    assert.deepEqual(tauriApi.classifyInvokeFailure(error), {
      kind: "tool-error",
      toolError: {
        code: "runtime_busy",
        message: "Runtime is busy.",
        retryable: true,
        details: { phase: "planner" },
      },
    });
  }
});
```

### Acceptance checks

- `classifyInvokeFailure()` returns `tool-error` for errors thrown by `unwrapToolResult()`.
- Confirmation/backend error UI still receives code/retryable/details where applicable.

---

## P0.3 — Make external-link open failures visible to users

**Status:** PENDING  
**Files:**

- `src/panel-state-setters.ts`
- likely `src/panel-types.ts`
- likely `src/panel-state.ts`
- likely `src/settings-panels/shared-controls.tsx` or the relevant settings panel renderer
- tests as appropriate

### Problem

`openExternalLink()` catches errors and only logs to the console.

Current behavior:

```ts
void openExternalUrl({ url }).catch((error) => {
  console.error("Failed to open external link.", error);
});
```

For the target user, this is silent.

### Preferred minimal fix

Add a visible global/settings error state. If the code already has a settings guidance panel error, reuse it. If not, add a small `externalLinkError` string to app shell/settings state and render it near settings content.

Suggested helper:

```ts
function describeExternalLinkFailure(url: string, error: unknown): string {
  const detail = error instanceof Error && error.message.trim().length > 0
    ? ` ${error.message}`
    : "";
  return `Could not open the external link. Copy this URL and open it manually: ${url}.${detail}`;
}
```

Suggested state update pattern in `openExternalLink`:

```ts
export function openExternalLink(url: string) {
  void openExternalUrl({ url }).catch((error) => {
    console.error("Failed to open external link.", error);
    setAppShellStateOrRelevantPanelState({
      externalLinkError: describeExternalLinkFailure(url, error),
    });
  });
}
```

Use the actual setter available in this codebase. Do not leave the fallback as console-only.

### Acceptance checks

- A rejected `openExternalUrl()` call causes visible HTML containing `Could not open the external link`.
- The user can copy the failed URL from visible text or an accessible control.
- Existing external-link happy path remains unchanged.

---

## P1.1 — Complete CSS token cleanup and remove teal/blue remnants

**Status:** PENDING  
**Files:**

- `src/styles.css`

### Problem

The previous TODO says the color system phase is done, but the current stylesheet still contains blue/teal remnants and hardcoded component colors.

### Subtasks

#### P1.1.1 — Add application-level semantic tokens

Keep the existing `@theme` block if it is needed for Tailwind, but add semantic aliases in `:root` and use them in component CSS.

Suggested `:root` replacement shape:

```css
:root {
  color-scheme: light;
  font-family: var(--font-sans);

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

#### P1.1.2 — Remove the blue radial background

Delete this layer:

```css
radial-gradient(circle at top left, rgba(35, 103, 161, 0.18), transparent 32%),
```

Root background should be only:

```css
background: linear-gradient(180deg, var(--surface-base) 0%, var(--surface-mid) 100%);
```

#### P1.1.3 — Replace teal and hardcoded text colors

Minimum replacements:

```css
.voice-status-strip {
  background: var(--surface-card);
  color: var(--text-secondary);
}

.voice-status-strip[data-voice-state="processing"] .voice-status-dot {
  background: var(--amber-primary);
}

.audio-control-label,
.audio-control-value {
  color: var(--text-primary);
}

.audio-control-input {
  accent-color: var(--green-primary);
}
```

Replace confirmation transport blue treatment:

```css
.confirmation-error-transport {
  background: var(--amber-light);
  border-color: rgba(122, 87, 39, 0.22);
  color: var(--amber-active);
}
```

Replace approve button hardcoded green:

```css
.confirmation-button-approve {
  background: var(--green-primary);
  color: #f6fbf8;
}

.confirmation-button-approve:hover,
.confirmation-button-approve:focus-visible {
  background: var(--green-dark);
}

.confirmation-button-approve:disabled {
  background: color-mix(in srgb, var(--green-primary) 62%, var(--surface-card));
}
```

If `color-mix()` is not acceptable for target browsers/webview, use a tokenized static fallback and comment it.

#### P1.1.4 — Replace inline card surfaces where practical

Use:

```css
background: var(--surface-card);
```

or:

```css
background: var(--surface-card-inner);
```

instead of repeated `rgba(255, 252, 247, ...)` and `rgba(255, 255, 255, 0.68)` in component rules.

Do not blindly replace every `rgba()` shadow/border. Replace surface backgrounds and inner-card backgrounds first.

### Acceptance checks

Run:

```bash
rg -n "rgba\(35, 103, 161|#1c5871|#24404f" src/styles.css
```

Expected: no matches.

Run:

```bash
rg -n "#1f6b57|#185745|#4d8578" src/styles.css
```

Expected: no matches unless commented as intentional compatibility fallback.

---

## P1.2 — Complete focus-ring unification

**Status:** PENDING  
**Files:**

- `src/styles.css`

### Problem

Focus indicators still differ by component. The URL input uses amber focus, PTT uses a larger ring, and some buttons lack explicit focus-visible rules.

### Subtasks

#### P1.2.1 — Normalize all focus-visible outline declarations

Use this for every focusable element unless a comment explains a rare exception:

```css
outline: var(--focus-ring);
outline-offset: var(--focus-offset);
```

Suggested replacements:

```css
.push-to-talk-button:focus-visible {
  outline: var(--focus-ring);
  outline-offset: var(--focus-offset);
  transform: translateY(-1px);
  box-shadow: 0 18px 30px rgba(31, 127, 92, 0.3);
}

.url-input-control:focus-visible,
.settings-control-select:focus-visible {
  outline: var(--focus-ring);
  outline-offset: var(--focus-offset);
  border-color: rgba(41, 88, 63, 0.3);
}

.url-action-button:focus-visible,
.settings-control-button:focus-visible,
.confirmation-button:focus-visible,
.status-toggle-button:focus-visible,
.ptt-setup-banner-button:focus-visible,
.settings-model-missing-button:focus-visible {
  outline: var(--focus-ring);
  outline-offset: var(--focus-offset);
}
```

Be careful not to accidentally remove existing hover/focus transforms/shadows that are useful. The requirement is shared outline, not identical full visual treatment.

#### P1.2.2 — Add missing setup-banner focus state

Add:

```css
.ptt-setup-banner-button:focus-visible {
  outline: var(--focus-ring);
  outline-offset: var(--focus-offset);
}
```

### Acceptance checks

Run:

```bash
rg -n ":focus-visible" src/styles.css
```

Manually inspect every block. Every visible outline must use `var(--focus-ring)` and `var(--focus-offset)`.

---

## P1.3 — Finish dark mode

**Status:** PENDING  
**Files:**

- `src/styles.css`

### Problem

Dark mode currently overrides only three surface variables. Many hardcoded light backgrounds and dark text colors remain.

### Subtasks

#### P1.3.1 — Replace the existing partial dark-mode block

Use a complete token override after `:root` or directly after the root token declarations.

Suggested block:

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

#### P1.3.2 — Tokenize inputs and secondary buttons

Suggested replacements:

```css
.url-input-control,
.settings-control-select {
  background: var(--surface-card-inner);
  color: var(--text-primary);
}

.settings-control-button-secondary {
  background: var(--surface-card-inner);
  color: var(--text-primary);
  border: var(--inner-card-border);
}

.confirmation-button-reject {
  background: var(--surface-card-inner);
  color: var(--amber-active);
  border: 1px solid rgba(122, 87, 39, 0.18);
}
```

#### P1.3.3 — Tokenize remaining text values

At minimum:

```css
.status-card dt {
  color: var(--text-label);
}

.status-card dd {
  color: var(--text-primary);
}

.settings-group-eyebrow {
  color: var(--eyebrow-color);
}

.confirmation-prompt {
  color: var(--text-primary);
}
```

### Acceptance checks

Run:

```bash
rg -n "background: rgba\(255, 255, 255|color: #1f2527|color: #2f3436|color: #433d37|color-scheme: light" src/styles.css
```

Expected: no problematic component-level matches. `color-scheme: light` may remain inside `:root` only if the dark block overrides it with `color-scheme: dark`.

Manual check dark mode.

---

## P1.4 — Fix push-to-talk setup-required visual hierarchy

**Status:** PENDING  
**Files:**

- `src/confirmation-panels/push-to-talk.tsx`
- `src/styles.css`
- `src/confirmation-panel-core.test.mjs`

### Problem

When `state.enabled === false`, the large disabled circular PTT button still renders before the setup banner. The setup guidance remains visually secondary.

### Preferred implementation

Early-return a setup-only layout when PTT is disabled.

Suggested replacement near the start of `renderPushToTalkPanelNode()` after `buttonLabel` calculation or before it:

```tsx
if (!state.enabled) {
  return (
    <section className="push-to-talk-panel push-to-talk-panel-setup-required" aria-label="Talk control setup required">
      <div className="ptt-setup-banner" role="status" aria-live="polite">
        <p className="ptt-setup-banner-message">
          Voice input isn't set up yet. Open settings to configure your microphone and speech providers.
        </p>
        <button
          type="button"
          className="ptt-setup-banner-button"
          data-ptt-open-settings="true"
          onClick={handlers?.onOpenSettings}
        >
          Open settings
        </button>
      </div>
      {state.lastError
        ? <span className="sr-only" role="alert">{state.lastError}</span>
        : null}
      {state.lastError
        ? <p className="push-to-talk-error" aria-hidden="true">{state.lastError}</p>
        : null}
    </section>
  );
}
```

Then remove the old bottom `{!state.enabled ? ... : null}` banner block from the normal enabled layout.

Suggested CSS:

```css
.push-to-talk-panel-setup-required {
  align-items: stretch;
}

.push-to-talk-panel-setup-required .ptt-setup-banner {
  margin: 0;
}
```

### Test update

Update `renders setup banner when push-to-talk is disabled, hides it when enabled` in `src/confirmation-panel-core.test.mjs`.

Suggested assertions:

```js
assert.match(disabledHtml, /ptt-setup-banner/);
assert.match(disabledHtml, /Voice input isn&#39;t set up yet/);
assert.match(disabledHtml, /data-ptt-open-settings="true"/);
assert.doesNotMatch(disabledHtml, /data-push-to-talk-button="true"/);
assert.doesNotMatch(enabledHtml, /ptt-setup-banner/);
assert.match(enabledHtml, /data-push-to-talk-button="true"/);
```

### Acceptance checks

- Setup-required state has no large disabled talk button.
- Enabled state remains unchanged.
- Tests pass.

---

## P1.5 — Improve planner loading indicator placement and copy

**Status:** PENDING  
**Files:**

- `src/settings-panels/planner.tsx`
- `src/styles.css`
- `src/confirmation-panel-settings-planner.test.mjs`

### Problem

Auto-loading exists on endpoint blur, but the visible spinner is on the model-list button, not next to the endpoint field as requested. Copy still says `Load models` in places, and freshness copy is too terse.

### Subtasks

#### P1.5.1 — Add endpoint-adjacent loading copy

Inside the endpoint field group, render loading text when `state.isLoadingModels`:

```tsx
{state.isLoadingModels ? (
  <span className="settings-inline-loading" role="status" aria-live="polite">
    <span className="btn-spinner" aria-hidden="true" /> Loading models...
  </span>
) : null}
```

Place it close to the endpoint input, not only inside the model-list button.

#### P1.5.2 — Rename manual button

Use `Refresh model list`, not `Load models`, except maybe for first-load copy if you intentionally choose dynamic text.

### CSS snippet

```css
.settings-inline-loading {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
  color: var(--text-muted);
  font-size: 0.9rem;
  font-weight: 600;
}
```

### Acceptance checks

- While loading models, HTML contains `Loading models...` near endpoint section.
- Button copy says `Refresh model list` when not loading.

---

## P2.1 — Bundle fonts locally or document network-font dependency explicitly

**Status:** PENDING  
**Files:**

- `index.html`
- `src/styles.css`
- possible `src/fonts/*`
- docs if local bundling is deferred

### Problem

Fonts are loaded via Google Fonts. The fallback is documented, which is better than silent fallback, but Tauri apps are often used offline.

### Preferred fix

Bundle IBM Plex Sans and Fraunces locally, then replace Google Fonts links with `@font-face` declarations.

If you do not bundle fonts in this pass, add an explicit docs note:

- Fonts require network access on first load.
- Offline fallback is intentional.
- Local bundling is deferred.

### Acceptance checks

- Either local font files are present and loaded, or docs explicitly document the network dependency.
- No one can confuse the fallback as a bug or silent missing asset.

---

## P2.2 — Make audio capture lock failure observable

**Status:** PENDING  
**Files:**

- `src-tauri/src/asr/capture.rs`
- tests if practical

### Problem

The audio callback currently uses an `if let Ok(mut guard) = buffer.lock()` pattern that silently drops audio if the lock fails.

### Subtasks

#### P2.2.1 — Add an explicit failure path

Do not log every callback. Use one-shot logging or an atomic flag.

Conceptual snippet:

```rust
let lock_failed = Arc::new(AtomicBool::new(false));
let lock_failed_for_callback = Arc::clone(&lock_failed);

// inside callback
match buffer.lock() {
    Ok(mut guard) => guard.extend(input.iter().copied()),
    Err(_) => {
        lock_failed_for_callback.store(true, Ordering::Relaxed);
    }
}
```

After capture, check the flag and surface a warning/error through the existing result path if possible.

#### P2.2.2 — Document why the chosen behavior is safe

If you choose not to surface this as a user-visible error, add a comment explaining why a one-shot diagnostic is enough.

### Acceptance checks

- No silent bare ignored lock failure remains.
- Behavior is documented.

---

## P2.3 — Resolve or document the >600-line non-CSS fixture

**Status:** PENDING  
**Files:**

- `src-tauri/src/commands/tests/fixtures/mock_executor_impl.rs`
- docs/final audit checklist, if exempting fixtures

### Problem

The previous TODO target says no non-CSS file over 600 lines. Current audit found `mock_executor_impl.rs` above that target.

### Options

Option A — split fixture:

- Move fixture helpers into separate modules.
- Keep each file under 600 lines.

Option B — explicit exemption:

- Update docs/checklist to say test fixtures are exempt from the 600-line target when they are intentionally centralized and generated-like.
- Add a short comment at the top of the fixture explaining why it is exempt.

### Acceptance checks

Run:

```bash
find . -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.tsx" -o -name "*.mjs" -o -name "*.css" \) \
  -not -path "*/node_modules/*" \
  -not -path "*/dist/*" \
  -not -path "*/target/*" \
  | xargs wc -l | sort -rn | head -20
```

Then either:

- no non-CSS/non-exempt file exceeds 600 lines, or
- the exemption is documented.

---

## P2.4 — Add final validation and memory entry

**Status:** PENDING  
**Files:**

- `memory.md`
- optionally updated docs/TODO status file

### Subtasks

#### P2.4.1 — Run validation gate

```bash
source ./fix-node-version.sh
pnpm lint
pnpm test:ui
pnpm build
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
```

#### P2.4.2 — Run static searches

```bash
rg -n "rgba\(35, 103, 161|#1c5871|#24404f" src/styles.css
rg -n "availableModels: \[result\.model|loadedModelsEndpoint: result\.base_url" src/planner-actions.ts src/runtime-refresh.ts
rg -n ":focus-visible" src/styles.css
```

Confirm output is clean or every remaining hit has a justified comment.

#### P2.4.3 — Update `memory.md`

Use a real UTC timestamp:

```bash
date -u +"%Y-%m-%dT%H:%M:%SZ"
```

Suggested memory entry format:

```md
- 2026-XX-XXTXX:XX:XXZ — Completed UIUX Fix 3 hardening: fixed truthful remote planner model-list state, preserved frontend ToolError metadata, completed CSS token/focus/dark-mode cleanup, made PTT setup state primary, and ran full validation gate.
```

Do not fabricate the timestamp. Run the command.

---

## Suggested commit sequence

1. `fix(planner): keep manual model separate from loaded model list`
2. `fix(api): preserve structured tool errors in frontend unwraps`
3. `fix(ui): surface external link failures`
4. `style(ui): complete color tokens and remove teal remnants`
5. `style(ui): unify keyboard focus rings`
6. `style(ui): complete dark mode token coverage`
7. `fix(ui): make push-to-talk setup guidance primary`
8. `test(ui): cover planner freshness and PTT setup regressions`
9. `docs: resolve final audit and memory entry`

---

## Final done checklist

- [ ] Manual planner model save does not mark model list as loaded.
- [ ] Runtime refresh does not synthesize a verified model list.
- [ ] Structured backend `ToolError` survives `unwrapToolResult()`.
- [ ] External-link failures are visible to the user.
- [ ] Blue radial background is gone.
- [ ] Teal remnants `#1c5871` and `#24404f` are gone.
- [ ] Focus rings use shared `--focus-ring` / `--focus-offset`.
- [ ] Dark mode covers surfaces, inputs, text, warnings, errors, and secondary buttons.
- [ ] PTT setup-required state renders setup guidance as primary and does not show the large disabled talk button.
- [ ] Planner loading and stale/fresh model-list copy are explicit.
- [ ] Font dependency is bundled locally or documented as network-dependent.
- [ ] Audio capture lock failure is observable or explicitly justified.
- [ ] 600-line file-size target is satisfied or fixture exemption is documented.
- [ ] Full validation gate passes.
- [ ] `memory.md` updated with a real UTC timestamp.
