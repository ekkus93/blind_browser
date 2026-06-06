use super::*;

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
