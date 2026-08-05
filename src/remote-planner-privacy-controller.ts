import { classifyInvokeFailure } from "./api/errors.ts";
import {
  applyRemotePlannerPrivacyOperation,
  executePlannerOutput,
  submitRemotePlannerConsentResponse,
  type PlannerOutput,
  type RemotePlannerConsentDecision,
  type RemotePlannerConsentResponseOutcome,
  type RemotePlannerExecutionOutcome,
  type RemotePlannerPrivacyOperation,
  type RemotePlannerPrivacyOperationResult,
} from "./tauri-api.ts";
import {
  remotePlannerPrivacyOperationFailed,
  remotePlannerPrivacyOperationStarted,
  remotePlannerPrivacyOperationSucceeded,
  type RemotePlannerPrivacyOperationKind,
  type RemotePlannerPrivacyState,
} from "./remote-planner-privacy-state.ts";
import {
  describeRemoteDataConsentSubmissionFailure,
  type ExecutionUiState,
  type RemoteDataConsentSubmissionFailure,
} from "./planner-orchestration.ts";
import { appShellStore } from "./store.ts";
import { refreshRuntimePanels } from "./refresh-handle.ts";
import { createRequestId, setAppAlertState } from "./panel-state-setters.ts";
import { uiStore } from "./ui-store.ts";

export interface RemotePlannerPrivacyControllerDependencies {
  getPrivacyState: () => RemotePlannerPrivacyState;
  getExecutionUiState: () => ExecutionUiState;
  applyPrivacyOperation: (
    input: {
      requestId: string;
      operation: RemotePlannerPrivacyOperation;
    },
  ) => Promise<RemotePlannerPrivacyOperationResult>;
  submitConsentResponse: (input: {
    challengeId: string;
    challengeDigest: string;
    decision: RemotePlannerConsentDecision;
  }) => Promise<RemotePlannerConsentResponseOutcome>;
  executePlannerOutput: (
    requestId: string,
    plannerOutput: PlannerOutput,
  ) => Promise<RemotePlannerExecutionOutcome>;
  createRequestId: (prefix: string) => string;
  markOperationStarted: (operation: RemotePlannerPrivacyOperationKind) => void;
  applyOperationSuccess: (status: unknown) => void;
  applyOperationFailure: (
    operation: RemotePlannerPrivacyOperationKind,
    message: string,
  ) => void;
  setConsentSubmitting: (challengeId: string, isSubmitting: boolean) => void;
  setConsentError: (
    challengeId: string,
    failure: RemoteDataConsentSubmissionFailure | null,
  ) => void;
  clearConsent: (challengeId: string) => void;
  applyExecutionOutcome: (outcome: RemotePlannerExecutionOutcome) => void;
  refreshRuntime: () => Promise<void>;
  reportGlobalError: (message: string) => void;
  warn: (message: string, details?: Record<string, unknown>) => void;
}

export interface RemotePlannerPrivacyController {
  runOperation: (
    operation: RemotePlannerPrivacyOperation,
  ) => Promise<RemotePlannerPrivacyOperationResult | null>;
  submitConsentDecision: (
    decision: RemotePlannerConsentDecision,
    challengeId: string,
  ) => Promise<RemotePlannerConsentResponseOutcome | null>;
}

export function describeRemotePlannerPrivacyOperationFailure(error: unknown): string {
  const failure = classifyInvokeFailure(error);
  const message = failure.kind === "tool-error"
    ? failure.toolError.message
    : failure.message;
  return `Remote planner privacy settings were not changed. ${message}`;
}

function staleConsentFailure(): RemoteDataConsentSubmissionFailure {
  return {
    kind: "transport-error",
    title: "Privacy request no longer active",
    message: "That privacy request is no longer the active request.",
    guidance: "Review the current privacy request before choosing an option.",
  };
}

