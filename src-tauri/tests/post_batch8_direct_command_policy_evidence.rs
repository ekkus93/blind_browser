use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Copy)]
struct Evidence {
    name: &'static str,
    networked: bool,
    credential_bearing: bool,
    verified_model_download: bool,
    transmits_page_context: bool,
}

const EVIDENCE: &[Evidence] = &[
    Evidence {
        name: "resolve_command",
        networked: true,
        credential_bearing: true,
        verified_model_download: false,
        transmits_page_context: true,
    },
    Evidence {
        name: "execute_planner_output",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "submit_confirmation_response",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "submit_remote_planner_consent_response",
        networked: true,
        credential_bearing: true,
        verified_model_download: false,
        transmits_page_context: true,
    },
    Evidence {
        name: "submit_narration_consent_response",
        networked: true,
        credential_bearing: true,
        verified_model_download: false,
        transmits_page_context: true,
    },
    Evidence {
        name: "submit_microphone_consent_response",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "start_listening",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "stop_listening",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "transcribe_command",
        networked: true,
        credential_bearing: true,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "transcribe_and_execute_command",
        networked: true,
        credential_bearing: true,
        verified_model_download: false,
        transmits_page_context: true,
    },
    Evidence {
        name: "open_url",
        networked: true,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "open_external_url",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "get_agent_state",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "set_playback_volume",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "set_playback_speed",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "set_browser_visibility",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "set_tts_voice",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "set_confirmation_threshold",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "set_allow_click_without_confirmation",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "set_remote_planner_privacy_settings",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "set_ocr_thresholds",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "set_remote_planner_connection_settings",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "reset_remote_planner_connection_settings",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "list_remote_planner_models",
        networked: true,
        credential_bearing: true,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "set_remote_planner_api_key",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "set_remote_tts_api_key",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "set_remote_asr_api_key",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "test_remote_planner_api_key",
        networked: true,
        credential_bearing: true,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "test_remote_tts_api_key",
        networked: true,
        credential_bearing: true,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "test_remote_asr_api_key",
        networked: true,
        credential_bearing: true,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "get_model_management_settings",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "set_model_management_settings",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "download_active_local_tts_model",
        networked: true,
        credential_bearing: false,
        verified_model_download: true,
        transmits_page_context: false,
    },
    Evidence {
        name: "download_active_local_asr_model",
        networked: true,
        credential_bearing: false,
        verified_model_download: true,
        transmits_page_context: false,
    },
    Evidence {
        name: "set_tts_provider_selection",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "set_asr_provider_selection",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
    Evidence {
        name: "set_tts_model_selection",
        networked: false,
        credential_bearing: false,
        verified_model_download: false,
        transmits_page_context: false,
    },
];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn source(relative: &str) -> String {
    fs::read_to_string(root().join(relative)).unwrap_or_else(|error| panic!("{relative}: {error}"))
}

fn quoted_strings(value: &str) -> BTreeSet<String> {
    let mut values = BTreeSet::new();
    let mut chars = value.chars().peekable();
    while let Some(character) = chars.next() {
        if character != '"' {
            continue;
        }
        let mut current = String::new();
        let mut escaped = false;
        for next in chars.by_ref() {
            if escaped {
                escaped = false;
                current.push(next);
            } else if next == '\\' {
                escaped = true;
            } else if next == '"' {
                break;
            } else {
                current.push(next);
            }
        }
        if current.chars().all(|c| c.is_ascii_lowercase() || c == '_') && current.contains('_') {
            values.insert(current);
        }
    }
    values
}

fn generated_handlers() -> BTreeSet<String> {
    let lib = source("src/lib.rs");
    let marker = "tauri::generate_handler![";
    let start = lib.find(marker).unwrap() + marker.len();
    let body = &lib[start..];
    let end = body.find("])").unwrap();
    body[..end]
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn registered_handlers() -> BTreeSet<String> {
    let policy = source("src/direct_command_policy.rs");
    let start = policy.find("pub(crate) const fn as_handler_name").unwrap();
    let body = &policy[start..];
    let end = body
        .find("#[derive(Debug, Clone, Copy, PartialEq, Eq)]")
        .unwrap();
    quoted_strings(&body[..end])
}

fn evidence_names() -> BTreeSet<String> {
    EVIDENCE
        .iter()
        .map(|entry| entry.name.to_string())
        .collect()
}

#[test]
fn evidence_inventory_matches_registry_and_tauri_surface() {
    assert_eq!(evidence_names(), registered_handlers());
    assert_eq!(evidence_names(), generated_handlers());
}

#[test]
fn source_drift_networked_direct_commands_retain_timeout_and_redirect_evidence() {
    let api_keys = source("src/app_core/api_key_tools.rs");
    let remote_planner = source("src/app_core/remote_planner.rs");
    let remote_asr = source("src/asr/remote.rs");
    let navigation = source("src/app_core/navigation_tools.rs");
    let runtime_config = source("src/app_core/runtime_config.rs");
    let model_handlers = source("src/command_handlers/model_handlers.rs");
    let model_download = source("src/app_core/model_management/download.rs");

    assert!(api_keys.contains("fn credential_client(timeout_ms: u64, purpose: &str)"));
    assert!(api_keys.matches("timeout_ms.max(1)").count() >= 1);
    assert!(api_keys.contains(".redirect(Policy::none())"));
    assert!(remote_planner.contains(r#"credential_client(profile.timeout_ms, "remote planner")"#));
    assert!(remote_asr.contains(".timeout(Duration::from_millis(timeout_ms))"));
    assert!(remote_asr.contains(".redirect(reqwest::redirect::Policy::none())"));
    assert!(navigation.contains(".open_url(&final_url, load_state, input.timeout_ms)"));
    assert!(runtime_config.contains("fetch_openai_compatible_models"));
    assert!(model_handlers.contains("download_active_local_tts_model"));
    assert!(model_handlers.contains("download_active_local_asr_model"));
    assert!(model_download.contains("MODEL_DOWNLOAD_REQUEST_TIMEOUT"));
    assert!(model_download.contains("model_redirect_policy"));

    let networked = EVIDENCE
        .iter()
        .filter(|entry| entry.networked)
        .map(|entry| entry.name)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        networked,
        BTreeSet::from([
            "resolve_command",
            "submit_remote_planner_consent_response",
            "submit_narration_consent_response",
            "transcribe_command",
            "transcribe_and_execute_command",
            "open_url",
            "list_remote_planner_models",
            "test_remote_planner_api_key",
            "test_remote_tts_api_key",
            "test_remote_asr_api_key",
            "download_active_local_tts_model",
            "download_active_local_asr_model",
        ])
    );
}

#[test]
fn source_drift_credential_bearing_commands_retain_endpoint_binding() {
    let api_keys = source("src/app_core/api_key_tools.rs");
    let remote_planner = source("src/app_core/remote_planner.rs");
    let remote_asr = source("src/asr/remote.rs");
    let runtime_config = source("src/app_core/runtime_config.rs");

    for source in [&api_keys, &remote_planner, &remote_asr, &runtime_config] {
        assert!(source.contains("ProviderEndpointScope"));
        assert!(source.contains("resolve_secret_ref_for_endpoint"));
    }

    let credential_bearing = EVIDENCE
        .iter()
        .filter(|entry| entry.credential_bearing)
        .map(|entry| entry.name)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        credential_bearing,
        BTreeSet::from([
            "resolve_command",
            "submit_remote_planner_consent_response",
            "submit_narration_consent_response",
            "transcribe_command",
            "transcribe_and_execute_command",
            "list_remote_planner_models",
            "test_remote_planner_api_key",
            "test_remote_tts_api_key",
            "test_remote_asr_api_key",
        ])
    );
}

#[test]
fn source_drift_model_downloads_retain_verified_activation_wiring() {
    let handlers = source("src/command_handlers/model_handlers.rs");
    let runtime = source("src/app_core/runtime_config.rs");
    let download = source("src/app_core/model_management/download.rs");

    assert!(handlers.contains(".prepare_active_local_tts_model_download()"));
    assert!(handlers.contains(".prepare_active_local_asr_model_download()"));
    assert!(
        handlers
            .matches(".finalize_local_model_download(completed)")
            .count()
            >= 2
    );
    assert!(runtime.contains("download_hugging_face_directory"));
    assert!(runtime.contains("download_hugging_face_file"));
    assert!(download.contains("write_verified_reader_atomically"));
    assert!(download.contains("HashMismatch"));
    assert!(download.contains("replace_file_atomically"));

    let verified = EVIDENCE
        .iter()
        .filter(|entry| entry.verified_model_download)
        .map(|entry| entry.name)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        verified,
        BTreeSet::from([
            "download_active_local_tts_model",
            "download_active_local_asr_model",
        ])
    );
}

#[test]
fn source_drift_page_context_commands_retain_privacy_sanitizer_wiring() {
    let core_handlers = source("src/command_handlers/core_handlers.rs");
    let voice_handlers = source("src/command_handlers/voice_handlers.rs");
    let replanning = source("src/app_core/replanning_orchestrator.rs");
    let consent = source("src/app_core/remote_data_consent/mod.rs");
    let consent_draft = source("src/app_core/remote_data_consent/draft.rs");
    let redaction = source("src/app_core/planner_redaction/mod.rs");
    let narration = source("src/app_core/narration.rs");
    let narration_consent = source("src/app_core/remote_data_consent/narration_consent.rs");

    assert!(core_handlers.contains("resolve_command_lock_scoped"));
    assert!(voice_handlers.contains("run_command_with_lock_scoped_replanning"));
    assert!(replanning.contains("guard.prepare_remote_planner_request("));
    assert!(consent.contains("match evaluate_remote_planner_policy("));
    assert!(
        consent_draft.contains("sanitize_remote_planner_input_authorized(&planner_input, mode)?")
    );
    assert!(redaction.contains("pub(crate) fn sanitize_remote_planner_input_authorized("));
    // Narration deliberately has no sanitizer -- redacting the text would
    // break the feature (the point is to speak the page text as-is) -- but
    // it must still go through the shared policy gate before any network
    // send, which these assert instead of a sanitizer call.
    assert!(narration.contains(".prepare_narration_request("));
    assert!(narration_consent.contains("match evaluate_remote_planner_policy("));

    let transmitting = EVIDENCE
        .iter()
        .filter(|entry| entry.transmits_page_context)
        .map(|entry| entry.name)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        transmitting,
        BTreeSet::from([
            "resolve_command",
            "submit_remote_planner_consent_response",
            "submit_narration_consent_response",
            "transcribe_and_execute_command",
        ])
    );
}

#[test]
fn source_drift_external_launch_retains_validated_url_and_user_gesture_policy() {
    let handlers = source("src/command_handlers/url_handlers.rs");
    let frontend = source("../src/panel-state-setters.ts");
    let policy = source("src/direct_command_policy.rs");
    assert!(handlers.contains("validate_external_url"));
    assert!(frontend.contains("http:") || frontend.contains("https:"));
    assert!(policy.contains("ValidatedHttpUrlWithUserGesture"));
    assert!(policy.contains("requires_user_gesture"));
}
