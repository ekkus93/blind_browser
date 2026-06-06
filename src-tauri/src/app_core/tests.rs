use super::settings_adapters::{
    build_asr_provider_settings, build_confirmation_settings, build_local_asr_model_settings,
    build_local_tts_model_settings, build_ocr_threshold_settings, build_provider_failover_settings,
    build_remote_asr_settings, build_remote_planner_settings, build_remote_tts_settings,
    build_tts_model_settings, build_tts_provider_settings, build_tts_voice_settings,
};
use super::replanning::execute_bounded_replanning_loop;
use super::planner_prompt::{planner_interpretation_unavailable_error, planner_system_prompt};
use super::form_fill::{
    resolve_direct_fill_and_submit_command, resolve_direct_fill_field_command,
    resolve_direct_focus_field_command, resolve_direct_submit_form_command,
    resolve_recent_fill_correction_command, RecentFieldContext,
};
use super::element_scoring::{
    build_find_element_query, determine_find_element_resolution, filter_interactive_elements,
    normalize_optional_text, rank_find_element_candidates, region_bbox_by_id,
};
use super::interaction_tools::{
    resolve_clickable_element, resolve_form_element, resolve_typeable_element,
};
use super::api_key_tools::{fetch_openai_compatible_models, test_openai_api_key_connectivity};
use super::extraction_tools::should_trigger_extract_page_model_ocr_fallback;
use super::ocr_merge::{
    extracted_text_metrics, merge_ocr_text_into_page_model, merged_region_text,
    region_first_ocr_target_ids,
};
use super::page_model_builder::{
    build_extracted_page_model, build_visible_text_excerpt, infer_extraction_source,
};
use super::navigation_tools::{
    browser_error_to_tool_error, clear_navigation_follow_up_state, normalize_absolute_url,
    refresh_current_page_after_navigation,
};
use super::replanning::ReplanningRuntime;
use crate::audio_io::RuntimeAudioState;
use crate::browser::BrowserError;
use crate::commands::{
    ExecutionOutcome, ExecutionTrace, ExtractPageModelInput, FindElementInput, IntentName,
    IntentSummary, PlannedStep, PlannerOutput, PlannerStatus, PlannerToolHistoryEntry,
    ReportStatus, StepTransition, ToolName, ToolResult,
};
use crate::config::{AppConfig, KeyringRef, ProviderMode, SecretRef};
use crate::ocr::OcrSettings;
use crate::page_model::{
    ElementRole, ExtractionSource, InteractiveElement, PageModel, PageRegion, Rect, RegionRole,
    RegionSource,
};
use crate::state::AppState;

