import { type ReactNode } from "react";

import { renderSecretEntryCard } from "../confirmation-panel-helpers.tsx";
import type { RemotePlannerPanelState } from "../panel-types.ts";
import { renderConnectedRemotePlannerPrivacySettingsCard } from "./planner-privacy.tsx";
import {
  BTN_SPINNER_CLASS,
  CONTROL_LABEL,
  SETTINGS_BUTTON_ROW_WRAP_CLASS,
  SETTINGS_CONTROL_BUTTON_CLASS,
  SETTINGS_CONTROL_BUTTON_DANGER_CLASS,
  SETTINGS_CONTROL_BUTTON_SECONDARY_CLASS,
  SETTINGS_CONTROL_SELECT_CLASS,
  SETTINGS_FIELD_GROUP_CLASS,
  SETTINGS_GRID_SINGLE_CLASS,
  SETTINGS_GRID_SINGLE_COMPACT_CLASS,
  SETTINGS_INLINE_CONTROL_FILL_CLASS,
  SETTINGS_INLINE_CONTROL_ROW_WRAP_CLASS,
  SETTINGS_INLINE_LABEL_ROW_CLASS,
  SETTINGS_INLINE_LOADING_CLASS,
  SETTINGS_MODEL_FRESHNESS_INDICATOR_CLASS,
  SETTINGS_MODEL_FRESHNESS_LABEL_CLASS,
  SETTINGS_PANEL_WARNING_CLASS,
  SETTINGS_PLANNER_CONNECTION_CARD_CLASS,
  SETTINGS_RESET_CONFIRM_MESSAGE_CLASS,
  SETTINGS_RESET_CONFIRM_ROW_CLASS,
  SETTINGS_STATUS_LIGHT_CLASS,
  SETTINGS_STATUS_LIGHT_FRESH_CLASS,
  SETTINGS_STATUS_LIGHT_STALE_CLASS,
  renderSettingsPanelSection,
} from "./shared-controls.tsx";

export interface RemotePlannerPanelHandlers {
  onApiKeyInput?: (value: string) => void;
  onSaveApiKey?: () => void;
  onTestApiKey?: () => void;
  onOpenExternalLink?: (url: string) => void;
  onEndpointInput?: (value: string) => void;
  onEndpointBlur?: () => void;
  onModelSelect?: (value: string) => void;
  onModelInput?: (value: string) => void;
  onLoadModels?: () => void;
  onSaveSettings?: () => void;
  onBeginReset?: () => void;
  onConfirmReset?: () => void;
  onCancelReset?: () => void;
  onDismissError?: () => void;
  onRetry?: () => void;
}

