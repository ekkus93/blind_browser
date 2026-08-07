import type { ReactNode } from "react";
import type { ConfirmationUiState } from "./planner-orchestration";

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

export function renderOpenAiApiKeysLink(label: string = OPENAI_API_KEYS_URL): string {
  return `<a href="${escapeHtml(OPENAI_API_KEYS_URL)}" target="_blank" rel="noreferrer" data-external-link-url="${escapeHtml(OPENAI_API_KEYS_URL)}">${escapeHtml(label)}</a>`;
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
    <div className="settings-control-card settings-secret-entry-card">
      <span className="settings-control-label">API key</span>
      <div className="settings-api-key-inline-actions">
        <input
          id={`settings-remote-${kind}-api-key-input`}
          className="settings-control-select settings-api-key-input"
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
        <div className="settings-api-key-button-row">
          <button
            type="button"
            className="settings-control-button"
            data-remote-api-key-save={kind}
            disabled={saveDisabled || undefined}
            aria-disabled={saveDisabled ? "true" : undefined}
            onClick={handlers.onSave}
          >
            Save API key
          </button>
          <button
            type="button"
            className="settings-control-button settings-control-button-secondary"
            data-remote-api-key-test={kind}
            disabled={testDisabled || undefined}
            aria-disabled={testDisabled ? "true" : undefined}
            onClick={handlers.onTest}
          >
            {isTestingApiKey
              ? (
                <>
                  <span className="btn-spinner" aria-hidden="true" />
                  Testing...
                </>
              )
              : "Test API key"}
          </button>
        </div>
      </div>
      <p className="settings-panel-description">
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
          <div className="settings-api-key-test-status" role="status" aria-live="polite">
            <p className="settings-api-key-test-status-label">Latest test result</p>
            <p className="settings-api-key-test-status-message">
              {renderTextWithKnownLinkNodes(apiKeyTestMessage, handlers.onOpenExternalLink)}
            </p>
          </div>
        )
        : null}
    </div>
  );
}

export function renderConfirmationErrorClassName(
  state: Extract<ConfirmationUiState, { kind: "awaiting-confirmation" }>,
): string {
  const classNames = ["confirmation-error"];

  if (!state.submissionError) {
    return classNames.join(" ");
  }

  if (state.submissionError.kind === "transport-error") {
    classNames.push("confirmation-error-transport");
    return classNames.join(" ");
  }

  classNames.push("confirmation-error-tool");
  classNames.push(
    state.submissionError.retryable
      ? "confirmation-error-tool-retryable"
      : "confirmation-error-tool-hard-stop",
  );

  return classNames.join(" ");
}

export function renderConfirmationErrorBadge(
  state: Extract<ConfirmationUiState, { kind: "awaiting-confirmation" }>,
): string {
  if (
    !state.submissionError
    || state.submissionError.kind !== "tool-error"
    || state.submissionError.retryable
  ) {
    return "";
  }

  return '<p class="confirmation-error-badge">Requires planner change</p>';
}

export function renderConfirmationErrorMeta(
  state: Extract<ConfirmationUiState, { kind: "awaiting-confirmation" }>,
): string {
  if (!state.submissionError || state.submissionError.kind !== "tool-error") {
    return "";
  }

  const retryableLabel = state.submissionError.retryable ? "Retryable" : "Non-retryable";
  const retryStatus = state.submissionError.retryable ? "Can retry." : "Cannot retry.";
  return `
    <div class="confirmation-error-meta-block">
      <p class="confirmation-error-meta">
        Error code: ${escapeHtml(state.submissionError.code)}. ${retryableLabel} backend failure.
      </p>
      <p class="confirmation-error-retry-status">${retryStatus}</p>
    </div>
  `;
}

export function escapeHtml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}
