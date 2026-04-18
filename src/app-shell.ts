export type PanelRootKey =
  | "push-to-talk"
  | "url-input"
  | "status"
  | "audio-controls"
  | "settings-guidance"
  | "settings-planner-provider"
  | "settings-remote-planner"
  | "settings-provider-failover"
  | "settings-confirmation"
  | "settings-ocr-threshold"
  | "settings-asr-provider"
  | "settings-local-asr-model"
  | "settings-model-management"
  | "settings-remote-asr"
  | "settings-tts-provider"
  | "settings-tts-model"
  | "settings-local-tts-model"
  | "settings-remote-tts"
  | "settings-tts-voice"
  | "settings-volume"
  | "settings-speed"
  | "confirmation-panel";

export type AppView = "workspace" | "settings";

export type PanelRootMap = Record<PanelRootKey, HTMLDivElement>;

function renderPanelRootPlaceholder(rootKey: PanelRootKey): string {
  return `<div data-panel-root="${rootKey}"></div>`;
}

export function renderAppShell(): string {
  return `
    <main class="shell">
      <header class="shell-toolbar">
        <nav class="shell-nav" aria-label="App pages">
          <button
            type="button"
            class="shell-nav-button shell-nav-button-active"
            data-app-view-button="workspace"
            aria-pressed="true"
          >
            Workspace
          </button>
          <button
            type="button"
            class="shell-nav-button"
            data-app-view-button="settings"
            aria-pressed="false"
          >
            Settings
          </button>
        </nav>
      </header>

      <section class="app-view app-view-active" data-app-view-section="workspace">
        <section class="hero">
          <p class="eyebrow">Voice-first browser</p>
          <h1>Workspace</h1>
          <p class="lede">
            Use voice input, open pages, control reading, and review the current runtime state here.
            Settings live on a separate page so the main workflow stays simpler.
          </p>
        </section>

        <section class="panels" aria-label="Workspace sections">
          <article class="panel">
            <h2>Voice input</h2>
            <p>Start commands here, then keep the browser flow focused on listening, reading, and confirmation.</p>
          </article>
          <article class="panel">
            <h2>Page control</h2>
            <p>Open a page, start reading, move forward or backward, and stop without digging through settings.</p>
          </article>
          <article class="panel">
            <h2>Runtime status</h2>
            <p>See what the live browser, narration, and listening state are doing right now.</p>
          </article>
        </section>

        ${renderPanelRootPlaceholder("push-to-talk")}
        ${renderPanelRootPlaceholder("url-input")}
        ${renderPanelRootPlaceholder("status")}
        ${renderPanelRootPlaceholder("confirmation-panel")}
      </section>

      <section class="app-view" data-app-view-section="settings" hidden aria-hidden="true">
        ${renderPanelRootPlaceholder("audio-controls")}
        ${renderPanelRootPlaceholder("settings-guidance")}
        ${renderPanelRootPlaceholder("settings-planner-provider")}
        ${renderPanelRootPlaceholder("settings-remote-planner")}
        ${renderPanelRootPlaceholder("settings-provider-failover")}
        ${renderPanelRootPlaceholder("settings-confirmation")}
        ${renderPanelRootPlaceholder("settings-ocr-threshold")}
        ${renderPanelRootPlaceholder("settings-asr-provider")}
        ${renderPanelRootPlaceholder("settings-local-asr-model")}
        ${renderPanelRootPlaceholder("settings-model-management")}
        ${renderPanelRootPlaceholder("settings-remote-asr")}
        ${renderPanelRootPlaceholder("settings-tts-provider")}
        ${renderPanelRootPlaceholder("settings-tts-model")}
        ${renderPanelRootPlaceholder("settings-local-tts-model")}
        ${renderPanelRootPlaceholder("settings-remote-tts")}
        ${renderPanelRootPlaceholder("settings-tts-voice")}
        ${renderPanelRootPlaceholder("settings-volume")}
        ${renderPanelRootPlaceholder("settings-speed")}
      </section>
    </main>
  `;
}

export function setActiveAppView(appRoot: HTMLDivElement, nextView: AppView) {
  const sections = appRoot.querySelectorAll<HTMLElement>("[data-app-view-section]");
  sections.forEach((section) => {
    const isActive = section.dataset.appViewSection === nextView;
    section.hidden = !isActive;
    section.setAttribute("aria-hidden", String(!isActive));
    section.classList.toggle("app-view-active", isActive);
  });

  const buttons = appRoot.querySelectorAll<HTMLButtonElement>("[data-app-view-button]");
  buttons.forEach((button) => {
    const isActive = button.dataset.appViewButton === nextView;
    button.setAttribute("aria-pressed", String(isActive));
    button.classList.toggle("shell-nav-button-active", isActive);
  });
}

