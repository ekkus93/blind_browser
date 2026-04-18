import assert from "node:assert/strict";
import test from "node:test";

class FakeEventTarget {
  #listeners = new Map();

  addEventListener(type, listener) {
    const listeners = this.#listeners.get(type) ?? [];
    listeners.push(listener);
    this.#listeners.set(type, listeners);
  }

  dispatch(type, event) {
    const listeners = this.#listeners.get(type) ?? [];
    for (const listener of listeners) {
      listener(event);
    }
  }
}

class FakeElement extends FakeEventTarget {
  constructor() {
    super();
    this.dataset = {};
    this.disabled = false;
    this.value = "";
    this.checked = false;
    this.id = "";
    this.ownerDocument = null;
    this.selectorMatches = new Set();
    this.children = [];
    this.selectionStart = null;
    this.selectionEnd = null;
    this.selectionDirection = null;
    this.focusOptions = null;
    this.scrollOptions = null;
  }

  closest(selector) {
    return this.selectorMatches.has(selector) ? this : null;
  }

  focus(options) {
    this.focusOptions = options ?? null;
    if (this.ownerDocument) {
      this.ownerDocument.activeElement = this;
    }
  }

  scrollIntoView(options) {
    this.scrollOptions = options ?? null;
  }

  contains(element) {
    return this === element || this.children.includes(element);
  }

  setSelectionRange(start, end, direction) {
    this.selectionStart = start;
    this.selectionEnd = end;
    this.selectionDirection = direction ?? null;
  }
}

class FakeInputElement extends FakeElement {}
class FakeTextareaElement extends FakeElement {}
class FakeSelectElement extends FakeElement {}
class FakeButtonElement extends FakeElement {}
class FakeDivElement extends FakeElement {}

class FakeDocument {
  constructor() {
    this.activeElement = null;
    this.elements = new Map();
  }

  register(element) {
    element.ownerDocument = this;
    if (element.id) {
      this.elements.set(element.id, element);
    }
    return element;
  }

  getElementById(id) {
    return this.elements.get(id) ?? null;
  }
}

class FakeWindow extends FakeEventTarget {}

globalThis.HTMLElement = FakeElement;
globalThis.HTMLInputElement = FakeInputElement;
globalThis.HTMLTextAreaElement = FakeTextareaElement;
globalThis.HTMLSelectElement = FakeSelectElement;
globalThis.HTMLButtonElement = FakeButtonElement;
globalThis.HTMLDivElement = FakeDivElement;

const { renderPanelRoot } = await import("./app-shell.ts");
const { registerAppEventHandlers } = await import("./event-handlers.ts");

function createEventHandlerDeps() {
  const appRoot = new FakeDivElement();
  const document = new FakeDocument();
  const window = new FakeWindow();
  const calls = {
    runUrlAction: [],
    persistAudioChange: [],
    persistAsrProvider: [],
    setAppView: [],
  };

  registerAppEventHandlers({
    appRoot,
    document,
    window,
    isUrlInputActionBusy: () => false,
    isBrowserVisibilityUpdating: () => false,
    isSettingsActionBusy: (key) => key === "volume",
    isPushToTalkKeyEvent: () => false,
    saveRemoteApiKey: () => {},
    downloadModel: () => {},
    setBrowserVisibility: () => {},
    runUrlAction: (action) => {
      calls.runUrlAction.push(action);
    },
    submitConfirmationAction: () => {},
    updateAudioInput: () => {},
    updateConfirmationThresholdInput: () => {},
    updateOcrThresholdInput: () => {},
    updateRemoteApiKeyInput: () => {},
    updateModelManagementInput: () => {},
    updateUrlInput: () => {},
    persistAudioChange: (kind, value) => {
      calls.persistAudioChange.push([kind, value]);
    },
    persistConfirmationThreshold: () => {},
    persistClickWithoutConfirmation: () => {},
    persistOcrThresholdChange: () => {},
    persistModelManagementToggle: () => {},
    persistModelsDir: () => {},
    persistAsrProvider: (mode) => {
      calls.persistAsrProvider.push(mode);
    },
    persistTtsProvider: () => {},
    persistTtsModel: () => {},
    persistTtsVoice: () => {},
    setAppView: (view) => {
      calls.setAppView.push(view);
    },
    beginPushToTalk: () => {},
    releasePushToTalk: () => {},
    cancelPushToTalk: () => {},
  });

  return { appRoot, calls, document };
}

