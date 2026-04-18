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

globalThis.HTMLElement = FakeElement;
globalThis.HTMLInputElement = FakeInputElement;
globalThis.HTMLTextAreaElement = FakeTextareaElement;
globalThis.HTMLSelectElement = FakeSelectElement;
globalThis.HTMLButtonElement = FakeButtonElement;
globalThis.HTMLDivElement = FakeDivElement;

const { AppShellMarkup, preserveActivePanelControl } = await import("./app-shell.ts");
const {
  renderSecretEntryCard,
  OPENAI_API_KEYS_URL,
} = await import("./confirmation-panel-helpers.ts");
const {
  renderSettingsGuidancePanelNode,
  renderUrlInputPanelNode,
} = await import("./confirmation-panel.ts");
const { registerApiKeyMaskEventHandlers } = await import("./event-handlers.ts");
const {
  createAppShellStore,
  setAppView,
  setSettingsView,
  setUrlInputPanelState,
} = await import("./app-shell-store.ts");

function isReactElement(value) {
  return typeof value === "object" && value !== null && "type" in value && "props" in value;
}

function visitReactTree(node, visitor) {
  if (Array.isArray(node)) {
    for (const child of node) {
      visitReactTree(child, visitor);
    }
    return;
  }

  if (!isReactElement(node)) {
    return;
  }

  visitor(node);
  visitReactTree(node.props?.children, visitor);
}

function findElements(node, predicate) {
  const elements = [];
  visitReactTree(node, (element) => {
    if (predicate(element)) {
      elements.push(element);
    }
  });
  return elements;
}

function findElement(node, predicate) {
  const matches = findElements(node, predicate);
  assert.equal(matches.length, 1);
  return matches[0];
}

test("preserveActivePanelControl keeps focus and selection on the replacement control", () => {
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

  preserveActivePanelControl(root, () => {
    root.children = [replacementInput];
  });

  assert.equal(document.activeElement, replacementInput);
  assert.deepEqual(replacementInput.focusOptions, { preventScroll: true });
  assert.equal(replacementInput.selectionStart, 8);
  assert.equal(replacementInput.selectionEnd, 15);
  assert.equal(replacementInput.selectionDirection, "forward");
});

test("masked remote API key display clears on focus and restores on blur", () => {
  const appRoot = new FakeDivElement();
  registerApiKeyMaskEventHandlers({ appRoot });

  const card = renderSecretEntryCard(
    "planner",
    "openai",
    "",
    "***7890",
    false,
    false,
    true,
    null,
  );
  const inputElement = findElement(card, (element) => element.props?.["data-remote-api-key-input"] === "planner");
  const input = new FakeInputElement();
  input.dataset.remoteApiKeyInput = inputElement.props["data-remote-api-key-input"];
  input.dataset.maskedApiKeyDisplay = inputElement.props["data-masked-api-key-display"];
  input.value = inputElement.props.value;
  input.type = inputElement.props.type;

  appRoot.dispatch("focusin", { target: input });

  assert.equal(input.value, "");
  assert.equal(input.type, "password");

  input.value = "";
  appRoot.dispatch("focusout", { target: input });

  assert.equal(input.value, "***7890");
  assert.equal(input.type, "text");
});

test("view navigation buttons switch between workspace and settings", () => {
  const calls = [];
  const tree = AppShellMarkup({
    initialAppView: "workspace",
    initialSettingsView: "overview",
    panelContent: {},
    navigationHandlers: {
      onAppViewSelect: (view) => {
        calls.push(view);
      },
    },
  });

  findElement(tree, (element) => element.props?.["data-app-view-button"] === "settings").props.onClick();
  findElement(tree, (element) => element.props?.["data-app-view-button"] === "workspace").props.onClick();

  assert.deepEqual(calls, ["settings", "workspace"]);
});

