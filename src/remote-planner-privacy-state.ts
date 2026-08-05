import { createSlice, type PayloadAction } from "@reduxjs/toolkit";

import type {
  PersistedOriginDecision,
  RemotePlannerConsentChallengeSummary,
  RemotePlannerDisclosureClass,
  RemotePlannerEffectiveDecision,
  RemotePlannerNetworkMode,
  RemotePlannerOriginRuleStatus,
  RemotePlannerPrivacyOperation,
  RemotePlannerPrivacyStatus,
} from "./api/remote-planner-privacy.ts";

export type RemotePlannerPrivacyOperationKind = RemotePlannerPrivacyOperation["operation"];

export interface RemotePlannerPrivacyState {
  status: RemotePlannerPrivacyStatus | null;
  isLoaded: boolean;
  refreshBusy: boolean;
  refreshError: string | null;
  operationBusy: boolean;
  activeOperation: RemotePlannerPrivacyOperationKind | null;
  operationError: string | null;
}

const NETWORK_MODES = [
  "local_only",
  "ask_per_origin",
  "allow_sanitized_non_high_risk",
] as const satisfies readonly RemotePlannerNetworkMode[];

const EFFECTIVE_DECISIONS = [
  "loopback_local",
  "local_only",
  "high_risk_blocked",
  "origin_blocked",
  "allowed_global",
  "allowed_persistent",
  "allowed_session",
  "consent_required",
  "origin_unavailable",
  "planner_unavailable",
] as const satisfies readonly RemotePlannerEffectiveDecision[];

const PERSISTED_DECISIONS = [
  "allow",
  "block",
] as const satisfies readonly PersistedOriginDecision[];

const DISCLOSURE_CLASSES = [
  "user_transcript",
  "page_origin",
  "selected_page_regions",
  "selected_element_metadata",
  "ocr_derived_regions",
  "tool_observation_summaries",
  "skill_summaries",
  "trusted_runtime_contracts",
] as const satisfies readonly RemotePlannerDisclosureClass[];

function requireRecord(value: unknown, path: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`${path} must be an object`);
  }
  return value as Record<string, unknown>;
}

function requireString(value: unknown, path: string): string {
  if (typeof value !== "string") {
    throw new Error(`${path} must be a string`);
  }
  return value;
}

function requireNullableString(value: unknown, path: string): string | null {
  if (value === null) {
    return null;
  }
  return requireString(value, path);
}

function requireBoolean(value: unknown, path: string): boolean {
  if (typeof value !== "boolean") {
    throw new Error(`${path} must be a boolean`);
  }
  return value;
}

function requireNullableBoolean(value: unknown, path: string): boolean | null {
  if (value === null) {
    return null;
  }
  return requireBoolean(value, path);
}

function requireNonNegativeInteger(value: unknown, path: string): number {
  if (!Number.isSafeInteger(value) || (value as number) < 0) {
    throw new Error(`${path} must be a non-negative safe integer`);
  }
  return value as number;
}

function requireEnum<T extends string>(
  value: unknown,
  allowedValues: readonly T[],
  path: string,
): T {
  if (typeof value !== "string" || !(allowedValues as readonly string[]).includes(value)) {
    throw new Error(`${path} contained an unsupported value`);
  }
  return value as T;
}

function requireNullableEnum<T extends string>(
  value: unknown,
  allowedValues: readonly T[],
  path: string,
): T | null {
  if (value === null) {
    return null;
  }
  return requireEnum(value, allowedValues, path);
}

