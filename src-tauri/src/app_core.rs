use std::collections::BTreeMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asr::{
    AsrController, AsrRuntimeError, DEFAULT_TRANSCRIBE_DURATION_MS, MAX_TRANSCRIBE_DURATION_MS,
};
use crate::audio_io::{AudioPlaybackController, AudioPlaybackError};
use crate::browser::{
    BrowserController, BrowserError, BrowserSessionConfig, BrowserVisibilityMode, LoadState,
};
use crate::commands::{
    build_planner_skill_selection, execute_planner_output, planner_available_tools,
    planner_output_schema, resume_after_confirmation, tool_input_schema, validate_planner_output,
    AgentStateData, ClickElementData, ClickElementInput, ConfirmActionData, ConfirmActionInput,
    ConfirmActionResolution, DeterministicToolExecutor, ExecutionOutcome, ExtractPageModelData,
    ExtractPageModelInput, FindElementData, FindElementInput, GetAgentStateInput,
    GetPageSnapshotInput, GetRuntimeStatusData, GetRuntimeStatusInput, GoBackData, GoBackInput,
    GoForwardData, GoForwardInput, ListInteractiveElementsData, ListInteractiveElementsInput,
    OpenUrlData, OpenUrlInput, PageSnapshotData, PlannerInput, PlannerOutput,
    ProviderSelectionStatus, ReadNextRegionData, ReadNextRegionInput, ReadPreviousRegionData,
    ReadPreviousRegionInput, ReadRegionData, ReadRegionInput, ReloadPageData, ReloadPageInput,
    ReportResultData, ReportResultInput, ScrollPageData, ScrollPageInput, SetBrowserVisibilityData,
    SetBrowserVisibilityInput, SetPlaybackSpeedData, SetPlaybackSpeedInput, SetPlaybackVolumeData,
    SetPlaybackVolumeInput, SetTtsVoiceData, SetTtsVoiceInput, StartListeningData,
    StartListeningInput, StopListeningData, StopListeningInput, StopSpeakingData,
    StopSpeakingInput, ToolError, ToolName, ToolResult, TranscribeCommandData,
    TranscribeCommandInput,
};
use crate::config::{
    AppConfig, AudioSettings, ConfigError, RemotePlannerProfile, RemoteProviderKind, SecretRef,
};
use crate::narration::{
    cursor_for_index, find_region_index, next_region_index, previous_region_index,
    spoken_text_for_region,
};
use crate::page_model::PageRegion;
use crate::page_model::{ElementRole, ExtractionSource, PageModel, RegionSource};
use crate::state::AppState;
use crate::tts::{TtsController, TtsRuntimeError};
use serde::Serialize;
use tauri::{AppHandle, Manager};

const DEFAULT_FIND_ELEMENT_MAX_CANDIDATES: usize = 3;
const MAX_FIND_ELEMENT_CANDIDATES: usize = 5;
const FIND_ELEMENT_STRONG_MATCH_BPS: u16 = 8_500;
const FIND_ELEMENT_AMBIGUITY_MARGIN_BPS: u16 = 800;
const MAX_HISTORY_STEPS: u8 = 5;
const MAX_SCROLL_AMOUNT_PX: f32 = 4_000.0;
#[derive(Serialize)]
struct PlannerPromptPayload<'a> {
    planner_input: &'a PlannerInput,
    planner_output_schema: serde_json::Value,
    tool_input_schemas: BTreeMap<String, serde_json::Value>,
}

pub struct AppCore {
    pub app_handle: AppHandle,
    pub config: AppConfig,
    pub state: AppState,
    pub browser: BrowserController,
    tts: TtsController,
    playback: AudioPlaybackController,
    asr: AsrController,
}

impl AppCore {
    pub fn new(app_handle: AppHandle) -> Result<Self, ConfigError> {
        let config = AppConfig::load_for_app(&app_handle)?;
        let state = AppState::from_config(&config);
        let browser = BrowserController::new(BrowserSessionConfig {
            visibility: state.browser_visibility,
            user_agent: None,
        });

        Ok(Self {
            app_handle,
            config,
            state,
            browser,
            tts: TtsController::new(),
            playback: AudioPlaybackController::new(),
            asr: AsrController::new(),
        })
    }

    pub fn update_audio_settings(&mut self, audio: AudioSettings) -> Result<(), ConfigError> {
        let config = AppConfig::persist_audio_settings_for_app(&self.app_handle, &audio)?;
        self.state.apply_audio_settings(&config.audio);
        self.config = config;
        Ok(())
    }

    pub fn set_playback_volume(&mut self, playback_volume: f32) -> Result<(), ConfigError> {
        let mut audio = self.config.audio.clone();
        audio.playback_volume = playback_volume;
        self.update_audio_settings(audio)
    }

    pub fn set_playback_speed(&mut self, playback_speed: f32) -> Result<(), ConfigError> {
        let mut audio = self.config.audio.clone();
        audio.playback_speed = playback_speed;
        self.update_audio_settings(audio)
    }

    pub fn set_default_tts_voice(
        &mut self,
        default_tts_voice: impl Into<String>,
    ) -> Result<(), ConfigError> {
        let mut audio = self.config.audio.clone();
        audio.default_tts_voice = default_tts_voice.into();
        self.update_audio_settings(audio)
    }

    pub fn set_browser_visibility(&mut self, mode: BrowserVisibilityMode) {
        self.state.browser_visibility = mode;
    }

