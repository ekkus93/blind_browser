use std::path::PathBuf;

mod availability;
mod download;
mod manifest;

pub(crate) use availability::{local_asr_model_is_available, local_tts_model_is_available};
pub(crate) use download::{download_hugging_face_directory, download_hugging_face_file};
pub(crate) use manifest::{kitten_download_plan_for_model_id, whisper_download_plan_for_model_id};

pub(crate) fn resolved_models_dir_for_app(
    app_handle: &tauri::AppHandle,
    configured_models_dir: &str,
) -> Result<PathBuf, String> {
    use tauri::Manager;

    let trimmed = configured_models_dir.trim();
    if trimmed.is_empty() {
        return Err(String::from("Configured models_dir must not be empty."));
    }

    if let Some(relative_to_home) = trimmed.strip_prefix("~/") {
        let home_dir = app_handle.path().home_dir().map_err(|error| {
            format!("Failed to resolve the current user's home directory: {error}")
        })?;
        return Ok(home_dir.join(relative_to_home));
    }

    let candidate = PathBuf::from(trimmed);
    if candidate.is_absolute() {
        return Ok(candidate);
    }

    let config_path = crate::config::AppConfig::config_path_for_app(app_handle)
        .map_err(|error| error.to_string())?;
    let config_dir = config_path.parent().ok_or_else(|| {
        format!(
            "Failed to resolve the parent config directory for {}",
            config_path.display()
        )
    })?;
    Ok(config_dir.join(candidate))
}

#[cfg(test)]
mod tests;
