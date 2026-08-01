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
    "src-tauri/src/app_core/command_dispatch.rs",
    '''        profile: RemotePlannerProfile,
        available_tools: Vec<AvailableTool>,''',
    '''        profile_name: String,
        profile: RemotePlannerProfile,
        available_tools: Vec<AvailableTool>,''',
)
replace_once(
    "src-tauri/src/app_core/command_dispatch.rs",
    "        let profile = self.remote_planner_profile_snapshot()?;",
    "        let (profile_name, profile) = self.remote_planner_profile_snapshot()?;",
)
replace_once(
    "src-tauri/src/app_core/command_dispatch.rs",
    '''            planner_input: Box::new(planner_input),
            profile,
            available_tools,''',
    '''            planner_input: Box::new(planner_input),
            profile_name,
            profile,
            available_tools,''',
)

replace_once(
    "src-tauri/src/app_core/replanning_orchestrator.rs",
    '''                planner_input,
                profile,
                available_tools,''',
    '''                planner_input,
                profile_name,
                profile,
                available_tools,''',
)
replace_once(
    "src-tauri/src/app_core/replanning_orchestrator.rs",
    "                let planner_output = resolve_remote_planner(&profile, &planner_input)?;",
    "                let planner_output =\n                    resolve_remote_planner(&profile_name, &profile, &planner_input)?;",
)

remote = read("src-tauri/src/app_core/remote_planner.rs")
remote = remote.replace(
    'use super::planner_prompt::PlannerPromptPayload;',
    'use super::api_key_tools::credential_async_client;\n#[cfg(feature = "remote-openai")]\nuse super::planner_prompt::PlannerPromptPayload;',
    1,
)
remote = remote.replace(
    '#[cfg(feature = "remote-openai")]\nuse crate::config::resolve_secret_ref;\nuse crate::config::{RemotePlannerProfile, RemoteProviderKind};',
    '#[cfg(feature = "remote-openai")]\nuse crate::config::resolve_secret_ref_for_endpoint;\nuse crate::config::{RemotePlannerProfile, RemoteProviderKind};\n#[cfg(feature = "remote-openai")]\nuse crate::provider_endpoint::ProviderEndpointScope;',
    1,
)
remote = remote.replace(
    '''    ) -> Result<RemotePlannerProfile, ToolError> {''',
    '''    ) -> Result<(String, RemotePlannerProfile), ToolError> {''',
    1,
)
remote = remote.replace(
    "        Ok(profile.clone())",
    "        Ok((profile_name.to_string(), profile.clone()))",
    1,
)
remote = remote.replace(
    '''pub(crate) fn resolve_remote_planner(
    profile: &RemotePlannerProfile,
    planner_input: &PlannerInput,
) -> Result<PlannerOutput, ToolError> {
    match profile.provider {
        RemoteProviderKind::OpenAi => resolve_with_openai_planner(profile, planner_input),
        RemoteProviderKind::Ollama => resolve_with_ollama_planner(profile, planner_input),
    }
}''',
    '''pub(crate) fn resolve_remote_planner(
    profile_name: &str,
    profile: &RemotePlannerProfile,
    planner_input: &PlannerInput,
) -> Result<PlannerOutput, ToolError> {
    match profile.provider {
        RemoteProviderKind::OpenAi => {
            resolve_with_openai_planner(profile_name, profile, planner_input)
        }
        RemoteProviderKind::Ollama => {
            resolve_with_ollama_planner(profile_name, profile, planner_input)
        }
    }
}''',
    1,
)
remote = remote.replace(
    '''fn resolve_with_openai_planner(
    profile: &RemotePlannerProfile,
    planner_input: &PlannerInput,''',
    '''fn resolve_with_openai_planner(
    profile_name: &str,
    profile: &RemotePlannerProfile,
    planner_input: &PlannerInput,''',
    1,
)
openai_secret_pattern = r'''    let api_key = resolve_secret_ref\(&profile\.api_key\)\.map_err\(\|reason\| \{.*?    \}\)\?;

    let mut openai_config = OpenAIConfig::new\(\)
        \.with_api_base\(profile\.base_url\.clone\(\)\)
        \.with_api_key\(api_key\);'''
