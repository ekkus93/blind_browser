import {
  classifyInvokeFailure,
  executePlannerOutput,
  submitConfirmationResponse,
  type ConfirmActionResolution,
  type ConfirmActionResponseInput,
  type ExecutionOutcome,
  type InvokeFailure,
  type PlannerOutput,
} from "./tauri-api.ts";

export {
  executePlannerOutput,
  submitConfirmationResponse,
  type ConfirmActionData,
  type ConfirmActionResolution,
  type ConfirmActionResponseInput,
  type ExecutionOutcome,
  type InvokeFailure,
  type PlannerOutput,
  type ToolError,
} from "./tauri-api.ts";

export type AwaitingConfirmationOutcome = Extract<
  ExecutionOutcome,
  { AwaitingConfirmation: unknown }
>;

export type ConfirmationUiState =
  | {
      kind: "idle";
    }
  | {
      kind: "awaiting-confirmation";
      isSubmitting: boolean;
      submissionError: ConfirmationSubmissionFailure | null;
      confirmationId: string;
      promptText: string;
      requestId: string;
      selectedSkills: string[];
      nextStepId: string | null;
      queuedStepIds: string[];
    };

export type ConfirmationSubmissionFailure =
  | {
      kind: "tool-error";
      title: string;
      message: string;
      guidance: string;
      retryable: boolean;
      code: string;
    }
  | {
      kind: "transport-error";
      title: string;
      message: string;
      guidance: string;
    };

export interface ExecutionUiState {
  lastOutcome: ExecutionOutcome | null;
  confirmation: ConfirmationUiState;
}

export interface PlannerExecutionResult {
  outcome: ExecutionOutcome;
  uiState: ExecutionUiState;
}

export interface ConfirmationResolutionResult {
  resolution: ConfirmActionResolution;
  uiState: ExecutionUiState;
}

export interface ExecutionUiStore {
  getState: () => ExecutionUiState;
  setState: (nextState: ExecutionUiState) => void;
  applyOutcome: (outcome: ExecutionOutcome) => ExecutionUiState;
  setConfirmationSubmitting: (confirmationId: string, isSubmitting: boolean) => ExecutionUiState;
  setConfirmationError: (
    confirmationId: string,
    submissionError: ConfirmationSubmissionFailure | null,
  ) => ExecutionUiState;
  subscribe: (listener: (state: ExecutionUiState) => void) => () => void;
}

export function createInitialExecutionUiState(): ExecutionUiState {
  return {
    lastOutcome: null,
    confirmation: {
      kind: "idle",
    },
  };
}

export function createExecutionUiStore(
  initialState: ExecutionUiState = createInitialExecutionUiState(),
): ExecutionUiStore {
  let currentState = initialState;
  const listeners = new Set<(state: ExecutionUiState) => void>();

  const notify = () => {
    for (const listener of listeners) {
      listener(currentState);
    }
  };

  return {
    getState: () => currentState,
    setState: (nextState) => {
      currentState = nextState;
      notify();
    },
    applyOutcome: (outcome) => {
      const nextState = applyExecutionOutcomeToUiState(outcome);
      currentState = nextState;
      notify();
      return nextState;
    },
    setConfirmationSubmitting: (confirmationId, isSubmitting) => {
      if (
        currentState.confirmation.kind !== "awaiting-confirmation" ||
        currentState.confirmation.confirmationId !== confirmationId
      ) {
        return currentState;
      }

      currentState = {
        ...currentState,
        confirmation: {
          ...currentState.confirmation,
          isSubmitting,
          submissionError: isSubmitting ? null : currentState.confirmation.submissionError,
        },
      };
      notify();
      return currentState;
    },
    setConfirmationError: (confirmationId, submissionError) => {
      if (
        currentState.confirmation.kind !== "awaiting-confirmation" ||
        currentState.confirmation.confirmationId !== confirmationId
      ) {
        return currentState;
      }

      currentState = {
        ...currentState,
        confirmation: {
          ...currentState.confirmation,
          isSubmitting: false,
          submissionError,
        },
      };
      notify();
      return currentState;
    },
    subscribe: (listener) => {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
  };
}

export function isAwaitingConfirmationOutcome(
  outcome: ExecutionOutcome,
): outcome is AwaitingConfirmationOutcome {
  return "AwaitingConfirmation" in outcome;
}

export function applyExecutionOutcomeToUiState(
  outcome: ExecutionOutcome,
): ExecutionUiState {
  if (!isAwaitingConfirmationOutcome(outcome)) {
    return {
      lastOutcome: outcome,
      confirmation: {
        kind: "idle",
      },
    };
  }

  const pending = outcome.AwaitingConfirmation.pending_plan_execution;
  return {
    lastOutcome: outcome,
    confirmation: {
      kind: "awaiting-confirmation",
      isSubmitting: false,
      submissionError: null,
      confirmationId: outcome.AwaitingConfirmation.pending_confirmation_id,
      promptText: pending.prompt_text,
      requestId: pending.request_id,
      selectedSkills: pending.selected_skills,
      nextStepId: pending.next_step_id,
      queuedStepIds: pending.queued_step_ids,
    },
  };
}

export async function runPlannerExecution(
  requestId: string,
  plannerOutput: PlannerOutput,
  store?: ExecutionUiStore,
): Promise<PlannerExecutionResult> {
  const outcome = await executePlannerOutput(requestId, plannerOutput);
  const uiState = store ? store.applyOutcome(outcome) : applyExecutionOutcomeToUiState(outcome);
  return {
    outcome,
    uiState,
  };
}

export async function resolveConfirmationResponse(
  input: ConfirmActionResponseInput,
  store?: ExecutionUiStore,
): Promise<ConfirmationResolutionResult> {
  const resolution = await submitConfirmationResponse(input);
  const uiState = store
    ? store.applyOutcome(resolution.resume_outcome)
    : applyExecutionOutcomeToUiState(resolution.resume_outcome);
  return {
    resolution,
    uiState,
  };
}

export function describeConfirmationSubmissionFailure(
  error: unknown,
): ConfirmationSubmissionFailure {
  const failure = classifyInvokeFailure(error);

  if (failure.kind === "tool-error") {
    return mapToolErrorFailure(failure);
  }

  return {
    kind: "transport-error",
    title: "Connection problem",
    message: failure.message,
    guidance: "The desktop runtime did not accept the confirmation request. Check that the app is still running, then try again.",
  };
}

function mapToolErrorFailure(failure: InvokeFailure & { kind: "tool-error" }): ConfirmationSubmissionFailure {
  const { toolError } = failure;
  return {
    kind: "tool-error",
    title: toolError.retryable ? "Runtime rejected the request" : "Runtime cannot complete this request",
    message: toolError.message,
    guidance: toolError.retryable
      ? "The backend reported a retryable error. Review the runtime state and try the confirmation again."
      : "The backend reported a non-retryable tool error. Review the current request or planner state before trying again.",
    retryable: toolError.retryable,
    code: toolError.code,
  };
}