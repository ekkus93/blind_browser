# Blind Browser Code Review 3 Spec

## Purpose

Code Review 3 is a correctness, safety, and architecture pass driven by a full
six-subsystem review of the `master` snapshot at commit `af89a22`
(2026-08-07). It is **not** a redesign or a feature pass.

The review covered the privacy/consent layer, the command/planner layer, the
browser and deterministic tool layer, config/state/persistence, the speech
(ASR/TTS/audio) providers, and the React frontend, plus a cross-cutting pass over
hygiene, CI, and dependency surface.

It found:

- Eight correctness/safety defects reachable in normal use, including a panic, a
  configuration write that can prevent the app from starting, two paths where a
  confirmation-requiring action executes without confirmation, and two paths where
  a blind user receives silence or an uninformative prompt where the design
  intends speech.
- One architectural gap: remote TTS and remote ASR send page text and microphone
  audio to a third-party provider with no consent gate, while the remote *planner*
  path has an elaborate fail-closed one.
- Two frontend reachability failures that strand a blind user on a dead-end
  screen or make a confirmation dialog unreachable entirely.
- Four visual regressions introduced by the Tailwind migration in `af89a22`.
- A set of enabling debt items — most importantly, that the click-authorization
  subsystem carries most of the click-safety invariants and has essentially no
  tests.

Code Review 3 fixes those in priority order without regressing the existing agent
safety guarantees or the passing test suites.

## What was validated during review

Confirmed in the review environment against this snapshot:

- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` — clean.
- `cargo test --manifest-path src-tauri/Cargo.toml --all-features` — 499 passed,
  0 failed, 7 ignored (the 7 are the isolated-Wry security tests, run by
  `scripts/run-rust-tests-linux.sh`), plus 4 + 6 in the auxiliary test binaries.
- `pnpm lint` — clean.
- `pnpm test:ui` — 238 / 238 passed.
- `pnpm build` — clean (`tsc && vite build`).
- All four CI guard scripts pass: `check-silent-fallbacks.sh`,
  `check-security-fallbacks.py`, `check-security-fallback-inventory.py`,
  `check-sensitive-diagnostics.py`.

Cross-cutting hygiene measured on this snapshot: zero `unsafe`, zero
TODO/FIXME/HACK/XXX, zero `as any` / `@ts-ignore`, zero stray `console.log`, 23
panic-capable calls in true production code (excluding inline `#[cfg(test)]`
modules), 675 transitive crates, 518 Rust + 238 JS tests, Rust test:production
line ratio 0.59.

## Finding confidence

Tasks in the TODO are tagged with one of two confidence levels. **Respect these
tags** — do not treat an unverified finding as settled.

- `[VERIFIED]` — the defect was confirmed by reading the code (and, where noted,
  by inspecting compiled output) during this review. The described mechanism is
  established; only the fix needs design.
- `[VERIFY FIRST]` — the defect was reported with concrete evidence and a
  plausible mechanism, but was not independently re-derived. **Reproduce it before
  writing a fix.** If it does not reproduce, record that in the TODO and close the
  item rather than changing code speculatively.

## Current known good behavior to preserve

Do not regress these. Each was specifically traced during review and found
correct:

- **The consent gate is structurally unbypassable.** `PreparedRemotePlannerRequest`
  has no public constructor; it can only be produced by
  `RemotePlannerRequestDraft::authorize`, which is `pub(super)`. There is exactly
  one choke point into `resolve_remote_planner`. Any change to the remote-planner
  path must preserve this type-state property rather than replacing it with a
  runtime check.
- **Click authorization is a one-shot capability system.** Server-minted tokens
  bound to page id, page generation, origin, element id, DOM locator, and a
  SHA-256 element fingerprint; re-validated against a freshly re-extracted live
  DOM before dispatch; consumed on use; all tokens cleared on any page-generation
  bump.
- **Planner argument forgery is defeated by construction.**
  `clear_runtime_annotations` strips every planner-supplied `_runtime_click_*` and
  `_runtime_form_*` key, and the values are re-derived from the server-side
  authorization record.
