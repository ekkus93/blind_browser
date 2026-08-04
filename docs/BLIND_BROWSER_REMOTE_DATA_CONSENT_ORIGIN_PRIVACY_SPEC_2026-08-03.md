# Blind Browser Remote Data Consent and Origin Privacy Spec

**Date:** 2026-08-03  
**Repository:** `ekkus93/blind_browser`  
**Branch:** `master`  
**Reviewed baseline:** `0c0acb0d76210afc6fe40a0ebd32f50e89897d91`  
**Companion TODO:** `docs/BLIND_BROWSER_REMOTE_DATA_CONSENT_ORIGIN_PRIVACY_TODO_2026-08-03.md`  
**Parent remediation items:** BBCR-003, BBCR-006, BBCR-008, BBCR-015, and BBCR-021  
**Scope:** Deterministic user consent and per-origin policy for data sent to non-loopback remote planner endpoints.  
**Release boundary:** This is a focused privacy-boundary milestone. Completing it does not by itself complete the broader BBCR remediation program or establish production release readiness.

---

## 1. Purpose

Blind Browser already has a meaningful remote-planner privacy boundary:

- planner-visible page data uses a typed sanitized representation;
- password, hidden, token-bearing, payment, identity, and other sensitive fields are redacted or omitted;
- page regions and interactive elements are locally relevance-ranked and bounded;
- loopback planner endpoints are distinguished from network endpoints;
- non-loopback planning requires a global consent boolean;
- local-only mode blocks non-loopback planning;
- manually blocked origins remain local;
- authentication, payment, identity, health, wallet, and administrative contexts are blocked from network planning;
- destination-bound credential controls and deterministic action safety remain authoritative.

The remaining product and architecture gap is not the absence of all privacy controls. The gap is that the current controls are too coarse and too detached from the moment data would leave the device:

1. Consent is represented primarily as a global boolean in settings.
2. The user must manually type blocked origins into a textarea.
3. There is no first-class just-in-time consent challenge for the current page and destination.
4. There are no allow-once or allow-for-session decisions.
5. Persisted allows are not modeled as destination-bound grants.
6. The UI does not expose a single typed current-origin decision such as local, ask, allowed, or blocked.
7. A privacy refusal currently appears as an error rather than a resumable deterministic consent outcome.
8. There is no immutable challenge binding the page origin, planner destination, runtime state, disclosure classes, and sanitized payload digest.
9. Existing global settings do not provide a structured migration path to a stronger origin-policy model.

This specification closes those gaps without weakening the existing sanitization, high-risk blocking, credential scoping, confirmation, or runtime-state protections.

---

## 2. Goals

The implementation must:

1. Make the effective remote-data decision visible for the current page before network planning occurs.
2. Require a deterministic policy decision before any non-loopback planner request containing user or page-derived data is sent.
3. Support local-only, ask-per-origin, and explicit broad sanitized-network modes.
4. Support allow-once, allow-for-session, persistent allow, and persistent block decisions.
5. Bind every allow decision to the current page origin, exact normalized planner endpoint scope, and privacy-policy version.
6. Make persistent blocks apply across all non-loopback planner destinations.
7. Keep high-risk-origin blocking non-overridable in the first version.
8. Resume an exact pending sanitized request after consent without serializing raw page data or raw transcripts into persistent state.
9. Invalidate pending consent when relevant runtime state or destination identity changes.
10. Preserve direct deterministic commands and loopback local-planner operation when network planning is denied.
11. Provide accessible, plain-language UI that identifies the destination and categories of data that would be sent.
12. Migrate the existing privacy configuration conservatively and explicitly.
13. Add exhaustive Rust and frontend regression coverage.

---

## 3. Non-goals

This milestone does not:

