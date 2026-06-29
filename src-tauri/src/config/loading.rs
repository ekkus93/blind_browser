use std::collections::BTreeMap;
use std::path::Path;

use serde::de::DeserializeOwned;

use super::*;

pub(in crate::config) fn load_document_table_from_path(
    path: &Path,
) -> Result<toml::Table, ConfigError> {
    let contents = fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path.to_path_buf(),
        source,
    })?;

    load_document_table_from_str(&contents)
}

pub(in crate::config) fn load_document_table_from_str(
    contents: &str,
) -> Result<toml::Table, ConfigError> {
    toml::from_str(contents).map_err(ConfigError::Parse)
}

pub(in crate::config) fn load_planner_profiles(
    selection: &ProviderSelection,
    remote_profiles: &BTreeMap<String, toml::Table>,
    resolved_remote_profiles: &mut BTreeMap<String, RemotePlannerProfile>,
    issues: &mut Vec<String>,
) {
    if selection.mode != ProviderMode::Remote {
        issues.push(String::from(
            "providers.planner.mode must be \"remote\"; local planner support has been removed",
        ));
    }

    if selection.local_profile.is_some() {
        issues.push(String::from(
            "providers.planner.local_profile is not supported; use a remote planner profile such as openai-default or ollama-default",
        ));
    }

    if selection.failover_to_local.is_some() {
        issues.push(String::from(
            "providers.planner.failover_to_local is not supported; use the selected remote planner directly",
        ));
    }

    if let Some(profile_name) = selection.remote_profile.as_deref() {
        if let Some(profile) = resolve_profile::<RemotePlannerProfile>(
            "remote_profiles",
            "planner",
            profile_name,
            remote_profiles,
            issues,
        ) {
            resolved_remote_profiles.insert(profile_name.to_owned(), profile);
        }
    } else {
        issues.push(String::from(
            "providers.planner.remote_profile is required when mode = \"remote\"",
        ));
    }
}

pub(in crate::config) fn load_provider_profiles<RemoteProfile, LocalProfile>(
    category: &str,
    selection: &ProviderSelection,
    remote_profiles: &BTreeMap<String, toml::Table>,
    local_profiles: &BTreeMap<String, toml::Table>,
    resolved_remote_profiles: &mut BTreeMap<String, RemoteProfile>,
    resolved_local_profiles: &mut BTreeMap<String, LocalProfile>,
    issues: &mut Vec<String>,
) where
    RemoteProfile: DeserializeOwned,
    LocalProfile: DeserializeOwned,
{
    if let Some(profile_name) = selection.remote_profile.as_deref() {
        if let Some(profile) = resolve_profile::<RemoteProfile>(
            "remote_profiles",
            category,
            profile_name,
            remote_profiles,
            issues,
        ) {
            resolved_remote_profiles.insert(profile_name.to_owned(), profile);
        }
    } else if selection.mode == ProviderMode::Remote {
        issues.push(format!(
            "providers.{category}.remote_profile is required when mode = \"remote\""
        ));
    }

    if let Some(profile_name) = selection.local_profile.as_deref() {
        if let Some(profile) = resolve_profile::<LocalProfile>(
            "local_profiles",
            category,
            profile_name,
            local_profiles,
            issues,
        ) {
            resolved_local_profiles.insert(profile_name.to_owned(), profile);
        }
    } else if selection.mode == ProviderMode::Local {
        issues.push(format!(
            "providers.{category}.local_profile is required when mode = \"local\""
        ));
    }
}

pub(in crate::config) fn resolve_profile<T>(
    profile_group: &str,
    category: &str,
    profile_name: &str,
    profiles: &BTreeMap<String, toml::Table>,
    issues: &mut Vec<String>,
) -> Option<T>
where
    T: DeserializeOwned,
{
    let Some(table) = profiles.get(profile_name) else {
        issues.push(format!(
            "providers.{category} references missing {profile_group}.{profile_name}"
        ));
        return None;
    };

    match toml::Value::Table(table.clone()).try_into::<T>() {
        Ok(profile) => Some(profile),
        Err(error) => {
            issues.push(format!(
                "{profile_group}.{profile_name} is not valid for {category}: {error}"
            ));
            None
        }
    }
}
