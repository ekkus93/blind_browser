use super::*;

#[test]
fn config_enums_round_trip_and_reject_invalid_variants() {
    fn assert_enum_round_trip<T>(
        value: T,
        expected: serde_json::Value,
        invalid: serde_json::Value,
    ) where
        T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        let serialized = serde_json::to_value(&value).expect("enum should serialize");
        assert_eq!(serialized, expected);

        let round_tripped: T =
            serde_json::from_value(serialized).expect("enum should deserialize");
        assert_eq!(round_tripped, value);
        assert!(serde_json::from_value::<T>(invalid).is_err());
    }

    assert_enum_round_trip(
        ProviderMode::Remote,
        serde_json::json!("remote"),
        serde_json::json!("REMOTE"),
    );
    assert_enum_round_trip(
        RemoteProviderKind::OpenAi,
        serde_json::json!("OpenAi"),
        serde_json::json!("Anthropic"),
    );
    assert_enum_round_trip(
        RemoteTtsAudioFormat::Wav,
        serde_json::json!("wav"),
        serde_json::json!("mp3"),
    );
    assert_enum_round_trip(
        LocalTtsBackend::KittenTtsRs,
        serde_json::json!("kitten_tts_rs"),
        serde_json::json!("kitten"),
    );
    assert_enum_round_trip(
        LocalAsrBackend::Whisper,
        serde_json::json!("whisper"),
        serde_json::json!("deepgram"),
    );
    assert_enum_round_trip(
        SpeechFeedbackStyle::Detailed,
        serde_json::json!("Detailed"),
        serde_json::json!("Verbose"),
    );
}

