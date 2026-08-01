use crate::commands::{
    AgentStateData, CaptureScreenshotData, CaptureScreenshotInput, ClickElementData,
    ClickElementInput, ConfirmActionData, ConfirmActionInput, ConfirmationRuntimeContext,
    DeterministicToolExecutor, EvalJsData, EvalJsInput, ExtractPageModelData,
    ExtractPageModelInput, FindElementData, FindElementInput, FocusElementData,
    FocusElementInput, GetAgentStateInput, GetHtmlData, GetHtmlInput, GetPageSnapshotInput,
    GetRuntimeStatusData, GetRuntimeStatusInput, GoBackData, GoBackInput, GoForwardData,
    GoForwardInput, ListInteractiveElementsData, ListInteractiveElementsInput,
    MergeOcrIntoPageModelData, MergeOcrIntoPageModelInput, OpenUrlData, OpenUrlInput,
    PageSnapshotData, ReadNextRegionData, ReadNextRegionInput, ReadPreviousRegionData,
    ReadPreviousRegionInput, ReadRegionData, ReadRegionInput, ReloadPageData, ReloadPageInput,
    ReportResultData, ReportResultInput, RunOcrData, RunOcrInput, ScrollPageData,
    ScrollPageInput, SetBrowserVisibilityData, SetBrowserVisibilityInput, SetPlaybackSpeedData,
    SetPlaybackSpeedInput, SetPlaybackVolumeData, SetPlaybackVolumeInput, SetTtsVoiceData,
    SetTtsVoiceInput, StartListeningData, StartListeningInput, StopListeningData,
    StopListeningInput, StopSpeakingData, StopSpeakingInput, SubmitActiveFormData,
    SubmitActiveFormInput, ToolResult, TranscribeCommandData, TranscribeCommandInput,
    TypeIntoElementData, TypeIntoElementInput,
};

impl DeterministicToolExecutor for super::AppCore {
    fn confirmation_runtime_context(&self) -> ConfirmationRuntimeContext {
        ConfirmationRuntimeContext::current(
            self.state.current_page_id.clone(),
            self.state
                .current_page
                .as_ref()
                .and_then(|page| page.url.clone()),
        )
    }

    fn execute_open_url(&mut self, input: OpenUrlInput) -> ToolResult<OpenUrlData> {
        super::AppCore::execute_open_url(self, input)
    }

    fn execute_read_region(&mut self, input: ReadRegionInput) -> ToolResult<ReadRegionData> {
        super::AppCore::execute_read_region(self, input)
    }

    fn execute_read_next_region(
        &mut self,
        input: ReadNextRegionInput,
    ) -> ToolResult<ReadNextRegionData> {
        super::AppCore::execute_read_next_region(self, input)
    }

    fn execute_read_previous_region(
        &mut self,
        input: ReadPreviousRegionInput,
    ) -> ToolResult<ReadPreviousRegionData> {
        super::AppCore::execute_read_previous_region(self, input)
    }

    fn execute_stop_speaking(&mut self, input: StopSpeakingInput) -> ToolResult<StopSpeakingData> {
        super::AppCore::execute_stop_speaking(self, input)
    }

    fn execute_start_listening(
        &mut self,
        input: StartListeningInput,
    ) -> ToolResult<StartListeningData> {
        super::AppCore::execute_start_listening(self, input)
    }

    fn execute_stop_listening(
        &mut self,
        input: StopListeningInput,
    ) -> ToolResult<StopListeningData> {
        super::AppCore::execute_stop_listening(self, input)
    }

    fn execute_transcribe_command(
        &mut self,
        input: TranscribeCommandInput,
    ) -> ToolResult<TranscribeCommandData> {
        super::AppCore::execute_transcribe_command(self, input)
    }

    fn execute_go_back(&mut self, input: GoBackInput) -> ToolResult<GoBackData> {
        super::AppCore::execute_go_back(self, input)
    }

    fn execute_go_forward(&mut self, input: GoForwardInput) -> ToolResult<GoForwardData> {
        super::AppCore::execute_go_forward(self, input)
    }

    fn execute_reload_page(&mut self, input: ReloadPageInput) -> ToolResult<ReloadPageData> {
        super::AppCore::execute_reload_page(self, input)
    }

    fn execute_get_html(&mut self, input: GetHtmlInput) -> ToolResult<GetHtmlData> {
        super::AppCore::execute_get_html(self, input)
    }

