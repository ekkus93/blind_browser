from __future__ import annotations

import re
from pathlib import Path

PATH = Path("src-tauri/src/app_core/planner_redaction.rs")
HELPERS = (
    ("sanitize_remote_planner_input", "pub(crate) "),
    ("enforce_remote_planner_privacy", ""),
    ("privacy_error", ""),
)


def find_matching_brace(source: str, open_index: int) -> int:
    depth = 0
    in_string = False
    escaped = False
    for index in range(open_index, len(source)):
        char = source[index]
        if in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
            continue
        if char == '"':
            in_string = True
        elif char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return index
    raise SystemExit("legacy helper finalizer: unmatched Rust brace")


text = PATH.read_text()

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
    cfg_pattern = re.compile(r"(?m)^[ \t]*#\[cfg\(test\)\]\n$")
    cfg_match = cfg_pattern.search(prefix)
    if cfg_match is not None and cfg_match.end() == len(prefix):
        prefix = prefix[: cfg_match.start()]

    declaration = (
        f"#[cfg(test)]\n{expected_visibility}fn {function_name}("
    )
    text = prefix + declaration + text[match.end():]

    declaration_start = len(prefix) + len("#[cfg(test)]\n")
    open_index = text.find("{", declaration_start)
    if open_index == -1:
        raise SystemExit(
            f"legacy helper finalizer: {function_name} body was not found"
        )
    close_index = find_matching_brace(text, open_index)
    function_source = text[declaration_start : close_index + 1]
    function_source = function_source.replace(
        "crate::commands::ToolError", "__STAGE2A_TOOL_ERROR__"
    )
    function_source = function_source.replace(
        "ToolError", "crate::commands::ToolError"
    )
    function_source = function_source.replace(
        "__STAGE2A_TOOL_ERROR__", "crate::commands::ToolError"
    )
    text = (
        text[:declaration_start]
        + function_source
        + text[close_index + 1 :]
    )

text = re.sub(
    r"(?m)^#\[cfg\(test\)\]\nuse crate::commands::ToolError;\n?",
    "",
    text,
)

for function_name, expected_visibility in HELPERS:
    adjacency = (
        f"#[cfg(test)]\n{expected_visibility}fn {function_name}("
    )
    if text.count(adjacency) != 1:
        raise SystemExit(
            f"legacy helper finalizer: {function_name} is not exactly test-only"
        )
    declaration_start = text.index(adjacency) + len("#[cfg(test)]\n")
    open_index = text.find("{", declaration_start)
    close_index = find_matching_brace(text, open_index)
    function_source = text[declaration_start : close_index + 1]
    unqualified = re.findall(
        r"(?<!crate::commands::)\bToolError\b", function_source
    )
    if unqualified:
        raise SystemExit(
            f"legacy helper finalizer: {function_name} retains unqualified ToolError"
        )

PATH.write_text(text)
