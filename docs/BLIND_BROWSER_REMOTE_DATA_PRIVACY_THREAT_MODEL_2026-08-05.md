# Blind Browser Remote Data Privacy Threat Model

**Date:** 2026-08-05
**Repository:** `ekkus93/blind_browser`
**Scope:** Page-context transmission to remote planners, consent, per-origin policy, runtime grants, frontend privacy state, persistence, and diagnostics
**Implementation evidence SHA:** `0beb531f963297bf0e29c559141b520ba221823c`
**Permanent CI:** run `31070751355`, job `92518011921`, conclusion `success`

## 1. Security objective

Blind Browser must prevent any current first-party page-context planning path from sending data to a non-loopback planner unless deterministic Rust policy authorizes the exact sanitized prepared request. Authorization must be explicit, scope-bound, expiring where appropriate, replay-resistant, fail closed on relevant state changes, separate from protected-action confirmation, and absent from ambient serialized or diagnostic state.

This threat model does not claim that sanitized content is anonymous or that an authorized remote provider is trustworthy. It constrains whether and what Blind Browser sends.

## 2. Protected assets

- raw command transcripts;
- raw and sanitized page text;
- raw form values and hidden fields;
- OCR-derived content;
- tool observations and remote response details;
- skill text and summaries;
- credentials and endpoint-bound secret references;
- the sanitized pending planner input;
- challenge digests and single-use authorization material;
- persistent privacy rules;
- runtime-only grants and pending transactions;
- truthful user-visible policy status; and
- integrity of protected-action confirmation after planning.

## 3. Trust boundaries

### 3.1 Untrusted page and OCR content

DOM text, attributes, URLs, OCR, and page-derived metadata are attacker-controlled. They cannot grant permission, change deterministic policy, or become trusted instructions merely because they appear in page content.

### 3.2 Frontend to Rust command boundary

The React frontend is a presentation and typed-command client. It can request a decision or policy operation but cannot construct an authorized prepared request, choose the endpoint scope for a persistent allow, or make stale metadata sufficient for transmission.

### 3.3 Rust preparation to network sender boundary

Only `PreparedRemotePlannerRequest` crosses into the network sender. Raw `PlannerInput` is sanitized and policy-evaluated before authorization. Preparation performs no network I/O.

### 3.4 Persistent configuration boundary

Persistent rules are durable configuration. Pending consent, sanitized drafts, one-shot grants, and session grants are runtime-only and must not enter configuration or reconstruct after restart.

### 3.5 Remote planner boundary

An authorized non-loopback planner receives the bounded sanitized request. The provider remains outside Blind Browser's trust boundary and can observe transmitted content and network metadata.

## 4. Attacker capabilities and mitigations

### T1 — Malicious page content attempts to trigger or spoof consent

**Attack:** A page displays text that imitates Blind Browser, instructs the model or user to approve transmission, or embeds hidden values intended to enter the planner request.

**Mitigations:**

- consent is produced by application UI from a typed Rust challenge, not page markup;
- page content cannot invoke the Rust authorization constructor;
- disclosure UI contains bounded categories/counts and trusted destination metadata, not page-provided permission text;
- hidden/password/token/payment fields and unrestricted attributes are excluded by the planner-safe representation;
- no allow choice is the implicit default; cancel receives initial focus; and
- high-risk classification remains deterministic and non-overridable.

**Residual risk:** A user can still choose to authorize a request after reading malicious page content. The UI warns that sanitization does not remove all sensitive or identifying information.

### T2 — Prompt injection embedded in page, OCR, tool, or skill content

**Attack:** Untrusted content attempts to override policy, request credentials, or cause protected actions.

**Mitigations:**

- all planner input channels pass through typed sanitization and bounded selection;
- hostile content remains untrusted text in the planner contract;
- deterministic privacy policy runs before network transmission;
- planner output undergoes semantic and action-policy validation; and
- remote-data consent cannot replace protected-action confirmation.

