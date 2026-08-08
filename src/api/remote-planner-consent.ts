import type {
  ExecutionOutcome,
  ExecutionTrace,
  PlannerOutput,
  StartListeningData,
  ToolError,
  ToolResult,
  TranscribeAndExecuteCommandData,
  TranscribeCommandData,
} from "../tauri-types.ts";
import { invokeCommand, isRecord, parseToolError } from "./errors.ts";

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
  | "trusted_runtime_contracts"
  | "narration_text"
  | "microphone_audio";

export interface RemotePlannerConsentDisclosureCounts {
  selected_region_count: number;
  selected_element_count: number;
  ocr_derived_region_count: number;
  tool_history_count: number;
  skill_summary_count: number;
  sanitized_serialized_bytes: number;
  narration_text_bytes: number;
  microphone_audio_duration_ms: number;
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

export type NarrationConsentResponseOutcome =
  | { status: "denied" }
  | { status: "blocked_persistent" }
  | { status: "spoken" };

export type MicrophoneConsentResponseOutcome =
  | { status: "denied" }
  | { status: "blocked_persistent" }
  | {
      status: "listening_started";
      result: ToolResult<StartListeningData>;
    }
  | {
      status: "transcribed";
      result: ToolResult<TranscribeCommandData>;
    }
  | {
      status: "transcribed_and_executed";
      result: TranscribeAndExecuteCommandData;
    }
  | { status: "planner_resume_blocked" };

export type RemoteDataConsentResponseOutcome =
  | RemotePlannerConsentResponseOutcome
  | NarrationConsentResponseOutcome
  | MicrophoneConsentResponseOutcome;

export interface SubmitRemotePlannerConsentResponseInput {
  challengeId: string;
  challengeDigest: string;
  decision: RemotePlannerConsentDecision;
}

const DISCLOSURE_CLASSES = new Set<RemotePlannerConsentDisclosureClass>([
  "user_transcript",
  "page_origin",
  "selected_page_regions",
  "selected_element_metadata",
  "ocr_derived_regions",
  "tool_observation_summaries",
  "skill_summaries",
  "trusted_runtime_contracts",
  "narration_text",
  "microphone_audio",
]);

function isFiniteNonNegativeNumber(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value) && value >= 0;
}

function parseDisclosureCounts(value: unknown): RemotePlannerConsentDisclosureCounts | null {
  if (!isRecord(value)) {
    return null;
  }
  const fields = [
    "selected_region_count",
    "selected_element_count",
    "ocr_derived_region_count",
    "tool_history_count",
    "skill_summary_count",
    "sanitized_serialized_bytes",
  ] as const;
  if (fields.some((field) => !isFiniteNonNegativeNumber(value[field]))) {
    return null;
  }

  const narrationTextBytes = value.narration_text_bytes ?? 0;
  const microphoneAudioDurationMs = value.microphone_audio_duration_ms ?? 0;
  if (
    !isFiniteNonNegativeNumber(narrationTextBytes)
    || !isFiniteNonNegativeNumber(microphoneAudioDurationMs)
  ) {
    return null;
  }

  return {
    selected_region_count: value.selected_region_count as number,
    selected_element_count: value.selected_element_count as number,
    ocr_derived_region_count: value.ocr_derived_region_count as number,
    tool_history_count: value.tool_history_count as number,
    skill_summary_count: value.skill_summary_count as number,
    sanitized_serialized_bytes: value.sanitized_serialized_bytes as number,
    narration_text_bytes: narrationTextBytes,
    microphone_audio_duration_ms: microphoneAudioDurationMs,
  };
}

export function parseRemoteDataConsentChallenge(
  value: unknown,
): RemotePlannerConsentChallenge | null {
  if (!isRecord(value) || !Array.isArray(value.disclosure_classes)) {
    return null;
  }

  const disclosureClasses: RemotePlannerConsentDisclosureClass[] = [];
  for (const candidate of value.disclosure_classes) {
    if (typeof candidate !== "string" || !DISCLOSURE_CLASSES.has(
      candidate as RemotePlannerConsentDisclosureClass,
    )) {
      return null;
    }
    disclosureClasses.push(candidate as RemotePlannerConsentDisclosureClass);
  }

  const counts = parseDisclosureCounts(value.disclosure_counts);
  if (!counts) {
    return null;
  }

  const stringFields = [
    "challenge_id",
    "challenge_digest",
    "request_id",
    "page_origin",
    "endpoint_display",
    "endpoint_scope",
    "profile_name",
    "model_label",
  ] as const;
  if (stringFields.some((field) => typeof value[field] !== "string")) {
    return null;
  }

  const numberFields = ["policy_version", "expires_at_ms"] as const;
  if (numberFields.some((field) => !isFiniteNonNegativeNumber(value[field]))) {
    return null;
  }

  const booleanFields = [
    "allow_once",
    "allow_session",
    "allow_persistent",
    "block_persistent",
  ] as const;
  if (booleanFields.some((field) => typeof value[field] !== "boolean")) {
    return null;
  }

  return {
    challenge_id: value.challenge_id as string,
    challenge_digest: value.challenge_digest as string,
    request_id: value.request_id as string,
    page_origin: value.page_origin as string,
    endpoint_display: value.endpoint_display as string,
    endpoint_scope: value.endpoint_scope as string,
    profile_name: value.profile_name as string,
    model_label: value.model_label as string,
    policy_version: value.policy_version as number,
    disclosure_classes: disclosureClasses,
    disclosure_counts: counts,
    expires_at_ms: value.expires_at_ms as number,
    allow_once: value.allow_once as boolean,
    allow_session: value.allow_session as boolean,
    allow_persistent: value.allow_persistent as boolean,
    block_persistent: value.block_persistent as boolean,
  };
}

export function remoteDataConsentChallengeFromToolError(
  error: ToolError,
): RemotePlannerConsentChallenge | null {
  if (error.code !== "remote_data_consent_required" || !isRecord(error.details)) {
    return null;
  }
  return parseRemoteDataConsentChallenge(error.details.challenge);
}

export function remoteDataConsentChallengeFromInvokeError(
  error: unknown,
): RemotePlannerConsentChallenge | null {
  const toolError = parseToolError(error);
  return toolError ? remoteDataConsentChallengeFromToolError(toolError) : null;
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

export async function submitNarrationConsentResponse(
  input: SubmitRemotePlannerConsentResponseInput,
): Promise<NarrationConsentResponseOutcome> {
  return invokeCommand<NarrationConsentResponseOutcome>(
    "submit_narration_consent_response",
    {
      challengeId: input.challengeId,
      challengeDigest: input.challengeDigest,
      decision: input.decision,
    },
  );
}

export async function submitMicrophoneConsentResponse(
  input: SubmitRemotePlannerConsentResponseInput,
): Promise<MicrophoneConsentResponseOutcome> {
  return invokeCommand<MicrophoneConsentResponseOutcome>(
    "submit_microphone_consent_response",
    {
      challengeId: input.challengeId,
      challengeDigest: input.challengeDigest,
      decision: input.decision,
    },
  );
}
