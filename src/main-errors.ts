import type { SettingsGuidancePanelState } from "./confirmation-panel";
import type { PanelStates } from "./panel-state";
import { classifyInvokeFailure } from "./tauri-api";

function describeInvokeFailureMessage(error: unknown): string {
  const failure = classifyInvokeFailure(error);
  if (failure.kind === "tool-error") {
    return failure.toolError.message;
  }

  return failure.message;
}

export function describePushToTalkFailure(error: unknown): string {
  const failure = classifyInvokeFailure(error);
  if (failure.kind === "tool-error") {
    switch (failure.toolError.code) {
      case "asr_model_unavailable":
        return "Voice input setup is incomplete. Switch ASR to OpenAI or finish the local Whisper setup.";
      case "asr_secret_unavailable":
        return "Voice input needs an OpenAI API key before it can start.";
      default:
        return failure.toolError.message;
    }
  }

  return failure.message;
}

export function describeAudioControlFailure(error: unknown): string {
  return describeInvokeFailureMessage(error);
}

export function describeUrlInputFailure(error: unknown): string {
  return describeInvokeFailureMessage(error);
}

export function describeScopedRuntimeRefreshFailure(
  scope: "runtime state" | "model management",
  error: unknown,
): string {
  return `${scope} refresh failed: ${describeAudioControlFailure(error)}`;
}

export function describePlannerBlockedMessage(defaultMessage: string, userMessage: string | null): string {
  const trimmed = userMessage?.trim();
  return trimmed && trimmed.length > 0 ? trimmed : defaultMessage;
}

export function guidanceStateForErrorMessage(message: string | null): SettingsGuidancePanelState | null {
  const normalized = message?.trim().toLowerCase();
  if (!normalized) {
    return null;
  }

  if (
    normalized.includes("local tts model path")
    || normalized.includes("failed to load the local tts model")
    || normalized.includes("tts local profile")
  ) {
    return {
      title: "Model setup needs attention",
      message: "The current local TTS setup is unavailable. Review the TTS provider or TTS model settings below.",
      actions: [
        { label: "Review TTS provider", targetId: "settings-tts-provider-control" },
        { label: "Review TTS model", targetId: "settings-tts-model-control" },
        { label: "Open model management", targetId: "settings-model-management-title" },
        { label: "Review local TTS reference", targetId: "settings-local-tts-model-title" },
      ],
    };
  }

  if (
    normalized.includes("local asr model path")
    || normalized.includes("failed to load the local asr model")
    || normalized.includes("asr local profile")
    || normalized.includes("voice input setup is incomplete")
  ) {
    return {
      title: "Model setup needs attention",
      message: "The current local ASR setup is unavailable. Review the ASR provider and local model settings below.",
      actions: [
        { label: "Review ASR provider", targetId: "settings-asr-provider-control" },
        { label: "Open model management", targetId: "settings-model-management-title" },
        { label: "Review local ASR reference", targetId: "settings-local-asr-model-title" },
      ],
    };
  }

  if (
    normalized.includes("planner profile")
    || normalized.includes("planner_profile_unavailable")
    || normalized.includes("configured remote planner profile")
  ) {
    return {
      title: "Planner setup needs attention",
      message: "Command interpretation is unavailable until a remote planner profile is configured. Review the planner provider and remote planner settings below.",
      actions: [
        { label: "Review planner provider", targetId: "settings-planner-provider-control" },
        { label: "Review planner API reference", targetId: "settings-remote-planner-title" },
      ],
    };
  }

  if (
    normalized.includes("planner api key")
    || normalized.includes("planner_secret_unavailable")
    || normalized.includes("remote planner secret")
    || normalized.includes("keyring secret")
  ) {
    return {
      title: "Remote planner secret needs attention",
      message: "The current remote planner secret is unavailable. Review the planner API reference and save a replacement key below.",
      actions: [
        { label: "Review planner API reference", targetId: "settings-remote-planner-title" },
        { label: "Enter planner API key", targetId: "settings-remote-planner-api-key-input" },
      ],
    };
  }

  if (
    normalized.includes("tts api key")
    || normalized.includes("tts_secret_unavailable")
    || normalized.includes("remote tts secret")
  ) {
    return {
      title: "Remote TTS secret needs attention",
      message: "The current remote TTS secret is unavailable. Review the remote TTS profile and save a replacement key below.",
      actions: [
        { label: "Review remote TTS profile", targetId: "settings-remote-tts-title" },
        { label: "Enter remote TTS API key", targetId: "settings-remote-tts-api-key-input" },
      ],
    };
  }

  if (
    normalized.includes("asr api key")
    || normalized.includes("asr_secret_unavailable")
    || normalized.includes("remote asr secret")
    || normalized.includes("voice input needs an openai api key")
  ) {
    return {
      title: "Remote ASR secret needs attention",
      message: "The current remote ASR secret is unavailable. Review the remote ASR profile and save a replacement key below.",
      actions: [
        { label: "Review remote ASR profile", targetId: "settings-remote-asr-title" },
        { label: "Enter remote ASR API key", targetId: "settings-remote-asr-api-key-input" },
      ],
    };
  }

  return null;
}

export function currentSettingsGuidanceState(panelStates: PanelStates): SettingsGuidancePanelState | null {
  return (
    guidanceStateForErrorMessage(panelStates.pushToTalkState.lastError)
    ?? guidanceStateForErrorMessage(panelStates.urlInputPanelState.error)
    ?? guidanceStateForErrorMessage(panelStates.statusPanelState.error)
    ?? guidanceStateForErrorMessage(panelStates.ttsProviderPanelState.error)
    ?? guidanceStateForErrorMessage(panelStates.ttsModelPanelState.error)
    ?? guidanceStateForErrorMessage(panelStates.ttsVoicePanelState.error)
    ?? guidanceStateForErrorMessage(panelStates.asrProviderPanelState.error)
    ?? guidanceStateForErrorMessage(panelStates.remotePlannerPanelState.error)
    ?? guidanceStateForErrorMessage(panelStates.remoteTtsPanelState.error)
    ?? guidanceStateForErrorMessage(panelStates.remoteAsrPanelState.error)
    ?? guidanceStateForErrorMessage(panelStates.modelManagementPanelState.error)
  );
}
