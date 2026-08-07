use std::time::Duration;

use crate::config::{resolve_secret_ref_for_endpoint, RemoteProviderKind};
use crate::provider_endpoint::ProviderEndpointScope;
use reqwest::blocking::Client;
use reqwest::redirect::Policy;

use crate::resource_limits::{
    model_list_requests, read_bounded_response, record_resource_size, MAX_MODEL_LIST_RESPONSE_BYTES,
};

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

    pub(crate) fn provider_kind(self) -> &'static str {
        match self {
            Self::Planner => "planner",
            Self::Tts => "tts",
            Self::Asr => "asr",
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

    let endpoint_scope = ProviderEndpointScope::parse(profile.base_url).map_err(|reason| {
        format!(
            "Remote {} profile '{}' has an invalid endpoint: {reason}",
            target.label(),
            profile.profile_name
        )
    })?;
    let entered_key = api_key_override
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let api_key = match entered_key {
        Some(value) => value.to_string(),
        None => resolve_secret_ref_for_endpoint(
            profile.configured_api_key,
            target.provider_kind(),
            profile.profile_name,
            &endpoint_scope,
        )
        .map_err(|reason| {
            format!(
                "Remote {} API key test could not read the configured secret: {reason}",
                target.label()
            )
        })?,
    };
    let organization = profile
        .organization
        .map(|secret| {
            resolve_secret_ref_for_endpoint(
                secret,
                target.provider_kind(),
                profile.profile_name,
                &endpoint_scope,
            )
        })
        .transpose()
        .map_err(|reason| {
            format!(
                "Remote {} API key test could not read the configured organization secret: {reason}",
                target.label()
            )
        })?;

    test_openai_api_key_connectivity(
        &endpoint_scope,
        &api_key,
        organization.as_deref(),
        profile.project,
        profile.timeout_ms,
    )?;

    Ok(match entered_key {
        Some(_) => format!(
            "OpenAI accepted the entered API key for {}.",
            endpoint_scope.normalized_base_url()
        ),
        None => format!(
            "OpenAI accepted the configured API key for {}.",
            endpoint_scope.normalized_base_url()
        ),
    })
}

pub(crate) fn test_openai_api_key_connectivity(
    endpoint_scope: &ProviderEndpointScope,
    api_key: &str,
    organization: Option<&str>,
    project: Option<&str>,
    timeout_ms: u64,
) -> Result<(), String> {
    let client = credential_client(timeout_ms, "OpenAI API key test")?;
    let mut request = client.get(endpoint_scope.models_url()).bearer_auth(api_key);
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

    Err(openai_api_key_test_failure_message(response.status()))
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
    endpoint_scope: &ProviderEndpointScope,
    api_key: Option<&str>,
    organization: Option<&str>,
    project: Option<&str>,
    timeout_ms: u64,
) -> Result<Vec<String>, String> {
    let _permit = model_list_requests()
        .try_acquire()
        .map_err(|limit| format!("Remote model-list request was not started: {limit}."))?;
    let client = credential_client(timeout_ms, "remote model list")?;

    let mut request = client.get(endpoint_scope.models_url());
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
            reqwest::StatusCode::MOVED_PERMANENTLY
            | reqwest::StatusCode::FOUND
            | reqwest::StatusCode::SEE_OTHER
            | reqwest::StatusCode::TEMPORARY_REDIRECT
            | reqwest::StatusCode::PERMANENT_REDIRECT => String::from(
                "The endpoint attempted to redirect a credential-bearing model request. Redirects are not allowed.",
            ),
            _ => format!("The endpoint returned {} while loading models.", status),
        });
    }

    let body = read_bounded_response(response, MAX_MODEL_LIST_RESPONSE_BYTES).map_err(|error| {
        format!("remote model list response exceeded or failed its byte budget: {error}")
    })?;
    record_resource_size("remote_model_list_response", body.len());
    let mut models = serde_json::from_slice::<OpenAiCompatibleModelsResponse>(&body)
        .map_err(|_| String::from("failed to parse the remote model list response"))?
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

pub(crate) fn credential_client(timeout_ms: u64, purpose: &str) -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_millis(timeout_ms.max(1)))
        .redirect(Policy::none())
        .build()
        .map_err(|error| format!("failed to build the {purpose} client: {error}"))
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
        reqwest::StatusCode::MOVED_PERMANENTLY
        | reqwest::StatusCode::FOUND
        | reqwest::StatusCode::SEE_OTHER
        | reqwest::StatusCode::TEMPORARY_REDIRECT
        | reqwest::StatusCode::PERMANENT_REDIRECT => String::from(
            "OpenAI attempted to redirect the credential-bearing API key test. Redirects are not allowed.",
        ),
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

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    use super::*;

    #[test]
    fn credential_bearing_requests_do_not_follow_redirects() {
        let destination = TcpListener::bind("127.0.0.1:0").unwrap();
        destination.set_nonblocking(true).unwrap();
        let destination_address = destination.local_addr().unwrap();
        let (destination_tx, destination_rx) = mpsc::channel();
        let destination_thread = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_millis(750);
            while Instant::now() < deadline {
                match destination.accept() {
                    Ok((_stream, _)) => {
                        let _ = destination_tx.send(true);
                        return;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => return,
                }
            }
        });

        let redirect = TcpListener::bind("127.0.0.1:0").unwrap();
        let redirect_address = redirect.local_addr().unwrap();
        let redirect_thread = thread::spawn(move || {
            let (mut stream, _) = redirect.accept().unwrap();
            let mut buffer = [0_u8; 2048];
            let _ = stream.read(&mut buffer);
            write!(
                stream,
                "HTTP/1.1 302 Found\r\nLocation: http://{destination_address}/models\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            )
            .unwrap();
        });

        let scope = ProviderEndpointScope::parse(&format!("http://{redirect_address}/v1")).unwrap();
        let error = test_openai_api_key_connectivity(&scope, "secret", None, None, 2_000)
            .expect_err("redirect must be rejected");
        assert!(error.contains("Redirects are not allowed"));
        assert!(destination_rx
            .recv_timeout(Duration::from_millis(900))
            .is_err());

        redirect_thread.join().unwrap();
        destination_thread.join().unwrap();
    }
}
