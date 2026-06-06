use super::*;

pub fn unique_temp_path(label: &str) -> std::path::PathBuf {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "blind-browser-{label}-{}-{timestamp}",
        std::process::id()
    ))
}

pub fn write_skill_document(root: &std::path::Path, skill_name: &str, content: &str) {
    let skill_dir = root.join(skill_name);
    std::fs::create_dir_all(&skill_dir).expect("skill directory should be created");
    std::fs::write(skill_dir.join("SKILL.md"), content).expect("skill document should be written");
}

pub struct MockExecutor {
    pub last_open_url: Option<String>,
    pub last_go_back_request: Option<GoBackInput>,
    pub last_go_forward_request: Option<GoForwardInput>,
    pub last_reload_request: Option<ReloadPageInput>,
    pub last_get_html_request: Option<GetHtmlInput>,
    pub last_eval_js_request: Option<EvalJsInput>,
    pub last_scroll_request: Option<ScrollPageInput>,
    pub last_capture_screenshot_request: Option<CaptureScreenshotInput>,
    pub last_run_ocr_request: Option<RunOcrInput>,
    pub last_merge_ocr_request: Option<MergeOcrIntoPageModelInput>,
    pub last_read_region_request: Option<ReadRegionInput>,
    pub last_read_next_region_request: Option<ReadNextRegionInput>,
    pub last_read_previous_region_request: Option<ReadPreviousRegionInput>,
    pub last_stop_speaking_request: Option<StopSpeakingInput>,
    pub last_start_listening_request: Option<StartListeningInput>,
    pub last_stop_listening_request: Option<StopListeningInput>,
    pub last_transcribe_command_request: Option<TranscribeCommandInput>,
    pub last_snapshot_request: Option<GetPageSnapshotInput>,
    pub last_list_request: Option<ListInteractiveElementsInput>,
    pub last_find_request: Option<FindElementInput>,
    pub last_click_request: Option<ClickElementInput>,
    pub last_focus_request: Option<FocusElementInput>,
    pub last_type_request: Option<TypeIntoElementInput>,
    pub last_submit_request: Option<SubmitActiveFormInput>,
    pub last_extract_request: Option<ExtractPageModelInput>,
    pub last_voice: Option<String>,
    pub last_volume: Option<f32>,
    pub last_speed: Option<f32>,
    pub last_visibility: Option<BrowserVisibilityMode>,
    pub last_confirmation_prompt: Option<String>,
    pub last_report_result: Option<ReportResultData>,
    pub audio: RuntimeAudioState,
    pub browser_visibility: BrowserVisibilityMode,
    pub browser_visibility_switch_supported: bool,
}

impl Default for MockExecutor {
    fn default() -> Self {
        Self {
            last_open_url: None,
            last_go_back_request: None,
            last_go_forward_request: None,
            last_reload_request: None,
            last_get_html_request: None,
            last_eval_js_request: None,
            last_scroll_request: None,
            last_capture_screenshot_request: None,
            last_run_ocr_request: None,
            last_merge_ocr_request: None,
            last_read_region_request: None,
            last_read_next_region_request: None,
            last_read_previous_region_request: None,
            last_stop_speaking_request: None,
            last_start_listening_request: None,
            last_stop_listening_request: None,
            last_transcribe_command_request: None,
            last_snapshot_request: None,
            last_list_request: None,
            last_find_request: None,
            last_click_request: None,
            last_focus_request: None,
            last_type_request: None,
            last_submit_request: None,
            last_extract_request: None,
            last_voice: None,
            last_volume: None,
            last_speed: None,
            last_visibility: None,
            last_confirmation_prompt: None,
            last_report_result: None,
            audio: RuntimeAudioState::default(),
            browser_visibility: BrowserVisibilityMode::Visible,
            browser_visibility_switch_supported: true,
        }
    }
}

impl MockExecutor {
    pub fn current_browser_history(&self) -> BrowserHistoryState {
        if self.last_reload_request.is_some() {
            return BrowserHistoryState {
                can_go_back: true,
                can_go_forward: false,
                current_entry_index: Some(1),
                entry_count: 2,
            };
        }
        if self.last_go_forward_request.is_some() {
            return BrowserHistoryState {
                can_go_back: true,
                can_go_forward: false,
                current_entry_index: Some(1),
                entry_count: 2,
            };
        }
        if self.last_go_back_request.is_some() {
            return BrowserHistoryState {
                can_go_back: false,
                can_go_forward: true,
                current_entry_index: Some(0),
                entry_count: 2,
            };
        }
        if self.last_open_url.is_some() {
            return BrowserHistoryState {
                can_go_back: false,
                can_go_forward: false,
                current_entry_index: Some(0),
                entry_count: 1,
            };
        }

        BrowserHistoryState::default()
    }

    pub fn current_listening_state(&self) -> ListeningState {
        if let Some(input) = self.last_transcribe_command_request.as_ref() {
            return ListeningState {
                is_listening: !input.stop_mode.auto_stops(),
                push_to_talk_enabled: true,
            };
        }
        if self.last_stop_listening_request.is_some() {
            return ListeningState {
                is_listening: false,
                push_to_talk_enabled: true,
            };
        }
        if self.last_start_listening_request.is_some() {
            return ListeningState {
                is_listening: true,
                push_to_talk_enabled: true,
            };
        }

        ListeningState::default()
    }

    pub fn current_last_transcript(&self) -> Option<String> {
        if self.last_transcribe_command_request.is_some() {
            return Some(String::from("read the next section"));
        }

        None
    }

    pub fn current_browser_visibility(&self) -> BrowserVisibilityMode {
        self.browser_visibility
    }
}

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

