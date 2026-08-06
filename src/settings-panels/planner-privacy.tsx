import {
  useEffect,
  useRef,
  useState,
  useSyncExternalStore,
  type KeyboardEvent,
  type ReactNode,
} from "react";

import type {
  PersistedOriginDecision,
  RemotePlannerNetworkMode,
  RemotePlannerOriginRuleStatus,
  RemotePlannerPrivacyOperation,
  RemotePlannerPrivacyOperationResult,
  RemotePlannerPrivacyStatus,
} from "../api/remote-planner-privacy.ts";
import { runRemotePlannerPrivacyOperation } from "../remote-planner-privacy-controller.ts";
import {
  dismissRemotePlannerPrivacyOperationError,
  type RemotePlannerPrivacyState,
} from "../remote-planner-privacy-state.ts";
import { appShellStore } from "../store.ts";
import {
  activatePrivacyConfirmationFocus,
  handlePrivacyConfirmationKeyboard,
} from "./planner-privacy-confirmation-interactions.ts";

const NETWORK_MODE_OPTIONS = [
  {
    value: "local_only",
    label: "Local only",
    description: "Block every non-loopback planner destination. Commands can still use an on-device planner and direct-command handling.",
  },
  {
    value: "ask_per_origin",
    label: "Ask for each site",
    description: "Pause before the first sanitized network-planner request for a site unless a matching saved or session permission already exists.",
  },
  {
    value: "allow_sanitized_non_high_risk",
    label: "Allow sanitized network planning for non-high-risk sites",
    description: "Advanced broad mode. Sanitized context may be sent without a per-site prompt, but high-risk pages and saved site blocks remain fail-closed.",
  },
] as const satisfies readonly {
  value: RemotePlannerNetworkMode;
  label: string;
  description: string;
}[];

const EFFECTIVE_DECISION_LABELS: Record<RemotePlannerPrivacyStatus["effective_decision"], string> = {
  loopback_local: "On-device planner",
  local_only: "Blocked by local-only mode",
  high_risk_blocked: "Blocked because this page is high risk",
  origin_blocked: "Blocked by a saved site rule",
  allowed_global: "Allowed by broad sanitized-network mode",
  allowed_persistent: "Allowed by a saved site-and-destination rule",
  allowed_session: "Allowed for this application session",
  consent_required: "Permission required before network planning",
  origin_unavailable: "This page has no supported HTTP(S) origin",
  planner_unavailable: "Remote planner is unavailable",
};

const OPERATION_LABELS: Record<RemotePlannerPrivacyOperation["operation"], string> = {
  get_status: "refreshing planner privacy status",
  set_network_mode: "changing the planner network mode",
  upsert_origin_rule: "saving a site privacy rule",
  upsert_current_origin_rule: "saving the current-site privacy rule",
  revoke_origin_rule: "revoking a saved site privacy rule",
  clear_session_grants: "clearing session permissions",
  clear_persistent_allows: "clearing saved allow rules",
  clear_all_persistent_rules: "clearing every saved site rule",
  acknowledge_migration_notice: "acknowledging the privacy migration notice",
};

type PrivacyConfirmationKind =
  | "broad-network-mode"
  | "clear-persistent-allows"
  | "clear-all-rules";

export interface RemotePlannerPrivacySettingsHandlers {
  onOperation?: (
    operation: RemotePlannerPrivacyOperation,
  ) => Promise<RemotePlannerPrivacyOperationResult | null>;
  onDismissOperationError?: () => void;
}

export function createNetworkModeOperation(
  networkMode: RemotePlannerNetworkMode,
): RemotePlannerPrivacyOperation {
  return {
    operation: "set_network_mode",
    network_mode: networkMode,
  };
}

export function createManualOriginRuleOperation(
  pageOrigin: string,
  decision: PersistedOriginDecision,
): RemotePlannerPrivacyOperation {
  return {
    operation: "upsert_origin_rule",
    page_origin: pageOrigin.trim(),
    decision,
  };
}

