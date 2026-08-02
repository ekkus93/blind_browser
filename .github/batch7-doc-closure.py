from __future__ import annotations

from pathlib import Path

SOURCE_SHA = "fbec02a5b697720c88a3f46054110cd8e7c5c1a6"
RUN_ID = "30746879137"
JOB_ID = "91493868153"

TODO_PATH = Path("docs/BLIND_BROWSER_COMPREHENSIVE_CODE_REVIEW_FIX_TODO_2026-08-01.md")
REPORT_PATH = Path(
    "docs/BLIND_BROWSER_COMPREHENSIVE_CODE_REVIEW_FIX_IMPLEMENTATION_REPORT_2026-08-01.md"
)
EVIDENCE_PATH = Path(
    "docs/BBCR-003_BBCR-006_BATCH7_FINAL_VALIDATION_EVIDENCE_2026-08-02.md"
)
MEMORY_PATH = Path("memory.md")


def replace_between(text: str, start: str, end: str, replacement: str) -> str:
    start_index = text.find(start)
    if start_index < 0:
        raise SystemExit(f"missing start marker: {start}")
    end_index = text.find(end, start_index)
    if end_index < 0:
        raise SystemExit(f"missing end marker: {end}")
    return text[:start_index] + replacement.rstrip() + "\n\n" + text[end_index:]


bbcr003 = '''## BBCR-003 — Add a Strict Page-Data Redaction Boundary Before Remote Planning

### Problem

DOM extraction currently captures live input values and all element attributes, including password, hidden, token-bearing, payment, identity, and private draft data. The resulting page model may be serialized into a remote planner request.

### Required invariant

Only the minimum necessary, explicitly allowlisted page data may cross the remote-planner boundary. Passwords, authentication tokens, hidden values, private form values, and high-risk personal data must never be included.

### Tasks

- [x] Create a dedicated redaction/sanitization module between browser extraction and planner serialization.
  - [x] Keep the raw local page model separate from the planner-safe page view.
  - [x] Make the planner payload type incapable of carrying unrestricted raw attributes or values.
  - [x] Avoid a generic `BTreeMap<String, String>` for planner-visible attributes.
- [x] Remove or redact sensitive input values at extraction time.
  - [x] Never collect values from `input[type=password]`.
  - [x] Never collect values from hidden inputs.
  - [x] Never collect one-time-password, authentication, API-key, secret, token, payment, or security-answer fields.
  - [x] Treat autocomplete hints such as `current-password`, `new-password`, `one-time-code`, and credit-card fields as sensitive.
  - [x] Treat suspicious names/IDs such as `token`, `secret`, `password`, `passwd`, `csrf`, `authorization`, `api_key`, `credit_card`, `ssn`, and equivalents as sensitive.
  - [x] Default unknown form-control values to omitted unless a specific local workflow needs them.
- [x] Replace full attribute collection with an allowlist.
  - [x] Allow only role/type and bounded safe label, placeholder, state, name, and navigation metadata needed for grounding.
  - [x] Exclude inline event handlers, `data-*` payloads, style text, hidden values, nonce/integrity data, authentication-related attributes, and DOM locators.
  - [x] Limit planner-visible attribute and label string lengths.
- [x] Redact URLs and links where necessary.
  - [x] Remove embedded credentials.
  - [x] Remove query parameters and fragments from planner-visible destinations.
  - [x] Detect common secret-bearing URL and text markers before serialization.
- [x] Bound planner-visible page content.
  - [x] Limit number of regions and interactive elements.
  - [x] Limit text per region and total serialized payload size.
  - [ ] Prefer relevance selection performed locally before remote transmission.
  - [x] Record truncation metadata without leaking omitted content.
- [ ] Add explicit remote-data consent and mode behavior.
  - [ ] Clearly indicate when page content will be sent to a remote provider.
  - [ ] Consider a local-only mode or per-origin opt-out.
  - [ ] Define handling for high-risk origins such as banking, healthcare, identity, password managers, and administrative consoles.
- [x] Sanitize every planner input source.
  - [x] Page snapshot.
  - [x] Page model.
  - [x] OCR output.
  - [x] Recent tool results and observations.
  - [x] Skill summaries or other untrusted text.
  - [x] Error details that may contain remote response bodies or page content.
- [ ] Prevent sensitive data from entering logs and diagnostics.
  - [ ] Audit `tracing` calls involving page models, planner payloads, HTTP errors, form data, and tool arguments.
  - [ ] Add structured redaction wrappers where appropriate.
  - Batch 7 removed raw remote response bodies from planner errors and keeps serialized remote payloads typed and redacted, but the broader repository-wide tracing/UI/Redux diagnostic audit remains open.

### Required regression tests

- [x] Password input values never appear in raw planner JSON.
- [x] Hidden input values never appear in planner JSON.
- [x] CSRF tokens and one-time codes never appear.
- [x] Credit-card and identity fields are redacted.
- [x] `data-*`, inline handlers, and arbitrary attributes are omitted.
- [x] Safe accessible labels and roles remain available for grounding.
- [x] Long pages are deterministically truncated within configured limits.
- [x] Sensitive URL query parameters are removed or redacted.
- [x] OCR text passes through the same redaction policy.
- [x] Recent tool history cannot reintroduce a secret that was removed from the page model.
- [ ] No secret appears in debug formatting or error details used by the UI.

### Acceptance criteria

- [x] A typed planner-safe page representation exists.
- [x] Remote planner requests contain no raw form values or unrestricted attributes.
- [ ] Privacy behavior is documented and tested.
  - The implemented serialization/redaction boundary is documented and tested by Batch 7 evidence. Explicit remote-transmission consent, per-origin controls, and the wider diagnostics audit remain open before this broader acceptance criterion can be closed.

### Batch 7 evidence

- Validated source commit: `fbec02a5b697720c88a3f46054110cd8e7c5c1a6`.
- Successful bounded validation: run `30746879137`, job `91493868153`.
- Complete all-feature Rust suite: 427 passed.
- Strict all-target/all-feature Clippy with warnings denied, frontend lint, UI tests, and production build all passed.
- Detailed evidence: `docs/BBCR-003_BBCR-006_BATCH7_FINAL_VALIDATION_EVIDENCE_2026-08-02.md`.
- The exact final documentation SHA and `ci/permanent` result are recorded in issue #5 to avoid mutating the validated final SHA.

---'''

