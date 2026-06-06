interface PreservedPanelControlState {
  elementId: string;
  value: string;
  selectionStart: number | null;
  selectionEnd: number | null;
  selectionDirection: "forward" | "backward" | "none" | null;
}

function captureActivePanelControl(root: HTMLDivElement): PreservedPanelControlState | null {
  const activeElement = document.activeElement;
  if (
    !activeElement
    || !root.contains(activeElement)
    || (
      !(activeElement instanceof HTMLInputElement)
      && !(activeElement instanceof HTMLTextAreaElement)
      && !(activeElement instanceof HTMLSelectElement)
    )
    || !activeElement.id
  ) {
    return null;
  }

  return {
    elementId: activeElement.id,
    value: activeElement.value,
    selectionStart:
      activeElement instanceof HTMLSelectElement ? null : activeElement.selectionStart,
    selectionEnd:
      activeElement instanceof HTMLSelectElement ? null : activeElement.selectionEnd,
    selectionDirection:
      activeElement instanceof HTMLSelectElement ? null : activeElement.selectionDirection,
  };
}

function restoreActivePanelControl(
  root: HTMLDivElement,
  controlState: PreservedPanelControlState | null,
) {
  if (!controlState) {
    return;
  }

  const nextElement = document.getElementById(controlState.elementId);
  if (
    !nextElement
    || !root.contains(nextElement)
    || (
      !(nextElement instanceof HTMLInputElement)
      && !(nextElement instanceof HTMLTextAreaElement)
      && !(nextElement instanceof HTMLSelectElement)
    )
  ) {
    return;
  }

  nextElement.focus({ preventScroll: true });
  if (
    nextElement instanceof HTMLInputElement
    || nextElement instanceof HTMLTextAreaElement
  ) {
    if (nextElement.value === controlState.value) {
      nextElement.setSelectionRange(
        controlState.selectionStart,
        controlState.selectionEnd,
        controlState.selectionDirection ?? undefined,
      );
    }
  }
}

export function preserveActivePanelControl(root: HTMLDivElement, renderPanel: () => void) {
  const controlState = captureActivePanelControl(root);
  renderPanel();
  restoreActivePanelControl(root, controlState);
}
