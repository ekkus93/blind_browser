use super::api_key_tools::{
    fetch_openai_compatible_models, test_remote_openai_profile_api_key, RemoteApiKeyTarget,
    RemoteOpenAiApiKeyTestProfile,
};
use super::model_management::{
    download_hugging_face_directory, download_hugging_face_file, kitten_download_plan_for_model_id,
    resolved_models_dir_for_app, whisper_download_plan_for_model_id,
};
use super::settings_adapters::{
    active_local_asr_profile, active_local_tts_profile, build_model_management_settings,
};
use super::{AppCore, DownloadedLocalModelData, ModelManagementSettingsData};
use crate::browser::BrowserVisibilityMode;
#[cfg(feature = "remote-openai")]
use crate::config::resolve_secret_ref_for_endpoint;
use crate::config::{AppConfig, ConfigError, ModelManagementSettings};
#[cfg(feature = "remote-openai")]
use crate::provider_endpoint::ProviderEndpointScope;

#[cfg(feature = "remote-openai")]
#[derive(Debug, PartialEq, Eq)]
struct RemotePlannerModelCredentials {
    api_key: Option<String>,
    organization: Option<String>,
    project: Option<String>,
}

#[cfg(feature = "remote-openai")]
fn resolve_remote_planner_model_credentials(
    profile_name: &str,
    profile: &crate::config::RemotePlannerProfile,
    requested_scope: &ProviderEndpointScope,
    api_key_override: Option<&str>,
) -> Result<RemotePlannerModelCredentials, String> {
    let configured_scope = ProviderEndpointScope::parse(&profile.base_url).map_err(|reason| {
        format!(
            "Remote planner profile '{profile_name}' has an invalid configured endpoint: {reason}"
        )
    })?;
    let entered_key = api_key_override
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if let Some(entered_key) = entered_key {
        let same_scope = requested_scope == &configured_scope;
        let organization = if same_scope {
            profile
                .organization
                .as_ref()
                .map(|secret| {
                    resolve_secret_ref_for_endpoint(
                        secret,
                        "planner",
                        profile_name,
                        &configured_scope,
                    )
                })
                .transpose()
                .map_err(|reason| {
                    format!(
                        "Remote planner model list could not read the configured organization secret: {reason}"
                    )
                })?
        } else {
            None
        };
        return Ok(RemotePlannerModelCredentials {
            api_key: Some(entered_key.to_string()),
            organization,
            project: same_scope.then(|| profile.project.clone()).flatten(),
        });
    }

    if requested_scope != &configured_scope {
        return Err(format!(
            "The requested endpoint {} differs from the saved credential destination {}. Enter a temporary key for the displayed endpoint, or save the endpoint and re-enter the key to authorize it.",
            requested_scope.normalized_base_url(),
            configured_scope.normalized_base_url()
        ));
    }

    let api_key = resolve_secret_ref_for_endpoint(
        &profile.api_key,
        "planner",
        profile_name,
        &configured_scope,
    )
    .map_err(|reason| {
        format!("Remote planner model list could not read the configured API key: {reason}")
    })?;
    let organization = profile
        .organization
        .as_ref()
        .map(|secret| {
            resolve_secret_ref_for_endpoint(
                secret,
                "planner",
                profile_name,
                &configured_scope,
            )
        })
        .transpose()
        .map_err(|reason| {
            format!(
                "Remote planner model list could not read the configured organization secret: {reason}"
            )
        })?;

    Ok(RemotePlannerModelCredentials {
        api_key: Some(api_key),
        organization,
        project: profile.project.clone(),
    })
}

impl AppCore {
    pub fn set_playback_volume(&mut self, playback_volume: f32) -> Result<(), ConfigError> {
        let mut audio = self.config.audio.clone();
        audio.playback_volume = playback_volume;
        self.update_audio_settings(audio)
    }

    pub fn set_playback_speed(&mut self, playback_speed: f32) -> Result<(), ConfigError> {
        let mut audio = self.config.audio.clone();
        audio.playback_speed = playback_speed;
        self.update_audio_settings(audio)
    }

    pub fn set_default_tts_voice(
        &mut self,
        default_tts_voice: impl Into<String>,
    ) -> Result<(), ConfigError> {
        let mut audio = self.config.audio.clone();
        audio.default_tts_voice = default_tts_voice.into();
        self.update_audio_settings(audio)
    }

