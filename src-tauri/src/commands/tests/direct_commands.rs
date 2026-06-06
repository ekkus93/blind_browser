use super::*;

#[test]
fn resolve_direct_audio_command_normalizes_absolute_volume_percent() {
    let planner_output = resolve_direct_audio_command(
        "set volume to 70 percent",
        "req-volume",
        1.0,
        1.0,
        &[String::from("set_volume")],
    )
    .expect("volume command should normalize");

    assert_eq!(planner_output.intent.name, IntentName::SetPlaybackVolume);
    assert_eq!(
        planner_output.selected_skills,
        vec![String::from("set_volume")]
    );
    assert_eq!(planner_output.steps.len(), 2);
    assert_eq!(
        planner_output.steps[0].tool_name,
        ToolName::SetPlaybackVolume
    );
    let volume = planner_output.steps[0]
        .arguments
        .get("volume")
        .and_then(serde_json::Value::as_f64)
        .expect("volume should be numeric");
    assert!((volume - 0.7).abs() < 0.000_001);
    assert_eq!(planner_output.steps[1].tool_name, ToolName::ReportResult);
    assert_eq!(
        planner_output.steps[1].arguments.get("summary"),
        Some(&serde_json::json!("Playback volume set to 70%."))
    );

    let fuzzy_planner_output = resolve_direct_audio_command(
        "set volum to 70 percent",
        "req-volume-fuzzy",
        1.0,
        1.0,
        &[String::from("set_volume")],
    )
    .expect("fuzzy volume command should normalize");

    let fuzzy_volume = fuzzy_planner_output.steps[0]
        .arguments
        .get("volume")
        .and_then(serde_json::Value::as_f64)
        .expect("fuzzy volume should be numeric");
    assert!((fuzzy_volume - 0.7).abs() < 0.000_001);
}

#[test]
fn resolve_direct_audio_command_applies_large_relative_speed_step() {
    let planner_output = resolve_direct_audio_command(
        "go faster a lot",
        "req-speed",
        1.0,
        1.0,
        &[String::from("increase_playback_speed")],
    )
    .expect("speed command should normalize");

    assert_eq!(planner_output.intent.name, IntentName::SetPlaybackSpeed);
    assert_eq!(
        planner_output.selected_skills,
        vec![String::from("increase_playback_speed")]
    );
    assert_eq!(
        planner_output.steps[0].arguments.get("speed"),
        Some(&serde_json::json!(1.5))
    );
    assert_eq!(
        planner_output.steps[1].arguments.get("summary"),
        Some(&serde_json::json!("Playback speed set to 1.5x."))
    );
}

#[test]
fn resolve_direct_audio_command_reports_current_speed_for_queries() {
    let planner_output = resolve_direct_audio_command(
        "tell me the speed",
        "req-speed-query",
        0.8,
        1.25,
        &[String::from("get_playback_speed")],
    )
    .expect("speed query should normalize");

    assert_eq!(planner_output.intent.name, IntentName::GetPlaybackSpeed);
    assert_eq!(
        planner_output.selected_skills,
        vec![String::from("get_playback_speed")]
    );
    assert_eq!(planner_output.steps.len(), 1);
    assert_eq!(planner_output.steps[0].tool_name, ToolName::ReportResult);
    assert_eq!(
        planner_output.steps[0].arguments.get("summary"),
        Some(&serde_json::json!("Playback speed is 1.25x."))
    );
}

