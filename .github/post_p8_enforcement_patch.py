#!/usr/bin/env python3
from __future__ import annotations

import json
import re
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, content: str) -> None:
    (ROOT / path).write_text(content, encoding="utf-8")


def replace_once(path: str, old: str, new: str) -> None:
    content = read(path)
    count = content.count(old)
    if count != 1:
        raise RuntimeError(f"{path}: expected one match, found {count}: {old[:100]!r}")
    write(path, content.replace(old, new, 1))


# Typed, first-class skill discovery diagnostics.
replace_once(
    "src-tauri/src/commands/registry.rs",
    """#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannerSkillSelection {
    pub active_skill_names: Vec<String>,
    pub relevant_skill_summaries: Vec<SkillSummary>,
}
""",
    """#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannerSkillSelection {
    pub active_skill_names: Vec<String>,
    pub relevant_skill_summaries: Vec<SkillSummary>,
    pub diagnostics: SkillDiscoveryDiagnostics,
}
""",
)
replace_once(
    "src-tauri/src/commands/registry.rs",
    """#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkillSource {
    Project,
    User,
    Bundled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedSkill {
""",
    """#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkillSource {
    Project,
    User,
    Bundled,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SkillLoadWarning {
    pub source: String,
    pub code: String,
    pub count: usize,
    pub skill: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SkillDiscoveryDiagnostics {
    pub warnings: Vec<SkillLoadWarning>,
}

impl SkillDiscoveryDiagnostics {
    pub(crate) fn push(&mut self, source: &str, code: &str, count: usize, skill: Option<String>) {
        if count == 0 {
            return;
        }
        if let Some(existing) = self.warnings.iter_mut().find(|warning| {
            warning.source == source && warning.code == code && warning.skill == skill
        }) {
            existing.count = existing.count.saturating_add(count);
            return;
        }
        self.warnings.push(SkillLoadWarning {
            source: source.to_string(),
            code: code.to_string(),
            count,
            skill,
        });
    }

    pub(crate) fn extend(&mut self, other: Self) {
        for warning in other.warnings {
            self.push(&warning.source, &warning.code, warning.count, warning.skill);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiscoveredSkills {
    pub(crate) skills: Vec<LoadedSkill>,
    pub(crate) diagnostics: SkillDiscoveryDiagnostics,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedSkill {
""",
)
replace_once(
    "src-tauri/src/commands/registry.rs",
    """    let loaded_skills = discover_skills(project_root, user_skill_root, available_tools);
    let mut active_skill_names = loaded_skills
""",
    """    let discovery = discover_skills(project_root, user_skill_root, available_tools);
    let diagnostics = discovery.diagnostics;
    let loaded_skills = discovery.skills;
    let mut active_skill_names = loaded_skills
""",
)
replace_once(
    "src-tauri/src/commands/registry.rs",
    """    PlannerSkillSelection {
        active_skill_names,
        relevant_skill_summaries,
    }
""",
    """    PlannerSkillSelection {
        active_skill_names,
        relevant_skill_summaries,
        diagnostics,
    }
""",
)

replace_once(
    "src-tauri/src/commands/skill_loader.rs",
    ") -> Vec<LoadedSkill> {\n",
    ") -> DiscoveredSkills {\n",
)
replace_once(
    "src-tauri/src/commands/skill_loader.rs",
    """    let mut discovered = HashMap::<String, LoadedSkill>::new();

    if let Some(project_root) = project_root {
""",
    """    let mut discovered = HashMap::<String, LoadedSkill>::new();
    let mut diagnostics = SkillDiscoveryDiagnostics::default();

    if let Some(project_root) = project_root {
""",
)
for _ in range(2):
    replace_once(
        "src-tauri/src/commands/skill_loader.rs",
        """            &available_tool_names,
            &mut discovered,
        );
""",
        """            &available_tool_names,
            &mut discovered,
            &mut diagnostics,
        );
""",
    )
