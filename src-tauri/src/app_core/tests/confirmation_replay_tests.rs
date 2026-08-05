use super::*;

use crate::browser::BrowserVisibilityMode;
use crate::commands::{
    build_confirmation_manifest, ConfirmationRuntimeContext, PendingPlanExecutionState,
};

fn visibility_step(request_id: &str) -> PlannedStep {
    PlannedStep {
        step_id: String::from("step-visible"),
        tool_name: ToolName::SetBrowserVisibility,
        arguments: serde_json::json!({
            "request_id": request_id,
            "timeout_ms": 1000,
            "mode": "Headless"
        }),
        purpose: String::from("apply the confirmed visibility change"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    }
}

fn install_pending_visibility_confirmation(
    core: &mut super::super::AppCore,
    request_id: &str,
    confirmation_id: &str,
) -> String {
    let queued_steps = vec![visibility_step(request_id)];
    let context = ConfirmationRuntimeContext::current(None, None);
    let built = build_confirmation_manifest(request_id, &queued_steps, &context)
        .expect("confirmation manifest should build");
    let confirmation_digest = built.digest.clone();

    core.state.pending_confirmation_id = Some(confirmation_id.to_string());
    core.state.pending_plan_execution = Some(PendingPlanExecutionState {
        request_id: request_id.to_string(),
        intent_name: IntentName::SetBrowserVisibility,
        selected_skills: vec![String::from("confirm_action")],
        confirmation_id: confirmation_id.to_string(),
        manifest_digest: built.digest,
        manifest: built.manifest,
        prompt_text: built.prompt_text,
        runtime_state_token: String::new(),
        next_step_id: Some(String::from("step-visible")),
        queued_step_ids: vec![String::from("step-visible")],
        queued_steps,
    });
    let runtime_state_token = core.current_runtime_state_token();
    core.state
        .pending_plan_execution
        .as_mut()
        .expect("pending confirmation fixture should be installed")
        .runtime_state_token = runtime_state_token;

    confirmation_digest
}

#[test]
#[cfg_attr(
    any(windows, target_os = "linux"),
    ignore = "real Wry AppCore fixture must run in a process-isolated test invocation"
)]
#[cfg_attr(
    not(any(windows, target_os = "linux")),
    ignore = "real Wry AppCore fixture requires Tauri's any-thread desktop builder"
)]
fn app_core_confirmation_replay_and_runtime_state_binding_are_enforced() {
    let builder = tauri::Builder::<tauri::Wry>::default();
    #[cfg(any(windows, target_os = "linux"))]
    let builder = builder.any_thread();
    let app = builder
        .build(tauri::generate_context!())
        .expect("test Tauri application should build");

    let mut replay_core = super::super::AppCore::new(app.handle().clone())
        .expect("AppCore should initialize for the replay regression");
    let replay_confirmation_id = "confirm-replay-1";
    let replay_digest = install_pending_visibility_confirmation(
        &mut replay_core,
        "req-confirm-replay",
        replay_confirmation_id,
    );

    let first = replay_core.submit_confirmation_response(
        replay_confirmation_id,
        &replay_digest,
        true,
        false,
    );
    assert!(first.tool_result.ok);
    assert!(matches!(
        first.resume_outcome,
        ExecutionOutcome::Complete { .. }
    ));
    assert_eq!(
        replay_core.state.browser_visibility,
        BrowserVisibilityMode::Headless
    );
    assert_eq!(replay_core.state.pending_confirmation_id, None);
    assert!(replay_core.state.pending_plan_execution.is_none());

    let duplicate = replay_core.submit_confirmation_response(
        replay_confirmation_id,
        &replay_digest,
        true,
        false,
    );
    assert!(!duplicate.tool_result.ok);
    assert!(matches!(
        duplicate.resume_outcome,
        ExecutionOutcome::Aborted { ref error, .. }
            if error.code == "missing_pending_execution"
    ));
    assert_eq!(
        duplicate
            .tool_result
            .error
            .as_ref()
            .map(|error| error.code.as_str()),
        Some("missing_pending_execution")
    );

    let mut stale_core = super::super::AppCore::new(app.handle().clone())
        .expect("AppCore should initialize for the stale-state regression");
    let stale_confirmation_id = "confirm-stale-1";
    let stale_digest = install_pending_visibility_confirmation(
        &mut stale_core,
        "req-confirm-stale",
        stale_confirmation_id,
    );

    // Audio state is part of the relevant-configuration fingerprint. A change
    // while the user decides must invalidate the queued protected action.
    stale_core.state.audio.playback_volume = 0.25;
    let stale =
        stale_core.submit_confirmation_response(stale_confirmation_id, &stale_digest, true, false);

    assert!(!stale.tool_result.ok);
    assert!(matches!(
        stale.resume_outcome,
        ExecutionOutcome::Aborted { ref error, .. }
            if error.code == "stale_confirmation_runtime_state"
    ));
    assert_eq!(
        stale
            .tool_result
            .error
            .as_ref()
            .map(|error| error.code.as_str()),
        Some("stale_confirmation_runtime_state")
    );
    assert_eq!(
        stale_core.state.browser_visibility,
        BrowserVisibilityMode::Visible
    );
    assert_eq!(stale_core.state.pending_confirmation_id, None);
    assert!(stale_core.state.pending_plan_execution.is_none());
}