- replace or weaken planner redaction;
- permit high-risk pages to use non-loopback planning;
- make the remote planner authoritative for any safety or privacy decision;
- grant permission to execute clicks, typing, submissions, downloads, credentials, or external launches;
- implement planner cancellation and bounded response-body work outside what is required to pause before the request;
- redesign TTS or ASR privacy policy;
- implement account synchronization of privacy rules;
- create a durable browsing-history or privacy-event log;
- add arbitrary wildcard domain rules in version 1;
- support path-level consent rules in version 1;
- treat sanitization as anonymization;
- complete all remaining P1, P2, or P3 BBCR work.

---

## 4. Current implementation baseline

The implementation must build on, not duplicate, the following current source boundaries:

- `src-tauri/src/config/types.rs`
  - `RemotePlannerPrivacySettings`
  - `HighRiskOriginPolicy`
- `src-tauri/src/config/validation.rs`
  - blocked-origin normalization and validation
- `src-tauri/src/app_core/planner_redaction.rs`
  - `sanitize_remote_planner_input`
  - `enforce_remote_planner_privacy`
  - relevance selection
  - high-risk detection
- `src-tauri/src/app_core/remote_planner.rs`
  - remote planner request construction and network execution
- `src-tauri/src/app_core/command_dispatch.rs`
  - deterministic direct-command resolution before remote planning
- `src-tauri/src/app_core/replanning.rs`
  - bounded replanning and lock-scoped remote work
- `src-tauri/src/app_core/settings_adapters.rs`
  - planner privacy settings and notices
- `src-tauri/src/command_handlers/safety_handlers.rs`
  - persisted privacy changes
- `src-tauri/src/commands/contracts/planner.rs`
  - `ExecutionOutcome`
- `src-tauri/src/commands/contracts/providers.rs`
  - remote planner settings/status contracts
- `src/settings-panels/planner.tsx`
  - current global privacy controls
- `src/planner-actions.ts`, `src/runtime-refresh.ts`, `src/panel-state.ts`, and `src/panel-types.ts`
  - frontend state and command wiring

The existing global consent, local-only flag, blocked-origin list, sanitization, relevance filtering, loopback detection, and high-risk block remain valid inputs to the migration and implementation plan.

---

## 5. Terms

### 5.1 Page origin

A page origin is the normalized tuple of scheme, hostname, and effective port produced by URL origin serialization. Only `http` and `https` page origins are eligible for persistent rules.

Examples:

- `https://example.com`
- `https://example.com:8443`
- `http://localhost:3000`

Paths, query strings, fragments, userinfo, Unicode display aliases, and raw page URLs are not part of the consent key.

### 5.2 Planner destination scope

A planner destination scope is the existing normalized `ProviderEndpointScope`, including scheme, hostname, effective port, and approved API path prefix. It must never contain username, password, query, or fragment data.

### 5.3 Network planner

A network planner is any configured planner endpoint for which `ProviderEndpointScope::is_loopback()` is false.

### 5.4 Loopback local planner

A loopback local planner is an endpoint resolved by the existing endpoint policy as loopback. It may receive the sanitized planner payload without network-remote consent because the destination remains on the user’s device. It remains subject to all redaction and deterministic safety controls.

### 5.5 Disclosure classes

The consent challenge must identify the categories that may be included in the sanitized request:

- user transcript;
- current origin and sanitized URL path;
- selected page title and region text;
- selected interactive-element labels and safe attributes;
- OCR-derived regions, when present;
- recent tool-observation summaries;
- relevant skill summaries;
- trusted runtime safety/tool contracts.

The UI must not imply that these categories are anonymous. They are locally selected and sanitized, but they may still contain user or page information.

### 5.6 Persistent origin rule

A persistent origin rule is saved to configuration. A persistent block applies to the page origin for every non-loopback planner destination. A persistent allow is valid only for the exact page origin, destination scope, and privacy-policy version recorded in the rule.

### 5.7 Ephemeral grant

An ephemeral grant is held only in runtime memory. `AllowOnce` is consumed by one exact pending consent challenge. `AllowSession` remains valid for the current application process while its scope remains unchanged.

### 5.8 Consent challenge

A consent challenge is a typed, immutable, expiring request for a user decision. It is generated before network I/O and binds the current page origin, planner destination, disclosure classes, runtime state, privacy-policy version, and sanitized payload digest.

