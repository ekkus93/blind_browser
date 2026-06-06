import { invokeCommand, unwrapToolResult } from "./errors.ts";
import type {
  AgentStateData,
  ConfirmActionResolution,
  ConfirmActionResponseInput,
  DirectToolRequestInput,
  ExecutionOutcome,
  PlannerOutput,
  ToolResult,
} from "../tauri-types.ts";

export async function executePlannerOutput(
  requestId: string,
  plannerOutput: PlannerOutput,
): Promise<ExecutionOutcome> {
  return invokeCommand<ExecutionOutcome>("execute_planner_output", {
    requestId,
    plannerOutput,
  });
}

export async function resolveCommand(
  requestId: string,
  transcript: string,
): Promise<PlannerOutput> {
  return invokeCommand<PlannerOutput>("resolve_command", {
    requestId,
    transcript,
  });
}

export async function submitConfirmationResponse(
  input: ConfirmActionResponseInput,
): Promise<ConfirmActionResolution> {
  return invokeCommand<ConfirmActionResolution>("submit_confirmation_response", {
    confirmationId: input.confirmationId,
    confirmed: input.confirmed,
    timedOut: input.timedOut,
  });
}

export async function getAgentState(input: DirectToolRequestInput): Promise<AgentStateData> {
  const result = await invokeCommand<ToolResult<AgentStateData>>("get_agent_state", {
    requestId: input.requestId,
    timeoutMs: input.timeoutMs,
    includeLastTranscript: input.includeLastTranscript ?? false,
  });
  return unwrapToolResult(result);
}
