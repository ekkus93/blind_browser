use super::*;

#[test]
fn resolve_direct_read_page_command_reads_from_first_region_when_available() {
    let page_model = PageModel {
        title: Some(String::from("Example page")),
        url: Some(String::from("https://example.com/article")),
        regions: vec![
            crate::page_model::PageRegion {
                region_id: String::from("region-1"),
                role: RegionRole::Section,
                label: Some(String::from("Main")),
                text: String::from("Welcome to the article."),
                bbox: None,
                source: crate::page_model::RegionSource::Dom,
            },
            crate::page_model::PageRegion {
                region_id: String::from("region-2"),
                role: RegionRole::Section,
                label: Some(String::from("Details")),
                text: String::from("More details."),
                bbox: None,
                source: crate::page_model::RegionSource::Dom,
            },
        ],
        interactive_elements: Vec::new(),
    };
    let agent_state = AgentStateData {
        page_id: Some(String::from("page-1")),
        url: page_model.url.clone(),
        title: page_model.title.clone(),
        browser_visibility: BrowserVisibilityMode::Visible,
        browser_history: BrowserHistoryState::default(),
        narration_cursor: None,
        speaking: false,
        listening_state: ListeningState::default(),
        audio: RuntimeAudioState::default(),
        last_transcript: None,
        last_tool_call: None,
        pending_confirmation_id: None,
        pending_plan_execution: None,
        tts_model_settings: TtsModelSettings {
            mode: ProviderMode::Local,
            active_profile: Some(String::from("kitten-default")),
            available_profiles: vec![TtsModelOption {
                profile_name: String::from("kitten-default"),
                model_label: String::from("default"),
            }],
        },
        local_tts_model_settings: LocalTtsModelSettings {
            profile_name: Some(String::from("kitten-default")),
            backend: Some(LocalTtsBackend::KittenTtsRs),
            model_id: Some(String::from("default")),
            model_path: Some(String::from("/path/to/kitten/model")),
            default_voice: Some(String::from("Bruno")),
            sample_rate: Some(24_000),
        },
        tts_voice_settings: TtsVoiceSettings {
            mode: ProviderMode::Local,
            active_voice: Some(String::from("Bruno")),
            available_voices: vec![
                TtsVoiceOption {
                    voice_name: String::from("Bella"),
                    display_label: String::from("Bella"),
                },
                TtsVoiceOption {
                    voice_name: String::from("Bruno"),
                    display_label: String::from("Bruno"),
                },
            ],
        },
        tts_provider_settings: TtsProviderSettings {
            active_mode: ProviderMode::Local,
            available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
        },
        asr_provider_settings: AsrProviderSettings {
            active_mode: ProviderMode::Local,
            available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
        },
        local_asr_model_settings: LocalAsrModelSettings {
            profile_name: Some(String::from("whisper-default")),
            backend: Some(LocalAsrBackend::Whisper),
            model_id: Some(String::from("tiny")),
            model_path: Some(String::from("/path/to/whisper/model")),
            language: Some(String::from("en")),
            threads: Some(4),
        },
        remote_planner_settings: RemotePlannerSettings {
            profile_name: Some(String::from("openai-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-5.4-mini")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            api_key_reference_error: None,
            organization_reference: None,
            project: None,
            temperature_milli: Some(200),
            max_output_tokens: Some(1024),
            timeout_ms: Some(30_000),
            ..RemotePlannerSettings::default()
        },
        remote_planner_privacy_status: RemotePlannerPrivacyStatus::default(),
        remote_tts_settings: RemoteTtsSettings {
            profile_name: Some(String::from("openai-tts-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-tts")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            api_key_reference_error: None,
            organization_reference: None,
            project: None,
            voice: Some(String::from("alloy")),
            audio_format: Some(RemoteTtsAudioFormat::Wav),
            timeout_ms: Some(30_000),
            endpoint_is_loopback: None,
            availability_reason: None,
            privacy_network_mode: Default::default(),
            privacy_origin_rule_count: 0,
            privacy_notice: String::new(),
        },
        remote_asr_settings: RemoteAsrSettings {
            profile_name: Some(String::from("openai-transcribe-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-transcribe")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            api_key_reference_error: None,
            organization_reference: None,
            project: None,
            language: Some(String::from("en")),
            temperature_milli: Some(0),
            timeout_ms: Some(30_000),
            endpoint_is_loopback: None,
            availability_reason: None,
            privacy_network_mode: Default::default(),
            privacy_origin_rule_count: 0,
            privacy_notice: String::new(),
        },
        provider_failover_settings: ProviderFailoverSettings {
            planner_available: false,
            tts_available: false,
            asr_available: false,
            summary: String::from(
                "Automatic provider failover is not currently available in the live runtime.",
            ),
        },
        confirmation_settings: ConfirmationSettings {
            confirmation_confidence_threshold: 0.9,
            allow_click_without_confirmation: true,
            always_confirm_submit: true,
        },
        ocr_threshold_settings: OcrThresholdSettings {
            sparse_text_char_threshold: 200,
            sparse_text_region_threshold: 2,
        },
    };

    let planner_output = resolve_direct_read_page_command(
        "read this page",
        "req-read-page",
        Some(&page_model),
        &agent_state,
        &[String::from("read_page")],
    )
    .expect("read-page command should resolve");

    assert_eq!(planner_output.intent.name, IntentName::ReadPage);
    assert_eq!(
        planner_output.selected_skills,
        vec![String::from("read_page")]
    );
    assert_eq!(planner_output.steps.len(), 1);
    assert_eq!(planner_output.steps[0].tool_name, ToolName::ReadRegion);
    assert_eq!(
        planner_output.steps[0].arguments.get("region_id"),
        Some(&serde_json::json!("region-1"))
    );
    assert_eq!(
        planner_output.steps[0].arguments.get("interruption_mode"),
        Some(&serde_json::json!(NarrationInterruptionMode::Interrupt))
    );
}

#[test]
fn resolve_direct_read_page_command_extracts_then_reads_when_regions_missing() {
    let page_model = PageModel {
        title: Some(String::from("Example page")),
        url: Some(String::from("https://example.com/article")),
        regions: Vec::new(),
        interactive_elements: Vec::new(),
    };
    let agent_state = AgentStateData {
        page_id: Some(String::from("page-1")),
        url: page_model.url.clone(),
        title: page_model.title.clone(),
        browser_visibility: BrowserVisibilityMode::Visible,
        browser_history: BrowserHistoryState::default(),
        narration_cursor: None,
        speaking: false,
        listening_state: ListeningState::default(),
        audio: RuntimeAudioState::default(),
        last_transcript: None,
        last_tool_call: None,
        pending_confirmation_id: None,
        pending_plan_execution: None,
        tts_model_settings: TtsModelSettings {
            mode: ProviderMode::Local,
            active_profile: Some(String::from("kitten-default")),
            available_profiles: vec![TtsModelOption {
                profile_name: String::from("kitten-default"),
                model_label: String::from("default"),
            }],
        },
        local_tts_model_settings: LocalTtsModelSettings {
            profile_name: Some(String::from("kitten-default")),
            backend: Some(LocalTtsBackend::KittenTtsRs),
            model_id: Some(String::from("default")),
            model_path: Some(String::from("/path/to/kitten/model")),
            default_voice: Some(String::from("Bruno")),
            sample_rate: Some(24_000),
        },
        tts_voice_settings: TtsVoiceSettings {
            mode: ProviderMode::Local,
            active_voice: Some(String::from("Bruno")),
            available_voices: vec![
                TtsVoiceOption {
                    voice_name: String::from("Bella"),
                    display_label: String::from("Bella"),
                },
                TtsVoiceOption {
                    voice_name: String::from("Bruno"),
                    display_label: String::from("Bruno"),
                },
            ],
        },
        tts_provider_settings: TtsProviderSettings {
            active_mode: ProviderMode::Local,
            available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
        },
        asr_provider_settings: AsrProviderSettings {
            active_mode: ProviderMode::Local,
            available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
        },
        local_asr_model_settings: LocalAsrModelSettings {
            profile_name: Some(String::from("whisper-default")),
            backend: Some(LocalAsrBackend::Whisper),
            model_id: Some(String::from("tiny")),
            model_path: Some(String::from("/path/to/whisper/model")),
            language: Some(String::from("en")),
            threads: Some(4),
        },
        remote_planner_settings: RemotePlannerSettings {
            profile_name: Some(String::from("openai-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-5.4-mini")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            api_key_reference_error: None,
            organization_reference: None,
            project: None,
            temperature_milli: Some(200),
            max_output_tokens: Some(1024),
            timeout_ms: Some(30_000),
            ..RemotePlannerSettings::default()
        },
        remote_planner_privacy_status: RemotePlannerPrivacyStatus::default(),
        remote_tts_settings: RemoteTtsSettings {
            profile_name: Some(String::from("openai-tts-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-tts")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            api_key_reference_error: None,
            organization_reference: None,
            project: None,
            voice: Some(String::from("alloy")),
            audio_format: Some(RemoteTtsAudioFormat::Wav),
            timeout_ms: Some(30_000),
            endpoint_is_loopback: None,
            availability_reason: None,
            privacy_network_mode: Default::default(),
            privacy_origin_rule_count: 0,
            privacy_notice: String::new(),
        },
        remote_asr_settings: RemoteAsrSettings {
            profile_name: Some(String::from("openai-transcribe-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-transcribe")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            api_key_reference_error: None,
            organization_reference: None,
            project: None,
            language: Some(String::from("en")),
            temperature_milli: Some(0),
            timeout_ms: Some(30_000),
            endpoint_is_loopback: None,
            availability_reason: None,
            privacy_network_mode: Default::default(),
            privacy_origin_rule_count: 0,
            privacy_notice: String::new(),
        },
        provider_failover_settings: ProviderFailoverSettings {
            planner_available: false,
            tts_available: false,
            asr_available: false,
            summary: String::from(
                "Automatic provider failover is not currently available in the live runtime.",
            ),
        },
        confirmation_settings: ConfirmationSettings {
            confirmation_confidence_threshold: 0.9,
            allow_click_without_confirmation: true,
            always_confirm_submit: true,
        },
        ocr_threshold_settings: OcrThresholdSettings {
            sparse_text_char_threshold: 200,
            sparse_text_region_threshold: 2,
        },
    };

    let planner_output = resolve_direct_read_page_command(
        "read page",
        "req-read-page-extract",
        Some(&page_model),
        &agent_state,
        &[String::from("read_page")],
    )
    .expect("read-page command should resolve");

    assert_eq!(planner_output.steps.len(), 2);
    assert_eq!(
        planner_output.steps[0].tool_name,
        ToolName::ExtractPageModel
    );
    assert_eq!(
        planner_output.steps[0].arguments.get("use_dom_extraction"),
        Some(&serde_json::json!(true))
    );
    assert_eq!(planner_output.steps[1].tool_name, ToolName::ReadNextRegion);
    assert_eq!(
        planner_output.steps[1].arguments.get("interruption_mode"),
        Some(&serde_json::json!(NarrationInterruptionMode::Interrupt))
    );
}

#[test]
fn resolve_direct_read_page_command_reports_missing_active_page() {
    let agent_state = AgentStateData {
        page_id: None,
        url: None,
        title: None,
        browser_visibility: BrowserVisibilityMode::Visible,
        browser_history: BrowserHistoryState::default(),
        narration_cursor: None,
        speaking: false,
        listening_state: ListeningState::default(),
        audio: RuntimeAudioState::default(),
        last_transcript: None,
        last_tool_call: None,
        pending_confirmation_id: None,
        pending_plan_execution: None,
        tts_model_settings: TtsModelSettings {
            mode: ProviderMode::Local,
            active_profile: Some(String::from("kitten-default")),
            available_profiles: vec![TtsModelOption {
                profile_name: String::from("kitten-default"),
                model_label: String::from("default"),
            }],
        },
        local_tts_model_settings: LocalTtsModelSettings {
            profile_name: Some(String::from("kitten-default")),
            backend: Some(LocalTtsBackend::KittenTtsRs),
            model_id: Some(String::from("default")),
            model_path: Some(String::from("/path/to/kitten/model")),
            default_voice: Some(String::from("Bruno")),
            sample_rate: Some(24_000),
        },
        tts_voice_settings: TtsVoiceSettings {
            mode: ProviderMode::Local,
            active_voice: Some(String::from("Bruno")),
            available_voices: vec![
                TtsVoiceOption {
                    voice_name: String::from("Bella"),
                    display_label: String::from("Bella"),
                },
                TtsVoiceOption {
                    voice_name: String::from("Bruno"),
                    display_label: String::from("Bruno"),
                },
            ],
        },
        tts_provider_settings: TtsProviderSettings {
            active_mode: ProviderMode::Local,
            available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
        },
        asr_provider_settings: AsrProviderSettings {
            active_mode: ProviderMode::Local,
            available_modes: vec![ProviderMode::Local, ProviderMode::Remote],
        },
        local_asr_model_settings: LocalAsrModelSettings {
            profile_name: Some(String::from("whisper-default")),
            backend: Some(LocalAsrBackend::Whisper),
            model_id: Some(String::from("tiny")),
            model_path: Some(String::from("/path/to/whisper/model")),
            language: Some(String::from("en")),
            threads: Some(4),
        },
        remote_planner_settings: RemotePlannerSettings {
            profile_name: Some(String::from("openai-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-5.4-mini")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            api_key_reference_error: None,
            organization_reference: None,
            project: None,
            temperature_milli: Some(200),
            max_output_tokens: Some(1024),
            timeout_ms: Some(30_000),
            ..RemotePlannerSettings::default()
        },
        remote_planner_privacy_status: RemotePlannerPrivacyStatus::default(),
        remote_tts_settings: RemoteTtsSettings {
            profile_name: Some(String::from("openai-tts-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-tts")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            api_key_reference_error: None,
            organization_reference: None,
            project: None,
            voice: Some(String::from("alloy")),
            audio_format: Some(RemoteTtsAudioFormat::Wav),
            timeout_ms: Some(30_000),
            endpoint_is_loopback: None,
            availability_reason: None,
            privacy_network_mode: Default::default(),
            privacy_origin_rule_count: 0,
            privacy_notice: String::new(),
        },
        remote_asr_settings: RemoteAsrSettings {
            profile_name: Some(String::from("openai-transcribe-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-transcribe")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            api_key_reference_error: None,
            organization_reference: None,
            project: None,
            language: Some(String::from("en")),
            temperature_milli: Some(0),
            timeout_ms: Some(30_000),
            endpoint_is_loopback: None,
            availability_reason: None,
            privacy_network_mode: Default::default(),
            privacy_origin_rule_count: 0,
            privacy_notice: String::new(),
        },
        provider_failover_settings: ProviderFailoverSettings {
            planner_available: false,
            tts_available: false,
            asr_available: false,
            summary: String::from(
                "Automatic provider failover is not currently available in the live runtime.",
            ),
        },
        confirmation_settings: ConfirmationSettings {
            confirmation_confidence_threshold: 0.9,
            allow_click_without_confirmation: true,
            always_confirm_submit: true,
        },
        ocr_threshold_settings: OcrThresholdSettings {
            sparse_text_char_threshold: 200,
            sparse_text_region_threshold: 2,
        },
    };

    let planner_output = resolve_direct_read_page_command(
        "read current page",
        "req-read-page-missing",
        None,
        &agent_state,
        &[String::from("read_page")],
    )
    .expect("read-page command should resolve");

    assert_eq!(planner_output.steps.len(), 1);
    assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
    assert_eq!(
        planner_output.steps[0].arguments.get("status"),
        Some(&serde_json::json!(ReportStatus::NeedsFollowUp))
    );
}
