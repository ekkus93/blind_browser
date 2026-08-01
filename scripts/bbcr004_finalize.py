from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def read(path: str) -> str:
    return (ROOT / path).read_text()

def write(path: str, content: str) -> None:
    (ROOT / path).write_text(content)

def replace_once(path: str, old: str, new: str) -> None:
    text = read(path)
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one exact match, found {count}")
    write(path, text.replace(old, new, 1))

def replace_regex(path: str, pattern: str, replacement: str, flags: int = re.S) -> None:
    text = read(path)
    updated, count = re.subn(pattern, replacement, text, count=1, flags=flags)
    if count != 1:
        raise SystemExit(f"{path}: expected one regex match, found {count}")
    write(path, updated)

replace_once(
    "src-tauri/src/config/mod.rs",
    "pub use keyring_store::{keyring_ref_for_remote_api_key, resolve_secret_ref, secret_ref_reference};",
    "pub use keyring_store::{\n    keyring_ref_for_remote_api_key, resolve_secret_ref, resolve_secret_ref_for_endpoint,\n    secret_ref_reference,\n};",
)

replace_once(
    "src-tauri/src/config/persistence.rs",
    "use crate::ocr::OcrSettings;",
    "use crate::ocr::OcrSettings;\nuse crate::provider_endpoint::ProviderEndpointScope;",
)

persistence_fn = r'''    pub fn persist_remote_api_key_at_path(
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
        let endpoint_scope = ProviderEndpointScope::parse(base_url)
            .map_err(ConfigError::Validation)?;
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

        let serialized = toml::to_string_pretty(&document)?;
        write_config_atomic(path, &serialized)?;

        Self::load_from_path(path)
    }

'''
replace_regex(
    "src-tauri/src/config/persistence.rs",
    r"    pub fn persist_remote_api_key_at_path\(.*?\n    pub fn persist_remote_planner_connection_settings_at_path\(",
    persistence_fn + "    pub fn persist_remote_planner_connection_settings_at_path(",
)

replace_once(
    "src-tauri/src/config/validation.rs",
    "use super::*;",
    "use super::*;\nuse crate::provider_endpoint::ProviderEndpointScope;",
)
replace_regex(
    "src-tauri/src/config/validation.rs",
    r"pub\(in crate::config\) fn normalize_remote_endpoint\(base_url: &str\) -> Result<String, ConfigError> \{.*?\n\}",
    r'''pub(in crate::config) fn normalize_remote_endpoint(base_url: &str) -> Result<String, ConfigError> {
    ProviderEndpointScope::parse(base_url)
        .map(|scope| scope.normalized_base_url().to_string())
        .map_err(ConfigError::Validation)
}''',
)

runtime = read("src-tauri/src/app_core/runtime_config.rs")
runtime = runtime.replace(
    '#[cfg(feature = "remote-openai")]\nuse crate::config::resolve_secret_ref;\nuse crate::config::{AppConfig, ConfigError, ModelManagementSettings};',
    '#[cfg(feature = "remote-openai")]\nuse crate::config::resolve_secret_ref_for_endpoint;\nuse crate::config::{AppConfig, ConfigError, ModelManagementSettings};\n#[cfg(feature = "remote-openai")]\nuse crate::provider_endpoint::ProviderEndpointScope;',
    1,
)
helper_pattern = r'''#\[cfg\(feature = "remote-openai"\)\]
fn resolve_optional_remote_planner_api_key\(.*?
\}

impl AppCore \{'''
helper_replacement = r'''#[cfg(feature = "remote-openai")]
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
        format!("Remote planner profile '{profile_name}' has an invalid configured endpoint: {reason}")
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

impl AppCore {'''
runtime, count = re.subn(helper_pattern, helper_replacement, runtime, count=1, flags=re.S)
if count != 1:
    raise SystemExit(f"runtime_config.rs: helper replacement count {count}")

list_method = r'''    pub fn list_remote_planner_models(
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

        let requested_scope = ProviderEndpointScope::parse(
            base_url_override.unwrap_or(&profile.base_url),
        )
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

'''
runtime, count = re.subn(
    r"    pub fn list_remote_planner_models\(.*?\n    pub fn test_remote_tts_api_key\(",
    list_method + "    pub fn test_remote_tts_api_key(",
    runtime,
    count=1,
    flags=re.S,
)
if count != 1:
    raise SystemExit(f"runtime_config.rs: list method replacement count {count}")

runtime_tests = r'''#[cfg(all(test, feature = "remote-openai"))]
mod tests {
    use super::{
        resolve_remote_planner_model_credentials, RemotePlannerModelCredentials,
    };
    use crate::config::{
        KeyringRef, RemotePlannerProfile, RemoteProviderKind, SecretRef,
    };
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
        let requested =
            ProviderEndpointScope::parse("https://other.example.com/v1").unwrap();
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
'''
runtime, count = re.subn(
    r'#\[cfg\(all\(test, feature = "remote-openai"\)\)\]\nmod tests \{.*\}\s*$',
    runtime_tests,
    runtime,
    count=1,
    flags=re.S,
)
if count != 1:
    raise SystemExit(f"runtime_config.rs: tests replacement count {count}")
write("src-tauri/src/app_core/runtime_config.rs", runtime)

