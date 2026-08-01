use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use zeroize::Zeroizing;

use super::*;
use crate::provider_endpoint::ProviderEndpointScope;

const KEYRING_SERVICE: &str = "blind_browser";

/// Session cache of resolved keyring secrets, keyed by `(service, account)`.
/// Values are `Zeroizing` so they are scrubbed on replacement and on drop.
type SessionKeyringStore = BTreeMap<(String, String), Zeroizing<String>>;

pub fn keyring_ref_for_remote_api_key(
    provider_kind: &str,
    profile_name: &str,
    endpoint: &ProviderEndpointScope,
) -> Result<KeyringRef, String> {
    let provider_kind = normalized_identity_component("provider kind", provider_kind)?;
    let profile_name = normalized_identity_component("profile name", profile_name)?;
    Ok(KeyringRef {
        service: String::from(KEYRING_SERVICE),
        account: format!(
            "remote_{provider_kind}:{profile_name}:scope:{}:api_key",
            endpoint.scope_id()
        ),
    })
}

pub fn resolve_secret_ref_for_endpoint(
    secret_ref: &SecretRef,
    provider_kind: &str,
    profile_name: &str,
    endpoint: &ProviderEndpointScope,
) -> Result<String, String> {
    if let SecretRef::FromKeyring { from_keyring } = secret_ref {
        let expected = keyring_ref_for_remote_api_key(provider_kind, profile_name, endpoint)?;
        if *from_keyring != expected {
            let legacy_account = format!(
                "remote_{}:{}:api_key",
                provider_kind.trim(),
                profile_name.trim()
            );
            if from_keyring.service == KEYRING_SERVICE && from_keyring.account == legacy_account {
                return Err(format!(
                    "the configured credential is a legacy unbound keyring entry; re-enter it to bind it to {}",
                    endpoint.normalized_base_url()
                ));
            }
            return Err(format!(
                "the configured credential is not authorized for provider profile '{}' at {}",
                profile_name.trim(),
                endpoint.normalized_base_url()
            ));
        }
    }

    resolve_secret_ref(secret_ref)
}

fn normalized_identity_component(label: &str, value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("keyring {label} must not be empty"));
    }
    if value.contains(':') || value.chars().any(char::is_control) {
        return Err(format!(
            "keyring {label} must not contain colons or control characters"
        ));
    }
    Ok(value.to_string())
}

pub fn secret_ref_reference(secret_ref: &SecretRef) -> String {
    match secret_ref {
        SecretRef::FromEnv { from_env } => format!("Environment variable: {from_env}"),
        SecretRef::FromFile { from_file } => format!("File reference: {from_file}"),
        SecretRef::FromKeyring { from_keyring } => {
            format!(
                "OS keyring entry: {} / {}",
                from_keyring.service, from_keyring.account
            )
        }
    }
}

pub fn resolve_secret_ref(secret_ref: &SecretRef) -> Result<String, String> {
    match secret_ref {
        SecretRef::FromEnv { from_env } => std::env::var(from_env)
            .map(|value| value.trim().to_string())
            .map_err(|error| format!("failed to read environment variable '{from_env}': {error}")),
        SecretRef::FromFile { from_file } => fs::read_to_string(from_file)
            .map(|value| value.trim().to_string())
            .map_err(|error| format!("failed to read secret file '{from_file}': {error}")),
        SecretRef::FromKeyring { from_keyring } => {
            get_keyring_secret(&from_keyring.service, &from_keyring.account)
        }
    }
    .and_then(|value| {
        if value.is_empty() {
            Err(String::from("resolved secret value was empty"))
        } else {
            Ok(value)
        }
    })
}

fn cache_keyring_secret(service: &str, account: &str, secret: &str) -> Result<(), String> {
    let store = session_keyring_store();
    let mut store = store
        .lock()
        .map_err(|_| String::from("failed to acquire the session keyring cache lock"))?;
    let key = (String::from(service), String::from(account));
    // `Zeroizing` scrubs the prior cached value on replacement and scrubs every
    // entry when the session store is dropped, so a resolved secret never lingers
    // in the process heap after it is no longer cached.
    store.insert(key, Zeroizing::new(String::from(secret)));
    Ok(())
}

fn cached_keyring_secret(service: &str, account: &str) -> Result<Option<String>, String> {
    let store = session_keyring_store();
    let store = store
        .lock()
        .map_err(|_| String::from("failed to acquire the session keyring cache lock"))?;
    Ok(store
        .get(&(String::from(service), String::from(account)))
        .map(|cached| cached.as_str().to_string()))
}

fn session_keyring_store() -> &'static Mutex<SessionKeyringStore> {
    static STORE: OnceLock<Mutex<SessionKeyringStore>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(SessionKeyringStore::new()))
}

#[cfg(not(test))]
pub(in crate::config) fn set_keyring_secret(
    service: &str,
    account: &str,
    secret: &str,
) -> Result<(), String> {
    let entry = keyring::Entry::new(service, account)
        .map_err(|error| format!("failed to open keyring entry '{service}/{account}': {error}"))?;
    entry.set_password(secret).map_err(|error| {
        format!("failed to store keyring secret '{service}/{account}': {error}")
    })?;

    cache_keyring_secret(service, account, secret)
}

#[cfg(not(test))]
pub(in crate::config) fn get_keyring_secret(
    service: &str,
    account: &str,
) -> Result<String, String> {
    if let Some(cached_secret) = cached_keyring_secret(service, account)? {
        return Ok(cached_secret);
    }

    let entry = keyring::Entry::new(service, account)
        .map_err(|error| format!("failed to open keyring entry '{service}/{account}': {error}"))?;
    let secret = entry
        .get_password()
        .map_err(|error| format!("failed to read keyring secret '{service}/{account}': {error}"))?;

    cache_keyring_secret(service, account, &secret)?;
    Ok(secret)
}

#[cfg(test)]
pub(in crate::config) fn set_keyring_secret(
    service: &str,
    account: &str,
    secret: &str,
) -> Result<(), String> {
    cache_keyring_secret(service, account, secret)
}

#[cfg(test)]
pub(in crate::config) fn get_keyring_secret(
    service: &str,
    account: &str,
) -> Result<String, String> {
    cached_keyring_secret(service, account)?
        .ok_or_else(|| format!("failed to read keyring secret '{service}/{account}': no entry"))
}