fn spawn_openai_models_test_server(
    status_line: &str,
    response_body: &str,
) -> (String, std::thread::JoinHandle<()>) {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    let listener =
        TcpListener::bind("127.0.0.1:0").expect("test server should bind an ephemeral port");
    let address = listener
        .local_addr()
        .expect("test server should expose its bound address");
    let base_url = format!("http://{address}/v1");
    let body = response_body.to_string();
    let status_line = status_line.to_string();

    let handle = std::thread::spawn(move || {
        let (mut stream, _) = listener
            .accept()
            .expect("test server should accept one request");
        let mut buffer = [0_u8; 8192];
        let bytes_read = stream
            .read(&mut buffer)
            .expect("test server should read request bytes");
        let request = String::from_utf8_lossy(&buffer[..bytes_read]);

        assert!(
            request.starts_with("GET /v1/models HTTP/1.1\r\n"),
            "expected GET /v1/models request: {request}"
        );
        assert!(
            request.contains("authorization: Bearer blind-browser-test-key")
                || request.contains("Authorization: Bearer blind-browser-test-key"),
            "expected bearer auth header in request: {request}"
        );
        assert!(
            request.contains("openai-organization: org_test")
                || request.contains("OpenAI-Organization: org_test"),
            "expected organization header in request: {request}"
        );
        assert!(
            request.contains("openai-project: proj_test")
                || request.contains("OpenAI-Project: proj_test"),
            "expected project header in request: {request}"
        );

        let headers = format!(
            "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        stream
            .write_all(headers.as_bytes())
            .expect("test server should write response headers");
        stream
            .write_all(body.as_bytes())
            .expect("test server should write response body");
        stream.flush().expect("test server should flush response");
    });

    (base_url, handle)
}

fn fixture_page(interactive_elements: Vec<InteractiveElement>) -> PageModel {
    PageModel {
        title: Some(String::from("Example form")),
        url: Some(String::from("https://example.com/form")),
        regions: Vec::new(),
        interactive_elements,
    }
}

fn fixture_page_with_metadata(
    title: &str,
    url: &str,
    interactive_elements: Vec<InteractiveElement>,
) -> PageModel {
    PageModel {
        title: Some(String::from(title)),
        url: Some(String::from(url)),
        regions: Vec::new(),
        interactive_elements,
    }
}

fn fixture_field(
    element_id: &str,
    dom_locator: &str,
    accessible_name: &str,
    placeholder: &str,
) -> InteractiveElement {
    InteractiveElement {
        element_id: String::from(element_id),
        dom_locator: Some(String::from(dom_locator)),
        role: ElementRole::Input,
        tag_name: String::from("input"),
        text: None,
        accessible_name: Some(String::from(accessible_name)),
        placeholder: Some(String::from(placeholder)),
        href: None,
        value: None,
        bbox: None,
        visible: true,
        enabled: true,
        attributes: std::collections::BTreeMap::new(),
    }
}

fn fixture_form(
    element_id: &str,
    dom_locator: &str,
    accessible_name: &str,
) -> InteractiveElement {
    InteractiveElement {
        element_id: String::from(element_id),
        dom_locator: Some(String::from(dom_locator)),
        role: ElementRole::Form,
        tag_name: String::from("form"),
        text: Some(String::from(accessible_name)),
        accessible_name: Some(String::from(accessible_name)),
        placeholder: None,
        href: None,
        value: None,
        bbox: None,
        visible: true,
        enabled: true,
        attributes: std::collections::BTreeMap::new(),
    }
}

fn fixture_problematic_checkout_page() -> PageModel {
    fixture_page_with_metadata(
        "Example Shop | Checkout",
        "https://shop.example.com/checkout",
        vec![
            fixture_form("form-shipping", "#shipping-form", "Shipping address"),
            fixture_field(
                "input-shipping-email",
                "#shipping-email",
                "Shipping email",
                "Email for shipping updates",
            ),
            fixture_field(
                "input-shipping-name",
                "#shipping-name",
                "Full name",
                "Full name",
            ),
            fixture_form("form-billing", "#billing-form", "Billing address"),
            fixture_field(
                "input-billing-email",
                "#billing-email",
                "Billing email",
                "Billing email for receipts",
            ),
            fixture_field(
                "input-card-name",
                "#card-name",
                "Name on card",
                "Name on card",
            ),
        ],
    )
}

fn fixture_problematic_landing_page() -> PageModel {
    fixture_page_with_metadata(
        "Example Cloud | Start free trial",
        "https://www.example.com/start",
        vec![
            InteractiveElement {
                element_id: String::from("button-hero-get-started"),
                dom_locator: Some(String::from("#hero-get-started")),
                role: ElementRole::Button,
                tag_name: String::from("button"),
                text: Some(String::from("Get started")),
                accessible_name: Some(String::from("Get started")),
                placeholder: None,
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
            InteractiveElement {
                element_id: String::from("button-footer-get-started"),
                dom_locator: Some(String::from("#footer-get-started")),
                role: ElementRole::Button,
                tag_name: String::from("button"),
                text: Some(String::from("Get started")),
                accessible_name: Some(String::from("Get started")),
                placeholder: None,
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
        ],
    )
}

fn fixture_problematic_newsletter_page() -> PageModel {
    fixture_page_with_metadata(
        "Metro news | Sign up for morning headlines",
        "https://news.example.com/newsletters/morning-headlines",
        vec![fixture_field(
            "input-newsletter-email",
            "#newsletter-email",
            "Email",
            "Email address",
        )],
    )
}

fn planner_tool_sequence(planner_output: &PlannerOutput) -> Vec<ToolName> {
    planner_output
        .steps
        .iter()
        .map(|step| step.tool_name.clone())
        .collect()
}

#[derive(Clone, Copy)]
enum AppCorePlannerFixtureKind {
    FocusField,
    FillField,
    FillAndSubmit,
    FollowUpCorrection,
    SubmitForm,
}

struct AppCorePlannerFixture {
    name: &'static str,
    kind: AppCorePlannerFixtureKind,
    transcript: &'static str,
    current_page_id: Option<&'static str>,
    page: Option<PageModel>,
    active_skills: Vec<&'static str>,
    recent_context: Option<RecentFieldContext>,
    confirmation_threshold: f32,
    expected_intent: IntentName,
    expected_status: PlannerStatus,
    expected_selected_skills: Vec<&'static str>,
    expected_tool_sequence: Vec<ToolName>,
    expected_focus_element_id: Option<&'static str>,
    expected_typed_text: Option<&'static str>,
    expected_next_active_element_id: Option<&'static str>,
    expected_next_pending_text: Option<&'static str>,
}

fn resolve_app_core_planner_fixture(
    fixture: &AppCorePlannerFixture,
) -> (PlannerOutput, Option<RecentFieldContext>) {
    match fixture.kind {
        AppCorePlannerFixtureKind::FocusField => (
            resolve_direct_focus_field_command(
                fixture.transcript,
                fixture.name,
                fixture.page.as_ref(),
                &fixture
                    .active_skills
                    .iter()
                    .map(|skill| String::from(*skill))
                    .collect::<Vec<_>>(),
                fixture.confirmation_threshold,
            )
            .unwrap_or_else(|| panic!("fixture {} should resolve", fixture.name)),
            None,
        ),
        AppCorePlannerFixtureKind::FillField => (
            resolve_direct_fill_field_command(
                fixture.transcript,
                fixture.name,
                fixture.page.as_ref(),
                &fixture
                    .active_skills
                    .iter()
                    .map(|skill| String::from(*skill))
                    .collect::<Vec<_>>(),
                fixture.confirmation_threshold,
            )
            .unwrap_or_else(|| panic!("fixture {} should resolve", fixture.name)),
            None,
        ),
        AppCorePlannerFixtureKind::FillAndSubmit => (
            resolve_direct_fill_and_submit_command(
                fixture.transcript,
                fixture.name,
                fixture.page.as_ref(),
                &fixture
                    .active_skills
                    .iter()
                    .map(|skill| String::from(*skill))
                    .collect::<Vec<_>>(),
                fixture.confirmation_threshold,
            )
            .unwrap_or_else(|| panic!("fixture {} should resolve", fixture.name)),
            None,
        ),
        AppCorePlannerFixtureKind::FollowUpCorrection => {
            resolve_recent_fill_correction_command(
                fixture.transcript,
                fixture.name,
                fixture.current_page_id,
                fixture.page.as_ref(),
                &fixture
                    .active_skills
                    .iter()
                    .map(|skill| String::from(*skill))
                    .collect::<Vec<_>>(),
                fixture.recent_context.as_ref(),
            )
            .unwrap_or_else(|| panic!("fixture {} should resolve", fixture.name))
        }
        AppCorePlannerFixtureKind::SubmitForm => (
            resolve_direct_submit_form_command(
                fixture.transcript,
                fixture.name,
                fixture.page.as_ref(),
                &fixture
                    .active_skills
                    .iter()
                    .map(|skill| String::from(*skill))
                    .collect::<Vec<_>>(),
            )
            .unwrap_or_else(|| panic!("fixture {} should resolve", fixture.name)),
            None,
        ),
    }
}

fn assert_app_core_planner_fixture(fixture: AppCorePlannerFixture) {
    let (planner_output, next_context) = resolve_app_core_planner_fixture(&fixture);
    let expected_selected_skills = fixture
        .expected_selected_skills
        .iter()
        .map(|skill| String::from(*skill))
        .collect::<Vec<_>>();

    assert_eq!(
        planner_output.intent.name, fixture.expected_intent,
        "fixture {} resolved unexpected intent",
        fixture.name
    );
    assert_eq!(
        planner_output.status, fixture.expected_status,
        "fixture {} resolved unexpected planner status",
        fixture.name
    );
    assert_eq!(
        planner_output.selected_skills, expected_selected_skills,
        "fixture {} selected unexpected skills",
        fixture.name
    );
    assert_eq!(
        planner_tool_sequence(&planner_output),
        fixture.expected_tool_sequence,
        "fixture {} produced unexpected tool sequence",
        fixture.name
    );

    if let Some(expected_focus_element_id) = fixture.expected_focus_element_id {
        let focus_step = planner_output
            .steps
            .iter()
            .find(|step| step.tool_name == ToolName::FocusElement)
            .unwrap_or_else(|| panic!("fixture {} should include a focus step", fixture.name));
        assert_eq!(
            focus_step.arguments.get("element_id"),
            Some(&serde_json::json!(expected_focus_element_id)),
            "fixture {} focused the wrong element",
            fixture.name
        );
    }

    if let Some(expected_typed_text) = fixture.expected_typed_text {
        let type_step = planner_output
            .steps
            .iter()
            .find(|step| step.tool_name == ToolName::TypeIntoElement)
            .unwrap_or_else(|| panic!("fixture {} should include a type step", fixture.name));
        assert_eq!(
            type_step.arguments.get("text"),
            Some(&serde_json::json!(expected_typed_text)),
            "fixture {} typed unexpected text",
            fixture.name
        );
    }

    assert_eq!(
        next_context
            .as_ref()
            .and_then(|context| context.active_element_id.as_deref()),
        fixture.expected_next_active_element_id,
        "fixture {} produced unexpected next active element",
        fixture.name
    );
    assert_eq!(
        next_context
            .as_ref()
            .and_then(|context| context.pending_text.as_deref()),
        fixture.expected_next_pending_text,
        "fixture {} produced unexpected next pending text",
        fixture.name
    );
}
#[test]
fn planner_interpretation_unavailable_error_wraps_reason_for_voice_feedback() {
    let error = planner_interpretation_unavailable_error(
        "planner_profile_unavailable",
        "remote planner mode requires a configured planner profile",
        false,
        None,
    );

    assert_eq!(error.code, "planner_profile_unavailable");
    assert_eq!(
        error.message,
        "Command interpretation is unavailable because remote planner mode requires a configured planner profile."
    );
    assert!(!error.retryable);
    assert_eq!(error.details, None);
}

#[test]
fn build_remote_planner_settings_reflects_configured_profile_details() {
    let config = AppConfig::default();

    let settings = build_remote_planner_settings(&config);

    assert_eq!(settings.profile_name.as_deref(), Some("openai-default"));
    assert_eq!(
        settings.provider,
        Some(crate::commands::RemoteProviderLabel::OpenAi)
    );
    assert_eq!(
        settings.base_url.as_deref(),
        Some("https://api.openai.com/v1")
    );
    assert_eq!(settings.model.as_deref(), Some("gpt-5.4-mini"));
    assert_eq!(
        settings.api_key_reference.as_deref(),
        Some("Environment variable: OPENAI_API_KEY")
    );
    assert_eq!(settings.organization_reference, None);
    assert_eq!(settings.project, None);
    assert_eq!(settings.temperature_milli, Some(200));
    assert_eq!(settings.max_output_tokens, Some(1024));
    assert_eq!(settings.timeout_ms, Some(30_000));
}

#[test]
fn build_remote_planner_settings_reflects_selected_ollama_profile_details() {
    let mut config = AppConfig::default();
    config.providers.planner.remote_profile = Some(String::from("ollama-default"));

    let settings = build_remote_planner_settings(&config);

    assert_eq!(settings.profile_name.as_deref(), Some("ollama-default"));
    assert_eq!(
        settings.provider,
        Some(crate::commands::RemoteProviderLabel::Ollama)
    );
    assert_eq!(
        settings.base_url.as_deref(),
        Some("http://localhost:11434/v1")
    );
    assert_eq!(settings.model.as_deref(), Some("qwen2.5:3b-instruct"));
    assert_eq!(
        settings.api_key_reference.as_deref(),
        Some("Environment variable: OLLAMA_API_KEY")
    );
    assert_eq!(settings.organization_reference, None);
    assert_eq!(settings.project, None);
    assert_eq!(settings.temperature_milli, Some(200));
    assert_eq!(settings.max_output_tokens, Some(1024));
    assert_eq!(settings.timeout_ms, Some(30_000));
}

#[test]
fn build_remote_tts_settings_reflects_configured_profile_details() {
    let config = AppConfig::default();

    let settings = build_remote_tts_settings(&config);

    assert_eq!(settings.profile_name.as_deref(), Some("openai-tts-default"));
    assert_eq!(
        settings.provider,
        Some(crate::commands::RemoteProviderLabel::OpenAi)
    );
    assert_eq!(
        settings.base_url.as_deref(),
        Some("https://api.openai.com/v1")
    );
    assert_eq!(settings.model.as_deref(), Some("gpt-4o-mini-tts"));
    assert_eq!(
        settings.api_key_reference.as_deref(),
        Some("Environment variable: OPENAI_API_KEY")
    );
    if let Some(masked_value) = settings.api_key_masked_value.as_deref() {
        assert!(masked_value.starts_with("***"));
    }
    assert_eq!(settings.organization_reference, None);
    assert_eq!(settings.project, None);
    assert_eq!(settings.voice.as_deref(), Some("alloy"));
    assert_eq!(
        settings.audio_format,
        Some(crate::config::RemoteTtsAudioFormat::Wav)
    );
    assert_eq!(settings.timeout_ms, Some(30_000));
}

#[test]
fn build_remote_asr_settings_reflects_configured_profile_details() {
    let config = AppConfig::default();

    let settings = build_remote_asr_settings(&config);

    assert_eq!(
        settings.profile_name.as_deref(),
        Some("openai-transcribe-default")
    );
    assert_eq!(
        settings.provider,
        Some(crate::commands::RemoteProviderLabel::OpenAi)
    );
    assert_eq!(
        settings.base_url.as_deref(),
        Some("https://api.openai.com/v1")
    );
    assert_eq!(settings.model.as_deref(), Some("gpt-4o-mini-transcribe"));
    assert_eq!(
        settings.api_key_reference.as_deref(),
        Some("Environment variable: OPENAI_API_KEY")
    );
    if let Some(masked_value) = settings.api_key_masked_value.as_deref() {
        assert!(masked_value.starts_with("***"));
    }
    assert_eq!(settings.organization_reference, None);
    assert_eq!(settings.project, None);
    assert_eq!(settings.language.as_deref(), Some("en"));
    assert_eq!(settings.temperature_milli, Some(0));
    assert_eq!(settings.timeout_ms, Some(30_000));
}

#[test]
fn build_remote_settings_expose_secret_references_without_raw_values() {
    let mut config = AppConfig::default();
    let planner_profile = config
        .remote_planner_profiles
        .get_mut("openai-default")
        .expect("planner profile should exist");
    planner_profile.api_key = SecretRef::FromFile {
        from_file: String::from("/secure/planner.key"),
    };
    planner_profile.organization = Some(SecretRef::FromKeyring {
        from_keyring: KeyringRef {
            service: String::from("blind-browser"),
            account: String::from("planner/openai-default"),
        },
    });

    let settings = build_remote_planner_settings(&config);

    assert_eq!(
        settings.api_key_reference.as_deref(),
        Some("File reference: /secure/planner.key")
    );
    assert_eq!(
        settings.organization_reference.as_deref(),
        Some("OS keyring entry: blind-browser / planner/openai-default")
    );
    assert!(!settings
        .api_key_reference
        .as_deref()
        .unwrap_or_default()
        .contains("super-secret"));
    assert!(!settings
        .organization_reference
        .as_deref()
        .unwrap_or_default()
        .contains("super-secret"));
}

#[test]
fn build_provider_failover_settings_reports_unavailable_runtime_support() {
    let config = AppConfig::default();

    let settings = build_provider_failover_settings(&config);

    assert!(!settings.planner_available);
    assert!(!settings.tts_available);
    assert!(!settings.asr_available);
    assert_eq!(
        settings.summary,
        String::from(
            "Provider failover settings are defined in config, but automatic failover is still disabled in the live runtime."
        )
    );
}

#[test]
fn build_confirmation_settings_reflects_configured_safety_values() {
    let config = AppConfig::default();

    let settings = build_confirmation_settings(&config);

    assert_eq!(settings.confirmation_confidence_threshold, 0.9);
    assert!(settings.allow_click_without_confirmation);
    assert!(settings.always_confirm_submit);
}

#[test]
fn build_local_tts_model_settings_reflects_configured_profile_details() {
    let config = AppConfig::default();

    let settings = build_local_tts_model_settings(&config);

    assert_eq!(settings.profile_name.as_deref(), Some("kitten-default"));
    assert_eq!(
        settings.backend,
        Some(crate::config::LocalTtsBackend::KittenTtsRs)
    );
    assert_eq!(settings.model_id.as_deref(), Some("default"));
    assert_eq!(
        settings.model_path.as_deref(),
        Some("/path/to/kitten/model")
    );
    assert_eq!(settings.default_voice.as_deref(), Some("Bruno"));
    assert_eq!(settings.sample_rate, Some(24_000));
}

#[test]
fn build_tts_model_settings_uses_selected_local_profile() {
    let mut config = AppConfig::default();
    config.providers.tts.mode = ProviderMode::Local;
    config.local_tts_profiles.insert(
        String::from("kitten-alt"),
        crate::config::LocalTtsProfile {
            backend: crate::config::LocalTtsBackend::KittenTtsRs,
            model_id: String::from("expressive"),
            model_path: String::from("/path/to/kitten/expressive"),
            default_voice: String::from("Bella"),
            sample_rate: 22_050,
        },
    );
    config.providers.tts.local_profile = Some(String::from("kitten-alt"));

    let settings = build_tts_model_settings(&config);

    assert_eq!(settings.mode, ProviderMode::Local);
    assert_eq!(settings.active_profile.as_deref(), Some("kitten-alt"));
    assert!(settings
        .available_profiles
        .iter()
        .any(
            |option| option.profile_name == "kitten-default" && option.model_label == "default"
        ));
    assert!(settings
        .available_profiles
        .iter()
        .any(
            |option| option.profile_name == "kitten-alt" && option.model_label == "expressive"
        ));
}

#[test]
fn build_local_tts_model_settings_reflects_selected_profile_details() {
    let mut config = AppConfig::default();
    config.local_tts_profiles.insert(
        String::from("kitten-alt"),
        crate::config::LocalTtsProfile {
            backend: crate::config::LocalTtsBackend::KittenTtsRs,
            model_id: String::from("expressive"),
            model_path: String::from("/path/to/kitten/expressive"),
            default_voice: String::from("Bella"),
            sample_rate: 22_050,
        },
    );
    config.providers.tts.local_profile = Some(String::from("kitten-alt"));

    let settings = build_local_tts_model_settings(&config);

    assert_eq!(settings.profile_name.as_deref(), Some("kitten-alt"));
    assert_eq!(
        settings.backend,
        Some(crate::config::LocalTtsBackend::KittenTtsRs)
    );
    assert_eq!(settings.model_id.as_deref(), Some("expressive"));
    assert_eq!(
        settings.model_path.as_deref(),
        Some("/path/to/kitten/expressive")
    );
    assert_eq!(settings.default_voice.as_deref(), Some("Bella"));
    assert_eq!(settings.sample_rate, Some(22_050));
}

#[test]
fn build_local_asr_model_settings_reflects_configured_profile_details() {
    let config = AppConfig::default();

    let settings = build_local_asr_model_settings(&config);

    assert_eq!(settings.profile_name.as_deref(), Some("whisper-default"));
    assert_eq!(
        settings.backend,
        Some(crate::config::LocalAsrBackend::Whisper)
    );
    assert_eq!(settings.model_id.as_deref(), Some("tiny"));
    assert_eq!(
        settings.model_path.as_deref(),
        Some("/path/to/whisper/model")
    );
    assert_eq!(settings.language.as_deref(), Some("en"));
    assert_eq!(settings.threads, Some(4));
}

#[test]
fn build_local_asr_model_settings_reflects_selected_profile_details() {
    let mut config = AppConfig::default();
    config.local_asr_profiles.insert(
        String::from("whisper-alt"),
        crate::config::LocalAsrProfile {
            backend: crate::config::LocalAsrBackend::Whisper,
            model_id: String::from("base"),
            model_path: String::from("/path/to/whisper/base"),
            language: Some(String::from("fr")),
            threads: 6,
        },
    );
    config.providers.asr.local_profile = Some(String::from("whisper-alt"));

    let settings = build_local_asr_model_settings(&config);

    assert_eq!(settings.profile_name.as_deref(), Some("whisper-alt"));
    assert_eq!(
        settings.backend,
        Some(crate::config::LocalAsrBackend::Whisper)
    );
    assert_eq!(settings.model_id.as_deref(), Some("base"));
    assert_eq!(
        settings.model_path.as_deref(),
        Some("/path/to/whisper/base")
    );
    assert_eq!(settings.language.as_deref(), Some("fr"));
    assert_eq!(settings.threads, Some(6));
}

#[test]
fn build_ocr_threshold_settings_reflects_configured_ocr_values() {
    let config = AppConfig::default();

    let settings = build_ocr_threshold_settings(&config);

    assert_eq!(settings.sparse_text_char_threshold, 200);
    assert_eq!(settings.sparse_text_region_threshold, 2);
}

#[test]
fn build_asr_provider_settings_returns_available_modes() {
    let config = AppConfig::default();

    let settings = build_asr_provider_settings(&config);

    assert_eq!(settings.active_mode, ProviderMode::Remote);
    assert_eq!(
        settings.available_modes,
        vec![ProviderMode::Local, ProviderMode::Remote]
    );
}

#[test]
fn build_tts_provider_settings_returns_available_modes() {
    let config = AppConfig::default();

    let settings = build_tts_provider_settings(&config);

    assert_eq!(settings.active_mode, ProviderMode::Remote);
    assert_eq!(
        settings.available_modes,
        vec![ProviderMode::Local, ProviderMode::Remote]
    );
}

#[test]
fn build_tts_voice_settings_returns_kitten_voice_choices_for_local_mode() {
    let mut config = AppConfig::default();
    config.providers.tts.mode = ProviderMode::Local;
    let runtime_audio = RuntimeAudioState::from(&config.audio);

    let settings = build_tts_voice_settings(&config, &runtime_audio);

    assert_eq!(settings.mode, ProviderMode::Local);
    assert_eq!(settings.active_voice.as_deref(), Some("Bruno"));
    assert_eq!(settings.available_voices.len(), 8);
    assert!(settings
        .available_voices
        .iter()
        .any(|option| option.voice_name == "Bella"));
    assert!(settings
        .available_voices
        .iter()
        .any(|option| option.voice_name == "Leo"));
}

#[test]
fn build_tts_voice_settings_preserves_custom_active_voice() {
    let config = AppConfig::default();
    let runtime_audio = RuntimeAudioState {
        tts_voice: Some(String::from("CustomVoice")),
        ..RuntimeAudioState::from(&config.audio)
    };

    let settings = build_tts_voice_settings(&config, &runtime_audio);

    assert_eq!(settings.active_voice.as_deref(), Some("CustomVoice"));
    assert_eq!(settings.available_voices[0].voice_name, "CustomVoice");
}

#[test]
fn build_tts_voice_settings_returns_openai_builtin_voices_for_remote_mode() {
    let mut config = AppConfig::default();
    config.providers.tts.mode = ProviderMode::Remote;
    let runtime_audio = RuntimeAudioState {
        tts_voice: Some(String::from("Alloy")),
        ..RuntimeAudioState::from(&config.audio)
    };

    let settings = build_tts_voice_settings(&config, &runtime_audio);

    assert_eq!(settings.mode, ProviderMode::Remote);
    assert_eq!(settings.active_voice.as_deref(), Some("alloy"));
    assert!(settings
        .available_voices
        .iter()
        .any(|option| option.voice_name == "cedar"));
}

#[test]
fn normalize_optional_text_trims_and_drops_empty_values() {
    assert_eq!(normalize_optional_text(None), None);
    assert_eq!(normalize_optional_text(Some(String::from("   "))), None);
    assert_eq!(
        normalize_optional_text(Some(String::from("  next step  "))),
        Some(String::from("next step"))
    );
}

#[test]
fn normalize_absolute_url_accepts_trimmed_absolute_urls() {
    assert_eq!(
        normalize_absolute_url("  https://example.com/page  ").unwrap(),
        String::from("https://example.com/page")
    );
    assert_eq!(
        normalize_absolute_url("about:blank").unwrap(),
        String::from("about:blank")
    );
}

#[test]
fn normalize_absolute_url_rejects_relative_urls() {
    let error = normalize_absolute_url("/relative/path").unwrap_err();
    assert_eq!(error.code, "invalid_url");
}

#[test]
fn browser_error_to_tool_error_keeps_navigation_failures_retryable_and_structured() {
    let navigate_error = browser_error_to_tool_error(
        String::from("open_url failed to navigate the active page"),
        BrowserError::Navigate(String::from("dns resolution failed")),
    );
    assert_eq!(navigate_error.code, "browser_navigation_failed");
    assert!(navigate_error.retryable);
    assert_eq!(
        navigate_error.details,
        Some(serde_json::json!({
            "reason": "failed to navigate browser page: dns resolution failed"
        }))
    );

    let history_error = browser_error_to_tool_error(
        String::from("go_back failed to update the current page"),
        BrowserError::History(String::from("no previous entry")),
    );
    assert_eq!(history_error.code, "browser_history_failed");
    assert!(history_error.retryable);
    assert_eq!(
        history_error.details,
        Some(serde_json::json!({
            "reason": "failed to read browser navigation history: no previous entry"
        }))
    );
}

#[test]
fn refresh_current_page_after_navigation_replaces_metadata_and_clears_stale_content() {
    let mut current_page = Some(PageModel {
        title: Some(String::from("Old page")),
        url: Some(String::from("https://example.com/old")),
        regions: vec![PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Paragraph,
            label: None,
            text: String::from("Stale extracted text"),
            bbox: None,
            source: RegionSource::Dom,
        }],
        interactive_elements: vec![InteractiveElement {
            element_id: String::from("button-1"),
            dom_locator: Some(String::from("#old-button")),
            role: ElementRole::Button,
            tag_name: String::from("button"),
            text: Some(String::from("Continue")),
            accessible_name: Some(String::from("Continue")),
            placeholder: None,
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::new(),
        }],
    });

    refresh_current_page_after_navigation(
        &mut current_page,
        Some(String::from("https://example.com/new")),
        Some(String::from("New page")),
    );

    let current_page = current_page.expect("page should still exist");
    assert_eq!(current_page.url.as_deref(), Some("https://example.com/new"));
    assert_eq!(current_page.title.as_deref(), Some("New page"));
    assert!(current_page.regions.is_empty());
    assert!(current_page.interactive_elements.is_empty());
}

#[test]
fn clear_navigation_follow_up_state_resets_cursor_and_recent_field_context() {
    let mut state = AppState::default();
    state.narration_cursor.current_index = Some(3);
    state.narration_cursor.current_region_id = Some(String::from("region-3"));
    state.narration_cursor.total_regions = 8;

    let mut recent_field_context = Some(RecentFieldContext {
        page_id: String::from("page-1"),
        target_description: Some(String::from("email field")),
        active_element_id: Some(String::from("input-email")),
        candidate_element_ids: vec![String::from("input-email"), String::from("input-alt")],
        pending_text: Some(String::from("user@example.com")),
        submit_after: true,
    });

    clear_navigation_follow_up_state(&mut state, &mut recent_field_context);

    assert_eq!(state.narration_cursor, Default::default());
    assert_eq!(recent_field_context, None);
}

#[test]
fn build_visible_text_excerpt_joins_regions_and_applies_limit() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![
            PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Other,
                label: None,
                text: String::from("First paragraph"),
                bbox: None,
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("region-2"),
                role: RegionRole::Other,
                label: None,
                text: String::from("Second paragraph"),
                bbox: None,
                source: RegionSource::Dom,
            },
        ],
        interactive_elements: Vec::new(),
    };

    assert_eq!(
        build_visible_text_excerpt(&page, None),
        String::from("First paragraph\n\nSecond paragraph")
    );
    assert_eq!(
        build_visible_text_excerpt(&page, Some(5)),
        String::from("First")
    );
}

