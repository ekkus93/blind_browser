type RemoteApiKeyKind = "planner" | "tts" | "asr";
type ModelDownloadKind = "tts" | "asr";
type BrowserVisibilityMode = "Visible" | "Headless";
type UrlAction = "open" | "read" | "stop" | "previous" | "next";
type AudioControlKind = "volume" | "speed";
type OcrThresholdControlKind = "char" | "region";
type ModelManagementToggleKind = "check-on-startup" | "auto-download-missing";
type PushToTalkSource = "keyboard" | "pointer";
type AppView = "workspace" | "settings";
type SettingsView = "overview" | "planner";
type SettingsBusyKey =
  | "volume"
  | "speed"
  | "confirmation-threshold"
  | "click-without-confirmation"
  | "ocr-char-threshold"
  | "ocr-region-threshold"
  | "model-check-on-startup"
  | "model-auto-download-missing"
  | "models-dir"
  | "asr-provider"
  | "tts-provider"
  | "tts-model"
  | "tts-voice";

interface EventHandlerDependencies {
  appRoot: HTMLDivElement;
  document: Document;
  window: Window;
  isUrlInputActionBusy: () => boolean;
  isBrowserVisibilityUpdating: () => boolean;
  isSettingsActionBusy: (key: SettingsBusyKey) => boolean;
  isPushToTalkKeyEvent: (event: KeyboardEvent) => boolean;
  saveRemoteApiKey: (kind: RemoteApiKeyKind) => void;
  testRemoteApiKey: (kind: RemoteApiKeyKind) => void;
  downloadModel: (kind: ModelDownloadKind) => void;
  setBrowserVisibility: (mode: BrowserVisibilityMode) => void;
  runUrlAction: (action: UrlAction) => void;
  submitConfirmationAction: (action: "approve" | "reject", confirmationId: string) => void;
  updateAudioInput: (kind: AudioControlKind, value: number) => void;
  updateConfirmationThresholdInput: (value: number) => void;
  updateOcrThresholdInput: (kind: OcrThresholdControlKind, value: number) => void;
  updateRemoteApiKeyInput: (kind: RemoteApiKeyKind, value: string) => void;
  updateRemotePlannerEndpointInput: (value: string) => void;
  updateRemotePlannerModelSelection: (value: string) => void;
  updateModelManagementInput: (value: string) => void;
  updateUrlInput: (value: string) => void;
  persistAudioChange: (kind: AudioControlKind, value: number) => void;
  persistConfirmationThreshold: (value: number) => void;
  persistClickWithoutConfirmation: (checked: boolean) => void;
  persistOcrThresholdChange: (kind: OcrThresholdControlKind, value: number) => void;
  persistModelManagementToggle: (kind: ModelManagementToggleKind, checked: boolean) => void;
  persistModelsDir: () => void;
  persistAsrProvider: (mode: "Local" | "Remote") => void;
  persistTtsProvider: (mode: "Local" | "Remote") => void;
  persistTtsModel: (profileName: string) => void;
  persistTtsVoice: (voice: string) => void;
  loadRemotePlannerModels: () => void;
  persistRemotePlannerConnectionSettings: () => void;
  resetRemotePlannerConnectionSettings: () => void;
  openExternalLink: (url: string) => void;
  setAppView: (view: AppView) => void;
  setSettingsView: (view: SettingsView) => void;
  beginPushToTalk: (source: PushToTalkSource) => void;
  releasePushToTalk: (source: PushToTalkSource) => void;
  cancelPushToTalk: () => void;
}

function resolveSettingsViewForTarget(targetId: string): SettingsView {
  if (targetId.startsWith("settings-remote-planner")) {
    return "planner";
  }

  return "overview";
}