    pub fn execute_open_url(&mut self, input: OpenUrlInput) -> ToolResult<OpenUrlData> {
        let final_url = match normalize_absolute_url(&input.url) {
            Ok(url) => url,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::OpenUrl,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Navigation request was rejected because the URL was not an absolute URL.",
                    )],
                )
            }
        };

        let load_state = input.wait_for_load_state.unwrap_or(LoadState::Load);
        let browser_page = match self
            .browser
            .open_url(&final_url, load_state, input.timeout_ms)
        {
            Ok(browser_page) => browser_page,
            Err(error) => {
                return self.browser_tool_failure(
                    ToolName::OpenUrl,
                    input.request_id,
                    String::from("Browser navigation did not complete successfully."),
                    error,
                )
            }
        };

        let page_id = self.next_page_id(&input.request_id);
        self.stop_narration_playback();
        self.state
            .record_navigation(page_id.clone(), browser_page.url.clone());
        if let Some(current_page) = self.state.current_page.as_mut() {
            current_page.title = browser_page.title.clone();
        }
        self.state.browser_history = browser_page.history.clone();

        ToolResult::success(
            ToolName::OpenUrl,
            input.request_id,
            OpenUrlData {
                final_url: browser_page.url,
                title: browser_page.title,
                page_id,
                load_state,
                http_status: None,
                history: browser_page.history,
            },
            vec![
                String::from(
                    "Validated the requested absolute URL and navigated the live Chromium page.",
                ),
                String::from(
                    "Runtime navigation state now reflects the live browser URL and document title.",
                ),
            ],
        )
    }

    pub fn execute_go_back(&mut self, input: GoBackInput) -> ToolResult<GoBackData> {
        if self.state.current_page_id.is_none() {
            return Self::browser_runtime_missing_page(ToolName::GoBack, input.request_id);
        }

        let requested_steps = input.steps.unwrap_or(1).clamp(1, MAX_HISTORY_STEPS);
        let load_state = input.wait_for_load_state.unwrap_or(LoadState::Load);
        let browser_navigation =
            match self
                .browser
                .go_back(requested_steps, load_state, input.timeout_ms)
            {
                Ok(browser_navigation) => browser_navigation,
                Err(error) => {
                    return self.browser_tool_failure(
                        ToolName::GoBack,
                        input.request_id,
                        String::from(
                            "Live browser history navigation did not complete successfully.",
                        ),
                        error,
                    )
                }
            };

        self.state.browser_history = browser_navigation.history.clone();
        if browser_navigation.navigated {
            self.stop_narration_playback();
            if let Some(current_page) = self.state.current_page.as_mut() {
                current_page.url = browser_navigation.url.clone();
                current_page.title = browser_navigation.title.clone();
                current_page.regions.clear();
                current_page.interactive_elements.clear();
            }
            self.state.narration_cursor = Default::default();
        }

        let mut observations = vec![format!(
            "Requested backward history navigation for up to {} step(s).",
            requested_steps
        )];
        if input.steps.is_some_and(|steps| steps > MAX_HISTORY_STEPS) {
            observations.push(format!(
                "Requested steps were clamped to the supported maximum of {}.",
                MAX_HISTORY_STEPS
            ));
        }
        observations.push(if browser_navigation.navigated {
            String::from("The live browser moved backward in history and runtime page metadata was refreshed.")
        } else {
            String::from("The live browser was already at the earliest reachable history entry.")
        });

        ToolResult::success(
            ToolName::GoBack,
            input.request_id,
            GoBackData {
                navigated: browser_navigation.navigated,
                actual_steps: if browser_navigation.navigated {
                    requested_steps
                } else {
                    0
                },
                final_url: browser_navigation.url,
                title: browser_navigation.title,
                load_state: browser_navigation.navigated.then_some(load_state),
                history: browser_navigation.history,
            },
            observations,
        )
    }

    pub fn execute_go_forward(&mut self, input: GoForwardInput) -> ToolResult<GoForwardData> {
        if self.state.current_page_id.is_none() {
            return Self::browser_runtime_missing_page(ToolName::GoForward, input.request_id);
        }

        let requested_steps = input.steps.unwrap_or(1).clamp(1, MAX_HISTORY_STEPS);
        let load_state = input.wait_for_load_state.unwrap_or(LoadState::Load);
        let browser_navigation =
            match self
                .browser
                .go_forward(requested_steps, load_state, input.timeout_ms)
            {
                Ok(browser_navigation) => browser_navigation,
                Err(error) => {
                    return self.browser_tool_failure(
                        ToolName::GoForward,
                        input.request_id,
                        String::from(
                            "Live browser forward navigation did not complete successfully.",
                        ),
                        error,
                    )
                }
            };

        self.state.browser_history = browser_navigation.history.clone();
        if browser_navigation.navigated {
            self.stop_narration_playback();
            if let Some(current_page) = self.state.current_page.as_mut() {
                current_page.url = browser_navigation.url.clone();
                current_page.title = browser_navigation.title.clone();
                current_page.regions.clear();
                current_page.interactive_elements.clear();
            }
            self.state.narration_cursor = Default::default();
        }

        let mut observations = vec![format!(
            "Requested forward history navigation for up to {} step(s).",
            requested_steps
        )];
        if input.steps.is_some_and(|steps| steps > MAX_HISTORY_STEPS) {
            observations.push(format!(
                "Requested steps were clamped to the supported maximum of {}.",
                MAX_HISTORY_STEPS
            ));
        }
        observations.push(if browser_navigation.navigated {
            String::from("The live browser moved forward in history and runtime page metadata was refreshed.")
        } else {
            String::from("The live browser was already at the latest reachable history entry.")
        });

        ToolResult::success(
            ToolName::GoForward,
            input.request_id,
            GoForwardData {
                navigated: browser_navigation.navigated,
                actual_steps: if browser_navigation.navigated {
                    requested_steps
                } else {
                    0
                },
                final_url: browser_navigation.url,
                title: browser_navigation.title,
                load_state: browser_navigation.navigated.then_some(load_state),
                history: browser_navigation.history,
            },
            observations,
        )
    }

    pub fn execute_reload_page(&mut self, input: ReloadPageInput) -> ToolResult<ReloadPageData> {
        if self.state.current_page_id.is_none() {
            return Self::browser_runtime_missing_page(ToolName::ReloadPage, input.request_id);
        }

        let load_state = input.wait_for_load_state.unwrap_or(LoadState::Load);
        let browser_page =
            match self
                .browser
                .reload_page(input.hard_reload, load_state, input.timeout_ms)
            {
                Ok(browser_page) => browser_page,
                Err(error) => {
                    return self.browser_tool_failure(
                        ToolName::ReloadPage,
                        input.request_id,
                        String::from("Live browser reload did not complete successfully."),
                        error,
                    )
                }
            };

        self.state.browser_history = browser_page.history.clone();
        self.stop_narration_playback();
        if let Some(current_page) = self.state.current_page.as_mut() {
            current_page.url = Some(browser_page.url.clone());
            current_page.title = browser_page.title.clone();
            current_page.regions.clear();
            current_page.interactive_elements.clear();
        }
        self.state.narration_cursor = Default::default();

        let mut observations = vec![String::from(
            "Reloaded the live browser page and refreshed runtime page metadata.",
        )];
        if input.hard_reload {
            observations.push(String::from(
                "The reload ignored browser cache as requested.",
            ));
        }

        ToolResult::success(
            ToolName::ReloadPage,
            input.request_id,
            ReloadPageData {
                reloaded: true,
                final_url: browser_page.url,
                title: browser_page.title,
                load_state,
                http_status: None,
                history: browser_page.history,
            },
            observations,
        )
    }

    pub fn execute_scroll_page(&mut self, input: ScrollPageInput) -> ToolResult<ScrollPageData> {
        if self.state.current_page_id.is_none() {
            return Self::browser_runtime_missing_page(ToolName::ScrollPage, input.request_id);
        }

        if input.amount_px.is_none() && input.target.is_none() {
            return ToolResult::failure(
                ToolName::ScrollPage,
                input.request_id,
                ToolError {
                    code: String::from("invalid_scroll_request"),
                    message: String::from(
                        "scroll_page requires amount_px or target to be provided",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Scroll request was rejected because it did not specify an amount or a target.",
                )],
            );
        }

        let clamped_amount = input
            .amount_px
            .map(|amount| amount.clamp(0.0, MAX_SCROLL_AMOUNT_PX));
        let scroll_state = match self.browser.scroll_page(
            input.direction,
            clamped_amount,
            input.target,
            input.timeout_ms,
        ) {
            Ok(scroll_state) => scroll_state,
            Err(error) => {
                return self.browser_tool_failure(
                    ToolName::ScrollPage,
                    input.request_id,
                    String::from("Live browser scrolling did not complete successfully."),
                    error,
                )
            }
        };

        let mut observations = vec![String::from("Executed a live browser scroll request.")];
        if input
            .amount_px
            .zip(clamped_amount)
            .is_some_and(|(requested, clamped)| (requested - clamped).abs() > f32::EPSILON)
        {
            observations.push(format!(
                "Requested scroll amount was clamped to the supported maximum of {} px.",
                MAX_SCROLL_AMOUNT_PX
            ));
        }
        if scroll_state.reached_boundary {
            observations.push(String::from(
                "The scroll request reached a document boundary.",
            ));
        }

        ToolResult::success(
            ToolName::ScrollPage,
            input.request_id,
            ScrollPageData {
                previous_scroll_y: scroll_state.previous_scroll_y,
                current_scroll_y: scroll_state.current_scroll_y,
                reached_boundary: scroll_state.reached_boundary,
            },
            observations,
        )
    }

    pub fn execute_get_page_snapshot(
        &mut self,
        input: GetPageSnapshotInput,
    ) -> ToolResult<PageSnapshotData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return ToolResult::failure(
                ToolName::GetPageSnapshot,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from(
                        "get_page_snapshot requires an active page in runtime state",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Page snapshot could not be created because no page has been opened yet.",
                )],
            );
        };

        let Some(current_page) = self.state.current_page.as_ref() else {
            return ToolResult::failure(
                ToolName::GetPageSnapshot,
                input.request_id,
                ToolError {
                    code: String::from("missing_page_model"),
                    message: String::from(
                        "get_page_snapshot requires runtime page data for the active page",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({ "page_id": page_id })),
                },
                vec![String::from(
                    "Page snapshot could not be created because the runtime page model is missing.",
                )],
            );
        };

        let Some(url) = current_page.url.clone() else {
            return ToolResult::failure(
                ToolName::GetPageSnapshot,
                input.request_id,
                ToolError {
                    code: String::from("missing_page_url"),
                    message: String::from(
                        "get_page_snapshot requires a current page URL in runtime state",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({ "page_id": page_id })),
                },
                vec![String::from(
                    "Page snapshot could not be created because the runtime page URL is missing.",
                )],
            );
        };

        let visible_text_excerpt =
            build_visible_text_excerpt(current_page, input.text_excerpt_max_chars);
        let interactive_elements = if input.include_interactive_elements {
            current_page.interactive_elements.clone()
        } else {
            Vec::new()
        };

        ToolResult::success(
            ToolName::GetPageSnapshot,
            input.request_id,
            PageSnapshotData {
                page_id,
                url,
                title: current_page.title.clone(),
                visible_text_excerpt,
                interactive_elements,
                scroll_y: 0.0,
                viewport_width: 0.0,
                viewport_height: 0.0,
                document_height: 0.0,
            },
            vec![
                String::from(
                    "Built a deterministic page snapshot from the current runtime page state.",
                ),
                String::from(
                    "Scroll and viewport metrics remain placeholder values until the browser backend is wired.",
                ),
            ],
        )
    }

    pub fn execute_extract_page_model(
        &mut self,
        input: ExtractPageModelInput,
    ) -> ToolResult<ExtractPageModelData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return ToolResult::failure(
                ToolName::ExtractPageModel,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from(
                        "extract_page_model requires an active page in runtime state",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Page model extraction could not run because no page has been opened yet.",
                )],
            );
        };

        let base_page_model = if input.use_dom_extraction {
            let extracted_page_model = match self.browser.extract_page_model() {
                Ok(extracted_page_model) => extracted_page_model,
                Err(error) => {
                    return self.browser_tool_failure(
                        ToolName::ExtractPageModel,
                        input.request_id,
                        String::from(
                            "Live browser page-model extraction did not complete successfully.",
                        ),
                        error,
                    )
                }
            };
            self.state.current_page = Some(extracted_page_model.clone());
            extracted_page_model
        } else {
            let Some(current_page) = self.state.current_page.as_ref() else {
                return ToolResult::failure(
                    ToolName::ExtractPageModel,
                    input.request_id,
                    ToolError {
                        code: String::from("missing_page_model"),
                        message: String::from(
                            "extract_page_model requires runtime page data for the active page",
                        ),
                        retryable: false,
                        details: Some(serde_json::json!({ "page_id": page_id })),
                    },
                    vec![String::from(
                        "Page model extraction could not run because the runtime page model is missing.",
                    )],
                );
            };

            current_page.clone()
        };

        let extracted_page_model = build_extracted_page_model(&base_page_model, &input);
        let region_count = extracted_page_model.regions.len();
        let readable_region_count = extracted_page_model
            .regions
            .iter()
            .filter(|region| !region.text.trim().is_empty())
            .count();
        let extraction_source = infer_extraction_source(&base_page_model, input.use_dom_extraction);

        let mut observations = if input.use_dom_extraction {
            vec![String::from(
                "Built a deterministic page model from the live Chromium DOM and persisted it into runtime state.",
            )]
        } else {
            vec![String::from(
                "Built a deterministic page model from the current runtime page state.",
            )]
        };
        if !input.include_links {
            observations.push(String::from(
                "Link elements were omitted from the extracted page model as requested.",
            ));
        }
        if input.include_headings {
            observations.push(String::from(
                "Heading-specific extraction is not yet distinguished in the current page model schema, so regions were returned unchanged.",
            ));
        }
        if !input.use_dom_extraction {
            observations.push(String::from(
                "A non-DOM extraction request currently reuses runtime page state until OCR-specific extraction is wired.",
            ));
        }

        ToolResult::success(
            ToolName::ExtractPageModel,
            input.request_id,
            ExtractPageModelData {
                page_model: extracted_page_model,
                region_count,
                readable_region_count,
                extraction_source,
            },
            observations,
        )
    }

    pub fn execute_list_interactive_elements(
        &mut self,
        input: ListInteractiveElementsInput,
    ) -> ToolResult<ListInteractiveElementsData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return ToolResult::failure(
                ToolName::ListInteractiveElements,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from(
                        "list_interactive_elements requires an active page in runtime state",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Interactive elements could not be listed because no page has been opened yet.",
                )],
            );
        };

        let Some(current_page) = self.state.current_page.as_ref() else {
            return ToolResult::failure(
                ToolName::ListInteractiveElements,
                input.request_id,
                ToolError {
                    code: String::from("missing_page_model"),
                    message: String::from(
                        "list_interactive_elements requires runtime page data for the active page",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({ "page_id": page_id })),
                },
                vec![String::from(
                    "Interactive elements could not be listed because the runtime page model is missing.",
                )],
            );
        };

        let elements = filter_interactive_elements(
            &current_page.interactive_elements,
            input.visible_only,
            input.roles.as_deref(),
        );
        let visible_count = elements.iter().filter(|element| element.visible).count();

        let mut observations = vec![String::from(
            "Listed deterministic interactive elements from the current runtime page state.",
        )];
        if input.visible_only {
            observations.push(String::from(
                "Results were filtered to elements currently marked visible in runtime state.",
            ));
        }
        if let Some(roles) = input.roles.as_ref() {
            observations.push(format!(
                "Results were filtered to {} requested role(s).",
                roles.len()
            ));
        }

        ToolResult::success(
            ToolName::ListInteractiveElements,
            input.request_id,
            ListInteractiveElementsData {
                page_id,
                elements,
                visible_count,
            },
            observations,
        )
    }

    pub fn execute_find_element(&mut self, input: FindElementInput) -> ToolResult<FindElementData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return ToolResult::failure(
                ToolName::FindElement,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from("find_element requires an active page in runtime state"),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Element search could not run because no page has been opened yet.",
                )],
            );
        };

        let Some(current_page) = self.state.current_page.as_ref() else {
            return ToolResult::failure(
                ToolName::FindElement,
                input.request_id,
                ToolError {
                    code: String::from("missing_page_model"),
                    message: String::from(
                        "find_element requires runtime page data for the active page",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({ "page_id": page_id })),
                },
                vec![String::from(
                    "Element search could not run because the runtime page model is missing.",
                )],
            );
        };

        let search_query = match build_find_element_query(&input) {
            Ok(search_query) => search_query,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::FindElement,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Element search was rejected because the search criteria were empty.",
                    )],
                )
            }
        };

        let candidate_limit = input
            .max_candidates
            .unwrap_or(DEFAULT_FIND_ELEMENT_MAX_CANDIDATES)
            .clamp(1, MAX_FIND_ELEMENT_CANDIDATES);
        let elements = filter_interactive_elements(
            &current_page.interactive_elements,
            input.visible_only,
            input.role.as_ref().map(std::slice::from_ref),
        );
        let ranked_candidates =
            rank_find_element_candidates(&elements, &search_query, candidate_limit);
        let (chosen_element_id, chosen_confidence, requires_confirmation) =
            determine_find_element_resolution(&ranked_candidates);

        let mut observations = vec![format!(
            "Searched {} interactive element(s) from the current runtime page state.",
            elements.len()
        )];
        if input.visible_only {
            observations.push(String::from(
                "Search was limited to elements currently marked visible in runtime state.",
            ));
        }
        if input
            .max_candidates
            .is_some_and(|value| value > MAX_FIND_ELEMENT_CANDIDATES)
        {
            observations.push(format!(
                "Candidate count was clamped to the supported maximum of {}.",
                MAX_FIND_ELEMENT_CANDIDATES
            ));
        }
        if ranked_candidates.is_empty() {
            observations.push(String::from(
                "No interactive elements produced a positive match score for the requested search criteria.",
            ));
        } else if requires_confirmation {
            observations.push(String::from(
                "Top candidates are too close to choose deterministically, so planner clarification is required before any side effect.",
            ));
        } else {
            observations.push(String::from(
                "A single strongest candidate was identified from the filtered interactive elements.",
            ));
        }

        ToolResult::success(
            ToolName::FindElement,
            input.request_id,
            FindElementData {
                query_summary: search_query.summary,
                chosen_element_id,
                chosen_confidence,
                candidates: ranked_candidates,
                requires_confirmation,
            },
            observations,
        )
    }

    pub fn execute_click_element(
        &mut self,
        input: ClickElementInput,
    ) -> ToolResult<ClickElementData> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return ToolResult::failure(
                ToolName::ClickElement,
                input.request_id,
                ToolError {
                    code: String::from("no_active_page"),
                    message: String::from("click_element requires an active page in runtime state"),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Click could not run because no page has been opened yet.",
                )],
            );
        };

        let element = {
            let Some(current_page) = self.state.current_page.as_ref() else {
                return ToolResult::failure(
                    ToolName::ClickElement,
                    input.request_id,
                    ToolError {
                        code: String::from("missing_page_model"),
                        message: String::from(
                            "click_element requires runtime page data for the active page",
                        ),
                        retryable: false,
                        details: Some(serde_json::json!({ "page_id": page_id })),
                    },
                    vec![String::from(
                        "Click could not run because the runtime page model is missing.",
                    )],
                );
            };

            match resolve_clickable_element(current_page, &input.element_id) {
                Ok(element) => element.clone(),
                Err(error) => {
                    return ToolResult::failure(
                        ToolName::ClickElement,
                        input.request_id,
                        error,
                        vec![String::from(
                            "Click could not run because the requested deterministic element_id was not currently interactable.",
                        )],
                    )
                }
            }
        };

        let browser_click =
            match self
                .browser
                .click_element(&element, input.double_click, input.timeout_ms)
            {
                Ok(browser_click) => browser_click,
                Err(error) => {
                    return self.browser_tool_failure(
                        ToolName::ClickElement,
                        input.request_id,
                        String::from("Live browser click did not complete successfully."),
                        error,
                    )
                }
            };

        if browser_click.page_changed {
            let next_page_id = self.next_page_id(&input.request_id);
            self.state
                .record_navigation(next_page_id, browser_click.url.clone());
            if let Some(current_page) = self.state.current_page.as_mut() {
                current_page.title = browser_click.title.clone();
            }
        } else if let Some(current_page) = self.state.current_page.as_mut() {
            current_page.url = Some(browser_click.url.clone());
            current_page.title = browser_click.title.clone();
        }
        self.state.browser_history = browser_click.history.clone();

        let mut observations = vec![format!(
            "Triggered a live Chromium DOM click for element_id={}",
            element.element_id
        )];
        if input.double_click {
            observations.push(String::from(
                "The browser backend executed the click with a double-click count.",
            ));
        }
        if browser_click.page_changed {
            observations.push(String::from(
                "The live browser URL changed after the click, so runtime page state advanced to a new page.",
            ));
        } else {
            observations.push(String::from(
                "The click completed without a live browser navigation, so runtime state stayed on the current page.",
            ));
        }

        ToolResult::success(
            ToolName::ClickElement,
            input.request_id,
            ClickElementData {
                element_id: element.element_id.clone(),
                action_performed: true,
                page_changed: browser_click.page_changed,
                navigation_url: browser_click
                    .page_changed
                    .then_some(browser_click.url.clone()),
                resulting_title: browser_click.title,
            },
            observations,
        )
    }

    pub fn execute_planner_output(
        &mut self,
        request_id: String,
        planner_output: &PlannerOutput,
    ) -> ExecutionOutcome {
        let outcome = execute_planner_output(self, request_id, planner_output);
        self.state.apply_execution_outcome(&outcome);
        outcome
    }

    pub fn resolve_command(
        &mut self,
        request_id: String,
        transcript: String,
    ) -> Result<PlannerOutput, ToolError> {
        let transcript = transcript.trim();
        if transcript.is_empty() {
            return Err(ToolError {
                code: String::from("empty_transcript"),
                message: String::from("resolve_command requires a non-empty transcript"),
                retryable: false,
                details: None,
            });
        }

        let available_tools = planner_available_tools();
        let current_dir = std::env::current_dir().ok();
        let user_skill_root = self
            .app_handle
            .path()
            .app_config_dir()
            .ok()
            .map(|path| path.join("skills"));
        let skill_selection = build_planner_skill_selection(
            current_dir.as_deref(),
            user_skill_root.as_deref(),
            transcript,
            &available_tools,
        );

        let planner_input = PlannerInput {
            request_id: request_id.clone(),
            transcript: transcript.to_string(),
            agent_state: self.current_agent_state_snapshot(true),
            available_tools: available_tools.clone(),
            active_skill_names: skill_selection.active_skill_names.clone(),
            relevant_skill_summaries: skill_selection.relevant_skill_summaries.clone(),
            page_snapshot: self.current_page_snapshot(Some(1_200), true),
            page_model: self.state.current_page.clone(),
            recent_tool_results: Vec::new(),
        };

        let planner_output = self.resolve_planner_output(&planner_input)?;
        validate_planner_output(
            &planner_output,
            &available_tools,
            &planner_input.active_skill_names,
        )?;
        Ok(planner_output)
    }

    pub fn resume_after_confirmation(
        &mut self,
        confirmation_id: &str,
        confirmed: bool,
    ) -> ExecutionOutcome {
        let Some(pending_plan_execution) = self.state.pending_plan_execution.clone() else {
            return ExecutionOutcome::Aborted {
                trace: crate::commands::ExecutionTrace {
                    executed_step_ids: Vec::new(),
                    tool_results: Vec::new(),
                },
                error: ToolError {
                    code: String::from("missing_pending_execution"),
                    message: String::from(
                        "there is no pending plan execution to resume for confirmation",
                    ),
                    retryable: false,
                    details: None,
                },
            };
        };

        if self.state.pending_confirmation_id.as_deref() != Some(confirmation_id) {
            return ExecutionOutcome::Aborted {
                trace: crate::commands::ExecutionTrace {
                    executed_step_ids: Vec::new(),
                    tool_results: Vec::new(),
                },
                error: ToolError {
                    code: String::from("confirmation_id_mismatch"),
                    message: String::from(
                        "confirmation response did not match the stored pending confirmation id",
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({
                        "expected_confirmation_id": self.state.pending_confirmation_id,
                        "received_confirmation_id": confirmation_id,
                    })),
                },
            };
        }

        let outcome =
            resume_after_confirmation(self, &pending_plan_execution, confirmation_id, confirmed);
        self.state.apply_execution_outcome(&outcome);
        outcome
    }

    pub fn submit_confirmation_response(
        &mut self,
        confirmation_id: &str,
        confirmed: bool,
        timed_out: bool,
    ) -> ConfirmActionResolution {
        let prompt_text = self
            .state
            .pending_plan_execution
            .as_ref()
            .filter(|pending| pending.confirmation_id == confirmation_id)
            .map(|pending| pending.prompt_text.clone())
            .unwrap_or_default();

        let should_resume = confirmed && !timed_out;
        let resume_outcome = self.resume_after_confirmation(confirmation_id, should_resume);
        let tool_result = match &resume_outcome {
            ExecutionOutcome::Aborted { error, .. } => ToolResult::failure(
                ToolName::ConfirmAction,
                confirmation_id.to_string(),
                error.clone(),
                vec![String::from(
                    "Confirmation response could not be applied to the pending plan.",
                )],
            ),
            _ => ToolResult::success(
                ToolName::ConfirmAction,
                confirmation_id.to_string(),
                ConfirmActionData {
                    confirmation_id: confirmation_id.to_string(),
                    prompt_text,
                    confirmed: Some(confirmed),
                    timed_out,
                },
                vec![String::from(
                    "Confirmation response was applied to the pending plan execution.",
                )],
            ),
        };

        ConfirmActionResolution {
            tool_result,
            resume_outcome,
        }
    }

    pub fn execute_confirm_action(
        &mut self,
        input: ConfirmActionInput,
    ) -> ToolResult<ConfirmActionData> {
        let prompt_text = input.prompt_text.trim().to_string();
        if prompt_text.is_empty() {
            return ToolResult::failure(
                ToolName::ConfirmAction,
                input.request_id,
                ToolError {
                    code: String::from("invalid_confirmation_prompt"),
                    message: String::from("confirm_action requires a non-empty prompt_text"),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Confirmation request was rejected because the prompt text was empty.",
                )],
            );
        }

        let mut observations = vec![String::from("Prepared a confirmation prompt for the user.")];
        let reason = input.reason.trim();
        if !reason.is_empty() {
            observations.push(reason.to_string());
        }

        ToolResult::success(
            ToolName::ConfirmAction,
            input.request_id.clone(),
            ConfirmActionData {
                confirmation_id: self.next_confirmation_id(&input.request_id),
                prompt_text,
                confirmed: None,
                timed_out: false,
            },
            observations,
        )
    }

    pub fn execute_set_tts_voice(
        &mut self,
        input: SetTtsVoiceInput,
    ) -> ToolResult<SetTtsVoiceData> {
        let voice = input.voice.trim().to_string();
        if voice.is_empty() {
            return ToolResult::failure(
                ToolName::SetTtsVoice,
                input.request_id,
                ToolError {
                    code: String::from("invalid_voice"),
                    message: String::from(
                        "voice must be a non-empty provider-supported voice name",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Voice update rejected because the voice name was empty.",
                )],
            );
        }

        let changed = self.config.audio.default_tts_voice != voice;
        match self.set_default_tts_voice(voice.clone()) {
            Ok(()) => ToolResult::success(
                ToolName::SetTtsVoice,
                input.request_id,
                SetTtsVoiceData { voice, changed },
                vec![String::from("Updated the default TTS voice setting.")],
            ),
            Err(error) => self.audio_tool_failure(
                ToolName::SetTtsVoice,
                input.request_id,
                String::from("Failed to persist the requested TTS voice."),
                error,
            ),
        }
    }

    pub fn execute_set_playback_volume(
        &mut self,
        input: SetPlaybackVolumeInput,
    ) -> ToolResult<SetPlaybackVolumeData> {
        let requested_volume = input.volume;
        let clamped_volume = requested_volume.clamp(
            crate::config::MIN_PLAYBACK_VOLUME,
            crate::config::MAX_PLAYBACK_VOLUME,
        );
        let changed = (self.config.audio.playback_volume - clamped_volume).abs() > f32::EPSILON;

        match self.set_playback_volume(clamped_volume) {
            Ok(()) => {
                self.playback.set_volume(self.state.audio.playback_volume);
                let mut observations = vec![String::from("Updated the playback volume setting.")];
                if (requested_volume - clamped_volume).abs() > f32::EPSILON {
                    observations.push(String::from(
                        "Requested playback volume was clamped to the supported range.",
                    ));
                }

                ToolResult::success(
                    ToolName::SetPlaybackVolume,
                    input.request_id,
                    SetPlaybackVolumeData {
                        playback_volume: self.state.audio.playback_volume,
                        muted: self.state.audio.muted,
                        changed,
                    },
                    observations,
                )
            }
            Err(error) => self.audio_tool_failure(
                ToolName::SetPlaybackVolume,
                input.request_id,
                String::from("Failed to persist the requested playback volume."),
                error,
            ),
        }
    }

    pub fn execute_set_playback_speed(
        &mut self,
        input: SetPlaybackSpeedInput,
    ) -> ToolResult<SetPlaybackSpeedData> {
        let requested_speed = input.speed;
        let clamped_speed = requested_speed.clamp(
            crate::config::MIN_PLAYBACK_SPEED,
            crate::config::MAX_PLAYBACK_SPEED,
        );
        let changed = (self.config.audio.playback_speed - clamped_speed).abs() > f32::EPSILON;

        match self.set_playback_speed(clamped_speed) {
            Ok(()) => {
                let mut observations = vec![String::from("Updated the playback speed setting.")];
                observations.push(String::from(
                    "New narration requests will use the updated native TTS speed.",
                ));
                if (requested_speed - clamped_speed).abs() > f32::EPSILON {
                    observations.push(String::from(
                        "Requested playback speed was clamped to the supported range.",
                    ));
                }

                ToolResult::success(
                    ToolName::SetPlaybackSpeed,
                    input.request_id,
                    SetPlaybackSpeedData {
                        playback_speed: self.state.audio.playback_speed,
                        changed,
                    },
                    observations,
                )
            }
            Err(error) => self.audio_tool_failure(
                ToolName::SetPlaybackSpeed,
                input.request_id,
                String::from("Failed to persist the requested playback speed."),
                error,
            ),
        }
    }

    pub fn execute_set_browser_visibility(
        &mut self,
        input: SetBrowserVisibilityInput,
    ) -> ToolResult<SetBrowserVisibilityData> {
        let changed = self.state.browser_visibility != input.mode;
        self.set_browser_visibility(input.mode);

        ToolResult::success(
            ToolName::SetBrowserVisibility,
            input.request_id,
            SetBrowserVisibilityData {
                mode: self.state.browser_visibility,
                changed,
                supported: true,
            },
            vec![String::from("Updated the browser visibility mode.")],
        )
    }

    pub fn execute_read_region(&mut self, input: ReadRegionInput) -> ToolResult<ReadRegionData> {
        self.sync_narration_playback_state();
        let region_id = input.region_id.trim().to_string();
        if region_id.is_empty() {
            return ToolResult::failure(
                ToolName::ReadRegion,
                input.request_id,
                ToolError {
                    code: String::from("invalid_region_id"),
                    message: String::from("read_region requires a non-empty region_id"),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Narration request was rejected because the region_id was empty.",
                )],
            );
        }

        let (region_index, region) = match self.region_by_id(&region_id) {
            Ok(region) => region,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::ReadRegion,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Narration request could not resolve the requested region in the current page model.",
                    )],
                )
            }
        };

        let interrupted_region_id =
            match self.begin_region_narration(region_index, &region, input.interrupt_current) {
                Ok(interrupted_region_id) => interrupted_region_id,
                Err(error) => {
                    return ToolResult::failure(
                        ToolName::ReadRegion,
                        input.request_id,
                        error,
                        vec![String::from(
                            "Narration request could not start playback for the requested region.",
                        )],
                    )
                }
            };

        let mut observations = vec![format!(
            "Moved the narration cursor to region_id={} at index {}.",
            region.region_id, region_index
        )];
        observations.push(String::from(
            "Started real narration playback using the configured TTS and audio backends.",
        ));
        if let Some(interrupted_region_id) = interrupted_region_id {
            observations.push(format!(
                "Interrupted the previously active narration region {} before starting the new region.",
                interrupted_region_id
            ));
        }

        ToolResult::success(
            ToolName::ReadRegion,
            input.request_id,
            ReadRegionData {
                region_id: region.region_id,
                region_index,
                text_length: region.text.chars().count(),
                speech_started: true,
            },
            observations,
        )
    }

    pub fn execute_read_next_region(
        &mut self,
        input: ReadNextRegionInput,
    ) -> ToolResult<ReadNextRegionData> {
        self.sync_narration_playback_state();
        let regions = match self.readable_regions() {
            Ok(regions) => regions,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::ReadNextRegion,
                    input.request_id,
                    error,
                    vec![String::from(
                    "Narration could not advance because the current page has no readable regions.",
                )],
                )
            }
        };

        let Some(region_index) = next_region_index(&self.state.narration_cursor, regions.len())
        else {
            return ToolResult::success(
                ToolName::ReadNextRegion,
                input.request_id,
                ReadNextRegionData {
                    cursor: self.state.narration_cursor.clone(),
                    region_id: None,
                    speech_started: false,
                    reached_end: true,
                },
                vec![String::from(
                    "Narration is already at the end of the readable region list.",
                )],
            );
        };
        let region = regions[region_index].clone();
        let interrupted_region_id =
            match self.begin_region_narration(region_index, &region, input.interrupt_current) {
                Ok(interrupted_region_id) => interrupted_region_id,
                Err(error) => {
                    return ToolResult::failure(
                        ToolName::ReadNextRegion,
                        input.request_id,
                        error,
                        vec![String::from(
                            "Narration could not advance to the next region for playback.",
                        )],
                    )
                }
            };

        let mut observations = vec![format!(
            "Advanced narration to region_id={} at index {}.",
            region.region_id, region_index
        )];
        observations.push(String::from(
            "Started real narration playback using the configured TTS and audio backends.",
        ));
        if let Some(interrupted_region_id) = interrupted_region_id {
            observations.push(format!(
                "Interrupted the previously active narration region {} before reading the next region.",
                interrupted_region_id
            ));
        }

        ToolResult::success(
            ToolName::ReadNextRegion,
            input.request_id,
            ReadNextRegionData {
                cursor: self.state.narration_cursor.clone(),
                region_id: Some(region.region_id),
                speech_started: true,
                reached_end: false,
            },
            observations,
        )
    }

    pub fn execute_read_previous_region(
        &mut self,
        input: ReadPreviousRegionInput,
    ) -> ToolResult<ReadPreviousRegionData> {
        self.sync_narration_playback_state();
        let regions = match self.readable_regions() {
            Ok(regions) => regions,
            Err(error) => {
                return ToolResult::failure(
                    ToolName::ReadPreviousRegion,
                    input.request_id,
                    error,
                    vec![String::from(
                        "Narration could not move backward because the current page has no readable regions.",
                    )],
                )
            }
        };

        let Some(region_index) = previous_region_index(&self.state.narration_cursor, regions.len())
        else {
            return ToolResult::success(
                ToolName::ReadPreviousRegion,
                input.request_id,
                ReadPreviousRegionData {
                    cursor: self.state.narration_cursor.clone(),
                    region_id: None,
                    speech_started: false,
                    reached_start: true,
                },
                vec![String::from(
                    "Narration is already at the start of the readable region list.",
                )],
            );
        };
        let region = regions[region_index].clone();
        let interrupted_region_id =
            match self.begin_region_narration(region_index, &region, input.interrupt_current) {
                Ok(interrupted_region_id) => interrupted_region_id,
                Err(error) => {
                    return ToolResult::failure(
                        ToolName::ReadPreviousRegion,
                        input.request_id,
                        error,
                        vec![String::from(
                        "Narration could not move backward to the previous region for playback.",
                    )],
                    )
                }
            };

        let mut observations = vec![format!(
            "Moved narration backward to region_id={} at index {}.",
            region.region_id, region_index
        )];
        observations.push(String::from(
            "Started real narration playback using the configured TTS and audio backends.",
        ));
        if let Some(interrupted_region_id) = interrupted_region_id {
            observations.push(format!(
                "Interrupted the previously active narration region {} before reading the previous region.",
                interrupted_region_id
            ));
        }

        ToolResult::success(
            ToolName::ReadPreviousRegion,
            input.request_id,
            ReadPreviousRegionData {
                cursor: self.state.narration_cursor.clone(),
                region_id: Some(region.region_id),
                speech_started: true,
                reached_start: false,
            },
            observations,
        )
    }

    pub fn execute_stop_speaking(
        &mut self,
        input: StopSpeakingInput,
    ) -> ToolResult<StopSpeakingData> {
        self.sync_narration_playback_state();
        let interrupted_region_id = self.stop_narration_playback();
        let stopped = interrupted_region_id.is_some();

        ToolResult::success(
            ToolName::StopSpeaking,
            input.request_id,
            StopSpeakingData {
                stopped,
                interrupted_region_id,
            },
            vec![if stopped {
                String::from("Stopped the active narration region.")
            } else {
                String::from("No narration was active, so there was nothing to stop.")
            }],
        )
    }

    pub fn execute_start_listening(
        &mut self,
        input: StartListeningInput,
    ) -> ToolResult<StartListeningData> {
        match self.asr.start_listening() {
            Ok(activated) => {
                self.state.set_listening(self.asr.is_listening());
                ToolResult::success(
                    ToolName::StartListening,
                    input.request_id,
                    StartListeningData {
                        listening_state: self.state.listening.clone(),
                        activated,
                    },
                    vec![if activated {
                        String::from("Started microphone capture for voice input.")
                    } else {
                        String::from(
                            "Listening was already active, so capture continued unchanged.",
                        )
                    }],
                )
            }
            Err(error) => ToolResult::failure(
                ToolName::StartListening,
                input.request_id,
                asr_runtime_error_to_tool_error(&error),
                vec![String::from(
                    "Could not activate voice input listening in the configured ASR backend.",
                )],
            ),
        }
    }

    pub fn execute_stop_listening(
        &mut self,
        input: StopListeningInput,
    ) -> ToolResult<StopListeningData> {
        let deactivated = self.asr.stop_listening();
        self.state.set_listening(self.asr.is_listening());

        ToolResult::success(
            ToolName::StopListening,
            input.request_id,
            StopListeningData {
                listening_state: self.state.listening.clone(),
                deactivated,
            },
            vec![if deactivated {
                String::from("Stopped microphone capture for voice input.")
            } else {
                String::from("Listening was already inactive, so there was nothing to stop.")
            }],
        )
    }

    pub fn execute_transcribe_command(
        &mut self,
        input: TranscribeCommandInput,
    ) -> ToolResult<TranscribeCommandData> {
        let requested_duration_ms = input
            .max_duration_ms
            .unwrap_or(DEFAULT_TRANSCRIBE_DURATION_MS);
        if requested_duration_ms == 0 {
            return ToolResult::failure(
                ToolName::TranscribeCommand,
                input.request_id,
                ToolError {
                    code: String::from("invalid_max_duration_ms"),
                    message: String::from(
                        "transcribe_command requires max_duration_ms to be greater than zero",
                    ),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Transcription request was rejected because the requested capture duration was zero.",
                )],
            );
        }

        let mut effective_duration_ms = requested_duration_ms.min(MAX_TRANSCRIBE_DURATION_MS);
        if let Some(timeout_ms) = input.timeout_ms {
            effective_duration_ms = effective_duration_ms.min(timeout_ms.max(1));
        }

        match self
            .asr
            .transcribe_command(&self.config, effective_duration_ms, input.auto_stop)
        {
            Ok(result) => {
                self.state.set_listening(result.listening_active);
                self.state.record_transcript(result.transcript.clone());

                let mut observations = vec![String::from(
                    "Captured microphone audio and ran deterministic speech transcription.",
                )];
                if requested_duration_ms > MAX_TRANSCRIBE_DURATION_MS {
                    observations.push(format!(
                        "Requested capture duration was clamped to the supported maximum of {} ms.",
                        MAX_TRANSCRIBE_DURATION_MS
                    ));
                }
                if input
                    .timeout_ms
                    .is_some_and(|timeout_ms| timeout_ms < effective_duration_ms)
                {
                    observations.push(String::from(
                        "Capture duration was reduced to respect the requested tool timeout.",
                    ));
                }
                observations.push(if result.transcript.is_some() {
                    String::from("ASR returned a non-empty spoken command transcript.")
                } else {
                    String::from("ASR completed but did not detect a spoken command transcript.")
                });

                ToolResult::success(
                    ToolName::TranscribeCommand,
                    input.request_id,
                    TranscribeCommandData {
                        transcript: result.transcript,
                        confidence: result.confidence,
                        audio_duration_ms: result.audio_duration_ms,
                        listening_state: self.state.listening.clone(),
                    },
                    observations,
                )
            }
            Err(error) => {
                self.state.set_listening(self.asr.is_listening());
                ToolResult::failure(
                    ToolName::TranscribeCommand,
                    input.request_id,
                    asr_runtime_error_to_tool_error(&error),
                    vec![String::from(
                        "Could not complete spoken-command transcription in the configured ASR backend.",
                    )],
                )
            }
        }
    }

    pub fn execute_get_agent_state(
        &mut self,
        input: GetAgentStateInput,
    ) -> ToolResult<AgentStateData> {
        self.sync_narration_playback_state();
        ToolResult::success(
            ToolName::GetAgentState,
            input.request_id,
            self.current_agent_state_snapshot(input.include_last_transcript),
            vec![String::from("Read the current agent state.")],
        )
    }

    pub fn execute_get_runtime_status(
        &mut self,
        input: GetRuntimeStatusInput,
    ) -> ToolResult<GetRuntimeStatusData> {
        self.sync_narration_playback_state();
        ToolResult::success(
            ToolName::GetRuntimeStatus,
            input.request_id,
            self.current_runtime_status_snapshot(input.include_provider_modes),
            vec![String::from("Read the current runtime status.")],
        )
    }

    pub fn execute_report_result(
        &mut self,
        input: ReportResultInput,
    ) -> ToolResult<ReportResultData> {
        let summary = input.summary.trim().to_string();
        if summary.is_empty() {
            return ToolResult::failure(
                ToolName::ReportResult,
                input.request_id,
                ToolError {
                    code: String::from("invalid_report_summary"),
                    message: String::from("report_result requires a non-empty summary"),
                    retryable: false,
                    details: None,
                },
                vec![String::from(
                    "Final result reporting was rejected because the summary was empty.",
                )],
            );
        }

        let next_recommended_action = normalize_optional_text(input.next_recommended_action);
        let user_message = normalize_optional_text(input.user_message);

        ToolResult::success(
            ToolName::ReportResult,
            input.request_id,
            ReportResultData {
                status: input.status,
                summary,
                next_recommended_action,
                user_message,
            },
            vec![String::from(
                "Reported the final planner result in a structured deterministic payload.",
            )],
        )
    }

    fn readable_regions(&self) -> Result<&[PageRegion], ToolError> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return Err(ToolError {
                code: String::from("no_active_page"),
                message: String::from("narration tool requires an active page in runtime state"),
                retryable: false,
                details: None,
            });
        };
        let Some(current_page) = self.state.current_page.as_ref() else {
            return Err(ToolError {
                code: String::from("missing_page_model"),
                message: String::from(
                    "narration tool requires runtime page data for the active page",
                ),
                retryable: false,
                details: Some(serde_json::json!({ "page_id": page_id })),
            });
        };
        if current_page.regions.is_empty() {
            return Err(ToolError {
                code: String::from("no_readable_regions"),
                message: String::from(
                    "narration tool requires at least one readable region in the current page model",
                ),
                retryable: false,
                details: Some(serde_json::json!({ "page_id": page_id })),
            });
        }

        Ok(&current_page.regions)
    }

    fn region_by_id(&self, region_id: &str) -> Result<(usize, PageRegion), ToolError> {
        let regions = self.readable_regions()?;
        let Some(region_index) = find_region_index(regions, region_id) else {
            return Err(ToolError {
                code: String::from("region_not_found"),
                message: String::from(
                    "read_region could not find the requested region_id in the current page model",
                ),
                retryable: false,
                details: Some(serde_json::json!({ "region_id": region_id })),
            });
        };

        Ok((region_index, regions[region_index].clone()))
    }

    fn begin_region_narration(
        &mut self,
        region_index: usize,
        region: &PageRegion,
        interrupt_current: bool,
    ) -> Result<Option<String>, ToolError> {
        self.sync_narration_playback_state();
        if self.state.speaking && !interrupt_current {
            return Err(ToolError {
                code: String::from("speech_in_progress"),
                message: String::from(
                    "a narration region is already active; set interrupt_current to true to replace it",
                ),
                retryable: false,
                details: Some(serde_json::json!({
                    "active_region_id": self.state.speaking_region_id.clone(),
                })),
            });
        }

        let spoken_text = spoken_text_for_region(region);
        if spoken_text.trim().is_empty() {
            return Err(ToolError {
                code: String::from("empty_region_text"),
                message: String::from(
                    "narration tool requires the selected region to contain readable text",
                ),
                retryable: false,
                details: Some(serde_json::json!({
                    "region_id": region.region_id,
                    "region_index": region_index,
                })),
            });
        }

        let speech = self
            .tts
            .synthesize_narration(&self.config, &self.state.audio, &spoken_text)
            .map_err(tts_runtime_error_to_tool_error)?;

        let interrupted_region_id = if self.state.speaking {
            self.stop_narration_playback()
        } else {
            None
        };

        self.playback
            .play_samples(
                speech.samples,
                speech.channels,
                speech.sample_rate,
                self.state.audio.playback_volume,
            )
            .map_err(audio_playback_error_to_tool_error)?;
        self.state.narration_cursor = cursor_for_index(self.readable_regions()?, region_index);
        self.state.start_speaking_region(region.region_id.clone());

        Ok(interrupted_region_id)
    }

    fn sync_narration_playback_state(&mut self) {
        if !self.playback.is_active() && self.state.speaking {
            self.state.stop_speaking();
        }
    }

    fn stop_narration_playback(&mut self) -> Option<String> {
        let stopped_playback = self.playback.stop();
        let interrupted_region_id = self.state.stop_speaking();

        if interrupted_region_id.is_some() || stopped_playback {
            interrupted_region_id
        } else {
            None
        }
    }

    fn current_agent_state_snapshot(&self, include_last_transcript: bool) -> AgentStateData {
        AgentStateData {
            page_id: self.state.current_page_id.clone(),
            url: self
                .state
                .current_page
                .as_ref()
                .and_then(|page| page.url.clone()),
            title: self
                .state
                .current_page
                .as_ref()
                .and_then(|page| page.title.clone()),
            browser_visibility: self.state.browser_visibility,
            browser_history: self.state.browser_history.clone(),
            narration_cursor: Some(self.state.narration_cursor.clone()),
            speaking: self.state.speaking,
            listening_state: self.state.listening.clone(),
            audio: self.state.audio.clone(),
            last_transcript: if include_last_transcript {
                self.state.last_transcript.clone()
            } else {
                None
            },
            last_action: None,
            pending_confirmation_id: self.state.pending_confirmation_id.clone(),
            pending_plan_execution: self.state.pending_plan_execution.clone(),
        }
    }

    fn current_runtime_status_snapshot(
        &self,
        include_provider_modes: bool,
    ) -> GetRuntimeStatusData {
        GetRuntimeStatusData {
            page_id: self.state.current_page_id.clone(),
            url: self
                .state
                .current_page
                .as_ref()
                .and_then(|page| page.url.clone()),
            title: self
                .state
                .current_page
                .as_ref()
                .and_then(|page| page.title.clone()),
            browser_visibility: self.state.browser_visibility,
            browser_history: self.state.browser_history.clone(),
            listening_state: self.state.listening.clone(),
            speaking: self.state.speaking,
            audio: self.state.audio.clone(),
            pending_confirmation_id: self.state.pending_confirmation_id.clone(),
            pending_plan_execution: self.state.pending_plan_execution.clone(),
            provider_modes: if include_provider_modes {
                Some(ProviderSelectionStatus {
                    planner_mode: self.config.providers.planner.mode.clone(),
                    tts_mode: self.config.providers.tts.mode.clone(),
                    asr_mode: self.config.providers.asr.mode.clone(),
                })
            } else {
                None
            },
        }
    }

    fn current_page_snapshot(
        &self,
        text_excerpt_max_chars: Option<usize>,
        include_interactive_elements: bool,
    ) -> Option<PageSnapshotData> {
        let page_id = self.state.current_page_id.clone()?;
        let current_page = self.state.current_page.as_ref()?;
        let url = current_page.url.clone()?;

        Some(PageSnapshotData {
            page_id,
            url,
            title: current_page.title.clone(),
            visible_text_excerpt: build_visible_text_excerpt(current_page, text_excerpt_max_chars),
            interactive_elements: if include_interactive_elements {
                current_page.interactive_elements.clone()
            } else {
                Vec::new()
            },
            scroll_y: 0.0,
            viewport_width: 0.0,
            viewport_height: 0.0,
            document_height: 0.0,
        })
    }

    fn resolve_planner_output(
        &self,
        planner_input: &PlannerInput,
    ) -> Result<PlannerOutput, ToolError> {
        self.resolve_remote_planner_output(planner_input)
    }

    fn resolve_remote_planner_output(
        &self,
        planner_input: &PlannerInput,
    ) -> Result<PlannerOutput, ToolError> {
        let Some(profile_name) = self.config.providers.planner.remote_profile.as_deref() else {
            return Err(ToolError {
                code: String::from("planner_profile_unavailable"),
                message: String::from("remote planner mode requires a configured planner profile"),
                retryable: false,
                details: None,
            });
        };
        let Some(profile) = self.config.remote_planner_profiles.get(profile_name) else {
            return Err(ToolError {
                code: String::from("planner_profile_unavailable"),
                message: format!(
                    "configured remote planner profile '{profile_name}' was not found"
                ),
                retryable: false,
                details: None,
            });
        };

        match profile.provider {
            RemoteProviderKind::OpenAi => self.resolve_with_openai_planner(profile, planner_input),
            RemoteProviderKind::Ollama => self.resolve_with_ollama_planner(profile, planner_input),
        }
    }

    fn planner_prompt_payload<'a>(
        &self,
        planner_input: &'a PlannerInput,
    ) -> PlannerPromptPayload<'a> {
        let tool_schemas = planner_input
            .available_tools
            .iter()
            .filter_map(|tool| {
                tool_input_schema(&tool.name).map(|schema| (format!("{:?}", tool.name), schema))
            })
            .collect::<BTreeMap<_, _>>();

        PlannerPromptPayload {
            planner_input,
            planner_output_schema: planner_output_schema(),
            tool_input_schemas: tool_schemas,
        }
    }

    #[cfg(feature = "remote-openai")]
    fn resolve_with_openai_planner(
        &self,
        profile: &RemotePlannerProfile,
        planner_input: &PlannerInput,
    ) -> Result<PlannerOutput, ToolError> {
        use async_openai::types::chat::{
            ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
            CreateChatCompletionRequestArgs,
        };
        use async_openai::types::chat::{ResponseFormat, ResponseFormatJsonSchema};
        use async_openai::{config::OpenAIConfig, Client};

        let api_key = resolve_secret_ref(&profile.api_key).map_err(|reason| ToolError {
            code: String::from("planner_secret_unavailable"),
            message: String::from("remote planner API key could not be resolved"),
            retryable: false,
            details: Some(serde_json::json!({ "reason": reason })),
        })?;

        let mut openai_config = OpenAIConfig::new()
            .with_api_base(profile.base_url.clone())
            .with_api_key(api_key);
        if let Some(organization) = profile.organization.as_ref() {
            openai_config =
                openai_config.with_org_id(resolve_secret_ref(organization).map_err(|reason| {
                    ToolError {
                        code: String::from("planner_secret_unavailable"),
                        message: String::from(
                            "remote planner organization secret could not be resolved",
                        ),
                        retryable: false,
                        details: Some(serde_json::json!({ "reason": reason })),
                    }
                })?);
        }
        if let Some(project) = profile.project.as_ref() {
            openai_config = openai_config.with_project_id(project.clone());
        }

        let client = Client::with_config(openai_config);
        let prompt_payload = self.planner_prompt_payload(planner_input);
        let user_content =
            serde_json::to_string_pretty(&prompt_payload).expect("planner prompt should serialize");
        let request = CreateChatCompletionRequestArgs::default()
            .model(profile.model.clone())
            .temperature(profile.temperature_milli as f32 / 1_000.0)
            .max_completion_tokens(profile.max_output_tokens)
            .response_format(ResponseFormat::JsonSchema {
                json_schema: ResponseFormatJsonSchema {
                    description: Some(String::from(
                        "Structured deterministic planner output for blind_browser.",
                    )),
                    name: String::from("planner_output"),
                    schema: Some(planner_output_schema()),
                    strict: Some(true),
                },
            })
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(planner_system_prompt())
                    .build()
                    .map_err(|error| ToolError {
                        code: String::from("planner_request_build_failed"),
                        message: format!(
                            "failed to build planner system message for remote resolution: {error}"
                        ),
                        retryable: false,
                        details: None,
                    })?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(user_content)
                    .build()
                    .map_err(|error| ToolError {
                        code: String::from("planner_request_build_failed"),
                        message: format!(
                            "failed to build planner user message for remote resolution: {error}"
                        ),
                        retryable: false,
                        details: None,
                    })?
                    .into(),
            ])
            .build()
            .map_err(|error| ToolError {
                code: String::from("planner_request_build_failed"),
                message: format!("failed to build remote planner request: {error}"),
                retryable: false,
                details: None,
            })?;

        let response =
            futures::executor::block_on(client.chat().create(request)).map_err(|error| {
                ToolError {
                    code: String::from("planner_request_failed"),
                    message: format!("remote planner request failed: {error}"),
                    retryable: true,
                    details: Some(serde_json::json!({
                        "provider": "OpenAI",
                        "model": profile.model,
                        "base_url": profile.base_url,
                    })),
                }
            })?;
        let content = response
            .choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .ok_or_else(|| ToolError {
                code: String::from("planner_response_missing"),
                message: String::from("remote planner returned no structured content"),
                retryable: true,
                details: None,
            })?;

        serde_json::from_str::<PlannerOutput>(&content).map_err(|error| ToolError {
            code: String::from("planner_response_invalid"),
            message: format!("remote planner returned invalid planner JSON: {error}"),
            retryable: true,
            details: Some(serde_json::json!({ "content": content })),
        })
    }

    #[cfg(not(feature = "remote-openai"))]
    fn resolve_with_openai_planner(
        &self,
        _profile: &RemotePlannerProfile,
        _planner_input: &PlannerInput,
    ) -> Result<PlannerOutput, ToolError> {
        Err(ToolError {
            code: String::from("planner_backend_unavailable"),
            message: String::from("remote OpenAI planner support is not enabled in this build"),
            retryable: false,
            details: None,
        })
    }

    #[cfg(feature = "remote-openai")]
    fn resolve_with_ollama_planner(
        &self,
        profile: &RemotePlannerProfile,
        planner_input: &PlannerInput,
    ) -> Result<PlannerOutput, ToolError> {
        use async_openai::types::chat::ResponseFormat;
        use async_openai::types::chat::{
            ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
            CreateChatCompletionRequestArgs,
        };
        use async_openai::{config::OpenAIConfig, Client};

        let api_key = resolve_secret_ref(&profile.api_key).map_err(|reason| ToolError {
            code: String::from("planner_secret_unavailable"),
            message: String::from("Ollama planner API key placeholder could not be resolved"),
            retryable: false,
            details: Some(serde_json::json!({ "reason": reason })),
        })?;

        let client = Client::with_config(
            OpenAIConfig::new()
                .with_api_base(profile.base_url.clone())
                .with_api_key(api_key),
        );
        let prompt_payload = self.planner_prompt_payload(planner_input);
        let user_content =
            serde_json::to_string_pretty(&prompt_payload).expect("planner prompt should serialize");
        let request = CreateChatCompletionRequestArgs::default()
            .model(profile.model.clone())
            .temperature(profile.temperature_milli as f32 / 1_000.0)
            .max_tokens(profile.max_output_tokens)
            .response_format(ResponseFormat::JsonObject)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(planner_system_prompt())
                    .build()
                    .map_err(|error| ToolError {
                        code: String::from("planner_request_build_failed"),
                        message: format!(
                            "failed to build planner system message for Ollama resolution: {error}"
                        ),
                        retryable: false,
                        details: None,
                    })?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(user_content)
                    .build()
                    .map_err(|error| ToolError {
                        code: String::from("planner_request_build_failed"),
                        message: format!(
                            "failed to build planner user message for Ollama resolution: {error}"
                        ),
                        retryable: false,
                        details: None,
                    })?
                    .into(),
            ])
            .build()
            .map_err(|error| ToolError {
                code: String::from("planner_request_build_failed"),
                message: format!("failed to build Ollama planner request: {error}"),
                retryable: false,
                details: None,
            })?;

        let response =
            futures::executor::block_on(client.chat().create(request)).map_err(|error| {
                ToolError {
                    code: String::from("planner_request_failed"),
                    message: format!("Ollama planner request failed: {error}"),
                    retryable: true,
                    details: Some(serde_json::json!({
                        "provider": "Ollama",
                        "model": profile.model,
                        "base_url": profile.base_url,
                    })),
                }
            })?;
        let content = response
            .choices
            .into_iter()
            .next()
            .and_then(|choice| choice.message.content)
            .ok_or_else(|| ToolError {
                code: String::from("planner_response_missing"),
                message: String::from("Ollama planner returned no structured content"),
                retryable: true,
                details: None,
            })?;

        serde_json::from_str::<PlannerOutput>(&content).map_err(|error| ToolError {
            code: String::from("planner_response_invalid"),
            message: format!("Ollama planner returned invalid planner JSON: {error}"),
            retryable: true,
            details: Some(serde_json::json!({ "content": content })),
        })
    }

    #[cfg(not(feature = "remote-openai"))]
    fn resolve_with_ollama_planner(
        &self,
        _profile: &RemotePlannerProfile,
        _planner_input: &PlannerInput,
    ) -> Result<PlannerOutput, ToolError> {
        Err(ToolError {
            code: String::from("planner_backend_unavailable"),
            message: String::from("remote Ollama planner support is not enabled in this build"),
            retryable: false,
            details: None,
        })
    }

    fn audio_tool_failure<T>(
        &self,
        tool_name: ToolName,
        request_id: String,
        message: String,
        error: ConfigError,
    ) -> ToolResult<T> {
        ToolResult::failure(
            tool_name,
            request_id,
            ToolError {
                code: String::from("config_update_failed"),
                message,
                retryable: false,
                details: Some(serde_json::json!({ "reason": error.to_string() })),
            },
            vec![String::from(
                "Audio setting update did not complete successfully.",
            )],
        )
    }

    fn browser_runtime_missing_page<T>(tool_name: ToolName, request_id: String) -> ToolResult<T> {
        ToolResult::failure(
            tool_name,
            request_id,
            ToolError {
                code: String::from("no_active_page"),
                message: String::from("browser tool requires an active page in runtime state"),
                retryable: false,
                details: None,
            },
            vec![String::from(
                "Browser action could not run because no page has been opened yet.",
            )],
        )
    }

    fn browser_tool_failure<T>(
        &self,
        tool_name: ToolName,
        request_id: String,
        message: String,
        error: BrowserError,
    ) -> ToolResult<T> {
        ToolResult::failure(
            tool_name,
            request_id,
            browser_error_to_tool_error(message, error),
            vec![String::from(
                "Browser backend action did not complete successfully.",
            )],
        )
    }

    fn next_confirmation_id(&self, request_id: &str) -> String {
        let timestamp_ms = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(_) => 0,
        };
        format!("confirm-{request_id}-{timestamp_ms}")
    }

    fn next_page_id(&self, request_id: &str) -> String {
        let timestamp_ms = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(_) => 0,
        };
        format!("page-{request_id}-{timestamp_ms}")
    }
}