#[test]
fn region_bbox_by_id_returns_region_geometry_when_available() {
    let regions = vec![PageRegion {
        region_id: String::from("region-1"),
        role: RegionRole::Section,
        label: Some(String::from("Main")),
        text: String::from("Text"),
        bbox: Some(Rect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        }),
        source: RegionSource::Dom,
    }];

    assert_eq!(
        region_bbox_by_id(&regions, "region-1").expect("region bbox should resolve"),
        Rect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        }
    );
}

#[test]
fn build_extracted_page_model_can_omit_links() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: Vec::new(),
        interactive_elements: vec![
            InteractiveElement {
                element_id: String::from("link-1"),
                dom_locator: Some(String::from("#link-1")),
                role: ElementRole::Link,
                tag_name: String::from("a"),
                text: Some(String::from("Read more")),
                accessible_name: Some(String::from("Read more")),
                placeholder: None,
                href: Some(String::from("https://example.com/more")),
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
            InteractiveElement {
                element_id: String::from("button-1"),
                dom_locator: Some(String::from("#button-1")),
                role: ElementRole::Button,
                tag_name: String::from("button"),
                text: Some(String::from("Continue")),
                accessible_name: Some(String::from("Continue")),
                placeholder: None,
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
        ],
    };
    let input = ExtractPageModelInput {
        request_id: String::from("req-extract"),
        timeout_ms: None,
        use_dom_extraction: true,
        include_headings: true,
        include_links: false,
    };

    let extracted = build_extracted_page_model(&page, &input);

    assert_eq!(extracted.interactive_elements.len(), 1);
    assert_eq!(extracted.interactive_elements[0].role, ElementRole::Button);
}

#[test]
fn build_extracted_page_model_preserves_link_metadata_when_requested() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: Vec::new(),
        interactive_elements: vec![InteractiveElement {
            element_id: String::from("link-1"),
            dom_locator: Some(String::from("#link-1")),
            role: ElementRole::Link,
            tag_name: String::from("a"),
            text: Some(String::from("Read more")),
            accessible_name: Some(String::from("Read more about examples")),
            placeholder: None,
            href: Some(String::from("https://example.com/more")),
            value: None,
            bbox: Some(Rect {
                x: 10.0,
                y: 20.0,
                width: 30.0,
                height: 12.0,
            }),
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::from([(
                String::from("rel"),
                String::from("noopener"),
            )]),
        }],
    };
    let input = ExtractPageModelInput {
        request_id: String::from("req-extract"),
        timeout_ms: None,
        use_dom_extraction: true,
        include_headings: true,
        include_links: true,
    };

    let extracted = build_extracted_page_model(&page, &input);

    assert_eq!(extracted.interactive_elements.len(), 1);
    let link = &extracted.interactive_elements[0];
    assert_eq!(link.role, ElementRole::Link);
    assert_eq!(link.href.as_deref(), Some("https://example.com/more"));
    assert_eq!(link.text.as_deref(), Some("Read more"));
    assert_eq!(
        link.accessible_name.as_deref(),
        Some("Read more about examples")
    );
    assert_eq!(
        link.bbox,
        Some(Rect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 12.0,
        })
    );
    assert_eq!(
        link.attributes.get("rel").map(String::as_str),
        Some("noopener")
    );
}

#[test]
fn build_extracted_page_model_preserves_region_order_and_sources() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![
            PageRegion {
                region_id: String::from("dom-region-title"),
                role: RegionRole::Title,
                label: Some(String::from("Title")),
                text: String::from("Example"),
                bbox: None,
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("dom-region-1"),
                role: RegionRole::Other,
                label: None,
                text: String::from("First paragraph."),
                bbox: None,
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("ocr-region-1"),
                role: RegionRole::Other,
                label: None,
                text: String::from("Recovered OCR text."),
                bbox: None,
                source: RegionSource::Ocr,
            },
        ],
        interactive_elements: Vec::new(),
    };
    let input = ExtractPageModelInput {
        request_id: String::from("req-extract"),
        timeout_ms: None,
        use_dom_extraction: true,
        include_headings: true,
        include_links: true,
    };

    let extracted = build_extracted_page_model(&page, &input);

    let ordered_region_ids = extracted
        .regions
        .iter()
        .map(|region| region.region_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        ordered_region_ids,
        vec!["dom-region-title", "dom-region-1", "ocr-region-1"]
    );
    assert_eq!(
        extracted
            .regions
            .iter()
            .map(|region| region.source.clone())
            .collect::<Vec<_>>(),
        vec![RegionSource::Dom, RegionSource::Dom, RegionSource::Ocr]
    );
}

#[test]
fn build_extracted_page_model_leaves_heading_regions_unchanged_when_disabled() {
    let page = PageModel {
        title: Some(String::from("Example article")),
        url: Some(String::from("https://example.com/article")),
        regions: vec![
            PageRegion {
                region_id: String::from("region-title"),
                role: RegionRole::Title,
                label: Some(String::from("Title")),
                text: String::from("Example article"),
                bbox: None,
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("region-heading"),
                role: RegionRole::Heading,
                label: Some(String::from("Heading")),
                text: String::from("Section one"),
                bbox: None,
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("region-paragraph"),
                role: RegionRole::Paragraph,
                label: None,
                text: String::from("First paragraph."),
                bbox: None,
                source: RegionSource::Dom,
            },
        ],
        interactive_elements: Vec::new(),
    };
    let input = ExtractPageModelInput {
        request_id: String::from("req-extract"),
        timeout_ms: None,
        use_dom_extraction: false,
        include_headings: false,
        include_links: true,
    };

    let extracted = build_extracted_page_model(&page, &input);

    assert_eq!(extracted.title, page.title);
    assert_eq!(extracted.url, page.url);
    assert_eq!(extracted.regions, page.regions);
}