#[test]
fn resolve_direct_browser_visibility_command_normalizes_headless_phrase() {
    let planner_output = resolve_direct_browser_visibility_command(
        "go headless",
        "req-headless",
        BrowserVisibilityMode::Visible,
        &[String::from("toggle_browser_visibility")],
    )
    .expect("visibility command should normalize");

    assert_eq!(planner_output.intent.name, IntentName::SetBrowserVisibility);
    assert_eq!(
        planner_output.selected_skills,
        vec![String::from("toggle_browser_visibility")]
    );
    assert_eq!(planner_output.steps.len(), 2);
    assert_eq!(
        planner_output.steps[0].arguments.get("mode"),
        Some(&serde_json::json!(BrowserVisibilityMode::Headless))
    );
    assert_eq!(
        planner_output.steps[1].arguments.get("summary"),
        Some(&serde_json::json!("Browser mode set to headless."))
    );

    let fuzzy_planner_output = resolve_direct_browser_visibility_command(
        "show the browsr",
        "req-visible-fuzzy",
        BrowserVisibilityMode::Headless,
        &[String::from("toggle_browser_visibility")],
    )
    .expect("fuzzy visibility command should normalize");

    assert_eq!(
        fuzzy_planner_output.steps[0].arguments.get("mode"),
        Some(&serde_json::json!(BrowserVisibilityMode::Visible))
    );
}

#[test]
fn resolve_direct_browser_visibility_command_toggles_when_requested() {
    let planner_output = resolve_direct_browser_visibility_command(
        "toggle browser visibility",
        "req-toggle",
        BrowserVisibilityMode::Headless,
        &[String::from("toggle_browser_visibility")],
    )
    .expect("toggle visibility command should normalize");

    assert_eq!(
        planner_output.steps[0].arguments.get("mode"),
        Some(&serde_json::json!(BrowserVisibilityMode::Visible))
    );
    assert_eq!(
        planner_output.steps[1].arguments.get("summary"),
        Some(&serde_json::json!("Browser mode set to visible."))
    );
}

#[test]
fn resolve_direct_navigation_readback_command_builds_history_and_reload_plans() {
    let go_back_plan =
        resolve_direct_navigation_readback_command("back", "req-back", &[String::from("go_back")])
            .expect("back command should normalize");

    assert_eq!(go_back_plan.intent.name, IntentName::GoBack);
    assert_eq!(go_back_plan.selected_skills, vec![String::from("go_back")]);
    assert_eq!(go_back_plan.steps.len(), 1);
    assert_eq!(go_back_plan.steps[0].tool_name, ToolName::GoBack);
    assert_eq!(
        go_back_plan.steps[0].arguments.get("steps"),
        Some(&serde_json::json!(1))
    );
    assert_eq!(
        go_back_plan.steps[0].arguments.get("wait_for_load_state"),
        Some(&serde_json::json!(LoadState::Load))
    );

    let reload_plan = resolve_direct_navigation_readback_command(
        "refesh page",
        "req-reload",
        &[String::from("reload_page")],
    )
    .expect("reload command should normalize");

    assert_eq!(reload_plan.intent.name, IntentName::ReloadPage);
    assert_eq!(
        reload_plan.selected_skills,
        vec![String::from("reload_page")]
    );
    assert_eq!(reload_plan.steps[0].tool_name, ToolName::ReloadPage);
    assert_eq!(
        reload_plan.steps[0].arguments.get("mode"),
        Some(&serde_json::json!(ReloadMode::Standard))
    );
}

#[test]
fn resolve_direct_navigation_readback_command_builds_reading_and_stop_plans() {
    let next_plan = resolve_direct_navigation_readback_command(
        "continue reading",
        "req-next",
        &[String::from("read_next")],
    )
    .expect("next command should normalize");

    assert_eq!(next_plan.intent.name, IntentName::ReadNext);
    assert_eq!(next_plan.selected_skills, vec![String::from("read_next")]);
    assert_eq!(next_plan.steps[0].tool_name, ToolName::ReadNextRegion);
    assert_eq!(
        next_plan.steps[0].arguments.get("interruption_mode"),
        Some(&serde_json::json!(NarrationInterruptionMode::Interrupt))
    );

    let previous_plan = resolve_direct_navigation_readback_command(
        "prevous section",
        "req-previous",
        &[String::from("read_previous")],
    )
    .expect("previous command should normalize");

    assert_eq!(previous_plan.intent.name, IntentName::ReadPrevious);
    assert_eq!(
        previous_plan.selected_skills,
        vec![String::from("read_previous")]
    );
    assert_eq!(
        previous_plan.steps[0].tool_name,
        ToolName::ReadPreviousRegion
    );

    let stop_plan = resolve_direct_navigation_readback_command(
        "stpo reading",
        "req-stop",
        &[String::from("stop_reading")],
    )
    .expect("stop command should normalize");

    assert_eq!(stop_plan.intent.name, IntentName::Stop);
    assert_eq!(
        stop_plan.selected_skills,
        vec![String::from("stop_reading")]
    );
    assert_eq!(stop_plan.steps[0].tool_name, ToolName::StopSpeaking);
}

