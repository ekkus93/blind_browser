# Tailwind + TSX Migration TODO — blind_browser

Migration from hand-written CSS + MUI + `createElement` to Tailwind v4 + JSX/TSX with no
MUI dependency. This migration absorbs Phases 1, 2, and 4 of UIUX_IMPROVEMENTS2_TODO.md
(color tokens, focus rings, dark mode) and delivers them as Tailwind theme configuration
instead. Phases 3, 5, 6, 7 of UIUX_IMPROVEMENTS2_TODO.md continue after this migration.

---

## Scope

| Area | Before | After |
|---|---|---|
| CSS | `styles.css` (1555 lines, scattered hex values) | Tailwind v4 `@theme` tokens + utility classes |
| React render | `createElement` / `h(...)` in `.ts` files | JSX in `.tsx` files |
| Icons | `@mui/icons-material` (SettingsRounded, ArrowBackRounded) | Inline SVG components in `src/icons.tsx` |
| MUI shell | `ThemeProvider`, `CssBaseline`, `StyledEngineProvider`, `IconButton` | Removed entirely |
| Dark mode | None | Tailwind `dark:` variant, `@media (prefers-color-scheme: dark)` |
| Tests | Assert on CSS class names | Assert on `data-*` / `aria-*` attributes instead |

## Files that become TSX (use React JSX)

```
src/app-shell.ts              → src/app-shell.tsx
src/app-shell-nav.ts          → src/app-shell-nav.tsx
src/confirmation-panels/confirmation.ts  → confirmation.tsx
src/confirmation-panels/push-to-talk.ts  → push-to-talk.tsx
src/settings-panels/asr.ts        → asr.tsx
src/settings-panels/planner.ts    → planner.tsx
src/settings-panels/playback.ts   → playback.tsx
src/settings-panels/runtime.ts    → runtime.tsx
src/settings-panels/shared-controls.ts → shared-controls.tsx
src/settings-panels/tts.ts        → tts.tsx
src/settings-panels/workspace.ts  → workspace.tsx
src/app.ts                    → src/app.tsx  (if it renders JSX)
```

All other `.ts` files stay as `.ts` — they are pure logic/state, no JSX.

## Files deleted

```
src/app-shell-theme.ts   (MUI theme — replaced by Tailwind @theme)
```

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

## Phase 1 — Tooling setup

Install and configure Tailwind v4, enable JSX in TypeScript.

### 1.1 Install Tailwind v4 and remove MUI

- [ ] Install Tailwind and its Vite plugin:
  ```bash
  pnpm add -D tailwindcss @tailwindcss/vite
  ```
- [ ] Remove MUI and Emotion:
  ```bash
  pnpm remove @mui/material @mui/icons-material @emotion/react @emotion/styled
  ```

### 1.2 Configure Vite for Tailwind

- [ ] Edit `vite.config.ts` to add the Tailwind plugin:
  ```typescript
  import tailwindcss from "@tailwindcss/vite";
  import { defineConfig } from "vite";

  export default defineConfig({
    plugins: [tailwindcss()],
    clearScreen: false,
    server: {
      host: "127.0.0.1",
      port: 1420,
      strictPort: true,
    },
    preview: {
      host: "127.0.0.1",
      port: 1420,
      strictPort: true,
    },
  });
  ```

### 1.3 Enable JSX in TypeScript

- [ ] Edit `tsconfig.json` — add `jsx` and update `include` to cover `.tsx`:
  ```json
  {
    "compilerOptions": {
      "target": "ES2020",
      "useDefineForClassFields": true,
      "module": "ESNext",
      "lib": ["ES2020", "DOM", "DOM.Iterable"],
      "skipLibCheck": true,
      "moduleResolution": "Bundler",
      "allowImportingTsExtensions": true,
      "resolveJsonModule": true,
      "isolatedModules": true,
      "noEmit": true,
      "strict": true,
      "noUnusedLocals": true,
      "noUnusedParameters": true,
      "jsx": "react-jsx"
    },
    "include": ["src"]
  }
  ```
  (The `"include": ["src"]` already covers `.tsx` files when `jsx` is set.)

