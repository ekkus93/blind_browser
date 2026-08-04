use super::*;

#[derive(Clone, Copy)]
pub enum PlannerSkillFixtureResolver {
    Audio,
    NavigationReadback,
    ReadPage,
    StatusQuery,
}

pub struct PlannerSkillFixture {
    pub name: &'static str,
    pub transcript: &'static str,
    pub resolver: PlannerSkillFixtureResolver,
    pub agent_state: AgentStateData,
    pub page_model: Option<PageModel>,
    pub expected_intent: IntentName,
    pub expected_selected_skills: Vec<&'static str>,
    pub expected_tool_sequence: Vec<ToolName>,
}

pub fn fixture_agent_state() -> AgentStateData {
    AgentStateData {
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
        narration_cursor: Some(NarrationCursor {
            current_region_id: Some(String::from("region-1")),
            current_index: Some(0),
            total_regions: 2,
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
    }
}

pub fn fixture_runtime_status(agent_state: &AgentStateData) -> GetRuntimeStatusData {
    GetRuntimeStatusData {
        page_id: agent_state.page_id.clone(),
        url: agent_state.url.clone(),
        title: agent_state.title.clone(),
        browser_visibility: agent_state.browser_visibility,
        browser_history: agent_state.browser_history.clone(),
        listening_state: agent_state.listening_state.clone(),
        speaking: agent_state.speaking,
        audio: agent_state.audio.clone(),
        pending_confirmation_id: agent_state.pending_confirmation_id.clone(),
        pending_plan_execution: agent_state.pending_plan_execution.clone(),
        provider_modes: None,
        skill_discovery_diagnostics: Default::default(),
    }
}

pub fn fixture_page_model_without_regions() -> PageModel {
    PageModel {
        title: Some(String::from("Example article")),
        url: Some(String::from("https://example.com/article")),
        regions: Vec::new(),
        interactive_elements: Vec::new(),
    }
}

pub fn fixture_agent_state_for_page(title: &str, url: &str) -> AgentStateData {
    let mut agent_state = fixture_agent_state();
    agent_state.title = Some(String::from(title));
    agent_state.url = Some(String::from(url));
    agent_state
}

pub fn fixture_problematic_article_page_without_regions() -> PageModel {
    PageModel {
        title: Some(String::from("Metro news | Night trains finally return")),
        url: Some(String::from(
            "https://news.example.com/city/night-trains-return",
        )),
        regions: Vec::new(),
        interactive_elements: vec![
            InteractiveElement {
                element_id: String::from("link-skip"),
                dom_locator: Some(String::from("#skip-link")),
                role: crate::page_model::ElementRole::Link,
                tag_name: String::from("a"),
                text: Some(String::from("Skip to content")),
                accessible_name: Some(String::from("Skip to content")),
                placeholder: None,
                href: Some(String::from("#content")),
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
            InteractiveElement {
                element_id: String::from("button-cookie-accept"),
                dom_locator: Some(String::from("#cookie-accept")),
                role: crate::page_model::ElementRole::Button,
                tag_name: String::from("button"),
                text: Some(String::from("Accept")),
                accessible_name: Some(String::from("Accept cookies")),
                placeholder: None,
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
            InteractiveElement {
                element_id: String::from("button-subscribe"),
                dom_locator: Some(String::from("#subscribe")),
                role: crate::page_model::ElementRole::Button,
                tag_name: String::from("button"),
                text: Some(String::from("Subscribe")),
                accessible_name: Some(String::from("Subscribe to metro news")),
                placeholder: None,
                href: None,
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            },
        ],
    }
}

pub fn fixture_problematic_docs_agent_state() -> AgentStateData {
    fixture_agent_state_for_page(
        "Blind Browser docs | Voice commands",
        "https://docs.example.com/blind-browser/voice-commands?ref=sidebar",
    )
}