replace_once(
    "src-tauri/src/commands/skill_loader.rs",
    """    discovered.into_values().collect()
}
""",
    """    DiscoveredSkills {
        skills: discovered.into_values().collect(),
        diagnostics,
    }
}
""",
)
replace_once(
    "src-tauri/src/commands/skill_loader.rs",
    """    available_tool_names: &[ToolName],
    discovered: &mut HashMap<String, LoadedSkill>,
) {
""",
    """    available_tool_names: &[ToolName],
    discovered: &mut HashMap<String, LoadedSkill>,
    diagnostics: &mut SkillDiscoveryDiagnostics,
) {
""",
)
replace_once(
    "src-tauri/src/commands/skill_loader.rs",
    """        Err(error) => {
            tracing::warn!(
                source = source_label,
                error_kind = ?error.kind(),
                "failed to read skill directory"
            );
            return;
        }
""",
    """        Err(error) => {
            diagnostics.push(source_label, "directory_unreadable", 1, None);
            tracing::warn!(
                source = source_label,
                error_kind = ?error.kind(),
                "failed to read skill directory"
            );
            return;
        }
""",
)
replace_once(
    "src-tauri/src/commands/skill_loader.rs",
    """    if entry_warnings.skipped_entries > 0 {
        tracing::warn!(
            source = source_label,
            skipped_entries = entry_warnings.skipped_entries,
            error_categories = ?entry_warnings.error_categories,
            "skipped unreadable skill directory entries"
        );
    }
""",
    """    if entry_warnings.skipped_entries > 0 {
        for (category, count) in &entry_warnings.error_categories {
            diagnostics.push(source_label, category, *count, None);
        }
        tracing::warn!(
            source = source_label,
            skipped_entries = entry_warnings.skipped_entries,
            error_categories = ?entry_warnings.error_categories,
            "skipped unreadable skill directory entries"
        );
    }
""",
)
replace_once(
    "src-tauri/src/commands/skill_loader.rs",
    """            Err(error) => {
                tracing::warn!(
                    source = source_label,
                    skill = %directory_name,
                    error_kind = ?error.kind(),
                    "failed to read SKILL.md"
                );
                continue;
            }
""",
    """            Err(error) => {
                diagnostics.push(
                    source_label,
                    "manifest_unreadable",
                    1,
                    Some(directory_name.clone()),
                );
                tracing::warn!(
                    source = source_label,
                    skill = %directory_name,
                    error_kind = ?error.kind(),
                    "failed to read SKILL.md"
                );
                continue;
            }
""",
)
replace_once(
    "src-tauri/src/commands/skill_loader.rs",
    """                if directory_name != skill.summary.name {
                    tracing::warn!(
                        source = source_label,
                        expected = %directory_name,
                        actual = %skill.summary.name,
                        "skipping skill because directory name does not match frontmatter name"
                    );
                    continue;
                }
""",
    """                if directory_name != skill.summary.name {
                    diagnostics.push(
                        source_label,
                        "name_mismatch",
                        1,
                        Some(directory_name.clone()),
                    );
                    tracing::warn!(
                        source = source_label,
                        expected = %directory_name,
                        actual = %skill.summary.name,
                        "skipping skill because directory name does not match frontmatter name"
                    );
                    continue;
                }
""",
)
replace_once(
    "src-tauri/src/commands/skill_loader.rs",
    """            Err(error) => {
                tracing::warn!(
                    source = source_label,
                    skill = %directory_name,
                    error = %error,
                    "skipping invalid skill document"
                );
            }
""",
    """            Err(_error) => {
                diagnostics.push(
                    source_label,
                    "invalid_manifest",
                    1,
                    Some(directory_name.clone()),
                );
                tracing::warn!(
                    source = source_label,
                    skill = %directory_name,
                    error_category = "invalid_manifest",
                    "skipping invalid skill document"
                );
            }
""",
)
replace_once(
    "src-tauri/src/commands/skill_loader.rs",
    """    use super::{collect_readable_entries, skill_directory_label, skill_source_label, SkillSource};
""",
    """    use super::{
        collect_readable_entries, skill_directory_label, skill_source_label,
        SkillDiscoveryDiagnostics, SkillSource,
    };
""",
)
replace_once(
    "src-tauri/src/commands/skill_loader.rs",
    """    fn unreadable_entries_are_aggregated_without_dropping_valid_neighbors() {
""",
    """    fn typed_skill_diagnostics_merge_without_private_paths() {
        let mut diagnostics = SkillDiscoveryDiagnostics::default();
        diagnostics.push("project", "permission_denied", 1, None);
        diagnostics.push("project", "permission_denied", 2, None);
        diagnostics.push(
            "user",
            "invalid_manifest",
            1,
            Some(String::from("navigation")),
        );
        assert_eq!(diagnostics.warnings.len(), 2);
        assert_eq!(diagnostics.warnings[0].count, 3);
        let encoded = serde_json::to_string(&diagnostics).expect("diagnostics serialize");
        assert!(!encoded.contains("/home/"));
        assert!(!encoded.contains("private-user"));
    }

    #[test]
    fn unreadable_entries_are_aggregated_without_dropping_valid_neighbors() {
""",
)

# Store diagnostics and expose them through runtime status.
replace_once(
    "src-tauri/src/app_core/mod.rs",
    "use crate::commands::{ToolError, ToolName, ToolResult};\n",
    "use crate::commands::{SkillDiscoveryDiagnostics, ToolError, ToolName, ToolResult};\n",
)
replace_once(
    "src-tauri/src/app_core/mod.rs",
    """    recent_field_context: Option<RecentFieldContext>,
    image_cache: ImageCache,
""",
    """    recent_field_context: Option<RecentFieldContext>,
    last_skill_discovery_diagnostics: SkillDiscoveryDiagnostics,
    image_cache: ImageCache,
""",
)
replace_once(
    "src-tauri/src/app_core/mod.rs",
    """            recent_field_context: None,
            image_cache: ImageCache::default(),
""",
    """            recent_field_context: None,
            last_skill_discovery_diagnostics: SkillDiscoveryDiagnostics::default(),
            image_cache: ImageCache::default(),
""",
)
replace_once(
    "src-tauri/src/app_core/command_dispatch.rs",
    """    PlannerOutput, PlannerSafetySettings, PlannerToolHistoryEntry, SerializedToolResult, ToolError,
    ToolName,
""",
    """    PlannerOutput, PlannerSafetySettings, PlannerToolHistoryEntry, SerializedToolResult,
    SkillDiscoveryDiagnostics, ToolError, ToolName,
""",
)
replace_once(
    "src-tauri/src/app_core/command_dispatch.rs",
    """        let available_tools = planner_available_tools();
        let planner_safety = PlannerSafetySettings::from(&self.config.safety);
        let current_dir = std::env::current_dir().ok();
        let user_skill_root = self
            .app_handle
            .path()
            .app_config_dir()
            .ok()
            .map(|path| path.join("skills"));
        let skill_selection = build_planner_skill_selection(
            current_dir.as_deref(),
            user_skill_root.as_deref(),
            transcript,
            &available_tools,
        );
""",
    """        let available_tools = planner_available_tools();
        let planner_safety = PlannerSafetySettings::from(&self.config.safety);
        let mut context_diagnostics = SkillDiscoveryDiagnostics::default();
        let current_dir = match std::env::current_dir() {
            Ok(path) => Some(path),
            Err(_) => {
                context_diagnostics.push("project", "project_root_unavailable", 1, None);
                None
            }
        };
        let user_skill_root = match self.app_handle.path().app_config_dir() {
            Ok(path) => Some(path.join("skills")),
            Err(_) => {
                context_diagnostics.push("user", "user_skill_root_unavailable", 1, None);
                None
            }
        };
        let mut skill_selection = build_planner_skill_selection(
            current_dir.as_deref(),
            user_skill_root.as_deref(),
            transcript,
            &available_tools,
        );
        skill_selection.diagnostics.extend(context_diagnostics);
        self.last_skill_discovery_diagnostics = skill_selection.diagnostics.clone();
""",
)
replace_once(
    "src-tauri/src/commands/contracts/providers.rs",
    """    pub pending_plan_execution: Option<PendingPlanExecutionState>,
    pub provider_modes: Option<ProviderSelectionStatus>,
}
""",
    """    pub pending_plan_execution: Option<PendingPlanExecutionState>,
    pub provider_modes: Option<ProviderSelectionStatus>,
    pub skill_discovery_diagnostics: SkillDiscoveryDiagnostics,
}
""",
)
replace_once(
    "src-tauri/src/app_core/state_snapshots.rs",
    """            } else {
                None
            },
        }
""",
    """            } else {
                None
            },
            skill_discovery_diagnostics: self.last_skill_discovery_diagnostics.clone(),
        }
""",
)
replace_once(
    "src/tauri-types.ts",
    """export interface ProviderSelectionStatus {
  planner_mode: ProviderMode;
  tts_mode: ProviderMode;
  asr_mode: ProviderMode;
}
""",
    """export interface ProviderSelectionStatus {
  planner_mode: ProviderMode;
  tts_mode: ProviderMode;
  asr_mode: ProviderMode;
}

export interface SkillLoadWarning {
  source: string;
  code: string;
  count: number;
  skill: string | null;
}

export interface SkillDiscoveryDiagnostics {
  warnings: SkillLoadWarning[];
}
""",
)

