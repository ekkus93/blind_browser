from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text()


def write(path: str, content: str) -> None:
    (ROOT / path).write_text(content)


def replace_once(path: str, old: str, new: str) -> None:
    content = read(path)
    count = content.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one match, found {count}: {old[:140]!r}")
    write(path, content.replace(old, new, 1))


# Reject cyclic planner transition graphs during validation, before any tool can
# execute. The executor retains its visited-step guard as defense in depth.
replace_once(
    "src-tauri/src/commands/validators/mod.rs",
    "    for step in &planner_output.steps {\n        validate_step_transition(&step.on_success, &seen_step_ids, &step.step_id)?;\n        validate_step_transition(&step.on_failure, &seen_step_ids, &step.step_id)?;\n    }\n\n    for skill_name in &planner_output.selected_skills {",
    "    for step in &planner_output.steps {\n        validate_step_transition(&step.on_success, &seen_step_ids, &step.step_id)?;\n        validate_step_transition(&step.on_failure, &seen_step_ids, &step.step_id)?;\n    }\n    validate_acyclic_step_graph(&planner_output.steps)?;\n\n    for skill_name in &planner_output.selected_skills {",
)

replace_once(
    "src-tauri/src/commands/validators/mod.rs",
    "pub fn validate_planner_output_with_safety(\n",
    r'''fn validate_acyclic_step_graph(steps: &[PlannedStep]) -> Result<(), ToolError> {
    fn visit_step(
        step_index: usize,
        steps: &[PlannedStep],
        visit_state: &mut [u8],
    ) -> Result<(), ToolError> {
        visit_state[step_index] = 1;
        let step = &steps[step_index];
        for transition in [&step.on_success, &step.on_failure] {
            let StepTransition::NextStep { step_id } = transition else {
                continue;
            };
            let next_index = steps
                .iter()
                .position(|candidate| candidate.step_id == *step_id)
                .expect("transition targets were validated before cycle detection");
            match visit_state[next_index] {
                1 => {
                    return Err(invalid_planner_output(
                        format!(
                            "planner transition graph contains a cycle through '{}' and '{}'",
                            step.step_id, step_id
                        ),
                        Some(serde_json::json!({
                            "step_id": step.step_id,
                            "next_step_id": step_id,
                        })),
                    ));
                }
                0 => visit_step(next_index, steps, visit_state)?,
                _ => {}
            }
        }
        visit_state[step_index] = 2;
        Ok(())
    }

    let mut visit_state = vec![0_u8; steps.len()];
    for step_index in 0..steps.len() {
        if visit_state[step_index] == 0 {
            visit_step(step_index, steps, &mut visit_state)?;
        }
    }
    Ok(())
}

pub fn validate_planner_output_with_safety(
''',
)

# Exercise the production AppCore confirmation path with one Wry application.
# Tauri explicitly requires any_thread() for Linux/Windows unit-test worker
# threads. Combining both scenarios prevents competing native event loops.
write(
    "src-tauri/src/app_core/tests/confirmation_replay_tests.rs",
    r'''use super::*;

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
    let stale = stale_core.submit_confirmation_response(
        stale_confirmation_id,
        &stale_digest,
        true,
        false,
    );

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
''',
)

print("Applied BBCR-015 graph validation and real AppCore test harness fixes")