openai_secret_replacement = r'''    let endpoint_scope = ProviderEndpointScope::parse(&profile.base_url).map_err(|reason| {
        planner_interpretation_unavailable_error(
            "planner_endpoint_invalid",
            "remote planner endpoint is invalid",
            false,
            Some(serde_json::json!({ "reason": reason })),
        )
    })?;
    let api_key = resolve_secret_ref_for_endpoint(
        &profile.api_key,
        "planner",
        profile_name,
        &endpoint_scope,
    )
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
        .with_api_key(api_key);'''
remote, count = re.subn(openai_secret_pattern, openai_secret_replacement, remote, count=1, flags=re.S)
if count != 1:
    raise SystemExit(f"remote_planner.rs: OpenAI secret block count {count}")
remote = remote.replace(
    "openai_config.with_org_id(resolve_secret_ref(organization).map_err(|reason| {",
    '''openai_config.with_org_id(resolve_secret_ref_for_endpoint(
                organization,
                "planner",
                profile_name,
                &endpoint_scope,
            ).map_err(|reason| {''',
    1,
)
remote = remote.replace(
    "    let client = Client::with_config(openai_config);",
    "    let client = Client::with_config(openai_config).with_http_client(http_client);",
    1,
)
remote = remote.replace(
    '''fn resolve_with_openai_planner(
    _profile: &RemotePlannerProfile,''',
    '''fn resolve_with_openai_planner(
    _profile_name: &str,
    _profile: &RemotePlannerProfile,''',
    1,
)

remote = remote.replace(
    '''fn resolve_with_ollama_planner(
    profile: &RemotePlannerProfile,
    planner_input: &PlannerInput,''',
    '''fn resolve_with_ollama_planner(
    profile_name: &str,
    profile: &RemotePlannerProfile,
    planner_input: &PlannerInput,''',
    1,
)
ollama_secret_pattern = r'''    let api_key = resolve_secret_ref\(&profile\.api_key\)\.map_err\(\|reason\| \{.*?    \}\)\?;

    let client = Client::with_config\(
        OpenAIConfig::new\(\)
            \.with_api_base\(profile\.base_url\.clone\(\)\)
            \.with_api_key\(api_key\),
    \);'''
ollama_secret_replacement = r'''    let endpoint_scope = ProviderEndpointScope::parse(&profile.base_url).map_err(|reason| {
        planner_interpretation_unavailable_error(
            "planner_endpoint_invalid",
            "Ollama planner endpoint is invalid",
            false,
            Some(serde_json::json!({ "reason": reason })),
        )
    })?;
    let api_key = resolve_secret_ref_for_endpoint(
        &profile.api_key,
        "planner",
        profile_name,
        &endpoint_scope,
    )
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
    .with_http_client(http_client);'''
remote, count = re.subn(ollama_secret_pattern, ollama_secret_replacement, remote, count=1, flags=re.S)
if count != 1:
    raise SystemExit(f"remote_planner.rs: Ollama secret block count {count}")
remote = remote.replace(
    '''fn resolve_with_ollama_planner(
    _profile: &RemotePlannerProfile,''',
    '''fn resolve_with_ollama_planner(
    _profile_name: &str,
    _profile: &RemotePlannerProfile,''',
    1,
)
write("src-tauri/src/app_core/remote_planner.rs", remote)

