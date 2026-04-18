type PushToTalkSource = "keyboard" | "pointer";

interface ApiKeyMaskHandlerDependencies {
  appRoot: HTMLDivElement;
}

interface GlobalPushToTalkHandlerDependencies {
  window: Window;
  isPushToTalkKeyEvent: (event: KeyboardEvent) => boolean;
  beginPushToTalk: (source: PushToTalkSource) => void;
  releasePushToTalk: (source: PushToTalkSource) => void;
  cancelPushToTalk: () => void;
}

export function registerApiKeyMaskEventHandlers({ appRoot }: ApiKeyMaskHandlerDependencies) {
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
}

export function registerGlobalPushToTalkHandlers({
  window,
  isPushToTalkKeyEvent,
  beginPushToTalk,
  releasePushToTalk,
  cancelPushToTalk,
}: GlobalPushToTalkHandlerDependencies) {
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
