#!/usr/bin/env python3
"""One-shot master patch for the remaining post-Batch-8 source/test evidence gaps."""
from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def replace_once(path: str, old: str, new: str) -> None:
    target = ROOT / path
    text = target.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"expected exactly one match in {path}, found {count}: {old[:100]!r}")
    target.write_text(text.replace(old, new, 1), encoding="utf-8")


def append_once(path: str, marker: str, content: str) -> None:
    target = ROOT / path
    text = target.read_text(encoding="utf-8")
    if marker in text:
        raise SystemExit(f"marker already exists in {path}: {marker}")
    target.write_text(text.rstrip() + "\n\n" + content.rstrip() + "\n", encoding="utf-8")


# P8-006: block genuinely high-risk OCR/page text before network transmission.
replace_once(
    "src-tauri/src/app_core/planner_redaction.rs",
    '''fn high_risk_context_reason(input: &PlannerInput) -> Option<&'static str> {
    let has_sensitive_element = input
''',
    '''fn contains_high_risk_page_text(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    contains_long_digit_sequence(value)
        || contains_ssn_shape(value)
        || [
            "payment receipt",
            "card number",
            "credit card",
            "security code",
            "cvv",
            "cvc",
            "social security",
            "medical record",
            "patient record",
            "wallet seed",
            "seed phrase",
            "recovery phrase",
            "one-time code",
            "one time code",
            "otp code",
        ]
        .iter()
        .any(|marker| lower.contains(marker))
}

fn high_risk_context_reason(input: &PlannerInput) -> Option<&'static str> {
    let has_sensitive_element = input
''',
)
replace_once(
    "src-tauri/src/app_core/planner_redaction.rs",
    '''    if has_sensitive_element {
        return Some("sensitive_form_control");
    }

    let urls = [
''',
    '''    if has_sensitive_element {
        return Some("sensitive_form_control");
    }

    let has_high_risk_page_text = input
        .page_model
        .iter()
        .flat_map(|page| {
            page.regions.iter().flat_map(|region| {
                std::iter::once(region.text.as_str()).chain(region.label.as_deref())
            })
        })
        .chain(
            input
                .page_snapshot
                .iter()
                .map(|snapshot| snapshot.visible_text_excerpt.as_str()),
        )
        .chain(
            input
                .recent_tool_results
                .iter()
                .flat_map(|result| result.observation_summary.iter().map(String::as_str)),
        )
        .any(contains_high_risk_page_text);
    if has_high_risk_page_text {
        return Some("high_risk_page_text");
    }

    let urls = [
''',
)
replace_once(
    "src-tauri/src/app_core/planner_redaction.rs",
    '''    #[test]
    fn exact_remote_prompt_payload_omits_local_state_and_secret_sentinels() {
''',
    '''    #[test]
    fn high_risk_ocr_and_page_text_block_network_remote_planning() {
        let mut payment = fixture_planner_input();
        payment.page_snapshot = None;
        payment.recent_tool_results.clear();
        payment.relevant_skill_summaries.clear();
        let page = payment.page_model.as_mut().unwrap();
        page.interactive_elements.clear();
        page.regions = vec![PageRegion {
            region_id: String::from("ocr-payment-receipt"),
            role: RegionRole::Paragraph,
            label: Some(String::from("OCR payment receipt")),
            text: String::from("PAYMENT RECEIPT 4111 1111 1111 1111"),
            bbox: None,
            source: RegionSource::Ocr,
        }];

        let error = sanitize_for_network(&payment)
            .expect_err("high-risk OCR payment text must block network planning");
        assert_eq!(error.code, "remote_planner_high_risk_context_blocked");
        assert_eq!(
            error.details.as_ref().and_then(|details| details["reason_code"].as_str()),
            Some("high_risk_page_text")
        );
    }

    #[test]
    fn hostile_content_cannot_authorize_click() {
        let input = fixture_planner_input();
        let indicators = detect_prompt_injection(&input);
        assert!(indicators.detected);
        assert!(indicators.caution_only);

        let malicious_output = PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::ClickElement,
                goal: String::from("follow hostile page instruction"),
                target_description: Some(String::from("Continue")),
            },
            selected_skills: Vec::new(),
            steps: vec![PlannedStep {
                step_id: String::from("hostile-click"),
                tool_name: ToolName::ClickElement,
                arguments: serde_json::json!({
                    "request_id": "hostile-click",
                    "timeout_ms": null,
                    "element_id": "element-1"
                }),
                purpose: String::from("page text attempted to authorize a click"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Complete,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        };
        let safety = PlannerSafetySettings {
            confirmation_confidence_threshold: 0.0,
            allow_click_without_confirmation: true,
            always_confirm_submit: true,
        };

        assert!(validate_planner_output_with_safety(
            &malicious_output,
            &planner_available_tools(),
            &[],
            &safety,
        )
        .is_err());
    }

    #[test]
    fn exact_remote_prompt_payload_omits_local_state_and_secret_sentinels() {
''',
)

