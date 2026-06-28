# UI/UX Improvements 2 TODO — blind_browser

Based on the UI/UX review performed in June 2026 (second pass, post-REFACTOR5).  
**Status key:** PENDING · IN PROGRESS · DONE

---

## Goal

Address the visual inconsistencies, interaction friction, missing features, and accessibility
gaps identified in the second UI/UX review. Organized from highest-impact to lower-impact
so each phase can be validated and committed independently.

---

## Implementation strategy

Work in this order:

1. **Color system** — establish CSS design tokens and eliminate the fragmented palette.
   Every subsequent phase depends on this foundation.
2. **Focus ring unification** — single focus indicator system; functional accessibility fix.
3. **Font loading verification** — confirm IBM Plex Sans and Fraunces actually render.
4. **Dark mode** — `prefers-color-scheme: dark` support.
5. **Remote planner UX** — reduce the 4-step setup friction.
6. **Settings page UX** — descriptive context, navigation clarity, disabled-state guidance.
7. **Feedback and progress** — error dismiss, planner progress indicator, loading states.
8. **Final validation** — full suite, visual audit checklist.

---

## Validation gate (run after every phase)

```bash
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
pnpm lint
pnpm test:ui
pnpm build
```

---

## Phase 1 — CSS design token system ✅ DONE (absorbed into TAILWIND1)

**Problem:** The codebase has five near-identical greens (`#29583f`, `#1f7f5c`, `#1f6b57`,
`#1d4a30`, `#2d9b57`), two teals (`#1c5871`, `#24404f`), and inline hex values scattered
across 1555 lines of CSS with no semantic naming. The blue radial gradient in the page
background is visually inconsistent with the warm parchment palette everywhere else. The
confirmation panel has a blue-teal tint that appears nowhere else.

### 1.1 Define CSS variables for all color tokens

- [x] In `src/styles.css`, expand the `:root` block to include a complete token set:
  ```css
  :root {
    /* Brand greens */
    --green-primary:  #29583f;
    --green-active:   #1f7f5c;
    --green-dark:     #1d4a30;
    --green-fresh:    #2d9b57;

    /* Amber / warm accent */
    --amber-primary:  #7a5727;
    --amber-active:   #5a3e10;
    --amber-light:    rgba(122, 87, 39, 0.12);

    /* Error / danger */
    --error-primary:  #8b342a;
    --error-active:   #b04736;
    --error-dark:     #6c1010;
    --error-light:    rgba(139, 52, 42, 0.09);

    /* Surfaces */
    --surface-base:   #f7f4ec;
    --surface-mid:    #ece4d5;
    --surface-card:   rgba(255, 252, 247, 0.9);
    --surface-card-inner: rgba(255, 255, 255, 0.68);

    /* Text */
    --text-primary:   #1d1a16;
    --text-secondary: #433d37;
    --text-muted:     #6b5e52;
    --text-label:     #6f675c;

    /* Focus */
    --focus-ring:     2px solid var(--green-active);
    --focus-offset:   2px;

    /* Borders */
    --card-border:    1px solid rgba(123, 98, 70, 0.16);
    --inner-card-border: 1px solid rgba(123, 98, 70, 0.12);
    --eyebrow-color:  #7b6246;

    /* Nav button */
    --btn-nav-start:  #5a5048;
    --btn-nav-end:    #7a6860;
    --btn-nav-shadow: rgba(90, 80, 72, 0.18);
  }
  ```

### 1.2 Replace inline hex green/teal values with tokens

- [x] Search for all occurrences of `#29583f` and replace with `var(--green-primary)`.
- [ ] Search for all occurrences of `#1f7f5c` and replace with `var(--green-active)`.
- [ ] Search for all occurrences of `#1f6b57`, `#1f6b57`, `#185745` (approve button hover)
      — these are the "third green". Decide: either map to `--green-primary`/`--green-active`,
      or define a `--green-confirm` token. Use one choice consistently for the approve button.
- [ ] Search for all occurrences of `#1d4a30` and replace with `var(--green-dark)`.
- [ ] Search for all occurrences of `#2d9b57` and replace with `var(--green-fresh)`.
- [ ] Search for all occurrences of `#7a5727`, `#7b6246` (eyebrow), `#5a3e10` (amber text)
      and replace with amber tokens.