**Residual risk:** An authorized remote model can still be influenced by sanitized hostile content. Deterministic action policy, not model compliance, is the final authority.

### T3 — Compromised, redirected, or changed planner destination

**Attack:** A saved allow for one destination is reused after scheme, host, port, path prefix, profile, or model changes; a redirect attempts to move credentials or content elsewhere.

**Mitigations:**

- endpoint scope is normalized by `ProviderEndpointScope`;
- persistent and session allows bind exact normalized origin, destination, and policy version;
- challenge digest binds destination profile/model metadata;
- response handling revalidates the current configured destination;
- endpoint-bound credential rules remain in force; and
- redirect refusal remains enforced.

**Residual risk:** The approved destination itself can be compromised. Blind Browser cannot prevent an authorized provider from mishandling received data.

### T4 — Stale or replayed challenge response

**Attack:** A response is submitted after page/destination/policy change, after expiry, after replacement, or a second time.

**Mitigations:**

- random challenge ID plus canonical SHA-256 manifest digest;
- digest binds request ID, origin, destination, profile/model, policy version, disclosure summary, payload digest, relevant runtime token, and expiry;
- one pending transaction is authoritative and replacement invalidates the older transaction;
- response handling removes pending state atomically before authorization or terminal completion;
- one-shot authorization uses atomic single-use consumption; and
- missing, mismatched, expired, or changed state produces a typed failure and no request.

### T5 — Concurrent duplicate responses

**Attack:** Double click, repeated keyboard activation, or concurrent commands attempt to obtain two sends.

**Mitigations:**

- UI submission gate accepts one activation before the busy rerender;
- controller rejects duplicate submission from current state;
- all controls are disabled while submitting;
- backend pending state is consumed atomically; and
- process-isolated evidence proves two concurrent allow-once responses produce one authorization and one missing-challenge result, followed by exactly one network request.

### T6 — Persistence failure becomes authorization

**Attack:** Durable rule write fails and the application silently falls back to a session or one-shot allow.

**Mitigations:**

- persistent decision is written before authorization;
- write failure returns `remote_data_consent_persist_failed`;
- no in-memory rule or grant is installed;
- pending state is terminally consumed; and
- frontend clears stale allow controls, reports the error, and refreshes authoritative status.

No weaker authorization fallback is accepted.

### T7 — Frontend state tampering or stale UI metadata

**Attack:** Modified Redux/panel state, a stale challenge, or frontend-supplied endpoint scope attempts to authorize a request.

**Mitigations:**

- frontend challenge state contains bounded public metadata only;
- response must carry the exact live challenge ID/digest;
- Rust owns pending sanitized input and all authorization state;
- persistent allow endpoint scope is selected from authoritative Rust configuration;
- stale metadata cannot reconstruct backend pending state after restart; and
- backend state mismatch consumes/rejects the request and causes authoritative refresh.

### T8 — Privacy-dialog spoofing or hidden override controls

**Attack:** UI accidentally renders an allow control in a high-risk, local-only, loopback, blocked, or opaque-origin state; inaccessible behavior encourages unsafe approval.

**Mitigations:**

- effective decision comes from typed Rust status;
- high-risk status and settings omit visible and hidden allow controls;
- current-site allow helper checks loopback, local-only, high-risk, opaque-origin, and persistent-block state;
- dialog has explicit role, name, description, warning, destination, and scope labels;
- cancel receives focus and Escape denies; and
- automated rendering/interaction tests cover status text, decision order, labels, focus handling, duplicate gating, and control suppression.

**Residual risk:** Automated tests do not certify every screen reader/browser combination. Manual release QA remains required for platform-specific behavior.

### T9 — Accidental diagnostic or serialized-state leakage

**Attack:** Raw or sanitized pending content, challenge material, credentials, or remote response bodies enter logs, error details, configuration, runtime snapshots, frontend actions, CI artifacts, or debug formatting.

