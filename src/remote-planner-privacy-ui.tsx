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
import {
  REMOTE_CONSENT_ACTION_BUTTON_CLASS,
  REMOTE_CONSENT_ACTIONS_CLASS,
  REMOTE_CONSENT_DD_CLASS,
  REMOTE_CONSENT_DESTINATION_COUNTS_CLASS,
  REMOTE_CONSENT_DESTINATION_COUNTS_ROW_CLASS,
  REMOTE_CONSENT_DIALOG_CLASS,
  REMOTE_CONSENT_DISCLOSURES_CLASS,
  REMOTE_CONSENT_DT_CLASS,
  REMOTE_CONSENT_ERROR_CLASS,
  REMOTE_CONSENT_ERROR_PARAGRAPH_CLASS,
  REMOTE_CONSENT_LOCAL_CANCEL_BUTTON_CLASS,
  REMOTE_PRIVACY_DETAIL_CLASS,
  REMOTE_PRIVACY_ERROR_CLASS,
  REMOTE_PRIVACY_EYEBROW_CLASS,
  REMOTE_PRIVACY_GUIDANCE_CLASS,
  REMOTE_PRIVACY_HEADING_CLASS,
  REMOTE_PRIVACY_SETTINGS_BUTTON_CLASS,
  REMOTE_PRIVACY_STATUS_CLASS,
  REMOTE_PRIVACY_STATUS_COPY_CLASS,
  REMOTE_PRIVACY_SUBHEADING_CLASS,
  REMOTE_PRIVACY_WARNING_CLASS,
} from "./settings-panels/shared-controls.tsx";

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
  narration_text: "Page narration text sent to the remote speech provider",
  microphone_audio: "Captured microphone audio sent to the remote transcription provider",
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
      className={REMOTE_PRIVACY_STATUS_CLASS}
      aria-labelledby="remote-privacy-status-title"
      data-remote-privacy-status="true"
    >
      <div className={REMOTE_PRIVACY_STATUS_COPY_CLASS}>
        <p className={REMOTE_PRIVACY_EYEBROW_CLASS}>Planner privacy</p>
        <h2 id="remote-privacy-status-title" className={REMOTE_PRIVACY_HEADING_CLASS}>
          {status
            ? DECISION_LABELS[status.effective_decision]
            : "Checking planner privacy status"}
        </h2>
        {status?.current_page_origin ? (
          <p className={REMOTE_PRIVACY_DETAIL_CLASS}>
            Site: <code>{status.current_page_origin}</code>
          </p>
        ) : null}
        {status?.endpoint_display ? (
          <p className={REMOTE_PRIVACY_DETAIL_CLASS}>
            Planner: <code>{status.endpoint_display}</code>
          </p>
        ) : null}
        {status?.effective_decision === "high_risk_blocked" ? (
          <p className={REMOTE_PRIVACY_GUIDANCE_CLASS}>
            Network planning cannot be enabled for this page. Use a local planner or continue with direct commands.
          </p>
        ) : null}
        {status?.stale_allow_rule_count ? (
          <p className={REMOTE_PRIVACY_WARNING_CLASS} role="status">
            {status.stale_allow_rule_count} saved allow rule{status.stale_allow_rule_count === 1 ? " is" : "s are"} inactive because the destination or privacy policy changed.
          </p>
        ) : null}
        {state.refreshBusy ? (
          <p className={REMOTE_PRIVACY_DETAIL_CLASS} role="status" aria-live="polite">
            Refreshing authoritative privacy status…
          </p>
        ) : null}
        {state.refreshError ? (
          <p className={REMOTE_PRIVACY_ERROR_CLASS} role="alert">
            {state.refreshError}
          </p>
        ) : null}
      </div>
      {handlers?.onOpenSettings ? (
        <button
          type="button"
          className={REMOTE_PRIVACY_SETTINGS_BUTTON_CLASS}
          onClick={handlers.onOpenSettings}
        >
          Privacy settings
        </button>
      ) : null}
    </section>
  );
}

type RemoteConsentKind = "planner" | "narration" | "microphone";

function remoteConsentKind(
  challenge: Extract<RemoteDataConsentUiState, { kind: "awaiting-remote-data-consent" }>["challenge"],
): RemoteConsentKind {
  if (challenge.disclosure_classes.includes("microphone_audio")) {
    return "microphone";
  }
  if (challenge.disclosure_classes.includes("narration_text")) {
    return "narration";
  }
  return "planner";
}

