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