### 5.9 Privacy-policy version

The privacy-policy version changes whenever the meaning or set of planner-visible disclosure classes materially changes. Persistent allows from an older version must not silently authorize a newer disclosure contract.

---

## 6. Security and privacy invariants

The following invariants are mandatory:

1. No non-loopback planner request may begin before deterministic privacy evaluation returns `Allowed`.
2. Loopback detection must use `ProviderEndpointScope`, not hostname string comparison in frontend code.
3. `LocalOnly` overrides every allow rule and ephemeral grant for non-loopback destinations.
4. High-risk blocking overrides every global mode, persistent allow, session grant, and one-shot grant.
5. A persistent block overrides broad global allow mode.
6. A persistent allow must match exact normalized page origin, exact normalized destination scope, and current privacy-policy version.
7. Changing scheme, host, port, or approved destination path invalidates all allows bound to the old destination.
8. A one-shot grant authorizes only the exact pending sanitized payload and runtime-state binding represented by its challenge.
9. Navigation, page-generation change, origin change, relevant safety/config change, endpoint change, privacy-mode change, challenge expiry, or payload-digest mismatch invalidates pending consent.
10. Consent to data transmission never authorizes a side effect or reduces confirmation requirements.
11. Denial, persistence failure, malformed rule data, unknown origin, or evaluation uncertainty fails closed.
12. Persistent state must never contain raw transcripts, page text, OCR text, form values, tool arguments, credentials, or complete planner payloads.
13. Runtime status and logs may expose normalized origins, sanitized endpoint displays, counts, reason codes, and decision classes, but not raw private content.
14. A privacy challenge must be generated without performing planner network I/O.
15. Direct deterministic commands must remain available whenever they do not require remote planning.
16. A loopback planner remains subject to redaction, prompt-injection separation, resource limits, and action-policy validation.
17. Frontend state must not become the privacy authority. Rust evaluates the effective policy from authoritative runtime and persisted state.
18. Frontend retries must not bypass challenge binding or create duplicate network requests.

---

## 7. Selected policy model

### 7.1 Global network mode