function disclosureCountSummary(
  consentState: Extract<RemoteDataConsentUiState, { kind: "awaiting-remote-data-consent" }>,
): ReactNode {
  const counts = consentState.challenge.disclosure_counts;
  const kind = remoteConsentKind(consentState.challenge);
  if (kind === "narration") {
    return (
      <dl className={REMOTE_CONSENT_DESTINATION_COUNTS_CLASS} data-remote-consent-counts="true">
        <div className={REMOTE_CONSENT_DESTINATION_COUNTS_ROW_CLASS}><dt className={REMOTE_CONSENT_DT_CLASS}>Narration text size</dt><dd className={REMOTE_CONSENT_DD_CLASS}>{counts.narration_text_bytes} bytes</dd></div>
      </dl>
    );
  }
  if (kind === "microphone") {
    return (
      <dl className={REMOTE_CONSENT_DESTINATION_COUNTS_CLASS} data-remote-consent-counts="true">
        <div className={REMOTE_CONSENT_DESTINATION_COUNTS_ROW_CLASS}><dt className={REMOTE_CONSENT_DT_CLASS}>Requested microphone capture</dt><dd className={REMOTE_CONSENT_DD_CLASS}>{counts.microphone_audio_duration_ms} ms</dd></div>
      </dl>
    );
  }
  return (
    <dl className={REMOTE_CONSENT_DESTINATION_COUNTS_CLASS} data-remote-consent-counts="true">
      <div className={REMOTE_CONSENT_DESTINATION_COUNTS_ROW_CLASS}><dt className={REMOTE_CONSENT_DT_CLASS}>Selected text regions</dt><dd className={REMOTE_CONSENT_DD_CLASS}>{counts.selected_region_count}</dd></div>
      <div className={REMOTE_CONSENT_DESTINATION_COUNTS_ROW_CLASS}><dt className={REMOTE_CONSENT_DT_CLASS}>Selected elements</dt><dd className={REMOTE_CONSENT_DD_CLASS}>{counts.selected_element_count}</dd></div>
      <div className={REMOTE_CONSENT_DESTINATION_COUNTS_ROW_CLASS}><dt className={REMOTE_CONSENT_DT_CLASS}>OCR-derived regions</dt><dd className={REMOTE_CONSENT_DD_CLASS}>{counts.ocr_derived_region_count}</dd></div>
      <div className={REMOTE_CONSENT_DESTINATION_COUNTS_ROW_CLASS}><dt className={REMOTE_CONSENT_DT_CLASS}>Tool summaries</dt><dd className={REMOTE_CONSENT_DD_CLASS}>{counts.tool_history_count}</dd></div>
      <div className={REMOTE_CONSENT_DESTINATION_COUNTS_ROW_CLASS}><dt className={REMOTE_CONSENT_DT_CLASS}>Skill summaries</dt><dd className={REMOTE_CONSENT_DD_CLASS}>{counts.skill_summary_count}</dd></div>
      <div className={REMOTE_CONSENT_DESTINATION_COUNTS_ROW_CLASS}><dt className={REMOTE_CONSENT_DT_CLASS}>Sanitized request size</dt><dd className={REMOTE_CONSENT_DD_CLASS}>{counts.sanitized_serialized_bytes} bytes</dd></div>
    </dl>
  );
}

