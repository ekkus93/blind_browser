import { appShellStore } from "./store";
import {
  applyExecutionOutcome as applyExecutionUiOutcome,
  setConfirmationError as setConfirmationUiError,
  setConfirmationSubmitting as setConfirmationUiSubmitting,
  setExecutionUiState,
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
  subscribe: (listener) =>
    appShellStore.subscribe(() => {
      listener(appShellStore.getState().executionUi);
    }),
};