    pub fn set_active_tts_profile(
        &mut self,
        profile_name: impl Into<String>,
    ) -> Result<(), ConfigError> {
        let profile_name = profile_name.into();
        let mut selection = self.config.providers.tts.clone();
        match selection.mode {
            crate::config::ProviderMode::Local => {
                selection.local_profile = Some(profile_name);
            }
            crate::config::ProviderMode::Remote => {
                selection.remote_profile = Some(profile_name);
            }
        }

        let config =
            AppConfig::persist_tts_provider_selection_for_app(&self.app_handle, &selection)?;
        self.config = config;
        Ok(())
    }

    pub fn set_tts_provider_mode(
        &mut self,
        mode: crate::config::ProviderMode,
    ) -> Result<(), ConfigError> {
        let mut selection = self.config.providers.tts.clone();
        selection.mode = mode;

        let config =
            AppConfig::persist_tts_provider_selection_for_app(&self.app_handle, &selection)?;
        self.config = config;
        Ok(())
    }

    pub fn set_asr_provider_mode(
        &mut self,
        mode: crate::config::ProviderMode,
    ) -> Result<(), ConfigError> {
        let mut selection = self.config.providers.asr.clone();
        selection.mode = mode;

        let config =
            AppConfig::persist_asr_provider_selection_for_app(&self.app_handle, &selection)?;
        self.config = config;
        Ok(())
    }

    pub fn set_remote_planner_api_key(
        &mut self,
        profile_name: &str,
        api_key: &str,
    ) -> Result<(), ConfigError> {
        self.config = AppConfig::persist_remote_planner_api_key_for_app(
            &self.app_handle,
            profile_name,
            api_key,
        )?;
        Ok(())
    }

    pub fn set_remote_planner_connection_settings(
        &mut self,
        profile_name: &str,
        base_url: &str,
        model: &str,
    ) -> Result<(), ConfigError> {
        self.config = AppConfig::persist_remote_planner_connection_settings_for_app(
            &self.app_handle,
            profile_name,
            base_url,
            model,
        )?;
        Ok(())
    }

    pub fn reset_remote_planner_connection_settings_to_defaults(
        &mut self,
        profile_name: &str,
    ) -> Result<(), ConfigError> {
        self.config = AppConfig::reset_remote_planner_connection_settings_to_defaults_for_app(
            &self.app_handle,
            profile_name,
        )?;
        Ok(())
    }

    pub fn set_remote_tts_api_key(
        &mut self,
        profile_name: &str,
        api_key: &str,
    ) -> Result<(), ConfigError> {
        self.config =
            AppConfig::persist_remote_tts_api_key_for_app(&self.app_handle, profile_name, api_key)?;
        Ok(())
    }

    pub fn set_remote_asr_api_key(
        &mut self,
        profile_name: &str,
        api_key: &str,
    ) -> Result<(), ConfigError> {
        self.config =
            AppConfig::persist_remote_asr_api_key_for_app(&self.app_handle, profile_name, api_key)?;
        Ok(())
    }

    pub fn test_remote_planner_api_key(
        &self,
        profile_name: &str,
        api_key_override: Option<&str>,
        timeout_ms_override: Option<u64>,
    ) -> Result<String, String> {
        let profile = self
            .config
            .remote_planner_profiles
            .get(profile_name)
            .ok_or_else(|| format!("unknown remote planner profile '{profile_name}'"))?;
        test_remote_openai_profile_api_key(
            RemoteApiKeyTarget::Planner,
            RemoteOpenAiApiKeyTestProfile {
                profile_name,
                provider: &profile.provider,
                base_url: &profile.base_url,
                configured_api_key: &profile.api_key,
                organization: profile.organization.as_ref(),
                project: profile.project.as_deref(),
                timeout_ms: timeout_ms_override.unwrap_or(profile.timeout_ms),
            },
            api_key_override,
        )
    }

    pub fn list_remote_planner_models(
        &self,
        profile_name: &str,
        base_url_override: Option<&str>,
        api_key_override: Option<&str>,
        timeout_ms_override: Option<u64>,
    ) -> Result<Vec<String>, String> {
        let profile = self
            .config
            .remote_planner_profiles
            .get(profile_name)
            .ok_or_else(|| format!("unknown remote planner profile '{profile_name}'"))?;

        let requested_scope =
            ProviderEndpointScope::parse(base_url_override.unwrap_or(&profile.base_url))
                .map_err(|reason| format!("Remote planner model endpoint is invalid: {reason}"))?;
        let credentials = resolve_remote_planner_model_credentials(
            profile_name,
            profile,
            &requested_scope,
            api_key_override,
        )?;

        fetch_openai_compatible_models(
            &requested_scope,
            credentials.api_key.as_deref(),
            credentials.organization.as_deref(),
            credentials.project.as_deref(),
            timeout_ms_override.unwrap_or(profile.timeout_ms),
        )
    }

