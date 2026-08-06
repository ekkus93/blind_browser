use super::api_key_tools::credential_client;
use super::planner_prompt::planner_interpretation_unavailable_error;
#[cfg(any(feature = "remote-openai", test))]
use super::planner_prompt::planner_system_prompt;
#[cfg(feature = "remote-openai")]
use super::planner_prompt::serialize_remote_planner_prompt;
use super::remote_data_consent::PreparedRemotePlannerRequest;
use crate::commands::{planner_output_schema, PlannerOutput, ToolError};
#[cfg(feature = "remote-openai")]
use crate::config::resolve_secret_ref_for_endpoint;
use crate::config::RemoteProviderKind;
#[cfg(any(feature = "remote-openai", test))]
use crate::provider_endpoint::ProviderEndpointScope;
#[cfg(feature = "remote-openai")]
use crate::resource_limits::{
    planner_requests, read_bounded_response, record_resource_size, BoundedResponseError,
    MAX_PLANNER_RESPONSE_BYTES,
};

pub(crate) fn resolve_remote_planner(
    prepared: &PreparedRemotePlannerRequest,
) -> Result<PlannerOutput, ToolError> {
    if prepared.page_origin.is_empty() || prepared.runtime_state_token.is_empty() {
        return Err(planner_interpretation_unavailable_error(
            "prepared_planner_request_invalid",
            "authorized remote planner request is missing its runtime binding",
            false,
            None,
        ));
    }
    let _authorization = prepared.authorization;
    match prepared.profile.provider {
        RemoteProviderKind::OpenAi => resolve_with_openai_planner(prepared),
        RemoteProviderKind::Ollama => resolve_with_ollama_planner(prepared),
    }
}

#[cfg(any(feature = "remote-openai", test))]
fn planner_safe_details(
    provider: &str,
    model: &str,
    endpoint_scope: &ProviderEndpointScope,
) -> serde_json::Value {
    serde_json::json!({
        "provider": provider,
        "model": model,
        "base_url": endpoint_scope.normalized_base_url(),
    })
}

#[cfg(any(feature = "remote-openai", test))]
fn planner_network_error(
    provider: &str,
    model: &str,
    endpoint_scope: &ProviderEndpointScope,
    error: &reqwest::Error,
    timeout_ms: u64,
) -> ToolError {
    let (code, message, retryable) = if error.is_timeout() {
        (
            "planner_request_timed_out",
            format!("{provider} planner request exceeded its configured timeout"),
            true,
        )
    } else if error.is_connect() && error_chain_indicates_tls(error) {
        (
            "planner_tls_failed",
            format!("{provider} planner TLS negotiation failed"),
            false,
        )
    } else if error.is_connect() {
        (
            "planner_connection_failed",
            format!("{provider} planner connection failed"),
            true,
        )
    } else if error_chain_indicates_tls(error) {
        (
            "planner_tls_failed",
            format!("{provider} planner TLS negotiation failed"),
            false,
        )
    } else {
        (
            "planner_request_failed",
            format!("{provider} planner request failed"),
            true,
        )
    };
    let mut details = planner_safe_details(provider, model, endpoint_scope);
    if let Some(object) = details.as_object_mut() {
        object.insert("timeout_ms".into(), timeout_ms.into());
    }
    planner_interpretation_unavailable_error(code, message, retryable, Some(details))
}