Replace the two interacting booleans with one authoritative enum:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RemotePlannerNetworkMode {
    LocalOnly,
    AskPerOrigin,
    AllowSanitizedNonHighRisk,
}
```

Semantics:

- `LocalOnly`
  - permit loopback planner endpoints;
  - block all non-loopback planner requests;
  - do not show a consent challenge that could override the block.
- `AskPerOrigin`
  - require an exact allow rule or ephemeral grant;
  - otherwise return a consent challenge;
  - this is the default for new installations.
- `AllowSanitizedNonHighRisk`
  - permit non-loopback planner requests after sanitization unless a persistent block or high-risk rule applies;
  - this is an explicit advanced choice, not the new-install default.

### 7.2 High-risk policy

Version 1 retains one policy:

```rust
pub enum HighRiskOriginPolicy {
    Block,
}
```

There is no user override in this milestone. The UI may explain why the page remains local, but must not present an “allow anyway” control.

### 7.3 Persistent rules

```rust
pub const REMOTE_DATA_POLICY_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PersistedOriginDecision {
    Allow,
    Block,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RemotePlannerOriginRule {
    pub page_origin: String,
    pub decision: PersistedOriginDecision,
    pub endpoint_scope: Option<String>,
    pub policy_version: u32,
    pub created_at_ms: u64,
}
```

Validation rules:

- `Block` requires `endpoint_scope = None` and applies across all non-loopback planner endpoints.
- `Allow` requires a valid normalized `endpoint_scope` and current `policy_version`.
- duplicate rules are rejected or deterministically deduplicated according to exact key semantics;
- at most 256 persistent rules are allowed;
- invalid or stale allow rules remain visible as inactive/stale rules but do not authorize transmission;
- the persisted representation must be deterministic and sorted.

### 7.4 Runtime-only grants

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EphemeralConsentKind {
    Once,
    Session,
}

pub struct RemotePlannerEphemeralGrant {
    pub page_origin: String,
    pub endpoint_scope: String,
    pub policy_version: u32,
    pub kind: EphemeralConsentKind,
    pub challenge_digest: Option<String>,
    pub expires_at_ms: u64,
    pub remaining_uses: Option<u8>,
}
```

Rules:

- one-shot grants have `remaining_uses = Some(1)` and a matching challenge digest;
- session grants have `remaining_uses = None` and clear on application exit;
- all ephemeral grants clear when global mode becomes `LocalOnly`;
- grants for a destination become unusable immediately when the configured destination changes;
- expired or mismatched grants are removed before evaluation;
- runtime grants are not serialized into `AppState`, config, logs, Redux persistence, or crash reports.

---

## 8. Policy evaluation precedence

Rust must evaluate privacy in this exact order:

1. Parse and validate the configured planner destination.
   - Invalid or missing destination produces the existing typed capability error.
2. If the destination is loopback, return `LoopbackLocalService`.
3. If global mode is `LocalOnly`, block with `remote_data_local_only`.
4. Derive a normalized current page origin.
   - Missing, opaque, non-HTTP(S), credential-bearing, or malformed origins block network page-context planning with `remote_data_opaque_origin_blocked`.
5. Evaluate high-risk context.
   - Any high-risk reason blocks with `remote_data_high_risk_blocked`.
6. Apply a persistent origin-wide block.
   - Block with `remote_data_origin_blocked`.
7. Apply an exact valid one-shot grant.
   - Permit the exact pending request and consume the grant atomically.
8. Apply a valid session grant matching origin, endpoint scope, and policy version.
9. Apply a valid persistent allow matching origin, endpoint scope, and policy version.
10. If global mode is `AllowSanitizedNonHighRisk`, permit the request.
11. Otherwise return a typed consent challenge.

No frontend condition may skip or reorder this evaluation.

---

## 9. Consent challenge and pending request

### 9.1 Execution outcome

Privacy refusal that can be resolved by user consent must not be represented as a generic terminal error. Add a first-class outcome:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub enum ExecutionOutcome {
    Complete { trace: ExecutionTrace },
    AwaitingConfirmation { /* existing fields */ },
    NeedsRemoteDataConsent {
        trace: ExecutionTrace,
        challenge: RemotePlannerConsentChallenge,
    },
    NeedsReplan { trace: ExecutionTrace },
    Aborted { trace: ExecutionTrace, error: ToolError },
}
```

`NeedsRemoteDataConsent` means:

- no planner network request has occurred;
- direct command resolution did not match;
- a sanitized remote request has been prepared and bound to the challenge;
- the request may be resumed only through the consent-response path.

### 9.2 Challenge contract

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct RemotePlannerConsentChallenge {
    pub challenge_id: String,
    pub challenge_digest: String,
    pub request_id: String,
    pub page_origin: String,
    pub endpoint_display: String,
    pub endpoint_scope: String,
    pub profile_name: String,
    pub model_label: String,
    pub policy_version: u32,
    pub disclosure_classes: Vec<RemotePlannerDisclosureClass>,
    pub disclosure_counts: RemotePlannerDisclosureCounts,
    pub expires_at_ms: u64,
    pub allow_once_available: bool,
    pub allow_session_available: bool,
    pub allow_persistent_available: bool,
    pub block_persistent_available: bool,
}
```

The challenge must not contain raw page text, transcript text, OCR text, tool observations, skill descriptions, form values, credentials, or raw provider responses.

`disclosure_counts` may include only bounded counts and byte estimates such as selected region count, selected element count, OCR-region count, tool-history count, skill-summary count, and serialized sanitized payload bytes.

### 9.3 Pending request representation

Introduce a runtime-only pending object owned by `AppCore`, not serializable application state:

```rust
pub(crate) struct PendingRemotePlannerConsent {
    pub challenge: RemotePlannerConsentChallenge,
    pub runtime_state_token: String,
    pub sanitized_payload_digest: String,
    pub sanitized_input: RemotePlannerInput,
    pub profile_name: String,
    pub profile: RemotePlannerProfile,
    pub available_tools: Vec<AvailableTool>,
    pub active_skill_names: Vec<String>,
}
```

Requirements:

- at most one pending remote consent request exists per `AppCore` instance;
- creating a new challenge cancels and zeroizes/drops the previous pending request;
- the pending object is omitted from `AppState` serialization and public debug output;
- only sanitized input is retained, not the unrestricted `PlannerInput`;
- challenge lifetime is bounded, initially 120 seconds;
- the challenge digest covers request ID, page origin, endpoint scope, policy version, disclosure classes/counts, sanitized payload digest, and runtime-state token digest;
- the endpoint display is reconstructed from the normalized endpoint scope and cannot contain userinfo, query, or fragment data.

### 9.4 Consent response command

Add a direct Tauri command such as:

```rust
#[tauri::command]
pub async fn submit_remote_planner_consent_response(
    request_id: String,
    timeout_ms: Option<u64>,
    challenge_id: String,
    challenge_digest: String,
    decision: RemotePlannerConsentDecision,
    app_core: State<'_, Arc<Mutex<AppCore>>>,
) -> Result<ExecutionOutcome, ToolError>;
```

Decision values:

```rust
pub enum RemotePlannerConsentDecision {
    AllowOnce,
    AllowSession,
    AllowPersistent,
    BlockPersistent,
    Deny,
}
```

The command must:

1. validate challenge ID and digest;
2. reject replay or duplicate responses;
3. verify expiry;
4. revalidate runtime state, page origin, planner destination, privacy mode, and policy version;
5. persist a rule before resuming when the decision is persistent;
6. fail closed if persistence fails;
7. install the matching ephemeral grant when needed;
8. consume and remove the pending challenge atomically;
9. release the `AppCore` mutex before network I/O;
10. send exactly the stored sanitized payload;
11. validate and execute returned planner output through the existing deterministic policy and runtime-state machinery.

`Deny` clears the pending request and returns a stable local-only user outcome without network I/O.

---

## 10. Remote planner integration

Refactor the current remote-planner boundary into explicit stages:

1. `prepare_remote_planner_request`
   - validate destination;
   - sanitize the `PlannerInput`;
   - calculate disclosure classes/counts;
   - calculate sanitized payload digest;
   - evaluate privacy;
   - return either an allowed prepared request or a consent challenge with pending sanitized request.
2. `send_prepared_remote_planner_request`
   - accept only a prepared request that already carries a deterministic authorization result;
   - perform network I/O with the existing endpoint-bound credentials, timeout, and redirect controls;
   - parse and validate the planner output.
3. `execute_remote_planner_result`
   - preserve existing planner safety validation, runtime-state token validation, confirmation handling, and bounded replanning.

The network sender must not accept raw `PlannerInput` directly after this refactor. This makes it structurally difficult to bypass privacy evaluation.

Suggested typed boundary:

```rust
pub(crate) struct PreparedRemotePlannerRequest {
    pub sanitized_input: RemotePlannerInput,
    pub authorization: RemotePlannerDataAuthorization,
    pub endpoint_scope: ProviderEndpointScope,
    pub profile_name: String,
    pub profile: RemotePlannerProfile,
}

pub(crate) enum RemotePlannerDataAuthorization {
    Loopback,
    GlobalAllow,
    PersistentAllow,
    SessionAllow,
    OneShotAllow { challenge_digest: String },
}
```

The sender must reject construction of `PreparedRemotePlannerRequest` without an authorization variant.

---

## 11. Origin and high-risk classification

### 11.1 Origin derivation

Use one shared Rust helper to derive the current page origin from authoritative page state. The helper must:

- prefer the current authoritative page URL;
- require `http` or `https`;
- reject userinfo, malformed authority, opaque origin, and `null` origin;
- use URL origin serialization;
- return a typed absence/block reason;
- never accept an origin supplied only by the frontend.

### 11.2 High-risk classification

Retain the current deterministic high-risk block and expose bounded reason codes:

- `authentication_context`;
- `payment_context`;
- `identity_context`;
- `health_context`;
- `wallet_context`;
- `administrative_context`;
- `sensitive_field_context`;
- `high_risk_prompt_injection_context` where existing policy requires it.

The public status/challenge must expose only reason codes and safe category labels, not the matching page text or secret field names.

High-risk classification remains conservative. False positives reduce capability and direct the user to local/direct operation; they must never authorize network transmission.

---

## 12. Runtime status and settings contracts

Extend remote planner status with a first-class privacy view:

```rust
pub struct RemotePlannerPrivacyStatus {
    pub network_mode: RemotePlannerNetworkMode,
    pub endpoint_is_loopback: Option<bool>,
    pub current_page_origin: Option<String>,
    pub effective_decision: RemotePlannerEffectiveDecision,
    pub reason_code: Option<String>,
    pub persistent_rule: Option<PersistedOriginDecision>,
    pub session_grant_active: bool,
    pub pending_challenge: Option<RemotePlannerConsentChallengeSummary>,
    pub policy_version: u32,
    pub persistent_rule_count: usize,
    pub stale_allow_rule_count: usize,
}
```

Effective decision values:

- `LoopbackLocal`;
- `LocalOnly`;
- `HighRiskBlocked`;
- `OriginBlocked`;
- `AllowedGlobal`;
- `AllowedPersistent`;
- `AllowedSession`;
- `ConsentRequired`;
- `OriginUnavailable`;
- `PlannerUnavailable`.

The frontend must render this typed decision rather than infer privacy state from multiple booleans and free-form notices.

The settings API must support:

- changing global network mode;
- listing normalized persistent rules;
- adding/updating a persistent block for the current origin;
- revoking a persistent rule;
- clearing all session grants;
- clearing all persistent allows;
- retaining origin blocks during “clear allows” operations;
- reporting stale allows after endpoint or policy-version changes.

---

## 13. Frontend user experience

### 13.1 Always-visible current status

The assistant/status surface must show one compact privacy state whenever a planner is configured:

- `On-device planner`;
- `Local-only mode`;
- `Remote data: ask for this site`;
- `Remote data allowed for this request`;
- `Remote data allowed for this session`;
- `Remote data always allowed for this site and destination`;
- `This site stays local`;
- `High-risk page: network planner blocked`.

The status must be screen-reader accessible and must not rely on color alone.

### 13.2 Just-in-time consent dialog

When `NeedsRemoteDataConsent` is returned, display an accessible modal or focused panel with:

- normalized page origin;
- sanitized planner endpoint display;
- provider/profile and model label;
- explicit disclosure categories;
- bounded counts, not content previews;
- statement that content is sanitized but may still contain page or user information;
- statement that action confirmation remains separate;
- expiry handling;
- keyboard and screen-reader support.

Buttons, in recommended order:

1. `Allow this request`
2. `Allow for this session`
3. `Always allow for this site`
4. `Keep this site local`
5. `Cancel`

No allow button may receive implicit default activation from opening the dialog. Escape/cancel denies and clears the pending request.

### 13.3 High-risk block UI

For high-risk contexts, show a non-overridable block explanation with:

- safe category label;
- current origin;
- `Use a local planner` guidance;
- `Continue with direct commands` guidance;
- no network-allow control.

### 13.4 Settings redesign

Replace the manual blocked-origin textarea as the primary interface with:

- global mode selector;
- current-origin card;
- `Keep current site local` action;
- persistent-rule table with origin, decision, destination display for allows, status, and revoke action;
- stale-allow section;
- clear-session-grants action;
- clear-persistent-allows action;
- advanced disclosure explanation.

A manual normalized-origin entry may remain only as an advanced accessible fallback, with backend validation authoritative.

### 13.5 Voice and retry behavior

The voice command path must surface the same challenge and resume the exact pending request after a consent decision. It must not require the user to repeat a transcript solely because consent was requested, unless the challenge expires or state changes.

The UI must disable duplicate consent submissions while the response is being processed.

---

## 14. Migration

Migrate the existing fields deterministically:

| Existing state | New state |
|---|---|
| `local_only = true` | `network_mode = local_only` |
| `local_only = false`, `consent_to_remote_page_data = false` | `network_mode = ask_per_origin` |
| `local_only = false`, `consent_to_remote_page_data = true` | `network_mode = allow_sanitized_non_high_risk` |
| each `blocked_origins` entry | persistent origin-wide `Block` rule |
| `high_risk_origin_policy = block` | unchanged |

Migration requirements:

- migration is idempotent;
- malformed legacy origins fail configuration validation rather than being silently ignored;
- existing broad consent is preserved as an explicit broad mode, not silently expanded;
- new installations default to `AskPerOrigin`;
- the UI shows a one-time migration notice explaining the new per-origin controls;
- no legacy global consent is converted into destination-bound persistent allow rules;
- old fields remain readable for one schema migration boundary and are removed only after migration tests exist;
- config examples and documentation must show the new schema;
- rollback behavior must not write a partially migrated policy.

---

## 15. Stable reason and error codes

Use stable codes for automation, UI, and tests:

- `remote_data_consent_required`;
- `remote_data_local_only`;
- `remote_data_high_risk_blocked`;
- `remote_data_origin_blocked`;
- `remote_data_opaque_origin_blocked`;
- `remote_data_consent_expired`;
- `remote_data_consent_not_found`;
- `remote_data_consent_digest_mismatch`;
- `remote_data_consent_state_changed`;
- `remote_data_consent_destination_changed`;
- `remote_data_consent_policy_changed`;
- `remote_data_consent_replayed`;
- `remote_data_consent_persist_failed`;
- `remote_data_rule_invalid`;
- `remote_data_rule_limit_exceeded`;
- `remote_data_prepared_request_unauthorized`.

Errors and status details may include normalized origin, sanitized endpoint display, policy version, decision class, expiry, and bounded reason codes. They must not include raw payloads or credentials.

---

## 16. Testing requirements

### 16.1 Policy evaluator tests

Test every precedence edge:

- loopback bypasses network consent but still sanitizes;
- local-only blocks non-loopback despite all allow types;
- high-risk block overrides global and per-origin allows;
- persistent block overrides broad global allow;
- exact destination-bound persistent allow succeeds;
- changed scheme, host, port, or path invalidates persistent allow;
- policy-version mismatch invalidates persistent allow;
- session grants clear or become unusable after destination/mode change;
- one-shot grant is consumed exactly once;
- unknown or opaque origin fails closed;
- ask mode returns a challenge without network I/O.

### 16.2 Challenge lifecycle tests

- challenge contains no raw transcript/page/OCR/tool/skill content;
- challenge digest changes when origin, destination, policy version, disclosure classes, payload digest, or runtime state changes;
- expired challenge fails;
- wrong ID or digest fails;
- duplicate response fails;
- navigation/page-generation/origin change fails;
- endpoint/profile destination change fails;
- persistence failure does not resume network planning;
- denial performs no network I/O;
- allow-once sends exactly one prepared payload;
- allow-session resumes and applies only to matching scope;
- persistent allow is durable and destination-bound;
- pending request is absent from serialized state and debug output;
- replacing a pending challenge drops the old one.

### 16.3 Remote-planner integration tests

Use a local HTTP test server and request counter to prove:

- no request occurs before consent;
- one request occurs after valid consent;
- no duplicate request occurs after double click/retry;
- direct commands do not create consent challenges;
- loopback requests operate without network-remote consent;
- high-risk pages never reach the server;
- blocked origins never reach the server;
- returned planner output still passes deterministic action validation;
- stale state after consent triggers replan/abort rather than execution.

### 16.4 Migration tests

- every legacy boolean combination maps correctly;
- blocked origins become origin-wide blocks;
- migration is idempotent;
- malformed legacy data fails closed;
- new-install default is ask-per-origin;
- legacy broad consent does not create destination-bound allows;
- config serialization is deterministic.

### 16.5 Frontend tests

- typed effective status renders correctly;
- current origin and endpoint are displayed safely;
- all disclosure categories are visible;
- no raw content preview is rendered;
- allow buttons have no implicit default activation;
- escape/cancel denies;
- high-risk UI has no override;
- duplicate submissions are disabled;
- focus trapping and return focus work;
- screen-reader labels identify decision duration;
- persistent rule list supports revoke;
- stale allows are visibly inactive;
- session grants clear in UI after backend status refresh;
- voice flow resumes without transcript repetition when valid.

### 16.6 Privacy scanner tests

Extend sensitive-diagnostic/frontend-state scanners to ensure:

- pending sanitized payloads are not serialized;
- raw consent payloads are not stored in Redux;
- logs do not include challenge payload contents;
- endpoint displays contain no userinfo/query/fragment;
- origin rules contain only normalized origins and destination scopes.

---

## 17. CI and validation

Permanent CI for the final implementation SHA must run:

```text
python3 scripts/check-silent-fallbacks.py
python3 scripts/check-security-fallbacks.py
python3 scripts/check-security-fallback-inventory.py
python3 scripts/check-sensitive-diagnostics.py
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-features
pnpm lint
pnpm test:ui
pnpm build
```

Add a focused integration target for remote-data policy and consent lifecycle so the critical gate is visible separately from the full Rust suite.

No temporary workflow, patch generator, consent bypass, broad test-only allow, or sensitive fixture may remain in the final tree.

---

## 18. Documentation requirements

Update or create:

- `docs/SPECS.md` privacy architecture section;
- remote planner setup documentation;
- privacy disclosure documentation;
- configuration example and migration notes;
- comprehensive BBCR TODO reconciliation for BBCR-003;
- implementation report with exact source/test/CI evidence;
- threat-model coverage for malicious pages, compromised remote planners, stale consent, destination changes, and privacy-prompt spoofing.

Documentation must state:

- what data categories can leave the device;
- what sanitization does and does not guarantee;
- how loopback differs from network remote;
- how global mode and per-origin rules interact;
- why persistent allows are destination-bound;
- why high-risk pages cannot be overridden;
- how to revoke allows and clear session grants.

---

## 19. Recommended implementation sequence

1. Add new config types, normalization, migration, and unit tests.
2. Add the pure policy evaluator and precedence tests.
3. Split remote request preparation from network sending.
4. Add prepared-request authorization types.
5. Add challenge and runtime-only pending consent state.
6. Add consent-response command and exact resume flow.
7. Integrate voice/replanning/runtime-state invalidation.
8. Extend runtime/settings contracts and TypeScript types.
9. Implement current-origin status and just-in-time consent UI.
10. Replace the primary manual blocked-origin textarea with structured rule management.
11. Add Rust integration, migration, frontend, accessibility, and scanner tests.
12. Update documentation and the comprehensive remediation record.
13. Run permanent CI on the exact cleaned implementation/documentation SHA.

---

## 20. Acceptance criteria

This milestone is complete only when all of the following are true:

1. No non-loopback planner request can occur without a deterministic allow result.
2. Ask-per-origin is the default for new installations.
3. The current page’s effective decision is visible and typed.
4. Just-in-time consent supports once, session, persistent allow, persistent block, and deny.
5. Persistent allows are bound to exact page origin, destination scope, and policy version.
6. Persistent blocks apply across all network destinations.
7. High-risk contexts remain non-overridable and never reach a network planner.
8. Pending consent binds an exact sanitized payload and relevant runtime state.
9. No raw private content is persisted or exposed by challenge/status contracts.
10. Consent responses are expiring, replay-resistant, and state-bound.
11. Persistence failure cannot create an effective allow.
12. Direct commands and loopback local planning continue to work.
13. Migration is tested and fail-closed.
14. Rust, frontend, accessibility, privacy-scanner, and integration tests pass.
15. Permanent CI is green on the exact final SHA.
16. The authoritative TODO is fully reconciled with exact evidence.
17. The broader BBCR program remains explicitly open unless separately completed.
