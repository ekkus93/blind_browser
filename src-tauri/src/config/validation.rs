use std::collections::BTreeSet;

use super::*;
use crate::provider_endpoint::ProviderEndpointScope;

pub(in crate::config) fn validate_audio_settings(audio: &AudioSettings, issues: &mut Vec<String>) {
    if !(MIN_PLAYBACK_VOLUME..=MAX_PLAYBACK_VOLUME).contains(&audio.playback_volume) {
        issues.push(String::from(
            "audio.playback_volume must be between 0.0 and 1.0",
        ));
    }

    if !(MIN_PLAYBACK_SPEED..=MAX_PLAYBACK_SPEED).contains(&audio.playback_speed) {
        issues.push(format!(
            "audio.playback_speed must be between {MIN_PLAYBACK_SPEED} and {MAX_PLAYBACK_SPEED}"
        ));
    }

    if audio.default_tts_voice.trim().is_empty() {
        issues.push(String::from("audio.default_tts_voice must not be empty"));
    }
}

pub(in crate::config) fn validate_safety_settings(
    safety: &SafetySettings,
    issues: &mut Vec<String>,
) {
    if !(0.0..=1.0).contains(&safety.confirmation_confidence_threshold) {
        issues.push(String::from(
            "safety.confirmation_confidence_threshold must be between 0.0 and 1.0",
        ));
    }

    // CR3 P3.1.2: docs/SPECS.md has documented "`always_confirm_submit` must
    // remain `true` in v1" as a validation rule since the doc's original
    // draft, but nothing ever enforced it -- a hand-edited
    // `always_confirm_submit = false` loaded without error. It never actually
    // weakened submit-confirmation behavior (action_policy.rs's
    // `SubmitActiveForm` arm hard-codes `ConfirmationRequirement::
    // ConfirmationRequired` regardless of this field's value -- see the
    // comment there), but silently accepting a value that has no effect is
    // itself misleading: it invites a user or future change to believe
    // setting it `false` does something. Rejecting it here makes the
    // documented invariant real instead of aspirational.
    if !safety.always_confirm_submit {
        issues.push(String::from(
            "safety.always_confirm_submit must remain true in v1",
        ));
    }
}

fn validate_remote_profile_base_url(name: &str, base_url: &str, issues: &mut Vec<String>) {
    if let Err(reason) = ProviderEndpointScope::parse(base_url) {
        issues.push(format!(
            "remote_profiles.{name}.base_url is invalid: {reason}"
        ));
    }
}

pub(in crate::config) fn validate_remote_planner_profile(
    name: &str,
    profile: &RemotePlannerProfile,
    issues: &mut Vec<String>,
) {
    validate_remote_profile_base_url(name, &profile.base_url, issues);
    if profile.timeout_ms == 0 {
        issues.push(format!(
            "remote_profiles.{name}.timeout_ms must be positive"
        ));
    }
    if profile.max_output_tokens == 0 {
        issues.push(format!(
            "remote_profiles.{name}.max_output_tokens must be positive"
        ));
    }
    if profile.temperature_milli > MAX_TEMPERATURE_MILLI {
        issues.push(format!(
            "remote_profiles.{name}.temperature_milli must be between 0 and {MAX_TEMPERATURE_MILLI}"
        ));
    }
}

pub(in crate::config) fn validate_remote_tts_profile(
    name: &str,
    profile: &RemoteTtsProfile,
    issues: &mut Vec<String>,
) {
    validate_remote_profile_base_url(name, &profile.base_url, issues);
    if profile.timeout_ms == 0 {
        issues.push(format!(
            "remote_profiles.{name}.timeout_ms must be positive"
        ));
    }
}

pub(in crate::config) fn validate_remote_asr_profile(
    name: &str,
    profile: &RemoteAsrProfile,
    issues: &mut Vec<String>,
) {
    validate_remote_profile_base_url(name, &profile.base_url, issues);
    if profile.timeout_ms == 0 {
        issues.push(format!(
            "remote_profiles.{name}.timeout_ms must be positive"
        ));
    }
    if profile.temperature_milli > MAX_TEMPERATURE_MILLI {
        issues.push(format!(
            "remote_profiles.{name}.temperature_milli must be between 0 and {MAX_TEMPERATURE_MILLI}"
        ));
    }
}

