import type { ReactNode } from "react";
import type { ConfirmationUiState } from "./planner-orchestration";
import {
  BTN_SPINNER_CLASS,
  CONTROL_LABEL,
  SETTINGS_API_KEY_BUTTON_ROW_CLASS,
  SETTINGS_API_KEY_INLINE_ACTIONS_CLASS,
  SETTINGS_API_KEY_INPUT_CLASS,
  SETTINGS_API_KEY_TEST_STATUS_CLASS,
  SETTINGS_API_KEY_TEST_STATUS_LABEL_CLASS,
  SETTINGS_API_KEY_TEST_STATUS_MESSAGE_CLASS,
  SETTINGS_CONTROL_BUTTON_CLASS,
  SETTINGS_CONTROL_BUTTON_SECONDARY_CLASS,
  SETTINGS_PANEL_DESCRIPTION_CLASS,
  SETTINGS_SECRET_ENTRY_CARD_CLASS,
} from "./settings-panels/shared-controls.tsx";

export const OPENAI_API_KEYS_URL = "https://platform.openai.com/account/api-keys";

export interface SecretEntryCardHandlers {
  onInput?: (value: string) => void;
  onSave?: () => void;
  onTest?: () => void;
  onOpenExternalLink?: (url: string) => void;
}

export function renderTtsModelOptionLabel(profileName: string, modelLabel: string): string {
  return `${modelLabel} (${profileName})`;
}

export function renderTtsVoiceOptionLabel(displayLabel: string, voiceName: string): string {
  return displayLabel === voiceName ? displayLabel : `${displayLabel} (${voiceName})`;
}

export function renderProviderModeLabel(mode: "Local" | "Remote"): string {
  return mode === "Local" ? "Local provider" : "Remote provider";
}

export function renderFailoverAvailabilityLabel(available: boolean): string {
  return available ? "Available" : "Unavailable";
}

export function renderConfirmationThresholdValue(confidenceThreshold: number): string {
  return `${Math.round(confidenceThreshold * 100)}%`;
}

export function renderOcrThresholdValue(value: number): string {
  return `${value}`;
}

export function renderModelAvailabilityLabel(available: boolean): string {
  return available ? "Downloaded" : "Missing";
}

export function renderTextWithKnownLinkNodes(value: string, onOpenExternalLink?: (url: string) => void): ReactNode[] {
  const segments = value.split(OPENAI_API_KEYS_URL);
  const children: ReactNode[] = [];

  segments.forEach((segment, index) => {
    if (segment.length > 0) {
      children.push(segment);
    }

    if (index < segments.length - 1) {
      children.push(
        <a
          href={OPENAI_API_KEYS_URL}
          target="_blank"
          rel="noreferrer"
          data-external-link-url={OPENAI_API_KEYS_URL}
          onClick={onOpenExternalLink
            ? (event: { preventDefault: () => void }) => {
              event.preventDefault();
              onOpenExternalLink(OPENAI_API_KEYS_URL);
            }
            : undefined}
          key={`known-link-${index}`}
        >
          {OPENAI_API_KEYS_URL}
        </a>,
      );
    }
  });

  return children;
}

