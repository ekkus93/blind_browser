use super::*;
use crate::page_model::RegionRole;

fn unique_temp_path(label: &str) -> std::path::PathBuf {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "blind-browser-{label}-{}-{timestamp}",
        std::process::id()
    ))
}

fn write_skill_document(root: &std::path::Path, skill_name: &str, content: &str) {
    let skill_dir = root.join(skill_name);
    std::fs::create_dir_all(&skill_dir).expect("skill directory should be created");
    std::fs::write(skill_dir.join("SKILL.md"), content).expect("skill document should be written");
}

struct MockExecutor {
    last_open_url: Option<String>,
    last_go_back_request: Option<GoBackInput>,
    last_go_forward_request: Option<GoForwardInput>,
    last_reload_request: Option<ReloadPageInput>,
    last_get_html_request: Option<GetHtmlInput>,
    last_eval_js_request: Option<EvalJsInput>,
    last_scroll_request: Option<ScrollPageInput>,
    last_capture_screenshot_request: Option<CaptureScreenshotInput>,
    last_run_ocr_request: Option<RunOcrInput>,
    last_merge_ocr_request: Option<MergeOcrIntoPageModelInput>,
    last_read_region_request: Option<ReadRegionInput>,
    last_read_next_region_request: Option<ReadNextRegionInput>,
    last_read_previous_region_request: Option<ReadPreviousRegionInput>,
    last_stop_speaking_request: Option<StopSpeakingInput>,
    last_start_listening_request: Option<StartListeningInput>,
    last_stop_listening_request: Option<StopListeningInput>,
    last_transcribe_command_request: Option<TranscribeCommandInput>,
    last_snapshot_request: Option<GetPageSnapshotInput>,
    last_list_request: Option<ListInteractiveElementsInput>,
    last_find_request: Option<FindElementInput>,
    last_click_request: Option<ClickElementInput>,
    last_focus_request: Option<FocusElementInput>,
    last_type_request: Option<TypeIntoElementInput>,
    last_submit_request: Option<SubmitActiveFormInput>,
    last_extract_request: Option<ExtractPageModelInput>,
    last_voice: Option<String>,
    last_volume: Option<f32>,
    last_speed: Option<f32>,
    last_visibility: Option<BrowserVisibilityMode>,
    last_confirmation_prompt: Option<String>,
    last_report_result: Option<ReportResultData>,
    audio: RuntimeAudioState,
    browser_visibility: BrowserVisibilityMode,
    browser_visibility_switch_supported: bool,
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
    fn current_browser_history(&self) -> BrowserHistoryState {
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

    fn current_listening_state(&self) -> ListeningState {
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

    fn current_last_transcript(&self) -> Option<String> {
        if self.last_transcribe_command_request.is_some() {
            return Some(String::from("read the next section"));
        }

        None
    }

    fn current_browser_visibility(&self) -> BrowserVisibilityMode {
        self.browser_visibility
    }
}

#[derive(Clone, Copy)]
enum PlannerSkillFixtureResolver {
    Audio,
    NavigationReadback,
    ReadPage,
    StatusQuery,
}

struct PlannerSkillFixture {
    name: &'static str,
    transcript: &'static str,
    resolver: PlannerSkillFixtureResolver,
    agent_state: AgentStateData,
    page_model: Option<PageModel>,
    expected_intent: IntentName,
    expected_selected_skills: Vec<&'static str>,
    expected_tool_sequence: Vec<ToolName>,
}

fn fixture_agent_state() -> AgentStateData {
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

fn fixture_runtime_status(agent_state: &AgentStateData) -> GetRuntimeStatusData {
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

fn fixture_page_model_without_regions() -> PageModel {
    PageModel {
        title: Some(String::from("Example article")),
        url: Some(String::from("https://example.com/article")),
        regions: Vec::new(),
        interactive_elements: Vec::new(),
    }
}

fn fixture_agent_state_for_page(title: &str, url: &str) -> AgentStateData {
    let mut agent_state = fixture_agent_state();
    agent_state.title = Some(String::from(title));
    agent_state.url = Some(String::from(url));
    agent_state
}

fn fixture_problematic_article_page_without_regions() -> PageModel {
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

fn fixture_problematic_docs_agent_state() -> AgentStateData {
    fixture_agent_state_for_page(
        "Blind Browser docs | Voice commands",
        "https://docs.example.com/blind-browser/voice-commands?ref=sidebar",
    )
}

fn resolve_planner_skill_fixture(
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

fn assert_planner_skill_fixture(fixture: PlannerSkillFixture) {
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

fn sample_planned_step(tool_name: ToolName) -> PlannedStep {
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

fn sample_planned_steps_for_registered_tools() -> Vec<PlannedStep> {
    registered_tools()
        .into_iter()
        .map(|tool| sample_planned_step(tool.name))
        .collect()
}

#[test]
fn dispatches_set_playback_volume_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-1"),
        tool_name: ToolName::SetPlaybackVolume,
        arguments: serde_json::json!({
            "request_id": "req-1",
            "timeout_ms": 1000,
            "volume": 0.4
        }),
        purpose: String::from("update volume"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(executor.last_volume, Some(0.4));
    assert_eq!(result.tool_name, ToolName::SetPlaybackVolume);
    assert_eq!(result.request_id, "req-1");
    let data = result
        .data
        .expect("serialized tool result data should exist");
    assert_eq!(data.get("muted"), Some(&serde_json::Value::Bool(false)));
    assert_eq!(data.get("changed"), Some(&serde_json::Value::Bool(true)));
    let playback_volume = data
        .get("playback_volume")
        .and_then(serde_json::Value::as_f64)
        .expect("playback_volume should be serialized as a number");
    assert!((playback_volume - 0.4).abs() < 0.000_001);
}

#[test]
fn dispatches_open_url_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-open-url"),
        tool_name: ToolName::OpenUrl,
        arguments: serde_json::json!({
            "request_id": "req-open-url",
            "timeout_ms": 1000,
            "url": "https://example.com/article",
            "wait_for_load_state": "NetworkIdle"
        }),
        purpose: String::from("navigate to a page"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor.last_open_url.as_deref(),
        Some("https://example.com/article")
    );
    let data = result.data.expect("open_url should serialize");
    assert_eq!(
        data.get("final_url"),
        Some(&serde_json::Value::String(String::from(
            "https://example.com/article"
        )))
    );
    assert_eq!(
        data.get("load_state"),
        Some(&serde_json::Value::String(String::from("NetworkIdle")))
    );
}

#[test]
fn dispatches_go_back_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-go-back"),
        tool_name: ToolName::GoBack,
        arguments: serde_json::json!({
            "request_id": "req-go-back",
            "timeout_ms": 1000,
            "steps": 2,
            "wait_for_load_state": "Load"
        }),
        purpose: String::from("go back in history"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_go_back_request
            .as_ref()
            .and_then(|input| input.steps),
        Some(2)
    );
}

#[test]
fn dispatches_go_forward_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-go-forward"),
        tool_name: ToolName::GoForward,
        arguments: serde_json::json!({
            "request_id": "req-go-forward",
            "timeout_ms": 1000,
            "steps": 1,
            "wait_for_load_state": "NetworkIdle"
        }),
        purpose: String::from("go forward in history"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_go_forward_request
            .as_ref()
            .and_then(|input| input.steps),
        Some(1)
    );
}

#[test]
fn dispatches_reload_page_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-reload"),
        tool_name: ToolName::ReloadPage,
        arguments: serde_json::json!({
            "request_id": "req-reload",
            "timeout_ms": 1000,
            "mode": "Hard",
            "wait_for_load_state": "Load"
        }),
        purpose: String::from("reload the current page"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_reload_request
            .as_ref()
            .map(|input| input.mode),
        Some(ReloadMode::Hard)
    );
}

#[test]
fn dispatches_get_html_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-get-html"),
        tool_name: ToolName::GetHtml,
        arguments: serde_json::json!({
            "request_id": "req-get-html",
            "timeout_ms": 1000
        }),
        purpose: String::from("read current page HTML"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_get_html_request
            .as_ref()
            .map(|input| input.request_id.as_str()),
        Some("req-get-html")
    );
    let data = result.data.expect("get_html should serialize");
    assert_eq!(
        data.get("page_id"),
        Some(&serde_json::Value::String(String::from("page-1")))
    );
    assert_eq!(
        data.get("html_length").and_then(serde_json::Value::as_u64),
        Some(54)
    );
    assert!(data
        .get("html")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|html| html.contains("<main>Example article</main>")));
}

#[test]
fn dispatches_eval_js_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-eval-js"),
        tool_name: ToolName::EvalJs,
        arguments: serde_json::json!({
            "request_id": "req-eval-js",
            "timeout_ms": 1000,
            "expression": "({ headline: document.title, regionCount: document.querySelectorAll('main, article, section').length })"
        }),
        purpose: String::from("evaluate a bounded JavaScript expression"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_eval_js_request
            .as_ref()
            .map(|input| input.request_id.as_str()),
        Some("req-eval-js")
    );
    let data = result.data.expect("eval_js should serialize");
    assert_eq!(
        data.get("page_id"),
        Some(&serde_json::Value::String(String::from("page-1")))
    );
    assert_eq!(
        data.get("result")
            .and_then(|value| value.get("regionCount"))
            .and_then(serde_json::Value::as_u64),
        Some(3)
    );
}

#[test]
fn dispatches_scroll_page_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-scroll"),
        tool_name: ToolName::ScrollPage,
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
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_scroll_request
            .as_ref()
            .and_then(|input| input.amount_px),
        Some(480.0)
    );
}

#[test]
fn dispatches_read_region_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-read-region"),
        tool_name: ToolName::ReadRegion,
        arguments: serde_json::json!({
            "request_id": "req-read-region",
            "timeout_ms": 1000,
            "region_id": "region-2",
            "interruption_mode": "Interrupt"
        }),
        purpose: String::from("read a specific region"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_read_region_request
            .as_ref()
            .map(|input| input.region_id.as_str()),
        Some("region-2")
    );
}

#[test]
fn dispatches_read_next_region_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-read-next"),
        tool_name: ToolName::ReadNextRegion,
        arguments: serde_json::json!({
            "request_id": "req-read-next",
            "timeout_ms": 1000,
            "interruption_mode": "Queue"
        }),
        purpose: String::from("read the next region"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_read_next_region_request
            .as_ref()
            .map(|input| input.interruption_mode),
        Some(NarrationInterruptionMode::Queue)
    );
}

#[test]
fn dispatches_read_previous_region_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-read-previous"),
        tool_name: ToolName::ReadPreviousRegion,
        arguments: serde_json::json!({
            "request_id": "req-read-previous",
            "timeout_ms": 1000,
            "interruption_mode": "Interrupt"
        }),
        purpose: String::from("read the previous region"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_read_previous_region_request
            .as_ref()
            .map(|input| input.interruption_mode),
        Some(NarrationInterruptionMode::Interrupt)
    );
}

#[test]
fn dispatches_stop_speaking_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-stop-speaking"),
        tool_name: ToolName::StopSpeaking,
        arguments: serde_json::json!({
            "request_id": "req-stop-speaking",
            "timeout_ms": 1000
        }),
        purpose: String::from("stop current narration"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_stop_speaking_request
            .as_ref()
            .map(|input| input.request_id.as_str()),
        Some("req-stop-speaking")
    );
}

#[test]
fn dispatches_start_listening_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-start-listening"),
        tool_name: ToolName::StartListening,
        arguments: serde_json::json!({
            "request_id": "req-start-listening",
            "timeout_ms": 1500
        }),
        purpose: String::from("start listening"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_start_listening_request
            .as_ref()
            .map(|input| input.request_id.as_str()),
        Some("req-start-listening")
    );
}

#[test]
fn dispatches_stop_listening_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-stop-listening"),
        tool_name: ToolName::StopListening,
        arguments: serde_json::json!({
            "request_id": "req-stop-listening"
        }),
        purpose: String::from("stop listening"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_stop_listening_request
            .as_ref()
            .map(|input| input.request_id.as_str()),
        Some("req-stop-listening")
    );
}

#[test]
fn dispatches_transcribe_command_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-transcribe-command"),
        tool_name: ToolName::TranscribeCommand,
        arguments: serde_json::json!({
            "request_id": "req-transcribe-command",
            "timeout_ms": 2000,
            "max_duration_ms": 3000,
            "stop_mode": "AutoStop"
        }),
        purpose: String::from("transcribe a command"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_transcribe_command_request
            .as_ref()
            .map(|input| input.request_id.as_str()),
        Some("req-transcribe-command")
    );
    assert_eq!(
        executor
            .last_transcribe_command_request
            .as_ref()
            .and_then(|input| input.max_duration_ms),
        Some(3000)
    );
}

#[test]
fn dispatches_capture_screenshot_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-capture-screenshot"),
        tool_name: ToolName::CaptureScreenshot,
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
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_capture_screenshot_request
            .as_ref()
            .map(|input| input.scope),
        Some(ScreenshotScope::Viewport)
    );
    assert_eq!(
        executor
            .last_capture_screenshot_request
            .as_ref()
            .map(|input| input.request_id.as_str()),
        Some("req-capture-screenshot")
    );
    assert_eq!(
        executor
            .last_capture_screenshot_request
            .as_ref()
            .and_then(|input| input.bbox.as_ref())
            .map(|bbox| (bbox.x, bbox.y, bbox.width, bbox.height)),
        Some((10.0, 20.0, 300.0, 120.0))
    );
}

#[test]
fn dispatches_run_ocr_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-run-ocr"),
        tool_name: ToolName::RunOcr,
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
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_run_ocr_request
            .as_ref()
            .and_then(|input| input.image_id.as_deref()),
        Some("image-1")
    );
    assert_eq!(
        executor
            .last_run_ocr_request
            .as_ref()
            .and_then(|input| input.bbox.as_ref())
            .map(|bbox| (bbox.x, bbox.y, bbox.width, bbox.height)),
        Some((4.0, 8.0, 120.0, 48.0))
    );
}

#[test]
fn dispatches_merge_ocr_into_page_model_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-merge-ocr"),
        tool_name: ToolName::MergeOcrIntoPageModel,
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
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_merge_ocr_request
            .as_ref()
            .map(|input| input.page_id.as_str()),
        Some("page-1")
    );
    assert_eq!(
        executor
            .last_merge_ocr_request
            .as_ref()
            .and_then(|input| input.region_id.as_deref()),
        Some("region-2")
    );
}

#[test]
fn dispatches_get_page_snapshot_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-snapshot"),
        tool_name: ToolName::GetPageSnapshot,
        arguments: serde_json::json!({
            "request_id": "req-snapshot",
            "timeout_ms": 1000,
            "include_interactive_elements": true,
            "text_excerpt_max_chars": 120
        }),
        purpose: String::from("read current page snapshot"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_snapshot_request
            .as_ref()
            .map(|input| input.include_interactive_elements),
        Some(true)
    );
    let data = result.data.expect("get_page_snapshot should serialize");
    assert_eq!(
        data.get("page_id"),
        Some(&serde_json::Value::String(String::from("page-1")))
    );
    assert!(data
        .get("interactive_elements")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|elements| !elements.is_empty()));
    assert_eq!(data.get("scroll_y"), Some(&serde_json::json!(120.0)));
    assert_eq!(data.get("viewport_width"), Some(&serde_json::json!(1280.0)));
    assert_eq!(data.get("viewport_height"), Some(&serde_json::json!(720.0)));
    assert_eq!(
        data.get("document_height"),
        Some(&serde_json::json!(2400.0))
    );
}

#[test]
fn dispatches_list_interactive_elements_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-list"),
        tool_name: ToolName::ListInteractiveElements,
        arguments: serde_json::json!({
            "request_id": "req-list",
            "timeout_ms": 1000,
            "visibility_filter": "VisibleOnly",
            "roles": ["Button"]
        }),
        purpose: String::from("list visible buttons"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_list_request
            .as_ref()
            .map(|input| input.visibility_filter),
        Some(ElementVisibilityFilter::VisibleOnly)
    );
    let data = result
        .data
        .expect("list_interactive_elements should serialize");
    assert_eq!(
        data.get("page_id"),
        Some(&serde_json::Value::String(String::from("page-1")))
    );
    assert_eq!(
        data.get("visible_count"),
        Some(&serde_json::Value::Number(serde_json::Number::from(1)))
    );
    assert!(data
        .get("elements")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|elements| elements.len() == 1));
}

