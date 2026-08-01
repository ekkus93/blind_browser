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


path = "src-tauri/src/commands/confirmation_manifest.rs"
text = read(path)
text = replace_once(
    text,
    '''    pub argument_digest: String,
    pub safe_summary: String,
''',
    '''    pub argument_digest: String,
    pub transition_digest: String,
    pub safe_summary: String,
''',
    path,
)
text = replace_once(
    text,
    '''            argument_digest: digest_json(&step.arguments),
            safe_summary: safe_action_summary(step),
''',
    '''            argument_digest: digest_json(&step.arguments),
            transition_digest: digest_json(&serde_json::json!({
                "on_success": &step.on_success,
                "on_failure": &step.on_failure,
            })),
            safe_summary: safe_action_summary(step),
''',
    path,
)
write(path, text)

path = "src/tauri-types.ts"
text = read(path)
text = replace_once(
    text,
    '''  argument_digest: string;
  safe_summary: string;
''',
    '''  argument_digest: string;
  transition_digest: string;
  safe_summary: string;
''',
    path,
)
write(path, text)

path = "src-tauri/src/commands/tests/confirmation.rs"
text = read(path)
marker = '''#[test]
fn reordering_queued_actions_invalidates_confirmation()'''
new_test = r'''#[test]
fn changing_a_queued_transition_invalidates_confirmation() {
    let mut executor = MockExecutor::default();
    let mut pending = pending_visibility_confirmation(&bound_context(10_000));
    pending.queued_steps[0].on_success = StepTransition::Replan;

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
        other => panic!("expected transition-bound manifest mismatch, got {other:?}"),
    }
}

'''
text = replace_once(text, marker, new_test + marker, path)
write(path, text)
