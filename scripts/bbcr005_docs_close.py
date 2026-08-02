from pathlib import Path

TODO = Path("docs/BLIND_BROWSER_COMPREHENSIVE_CODE_REVIEW_FIX_TODO_2026-08-01.md")
REPORT = Path("docs/BLIND_BROWSER_COMPREHENSIVE_CODE_REVIEW_FIX_IMPLEMENTATION_REPORT_2026-08-01.md")


def mark(text: str, item: str) -> str:
    old = f"- [ ] {item}"
    new = f"- [x] {item}"
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"expected exactly one unchecked item: {item!r}; found {count}")
    return text.replace(old, new, 1)


todo = TODO.read_text()

# BBCR-001: mark only implementation and regression claims demonstrated by the
# merged Batch 1 policy work and the exact-head Batch 5 worker. Keep the broad
# direct-command-entry-point requirement open.
bbcr001_items = [
    "Inventory every `ToolName` and classify it in one centralized policy table.",
    "Classify tools as read-only, reversible local state change, browser navigation, page interaction, data entry, form submission, arbitrary script execution, credential operation, model download, or other side effect.",
    "Define the minimum confirmation requirement for each class.",
    "Make the classification exhaustive so adding a new `ToolName` causes a compile failure until its risk policy is specified.",
    "Introduce a deterministic action-policy type, such as `ActionRisk`, `ConfirmationRequirement`, or an equivalent strongly typed representation.",
    "Include at least `NoConfirmation`, `ConfirmationRequired`, and `Prohibited` outcomes.",
    "Include a machine-readable reason code.",
    "Include the specific step IDs and normalized actions that caused the requirement.",
    "Change planner-output validation to inspect the actual step list.",
    "Require confirmation for every plan containing `SubmitActiveForm`, regardless of `intent.name`.",
    "Reject any `Ready` plan containing an action that deterministic policy says requires confirmation.",
    "Reject any `Complete` or `Blocked` output that contains executable steps.",
    "Reject inconsistent planner metadata rather than trusting or repairing it silently.",
    "Pass current safety settings into deterministic validation.",
    "Apply `always_confirm_submit` in deterministic code.",
    "Apply `allow_click_without_confirmation` in deterministic code.",
    "Apply `confirmation_confidence_threshold` using deterministic grounding evidence rather than a planner assertion.",
    "Define and test behavior when confidence is unavailable: fail closed or require confirmation.",
    "Design a deterministic click-safety contract.",
    "Ensure `ClickElement` can be authorized only against a current, validated element resolution.",
    "Carry a deterministic grounding record or opaque authorization token from `FindElement`/resolution to `ClickElement`, rather than accepting an unverified element ID alone.",
    "Bind the grounding record to page identity, element identity, locator, confidence, visibility, enabled state, and a bounded age/version.",
    "Require confirmation when configured, when confidence is below threshold, when confidence is missing, or when the target is ambiguous or potentially destructive.",
    "Reject stale grounding after navigation, DOM replacement, page identity change, or relevant runtime-state change.",
    "Treat `EvalJs` as a high-risk capability.",
    "Decide whether planner-generated arbitrary JavaScript should be prohibited entirely.",
    "Prevent planner text from directly becoming unrestricted JavaScript without a deterministic policy decision.",
    "Add executor-level defense in depth.",
    "Recompute or verify the required confirmation immediately before executing each side effect.",
    "Do not rely only on pre-execution planner validation.",
    "Return a stable fail-closed error code when an unconfirmed side effect reaches dispatch.",
    "Validate plan/action consistency.",
    "Verify that `intent.name` is compatible with the actual tools, but never use the intent to weaken policy.",
    "Reject plans that disguise submit, data entry, scripting, or destructive clicks under unrelated intents.",
    "Reject planner-provided `requires_confirmation = false` when deterministic policy requires it.",
    "Verify safety after replanning.",
    "Apply the same policy to every replanned output.",
    "Prevent a failed confirmed plan from replanning into an unconfirmed equivalent side effect.",
    "Ensure accumulated tool history cannot be used to bypass confirmation.",
    "A `Ready` plan with `intent.name = ReadPage` and a `SubmitActiveForm` step is rejected.",
    "A `Ready` plan with any tool classified as confirmation-required is rejected.",
    "A plan cannot bypass submit confirmation by placing submit after a non-submit step.",
    "A plan cannot bypass confirmation through `on_failure`, `NextStep`, `Replan`, or a cycle.",
    "A replanned output containing a protected side effect still requires confirmation.",
    "`allow_click_without_confirmation = false` forces confirmation for ordinary clicks.",
    "`allow_click_without_confirmation = true` does not bypass confirmation for ambiguous, low-confidence, stale, or risky clicks.",
    "Missing click confidence fails closed.",
    "A click authorization becomes invalid after navigation or page-model replacement.",
    "`EvalJs` follows the selected prohibit-or-confirm policy.",
    "Executor defense-in-depth rejects a side effect even if validation is accidentally skipped in a test harness.",
    "Existing intended safe, read-only plans continue to execute without unnecessary confirmation.",
    "No planner-controlled field can reduce a deterministically calculated confirmation requirement.",
    "Submit confirmation is enforced by actual tool presence.",
    "Click confirmation follows current settings and deterministic grounding evidence.",
]
for item in bbcr001_items:
    todo = mark(todo, item)

