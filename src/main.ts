import "./styles.css";

import { renderConfirmationPanel, renderPushToTalkPanel, type PushToTalkPanelState } from "./confirmation-panel";
import {
  createExecutionUiStore,
  describeConfirmationSubmissionFailure,
  runPlannerExecution,
  resolveConfirmationResponse,
  type ExecutionUiState,
} from "./planner-orchestration";
import {
  classifyInvokeFailure,
  resolveCommand,
  startListening,
  stopListening,
  transcribeCommand,
} from "./tauri-api";

export {
  applyExecutionOutcomeToUiState,
  createExecutionUiStore,
  createInitialExecutionUiState,
  executePlannerOutput,
  isAwaitingConfirmationOutcome,
  runPlannerExecution,
  resolveConfirmationResponse,
  submitConfirmationResponse,
  type ConfirmActionData,
  type ConfirmActionResolution,
  type ConfirmationUiState,
  type ConfirmActionResponseInput,
  type ExecutionUiState,
  type ExecutionOutcome,
  type PlannerOutput,
  type ToolError,
} from "./planner-orchestration";

export {
  executePlannerOutput as invokeExecutePlannerOutput,
  resolveCommand as invokeResolveCommand,
  startListening as invokeStartListening,
  stopListening as invokeStopListening,
  submitConfirmationResponse as invokeSubmitConfirmationResponse,
  transcribeCommand as invokeTranscribeCommand,
} from "./tauri-api";

const app = document.querySelector<HTMLDivElement>("#app");
const uiStore = createExecutionUiStore();
const PUSH_TO_TALK_RELEASE_CAPTURE_MS = 1;
let currentExecutionUiState = uiStore.getState();
let pushToTalkState: PushToTalkPanelState = createInitialPushToTalkState();
let activePushToTalkSource: "keyboard" | "pointer" | null = null;

if (!app) {
  throw new Error("App root element was not found.");
}

function createInitialPushToTalkState(): PushToTalkPanelState {
  return {
    enabled: true,
    isHolding: false,
    isListening: false,
    isBusy: false,
    lastTranscript: null,
    lastError: null,
  };
}

const renderApp = (uiState: ExecutionUiState, pushToTalk: PushToTalkPanelState) => {
  app.innerHTML = `
    <main class="shell">
      <section class="hero">
        <p class="eyebrow">Phase 0 scaffold</p>
        <h1>Voice-first browser workspace</h1>
        <p class="lede">
          Initial Tauri shell for the blind_browser runtime. This scaffold keeps the
          frontend thin while the Rust tool layer, planner contracts, and provider
          configuration are established under src-tauri.
        </p>
      </section>

      <section class="panels" aria-label="Application sections">
        <article class="panel">
          <h2>Current focus</h2>
          <p>Deterministic tool schemas, provider config models, and runtime state live in Rust.</p>
        </article>
        <article class="panel">
          <h2>Next phases</h2>
          <p>Browser control, extraction, narration, ASR, TTS, and planner execution follow this baseline.</p>
        </article>
        <article class="panel">
          <h2>UI stance</h2>
          <p>The UI remains intentionally light so voice-first behavior stays the primary product path.</p>
        </article>
        <article class="panel">
          <h2>Confirmation flow</h2>
          <p>
            Frontend orchestration now opens a dedicated confirmation UI state whenever a planner
            execution returns <strong>AwaitingConfirmation</strong>.
          </p>
        </article>
      </section>

      ${renderPushToTalkPanel(pushToTalk)}
      ${renderConfirmationPanel(uiState.confirmation)}
    </main>
  `;
};

function rerender() {
  renderApp(currentExecutionUiState, pushToTalkState);
}

function setPushToTalkState(nextState: Partial<PushToTalkPanelState>) {
  pushToTalkState = {
    ...pushToTalkState,
    ...nextState,
  };
  rerender();
}