#[test]
fn resolve_direct_voice_input_command_builds_start_and_stop_listening_plans() {
    let start_plan = resolve_direct_voice_input_command(
        "start listening",
        "req-start-listening",
        &[String::from("start_listening")],
    )
    .expect("start listening command should normalize");

    assert_eq!(start_plan.intent.name, IntentName::StartListening);
    assert_eq!(
        start_plan.selected_skills,
        vec![String::from("start_listening")]
    );
    assert_eq!(start_plan.steps[0].tool_name, ToolName::StartListening);

    let stop_plan = resolve_direct_voice_input_command(
        "stop listenin",
        "req-stop-listening",
        &[String::from("stop_listening")],
    )
    .expect("stop listening command should normalize");

    assert_eq!(stop_plan.intent.name, IntentName::StopListening);
    assert_eq!(
        stop_plan.selected_skills,
        vec![String::from("stop_listening")]
    );
    assert_eq!(stop_plan.steps[0].tool_name, ToolName::StopListening);
}

#[test]
fn resolve_direct_voice_input_command_builds_transcribe_plan() {
    let planner_output = resolve_direct_voice_input_command(
        "what did i just say",
        "req-transcribe",
        &[String::from("transcribe_command")],
    )
    .expect("transcribe command should normalize");

    assert_eq!(planner_output.intent.name, IntentName::TranscribeCommand);
    assert_eq!(
        planner_output.selected_skills,
        vec![String::from("transcribe_command")]
    );
    assert_eq!(planner_output.steps.len(), 1);
    assert_eq!(
        planner_output.steps[0].tool_name,
        ToolName::TranscribeCommand
    );
    assert_eq!(
        planner_output.steps[0].arguments.get("stop_mode"),
        Some(&serde_json::json!(TranscriptionStopMode::AutoStop))
    );
    assert_eq!(
        planner_output.steps[0].arguments.get("max_duration_ms"),
        Some(&serde_json::Value::Null)
    );
}

