import { useEffect, useRef, type KeyboardEvent } from "react";

import type { PrivacyConfirmationKind } from "./planner-privacy-operations.ts";
import {
  activatePrivacyConfirmationFocus,
  handlePrivacyConfirmationKeyboard,
} from "./planner-privacy-confirmation-interactions.ts";

function confirmationCopy(kind: PrivacyConfirmationKind): {
  title: string;
  description: string;
  confirmLabel: string;
} {
  switch (kind) {
    case "broad-network-mode":
      return {
        title: "Enable broad sanitized network planning?",
        description: "This permits sanitized context from any non-high-risk site to reach the configured non-loopback planner without a per-site prompt. Saved blocks and high-risk detection still override this mode.",
        confirmLabel: "Enable broad mode",
      };
    case "clear-persistent-allows":
      return {
        title: "Clear every saved allow rule?",
        description: "Saved site blocks remain. Sites that depended on a saved allow will require consent again or remain blocked by the selected network mode.",
        confirmLabel: "Clear saved allows",
      };
    case "clear-all-rules":
      return {
        title: "Clear every saved site privacy rule?",
        description: "This removes both allow and block rules. The global network mode will become the only persistent policy until new rules are created.",
        confirmLabel: "Clear all saved rules",
      };
  }
}

export function PrivacySettingsConfirmationDialog(props: {
  kind: PrivacyConfirmationKind;
  busy: boolean;
  onCancel: () => void;
  onConfirm: () => Promise<void>;
}) {
  const rootRef = useRef<HTMLDivElement>(null);
  const cancelRef = useRef<HTMLButtonElement>(null);
  const returnFocusRef = useRef<HTMLElement | null>(null);
  const copy = confirmationCopy(props.kind);

  useEffect(() => {
    returnFocusRef.current = document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null;
    const cancelControl = cancelRef.current;
    if (cancelControl === null) {
      return undefined;
    }
    return activatePrivacyConfirmationFocus(
      returnFocusRef.current,
      cancelControl,
    );
  }, [props.kind]);

  const handleKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const dialogRoot = rootRef.current;
    if (dialogRoot === null) {
      return;
    }
    const focusable = Array.from(
      dialogRoot.querySelectorAll<HTMLElement>(
        "button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex='-1'])",
      ),
    );
    handlePrivacyConfirmationKeyboard({
      event,
      busy: props.busy,
      activeElement: document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null,
      focusableElements: focusable,
      dialogRoot,
      cancel: props.onCancel,
    });
  };

  return (
    <div
      ref={rootRef}
      className="remote-privacy-settings-confirmation"
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="remote-privacy-settings-confirmation-title"
      aria-describedby="remote-privacy-settings-confirmation-description"
      aria-busy={props.busy}
      tabIndex={-1}
      onKeyDown={handleKeyDown}
      data-remote-planner-privacy-confirmation={props.kind}
    >
      <h4 id="remote-privacy-settings-confirmation-title">{copy.title}</h4>
      <p id="remote-privacy-settings-confirmation-description">{copy.description}</p>
      <div className="settings-button-row settings-button-row-wrap">
        <button
          ref={cancelRef}
          type="button"
          className="settings-control-button settings-control-button-secondary"
          disabled={props.busy || undefined}
          onClick={props.onCancel}
        >
          Cancel
        </button>
        <button
          type="button"
          className="settings-control-button settings-control-button-danger"
          disabled={props.busy || undefined}
          onClick={() => { void props.onConfirm(); }}
        >
          {props.busy ? "Applying privacy change…" : copy.confirmLabel}
        </button>
      </div>
    </div>
  );
}
