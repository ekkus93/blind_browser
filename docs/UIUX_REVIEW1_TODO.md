# UI/UX Review TODO — blind_browser

Based on `docs/UIUX_REVIEW1.md`.  
**Status key:** PENDING · IN PROGRESS · DONE

---

## Goal

Fix every issue identified in the UI/UX review so the app's visual design, information hierarchy, and interaction model match its core purpose: a **voice-first desktop browser for vision-impaired users**.

---

## Implementation strategy

Work in phases so each phase can be validated and committed independently. Later phases build on earlier ones; do not start a later phase until the prior one passes lint, tests, and a visual check.

1. Fix foundational issues first (fonts, color system, placeholder removal).
2. Redesign the workspace layout around voice-first hierarchy.
3. Add the persistent status strip.
4. Clean up the confirmation panel.
5. Clean up settings.
6. Polish and final audit.

---

## Phase 1 — Fix foundational issues

These are quick, isolated fixes that unblock everything else. Do them first.

### 1.1 Fix font loading — PENDING

The `Fraunces` and `IBM Plex Sans` fonts are referenced in CSS but never loaded. The app silently falls back to Georgia and Segoe UI.

- [ ] Add a `<link rel="preconnect">` and `<link rel="stylesheet">` to `index.html` for Google Fonts, loading both `Fraunces` (display weights 400, 700, 900) and `IBM Plex Sans` (weights 400, 600, 700).
- [ ] Alternatively, download and self-host the font files under `src/assets/fonts/` and add `@font-face` declarations at the top of `src/styles.css`.
- [ ] Verify both fonts render correctly in a `pnpm dev` run.
- [ ] Confirm fallback stack still looks acceptable if fonts fail to load.

### 1.2 Remove the developer placeholder note from the confirmation panel — PENDING

A code comment made it into production: `"The frontend can now present approve or reject controls against this state and send the user response back through the Tauri confirmation command."`

- [ ] Locate the `<p className="confirmation-note">` element in `src/confirmation-panels/confirmation.ts`.
- [ ] Delete it entirely — there is no user-facing value in this text.
- [ ] Update any snapshot or render tests that assert on this text.

### 1.3 Establish a unified panel border color token — PENDING

Each panel currently has a different accent color for its border with no semantic meaning. Replace with a single neutral token.

- [ ] Choose one border color for all content cards. The existing warm neutral `rgba(123, 98, 70, 0.16)` is a good starting point.
- [ ] In `src/styles.css`, replace all per-panel border color values with a CSS custom property, e.g. `--card-border: rgba(123, 98, 70, 0.16)`.
- [ ] Update `.url-input-panel`, `.status-panel`, `.audio-controls-panel`, `.settings-panel`, and `.confirmation-panel` to all use `var(--card-border)`.
- [ ] Keep the confirmation panel's blue-tinted background gradient — that's a background, not a border, and adds useful visual distinction for a high-stakes action.

### 1.4 Consolidate URL action button colors — PENDING

Five adjacent buttons each have an unrelated gradient color. Collapse to two semantic color roles.

- [ ] Define two semantic button color roles:
  - **Destructive** (Stop): keep the existing red gradient.
  - **Neutral action** (Open, Read, Previous, Next): choose one consistent dark neutral, e.g. the existing green `linear-gradient(135deg, #29583f, #1f7f5c)` or a dark neutral. Do not use 4 unrelated colors.
- [ ] Update `.url-open-button`, `.url-read-button`, `.url-previous-button`, `.url-next-button` in `src/styles.css` to share the neutral action color.
- [ ] Keep `.url-stop-button` red.
- [ ] Verify the button row still looks clear and distinguishable after consolidation.

### 1.5 Unify eyebrow text color — PENDING

Eyebrow labels use different accent colors per panel with no semantic meaning.

