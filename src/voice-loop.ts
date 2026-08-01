import { appShellStore } from "./store";
import { createRequestId, setPushToTalkState } from "./panel-state-setters";
import { refreshRuntimePanels } from "./refresh-handle";
import { uiStore } from "./ui-store";
import {
  describeConfirmationSubmissionFailure,
  resolveConfirmationResponse,
} from "./planner-orchestration";
import { describePushToTalkFailure } from "./main-errors";
import {
  startListening,
  stopListening,
  transcribeAndExecuteCommand,
} from "./tauri-api";

const PUSH_TO_TALK_RELEASE_CAPTURE_MS = 1;
const CONTINUOUS_LISTEN_CAPTURE_MS = 3_000;
let activePushToTalkSource: "keyboard" | "pointer" | null = null;
let continuousListeningLoopActive = false;

function getPushToTalkState() {
  return appShellStore.getState().panelStates.pushToTalkState;
}

// Report a stop/transcribe failure without asserting a listening state the backend
// never confirmed. The runtime may still be listening after a failed stop, so we
// keep the previous isListening and try to refresh authoritative runtime state; if
// the refresh also fails we surface that explicitly rather than inventing `false`.
async function reportPushToTalkFailureWithoutInventingListeningState(message: string) {
  const previous = getPushToTalkState();
  setPushToTalkState({
    isBusy: false,
    isListening: previous.isListening,
    lastError: `${message} Runtime listening state could not be confirmed yet.`,
  });

  try {
    await refreshRuntimePanels();
  } catch (refreshError: unknown) {
    setPushToTalkState({
      isBusy: false,
      isListening: previous.isListening,
      lastError: `${message} Also failed to refresh runtime state: ${describePushToTalkFailure(
        refreshError,
      )}`,
    });
  }
}

export async function stopContinuousListeningAfterFailure(message: string) {
  const pttState = getPushToTalkState();
  if (!pttState.isListening) {
    setPushToTalkState({
      isBusy: false,
      lastError: message,
    });
    return;
  }

  try {
    const result = await stopListening({
      requestId: createRequestId("continuous-listen-stop"),
    });
    setPushToTalkState({
      enabled: result.listening_state.push_to_talk_enabled,
      isListening: result.listening_state.is_listening,
      isBusy: false,
      lastError: message,
    });
  } catch (error: unknown) {
    const stopFailure = describePushToTalkFailure(error);
    await reportPushToTalkFailureWithoutInventingListeningState(
      `${message} The runtime also failed to stop hands-free listening: ${stopFailure}`,
    );
  }
}

export async function ensureContinuousListeningLoop() {
  const pttState = getPushToTalkState();
  if (continuousListeningLoopActive || !pttState.isListening || pttState.isHolding) {
    return;
  }

  continuousListeningLoopActive = true;

  try {
    while (getPushToTalkState().isListening && !getPushToTalkState().isHolding) {
      setPushToTalkState({
        isBusy: true,
        lastError: null,
      });

      const result = await transcribeAndExecuteCommand({
        requestId: createRequestId("continuous-listen"),
        maxDurationMs: CONTINUOUS_LISTEN_CAPTURE_MS,
        autoStop: false,
      });

      setPushToTalkState({
        enabled: result.transcription.listening_state.push_to_talk_enabled,
        isListening: result.transcription.listening_state.is_listening,
        isBusy: false,
        lastTranscript: result.transcription.transcript ?? getPushToTalkState().lastTranscript,
        lastError: result.command_error?.message ?? null,
      });

      if (result.execution_outcome) {
        uiStore.applyOutcome(result.execution_outcome);
      }

      await refreshRuntimePanels();

      if (!result.transcription.listening_state.is_listening) {
        break;
      }
    }
  } catch (error: unknown) {
    const message = describePushToTalkFailure(error);
    await stopContinuousListeningAfterFailure(message);
    await refreshRuntimePanels();
  } finally {
    continuousListeningLoopActive = false;
  }
}

export async function beginPushToTalk(source: "keyboard" | "pointer") {
  const pttState = getPushToTalkState();
  if (
    !pttState.enabled ||
    pttState.isHolding ||
    pttState.isBusy ||
    pttState.isListening
  ) {
    return;
  }

  activePushToTalkSource = source;
  setPushToTalkState({
    isHolding: true,
    isBusy: true,
    lastError: null,
    lastTranscript: null,
  });

  try {
    const result = await startListening({
      requestId: createRequestId("push-to-talk-start"),
    });
    setPushToTalkState({
      enabled: result.listening_state.push_to_talk_enabled,
      isHolding: true,
      isListening: result.listening_state.is_listening,
      isBusy: false,
    });
    await refreshRuntimePanels();
  } catch (error: unknown) {
    activePushToTalkSource = null;
    setPushToTalkState({
      isHolding: false,
      isListening: false,
      isBusy: false,
      lastError: describePushToTalkFailure(error),
    });
  }
}