# Direct focus query construction cannot silently disappear.
replace_once(
    "src-tauri/src/app_core/form_fill/field_focus.rs",
    "    let search_query = build_find_element_query(&query).ok()?;\n",
    """    let search_query = match build_find_element_query(&query) {
        Ok(search_query) => search_query,
        Err(_) => {
            let summary = String::from(
                "I could not build a safe field-search query. Please name the field more specifically.",
            );
            return Some(build_direct_follow_up_output(
                request_id,
                DirectFollowUpSpec {
                    intent_name: IntentName::FillInput,
                    goal: String::from("Focus the requested field."),
                    target_description: Some(description),
                    selected_skills,
                    summary,
                    next_recommended_action: Some(String::from(
                        "Use the visible field label or placeholder.",
                    )),
                    step_id: String::from("focus-query-construction-failed"),
                    purpose: String::from(
                        "Report a bounded deterministic field-query construction failure.",
                    ),
                },
            ));
        }
    };
""",
)
field_focus = read("src-tauri/src/app_core/form_fill/field_focus.rs")
field_focus += """

#[cfg(test)]
mod post_p8_enforcement_tests {
    use super::*;
    use crate::commands::{ReportStatus, ToolName};

    #[test]
    fn focus_query_failure_follow_up_is_non_authorizing() {
        let output = build_direct_follow_up_output(
            "req-1",
            DirectFollowUpSpec {
                intent_name: IntentName::FillInput,
                goal: String::from("Focus the requested field."),
                target_description: Some(String::from("email")),
                selected_skills: Vec::new(),
                summary: String::from("I could not build a safe field-search query."),
                next_recommended_action: Some(String::from("Use the visible field label.")),
                step_id: String::from("focus-query-construction-failed"),
                purpose: String::from("Report a bounded field-query construction failure."),
            },
        );
        assert_eq!(output.steps.len(), 1);
        assert_eq!(output.steps[0].tool_name, ToolName::ReportResult);
        assert_eq!(
            output.steps[0].arguments.get("status"),
            Some(&serde_json::json!(ReportStatus::NeedsFollowUp))
        );
        assert!(!output.steps.iter().any(|step| matches!(
            step.tool_name,
            ToolName::FocusElement | ToolName::TypeIntoElement | ToolName::SubmitActiveForm
        )));
    }
}
"""
write("src-tauri/src/app_core/form_fill/field_focus.rs", field_focus)

# Fill correction keeps explicit follow-up behavior without Result::ok omission.
replace_once(
    "src-tauri/src/app_core/fill_correction.rs",
    """                .and_then(|candidate_id| {
                    resolve_typeable_element(current_page, candidate_id)
                        .ok()
                        .map(|_| candidate_id.clone())
                });
""",
    """                .find(|candidate_id| {
                    resolve_typeable_element(current_page, candidate_id).is_ok()
                })
                .cloned();
""",
)

