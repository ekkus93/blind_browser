use std::fs;
use std::path::Path;

use crate::config::{LocalAsrProfile, LocalTtsProfile};

const MIN_LOCAL_ASR_MODEL_BYTES: u64 = 1_000_000;
const MIN_TTS_CONFIG_BYTES: u64 = 2;
const MIN_TTS_VOICES_BYTES: u64 = 1_000;
const MIN_TTS_ONNX_BYTES: u64 = 1_000_000;

fn file_size_at_least(path: &Path, min_bytes: u64) -> bool {
    path.metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.len() >= min_bytes)
}

pub(crate) fn local_tts_model_is_available(profile: &LocalTtsProfile) -> bool {
    let model_path = Path::new(profile.model_path.trim());
    if !model_path.is_dir() {
        return false;
    }

    let has_config = file_size_at_least(&model_path.join("config.json"), MIN_TTS_CONFIG_BYTES);
    let has_voices = file_size_at_least(&model_path.join("voices.npz"), MIN_TTS_VOICES_BYTES);
    let Ok(mut entries) = fs::read_dir(model_path) else {
        return false;
    };

    let mut has_onnx = false;
    if entries
        .try_for_each(|entry| -> Result<(), std::io::Error> {
            let path = entry?.path();
            if path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("onnx"))
                && file_size_at_least(&path, MIN_TTS_ONNX_BYTES)
            {
                has_onnx = true;
            }
            Ok(())
        })
        .is_err()
    {
        return false;
    }

    has_config && has_voices && has_onnx
}

pub(crate) fn local_asr_model_is_available(profile: &LocalAsrProfile) -> bool {
    file_size_at_least(
        Path::new(profile.model_path.trim()),
        MIN_LOCAL_ASR_MODEL_BYTES,
    )
}
