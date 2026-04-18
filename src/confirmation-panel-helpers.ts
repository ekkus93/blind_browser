import type { ConfirmationUiState } from "./planner-orchestration";

export const OPENAI_API_KEYS_URL = "https://platform.openai.com/account/api-keys";

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

export function renderReadOnlySettingValue(value: string | number | null): string {
  if (value === null) {
    return "Not configured";
  }

  return escapeHtml(`${value}`);
}

export function renderModelAvailabilityLabel(available: boolean): string {
  return available ? "Downloaded" : "Missing";
}

export function renderOpenAiApiKeysLink(label: string = OPENAI_API_KEYS_URL): string {
  return `<a href="${escapeHtml(OPENAI_API_KEYS_URL)}" target="_blank" rel="noreferrer" data-external-link-url="${escapeHtml(OPENAI_API_KEYS_URL)}">${escapeHtml(label)}</a>`;
}

export function renderTextWithKnownLinks(value: string): string {
  return escapeHtml(value).split(OPENAI_API_KEYS_URL).join(renderOpenAiApiKeysLink());
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
): string {
  const disabledAttribute = isSavingApiKey || isTestingApiKey ? " disabled aria-disabled=\"true\"" : "";
  const saveDisabledAttribute =
    isSavingApiKey || isTestingApiKey || profileName === null || apiKeyDraft.trim().length === 0
      ? " disabled aria-disabled=\"true\""
      : "";
  const testDisabledAttribute =
    isSavingApiKey
    || isTestingApiKey
    || profileName === null
    || (apiKeyDraft.trim().length === 0 && !hasApiKeyReference)
      ? " disabled aria-disabled=\"true\""
      : "";
  const displayedApiKeyValue = apiKeyDraft.length > 0 ? apiKeyDraft : (apiKeyMaskedValue ?? "");
  const inputType = apiKeyDraft.length > 0 ? "password" : (apiKeyMaskedValue ? "text" : "password");
  const maskedDisplayAttribute = apiKeyDraft.length === 0 && apiKeyMaskedValue
    ? ` data-masked-api-key-display="${escapeHtml(apiKeyMaskedValue)}"`
    : "";
  const testStatusCopy = apiKeyTestMessage
    ? `
      <div class="settings-api-key-test-status" role="status" aria-live="polite">
        <p class="settings-api-key-test-status-label">Latest test result</p>
        <p class="settings-api-key-test-status-message">${renderTextWithKnownLinks(apiKeyTestMessage)}</p>
      </div>
    `
    : "";

  return `
    <div class="settings-control-card settings-secret-entry-card">
      <span class="settings-control-label">API key</span>
      <div class="settings-api-key-inline-actions">
        <input
          id="settings-remote-${kind}-api-key-input"
          class="settings-control-select settings-api-key-input"
          data-remote-api-key-input="${escapeHtml(kind)}"
          type="${inputType}"
          value="${escapeHtml(displayedApiKeyValue)}"
          placeholder="Enter a replacement API key"
          autocomplete="off"
          spellcheck="false"
          ${maskedDisplayAttribute}
          ${disabledAttribute}
        />
        <div class="settings-api-key-button-row">
          <button
            type="button"
            class="settings-control-button"
            data-remote-api-key-save="${escapeHtml(kind)}"
            ${saveDisabledAttribute}
          >
            Save API key
          </button>
          <button
            type="button"
            class="settings-control-button settings-control-button-secondary"
            data-remote-api-key-test="${escapeHtml(kind)}"
            ${testDisabledAttribute}
          >
            ${escapeHtml(isTestingApiKey ? "Testing..." : "Test API key")}
          </button>
        </div>
      </div>
      <p class="settings-panel-description">
        Need an OpenAI API key? Get one at ${renderOpenAiApiKeysLink()}.
      </p>
      ${testStatusCopy}
    </div>
  `;
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
