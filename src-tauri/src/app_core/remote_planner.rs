#[cfg(feature = "remote-openai")]
use super::api_key_tools::credential_async_client;
use super::planner_prompt::planner_interpretation_unavailable_error;
#[cfg(any(feature = "remote-openai", test))]
use super::planner_prompt::planner_system_prompt;
#[cfg(feature = "remote-openai")]
use super::planner_prompt::serialize_remote_planner_prompt;
#[cfg(feature = "remote-openai")]
use super::planner_redaction::sanitize_remote_planner_input;
use super::AppCore;
use crate::commands::{planner_output_schema, PlannerInput, PlannerOutput, ToolError};
#[cfg(feature = "remote-openai")]
use crate::config::resolve_secret_ref_for_endpoint;
use crate::config::{RemotePlannerPrivacySettings, RemotePlannerProfile, RemoteProviderKind};
#[cfg(any(feature = "remote-openai", test))]
use crate::provider_endpoint::ProviderEndpointScope;

impl AppCore {
    /// Snapshot the configured remote planner profile under the `AppCore` lock so
    /// the network resolution can run unlocked against an owned copy. Returns the
    /// same "profile unavailable" errors the inline path used to.
    pub(crate) fn remote_planner_profile_snapshot(
        &self,
    ) -> Result<(String, RemotePlannerProfile), ToolError> {
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

/// Resolve a planner output via the remote LLM for an already-snapshotted profile.
///
/// This is a free function (no `AppCore`), so it can run with the `AppCore` lock
/// released — the lock-scoped replanning orchestrator calls it after dropping the
/// guard, while the synchronous trait path calls it under the held guard.
pub(crate) fn resolve_remote_planner(
    profile_name: &str,
    profile: &RemotePlannerProfile,
    planner_input: &PlannerInput,
    privacy: &RemotePlannerPrivacySettings,
) -> Result<PlannerOutput, ToolError> {
    match profile.provider {
        RemoteProviderKind::OpenAi => {
            resolve_with_openai_planner(profile_name, profile, planner_input, privacy)
        }
        RemoteProviderKind::Ollama => {
            resolve_with_ollama_planner(profile_name, profile, planner_input, privacy)
        }
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
    profile_name: &str,
    profile: &RemotePlannerProfile,
    planner_input: &PlannerInput,
    privacy: &RemotePlannerPrivacySettings,
) -> Result<PlannerOutput, ToolError> {
    use async_openai::types::chat::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    };
    use async_openai::types::chat::{ResponseFormat, ResponseFormatJsonSchema};
    use async_openai::{config::OpenAIConfig, Client};

    let endpoint_scope = ProviderEndpointScope::parse(&profile.base_url).map_err(|reason| {
        planner_interpretation_unavailable_error(
            "planner_endpoint_invalid",
            "remote planner endpoint is invalid",
            false,
            Some(serde_json::json!({ "reason": reason })),
        )
    })?;
    let api_key =
        resolve_secret_ref_for_endpoint(&profile.api_key, "planner", profile_name, &endpoint_scope)
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
            resolve_secret_ref_for_endpoint(organization, "planner", profile_name, &endpoint_scope)
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
    let planner_safe_input =
        sanitize_remote_planner_input(planner_input, privacy, &endpoint_scope)?;
    let user_content = serialize_remote_planner_prompt(&planner_safe_input)?;
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
                        format!(
                            "failed to build planner system message for remote resolution: {error}"
                        ),
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
                        format!(
                            "failed to build planner user message for remote resolution: {error}"
                        ),
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
        .map_err(|_| planner_request_failed_error("OpenAI", &profile.model, &endpoint_scope))?;
    let content = response
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.message.content)
        .ok_or_else(|| {
            planner_interpretation_unavailable_error(
                "planner_response_missing",
                "remote planner returned no structured content",
                true,
                None,
            )
        })?;

    serde_json::from_str::<PlannerOutput>(&content).map_err(|error| {
        planner_interpretation_unavailable_error(
            "planner_response_invalid",
            format!("remote planner returned invalid planner JSON: {error}"),
            true,
            Some(serde_json::json!({ "content_length": content.len() })),
        )
    })
}

#[cfg(not(feature = "remote-openai"))]
fn resolve_with_openai_planner(
    _profile_name: &str,
    _profile: &RemotePlannerProfile,
    _planner_input: &PlannerInput,
    _privacy: &RemotePlannerPrivacySettings,
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
    profile_name: &str,
    profile: &RemotePlannerProfile,
    planner_input: &PlannerInput,
    privacy: &RemotePlannerPrivacySettings,
) -> Result<PlannerOutput, ToolError> {
    use async_openai::types::chat::ResponseFormat;
    use async_openai::types::chat::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    };
    use async_openai::{config::OpenAIConfig, Client};

    let endpoint_scope = ProviderEndpointScope::parse(&profile.base_url).map_err(|reason| {
        planner_interpretation_unavailable_error(
            "planner_endpoint_invalid",
            "Ollama planner endpoint is invalid",
            false,
            Some(serde_json::json!({ "reason": reason })),
        )
    })?;
    let api_key =
        resolve_secret_ref_for_endpoint(&profile.api_key, "planner", profile_name, &endpoint_scope)
            .map_err(|reason| {
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
    let planner_safe_input =
        sanitize_remote_planner_input(planner_input, privacy, &endpoint_scope)?;
    let user_content = serialize_remote_planner_prompt(&planner_safe_input)?;
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
                        format!(
                            "failed to build planner system message for Ollama resolution: {error}"
                        ),
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
                        format!(
                            "failed to build planner user message for Ollama resolution: {error}"
                        ),
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
        .map_err(|_| planner_request_failed_error("Ollama", &profile.model, &endpoint_scope))?;
    let content = response
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.message.content)
        .ok_or_else(|| {
            planner_interpretation_unavailable_error(
                "planner_response_missing",
                "Ollama planner returned no structured content",
                true,
                None,
            )
        })?;

    serde_json::from_str::<PlannerOutput>(&content).map_err(|error| {
        planner_interpretation_unavailable_error(
            "planner_response_invalid",
            format!("Ollama planner returned invalid planner JSON: {error}"),
            true,
            Some(serde_json::json!({ "content_length": content.len() })),
        )
    })
}

#[cfg(not(feature = "remote-openai"))]
fn resolve_with_ollama_planner(
    _profile_name: &str,
    _profile: &RemotePlannerProfile,
    _planner_input: &PlannerInput,
    _privacy: &RemotePlannerPrivacySettings,
) -> Result<PlannerOutput, ToolError> {
    Err(planner_interpretation_unavailable_error(
        "planner_backend_unavailable",
        "remote Ollama planner support is not enabled in this build",
        false,
        None,
    ))
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