replace_once(
    "src-tauri/src/command_handlers/api_key_handlers.rs",
    "use crate::commands::ToolError;\nuse crate::{join_error_to_tool_error, lock_app_core};",
    "use crate::commands::ToolError;\nuse crate::provider_endpoint::ProviderEndpointScope;\nuse crate::{join_error_to_tool_error, lock_app_core};",
)
handler = read("src-tauri/src/command_handlers/api_key_handlers.rs")
old = '''        let models = app_core
            .list_remote_planner_models(
                &profile_name,
                Some(&base_url),
                (!api_key.trim().is_empty()).then_some(api_key.as_str()),
                timeout_ms,
            )
'''
new = '''        let endpoint_scope = ProviderEndpointScope::parse(&base_url).map_err(|reason| ToolError {
            code: String::from("invalid_remote_planner_endpoint"),
            message: format!("Remote planner model endpoint is invalid: {reason}"),
            retryable: false,
            details: None,
        })?;
        let normalized_base_url = endpoint_scope.normalized_base_url().to_string();
        let models = app_core
            .list_remote_planner_models(
                &profile_name,
                Some(&normalized_base_url),
                (!api_key.trim().is_empty()).then_some(api_key.as_str()),
                timeout_ms,
            )
'''
if old not in handler:
    raise SystemExit("api_key_handlers.rs: model call block not found")
handler = handler.replace(old, new, 1)
handler = handler.replace(
    '''        Ok(RemotePlannerModelListData {
            profile_name,
            base_url,
            models,
        })''',
    '''        Ok(RemotePlannerModelListData {
            profile_name,
            base_url: normalized_base_url,
            models,
        })''',
    1,
)
write("src-tauri/src/command_handlers/api_key_handlers.rs", handler)

keyring_tests = read("src-tauri/src/config/tests/keyring_tests.rs")
keyring_tests = keyring_tests.replace(
    "use super::*;",
    "use super::*;\nuse crate::provider_endpoint::ProviderEndpointScope;",
    1,
)
keyring_tests = keyring_tests.replace(
    'let expected_keyring_ref = keyring_ref_for_remote_api_key("planner", "openai-default");',
    '''let endpoint =
        ProviderEndpointScope::parse("https://api.openai.com/v1").unwrap();
    let expected_keyring_ref =
        keyring_ref_for_remote_api_key("planner", "openai-default", &endpoint)
            .expect("bound keyring reference should build");''',
    1,
)
keyring_tests += r'''

#[test]
fn keyring_reference_changes_with_destination_scope() {
    let first = ProviderEndpointScope::parse("https://api.example.com/v1").unwrap();
    for changed in [
        "https://other.example.com/v1",
        "https://api.example.com:8443/v1",
        "https://api.example.com/v2",
    ] {
        let changed = ProviderEndpointScope::parse(changed).unwrap();
        let first_ref =
            keyring_ref_for_remote_api_key("planner", "profile", &first).unwrap();
        let changed_ref =
            keyring_ref_for_remote_api_key("planner", "profile", &changed).unwrap();
        assert_ne!(first_ref.account, changed_ref.account);
    }
}

#[test]
fn legacy_unbound_keyring_reference_requires_explicit_rebinding() {
    let endpoint = ProviderEndpointScope::parse("https://api.example.com/v1").unwrap();
    let legacy = SecretRef::FromKeyring {
        from_keyring: KeyringRef {
            service: String::from("blind_browser"),
            account: String::from("remote_planner:profile:api_key"),
        },
    };
    let error = resolve_secret_ref_for_endpoint(
        &legacy,
        "planner",
        "profile",
        &endpoint,
    )
    .expect_err("legacy entry must not be guessed or reused");
    assert!(error.contains("legacy unbound"));
    assert!(error.contains("re-enter"));
}

#[test]
fn credential_bound_to_another_endpoint_is_rejected_before_read() {
    let first = ProviderEndpointScope::parse("https://api.example.com/v1").unwrap();
    let second = ProviderEndpointScope::parse("https://other.example.com/v1").unwrap();
    let reference = keyring_ref_for_remote_api_key("planner", "profile", &first).unwrap();
    set_keyring_secret(&reference.service, &reference.account, "secret").unwrap();
    let secret_ref = SecretRef::FromKeyring {
        from_keyring: reference,
    };

    let error = resolve_secret_ref_for_endpoint(
        &secret_ref,
        "planner",
        "profile",
        &second,
    )
    .expect_err("scope mismatch must fail closed");
    assert!(error.contains("not authorized"));
    assert!(!error.contains("secret"));
}
'''
write("src-tauri/src/config/tests/keyring_tests.rs", keyring_tests)

replace_regex(
    "src/app.tsx",
    r'''          onEndpointBlur: \(\) => \{
            const s = panelStates\.remotePlannerPanelState;
            const url = s\.baseUrl\?\.trim\(\) \?\? "";
            if \(url\.length > 0 && url !== s\.loadedModelsEndpoint\) \{
              void loadRemotePlannerModels\(\);
            \}
          \},''',
    '''          onEndpointBlur: () => {
            // Endpoint editing is intentionally side-effect free. Credential-bearing
            // model discovery requires the explicit Load models action.
          },''',
    flags=0,
)

print("BBCR-004 deterministic source transformations applied")
