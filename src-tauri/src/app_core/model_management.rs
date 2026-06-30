use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{LocalAsrProfile, LocalTtsProfile};
use reqwest::blocking::Client;

const MIN_LOCAL_ASR_MODEL_BYTES: u64 = 1_000_000; // 1 MB sanity floor
const MIN_TTS_CONFIG_BYTES: u64 = 2;
const MIN_TTS_VOICES_BYTES: u64 = 1_000;
const MIN_TTS_ONNX_BYTES: u64 = 1_000_000; // 1 MB sanity floor

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

fn file_size_at_least(path: &Path, min_bytes: u64) -> bool {
    path.metadata()
        .map(|metadata| metadata.is_file() && metadata.len() >= min_bytes)
        .unwrap_or(false)
}

pub(crate) fn local_tts_model_is_available(profile: &LocalTtsProfile) -> bool {
    let model_path = Path::new(profile.model_path.trim());
    if !model_path.is_dir() {
        return false;
    }

    let has_config = file_size_at_least(&model_path.join("config.json"), MIN_TTS_CONFIG_BYTES);
    let has_voices = file_size_at_least(&model_path.join("voices.npz"), MIN_TTS_VOICES_BYTES);
    let has_onnx = fs::read_dir(model_path)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .any(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("onnx"))
                && file_size_at_least(&path, MIN_TTS_ONNX_BYTES)
        });

    has_config && has_voices && has_onnx
}

pub(crate) fn local_asr_model_is_available(profile: &LocalAsrProfile) -> bool {
    file_size_at_least(
        Path::new(profile.model_path.trim()),
        MIN_LOCAL_ASR_MODEL_BYTES,
    )
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
    let (file_name, display_name) = match normalized.as_str() {
        "tiny" => ("ggml-tiny.bin", "tiny model"),
        "base" => ("ggml-base.bin", "base model"),
        "small" => ("ggml-small.bin", "small model"),
        "medium" => ("ggml-medium.bin", "medium model"),
        "large-v3" => ("ggml-large-v3.bin", "large-v3 model"),
        "large-v3-turbo" => ("ggml-large-v3-turbo.bin", "large-v3-turbo model"),
        _ => {
            return Err(format!(
                "local ASR model_id '{}' does not have a known Hugging Face download mapping",
                model_id.trim()
            ))
        }
    };
    Ok(WhisperDownloadPlan {
        repository: "ggerganov/whisper.cpp",
        display_name,
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
    let url = format!("https://huggingface.co/{repository}/resolve/main/{file_name}");
    let client = Client::builder()
        .build()
        .map_err(|error| format!("Failed to create the download client: {error}"))?;
    let response = client
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

    download_response_to_file_atomically(response, target_path)
}

fn download_response_to_file_atomically(
    mut response: reqwest::blocking::Response,
    target_path: &Path,
) -> Result<(), String> {
    let parent = target_path.parent().ok_or_else(|| {
        format!(
            "download target {} has no parent directory",
            target_path.display()
        )
    })?;

    fs::create_dir_all(parent)
        .map_err(|error| format!("failed to create model directory {}: {error}", parent.display()))?;

    let file_name = target_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            format!(
                "download target {} has no valid file name",
                target_path.display()
            )
        })?;

    let tmp_path = target_path.with_file_name(format!("{file_name}.part"));

    let result = (|| -> Result<(), String> {
        let mut output = fs::File::create(&tmp_path).map_err(|error| {
            format!(
                "failed to create temporary model file {}: {error}",
                tmp_path.display()
            )
        })?;

        response.copy_to(&mut output).map_err(|error| {
            format!("failed to write model file {}: {error}", tmp_path.display())
        })?;

        output.sync_all().map_err(|error| {
            format!("failed to sync model file {}: {error}", tmp_path.display())
        })?;

        fs::rename(&tmp_path, target_path).map_err(|error| {
            format!(
                "failed to finalize model file {} from {}: {error}",
                target_path.display(),
                tmp_path.display()
            )
        })?;

        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }

    result
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
mod tests {
    use super::*;
    use crate::config::{LocalAsrBackend, LocalAsrProfile, LocalTtsBackend, LocalTtsProfile};

    fn asr_profile_at(path: &str) -> LocalAsrProfile {
        LocalAsrProfile {
            backend: LocalAsrBackend::Whisper,
            model_id: String::from("tiny"),
            model_path: path.to_string(),
            language: Some(String::from("en")),
            threads: 4,
        }
    }

    fn tts_profile_at(path: &str) -> LocalTtsProfile {
        LocalTtsProfile {
            backend: LocalTtsBackend::KittenTtsRs,
            model_id: String::from("default"),
            model_path: path.to_string(),
            default_voice: String::from("Bruno"),
            sample_rate: 24_000,
        }
    }

    #[test]
    fn local_asr_model_availability_rejects_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let model_path = dir.path().join("tiny.gguf");
        std::fs::write(&model_path, []).unwrap();
        let profile = asr_profile_at(model_path.to_str().unwrap());
        assert!(!local_asr_model_is_available(&profile));
    }

    #[test]
    fn local_asr_model_availability_rejects_file_below_minimum_size() {
        let dir = tempfile::tempdir().unwrap();
        let model_path = dir.path().join("tiny.gguf");
        std::fs::write(&model_path, vec![0u8; 100]).unwrap();
        let profile = asr_profile_at(model_path.to_str().unwrap());
        assert!(!local_asr_model_is_available(&profile));
    }

    #[test]
    fn local_asr_model_availability_rejects_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let profile = asr_profile_at(dir.path().join("missing.gguf").to_str().unwrap());
        assert!(!local_asr_model_is_available(&profile));
    }

    #[test]
    fn local_tts_model_availability_rejects_empty_config_file() {
        let dir = tempfile::tempdir().unwrap();
        let model_dir = dir.path();
        std::fs::write(model_dir.join("config.json"), []).unwrap();
        std::fs::write(model_dir.join("voices.npz"), vec![0u8; 2_000]).unwrap();
        // Write a plausibly-sized .onnx to ensure only config.json triggers rejection.
        std::fs::write(model_dir.join("model.onnx"), vec![0u8; 2_000_000]).unwrap();
        let profile = tts_profile_at(model_dir.to_str().unwrap());
        assert!(!local_tts_model_is_available(&profile));
    }

    #[test]
    fn local_tts_model_availability_rejects_empty_onnx_file() {
        let dir = tempfile::tempdir().unwrap();
        let model_dir = dir.path();
        std::fs::write(model_dir.join("config.json"), b"{}").unwrap();
        std::fs::write(model_dir.join("voices.npz"), vec![0u8; 2_000]).unwrap();
        std::fs::write(model_dir.join("model.onnx"), []).unwrap();
        let profile = tts_profile_at(model_dir.to_str().unwrap());
        assert!(!local_tts_model_is_available(&profile));
    }

    #[test]
    fn local_tts_model_availability_rejects_missing_directory() {
        let dir = tempfile::tempdir().unwrap();
        let profile = tts_profile_at(dir.path().join("nonexistent-model").to_str().unwrap());
        assert!(!local_tts_model_is_available(&profile));
    }
}
