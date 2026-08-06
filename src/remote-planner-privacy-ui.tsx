import {
  Fragment,
  useEffect,
  useRef,
  type KeyboardEvent,
  type ReactNode,
} from "react";

import {
  activateConsentDialogFocus,
  handleConsentDialogKeyboard,
  submitConsentDialogDecision,
  synchronizeConsentDialogSubmissionGate,
} from "./remote-planner-consent-dialog-interactions.ts";

import type {
  RemoteDataConsentUiState,
} from "./planner-orchestration.ts";
import type {
  RemotePlannerConsentDecision,
  RemotePlannerConsentDisclosureClass,
} from "./tauri-api.ts";
import type { RemotePlannerPrivacyState } from "./remote-planner-privacy-state.ts";

const DECISION_LABELS = {
  loopback_local: "On-device planner",
  local_only: "Local-only mode",
  high_risk_blocked: "High-risk page: network planner blocked",
  origin_blocked: "This site stays local",
  allowed_global: "Remote data allowed by the global setting",
  allowed_persistent: "Remote data always allowed for this site and destination",
  allowed_session: "Remote data allowed for this session",
  consent_required: "Remote data: ask for this site",
  origin_unavailable: "Current page cannot use a network planner",
  planner_unavailable: "Remote planner unavailable",
} as const;

const DISCLOSURE_LABELS: Record<RemotePlannerConsentDisclosureClass, string> = {
  user_transcript: "Your command transcript",
  page_origin: "The current site origin and sanitized URL information",
  selected_page_regions: "Locally selected page text regions",
  selected_element_metadata: "Locally selected element labels and safe attributes",
  ocr_derived_regions: "Locally selected OCR-derived regions",
  tool_observation_summaries: "Recent tool-result summaries",
  skill_summaries: "Relevant skill summaries",
  trusted_runtime_contracts: "Trusted runtime safety and tool contracts",
};

export interface RemotePlannerPrivacyWorkspaceHandlers {
  onOpenSettings?: () => void;
  onConsentDecision?: (
    decision: RemotePlannerConsentDecision,
    challengeId: string,
  ) => void;
}

function renderPrivacyStatus(
  state: RemotePlannerPrivacyState,
  handlers?: RemotePlannerPrivacyWorkspaceHandlers,
): ReactNode {
  if (!state.isLoaded && !state.refreshBusy && state.refreshError === null) {
    return null;
  }

  const status = state.status;
  return (
    <section
      className="remote-privacy-status"
      aria-labelledby="remote-privacy-status-title"
      data-remote-privacy-status="true"
    >
      <div className="remote-privacy-status-copy">
        <p className="remote-privacy-eyebrow">Planner privacy</p>
        <h2 id="remote-privacy-status-title">
          {status
            ? DECISION_LABELS[status.effective_decision]
            : "Checking planner privacy status"}
        </h2>
        {status?.current_page_origin ? (
          <p className="remote-privacy-detail">
            Site: <code>{status.current_page_origin}</code>
          </p>
        ) : null}
        {status?.endpoint_display ? (
          <p className="remote-privacy-detail">
            Planner: <code>{status.endpoint_display}</code>
          </p>
        ) : null}
        {status?.effective_decision === "high_risk_blocked" ? (
          <p className="remote-privacy-guidance">
            Network planning cannot be enabled for this page. Use a local planner or continue with direct commands.
          </p>
        ) : null}
        {status?.stale_allow_rule_count ? (
          <p className="remote-privacy-warning" role="status">
            {status.stale_allow_rule_count} saved allow rule{status.stale_allow_rule_count === 1 ? " is" : "s are"} inactive because the destination or privacy policy changed.
          </p>
        ) : null}
        {state.refreshBusy ? (
          <p className="remote-privacy-detail" role="status" aria-live="polite">
            Refreshing authoritative privacy status…
          </p>
        ) : null}
        {state.refreshError ? (
          <p className="remote-privacy-error" role="alert">
            {state.refreshError}
          </p>
        ) : null}
      </div>
      {handlers?.onOpenSettings ? (
        <button
          type="button"
          className="remote-privacy-settings-button"
          onClick={handlers.onOpenSettings}
        >
          Privacy settings
        </button>
      ) : null}
    </section>
  );
}