### 1.4 Replace styles.css with Tailwind entry + custom theme

- [ ] Replace the contents of `src/styles.css` with a Tailwind v4 CSS file that:
  - Imports Tailwind base
  - Defines the custom `@theme` block with all design tokens
  - Keeps any CSS that cannot be expressed as Tailwind utilities (animations,
    `font-face`, `radial-gradient` backgrounds, complex nested selectors)

  ```css
  @import "tailwindcss";

  @theme {
    /* Fonts */
    --font-sans: "IBM Plex Sans", "Segoe UI", sans-serif;
    --font-display: "Fraunces", Georgia, serif;

    /* Brand palette */
    --color-green-primary:  #29583f;
    --color-green-active:   #1f7f5c;
    --color-green-dark:     #1d4a30;
    --color-green-fresh:    #2d9b57;
    --color-amber-primary:  #7a5727;
    --color-amber-active:   #5a3e10;
    --color-error-primary:  #8b342a;
    --color-error-active:   #b04736;
    --color-error-dark:     #6c1010;

    /* Surface */
    --color-surface-base:   #f7f4ec;
    --color-surface-mid:    #ece4d5;
    --color-surface-card:   rgba(255, 252, 247, 0.9);

    /* Border radius */
    --radius-card:    22px;
    --radius-button:  999px;
    --radius-inner:   18px;
    --radius-input:   16px;
    --radius-badge:   999px;
  }

  /* Dark mode token overrides */
  @media (prefers-color-scheme: dark) {
    :root {
      --color-surface-base:   #1a1712;
      --color-surface-mid:    #141210;
      --color-surface-card:   rgba(36, 32, 26, 0.92);
    }
  }

  /* Keep animations that cannot be expressed as Tailwind utilities */
  @keyframes pulse-ring {
    0%   { box-shadow: 0 18px 30px rgba(176, 71, 54, 0.28), 0 0 0 0 rgba(176, 71, 54, 0.28); }
    70%  { box-shadow: 0 18px 30px rgba(176, 71, 54, 0.28), 0 0 0 18px rgba(176, 71, 54, 0); }
    100% { box-shadow: 0 18px 30px rgba(176, 71, 54, 0.28), 0 0 0 0 rgba(176, 71, 54, 0); }
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* Body baseline */
  body {
    background: linear-gradient(180deg, var(--color-surface-base) 0%, var(--color-surface-mid) 100%);
    color: #1d1a16;
    font-family: var(--font-sans);
    min-height: 100vh;
    margin: 0;
  }
  ```

### 1.5 Run validation gate

- [ ] `pnpm build` — should succeed (no components converted yet; MUI removed but not
      referenced by logic-only files). If it fails due to missing MUI imports in render
      files, that's expected — those files are converted in Phase 2.
- [ ] Fix any TypeScript errors caused by removing `@mui` types from the dependency tree.

---

## Phase 2 — Create icon components and remove MUI shell

### 2.1 Create `src/icons.tsx`

- [ ] Create `src/icons.tsx` with inline SVG replacements for the two MUI icons used:
  ```tsx
  export function SettingsIcon({ className }: { className?: string }) {
    return (
      <svg className={className} xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
        <path d="M19.14 12.94c.04-.3.06-.61.06-.94s-.02-.64-.07-.94l2.03-1.58a.49.49 0 0 0 .12-.61l-1.92-3.32a.49.49 0 0 0-.59-.22l-2.39.96a7.01 7.01 0 0 0-1.62-.94l-.36-2.54a.484.484 0 0 0-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96a.48.48 0 0 0-.59.22L2.74 8.87a.47.47 0 0 0 .12.61l2.03 1.58c-.05.3-.07.62-.07.94s.02.64.07.94l-2.03 1.58a.47.47 0 0 0-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32a.47.47 0 0 0-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"/>
      </svg>
    );
  }

  export function ArrowBackIcon({ className }: { className?: string }) {
    return (
      <svg className={className} xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
        <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"/>
      </svg>
    );
  }
  ```