#[test]
fn resolve_direct_open_url_command_normalizes_spoken_and_absolute_urls() {
    let spoken_plan = resolve_direct_open_url_command(
        "open github dot com slash features",
        "req-open-spoken",
        &[String::from("open_url")],
    )
    .expect("spoken open-url command should normalize");

    assert_eq!(spoken_plan.intent.name, IntentName::OpenUrl);
    assert_eq!(spoken_plan.selected_skills, vec![String::from("open_url")]);
    assert_eq!(spoken_plan.steps.len(), 1);
    assert_eq!(spoken_plan.steps[0].tool_name, ToolName::OpenUrl);
    assert_eq!(
        spoken_plan.steps[0].arguments.get("url"),
        Some(&serde_json::json!("https://github.com/features"))
    );
    assert_eq!(
        spoken_plan.steps[0].arguments.get("wait_for_load_state"),
        Some(&serde_json::json!(LoadState::Load))
    );

    let localhost_plan = resolve_direct_open_url_command(
        "visit localhost colon 3000",
        "req-open-localhost",
        &[String::from("open_url")],
    )
    .expect("localhost command should normalize");

    assert_eq!(
        localhost_plan.steps[0].arguments.get("url"),
        Some(&serde_json::json!("http://localhost:3000"))
    );

    let absolute_plan = resolve_direct_open_url_command(
        "go to https://example.com/docs",
        "req-open-absolute",
        &[String::from("open_url")],
    )
    .expect("absolute open-url command should normalize");

    assert_eq!(
        absolute_plan.steps[0].arguments.get("url"),
        Some(&serde_json::json!("https://example.com/docs"))
    );
}

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
            organization_reference: None,
            project: None,
            temperature_milli: Some(200),
            max_output_tokens: Some(1024),
            timeout_ms: Some(30_000),
        },
        remote_tts_settings: RemoteTtsSettings {
            profile_name: Some(String::from("openai-tts-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-tts")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            organization_reference: None,
            project: None,
            voice: Some(String::from("alloy")),
            audio_format: Some(RemoteTtsAudioFormat::Wav),
            timeout_ms: Some(30_000),
        },
        remote_asr_settings: RemoteAsrSettings {
            profile_name: Some(String::from("openai-transcribe-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-transcribe")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            organization_reference: None,
            project: None,
            language: Some(String::from("en")),
            temperature_milli: Some(0),
            timeout_ms: Some(30_000),
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
            organization_reference: None,
            project: None,
            temperature_milli: Some(200),
            max_output_tokens: Some(1024),
            timeout_ms: Some(30_000),
        },
        remote_tts_settings: RemoteTtsSettings {
            profile_name: Some(String::from("openai-tts-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-tts")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            organization_reference: None,
            project: None,
            voice: Some(String::from("alloy")),
            audio_format: Some(RemoteTtsAudioFormat::Wav),
            timeout_ms: Some(30_000),
        },
        remote_asr_settings: RemoteAsrSettings {
            profile_name: Some(String::from("openai-transcribe-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-transcribe")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            organization_reference: None,
            project: None,
            language: Some(String::from("en")),
            temperature_milli: Some(0),
            timeout_ms: Some(30_000),
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
            organization_reference: None,
            project: None,
            temperature_milli: Some(200),
            max_output_tokens: Some(1024),
            timeout_ms: Some(30_000),
        },
        remote_tts_settings: RemoteTtsSettings {
            profile_name: Some(String::from("openai-tts-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-tts")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            organization_reference: None,
            project: None,
            voice: Some(String::from("alloy")),
            audio_format: Some(RemoteTtsAudioFormat::Wav),
            timeout_ms: Some(30_000),
        },
        remote_asr_settings: RemoteAsrSettings {
            profile_name: Some(String::from("openai-transcribe-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-transcribe")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            organization_reference: None,
            project: None,
            language: Some(String::from("en")),
            temperature_milli: Some(0),
            timeout_ms: Some(30_000),
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
            organization_reference: None,
            project: None,
            temperature_milli: Some(200),
            max_output_tokens: Some(1024),
            timeout_ms: Some(30_000),
        },
        remote_tts_settings: RemoteTtsSettings {
            profile_name: Some(String::from("openai-tts-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-tts")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            organization_reference: None,
            project: None,
            voice: Some(String::from("alloy")),
            audio_format: Some(RemoteTtsAudioFormat::Wav),
            timeout_ms: Some(30_000),
        },
        remote_asr_settings: RemoteAsrSettings {
            profile_name: Some(String::from("openai-transcribe-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-transcribe")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            organization_reference: None,
            project: None,
            language: Some(String::from("en")),
            temperature_milli: Some(0),
            timeout_ms: Some(30_000),
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
            organization_reference: None,
            project: None,
            temperature_milli: Some(200),
            max_output_tokens: Some(1024),
            timeout_ms: Some(30_000),
        },
        remote_tts_settings: RemoteTtsSettings {
            profile_name: Some(String::from("openai-tts-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-tts")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            organization_reference: None,
            project: None,
            voice: Some(String::from("alloy")),
            audio_format: Some(RemoteTtsAudioFormat::Wav),
            timeout_ms: Some(30_000),
        },
        remote_asr_settings: RemoteAsrSettings {
            profile_name: Some(String::from("openai-transcribe-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-transcribe")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            organization_reference: None,
            project: None,
            language: Some(String::from("en")),
            temperature_milli: Some(0),
            timeout_ms: Some(30_000),
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
            organization_reference: None,
            project: None,
            temperature_milli: Some(200),
            max_output_tokens: Some(1024),
            timeout_ms: Some(30_000),
        },
        remote_tts_settings: RemoteTtsSettings {
            profile_name: Some(String::from("openai-tts-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-tts")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            organization_reference: None,
            project: None,
            voice: Some(String::from("alloy")),
            audio_format: Some(RemoteTtsAudioFormat::Wav),
            timeout_ms: Some(30_000),
        },
        remote_asr_settings: RemoteAsrSettings {
            profile_name: Some(String::from("openai-transcribe-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-transcribe")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            organization_reference: None,
            project: None,
            language: Some(String::from("en")),
            temperature_milli: Some(0),
            timeout_ms: Some(30_000),
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
            organization_reference: None,
            project: None,
            temperature_milli: Some(200),
            max_output_tokens: Some(1024),
            timeout_ms: Some(30_000),
        },
        remote_tts_settings: RemoteTtsSettings {
            profile_name: Some(String::from("openai-tts-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-tts")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            organization_reference: None,
            project: None,
            voice: Some(String::from("alloy")),
            audio_format: Some(RemoteTtsAudioFormat::Wav),
            timeout_ms: Some(30_000),
        },
        remote_asr_settings: RemoteAsrSettings {
            profile_name: Some(String::from("openai-transcribe-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-transcribe")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            organization_reference: None,
            project: None,
            language: Some(String::from("en")),
            temperature_milli: Some(0),
            timeout_ms: Some(30_000),
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
            organization_reference: None,
            project: None,
            temperature_milli: Some(200),
            max_output_tokens: Some(1024),
            timeout_ms: Some(30_000),
        },
        remote_tts_settings: RemoteTtsSettings {
            profile_name: Some(String::from("openai-tts-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-tts")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            organization_reference: None,
            project: None,
            voice: Some(String::from("alloy")),
            audio_format: Some(RemoteTtsAudioFormat::Wav),
            timeout_ms: Some(30_000),
        },
        remote_asr_settings: RemoteAsrSettings {
            profile_name: Some(String::from("openai-transcribe-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-transcribe")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            organization_reference: None,
            project: None,
            language: Some(String::from("en")),
            temperature_milli: Some(0),
            timeout_ms: Some(30_000),
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
            organization_reference: None,
            project: None,
            temperature_milli: Some(200),
            max_output_tokens: Some(1024),
            timeout_ms: Some(30_000),
        },
        remote_tts_settings: RemoteTtsSettings {
            profile_name: Some(String::from("openai-tts-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-tts")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            organization_reference: None,
            project: None,
            voice: Some(String::from("alloy")),
            audio_format: Some(RemoteTtsAudioFormat::Wav),
            timeout_ms: Some(30_000),
        },
        remote_asr_settings: RemoteAsrSettings {
            profile_name: Some(String::from("openai-transcribe-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-transcribe")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            organization_reference: None,
            project: None,
            language: Some(String::from("en")),
            temperature_milli: Some(0),
            timeout_ms: Some(30_000),
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
            organization_reference: None,
            project: None,
            temperature_milli: Some(200),
            max_output_tokens: Some(1024),
            timeout_ms: Some(30_000),
        },
        remote_tts_settings: RemoteTtsSettings {
            profile_name: Some(String::from("openai-tts-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-tts")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            organization_reference: None,
            project: None,
            voice: Some(String::from("alloy")),
            audio_format: Some(RemoteTtsAudioFormat::Wav),
            timeout_ms: Some(30_000),
        },
        remote_asr_settings: RemoteAsrSettings {
            profile_name: Some(String::from("openai-transcribe-default")),
            provider: Some(RemoteProviderLabel::OpenAi),
            base_url: Some(String::from("https://api.openai.com/v1")),
            model: Some(String::from("gpt-4o-mini-transcribe")),
            api_key_reference: Some(String::from("Environment variable: OPENAI_API_KEY")),
            api_key_masked_value: None,
            organization_reference: None,
            project: None,
            language: Some(String::from("en")),
            temperature_milli: Some(0),
            timeout_ms: Some(30_000),
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

