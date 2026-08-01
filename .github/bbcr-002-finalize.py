from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, content: str) -> None:
    (ROOT / path).write_text(content, encoding="utf-8")


def replace_once(text: str, old: str, new: str, path: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{path}: expected one occurrence, found {count}: {old[:100]!r}")
    return text.replace(old, new, 1)


# Keep the frontend contract aligned with the serialized Rust challenge.
path = "src/tauri-types.ts"
text = read(path)
old_tool_names = '''export type ToolName =
  | "OpenUrl"
  | "GoBack"
  | "GoForward"
  | "ReloadPage"
  | "ScrollPage"
  | "CaptureScreenshot"
  | "SetBrowserVisibility"
  | "GetPageSnapshot"
  | "ExtractPageModel"
  | "ListInteractiveElements"
  | "StopSpeaking"
  | "StartListening"
  | "StopListening"
  | "TranscribeCommand"
  | "SetTtsVoice"
  | "SetPlaybackVolume"
  | "SetPlaybackSpeed"
  | "RunOcr"
  | "MergeOcrIntoPageModel"
  | "GetAgentState"
  | "GetRuntimeStatus"
  | "ConfirmAction"
  | "ReportResult";
'''
new_tool_names = '''export type ToolName =
  | "OpenUrl"
  | "GoBack"
  | "GoForward"
  | "ReloadPage"
  | "GetHtml"
  | "EvalJs"
  | "ScrollPage"
  | "CaptureScreenshot"
  | "SetBrowserVisibility"
  | "GetPageSnapshot"
  | "ExtractPageModel"
  | "ListInteractiveElements"
  | "FindElement"
  | "ClickElement"
  | "FocusElement"
  | "TypeIntoElement"
  | "SubmitActiveForm"
  | "ReadRegion"
  | "ReadNextRegion"
  | "ReadPreviousRegion"
  | "StopSpeaking"
  | "StartListening"
  | "StopListening"
  | "TranscribeCommand"
  | "SetTtsVoice"
  | "SetPlaybackVolume"
  | "SetPlaybackSpeed"
  | "RunOcr"
  | "MergeOcrIntoPageModel"
  | "GetAgentState"
  | "GetRuntimeStatus"
  | "ConfirmAction"
  | "ReportResult";
'''
text = replace_once(text, old_tool_names, new_tool_names, path)
old_pending = '''export interface PendingPlanExecutionState {
  request_id: string;
  intent_name: IntentName;
  selected_skills: string[];
  confirmation_id: string;
  prompt_text: string;
  next_step_id: string | null;
  queued_step_ids: string[];
  queued_steps: PlannedStep[];
}
'''
new_pending = '''export interface ConfirmationActionManifest {
  sequence: number;
  step_id: string;
  tool_name: ToolName;
  argument_digest: string;
  safe_summary: string;
}

export interface ConfirmationManifest {
  request_id: string;
  page_id: string | null;
  origin: string | null;
  issued_at_ms: number;
  expires_at_ms: number;
  actions: ConfirmationActionManifest[];
}

export interface PendingPlanExecutionState {
  request_id: string;
  intent_name: IntentName;
  selected_skills: string[];
  confirmation_id: string;
  manifest_digest: string;
  manifest: ConfirmationManifest;
  prompt_text: string;
  next_step_id: string | null;
  queued_step_ids: string[];
}
'''
text = replace_once(text, old_pending, new_pending, path)
old_response = '''export interface ConfirmActionResponseInput {
  confirmationId: string;
  confirmed: boolean;
  timedOut: boolean;
}
'''
new_response = '''export interface ConfirmActionResponseInput {
  confirmationId: string;
  confirmationDigest: string;
  confirmed: boolean;
  timedOut: boolean;
}
'''
text = replace_once(text, old_response, new_response, path)
write(path, text)

# Update AppState unit fixtures for the expanded private pending state.
path = "src-tauri/src/state.rs"
text = read(path)
old_import = '''    use crate::commands::{
        ExecutionTrace, IntentName, PendingPlanExecutionState, SerializedToolResult, ToolError,
        ToolName,
    };
'''
new_import = '''    use crate::commands::{
        ConfirmationManifest, ExecutionTrace, IntentName, PendingPlanExecutionState,
        SerializedToolResult, ToolError, ToolName,
    };
'''
text = replace_once(text, old_import, new_import, path)
needle = '''                confirmation_id: String::from("confirm-1"),
                prompt_text: String::from("Proceed?"),
'''
replacement = '''                confirmation_id: String::from("confirm-1"),
                manifest_digest: String::from("digest-1"),
                manifest: ConfirmationManifest {
                    request_id: String::from("req-1"),
                    page_id: None,
                    origin: None,
                    issued_at_ms: 1,
                    expires_at_ms: 2,
                    actions: Vec::new(),
                },
                prompt_text: String::from("Proceed?"),
'''
count = text.count(needle)
if count != 2:
    raise RuntimeError(f"{path}: expected two pending fixtures, found {count}")
text = text.replace(needle, replacement)
write(path, text)

# Replace legacy resume tests with manifest-bound, adversarial regression tests.
path = "src-tauri/src/commands/tests/confirmation.rs"
text = read(path)
marker = '''#[test]
fn resumes_confirmed_pending_execution_from_stored_steps()'''
if marker not in text:
    raise RuntimeError(f"{path}: resume test marker missing")
prefix = text.split(marker, 1)[0]
new_tail = r'''fn visibility_step(step_id: &str, mode: &str) -> PlannedStep {
    PlannedStep {
        step_id: step_id.to_string(),
        tool_name: ToolName::SetBrowserVisibility,
        arguments: serde_json::json!({
            "request_id": "req-resume",
            "timeout_ms": 1000,
            "mode": mode
        }),
        purpose: String::from("apply confirmed action"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    }
}

fn pending_visibility_confirmation(
    context: &ConfirmationRuntimeContext,
) -> PendingPlanExecutionState {
    let queued_steps = vec![visibility_step("step-2", "Headless")];
    let built = build_confirmation_manifest("req-resume", &queued_steps, context)
        .expect("manifest should build");
    PendingPlanExecutionState {
        request_id: String::from("req-resume"),
        intent_name: IntentName::SetBrowserVisibility,
        selected_skills: vec![String::from("confirm_action")],
        confirmation_id: String::from("confirm-1"),
        manifest_digest: built.digest,
        manifest: built.manifest,
        prompt_text: built.prompt_text,
        next_step_id: Some(String::from("step-2")),
        queued_step_ids: vec![String::from("step-2")],
        queued_steps,
    }
}

fn bound_context(now_ms: u64) -> ConfirmationRuntimeContext {
    ConfirmationRuntimeContext::at(
        Some("page-1"),
        Some("https://example.com/form?token=secret"),
        now_ms,
    )
}

#[test]
fn resumes_confirmed_pending_execution_from_stored_steps() {
    let mut executor = MockExecutor::default();
    let issued_context = bound_context(10_000);
    let pending = pending_visibility_confirmation(&issued_context);
    let resume_context = bound_context(10_001);

    let outcome = resume_after_confirmation_with_context(
        &mut executor,
        &pending,
        "confirm-1",
        &pending.manifest_digest,
        true,
        &resume_context,
    );

    match outcome {
        ExecutionOutcome::Complete { trace } => {
            assert_eq!(trace.executed_step_ids, vec!["step-2"]);
            assert_eq!(executor.last_visibility, Some(BrowserVisibilityMode::Headless));
        }
        other => panic!("expected complete outcome after resume, got {other:?}"),
    }
}

#[test]
fn rejected_confirmation_does_not_execute_actions() {
    let mut executor = MockExecutor::default();
    let context = bound_context(10_000);
    let pending = pending_visibility_confirmation(&context);

    let outcome = resume_after_confirmation_with_context(
        &mut executor,
        &pending,
        "confirm-1",
        &pending.manifest_digest,
        false,
        &bound_context(10_001),
    );

    assert!(matches!(outcome, ExecutionOutcome::NeedsReplan { .. }));
    assert_eq!(executor.last_visibility, None);
}

#[test]
fn rejects_resume_with_mismatched_confirmation_id() {
    let mut executor = MockExecutor::default();
    let context = bound_context(10_000);
    let pending = pending_visibility_confirmation(&context);

    let outcome = resume_after_confirmation_with_context(
        &mut executor,
        &pending,
        "wrong-confirmation-id",
        &pending.manifest_digest,
        true,
        &bound_context(10_001),
    );

    match outcome {
        ExecutionOutcome::Aborted { error, .. } => {
            assert_eq!(error.code, "confirmation_id_mismatch");
            assert_eq!(executor.last_visibility, None);
        }
        other => panic!("expected aborted outcome after mismatch, got {other:?}"),
    }
}

#[test]
fn rejects_resume_with_mismatched_manifest_digest() {
    let mut executor = MockExecutor::default();
    let pending = pending_visibility_confirmation(&bound_context(10_000));

    let outcome = resume_after_confirmation_with_context(
        &mut executor,
        &pending,
        "confirm-1",
        "wrong-digest",
        true,
        &bound_context(10_001),
    );

    match outcome {
        ExecutionOutcome::Aborted { error, .. } => {
            assert_eq!(error.code, "confirmation_digest_mismatch");
            assert_eq!(executor.last_visibility, None);
        }
        other => panic!("expected digest mismatch, got {other:?}"),
    }
}

#[test]
fn changing_a_queued_argument_invalidates_confirmation() {
    let mut executor = MockExecutor::default();
    let mut pending = pending_visibility_confirmation(&bound_context(10_000));
    pending.queued_steps[0].arguments["mode"] = serde_json::json!("Visible");

    let outcome = resume_after_confirmation_with_context(
        &mut executor,
        &pending,
        "confirm-1",
        &pending.manifest_digest,
        true,
        &bound_context(10_001),
    );

    match outcome {
        ExecutionOutcome::Aborted { error, .. } => {
            assert_eq!(error.code, "confirmation_manifest_mismatch");
            assert_eq!(executor.last_visibility, None);
        }
        other => panic!("expected manifest mismatch, got {other:?}"),
    }
}

#[test]
fn reordering_queued_actions_invalidates_confirmation() {
    let context = bound_context(10_000);
    let queued_steps = vec![
        visibility_step("step-2", "Headless"),
        visibility_step("step-3", "Visible"),
    ];
    let built = build_confirmation_manifest("req-resume", &queued_steps, &context)
        .expect("manifest should build");
    let mut pending = PendingPlanExecutionState {
        request_id: String::from("req-resume"),
        intent_name: IntentName::SetBrowserVisibility,
        selected_skills: vec![String::from("confirm_action")],
        confirmation_id: String::from("confirm-1"),
        manifest_digest: built.digest,
        manifest: built.manifest,
        prompt_text: built.prompt_text,
        next_step_id: Some(String::from("step-2")),
        queued_step_ids: vec![String::from("step-2"), String::from("step-3")],
        queued_steps,
    };
    pending.queued_steps.reverse();
    let mut executor = MockExecutor::default();

    let outcome = resume_after_confirmation_with_context(
        &mut executor,
        &pending,
        "confirm-1",
        &pending.manifest_digest,
        true,
        &bound_context(10_001),
    );

    match outcome {
        ExecutionOutcome::Aborted { error, .. } => {
            assert_eq!(error.code, "confirmation_queue_mismatch");
            assert_eq!(executor.last_visibility, None);
        }
        other => panic!("expected queue mismatch, got {other:?}"),
    }
}

#[test]
fn page_or_origin_change_invalidates_confirmation() {
    let pending = pending_visibility_confirmation(&bound_context(10_000));
    let cases = [
        ConfirmationRuntimeContext::at(
            Some("page-2"),
            Some("https://example.com/form"),
            10_001,
        ),
        ConfirmationRuntimeContext::at(
            Some("page-1"),
            Some("https://attacker.example/form"),
            10_001,
        ),
    ];
    let expected_codes = ["confirmation_page_changed", "confirmation_origin_changed"];

    for (context, expected_code) in cases.into_iter().zip(expected_codes) {
        let mut executor = MockExecutor::default();
        let outcome = resume_after_confirmation_with_context(
            &mut executor,
            &pending,
            "confirm-1",
            &pending.manifest_digest,
            true,
            &context,
        );
        match outcome {
            ExecutionOutcome::Aborted { error, .. } => {
                assert_eq!(error.code, expected_code);
                assert_eq!(executor.last_visibility, None);
            }
            other => panic!("expected state-bound rejection, got {other:?}"),
        }
    }
}

#[test]
fn expired_confirmation_is_rejected() {
    let pending = pending_visibility_confirmation(&bound_context(10_000));
    let mut executor = MockExecutor::default();
    let expired = bound_context(pending.manifest.expires_at_ms);

    let outcome = resume_after_confirmation_with_context(
        &mut executor,
        &pending,
        "confirm-1",
        &pending.manifest_digest,
        true,
        &expired,
    );

    match outcome {
        ExecutionOutcome::Aborted { error, .. } => {
            assert_eq!(error.code, "confirmation_expired");
            assert_eq!(executor.last_visibility, None);
        }
        other => panic!("expected expired confirmation rejection, got {other:?}"),
    }
}

#[test]
fn serialized_pending_state_hides_raw_queued_arguments_and_secrets() {
    let context = bound_context(10_000);
    let queued_steps = vec![PlannedStep {
        step_id: String::from("step-secret"),
        tool_name: ToolName::TypeIntoElement,
        arguments: serde_json::json!({
            "request_id": "req-secret",
            "timeout_ms": 1000,
            "element_id": "password-field",
            "text": "super-secret-password",
            "mode": "Replace",
            "submit": "KeepEditing"
        }),
        purpose: String::from("type a password"),
        on_success: StepTransition::Complete,
        on_failure: StepTransition::Replan,
    }];
    let built = build_confirmation_manifest("req-secret", &queued_steps, &context)
        .expect("manifest should build");
    let pending = PendingPlanExecutionState {
        request_id: String::from("req-secret"),
        intent_name: IntentName::FillInput,
        selected_skills: vec![String::from("confirm_action")],
        confirmation_id: String::from("confirm-secret"),
        manifest_digest: built.digest,
        manifest: built.manifest,
        prompt_text: built.prompt_text,
        next_step_id: Some(String::from("step-secret")),
        queued_step_ids: vec![String::from("step-secret")],
        queued_steps,
    };

    let value = serde_json::to_value(&pending).expect("pending state should serialize");
    let encoded = value.to_string();
    assert!(value.get("queued_steps").is_none());
    assert!(!encoded.contains("super-secret-password"));
    assert!(pending.prompt_text.contains("21 characters"));
    assert!(!pending.prompt_text.contains("super-secret-password"));
}
'''
write(path, prefix + new_tail)

# The planner-written prompt must no longer control user-visible confirmation copy.
path = "src-tauri/src/commands/tests/planner_flow/execution.rs"
text = read(path)
old_assert = '''            assert_eq!(pending_plan_execution.prompt_text, "Proceed?");
            assert_eq!(
                pending_plan_execution.next_step_id,
                Some(String::from("step-2"))
            );
'''
new_assert = '''            assert_eq!(
                pending_plan_execution.prompt_text,
                "Approve this action: Change browser visibility."
            );
            assert_ne!(pending_plan_execution.prompt_text, "Proceed?");
            assert_eq!(pending_plan_execution.manifest.actions.len(), 1);
            assert!(!pending_plan_execution.manifest_digest.is_empty());
            assert_eq!(
                pending_plan_execution.next_step_id,
                Some(String::from("step-2"))
            );
'''
text = replace_once(text, old_assert, new_assert, path)
write(path, text)

# Frontend fixtures carry the digest that must be echoed to the runtime.
path = "src/voice-loop.test.mjs"
text = read(path)
needle = '''      confirmationId,
      promptText: "Submit this form?",
'''
replacement = '''      confirmationId,
      confirmationDigest: "digest-active",
      promptText: "Submit this form?",
'''
text = replace_once(text, needle, replacement, path)
write(path, text)

path = "src/confirmation-panel-test-helpers.mjs"
text = read(path)
for index in (1, 2, 3):
    needle = f'''    confirmationId: "confirmation-{index}",
    promptText: "Submit the form?",
'''
    replacement = f'''    confirmationId: "confirmation-{index}",
    confirmationDigest: "digest-{index}",
    promptText: "Submit the form?",
'''
    text = replace_once(text, needle, replacement, path)
write(path, text)

# Remove the superseded private wrapper before Clippy evaluates dead code.
path = "src-tauri/src/commands/planner_executor/execution.rs"
text = read(path)
start = text.find("pub(super) fn resume_after_confirmation_with_runner<Runner>(")
end = text.find("pub(super) fn resume_after_confirmation_with_runner_and_context<Runner>(")
if start == -1 or end == -1 or end <= start:
    raise RuntimeError(f"{path}: legacy resume wrapper boundaries missing")
text = text[:start] + text[end:]
write(path, text)
