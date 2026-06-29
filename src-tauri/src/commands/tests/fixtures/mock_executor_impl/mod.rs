// Re-export the fixtures scope so submodules can reach all tool types via `use super::*;`.
pub(crate) use super::*;

mod interaction;
mod media;
mod navigation;
mod settings;
mod state;

impl DeterministicToolExecutor for MockExecutor {
    fn execute_open_url(&mut self, input: OpenUrlInput) -> ToolResult<OpenUrlData> {
        navigation::execute_open_url(self, input)
    }

    fn execute_go_back(&mut self, input: GoBackInput) -> ToolResult<GoBackData> {
        navigation::execute_go_back(self, input)
    }

    fn execute_go_forward(&mut self, input: GoForwardInput) -> ToolResult<GoForwardData> {
        navigation::execute_go_forward(self, input)
    }

    fn execute_reload_page(&mut self, input: ReloadPageInput) -> ToolResult<ReloadPageData> {
        navigation::execute_reload_page(self, input)
    }

    fn execute_get_html(&mut self, input: GetHtmlInput) -> ToolResult<GetHtmlData> {
        navigation::execute_get_html(self, input)
    }

    fn execute_eval_js(&mut self, input: EvalJsInput) -> ToolResult<EvalJsData> {
        navigation::execute_eval_js(self, input)
    }

    fn execute_scroll_page(&mut self, input: ScrollPageInput) -> ToolResult<ScrollPageData> {
        navigation::execute_scroll_page(self, input)
    }

    fn execute_capture_screenshot(
        &mut self,
        input: CaptureScreenshotInput,
    ) -> ToolResult<CaptureScreenshotData> {
        media::execute_capture_screenshot(self, input)
    }

    fn execute_run_ocr(&mut self, input: RunOcrInput) -> ToolResult<RunOcrData> {
        media::execute_run_ocr(self, input)
    }

    fn execute_merge_ocr_into_page_model(
        &mut self,
        input: MergeOcrIntoPageModelInput,
    ) -> ToolResult<MergeOcrIntoPageModelData> {
        media::execute_merge_ocr_into_page_model(self, input)
    }

    fn execute_read_region(&mut self, input: ReadRegionInput) -> ToolResult<ReadRegionData> {
        media::execute_read_region(self, input)
    }

    fn execute_read_next_region(
        &mut self,
        input: ReadNextRegionInput,
    ) -> ToolResult<ReadNextRegionData> {
        media::execute_read_next_region(self, input)
    }

    fn execute_read_previous_region(
        &mut self,
        input: ReadPreviousRegionInput,
    ) -> ToolResult<ReadPreviousRegionData> {
        media::execute_read_previous_region(self, input)
    }

    fn execute_stop_speaking(&mut self, input: StopSpeakingInput) -> ToolResult<StopSpeakingData> {
        media::execute_stop_speaking(self, input)
    }

    fn execute_start_listening(
        &mut self,
        input: StartListeningInput,
    ) -> ToolResult<StartListeningData> {
        interaction::execute_start_listening(self, input)
    }

    fn execute_stop_listening(
        &mut self,
        input: StopListeningInput,
    ) -> ToolResult<StopListeningData> {
        interaction::execute_stop_listening(self, input)
    }

    fn execute_transcribe_command(
        &mut self,
        input: TranscribeCommandInput,
    ) -> ToolResult<TranscribeCommandData> {
        interaction::execute_transcribe_command(self, input)
    }

    fn execute_get_page_snapshot(
        &mut self,
        input: GetPageSnapshotInput,
    ) -> ToolResult<PageSnapshotData> {
        interaction::execute_get_page_snapshot(self, input)
    }

    fn execute_list_interactive_elements(
        &mut self,
        input: ListInteractiveElementsInput,
    ) -> ToolResult<ListInteractiveElementsData> {
        interaction::execute_list_interactive_elements(self, input)
    }

    fn execute_find_element(&mut self, input: FindElementInput) -> ToolResult<FindElementData> {
        interaction::execute_find_element(self, input)
    }

    fn execute_click_element(&mut self, input: ClickElementInput) -> ToolResult<ClickElementData> {
        interaction::execute_click_element(self, input)
    }

    fn execute_focus_element(&mut self, input: FocusElementInput) -> ToolResult<FocusElementData> {
        interaction::execute_focus_element(self, input)
    }

    fn execute_type_into_element(
        &mut self,
        input: TypeIntoElementInput,
    ) -> ToolResult<TypeIntoElementData> {
        interaction::execute_type_into_element(self, input)
    }

    fn execute_submit_active_form(
        &mut self,
        input: SubmitActiveFormInput,
    ) -> ToolResult<SubmitActiveFormData> {
        interaction::execute_submit_active_form(self, input)
    }

    fn execute_extract_page_model(
        &mut self,
        input: ExtractPageModelInput,
    ) -> ToolResult<ExtractPageModelData> {
        interaction::execute_extract_page_model(self, input)
    }

    fn execute_set_tts_voice(&mut self, input: SetTtsVoiceInput) -> ToolResult<SetTtsVoiceData> {
        settings::execute_set_tts_voice(self, input)
    }

    fn execute_set_playback_volume(
        &mut self,
        input: SetPlaybackVolumeInput,
    ) -> ToolResult<SetPlaybackVolumeData> {
        settings::execute_set_playback_volume(self, input)
    }

    fn execute_set_playback_speed(
        &mut self,
        input: SetPlaybackSpeedInput,
    ) -> ToolResult<SetPlaybackSpeedData> {
        settings::execute_set_playback_speed(self, input)
    }

    fn execute_set_browser_visibility(
        &mut self,
        input: SetBrowserVisibilityInput,
    ) -> ToolResult<SetBrowserVisibilityData> {
        settings::execute_set_browser_visibility(self, input)
    }

    fn execute_get_agent_state(&mut self, input: GetAgentStateInput) -> ToolResult<AgentStateData> {
        state::execute_get_agent_state(self, input)
    }

    fn execute_get_runtime_status(
        &mut self,
        input: GetRuntimeStatusInput,
    ) -> ToolResult<GetRuntimeStatusData> {
        state::execute_get_runtime_status(self, input)
    }

    fn execute_confirm_action(
        &mut self,
        input: ConfirmActionInput,
    ) -> ToolResult<ConfirmActionData> {
        state::execute_confirm_action(self, input)
    }

    fn execute_report_result(&mut self, input: ReportResultInput) -> ToolResult<ReportResultData> {
        state::execute_report_result(self, input)
    }
}
