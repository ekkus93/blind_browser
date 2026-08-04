use std::fs;
use std::io::Write as _;
use std::path::Path;

use tauri::AppHandle;

use super::keyring_store::{keyring_ref_for_remote_api_key, set_keyring_secret};
use super::loading::{load_document_table_from_path, load_document_table_from_str};
use super::validation::{
    normalize_remote_endpoint, normalize_remote_planner_privacy_settings, validate_audio_settings,
    validate_model_settings,
};
use super::{
    AppConfig, AudioSettings, ConfigError, ModelManagementSettings, ProviderSelection,
    RemotePlannerPrivacySettings, SafetySettings, SecretRef,
};
use crate::ocr::OcrSettings;
use crate::provider_endpoint::ProviderEndpointScope;

fn write_config_atomic(path: &Path, serialized: &str) -> Result<(), ConfigError> {
    let parent = path.parent().ok_or_else(|| {
        ConfigError::Validation(format!(
            "config path {} has no parent directory",
            path.display()
        ))
    })?;

    fs::create_dir_all(parent).map_err(|source| ConfigError::CreateDir {
        path: parent.to_path_buf(),
        source,
    })?;

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            ConfigError::Validation(format!(
                "config path {} has no valid file name",
                path.display()
            ))
        })?;

    let tmp_path = path.with_file_name(format!("{file_name}.tmp"));

    let write_result = (|| -> Result<(), ConfigError> {
        let mut file = fs::File::create(&tmp_path).map_err(|source| ConfigError::Write {
            path: tmp_path.clone(),
            source,
        })?;

        file.write_all(serialized.as_bytes())
            .map_err(|source| ConfigError::Write {
                path: tmp_path.clone(),
                source,
            })?;

        file.sync_all().map_err(|source| ConfigError::Write {
            path: tmp_path.clone(),
            source,
        })?;

        crate::atomic_file::replace_file_atomically(&tmp_path, path).map_err(|message| {
            ConfigError::Write {
                path: path.to_path_buf(),
                source: std::io::Error::other(message),
            }
        })?;

        Ok(())
    })();

    match write_result {
        Ok(()) => Ok(()),
        Err(primary) => {
            // Temporary-file cleanup is part of the atomic persistence contract.
            // A cleanup failure is surfaced with the primary failure instead of ignored.
            match remove_failed_config_temp_file(&tmp_path) {
                Ok(()) => Err(primary),
                Err(cleanup) => Err(ConfigError::Write {
                    path: tmp_path,
                    source: std::io::Error::other(format!(
                        "config write failed: {primary}; temporary-file cleanup failed: {cleanup}"
                    )),
                }),
            }
        }
    }
}

fn remove_failed_config_temp_file(path: &Path) -> std::io::Result<()> {
    remove_failed_config_temp_file_with(path, |candidate| fs::remove_file(candidate))
}