export async function cancelPushToTalk() {
  activePushToTalkSource = null;

  const pttState = getPushToTalkState();
  if (!pttState.isHolding && !pttState.isListening) {
    return;
  }

  setPushToTalkState({
    isHolding: false,
    isBusy: true,
    lastError: null,
  });

  try {
    const result = await stopListening({
      requestId: createRequestId("push-to-talk-cancel"),
    });
    setPushToTalkState({
      enabled: result.listening_state.push_to_talk_enabled,
      isListening: result.listening_state.is_listening,
      isBusy: false,
    });
    await refreshRuntimePanels();
  } catch (error: unknown) {
    await reportPushToTalkFailureWithoutInventingListeningState(
      describePushToTalkFailure(error),
    );
  }
}

export async function releasePushToTalk(source: "keyboard" | "pointer") {
  if (!getPushToTalkState().isHolding || activePushToTalkSource !== source) {
    return;
  }

  activePushToTalkSource = null;
  setPushToTalkState({
    isHolding: false,
    isBusy: true,
    lastError: null,
  });

  try {
    const result = await transcribeAndExecuteCommand({
      requestId: createRequestId("push-to-talk-transcribe"),
      maxDurationMs: PUSH_TO_TALK_RELEASE_CAPTURE_MS,
      autoStop: true,
    });
    const {
      transcription,
      command_error: commandError,
      execution_outcome: executionOutcome,
    } = result;
    setPushToTalkState({
      enabled: transcription.listening_state.push_to_talk_enabled,
      isListening: transcription.listening_state.is_listening,
      isBusy: false,
      lastTranscript: transcription.transcript,
      lastError: commandError?.message ?? null,
    });

    if (!transcription.transcript) {
      await refreshRuntimePanels();
      return;
    }

    if (commandError) {
      await refreshRuntimePanels();
      return;
    }

    if (executionOutcome) {
      uiStore.applyOutcome(executionOutcome);
    }
    await refreshRuntimePanels();
    if (getPushToTalkState().isListening && !getPushToTalkState().isHolding) {
      void ensureContinuousListeningLoop();
    }
  } catch (error: unknown) {
    await reportPushToTalkFailureWithoutInventingListeningState(
      describePushToTalkFailure(error),
    );
  }
}

export function submitConfirmationAction(action: "approve" | "reject", confirmationId: string) {
  const confirmationState = uiStore.getState().confirmation;
  if (confirmationState.kind !== "awaiting-confirmation" || confirmationState.isSubmitting) {
    // Duplicate click, or no confirmation is awaiting: safe to ignore, but log it so
    // the no-op is debuggable rather than silently vanishing.
    console.warn("Ignored confirmation submission with no active awaiting confirmation.", {
      submittedConfirmationId: confirmationId,
      confirmationKind: confirmationState.kind,
    });
    return;
  }

  if (confirmationId !== confirmationState.confirmationId) {
    // A different confirmation is awaiting than the one clicked (a stale button):
    // surface a visible error instead of disappearing silently, since a blind user
    // gets no other feedback that the click was dropped.
    uiStore.setConfirmationError(confirmationState.confirmationId, {
      kind: "transport-error",
      title: "Confirmation no longer active",
      message:
        "That confirmation is no longer active. Review the current confirmation before approving.",
      guidance: "Review the current confirmation, then approve or reject that one.",
    });
    console.warn("Ignored stale confirmation response.", {
      submittedConfirmationId: confirmationId,
      activeConfirmationId: confirmationState.confirmationId,
    });
    return;
  }

  const confirmed = action === "approve";
  uiStore.setConfirmationSubmitting(confirmationId, true);

  void resolveConfirmationResponse(
    {
      confirmationId,
      confirmationDigest: confirmationState.confirmationDigest,
      confirmed,
      timedOut: false,
    },
    uiStore,
  )
    .then(async () => {
      await refreshRuntimePanels();
    })
    .catch((error: unknown) => {
      uiStore.setConfirmationError(confirmationId, describeConfirmationSubmissionFailure(error));
      console.error("Failed to submit confirmation response.", error);
    });
}