# Remote TTS/ASR typed absence and sanitized endpoint parity.
replace_once(
    "src-tauri/src/commands/contracts/providers.rs",
    """    pub audio_format: Option<RemoteTtsAudioFormat>,
    pub timeout_ms: Option<u64>,
}
""",
    """    pub audio_format: Option<RemoteTtsAudioFormat>,
    pub timeout_ms: Option<u64>,
    pub endpoint_is_loopback: Option<bool>,
    pub availability_reason: Option<CapabilityAbsenceReason>,
}
""",
)
replace_once(
    "src-tauri/src/commands/contracts/providers.rs",
    """    pub temperature_milli: Option<u16>,
    pub timeout_ms: Option<u64>,
}
""",
    """    pub temperature_milli: Option<u16>,
    pub timeout_ms: Option<u64>,
    pub endpoint_is_loopback: Option<bool>,
    pub availability_reason: Option<CapabilityAbsenceReason>,
}
""",
)
replace_once(
    "src-tauri/src/app_core/settings_adapters.rs",
    "fn remote_provider_label(provider: &RemoteProviderKind) -> RemoteProviderLabel {\n",
    """fn remote_endpoint_status(
    profile_name: Option<&String>,
    base_url: Option<&str>,
    secret_error: Option<&String>,
) -> (Option<String>, Option<bool>, Option<CapabilityAbsenceReason>) {
    let Some(_) = profile_name else {
        return (None, None, Some(CapabilityAbsenceReason::NotConfigured));
    };
    let Some(base_url) = base_url else {
        return (None, None, Some(CapabilityAbsenceReason::ProfileMissing));
    };
    match ProviderEndpointScope::parse(base_url) {
        Ok(scope) => (
            Some(scope.normalized_base_url().to_string()),
            Some(scope.is_loopback()),
            secret_error
                .is_some()
                .then_some(CapabilityAbsenceReason::CredentialReferenceMissing),
        ),
        Err(_) => (
            sanitize_url_for_display(base_url)
                .map(|safe| safe.value)
                .or_else(|| Some(String::from("[REDACTED INVALID ENDPOINT]"))),
            None,
            Some(CapabilityAbsenceReason::InvalidEndpoint),
        ),
    }
}

fn remote_provider_label(provider: &RemoteProviderKind) -> RemoteProviderLabel {
""",
)
settings = read("src-tauri/src/app_core/settings_adapters.rs")
pattern = re.compile(r"pub\(crate\) fn build_remote_tts_settings\(config: &AppConfig\) -> RemoteTtsSettings \{.*?\n\}\n\npub\(crate\) fn build_remote_asr_settings", re.S)
replacement = """pub(crate) fn build_remote_tts_settings(config: &AppConfig) -> RemoteTtsSettings {
    let profile_name = config.providers.tts.remote_profile.clone();
    let profile = profile_name
        .as_ref()
        .and_then(|configured_profile| config.remote_tts_profiles.get(configured_profile));
    let (api_key_masked_value, api_key_reference_error) = profile
        .map(|p| masked_secret_status(&p.api_key))
        .unwrap_or((None, None));
    let (base_url, endpoint_is_loopback, availability_reason) = remote_endpoint_status(
        profile_name.as_ref(),
        profile.map(|configured_profile| configured_profile.base_url.as_str()),
        api_key_reference_error.as_ref(),
    );

    RemoteTtsSettings {
        profile_name,
        provider: profile.map(|configured_profile| remote_provider_label(&configured_profile.provider)),
        base_url,
        model: profile.map(|configured_profile| configured_profile.model.clone()),
        api_key_reference: profile.map(|configured_profile| secret_ref_reference(&configured_profile.api_key)),
        api_key_masked_value,
        api_key_reference_error,
        organization_reference: profile
            .and_then(|configured_profile| configured_profile.organization.as_ref())
            .map(secret_ref_reference),
        project: profile.and_then(|configured_profile| configured_profile.project.clone()),
        voice: profile.map(|configured_profile| configured_profile.voice.clone()),
        audio_format: profile.map(|configured_profile| configured_profile.audio_format.clone()),
        timeout_ms: profile.map(|configured_profile| configured_profile.timeout_ms),
        endpoint_is_loopback,
        availability_reason,
    }
}

pub(crate) fn build_remote_asr_settings"""
settings, count = pattern.subn(replacement, settings)
if count != 1:
    raise RuntimeError(f"remote TTS replacement count {count}")
write("src-tauri/src/app_core/settings_adapters.rs", settings)
settings = read("src-tauri/src/app_core/settings_adapters.rs")
pattern = re.compile(r"pub\(crate\) fn build_remote_asr_settings\(config: &AppConfig\) -> RemoteAsrSettings \{.*?\n\}\n\npub\(crate\) fn build_provider_failover_settings", re.S)
replacement = """pub(crate) fn build_remote_asr_settings(config: &AppConfig) -> RemoteAsrSettings {
    let profile_name = config.providers.asr.remote_profile.clone();
    let profile = profile_name
        .as_ref()
        .and_then(|configured_profile| config.remote_asr_profiles.get(configured_profile));
    let (api_key_masked_value, api_key_reference_error) = profile
        .map(|p| masked_secret_status(&p.api_key))
        .unwrap_or((None, None));
    let (base_url, endpoint_is_loopback, availability_reason) = remote_endpoint_status(
        profile_name.as_ref(),
        profile.map(|configured_profile| configured_profile.base_url.as_str()),
        api_key_reference_error.as_ref(),
    );

    RemoteAsrSettings {
        profile_name,
        provider: profile.map(|configured_profile| remote_provider_label(&configured_profile.provider)),
        base_url,
        model: profile.map(|configured_profile| configured_profile.model.clone()),
        api_key_reference: profile.map(|configured_profile| secret_ref_reference(&configured_profile.api_key)),
        api_key_masked_value,
        api_key_reference_error,
        organization_reference: profile
            .and_then(|configured_profile| configured_profile.organization.as_ref())
            .map(secret_ref_reference),
        project: profile.and_then(|configured_profile| configured_profile.project.clone()),
        language: profile.and_then(|configured_profile| configured_profile.language.clone()),
        temperature_milli: profile.map(|configured_profile| configured_profile.temperature_milli),
        timeout_ms: profile.map(|configured_profile| configured_profile.timeout_ms),
        endpoint_is_loopback,
        availability_reason,
    }
}

pub(crate) fn build_provider_failover_settings"""
settings, count = pattern.subn(replacement, settings)
if count != 1:
    raise RuntimeError(f"remote ASR replacement count {count}")