### 2.2 Delete `src/app-shell-theme.ts`

- [ ] Delete `src/app-shell-theme.ts` — the MUI theme is fully replaced by the Tailwind
      `@theme` config.

### 2.3 Run validation gate

- [ ] `pnpm build` — expect errors in files that still import MUI; that's resolved in Phase 3.
- [ ] Confirm `src/icons.tsx` compiles cleanly on its own.

---

## Phase 3 — Convert render files to TSX

Convert each render file from `h(...)` / `createElement` to JSX. Work through files in
dependency order (leaf components first, shell last). After each file conversion, run
`pnpm build` to catch type errors early.

**Convention for each file:**
- Rename `.ts` → `.tsx`
- Remove `import { createElement } from "react"` and `const h = createElement`
- Convert all `h("tag", props, ...children)` → `<tag {...props}>children</tag>`
- Replace MUI imports with the new SVG icon components or plain HTML
- Apply Tailwind classes (see Phase 4 for the class mapping)
- Update any `import` statements in other files that referenced the old `.ts` path
  (since `allowImportingTsExtensions: true` is set, explicit `.ts` extension imports
  need updating to `.tsx`)

### 3.1 Convert `src/settings-panels/shared-controls.tsx`

- [ ] Rename and convert `shared-controls.ts` → `shared-controls.tsx`.
- [ ] This file likely exports reusable control components (labels, selects, buttons,
      cards). Convert all `h(...)` calls to JSX.
- [ ] Apply Tailwind classes (see Phase 4 class guide below).
- [ ] `pnpm build` — fix any errors.

### 3.2 Convert `src/confirmation-panels/confirmation.tsx`

- [ ] Rename and convert `confirmation.ts` → `confirmation.tsx`.
- [ ] Convert all `h(...)` calls to JSX.
- [ ] Apply Tailwind classes.
- [ ] `pnpm build` — fix any errors.

### 3.3 Convert `src/confirmation-panels/push-to-talk.tsx`

- [ ] Rename and convert `push-to-talk.ts` → `push-to-talk.tsx`.
- [ ] Convert all `h(...)` calls to JSX.
- [ ] Apply Tailwind classes.
- [ ] `pnpm build` — fix any errors.

### 3.4 Convert `src/settings-panels/playback.tsx`

- [ ] Rename and convert `playback.ts` → `playback.tsx`.
- [ ] Convert all `h(...)` calls to JSX.
- [ ] Apply Tailwind classes.
- [ ] `pnpm build` — fix any errors.

### 3.5 Convert `src/settings-panels/workspace.tsx`

- [ ] Rename and convert `workspace.ts` → `workspace.tsx`.
- [ ] Convert all `h(...)` calls to JSX.
- [ ] Apply Tailwind classes.
- [ ] `pnpm build` — fix any errors.

### 3.6 Convert `src/settings-panels/asr.tsx`

- [ ] Rename and convert `asr.ts` → `asr.tsx`.
- [ ] Convert all `h(...)` calls to JSX.
- [ ] Apply Tailwind classes.
- [ ] `pnpm build` — fix any errors.

### 3.7 Convert `src/settings-panels/tts.tsx`

- [ ] Rename and convert `tts.ts` → `tts.tsx`.
- [ ] Convert all `h(...)` calls to JSX.
- [ ] Apply Tailwind classes.
- [ ] `pnpm build` — fix any errors.

### 3.8 Convert `src/settings-panels/planner.tsx`

- [ ] Rename and convert `planner.ts` → `planner.tsx`.
- [ ] Convert all `h(...)` calls to JSX.
- [ ] Apply Tailwind classes.
- [ ] `pnpm build` — fix any errors.

### 3.9 Convert `src/settings-panels/runtime.tsx`

- [ ] Rename and convert `runtime.ts` → `runtime.tsx`.
- [ ] Convert all `h(...)` calls to JSX.
- [ ] Apply Tailwind classes.
- [ ] `pnpm build` — fix any errors.

