import {
  type AppShellNavigationHandlers,
  type AppShellPanelContent,
  type AppView,
  type SettingsStatuses,
  type SettingsView,
  renderAppViewActionButton,
  renderPanelContent,
  renderSettingsSubpageBackButton,
  renderSettingsSubpageLink,
} from "./app-shell-nav.tsx";

export type {
  AppShellNavigationHandlers,
  AppShellPanelContent,
  AppView,
  PanelRootKey,
  SettingsCardStatus,
  SettingsStatuses,
  SettingsView,
} from "./app-shell-nav.tsx";

export { preserveActivePanelControl } from "./app-shell-controls.ts";

interface AppShellMarkupProps {
  initialAppView: AppView;
  initialSettingsView: SettingsView;
  panelContent?: AppShellPanelContent;
  navigationHandlers?: AppShellNavigationHandlers;
  settingsStatuses?: SettingsStatuses;
}

export function AppShellMarkup({
  initialAppView,
  initialSettingsView,
  panelContent,
  navigationHandlers,
  settingsStatuses,
}: AppShellMarkupProps) {
  const workspaceActive = initialAppView === "workspace";
  const settingsActive = initialAppView === "settings";
  const showBackButton = settingsActive && initialSettingsView !== "overview";
  const showAppViewAction = workspaceActive || (settingsActive && !showBackButton);

  return (
    <main className="shell">
      <header className="shell-toolbar">
        {showAppViewAction ? renderAppViewActionButton(initialAppView, navigationHandlers) : null}
        {renderSettingsSubpageBackButton(showBackButton, navigationHandlers)}
        {renderPanelContent("voice-status", panelContent)}
      </header>

      {renderPanelContent("app-alert", panelContent)}

      <section
        className={`app-view${workspaceActive ? " app-view-active" : ""}`}
        data-app-view-section="workspace"
        hidden={!workspaceActive}
        aria-hidden={!workspaceActive}
      >
        <div className="workspace-control-bar" data-workspace-control-bar="true">
          {renderPanelContent("push-to-talk", panelContent)}
          {renderPanelContent("url-input", panelContent)}
        </div>
        {renderPanelContent("status", panelContent)}
        {renderPanelContent("confirmation-panel", panelContent)}
      </section>

      <section
        className={`app-view${settingsActive ? " app-view-active" : ""}`}
        data-app-view-section="settings"
        hidden={!settingsActive}
        aria-hidden={!settingsActive}
      >
        <div
          className={`settings-view${initialSettingsView === "overview" ? " settings-view-active" : ""}`}
          data-settings-view-section="overview"
          hidden={initialSettingsView !== "overview"}
          aria-hidden={initialSettingsView !== "overview"}
        >
          <section className="hero hero-settings">
            <h1>Settings</h1>
          </section>
          {renderPanelContent("settings-guidance", panelContent)}
          <section className="settings-group" aria-labelledby="settings-group-playback-title">
            <div className="settings-group-copy">
              <p className="settings-group-eyebrow">Listening</p>
              <h2 id="settings-group-playback-title">Playback</h2>
            </div>
            {renderPanelContent("audio-controls", panelContent)}
          </section>
          <section className="settings-group settings-group-link" aria-labelledby="settings-group-planner-title">
            <div className="settings-group-copy">
              <p className="settings-group-eyebrow">Command interpretation</p>
              <h2 id="settings-group-planner-title">AI assistant</h2>
            </div>
            {renderSettingsSubpageLink("planner", "Open AI assistant setup", navigationHandlers, settingsStatuses?.planner)}
          </section>
          <section className="settings-group settings-group-link" aria-labelledby="settings-group-tts-title">
            <div className="settings-group-copy">
              <p className="settings-group-eyebrow">Speech output</p>
              <h2 id="settings-group-tts-title">Voice output</h2>
            </div>
            {renderSettingsSubpageLink("tts", "Open voice output setup", navigationHandlers, settingsStatuses?.tts)}
          </section>
          <section className="settings-group settings-group-link" aria-labelledby="settings-group-asr-title">
            <div className="settings-group-copy">
              <p className="settings-group-eyebrow">Speech input</p>
              <h2 id="settings-group-asr-title">Voice input</h2>
            </div>
            {renderSettingsSubpageLink("asr", "Open voice input setup", navigationHandlers, settingsStatuses?.asr)}
          </section>
          <section className="settings-group settings-group-link" aria-labelledby="settings-group-runtime-title">
            <div className="settings-group-copy">
              <p className="settings-group-eyebrow">Advanced</p>
              <h2 id="settings-group-runtime-title">Advanced settings</h2>
            </div>
            {renderSettingsSubpageLink("runtime", "Open advanced settings", navigationHandlers, settingsStatuses?.runtime)}
          </section>
        </div>

        <div
          className={`settings-view${initialSettingsView === "planner" ? " settings-view-active" : ""}`}
          data-settings-view-section="planner"
          hidden={initialSettingsView !== "planner"}
          aria-hidden={initialSettingsView !== "planner"}
        >
          <section className="hero hero-settings hero-settings-subpage">
            <p className="settings-breadcrumb">Settings › AI assistant setup</p>
            <h2>AI assistant setup</h2>
            <p className="lede">The AI assistant interprets your voice commands and decides what to do. It requires an OpenAI-compatible API endpoint and key. If you're using OpenAI, the endpoint is <code>https://api.openai.com/v1</code>. For local models via Ollama, use <code>http://localhost:11434</code>.</p>
          </section>
          {renderPanelContent("settings-remote-planner", panelContent)}
        </div>

        <div
          className={`settings-view${initialSettingsView === "tts" ? " settings-view-active" : ""}`}
          data-settings-view-section="tts"
          hidden={initialSettingsView !== "tts"}
          aria-hidden={initialSettingsView !== "tts"}
        >
          <section className="hero hero-settings hero-settings-subpage">
            <p className="settings-breadcrumb">Settings › Voice output setup</p>
            <h2>Voice output setup</h2>
            <p className="lede">Voice output converts the assistant's text responses to speech. Choose a local model for offline use or a remote service for higher quality voices.</p>
          </section>
          {renderPanelContent("settings-tts-provider", panelContent)}
          {renderPanelContent("settings-tts-model", panelContent)}
          {renderPanelContent("settings-local-tts-model", panelContent)}
          {renderPanelContent("settings-remote-tts", panelContent)}
          {renderPanelContent("settings-tts-voice", panelContent)}
        </div>

        <div
          className={`settings-view${initialSettingsView === "asr" ? " settings-view-active" : ""}`}
          data-settings-view-section="asr"
          hidden={initialSettingsView !== "asr"}
          aria-hidden={initialSettingsView !== "asr"}
        >
          <section className="hero hero-settings hero-settings-subpage">
            <p className="settings-breadcrumb">Settings › Voice input setup</p>
            <h2>Voice input setup</h2>
            <p className="lede">Voice input converts your speech to text. Choose a local Whisper model for offline use or a remote transcription service.</p>
          </section>
          {renderPanelContent("settings-asr-provider", panelContent)}
          {renderPanelContent("settings-local-asr-model", panelContent)}
          {renderPanelContent("settings-remote-asr", panelContent)}
        </div>

        <div
          className={`settings-view${initialSettingsView === "runtime" ? " settings-view-active" : ""}`}
          data-settings-view-section="runtime"
          hidden={initialSettingsView !== "runtime"}
          aria-hidden={initialSettingsView !== "runtime"}
        >
          <section className="hero hero-settings hero-settings-subpage">
            <p className="settings-breadcrumb">Settings › Advanced settings</p>
            <h2>Advanced settings</h2>
            <p className="lede">Model management, confirmation behavior, and OCR settings. Most users won't need to change these.</p>
          </section>
          {renderPanelContent("settings-model-management", panelContent)}
          {renderPanelContent("settings-confirmation", panelContent)}
          {renderPanelContent("settings-ocr-threshold", panelContent)}
        </div>
      </section>
    </main>
  );
}

