import type { ConfirmationUiState } from "./planner-orchestration";

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

export function renderSecretEntryCard(
  kind: "planner" | "tts" | "asr",
  profileName: string | null,
  apiKeyDraft: string,
  isSavingApiKey: boolean,
): string {
  const disabledAttribute = isSavingApiKey ? " disabled aria-disabled=\"true\"" : "";
  const saveDisabledAttribute =
    isSavingApiKey || profileName === null || apiKeyDraft.trim().length === 0
      ? " disabled aria-disabled=\"true\""
      : "";

  return `
    <div class="settings-control-card settings-secret-entry-card">
      <span class="settings-control-label">Secure API key entry</span>
      <span class="settings-control-value">Store in OS keyring</span>
      <input
        id="settings-remote-${kind}-api-key-input"
        class="settings-control-select"
        data-remote-api-key-input="${escapeHtml(kind)}"
        type="password"
        value="${escapeHtml(apiKeyDraft)}"
        placeholder="Enter a replacement API key"
        autocomplete="off"
        spellcheck="false"
        ${disabledAttribute}
      />
      <button
        type="button"
        class="settings-control-button"
        data-remote-api-key-save="${escapeHtml(kind)}"
        ${saveDisabledAttribute}
      >
        Save API key
      </button>
      <p class="settings-panel-description">
        Saving stores the secret in the OS keyring and updates the config to keep only a masked
        keyring reference.
      </p>
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
    .replace(/\"/g, "&quot;")
    .replace(/'/g, "&#39;");
}