    fn execute_eval_js(&mut self, input: EvalJsInput) -> ToolResult<EvalJsData> {
        super::AppCore::execute_eval_js(self, input)
    }

    fn execute_scroll_page(&mut self, input: ScrollPageInput) -> ToolResult<ScrollPageData> {
        super::AppCore::execute_scroll_page(self, input)
    }

    fn execute_capture_screenshot(
        &mut self,
        input: CaptureScreenshotInput,
    ) -> ToolResult<CaptureScreenshotData> {
        super::AppCore::execute_capture_screenshot(self, input)
    }

    fn execute_run_ocr(&mut self, input: RunOcrInput) -> ToolResult<RunOcrData> {
        super::AppCore::execute_run_ocr(self, input)
    }

    fn execute_merge_ocr_into_page_model(
        &mut self,
        input: MergeOcrIntoPageModelInput,
    ) -> ToolResult<MergeOcrIntoPageModelData> {
        super::AppCore::execute_merge_ocr_into_page_model(self, input)
    }

    fn execute_list_interactive_elements(
        &mut self,
        input: ListInteractiveElementsInput,
    ) -> ToolResult<ListInteractiveElementsData> {
        super::AppCore::execute_list_interactive_elements(self, input)
    }

    fn execute_find_element(&mut self, input: FindElementInput) -> ToolResult<FindElementData> {
        super::AppCore::execute_find_element(self, input)
    }

    fn execute_click_element(&mut self, input: ClickElementInput) -> ToolResult<ClickElementData> {
        super::AppCore::execute_click_element(self, input)
    }

    fn execute_focus_element(&mut self, input: FocusElementInput) -> ToolResult<FocusElementData> {
        super::AppCore::execute_focus_element(self, input)
    }

    fn execute_type_into_element(
        &mut self,
        input: TypeIntoElementInput,
    ) -> ToolResult<TypeIntoElementData> {
        super::AppCore::execute_type_into_element(self, input)
    }

    fn execute_submit_active_form(
        &mut self,
        input: SubmitActiveFormInput,
    ) -> ToolResult<SubmitActiveFormData> {
        super::AppCore::execute_submit_active_form(self, input)
    }

    fn execute_extract_page_model(
        &mut self,
        input: ExtractPageModelInput,
    ) -> ToolResult<ExtractPageModelData> {
        super::AppCore::execute_extract_page_model(self, input)
    }

    fn execute_get_page_snapshot(
        &mut self,
        input: GetPageSnapshotInput,
    ) -> ToolResult<PageSnapshotData> {
        super::AppCore::execute_get_page_snapshot(self, input)
    }

    fn execute_set_tts_voice(&mut self, input: SetTtsVoiceInput) -> ToolResult<SetTtsVoiceData> {
        super::AppCore::execute_set_tts_voice(self, input)
    }

    fn execute_set_playback_volume(
        &mut self,
        input: SetPlaybackVolumeInput,
    ) -> ToolResult<SetPlaybackVolumeData> {
        super::AppCore::execute_set_playback_volume(self, input)
    }

    fn execute_set_playback_speed(
        &mut self,
        input: SetPlaybackSpeedInput,
    ) -> ToolResult<SetPlaybackSpeedData> {
        super::AppCore::execute_set_playback_speed(self, input)
    }

    fn execute_set_browser_visibility(
        &mut self,
        input: SetBrowserVisibilityInput,
    ) -> ToolResult<SetBrowserVisibilityData> {
        super::AppCore::execute_set_browser_visibility(self, input)
    }

    fn execute_get_agent_state(&mut self, input: GetAgentStateInput) -> ToolResult<AgentStateData> {
        super::AppCore::execute_get_agent_state(self, input)
    }

    fn execute_get_runtime_status(
        &mut self,
        input: GetRuntimeStatusInput,
    ) -> ToolResult<GetRuntimeStatusData> {
        super::AppCore::execute_get_runtime_status(self, input)
    }

    fn execute_confirm_action(
        &mut self,
        input: ConfirmActionInput,
    ) -> ToolResult<ConfirmActionData> {
        super::AppCore::execute_confirm_action(self, input)
    }

    fn execute_report_result(&mut self, input: ReportResultInput) -> ToolResult<ReportResultData> {
        super::AppCore::execute_report_result(self, input)
    }
}
