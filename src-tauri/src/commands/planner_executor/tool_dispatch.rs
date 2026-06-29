use super::super::{
    DeterministicToolExecutor, PlannedStep, SerializedToolResult, ToolError, ToolName, ToolResult,
};
use super::step_helpers::{inferred_request_id, serialize_tool_result};
use serde::{Deserialize, Serialize};

pub fn execute_planned_step<E: DeterministicToolExecutor>(
    executor: &mut E,
    step: &PlannedStep,
) -> SerializedToolResult {
    match step.tool_name {
        ToolName::OpenUrl => {
            execute_serialized_tool(step, ToolName::OpenUrl, executor, |executor, input| {
                executor.execute_open_url(input)
            })
        }
        ToolName::GoBack => {
            execute_serialized_tool(step, ToolName::GoBack, executor, |executor, input| {
                executor.execute_go_back(input)
            })
        }
        ToolName::GoForward => {
            execute_serialized_tool(step, ToolName::GoForward, executor, |executor, input| {
                executor.execute_go_forward(input)
            })
        }
        ToolName::ReloadPage => {
            execute_serialized_tool(step, ToolName::ReloadPage, executor, |executor, input| {
                executor.execute_reload_page(input)
            })
        }
        ToolName::GetHtml => {
            execute_serialized_tool(step, ToolName::GetHtml, executor, |executor, input| {
                executor.execute_get_html(input)
            })
        }
        ToolName::EvalJs => {
            execute_serialized_tool(step, ToolName::EvalJs, executor, |executor, input| {
                executor.execute_eval_js(input)
            })
        }
        ToolName::ScrollPage => {
            execute_serialized_tool(step, ToolName::ScrollPage, executor, |executor, input| {
                executor.execute_scroll_page(input)
            })
        }
        ToolName::CaptureScreenshot => execute_serialized_tool(
            step,
            ToolName::CaptureScreenshot,
            executor,
            |executor, input| executor.execute_capture_screenshot(input),
        ),
        ToolName::RunOcr => {
            execute_serialized_tool(step, ToolName::RunOcr, executor, |executor, input| {
                executor.execute_run_ocr(input)
            })
        }
        ToolName::MergeOcrIntoPageModel => execute_serialized_tool(
            step,
            ToolName::MergeOcrIntoPageModel,
            executor,
            |executor, input| executor.execute_merge_ocr_into_page_model(input),
        ),
        ToolName::ReadRegion => {
            execute_serialized_tool(step, ToolName::ReadRegion, executor, |executor, input| {
                executor.execute_read_region(input)
            })
        }
        ToolName::ReadNextRegion => execute_serialized_tool(
            step,
            ToolName::ReadNextRegion,
            executor,
            |executor, input| executor.execute_read_next_region(input),
        ),
        ToolName::ReadPreviousRegion => execute_serialized_tool(
            step,
            ToolName::ReadPreviousRegion,
            executor,
            |executor, input| executor.execute_read_previous_region(input),
        ),
        ToolName::StopSpeaking => {
            execute_serialized_tool(step, ToolName::StopSpeaking, executor, |executor, input| {
                executor.execute_stop_speaking(input)
            })
        }
        ToolName::StartListening => execute_serialized_tool(
            step,
            ToolName::StartListening,
            executor,
            |executor, input| executor.execute_start_listening(input),
        ),
        ToolName::StopListening => execute_serialized_tool(
            step,
            ToolName::StopListening,
            executor,
            |executor, input| executor.execute_stop_listening(input),
        ),
        ToolName::TranscribeCommand => execute_serialized_tool(
            step,
            ToolName::TranscribeCommand,
            executor,
            |executor, input| executor.execute_transcribe_command(input),
        ),
        ToolName::GetPageSnapshot => execute_serialized_tool(
            step,
            ToolName::GetPageSnapshot,
            executor,
            |executor, input| executor.execute_get_page_snapshot(input),
        ),
        ToolName::ListInteractiveElements => execute_serialized_tool(
            step,
            ToolName::ListInteractiveElements,
            executor,
            |executor, input| executor.execute_list_interactive_elements(input),
        ),
        ToolName::FindElement => {
            execute_serialized_tool(step, ToolName::FindElement, executor, |executor, input| {
                executor.execute_find_element(input)
            })
        }
        ToolName::ClickElement => {
            execute_serialized_tool(step, ToolName::ClickElement, executor, |executor, input| {
                executor.execute_click_element(input)
            })
        }
        ToolName::FocusElement => {
            execute_serialized_tool(step, ToolName::FocusElement, executor, |executor, input| {
                executor.execute_focus_element(input)
            })
        }
        ToolName::TypeIntoElement => execute_serialized_tool(
            step,
            ToolName::TypeIntoElement,
            executor,
            |executor, input| executor.execute_type_into_element(input),
        ),
        ToolName::SubmitActiveForm => execute_serialized_tool(
            step,
            ToolName::SubmitActiveForm,
            executor,
            |executor, input| executor.execute_submit_active_form(input),
        ),
        ToolName::ExtractPageModel => execute_serialized_tool(
            step,
            ToolName::ExtractPageModel,
            executor,
            |executor, input| executor.execute_extract_page_model(input),
        ),
        ToolName::SetTtsVoice => {
            execute_serialized_tool(step, ToolName::SetTtsVoice, executor, |executor, input| {
                executor.execute_set_tts_voice(input)
            })
        }
        ToolName::SetPlaybackVolume => execute_serialized_tool(
            step,
            ToolName::SetPlaybackVolume,
            executor,
            |executor, input| executor.execute_set_playback_volume(input),
        ),
        ToolName::SetPlaybackSpeed => execute_serialized_tool(
            step,
            ToolName::SetPlaybackSpeed,
            executor,
            |executor, input| executor.execute_set_playback_speed(input),
        ),
        ToolName::SetBrowserVisibility => execute_serialized_tool(
            step,
            ToolName::SetBrowserVisibility,
            executor,
            |executor, input| executor.execute_set_browser_visibility(input),
        ),
        ToolName::GetAgentState => execute_serialized_tool(
            step,
            ToolName::GetAgentState,
            executor,
            |executor, input| executor.execute_get_agent_state(input),
        ),
        ToolName::GetRuntimeStatus => execute_serialized_tool(
            step,
            ToolName::GetRuntimeStatus,
            executor,
            |executor, input| executor.execute_get_runtime_status(input),
        ),
        ToolName::ConfirmAction => execute_serialized_tool(
            step,
            ToolName::ConfirmAction,
            executor,
            |executor, input| executor.execute_confirm_action(input),
        ),
        ToolName::ReportResult => {
            execute_serialized_tool(step, ToolName::ReportResult, executor, |executor, input| {
                executor.execute_report_result(input)
            })
        }
    }
}

