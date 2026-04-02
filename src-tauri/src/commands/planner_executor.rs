use super::*;

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

pub fn execute_planner_output<E: DeterministicToolExecutor>(
    executor: &mut E,
    request_id: String,
    planner_output: &PlannerOutput,
) -> ExecutionOutcome {
    execute_planner_output_with_runner(request_id, planner_output, |step| {
        execute_planned_step(executor, step)
    })
}

pub fn resume_after_confirmation<E: DeterministicToolExecutor>(
    executor: &mut E,
    pending_plan_execution: &PendingPlanExecutionState,
    confirmation_id: &str,
    confirmed: bool,
) -> ExecutionOutcome {
    resume_after_confirmation_with_runner(
        pending_plan_execution,
        confirmation_id,
        confirmed,
        |step| execute_planned_step(executor, step),
    )
}

pub(crate) fn execute_planner_output_with_runner<Runner>(
    request_id: String,
    planner_output: &PlannerOutput,
    mut run_step: Runner,
) -> ExecutionOutcome
where
    Runner: FnMut(&PlannedStep) -> SerializedToolResult,
{
    let trace = ExecutionTrace {
        executed_step_ids: Vec::new(),
        tool_results: Vec::new(),
    };

    match planner_output.status {
        PlannerStatus::Complete => {
            return ExecutionOutcome::Complete { trace };
        }
        PlannerStatus::Blocked => {
            return ExecutionOutcome::Aborted {
                trace,
                error: ToolError {
                    code: String::from("planner_blocked"),
                    message: planner_output.user_message.clone().unwrap_or_else(|| {
                        String::from("planner blocked execution before any tools could run")
                    }),
                    retryable: false,
                    details: Some(serde_json::json!({
                        "blocked_reason": planner_output.blocked_reason,
                    })),
                },
            };
        }
        PlannerStatus::Ready | PlannerStatus::NeedsConfirmation => {}
    }

    let Some(current_step_id) = planner_output
        .steps
        .first()
        .map(|step| step.step_id.clone())
    else {
        return ExecutionOutcome::Aborted {
            trace,
            error: ToolError {
                code: String::from("empty_plan"),
                message: String::from("planner returned no executable steps"),
                retryable: false,
                details: Some(serde_json::json!({
                    "planner_status": planner_output.status,
                })),
            },
        };
    };

    execute_steps_with_runner(
        StepExecutionContext {
            request_id,
            intent_name: planner_output.intent.name.clone(),
            selected_skills: planner_output.selected_skills.clone(),
            steps: &planner_output.steps,
            initial_step_id: current_step_id,
            block_side_effects_until_confirmation: planner_output.status
                == PlannerStatus::NeedsConfirmation,
        },
        &mut run_step,
        trace,
    )
}

fn resume_after_confirmation_with_runner<Runner>(
    pending_plan_execution: &PendingPlanExecutionState,
    confirmation_id: &str,
    confirmed: bool,
    mut run_step: Runner,
) -> ExecutionOutcome
where
    Runner: FnMut(&PlannedStep) -> SerializedToolResult,
{
    let trace = ExecutionTrace {
        executed_step_ids: Vec::new(),
        tool_results: Vec::new(),
    };

    if pending_plan_execution.confirmation_id != confirmation_id {
        return ExecutionOutcome::Aborted {
            trace,
            error: ToolError {
                code: String::from("confirmation_id_mismatch"),
                message: String::from(
                    "confirmation response did not match the pending confirmation id",
                ),
                retryable: false,
                details: Some(serde_json::json!({
                    "expected_confirmation_id": pending_plan_execution.confirmation_id,
                    "received_confirmation_id": confirmation_id,
                })),
            },
        };
    }

    if !confirmed {
        return ExecutionOutcome::NeedsReplan { trace };
    }

    let Some(next_step_id) = pending_plan_execution.next_step_id.clone() else {
        return ExecutionOutcome::Complete { trace };
    };

    execute_steps_with_runner(
        StepExecutionContext {
            request_id: pending_plan_execution.request_id.clone(),
            intent_name: pending_plan_execution.intent_name.clone(),
            selected_skills: pending_plan_execution.selected_skills.clone(),
            steps: &pending_plan_execution.queued_steps,
            initial_step_id: next_step_id,
            block_side_effects_until_confirmation: false,
        },
        &mut run_step,
        trace,
    )
}