test("settings subpage buttons switch between planner, tts, asr, runtime, and overview", () => {
  const calls = [];
  const tree = AppShellMarkup({
    initialAppView: "settings",
    initialSettingsView: "planner",
    panelContent: {},
    navigationHandlers: {
      onSettingsViewSelect: (view) => {
        calls.push(view);
      },
    },
  });

  findElement(tree, (element) => element.props?.["data-settings-view-button"] === "planner").props.onClick();
  findElement(tree, (element) => element.props?.["data-settings-view-button"] === "tts").props.onClick();
  findElement(tree, (element) => element.props?.["data-settings-view-button"] === "asr").props.onClick();
  findElement(tree, (element) => element.props?.["data-settings-view-button"] === "runtime").props.onClick();
  findElement(tree, (element) => element.props?.["data-settings-view-button"] === "overview").props.onClick();

  assert.deepEqual(calls, ["planner", "tts", "asr", "runtime", "overview"]);
});

test("URL panel handlers fire from the React-rendered controls", () => {
  const calls = [];
  const panel = renderUrlInputPanelNode(
    {
      currentUrl: "https://example.com",
      draftValue: "https://example.com/docs",
      hasUnsubmittedChanges: true,
      isOpening: false,
      isReading: false,
      isStopping: false,
      isAdvancing: false,
      isRewinding: false,
      error: null,
    },
    {
      onDraftInput: (value) => {
        calls.push(["draft", value]);
      },
      onOpen: () => {
        calls.push(["open"]);
      },
      onRead: () => {
        calls.push(["read"]);
      },
      onStop: () => {
        calls.push(["stop"]);
      },
      onPrevious: () => {
        calls.push(["previous"]);
      },
      onNext: () => {
        calls.push(["next"]);
      },
    },
  );

  findElement(panel, (element) => element.props?.["data-url-input"] === "true").props.onChange({
    currentTarget: { value: "https://example.com/next" },
  });
  findElement(panel, (element) => element.props?.["data-url-open-button"] === "true").props.onClick();
  findElement(panel, (element) => element.props?.["data-url-read-button"] === "true").props.onClick();
  findElement(panel, (element) => element.props?.["data-url-stop-button"] === "true").props.onClick();
  findElement(panel, (element) => element.props?.["data-url-previous-button"] === "true").props.onClick();
  findElement(panel, (element) => element.props?.["data-url-next-button"] === "true").props.onClick();

  assert.deepEqual(calls, [
    ["draft", "https://example.com/next"],
    ["open"],
    ["read"],
    ["stop"],
    ["previous"],
    ["next"],
  ]);
});

test("settings guidance buttons and known links use React-owned callbacks", () => {
  const calls = [];
  const panel = renderSettingsGuidancePanelNode(
    {
      title: "Remote planner API key required",
      message: `Add a key at ${OPENAI_API_KEYS_URL} before testing the planner profile.`,
      actions: [{ targetId: "settings-remote-planner-title", label: "Open planner settings" }],
    },
    {
      onSelectTarget: (targetId) => {
        calls.push(["target", targetId]);
      },
      onOpenExternalLink: (url) => {
        calls.push(["link", url]);
      },
    },
  );

  findElement(panel, (element) => element.props?.["data-settings-target"] === "settings-remote-planner-title").props.onClick();

  let prevented = false;
  findElement(panel, (element) => element.props?.["data-external-link-url"] === OPENAI_API_KEYS_URL).props.onClick({
    preventDefault() {
      prevented = true;
    },
  });

  assert.equal(prevented, true);
  assert.deepEqual(calls, [
    ["target", "settings-remote-planner-title"],
    ["link", OPENAI_API_KEYS_URL],
  ]);
});

test("Redux view and panel state updates stay explicit through store actions", () => {
  const store = createAppShellStore();

  store.dispatch(setAppView("settings"));
  store.dispatch(setSettingsView("tts"));
  store.dispatch(setUrlInputPanelState({
    draftValue: "https://example.com/runtime",
    hasUnsubmittedChanges: true,
  }));

  const state = store.getState();
  assert.equal(state.shellView.appView, "settings");
  assert.equal(state.shellView.settingsView, "tts");
  assert.equal(state.panelStates.urlInputPanelState.draftValue, "https://example.com/runtime");
  assert.equal(state.panelStates.urlInputPanelState.hasUnsubmittedChanges, true);
});
