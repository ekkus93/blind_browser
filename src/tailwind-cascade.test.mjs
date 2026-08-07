// Regression coverage for the CR3 P0.9 Tailwind cascade bugs (from af89a22):
// class order within a `className` string does NOT determine which rule
// wins -- Tailwind's own utility-emission order does -- so a base class plus
// a conditional "delta" class fighting over the same CSS property is a
// latent bug that lint, tsc, build, and ordinary rendering tests all miss
// (nothing in those checks looks at computed style). This is why all four
// regressions shipped with a fully green build and 238/238 passing tests.
//
// These tests compile the project's actual exported Tailwind class-string
// constants through Tailwind's own compiler and assert on the emitted CSS
// declarations directly, so a future reintroduction of either bug shape --
// a `border-*` colour utility used on a variable that holds a full
// `<width> <style> <color>` shorthand, or a "state" constant built as
// base-plus-conditional-delta instead of a complete, self-contained string
// per state -- fails here instead of shipping invisibly.
import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createRequire } from "node:module";
import path from "node:path";

import { compile } from "tailwindcss";

import { SETTINGS_PANEL_SECTION_CLASS, CONTROL_CARD, CONTROL_CARD_READONLY_CLASS, REMOTE_PRIVACY_STATUS_CLASS, REMOTE_PRIVACY_SETTINGS_BUTTON_CLASS, REMOTE_CONSENT_DESTINATION_COUNTS_ROW_CLASS, REMOTE_PRIVACY_ROW_CLASS, SETTINGS_SECTION_HEADING_CLASS, SETTINGS_CARD_HEADING_CLASS } from "./settings-panels/shared-controls.tsx";
import { AUDIO_CONTROL_CLASS } from "./settings-panels/playback.tsx";
import { TOGGLE_BUTTON_PRESSED_CLASS, TOGGLE_BUTTON_UNPRESSED_CLASS } from "./settings-panels/workspace.tsx";
import { GUIDANCE_ACTION_BUTTON_CLASS } from "./settings-panels/runtime.tsx";

const require = createRequire(import.meta.url);

// Resolves `@import "tailwindcss";` against the installed package without
// going through Vite/PostCSS -- `tailwindcss`'s own `compile()` API takes a
// list of candidate class names directly and returns the generated CSS text,
// which is the narrowest and fastest way to get real computed-declaration
// output for a handful of known strings.
async function loadStylesheet(id) {
  const specifier = id === "tailwindcss" ? "tailwindcss/index.css" : id;
  const resolvedPath = require.resolve(specifier);
  return {
    path: resolvedPath,
    base: path.dirname(resolvedPath),
    content: await readFile(resolvedPath, "utf8"),
  };
}

// Tailwind's `build()` is designed for incremental rebuilds: candidates
// accumulate across calls on the same `compile()` instance rather than each
// call returning only that call's own CSS. A fresh `compile()` per assertion
// keeps each test's output scoped to exactly the candidates it passed in --
// otherwise an earlier test's utilities (e.g. `bg-[var(--color-surface-inner)]`
// from the audio-control-card test) would leak into a later test's output
// and produce false "not self-contained" failures. Tailwind candidates are
// whitespace-separated tokens; arbitrary values use underscores in place of
// spaces (already the convention throughout this codebase), so splitting a
// full className string on whitespace is safe.
async function compileClassString(classString) {
  const { build } = await compile('@import "tailwindcss";', {
    base: process.cwd(),
    loadStylesheet,
  });
  return build(classString.trim().split(/\s+/));
}

test("[border:var(--x)] compiles to the border shorthand, not border-color", async () => {
  const css = await compileClassString("[border:var(--card-border)] [border:var(--inner-card-border)]");

  assert.match(css, /border:\s*var\(--card-border\)/);
  assert.match(css, /border:\s*var\(--inner-card-border\)/);
  assert.doesNotMatch(css, /border-color:\s*var\(--card-border\)/);
  assert.doesNotMatch(css, /border-color:\s*var\(--inner-card-border\)/);
});

test("the border-[var(--x)] colour-utility form is the broken shape this fix removed", async () => {
  // Documents *why* P0.9.1 matters: the form the bug used compiles cleanly
  // (Tailwind can't know --card-border holds a shorthand, not a colour), so
  // nothing at build time flags it -- the variable's shorthand value only
  // becomes invalid once the browser tries to compute `border-color` from
  // it, at which point it silently resets to `currentColor`. This test pins
  // that compiled shape as a historical fact, not as something we want; the
  // fix is that the app's source no longer uses this form (see the source-
  // scan test below).
  const css = await compileClassString("border-[var(--card-border)]");

  assert.match(css, /border-color:\s*var\(--card-border\)/);
});

test("no source file uses the border-[var(--x)] colour-utility form of the shorthand variables", async () => {
  const { execFile } = await import("node:child_process");
  const { promisify } = await import("node:util");
  const execFileAsync = promisify(execFile);

  const { stdout } = await execFileAsync("git", [
    "grep",
    // Include untracked-but-not-ignored files too, so a newly-added file
    // with the bug is caught before it's ever staged, not only after.
    "--untracked",
    "-n",
    "-E",
    String.raw`border-\[var\(--card-border\)\]|border-\[var\(--inner-card-border\)\]`,
    "--",
    "src/",
    // Excludes this file itself: it intentionally documents the broken form
    // as a string literal (see the test above) rather than using it.
    ":!src/tailwind-cascade.test.mjs",
  ]).catch((error) => {
    // `git grep` exits 1 with empty stdout when there are no matches --
    // that's the success case here, not a failure.
    if (error.code === 1 && !error.stdout) {
      return { stdout: "" };
    }
    throw error;
  });

  assert.equal(
    stdout,
    "",
    `found the broken border-[var(--x)] colour-utility form still in use:\n${stdout}`,
  );
});

