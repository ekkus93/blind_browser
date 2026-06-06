import {
  registerApiKeyMaskEventHandlers,
  registerGlobalPushToTalkHandlers,
} from "./event-handlers";
import {
  beginPushToTalk,
  cancelPushToTalk,
  releasePushToTalk,
} from "./voice-loop";

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }

  return (
    target.isContentEditable ||
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement
  );
}

export function isPushToTalkKeyEvent(event: KeyboardEvent): boolean {
  return (
    event.code === "Space" &&
    !event.repeat &&
    !event.altKey &&
    !event.ctrlKey &&
    !event.metaKey &&
    !isEditableTarget(event.target)
  );
}

export function registerShellEventHandlers(appRoot: HTMLDivElement) {
  registerApiKeyMaskEventHandlers({ appRoot });
  registerGlobalPushToTalkHandlers({
    window,
    isPushToTalkKeyEvent,
    beginPushToTalk: (source) => {
      void beginPushToTalk(source);
    },
    releasePushToTalk: (source) => {
      void releasePushToTalk(source);
    },
    cancelPushToTalk: () => {
      void cancelPushToTalk();
    },
  });
}