test("renderPanelRoot preserves focus and selection for the active control", () => {
  const document = new FakeDocument();
  globalThis.document = document;

  const root = new FakeDivElement();
  const activeInput = new FakeInputElement();
  activeInput.id = "url-input";
  document.register(activeInput);
  activeInput.value = "https://example.com";
  activeInput.selectionStart = 8;
  activeInput.selectionEnd = 15;
  activeInput.selectionDirection = "forward";
  root.children = [activeInput];
  document.activeElement = activeInput;

  const replacementInput = new FakeInputElement();
  replacementInput.id = "url-input";
  document.register(replacementInput);
  replacementInput.value = "https://example.com";

  Object.defineProperty(root, "innerHTML", {
    set() {
      root.children = [replacementInput];
    },
  });

  renderPanelRoot({ "url-input": root }, "url-input", "<input id=\"url-input\" />");

  assert.equal(document.activeElement, replacementInput);
  assert.deepEqual(replacementInput.focusOptions, { preventScroll: true });
  assert.equal(replacementInput.selectionStart, 8);
  assert.equal(replacementInput.selectionEnd, 15);
  assert.equal(replacementInput.selectionDirection, "forward");
});

test("event delegation keeps handling newly replaced URL buttons", () => {
  const { appRoot, calls } = createEventHandlerDeps();
  const firstButton = new FakeButtonElement();
  firstButton.selectorMatches.add("[data-url-open-button]");
  const secondButton = new FakeButtonElement();
  secondButton.selectorMatches.add("[data-url-open-button]");

  appRoot.dispatch("click", { target: firstButton });
  appRoot.dispatch("click", { target: secondButton });

  assert.deepEqual(calls.runUrlAction, ["open", "open"]);
});

test("settings target click scrolls and focuses the matching control", () => {
  const { appRoot, document } = createEventHandlerDeps();
  const settingsButton = new FakeButtonElement();
  settingsButton.selectorMatches.add("[data-settings-target]");
  settingsButton.dataset.settingsTarget = "settings-tts-provider-control";

  const targetControl = new FakeInputElement();
  targetControl.id = "settings-tts-provider-control";
  document.register(targetControl);

  appRoot.dispatch("click", { target: settingsButton });

  assert.deepEqual(targetControl.scrollOptions, { behavior: "smooth", block: "center" });
  assert.deepEqual(targetControl.focusOptions, { preventScroll: true });
});

test("settings target click switches to the settings view", () => {
  const { appRoot, calls, document } = createEventHandlerDeps();
  const settingsButton = new FakeButtonElement();
  settingsButton.selectorMatches.add("[data-settings-target]");
  settingsButton.dataset.settingsTarget = "settings-tts-provider-control";

  const targetControl = new FakeInputElement();
  targetControl.id = "settings-tts-provider-control";
  document.register(targetControl);

  appRoot.dispatch("click", { target: settingsButton });

  assert.deepEqual(calls.setAppView, ["settings"]);
});

test("view navigation buttons switch between workspace and settings", () => {
  const { appRoot, calls } = createEventHandlerDeps();
  const settingsButton = new FakeButtonElement();
  settingsButton.selectorMatches.add("[data-app-view-button]");
  settingsButton.dataset.appViewButton = "settings";

  const workspaceButton = new FakeButtonElement();
  workspaceButton.selectorMatches.add("[data-app-view-button]");
  workspaceButton.dataset.appViewButton = "workspace";

  appRoot.dispatch("click", { target: settingsButton });
  appRoot.dispatch("click", { target: workspaceButton });

  assert.deepEqual(calls.setAppView, ["settings", "workspace"]);
});

test("busy change guards block only the matching settings control", () => {
  const { appRoot, calls } = createEventHandlerDeps();
  const volumeInput = new FakeInputElement();
  volumeInput.dataset.audioControl = "volume";
  volumeInput.value = "0.35";

  const asrSelect = new FakeSelectElement();
  asrSelect.dataset.asrProviderSelect = "true";
  asrSelect.value = "Remote";

  appRoot.dispatch("change", { target: volumeInput });
  appRoot.dispatch("change", { target: asrSelect });

  assert.deepEqual(calls.persistAudioChange, []);
  assert.deepEqual(calls.persistAsrProvider, ["Remote"]);
});
