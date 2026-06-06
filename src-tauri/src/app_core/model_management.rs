use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{LocalAsrProfile, LocalTtsProfile};
use reqwest::blocking::Client;

pub(crate) struct KittenDownloadPlan {
    pub(crate) repository: &'static str,
    pub(crate) directory_name: &'static str,
    pub(crate) display_name: &'static str,
    pub(crate) files: &'static [&'static str],
}

pub(crate) struct WhisperDownloadPlan {
    pub(crate) repository: &'static str,
    pub(crate) display_name: &'static str,
    pub(crate) file_name: &'static str,
}

pub(crate) fn local_tts_model_is_available(profile: &LocalTtsProfile) -> bool {
    let model_path = Path::new(profile.model_path.trim());
    if !model_path.is_dir() {
        return false;
    }

    let has_config = model_path.join("config.json").is_file();
    let has_voices = model_path.join("voices.npz").is_file();
    let has_onnx = fs::read_dir(model_path)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .any(|entry| {
            entry
                .path()
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("onnx"))
        });

    has_config && has_voices && has_onnx
}

pub(crate) fn local_asr_model_is_available(profile: &LocalAsrProfile) -> bool {
    Path::new(profile.model_path.trim()).is_file()
}

pub(crate) fn kitten_download_plan_for_model_id(
    model_id: &str,
) -> Result<KittenDownloadPlan, String> {
    let normalized = model_id.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "default" | "mini" | "kitten-tts-mini" => Ok(KittenDownloadPlan {
            repository: "KittenML/kitten-tts-mini-0.8",
            directory_name: "kitten-tts-mini",
            display_name: "KittenTTS mini model",
            files: &["config.json", "kitten_tts_mini_v0_8.onnx", "voices.npz"],
        }),
        "micro" | "kitten-tts-micro" => Ok(KittenDownloadPlan {
            repository: "KittenML/kitten-tts-micro-0.8",
            directory_name: "kitten-tts-micro",
            display_name: "KittenTTS micro model",
            files: &["config.json", "kitten_tts_micro_v0_8.onnx", "voices.npz"],
        }),
        "nano" | "kitten-tts-nano" => Ok(KittenDownloadPlan {
            repository: "KittenML/kitten-tts-nano-0.8-fp32",
            directory_name: "kitten-tts-nano",
            display_name: "KittenTTS nano model",
            files: &["config.json", "kitten_tts_nano_v0_8.onnx", "voices.npz"],
        }),
        "nano-int8" | "kitten-tts-nano-int8" => Ok(KittenDownloadPlan {
            repository: "KittenML/kitten-tts-nano-0.8-int8",
            directory_name: "kitten-tts-nano-int8",
            display_name: "KittenTTS nano int8 model",
            files: &[
                "config.json",
                "kitten_tts_nano_v0_8_int8.onnx",
                "voices.npz",
            ],
        }),
        _ => Err(format!(
            "local TTS model_id '{}' does not have a known Hugging Face download mapping",
            model_id.trim()
        )),
    }
}

pub(crate) fn whisper_download_plan_for_model_id(
    model_id: &str,
) -> Result<WhisperDownloadPlan, String> {
    let normalized = model_id.trim().to_ascii_lowercase();
    let file_name = match normalized.as_str() {
        "tiny" => "ggml-tiny.bin",
        "base" => "ggml-base.bin",
        "small" => "ggml-small.bin",
        "medium" => "ggml-medium.bin",
        "large-v3" => "ggml-large-v3.bin",
        "large-v3-turbo" => "ggml-large-v3-turbo.bin",
        _ => {
            return Err(format!(
                "local ASR model_id '{}' does not have a known Hugging Face download mapping",
                model_id.trim()
            ))
        }
    };

    Ok(WhisperDownloadPlan {
        repository: "ggerganov/whisper.cpp",
        display_name: match normalized.as_str() {
            "tiny" => "tiny model",
            "base" => "base model",
            "small" => "small model",
            "medium" => "medium model",
            "large-v3" => "large-v3 model",
            "large-v3-turbo" => "large-v3-turbo model",
            _ => unreachable!(),
        },
        file_name,
    })
}

pub(crate) fn download_hugging_face_directory(
    target_dir: &Path,
    repository: &str,
    files: &[&str],
) -> Result<(), String> {
    fs::create_dir_all(target_dir).map_err(|error| {
        format!(
            "Failed to create model directory {}: {error}",
            target_dir.display()
        )
    })?;
    for file_name in files {
        let target_path = target_dir.join(file_name);
        download_hugging_face_file(&target_path, repository, file_name)?;
    }
    Ok(())
}

pub(crate) fn download_hugging_face_file(
    target_path: &Path,
    repository: &str,
    file_name: &str,
) -> Result<(), String> {
    let parent = target_path.parent().ok_or_else(|| {
        format!(
            "Failed to resolve the parent directory for download target {}",
            target_path.display()
        )
    })?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Failed to create directory {}: {error}", parent.display()))?;

    let url = format!("https://huggingface.co/{repository}/resolve/main/{file_name}");
    let client = Client::builder()
        .build()
        .map_err(|error| format!("Failed to create the download client: {error}"))?;
    let mut response = client
        .get(&url)
        .send()
        .map_err(|error| format!("Failed to download {url}: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Hugging Face returned {} while downloading {}",
            response.status(),
            url
        ));
    }

    let mut output = fs::File::create(target_path)
        .map_err(|error| format!("Failed to create {}: {error}", target_path.display()))?;
    response
        .copy_to(&mut output)
        .map_err(|error| format!("Failed to write {}: {error}", target_path.display()))?;
    Ok(())
}

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
        let Some(home_dir) = app_handle.path().home_dir().ok() else {
            return Err(String::from(
                "Failed to resolve the current user's home directory.",
            ));
        };
        return Ok(home_dir.join(relative_to_home));
    }

    let candidate = PathBuf::from(trimmed);
    if candidate.is_absolute() {
        return Ok(candidate);
    }

    let config_path =
        crate::config::AppConfig::config_path_for_app(app_handle)
            .map_err(|error| error.to_string())?;
    let config_dir = config_path.parent().ok_or_else(|| {
        format!(
            "Failed to resolve the parent config directory for {}",
            config_path.display()
        )
    })?;
    Ok(config_dir.join(candidate))
}
