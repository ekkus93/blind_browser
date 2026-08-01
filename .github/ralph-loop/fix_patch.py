from pathlib import Path

root = Path(__file__).resolve().parents[2]
path = root / "src-tauri/src/commands/validators/mod.rs"
text = path.read_text(encoding="utf-8")
old = '''pub fn validate_planner_output_with_safety(
    planner_output: &PlannerOutput,
    available_tools: &[AvailableTool],
    active_skill_names: &[String],
    safety: &PlannerSafetySettings,
) -> Result<(), ToolError> {
    // Preserve all schema, transition, availability, and legacy metadata checks,
'''
new = '''pub fn validate_planner_output_with_safety(
    planner_output: &PlannerOutput,
    available_tools: &[AvailableTool],
    active_skill_names: &[String],
    safety: &PlannerSafetySettings,
) -> Result<(), ToolError> {
    // Prohibited actions are security policy violations, not ordinary schema
    // mistakes. Reject them before less-specific structural validation so the
    // caller and audit trail retain the authoritative reason code.
    let preliminary_decision = evaluate_action_policy(&planner_output.steps, safety);
    if preliminary_decision.requirement == ConfirmationRequirement::Prohibited {
        return Err(ToolError {
            code: String::from("prohibited_planner_action"),
            message: String::from(
                "planner output contains an action prohibited by deterministic runtime policy",
            ),
            retryable: false,
            details: serde_json::to_value(preliminary_decision).ok(),
        });
    }

    // Preserve all schema, transition, availability, and legacy metadata checks,
'''
if text.count(old) != 1:
    raise RuntimeError("expected validator insertion point exactly once")
path.write_text(text.replace(old, new, 1), encoding="utf-8")