function renderShellTree(
  initialAppView: AppView,
  initialSettingsView: SettingsView,
  panelContent?: AppShellPanelContent,
  navigationHandlers?: AppShellNavigationHandlers,
  settingsStatuses?: SettingsStatuses,
) {
  return (
    <AppShellMarkup
      initialAppView={initialAppView}
      initialSettingsView={initialSettingsView}
      panelContent={panelContent}
      navigationHandlers={navigationHandlers}
      settingsStatuses={settingsStatuses}
    />
  );
}

export function AppShellRuntime(props: {
  appView: AppView;
  settingsView: SettingsView;
  panelContent: AppShellPanelContent;
  navigationHandlers?: AppShellNavigationHandlers;
  settingsStatuses?: SettingsStatuses;
}) {
  return renderShellTree(
    props.appView,
    props.settingsView,
    props.panelContent,
    props.navigationHandlers,
    props.settingsStatuses,
  );
}

export async function renderAppShell(props?: Partial<AppShellMarkupProps>): Promise<string> {
  const { renderToStaticMarkup } = await import("react-dom/server");
  return renderToStaticMarkup(renderShellTree(
    props?.initialAppView ?? "workspace",
    props?.initialSettingsView ?? "overview",
    props?.panelContent,
    props?.navigationHandlers,
    props?.settingsStatuses,
  ));
}