function projectConsentChallenge(
  value: unknown,
): RemotePlannerConsentChallengeSummary | null {
  if (value === null) {
    return null;
  }

  const challenge = requireRecord(value, "remote planner privacy pending challenge");
  if (!Array.isArray(challenge.disclosure_classes)) {
    throw new Error("remote planner privacy disclosure classes must be an array");
  }
  const disclosureCounts = requireRecord(
    challenge.disclosure_counts,
    "remote planner privacy disclosure counts",
  );

  return {
    challenge_id: requireString(challenge.challenge_id, "pending challenge challenge_id"),
    request_id: requireString(challenge.request_id, "pending challenge request_id"),
    page_origin: requireString(challenge.page_origin, "pending challenge page_origin"),
    endpoint_display: requireString(
      challenge.endpoint_display,
      "pending challenge endpoint_display",
    ),
    profile_name: requireString(challenge.profile_name, "pending challenge profile_name"),
    model_label: requireString(challenge.model_label, "pending challenge model_label"),
    policy_version: requireNonNegativeInteger(
      challenge.policy_version,
      "pending challenge policy_version",
    ),
    disclosure_classes: challenge.disclosure_classes.map((entry, index) =>
      requireEnum(entry, DISCLOSURE_CLASSES, `pending challenge disclosure_classes[${index}]`)
    ),
    disclosure_counts: {
      selected_region_count: requireNonNegativeInteger(
        disclosureCounts.selected_region_count,
        "pending challenge selected_region_count",
      ),
      selected_element_count: requireNonNegativeInteger(
        disclosureCounts.selected_element_count,
        "pending challenge selected_element_count",
      ),
      ocr_derived_region_count: requireNonNegativeInteger(
        disclosureCounts.ocr_derived_region_count,
        "pending challenge ocr_derived_region_count",
      ),
      tool_history_count: requireNonNegativeInteger(
        disclosureCounts.tool_history_count,
        "pending challenge tool_history_count",
      ),
      skill_summary_count: requireNonNegativeInteger(
        disclosureCounts.skill_summary_count,
        "pending challenge skill_summary_count",
      ),
      sanitized_serialized_bytes: requireNonNegativeInteger(
        disclosureCounts.sanitized_serialized_bytes,
        "pending challenge sanitized_serialized_bytes",
      ),
    },
    expires_at_ms: requireNonNegativeInteger(
      challenge.expires_at_ms,
      "pending challenge expires_at_ms",
    ),
    allow_once: requireBoolean(challenge.allow_once, "pending challenge allow_once"),
    allow_session: requireBoolean(challenge.allow_session, "pending challenge allow_session"),
    allow_persistent: requireBoolean(
      challenge.allow_persistent,
      "pending challenge allow_persistent",
    ),
    block_persistent: requireBoolean(
      challenge.block_persistent,
      "pending challenge block_persistent",
    ),
  };
}

function projectOriginRule(value: unknown, index: number): RemotePlannerOriginRuleStatus {
  const rule = requireRecord(value, `remote planner privacy persistent_rules[${index}]`);
  return {
    page_origin: requireString(rule.page_origin, `persistent_rules[${index}].page_origin`),
    decision: requireEnum(
      rule.decision,
      PERSISTED_DECISIONS,
      `persistent_rules[${index}].decision`,
    ),
    endpoint_scope: requireNullableString(
      rule.endpoint_scope,
      `persistent_rules[${index}].endpoint_scope`,
    ),
    endpoint_display: requireNullableString(
      rule.endpoint_display,
      `persistent_rules[${index}].endpoint_display`,
    ),
    policy_version: requireNonNegativeInteger(
      rule.policy_version,
      `persistent_rules[${index}].policy_version`,
    ),
    created_at_ms: requireNonNegativeInteger(
      rule.created_at_ms,
      `persistent_rules[${index}].created_at_ms`,
    ),
    stale: requireBoolean(rule.stale, `persistent_rules[${index}].stale`),
  };
}

