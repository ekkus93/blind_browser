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

pub(super) fn build_report_result_step(
    request_id: &str,
    step_id: &str,
    summary: String,
) -> PlannedStep {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::browser::BrowserVisibilityMode;
    use crate::commands::{
        IntentName, IntentSummary, PlannedStep, PlannerStatus, ReportStatus, StepTransition,
        ToolName,
    };

    #[test]
    fn round_audio_setting_value_rounds_to_two_decimal_places() {
        assert_eq!(round_audio_setting_value(0.756_789_f32), 0.76_f32);
        assert_eq!(round_audio_setting_value(1.0_f32), 1.0_f32);
        assert_eq!(round_audio_setting_value(0.5_f32), 0.5_f32);
        assert_eq!(round_audio_setting_value(0.0_f32), 0.0_f32);
    }

    #[test]
    fn build_report_result_step_produces_correctly_shaped_report_step() {
        let step = build_report_result_step("req-1", "report-step", String::from("Done."));
        assert_eq!(step.step_id, "report-step");
        assert_eq!(step.tool_name, ToolName::ReportResult);
        assert!(matches!(step.on_success, StepTransition::Complete));
        assert!(matches!(step.on_failure, StepTransition::Replan));
        assert_eq!(step.arguments["request_id"], serde_json::json!("req-1"));
        assert_eq!(
            step.arguments["status"],
            serde_json::json!(ReportStatus::Success)
        );
        assert_eq!(step.arguments["summary"], serde_json::json!("Done."));
        assert_eq!(step.arguments["user_message"], serde_json::json!("Done."));
    }

    #[test]
    fn build_single_step_planner_output_wraps_step_in_ready_plan() {
        let step = PlannedStep {
            step_id: String::from("action-step"),
            tool_name: ToolName::ScrollPage,
            arguments: serde_json::json!({"request_id": "req-1"}),
            purpose: String::from("Scroll down."),
            on_success: StepTransition::NextStep {
                step_id: String::from("report"),
            },
            on_failure: StepTransition::Replan,
        };
        let output = build_single_step_planner_output(
            IntentSummary {
                name: IntentName::ReadNext,
                goal: String::from("Scroll down."),
                target_description: None,
            },
            vec![String::from("reading")],
            step,
        );
        assert_eq!(output.status, PlannerStatus::Ready);
        assert!(!output.requires_confirmation);
        assert_eq!(output.steps.len(), 1);
        assert_eq!(output.steps[0].tool_name, ToolName::ScrollPage);
    }

    #[test]
    fn build_browser_visibility_planner_output_produces_two_step_plan() {
        let output = build_browser_visibility_planner_output(
            "req-vis",
            BrowserVisibilityMode::Headless,
            vec![String::from("browser_visibility")],
            String::from("Browser is now headless."),
        );
        assert_eq!(output.status, PlannerStatus::Ready);
        assert_eq!(output.steps.len(), 2);
        assert_eq!(output.steps[0].tool_name, ToolName::SetBrowserVisibility);
        assert_eq!(
            output.steps[0].arguments["mode"],
            serde_json::json!(BrowserVisibilityMode::Headless)
        );
        assert_eq!(output.steps[1].tool_name, ToolName::ReportResult);
        assert!(matches!(
            output.steps[1].on_success,
            StepTransition::Complete
        ));
        assert_eq!(
            output.steps[1].arguments["summary"],
            serde_json::json!("Browser is now headless.")
        );
    }
}