- [ ] Search for all occurrences of `#8b342a`, `#b04736`, `#6c1010` and replace with error tokens.
- [ ] Search for all occurrences of `#1c5871` (slider accent, confirmation panel tint)
      and replace with `var(--green-primary)` — there is no reason for teal on sliders.
- [ ] Search for all occurrences of `#24404f` (audio control label color) — this is a dark
      teal used only here; replace with `var(--text-primary)`.
- [ ] Replace all inline `rgba(255, 252, 247, ...)` surface values with `--surface-card`.
- [ ] Replace all inline `rgba(255, 255, 255, 0.68)` inner card values with `--surface-card-inner`.

### 1.3 Remove the blue radial gradient from the page background

- [ ] In `:root` background, remove the `radial-gradient(circle at top left, rgba(35, 103, 161, 0.18), ...)` layer.
- [ ] Keep only: `background: linear-gradient(180deg, var(--surface-base) 0%, var(--surface-mid) 100%);`
- [ ] Verify visually that the page background looks intentional without the blue accent.

### 1.4 Replace the confirmation panel's blue-teal tint with a warm amber tint

- [ ] In `.confirmation-panel`, change the background from:
  ```css
  background:
    linear-gradient(135deg, rgba(28, 88, 113, 0.08), rgba(255, 252, 247, 0.96)),
    rgba(255, 252, 247, 0.96);
  ```
  to:
  ```css
  background:
    linear-gradient(135deg, var(--amber-light), var(--surface-card)),
    var(--surface-card);
  ```
- [ ] Verify `.confirmation-error-transport` (which uses its own blue-teal tint) still reads
      clearly — adjust border color if needed to stay in the warm palette while remaining
      visually distinct from tool errors.

### 1.5 Run validation gate

- [ ] `pnpm build` — confirm no CSS parse errors.
- [ ] `pnpm test:ui` — confirm all 97 JS tests pass (visual output hasn't changed).
- [ ] Visual check: open the built app and compare all panels against screenshots or expected states.

---

## Phase 2 — Focus ring unification ✅ DONE (absorbed into TAILWIND1)

**Problem:** Four different focus indicator styles exist across the app: green on toolbar
buttons, amber on the URL input, a different green shade on settings selects, and blue on
confirmation buttons. Keyboard navigation produces a visually inconsistent experience and
makes it harder for low-vision users to track focus.

### 2.1 Audit all focus-visible rules in styles.css

- [ ] Search for all `:focus-visible` blocks and list the `outline` color/style used in each.
- [ ] Identify every unique outline value.

### 2.2 Replace all :focus-visible outlines with the unified token

- [ ] `.shell-toolbar-action:focus-visible` — replace outline with `var(--focus-ring)`.
- [ ] `.settings-subpage-card:focus-visible` — replace outline with `var(--focus-ring)`.
- [ ] `.settings-subpage-back:focus-visible` — replace outline with `var(--focus-ring)`.
- [ ] `.push-to-talk-button:focus-visible` — already uses `#1f7f5c`; replace with `var(--focus-ring)`.
- [ ] `.url-input-control:focus-visible` — currently amber (`rgba(122, 87, 39, 0.28)`);
      replace with `var(--focus-ring)` and update `border-color` to `rgba(41, 88, 63, 0.3)`.
- [ ] `.url-action-button:focus-visible` — add `outline: var(--focus-ring); outline-offset: var(--focus-offset);`.
- [ ] `.settings-control-select:focus-visible` — replace with `var(--focus-ring)`.
- [ ] `.settings-control-button:focus-visible` — add `outline: var(--focus-ring); outline-offset: var(--focus-offset);`.
- [ ] `.confirmation-button:focus-visible` — currently `rgba(28, 88, 113, 0.45)` (blue);
      replace with `var(--focus-ring)`.
- [ ] `.status-toggle-button:focus-visible` — replace `rgba(123, 98, 70, 0.3)` with `var(--focus-ring)`.
- [ ] `.ptt-setup-banner-button:focus-visible` — add `:focus-visible` rule if missing.
- [ ] `.settings-model-missing-button:focus-visible` — add `:focus-visible` rule if missing.

### 2.3 Unify slider accent-color

- [ ] `.audio-control-input` uses `accent-color: #1c5871` (teal); change to `var(--green-primary)`.
- [ ] `.settings-control-input` already uses `#29583f`; change to `var(--green-primary)`.

### 2.4 Run validation gate

- [ ] `pnpm test:ui` — all tests pass.
- [ ] Manual keyboard-tab through the workspace and settings views: every focused element
      should show the same green outline.

---

## Phase 3 — Font loading verification and fix ✅ DONE

**Problem:** `styles.css` declares `font-family: "IBM Plex Sans"` and `font-family: "Fraunces"`
as preferred fonts, but there is no `@font-face`, `@import`, or `<link>` confirmed to be
loading these. If fonts aren't loaded, the app silently falls back to Segoe UI / Georgia.

### 3.1 Check index.html for font loading

- [ ] Read `index.html` (or `src-tauri/index.html`) to verify whether Google Fonts `<link>`
      tags or local font imports exist.
- [ ] Check `src/main.ts` — does it import a font CSS file?
- [ ] Check `vite.config.ts` for any font bundling.

### 3.2 Fix font loading if missing

- [ ] If fonts are not loaded: add to `index.html`:
  ```html
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Fraunces:opsz,wght@9..144,400;9..144,600&family=IBM+Plex+Sans:wght@400;500;600;700&display=swap" rel="stylesheet">
  ```
- [ ] Alternatively, download and bundle the fonts locally under `src/fonts/` and add
      `@font-face` declarations to `styles.css` — preferred for a Tauri app that may run
      offline.
- [ ] If fonts were already loading correctly, document the mechanism in a comment in `styles.css`.

### 3.3 Run validation gate

- [ ] `pnpm build` — no errors.
- [ ] Launch the app and visually confirm headings use Fraunces serif (distinctive curved S,
      optical size variation) and body text uses IBM Plex Sans (distinctive monospaced-influenced
      design with straight terminals on letters like "l" and "1").

---

## Phase 4 — Dark mode ✅ DONE (absorbed into TAILWIND1)

**Problem:** The app has no `@media (prefers-color-scheme: dark)` support. Dark mode is
particularly important for this app's target audience (users with low vision or light
sensitivity). The warm parchment palette maps cleanly to a dark equivalent.

