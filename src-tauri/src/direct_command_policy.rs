use crate::commands::ActionClass;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum DirectCommandName {
    ResolveCommand,
    ExecutePlannerOutput,
    SubmitConfirmationResponse,
    StartListening,
    StopListening,
    TranscribeCommand,
    TranscribeAndExecuteCommand,
    OpenUrl,
    OpenExternalUrl,
    GetAgentState,
    SetPlaybackVolume,
    SetPlaybackSpeed,
    SetBrowserVisibility,
    SetTtsVoice,
    SetConfirmationThreshold,
    SetAllowClickWithoutConfirmation,
    SetRemotePlannerPrivacySettings,
    SetOcrThresholds,
    SetRemotePlannerConnectionSettings,
    ResetRemotePlannerConnectionSettings,
    ListRemotePlannerModels,
    SetRemotePlannerApiKey,
    SetRemoteTtsApiKey,
    SetRemoteAsrApiKey,
    TestRemotePlannerApiKey,
    TestRemoteTtsApiKey,
    TestRemoteAsrApiKey,
    GetModelManagementSettings,
    SetModelManagementSettings,
    DownloadActiveLocalTtsModel,
    DownloadActiveLocalAsrModel,
    SetTtsProviderSelection,
    SetAsrProviderSelection,
    SetTtsModelSelection,
}

