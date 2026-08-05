import { appShellStore } from "./store";
import {
  applyExecutionOutcome as applyExecutionUiOutcome,
  clearRemoteDataConsent as clearRemoteDataConsentUi,
  setConfirmationError as setConfirmationUiError,
  setConfirmationSubmitting as setConfirmationUiSubmitting,
  setExecutionUiState,
  setRemoteDataConsentError as setRemoteDataConsentUiError,
  setRemoteDataConsentSubmitting as setRemoteDataConsentUiSubmitting,
} from "./app-shell-store";
import { type ExecutionUiStore } from "./planner-orchestration";

export const uiStore: ExecutionUiStore = {
  getState: () => appShellStore.getState().executionUi,
  setState: (nextState) => {
    appShellStore.dispatch(setExecutionUiState(nextState));
  },
  applyOutcome: (outcome) => {
    appShellStore.dispatch(applyExecutionUiOutcome(outcome));
    return appShellStore.getState().executionUi;
  },
  setConfirmationSubmitting: (confirmationId, isSubmitting) => {
    appShellStore.dispatch(setConfirmationUiSubmitting({ confirmationId, isSubmitting }));
    return appShellStore.getState().executionUi;
  },
  setConfirmationError: (confirmationId, submissionError) => {
    appShellStore.dispatch(setConfirmationUiError({ confirmationId, submissionError }));
    return appShellStore.getState().executionUi;
  },
  setRemoteDataConsentSubmitting: (challengeId, isSubmitting) => {
    appShellStore.dispatch(setRemoteDataConsentUiSubmitting({ challengeId, isSubmitting }));
    return appShellStore.getState().executionUi;
  },
  setRemoteDataConsentError: (challengeId, submissionError) => {
    appShellStore.dispatch(setRemoteDataConsentUiError({ challengeId, submissionError }));
    return appShellStore.getState().executionUi;
  },
  clearRemoteDataConsent: (challengeId) => {
    appShellStore.dispatch(clearRemoteDataConsentUi({ challengeId }));
    return appShellStore.getState().executionUi;
  },
  subscribe: (listener) =>
    appShellStore.subscribe(() => {
      listener(appShellStore.getState().executionUi);
    }),
};
