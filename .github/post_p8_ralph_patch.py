#!/usr/bin/env python3
"""Apply the bounded post-P8 fallback/evidence hardening patch."""
from __future__ import annotations

import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, text: str) -> None:
    (ROOT / path).write_text(text, encoding="utf-8")


def replace_once(path: str, old: str, new: str) -> None:
    text = read(path)
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"expected one match in {path}, found {count}: {old[:120]!r}")
    write(path, text.replace(old, new, 1))


def append_once(path: str, marker: str, content: str) -> None:
    text = read(path)
    if marker in text:
        raise SystemExit(f"marker already present in {path}: {marker}")
    write(path, text.rstrip() + "\n\n" + content.rstrip() + "\n")


# ---------------------------------------------------------------------------
# URL sanitization: reconstruct from approved components instead of ignoring
# Url mutator results. The helper is shared by diagnostic, settings, and planner
# surfaces so credentials/query/fragment handling stays consistent.
# ---------------------------------------------------------------------------
replace_once(
    "src-tauri/src/diagnostic_redaction.rs",
    '''pub(crate) fn redact_diagnostic_text(value: &str) -> String {''',
    '''#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SanitizedUrlDisplay {
    pub(crate) value: String,
    pub(crate) removed_query: bool,
    pub(crate) removed_fragment: bool,
}

/// Reconstruct a display-safe URL from its origin and path. Userinfo, query,
/// and fragment components are never copied into the returned value.
pub(crate) fn sanitize_url_for_display(value: &str) -> Option<SanitizedUrlDisplay> {
    let parsed = match url::Url::parse(value) {
        Ok(parsed) => parsed,
        Err(_) => return None,
    };
    if parsed.host_str().is_none() {
        return None;
    }
    let origin = parsed.origin().ascii_serialization();
    if origin == "null" {
        return None;
    }
    let path = parsed.path();
    let safe_path = if path == "/" { "" } else { path };
    Some(SanitizedUrlDisplay {
        value: format!("{origin}{safe_path}"),
        removed_query: parsed.query().is_some(),
        removed_fragment: parsed.fragment().is_some(),
    })
}

pub(crate) fn redact_diagnostic_text(value: &str) -> String {''',
)
replace_once(
    "src-tauri/src/diagnostic_redaction.rs",
    '''fn redact_url_query(value: &str) -> String {
    if let Ok(mut url) = url::Url::parse(value) {
        let _ = url.set_username("");
        let _ = url.set_password(None);
        url.set_query(None);
        url.set_fragment(None);
        return url.to_string();
    }
    value.to_string()
}''',
    '''fn redact_url_query(value: &str) -> String {
    match sanitize_url_for_display(value) {
        Some(safe) => safe.value,
        None if value.contains("://") => String::from("[REDACTED INVALID URL]"),
        None => value.to_string(),
    }
}''',
)
replace_once(
    "src-tauri/src/diagnostic_redaction.rs",
    '''        assert!(safe.contains("safe reason"));
    }
}''',
    '''        assert!(safe.contains("safe reason"));
    }

    #[test]
    fn reconstructs_urls_without_userinfo_query_or_fragment() {
        let safe = sanitize_url_for_display(
            "https://user:pass@example.com:8443/safe/path?token=secret#fragment",
        )
        .expect("URL should be reconstructable");
        assert_eq!(safe.value, "https://example.com:8443/safe/path");
        assert!(safe.removed_query);
        assert!(safe.removed_fragment);
        assert!(!safe.value.contains("user"));
        assert!(!safe.value.contains("pass"));
        assert!(!safe.value.contains("secret"));

        assert_eq!(
            redact_diagnostic_text("https://[invalid?token=secret"),
            "[REDACTED INVALID URL]"
        );
    }
}''',
)
replace_once(
    "src-tauri/src/app_core/planner_redaction.rs",
    '''use serde::Serialize;

use crate::audio_io::RuntimeAudioState;''',
    '''use serde::Serialize;

use crate::audio_io::RuntimeAudioState;
use crate::diagnostic_redaction::sanitize_url_for_display;''',
)
replace_once(
    "src-tauri/src/app_core/planner_redaction.rs",
    '''fn sanitize_url(raw: &str, metadata: &mut SanitizationMetadata) -> PlannerSafeUrl {
    let Ok(mut parsed) = url::Url::parse(raw) else {
        metadata.redacted_text_fields += 1;
        return PlannerSafeUrl(String::from("[REDACTED INVALID URL]"));
    };

    let _ = parsed.set_username("");
    let _ = parsed.set_password(None);
    parsed.set_fragment(None);
    if parsed.query().is_some() {
        metadata.query_values_removed += 1;
        parsed.set_query(None);
    }

    PlannerSafeUrl(truncate_chars(parsed.as_str(), MAX_URL_CHARS, metadata))
}''',
    '''fn sanitize_url(raw: &str, metadata: &mut SanitizationMetadata) -> PlannerSafeUrl {
    let Some(safe) = sanitize_url_for_display(raw) else {
        metadata.redacted_text_fields += 1;
        return PlannerSafeUrl(String::from("[REDACTED INVALID URL]"));
    };
    if safe.removed_query {
        metadata.query_values_removed += 1;
    }
    PlannerSafeUrl(truncate_chars(&safe.value, MAX_URL_CHARS, metadata))
}''',
)
append_once(
    "src-tauri/src/app_core/planner_redaction.rs",
    "post_p8_url_sanitization_reconstructs_approved_components",
    '''#[cfg(test)]
mod post_p8_url_sanitization_tests {
    use super::*;

    #[test]
    fn post_p8_url_sanitization_reconstructs_approved_components() {
        let mut metadata = SanitizationMetadata::default();
        let safe = sanitize_url(
            "https://user:pass@example.com:8443/safe/path?token=secret#fragment",
            &mut metadata,
        );
        assert_eq!(safe.0, "https://example.com:8443/safe/path");
        assert_eq!(metadata.query_values_removed, 1);
        assert!(!safe.0.contains("user"));
        assert!(!safe.0.contains("pass"));
        assert!(!safe.0.contains("token"));
        assert!(!safe.0.contains("fragment"));

        let malformed = sanitize_url("https://[invalid?token=secret", &mut metadata);
        assert_eq!(malformed.0, "[REDACTED INVALID URL]");
    }
}''',
)