#[test]
fn provider_configs_round_trip_through_json() {
    let providers = ProviderSelections {
        planner: ProviderSelection {
            mode: ProviderMode::Remote,
            remote_profile: Some(String::from("openai-default")),
            local_profile: None,
            failover_to_local: None,
        },
        tts: ProviderSelection {
            mode: ProviderMode::Local,
            remote_profile: Some(String::from("openai-tts-default")),
            local_profile: Some(String::from("kitten-default")),
            failover_to_local: Some(false),
        },
        asr: ProviderSelection {
            mode: ProviderMode::Remote,
            remote_profile: Some(String::from("openai-asr-default")),
            local_profile: Some(String::from("whisper-default")),
            failover_to_local: Some(true),
        },
    };
    let planner_profile = RemotePlannerProfile {
        provider: RemoteProviderKind::OpenAi,
        base_url: String::from("https://api.openai.com/v1"),
        model: String::from("gpt-4.1"),
        api_key: SecretRef::FromEnv {
            from_env: String::from("OPENAI_API_KEY"),
        },
        organization: Some(SecretRef::FromEnv {
            from_env: String::from("OPENAI_ORG_ID"),
        }),
        project: Some(String::from("blind-browser")),
        temperature_milli: 250,
        max_output_tokens: 1024,
        timeout_ms: 30_000,
    };
    let remote_tts_profile = RemoteTtsProfile {
        provider: RemoteProviderKind::OpenAi,
        base_url: String::from("https://api.openai.com/v1"),
        model: String::from("gpt-4o-mini-tts"),
        api_key: SecretRef::FromKeyring {
            from_keyring: KeyringRef {
                service: String::from("blind-browser"),
                account: String::from("tts/openai-tts-default"),
            },
        },
        organization: None,
        project: Some(String::from("blind-browser")),
        voice: String::from("alloy"),
        audio_format: RemoteTtsAudioFormat::Wav,
        timeout_ms: 20_000,
    };
    let remote_asr_profile = RemoteAsrProfile {
        provider: RemoteProviderKind::OpenAi,
        base_url: String::from("https://api.openai.com/v1"),
        model: String::from("gpt-4o-mini-transcribe"),
        api_key: SecretRef::FromFile {
            from_file: String::from("/tmp/openai-asr.key"),
        },
        organization: None,
        project: Some(String::from("blind-browser")),
        language: Some(String::from("en")),
        temperature_milli: 0,
        timeout_ms: 20_000,
    };
    let local_tts_profile = LocalTtsProfile {
        backend: LocalTtsBackend::KittenTtsRs,
        model_id: String::from("default"),
        model_path: String::from("/models/kitten/default.onnx"),
        default_voice: String::from("Bruno"),
        sample_rate: 24_000,
    };
    let local_asr_profile = LocalAsrProfile {
        backend: LocalAsrBackend::Whisper,
        model_id: String::from("tiny"),
        model_path: String::from("/models/whisper/tiny.bin"),
        language: Some(String::from("en")),
        threads: 4,
    };

    let serialized = serde_json::json!({
        "providers": providers,
        "planner_profile": planner_profile,
        "remote_tts_profile": remote_tts_profile,
        "remote_asr_profile": remote_asr_profile,
        "local_tts_profile": local_tts_profile,
        "local_asr_profile": local_asr_profile
    });

    let round_tripped = serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(
        serialized.clone(),
    )
    .expect("provider config payload should deserialize as JSON object");

    let decoded_providers: ProviderSelections =
        serde_json::from_value(serialized.get("providers").cloned().unwrap())
            .expect("provider selections should deserialize");
    let decoded_planner_profile: RemotePlannerProfile =
        serde_json::from_value(serialized.get("planner_profile").cloned().unwrap())
            .expect("planner profile should deserialize");
    let decoded_remote_tts_profile: RemoteTtsProfile =
        serde_json::from_value(serialized.get("remote_tts_profile").cloned().unwrap())
            .expect("remote tts profile should deserialize");
    let decoded_remote_asr_profile: RemoteAsrProfile =
        serde_json::from_value(serialized.get("remote_asr_profile").cloned().unwrap())
            .expect("remote asr profile should deserialize");
    let decoded_local_tts_profile: LocalTtsProfile =
        serde_json::from_value(serialized.get("local_tts_profile").cloned().unwrap())
            .expect("local tts profile should deserialize");
    let decoded_local_asr_profile: LocalAsrProfile =
        serde_json::from_value(serialized.get("local_asr_profile").cloned().unwrap())
            .expect("local asr profile should deserialize");

    assert_eq!(decoded_providers, providers);
    assert_eq!(decoded_planner_profile, planner_profile);
    assert_eq!(decoded_remote_tts_profile, remote_tts_profile);
    assert_eq!(decoded_remote_asr_profile, remote_asr_profile);
    assert_eq!(decoded_local_tts_profile, local_tts_profile);
    assert_eq!(decoded_local_asr_profile, local_asr_profile);
    assert!(round_tripped.contains_key("providers"));
}


#[test]
fn parses_default_template() {
    let config = AppConfig::load_from_str(AppConfig::default_template())
        .expect("default template should parse and validate");

    assert_eq!(
        config.providers.planner.remote_profile.as_deref(),
        Some("openai-default")
    );
    assert_eq!(config.providers.tts.mode, ProviderMode::Remote);
    assert_eq!(config.providers.asr.mode, ProviderMode::Remote);
    assert!(config
        .remote_planner_profiles
        .contains_key("openai-default"));
    assert!(config.local_tts_profiles.contains_key("kitten-default"));
    assert!(config.local_asr_profiles.contains_key("whisper-default"));
    assert!(config.ocr.trigger_on_no_extractable_text);
    assert_eq!(config.ocr.sparse_text_char_threshold, 200);
    assert_eq!(config.ocr.sparse_text_region_threshold, 2);
    assert!(config.ocr.prefer_region_ocr);
}