impl DirectCommandName {
    pub(crate) const ALL: &'static [Self] = &[
        Self::ResolveCommand,
        Self::ExecutePlannerOutput,
        Self::SubmitConfirmationResponse,
        Self::StartListening,
        Self::StopListening,
        Self::TranscribeCommand,
        Self::TranscribeAndExecuteCommand,
        Self::OpenUrl,
        Self::OpenExternalUrl,
        Self::GetAgentState,
        Self::SetPlaybackVolume,
        Self::SetPlaybackSpeed,
        Self::SetBrowserVisibility,
        Self::SetTtsVoice,
        Self::SetConfirmationThreshold,
        Self::SetAllowClickWithoutConfirmation,
        Self::SetRemotePlannerPrivacySettings,
        Self::SetOcrThresholds,
        Self::SetRemotePlannerConnectionSettings,
        Self::ResetRemotePlannerConnectionSettings,
        Self::ListRemotePlannerModels,
        Self::SetRemotePlannerApiKey,
        Self::SetRemoteTtsApiKey,
        Self::SetRemoteAsrApiKey,
        Self::TestRemotePlannerApiKey,
        Self::TestRemoteTtsApiKey,
        Self::TestRemoteAsrApiKey,
        Self::GetModelManagementSettings,
        Self::SetModelManagementSettings,
        Self::DownloadActiveLocalTtsModel,
        Self::DownloadActiveLocalAsrModel,
        Self::SetTtsProviderSelection,
        Self::SetAsrProviderSelection,
        Self::SetTtsModelSelection,
    ];

    pub(crate) const fn as_handler_name(self) -> &'static str {
        match self {
            Self::ResolveCommand => "resolve_command",
            Self::ExecutePlannerOutput => "execute_planner_output",
            Self::SubmitConfirmationResponse => "submit_confirmation_response",
            Self::StartListening => "start_listening",
            Self::StopListening => "stop_listening",
            Self::TranscribeCommand => "transcribe_command",
            Self::TranscribeAndExecuteCommand => "transcribe_and_execute_command",
            Self::OpenUrl => "open_url",
            Self::OpenExternalUrl => "open_external_url",
            Self::GetAgentState => "get_agent_state",
            Self::SetPlaybackVolume => "set_playback_volume",
            Self::SetPlaybackSpeed => "set_playback_speed",
            Self::SetBrowserVisibility => "set_browser_visibility",
            Self::SetTtsVoice => "set_tts_voice",
            Self::SetConfirmationThreshold => "set_confirmation_threshold",
            Self::SetAllowClickWithoutConfirmation => "set_allow_click_without_confirmation",
            Self::SetRemotePlannerPrivacySettings => "set_remote_planner_privacy_settings",
            Self::SetOcrThresholds => "set_ocr_thresholds",
            Self::SetRemotePlannerConnectionSettings => "set_remote_planner_connection_settings",
            Self::ResetRemotePlannerConnectionSettings => {
                "reset_remote_planner_connection_settings"
            }
            Self::ListRemotePlannerModels => "list_remote_planner_models",
            Self::SetRemotePlannerApiKey => "set_remote_planner_api_key",
            Self::SetRemoteTtsApiKey => "set_remote_tts_api_key",
            Self::SetRemoteAsrApiKey => "set_remote_asr_api_key",
            Self::TestRemotePlannerApiKey => "test_remote_planner_api_key",
            Self::TestRemoteTtsApiKey => "test_remote_tts_api_key",
            Self::TestRemoteAsrApiKey => "test_remote_asr_api_key",
            Self::GetModelManagementSettings => "get_model_management_settings",
            Self::SetModelManagementSettings => "set_model_management_settings",
            Self::DownloadActiveLocalTtsModel => "download_active_local_tts_model",
            Self::DownloadActiveLocalAsrModel => "download_active_local_asr_model",
            Self::SetTtsProviderSelection => "set_tts_provider_selection",
            Self::SetAsrProviderSelection => "set_asr_provider_selection",
            Self::SetTtsModelSelection => "set_tts_model_selection",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DirectCommandPolicy {
    pub(crate) class: ActionClass,
    pub(crate) requires_user_gesture: bool,
    pub(crate) mutates_runtime_state: bool,
    pub(crate) mutates_config: bool,
    pub(crate) persists_secret: bool,
    pub(crate) performs_network_io: bool,
    pub(crate) credential_bearing_network_io: bool,
    pub(crate) transmits_page_context: bool,
    pub(crate) downloads_executable_or_model_artifact: bool,
    pub(crate) launches_external_program: bool,
}

// The constructor mirrors every security field so each registry entry must make
// every authority and side-effect property explicit at the call site.
#[allow(clippy::too_many_arguments)]
const fn policy(
    class: ActionClass,
    requires_user_gesture: bool,
    mutates_runtime_state: bool,
    mutates_config: bool,
    persists_secret: bool,
    performs_network_io: bool,
    credential_bearing_network_io: bool,
    transmits_page_context: bool,
    downloads_executable_or_model_artifact: bool,
    launches_external_program: bool,
) -> DirectCommandPolicy {
    DirectCommandPolicy {
        class,
        requires_user_gesture,
        mutates_runtime_state,
        mutates_config,
        persists_secret,
        performs_network_io,
        credential_bearing_network_io,
        transmits_page_context,
        downloads_executable_or_model_artifact,
        launches_external_program,
    }
}

pub(crate) const fn direct_command_policy(name: DirectCommandName) -> DirectCommandPolicy {
    use ActionClass as A;
    use DirectCommandName as D;
    match name {
        D::ResolveCommand => policy(
            A::OtherSideEffect,
            true,
            false,
            false,
            false,
            true,
            true,
            true,
            false,
            false,
        ),
        D::ExecutePlannerOutput => policy(
            A::OtherSideEffect,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        ),
        D::SubmitConfirmationResponse => policy(
            A::OtherSideEffect,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        ),
        D::StartListening | D::StopListening => policy(
            A::ReversibleLocalStateChange,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        ),
        D::TranscribeCommand => policy(
            A::ReadOnly,
            true,
            true,
            false,
            false,
            true,
            true,
            false,
            false,
            false,
        ),
        D::TranscribeAndExecuteCommand => policy(
            A::OtherSideEffect,
            true,
            true,
            false,
            false,
            true,
            true,
            true,
            false,
            false,
        ),
        D::OpenUrl => policy(
            A::BrowserNavigation,
            true,
            true,
            false,
            false,
            true,
            false,
            false,
            false,
            false,
        ),
        D::OpenExternalUrl => policy(
            A::BrowserNavigation,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            true,
        ),
        D::GetAgentState | D::GetModelManagementSettings => policy(
            A::ReadOnly,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        ),
        D::SetPlaybackVolume | D::SetPlaybackSpeed | D::SetBrowserVisibility | D::SetTtsVoice => {
            policy(
                A::ReversibleLocalStateChange,
                true,
                true,
                true,
                false,
                false,
                false,
                false,
                false,
                false,
            )
        }
        D::SetConfirmationThreshold
        | D::SetAllowClickWithoutConfirmation
        | D::SetRemotePlannerPrivacySettings
        | D::SetOcrThresholds
        | D::SetRemotePlannerConnectionSettings
        | D::ResetRemotePlannerConnectionSettings
        | D::SetModelManagementSettings
        | D::SetTtsProviderSelection
        | D::SetAsrProviderSelection
        | D::SetTtsModelSelection => policy(
            A::ReversibleLocalStateChange,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
        ),
        D::ListRemotePlannerModels
        | D::TestRemotePlannerApiKey
        | D::TestRemoteTtsApiKey
        | D::TestRemoteAsrApiKey => policy(
            A::CredentialOperation,
            true,
            false,
            false,
            false,
            true,
            true,
            false,
            false,
            false,
        ),
        D::SetRemotePlannerApiKey | D::SetRemoteTtsApiKey | D::SetRemoteAsrApiKey => policy(
            A::CredentialOperation,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
        ),
        D::DownloadActiveLocalTtsModel | D::DownloadActiveLocalAsrModel => policy(
            A::ModelDownload,
            true,
            true,
            true,
            false,
            true,
            false,
            false,
            true,
            false,
        ),
    }
}

pub(crate) fn validate_direct_command_registry() {
    for name in DirectCommandName::ALL {
        let policy = direct_command_policy(*name);
        assert!(
            !name.as_handler_name().is_empty(),
            "direct command name must not be empty"
        );
        assert!(
            !policy.persists_secret || policy.mutates_config,
            "secret persistence must be classified as a config mutation"
        );
        assert!(
            !policy.credential_bearing_network_io || policy.performs_network_io,
            "credential-bearing I/O must also be network I/O"
        );
        assert!(
            !policy.transmits_page_context || policy.performs_network_io,
            "page-context transmission must also be network I/O"
        );
        assert!(
            !policy.downloads_executable_or_model_artifact || policy.performs_network_io,
            "artifact download must also be network I/O"
        );
        assert!(
            !policy.launches_external_program || policy.requires_user_gesture,
            "external program launch requires an explicit user gesture"
        );
        std::hint::black_box((policy.class, policy.mutates_runtime_state));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn generated_handler_names() -> BTreeSet<String> {
        let source = include_str!("lib.rs");
        let marker = "tauri::generate_handler![";
        let start = source.find(marker).expect("generate_handler! must exist") + marker.len();
        let rest = &source[start..];
        let end = rest.find("])").expect("generate_handler! must close");
        rest[..end]
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(ToOwned::to_owned)
            .collect()
    }

    #[test]
    fn direct_command_registry_matches_tauri_handler_surface() {
        let generated = generated_handler_names();
        let registered = DirectCommandName::ALL
            .iter()
            .map(|name| name.as_handler_name().to_string())
            .collect::<BTreeSet<_>>();
        assert_eq!(registered, generated);
    }

    #[test]
    fn every_direct_command_has_explicit_policy() {
        for name in DirectCommandName::ALL {
            let policy = direct_command_policy(*name);
            if policy.persists_secret {
                assert!(policy.mutates_config);
                assert_eq!(policy.class, ActionClass::CredentialOperation);
            }
            if policy.credential_bearing_network_io {
                assert!(policy.performs_network_io);
            }
            if policy.downloads_executable_or_model_artifact {
                assert!(policy.performs_network_io);
                assert_eq!(policy.class, ActionClass::ModelDownload);
            }
            if policy.transmits_page_context {
                assert!(policy.performs_network_io);
            }
            if policy.launches_external_program {
                assert!(policy.requires_user_gesture);
            }
        }
    }
}
