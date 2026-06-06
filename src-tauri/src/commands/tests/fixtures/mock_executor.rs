use super::*;

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