function createRequestId(prefix: string): string {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return `${prefix}-${crypto.randomUUID()}`;
  }

  return `${prefix}-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }

  return (
    target.isContentEditable ||
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement
  );
}

function isPushToTalkKeyEvent(event: KeyboardEvent): boolean {
  return (
    event.code === "Space" &&
    !event.repeat &&
    !event.altKey &&
    !event.ctrlKey &&
    !event.metaKey &&
    !isEditableTarget(event.target)
  );
}

function describePushToTalkFailure(error: unknown): string {
  const failure = classifyInvokeFailure(error);
  if (failure.kind === "tool-error") {
    return failure.toolError.message;
  }

  return failure.message;
}

async function beginPushToTalk(source: "keyboard" | "pointer") {
  if (!pushToTalkState.enabled || pushToTalkState.isHolding || pushToTalkState.isBusy) {
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

async function cancelPushToTalk() {
  activePushToTalkSource = null;

  if (!pushToTalkState.isHolding && !pushToTalkState.isListening) {
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
  } catch (error: unknown) {
    setPushToTalkState({
      isListening: false,
      isBusy: false,
      lastError: describePushToTalkFailure(error),
    });
  }
}

async function releasePushToTalk(source: "keyboard" | "pointer") {
  if (!pushToTalkState.isHolding || activePushToTalkSource !== source) {
    return;
  }

  activePushToTalkSource = null;
  setPushToTalkState({
    isHolding: false,
    isBusy: true,
    lastError: null,
  });

  try {
    const transcription = await transcribeCommand({
      requestId: createRequestId("push-to-talk-transcribe"),
      maxDurationMs: PUSH_TO_TALK_RELEASE_CAPTURE_MS,
      autoStop: true,
    });
    setPushToTalkState({
      enabled: transcription.listening_state.push_to_talk_enabled,
      isListening: transcription.listening_state.is_listening,
      isBusy: false,
      lastTranscript: transcription.transcript,
    });

    if (!transcription.transcript) {
      return;
    }

    const plannerOutput = await resolveCommand(
      createRequestId("push-to-talk-resolve"),
      transcription.transcript,
    );
    await runPlannerExecution(createRequestId("push-to-talk-execute"), plannerOutput, uiStore);
  } catch (error: unknown) {
    setPushToTalkState({
      isListening: false,
      isBusy: false,
      lastError: describePushToTalkFailure(error),
    });
  }
}

rerender();
uiStore.subscribe((uiState) => {
  currentExecutionUiState = uiState;
  rerender();
});

app.addEventListener("click", (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }

  const actionButton = target.closest<HTMLButtonElement>("[data-confirmation-action]");
  if (!actionButton) {
    return;
  }

  const confirmationState = uiStore.getState().confirmation;
  if (confirmationState.kind !== "awaiting-confirmation" || confirmationState.isSubmitting) {
    return;
  }

  const action = actionButton.dataset.confirmationAction;
  const confirmationId = actionButton.dataset.confirmationId;
  if (!confirmationId || confirmationId !== confirmationState.confirmationId) {
    return;
  }

  if (action !== "approve" && action !== "reject") {
    return;
  }

  const confirmed = action === "approve";
  uiStore.setConfirmationSubmitting(confirmationId, true);

  void resolveConfirmationResponse(
    {
      confirmationId,
      confirmed,
      timedOut: false,
    },
    uiStore,
  ).catch((error: unknown) => {
    uiStore.setConfirmationError(confirmationId, describeConfirmationSubmissionFailure(error));
    console.error("Failed to submit confirmation response.", error);
  });
});

app.addEventListener("pointerdown", (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) {
    return;
  }

  const button = target.closest<HTMLButtonElement>("[data-push-to-talk-button]");
  if (!button || button.disabled || event.button !== 0) {
    return;
  }

  event.preventDefault();
  void beginPushToTalk("pointer");
});

window.addEventListener("pointerup", () => {
  void releasePushToTalk("pointer");
});

window.addEventListener("pointercancel", () => {
  void cancelPushToTalk();
});

window.addEventListener("blur", () => {
  void cancelPushToTalk();
});

window.addEventListener("keydown", (event) => {
  if (!isPushToTalkKeyEvent(event)) {
    return;
  }

  event.preventDefault();
  void beginPushToTalk("keyboard");
});

window.addEventListener("keyup", (event) => {
  if (!isPushToTalkKeyEvent(event)) {
    return;
  }

  event.preventDefault();
  void releasePushToTalk("keyboard");
});

export { renderConfirmationPanel } from "./confirmation-panel";