function disclosureCountSummary(
  consentState: Extract<RemoteDataConsentUiState, { kind: "awaiting-remote-data-consent" }>,
): ReactNode {
  const counts = consentState.challenge.disclosure_counts;
  return (
    <dl className="remote-consent-counts" data-remote-consent-counts="true">
      <div><dt>Selected text regions</dt><dd>{counts.selected_region_count}</dd></div>
      <div><dt>Selected elements</dt><dd>{counts.selected_element_count}</dd></div>
      <div><dt>OCR-derived regions</dt><dd>{counts.ocr_derived_region_count}</dd></div>
      <div><dt>Tool summaries</dt><dd>{counts.tool_history_count}</dd></div>
      <div><dt>Skill summaries</dt><dd>{counts.skill_summary_count}</dd></div>
      <div><dt>Sanitized request size</dt><dd>{counts.sanitized_serialized_bytes} bytes</dd></div>
    </dl>
  );
}

function safeExpiryDisplay(expiresAtMs: number): {
  dateTime?: string;
  label: string;
} {
  const date = new Date(expiresAtMs);
  if (!Number.isFinite(date.getTime())) {
    return { label: "an invalid time; cancel this request and try again" };
  }
  const iso = date.toISOString();
  return { dateTime: iso, label: iso };
}

