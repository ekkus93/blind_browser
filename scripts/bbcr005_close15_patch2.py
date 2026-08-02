from __future__ import annotations

import re
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


# Use the named token helper installed by patch1 instead of creating a throwaway
# planning snapshot solely to expose its opaque token.
replace_once(
    "src-tauri/src/app_core/command_dispatch.rs",
    "            runtime_state_token: self.capture_planning_state_snapshot().runtime_state_token,",
    "            runtime_state_token: self.current_runtime_state_token(),",
)

# A pending confirmation carries a server-only runtime binding. It is never
# accepted from serialized client state and remains backward-compatible with
# older persisted state through the empty default.
replace_once(
    "src-tauri/src/commands/contracts/planner.rs",
    "    pub manifest: ConfirmationManifest,\n    pub prompt_text: String,\n    pub next_step_id: Option<String>,",
    "    pub manifest: ConfirmationManifest,\n    pub prompt_text: String,\n    #[serde(skip_serializing, default)]\n    #[schemars(skip)]\n    pub runtime_state_token: String,\n    pub next_step_id: Option<String>,",
)

pending_pattern = re.compile(
    r"(PendingPlanExecutionState\s*\{.*?\n(?P<indent>\s*)prompt_text:[^\n]+,\n)(?!\s*runtime_state_token:)",
    re.DOTALL,
)
for path in sorted((ROOT / "src-tauri" / "src").rglob("*.rs")):
    content = path.read_text()

    def inject_pending_token(match: re.Match[str]) -> str:
        indent = match.group("indent")
        return match.group(1) + f"{indent}runtime_state_token: String::new(),\n"

    updated, count = pending_pattern.subn(inject_pending_token, content)
    if count:
        path.write_text(updated)

# Install the authoritative token only after AppState has stored the pending
# confirmation ID. This makes the token include the exact live challenge while
# keeping construction in the generic planner executor client-agnostic.
replace_once(
    "src-tauri/src/app_core/command_dispatch.rs",
    "        let outcome =\n            execute_planner_output_with_runtime_safety(self, request_id, &prepared, &safety);\n        self.state.apply_execution_outcome(&outcome);\n        outcome",
    "        let mut outcome =\n            execute_planner_output_with_runtime_safety(self, request_id, &prepared, &safety);\n        self.state.apply_execution_outcome(&outcome);\n        if let ExecutionOutcome::AwaitingConfirmation {\n            pending_plan_execution,\n            ..\n        } = &mut outcome\n        {\n            let runtime_state_token = self.current_runtime_state_token();\n            pending_plan_execution.runtime_state_token = runtime_state_token.clone();\n            if let Some(stored_pending) = self.state.pending_plan_execution.as_mut() {\n                stored_pending.runtime_state_token = runtime_state_token;\n            }\n        }\n        outcome",
)

# Revalidate the entire runtime/configuration binding after live click preflight
# and immediately before consuming the single-use challenge and resuming queued
# protected steps. Stale challenges are destroyed fail-closed.
replace_once(
    "src-tauri/src/app_core/confirmation_workflow.rs",
    "        if confirmed {\n            if let Err(error) =\n                self.preflight_pending_click_authorizations(&pending_plan_execution.queued_steps)\n            {\n                return ExecutionOutcome::Aborted {\n                    trace: ExecutionTrace {\n                        executed_step_ids: Vec::new(),\n                        tool_results: Vec::new(),\n                    },\n                    error,\n                };\n            }\n        }\n\n        // Matching challenges are consumed before dispatch so duplicate responses,",
    "        if confirmed {\n            if let Err(error) =\n                self.preflight_pending_click_authorizations(&pending_plan_execution.queued_steps)\n            {\n                return ExecutionOutcome::Aborted {\n                    trace: ExecutionTrace {\n                        executed_step_ids: Vec::new(),\n                        tool_results: Vec::new(),\n                    },\n                    error,\n                };\n            }\n\n            let observed_runtime_state_token = self.current_runtime_state_token();\n            if pending_plan_execution.runtime_state_token.is_empty()\n                || pending_plan_execution.runtime_state_token != observed_runtime_state_token\n            {\n                let expected_runtime_state_token =\n                    pending_plan_execution.runtime_state_token.clone();\n                self.state.clear_pending_execution();\n                return confirmation_abort(\n                    \"stale_confirmation_runtime_state\",\n                    \"runtime or relevant configuration state changed while confirmation was pending\",\n                    Some(serde_json::json!({\n                        \"expected_runtime_state_token\": expected_runtime_state_token,\n                        \"observed_runtime_state_token\": observed_runtime_state_token,\n                    })),\n                );\n            }\n        }\n\n        // Matching challenges are consumed before dispatch so duplicate responses,",
)