#[test]
fn parses_ollama_planner_profile_when_selected() {
    let config = AppConfig::load_from_str(
        r#"
[providers.planner]
mode = "remote"
remote_profile = "ollama-default"

[providers.tts]
mode = "local"
local_profile = "kitten-default"

[providers.asr]
mode = "local"
local_profile = "whisper-default"

[audio]
playback_volume = 1.0
playback_speed = 1.0
default_tts_voice = "Bruno"

[safety]
confirmation_confidence_threshold = 0.9
allow_click_without_confirmation = true
always_confirm_submit = true

[ocr]
trigger_on_no_extractable_text = true
sparse_text_char_threshold = 200
sparse_text_region_threshold = 2
prefer_region_ocr = true

[models]
models_dir = "~/.config/blind_browser/models"
check_on_startup = true
auto_download_missing = false

[speech_feedback]
style = "Short"
confirm_setting_changes = true
include_previous_value = false

[remote_profiles.ollama-default]
provider = "Ollama"
base_url = "http://localhost:11434/v1"
model = "qwen2.5:3b-instruct"
api_key = { from_env = "OLLAMA_API_KEY" }
temperature_milli = 200
max_output_tokens = 1024
timeout_ms = 30000

[local_profiles.kitten-default]
backend = "kitten_tts_rs"
model_id = "default"
model_path = "/path/to/kitten/model"
default_voice = "Bruno"
sample_rate = 24000

[local_profiles.whisper-default]
backend = "whisper"
model_id = "tiny"
model_path = "/path/to/whisper/model"
language = "en"
threads = 4
"#,
    )
    .expect("Ollama planner config should parse and validate");

    let profile = config
        .remote_planner_profiles
        .get("ollama-default")
        .expect("selected Ollama profile should be loaded");
    assert_eq!(profile.provider, RemoteProviderKind::Ollama);
    assert_eq!(profile.base_url, "http://localhost:11434/v1");
    assert_eq!(profile.model, "qwen2.5:3b-instruct");
    assert_eq!(
        profile.api_key,
        SecretRef::FromEnv {
            from_env: String::from("OLLAMA_API_KEY"),
        }
    );
}

#[test]
fn rejects_missing_selected_remote_planner_profile_reference() {
    let invalid = r#"
[providers.planner]
mode = "remote"
remote_profile = "missing-planner-profile"

[providers.tts]
mode = "local"
local_profile = "kitten-default"

[providers.asr]
mode = "local"
local_profile = "whisper-default"

[audio]
playback_volume = 1.0
playback_speed = 1.0
default_tts_voice = "Bruno"

[safety]
confirmation_confidence_threshold = 0.9
allow_click_without_confirmation = true
always_confirm_submit = true

[ocr]
trigger_on_no_extractable_text = true
sparse_text_char_threshold = 200
sparse_text_region_threshold = 2
prefer_region_ocr = true

[models]
models_dir = "~/.config/blind_browser/models"
check_on_startup = true
auto_download_missing = false

[speech_feedback]
style = "Short"
confirm_setting_changes = true
include_previous_value = false

[local_profiles.kitten-default]
backend = "kitten_tts_rs"
model_id = "default"
model_path = "/path/to/kitten/model"
default_voice = "Bruno"
sample_rate = 24000

[local_profiles.whisper-default]
backend = "whisper"
model_id = "tiny"
model_path = "/path/to/whisper/model"
language = "en"
threads = 4
"#;

    let error = AppConfig::load_from_str(invalid).expect_err("config should be invalid");

    match error {
        ConfigError::Validation(message) => {
            assert!(message.contains(
                "providers.planner references missing remote_profiles.missing-planner-profile"
            ));
        }
        other => panic!("expected validation error, got {other}"),
    }
}

