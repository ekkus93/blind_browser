import type {
  ExecutionOutcome,
  ExecutionTrace,
  PlannerOutput,
} from "../tauri-types.ts";
import { invokeCommand } from "./errors.ts";

export type RemotePlannerConsentDecision =
  | "allow_once"
  | "allow_session"
  | "allow_persistent"
  | "block_persistent"
  | "deny";

export type RemotePlannerConsentDisclosureClass =
  | "user_transcript"
  | "page_origin"
  | "selected_page_regions"
  | "selected_element_metadata"
  | "ocr_derived_regions"
  | "tool_observation_summaries"
  | "skill_summaries"
  | "trusted_runtime_contracts";

export interface RemotePlannerConsentDisclosureCounts {
  selected_region_count: number;
  selected_element_count: number;
  ocr_derived_region_count: number;
  tool_history_count: number;
  skill_summary_count: number;
  sanitized_serialized_bytes: number;
}

export interface RemotePlannerConsentChallenge {
  challenge_id: string;
  challenge_digest: string;
  request_id: string;
  page_origin: string;
  endpoint_display: string;
  endpoint_scope: string;
  profile_name: string;
  model_label: string;
  policy_version: number;
  disclosure_classes: RemotePlannerConsentDisclosureClass[];
  disclosure_counts: RemotePlannerConsentDisclosureCounts;
  expires_at_ms: number;
  allow_once: boolean;
  allow_session: boolean;
  allow_persistent: boolean;
  block_persistent: boolean;
}

export type RemotePlannerExecutionOutcome =
  | ExecutionOutcome
  | {
      NeedsRemoteDataConsent: {
        trace: ExecutionTrace;
        challenge: RemotePlannerConsentChallenge;
      };
    };

export type RemotePlannerConsentResponseOutcome =
  | { status: "denied" }
  | { status: "blocked_persistent" }
  | {
      status: "resolved";
      planner_output: PlannerOutput;
    }
  | {
      status: "executed";
      outcome: RemotePlannerExecutionOutcome;
    };

export interface SubmitRemotePlannerConsentResponseInput {
  challengeId: string;
  challengeDigest: string;
  decision: RemotePlannerConsentDecision;
}

export async function submitRemotePlannerConsentResponse(
  input: SubmitRemotePlannerConsentResponseInput,
): Promise<RemotePlannerConsentResponseOutcome> {
  return invokeCommand<RemotePlannerConsentResponseOutcome>(
    "submit_remote_planner_consent_response",
    {
      challengeId: input.challengeId,
      challengeDigest: input.challengeDigest,
      decision: input.decision,
    },
  );
}
