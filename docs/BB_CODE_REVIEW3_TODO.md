# Blind Browser Code Review 3 TODO

## How to use this file

This is a correctness / safety / architecture pass driven by Code Review 3
(`docs/BB_CODE_REVIEW3_SPEC.md`), against the `master` snapshot at `af89a22`.
Work top-to-bottom. Keep diffs scoped to the task at hand and keep the validation
gate green between tasks. Do not start the non-goals listed in the spec.

Status key: `PENDING` · `IN PROGRESS` · `DONE` · `BLOCKED`

Priority key:

- `P0`: correctness / safety defect reachable in normal use, or a user-visible
  regression already on `master`. Fix first.
- `P1`: architecture change that closes a policy gap or removes a UI-freeze /
  interruptibility failure.
- `P2`: enabling debt, hardening, and test coverage that prevents these defect
  classes from recurring.
- `P3`: documentation drift and dead configuration surface.

Confidence key (see the spec's "Finding confidence" section):

- `[VERIFIED]` — mechanism confirmed by reading the code during review. Fix it.
- `[VERIFY FIRST]` — reported with concrete evidence but not independently
  re-derived. **Reproduce before writing a fix.** If it does not reproduce, record
  that here and close the item instead of changing code.

Validation gate:

```bash
source ./fix-node-version.sh
pnpm lint
pnpm test:ui
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
bash scripts/check-silent-fallbacks.sh
python3 scripts/check-security-fallbacks.py
python3 scripts/check-security-fallback-inventory.py
python3 scripts/check-sensitive-diagnostics.py
python3 scripts/check-remote-planner-privacy-state.py
bash scripts/run-rust-tests-linux.sh
```

Do not mark this TODO complete unless the validation gate actually passes in the
developer environment.

If a fix removes an expression listed in `scripts/security-fallback-allowlist.txt`,
remove its allowlist entry in the same change — the inventory check is exact.

---

## P0.1 — Restore the element label in click confirmations

**Status:** PENDING · `[VERIFIED]`
**Spec:** constraint 1
**Files:**

- `src-tauri/src/app_core/click_authorization.rs`
- `src-tauri/src/commands/confirmation_manifest.rs` (tests)

### Problem

`clear_runtime_annotations` deletes `RUNTIME_TARGET_LABEL_ARG` from every step.
`annotate_target_step` restores it for `TypeIntoElement` and `annotate_submit_step`
for `SubmitActiveForm`, but `annotate_click_step` writes back only the five
`CLICK_AUTH_*` keys. `safe_action_summary`'s `ClickElement` arm therefore falls
through to `element_id`, which `dom_extraction.rs` generates positionally as
`` `element-${index + 1}` ``.

A blind user asking to click a destructive control hears:

> "Approve this action on https://bank.example: Click element 'element-7'."

The spoken prompt is the only safety barrier for this action class, and it is
uninformative. The runtime already has the label — `mint_click_authorization` uses
it to decide destructiveness.

### Required behavior

- A click confirmation names the element with its human-readable label.
- The label is runtime-derived, never planner-supplied.
- It continues to pass through `safe_element_label` so it stays bounded and
  sanitized.

### P0.1.1 — Carry the label on the authorization record

Add a label field to `ClickAuthorizationRecord`, populated in
`mint_click_authorization` from the already-resolved element via
`safe_element_label`. Prefer this over re-resolving the element in
`annotate_click_step`, so the label is bound to the same element the token was
minted for.

### P0.1.2 — Write it back in `annotate_click_step`

Insert `RUNTIME_TARGET_LABEL_ARG` from the record alongside the five
`CLICK_AUTH_*` keys.

### P0.1.3 — Cover it in the manifest tests

`confirmation_manifest.rs`'s test module covers `SubmitActiveForm` and
`TypeIntoElement` summaries but has **no `ClickElement` case** — that is why this
survived. Add one asserting the summary contains the label and does **not**
contain an `element-N` id.

### Acceptance checks

```bash
rg -n "RUNTIME_TARGET_LABEL_ARG" src-tauri/src/app_core/click_authorization.rs
rg -n "ClickElement" src-tauri/src/commands/confirmation_manifest.rs
```

Expected: the label is inserted in `annotate_click_step`; a `ClickElement` summary
test exists and asserts the label appears.

---

## P0.2 — Bound the narration cursor against the current region list

**Status:** PENDING · `[VERIFIED]`
**Spec:** constraint 2
**Files:**

- `src-tauri/src/narration.rs`
- `src-tauri/src/app_core/reading_tools.rs`

### Problem

`previous_region_index` returns `Some(index - 1)` with no comparison against
`region_count`, while `next_region_index` correctly checks
`index + 1 < region_count`. `reading_tools.rs` then indexes the region vector
directly.

`replace_current_page_model` bumps the page generation but does **not** reset
`narration_cursor` (only `record_navigation` and `clear_navigation_follow_up_state`
do). So: read to region 8 of 10 → the page re-extracts with 3 regions (SPA
re-render, cookie banner, lazy collapse) → `read_previous_region` →
**index out of bounds: the len is 3 but the index is 6**. A hostile page can force
this deterministically.

### Required behavior

- Cursor movement is total over the current region list.
- No narration path indexes a region vector without a bounds check.
- A shrinking region list clamps the cursor rather than panicking.

### P0.2.1 — Make `previous_region_index` bounds-aware

Clamp against `region_count`, mirroring `next_region_index`'s existing guard.

### P0.2.2 — Remove unchecked indexing at the call sites

Replace direct `regions[region_index]` indexing in `reading_tools.rs` with
bounds-checked access that returns the existing "no readable regions" /
"already at the start" tool result rather than panicking. Check both the previous
and next region paths.

### P0.2.3 — Decide and document cursor behavior on model replacement

Either reset the cursor in `replace_current_page_model`, or clamp on read. Pick
one and add a comment stating which, so the invariant is explicit. Note that
resetting changes user-visible behavior (a re-extraction would restart narration
from the top), so clamping is likely preferable — confirm against the reading UX
before choosing.

### P0.2.4 — Regression test

Test: cursor at index 7, region list replaced with 3 regions, then
`read_previous_region` and `read_next_region` — both return a bounded result and
do not panic.

### Acceptance checks

```bash
rg -n "regions\[" src-tauri/src/app_core/reading_tools.rs
rg -n "fn previous_region_index" -A8 src-tauri/src/narration.rs
```

Expected: no unchecked indexing; `previous_region_index` compares against
`region_count`.

---

## P0.3 — Validate settings before writing them to disk

**Status:** PENDING · `[VERIFIED]`
**Spec:** constraint 3
**Files:**

- `src-tauri/src/config/persistence.rs`
- `src-tauri/src/command_handlers/safety_handlers.rs`

### Problem

`persist_safety_settings_at_path` and `persist_ocr_settings_at_path` go straight
from `document.insert(...)` to `write_config_atomic(...)` with **no validation**.
The sibling `persist_audio_settings_at_path` validates first and returns early.
Validation happens only in the trailing `load_from_path`, i.e. after the invalid
value is already on disk. `set_ocr_thresholds` and `set_confirmation_threshold`
pass their raw values through with no clamping, and `validate_ocr_settings` /
`validate_safety_settings` are called only from the load path.

Result: `set_ocr_thresholds(…, 0, 0)` writes `0` to disk, returns `Err` to the
caller, and the **next launch fails to start** — with no in-app recovery path.

### Required behavior

- Every persister validates before writing and returns `ConfigError::Validation`
  without touching the file.
- No code path leaves `config.toml` in a state its own loader rejects.

### P0.3.1 — Validate in both persisters

Add the `validate_*` + `return Err(ConfigError::Validation(...))` preamble to
`persist_safety_settings_at_path` and `persist_ocr_settings_at_path`, matching the
audio/models/privacy pattern exactly.

### P0.3.2 — Audit the remaining persisters

Confirm every `persist_*_at_path` either validates or normalizes before writing.
Record any that legitimately need neither.

### P0.3.3 — Consider extracting the shared shape

There are ten near-identical persisters with four different validation postures.
A single `fn mutate_config_document(path, validate, |doc| ...)` helper would make
this defect class structurally impossible to reintroduce. Optional here; if
deferred, note it under P2.8's cleanup umbrella.

### P0.3.4 — Regression test

Test: persisting an invalid OCR threshold returns `Err` **and** leaves the
on-disk file loadable (assert `load_from_path` still succeeds afterwards).

### Acceptance checks

```bash
rg -n "validate_safety_settings|validate_ocr_settings" src-tauri/src/config/persistence.rs
```

Expected: both called before `write_config_atomic` in their respective persisters.

---

## P0.4 — Close the two confirmation-bypass paths in element resolution

**Status:** PENDING · `[VERIFIED]`
**Spec:** constraint 4
**Files:**

- `src-tauri/src/app_core/element_scoring.rs`
- `src-tauri/src/app_core/form_fill/field_fill.rs`
- `src-tauri/src/app_core/form_fill/field_focus.rs`
- `src-tauri/src/commands/validators/element.rs`

### Problem

Two independent instances of "the ambiguity check is skipped on a degenerate
candidate list":

1. `rank_find_element_candidates` calls `candidates.truncate(candidate_limit)`
   **before** `determine_find_element_resolution` reads `candidates.get(1)` for the
   runner-up margin. The validator accepts `max_candidates: 1` (it rejects only `0`
   and `> 3`), so a planner emitting `1` makes `get(1)` always `None` and
   `ambiguous_with_runner_up` always `false`. Two identical "Continue" buttons both
   scoring 10000 → resolved with no confirmation. The same query with
   `max_candidates: 3` correctly requires confirmation.
2. `field_fill.rs` and `field_focus.rs` special-case `candidates.len() == 1` and
   hardcode `requires_confirmation = false`, never consulting
   `confirmation_confidence_threshold` (default 0.90). A 0.205-confidence match of
   "card number" against a lone "Number of guests" field is typed into with no
   confirmation and no clarification.

### Required behavior

- The ambiguity margin is computed over the full ranked set before truncation.
- The confidence threshold applies regardless of surviving candidate count.
- A degenerate candidate list makes the system more cautious, never less.

### P0.4.1 — Compute the runner-up margin before truncation

Either compute ambiguity inside `rank_find_element_candidates` before truncating,
or return enough information (e.g. the runner-up confidence) for
`determine_find_element_resolution` to decide correctly. Do not simply raise the
minimum `max_candidates`, which would leave the ordering hazard in place.

### P0.4.2 — Remove the single-candidate special case in fill/focus

Delete the `candidates.len() == 1` branch in both files so
`determine_find_element_resolution` — and therefore the confidence threshold —
always governs. Verify the intended direct-fill UX still works for a genuinely
confident single match (a high-scoring exact label match should still pass the
threshold and proceed without confirmation).

### P0.4.3 — Note the tie-break ordering

`element_scoring.rs` tie-breaks on `left.element_id.cmp(&right.element_id)`, which
is lexicographic over `"element-N"` — so `"element-10" < "element-2"`. It is
deterministic but not document order. Not a bug; add a comment so the next reader
does not "fix" it into nondeterminism.

### P0.4.4 — Regression tests

- Two identically-scoring elements with `max_candidates: 1` → requires
  confirmation.
- A single weak-scoring candidate below threshold in `field_fill` → requires
  confirmation / clarification rather than proceeding.

### Acceptance checks

```bash
rg -n "truncate|get\(1\)" src-tauri/src/app_core/element_scoring.rs
rg -n "candidates.len\(\) == 1" src-tauri/src/app_core/form_fill/
```

Expected: no confirmation-skipping single-candidate special case remains; the
margin is computed pre-truncation.

---

## P0.5 — Validate the settings navigation target

**Status:** PENDING · `[VERIFIED]`
**Spec:** constraint 9
**Files:**

- `src/panel-state-setters.ts`
- `src/main-errors.ts`
- `src/dom-seams.test.mjs`

### Problem

`focusSettingsTarget` casts a DOM element id directly to `SettingsView`:

```ts
setSettingsView(targetId as SettingsView);   // unchecked cast
```

The ids supplied by `main-errors.ts` are values like
`"settings-model-management-title"`; `SettingsView` is only
`overview | planner | tts | asr | runtime`. Because every subview renders
`hidden={initialSettingsView !== "<view>"}`, an unmatched value hides **all five**
— including `overview`, which holds the page heading and the guidance panel
itself.

The reachable path: local ASR model missing → guidance panel offers "Open model
management" → activating it strands a blind user on a page whose only control is
the back arrow, which returns to overview and re-offers the same broken CTA.

### Required behavior

- Navigation targets are validated against the `SettingsView` union, not cast.
- An unmatched target fails safe to a real view and surfaces an error rather than
  rendering an empty page.

### P0.5.1 — Map element ids to views explicitly

Introduce an explicit `targetId → SettingsView` map (or change the guidance
actions to carry a `SettingsView` plus an optional element id for focus). The
element ids are still useful for focusing the specific control after the view
switches — keep that capability rather than discarding the id.

### P0.5.2 — Fail safe on an unknown target

An unrecognized target must not reach `setSettingsView`. Route to `overview` and
raise a visible app alert.

### P0.5.3 — Strengthen the test

`dom-seams.test.mjs` currently asserts only that the callback fires with the id.
Assert the **resulting** `settingsView` is a valid member and that the rendered
shell has at most one non-hidden settings subview.

### Acceptance checks

```bash
rg -n "as SettingsView" src/
pnpm test:ui
```

Expected: no unchecked cast remains; a test asserts the resulting view.

---

## P0.6 — Make confirmation and consent surfaces reachable from any view

**Status:** PENDING · `[VERIFIED]`
**Spec:** constraint 9
**Files:**

- `src/app-shell.tsx`
- `src/confirmation-panel.tsx`
- `src/app.tsx`

### Problem

The `confirmation-panel` root — which hosts both the action-approval panel and the
remote-data consent dialog (`WorkspaceDecisionPanels`) — is rendered inside the
workspace `<section>`, which carries `hidden` and `aria-hidden="true"` whenever
the app is in Settings view.

Push-to-talk is bound globally to `window` with no view gate, and
`ensureContinuousListeningLoop` runs regardless of `appView`. So a command issued
while in Settings can raise a confirmation or consent dialog into a
`display:none` / `aria-hidden` subtree: focus calls become no-ops, the screen
reader announces nothing, the focus trap never engages, and the challenge silently
expires. Nothing switches the view back — `setAppView("workspace")` appears only
in the nav button.

### Required behavior

- A confirmation or consent surface is visible and focusable whenever it can be
  raised.
- A safety gate that cannot be seen or heard is equivalent to no gate.

### P0.6.1 — Choose the approach

Two viable options; pick one and record the rationale:

- **(a)** Hoist the decision surfaces out of the view-switched containers so they
  render as an overlay at shell level regardless of `appView`. Preferred — it
  makes the guarantee structural.
- **(b)** Automatically switch `appView` to `workspace` when a confirmation or
  consent state is raised. Simpler, but it moves the user's view without them
  asking, which needs care in a voice-first UI.

### P0.6.2 — Preserve focus-trap behavior

Both existing focus-trap modules (Escape/Tab wrap-around, capture-and-restore,
`isConnected` check before restoring) were reviewed and found correct. Whichever
approach is chosen, verify they still engage and that focus returns to the prior
element on dismiss.

### P0.6.3 — Regression test

Render the shell in Settings view with an `awaiting-confirmation` state and assert
the confirmation panel is **not** inside any `hidden` / `aria-hidden` subtree.

### Acceptance checks

```bash
pnpm test:ui
rg -n "data-app-view-section=\"workspace\"" -A6 src/app-shell.tsx
```

Expected: the confirmation/consent root is not nested inside the view-hidden
workspace section (or the view switches automatically, per the chosen approach).

---

## P0.7 — Reject empty synthesized audio

**Status:** PENDING · `[VERIFIED]`
**Spec:** constraint 6
**Files:**

- `src-tauri/src/tts/mod.rs`
- `src-tauri/src/tts/local.rs`, `src-tauri/src/tts/remote.rs` (caching)

### Problem

`synthesize_narration` checks that the *input text* is non-empty but never checks
the *output samples*. Both providers can return empty: the pinned local TTS
returns `Ok(vec![])` for punctuation-only input such as `"..."` or `"?!"` (ordinary
on the web), and a remote WAV with a zero-length `data` chunk decodes to an empty
vector.

`play_samples` accepts it, `speaking = true` is recorded, the narration cursor
advances, and the tool reports **success**. The empty result is then **cached**, so
re-reading the region is silent for the life of the process. A blind user gets
silence with no error.

### Required behavior

- Empty synthesized audio is a typed error surfaced to the user.
- Empty results are never cached.

### P0.7.1 — Add the guard

Add `TtsRuntimeError::EmptySynthesizedAudio` and return it from
`synthesize_narration` when `samples.is_empty()` — **before** the cache insert in
both provider paths.

### P0.7.2 — Confirm the error reaches the user audibly

Trace `tts_runtime_error_to_tool_error` and confirm the resulting message is
surfaced on a path the user can perceive. A silent failure replaced by a silent
error is no improvement.

### P0.7.3 — Regression test

Test: a synthesis returning zero samples produces an error, and the cache does not
retain the entry (a second call re-attempts rather than returning cached silence).

### Acceptance checks

```bash
rg -n "samples.is_empty|EmptySynthesizedAudio" src-tauri/src/tts/
```

Expected: guard present and applied before caching in both providers.

---

## P0.8 — Reset and cap the hands-free capture buffer

**Status:** PENDING · `[VERIFIED]`
**Spec:** constraint 7
**Files:**

- `src-tauri/src/asr/mod.rs`
- `src-tauri/src/asr/capture.rs`

### Problem

**This is a different defect from the one fixed in Code Review 2.** CR2 fixed
`CaptureSession::snapshot()` cloning without clearing. This is `begin_capture`
short-circuiting when a session exists and never draining, combined with a cpal
callback that appends unconditionally with no cap.

Push-to-talk depends on that accumulation (it deliberately relies on the hold-time
buffer). The hands-free loop does not: with `auto_stop = false` the session stays
alive across the entire command execution — planner round trips, browser work, and
TTS playback — with the microphone open throughout.

Consequences: the app transcribes **its own spoken narration** into the next
command (a self-triggering loop); `MAX_TRANSCRIBE_DURATION_MS` stops bounding the
audio actually sent; remote ASR fails permanently past roughly 4.4 minutes of
session once the 8 MiB upload cap trips; memory grows at roughly 1.4 GB/hour.

### Required behavior

- A hands-free window transcribes only audio captured for that window.
- Push-to-talk still returns the full held utterance.
- The shared buffer has a hard cap so a stuck session cannot grow without bound.

### P0.8.1 — Reset at the start of a hands-free window

Drain-and-discard the buffer when beginning a window whose stop mode is not the
PTT hold case. Prefer making the phase explicit at the handler level (the caller
knows whether it is PTT or hands-free) over inferring it inside `begin_capture`.

### P0.8.2 — Cap the buffer in the callback

Add a hard cap in `capture_input_data` — drop oldest frames past
`MAX_TRANSCRIBE_DURATION_MS` worth of samples. This must hold even if a session is
never stopped.

### P0.8.3 — Verify PTT is unaffected

Confirm by reading the call path and by test that press-to-release still returns
the full held utterance.

### P0.8.4 — Regression tests

- Two consecutive hands-free windows do not return overlapping samples.
- The buffer does not exceed the cap after sustained appends.

### Acceptance checks

```bash
rg -n "mem::take|drain|MAX_CAPTURE|cap" src-tauri/src/asr/capture.rs src-tauri/src/asr/mod.rs
```

Expected: hands-free windows reset; a cap exists and is unit-tested.

---

## P0.9 — Fix the Tailwind migration regressions from `af89a22`

**Status:** PENDING · `[VERIFIED]`
**Spec:** constraint 10
**Files:**

- `src/settings-panels/shared-controls.tsx`
- `src/settings-panels/playback.tsx`
- `src/settings-panels/workspace.tsx`
- `src/app-shell-nav.tsx`
- `src/confirmation-panels/push-to-talk.tsx`
- `src/confirmation-panels/confirmation.tsx`

### Problem

Four visual regressions, all from one root cause: class order within the `class`
attribute does not determine precedence — the rule Tailwind emits later wins. All
four passed lint, build, and 238/238 tests, because nothing asserts computed style.

1. **Borders resolve to `currentColor`, or vanish.**
   `border-[var(--card-border)]` compiles to `border-color: var(--card-border)`,
   but that variable holds the shorthand `1px solid rgba(...)`.
   `border-color: 1px solid rgba(...)` is invalid at computed-value time and resets
   to `currentColor`. Every top-level card draws a near-black/near-white hairline.
   Worse, `AUDIO_CONTROL_CLASS` in `playback.tsx` has the colour utility with **no
   width utility**, and preflight sets `*{border:0 solid}` — those cards render no
   border at all. The correct form `[border:var(--inner-card-border)]` is already
   used correctly at `shared-controls.tsx:36`.
2. **Browser-visibility toggle loses its pressed state.** `workspace.tsx` sets
   `bg-`/`text-`/`border-` in both the base string and the conditional; the base is
   emitted later and wins. Verified against the built stylesheet:
   `text-[var(--color-text-secondary)]` @27683 beats
   `text-[var(--color-green-active)]` @27273. `aria-pressed` stays correct, so this
   affects sighted and low-vision users only.
3. **Read-only cards lost their muted background.** `shared-controls.tsx` appends
   `bg-[color-mix(...)]` to `CONTROL_CARD`, which already sets a background.
   Verified: color-mix @15075, base @16906 — base wins. ~20 cards look editable
   while remaining `pointer-events-none`.
4. **Privacy/consent breakpoint moved 768 px → 640 px.** The deleted CSS used
   `@media (max-width: 48rem)`; the port used `max-sm:` (40rem). `max-md:` was the
   faithful mapping.

### Required behavior

- Any class constant representing a mutually exclusive visual state is a complete,
  self-contained string — the rule documented in `shared-controls.tsx` and then
  violated in three places.
- CSS variables holding shorthand values use the arbitrary-property form
  `[border:var(--x)]`, never a `border-*` colour utility.

### P0.9.1 — Fix the border shorthand usages

Replace every `border-[var(--card-border)]` / `border-[var(--inner-card-border)]`
with `[border:var(--card-border)]` / `[border:var(--inner-card-border)]`, and
remove the now-redundant `border` width utility where one was paired with it.
Check all six files listed above.

### P0.9.2 — Make the toggle pressed state self-contained

Restructure the two visibility-toggle class strings so the pressed and unpressed
variants are complete alternatives rather than base-plus-delta.

### P0.9.3 — Make the read-only card variant self-contained

Same treatment for `renderReadOnlyCard` — do not append a background to a constant
that already sets one.

### P0.9.4 — Restore the 768 px breakpoint

Change the four `max-sm:` variants in the privacy/consent constants to `max-md:`.

### P0.9.5 — Add a computed-style regression test

This is the important part — it is why all four slipped past a green build. Add a
test that builds (or reads the built) stylesheet and asserts, for each known
conflict pair, that the intended winner is emitted last; plus that
`[border:var(--card-border)]` compiles to the `border` shorthand rather than
`border-color`. Keep it narrow and fast enough for `pnpm test:ui`.

### Acceptance checks

```bash
pnpm build
rg -n "border-\[var\(--card-border\)\]|border-\[var\(--inner-card-border\)\]" src/
pnpm test:ui
```

Expected: no `border-*` colour-utility usage of the shorthand variables remains;
the cascade regression test exists and passes.

---

## P1.1 — Gate remote TTS and ASR through the shared consent layer

**Status:** PENDING · `[VERIFIED]` (gap confirmed; design is new work)
**Spec:** constraint 5
**Files:**

- `src-tauri/src/app_core/remote_data_consent/**`
- `src-tauri/src/tts/**`, `src-tauri/src/asr/**`
- `src-tauri/src/app_core/narration.rs`
- `src/` consent UI (challenge copy for the new disclosure kinds)

### Problem

The remote-planner path enforces `network_mode`, `origin_rules`, high-risk-origin
blocking, sanitization, and a tamper-evident consent challenge before any
page-derived data leaves the device. Remote TTS and remote ASR enforce **none** of
it — confirmed by exhaustive grep: zero references to `remote_planner_privacy`,
`network_mode`, `origin_rules`, `high_risk`, `consent`, `sanitize`, or `redact`
anywhere in `src-tauri/src/tts/` or `src-tauri/src/asr/`.

`synthesize_narration` passes verbatim page region text to `"input": text`;
remote ASR multipart-uploads raw microphone audio.

Concretely: on a page the policy classifies high-risk, the planner is blocked from
receiving even *sanitized* summaries, yet "read this page" ships that page's raw,
unredacted text to a third party.

This is the largest architectural gap found in the review. **The decision to close
it (rather than document it as out-of-scope) was made explicitly** — see the spec's
constraint 5 for the required boundary.

### Required behavior

- Page-derived text sent to a remote TTS provider, and microphone audio sent to a
  remote ASR provider, pass the same origin / network-mode / high-risk policy
  evaluation that governs the remote planner.
- One shared decision point, not three parallel reimplementations.
- The type-state property is preserved: a caller cannot construct an authorized
  remote request without going through the policy.
- Local TTS/ASR remain ungated — no data leaves the device.

### P1.1.1 — Generalize the policy evaluation

Lift `evaluate_remote_planner_policy` and the `PreparedRemotePlannerRequest`
type-state into a provider-agnostic form parameterized by disclosure kind
(planner payload / narration text / microphone audio). Keep the constructor
private so the choke-point property survives. Do **not** duplicate the policy into
`tts/` and `asr/`.

### P1.1.2 — Route remote TTS through it

`synthesize_narration`'s remote branch must require an authorized request. Decide
how narration text is scoped to an origin — the region text comes from the current
page, so the current page origin is the natural binding, but confirm the case
where narration text originates from a tool result rather than page content
(`execute_report_result`) and handle it explicitly rather than defaulting.

### P1.1.3 — Route remote ASR through it

Microphone audio is not page-derived, so the origin binding is different in kind.
Decide and document what it is bound to (likely the current page origin plus the
endpoint scope, since the transcript will be used to act on that page). Note this
interacts with P0.8: an unbounded buffer means the uploaded audio may span far more
than the intended window, so P0.8 should land first.

### P1.1.4 — Consent challenge copy for the new disclosure kinds

"Send this page's text to be spoken aloud" and "send microphone audio to be
transcribed" are materially different user decisions from "send sanitized page
context for planning". Extend the disclosure-class vocabulary and the dialog copy
accordingly. Keep the manifest tamper-evidence property (every field bound into
the digest).

### P1.1.5 — Settings surfacing

The settings UI must make clear, when TTS/ASR mode is `Remote`, that speech data
leaves the device — and reflect the governing policy.

### P1.1.6 — Tests

Extend the fail-closed evidence tests to cover all three disclosure kinds:
high-risk page blocks remote narration; an origin block rule blocks remote
narration; `local_only` blocks remote ASR; loopback endpoints stay ungated.
These belong with the isolated-Wry security tests — see P2.2 for the list guard.

### Acceptance checks

```bash
rg -n "network_mode|origin_rules|high_risk|consent" src-tauri/src/tts src-tauri/src/asr
```

Expected: non-empty; remote paths evaluate the shared policy before dispatch, and
local paths remain untouched.

---

## P1.2 — Release the runtime lock across network and capture windows

**Status:** PENDING · `[VERIFIED]`
**Spec:** constraint 8
**Files:**

- `src-tauri/src/app_core/replanning_orchestrator.rs`
- `src-tauri/src/app_core/narration.rs`
- `src-tauri/src/app_core/listening_tools.rs`

### Problem

Two paths hold the `AppCore` mutex across multi-second blocking work:

1. Remote TTS synthesis runs a blocking request with the profile timeout (default
   30 000 ms) under the held lock, reached through the orchestrator's tool dispatch.
2. A planner-emitted `transcribe_command` step blocks for the capture window (up to
   `MAX_TRANSCRIBE_DURATION_MS`) plus the ASR round trip, also under the lock.

While either is in flight, `stop_listening` and `get_agent_state` block — **a blind
user cannot stop the microphone or interrupt a hung synthesis.**

`run_phased_transcribe` was built specifically to release the lock across exactly
these windows; both paths bypass it.

### Required behavior

- No network round trip and no capture sleep occurs while the lock is held.
- The established phased pattern is applied: snapshot config under the lock,
  perform blocking work unlocked, re-acquire to commit.
- Existing stop-during-window semantics are preserved (`Ok(None)` when the session
  was dropped mid-window).

### P1.2.1 — Lock-scope remote TTS synthesis

Snapshot the config and audio state needed for synthesis, drop the guard,
synthesize, then re-acquire to hand samples to playback and update speaking state.
Note that `play_samples` also performs device I/O (`open_default_sink`) under the
same lock today — decide whether that also moves out.

### P1.2.2 — Route planner-driven transcription through the phased path

Make `execute_transcribe_command` use the same three-phase structure as
`run_phased_transcribe` rather than sleeping under the lock. If that requires
restructuring how a planner step yields control, record the constraint here — this
may be the point where the item becomes `BLOCKED` pending a larger executor change,
as happened in CR2's P1.1.2/P1.1.3.

### P1.2.3 — Verify interruptibility

Confirm by test or manual trace that `stop_listening` succeeds while a synthesis
or planner-driven capture is in flight.

### Acceptance checks

Read `replanning_orchestrator.rs` and `listening_tools.rs`: no `synthesize_*` or
capture sleep occurs between `lock_app_core` and the guard drop.

---

## P1.3 — Resolve the region bbox coordinate space

**Status:** PENDING · `[VERIFY FIRST]`
**Spec:** — (reported; not a numbered constraint pending verification)
**Files:**

- `src-tauri/src/browser/dom_extraction.rs`
- `src-tauri/src/browser/page_inspection.rs`
- `src-tauri/src/ocr.rs`

### Problem (to verify)

Region bboxes are produced from `getBoundingClientRect()`, which is
**viewport-relative**. They are consumed as document coordinates: passed as a CDP
`Page.captureScreenshot` `clip` (page-relative) and to `set_rectangle` against a
**full-page** screenshot. Grep confirms `scroll_y` is never added to a bbox
anywhere.

Predicted failure: after scrolling 1200 px, "read that section" captures/OCRs
content 1200 px above the intended region, and the blind user is narrated
confidently correct-sounding text from the **wrong part of the page** with no
error.

Partially confirmed during review (the producer is `getBoundingClientRect`; no
`scroll_y` correction exists anywhere). **Not** confirmed end-to-end against a real
scrolled page.

### P1.3.1 — Reproduce

Load a page taller than the viewport, scroll, capture a region screenshot, and
compare the captured content against the intended region. Record the result here.

### P1.3.2 — Fix if confirmed

Either add `scrollX`/`scrollY` to the extracted bbox at extraction time, or record
the scroll offset alongside the bbox and correct at consumption. Prefer correcting
at extraction so every consumer inherits the fix.

### P1.3.3 — Note the masked fallback path

`extractor.rs` sets `bbox: None` for dom_smoothie regions, so
`region_first_ocr_target_ids` returns empty on that path — the bug is live for the
direct `capture_screenshot` / `run_ocr` region tools and for the fallback whenever
dom_smoothie fails. Cover both in the fix.

---

## P1.4 — Close the `execute_planner_output` validation gap

**Status:** PENDING · `[VERIFY FIRST]`
**Files:**

- `src-tauri/src/app_core/command_dispatch.rs`
- `src-tauri/src/app_core/planning_snapshot.rs`

### Problem (to verify)

`execute_planner_output` is a registered Tauri command taking a caller-supplied
`PlannerOutput`, and reportedly applies only snapshot binding, preparation, and
initial policy — never `validate_planner_output_with_safety`.

Dangerous tools are said to be blocked by the snapshot requirement, but tools
absent from `planner_output_requires_snapshot` — `ExtractPageModel`,
`ReportResult`, `TranscribeCommand` — would get through unvalidated.
`ExtractPageModel` calls `mark_page_model_changed()`, which **clears the pending
confirmation**, so an unvalidated direct call could cancel a confirmation the user
is mid-way through answering.

### P1.4.1 — Reproduce

Construct a minimal `ExtractPageModel`-only `PlannerOutput`, invoke the command
directly, and confirm whether it executes and whether it clears a pending
confirmation. Record the result.

### P1.4.2 — Fix if confirmed

Either call `validate_planner_output_with_safety` at the top of
`AppCore::execute_planner_output`, or extend `planner_output_requires_snapshot` to
cover every tool that mutates `AppState`. Prefer the former — it restores the
layering rather than enumerating exceptions.

### P1.4.3 — Revisit the `is_side_effecting` classification

`TranscribeCommand` (opens the microphone, may upload audio) and
`ExtractPageModel` (bumps generation, clears authorizations and pending
confirmations) are currently classified as not side-effecting in two places. Even
if not presently exploitable, the classification is wrong on its face. Correct it
or document why it is safe.

---

## P2.1 — Test the click-authorization subsystem

**Status:** PENDING
**Spec:** constraint 11
**Files:** `src-tauri/src/app_core/click_authorization.rs` (+ a tests module)

### Problem

693 lines carrying most of the click-safety invariants, with
`prepare_planner_output_for_execution`, `preflight_pending_click_authorizations`,
`insert_deterministic_click_confirmation_gate`, and `ClickGroundingAuthorized`
having **zero references outside their own definitions**. The only tests are two
unit tests on fingerprinting and keyword matching. This is the enabling debt behind
P0.1.

### Required coverage

- A planner-supplied `_runtime_click_authorization` for a token that was never
  minted → `unknown_click_authorization`.
- A planner-supplied `_runtime_click_ambiguous: false` over a genuinely ambiguous
  record → stripped and re-derived.
- A `Ready` all-click plan actually receives the inserted confirmation gate.
- Token single-use across two click steps → `duplicate_click_authorization`.
- Expiry and page-generation staleness are enforced.
- The live-DOM fingerprint mismatch path rejects.

### Acceptance checks

```bash
rg -n "prepare_planner_output_for_execution|insert_deterministic_click_confirmation_gate" src-tauri/src
```

Expected: referenced from tests, not only their definitions.

---

## P2.2 — Guard the CI lists that can silently under-run

**Status:** PENDING
**Spec:** constraint 12
**Files:** `scripts/run-rust-tests-linux.sh`

### Problem

`run-rust-tests-linux.sh` hardcodes the names of the 7 isolated security tests —
which are **all** the fail-closed consent/privacy tests — with no check that the
list is complete. An eighth `#[ignore]`d test silently never runs, with a green
build. `check-remote-planner-privacy-state.py`'s `REQUIRED_PATHS` has the same
shape and has already needed manual repair twice during file splits; the `test:ui`
glob had it too (fixed in `d9dc6c7`).

### P2.2.1 — Assert list completeness

Derive the ignored-test set from `cargo test -- --ignored --list` and fail if it
differs from the hardcoded list. Keep the explicit list (it documents intent and
pins ordering) — just make divergence an error.

### P2.2.2 — Note P1.1's new tests

P1.1.6 adds fail-closed tests for the new disclosure kinds; if any are `#[ignore]`d
for Wry isolation, they must appear in the list, and P2.2.1 will now enforce that.

### Acceptance checks

```bash
bash scripts/run-rust-tests-linux.sh
```

Expected: fails if the hardcoded list and the actual ignored set diverge.

---

## P2.3 — Restrict config file permissions

**Status:** PENDING
**Files:** `src-tauri/src/config/persistence.rs`

### Problem

`write_config_atomic` uses bare `fs::File::create` with no mode, so the file lands
at the umask default (measured `0644`). It is not secret-bearing — the keyring
handles that — but since the consent work it holds `origin_rules`: a durable,
timestamped record of **which sites the user visited and what they consented to
send off-device**. On a shared machine any local user can read it. `image_cache.rs`
already does this correctly (`0700` dir / `0600` file).

### P2.3.1 — Set the mode at creation

Use `OpenOptions::new().mode(0o600)` on the temp file (Unix) before the rename —
mode must be set at creation, since setting it after the rename is itself a TOCTOU
window. Consider `0700` on the config directory to match the image cache.

### P2.3.2 — Consider the adjacent durability and concurrency notes

Two lower-severity items found in the same function, worth deciding on while it is
open: the parent directory is never fsynced after the rename (so the rename is not
durable across power loss), and the temp file has a fixed name (`config.toml.tmp`)
with no concurrency control, so two processes can splice or revert each other's
writes. Fix, or record as accepted with rationale.

---

## P2.4 — Make sensitive-content detection Unicode-aware

**Status:** PENDING
**Files:** `src-tauri/src/app_core/planner_redaction/sensitive.rs`

### Problem

`contains_long_digit_sequence` counts only `character.is_ascii_digit()`, so a card
number rendered with fullwidth (U+FF10–FF19) or Arabic-Indic digits produces a run
of zero. That single function gates three protections: sensitive-text redaction,
`contains_high_risk_page_text` (and therefore automatic high-risk blocking), and
the post-consent high-risk re-check. The marker lists have the same shape —
`to_ascii_lowercase()` folds only A–Z, so a Cyrillic homoglyph in `pаssword=`
misses every marker. `contains_ssn_shape` operates on raw bytes with the same
limitation.

This is defence-in-depth behind the consent gate, not a full bypass — but it is
the mechanism that makes high-risk pages block automatically, and a hostile or
merely internationalized page defeats it trivially.

### P2.4.1 — Normalize before matching

NFKC-normalize and case-fold (not `to_ascii_lowercase`) before marker matching, and
use `char::is_numeric()` in the run counter. Verify the normalization does not
introduce a denial-of-service on very large page text — it runs on the sanitize
path.

### P2.4.2 — Tests

Add hostile-corpus cases: fullwidth digits, Arabic-Indic digits, Cyrillic
homoglyph markers, and a credential-shaped token with no surrounding whitespace
(`is_credential_shaped_token` currently splits on `split_whitespace()`).

---

## P2.5 — Correct the sanitization metadata arithmetic

**Status:** PENDING · `[VERIFIED]`
**Files:** `src-tauri/src/app_core/planner_redaction/relevance.rs`

### Problem

Both counters are computed from `elements.len()` where they should use
`visible.len()`:

```rust
metadata.relevance_filtered_elements += elements.len().saturating_sub(selected.len());
metadata.omitted_elements            += elements.len().saturating_sub(limit);
```

With 50 elements, 30 hidden, limit 40: it reports "30 relevance-filtered, 10
omitted" when the true answer is 0 and 0 — double-counting the hidden elements and
claiming the cap bound when it did not. These fields ride in
`untrusted_data.sanitization` and reach the planner, degrading its ability to reason
about withheld evidence, in a subsystem whose whole job is accurate disclosure.

The regions path has a related defect: `relevance_filtered_regions` and
`omitted_regions` are computed such that they are always equal when the limit binds
and both zero otherwise — two differently-named fields that can never differ.

Note: the user-facing consent counts (`RemotePlannerDisclosureCounts`) are computed
separately from real post-sanitization lengths and are **correct**. This is
planner-facing metadata only.

### P2.5.1 — Fix both element counters

Compute against `visible.len()`.

### P2.5.2 — Fix or collapse the region counters

Either give them distinct meanings or collapse them into one field.

### P2.5.3 — Tests

Assert the exact counts for the 50/30-hidden/limit-40 case above.

---

## P2.6 — Resolve the "Allow always" snapshot invalidation

**Status:** PENDING · `[VERIFY FIRST]`
**Files:**

- `src-tauri/src/app_core/remote_data_consent/mod.rs`
- `src-tauri/src/app_core/planning_snapshot.rs`

### Problem (to verify)

`relevant_config_fingerprint` hashes `remote_planner_privacy` into the runtime
state token. `AllowPersistent` writes a new origin rule into that very config
*before* returning the authorized request, so the planning snapshot captured
before the challenge no longer validates — reportedly yielding `NeedsReplan`
instead of executing. `AllowOnce` and `AllowSession` are unaffected (ephemeral
grants are not in the fingerprint).

A reviewer reported confirming this with a probe test capturing differing tokens
and fingerprints before/after. Not independently re-derived.

### P2.6.1 — Reproduce

Exercise the `AllowPersistent` path end-to-end and confirm the first plan fails to
execute. Record the result.

### P2.6.2 — Fix if confirmed

Either re-capture the planning snapshot after `persist_origin_rule`, or exclude
`remote_planner_privacy` from `relevant_config_fingerprint` and bind it separately
(it is already bound via the policy re-evaluation on the post-consent path).
Prefer whichever keeps the fingerprint's meaning coherent.

### P2.6.3 — Test

`policy_and_disclosure_matrix_tests.rs` currently stops at asserting the
authorization kind and never reaches execution — extend it through execution so a
regression is caught.

---

## P2.7 — Cache the whisper context across utterances

**Status:** PENDING
**Files:** `src-tauri/src/asr/local.rs`

### Problem

`transcribe_with_whisper` constructs a fresh `WhisperContext` on every call; there
is no cache anywhere in the ASR module. The manifest pins models from 78 MB
(`ggml-tiny.bin`) to **3.09 GB** (`ggml-large-v3.bin`), so every spoken command
pays a full model load before transcription starts. TTS already does this correctly
with `CachedLocalTtsModel` keyed on `model_dir`.

Purely performance, but in a voice-first app every command goes through this path.

### P2.7.1 — Add a process-level cache

A `OnceLock` / `Mutex<Option<CachedWhisperContext>>` keyed on model path, reloaded
only when the path changes. Note that the CR2 lock-scoping refactor made
`transcribe_local` a free function specifically so it holds no controller state —
keep that property; do not reintroduce `AppCore` coupling.

### P2.7.2 — Verify interaction with P1.2

The cache must not reintroduce a lock held across model load.

---

## P2.8 — Reduce the duplication that hides validation gaps

**Status:** PENDING
**Files:**

- `src-tauri/src/app_core/command_dispatch.rs`
- `src-tauri/src/config/persistence.rs`
- `src/settings-panels/shared-controls.tsx` and callers

### Problem

Three duplication clusters that directly enable the defect classes above:

1. `build_planner_resolution` repeats an identical
   `validate_planner_output_with_safety(...)?; return Ok(...)` block **thirteen
   times**. A fourteenth resolver that forgets the validation call would fail
   silently.
2. Ten near-identical config persisters with four different validation postures —
   the direct cause of P0.3.
3. Frontend: `FOCUS_RING` is defined verbatim in five files and
   `DISMISS_BUTTON_CLASS` in three (plus two inlined copies) — which is precisely
   what `shared-controls.tsx` exists to hold.

### P2.8.1 — Collapse the resolver ladder

Introduce a small abstraction so validation cannot be omitted by construction.

### P2.8.2 — Collapse the persisters

A single `mutate_config_document(path, validate, |doc| ...)` helper. Do this only
after P0.3 lands, so the fix is not entangled with the refactor.

### P2.8.3 — Consolidate the frontend class constants

Move `FOCUS_RING` and `DISMISS_BUTTON_CLASS` into `shared-controls.tsx` and import
them. Do this together with P0.9 so the files are only touched once.

### P2.8.4 — Remove confirmed dead code

`renderOpenAiApiKeysLink` + `escapeHtml` (the latter referenced only by the
former), `preserveActivePanelControl`, `renderSettingsProviderFailoverPanelNode`,
and the duplicate `createExecutionUiStore` in `planner-orchestration.ts` (the real
adapter is `ui-store.ts`). Confirm each has no production reference before removing.

---

## P3.1 — Fix `docs/SPECS.md` drift

**Status:** PENDING
**Files:** `docs/SPECS.md`, `config.example.toml`

### Problem

The block labelled "Exact initial example contents" will not load. Confirmed
divergences:

| SPECS.md | Actual requirement |
|---|---|
| `[remote_profiles.planner.openai-default]` | flat `[remote_profiles.openai-default]` |
| `provider = "openai"` | `"OpenAi"` (no `rename_all`) |
| `temperature = 0.2` | `temperature_milli = 200` |
| `style = "short"` | `"Short"` |
| `[audio]` without `default_tts_voice` | required, no serde default → load fails |
| no `[remote_planner_privacy]` | present in `config.example.toml` |

The last matters most: CLAUDE.md names `[remote_planner_privacy]` as part of the
documented schema, but it appears nowhere in SPECS.md.

Separately, `SPECS.md` documents validation rules that are not implemented —
positive `timeout_ms` / `max_output_tokens` / `threads` / `sample_rate`, required
`model_path`, clamped temperature, and **`always_confirm_submit` must remain
`true` in v1**. A hand-edited `always_confirm_submit = false` is currently accepted
by the loader, weakening a stated safety invariant with no error.

### P3.1.1 — Point at the real artifact

Replace the inline example with a reference to `config.example.toml`, which is
accurate, rather than maintaining a second copy that drifts.

### P3.1.2 — Implement or retract the documented validation rules

For each rule in the "validation" section: implement it, or remove it from the doc.
**`always_confirm_submit` should be implemented, not retracted** — it is a safety
invariant. Note that P1.1 may add privacy validation in the same file.

---

## P3.2 — Resolve the `high_risk_origin_policy` phantom knob

**Status:** PENDING
**Files:** `src-tauri/src/config/types.rs`, `src-tauri/src/app_core/remote_data_consent/policy.rs`, `src-tauri/src/app_core/settings_adapters.rs`

### Problem

`HighRiskOriginPolicy` has exactly one variant (`Block`),
`evaluate_remote_planner_policy` never reads
`privacy.high_risk_origin_policy`, and `runtime_config.rs` force-overwrites it to
`Block` on every legacy write. Yet CLAUDE.md lists it as one of three consent
rules, `settings_adapters.rs` marshals it to the UI, and `safety_handlers.rs`
hard-codes `"block"` in the response.

Fail-closed, so not a security bug — but the config surface advertises a control
that is not wired.

### P3.2.1 — Decide: wire it or retract it

Either give the enum meaningful variants and read it in the policy evaluator, or
remove it from the config surface, the UI marshalling, and CLAUDE.md's description
of the consent rules. Do not leave it advertised and inert.

Related: four legacy privacy fields (`consent_to_remote_page_data`, `local_only`,
`blocked_origins`) are unconditionally overwritten from `network_mode` /
`origin_rules` on normalize, yet are still serialized and surfaced to the UI, so
nothing in the data says which is authoritative. Give them a dated removal plan or
they become permanent.

---

## P3.3 — Pre-existing visual issues (not migration regressions)

**Status:** PENDING
**Files:** `src/app-shell.tsx`, `src/settings-panels/planner-privacy.tsx`, `src/remote-planner-privacy-ui.tsx`, `src/settings-panels/runtime.tsx`, `src/confirmation-panels/confirmation.tsx`

### Problem

Confirmed present before `af89a22` (verified against `af89a22^`), so **not** caused
by the Tailwind migration:

- **Unclassed headings render as body text.** Tailwind preflight sets
  `h1..h6{font-size:inherit;font-weight:inherit}`. The four settings-subpage
  `<h2>`s, `app-alert-panel.tsx`, and eight headings in `planner-privacy.tsx` carry
  no size/weight utility, so visual hierarchy across the privacy card and every
  settings subpage is flat. Screen-reader semantics are unaffected.
- **Guidance CTA contrast failure.** `GUIDANCE_ACTION_BUTTON_CLASS` is a dark-green
  gradient with no `text-*`, so the label inherits `#1d1a16` — dark on dark, well
  below 4.5:1, on the primary remediation CTA.
- **Permanent empty error strip.** The always-mounted `aria-live` container in
  `confirmation.tsx` renders padding plus a 1px red border even with no error.

### P3.3.1 — Add a heading scale

Define heading utilities once (or a small set of shared constants) and apply them.

### P3.3.2 — Fix the CTA contrast

Add an explicit light `text-*` to the guidance button and verify ≥ 4.5:1.

### P3.3.3 — Collapse the empty error container

Keep the `aria-live` region mounted (that is deliberate and correct) but render no
visible box when there is no error.

---

## Completion criteria

This pass is complete when:

- Every `P0` and `P1` item is `DONE` or explicitly `BLOCKED` with a recorded
  reason.
- Every `[VERIFY FIRST]` item has its reproduction result recorded, and is either
  fixed or closed as not-reproducing.
- The full validation gate passes in the developer environment.
- `memory.md` is updated with the outcome.