function requirePanelRoot(appRoot: HTMLDivElement, rootKey: PanelRootKey): HTMLDivElement {
  const root = appRoot.querySelector<HTMLDivElement>(`[data-panel-root="${rootKey}"]`);
  if (!root) {
    throw new Error(`Panel root ${rootKey} was not found.`);
  }

  return root;
}

export function createPanelRoots(appRoot: HTMLDivElement): PanelRootMap {
  appRoot.innerHTML = renderAppShell();
  return {
    "push-to-talk": requirePanelRoot(appRoot, "push-to-talk"),
    "url-input": requirePanelRoot(appRoot, "url-input"),
    status: requirePanelRoot(appRoot, "status"),
    "audio-controls": requirePanelRoot(appRoot, "audio-controls"),
    "settings-guidance": requirePanelRoot(appRoot, "settings-guidance"),
    "settings-planner-provider": requirePanelRoot(appRoot, "settings-planner-provider"),
    "settings-remote-planner": requirePanelRoot(appRoot, "settings-remote-planner"),
    "settings-provider-failover": requirePanelRoot(appRoot, "settings-provider-failover"),
    "settings-confirmation": requirePanelRoot(appRoot, "settings-confirmation"),
    "settings-ocr-threshold": requirePanelRoot(appRoot, "settings-ocr-threshold"),
    "settings-asr-provider": requirePanelRoot(appRoot, "settings-asr-provider"),
    "settings-local-asr-model": requirePanelRoot(appRoot, "settings-local-asr-model"),
    "settings-model-management": requirePanelRoot(appRoot, "settings-model-management"),
    "settings-remote-asr": requirePanelRoot(appRoot, "settings-remote-asr"),
    "settings-tts-provider": requirePanelRoot(appRoot, "settings-tts-provider"),
    "settings-tts-model": requirePanelRoot(appRoot, "settings-tts-model"),
    "settings-local-tts-model": requirePanelRoot(appRoot, "settings-local-tts-model"),
    "settings-remote-tts": requirePanelRoot(appRoot, "settings-remote-tts"),
    "settings-tts-voice": requirePanelRoot(appRoot, "settings-tts-voice"),
    "settings-volume": requirePanelRoot(appRoot, "settings-volume"),
    "settings-speed": requirePanelRoot(appRoot, "settings-speed"),
    "confirmation-panel": requirePanelRoot(appRoot, "confirmation-panel"),
  };
}

interface PreservedPanelControlState {
  elementId: string;
  value: string;
  selectionStart: number | null;
  selectionEnd: number | null;
  selectionDirection: "forward" | "backward" | "none" | null;
}

function captureActivePanelControl(root: HTMLDivElement): PreservedPanelControlState | null {
  const activeElement = document.activeElement;
  if (
    !activeElement
    || !root.contains(activeElement)
    || (
      !(activeElement instanceof HTMLInputElement)
      && !(activeElement instanceof HTMLTextAreaElement)
      && !(activeElement instanceof HTMLSelectElement)
    )
    || !activeElement.id
  ) {
    return null;
  }

  return {
    elementId: activeElement.id,
    value: activeElement.value,
    selectionStart:
      activeElement instanceof HTMLSelectElement ? null : activeElement.selectionStart,
    selectionEnd:
      activeElement instanceof HTMLSelectElement ? null : activeElement.selectionEnd,
    selectionDirection:
      activeElement instanceof HTMLSelectElement ? null : activeElement.selectionDirection,
  };
}

function restoreActivePanelControl(
  root: HTMLDivElement,
  controlState: PreservedPanelControlState | null,
) {
  if (!controlState) {
    return;
  }

  const nextElement = document.getElementById(controlState.elementId);
  if (
    !nextElement
    || !root.contains(nextElement)
    || (
      !(nextElement instanceof HTMLInputElement)
      && !(nextElement instanceof HTMLTextAreaElement)
      && !(nextElement instanceof HTMLSelectElement)
    )
  ) {
    return;
  }

  nextElement.focus({ preventScroll: true });
  if (
    nextElement instanceof HTMLInputElement
    || nextElement instanceof HTMLTextAreaElement
  ) {
    if (nextElement.value === controlState.value) {
      nextElement.setSelectionRange(
        controlState.selectionStart,
        controlState.selectionEnd,
        controlState.selectionDirection ?? undefined,
      );
    }
  }
}

export function renderPanelRoot(
  panelRoots: PanelRootMap,
  rootKey: PanelRootKey,
  html: string,
) {
  const root = panelRoots[rootKey];
  const controlState = captureActivePanelControl(root);
  root.innerHTML = html;
  restoreActivePanelControl(root, controlState);
}