export function createRemotePlannerPrivacyController(
  dependencies: RemotePlannerPrivacyControllerDependencies,
): RemotePlannerPrivacyController {
  return {
    runOperation: async (operation) => {
      const operationKind = operation.operation;
      const state = dependencies.getPrivacyState();
      if (state.operationBusy) {
        dependencies.warn("Ignored duplicate remote planner privacy operation while another operation is active.", {
          requestedOperation: operationKind,
          activeOperation: state.activeOperation,
        });
        return null;
      }

      dependencies.markOperationStarted(operationKind);
      try {
        const result = await dependencies.applyPrivacyOperation({
          requestId: dependencies.createRequestId("remote-planner-privacy-operation"),
          operation,
        });
        dependencies.applyOperationSuccess(result.status);
        return result;
      } catch (error) {
        dependencies.applyOperationFailure(
          operationKind,
          describeRemotePlannerPrivacyOperationFailure(error),
        );
        return null;
      }
    },
    submitConsentDecision: async (decision, challengeId) => {
      const consentState = dependencies.getExecutionUiState().remoteDataConsent;
      if (consentState.kind !== "awaiting-remote-data-consent") {
        dependencies.warn("Ignored remote planner consent response with no active challenge.", {
          submittedChallengeId: challengeId,
        });
        return null;
      }

      const activeChallengeId = consentState.challenge.challenge_id;
      if (activeChallengeId !== challengeId) {
        dependencies.setConsentError(activeChallengeId, staleConsentFailure());
        dependencies.warn("Ignored stale remote planner consent response.", {
          submittedChallengeId: challengeId,
          activeChallengeId,
        });
        return null;
      }

      if (consentState.isSubmitting) {
        dependencies.warn("Ignored duplicate remote planner consent response while submission is active.", {
          challengeId,
          decision,
        });
        return null;
      }

      dependencies.setConsentSubmitting(challengeId, true);
      let response: RemotePlannerConsentResponseOutcome;
      try {
        response = await dependencies.submitConsentResponse({
          challengeId,
          challengeDigest: consentState.challenge.challenge_digest,
          decision,
        });
      } catch (error) {
        dependencies.setConsentError(
          challengeId,
          describeRemoteDataConsentSubmissionFailure(error),
        );
        return null;
      }

      try {
        if (response.status === "denied" || response.status === "blocked_persistent") {
          dependencies.clearConsent(challengeId);
        } else if (response.status === "resolved") {
          dependencies.clearConsent(challengeId);
          const outcome = await dependencies.executePlannerOutput(
            consentState.challenge.request_id,
            response.planner_output,
          );
          dependencies.applyExecutionOutcome(outcome);
        } else {
          dependencies.applyExecutionOutcome(response.outcome);
        }
      } catch (error) {
        dependencies.clearConsent(challengeId);
        const failure = classifyInvokeFailure(error);
        const message = failure.kind === "tool-error"
          ? failure.toolError.message
          : failure.message;
        dependencies.reportGlobalError(
          `The privacy decision was accepted, but the resumed command could not be completed. ${message}`,
        );
      }

      try {
        await dependencies.refreshRuntime();
      } catch (error) {
        const failure = classifyInvokeFailure(error);
        const message = failure.kind === "tool-error"
          ? failure.toolError.message
          : failure.message;
        dependencies.reportGlobalError(
          `The privacy decision was processed, but runtime status could not be refreshed. ${message}`,
        );
      }

      return response;
    },
  };
}

const productionController = createRemotePlannerPrivacyController({
  getPrivacyState: () => appShellStore.getState().remotePlannerPrivacy,
  getExecutionUiState: () => uiStore.getState(),
  applyPrivacyOperation: applyRemotePlannerPrivacyOperation,
  submitConsentResponse: submitRemotePlannerConsentResponse,
  executePlannerOutput,
  createRequestId,
  markOperationStarted: (operation) => {
    appShellStore.dispatch(remotePlannerPrivacyOperationStarted(operation));
  },
  applyOperationSuccess: (status) => {
    appShellStore.dispatch(remotePlannerPrivacyOperationSucceeded(status));
  },
  applyOperationFailure: (operation, error) => {
    appShellStore.dispatch(remotePlannerPrivacyOperationFailed({ operation, error }));
  },
  setConsentSubmitting: (challengeId, isSubmitting) => {
    uiStore.setRemoteDataConsentSubmitting(challengeId, isSubmitting);
  },
  setConsentError: (challengeId, failure) => {
    uiStore.setRemoteDataConsentError(challengeId, failure);
  },
  clearConsent: (challengeId) => {
    uiStore.clearRemoteDataConsent(challengeId);
  },
  applyExecutionOutcome: (outcome) => {
    uiStore.applyOutcome(outcome);
  },
  refreshRuntime: refreshRuntimePanels,
  reportGlobalError: (message) => {
    setAppAlertState({ kind: "error", message });
  },
  warn: (message, details) => {
    console.warn(message, details);
  },
});

export function runRemotePlannerPrivacyOperation(
  operation: RemotePlannerPrivacyOperation,
): Promise<RemotePlannerPrivacyOperationResult | null> {
  return productionController.runOperation(operation);
}

export function submitRemotePlannerConsentDecision(
  decision: RemotePlannerConsentDecision,
  challengeId: string,
): Promise<RemotePlannerConsentResponseOutcome | null> {
  return productionController.submitConsentDecision(decision, challengeId);
}