tts = read("src-tauri/src/tts/remote.rs")
tts = tts.replace(
    '''use crate::config::{
    resolve_secret_ref, RemoteProviderKind, RemoteTtsAudioFormat, RemoteTtsProfile,
};''',
    '''use crate::config::{
    resolve_secret_ref_for_endpoint, RemoteProviderKind, RemoteTtsAudioFormat, RemoteTtsProfile,
};
#[cfg(feature = "remote-openai")]
use crate::provider_endpoint::ProviderEndpointScope;''',
    1,
)
tts = tts.replace(
    '''                    self.synthesize_with_openai_remote(profile, runtime_audio, text)''',
    '''                    self.synthesize_with_openai_remote(
                        profile_name,
                        profile,
                        runtime_audio,
                        text,
                    )''',
    1,
)
tts = tts.replace(
    '''    fn synthesize_with_openai_remote(
        &mut self,
        profile: &RemoteTtsProfile,''',
    '''    fn synthesize_with_openai_remote(
        &mut self,
        profile_name: &str,
        profile: &RemoteTtsProfile,''',
    1,
)
tts = tts.replace(
    '''        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_millis(profile.timeout_ms.max(1)))
            .build()''',
    '''        let endpoint_scope = ProviderEndpointScope::parse(&profile.base_url)
            .map_err(|reason| TtsRuntimeError::RemoteRequestBuildFailed { reason })?;
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_millis(profile.timeout_ms.max(1)))
            .redirect(reqwest::redirect::Policy::none())
            .build()''',
    1,
)
tts = tts.replace(
    '''        let api_key = resolve_secret_ref(&profile.api_key)
            .map_err(|reason| TtsRuntimeError::RemoteSecretUnavailable { reason })?;''',
    '''        let api_key = resolve_secret_ref_for_endpoint(
            &profile.api_key,
            "tts",
            profile_name,
            &endpoint_scope,
        )
        .map_err(|reason| TtsRuntimeError::RemoteSecretUnavailable { reason })?;''',
    1,
)
tts = tts.replace(
    '''        let endpoint = format!("{}/audio/speech", profile.base_url.trim_end_matches('/'));''',
    '''        let endpoint = endpoint_scope
            .endpoint_url("audio/speech")
            .map_err(|reason| TtsRuntimeError::RemoteRequestBuildFailed { reason })?;''',
    1,
)
tts = tts.replace(
    '''                resolve_secret_ref(organization)
                    .map_err(|reason| TtsRuntimeError::RemoteSecretUnavailable { reason })?,''',
    '''                resolve_secret_ref_for_endpoint(
                    organization,
                    "tts",
                    profile_name,
                    &endpoint_scope,
                )
                .map_err(|reason| TtsRuntimeError::RemoteSecretUnavailable { reason })?,''',
    1,
)
write("src-tauri/src/tts/remote.rs", tts)

asr = read("src-tauri/src/asr/remote.rs")
asr = asr.replace(
    '#[cfg(feature = "remote-openai")]\nuse crate::config::resolve_secret_ref;',
    '#[cfg(feature = "remote-openai")]\nuse crate::config::resolve_secret_ref_for_endpoint;\n#[cfg(feature = "remote-openai")]\nuse crate::provider_endpoint::ProviderEndpointScope;',
    1,
)
asr = asr.replace(
    "        RemoteProviderKind::OpenAi => transcribe_with_openai_remote(profile, captured_audio),",
    '''        RemoteProviderKind::OpenAi => {
            transcribe_with_openai_remote(profile_name, profile, captured_audio)
        }''',
    1,
)
asr = asr.replace(
    '''fn transcribe_with_openai_remote(
    profile: &RemoteAsrProfile,''',
    '''fn transcribe_with_openai_remote(
    profile_name: &str,
    profile: &RemoteAsrProfile,''',
    1,
)
asr = asr.replace(
    '''    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()''',
    '''    let endpoint_scope = ProviderEndpointScope::parse(&profile.base_url)
        .map_err(|reason| AsrRuntimeError::RemoteRequestBuildFailed { reason })?;
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .redirect(reqwest::redirect::Policy::none())
        .build()''',
    1,
)
asr = asr.replace(
    '''    let api_key = resolve_secret_ref(&profile.api_key)
        .map_err(|reason| AsrRuntimeError::RemoteSecretUnavailable { reason })?;''',
    '''    let api_key = resolve_secret_ref_for_endpoint(
        &profile.api_key,
        "asr",
        profile_name,
        &endpoint_scope,
    )
    .map_err(|reason| AsrRuntimeError::RemoteSecretUnavailable { reason })?;''',
    1,
)
asr = asr.replace(
    '''    let endpoint = format!(
        "{}/audio/transcriptions",
        profile.base_url.trim_end_matches('/')
    );''',
    '''    let endpoint = endpoint_scope
        .endpoint_url("audio/transcriptions")
        .map_err(|reason| AsrRuntimeError::RemoteRequestBuildFailed { reason })?;''',
    1,
)
asr = asr.replace(
    '''            resolve_secret_ref(organization)
                .map_err(|reason| AsrRuntimeError::RemoteSecretUnavailable { reason })?,''',
    '''            resolve_secret_ref_for_endpoint(
                organization,
                "asr",
                profile_name,
                &endpoint_scope,
            )
            .map_err(|reason| AsrRuntimeError::RemoteSecretUnavailable { reason })?,''',
    1,
)
asr = asr.replace(
    '''fn transcribe_with_openai_remote(
    _profile: &RemoteAsrProfile,''',
    '''fn transcribe_with_openai_remote(
    _profile_name: &str,
    _profile: &RemoteAsrProfile,''',
    1,
)
write("src-tauri/src/asr/remote.rs", asr)

print("BBCR-004 credential-bearing runtime clients hardened")