export function createCurrentOriginRuleOperation(
  decision: PersistedOriginDecision,
): RemotePlannerPrivacyOperation {
  return {
    operation: "upsert_current_origin_rule",
    decision,
  };
}

export function createRevokeOriginRuleOperation(
  rule: RemotePlannerOriginRuleStatus,
): RemotePlannerPrivacyOperation {
  return {
    operation: "revoke_origin_rule",
    page_origin: rule.page_origin,
    decision: rule.decision,
    endpoint_scope: rule.endpoint_scope,
  };
}

export function createConfirmedClearAllRulesOperation(): RemotePlannerPrivacyOperation {
  return {
    operation: "clear_all_persistent_rules",
    confirmed: true,
  };
}

export function findCurrentOriginRule(
  status: RemotePlannerPrivacyStatus,
): RemotePlannerOriginRuleStatus | null {
  const pageOrigin = status.current_page_origin;
  const decision = status.persistent_rule;
  if (pageOrigin === null || decision === null) {
    return null;
  }

  if (decision === "block") {
    return status.persistent_rules.find((rule) =>
      rule.page_origin === pageOrigin && rule.decision === "block"
    ) ?? null;
  }

  return status.persistent_rules.find((rule) =>
    rule.page_origin === pageOrigin
      && rule.decision === "allow"
      && rule.endpoint_scope === status.endpoint_scope
  ) ?? null;
}

export function canPersistentlyAllowCurrentOrigin(
  status: RemotePlannerPrivacyStatus,
): boolean {
  return status.current_page_origin !== null
    && status.endpoint_scope !== null
    && status.endpoint_is_loopback === false
    && status.network_mode !== "local_only"
    && status.effective_decision !== "high_risk_blocked"
    && status.persistent_rule !== "block";
}

function safeTimestamp(createdAtMs: number): string {
  const date = new Date(createdAtMs);
  if (!Number.isFinite(date.getTime())) {
    return "Unknown creation time";
  }
  return date.toISOString();
}

function confirmationCopy(kind: PrivacyConfirmationKind): {
  title: string;
  description: string;
  confirmLabel: string;
} {
  switch (kind) {
    case "broad-network-mode":
      return {
        title: "Enable broad sanitized network planning?",
        description: "This permits sanitized context from any non-high-risk site to reach the configured non-loopback planner without a per-site prompt. Saved blocks and high-risk detection still override this mode.",
        confirmLabel: "Enable broad mode",
      };
    case "clear-persistent-allows":
      return {
        title: "Clear every saved allow rule?",
        description: "Saved site blocks remain. Sites that depended on a saved allow will require consent again or remain blocked by the selected network mode.",
        confirmLabel: "Clear saved allows",
      };
    case "clear-all-rules":
      return {
        title: "Clear every saved site privacy rule?",
        description: "This removes both allow and block rules. The global network mode will become the only persistent policy until new rules are created.",
        confirmLabel: "Clear all saved rules",
      };
  }
}