#[cfg(any(feature = "remote-openai", test))]
fn error_chain_indicates_tls(error: &(dyn std::error::Error + 'static)) -> bool {
    let mut current = Some(error);
    while let Some(source) = current {
        let label = source.to_string().to_ascii_lowercase();
        if label.contains("tls")
            || label.contains("certificate")
            || label.contains("cert error")
            || label.contains("ssl")
        {
            return true;
        }
        current = source.source();
    }
    false
}

#[cfg(any(feature = "remote-openai", test))]
fn planner_http_status_error(
    provider: &str,
    model: &str,
    endpoint_scope: &ProviderEndpointScope,
    status: reqwest::StatusCode,
) -> ToolError {
    let retryable = status == reqwest::StatusCode::REQUEST_TIMEOUT
        || status == reqwest::StatusCode::TOO_MANY_REQUESTS
        || status.is_server_error();
    let code = if status.is_redirection() {
        "planner_redirect_rejected"
    } else if status == reqwest::StatusCode::UNAUTHORIZED
        || status == reqwest::StatusCode::FORBIDDEN
    {
        "planner_auth_rejected"
    } else {
        "planner_http_status"
    };
    let mut details = planner_safe_details(provider, model, endpoint_scope);
    if let Some(object) = details.as_object_mut() {
        object.insert("http_status".into(), status.as_u16().into());
    }
    planner_interpretation_unavailable_error(
        code,
        if status.is_redirection() {
            format!("{provider} planner attempted a credential-bearing redirect")
        } else {
            format!("{provider} planner returned HTTP {}", status.as_u16())
        },
        retryable,
        Some(details),
    )
}

#[cfg(feature = "remote-openai")]
fn resolve_with_openai_planner(
    prepared: &PreparedRemotePlannerRequest,
) -> Result<PlannerOutput, ToolError> {
    resolve_openai_compatible(prepared, "OpenAI", true)
}

#[cfg(not(feature = "remote-openai"))]
fn resolve_with_openai_planner(
    _prepared: &PreparedRemotePlannerRequest,
) -> Result<PlannerOutput, ToolError> {
    Err(planner_interpretation_unavailable_error(
        "planner_backend_unavailable",
        "remote OpenAI planner support is not enabled in this build",
        false,
        None,
    ))
}

#[cfg(feature = "remote-openai")]
fn resolve_with_ollama_planner(
    prepared: &PreparedRemotePlannerRequest,
) -> Result<PlannerOutput, ToolError> {
    resolve_openai_compatible(prepared, "Ollama", false)
}

#[cfg(not(feature = "remote-openai"))]
fn resolve_with_ollama_planner(
    _prepared: &PreparedRemotePlannerRequest,
) -> Result<PlannerOutput, ToolError> {
    Err(planner_interpretation_unavailable_error(
        "planner_backend_unavailable",
        "remote Ollama planner support is not enabled in this build",
        false,
        None,
    ))
}

#[cfg(feature = "remote-openai")]
fn resolve_openai_compatible(
    prepared: &PreparedRemotePlannerRequest,
    provider: &str,
    strict_schema: bool,
) -> Result<PlannerOutput, ToolError> {
    let profile_name = prepared.profile_name.as_str();
    let profile = &prepared.profile;
    let endpoint_scope = &prepared.endpoint_scope;
    let _permit = planner_requests().try_acquire().map_err(|limit| {
        let code = limit.code("planner");
        planner_interpretation_unavailable_error(
            &code,
            &format!("Remote planner request was not started: {limit}."),
            true,
            Some(limit.details()),
        )
    })?;

    let api_key =
        resolve_secret_ref_for_endpoint(&profile.api_key, "planner", profile_name, endpoint_scope)
            .map_err(|reason| {
                planner_interpretation_unavailable_error(
                    "planner_secret_unavailable",
                    "remote planner API key could not be resolved for the configured endpoint",
                    false,
                    Some(serde_json::json!({ "reason": reason })),
                )
            })?;
    let organization = profile
        .organization
        .as_ref()
        .map(|secret| {
            resolve_secret_ref_for_endpoint(secret, "planner", profile_name, endpoint_scope)
        })
        .transpose()
        .map_err(|reason| {
            planner_interpretation_unavailable_error(
                "planner_secret_unavailable",
                "remote planner organization secret could not be resolved",
                false,
                Some(serde_json::json!({ "reason": reason })),
            )
        })?;

    let client = credential_client(profile.timeout_ms, "remote planner").map_err(|reason| {
        planner_interpretation_unavailable_error(
            "planner_request_build_failed",
            "failed to build the credential-bearing planner client",
            false,
            Some(serde_json::json!({ "reason": reason })),
        )
    })?;
    let endpoint = endpoint_scope
        .endpoint_url("chat/completions")
        .map_err(|reason| {
            planner_interpretation_unavailable_error(
                "planner_request_build_failed",
                "failed to construct the remote planner endpoint",
                false,
                Some(serde_json::json!({ "reason": reason })),
            )
        })?;
    let user_content = serialize_remote_planner_prompt(&prepared.sanitized_input)?;
    let mut request_body = serde_json::json!({
        "model": profile.model,
        "temperature": profile.temperature_milli as f32 / 1_000.0,
        "messages": [
            { "role": "system", "content": planner_system_prompt() },
            { "role": "user", "content": user_content },
        ],
    });
    let request_object = request_body
        .as_object_mut()
        .expect("planner request body is statically an object");
    if strict_schema {
        request_object.insert(
            "max_completion_tokens".into(),
            serde_json::json!(profile.max_output_tokens),
        );
        request_object.insert(
            "response_format".into(),
            serde_json::json!({
                "type": "json_schema",
                "json_schema": {
                    "name": "planner_output",
                    "description": "Structured deterministic planner output for blind_browser.",
                    "schema": planner_output_schema(),
                    "strict": true,
                }
            }),
        );
    } else {
        request_object.insert(
            "max_tokens".into(),
            serde_json::json!(profile.max_output_tokens),
        );
        request_object.insert(
            "response_format".into(),
            serde_json::json!({ "type": "json_object" }),
        );
    }

    let mut request = client
        .post(endpoint)
        .bearer_auth(api_key)
        .json(&request_body);
    if let Some(organization) = organization {
        request = request.header("OpenAI-Organization", organization);
    }
    if let Some(project) = profile.project.as_ref() {
        request = request.header("OpenAI-Project", project);
    }
    let response = request.send().map_err(|error| {
        planner_network_error(
            provider,
            &profile.model,
            endpoint_scope,
            &error,
            profile.timeout_ms.max(1),
        )
    })?;
    let status = response.status();
    if !status.is_success() {
        return Err(planner_http_status_error(
            provider,
            &profile.model,
            endpoint_scope,
            status,
        ));
    }
    let body = read_bounded_response(response, MAX_PLANNER_RESPONSE_BYTES).map_err(|error| {
        planner_response_read_error(provider, &profile.model, endpoint_scope, error)
    })?;
    record_resource_size("remote_planner_response", body.len());
    parse_openai_compatible_response(provider, &body)
}

#[cfg(any(feature = "remote-openai", test))]
fn planner_response_read_error(
    provider: &str,
    model: &str,
    endpoint_scope: &ProviderEndpointScope,
    error: BoundedResponseError,
) -> ToolError {
    match error {
        BoundedResponseError::DeclaredTooLarge { declared, maximum } => {
            let mut details = planner_safe_details(provider, model, endpoint_scope);
            if let Some(object) = details.as_object_mut() {
                object.insert("declared_bytes".into(), declared.into());
                object.insert("maximum_bytes".into(), maximum.into());
            }
            planner_interpretation_unavailable_error(
                "planner_response_too_large",
                format!("{provider} planner response exceeded the configured byte limit"),
                false,
                Some(details),
            )
        }
        BoundedResponseError::BodyTooLarge { maximum } => {
            let mut details = planner_safe_details(provider, model, endpoint_scope);
            if let Some(object) = details.as_object_mut() {
                object.insert("maximum_bytes".into(), maximum.into());
            }
            planner_interpretation_unavailable_error(
                "planner_response_too_large",
                format!("{provider} planner response exceeded the configured byte limit"),
                false,
                Some(details),
            )
        }
        BoundedResponseError::ReadFailed(_) => planner_interpretation_unavailable_error(
            "planner_response_read_failed",
            format!("{provider} planner response could not be read"),
            true,
            Some(planner_safe_details(provider, model, endpoint_scope)),
        ),
    }
}

#[derive(serde::Deserialize)]
struct OpenAiCompatibleResponse {
    choices: Vec<OpenAiCompatibleChoice>,
}

#[derive(serde::Deserialize)]
struct OpenAiCompatibleChoice {
    message: OpenAiCompatibleMessage,
}

#[derive(serde::Deserialize)]
struct OpenAiCompatibleMessage {
    content: Option<String>,
}

#[cfg(any(feature = "remote-openai", test))]
fn parse_openai_compatible_response(label: &str, body: &[u8]) -> Result<PlannerOutput, ToolError> {
    let response: OpenAiCompatibleResponse = serde_json::from_slice(body).map_err(|_| {
        planner_interpretation_unavailable_error(
            "planner_response_parse_failed",
            format!("{label} planner returned malformed response JSON"),
            true,
            Some(serde_json::json!({ "response_bytes": body.len() })),
        )
    })?;
    let content = response
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.message.content)
        .ok_or_else(|| {
            planner_interpretation_unavailable_error(
                "planner_response_missing",
                format!("{label} planner returned no structured content"),
                true,
                None,
            )
        })?;
    serde_json::from_str::<PlannerOutput>(&content).map_err(|_| {
        planner_interpretation_unavailable_error(
            "planner_response_schema_invalid",
            format!("{label} planner content did not match the planner schema"),
            true,
            Some(serde_json::json!({ "content_length": content.len() })),
        )
    })
}