write("src-tauri/src/app_core/settings_adapters.rs", settings)
replace_once(
    "src/tauri-types.ts",
    """  audio_format: RemoteTtsAudioFormat | null;
  timeout_ms: number | null;
}
""",
    """  audio_format: RemoteTtsAudioFormat | null;
  timeout_ms: number | null;
  endpoint_is_loopback: boolean | null;
  availability_reason: CapabilityAbsenceReason | null;
}
""",
)
replace_once(
    "src/tauri-types.ts",
    """  temperature_milli: number | null;
  timeout_ms: number | null;
}
""",
    """  temperature_milli: number | null;
  timeout_ms: number | null;
  endpoint_is_loopback: boolean | null;
  availability_reason: CapabilityAbsenceReason | null;
}
""",
)
settings_tests = read("src-tauri/src/app_core/tests/settings_tests.rs")
settings_tests += """

#[test]
fn post_p8_enforcement_remote_tts_asr_surface_typed_absence_and_sanitized_urls() {
    use crate::commands::CapabilityAbsenceReason;

    let mut config = AppConfig::default();
    config.remote_tts_profiles.get_mut("openai-tts-default").expect("TTS profile").base_url =
        String::from("https://user:pass@tts.example.com:8443/v1?token=secret#fragment");
    config.remote_asr_profiles.get_mut("openai-transcribe-default").expect("ASR profile").base_url =
        String::from("https://user:pass@asr.example.com:9443/v1?code=secret#fragment");
    let tts = build_remote_tts_settings(&config);
    let asr = build_remote_asr_settings(&config);
    assert_eq!(tts.availability_reason, Some(CapabilityAbsenceReason::InvalidEndpoint));
    assert_eq!(asr.availability_reason, Some(CapabilityAbsenceReason::InvalidEndpoint));
    assert_eq!(tts.base_url.as_deref(), Some("https://tts.example.com:8443/v1"));
    assert_eq!(asr.base_url.as_deref(), Some("https://asr.example.com:9443/v1"));
    for displayed in [tts.base_url.as_deref().unwrap(), asr.base_url.as_deref().unwrap()] {
        assert!(!displayed.contains("user"));
        assert!(!displayed.contains("pass"));
        assert!(!displayed.contains('?'));
        assert!(!displayed.contains('#'));
    }

    let mut none = AppConfig::default();
    none.providers.tts.remote_profile = None;
    none.providers.asr.remote_profile = None;
    assert_eq!(build_remote_tts_settings(&none).availability_reason, Some(CapabilityAbsenceReason::NotConfigured));
    assert_eq!(build_remote_asr_settings(&none).availability_reason, Some(CapabilityAbsenceReason::NotConfigured));

    let mut missing = AppConfig::default();
    missing.providers.tts.remote_profile = Some(String::from("missing-tts"));
    missing.providers.asr.remote_profile = Some(String::from("missing-asr"));
    assert_eq!(build_remote_tts_settings(&missing).availability_reason, Some(CapabilityAbsenceReason::ProfileMissing));
    assert_eq!(build_remote_asr_settings(&missing).availability_reason, Some(CapabilityAbsenceReason::ProfileMissing));
}
"""
write("src-tauri/src/app_core/tests/settings_tests.rs", settings_tests)

# Embedded URLs in prose are sanitized individually.
replace_once(
    "src-tauri/src/diagnostic_redaction.rs",
    "    redact_url_query(value)\n}\n",
    "    redact_embedded_urls(value)\n}\n",
)
replace_once(
    "src-tauri/src/diagnostic_redaction.rs",
    """fn redact_url_query(value: &str) -> String {
    match sanitize_url_for_display(value) {
        Some(safe) => safe.value,
        None if value.contains("://") => String::from("[REDACTED INVALID URL]"),
        None => value.to_string(),
    }
}
""",
    """fn redact_embedded_urls(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut remaining = value;
    loop {
        let start = match (remaining.find("http://"), remaining.find("https://")) {
            (Some(left), Some(right)) => left.min(right),
            (Some(index), None) | (None, Some(index)) => index,
            (None, None) => {
                output.push_str(remaining);
                break;
            }
        };
        output.push_str(&remaining[..start]);
        let candidate_and_tail = &remaining[start..];
        let end = candidate_and_tail
            .find(char::is_whitespace)
            .unwrap_or(candidate_and_tail.len());
        let raw_token = &candidate_and_tail[..end];
        let trimmed = raw_token.trim_end_matches([
            ',', '.', ';', '!', '?', ')', ']', '}', '"', '\'',
        ]);
        let trailing = &raw_token[trimmed.len()..];
        match sanitize_url_for_display(trimmed) {
            Some(safe) => output.push_str(&safe.value),
            None => output.push_str("[REDACTED INVALID URL]"),
        }
        output.push_str(trailing);
        remaining = &candidate_and_tail[end..];
    }
    output
}
""",
)
replace_once(
    "src-tauri/src/diagnostic_redaction.rs",
    """    fn reconstructs_urls_without_userinfo_query_or_fragment() {
""",
    """    fn redacts_embedded_urls_in_prose_and_json() {
        let prose = "failed https://user:pass@example.com/callback?code=abc&state=xyz#frag then https://cdn.example.com/file?signature=signed";
        let safe = redact_diagnostic_text(prose);
        assert!(safe.contains("failed https://example.com/callback"));
        assert!(safe.contains("then https://cdn.example.com/file"));
        for secret in ["user", "pass", "code=abc", "state=xyz", "signature=signed", "#frag"] {
            assert!(!safe.contains(secret));
        }
        let nested = serde_json::json!({
            "message": "callback https://example.com/cb?access_token=secret",
            "items": ["signed https://example.com/a?sig=secret"]
        });
        let encoded = redact_json_value(&nested).to_string();
        assert!(!encoded.contains("access_token"));
        assert!(!encoded.contains("sig=secret"));
        assert_eq!(redact_diagnostic_text("plain text"), "plain text");
        assert!(redact_diagnostic_text("broken https://[invalid?token=x")
            .contains("[REDACTED INVALID URL]"));
    }

    #[test]
    fn reconstructs_urls_without_userinfo_query_or_fragment() {
""",
)

# Provider handlers cannot report successful empty settings.
replace_once(
    "src-tauri/src/command_handlers/provider_handlers.rs",
    """pub struct SetTtsModelSelectionData {
    profile_name: String,
    changed: bool,
}
""",
    """pub struct SetTtsModelSelectionData {
    profile_name: String,
    changed: bool,
}

fn completed_remote_planner_connection_settings(
    profile_name: String,
    settings: crate::commands::RemotePlannerSettings,
) -> Result<RemotePlannerConnectionSettingsData, ToolError> {
    let base_url = settings
        .base_url
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| ToolError {
            code: String::from("remote_planner_settings_inconsistent"),
            message: String::from("Persisted remote planner settings did not produce a usable sanitized endpoint."),
            retryable: false,
            details: Some(serde_json::json!({ "availability_reason": settings.availability_reason })),
        })?;
    let model = settings
        .model
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| ToolError {
            code: String::from("remote_planner_settings_inconsistent"),
            message: String::from("Persisted remote planner settings did not produce a configured model."),
            retryable: false,
            details: None,
        })?;
    Ok(RemotePlannerConnectionSettingsData { profile_name, base_url, model })
}
""",
)
for _ in range(2):
    replace_once(
        "src-tauri/src/command_handlers/provider_handlers.rs",
        """    let settings = app_core.current_remote_planner_settings();
    Ok(RemotePlannerConnectionSettingsData {
        profile_name,
        base_url: settings.base_url.unwrap_or_default(),
        model: settings.model.unwrap_or_default(),
    })
""",
        """    let settings = app_core.current_remote_planner_settings();
    completed_remote_planner_connection_settings(profile_name, settings)
""",
    )