pub fn resolve_planner_skill_fixture(
    fixture: &PlannerSkillFixture,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    match fixture.resolver {
        PlannerSkillFixtureResolver::Audio => resolve_direct_audio_command(
            fixture.transcript,
            fixture.name,
            fixture.agent_state.audio.playback_volume,
            fixture.agent_state.audio.playback_speed,
            active_skill_names,
        ),
        PlannerSkillFixtureResolver::NavigationReadback => {
            resolve_direct_navigation_readback_command(
                fixture.transcript,
                fixture.name,
                active_skill_names,
            )
        }
        PlannerSkillFixtureResolver::ReadPage => resolve_direct_read_page_command(
            fixture.transcript,
            fixture.name,
            fixture.page_model.as_ref(),
            &fixture.agent_state,
            active_skill_names,
        ),
        PlannerSkillFixtureResolver::StatusQuery => resolve_direct_status_query_command(
            fixture.transcript,
            fixture.name,
            &fixture.agent_state,
            &fixture_runtime_status(&fixture.agent_state),
            active_skill_names,
        ),
    }
}

pub fn assert_planner_skill_fixture(fixture: PlannerSkillFixture) {
    let available_tools = planner_available_tools();
    let selection = build_planner_skill_selection(None, None, fixture.transcript, &available_tools);
    let expected_selected_skills = fixture
        .expected_selected_skills
        .iter()
        .map(|skill| String::from(*skill))
        .collect::<Vec<_>>();
    let relevant_skill_names = selection
        .relevant_skill_summaries
        .iter()
        .map(|summary| summary.name.clone())
        .collect::<Vec<_>>();

    for expected_skill in &expected_selected_skills {
        assert!(
            selection
                .active_skill_names
                .iter()
                .any(|active_name| active_name == expected_skill),
            "fixture {} should have active skill {expected_skill}",
            fixture.name
        );
        assert!(
            relevant_skill_names
                .iter()
                .any(|skill_name| skill_name == expected_skill),
            "fixture {} should rank skill {expected_skill}; got {:?}",
            fixture.name,
            relevant_skill_names
        );
    }

    let planner_output = resolve_planner_skill_fixture(&fixture, &selection.active_skill_names)
        .unwrap_or_else(|| panic!("fixture {} should resolve directly", fixture.name));

    assert_eq!(
        planner_output.intent.name, fixture.expected_intent,
        "fixture {} resolved unexpected intent",
        fixture.name
    );
    assert_eq!(
        planner_output.selected_skills, expected_selected_skills,
        "fixture {} selected unexpected skills",
        fixture.name
    );

    let planned_tool_sequence = planner_output
        .steps
        .iter()
        .map(|step| step.tool_name.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        planned_tool_sequence, fixture.expected_tool_sequence,
        "fixture {} planned unexpected tool sequence",
        fixture.name
    );

    validate_planner_output(
        &planner_output,
        &available_tools,
        &selection.active_skill_names,
    )
    .unwrap_or_else(|error| panic!("fixture {} should validate, got {error:?}", fixture.name));

    let mut executor = MockExecutor::default();
    let outcome =
        execute_planner_output(&mut executor, String::from(fixture.name), &planner_output);
    let trace = match outcome {
        ExecutionOutcome::Complete { trace } => trace,
        other => panic!(
            "fixture {} should execute to completion, got {other:?}",
            fixture.name
        ),
    };
    let executed_tool_sequence = trace
        .tool_results
        .iter()
        .map(|result| result.tool_name.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        executed_tool_sequence, fixture.expected_tool_sequence,
        "fixture {} executed unexpected tool sequence",
        fixture.name
    );
}

impl DeterministicToolExecutor for MockExecutor {
    fn execute_open_url(&mut self, input: OpenUrlInput) -> ToolResult<OpenUrlData> {
        self.last_open_url = Some(input.url.clone());
        ToolResult::success(
            ToolName::OpenUrl,
            input.request_id,
            OpenUrlData {
                final_url: input.url,
                title: None,
                page_id: String::from("page-1"),
                load_state: input.wait_for_load_state.unwrap_or(LoadState::Load),
                http_status: None,
                history: BrowserHistoryState {
                    can_go_back: false,
                    can_go_forward: false,
                    current_entry_index: Some(0),
                    entry_count: 1,
                },
            },
            vec![String::from("opened url")],
        )
    }

    fn execute_go_back(&mut self, input: GoBackInput) -> ToolResult<GoBackData> {
        self.last_go_back_request = Some(input.clone());
        ToolResult::success(
            ToolName::GoBack,
            input.request_id,
            GoBackData {
                navigated: true,
                actual_steps: input.steps.unwrap_or(1),
                final_url: Some(String::from("https://example.com/previous")),
                title: Some(String::from("Previous page")),
                load_state: Some(input.wait_for_load_state.unwrap_or(LoadState::Load)),
                history: BrowserHistoryState {
                    can_go_back: false,
                    can_go_forward: true,
                    current_entry_index: Some(0),
                    entry_count: 2,
                },
            },
            vec![String::from("went back in history")],
        )
    }

    fn execute_go_forward(&mut self, input: GoForwardInput) -> ToolResult<GoForwardData> {
        self.last_go_forward_request = Some(input.clone());
        ToolResult::success(
            ToolName::GoForward,
            input.request_id,
            GoForwardData {
                navigated: true,
                actual_steps: input.steps.unwrap_or(1),
                final_url: Some(String::from("https://example.com/next")),
                title: Some(String::from("Next page")),
                load_state: Some(input.wait_for_load_state.unwrap_or(LoadState::Load)),
                history: BrowserHistoryState {
                    can_go_back: true,
                    can_go_forward: false,
                    current_entry_index: Some(1),
                    entry_count: 2,
                },
            },
            vec![String::from("went forward in history")],
        )
    }