fn execute_steps_with_runner<Runner>(
    context: StepExecutionContext<'_>,
    run_step: &mut Runner,
    mut trace: ExecutionTrace,
) -> ExecutionOutcome
where
    Runner: FnMut(&PlannedStep) -> SerializedToolResult,
{
    let StepExecutionContext {
        request_id,
        intent_name,
        selected_skills,
        steps,
        initial_step_id,
        block_side_effects_until_confirmation,
    } = context;

    let step_positions = match build_step_positions(steps) {
        Ok(positions) => positions,
        Err(error) => {
            return ExecutionOutcome::Aborted { trace, error };
        }
    };

    if !step_positions.contains_key(&initial_step_id) {
        return ExecutionOutcome::Aborted {
            trace,
            error: ToolError {
                code: String::from("missing_resume_step"),
                message: format!(
                    "pending execution referenced missing step '{}'",
                    initial_step_id
                ),
                retryable: false,
                details: None,
            },
        };
    }

    let mut current_step_id = initial_step_id;
    let mut visited_step_ids = HashSet::new();
    loop {
        if !visited_step_ids.insert(current_step_id.clone()) {
            return ExecutionOutcome::Aborted {
                trace,
                error: ToolError {
                    code: String::from("planner_step_cycle"),
                    message: format!(
                        "planner attempted to execute step '{}' more than once",
                        current_step_id
                    ),
                    retryable: false,
                    details: None,
                },
            };
        }

        let step = &steps[*step_positions
            .get(&current_step_id)
            .expect("step positions should contain the current step")];

        if block_side_effects_until_confirmation && is_side_effecting_tool(&step.tool_name) {
            return ExecutionOutcome::Aborted {
                trace,
                error: ToolError {
                    code: String::from("side_effect_before_confirmation"),
                    message: format!(
                        "planner attempted to execute side-effecting tool {:?} before confirmation",
                        step.tool_name
                    ),
                    retryable: false,
                    details: Some(serde_json::json!({
                        "step_id": step.step_id,
                        "tool_name": step.tool_name,
                    })),
                },
            };
        }

        let result = run_step(step);
        trace.executed_step_ids.push(step.step_id.clone());
        trace.tool_results.push(result.clone());

        let transition = if result.ok {
            &step.on_success
        } else {
            &step.on_failure
        };

        match transition {
            StepTransition::NextStep { step_id } => {
                if !step_positions.contains_key(step_id.as_str()) {
                    return ExecutionOutcome::Aborted {
                        trace,
                        error: ToolError {
                            code: String::from("missing_transition_step"),
                            message: format!(
                                "planner referenced missing next step '{}' from '{}'",
                                step_id, step.step_id
                            ),
                            retryable: false,
                            details: None,
                        },
                    };
                }
                current_step_id = step_id.clone();
            }
            StepTransition::Complete => {
                return ExecutionOutcome::Complete { trace };
            }
            StepTransition::RequestConfirmation => {
                let confirmation_id = match extract_confirmation_id(&result) {
                    Ok(confirmation_id) => confirmation_id,
                    Err(error) => {
                        return ExecutionOutcome::Aborted { trace, error };
                    }
                };
                let prompt_text = match extract_confirmation_prompt_text(&result) {
                    Ok(prompt_text) => prompt_text,
                    Err(error) => {
                        return ExecutionOutcome::Aborted { trace, error };
                    }
                };

                let queued_step_ids = queued_step_ids_after(steps, step, &step_positions);
                let queued_steps = queued_steps_after(steps, step, &step_positions);
                let pending_plan_execution = PendingPlanExecutionState {
                    request_id,
                    intent_name: intent_name.clone(),
                    selected_skills: selected_skills.clone(),
                    confirmation_id: confirmation_id.clone(),
                    prompt_text,
                    next_step_id: queued_step_ids.first().cloned(),
                    queued_step_ids,
                    queued_steps,
                };

                return ExecutionOutcome::AwaitingConfirmation {
                    trace,
                    pending_confirmation_id: confirmation_id,
                    pending_plan_execution,
                };
            }
            StepTransition::Replan => {
                return ExecutionOutcome::NeedsReplan { trace };
            }
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

fn build_step_positions(steps: &[PlannedStep]) -> Result<HashMap<String, usize>, ToolError> {
    let mut positions = HashMap::with_capacity(steps.len());

    for (index, step) in steps.iter().enumerate() {
        if positions.insert(step.step_id.clone(), index).is_some() {
            return Err(ToolError {
                code: String::from("duplicate_step_id"),
                message: format!("planner returned duplicate step id '{}'", step.step_id),
                retryable: false,
                details: None,
            });
        }
    }

    Ok(positions)
}

fn queued_step_ids_after(
    steps: &[PlannedStep],
    current_step: &PlannedStep,
    step_positions: &HashMap<String, usize>,
) -> Vec<String> {
    let Some(current_index) = step_positions.get(&current_step.step_id).copied() else {
        return Vec::new();
    };

    steps
        .iter()
        .skip(current_index + 1)
        .map(|step| step.step_id.clone())
        .collect()
}

fn queued_steps_after(
    steps: &[PlannedStep],
    current_step: &PlannedStep,
    step_positions: &HashMap<String, usize>,
) -> Vec<PlannedStep> {
    let Some(current_index) = step_positions.get(&current_step.step_id).copied() else {
        return Vec::new();
    };

    steps.iter().skip(current_index + 1).cloned().collect()
}

fn extract_confirmation_id(result: &SerializedToolResult) -> Result<String, ToolError> {
    let confirmation_id = result
        .data
        .as_ref()
        .and_then(|data| data.get("confirmation_id"))
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    confirmation_id.ok_or_else(|| ToolError {
        code: String::from("missing_confirmation_id"),
        message: String::from(
            "step requested confirmation but the tool result did not include confirmation_id",
        ),
        retryable: false,
        details: Some(serde_json::json!({
            "tool_name": result.tool_name,
            "request_id": result.request_id,
        })),
    })
}

fn extract_confirmation_prompt_text(result: &SerializedToolResult) -> Result<String, ToolError> {
    let prompt_text = result
        .data
        .as_ref()
        .and_then(|data| data.get("prompt_text"))
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    prompt_text.ok_or_else(|| ToolError {
        code: String::from("missing_confirmation_prompt"),
        message: String::from(
            "step requested confirmation but the tool result did not include prompt_text",
        ),
        retryable: false,
        details: Some(serde_json::json!({
            "tool_name": result.tool_name,
            "request_id": result.request_id,
        })),
    })
}

fn is_side_effecting_tool(tool_name: &ToolName) -> bool {
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

fn serialize_tool_result<T>(result: ToolResult<T>) -> SerializedToolResult
where
    T: Serialize,
{
    let ToolResult {
        ok,
        tool_name,
        request_id,
        timestamp_ms,
        data,
        error,
        warnings,
        observations,
    } = result;

    let serialized_data = match data {
        Some(data) => match serde_json::to_value(data) {
            Ok(value) => Some(value),
            Err(error) => {
                return ToolResult::failure(
                    tool_name,
                    request_id,
                    ToolError {
                        code: String::from("tool_result_serialization_failed"),
                        message: format!("failed to serialize tool result payload: {error}"),
                        retryable: false,
                        details: None,
                    },
                    vec![String::from(
                        "Executor could not serialize the tool result payload.",
                    )],
                );
            }
        },
        None => None,
    };

    ToolResult {
        ok,
        tool_name,
        request_id,
        timestamp_ms,
        data: serialized_data,
        error,
        warnings,
        observations,
    }
}

fn inferred_request_id(step: &PlannedStep) -> String {
    step.arguments
        .get("request_id")
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| step.step_id.clone())
}
