import {
  Fragment,
  useSyncExternalStore,
  type ReactNode,
} from "react";

import { appShellStore } from "./store.ts";
import {
  setAppView,
  setSettingsView,
} from "./panel-state-setters.ts";
import {
  submitRemotePlannerConsentDecision,
} from "./remote-planner-privacy-controller.ts";
import {
  renderRemotePlannerPrivacyWorkspaceNode,
} from "./remote-planner-privacy-ui.tsx";
import {
  renderConfirmationPanelNode as renderActionConfirmationPanelNode,
} from "./confirmation-panels/confirmation.tsx";

export type {
  AppAlertState,
  AudioControlsPanelState,
  AsrProviderPanelState,
  ConfirmationSettingsPanelState,
  LocalAsrModelPanelState,
  LocalTtsModelPanelState,
  ModelManagementPanelState,
  OcrThresholdSettingsPanelState,
  ProviderFailoverPanelState,
  PushToTalkPanelState,
  RemoteAsrPanelState,
  RemotePlannerPanelState,
  RemoteTtsPanelState,
  SettingsGuidancePanelAction,
  SettingsGuidancePanelState,
  StatusPanelAgentStateLike,
  StatusPanelState,
  TtsModelPanelState,
  TtsProviderPanelState,
  TtsVoicePanelState,
  UrlInputPanelState,
} from "./panel-types.ts";
export type { VoiceStatusStripState } from "./confirmation-panels/push-to-talk.tsx";
export {
  renderPushToTalkPanelNode,
  renderVoiceStatusStripNode,
} from "./confirmation-panels/push-to-talk.tsx";
export {
  renderAudioControlsPanelNode,
  renderSettingsAsrProviderPanelNode,
  renderSettingsConfirmationPanelNode,
  renderSettingsGuidancePanelNode,
  renderSettingsLocalAsrModelPanelNode,
  renderSettingsLocalTtsModelPanelNode,
  renderSettingsModelManagementPanelNode,
  renderSettingsOcrThresholdPanelNode,
  renderSettingsProviderFailoverPanelNode,
  renderSettingsRemoteAsrPanelNode,
  renderSettingsRemotePlannerPanelNode,
  renderSettingsRemoteTtsPanelNode,
  renderSettingsTtsProviderPanelNode,
  renderSettingsTtsModelPanelNode,
  renderSettingsTtsVoicePanelNode,
  renderStatusPanelNode,
  renderUrlInputPanelNode,
  statusPanelStateFromAgentState,
} from "./settings-status-panels.ts";

type ActionConfirmationState = Parameters<typeof renderActionConfirmationPanelNode>[0];
type ActionConfirmationHandlers = Parameters<typeof renderActionConfirmationPanelNode>[1];

function WorkspaceDecisionPanels(props: {
  confirmationState: ActionConfirmationState;
  confirmationHandlers?: ActionConfirmationHandlers;
}) {
  const state = useSyncExternalStore(
    appShellStore.subscribe,
    appShellStore.getState,
    appShellStore.getState,
  );

  return (
    <Fragment>
      {renderRemotePlannerPrivacyWorkspaceNode(
        state.remotePlannerPrivacy,
        state.executionUi.remoteDataConsent,
        {
          onOpenSettings: () => {
            setSettingsView("planner");
            setAppView("settings");
          },
          onConsentDecision: (decision, challengeId) => {
            void submitRemotePlannerConsentDecision(decision, challengeId);
          },
        },
      )}
      {renderActionConfirmationPanelNode(
        props.confirmationState,
        props.confirmationHandlers,
      )}
    </Fragment>
  );
}

export function renderConfirmationPanelNode(
  confirmationState: ActionConfirmationState,
  handlers?: ActionConfirmationHandlers,
): ReactNode {
  return (
    <WorkspaceDecisionPanels
      confirmationState={confirmationState}
      confirmationHandlers={handlers}
    />
  );
}
