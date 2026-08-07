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

**Status:** DONE · `[VERIFIED]`
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

**Status:** DONE · `[VERIFIED]`
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

**Status:** DONE · `[VERIFIED]`

**Note:** the P0.3.2 audit found a second instance of the same bug class:
`set_active_tts_profile` (`app_core/runtime_config.rs`) wrote a caller-supplied
TTS profile name into `ProviderSelection` with no check that the profile exists,
reachable via the `set_tts_model_selection` command. `persist_tts_provider_selection_at_path`
itself validates nothing profile-specific (it only validates the document is
table-shaped), so an unknown profile name would land on disk before the
persister's own reload caught it on the next load. Fixed alongside safety/OCR by
checking `local_tts_profiles`/`remote_tts_profiles` before persisting. This fix
does not have a dedicated regression test — exercising it requires the full
`AppCore::new(app.handle())` isolated-Wry test harness (see P2.1's scope), which
was judged disproportionate for a single `contains_key` guard. Flagged here for
whoever picks up P2.1.
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

**Status:** DONE · `[VERIFIED]`

**Note:** the literal P0.4.2 fix (delete the `candidates.len() == 1` special
case, always call `determine_find_element_resolution`) was tried first and
reverted after running the test suite: `field_fill`/`field_focus` queries only
ever populate the `description` field, whose scoring ceiling (an exact
accessible-name/text/placeholder match) is 4300 bps -- well below the
production-default 9000 bps (0.90) threshold. Removing the special case
outright would have made *every* direct fill/focus command require
confirmation, including fully unambiguous ones, which is a severe UX
regression the spec's "verify the intended direct-fill UX still works for a
genuinely confident single match" caveat anticipated. The actual fix adds
`is_exact_identity_match` (element_scoring.rs) and gates the sole-candidate
fast path on it: an exact identity match still resolves directly; a sole
match that only scored via fuzzy lexical overlap now correctly falls through
to the threshold-gated resolution. All 5 previously-passing fixture tests
that exercised the exact-match path continue to pass unmodified.
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

**Status:** DONE · `[VERIFIED]`

**Note:** P0.5.1's suggestion to keep the element id available for
focus-after-navigate was not implemented -- no such capability existed
before this fix (the id was only ever used, incorrectly, as a settings-view
value), there is no existing timing idiom in this codebase for
post-render DOM focus, and some target ids are heading ids (`titleId`,
used only for `aria-labelledby`) rather than focusable controls. Scoped
this to the required behavior: navigate to a valid view, fail safe
otherwise. A dedicated focus-after-navigate enhancement can be proposed
separately if wanted.
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

**Status:** DONE · `[VERIFIED]`

**Note:** implemented option (a) from P0.6.1 -- hoisted
`renderPanelContent("confirmation-panel", panelContent)` out of the
workspace-only section to render unconditionally as a sibling of both the
workspace and settings sections, right after the app-alert panel. Option
(b) (auto-switch `appView`) was not used, per the spec's own caveat about
moving the user's view without them asking in a voice-first UI. The
existing focus-trap modules needed no changes -- they already worked
correctly on the dialog's own DOM subtree; the bug was purely that the
subtree was hidden/unreachable via its ancestor, not that the trap logic
was broken.
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

**Status:** DONE · `[VERIFIED]`
**Note:** Implemented as specified. `TtsRuntimeError::EmptySynthesizedAudio` is
returned from `synthesize_narration` after the provider call and before the
cache insert, covering both providers from one call site (rather than
duplicating the guard in `local.rs`/`remote.rs`). `store_cached_speech` also
carries its own empty-guard as defense in depth. P0.7.2: the resulting
`ToolError` (`tts_empty_synthesized_audio`, `retryable: false`) flows through
the same `tts_runtime_error_to_tool_error` → caller `?` path already used by
every other `TtsRuntimeError` variant (e.g. `RemoteHttpStatus`,
`EmptyNarrationText`) in `narration.rs`'s `begin_feedback_narration`/region
narration — no new or different surfacing path was needed or introduced.
Tests: an end-to-end remote-path test via the existing mock-server harness
(a synthetic WAV with a valid header but a zero-length `data` chunk, proven
first to actually decode to zero samples) asserts `synthesize_narration`
returns the new error; a narrower unit test asserts `store_cached_speech`
never retains or serves back an empty-sample result. The local-provider path
is not separately end-to-end tested (would require a real KittenTTS model
load), but shares the same single guard in `synthesize_narration`, so it is
covered by construction, not by a duplicate test.
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

**Status:** DONE · `[VERIFIED]`
**Note:** Implemented as specified, with the reset driven explicitly by
`stop_mode`/`auto_stop` (the caller-known signal) rather than any state
inferred inside `begin_capture`/`capture_audio` — `begin_capture` now takes
`auto_stop: bool` so both the phased handler path
(`AppCore::begin_transcribe_command`) and the synchronous planner-dispatched
path (`AppCore::execute_transcribe_command` → `AsrController::capture_audio`)
share one `reset_hands_free_capture_window` method, so the two paths cannot
drift apart. P0.8.2's cap is threaded from `cpal::StreamConfig` into the
capture callback as `MAX_TRANSCRIBE_DURATION_MS` worth of samples at the
device's actual sample rate/channel count, dropping the oldest samples once
exceeded. P0.8.3 (PTT unaffected) was verified by reading the call path —
`reset_hands_free_capture_window` short-circuits to `Ok(())` before touching
the buffer whenever `auto_stop = true`, which is exactly the PTT-release
case — rather than by an automated test: exercising the "already-buffered
session" branch end-to-end needs a live `CaptureSession`, which needs real
audio hardware unavailable in CI (the existing pre-CR3 test suite for this
module already avoids ever calling `CaptureSession::start()` for the same
reason). P0.8.4's regression tests are written at the two levels that don't
require hardware: `asr::capture`'s buffer-level tests
(`discard_capture_buffer`, `two_consecutive_hands_free_windows_do_not_return_overlapping_samples`,
`cap_buffered_samples_*`, `max_buffered_samples_*`) cover the actual discard
and cap primitives production code calls, and `asr::tests::reset_hands_free_capture_window_is_a_no_op_with_no_active_session`
covers the no-active-session branch of the policy method itself.
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

**Status:** DONE · `[VERIFIED]`
**Note:** All four fixed as specified.
P0.9.1: every `border-[var(--card-border)]`/`border-[var(--inner-card-border)]`
colour-utility usage (6 files) is now `[border:var(--x)]`; the now-redundant
paired `border` width utility was removed alongside each. P0.9.2: the
browser-visibility toggle's pressed/unpressed states are now two complete,
exported, mutually-exclusive class constants
(`TOGGLE_BUTTON_PRESSED_CLASS`/`_UNPRESSED_CLASS` in `workspace.tsx`) selected
by ternary, not a base string plus a conditional delta. P0.9.3:
`renderReadOnlyCard` now uses its own complete `CONTROL_CARD_READONLY_CLASS`
(a `CONTROL_CARD_STRUCTURE` with no background of its own, shared by both
`CONTROL_CARD` and the read-only variant) instead of appending a background
on top of `CONTROL_CARD`'s. P0.9.4: all four `max-sm:` occurrences in the
remote-privacy/consent constants are now `max-md:` (768px). P0.9.5: added
`src/tailwind-cascade.test.mjs`, which compiles the project's actual exported
class-string constants through Tailwind v4's own `compile()`/`build()` API
(no Vite/PostCSS pipeline needed — narrow and fast, ~2ms/assertion) and
asserts on the emitted declarations directly: the shorthand-vs-`border-color`
distinction, each toggle/card variant's declarations in isolation, and the
768px vs 640px breakpoint. A companion `git grep --untracked` test guards the
whole `src/` tree against the broken `border-[var(--x)]` form ever
reappearing outside the six touched files. One implementation-affecting
discovery along the way: Tailwind's `build()` is an *incremental*-rebuild
API — candidates accumulate across calls on the same `compile()` instance
rather than each call returning only that call's own CSS — so the test
recompiles fresh per assertion rather than sharing one `compile()` across the
file, or an earlier assertion's utilities silently leak into a later one's
output.
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

**Status:** PARTIAL · narration (remote TTS) DONE end-to-end; remote ASR
DEFERRED to a follow-up (scoped below, groundwork already in place).
**Note:** Scope check with the user (mid-implementation, once the real size
became clear) confirmed the full interactive dialog, not the cheaper
fail-closed-only or narrow-enforcement alternatives — see the design notes
below for what that ruled out.

**What shipped for narration:**
- `evaluate_remote_planner_policy` is reused **completely unchanged** for
  narration — it already took no planner-specific types, so "generalizing"
  it (P1.1.1) turned out to mean "call it a second time," not rewrite it.
- `AppConfig.remote_narration_privacy: RemotePlannerPrivacySettings` — an
  independent network-mode/origin-rules instance, config-schema documented
  in `config.example.toml`, so a planner-only grant can never silently
  authorize narration text leaving the device too (a real privacy question,
  not just plumbing — a user who trusts an origin with sanitized planner
  context has not thereby agreed to raw page text going to a TTS vendor).
- `AppCore` gained parallel (not unified) `remote_narration_ephemeral_grants`
  / `pending_narration_consent` state, chosen deliberately over folding into
  the existing `pending_remote_planner_consent` slot/types to avoid touching
  the planner's own delicate, already-well-tested consent code at all — a
  narrower, lower-risk diff for an "active, current-focus" subsystem.
- `remote_data_consent::grants`/`origin_rules` were generalized in place
  (a `RemoteDataDisclosureKind` selector, not three reimplementations);
  `challenge.rs` grew one shared `build_consent_challenge_from_fields` +
  a small per-kind field-extraction wrapper.
- `app_core::narration.rs`'s `begin_region_narration`/`begin_feedback_narration`
  are the **single choke point** every narration call site already funneled
  through (read_region, read_next_region, read_previous_region,
  report_result's spoken feedback) — gating there covered all four
  call sites with zero changes to any of them beyond one new match arm each
  for the `ConsentRequired` outcome.
- **Real interactive dialog, not just fail-closed enforcement**: when the
  policy says `ConsentRequired`, the challenge is stored as pending state
  and a new `submit_narration_consent_response` Tauri command (registered in
  `direct_command_policy.rs`'s security-evidence registry, same as every
  other networked/credential-bearing/page-context-transmitting command)
  resolves it and, if authorized, **redoes the exact paused narration**
  (`NarrationResumeContext::Region`/`Feedback`, storing the *resolved*
  region id so a stale "next"/"previous" never redirects mid-flow) by
  re-entering `begin_region_narration`/`begin_feedback_narration`, which
  proceeds this time because the freshly installed grant makes the
  re-evaluation pass.
- Two new disclosure vocabulary additions reused everywhere: a
  `NarrationText` (P1.1.4-equivalent) `RemotePlannerDisclosureClass` variant
  and `narration_text_bytes` disclosure count; `NarrationConsentResponseOutcome`
  contract type. The origin-binding decision from P1.1.2's spec text is
  resolved explicitly, not defaulted: both page-derived region text and
  `execute_report_result`'s assistant-generated feedback text are bound to
  the current page origin (feedback isn't literally page text, but it can
  echo page-derived details and describes that page, so it's gated the same
  way rather than left ungated by omission).
- Tests: a new isolated-Wry evidence test
  (`narration_consent_tests::remote_narration_consent_policy_matrix_is_fail_closed`,
  registered in `scripts/run-rust-tests-linux.sh` alongside the planner's
  own evidence tests) proves high-risk blocks remote narration, an
  origin-block rule blocks it, a planner-only origin allow does **not**
  cross-authorize narration, and a loopback endpoint stays ungated.

**What did NOT ship (deferred, not silently dropped):**
- P1.1.3 (remote ASR / microphone audio) — `execute_transcribe_command`'s
  synchronous capture-then-transcribe call is one opaque `AsrController`
  call with no separation point to insert a policy gate before the network
  send, unlike narration's already-separated choke point. Wiring it
  correctly needs that call split into phases first (mirroring the
  already-separated `begin_transcribe_command`/`drain_transcribe_command`
  path), which is its own well-scoped follow-up, not a design ambiguity.
  The disclosure-kind-generic policy/grants/origin-rules/challenge
  machinery built here is already shaped to add a `MicrophoneAudio`
  kind back onto — the machinery originally written for it during this pass
  was removed rather than left half-wired (dead code with no execution
  path), consistent with not shipping stubs; re-add it alongside the actual
  gate rather than ahead of it.
- P1.1.5 (settings surfacing) — `config.example.toml` documents the new
  `[remote_narration_privacy]`/`[remote_microphone_privacy]` sections, but
  there is no in-app settings UI yet to view/manage narration's origin
  rules (the planner has one: `settings-panels/planner-privacy.tsx`).
  Users on the default `ask_per_origin` mode can only grant narration access
  by editing `config.toml` directly until this lands.
- Frontend dialog wiring — the backend embeds the full
  `RemotePlannerConsentChallenge` in the `remote_data_consent_required`
  `ToolError`'s `details` (since no "fetch the pending challenge" status
  query exists yet for this disclosure kind), so a future pass has
  everything needed to reuse `remote-planner-privacy-ui.tsx`'s existing
  dialog rendering (already keyed by a `Record<DisclosureClass, string>`
  label lookup, so adding the `NarrationText` label is small) — but no
  frontend code was written to detect the error, show the dialog, or call
  `submit_narration_consent_response`. **Practical consequence**: under the
  default `ask_per_origin` mode, remote narration for a not-yet-granted
  origin currently fails with a clear, honest error and no way to grant
  access from the UI, until either the settings UI or the dialog wiring
  above lands. This is a real UX gap, not a security one — the gate fails
  closed either way.

**Original problem/required-behavior text preserved below for reference.**
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

**Status:** BLOCKED · `[VERIFIED]`
**Note:** Both P1.2.1 (TTS) and P1.2.2 (planner-driven transcription) hit the
same structural blocker, confirmed by reading the call path rather than
assumed: `LockScopedReplanningRuntime::execute_plan`
(`replanning_orchestrator.rs`) acquires the lock **once** and calls
`AppCore::execute_planner_output` → `execute_planner_output_with_runtime_safety`,
which iterates every step of a multi-step plan synchronously in one call
while still holding that single guard. Both remote TTS synthesis
(`begin_region_narration`/`begin_feedback_narration`, reached when a plan
step is a narration tool) and planner-driven `transcribe_command` are
individual steps *inside* that loop, not separate top-level calls — so
releasing the lock around just one step's blocking work would require the
step loop itself to become pausable/resumable (yield control back to the
lock-holding caller mid-plan, then resume), not a local change to either
callee. This is the same shape of problem CR2's P1.1.2/P1.1.3 hit and marked
BLOCKED for the same reason; `run_phased_transcribe` (which already solves
this) works only because it sits *outside* the step loop, at the top-level
Tauri-command handler for a single `transcribe_command`/
`transcribe_and_execute_command` call, not for a step embedded in a larger
plan.
**What already doesn't have this problem:** the top-level, non-plan-embedded
paths (`transcribe_command`/`transcribe_and_execute_command` via
`run_phased_transcribe`, and the remote planner's own resolve round-trip via
`LockScopedReplanningRuntime::resolve`) already release the lock correctly —
this item is specifically about TTS/ASR steps reached *from inside* an
executing plan.
**Follow-up scope**: restructuring `execute_planner_output_with_runtime_safety`
into a step-by-step pausable loop (so the top-level handler can drop the
lock between steps whenever the next step needs blocking I/O) is a
significant executor change, not a narrow fix — sizing and design should be
its own pass, informed by how `AwaitingConfirmation`'s existing
pause/resume (`PendingPlanExecutionState`) already models suspending
mid-plan, which may be the right template to generalize from.
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

**Status:** DONE · `[VERIFIED — reproduced empirically]`
**Note:** P1.3.1's reproduction was done, not skipped, using a real headless
Chromium session over raw CDP (not through this app, since it has no
live-browser test harness to reuse): a 3000px-tall page with marker
elements, scrolled to y=1500, `getBoundingClientRect()` read for a marker at
document `(50, 1550)` (→ viewport-relative `(50, 50)`), and three
`Page.captureScreenshot` calls compared:
- raw viewport-relative `clip=(50,50)`, `captureBeyondViewport` unset →
  **blank** (wrong region).
- document-absolute `clip=(50, 1550)` → **correct**, lands on the marker.
- raw viewport-relative `clip=(50,50)` with `captureBeyondViewport=true` →
  **wrong**, lands on an unrelated marker sitting at the document origin.

This conclusively confirms `Page.captureScreenshot`'s `clip.x`/`clip.y` are
always document/page-absolute, independent of `captureBeyondViewport` —
matching the predicted failure exactly: passing a raw
`getBoundingClientRect()` bbox as `clip` silently captures the wrong region
whenever the page is scrolled.

**Fix (P1.3.2)**, at extraction (`browser/dom_extraction.rs`), as preferred
so every consumer inherits it: a `documentAbsoluteRect(rect)` helper adds
`window.scrollX`/`window.scrollY` to `rect.x`/`rect.y` once, used by both
bbox sources (interactive-element bbox and region bbox). `page_model::Rect`
now documents the coordinate-space contract explicitly. No consumer-side
change was needed for the CDP `clip` path (`page_inspection.rs`, now
commented explaining why) or for the dominant, actually-used OCR fallback
path (`extraction_tools/page_extraction.rs`'s region-first OCR always
sources from a `ScreenshotScope::FullPage` capture, whose raster origin is
already the document origin) — both now line up with document-absolute
bboxes automatically.

**P1.3.3 confirmed as a correct, incidental mitigation, not a second bug**:
`extractor.rs`'s dom_smoothie regions set `bbox: None` (no live DOM to
measure offline), and `region_first_ocr_target_ids`/`has_positive_bbox`
already exclude `None` bboxes from region-first OCR targeting — that path
was never exposed to this bug in the first place, so nothing needed fixing
there beyond confirming it.