# ---------------------------------------------------------------------------
# Skill discovery: aggregate unreadable directory-entry warnings without paths,
# while continuing to load valid adjacent entries.
# ---------------------------------------------------------------------------
replace_once(
    "src-tauri/src/commands/skill_loader.rs",
    '''use std::collections::HashMap;
use std::fs;
use std::path::Path;''',
    '''use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io;
use std::path::Path;''',
)
replace_once(
    "src-tauri/src/commands/skill_loader.rs",
    '''fn load_skills_from_directory(
    skill_root: &Path,''',
    '''#[derive(Debug, Default, PartialEq, Eq)]
struct SkillEntryWarningSummary {
    skipped_entries: usize,
    error_categories: BTreeMap<&'static str, usize>,
}

fn skill_entry_error_category(kind: io::ErrorKind) -> &'static str {
    match kind {
        io::ErrorKind::PermissionDenied => "permission_denied",
        io::ErrorKind::InvalidData => "invalid_data",
        io::ErrorKind::NotFound => "not_found",
        io::ErrorKind::Interrupted => "interrupted",
        _ => "other_io",
    }
}

fn collect_readable_entries<T, I>(entries: I) -> (Vec<T>, SkillEntryWarningSummary)
where
    I: IntoIterator<Item = io::Result<T>>,
{
    let mut readable = Vec::new();
    let mut warnings = SkillEntryWarningSummary::default();
    for entry in entries {
        match entry {
            Ok(entry) => readable.push(entry),
            Err(error) => {
                warnings.skipped_entries += 1;
                *warnings
                    .error_categories
                    .entry(skill_entry_error_category(error.kind()))
                    .or_insert(0) += 1;
            }
        }
    }
    (readable, warnings)
}

fn load_skills_from_directory(
    skill_root: &Path,''',
)
replace_once(
    "src-tauri/src/commands/skill_loader.rs",
    '''    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();''',
    '''    let (entries, entry_warnings) = collect_readable_entries(entries);
    if entry_warnings.skipped_entries > 0 {
        tracing::warn!(
            source = source_label,
            skipped_entries = entry_warnings.skipped_entries,
            error_categories = ?entry_warnings.error_categories,
            "skipped unreadable skill directory entries"
        );
    }

    for entry in entries {
        let path = entry.path();''',
)
replace_once(
    "src-tauri/src/commands/skill_loader.rs",
    '''mod tests {
    use super::{skill_directory_label, skill_source_label, SkillSource};
    use std::path::Path;''',
    '''mod tests {
    use super::{
        collect_readable_entries, skill_directory_label, skill_source_label, SkillSource,
    };
    use std::io;
    use std::path::Path;''',
)
replace_once(
    "src-tauri/src/commands/skill_loader.rs",
    '''        assert!(!skill_directory_label(path).contains("secret-project"));
    }
}''',
    '''        assert!(!skill_directory_label(path).contains("secret-project"));
    }

    #[test]
    fn unreadable_entries_are_aggregated_without_dropping_valid_neighbors() {
        let entries = vec![
            Ok("valid-one"),
            Err(io::Error::from(io::ErrorKind::PermissionDenied)),
            Ok("valid-two"),
            Err(io::Error::from(io::ErrorKind::InvalidData)),
        ];
        let (readable, warnings) = collect_readable_entries(entries);
        assert_eq!(readable, vec!["valid-one", "valid-two"]);
        assert_eq!(warnings.skipped_entries, 2);
        assert_eq!(warnings.error_categories.get("permission_denied"), Some(&1));
        assert_eq!(warnings.error_categories.get("invalid_data"), Some(&1));
        let diagnostic = format!("{warnings:?}");
        assert!(!diagnostic.contains("/home/"));
        assert!(!diagnostic.contains("secret-project"));
    }
}''',
)


