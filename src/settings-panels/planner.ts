import { createElement, type ReactNode } from "react";

import { renderSecretEntryCard } from "../confirmation-panel-helpers.ts";
import type { RemotePlannerPanelState } from "../panel-types.ts";
import { renderSettingsPanelSection } from "./shared-controls.ts";

const h = createElement;

export interface RemotePlannerPanelHandlers {
  onApiKeyInput?: (value: string) => void;
  onSaveApiKey?: () => void;
  onTestApiKey?: () => void;
  onOpenExternalLink?: (url: string) => void;
  onEndpointInput?: (value: string) => void;
  onModelSelect?: (value: string) => void;
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
  const modelsAreFresh = (state.baseUrl?.trim().length ?? 0) > 0
    && state.loadedModelsEndpoint === state.baseUrl
    && state.availableModels.length > 0;
  const isConnectionBusy = state.isLoadingModels || state.isSavingConnection || state.isResettingConnection;
  const modelDisabled = isConnectionBusy || state.availableModels.length === 0;
  const loadModelsDisabled = isConnectionBusy || (state.baseUrl?.trim().length ?? 0) === 0;
  const saveSettingsDisabled = isConnectionBusy
    || !state.profileName
    || (state.baseUrl?.trim().length ?? 0) === 0
    || (state.model?.trim().length ?? 0) === 0
    || state.loadedModelsEndpoint !== state.baseUrl;
  const resetSettingsDisabled = isConnectionBusy || !state.profileName;
  const modelOptions = state.availableModels.length > 0
    ? state.availableModels
    : [state.loadedModelsEndpoint === state.baseUrl && state.model ? state.model : "Load models for this endpoint"];

  return renderSettingsPanelSection({
    titleId: "settings-remote-planner-title",
    title: "AI assistant setup",
    description: "Set the endpoint, model, and API key used to interpret commands.",
    error: state.error,
    children: [
      h(
        "div",
        { className: "settings-grid settings-grid-single", key: "planner-api" },
        renderSecretEntryCard(
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
        ),
      ),
      h(
        "div",
        { className: "settings-grid settings-grid-single settings-grid-compact", key: "planner-endpoint" },
        h(
          "div",
          { className: "settings-control-card settings-planner-connection-card" },
          h(
            "label",
            { className: "settings-field-group", htmlFor: "settings-remote-planner-endpoint-input" },
            h("span", { className: "settings-control-label" }, "Endpoint"),
            h("input", {
              id: "settings-remote-planner-endpoint-input",
              className: "settings-control-select",
              "data-remote-planner-endpoint-input": "true",
              type: "text",
              value: state.baseUrl ?? "",
              placeholder: "https://api.openai.com/v1",
              spellCheck: false,
              autoComplete: "off",
              disabled: isConnectionBusy || undefined,
              "aria-disabled": isConnectionBusy ? "true" : undefined,
              onChange: handlers?.onEndpointInput
                ? (event: { currentTarget: { value: string } }) => {
                  handlers.onEndpointInput?.(event.currentTarget.value);
                }
                : undefined,
            }),
          ),
        ),
      ),
      h(
        "div",
        { className: "settings-grid settings-grid-single settings-grid-compact", key: "planner-model" },
        h(
          "div",
          { className: "settings-control-card settings-planner-connection-card" },
          h(
            "label",
            { className: "settings-field-group", htmlFor: "settings-remote-planner-model-select" },
            h(
              "span",
              { className: "settings-control-label settings-inline-label-row" },
              h("span", null, "Model"),
              h("span", {
                className: `settings-status-light ${modelsAreFresh ? "settings-status-light-fresh" : "settings-status-light-stale"}`,
                role: "img",
                "aria-label": modelsAreFresh
                  ? "Models are loaded for the current endpoint"
                  : "Models need to be reloaded for the current endpoint",
              }),
            ),
            h(
              "div",
              { className: "settings-inline-control-row settings-inline-control-row-wrap" },
              h(
                "select",
                {
                  id: "settings-remote-planner-model-select",
                  className: "settings-control-select settings-inline-control-fill",
                  "data-remote-planner-model-select": "true",
                  value: state.model ?? "",
                  disabled: modelDisabled || undefined,
                  "aria-disabled": modelDisabled ? "true" : undefined,
                  onChange: handlers?.onModelSelect
                    ? (event: { currentTarget: { value: string } }) => {
                      handlers.onModelSelect?.(event.currentTarget.value);
                    }
                    : () => undefined,
                },
                ...modelOptions.map((model) => h("option", { value: model, key: model }, model)),
              ),
              h(
                "button",
                {
                  type: "button",
                  className: "settings-control-button settings-control-button-secondary",
                  "data-remote-planner-models-refresh": "true",
                  disabled: loadModelsDisabled || undefined,
                  "aria-disabled": loadModelsDisabled ? "true" : undefined,
                  onClick: handlers?.onLoadModels,
                },
                state.isLoadingModels ? "Loading models..." : "Load models",
              ),
            ),
          ),
          state.isConfirmingReset
            ? h(
              "div",
              { className: "settings-button-row settings-reset-confirm-row" },
              h("p", { className: "settings-reset-confirm-message" }, "Reset all settings to defaults? This cannot be undone."),
              h(
                "button",
                {
                  type: "button",
                  className: "settings-control-button settings-control-button-danger",
                  "data-remote-planner-settings-confirm-reset": "true",
                  disabled: resetSettingsDisabled || undefined,
                  "aria-disabled": resetSettingsDisabled ? "true" : undefined,
                  onClick: handlers?.onConfirmReset,
                },
                state.isResettingConnection ? "Resetting..." : "Yes, reset",
              ),
              h(
                "button",
                {
                  type: "button",
                  className: "settings-control-button settings-control-button-secondary",
                  "data-remote-planner-settings-cancel-reset": "true",
                  disabled: resetSettingsDisabled || undefined,
                  "aria-disabled": resetSettingsDisabled ? "true" : undefined,
                  onClick: handlers?.onCancelReset,
                },
                "Cancel",
              ),
            )
            : h(
              "div",
              { className: "settings-button-row settings-button-row-wrap" },
              h(
                "button",
                {
                  type: "button",
                  className: "settings-control-button",
                  "data-remote-planner-settings-save": "true",
                  disabled: saveSettingsDisabled || undefined,
                  "aria-disabled": saveSettingsDisabled ? "true" : undefined,
                  onClick: handlers?.onSaveSettings,
                },
                state.isSavingConnection ? "Saving..." : "Save settings",
              ),
              h(
                "button",
                {
                  type: "button",
                  className: "settings-control-button settings-control-button-secondary",
                  "data-remote-planner-settings-reset": "true",
                  disabled: resetSettingsDisabled || undefined,
                  "aria-disabled": resetSettingsDisabled ? "true" : undefined,
                  onClick: handlers?.onBeginReset,
                },
                "Reset to defaults",
              ),
            ),
        ),
      ),
    ],
  });
}