**Residual gap found but left out of scope**: `execute_run_ocr` accepts an
arbitrary caller-supplied `image_id` plus `region_id`, with no check that
the underlying persisted image was a full-page capture. If a plain viewport
screenshot (`capture_screenshot` with no scope/bbox, at whatever scroll
position was active then) is later OCR'd by `region_id`, the now-document-
absolute bbox doesn't line up with that raster's pixel origin, since
neither `BrowserScreenshotState` nor the image cache record the scroll
offset active at capture time. This is a real edge case, but distinct from
and narrower than the review's described failure (which is about the
region-capture and full-page-OCR-fallback paths, both now fixed) — closing
it needs new capture-time scroll metadata threaded through the image cache,
which is its own follow-up rather than bundled into this fix.

**Tests**: no live-browser (real CDP) test harness exists anywhere in this
codebase — every browser-dependent test uses a mock executor instead — so
building one as a first-of-its-kind permanent CI test was judged out of
proportion to this fix; the empirical CDP reproduction above stands as the
recorded verification instead, per P1.3.1's instruction to "record the
result here." Added a narrower, always-runnable regression test instead:
`dom_extraction::tests::extraction_script_corrects_both_bbox_sources_for_scroll`
asserts (by string content) that the extraction script's two bbox sources
both route through `documentAbsoluteRect`, that no raw `rect.x`/`rect.y`
bbox construction exists outside that helper, and that the helper itself
adds the scroll offset — narrow, but catches someone reintroducing the
exact shape of this bug (a raw `getBoundingClientRect()` bbox) even without
a live page.

