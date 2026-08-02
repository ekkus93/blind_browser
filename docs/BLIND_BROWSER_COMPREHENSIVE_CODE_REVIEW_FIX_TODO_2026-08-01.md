# Blind Browser Comprehensive Code Review Fix TODO

**Created:** 2026-08-01  
**Repository:** `ekkus93/blind_browser`  
**Reviewed baseline:** `3f5ce6f04524753855e3b3c33c0eab4410720764` (`master`)  
**Status:** Open  
**Scope:** Security, privacy, deterministic safety enforcement, credential handling, filesystem containment, remote-provider hardening, persistence durability, resource limits, frontend secret handling, CI, dependency security, and secret scanning.

---

## 1. Purpose

This document converts the findings from the 2026-08-01 comprehensive code review into an implementation-ready remediation plan.

The central architectural requirement is:

> Safety and security properties must be enforced by deterministic code. The remote planner may propose actions, but it must never be the authority that decides whether an action is safe, whether confirmation is required, what sensitive data may leave the device, or where stored credentials may be transmitted.

The implementation is not complete until every applicable checkbox is checked, all required tests pass, and the final evidence section is filled in with exact commands, commit SHAs, and CI run links.

---

## 2. Priority Definitions

- **P0 — Release blocker:** A defect that can permit unintended side effects, disclosure of credentials or private page data, or arbitrary local-file access. Do not release or use the application with sensitive accounts until these items are complete.
- **P1 — Security/reliability hardening:** A defect that can cause credential misuse, supply-chain compromise, hangs, partial updates, crash-durability failures, or resource exhaustion.
- **P2 — Defense in depth and CI:** Improvements that prevent regressions, detect dependency or secret issues, and expand platform confidence.
- **P3 — Documentation and operational follow-through:** Documentation, migration guidance, evidence capture, and cleanup required to make the remediation maintainable.

---

## 3. Global Completion Rules

- [ ] Do not mark a task complete solely because a planner prompt was changed.
- [ ] Do not mark a safety task complete unless deterministic Rust code enforces the invariant independently of planner output.
- [ ] Add a regression test that fails against the reviewed baseline for every bug fixed.
- [ ] Add negative tests for malformed, malicious, ambiguous, and boundary inputs.
- [ ] Preserve fail-closed behavior. Validation uncertainty must reject or require confirmation rather than silently proceed.
- [ ] Do not expose raw API keys, page secrets, passwords, tokens, or private form values in logs, errors, Redux state snapshots, test artifacts, or planner payloads.
- [ ] Keep all existing formatting, lint, unit-test, integration-test, and build gates green.
- [ ] Update relevant specifications and architecture documentation whenever an externally visible contract changes.
- [ ] Record the exact final commit SHA and CI evidence at the end of this file.

---

# P0 — Release Blockers

## BBCR-001 — Enforce Action Safety From Actual Tools, Not Planner Metadata

### Problem

The current validator applies submit-confirmation policy only when the planner declares `intent.name == SubmitForm`. A plan may declare a different intent while containing `SubmitActiveForm`. More broadly, the executor treats planner-selected `Ready` versus `NeedsConfirmation` as authoritative, so valid side-effecting tools may execute without the deterministic runtime applying the configured safety policy.

### Required invariant

The required confirmation level must be derived from the normalized action graph, actual tool names, validated arguments, deterministic grounding results, current safety configuration, and current runtime state. Planner-declared intent, status, reasons, and confirmation booleans may be checked for consistency but must not reduce the required safety level.

### Tasks

- [x] Inventory every `ToolName` and classify it in one centralized policy table.
  - [x] Classify tools as read-only, reversible local state change, browser navigation, page interaction, data entry, form submission, arbitrary script execution, credential operation, model download, or other side effect.
  - [x] Define the minimum confirmation requirement for each class.
  - [ ] Document why each tool is classified as it is.
  - [x] Make the classification exhaustive so adding a new `ToolName` causes a compile failure until its risk policy is specified.
- [x] Introduce a deterministic action-policy type, such as `ActionRisk`, `ConfirmationRequirement`, or an equivalent strongly typed representation.
  - [x] Include at least `NoConfirmation`, `ConfirmationRequired`, and `Prohibited` outcomes.
  - [x] Include a machine-readable reason code.
  - [x] Include the specific step IDs and normalized actions that caused the requirement.
- [x] Change planner-output validation to inspect the actual step list.
  - [x] Require confirmation for every plan containing `SubmitActiveForm`, regardless of `intent.name`.
  - [x] Reject any `Ready` plan containing an action that deterministic policy says requires confirmation.
  - [x] Reject any `Complete` or `Blocked` output that contains executable steps.
  - [x] Reject inconsistent planner metadata rather than trusting or repairing it silently.
- [x] Pass current safety settings into deterministic validation.
  - [x] Apply `always_confirm_submit` in deterministic code.
  - [x] Apply `allow_click_without_confirmation` in deterministic code.
  - [x] Apply `confirmation_confidence_threshold` using deterministic grounding evidence rather than a planner assertion.
  - [x] Define and test behavior when confidence is unavailable: fail closed or require confirmation.
- [x] Design a deterministic click-safety contract.
  - [x] Ensure `ClickElement` can be authorized only against a current, validated element resolution.
  - [x] Carry a deterministic grounding record or opaque authorization token from `FindElement`/resolution to `ClickElement`, rather than accepting an unverified element ID alone.
  - [x] Bind the grounding record to page identity, element identity, locator, confidence, visibility, enabled state, and a bounded age/version.
  - [x] Require confirmation when configured, when confidence is below threshold, when confidence is missing, or when the target is ambiguous or potentially destructive.
  - [x] Reject stale grounding after navigation, DOM replacement, page identity change, or relevant runtime-state change.
- [x] Treat `EvalJs` as a high-risk capability.
  - [x] Decide whether planner-generated arbitrary JavaScript should be prohibited entirely.
  - [ ] If retained, require explicit confirmation and a narrowly defined allowlist or constrained expression language.
  - [x] Prevent planner text from directly becoming unrestricted JavaScript without a deterministic policy decision.
- [x] Add executor-level defense in depth.
  - [x] Recompute or verify the required confirmation immediately before executing each side effect.
  - [x] Do not rely only on pre-execution planner validation.
  - [x] Return a stable fail-closed error code when an unconfirmed side effect reaches dispatch.
  - [ ] Ensure direct command entry points cannot bypass the same policy.
- [x] Validate plan/action consistency.
  - [x] Verify that `intent.name` is compatible with the actual tools, but never use the intent to weaken policy.
  - [x] Reject plans that disguise submit, data entry, scripting, or destructive clicks under unrelated intents.
  - [x] Reject planner-provided `requires_confirmation = false` when deterministic policy requires it.
