use std::fs;
use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::Path;

use tauri::AppHandle;

use super::keyring_store::{keyring_ref_for_remote_api_key, set_keyring_secret};
use super::loading::{load_document_table_from_path, load_document_table_from_str};
use super::validation::{
    normalize_remote_endpoint, normalize_remote_planner_privacy_settings, validate_audio_settings,
    validate_model_settings, validate_ocr_settings, validate_safety_settings,
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
    // config.toml is not secret-bearing (the keyring holds actual secrets),
    // but it holds remote_planner_privacy.origin_rules: a durable,
    // timestamped record of which sites the user visited and what they
    // consented to send off-device. Restrict the directory the same way
    // image_cache.rs already does for its own (also privacy-sensitive)
    // files.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700)).map_err(|source| {
            ConfigError::Write {
                path: parent.to_path_buf(),
                source,
            }
        })?;
    }

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
        // Mode must be set at creation (an OpenOptions flag), not via a
        // set_permissions call after the file exists -- setting it after
        // creation leaves a TOCTOU window where the file is briefly
        // world/group-readable at the umask default before the mode change
        // lands. fs::rename preserves the source file's mode, so the
        // renamed config.toml inherits 0600 from this temp file with no
        // separate step needed.
        let mut open_options = OpenOptions::new();
        open_options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            open_options.mode(0o600);
        }
        let mut file = open_options
            .open(&tmp_path)
            .map_err(|source| ConfigError::Write {
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

/// Get-or-create the top-level `[key]` table in `document`, failing if that
/// key already exists but isn't a table.
fn table_mut<'a>(
    document: &'a mut toml::Table,
    key: &str,
) -> Result<&'a mut toml::Table, ConfigError> {
    let value = document
        .entry(String::from(key))
        .or_insert_with(|| toml::Value::Table(toml::Table::new()));
    value
        .as_table_mut()
        .ok_or_else(|| ConfigError::Validation(format!("{key} must remain a TOML table")))
}