- [ ] Add a CSS custom property `--eyebrow-color` with a single muted tone (the warm brown `#7b6246` is a good pick — it's already used in several places).
- [ ] Update `.settings-panel-eyebrow` (currently green `#29583f`), `.audio-controls-eyebrow` (currently blue `#1c5871`), and any other per-panel eyebrow color overrides to all use `var(--eyebrow-color)`.
- [ ] Keep `.confirmation-eyebrow` distinct only if there is a genuine semantic reason; otherwise unify it too.

---

## Phase 2 — Workspace redesign

This is the highest-impact change. The workspace layout must be rebuilt around voice-first hierarchy.

### 2.1 Audit the current workspace layout — PENDING

Before touching code, document the exact current DOM structure and CSS layout of the workspace view.

- [ ] Note the current grid: `grid-template-columns: minmax(0, 1fr) 144px` in `.workspace-control-bar`.
- [ ] Note the current push-to-talk button dimensions and shape.
- [ ] Note what the user sees on first load (no page open state).
- [ ] Identify all layout breakpoints that will need updating.

### 2.2 Make the push-to-talk button the dominant visual element — PENDING

The push-to-talk button must be redesigned from a small square crammed next to the URL bar into the visual centerpiece of the workspace.

- [ ] Change `.push-to-talk-button` to a **circle**: set `border-radius: 999px` and `aspect-ratio: 1 / 1`.
- [ ] Increase the size significantly. A minimum diameter of 120px on desktop, 100px on mobile is a reasonable starting point. Adjust based on visual balance.
- [ ] Remove the button from the `.workspace-control-bar` two-column grid. It should not be a peer of the URL card.
- [ ] Position the button prominently — options:
  - **Option A (recommended):** Center it above the URL bar as the first thing the user sees. URL bar below it, status panel below that.
  - **Option B:** Left-side vertical column with the button at top, URL/status content filling the right.
- [ ] Add a visible text label below or beside the button (e.g., "Hold to talk") so a first-time user immediately understands the affordance. This label should hide when the button is active/listening.
- [ ] Ensure the button's active (listening) and busy (processing) states are visually distinct — color change is already done, but consider adding a subtle pulse animation for the listening state to give sighted users an ambient cue.

### 2.3 Demote the URL input to secondary position — PENDING

The URL bar is a power-user shortcut. Voice is the primary navigation method. Adjust the visual weight accordingly.

- [ ] Move the URL input panel below the push-to-talk button in the DOM order and visual layout.
- [ ] Consider reducing the visual weight of the URL panel slightly — it does not need a full card treatment if the push-to-talk button is already in a card/section of its own. A lighter input row without the heavy box-shadow may work better.
- [ ] Ensure the URL input is still reachable by keyboard (Tab order) and screen reader.
- [ ] On mobile (`max-width: 640px`), the URL panel should stack below the PTT button naturally.

### 2.4 Add a first-load / empty state for the workspace — PENDING

When no page is open, the user sees a blank status card with no guidance. Fix this.

- [ ] In `src/settings-panels/workspace.ts` (the `renderStatusPanelNode` function), detect when `state.pageTitle` is null and no page has been opened.
- [ ] In that empty state, render a brief instructional message in place of the empty data grid. Example: *"Hold the Talk button and say a URL or command to get started."*
- [ ] This guidance should disappear once a page has been opened (i.e., `pageTitle` becomes non-null).
- [ ] Make this instructional copy part of the status panel's copy section, not a separate component.
- [ ] Ensure the guidance text is read by screen readers on first render.

### 2.5 Reduce heading sizes to utility-appropriate proportions — PENDING

Landing-page font sizes waste vertical space in a utility app.

- [ ] In `src/styles.css`, reduce the workspace `h1` size. The current `clamp(2.6rem, 6vw, 4.5rem)` should become something like `clamp(1.6rem, 3vw, 2.2rem)` — large enough to be a clear heading but not consuming a quarter of the screen.
- [ ] Reduce settings `h2` sizes similarly. `clamp(1.6rem, 3vw, 2.2rem)` down to approximately `clamp(1.2rem, 2vw, 1.5rem)`.
- [ ] Check all pages visually after the change to ensure the hierarchy still reads clearly.

---

## Phase 3 — Persistent header status strip

### 3.1 Design the status strip — PENDING

Define what information the strip shows and how it looks before implementing.

- [ ] The strip must be visible from **every view** — workspace and all settings subpages.
- [ ] Minimum content: a colored dot/icon + label for current voice state:
  - **Idle**: muted neutral dot, label "Ready"
  - **Listening**: green pulsing dot, label "Listening"
  - **Speaking**: amber dot, label "Speaking"
  - **Processing**: spinner or pulsing dot, label "Processing"
- [ ] Optional but useful: current page title truncated to one line.
- [ ] The strip should be compact — no taller than 36–40px — so it doesn't eat into workspace space.
- [ ] Decide placement: either in the existing `shell-toolbar` row (right-aligned, with the settings icon) or as a separate thin strip between the toolbar and the main content area.

### 3.2 Implement the status strip in the shell — PENDING

- [ ] Add the strip's state (listening, speaking, processing, idle) to the `AppShellPanelContent` or shell props so the shell can render it without coupling to panel state directly.
- [ ] In `src/app-shell.ts`, render the strip in the `shell-toolbar` header section, visible regardless of `initialAppView` or `initialSettingsView`.
- [ ] Add the necessary CSS classes to `src/styles.css` for the strip, the dot indicator, and its state variants.
- [ ] Add a `role="status"` and `aria-live="polite"` on the strip text so screen readers announce state changes without being disruptive.
- [ ] Wire the strip up to the actual `listening` and `speaking` state from the agent state in `src/main.ts` or wherever runtime state is reconciled.

### 3.3 Remove redundant listening/speaking cards from the status panel — PENDING

Once the persistent strip exists, the standalone Listening and Speaking status cards in the status panel are redundant.

- [ ] In `src/settings-panels/workspace.ts`, remove (or fold into a single combined row) the separate Listening and Speaking `status-card` elements from the `dl` grid.
- [ ] Verify the status panel still reads sensibly with just: page title, current region, last transcript, browser mode, history.
- [ ] Update any tests that assert on the removed elements.

---

## Phase 4 — Confirmation panel content cleanup

### 4.1 Strip internal debugging data — PENDING

- [ ] In `src/confirmation-panels/confirmation.ts`, remove the `<dl className="confirmation-meta">` block entirely. This block renders Confirmation ID, Request ID, and Next step ID — all internal artifacts that mean nothing to a user.
- [ ] Remove the `<div className="confirmation-columns">` block that renders Selected skills and Queued steps. These are planning internals, not user-facing decision information.
- [ ] If there is a product requirement to expose any of this for power users or debug builds, consider a `details`/`summary` disclosure element that is collapsed by default — not expanded by default.
- [ ] Update all render tests in `src/confirmation-panel.test.mjs` that assert on the removed elements.

### 4.2 Tighten the confirmation panel copy — PENDING

After stripping the debugging data, the confirmation panel should be: prompt + error state + approve/reject buttons.

- [ ] Verify the `state.promptText` in the `<p className="confirmation-prompt">` is consistently populated with genuinely human-readable action descriptions from the backend. If it can ever be empty, add a fallback.
- [ ] Ensure the `<h2>` text ("User approval is required before the next action runs.") is still correct and useful. Consider shortening to "Action requires your approval."
- [ ] Confirm the approve and reject button labels are clear enough without the surrounding metadata. If not, adjust the button labels or add a brief context line.
- [ ] Check the `isSubmitting` state still renders a visible "Submitting..." indicator after the metadata removal — it should still be present.

### 4.3 Validate confirmation panel accessibility after cleanup — PENDING

- [ ] Ensure `aria-live="polite"` on the confirmation panel section still fires correctly when the panel appears.
- [ ] Ensure `aria-busy` is correctly set during submission.
- [ ] Ensure the approve/reject `role="group"` container is still present with a clear `aria-label`.
- [ ] Run a manual screen reader check or review the render output to confirm the cleaned-up panel reads sensibly from top to bottom.

---

## Phase 5 — Settings cleanup

### 5.1 Trim the status panel description paragraph — PENDING

- [ ] In `src/settings-panels/workspace.ts`, remove the paragraph: *"This panel mirrors the live runtime so the nearby UI stays aligned with what the browser, narration, and listening tools are doing right now."*
- [ ] The eyebrow ("Runtime status") and heading ("Current browser state") are sufficient context. If a very brief one-sentence description is needed, keep it to one short sentence.
- [ ] Update any render tests that assert on that text.

### 5.2 Fix eyebrow repetition on settings subpages — PENDING

The settings overview and subpages repeat eyebrow + heading with slightly different wording, creating redundancy.

- [ ] In `src/app-shell.ts`, remove the eyebrow from settings subpage hero sections (the `hero-settings-subpage` sections). The breadcrumb-like context is already provided by the back button and the heading itself.
- [ ] Alternatively, keep the eyebrow but make it identical to what appears on the overview card (e.g., if the overview says "Command interpretation", the subpage eyebrow should also say exactly "Command interpretation", not "Command interpretation" again with a slightly different heading).
- [ ] Check all four subpages (Planner, TTS, ASR, Runtime) for this pattern and fix each one.

### 5.3 Reduce settings subpage heading sizes — PENDING

Covered partially in 2.5 but settings subpages deserve a specific check.

- [ ] Confirm `.settings-group h2` and `hero-settings-subpage h2` sizes are reduced to utility proportions after Phase 2 changes.
- [ ] Ensure settings overview section headings (Playback, Planner, TTS, ASR, Runtime) still have enough visual weight to be clear section dividers even at reduced size.

### 5.4 Improve settings subpage link affordance — PENDING

The current "Open planner setup" text link is a small underlined text next to a large serif heading — easy to miss.

- [ ] Replace the plain text link with a more tappable/clickable card element that includes a chevron icon (›) or arrow to reinforce that it navigates to a subpage.
- [ ] The card should fill the available width in the two-column grid, not just be a small text link.
- [ ] Ensure the improved link still has correct keyboard focus and `data-settings-view-button` attribute for event delegation.
- [ ] Check the mobile layout — on narrow screens the grid collapses to 1 column, so the link card should span the full width.

---

## Phase 6 — Polish and final audit

### 6.1 Audit all eyebrow colors after Phase 1 changes — PENDING

- [ ] After Phase 1.5 is complete, visually inspect every panel's eyebrow in a `pnpm dev` run.
- [ ] Confirm all eyebrows use `var(--eyebrow-color)` and none have reverted to per-panel accent colors.

### 6.2 Audit all panel border colors after Phase 1 changes — PENDING

- [ ] After Phase 1.3 is complete, visually inspect every panel.
- [ ] Confirm all content cards use `var(--card-border)` and the visual result is coherent.

### 6.3 Add a pulse animation for the listening state — PENDING

The push-to-talk button's listening state should have an ambient visual cue beyond just a color change.

- [ ] In `src/styles.css`, add a `@keyframes pulse-ring` animation (a subtle expanding ring or opacity pulse on a pseudo-element) that activates on `.push-to-talk-button-active`.
- [ ] Keep the animation subtle — it should not be distracting for sighted users and should respect `prefers-reduced-motion`.
- [ ] Add a `@media (prefers-reduced-motion: reduce)` override that disables the animation.

### 6.4 Verify responsive layout after workspace redesign — PENDING

- [ ] Test the new workspace layout at `320px`, `480px`, `640px`, `980px`, and wider viewports.
- [ ] Confirm the push-to-talk button remains accessible and not awkwardly sized at any breakpoint.
- [ ] Confirm the URL input and status panel stack correctly at narrow widths.
- [ ] Update the `@media (max-width: 640px)` block in `src/styles.css` as needed.

### 6.5 Run a full accessibility audit after all changes — PENDING

- [ ] Review all `aria-label`, `aria-live`, `aria-pressed`, `role`, and `aria-describedby` attributes in the modified components.
- [ ] Confirm focus order is logical in the redesigned workspace: PTT button → URL input → URL action buttons → status area.
- [ ] Confirm the persistent status strip announces state changes without being too chatty (polite, not assertive).
- [ ] Confirm the simplified confirmation panel reads correctly from top to bottom in a screen reader simulation.

### 6.6 Update or add render tests covering the changed components — PENDING

- [ ] Update `src/confirmation-panel.test.mjs` to reflect the stripped confirmation panel (no metadata block, no placeholder note).
- [ ] Update `src/main-behavior.test.mjs` if any workspace panel state assertions changed.
- [ ] Add render tests for the persistent status strip if it is a new component.
- [ ] Add a render test for the empty/first-load workspace state introduced in Phase 2.4.

### 6.7 Run full validation — PENDING

- [ ] `source ./fix-node-version.sh && . "$HOME/.cargo/env" && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml --all-features`
- [ ] `pnpm test:ui`
- [ ] `pnpm build`
- [ ] Fix any lint errors or test failures before committing.

### 6.8 Final visual review — PENDING

- [ ] Open the app in `pnpm tauri:dev:full` or `pnpm dev`.
- [ ] Walk through the first-time user flow: open app → see PTT button → hold to talk → give a URL → page opens → status updates → speak a navigation command → confirmation panel appears → approve.
- [ ] Walk through the settings flow: open settings → see all groups → navigate to a subpage → adjust a setting → go back.
- [ ] Confirm every issue from `docs/UIUX_REVIEW1.md` is visibly resolved.

---

## Suggested commit sequence

### Commit 1
Phases 1.1–1.5: font loading, placeholder removal, color token unification.

### Commit 2
Phase 2.1–2.5: workspace redesign (push-to-talk dominant, URL demoted, first-load state, heading sizes).

### Commit 3
Phase 3.1–3.3: persistent header status strip.

### Commit 4
Phase 4.1–4.3: confirmation panel content cleanup.

### Commit 5
Phase 5.1–5.4: settings cleanup.

### Commit 6
Phase 6.1–6.8: polish, pulse animation, responsive audit, full test pass, final visual review.
