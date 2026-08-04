use super::*;

#[test]
fn resolve_direct_status_query_command_reports_current_url() {
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
    let runtime_status = GetRuntimeStatusData {
        page_id: agent_state.page_id.clone(),
        url: agent_state.url.clone(),
        title: agent_state.title.clone(),
        browser_visibility: agent_state.browser_visibility,
        browser_history: agent_state.browser_history.clone(),
        listening_state: agent_state.listening_state.clone(),
        speaking: agent_state.speaking,
        audio: agent_state.audio.clone(),
        pending_confirmation_id: None,
        pending_plan_execution: None,
        provider_modes: None,
        skill_discovery_diagnostics: Default::default(),
    };

    let planner_output = resolve_direct_status_query_command(
        "what page am i on",
        "req-current-url",
        &agent_state,
        &runtime_status,
        &[String::from("get_current_url")],
    )
    .expect("current url query should normalize");

    assert_eq!(planner_output.intent.name, IntentName::GetCurrentUrl);
    assert_eq!(
        planner_output.selected_skills,
        vec![String::from("get_current_url")]
    );
    assert_eq!(planner_output.steps[0].tool_name, ToolName::GetAgentState);
    assert_eq!(
        planner_output.steps[1].arguments.get("summary"),
        Some(&serde_json::json!(
            "Current page is Example article at https://example.com/article."
        ))
    );
}

#[test]
fn resolve_direct_status_query_command_reports_back_history_availability() {
    let agent_state = AgentStateData {
        page_id: Some(String::from("page-1")),
        url: Some(String::from("https://example.com/article")),
        title: Some(String::from("Example article")),
        browser_visibility: BrowserVisibilityMode::Visible,
        browser_history: BrowserHistoryState {
            can_go_back: true,
            can_go_forward: false,
            current_entry_index: Some(1),
            entry_count: 2,
        },
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
    let runtime_status = GetRuntimeStatusData {
        page_id: agent_state.page_id.clone(),
        url: agent_state.url.clone(),
        title: agent_state.title.clone(),
        browser_visibility: agent_state.browser_visibility,
        browser_history: agent_state.browser_history.clone(),
        listening_state: agent_state.listening_state.clone(),
        speaking: agent_state.speaking,
        audio: agent_state.audio.clone(),
        pending_confirmation_id: None,
        pending_plan_execution: None,
        provider_modes: None,
        skill_discovery_diagnostics: Default::default(),
    };

    let planner_output = resolve_direct_status_query_command(
        "can i go back",
        "req-back-status",
        &agent_state,
        &runtime_status,
        &[String::from("get_status")],
    )
    .expect("back history query should normalize");

    assert_eq!(planner_output.intent.name, IntentName::GetStatus);
    assert_eq!(
        planner_output.steps[0].tool_name,
        ToolName::GetRuntimeStatus
    );
    assert_eq!(
        planner_output.steps[1].arguments.get("summary"),
        Some(&serde_json::json!("Back navigation is available."))
    );
}

#[test]
fn resolve_direct_status_query_command_reports_listening_state() {
    let agent_state = AgentStateData {
        page_id: None,
        url: None,
        title: None,
        browser_visibility: BrowserVisibilityMode::Headless,
        browser_history: BrowserHistoryState::default(),
        narration_cursor: Some(NarrationCursor::default()),
        speaking: false,
        listening_state: ListeningState {
            is_listening: true,
            push_to_talk_enabled: true,
        },
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
    let runtime_status = GetRuntimeStatusData {
        page_id: None,
        url: None,
        title: None,
        browser_visibility: BrowserVisibilityMode::Headless,
        browser_history: BrowserHistoryState::default(),
        listening_state: agent_state.listening_state.clone(),
        speaking: false,
        audio: agent_state.audio.clone(),
        pending_confirmation_id: None,
        pending_plan_execution: None,
        provider_modes: None,
        skill_discovery_diagnostics: Default::default(),
    };

    let planner_output = resolve_direct_status_query_command(
        "are you listening",
        "req-listening-status",
        &agent_state,
        &runtime_status,
        &[String::from("get_status")],
    )
    .expect("listening query should normalize");

    assert_eq!(planner_output.intent.name, IntentName::GetStatus);
    assert_eq!(
        planner_output.steps[1].arguments.get("summary"),
        Some(&serde_json::json!("Listening is on."))
    );

    let fuzzy_planner_output = resolve_direct_status_query_command(
        "are you listenin",
        "req-listening-status-fuzzy",
        &agent_state,
        &runtime_status,
        &[String::from("get_status")],
    )
    .expect("fuzzy listening query should normalize");

    assert_eq!(
        fuzzy_planner_output.steps[1].arguments.get("summary"),
        Some(&serde_json::json!("Listening is on."))
    );
}