# BBCR-002: the manifest, digest, state binding, replay resistance, and click
# live-revalidation are proven. The broader all-tool locator-resolution item and
# richer submit/data-entry summaries remain open.
bbcr002_items = [
    "Stop treating planner-provided `prompt_text` as authoritative confirmation copy.",
    "Planner text may be retained only as untrusted explanatory context.",
    "Generate the primary confirmation message in deterministic Rust code.",
    "Define a normalized confirmation manifest.",
    "Include request ID, page ID, current origin, action sequence, tool names, normalized arguments, target descriptions, and relevant safety reasons.",
    "Redact sensitive values while preserving enough information for meaningful approval.",
    "Represent typed text by category and safe summary; do not speak or display passwords or full secrets.",
    "Compute a stable digest or equivalent immutable identifier over the normalized manifest.",
    "Store the digest in pending execution state.",
    "Return it with the confirmation challenge.",
    "Require the same digest when applying the confirmation response.",
    "Revalidate immediately before resume.",
    "Confirm the current page identity and origin still match.",
    "Confirm queued steps and arguments still hash to the approved manifest.",
    "Abort and replan if any relevant state changed.",
    "Click: target role/name and whether navigation or another consequential action is expected.",
    "Ensure confirmation cannot be reused.",
    "Mark confirmation IDs/digests consumed after one response.",
    "Reject replay, duplicate submission, mismatched ID, mismatched digest, and expired confirmation.",
    "Add a bounded expiration time.",
    "Planner-supplied misleading prompt text cannot replace the deterministic summary.",
    "Changing any queued tool or argument after prompt generation invalidates approval.",
    "Reordering queued actions invalidates approval.",
    "Navigation or origin change invalidates approval.",
    "DOM/page identity change invalidates stale approval.",
    "A confirmation response cannot be replayed.",
    "Timeout and rejection clear all pending protected actions.",
    "Sensitive field values are redacted in confirmation text and serialized pending state.",
    "The user approves exactly the actions that execute.",
    "Confirmation state is immutable, expiring, single-use, and state-bound.",
]
for item in bbcr002_items:
    todo = mark(todo, item)

