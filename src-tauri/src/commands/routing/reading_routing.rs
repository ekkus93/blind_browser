use super::*;

pub(crate) fn resolve_direct_read_page_command(
    transcript: &str,
    request_id: &str,
    current_page: Option<&PageModel>,
    agent_state: &AgentStateData,
    active_skill_names: &[String],
) -> Option<PlannerOutput> {
    let normalized = normalize_transcript_for_routing(transcript);
    if normalized.is_empty() || !is_read_page_phrase(&normalized) {
        return None;
    }

    let selected_skills = selected_skill(active_skill_names, "read_page");

    if agent_state.page_id.is_none() {
        let summary = String::from("There is no current page to read yet.");
        return Some(PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::ReadPage,
                goal: String::from("Read the current page from the beginning."),
                target_description: Some(String::from("current page")),
            },
            selected_skills,
            steps: vec![PlannedStep {
                step_id: String::from("report-missing-page"),
                tool_name: ToolName::ReportResult,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "status": ReportStatus::NeedsFollowUp,
                    "summary": summary.clone(),
                    "next_recommended_action": "Open a page first, then ask me to read it.",
                    "user_message": summary
                }),
                purpose: String::from("Report that there is no active page available to read."),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        });
    }

    if let Some(region_id) = current_page.and_then(first_readable_region_id) {
        return Some(PlannerOutput {
            status: PlannerStatus::Ready,
            intent: IntentSummary {
                name: IntentName::ReadPage,
                goal: String::from("Read the current page from the beginning."),
                target_description: Some(String::from("current page")),
            },
            selected_skills,
            steps: vec![PlannedStep {
                step_id: String::from("read-page-from-start"),
                tool_name: ToolName::ReadRegion,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "region_id": region_id,
                    "interruption_mode": "Interrupt"
                }),
                purpose: String::from(
                    "Restart narration from the first readable region of the current page.",
                ),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            }],
            requires_confirmation: false,
            confirmation_reason: None,
            blocked_reason: None,
            user_message: None,
        });
    }

    Some(PlannerOutput {
        status: PlannerStatus::Ready,
        intent: IntentSummary {
            name: IntentName::ReadPage,
            goal: String::from("Read the current page from the beginning."),
            target_description: Some(String::from("current page")),
        },
        selected_skills,
        steps: vec![
            PlannedStep {
                step_id: String::from("extract-page-for-reading"),
                tool_name: ToolName::ExtractPageModel,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "use_dom_extraction": true,
                    "include_headings": true,
                    "include_links": true
                }),
                purpose: String::from("Refresh the readable page model before starting narration."),
                on_success: StepTransition::NextStep {
                    step_id: String::from("read-first-region"),
                },
                on_failure: StepTransition::Replan,
            },
            PlannedStep {
                step_id: String::from("read-first-region"),
                tool_name: ToolName::ReadNextRegion,
                arguments: serde_json::json!({
                    "request_id": request_id,
                    "timeout_ms": serde_json::Value::Null,
                    "interruption_mode": "Interrupt"
                }),
                purpose: String::from(
                    "Start narration from the first readable region of the refreshed page.",
                ),
                on_success: StepTransition::Complete,
                on_failure: StepTransition::Replan,
            },
        ],
        requires_confirmation: false,
        confirmation_reason: None,
        blocked_reason: None,
        user_message: None,
    })
}