### 3.10 Convert `src/app-shell-nav.tsx`

- [ ] Rename and convert `app-shell-nav.ts` → `app-shell-nav.tsx`.
- [ ] Replace MUI `IconButton` with plain `<button>` elements styled with Tailwind.
- [ ] Replace `SettingsRoundedIcon` with `<SettingsIcon>` from `src/icons.tsx`.
- [ ] Replace `ArrowBackRoundedIcon` with `<ArrowBackIcon>` from `src/icons.tsx`.
- [ ] Remove MUI `ComponentProps`, `IconButton` imports.
- [ ] Apply Tailwind classes.
- [ ] `pnpm build` — fix any errors.

### 3.11 Convert `src/app-shell.tsx`

- [ ] Rename and convert `app-shell.ts` → `app-shell.tsx`.
- [ ] Remove `StyledEngineProvider`, `ThemeProvider`, `CssBaseline` — the `renderShellTree`
      function no longer wraps in MUI providers. Just render `<AppShellMarkup>` directly.
- [ ] Remove `import { appShellTheme }` (file is deleted).
- [ ] Convert all `h(...)` calls to JSX.
- [ ] Apply Tailwind classes.
- [ ] `pnpm build` — fix any errors.

### 3.12 Convert `src/app.tsx` (if needed)

- [ ] Read `src/app.ts` — if it contains `createElement` calls, rename to `.tsx` and convert.
- [ ] If it's pure logic (store setup, event wiring), leave as `.ts`.
- [ ] `pnpm build` — fix any errors.

### 3.13 Update barrel exports and import paths

- [ ] `src/confirmation-panel.ts` — update all `from "./confirmation-panels/confirmation.ts"`
      to `".../confirmation.tsx"` etc.
- [ ] `src/settings-status-panels.ts` — update all panel imports to `.tsx` extensions.
- [ ] Any other files that import the renamed render modules — search for all `.ts` extension
      imports referencing the converted files and update them.
- [ ] Run `pnpm lint && pnpm build` — should be fully clean.

---

## Phase 4 — Tailwind class mapping guide

This phase documents the Tailwind class equivalents for every major CSS pattern in the
old `styles.css`. Use this reference when writing classes in Phase 3 conversions.

The guide is organized by component. When implementing, apply classes directly in the
JSX. Only add CSS to `styles.css` for things Tailwind cannot express (complex animations,
pseudo-element tricks, third-party overrides).

### Shell and layout

| Old CSS | Tailwind classes |
|---|---|
| `.shell` | `max-w-[980px] mx-auto px-6 pt-14 pb-18` |
| `.shell-toolbar` | `flex items-center gap-3 mb-6` |
| `.app-view[hidden]` | `hidden` (use conditional rendering or `hidden` class) |
| `.app-view-active` | `block` |
| `.sr-only` | `sr-only` (Tailwind has this built in) |

### Voice status strip

| Old CSS | Tailwind classes |
|---|---|
| `.voice-status-strip` | `inline-flex items-center gap-[7px] py-[5px] px-3 rounded-full bg-[rgba(255,252,247,0.82)] border border-[rgba(123,98,70,0.16)] shadow-sm text-[0.8rem] text-[#433d37] ml-auto select-none` |
| `.voice-status-dot` | `w-2 h-2 rounded-full bg-[rgba(123,98,70,0.35)] shrink-0 transition-colors duration-200` |
| dot listening | add `bg-[--color-green-primary]` |
| dot speaking | add `bg-[--color-amber-primary]` |
| dot processing | add `bg-[#1c5871]` |
| `.voice-status-label` | `font-semibold tracking-wide` |

### Toolbar action buttons