bbcr006 = '''## BBCR-006 — Treat Page Content and OCR as Hostile Prompt-Injection Input

### Problem

Page text, attributes, OCR output, and related observations are untrusted content but are embedded in planner requests. Structural schema validation prevents invented tools but does not prevent malicious selection of valid tools.

### Required invariant

Untrusted content may inform grounding but may never alter trusted policy, authorize side effects, reveal protected data, change confirmation requirements, or instruct the agent to ignore system/runtime rules.

### Tasks

- [x] Separate trusted and untrusted planner payload sections.
  - [x] Place runtime policy, tool schemas, output schema, and safety constraints in a trusted contract section.
  - [x] Place page text, OCR text, attributes, skills, and tool observations in clearly labeled untrusted-data fields.
  - [x] Avoid concatenating untrusted content into trusted instruction text.
- [x] Strengthen the planner system prompt.
  - [x] State that webpage, OCR, document, skill, and tool-output text may contain malicious instructions.
  - [x] State that such instructions are evidence/data, not authority.
  - [x] Prohibit disclosure of hidden, redacted, credential, or system data.
  - [x] Prohibit using page instructions to bypass confirmation or policy.
- [x] Keep deterministic enforcement authoritative.
  - [x] Do not rely on prompt wording for confirmation, credential origin binding, redaction, filesystem safety, prohibited tools, or action authorization.
- [x] Add local prompt-injection indicators for telemetry or warnings without treating them as a complete defense.
  - [x] Detect common override, fake-authority, and secret-exfiltration phrases.
  - [x] Use detection only as caution telemetry; it cannot authorize or weaken an action.
- [ ] Add hostile-page fixtures to the agentic corpus.
  - [ ] Hidden text instructing the model to submit a form.
  - [x] Visible text instructing the model to ignore confirmation.
  - [x] Fake system/developer-message content.
  - [x] Instructions to reveal passwords or tokens.
  - [x] Instructions embedded in safe-label/placeholder/attribute and tool-observation inputs.
  - [ ] Instructions embedded in a real OCR image fixture.
  - [x] Instructions that disguise a destructive action as a harmless one.
- [x] Verify tool observations and errors cannot inject trusted planner instructions during replanning.

### Required regression tests

- [x] Malicious page text cannot cause unconfirmed submission.
- [ ] Hidden or real-image OCR-injected instructions cannot change safety policy.
  - Typed OCR/untrusted-text sanitization and deterministic policy are tested, but the complete hidden-DOM plus real OCR-image corpus remains open.
- [x] A page cannot cause the planner to request protected secrets.
- [x] A page cannot cause execution of unavailable or prohibited tools.
- [x] Replanning remains safe when the previous tool observation contains injection text.
- [x] Benign pages continue to produce useful planner payloads and plans.

### Acceptance criteria

- [x] Untrusted-data boundaries are explicit in types and prompts.
- [x] Hostile-page regression tests demonstrate deterministic safety even when the planner proposes unsafe actions.

### Batch 7 evidence

- Validated source commit: `fbec02a5b697720c88a3f46054110cd8e7c5c1a6`.
- Successful bounded validation: run `30746879137`, job `91493868153`.
- The remote payload is structurally separated into `trusted_contract`, `user_request`, and `untrusted_data`.
- Prompt-injection indicators are non-authoritative caution telemetry; deterministic runtime policy remains the final authority.
- Detailed evidence: `docs/BBCR-003_BBCR-006_BATCH7_FINAL_VALIDATION_EVIDENCE_2026-08-02.md`.
- The exact final documentation SHA and `ci/permanent` result are recorded in issue #5 to avoid mutating the validated final SHA.

---'''


