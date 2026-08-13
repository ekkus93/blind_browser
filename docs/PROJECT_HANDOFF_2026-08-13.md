# Blind Browser Project Handoff — 2026-08-13

## Purpose

This file is the restart point for the next development session. It records the
current repository state, the major implementation and hardening work that has
already landed, the safety/architecture invariants that must be preserved, the
small amount of v1 work still open, and the recommended order for resuming.

Generated at approximately 2026-08-13 13:14 PDT / 20:14 UTC.

---

## 1. Executive summary

The project is in **late v1 hardening / acceptance / release-closure**, not active
feature construction.

All Code Review 3 P0/P1/P2/P3 implementation work is effectively complete.
The recent P1.3 and P1.4 follow-ups have also landed:

- OCR crops are now bound to screenshot capture provenance and fail closed when a
  document-space bbox is incompatible with the cached raster.
- Planner execution now revalidates output against the exact planning-time
  `active_skill_names` bound into the consumed planning snapshot.

The final `master` baseline before this handoff document was added is:

`5b538276afd2b606f1bdb67f0f80736bd88dba62`

Commit message:

`docs: reconcile P1.4 execution safety closure`

Permanent hosted CI for that exact SHA is green:

- workflow run: `31738592363`
- validation job: `94576159495`
- `ci/permanent`: `success`
- problem steps: none

That run passed the complete permanent gate: fallback/privacy/security scanners,
Rust formatting, default-feature compilation, strict all-target/all-feature
Clippy, focused direct-command policy evidence, the complete Rust/Wry test sweep,
frontend lint, UI tests, and frontend production build.

There are currently **no open pull requests**. The only open GitHub issue is the
automatically-maintained CI-status issue (`#1`).

There is one disposable recovery branch remaining:

`agent/restore-clean-p14-tree`

It is not needed for production work and is safe to delete. PR #15 associated
with that recovery branch is already closed and was not merged.

---

## 2. Project identity and architecture

`blind_browser` is a voice-first desktop browser for vision-impaired users,
built with Rust/Tauri and a React + Redux frontend.

### Repository layout

- `src-tauri/` — Rust/Tauri runtime, browser control, planner/tool execution,
  ASR/TTS/OCR, confirmation/consent, config, privacy policy, and runtime state.
- `src/` — React 19 + Redux frontend, settings, confirmation/consent surfaces,
  voice controls, navigation and Tauri API adapters.
- `docs/` — specifications, review/hardening trackers, architecture notes and
  implementation history.
- `scripts/` — permanent CI guardrails for silent fallbacks, security fallbacks,
  sensitive diagnostics, remote-planner privacy state and Rust/Wry test coverage.

### Frontend ownership model

The frontend React migration is complete:

- a single top-level React app owns the shell;
- Redux owns shell view, settings subpages, panel state and confirmation state;
- live panels render React nodes rather than replacing HTML strings;
- Tauri wrappers live under `src/api/` and presentational components do not call
  backend commands directly;
- the old `innerHTML`-driven rendering seam has been retired from the runtime
  path.

### Backend execution model

The Rust runtime is built around deterministic tool execution and explicit
runtime state:

- browser/page extraction produces a bounded page model;
- local/remote planners produce typed planner output;
- planner output is structurally validated and policy validated before use;
- side-effecting work is guarded by deterministic confirmation/click
  authorization rules;
- planner execution is snapshot-bound and digest-bound;
- remote planner/TTS/ASR disclosures are gated by the shared consent/privacy
  layer;
- capture/network work that can block is executed without holding the global
  `AppCore` mutex across the blocking window.

---

## 3. Safety and correctness invariants that must not regress

Future work should treat the following as load-bearing constraints.

### 3.1 Fail closed; do not introduce silent fallbacks

The repository has permanent scanners specifically to prevent silent or unsafe
fallback behavior. Do not weaken them merely to make CI green.

If a provider, authorization, snapshot, image provenance, config value, secret,
or planner/tool contract is invalid, the runtime should return a typed error or
bounded replan instead of silently substituting a different behavior.

Relevant permanent checks:

