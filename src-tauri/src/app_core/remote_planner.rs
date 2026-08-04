#[cfg(feature = "remote-openai")]
use super::api_key_tools::credential_async_client;
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
fn planner_request_failed_error(
    provider: &str,
    model: &str,
    endpoint_scope: &ProviderEndpointScope,
) -> ToolError {
    planner_interpretation_unavailable_error(
        "planner_request_failed",
        format!("{provider} planner request failed"),
        true,
        Some(serde_json::json!({
            "provider": provider,
            "model": model,
            "base_url": endpoint_scope.normalized_base_url(),
        })),
    )
}

#[cfg(feature = "remote-openai")]
fn resolve_with_openai_planner(
    prepared: &PreparedRemotePlannerRequest,
) -> Result<PlannerOutput, ToolError> {
    use async_openai::types::chat::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    };
    use async_openai::types::chat::{ResponseFormat, ResponseFormatJsonSchema};
    use async_openai::{config::OpenAIConfig, Client};

    let profile_name = prepared.profile_name.as_str();
    let profile = &prepared.profile;
    let endpoint_scope = &prepared.endpoint_scope;
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
    let http_client = credential_async_client(profile.timeout_ms).map_err(|reason| {
        planner_interpretation_unavailable_error(
            "planner_request_build_failed",
            "failed to build the credential-bearing planner client",
            false,
            Some(serde_json::json!({ "reason": reason })),
        )
    })?;

    let mut openai_config = OpenAIConfig::new()
        .with_api_base(endpoint_scope.normalized_base_url().to_string())
        .with_api_key(api_key);
    if let Some(organization) = profile.organization.as_ref() {
        openai_config = openai_config.with_org_id(
            resolve_secret_ref_for_endpoint(organization, "planner", profile_name, endpoint_scope)
                .map_err(|reason| {
                    planner_interpretation_unavailable_error(
                        "planner_secret_unavailable",
                        "remote planner organization secret could not be resolved",
                        false,
                        Some(serde_json::json!({ "reason": reason })),
                    )
                })?,
        );
    }
    if let Some(project) = profile.project.as_ref() {
        openai_config = openai_config.with_project_id(project.clone());
    }

    let client = Client::with_config(openai_config).with_http_client(http_client);
    let user_content = serialize_remote_planner_prompt(&prepared.sanitized_input)?;
    let request = CreateChatCompletionRequestArgs::default()
        .model(profile.model.clone())
        .temperature(profile.temperature_milli as f32 / 1_000.0)
        .max_completion_tokens(profile.max_output_tokens)
        .response_format(ResponseFormat::JsonSchema {
            json_schema: ResponseFormatJsonSchema {
                description: Some(String::from(
                    "Structured deterministic planner output for blind_browser.",
                )),
                name: String::from("planner_output"),
                schema: Some(planner_output_schema()),
                strict: Some(true),
            },
        })
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content(planner_system_prompt())
                .build()
                .map_err(|error| {
                    planner_interpretation_unavailable_error(
                        "planner_request_build_failed",
                        format!("failed to build planner system message: {error}"),
                        false,
                        None,
                    )
                })?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(user_content)
                .build()
                .map_err(|error| {
                    planner_interpretation_unavailable_error(
                        "planner_request_build_failed",
                        format!("failed to build planner user message: {error}"),
                        false,
                        None,
                    )
                })?
                .into(),
        ])
        .build()
        .map_err(|error| {
            planner_interpretation_unavailable_error(
                "planner_request_build_failed",
                format!("failed to build remote planner request: {error}"),
                false,
                None,
            )
        })?;

    let response = futures::executor::block_on(client.chat().create(request))
        .map_err(|_| planner_request_failed_error("OpenAI", &profile.model, endpoint_scope))?;
    parse_planner_response("remote planner", response.choices)
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
    use async_openai::types::chat::ResponseFormat;
    use async_openai::types::chat::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    };
    use async_openai::{config::OpenAIConfig, Client};

    let profile_name = prepared.profile_name.as_str();
    let profile = &prepared.profile;
    let endpoint_scope = &prepared.endpoint_scope;
    let api_key_result = resolve_secret_ref_for_endpoint(
        &profile.api_key,
        "planner",
        profile_name,
        endpoint_scope,
    );
    let api_key = api_key_result.map_err(|reason| {
        planner_interpretation_unavailable_error(
            "planner_secret_unavailable",
            "Ollama planner API key placeholder could not be resolved for the configured endpoint",
            false,
            Some(serde_json::json!({ "reason": reason })),
        )
    })?;
    let http_client = credential_async_client(profile.timeout_ms).map_err(|reason| {
        planner_interpretation_unavailable_error(
            "planner_request_build_failed",
            "failed to build the credential-bearing Ollama client",
            false,
            Some(serde_json::json!({ "reason": reason })),
        )
    })?;

    let client = Client::with_config(
        OpenAIConfig::new()
            .with_api_base(endpoint_scope.normalized_base_url().to_string())
            .with_api_key(api_key),
    )
    .with_http_client(http_client);
    let user_content = serialize_remote_planner_prompt(&prepared.sanitized_input)?;
    let request = CreateChatCompletionRequestArgs::default()
        .model(profile.model.clone())
        .temperature(profile.temperature_milli as f32 / 1_000.0)
        .max_tokens(profile.max_output_tokens)
        .response_format(ResponseFormat::JsonObject)
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content(planner_system_prompt())
                .build()
                .map_err(|error| {
                    planner_interpretation_unavailable_error(
                        "planner_request_build_failed",
                        format!("failed to build Ollama system message: {error}"),
                        false,
                        None,
                    )
                })?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(user_content)
                .build()
                .map_err(|error| {
                    planner_interpretation_unavailable_error(
                        "planner_request_build_failed",
                        format!("failed to build Ollama user message: {error}"),
                        false,
                        None,
                    )
                })?
                .into(),
        ])
        .build()
        .map_err(|error| {
            planner_interpretation_unavailable_error(
                "planner_request_build_failed",
                format!("failed to build Ollama planner request: {error}"),
                false,
                None,
            )
        })?;

    let response = futures::executor::block_on(client.chat().create(request))
        .map_err(|_| planner_request_failed_error("Ollama", &profile.model, endpoint_scope))?;
    parse_planner_response("Ollama planner", response.choices)
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
fn parse_planner_response(
    label: &str,
    choices: Vec<async_openai::types::chat::ChatChoice>,
) -> Result<PlannerOutput, ToolError> {
    let content = choices
        .into_iter()
        .next()
        .and_then(|choice| choice.message.content)
        .ok_or_else(|| {
            planner_interpretation_unavailable_error(
                "planner_response_missing",
                format!("{label} returned no structured content"),
                true,
                None,
            )
        })?;
    serde_json::from_str::<PlannerOutput>(&content).map_err(|error| {
        planner_interpretation_unavailable_error(
            "planner_response_invalid",
            format!("{label} returned invalid planner JSON: {error}"),
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
    use super::*;

    #[test]
    fn planner_request_failure_exposes_only_safe_connection_metadata() {
        let endpoint = ProviderEndpointScope::parse("https://api.example.com/v1")
            .expect("test endpoint must be valid");
        let error = planner_request_failed_error("OpenAI", "gpt-test", &endpoint);
        let serialized = serde_json::to_string(&error).expect("ToolError must serialize");
        assert!(serialized.contains("OpenAI"));
        assert!(serialized.contains("gpt-test"));
        assert!(serialized.contains("https://api.example.com/v1"));
        assert!(!serialized.contains("response_body"));
        assert!(!serialized.contains("authorization"));
        assert!(!serialized.contains("sk-abcdefghijklmnopqrstuv"));
    }
}