#[test]
fn rejects_inline_secret_refs() {
    let invalid = r#"
[providers.planner]
mode = "remote"
remote_profile = "ollama-default"

[providers.tts]
mode = "local"
local_profile = "kitten-default"

[providers.asr]
mode = "local"
local_profile = "whisper-default"

[audio]
playback_volume = 1.0
playback_speed = 1.0
default_tts_voice = "Bruno"

[safety]
confirmation_confidence_threshold = 0.9
allow_click_without_confirmation = true
always_confirm_submit = true

[ocr]
trigger_on_no_extractable_text = true
sparse_text_char_threshold = 200
sparse_text_region_threshold = 2
prefer_region_ocr = true

[models]
models_dir = "~/.config/blind_browser/models"
check_on_startup = true
auto_download_missing = false

[speech_feedback]
style = "Short"
confirm_setting_changes = true
include_previous_value = false

[remote_profiles.ollama-default]
provider = "Ollama"
base_url = "http://localhost:11434/v1"
model = "qwen2.5:3b-instruct"
api_key = { inline = "ollama" }
temperature_milli = 200
max_output_tokens = 1024
timeout_ms = 30000

[local_profiles.kitten-default]
backend = "kitten_tts_rs"
model_id = "default"
model_path = "/path/to/kitten/model"
default_voice = "Bruno"
sample_rate = 24000

[local_profiles.whisper-default]
backend = "whisper"
model_id = "tiny"
model_path = "/path/to/whisper/model"
language = "en"
threads = 4
"#;

    let error = AppConfig::load_from_str(invalid).expect_err("config should be invalid");
    assert!(
        matches!(error, ConfigError::Validation(ref message) if message.contains("data did not match any variant of untagged enum SecretRef")),
        "expected inline secret refs to fail validation, got {error}"
    );
}

#[test]
fn rejects_local_planner_configuration() {
    let invalid = r#"
[providers.planner]
mode = "local"
local_profile = "ollama-default"
failover_to_local = true

[providers.tts]
mode = "local"
local_profile = "kitten-default"

[providers.asr]
mode = "local"
local_profile = "whisper-default"

[audio]
playback_volume = 1.0
playback_speed = 1.0
default_tts_voice = "Bruno"

[safety]
confirmation_confidence_threshold = 0.9
allow_click_without_confirmation = true
always_confirm_submit = true

[ocr]
trigger_on_no_extractable_text = true
sparse_text_char_threshold = 200
sparse_text_region_threshold = 2
prefer_region_ocr = true

[models]
models_dir = "~/.config/blind_browser/models"
check_on_startup = true
auto_download_missing = false

[speech_feedback]
style = "Short"
confirm_setting_changes = true
include_previous_value = false

[remote_profiles.ollama-default]
provider = "Ollama"
base_url = "http://localhost:11434/v1"
model = "qwen2.5:3b-instruct"
api_key = { from_env = "OLLAMA_API_KEY" }
temperature_milli = 200
max_output_tokens = 1024
timeout_ms = 30000

[local_profiles.kitten-default]
backend = "kitten_tts_rs"
model_id = "default"
model_path = "/path/to/kitten/model"
default_voice = "Bruno"
sample_rate = 24000

[local_profiles.whisper-default]
backend = "whisper"
model_id = "tiny"
model_path = "/path/to/whisper/model"
language = "en"
threads = 4
"#;

    let error = AppConfig::load_from_str(invalid).expect_err("config should be invalid");

    match error {
        ConfigError::Validation(message) => {
            assert!(message.contains("providers.planner.mode must be \"remote\""));
            assert!(message.contains("providers.planner.local_profile is not supported"));
            assert!(message.contains("providers.planner.failover_to_local is not supported"));
        }
        other => panic!("expected validation error, got {other}"),
    }
}

#[test]
fn rejects_missing_remote_profile_for_remote_mode() {
    let invalid = r#"
[providers.planner]
mode = "remote"

[providers.tts]
mode = "local"
local_profile = "kitten-default"

[providers.asr]
mode = "local"
local_profile = "whisper-default"

[audio]
playback_volume = 1.0
playback_speed = 1.0
default_tts_voice = "Bruno"

[safety]
confirmation_confidence_threshold = 0.9
allow_click_without_confirmation = true
always_confirm_submit = true

[ocr]
trigger_on_no_extractable_text = true
sparse_text_char_threshold = 200
sparse_text_region_threshold = 2
prefer_region_ocr = true

[models]
models_dir = "~/.config/blind_browser/models"
check_on_startup = true
auto_download_missing = false

[speech_feedback]
style = "Short"
confirm_setting_changes = true
include_previous_value = false

[local_profiles.kitten-default]
backend = "kitten_tts_rs"
model_id = "default"
model_path = "/path/to/kitten/model"
default_voice = "Bruno"
sample_rate = 24000

[local_profiles.whisper-default]
backend = "whisper"
model_id = "tiny"
model_path = "/path/to/whisper/model"
language = "en"
threads = 4
"#;

    let error = AppConfig::load_from_str(invalid).expect_err("config should be invalid");

    match error {
        ConfigError::Validation(message) => {
            assert!(message.contains("providers.planner.remote_profile is required"));
        }
        other => panic!("expected validation error, got {other}"),
    }
}

