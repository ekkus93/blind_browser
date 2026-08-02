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

pub(in crate::config) fn normalize_remote_planner_privacy_settings(
    settings: &mut RemotePlannerPrivacySettings,
    issues: &mut Vec<String>,
) {
    match normalize_remote_planner_blocked_origins(&settings.blocked_origins) {
        Ok(origins) => settings.blocked_origins = origins,
        Err(error) => issues.push(error.to_string()),
    }
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
}