export function projectRemotePlannerPrivacyStatus(value: unknown): RemotePlannerPrivacyStatus {
  const status = requireRecord(value, "remote planner privacy status");
  if (!Array.isArray(status.persistent_rules)) {
    throw new Error("remote planner privacy persistent_rules must be an array");
  }

  return {
    network_mode: requireEnum(status.network_mode, NETWORK_MODES, "network_mode"),
    endpoint_scope: requireNullableString(status.endpoint_scope, "endpoint_scope"),
    endpoint_display: requireNullableString(status.endpoint_display, "endpoint_display"),
    endpoint_is_loopback: requireNullableBoolean(
      status.endpoint_is_loopback,
      "endpoint_is_loopback",
    ),
    current_page_origin: requireNullableString(
      status.current_page_origin,
      "current_page_origin",
    ),
    effective_decision: requireEnum(
      status.effective_decision,
      EFFECTIVE_DECISIONS,
      "effective_decision",
    ),
    reason_code: requireNullableString(status.reason_code, "reason_code"),
    persistent_rule: requireNullableEnum(
      status.persistent_rule,
      PERSISTED_DECISIONS,
      "persistent_rule",
    ),
    session_grant_active: requireBoolean(
      status.session_grant_active,
      "session_grant_active",
    ),
    pending_challenge: projectConsentChallenge(status.pending_challenge),
    policy_version: requireNonNegativeInteger(status.policy_version, "policy_version"),
    persistent_rule_count: requireNonNegativeInteger(
      status.persistent_rule_count,
      "persistent_rule_count",
    ),
    stale_allow_rule_count: requireNonNegativeInteger(
      status.stale_allow_rule_count,
      "stale_allow_rule_count",
    ),
    persistent_rules: status.persistent_rules.map(projectOriginRule),
    migration_notice_pending: requireBoolean(
      status.migration_notice_pending,
      "migration_notice_pending",
    ),
  };
}

export function createInitialRemotePlannerPrivacyState(): RemotePlannerPrivacyState {
  return {
    status: null,
    isLoaded: false,
    refreshBusy: false,
    refreshError: null,
    operationBusy: false,
    activeOperation: null,
    operationError: null,
  };
}

const remotePlannerPrivacySlice = createSlice({
  name: "remotePlannerPrivacy",
  initialState: createInitialRemotePlannerPrivacyState(),
  reducers: {
    remotePlannerPrivacyRefreshStarted(state) {
      state.refreshBusy = true;
      state.refreshError = null;
    },
    remotePlannerPrivacyRefreshSucceeded(state, action: PayloadAction<unknown>) {
      state.status = projectRemotePlannerPrivacyStatus(action.payload);
      state.isLoaded = true;
      state.refreshBusy = false;
      state.refreshError = null;
    },
    remotePlannerPrivacyRefreshFailed(state, action: PayloadAction<string>) {
      state.refreshBusy = false;
      state.refreshError = action.payload;
    },
    remotePlannerPrivacyOperationStarted(
      state,
      action: PayloadAction<RemotePlannerPrivacyOperationKind>,
    ) {
      state.operationBusy = true;
      state.activeOperation = action.payload;
      state.operationError = null;
    },
    remotePlannerPrivacyOperationSucceeded(state, action: PayloadAction<unknown>) {
      state.status = projectRemotePlannerPrivacyStatus(action.payload);
      state.isLoaded = true;
      state.operationBusy = false;
      state.activeOperation = null;
      state.operationError = null;
    },
    remotePlannerPrivacyOperationFailed(
      state,
      action: PayloadAction<{
        operation: RemotePlannerPrivacyOperationKind;
        error: string;
      }>,
    ) {
      state.operationBusy = false;
      state.activeOperation = action.payload.operation;
      state.operationError = action.payload.error;
    },
    dismissRemotePlannerPrivacyOperationError(state) {
      if (state.operationBusy) {
        return;
      }
      state.activeOperation = null;
      state.operationError = null;
    },
  },
});

export const {
  remotePlannerPrivacyRefreshStarted,
  remotePlannerPrivacyRefreshSucceeded,
  remotePlannerPrivacyRefreshFailed,
  remotePlannerPrivacyOperationStarted,
  remotePlannerPrivacyOperationSucceeded,
  remotePlannerPrivacyOperationFailed,
  dismissRemotePlannerPrivacyOperationError,
} = remotePlannerPrivacySlice.actions;

export const remotePlannerPrivacyReducer = remotePlannerPrivacySlice.reducer;
