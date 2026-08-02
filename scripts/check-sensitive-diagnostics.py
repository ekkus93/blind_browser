#!/usr/bin/env python3
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
SENSITIVE = re.compile(
    r"planner_input|page_model|transcript|api_key|authorization|cookie|response_body|tool_result|\.arguments",
    re.IGNORECASE,
)
LOG_START = re.compile(r"(?:tracing::)?(?:trace|debug|info|warn|error)!\s*\(|console\.(?:debug|info|warn|error)\s*\(")
def diagnostic_call_expression(lines: list[str], index: int) -> str:
    source = "\n".join(lines[index:])
    match = LOG_START.search(source)
    if match is None:
        return lines[index]

    start = match.start()
    depth = 0
    quote: str | None = None
    escaped = False
    saw_open = False
    for position in range(start, min(len(source), start + 16_384)):
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
                return source[start:position + 1]
    return source[start:start + 16_384]


violations = []
for directory, suffixes in [(ROOT / "src-tauri" / "src", {".rs"}), (ROOT / "src", {".ts", ".tsx", ".mjs"})]:
    for path in directory.rglob("*"):
        if path.suffix not in suffixes:
            continue
        lines = path.read_text(errors="replace").splitlines()
        for index, line in enumerate(lines):
            if LOG_START.search(line):
                expression = diagnostic_call_expression(lines, index)
                if SENSITIVE.search(expression):
                    violations.append(f"{path.relative_to(ROOT)}:{index + 1}: sensitive value referenced by diagnostic call")

planner_contract = (ROOT / "src-tauri/src/commands/contracts/planner.rs").read_text()
if re.search(r"derive\([^)]*Debug[^)]*\)\s*\npub struct PlannerInput", planner_contract):
    violations.append("PlannerInput must not derive Debug")

tool_contract = (ROOT / "src-tauri/src/commands/contracts/mod.rs").read_text()
if "impl Serialize for ToolError" not in tool_contract or "redact_json_value" not in tool_contract:
    violations.append("ToolError must use the centralized redacting serializer")

frontend_errors = (ROOT / "src/api/errors.ts").read_text()
if "sanitizeToolError" not in frontend_errors or "redactDiagnosticText" not in frontend_errors:
    violations.append("frontend invoke errors must pass through privacy redaction")

if violations:
    print("Sensitive diagnostics audit failed:", file=sys.stderr)
    for violation in violations:
        print(f"- {violation}", file=sys.stderr)
    raise SystemExit(1)
print("Sensitive diagnostics audit passed")
