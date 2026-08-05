import { configureStore, createSlice, type PayloadAction } from "@reduxjs/toolkit";

import type { AppView, SettingsView } from "./app-shell.ts";
import type { PanelStates } from "./panel-state.ts";
import { createInitialPanelStates } from "./panel-state.ts";
import {
  applyExecutionOutcomeToUiState,
  createInitialExecutionUiState,
  type ConfirmationSubmissionFailure,
  type ExecutionUiState,
} from "./planner-orchestration.ts";
import {
  createInitialRemotePlannerPrivacyState,
  remotePlannerPrivacyReducer,
  type RemotePlannerPrivacyState,
} from "./remote-planner-privacy-state.ts";
import type { ExecutionOutcome } from "./tauri-api.ts";

interface AppShellViewState {
  appView: AppView;
  settingsView: SettingsView;
}

export interface AppShellState {
  shellView: AppShellViewState;
  panelStates: PanelStates;
  executionUi: ExecutionUiState;
  remotePlannerPrivacy: RemotePlannerPrivacyState;
}

const initialViewState: AppShellViewState = {
  appView: "workspace",
  settingsView: "overview",
};

const initialPanelStates = createInitialPanelStates();
const initialExecutionUiState = createInitialExecutionUiState();
const initialRemotePlannerPrivacyState = createInitialRemotePlannerPrivacyState();

const appShellViewSlice = createSlice({
  name: "appShellView",
  initialState: initialViewState,
  reducers: {
    setAppView(state, action: PayloadAction<AppView>) {
      state.appView = action.payload;
    },
    setSettingsView(state, action: PayloadAction<SettingsView>) {
      state.settingsView = action.payload;
    },
  },
});

const panelStateSlice = createSlice({
  name: "panelStates",
  initialState: initialPanelStates,
  reducers: {
    setAppAlertPanelState(state, action: PayloadAction<Partial<PanelStates["appAlertState"]>>) {
      Object.assign(state.appAlertState, action.payload);
    },
    setPushToTalkPanelState(state, action: PayloadAction<Partial<PanelStates["pushToTalkState"]>>) {
      Object.assign(state.pushToTalkState, action.payload);
    },
    setAudioControlsPanelState(state, action: PayloadAction<Partial<PanelStates["audioControlsState"]>>) {
      Object.assign(state.audioControlsState, action.payload);
    },
    setRemotePlannerPanelState(state, action: PayloadAction<Partial<PanelStates["remotePlannerPanelState"]>>) {
      Object.assign(state.remotePlannerPanelState, action.payload);
    },
    setProviderFailoverPanelState(state, action: PayloadAction<Partial<PanelStates["providerFailoverPanelState"]>>) {
      Object.assign(state.providerFailoverPanelState, action.payload);
    },
    setConfirmationSettingsPanelState(state, action: PayloadAction<Partial<PanelStates["confirmationSettingsPanelState"]>>) {
      Object.assign(state.confirmationSettingsPanelState, action.payload);
    },
    setOcrThresholdSettingsPanelState(state, action: PayloadAction<Partial<PanelStates["ocrThresholdSettingsPanelState"]>>) {
      Object.assign(state.ocrThresholdSettingsPanelState, action.payload);
    },
    setAsrProviderPanelState(state, action: PayloadAction<Partial<PanelStates["asrProviderPanelState"]>>) {
      Object.assign(state.asrProviderPanelState, action.payload);
    },
    setLocalAsrModelPanelState(state, action: PayloadAction<Partial<PanelStates["localAsrModelPanelState"]>>) {
      Object.assign(state.localAsrModelPanelState, action.payload);
    },
    setModelManagementPanelState(state, action: PayloadAction<Partial<PanelStates["modelManagementPanelState"]>>) {
      Object.assign(state.modelManagementPanelState, action.payload);
    },
    setRemoteAsrPanelState(state, action: PayloadAction<Partial<PanelStates["remoteAsrPanelState"]>>) {
      Object.assign(state.remoteAsrPanelState, action.payload);
    },
    setTtsProviderPanelState(state, action: PayloadAction<Partial<PanelStates["ttsProviderPanelState"]>>) {
      Object.assign(state.ttsProviderPanelState, action.payload);
    },
    setTtsModelPanelState(state, action: PayloadAction<Partial<PanelStates["ttsModelPanelState"]>>) {
      Object.assign(state.ttsModelPanelState, action.payload);
    },
    setLocalTtsModelPanelState(state, action: PayloadAction<Partial<PanelStates["localTtsModelPanelState"]>>) {
      Object.assign(state.localTtsModelPanelState, action.payload);
    },
    setRemoteTtsPanelState(state, action: PayloadAction<Partial<PanelStates["remoteTtsPanelState"]>>) {
      Object.assign(state.remoteTtsPanelState, action.payload);
    },
    setTtsVoicePanelState(state, action: PayloadAction<Partial<PanelStates["ttsVoicePanelState"]>>) {
      Object.assign(state.ttsVoicePanelState, action.payload);
    },
    setStatusPanelState(state, action: PayloadAction<Partial<PanelStates["statusPanelState"]>>) {
      Object.assign(state.statusPanelState, action.payload);
    },
    setUrlInputPanelState(state, action: PayloadAction<Partial<PanelStates["urlInputPanelState"]>>) {
      Object.assign(state.urlInputPanelState, action.payload);
    },
  },
});