function PrivacySettingsConfirmationDialog(props: {
  kind: PrivacyConfirmationKind;
  busy: boolean;
  onCancel: () => void;
  onConfirm: () => Promise<void>;
}) {
  const rootRef = useRef<HTMLDivElement>(null);
  const cancelRef = useRef<HTMLButtonElement>(null);
  const returnFocusRef = useRef<HTMLElement | null>(null);
  const copy = confirmationCopy(props.kind);

  useEffect(() => {
    returnFocusRef.current = document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null;
    const cancelControl = cancelRef.current;
    if (cancelControl === null) {
      return undefined;
    }
    return activatePrivacyConfirmationFocus(
      returnFocusRef.current,
      cancelControl,
    );
  }, [props.kind]);

  const handleKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const dialogRoot = rootRef.current;
    if (dialogRoot === null) {
      return;
    }
    const focusable = Array.from(
      dialogRoot.querySelectorAll<HTMLElement>(
        "button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex='-1'])",
      ),
    );
    handlePrivacyConfirmationKeyboard({
      event,
      busy: props.busy,
      activeElement: document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null,
      focusableElements: focusable,
      dialogRoot,
      cancel: props.onCancel,
    });
  };

  return (
    <div
      ref={rootRef}
      className="remote-privacy-settings-confirmation"
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="remote-privacy-settings-confirmation-title"
      aria-describedby="remote-privacy-settings-confirmation-description"
      aria-busy={props.busy}
      tabIndex={-1}
      onKeyDown={handleKeyDown}
      data-remote-planner-privacy-confirmation={props.kind}
    >
      <h4 id="remote-privacy-settings-confirmation-title">{copy.title}</h4>
      <p id="remote-privacy-settings-confirmation-description">{copy.description}</p>
      <div className="settings-button-row settings-button-row-wrap">
        <button
          ref={cancelRef}
          type="button"
          className="settings-control-button settings-control-button-secondary"
          disabled={props.busy || undefined}
          onClick={props.onCancel}
        >
          Cancel
        </button>
        <button
          type="button"
          className="settings-control-button settings-control-button-danger"
          disabled={props.busy || undefined}
          onClick={() => { void props.onConfirm(); }}
        >
          {props.busy ? "Applying privacy change…" : copy.confirmLabel}
        </button>
      </div>
    </div>
  );
}