- [x] Verify safety after replanning.
  - [x] Apply the same policy to every replanned output.
  - [x] Prevent a failed confirmed plan from replanning into an unconfirmed equivalent side effect.
  - [x] Ensure accumulated tool history cannot be used to bypass confirmation.

### Required regression tests

- [x] A `Ready` plan with `intent.name = ReadPage` and a `SubmitActiveForm` step is rejected.
- [x] A `Ready` plan with any tool classified as confirmation-required is rejected.
- [x] A plan cannot bypass submit confirmation by placing submit after a non-submit step.
- [x] A plan cannot bypass confirmation through `on_failure`, `NextStep`, `Replan`, or a cycle.
- [x] A replanned output containing a protected side effect still requires confirmation.
- [x] `allow_click_without_confirmation = false` forces confirmation for ordinary clicks.
- [x] `allow_click_without_confirmation = true` does not bypass confirmation for ambiguous, low-confidence, stale, or risky clicks.
- [x] Missing click confidence fails closed.
- [x] A click authorization becomes invalid after navigation or page-model replacement.
- [x] `EvalJs` follows the selected prohibit-or-confirm policy.
- [x] Executor defense-in-depth rejects a side effect even if validation is accidentally skipped in a test harness.
- [x] Existing intended safe, read-only plans continue to execute without unnecessary confirmation.

### Acceptance criteria

- [x] No planner-controlled field can reduce a deterministically calculated confirmation requirement.
- [x] Submit confirmation is enforced by actual tool presence.
- [x] Click confirmation follows current settings and deterministic grounding evidence.
- [ ] All side-effect entry points share the same policy.

---

## BBCR-002 — Bind Confirmation to the Exact Immutable Action Set

### Problem

The planner currently supplies the confirmation wording, while queued actions are stored separately. The user may therefore be shown an innocuous description that does not accurately describe the action that will execute.

### Required invariant

The confirmation prompt and approval token must be generated from, and cryptographically or structurally bound to, the exact normalized actions and arguments that will run. No action may be added, changed, reordered, or retargeted after approval.

### Tasks

- [x] Stop treating planner-provided `prompt_text` as authoritative confirmation copy.
  - [x] Planner text may be retained only as untrusted explanatory context.
  - [x] Generate the primary confirmation message in deterministic Rust code.
- [x] Define a normalized confirmation manifest.
  - [x] Include request ID, page ID, current origin, action sequence, tool names, normalized arguments, target descriptions, and relevant safety reasons.
  - [x] Redact sensitive values while preserving enough information for meaningful approval.
  - [x] Represent typed text by category and safe summary; do not speak or display passwords or full secrets.
- [x] Compute a stable digest or equivalent immutable identifier over the normalized manifest.
  - [x] Store the digest in pending execution state.
  - [x] Return it with the confirmation challenge.
  - [x] Require the same digest when applying the confirmation response.
- [x] Revalidate immediately before resume.
  - [x] Confirm the current page identity and origin still match.
  - [ ] Confirm all referenced elements and locators still resolve as expected.
  - [x] Confirm queued steps and arguments still hash to the approved manifest.
  - [x] Abort and replan if any relevant state changed.
- [ ] Define user-facing confirmation summaries for each protected action.
  - [ ] Submit: form identity, destination origin, and a safe list of fields being submitted.
  - [x] Click: target role/name and whether navigation or another consequential action is expected.
  - [ ] Data entry: target field and redacted value summary.
  - [ ] JavaScript: prohibit or clearly identify the exact constrained operation.
  - [ ] Multi-step flows: summarize the whole protected sequence, not only the first step.
- [x] Ensure confirmation cannot be reused.
  - [x] Mark confirmation IDs/digests consumed after one response.
  - [x] Reject replay, duplicate submission, mismatched ID, mismatched digest, and expired confirmation.
  - [x] Add a bounded expiration time.

### Required regression tests

- [x] Planner-supplied misleading prompt text cannot replace the deterministic summary.
- [x] Changing any queued tool or argument after prompt generation invalidates approval.
- [x] Reordering queued actions invalidates approval.
- [x] Navigation or origin change invalidates approval.
- [x] DOM/page identity change invalidates stale approval.
- [x] A confirmation response cannot be replayed.
- [x] Timeout and rejection clear all pending protected actions.
- [x] Sensitive field values are redacted in confirmation text and serialized pending state.

### Acceptance criteria

- [x] The user approves exactly the actions that execute.
- [x] Confirmation state is immutable, expiring, single-use, and state-bound.

---

## BBCR-003 — Add a Strict Page-Data Redaction Boundary Before Remote Planning

### Problem

DOM extraction currently captures live input values and all element attributes, including password, hidden, token-bearing, payment, identity, and private draft data. The resulting page model may be serialized into a remote planner request.

### Required invariant

Only the minimum necessary, explicitly allowlisted page data may cross the remote-planner boundary. Passwords, authentication tokens, hidden values, private form values, and high-risk personal data must never be included.

### Tasks

- [ ] Create a dedicated redaction/sanitization module between browser extraction and planner serialization.
  - [ ] Keep the raw local page model separate from the planner-safe page view.
  - [ ] Make the planner payload type incapable of carrying unrestricted raw attributes or values.
  - [ ] Avoid a generic `BTreeMap<String, String>` for planner-visible attributes.
- [ ] Remove or redact sensitive input values at extraction time.
  - [ ] Never collect values from `input[type=password]`.
  - [ ] Never collect values from hidden inputs.
  - [ ] Never collect one-time-password, authentication, API-key, secret, token, payment, or security-answer fields.
  - [ ] Treat autocomplete hints such as `current-password`, `new-password`, `one-time-code`, and credit-card fields as sensitive.
  - [ ] Treat suspicious names/IDs such as `token`, `secret`, `password`, `passwd`, `csrf`, `authorization`, `api_key`, `credit_card`, `ssn`, and equivalents as sensitive.
  - [ ] Default unknown form-control values to omitted unless a specific local workflow needs them.
- [ ] Replace full attribute collection with an allowlist.
  - [ ] Consider allowing only role, type category, safe name/label metadata, placeholder, checked/selected state, disabled state, and a redacted navigation destination.
  - [ ] Exclude inline event handlers, `data-*` payloads, style text, hidden values, nonce/integrity data, and authentication-related attributes.
  - [ ] Limit attribute string lengths.