impl crate::AppCore {
    /// Snapshot the configured remote planner profile under the `AppCore` lock so
    /// network preparation can run against an owned, immutable copy.
    pub(crate) fn remote_planner_profile_snapshot(
        &self,
    ) -> Result<(String, crate::config::RemotePlannerProfile), ToolError> {
        let Some(profile_name) = self.config.providers.planner.remote_profile.as_deref() else {
            return Err(planner_interpretation_unavailable_error(
                "planner_profile_unavailable",
                "remote planner mode requires a configured planner profile",
                false,
                None,
            ));
        };
        let Some(profile) = self.config.remote_planner_profiles.get(profile_name) else {
            return Err(planner_interpretation_unavailable_error(
                "planner_profile_unavailable",
                format!("configured remote planner profile '{profile_name}' was not found"),
                false,
                None,
            ));
        };
        Ok((profile_name.to_string(), profile.clone()))
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::{Shutdown, TcpListener};
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    use super::*;

    fn valid_response() -> Vec<u8> {
        let content = serde_json::json!({
            "status": "Complete",
            "intent": {
                "name": "ReadPage",
                "goal": "Read the current page",
                "target_description": null
            },
            "selected_skills": [],
            "steps": [],
            "requires_confirmation": false,
            "confirmation_reason": null,
            "blocked_reason": null,
            "user_message": "Done"
        })
        .to_string();
        serde_json::json!({
            "choices": [{ "message": { "content": content } }]
        })
        .to_string()
        .into_bytes()
    }

    #[test]
    fn planner_error_taxonomy_exposes_only_safe_metadata() {
        let endpoint = ProviderEndpointScope::parse("https://api.example.com/v1")
            .expect("test endpoint must be valid");
        let error = planner_http_status_error(
            "OpenAI",
            "gpt-test",
            &endpoint,
            reqwest::StatusCode::TOO_MANY_REQUESTS,
        );
        assert_eq!(error.code, "planner_http_status");
        assert!(error.retryable);
        let serialized = serde_json::to_string(&error).expect("ToolError must serialize");
        assert!(serialized.contains("OpenAI"));
        assert!(serialized.contains("gpt-test"));
        assert!(serialized.contains("429"));
        assert!(!serialized.contains("response_body"));
        assert!(!serialized.contains("authorization"));
        assert!(!serialized.contains("sk-abcdefghijklmnopqrstuv"));
    }

    #[test]
    fn response_parse_and_schema_errors_are_distinct() {
        let parse = parse_openai_compatible_response("OpenAI", b"not-json")
            .expect_err("malformed envelope must fail");
        assert_eq!(parse.code, "planner_response_parse_failed");

        let schema_body = serde_json::json!({
            "choices": [{ "message": { "content": "{}" } }]
        })
        .to_string();
        let schema = parse_openai_compatible_response("OpenAI", schema_body.as_bytes())
            .expect_err("invalid planner content must fail");
        assert_eq!(schema.code, "planner_response_schema_invalid");
    }

    #[test]
    fn normal_openai_compatible_response_parses() {
        parse_openai_compatible_response("OpenAI", &valid_response())
            .expect("representative response should parse");
    }

    #[test]
    fn blocking_client_timeout_closes_the_underlying_request() {
        std::env::set_var("NO_PROXY", "localhost,127.0.0.1");
        std::env::set_var("no_proxy", "localhost,127.0.0.1");
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let address = listener.local_addr().expect("listener has address");
        let (closed_tx, closed_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("request should connect");
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .expect("read timeout should install");
            let mut request = [0_u8; 8192];
            let _ = stream
                .read(&mut request)
                .expect("request should be readable");
            let mut probe = [0_u8; 1];
            let closed = loop {
                match stream.read(&mut probe) {
                    Ok(0) => break true,
                    Ok(_) => continue,
                    Err(error)
                        if matches!(
                            error.kind(),
                            std::io::ErrorKind::ConnectionReset
                                | std::io::ErrorKind::BrokenPipe
                                | std::io::ErrorKind::UnexpectedEof
                        ) =>
                    {
                        break true;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => continue,
                    Err(_) => break false,
                }
            };
            let _ = stream.shutdown(Shutdown::Both);
            closed_tx.send(closed).expect("closed state should send");
        });

        let client = credential_client(100, "planner timeout test").expect("client should build");
        let started = Instant::now();
        let error = client
            .post(format!("http://{address}/v1/chat/completions"))
            .body("{}")
            .send()
            .expect_err("server never responds");
        assert!(error.is_timeout());
        assert!(started.elapsed() < Duration::from_secs(1));
        assert!(closed_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("server should observe closure"));
        server.join().expect("server should join");
    }
}
