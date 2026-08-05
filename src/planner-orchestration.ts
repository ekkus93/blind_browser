import {
  classifyInvokeFailure,
  executePlannerOutput,
  submitConfirmationResponse,
  type ConfirmActionResolution,
  type ConfirmActionResponseInput,
  type ExecutionOutcome,
  type InvokeFailure,
  type PlannerOutput,
  type RemotePlannerConsentChallenge,
  type RemotePlannerExecutionOutcome,
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
  type RemotePlannerConsentChallenge,
  type RemotePlannerExecutionOutcome,
  type ToolError,
} from "./tauri-api.ts";

export type AwaitingConfirmationOutcome = Extract<
  RemotePlannerExecutionOutcome,
  { AwaitingConfirmation: unknown }
>;

export type NeedsRemoteDataConsentOutcome = Extract<
  RemotePlannerExecutionOutcome,
  { NeedsRemoteDataConsent: unknown }
>;

export type DecisionSubmissionFailure =
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

export type ConfirmationSubmissionFailure = DecisionSubmissionFailure;
export type RemoteDataConsentSubmissionFailure = DecisionSubmissionFailure;

export type ConfirmationUiState =
  | {
      kind: "idle";
    }
  | {
      kind: "awaiting-confirmation";
      isSubmitting: boolean;
      submissionError: ConfirmationSubmissionFailure | null;
      confirmationId: string;
      confirmationDigest: string;
      promptText: string;
      requestId: string;
      selectedSkills: string[];
      nextStepId: string | null;
      queuedStepIds: string[];
    };

export type RemoteDataConsentUiState =
  | {
      kind: "idle";
    }
  | {
      kind: "awaiting-remote-data-consent";
      isSubmitting: boolean;
      submissionError: RemoteDataConsentSubmissionFailure | null;
      challenge: RemotePlannerConsentChallenge;
    };

export interface ExecutionUiState {
  lastOutcome: RemotePlannerExecutionOutcome | null;
  confirmation: ConfirmationUiState;
  remoteDataConsent: RemoteDataConsentUiState;
}

export interface PlannerExecutionResult {
  outcome: RemotePlannerExecutionOutcome;
  uiState: ExecutionUiState;
}

export interface ConfirmationResolutionResult {
  resolution: ConfirmActionResolution;
  uiState: ExecutionUiState;
}

export interface ExecutionUiStore {
  getState: () => ExecutionUiState;
  setState: (nextState: ExecutionUiState) => void;
  applyOutcome: (outcome: RemotePlannerExecutionOutcome) => ExecutionUiState;
  setConfirmationSubmitting: (confirmationId: string, isSubmitting: boolean) => ExecutionUiState;
  setConfirmationError: (
    confirmationId: string,
    submissionError: ConfirmationSubmissionFailure | null,
  ) => ExecutionUiState;
  setRemoteDataConsentSubmitting: (
    challengeId: string,
    isSubmitting: boolean,
  ) => ExecutionUiState;
  setRemoteDataConsentError: (
    challengeId: string,
    submissionError: RemoteDataConsentSubmissionFailure | null,
  ) => ExecutionUiState;
  clearRemoteDataConsent: (challengeId: string) => ExecutionUiState;
  subscribe: (listener: (state: ExecutionUiState) => void) => () => void;
}