- [ ] Redact URLs and links where necessary.
  - [ ] Remove embedded credentials.
  - [ ] Consider removing or hashing sensitive query parameters and fragments.
  - [ ] Define an allowlist/denylist for common secret-bearing parameters.
- [ ] Bound planner-visible page content.
  - [ ] Limit number of regions and interactive elements.
  - [ ] Limit text per region and total payload size.
  - [ ] Prefer relevance selection performed locally before remote transmission.
  - [ ] Record truncation metadata without leaking omitted content.
- [ ] Add explicit remote-data consent and mode behavior.
  - [ ] Clearly indicate when page content will be sent to a remote provider.
  - [ ] Consider a local-only mode or per-origin opt-out.
  - [ ] Define handling for high-risk origins such as banking, healthcare, identity, password managers, and administrative consoles.
- [ ] Sanitize every planner input source.
  - [ ] Page snapshot.
  - [ ] Page model.
  - [ ] OCR output.
  - [ ] Recent tool results and observations.
  - [ ] Skill summaries or other untrusted text.
  - [ ] Error details that may contain remote response bodies or page content.
- [ ] Prevent sensitive data from entering logs and diagnostics.
  - [ ] Audit `tracing` calls involving page models, planner payloads, HTTP errors, form data, and tool arguments.
  - [ ] Add structured redaction wrappers where appropriate.

### Required regression tests

- [ ] Password input values never appear in raw planner JSON.
- [ ] Hidden input values never appear in planner JSON.
- [ ] CSRF tokens and one-time codes never appear.
- [ ] Credit-card and identity fields are redacted.
- [ ] `data-*`, inline handlers, and arbitrary attributes are omitted.
- [ ] Safe accessible labels and roles remain available for grounding.
- [ ] Long pages are deterministically truncated within configured limits.
- [ ] Sensitive URL query parameters are removed or redacted.
- [ ] OCR text passes through the same redaction policy.
- [ ] Recent tool history cannot reintroduce a secret that was removed from the page model.
- [ ] No secret appears in debug formatting or error details used by the UI.

### Acceptance criteria

- [ ] A typed planner-safe page representation exists.
- [ ] Remote planner requests contain no raw form values or unrestricted attributes.
- [ ] Privacy behavior is documented and tested.

---

## BBCR-004 — Bind Stored Credentials to Approved Origins

### Problem

Model listing accepts an endpoint override and may combine it with the configured API key and organization headers. An unsaved or malicious endpoint can therefore receive a stored credential.

### Required invariant

A stored credential may be sent only to the exact normalized origin for which it was explicitly stored or approved. Changing scheme, host, port, or approved path prefix requires explicit reauthorization and never silently reuses the existing secret.

### Tasks

- [x] Introduce a normalized provider-origin type.
  - [x] Normalize scheme, hostname, effective port, and allowed path prefix.
  - [x] Reject embedded username/password information.
  - [x] Reject fragments.
  - [x] Define path normalization rules.
- [x] Bind each keyring entry to provider kind, profile, and normalized destination scope.
  - [x] Include destination identity in the keyring account naming.
  - [x] Prevent one profile's key from being reused by another destination without explicit rebinding.
- [x] Change model listing and API-key testing behavior.
  - [x] If the endpoint differs from the stored destination, do not resolve or attach the configured key.
  - [x] Require a newly entered key, or save the endpoint and re-enter the key to bind it to that destination.
  - [x] Display the exact endpoint in the editable destination field and normalized success/error messages before or after the explicit action.
  - [x] Do not automatically load models on blur.
- [x] Separate endpoint editing from credential-bearing network actions.
  - [x] Validate the endpoint before transmission; stored-key rebinding requires saving the endpoint and re-entering the key. An unsaved endpoint can use only a newly entered temporary key.
  - [x] Require explicit user action to test or list models.
  - [x] Prohibit reuse of a stored key after a destination change rather than offering a weaker confirmation-only path.
- [x] Restrict organization and project headers to the bound destination scope.
- [x] Add redirect controls.
  - [x] Prohibit redirects for credential-bearing requests.
  - [x] Prevent authorization headers from being forwarded cross-origin.
  - [x] Test same-origin and cross-origin redirect refusal.
- [x] Ensure error messages never include the secret or authorization header.
- [x] Provide migration behavior for existing keyring entries.
  - [x] Detect legacy unbound entries.
  - [x] Require explicit rebinding rather than guessing.
  - [x] Document cleanup of orphaned legacy entries in `docs/BBCR-004_PR_VALIDATION_EVIDENCE_2026-08-01.md`.

### Required regression tests

- [x] A changed host cannot receive the stored key.
- [x] A changed scheme cannot receive the stored key.
- [x] A changed port cannot receive the stored key.
- [x] A changed path is handled according to the documented path-prefix policy.
- [x] Empty key override plus changed endpoint fails closed.
- [x] Organization and project headers are not sent to a changed destination.
- [x] Cross-origin redirects do not receive authorization headers.
- [x] Same-origin redirects are also refused.
- [x] Explicitly entered temporary credentials are scoped to the displayed approved destination.
- [x] Existing same-destination API-key testing and model listing continue to work.

### Acceptance criteria

- [x] Stored credentials are destination-bound.
- [x] Endpoint edits cannot silently exfiltrate stored credentials.

### Evidence

- Cleaned implementation commit: `4d38b71363a83dc343dfd555a9e3a353ed6801b1`.
- Successful bounded finalizer: run `30722003167`, job `91427197740`.
- Synchronized implementation commit: `3f91c2d716f83adfdc807ea3fa0eb7ad1da63296`.
- Prior owner-authored exact-head validation: commit `27dda7c43f2015cc33c051120dd1e721cc49c0b0`, run `30723051745`, job `91429862875`.
- The final TODO-closure head and its permanent CI identifiers are recorded in PR #4 and issue #5 so recording them does not mutate the exact validated SHA.

---

## BBCR-005 — Replace Filesystem-Derived Image IDs With Opaque Contained Handles

### Problem

`RunOcrInput.image_id` is caller-controlled and is used to construct a filesystem path without strict validation or canonical containment checks. Absolute paths or traversal components may escape the screenshot cache.

### Required invariant

External/planner-visible image identifiers must be opaque, application-generated handles that resolve only through an internal registry to files beneath the canonical screenshot cache directory.

### Tasks

- [ ] Define an opaque image-handle type.
  - [ ] Use a UUID, random nonce, or strict application-generated identifier.
  - [ ] Implement strict parsing and length limits.
  - [ ] Reject path separators, dots-as-components, absolute paths, percent-encoded separators, Unicode separator tricks, control characters, and whitespace.
