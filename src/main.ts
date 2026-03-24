import "./styles.css";

import { renderConfirmationPanel } from "./confirmation-panel";
import {
  createExecutionUiStore,
  describeConfirmationSubmissionFailure,
  resolveConfirmationResponse,
  type ExecutionUiState,
} from "./planner-orchestration";

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
  submitConfirmationResponse as invokeSubmitConfirmationResponse,
} from "./tauri-api";

const app = document.querySelector<HTMLDivElement>("#app");
const uiStore = createExecutionUiStore();

if (!app) {
  throw new Error("App root element was not found.");
}

const renderApp = (uiState: ExecutionUiState) => {
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

      ${renderConfirmationPanel(uiState.confirmation)}
    </main>
  `;
};

renderApp(uiStore.getState());
uiStore.subscribe(renderApp);

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

export { renderConfirmationPanel } from "./confirmation-panel";