export function renderSecretEntryCard(
  kind: "planner" | "tts" | "asr",
  profileName: string | null,
  apiKeyDraft: string,
  apiKeyMaskedValue: string | null,
  isSavingApiKey: boolean,
  isTestingApiKey: boolean,
  hasApiKeyReference: boolean,
  apiKeyTestMessage: string | null,
  handlers: SecretEntryCardHandlers = {},
): ReactNode {
  const controlsDisabled = isSavingApiKey || isTestingApiKey;
  const saveDisabled =
    isSavingApiKey || isTestingApiKey || profileName === null || apiKeyDraft.trim().length === 0;
  const testDisabled =
    isSavingApiKey
    || isTestingApiKey
    || profileName === null
    || (apiKeyDraft.trim().length === 0 && !hasApiKeyReference);
  const displayedApiKeyValue = apiKeyDraft.length > 0 ? apiKeyDraft : (apiKeyMaskedValue ?? "");
  const inputType = apiKeyDraft.length > 0 ? "password" : (apiKeyMaskedValue ? "text" : "password");

  return (
    <div className={SETTINGS_SECRET_ENTRY_CARD_CLASS}>
      <span className={CONTROL_LABEL}>API key</span>
      <div className={SETTINGS_API_KEY_INLINE_ACTIONS_CLASS}>
        <input
          id={`settings-remote-${kind}-api-key-input`}
          className={SETTINGS_API_KEY_INPUT_CLASS}
          data-remote-api-key-input={kind}
          type={inputType}
          value={displayedApiKeyValue}
          placeholder="Enter a replacement API key"
          autoComplete="off"
          spellCheck={false}
          data-masked-api-key-display={apiKeyDraft.length === 0 && apiKeyMaskedValue ? apiKeyMaskedValue : undefined}
          disabled={controlsDisabled || undefined}
          aria-disabled={controlsDisabled ? "true" : undefined}
          onChange={handlers.onInput
            ? (event: { currentTarget: { value: string } }) => {
              handlers.onInput?.(event.currentTarget.value);
            }
            : undefined}
        />
        <div className={SETTINGS_API_KEY_BUTTON_ROW_CLASS}>
          <button
            type="button"
            className={SETTINGS_CONTROL_BUTTON_CLASS}
            data-remote-api-key-save={kind}
            disabled={saveDisabled || undefined}
            aria-disabled={saveDisabled ? "true" : undefined}
            onClick={handlers.onSave}
          >
            Save API key
          </button>
          <button
            type="button"
            className={SETTINGS_CONTROL_BUTTON_SECONDARY_CLASS}
            data-remote-api-key-test={kind}
            disabled={testDisabled || undefined}
            aria-disabled={testDisabled ? "true" : undefined}
            onClick={handlers.onTest}
          >
            {isTestingApiKey
              ? (
                <>
                  <span className={BTN_SPINNER_CLASS} data-btn-spinner="true" aria-hidden="true" />
                  Testing...
                </>
              )
              : "Test API key"}
          </button>
        </div>
      </div>
      <p className={SETTINGS_PANEL_DESCRIPTION_CLASS}>
        Need an OpenAI API key? Get one at{" "}
        <a
          href={OPENAI_API_KEYS_URL}
          target="_blank"
          rel="noreferrer"
          data-external-link-url={OPENAI_API_KEYS_URL}
          onClick={handlers.onOpenExternalLink
            ? (event: { preventDefault: () => void }) => {
              event.preventDefault();
              handlers.onOpenExternalLink?.(OPENAI_API_KEYS_URL);
            }
            : undefined}
        >
          {OPENAI_API_KEYS_URL}
        </a>.
      </p>
      {apiKeyTestMessage
        ? (
          <div className={SETTINGS_API_KEY_TEST_STATUS_CLASS} data-api-key-test-status="true" role="status" aria-live="polite">
            <p className={SETTINGS_API_KEY_TEST_STATUS_LABEL_CLASS}>Latest test result</p>
            <p className={SETTINGS_API_KEY_TEST_STATUS_MESSAGE_CLASS}>
              {renderTextWithKnownLinkNodes(apiKeyTestMessage, handlers.onOpenExternalLink)}
            </p>
          </div>
        )
        : null}
    </div>
  );
}

type ConfirmationErrorVariant = "none" | "transport" | "tool-retryable" | "tool-hard-stop";

function confirmationErrorVariant(
  state: Extract<ConfirmationUiState, { kind: "awaiting-confirmation" }>,
): ConfirmationErrorVariant {
  if (!state.submissionError) {
    return "none";
  }
  if (state.submissionError.kind === "transport-error") {
    return "transport";
  }
  return state.submissionError.retryable ? "tool-retryable" : "tool-hard-stop";
}