#[test]
fn rejects_missing_selected_profiles_for_tts_and_asr_modes() {
    let invalid = r#"
[providers.planner]
mode = "remote"
remote_profile = "openai-default"

[providers.tts]
mode = "remote"

[providers.asr]
mode = "local"

[audio]
playback_volume = 1.0
playback_speed = 1.0
default_tts_voice = "Bruno"

[safety]
confirmation_confidence_threshold = 0.9
allow_click_without_confirmation = true
always_confirm_submit = true

[ocr]
trigger_on_no_extractable_text = true
sparse_text_char_threshold = 200
sparse_text_region_threshold = 2
prefer_region_ocr = true

[models]
models_dir = "~/.config/blind_browser/models"
check_on_startup = true
auto_download_missing = false

[speech_feedback]
style = "Short"
confirm_setting_changes = true
include_previous_value = false

[remote_profiles.openai-default]
provider = "OpenAi"
base_url = "https://api.openai.com/v1"
model = "gpt-4.1"
api_key = { from_env = "OPENAI_API_KEY" }
temperature_milli = 200
max_output_tokens = 1024
timeout_ms = 30000

[local_profiles.kitten-default]
backend = "kitten_tts_rs"
model_id = "default"
model_path = "/path/to/kitten/model"
default_voice = "Bruno"
sample_rate = 24000

[local_profiles.whisper-default]
backend = "whisper"
model_id = "tiny"
model_path = "/path/to/whisper/model"
language = "en"
threads = 4
"#;

    let error = AppConfig::load_from_str(invalid).expect_err("config should be invalid");

    match error {
        ConfigError::Validation(message) => {
            assert!(message.contains("providers.tts.remote_profile is required"));
            assert!(message.contains("providers.asr.local_profile is required"));
        }
        other => panic!("expected validation error, got {other}"),
    }
}

#[test]
fn rejects_missing_selected_local_profile_references_for_tts_and_asr() {
    let invalid = r#"
[providers.planner]
mode = "remote"
remote_profile = "openai-default"

[providers.tts]
mode = "local"
local_profile = "missing-kitten-profile"

[providers.asr]
mode = "local"
local_profile = "missing-whisper-profile"

[audio]
playback_volume = 1.0
playback_speed = 1.0
default_tts_voice = "Bruno"

[safety]
confirmation_confidence_threshold = 0.9
allow_click_without_confirmation = true
always_confirm_submit = true

[ocr]
trigger_on_no_extractable_text = true
sparse_text_char_threshold = 200
sparse_text_region_threshold = 2
prefer_region_ocr = true

[models]
models_dir = "~/.config/blind_browser/models"
check_on_startup = true
auto_download_missing = false

[speech_feedback]
style = "Short"
confirm_setting_changes = true
include_previous_value = false

[remote_profiles.openai-default]
provider = "OpenAi"
base_url = "https://api.openai.com/v1"
model = "gpt-4.1"
api_key = { from_env = "OPENAI_API_KEY" }
temperature_milli = 200
max_output_tokens = 1024
timeout_ms = 30000
"#;

    let error = AppConfig::load_from_str(invalid).expect_err("config should be invalid");

    match error {
        ConfigError::Validation(message) => {
            assert!(message.contains(
                "providers.tts references missing local_profiles.missing-kitten-profile"
            ));
            assert!(message.contains(
                "providers.asr references missing local_profiles.missing-whisper-profile"
            ));
        }
        other => panic!("expected validation error, got {other}"),
    }
}