    pub fn test_remote_tts_api_key(
        &self,
        profile_name: &str,
        api_key_override: Option<&str>,
        timeout_ms_override: Option<u64>,
    ) -> Result<String, String> {
        let profile = self
            .config
            .remote_tts_profiles
            .get(profile_name)
            .ok_or_else(|| format!("unknown remote TTS profile '{profile_name}'"))?;
        test_remote_openai_profile_api_key(
            RemoteApiKeyTarget::Tts,
            RemoteOpenAiApiKeyTestProfile {
                profile_name,
                provider: &profile.provider,
                base_url: &profile.base_url,
                configured_api_key: &profile.api_key,
                organization: profile.organization.as_ref(),
                project: profile.project.as_deref(),
                timeout_ms: timeout_ms_override.unwrap_or(profile.timeout_ms),
            },
            api_key_override,
        )
    }

    pub fn test_remote_asr_api_key(
        &self,
        profile_name: &str,
        api_key_override: Option<&str>,
        timeout_ms_override: Option<u64>,
    ) -> Result<String, String> {
        let profile = self
            .config
            .remote_asr_profiles
            .get(profile_name)
            .ok_or_else(|| format!("unknown remote ASR profile '{profile_name}'"))?;
        test_remote_openai_profile_api_key(
            RemoteApiKeyTarget::Asr,
            RemoteOpenAiApiKeyTestProfile {
                profile_name,
                provider: &profile.provider,
                base_url: &profile.base_url,
                configured_api_key: &profile.api_key,
                organization: profile.organization.as_ref(),
                project: profile.project.as_deref(),
                timeout_ms: timeout_ms_override.unwrap_or(profile.timeout_ms),
            },
            api_key_override,
        )
    }

    pub fn current_model_management_settings(&self) -> ModelManagementSettingsData {
        build_model_management_settings(&self.config)
    }

    pub fn set_model_management_settings(
        &mut self,
        models_dir: &str,
        check_on_startup: bool,
        auto_download_missing: bool,
    ) -> Result<(), ConfigError> {
        let settings = ModelManagementSettings {
            models_dir: models_dir.trim().to_string(),
            check_on_startup,
            auto_download_missing,
        };
        self.config =
            AppConfig::persist_model_management_settings_for_app(&self.app_handle, &settings)?;
        Ok(())
    }

    pub fn download_active_local_tts_model(&mut self) -> Result<DownloadedLocalModelData, String> {
        let (profile_name, profile) = active_local_tts_profile(&self.config)?;
        let model_id = profile.model_id.clone();
        let plan = kitten_download_plan_for_model_id(&model_id)?;
        let models_dir =
            resolved_models_dir_for_app(&self.app_handle, &self.config.models.models_dir)?;
        let target_dir = models_dir.join(plan.directory_name);

        download_hugging_face_directory(&target_dir, plan.repository, plan.files)?;

        let model_path = target_dir
            .to_str()
            .ok_or_else(|| {
                format!(
                    "downloaded model path is not valid UTF-8: {}",
                    target_dir.display()
                )
            })?
            .to_string();
        self.config = AppConfig::persist_local_tts_model_path_for_app(
            &self.app_handle,
            &profile_name,
            &model_path,
        )
        .map_err(|error| error.to_string())?;

        Ok(DownloadedLocalModelData {
            profile_name,
            model_id,
            model_path,
            source_url: format!("https://huggingface.co/{}", plan.repository),
        })
    }

    pub fn download_active_local_asr_model(&mut self) -> Result<DownloadedLocalModelData, String> {
        let (profile_name, profile) = active_local_asr_profile(&self.config)?;
        let model_id = profile.model_id.clone();
        let plan = whisper_download_plan_for_model_id(&model_id)?;
        let models_dir =
            resolved_models_dir_for_app(&self.app_handle, &self.config.models.models_dir)?;
        let target_path = models_dir.join("whisper").join(plan.file_name);

        download_hugging_face_file(&target_path, plan.repository, plan.file_name)?;

        let model_path = target_path
            .to_str()
            .ok_or_else(|| {
                format!(
                    "downloaded model path is not valid UTF-8: {}",
                    target_path.display()
                )
            })?
            .to_string();
        self.config = AppConfig::persist_local_asr_model_path_for_app(
            &self.app_handle,
            &profile_name,
            &model_path,
        )
        .map_err(|error| error.to_string())?;

        Ok(DownloadedLocalModelData {
            profile_name,
            model_id,
            model_path,
            source_url: format!(
                "https://huggingface.co/{}/resolve/main/{}",
                plan.repository, plan.file_name
            ),
        })
    }