### 4.1 Define dark-mode token overrides

- [ ] After the `:root` block in `styles.css`, add a `@media (prefers-color-scheme: dark)`
      block that overrides the surface, text, and border tokens:
  ```css
  @media (prefers-color-scheme: dark) {
    :root {
      color-scheme: dark;
      background: linear-gradient(180deg, #1a1712 0%, #141210 100%);

      --surface-base:       #1a1712;
      --surface-mid:        #141210;
      --surface-card:       rgba(36, 32, 26, 0.92);
      --surface-card-inner: rgba(48, 43, 36, 0.80);

      --text-primary:       #f0ead8;
      --text-secondary:     #c8bfae;
      --text-muted:         #8c7e6a;
      --text-label:         #9a8c78;

      --card-border:        1px solid rgba(180, 155, 110, 0.14);
      --inner-card-border:  1px solid rgba(180, 155, 110, 0.10);
      --eyebrow-color:      #9a8c78;

      /* Keep green primary unchanged — it reads well on dark */
      /* Lighten amber slightly for dark backgrounds */
      --amber-primary:      #c49a50;
      --amber-active:       #e0b86a;
    }
  }
  ```

### 4.2 Audit hardcoded light-mode colors in component-specific rules

- [ ] After Phase 1 token replacement, any remaining hardcoded hex values in component rules
      will need dark-mode overrides. Work through these component by component:
  - [ ] `.voice-status-strip` — `color: #433d37` → use `var(--text-secondary)`.
  - [ ] `.settings-group-eyebrow` — `color: #7b6246` → use `var(--eyebrow-color)`.
  - [ ] `.settings-group h2` — no color set (inherits body); confirm this works in dark.
  - [ ] `.status-card dt` — `color: #6f675c` → use `var(--text-label)`.
  - [ ] `.status-card dd` — `color: #1f2527` → use `var(--text-primary)`.
  - [ ] `.audio-control-label`, `.audio-control-value` — `color: #24404f` → use `var(--text-primary)`.
  - [ ] `.confirmation-button-reject` — `background: #efe6d8; color: #5b2a1f` → needs
        dark override (dark background, light warm text).
  - [ ] `.url-input-control` — `background: rgba(255, 255, 255, 0.92)` → needs dark override.
  - [ ] `.settings-control-select` — same issue.
  - [ ] All `.settings-control-card-readonly` rules — verify muted appearance still works.
  - [ ] Error/warning banners (`.ptt-setup-banner`, `.settings-model-missing-warning`,
        `.settings-reset-confirm-row`) — verify contrast in dark.

