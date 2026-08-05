# Blind Browser Remote Data Consent and Origin Privacy — Stage 2B Settings Addendum

**Date:** 2026-08-05  
**Repository:** `ekkus93/blind_browser`  
**Branch:** `master`  
**Scope:** Typed frontend privacy settings and structured site-rule management  
**Source implementation SHA:** `d5a06f9ef23bcd2dc3c4e8b4851163024a254f8b`  
**Source permanent CI:** run `31006899810`, job `92308939648`, conclusion `success`

## Bounded conclusion

The Stage 2B planner-privacy settings and rule-management surface is implemented and validated. The UI now consumes the authoritative typed `RemotePlannerPrivacyStatus` and dispatches only typed `RemotePlannerPrivacyOperation` values through the existing fail-closed controller. It does not optimistically mutate privacy policy or persistent rules; every successful operation replaces frontend state with the status returned by Rust.

This addendum closes the settings/rule-management subsection of Stage 2B. It does not, by itself, close the complete remote-data-consent milestone, the complete privacy TODO, or the broader BBCR program.

## Implemented settings behavior

### One authoritative network-mode selector

The legacy two-checkbox privacy form is no longer rendered. The settings panel presents one mutually exclusive mode selector:

- `Local only`
- `Ask for each site`
- `Allow sanitized network planning for non-high-risk sites`

Selecting broad sanitized-network mode requires a focused confirmation dialog. Cancel receives initial focus, Escape cancels, focus is trapped while the dialog is open, and focus returns to the invoking control after dismissal. The dialog explains that high-risk blocking and saved site blocks continue to override broad mode.

### Separate loopback presentation

The panel presents loopback/on-device planner behavior separately from non-loopback privacy policy. For a loopback planner, the UI states that context remains on the device and does not offer persistent remote-data allow controls.

Invalid or unavailable planner destinations remain visibly blocked rather than falling back to an allow state.

### Current-site card

For a supported normalized HTTP(S) origin, the panel displays:

- the normalized site origin;
- the effective privacy decision;
- active session-permission status;
- the exact sanitized destination display for an active saved allow;
- stale/inactive status where applicable.

The card offers only operations allowed by the backend contract:

- keep the current site local;
- allow the current site for the exact authoritative configured planner destination, when policy permits;
- revoke the exact current-site rule.

Persistent allow is not rendered for:

- loopback destinations;
- local-only mode;
- unsupported/opaque origins;
- current high-risk page contexts;
- origins with a persistent block.

High-risk blocking is presented as non-overridable.

### Structured rule management

The free-form blocked-origin textarea is no longer the primary rule interface. Saved rules are listed with:

- normalized site origin;
- allow or block decision;
- sanitized destination display for allows;
- non-sensitive creation timestamp;
- stale status;
- an exact revoke action.

Stale allows remain visible and are explicitly described as non-authorizing.

The advanced manual-entry fallback accepts only a page-origin string and an allow/block decision. It does not accept an endpoint scope. Rust validates and normalizes the origin and selects the authoritative configured destination for allow creation, preventing frontend destination-scope substitution.

### Clear and revocation operations

The panel exposes:

- clear runtime-only session permissions;
- clear every persistent allow while retaining blocks;
- clear every persistent rule with explicit confirmation;
- exact per-rule revocation;
- current-site rule revocation.

The clear-all UI helper can emit only:

```json
{
  "operation": "clear_all_persistent_rules",
  "confirmed": true
}
```

There is no settings UI path that submits an unconfirmed clear-all request.

### Migration notice

When `migration_notice_pending` is true, the panel explains that legacy consent and blocked-origin fields were converted to the typed mode/rule model. It states that broad legacy consent was not manufactured into destination-bound site allows. A typed acknowledgment operation dismisses the notice authoritatively.

### Errors, busy state, and no-op results

Only one privacy operation may be active at a time. Controls are disabled while an operation is in progress. Duplicate operations are rejected by the controller. Backend errors remain visible, and changed/no-op results are announced through live status regions.

No operation logs or renders raw transcript, page, OCR, tool, skill, challenge digest, request payload, credential, or planner-content data.

## Changed source and test files

- `src/settings-panels/planner-privacy.tsx`
- `src/settings-panels/planner-panel.tsx`
- `src/settings-panels/planner.tsx`
- `src/settings-panels/planner-privacy.test.mjs`
- `src/remote-planner-privacy.css`

## Focused frontend evidence

The settings tests verify:

- legacy privacy checkbox/textarea controls are absent;
- all three network modes are present as one radio group;
- current-origin keep-local and destination-bound allow controls are shown only when permitted;
- high-risk and opaque origins do not render persistent allow controls;
- loopback state is presented as on-device behavior;
- structured allow, block, and stale rules are visible;
- migration acknowledgment and all clear controls are present;
- manual allow operations do not carry frontend endpoint scope;
- revoke operations preserve exact backend rule identity;
- clear-all construction always includes explicit confirmation.

## Permanent CI evidence

Permanent CI run `31006899810`, job `92308939648`, passed on exact source SHA `d5a06f9ef23bcd2dc3c4e8b4851163024a254f8b`:

- silent-fallback scanner;
- reviewed security-fallback scanner;
- exact fallback inventory;
- sensitive-diagnostics scanner;
- Rust formatting;
- default Rust compilation;
- strict all-target/all-feature Clippy;
- focused direct-command semantic evidence;
- complete Rust/Wry test suite;
- frontend lint;
- frontend UI tests;
- production frontend build.

## Compatibility boundary and remaining cleanup

The planner settings renderer retains compatibility-shaped legacy handler properties because `app.tsx` and `planner-actions.ts` still contain the previous boolean/list save adapter. The new settings component never renders or invokes those handlers, so that path is not user-reachable from the Stage 2B UI and is not an authorization fallback.

Removing the obsolete adapter and its compatibility props remains a bounded source-cleanup task. It should be performed without changing the typed Rust operation contract or reintroducing the old two-boolean policy as an authoritative path.

## Remaining milestone work

The following work remains outside this settings addendum:

- broader request-count, replay, concurrency, expiry, invalidation, persistence-failure, and hostile-state evidence required by the full privacy TODO;
- remaining scanner and serialized-state privacy coverage;
- full TODO checkbox reconciliation against exact evidence;
- privacy/threat-model and user-documentation closure;
- BBCR/post-Batch-8 reconciliation;
- final milestone-wide exact-SHA signoff.
