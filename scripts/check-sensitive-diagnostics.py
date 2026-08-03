#!/usr/bin/env python3
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
SENSITIVE = re.compile(
    r"planner_input|page_model|transcript|api_key|authorization|cookie|response_body|raw_response|tool_result|\.arguments",
    re.IGNORECASE,
)
LOG_START = re.compile(
    r"(?:(?:tracing|log)::)?(?:trace|debug|info|warn|error)!\s*\("
    r"|console\.(?:debug|info|warn|error)\s*\("
    r"|(?:e?println|dbg)!\s*\("
)
FRONTEND_RAW_ERROR_ARGUMENT = re.compile(
    r"(?:,\s*|\{\s*)(?:error|[A-Za-z][A-Za-z0-9]*Error)\s*(?:[,})])"
)
FRONTEND_ERROR_REDACTION = re.compile(
    r"classifyInvokeFailure|sanitizeToolError|redactDiagnostic(?:Text|Value)"
)
SENSITIVE_DEBUG_STRUCTS = {
    "PlannerInput",
    "SetRemoteApiKeyData",
    "TestRemoteApiKeyData",
}
DERIVED_STRUCT = re.compile(
    r"#\s*\[\s*derive\s*\((?P<traits>.*?)\)\s*\]\s*"
    r"(?:pub(?:\([^)]*\))?\s+)?struct\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\b",
    re.DOTALL,
)


def diagnostic_call_expression(source: str, start: int) -> str:
    depth = 0
    quote: str | None = None
    escaped = False
    saw_open = False
    limit = min(len(source), start + 16_384)
    for position in range(start, limit):
        character = source[position]
        if quote is not None:
            if escaped:
                escaped = False
            elif character == "\\":
                escaped = True
            elif character == quote:
                quote = None
            continue
        if character in {"\"", "'"}:
            quote = character
        elif character == "(":
            depth += 1
            saw_open = True
        elif character == ")" and saw_open:
            depth -= 1
            if depth == 0:
                return source[start : position + 1]
    return source[start:limit]


def scan_diagnostic_source(label: str, source: str) -> list[str]:
    violations: list[str] = []
    is_frontend = Path(label).suffix in {".ts", ".tsx", ".mjs"}
    for match in LOG_START.finditer(source):
        expression = diagnostic_call_expression(source, match.start())
        line = source.count("\n", 0, match.start()) + 1
        if SENSITIVE.search(expression):
            violations.append(
                f"{label}:{line}: sensitive value referenced by diagnostic call"
            )
        if (
            is_frontend
            and FRONTEND_RAW_ERROR_ARGUMENT.search(expression)
            and not FRONTEND_ERROR_REDACTION.search(expression)
        ):
            violations.append(
                f"{label}:{line}: raw frontend error object referenced by diagnostic call"
            )
    return violations


def scan_sensitive_debug_derives(label: str, source: str) -> list[str]:
    violations: list[str] = []
    for match in DERIVED_STRUCT.finditer(source):
        traits = {trait.strip().split("::")[-1] for trait in match.group("traits").split(",")}
        name = match.group("name")
        if name in SENSITIVE_DEBUG_STRUCTS and "Debug" in traits:
            line = source.count("\n", 0, match.start()) + 1
            violations.append(f"{label}:{line}: {name} must not derive Debug")
    return violations


def scan_repository() -> list[str]:
    violations: list[str] = []
    for directory, suffixes in [
        (ROOT / "src-tauri" / "src", {".rs"}),
        (ROOT / "src", {".ts", ".tsx", ".mjs"}),
    ]:
        for path in directory.rglob("*"):
            if path.suffix not in suffixes:
                continue
            source = path.read_text(errors="replace")
            label = str(path.relative_to(ROOT))
            violations.extend(scan_diagnostic_source(label, source))
            if path.suffix == ".rs":
                violations.extend(scan_sensitive_debug_derives(label, source))

    tool_contract = (ROOT / "src-tauri/src/commands/contracts/mod.rs").read_text()
    if "impl Serialize for ToolError" not in tool_contract or "redact_json_value" not in tool_contract:
        violations.append("ToolError must use the centralized redacting serializer")

    frontend_errors = (ROOT / "src/api/errors.ts").read_text()
    if "sanitizeToolError" not in frontend_errors or "redactDiagnosticText" not in frontend_errors:
        violations.append("frontend invoke errors must pass through privacy redaction")

    return violations


def run_self_test() -> None:
    hostile_sources = {
        "rust-multiline.rs": """
tracing::warn!(
    request_id = %request_id,
    response_body = %response_body,
    "remote provider failed"
);
""",
        "frontend-multiline.ts": """
console.error(
  "planner failed",
  planner_input,
);
""",
        "frontend-raw-error.ts": """
console.error(
  "request failed",
  error,
);
""",
        "raw-arguments.rs": "dbg!(step.arguments.clone());",
        "debug-derive.rs": """
#[derive(
    Clone,
    Debug,
    Serialize,
)]
pub struct PlannerInput {
    pub request_id: String,
}
""",
    }
    for label, source in hostile_sources.items():
        violations = scan_diagnostic_source(label, source)
        violations.extend(scan_sensitive_debug_derives(label, source))
        if not violations:
            raise SystemExit(f"self-test failed to reject {label}")

    benign = """
tracing::info!(request_id = %request_id, status = "failed", "provider request finished");
#[derive(Clone, Serialize)]
pub struct PlannerInput { pub request_id: String }
console.warn("request failed", sanitizeToolError(error));
console.error("request failed", classifyInvokeFailure(error));
"""
    violations = scan_diagnostic_source("benign.ts", benign)
    violations.extend(scan_sensitive_debug_derives("benign.ts", benign))
    if violations:
        raise SystemExit(f"self-test rejected benign diagnostics: {violations}")

    print("Sensitive diagnostics scanner self-test passed")


def main() -> None:
    if sys.argv[1:] == ["--self-test"]:
        run_self_test()
        return
    if sys.argv[1:]:
        raise SystemExit("usage: check-sensitive-diagnostics.py [--self-test]")

    violations = scan_repository()
    if violations:
        print("Sensitive diagnostics audit failed:", file=sys.stderr)
        for violation in violations:
            print(f"- {violation}", file=sys.stderr)
        raise SystemExit(1)
    print("Sensitive diagnostics audit passed")


if __name__ == "__main__":
    main()