// Each variant is a full, self-contained class string (base anatomy +
// its own border/background/color) rather than base-plus-delta, matching
// this migration's established pattern for mutually exclusive visual states.
//
// CR3 P3.3.3: the container itself is always mounted (deliberate and
// correct -- an aria-live region has to already be in the DOM before its
// content changes for assistive tech to announce it), but the "none" variant
// used to reuse the same padding/rounded-corners/1px-border anatomy as every
// error variant, so a permanent empty red-bordered strip rendered even with
// no error. "none" is the only variant with no visible chrome at all -- an
// empty string, not `CONFIRMATION_ERROR_CONTAINER_BASE` plus zero color --
// so nothing (no box, no spacing) is visible until there is actually an
// error to show.
const CONFIRMATION_ERROR_CONTAINER_BASE = "mt-3 p-[12px_14px] rounded-[14px] leading-[1.5]";
const CONFIRMATION_ERROR_CONTAINER_CLASS: Record<ConfirmationErrorVariant, string> = {
  none: "",
  transport: `${CONFIRMATION_ERROR_CONTAINER_BASE} border border-[rgba(122,87,39,0.22)] text-[var(--color-amber-active)] bg-[var(--color-amber-light)]`,
  "tool-retryable": `${CONFIRMATION_ERROR_CONTAINER_BASE} border border-[rgba(150,39,39,0.18)] border-l-4 border-l-[rgba(150,39,39,0.48)] text-[var(--color-error-primary)] bg-[rgba(150,39,39,0.09)]`,
  "tool-hard-stop": `${CONFIRMATION_ERROR_CONTAINER_BASE} border border-[rgba(108,16,16,0.5)] border-l-[6px] border-l-[var(--color-error-dark)] text-[var(--color-error-dark)] bg-gradient-to-b from-[rgba(108,16,16,0.12)] to-[rgba(150,39,39,0.18)] shadow-[inset_0_0_0_1px_rgba(108,16,16,0.1)]`,
};

// The badge, meta-code, and retry-status rows only ever render for the
// "tool-error" cases in confirmation.tsx; the badge specifically only for
// the hard-stop case, so its class always matches that variant's computed
// style (the old `.confirmation-error-tool-hard-stop .confirmation-error-badge`
// override is the ONLY style it's ever actually rendered with).
export const CONFIRMATION_ERROR_BADGE_CLASS = "m-0 inline-flex items-center mb-[10px] py-1 px-[10px] rounded-full bg-[rgba(108,16,16,0.2)] border border-[rgba(108,16,16,0.42)] text-[var(--color-error-dark)] text-[0.8rem] font-bold tracking-[0.08em] uppercase";
export const CONFIRMATION_ERROR_MESSAGE_CLASS = "m-0 mt-[6px]";

export interface ConfirmationErrorPresentation {
  containerClassName: string;
  showBadge: boolean;
  titleClassName: string;
  guidanceClassName: string;
  showMeta: boolean;
  metaBlockClassName: string;
  metaClassName: string;
  retryStatusClassName: string;
}

export function resolveConfirmationErrorPresentation(
  state: Extract<ConfirmationUiState, { kind: "awaiting-confirmation" }>,
): ConfirmationErrorPresentation {
  const variant = confirmationErrorVariant(state);
  const isHardStop = variant === "tool-hard-stop";
  const isToolError = variant === "tool-retryable" || isHardStop;
  const retryStatusColor = isHardStop ? "text-[var(--color-error-dark)]" : "text-[var(--color-error-primary)]";

  return {
    containerClassName: CONFIRMATION_ERROR_CONTAINER_CLASS[variant],
    showBadge: isHardStop,
    titleClassName: isHardStop ? "m-0 font-bold tracking-[0.04em] uppercase" : "m-0 font-bold",
    guidanceClassName: isHardStop ? "m-0 mt-[6px] font-semibold" : "m-0 mt-[6px]",
    showMeta: isToolError,
    metaBlockClassName: "mt-[6px]",
    metaClassName: `m-0 mt-[6px] text-[0.94rem] ${isHardStop ? "text-[var(--color-error-dark)]" : "text-[var(--color-error-primary)]"}`,
    retryStatusClassName: `m-0 mt-1 text-[0.88rem] font-bold tracking-[0.04em] uppercase ${retryStatusColor}`,
  };
}