# ---------------------------------------------------------------------------
# Typed settings absence/degradation reasons.
# ---------------------------------------------------------------------------
replace_once(
    "src-tauri/src/commands/contracts/providers.rs",
    '''#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TtsModelOption {''',
    '''#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityAbsenceReason {
    NotConfigured,
    ProfileMissing,
    InvalidEndpoint,
    UnknownModelId,
    ManifestUnavailable,
    FeatureDisabled,
    CredentialReferenceMissing,
    LocalBinaryUnavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct TtsModelOption {''',
)
replace_once(
    "src-tauri/src/commands/contracts/providers.rs",
    '''    pub endpoint_is_loopback: Option<bool>,
    pub consent_to_remote_page_data: bool,''',
    '''    pub endpoint_is_loopback: Option<bool>,
    pub availability_reason: Option<CapabilityAbsenceReason>,
    pub consent_to_remote_page_data: bool,''',
)
replace_once(
    "src-tauri/src/app_core/mod.rs",
    '''    pub download_supported: bool,
    pub download_label: Option<String>,''',
    '''    pub download_supported: bool,
    pub download_label: Option<String>,
    pub download_absence_reason: Option<crate::commands::CapabilityAbsenceReason>,''',
)
replace_once(
    "src-tauri/src/app_core/settings_adapters.rs",
    '''    AsrProviderSettings, ConfirmationSettings, LocalAsrModelSettings, LocalTtsModelSettings,
    OcrThresholdSettings, ProviderFailoverSettings, RemoteAsrSettings, RemotePlannerSettings,''',
    '''    AsrProviderSettings, CapabilityAbsenceReason, ConfirmationSettings, LocalAsrModelSettings,
    LocalTtsModelSettings, OcrThresholdSettings, ProviderFailoverSettings, RemoteAsrSettings,
    RemotePlannerSettings,''',
)
replace_once(
    "src-tauri/src/app_core/settings_adapters.rs",
    '''use crate::provider_endpoint::ProviderEndpointScope;''',
    '''use crate::diagnostic_redaction::sanitize_url_for_display;
use crate::provider_endpoint::ProviderEndpointScope;''',
)
old_remote_planner = '''pub(crate) fn build_remote_planner_settings(config: &AppConfig) -> RemotePlannerSettings {
    let profile_name = config.providers.planner.remote_profile.clone();
    let profile = profile_name
        .as_ref()
        .and_then(|configured_profile| config.remote_planner_profiles.get(configured_profile));

    let (api_key_masked_value, api_key_reference_error) = profile
        .map(|p| masked_secret_status(&p.api_key))
        .unwrap_or((None, None));

    RemotePlannerSettings {
        profile_name,
        provider: profile
            .map(|configured_profile| remote_provider_label(&configured_profile.provider)),
        base_url: profile.map(|configured_profile| configured_profile.base_url.clone()),
        model: profile.map(|configured_profile| configured_profile.model.clone()),
        api_key_reference: profile
            .map(|configured_profile| secret_ref_reference(&configured_profile.api_key)),
        api_key_masked_value,
        api_key_reference_error,
        organization_reference: profile
            .and_then(|configured_profile| configured_profile.organization.as_ref())
            .map(secret_ref_reference),
        project: profile.and_then(|configured_profile| configured_profile.project.clone()),
        temperature_milli: profile.map(|configured_profile| configured_profile.temperature_milli),
        max_output_tokens: profile.map(|configured_profile| configured_profile.max_output_tokens),
        timeout_ms: profile.map(|configured_profile| configured_profile.timeout_ms),
        endpoint_is_loopback: profile
            .and_then(|configured_profile| ProviderEndpointScope::parse(&configured_profile.base_url).ok())
            .map(|scope| scope.is_loopback()),
        consent_to_remote_page_data: config.remote_planner_privacy.consent_to_remote_page_data,
        local_only: config.remote_planner_privacy.local_only,
        blocked_origins: config.remote_planner_privacy.blocked_origins.clone(),
        high_risk_origin_policy: match config.remote_planner_privacy.high_risk_origin_policy {
            HighRiskOriginPolicy::Block => String::from("block"),
        },
        remote_data_notice: String::from(
            "Network planner endpoints receive only locally selected, sanitized page, OCR, tool, and skill context after explicit consent. Loopback endpoints stay on this device. High-risk pages and blocked origins never leave the device.",
        ),
    }
}'''
new_remote_planner = '''pub(crate) fn build_remote_planner_settings(config: &AppConfig) -> RemotePlannerSettings {
    let profile_name = config.providers.planner.remote_profile.clone();
    let profile = profile_name
        .as_ref()
        .and_then(|configured_profile| config.remote_planner_profiles.get(configured_profile));

    let (endpoint_scope, availability_reason) = match (profile_name.as_ref(), profile) {
        (None, _) => (None, Some(CapabilityAbsenceReason::NotConfigured)),
        (Some(_), None) => (None, Some(CapabilityAbsenceReason::ProfileMissing)),
        (Some(_), Some(configured_profile)) => {
            match ProviderEndpointScope::parse(&configured_profile.base_url) {
                Ok(scope) => (Some(scope), None),
                Err(_) => (None, Some(CapabilityAbsenceReason::InvalidEndpoint)),
            }
        }
    };
    let (api_key_masked_value, api_key_reference_error) = profile
        .map(|p| masked_secret_status(&p.api_key))
        .unwrap_or((None, None));

    RemotePlannerSettings {
        profile_name,
        provider: profile
            .map(|configured_profile| remote_provider_label(&configured_profile.provider)),
        base_url: profile.map(|configured_profile| {
            endpoint_scope
                .as_ref()
                .map(|scope| scope.normalized_base_url().to_string())
                .or_else(|| {
                    sanitize_url_for_display(&configured_profile.base_url).map(|safe| safe.value)
                })
                .unwrap_or_else(|| String::from("[REDACTED INVALID ENDPOINT]"))
        }),
        model: profile.map(|configured_profile| configured_profile.model.clone()),
        api_key_reference: profile
            .map(|configured_profile| secret_ref_reference(&configured_profile.api_key)),
        api_key_masked_value,
        api_key_reference_error,
        organization_reference: profile
            .and_then(|configured_profile| configured_profile.organization.as_ref())
            .map(secret_ref_reference),
        project: profile.and_then(|configured_profile| configured_profile.project.clone()),
        temperature_milli: profile.map(|configured_profile| configured_profile.temperature_milli),
        max_output_tokens: profile.map(|configured_profile| configured_profile.max_output_tokens),
        timeout_ms: profile.map(|configured_profile| configured_profile.timeout_ms),
        endpoint_is_loopback: endpoint_scope.as_ref().map(ProviderEndpointScope::is_loopback),
        availability_reason,
        consent_to_remote_page_data: config.remote_planner_privacy.consent_to_remote_page_data,
        local_only: config.remote_planner_privacy.local_only,
        blocked_origins: config.remote_planner_privacy.blocked_origins.clone(),
        high_risk_origin_policy: match config.remote_planner_privacy.high_risk_origin_policy {
            HighRiskOriginPolicy::Block => String::from("block"),
        },
        remote_data_notice: String::from(
            "Network planner endpoints receive only locally selected, sanitized page, OCR, tool, and skill context after explicit consent. Loopback endpoints stay on this device. High-risk pages and blocked origins never leave the device.",
        ),
    }
}'''
replace_once("src-tauri/src/app_core/settings_adapters.rs", old_remote_planner, new_remote_planner)
old_models = '''pub(crate) fn build_model_management_settings(config: &AppConfig) -> ModelManagementSettingsData {
    let (local_tts_profile_name, local_tts_profile) =
        match config.providers.tts.local_profile.as_ref() {
            Some(profile_name) => (
                Some(profile_name.clone()),
                config.local_tts_profiles.get(profile_name),
            ),
            None => (None, None),
        };
    let (local_asr_profile_name, local_asr_profile) =
        match config.providers.asr.local_profile.as_ref() {
            Some(profile_name) => (
                Some(profile_name.clone()),
                config.local_asr_profiles.get(profile_name),
            ),
            None => (None, None),
        };

    ModelManagementSettingsData {
        models_dir: config.models.models_dir.clone(),
        check_on_startup: config.models.check_on_startup,
        auto_download_missing: config.models.auto_download_missing,
        local_tts: ManagedLocalModelStatusData {
            profile_name: local_tts_profile_name,
            backend: local_tts_profile.map(|profile| profile.backend.to_string()),
            model_id: local_tts_profile.map(|profile| profile.model_id.clone()),
            model_path: local_tts_profile.map(|profile| profile.model_path.clone()),
            available: local_tts_profile.is_some_and(local_tts_model_is_available),
            download_supported: local_tts_profile.is_some_and(|profile| {
                kitten_download_plan_for_model_id(&profile.model_id).is_ok()
            }),
            download_label: local_tts_profile
                .and_then(|profile| kitten_download_plan_for_model_id(&profile.model_id).ok())
                .map(|plan| format!("Download {}", plan.display_name)),
        },
        local_asr: ManagedLocalModelStatusData {
            profile_name: local_asr_profile_name,
            backend: local_asr_profile.map(|profile| profile.backend.to_string()),
            model_id: local_asr_profile.map(|profile| profile.model_id.clone()),
            model_path: local_asr_profile.map(|profile| profile.model_path.clone()),
            available: local_asr_profile.is_some_and(local_asr_model_is_available),
            download_supported: local_asr_profile.is_some_and(|profile| {
                whisper_download_plan_for_model_id(&profile.model_id).is_ok()
            }),
            download_label: local_asr_profile
                .and_then(|profile| whisper_download_plan_for_model_id(&profile.model_id).ok())
                .map(|plan| format!("Download Whisper {}", plan.display_name)),
        },
    }
}'''
new_models = '''pub(crate) fn build_model_management_settings(config: &AppConfig) -> ModelManagementSettingsData {
    let (local_tts_profile_name, local_tts_profile) =
        match config.providers.tts.local_profile.as_ref() {
            Some(profile_name) => (
                Some(profile_name.clone()),
                config.local_tts_profiles.get(profile_name),
            ),
            None => (None, None),
        };
    let (local_asr_profile_name, local_asr_profile) =
        match config.providers.asr.local_profile.as_ref() {
            Some(profile_name) => (
                Some(profile_name.clone()),
                config.local_asr_profiles.get(profile_name),
            ),
            None => (None, None),
        };

    let (tts_download_supported, tts_download_label, tts_download_absence_reason) =
        match (local_tts_profile_name.as_ref(), local_tts_profile) {
            (None, _) => (false, None, Some(CapabilityAbsenceReason::NotConfigured)),
            (Some(_), None) => (false, None, Some(CapabilityAbsenceReason::ProfileMissing)),
            (Some(_), Some(profile)) => match kitten_download_plan_for_model_id(&profile.model_id) {
                Ok(plan) => (true, Some(format!("Download {}", plan.display_name)), None),
                Err(_) => (false, None, Some(CapabilityAbsenceReason::UnknownModelId)),
            },
        };
    let (asr_download_supported, asr_download_label, asr_download_absence_reason) =
        match (local_asr_profile_name.as_ref(), local_asr_profile) {
            (None, _) => (false, None, Some(CapabilityAbsenceReason::NotConfigured)),
            (Some(_), None) => (false, None, Some(CapabilityAbsenceReason::ProfileMissing)),
            (Some(_), Some(profile)) => match whisper_download_plan_for_model_id(&profile.model_id) {
                Ok(plan) => (
                    true,
                    Some(format!("Download Whisper {}", plan.display_name)),
                    None,
                ),
                Err(_) => (false, None, Some(CapabilityAbsenceReason::UnknownModelId)),
            },
        };

    ModelManagementSettingsData {
        models_dir: config.models.models_dir.clone(),
        check_on_startup: config.models.check_on_startup,
        auto_download_missing: config.models.auto_download_missing,
        local_tts: ManagedLocalModelStatusData {
            profile_name: local_tts_profile_name,
            backend: local_tts_profile.map(|profile| profile.backend.to_string()),
            model_id: local_tts_profile.map(|profile| profile.model_id.clone()),
            model_path: local_tts_profile.map(|profile| profile.model_path.clone()),
            available: local_tts_profile.is_some_and(local_tts_model_is_available),
            download_supported: tts_download_supported,
            download_label: tts_download_label,
            download_absence_reason: tts_download_absence_reason,
        },
        local_asr: ManagedLocalModelStatusData {
            profile_name: local_asr_profile_name,
            backend: local_asr_profile.map(|profile| profile.backend.to_string()),
            model_id: local_asr_profile.map(|profile| profile.model_id.clone()),
            model_path: local_asr_profile.map(|profile| profile.model_path.clone()),
            available: local_asr_profile.is_some_and(local_asr_model_is_available),
            download_supported: asr_download_supported,
            download_label: asr_download_label,
            download_absence_reason: asr_download_absence_reason,
        },
    }
}'''
replace_once("src-tauri/src/app_core/settings_adapters.rs", old_models, new_models)
replace_once(
    "src-tauri/src/app_core/settings_adapters.rs",
    '''            match active_remote_profile.map(|profile| &profile.provider) {
                Some(RemoteProviderKind::OpenAi) => OPENAI_TTS_VOICES
                    .iter()
                    .map(|voice| TtsVoiceOption {
                        voice_name: (*voice).to_string(),
                        display_label: (*voice).to_string(),
                    })
                    .collect(),
                Some(_) => active_remote_profile
                    .map(|profile| {
                        vec![TtsVoiceOption {
                            voice_name: profile.voice.clone(),
                            display_label: profile.voice.clone(),
                        }]
                    })
                    .unwrap_or_default(),
                None => Vec::new(),
            }''',
    '''            match active_remote_profile {
                Some(profile) if profile.provider == RemoteProviderKind::OpenAi => OPENAI_TTS_VOICES
                    .iter()
                    .map(|voice| TtsVoiceOption {
                        voice_name: (*voice).to_string(),
                        display_label: (*voice).to_string(),
                    })
                    .collect(),
                Some(profile) => vec![TtsVoiceOption {
                    voice_name: profile.voice.clone(),
                    display_label: profile.voice.clone(),
                }],
                None => Vec::new(),
            }''',
)
append_once(
    "src-tauri/src/app_core/tests/settings_tests.rs",
    "post_p8_settings_surface_typed_absence_reasons",
    '''#[test]
fn post_p8_settings_surface_typed_absence_reasons() {
    use crate::commands::CapabilityAbsenceReason;

    let mut invalid_endpoint = AppConfig::default();
    invalid_endpoint
        .remote_planner_profiles
        .get_mut("openai-default")
        .expect("default profile")
        .base_url = String::from(
        "https://user:pass@api.example.com:8443/v1?token=secret#fragment",
    );
    let settings = build_remote_planner_settings(&invalid_endpoint);
    assert_eq!(
        settings.availability_reason,
        Some(CapabilityAbsenceReason::InvalidEndpoint)
    );
    assert_eq!(
        settings.base_url.as_deref(),
        Some("https://api.example.com:8443/v1")
    );
    let displayed = settings.base_url.as_deref().unwrap_or_default();
    assert!(!displayed.contains("user"));
    assert!(!displayed.contains("pass"));
    assert!(!displayed.contains("secret"));
    assert!(!displayed.contains('?'));
    assert!(!displayed.contains('#'));

    let mut not_configured = AppConfig::default();
    not_configured.providers.planner.remote_profile = None;
    assert_eq!(
        build_remote_planner_settings(&not_configured).availability_reason,
        Some(CapabilityAbsenceReason::NotConfigured)
    );

    let mut profile_missing = AppConfig::default();
    profile_missing.providers.planner.remote_profile = Some(String::from("missing-profile"));
    assert_eq!(
        build_remote_planner_settings(&profile_missing).availability_reason,
        Some(CapabilityAbsenceReason::ProfileMissing)
    );

    let mut unknown_models = AppConfig::default();
    unknown_models
        .local_tts_profiles
        .get_mut("kitten-default")
        .expect("default TTS profile")
        .model_id = String::from("unknown-kitten-model");
    unknown_models
        .local_asr_profiles
        .get_mut("whisper-default")
        .expect("default ASR profile")
        .model_id = String::from("unknown-whisper-model");
    let model_settings = build_model_management_settings(&unknown_models);
    assert_eq!(
        model_settings.local_tts.download_absence_reason,
        Some(CapabilityAbsenceReason::UnknownModelId)
    );
    assert_eq!(
        model_settings.local_asr.download_absence_reason,
        Some(CapabilityAbsenceReason::UnknownModelId)
    );
    assert!(!model_settings.local_tts.download_supported);
    assert!(!model_settings.local_asr.download_supported);

    let valid = build_model_management_settings(&AppConfig::default());
    assert_eq!(valid.local_tts.download_absence_reason, None);
    assert_eq!(valid.local_asr.download_absence_reason, None);
}''',
)
replace_once(
    "src/tauri-types.ts",
    '''export type RemoteTtsAudioFormat = "wav";''',
    '''export type RemoteTtsAudioFormat = "wav";
export type CapabilityAbsenceReason =
  | "not_configured"
  | "profile_missing"
  | "invalid_endpoint"
  | "unknown_model_id"
  | "manifest_unavailable"
  | "feature_disabled"
  | "credential_reference_missing"
  | "local_binary_unavailable";''',
)
replace_once(
    "src/tauri-types.ts",
    '''  download_supported: boolean;
  download_label: string | null;
}''',
    '''  download_supported: boolean;
  download_label: string | null;
  download_absence_reason: CapabilityAbsenceReason | null;
}''',
)
replace_once(
    "src/tauri-types.ts",
    '''  endpoint_is_loopback: boolean | null;
  consent_to_remote_page_data: boolean;''',
    '''  endpoint_is_loopback: boolean | null;
  availability_reason: CapabilityAbsenceReason | null;
  consent_to_remote_page_data: boolean;''',
)