**Original problem text preserved below for reference.**
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

**Status:** DONE · `[VERIFIED — confirmed exploitable, then closed]`
**Note:** P1.4.1's reproduction confirmed the gap is real and reachable, not
theoretical: `execute_planner_output` is a directly Tauri-invocable command
(`command_handlers/core_handlers.rs`) taking a `PlannerOutput` deserialized
straight from the IPC payload, with no session/origin binding to a prior
`resolve_command` call. `planner_output_requires_snapshot`
(`app_core/planning_snapshot.rs`) — the only gate that could otherwise
reject an unbound/forged plan — excluded `ExtractPageModel`, `ReportResult`,
and `TranscribeCommand`, so a single-step `ExtractPageModel`-only plan
skipped `validate_and_consume_planning_snapshot` entirely (returns `Ok(())`
immediately with no digest lookup) and reached
`execute_extract_page_model` → `mark_page_model_changed()`, which clears
`pending_confirmation_id`/`pending_plan_execution`/every click
authorization unconditionally.

**Fix diverges from this doc's stated preference, with reasons recorded
here** (per the "VERIFY FIRST" instruction to record what verification
actually shows): P1.4.2 said "prefer [calling
`validate_planner_output_with_safety`] — it restores the layering." Tracing
what that call would actually reject proved it does **not** close this gap:
`ExtractPageModel` is itself classified `ReadOnly`/`NoConfirmation` by
`action_policy.rs`'s `tool_policy`, so a bare `ExtractPageModel`-only plan
passes every check `validate_planner_output_with_safety` runs (step
structure, tool availability, policy) — that function was never designed to
reject "this tool by itself," only malformed or policy-prohibited plans.
Implemented the doc's second-listed option instead — extending
`planner_output_requires_snapshot` to include `ExtractPageModel`,
`ReportResult`, `TranscribeCommand` — because `validate_and_consume_planning_snapshot`
requires a digest-bound match against a snapshot only `register_planning_snapshot`
creates (and only `resolve_command`/the replanning loop calls that), and
`planner_output_digest` hashes the *entire* serialized `PlannerOutput` via
SHA-256. This is a provenance check, not a structural/policy one: it
guarantees only a `PlannerOutput` a real planning call actually produced —
byte-identical — can ever reach execution, which is what actually closes an
out-of-band/forged-input gap. Also added `validate_planner_output_with_safety`
as defense-in-depth was considered and explicitly deferred: doing it
correctly needs `active_skill_names` re-derived at execute time (the check
validates `planner_output.selected_skills` against what was active when
planned), which `PlanningStateSnapshot` doesn't currently carry — adding it
naively with an empty list would false-positive-reject any legitimate
snapshot-bound plan that uses skills. Threading `active_skill_names` through
the snapshot properly is a reasonable follow-up, not required to close the
verified gap.

