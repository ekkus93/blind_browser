from __future__ import annotations

import re
from pathlib import Path

PATH = Path("src-tauri/src/app_core/planner_redaction.rs")
HELPERS = (
    ("sanitize_remote_planner_input", "pub(crate) "),
    ("enforce_remote_planner_privacy", ""),
    ("privacy_error", ""),
)

text = PATH.read_text()

# Make each compatibility helper unambiguously test-only. Rebuild the declaration
# prefix rather than stacking attributes left by an earlier repair attempt.
for function_name, expected_visibility in HELPERS:
    declaration_pattern = re.compile(
        rf"(?m)^(?P<indent>[ \t]*)(?P<visibility>pub\(crate\)\s+)?fn {function_name}\("
    )
    matches = list(declaration_pattern.finditer(text))
    if len(matches) != 1:
        raise SystemExit(
            f"legacy helper finalizer: expected one {function_name} declaration, "
            f"found {len(matches)}"
        )

    match = matches[0]
    visibility = match.group("visibility") or ""
    if visibility != expected_visibility:
        raise SystemExit(
            f"legacy helper finalizer: {function_name} visibility was {visibility!r}, "
            f"expected {expected_visibility!r}"
        )

    declaration_start = match.start()
    prefix = text[:declaration_start]
    adjacent_cfg = re.search(r"(?m)^[ \t]*#\[cfg\(test\)\]\n$", prefix)
    if adjacent_cfg is not None and adjacent_cfg.end() == len(prefix):
        prefix = prefix[: adjacent_cfg.start()]

    declaration = f"#[cfg(test)]\n{expected_visibility}fn {function_name}("
    text = prefix + declaration + text[match.end() :]

# ToolError is needed only by test-only compatibility code in this module. Remove
# the grouped import before fully qualifying every standalone use so the default
# library build cannot depend on a cfg(test)-only import boundary.
commands_import_pattern = re.compile(
    r"use crate::commands::\{(?P<body>.*?)\n\};",
    re.DOTALL,
)
commands_import_match = commands_import_pattern.search(text)
if commands_import_match is None:
    raise SystemExit("legacy helper finalizer: commands import block was not found")

commands_body = commands_import_match.group("body")
commands_body, removed_imports = re.subn(
    r"(?<![A-Za-z0-9_])ToolError\s*,\s*",
    "",
    commands_body,
)
if removed_imports > 1:
    raise SystemExit(
        "legacy helper finalizer: multiple ToolError grouped imports were found"
    )
text = (
    text[: commands_import_match.start("body")]
    + commands_body
    + text[commands_import_match.end("body") :]
)

# Qualify the entire module, not merely parsed helper bodies. The previous
# body-bounded pass could report success while leaving a signature or test helper
# outside its effective replacement range. A preceding ':' or identifier means
# the token is already qualified or is part of a different identifier.
text, qualified_count = re.subn(
    r"(?<![A-Za-z0-9_:])ToolError\b",
    "crate::commands::ToolError",
    text,
)
if qualified_count == 0:
    raise SystemExit(
        "legacy helper finalizer: no standalone ToolError use was available to qualify"
    )

# Remove any obsolete standalone cfg(test) import left by an earlier repair.
text, standalone_imports = re.subn(
    r"(?m)^#\[cfg\(test\)\]\nuse crate::commands::ToolError;\n?",
    "",
    text,
)
if standalone_imports > 1:
    raise SystemExit(
        "legacy helper finalizer: duplicate standalone ToolError imports were found"
    )

for function_name, expected_visibility in HELPERS:
    adjacency = f"#[cfg(test)]\n{expected_visibility}fn {function_name}("
    if text.count(adjacency) != 1:
        raise SystemExit(
            f"legacy helper finalizer: {function_name} is not exactly test-only"
        )

remaining_unqualified = re.findall(
    r"(?<![A-Za-z0-9_:])ToolError\b",
    text,
)
if remaining_unqualified:
    raise SystemExit(
        "legacy helper finalizer: planner_redaction.rs retains unqualified ToolError"
    )

refreshed_commands_import = commands_import_pattern.search(text)
if refreshed_commands_import is None:
    raise SystemExit("legacy helper finalizer: commands import block disappeared")
if re.search(r"\bToolError\b", refreshed_commands_import.group("body")):
    raise SystemExit(
        "legacy helper finalizer: planner_redaction.rs retains a grouped ToolError import"
    )

PATH.write_text(text)