# ---------------------------------------------------------------------------
# Direct-command semantic evidence mappings. Source-string checks remain only
# supplemental drift detectors in the integration test.
# ---------------------------------------------------------------------------
replace_once(
    "src-tauri/src/direct_command_policy.rs",
    '''#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DirectCommandPolicy {''',
    '''#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DirectCommandNetworkPolicy {
    RemotePlanner,
    RemoteAsr,
    RemoteAsrAndPlanner,
    BrowserNavigation,
    CredentialProbe,
    VerifiedModelDownload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DirectCommandCredentialPolicy {
    EndpointBound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DirectCommandPageContextPolicy {
    SanitizedRemotePlanner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DirectCommandArtifactPolicy {
    VerifiedAtomicActivation,
}

pub(crate) const fn direct_command_network_policy(
    name: DirectCommandName,
) -> Option<DirectCommandNetworkPolicy> {
    use DirectCommandName as D;
    match name {
        D::ResolveCommand => Some(DirectCommandNetworkPolicy::RemotePlanner),
        D::TranscribeCommand => Some(DirectCommandNetworkPolicy::RemoteAsr),
        D::TranscribeAndExecuteCommand => {
            Some(DirectCommandNetworkPolicy::RemoteAsrAndPlanner)
        }
        D::OpenUrl => Some(DirectCommandNetworkPolicy::BrowserNavigation),
        D::ListRemotePlannerModels
        | D::TestRemotePlannerApiKey
        | D::TestRemoteTtsApiKey
        | D::TestRemoteAsrApiKey => Some(DirectCommandNetworkPolicy::CredentialProbe),
        D::DownloadActiveLocalTtsModel | D::DownloadActiveLocalAsrModel => {
            Some(DirectCommandNetworkPolicy::VerifiedModelDownload)
        }
        _ => None,
    }
}

pub(crate) const fn direct_command_credential_policy(
    name: DirectCommandName,
) -> Option<DirectCommandCredentialPolicy> {
    match name {
        DirectCommandName::ResolveCommand
        | DirectCommandName::TranscribeCommand
        | DirectCommandName::TranscribeAndExecuteCommand
        | DirectCommandName::ListRemotePlannerModels
        | DirectCommandName::TestRemotePlannerApiKey
        | DirectCommandName::TestRemoteTtsApiKey
        | DirectCommandName::TestRemoteAsrApiKey => {
            Some(DirectCommandCredentialPolicy::EndpointBound)
        }
        _ => None,
    }
}

pub(crate) const fn direct_command_page_context_policy(
    name: DirectCommandName,
) -> Option<DirectCommandPageContextPolicy> {
    match name {
        DirectCommandName::ResolveCommand | DirectCommandName::TranscribeAndExecuteCommand => {
            Some(DirectCommandPageContextPolicy::SanitizedRemotePlanner)
        }
        _ => None,
    }
}

pub(crate) const fn direct_command_artifact_policy(
    name: DirectCommandName,
) -> Option<DirectCommandArtifactPolicy> {
    match name {
        DirectCommandName::DownloadActiveLocalTtsModel
        | DirectCommandName::DownloadActiveLocalAsrModel => {
            Some(DirectCommandArtifactPolicy::VerifiedAtomicActivation)
        }
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DirectCommandPolicy {''',
)
replace_once(
    "src-tauri/src/direct_command_policy.rs",
    '''        assert!(
            !policy.launches_external_program || policy.requires_user_gesture,
            "external program launch requires an explicit user gesture"
        );
        std::hint::black_box((policy.class, policy.mutates_runtime_state));''',
    '''        assert!(
            !policy.launches_external_program || policy.requires_user_gesture,
            "external program launch requires an explicit user gesture"
        );
        assert_eq!(
            policy.performs_network_io,
            direct_command_network_policy(*name).is_some(),
            "networked direct commands require a typed semantic network mapping"
        );
        assert_eq!(
            policy.credential_bearing_network_io,
            direct_command_credential_policy(*name).is_some(),
            "credential-bearing direct commands require endpoint-bound mapping"
        );
        assert_eq!(
            policy.transmits_page_context,
            direct_command_page_context_policy(*name).is_some(),
            "page-context direct commands require sanitizer mapping"
        );
        assert_eq!(
            policy.downloads_executable_or_model_artifact,
            direct_command_artifact_policy(*name).is_some(),
            "artifact direct commands require verified activation mapping"
        );
        std::hint::black_box((policy.class, policy.mutates_runtime_state));''',
)
replace_once(
    "src-tauri/src/direct_command_policy.rs",
    '''    #[test]
    fn every_direct_command_has_explicit_policy() {''',
    '''    #[test]
    fn semantic_direct_command_mappings_match_policy_flags() {
        for name in DirectCommandName::ALL {
            let policy = direct_command_policy(*name);
            assert_eq!(
                policy.performs_network_io,
                direct_command_network_policy(*name).is_some(),
                "{}",
                name.as_handler_name()
            );
            assert_eq!(
                policy.credential_bearing_network_io,
                direct_command_credential_policy(*name).is_some(),
                "{}",
                name.as_handler_name()
            );
            assert_eq!(
                policy.transmits_page_context,
                direct_command_page_context_policy(*name).is_some(),
                "{}",
                name.as_handler_name()
            );
            assert_eq!(
                policy.downloads_executable_or_model_artifact,
                direct_command_artifact_policy(*name).is_some(),
                "{}",
                name.as_handler_name()
            );
        }
    }

    #[test]
    fn every_direct_command_has_explicit_policy() {''',
)
for old, new in [
    ("fn every_networked_direct_command_has_timeout_and_redirect_evidence()", "fn source_drift_networked_direct_commands_retain_timeout_and_redirect_evidence()"),
    ("fn every_credential_bearing_direct_command_is_endpoint_bound()", "fn source_drift_credential_bearing_commands_retain_endpoint_binding()"),
    ("fn direct_model_downloads_are_wired_to_verified_activation()", "fn source_drift_model_downloads_retain_verified_activation_wiring()"),
    ("fn direct_page_context_transmission_cannot_bypass_privacy_sanitization()", "fn source_drift_page_context_commands_retain_privacy_sanitizer_wiring()"),
]:
    replace_once("src-tauri/tests/post_batch8_direct_command_policy_evidence.rs", old, new)


