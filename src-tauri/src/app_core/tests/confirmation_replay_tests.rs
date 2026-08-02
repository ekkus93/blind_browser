use super::*;

use tauri::Manager;

use crate::browser::BrowserVisibilityMode;
use crate::commands::{
    build_confirmation_manifest, ConfirmationRuntimeContext, PendingPlanExecutionState,
};

fn visibility_step() -> PlannedStep {
    PlannedStep {
        step_id: String::from("step-visible"),
        tool_name: ToolName::SetBrowserVisibility,
        arguments: serde_json::json!({
            "request_id": "req-confirm-replay",
            "timeout_ms": 1000,
            "mode": "Headless"
        }),
        purpose: String::from("apply the confirmed visibility change"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    }
}

#[test]
fn app_core_confirmation_response_is_single_use() {
    let app = tauri::Builder::<tauri::Wry>::default()
        .build(tauri::generate_context!())
        .expect("test Tauri application should build");
    let mut core = super::super::AppCore::new(app.handle().clone())
        .expect("AppCore should initialize for the replay regression");

    let queued_steps = vec![visibility_step()];
    let context = ConfirmationRuntimeContext::current(None, None);
    let built = build_confirmation_manifest("req-confirm-replay", &queued_steps, &context)
        .expect("confirmation manifest should build");
    let confirmation_id = String::from("confirm-replay-1");
    let confirmation_digest = built.digest.clone();

    core.state.pending_confirmation_id = Some(confirmation_id.clone());
    core.state.pending_plan_execution = Some(PendingPlanExecutionState {
        request_id: String::from("req-confirm-replay"),
        intent_name: IntentName::SetBrowserVisibility,
        selected_skills: vec![String::from("confirm_action")],
        confirmation_id: confirmation_id.clone(),
        manifest_digest: built.digest,
        manifest: built.manifest,
        prompt_text: built.prompt_text,
        next_step_id: Some(String::from("step-visible")),
        queued_step_ids: vec![String::from("step-visible")],
        queued_steps,
    });

    let first =
        core.submit_confirmation_response(&confirmation_id, &confirmation_digest, true, false);
    assert!(first.tool_result.ok);
    assert!(matches!(
        first.resume_outcome,
        ExecutionOutcome::Complete { .. }
    ));
    assert_eq!(
        core.state.browser_visibility,
        BrowserVisibilityMode::Headless
    );
    assert_eq!(core.state.pending_confirmation_id, None);
    assert!(core.state.pending_plan_execution.is_none());

    let second =
        core.submit_confirmation_response(&confirmation_id, &confirmation_digest, true, false);
    assert!(!second.tool_result.ok);
    assert!(matches!(
        second.resume_outcome,
        ExecutionOutcome::Aborted { ref error, .. }
            if error.code == "missing_pending_execution"
    ));
    assert_eq!(
        second
            .tool_result
            .error
            .as_ref()
            .map(|error| error.code.as_str()),
        Some("missing_pending_execution")
    );
}
