import { type ReactNode } from "react";

import type { AppAlertState } from "./panel-types.ts";
import { DISMISS_BUTTON_CLASS, SETTINGS_CARD_HEADING_CLASS } from "./settings-panels/shared-controls.tsx";

export interface AppAlertPanelHandlers {
  onDismiss?: () => void;
}

export function renderAppAlertPanelNode(
  state: AppAlertState,
  handlers?: AppAlertPanelHandlers,
): ReactNode {
  if (!state.message) {
    return null;
  }

  const title = state.kind === "error"
    ? "Action failed"
    : state.kind === "warning"
      ? "Warning"
      : "Notice";

  return (
    <section
      className="flex items-start justify-between gap-4 m-[0_0_18px] p-[16px_18px] rounded-[18px] bg-[var(--color-error-light)] border border-[rgba(139,52,42,0.24)] text-[var(--color-text-primary)]"
      data-app-alert-kind={state.kind}
      role={state.kind === "error" ? "alert" : "status"}
      aria-live={state.kind === "error" ? "assertive" : "polite"}
      aria-labelledby="app-alert-title"
    >
      <div className="flex flex-col gap-[6px] min-w-0">
        <p className="m-0 uppercase tracking-[0.08em] text-[0.76rem] font-bold text-[var(--color-error-primary)]">{state.kind}</p>
        <h2 id="app-alert-title" className={`m-0 ${SETTINGS_CARD_HEADING_CLASS}`}>{title}</h2>
        <p className="m-0 [overflow-wrap:anywhere] leading-[1.5]">{state.message}</p>
      </div>
      <button
        type="button"
        className={`${DISMISS_BUTTON_CLASS} flex-none`}
        data-app-alert-dismiss="true"
        onClick={handlers?.onDismiss}
      >
        Dismiss
      </button>
    </section>
  );
}
