import { type ReactNode } from "react";

import type { AppAlertState } from "./panel-types.ts";

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
      className={`app-alert app-alert-${state.kind}`}
      role={state.kind === "error" ? "alert" : "status"}
      aria-live={state.kind === "error" ? "assertive" : "polite"}
      aria-labelledby="app-alert-title"
    >
      <div className="app-alert-copy">
        <p className="app-alert-eyebrow">{state.kind}</p>
        <h2 id="app-alert-title">{title}</h2>
        <p className="app-alert-message">{state.message}</p>
      </div>
      <button
        type="button"
        className="panel-error-dismiss app-alert-dismiss"
        data-app-alert-dismiss="true"
        onClick={handlers?.onDismiss}
      >
        Dismiss
      </button>
    </section>
  );
}