**P1.4.3 addressed as documentation, not a classification change**: making
`ExtractPageModel`/`TranscribeCommand` require confirmation was considered
and rejected — `ExtractPageModel` runs on essentially every page interaction
(a voice-first "look at the page" primitive), so confirming every call would
be a severe usability regression, not a proportionate fix for an
out-of-band-invocation problem the snapshot binding already solves. Instead,
added comments at both misclassification sites
(`commands/planner_executor/tool_dispatch.rs`'s `is_side_effecting_tool`,
`commands/action_policy.rs`'s `tool_policy`) explicitly stating what each
classification does and does not guard, pointing future readers at
`planner_output_requires_snapshot` as the actual authority on unauthenticated
invocation — so "not side-effecting"/`NoConfirmation` here is never again
mistaken for "safe to invoke out-of-band."

**Tests**: `planning_snapshot.rs::tests::extract_page_model_report_result_and_transcribe_command_now_require_snapshot`
asserts all three tools now require a bound snapshot. Ran the full suite
after the fix (no existing test relied on the old permissive behavior — 0
regressions across 521 Rust tests), consistent with the gap being genuinely
unauthenticated/never legitimately exercised standalone before.

**Original problem text preserved below for reference.**
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

**Status:** DONE
**Note:** Added `click_authorization_subsystem_is_fail_closed`, one
isolated-Wry evidence test (`app_core::click_authorization::tests::app_core_evidence_tests`,
mirroring `confirmation_replay_tests.rs`'s established pattern — a real
`AppCore::new(app.handle())`, `#[ignore]`d and registered in
`scripts/run-rust-tests-linux.sh`) exercising all six required scenarios
against the real `AppCore` methods, not just the free-function unit tests
that existed before:
1. A forged, never-minted token → `unknown_click_authorization`.
2. A forged `_runtime_click_ambiguous: false` claim over a genuinely
   unresolved element (no prior authorization) → stripped by
   `clear_runtime_annotations` and correctly re-derived to `true`.
3. A `Ready` all-click plan → the deterministic `ConfirmAction` gate is
   actually inserted at index 0 and `status`/`requires_confirmation` flip.
4. One valid token reused across two click steps in the same plan →
   `duplicate_click_authorization`.
5. Expiry and page-generation staleness are both enforced, as two separate
   sub-cases (`click_authorization_expired`, `stale_click_authorization`).
6. The fingerprint-mismatch rejection (`click_target_changed`) fires when
   the authorized element's identity changes.

**One implementation-affecting discovery**: the literal case 5 ask
("expiry ... enforced") needed a different entry point than initially
tried. `prepare_planner_output_for_execution` eagerly prunes expired tokens
from the store *before* processing any step (`self.prune_click_authorizations()`
at its top) — so a statically pre-expired token is already gone by the time
the per-step check runs, and the call surfaces `unknown_click_authorization`
instead of `click_authorization_expired`. These are genuinely different,
both-real error codes (unknown = never existed here; expired = existed, then
lapsed), not equivalent — caught only by running the test and reading the
actual failure, not by reasoning alone. Switched that one sub-case to
`preflight_pending_click_authorizations`, which doesn't prune first and so
reaches the expiry check the scenario is actually about; the other five
sub-cases use `prepare_planner_output_for_execution` as originally intended.

**Case 6 (live-DOM fingerprint mismatch) is tested at the reachable layer,
not literally through a live browser**: `verify_element_matches_record` — the
exact comparison function the live-DOM re-check (`validate_live_dom: true`)
uses — is *first* run against the current **stored** page model
(`self.state.current_page`, no browser needed) before the live-DOM branch is
ever reached; a mismatch there returns `click_target_changed` through the
identical code path. This codebase has no live-browser (real CDP/chromiumoxide)
test harness anywhere (confirmed again during this pass, same constraint as
CR3 P1.3), so exercising the *live* half of that check specifically remains
untested; the shared rejection logic it depends on is not.
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

**Status:** DONE
**Note:** Added `verify_isolated_wry_test_list_is_complete` to
`scripts/run-rust-tests-linux.sh`, run before any isolated test: it derives
the real `#[ignore]`d test set from `cargo test -- --ignored --list`,
diffs it against the (now array-form, still ordered/explicit)
`ISOLATED_WRY_TESTS` list, and fails loudly with the diff on any divergence
in either direction — a test added without a matching entry, or an entry
for a test that no longer exists. The individual `run_isolated_wry_test`
calls were also collapsed into a loop over the same array, so there is now
exactly one place that needs updating when a new isolated test is added,
not two. Verified both directions by hand: the happy-path run reports "PASS:
... (9 tests)"; temporarily deleting one entry from the array reproduces the
expected failure with a readable diff and exit code 1. P2.2.2 (note P1.1's
new tests) is satisfied by construction — `narration_consent_tests`'s
evidence test and P2.1's new click-authorization evidence test are both
already in the array, and the completeness check would have caught it if
either had been missed.
**Out of scope, by the TODO's own file list**: `check-remote-planner-privacy-state.py`'s
`REQUIRED_PATHS` and the historical `test:ui` glob are cited in the Problem
section as prior instances of the same bug shape, not additional
deliverables of this item (`Files:` names only `run-rust-tests-linux.sh`).
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

**Status:** DONE
**Note:** `write_config_atomic`'s temp file is now opened with
`OpenOptions::new().mode(0o600)` set at creation (not via a post-creation
`set_permissions` call, which would itself be the TOCTOU window P2.3.1
warns about), and the config directory is restricted to `0700`, mirroring
`image_cache.rs`'s existing convention for its own privacy-sensitive files.
`fs::rename` preserves the source file's mode, so the renamed `config.toml`
inherits `0600` with no separate step. New unit test
(`write_config_atomic_restricts_file_and_directory_permissions`) asserts
both modes directly.

P2.3.2's two adjacent items: the missing parent-directory fsync-after-rename
was fixed in the **shared** `atomic_file::replace_file_atomically` helper
(used by both config persistence and model-management downloads, so both
inherit the durability fix from one place) rather than duplicated locally.
The fixed-temp-filename/no-concurrency-control item is recorded as
**accepted, not fixed**: every config write already funnels through the
single `Arc<Mutex<AppCore>>` guard, so no intra-process race is possible;
a cross-process race would require running two copies of this desktop app
against the same profile directory simultaneously, a scenario this app
does not otherwise defend against (no existing single-instance lock, PID
file, or similar), so adding concurrency control for just the config-write
path alone would be inconsistent with the rest of the app's threat model
rather than a genuine hardening improvement.
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

**Status:** DONE
**Note:** Implemented with one significant, verified divergence from the
literal instruction — recorded here because it changes what the fix
actually covers.

**Digit detection (P2.4.1, as specified)**: `contains_long_digit_sequence`
and `contains_ssn_shape` now use `char::is_numeric()` (Unicode decimal
digits across scripts) instead of `is_ascii_digit()`/raw-byte comparison.
`contains_ssn_shape` was rewritten from a byte-windowed scan to a
char-windowed one — the byte version wasn't unsound (UTF-8 continuation
bytes never collide with ASCII digit byte values), just blind to non-ASCII
digits. Bounded input confirmed (`MAX_REGION_TEXT_CHARS = 2_000`), so the
O(n) char-vec allocation this needs is not a DoS concern, per P2.4.1's own
instruction to check that.

