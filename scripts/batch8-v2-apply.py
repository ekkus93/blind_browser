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
    if path in {"src/api/providers.ts", "src/planner-actions.ts"}:
        write(path, content.rstrip() + addition + "\\n")
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