pub(in crate::config) fn validate_local_tts_profile(
    name: &str,
    profile: &LocalTtsProfile,
    issues: &mut Vec<String>,
) {
    if profile.model_path.trim().is_empty() {
        issues.push(format!(
            "local_profiles.{name}.model_path must not be empty"
        ));
    }
    if profile.sample_rate == 0 {
        issues.push(format!(
            "local_profiles.{name}.sample_rate must be positive"
        ));
    }
}

pub(in crate::config) fn validate_local_asr_profile(
    name: &str,
    profile: &LocalAsrProfile,
    issues: &mut Vec<String>,
) {
    if profile.model_path.trim().is_empty() {
        issues.push(format!(
            "local_profiles.{name}.model_path must not be empty"
        ));
    }
    if profile.threads == 0 {
        issues.push(format!("local_profiles.{name}.threads must be positive"));
    }
}

pub(in crate::config) fn validate_ocr_settings(ocr: &OcrSettings, issues: &mut Vec<String>) {
    if ocr.sparse_text_char_threshold == 0 {
        issues.push(String::from(
            "ocr.sparse_text_char_threshold must be greater than 0",
        ));
    }

    if ocr.sparse_text_region_threshold == 0 {
        issues.push(String::from(
            "ocr.sparse_text_region_threshold must be greater than 0",
        ));
    }
}

pub(in crate::config) fn normalize_remote_planner_blocked_origins(
    origins: &[String],
) -> Result<Vec<String>, ConfigError> {
    if origins.len() > 128 {
        return Err(ConfigError::Validation(String::from(
            "remote_planner_privacy.blocked_origins must contain at most 128 origins",
        )));
    }

    let mut normalized = BTreeSet::new();
    for raw in origins {
        let raw = raw.trim();
        if raw.is_empty() {
            continue;
        }
        let parsed = url::Url::parse(raw).map_err(|error| {
            ConfigError::Validation(format!(
                "remote_planner_privacy blocked origin must be an absolute URL origin: {error}"
            ))
        })?;
        if !matches!(parsed.scheme(), "http" | "https")
            || parsed.host_str().is_none()
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || parsed.query().is_some()
            || parsed.fragment().is_some()
            || !matches!(parsed.path(), "" | "/")
        {
            return Err(ConfigError::Validation(format!(
                "remote_planner_privacy blocked origin must contain only scheme, host, and optional port: {raw}"
            )));
        }
        normalized.insert(parsed.origin().ascii_serialization());
    }
    Ok(normalized.into_iter().collect())
}

fn normalize_remote_planner_origin(raw: &str) -> Result<String, ConfigError> {
    let parsed = url::Url::parse(raw.trim()).map_err(|error| {
        ConfigError::Validation(format!(
            "remote planner origin rule must use an absolute HTTP(S) origin: {error}"
        ))
    })?;
    if !matches!(parsed.scheme(), "http" | "https")
        || parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || !matches!(parsed.path(), "" | "/")
    {
        return Err(ConfigError::Validation(format!(
            "remote planner origin rule must contain only scheme, host, and optional port: {raw}"
        )));
    }
    Ok(parsed.origin().ascii_serialization())
}