provider_handlers = read("src-tauri/src/command_handlers/provider_handlers.rs")
provider_handlers += """

#[cfg(test)]
mod post_p8_enforcement_tests {
    use super::*;
    use crate::commands::{CapabilityAbsenceReason, RemotePlannerSettings};

    #[test]
    fn inconsistent_post_persist_settings_are_typed_failures() {
        let error = completed_remote_planner_connection_settings(
            String::from("profile"),
            RemotePlannerSettings {
                profile_name: Some(String::from("profile")),
                availability_reason: Some(CapabilityAbsenceReason::InvalidEndpoint),
                ..RemotePlannerSettings::default()
            },
        )
        .expect_err("missing endpoint/model must fail");
        assert_eq!(error.code, "remote_planner_settings_inconsistent");
    }

    #[test]
    fn complete_post_persist_settings_are_returned() {
        let result = completed_remote_planner_connection_settings(
            String::from("profile"),
            RemotePlannerSettings {
                profile_name: Some(String::from("profile")),
                base_url: Some(String::from("https://example.com/v1")),
                model: Some(String::from("model")),
                ..RemotePlannerSettings::default()
            },
        )
        .expect("complete settings");
        assert_eq!(result.base_url, "https://example.com/v1");
        assert_eq!(result.model, "model");
    }
}
"""
write("src-tauri/src/command_handlers/provider_handlers.rs", provider_handlers)

# External launch gets a typed semantic policy mapping.
replace_once(
    "src-tauri/src/direct_command_policy.rs",
    """pub(crate) enum DirectCommandArtifactPolicy {
    VerifiedAtomicActivation,
}
""",
    """pub(crate) enum DirectCommandArtifactPolicy {
    VerifiedAtomicActivation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DirectCommandExternalLaunchPolicy {
    ValidatedHttpUrlWithUserGesture,
}
""",
)
replace_once(
    "src-tauri/src/direct_command_policy.rs",
    "pub(crate) const fn direct_command_artifact_policy(\n",
    """pub(crate) const fn direct_command_external_launch_policy(
    name: DirectCommandName,
) -> Option<DirectCommandExternalLaunchPolicy> {
    match name {
        DirectCommandName::OpenExternalUrl => {
            Some(DirectCommandExternalLaunchPolicy::ValidatedHttpUrlWithUserGesture)
        }
        _ => None,
    }
}

pub(crate) const fn direct_command_artifact_policy(
""",
)
replace_once(
    "src-tauri/src/direct_command_policy.rs",
    """        assert_eq!(
            policy.downloads_executable_or_model_artifact,
            direct_command_artifact_policy(*name).is_some(),
            "artifact direct commands require verified activation mapping"
        );
""",
    """        assert_eq!(
            policy.downloads_executable_or_model_artifact,
            direct_command_artifact_policy(*name).is_some(),
            "artifact direct commands require verified activation mapping"
        );
        assert_eq!(
            policy.launches_external_program,
            direct_command_external_launch_policy(*name).is_some(),
            "external launch commands require validated URL/user-gesture mapping"
        );
""",
)
replace_once(
    "src-tauri/src/direct_command_policy.rs",
    """            assert_eq!(
                policy.downloads_executable_or_model_artifact,
                direct_command_artifact_policy(*name).is_some(),
                "{}",
                name.as_handler_name()
            );
""",
    """            assert_eq!(
                policy.downloads_executable_or_model_artifact,
                direct_command_artifact_policy(*name).is_some(),
                "{}",
                name.as_handler_name()
            );
            assert_eq!(
                policy.launches_external_program,
                direct_command_external_launch_policy(*name).is_some(),
                "{}",
                name.as_handler_name()
            );
""",
)
evidence = read("src-tauri/tests/post_batch8_direct_command_policy_evidence.rs")
evidence += """

#[test]
fn source_drift_external_launch_retains_validated_url_and_user_gesture_policy() {
    let handlers = source("src/command_handlers/url_handlers.rs");
    let frontend = source("../src/external-link.ts");
    let policy = source("src/direct_command_policy.rs");
    assert!(handlers.contains("validate_external_url"));
    assert!(frontend.contains("http:") || frontend.contains("https:"));
    assert!(policy.contains("ValidatedHttpUrlWithUserGesture"));
    assert!(policy.contains("requires_user_gesture"));
}
"""
write("src-tauri/tests/post_batch8_direct_command_policy_evidence.rs", evidence)

# Remove converted temporary fallback records and reclassify optional intent tags.
inventory_path = ROOT / "scripts/security-fallback-inventory.json"
payload = json.loads(inventory_path.read_text(encoding="utf-8"))
converted_pairs = {
    ("src-tauri/src/app_core/command_dispatch.rs", ".ok()"),
    ("src-tauri/src/app_core/command_dispatch.rs", "let current_dir = std::env::current_dir().ok();"),
    ("src-tauri/src/app_core/fill_correction.rs", ".ok()"),
    ("src-tauri/src/app_core/form_fill/field_focus.rs", "let search_query = build_find_element_query(&query).ok()?;"),
}
entries = [
    entry for entry in payload["entries"]
    if (entry["path"], entry["expression"]) not in converted_pairs
]
for entry in entries:
    if entry["path"] == "src-tauri/src/commands/skill_parser.rs" and entry["expression"] == ".unwrap_or_default()":
        entry["disposition"] = "permanent_accepted"
        entry["review_due"] = "not_applicable"
        entry["owner_note"] = (
            "Missing intent_tags is explicitly optional and only reduces skill matching; "
            "required descriptions, confirmation flags, names, and tool references remain validated."
        )
        entry["future_replacement"] = "Retain unless intent tags become mandatory in the public skill contract."