function RemotePlannerConsentDialog(props: {
  consentState: Extract<RemoteDataConsentUiState, { kind: "awaiting-remote-data-consent" }>;
  onDecision?: RemotePlannerPrivacyWorkspaceHandlers["onConsentDecision"];
}) {
  const rootRef = useRef<HTMLElement>(null);
  const cancelRef = useRef<HTMLButtonElement>(null);
  const returnFocusRef = useRef<HTMLElement | null>(null);
  const submissionGateRef = useRef({ started: false });
  const { challenge, isSubmitting, submissionError } = props.consentState;
  const expiry = safeExpiryDisplay(challenge.expires_at_ms);

  useEffect(() => {
    returnFocusRef.current = document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null;
    return activateConsentDialogFocus(
      returnFocusRef.current,
      cancelRef.current,
    );
  }, [challenge.challenge_id]);

  useEffect(() => {
    synchronizeConsentDialogSubmissionGate(
      submissionGateRef.current,
      isSubmitting,
    );
  }, [challenge.challenge_id, isSubmitting]);

  const submitDecision = (decision: RemotePlannerConsentDecision) => {
    submitConsentDialogDecision({
      gate: submissionGateRef.current,
      isSubmitting,
      decision,
      challengeId: challenge.challenge_id,
      submitDecision: props.onDecision,
    });
  };

  const handleKeyDown = (event: KeyboardEvent<HTMLElement>) => {
    const root = rootRef.current;
    if (!root) {
      return;
    }
    handleConsentDialogKeyboard({
      event,
      activeElement: document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null,
      focusableElements: Array.from(
        root.querySelectorAll<HTMLElement>(
          "button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex='-1'])",
        ),
      ),
      dialogRoot: root,
      submitDecision,
    });
  };

  return (
    <section
      ref={rootRef}
      className="remote-consent-dialog"
      role="dialog"
      aria-modal="true"
      aria-labelledby="remote-consent-title"
      aria-describedby="remote-consent-description remote-consent-warning"
      aria-busy={isSubmitting}
      data-remote-consent-dialog="true"
      tabIndex={-1}
      onKeyDown={handleKeyDown}
    >
      <p className="remote-privacy-eyebrow">Permission required</p>
      <h2 id="remote-consent-title">Send sanitized information to the network planner?</h2>
      <p id="remote-consent-description">
        Blind Browser paused before network access. No planner request has been sent yet.
      </p>

      <dl className="remote-consent-destination">
        <div><dt>Site</dt><dd><code>{challenge.page_origin}</code></dd></div>
        <div><dt>Planner</dt><dd><code>{challenge.endpoint_display}</code></dd></div>
        <div><dt>Profile</dt><dd>{challenge.profile_name}</dd></div>
        <div><dt>Model</dt><dd>{challenge.model_label}</dd></div>
      </dl>

      <h3>Information categories</h3>
      <ul className="remote-consent-disclosures">
        {challenge.disclosure_classes.map((disclosureClass) => (
          <li key={disclosureClass}>{DISCLOSURE_LABELS[disclosureClass]}</li>
        ))}
      </ul>
      {disclosureCountSummary(props.consentState)}

      <p id="remote-consent-warning" className="remote-privacy-warning">
        The request is locally selected and sanitized, but it may still contain page or user information. This permission does not approve clicks, typing, submissions, downloads, credentials, or other actions.
      </p>
      <p className="remote-privacy-detail">
        This request expires at {expiry.dateTime
          ? <time dateTime={expiry.dateTime}>{expiry.label}</time>
          : <span>{expiry.label}</span>}.
      </p>

      {submissionError ? (
        <div className="remote-consent-error" role="alert">
          <strong>{submissionError.title}</strong>
          <p>{submissionError.message}</p>
          <p>{submissionError.guidance}</p>
        </div>
      ) : null}

      <div className="remote-consent-actions" aria-label="Remote data choices">
        {challenge.allow_once ? (
          <button
            type="button"
            data-remote-consent-decision="allow_once"
            disabled={isSubmitting || undefined}
            aria-label="Allow sanitized data for this request only"
            onClick={() => { submitDecision("allow_once"); }}
          >
            Allow this request
          </button>
        ) : null}
        {challenge.allow_session ? (
          <button
            type="button"
            data-remote-consent-decision="allow_session"
            disabled={isSubmitting || undefined}
            aria-label="Allow sanitized data for this site and planner for this application session"
            onClick={() => { submitDecision("allow_session"); }}
          >
            Allow for this session
          </button>
        ) : null}
        {challenge.allow_persistent ? (
          <button
            type="button"
            data-remote-consent-decision="allow_persistent"
            disabled={isSubmitting || undefined}
            aria-label="Always allow sanitized data for this site and exact planner destination"
            onClick={() => { submitDecision("allow_persistent"); }}
          >
            Always allow for this site
          </button>
        ) : null}
        {challenge.block_persistent ? (
          <button
            type="button"
            className="remote-consent-local-button"
            data-remote-consent-decision="block_persistent"
            disabled={isSubmitting || undefined}
            aria-label="Keep this site local for every network planner"
            onClick={() => { submitDecision("block_persistent"); }}
          >
            Keep this site local
          </button>
        ) : null}
        <button
          ref={cancelRef}
          type="button"
          className="remote-consent-cancel-button"
          data-remote-consent-decision="deny"
          disabled={isSubmitting || undefined}
          aria-label="Cancel and do not send data"
          onClick={() => { submitDecision("deny"); }}
        >
          {isSubmitting ? "Processing privacy choice…" : "Cancel"}
        </button>
      </div>
    </section>
  );
}

export function renderRemotePlannerPrivacyWorkspaceNode(
  privacyState: RemotePlannerPrivacyState,
  consentState: RemoteDataConsentUiState,
  handlers?: RemotePlannerPrivacyWorkspaceHandlers,
): ReactNode {
  return (
    <Fragment>
      {renderPrivacyStatus(privacyState, handlers)}
      {consentState.kind === "awaiting-remote-data-consent" ? (
        <RemotePlannerConsentDialog
          consentState={consentState}
          onDecision={handlers?.onConsentDecision}
        />
      ) : null}
    </Fragment>
  );
}
