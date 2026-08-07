import {
  useState,
  useSyncExternalStore,
  type ReactNode,
} from "react";

import { runRemotePlannerPrivacyOperation } from "../remote-planner-privacy-controller.ts";
import {
  dismissRemotePlannerPrivacyOperationError,
  type RemotePlannerPrivacyState,
} from "../remote-planner-privacy-state.ts";
import { appShellStore } from "../store.ts";
import { PrivacySettingsConfirmationDialog } from "./planner-privacy-confirmation-dialog.tsx";
import {
  canPersistentlyAllowCurrentOrigin,
  createConfirmedClearAllRulesOperation,
  createCurrentOriginRuleOperation,
  createManualOriginRuleOperation,
  createNetworkModeOperation,
  createRevokeOriginRuleOperation,
  EFFECTIVE_DECISION_LABELS,
  findCurrentOriginRule,
  NETWORK_MODE_OPTIONS,
  OPERATION_LABELS,
  safeTimestamp,
  type PrivacyConfirmationKind,
  type RemotePlannerPrivacySettingsHandlers,
} from "./planner-privacy-operations.ts";
import type {
  PersistedOriginDecision,
  RemotePlannerNetworkMode,
  RemotePlannerPrivacyOperation,
  RemotePlannerPrivacyOperationResult,
} from "../api/remote-planner-privacy.ts";

export {
  canPersistentlyAllowCurrentOrigin,
  createConfirmedClearAllRulesOperation,
  createManualOriginRuleOperation,
  createNetworkModeOperation,
  createRevokeOriginRuleOperation,
  findCurrentOriginRule,
};
export type { RemotePlannerPrivacySettingsHandlers };

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
