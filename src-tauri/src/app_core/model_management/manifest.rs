use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KittenDownloadPlan {
    pub(crate) repository: &'static str,
    pub(crate) directory_name: &'static str,
    pub(crate) display_name: &'static str,
    pub(crate) files: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WhisperDownloadPlan {
    pub(crate) repository: &'static str,
    pub(crate) display_name: &'static str,
    pub(crate) file_name: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct VerifiedModelFile {
    pub(super) file_name: &'static str,
    pub(super) sha256_hex: &'static str,
    pub(super) min_bytes: u64,
    pub(super) max_bytes: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct VerifiedRepositoryPlan {
    pub(super) repository: &'static str,
    pub(super) revision: &'static str,
    pub(super) files: &'static [VerifiedModelFile],
}

#[derive(Debug, Error)]
pub(super) enum ModelDownloadError {
    #[error("invalid integrity manifest for '{file_name}': {reason}")]
    InvalidManifest { file_name: String, reason: String },
    #[error("failed to create the verified model download client: {reason}")]
    ClientBuild { reason: String },
    #[error("model download for '{file_name}' timed out")]
    RequestTimedOut { file_name: String },
    #[error("model download request for '{file_name}' failed: {reason}")]
    RequestFailed { file_name: String, reason: String },
    #[error("model download for '{file_name}' attempted an unapproved redirect{destination}")]
    RedirectRefused {
        file_name: String,
        destination: String,
    },
    #[error("model download for '{file_name}' returned HTTP {status}")]
    HttpStatus { file_name: String, status: u16 },
    #[error("model file '{file_name}' was too small: received {actual} bytes, minimum {minimum}")]
    TooSmall {
        file_name: String,
        actual: u64,
        minimum: u64,
    },
    #[error("model file '{file_name}' exceeded its maximum size of {maximum} bytes")]
    TooLarge { file_name: String, maximum: u64 },
    #[error("model file '{file_name}' failed SHA-256 verification")]
    HashMismatch { file_name: String },
    #[error("failed to create the model directory for '{file_name}': {reason}")]
    CreateDirectory { file_name: String, reason: String },
    #[error("failed to create the temporary model file for '{file_name}': {reason}")]
    CreateTemporary { file_name: String, reason: String },
    #[error("failed to read model bytes for '{file_name}': {reason}")]
    ReadResponse { file_name: String, reason: String },
    #[error("failed to write model bytes for '{file_name}': {reason}")]
    WriteTemporary { file_name: String, reason: String },
    #[error("failed to sync the temporary model file for '{file_name}': {reason}")]
    SyncTemporary { file_name: String, reason: String },
    #[error("failed to atomically replace model file '{file_name}': {reason}")]
    ReplaceTarget { file_name: String, reason: String },
    #[error("failed to clean up the partial model file for '{file_name}' after '{primary}': {reason}")]
    CleanupFailed {
        file_name: String,
        primary: String,
        reason: String,
    },
}

const KITTEN_MINI_FILE_NAMES: &[&str] = &[
    "config.json",
    "kitten_tts_mini_v0_8.onnx",
    "voices.npz",
];
const KITTEN_MICRO_FILE_NAMES: &[&str] = &[
    "config.json",
    "kitten_tts_micro_v0_8.onnx",
    "voices.npz",
];
const KITTEN_NANO_FILE_NAMES: &[&str] = &[
    "config.json",
    "kitten_tts_nano_v0_8.onnx",
    "voices.npz",
];

const KITTEN_MINI_FILES: &[VerifiedModelFile] = &[
    file(
        "config.json",
        "6b160bc9b19e24ecb21e84bc14f8a7da21fdf47ec72d42450bc5cf514b61804a",
        470,
    ),
    file(
        "kitten_tts_mini_v0_8.onnx",
        "0f5bbae4fc4800c98dbc544a87ecfa79510de2fb8222db30d12e5bfe9177df91",
        78_268_016,
    ),
    file(
        "voices.npz",
        "40ad2638952b77b7b2f30127e2608e169fc69dd256b53bd8aaa3409a33193c42",
        3_278_902,
    ),
];
const KITTEN_MICRO_FILES: &[VerifiedModelFile] = &[
    file(
        "config.json",
        "1f0bd2208348f9211cb0da64fcd1536eb28228571cc6b09e767eb6e203a0a532",
        473,
    ),
    file(
        "kitten_tts_micro_v0_8.onnx",
        "95481626fee1ba70ce683e69c534fc7cb38433c46ce42d3abbeafb4b9f1a4123",
        41_384_970,
    ),
    file(
        "voices.npz",
        "112710c1be8ad0e967c190fb0fd95cbe5848ec4791b93209f20b28b7da20dac1",
        3_278_902,
    ),
];
const KITTEN_NANO_FP32_FILES: &[VerifiedModelFile] = &[
    file(
        "config.json",
        "b66006ccbeccd4de5fc3c9272059c47f5725df7215fd889785c03602652fab64",
        688,
    ),
    file(
        "kitten_tts_nano_v0_8.onnx",
        "320564d2615f235de972ca27a7f39551c94185cfa24ca85b07a29084135f1e5e",
        56_767_095,
    ),
    file(
        "voices.npz",
        "8aa7cee235abb0739cb51e6559685f65a4dacd95568833d05699b1633f519b3f",
        3_278_902,
    ),
];
const KITTEN_NANO_INT8_FILES: &[VerifiedModelFile] = &[
    file(
        "config.json",
        "b66006ccbeccd4de5fc3c9272059c47f5725df7215fd889785c03602652fab64",
        688,
    ),
    file(
        "kitten_tts_nano_v0_8.onnx",
        "f7b0afcbee92870b32b8e0276d855b954dc25470c9f051b376ac7eee537c76fc",
        24_369_971,
    ),
    file(
        "voices.npz",
        "8aa7cee235abb0739cb51e6559685f65a4dacd95568833d05699b1633f519b3f",
        3_278_902,
    ),
];

const WHISPER_FILES: &[VerifiedModelFile] = &[
    file(
        "ggml-tiny.bin",
        "be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21",
        77_691_713,
    ),
    file(
        "ggml-base.bin",
        "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
        147_951_465,
    ),
    file(
        "ggml-small.bin",
        "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b",
        487_601_967,
    ),
    file(
        "ggml-medium.bin",
        "6c14d5adee5f86394037b4e4e8b59f1673b6cee10e3cf0b11bbdbee79c156208",
        1_533_763_059,
    ),
    file(
        "ggml-large-v3.bin",
        "64d182b440b98d5203c4f9bd541544d84c605196c4f7b845dfa11fb23594d1e2",
        3_095_033_483,
    ),
    file(
        "ggml-large-v3-turbo.bin",
        "1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69",
        1_624_555_275,
    ),
];

const fn file(file_name: &'static str, sha256_hex: &'static str, bytes: u64) -> VerifiedModelFile {
    VerifiedModelFile {
        file_name,
        sha256_hex,
        min_bytes: bytes,
        max_bytes: Some(bytes),
    }
}

pub(crate) fn kitten_download_plan_for_model_id(
    model_id: &str,
) -> Result<KittenDownloadPlan, String> {
    let plan = match model_id.trim().to_ascii_lowercase().as_str() {
        "default" | "mini" | "kitten-tts-mini" => KittenDownloadPlan {
            repository: "KittenML/kitten-tts-mini-0.8",
            directory_name: "kitten-tts-mini",
            display_name: "KittenTTS mini model",
            files: KITTEN_MINI_FILE_NAMES,
        },
        "micro" | "kitten-tts-micro" => KittenDownloadPlan {
            repository: "KittenML/kitten-tts-micro-0.8",
            directory_name: "kitten-tts-micro",
            display_name: "KittenTTS micro model",
            files: KITTEN_MICRO_FILE_NAMES,
        },
        "nano" | "kitten-tts-nano" => KittenDownloadPlan {
            repository: "KittenML/kitten-tts-nano-0.8-fp32",
            directory_name: "kitten-tts-nano",
            display_name: "KittenTTS nano model",
            files: KITTEN_NANO_FILE_NAMES,
        },
        "nano-int8" | "kitten-tts-nano-int8" => KittenDownloadPlan {
            repository: "KittenML/kitten-tts-nano-0.8-int8",
            directory_name: "kitten-tts-nano-int8",
            display_name: "KittenTTS nano int8 model",
            files: KITTEN_NANO_FILE_NAMES,
        },
        _ => {
            return Err(format!(
                "local TTS model_id '{}' does not have a pinned integrity manifest",
                model_id.trim()
            ));
        }
    };
    verified_repository_plan(plan.repository).map_err(|error| error.to_string())?;
    Ok(plan)
}

pub(crate) fn whisper_download_plan_for_model_id(
    model_id: &str,
) -> Result<WhisperDownloadPlan, String> {
    let (file_name, display_name) = match model_id.trim().to_ascii_lowercase().as_str() {
        "tiny" => ("ggml-tiny.bin", "tiny model"),
        "base" => ("ggml-base.bin", "base model"),
        "small" => ("ggml-small.bin", "small model"),
        "medium" => ("ggml-medium.bin", "medium model"),
        "large-v3" => ("ggml-large-v3.bin", "large-v3 model"),
        "large-v3-turbo" => ("ggml-large-v3-turbo.bin", "large-v3-turbo model"),
        _ => {
            return Err(format!(
                "local ASR model_id '{}' does not have a pinned integrity manifest",
                model_id.trim()
            ));
        }
    };
    let plan = WhisperDownloadPlan {
        repository: "ggerganov/whisper.cpp",
        display_name,
        file_name,
    };
    verified_file_for(plan.repository, plan.file_name).map_err(|error| error.to_string())?;
    Ok(plan)
}

pub(super) fn verified_repository_plan(
    repository: &str,
) -> Result<VerifiedRepositoryPlan, ModelDownloadError> {
    let plan = match repository {
        "KittenML/kitten-tts-mini-0.8" => VerifiedRepositoryPlan {
            repository: "KittenML/kitten-tts-mini-0.8",
            revision: "c02725660cea441db4c383af69f1f26f5cd00947",
            files: KITTEN_MINI_FILES,
        },
        "KittenML/kitten-tts-micro-0.8" => VerifiedRepositoryPlan {
            repository: "KittenML/kitten-tts-micro-0.8",
            revision: "1ccf72b2c2048fd17efac7de2fab32d10e225084",
            files: KITTEN_MICRO_FILES,
        },
        "KittenML/kitten-tts-nano-0.8-fp32" => VerifiedRepositoryPlan {
            repository: "KittenML/kitten-tts-nano-0.8-fp32",
            revision: "7a1db645b1f3ab9420761d87428e042b9cec3f26",
            files: KITTEN_NANO_FP32_FILES,
        },
        "KittenML/kitten-tts-nano-0.8-int8" => VerifiedRepositoryPlan {
            repository: "KittenML/kitten-tts-nano-0.8-int8",
            revision: "84781d74e29ee25217551556398b42f80593a813",
            files: KITTEN_NANO_INT8_FILES,
        },
        "ggerganov/whisper.cpp" => VerifiedRepositoryPlan {
            repository: "ggerganov/whisper.cpp",
            revision: "5359861c739e955e79d9a303bcbc70fb988958b1",
            files: WHISPER_FILES,
        },
        _ => {
            return Err(ModelDownloadError::InvalidManifest {
                file_name: repository.to_string(),
                reason: String::from("repository has no pinned integrity manifest"),
            });
        }
    };
    validate_plan(plan.repository, plan.revision, plan.files)?;
    Ok(plan)
}

pub(super) fn verified_file_for(
    repository: &str,
    file_name: &str,
) -> Result<(&'static str, &'static VerifiedModelFile), ModelDownloadError> {
    let plan = verified_repository_plan(repository)?;
    let file = plan
        .files
        .iter()
        .find(|file| file.file_name == file_name)
        .ok_or_else(|| ModelDownloadError::InvalidManifest {
            file_name: file_name.to_string(),
            reason: format!("file is not in the pinned manifest for {repository}"),
        })?;
    Ok((plan.revision, file))
}

fn validate_plan(
    repository: &str,
    revision: &str,
    files: &[VerifiedModelFile],
) -> Result<(), ModelDownloadError> {
    if repository.split('/').count() != 2
        || repository.contains("..")
        || repository.chars().any(char::is_control)
    {
        return Err(ModelDownloadError::InvalidManifest {
            file_name: repository.to_string(),
            reason: String::from("repository must be an owner/name identifier"),
        });
    }
    if revision.len() != 40 || !revision.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(ModelDownloadError::InvalidManifest {
            file_name: repository.to_string(),
            reason: String::from("revision must be a pinned 40-character commit SHA"),
        });
    }
    if files.is_empty() {
        return Err(ModelDownloadError::InvalidManifest {
            file_name: repository.to_string(),
            reason: String::from("download plan must include at least one file"),
        });
    }
    for file in files {
        validate_file_manifest(file)?;
    }
    Ok(())
}

pub(super) fn validate_file_manifest(
    file: &VerifiedModelFile,
) -> Result<(), ModelDownloadError> {
    let invalid_name = file.file_name.is_empty()
        || file.file_name.contains('/')
        || file.file_name.contains('\\')
        || file.file_name.contains("..")
        || file.file_name.chars().any(char::is_control);
    if invalid_name {
        return Err(ModelDownloadError::InvalidManifest {
            file_name: file.file_name.to_string(),
            reason: String::from("file name must be a single safe path component"),
        });
    }
    if file.sha256_hex.len() != 64
        || !file
            .sha256_hex
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(ModelDownloadError::InvalidManifest {
            file_name: file.file_name.to_string(),
            reason: String::from("SHA-256 must contain exactly 64 hexadecimal characters"),
        });
    }
    if file.min_bytes == 0 {
        return Err(ModelDownloadError::InvalidManifest {
            file_name: file.file_name.to_string(),
            reason: String::from("minimum byte size must be greater than zero"),
        });
    }
    if file.max_bytes.is_some_and(|maximum| maximum < file.min_bytes) {
        return Err(ModelDownloadError::InvalidManifest {
            file_name: file.file_name.to_string(),
            reason: String::from("maximum byte size must not be smaller than minimum"),
        });
    }
    Ok(())
}