function consentDialogCopy(kind: RemoteConsentKind) {
  switch (kind) {
    case "narration":
      return {
        title: "Send this page narration text to the remote speech provider?",
        description: "Blind Browser paused before network access. No narration text has been sent yet.",
        destinationLabel: "Speech provider",
        warning: "Remote narration sends the page text to the configured speech provider as-is so it can be spoken. This permission is separate from planner permission and does not approve clicks, typing, submissions, downloads, credentials, or other actions.",
        onceLabel: "Allow narration text for this request only",
        sessionLabel: "Allow narration text for this site and speech provider for this application session",
        persistentLabel: "Always allow narration text for this site and exact speech destination",
        blockLabel: "Keep narration text local for this site",
      };
    case "microphone":
      return {
        title: "Send microphone audio to the remote transcription provider?",
        description: "Blind Browser paused before remote transcription. Captured audio from the paused attempt is not retained or sent; after allowing access, repeat the voice input.",
        destinationLabel: "Transcription provider",
        warning: "Remote voice input sends captured microphone audio to the configured transcription provider. This permission is separate from planner and narration permission. Pre-consent audio is discarded, so authorization applies to a new voice capture.",
        onceLabel: "Allow one new microphone upload with the same request settings",
        sessionLabel: "Allow microphone audio for this site and transcription provider for this application session",
        persistentLabel: "Always allow microphone audio for this site and exact transcription destination",
        blockLabel: "Keep microphone audio local for this site",
      };
    case "planner":
      return {
        title: "Send sanitized information to the network planner?",
        description: "Blind Browser paused before network access. No planner request has been sent yet.",
        destinationLabel: "Planner",
        warning: "The request is locally selected and sanitized, but it may still contain page or user information. This permission does not approve clicks, typing, submissions, downloads, credentials, or other actions.",
        onceLabel: "Allow sanitized data for this request only",
        sessionLabel: "Allow sanitized data for this site and planner for this application session",
        persistentLabel: "Always allow sanitized data for this site and exact planner destination",
        blockLabel: "Keep this site local for every network planner",
      };
  }
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
  const consentKind = remoteConsentKind(challenge);
  const dialogCopy = consentDialogCopy(consentKind);

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
      className={REMOTE_CONSENT_DIALOG_CLASS}
      role="dialog"
      aria-modal="true"
      aria-labelledby="remote-consent-title"
      aria-describedby="remote-consent-description remote-consent-warning"
      aria-busy={isSubmitting}
      data-remote-consent-dialog="true"
      tabIndex={-1}
      onKeyDown={handleKeyDown}
    >
      <p className={REMOTE_PRIVACY_EYEBROW_CLASS}>Permission required</p>
      <h2 id="remote-consent-title" className={REMOTE_PRIVACY_HEADING_CLASS}>{dialogCopy.title}</h2>
      <p id="remote-consent-description">{dialogCopy.description}</p>

      <dl className={REMOTE_CONSENT_DESTINATION_COUNTS_CLASS}>
        <div className={REMOTE_CONSENT_DESTINATION_COUNTS_ROW_CLASS}><dt className={REMOTE_CONSENT_DT_CLASS}>Site</dt><dd className={REMOTE_CONSENT_DD_CLASS}><code>{challenge.page_origin}</code></dd></div>
        <div className={REMOTE_CONSENT_DESTINATION_COUNTS_ROW_CLASS}><dt className={REMOTE_CONSENT_DT_CLASS}>{dialogCopy.destinationLabel}</dt><dd className={REMOTE_CONSENT_DD_CLASS}><code>{challenge.endpoint_display}</code></dd></div>
        <div className={REMOTE_CONSENT_DESTINATION_COUNTS_ROW_CLASS}><dt className={REMOTE_CONSENT_DT_CLASS}>Profile</dt><dd className={REMOTE_CONSENT_DD_CLASS}>{challenge.profile_name}</dd></div>
        <div className={REMOTE_CONSENT_DESTINATION_COUNTS_ROW_CLASS}><dt className={REMOTE_CONSENT_DT_CLASS}>Model</dt><dd className={REMOTE_CONSENT_DD_CLASS}>{challenge.model_label}</dd></div>
      </dl>

      <h3 className={REMOTE_PRIVACY_SUBHEADING_CLASS}>Information categories</h3>
      <ul className={REMOTE_CONSENT_DISCLOSURES_CLASS}>
        {challenge.disclosure_classes.map((disclosureClass) => (
          <li key={disclosureClass}>{DISCLOSURE_LABELS[disclosureClass]}</li>
        ))}
      </ul>
      {disclosureCountSummary(props.consentState)}

      <p id="remote-consent-warning" className={REMOTE_PRIVACY_WARNING_CLASS}>
        {dialogCopy.warning}
      </p>
      <p className={REMOTE_PRIVACY_DETAIL_CLASS}>
        This request expires at {expiry.dateTime
          ? <time dateTime={expiry.dateTime}>{expiry.label}</time>
          : <span>{expiry.label}</span>}.
      </p>

      {submissionError ? (
        <div className={REMOTE_CONSENT_ERROR_CLASS} data-remote-consent-error="true" role="alert">
          <strong>{submissionError.title}</strong>
          <p className={REMOTE_CONSENT_ERROR_PARAGRAPH_CLASS}>{submissionError.message}</p>
          <p className={REMOTE_CONSENT_ERROR_PARAGRAPH_CLASS}>{submissionError.guidance}</p>
        </div>
      ) : null}

      <div className={REMOTE_CONSENT_ACTIONS_CLASS} aria-label="Remote data choices">
        {challenge.allow_once ? (
          <button
            type="button"
            className={REMOTE_CONSENT_ACTION_BUTTON_CLASS}
            data-remote-consent-decision="allow_once"
            disabled={isSubmitting || undefined}
            aria-label={dialogCopy.onceLabel}
            onClick={() => { submitDecision("allow_once"); }}
          >
            Allow this request
          </button>
        ) : null}
        {challenge.allow_session ? (
          <button
            type="button"
            className={REMOTE_CONSENT_ACTION_BUTTON_CLASS}
            data-remote-consent-decision="allow_session"
            disabled={isSubmitting || undefined}
            aria-label={dialogCopy.sessionLabel}
            onClick={() => { submitDecision("allow_session"); }}
          >
            Allow for this session
          </button>
        ) : null}
        {challenge.allow_persistent ? (
          <button
            type="button"
            className={REMOTE_CONSENT_ACTION_BUTTON_CLASS}
            data-remote-consent-decision="allow_persistent"
            disabled={isSubmitting || undefined}
            aria-label={dialogCopy.persistentLabel}
            onClick={() => { submitDecision("allow_persistent"); }}
          >
            Always allow for this site
          </button>
        ) : null}
        {challenge.block_persistent ? (
          <button
            type="button"
            className={`${REMOTE_CONSENT_ACTION_BUTTON_CLASS} ${REMOTE_CONSENT_LOCAL_CANCEL_BUTTON_CLASS}`}
            data-remote-consent-decision="block_persistent"
            disabled={isSubmitting || undefined}
            aria-label={dialogCopy.blockLabel}
            onClick={() => { submitDecision("block_persistent"); }}
          >
            Keep this site local
          </button>
        ) : null}
        <button
          ref={cancelRef}
          type="button"
          className={`${REMOTE_CONSENT_ACTION_BUTTON_CLASS} ${REMOTE_CONSENT_LOCAL_CANCEL_BUTTON_CLASS}`}
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