fn normalize_remote_planner_origin_rules(
    rules: &[RemotePlannerOriginRule],
) -> Result<Vec<RemotePlannerOriginRule>, ConfigError> {
    if rules.len() > 256 {
        return Err(ConfigError::Validation(String::from(
            "remote_planner_privacy.origin_rules must contain at most 256 rules",
        )));
    }
    let mut normalized = Vec::with_capacity(rules.len());
    for rule in rules {
        let page_origin = normalize_remote_planner_origin(&rule.page_origin)?;
        let endpoint_scope = match rule.decision {
            PersistedOriginDecision::Block => {
                if rule.endpoint_scope.is_some() {
                    return Err(ConfigError::Validation(format!(
                        "origin-wide block for {page_origin} must not contain an endpoint scope"
                    )));
                }
                None
            }
            PersistedOriginDecision::Allow => {
                let raw = rule.endpoint_scope.as_deref().ok_or_else(|| {
                    ConfigError::Validation(format!(
                        "persistent allow for {page_origin} requires an endpoint scope"
                    ))
                })?;
                Some(
                    ProviderEndpointScope::parse(raw)
                        .map_err(ConfigError::Validation)?
                        .normalized_base_url()
                        .to_string(),
                )
            }
        };
        normalized.push(RemotePlannerOriginRule {
            page_origin,
            decision: rule.decision.clone(),
            endpoint_scope,
            policy_version: rule.policy_version,
            created_at_ms: rule.created_at_ms,
        });
    }
    normalized.sort_by(|left, right| {
        (
            left.page_origin.as_str(),
            &left.decision,
            left.endpoint_scope.as_deref(),
            left.policy_version,
            left.created_at_ms,
        )
            .cmp(&(
                right.page_origin.as_str(),
                &right.decision,
                right.endpoint_scope.as_deref(),
                right.policy_version,
                right.created_at_ms,
            ))
    });
    normalized.dedup_by(|left, right| {
        left.page_origin == right.page_origin
            && left.decision == right.decision
            && left.endpoint_scope == right.endpoint_scope
            && left.policy_version == right.policy_version
    });
    Ok(normalized)
}

pub(in crate::config) fn normalize_remote_planner_privacy_settings(
    settings: &mut RemotePlannerPrivacySettings,
    issues: &mut Vec<String>,
) {
    let normalized_legacy_origins =
        match normalize_remote_planner_blocked_origins(&settings.blocked_origins) {
            Ok(origins) => origins,
            Err(error) => {
                issues.push(error.to_string());
                Vec::new()
            }
        };

    if settings.policy_schema_version == 0 {
        settings.network_mode = if settings.local_only {
            RemotePlannerNetworkMode::LocalOnly
        } else if settings.consent_to_remote_page_data {
            RemotePlannerNetworkMode::AllowSanitizedNonHighRisk
        } else {
            RemotePlannerNetworkMode::AskPerOrigin
        };
        for page_origin in &normalized_legacy_origins {
            settings.origin_rules.push(RemotePlannerOriginRule {
                page_origin: page_origin.clone(),
                decision: PersistedOriginDecision::Block,
                endpoint_scope: None,
                policy_version: REMOTE_DATA_POLICY_VERSION,
                created_at_ms: 0,
            });
        }
        settings.policy_schema_version = REMOTE_DATA_POLICY_VERSION;
        settings.migration_notice_pending = true;
    } else if settings.policy_schema_version > REMOTE_DATA_POLICY_VERSION {
        issues.push(format!(
            "remote_planner_privacy.policy_schema_version {} is newer than supported version {}",
            settings.policy_schema_version, REMOTE_DATA_POLICY_VERSION
        ));
    }

    match normalize_remote_planner_origin_rules(&settings.origin_rules) {
        Ok(rules) => settings.origin_rules = rules,
        Err(error) => issues.push(error.to_string()),
    }

    // Keep legacy fields synchronized while they remain readable. They are never
    // authoritative after normalization.
    settings.local_only = matches!(settings.network_mode, RemotePlannerNetworkMode::LocalOnly);
    settings.consent_to_remote_page_data = matches!(
        settings.network_mode,
        RemotePlannerNetworkMode::AllowSanitizedNonHighRisk
    );
    settings.blocked_origins = settings
        .origin_rules
        .iter()
        .filter(|rule| matches!(rule.decision, PersistedOriginDecision::Block))
        .map(|rule| rule.page_origin.clone())
        .collect();
    settings.blocked_origins.sort();
    settings.blocked_origins.dedup();
}