impl DeterministicToolExecutor for AppCore {
    fn execute_open_url(&mut self, input: OpenUrlInput) -> ToolResult<OpenUrlData> {
        AppCore::execute_open_url(self, input)
    }

    fn execute_read_region(&mut self, input: ReadRegionInput) -> ToolResult<ReadRegionData> {
        AppCore::execute_read_region(self, input)
    }

    fn execute_read_next_region(
        &mut self,
        input: ReadNextRegionInput,
    ) -> ToolResult<ReadNextRegionData> {
        AppCore::execute_read_next_region(self, input)
    }

    fn execute_read_previous_region(
        &mut self,
        input: ReadPreviousRegionInput,
    ) -> ToolResult<ReadPreviousRegionData> {
        AppCore::execute_read_previous_region(self, input)
    }

    fn execute_stop_speaking(&mut self, input: StopSpeakingInput) -> ToolResult<StopSpeakingData> {
        AppCore::execute_stop_speaking(self, input)
    }

    fn execute_start_listening(
        &mut self,
        input: StartListeningInput,
    ) -> ToolResult<StartListeningData> {
        AppCore::execute_start_listening(self, input)
    }

    fn execute_stop_listening(
        &mut self,
        input: StopListeningInput,
    ) -> ToolResult<StopListeningData> {
        AppCore::execute_stop_listening(self, input)
    }