#[test]
fn infer_extraction_source_detects_merged_models() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![
            PageRegion {
                region_id: String::from("dom-region"),
                role: RegionRole::Other,
                label: None,
                text: String::from("DOM text"),
                bbox: None,
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("ocr-region"),
                role: RegionRole::Other,
                label: None,
                text: String::from("OCR text"),
                bbox: None,
                source: RegionSource::Ocr,
            },
        ],
        interactive_elements: Vec::new(),
    };

    assert_eq!(
        infer_extraction_source(&page, true, false),
        ExtractionSource::Merged
    );
}

#[test]
fn infer_extraction_source_treats_mixed_regions_as_merged() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![PageRegion {
            region_id: String::from("mixed-region"),
            role: RegionRole::Other,
            label: None,
            text: String::from("DOM text\n\nOCR text"),
            bbox: None,
            source: RegionSource::Mixed,
        }],
        interactive_elements: Vec::new(),
    };

    assert_eq!(
        infer_extraction_source(&page, true, false),
        ExtractionSource::Merged
    );
}

#[test]
fn infer_extraction_source_reports_dom_smoothie_when_dom_only() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![PageRegion {
            region_id: String::from("dom-region"),
            role: RegionRole::Other,
            label: None,
            text: String::from("Readable text"),
            bbox: None,
            source: RegionSource::Dom,
        }],
        interactive_elements: Vec::new(),
    };

    assert_eq!(
        infer_extraction_source(&page, true, true),
        ExtractionSource::DomSmoothie
    );
    assert_eq!(
        infer_extraction_source(&page, true, false),
        ExtractionSource::DomFallback
    );
}

#[test]
fn should_trigger_no_extractable_text_ocr_fallback_when_dom_regions_are_empty() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Other,
            label: None,
            text: String::from("   "),
            bbox: None,
            source: RegionSource::Dom,
        }],
        interactive_elements: Vec::new(),
    };

    assert!(should_trigger_extract_page_model_ocr_fallback(
        true,
        &page,
        &OcrSettings::default()
    ));
}

#[test]
fn extracted_text_metrics_counts_trimmed_text_and_regions() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![
            PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Other,
                label: None,
                text: String::from("  Visible DOM text  "),
                bbox: None,
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("region-2"),
                role: RegionRole::Other,
                label: None,
                text: String::from(" "),
                bbox: None,
                source: RegionSource::Dom,
            },
        ],
        interactive_elements: Vec::new(),
    };

    assert_eq!(extracted_text_metrics(&page), (16, 1));
}

#[test]
fn should_trigger_extract_page_model_ocr_fallback_when_text_is_below_char_threshold() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Other,
            label: None,
            text: String::from("Short text"),
            bbox: None,
            source: RegionSource::Dom,
        }],
        interactive_elements: Vec::new(),
    };
    let settings = OcrSettings {
        sparse_text_char_threshold: 20,
        sparse_text_region_threshold: 1,
        ..OcrSettings::default()
    };

    assert!(should_trigger_extract_page_model_ocr_fallback(
        true, &page, &settings
    ));
}

#[test]
fn should_trigger_extract_page_model_ocr_fallback_when_region_count_is_below_threshold() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Other,
            label: None,
            text: String::from("This region has enough text to pass the char threshold alone."),
            bbox: None,
            source: RegionSource::Dom,
        }],
        interactive_elements: Vec::new(),
    };
    let settings = OcrSettings {
        sparse_text_char_threshold: 10,
        sparse_text_region_threshold: 2,
        ..OcrSettings::default()
    };

    assert!(should_trigger_extract_page_model_ocr_fallback(
        true, &page, &settings
    ));
}

#[test]
fn should_trigger_extract_page_model_ocr_fallback_at_default_char_boundary() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![
            PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Other,
                label: None,
                text: "a".repeat(100),
                bbox: None,
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("region-2"),
                role: RegionRole::Other,
                label: None,
                text: "b".repeat(100),
                bbox: None,
                source: RegionSource::Dom,
            },
        ],
        interactive_elements: Vec::new(),
    };

    assert!(should_trigger_extract_page_model_ocr_fallback(
        true,
        &page,
        &OcrSettings::default()
    ));
}

#[test]
fn should_trigger_extract_page_model_ocr_fallback_at_default_region_boundary() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Other,
            label: None,
            text: "a".repeat(201),
            bbox: None,
            source: RegionSource::Dom,
        }],
        interactive_elements: Vec::new(),
    };

    assert!(should_trigger_extract_page_model_ocr_fallback(
        true,
        &page,
        &OcrSettings::default()
    ));
}

#[test]
fn should_not_trigger_extract_page_model_ocr_fallback_above_default_boundaries() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![
            PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Other,
                label: None,
                text: "a".repeat(101),
                bbox: None,
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("region-2"),
                role: RegionRole::Other,
                label: None,
                text: "b".repeat(100),
                bbox: None,
                source: RegionSource::Dom,
            },
        ],
        interactive_elements: Vec::new(),
    };

    assert!(!should_trigger_extract_page_model_ocr_fallback(
        true,
        &page,
        &OcrSettings::default()
    ));
}

#[test]
fn should_not_trigger_extract_page_model_ocr_fallback_when_thresholds_are_satisfied() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![
            PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Other,
                label: None,
                text: String::from(
                    "This first region contains comfortably more than twenty characters.",
                ),
                bbox: None,
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("region-2"),
                role: RegionRole::Other,
                label: None,
                text: String::from("This second region also contains enough text."),
                bbox: None,
                source: RegionSource::Dom,
            },
        ],
        interactive_elements: Vec::new(),
    };
    let settings = OcrSettings {
        sparse_text_char_threshold: 20,
        sparse_text_region_threshold: 2,
        ..OcrSettings::default()
    };

    assert!(!should_trigger_extract_page_model_ocr_fallback(
        true, &page, &settings
    ));
}

#[test]
fn should_not_trigger_extract_page_model_ocr_fallback_when_disabled_or_non_dom() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Other,
            label: None,
            text: String::new(),
            bbox: None,
            source: RegionSource::Dom,
        }],
        interactive_elements: Vec::new(),
    };
    let disabled_settings = OcrSettings {
        trigger_on_no_extractable_text: false,
        ..OcrSettings::default()
    };

    assert!(!should_trigger_extract_page_model_ocr_fallback(
        true,
        &page,
        &disabled_settings
    ));
    assert!(!should_trigger_extract_page_model_ocr_fallback(
        false,
        &page,
        &OcrSettings::default()
    ));
}

#[test]
fn region_first_ocr_target_ids_prefers_bbox_backed_readable_regions() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![
            PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Other,
                label: None,
                text: String::from("Readable text"),
                bbox: Some(Rect {
                    x: 1.0,
                    y: 2.0,
                    width: 30.0,
                    height: 40.0,
                }),
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("region-2"),
                role: RegionRole::Other,
                label: None,
                text: String::from("Readable but no bbox"),
                bbox: None,
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("region-3"),
                role: RegionRole::Other,
                label: None,
                text: String::from(""),
                bbox: Some(Rect {
                    x: 5.0,
                    y: 6.0,
                    width: 50.0,
                    height: 60.0,
                }),
                source: RegionSource::Dom,
            },
            PageRegion {
                region_id: String::from("region-4"),
                role: RegionRole::Other,
                label: None,
                text: String::from("Readable but invalid bbox"),
                bbox: Some(Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 0.0,
                    height: 10.0,
                }),
                source: RegionSource::Dom,
            },
        ],
        interactive_elements: Vec::new(),
    };

    assert_eq!(
        region_first_ocr_target_ids(&page, &OcrSettings::default()),
        vec![String::from("region-1")]
    );
}

#[test]
fn region_first_ocr_target_ids_respects_preference_toggle() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Other,
            label: None,
            text: String::from("Readable text"),
            bbox: Some(Rect {
                x: 1.0,
                y: 2.0,
                width: 30.0,
                height: 40.0,
            }),
            source: RegionSource::Dom,
        }],
        interactive_elements: Vec::new(),
    };
    let settings = OcrSettings {
        prefer_region_ocr: false,
        ..OcrSettings::default()
    };

    assert!(region_first_ocr_target_ids(&page, &settings).is_empty());
}

#[test]
fn merged_region_text_prefers_more_complete_or_combined_text() {
    assert_eq!(
        merged_region_text("Short label", "Short label with extra detail"),
        String::from("Short label with extra detail")
    );
    assert_eq!(
        merged_region_text("DOM text", "OCR text"),
        String::from("DOM text\n\nOCR text")
    );
}

#[test]
fn merge_ocr_text_into_page_model_updates_existing_region_as_mixed_and_adopts_bbox() {
    let mut page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Section,
            label: Some(String::from("Main")),
            text: String::from("DOM summary"),
            bbox: None,
            source: RegionSource::Dom,
        }],
        interactive_elements: Vec::new(),
    };

    let updated_region_ids = merge_ocr_text_into_page_model(
        &mut page,
        Some("region-1"),
        "OCR detail",
        Some(Rect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        }),
        String::from("unused"),
    )
    .expect("merge should update the requested region");

    assert_eq!(updated_region_ids, vec![String::from("region-1")]);
    assert_eq!(page.regions[0].source, RegionSource::Mixed);
    assert_eq!(
        page.regions[0].text,
        String::from("DOM summary\n\nOCR detail")
    );
    assert_eq!(
        page.regions[0].bbox,
        Some(Rect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        })
    );
}

#[test]
fn merge_ocr_text_into_page_model_preserves_existing_region_bbox() {
    let mut page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Section,
            label: Some(String::from("Main")),
            text: String::from("DOM summary"),
            bbox: Some(Rect {
                x: 1.0,
                y: 2.0,
                width: 3.0,
                height: 4.0,
            }),
            source: RegionSource::Dom,
        }],
        interactive_elements: Vec::new(),
    };

    let updated_region_ids = merge_ocr_text_into_page_model(
        &mut page,
        Some("region-1"),
        "OCR detail",
        Some(Rect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        }),
        String::from("unused"),
    )
    .expect("merge should update the requested region");

    assert_eq!(updated_region_ids, vec![String::from("region-1")]);
    assert_eq!(
        page.regions[0].bbox,
        Some(Rect {
            x: 1.0,
            y: 2.0,
            width: 3.0,
            height: 4.0,
        })
    );
}

#[test]
fn merge_ocr_text_into_page_model_appends_new_ocr_region_when_target_missing() {
    let mut page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: Vec::new(),
        interactive_elements: Vec::new(),
    };

    let updated_region_ids = merge_ocr_text_into_page_model(
        &mut page,
        None,
        "Recovered OCR text",
        Some(Rect {
            x: 5.0,
            y: 6.0,
            width: 70.0,
            height: 80.0,
        }),
        String::from("ocr-region-generated"),
    )
    .expect("merge should create a new OCR region when no target region_id is supplied");

    assert_eq!(
        updated_region_ids,
        vec![String::from("ocr-region-generated")]
    );
    assert_eq!(page.regions.len(), 1);
    assert_eq!(page.regions[0].region_id, "ocr-region-generated");
    assert_eq!(page.regions[0].source, RegionSource::Ocr);
    assert_eq!(page.regions[0].text, "Recovered OCR text");
    assert_eq!(
        page.regions[0].bbox,
        Some(Rect {
            x: 5.0,
            y: 6.0,
            width: 70.0,
            height: 80.0,
        })
    );
}

#[test]
fn merge_ocr_text_into_page_model_rejects_blank_ocr_text() {
    let mut page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Paragraph,
            label: None,
            text: String::from("Existing text"),
            bbox: None,
            source: RegionSource::Dom,
        }],
        interactive_elements: Vec::new(),
    };

    let error = merge_ocr_text_into_page_model(
        &mut page,
        Some("region-1"),
        "   ",
        None,
        String::from("ocr-region-1"),
    )
    .unwrap_err();

    assert_eq!(error.code, "invalid_ocr_text");
    assert_eq!(page.regions[0].text, "Existing text");
    assert_eq!(page.regions[0].source, RegionSource::Dom);
}