export function registerAppEventHandlers({
  appRoot,
  document,
  window,
  isUrlInputActionBusy,
  isBrowserVisibilityUpdating,
  isSettingsActionBusy,
  isPushToTalkKeyEvent,
  saveRemoteApiKey,
  testRemoteApiKey,
  downloadModel,
  setBrowserVisibility,
  runUrlAction,
  submitConfirmationAction,
  updateAudioInput,
  updateConfirmationThresholdInput,
  updateOcrThresholdInput,
  updateRemoteApiKeyInput,
  updateRemotePlannerEndpointInput,
  updateRemotePlannerModelSelection,
  updateModelManagementInput,
  updateUrlInput,
  persistAudioChange,
  persistConfirmationThreshold,
  persistClickWithoutConfirmation,
  persistOcrThresholdChange,
  persistModelManagementToggle,
  persistModelsDir,
  persistAsrProvider,
  persistTtsProvider,
  persistTtsModel,
  persistTtsVoice,
  loadRemotePlannerModels,
  persistRemotePlannerConnectionSettings,
  resetRemotePlannerConnectionSettings,
  openExternalLink,
  setAppView,
  setSettingsView,
  beginPushToTalk,
  releasePushToTalk,
  cancelPushToTalk,
}: EventHandlerDependencies) {
  appRoot.addEventListener("click", (event) => {
    const target = event.target;
    if (!(target instanceof HTMLElement)) {
      return;
    }

    const externalLink = target.closest<HTMLElement>("[data-external-link-url]");
    if (externalLink) {
      const url = externalLink.dataset.externalLinkUrl;
      if (!url) {
        return;
      }

      event.preventDefault();
      openExternalLink(url);
      return;
    }

    const settingsTargetButton = target.closest<HTMLButtonElement>("[data-settings-target]");
    if (settingsTargetButton) {
      const targetId = settingsTargetButton.dataset.settingsTarget;
      if (!targetId) {
        return;
      }

      setAppView("settings");
      setSettingsView(resolveSettingsViewForTarget(targetId));

      const targetElement = document.getElementById(targetId);
      if (!targetElement) {
        return;
      }

      targetElement.scrollIntoView({ behavior: "smooth", block: "center" });
      if (
        targetElement instanceof HTMLInputElement
        || targetElement instanceof HTMLSelectElement
        || targetElement instanceof HTMLButtonElement
        || targetElement instanceof HTMLTextAreaElement
      ) {
        targetElement.focus({ preventScroll: true });
      }
      return;
    }

    const settingsViewButton = target.closest<HTMLButtonElement>("[data-settings-view-button]");
    if (settingsViewButton) {
      const view = settingsViewButton.dataset.settingsViewButton;
      if (view === "overview" || view === "planner") {
        setAppView("settings");
        setSettingsView(view);
      }
      return;
    }

    const appViewButton = target.closest<HTMLButtonElement>("[data-app-view-button]");
    if (appViewButton) {
      const view = appViewButton.dataset.appViewButton;
      if (view === "workspace" || view === "settings") {
        setAppView(view);
      }
      return;
    }

    const remoteApiKeySaveButton = target.closest<HTMLButtonElement>("[data-remote-api-key-save]");
    if (remoteApiKeySaveButton) {
      if (remoteApiKeySaveButton.disabled) {
        return;
      }

      const kind = remoteApiKeySaveButton.dataset.remoteApiKeySave;
      if (kind === "planner" || kind === "tts" || kind === "asr") {
        saveRemoteApiKey(kind);
      }
      return;
    }

    const loadPlannerModelsButton = target.closest<HTMLButtonElement>("[data-remote-planner-models-refresh]");
    if (loadPlannerModelsButton) {
      if (loadPlannerModelsButton.disabled) {
        return;
      }

      loadRemotePlannerModels();
      return;
    }

    const savePlannerSettingsButton = target.closest<HTMLButtonElement>("[data-remote-planner-settings-save]");
    if (savePlannerSettingsButton) {
      if (savePlannerSettingsButton.disabled) {
        return;
      }

      persistRemotePlannerConnectionSettings();
      return;
    }

    const resetPlannerSettingsButton = target.closest<HTMLButtonElement>("[data-remote-planner-settings-reset]");
    if (resetPlannerSettingsButton) {
      if (resetPlannerSettingsButton.disabled) {
        return;
      }

      resetRemotePlannerConnectionSettings();
      return;
    }

    const remoteApiKeyTestButton = target.closest<HTMLButtonElement>("[data-remote-api-key-test]");
    if (remoteApiKeyTestButton) {
      if (remoteApiKeyTestButton.disabled) {
        return;
      }

      const kind = remoteApiKeyTestButton.dataset.remoteApiKeyTest;
      if (kind === "planner" || kind === "tts" || kind === "asr") {
        testRemoteApiKey(kind);
      }
      return;
    }

    const modelDownloadButton = target.closest<HTMLButtonElement>("[data-model-download]");
    if (modelDownloadButton) {
      if (modelDownloadButton.disabled) {
        return;
      }

      const kind = modelDownloadButton.dataset.modelDownload;
      if (kind === "tts" || kind === "asr") {
        downloadModel(kind);
      }
      return;
    }

    const visibilityButton = target.closest<HTMLButtonElement>("[data-browser-visibility-mode]");
    if (visibilityButton) {
      if (isBrowserVisibilityUpdating() || visibilityButton.disabled) {
        return;
      }

      const mode = visibilityButton.dataset.browserVisibilityMode;
      if (mode === "Visible" || mode === "Headless") {
        setBrowserVisibility(mode);
      }
      return;
    }

    const urlButtonMappings: Array<[selector: string, action: UrlAction]> = [
      ["[data-url-open-button]", "open"],
      ["[data-url-read-button]", "read"],
      ["[data-url-stop-button]", "stop"],
      ["[data-url-previous-button]", "previous"],
      ["[data-url-next-button]", "next"],
    ];
    for (const [selector, action] of urlButtonMappings) {
      const button = target.closest<HTMLButtonElement>(selector);
      if (!button) {
        continue;
      }

      if (isUrlInputActionBusy() || button.disabled) {
        return;
      }

      runUrlAction(action);
      return;
    }

    const actionButton = target.closest<HTMLButtonElement>("[data-confirmation-action]");
    if (!actionButton) {
      return;
    }

    const action = actionButton.dataset.confirmationAction;
    const confirmationId = actionButton.dataset.confirmationId;
    if (!confirmationId || (action !== "approve" && action !== "reject")) {
      return;
    }

    submitConfirmationAction(action, confirmationId);
  });

  appRoot.addEventListener("focusin", (event) => {
    const target = event.target;
    if (!(target instanceof HTMLInputElement)) {
      return;
    }

    const maskedDisplayValue = target.dataset.maskedApiKeyDisplay;
    if (!target.dataset.remoteApiKeyInput || !maskedDisplayValue || target.value !== maskedDisplayValue) {
      return;
    }

    target.value = "";
    target.type = "password";
  });

  appRoot.addEventListener("focusout", (event) => {
    const target = event.target;
    if (!(target instanceof HTMLInputElement)) {
      return;
    }

    const maskedDisplayValue = target.dataset.maskedApiKeyDisplay;
    if (!target.dataset.remoteApiKeyInput || !maskedDisplayValue || target.value.length > 0) {
      return;
    }

    target.value = maskedDisplayValue;
    target.type = "text";
  });

  appRoot.addEventListener("input", (event) => {
    const target = event.target;
    if (!(target instanceof HTMLInputElement)) {
      return;
    }

    if (target.dataset.audioControl === "volume") {
      updateAudioInput("volume", Number.parseFloat(target.value));
      return;
    }

    if (target.dataset.audioControl === "speed") {
      updateAudioInput("speed", Number.parseFloat(target.value));
      return;
    }

    if (target.dataset.confirmationThresholdControl === "true") {
      updateConfirmationThresholdInput(Number.parseFloat(target.value));
      return;
    }

    if (target.dataset.ocrThresholdControl === "char") {
      updateOcrThresholdInput("char", Number.parseInt(target.value, 10));
      return;
    }

    if (target.dataset.ocrThresholdControl === "region") {
      updateOcrThresholdInput("region", Number.parseInt(target.value, 10));
      return;
    }

    if (target.dataset.remoteApiKeyInput === "planner") {
      updateRemoteApiKeyInput("planner", target.value);
      return;
    }

    if (target.dataset.remoteApiKeyInput === "tts") {
      updateRemoteApiKeyInput("tts", target.value);
      return;
    }

    if (target.dataset.remoteApiKeyInput === "asr") {
      updateRemoteApiKeyInput("asr", target.value);
      return;
    }

    if (target.dataset.remotePlannerEndpointInput === "true") {
      updateRemotePlannerEndpointInput(target.value);
      return;
    }

    if (target.dataset.modelManagementInput === "models-dir") {
      updateModelManagementInput(target.value);
      return;
    }

    if (target.dataset.urlInput === "true") {
      updateUrlInput(target.value);
    }
  });

  appRoot.addEventListener("change", (event) => {
    const target = event.target;
    if (!(target instanceof HTMLInputElement) && !(target instanceof HTMLSelectElement)) {
      return;
    }

    if (target instanceof HTMLInputElement && target.dataset.audioControl === "volume") {
      if (isSettingsActionBusy("volume")) {
        return;
      }
      persistAudioChange("volume", Number.parseFloat(target.value));
      return;
    }

    if (target instanceof HTMLInputElement && target.dataset.audioControl === "speed") {
      if (isSettingsActionBusy("speed")) {
        return;
      }
      persistAudioChange("speed", Number.parseFloat(target.value));
      return;
    }

    if (target instanceof HTMLInputElement && target.dataset.confirmationThresholdControl === "true") {
      if (isSettingsActionBusy("confirmation-threshold")) {
        return;
      }
      persistConfirmationThreshold(Number.parseFloat(target.value));
      return;
    }

    if (target instanceof HTMLInputElement && target.dataset.clickWithoutConfirmationToggle === "true") {
      if (isSettingsActionBusy("click-without-confirmation")) {
        return;
      }
      persistClickWithoutConfirmation(target.checked);
      return;
    }

    if (target instanceof HTMLInputElement && target.dataset.ocrThresholdControl === "char") {
      if (isSettingsActionBusy("ocr-char-threshold")) {
        return;
      }
      persistOcrThresholdChange("char", Number.parseInt(target.value, 10));
      return;
    }

    if (target instanceof HTMLInputElement && target.dataset.ocrThresholdControl === "region") {
      if (isSettingsActionBusy("ocr-region-threshold")) {
        return;
      }
      persistOcrThresholdChange("region", Number.parseInt(target.value, 10));
      return;
    }

    if (target instanceof HTMLSelectElement && target.dataset.remotePlannerModelSelect === "true") {
      updateRemotePlannerModelSelection(target.value);
      return;
    }

    if (target instanceof HTMLInputElement && target.dataset.modelManagementToggle === "check-on-startup") {
      if (isSettingsActionBusy("model-check-on-startup")) {
        return;
      }
      persistModelManagementToggle("check-on-startup", target.checked);
      return;
    }

    if (target instanceof HTMLInputElement && target.dataset.modelManagementToggle === "auto-download-missing") {
      if (isSettingsActionBusy("model-auto-download-missing")) {
        return;
      }
      persistModelManagementToggle("auto-download-missing", target.checked);
      return;
    }

    if (target instanceof HTMLInputElement && target.dataset.modelManagementInput === "models-dir") {
      if (isSettingsActionBusy("models-dir")) {
        return;
      }
      persistModelsDir();
      return;
    }

    if (target instanceof HTMLSelectElement && target.dataset.asrProviderSelect === "true") {
      if (isSettingsActionBusy("asr-provider")) {
        return;
      }
      if (target.value === "Local" || target.value === "Remote") {
        persistAsrProvider(target.value);
      }
      return;
    }

    if (target instanceof HTMLSelectElement && target.dataset.ttsProviderSelect === "true") {
      if (isSettingsActionBusy("tts-provider")) {
        return;
      }
      if (target.value === "Local" || target.value === "Remote") {
        persistTtsProvider(target.value);
      }
      return;
    }

    if (target instanceof HTMLSelectElement && target.dataset.ttsModelSelect === "true") {
      if (isSettingsActionBusy("tts-model")) {
        return;
      }
      persistTtsModel(target.value);
      return;
    }

    if (target instanceof HTMLSelectElement && target.dataset.ttsVoiceSelect === "true") {
      if (isSettingsActionBusy("tts-voice")) {
        return;
      }
      persistTtsVoice(target.value);
    }
  });

  appRoot.addEventListener("pointerdown", (event) => {
    const target = event.target;
    if (!(target instanceof HTMLElement)) {
      return;
    }

    const button = target.closest<HTMLButtonElement>("[data-push-to-talk-button]");
    if (!button || button.disabled || event.button !== 0) {
      return;
    }

    event.preventDefault();
    beginPushToTalk("pointer");
  });

  window.addEventListener("pointerup", () => {
    releasePushToTalk("pointer");
  });

  window.addEventListener("pointercancel", () => {
    cancelPushToTalk();
  });

  window.addEventListener("blur", () => {
    cancelPushToTalk();
  });

  window.addEventListener("keydown", (event) => {
    if (!isPushToTalkKeyEvent(event)) {
      return;
    }

    event.preventDefault();
    beginPushToTalk("keyboard");
  });

  window.addEventListener("keyup", (event) => {
    if (!isPushToTalkKeyEvent(event)) {
      return;
    }

    event.preventDefault();
    releasePushToTalk("keyboard");
  });
}