- **Planner-authored confirmation copy is never surfaced.** It is discarded and
  replaced with deterministic manifest text.
- **Policy fails closed on missing grounding.** Absent `ambiguous` /
  `potentially_destructive` metadata defaults to `true`. `SubmitActiveForm` keeps
  an unconditional confirmation minimum that the legacy setting cannot weaken.
  `EvalJs` stays `Prohibited` and blocked at two independent layers.
- **Snapshot binding.** Plans are bound to a SHA-256 of the entire `PlannerOutput`
  and the snapshot is consumed on use, so a hand-crafted plan cannot be executed.
- **Secrets stay keyring-backed and endpoint-scoped.** `SecretRef` is an untagged
  enum of struct-only variants, so a plaintext key in `config.toml` fails
  deserialization. Credential-bearing requests keep `Policy::none()`.
- **No silent provider fallback.** ASR and TTS both match `ProviderMode`
  exhaustively with no `_ =>` arm and no local-on-remote-failure retry.
- **Model download hardening.** Stream to `.part`, enforce size bounds during the
  stream, verify SHA-256, `sync_all()`, atomic rename, and escalate cleanup
  failures.
- **Frontend decision integrity.** Every confirmation/consent mutation is gated on
  identity before it changes state; a stale confirmation id raises a visible
  transport error rather than being silently dropped.
- **The security-fallback allowlist stays exact.** Entries are pinned to
  `file|exact-expression` and CI-enforced. If a fix removes an allowlisted
  expression, remove its allowlist entry in the same change.
- All currently passing Rust and UI tests stay green.

## Non-goals

Do not:

- Redesign the planner, the tool contracts, or the confirmation UI.
- Decompose `AppCore` (noted as debt; out of scope for this pass).
- Enable provider failover (still intentionally disabled).
- Rewrite the browser/CDP integration or swap `chromiumoxide`.
- Rewrite `docs/SPECS.md` wholesale — the targeted drift fix in P3.1 is the scope.
- Introduce voice-activity detection / silence detection.
- Perform broad refactors unrelated to the tasks below.

---

## Design constraints

### 1. A spoken confirmation must name the element the user recognizes

`clear_runtime_annotations` removes `RUNTIME_TARGET_LABEL_ARG` from every step.
`annotate_target_step` restores it for `TypeIntoElement`, and `annotate_submit_step`
for `SubmitActiveForm`, but `annotate_click_step` writes back only the five
`CLICK_AUTH_*` keys and never restores the label. `safe_action_summary` therefore
falls through to `element_id`, which the DOM extractor generates positionally as
`` `element-${index + 1}` ``.

The result is that the most dangerous action class produces the least informative
prompt:

> "Approve this action on https://bank.example: Click element 'element-7'."

Required behavior: a click confirmation must name the element using the same
human-readable label the runtime already computed when it decided the click was
potentially destructive. The label must continue to flow through
`safe_element_label` so it stays length-bounded and sanitized, and it must remain
a runtime-derived value — never a planner-supplied string.

### 2. Narration cursor movement must be bounded by the current region list

`previous_region_index` returns `Some(index - 1)` with no comparison against
`region_count`, while its sibling `next_region_index` correctly checks
`index + 1 < region_count`. The consumer indexes directly.

The cursor is only reset on navigation (`record_navigation`,
`clear_navigation_follow_up_state`). `replace_current_page_model` bumps the page
generation but leaves the cursor untouched, so any re-extraction that yields fewer
regions — an SPA re-render, a cookie banner replacing content, lazy content
collapsing — leaves the cursor pointing past the end.

Required behavior: cursor movement must be total over the current region list. No
narration path may index a region vector without a bounds check, and a shrinking
region list must clamp the cursor rather than panic. A hostile page must not be
able to crash the process through this path.

### 3. Settings must be validated before they are written, not after