#[test]
fn merge_ocr_text_into_page_model_rejects_unknown_target_region() {
    let mut page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: vec![PageRegion {
            region_id: String::from("region-1"),
            role: RegionRole::Paragraph,
            label: None,
            text: String::from("Existing text"),
            bbox: None,
            source: RegionSource::Dom,
        }],
        interactive_elements: Vec::new(),
    };

    let error = merge_ocr_text_into_page_model(
        &mut page,
        Some("missing-region"),
        "Scanned text",
        Some(Rect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 40.0,
        }),
        String::from("ocr-region-1"),
    )
    .unwrap_err();

    assert_eq!(error.code, "unknown_region_id");
    assert_eq!(
        error.details,
        Some(serde_json::json!({ "region_id": "missing-region" }))
    );
    assert_eq!(page.regions.len(), 1);
    assert_eq!(page.regions[0].text, "Existing text");
}

#[test]
fn filter_interactive_elements_applies_visibility_and_role_filters() {
    let elements = vec![
        InteractiveElement {
            element_id: String::from("button-1"),
            dom_locator: Some(String::from("#button-1")),
            role: ElementRole::Button,
            tag_name: String::from("button"),
            text: Some(String::from("Continue")),
            accessible_name: Some(String::from("Continue")),
            placeholder: None,
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::new(),
        },
        InteractiveElement {
            element_id: String::from("link-1"),
            dom_locator: Some(String::from("#link-1")),
            role: ElementRole::Link,
            tag_name: String::from("a"),
            text: Some(String::from("Read more")),
            accessible_name: Some(String::from("Read more")),
            placeholder: None,
            href: Some(String::from("https://example.com/more")),
            value: None,
            bbox: None,
            visible: false,
            enabled: true,
            attributes: std::collections::BTreeMap::new(),
        },
    ];

    let filtered = filter_interactive_elements(&elements, true, Some(&[ElementRole::Button]));

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].element_id, "button-1");
}

