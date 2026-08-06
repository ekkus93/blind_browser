import type { RemotePlannerConsentDecision } from "./tauri-api.ts";

export interface ConsentDialogFocusableTarget {
  focus: () => void;
  isConnected?: boolean;
}

export interface ConsentDialogKeyboardEvent {
  key: string;
  shiftKey: boolean;
  preventDefault: () => void;
}

export interface ConsentDialogSubmissionGate {
  started: boolean;
}

export interface ConsentDialogSubmissionContext {
  gate: ConsentDialogSubmissionGate;
  isSubmitting: boolean;
  decision: RemotePlannerConsentDecision;
  challengeId: string;
  submitDecision?: (
    decision: RemotePlannerConsentDecision,
    challengeId: string,
  ) => void;
}

export function synchronizeConsentDialogSubmissionGate(
  gate: ConsentDialogSubmissionGate,
  isSubmitting: boolean,
): void {
  gate.started = isSubmitting;
}

export function submitConsentDialogDecision(
  context: ConsentDialogSubmissionContext,
): boolean {
  const { gate, isSubmitting, decision, challengeId, submitDecision } = context;
  if (isSubmitting || gate.started || !submitDecision) {
    return false;
  }
  gate.started = true;
  submitDecision(decision, challengeId);
  return true;
}

export interface ConsentDialogKeyboardContext {
  event: ConsentDialogKeyboardEvent;
  activeElement: ConsentDialogFocusableTarget | null;
  focusableElements: readonly ConsentDialogFocusableTarget[];
  dialogRoot: ConsentDialogFocusableTarget;
  submitDecision: (decision: RemotePlannerConsentDecision) => void;
}

export function activateConsentDialogFocus(
  invokingElement: ConsentDialogFocusableTarget | null,
  cancelButton: ConsentDialogFocusableTarget | null,
): () => void {
  cancelButton?.focus();
  return () => {
    if (invokingElement?.isConnected !== false) {
      invokingElement?.focus();
    }
  };
}

export function handleConsentDialogKeyboard(
  context: ConsentDialogKeyboardContext,
): void {
  const {
    event,
    activeElement,
    focusableElements,
    dialogRoot,
    submitDecision,
  } = context;

  if (event.key === "Escape") {
    event.preventDefault();
    submitDecision("deny");
    return;
  }

  if (event.key !== "Tab") {
    return;
  }

  if (focusableElements.length === 0) {
    event.preventDefault();
    dialogRoot.focus();
    return;
  }

  const first = focusableElements[0];
  const last = focusableElements[focusableElements.length - 1];
  if (event.shiftKey && activeElement === first) {
    event.preventDefault();
    last.focus();
  } else if (!event.shiftKey && activeElement === last) {
    event.preventDefault();
    first.focus();
  }
}