export function renderSettingsRemotePlannerPanelNode(
  state: RemotePlannerPanelState,
  handlers?: RemotePlannerPanelHandlers,
): ReactNode {
  const baseUrlTrimmed = state.baseUrl?.trim() ?? "";
  const modelsAreFresh = baseUrlTrimmed.length > 0
    && state.loadedModelsEndpoint === state.baseUrl
    && state.availableModels.length > 0;
  const hasLoadedModels = state.availableModels.length > 0;
  const modelsNotLoadedForEndpoint = baseUrlTrimmed.length > 0 && !modelsAreFresh;
  const isConnectionBusy = state.isLoadingModels || state.isSavingConnection || state.isResettingConnection;
  const modelDisabled = isConnectionBusy || !hasLoadedModels;
  const loadModelsDisabled = isConnectionBusy || baseUrlTrimmed.length === 0;
  const saveSettingsDisabled = isConnectionBusy
    || !state.profileName
    || baseUrlTrimmed.length === 0
    || (state.model?.trim().length ?? 0) === 0;
  const resetSettingsDisabled = isConnectionBusy || !state.profileName;
  const modelOptions = hasLoadedModels ? state.availableModels : [];

  return renderSettingsPanelSection({
    titleId: "settings-remote-planner-title",
    title: "AI assistant setup",
    description: "Set the endpoint, model, API key, and fail-closed site privacy rules used to interpret commands.",
    error: state.error,
    onDismissError: handlers?.onDismissError,
    onRetry: handlers?.onRetry,
    children: [
      renderConnectedRemotePlannerPrivacySettingsCard(),
      <div className={SETTINGS_GRID_SINGLE_CLASS} key="planner-api">
        {renderSecretEntryCard(
          "planner",
          state.profileName,
          state.apiKeyDraft,
          state.apiKeyMaskedValue,
          state.isSavingApiKey,
          state.isTestingApiKey,
          state.apiKeyReference !== null,
          state.apiKeyTestMessage,
          {
            onInput: handlers?.onApiKeyInput,
            onSave: handlers?.onSaveApiKey,
            onTest: handlers?.onTestApiKey,
            onOpenExternalLink: handlers?.onOpenExternalLink,
          },
        )}
        {state.apiKeyReferenceError ? (
          <p className={SETTINGS_PANEL_WARNING_CLASS} role="alert">
            {state.apiKeyReferenceError}
          </p>
        ) : null}
      </div>,
      <div className={SETTINGS_GRID_SINGLE_COMPACT_CLASS} key="planner-endpoint">
        <div className={SETTINGS_PLANNER_CONNECTION_CARD_CLASS}>
          <label className={SETTINGS_FIELD_GROUP_CLASS} htmlFor="settings-remote-planner-endpoint-input">
            <span className={CONTROL_LABEL}>Endpoint</span>
            <input
              id="settings-remote-planner-endpoint-input"
              className={SETTINGS_CONTROL_SELECT_CLASS}
              data-remote-planner-endpoint-input="true"
              type="text"
              value={state.baseUrl ?? ""}
              placeholder="https://api.openai.com/v1"
              spellCheck={false}
              autoComplete="off"
              disabled={isConnectionBusy || undefined}
              aria-disabled={isConnectionBusy ? "true" : undefined}
              onChange={handlers?.onEndpointInput
                ? (event) => { handlers.onEndpointInput?.(event.currentTarget.value); }
                : undefined}
              onBlur={handlers?.onEndpointBlur}
            />
          </label>
          {state.isLoadingModels ? (
            <span className={SETTINGS_INLINE_LOADING_CLASS} data-inline-loading="true" role="status" aria-live="polite">
              <span className={BTN_SPINNER_CLASS} data-btn-spinner="true" aria-hidden="true" /> Loading models...
            </span>
          ) : null}
        </div>
      </div>,
      <div className={SETTINGS_GRID_SINGLE_COMPACT_CLASS} key="planner-model">
        <div className={SETTINGS_PLANNER_CONNECTION_CARD_CLASS}>
          <div className={SETTINGS_FIELD_GROUP_CLASS}>
            <span className={`${CONTROL_LABEL} ${SETTINGS_INLINE_LABEL_ROW_CLASS}`}>
              <span>Model</span>
              <span className={SETTINGS_MODEL_FRESHNESS_INDICATOR_CLASS} aria-hidden="true">
                <span className={`${SETTINGS_STATUS_LIGHT_CLASS} ${modelsAreFresh ? SETTINGS_STATUS_LIGHT_FRESH_CLASS : SETTINGS_STATUS_LIGHT_STALE_CLASS}`} />
                <span className={SETTINGS_MODEL_FRESHNESS_LABEL_CLASS}>
                  {modelsAreFresh ? "Model list up to date" : "Model list may be outdated — reload to refresh"}
                </span>
              </span>
              <span className="sr-only">
                {modelsAreFresh
                  ? "Model list is loaded for the current endpoint"
                  : "Model list has not been loaded for the current endpoint"}
              </span>
            </span>
            {hasLoadedModels ? (
              <div className={SETTINGS_INLINE_CONTROL_ROW_WRAP_CLASS}>
                <select
                  id="settings-remote-planner-model-select"
                  className={`${SETTINGS_CONTROL_SELECT_CLASS} ${SETTINGS_INLINE_CONTROL_FILL_CLASS}`}
                  data-remote-planner-model-select="true"
                  value={state.model ?? ""}
                  disabled={modelDisabled || undefined}
                  aria-disabled={modelDisabled ? "true" : undefined}
                  onChange={handlers?.onModelSelect
                    ? (event) => { handlers.onModelSelect?.(event.currentTarget.value); }
                    : () => undefined}
                >
                  {modelOptions.map((model) => (
                    <option key={model} value={model}>{model}</option>
                  ))}
                </select>
                <button
                  type="button"
                  className={SETTINGS_CONTROL_BUTTON_SECONDARY_CLASS}
                  data-remote-planner-models-refresh="true"
                  disabled={loadModelsDisabled || undefined}
                  aria-disabled={loadModelsDisabled ? "true" : undefined}
                  onClick={handlers?.onLoadModels}
                >
                  {state.isLoadingModels
                    ? <><span className={BTN_SPINNER_CLASS} data-btn-spinner="true" aria-hidden="true" />Loading models...</>
                    : "Refresh model list"}
                </button>
              </div>
            ) : (
              <div className={SETTINGS_INLINE_CONTROL_ROW_WRAP_CLASS}>
                <button
                  type="button"
                  className={SETTINGS_CONTROL_BUTTON_SECONDARY_CLASS}
                  data-remote-planner-models-refresh="true"
                  disabled={loadModelsDisabled || undefined}
                  aria-disabled={loadModelsDisabled ? "true" : undefined}
                  onClick={handlers?.onLoadModels}
                >
                  {state.isLoadingModels
                    ? <><span className={BTN_SPINNER_CLASS} data-btn-spinner="true" aria-hidden="true" />Loading models...</>
                    : "Refresh model list"}
                </button>
              </div>
            )}
            <label className={SETTINGS_FIELD_GROUP_CLASS} htmlFor="settings-remote-planner-model-input">
              <span className={CONTROL_LABEL}>
                {hasLoadedModels ? "Or enter a model name manually" : "Model name"}
              </span>
              <input
                id="settings-remote-planner-model-input"
                className={SETTINGS_CONTROL_SELECT_CLASS}
                data-remote-planner-model-input="true"
                type="text"
                value={state.model ?? ""}
                placeholder="e.g. gpt-4o"
                spellCheck={false}
                autoComplete="off"
                disabled={isConnectionBusy || undefined}
                aria-disabled={isConnectionBusy ? "true" : undefined}
                onChange={handlers?.onModelInput
                  ? (event) => { handlers.onModelInput?.(event.currentTarget.value); }
                  : undefined}
              />
            </label>
            {modelsNotLoadedForEndpoint && !state.isLoadingModels ? (
              <p className={SETTINGS_PANEL_WARNING_CLASS}>
                Model list hasn't been loaded for this endpoint — make sure the model name is correct before saving.
              </p>
            ) : null}
          </div>
          {state.isConfirmingReset
            ? (
              <div className={SETTINGS_RESET_CONFIRM_ROW_CLASS}>
                <p className={SETTINGS_RESET_CONFIRM_MESSAGE_CLASS}>Reset all settings to defaults? This cannot be undone.</p>
                <button
                  type="button"
                  className={SETTINGS_CONTROL_BUTTON_DANGER_CLASS}
                  data-remote-planner-settings-confirm-reset="true"
                  disabled={resetSettingsDisabled || undefined}
                  aria-disabled={resetSettingsDisabled ? "true" : undefined}
                  onClick={handlers?.onConfirmReset}
                >
                  {state.isResettingConnection ? "Resetting..." : "Yes, reset"}
                </button>
                <button
                  type="button"
                  className={SETTINGS_CONTROL_BUTTON_SECONDARY_CLASS}
                  data-remote-planner-settings-cancel-reset="true"
                  disabled={resetSettingsDisabled || undefined}
                  aria-disabled={resetSettingsDisabled ? "true" : undefined}
                  onClick={handlers?.onCancelReset}
                >
                  Cancel
                </button>
              </div>
            )
            : (
              <div className={SETTINGS_BUTTON_ROW_WRAP_CLASS}>
                <button
                  type="button"
                  className={SETTINGS_CONTROL_BUTTON_CLASS}
                  data-remote-planner-settings-save="true"
                  disabled={saveSettingsDisabled || undefined}
                  aria-disabled={saveSettingsDisabled ? "true" : undefined}
                  onClick={handlers?.onSaveSettings}
                >
                  {state.isSavingConnection
                    ? <><span className={BTN_SPINNER_CLASS} data-btn-spinner="true" aria-hidden="true" />Saving...</>
                    : "Save settings"}
                </button>
                <button
                  type="button"
                  className={SETTINGS_CONTROL_BUTTON_SECONDARY_CLASS}
                  data-remote-planner-settings-reset="true"
                  disabled={resetSettingsDisabled || undefined}
                  aria-disabled={resetSettingsDisabled ? "true" : undefined}
                  onClick={handlers?.onBeginReset}
                >
                  Reset to defaults
                </button>
              </div>
            )}
        </div>
      </div>,
    ],
  });
}
