use super::*;

#[test]
fn resolve_direct_repeat_command_replays_current_region() {
    let agent_state = AgentStateData {
        page_id: Some(String::from("page-1")),
        url: Some(String::from("https://example.com/article")),
        title: Some(String::from("Example article")),
        browser_visibility: BrowserVisibilityMode::Visible,
        browser_history: BrowserHistoryState::default(),
        narration_cursor: Some(NarrationCursor {
            current_region_id: Some(String::from("region-2")),
            current_index: Some(1),
            total_regions: 3,
        }),
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

    let planner_output = resolve_direct_repeat_command(
        "say that again",
        "req-repeat",
        &agent_state,
        &[String::from("repeat")],
    )
    .expect("repeat command should normalize");

    assert_eq!(planner_output.intent.name, IntentName::Repeat);
    assert_eq!(planner_output.selected_skills, vec![String::from("repeat")]);
    assert_eq!(planner_output.steps.len(), 1);
    assert_eq!(planner_output.steps[0].tool_name, ToolName::ReadRegion);
    assert_eq!(
        planner_output.steps[0].arguments.get("region_id"),
        Some(&serde_json::json!("region-2"))
    );
    assert_eq!(
        planner_output.steps[0].arguments.get("interruption_mode"),
        Some(&serde_json::json!(NarrationInterruptionMode::Interrupt))
    );
}

#[test]
fn resolve_direct_repeat_command_reports_missing_current_region() {
    let agent_state = AgentStateData {
        page_id: Some(String::from("page-1")),
        url: Some(String::from("https://example.com/article")),
        title: Some(String::from("Example article")),
        browser_visibility: BrowserVisibilityMode::Visible,
        browser_history: BrowserHistoryState::default(),
        narration_cursor: Some(NarrationCursor::default()),
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

    let planner_output =
        resolve_direct_repeat_command("repeat that", "req-repeat-missing", &agent_state, &[])
            .expect("repeat command should still produce a bounded response");

    assert_eq!(planner_output.intent.name, IntentName::Repeat);
    assert_eq!(planner_output.steps.len(), 1);
    assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
    assert_eq!(
        planner_output.steps[0].arguments.get("status"),
        Some(&serde_json::json!(ReportStatus::NeedsFollowUp))
    );
    assert_eq!(
        planner_output.steps[0].arguments.get("summary"),
        Some(&serde_json::json!(
            "There is no current region to repeat yet."
        ))
    );
    assert_eq!(
        planner_output.steps[0]
            .arguments
            .get("next_recommended_action"),
        Some(&serde_json::json!(
            "Read the page or move to a region first."
        ))
    );
}

#[test]
fn resolve_direct_read_title_command_reports_current_title() {
    let agent_state = AgentStateData {
        page_id: Some(String::from("page-1")),
        url: Some(String::from("https://example.com/article")),
        title: Some(String::from("Example article")),
        browser_visibility: BrowserVisibilityMode::Visible,
        browser_history: BrowserHistoryState::default(),
        narration_cursor: Some(NarrationCursor::default()),
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

    let planner_output = resolve_direct_read_title_command(
        "read the page title",
        "req-read-title",
        &agent_state,
        &[String::from("read_title")],
    )
    .expect("read title command should normalize");

    assert_eq!(planner_output.intent.name, IntentName::ReadTitle);
    assert_eq!(
        planner_output.selected_skills,
        vec![String::from("read_title")]
    );
    assert_eq!(planner_output.steps.len(), 1);
    assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
    assert_eq!(
        planner_output.steps[0].arguments.get("summary"),
        Some(&serde_json::json!("Page title is Example article."))
    );
}

#[test]
fn resolve_direct_read_title_command_reports_missing_title() {
    let agent_state = AgentStateData {
        page_id: Some(String::from("page-1")),
        url: Some(String::from("https://example.com/article")),
        title: None,
        browser_visibility: BrowserVisibilityMode::Visible,
        browser_history: BrowserHistoryState::default(),
        narration_cursor: Some(NarrationCursor::default()),
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

    let planner_output = resolve_direct_read_title_command(
        "what is the title",
        "req-read-title-missing",
        &agent_state,
        &[],
    )
    .expect("missing-title command should still produce a bounded response");

    assert_eq!(planner_output.intent.name, IntentName::ReadTitle);
    assert_eq!(planner_output.steps.len(), 1);
    assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
    assert_eq!(
        planner_output.steps[0].arguments.get("summary"),
        Some(&serde_json::json!(
            "This page does not have a readable title yet."
        ))
    );
}
