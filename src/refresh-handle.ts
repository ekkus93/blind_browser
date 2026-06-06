let _refresh: (() => Promise<void>) | undefined;

export function setRuntimeRefreshHandle(fn: () => Promise<void>): void {
  _refresh = fn;
}

export function refreshRuntimePanels(): Promise<void> {
  return _refresh ? _refresh() : Promise.resolve();
}