export function createInitialExecutionUiState(): ExecutionUiState {
  return {
    lastOutcome: null,
    confirmation: {
      kind: "idle",
    },
    remoteDataConsent: {
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

  const setCurrentState = (nextState: ExecutionUiState) => {
    currentState = nextState;
    notify();
    return currentState;
  };

  return {
    getState: () => currentState,
    setState: (nextState) => {
      setCurrentState(nextState);
    },
    applyOutcome: (outcome) => setCurrentState(applyExecutionOutcomeToUiState(outcome)),
    setConfirmationSubmitting: (confirmationId, isSubmitting) => {
      if (
        currentState.confirmation.kind !== "awaiting-confirmation" ||
        currentState.confirmation.confirmationId !== confirmationId
      ) {
        return currentState;
      }

      return setCurrentState({
        ...currentState,
        confirmation: {
          ...currentState.confirmation,
          isSubmitting,
          submissionError: isSubmitting ? null : currentState.confirmation.submissionError,
        },
      });
    },
    setConfirmationError: (confirmationId, submissionError) => {
      if (
        currentState.confirmation.kind !== "awaiting-confirmation" ||
        currentState.confirmation.confirmationId !== confirmationId
      ) {
        return currentState;
      }

      return setCurrentState({
        ...currentState,
        confirmation: {
          ...currentState.confirmation,
          isSubmitting: false,
          submissionError,
        },
      });
    },
    setRemoteDataConsentSubmitting: (challengeId, isSubmitting) => {
      if (
        currentState.remoteDataConsent.kind !== "awaiting-remote-data-consent" ||
        currentState.remoteDataConsent.challenge.challenge_id !== challengeId
      ) {
        return currentState;
      }

      return setCurrentState({
        ...currentState,
        remoteDataConsent: {
          ...currentState.remoteDataConsent,
          isSubmitting,
          submissionError: isSubmitting
            ? null
            : currentState.remoteDataConsent.submissionError,
        },
      });
    },
    setRemoteDataConsentError: (challengeId, submissionError) => {
      if (
        currentState.remoteDataConsent.kind !== "awaiting-remote-data-consent" ||
        currentState.remoteDataConsent.challenge.challenge_id !== challengeId
      ) {
        return currentState;
      }

      return setCurrentState({
        ...currentState,
        remoteDataConsent: {
          ...currentState.remoteDataConsent,
          isSubmitting: false,
          submissionError,
        },
      });
    },
    clearRemoteDataConsent: (challengeId) => {
      if (
        currentState.remoteDataConsent.kind !== "awaiting-remote-data-consent" ||
        currentState.remoteDataConsent.challenge.challenge_id !== challengeId
      ) {
        return currentState;
      }

      return setCurrentState({
        ...currentState,
        remoteDataConsent: {
          kind: "idle",
        },
      });
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
  outcome: RemotePlannerExecutionOutcome,
): outcome is AwaitingConfirmationOutcome {
  return "AwaitingConfirmation" in outcome;
}

export function isNeedsRemoteDataConsentOutcome(
  outcome: RemotePlannerExecutionOutcome,
): outcome is NeedsRemoteDataConsentOutcome {
  return "NeedsRemoteDataConsent" in outcome;
}

export function applyExecutionOutcomeToUiState(
  outcome: RemotePlannerExecutionOutcome,
): ExecutionUiState {
  if (isNeedsRemoteDataConsentOutcome(outcome)) {
    return {
      lastOutcome: outcome,
      confirmation: {
        kind: "idle",
      },
      remoteDataConsent: {
        kind: "awaiting-remote-data-consent",
        isSubmitting: false,
        submissionError: null,
        challenge: outcome.NeedsRemoteDataConsent.challenge,
      },
    };
  }

  if (!isAwaitingConfirmationOutcome(outcome)) {
    return {
      lastOutcome: outcome,
      confirmation: {
        kind: "idle",
      },
      remoteDataConsent: {
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
      confirmationDigest: pending.manifest_digest,
      promptText: pending.prompt_text,
      requestId: pending.request_id,
      selectedSkills: pending.selected_skills,
      nextStepId: pending.next_step_id,
      queuedStepIds: pending.queued_step_ids,
    },
    remoteDataConsent: {
      kind: "idle",
    },
  };
}

export async function runPlannerExecution(
  requestId: string,
  plannerOutput: PlannerOutput,
  store?: ExecutionUiStore,
): Promise<PlannerExecutionResult> {
  const outcome: ExecutionOutcome = await executePlannerOutput(requestId, plannerOutput);
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
  return describeDecisionSubmissionFailure(
    error,
    "The desktop runtime did not accept the confirmation request. Check that the app is still running, then try again.",
  );
}

export function describeRemoteDataConsentSubmissionFailure(
  error: unknown,
): RemoteDataConsentSubmissionFailure {
  return describeDecisionSubmissionFailure(
    error,
    "The desktop runtime did not accept the privacy decision. Review the current request and runtime state, then try again.",
  );
}

function describeDecisionSubmissionFailure(
  error: unknown,
  transportGuidance: string,
): DecisionSubmissionFailure {
  const failure = classifyInvokeFailure(error);

  if (failure.kind === "tool-error") {
    return mapToolErrorFailure(failure);
  }

  return {
    kind: "transport-error",
    title: "Connection problem",
    message: failure.message,
    guidance: transportGuidance,
  };
}

function mapToolErrorFailure(
  failure: InvokeFailure & { kind: "tool-error" },
): DecisionSubmissionFailure {
  const { toolError } = failure;
  return {
    kind: "tool-error",
    title: toolError.retryable
      ? "Runtime rejected the request"
      : "Runtime cannot complete this request",
    message: toolError.message,
    guidance: toolError.retryable
      ? "The backend reported a retryable error. Review the runtime state and try again."
      : "The backend reported a non-retryable tool error. Review the current request or planner state before trying again.",
    retryable: toolError.retryable,
    code: toolError.code,
  };
}
