// Pure operation-building, selection, and formatting logic for the remote
// planner privacy settings card. Kept free of JSX so it can be unit tested
// (and reasoned about) independently of rendering.

import type {
  PersistedOriginDecision,
  RemotePlannerNetworkMode,
  RemotePlannerOriginRuleStatus,
  RemotePlannerPrivacyOperation,
  RemotePlannerPrivacyOperationResult,
  RemotePlannerPrivacyStatus,
} from "../api/remote-planner-privacy.ts";

export const NETWORK_MODE_OPTIONS = [
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

export const EFFECTIVE_DECISION_LABELS: Record<RemotePlannerPrivacyStatus["effective_decision"], string> = {
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

export const OPERATION_LABELS: Record<RemotePlannerPrivacyOperation["operation"], string> = {
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

export type PrivacyConfirmationKind =
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

export function safeTimestamp(createdAtMs: number): string {
  const date = new Date(createdAtMs);
  if (!Number.isFinite(date.getTime())) {
    return "Unknown creation time";
  }
  return date.toISOString();
}
