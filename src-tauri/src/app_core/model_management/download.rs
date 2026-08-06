use std::collections::BTreeSet;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

use reqwest::blocking::{Client, Response};
use reqwest::redirect::{Attempt, Policy};
use sha2::{Digest, Sha256};
use url::Url;

use crate::resource_limits::{model_downloads, record_resource_size, MAX_MODEL_DOWNLOAD_BYTES};

use super::manifest::{
    validate_file_manifest, verified_file_for, verified_repository_plan, ModelDownloadError,
    VerifiedModelFile,
};

pub(super) const MODEL_DOWNLOAD_CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
pub(super) const MODEL_DOWNLOAD_REQUEST_TIMEOUT: Duration = Duration::from_secs(30 * 60);
const MAX_MODEL_REDIRECTS: usize = 5;

pub(crate) fn download_hugging_face_directory(
    target_dir: &Path,
    repository: &str,
    files: &[&str],
) -> Result<(), String> {
    let _permit = model_downloads()
        .try_acquire()
        .map_err(|limit| format!("Model download was not started: {limit}."))?;
    let plan = verified_repository_plan(repository).map_err(|error| error.to_string())?;
    let requested = files.iter().copied().collect::<BTreeSet<_>>();
    let expected = plan
        .files
        .iter()
        .map(|file| file.file_name)
        .collect::<BTreeSet<_>>();
    if requested != expected {
        return Err(ModelDownloadError::InvalidManifest {
            file_name: repository.to_string(),
            reason: String::from("requested directory files do not match the pinned manifest"),
        }
        .to_string());
    }

    for file in plan.files {
        download_verified_hugging_face_file(
            &target_dir.join(file.file_name),
            plan.repository,
            plan.revision,
            file,
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub(crate) fn download_hugging_face_file(
    target_path: &Path,
    repository: &str,
    file_name: &str,
) -> Result<(), String> {
    let _permit = model_downloads()
        .try_acquire()
        .map_err(|limit| format!("Model download was not started: {limit}."))?;
    let (revision, file) =
        verified_file_for(repository, file_name).map_err(|error| error.to_string())?;
    download_verified_hugging_face_file(target_path, repository, revision, file)
        .map_err(|error| error.to_string())
}

fn download_verified_hugging_face_file(
    target_path: &Path,
    repository: &str,
    revision: &str,
    file: &VerifiedModelFile,
) -> Result<(), ModelDownloadError> {
    let url = format!(
        "https://huggingface.co/{repository}/resolve/{revision}/{}",
        file.file_name
    );
    let client = model_download_client()?;
    let response = client.get(&url).send().map_err(|error| {
        if error.is_timeout() {
            ModelDownloadError::RequestTimedOut {
                file_name: file.file_name.to_string(),
            }
        } else {
            ModelDownloadError::RequestFailed {
                file_name: file.file_name.to_string(),
                reason: error.to_string(),
            }
        }
    })?;

    if response.status().is_redirection() {
        return Err(ModelDownloadError::RedirectRefused {
            file_name: file.file_name.to_string(),
            destination: safe_redirect_destination(&response),
        });
    }
    if !response.status().is_success() {
        return Err(ModelDownloadError::HttpStatus {
            file_name: file.file_name.to_string(),
            status: response.status().as_u16(),
        });
    }
    let maximum = match file.max_bytes {
        Some(maximum) => maximum,
        None => MAX_MODEL_DOWNLOAD_BYTES,
    };
    if response
        .content_length()
        .is_some_and(|declared| declared > maximum || declared > MAX_MODEL_DOWNLOAD_BYTES)
    {
        return Err(ModelDownloadError::TooLarge {
            file_name: file.file_name.to_string(),
            maximum: maximum.min(MAX_MODEL_DOWNLOAD_BYTES),
        });
    }

    write_verified_reader_atomically(response, target_path, file)
}

pub(super) fn model_download_client() -> Result<Client, ModelDownloadError> {
    Client::builder()
        .connect_timeout(MODEL_DOWNLOAD_CONNECT_TIMEOUT)
        .timeout(MODEL_DOWNLOAD_REQUEST_TIMEOUT)
        .redirect(Policy::custom(model_redirect_policy))
        .build()
        .map_err(|error| ModelDownloadError::ClientBuild {
            reason: error.to_string(),
        })
}

fn model_redirect_policy(attempt: Attempt<'_>) -> reqwest::redirect::Action {
    if attempt.previous().len() >= MAX_MODEL_REDIRECTS {
        return attempt.stop();
    }
    if model_redirect_url_allowed(attempt.url()) {
        attempt.follow()
    } else {
        attempt.stop()
    }
}

pub(super) fn model_redirect_url_allowed(url: &Url) -> bool {
    if url.scheme() != "https" || !url.username().is_empty() || url.password().is_some() {
        return false;
    }
    let Some(host) = url.host_str().map(str::to_ascii_lowercase) else {
        return false;
    };
    host == "huggingface.co"
        || host.ends_with(".huggingface.co")
        || host == "hf.co"
        || host.ends_with(".hf.co")
        || host == "xethub.hf.co"
        || host.ends_with(".xethub.hf.co")
}

fn safe_redirect_destination(response: &Response) -> String {
    let raw = match response.headers().get(reqwest::header::LOCATION) {
        Some(value) => match value.to_str() {
            Ok(value) => value,
            Err(_) => return String::from(" to an invalid destination"),
        },
        None => return String::new(),
    };
    let parsed = match Url::parse(raw) {
        Ok(parsed) => parsed,
        Err(_) => return String::from(" to an invalid destination"),
    };
    let Some(host) = parsed.host_str() else {
        return String::from(" to an invalid destination");
    };
    format!(" to host '{host}'")
}

pub(super) fn write_verified_reader_atomically<R: Read>(
    mut reader: R,
    target_path: &Path,
    file: &VerifiedModelFile,
) -> Result<(), ModelDownloadError> {
    validate_file_manifest(file)?;
    let parent = target_path
        .parent()
        .ok_or_else(|| ModelDownloadError::InvalidManifest {
            file_name: file.file_name.to_string(),
            reason: String::from("target has no parent directory"),
        })?;
    fs::create_dir_all(parent).map_err(|error| ModelDownloadError::CreateDirectory {
        file_name: file.file_name.to_string(),
        reason: error.to_string(),
    })?;
    let target_name = target_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| ModelDownloadError::InvalidManifest {
            file_name: file.file_name.to_string(),
            reason: String::from("target has no valid UTF-8 file name"),
        })?;
    let temporary_path = target_path.with_file_name(format!("{target_name}.part"));

    let result = (|| -> Result<(), ModelDownloadError> {
        let mut output = fs::File::create(&temporary_path).map_err(|error| {
            ModelDownloadError::CreateTemporary {
                file_name: file.file_name.to_string(),
                reason: error.to_string(),
            }
        })?;
        let mut hasher = Sha256::new();
        let mut total = 0_u64;
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let read =
                reader
                    .read(&mut buffer)
                    .map_err(|error| ModelDownloadError::ReadResponse {
                        file_name: file.file_name.to_string(),
                        reason: error.to_string(),
                    })?;
            if read == 0 {
                break;
            }
            total = total.saturating_add(read as u64);
            if let Some(maximum) = file.max_bytes.filter(|maximum| total > *maximum) {
                return Err(ModelDownloadError::TooLarge {
                    file_name: file.file_name.to_string(),
                    maximum,
                });
            }
            output.write_all(&buffer[..read]).map_err(|error| {
                ModelDownloadError::WriteTemporary {
                    file_name: file.file_name.to_string(),
                    reason: error.to_string(),
                }
            })?;
            hasher.update(&buffer[..read]);
        }

        if total < file.min_bytes {
            return Err(ModelDownloadError::TooSmall {
                file_name: file.file_name.to_string(),
                actual: total,
                minimum: file.min_bytes,
            });
        }
        let observed = format!("{:x}", hasher.finalize());
        if !observed.eq_ignore_ascii_case(file.sha256_hex) {
            return Err(ModelDownloadError::HashMismatch {
                file_name: file.file_name.to_string(),
            });
        }
        if let Ok(total_bytes) = usize::try_from(total) {
            record_resource_size("model_download", total_bytes);
        } else {
            tracing::debug!(
                resource = "model_download",
                bytes = total,
                "bounded resource size"
            );
        }
        output
            .sync_all()
            .map_err(|error| ModelDownloadError::SyncTemporary {
                file_name: file.file_name.to_string(),
                reason: error.to_string(),
            })?;
        drop(output);
        crate::atomic_file::replace_file_atomically(&temporary_path, target_path).map_err(
            |reason| ModelDownloadError::ReplaceTarget {
                file_name: file.file_name.to_string(),
                reason,
            },
        )?;
        Ok(())
    })();

    if let Err(error) = result {
        if temporary_path.exists() {
            if let Err(cleanup) = fs::remove_file(&temporary_path) {
                return Err(ModelDownloadError::CleanupFailed {
                    file_name: file.file_name.to_string(),
                    primary: error.to_string(),
                    reason: cleanup.to_string(),
                });
            }
        }
        return Err(error);
    }

    Ok(())
}