fn execute_serialized_tool<E, Input, Output, Handler>(
    step: &PlannedStep,
    tool_name: ToolName,
    executor: &mut E,
    handler: Handler,
) -> SerializedToolResult
where
    E: DeterministicToolExecutor,
    Input: for<'de> Deserialize<'de>,
    Output: Serialize,
    Handler: FnOnce(&mut E, Input) -> ToolResult<Output>,
{
    match serde_json::from_value::<Input>(step.arguments.clone()) {
        Ok(input) => serialize_tool_result(handler(executor, input)),
        Err(error) => ToolResult::failure(
            tool_name,
            inferred_request_id(step),
            ToolError {
                code: String::from("invalid_tool_arguments"),
                message: format!("tool arguments did not match the expected schema: {error}"),
                retryable: false,
                details: Some(serde_json::json!({
                    "step_id": step.step_id,
                    "arguments": step.arguments,
                })),
            },
            vec![String::from(
                "Executor rejected the tool call because the arguments were invalid.",
            )],
        ),
    }
}

pub(in crate::commands::planner_executor) fn is_side_effecting_tool(tool_name: &ToolName) -> bool {
    !matches!(
        tool_name,
        ToolName::CaptureScreenshot
            | ToolName::GetHtml
            | ToolName::GetPageSnapshot
            | ToolName::ExtractPageModel
            | ToolName::ListInteractiveElements
            | ToolName::FindElement
            | ToolName::TranscribeCommand
            | ToolName::GetAgentState
            | ToolName::GetRuntimeStatus
            | ToolName::ConfirmAction
            | ToolName::ReportResult
    )
}
