use std::fs;
use std::path::Path;

use crate::config::{LocalAsrProfile, LocalTtsProfile};

const MIN_LOCAL_ASR_MODEL_BYTES: u64 = 1_000_000;
const MIN_TTS_CONFIG_BYTES: u64 = 2;
const MIN_TTS_VOICES_BYTES: u64 = 1_000;
const MIN_TTS_ONNX_BYTES: u64 = 1_000_000;

fn file_size_at_least(path: &Path, min_bytes: u64) -> bool {
    match path.metadata() {
        Ok(metadata) => metadata.is_file() && metadata.len() >= min_bytes,
        Err(_) => false,
    }
}

pub(crate) fn local_tts_model_is_available(profile: &LocalTtsProfile) -> bool {
    let model_path = Path::new(profile.model_path.trim());
    if !model_path.is_dir() {
        return false;
    }

    let has_config = file_size_at_least(&model_path.join("config.json"), MIN_TTS_CONFIG_BYTES);
    let has_voices = file_size_at_least(&model_path.join("voices.npz"), MIN_TTS_VOICES_BYTES);
    let entries = match fs::read_dir(model_path) {
        Ok(entries) => entries,
        Err(_) => return false,
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => return false,
        };
        let path = entry.path();
        if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("onnx"))
            && file_size_at_least(&path, MIN_TTS_ONNX_BYTES)
        {
            return has_config && has_voices;
        }
    }

    false
}

pub(crate) fn local_asr_model_is_available(profile: &LocalAsrProfile) -> bool {
    file_size_at_least(
        Path::new(profile.model_path.trim()),
        MIN_LOCAL_ASR_MODEL_BYTES,
    )
}