### 4.3 Dark-mode button gradient adjustments

- [ ] `.push-to-talk-button` gradient (`#29583f → #1f7f5c`) will look acceptable on dark
      backgrounds; verify there is sufficient contrast on the dark surface.
- [ ] `.url-open-button`, `.url-read-button` — same green gradient; verify.
- [ ] `.url-stop-button` (red gradient) — verify on dark.
- [ ] `.settings-control-button` — verify green gradient on dark surface.
- [ ] `.settings-control-button-secondary` (`#d9e5dd → #eef5ef`, light green) — this will
      look washed out on dark backgrounds. Override to a dark-appropriate secondary style
      (e.g., dark border, transparent background with light text).

### 4.4 Test dark mode

- [ ] `pnpm test:ui` — all tests pass (tests render to static markup, not affected by color).
- [ ] Switch OS to dark mode and launch the built app. Check every panel:
  - Workspace view: PTT button, URL input, status panel
  - Settings overview: all 4 settings cards
  - Each settings subpage

---

## Phase 5 — Remote planner setup UX

**Problem:** Configuring the AI assistant requires a rigid 4-step sequence (enter URL → load
models → select model → save) with an error gate that blocks saving if models haven't been
loaded. This forces a non-obvious workflow on users who already know their model ID.

### 5.1 Allow free-text model entry alongside the dropdown

- [ ] In `src/settings-panels/planner.ts`, locate the model selection control.
- [ ] Change the model selection from a `<select>` to a combined control:
  - When `state.availableModels` is non-empty: show a `<select>` dropdown populated with
    `state.availableModels`, but also include a text input below it labeled
    "Or enter a model name manually".
  - When `state.availableModels` is empty (models not yet loaded): show only the text input
    with placeholder "e.g. gpt-4o".
- [ ] Update `RemotePlannerPanelState` in `src/panel-types.ts` if a new `modelDraft` field
      is needed to track the free-text value independently of the selected value.
- [ ] Update `persistRemotePlannerConnection` in `src/planner-actions.ts` to use the manual
      text input value when no model is selected from the dropdown.
- [ ] Remove the "Load models for the current endpoint before saving" error gate — it is no
      longer necessary now that manual model entry is available. Replace it with a softer
      warning: "You haven't loaded models for this endpoint yet — make sure the model name
      is correct before saving."

### 5.2 Auto-trigger model loading when the endpoint URL is stable

- [ ] In the planner settings panel, when the `baseUrl` input loses focus (`onBlur`), if the
      URL is non-empty and differs from `state.loadedModelsEndpoint`, automatically trigger
      `loadRemotePlannerModels()`.
- [ ] Show a subtle inline spinner ("Loading models...") next to the endpoint field during
      auto-load.
- [ ] If the auto-load fails (bad URL, no network), show a dismissible inline warning without
      blocking the rest of the form — the user can still save manually.
- [ ] Do NOT auto-load on every keystroke — only on `onBlur` to avoid hammering the API.
- [ ] Update the planner panel render function in `src/settings-panels/planner.ts` to wire
      the `onBlur` handler.
- [ ] Update the panel state if a `isAutoLoadingModels` flag is needed.

### 5.3 Improve the stale model indicator copy

- [ ] Currently the stale indicator is a colored dot with no label explaining what "stale" means.
- [ ] In `src/settings-panels/planner.ts`, locate the freshness indicator.
- [ ] Replace or augment the dot with a text label:
  - Fresh: "Model list up to date" (or suppress the label entirely when fresh — no news is good news)
  - Stale: "Model list may be outdated — reload to refresh"
- [ ] Add a "Reload" button or link next to the stale indicator that triggers `loadRemotePlannerModels()`.
- [ ] Remove the separate "Load models" button if the auto-load on blur (5.2) covers the same need,
      or keep it as a manual fallback labeled "Refresh model list".

### 5.4 Run validation gate

- [ ] `pnpm test:ui` — all tests pass.
- [ ] Manual walkthrough: open AI assistant settings with a fresh config, enter an endpoint,
      tab away, confirm models auto-load, select one, save. Verify no friction steps remain.

