use super::*;

pub(super) fn build_single_step_planner_output(
    intent: IntentSummary,
    selected_skills: Vec<String>,
    step: PlannedStep,
) -> PlannerOutput {
    PlannerOutput {
        status: PlannerStatus::Ready,
        intent,
        selected_skills,
        steps: vec![step],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    }
}

pub(super) fn build_audio_set_planner_output(spec: AudioSetPlanSpec<'_>) -> PlannerOutput {
    PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: spec.intent_name,
            goal: spec.goal,
            target_description: spec.target_description,
        },
        selected_skills: spec.selected_skills,
        steps: vec![
            PlannedStep {
                step_id: String::from(spec.set_step_id),
                tool_name: spec.tool_name,
                arguments: spec.tool_arguments,
                purpose: spec.tool_purpose,
                on_success: StepTransition::NextStep {
                    step_id: String::from(spec.report_step_id),
                },
                on_failure: StepTransition::Replan,
            },
            build_report_result_step(spec.request_id, spec.report_step_id, spec.report_summary),
        ],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    }
}

pub(super) fn build_audio_report_planner_output(
    request_id: &str,
    intent_name: IntentName,
    goal: String,
    selected_skills: Vec<String>,
    target_description: Option<String>,
    report_summary: String,
) -> PlannerOutput {
    PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: intent_name,
            goal,
            target_description,
        },
        selected_skills,
        steps: vec![build_report_result_step(
            request_id,
            "report-audio-setting",
            report_summary,
        )],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    }
}

pub(super) fn build_browser_visibility_planner_output(
    request_id: &str,
    target_mode: BrowserVisibilityMode,
    selected_skills: Vec<String>,
    report_summary: String,
) -> PlannerOutput {
    PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::SetBrowserVisibility,
            goal: String::from("Set the browser visibility mode to the requested target."),
            target_description: Some(format_browser_visibility_mode(target_mode)),
        },
        selected_skills,
        steps: vec![
            PlannedStep {
                step_id: String::from("set-browser-visibility"),
                tool_name: ToolName::SetBrowserVisibility,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "mode": target_mode
                }),
                purpose: String::from("Apply the requested browser visibility mode."),
                on_success: StepTransition::NextStep {
                    step_id: String::from("report-browser-visibility"),
                },
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from("report-browser-visibility"),
                tool_name: ToolName::ReportResult,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "status": ReportStatus::Success,
                    "summary": report_summary.clone(),
                    "next_recommended_action": serde_json::Value::Null,
                    "user_message": report_summary
                }),
                purpose: String::from("Report the resulting browser visibility mode."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    }
}

pub(super) fn build_status_query_planner_output(spec: StatusQueryPlanSpec<'_>) -> PlannerOutput {
    PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: spec.intent_name,
            goal: spec.goal,
            target_description: spec.target_description,
        },
        selected_skills: spec.selected_skills,
        steps: vec![
            PlannedStep {
                step_id: String::from(spec.read_step_id),
                tool_name: spec.read_tool_name,
                arguments: spec.read_tool_arguments,
                purpose: spec.read_tool_purpose,
                on_success: StepTransition::NextStep {
                    step_id: String::from(spec.report_step_id),
                },
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from(spec.report_step_id),
                tool_name: ToolName::ReportResult,
                arguments: serde_json::json!({
                    "request_id": spec.request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "status": ReportStatus::Success,
                    "summary": spec.report_summary.clone(),
                    "next_recommended_action": serde_json::Value::Null,
                    "user_message": spec.report_summary
                }),
                purpose: String::from("Report the resulting status query answer."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    }
}

pub(super) fn build_report_result_step(request_id: &str, step_id: &str, summary: String) -> PlannedStep {
    PlannedStep {
        step_id: String::from(step_id),
        tool_name: ToolName::ReportResult,
        arguments: serde_json::json!({
            "request_id": request_id,
            "timeout_ms": serde_json::Value::Null,
            "status": ReportStatus::Success,
            "summary": summary.clone(),
            "next_recommended_action": serde_json::Value::Null,
            "user_message": summary
        }),
        purpose: String::from("Report the resulting playback setting."),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    }
}
pub(super) struct StatusQueryPlanSpec<'a> {
    pub(super) request_id: &'a str,
    pub(super) intent_name: IntentName,
    pub(super) goal: String,
    pub(super) selected_skills: Vec<String>,
    pub(super) target_description: Option<String>,
    pub(super) read_step_id: &'a str,
    pub(super) read_tool_name: ToolName,
    pub(super) read_tool_arguments: serde_json::Value,
    pub(super) read_tool_purpose: String,
    pub(super) report_step_id: &'a str,
    pub(super) report_summary: String,
}

pub(super) fn round_audio_setting_value(value: f32) -> f32 {
    (value * 100.0).round() / 100.0
}

pub(super) struct AudioSetPlanSpec<'a> {
    pub(super) request_id: &'a str,
    pub(super) intent_name: IntentName,
    pub(super) goal: String,
    pub(super) selected_skills: Vec<String>,
    pub(super) target_description: Option<String>,
    pub(super) set_step_id: &'a str,
    pub(super) tool_name: ToolName,
    pub(super) tool_arguments: serde_json::Value,
    pub(super) tool_purpose: String,
    pub(super) report_step_id: &'a str,
    pub(super) report_summary: String,
}