const executionUiSlice = createSlice({
  name: "executionUi",
  initialState: initialExecutionUiState,
  reducers: {
    setExecutionUiState(_state, action: PayloadAction<ExecutionUiState>) {
      return action.payload;
    },
    applyExecutionOutcome(_state, action: PayloadAction<ExecutionOutcome>) {
      return applyExecutionOutcomeToUiState(action.payload);
    },
    setConfirmationSubmitting(
      state,
      action: PayloadAction<{ confirmationId: string; isSubmitting: boolean }>,
    ) {
      if (
        state.confirmation.kind !== "awaiting-confirmation"
        || state.confirmation.confirmationId !== action.payload.confirmationId
      ) {
        return state;
      }

      return {
        ...state,
        confirmation: {
          ...state.confirmation,
          isSubmitting: action.payload.isSubmitting,
          submissionError: action.payload.isSubmitting ? null : state.confirmation.submissionError,
        },
      };
    },
    setConfirmationError(
      state,
      action: PayloadAction<{
        confirmationId: string;
        submissionError: ConfirmationSubmissionFailure | null;
      }>,
    ) {
      if (
        state.confirmation.kind !== "awaiting-confirmation"
        || state.confirmation.confirmationId !== action.payload.confirmationId
      ) {
        return state;
      }

      return {
        ...state,
        confirmation: {
          ...state.confirmation,
          isSubmitting: false,
          submissionError: action.payload.submissionError,
        },
      };
    },
  },
});

export const { setAppView, setSettingsView } = appShellViewSlice.actions;
export const {
  setAppAlertPanelState,
  setPushToTalkPanelState,
  setAudioControlsPanelState,
  setRemotePlannerPanelState,
  setProviderFailoverPanelState,
  setConfirmationSettingsPanelState,
  setOcrThresholdSettingsPanelState,
  setAsrProviderPanelState,
  setLocalAsrModelPanelState,
  setModelManagementPanelState,
  setRemoteAsrPanelState,
  setTtsProviderPanelState,
  setTtsModelPanelState,
  setLocalTtsModelPanelState,
  setRemoteTtsPanelState,
  setTtsVoicePanelState,
  setStatusPanelState,
  setUrlInputPanelState,
} = panelStateSlice.actions;
export const {
  setExecutionUiState,
  applyExecutionOutcome,
  setConfirmationSubmitting,
  setConfirmationError,
} = executionUiSlice.actions;

export function createAppShellStore(preloadedState?: Partial<AppShellState>) {
  return configureStore({
    reducer: {
      shellView: appShellViewSlice.reducer,
      panelStates: panelStateSlice.reducer,
      executionUi: executionUiSlice.reducer,
      remotePlannerPrivacy: remotePlannerPrivacyReducer,
    },
    preloadedState: {
      shellView: {
        ...initialViewState,
        ...preloadedState?.shellView,
      },
      panelStates: {
        ...initialPanelStates,
        ...preloadedState?.panelStates,
      },
      executionUi: preloadedState?.executionUi ?? initialExecutionUiState,
      remotePlannerPrivacy: {
        ...initialRemotePlannerPrivacyState,
        ...preloadedState?.remotePlannerPrivacy,
      },
    },
  });
}

export type AppShellStore = ReturnType<typeof createAppShellStore>;