`persist_safety_settings_at_path` and `persist_ocr_settings_at_path` insert into
the document and call `write_config_atomic` with no validation. Validation happens
only in the trailing `load_from_path`, i.e. after the value is already on disk.
The sibling persisters (`audio`, `models`, `privacy`) all validate first and
return early. The command handlers do not clamp either.

The consequence is that a recoverable input error becomes an unrecoverable startup
failure: writing `sparse_text_char_threshold = 0` returns `Err` to the caller but
leaves the invalid value on disk, and the next launch fails validation during
`AppConfig::load_for_app`. For a blind-first application, "hand-edit the TOML" is
not a recovery path.

Required behavior: every persister validates before writing and returns
`ConfigError::Validation` without touching the file. No code path may leave
`config.toml` in a state that its own loader rejects.

### 4. Ambiguity detection must not be disabled by a degenerate candidate list

Two independent instances of the same mistake:

- `rank_find_element_candidates` truncates to `candidate_limit` **before**
  `determine_find_element_resolution` inspects `candidates.get(1)` for the
  runner-up margin. With a planner-supplied `max_candidates: 1` — which the
  validator accepts, rejecting only `0` and `> 3` — the runner-up is always
  `None`, so two identically-scoring elements resolve to a no-confirmation click.
- `field_fill.rs` and `field_focus.rs` special-case `candidates.len() == 1` and
  hardcode `requires_confirmation = false`, bypassing
  `confirmation_confidence_threshold` (default 0.90) entirely. A single weak match
  — e.g. "card number" scoring 0.205 against a lone "Number of guests" field —
  is typed into with no confirmation and no clarification.

Required behavior: the ambiguity margin must be computed over the full ranked
candidate set before any truncation, and the confidence threshold must apply
regardless of how many candidates survived scoring. A degenerate candidate list
must make the system *more* cautious, never less. This directly serves the
project rule that ambiguity prefers brief clarification over silent guessing.

### 5. Remote speech providers must be gated by the same consent layer as the planner

The remote-planner path enforces `network_mode`, `origin_rules`,
high-risk-origin blocking, sanitization, and a tamper-evident consent challenge
before any page-derived data leaves the device. Remote TTS and remote ASR enforce
none of it: `synthesize_narration` passes verbatim page region text to
`"input": text` in the OpenAI request, and remote ASR multipart-uploads raw
microphone audio. Neither path references `remote_planner_privacy`, `network_mode`,
`origin_rules`, `high_risk`, `consent`, `sanitize`, or `redact`.

The concrete failure: on a page the policy classifies high-risk, the planner is
blocked from receiving even *sanitized* region summaries, yet "read this page"
ships that page's raw, unredacted text to a third party.

Required behavior: page-derived text sent to a remote TTS provider, and microphone
audio sent to a remote ASR provider, must pass the same origin/network-mode/
high-risk policy evaluation that governs the remote planner. The consent gate must
remain a single shared decision point rather than three parallel reimplementations,
and it must retain the type-state property described under "known good behavior" —
a caller must not be able to construct an authorized remote request without going
through the policy.

Design direction (not prescriptive): lift the policy evaluation and the
authorized-request type-state out of `remote_data_consent` into a provider-agnostic
form parameterized by disclosure kind (planner payload / narration text /
microphone audio), so all three remote paths share one evaluation, one grant store,
and one consent challenge vocabulary. Consent copy must distinguish the three
disclosure kinds — "send this page's text to be spoken aloud" is a different user
decision from "send sanitized page context for planning".

Scope boundary: this constraint covers **remote** providers only. Local TTS/ASR
keep operating with no gate, since no data leaves the device.

### 6. Audio output must fail loudly rather than produce silence

`synthesize_narration` checks that the *input text* is non-empty but never checks
that the *output samples* are non-empty. Both providers can return an empty
sample vector: the pinned local TTS returns `Ok(vec![])` for punctuation-only
input such as `"..."` or `"?!"` (ordinary on the web), and a remote WAV with a
zero-length `data` chunk decodes to an empty vector. `play_samples` accepts it,
the state records `speaking = true`, the narration cursor advances, and the tool
reports success. The empty result is then cached, so re-reading the region is
silent for the life of the process.