**Marker matching diverges from "NFKC-normalize"**: verified empirically
(Python `unicodedata.normalize`) that NFKC does **not** fold the Cyrillic
homoglyph in the motivating example ("pаssword=" with a Cyrillic а,
U+0430) to its Latin equivalent — NFKC only unifies compatibility variants
of the *same* character (e.g. fullwidth Latin letters, which it *does*
correctly fold), not lookalikes across different scripts. Pulling in NFKC
normalization (which needs the `unicode-normalization` crate — a new
dependency, ask-first territory this pass doesn't have sign-off for) would
not have closed the gap it was proposed to close. Implemented instead: a
hand-rolled, dependency-free `fold_confusable_ascii` — a fixed `-0xFEE0`
offset for the Halfwidth/Fullwidth Forms block (mathematically identical to
what NFKC does for that block, confirmed against Python) plus a small,
explicit table of the Cyrillic/Greek letters that are lookalikes for the
specific Latin letters (a-z) this module's marker vocabulary is built from
— not general Unicode confusable detection (Unicode TR39's full skeleton
algorithm would be needed for that), scoped to what's actually being
matched against. Case folding switched from `to_ascii_lowercase()` to
`str::to_lowercase()` (full Unicode case folding) throughout.

**`is_credential_shaped_token`'s whitespace gap (implied by P2.4.2, not
explicitly listed under P2.4.1) was also fixed, not just tested**: tested
first and confirmed `key=sk-...`/`{"authorization":"ghp_..."}` were missed
entirely, since `split_whitespace()` left the credential fused to
surrounding text with no split point. Changed the tokenizer from
`split_whitespace()` to a boundary split on anything that isn't
alphanumeric or one of `-_.` (the characters a credential token is itself
built from) — correctly isolates a credential-shaped token glued to
surrounding text via `=`, `:`, quotes, or similar, while leaving genuine
JWT `.`-separated segments intact for `is_credential_shaped_token`'s own
internal split.