function RemotePlannerPrivacySettingsCard(props: {
  state: RemotePlannerPrivacyState;
  handlers?: RemotePlannerPrivacySettingsHandlers;
}) {
  const [manualOrigin, setManualOrigin] = useState("");
  const [manualDecision, setManualDecision] = useState<PersistedOriginDecision>("block");
  const [confirmation, setConfirmation] = useState<PrivacyConfirmationKind | null>(null);
  const [announcement, setAnnouncement] = useState<string | null>(null);
  const { state, handlers } = props;
  const status = state.status;

  const applyOperation = async (
    operation: RemotePlannerPrivacyOperation,
    changedMessage: string,
    noChangeMessage: string,
  ): Promise<RemotePlannerPrivacyOperationResult | null> => {
    setAnnouncement(null);
    const result = await handlers?.onOperation?.(operation) ?? null;
    if (result !== null) {
      setAnnouncement(result.changed ? changedMessage : noChangeMessage);
    }
    return result;
  };

  const requestModeChange = (networkMode: RemotePlannerNetworkMode) => {
    if (networkMode === "allow_sanitized_non_high_risk") {
      setConfirmation("broad-network-mode");
      return;
    }
    void applyOperation(
      createNetworkModeOperation(networkMode),
      "Planner network mode changed.",
      "Planner network mode was already set to that value.",
    );
  };

  const confirmPendingOperation = async () => {
    let result: RemotePlannerPrivacyOperationResult | null = null;
    switch (confirmation) {
      case "broad-network-mode":
        result = await applyOperation(
          createNetworkModeOperation("allow_sanitized_non_high_risk"),
          "Broad sanitized network-planning mode enabled.",
          "Broad sanitized network-planning mode was already enabled.",
        );
        break;
      case "clear-persistent-allows":
        result = await applyOperation(
          { operation: "clear_persistent_allows" },
          "Every saved allow rule was cleared.",
          "There were no saved allow rules to clear.",
        );
        break;
      case "clear-all-rules":
        result = await applyOperation(
          createConfirmedClearAllRulesOperation(),
          "Every saved site privacy rule was cleared.",
          "There were no saved site privacy rules to clear.",
        );
        break;
      case null:
        return;
    }

    if (result !== null) {
      setConfirmation(null);
    }
  };

  const submitManualRule = async () => {
    const result = await applyOperation(
      createManualOriginRuleOperation(manualOrigin, manualDecision),
      manualDecision === "allow"
        ? "Saved an allow rule for the entered site and the authoritative planner destination."
        : "Saved an origin-wide block for the entered site.",
      "An identical saved rule already existed.",
    );
    if (result !== null) {
      setManualOrigin("");
    }
  };

  if (status === null) {
    return (
      <div className="settings-control-card remote-privacy-settings-card" data-remote-planner-privacy-settings="true">
        <h3>Planner privacy and site permissions</h3>
        <p className="settings-panel-description" role="status" aria-live="polite">
          {state.refreshBusy ? "Loading authoritative planner privacy status…" : "Planner privacy status is unavailable."}
        </p>
        {state.refreshError ? (
          <p className="settings-panel-description settings-panel-warning" role="alert">
            {state.refreshError}
          </p>
        ) : null}
        <button
          type="button"
          className="settings-control-button settings-control-button-secondary"
          disabled={state.operationBusy || undefined}
          onClick={() => {
            void applyOperation(
              { operation: "get_status" },
              "Planner privacy status refreshed.",
              "Planner privacy status refreshed.",
            );
          }}
        >
          Refresh privacy status
        </button>
      </div>
    );
  }

  const currentRule = findCurrentOriginRule(status);
  const allowCount = status.persistent_rules.filter((rule) => rule.decision === "allow").length;
  const blockCount = status.persistent_rules.filter((rule) => rule.decision === "block").length;
  const currentAllowPermitted = canPersistentlyAllowCurrentOrigin(status);
  const manualAllowPermitted = status.endpoint_scope !== null
    && status.endpoint_is_loopback === false
    && status.network_mode !== "local_only";
  const manualSubmitDisabled = state.operationBusy
    || manualOrigin.trim().length === 0
    || (manualDecision === "allow" && !manualAllowPermitted);

  return (
    <div className="settings-control-card remote-privacy-settings-card" data-remote-planner-privacy-settings="true">
      <div className="remote-privacy-settings-heading">
        <div>
          <h3>Planner privacy and site permissions</h3>
          <p className="settings-panel-description">
            These controls govern whether sanitized page context may reach a non-loopback planner. They never approve clicks, typing, submissions, downloads, credentials, or other protected actions.
          </p>
        </div>
        <button
          type="button"
          className="settings-control-button settings-control-button-secondary"
          disabled={state.operationBusy || undefined}
          onClick={() => {
            void applyOperation(
              { operation: "get_status" },
              "Planner privacy status refreshed.",
              "Planner privacy status refreshed.",
            );
          }}
        >
          Refresh status
        </button>
      </div>

      {status.migration_notice_pending ? (
        <section className="remote-privacy-migration-notice" aria-labelledby="remote-privacy-migration-title">
          <h4 id="remote-privacy-migration-title">Privacy settings were migrated</h4>
          <p>
            Legacy consent and blocked-origin settings were converted to the typed network mode and structured site rules. Review the choices below; broad legacy consent was not converted into destination-bound site allows.
          </p>
          <button
            type="button"
            className="settings-control-button settings-control-button-secondary"
            disabled={state.operationBusy || undefined}
            onClick={() => {
              void applyOperation(
                { operation: "acknowledge_migration_notice" },
                "Privacy migration notice acknowledged.",
                "Privacy migration notice was already acknowledged.",
              );
            }}
          >
            Acknowledge migration notice
          </button>
        </section>
      ) : null}

      {state.operationError ? (
        <div className="remote-privacy-operation-error" role="alert">
          <p>{state.operationError}</p>
          {handlers?.onDismissOperationError ? (
            <button
              type="button"
              className="settings-control-button settings-control-button-secondary"
              disabled={state.operationBusy || undefined}
              onClick={handlers.onDismissOperationError}
            >
              Dismiss error
            </button>
          ) : null}
        </div>
      ) : null}

      {announcement ? (
        <p className="remote-privacy-operation-status" role="status" aria-live="polite">
          {announcement}
        </p>
      ) : null}
      {state.operationBusy && state.activeOperation ? (
        <p className="remote-privacy-operation-status" role="status" aria-live="polite">
          {OPERATION_LABELS[state.activeOperation]}…
        </p>
      ) : null}

      <fieldset className="remote-privacy-mode-selector" disabled={state.operationBusy || undefined}>
        <legend>Network planner mode</legend>
        {NETWORK_MODE_OPTIONS.map((option) => (
          <label key={option.value} className="remote-privacy-mode-option">
            <input
              type="radio"
              name="remote-planner-network-mode"
              value={option.value}
              data-remote-planner-network-mode={option.value}
              checked={status.network_mode === option.value}
              onChange={() => { requestModeChange(option.value); }}
            />
            <span>
              <strong>{option.label}</strong>
              <span>{option.description}</span>
            </span>
          </label>
        ))}
      </fieldset>

      <section
        className="remote-privacy-loopback-status"
        aria-labelledby="remote-privacy-loopback-title"
        data-remote-planner-loopback-status="true"
      >
        <h4 id="remote-privacy-loopback-title">Current planner destination</h4>
        {status.endpoint_is_loopback === true ? (
          <p>
            <strong>On device:</strong> <code>{status.endpoint_display ?? "loopback planner"}</code>. Context stays on this device; saved remote-data permissions are not used for this destination.
          </p>
        ) : status.endpoint_is_loopback === false ? (
          <p>
            <strong>Network destination:</strong> <code>{status.endpoint_display ?? "configured planner"}</code>. Site allows are bound to the exact normalized destination and current privacy-policy version.
          </p>
        ) : (
          <p>The configured planner destination is unavailable or invalid. Network context remains blocked.</p>
        )}
      </section>

      <section
        className="remote-privacy-current-origin"
        aria-labelledby="remote-privacy-current-origin-title"
        data-remote-planner-current-origin="true"
      >
        <h4 id="remote-privacy-current-origin-title">Current site</h4>
        {status.current_page_origin ? (
          <>
            <p><strong>Origin:</strong> <code>{status.current_page_origin}</code></p>
            <p><strong>Effective policy:</strong> {EFFECTIVE_DECISION_LABELS[status.effective_decision]}</p>
            {status.session_grant_active ? (
              <p className="remote-privacy-inline-status" role="status">A session permission is active for this site and destination.</p>
            ) : null}
            {currentRule?.decision === "allow" ? (
              <p>
                <strong>Saved allow destination:</strong> <code>{currentRule.endpoint_display ?? "destination unavailable"}</code>
                {currentRule.stale ? " — inactive because the destination or privacy-policy version changed" : ""}
              </p>
            ) : currentRule?.decision === "block" ? (
              <p>A saved origin-wide block keeps this site local for every non-loopback planner destination.</p>
            ) : null}
            {status.effective_decision === "high_risk_blocked" ? (
              <p className="settings-panel-description settings-panel-warning" role="status">
                High-risk page blocking is non-overridable. A persistent allow cannot be created for the current page context.
              </p>
            ) : null}
            <div className="settings-button-row settings-button-row-wrap">
              {currentRule?.decision !== "block" ? (
                <button
                  type="button"
                  className="settings-control-button settings-control-button-secondary"
                  data-remote-planner-current-origin-block="true"
                  disabled={state.operationBusy || undefined}
                  onClick={() => {
                    void applyOperation(
                      createCurrentOriginRuleOperation("block"),
                      "The current site will stay local for every non-loopback planner destination.",
                      "The current site already had an identical block rule.",
                    );
                  }}
                >
                  Keep current site local
                </button>
              ) : null}
              {currentAllowPermitted && currentRule?.decision !== "allow" ? (
                <button
                  type="button"
                  className="settings-control-button"
                  data-remote-planner-current-origin-allow="true"
                  disabled={state.operationBusy || undefined}
                  onClick={() => {
                    void applyOperation(
                      createCurrentOriginRuleOperation("allow"),
                      "The current site is allowed for the exact configured planner destination.",
                      "The current site already had an identical allow rule.",
                    );
                  }}
                >
                  Allow current site for {status.endpoint_display ?? "this planner"}
                </button>
              ) : null}
              {currentRule ? (
                <button
                  type="button"
                  className="settings-control-button settings-control-button-secondary"
                  data-remote-planner-current-origin-revoke="true"
                  disabled={state.operationBusy || undefined}
                  onClick={() => {
                    void applyOperation(
                      createRevokeOriginRuleOperation(currentRule),
                      "The current site's saved privacy rule was revoked.",
                      "The current site's saved privacy rule was already absent.",
                    );
                  }}
                >
                  Revoke current-site rule
                </button>
              ) : null}
            </div>
          </>
        ) : (
          <p>
            The current page does not expose a supported normalized HTTP(S) origin. Persistent current-site controls are disabled and network page-context planning remains blocked.
          </p>
        )}
      </section>

      <section
        className="remote-privacy-rule-management"
        aria-labelledby="remote-privacy-rules-title"
        data-remote-planner-rule-management="true"
      >
        <div className="remote-privacy-section-heading">
          <div>
            <h4 id="remote-privacy-rules-title">Saved site rules</h4>
            <p>{allowCount} allow rule{allowCount === 1 ? "" : "s"}; {blockCount} block rule{blockCount === 1 ? "" : "s"}.</p>
          </div>
          {status.stale_allow_rule_count > 0 ? (
            <p className="settings-panel-description settings-panel-warning" role="status">
              {status.stale_allow_rule_count} saved allow rule{status.stale_allow_rule_count === 1 ? " is" : "s are"} stale and cannot authorize network planning.
            </p>
          ) : null}
        </div>
        {status.persistent_rules.length === 0 ? (
          <p>No saved site rules.</p>
        ) : (
          <ul className="remote-privacy-rule-list">
            {status.persistent_rules.map((rule) => (
              <li
                key={`${rule.page_origin}|${rule.decision}|${rule.endpoint_scope ?? ""}|${rule.policy_version}`}
                className="remote-privacy-rule-item"
                data-remote-planner-rule={rule.decision}
                data-remote-planner-rule-stale={rule.stale ? "true" : "false"}
              >
                <div>
                  <p><strong>{rule.decision === "allow" ? "Allow" : "Keep local"}</strong> — <code>{rule.page_origin}</code></p>
                  <p>
                    {rule.decision === "allow"
                      ? <>Destination: <code>{rule.endpoint_display ?? "destination unavailable"}</code></>
                      : "Applies to every non-loopback planner destination."}
                  </p>
                  <p>Created: <time dateTime={safeTimestamp(rule.created_at_ms)}>{safeTimestamp(rule.created_at_ms)}</time></p>
                  {rule.stale ? (
                    <p className="settings-panel-description settings-panel-warning" role="status">
                      Inactive: the destination or privacy-policy version changed. This rule is visible but cannot authorize transmission.
                    </p>
                  ) : null}
                </div>
                <button
                  type="button"
                  className="settings-control-button settings-control-button-secondary"
                  data-remote-planner-rule-revoke="true"
                  disabled={state.operationBusy || undefined}
                  aria-label={`Revoke ${rule.decision} rule for ${rule.page_origin}`}
                  onClick={() => {
                    void applyOperation(
                      createRevokeOriginRuleOperation(rule),
                      `Saved ${rule.decision} rule for ${rule.page_origin} revoked.`,
                      `Saved ${rule.decision} rule for ${rule.page_origin} was already absent.`,
                    );
                  }}
                >
                  Revoke rule
                </button>
              </li>
            ))}
          </ul>
        )}

        <details className="remote-privacy-manual-rule" data-remote-planner-manual-rule="true">
          <summary>Advanced: add a rule for another site</summary>
          <p className="settings-panel-description">
            Enter only an HTTP(S) origin such as <code>https://example.com</code>. Rust validates and normalizes the origin. For allows, Rust binds the rule to the authoritative configured planner destination; this form cannot supply or override that scope.
          </p>
          <label className="settings-field-group" htmlFor="remote-planner-manual-origin">
            <span className="settings-control-label">Site origin</span>
            <input
              id="remote-planner-manual-origin"
              className="settings-control-select"
              type="text"
              value={manualOrigin}
              placeholder="https://example.com"
              spellCheck={false}
              autoComplete="off"
              disabled={state.operationBusy || undefined}
              onChange={(event) => { setManualOrigin(event.currentTarget.value); }}
            />
          </label>
          <label className="settings-field-group" htmlFor="remote-planner-manual-decision">
            <span className="settings-control-label">Rule</span>
            <select
              id="remote-planner-manual-decision"
              className="settings-control-select"
              value={manualDecision}
              disabled={state.operationBusy || undefined}
              onChange={(event) => {
                setManualDecision(event.currentTarget.value as PersistedOriginDecision);
              }}
            >
              <option value="block">Keep local for every network planner</option>
              <option value="allow" disabled={!manualAllowPermitted}>
                Allow for the exact configured planner destination
              </option>
            </select>
          </label>
          {manualDecision === "allow" && !manualAllowPermitted ? (
            <p className="settings-panel-description settings-panel-warning" role="status">
              A persistent allow requires a valid non-loopback planner destination and a network mode that is not Local only.
            </p>
          ) : null}
          <button
            type="button"
            className="settings-control-button"
            data-remote-planner-manual-rule-save="true"
            disabled={manualSubmitDisabled || undefined}
            onClick={() => { void submitManualRule(); }}
          >
            Save structured rule
          </button>
        </details>
      </section>

      <section
        className="remote-privacy-clear-controls"
        aria-labelledby="remote-privacy-clear-title"
        data-remote-planner-clear-controls="true"
      >
        <h4 id="remote-privacy-clear-title">Clear permissions and rules</h4>
        <p className="settings-panel-description">
          Session permissions exist only in memory. Saved rules are durable and remain until explicitly revoked or cleared.
        </p>
        <div className="settings-button-row settings-button-row-wrap">
          <button
            type="button"
            className="settings-control-button settings-control-button-secondary"
            data-remote-planner-clear-session-grants="true"
            disabled={state.operationBusy || undefined}
            onClick={() => {
              void applyOperation(
                { operation: "clear_session_grants" },
                "Every in-memory session permission was cleared.",
                "There were no session permissions to clear.",
              );
            }}
          >
            Clear session permissions
          </button>
          <button
            type="button"
            className="settings-control-button settings-control-button-secondary"
            data-remote-planner-clear-persistent-allows="true"
            disabled={state.operationBusy || allowCount === 0 || undefined}
            onClick={() => { setConfirmation("clear-persistent-allows"); }}
          >
            Clear saved allows
          </button>
          <button
            type="button"
            className="settings-control-button settings-control-button-danger"
            data-remote-planner-clear-all-rules="true"
            disabled={state.operationBusy || status.persistent_rule_count === 0 || undefined}
            onClick={() => { setConfirmation("clear-all-rules"); }}
          >
            Clear all saved rules
          </button>
        </div>
      </section>

      {confirmation ? (
        <PrivacySettingsConfirmationDialog
          kind={confirmation}
          busy={state.operationBusy}
          onCancel={() => { setConfirmation(null); }}
          onConfirm={confirmPendingOperation}
        />
      ) : null}
    </div>
  );
}

export function renderRemotePlannerPrivacySettingsCard(
  state: RemotePlannerPrivacyState,
  handlers?: RemotePlannerPrivacySettingsHandlers,
): ReactNode {
  return <RemotePlannerPrivacySettingsCard state={state} handlers={handlers} />;
}

function ConnectedRemotePlannerPrivacySettingsCard() {
  const state = useSyncExternalStore(
    (onStoreChange) => appShellStore.subscribe(onStoreChange),
    () => appShellStore.getState().remotePlannerPrivacy,
    () => appShellStore.getState().remotePlannerPrivacy,
  );

  return (
    <RemotePlannerPrivacySettingsCard
      state={state}
      handlers={{
        onOperation: runRemotePlannerPrivacyOperation,
        onDismissOperationError: () => {
          appShellStore.dispatch(dismissRemotePlannerPrivacyOperationError());
        },
      }}
    />
  );
}

export function renderConnectedRemotePlannerPrivacySettingsCard(): ReactNode {
  return <ConnectedRemotePlannerPrivacySettingsCard key="planner-privacy" />;
}