    fn execute_transcribe_command(
        &mut self,
        input: TranscribeCommandInput,
    ) -> ToolResult<TranscribeCommandData> {
        AppCore::execute_transcribe_command(self, input)
    }

    fn execute_go_back(&mut self, input: GoBackInput) -> ToolResult<GoBackData> {
        AppCore::execute_go_back(self, input)
    }

    fn execute_go_forward(&mut self, input: GoForwardInput) -> ToolResult<GoForwardData> {
        AppCore::execute_go_forward(self, input)
    }

    fn execute_reload_page(&mut self, input: ReloadPageInput) -> ToolResult<ReloadPageData> {
        AppCore::execute_reload_page(self, input)
    }

    fn execute_scroll_page(&mut self, input: ScrollPageInput) -> ToolResult<ScrollPageData> {
        AppCore::execute_scroll_page(self, input)
    }

    fn execute_list_interactive_elements(
        &mut self,
        input: ListInteractiveElementsInput,
    ) -> ToolResult<ListInteractiveElementsData> {
        AppCore::execute_list_interactive_elements(self, input)
    }

    fn execute_find_element(&mut self, input: FindElementInput) -> ToolResult<FindElementData> {
        AppCore::execute_find_element(self, input)
    }

    fn execute_click_element(&mut self, input: ClickElementInput) -> ToolResult<ClickElementData> {
        AppCore::execute_click_element(self, input)
    }