/// Navigate into `document.<profiles_key>.<profile_name>`. Unlike
/// [`table_mut`]'s outer `profiles_key` table, an individual profile is
/// never created here -- persisting a field onto a profile that was never
/// configured is a validation error, not an implicit profile creation.
fn profile_table_mut<'a>(
    document: &'a mut toml::Table,
    profiles_key: &str,
    profile_name: &str,
) -> Result<&'a mut toml::Table, ConfigError> {
    let profiles_table = table_mut(document, profiles_key)?;
    let Some(profile_value) = profiles_table.get_mut(profile_name) else {
        return Err(ConfigError::Validation(format!(
            "{profiles_key}.{profile_name} is not configured"
        )));
    };
    profile_value.as_table_mut().ok_or_else(|| {
        ConfigError::Validation(format!(
            "{profiles_key}.{profile_name} must remain a TOML table"
        ))
    })
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

    pub fn persist_remote_narration_privacy_settings_for_app(
        app_handle: &AppHandle,
        settings: &RemotePlannerPrivacySettings,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_remote_narration_privacy_settings_at_path(&config_path, settings)
    }

    pub fn persist_remote_microphone_privacy_settings_for_app(
        app_handle: &AppHandle,
        settings: &RemotePlannerPrivacySettings,
    ) -> Result<Self, ConfigError> {
        let config_path = Self::config_path_for_app(app_handle)?;
        Self::persist_remote_microphone_privacy_settings_at_path(&config_path, settings)
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

    /// Shared "load the active document (or the default template if none
    /// exists yet), let the caller validate/mutate it, serialize, write
    /// atomically, reload" skeleton. CR3 P2.8.2: every `persist_*_at_path`
    /// function below used to repeat this load/write/reload bookkeeping by
    /// hand -- ten near-identical copies, with four different validation
    /// postures (whole-struct via an `issues` vec, per-field trim/non-empty
    /// checks, a keyring side effect before the write, and no validation at
    /// all for provider selection). Collapsing the bookkeeping here, while
    /// leaving each function's own validation and section-specific mutation
    /// in its own `mutate` closure, means a copy/paste error in the shared
    /// part (e.g. forgetting the atomic write, or reloading from the wrong
    /// path) can no longer diverge between persisters -- there is only one
    /// copy of it left. Each persister still owns entirely its own
    /// validation, called *before* this so a validation failure never even
    /// touches the document (matching the pre-refactor behavior the
    /// `..._without_touching_the_file` tests below assert).
    fn mutate_config_document(
        path: &Path,
        mutate: impl FnOnce(&mut toml::Table) -> Result<(), ConfigError>,
    ) -> Result<Self, ConfigError> {
        let mut document = if path.exists() {
            load_document_table_from_path(path)?
        } else {
            load_document_table_from_str(Self::default_template())?
        };
        mutate(&mut document)?;
        let serialized = toml::to_string_pretty(&document)?;
        write_config_atomic(path, &serialized)?;
        Self::load_from_path(path)
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
        Self::mutate_config_document(path, |document| {
            document.insert(String::from("audio"), toml::Value::try_from(audio.clone())?);
            Ok(())
        })
    }

    pub fn persist_safety_settings_at_path(
        path: impl AsRef<Path>,
        safety: &SafetySettings,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let mut issues = Vec::new();
        validate_safety_settings(safety, &mut issues);
        if !issues.is_empty() {
            return Err(ConfigError::Validation(issues.join("\n")));
        }
        Self::mutate_config_document(path, |document| {
            document.insert(
                String::from("safety"),
                toml::Value::try_from(safety.clone())?,
            );
            Ok(())
        })
    }

    pub fn persist_ocr_settings_at_path(
        path: impl AsRef<Path>,
        ocr: &OcrSettings,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let mut issues = Vec::new();
        validate_ocr_settings(ocr, &mut issues);
        if !issues.is_empty() {
            return Err(ConfigError::Validation(issues.join("\n")));
        }
        Self::mutate_config_document(path, |document| {
            document.insert(String::from("ocr"), toml::Value::try_from(ocr.clone())?);
            Ok(())
        })
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
        Self::mutate_config_document(path, |document| {
            document.insert(
                String::from("models"),
                toml::Value::try_from(models.clone())?,
            );
            Ok(())
        })
    }

    pub fn persist_remote_planner_privacy_settings_at_path(
        path: impl AsRef<Path>,
        settings: &RemotePlannerPrivacySettings,
    ) -> Result<Self, ConfigError> {
        Self::persist_remote_data_privacy_settings_at_path(path, "remote_planner_privacy", settings)
    }

    /// Narration and microphone-audio disclosure use the exact same policy
    /// shape and TOML persistence as the planner (see
    /// `remote_data_consent::evaluate_remote_planner_policy`, which is reused
    /// unchanged for all three) -- only the section key differs, since each
    /// disclosure kind keeps its own independent origin-rules/network-mode
    /// instance (see the field doc comment on `AppConfig::remote_narration_privacy`).
    pub fn persist_remote_narration_privacy_settings_at_path(
        path: impl AsRef<Path>,
        settings: &RemotePlannerPrivacySettings,
    ) -> Result<Self, ConfigError> {
        Self::persist_remote_data_privacy_settings_at_path(
            path,
            "remote_narration_privacy",
            settings,
        )
    }

    pub fn persist_remote_microphone_privacy_settings_at_path(
        path: impl AsRef<Path>,
        settings: &RemotePlannerPrivacySettings,
    ) -> Result<Self, ConfigError> {
        Self::persist_remote_data_privacy_settings_at_path(
            path,
            "remote_microphone_privacy",
            settings,
        )
    }

    fn persist_remote_data_privacy_settings_at_path(
        path: impl AsRef<Path>,
        toml_key: &str,
        settings: &RemotePlannerPrivacySettings,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let mut normalized = settings.clone();
        let mut issues = Vec::new();
        normalize_remote_planner_privacy_settings(&mut normalized, &mut issues);
        if !issues.is_empty() {
            return Err(ConfigError::Validation(issues.join("\n")));
        }
        Self::mutate_config_document(path, |document| {
            document.insert(String::from(toml_key), toml::Value::try_from(normalized)?);
            Ok(())
        })
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

        Self::mutate_config_document(path, |document| {
            let profile_table =
                profile_table_mut(document, "remote_profiles", normalized_profile_name)?;
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
            let keyring_ref = keyring_ref_for_remote_api_key(
                provider_kind,
                normalized_profile_name,
                &endpoint_scope,
            )
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
            Ok(())
        })
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

        Self::mutate_config_document(path, |document| {
            let profile_table =
                profile_table_mut(document, "remote_profiles", normalized_profile_name)?;
            profile_table.insert(
                String::from("base_url"),
                toml::Value::String(normalized_base_url),
            );
            profile_table.insert(
                String::from("model"),
                toml::Value::String(String::from(normalized_model)),
            );
            Ok(())
        })
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
        let normalized_model_path = normalized_model_path.to_string();

        Self::mutate_config_document(path, |document| {
            let profile_table =
                profile_table_mut(document, "local_profiles", normalized_profile_name)?;
            profile_table.insert(
                String::from("model_path"),
                toml::Value::String(normalized_model_path),
            );
            Ok(())
        })
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
        Self::mutate_config_document(path, |document| {
            let providers_table = table_mut(document, "providers")?;
            providers_table.insert(
                String::from("asr"),
                toml::Value::try_from(selection.clone())?,
            );
            Ok(())
        })
    }

    pub fn persist_tts_provider_selection_at_path(
        path: impl AsRef<Path>,
        selection: &ProviderSelection,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        Self::mutate_config_document(path, |document| {
            let providers_table = table_mut(document, "providers")?;
            providers_table.insert(
                String::from("tts"),
                toml::Value::try_from(selection.clone())?,
            );
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        profile_table_mut, table_mut, write_config_atomic, AppConfig, ConfigError, SafetySettings,
    };

    // CR3 P2.8.2: `table_mut`/`profile_table_mut` are the two navigation
    // helpers every `persist_*_at_path` function now shares instead of
    // repeating its own `document.entry(...).or_insert_with(...)`/
    // `as_table_mut()` boilerplate by hand. Pin their behavior and exact
    // error wording directly, since after the refactor a mistake here would
    // affect every persister that uses them, not just one.
    #[test]
    fn table_mut_creates_a_missing_table() {
        let mut document = toml::Table::new();
        table_mut(&mut document, "providers").unwrap();
        assert!(document.get("providers").unwrap().is_table());
    }

    #[test]
    fn table_mut_rejects_a_non_table_value() {
        let mut document = toml::Table::new();
        document.insert(
            String::from("providers"),
            toml::Value::String(String::from("not a table")),
        );
        let error = table_mut(&mut document, "providers").unwrap_err();
        match error {
            ConfigError::Validation(message) => {
                assert_eq!(message, "providers must remain a TOML table");
            }
            other => panic!("expected ConfigError::Validation, got {other:?}"),
        }
    }

    #[test]
    fn profile_table_mut_rejects_an_unconfigured_profile() {
        let mut document = toml::Table::new();
        let error = profile_table_mut(&mut document, "remote_profiles", "ghost").unwrap_err();
        match error {
            ConfigError::Validation(message) => {
                assert_eq!(message, "remote_profiles.ghost is not configured");
            }
            other => panic!("expected ConfigError::Validation, got {other:?}"),
        }
    }

    #[test]
    fn profile_table_mut_does_not_implicitly_create_a_profile() {
        let mut document = toml::Table::new();
        // Even though `table_mut` (used internally) would create a missing
        // `remote_profiles` table, `profile_table_mut` must still refuse to
        // fabricate the *profile itself* -- persisting a field onto a
        // profile that was never configured is a validation error, not an
        // implicit profile creation.
        assert!(profile_table_mut(&mut document, "remote_profiles", "ghost").is_err());
        assert!(document
            .get("remote_profiles")
            .and_then(toml::Value::as_table)
            .is_some_and(|table| !table.contains_key("ghost")));
    }

    #[test]
    fn profile_table_mut_resolves_an_existing_profile() {
        let mut document = toml::Table::new();
        let mut profiles = toml::Table::new();
        profiles.insert(String::from("mine"), toml::Value::Table(toml::Table::new()));
        document.insert(
            String::from("remote_profiles"),
            toml::Value::Table(profiles),
        );

        let profile_table = profile_table_mut(&mut document, "remote_profiles", "mine").unwrap();
        profile_table.insert(
            String::from("model"),
            toml::Value::String(String::from("gpt")),
        );

        assert_eq!(
            document["remote_profiles"]["mine"]["model"].as_str(),
            Some("gpt")
        );
    }

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

    // CR3 P2.3: config.toml holds remote_planner_privacy.origin_rules -- a
    // durable, timestamped record of which sites the user visited and what
    // they consented to send off-device. It is not secret-bearing (the
    // keyring handles secrets), but on a shared machine any local user could
    // previously read it at the umask default (measured 0644). Mirrors
    // image_cache.rs's existing 0700 dir / 0600 file convention for its own
    // privacy-sensitive files.
    #[cfg(unix)]
    #[test]
    fn write_config_atomic_restricts_file_and_directory_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("nested-config-dir");
        let path = dir.join("config.toml");

        write_config_atomic(&path, "value = 1\n").unwrap();

        let file_mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(
            file_mode, 0o600,
            "config.toml must be created 0600, not left at the umask default"
        );
        let dir_mode = std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777;
        assert_eq!(
            dir_mode, 0o700,
            "the config directory must be restricted to 0700"
        );
    }

    #[test]
    fn persist_safety_settings_rejects_an_invalid_threshold_without_touching_the_file() {
        // Regression test: previously this validated only on the *next* load,
        // after the invalid value was already written to disk -- which could
        // brick the next app launch. It must now fail before any write.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let invalid_safety = SafetySettings {
            confirmation_confidence_threshold: 5.0,
            allow_click_without_confirmation: false,
            always_confirm_submit: true,
        };

        let result = AppConfig::persist_safety_settings_at_path(&path, &invalid_safety);

        assert!(
            matches!(result, Err(ConfigError::Validation(_))),
            "expected a validation error, got {result:?}"
        );
        assert!(
            !path.exists(),
            "an invalid safety threshold must not be written to disk"
        );
    }

    #[test]
    fn persist_ocr_settings_rejects_a_zero_threshold_without_touching_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let invalid_ocr = crate::ocr::OcrSettings {
            trigger_on_no_extractable_text: true,
            sparse_text_char_threshold: 0,
            sparse_text_region_threshold: 0,
            prefer_region_ocr: true,
        };

        let result = AppConfig::persist_ocr_settings_at_path(&path, &invalid_ocr);

        assert!(
            matches!(result, Err(ConfigError::Validation(_))),
            "expected a validation error, got {result:?}"
        );
        assert!(
            !path.exists(),
            "an invalid OCR threshold must not be written to disk"
        );
    }

    #[test]
    fn persist_safety_settings_leaves_an_existing_file_untouched_on_validation_failure() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let original_contents = "[safety]\nconfirmation_confidence_threshold = 0.9\nallow_click_without_confirmation = false\nalways_confirm_submit = true\n";
        std::fs::write(&path, original_contents).unwrap();

        let invalid_safety = SafetySettings {
            confirmation_confidence_threshold: -1.0,
            allow_click_without_confirmation: false,
            always_confirm_submit: true,
        };

        let result = AppConfig::persist_safety_settings_at_path(&path, &invalid_safety);

        assert!(result.is_err());
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            original_contents,
            "a rejected write must not modify an existing config file"
        );
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