# Fix the exact current Clippy initializer without suppressing the lint.
replace_once(
    "src-tauri/src/commands/tests/batch5_safety.rs",
    "    let mut state = AppState::default();\n    state.current_page_id = Some(String::from(\"page-1\"));\n    state.page_generation = 7;\n    state.pending_confirmation_id = Some(String::from(\"confirm-1\"));",
    "    let mut state = AppState {\n        current_page_id: Some(String::from(\"page-1\")),\n        page_generation: 7,\n        pending_confirmation_id: Some(String::from(\"confirm-1\")),\n        ..AppState::default()\n    };",
)

# The trait import became unnecessary with the current Tauri API.
replace_once(
    "src-tauri/src/app_core/tests/confirmation_replay_tests.rs",
    "\nuse tauri::Manager;\n",
    "\n",
)

# Bind the hand-built replay fixture exactly as production does.
replace_once(
    "src-tauri/src/app_core/tests/confirmation_replay_tests.rs",
    "    core.state.pending_plan_execution = Some(PendingPlanExecutionState {\n        request_id: String::from(\"req-confirm-replay\"),",
    "    core.state.pending_plan_execution = Some(PendingPlanExecutionState {\n        request_id: String::from(\"req-confirm-replay\"),",
)
replace_once(
    "src-tauri/src/app_core/tests/confirmation_replay_tests.rs",
    "        queued_steps,\n    });\n\n    let first =",
    "        queued_steps,\n    });\n    let runtime_state_token = core.current_runtime_state_token();\n    core.state\n        .pending_plan_execution\n        .as_mut()\n        .expect(\"pending replay fixture should be installed\")\n        .runtime_state_token = runtime_state_token;\n\n    let first =",
)

# Real AppCore regression: a relevant state change while the user decides must
# consume the stale challenge and must not execute the queued protected action.
confirmation_tests = read("src-tauri/src/app_core/tests/confirmation_replay_tests.rs")
confirmation_tests += r'''

#[test]
fn app_core_confirmation_rejects_runtime_state_change_before_resume() {
    let app = tauri::Builder::<tauri::Wry>::default()
        .build(tauri::generate_context!())
        .expect("test Tauri application should build");
    let mut core = super::super::AppCore::new(app.handle().clone())
        .expect("AppCore should initialize for the stale confirmation regression");

    let queued_steps = vec![visibility_step()];
    let context = ConfirmationRuntimeContext::current(None, None);
    let built = build_confirmation_manifest("req-confirm-stale", &queued_steps, &context)
        .expect("confirmation manifest should build");
    let confirmation_id = String::from("confirm-stale-1");
    let confirmation_digest = built.digest.clone();

    core.state.pending_confirmation_id = Some(confirmation_id.clone());
    core.state.pending_plan_execution = Some(PendingPlanExecutionState {
        request_id: String::from("req-confirm-stale"),
        intent_name: IntentName::SetBrowserVisibility,
        selected_skills: vec![String::from("confirm_action")],
        confirmation_id: confirmation_id.clone(),
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
        .expect("pending stale-state fixture should be installed")
        .runtime_state_token = runtime_state_token;

    core.state.audio.playback_volume = 0.25;
    let resolution =
        core.submit_confirmation_response(&confirmation_id, &confirmation_digest, true, false);

    assert!(!resolution.tool_result.ok);
    assert!(matches!(
        resolution.resume_outcome,
        ExecutionOutcome::Aborted { ref error, .. }
            if error.code == "stale_confirmation_runtime_state"
    ));
    assert_eq!(
        resolution
            .tool_result
            .error
            .as_ref()
            .map(|error| error.code.as_str()),
        Some("stale_confirmation_runtime_state")
    );
    assert_eq!(
        core.state.browser_visibility,
        BrowserVisibilityMode::Visible
    );
    assert_eq!(core.state.pending_confirmation_id, None);
    assert!(core.state.pending_plan_execution.is_none());
}
'''
write("src-tauri/src/app_core/tests/confirmation_replay_tests.rs", confirmation_tests)

# Focused mapping regression for the bounded replanning contract.
command_dispatch = read("src-tauri/src/app_core/command_dispatch.rs")
command_dispatch += r'''

#[cfg(test)]
mod planning_snapshot_outcome_tests {
    use super::*;

    fn snapshot_error(code: &str) -> ToolError {
        ToolError {
            code: code.to_string(),
            message: String::from("runtime planning snapshot rejected"),
            retryable: false,
            details: None,
        }
    }

    #[test]
    fn stale_snapshot_failures_request_bounded_replanning() {
        for code in [
            "missing_planning_snapshot",
            "planning_snapshot_expired",
            "stale_planning_snapshot",
        ] {
            assert!(matches!(
                planner_snapshot_validation_outcome(snapshot_error(code)),
                ExecutionOutcome::NeedsReplan { .. }
            ));
        }
    }

    #[test]
    fn non_snapshot_validation_failures_remain_terminal() {
        assert!(matches!(
            planner_snapshot_validation_outcome(snapshot_error("invalid_planner_output")),
            ExecutionOutcome::Aborted { .. }
        ));
    }
}
'''
write("src-tauri/src/app_core/command_dispatch.rs", command_dispatch)

print("Applied BBCR-015 post-confirmation runtime binding and Clippy fixes")