start = todo.index("## BBCR-015 — Revalidate Plans Against the State Snapshot Used for Planning")
end = todo.index("\n---\n\n# P2 — CI, Dependency, and Secret-Handling Defense in Depth", start)
bbcr015 = """## BBCR-015 — Revalidate Plans Against the State Snapshot Used for Planning

### Problem

The application intentionally releases the `AppCore` lock during remote planning. Side-effecting plans must therefore be rejected and boundedly replanned whenever the authoritative runtime state differs from the snapshot used for planning or confirmation.

### Tasks

- [x] Add an opaque runtime state token to `PlannerInput` while retaining the authoritative snapshot server-side.
  - [x] Bind page ID, page/document generation, normalized origin, browser-history position, deterministic safety settings, relevant configuration, and pending-confirmation identity.
- [x] Bind the server-side snapshot to the exact serialized planner-output digest; planner output cannot replace or weaken it.
- [x] Revalidate before execution.
  - [x] Reject side-effecting plans when relevant state changed.
  - [x] Permit status-only/read-only operations without unnecessary snapshot failure.
  - [x] Trigger bounded replanning when stale.
- [x] Revalidate again after confirmation using the immutable BBCR-002 manifest and runtime-state binding.
- [x] Serialize `AppCore` mutations while safely detecting relevant commands interleaved during remote planning.
- [x] Document the state/tool invalidation matrix in `docs/BBCR-005_RUNTIME_STATE_BINDING_2026-08-01.md`.

### Required regression tests

- [x] Navigation during planner request invalidates a click/submit plan.
- [x] Page-model refresh invalidates stale element references.
- [x] Safety-setting changes invalidate a plan resolved under weaker settings.
- [x] Unrelated read-only state changes do not cause unnecessary failure.
- [x] A real `AppCore` confirmation executes once and rejects replay.
- [x] A real `AppCore` confirmation aborts when relevant runtime state changes after approval.
- [x] A protected click hidden in a cyclic plan is rejected before execution.
- [x] Legacy serialized `AppState` without `page_generation` remains readable.

### Acceptance criteria

- [x] Side effects execute only against the state they were validated and approved for.
- [x] Stale planning and confirmation state fails closed with stable error/replan outcomes.

### Validation evidence

- **Validated source commit:** `1a6c2b213777766d9e1de056127cafcf0ca45bfa`
- **Exact worker trigger head:** `e035ab3757911853ee7f015b35dd13dc5df795a0`
- **Bounded closure run:** `30734369368`
- **Bounded closure job:** `91460289871`
- **Worker result:** success across transformation, silent-fallback scan, formatting, default compilation, all-target/all-feature Clippy with warnings denied, the complete Xvfb-backed Rust suite, frontend lint, UI tests, production build, and whitespace verification.
- **Permanent CI requirement:** the final documentation/evidence commit must receive a successful `ci/permanent` status before Batch 5 is declared complete.
"""
todo = todo[:start] + bbcr015 + todo[end:]
TODO.write_text(todo)

report = REPORT.read_text()
report = report.replace(
    "This Ralph Loop has implemented and validated four bounded security batches. The comprehensive TODO remains open because deterministic click-grounding authorization, the remaining confirmation-grounding dependencies, distinct remote-only planner payloads and consent controls, opaque image handles, model-download integrity, the remaining hostile-content corpus and telemetry work, and the P1/P2/P3 program are not yet complete.",
    "This Ralph Loop has implemented and validated five bounded security batches. Batch 5 completes runtime-bound click authorization, stale-planning detection, bounded replanning, and post-confirmation runtime revalidation. The comprehensive TODO remains open because a complete audit of every direct side-effect command entry point, broader non-click locator re-resolution, richer form/data-entry confirmation summaries, distinct remote-only planner payloads and consent controls, opaque image handles, model-download integrity, the remaining hostile-content corpus and telemetry work, and the P1/P2/P3 program are not yet complete.",
    1,
)