Required behavior: empty synthesized audio must be a typed error, surfaced to the
user, and must never be cached. For an accessibility application, silence reported
as success is a worse failure than an audible error.

### 7. Microphone capture must be bounded and must not accumulate across utterances

`begin_capture` short-circuits when a session already exists and never drains the
buffer; the cpal callback appends unconditionally with no cap. Push-to-talk
depends on this accumulation (it deliberately relies on the hold-time buffer), but
the hands-free loop does not: with `auto_stop = false` the session stays alive
across the *entire* command execution — planner round trips, browser work, and TTS
playback — with the microphone open throughout.

Consequences: the application transcribes its own spoken narration into the next
command (a self-triggering loop); `MAX_TRANSCRIBE_DURATION_MS` stops bounding the
audio actually sent to ASR; remote ASR fails permanently past roughly 4.4 minutes
of session once the 8 MiB upload cap is exceeded; memory grows at roughly
1.4 GB/hour.

Note this is a **different defect** from the one fixed in Code Review 2. CR2
fixed `CaptureSession::snapshot()` cloning without clearing on the *snapshot*
path. This constraint concerns `begin_capture` not resetting between *windows* in
the hands-free loop. Do not assume CR2 already covers it.

Required behavior: a hands-free capture window must transcribe only audio captured
for that window. Push-to-talk must keep returning the full held utterance. The
shared buffer must have a hard cap so a stuck session cannot grow without bound.

### 8. The global runtime lock must not be held across network or capture windows

Two paths hold the `AppCore` mutex across multi-second blocking work:

- Remote TTS synthesis runs a blocking request with the profile timeout (default
  30 000 ms) while the lock is held, reached through the replanning orchestrator's
  tool dispatch.
- A planner-emitted `transcribe_command` step blocks for the capture window (up to
  `MAX_TRANSCRIBE_DURATION_MS`) plus the ASR round trip, also under the lock.

While either is in flight, `stop_listening` and `get_agent_state` block. A blind
user cannot stop the microphone or interrupt a hung synthesis.

The asymmetry is the point: `run_phased_transcribe` was built specifically to
release the lock across exactly these windows, and both of these paths bypass it.

Required behavior: no network round trip and no capture sleep may occur while the
`AppCore` lock is held. Apply the established phased pattern — snapshot the
configuration under the lock, perform the blocking work unlocked, re-acquire to
commit the result — and preserve the existing stop-during-window semantics
(returning `Ok(None)` when the session was dropped mid-window).

### 9. A confirmation or consent gate must be reachable whenever it can be raised

Two frontend reachability failures:

- `focusSettingsTarget` casts a DOM element id (e.g.
  `"settings-model-management-title"`) directly to `SettingsView`, whose members
  are only `overview | planner | tts | asr | runtime`. Because every subview
  renders `hidden={initialSettingsView !== "<view>"}`, an unmatched value hides
  **all five**, including `overview` — which holds the page heading and the
  guidance panel itself. The remediation path for "your speech setup is broken"
  therefore lands a blind user on an empty page whose only control is the back
  arrow.
- The `confirmation-panel` root — which hosts both the action-approval panel and
  the remote-data consent dialog — lives inside the workspace section, which
  carries `hidden` and `aria-hidden="true"` whenever the app is in Settings view.
  Push-to-talk is bound globally to `window` with no view gate and the continuous
  listening loop runs regardless of view, so a command issued from Settings can
  raise a confirmation or consent dialog into a `display:none`,
  `aria-hidden` subtree. Focus calls become no-ops, the dialog is invisible to the
  screen reader, the focus trap never engages, and the challenge silently expires.

Required behavior: navigation targets must be validated against the `SettingsView`
union rather than cast, with an unmatched target failing safe to a real view.
Confirmation and consent surfaces must be rendered outside any container that can
be hidden by view state, or the view must switch automatically when such a surface
is raised. A safety gate that cannot be seen or heard is equivalent to no gate.

