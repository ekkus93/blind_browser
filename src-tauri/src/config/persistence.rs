use std::fs;
use std::path::Path;

use tauri::AppHandle;

use crate::ocr::OcrSettings;
use super::{
    AppConfig, AudioSettings, ConfigError, ModelManagementSettings, ProviderSelection,
    SafetySettings, SecretRef,
};
use super::keyring_store::{keyring_ref_for_remote_api_key, set_keyring_secret};
use super::loading::{load_document_table_from_path, load_document_table_from_str};
use super::validation::{
    normalize_remote_endpoint, validate_audio_settings, validate_model_settings,
};

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
        Self::reset_remote_planner_connection_settings_to_defaults_at_path(&config_path, profile_name)
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

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let serialized = toml::to_string_pretty(&document)?;
        fs::write(path, serialized).map_err(|source| ConfigError::Write {
            path: path.to_path_buf(),
            source,
        })?;

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

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let serialized = toml::to_string_pretty(&document)?;
        fs::write(path, serialized).map_err(|source| ConfigError::Write {
            path: path.to_path_buf(),
            source,
        })?;

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

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let serialized = toml::to_string_pretty(&document)?;
        fs::write(path, serialized).map_err(|source| ConfigError::Write {
            path: path.to_path_buf(),
            source,
        })?;

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

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let serialized = toml::to_string_pretty(&document)?;
        fs::write(path, serialized).map_err(|source| ConfigError::Write {
            path: path.to_path_buf(),
            source,
        })?;

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

        let keyring_ref = keyring_ref_for_remote_api_key(provider_kind, normalized_profile_name);
        set_keyring_secret(
            &keyring_ref.service,
            &keyring_ref.account,
            normalized_api_key,
        )
        .map_err(ConfigError::Keyring)?;

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
            String::from("api_key"),
            toml::Value::try_from(SecretRef::FromKeyring {
                from_keyring: keyring_ref,
            })?,
        );

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let serialized = toml::to_string_pretty(&document)?;
        fs::write(path, serialized).map_err(|source| ConfigError::Write {
            path: path.to_path_buf(),
            source,
        })?;

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

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let serialized = toml::to_string_pretty(&document)?;
        fs::write(path, serialized).map_err(|source| ConfigError::Write {
            path: path.to_path_buf(),
            source,
        })?;

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
        let Some(default_profile) = default_config.remote_planner_profiles.get(normalized_profile_name) else {
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

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let serialized = toml::to_string_pretty(&document)?;
        fs::write(path, serialized).map_err(|source| ConfigError::Write {
            path: path.to_path_buf(),
            source,
        })?;

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

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let serialized = toml::to_string_pretty(&document)?;
        fs::write(path, serialized).map_err(|source| ConfigError::Write {
            path: path.to_path_buf(),
            source,
        })?;

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

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let serialized = toml::to_string_pretty(&document)?;
        fs::write(path, serialized).map_err(|source| ConfigError::Write {
            path: path.to_path_buf(),
            source,
        })?;

        Self::load_from_path(path)
    }
}