| Old CSS | Tailwind classes |
|---|---|
| `.shell-toolbar-action` | `inline-flex items-center justify-center shrink-0 w-11 h-11 p-0 rounded-full bg-[rgba(255,252,247,0.82)] border border-[rgba(41,88,63,0.16)] shadow-lg` |
| `:hover` | `hover:-translate-y-px hover:shadow-xl` |
| `:focus-visible` | `focus-visible:outline-2 focus-visible:outline-[--color-green-active] focus-visible:outline-offset-2 focus-visible:-translate-y-px` |

### Push-to-talk button

| Old CSS | Tailwind classes |
|---|---|
| `.push-to-talk-button` (idle) | `w-40 h-40 rounded-full bg-gradient-to-br from-[--color-green-primary] to-[--color-green-active] text-[#fffdf8] shadow-[0_14px_28px_rgba(31,127,92,0.24)] cursor-pointer transition-[transform,box-shadow] duration-[120ms] ease-out inline-flex items-center justify-center border-none` |
| `:hover:not(:disabled)` | `hover:-translate-y-px hover:shadow-[0_18px_30px_rgba(31,127,92,0.3)]` |
| `:focus-visible` | `focus-visible:outline-[3px] focus-visible:outline-[--color-green-active] focus-visible:outline-offset-1` |
| `:disabled` | `disabled:cursor-not-allowed disabled:bg-gradient-to-br disabled:from-[#8a8070] disabled:to-[#a0947e] disabled:shadow-none disabled:opacity-70` |
| active (recording) | `bg-gradient-to-br from-[--color-error-primary] to-[--color-error-active] [animation:pulse-ring_1.6s_ease-out_infinite]` |
| `motion-reduce:` | `motion-reduce:[animation:none]` |

### Card anatomy (shared by all panels)

| Old CSS | Tailwind classes |
|---|---|
| `.url-input-panel` / `.status-panel` / `.audio-controls-panel` / `.settings-panel` / `.confirmation-panel` | `mt-[18px] p-[22px_24px] rounded-[22px] bg-[--color-surface-card] border border-[rgba(123,98,70,0.16)] shadow-[0_18px_36px_rgba(49,63,74,0.08)]` |
| inner card (status card, audio control, settings control card) | `p-[16px_18px] rounded-[18px] bg-[rgba(255,255,255,0.68)] border border-[rgba(123,98,70,0.12)]` |

### Typography

| Old CSS | Tailwind classes |
|---|---|
| `.eyebrow` / `*-eyebrow` | `text-[0.76rem] uppercase tracking-[0.18em] text-[--eyebrow-color] mb-2` |
| panel `h2` | `font-[--font-display] text-[clamp(1.1rem,2vw,1.4rem)] leading-[1.05] mb-[10px]` |
| `h1` (hero) | `font-[--font-display] text-[clamp(1.8rem,3.5vw,2.6rem)] leading-[0.94] max-w-[10ch]` |
| `.lede` | `mt-[18px] max-w-[60ch] text-[1.05rem] leading-relaxed text-[#3a342e]` |

### Settings subpage cards (overview links)

| Old CSS | Tailwind classes |
|---|---|
| `.settings-subpage-card` | `flex items-center justify-between w-full px-4 py-3 appearance-none border border-[rgba(123,98,70,0.16)] rounded-[10px] bg-[rgba(255,252,247,0.6)] text-[--color-green-primary] font-bold cursor-pointer text-left transition-[background,box-shadow,transform] duration-150` |
| `:hover` | `hover:bg-[rgba(255,252,247,0.9)] hover:shadow-[0_6px_16px_rgba(41,88,63,0.12)] hover:-translate-y-px` |
| `:focus-visible` | `focus-visible:outline-2 focus-visible:outline-[--color-green-active] focus-visible:outline-offset-2` |
| status badge ok | `text-[0.75rem] font-semibold px-2 py-[2px] rounded-full bg-[rgba(41,88,63,0.12)] text-[--color-green-dark]` |
| status badge warning | `bg-[rgba(122,87,39,0.14)] text-[--color-amber-active]` |
| status badge error | `bg-[rgba(139,52,42,0.14)] text-[--color-error-primary]` |
| status badge unconfigured | `bg-[rgba(67,61,55,0.1)] text-[#433d37]` |

