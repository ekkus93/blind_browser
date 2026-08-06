export interface PrivacyConfirmationFocusableTarget {
  isConnected: boolean;
  focus: () => void;
}

export interface PrivacyConfirmationKeyboardEvent {
  key: string;
  shiftKey: boolean;
  preventDefault: () => void;
}

export function activatePrivacyConfirmationFocus(
  invokingControl: PrivacyConfirmationFocusableTarget | null,
  cancelControl: PrivacyConfirmationFocusableTarget,
): () => void {
  cancelControl.focus();
  return () => {
    if (invokingControl?.isConnected) {
      invokingControl.focus();
    }
  };
}

export function handlePrivacyConfirmationKeyboard(context: {
  event: PrivacyConfirmationKeyboardEvent;
  busy: boolean;
  activeElement: PrivacyConfirmationFocusableTarget | null;
  focusableElements: readonly PrivacyConfirmationFocusableTarget[];
  dialogRoot: PrivacyConfirmationFocusableTarget;
  cancel: () => void;
}): void {
  const {
    event,
    busy,
    activeElement,
    focusableElements,
    dialogRoot,
    cancel,
  } = context;

  if (event.key === "Escape" && !busy) {
    event.preventDefault();
    cancel();
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