- [ ] Maintain an internal image registry.
  - [ ] Map handle to canonical path, owning page ID, creation time, size, and optional content hash.
  - [ ] Do not derive arbitrary paths directly from caller-provided strings.
  - [ ] Remove entries when files are deleted or expire.
- [ ] Canonicalize and verify containment.
  - [ ] Canonicalize the screenshot cache root.
  - [ ] Canonicalize or safely create the target file.
  - [ ] Verify every resolved path remains beneath the root.
  - [ ] Reject symlinks or use safe no-follow semantics where available.
- [ ] Harden screenshot file creation.
  - [ ] Use unique filenames independent of request IDs.
  - [ ] Create files with restrictive permissions.
  - [ ] Avoid overwriting an existing screenshot silently.
- [ ] Bind images to runtime context.
  - [ ] Record page ID and origin at capture time.
  - [ ] Decide whether OCR may use an image captured from a previous page.
  - [ ] Reject stale or cross-page handles unless explicitly supported.
- [ ] Add cleanup policy.
  - [ ] Set maximum cache count and total bytes.
  - [ ] Delete expired screenshots.
  - [ ] Remove registry entries transactionally with files.
- [ ] Audit other filesystem-facing identifiers and paths.
  - [ ] Request IDs used in filenames.
  - [ ] Model IDs and model paths.
  - [ ] Skill names/paths.
  - [ ] Configured directories.
  - [ ] Any future export/import path.

### Required regression tests

- [ ] Reject `../` traversal.
- [ ] Reject absolute Unix paths.
- [ ] Reject Windows drive and UNC paths.
- [ ] Reject encoded and Unicode separator variants.
- [ ] Reject symlink escape from the cache root.
- [ ] Reject unknown, expired, and cross-page handles.
- [ ] Valid captured image handles still resolve and OCR correctly.
- [ ] Cleanup removes files and registry entries without races.

### Acceptance criteria

- [ ] No planner-controlled string is converted directly into a local screenshot path.
- [ ] Canonical containment is enforced and tested on supported platforms.

---

## BBCR-006 — Treat Page Content and OCR as Hostile Prompt-Injection Input

### Problem

Page text, attributes, OCR output, and related observations are untrusted content but are embedded in planner requests. Structural schema validation prevents invented tools but does not prevent malicious selection of valid tools.

### Required invariant

Untrusted content may inform grounding but may never alter trusted policy, authorize side effects, reveal protected data, change confirmation requirements, or instruct the agent to ignore system/runtime rules.

### Tasks

- [ ] Separate trusted and untrusted planner payload sections.
  - [ ] Place runtime policy, tool schemas, and safety constraints in a trusted system/developer section.
  - [ ] Place page text, OCR text, attributes, and tool observations in clearly labeled untrusted-data fields.
  - [ ] Avoid concatenating untrusted content into instruction text.
- [ ] Strengthen the planner system prompt.
  - [ ] State that webpage, OCR, document, and tool-output text may contain malicious instructions.
  - [ ] State that such instructions are data, not authority.
  - [ ] Prohibit disclosure of hidden, redacted, credential, or system data.
  - [ ] Prohibit using page instructions to bypass confirmation or policy.
- [ ] Keep deterministic enforcement authoritative.
  - [ ] Do not rely on prompt wording for confirmation, credential origin binding, redaction, or filesystem safety.
- [ ] Add local prompt-injection indicators for telemetry or warnings without treating them as a complete defense.
  - [ ] Detect common override and secret-exfiltration phrases.
  - [ ] Use detection only to raise caution or require confirmation, never to permit action.
- [ ] Add hostile-page fixtures to the agentic corpus.
  - [ ] Hidden text instructing the model to submit a form.
  - [ ] Visible text instructing the model to ignore confirmation.
  - [ ] Fake system-message content.
  - [ ] Instructions to reveal passwords or tokens.
  - [ ] Instructions embedded in `aria-label`, placeholder, attributes, and OCR images.
  - [ ] Instructions that disguise a destructive action as a harmless one.
- [ ] Verify tool observations and errors cannot inject trusted planner instructions during replanning.

### Required regression tests

- [ ] Malicious page text cannot cause unconfirmed submission.
- [ ] Hidden or OCR-injected instructions cannot change safety policy.
- [ ] A page cannot cause the planner to request protected secrets.
- [ ] A page cannot cause execution of unavailable or prohibited tools.
- [ ] Replanning remains safe when the previous tool observation contains injection text.
- [ ] Benign pages continue to produce useful plans.

### Acceptance criteria

- [ ] Untrusted-data boundaries are explicit in types and prompts.
- [ ] Hostile-page regression tests demonstrate deterministic safety even when the planner proposes unsafe actions.

---

# P1 — Security and Reliability Hardening

## BBCR-007 — Secure the Model Download Supply Chain

### Problem

Model downloads use mutable `resolve/main` URLs and accept unverified response bytes without pinned revisions, hashes, signatures, size ceilings, or request timeouts.

### Tasks

- [ ] Define a signed or code-pinned model manifest.
  - [ ] Pin repository and immutable revision/commit for every supported model.
  - [ ] List exact expected files.
  - [ ] Store expected SHA-256 or stronger digest for every file.
  - [ ] Store expected minimum and maximum byte sizes.
  - [ ] Store model/backend compatibility metadata.
- [ ] Change download URLs to immutable revisions rather than `main`.
- [ ] Add network timeouts.
  - [ ] Connection timeout.
  - [ ] Read/overall timeout.
  - [ ] Optional low-speed timeout.
- [ ] Add strict size enforcement.
  - [ ] Reject declared `Content-Length` above the maximum.
  - [ ] Enforce a streaming byte ceiling when length is absent or dishonest.
  - [ ] Abort and remove partial files on overflow.
- [ ] Verify downloaded bytes before activation.
  - [ ] Compute digest while streaming.
  - [ ] Compare in constant-time or equivalent safe equality where appropriate.
  - [ ] Reject mismatch and remove the partial file.
  - [ ] Validate expected file type/format where feasible.
- [ ] Make multi-file model installation transactional.
  - [ ] Download into a unique staging directory.
  - [ ] Verify all files.
  - [ ] Atomically activate the complete directory.
  - [ ] Never leave a partially installed model marked available.
- [ ] Handle redirects safely.
  - [ ] Restrict allowed destination hosts for Hugging Face/CDN redirects.
  - [ ] Limit redirect count.
  - [ ] Do not forward unrelated credentials.
- [ ] Record provenance.
  - [ ] Persist repository, revision, digest, size, and installation time.
  - [ ] Report this information in runtime/model status.
- [ ] Add an explicit update workflow rather than silently following `main`.