---

## Phase 6 — Settings page UX and navigation clarity

### 6.1 Add descriptive context paragraphs to each settings subpage

**Problem:** Each settings subpage opens with only a bare heading. New users have no
guidance on what they are configuring or what credentials/values they need.

- [ ] In `src/app-shell.ts`, add a `<p className="lede">` after each settings subpage `<h2>`:

  **AI assistant setup** (view: `planner`):
  > "The AI assistant interprets your voice commands and decides what to do. It requires an
  > OpenAI-compatible API endpoint and key. If you're using OpenAI, the endpoint is
  > `https://api.openai.com/v1`. For local models via Ollama, use `http://localhost:11434`."

  **Voice output setup** (view: `tts`):
  > "Voice output converts the assistant's text responses to speech. Choose a local model
  > for offline use or a remote service for higher quality voices."

  **Voice input setup** (view: `asr`):
  > "Voice input converts your speech to text. Choose a local Whisper model for offline use
  > or a remote transcription service."

  **Advanced settings** (view: `runtime`):
  > "Model management, confirmation behavior, and OCR settings. Most users won't need to
  > change these."

- [ ] Style the lede paragraphs with `className="lede"` (already defined in `styles.css` as
      max-width 60ch, 1.05rem, 1.6 line-height).

### 6.2 Clarify the dual back-navigation pattern in the toolbar

**Problem:** In workspace view, the gear icon goes to settings. In settings overview, the
same position shows a back arrow that goes to workspace. In settings subpages, there are
two back arrows (one for subpage → overview, one for overview → workspace) but the first
is the same button that was the gear icon. This is confusing.

- [ ] In `src/app-shell.ts`, audit the toolbar in each app state:
  - Workspace: gear icon → settings (current behavior is fine)
  - Settings overview: back arrow → workspace (current behavior is fine)
  - Settings subpage: two arrows are present — clarify visually
- [ ] In settings subpages, add a breadcrumb text label below the toolbar (or inline with it)
      showing the current location, e.g.:
      ```
      ← Settings › AI assistant setup
      ```
      This can be a simple `<p className="settings-breadcrumb">Settings › AI assistant setup</p>`
      rendered inside each subpage hero section.
- [ ] Add `.settings-breadcrumb` to `styles.css`:
  ```css
  .settings-breadcrumb {
    font-size: 0.84rem;
    color: var(--text-muted);
    margin: 0 0 8px;
  }
  ```
- [ ] Remove the duplicate `renderSettingsSubpageBackButton` from the toolbar on subpages —
      the app-view back arrow (which goes to workspace) and the subpage back arrow (which goes
      to settings overview) currently both appear. Clarify the intended behavior:
  - **Option A:** Only show the subpage back button (→ settings overview). Remove the
    workspace back arrow on subpages. The user gets to workspace by going overview → workspace.
  - **Option B:** Show both but label them differently with `title` attributes and distinct
    icon sizes. Document the chosen approach.

### 6.3 Make the PTT disabled state prominent when setup is incomplete

**Problem:** When TTS/ASR is not configured, the PTT button renders as a large disabled
circle. The setup banner that explains what to do appears *below* the button — secondary
to the thing the user wants to click.

- [ ] In `src/confirmation-panels/push-to-talk.ts`, detect the "not configured / setup
      required" state — this is when `state.enabled === false` and a setup banner message is present.
- [ ] When in the setup state, restructure the panel layout:
  - Show the setup banner *first* (above the PTT button or instead of it).
  - Render the PTT button below the banner, smaller, or replace it with a muted
    placeholder so it doesn't dominate the visual hierarchy.
- [ ] Alternatively: if `!state.enabled` and setup is incomplete, hide the PTT button
      entirely and render only the setup banner with the "Go to settings" button.
- [ ] Update the CSS to support whichever layout is chosen.
- [ ] Update the push-to-talk render tests to cover the setup-state layout.

### 6.4 Settings card status badges — improve "Not configured" label

- [ ] In `src/app-shell-nav.ts`, the `SETTINGS_STATUS_LABEL` map has:
  ```typescript
  unconfigured: "Not configured",
  ```
- [ ] Change `unconfigured` label to `"Setup required"` — it's more actionable than "Not
      configured" and pairs better with the status color.