todo = TODO_PATH.read_text()
todo = replace_between(todo, "## BBCR-003", "## BBCR-004", bbcr003)
todo = replace_between(
    todo,
    "## BBCR-006",
    "# P1 — Security and Reliability Hardening",
    bbcr006,
)
old_evidence_line = "- Page redaction/prompt-injection commit(s):"
new_evidence_line = (
    f"- Page redaction/prompt-injection commit(s): `{SOURCE_SHA}` (Batch 7)"
)
if todo.count(old_evidence_line) != 1:
    raise SystemExit(
        f"expected one evidence-record marker, found {todo.count(old_evidence_line)}"
    )
todo = todo.replace(old_evidence_line, new_evidence_line, 1)
TODO_PATH.write_text(todo)

report = REPORT_PATH.read_text()
report_marker = (
    "## Batch 7 — BBCR-003/BBCR-006 Remote Planner Privacy and Hostile-Input Boundary"
)
if report_marker in report:
    raise SystemExit("Batch 7 report section already exists")
report += f'''\n\n---\n\n{report_marker}\n\n**Status:** Core typed boundary implemented and validated; explicitly listed residual privacy/UI work remains open.  \n**Validated source commit:** `{SOURCE_SHA}`  \n**Bounded validation run:** `{RUN_ID}`  \n**Bounded validation job:** `{JOB_ID}`\n\n### Implemented\n\n- Added a dedicated typed remote-planner payload with `trusted_contract`, `user_request`, and `untrusted_data` sections.\n- Added planner-safe page, element, observation, skill, history, and runtime-state representations that cannot serialize raw form values, DOM locators, unrestricted attributes, local model paths, pending execution state, or credential metadata.\n- Changed DOM extraction to omit live form-control values and retain only a narrow, bounded grounding allowlist.\n- Sanitized page models, page snapshots, OCR-derived text, transcript/history, tool observations, skill descriptions, URLs, and error-derived text before remote serialization.\n- Removed credentials, queries, and fragments from planner-visible URLs and bounded all major collection/string/payload dimensions.\n- Added fail-closed handling for authentication, password, OTP/PIN, payment, identity, token, passkey, and similar sensitive contexts.\n- Strengthened the system prompt so page/OCR/skill/tool text is untrusted evidence and cannot override deterministic runtime policy.\n- Added non-authoritative prompt-injection caution indicators and adversarial tests for fake authority, confirmation bypass, credential requests, hostile skills/tool observations, unsafe action proposals, URL leakage, truncation, and safe-label preservation.\n- Removed raw remote response bodies from planner-facing error details.\n\n### Validation\n\n- Silent-fallback scan: passed.\n- Rust formatting: passed.\n- Default Rust compilation: passed.\n- Strict all-target/all-feature Clippy with warnings denied: passed.\n- Complete all-feature Rust test suite under Xvfb: 427 passed.\n- Frontend lint: passed.\n- UI tests: passed.\n- Production frontend build: passed.\n- Bounded change-set and one-shot workflow cleanup verification: passed.\n\n### Residual work intentionally left open\n\n- Explicit user indication/consent when page content is transmitted remotely.\n- Local-only mode and per-origin remote-planning opt-out.\n- Explicit high-risk-origin policy for banking, healthcare, identity, password-manager, and administrative sites.\n- Local relevance selection before transmission.\n- Repository-wide tracing, Redux-state, UI-error, invocation-instrumentation, and diagnostic secret-leak audit.\n- Complete hidden-DOM and real OCR-image prompt-injection corpus.\n\nDetailed evidence is recorded in `docs/BBCR-003_BBCR-006_BATCH7_FINAL_VALIDATION_EVIDENCE_2026-08-02.md`. The exact final documentation SHA and permanent-CI result are recorded in issue #5 so the validated SHA is not mutated afterward.\n'''
REPORT_PATH.write_text(report)