#[test]
fn dispatches_find_element_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-find"),
        tool_name: ToolName::FindElement,
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
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_find_request
            .as_ref()
            .and_then(|input| input.role.as_ref()),
        Some(&crate::page_model::ElementRole::Button)
    );
    let data = result.data.expect("find_element should serialize");
    assert_eq!(
        data.get("chosen_element_id"),
        Some(&serde_json::Value::String(String::from("button-1")))
    );
    assert_eq!(
        data.get("requires_confirmation"),
        Some(&serde_json::Value::Bool(false))
    );
}

#[test]
fn dispatches_click_element_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-click"),
        tool_name: ToolName::ClickElement,
        arguments: serde_json::json!({
            "request_id": "req-click",
            "timeout_ms": 1000,
            "element_id": "button-1",
            "click_mode": "Single"
        }),
        purpose: String::from("click the resolved button"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_click_request
            .as_ref()
            .map(|input| input.element_id.as_str()),
        Some("button-1")
    );
    let data = result.data.expect("click_element should serialize");
    assert_eq!(
        data.get("element_id"),
        Some(&serde_json::Value::String(String::from("button-1")))
    );
    assert_eq!(
        data.get("action_performed"),
        Some(&serde_json::Value::Bool(true))
    );
    assert_eq!(
        data.get("page_changed"),
        Some(&serde_json::Value::Bool(false))
    );
}

#[test]
fn dispatches_focus_element_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-focus"),
        tool_name: ToolName::FocusElement,
        arguments: serde_json::json!({
            "request_id": "req-focus",
            "timeout_ms": 1000,
            "element_id": "input-1"
        }),
        purpose: String::from("focus the resolved field"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_focus_request
            .as_ref()
            .map(|input| input.element_id.as_str()),
        Some("input-1")
    );
    let data = result.data.expect("focus_element should serialize");
    assert_eq!(data.get("focused"), Some(&serde_json::Value::Bool(true)));
}

#[test]
fn dispatches_type_into_element_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-type"),
        tool_name: ToolName::TypeIntoElement,
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
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_type_request
            .as_ref()
            .map(|input| input.text.as_str()),
        Some("phil@example.com")
    );
    let data = result.data.expect("type_into_element should serialize");
    assert_eq!(
        data.get("accepted_input"),
        Some(&serde_json::Value::Bool(true))
    );
}

#[test]
fn dispatches_submit_active_form_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-submit"),
        tool_name: ToolName::SubmitActiveForm,
        arguments: serde_json::json!({
            "request_id": "req-submit",
            "timeout_ms": 1000,
            "form_element_id": "form-login"
        }),
        purpose: String::from("submit the active form"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_submit_request
            .as_ref()
            .and_then(|input| input.form_element_id.as_deref()),
        Some("form-login")
    );
    let data = result.data.expect("submit_active_form should serialize");
    assert_eq!(data.get("submitted"), Some(&serde_json::Value::Bool(true)));
    assert_eq!(
        data.get("page_changed"),
        Some(&serde_json::Value::Bool(true))
    );
}

#[test]
fn dispatches_extract_page_model_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-extract"),
        tool_name: ToolName::ExtractPageModel,
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
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor
            .last_extract_request
            .as_ref()
            .map(|input| input.include_links),
        Some(false)
    );
    let data = result.data.expect("extract_page_model should serialize");
    assert_eq!(
        data.get("extraction_source"),
        Some(&serde_json::Value::String(String::from("DomFallback")))
    );
    assert!(data
        .get("page_model")
        .and_then(|model| model.get("interactive_elements"))
        .and_then(serde_json::Value::as_array)
        .is_some_and(|elements| elements.is_empty()));
}

#[test]
fn rejects_invalid_tool_arguments_before_dispatch() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-2"),
        tool_name: ToolName::SetPlaybackSpeed,
        arguments: serde_json::json!({
            "request_id": "req-2",
            "timeout_ms": 1000,
            "speed": "fast"
        }),
        purpose: String::from("update speed"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(!result.ok);
    assert_eq!(result.tool_name, ToolName::SetPlaybackSpeed);
    assert_eq!(result.request_id, "req-2");
    assert_eq!(
        result.error.expect("error should be present").code,
        "invalid_tool_arguments"
    );
    assert_eq!(executor.last_speed, None);
}

#[test]
fn validate_planned_step_arguments_reports_schema_mismatch_details() {
    let step = PlannedStep {
        step_id: String::from("step-speed"),
        tool_name: ToolName::SetPlaybackSpeed,
        arguments: serde_json::json!({
            "request_id": "req-speed",
            "speed": "fast"
        }),
        purpose: String::from("set playback speed"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let error = validate_planned_step_arguments(&step)
        .expect_err("validation should reject malformed step arguments");

    assert_eq!(error.code, "invalid_planner_output");
    assert!(error.message.contains("expected schema"));
    assert_eq!(
        error
            .details
            .as_ref()
            .and_then(|details| details.get("step_id")),
        Some(&serde_json::json!("step-speed"))
    );
    assert_eq!(
        error
            .details
            .as_ref()
            .and_then(|details| details.get("tool_name")),
        Some(&serde_json::json!("SetPlaybackSpeed"))
    );
}

#[test]
fn dispatches_set_browser_visibility_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-3"),
        tool_name: ToolName::SetBrowserVisibility,
        arguments: serde_json::json!({
            "request_id": "req-3",
            "timeout_ms": 1000,
            "mode": "Headless"
        }),
        purpose: String::from("toggle browser visibility"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor.last_visibility,
        Some(BrowserVisibilityMode::Headless)
    );
    assert_eq!(result.tool_name, ToolName::SetBrowserVisibility);
}

#[test]
fn dispatches_get_runtime_status_with_provider_modes() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-4"),
        tool_name: ToolName::GetRuntimeStatus,
        arguments: serde_json::json!({
            "request_id": "req-4",
            "timeout_ms": 1000,
            "include_provider_modes": true
        }),
        purpose: String::from("read runtime status"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    let data = result.data.expect("runtime status should serialize");
    assert!(data.get("provider_modes").is_some());
}

#[test]
fn set_playback_volume_clamps_requested_value_and_updates_readback() {
    let mut executor = MockExecutor::default();

    let result = executor.execute_set_playback_volume(SetPlaybackVolumeInput {
        request_id: String::from("req-volume-clamp"),
        timeout_ms: Some(1_000),
        volume: -0.25,
    });

    assert!(result.ok);
    assert_eq!(
        result.observations,
        vec![
            String::from("Updated the playback volume setting."),
            String::from("New narration requests will use the updated playback volume."),
            String::from("Requested playback volume was clamped to the supported range."),
        ]
    );
    assert_eq!(
        result.data.expect("volume tool should return data"),
        SetPlaybackVolumeData {
            playback_volume: 0.0,
            muted: true,
            changed: true,
        }
    );
    assert_eq!(executor.last_volume, Some(-0.25));

    let state = executor.execute_get_agent_state(GetAgentStateInput {
        request_id: String::from("req-agent-state"),
        timeout_ms: Some(1_000),
        include_last_transcript: false,
    });
    assert!(state.ok);
    let state_data = state.data.expect("agent state should return data");
    assert_eq!(state_data.audio.playback_volume, 0.0);
    assert!(state_data.audio.muted);
}

#[test]
fn set_playback_speed_clamps_requested_value_and_updates_readback() {
    let mut executor = MockExecutor::default();

    let result = executor.execute_set_playback_speed(SetPlaybackSpeedInput {
        request_id: String::from("req-speed-clamp"),
        timeout_ms: Some(1_000),
        speed: 9.0,
    });

    assert!(result.ok);
    assert_eq!(
        result.observations,
        vec![
            String::from("Updated the playback speed setting."),
            String::from("New narration requests will use the updated native TTS speed."),
            String::from("Requested playback speed was clamped to the supported range."),
        ]
    );
    assert_eq!(
        result.data.expect("speed tool should return data"),
        SetPlaybackSpeedData {
            playback_speed: crate::config::MAX_PLAYBACK_SPEED,
            changed: true,
        }
    );
    assert_eq!(executor.last_speed, Some(9.0));

    let status = executor.execute_get_runtime_status(GetRuntimeStatusInput {
        request_id: String::from("req-runtime-status"),
        timeout_ms: Some(1_000),
        include_provider_modes: false,
    });
    assert!(status.ok);
    let status_data = status.data.expect("runtime status should return data");
    assert_eq!(
        status_data.audio.playback_speed,
        crate::config::MAX_PLAYBACK_SPEED
    );
}

#[test]
fn set_browser_visibility_reports_no_change_when_mode_is_already_active() {
    let mut executor = MockExecutor {
        browser_visibility: BrowserVisibilityMode::Headless,
        ..Default::default()
    };

    let result = executor.execute_set_browser_visibility(SetBrowserVisibilityInput {
        request_id: String::from("req-visibility-noop"),
        timeout_ms: Some(1_000),
        mode: BrowserVisibilityMode::Headless,
    });

    assert!(result.ok);
    assert_eq!(
        result.data.expect("visibility tool should return data"),
        SetBrowserVisibilityData {
            mode: BrowserVisibilityMode::Headless,
            changed: false,
            supported: true,
        }
    );
    assert_eq!(
        result.observations,
        vec![String::from(
            "Browser visibility mode is already set to the requested value.",
        )]
    );
    assert_eq!(
        executor.current_browser_visibility(),
        BrowserVisibilityMode::Headless
    );
}

#[test]
fn set_browser_visibility_reports_unsupported_when_switching_is_disabled() {
    let mut executor = MockExecutor {
        browser_visibility_switch_supported: false,
        ..Default::default()
    };

    let result = executor.execute_set_browser_visibility(SetBrowserVisibilityInput {
        request_id: String::from("req-visibility-unsupported"),
        timeout_ms: Some(1_000),
        mode: BrowserVisibilityMode::Headless,
    });

    assert!(result.ok);
    assert_eq!(
        result
            .data
            .expect("unsupported visibility tool should return data"),
        SetBrowserVisibilityData {
            mode: BrowserVisibilityMode::Visible,
            changed: false,
            supported: false,
        }
    );
    assert_eq!(
        result.observations,
        vec![String::from(
            "Browser visibility switching is not supported in this build.",
        )]
    );

    let state = executor.execute_get_agent_state(GetAgentStateInput {
        request_id: String::from("req-visibility-state"),
        timeout_ms: Some(1_000),
        include_last_transcript: false,
    });
    assert!(state.ok);
    let state_data = state.data.expect("agent state should return data");
    assert_eq!(
        state_data.browser_visibility,
        BrowserVisibilityMode::Visible
    );
}

#[test]
fn provider_selection_status_round_trips_with_snake_case_modes() {
    let status = ProviderSelectionStatus {
        planner_mode: ProviderMode::Remote,
        tts_mode: ProviderMode::Local,
        asr_mode: ProviderMode::Local,
    };

    let serialized =
        serde_json::to_value(&status).expect("provider selection status should serialize");
    assert_eq!(
        serialized,
        serde_json::json!({
            "planner_mode": "remote",
            "tts_mode": "local",
            "asr_mode": "local"
        })
    );

    let round_tripped: ProviderSelectionStatus =
        serde_json::from_value(serialized).expect("provider selection status should deserialize");
    assert_eq!(round_tripped, status);
}

#[test]
fn shared_command_enums_round_trip_and_reject_invalid_variants() {
    fn assert_enum_round_trip<T>(value: T, expected: serde_json::Value, invalid: serde_json::Value)
    where
        T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        let serialized = serde_json::to_value(&value).expect("enum should serialize");
        assert_eq!(serialized, expected);

        let round_tripped: T = serde_json::from_value(serialized).expect("enum should deserialize");
        assert_eq!(round_tripped, value);
        assert!(serde_json::from_value::<T>(invalid).is_err());
    }

    assert_enum_round_trip(
        NarrationInterruptionMode::Interrupt,
        serde_json::json!("Interrupt"),
        serde_json::json!("Pause"),
    );
    assert_enum_round_trip(
        NarrationBoundary::End,
        serde_json::json!("End"),
        serde_json::json!("Middle"),
    );
    assert_enum_round_trip(
        ElementVisibilityFilter::VisibleOnly,
        serde_json::json!("VisibleOnly"),
        serde_json::json!("HiddenOnly"),
    );
    assert_enum_round_trip(
        ReloadMode::Hard,
        serde_json::json!("Hard"),
        serde_json::json!("Soft"),
    );
    assert_enum_round_trip(
        ClickMode::Double,
        serde_json::json!("Double"),
        serde_json::json!("Triple"),
    );
    assert_enum_round_trip(
        TextEntryMode::Replace,
        serde_json::json!("Replace"),
        serde_json::json!("Overwrite"),
    );
    assert_enum_round_trip(
        TextEntrySubmitMode::Submit,
        serde_json::json!("Submit"),
        serde_json::json!("Enter"),
    );
    assert_enum_round_trip(
        TranscriptionStopMode::AutoStop,
        serde_json::json!("AutoStop"),
        serde_json::json!("ManualStop"),
    );
    assert_enum_round_trip(
        ScreenshotScope::FullPage,
        serde_json::json!("FullPage"),
        serde_json::json!("Region"),
    );
    assert_enum_round_trip(
        ReportStatus::NeedsFollowUp,
        serde_json::json!("NeedsFollowUp"),
        serde_json::json!("Retry"),
    );
    assert_enum_round_trip(
        BrowserVisibilityMode::Headless,
        serde_json::json!("Headless"),
        serde_json::json!("Minimized"),
    );
}

#[test]
fn get_runtime_status_result_matches_schema_with_provider_modes() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-runtime-schema"),
        tool_name: ToolName::GetRuntimeStatus,
        arguments: serde_json::json!({
            "request_id": "req-runtime-schema",
            "timeout_ms": 1000,
            "include_provider_modes": true
        }),
        purpose: String::from("read runtime status with provider modes"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);
    assert!(result.ok);

    let serialized =
        serde_json::to_value(&result).expect("runtime status tool result should serialize");
    let schema = tool_output_schema(&ToolName::GetRuntimeStatus)
        .expect("get_runtime_status should expose an output schema");
    assert_json_matches_schema(&serialized, &schema)
        .expect("serialized get_runtime_status result should match its output schema");

    let provider_modes = serialized
        .get("data")
        .and_then(|data| data.get("provider_modes"))
        .expect("provider_modes should be present when requested");
    assert_eq!(
        provider_modes,
        &serde_json::json!({
            "planner_mode": "remote",
            "tts_mode": "local",
            "asr_mode": "local"
        })
    );
}

#[test]
fn get_runtime_status_reports_null_provider_modes_when_not_requested() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-runtime-no-provider-modes"),
        tool_name: ToolName::GetRuntimeStatus,
        arguments: serde_json::json!({
            "request_id": "req-runtime-no-provider-modes",
            "timeout_ms": 1000,
            "include_provider_modes": false
        }),
        purpose: String::from("read runtime status without provider modes"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);
    assert!(result.ok);

    let serialized =
        serde_json::to_value(&result).expect("runtime status tool result should serialize");
    let provider_modes = serialized
        .get("data")
        .and_then(|data| data.get("provider_modes"))
        .expect("provider_modes field should still be present in serialized output");
    assert_eq!(provider_modes, &serde_json::Value::Null);
}