**Tests (P2.4.2)**: added a `#[cfg(test)] mod tests` directly in
`sensitive.rs` (it had none) rather than extending the hostile-content
JSON corpus (`hostile_content_corpus_manifest.rs`) — that corpus is scoped
to a different, adjacent concern (prompt-injection attack shapes), not
credential/PII marker detection, and its fixture format doesn't map onto
these functions' unit-level inputs. 9 new tests cover fullwidth digits,
Arabic-Indic digits, a Cyrillic homoglyph marker, a fullwidth-letter
marker, credential tokens with no surrounding whitespace, SSN shape with
Arabic-Indic digits, a fullwidth/homoglyph element descriptor, and two
negative cases (plain text, short digit runs) proving the changes didn't
make matching over-eager.
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

**Status:** DONE · `[VERIFIED]`
**Files:** `src-tauri/src/app_core/planner_redaction/relevance.rs`,
`src-tauri/src/app_core/planner_redaction/types.rs`,
`src-tauri/src/app_core/planner_redaction/tests.rs`

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

### Implementation note

- **P2.5.1**: `select_relevant_elements` now captures `visible_count` before
  consuming the `visible` vec, and computes `metadata.omitted_elements` against
  it instead of `elements.len()`. `omitted_hidden_elements` already accounted
  for the hidden elements separately, so this stops double-counting them.