EVIDENCE_PATH.write_text(
    f'''# BBCR-003 / BBCR-006 Batch 7 Final Validation Evidence\n\n**Date:** 2026-08-02  \n**Repository:** `ekkus93/blind_browser`  \n**Branch:** `master`  \n**Validated source commit:** `{SOURCE_SHA}`  \n**Bounded validation run:** `{RUN_ID}`  \n**Bounded validation job:** `{JOB_ID}`  \n**Result:** Success\n\n## Scope\n\nBatch 7 establishes the typed remote-planner privacy boundary and the core hostile-input/prompt-injection boundary for BBCR-003 and BBCR-006. It does not claim completion of the explicitly listed consent/UI, relevance-selection, full diagnostic-audit, or complete hidden/OCR-image corpus residuals.\n\n## Proven implementation\n\n- Remote requests serialize a dedicated payload separated into `trusted_contract`, `user_request`, and `untrusted_data`.\n- Planner-safe types cannot carry raw form values, DOM locators, unrestricted attribute maps, local model paths, pending confirmation/execution state, or credential metadata.\n- Browser extraction omits live form-control values and collects only a bounded grounding allowlist.\n- Page model, snapshot, OCR text, transcript/history, tool observations, skills, URLs, and error-derived text pass through one remote sanitization boundary.\n- Planner-visible URLs omit credentials, query strings, and fragments.\n- High-risk authentication, password, OTP/PIN, payment, identity, token, and passkey contexts fail closed before a remote request.\n- Page/OCR/skill/tool text is explicitly labeled untrusted evidence and cannot override runtime policy.\n- Injection indicators are caution telemetry only and cannot authorize an action or reduce confirmation.\n- Deterministic runtime policy remains authoritative for confirmations, grounding, prohibited capabilities, credential handling, and filesystem safety.\n- Raw remote response bodies are excluded from application-facing planner errors.\n\n## Validation gates\n\nThe bounded worker passed all of the following before publishing the source commit:\n\n- exact-head and repository-state refusal checks;\n- deterministic transformation and generated-source invariants;\n- silent-fallback scan;\n- Rust formatting;\n- default Rust compilation;\n- strict all-target/all-feature Clippy with warnings denied;\n- complete all-feature Rust test suite under Xvfb: **427 passed**;\n- frontend lint;\n- UI test suite;\n- production frontend build;\n- whitespace validation;\n- bounded final change-set verification;\n- removal of all Batch 7 transformation, diagnostic, trigger, and workflow files before the source commit.\n\n## Regression coverage\n\nThe source tests cover typed payload shape, omission of raw values/locators/arbitrary attributes, password/hidden/OTP/token/payment/identity redaction, URL stripping, deterministic truncation, OCR and tool-history sanitization, hostile page/skill/tool-observation content, fake authority, credential requests, unsafe action proposals, prohibited tools, replanning safety, and preservation of safe grounding labels.\n\n## Residual work\n\nThe following remain open and are not part of this closure claim:\n\n- explicit remote-data indication and consent;\n- local-only mode or per-origin opt-out;\n- explicit high-risk-origin policy;\n- local relevance selection;\n- full tracing/UI/Redux/invocation diagnostic leak audit;\n- complete hidden-DOM and real OCR-image adversarial corpus.\n\n## Exact-final-SHA policy\n\nThis file intentionally does not embed the final documentation commit SHA or its Permanent CI run, because doing so would mutate the SHA after validation. The exact final SHA, Permanent CI run/job, and `ci/permanent` conclusion are recorded in GitHub issue #5.\n'''
)

memory = MEMORY_PATH.read_text()
entry_heading = "## 2026-08-02T14:47:00Z — Batch 7 remote-planner privacy boundary validated"
if entry_heading in memory:
    raise SystemExit("Batch 7 memory entry already exists")
entry = f'''{entry_heading}\n\n- Implemented the typed BBCR-003/BBCR-006 remote-planner boundary in `{SOURCE_SHA}`.\n- Bounded validation run `{RUN_ID}`, job `{JOB_ID}`, passed formatting, default compilation, strict all-target/all-feature Clippy, 427 Rust tests, frontend lint, UI tests, production build, bounded-change verification, and one-shot cleanup.\n- Remote payloads now separate trusted policy/schema from user request and untrusted page/OCR/skill/tool data; raw form values, DOM locators, unrestricted attributes, sensitive URLs, credential metadata, pending execution state, and raw remote error bodies cannot cross the remote boundary.\n- Prompt-injection indicators remain caution telemetry only; deterministic runtime policy owns confirmation and execution safety.\n- Residual consent/UI, high-risk-origin, relevance-selection, diagnostic-audit, and complete hidden/OCR-image corpus work remains explicitly open.\n- Exact final documentation SHA and Permanent CI evidence are recorded in issue #5.\n\n'''
MEMORY_PATH.write_text(entry + memory)

print("Batch 7 documentation closure applied")