### Required regression tests

- [ ] Hash mismatch is rejected.
- [ ] Oversized response is aborted and partial files are removed.
- [ ] Timeout leaves no activated model.
- [ ] One failed file prevents activation of a multi-file model.
- [ ] Existing valid model remains intact after failed update.
- [ ] Unexpected redirect host is rejected.
- [ ] Correct pinned files install and are reported available.

### Acceptance criteria

- [ ] Every managed model is revision-pinned and hash-verified before use.
- [ ] Partial, oversized, timed-out, or modified downloads cannot become active.

---

## BBCR-008 — Apply Real Planner Request Timeouts and Cancellation

### Problem

Planner profiles contain `timeout_ms`, but the remote planner path does not apply an actual request-level timeout comparable to the ASR and TTS paths.

### Tasks

- [ ] Apply `profile.timeout_ms` to OpenAI-compatible planner requests.
  - [ ] Use a client-level timeout and/or async timeout wrapper that cancels the underlying operation.
  - [ ] Avoid spawning work that continues after the caller times out.
- [ ] Distinguish timeout, connection, TLS, HTTP status, parse, and schema-validation errors.
- [ ] Return stable error codes and correct `retryable` values.
- [ ] Bound response-body size before parsing.
- [ ] Ensure lock release remains correct during timed network calls.
- [ ] Ensure cancellation does not leave stale planner state or pending execution.
- [ ] Apply the same behavior to OpenAI and Ollama-compatible paths.

### Required regression tests

- [ ] A server that never responds is terminated at the configured timeout.
- [ ] The underlying request does not continue indefinitely after timeout.
- [ ] Timeout does not hold the `AppCore` mutex.
- [ ] Oversized response is rejected before unbounded allocation.
- [ ] Normal planner requests still succeed.

### Acceptance criteria

- [ ] Planner timeout settings are enforced by the actual network operation.

---

## BBCR-009 — Create a Strict Remote Endpoint Policy

### Problem

Remote endpoint validation currently accepts any absolute URL that `reqwest::Url` parses, including unsafe or unsupported schemes and credential-bearing URLs.

### Tasks

- [ ] Create a shared `RemoteEndpointPolicy` separate from browser navigation policy.
- [ ] Require `https` for non-loopback remote services.
- [ ] Permit `http` only for explicitly recognized loopback hosts used by local services.
  - [ ] Support `localhost`, `127.0.0.0/8`, and `::1` according to a documented policy.
  - [ ] Decide whether private LAN hosts are permitted; default to HTTPS or explicit advanced opt-in.
- [ ] Reject unsupported schemes.
- [ ] Reject embedded username/password information.
- [ ] Reject fragments.
- [ ] Validate host and port.
- [ ] Normalize trailing slashes and API path prefixes consistently.
- [ ] Prevent DNS rebinding/SSRF surprises where applicable.
  - [ ] Evaluate resolving hostnames and prohibiting unexpected local/private address transitions for remote profiles.
  - [ ] Treat loopback exceptions explicitly rather than by hostname string alone.
- [ ] Use the policy for save, test, model-list, planner, TTS, and ASR paths.
- [ ] Present normalized destination information to the UI.

### Required regression tests

- [ ] Reject `file:`, `data:`, `javascript:`, FTP, and unknown schemes.
- [ ] Reject non-loopback plain HTTP.
- [ ] Accept documented loopback HTTP endpoints.
- [ ] Reject userinfo credentials.
- [ ] Reject malformed authorities and missing hosts.
- [ ] Normalize valid endpoints consistently across all provider operations.

### Acceptance criteria

- [ ] All credential-bearing and remote-provider paths share one strict endpoint policy.

---

## BBCR-010 — Make Keyring and Configuration Updates Transactional

### Problem

API-key persistence mutates the keyring before fully validating and committing the corresponding config update. Failure can leave an orphaned or unexpectedly replaced secret.

### Tasks

- [ ] Validate the complete proposed configuration before mutating the keyring.
  - [ ] Verify profile existence and type.
  - [ ] Verify serialized config can be produced.
  - [ ] Verify endpoint-origin binding metadata.
- [ ] Design an explicit transaction/recovery strategy.
  - [ ] Read and retain the previous keyring value/reference when possible.
  - [ ] Stage the new config separately.
  - [ ] Apply keyring and config changes in a defined order.
  - [ ] Roll back the keyring if config commit fails.
  - [ ] If rollback fails, return a distinct recovery-required error and preserve actionable metadata without exposing the secret.
- [ ] Avoid updating the in-memory cache until the durable operation succeeds.
- [ ] Clean up orphaned keyring entries.
  - [ ] Add a safe migration/maintenance operation.
  - [ ] Never enumerate or display secret values.
- [ ] Make errors explicit about which component committed and which failed.
- [ ] Add fault injection around each transaction stage.

### Required regression tests

- [ ] Unknown profile does not mutate keyring state.
- [ ] Config parse/serialization failure does not mutate keyring state.
- [ ] Config write failure restores the previous keyring value.
- [ ] Keyring failure does not write a config reference to a nonexistent secret.
- [ ] In-memory config/cache remains consistent after every failure point.
- [ ] Successful update commits both keyring and config.

### Acceptance criteria

- [ ] Keyring and config cannot silently diverge after a handled failure.

---

## BBCR-011 — Complete Crash-Durable and Concurrent-Safe Atomic Persistence

### Problem

Temporary-file writes and atomic rename are used, but parent-directory fsync, unique temp names, and cross-process coordination are missing.

### Tasks

- [ ] Replace fixed `.tmp` and `.part` names with unique same-directory temporary files.
- [ ] Create temporary files with exclusive creation semantics.
- [ ] Write and flush all bytes.
- [ ] Call file `sync_all()` before rename.
- [ ] Atomically replace the target.
- [ ] Sync the parent directory after rename on platforms that support it.
- [ ] Document Windows/macOS/Linux differences and guarantees.
- [ ] Add cross-process or cross-instance locking for config updates.
  - [ ] Prevent two application instances from interleaving read-modify-write operations.
  - [ ] Define lock timeout and stale-lock recovery.
- [ ] Preserve permissions and ownership appropriately.
- [ ] Clean up abandoned temporary files safely at startup.
- [ ] Reuse the hardened primitive for config and model activation where appropriate.
- [ ] Add fault injection for write, sync, rename, directory sync, and cleanup failures.

### Required regression tests

- [ ] Concurrent writers do not corrupt config.
- [ ] Temp filenames do not collide.
- [ ] Failure before rename preserves the old target.
- [ ] Failure after temp write leaves no active partial target.
- [ ] Parent-directory sync is invoked on supported platforms.
- [ ] Startup cleanup does not remove a valid active file.

