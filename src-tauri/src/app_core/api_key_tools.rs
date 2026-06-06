use std::time::Duration;

use crate::config::RemoteProviderKind;
#[cfg(feature = "remote-openai")]
use crate::config::resolve_secret_ref;
use reqwest::blocking::Client;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RemoteApiKeyTarget {
    Planner,
    Tts,
    Asr,
}

impl RemoteApiKeyTarget {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Planner => "planner",
            Self::Tts => "TTS",
            Self::Asr => "ASR",
        }
    }
}

pub(crate) struct RemoteOpenAiApiKeyTestProfile<'a> {
    pub(crate) profile_name: &'a str,
    pub(crate) provider: &'a RemoteProviderKind,
    pub(crate) base_url: &'a str,
    pub(crate) configured_api_key: &'a crate::config::SecretRef,
    pub(crate) organization: Option<&'a crate::config::SecretRef>,
    pub(crate) project: Option<&'a str>,
    pub(crate) timeout_ms: u64,
}

pub(crate) fn test_remote_openai_profile_api_key(
    target: RemoteApiKeyTarget,
    profile: RemoteOpenAiApiKeyTestProfile<'_>,
    api_key_override: Option<&str>,
) -> Result<String, String> {
    if *profile.provider != RemoteProviderKind::OpenAi {
        return Err(format!(
            "Remote {} profile '{}' is not using OpenAI.",
            target.label(),
            profile.profile_name
        ));
    }

    let api_key = match api_key_override.map(str::trim) {
        Some(override_value) if !override_value.is_empty() => override_value.to_string(),
        _ => resolve_secret_ref(profile.configured_api_key).map_err(|reason| {
            format!(
                "Remote {} API key test could not read the configured secret: {reason}",
                target.label()
            )
        })?,
    };
    let organization = profile
        .organization
        .map(resolve_secret_ref)
        .transpose()
        .map_err(|reason| {
            format!(
                "Remote {} API key test could not read the configured organization secret: {reason}",
                target.label()
            )
        })?;

    test_openai_api_key_connectivity(
        profile.base_url,
        &api_key,
        organization.as_deref(),
        profile.project,
        profile.timeout_ms,
    )?;

    Ok(match api_key_override.map(str::trim) {
        Some(override_value) if !override_value.is_empty() => {
            String::from("OpenAI accepted the entered API key.")
        }
        _ => String::from("OpenAI accepted the configured API key."),
    })
}

pub(crate) fn test_openai_api_key_connectivity(
    base_url: &str,
    api_key: &str,
    organization: Option<&str>,
    project: Option<&str>,
    timeout_ms: u64,
) -> Result<(), String> {
    let endpoint = format!("{}/models", base_url.trim_end_matches('/'));
    let client = Client::builder()
        .timeout(Duration::from_millis(timeout_ms.max(1)))
        .build()
        .map_err(|error| format!("failed to build the OpenAI API key test client: {error}"))?;

    let mut request = client.get(endpoint).bearer_auth(api_key);
    if let Some(organization) = organization {
        request = request.header("OpenAI-Organization", organization);
    }
    if let Some(project) = project {
        request = request.header("OpenAI-Project", project);
    }

    let response = request
        .send()
        .map_err(|error| format!("OpenAI API key test request failed: {error}"))?;

    if response.status().is_success() {
        return Ok(());
    }

    let status = response.status();
    Err(openai_api_key_test_failure_message(status))
}

#[derive(Debug, serde::Deserialize)]
struct OpenAiCompatibleModelsResponse {
    data: Vec<OpenAiCompatibleModelEntry>,
}

#[derive(Debug, serde::Deserialize)]
struct OpenAiCompatibleModelEntry {
    id: String,
}

pub(crate) fn fetch_openai_compatible_models(
    base_url: &str,
    api_key: Option<&str>,
    organization: Option<&str>,
    project: Option<&str>,
    timeout_ms: u64,
) -> Result<Vec<String>, String> {
    let endpoint = format!("{}/models", base_url.trim().trim_end_matches('/'));
    let client = Client::builder()
        .timeout(Duration::from_millis(timeout_ms.max(1)))
        .build()
        .map_err(|error| format!("failed to build the remote model list client: {error}"))?;

    let mut request = client.get(endpoint);
    if let Some(api_key) = api_key.filter(|value| !value.trim().is_empty()) {
        request = request.bearer_auth(api_key.trim());
    }
    if let Some(organization) = organization {
        request = request.header("OpenAI-Organization", organization);
    }
    if let Some(project) = project {
        request = request.header("OpenAI-Project", project);
    }

    let response = request
        .send()
        .map_err(|error| format!("remote model list request failed: {error}"))?;

    let status = response.status();
    if !status.is_success() {
        return Err(match status {
            reqwest::StatusCode::UNAUTHORIZED => String::from(
                "The endpoint rejected the current API key while loading models.",
            ),
            reqwest::StatusCode::FORBIDDEN => String::from(
                "The endpoint refused this model list request. Check that the current API key has access.",
            ),
            _ => format!("The endpoint returned {} while loading models.", status),
        });
    }

    let mut models = response
        .json::<OpenAiCompatibleModelsResponse>()
        .map_err(|error| format!("failed to parse the remote model list response: {error}"))?
        .data
        .into_iter()
        .map(|entry| entry.id.trim().to_string())
        .filter(|model: &String| !model.is_empty())
        .collect::<Vec<_>>();
    models.sort();
    models.dedup();

    if models.is_empty() {
        return Err(String::from(
            "The endpoint responded successfully but did not return any models.",
        ));
    }

    Ok(models)
}

fn openai_api_key_test_failure_message(status: reqwest::StatusCode) -> String {
    match status {
        reqwest::StatusCode::UNAUTHORIZED => String::from(
            "OpenAI rejected that API key. Check the key and try again, or create one at https://platform.openai.com/account/api-keys.",
        ),
        reqwest::StatusCode::FORBIDDEN => String::from(
            "OpenAI refused this request. Check that the API key has access to this project.",
        ),
        reqwest::StatusCode::TOO_MANY_REQUESTS => {
            String::from("OpenAI rate-limited the API key test. Wait a moment and try again.")
        }
        reqwest::StatusCode::REQUEST_TIMEOUT
        | reqwest::StatusCode::GATEWAY_TIMEOUT
        | reqwest::StatusCode::BAD_GATEWAY
        | reqwest::StatusCode::SERVICE_UNAVAILABLE => {
            String::from("OpenAI did not respond in time. Try the API key test again.")
        }
        _ => format!(
            "OpenAI could not verify the API key right now (HTTP {}).",
            status.as_u16()
        ),
    }
}