#[test]
fn resolve_direct_focus_field_command_focuses_single_matching_field() {
    let page = PageModel {
        title: Some(String::from("Example form")),
        url: Some(String::from("https://example.com/form")),
        regions: Vec::new(),
        interactive_elements: vec![
            InteractiveElement {
                element_id: String::from("input-email"),
                dom_locator: Some(String::from("#email")),
                role: ElementRole::Input,
                tag_name: String::from("input"),
                text: None,
                accessible_name: Some(String::from("Email")),
                placeholder: Some(String::from("Email address")),
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
            InteractiveElement {
                element_id: String::from("input-password"),
                dom_locator: Some(String::from("#password")),
                role: ElementRole::Input,
                tag_name: String::from("input"),
                text: None,
                accessible_name: Some(String::from("Password")),
                placeholder: Some(String::from("Password")),
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
        ],
    };

    let planner_output = resolve_direct_focus_field_command(
        "focus the email field",
        "req-focus-field",
        Some(&page),
        &[String::from("focus_field")],
        0.9,
    )
    .expect("focus-field command should resolve");

    assert_eq!(planner_output.intent.name, IntentName::FillInput);
    assert_eq!(
        planner_output.selected_skills,
        vec![String::from("focus_field")]
    );
    assert_eq!(planner_output.steps.len(), 1);
    assert_eq!(planner_output.steps[0].tool_name, ToolName::FocusElement);
    assert_eq!(
        planner_output.steps[0].arguments.get("element_id"),
        Some(&serde_json::json!("input-email"))
    );
}

#[test]
fn resolve_direct_focus_field_command_reports_missing_description() {
    let page = PageModel {
        title: Some(String::from("Example form")),
        url: Some(String::from("https://example.com/form")),
        regions: Vec::new(),
        interactive_elements: Vec::new(),
    };

    let planner_output = resolve_direct_focus_field_command(
        "focus field",
        "req-focus-field-missing",
        Some(&page),
        &[String::from("focus_field")],
        0.9,
    )
    .expect("focus-field command should resolve");

    assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
    assert_eq!(
        planner_output.steps[0].arguments.get("status"),
        Some(&serde_json::json!(ReportStatus::NeedsFollowUp))
    );
}

#[test]
fn resolve_direct_focus_field_command_reports_ambiguous_match() {
    let page = PageModel {
        title: Some(String::from("Example form")),
        url: Some(String::from("https://example.com/form")),
        regions: Vec::new(),
        interactive_elements: vec![
            InteractiveElement {
                element_id: String::from("input-email"),
                dom_locator: Some(String::from("#email")),
                role: ElementRole::Input,
                tag_name: String::from("input"),
                text: None,
                accessible_name: Some(String::from("Email")),
                placeholder: Some(String::from("Email address")),
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
            InteractiveElement {
                element_id: String::from("input-email-confirm"),
                dom_locator: Some(String::from("#email-confirm")),
                role: ElementRole::Input,
                tag_name: String::from("input"),
                text: None,
                accessible_name: Some(String::from("Email confirmation")),
                placeholder: Some(String::from("Confirm email")),
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
        ],
    };

    let planner_output = resolve_direct_focus_field_command(
        "focus the email field",
        "req-focus-field-ambiguous",
        Some(&page),
        &[String::from("focus_field")],
        0.95,
    )
    .expect("focus-field command should resolve");

    assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
    assert_eq!(
        planner_output.steps[0].arguments.get("status"),
        Some(&serde_json::json!(ReportStatus::NeedsFollowUp))
    );
}

#[test]
fn resolve_direct_fill_field_command_focuses_then_types_into_matching_field() {
    let page = PageModel {
        title: Some(String::from("Example form")),
        url: Some(String::from("https://example.com/form")),
        regions: Vec::new(),
        interactive_elements: vec![
            InteractiveElement {
                element_id: String::from("input-email"),
                dom_locator: Some(String::from("#email")),
                role: ElementRole::Input,
                tag_name: String::from("input"),
                text: None,
                accessible_name: Some(String::from("Email")),
                placeholder: Some(String::from("Email address")),
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
            InteractiveElement {
                element_id: String::from("input-password"),
                dom_locator: Some(String::from("#password")),
                role: ElementRole::Input,
                tag_name: String::from("input"),
                text: None,
                accessible_name: Some(String::from("Password")),
                placeholder: Some(String::from("Password")),
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
        ],
    };

    let planner_output = resolve_direct_fill_field_command(
        "fill the email field with phil@example.com",
        "req-fill-field",
        Some(&page),
        &[String::from("fill_field_by_label")],
        0.9,
    )
    .expect("fill-field command should resolve");

    assert_eq!(planner_output.intent.name, IntentName::FillInput);
    assert_eq!(
        planner_output.selected_skills,
        vec![String::from("fill_field_by_label")]
    );
    assert_eq!(planner_output.steps.len(), 2);
    assert_eq!(planner_output.steps[0].tool_name, ToolName::FocusElement);
    assert_eq!(planner_output.steps[1].tool_name, ToolName::TypeIntoElement);
    assert_eq!(
        planner_output.steps[1].arguments.get("text"),
        Some(&serde_json::json!("phil@example.com"))
    );
}

#[test]
fn resolve_direct_fill_field_command_reports_missing_value() {
    let page = PageModel {
        title: Some(String::from("Example form")),
        url: Some(String::from("https://example.com/form")),
        regions: Vec::new(),
        interactive_elements: Vec::new(),
    };

    let planner_output = resolve_direct_fill_field_command(
        "fill the email field",
        "req-fill-field-missing-value",
        Some(&page),
        &[String::from("fill_field_by_label")],
        0.9,
    )
    .expect("fill-field command should resolve");

    assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
    assert_eq!(
        planner_output.steps[0].arguments.get("status"),
        Some(&serde_json::json!(ReportStatus::NeedsFollowUp))
    );
}

#[test]
fn resolve_direct_fill_and_submit_command_builds_confirmation_gated_plan() {
    let page = PageModel {
        title: Some(String::from("Example form")),
        url: Some(String::from("https://example.com/form")),
        regions: Vec::new(),
        interactive_elements: vec![InteractiveElement {
            element_id: String::from("input-email"),
            dom_locator: Some(String::from("#email")),
            role: ElementRole::Input,
            tag_name: String::from("input"),
            text: None,
            accessible_name: Some(String::from("Email")),
            placeholder: Some(String::from("Email address")),
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::new(),
        }],
    };

    let planner_output = resolve_direct_fill_and_submit_command(
        "fill the email field with phil@example.com and then submit",
        "req-fill-submit",
        Some(&page),
        &[String::from("fill_and_submit_form")],
        0.9,
    )
    .expect("fill-and-submit command should resolve");

    assert_eq!(planner_output.intent.name, IntentName::SubmitForm);
    assert_eq!(planner_output.status, PlannerStatus::NeedsConfirmation);
    assert_eq!(
        planner_output.selected_skills,
        vec![String::from("fill_and_submit_form")]
    );
    assert_eq!(planner_output.steps.len(), 4);
    assert_eq!(planner_output.steps[0].tool_name, ToolName::ConfirmAction);
    assert_eq!(planner_output.steps[1].tool_name, ToolName::FocusElement);
    assert_eq!(planner_output.steps[2].tool_name, ToolName::TypeIntoElement);
    assert_eq!(
        planner_output.steps[3].tool_name,
        ToolName::SubmitActiveForm
    );
    assert_eq!(
        planner_output.steps[2].arguments.get("text"),
        Some(&serde_json::json!("phil@example.com"))
    );
    assert_eq!(
        planner_output.steps[3].arguments.get("form_element_id"),
        Some(&serde_json::Value::Null)
    );
    assert!(planner_output.requires_confirmation);
}

#[test]
fn resolve_direct_fill_and_submit_command_reports_missing_value() {
    let page = PageModel {
        title: Some(String::from("Example form")),
        url: Some(String::from("https://example.com/form")),
        regions: Vec::new(),
        interactive_elements: Vec::new(),
    };

    let planner_output = resolve_direct_fill_and_submit_command(
        "fill the email field and submit",
        "req-fill-submit-missing-value",
        Some(&page),
        &[String::from("fill_and_submit_form")],
        0.9,
    )
    .expect("fill-and-submit command should resolve");

    assert_eq!(planner_output.intent.name, IntentName::SubmitForm);
    assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
    assert_eq!(
        planner_output.steps[0].arguments.get("status"),
        Some(&serde_json::json!(ReportStatus::NeedsFollowUp))
    );
}

#[test]
fn resolve_recent_fill_correction_command_reuses_recent_target_for_replacement() {
    let page = PageModel {
        title: Some(String::from("Example form")),
        url: Some(String::from("https://example.com/form")),
        regions: Vec::new(),
        interactive_elements: vec![InteractiveElement {
            element_id: String::from("input-city"),
            dom_locator: Some(String::from("#city")),
            role: ElementRole::Input,
            tag_name: String::from("input"),
            text: None,
            accessible_name: Some(String::from("City")),
            placeholder: Some(String::from("City")),
            href: None,
            value: Some(String::from("Portland")),
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::new(),
        }],
    };

    let (planner_output, next_context) = resolve_recent_fill_correction_command(
        "put Seattle there instead",
        "req-fill-correction",
        Some("page-1"),
        Some(&page),
        &[String::from("fill_field_by_label")],
        Some(&RecentFieldContext {
            page_id: String::from("page-1"),
            target_description: Some(String::from("city")),
            active_element_id: Some(String::from("input-city")),
            candidate_element_ids: vec![String::from("input-city")],
            pending_text: Some(String::from("Portland")),
            submit_after: false,
        }),
    )
    .expect("follow-up correction should resolve");

    assert_eq!(planner_output.intent.name, IntentName::FillInput);
    assert_eq!(planner_output.status, PlannerStatus::Ready);
    assert_eq!(planner_output.steps[0].tool_name, ToolName::FocusElement);
    assert_eq!(planner_output.steps[1].tool_name, ToolName::TypeIntoElement);
    assert_eq!(
        planner_output.steps[1].arguments.get("text"),
        Some(&serde_json::json!("Seattle"))
    );
    assert_eq!(
        next_context.and_then(|context| context.pending_text),
        Some(String::from("Seattle"))
    );
}

#[test]
fn resolve_recent_fill_correction_command_switches_to_alternate_candidate() {
    let page = PageModel {
        title: Some(String::from("Example form")),
        url: Some(String::from("https://example.com/form")),
        regions: Vec::new(),
        interactive_elements: vec![
            InteractiveElement {
                element_id: String::from("input-email"),
                dom_locator: Some(String::from("#email")),
                role: ElementRole::Input,
                tag_name: String::from("input"),
                text: None,
                accessible_name: Some(String::from("Email")),
                placeholder: Some(String::from("Email")),
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
            InteractiveElement {
                element_id: String::from("input-billing-email"),
                dom_locator: Some(String::from("#billing-email")),
                role: ElementRole::Input,
                tag_name: String::from("input"),
                text: None,
                accessible_name: Some(String::from("Billing email")),
                placeholder: Some(String::from("Billing email")),
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
        ],
    };

    let (planner_output, next_context) = resolve_recent_fill_correction_command(
        "no, the other field",
        "req-fill-other-field",
        Some("page-1"),
        Some(&page),
        &[String::from("fill_and_submit_form")],
        Some(&RecentFieldContext {
            page_id: String::from("page-1"),
            target_description: Some(String::from("email")),
            active_element_id: Some(String::from("input-email")),
            candidate_element_ids: vec![
                String::from("input-email"),
                String::from("input-billing-email"),
            ],
            pending_text: Some(String::from("phil@example.com")),
            submit_after: true,
        }),
    )
    .expect("alternate-field correction should resolve");

    assert_eq!(planner_output.intent.name, IntentName::SubmitForm);
    assert_eq!(planner_output.status, PlannerStatus::NeedsConfirmation);
    assert_eq!(planner_output.steps[0].tool_name, ToolName::ConfirmAction);
    assert_eq!(planner_output.steps[1].tool_name, ToolName::FocusElement);
    assert_eq!(
        planner_output.steps[1].arguments.get("element_id"),
        Some(&serde_json::json!("input-billing-email"))
    );
    assert_eq!(
        next_context.and_then(|context| context.active_element_id),
        Some(String::from("input-billing-email"))
    );
}

#[test]
fn resolve_recent_fill_correction_command_asks_for_target_without_recent_context() {
    let (planner_output, next_context) = resolve_recent_fill_correction_command(
        "put Seattle there instead",
        "req-fill-no-context",
        None,
        None,
        &[String::from("fill_field_by_label")],
        None,
    )
    .expect("correction phrase should still produce a bounded follow-up");

    assert_eq!(planner_output.intent.name, IntentName::FillInput);
    assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
    assert_eq!(
        planner_output.steps[0].arguments.get("status"),
        Some(&serde_json::json!(ReportStatus::NeedsFollowUp))
    );
    assert!(next_context.is_none());
}

#[test]
fn resolve_typeable_element_rejects_non_field_roles() {
    let page = PageModel {
        title: Some(String::from("Example page")),
        url: Some(String::from("https://example.com")),
        regions: Vec::new(),
        interactive_elements: vec![InteractiveElement {
            element_id: String::from("button-1"),
            dom_locator: Some(String::from("#button-1")),
            role: ElementRole::Button,
            tag_name: String::from("button"),
            text: Some(String::from("Continue")),
            accessible_name: Some(String::from("Continue")),
            placeholder: None,
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::new(),
        }],
    };

    let error = resolve_typeable_element(&page, "button-1")
        .expect_err("non-field roles should be rejected");
    assert_eq!(error.code, "element_not_editable");
}

#[test]
fn resolve_direct_submit_form_command_builds_confirmation_gated_submit_plan() {
    let page = PageModel {
        title: Some(String::from("Login")),
        url: Some(String::from("https://example.com/login")),
        regions: Vec::new(),
        interactive_elements: vec![InteractiveElement {
            element_id: String::from("form-login"),
            dom_locator: Some(String::from("#login-form")),
            role: ElementRole::Form,
            tag_name: String::from("form"),
            text: Some(String::from("Sign in")),
            accessible_name: Some(String::from("Login")),
            placeholder: None,
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::new(),
        }],
    };

    let planner_output = resolve_direct_submit_form_command(
        "submit form",
        "req-submit-form",
        Some(&page),
        &[String::from("submit_form")],
    )
    .expect("submit-form command should resolve");

    assert_eq!(planner_output.status, PlannerStatus::NeedsConfirmation);
    assert_eq!(planner_output.intent.name, IntentName::SubmitForm);
    assert_eq!(
        planner_output.selected_skills,
        vec![String::from("submit_form")]
    );
    assert_eq!(planner_output.steps.len(), 2);
    assert_eq!(planner_output.steps[0].tool_name, ToolName::ConfirmAction);
    assert_eq!(
        planner_output.steps[1].tool_name,
        ToolName::SubmitActiveForm
    );
    assert_eq!(
        planner_output.steps[1].arguments.get("form_element_id"),
        Some(&serde_json::json!("form-login"))
    );
    assert!(planner_output.requires_confirmation);
}

#[test]
fn resolve_direct_submit_form_command_reports_ambiguous_forms() {
    let page = PageModel {
        title: Some(String::from("Checkout")),
        url: Some(String::from("https://example.com/checkout")),
        regions: Vec::new(),
        interactive_elements: vec![
            InteractiveElement {
                element_id: String::from("form-shipping"),
                dom_locator: Some(String::from("#shipping-form")),
                role: ElementRole::Form,
                tag_name: String::from("form"),
                text: Some(String::from("Shipping")),
                accessible_name: Some(String::from("Shipping")),
                placeholder: None,
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
            InteractiveElement {
                element_id: String::from("form-billing"),
                dom_locator: Some(String::from("#billing-form")),
                role: ElementRole::Form,
                tag_name: String::from("form"),
                text: Some(String::from("Billing")),
                accessible_name: Some(String::from("Billing")),
                placeholder: None,
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
        ],
    };

    let planner_output = resolve_direct_submit_form_command(
        "submit form",
        "req-submit-form-ambiguous",
        Some(&page),
        &[String::from("submit_form")],
    )
    .expect("submit-form command should resolve");

    assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
    assert_eq!(
        planner_output.steps[0].arguments.get("status"),
        Some(&serde_json::json!(ReportStatus::NeedsFollowUp))
    );
}

#[test]
fn app_core_form_regression_fixtures_cover_ambiguous_fill_submit_and_follow_up_cases() {
    let fixtures = vec![
        AppCorePlannerFixture {
            name: "ambiguous-focus-field",
            kind: AppCorePlannerFixtureKind::FocusField,
            transcript: "focus the email field",
            current_page_id: None,
            page: Some(fixture_page(vec![
                fixture_field("input-email", "#email", "Email", "Email address"),
                fixture_field(
                    "input-email-confirm",
                    "#email-confirm",
                    "Email confirmation",
                    "Confirm email",
                ),
            ])),
            active_skills: vec!["focus_field"],
            recent_context: None,
            confirmation_threshold: 0.95,
            expected_intent: IntentName::FillInput,
            expected_status: PlannerStatus::Ready,
            expected_selected_skills: vec!["focus_field"],
            expected_tool_sequence: vec![ToolName::ReportResult],
            expected_focus_element_id: None,
            expected_typed_text: None,
            expected_next_active_element_id: None,
            expected_next_pending_text: None,
        },
        AppCorePlannerFixture {
            name: "fill-field-success",
            kind: AppCorePlannerFixtureKind::FillField,
            transcript: "fill the email field with phil@example.com",
            current_page_id: None,
            page: Some(fixture_page(vec![
                fixture_field("input-email", "#email", "Email", "Email address"),
                fixture_field("input-password", "#password", "Password", "Password"),
            ])),
            active_skills: vec!["fill_field_by_label"],
            recent_context: None,
            confirmation_threshold: 0.9,
            expected_intent: IntentName::FillInput,
            expected_status: PlannerStatus::Ready,
            expected_selected_skills: vec!["fill_field_by_label"],
            expected_tool_sequence: vec![ToolName::FocusElement, ToolName::TypeIntoElement],
            expected_focus_element_id: Some("input-email"),
            expected_typed_text: Some("phil@example.com"),
            expected_next_active_element_id: None,
            expected_next_pending_text: None,
        },
        AppCorePlannerFixture {
            name: "fill-and-submit-confirmation",
            kind: AppCorePlannerFixtureKind::FillAndSubmit,
            transcript: "fill the email field with phil@example.com and then submit",
            current_page_id: None,
            page: Some(fixture_page(vec![fixture_field(
                "input-email",
                "#email",
                "Email",
                "Email address",
            )])),
            active_skills: vec!["fill_and_submit_form"],
            recent_context: None,
            confirmation_threshold: 0.9,
            expected_intent: IntentName::SubmitForm,
            expected_status: PlannerStatus::NeedsConfirmation,
            expected_selected_skills: vec!["fill_and_submit_form"],
            expected_tool_sequence: vec![
                ToolName::ConfirmAction,
                ToolName::FocusElement,
                ToolName::TypeIntoElement,
                ToolName::SubmitActiveForm,
            ],
            expected_focus_element_id: Some("input-email"),
            expected_typed_text: Some("phil@example.com"),
            expected_next_active_element_id: None,
            expected_next_pending_text: None,
        },
        AppCorePlannerFixture {
            name: "follow-up-replacement",
            kind: AppCorePlannerFixtureKind::FollowUpCorrection,
            transcript: "put Seattle there instead",
            current_page_id: Some("page-1"),
            page: Some(fixture_page(vec![InteractiveElement {
                element_id: String::from("input-city"),
                dom_locator: Some(String::from("#city")),
                role: ElementRole::Input,
                tag_name: String::from("input"),
                text: None,
                accessible_name: Some(String::from("City")),
                placeholder: Some(String::from("City")),
                href: None,
                value: Some(String::from("Portland")),
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            }])),
            active_skills: vec!["fill_field_by_label"],
            recent_context: Some(RecentFieldContext {
                page_id: String::from("page-1"),
                target_description: Some(String::from("city")),
                active_element_id: Some(String::from("input-city")),
                candidate_element_ids: vec![String::from("input-city")],
                pending_text: Some(String::from("Portland")),
                submit_after: false,
            }),
            confirmation_threshold: 0.9,
            expected_intent: IntentName::FillInput,
            expected_status: PlannerStatus::Ready,
            expected_selected_skills: vec!["fill_field_by_label"],
            expected_tool_sequence: vec![ToolName::FocusElement, ToolName::TypeIntoElement],
            expected_focus_element_id: Some("input-city"),
            expected_typed_text: Some("Seattle"),
            expected_next_active_element_id: Some("input-city"),
            expected_next_pending_text: Some("Seattle"),
        },
        AppCorePlannerFixture {
            name: "follow-up-other-field",
            kind: AppCorePlannerFixtureKind::FollowUpCorrection,
            transcript: "no, the other field",
            current_page_id: Some("page-1"),
            page: Some(fixture_page(vec![
                fixture_field("input-email", "#email", "Email", "Email"),
                fixture_field(
                    "input-billing-email",
                    "#billing-email",
                    "Billing email",
                    "Billing email",
                ),
            ])),
            active_skills: vec!["fill_and_submit_form"],
            recent_context: Some(RecentFieldContext {
                page_id: String::from("page-1"),
                target_description: Some(String::from("email")),
                active_element_id: Some(String::from("input-email")),
                candidate_element_ids: vec![
                    String::from("input-email"),
                    String::from("input-billing-email"),
                ],
                pending_text: Some(String::from("phil@example.com")),
                submit_after: true,
            }),
            confirmation_threshold: 0.9,
            expected_intent: IntentName::SubmitForm,
            expected_status: PlannerStatus::NeedsConfirmation,
            expected_selected_skills: vec!["fill_and_submit_form"],
            expected_tool_sequence: vec![
                ToolName::ConfirmAction,
                ToolName::FocusElement,
                ToolName::TypeIntoElement,
                ToolName::SubmitActiveForm,
            ],
            expected_focus_element_id: Some("input-billing-email"),
            expected_typed_text: Some("phil@example.com"),
            expected_next_active_element_id: Some("input-billing-email"),
            expected_next_pending_text: Some("phil@example.com"),
        },
        AppCorePlannerFixture {
            name: "ambiguous-submit-form",
            kind: AppCorePlannerFixtureKind::SubmitForm,
            transcript: "submit form",
            current_page_id: None,
            page: Some(fixture_page(vec![
                fixture_form("form-shipping", "#shipping-form", "Shipping"),
                fixture_form("form-billing", "#billing-form", "Billing"),
            ])),
            active_skills: vec!["submit_form"],
            recent_context: None,
            confirmation_threshold: 0.9,
            expected_intent: IntentName::SubmitForm,
            expected_status: PlannerStatus::Ready,
            expected_selected_skills: vec!["submit_form"],
            expected_tool_sequence: vec![ToolName::ReportResult],
            expected_focus_element_id: None,
            expected_typed_text: None,
            expected_next_active_element_id: None,
            expected_next_pending_text: None,
        },
    ];

    for fixture in fixtures {
        assert_app_core_planner_fixture(fixture);
    }
}

#[test]
fn ambiguous_click_regression_fixtures_pin_confirmation_threshold_behavior() {
    struct AmbiguousClickFixture {
        name: &'static str,
        candidates: Vec<crate::commands::ElementCandidate>,
        confirmation_threshold: f32,
        expected_element_id: Option<&'static str>,
        expected_confidence: Option<f32>,
        expected_requires_confirmation: bool,
    }

    let fixtures = vec![
        AmbiguousClickFixture {
            name: "close-candidates-trigger-follow-up",
            candidates: vec![
                crate::commands::ElementCandidate {
                    element_id: String::from("button-1"),
                    confidence_bps: 8_900,
                    matched_on: vec![String::from("description")],
                    rationale_codes: vec![String::from("accessible_name_exact")],
                },
                crate::commands::ElementCandidate {
                    element_id: String::from("button-2"),
                    confidence_bps: 8_400,
                    matched_on: vec![String::from("description")],
                    rationale_codes: vec![String::from("accessible_name_exact")],
                },
            ],
            confirmation_threshold: 0.9,
            expected_element_id: None,
            expected_confidence: Some(0.89),
            expected_requires_confirmation: true,
        },
        AmbiguousClickFixture {
            name: "threshold-crossing-allows-direct-click",
            candidates: vec![crate::commands::ElementCandidate {
                element_id: String::from("link-help"),
                confidence_bps: 8_800,
                matched_on: vec![String::from("accessible_name")],
                rationale_codes: vec![String::from("accessible_name_exact")],
            }],
            confirmation_threshold: 0.85,
            expected_element_id: Some("link-help"),
            expected_confidence: Some(0.88),
            expected_requires_confirmation: false,
        },
    ];

    for fixture in fixtures {
        let (chosen_element_id, chosen_confidence, requires_confirmation) =
            determine_find_element_resolution(
                &fixture.candidates,
                fixture.confirmation_threshold,
            );

        assert_eq!(
            chosen_element_id.as_deref(),
            fixture.expected_element_id,
            "fixture {} chose the wrong element",
            fixture.name
        );
        assert_eq!(
            chosen_confidence, fixture.expected_confidence,
            "fixture {} produced unexpected confidence",
            fixture.name
        );
        assert_eq!(
            requires_confirmation, fixture.expected_requires_confirmation,
            "fixture {} produced unexpected confirmation behavior",
            fixture.name
        );
    }
}

#[test]
fn problematic_page_regression_fixtures_cover_checkout_and_duplicate_cta_shapes() {
    let checkout_page = fixture_problematic_checkout_page();
    let newsletter_page = fixture_problematic_newsletter_page();
    let fixtures = vec![
        AppCorePlannerFixture {
            name: "problematic-checkout-ambiguous-email-focus",
            kind: AppCorePlannerFixtureKind::FocusField,
            transcript: "focus the email field",
            current_page_id: None,
            page: Some(checkout_page.clone()),
            active_skills: vec!["focus_field"],
            recent_context: None,
            confirmation_threshold: 0.95,
            expected_intent: IntentName::FillInput,
            expected_status: PlannerStatus::Ready,
            expected_selected_skills: vec!["focus_field"],
            expected_tool_sequence: vec![ToolName::ReportResult],
            expected_focus_element_id: None,
            expected_typed_text: None,
            expected_next_active_element_id: None,
            expected_next_pending_text: None,
        },
        AppCorePlannerFixture {
            name: "problematic-newsletter-fill-email",
            kind: AppCorePlannerFixtureKind::FillField,
            transcript: "fill the email field with phil@example.com",
            current_page_id: None,
            page: Some(newsletter_page),
            active_skills: vec!["fill_field_by_label"],
            recent_context: None,
            confirmation_threshold: 0.9,
            expected_intent: IntentName::FillInput,
            expected_status: PlannerStatus::Ready,
            expected_selected_skills: vec!["fill_field_by_label"],
            expected_tool_sequence: vec![ToolName::FocusElement, ToolName::TypeIntoElement],
            expected_focus_element_id: Some("input-newsletter-email"),
            expected_typed_text: Some("phil@example.com"),
            expected_next_active_element_id: None,
            expected_next_pending_text: None,
        },
        AppCorePlannerFixture {
            name: "problematic-checkout-other-field-correction",
            kind: AppCorePlannerFixtureKind::FollowUpCorrection,
            transcript: "no, the other field",
            current_page_id: Some("checkout-page"),
            page: Some(checkout_page.clone()),
            active_skills: vec!["fill_and_submit_form"],
            recent_context: Some(RecentFieldContext {
                page_id: String::from("checkout-page"),
                target_description: Some(String::from("email")),
                active_element_id: Some(String::from("input-shipping-email")),
                candidate_element_ids: vec![
                    String::from("input-shipping-email"),
                    String::from("input-billing-email"),
                ],
                pending_text: Some(String::from("phil@example.com")),
                submit_after: true,
            }),
            confirmation_threshold: 0.9,
            expected_intent: IntentName::SubmitForm,
            expected_status: PlannerStatus::NeedsConfirmation,
            expected_selected_skills: vec!["fill_and_submit_form"],
            expected_tool_sequence: vec![
                ToolName::ConfirmAction,
                ToolName::FocusElement,
                ToolName::TypeIntoElement,
                ToolName::SubmitActiveForm,
            ],
            expected_focus_element_id: Some("input-billing-email"),
            expected_typed_text: Some("phil@example.com"),
            expected_next_active_element_id: Some("input-billing-email"),
            expected_next_pending_text: Some("phil@example.com"),
        },
        AppCorePlannerFixture {
            name: "problematic-checkout-ambiguous-submit",
            kind: AppCorePlannerFixtureKind::SubmitForm,
            transcript: "submit form",
            current_page_id: None,
            page: Some(checkout_page),
            active_skills: vec!["submit_form"],
            recent_context: None,
            confirmation_threshold: 0.9,
            expected_intent: IntentName::SubmitForm,
            expected_status: PlannerStatus::Ready,
            expected_selected_skills: vec!["submit_form"],
            expected_tool_sequence: vec![ToolName::ReportResult],
            expected_focus_element_id: None,
            expected_typed_text: None,
            expected_next_active_element_id: None,
            expected_next_pending_text: None,
        },
    ];

    for fixture in fixtures {
        assert_app_core_planner_fixture(fixture);
    }

    let landing_page = fixture_problematic_landing_page();
    let query = build_find_element_query(&FindElementInput {
        request_id: String::from("req-problematic-cta"),
        timeout_ms: None,
        description: String::from("Get started"),
        text: None,
        role: Some(ElementRole::Button),
        color_hint: None,
        nearby_text: None,
        selector_hint: None,
        visibility_filter: crate::commands::ElementVisibilityFilter::VisibleOnly,
        max_candidates: Some(3),
    })
    .expect("landing-page query should be valid");
    let candidates =
        rank_find_element_candidates(&landing_page.interactive_elements, &query, 3);
    let (chosen_element_id, _, requires_confirmation) =
        determine_find_element_resolution(&candidates, 0.9);

    assert_eq!(candidates.len(), 2);
    assert_eq!(chosen_element_id, None);
    assert!(requires_confirmation);
}

#[test]
fn resolve_form_element_rejects_non_form_roles() {
    let page = PageModel {
        title: Some(String::from("Example page")),
        url: Some(String::from("https://example.com")),
        regions: Vec::new(),
        interactive_elements: vec![InteractiveElement {
            element_id: String::from("button-1"),
            dom_locator: Some(String::from("#button-1")),
            role: ElementRole::Button,
            tag_name: String::from("button"),
            text: Some(String::from("Submit")),
            accessible_name: Some(String::from("Submit")),
            placeholder: None,
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::new(),
        }],
    };

    let error =
        resolve_form_element(&page, "button-1").expect_err("non-form roles should be rejected");
    assert_eq!(error.code, "element_not_form");
}

#[test]
fn rank_find_element_candidates_prefers_exact_accessible_name_matches() {
    let elements = vec![
        InteractiveElement {
            element_id: String::from("button-1"),
            dom_locator: Some(String::from("#button-1")),
            role: ElementRole::Button,
            tag_name: String::from("button"),
            text: Some(String::from("Continue")),
            accessible_name: Some(String::from("Continue")),
            placeholder: None,
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::new(),
        },
        InteractiveElement {
            element_id: String::from("button-2"),
            dom_locator: Some(String::from("#button-2")),
            role: ElementRole::Button,
            tag_name: String::from("button"),
            text: Some(String::from("Continue reading")),
            accessible_name: Some(String::from("Continue reading")),
            placeholder: None,
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::new(),
        },
    ];
    let query = build_find_element_query(&FindElementInput {
        request_id: String::from("req-find"),
        timeout_ms: None,
        description: String::from("Continue"),
        text: None,
        role: Some(ElementRole::Button),
        color_hint: None,
        nearby_text: None,
        selector_hint: None,
        visibility_filter: crate::commands::ElementVisibilityFilter::VisibleOnly,
        max_candidates: Some(3),
    })
    .expect("query should be valid");

    let candidates = rank_find_element_candidates(&elements, &query, 3);

    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].element_id, "button-1");
    assert!(candidates[0].confidence_bps > candidates[1].confidence_bps);
}

#[test]
fn rank_find_element_candidates_uses_selector_hint_and_respects_candidate_limit() {
    let elements = vec![
        InteractiveElement {
            element_id: String::from("button-primary"),
            dom_locator: Some(String::from("#checkout-submit")),
            role: ElementRole::Button,
            tag_name: String::from("button"),
            text: Some(String::from("Continue")),
            accessible_name: Some(String::from("Continue")),
            placeholder: None,
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::from([
                (String::from("data-testid"), String::from("checkout-submit")),
                (String::from("class"), String::from("cta primary")),
            ]),
        },
        InteractiveElement {
            element_id: String::from("button-secondary"),
            dom_locator: Some(String::from("#continue-reading")),
            role: ElementRole::Button,
            tag_name: String::from("button"),
            text: Some(String::from("Continue")),
            accessible_name: Some(String::from("Continue")),
            placeholder: None,
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::from([(
                String::from("data-testid"),
                String::from("continue-reading"),
            )]),
        },
        InteractiveElement {
            element_id: String::from("button-tertiary"),
            dom_locator: Some(String::from("#continue-later")),
            role: ElementRole::Button,
            tag_name: String::from("button"),
            text: Some(String::from("Continue later")),
            accessible_name: Some(String::from("Continue later")),
            placeholder: None,
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::from([(
                String::from("data-testid"),
                String::from("continue-later"),
            )]),
        },
    ];
    let query = build_find_element_query(&FindElementInput {
        request_id: String::from("req-find"),
        timeout_ms: None,
        description: String::from("Continue"),
        text: None,
        role: Some(ElementRole::Button),
        color_hint: None,
        nearby_text: None,
        selector_hint: Some(String::from("checkout-submit")),
        visibility_filter: crate::commands::ElementVisibilityFilter::VisibleOnly,
        max_candidates: Some(2),
    })
    .expect("query should be valid");

    let candidates = rank_find_element_candidates(&elements, &query, 2);

    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0].element_id, "button-primary");
    assert!(candidates[0]
        .matched_on
        .iter()
        .any(|matched_on| matched_on == "selector_hint"));
    assert!(candidates[0].confidence_bps > candidates[1].confidence_bps);
    assert!(!candidates
        .iter()
        .any(|candidate| candidate.element_id == "button-tertiary"));
}

#[test]
fn build_find_element_query_normalizes_optional_hints_into_summary() {
    let query = build_find_element_query(&FindElementInput {
        request_id: String::from("req-find"),
        timeout_ms: None,
        description: String::from("  Continue  "),
        text: Some(String::from("  Start now  ")),
        role: Some(ElementRole::Button),
        color_hint: Some(String::from("  primary blue  ")),
        nearby_text: Some(String::from("  pricing  ")),
        selector_hint: Some(String::from("  cta-primary  ")),
        visibility_filter: crate::commands::ElementVisibilityFilter::VisibleOnly,
        max_candidates: Some(3),
    })
    .expect("query should be valid");

    assert_eq!(query.description.as_deref(), Some("Continue"));
    assert_eq!(query.text.as_deref(), Some("Start now"));
    assert_eq!(query.color_hint.as_deref(), Some("primary blue"));
    assert_eq!(query.nearby_text.as_deref(), Some("pricing"));
    assert_eq!(query.selector_hint.as_deref(), Some("cta-primary"));
    assert_eq!(
        query.summary,
        "description=Continue; text=Start now; role=Button; color_hint=primary blue; nearby_text=pricing; selector_hint=cta-primary"
    );
}

#[test]
fn determine_find_element_resolution_flags_close_candidates_for_confirmation() {
    let candidates = vec![
        crate::commands::ElementCandidate {
            element_id: String::from("button-1"),
            confidence_bps: 8_900,
            matched_on: vec![String::from("description")],
            rationale_codes: vec![String::from("accessible_name_exact")],
        },
        crate::commands::ElementCandidate {
            element_id: String::from("button-2"),
            confidence_bps: 8_400,
            matched_on: vec![String::from("description")],
            rationale_codes: vec![String::from("accessible_name_exact")],
        },
    ];

    let (chosen_element_id, chosen_confidence, requires_confirmation) =
        determine_find_element_resolution(&candidates, 0.9);

    assert_eq!(chosen_element_id, None);
    assert_eq!(chosen_confidence, Some(0.89));
    assert!(requires_confirmation);
}

#[test]
fn determine_find_element_resolution_uses_configured_confidence_threshold() {
    let candidates = vec![crate::commands::ElementCandidate {
        element_id: String::from("link-help"),
        confidence_bps: 8_800,
        matched_on: vec![String::from("accessible_name")],
        rationale_codes: vec![String::from("accessible_name_exact")],
    }];

    let (chosen_element_id, chosen_confidence, requires_confirmation) =
        determine_find_element_resolution(&candidates, 0.9);
    assert_eq!(chosen_element_id, None);
    assert_eq!(chosen_confidence, Some(0.88));
    assert!(requires_confirmation);

    let (chosen_element_id, chosen_confidence, requires_confirmation) =
        determine_find_element_resolution(&candidates, 0.85);
    assert_eq!(chosen_element_id, Some(String::from("link-help")));
    assert_eq!(chosen_confidence, Some(0.88));
    assert!(!requires_confirmation);
}

#[test]
fn planner_system_prompt_mentions_click_confirmation_config() {
    let prompt = planner_system_prompt();

    assert!(prompt.contains("planner_input.safety.allow_click_without_confirmation"));
    assert!(prompt.contains("ordinary ClickElement plans may use Ready"));
    assert!(prompt.contains("planner_input.safety.confirmation_confidence_threshold"));
}

struct MockReplanningRuntime {
    resolve_results: Vec<Result<PlannerOutput, crate::commands::ToolError>>,
    execute_results: Vec<ExecutionOutcome>,
    resolve_recent_tool_results: Vec<Vec<PlannerToolHistoryEntry>>,
    execute_request_ids: Vec<String>,
}

impl ReplanningRuntime for MockReplanningRuntime {
    fn resolve_plan(
        &mut self,
        _request_id: String,
        _transcript: &str,
        recent_tool_results: &[PlannerToolHistoryEntry],
    ) -> Result<PlannerOutput, crate::commands::ToolError> {
        self.resolve_recent_tool_results
            .push(recent_tool_results.to_vec());
        self.resolve_results.remove(0)
    }

    fn execute_plan(
        &mut self,
        request_id: String,
        _planner_output: &PlannerOutput,
    ) -> ExecutionOutcome {
        self.execute_request_ids.push(request_id);
        self.execute_results.remove(0)
    }
}

fn mock_planner_output(step_id: &str) -> PlannerOutput {
    PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::GetStatus,
            goal: String::from("report runtime status"),
            target_description: None,
        },
        selected_skills: vec![String::from("get_status")],
        steps: vec![PlannedStep {
            step_id: step_id.to_string(),
            tool_name: ToolName::GetRuntimeStatus,
            arguments: serde_json::json!({
                "request_id": format!("req-{step_id}"),
                "timeout_ms": null,
                "include_provider_modes": false
            }),
            purpose: String::from("read runtime status"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    }
}

fn mock_trace(step_id: &str, tool_name: ToolName, observation: &str) -> ExecutionTrace {
    ExecutionTrace {
        executed_step_ids: vec![step_id.to_string()],
        tool_results: vec![ToolResult::success(
            tool_name,
            format!("req-{step_id}"),
            serde_json::json!({}),
            vec![observation.to_string()],
        )],
    }
}

#[test]
fn bounded_replanning_loop_replans_once_with_recent_tool_history() {
    let mut runtime = MockReplanningRuntime {
        resolve_results: vec![
            Ok(mock_planner_output("step-1")),
            Ok(mock_planner_output("step-2")),
        ],
        execute_results: vec![
            ExecutionOutcome::NeedsReplan {
                trace: mock_trace("step-1", ToolName::GetRuntimeStatus, "first plan failed"),
            },
            ExecutionOutcome::Complete {
                trace: mock_trace("step-2", ToolName::ReportResult, "second plan succeeded"),
            },
        ],
        resolve_recent_tool_results: Vec::new(),
        execute_request_ids: Vec::new(),
    };

    let outcome = execute_bounded_replanning_loop(&mut runtime, "req", "what is the status")
        .expect("bounded replanning should succeed");

    match outcome {
        ExecutionOutcome::Complete { trace } => {
            assert_eq!(trace.executed_step_ids, vec!["step-1", "step-2"]);
            assert_eq!(trace.tool_results.len(), 2);
        }
        other => panic!("expected complete outcome, got {other:?}"),
    }

    assert_eq!(runtime.resolve_recent_tool_results.len(), 2);
    assert!(runtime.resolve_recent_tool_results[0].is_empty());
    assert_eq!(runtime.resolve_recent_tool_results[1].len(), 1);
    assert_eq!(
        runtime.resolve_recent_tool_results[1][0].observation_summary,
        vec![String::from("first plan failed")]
    );
    assert_eq!(
        runtime.execute_request_ids,
        vec![
            String::from("req-execute"),
            String::from("req-execute-replan-1")
        ]
    );
}

#[test]
fn bounded_replanning_loop_stops_after_replan_limit() {
    let mut runtime = MockReplanningRuntime {
        resolve_results: vec![
            Ok(mock_planner_output("step-1")),
            Ok(mock_planner_output("step-2")),
        ],
        execute_results: vec![
            ExecutionOutcome::NeedsReplan {
                trace: mock_trace(
                    "step-1",
                    ToolName::GetRuntimeStatus,
                    "first replan requested",
                ),
            },
            ExecutionOutcome::NeedsReplan {
                trace: mock_trace(
                    "step-2",
                    ToolName::GetRuntimeStatus,
                    "second replan requested",
                ),
            },
        ],
        resolve_recent_tool_results: Vec::new(),
        execute_request_ids: Vec::new(),
    };

    let outcome = execute_bounded_replanning_loop(&mut runtime, "req", "what is the status")
        .expect("bounded replanning should return an execution outcome");

    match outcome {
        ExecutionOutcome::Aborted { trace, error } => {
            assert_eq!(error.code, "replan_limit_exceeded");
            assert_eq!(trace.executed_step_ids, vec!["step-1", "step-2"]);
            assert_eq!(trace.tool_results.len(), 2);
        }
        other => panic!("expected aborted outcome, got {other:?}"),
    }
}

#[test]
fn bounded_replanning_loop_aborts_with_accumulated_trace_when_follow_up_resolution_fails() {
    let mut runtime = MockReplanningRuntime {
        resolve_results: vec![
            Ok(mock_planner_output("step-1")),
            Err(crate::commands::ToolError {
                code: String::from("planner_backend_unavailable"),
                message: String::from("planner could not resolve a follow-up plan"),
                retryable: true,
                details: Some(serde_json::json!({
                    "attempt": 2
                })),
            }),
        ],
        execute_results: vec![ExecutionOutcome::NeedsReplan {
            trace: mock_trace("step-1", ToolName::GetRuntimeStatus, "first plan failed"),
        }],
        resolve_recent_tool_results: Vec::new(),
        execute_request_ids: Vec::new(),
    };

    let outcome = execute_bounded_replanning_loop(&mut runtime, "req", "what is the status")
        .expect("bounded replanning should surface an aborted execution outcome");

    match outcome {
        ExecutionOutcome::Aborted { trace, error } => {
            assert_eq!(error.code, "planner_backend_unavailable");
            assert_eq!(trace.executed_step_ids, vec![String::from("step-1")]);
            assert_eq!(trace.tool_results.len(), 1);
            assert_eq!(
                trace.tool_results[0].observations,
                vec![String::from("first plan failed")]
            );
        }
        other => panic!("expected aborted outcome, got {other:?}"),
    }

    assert_eq!(runtime.resolve_recent_tool_results.len(), 2);
    assert!(runtime.resolve_recent_tool_results[0].is_empty());
    assert_eq!(runtime.resolve_recent_tool_results[1].len(), 1);
    assert_eq!(
        runtime.execute_request_ids,
        vec![String::from("req-execute")]
    );
}

#[test]
fn resolve_clickable_element_requires_an_enabled_visible_exact_match() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: Vec::new(),
        interactive_elements: vec![InteractiveElement {
            element_id: String::from("button-disabled"),
            dom_locator: Some(String::from("#button-disabled")),
            role: ElementRole::Button,
            tag_name: String::from("button"),
            text: Some(String::from("Continue")),
            accessible_name: Some(String::from("Continue")),
            placeholder: None,
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: false,
            attributes: std::collections::BTreeMap::new(),
        }],
    };

    let error = resolve_clickable_element(&page, "button-disabled").unwrap_err();

    assert_eq!(error.code, "element_disabled");
}

#[test]
fn resolve_clickable_element_requires_a_stable_dom_locator() {
    let page = PageModel {
        title: Some(String::from("Example")),
        url: Some(String::from("https://example.com")),
        regions: Vec::new(),
        interactive_elements: vec![InteractiveElement {
            element_id: String::from("button-1"),
            dom_locator: None,
            role: ElementRole::Button,
            tag_name: String::from("button"),
            text: Some(String::from("Continue")),
            accessible_name: Some(String::from("Continue")),
            placeholder: None,
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::new(),
        }],
    };

    let error = resolve_clickable_element(&page, "button-1").unwrap_err();

    assert_eq!(error.code, "missing_dom_locator");
}

#[test]
fn resolve_clickable_element_rejects_blank_and_unknown_ids() {
    let page = fixture_page(vec![InteractiveElement {
        element_id: String::from("button-1"),
        dom_locator: Some(String::from("#button-1")),
        role: ElementRole::Button,
        tag_name: String::from("button"),
        text: Some(String::from("Continue")),
        accessible_name: Some(String::from("Continue")),
        placeholder: None,
        href: None,
        value: None,
        bbox: None,
        visible: true,
        enabled: true,
        attributes: std::collections::BTreeMap::new(),
    }]);

    let blank_error = resolve_clickable_element(&page, "   ").unwrap_err();
    assert_eq!(blank_error.code, "invalid_element_id");

    let unknown_error = resolve_clickable_element(&page, "missing-button").unwrap_err();
    assert_eq!(unknown_error.code, "unknown_element_id");
    assert_eq!(
        unknown_error.details,
        Some(serde_json::json!({ "element_id": "missing-button" }))
    );
}

#[test]
fn test_openai_api_key_connectivity_accepts_valid_response() {
    let (base_url, server) = spawn_openai_models_test_server(
        "200 OK",
        r#"{"object":"list","data":[]}"#,
    );

    let result = test_openai_api_key_connectivity(
        &base_url,
        "blind-browser-test-key",
        Some("org_test"),
        Some("proj_test"),
        5_000,
    );

    server.join().expect("test server should exit cleanly");
    assert!(result.is_ok());
}

#[test]
fn test_openai_api_key_connectivity_reports_http_failures() {
    let (base_url, server) = spawn_openai_models_test_server(
        "401 Unauthorized",
        r#"{"error":{"message":"Incorrect API key provided: sk-proj-test-secret"}}"#,
    );

    let error = test_openai_api_key_connectivity(
        &base_url,
        "blind-browser-test-key",
        Some("org_test"),
        Some("proj_test"),
        5_000,
    )
    .expect_err("request should fail with an HTTP error");

    server.join().expect("test server should exit cleanly");
    assert_eq!(
        error,
        "OpenAI rejected that API key. Check the key and try again, or create one at https://platform.openai.com/account/api-keys."
    );
    assert!(!error.contains("sk-proj"));
}

#[test]
fn fetch_openai_compatible_models_returns_sorted_model_ids() {
    let (base_url, server) = spawn_openai_models_test_server(
        "200 OK",
        r#"{"object":"list","data":[{"id":"gpt-4o-mini"},{"id":"gpt-5.4-mini"},{"id":"gpt-4o-mini"}]}"#,
    );

    let models = fetch_openai_compatible_models(
        &base_url,
        Some("blind-browser-test-key"),
        Some("org_test"),
        Some("proj_test"),
        5_000,
    )
    .expect("model list should load");

    server.join().expect("test server should exit cleanly");
    assert_eq!(models, vec![String::from("gpt-4o-mini"), String::from("gpt-5.4-mini")]);
}

#[test]
fn fetch_openai_compatible_models_rejects_empty_lists() {
    let (base_url, server) = spawn_openai_models_test_server(
        "200 OK",
        r#"{"object":"list","data":[]}"#,
    );

    let error = fetch_openai_compatible_models(
        &base_url,
        Some("blind-browser-test-key"),
        Some("org_test"),
        Some("proj_test"),
        5_000,
    )
    .expect_err("empty model lists should be rejected");

    server.join().expect("test server should exit cleanly");
    assert_eq!(
        error,
        "The endpoint responded successfully but did not return any models."
    );
}
