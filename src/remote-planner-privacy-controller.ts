import { classifyInvokeFailure, unwrapToolResult } from "./api/errors.ts";
import {
  applyRemotePlannerPrivacyOperation,
  executePlannerOutput,
  submitMicrophoneConsentResponse,
  submitNarrationConsentResponse,
  submitRemotePlannerConsentResponse,
  type MicrophoneConsentResponseOutcome,
  type NarrationConsentResponseOutcome,
  type PlannerOutput,
  type RemoteDataConsentResponseOutcome,
  type RemotePlannerConsentChallenge,
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
  submitNarrationConsentResponse: (input: {
    challengeId: string;
    challengeDigest: string;
    decision: RemotePlannerConsentDecision;
  }) => Promise<NarrationConsentResponseOutcome>;
  submitMicrophoneConsentResponse: (input: {
    challengeId: string;
    challengeDigest: string;
    decision: RemotePlannerConsentDecision;
  }) => Promise<MicrophoneConsentResponseOutcome>;
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
  reportGlobalInfo: (message: string) => void;
  warn: (message: string, details?: Record<string, unknown>) => void;
}

export interface RemotePlannerPrivacyController {
  runOperation: (
    operation: RemotePlannerPrivacyOperation,
  ) => Promise<RemotePlannerPrivacyOperationResult | null>;
  submitConsentDecision: (
    decision: RemotePlannerConsentDecision,
    challengeId: string,
  ) => Promise<RemoteDataConsentResponseOutcome | null>;
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

type ConsentSubmissionKind = "planner" | "narration" | "microphone";

function consentSubmissionKind(
  challenge: RemotePlannerConsentChallenge,
): ConsentSubmissionKind | null {
  const hasNarration = challenge.disclosure_classes.includes("narration_text");
  const hasMicrophone = challenge.disclosure_classes.includes("microphone_audio");
  if (hasNarration && hasMicrophone) {
    return null;
  }
  if (hasNarration) {
    return challenge.disclosure_classes.length === 1 ? "narration" : null;
  }
  if (hasMicrophone) {
    return challenge.disclosure_classes.length === 1 ? "microphone" : null;
  }
  return "planner";
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

      const submissionKind = consentSubmissionKind(consentState.challenge);
      if (submissionKind === null) {
        dependencies.setConsentError(activeChallengeId, {
          kind: "transport-error",
          title: "Invalid privacy request",
          message: "The active privacy request mixes incompatible disclosure categories.",
          guidance: "Cancel this request and retry the original action.",
        });
        dependencies.warn("Rejected malformed remote-data consent challenge with mixed speech disclosure kinds.", {
          challengeId,
        });
        return null;
      }

      dependencies.setConsentSubmitting(challengeId, true);
      let response: RemoteDataConsentResponseOutcome;
      const submission = {
        challengeId,
        challengeDigest: consentState.challenge.challenge_digest,
        decision,
      };
      try {
        response = submissionKind === "narration"
          ? await dependencies.submitNarrationConsentResponse(submission)
          : submissionKind === "microphone"
            ? await dependencies.submitMicrophoneConsentResponse(submission)
            : await dependencies.submitConsentResponse(submission);
      } catch (error) {
        const failure = describeRemoteDataConsentSubmissionFailure(error);
        if (failure.kind === "tool-error") {
          // The Rust consent handler atomically takes the pending transaction before
          // validating or persisting the response. A returned tool error therefore
          // means this dialog can no longer authorize anything and must not remain
          // actionable. Transport errors are different: the command may not have
          // reached Rust, so the bounded challenge remains visible for an explicit
          // retry and any duplicate is still rejected by the backend.
          dependencies.clearConsent(challengeId);
          dependencies.reportGlobalError(
            `${failure.title}. ${failure.message} ${failure.guidance}`,
          );
          try {
            await dependencies.refreshRuntime();
          } catch (refreshError) {
            const refreshFailure = classifyInvokeFailure(refreshError);
            const message = refreshFailure.kind === "tool-error"
              ? refreshFailure.toolError.message
              : refreshFailure.message;
            dependencies.reportGlobalError(
              `The privacy request was rejected, but runtime status could not be refreshed. ${message}`,
            );
          }
        } else {
          dependencies.setConsentError(challengeId, failure);
        }
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
        } else if (response.status === "executed") {
          dependencies.applyExecutionOutcome(response.outcome);
        } else if (response.status === "spoken") {
          dependencies.clearConsent(challengeId);
        } else if (response.status === "listening_started") {
          unwrapToolResult(response.result);
          dependencies.clearConsent(challengeId);
          dependencies.reportGlobalInfo(
            "Microphone privacy permission was accepted. Listening started only after authorization.",
          );
        } else if (response.status === "transcribed") {
          unwrapToolResult(response.result);
          dependencies.clearConsent(challengeId);
        } else if (response.status === "transcribed_and_executed") {
          dependencies.clearConsent(challengeId);
          if (response.result.execution_outcome) {
            dependencies.applyExecutionOutcome(response.result.execution_outcome);
          }
          if (response.result.command_error) {
            dependencies.reportGlobalError(
              `The microphone request was authorized and transcribed, but the spoken command could not be completed. ${response.result.command_error.message}`,
            );
          }
        } else if (response.status === "planner_resume_blocked") {
          dependencies.clearConsent(challengeId);
          dependencies.reportGlobalError(
            "Microphone permission was accepted, but this planner-embedded transcription step cannot safely resume while the executor holds the application lock. No microphone audio was captured or sent. Run the voice input again outside the paused planner step.",
          );
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
  submitNarrationConsentResponse,
  submitMicrophoneConsentResponse,
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
  reportGlobalInfo: (message) => {
    setAppAlertState({ kind: "info", message });
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
): Promise<RemoteDataConsentResponseOutcome | null> {
  return productionController.submitConsentDecision(decision, challengeId);
}
