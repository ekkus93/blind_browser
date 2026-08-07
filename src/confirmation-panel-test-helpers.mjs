import { renderToStaticMarkup } from "react-dom/server";

import {
  renderAudioControlsPanelNode,
  renderConfirmationPanelNode,
  renderPushToTalkPanelNode,
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
  statusPanelStateFromAgentState,
  renderUrlInputPanelNode,
  renderVoiceStatusStripNode,
} from "./confirmation-panel.tsx";

export { statusPanelStateFromAgentState, renderVoiceStatusStripNode };

// Renders a React node tree to static HTML using React's own server renderer,
// so components that call hooks (e.g. useSyncExternalStore, as
// WorkspaceDecisionPanels does) render correctly instead of silently
// producing empty output. React's renderer also handles controlled
// <select value="..."> -> matching <option selected> mapping natively.
export function renderNodeMarkup(node) {
  return renderToStaticMarkup(node);
}

export function renderConfirmationPanel(state) {
  return renderNodeMarkup(renderConfirmationPanelNode(state));
}

export function renderPushToTalkPanel(state) {
  return renderNodeMarkup(renderPushToTalkPanelNode(state));
}

export function renderAudioControlsPanel(state) {
  return renderNodeMarkup(renderAudioControlsPanelNode(state));
}

export function renderSettingsAsrProviderPanel(state) {
  return renderNodeMarkup(renderSettingsAsrProviderPanelNode(state));
}

export function renderSettingsConfirmationPanel(state) {
  return renderNodeMarkup(renderSettingsConfirmationPanelNode(state));
}

export function renderSettingsGuidancePanel(state) {
  return renderNodeMarkup(renderSettingsGuidancePanelNode(state));
}

export function renderSettingsLocalAsrModelPanel(state) {
  return renderNodeMarkup(renderSettingsLocalAsrModelPanelNode(state));
}

export function renderSettingsLocalTtsModelPanel(state) {
  return renderNodeMarkup(renderSettingsLocalTtsModelPanelNode(state));
}

export function renderSettingsModelManagementPanel(state) {
  return renderNodeMarkup(renderSettingsModelManagementPanelNode(state));
}

export function renderSettingsOcrThresholdPanel(state) {
  return renderNodeMarkup(renderSettingsOcrThresholdPanelNode(state));
}

export function renderSettingsProviderFailoverPanel(state) {
  return renderNodeMarkup(renderSettingsProviderFailoverPanelNode(state));
}

export function renderSettingsRemoteAsrPanel(state) {
  return renderNodeMarkup(renderSettingsRemoteAsrPanelNode(state));
}

export function renderSettingsRemotePlannerPanel(state) {
  return renderNodeMarkup(renderSettingsRemotePlannerPanelNode(state));
}

export function renderSettingsRemoteTtsPanel(state) {
  return renderNodeMarkup(renderSettingsRemoteTtsPanelNode(state));
}

export function renderSettingsTtsProviderPanel(state) {
  return renderNodeMarkup(renderSettingsTtsProviderPanelNode(state));
}

export function renderSettingsTtsModelPanel(state) {
  return renderNodeMarkup(renderSettingsTtsModelPanelNode(state));
}

export function renderSettingsTtsVoicePanel(state) {
  return renderNodeMarkup(renderSettingsTtsVoicePanelNode(state));
}

export function renderStatusPanel(state) {
  return renderNodeMarkup(renderStatusPanelNode(state));
}

export function renderUrlInputPanel(state) {
  return renderNodeMarkup(renderUrlInputPanelNode(state));
}

export function renderFixtures() {
  const nonRetryableHtml = renderConfirmationPanel({
    kind: "awaiting-confirmation",
    isSubmitting: false,
    submissionError: {
      kind: "tool-error",
      title: "Runtime cannot complete this request",
      message: "The backend rejected the action.",
      guidance: "Review the planner state before trying again.",
      retryable: false,
      code: "confirmation_denied",
    },
    confirmationId: "confirmation-1",
    confirmationDigest: "digest-1",
    promptText: "Submit the form?",
    requestId: "request-1",
    selectedSkills: ["form_submit"],
    nextStepId: "step-2",
    queuedStepIds: ["step-2"],
  });

  const retryableHtml = renderConfirmationPanel({
    kind: "awaiting-confirmation",
    isSubmitting: false,
    submissionError: {
      kind: "tool-error",
      title: "Runtime rejected the request",
      message: "The backend is temporarily unavailable.",
      guidance: "Review the runtime state and try again.",
      retryable: true,
      code: "runtime_busy",
    },
    confirmationId: "confirmation-2",
    confirmationDigest: "digest-2",
    promptText: "Submit the form?",
    requestId: "request-2",
    selectedSkills: ["form_submit"],
    nextStepId: "step-2",
    queuedStepIds: ["step-2"],
  });

  const transportHtml = renderConfirmationPanel({
    kind: "awaiting-confirmation",
    isSubmitting: false,
    submissionError: {
      kind: "transport-error",
      title: "Connection problem",
      message: "The app could not reach the confirmation command.",
      guidance: "Check that the runtime is still running, then try again.",
    },
    confirmationId: "confirmation-3",
    confirmationDigest: "digest-3",
    promptText: "Submit the form?",
    requestId: "request-3",
    selectedSkills: ["form_submit"],
    nextStepId: "step-2",
    queuedStepIds: ["step-2"],
  });

  return {
    nonRetryableHtml,
    retryableHtml,
    transportHtml,
  };
}