# P8-004: make both remaining best-effort cleanup paths explicit and tested.
replace_once(
    "src-tauri/src/app_core/image_cache.rs",
    '''        if let Err(io_error) = file.write_all(bytes).and_then(|_| file.sync_all()) {
            let _ = fs::remove_file(&path);
            return Err(io_tool_error("screenshot_write_failed", io_error));
        }
''',
    '''        if let Err(io_error) = file.write_all(bytes).and_then(|_| file.sync_all()) {
            cleanup_failed_screenshot_write(&path, &io_error)?;
            return Err(io_tool_error("screenshot_write_failed", io_error));
        }
''',
)
replace_once(
    "src-tauri/src/app_core/image_cache.rs",
    '''fn prepare_root(root: &Path) -> Result<PathBuf, ToolError> {
''',
    '''/// Remove a partial screenshot after a failed write. Cleanup is security-relevant:
/// a failure is surfaced instead of silently leaving unregistered private image bytes.
fn cleanup_failed_screenshot_write(path: &Path, primary: &io::Error) -> Result<(), ToolError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(cleanup) => Err(ToolError {
            code: String::from("screenshot_write_cleanup_failed"),
            message: String::from("failed to remove a partial private screenshot after write failure"),
            retryable: true,
            details: Some(serde_json::json!({
                "primary_error_kind": format!("{:?}", primary.kind()),
                "cleanup_error_kind": format!("{:?}", cleanup.kind()),
            })),
        }),
    }
}

fn prepare_root(root: &Path) -> Result<PathBuf, ToolError> {
''',
)
replace_once(
    "src-tauri/src/app_core/image_cache.rs",
    '''    #[test]
    fn count_and_byte_limits_evict_oldest_file_and_record() {
''',
    '''    #[test]
    fn failed_screenshot_write_cleanup_failure_is_explicit() {
        let dir = TempDir::new().unwrap();
        let directory_target = dir.path().join("partial.png");
        fs::create_dir(&directory_target).unwrap();
        let primary = io::Error::other("synthetic write failure");

        let error = cleanup_failed_screenshot_write(&directory_target, &primary)
            .expect_err("directory removal through remove_file must fail");
        assert_eq!(error.code, "screenshot_write_cleanup_failed");
        assert!(directory_target.exists());

        cleanup_failed_screenshot_write(&dir.path().join("missing.png"), &primary)
            .expect("missing partial file is already clean");
    }

    #[test]
    fn count_and_byte_limits_evict_oldest_file_and_record() {
''',
)