```bash
bash scripts/check-silent-fallbacks.sh
python3 scripts/check-security-fallbacks.py --self-test
python3 scripts/check-security-fallbacks.py
python3 scripts/check-security-fallback-inventory.py --self-test
python3 scripts/check-security-fallback-inventory.py
python3 scripts/check-sensitive-diagnostics.py --self-test
python3 scripts/check-sensitive-diagnostics.py
python3 scripts/check-remote-planner-privacy-state.py --self-test
python3 scripts/check-remote-planner-privacy-state.py
```

### 3.2 Remote speech and planner data require policy authorization

Remote planner, remote narration and remote microphone transcription all go
through the shared remote-data policy. Local TTS/ASR remain local and ungated.

Remote provider dispatch is type-state guarded; callers must not be able to forge
an authorized remote request from the public command surface.

For remote ASR, policy evaluation happens before new microphone capture begins.
Consent-required state stores metadata only, not captured audio. An interactive
microphone approval requires the user to repeat the utterance; pre-consent audio
is not retained and replayed.

### 3.3 Planner execution provenance is mandatory

The original CR3 P1.4 bug allowed some directly-invoked planner outputs to bypass
snapshot provenance. That is closed.

`ExtractPageModel`, `ReportResult` and `TranscribeCommand` now require the same
snapshot provenance as other planner-executed work where applicable.

The later defense-in-depth follow-up is also complete:

- `PlanningStateSnapshot` carries planning-time `active_skill_names`;
- `register_planning_snapshot` binds those skill names to the exact planner-output
  digest;
- direct, remote and remote-consent-resume paths preserve the validated skill set;
- `validate_and_consume_planning_snapshot` reruns
  `validate_planner_output_with_safety` with the snapshot-bound skills and safety
  settings before execution;
- unbound read-only output receives no fabricated skill capability.

Implementation commit:

`7b61a72b675086a737ad80560c29af15b088cddb` —
`fix: revalidate planner output against bound skills`

Exact clean-tree validation SHA for this change:

`66d975aa9d866b6a6d1bd9952a4e1e1e8fd5e2af`

Permanent validation evidence:

- run `31730184347`
- job `94548430293`
- success

### 3.4 OCR bboxes are document-space; cached screenshots carry raster provenance

The original P1.3 bug came from mixing `getBoundingClientRect()` viewport-relative
coordinates with document/page-absolute screenshot/OCR consumers. The original
fix converts extracted bboxes to document-absolute coordinates.

A later residual edge case was then closed: an arbitrary cached viewport screenshot
could otherwise be OCR'd using a document-space `region_id` with no knowledge of
the raster's capture-time scroll origin.

Current behavior:

- screenshot provenance records capture kind and document-space raster origin;
- viewport screenshots bind capture-time scroll origin;
- full-page screenshots use document origin `(0, 0)`;
- clipped region screenshots use the clip's document-space origin;
- image-cache records retain provenance plus raster width/height;
- OCR keeps external/source bboxes in document coordinates but translates the
  bbox passed to Tesseract into image-relative coordinates;
- invalid provenance returns `invalid_screenshot_provenance`;
- a requested bbox not fully represented by the cached raster returns
  `ocr_bbox_outside_screenshot` instead of OCR'ing the wrong pixels.

Implementation commit:

`cd78bf5d9eb2c066d130966310c4a813c49a5388` —
`fix: bind OCR crops to screenshot provenance`

Important: `docs/BB_CODE_REVIEW3_TODO.md` still contains an older paragraph under
P1.3 saying this residual gap was "left out of scope." That paragraph is now
**stale documentation**; the source fix above is already on `master` and the
current complete CI suite is green.

### 3.5 Do not hold the global runtime lock across blocking speech/network windows

CR3 P1.2 closed the old executor-wide-lock blocker without creating a second
planner state machine.

Planner execution uses the existing per-step runner seam through
`LockScopedStepRunner`:

- planner-driven capture sleeps without the `AppCore` guard held;
- ASR transcription runs unlocked;
- remote TTS request/response processing runs unlocked;
- the runtime is re-acquired only to validate/commit state;
- relevant state drift requests bounded replanning;
- listening `true -> false` is the explicitly permitted interleaving so
  `stop_listening` can interrupt an unlocked capture window;
