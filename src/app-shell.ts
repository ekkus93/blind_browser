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

export type PanelRootMap = Record<PanelRootKey, HTMLDivElement>;

function renderPanelRootPlaceholder(rootKey: PanelRootKey): string {
  return `<div data-panel-root="${rootKey}"></div>`;
}

export function renderAppShell(): string {
  return `
    <main class="shell">
      <section class="hero">
        <p class="eyebrow">Voice-first desktop runtime</p>
        <h1>Voice-first browser workspace</h1>
        <p class="lede">
          Thin Tauri frontend for the live blind_browser runtime. Browser control,
          narration, voice input, settings, and confirmation flows stay lightweight
          here while deterministic tools and runtime state live in Rust.
        </p>
      </section>

      <section class="panels" aria-label="Application sections">
        <article class="panel">
          <h2>Current runtime</h2>
          <p>Deterministic browser, audio, extraction, and planner state still live in Rust.</p>
        </article>
        <article class="panel">
          <h2>Remaining gaps</h2>
          <p>Provider failover and some internal cleanup work remain, but the core voice-first runtime is already wired.</p>
        </article>
        <article class="panel">
          <h2>UI stance</h2>
          <p>The UI remains intentionally light so voice-first behavior stays the primary product path.</p>
        </article>
        <article class="panel">
          <h2>Confirmation flow</h2>
          <p>
            Frontend orchestration now opens a dedicated confirmation UI state whenever a planner
            execution returns <strong>AwaitingConfirmation</strong>.
          </p>
        </article>
      </section>

      ${renderPanelRootPlaceholder("push-to-talk")}
      ${renderPanelRootPlaceholder("url-input")}
      ${renderPanelRootPlaceholder("status")}
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
      ${renderPanelRootPlaceholder("confirmation-panel")}
    </main>
  `;
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