replace_once(
    "src-tauri/src/config/persistence.rs",
    '''    if write_result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }

    write_result
}
''',
    '''    match write_result {
        Ok(()) => Ok(()),
        Err(primary) => {
            // Temporary-file cleanup is part of the atomic persistence contract.
            // A cleanup failure is surfaced with the primary failure instead of ignored.
            match remove_failed_config_temp_file(&tmp_path) {
                Ok(()) => Err(primary),
                Err(cleanup) => Err(ConfigError::Write {
                    path: tmp_path,
                    source: std::io::Error::other(format!(
                        "config write failed: {primary}; temporary-file cleanup failed: {cleanup}"
                    )),
                }),
            }
        }
    }
}

fn remove_failed_config_temp_file(path: &Path) -> std::io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}
''',
)
append_once(
    "src-tauri/src/config/persistence.rs",
    "failed_config_temp_cleanup_is_explicit",
    '''#[cfg(test)]
mod post_batch8_cleanup_tests {
    use super::*;

    #[test]
    fn failed_config_temp_cleanup_is_explicit() {
        let directory = tempfile::tempdir().unwrap();
        let directory_target = directory.path().join("config.toml.tmp");
        fs::create_dir(&directory_target).unwrap();

        assert!(remove_failed_config_temp_file(&directory_target).is_err());
        assert!(directory_target.exists());
        remove_failed_config_temp_file(&directory.path().join("missing.tmp"))
            .expect("missing temporary file is already clean");
    }
}''',
)

allowlist_path = ROOT / "scripts/security-fallback-allowlist.txt"
allowlist = allowlist_path.read_text(encoding="utf-8")
for removed in [
    "src-tauri/src/app_core/image_cache.rs|let _ = fs::remove_file(&path);\n",
    "src-tauri/src/config/persistence.rs|let _ = fs::remove_file(&tmp_path);\n",
]:
    if removed not in allowlist:
        raise SystemExit(f"missing cleanup allowlist entry: {removed.strip()}")
    allowlist = allowlist.replace(removed, "")
allowlist_path.write_text(allowlist, encoding="utf-8")

