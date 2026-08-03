use std::fs;
use std::io::Cursor;
use std::time::Duration;

use sha2::{Digest, Sha256};
use url::Url;

use super::availability::{local_asr_model_is_available, local_tts_model_is_available};
use super::download::{
    model_download_client, model_redirect_url_allowed, write_verified_reader_atomically,
    MODEL_DOWNLOAD_CONNECT_TIMEOUT, MODEL_DOWNLOAD_REQUEST_TIMEOUT,
};
use super::manifest::{
    kitten_download_plan_for_model_id, validate_file_manifest, whisper_download_plan_for_model_id,
    ModelDownloadError, VerifiedModelFile,
};
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

fn fixture_manifest(bytes: &[u8]) -> VerifiedModelFile {
    let digest = format!("{:x}", Sha256::digest(bytes));
    VerifiedModelFile {
        file_name: "fixture.bin",
        sha256_hex: Box::leak(digest.into_boxed_str()),
        min_bytes: bytes.len() as u64,
        max_bytes: Some(bytes.len() as u64),
    }
}

#[test]
fn known_download_plans_have_valid_pinned_manifests() {
    for model in ["mini", "micro", "nano", "nano-int8"] {
        kitten_download_plan_for_model_id(model).unwrap();
    }
    for model in [
        "tiny",
        "base",
        "small",
        "medium",
        "large-v3",
        "large-v3-turbo",
    ] {
        whisper_download_plan_for_model_id(model).unwrap();
    }
}

#[test]
fn unknown_models_fail_closed_without_integrity_metadata() {
    assert!(kitten_download_plan_for_model_id("future-model").is_err());
    assert!(whisper_download_plan_for_model_id("future-model").is_err());
}

#[test]
fn invalid_manifest_entries_are_rejected() {
    let invalid = VerifiedModelFile {
        file_name: "../escape.bin",
        sha256_hex: "bad",
        min_bytes: 0,
        max_bytes: Some(0),
    };
    assert!(matches!(
        validate_file_manifest(&invalid),
        Err(ModelDownloadError::InvalidManifest { .. })
    ));
}

#[test]
fn verified_write_succeeds_and_removes_part_file() {
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("fixture.bin");
    let bytes = b"verified model bytes";
    write_verified_reader_atomically(Cursor::new(bytes), &target, &fixture_manifest(bytes))
        .unwrap();
    assert_eq!(fs::read(&target).unwrap(), bytes);
    assert!(!directory.path().join("fixture.bin.part").exists());
}

#[test]
fn hash_mismatch_preserves_old_target_and_cleans_part() {
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("fixture.bin");
    fs::write(&target, b"known good old model").unwrap();
    let mut manifest = fixture_manifest(b"expected bytes");
    manifest.min_bytes = 1;
    manifest.max_bytes = Some(64);
    let error = write_verified_reader_atomically(Cursor::new(b"wrong bytes"), &target, &manifest)
        .unwrap_err();
    assert!(matches!(error, ModelDownloadError::HashMismatch { .. }));
    assert_eq!(fs::read(&target).unwrap(), b"known good old model");
    assert!(!directory.path().join("fixture.bin.part").exists());
}

#[test]
fn size_bounds_fail_before_replacement() {
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("fixture.bin");
    fs::write(&target, b"old").unwrap();

    let mut too_small = fixture_manifest(b"12345");
    too_small.min_bytes = 6;
    too_small.max_bytes = Some(6);
    assert!(matches!(
        write_verified_reader_atomically(Cursor::new(b"12345"), &target, &too_small),
        Err(ModelDownloadError::TooSmall { .. })
    ));

    let mut too_large = fixture_manifest(b"12345");
    too_large.min_bytes = 1;
    too_large.max_bytes = Some(4);
    assert!(matches!(
        write_verified_reader_atomically(Cursor::new(b"12345"), &target, &too_large),
        Err(ModelDownloadError::TooLarge { .. })
    ));

    assert_eq!(fs::read(&target).unwrap(), b"old");
    assert!(!directory.path().join("fixture.bin.part").exists());
}

#[test]
fn redirect_policy_accepts_only_https_hugging_face_delivery_hosts() {
    for url in [
        "https://huggingface.co/repo/file",
        "https://cdn-lfs.huggingface.co/file",
        "https://cdn-lfs-us-1.hf.co/file",
        "https://cas-bridge.xethub.hf.co/file",
    ] {
        assert!(
            model_redirect_url_allowed(&Url::parse(url).unwrap()),
            "{url}"
        );
    }
    for url in [
        "http://huggingface.co/repo/file",
        "https://evil.example/file",
        "https://user:pass@huggingface.co/file",
    ] {
        assert!(
            !model_redirect_url_allowed(&Url::parse(url).unwrap()),
            "{url}"
        );
    }
}

#[test]
fn model_download_timeout_is_explicit_and_bounded() {
    assert_eq!(MODEL_DOWNLOAD_CONNECT_TIMEOUT, Duration::from_secs(15));
    assert_eq!(MODEL_DOWNLOAD_REQUEST_TIMEOUT, Duration::from_secs(1_800));
    model_download_client().unwrap();
}

#[test]
fn local_asr_model_availability_rejects_empty_small_and_missing_files() {
    let directory = tempfile::tempdir().unwrap();
    let model_path = directory.path().join("tiny.gguf");
    fs::write(&model_path, []).unwrap();
    assert!(!local_asr_model_is_available(&asr_profile_at(
        model_path.to_str().unwrap()
    )));
    fs::write(&model_path, vec![0_u8; 100]).unwrap();
    assert!(!local_asr_model_is_available(&asr_profile_at(
        model_path.to_str().unwrap()
    )));
    assert!(!local_asr_model_is_available(&asr_profile_at(
        directory.path().join("missing.gguf").to_str().unwrap()
    )));
}

#[test]
fn local_tts_model_availability_rejects_incomplete_model() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(directory.path().join("config.json"), b"{}").unwrap();
    fs::write(directory.path().join("voices.npz"), vec![0_u8; 2_000]).unwrap();
    fs::write(directory.path().join("model.onnx"), []).unwrap();
    assert!(!local_tts_model_is_available(&tts_profile_at(
        directory.path().to_str().unwrap()
    )));
}