- [ ] Update any tests that assert on the "Not configured" text string.

### 6.5 Run validation gate

- [ ] `pnpm test:ui` — all tests pass, including any updated string assertions.
- [ ] Manual walkthrough: open each settings subpage and read the lede. Navigate forward
      and backward; confirm breadcrumb renders correctly.

---

## Phase 7 — Feedback, progress, and error handling

### 7.1 Add explicit dismiss for inline errors

**Problem:** Inline errors in panels (API key test failure, model load failure, save
failure) clear only when the next successful action runs. Users who hit an error have no
explicit way to dismiss it or understand that they should try again.

- [ ] In `src/styles.css`, add a dismiss button style:
  ```css
  .panel-error-dismiss {
    appearance: none;
    background: none;
    border: none;
    padding: 0 0 0 8px;
    font: inherit;
    font-size: 0.84rem;
    font-weight: 700;
    color: inherit;
    cursor: pointer;
    opacity: 0.7;
    text-decoration: underline;
  }
  .panel-error-dismiss:hover { opacity: 1; }
  ```
- [ ] In each panel render function that shows an `error` field, add a small "Dismiss" link
      (or "×" button) alongside the error text.
- [ ] Wire the dismiss to a panel state update that sets `error: null`. This will require a
      new action or a generic `clearPanelError` dispatch in `src/panel-state-setters.ts`.
- [ ] Affected panels: remote planner, remote TTS, remote ASR, audio controls, TTS provider,
      TTS model, TTS voice, ASR provider, model management, OCR threshold, confirmation settings,
      status panel, URL input panel.
- [ ] Update tests for panels that render errors to assert the dismiss button is present.

### 7.2 Add a "Retry" affordance next to dismissible errors where retrying makes sense

- [ ] For errors on actions that can be safely retried (API key test, model load, save
      settings), show a "Try again" button inline next to the dismiss link.
- [ ] "Try again" should re-invoke the same action that failed (e.g., `testConfiguredRemotePlannerApiKey`).
- [ ] Do NOT show "Try again" for errors where the user needs to change input first
      (e.g., "Enter an endpoint before loading models" — there's nothing to retry without
      changing the field).

### 7.3 Add a planner step progress indicator

**Problem:** During multi-step planner execution (OpenUrl → ExtractPageModel → ReadPage),
the UI is visually idle between steps. A low-vision user waiting for a page to be read
gets no cue that the system is working.

- [ ] In `src/panel-types.ts`, check whether `StatusPanelState` already includes a
      "planner busy" or "current step" field. If not, consider adding:
      `plannerBusy: boolean; plannerCurrentStep: string | null;`
- [ ] In `src/settings-panels/workspace.ts`, update `renderStatusPanelNode` to show a
      subtle busy indicator when `plannerBusy` is true:
      - A small spinning dot or animated text ("Working…") in the status panel header area
      - Do not use a modal overlay — it should be subtle and non-blocking
- [ ] Wire the Tauri event system to update `plannerBusy` / `plannerCurrentStep` as steps
      execute. Check `src/main.ts` for where Tauri events are consumed.
- [ ] If wiring Tauri events is out of scope for this phase, at minimum add a CSS animation
      class that can be applied to the status panel during known-busy states (URL opening,
      extraction running, etc.) and apply it from the `isOpening` / `isReading` flags
      already present on `UrlInputPanelState`.
- [ ] Update tests as needed.

### 7.4 Show a "loading page" state in the status panel when a URL is opening

**Problem:** When `UrlInputPanelState.isOpening` is true, the URL button shows a spinner,
but the status panel (which shows page title, region, transcript) shows stale data from the
previous page. There is no visible transition between "navigating" and "page ready."

- [ ] In `src/settings-panels/workspace.ts`, detect when the URL input panel is in an
      `isOpening` state (thread state through from parent, or use a Redux selector).
- [ ] When `isOpening`, show a muted "Loading page…" placeholder in the status panel's
      page title and region slots instead of the stale previous values.
- [ ] Clear this placeholder when a new page title arrives via the runtime refresh.
- [ ] Update tests to cover the "loading" placeholder state.

### 7.5 Run validation gate

- [ ] `pnpm test:ui` — all tests pass.
- [ ] Manual walkthrough: trigger an API key test error, confirm the dismiss button appears.
      Click dismiss, confirm the error clears. Open a URL, confirm the loading state appears
      in the status panel.