# ---------------------------------------------------------------------------
# Policy detail serialization: always retain the primary refusal and include a
# deterministic supplemental marker if detail serialization ever fails.
# ---------------------------------------------------------------------------
replace_once(
    "src-tauri/src/commands/planner_executor/execution.rs",
    '''fn initial_execution_policy_error(''',
    '''fn policy_details(
    result: Result<serde_json::Value, serde_json::Error>,
) -> Option<serde_json::Value> {
    Some(match result {
        Ok(details) => details,
        Err(_) => serde_json::json!({ "detail_serialization": "failed" }),
    })
}

fn initial_execution_policy_error(''',
)
replace_once(
    "src-tauri/src/commands/planner_executor/execution.rs",
    "details: serde_json::to_value(decision).ok(),",
    "details: policy_details(serde_json::to_value(decision)),",
)
# The expression occurs three times; replace the remaining two deliberately.
for _ in range(2):
    replace_once(
        "src-tauri/src/commands/planner_executor/execution.rs",
        "details: serde_json::to_value(decision).ok(),",
        "details: policy_details(serde_json::to_value(decision)),",
    )
append_once(
    "src-tauri/src/commands/planner_executor/execution.rs",
    "post_p8_policy_detail_serialization_failure_keeps_typed_marker",
    '''#[cfg(test)]
mod post_p8_policy_detail_tests {
    use super::*;

    #[test]
    fn post_p8_policy_detail_serialization_failure_keeps_typed_marker() {
        let failed = serde_json::from_str::<serde_json::Value>("{");
        let details = policy_details(failed).expect("details remain present");
        assert_eq!(details["detail_serialization"], "failed");
    }
}''',
)
replace_once(
    "src-tauri/src/commands/validators/mod.rs",
    '''pub fn validate_planner_output(''',
    '''fn policy_details(
    result: Result<serde_json::Value, serde_json::Error>,
) -> Option<serde_json::Value> {
    Some(match result {
        Ok(details) => details,
        Err(_) => serde_json::json!({ "detail_serialization": "failed" }),
    })
}

pub fn validate_planner_output(''',
)
validators = read("src-tauri/src/commands/validators/mod.rs")
validators = re.sub(
    r"serde_json::to_value\(([^\n]+?)\)\.ok\(\)",
    r"policy_details(serde_json::to_value(\1))",
    validators,
)
write("src-tauri/src/commands/validators/mod.rs", validators)
append_once(
    "src-tauri/src/commands/validators/mod.rs",
    "post_p8_validator_policy_detail_failure_is_explicit",
    '''#[cfg(test)]
mod post_p8_policy_detail_tests {
    use super::*;

    #[test]
    fn post_p8_validator_policy_detail_failure_is_explicit() {
        let failed = serde_json::from_str::<serde_json::Value>("{");
        let details = policy_details(failed).expect("details remain present");
        assert_eq!(details["detail_serialization"], "failed");
    }
}''',
)