- **P2.5.2**: chose **collapse**, not "give distinct meanings." Once
  `omitted_elements` is fixed to use `visible_count`, it is provably always
  equal to what `relevance_filtered_elements` was computing
  (`selected.len() == visible_count.min(limit)` from the `.take(limit)`
  construction, so both reduce to `visible_count.saturating_sub(limit)`) — the
  review's own worked example already assumes this (it asks for "0 and 0," not
  two independently-meaningful numbers). The regions path never had a
  visibility-filtering stage at all, so `relevance_filtered_regions` and
  `omitted_regions` were computing the exact same expression off the exact
  same inputs from the start; there is no distinct meaning to give it.
  `relevance_filtered_elements` and `relevance_filtered_regions` were removed
  from `SanitizationMetadata`, keeping `omitted_elements`/`omitted_regions` as
  the sole counts. Confirmed via `grep -rn` across `src-tauri/src` and `src`
  that these 4 field names were referenced only inside
  `planner_redaction/{relevance,types,tests}.rs` — no frontend TypeScript
  consumer exists — so removing a field has no cross-boundary contract to
  update. `SanitizationMetadata` is part of `RemoteUntrustedData`, which is
  serialized and sent to the remote planner LLM, so this also means the LLM
  planner receives one less redundant number to reason about.
