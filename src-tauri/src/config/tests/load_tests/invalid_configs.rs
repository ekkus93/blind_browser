use super::*;

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