### Settings control buttons

| Old CSS | Tailwind classes |
|---|---|
| Primary button | `w-fit px-4 py-[10px] rounded-full border-none bg-gradient-to-br from-[--color-green-primary] to-[#347f55] text-[#f6f2eb] font-bold cursor-pointer transition-[transform,box-shadow,opacity] duration-[120ms] hover:-translate-y-px hover:shadow-[0_10px_18px_rgba(41,88,63,0.2)] disabled:cursor-progress disabled:opacity-60 disabled:shadow-none` |
| Secondary button | `bg-gradient-to-br from-[#d9e5dd] to-[#eef5ef] text-[#1f2527] border border-[rgba(41,88,63,0.16)]` |
| Danger button | `bg-gradient-to-br from-[--color-error-primary] to-[#b84436] text-[#fffdf8] border border-[rgba(139,52,42,0.3)]` |

### Confirmation panel buttons

| Old CSS | Tailwind classes |
|---|---|
| Approve button | `appearance-none border-0 rounded-full px-[18px] py-3 font-semibold cursor-pointer transition-[transform,box-shadow,background-color] duration-[140ms] bg-[--color-green-primary] text-[#f6fbf8] hover:bg-[--color-green-dark] hover:-translate-y-px disabled:cursor-progress disabled:opacity-60` |
| Reject button | `bg-[#efe6d8] text-[#5b2a1f] border border-[rgba(91,42,31,0.12)] hover:bg-[#e7d8c6] hover:-translate-y-px` |

### URL input panel

| Old CSS | Tailwind classes |
|---|---|
| Input field | `w-full px-4 py-[14px] rounded-[16px] border border-[rgba(123,98,70,0.18)] bg-[rgba(255,255,255,0.92)] text-[#1f2527] shadow-[inset_0_1px_0_rgba(255,255,255,0.5)] focus-visible:outline-2 focus-visible:outline-[--color-green-active] focus-visible:outline-offset-2 disabled:cursor-progress disabled:opacity-70` |
| Open/Read button | `bg-gradient-to-br from-[--color-green-primary] to-[--color-green-active] shadow-[0_12px_24px_rgba(31,127,92,0.18)]` |
| Stop button | `bg-gradient-to-br from-[--color-error-primary] to-[--color-error-active]` |
| Nav buttons | `bg-gradient-to-br from-[--btn-nav-start] to-[--btn-nav-end]` |

### Error and warning banners

| Old CSS | Tailwind classes |
|---|---|
| Setup banner / model missing | `mt-3 p-3 bg-[rgba(41,88,63,0.08)] border border-[rgba(41,88,63,0.22)] rounded-[12px] flex flex-col gap-2` |
| Reset confirm row | `flex-col items-start gap-[10px] bg-[rgba(139,52,42,0.06)] border border-[rgba(139,52,42,0.18)] rounded-[12px] p-[12px_14px]` |
| Inline panel error text | `mt-2 text-[--color-error-primary] font-semibold leading-[1.55]` |

### Dark mode

Tailwind dark mode using `@media (prefers-color-scheme: dark)`. Prefix any dark-specific
class with `dark:`. The `@theme` dark overrides in `styles.css` handle surface tokens;
component classes just need `dark:` prefixes for text and border where they differ:

| Pattern | Tailwind |
|---|---|
| Dark text on surfaces | `dark:text-[#f0ead8]` |
| Dark muted text | `dark:text-[#c8bfae]` |
| Dark card border | `dark:border-[rgba(180,155,110,0.14)]` |
| Dark input background | `dark:bg-[rgba(48,43,36,0.8)]` |

---

## Phase 5 — Update tests

**Problem:** Many of the 97 JS tests (`pnpm test:ui`) assert on CSS class names. After the
conversion to Tailwind, those class names either disappear or become long utility strings.
Tests must be migrated to assert on `data-*` attributes, `aria-*` attributes, and text content.