batch5 = """
## Batch 5 — Runtime-bound planning, click authorization, and confirmation replay defense

**Validated source commit:** `1a6c2b213777766d9e1de056127cafcf0ca45bfa`  
**Exact worker trigger head:** `e035ab3757911853ee7f015b35dd13dc5df795a0`  
**Bounded closure run:** `30734369368`  
**Bounded closure job:** `91460289871`  
**Result:** source implementation and bounded validation complete; final exact-SHA permanent CI remains the Batch 5 declaration gate

Implemented:

- Added an opaque `runtime_state_token` to planner input while preserving the authoritative `PlanningStateSnapshot` exclusively in Rust.
- Bound snapshots to page ID, page/document generation, normalized origin, browser-history position, deterministic safety settings, relevant configuration, pending-confirmation identity, and the exact serialized planner-output digest.
- Converted stale side-effect snapshots into bounded `NeedsReplan` outcomes while allowing semantically independent read-only status operations.
- Added runtime-owned click authorizations bound to page identity/generation, origin, element ID, locator, live element fingerprint, confidence, ambiguity, destructive classification, issue time, and expiry.
- Re-extract and re-resolve click targets against the live DOM immediately before dispatch.
- Invalidated click authorizations and pending confirmations after navigation, page-model replacement, generation changes, relevant configuration changes, expiry, or live target drift.
- Added a final post-confirmation runtime-state comparison after live click revalidation and before resuming protected execution.
- Kept pending runtime tokens server-only through Serde and preserved legacy `AppState` deserialization with a default page generation.
- Rejected cycles across both success and failure transitions before protected steps can hide inside a graph loop.
- Added real `AppCore` replay and stale-runtime confirmation regressions and ran them under Xvfb so Wry/GTK initializes on headless Linux CI.
- Documented the runtime invalidation matrix in `docs/BBCR-005_RUNTIME_STATE_BINDING_2026-08-01.md`.
- Removed the one-shot transformation workflow, trigger, and scripts from the validated source commit.

Validation evidence:

- Run `30734369368`, job `91460289871`, passed exact-head checkout, deterministic transformations, dependency installation, Rust formatting, silent-fallback scanning, default Rust compilation, all-target/all-feature Clippy with warnings denied, the complete Xvfb-backed Rust test suite, frontend lint, UI tests, production frontend build, formatting/whitespace verification, cleanup, and source commit/push.
- The final evidence commit is not considered Batch 5 complete until the permanent repository workflow publishes a successful `ci/permanent` status on that exact SHA.

Remaining boundary after Batch 5:

- Audit every direct non-planner side-effect command entry point against the centralized action policy.
- Generalize immediate locator re-resolution beyond click actions where future protected tools carry element/form grounding.
- Add richer deterministic form identity, destination, and safe field-name summaries when form-grounding metadata is available.

"""
marker = "## Validation gate\n"
if report.count(marker) != 1:
    raise SystemExit("validation-gate marker not found exactly once")
report = report.replace(marker, batch5 + marker, 1)
report = report.replace(
    "- **BBCR-001:** Partially implemented; core actual-tool policy and executor guard complete, deterministic click authorization still open.\n- **BBCR-002:** Core immutable confirmation-manifest implementation merged and fully validated; DOM-generation and element revalidation dependencies remain open.\n- **BBCR-003:** Partially implemented; strong extraction and serialization redaction exists, but distinct remote-only types and consent policy remain open.\n- **BBCR-004:** Complete, validated, merged to `master`, and branch cleanup complete.\n- **BBCR-005:** Open.",
    "- **BBCR-001:** Partially implemented; core actual-tool policy, executor guard, and runtime-owned live-DOM click authorization are complete. A complete audit of direct non-planner side-effect entry points remains open.\n- **BBCR-002:** Core immutable confirmation manifests, generation-qualified page binding, click live-revalidation, expiry, and real `AppCore` replay defense are complete. Broader non-click locator re-resolution and richer form/data-entry summaries remain open.\n- **BBCR-003:** Partially implemented; strong extraction and serialization redaction exists, but distinct remote-only types and consent policy remain open.\n- **BBCR-004:** Complete, validated, merged to `master`, and branch cleanup complete.\n- **BBCR-005 / BBCR-015 Batch 5:** Source implementation and bounded validation complete; final declaration depends on successful `ci/permanent` status for the exact final evidence SHA.",
    1,
)
report = report.replace(
    "No release-readiness, comprehensive-TODO completion, BBCR-004 merge-readiness, or full security-signoff claim is made by this report.",
    "No release-readiness, comprehensive-TODO completion, or full security-signoff claim is made by this report.",
    1,
)
REPORT.write_text(report)

print("Applied conservative BBCR-001/002 checklist evidence and completed BBCR-015 documentation")
