# Remote Planner Privacy

Blind Browser can use either an on-device planner or a planner reached over the network. This document explains when page and command context can leave the device, what choices are available, and what the privacy controls do.

## The core rule

A non-loopback planner request is not sent until Rust has:

1. selected and sanitized the bounded planner input locally;
2. classified the current page and destination;
3. evaluated the current privacy mode, site rules, and runtime grants;
4. produced a prepared request carrying an explicit authorization; and
5. either found an existing authorization or received a valid response to the exact pending consent challenge.

The frontend can display choices and send a typed decision, but it cannot authorize transmission by itself. A loopback destination such as `127.0.0.1` or `localhost` is treated as on-device behavior and is presented separately.

## Information categories that may be included

A consent dialog lists categories and counts, not content previews. Depending on the command, the sanitized request can include:

- the command transcript;
- the normalized page origin and sanitized URL information;
- locally selected page-text regions;
- locally selected element labels and safe attributes;
- selected OCR-derived regions;
- recent tool-result summaries;
- relevant skill summaries; and
- trusted runtime safety and tool contracts.

The dialog also reports bounded counts and the serialized sanitized request size. It does not display the pending sanitized payload, raw form values, challenge digest, internal endpoint scope, or request ID.

Sanitization reduces exposure; it does **not** make the request anonymous. Selected page text or a command can still contain personal or identifying information. The remote provider can process whatever sanitized content is transmitted under that provider's own terms and security boundary.

## Privacy modes

### Local only

All non-loopback planner requests are blocked. Direct deterministic commands and an on-device planner can still be used.

### Ask for each site

This is the new-install default. A network planner request pauses before transmission unless a matching session or persistent authorization already exists.

### Allow sanitized network planning for non-high-risk sites

Sanitized requests may be sent without a per-site prompt when no stronger block applies. Selecting this broad mode requires explicit confirmation. High-risk blocking, opaque-origin blocking, and saved site blocks still take precedence.

## Choices in the consent dialog

### Allow this request

Authorizes only the exact pending sanitized request. The grant is challenge-bound and single-use.

### Allow for this session

Authorizes later matching requests for the same normalized site origin, exact planner destination, and current privacy-policy version during the current application session. Session grants are runtime-only and expire; they do not survive reconstruction or restart.

### Always allow for this site

Creates a persistent allow for the exact normalized site origin, exact configured planner destination, and current privacy-policy version. A different scheme, host, effective port, path prefix, destination profile/model contract, or policy version does not inherit the allow. Such rules remain visible as stale but cannot authorize transmission.

### Keep this site local

Creates a persistent origin-wide block. It applies to every non-loopback planner destination and overrides broad global allow mode.

### Cancel

Denies the pending request. No planner request is sent.

No allow option is the implicit default. Cancel receives initial focus. Escape invokes deny, Tab and Shift+Tab remain within the dialog, focus returns to the invoking control when possible, and all controls are disabled after the first accepted submission.

## Policy precedence

The authoritative evaluator applies this order:

1. loopback/on-device destination;
2. local-only mode;
3. missing or opaque page origin;
4. current high-risk page classification;
5. persistent site block;
6. matching session grant;
7. matching exact persistent allow;
8. broad sanitized-network mode; and
9. explicit consent required.

A lower item cannot override a higher one. Malformed, conflicting, unsupported-version, or stale rules do not authorize transmission.

## High-risk and opaque pages

High-risk contexts are non-overridable for network planning. This includes deterministic detection of contexts such as payment, authentication, identity, healthcare, or other protected data patterns covered by the runtime policy. The status UI offers local/direct alternatives and renders no visible or hidden network-allow control.

Pages without a supported normalized HTTP(S) origin, including opaque or internal contexts, cannot create persistent site rules and cannot use a non-loopback planner through the consent path.

The page is classified again when a consent response is submitted. A page that becomes high-risk after the dialog opens is blocked rather than relying on the earlier classification.

## Challenge expiry and state changes

A consent response is accepted only for the live pending transaction with the exact challenge ID and digest. The challenge binds the request ID, origin, destination, profile/model, policy version, disclosure summary, sanitized payload digest, relevant runtime-state token, and expiry.

The request fails closed if the challenge expires or relevant state changes. Examples include:

- page identity, generation, or normalized origin changes;
- planner scheme, host, effective port, path prefix, profile, or model changes;
- privacy mode, persistent block state, policy version, or relevant safety configuration changes; and
- current page content becoming high-risk.