- stopped capture drains as `Ok(None)` rather than using stale audio.

Validated implementation SHA:

`5f309e360a283d7043e71403fa616e6c9f6d22fb`

Permanent validation:

- run `31673285045`
- job `94362303907`
- success

### 3.6 Confirmation and click authorization remain deterministic

Do not let planner-supplied metadata bypass confirmation policy.

Click authorization tokens are runtime-derived, context-bound, single-use and
checked against current page/element identity. Runtime annotations are stripped
and re-derived. Dangerous/ambiguous actions use deterministic confirmation paths.
Submit remains confirmation-required in v1.

---

## 4. Major completed work

This section is intentionally redundant with the individual TODO files so a new
session can orient quickly.

### Code Review 2 / async runtime

Completed:

- capture buffer drain semantics;
- async command bridge using managed `Arc<Mutex<AppCore>>` + `spawn_blocking`;
- lock scoping around capture;
- lock scoping around remote planner and remote ASR calls;
- defensive panic removal at reviewed production sites;
- secret-memory/config/formatting hardening;
- runtime Phase 3 network lock scoping;
- later CR3 P1.2 planner-embedded speech lock scoping.

`docs/BB_CODE_REVIEW2_TODO.md` records P1.1.2, P1.1.3 and P1.1.4 as DONE.

### Code Review 3 P0

All P0 items are complete, including:

- useful click confirmation labels;
- narration cursor bounds;
- settings validation before disk writes;
- element-resolution confirmation bypass fixes;
- valid settings navigation targets;
- confirmation/consent surfaces reachable from any app view;
- empty synthesized-audio rejection;
- hands-free capture-buffer reset/cap;
- Tailwind migration/cascade regressions and regression coverage.

### Code Review 3 P1

Completed:

- shared remote planner/TTS/ASR privacy and consent boundary;
- lock-scoped planner speech I/O;
- document-absolute region bbox handling;
- screenshot-coordinate provenance for cached OCR images;
- direct planner-output provenance binding;
- execution-time safety revalidation with snapshot-bound `active_skill_names`.

### Code Review 3 P2/P3

Completed hardening includes:

- click-authorization subsystem evidence;
- guarded isolated-Wry test inventory;
- restricted config permissions and parent-directory durability sync;
- Unicode-aware sensitive-content detection;
- corrected sanitization metadata accounting;
- persistent-consent snapshot recapture fix;
- cached Whisper context;
- validation/config/frontend duplication reduction;
- `docs/SPECS.md` validation/schema drift corrections;
- removal of the inert `high_risk_origin_policy` knob;
- visual/accessibility fixes for headings, CTA contrast and empty error chrome.

### Frontend modernization

`docs/TODO.md` Priorities 1–6 and their validation checklist are complete.
The React/Redux ownership migration and old HTML rendering seam removal are done.

---

## 5. Current repository state

### `master`

Clean baseline before this handoff document:

`5b538276afd2b606f1bdb67f0f80736bd88dba62`

Final CI is green on that exact SHA.

A compare from the clean P1.4 source-validation SHA
`66d975aa9d866b6a6d1bd9952a4e1e1e8fd5e2af` to `5b538276...` shows the only net
file difference is the CR3 tracker documentation. No temporary P1.4 workflow or
recovery source survived in the final production tree.

### Permanent CI workflow

`.github/workflows/ci.yml` is restored to its canonical read-only contents
permission:

```yaml
permissions:
  contents: read
  statuses: write
```

Canonical workflow blob at the handoff baseline:

`5f42726e4c81d6f7e8ecc8ddb47426021b05426c`

### Branches

At handoff time the repository has only:

- `master`
- `agent/restore-clean-p14-tree`

The recovery branch is obsolete and safe to delete. It was created solely while
recovering from a temporary workflow-edit attempt. PR #15 is closed and unmerged.

Earlier P1.2/P1.3/remote-speech development branches were audited and cleaned up;
required production content was already on `master` or superseded by later
validated implementations.