---

## Phase 8 — Final validation and audit

### 8.1 Run the full validation suite

- [ ] `source ./fix-node-version.sh`
- [ ] `pnpm lint`
- [ ] `pnpm test:ui`
- [ ] `pnpm build`
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml --all-features`

### 8.2 Visual audit checklist

Walk through the app and verify each item:

**Color system**
- [ ] No stray hex green or teal values — every interactive element uses the token system.
- [ ] Blue radial gradient is gone from the page background.
- [ ] Confirmation panel has a warm amber tint, not blue-teal.
- [ ] Error states (red) are visually distinct from warning states (amber).

**Focus**
- [ ] Tab through workspace: PTT button, URL input field, Open button, secondary nav buttons.
      Every focused element shows the same green outline.
- [ ] Tab through settings overview: all four settings cards show the same outline.
- [ ] Tab through a settings subpage: inputs, selects, buttons all show the same outline.
- [ ] Sliders (volume, speed) show green accent color, not teal.

**Typography**
- [ ] Headings render in Fraunces (distinctive serif — visible curve on `s`, optical size).
- [ ] Body text renders in IBM Plex Sans (monospaced-influenced — straight `l`, distinct `1`).

**Dark mode** (switch OS to dark)
- [ ] Workspace is readable with dark surfaces and light text.
- [ ] Error states (red) remain high-contrast in dark.
- [ ] All buttons remain visible and legible.
- [ ] Settings pages: no cards with white/light backgrounds that become invisible.

**Remote planner setup**
- [ ] Enter endpoint URL, tab away → model list auto-loads.
- [ ] Manually type a model name → can save without loading model list.
- [ ] Stale indicator shows actionable text, not just a colored dot.

**Settings navigation**
- [ ] Each settings subpage shows a lede paragraph explaining what to configure.
- [ ] Breadcrumb appears in subpage hero section.
- [ ] Back navigation is unambiguous at each level.
- [ ] Settings card "Not configured" → "Setup required" wording.

**PTT setup state**
- [ ] When voice input is not configured, the setup banner is visually primary.
- [ ] Disabled PTT button does not dominate the layout.

**Error handling**
- [ ] Every inline error has a visible dismiss button.
- [ ] API key test errors include a "Try again" affordance.
- [ ] Dismissing an error clears it without requiring a full page refresh.

**Planner progress**
- [ ] Opening a URL shows "Loading page…" in the status panel during navigation.
- [ ] Multi-step planner execution shows some visible activity indicator.

### 8.3 Verify file sizes haven't regressed

- [ ] `find . -type f \( -name "*.rs" -o -name "*.ts" -o -name "*.mjs" -o -name "*.css" \) -not -path "*/node_modules/*" -not -path "*/dist/*" -not -path "*/target/*" | xargs wc -l | sort -rn | head -20`
- [ ] Target: no file over 600 lines (CSS is exempt from this target due to necessary length).

### 8.4 Update `memory.md`

- [ ] Run `date -u +"%Y-%m-%dT%H:%M:%SZ"` and add an entry to `memory.md` summarizing
      what was completed.

---

## Suggested commit sequence

```
Commit 1:  Phase 1 — CSS design token system
Commit 2:  Phase 2 — Focus ring unification
Commit 3:  Phase 3 — Font loading verification and fix
Commit 4:  Phase 4 — Dark mode
Commit 5:  Phase 5 — Remote planner setup UX
Commit 6:  Phase 6 — Settings page UX and navigation clarity
Commit 7:  Phase 7 — Feedback, progress, and error handling
Commit 8:  Phase 8 — Final validation and audit
```

---

## Issue reference

| Phase | Issue from review |
|-------|------------------|
| 1 | Fragmented green/teal palette; blue radial gradient; confirmation panel blue tint |
| 2 | Inconsistent focus ring colors; teal slider accent |
| 3 | Font loading unverified |
| 4 | No dark mode |
| 5 | Remote planner 4-step friction; stale indicator unclear |
| 6 | Settings subpages have no context copy; dual back-arrow confusion; disabled PTT layout; "Not configured" label |
| 7 | No error dismiss; no "retry" affordance; no planner progress indicator; no loading state in status panel |
| 8 | Final validation |