- **P2.5.3**: added `element_relevance_selection_does_not_double_count_hidden_elements_when_limit_does_not_bind`
  (the review's own 50/30-hidden/limit-40 example, asserting `omitted_elements
  == 0`), `element_relevance_selection_counts_correctly_when_limit_binds` (50
  elements/10 hidden/limit 25, asserting `omitted_hidden_elements == 10` and
  `omitted_elements == 15`), and
  `region_relevance_selection_counts_are_correct_whether_or_not_the_limit_binds`
  (20-below-limit-40 and 70-above-limit-40 cases) to
  `planner_redaction/tests.rs`, calling `select_relevant_elements`/
  `select_relevant_regions` directly rather than through the full
  `sanitize_for_network` pipeline (whose fixed `MAX_REMOTE_ELEMENTS`/
  `MAX_REMOTE_REGIONS` limits don't match the review's example limits). The
  pre-existing `page_payload_is_deterministically_bounded` test (which
  exercises the full pipeline with all-visible elements) still passes
  unmodified, since with zero hidden elements the old and new formulas agree.
- Full validation green: `cargo fmt --check`, `cargo clippy --all-targets
  --all-features -- -D warnings`, `cargo test --all-features` (534 passed, 9
  ignored — 3 net-new tests), all 4 CI guard scripts, and
  `xvfb-run -a cargo test --all-features`. One isolated-Wry test
  (`remote_data_consent_request_counts_replay_and_concurrency_are_enforced`)
  failed once inside the full `run-rust-tests-linux.sh` sweep on an assertion
  unrelated to this change (`planner_request_failed` vs
  `planner_http_status`); reproduced as a pre-existing flake by running it in
  isolation both with and without this diff's changes (`git stash`) — passed
  both times, confirming it is not caused by this change.

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