**Mitigations:**

- pending drafts and grants are outside serializable `AppState`;
- public privacy status uses a digest-free challenge summary;
- configuration excludes pending state and grants;
- errors and diagnostics are bounded/redacted;
- hostile sentinel tests inspect configuration, state/status snapshots, challenge surfaces, and frontend state; and
- permanent sensitive-diagnostic and remote-planner privacy-state scanners reject prohibited fields, logging, and permissive fallback patterns.

**Residual risk:** Future instrumentation or new state surfaces can create leakage unless added to scanner/test coverage.

### T10 — Restart with stale frontend state

**Attack:** The frontend retains a challenge after backend restart and attempts to resume it.

**Mitigations:**

- pending transaction, one-shot grant, and session grant are runtime-only;
- reconstructed `AppCore` contains none of them;
- stale frontend response receives a missing/mismatch outcome and cannot cause a send; and
- persistent rules survive only through successful durable configuration.

### T11 — Policy downgrade through fallback or malformed data

**Attack:** Missing fields, malformed origins/rules, unsupported versions, refresh failure, or a swallowed error results in broad authorization.

**Mitigations:**

- deterministic evaluator has explicit precedence and no permissive default arm;
- malformed/opaque origin blocks non-loopback planning;
- stale/unsupported allows cannot authorize;
- persistent block remains authoritative in conflicts;
- frontend status projection rejects unsupported decisions rather than guessing;
- refresh failures remain visible; and
- fallback scanners and exact accepted-fallback inventory run in permanent CI.

## 5. Security invariants

1. No current first-party page-context network send accepts raw `PlannerInput`.
2. Every non-loopback send requires an authorized `PreparedRemotePlannerRequest`.
3. Sanitization and disclosure accounting occur before authorization.
4. Rust is the sole transmission-policy authority.
5. High-risk and opaque-origin blocks are non-overridable.
6. Blocks are origin-wide; allows are origin-, destination-, and policy-version-bound.
7. Pending consent is runtime-only, expiring, digest-bound, and single-transaction.
8. Terminal handling consumes pending state before network work.
9. Persistence failure cannot authorize.
10. Transmission consent cannot reduce action confirmation.
11. Ambient serialized and diagnostic state excludes raw/sanitized pending payload content.
12. Unknown, malformed, stale, or unsupported state fails closed.

## 6. Evidence

Primary executable evidence is in:

- `src-tauri/src/app_core/remote_data_consent.rs`;
- `src-tauri/src/app_core/remote_privacy_api.rs`;
- `src-tauri/src/app_core/tests/remote_data_consent_evidence_tests.rs`;
- `src-tauri/src/app_core/tests/remote_privacy_api_tests.rs`;
- `src/remote-planner-consent-dialog-interactions.test.mjs`;
- `src/remote-planner-privacy-controller.test.mjs`;
- `src/remote-planner-privacy-state.test.mjs`;
- `src/remote-planner-privacy-ui.test.mjs`;
- `src/settings-panels/planner-privacy.test.mjs`;
- `scripts/check-remote-planner-privacy-state.py`; and
- `.github/workflows/ci.yml`.

Implementation SHA `0beb531f963297bf0e29c559141b520ba221823c` passed permanent CI run `31070751355`, job `92518011921`.

## 7. Residual and out-of-scope risks

- authorized remote providers can observe and retain transmitted sanitized content;
- a compromised local application process or operating system is outside this boundary;
- broad sanitized-network mode intentionally reduces prompting for non-high-risk sites;
- high-risk classification is deterministic but cannot guarantee detection of every sensitive context;
- manual assistive-technology and packaged-platform QA is not fully automated;
- future input channels/providers/network paths require explicit integration into this boundary; and
- unrelated BBCR items, including CSP, secret-history scanning, dependency/SAST gates, cross-platform packaged CI, fuzzing/mutation, and other release-readiness work, remain open.