    fn execute_reload_page(&mut self, input: ReloadPageInput) -> ToolResult<ReloadPageData> {
        self.last_reload_request = Some(input.clone());
        ToolResult::success(
            ToolName::ReloadPage,
            input.request_id,
            ReloadPageData {
                reloaded: true,
                final_url: String::from("https://example.com/current"),
                title: Some(String::from("Current page")),
                load_state: input.wait_for_load_state.unwrap_or(LoadState::Load),
                http_status: None,
                history: BrowserHistoryState {
                    can_go_back: true,
                    can_go_forward: false,
                    current_entry_index: Some(1),
                    entry_count: 2,
                },
            },
            vec![String::from("reloaded the page")],
        )
    }

    fn execute_get_html(&mut self, input: GetHtmlInput) -> ToolResult<GetHtmlData> {
        self.last_get_html_request = Some(input.clone());
        let html = String::from("<html><body><main>Example article</main></body></html>");
        let html_length = html.len();
        ToolResult::success(
            ToolName::GetHtml,
            input.request_id,
            GetHtmlData {
                page_id: String::from("page-1"),
                url: String::from("https://example.com/article"),
                title: Some(String::from("Example article")),
                html,
                html_length,
            },
            vec![String::from("read the current page HTML")],
        )
    }

    fn execute_eval_js(&mut self, input: EvalJsInput) -> ToolResult<EvalJsData> {
        self.last_eval_js_request = Some(input.clone());
        ToolResult::success(
            ToolName::EvalJs,
            input.request_id,
            EvalJsData {
                page_id: String::from("page-1"),
                url: String::from("https://example.com/article"),
                title: Some(String::from("Example article")),
                result: serde_json::json!({
                    "headline": "Example article",
                    "regionCount": 3
                }),
            },
            vec![String::from(
                "evaluated the requested JavaScript expression",
            )],
        )
    }

    fn execute_scroll_page(&mut self, input: ScrollPageInput) -> ToolResult<ScrollPageData> {
        self.last_scroll_request = Some(input.clone());
        ToolResult::success(
            ToolName::ScrollPage,
            input.request_id,
            ScrollPageData {
                previous_scroll_y: 120.0,
                current_scroll_y: 640.0,
                reached_boundary: false,
            },
            vec![String::from("scrolled the page")],
        )
    }

    fn execute_capture_screenshot(
        &mut self,
        input: CaptureScreenshotInput,
    ) -> ToolResult<CaptureScreenshotData> {
        self.last_capture_screenshot_request = Some(input.clone());
        ToolResult::success(
            ToolName::CaptureScreenshot,
            input.request_id,
            CaptureScreenshotData {
                image_id: String::from("image-1"),
                path: String::from("/tmp/image-1.png"),
                bbox: input.bbox,
                width: 640,
                height: 480,
            },
            vec![String::from("captured a screenshot")],
        )
    }

    fn execute_run_ocr(&mut self, input: RunOcrInput) -> ToolResult<RunOcrData> {
        self.last_run_ocr_request = Some(input.clone());
        ToolResult::success(
            ToolName::RunOcr,
            input.request_id,
            RunOcrData {
                image_id: input.image_id,
                extracted_text: String::from("recognized text"),
                text_length: 15,
                confidence: Some(0.82),
                source_bbox: input.bbox,
            },
            vec![String::from("ran OCR on the requested image")],
        )
    }

    fn execute_merge_ocr_into_page_model(
        &mut self,
        input: MergeOcrIntoPageModelInput,
    ) -> ToolResult<MergeOcrIntoPageModelData> {
        self.last_merge_ocr_request = Some(input.clone());
        ToolResult::success(
            ToolName::MergeOcrIntoPageModel,
            input.request_id,
            MergeOcrIntoPageModelData {
                page_id: input.page_id,
                updated_region_ids: vec![input
                    .region_id
                    .unwrap_or_else(|| String::from("ocr-region-1"))],
                merged_text_length: input.ocr_text.trim().len(),
            },
            vec![String::from("merged OCR text into the page model")],
        )
    }

    fn execute_read_region(&mut self, input: ReadRegionInput) -> ToolResult<ReadRegionData> {
        self.last_read_region_request = Some(input.clone());
        ToolResult::success(
            ToolName::ReadRegion,
            input.request_id,
            ReadRegionData {
                region_id: input.region_id,
                region_index: 1,
                text_length: 128,
                speech_started: true,
            },
            vec![String::from("started reading the requested region")],
        )
    }

    fn execute_read_next_region(
        &mut self,
        input: ReadNextRegionInput,
    ) -> ToolResult<ReadNextRegionData> {
        self.last_read_next_region_request = Some(input.clone());
        ToolResult::success(
            ToolName::ReadNextRegion,
            input.request_id,
            ReadNextRegionData {
                cursor: NarrationCursor {
                    current_region_id: Some(String::from("region-2")),
                    current_index: Some(1),
                    total_regions: 3,
                },
                region_id: Some(String::from("region-2")),
                speech_started: true,
                boundary: NarrationBoundary::None,
            },
            vec![String::from("advanced narration to the next region")],
        )
    }

    fn execute_read_previous_region(
        &mut self,
        input: ReadPreviousRegionInput,
    ) -> ToolResult<ReadPreviousRegionData> {
        self.last_read_previous_region_request = Some(input.clone());
        ToolResult::success(
            ToolName::ReadPreviousRegion,
            input.request_id,
            ReadPreviousRegionData {
                cursor: NarrationCursor {
                    current_region_id: Some(String::from("region-1")),
                    current_index: Some(0),
                    total_regions: 3,
                },
                region_id: Some(String::from("region-1")),
                speech_started: true,
                boundary: NarrationBoundary::None,
            },
            vec![String::from("moved narration to the previous region")],
        )
    }

