import { invokeCommand } from "./errors.ts";

export type RemotePlannerNetworkMode =
  | "local_only"
  | "ask_per_origin"
  | "allow_sanitized_non_high_risk";

export type PersistedOriginDecision = "allow" | "block";

export type RemotePlannerEffectiveDecision =
  | "loopback_local"
  | "local_only"
  | "high_risk_blocked"
  | "origin_blocked"
  | "allowed_global"
  | "allowed_persistent"
  | "allowed_session"
  | "consent_required"
  | "origin_unavailable"
  | "planner_unavailable";

export type RemotePlannerDisclosureClass =
  | "user_transcript"
  | "page_origin"
  | "selected_page_regions"
  | "selected_element_metadata"
  | "ocr_derived_regions"
  | "tool_observation_summaries"
  | "skill_summaries"
  | "trusted_runtime_contracts";

export interface RemotePlannerDisclosureCounts {
  selected_region_count: number;
  selected_element_count: number;
  ocr_derived_region_count: number;
  tool_history_count: number;
  skill_summary_count: number;
  sanitized_serialized_bytes: number;
}

export interface RemotePlannerConsentChallengeSummary {
  challenge_id: string;
  request_id: string;
  page_origin: string;
  endpoint_display: string;
  profile_name: string;
  model_label: string;
  policy_version: number;
  disclosure_classes: RemotePlannerDisclosureClass[];
  disclosure_counts: RemotePlannerDisclosureCounts;
  expires_at_ms: number;
  allow_once: boolean;
  allow_session: boolean;
  allow_persistent: boolean;
  block_persistent: boolean;
}

export interface RemotePlannerOriginRuleStatus {
  page_origin: string;
  decision: PersistedOriginDecision;
  endpoint_scope: string | null;
  endpoint_display: string | null;
  policy_version: number;
  created_at_ms: number;
  stale: boolean;
}

export interface RemotePlannerPrivacyStatus {
  network_mode: RemotePlannerNetworkMode;
  endpoint_scope: string | null;
  endpoint_display: string | null;
  endpoint_is_loopback: boolean | null;
  current_page_origin: string | null;
  effective_decision: RemotePlannerEffectiveDecision;
  reason_code: string | null;
  persistent_rule: PersistedOriginDecision | null;
  session_grant_active: boolean;
  pending_challenge: RemotePlannerConsentChallengeSummary | null;
  policy_version: number;
  persistent_rule_count: number;
  stale_allow_rule_count: number;
  persistent_rules: RemotePlannerOriginRuleStatus[];
  migration_notice_pending: boolean;
}

export type RemotePlannerPrivacyOperation =
  | { operation: "get_status" }
  | {
      operation: "set_network_mode";
      network_mode: RemotePlannerNetworkMode;
    }
  | {
      operation: "upsert_origin_rule";
      page_origin: string;
      decision: PersistedOriginDecision;
    }
  | {
      operation: "upsert_current_origin_rule";
      decision: PersistedOriginDecision;
    }
  | {
      operation: "revoke_origin_rule";
      page_origin: string;
      decision: PersistedOriginDecision;
      endpoint_scope: string | null;
    }
  | { operation: "clear_session_grants" }
  | { operation: "clear_persistent_allows" }
  | {
      operation: "clear_all_persistent_rules";
      confirmed: boolean;
    }
  | { operation: "acknowledge_migration_notice" };

export interface RemotePlannerPrivacyOperationResult {
  status: RemotePlannerPrivacyStatus;
  changed: boolean;
  network_mode: RemotePlannerNetworkMode;
  consent_to_remote_page_data: boolean;
  local_only: boolean;
  blocked_origins: string[];
  high_risk_origin_policy: string;
}

export interface RemotePlannerPrivacyOperationInput {
  requestId: string;
  timeoutMs?: number;
  operation: RemotePlannerPrivacyOperation;
}

declare module "../tauri-types.ts" {
  interface AgentStateData {
    remote_planner_privacy_status: RemotePlannerPrivacyStatus;
  }
}

export async function applyRemotePlannerPrivacyOperation(
  input: RemotePlannerPrivacyOperationInput,
): Promise<RemotePlannerPrivacyOperationResult> {
  return invokeCommand<RemotePlannerPrivacyOperationResult>(
    "set_remote_planner_privacy_settings",
    {
      requestId: input.requestId,
      timeoutMs: input.timeoutMs,
      operation: input.operation,
    },
  );
}

export async function getRemotePlannerPrivacyStatus(input: {
  requestId: string;
  timeoutMs?: number;
}): Promise<RemotePlannerPrivacyStatus> {
  const result = await applyRemotePlannerPrivacyOperation({
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
    operation: { operation: "get_status" },
  });
  return result.status;
}
