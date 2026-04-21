# UI/UX Review — blind_browser

**Reviewed by:** Claude Sonnet 4.6  
**Date:** 2026-04-21

---

## What's genuinely good

**Color palette and typography** — the warm cream/green/brown palette (`#f7f4ec`, `#29583f`, `#7a5727`) is tasteful and calming. IBM Plex Sans + Fraunces serif is a good pairing. These are keepers.

**Card system** — the rounded-rect panel cards with subtle shadows are clean and work well for dense settings content.

**ARIA semantics** — `aria-live`, `aria-pressed`, `aria-disabled`, `role="group"` etc. are correctly applied throughout. This is solid work.

**Settings organization** — Planner / TTS / ASR / Runtime is a logical grouping and the subpage navigation pattern is sound.

**Confirmation error states** — the tiered retryable/hard-stop visual treatment is actually quite thoughtful.

---

## What's bad — honest assessment

### 1. The information hierarchy is inverted for the product's purpose

This is supposed to be a **voice-first app for vision-impaired users**. The visual design treats it as a **web browser with a microphone button tacked on**.

Right now the hierarchy is:
1. Big URL bar (dominant)
2. Green square button (secondary, crammed against the URL bar's right edge)
3. Status panel (below the fold, verbose)

The correct hierarchy for this product is:
1. **Voice state** — huge, center-screen, always visible
2. **Feedback** — what did you say? what's happening? what region?
3. **URL/navigation** — secondary, since users get there by voice anyway

### 2. The push-to-talk button is visually wrong

It has `aspect-ratio: 1/1`, `min-height: 144px`, and `border-radius: 18px` — making it a **rounded rectangle** jammed next to the URL bar. It looks like a peer of the URL card, not the primary action. A microphone button universally reads as a **circle**. It should be center-dominant, not a sidebar attachment.

### 3. The status panel description text is wasted space

> "This panel mirrors the live runtime so the nearby UI stays aligned with what the browser, narration, and listening tools are doing right now."

This paragraph is never useful after the first read and it pushes the actual live data (listening state, current region, last transcript) further down. Cut it.

### 4. There's a developer placeholder left in the confirmation panel

```
"The frontend can now present approve or reject controls against this state and send the user response back through the Tauri confirmation command."
```

This is a code comment that made it into production. It needs to be removed.

### 5. The confirmation panel shows internal debugging data to users

Showing **Confirmation ID**, **Request ID**, **Next step ID**, **Selected skills**, **Queued steps** — these are debugging artifacts, not user-facing information. A vision-impaired user being read this out loud would hear: *"Confirmation ID: a7f3b2c1-0e84... Request ID: 9d5c..."* That's noise. The user needs exactly: **What is about to happen?** and **Do you approve?**

### 6. Five different accent colors on adjacent buttons

The URL action buttons each have a different color family:
- Open → amber gradient
- Read → green gradient
- Stop → red gradient
- Previous → indigo/violet gradient
- Next → teal gradient

Five unrelated colors for five adjacent buttons in a 44px row. There's no semantic reason for this — they aren't categorically different actions. It looks like someone picked a different favorite color for each button. This should collapse to 1–2 colors (neutral/ghost for navigation controls, green for read, red for stop).

### 7. Panel borders have meaningless color variation

- URL panel: warm brown border
- Audio controls panel: blue border
- Settings panel: green border
- Confirmation panel: blue border again (same blue as audio, different semantic context)
- Status panel: brown border

No consistent semantic rule. Pick one border color for all cards — the warm neutral.

### 8. No persistent voice/status indicator in the header

The toolbar has only the settings gear. There is **nowhere** that always shows whether the app is listening, speaking, or processing. If you're a sighted helper assisting a blind user, you have to scroll down to see whether the mic is hot. This should be a persistent ambient indicator in the header — always visible, no matter what view you're on.

### 9. The workspace has no onboarding state

When the app first loads, the user sees: a URL bar, a green button, and an empty status card. There's no "Say something to get started" prompt, no guidance about push-to-talk, no indication the app is ready. A first-time user has no idea the green button is for voice.

### 10. The serif display font probably isn't loading

`@font-face` rules for **Fraunces** and **IBM Plex Sans** are referenced in the CSS but not imported anywhere (`index.html` has no `<link>` for Google Fonts, and there's no `@import` in the CSS). These are falling back to Georgia and Segoe UI silently. The designed typographic hierarchy isn't actually being delivered.

### 11. "Hero" heading sizes belong in a marketing site, not a utility tool

`font-size: clamp(2.6rem, 6vw, 4.5rem)` for the workspace `h1` and `clamp(1.6rem, 3vw, 2.2rem)` for settings section headings are landing page proportions. They waste vertical space that vision-impaired users need for controls. A utility app should use compact, functional type sizes.

### 12. Settings "eyebrow" labels repeat differently on subpages

Overview shows: *"Command interpretation"* eyebrow → *"Planner"* heading. Then the subpage shows: *"Command interpretation"* eyebrow (repeated) → *"Planner setup"* heading (slightly different). The pattern doesn't add clarity — it adds redundancy.

---

## The honest verdict

**The settings area is fine** — structurally sound, needs minor cleanup (colors, fonts, trim the eyebrow repetition).

**The workspace needs to be redesigned from scratch** around the correct mental model: voice assistant first, URL browser second. The current layout was designed as a browser with voice sprinkled in. It should be the opposite.

**The confirmation flow needs major content surgery** — strip the internal IDs, strip the placeholder note, surface only what the user must know to decide approve/reject.

---

## Proposed fixes, prioritized

### Priority 1 — Workspace redesign
- Redesign the workspace layout: large circular push-to-talk button centered or prominently placed, URL input collapsed/secondary, status feedback prominent above the fold.
- The push-to-talk button should be a circle (`border-radius: 999px`), substantially larger than its current size, and visually dominant — not a peer of the URL card.
- Add a "Say something to get started" first-load state so new users understand the interaction model immediately.

### Priority 2 — Persistent header status strip
- Add a persistent ambient indicator in the header (visible from every view — workspace and settings) that always shows listening / speaking / idle state with a colored dot or icon.
- This makes the app's live state legible at a glance for sighted helpers and low-vision users without requiring a scroll.

### Priority 3 — Confirmation panel content cleanup
- Remove the developer placeholder note entirely.
- Remove Confirmation ID, Request ID, Next step ID — these are debugging artifacts.
- Remove Selected skills and Queued steps from the default view (or collapse them behind a "Details" disclosure if needed for power users).
- Leave only: prompt text, approve button, reject button, and error state if present.

### Priority 4 — Color consolidation
- **Panel borders**: choose one border color for all content cards (the existing warm neutral `rgba(123, 98, 70, 0.16)` is a good pick). Remove the per-panel blue/green/brown variations that carry no semantic meaning.
- **URL action buttons**: collapse to two color semantics — red for Stop, neutral dark for all navigation/utility buttons (Open, Read, Previous, Next). Stop trying to rainbow the button row.

### Priority 5 — Font loading
- Add a `<link>` to Google Fonts (or self-host) for **Fraunces** and **IBM Plex Sans** in `index.html`. Until this is done, the type system is not being delivered.

### Priority 6 — Status panel trim
- Remove the explanatory description paragraph from the status panel.
- Move listening/speaking indicators to the persistent header strip (Priority 2) and shrink or remove the redundant status cards for those fields.
- Keep: page title, current region, last transcript. These are the three things a user cares about.

### Priority 7 — Settings minor cleanup
- Reduce heading sizes to utility-appropriate proportions (e.g., `1.4rem` max for `h2` in settings, not `clamp(1.6rem, 3vw, 2.2rem)`).
- Remove or simplify the eyebrow repetition on settings subpages.
- Unify eyebrow text color — all eyebrows should use the same muted tone, not per-panel accent colors.