    fn execute_stop_speaking(&mut self, input: StopSpeakingInput) -> ToolResult<StopSpeakingData> {
        self.last_stop_speaking_request = Some(input.clone());
        ToolResult::success(
            ToolName::StopSpeaking,
            input.request_id,
            StopSpeakingData {
                stopped: true,
                interrupted_region_id: Some(String::from("region-2")),
            },
            vec![String::from("stopped current narration playback")],
        )
    }

    fn execute_start_listening(
        &mut self,
        input: StartListeningInput,
    ) -> ToolResult<StartListeningData> {
        self.last_start_listening_request = Some(input.clone());
        ToolResult::success(
            ToolName::StartListening,
            input.request_id,
            StartListeningData {
                listening_state: ListeningState {
                    is_listening: true,
                    push_to_talk_enabled: true,
                },
                activated: true,
            },
            vec![String::from("started listening for voice input")],
        )
    }

    fn execute_stop_listening(
        &mut self,
        input: StopListeningInput,
    ) -> ToolResult<StopListeningData> {
        self.last_stop_listening_request = Some(input.clone());
        ToolResult::success(
            ToolName::StopListening,
            input.request_id,
            StopListeningData {
                listening_state: ListeningState {
                    is_listening: false,
                    push_to_talk_enabled: true,
                },
                deactivated: true,
            },
            vec![String::from("stopped listening for voice input")],
        )
    }

    fn execute_transcribe_command(
        &mut self,
        input: TranscribeCommandInput,
    ) -> ToolResult<TranscribeCommandData> {
        self.last_transcribe_command_request = Some(input.clone());
        ToolResult::success(
            ToolName::TranscribeCommand,
            input.request_id,
            TranscribeCommandData {
                transcript: Some(String::from("read the next section")),
                confidence: None,
                audio_duration_ms: input.max_duration_ms.or(Some(3_000)),
                listening_state: ListeningState {
                    is_listening: !input.stop_mode.auto_stops(),
                    push_to_talk_enabled: true,
                },
            },
            vec![String::from("transcribed a spoken command")],
        )
    }

    fn execute_get_page_snapshot(
        &mut self,
        input: GetPageSnapshotInput,
    ) -> ToolResult<PageSnapshotData> {
        self.last_snapshot_request = Some(input.clone());
        ToolResult::success(
            ToolName::GetPageSnapshot,
            input.request_id,
            PageSnapshotData {
                page_id: String::from("page-1"),
                url: String::from("https://example.com/article"),
                title: Some(String::from("Example article")),
                visible_text_excerpt: String::from("First paragraph"),
                interactive_elements: if input.include_interactive_elements {
                    vec![InteractiveElement {
                        element_id: String::from("link-1"),
                        dom_locator: Some(String::from("#link-1")),
                        role: crate::page_model::ElementRole::Link,
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
                    }]
                } else {
                    Vec::new()
                },
                scroll_y: 120.0,
                viewport_width: 1280.0,
                viewport_height: 720.0,
                document_height: 2400.0,
            },
            vec![String::from("captured page snapshot")],
        )
    }

