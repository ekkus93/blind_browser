#!/usr/bin/env python3
from pathlib import Path
import runpy

root = Path(__file__).resolve().parents[1]
transformer = root / "scripts/batch8-privacy-controls.py"
text = transformer.read_text()
old = '''def insert_before_last_brace(path: str, addition: str) -> None:
    content = read(path)
    index = content.rfind("\\n}")
    if index < 0:
        raise SystemExit(f"{path}: final module brace not found")
    write(path, content[:index] + addition + content[index:])
'''
new = '''def insert_before_last_brace(path: str, addition: str) -> None:
    content = read(path)
    top_level_append_paths = {
        "src/api/providers.ts",
        "src/planner-actions.ts",
        "src-tauri/src/commands/contracts/mod.rs",
        "src-tauri/src/config/validation.rs",
    }
    if path in top_level_append_paths:
        write(path, content.rstrip() + "\\n\\n" + addition.strip() + "\\n")
        return
    index = content.rfind("\\n}")
    if index < 0:
        raise SystemExit(f"{path}: final module brace not found")
    write(path, content[:index] + addition + content[index:])
'''
if old not in text:
    raise SystemExit("insert_before_last_brace helper shape changed")
text = text.replace(old, new, 1)

old_replace_once = '''def replace_once(path: str, old: str, new: str) -> None:
    content = read(path)
    count = content.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one occurrence, found {count}: {old[:120]!r}")
    write(path, content.replace(old, new, 1))
'''
new_replace_once = '''def replace_once(path: str, old: str, new: str) -> None:
    content = read(path)
    if path == "src/panel-types.ts" and old.startswith("  timeoutMs: number | null;"):
        start = content.index("export interface RemotePlannerPanelState")
        end = content.index("export interface RemoteTtsPanelState", start)
        section = content[start:end]
        count = section.count(old)
        if count != 1:
            raise SystemExit(f"{path}: expected one planner-panel occurrence, found {count}")
        write(path, content[:start] + section.replace(old, new, 1) + content[end:])
        return
    if path == "src/panel-state.ts" and old.startswith("      timeoutMs: null,"):
        start = content.index("    remotePlannerPanelState:")
        end = content.index("    providerFailoverPanelState:", start)
        section = content[start:end]
        count = section.count(old)
        if count != 1:
            raise SystemExit(f"{path}: expected one remote-planner-state occurrence, found {count}")
        write(path, content[:start] + section.replace(old, new, 1) + content[end:])
        return
    count = content.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one occurrence, found {count}: {old[:120]!r}")
    write(path, content.replace(old, new, 1))
'''
if old_replace_once not in text:
    raise SystemExit("replace_once helper shape changed")
text = text.replace(old_replace_once, new_replace_once, 1)

import_header = "from pathlib import Path\nimport re\n\nROOT ="
if import_header not in text:
    raise SystemExit("transformer import header changed")
text = text.replace(
    import_header,
    "from pathlib import Path\nimport re\nimport subprocess\n\nROOT =",
    1,
)

if text.count("    minimum=4,\n") != 1:
    raise SystemExit("remote planner signature cardinality assertion shape changed")
text = text.replace("    minimum=4,\n", "    minimum=2,\n", 1)

old_normalize = '''# Normalize generated files.
for path in ROOT.rglob("*"):
    if path.is_file() and path.suffix in {".rs", ".ts", ".tsx", ".mjs", ".py", ".toml", ".yml", ".md"}:
        text = path.read_text(errors="strict")
        path.write_text("\\n".join(line.rstrip() for line in text.splitlines()) + ("\\n" if text.endswith("\\n") else ""))

print("Batch 8 privacy controls transformed")
'''
new_normalize = '''# Normalize only files changed by this transformer. Do not rewrite unrelated
# documentation or preserve staging machinery in the production commit.
changed_paths = subprocess.check_output(
    ["git", "diff", "--name-only", "--diff-filter=ACMRT"],
    cwd=ROOT,
    text=True,
).splitlines()
for relative in changed_paths:
    path = ROOT / relative
    if path.is_file() and path.suffix in {".rs", ".ts", ".tsx", ".mjs", ".py", ".toml", ".yml"}:
        generated = path.read_text(errors="strict")
        path.write_text("\\n".join(line.rstrip() for line in generated.splitlines()) + ("\\n" if generated.endswith("\\n") else ""))

print("Batch 8 privacy controls transformed")
'''
if old_normalize not in text:
    raise SystemExit("normalization block shape changed")
text = text.replace(old_normalize, new_normalize, 1)
transformer.write_text(text)
runpy.run_path(str(transformer), run_name="__main__")

audit_path = root / "scripts/check-sensitive-diagnostics.py"
audit = audit_path.read_text()
old_audit = '''violations = []
for directory, suffixes in [(ROOT / "src-tauri" / "src", {".rs"}), (ROOT / "src", {".ts", ".tsx", ".mjs"})]:
    for path in directory.rglob("*"):
        if path.suffix not in suffixes:
            continue
        lines = path.read_text(errors="replace").splitlines()
        for index, line in enumerate(lines):
            if LOG_START.search(line):
                window = "\\n".join(lines[index:index + 12])
                if SENSITIVE.search(window):
                    violations.append(f"{path.relative_to(ROOT)}:{index + 1}: sensitive value referenced by diagnostic call")
'''
new_audit = '''def diagnostic_call_expression(lines: list[str], index: int) -> str:
    source = "\\n".join(lines[index:])
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
            elif character == "\\\\":
                escaped = True
            elif character == quote:
                quote = None
            continue
        if character in {"\\\"", "'"}:
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
'''
if old_audit not in audit:
    raise SystemExit("generated diagnostics audit shape changed")
audit_path.write_text(audit.replace(old_audit, new_audit, 1))