    pub fn set_browser_visibility(&mut self, mode: BrowserVisibilityMode) {
        self.state.browser_visibility = mode;
    }

    pub fn set_confirmation_confidence_threshold(
        &mut self,
        confirmation_confidence_threshold: f32,
    ) -> Result<(), ConfigError> {
        let mut safety = self.config.safety.clone();
        safety.confirmation_confidence_threshold = confirmation_confidence_threshold;
        let next_config = AppConfig::persist_safety_settings_for_app(&self.app_handle, &safety)?;
        self.config = next_config;
        Ok(())
    }

    pub fn set_allow_click_without_confirmation(
        &mut self,
        allow_click_without_confirmation: bool,
    ) -> Result<(), ConfigError> {
        let mut safety = self.config.safety.clone();
        safety.allow_click_without_confirmation = allow_click_without_confirmation;
        let next_config = AppConfig::persist_safety_settings_for_app(&self.app_handle, &safety)?;
        self.config = next_config;
        Ok(())
    }

    pub fn set_ocr_thresholds(
        &mut self,
        sparse_text_char_threshold: u32,
        sparse_text_region_threshold: u32,
    ) -> Result<(), ConfigError> {
        let mut ocr = self.config.ocr.clone();
        ocr.sparse_text_char_threshold = sparse_text_char_threshold;
        ocr.sparse_text_region_threshold = sparse_text_region_threshold;
        let next_config = AppConfig::persist_ocr_settings_for_app(&self.app_handle, &ocr)?;
        self.config = next_config;
        Ok(())
    }
}

#[cfg(all(test, feature = "remote-openai"))]
mod tests {
    use super::{resolve_remote_planner_model_credentials, RemotePlannerModelCredentials};
    use crate::config::{KeyringRef, RemotePlannerProfile, RemoteProviderKind, SecretRef};
    use crate::provider_endpoint::ProviderEndpointScope;

    fn planner_profile(secret: SecretRef) -> RemotePlannerProfile {
        RemotePlannerProfile {
            provider: RemoteProviderKind::OpenAi,
            base_url: String::from("https://api.example.com/v1"),
            model: String::from("gpt-4"),
            api_key: secret,
            organization: None,
            project: Some(String::from("project-a")),
            temperature_milli: 200,
            max_output_tokens: 1024,
            timeout_ms: 30_000,
        }
    }

    #[test]
    fn changed_endpoint_with_empty_override_fails_before_resolving_secret() {
        let profile = planner_profile(SecretRef::FromEnv {
            from_env: String::from("BLIND_BROWSER_NONEXISTENT_KEY_XYZ99"),
        });
        for endpoint in [
            "https://other.example.com/v1",
            "https://api.example.com:8443/v1",
            "https://api.example.com/v2",
        ] {
            let requested = ProviderEndpointScope::parse(endpoint).unwrap();
            let error = resolve_remote_planner_model_credentials(
                "openai-default",
                &profile,
                &requested,
                Some("  "),
            )
            .expect_err("changed endpoint must fail closed");
            assert!(error.contains("differs from the saved credential destination"));
            assert!(!error.contains("environment variable"));
        }
    }

    #[test]
    fn temporary_key_for_changed_endpoint_does_not_attach_profile_headers() {
        let profile = planner_profile(SecretRef::FromKeyring {
            from_keyring: KeyringRef {
                service: String::from("blind_browser"),
                account: String::from("legacy-unbound"),
            },
        });
        let requested = ProviderEndpointScope::parse("https://other.example.com/v1").unwrap();
        let credentials = resolve_remote_planner_model_credentials(
            "openai-default",
            &profile,
            &requested,
            Some("temporary-key"),
        )
        .expect("entered key should be scoped to the displayed endpoint");

        assert_eq!(
            credentials,
            RemotePlannerModelCredentials {
                api_key: Some(String::from("temporary-key")),
                organization: None,
                project: None,
            }
        );
    }
}