### 5.1 Add `data-testid` attributes to structural elements in render files

For every element that is tested by a structural assertion (not text content), add a
`data-testid` or a descriptive `data-*` attribute in the render function. Examples:

- Confirmation panel root: `data-panel="confirmation"`
- PTT button: `data-ptt-button="true"` (already has `data-ptt-button` — verify)
- Setup banner: `data-ptt-setup-banner="true"`
- Error elements: `data-error="true"` or existing `role="alert"`
- Status cards: `data-status-card="<name>"` (may already have `data-*` attributes)
- Settings subpage links: `data-settings-view-button` (already present)

### 5.2 Update test assertions in each test file

Work through each test file and update class-based assertions to use data attributes:

- [ ] `src/confirmation-panel-core.test.mjs`
- [ ] `src/confirmation-panel-url-audio.test.mjs`
- [ ] `src/confirmation-panel-settings-planner.test.mjs`
- [ ] `src/confirmation-panel-settings-voice.test.mjs`
- [ ] `src/confirmation-panel-status.test.mjs`
- [ ] `src/app-shell.test.mjs`
- [ ] `src/dom-seams.test.mjs`
- [ ] `src/main-behavior.test.mjs`
- [ ] `src/main-errors.test.mjs`
- [ ] `src/tauri-api.test.mjs`

**Pattern to follow:** Replace `assert.match(html, /class="confirmation-panel"/)` with
`assert.match(html, /data-panel="confirmation"/)` or `assert.match(html, /role="region"/)`.
Prefer aria attributes over data attributes where the aria attribute is already present.

### 5.3 Run full validation gate

- [ ] `pnpm test:ui` — all 97 tests pass.
- [ ] `pnpm lint` — no errors.
- [ ] `pnpm build` — clean build.

---

## Phase 6 — Final validation and cleanup

### 6.1 Run the full validation suite

- [ ] `source ./fix-node-version.sh`
- [ ] `pnpm lint`
- [ ] `pnpm test:ui`
- [ ] `pnpm build`
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml --all-features`

### 6.2 Visual audit

- [ ] Open the app. Confirm all panels render with correct layout and typography.
- [ ] Switch OS to dark mode. Confirm dark surfaces and legible text throughout.
- [ ] Tab through all interactive elements. Confirm consistent green focus ring on every
      focused element.
- [ ] Confirm the PTT button shows the pulse-ring animation when active.
- [ ] Confirm button spinners (`.btn-spinner`) appear during loading states.
- [ ] Confirm the confirmation panel has a warm amber tint (not blue-teal).
- [ ] Confirm Fraunces serif renders in headings and IBM Plex Sans in body text.

### 6.3 Remove old CSS dead code

- [ ] Search `styles.css` for any remaining class definitions that are no longer used
      (all structure should now be in Tailwind classes). Remove them.
- [ ] Confirm `styles.css` contains only: `@import`, `@theme`, `@keyframes`, `body` baseline,
      and any genuinely non-Tailwind-expressible rules.

### 6.4 Update `memory.md`

- [ ] Run `date -u +"%Y-%m-%dT%H:%M:%SZ"` and add an entry to `memory.md`.

### 6.5 Update `UIUX_IMPROVEMENTS2_TODO.md`

- [ ] Mark Phases 1 (color tokens), 2 (focus rings), and 4 (dark mode) as absorbed by
      this migration.
- [ ] Confirm Phases 3 (font loading), 5 (planner UX), 6 (settings UX), and 7 (feedback)
      still need implementation — they continue after this migration.

---

## Suggested commit sequence

```
Commit 1:  Phase 1 — tooling setup (Tailwind installed, JSX enabled, MUI removed)
Commit 2:  Phase 2 — icon components, app-shell-theme.ts deleted
Commit 3:  Phase 3 — all render files converted to TSX with Tailwind classes
Commit 4:  Phase 4 — (reference only, no commit needed)
Commit 5:  Phase 5 — tests updated for new data attributes
Commit 6:  Phase 6 — final validation and cleanup
```