allowlist_path = ROOT / "scripts/security-fallback-allowlist.txt"
kept = []
for raw in allowlist_path.read_text(encoding="utf-8").splitlines():
    stripped = raw.strip()
    if not stripped or stripped.startswith("#"):
        kept.append(raw)
        continue
    path, expression = raw.split("|", 1)
    if (path, expression) not in converted_pairs:
        kept.append(raw)
allowlist_path.write_text("\n".join(kept).rstrip() + "\n", encoding="utf-8")

# Exact per-occurrence inventory identity.
def normalize(line: str) -> str:
    return " ".join(line.strip().split())

signature = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*[<(]")


def source_occurrences(path: str, expression: str) -> list[dict]:
    lines = (ROOT / path).read_text(encoding="utf-8", errors="replace").splitlines()
    current = "module scope"
    per_function: dict[str, int] = defaultdict(int)
    found_occurrences = []
    for index, line in enumerate(lines):
        match = signature.search(line)
        if match:
            current = match.group(1)
        if normalize(line) != expression:
            continue
        per_function[current] += 1
        before = next((normalize(value) for value in reversed(lines[:index]) if normalize(value)), "")
        after = next((normalize(value) for value in lines[index + 1:] if normalize(value)), "")
        found_occurrences.append({
            "function": current,
            "occurrence": per_function[current],
            "context_before": before,
            "context_after": after,
        })
    return found_occurrences

migrated = []
for entry in entries:
    occurrences = source_occurrences(entry["path"], entry["expression"])
    allowed_functions = set(entry.get("functions", []))
    occurrences = [item for item in occurrences if not allowed_functions or item["function"] in allowed_functions]
    if not occurrences:
        raise RuntimeError(f"no occurrence for {entry['path']}|{entry['expression']}")
    for occurrence in occurrences:
        migrated_entry = {key: value for key, value in entry.items() if key != "functions"}
        migrated_entry.update(occurrence)
        migrated.append(migrated_entry)
payload["version"] = 3
payload["identity"] = "path + function + normalized expression + occurrence index + adjacent normalized context"
payload["entries"] = migrated
inventory_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

# Occurrence-aware scanner.
scanner = '''#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ALLOWLIST = ROOT / "scripts/security-fallback-allowlist.txt"
INVENTORY = ROOT / "scripts/security-fallback-inventory.json"
DOCUMENTATION = ROOT / "docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md"
REQUIRED = {
    "path", "function", "expression", "occurrence", "context_before", "context_after",
    "justification", "user_visibility", "side_effect_impact", "test_coverage",
    "future_replacement", "disposition", "review_due", "owner_note",
}
VALID_DISPOSITIONS = {
    "permanent_accepted", "temporary_accepted", "convert_to_warning", "convert_to_error", "remove",
}
SIGNATURE = re.compile(r"\\bfn\\s+([A-Za-z_][A-Za-z0-9_]*)\\s*[<(]")


def normalize(line: str) -> str:
    return " ".join(line.strip().split())


def allowlist_keys() -> set[tuple[str, str]]:
    keys = set()
    for raw in ALLOWLIST.read_text(encoding="utf-8").splitlines():
        raw = raw.strip()
        if not raw or raw.startswith("#"):
            continue
        path, expression = raw.split("|", 1)
        keys.add((path, expression))
    return keys


def occurrences_from_lines(lines: list[str], expression: str) -> list[dict]:
    current = "module scope"
    per_function: dict[str, int] = defaultdict(int)
    occurrences = []
    for index, line in enumerate(lines):
        match = SIGNATURE.search(line)
        if match:
            current = match.group(1)
        if normalize(line) != expression:
            continue
        per_function[current] += 1
        before = next((normalize(value) for value in reversed(lines[:index]) if normalize(value)), "")
        after = next((normalize(value) for value in lines[index + 1:] if normalize(value)), "")
        occurrences.append({
            "function": current,
            "occurrence": per_function[current],
            "context_before": before,
            "context_after": after,
        })
    return occurrences


def source_occurrences(path: str, expression: str) -> list[dict]:
    return occurrences_from_lines(
        (ROOT / path).read_text(encoding="utf-8", errors="replace").splitlines(), expression
    )


def occurrence_key(entry: dict) -> tuple:
    return (entry.get("path"), entry.get("function"), entry.get("expression"), entry.get("occurrence"))


def metadata_problems(key: tuple, entry: dict) -> list[str]:
    problems = []
    missing = REQUIRED - set(entry)
    if missing:
        return [f"{key}: missing fields {sorted(missing)}"]
    for field in REQUIRED - {"occurrence"}:
        if not isinstance(entry[field], str) or not entry[field].strip():
            problems.append(f"{key}: empty {field}")
    if not isinstance(entry["occurrence"], int) or entry["occurrence"] < 1:
        problems.append(f"{key}: occurrence must be a positive integer")
    if entry["disposition"] not in VALID_DISPOSITIONS:
        problems.append(f"{key}: invalid disposition {entry['disposition']!r}")
    if entry["disposition"] == "temporary_accepted":
        if entry["review_due"] == "not_applicable":
            problems.append(f"{key}: temporary fallback requires a review boundary")
        if len(entry["owner_note"].strip()) < 20:
            problems.append(f"{key}: temporary fallback requires an actionable owner_note")
    return problems


def documentation_problems(entries: list[dict], documentation: str) -> list[str]:
    problems = []
    counts = Counter(entry.get("disposition") for entry in entries)
    for disposition in ("permanent_accepted", "temporary_accepted"):
        expected = f"- `{disposition}`: **{counts[disposition]}**"
        if expected not in documentation:
            problems.append(f"accepted-fallback documentation missing current count line: {expected}")
    if "path + function + normalized expression + occurrence index" not in documentation:
        problems.append("accepted-fallback documentation is missing occurrence identity policy")
    return problems


def audit() -> list[str]:
    problems = []
    payload = json.loads(INVENTORY.read_text(encoding="utf-8"))
    if payload.get("version") != 3:
        problems.append("inventory schema version must be 3")
    entries = payload.get("entries", [])
    keys = [occurrence_key(entry) for entry in entries]
    duplicates = [key for key, count in Counter(keys).items() if count > 1]
    if duplicates:
        problems.append(f"duplicate inventory occurrence records: {duplicates}")
    expected_pairs = allowlist_keys()
    observed_pairs = {(entry.get("path"), entry.get("expression")) for entry in entries}
    if expected_pairs != observed_pairs:
        problems.append(
            f"inventory keys differ: missing={sorted(expected_pairs-observed_pairs)} extra={sorted(observed_pairs-expected_pairs)}"
        )
    records_by_pair: dict[tuple[str, str], list[dict]] = defaultdict(list)
    for entry in entries:
        key = occurrence_key(entry)
        entry_problems = metadata_problems(key, entry)
        problems.extend(entry_problems)
        if not entry_problems:
            records_by_pair[(entry["path"], entry["expression"])].append(entry)
    for path, expression in expected_pairs:
        source_keys = {
            (path, item["function"], expression, item["occurrence"], item["context_before"], item["context_after"])
            for item in source_occurrences(path, expression)
        }
        inventory_keys = {
            (entry["path"], entry["function"], entry["expression"], entry["occurrence"], entry["context_before"], entry["context_after"])
            for entry in records_by_pair.get((path, expression), [])
        }
        if source_keys != inventory_keys:
            problems.append(
                f"{path}|{expression}: occurrence mismatch missing={sorted(source_keys-inventory_keys)} stale={sorted(inventory_keys-source_keys)}"
            )
    problems.extend(documentation_problems(entries, DOCUMENTATION.read_text(encoding="utf-8")))
    return problems


def self_test() -> None:
    broad = occurrences_from_lines(["fn example() {", "  .ok()", "  .ok()", "}"], ".ok()")
    assert len(broad) == 2 and [item["occurrence"] for item in broad] == [1, 2]
    defaults = occurrences_from_lines(
        ["fn example() {", "  .unwrap_or_default()", "  .unwrap_or_default()", "}"],
        ".unwrap_or_default()",
    )
    assert len(defaults) == 2
    base = {
        "path": "src/example.rs", "function": "example", "expression": ".ok()", "occurrence": 1,
        "context_before": "let value = input", "context_after": "return value",
        "justification": "safe", "user_visibility": "visible", "side_effect_impact": "none",
        "test_coverage": "unit", "future_replacement": "none", "disposition": "permanent_accepted",
        "review_due": "not_applicable", "owner_note": "Permanent exact fallback with no authority impact.",
    }
    missing = dict(base); missing.pop("occurrence")
    assert "missing fields" in metadata_problems(occurrence_key(missing), missing)[0]
    temporary = dict(base, disposition="temporary_accepted", review_due="not_applicable", owner_note="short")
    issues = metadata_problems(occurrence_key(temporary), temporary)
    assert any("review boundary" in issue for issue in issues)
    assert any("actionable owner_note" in issue for issue in issues)
    assert len({occurrence_key(base), occurrence_key(dict(base))}) == 1
    stale = dict(base, context_after="changed")
    assert stale["context_after"] != base["context_after"]
    print("Security fallback inventory self-test passed")


def main() -> int:
    if sys.argv[1:] == ["--self-test"]:
        self_test(); return 0
    if sys.argv[1:]:
        print("usage: check-security-fallback-inventory.py [--self-test]", file=sys.stderr); return 2
    problems = audit()
    if problems:
        print("Security fallback inventory audit failed:", file=sys.stderr)
        for problem in problems:
            print(f"- {problem}", file=sys.stderr)
        return 1
    print("Security fallback inventory audit passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
'''
write("scripts/check-security-fallback-inventory.py", scanner)

# Reconcile accepted fallback documentation.
payload = json.loads(inventory_path.read_text(encoding="utf-8"))
counts = Counter(entry["disposition"] for entry in payload["entries"])
docs = read("docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md")
docs = re.sub(r"- `permanent_accepted`: \*\*\d+\*\*", f"- `permanent_accepted`: **{counts['permanent_accepted']}**", docs)
docs = re.sub(r"- `temporary_accepted`: \*\*\d+\*\*", f"- `temporary_accepted`: **{counts['temporary_accepted']}**", docs)
docs = re.sub(r"- converted or removed in this pass: \*\*\d+\*\*", "- converted or removed across post-P8 passes: **17**", docs)
docs = re.sub(
    r"`temporary_accepted` requires an actionable `owner_note`.*?Permanent CI verifies",
    "`temporary_accepted` requires an actionable `owner_note` and a concrete `review_due` boundary. "
    "This enforcement pass converted the four remaining command/fill temporary fallbacks and "
    "reclassified optional skill intent tags as permanently capability-reducing, leaving no temporary accepted fallback entries.\n\n"
    "Every accepted occurrence is now identified by **path + function + normalized expression + occurrence index + adjacent normalized context**. "
    "A new duplicate `.ok()` or `.unwrap_or_default()` in an already allowlisted function therefore fails CI instead of inheriting approval.\n\n"
    "Permanent CI verifies",
    docs,
    flags=re.S,
)
docs = re.sub(
    r"## Remaining temporary accepted fallbacks.*?## Permanent accepted categories",
    "## Remaining temporary accepted fallbacks\n\nNone. Any future temporary fallback must carry a unique occurrence identity, actionable owner note, and concrete review boundary.\n\n## Permanent accepted categories",
    docs,
    flags=re.S,
)
docs = docs.replace("This pass removed thirteen exact accepted expressions:", "The post-P8 passes removed or converted seventeen exact accepted expressions:")
docs = docs.replace(
    "- five validator policy-detail serialization `.ok()` expressions.",
    "- five validator policy-detail serialization `.ok()` expressions;\n"
    "- project-root and user-skill-root discovery `.ok()` fallbacks;\n"
    "- direct focus-query construction `.ok()?`;\n"
    "- recent fill-correction candidate `.ok()` omission.",
)
write("docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md", docs)

print("Applied post-P8 fallback enforcement hardening patch")
