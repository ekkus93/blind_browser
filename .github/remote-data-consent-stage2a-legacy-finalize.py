from __future__ import annotations

import re
from pathlib import Path

PLANNER_REDACTION_PATH = Path("src-tauri/src/app_core/planner_redaction.rs")
CONSENT_PATH = Path("src-tauri/src/app_core/remote_data_consent.rs")
ORCHESTRATOR_PATH = Path("src-tauri/src/app_core/replanning_orchestrator.rs")
REMOTE_PLANNER_PATH = Path("src-tauri/src/app_core/remote_planner.rs")

HELPERS = (
    ("sanitize_remote_planner_input", "pub(crate) "),
    ("enforce_remote_planner_privacy", ""),
    ("privacy_error", ""),
)


def find_matching_delimiter(
    source: str,
    open_index: int,
    opening: str,
    closing: str,
) -> int:
    if source[open_index] != opening:
        raise SystemExit(
            f"generated-layout finalizer: expected {opening!r} at index {open_index}"
        )

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
        elif char == opening:
            depth += 1
        elif char == closing:
            depth -= 1
            if depth == 0:
                return index

    raise SystemExit(
        f"generated-layout finalizer: unmatched {opening!r} at index {open_index}"
    )


def box_enum_tuple_variant(
    source: str,
    *,
    enum_name: str,
    variant_name: str,
    payload_type: str,
    expected_constructors: int,
) -> str:
    enum_match = re.search(
        rf"(?m)^(?:pub\(crate\)\s+)?enum {re.escape(enum_name)}\s*\{{",
        source,
    )
    if enum_match is None:
        raise SystemExit(
            f"generated-layout finalizer: enum {enum_name} was not found"
        )

    enum_open = source.find("{", enum_match.start())
    enum_close = find_matching_delimiter(source, enum_open, "{", "}")
    enum_body = source[enum_open + 1 : enum_close]
    variant_pattern = re.compile(
        rf"(?m)^(\s*){re.escape(variant_name)}\(\s*"
        rf"{re.escape(payload_type)}[ \t]*\),[ \t]*$"
    )
    variant_matches = list(variant_pattern.finditer(enum_body))
    if len(variant_matches) != 1:
        raise SystemExit(
            f"generated-layout finalizer: expected one {enum_name}::{variant_name} "
            f"definition, found {len(variant_matches)}"
        )

    variant_match = variant_matches[0]
    replacement = f"{variant_match.group(1)}{variant_name}(Box<{payload_type}>),"
    enum_body = (
        enum_body[: variant_match.start()]
        + replacement
        + enum_body[variant_match.end() :]
    )
    source = source[: enum_open + 1] + enum_body + source[enum_close:]

    constructor_prefix = f"{enum_name}::{variant_name}("
    constructor_starts: list[int] = []
    search_from = enum_open + 1 + len(enum_body) + 1
    while True:
        occurrence = source.find(constructor_prefix, search_from)
        if occurrence == -1:
            break
        constructor_starts.append(occurrence)
        search_from = occurrence + len(constructor_prefix)

    if len(constructor_starts) != expected_constructors:
        raise SystemExit(
            f"generated-layout finalizer: expected {expected_constructors} "
            f"{enum_name}::{variant_name} constructors, found "
            f"{len(constructor_starts)}"
        )

    for occurrence in reversed(constructor_starts):
        open_index = occurrence + len(constructor_prefix) - 1
        close_index = find_matching_delimiter(source, open_index, "(", ")")
        argument = source[open_index + 1 : close_index]
        if argument.lstrip().startswith("Box::new("):
            raise SystemExit(
                f"generated-layout finalizer: {enum_name}::{variant_name} "
                "constructor was already boxed"
            )
        source = (
            source[: open_index + 1]
            + "Box::new("
            + argument
            + ")"
            + source[close_index:]
        )

    return source


def box_resolve_phase_prepared(source: str) -> str:
    old = (
        "        prepared: "
        "super::remote_data_consent::PreparedRemotePlannerRequest,\n"
    )
    new = (
        "        prepared: "
        "Box<super::remote_data_consent::PreparedRemotePlannerRequest>,\n"
    )
    count = source.count(old)
    if count != 1:
        raise SystemExit(
            "generated-layout finalizer: expected one unboxed "
            f"ResolvePhase::Remote.prepared field, found {count}"
        )
    return source.replace(old, new, 1)


def unbox_pending_consent_at_boundary(source: str) -> str:
    old = "        PendingConsentResolution::Authorized(ready) => ready,\n"
    new = "        PendingConsentResolution::Authorized(ready) => *ready,\n"
    count = source.count(old)
    if count != 1:
        raise SystemExit(
            "generated-layout finalizer: expected one pending-consent "
            f"authorization boundary, found {count}"
        )
    return source.replace(old, new, 1)


def move_test_module_to_end(source: str) -> str:
    marker = "#[cfg(test)]\nmod tests {"
    starts = [match.start() for match in re.finditer(re.escape(marker), source)]
    if len(starts) != 1:
        raise SystemExit(
            "generated-layout finalizer: expected one remote_planner test module, "
            f"found {len(starts)}"
        )

    start = starts[0]
    open_index = source.find("{", start)
    close_index = find_matching_delimiter(source, open_index, "{", "}")
    block_end = close_index + 1
    while block_end < len(source) and source[block_end] == "\n":
        block_end += 1

    block = source[start : close_index + 1]
    without = (source[:start].rstrip() + "\n").rstrip() + "\n"
    tail = source[block_end:].strip()
    if tail:
        without += "\n" + tail + "\n"

    moved = without.rstrip() + "\n\n" + block.rstrip() + "\n"
    if moved.rfind(marker) < moved.rfind("impl crate::AppCore"):
        raise SystemExit(
            "generated-layout finalizer: remote_planner test module did not move "
            "after the appended AppCore implementation"
        )
    return moved


# Repair the generated enum layouts without suppressing Clippy. The authorized
# request box is carried directly into ResolvePhase, avoiding an extra allocation.
consent = CONSENT_PATH.read_text()
consent = box_enum_tuple_variant(
    consent,
    enum_name="RemotePlannerPreparation",
    variant_name="Authorized",
    payload_type="PreparedRemotePlannerRequest",
    expected_constructors=1,
)
consent = box_enum_tuple_variant(
    consent,
    enum_name="PendingConsentResolution",
    variant_name="Authorized",
    payload_type="AuthorizedPendingRemotePlannerRequest",
    expected_constructors=3,
)
CONSENT_PATH.write_text(consent)

orchestrator = ORCHESTRATOR_PATH.read_text()
orchestrator = box_resolve_phase_prepared(orchestrator)
orchestrator = unbox_pending_consent_at_boundary(orchestrator)
ORCHESTRATOR_PATH.write_text(orchestrator)

remote_planner = REMOTE_PLANNER_PATH.read_text()
REMOTE_PLANNER_PATH.write_text(move_test_module_to_end(remote_planner))

text = PLANNER_REDACTION_PATH.read_text()

# Make each compatibility helper unambiguously test-only. Rebuild the declaration
# prefix rather than stacking attributes left by an earlier repair attempt.
for function_name, expected_visibility in HELPERS:
    declaration_pattern = re.compile(
        rf"(?m)^(?P<indent>[ \t]*)(?P<visibility>pub\(crate\)\s+)?fn "
        rf"{function_name}\("
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
            f"legacy helper finalizer: {function_name} visibility was "
            f"{visibility!r}, expected {expected_visibility!r}"
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

PLANNER_REDACTION_PATH.write_text(text)