# ---------------------------------------------------------------------------
# Remove converted fallback expressions from the exact allowlist and enrich all
# remaining inventory entries with reviewed disposition metadata.
# ---------------------------------------------------------------------------
removed_allowlist = {
    'src-tauri/src/app_core/planner_redaction.rs|let _ = parsed.set_password(None);',
    'src-tauri/src/app_core/planner_redaction.rs|let _ = parsed.set_username("");',
    'src-tauri/src/app_core/settings_adapters.rs|.and_then(|configured_profile| ProviderEndpointScope::parse(&configured_profile.base_url).ok())',
    'src-tauri/src/app_core/settings_adapters.rs|.and_then(|profile| kitten_download_plan_for_model_id(&profile.model_id).ok())',
    'src-tauri/src/app_core/settings_adapters.rs|.and_then(|profile| whisper_download_plan_for_model_id(&profile.model_id).ok())',
    'src-tauri/src/app_core/settings_adapters.rs|.unwrap_or_default(),',
    'src-tauri/src/commands/planner_executor/execution.rs|details: serde_json::to_value(decision).ok(),',
    'src-tauri/src/commands/skill_loader.rs|for entry in entries.filter_map(Result::ok) {',
    'src-tauri/src/commands/validators/mod.rs|details: serde_json::to_value(&decision).ok(),',
    'src-tauri/src/commands/validators/mod.rs|details: serde_json::to_value(decision).ok(),',
    'src-tauri/src/commands/validators/mod.rs|details: serde_json::to_value(preliminary_decision).ok(),',
    'src-tauri/src/commands/validators/mod.rs|serde_json::to_value(&decision).ok(),',
    'src-tauri/src/commands/validators/mod.rs|serde_json::to_value(decision).ok(),',
}
allowlist_path = ROOT / "scripts/security-fallback-allowlist.txt"
allowlist_lines = allowlist_path.read_text(encoding="utf-8").splitlines()
missing_removed = removed_allowlist - set(allowlist_lines)
if missing_removed:
    raise SystemExit(f"expected converted allowlist entries missing: {sorted(missing_removed)}")