### Acceptance criteria

- [ ] Persistence has documented crash and concurrency semantics on every supported platform.

---

## BBCR-012 — Add Resource and Payload Limits

### Problem

DOM extraction, planner payloads, full-page screenshots, remote responses, OCR, TTS, and downloads lack comprehensive hard limits. Malicious or pathological inputs can cause high memory use, disk exhaustion, excessive remote cost, or long processing times.

### Tasks

- [ ] Define centralized resource budgets.
  - [ ] Maximum page regions.
  - [ ] Maximum interactive elements.
  - [ ] Maximum text per region.
  - [ ] Maximum total planner payload bytes.
  - [ ] Maximum tool-history entries and bytes.
  - [ ] Maximum screenshot dimensions, pixels, encoded bytes, cache count, and cache bytes.
  - [ ] Maximum OCR input image size and OCR output text.
  - [ ] Maximum planner, TTS, ASR, and model-list response bytes.
  - [ ] Maximum model download bytes.
- [ ] Apply limits before allocation where possible.
- [ ] Stream large responses instead of reading unbounded bodies into memory.
- [ ] Add deterministic truncation.
  - [ ] Preserve the most relevant content.
  - [ ] Include safe truncation metadata.
  - [ ] Never split or accidentally reveal redacted values.
- [ ] Add bounded cache eviction for screenshots and synthesized speech.
- [ ] Add request-rate and concurrency limits for expensive operations.
  - [ ] Planner requests.
  - [ ] OCR.
  - [ ] Full-page screenshots.
  - [ ] Model downloads.
  - [ ] TTS/ASR.
- [ ] Surface user-meaningful errors when limits are exceeded.
- [ ] Add metrics/telemetry that record sizes but not sensitive content.

### Required regression tests

- [ ] Extremely large DOM is truncated within limits.
- [ ] Oversized screenshot is rejected before disk/memory exhaustion.
- [ ] Oversized remote response is rejected.
- [ ] Cache eviction respects count and byte budgets.
- [ ] Concurrent expensive operations are bounded.
- [ ] Normal representative pages remain usable.

### Acceptance criteria

- [ ] Every untrusted or remote-controlled byte stream has an explicit tested limit.

---

## BBCR-013 — Remove Raw API-Key Drafts From Global Redux State

### Problem

API-key drafts are stored in Redux panel state. Failed saves/tests can leave them in globally inspectable state and development tooling.

### Tasks

- [ ] Move raw API-key drafts to component-local ephemeral state or an isolated secret-input controller.
- [ ] Keep only non-secret metadata in Redux.
  - [ ] Saved/not saved.
  - [ ] Masked reference.
  - [ ] Operation status.
  - [ ] Non-sensitive test result.
- [ ] Clear secret drafts after success, failure, cancellation, navigation away, component unmount, and application lock/exit.
- [ ] Disable Redux DevTools in production unless explicitly needed.
- [ ] Add middleware safeguards against secret-shaped values entering Redux actions/state.
- [ ] Avoid logging action payloads that may contain secrets.
- [ ] Ensure frontend errors never echo the submitted key.
- [ ] Review Tauri invocation instrumentation to ensure arguments are not logged.
- [ ] Consider using a mutable character buffer with best-effort zeroization where practical, while documenting JavaScript limitations.

### Required regression tests

- [ ] API-key characters never appear in Redux state.
- [ ] API-key characters never appear in dispatched Redux actions.
- [ ] Failure and cancellation clear local draft state.
- [ ] Navigation away clears local draft state.
- [ ] Production store does not expose DevTools state containing secret drafts.
- [ ] Existing save/test UX remains accessible.

### Acceptance criteria

- [ ] Raw API keys exist only for the minimum time and scope needed to invoke the backend command.

---

## BBCR-014 — Tighten CSP and Frontend Network Boundaries

### Problem

The CSP permits frontend connections to any HTTPS origin. If frontend script execution is compromised, arbitrary HTTPS exfiltration is allowed.

### Tasks

- [ ] Inventory every legitimate frontend network connection.
- [ ] Route provider and model-download traffic through validated Rust commands.
- [ ] Reduce `connect-src` to the minimum required set.
  - [ ] Prefer `'self'` only when all external traffic is backend-mediated.
  - [ ] Avoid broad `https:` unless a documented technical requirement remains.
- [ ] Verify no frontend component can issue arbitrary fetch/XHR/WebSocket requests.
- [ ] Review other CSP directives.
  - [ ] `script-src` without unsafe inline/eval.
  - [ ] `object-src 'none'`.
  - [ ] `base-uri 'none'` or equivalent.
  - [ ] Restrictive `frame-src`/`frame-ancestors` where supported.
  - [ ] Restrictive image/media/font sources.
- [ ] Add a build/test assertion for the expected CSP.
- [ ] Review Tauri capabilities whenever new plugins are added.

### Required regression tests

- [ ] Frontend cannot connect to an arbitrary external HTTPS test server.
- [ ] Required backend-mediated provider operations still work.
- [ ] CSP remains present and non-null in production configuration.
- [ ] No new broad Tauri permissions are introduced.

### Acceptance criteria

- [ ] Frontend code has no general-purpose network exfiltration path.

---

## BBCR-015 — Revalidate Plans Against the State Snapshot Used for Planning

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

---

# P2 — CI, Dependency, and Secret-Handling Defense in Depth

## BBCR-016 — Add Automated Secret Scanning for Current Tree and Git History

### Problem

No real key was found in the reviewed current tree, but the repository lacks automated secret scanning and `.env` protections. The review did not constitute a byte-for-byte scan of every reachable and unreachable Git object.

### Tasks

- [ ] Add Gitleaks, TruffleHog, or an equivalent maintained scanner to CI.
- [ ] Scan the current working tree and full reachable Git history.
- [ ] Run a one-time local/admin scan that includes deleted branches/tags and, where practical, unreachable objects.
- [ ] Add a baseline/allowlist only for clearly synthetic fixtures.
  - [ ] Document why `sk-proj-test-secret` and other fixtures are fake.
  - [ ] Prefer unmistakable non-live prefixes in future tests.
  - [ ] Do not broadly suppress `sk-` patterns.
- [ ] Enable GitHub secret scanning and push protection if available for the repository.
- [ ] Add pre-commit or pre-push secret scanning guidance.
- [ ] Define incident response.
  - [ ] Immediately revoke and rotate any discovered live credential.
  - [ ] Remove it from current content.
  - [ ] Rewrite history only after assessing clone/fork impact.
  - [ ] Document affected commits and remediation.