An unrelated presentation-only state change that does not affect the request contract does not invalidate the challenge.

The backend consumes a terminal or invalid challenge atomically. Duplicate or replayed responses cannot obtain the prepared request twice. If the backend rejects a stale challenge, the frontend removes stale allow controls and refreshes authoritative status. A transport failure is treated differently: because Rust may not have received the command, the bounded challenge remains visible with an explicit retry error.

## Persistence and restart behavior

Persistent rules are written durably before a persistent decision can authorize transmission. A persistence failure cannot fall back to a session or one-shot allow and cannot send the request.

The following are runtime-only and do not survive reconstruction or restart:

- pending consent transactions;
- one-shot grants; and
- session grants.

Persistent rules survive only after a successful durable write. Configuration does not store the pending challenge ID, challenge digest, or sanitized pending input.

## Revoking permissions

Planner privacy settings support:

- revoking an exact saved rule;
- keeping the current site local;
- allowing the current site for the exact authoritative destination when permitted;
- clearing session permissions;
- clearing all persistent allows while retaining blocks; and
- clearing every persistent rule after explicit confirmation.

Manual site entry is an advanced fallback. The frontend submits only the origin and allow/block decision. Rust normalizes the origin and selects the configured authoritative destination for an allow; the frontend cannot supply a replacement endpoint scope.

## Migration from legacy settings

Legacy consent, local-only, and blocked-origin fields are normalized into the typed network mode and structured rules. Migration is conservative:

- legacy local-only remains local-only;
- legacy broad consent maps to broad sanitized-network mode but does not manufacture destination-bound site allows;
- legacy blocked origins become persistent origin-wide blocks; and
- malformed legacy origins fail closed rather than being silently discarded.

The settings panel displays a migration notice until it is acknowledged through the authoritative typed operation.

## Transmission consent is not action confirmation

Remote-data consent answers only whether the bounded sanitized planner request may be transmitted. It does not approve clicks, typing, form submission, downloads, credentials, or any other protected action. Planner output still passes deterministic semantic and action-policy validation, and protected actions still require their own immutable confirmation flow.

## Errors and diagnostics

Privacy failures are explicit and typed. The implementation does not retry under broader authorization, convert persistence failure into a weaker allow, guess an allowed state after refresh failure, or close a stale dialog as though the request completed.

Production status, errors, logs, and ambient frontend/backend state must not contain raw transcript, page, OCR, tool, skill, credential, sanitized pending request, or full planner-payload content. Permanent scanners enforce the reviewed privacy-state and diagnostic boundaries.

## Accessibility validation contract

Automated tests verify the semantic dialog role, title/description relationships, explicit alert/status regions, distinct decision labels, deterministic button order, initial cancel focus behavior, forward and reverse focus wrapping, Escape denial, focus restoration, disconnected-invoker fallback, zero-focusable fallback, duplicate-submission blocking, disabled busy controls, textual high-risk status, and absence of hidden allow controls.

Maintainer release QA should additionally use this manual method with synthetic content only:

1. At 200% browser zoom and a narrow viewport, confirm the status, destination fields, disclosure list, warnings, and every decision remain readable without horizontal clipping; long origins and destinations must wrap.
2. Navigate using only Tab, Shift+Tab, Enter, Space, and Escape. Confirm visible focus, logical order, one accepted activation, and return focus.
3. Enable the operating system's forced-colors/high-contrast mode. Confirm borders, text, focus outlines, disabled state, alerts, and warnings remain distinguishable without relying on custom color.
4. With a screen reader, confirm the dialog name, description, destination terms, information categories, decision scope labels, busy state, errors, and stale-rule status are announced without duplicated content.

The CSS uses system color keywords, `currentColor`, explicit borders, visible focus outlines, wrapping, auto-fit decision columns, and narrow-screen reflow. Automated evidence validates the executable semantics and interaction logic; it is not a certification for every browser, operating system, or assistive-technology combination.

## Scope and limits

This privacy boundary covers current first-party page-context remote-planning paths. It does not make an untrusted remote provider safe, protect against a compromised operating system or application process, or close unrelated Blind Browser security and release-readiness work. New planner inputs, providers, destinations, or network paths must be integrated into the same prepared-request, policy, scanner, test, and documentation boundaries before use.