allowlist_lines = [line for line in allowlist_lines if line not in removed_allowlist]
allowlist_path.write_text("\n".join(allowlist_lines).rstrip() + "\n", encoding="utf-8")

inventory_path = ROOT / "scripts/security-fallback-inventory.json"
payload = json.loads(inventory_path.read_text(encoding="utf-8"))
payload["version"] = 2
removed_keys = {tuple(line.split("|", 1)) for line in removed_allowlist}
entries = [
    entry
    for entry in payload.get("entries", [])
    if (entry.get("path"), entry.get("expression")) not in removed_keys
]
temporary_paths = {
    "src-tauri/src/app_core/command_dispatch.rs",
    "src-tauri/src/app_core/fill_correction.rs",
    "src-tauri/src/app_core/form_fill/field_focus.rs",
    "src-tauri/src/commands/skill_parser.rs",
}
for entry in entries:
    if entry["path"] in temporary_paths:
        entry["disposition"] = "temporary_accepted"
        entry["review_due"] = "before_release_candidate"
        entry["owner_note"] = (
            "Promote this capability-reducing omission to a typed candidate or parse diagnostic "
            "before the release-candidate gate if the UI can render it safely."
        )
    else:
        entry["disposition"] = "permanent_accepted"
        entry["review_due"] = "not_applicable"
        entry["owner_note"] = (
            "Retain as an exact capability-reducing or presentation-only fallback; reevaluate if "
            "the expression begins affecting authority, persistence success, or a public error contract."
        )
payload["entries"] = entries
inventory_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