- [ ] Store scan results as CI evidence without exposing matched secrets.

### Required regression tests/validation

- [ ] CI fails on an intentionally added fake detector test pattern in a temporary test branch or scanner self-test.
- [ ] Known synthetic fixtures are narrowly allowlisted.
- [ ] Full-history scan completes successfully on the final remediation commit.

### Acceptance criteria

- [ ] Secret scanning is mandatory and continuously enforced.
- [ ] Final evidence states whether any historical live secret was found and what was rotated.

---

## BBCR-017 — Expand `.gitignore` and Local Secret Hygiene

### Tasks

- [ ] Add common environment files:
  - [ ] `.env`
  - [ ] `.env.*`
  - [ ] Preserve an explicit `!.env.example` exception if an example is needed.
- [ ] Ignore common private-key and credential artifacts:
  - [ ] `*.pem`
  - [ ] `*.key`
  - [ ] `*.p12`
  - [ ] `*.pfx`
  - [ ] `credentials.*`
  - [ ] `secrets.*`
- [ ] Ignore local OS keyring/export/test artifacts where applicable.
- [ ] Add safe example configuration files containing references/placeholders only.
- [ ] Document that `.gitignore` does not protect already tracked files and is not a substitute for secret scanning.
- [ ] Add a CI check that forbidden local secret filenames are not tracked.

### Acceptance criteria

- [ ] Common accidental secret files are ignored and CI-detected if tracked.

---

## BBCR-018 — Add Dependency, License, and Static Security Analysis Gates

### Tasks

- [ ] Add Rust dependency auditing.
  - [ ] `cargo audit` or equivalent advisory scan.
  - [ ] `cargo deny` for advisories, bans, duplicate policy, sources, and licenses.
- [ ] Add JavaScript dependency auditing.
  - [ ] Use the package manager's supported audit command or a maintained alternative.
  - [ ] Define severity thresholds and exception process.
- [ ] Add CodeQL or equivalent SAST for Rust/JavaScript where supported.
- [ ] Pin GitHub Actions by full commit SHA.
  - [ ] Document the upstream release/tag in comments.
  - [ ] Add a dependency-update mechanism such as Dependabot/Renovate.
- [ ] Review all Git dependencies and pin immutable revisions.
- [ ] Add license-policy enforcement for bundled dependencies and models.
- [ ] Fail CI on unexpected lockfile changes or unapproved sources.

### Acceptance criteria

- [ ] Dependency advisories, prohibited licenses/sources, and static-analysis findings are visible and gated in CI.

---

## BBCR-019 — Expand Platform and Packaged-Application CI

### Tasks

- [ ] Add Windows CI for Rust tests, frontend tests/build, and platform-specific path/persistence behavior.
- [ ] Add macOS CI for the same gates.
- [ ] Keep Linux CI.
- [ ] Add platform-specific tests for:
  - [ ] Path containment and separator handling.
  - [ ] Keyring behavior or a faithful test abstraction.
  - [ ] Atomic replacement and directory sync behavior.
  - [ ] Browser launch configuration.
  - [ ] Tauri capability/config validation.
- [ ] Build packaged desktop artifacts in CI.
- [ ] Add a bounded packaged-app smoke test where feasible.
- [ ] Verify production CSP and Tauri capabilities in packaged output.
- [ ] Publish test artifacts/logs without secrets.

### Acceptance criteria

- [ ] Supported desktop platforms have green validation on the exact final SHA.

---

## BBCR-020 — Add Security-Focused Fuzzing, Property Tests, and Mutation Coverage

### Tasks

- [ ] Fuzz URL and endpoint normalization.
- [ ] Fuzz image-handle parsing and path-containment logic.
- [ ] Fuzz planner-output deserialization and semantic validation.
- [ ] Fuzz confirmation-manifest canonicalization/digest validation.
- [ ] Property-test redaction to ensure known secret classes never survive sanitization.
- [ ] Property-test resource limits and truncation.
- [ ] Add mutation testing for critical validators and safety gates.
- [ ] Define minimum mutation/coverage expectations for security-critical modules.
- [ ] Seed fuzz corpora with all malicious examples from this TODO.

### Acceptance criteria

- [ ] Critical policy code has tests that fail when safety checks are removed or inverted.

---

# P3 — Documentation, Migration, and Operational Completion

## BBCR-021 — Update Architecture and Security Documentation

### Tasks

- [ ] Update `docs/SPECS.md` with the deterministic safety authority model.
- [ ] Document the action-risk policy and confirmation requirements.
- [ ] Document the confirmation manifest, digest, expiry, and replay prevention.
- [ ] Document planner-safe page data and redaction rules.
- [ ] Document remote data transmission and user consent behavior.
- [ ] Document credential-origin binding and endpoint-change behavior.
- [ ] Document model provenance, revisions, hashes, and update process.
- [ ] Document resource limits.
- [ ] Document supported platforms and persistence guarantees.
- [ ] Update README setup guidance for secrets and `.env` handling.
- [ ] Add a security-reporting policy or `SECURITY.md`.
- [ ] Add a threat-model document covering:
  - [ ] Malicious webpages and prompt injection.
  - [ ] Compromised or malicious remote planner.
  - [ ] Credential exfiltration.
  - [ ] Local filesystem traversal.
  - [ ] Compromised model host.
  - [ ] Malicious oversized responses.
  - [ ] Concurrent commands and stale state.

### Acceptance criteria

- [ ] Documentation matches implemented behavior and test evidence.

---

## BBCR-022 — Add Migration and Recovery Procedures

### Tasks

- [ ] Define migration for legacy keyring entries without origin binding.
- [ ] Define migration for existing config schema changes.
- [ ] Define cleanup for old screenshot files and path-derived IDs.
- [ ] Define cleanup for partially downloaded models.
- [ ] Define behavior for existing unverified model files.
  - [ ] Reverify against a known manifest or require redownload.
- [ ] Define user-visible recovery messages for transaction failures.
- [ ] Ensure migrations are idempotent and fail closed.
- [ ] Back up non-secret configuration before destructive migration.
- [ ] Add migration tests from representative previous versions.

### Acceptance criteria

- [ ] Existing users can upgrade without silently retaining unsafe credential bindings or unverified artifacts.

---

## BBCR-023 — Remove Dead or Misleading Safety Contracts

### Tasks

- [ ] Audit fields that imply safety but are not authoritative.
  - [ ] `PlannerStatus`.
  - [ ] `requires_confirmation`.
  - [ ] `confirmation_reason`.
  - [ ] `always_confirm_submit` usage.
  - [ ] Planner-supplied confirmation prompt fields.