### Pull requests / issues

- Open PRs: none.
- Open issue #1: automated hosted-quality-gate status publisher; not engineering
  backlog.

---

## 6. What still needs to be done

There is no known open CR3 P0/P1 implementation defect. The remaining v1 work is
primarily **live acceptance, documentation reconciliation and release closure**.

### 6.1 Highest priority: live `--features full` behavioral acceptance

`docs/BB_ASYNC_RUNTIME_TODO.md` still has two genuinely unchecked behavioral
acceptance items:

1. **Webview responsiveness**
   - Run the full native app on a real page.
   - Verify the webview stays responsive during:
     - a ~10 second microphone capture;
     - a deliberately slow planner call;
     - navigation/browser work.

2. **Voice → browser worker-thread safety**
   - With the full browser backend active, issue voice commands that exercise
     navigation/click/read paths.
   - Confirm they complete without the old worker-thread / nested `block_on`
     panic.

Useful additional live checks, even though code/regression evidence has already
allowed their checklist items to be marked done:

- call `stop_listening` during an active capture and confirm the capture ends
  promptly rather than waiting for the full window;
- call `get_agent_state` during active capture and during a slow remote planner
  request and confirm it returns promptly;
- run at least one remote TTS and remote ASR consent path on a real machine to
  verify the user-facing behavior matches the typed policy implementation.

Do not fabricate these as completed if they have not been run on a real app.

### 6.2 Documentation reconciliation

A focused docs-only cleanup remains worthwhile after the live checks.

#### A. Reconcile the stale P1.3 residual-gap paragraph

`docs/BB_CODE_REVIEW3_TODO.md` still says the screenshot-origin/image-cache OCR
edge case was left out of scope. That is stale.

Update it to record:

- source implementation `cd78bf5d9eb2c066d130966310c4a813c49a5388`;
- viewport/full-page/clip provenance behavior;
- `invalid_screenshot_provenance` and `ocr_bbox_outside_screenshot` fail-closed
  behavior;
- regression coverage;
- current permanent CI evidence.

#### B. Reconcile one stale async-runtime checklist sentence

`docs/BB_ASYNC_RUNTIME_TODO.md`'s final checklist still contains this historical
wording:

> `BB_CODE_REVIEW2_TODO.md` P1.1.2 / P1.1.4 reconciled (P1.1.3 still BLOCKED).

That sentence is stale. `docs/BB_CODE_REVIEW2_TODO.md` now correctly records
P1.1.3 as DONE for both remote planner and remote ASR network paths.

#### C. Add current closure entries to `memory.md`

`memory.md` contains the P1.2 closure but does not yet have concise entries for:

- P1.3 screenshot-coordinate provenance closure;
- P1.4 snapshot-bound `active_skill_names` execution revalidation;
- final CR3 docs reconciliation / live acceptance once completed.

Use real UTC timestamps when adding those entries.

#### D. Audit the bottom of `docs/TODO.md`

The main TODO's actual React/frontend work is complete, but its final `Deliverables`
section still has unchecked generic lines:

- Working desktop app
- README.md
- SPECS.md
- TODO.md
- Example configs

README/SPECS/TODO/example config artifacts already exist, so these lines are at
least partly stale bookkeeping rather than missing engineering work. Audit them
against the actual repository/live acceptance rather than blindly implementing
new work.

### 6.3 Delete the obsolete recovery branch

Safe to delete:

`agent/restore-clean-p14-tree`

No production content needs to be rescued from it.

### 6.4 Final v1 release/completion audit

After live acceptance and docs reconciliation, do one final completion pass:

- inspect every remaining unchecked v1 checkbox across active TODO files;
- distinguish stale documentation from actual missing behavior;
- verify no temporary CI/helper files or development branches remain;
- run the full permanent gate once on the exact release candidate;
- optionally run manual dark-mode coverage if CSS/theme work changed;
- decide whether a packaged desktop artifact/release is required for the user's
  definition of "Working desktop app";
- mark only genuinely verified items complete.

---

## 7. Intentionally incomplete / not automatically next

### Automatic provider failover

