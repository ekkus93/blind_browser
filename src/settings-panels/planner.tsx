import { type ReactNode } from "react";

import { renderSecretEntryCard } from "../confirmation-panel-helpers.ts";
import type { RemotePlannerPanelState } from "../panel-types.ts";
import { renderSettingsPanelSection } from "./shared-controls.tsx";

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
}

export function renderSettingsRemotePlannerPanelNode(
  state: RemotePlannerPanelState,
  handlers?: RemotePlannerPanelHandlers,
): ReactNode {
  const baseUrlTrimmed = state.baseUrl?.trim() ?? "";
  const modelsAreFresh = baseUrlTrimmed.length > 0
    && state.loadedModelsEndpoint === state.baseUrl
    && state.availableModels.length > 0;
  const isConnectionBusy = state.isLoadingModels || state.isSavingConnection || state.isResettingConnection;
  const modelDisabled = isConnectionBusy || state.availableModels.length === 0;
  const loadModelsDisabled = isConnectionBusy || baseUrlTrimmed.length === 0;
  const saveSettingsDisabled = isConnectionBusy
    || !state.profileName
    || baseUrlTrimmed.length === 0
    || (state.model?.trim().length ?? 0) === 0;
  const resetSettingsDisabled = isConnectionBusy || !state.profileName;
  const hasLoadedModels = state.availableModels.length > 0;
  const modelsNotLoadedForEndpoint = baseUrlTrimmed.length > 0 && state.loadedModelsEndpoint !== state.baseUrl;
  const currentModelForEndpoint = !hasLoadedModels && state.loadedModelsEndpoint === state.baseUrl ? state.model : null;
  const modelOptions = hasLoadedModels ? state.availableModels : (currentModelForEndpoint ? [currentModelForEndpoint] : []);

  return renderSettingsPanelSection({
    titleId: "settings-remote-planner-title",
    title: "AI assistant setup",
    description: "Set the endpoint, model, and API key used to interpret commands.",
    error: state.error,
    children: [
      <div className="settings-grid settings-grid-single" key="planner-api">
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
      </div>,
      <div className="settings-grid settings-grid-single settings-grid-compact" key="planner-endpoint">
        <div className="settings-control-card settings-planner-connection-card">
          <label className="settings-field-group" htmlFor="settings-remote-planner-endpoint-input">
            <span className="settings-control-label">Endpoint</span>
            <input
              id="settings-remote-planner-endpoint-input"
              className="settings-control-select"
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
        </div>
      </div>,
      <div className="settings-grid settings-grid-single settings-grid-compact" key="planner-model">
        <div className="settings-control-card settings-planner-connection-card">
          <div className="settings-field-group">
            <span className="settings-control-label settings-inline-label-row">
              <span>Model</span>
              <span className="settings-model-freshness-indicator" aria-hidden="true">
                <span className={`settings-status-light ${modelsAreFresh ? "settings-status-light-fresh" : "settings-status-light-stale"}`} />
                <span className="settings-model-freshness-label">
                  {modelsAreFresh ? "Up to date" : "Reload needed"}
                </span>
              </span>
              <span className="sr-only">
                {modelsAreFresh
                  ? "Models are loaded for the current endpoint"
                  : "Models need to be reloaded for the current endpoint"}
              </span>
            </span>
            {hasLoadedModels ? (
              <div className="settings-inline-control-row settings-inline-control-row-wrap">
                <select
                  id="settings-remote-planner-model-select"
                  className="settings-control-select settings-inline-control-fill"
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
                  className="settings-control-button settings-control-button-secondary"
                  data-remote-planner-models-refresh="true"
                  disabled={loadModelsDisabled || undefined}
                  aria-disabled={loadModelsDisabled ? "true" : undefined}
                  onClick={handlers?.onLoadModels}
                >
                  {state.isLoadingModels
                    ? <><span className="btn-spinner" aria-hidden="true" />Loading models...</>
                    : "Load models"}
                </button>
              </div>
            ) : (
              <div className="settings-inline-control-row settings-inline-control-row-wrap">
                <button
                  type="button"
                  className="settings-control-button settings-control-button-secondary"
                  data-remote-planner-models-refresh="true"
                  disabled={loadModelsDisabled || undefined}
                  aria-disabled={loadModelsDisabled ? "true" : undefined}
                  onClick={handlers?.onLoadModels}
                >
                  {state.isLoadingModels
                    ? <><span className="btn-spinner" aria-hidden="true" />Loading models...</>
                    : "Load models"}
                </button>
              </div>
            )}
            <label className="settings-field-group" htmlFor="settings-remote-planner-model-input">
              <span className="settings-control-label">
                {hasLoadedModels ? "Or enter a model name manually" : "Model name"}
              </span>
              <input
                id="settings-remote-planner-model-input"
                className="settings-control-select"
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
              <p className="settings-panel-description settings-panel-warning">
                Model list hasn't been loaded for this endpoint — make sure the model name is correct before saving.
              </p>
            ) : null}
          </div>
          {state.isConfirmingReset
            ? (
              <div className="settings-button-row settings-reset-confirm-row">
                <p className="settings-reset-confirm-message">Reset all settings to defaults? This cannot be undone.</p>
                <button
                  type="button"
                  className="settings-control-button settings-control-button-danger"
                  data-remote-planner-settings-confirm-reset="true"
                  disabled={resetSettingsDisabled || undefined}
                  aria-disabled={resetSettingsDisabled ? "true" : undefined}
                  onClick={handlers?.onConfirmReset}
                >
                  {state.isResettingConnection ? "Resetting..." : "Yes, reset"}
                </button>
                <button
                  type="button"
                  className="settings-control-button settings-control-button-secondary"
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
              <div className="settings-button-row settings-button-row-wrap">
                <button
                  type="button"
                  className="settings-control-button"
                  data-remote-planner-settings-save="true"
                  disabled={saveSettingsDisabled || undefined}
                  aria-disabled={saveSettingsDisabled ? "true" : undefined}
                  onClick={handlers?.onSaveSettings}
                >
                  {state.isSavingConnection
                    ? <><span className="btn-spinner" aria-hidden="true" />Saving...</>
                    : "Save settings"}
                </button>
                <button
                  type="button"
                  className="settings-control-button settings-control-button-secondary"
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