pub(in crate::config) fn validate_model_settings(
    models: &ModelManagementSettings,
    issues: &mut Vec<String>,
) {
    if models.models_dir.trim().is_empty() {
        issues.push(String::from("models.models_dir must not be empty"));
    }
}

pub(in crate::config) fn normalize_remote_endpoint(base_url: &str) -> Result<String, ConfigError> {
    ProviderEndpointScope::parse(base_url)
        .map(|scope| scope.normalized_base_url().to_string())
        .map_err(ConfigError::Validation)
}

#[cfg(test)]
mod privacy_tests {
    use super::*;

    #[test]
    fn blocked_origins_are_normalized_deduplicated_and_sorted() {
        let origins = vec![
            String::from("https://EXAMPLE.com:443/"),
            String::from("https://example.com"),
            String::from("http://localhost:3000/"),
        ];
        assert_eq!(
            normalize_remote_planner_blocked_origins(&origins).unwrap(),
            vec![
                String::from("http://localhost:3000"),
                String::from("https://example.com"),
            ]
        );
    }

    #[test]
    fn blocked_origins_reject_paths_credentials_queries_and_non_http_schemes() {
        for origin in [
            "https://example.com/private",
            "https://user:pass@example.com",
            "https://example.com?token=secret",
            "file:///tmp/private",
        ] {
            assert!(normalize_remote_planner_blocked_origins(&[origin.to_string()]).is_err());
        }
    }

    #[test]
    fn legacy_privacy_settings_migrate_idempotently() {
        let mut settings = RemotePlannerPrivacySettings {
            consent_to_remote_page_data: true,
            local_only: false,
            blocked_origins: vec![String::from("https://EXAMPLE.com:443")],
            network_mode: RemotePlannerNetworkMode::AskPerOrigin,
            origin_rules: Vec::new(),
            policy_schema_version: 0,
            migration_notice_pending: false,
        };
        let mut issues = Vec::new();
        normalize_remote_planner_privacy_settings(&mut settings, &mut issues);
        assert!(issues.is_empty());
        assert_eq!(
            settings.network_mode,
            RemotePlannerNetworkMode::AllowSanitizedNonHighRisk
        );
        assert_eq!(settings.policy_schema_version, REMOTE_DATA_POLICY_VERSION);
        assert!(settings.migration_notice_pending);
        assert_eq!(settings.origin_rules.len(), 1);
        assert_eq!(settings.origin_rules[0].page_origin, "https://example.com");
        let first = settings.clone();
        normalize_remote_planner_privacy_settings(&mut settings, &mut issues);
        assert_eq!(settings, first);
    }

    #[test]
    fn destination_bound_allows_and_origin_wide_blocks_are_validated() {
        let mut settings = RemotePlannerPrivacySettings {
            network_mode: RemotePlannerNetworkMode::AskPerOrigin,
            origin_rules: vec![
                RemotePlannerOriginRule {
                    page_origin: String::from("https://EXAMPLE.com:443"),
                    decision: PersistedOriginDecision::Allow,
                    endpoint_scope: Some(String::from("https://api.example.com:443/v1/")),
                    policy_version: REMOTE_DATA_POLICY_VERSION,
                    created_at_ms: 2,
                },
                RemotePlannerOriginRule {
                    page_origin: String::from("https://private.example"),
                    decision: PersistedOriginDecision::Block,
                    endpoint_scope: None,
                    policy_version: REMOTE_DATA_POLICY_VERSION,
                    created_at_ms: 1,
                },
            ],
            ..Default::default()
        };
        let mut issues = Vec::new();
        normalize_remote_planner_privacy_settings(&mut settings, &mut issues);
        assert!(issues.is_empty(), "{issues:?}");
        assert_eq!(settings.origin_rules[0].page_origin, "https://example.com");
        assert_eq!(
            settings.origin_rules[0].endpoint_scope.as_deref(),
            Some("https://api.example.com/v1")
        );
        assert_eq!(settings.blocked_origins, vec!["https://private.example"]);
    }
}