### 10. Tailwind conditional overrides must be self-contained

Four visual regressions were introduced by the migration in `af89a22`, all from
one root cause: reasoning about Tailwind as if class order within the `class`
attribute determines precedence. It does not — when two utilities set the same
property, the rule Tailwind emits later wins, regardless of attribute order.

- `border-[var(--card-border)]` and `border-[var(--inner-card-border)]` compile to
  `border-color: var(--card-border)`, but those variables hold the **shorthand**
  `1px solid rgba(...)`. `border-color: 1px solid rgba(...)` is invalid at
  computed-value time and resets to `currentColor`, so every top-level card draws a
  near-black/near-white hairline instead of a 16 %-alpha one. Worse, the audio
  control cards specify the colour utility with no width utility, and preflight
  sets `*{border:0 solid}`, so those cards render **no border at all**. The correct
  form, `[border:var(--inner-card-border)]`, is already used correctly elsewhere in
  the same file.
- The browser-visibility toggles set `bg-` / `text-` / `border-` in both the base
  string and the pressed-state conditional; the base is emitted later and wins, so
  the active mode is visually indistinguishable. (`aria-pressed` remains correct,
  so this affects sighted and low-vision users only.)
- Read-only setting cards append a `color-mix` background to a constant that
  already sets a background; the base wins, so ~20 cards look editable while
  remaining `pointer-events-none`.
- The privacy/consent responsive breakpoint moved from 768 px to 640 px
  (`max-sm:` where `max-md:` was the faithful mapping).

Required behavior: any class constant representing a mutually exclusive visual
state must be a complete, self-contained string, not base-plus-delta — the rule the
migration documented in `shared-controls.tsx` and then violated. CSS variables
holding shorthand values must be applied with the arbitrary-property form
`[border:var(--x)]`, never a `border-*` colour utility.

All four passed `pnpm lint`, `pnpm build`, and 238/238 tests, because nothing in
the suite asserts computed style. A regression test that asserts emitted CSS for
at least these conflict cases is part of the required behavior.

### 11. The click-authorization subsystem must be tested

`click_authorization.rs` is 693 lines carrying most of the click-safety
invariants, and `prepare_planner_output_for_execution`,
`preflight_pending_click_authorizations`, `insert_deterministic_click_confirmation_gate`,
and `ClickGroundingAuthorized` have zero references outside their own definitions.
The file's only tests are two unit tests on fingerprinting and keyword matching.