# Enhanced inventory scanner with disposition validation and hostile self-tests.
scanner = '''#!/usr/bin/env python3
"""Verify exact per-expression metadata for every accepted security fallback."""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ALLOWLIST = ROOT / "scripts/security-fallback-allowlist.txt"
INVENTORY = ROOT / "scripts/security-fallback-inventory.json"
REQUIRED = {
    "path",
    "functions",
    "expression",
    "justification",
    "user_visibility",
    "side_effect_impact",
    "test_coverage",
    "future_replacement",
    "disposition",
    "review_due",
    "owner_note",
}
VALID_DISPOSITIONS = {
    "permanent_accepted",
    "temporary_accepted",
    "convert_to_warning",
    "convert_to_error",
    "remove",
}


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


def source_functions(path: str, expression: str) -> list[str]:
    current = "module scope"
    functions = []
    signature = re.compile(r"\\bfn\\s+([A-Za-z_][A-Za-z0-9_]*)\\s*[<(]")
    for line in (ROOT / path).read_text(encoding="utf-8", errors="replace").splitlines():
        found = signature.search(line)
        if found:
            current = found.group(1)
        if normalize(line) == expression:
            functions.append(current)
    return sorted(set(functions))


def metadata_problems(key: tuple[str, str], entry: dict) -> list[str]:
    problems = []
    missing = REQUIRED - set(entry)
    if missing:
        return [f"{key}: missing fields {sorted(missing)}"]
    for field in REQUIRED - {"functions"}:
        if not isinstance(entry[field], str) or not entry[field].strip():
            problems.append(f"{key}: empty {field}")
    if not isinstance(entry["functions"], list) or not entry["functions"]:
        problems.append(f"{key}: functions must be a non-empty list")
    if entry["disposition"] not in VALID_DISPOSITIONS:
        problems.append(f"{key}: invalid disposition {entry['disposition']!r}")
    if entry["disposition"] == "temporary_accepted":
        if entry["review_due"] == "not_applicable":
            problems.append(f"{key}: temporary fallback requires a review boundary")
        if len(entry["owner_note"].strip()) < 20:
            problems.append(f"{key}: temporary fallback requires an actionable owner_note")
    return problems


def audit() -> list[str]:
    problems = []
    payload = json.loads(INVENTORY.read_text(encoding="utf-8"))
    entries = payload.get("entries", [])
    indexed = {(entry.get("path"), entry.get("expression")): entry for entry in entries}
    expected = allowlist_keys()
    observed = set(indexed)
    if expected != observed:
        problems.append(
            f"inventory keys differ: missing={sorted(expected-observed)} extra={sorted(observed-expected)}"
        )
    for key, entry in indexed.items():
        entry_problems = metadata_problems(key, entry)
        problems.extend(entry_problems)
        if entry_problems:
            continue
        actual_functions = source_functions(*key)
        if actual_functions != entry["functions"]:
            problems.append(f"{key}: functions {entry['functions']} != source {actual_functions}")
    return problems


def self_test() -> None:
    assert normalize("  let   x = 1; ") == "let x = 1;"
    base = {
        "path": "src-tauri/src/app_core/click_authorization.rs",
        "functions": ["example"],
        "expression": "example()",
        "justification": "safe",
        "user_visibility": "visible",
        "side_effect_impact": "none",
        "test_coverage": "unit",
        "future_replacement": "none",
        "disposition": "permanent_accepted",
        "review_due": "not_applicable",
        "owner_note": "Permanent exact fallback with no authority impact.",
    }
    missing = dict(base)
    missing.pop("disposition")
    assert "missing fields" in metadata_problems(("p", "e"), missing)[0]
    invalid = dict(base, disposition="maybe")
    assert any("invalid disposition" in problem for problem in metadata_problems(("p", "e"), invalid))
    temporary = dict(
        base,
        disposition="temporary_accepted",
        review_due="not_applicable",
        owner_note="short",
    )
    temporary_problems = metadata_problems(("p", "e"), temporary)
    assert any("review boundary" in problem for problem in temporary_problems)
    assert any("actionable owner_note" in problem for problem in temporary_problems)
    assert source_functions(
        "scripts/check-security-fallback-inventory.py", "definitely stale expression"
    ) == []
    print("Security fallback inventory self-test passed")


def main() -> int:
    if sys.argv[1:] == ["--self-test"]:
        self_test()
        return 0
    if sys.argv[1:]:
        print("usage: check-security-fallback-inventory.py [--self-test]", file=sys.stderr)
        return 2
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


# Human-readable disposition summary. The exact generated table remains intact
# and continues to be source/inventory checked by permanent CI.
counts = Counter(entry["disposition"] for entry in entries)
converted_count = len(removed_keys)
summary = f'''## Post-P8 disposition policy and counts

Every live accepted expression now carries a machine-enforced disposition in
`scripts/security-fallback-inventory.json`:

- `permanent_accepted`: {counts.get("permanent_accepted", 0)}
- `temporary_accepted`: {counts.get("temporary_accepted", 0)}
- converted or removed in this pass: {converted_count}

Temporary entries require both a concrete `review_due` boundary and an actionable
`owner_note`. Permanent entries remain exact-expression exceptions only; they must
be re-reviewed if they begin affecting authority, persistence success, or a public
error contract.

This pass converted the quiet skill-directory entry skip into bounded warning
aggregation, converted settings/model/provider ambiguity into typed absence
reasons, reconstructed sanitized URLs without ignored mutator results, and made
policy-detail serialization degradation explicit.
'''
doc_path = "docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md"
doc = read(doc_path)
doc = doc.replace(
    "**Status:** Reconciled for the post-Batch-8 hardening branch; final exact-SHA CI evidence remains pending.",
    "**Status:** Reconciled and disposition-classified for the post-P8 fallback/evidence hardening pass.",
)
pattern = re.compile(
    r"## Post-P8 disposition policy and counts\n.*?(?=\n## Exact per-expression fallback inventory)",
    re.S,
)
if pattern.search(doc):
    doc = pattern.sub(summary.rstrip(), doc)
else:
    marker = "## Exact per-expression fallback inventory"
    if marker not in doc:
        raise SystemExit("accepted fallback inventory marker missing")
    doc = doc.replace(marker, summary.rstrip() + "\n\n" + marker, 1)
write(doc_path, doc)

print("Applied post-P8 fallback/evidence hardening patch")
