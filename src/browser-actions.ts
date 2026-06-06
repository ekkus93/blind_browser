import { appShellStore } from "./store";
import { createRequestId, setUrlInputPanelState } from "./panel-state-setters";
import { refreshRuntimePanels } from "./refresh-handle";
import { uiStore } from "./ui-store";
import {
  describePlannerBlockedMessage,
  describeUrlInputFailure,
} from "./main-errors";
import { runPlannerExecution } from "./planner-orchestration";
import { openUrl, resolveCommand } from "./tauri-api";
import type { UrlInputPanelState } from "./confirmation-panel";

function getUrlInputPanelState() {
  return appShellStore.getState().panelStates.urlInputPanelState;
}

export function isUrlInputActionBusy(): boolean {
  const s = getUrlInputPanelState();
  return s.isOpening || s.isReading || s.isStopping || s.isAdvancing || s.isRewinding;
}

export async function executeUrlPanelPlannerCommand(input: {
  busyState: Partial<UrlInputPanelState>;
  clearState: Partial<UrlInputPanelState>;
  requestPrefix: string;
  transcript: string;
  blockedMessage: string;
  completeMessage: string;
}) {
  if (isUrlInputActionBusy()) {
    return;
  }

  const previousState = getUrlInputPanelState();
  setUrlInputPanelState({
    ...input.busyState,
    error: null,
  });

  try {
    const requestId = createRequestId(input.requestPrefix);
    const plannerOutput = await resolveCommand(requestId, input.transcript);
    if (plannerOutput.status === "Blocked") {
      setUrlInputPanelState({
        ...previousState,
        ...input.clearState,
        error: describePlannerBlockedMessage(input.blockedMessage, plannerOutput.user_message),
      });
      return;
    }

    if (plannerOutput.status === "Complete") {
      setUrlInputPanelState({
        ...previousState,
        ...input.clearState,
        error: describePlannerBlockedMessage(input.completeMessage, plannerOutput.user_message),
      });
      return;
    }

    await runPlannerExecution(requestId, plannerOutput, uiStore);
    setUrlInputPanelState({
      ...previousState,
      ...input.clearState,
      error: null,
    });
    await refreshRuntimePanels();
  } catch (error: unknown) {
    setUrlInputPanelState({
      ...previousState,
      ...input.clearState,
      error: describeUrlInputFailure(error),
    });
  }
}

export async function openDraftUrl() {
  if (isUrlInputActionBusy()) {
    return;
  }

  const panelState = getUrlInputPanelState();
  const nextUrl = panelState.draftValue.trim();
  if (nextUrl.length === 0) {
    setUrlInputPanelState({
      error: "Enter a URL before opening a page.",
    });
    return;
  }

  const previousState = panelState;
  setUrlInputPanelState({
    isOpening: true,
    error: null,
  });

  try {
    const result = await openUrl({
      requestId: createRequestId("open-url"),
      url: nextUrl,
    });
    setUrlInputPanelState({
      draftValue: result.final_url,
      currentUrl: result.final_url,
      hasUnsubmittedChanges: false,
      isOpening: false,
      error: null,
    });
    await refreshRuntimePanels();
  } catch (error: unknown) {
    setUrlInputPanelState({
      ...previousState,
      isOpening: false,
      error: describeUrlInputFailure(error),
    });
  }
}

export async function readCurrentPage() {
  await executeUrlPanelPlannerCommand({
    busyState: { isReading: true },
    clearState: { isReading: false },
    requestPrefix: "read-page",
    transcript: "read page",
    blockedMessage: "The runtime could not start reading the current page.",
    completeMessage: "The runtime did not need to run any reading steps.",
  });
}

export async function stopCurrentReading() {
  await executeUrlPanelPlannerCommand({
    busyState: { isStopping: true },
    clearState: { isStopping: false },
    requestPrefix: "stop-reading",
    transcript: "stop reading",
    blockedMessage: "The runtime could not stop the current reading session.",
    completeMessage: "The runtime did not need to stop any current reading.",
  });
}

export async function readNextRegion() {
  await executeUrlPanelPlannerCommand({
    busyState: { isAdvancing: true },
    clearState: { isAdvancing: false },
    requestPrefix: "read-next",
    transcript: "continue reading",
    blockedMessage: "The runtime could not move to the next reading region.",
    completeMessage: "The runtime did not need to move to another reading region.",
  });
}

export async function readPreviousRegion() {
  await executeUrlPanelPlannerCommand({
    busyState: { isRewinding: true },
    clearState: { isRewinding: false },
    requestPrefix: "read-previous",
    transcript: "previous section",
    blockedMessage: "The runtime could not move to the previous reading region.",
    completeMessage: "The runtime did not need to move to a previous reading region.",
  });
}