fn remove_failed_config_temp_file_with<F>(path: &Path, remover: F) -> std::io::Result<()>
where
    F: FnOnce(&Path) -> std::io::Result<()>,
{
    match remover(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

impl AppConfig {
    pub fn persist_audio_settings_for_app(
        app_handle: &AppHandle,
        audio: &AudioSettings,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_audio_settings_at_path(&config_path, audio)
    }

    pub fn persist_safety_settings_for_app(
        app_handle: &AppHandle,
        safety: &SafetySettings,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_safety_settings_at_path(&config_path, safety)
    }

    pub fn persist_ocr_settings_for_app(
        app_handle: &AppHandle,
        ocr: &OcrSettings,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_ocr_settings_at_path(&config_path, ocr)
    }

    pub fn persist_model_management_settings_for_app(
        app_handle: &AppHandle,
        models: &ModelManagementSettings,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_model_management_settings_at_path(&config_path, models)
    }

    pub fn persist_remote_planner_privacy_settings_for_app(
        app_handle: &AppHandle,
        settings: &RemotePlannerPrivacySettings,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_remote_planner_privacy_settings_at_path(&config_path, settings)
    }

    pub fn persist_remote_planner_api_key_for_app(
        app_handle: &AppHandle,
        profile_name: &str,
        api_key: &str,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_remote_api_key_at_path(&config_path, profile_name, api_key, "planner")
    }

    pub fn persist_remote_planner_connection_settings_for_app(
        app_handle: &AppHandle,
        profile_name: &str,
        base_url: &str,
        model: &str,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_remote_planner_connection_settings_at_path(
            &config_path,
            profile_name,
            base_url,
            model,
        )
    }

    pub fn reset_remote_planner_connection_settings_to_defaults_for_app(
        app_handle: &AppHandle,
        profile_name: &str,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::reset_remote_planner_connection_settings_to_defaults_at_path(
            &config_path,
            profile_name,
        )
    }

    pub fn persist_remote_tts_api_key_for_app(
        app_handle: &AppHandle,
        profile_name: &str,
        api_key: &str,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_remote_api_key_at_path(&config_path, profile_name, api_key, "tts")
    }

    pub fn persist_remote_asr_api_key_for_app(
        app_handle: &AppHandle,
        profile_name: &str,
        api_key: &str,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_remote_api_key_at_path(&config_path, profile_name, api_key, "asr")
    }

    pub fn persist_local_tts_model_path_for_app(
        app_handle: &AppHandle,
        profile_name: &str,
        model_path: &str,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_local_model_path_at_path(&config_path, profile_name, model_path)
    }

    pub fn persist_local_asr_model_path_for_app(
        app_handle: &AppHandle,
        profile_name: &str,
        model_path: &str,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_local_model_path_at_path(&config_path, profile_name, model_path)
    }

    pub fn persist_audio_settings_at_path(
        path: impl AsRef<Path>,
        audio: &AudioSettings,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let mut issues = Vec::new();
        validate_audio_settings(audio, &mut issues);
        if !issues.is_empty() {
            return Err(ConfigError::Validation(issues.join("\n")));
        }

        let mut document = if path.exists() {
            load_document_table_from_path(path)?
        } else {
            load_document_table_from_str(Self::default_template())?
        };

        document.insert(String::from("audio"), toml::Value::try_from(audio.clone())?);

        let serialized = toml::to_string_pretty(&document)?;
        write_config_atomic(path, &serialized)?;

        Self::load_from_path(path)
    }

    pub fn persist_safety_settings_at_path(
        path: impl AsRef<Path>,
        safety: &SafetySettings,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();

        let mut document = if path.exists() {
            load_document_table_from_path(path)?
        } else {
            load_document_table_from_str(Self::default_template())?
        };

        document.insert(
            String::from("safety"),
            toml::Value::try_from(safety.clone())?,
        );

        let serialized = toml::to_string_pretty(&document)?;
        write_config_atomic(path, &serialized)?;

        Self::load_from_path(path)
    }

    pub fn persist_ocr_settings_at_path(
        path: impl AsRef<Path>,
        ocr: &OcrSettings,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();

        let mut document = if path.exists() {
            load_document_table_from_path(path)?
        } else {
            load_document_table_from_str(Self::default_template())?
        };

        document.insert(String::from("ocr"), toml::Value::try_from(ocr.clone())?);

        let serialized = toml::to_string_pretty(&document)?;
        write_config_atomic(path, &serialized)?;

        Self::load_from_path(path)
    }

    pub fn persist_model_management_settings_at_path(
        path: impl AsRef<Path>,
        models: &ModelManagementSettings,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let mut issues = Vec::new();
        validate_model_settings(models, &mut issues);
        if !issues.is_empty() {
            return Err(ConfigError::Validation(issues.join("\n")));
        }

        let mut document = if path.exists() {
            load_document_table_from_path(path)?
        } else {
            load_document_table_from_str(Self::default_template())?
        };

        document.insert(
            String::from("models"),
            toml::Value::try_from(models.clone())?,
        );

        let serialized = toml::to_string_pretty(&document)?;
        write_config_atomic(path, &serialized)?;

        Self::load_from_path(path)
    }

    pub fn persist_remote_planner_privacy_settings_at_path(
        path: impl AsRef<Path>,
        settings: &RemotePlannerPrivacySettings,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let mut normalized = settings.clone();
        let mut issues = Vec::new();
        normalize_remote_planner_privacy_settings(&mut normalized, &mut issues);
        if !issues.is_empty() {
            return Err(ConfigError::Validation(issues.join("\n")));
        }

        let mut document = if path.exists() {
            load_document_table_from_path(path)?
        } else {
            load_document_table_from_str(Self::default_template())?
        };
        document.insert(
            String::from("remote_planner_privacy"),
            toml::Value::try_from(normalized)?,
        );
        let serialized = toml::to_string_pretty(&document)?;
        write_config_atomic(path, &serialized)?;
        Self::load_from_path(path)
    }

    pub fn persist_remote_api_key_at_path(
        path: impl AsRef<Path>,
        profile_name: &str,
        api_key: &str,
        provider_kind: &str,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let normalized_profile_name = profile_name.trim();
        if normalized_profile_name.is_empty() {
            return Err(ConfigError::Validation(String::from(
                "remote API key persistence requires a non-empty configured profile name",
            )));
        }

        let normalized_api_key = api_key.trim();
        if normalized_api_key.is_empty() {
            return Err(ConfigError::Validation(String::from(
                "remote API key persistence requires a non-empty API key value",
            )));
        }

        let mut document = if path.exists() {
            load_document_table_from_path(path)?
        } else {
            load_document_table_from_str(Self::default_template())?
        };

        let remote_profiles_value = document
            .entry(String::from("remote_profiles"))
            .or_insert_with(|| toml::Value::Table(toml::Table::new()));
        let Some(remote_profiles_table) = remote_profiles_value.as_table_mut() else {
            return Err(ConfigError::Validation(String::from(
                "remote_profiles must remain a TOML table",
            )));
        };

        let Some(profile_value) = remote_profiles_table.get_mut(normalized_profile_name) else {
            return Err(ConfigError::Validation(format!(
                "remote_profiles.{normalized_profile_name} is not configured"
            )));
        };
        let Some(profile_table) = profile_value.as_table_mut() else {
            return Err(ConfigError::Validation(format!(
                "remote_profiles.{normalized_profile_name} must remain a TOML table"
            )));
        };
        let base_url = profile_table
            .get("base_url")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| {
                ConfigError::Validation(format!(
                    "remote_profiles.{normalized_profile_name}.base_url must be configured before storing a credential"
                ))
            })?;
        let endpoint_scope =
            ProviderEndpointScope::parse(base_url).map_err(ConfigError::Validation)?;
        let keyring_ref =
            keyring_ref_for_remote_api_key(provider_kind, normalized_profile_name, &endpoint_scope)
                .map_err(ConfigError::Keyring)?;

        set_keyring_secret(
            &keyring_ref.service,
            &keyring_ref.account,
            normalized_api_key,
        )
        .map_err(ConfigError::Keyring)?;

        profile_table.insert(
            String::from("api_key"),
            toml::Value::try_from(SecretRef::FromKeyring {
                from_keyring: keyring_ref,
            })?,
        );

        let serialized = toml::to_string_pretty(&document)?;
        write_config_atomic(path, &serialized)?;

        Self::load_from_path(path)
    }

    pub fn persist_remote_planner_connection_settings_at_path(
        path: impl AsRef<Path>,
        profile_name: &str,
        base_url: &str,
        model: &str,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let normalized_profile_name = profile_name.trim();
        if normalized_profile_name.is_empty() {
            return Err(ConfigError::Validation(String::from(
                "remote planner settings persistence requires a non-empty configured profile name",
            )));
        }

        let normalized_base_url = normalize_remote_endpoint(base_url)?;
        let normalized_model = model.trim();
        if normalized_model.is_empty() {
            return Err(ConfigError::Validation(String::from(
                "remote planner settings persistence requires a non-empty model",
            )));
        }

        let mut document = if path.exists() {
            load_document_table_from_path(path)?
        } else {
            load_document_table_from_str(Self::default_template())?
        };

        let remote_profiles_value = document
            .entry(String::from("remote_profiles"))
            .or_insert_with(|| toml::Value::Table(toml::Table::new()));
        let Some(remote_profiles_table) = remote_profiles_value.as_table_mut() else {
            return Err(ConfigError::Validation(String::from(
                "remote_profiles must remain a TOML table",
            )));
        };

        let Some(profile_value) = remote_profiles_table.get_mut(normalized_profile_name) else {
            return Err(ConfigError::Validation(format!(
                "remote_profiles.{normalized_profile_name} is not configured"
            )));
        };
        let Some(profile_table) = profile_value.as_table_mut() else {
            return Err(ConfigError::Validation(format!(
                "remote_profiles.{normalized_profile_name} must remain a TOML table"
            )));
        };

        profile_table.insert(
            String::from("base_url"),
            toml::Value::String(normalized_base_url),
        );
        profile_table.insert(
            String::from("model"),
            toml::Value::String(String::from(normalized_model)),
        );

        let serialized = toml::to_string_pretty(&document)?;
        write_config_atomic(path, &serialized)?;

        Self::load_from_path(path)
    }

    pub fn reset_remote_planner_connection_settings_to_defaults_at_path(
        path: impl AsRef<Path>,
        profile_name: &str,
    ) -> Result<Self, ConfigError> {
        let normalized_profile_name = profile_name.trim();
        if normalized_profile_name.is_empty() {
            return Err(ConfigError::Validation(String::from(
                "remote planner defaults reset requires a non-empty configured profile name",
            )));
        }

        let default_config = Self::load_from_str(Self::default_template())?;
        let Some(default_profile) = default_config
            .remote_planner_profiles
            .get(normalized_profile_name)
        else {
            return Err(ConfigError::Validation(format!(
                "remote planner defaults are not defined for profile '{normalized_profile_name}'"
            )));
        };

        Self::persist_remote_planner_connection_settings_at_path(
            path,
            normalized_profile_name,
            &default_profile.base_url,
            &default_profile.model,
        )
    }

    pub fn persist_local_model_path_at_path(
        path: impl AsRef<Path>,
        profile_name: &str,
        model_path: &str,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let normalized_profile_name = profile_name.trim();
        if normalized_profile_name.is_empty() {
            return Err(ConfigError::Validation(String::from(
                "local model path persistence requires a non-empty configured profile name",
            )));
        }

        let normalized_model_path = model_path.trim();
        if normalized_model_path.is_empty() {
            return Err(ConfigError::Validation(String::from(
                "local model path persistence requires a non-empty model path",
            )));
        }

        let mut document = if path.exists() {
            load_document_table_from_path(path)?
        } else {
            load_document_table_from_str(Self::default_template())?
        };

        let local_profiles_value = document
            .entry(String::from("local_profiles"))
            .or_insert_with(|| toml::Value::Table(toml::Table::new()));
        let Some(local_profiles_table) = local_profiles_value.as_table_mut() else {
            return Err(ConfigError::Validation(String::from(
                "local_profiles must remain a TOML table",
            )));
        };

        let Some(profile_value) = local_profiles_table.get_mut(normalized_profile_name) else {
            return Err(ConfigError::Validation(format!(
                "local_profiles.{normalized_profile_name} is not configured"
            )));
        };
        let Some(profile_table) = profile_value.as_table_mut() else {
            return Err(ConfigError::Validation(format!(
                "local_profiles.{normalized_profile_name} must remain a TOML table"
            )));
        };
        profile_table.insert(
            String::from("model_path"),
            toml::Value::String(normalized_model_path.to_string()),
        );

        let serialized = toml::to_string_pretty(&document)?;
        write_config_atomic(path, &serialized)?;

        Self::load_from_path(path)
    }

    pub fn persist_tts_provider_selection_for_app(
        app_handle: &AppHandle,
        selection: &ProviderSelection,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_tts_provider_selection_at_path(&config_path, selection)
    }

    pub fn persist_asr_provider_selection_for_app(
        app_handle: &AppHandle,
        selection: &ProviderSelection,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_asr_provider_selection_at_path(&config_path, selection)
    }

    pub fn persist_asr_provider_selection_at_path(
        path: impl AsRef<Path>,
        selection: &ProviderSelection,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();

        let mut document = if path.exists() {
            load_document_table_from_path(path)?
        } else {
            load_document_table_from_str(Self::default_template())?
        };

        let providers_value = document
            .entry(String::from("providers"))
            .or_insert_with(|| toml::Value::Table(toml::Table::new()));
        let Some(providers_table) = providers_value.as_table_mut() else {
            return Err(ConfigError::Validation(String::from(
                "providers must remain a TOML table",
            )));
        };
        providers_table.insert(
            String::from("asr"),
            toml::Value::try_from(selection.clone())?,
        );

        let serialized = toml::to_string_pretty(&document)?;
        write_config_atomic(path, &serialized)?;

        Self::load_from_path(path)
    }

    pub fn persist_tts_provider_selection_at_path(
        path: impl AsRef<Path>,
        selection: &ProviderSelection,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();

        let mut document = if path.exists() {
            load_document_table_from_path(path)?
        } else {
            load_document_table_from_str(Self::default_template())?
        };

        let providers_value = document
            .entry(String::from("providers"))
            .or_insert_with(|| toml::Value::Table(toml::Table::new()));
        let Some(providers_table) = providers_value.as_table_mut() else {
            return Err(ConfigError::Validation(String::from(
                "providers must remain a TOML table",
            )));
        };
        providers_table.insert(
            String::from("tts"),
            toml::Value::try_from(selection.clone())?,
        );

        let serialized = toml::to_string_pretty(&document)?;
        write_config_atomic(path, &serialized)?;

        Self::load_from_path(path)
    }
}

#[cfg(test)]
mod tests {
    use super::write_config_atomic;

    #[test]
    fn write_config_atomic_writes_expected_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        write_config_atomic(&path, "value = 1\n").unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "value = 1\n");
    }

    #[test]
    fn write_config_atomic_creates_parent_directory() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("config.toml");

        write_config_atomic(&path, "x = 2\n").unwrap();

        assert!(path.exists());
    }

    #[test]
    fn write_config_atomic_does_not_leave_tmp_file_on_success() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        write_config_atomic(&path, "a = 1\n").unwrap();

        let tmp = dir.path().join("config.toml.tmp");
        assert!(
            !tmp.exists(),
            "temp file must not remain after successful write"
        );
    }

    #[test]
    fn atomic_config_write_replaces_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        write_config_atomic(&path, "value = 1\n").unwrap();
        write_config_atomic(&path, "value = 2\n").unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "value = 2\n");
    }
}

#[cfg(test)]
mod post_batch8_cleanup_tests {
    use super::*;

    #[test]
    fn failed_config_temp_cleanup_is_explicit() {
        let synthetic_path = Path::new("synthetic-config.toml.tmp");
        let failure = remove_failed_config_temp_file_with(synthetic_path, |_| {
            Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "synthetic cleanup refusal",
            ))
        })
        .expect_err("cleanup refusal must remain visible");
        assert_eq!(failure.kind(), std::io::ErrorKind::PermissionDenied);

        remove_failed_config_temp_file_with(synthetic_path, |_| {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "already absent",
            ))
        })
        .expect("missing temporary file is already clean");
    }
}
