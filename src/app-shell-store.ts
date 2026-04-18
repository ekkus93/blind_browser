import { configureStore, createSlice, type PayloadAction } from "@reduxjs/toolkit";

import type { AppView, SettingsView } from "./app-shell";

interface AppShellViewState {
  appView: AppView;
  settingsView: SettingsView;
}

const initialViewState: AppShellViewState = {
  appView: "workspace",
  settingsView: "overview",
};

const appShellViewSlice = createSlice({
  name: "appShellView",
  initialState: initialViewState,
  reducers: {
    setAppView(state, action: PayloadAction<AppView>) {
      state.appView = action.payload;
      if (action.payload === "settings") {
        state.settingsView = "overview";
      }
    },
    setSettingsView(state, action: PayloadAction<SettingsView>) {
      state.settingsView = action.payload;
    },
  },
});

export const {
  setAppView,
  setSettingsView,
} = appShellViewSlice.actions;

export function createAppShellStore(preloadedState?: Partial<AppShellViewState>) {
  return configureStore({
    reducer: appShellViewSlice.reducer,
    preloadedState: {
      ...initialViewState,
      ...preloadedState,
    },
  });
}

export type AppShellStore = ReturnType<typeof createAppShellStore>;