    fn execute_list_interactive_elements(
        &mut self,
        input: ListInteractiveElementsInput,
    ) -> ToolResult<ListInteractiveElementsData> {
        self.last_list_request = Some(input.clone());
        ToolResult::success(
            ToolName::ListInteractiveElements,
            input.request_id,
            ListInteractiveElementsData {
                page_id: String::from("page-1"),
                elements: vec![InteractiveElement {
                    element_id: String::from("button-1"),
                    dom_locator: Some(String::from("#button-1")),
                    role: crate::page_model::ElementRole::Button,
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
                visible_count: 1,
            },
            vec![String::from("listed interactive elements")],
        )
    }

    fn execute_find_element(&mut self, input: FindElementInput) -> ToolResult<FindElementData> {
        self.last_find_request = Some(input.clone());
        ToolResult::success(
            ToolName::FindElement,
            input.request_id,
            FindElementData {
                query_summary: String::from("role=Button; description=continue"),
                chosen_element_id: Some(String::from("button-1")),
                chosen_confidence: Some(0.94),
                candidates: vec![ElementCandidate {
                    element_id: String::from("button-1"),
                    confidence_bps: 9400,
                    matched_on: vec![String::from("description"), String::from("role")],
                    rationale_codes: vec![
                        String::from("accessible_name_exact"),
                        String::from("role_match"),
                    ],
                }],
                requires_confirmation: false,
            },
            vec![String::from("found a matching element")],
        )
    }

    fn execute_click_element(&mut self, input: ClickElementInput) -> ToolResult<ClickElementData> {
        self.last_click_request = Some(input.clone());
        ToolResult::success(
            ToolName::ClickElement,
            input.request_id,
            ClickElementData {
                element_id: input.element_id,
                action_performed: true,
                page_changed: false,
                navigation_url: None,
                resulting_title: Some(String::from("Example article")),
            },
            vec![String::from("clicked the requested element")],
        )
    }

    fn execute_focus_element(&mut self, input: FocusElementInput) -> ToolResult<FocusElementData> {
        self.last_focus_request = Some(input.clone());
        ToolResult::success(
            ToolName::FocusElement,
            input.request_id,
            FocusElementData {
                element_id: input.element_id,
                focused: true,
                element_role: Some(crate::page_model::ElementRole::Input),
            },
            vec![String::from("focused the requested element")],
        )
    }

    fn execute_type_into_element(
        &mut self,
        input: TypeIntoElementInput,
    ) -> ToolResult<TypeIntoElementData> {
        self.last_type_request = Some(input.clone());
        ToolResult::success(
            ToolName::TypeIntoElement,
            input.request_id,
            TypeIntoElementData {
                element_id: input.element_id,
                text_length: input.text.chars().count(),
                value_after: Some(input.text),
                accepted_input: true,
            },
            vec![String::from("typed into the requested element")],
        )
    }

    fn execute_submit_active_form(
        &mut self,
        input: SubmitActiveFormInput,
    ) -> ToolResult<SubmitActiveFormData> {
        self.last_submit_request = Some(input.clone());
        ToolResult::success(
            ToolName::SubmitActiveForm,
            input.request_id,
            SubmitActiveFormData {
                form_element_id: input.form_element_id,
                submitted: true,
                page_changed: true,
                navigation_url: Some(String::from("https://example.com/submitted")),
            },
            vec![String::from("submitted the active form")],
        )
    }

    fn execute_extract_page_model(
        &mut self,
        input: ExtractPageModelInput,
    ) -> ToolResult<ExtractPageModelData> {
        self.last_extract_request = Some(input.clone());
        ToolResult::success(
            ToolName::ExtractPageModel,
            input.request_id,
            ExtractPageModelData {
                page_model: PageModel {
                    title: Some(String::from("Example article")),
                    url: Some(String::from("https://example.com/article")),
                    regions: Vec::new(),
                    interactive_elements: if input.include_links {
                        vec![InteractiveElement {
                            element_id: String::from("link-1"),
                            dom_locator: Some(String::from("#link-1")),
                            role: crate::page_model::ElementRole::Link,
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
                        }]
                    } else {
                        Vec::new()
                    },
                },
                region_count: 0,
                readable_region_count: 0,
                extraction_source: ExtractionSource::DomFallback,
            },
            vec![String::from("extracted page model")],
        )
    }

    fn execute_set_tts_voice(&mut self, input: SetTtsVoiceInput) -> ToolResult<SetTtsVoiceData> {
        self.last_voice = Some(input.voice.to_string());
        ToolResult::success(
            ToolName::SetTtsVoice,
            input.request_id,
            SetTtsVoiceData {
                voice: input.voice.to_string(),
                changed: true,
            },
            vec![String::from("voice updated")],
        )
    }

    fn execute_set_playback_volume(
        &mut self,
        input: SetPlaybackVolumeInput,
    ) -> ToolResult<SetPlaybackVolumeData> {
        let clamped_volume = input.volume.clamp(
            crate::config::MIN_PLAYBACK_VOLUME,
            crate::config::MAX_PLAYBACK_VOLUME,
        );
        let changed = (self.audio.playback_volume - clamped_volume).abs() > f32::EPSILON;
        self.last_volume = Some(input.volume);
        self.audio.playback_volume = clamped_volume;
        self.audio.muted = clamped_volume == 0.0;
        let mut observations = vec![
            String::from("Updated the playback volume setting."),
            String::from("New narration requests will use the updated playback volume."),
        ];
        if (input.volume - clamped_volume).abs() > f32::EPSILON {
            observations.push(String::from(
                "Requested playback volume was clamped to the supported range.",
            ));
        }
        ToolResult::success(
            ToolName::SetPlaybackVolume,
            input.request_id,
            SetPlaybackVolumeData {
                playback_volume: self.audio.playback_volume,
                muted: self.audio.muted,
                changed,
            },
            observations,
        )
    }

    fn execute_set_playback_speed(
        &mut self,
        input: SetPlaybackSpeedInput,
    ) -> ToolResult<SetPlaybackSpeedData> {
        let clamped_speed = input.speed.clamp(
            crate::config::MIN_PLAYBACK_SPEED,
            crate::config::MAX_PLAYBACK_SPEED,
        );
        let changed = (self.audio.playback_speed - clamped_speed).abs() > f32::EPSILON;
        self.last_speed = Some(input.speed);
        self.audio.playback_speed = clamped_speed;
        let mut observations = vec![
            String::from("Updated the playback speed setting."),
            String::from("New narration requests will use the updated native TTS speed."),
        ];
        if (input.speed - clamped_speed).abs() > f32::EPSILON {
            observations.push(String::from(
                "Requested playback speed was clamped to the supported range.",
            ));
        }
        ToolResult::success(
            ToolName::SetPlaybackSpeed,
            input.request_id,
            SetPlaybackSpeedData {
                playback_speed: self.audio.playback_speed,
                changed,
            },
            observations,
        )
    }

    fn execute_set_browser_visibility(
        &mut self,
        input: SetBrowserVisibilityInput,
    ) -> ToolResult<SetBrowserVisibilityData> {
        self.last_visibility = Some(input.mode);
        if self.browser_visibility == input.mode {
            return ToolResult::success(
                ToolName::SetBrowserVisibility,
                input.request_id,
                SetBrowserVisibilityData {
                    mode: self.browser_visibility,
                    changed: false,
                    supported: true,
                },
                vec![String::from(
                    "Browser visibility mode is already set to the requested value.",
                )],
            );
        }
        if !self.browser_visibility_switch_supported {
            return ToolResult::success(
                ToolName::SetBrowserVisibility,
                input.request_id,
                SetBrowserVisibilityData {
                    mode: self.browser_visibility,
                    changed: false,
                    supported: false,
                },
                vec![String::from(
                    "Browser visibility switching is not supported in this build.",
                )],
            );
        }
        self.browser_visibility = input.mode;
        ToolResult::success(
            ToolName::SetBrowserVisibility,
            input.request_id,
            SetBrowserVisibilityData {
                mode: self.browser_visibility,
                changed: true,
                supported: true,
            },
            vec![String::from("Browser visibility mode was updated.")],
        )
    }

    fn execute_get_agent_state(&mut self, input: GetAgentStateInput) -> ToolResult<AgentStateData> {
        ToolResult::success(
            ToolName::GetAgentState,
            input.request_id,
            AgentStateData {
                page_id: None,
                url: Some(String::from("https://example.com")),
                title: Some(String::from("Example")),
                browser_visibility: self.current_browser_visibility(),
                browser_history: self.current_browser_history(),
                narration_cursor: Some(NarrationCursor::default()),
                speaking: false,
                listening_state: self.current_listening_state(),
                audio: self.audio.clone(),
                last_transcript: if input.include_last_transcript {
                    self.current_last_transcript()
                } else {
                    None
                },
                last_tool_call: Some(LastToolCallSummary {
                    request_id: String::from("req-5"),
                    tool_name: ToolName::GetAgentState,
                    ok: true,
                    observation_summary: vec![String::from("agent state read")],
                }),
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
                    summary: String::from("Automatic provider failover is not currently available in the live runtime."),
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
            },
            vec![String::from("agent state read")],
        )
    }

    fn execute_get_runtime_status(
        &mut self,
        input: GetRuntimeStatusInput,
    ) -> ToolResult<GetRuntimeStatusData> {
        ToolResult::success(
            ToolName::GetRuntimeStatus,
            input.request_id,
            GetRuntimeStatusData {
                page_id: None,
                url: Some(String::from("https://example.com")),
                title: Some(String::from("Example")),
                browser_visibility: self.current_browser_visibility(),
                browser_history: self.current_browser_history(),
                listening_state: self.current_listening_state(),
                speaking: false,
                audio: self.audio.clone(),
                pending_confirmation_id: None,
                pending_plan_execution: None,
                provider_modes: if input.include_provider_modes {
                    Some(ProviderSelectionStatus {
                        planner_mode: ProviderMode::Remote,
                        tts_mode: ProviderMode::Local,
                        asr_mode: ProviderMode::Local,
                    })
                } else {
                    None
                },
            },
            vec![String::from("runtime status read")],
        )
    }

    fn execute_confirm_action(
        &mut self,
        input: ConfirmActionInput,
    ) -> ToolResult<ConfirmActionData> {
        self.last_confirmation_prompt = Some(input.prompt_text.clone());
        ToolResult::success(
            ToolName::ConfirmAction,
            input.request_id,
            ConfirmActionData {
                confirmation_id: String::from("confirm-1"),
                prompt_text: input.prompt_text,
                confirmed: None,
                timed_out: false,
            },
            vec![input.reason],
        )
    }

    fn execute_report_result(&mut self, input: ReportResultInput) -> ToolResult<ReportResultData> {
        let data = ReportResultData {
            status: input.status,
            summary: input.summary,
            next_recommended_action: input.next_recommended_action,
            user_message: input.user_message,
        };
        self.last_report_result = Some(data.clone());

        ToolResult::success(
            ToolName::ReportResult,
            input.request_id,
            data,
            vec![String::from("reported final result")],
        )
    }
}

pub fn sample_planned_step(tool_name: ToolName) -> PlannedStep {
    match tool_name {
        ToolName::OpenUrl => PlannedStep {
            step_id: String::from("step-open-url"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-open-url",
                "timeout_ms": 1000,
                "url": "https://example.com/article",
                "wait_for_load_state": "NetworkIdle"
            }),
            purpose: String::from("navigate to a page"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::GoBack => PlannedStep {
            step_id: String::from("step-go-back"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-go-back",
                "timeout_ms": 1000,
                "steps": 2,
                "wait_for_load_state": "Load"
            }),
            purpose: String::from("go back in history"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::GoForward => PlannedStep {
            step_id: String::from("step-go-forward"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-go-forward",
                "timeout_ms": 1000,
                "steps": 1,
                "wait_for_load_state": "NetworkIdle"
            }),
            purpose: String::from("go forward in history"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::ReloadPage => PlannedStep {
            step_id: String::from("step-reload"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-reload",
                "timeout_ms": 1000,
                "mode": "Hard",
                "wait_for_load_state": "Load"
            }),
            purpose: String::from("reload the current page"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::GetHtml => PlannedStep {
            step_id: String::from("step-get-html"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-get-html",
                "timeout_ms": 1000
            }),
            purpose: String::from("read current page HTML"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::EvalJs => PlannedStep {
            step_id: String::from("step-eval-js"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-eval-js",
                "timeout_ms": 1000,
                "expression": "({ headline: document.title, regionCount: document.querySelectorAll('main, article, section').length })"
            }),
            purpose: String::from("evaluate a bounded JavaScript expression"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::ScrollPage => PlannedStep {
            step_id: String::from("step-scroll"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-scroll",
                "timeout_ms": 1000,
                "direction": "Down",
                "amount_px": 480.0,
                "target": null
            }),
            purpose: String::from("scroll the page"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::CaptureScreenshot => PlannedStep {
            step_id: String::from("step-capture-screenshot"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-capture-screenshot",
                "timeout_ms": 1000,
                "scope": "Viewport",
                "region_id": serde_json::Value::Null,
                "bbox": {
                    "x": 10.0,
                    "y": 20.0,
                    "width": 300.0,
                    "height": 120.0
                }
            }),
            purpose: String::from("capture a deterministic screenshot"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::SetBrowserVisibility => PlannedStep {
            step_id: String::from("step-set-browser-visibility"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-visibility",
                "timeout_ms": 1000,
                "mode": "Headless"
            }),
            purpose: String::from("toggle browser visibility"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::GetPageSnapshot => PlannedStep {
            step_id: String::from("step-snapshot"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-snapshot",
                "timeout_ms": 1000,
                "include_interactive_elements": true,
                "text_excerpt_max_chars": 120
            }),
            purpose: String::from("read current page snapshot"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::ExtractPageModel => PlannedStep {
            step_id: String::from("step-extract"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-extract",
                "timeout_ms": 1000,
                "use_dom_extraction": true,
                "include_headings": true,
                "include_links": false
            }),
            purpose: String::from("extract a page model"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::ListInteractiveElements => PlannedStep {
            step_id: String::from("step-list"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-list",
                "timeout_ms": 1000,
                "visibility_filter": "VisibleOnly",
                "roles": ["Button"]
            }),
            purpose: String::from("list visible buttons"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::FindElement => PlannedStep {
            step_id: String::from("step-find"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-find",
                "timeout_ms": 1000,
                "description": "continue",
                "text": null,
                "role": "Button",
                "color_hint": null,
                "nearby_text": null,
                "selector_hint": null,
                "visibility_filter": "VisibleOnly",
                "max_candidates": 3
            }),
            purpose: String::from("find the continue button"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::ClickElement => PlannedStep {
            step_id: String::from("step-click"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-click",
                "timeout_ms": 1000,
                "element_id": "button-1",
                "click_mode": "Single"
            }),
            purpose: String::from("click the resolved button"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::FocusElement => PlannedStep {
            step_id: String::from("step-focus"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-focus",
                "timeout_ms": 1000,
                "element_id": "input-1"
            }),
            purpose: String::from("focus the resolved field"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::TypeIntoElement => PlannedStep {
            step_id: String::from("step-type"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-type",
                "timeout_ms": 1000,
                "element_id": "input-1",
                "text": "phil@example.com",
                "text_entry_mode": "Replace",
                "submit_mode": "KeepEditing"
            }),
            purpose: String::from("type into the resolved field"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::SubmitActiveForm => PlannedStep {
            step_id: String::from("step-submit"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-submit",
                "timeout_ms": 1000,
                "form_element_id": "form-login"
            }),
            purpose: String::from("submit the active form"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::ReadRegion => PlannedStep {
            step_id: String::from("step-read-region"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-read-region",
                "timeout_ms": 1000,
                "region_id": "region-2",
                "interruption_mode": "Interrupt"
            }),
            purpose: String::from("read a specific region"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::ReadNextRegion => PlannedStep {
            step_id: String::from("step-read-next"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-read-next",
                "timeout_ms": 1000,
                "interruption_mode": "Queue"
            }),
            purpose: String::from("read the next region"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::ReadPreviousRegion => PlannedStep {
            step_id: String::from("step-read-previous"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-read-previous",
                "timeout_ms": 1000,
                "interruption_mode": "Interrupt"
            }),
            purpose: String::from("read the previous region"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::StopSpeaking => PlannedStep {
            step_id: String::from("step-stop-speaking"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-stop-speaking",
                "timeout_ms": 1000
            }),
            purpose: String::from("stop current narration"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::StartListening => PlannedStep {
            step_id: String::from("step-start-listening"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-start-listening",
                "timeout_ms": 1500
            }),
            purpose: String::from("start listening"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::StopListening => PlannedStep {
            step_id: String::from("step-stop-listening"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-stop-listening"
            }),
            purpose: String::from("stop listening"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::TranscribeCommand => PlannedStep {
            step_id: String::from("step-transcribe-command"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-transcribe-command",
                "timeout_ms": 2000,
                "max_duration_ms": 3000,
                "stop_mode": "AutoStop"
            }),
            purpose: String::from("transcribe a command"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::SetTtsVoice => PlannedStep {
            step_id: String::from("step-set-tts-voice"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-set-tts-voice",
                "timeout_ms": 1000,
                "voice": "Bruno"
            }),
            purpose: String::from("change the TTS voice"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::SetPlaybackVolume => PlannedStep {
            step_id: String::from("step-set-playback-volume"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-set-playback-volume",
                "timeout_ms": 1000,
                "volume": 0.4
            }),
            purpose: String::from("update volume"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::SetPlaybackSpeed => PlannedStep {
            step_id: String::from("step-set-playback-speed"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-set-playback-speed",
                "timeout_ms": 1000,
                "speed": 1.2
            }),
            purpose: String::from("update speed"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::RunOcr => PlannedStep {
            step_id: String::from("step-run-ocr"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-run-ocr",
                "timeout_ms": 1000,
                "image_id": "image-1",
                "region_id": serde_json::Value::Null,
                "bbox": {
                    "x": 4.0,
                    "y": 8.0,
                    "width": 120.0,
                    "height": 48.0
                }
            }),
            purpose: String::from("run OCR on a cached screenshot"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::MergeOcrIntoPageModel => PlannedStep {
            step_id: String::from("step-merge-ocr"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-merge-ocr",
                "timeout_ms": 1000,
                "page_id": "page-1",
                "region_id": "region-2",
                "ocr_text": "Recovered readable text",
                "source_bbox": {
                    "x": 10.0,
                    "y": 12.0,
                    "width": 200.0,
                    "height": 80.0
                }
            }),
            purpose: String::from("merge OCR text into the runtime page model"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::GetAgentState => PlannedStep {
            step_id: String::from("step-get-agent-state"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-agent-state",
                "timeout_ms": 1000,
                "include_last_transcript": false
            }),
            purpose: String::from("read agent state"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::GetRuntimeStatus => PlannedStep {
            step_id: String::from("step-get-runtime-status"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-runtime-status",
                "timeout_ms": 1000,
                "include_provider_modes": true
            }),
            purpose: String::from("read runtime status"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
        ToolName::ConfirmAction => PlannedStep {
            step_id: String::from("step-confirm-action"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-confirm-action",
                "timeout_ms": 1000,
                "prompt_text": "Do you want me to continue?",
                "reason": "The next step may submit data."
            }),
            purpose: String::from("request confirmation"),
            on_success: StepTransition::RequestConfirmation,
            on_failure: StepTransition::Replan,
        },
        ToolName::ReportResult => PlannedStep {
            step_id: String::from("step-report-result"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-report-result",
                "timeout_ms": 1000,
                "status": "Success",
                "summary": "Opened the requested page.",
                "next_recommended_action": null,
                "user_message": "The page is ready."
            }),
            purpose: String::from("report completion"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        },
    }
}

pub fn sample_planned_steps_for_registered_tools() -> Vec<PlannedStep> {
    registered_tools()
        .into_iter()
        .map(|tool| sample_planned_step(tool.name))
        .collect()
}

pub fn assert_json_matches_schema(
    instance: &serde_json::Value,
    schema: &serde_json::Value,
) -> Result<(), String> {
    assert_json_matches_schema_at(instance, schema, schema, "$")
}

pub fn assert_json_matches_schema_at(
    instance: &serde_json::Value,
    schema: &serde_json::Value,
    root_schema: &serde_json::Value,
    path: &str,
) -> Result<(), String> {
    let schema = resolve_schema_reference(schema, root_schema)?;

    if let Some(all_of) = schema.get("allOf").and_then(serde_json::Value::as_array) {
        for subschema in all_of {
            assert_json_matches_schema_at(instance, subschema, root_schema, path)?;
        }
    }

    if let Some(any_of) = schema.get("anyOf").and_then(serde_json::Value::as_array) {
        let mut errors = Vec::new();
        for subschema in any_of {
            match assert_json_matches_schema_at(instance, subschema, root_schema, path) {
                Ok(()) => {
                    errors.clear();
                    break;
                }
                Err(error) => errors.push(error),
            }
        }
        if !errors.is_empty() {
            return Err(format!(
                "{path}: value did not satisfy anyOf alternatives: {}",
                errors.join(" | ")
            ));
        }
    }

    if let Some(one_of) = schema.get("oneOf").and_then(serde_json::Value::as_array) {
        let mut match_count = 0;
        let mut errors = Vec::new();
        for subschema in one_of {
            match assert_json_matches_schema_at(instance, subschema, root_schema, path) {
                Ok(()) => match_count += 1,
                Err(error) => errors.push(error),
            }
        }
        if match_count != 1 {
            return Err(format!(
                "{path}: value matched {match_count} oneOf alternatives (expected exactly 1): {}",
                errors.join(" | ")
            ));
        }
    }

    if let Some(expected_const) = schema.get("const") {
        if instance != expected_const {
            return Err(format!(
                "{path}: expected const {expected_const:?}, got {instance:?}"
            ));
        }
    }

    if let Some(enum_values) = schema.get("enum").and_then(serde_json::Value::as_array) {
        if !enum_values.iter().any(|candidate| candidate == instance) {
            return Err(format!(
                "{path}: expected one of {enum_values:?}, got {instance:?}"
            ));
        }
    }

    if let Some(type_schema) = schema.get("type") {
        if !json_matches_type(instance, type_schema) {
            return Err(format!(
                "{path}: value {instance:?} did not match schema type {type_schema:?}"
            ));
        }
    }

    if let Some(required) = schema.get("required").and_then(serde_json::Value::as_array) {
        let Some(object) = instance.as_object() else {
            return Err(format!("{path}: required fields only apply to objects"));
        };
        for field_name in required.iter().filter_map(serde_json::Value::as_str) {
            if !object.contains_key(field_name) {
                return Err(format!("{path}: missing required field '{field_name}'"));
            }
        }
    }

    if let Some(properties) = schema
        .get("properties")
        .and_then(serde_json::Value::as_object)
    {
        let Some(object) = instance.as_object() else {
            return Err(format!("{path}: properties only apply to objects"));
        };
        let additional_properties_allowed = schema
            .get("additionalProperties")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);

        for (key, value) in object {
            if let Some(property_schema) = properties.get(key) {
                assert_json_matches_schema_at(
                    value,
                    property_schema,
                    root_schema,
                    &format!("{path}/{key}"),
                )?;
            } else if !additional_properties_allowed {
                return Err(format!(
                    "{path}: unexpected property '{key}' is not allowed by the schema"
                ));
            }
        }
    }

    if let Some(items_schema) = schema.get("items") {
        let Some(array) = instance.as_array() else {
            return Err(format!("{path}: items only apply to arrays"));
        };
        for (index, item) in array.iter().enumerate() {
            assert_json_matches_schema_at(
                item,
                items_schema,
                root_schema,
                &format!("{path}/{index}"),
            )?;
        }
    }

    Ok(())
}

pub fn resolve_schema_reference<'a>(
    schema: &'a serde_json::Value,
    root_schema: &'a serde_json::Value,
) -> Result<&'a serde_json::Value, String> {
    let Some(reference) = schema.get("$ref").and_then(serde_json::Value::as_str) else {
        return Ok(schema);
    };
    let pointer = reference
        .strip_prefix('#')
        .ok_or_else(|| format!("unsupported non-local schema reference '{reference}'"))?;
    let resolved = root_schema
        .pointer(pointer)
        .ok_or_else(|| format!("failed to resolve schema reference '{reference}'"))?;
    if std::ptr::eq(resolved, schema) {
        return Ok(resolved);
    }
    resolve_schema_reference(resolved, root_schema)
}

pub fn json_matches_type(instance: &serde_json::Value, type_schema: &serde_json::Value) -> bool {
    match type_schema {
        serde_json::Value::String(kind) => json_matches_single_type(instance, kind),
        serde_json::Value::Array(kinds) => kinds.iter().any(|kind| {
            kind.as_str()
                .is_some_and(|kind| json_matches_single_type(instance, kind))
        }),
        _ => true,
    }
}

pub fn json_matches_single_type(instance: &serde_json::Value, kind: &str) -> bool {
    match kind {
        "null" => instance.is_null(),
        "boolean" => instance.is_boolean(),
        "object" => instance.is_object(),
        "array" => instance.is_array(),
        "string" => instance.is_string(),
        "number" => instance.is_number(),
        "integer" => instance
            .as_f64()
            .is_some_and(|number| number.fract().abs() < f64::EPSILON),
        _ => false,
    }
}
