// FILE SIZE EXEMPTION: this file exceeds the 600-line target documented in the project TODO.
// It is intentionally exempt because it is a single coherent mock implementation of the entire
// DeterministicToolExecutor trait. Splitting it across submodules would fragment what is logically
// one unit (a test double for all deterministic tools) without reducing complexity.
// Exemption recorded in docs/BLIND_BROWSER_UIUX_FIX3_TODO.md P2.3.

use super::*;

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