README currently states that automatic provider failover is configured in schema
but disabled in the live runtime. The forward-staged failover settings panel was
also intentionally retained during CR3 dead-code cleanup because the backend
feature has not shipped.

Do **not** treat this as an accidental missing implementation without first
confirming whether the user wants provider failover in v1 or a later version.

### Phase 9 v2 work — DO NOT IMPLEMENT unless policy changes

`docs/TODO.md` explicitly marks the following as v2 notes:

- evaluate TensorFlow Lite `micro_speech` wake word;
- candidate extraction system for an LLM action resolver;
- LLM ranking;
- confidence gating;
- broader open-ended UI grounding beyond the deterministic v1 tool layer;
- evaluate whether advanced UI action grounding should remain v2-only.

These are intentionally deferred. Do not begin them merely because their
checkboxes are unchecked.

---

## 8. Recommended resume order

When development resumes, use this order unless the user explicitly changes
priority:

1. **Verify current `master`**
   - fetch latest head;
   - ensure it is this handoff commit or a known descendant;
   - inspect any intervening changes before acting.

2. **Delete `agent/restore-clean-p14-tree`**
   - it is disposable recovery history only.

3. **Run the live full-feature acceptance checks**
   - webview responsiveness during capture/planner/navigation;
   - voice → browser command without worker-thread panic;
   - optionally stop-listening and state-read interleaving while operations are
     active.

4. **Fix only defects actually reproduced by the live checks**
   - keep changes fail-closed;
   - preserve snapshot/consent/click-authorization invariants;
   - do not weaken scanners or add lint suppressions to hide a real issue.

5. **Reconcile stale docs**
   - CR3 P1.3 residual paragraph;
   - async-runtime stale P1.1.3 sentence;
   - `memory.md` P1.3/P1.4/live-acceptance entries;
   - main TODO deliverables bookkeeping.

6. **Run the final v1 completion audit**
   - full CI on the exact final SHA;
   - confirm no remaining active v1 engineering tasks;
   - decide packaging/release requirements.

7. **Only then discuss v2**
   - do not start the Phase 9 notes automatically.

---

## 9. Full validation gate

Permanent CI currently runs the following effective gate. Use this as the local
reference for any future runtime change:

```bash
bash scripts/check-silent-fallbacks.sh
python3 scripts/check-security-fallbacks.py --self-test
python3 scripts/check-security-fallbacks.py
python3 scripts/check-security-fallback-inventory.py --self-test
python3 scripts/check-security-fallback-inventory.py
python3 scripts/check-sensitive-diagnostics.py --self-test
python3 scripts/check-sensitive-diagnostics.py
python3 scripts/check-remote-planner-privacy-state.py --self-test
python3 scripts/check-remote-planner-privacy-state.py

cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features --test post_batch8_direct_command_policy_evidence
bash scripts/run-rust-tests-linux.sh

pnpm lint
pnpm test:ui
pnpm build
```

On Linux, the full native gate requires the Tauri/GTK/WebKit, ALSA, Tesseract and
Leptonica development packages documented in README.

Do not use `--features wry`; there is no such crate feature. The Cargo default
feature set is already `full`.

---

## 10. Important commits and evidence

### Current handoff baseline

`5b538276afd2b606f1bdb67f0f80736bd88dba62`

`docs: reconcile P1.4 execution safety closure`

Permanent CI:

- run `31738592363`
- job `94576159495`
- success

### P1.4 execution-time safety / bound skills

Implementation:

`7b61a72b675086a737ad80560c29af15b088cddb`

`fix: revalidate planner output against bound skills`

Clean source validation tree:

`66d975aa9d866b6a6d1bd9952a4e1e1e8fd5e2af`

Permanent CI:

- run `31730184347`
- job `94548430293`
- success

### P1.3 screenshot-coordinate provenance

Implementation:

`cd78bf5d9eb2c066d130966310c4a813c49a5388`

`fix: bind OCR crops to screenshot provenance`

The earlier development branch was formally merged into master history with:

`0907163793d11ba44f60dc8538da933422d994f4`

`merge: agent/p1-3-ocr-provenance`