- [ ] Clearly mark planner-provided values as proposals or remove redundant fields.
- [ ] Ensure UI labels do not claim stronger guarantees than deterministic code provides.
- [ ] Remove obsolete compatibility helpers after migration.
- [ ] Update generated TypeScript contracts after Rust schema changes.
- [ ] Check all docs and tests for stale claims such as “submit actions always require confirmation” before deterministic enforcement is complete.

### Acceptance criteria

- [ ] Public types and documentation accurately distinguish proposed planner metadata from enforced runtime policy.

---

# 4. Recommended Implementation Sequence

The following sequence minimizes the period where one fix depends on another incomplete boundary.

- [ ] **Stage 1 — Freeze unsafe release use**
  - [ ] Document that sensitive real-world use is blocked until P0 is complete.
  - [ ] Add failing regression tests for BBCR-001 through BBCR-006 before implementation.
- [ ] **Stage 2 — Build deterministic action policy**
  - [ ] Implement BBCR-001.
  - [ ] Implement state/version validation from BBCR-015.
- [ ] **Stage 3 — Bind confirmations**
  - [ ] Implement BBCR-002.
- [ ] **Stage 4 — Establish privacy boundary**
  - [ ] Implement BBCR-003.
  - [ ] Implement BBCR-006 hostile-input handling.
- [ ] **Stage 5 — Lock down credentials and endpoints**
  - [ ] Implement BBCR-004.
  - [ ] Implement BBCR-009.
  - [ ] Implement BBCR-010.
  - [ ] Implement BBCR-013.
- [ ] **Stage 6 — Lock down filesystem and supply chain**
  - [ ] Implement BBCR-005.
  - [ ] Implement BBCR-007.
  - [ ] Implement BBCR-011.
- [ ] **Stage 7 — Bound operations**
  - [ ] Implement BBCR-008.
  - [ ] Implement BBCR-012.
  - [ ] Implement BBCR-014.
- [ ] **Stage 8 — CI and repository controls**
  - [ ] Implement BBCR-016 through BBCR-020.
- [ ] **Stage 9 — Migration and documentation**
  - [ ] Implement BBCR-021 through BBCR-023.
- [ ] **Stage 10 — Final adversarial validation and signoff**
  - [ ] Run the full validation matrix below on the exact final SHA.
  - [ ] Complete the evidence record.

---

# 5. Required Validation Matrix

## Rust formatting, lint, and tests

- [ ] `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml --all-features`
- [ ] Security-focused integration tests pass.
- [ ] Platform-specific Rust tests pass on Linux, Windows, and macOS.

## Frontend validation

- [ ] `source ./fix-node-version.sh && pnpm lint`
- [ ] `source ./fix-node-version.sh && pnpm test:ui`
- [ ] `source ./fix-node-version.sh && pnpm build`
- [ ] Production Redux configuration test passes.
- [ ] CSP assertion test passes.
- [ ] API-key draft non-retention tests pass.

## Security and supply-chain validation

- [ ] Full-history secret scan passes or all findings are documented and remediated.
- [ ] Rust advisory audit passes.
- [ ] Rust source/license policy passes.
- [ ] JavaScript dependency audit passes under the documented threshold.
- [ ] SAST/CodeQL passes or all findings are explicitly resolved.
- [ ] Model-manifest hash verification tests pass.
- [ ] GitHub Actions are pinned by full commit SHA.

## Adversarial functional scenarios

- [ ] Misdeclared-intent submit plan is rejected.
- [ ] Ready-status protected action is rejected.
- [ ] Misleading planner confirmation text cannot change deterministic summary.
- [ ] Stale confirmation cannot execute.
- [ ] Password/hidden/token form values are absent from planner payload.
- [ ] Prompt-injected page cannot bypass policy.
- [ ] Changed endpoint cannot receive stored credentials.
- [ ] Cross-origin redirect cannot receive authorization headers.
- [ ] Traversal image ID cannot read outside screenshot cache.
- [ ] Modified model download cannot activate.
- [ ] Planner timeout cancels boundedly.
- [ ] Oversized page/response/screenshot/download is rejected within limits.
- [ ] Concurrent config writes do not corrupt state.

## Packaged application validation

- [ ] Linux package builds and launches.
- [ ] Windows package builds and launches.
- [ ] macOS package builds and launches.
- [ ] Packaged CSP and Tauri permissions match policy.
- [ ] Keyring-backed credential save/test works on supported platforms without exposing the key.
- [ ] Browser, planner, TTS, ASR, OCR, and confirmation smoke flows pass.

---

# 6. Final Signoff Requirements

Do not mark this TODO complete until all of the following are true:

- [ ] Every P0 task is complete with regression evidence.
- [ ] Every P1 task is complete or has a documented, explicitly accepted residual risk.
- [ ] P2 CI gates are mandatory on the protected branch.
- [ ] P3 documentation and migration procedures match the final implementation.
- [ ] No real API keys or credentials are present in the current tree or reachable history.
- [ ] Any historical secret finding has been revoked and rotated.
- [ ] All required CI checks are green on the exact signoff SHA.
- [ ] The exact signoff SHA has not changed after validation.
- [ ] No temporary diagnostic workflows, test secrets, bypass flags, or weakened policies remain.
- [ ] A final manual review confirms that deterministic code, not planner behavior, owns all safety decisions.

---

# 7. Evidence Record

Fill this section during implementation. Do not use approximate claims.

## Implementation commits

- P0 safety policy commit(s):
- Confirmation binding commit(s):
- Page redaction/prompt-injection commit(s):
- Credential/endpoint hardening commit(s):
- Filesystem/model/persistence hardening commit(s):
- Resource/CSP/frontend-secret commit(s):
- CI/security scanning commit(s):
- Documentation/migration commit(s):

## Final signoff

- Final signoff commit SHA:
- Branch:
- Pull request:
- Review approval:

## CI evidence

- Linux workflow run:
- Windows workflow run:
- macOS workflow run:
- Secret-scan run:
- Dependency/security scan run:
- Packaged smoke-test run:

## Local validation evidence

```text
cargo fmt:
cargo clippy:
cargo test:
pnpm lint:
pnpm test:ui:
pnpm build:
secret scan:
cargo audit/deny:
JavaScript audit:
additional adversarial tests:
```

## Historical secret scan result

- Scanner and version:
- Scope scanned:
- Findings:
- Revocations/rotations performed:
- Narrow fixture allowlist entries:

## Residual risks

- None, or list each explicitly accepted residual risk with owner, rationale, mitigation, and review date.