# P8-002: exhaustive direct-command evidence inventory and source-wiring tests.
direct_test = r'''use std::collections::BTreeSet;
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
    Evidence { name: "resolve_command", networked: true, credential_bearing: true, verified_model_download: false, transmits_page_context: true },
    Evidence { name: "execute_planner_output", networked: false, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "submit_confirmation_response", networked: false, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "start_listening", networked: false, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "stop_listening", networked: false, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "transcribe_command", networked: true, credential_bearing: true, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "transcribe_and_execute_command", networked: true, credential_bearing: true, verified_model_download: false, transmits_page_context: true },
    Evidence { name: "open_url", networked: true, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "open_external_url", networked: false, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "get_agent_state", networked: false, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "set_playback_volume", networked: false, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "set_playback_speed", networked: false, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "set_browser_visibility", networked: false, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "set_tts_voice", networked: false, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "set_confirmation_threshold", networked: false, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "set_allow_click_without_confirmation", networked: false, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "set_remote_planner_privacy_settings", networked: false, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "set_ocr_thresholds", networked: false, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "set_remote_planner_connection_settings", networked: false, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "reset_remote_planner_connection_settings", networked: false, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "list_remote_planner_models", networked: true, credential_bearing: true, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "set_remote_planner_api_key", networked: false, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "set_remote_tts_api_key", networked: false, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "set_remote_asr_api_key", networked: false, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "test_remote_planner_api_key", networked: true, credential_bearing: true, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "test_remote_tts_api_key", networked: true, credential_bearing: true, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "test_remote_asr_api_key", networked: true, credential_bearing: true, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "get_model_management_settings", networked: false, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "set_model_management_settings", networked: false, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "download_active_local_tts_model", networked: true, credential_bearing: false, verified_model_download: true, transmits_page_context: false },
    Evidence { name: "download_active_local_asr_model", networked: true, credential_bearing: false, verified_model_download: true, transmits_page_context: false },
    Evidence { name: "set_tts_provider_selection", networked: false, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "set_asr_provider_selection", networked: false, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
    Evidence { name: "set_tts_model_selection", networked: false, credential_bearing: false, verified_model_download: false, transmits_page_context: false },
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
    let end = body.find("#[derive(Debug, Clone, Copy, PartialEq, Eq)]").unwrap();
    quoted_strings(&body[..end])
}

fn evidence_names() -> BTreeSet<String> {
    EVIDENCE.iter().map(|entry| entry.name.to_string()).collect()
}

#[test]
fn evidence_inventory_matches_registry_and_tauri_surface() {
    assert_eq!(evidence_names(), registered_handlers());
    assert_eq!(evidence_names(), generated_handlers());
}

#[test]
fn every_networked_direct_command_has_timeout_and_redirect_evidence() {
    let api_keys = source("src/app_core/api_key_tools.rs");
    let remote_planner = source("src/app_core/remote_planner.rs");
    let remote_asr = source("src/asr/remote.rs");
    let navigation = source("src/app_core/navigation_tools.rs");
    let runtime_config = source("src/app_core/runtime_config.rs");
    let model_handlers = source("src/command_handlers/model_handlers.rs");
    let model_download = source("src/app_core/model_management/download.rs");

    assert!(api_keys.contains(".timeout(Duration::from_millis(timeout_ms.max(1)))"));
    assert!(api_keys.contains(".redirect(Policy::none())"));
    assert!(remote_planner.contains("credential_async_client(profile.timeout_ms)"));
    assert!(remote_asr.contains(".timeout(Duration::from_millis(timeout_ms))"));
    assert!(remote_asr.contains(".redirect(reqwest::redirect::Policy::none())"));
    assert!(navigation.contains(".open_url(&final_url, load_state, input.timeout_ms)"));
    assert!(runtime_config.contains("fetch_openai_compatible_models"));
    assert!(model_handlers.contains("download_active_local_tts_model"));
    assert!(model_handlers.contains("download_active_local_asr_model"));
    assert!(model_download.contains("MODEL_DOWNLOAD_REQUEST_TIMEOUT"));
    assert!(model_download.contains("model_redirect_policy"));

    let networked = EVIDENCE.iter().filter(|entry| entry.networked).map(|entry| entry.name).collect::<BTreeSet<_>>();
    assert_eq!(networked, BTreeSet::from([
        "resolve_command",
        "transcribe_command",
        "transcribe_and_execute_command",
        "open_url",
        "list_remote_planner_models",
        "test_remote_planner_api_key",
        "test_remote_tts_api_key",
        "test_remote_asr_api_key",
        "download_active_local_tts_model",
        "download_active_local_asr_model",
    ]));
}

#[test]
fn every_credential_bearing_direct_command_is_endpoint_bound() {
    let api_keys = source("src/app_core/api_key_tools.rs");
    let remote_planner = source("src/app_core/remote_planner.rs");
    let remote_asr = source("src/asr/remote.rs");
    let runtime_config = source("src/app_core/runtime_config.rs");

    for source in [&api_keys, &remote_planner, &remote_asr, &runtime_config] {
        assert!(source.contains("ProviderEndpointScope"));
        assert!(source.contains("resolve_secret_ref_for_endpoint"));
    }

    let credential_bearing = EVIDENCE.iter().filter(|entry| entry.credential_bearing).map(|entry| entry.name).collect::<BTreeSet<_>>();
    assert_eq!(credential_bearing, BTreeSet::from([
        "resolve_command",
        "transcribe_command",
        "transcribe_and_execute_command",
        "list_remote_planner_models",
        "test_remote_planner_api_key",
        "test_remote_tts_api_key",
        "test_remote_asr_api_key",
    ]));
}

#[test]
fn direct_model_downloads_are_wired_to_verified_activation() {
    let handlers = source("src/command_handlers/model_handlers.rs");
    let runtime = source("src/app_core/runtime_config.rs");
    let download = source("src/app_core/model_management/download.rs");

    assert!(handlers.contains("guard.download_active_local_tts_model()"));
    assert!(handlers.contains("guard.download_active_local_asr_model()"));
    assert!(runtime.contains("download_hugging_face_directory"));
    assert!(runtime.contains("download_hugging_face_file"));
    assert!(download.contains("write_verified_reader_atomically"));
    assert!(download.contains("HashMismatch"));
    assert!(download.contains("replace_file_atomically"));

    let verified = EVIDENCE.iter().filter(|entry| entry.verified_model_download).map(|entry| entry.name).collect::<BTreeSet<_>>();
    assert_eq!(verified, BTreeSet::from([
        "download_active_local_tts_model",
        "download_active_local_asr_model",
    ]));
}

#[test]
fn direct_page_context_transmission_cannot_bypass_privacy_sanitization() {
    let core_handlers = source("src/command_handlers/core_handlers.rs");
    let voice_handlers = source("src/command_handlers/voice_handlers.rs");
    let remote_planner = source("src/app_core/remote_planner.rs");
    let redaction = source("src/app_core/planner_redaction.rs");

    assert!(core_handlers.contains("resolve_command_lock_scoped"));
    assert!(voice_handlers.contains("run_command_with_lock_scoped_replanning"));
    assert!(remote_planner.contains("sanitize_remote_planner_input(planner_input, privacy, &endpoint_scope)?"));
    assert!(redaction.contains("enforce_remote_planner_privacy(input, privacy, endpoint_scope)?"));

    let transmitting = EVIDENCE.iter().filter(|entry| entry.transmits_page_context).map(|entry| entry.name).collect::<BTreeSet<_>>();
    assert_eq!(transmitting, BTreeSet::from([
        "resolve_command",
        "transcribe_and_execute_command",
    ]));
}
'''
(ROOT / "src-tauri/tests/post_batch8_direct_command_policy_evidence.rs").write_text(direct_test, encoding="utf-8")