#[test]
fn browser_visibility_changes_are_reflected_in_following_state_reads() {
    let mut executor = MockExecutor::default();
    let set_visibility_step = PlannedStep {
        step_id: String::from("step-visibility"),
        tool_name: ToolName::SetBrowserVisibility,
        arguments: serde_json::json!({
            "request_id": "req-visibility",
            "timeout_ms": 1000,
            "mode": "Headless"
        }),
        purpose: String::from("toggle browser visibility"),
        on_success: StepTransition::NextStep {
            step_id: String::from("step-runtime"),
        },
        on_failure: StepTransition::Replan,
    };
    let runtime_status_step = PlannedStep {
        step_id: String::from("step-runtime"),
        tool_name: ToolName::GetRuntimeStatus,
        arguments: serde_json::json!({
            "request_id": "req-runtime",
            "timeout_ms": 1000,
            "include_provider_modes": false
        }),
        purpose: String::from("read runtime status"),
        on_success: StepTransition::NextStep {
            step_id: String::from("step-agent"),
        },
        on_failure: StepTransition::Replan,
    };
    let agent_state_step = PlannedStep {
        step_id: String::from("step-agent"),
        tool_name: ToolName::GetAgentState,
        arguments: serde_json::json!({
            "request_id": "req-agent",
            "timeout_ms": 1000,
            "include_last_transcript": true
        }),
        purpose: String::from("read agent state"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let set_visibility_result = execute_planned_step(&mut executor, &set_visibility_step);
    let runtime_status_result = execute_planned_step(&mut executor, &runtime_status_step);
    let agent_state_result = execute_planned_step(&mut executor, &agent_state_step);

    assert!(set_visibility_result.ok);
    assert!(runtime_status_result.ok);
    assert!(agent_state_result.ok);
    assert_eq!(
        runtime_status_result
            .data
            .as_ref()
            .and_then(|data| data.get("browser_visibility")),
        Some(&serde_json::json!("Headless"))
    );
    assert_eq!(
        agent_state_result
            .data
            .as_ref()
            .and_then(|data| data.get("browser_visibility")),
        Some(&serde_json::json!("Headless"))
    );
}

#[test]
fn listening_tools_update_following_runtime_state_reads() {
    let mut executor = MockExecutor::default();
    let start_listening_step = PlannedStep {
        step_id: String::from("step-start-listening"),
        tool_name: ToolName::StartListening,
        arguments: serde_json::json!({
            "request_id": "req-start-listening",
            "timeout_ms": 1500
        }),
        purpose: String::from("start listening"),
        on_success: StepTransition::NextStep {
            step_id: String::from("step-runtime"),
        },
        on_failure: StepTransition::Replan,
    };
    let runtime_status_step = PlannedStep {
        step_id: String::from("step-runtime"),
        tool_name: ToolName::GetRuntimeStatus,
        arguments: serde_json::json!({
            "request_id": "req-runtime",
            "timeout_ms": 1000,
            "include_provider_modes": false
        }),
        purpose: String::from("read runtime status"),
        on_success: StepTransition::NextStep {
            step_id: String::from("step-stop-listening"),
        },
        on_failure: StepTransition::Replan,
    };
    let stop_listening_step = PlannedStep {
        step_id: String::from("step-stop-listening"),
        tool_name: ToolName::StopListening,
        arguments: serde_json::json!({
            "request_id": "req-stop-listening"
        }),
        purpose: String::from("stop listening"),
        on_success: StepTransition::NextStep {
            step_id: String::from("step-agent"),
        },
        on_failure: StepTransition::Replan,
    };
    let agent_state_step = PlannedStep {
        step_id: String::from("step-agent"),
        tool_name: ToolName::GetAgentState,
        arguments: serde_json::json!({
            "request_id": "req-agent",
            "timeout_ms": 1000,
            "include_last_transcript": true
        }),
        purpose: String::from("read agent state"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let start_result = execute_planned_step(&mut executor, &start_listening_step);
    let runtime_status_result = execute_planned_step(&mut executor, &runtime_status_step);
    let stop_result = execute_planned_step(&mut executor, &stop_listening_step);
    let agent_state_result = execute_planned_step(&mut executor, &agent_state_step);

    assert!(start_result.ok);
    assert!(runtime_status_result.ok);
    assert!(stop_result.ok);
    assert!(agent_state_result.ok);
    assert_eq!(
        runtime_status_result
            .data
            .as_ref()
            .and_then(|data| data.get("listening_state"))
            .and_then(|state| state.get("is_listening")),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        agent_state_result
            .data
            .as_ref()
            .and_then(|data| data.get("listening_state"))
            .and_then(|state| state.get("is_listening")),
        Some(&serde_json::json!(false))
    );
}

#[test]
fn transcribe_command_updates_following_state_reads_for_auto_stop_and_manual_stop() {
    for (request_id, stop_mode, expected_listening_after_transcribe) in [
        (
            "req-transcribe-auto",
            TranscriptionStopMode::AutoStop,
            false,
        ),
        (
            "req-transcribe-manual",
            TranscriptionStopMode::KeepListening,
            true,
        ),
    ] {
        let mut executor = MockExecutor::default();
        let transcribe_step = PlannedStep {
            step_id: format!("step-{request_id}"),
            tool_name: ToolName::TranscribeCommand,
            arguments: serde_json::json!({
                "request_id": request_id,
                "timeout_ms": 2000,
                "max_duration_ms": 3000,
                "stop_mode": stop_mode
            }),
            purpose: String::from("transcribe a command"),
            on_success: StepTransition::NextStep {
                step_id: String::from("step-runtime"),
            },
            on_failure: StepTransition::Replan,
        };
        let runtime_status_step = PlannedStep {
            step_id: String::from("step-runtime"),
            tool_name: ToolName::GetRuntimeStatus,
            arguments: serde_json::json!({
                "request_id": format!("{request_id}-runtime"),
                "timeout_ms": 1000,
                "include_provider_modes": false
            }),
            purpose: String::from("read runtime status"),
            on_success: StepTransition::NextStep {
                step_id: String::from("step-agent"),
            },
            on_failure: StepTransition::Replan,
        };
        let agent_state_step = PlannedStep {
            step_id: String::from("step-agent"),
            tool_name: ToolName::GetAgentState,
            arguments: serde_json::json!({
                "request_id": format!("{request_id}-agent"),
                "timeout_ms": 1000,
                "include_last_transcript": true
            }),
            purpose: String::from("read agent state"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        let transcribe_result = execute_planned_step(&mut executor, &transcribe_step);
        let runtime_status_result = execute_planned_step(&mut executor, &runtime_status_step);
        let agent_state_result = execute_planned_step(&mut executor, &agent_state_step);

        assert!(transcribe_result.ok);
        assert!(runtime_status_result.ok);
        assert!(agent_state_result.ok);
        assert_eq!(
            runtime_status_result
                .data
                .as_ref()
                .and_then(|data| data.get("listening_state"))
                .and_then(|state| state.get("is_listening")),
            Some(&serde_json::json!(expected_listening_after_transcribe))
        );
        assert_eq!(
            agent_state_result
                .data
                .as_ref()
                .and_then(|data| data.get("listening_state"))
                .and_then(|state| state.get("is_listening")),
            Some(&serde_json::json!(expected_listening_after_transcribe))
        );
        assert_eq!(
            agent_state_result
                .data
                .as_ref()
                .and_then(|data| data.get("last_transcript")),
            Some(&serde_json::json!("read the next section"))
        );
    }
}

#[test]
fn browser_history_navigation_updates_following_state_reads() {
    let mut executor = MockExecutor::default();
    let go_back_step = PlannedStep {
        step_id: String::from("step-go-back"),
        tool_name: ToolName::GoBack,
        arguments: serde_json::json!({
            "request_id": "req-go-back",
            "timeout_ms": 1000,
            "steps": 1,
            "wait_for_load_state": "Load"
        }),
        purpose: String::from("go back in history"),
        on_success: StepTransition::NextStep {
            step_id: String::from("step-runtime-after-back"),
        },
        on_failure: StepTransition::Replan,
    };
    let runtime_after_back_step = PlannedStep {
        step_id: String::from("step-runtime-after-back"),
        tool_name: ToolName::GetRuntimeStatus,
        arguments: serde_json::json!({
            "request_id": "req-runtime-after-back",
            "timeout_ms": 1000,
            "include_provider_modes": false
        }),
        purpose: String::from("read runtime status"),
        on_success: StepTransition::NextStep {
            step_id: String::from("step-go-forward"),
        },
        on_failure: StepTransition::Replan,
    };
    let go_forward_step = PlannedStep {
        step_id: String::from("step-go-forward"),
        tool_name: ToolName::GoForward,
        arguments: serde_json::json!({
            "request_id": "req-go-forward",
            "timeout_ms": 1000,
            "steps": 1,
            "wait_for_load_state": "Load"
        }),
        purpose: String::from("go forward in history"),
        on_success: StepTransition::NextStep {
            step_id: String::from("step-runtime-after-forward"),
        },
        on_failure: StepTransition::Replan,
    };
    let runtime_after_forward_step = PlannedStep {
        step_id: String::from("step-runtime-after-forward"),
        tool_name: ToolName::GetRuntimeStatus,
        arguments: serde_json::json!({
            "request_id": "req-runtime-after-forward",
            "timeout_ms": 1000,
            "include_provider_modes": false
        }),
        purpose: String::from("read runtime status"),
        on_success: StepTransition::NextStep {
            step_id: String::from("step-reload"),
        },
        on_failure: StepTransition::Replan,
    };
    let reload_step = PlannedStep {
        step_id: String::from("step-reload"),
        tool_name: ToolName::ReloadPage,
        arguments: serde_json::json!({
            "request_id": "req-reload",
            "timeout_ms": 1000,
            "mode": "Hard",
            "wait_for_load_state": "Load"
        }),
        purpose: String::from("reload the current page"),
        on_success: StepTransition::NextStep {
            step_id: String::from("step-agent-after-reload"),
        },
        on_failure: StepTransition::Replan,
    };
    let agent_after_reload_step = PlannedStep {
        step_id: String::from("step-agent-after-reload"),
        tool_name: ToolName::GetAgentState,
        arguments: serde_json::json!({
            "request_id": "req-agent-after-reload",
            "timeout_ms": 1000,
            "include_last_transcript": false
        }),
        purpose: String::from("read agent state"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let go_back_result = execute_planned_step(&mut executor, &go_back_step);
    let runtime_after_back_result = execute_planned_step(&mut executor, &runtime_after_back_step);
    let go_forward_result = execute_planned_step(&mut executor, &go_forward_step);
    let runtime_after_forward_result =
        execute_planned_step(&mut executor, &runtime_after_forward_step);
    let reload_result = execute_planned_step(&mut executor, &reload_step);
    let agent_after_reload_result = execute_planned_step(&mut executor, &agent_after_reload_step);

    assert!(go_back_result.ok);
    assert!(runtime_after_back_result.ok);
    assert!(go_forward_result.ok);
    assert!(runtime_after_forward_result.ok);
    assert!(reload_result.ok);
    assert!(agent_after_reload_result.ok);
    assert_eq!(
        runtime_after_back_result
            .data
            .as_ref()
            .and_then(|data| data.get("browser_history"))
            .and_then(|history| history.get("can_go_back")),
        Some(&serde_json::json!(false))
    );
    assert_eq!(
        runtime_after_back_result
            .data
            .as_ref()
            .and_then(|data| data.get("browser_history"))
            .and_then(|history| history.get("can_go_forward")),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        runtime_after_forward_result
            .data
            .as_ref()
            .and_then(|data| data.get("browser_history"))
            .and_then(|history| history.get("can_go_back")),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        runtime_after_forward_result
            .data
            .as_ref()
            .and_then(|data| data.get("browser_history"))
            .and_then(|history| history.get("can_go_forward")),
        Some(&serde_json::json!(false))
    );
    assert_eq!(
        agent_after_reload_result
            .data
            .as_ref()
            .and_then(|data| data.get("browser_history"))
            .and_then(|history| history.get("current_entry_index")),
        Some(&serde_json::json!(1))
    );
    assert_eq!(
        agent_after_reload_result
            .data
            .as_ref()
            .and_then(|data| data.get("browser_history"))
            .and_then(|history| history.get("entry_count")),
        Some(&serde_json::json!(2))
    );
}

#[test]
fn dispatches_get_agent_state_without_last_transcript() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-5"),
        tool_name: ToolName::GetAgentState,
        arguments: serde_json::json!({
            "request_id": "req-5",
            "timeout_ms": 1000,
            "include_last_transcript": false
        }),
        purpose: String::from("read agent state"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    let data = result.data.expect("agent state should serialize");
    assert_eq!(data.get("last_transcript"), Some(&serde_json::Value::Null));
    assert_eq!(
        data.get("last_tool_call")
            .and_then(|entry| entry.get("tool_name")),
        Some(&serde_json::Value::String(String::from("GetAgentState")))
    );
}

#[test]
fn dispatches_confirm_action_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-confirm"),
        tool_name: ToolName::ConfirmAction,
        arguments: serde_json::json!({
            "request_id": "req-confirm-dispatch",
            "timeout_ms": 1000,
            "prompt_text": "Do you want me to continue?",
            "reason": "The next step may submit data."
        }),
        purpose: String::from("request confirmation"),
        on_success: StepTransition::RequestConfirmation,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(
        executor.last_confirmation_prompt.as_deref(),
        Some("Do you want me to continue?")
    );
    let data = result.data.expect("confirm_action should serialize");
    assert_eq!(
        data.get("confirmation_id"),
        Some(&serde_json::Value::String(String::from("confirm-1")))
    );
}

#[test]
fn dispatches_report_result_from_planned_step() {
    let mut executor = MockExecutor::default();
    let step = PlannedStep {
        step_id: String::from("step-report"),
        tool_name: ToolName::ReportResult,
        arguments: serde_json::json!({
            "request_id": "req-report",
            "timeout_ms": 1000,
            "status": "Success",
            "summary": "Opened the requested page.",
            "next_recommended_action": null,
            "user_message": "The page is ready."
        }),
        purpose: String::from("report completion"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let result = execute_planned_step(&mut executor, &step);

    assert!(result.ok);
    assert_eq!(result.tool_name, ToolName::ReportResult);
    assert_eq!(
        executor.last_report_result,
        Some(ReportResultData {
            status: ReportStatus::Success,
            summary: String::from("Opened the requested page."),
            next_recommended_action: None,
            user_message: Some(String::from("The page is ready.")),
        })
    );
    let data = result.data.expect("report_result should serialize");
    assert_eq!(
        data.get("status"),
        Some(&serde_json::Value::String(String::from("Success")))
    );
}

#[test]
fn executes_next_step_chain_until_complete() {
    let mut executor = MockExecutor::default();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::SetPlaybackVolume,
            goal: String::from("adjust audio"),
            target_description: None,
        },
        selected_skills: vec![String::from("audio_controls")],
        steps: vec![
            PlannedStep {
                step_id: String::from("step-1"),
                tool_name: ToolName::SetPlaybackVolume,
                arguments: serde_json::json!({
                    "request_id": "req-plan",
                    "timeout_ms": 1000,
                    "volume": 0.4
                }),
                purpose: String::from("set the volume"),
                on_success: StepTransition::NextStep {
                    step_id: String::from("step-2"),
                },
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from("step-2"),
                tool_name: ToolName::GetRuntimeStatus,
                arguments: serde_json::json!({
                    "request_id": "req-plan",
                    "timeout_ms": 1000,
                    "include_provider_modes": false
                }),
                purpose: String::from("read back the runtime state"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let outcome = execute_planner_output(&mut executor, String::from("req-plan"), &planner_output);

    match outcome {
        ExecutionOutcome::Complete { trace } => {
            assert_eq!(trace.executed_step_ids, vec!["step-1", "step-2"]);
            assert_eq!(trace.tool_results.len(), 2);
        }
        other => panic!("expected complete outcome, got {other:?}"),
    }
}

#[test]
fn executes_load_page_extract_and_read_flow_from_resolved_read_page_plan() {
    let mut executor = MockExecutor::default();
    let page_model = fixture_problematic_article_page_without_regions();
    let agent_state = fixture_agent_state_for_page(
        "Metro news | Night trains finally return",
        "https://news.example.com/city/night-trains-return",
    );
    let planner_output = resolve_direct_read_page_command(
        "read page",
        "req-load-extract-read",
        Some(&page_model),
        &agent_state,
        &[String::from("read_page")],
    )
    .expect("read-page command should resolve");
    let expected_step_ids = planner_output
        .steps
        .iter()
        .map(|step| step.step_id.clone())
        .collect::<Vec<_>>();
    let expected_extract_input: ExtractPageModelInput =
        serde_json::from_value(planner_output.steps[0].arguments.clone())
            .expect("extract step should deserialize");
    let expected_read_next_input: ReadNextRegionInput =
        serde_json::from_value(planner_output.steps[1].arguments.clone())
            .expect("read-next step should deserialize");

    let outcome = execute_planner_output(
        &mut executor,
        String::from("req-load-extract-read"),
        &planner_output,
    );

    match outcome {
        ExecutionOutcome::Complete { trace } => {
            assert_eq!(trace.executed_step_ids, expected_step_ids);
            assert_eq!(
                trace
                    .tool_results
                    .iter()
                    .map(|result| result.tool_name.clone())
                    .collect::<Vec<_>>(),
                vec![ToolName::ExtractPageModel, ToolName::ReadNextRegion]
            );
            assert_eq!(executor.last_extract_request, Some(expected_extract_input));
            assert_eq!(
                executor.last_read_next_region_request,
                Some(expected_read_next_input)
            );
        }
        other => panic!("expected complete outcome, got {other:?}"),
    }
}

#[test]
fn executes_resolved_spoken_command_action_flow_for_continue_reading() {
    let mut executor = MockExecutor::default();
    let planner_output = resolve_direct_navigation_readback_command(
        "continue reading",
        "req-asr-command-action",
        &[String::from("read_next")],
    )
    .expect("continue-reading command should resolve");
    let expected_step_ids = planner_output
        .steps
        .iter()
        .map(|step| step.step_id.clone())
        .collect::<Vec<_>>();

    let outcome = execute_planner_output(
        &mut executor,
        String::from("req-asr-command-action"),
        &planner_output,
    );

    match outcome {
        ExecutionOutcome::Complete { trace } => {
            assert_eq!(planner_output.intent.name, IntentName::ReadNext);
            assert_eq!(trace.executed_step_ids, expected_step_ids);
            assert_eq!(trace.tool_results.len(), 1);
            assert_eq!(trace.tool_results[0].tool_name, ToolName::ReadNextRegion);
            assert_eq!(
                executor.last_read_next_region_request,
                Some(ReadNextRegionInput {
                    request_id: String::from("req-asr-command-action"),
                    timeout_ms: None,
                    interruption_mode: NarrationInterruptionMode::Interrupt,
                })
            );
        }
        other => panic!("expected complete outcome, got {other:?}"),
    }
}

#[test]
fn follows_failure_transition_to_replan() {
    let mut executor = MockExecutor::default();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::SetPlaybackSpeed,
            goal: String::from("adjust playback speed"),
            target_description: None,
        },
        selected_skills: vec![String::from("audio_controls")],
        steps: vec![PlannedStep {
            step_id: String::from("step-1"),
            tool_name: ToolName::SetPlaybackSpeed,
            arguments: serde_json::json!({
                "request_id": "req-replan",
                "timeout_ms": 1000,
                "speed": "fast"
            }),
            purpose: String::from("set invalid speed"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let outcome =
        execute_planner_output(&mut executor, String::from("req-replan"), &planner_output);

    match outcome {
        ExecutionOutcome::NeedsReplan { trace } => {
            assert_eq!(trace.executed_step_ids, vec!["step-1"]);
            assert_eq!(trace.tool_results.len(), 1);
            assert!(!trace.tool_results[0].ok);
        }
        other => panic!("expected replan outcome, got {other:?}"),
    }
}

#[test]
fn returns_awaiting_confirmation_when_transition_requests_it() {
    let planner_output = PlannerOutput {
        status: PlannerStatus::NeedsConfirmation,
        intent: IntentSummary {
            name: IntentName::ClickElement,
            goal: String::from("confirm button choice"),
            target_description: Some(String::from("submit button")),
        },
        selected_skills: vec![String::from("confirm_action")],
        steps: vec![
            PlannedStep {
                step_id: String::from("step-1"),
                tool_name: ToolName::ConfirmAction,
                arguments: serde_json::json!({
                    "request_id": "req-confirm"
                }),
                purpose: String::from("ask for confirmation"),
                on_success: StepTransition::RequestConfirmation,
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from("step-2"),
                tool_name: ToolName::SetBrowserVisibility,
                arguments: serde_json::json!({
                    "request_id": "req-confirm",
                    "timeout_ms": 1000,
                    "mode": "Visible"
                }),
                purpose: String::from("placeholder protected step"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ],
        requires_confirmation: true,
        confirmation_reason: Some(String::from("protected action")),
        blocked_reason: None,
        user_message: Some(String::from("Please confirm.")),
    };

    let outcome =
        execute_planner_output_with_runner(String::from("req-confirm"), &planner_output, |step| {
            assert_eq!(step.step_id, "step-1");
            ToolResult::success(
                ToolName::ConfirmAction,
                String::from("req-confirm"),
                serde_json::json!({
                    "confirmation_id": "confirm-1",
                    "prompt_text": "Proceed?",
                    "confirmed": serde_json::Value::Null,
                    "timed_out": false
                }),
                vec![String::from("confirmation requested")],
            )
        });

    match outcome {
        ExecutionOutcome::AwaitingConfirmation {
            trace,
            pending_confirmation_id,
            pending_plan_execution,
        } => {
            assert_eq!(trace.executed_step_ids, vec!["step-1"]);
            assert_eq!(pending_confirmation_id, "confirm-1");
            assert_eq!(pending_plan_execution.request_id, "req-confirm");
            assert_eq!(pending_plan_execution.intent_name, IntentName::ClickElement);
            assert_eq!(pending_plan_execution.prompt_text, "Proceed?");
            assert_eq!(
                pending_plan_execution.next_step_id,
                Some(String::from("step-2"))
            );
            assert_eq!(pending_plan_execution.queued_step_ids, vec!["step-2"]);
            assert_eq!(pending_plan_execution.queued_steps.len(), 1);
            assert_eq!(pending_plan_execution.queued_steps[0].step_id, "step-2");
        }
        other => panic!("expected awaiting confirmation outcome, got {other:?}"),
    }
}

#[test]
fn resumes_confirmed_pending_execution_from_stored_steps() {
    let mut executor = MockExecutor::default();
    let pending_plan_execution = PendingPlanExecutionState {
        request_id: String::from("req-resume"),
        intent_name: IntentName::SetBrowserVisibility,
        selected_skills: vec![String::from("confirm_action")],
        confirmation_id: String::from("confirm-1"),
        prompt_text: String::from("Proceed?"),
        next_step_id: Some(String::from("step-2")),
        queued_step_ids: vec![String::from("step-2")],
        queued_steps: vec![PlannedStep {
            step_id: String::from("step-2"),
            tool_name: ToolName::SetBrowserVisibility,
            arguments: serde_json::json!({
                "request_id": "req-resume",
                "timeout_ms": 1000,
                "mode": "Headless"
            }),
            purpose: String::from("apply confirmed action"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
    };

    let outcome =
        resume_after_confirmation(&mut executor, &pending_plan_execution, "confirm-1", true);

    match outcome {
        ExecutionOutcome::Complete { trace } => {
            assert_eq!(trace.executed_step_ids, vec!["step-2"]);
            assert_eq!(
                executor.last_visibility,
                Some(BrowserVisibilityMode::Headless)
            );
        }
        other => panic!("expected complete outcome after resume, got {other:?}"),
    }
}

#[test]
fn resumes_rejected_confirmation_to_replan() {
    let mut executor = MockExecutor::default();
    let pending_plan_execution = PendingPlanExecutionState {
        request_id: String::from("req-resume"),
        intent_name: IntentName::SetBrowserVisibility,
        selected_skills: vec![String::from("confirm_action")],
        confirmation_id: String::from("confirm-1"),
        prompt_text: String::from("Proceed?"),
        next_step_id: Some(String::from("step-2")),
        queued_step_ids: vec![String::from("step-2")],
        queued_steps: vec![PlannedStep {
            step_id: String::from("step-2"),
            tool_name: ToolName::SetBrowserVisibility,
            arguments: serde_json::json!({
                "request_id": "req-resume",
                "timeout_ms": 1000,
                "mode": "Headless"
            }),
            purpose: String::from("apply confirmed action"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
    };

    let outcome =
        resume_after_confirmation(&mut executor, &pending_plan_execution, "confirm-1", false);

    match outcome {
        ExecutionOutcome::NeedsReplan { trace } => {
            assert!(trace.executed_step_ids.is_empty());
            assert_eq!(executor.last_visibility, None);
        }
        other => panic!("expected replan outcome after rejection, got {other:?}"),
    }
}

#[test]
fn rejects_resume_with_mismatched_confirmation_id() {
    let mut executor = MockExecutor::default();
    let pending_plan_execution = PendingPlanExecutionState {
        request_id: String::from("req-resume"),
        intent_name: IntentName::SetBrowserVisibility,
        selected_skills: vec![String::from("confirm_action")],
        confirmation_id: String::from("confirm-1"),
        prompt_text: String::from("Proceed?"),
        next_step_id: Some(String::from("step-2")),
        queued_step_ids: vec![String::from("step-2")],
        queued_steps: vec![PlannedStep {
            step_id: String::from("step-2"),
            tool_name: ToolName::SetBrowserVisibility,
            arguments: serde_json::json!({
                "request_id": "req-resume",
                "timeout_ms": 1000,
                "mode": "Headless"
            }),
            purpose: String::from("apply confirmed action"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
    };

    let outcome = resume_after_confirmation(
        &mut executor,
        &pending_plan_execution,
        "wrong-confirmation-id",
        true,
    );

    match outcome {
        ExecutionOutcome::Aborted { trace, error } => {
            assert!(trace.executed_step_ids.is_empty());
            assert_eq!(error.code, "confirmation_id_mismatch");
            assert_eq!(executor.last_visibility, None);
        }
        other => panic!("expected aborted outcome after mismatch, got {other:?}"),
    }
}

#[test]
fn aborts_when_next_step_transition_is_missing() {
    let mut executor = MockExecutor::default();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::SetPlaybackVolume,
            goal: String::from("adjust audio"),
            target_description: None,
        },
        selected_skills: vec![String::from("audio_controls")],
        steps: vec![PlannedStep {
            step_id: String::from("step-1"),
            tool_name: ToolName::SetPlaybackVolume,
            arguments: serde_json::json!({
                "request_id": "req-bad-transition",
                "timeout_ms": 1000,
                "volume": 0.4
            }),
            purpose: String::from("set the volume"),
            on_success: StepTransition::NextStep {
                step_id: String::from("missing-step"),
            },
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let outcome = execute_planner_output(
        &mut executor,
        String::from("req-bad-transition"),
        &planner_output,
    );

    match outcome {
        ExecutionOutcome::Aborted { trace, error } => {
            assert_eq!(trace.executed_step_ids, vec!["step-1"]);
            assert_eq!(error.code, "missing_transition_step");
        }
        other => panic!("expected aborted outcome, got {other:?}"),
    }
}

#[test]
fn aborts_needs_confirmation_plan_before_side_effecting_step() {
    let mut executor = MockExecutor::default();
    let planner_output = PlannerOutput {
        status: PlannerStatus::NeedsConfirmation,
        intent: IntentSummary {
            name: IntentName::SetBrowserVisibility,
            goal: String::from("toggle browser visibility"),
            target_description: None,
        },
        selected_skills: vec![String::from("confirm_action")],
        steps: vec![PlannedStep {
            step_id: String::from("step-1"),
            tool_name: ToolName::SetBrowserVisibility,
            arguments: serde_json::json!({
                "request_id": "req-needs-confirm",
                "timeout_ms": 1000,
                "mode": "Visible"
            }),
            purpose: String::from("protected action before confirmation"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: true,
        confirmation_reason: Some(String::from("protected action")),
        blocked_reason: None,
        user_message: Some(String::from("Please confirm.")),
    };

    let outcome = execute_planner_output(
        &mut executor,
        String::from("req-needs-confirm"),
        &planner_output,
    );

    match outcome {
        ExecutionOutcome::Aborted { trace, error } => {
            assert!(trace.executed_step_ids.is_empty());
            assert_eq!(error.code, "side_effect_before_confirmation");
        }
        other => panic!("expected aborted outcome, got {other:?}"),
    }
}

#[test]
fn planner_available_tools_include_all_wave_two_tools() {
    let available_tools = planner_available_tools();

    assert!(available_tools
        .iter()
        .any(|tool| tool.name == ToolName::OpenUrl));
    assert!(available_tools
        .iter()
        .any(|tool| tool.name == ToolName::TranscribeCommand));
    assert!(available_tools
        .iter()
        .any(|tool| tool.name == ToolName::FocusElement));
    assert!(available_tools
        .iter()
        .any(|tool| tool.name == ToolName::TypeIntoElement));
    assert!(available_tools
        .iter()
        .any(|tool| tool.name == ToolName::SubmitActiveForm));
    assert!(available_tools
        .iter()
        .any(|tool| tool.name == ToolName::CaptureScreenshot));
    assert!(available_tools
        .iter()
        .any(|tool| tool.name == ToolName::GetHtml));
    assert!(available_tools
        .iter()
        .any(|tool| tool.name == ToolName::RunOcr));
    assert!(available_tools
        .iter()
        .any(|tool| tool.name == ToolName::MergeOcrIntoPageModel));
}

#[test]
fn parse_skill_document_rejects_invalid_frontmatter_cases() {
    let available_tool_names = planner_available_tools()
        .into_iter()
        .map(|tool| tool.name)
        .collect::<Vec<_>>();
    let cases = [
        (
            "missing frontmatter",
            "Use the browser state to guide the user.",
            "SKILL.md is missing a YAML frontmatter block",
        ),
        (
            "unsupported field",
            r#"---
name: browse_help
description: Help the user browse
unsupported: true
---
Use the browser state to guide the user."#,
            "unsupported frontmatter field 'unsupported'",
        ),
        (
            "missing description",
            r#"---
name: browse_help
allowed_tools:
  - get_runtime_status
---
Use the browser state to guide the user."#,
            "skill frontmatter is missing description",
        ),
        (
            "unknown tool",
            r#"---
name: browse_help
description: Help the user browse
allowed_tools:
  - not_a_real_tool
---
Use the browser state to guide the user."#,
            "unknown tool 'not_a_real_tool'",
        ),
    ];

    for (label, content, expected_error) in cases {
        let error = parse_skill_document(content, SkillSource::Project, &available_tool_names)
            .expect_err("invalid skill document should be rejected");
        assert!(
            error.contains(expected_error),
            "case {label} expected error containing {expected_error:?}, got {error:?}"
        );
    }
}

#[test]
fn discover_skills_prefers_higher_precedence_duplicate_skill_names() {
    let project_root = unique_temp_path("skill-project");
    let user_root = unique_temp_path("skill-user");
    let project_skills_root = project_root.join(".pi").join("skills");
    std::fs::create_dir_all(&project_skills_root).expect("project skill root should be created");
    std::fs::create_dir_all(&user_root).expect("user skill root should be created");

    write_skill_document(
        &project_skills_root,
        "open_url",
        r#"---
name: open_url
description: Project-local open URL workflow
priority: 90
allowed_tools:
  - open_url
intent_tags:
  - intent:OpenUrl
---
Project skills should override lower-precedence copies."#,
    );
    write_skill_document(
        &user_root,
        "open_url",
        r#"---
name: open_url
description: User-level open URL workflow
priority: 10
allowed_tools:
  - open_url
intent_tags:
  - intent:OpenUrl
---
User skills should lose to project-local copies."#,
    );

    let available_tools = planner_available_tools();
    let loaded_skills = discover_skills(
        Some(project_root.as_path()),
        Some(user_root.as_path()),
        &available_tools,
    );
    let matching_skills = loaded_skills
        .iter()
        .filter(|skill| skill.summary.name == "open_url")
        .collect::<Vec<_>>();

    assert_eq!(
        matching_skills.len(),
        1,
        "duplicate skill names should resolve to one loaded skill"
    );
    let resolved = matching_skills[0];
    assert_eq!(resolved.source, SkillSource::Project);
    assert_eq!(
        resolved.summary.description,
        "Project-local open URL workflow"
    );
    assert_eq!(resolved.summary.priority, 90);
    assert_eq!(
        resolved.body,
        "Project skills should override lower-precedence copies."
    );

    std::fs::remove_dir_all(&project_root).expect("project temp directory should be removed");
    std::fs::remove_dir_all(&user_root).expect("user temp directory should be removed");
}

#[test]
fn build_planner_skill_selection_ranks_custom_skills_and_caps_to_top_n() {
    let project_root = unique_temp_path("skill-ranking-project");
    let project_skills_root = project_root.join(".pi").join("skills");
    std::fs::create_dir_all(&project_skills_root).expect("project skill root should be created");

    write_skill_document(
        &project_skills_root,
        "open_dashboard_exact",
        r#"---
name: open_dashboard_exact
description: Open the dashboard URL directly
priority: 10
allowed_tools:
  - open_url
intent_tags:
  - intent:OpenUrl
---
Open the dashboard URL directly when the user asks to open the dashboard."#,
    );
    write_skill_document(
        &project_skills_root,
        "open_dashboard_priority",
        r#"---
name: open_dashboard_priority
description: Open the dashboard URL quickly
priority: 200
allowed_tools:
  - open_url
---
Use this when dashboard navigation should stay fast."#,
    );
    write_skill_document(
        &project_skills_root,
        "dashboard_url_reference",
        r#"---
name: dashboard_url_reference
description: Explain the dashboard URL steps
priority: 50
---
Guide the user through the dashboard URL flow."#,
    );
    write_skill_document(
        &project_skills_root,
        "dashboard_helper",
        r#"---
name: dashboard_helper
description: Help with the dashboard
priority: 0
---
This helper mentions dashboard guidance only."#,
    );
    write_skill_document(
        &project_skills_root,
        "completely_unrelated",
        r#"---
name: completely_unrelated
description: Explain OCR fallback tuning
priority: 500
---
Use OCR threshold tuning when extraction fails."#,
    );

    let available_tools = planner_available_tools();
    let selection = build_planner_skill_selection(
        Some(project_root.as_path()),
        None,
        "please open the dashboard url",
        &available_tools,
    );
    let ranked_skill_names = selection
        .relevant_skill_summaries
        .iter()
        .map(|summary| summary.name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        ranked_skill_names,
        vec![
            "open_dashboard_exact",
            "open_dashboard_priority",
            "dashboard_url_reference",
        ]
    );
    assert_eq!(
        selection.relevant_skill_summaries.len(),
        MAX_SELECTED_PLANNER_SKILLS
    );
    assert!(!selection
        .relevant_skill_summaries
        .iter()
        .any(|summary| summary.name == "dashboard_helper"));
    assert!(!selection
        .relevant_skill_summaries
        .iter()
        .any(|summary| summary.name == "completely_unrelated"));

    std::fs::remove_dir_all(&project_root).expect("project temp directory should be removed");
}

#[test]
fn build_planner_skill_selection_prefers_matching_bundled_skill() {
    let available_tools = planner_available_tools();
    let selection = build_planner_skill_selection(
        None,
        None,
        "please go back to the previous page",
        &available_tools,
    );

    assert!(selection
        .active_skill_names
        .iter()
        .any(|name| name == "go_back"));
    assert_eq!(
        selection
            .relevant_skill_summaries
            .first()
            .map(|skill| skill.name.as_str()),
        Some("go_back")
    );
}

#[test]
fn build_planner_skill_selection_prefers_set_tts_voice_skill_for_voice_commands() {
    let available_tools = planner_available_tools();
    let selection =
        build_planner_skill_selection(None, None, "change the voice to Bruno", &available_tools);

    assert!(selection
        .active_skill_names
        .iter()
        .any(|name| name == "set_tts_voice"));
    assert_eq!(
        selection
            .relevant_skill_summaries
            .first()
            .map(|skill| skill.name.as_str()),
        Some("set_tts_voice")
    );
}

#[test]
fn build_planner_skill_selection_selects_expected_bundled_skills_for_representative_tasks() {
    let available_tools = planner_available_tools();
    let cases = [
        ("open github dot com slash features", "open_url"),
        ("please go back to the previous page", "go_back"),
        ("read this page", "read_page"),
        ("what page am i on", "get_current_url"),
        ("continue reading", "read_next"),
        ("are you listening", "announce_state"),
        ("start listening", "start_listening"),
        ("what's the playback speed", "get_playback_speed"),
        ("change the voice to Bruno", "set_tts_voice"),
        ("show the browser window", "toggle_browser_visibility"),
    ];

    for (transcript, expected_skill_name) in cases {
        let selection = build_planner_skill_selection(None, None, transcript, &available_tools);
        let ranked_skill_names = selection
            .relevant_skill_summaries
            .iter()
            .map(|skill| skill.name.as_str())
            .collect::<Vec<_>>();

        assert!(
            selection
                .active_skill_names
                .iter()
                .any(|name| name == expected_skill_name),
            "transcript {transcript:?} should expose bundled skill {expected_skill_name}"
        );
        assert_eq!(
            selection
                .relevant_skill_summaries
                .first()
                .map(|skill| skill.name.as_str()),
            Some(expected_skill_name),
            "transcript {transcript:?} ranked unexpected bundled skill: {ranked_skill_names:?}"
        );
    }
}

#[test]
fn bundled_skills_cover_planner_visible_command_family_intents() {
    let available_tool_names = registered_tools()
        .into_iter()
        .map(|tool| tool.name)
        .collect::<Vec<_>>();
    let bundled_skills = parse_bundled_skills(BUNDLED_SKILLS_MARKDOWN, &available_tool_names);
    let bundled_intents = bundled_skills
        .iter()
        .flat_map(|skill| skill.summary.intent_tags.iter())
        .filter_map(|tag| tag.strip_prefix("intent:"))
        .map(parse_intent_name_value)
        .collect::<Result<HashSet<_>, _>>()
        .expect("bundled intent tags should parse");

    let required_intents = [
        IntentName::OpenUrl,
        IntentName::GoBack,
        IntentName::GoForward,
        IntentName::ReloadPage,
        IntentName::GetCurrentUrl,
        IntentName::ReadPage,
        IntentName::ReadTitle,
        IntentName::ReadNext,
        IntentName::ReadPrevious,
        IntentName::Repeat,
        IntentName::Stop,
        IntentName::StartListening,
        IntentName::StopListening,
        IntentName::TranscribeCommand,
        IntentName::SetTtsVoice,
        IntentName::SetPlaybackVolume,
        IntentName::GetPlaybackVolume,
        IntentName::SetPlaybackSpeed,
        IntentName::GetPlaybackSpeed,
        IntentName::SetBrowserVisibility,
        IntentName::GetStatus,
        IntentName::FindElement,
        IntentName::ClickElement,
        IntentName::Scroll,
        IntentName::OcrRecovery,
    ];

    let missing = required_intents
        .into_iter()
        .filter(|intent| !bundled_intents.contains(intent))
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "bundled skills are missing explicit intent coverage for {missing:?}"
    );
}

#[test]
fn canonical_planner_output_examples_validate_against_current_contract() {
    let available_tools = planner_available_tools();
    let active_skill_names =
        build_planner_skill_selection(None, None, "", &available_tools).active_skill_names;

    for (example_name, planner_output) in canonical_planner_output_examples() {
        validate_planner_output(&planner_output, &available_tools, &active_skill_names)
            .unwrap_or_else(|error| {
                panic!("canonical planner example '{example_name}' should validate: {error:?}")
            });
    }
}

#[test]
fn registered_tools_all_expose_input_schemas() {
    let missing = registered_tools()
        .into_iter()
        .filter(|tool| tool_input_schema(&tool.name).is_none())
        .map(|tool| tool.name)
        .collect::<Vec<_>>();

    assert!(
        missing.is_empty(),
        "registered tools missing input schemas: {missing:?}"
    );
}

#[test]
fn sample_planned_steps_match_generated_tool_input_schemas() {
    for step in sample_planned_steps_for_registered_tools() {
        let schema = tool_input_schema(&step.tool_name).unwrap_or_else(|| {
            panic!(
                "sample tool input uses tool {:?} without an input schema",
                step.tool_name
            )
        });
        assert_json_matches_schema(&step.arguments, &schema).unwrap_or_else(|error| {
            panic!(
                "sample {:?} arguments should match generated input schema: {error}",
                step.tool_name
            )
        });
        validate_planned_step_arguments(&step).unwrap_or_else(|error| {
            panic!(
                "sample {:?} arguments should pass runtime validator: {error:?}",
                step.tool_name
            )
        });
    }
}

#[test]
fn registered_tools_all_expose_output_schemas() {
    let missing = registered_tools()
        .into_iter()
        .filter(|tool| tool_output_schema(&tool.name).is_none())
        .map(|tool| tool.name)
        .collect::<Vec<_>>();

    assert!(
        missing.is_empty(),
        "registered tools missing output schemas: {missing:?}"
    );
}

#[test]
fn registered_tools_include_output_schema_refs() {
    for tool in registered_tools() {
        assert_eq!(
            tool.output_schema_ref,
            format!("schema://tool-output/{:?}", tool.name)
        );
    }
}

#[test]
fn sample_serialized_tool_results_match_generated_tool_output_schemas() {
    let mut executor = MockExecutor::default();

    for step in sample_planned_steps_for_registered_tools() {
        let result = execute_planned_step(&mut executor, &step);
        let serialized =
            serde_json::to_value(&result).expect("serialized tool result should serialize");
        let schema = tool_output_schema(&step.tool_name).unwrap_or_else(|| {
            panic!(
                "sample tool result uses tool {:?} without an output schema",
                step.tool_name
            )
        });
        assert_json_matches_schema(&serialized, &schema).unwrap_or_else(|error| {
            panic!(
                "sample serialized {:?} result should match generated output schema: {error}",
                step.tool_name
            )
        });
    }
}

#[test]
fn tool_result_success_populates_common_envelope_fields() {
    let result = ToolResult::success(
        ToolName::GetRuntimeStatus,
        String::from("req-envelope-success"),
        GetRuntimeStatusData {
            page_id: Some(String::from("page-1")),
            url: Some(String::from("https://example.com")),
            title: Some(String::from("Example")),
            browser_visibility: BrowserVisibilityMode::Visible,
            browser_history: BrowserHistoryState::default(),
            listening_state: ListeningState::default(),
            speaking: false,
            audio: RuntimeAudioState::default(),
            pending_confirmation_id: None,
            pending_plan_execution: None,
            provider_modes: None,
        },
        vec![String::from("runtime status read")],
    );

    assert!(result.ok);
    assert_eq!(result.tool_name, ToolName::GetRuntimeStatus);
    assert_eq!(result.request_id, "req-envelope-success");
    assert!(result.timestamp_ms > 0);
    assert!(result.error.is_none());
    assert!(result.warnings.is_empty());
    assert_eq!(
        result.observations,
        vec![String::from("runtime status read")]
    );
    assert_eq!(
        result.data.as_ref().and_then(|data| data.url.as_deref()),
        Some("https://example.com")
    );
}

#[test]
fn tool_result_failure_populates_common_envelope_fields() {
    let result: ToolResult<SetPlaybackVolumeData> = ToolResult::failure(
        ToolName::SetPlaybackVolume,
        String::from("req-envelope-failure"),
        ToolError {
            code: String::from("audio_update_failed"),
            message: String::from("failed to persist volume"),
            retryable: false,
            details: Some(serde_json::json!({
                "setting": "playback_volume"
            })),
        },
        vec![String::from("volume update failed")],
    );

    assert!(!result.ok);
    assert_eq!(result.tool_name, ToolName::SetPlaybackVolume);
    assert_eq!(result.request_id, "req-envelope-failure");
    assert!(result.timestamp_ms > 0);
    assert!(result.data.is_none());
    assert!(result.warnings.is_empty());
    assert_eq!(
        result.observations,
        vec![String::from("volume update failed")]
    );
    assert_eq!(
        result.error,
        Some(ToolError {
            code: String::from("audio_update_failed"),
            message: String::from("failed to persist volume"),
            retryable: false,
            details: Some(serde_json::json!({
                "setting": "playback_volume"
            })),
        })
    );
}

#[test]
fn serialized_tool_result_round_trips_with_warning_and_error_details() {
    let envelope = SerializedToolResult {
        ok: false,
        tool_name: ToolName::RunOcr,
        request_id: String::from("req-envelope-roundtrip"),
        timestamp_ms: 1_234_567,
        data: None,
        error: Some(ToolError {
            code: String::from("ocr_failed"),
            message: String::from("OCR provider was unavailable"),
            retryable: true,
            details: Some(serde_json::json!({
                "image_id": "image-1"
            })),
        }),
        warnings: vec![ToolWarning {
            code: String::from("low_contrast"),
            message: String::from("Image contrast was low."),
        }],
        observations: vec![String::from("OCR could not complete.")],
    };

    let serialized =
        serde_json::to_value(&envelope).expect("serialized tool result should serialize");
    assert_eq!(
        serialized,
        serde_json::json!({
            "ok": false,
            "tool_name": "RunOcr",
            "request_id": "req-envelope-roundtrip",
            "timestamp_ms": 1234567,
            "data": null,
            "error": {
                "code": "ocr_failed",
                "message": "OCR provider was unavailable",
                "retryable": true,
                "details": {
                    "image_id": "image-1"
                }
            },
            "warnings": [
                {
                    "code": "low_contrast",
                    "message": "Image contrast was low."
                }
            ],
            "observations": ["OCR could not complete."]
        })
    );

    let round_tripped: SerializedToolResult =
        serde_json::from_value(serialized).expect("serialized tool result should deserialize");
    assert_eq!(round_tripped, envelope);
}

#[test]
fn typed_tool_result_deserializes_common_envelope_and_payload() {
    let result: ToolResult<SetPlaybackSpeedData> = serde_json::from_value(serde_json::json!({
        "ok": true,
        "tool_name": "SetPlaybackSpeed",
        "request_id": "req-envelope-typed",
        "timestamp_ms": 987654,
        "data": {
            "playback_speed": 1.25,
            "changed": true
        },
        "error": null,
        "warnings": [
            {
                "code": "rounded_value",
                "message": "The requested speed was rounded."
            }
        ],
        "observations": ["Updated the playback speed setting."]
    }))
    .expect("typed tool result should deserialize");

    assert!(result.ok);
    assert_eq!(result.tool_name, ToolName::SetPlaybackSpeed);
    assert_eq!(result.request_id, "req-envelope-typed");
    assert_eq!(result.timestamp_ms, 987_654);
    assert_eq!(
        result.data,
        Some(SetPlaybackSpeedData {
            playback_speed: 1.25,
            changed: true,
        })
    );
    assert!(result.error.is_none());
    assert_eq!(
        result.warnings,
        vec![ToolWarning {
            code: String::from("rounded_value"),
            message: String::from("The requested speed was rounded."),
        }]
    );
    assert_eq!(
        result.observations,
        vec![String::from("Updated the playback speed setting.")]
    );
}

#[test]
fn shared_contract_enums_serialize_expected_variants() {
    assert_eq!(serde_json::json!(NarrationInterruptionMode::Queue), "Queue");
    assert_eq!(
        serde_json::json!(NarrationInterruptionMode::Interrupt),
        "Interrupt"
    );
    assert_eq!(serde_json::json!(NarrationBoundary::None), "None");
    assert_eq!(serde_json::json!(NarrationBoundary::Start), "Start");
    assert_eq!(serde_json::json!(NarrationBoundary::End), "End");
    assert_eq!(serde_json::json!(ElementVisibilityFilter::All), "All");
    assert_eq!(
        serde_json::json!(ElementVisibilityFilter::VisibleOnly),
        "VisibleOnly"
    );
    assert_eq!(serde_json::json!(ReloadMode::Standard), "Standard");
    assert_eq!(serde_json::json!(ReloadMode::Hard), "Hard");
    assert_eq!(serde_json::json!(ClickMode::Single), "Single");
    assert_eq!(serde_json::json!(ClickMode::Double), "Double");
    assert_eq!(serde_json::json!(TextEntryMode::Append), "Append");
    assert_eq!(serde_json::json!(TextEntryMode::Replace), "Replace");
    assert_eq!(
        serde_json::json!(TextEntrySubmitMode::KeepEditing),
        "KeepEditing"
    );
    assert_eq!(serde_json::json!(TextEntrySubmitMode::Submit), "Submit");
    assert_eq!(
        serde_json::json!(TranscriptionStopMode::KeepListening),
        "KeepListening"
    );
    assert_eq!(
        serde_json::json!(TranscriptionStopMode::AutoStop),
        "AutoStop"
    );
    assert_eq!(serde_json::json!(ScreenshotScope::Viewport), "Viewport");
    assert_eq!(serde_json::json!(ScreenshotScope::FullPage), "FullPage");
    assert_eq!(serde_json::json!(RemoteProviderLabel::OpenAi), "OpenAI");
    assert_eq!(serde_json::json!(RemoteProviderLabel::Ollama), "Ollama");
    assert_eq!(
        serde_json::json!(LocalTtsBackend::KittenTtsRs),
        "kitten_tts_rs"
    );
    assert_eq!(serde_json::json!(LocalAsrBackend::Whisper), "whisper");
    assert_eq!(serde_json::json!(RemoteTtsAudioFormat::Wav), "wav");
    assert_eq!(
        serde_json::json!(crate::page_model::ElementRole::Landmark),
        "Landmark"
    );
    assert_eq!(
        serde_json::json!(crate::page_model::ElementRole::Other),
        "Other"
    );
    assert_eq!(
        serde_json::json!(crate::page_model::RegionSource::Mixed),
        "Mixed"
    );
    assert_eq!(
        serde_json::json!(crate::page_model::ExtractionSource::Merged),
        "Merged"
    );
    assert_eq!(
        serde_json::json!(ReportStatus::NeedsFollowUp),
        "NeedsFollowUp"
    );
}

#[test]
fn canonical_planner_output_examples_serialize_expected_strings_and_fields() {
    let examples = canonical_planner_output_examples();

    let set_volume = serde_json::to_value(
        examples
            .get("set_playback_volume")
            .expect("set_playback_volume example should exist"),
    )
    .expect("planner example should serialize");
    assert_eq!(
        set_volume.pointer("/intent/name"),
        Some(&serde_json::json!("SetPlaybackVolume"))
    );
    assert_eq!(
        set_volume.pointer("/steps/0/arguments/request_id"),
        Some(&serde_json::json!("example-set-volume"))
    );
    assert_eq!(
        set_volume.pointer("/steps/0/arguments/volume"),
        Some(&serde_json::json!(0.7))
    );
    assert_eq!(
        set_volume.pointer("/steps/0/on_success"),
        Some(&serde_json::json!({
            "NextStep": {
                "step_id": "report-playback-volume"
            }
        }))
    );
    assert_eq!(
        set_volume.pointer("/steps/1/on_success"),
        Some(&serde_json::json!("Complete"))
    );

    let ready_click = serde_json::to_value(
        examples
            .get("click_element_ready")
            .expect("click_element_ready example should exist"),
    )
    .expect("planner example should serialize");
    assert_eq!(
        ready_click.pointer("/status"),
        Some(&serde_json::json!("Ready"))
    );
    assert_eq!(
        ready_click.pointer("/steps/0/tool_name"),
        Some(&serde_json::json!("ClickElement"))
    );
    assert_eq!(
        ready_click.pointer("/steps/0/arguments/element_id"),
        Some(&serde_json::json!("link-help"))
    );

    let needs_confirmation = serde_json::to_value(
        examples
            .get("click_element_with_confirmation")
            .expect("click_element_with_confirmation example should exist"),
    )
    .expect("planner example should serialize");
    assert_eq!(
        needs_confirmation.pointer("/status"),
        Some(&serde_json::json!("NeedsConfirmation"))
    );
    assert_eq!(
        needs_confirmation.pointer("/steps/0/on_success"),
        Some(&serde_json::json!("RequestConfirmation"))
    );
    assert_eq!(
        needs_confirmation.pointer("/steps/1/arguments/element_id"),
        Some(&serde_json::json!("button-submit"))
    );
}

#[test]
fn canonical_planner_output_examples_match_generated_planner_output_schema() {
    let schema = planner_output_schema();

    for (example_name, planner_output) in canonical_planner_output_examples() {
        let serialized = serde_json::to_value(&planner_output)
            .expect("planner example should serialize to JSON");
        assert_json_matches_schema(&serialized, &schema).unwrap_or_else(|error| {
            panic!(
                "canonical planner example '{example_name}' should match generated planner schema: {error}"
            )
        });
    }
}

#[test]
fn planner_output_round_trips_with_confirmation_metadata_and_matches_schema() {
    let planner_output = canonical_planner_output_examples()
        .remove("click_element_with_confirmation")
        .expect("click_element_with_confirmation example should exist");

    let serialized =
        serde_json::to_value(&planner_output).expect("planner output should serialize");
    assert_json_matches_schema(&serialized, &planner_output_schema())
        .expect("planner output should match generated planner schema");
    assert_eq!(
        serialized.pointer("/status"),
        Some(&serde_json::json!("NeedsConfirmation"))
    );
    assert_eq!(
        serialized.pointer("/requires_confirmation"),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        serialized.pointer("/steps/0/on_success"),
        Some(&serde_json::json!("RequestConfirmation"))
    );

    let round_tripped: PlannerOutput =
        serde_json::from_value(serialized).expect("planner output should deserialize");
    assert_eq!(round_tripped, planner_output);
}

#[test]
fn canonical_planner_output_step_arguments_match_generated_tool_input_schemas() {
    for (example_name, planner_output) in canonical_planner_output_examples() {
        for step in &planner_output.steps {
            let schema = tool_input_schema(&step.tool_name).unwrap_or_else(|| {
                panic!(
                    "canonical planner example '{example_name}' uses tool {:?} without an input schema",
                    step.tool_name
                )
            });
            assert_json_matches_schema(&step.arguments, &schema).unwrap_or_else(|error| {
                panic!(
                    "canonical planner example '{example_name}' step '{}' arguments should match generated {:?} schema: {error}",
                    step.step_id,
                    step.tool_name
                )
            });
        }
    }
}

#[test]
fn planner_input_round_trips_with_nested_runtime_context_and_matches_schema() {
    let page_model = PageModel {
        interactive_elements: vec![InteractiveElement {
            element_id: String::from("link-help"),
            dom_locator: Some(String::from("#help")),
            role: crate::page_model::ElementRole::Link,
            tag_name: String::from("a"),
            text: Some(String::from("Help")),
            accessible_name: Some(String::from("Help")),
            placeholder: None,
            href: Some(String::from("https://example.com/help")),
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: std::collections::BTreeMap::new(),
        }],
        ..fixture_page_model_without_regions()
    };

    let planner_input = PlannerInput {
        request_id: String::from("req-planner-roundtrip"),
        transcript: String::from("click the help link"),
        agent_state: fixture_agent_state(),
        safety: PlannerSafetySettings {
            confirmation_confidence_threshold: 0.9,
            allow_click_without_confirmation: true,
            always_confirm_submit: true,
        },
        available_tools: planner_available_tools(),
        active_skill_names: vec![String::from("open_link_by_text")],
        relevant_skill_summaries: vec![SkillSummary {
            name: String::from("open_link_by_text"),
            description: String::from("Open a link by matching visible text."),
            intent_tags: vec![String::from("intent:ClickElement")],
            allowed_tools: Some(vec![ToolName::FindElement, ToolName::ClickElement]),
            requires_confirmation: false,
            priority: 80,
        }],
        page_snapshot: Some(PageSnapshotData {
            page_id: String::from("page-1"),
            url: String::from("https://example.com/article"),
            title: Some(String::from("Example article")),
            visible_text_excerpt: String::from("Example article body"),
            interactive_elements: vec![InteractiveElement {
                element_id: String::from("link-help"),
                dom_locator: Some(String::from("#help")),
                role: crate::page_model::ElementRole::Link,
                tag_name: String::from("a"),
                text: Some(String::from("Help")),
                accessible_name: Some(String::from("Help")),
                placeholder: None,
                href: Some(String::from("https://example.com/help")),
                value: None,
                bbox: None,
                visible: true,
                enabled: true,
                attributes: std::collections::BTreeMap::new(),
            }],
            scroll_y: 120.0,
            viewport_width: 1280.0,
            viewport_height: 720.0,
            document_height: 2400.0,
        }),
        page_model: Some(page_model),
        recent_tool_results: vec![PlannerToolHistoryEntry {
            tool_name: ToolName::GetAgentState,
            ok: true,
            observation_summary: vec![String::from("agent state read")],
        }],
    };

    let serialized = serde_json::to_value(&planner_input).expect("planner input should serialize");
    let schema = schema_json::<PlannerInput>();
    assert_json_matches_schema(&serialized, &schema)
        .expect("planner input should match generated planner input schema");
    assert_eq!(
        serialized.pointer("/agent_state/browser_history/current_entry_index"),
        Some(&serde_json::json!(1))
    );
    assert_eq!(
        serialized.pointer("/relevant_skill_summaries/0/allowed_tools/1"),
        Some(&serde_json::json!("ClickElement"))
    );

    let round_tripped: PlannerInput =
        serde_json::from_value(serialized).expect("planner input should deserialize");
    assert_eq!(round_tripped, planner_input);
}

#[test]
fn planner_input_serializes_safety_settings_for_click_policy() {
    let planner_input = PlannerInput {
        request_id: String::from("req-planner"),
        transcript: String::from("click the help link"),
        agent_state: AgentStateData {
            page_id: Some(String::from("page-1")),
            url: Some(String::from("https://example.com")),
            title: Some(String::from("Example")),
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
        },
        safety: PlannerSafetySettings {
            confirmation_confidence_threshold: 0.9,
            allow_click_without_confirmation: true,
            always_confirm_submit: true,
        },
        available_tools: Vec::new(),
        active_skill_names: vec![String::from("open_link_by_text")],
        relevant_skill_summaries: Vec::new(),
        page_snapshot: None,
        page_model: None,
        recent_tool_results: Vec::new(),
    };

    let serialized = serde_json::to_value(&planner_input).expect("planner input should serialize");
    assert_eq!(
        serialized.pointer("/safety/allow_click_without_confirmation"),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        serialized.pointer("/safety/always_confirm_submit"),
        Some(&serde_json::json!(true))
    );
}

#[test]
fn validate_planner_output_rejects_unknown_selected_skill() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::GetStatus,
            goal: String::from("report the current status"),
            target_description: None,
        },
        selected_skills: vec![String::from("not-a-real-skill")],
        steps: vec![PlannedStep {
            step_id: String::from("step-status"),
            tool_name: ToolName::GetRuntimeStatus,
            arguments: serde_json::json!({
                "request_id": "req-status",
                "include_provider_modes": true
            }),
            purpose: String::from("read runtime status"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("get_status")],
    )
    .expect_err("validation should reject unknown selected skills");
    assert_eq!(error.code, "invalid_planner_output");
    assert!(error.message.contains("unknown or ineligible skill"));
}

#[test]
fn validate_planner_output_rejects_unavailable_tool_reference() {
    let mut available_tools = planner_available_tools();
    available_tools.retain(|tool| tool.name != ToolName::SetPlaybackVolume);
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::SetPlaybackVolume,
            goal: String::from("adjust audio"),
            target_description: None,
        },
        selected_skills: vec![String::from("audio_controls")],
        steps: vec![PlannedStep {
            step_id: String::from("step-volume"),
            tool_name: ToolName::SetPlaybackVolume,
            arguments: serde_json::json!({
                "request_id": "req-volume",
                "volume": 0.4
            }),
            purpose: String::from("set the volume"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("audio_controls")],
    )
    .expect_err("validation should reject unavailable tool references");
    assert_eq!(error.code, "invalid_planner_output");
    assert!(error
        .message
        .contains("planner referenced unavailable tool SetPlaybackVolume"));
    assert_eq!(
        error
            .details
            .as_ref()
            .and_then(|details| details.get("step_id")),
        Some(&serde_json::json!("step-volume"))
    );
}

#[test]
fn validate_planner_output_rejects_missing_next_step_transition() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::SetPlaybackVolume,
            goal: String::from("adjust audio"),
            target_description: None,
        },
        selected_skills: vec![String::from("audio_controls")],
        steps: vec![PlannedStep {
            step_id: String::from("step-volume"),
            tool_name: ToolName::SetPlaybackVolume,
            arguments: serde_json::json!({
                "request_id": "req-volume",
                "volume": 0.4
            }),
            purpose: String::from("set the volume"),
            on_success: StepTransition::NextStep {
                step_id: String::from("missing-step"),
            },
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("audio_controls")],
    )
    .expect_err("validation should reject missing next-step transitions");
    assert_eq!(error.code, "invalid_planner_output");
    assert!(error
        .message
        .contains("planner referenced missing next step 'missing-step' from 'step-volume'"));
    assert_eq!(
        error
            .details
            .as_ref()
            .and_then(|details| details.get("source_step_id")),
        Some(&serde_json::json!("step-volume"))
    );
    assert_eq!(
        error
            .details
            .as_ref()
            .and_then(|details| details.get("next_step_id")),
        Some(&serde_json::json!("missing-step"))
    );
}

#[test]
fn validate_planner_output_rejects_submit_form_without_needs_confirmation() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::SubmitForm,
            goal: String::from("submit the active form"),
            target_description: Some(String::from("login form")),
        },
        selected_skills: vec![String::from("confirm_action")],
        steps: vec![PlannedStep {
            step_id: String::from("confirm-submit"),
            tool_name: ToolName::ConfirmAction,
            arguments: serde_json::json!({
                "request_id": "req-submit",
                "timeout_ms": 1000,
                "prompt_text": "Submit the form now?",
                "reason": "Submitting the form may send data."
            }),
            purpose: String::from("ask for confirmation"),
            on_success: StepTransition::RequestConfirmation,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: true,
        confirmation_reason: Some(String::from("submitting the form may send data")),
        blocked_reason: None,
        user_message: Some(String::from("Please confirm before I submit the form.")),
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("confirm_action")],
    )
    .expect_err("submit-form plans should require NeedsConfirmation status");
    assert_eq!(error.code, "invalid_planner_output");
    assert!(error
        .message
        .contains("submit-form planner output must use NeedsConfirmation"));
}

#[test]
fn validate_planner_output_rejects_submit_form_without_confirm_action_step() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::NeedsConfirmation,
        intent: IntentSummary {
            name: IntentName::SubmitForm,
            goal: String::from("submit the active form"),
            target_description: Some(String::from("login form")),
        },
        selected_skills: vec![String::from("confirm_action")],
        steps: vec![PlannedStep {
            step_id: String::from("report-submit"),
            tool_name: ToolName::ReportResult,
            arguments: serde_json::json!({
                "request_id": "req-submit",
                "timeout_ms": 1000,
                "status": "NeedsFollowUp",
                "summary": "The form is ready to submit.",
                "next_recommended_action": "Confirm the submission.",
                "user_message": "The form is ready to submit."
            }),
            purpose: String::from("report submit readiness"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: true,
        confirmation_reason: Some(String::from("submitting the form may send data")),
        blocked_reason: None,
        user_message: Some(String::from("Please confirm before I submit the form.")),
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("confirm_action")],
    )
    .expect_err("submit-form plans should require a confirm_action step");
    assert_eq!(error.code, "invalid_planner_output");
    assert!(error.message.contains("must include a confirm_action step"));
}

#[test]
fn validate_planner_output_rejects_needs_confirmation_without_confirm_action_step() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::NeedsConfirmation,
        intent: IntentSummary {
            name: IntentName::ClickElement,
            goal: String::from("activate the selected button"),
            target_description: Some(String::from("submit button")),
        },
        selected_skills: vec![String::from("open_link_by_text")],
        steps: vec![PlannedStep {
            step_id: String::from("click-button"),
            tool_name: ToolName::ClickElement,
            arguments: serde_json::json!({
                "request_id": "req-click",
                "timeout_ms": 1000,
                "element_id": "button-submit",
                "click_mode": "Single"
            }),
            purpose: String::from("activate the chosen button"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: true,
        confirmation_reason: Some(String::from("clicking may trigger a protected action")),
        blocked_reason: None,
        user_message: Some(String::from("Please confirm before I activate the button.")),
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("open_link_by_text")],
    )
    .expect_err("needs-confirmation plans should require a confirm_action step");
    assert_eq!(error.code, "invalid_planner_output");
    assert!(error.message.contains("must include a confirm_action step"));
}

#[test]
fn validate_planner_output_rejects_ready_output_with_confirmation_metadata() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::ClickElement,
            goal: String::from("activate the selected button"),
            target_description: Some(String::from("submit button")),
        },
        selected_skills: vec![String::from("confirm_action")],
        steps: vec![PlannedStep {
            step_id: String::from("confirm-click"),
            tool_name: ToolName::ConfirmAction,
            arguments: serde_json::json!({
                "request_id": "req-click",
                "timeout_ms": 1000,
                "prompt_text": "Do you want me to activate the submit button?",
                "reason": "Activating it may send data."
            }),
            purpose: String::from("ask for confirmation"),
            on_success: StepTransition::RequestConfirmation,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: true,
        confirmation_reason: Some(String::from("activating the button may send data")),
        blocked_reason: None,
        user_message: Some(String::from("Please confirm before I activate the button.")),
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("confirm_action")],
    )
    .expect_err("ready plans should not carry confirmation-only metadata");
    assert_eq!(error.code, "invalid_planner_output");
    assert!(error.message.contains("must not set requires_confirmation"));
}

#[test]
fn validate_planner_output_accepts_submit_form_with_confirmation_gate() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::NeedsConfirmation,
        intent: IntentSummary {
            name: IntentName::SubmitForm,
            goal: String::from("submit the active form"),
            target_description: Some(String::from("login form")),
        },
        selected_skills: vec![String::from("confirm_action")],
        steps: vec![
            PlannedStep {
                step_id: String::from("confirm-submit"),
                tool_name: ToolName::ConfirmAction,
                arguments: serde_json::json!({
                    "request_id": "req-submit",
                    "timeout_ms": 1000,
                    "prompt_text": "The form is filled. Do you want me to submit it now?",
                    "reason": "Submitting the form may send data."
                }),
                purpose: String::from("require explicit confirmation before submission"),
                on_success: StepTransition::RequestConfirmation,
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from("report-submit-ready"),
                tool_name: ToolName::ReportResult,
                arguments: serde_json::json!({
                    "request_id": "req-submit",
                    "timeout_ms": 1000,
                    "status": "NeedsFollowUp",
                    "summary": "The form is ready to submit after you confirm.",
                    "next_recommended_action": "Confirm the submission.",
                    "user_message": "Please confirm before I submit the form."
                }),
                purpose: String::from("keep the user informed while awaiting confirmation"),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ],
        requires_confirmation: true,
        confirmation_reason: Some(String::from("submitting the form may send data")),
        blocked_reason: None,
        user_message: Some(String::from("Please confirm before I submit the form.")),
    };

    validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("confirm_action")],
    )
    .expect("submit-form plans should validate when confirmation is required");
}

#[test]
fn validate_planner_output_accepts_click_element_with_confirmation_gate() {
    let available_tools = planner_available_tools();
    let mut examples = canonical_planner_output_examples();
    let planner_output = examples
        .remove("click_element_with_confirmation")
        .expect("click confirmation example should exist");

    validate_planner_output(
        &planner_output,
        &available_tools,
        &[
            String::from("open_link_by_text"),
            String::from("confirm_action"),
        ],
    )
    .expect("click plans should validate when they use the bounded confirmation flow");
}

#[test]
fn validate_planner_output_rejects_invalid_step_arguments() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::SetPlaybackVolume,
            goal: String::from("adjust playback volume"),
            target_description: None,
        },
        selected_skills: vec![String::from("set_volume")],
        steps: vec![PlannedStep {
            step_id: String::from("step-volume"),
            tool_name: ToolName::SetPlaybackVolume,
            arguments: serde_json::json!({
                "request_id": "req-volume",
                "volume": "loud"
            }),
            purpose: String::from("set playback volume"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("set_volume")],
    )
    .expect_err("validation should reject malformed step arguments");
    assert_eq!(error.code, "invalid_planner_output");
    assert!(error.message.contains("expected schema"));
    assert_eq!(
        error
            .details
            .as_ref()
            .and_then(|details| details.get("step_id")),
        Some(&serde_json::json!("step-volume"))
    );
    assert_eq!(
        error
            .details
            .as_ref()
            .and_then(|details| details.get("tool_name")),
        Some(&serde_json::json!("SetPlaybackVolume"))
    );
}

#[test]
fn validate_planner_output_rejects_capture_screenshot_with_multiple_targets() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::OcrRecovery,
            goal: String::from("capture a screenshot for OCR"),
            target_description: None,
        },
        selected_skills: vec![String::from("ocr_current_region")],
        steps: vec![PlannedStep {
            step_id: String::from("step-capture"),
            tool_name: ToolName::CaptureScreenshot,
            arguments: serde_json::json!({
                "request_id": "req-capture",
                "scope": "FullPage",
                "region_id": "region-1",
                "bbox": serde_json::Value::Null
            }),
            purpose: String::from("capture an image for OCR"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("ocr_current_region")],
    )
    .expect_err("validation should reject conflicting screenshot target modes");
    assert_eq!(error.code, "invalid_planner_output");
    assert!(error
        .message
        .contains("capture_screenshot supports at most one targeting mode"));
}

#[test]
fn validate_planner_output_rejects_run_ocr_without_any_source() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::OcrRecovery,
            goal: String::from("read text from an image"),
            target_description: None,
        },
        selected_skills: vec![String::from("ocr_current_region")],
        steps: vec![PlannedStep {
            step_id: String::from("step-run-ocr"),
            tool_name: ToolName::RunOcr,
            arguments: serde_json::json!({
                "request_id": "req-run-ocr",
                "image_id": serde_json::Value::Null,
                "region_id": serde_json::Value::Null,
                "bbox": serde_json::Value::Null
            }),
            purpose: String::from("run OCR"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("ocr_current_region")],
    )
    .expect_err("validation should reject run_ocr without any source image or target");
    assert_eq!(error.code, "invalid_planner_output");
    assert!(error
        .message
        .contains("run_ocr requires at least one source"));
}

#[test]
fn validate_planner_output_rejects_merge_ocr_with_empty_text() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::OcrRecovery,
            goal: String::from("merge OCR text"),
            target_description: None,
        },
        selected_skills: vec![String::from("read_visible_text")],
        steps: vec![PlannedStep {
            step_id: String::from("step-merge-ocr"),
            tool_name: ToolName::MergeOcrIntoPageModel,
            arguments: serde_json::json!({
                "request_id": "req-merge-ocr",
                "page_id": "page-1",
                "region_id": serde_json::Value::Null,
                "ocr_text": "   ",
                "source_bbox": serde_json::Value::Null
            }),
            purpose: String::from("merge OCR text"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("read_visible_text")],
    )
    .expect_err("validation should reject merge_ocr_into_page_model without OCR text");
    assert_eq!(error.code, "invalid_planner_output");
    assert!(error
        .message
        .contains("merge_ocr_into_page_model requires non-empty ocr_text"));
}

#[test]
fn set_tts_voice_input_accepts_known_voice_names_only() {
    let local_voice: SetTtsVoiceInput = serde_json::from_value(serde_json::json!({
        "request_id": "req-local-voice",
        "voice": "Bruno"
    }))
    .expect("known local voice should deserialize");
    assert_eq!(local_voice.voice, TtsVoiceName::Bruno);

    let remote_voice: SetTtsVoiceInput = serde_json::from_value(serde_json::json!({
        "request_id": "req-remote-voice",
        "voice": "alloy"
    }))
    .expect("known remote voice should deserialize");
    assert_eq!(remote_voice.voice, TtsVoiceName::Alloy);

    let invalid_voice = serde_json::from_value::<SetTtsVoiceInput>(serde_json::json!({
        "request_id": "req-invalid-voice",
        "voice": "not-a-real-voice"
    }));
    assert!(invalid_voice.is_err());
}

#[test]
fn validate_planner_output_rejects_open_url_with_blank_url() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::OpenUrl,
            goal: String::from("open a page"),
            target_description: None,
        },
        selected_skills: vec![String::from("open_url_direct")],
        steps: vec![PlannedStep {
            step_id: String::from("step-open-url"),
            tool_name: ToolName::OpenUrl,
            arguments: serde_json::json!({
                "request_id": "req-open-url",
                "url": "   ",
                "wait_for_load_state": "NetworkIdle"
            }),
            purpose: String::from("open a page"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("open_url_direct")],
    )
    .expect_err("validation should reject blank open_url values");
    assert!(error.message.contains("open_url requires a non-empty url"));
}

#[test]
fn validate_eval_js_input_rejects_blank_expression() {
    let step = PlannedStep {
        step_id: String::from("step-eval-js"),
        tool_name: ToolName::EvalJs,
        arguments: serde_json::json!({
            "request_id": "req-eval-js",
            "expression": "   "
        }),
        purpose: String::from("evaluate a bounded JavaScript expression"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    };

    let error = validate_planned_step_arguments(&step)
        .expect_err("validation should reject blank eval_js expressions");
    assert!(error
        .message
        .contains("eval_js requires a non-empty expression"));
}

#[test]
fn validate_planner_output_rejects_open_url_with_relative_url() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::OpenUrl,
            goal: String::from("open a page"),
            target_description: None,
        },
        selected_skills: vec![String::from("open_url_direct")],
        steps: vec![PlannedStep {
            step_id: String::from("step-open-url"),
            tool_name: ToolName::OpenUrl,
            arguments: serde_json::json!({
                "request_id": "req-open-url",
                "url": "/relative/path",
                "wait_for_load_state": "Load"
            }),
            purpose: String::from("open a page"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("open_url_direct")],
    )
    .expect_err("validation should reject relative open_url values");
    assert!(error
        .message
        .contains("open_url requires an absolute URL with a scheme"));
}

#[test]
fn validate_planner_output_rejects_go_back_with_too_many_steps() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::GoBack,
            goal: String::from("go back"),
            target_description: None,
        },
        selected_skills: vec![String::from("go_back")],
        steps: vec![PlannedStep {
            step_id: String::from("step-go-back"),
            tool_name: ToolName::GoBack,
            arguments: serde_json::json!({
                "request_id": "req-go-back",
                "steps": MAX_HISTORY_STEPS + 1,
                "wait_for_load_state": "Load"
            }),
            purpose: String::from("go back"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("go_back")],
    )
    .expect_err("validation should reject go_back steps above the supported maximum");
    assert!(error
        .message
        .contains("go_back steps must be less than or equal to"));
}

#[test]
fn validate_planner_output_rejects_go_forward_with_zero_steps() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::GoForward,
            goal: String::from("go forward"),
            target_description: None,
        },
        selected_skills: vec![String::from("go_forward")],
        steps: vec![PlannedStep {
            step_id: String::from("step-go-forward"),
            tool_name: ToolName::GoForward,
            arguments: serde_json::json!({
                "request_id": "req-go-forward",
                "steps": 0,
                "wait_for_load_state": "Load"
            }),
            purpose: String::from("go forward"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("go_forward")],
    )
    .expect_err("validation should reject go_forward steps below the supported minimum");
    assert!(error
        .message
        .contains("go_forward steps must be greater than 0"));
}

#[test]
fn validate_planner_output_rejects_scroll_page_without_amount_or_target() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::ReadNext,
            goal: String::from("scroll the page"),
            target_description: None,
        },
        selected_skills: vec![String::from("scroll_page")],
        steps: vec![PlannedStep {
            step_id: String::from("step-scroll"),
            tool_name: ToolName::ScrollPage,
            arguments: serde_json::json!({
                "request_id": "req-scroll",
                "direction": "Down",
                "amount_px": serde_json::Value::Null,
                "target": serde_json::Value::Null
            }),
            purpose: String::from("scroll the page"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("scroll_page")],
    )
    .expect_err("validation should reject scroll_page requests without amount or target");
    assert!(error
        .message
        .contains("scroll_page requires amount_px or target to be provided"));
}

#[test]
fn validate_planner_output_rejects_scroll_page_with_non_positive_amount() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::ReadNext,
            goal: String::from("scroll the page"),
            target_description: None,
        },
        selected_skills: vec![String::from("scroll_page")],
        steps: vec![PlannedStep {
            step_id: String::from("step-scroll"),
            tool_name: ToolName::ScrollPage,
            arguments: serde_json::json!({
                "request_id": "req-scroll",
                "direction": "Down",
                "amount_px": 0.0,
                "target": serde_json::Value::Null
            }),
            purpose: String::from("scroll the page"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("scroll_page")],
    )
    .expect_err("validation should reject non-positive scroll amounts");
    assert!(error
        .message
        .contains("scroll_page amount_px must be greater than 0"));
}

#[test]
fn validate_planner_output_rejects_find_element_with_blank_description() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::FindElement,
            goal: String::from("find an element"),
            target_description: None,
        },
        selected_skills: vec![String::from("find_element")],
        steps: vec![PlannedStep {
            step_id: String::from("step-find-element"),
            tool_name: ToolName::FindElement,
            arguments: serde_json::json!({
                "request_id": "req-find-element",
                "description": "   ",
                "text": null,
                "role": null,
                "color_hint": null,
                "nearby_text": null,
                "selector_hint": null,
                "visibility_filter": "VisibleOnly",
                "max_candidates": 3
            }),
            purpose: String::from("find an element"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("find_element")],
    )
    .expect_err("validation should reject blank find_element descriptions");
    assert!(error
        .message
        .contains("find_element requires a non-empty description"));
}

#[test]
fn validate_planner_output_rejects_find_element_with_zero_max_candidates() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::FindElement,
            goal: String::from("find an element"),
            target_description: None,
        },
        selected_skills: vec![String::from("find_element")],
        steps: vec![PlannedStep {
            step_id: String::from("step-find-element"),
            tool_name: ToolName::FindElement,
            arguments: serde_json::json!({
                "request_id": "req-find-element",
                "description": "search field",
                "text": null,
                "role": null,
                "color_hint": null,
                "nearby_text": null,
                "selector_hint": null,
                "visibility_filter": "VisibleOnly",
                "max_candidates": 0
            }),
            purpose: String::from("find an element"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("find_element")],
    )
    .expect_err("validation should reject zero max_candidates");
    assert!(error
        .message
        .contains("find_element max_candidates must be greater than 0"));
}

#[test]
fn validate_planner_output_rejects_find_element_with_too_many_max_candidates() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::FindElement,
            goal: String::from("find an element"),
            target_description: None,
        },
        selected_skills: vec![String::from("find_element")],
        steps: vec![PlannedStep {
            step_id: String::from("step-find-element"),
            tool_name: ToolName::FindElement,
            arguments: serde_json::json!({
                "request_id": "req-find-element",
                "description": "search field",
                "text": null,
                "role": null,
                "color_hint": null,
                "nearby_text": null,
                "selector_hint": null,
                "visibility_filter": "VisibleOnly",
                "max_candidates": DEFAULT_FIND_ELEMENT_MAX_CANDIDATES + 1
            }),
            purpose: String::from("find an element"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("find_element")],
    )
    .expect_err("validation should reject max_candidates above the supported maximum");
    assert!(error
        .message
        .contains("find_element max_candidates must be less than or equal to"));
}

#[test]
fn validate_planner_output_rejects_set_playback_volume_out_of_range() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::SetPlaybackVolume,
            goal: String::from("set playback volume"),
            target_description: None,
        },
        selected_skills: vec![String::from("audio_controls")],
        steps: vec![PlannedStep {
            step_id: String::from("step-volume"),
            tool_name: ToolName::SetPlaybackVolume,
            arguments: serde_json::json!({
                "request_id": "req-volume",
                "volume": 1.5
            }),
            purpose: String::from("set the volume"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("audio_controls")],
    )
    .expect_err("validation should reject out-of-range playback volume");
    assert!(error
        .message
        .contains("set_playback_volume volume must be between 0.0"));
}

#[test]
fn validate_planner_output_rejects_set_playback_speed_out_of_range() {
    let available_tools = planner_available_tools();
    let planner_output = PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::SetPlaybackSpeed,
            goal: String::from("set playback speed"),
            target_description: None,
        },
        selected_skills: vec![String::from("audio_controls")],
        steps: vec![PlannedStep {
            step_id: String::from("step-speed"),
            tool_name: ToolName::SetPlaybackSpeed,
            arguments: serde_json::json!({
                "request_id": "req-speed",
                "speed": 10.0
            }),
            purpose: String::from("set the speed"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    };

    let error = validate_planner_output(
        &planner_output,
        &available_tools,
        &[String::from("audio_controls")],
    )
    .expect_err("validation should reject out-of-range playback speed");
    assert!(error
        .message
        .contains("set_playback_speed speed must be between"));
}

#[test]
fn validate_confirm_action_input_rejects_blank_prompt() {
    let error = validate_confirm_action_input(&ConfirmActionInput {
        request_id: String::from("req-confirm"),
        timeout_ms: None,
        prompt_text: String::from("   "),
        reason: String::from("Submission changes remote state."),
    })
    .expect_err("validation should reject blank confirm_action prompt_text");
    assert!(error
        .message
        .contains("confirm_action requires a non-empty prompt_text"));
}

#[test]
fn infer_intent_hint_prefers_audio_queries_over_setters() {
    assert_eq!(
        infer_intent_hint("what is the volume"),
        IntentName::GetPlaybackVolume
    );
    assert_eq!(
        infer_intent_hint("what's the playback speed"),
        IntentName::GetPlaybackSpeed
    );
    assert_eq!(
        infer_intent_hint("what is the volum"),
        IntentName::GetPlaybackVolume
    );
    assert_eq!(
        infer_intent_hint("what s the play back spead"),
        IntentName::GetPlaybackSpeed
    );
}

#[test]
fn planner_skill_regression_fixtures_cover_representative_direct_command_flows() {
    let fixtures = vec![
        PlannerSkillFixture {
            name: "fixture-set-volume",
            transcript: "set volume to 70 percent",
            resolver: PlannerSkillFixtureResolver::Audio,
            agent_state: fixture_agent_state(),
            page_model: None,
            expected_intent: IntentName::SetPlaybackVolume,
            expected_selected_skills: vec!["set_volume"],
            expected_tool_sequence: vec![ToolName::SetPlaybackVolume, ToolName::ReportResult],
        },
        PlannerSkillFixture {
            name: "fixture-go-back",
            transcript: "back",
            resolver: PlannerSkillFixtureResolver::NavigationReadback,
            agent_state: fixture_agent_state(),
            page_model: None,
            expected_intent: IntentName::GoBack,
            expected_selected_skills: vec!["go_back"],
            expected_tool_sequence: vec![ToolName::GoBack],
        },
        PlannerSkillFixture {
            name: "fixture-read-page-extract",
            transcript: "read page",
            resolver: PlannerSkillFixtureResolver::ReadPage,
            agent_state: fixture_agent_state(),
            page_model: Some(fixture_page_model_without_regions()),
            expected_intent: IntentName::ReadPage,
            expected_selected_skills: vec!["read_page"],
            expected_tool_sequence: vec![ToolName::ExtractPageModel, ToolName::ReadNextRegion],
        },
        PlannerSkillFixture {
            name: "fixture-current-url",
            transcript: "what page am i on",
            resolver: PlannerSkillFixtureResolver::StatusQuery,
            agent_state: fixture_agent_state(),
            page_model: None,
            expected_intent: IntentName::GetCurrentUrl,
            expected_selected_skills: vec!["get_current_url"],
            expected_tool_sequence: vec![ToolName::GetAgentState, ToolName::ReportResult],
        },
    ];

    for fixture in fixtures {
        assert_planner_skill_fixture(fixture);
    }
}

#[test]
fn planner_skill_regression_fixtures_cover_problematic_page_shapes() {
    let fixtures = vec![
        PlannerSkillFixture {
            name: "problematic-article-read-page",
            transcript: "read page",
            resolver: PlannerSkillFixtureResolver::ReadPage,
            agent_state: fixture_agent_state_for_page(
                "Metro news | Night trains finally return",
                "https://news.example.com/city/night-trains-return",
            ),
            page_model: Some(fixture_problematic_article_page_without_regions()),
            expected_intent: IntentName::ReadPage,
            expected_selected_skills: vec!["read_page"],
            expected_tool_sequence: vec![ToolName::ExtractPageModel, ToolName::ReadNextRegion],
        },
        PlannerSkillFixture {
            name: "problematic-docs-current-url",
            transcript: "what page am i on",
            resolver: PlannerSkillFixtureResolver::StatusQuery,
            agent_state: fixture_problematic_docs_agent_state(),
            page_model: None,
            expected_intent: IntentName::GetCurrentUrl,
            expected_selected_skills: vec!["get_current_url"],
            expected_tool_sequence: vec![ToolName::GetAgentState, ToolName::ReportResult],
        },
    ];

    for fixture in fixtures {
        assert_planner_skill_fixture(fixture);
    }
}

#[test]
fn infer_intent_hint_recognizes_browser_visibility_phrases() {
    assert_eq!(
        infer_intent_hint("go headless"),
        IntentName::SetBrowserVisibility
    );
    assert_eq!(
        infer_intent_hint("make the browser visible"),
        IntentName::SetBrowserVisibility
    );
    assert_eq!(
        infer_intent_hint("show the browsr"),
        IntentName::SetBrowserVisibility
    );
    assert_eq!(
        infer_intent_hint("go head less"),
        IntentName::SetBrowserVisibility
    );
}

#[test]
fn infer_intent_hint_recognizes_status_and_history_queries() {
    assert_eq!(infer_intent_hint("can i go back"), IntentName::GetStatus);
    assert_eq!(
        infer_intent_hint("are you listening"),
        IntentName::GetStatus
    );
    assert_eq!(
        infer_intent_hint("what page am i on"),
        IntentName::GetCurrentUrl
    );
    assert_eq!(
        infer_intent_hint("what is the statuz"),
        IntentName::GetStatus
    );
    assert_eq!(infer_intent_hint("are you listenin"), IntentName::GetStatus);
    assert_eq!(
        infer_intent_hint("what is the curent url"),
        IntentName::GetCurrentUrl
    );
}

#[test]
fn infer_intent_hint_recognizes_navigation_readback_action_phrases() {
    assert_eq!(infer_intent_hint("back"), IntentName::GoBack);
    assert_eq!(infer_intent_hint("go forward"), IntentName::GoForward);
    assert_eq!(infer_intent_hint("refesh page"), IntentName::ReloadPage);
    assert_eq!(infer_intent_hint("next"), IntentName::ReadNext);
    assert_eq!(
        infer_intent_hint("prevous region"),
        IntentName::ReadPrevious
    );
    assert_eq!(infer_intent_hint("stpo reading"), IntentName::Stop);
}

#[test]
fn infer_intent_hint_recognizes_voice_input_phrases() {
    assert_eq!(
        infer_intent_hint("start listening"),
        IntentName::StartListening
    );
    assert_eq!(
        infer_intent_hint("stop listenin"),
        IntentName::StopListening
    );
    assert_eq!(
        infer_intent_hint("what did i just say"),
        IntentName::TranscribeCommand
    );
    assert_eq!(
        infer_intent_hint("transcribe this"),
        IntentName::TranscribeCommand
    );
}

#[test]
fn infer_intent_hint_recognizes_open_url_phrases() {
    assert_eq!(
        infer_intent_hint("open github dot com"),
        IntentName::OpenUrl
    );
    assert_eq!(
        infer_intent_hint("go to https://example.com"),
        IntentName::OpenUrl
    );
    assert_eq!(
        infer_intent_hint("visit localhost colon 3000"),
        IntentName::OpenUrl
    );
}

#[test]
fn infer_intent_hint_recognizes_read_page_phrases() {
    assert_eq!(infer_intent_hint("read page"), IntentName::ReadPage);
    assert_eq!(infer_intent_hint("read this page"), IntentName::ReadPage);
    assert_eq!(infer_intent_hint("read current page"), IntentName::ReadPage);
}

#[test]
fn infer_intent_hint_recognizes_form_filling_and_submission_phrases() {
    assert_eq!(
        infer_intent_hint("focus the email field"),
        IntentName::FillInput
    );
    assert_eq!(
        infer_intent_hint("fill the password field"),
        IntentName::FillInput
    );
    assert_eq!(
        infer_intent_hint("type hello into the search field"),
        IntentName::FillInput
    );
    assert_eq!(
        infer_intent_hint("submit this form"),
        IntentName::SubmitForm
    );
    assert_eq!(
        infer_intent_hint("fill the email field and then submit"),
        IntentName::SubmitForm
    );
    assert_eq!(
        infer_intent_hint("no, the other field"),
        IntentName::FillInput
    );
    assert_eq!(
        infer_intent_hint("put Seattle there instead"),
        IntentName::FillInput
    );
    assert_eq!(
        infer_intent_hint("choose California from the state list"),
        IntentName::FillInput
    );
    assert_eq!(
        infer_intent_hint("foccus the email feild"),
        IntentName::FillInput
    );
    assert_eq!(
        infer_intent_hint("submitt this form"),
        IntentName::SubmitForm
    );
}

#[test]
fn parse_direct_focus_field_command_extracts_field_description() {
    assert_eq!(
        parse_direct_focus_field_command("focus the email field"),
        Some(FocusFieldCommand {
            description: Some(String::from("email"))
        })
    );
    assert_eq!(
        parse_direct_focus_field_command("foccus the password feild"),
        Some(FocusFieldCommand {
            description: Some(String::from("password"))
        })
    );
    assert_eq!(
        parse_direct_focus_field_command("focus field"),
        Some(FocusFieldCommand { description: None })
    );
    assert_eq!(parse_direct_focus_field_command("read page"), None);
}

#[test]
fn parse_direct_fill_field_command_extracts_description_and_text() {
    assert_eq!(
        parse_direct_fill_field_command("fill the email field with phil@example.com"),
        Some(FillFieldCommand {
            description: Some(String::from("email")),
            text: Some(String::from("phil@example.com"))
        })
    );
    assert_eq!(
        parse_direct_fill_field_command("type \"hello world\" into the search field"),
        Some(FillFieldCommand {
            description: Some(String::from("search")),
            text: Some(String::from("hello world"))
        })
    );
    assert_eq!(
        parse_direct_fill_field_command("enter secret in the password field"),
        Some(FillFieldCommand {
            description: Some(String::from("password")),
            text: Some(String::from("secret"))
        })
    );
    assert_eq!(
        parse_direct_fill_field_command("fill the email field"),
        Some(FillFieldCommand {
            description: Some(String::from("email")),
            text: None
        })
    );
    assert_eq!(
        parse_direct_fill_field_command("focus the email field"),
        None
    );
}

#[test]
fn parse_fill_field_correction_command_extracts_follow_up_corrections() {
    assert_eq!(
        parse_fill_field_correction_command("no, the other field"),
        Some(FillFieldCorrectionCommand::AlternateField)
    );
    assert_eq!(
        parse_fill_field_correction_command("put Seattle there instead"),
        Some(FillFieldCorrectionCommand::ReplaceValue {
            text: String::from("Seattle")
        })
    );
    assert_eq!(
        parse_fill_field_correction_command("type \"hello world\" there instead"),
        Some(FillFieldCorrectionCommand::ReplaceValue {
            text: String::from("hello world")
        })
    );
    assert_eq!(parse_fill_field_correction_command("read page"), None);
}

#[test]
fn parse_direct_fill_and_submit_command_extracts_description_and_text() {
    assert_eq!(
        parse_direct_fill_and_submit_command(
            "fill the email field with phil@example.com and then submit"
        ),
        Some(FillFieldCommand {
            description: Some(String::from("email")),
            text: Some(String::from("phil@example.com"))
        })
    );
    assert_eq!(
        parse_direct_fill_and_submit_command(
            "type hello world into the search field and submit form"
        ),
        Some(FillFieldCommand {
            description: Some(String::from("search")),
            text: Some(String::from("hello world"))
        })
    );
    assert_eq!(
        parse_direct_fill_and_submit_command("fill the email field and submit"),
        Some(FillFieldCommand {
            description: Some(String::from("email")),
            text: None
        })
    );
    assert_eq!(parse_direct_fill_and_submit_command("submit form"), None);
}

#[test]
fn normalize_transcript_for_routing_merges_compound_tokens_and_sanitizes_punctuation() {
    assert_eq!(
        normalize_transcript_for_routing("Go HEAD less, please!!"),
        "go headless please"
    );
    assert_eq!(
        normalize_transcript_for_routing("What'S the PLAY back spead???"),
        "what s the playback speed"
    );
    assert_eq!(
        normalize_transcript_for_routing("focus the e-mail field."),
        "focus the e mail field"
    );
}

#[test]
fn parse_intent_name_value_accepts_cleaned_values_and_rejects_unknown_intents() {
    assert_eq!(
        parse_intent_name_value(" `OpenUrl` ").expect("open url intent should parse"),
        IntentName::OpenUrl
    );
    assert_eq!(
        parse_intent_name_value("\"SetBrowserVisibility\"")
            .expect("browser visibility intent should parse"),
        IntentName::SetBrowserVisibility
    );
    assert_eq!(
        parse_intent_name_value("'Unknown'").expect("unknown sentinel should parse"),
        IntentName::Unknown
    );

    let error =
        parse_intent_name_value("LaunchMissiles").expect_err("unknown intents should be rejected");
    assert!(error.contains("unknown intent tag"));
    assert!(error.contains("LaunchMissiles"));
}

#[test]
fn infer_intent_hint_recognizes_repeat_phrases() {
    assert_eq!(infer_intent_hint("repeat"), IntentName::Repeat);
    assert_eq!(infer_intent_hint("repeat that"), IntentName::Repeat);
    assert_eq!(infer_intent_hint("read that again"), IntentName::Repeat);
    assert_eq!(infer_intent_hint("say that again"), IntentName::Repeat);
}

#[test]
fn infer_intent_hint_recognizes_read_title_phrases() {
    assert_eq!(infer_intent_hint("read title"), IntentName::ReadTitle);
    assert_eq!(
        infer_intent_hint("read the page title"),
        IntentName::ReadTitle
    );
    assert_eq!(
        infer_intent_hint("what is the title"),
        IntentName::ReadTitle
    );
}

#[test]
fn infer_intent_hint_recognizes_tts_voice_phrases() {
    assert_eq!(
        infer_intent_hint("change the voice to Bruno"),
        IntentName::SetTtsVoice
    );
    assert_eq!(
        infer_intent_hint("switch to the Bella voice"),
        IntentName::SetTtsVoice
    );
    assert_eq!(
        infer_intent_hint("use the Hugo voice"),
        IntentName::SetTtsVoice
    );
    assert_eq!(
        infer_intent_hint("set the voise to Luna"),
        IntentName::SetTtsVoice
    );
}

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
fn assert_json_matches_schema(
    instance: &serde_json::Value,
    schema: &serde_json::Value,
) -> Result<(), String> {
    assert_json_matches_schema_at(instance, schema, schema, "$")
}

fn assert_json_matches_schema_at(
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

fn resolve_schema_reference<'a>(
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

fn json_matches_type(instance: &serde_json::Value, type_schema: &serde_json::Value) -> bool {
    match type_schema {
        serde_json::Value::String(kind) => json_matches_single_type(instance, kind),
        serde_json::Value::Array(kinds) => kinds.iter().any(|kind| {
            kind.as_str()
                .is_some_and(|kind| json_matches_single_type(instance, kind))
        }),
        _ => true,
    }
}

fn json_matches_single_type(instance: &serde_json::Value, kind: &str) -> bool {
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