    fn execute_extract_page_model(
        &mut self,
        input: ExtractPageModelInput,
    ) -> ToolResult<ExtractPageModelData> {
        AppCore::execute_extract_page_model(self, input)
    }

    fn execute_get_page_snapshot(
        &mut self,
        input: GetPageSnapshotInput,
    ) -> ToolResult<PageSnapshotData> {
        AppCore::execute_get_page_snapshot(self, input)
    }

    fn execute_set_tts_voice(&mut self, input: SetTtsVoiceInput) -> ToolResult<SetTtsVoiceData> {
        AppCore::execute_set_tts_voice(self, input)
    }

    fn execute_set_playback_volume(
        &mut self,
        input: SetPlaybackVolumeInput,
    ) -> ToolResult<SetPlaybackVolumeData> {
        AppCore::execute_set_playback_volume(self, input)
    }

    fn execute_set_playback_speed(
        &mut self,
        input: SetPlaybackSpeedInput,
    ) -> ToolResult<SetPlaybackSpeedData> {
        AppCore::execute_set_playback_speed(self, input)
    }

    fn execute_set_browser_visibility(
        &mut self,
        input: SetBrowserVisibilityInput,
    ) -> ToolResult<SetBrowserVisibilityData> {
        AppCore::execute_set_browser_visibility(self, input)
    }