### P1.2 lock-scoped planner speech I/O

Validated implementation:

`5f309e360a283d7043e71403fa616e6c9f6d22fb`

Permanent CI:

- run `31673285045`
- job `94362303907`
- success

### P1.1 remote speech consent/privacy closure

Validated implementation:

`15b1f5890b17722ab126c97acdc6a050168a108d`

Permanent CI:

- run `31277834628`
- job `93154218640`
- success

---

## 11. Files to read first in a new session

Read these before making another broad change:

1. `docs/PROJECT_HANDOFF_2026-08-13.md` — this file.
2. `README.md` — current architecture, dev setup and validation commands.
3. `docs/TODO.md` — main frontend/project tracker; note Phase 9 is explicitly
   v2-only.
4. `docs/BB_CODE_REVIEW3_TODO.md` — detailed current safety/hardening history;
   remember the P1.3 residual paragraph is stale and needs reconciliation.
5. `docs/BB_ASYNC_RUNTIME_TODO.md` — live full-feature acceptance still pending;
   note the stale final-checklist P1.1.3 sentence.
6. `docs/BB_CODE_REVIEW2_TODO.md` — authoritative CR2 statuses; P1.1.3 is DONE.
7. `memory.md` — historical session log; latest P1.3/P1.4 closure entries still
   need to be added.

For planner/execution safety changes, also read:

- `src-tauri/src/app_core/planning_snapshot.rs`
- `src-tauri/src/app_core/command_dispatch.rs`
- `src-tauri/src/app_core/replanning_orchestrator.rs`
- `src-tauri/src/app_core/lock_scoped_tools.rs`
- `src-tauri/src/commands/planner_executor/**`

For OCR/screenshot changes, read:

- `src-tauri/src/browser/config.rs`
- `src-tauri/src/browser/page_inspection.rs`
- `src-tauri/src/browser/dom_extraction.rs`
- `src-tauri/src/app_core/image_cache.rs`
- `src-tauri/src/app_core/extraction_tools/ocr_tools.rs`
- `src-tauri/src/app_core/extraction_tools/page_extraction.rs`

For remote speech/privacy changes, read:

- `src-tauri/src/app_core/remote_data_consent/**`
- `src-tauri/src/app_core/narration.rs`
- `src-tauri/src/app_core/voice_tools.rs`
- `src-tauri/src/tts/**`
- `src-tauri/src/asr/**`

---

## 12. Process notes for the next session

- Work from `master` unless there is a concrete reason not to.
- Keep `master` authoritative; do not strand completed code on long-lived agent
  branches.
- If a temporary validation branch/workflow is unavoidable, ensure its production
  source is materialized and merged/transplanted before ending the session, then
  remove the scaffolding.
- Do not poll ordinary CI repeatedly; use local validation where practical and
  inspect CI when there is a concrete failure or when exact closure evidence is
  needed.
- Do not claim manual/live behavior was verified unless it was actually run.
- Prefer typed errors and fail-closed behavior over fallback/substitution.
- Do not add `#[allow(...)]` simply to bypass strict Clippy unless there is an
  independently justified policy decision; refactor first.
- When a TODO conflicts with current code, verify the implementation and treat the
  code plus passing tests as evidence before changing behavior merely to match
  stale prose.

---

## 13. Short restart checklist

If only a quick reminder is needed, resume here:

- [ ] Delete `agent/restore-clean-p14-tree`.
- [ ] Verify current `master` is at/after this handoff and CI is green.
- [ ] Run live `--features full` webview responsiveness acceptance.
- [ ] Run live voice → browser no-worker-thread-panic acceptance.
- [ ] If live checks expose defects, fix them with the full permanent gate.
- [ ] Reconcile stale CR3 P1.3 provenance note.
- [ ] Reconcile stale async-runtime P1.1.3 checklist sentence.
- [ ] Add P1.3/P1.4/live-acceptance entries to `memory.md` with real UTC time.
- [ ] Audit generic unchecked deliverables in `docs/TODO.md`.
- [ ] Perform final v1 release/completion audit.
- [ ] Do **not** start Phase 9/v2 work unless explicitly requested.