test("the exported panel/card border constants compile to the shorthand form", async () => {
  assert.match(await compileClassString(SETTINGS_PANEL_SECTION_CLASS), /border:\s*var\(--card-border\)/);
  assert.match(await compileClassString(AUDIO_CONTROL_CLASS), /border:\s*var\(--inner-card-border\)/);
});

test("the toggle button's pressed and unpressed states are complete, non-conflicting alternatives", async () => {
  const pressed = await compileClassString(TOGGLE_BUTTON_PRESSED_CLASS);
  const unpressed = await compileClassString(TOGGLE_BUTTON_UNPRESSED_CLASS);

  assert.match(pressed, /background-color:\s*rgba\(41,88,63,0\.14\)/);
  assert.match(pressed, /color:\s*var\(--color-green-active\)/);
  assert.doesNotMatch(pressed, /background-color:\s*var\(--color-surface-inner\)/);
  assert.doesNotMatch(pressed, /color:\s*var\(--color-text-secondary\)/);

  assert.match(unpressed, /background-color:\s*var\(--color-surface-inner\)/);
  assert.match(unpressed, /color:\s*var\(--color-text-secondary\)/);
  assert.doesNotMatch(unpressed, /background-color:\s*rgba\(41,88,63,0\.14\)/);
  assert.doesNotMatch(unpressed, /color:\s*var\(--color-green-active\)/);
});

test("the settings control card and its read-only variant each set exactly one background", async () => {
  const card = await compileClassString(CONTROL_CARD);
  const readOnlyCard = await compileClassString(CONTROL_CARD_READONLY_CLASS);

  // "color-mix" alone isn't a safe absence check -- Tailwind's preflight
  // base layer uses color-mix() for the ::placeholder rule regardless of
  // which candidates are compiled, so this checks for the read-only
  // variant's specific color-mix() call rather than the bare substring.
  assert.match(card, /background-color:\s*var\(--color-surface-inner\)/);
  assert.doesNotMatch(card, /color-mix\(in srgb,var\(--color-surface-card\)/);

  assert.match(readOnlyCard, /color-mix\(in srgb,var\(--color-surface-card\)\s*72%,transparent\)/);
  assert.doesNotMatch(readOnlyCard, /background-color:\s*var\(--color-surface-inner\)/);
});

test("the remote privacy/consent breakpoint is 768px (max-md), not 640px (max-sm)", async () => {
  for (const classString of [
    REMOTE_PRIVACY_STATUS_CLASS,
    REMOTE_PRIVACY_SETTINGS_BUTTON_CLASS,
    REMOTE_CONSENT_DESTINATION_COUNTS_ROW_CLASS,
    REMOTE_PRIVACY_ROW_CLASS,
  ]) {
    const css = await compileClassString(classString);
    assert.match(css, /@media \(width < 48rem\)/, `expected a 48rem (max-md) breakpoint in:\n${css}`);
    assert.doesNotMatch(css, /@media \(width < 40rem\)/, `found the narrower 40rem (max-sm) breakpoint in:\n${css}`);
  }
});

// CR3 P3.3.1: Tailwind's preflight resets `h1..h6{font-size:inherit;
// font-weight:inherit}`, so a heading with no explicit size utility silently
// renders as plain body text -- these two shared scales are now the one
// place every heading in the app draws its size from. Pin that each one
// actually emits a `font-size`/`font-family` declaration (not just an
// unclamped `inherit`), so a future edit that accidentally strips the
// `text-[clamp(...)]`/`[font-family:...]` piece fails here.
test("the shared heading scales emit real font-size and font-family declarations", async () => {
  const section = await compileClassString(SETTINGS_SECTION_HEADING_CLASS);
  const card = await compileClassString(SETTINGS_CARD_HEADING_CLASS);

  assert.match(section, /font-size:\s*clamp\(1\.2rem,\s*2\.2vw,\s*1\.6rem\)/);
  assert.match(section, /font-family:\s*var\(--font-display\)/);
  assert.match(card, /font-size:\s*clamp\(1\.1rem,\s*2vw,\s*1\.4rem\)/);
  assert.match(card, /font-family:\s*var\(--font-display\)/);
});

// CR3 P3.3.2: the guidance CTA's label had no `text-*` utility at all, so it
// fell back to inherited near-black body text against a dark-green
// gradient background -- well below 4.5:1. Pin that a `color` declaration
// is present now, guarding against the missing-utility shape recurring
// silently (compiling cleanly either way, since Tailwind has no way to know
// a class string is "missing" a color).
test("the guidance CTA sets an explicit light text color", async () => {
  const css = await compileClassString(GUIDANCE_ACTION_BUTTON_CLASS);

  assert.match(css, /color:\s*#fffdf8/);
});
