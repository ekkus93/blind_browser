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


def replace_once(source: str, old: str, new: str, label: str) -> str:
    count = source.count(old)
    if count != 1:
        raise SystemExit(
            f"generated-layout finalizer: expected one {label}, found {count}"
        )
    return source.replace(old, new, 1)


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
    pattern = re.compile(
        rf"(?m)^(\s*){re.escape(variant_name)}\(\s*"
        rf"{re.escape(payload_type)}[ \t]*\),[ \t]*$"
    )
    matches = list(pattern.finditer(enum_body))
    if len(matches) != 1:
        raise SystemExit(
            f"generated-layout finalizer: expected one {enum_name}::{variant_name} "
            f"definition, found {len(matches)}"
        )

    match = matches[0]
    enum_body = (
        enum_body[: match.start()]
        + f"{match.group(1)}{variant_name}(Box<{payload_type}>),"
        + enum_body[match.end() :]
    )
    source = source[: enum_open + 1] + enum_body + source[enum_close:]

    prefix = f"{enum_name}::{variant_name}("
    starts: list[int] = []
    offset = enum_open + len(enum_body) + 2
    while True:
        occurrence = source.find(prefix, offset)
        if occurrence < 0:
            break
        starts.append(occurrence)
        offset = occurrence + len(prefix)

    if len(starts) != expected_constructors:
        raise SystemExit(
            f"generated-layout finalizer: expected {expected_constructors} "
            f"{enum_name}::{variant_name} constructors, found {len(starts)}"
        )

    for occurrence in reversed(starts):
        open_index = occurrence + len(prefix) - 1
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


def box_consent_required(source: str) -> str:
    source = replace_once(
        source,
        """    ConsentRequired {
        challenge: RemotePlannerConsentChallenge,
        draft: RemotePlannerRequestDraft,
    },
""",
        """    ConsentRequired {
        challenge: Box<RemotePlannerConsentChallenge>,
        draft: Box<RemotePlannerRequestDraft>,
    },
""",
        "RemotePlannerPreparation::ConsentRequired definition",
    )
    return replace_once(
        source,
        "Ok(RemotePlannerPreparation::ConsentRequired { challenge, draft })",
        "Ok(RemotePlannerPreparation::ConsentRequired {\n"
        "                    challenge: Box::new(challenge),\n"
        "                    draft: Box::new(draft),\n"
        "                })",
        "RemotePlannerPreparation::ConsentRequired constructor",
    )


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
    before = source[:start].rstrip()
    after = source[block_end:].strip()
    pieces = [piece for piece in (before, after, block.rstrip()) if piece]
    moved = "\n\n".join(pieces) + "\n"
    if moved.rfind(marker) < moved.rfind("impl crate::AppCore"):
        raise SystemExit(
            "generated-layout finalizer: remote_planner test module did not move "
            "after the appended AppCore implementation"
        )
    return moved


# Repair generated enum layouts without suppressing Clippy. Boxes cross only
# orchestration boundaries; the underlying request/challenge values remain owned.
consent = CONSENT_PATH.read_text()
consent = box_enum_tuple_variant(
    consent,
    enum_name="RemotePlannerPreparation",
    variant_name="Authorized",
    payload_type="PreparedRemotePlannerRequest",
    expected_constructors=1,
)
consent = box_consent_required(consent)
consent = box_enum_tuple_variant(
    consent,
    enum_name="PendingConsentResolution",
    variant_name="Authorized",
    payload_type="AuthorizedPendingRemotePlannerRequest",
    expected_constructors=3,
)
CONSENT_PATH.write_text(consent)

orchestrator = ORCHESTRATOR_PATH.read_text()
orchestrator = replace_once(
    orchestrator,
    "        prepared: super::remote_data_consent::PreparedRemotePlannerRequest,\n",
    "        prepared: Box<super::remote_data_consent::PreparedRemotePlannerRequest>,\n",
    "ResolvePhase::Remote.prepared field",
)
orchestrator = replace_once(
    orchestrator,
    "                    RemotePlannerPreparation::ConsentRequired { challenge, draft } => {\n"
    "                        guard.store_pending_remote_planner_consent(\n",
    "                    RemotePlannerPreparation::ConsentRequired { challenge, draft } => {\n"
    "                        let challenge = *challenge;\n"
    "                        let draft = *draft;\n"
    "                        guard.store_pending_remote_planner_consent(\n",
    "remote planner consent-required ownership boundary",
)
orchestrator = replace_once(
    orchestrator,
    "        PendingConsentResolution::Authorized(ready) => ready,\n",
    "        PendingConsentResolution::Authorized(ready) => *ready,\n",
    "pending-consent authorization boundary",
)
ORCHESTRATOR_PATH.write_text(orchestrator)

remote_planner = REMOTE_PLANNER_PATH.read_text()
REMOTE_PLANNER_PATH.write_text(move_test_module_to_end(remote_planner))

text = PLANNER_REDACTION_PATH.read_text()
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

    prefix = text[: match.start()]
    adjacent_cfg = re.search(r"(?m)^[ \t]*#\[cfg\(test\)\]\n$", prefix)
    if adjacent_cfg is not None and adjacent_cfg.end() == len(prefix):
        prefix = prefix[: adjacent_cfg.start()]
    text = (
        prefix
        + f"#[cfg(test)]\n{expected_visibility}fn {function_name}("
        + text[match.end() :]
    )

commands_import_pattern = re.compile(
    r"use crate::commands::\{(?P<body>.*?)\n\};",
    re.DOTALL,
)
commands_import_match = commands_import_pattern.search(text)
if commands_import_match is None:
    raise SystemExit("legacy helper finalizer: commands import block was not found")

commands_body, removed_imports = re.subn(
    r"(?<![A-Za-z0-9_])ToolError\s*,\s*",
    "",
    commands_import_match.group("body"),
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

text, qualified_count = re.subn(
    r"(?<![A-Za-z0-9_:])ToolError\b",
    "crate::commands::ToolError",
    text,
)
if qualified_count == 0:
    raise SystemExit(
        "legacy helper finalizer: no standalone ToolError use was available to qualify"
    )

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

if re.search(r"(?<![A-Za-z0-9_:])ToolError\b", text):
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