    fn execute_get_agent_state(&mut self, input: GetAgentStateInput) -> ToolResult<AgentStateData> {
        AppCore::execute_get_agent_state(self, input)
    }

    fn execute_get_runtime_status(
        &mut self,
        input: GetRuntimeStatusInput,
    ) -> ToolResult<GetRuntimeStatusData> {
        AppCore::execute_get_runtime_status(self, input)
    }

    fn execute_confirm_action(
        &mut self,
        input: ConfirmActionInput,
    ) -> ToolResult<ConfirmActionData> {
        AppCore::execute_confirm_action(self, input)
    }

    fn execute_report_result(&mut self, input: ReportResultInput) -> ToolResult<ReportResultData> {
        AppCore::execute_report_result(self, input)
    }
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn tts_runtime_error_to_tool_error(error: TtsRuntimeError) -> ToolError {
    match error {
        TtsRuntimeError::EmptyNarrationText => ToolError {
            code: String::from("empty_narration_text"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        TtsRuntimeError::MissingLocalProfile
        | TtsRuntimeError::MissingLocalProfileDefinition { .. }
        | TtsRuntimeError::MissingRemoteProfile
        | TtsRuntimeError::MissingRemoteProfileDefinition { .. } => ToolError {
            code: String::from("tts_profile_unavailable"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        TtsRuntimeError::UnsupportedRemoteProvider { .. } => ToolError {
            code: String::from("unsupported_tts_provider"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        TtsRuntimeError::UnsupportedLocalBackend { .. } => ToolError {
            code: String::from("unsupported_tts_backend"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        TtsRuntimeError::LocalTtsFeatureUnavailable
        | TtsRuntimeError::RemoteTtsFeatureUnavailable => ToolError {
            code: String::from("tts_backend_unavailable"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        TtsRuntimeError::EmptyRemoteVoice => ToolError {
            code: String::from("tts_voice_unavailable"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        TtsRuntimeError::RemoteSecretUnavailable { .. } => ToolError {
            code: String::from("tts_secret_unavailable"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        TtsRuntimeError::RemoteRequestBuildFailed { .. } => ToolError {
            code: String::from("tts_request_build_failed"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        TtsRuntimeError::RemoteRequestFailed { .. } => ToolError {
            code: String::from("tts_request_failed"),
            message: error.to_string(),
            retryable: true,
            details: None,
        },
        TtsRuntimeError::UnsupportedRemoteAudioFormat { .. } => ToolError {
            code: String::from("unsupported_tts_audio_format"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        TtsRuntimeError::RemoteResponseDecodeFailed { .. } => ToolError {
            code: String::from("tts_response_invalid"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        TtsRuntimeError::EmptyLocalModelPath | TtsRuntimeError::MissingLocalModelPath { .. } => {
            ToolError {
                code: String::from("tts_model_unavailable"),
                message: error.to_string(),
                retryable: false,
                details: None,
            }
        }
        TtsRuntimeError::UnsupportedLocalSampleRate { .. } => ToolError {
            code: String::from("unsupported_tts_sample_rate"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        TtsRuntimeError::LocalModelLoad { .. } => ToolError {
            code: String::from("tts_model_load_failed"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        TtsRuntimeError::SynthesisFailed { .. } => ToolError {
            code: String::from("tts_synthesis_failed"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
    }
}

fn audio_playback_error_to_tool_error(error: AudioPlaybackError) -> ToolError {
    match error {
        AudioPlaybackError::AudioFeatureUnavailable => ToolError {
            code: String::from("audio_backend_unavailable"),
            message: error.to_string(),
            retryable: false,
            details: None,
        },
        AudioPlaybackError::InvalidChannelCount | AudioPlaybackError::InvalidSampleRate => {
            ToolError {
                code: String::from("invalid_audio_output"),
                message: error.to_string(),
                retryable: false,
                details: None,
            }
        }
        AudioPlaybackError::OpenDevice { .. } => ToolError {
            code: String::from("audio_output_unavailable"),
            message: error.to_string(),
            retryable: true,
            details: None,
        },
    }
}

fn asr_runtime_error_to_tool_error(error: &AsrRuntimeError) -> ToolError {
    let code = match error {
        AsrRuntimeError::MissingLocalProfile
        | AsrRuntimeError::MissingLocalProfileDefinition { .. }
        | AsrRuntimeError::MissingRemoteProfile
        | AsrRuntimeError::MissingRemoteProfileDefinition { .. } => "asr_profile_unavailable",
        AsrRuntimeError::UnsupportedRemoteProvider { .. } => "unsupported_asr_provider",
        AsrRuntimeError::UnsupportedLocalBackend { .. } => "unsupported_asr_backend",
        AsrRuntimeError::AudioFeatureUnavailable => "audio_backend_unavailable",
        AsrRuntimeError::LocalAsrFeatureUnavailable | AsrRuntimeError::RemoteAsrFeatureUnavailable => {
            "asr_backend_unavailable"
        }
        AsrRuntimeError::MissingInputDevice => "audio_input_unavailable",
        AsrRuntimeError::InputConfigUnavailable { .. } => "audio_input_config_unavailable",
        AsrRuntimeError::UnsupportedInputSampleFormat { .. } => "unsupported_audio_input_format",
        AsrRuntimeError::BuildInputStream { .. } => "audio_input_stream_build_failed",
        AsrRuntimeError::StartInputStream { .. } => "audio_input_stream_start_failed",
        AsrRuntimeError::AudioBufferLockFailed => "audio_buffer_lock_failed",
        AsrRuntimeError::EmptyLocalModelPath | AsrRuntimeError::MissingLocalModelPath { .. } => {
            "asr_model_unavailable"
        }
        AsrRuntimeError::RemoteSecretUnavailable { .. } => "asr_secret_unavailable",
        AsrRuntimeError::RemoteAudioEncodeFailed { .. } => "invalid_audio_input",
        AsrRuntimeError::RemoteRequestBuildFailed { .. } => "asr_request_build_failed",
        AsrRuntimeError::RemoteRequestTimedOut { .. } => "asr_request_timed_out",
        AsrRuntimeError::RemoteRequestFailed { .. } => "asr_request_failed",
        AsrRuntimeError::LocalModelLoad { .. } => "asr_model_load_failed",
        AsrRuntimeError::NoAudioCaptured => "no_audio_captured",
        AsrRuntimeError::TranscriptionFailed { .. } => "asr_transcription_failed",
    };

    ToolError {
        code: String::from(code),
        message: error.to_string(),
        retryable: matches!(
            error,
            AsrRuntimeError::MissingInputDevice
                | AsrRuntimeError::InputConfigUnavailable { .. }
                | AsrRuntimeError::BuildInputStream { .. }
                | AsrRuntimeError::StartInputStream { .. }
                | AsrRuntimeError::AudioBufferLockFailed
                | AsrRuntimeError::NoAudioCaptured
                | AsrRuntimeError::RemoteRequestTimedOut { .. }
                | AsrRuntimeError::RemoteRequestFailed { .. }
        ),
        details: None,
    }
}

fn resolve_secret_ref(secret_ref: &SecretRef) -> Result<String, String> {
    match secret_ref {
        SecretRef::FromEnv { from_env } => std::env::var(from_env)
            .map(|value| value.trim().to_string())
            .map_err(|error| format!("failed to read environment variable '{from_env}': {error}")),
        SecretRef::FromFile { from_file } => fs::read_to_string(from_file)
            .map(|value| value.trim().to_string())
            .map_err(|error| format!("failed to read secret file '{from_file}': {error}")),
        SecretRef::Inline { inline } => Ok(inline.trim().to_string()),
    }
    .and_then(|value| {
        if value.is_empty() {
            Err(String::from("resolved secret value was empty"))
        } else {
            Ok(value)
        }
    })
}

fn planner_system_prompt() -> &'static str {
    "You are the bounded planner for blind_browser, a voice-first desktop browser for vision-impaired users.
Return only JSON that matches the provided planner_output_schema.
Use only tool names that appear in planner_input.available_tools and only selected_skills that appear in planner_input.active_skill_names.
Every step arguments object must match the corresponding tool_input_schemas entry exactly, including snake_case field names.
Keep plans linear and short: at most five steps, with at most one NextStep edge from any step.
Use NeedsConfirmation plus a confirm_action step when the request is risky or ambiguous before side effects.
Use Blocked only when the request cannot be grounded safely or is outside the supported tool set.
Do not invent tools, skills, statuses, transition kinds, or argument fields."
}

fn normalize_absolute_url(url: &str) -> Result<String, ToolError> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_url"),
            message: String::from("open_url requires a non-empty absolute URL"),
            retryable: false,
            details: None,
        });
    }

    let Some(separator_index) = trimmed.find(':') else {
        return Err(ToolError {
            code: String::from("invalid_url"),
            message: String::from("open_url requires an absolute URL with a scheme"),
            retryable: false,
            details: Some(serde_json::json!({ "url": trimmed })),
        });
    };

    let scheme = &trimmed[..separator_index];
    let remainder = &trimmed[separator_index + 1..];
    let valid_scheme = scheme.chars().enumerate().all(|(index, ch)| match index {
        0 => ch.is_ascii_alphabetic(),
        _ => ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'),
    });

    if !valid_scheme || remainder.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_url"),
            message: String::from("open_url requires an absolute URL with a valid scheme"),
            retryable: false,
            details: Some(serde_json::json!({ "url": trimmed })),
        });
    }

    Ok(trimmed.to_string())
}

fn build_visible_text_excerpt(page: &PageModel, max_chars: Option<usize>) -> String {
    let joined_text = page
        .regions
        .iter()
        .map(|region| region.text.trim())
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");

    match max_chars {
        Some(limit) => joined_text.chars().take(limit).collect(),
        None => joined_text,
    }
}

fn build_extracted_page_model(page: &PageModel, input: &ExtractPageModelInput) -> PageModel {
    let interactive_elements = if input.include_links {
        page.interactive_elements.clone()
    } else {
        page.interactive_elements
            .iter()
            .filter(|element| element.role != ElementRole::Link)
            .cloned()
            .collect()
    };

    PageModel {
        title: page.title.clone(),
        url: page.url.clone(),
        regions: page.regions.clone(),
        interactive_elements,
    }
}

fn filter_interactive_elements(
    interactive_elements: &[crate::page_model::InteractiveElement],
    visible_only: bool,
    roles: Option<&[ElementRole]>,
) -> Vec<crate::page_model::InteractiveElement> {
    interactive_elements
        .iter()
        .filter(|element| !visible_only || element.visible)
        .filter(|element| roles.is_none_or(|roles| roles.contains(&element.role)))
        .cloned()
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FindElementQuery {
    summary: String,
    description: Option<String>,
    text: Option<String>,
    role: Option<ElementRole>,
    color_hint: Option<String>,
    nearby_text: Option<String>,
    selector_hint: Option<String>,
}

fn build_find_element_query(input: &FindElementInput) -> Result<FindElementQuery, ToolError> {
    let description = normalize_optional_text(Some(input.description.clone()));
    let text = normalize_optional_text(input.text.clone());
    let color_hint = normalize_optional_text(input.color_hint.clone());
    let nearby_text = normalize_optional_text(input.nearby_text.clone());
    let selector_hint = normalize_optional_text(input.selector_hint.clone());

    if description.is_none()
        && text.is_none()
        && input.role.is_none()
        && color_hint.is_none()
        && nearby_text.is_none()
        && selector_hint.is_none()
    {
        return Err(ToolError {
            code: String::from("invalid_find_query"),
            message: String::from("find_element requires at least one populated search field"),
            retryable: false,
            details: None,
        });
    }

    let mut summary_parts = Vec::new();
    if let Some(description) = description.as_ref() {
        summary_parts.push(format!("description={description}"));
    }
    if let Some(text) = text.as_ref() {
        summary_parts.push(format!("text={text}"));
    }
    if let Some(role) = input.role.as_ref() {
        summary_parts.push(format!("role={role:?}"));
    }
    if let Some(color_hint) = color_hint.as_ref() {
        summary_parts.push(format!("color_hint={color_hint}"));
    }
    if let Some(nearby_text) = nearby_text.as_ref() {
        summary_parts.push(format!("nearby_text={nearby_text}"));
    }
    if let Some(selector_hint) = selector_hint.as_ref() {
        summary_parts.push(format!("selector_hint={selector_hint}"));
    }

    Ok(FindElementQuery {
        summary: summary_parts.join("; "),
        description,
        text,
        role: input.role.clone(),
        color_hint,
        nearby_text,
        selector_hint,
    })
}

fn rank_find_element_candidates(
    elements: &[crate::page_model::InteractiveElement],
    query: &FindElementQuery,
    candidate_limit: usize,
) -> Vec<crate::commands::ElementCandidate> {
    let mut candidates = elements
        .iter()
        .filter_map(|element| score_interactive_element(element, query))
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| {
        right
            .confidence_bps
            .cmp(&left.confidence_bps)
            .then_with(|| left.element_id.cmp(&right.element_id))
    });
    candidates.truncate(candidate_limit);
    candidates
}

fn determine_find_element_resolution(
    candidates: &[crate::commands::ElementCandidate],
) -> (Option<String>, Option<f32>, bool) {
    let Some(top_candidate) = candidates.first() else {
        return (None, None, false);
    };

    let top_confidence = Some(f32::from(top_candidate.confidence_bps) / 10_000.0);
    let ambiguous = candidates.get(1).is_some_and(|second_candidate| {
        top_candidate.confidence_bps < FIND_ELEMENT_STRONG_MATCH_BPS
            || top_candidate
                .confidence_bps
                .saturating_sub(second_candidate.confidence_bps)
                <= FIND_ELEMENT_AMBIGUITY_MARGIN_BPS
    });

    if ambiguous {
        (None, top_confidence, true)
    } else {
        (
            Some(top_candidate.element_id.clone()),
            top_confidence,
            false,
        )
    }
}

fn resolve_clickable_element<'a>(
    page: &'a PageModel,
    element_id: &str,
) -> Result<&'a crate::page_model::InteractiveElement, ToolError> {
    let normalized_element_id = element_id.trim();
    if normalized_element_id.is_empty() {
        return Err(ToolError {
            code: String::from("invalid_element_id"),
            message: String::from("click_element requires a non-empty deterministic element_id"),
            retryable: false,
            details: None,
        });
    }

    let Some(element) = page
        .interactive_elements
        .iter()
        .find(|element| element.element_id == normalized_element_id)
    else {
        return Err(ToolError {
            code: String::from("unknown_element_id"),
            message: String::from(
                "click_element requires an element_id that exists in the current page model",
            ),
            retryable: false,
            details: Some(serde_json::json!({ "element_id": normalized_element_id })),
        });
    };

    if !element.visible {
        return Err(ToolError {
            code: String::from("element_not_visible"),
            message: String::from("click_element cannot act on an element marked not visible"),
            retryable: false,
            details: Some(serde_json::json!({ "element_id": normalized_element_id })),
        });
    }

    if !element.enabled {
        return Err(ToolError {
            code: String::from("element_disabled"),
            message: String::from("click_element cannot act on a disabled element"),
            retryable: false,
            details: Some(serde_json::json!({ "element_id": normalized_element_id })),
        });
    }

    if element
        .dom_locator
        .as_deref()
        .map(str::trim)
        .is_none_or(str::is_empty)
    {
        return Err(ToolError {
            code: String::from("missing_dom_locator"),
            message: String::from(
                "click_element requires the current page model to carry a stable dom_locator",
            ),
            retryable: false,
            details: Some(serde_json::json!({ "element_id": normalized_element_id })),
        });
    }

    Ok(element)
}

#[derive(Debug, Default)]
struct FindElementScore {
    score_bps: u16,
    matched_on: Vec<String>,
    rationale_codes: Vec<String>,
}

impl FindElementScore {
    fn push_match(&mut self, match_label: &str, rationale_code: impl Into<String>, score_bps: u16) {
        self.score_bps = self.score_bps.saturating_add(score_bps);
        self.matched_on.push(match_label.to_string());
        self.rationale_codes.push(rationale_code.into());
    }
}

struct AttributeHintSpec<'a> {
    match_label: &'a str,
    exact_score_bps: u16,
    contains_score_bps: u16,
}

fn score_interactive_element(
    element: &crate::page_model::InteractiveElement,
    query: &FindElementQuery,
) -> Option<crate::commands::ElementCandidate> {
    let mut score = FindElementScore::default();

    if let Some(role) = query.role.as_ref() {
        if &element.role == role {
            score.push_match("role", "role_match", 1_800);
        } else {
            return None;
        }
    }

    if let Some(description) = query.description.as_ref() {
        let field_match =
            score_text_query_against_element(description, element, "description", &mut score);
        if !field_match && query.role.is_none() {
            return None;
        }
    }

    if let Some(text) = query.text.as_ref() {
        let field_match = score_text_query_against_element(text, element, "text", &mut score);
        if !field_match && query.description.is_none() && query.role.is_none() {
            return None;
        }
    }

    if let Some(nearby_text) = query.nearby_text.as_ref() {
        score_attribute_hint(
            nearby_text,
            element,
            AttributeHintSpec {
                match_label: "nearby_text",
                exact_score_bps: 1_600,
                contains_score_bps: 900,
            },
            &mut score,
        );
    }

    if let Some(selector_hint) = query.selector_hint.as_ref() {
        score_attribute_hint(
            selector_hint,
            element,
            AttributeHintSpec {
                match_label: "selector_hint",
                exact_score_bps: 1_500,
                contains_score_bps: 800,
            },
            &mut score,
        );
    }

    if let Some(color_hint) = query.color_hint.as_ref() {
        score_attribute_hint(
            color_hint,
            element,
            AttributeHintSpec {
                match_label: "color_hint",
                exact_score_bps: 500,
                contains_score_bps: 250,
            },
            &mut score,
        );
    }

    if score.score_bps == 0 {
        return None;
    }

    if element.enabled {
        score.score_bps = score.score_bps.saturating_add(100);
    } else {
        score.rationale_codes.push(String::from("disabled_penalty"));
        score.score_bps = score.score_bps.saturating_sub(300);
    }

    Some(crate::commands::ElementCandidate {
        element_id: element.element_id.clone(),
        confidence_bps: score.score_bps.min(10_000),
        matched_on: score.matched_on,
        rationale_codes: score.rationale_codes,
    })
}

fn score_text_query_against_element(
    query_text: &str,
    element: &crate::page_model::InteractiveElement,
    match_label: &str,
    score: &mut FindElementScore,
) -> bool {
    let normalized_query = normalize_search_text(query_text);
    let accessible_name = element
        .accessible_name
        .as_deref()
        .map(normalize_search_text);
    let visible_text = element.text.as_deref().map(normalize_search_text);
    let placeholder = element.placeholder.as_deref().map(normalize_search_text);

    if accessible_name.as_deref() == Some(normalized_query.as_str()) {
        score.push_match(match_label, "accessible_name_exact", 4_200);
        return true;
    }
    if visible_text.as_deref() == Some(normalized_query.as_str()) {
        score.push_match(match_label, "visible_text_exact", 4_000);
        return true;
    }
    if placeholder.as_deref() == Some(normalized_query.as_str()) {
        score.push_match(match_label, "placeholder_exact", 3_400);
        return true;
    }

    let overlap_score = text_overlap_score(&normalized_query, element);
    if overlap_score > 0 {
        score.push_match(match_label, "lexical_overlap", overlap_score);
        return true;
    }

    false
}

fn score_attribute_hint(
    hint: &str,
    element: &crate::page_model::InteractiveElement,
    spec: AttributeHintSpec<'_>,
    score: &mut FindElementScore,
) -> bool {
    let normalized_hint = normalize_search_text(hint);
    let attribute_blob = element
        .attributes
        .iter()
        .map(|(key, value)| format!("{key} {value}"))
        .collect::<Vec<_>>()
        .join(" ");
    let normalized_attributes = normalize_search_text(&attribute_blob);

    if normalized_attributes.is_empty() {
        return false;
    }

    if normalized_attributes == normalized_hint {
        score.push_match(
            spec.match_label,
            format!("{}_exact", spec.match_label),
            spec.exact_score_bps,
        );
        true
    } else if normalized_attributes.contains(&normalized_hint) {
        score.push_match(
            spec.match_label,
            format!("{}_contains", spec.match_label),
            spec.contains_score_bps,
        );
        true
    } else {
        false
    }
}

fn text_overlap_score(query_text: &str, element: &crate::page_model::InteractiveElement) -> u16 {
    let query_terms = tokenize_search_text(query_text);
    if query_terms.is_empty() {
        return 0;
    }

    let element_blob = [
        element.accessible_name.as_deref().unwrap_or_default(),
        element.text.as_deref().unwrap_or_default(),
        element.placeholder.as_deref().unwrap_or_default(),
        element.href.as_deref().unwrap_or_default(),
        element.value.as_deref().unwrap_or_default(),
    ]
    .join(" ");
    let element_terms = tokenize_search_text(&element_blob);
    if element_terms.is_empty() {
        return 0;
    }

    let overlap = query_terms
        .iter()
        .filter(|term| element_terms.contains(*term))
        .count();
    if overlap == 0 {
        0
    } else {
        let ratio = overlap as f32 / query_terms.len() as f32;
        (900.0 + (ratio * 2_100.0)).round() as u16
    }
}

fn normalize_search_text(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn tokenize_search_text(value: &str) -> Vec<String> {
    normalize_search_text(value)
        .split(' ')
        .filter(|term| !term.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn infer_extraction_source(page: &PageModel, use_dom_extraction: bool) -> ExtractionSource {
    let _ = use_dom_extraction;
    let has_ocr = page
        .regions
        .iter()
        .any(|region| matches!(region.source, RegionSource::Ocr));
    let has_dom_like = page
        .regions
        .iter()
        .any(|region| matches!(region.source, RegionSource::Dom | RegionSource::Mixed));

    if has_ocr && has_dom_like {
        ExtractionSource::Merged
    } else if has_ocr {
        ExtractionSource::Ocr
    } else {
        ExtractionSource::DomFallback
    }
}

fn browser_error_to_tool_error(message: String, error: BrowserError) -> ToolError {
    let code = match &error {
        BrowserError::FeatureDisabled => "browser_feature_disabled",
        BrowserError::Launch(_) => "browser_launch_failed",
        BrowserError::CreatePage(_) => "browser_page_creation_failed",
        BrowserError::Navigate(_) => "browser_navigation_failed",
        BrowserError::Inspect(_) => "browser_state_read_failed",
        BrowserError::NoActivePage => "browser_no_active_page",
        BrowserError::MissingDomLocator { .. } => "missing_dom_locator",
        BrowserError::Resolve(_) => "browser_element_resolution_failed",
        BrowserError::ElementNotFound { .. } => "browser_element_not_found",
        BrowserError::Click(_) => "browser_click_failed",
        BrowserError::History(_) => "browser_history_failed",
        BrowserError::Reload(_) => "browser_reload_failed",
        BrowserError::Scroll(_) => "browser_scroll_failed",
    };

    ToolError {
        code: String::from(code),
        message,
        retryable: matches!(
            error,
            BrowserError::Launch(_)
                | BrowserError::CreatePage(_)
                | BrowserError::Navigate(_)
                | BrowserError::Inspect(_)
                | BrowserError::History(_)
                | BrowserError::Reload(_)
                | BrowserError::Scroll(_)
        ),
        details: Some(serde_json::json!({ "reason": error.to_string() })),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_extracted_page_model, build_find_element_query, build_visible_text_excerpt,
        determine_find_element_resolution, filter_interactive_elements, infer_extraction_source,
        normalize_absolute_url, normalize_optional_text, rank_find_element_candidates,
        resolve_clickable_element,
    };
    use crate::commands::{ExtractPageModelInput, FindElementInput};
    use crate::page_model::{
        ElementRole, ExtractionSource, InteractiveElement, PageModel, PageRegion, RegionSource,
    };

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
    fn build_visible_text_excerpt_joins_regions_and_applies_limit() {
        let page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: vec![
                PageRegion {
                    region_id: String::from("region-1"),
                    label: None,
                    text: String::from("First paragraph"),
                    source: RegionSource::Dom,
                },
                PageRegion {
                    region_id: String::from("region-2"),
                    label: None,
                    text: String::from("Second paragraph"),
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
    fn infer_extraction_source_detects_merged_models() {
        let page = PageModel {
            title: Some(String::from("Example")),
            url: Some(String::from("https://example.com")),
            regions: vec![
                PageRegion {
                    region_id: String::from("dom-region"),
                    label: None,
                    text: String::from("DOM text"),
                    source: RegionSource::Dom,
                },
                PageRegion {
                    region_id: String::from("ocr-region"),
                    label: None,
                    text: String::from("OCR text"),
                    source: RegionSource::Ocr,
                },
            ],
            interactive_elements: Vec::new(),
        };

        assert_eq!(
            infer_extraction_source(&page, true),
            ExtractionSource::Merged
        );
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
            visible_only: true,
            max_candidates: Some(3),
        })
        .expect("query should be valid");

        let candidates = rank_find_element_candidates(&elements, &query, 3);

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].element_id, "button-1");
        assert!(candidates[0].confidence_bps > candidates[1].confidence_bps);
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
            determine_find_element_resolution(&candidates);

        assert_eq!(chosen_element_id, None);
        assert_eq!(chosen_confidence, Some(0.89));
        assert!(requires_confirmation);
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
}
