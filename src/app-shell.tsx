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

const LEDE_CLASS = "mt-[18px] max-w-[60ch] text-[1.05rem] leading-[1.6] text-[var(--color-text-secondary)]";
const SETTINGS_GROUP_EYEBROW_CLASS = "m-[0_0_8px] uppercase tracking-[0.18em] text-[0.76rem] text-[var(--eyebrow-color)]";
const SETTINGS_GROUP_H2_CLASS = "[font-family:var(--font-display)] text-[clamp(1.2rem,2.2vw,1.6rem)] leading-[1.05]";
const SETTINGS_BREADCRUMB_CLASS = "text-[0.84rem] text-[var(--color-text-muted)] m-0";

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
    <main className="max-w-[980px] mx-auto p-[56px_24px_72px] max-sm:p-[40px_18px_56px]">
      <header className="flex items-center gap-3 mb-6 max-sm:mb-5">
        {showAppViewAction ? renderAppViewActionButton(initialAppView, navigationHandlers) : null}
        {renderSettingsSubpageBackButton(showBackButton, navigationHandlers)}
        {renderPanelContent("voice-status", panelContent)}
      </header>

      {renderPanelContent("app-alert", panelContent)}

      {/* Rendered unconditionally, outside both the workspace and settings
          sections below: a confirmation or remote-data-consent dialog can be
          raised while the app is in Settings view (push-to-talk and the
          continuous-listening loop are not gated to workspace view), and the
          workspace/settings sections below are hidden+aria-hidden whenever
          they are not the active view. A safety gate that cannot be seen or
          heard is equivalent to no gate -- this must stay reachable
          regardless of which view is currently showing. */}
      {renderPanelContent("confirmation-panel", panelContent)}

      <section
        data-app-view-section="workspace"
        hidden={!workspaceActive}
        aria-hidden={!workspaceActive}
      >
        <div className="flex flex-col gap-[18px] mt-[18px] max-sm:gap-[14px]" data-workspace-control-bar="true">
          {renderPanelContent("push-to-talk", panelContent)}
          {renderPanelContent("url-input", panelContent)}
        </div>
        {renderPanelContent("status", panelContent)}
      </section>

      <section
        data-app-view-section="settings"
        hidden={!settingsActive}
        aria-hidden={!settingsActive}
      >
        <div
          data-settings-view-section="overview"
          hidden={initialSettingsView !== "overview"}
          aria-hidden={initialSettingsView !== "overview"}
        >
          <section className="p-[24px_0_32px] pt-3">
            <h1 className="[font-family:var(--font-display)] text-[clamp(1.8rem,3.5vw,2.6rem)] leading-[0.94] max-w-[10ch] max-sm:max-w-none">Settings</h1>
          </section>
          {renderPanelContent("settings-guidance", panelContent)}
          <section className="mt-[30px]" aria-labelledby="settings-group-playback-title">
            <div className="max-w-[62ch]">
              <p className={SETTINGS_GROUP_EYEBROW_CLASS}>Listening</p>
              <h2 id="settings-group-playback-title" className={SETTINGS_GROUP_H2_CLASS}>Playback</h2>
            </div>
            {renderPanelContent("audio-controls", panelContent)}
          </section>
          <section className="mt-[30px] grid [grid-template-columns:minmax(0,1fr)_auto] items-end gap-[18px] max-sm:[grid-template-columns:1fr] max-sm:items-start" aria-labelledby="settings-group-planner-title">
            <div className="max-w-[62ch]">
              <p className={SETTINGS_GROUP_EYEBROW_CLASS}>Command interpretation</p>
              <h2 id="settings-group-planner-title" className={SETTINGS_GROUP_H2_CLASS}>AI assistant</h2>
            </div>
            {renderSettingsSubpageLink("planner", "Open AI assistant setup", navigationHandlers, settingsStatuses?.planner)}
          </section>
          <section className="mt-[30px] grid [grid-template-columns:minmax(0,1fr)_auto] items-end gap-[18px] max-sm:[grid-template-columns:1fr] max-sm:items-start" aria-labelledby="settings-group-tts-title">
            <div className="max-w-[62ch]">
              <p className={SETTINGS_GROUP_EYEBROW_CLASS}>Speech output</p>
              <h2 id="settings-group-tts-title" className={SETTINGS_GROUP_H2_CLASS}>Voice output</h2>
            </div>
            {renderSettingsSubpageLink("tts", "Open voice output setup", navigationHandlers, settingsStatuses?.tts)}
          </section>
          <section className="mt-[30px] grid [grid-template-columns:minmax(0,1fr)_auto] items-end gap-[18px] max-sm:[grid-template-columns:1fr] max-sm:items-start" aria-labelledby="settings-group-asr-title">
            <div className="max-w-[62ch]">
              <p className={SETTINGS_GROUP_EYEBROW_CLASS}>Speech input</p>
              <h2 id="settings-group-asr-title" className={SETTINGS_GROUP_H2_CLASS}>Voice input</h2>
            </div>
            {renderSettingsSubpageLink("asr", "Open voice input setup", navigationHandlers, settingsStatuses?.asr)}
          </section>
          <section className="mt-[30px] grid [grid-template-columns:minmax(0,1fr)_auto] items-end gap-[18px] max-sm:[grid-template-columns:1fr] max-sm:items-start" aria-labelledby="settings-group-runtime-title">
            <div className="max-w-[62ch]">
              <p className={SETTINGS_GROUP_EYEBROW_CLASS}>Advanced</p>
              <h2 id="settings-group-runtime-title" className={SETTINGS_GROUP_H2_CLASS}>Advanced settings</h2>
            </div>
            {renderSettingsSubpageLink("runtime", "Open advanced settings", navigationHandlers, settingsStatuses?.runtime)}
          </section>
        </div>

        <div
          data-settings-view-section="planner"
          hidden={initialSettingsView !== "planner"}
          aria-hidden={initialSettingsView !== "planner"}
        >
          <section className="p-[24px_0_32px] pt-3 flex flex-col gap-[10px]">
            <p className={SETTINGS_BREADCRUMB_CLASS}>Settings › AI assistant setup</p>
            <h2>AI assistant setup</h2>
            <p className={LEDE_CLASS}>The AI assistant interprets your voice commands and decides what to do. It requires an OpenAI-compatible API endpoint and key. If you're using OpenAI, the endpoint is <code>https://api.openai.com/v1</code>. For local models via Ollama, use <code>http://localhost:11434</code>.</p>
          </section>
          {renderPanelContent("settings-remote-planner", panelContent)}
        </div>

        <div
          data-settings-view-section="tts"
          hidden={initialSettingsView !== "tts"}
          aria-hidden={initialSettingsView !== "tts"}
        >
          <section className="p-[24px_0_32px] pt-3 flex flex-col gap-[10px]">
            <p className={SETTINGS_BREADCRUMB_CLASS}>Settings › Voice output setup</p>
            <h2>Voice output setup</h2>
            <p className={LEDE_CLASS}>Voice output converts the assistant's text responses to speech. Choose a local model for offline use or a remote service for higher quality voices.</p>
          </section>
          {renderPanelContent("settings-tts-provider", panelContent)}
          {renderPanelContent("settings-tts-model", panelContent)}
          {renderPanelContent("settings-local-tts-model", panelContent)}
          {renderPanelContent("settings-remote-tts", panelContent)}
          {renderPanelContent("settings-tts-voice", panelContent)}
        </div>

        <div
          data-settings-view-section="asr"
          hidden={initialSettingsView !== "asr"}
          aria-hidden={initialSettingsView !== "asr"}
        >
          <section className="p-[24px_0_32px] pt-3 flex flex-col gap-[10px]">
            <p className={SETTINGS_BREADCRUMB_CLASS}>Settings › Voice input setup</p>
            <h2>Voice input setup</h2>
            <p className={LEDE_CLASS}>Voice input converts your speech to text. Choose a local Whisper model for offline use or a remote transcription service.</p>
          </section>
          {renderPanelContent("settings-asr-provider", panelContent)}
          {renderPanelContent("settings-local-asr-model", panelContent)}
          {renderPanelContent("settings-remote-asr", panelContent)}
        </div>

        <div
          data-settings-view-section="runtime"
          hidden={initialSettingsView !== "runtime"}
          aria-hidden={initialSettingsView !== "runtime"}
        >
          <section className="p-[24px_0_32px] pt-3 flex flex-col gap-[10px]">
            <p className={SETTINGS_BREADCRUMB_CLASS}>Settings › Advanced settings</p>
            <h2>Advanced settings</h2>
            <p className={LEDE_CLASS}>Model management, confirmation behavior, and OCR settings. Most users won't need to change these.</p>
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