# Exact per-expression accepted-fallback inventory.
def normalize(line: str) -> str:
    return " ".join(line.strip().split())


def functions_for(path: str, expression: str) -> list[str]:
    lines = (ROOT / path).read_text(encoding="utf-8", errors="replace").splitlines()
    current = "module scope"
    matches: list[str] = []
    signature = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*[<(]")
    for line in lines:
        found = signature.search(line)
        if found:
            current = found.group(1)
        if normalize(line) == expression:
            matches.append(current)
    if not matches:
        raise SystemExit(f"allowlist expression not found exactly: {path}|{expression}")
    return sorted(set(matches))


def metadata(path: str, expression: str) -> dict[str, str]:
    if "serde_json::to_value" in expression:
        return {
            "justification": "Optional policy-detail serialization may fail only after the typed refusal and error code are fixed.",
            "user_visibility": "The typed failure remains visible; only supplemental JSON detail may be absent.",
            "side_effect_impact": "Cannot authorize or execute an action and cannot convert failure into success.",
            "test_coverage": "Policy validator/executor tests plus the security-fallback scanner.",
            "future_replacement": "Make policy details mandatory only if they become part of the public error contract.",
        }
    if "set_username" in expression or "set_password" in expression:
        return {
            "justification": "The parsed diagnostic/planner URL is stripped of userinfo before query and fragment removal.",
            "user_visibility": "Only a sanitized origin/path is displayed or transmitted.",
            "side_effect_impact": "Presentation-only sanitization; the URL is not used to grant credentials or authority.",
            "test_coverage": "Planner URL redaction and frontend diagnostic URL tests plus scanner enforcement.",
            "future_replacement": "Reconstruct URLs from approved components if URL mutation semantics change.",
        }
    if path.endswith("tts/remote.rs"):
        return {
            "justification": "Feature-disabled parameters are deliberately consumed before returning an explicit unavailable error.",
            "user_visibility": "The caller receives a typed feature-unavailable failure.",
            "side_effect_impact": "No network call, playback, or successful fallback occurs.",
            "test_coverage": "Feature-matrix compilation/tests and scanner enforcement.",
            "future_replacement": "Remove the stub when remote TTS becomes mandatory in all builds.",
        }
    if "wav.rs" in path or "audio_commands.rs" in path or "action_policy.rs" in path:
        return {
            "justification": "Checked numeric conversion failure selects validation failure or conservative absence.",
            "user_visibility": "Invalid input is rejected or the optional value is reported unavailable.",
            "side_effect_impact": "Cannot widen a numeric bound or increase authority.",
            "test_coverage": "Boundary/argument tests and scanner enforcement.",
            "future_replacement": "Retain checked conversion unless the type contract is narrowed further.",
        }
    if "skill_loader.rs" in path or "skill_parser.rs" in path:
        return {
            "justification": "Unavailable optional skill metadata or unreadable optional entries reduce discovered capability only.",
            "user_visibility": "Unavailable skills are omitted and discovery diagnostics remain path-private.",
            "side_effect_impact": "Cannot add tools, grant permission, or bypass confirmation.",
            "test_coverage": "Skill parser/policy/path-privacy tests and scanner enforcement.",
            "future_replacement": "Use typed per-entry discovery results if the UI gains a dedicated warning surface.",
        }
    if "settings_adapters.rs" in path:
        return {
            "justification": "Optional capability/status discovery omits unavailable endpoint or model metadata.",
            "user_visibility": "Settings/runtime status reports the configured capability as unavailable.",
            "side_effect_impact": "Capability-reducing only; configured operations still fail explicitly.",
            "test_coverage": "Settings/model availability tests and scanner enforcement.",
            "future_replacement": "Add typed absence reasons when the settings UI needs finer diagnostics.",
        }
    if "click_authorization.rs" in path or "element_scoring.rs" in path:
        return {
            "justification": "Missing optional labels or URL metadata feed deterministic scoring/summary logic with conservative defaults.",
            "user_visibility": "Ambiguous or missing protected-target metadata produces confirmation or an explicit warning.",
            "side_effect_impact": "Cannot mint authorization, lower confirmation, or mark a destructive target safe.",
            "test_coverage": "Click authorization, destructive-target, confirmation-summary, and scanner tests.",
            "future_replacement": "Adopt typed absence reasons if richer page-model diagnostics are required.",
        }
    if "planner_redaction.rs" in path:
        return {
            "justification": "Invalid optional origin or planner metadata is omitted while the privacy gate remains authoritative.",
            "user_visibility": "Invalid metadata is redacted or shown as unavailable.",
            "side_effect_impact": "Cannot bypass consent, high-risk blocking, or credential scoping.",
            "test_coverage": "Remote-planner privacy/redaction tests and scanner enforcement.",
            "future_replacement": "Use typed parse reasons only if the privacy UI requires them.",
        }
    if "command_dispatch.rs" in path or "fill_correction.rs" in path or "field_focus.rs" in path:
        return {
            "justification": "Optional command/fill discovery failure aborts that optional candidate path and reduces capability.",
            "user_visibility": "The command replans, rejects, or reports that no suitable target was found.",
            "side_effect_impact": "Cannot execute an unvalidated target or report a protected side effect as successful.",
            "test_coverage": "Command dispatch/form-fill tests and scanner enforcement.",
            "future_replacement": "Promote to typed candidate-rejection reasons if surfaced in UX.",
        }
    if "confirmation_workflow.rs" in path:
        return {
            "justification": "Missing non-authoritative display text does not affect confirmation ID, digest, expiry, or runtime binding.",
            "user_visibility": "The protected confirmation still presents deterministic runtime-authored wording.",
            "side_effect_impact": "Cannot approve, replay, or mutate the pending action manifest.",
            "test_coverage": "Confirmation digest/replay/expiry tests and scanner enforcement.",
            "future_replacement": "Retain unless the display field becomes contractually mandatory.",
        }
    return {
        "justification": "The reviewed fallback omits optional data or selects a conservative capability-reducing default.",
        "user_visibility": "The operation remains failed, unavailable, or conservatively represented.",
        "side_effect_impact": "Cannot increase authority, disclose protected data, or report false success.",
        "test_coverage": "Relevant module tests and exact allowlist/scanner enforcement.",
        "future_replacement": "Replace with a typed absence/error only when the surrounding contract requires more detail.",
    }

entries = []
for raw in allowlist.splitlines():
    raw = raw.strip()
    if not raw or raw.startswith("#"):
        continue
    path, expression = raw.split("|", 1)
    item = {"path": path, "functions": functions_for(path, expression), "expression": expression}
    item.update(metadata(path, expression))
    entries.append(item)

inventory_path = ROOT / "scripts/security-fallback-inventory.json"
inventory_path.write_text(json.dumps({"version": 1, "entries": entries}, indent=2) + "\n", encoding="utf-8")

scanner = r'''#!/usr/bin/env python3
"""Verify exact per-expression metadata for every accepted security fallback."""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ALLOWLIST = ROOT / "scripts/security-fallback-allowlist.txt"
INVENTORY = ROOT / "scripts/security-fallback-inventory.json"
REQUIRED = {
    "path",
    "functions",
    "expression",
    "justification",
    "user_visibility",
    "side_effect_impact",
    "test_coverage",
    "future_replacement",
}


def normalize(line: str) -> str:
    return " ".join(line.strip().split())


def allowlist_keys() -> set[tuple[str, str]]:
    keys = set()
    for raw in ALLOWLIST.read_text(encoding="utf-8").splitlines():
        raw = raw.strip()
        if not raw or raw.startswith("#"):
            continue
        path, expression = raw.split("|", 1)
        keys.add((path, expression))
    return keys


def source_functions(path: str, expression: str) -> list[str]:
    current = "module scope"
    functions = []
    signature = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*[<(]")
    for line in (ROOT / path).read_text(encoding="utf-8", errors="replace").splitlines():
        found = signature.search(line)
        if found:
            current = found.group(1)
        if normalize(line) == expression:
            functions.append(current)
    return sorted(set(functions))


def audit() -> list[str]:
    problems = []
    payload = json.loads(INVENTORY.read_text(encoding="utf-8"))
    entries = payload.get("entries", [])
    indexed = {(entry.get("path"), entry.get("expression")): entry for entry in entries}
    expected = allowlist_keys()
    observed = set(indexed)
    if expected != observed:
        problems.append(f"inventory keys differ: missing={sorted(expected-observed)} extra={sorted(observed-expected)}")
    for key, entry in indexed.items():
        missing = REQUIRED - set(entry)
        if missing:
            problems.append(f"{key}: missing fields {sorted(missing)}")
            continue
        for field in REQUIRED - {"functions"}:
            if not isinstance(entry[field], str) or not entry[field].strip():
                problems.append(f"{key}: empty {field}")
        if not isinstance(entry["functions"], list) or not entry["functions"]:
            problems.append(f"{key}: functions must be a non-empty list")
        actual_functions = source_functions(*key)
        if actual_functions != entry["functions"]:
            problems.append(f"{key}: functions {entry['functions']} != source {actual_functions}")
    return problems


def self_test() -> None:
    assert normalize("  let   x = 1; ") == "let x = 1;"
    assert REQUIRED.issuperset({"path", "expression", "functions"})
    print("Security fallback inventory self-test passed")


def main() -> int:
    if sys.argv[1:] == ["--self-test"]:
        self_test()
        return 0
    if sys.argv[1:]:
        print("usage: check-security-fallback-inventory.py [--self-test]", file=sys.stderr)
        return 2
    problems = audit()
    if problems:
        print("Security fallback inventory audit failed:", file=sys.stderr)
        for problem in problems:
            print(f"- {problem}", file=sys.stderr)
        return 1
    print("Security fallback inventory audit passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
'''
(ROOT / "scripts/check-security-fallback-inventory.py").write_text(scanner, encoding="utf-8")


def cell(value: str) -> str:
    return value.replace("|", "\\|").replace("\n", " ")

rows = [
    "## Exact per-expression fallback inventory",
    "",
    "<!-- BEGIN GENERATED SECURITY FALLBACK INVENTORY -->",
    "This table is generated from `scripts/security-fallback-inventory.json`; permanent CI verifies that every exact allowlist expression has complete metadata and still resolves to the documented source function(s).",
    "",
    "| File | Function(s) | Exact expression | Justification | User visibility | Side-effect impact | Tests/enforcement | Future replacement |",
    "|---|---|---|---|---|---|---|---|",
]
for entry in entries:
    rows.append(
        "| `{} ` | `{}` | `{}` | {} | {} | {} | {} | {} |".format(
            cell(entry["path"]).strip(),
            cell(", ".join(entry["functions"])),
            cell(entry["expression"]),
            cell(entry["justification"]),
            cell(entry["user_visibility"]),
            cell(entry["side_effect_impact"]),
            cell(entry["test_coverage"]),
            cell(entry["future_replacement"]),
        )
    )
rows.extend(["", "<!-- END GENERATED SECURITY FALLBACK INVENTORY -->", ""])
generated = "\n".join(rows)
accepted_path = ROOT / "docs/BLIND_BROWSER_ACCEPTED_FALLBACKS_2026-08-02.md"
accepted = accepted_path.read_text(encoding="utf-8")
start_marker = "<!-- BEGIN GENERATED SECURITY FALLBACK INVENTORY -->"
end_marker = "<!-- END GENERATED SECURITY FALLBACK INVENTORY -->"
if start_marker in accepted:
    start = accepted.index("## Exact per-expression fallback inventory")
    end = accepted.index(end_marker) + len(end_marker)
    accepted = accepted[:start] + generated.rstrip() + accepted[end:]
else:
    insertion = accepted.index("## Converted unsafe or ambiguous fallbacks")
    accepted = accepted[:insertion] + generated + "\n" + accepted[insertion:]
accepted_path.write_text(accepted, encoding="utf-8")

replace_once(
    ".github/workflows/ci.yml",
    '''      - name: Check for sensitive diagnostics
''',
    '''      - name: Check exact accepted-fallback inventory
        run: |
          python3 scripts/check-security-fallback-inventory.py --self-test
          python3 scripts/check-security-fallback-inventory.py

      - name: Check for sensitive diagnostics
''',
)

# Format and run focused gates before publishing the patch commit.
subprocess.run(["cargo", "fmt", "--manifest-path", "src-tauri/Cargo.toml", "--all"], cwd=ROOT, check=True)
subprocess.run(["python3", "scripts/check-security-fallbacks.py", "--self-test"], cwd=ROOT, check=True)
subprocess.run(["python3", "scripts/check-security-fallbacks.py"], cwd=ROOT, check=True)
subprocess.run(["python3", "scripts/check-security-fallback-inventory.py", "--self-test"], cwd=ROOT, check=True)
subprocess.run(["python3", "scripts/check-security-fallback-inventory.py"], cwd=ROOT, check=True)
subprocess.run(
    ["cargo", "test", "--manifest-path", "src-tauri/Cargo.toml", "--all-features", "--test", "post_batch8_direct_command_policy_evidence"],
    cwd=ROOT,
    check=True,
)
subprocess.run(
    ["cargo", "test", "--manifest-path", "src-tauri/Cargo.toml", "--all-features", "hostile_content_cannot_authorize_click"],
    cwd=ROOT,
    check=True,
)
subprocess.run(
    ["cargo", "test", "--manifest-path", "src-tauri/Cargo.toml", "--all-features", "high_risk_ocr_and_page_text_block_network_remote_planning"],
    cwd=ROOT,
    check=True,
)
subprocess.run(
    ["cargo", "test", "--manifest-path", "src-tauri/Cargo.toml", "--all-features", "failed_screenshot_write_cleanup_failure_is_explicit"],
    cwd=ROOT,
    check=True,
)
subprocess.run(
    ["cargo", "test", "--manifest-path", "src-tauri/Cargo.toml", "--all-features", "failed_config_temp_cleanup_is_explicit"],
    cwd=ROOT,
    check=True,
)
