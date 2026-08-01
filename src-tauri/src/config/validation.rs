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