This is the enabling debt behind constraint 1 (the missing click label survived
because `confirmation_manifest.rs`'s test module has no `ClickElement` case) and
is the single highest-risk coverage gap in the repository.

Required behavior: the security-relevant behaviors of this module are covered by
tests — at minimum: an unminted token is rejected; planner-supplied
`_runtime_click_*` values are stripped and re-derived; a `Ready` all-click plan
receives the inserted confirmation gate; a token is single-use across two click
steps; and expiry and page-generation staleness are enforced.

### 12. CI lists that can silently under-run must be guarded

Three places hardcode a list that CI depends on, with no check that the list is
complete:

- `scripts/run-rust-tests-linux.sh` hardcodes the names of the 7 isolated
  security tests. These are **all** the fail-closed consent/privacy tests. Adding
  an eighth `#[ignore]`d test means it silently never runs, with a green build.
- `scripts/check-remote-planner-privacy-state.py` hardcodes `REQUIRED_PATHS`,
  which has already needed manual repair twice during file splits.
- The `test:ui` glob had the same failure mode (silently running 15 of 238 tests)
  and was fixed in `d9dc6c7`.

Required behavior: the isolated-test list is asserted equal to the actual set of
ignored tests (`cargo test -- --ignored --list`), so the two cannot diverge
silently.

---

## Expected files touched

Rust:

- `src-tauri/src/app_core/click_authorization.rs` (constraints 1, 11)
- `src-tauri/src/narration.rs`, `src-tauri/src/app_core/reading_tools.rs` (2)
- `src-tauri/src/config/persistence.rs`, `src-tauri/src/command_handlers/safety_handlers.rs` (3)
- `src-tauri/src/app_core/element_scoring.rs`, `src-tauri/src/app_core/form_fill/field_fill.rs`, `field_focus.rs`, `src-tauri/src/commands/validators/element.rs` (4)
- `src-tauri/src/app_core/remote_data_consent/**`, `src-tauri/src/tts/**`, `src-tauri/src/asr/**`, `src-tauri/src/app_core/narration.rs` (5)
- `src-tauri/src/tts/mod.rs` (6)
- `src-tauri/src/asr/mod.rs`, `src-tauri/src/asr/capture.rs` (7)
- `src-tauri/src/app_core/replanning_orchestrator.rs`, `src-tauri/src/app_core/listening_tools.rs`, `src-tauri/src/app_core/narration.rs` (8)
- `src-tauri/src/app_core/planner_redaction/sensitive.rs`, `relevance.rs` (P2)
- `src-tauri/src/config/persistence.rs` (file mode, P2)
- `src-tauri/src/asr/local.rs` (whisper context cache, P2)

Frontend:

- `src/panel-state-setters.ts`, `src/app-shell.tsx`, `src/confirmation-panel.tsx` (9)
- `src/settings-panels/shared-controls.tsx`, `src/settings-panels/playback.tsx`, `src/settings-panels/workspace.tsx`, `src/app-shell-nav.tsx`, `src/confirmation-panels/push-to-talk.tsx`, `src/confirmation-panels/confirmation.tsx` (10)

Scripts / docs:

- `scripts/run-rust-tests-linux.sh` (12)
- `docs/SPECS.md`, `config.example.toml` (P3)

## Acceptance summary

```bash
# 1. Click confirmations name the element, not an internal id.
rg -n "RUNTIME_TARGET_LABEL_ARG" src-tauri/src/app_core/click_authorization.rs
# Expected: also referenced inside annotate_click_step, not only annotate_target_step.

# 2. No unchecked region indexing remains on the narration path.
rg -n "regions\[" src-tauri/src/app_core/reading_tools.rs
# Expected: no direct indexing; bounds-checked access only.

# 3. Safety and OCR persisters validate before writing.
rg -n "validate_safety_settings|validate_ocr_settings" src-tauri/src/config/persistence.rs
# Expected: both called before write_config_atomic.

# 4. Ambiguity margin is computed before truncation.
rg -n "truncate|get\(1\)" src-tauri/src/app_core/element_scoring.rs
rg -n "candidates.len\(\) == 1" src-tauri/src/app_core/form_fill/
# Expected: no confirmation-skipping single-candidate special case remains.

# 5. Remote speech paths consult the shared consent policy.
rg -n "network_mode|origin_rules|high_risk|consent" src-tauri/src/tts src-tauri/src/asr
# Expected: non-empty; remote paths evaluate policy before dispatch.

# 6. Empty synthesized audio is rejected.
rg -n "samples.is_empty" src-tauri/src/tts/mod.rs
# Expected: a typed error before caching.

# 7. Hands-free capture resets between windows and the buffer is capped.
rg -n "mem::take|drain|MAX_CAPTURE" src-tauri/src/asr/capture.rs src-tauri/src/asr/mod.rs

# 8. No blocking work under the AppCore lock on these paths.
# Verified by reading replanning_orchestrator.rs and listening_tools.rs.

# 9. Settings navigation validates its target.
rg -n "as SettingsView" src/
# Expected: no unchecked cast remains.

# 10. Cascade regression test exists and passes.
pnpm test:ui

# 11. Click-authorization behaviors are covered.
rg -n "prepare_planner_output_for_execution|insert_deterministic_click_confirmation_gate" src-tauri/src
# Expected: referenced from tests, not only their definitions.

# 12. The isolated-test list cannot silently diverge.
bash scripts/run-rust-tests-linux.sh
# Expected: fails if the hardcoded list != `cargo test -- --ignored --list`.
```

Full validation gate (must pass before this pass is considered complete):

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
