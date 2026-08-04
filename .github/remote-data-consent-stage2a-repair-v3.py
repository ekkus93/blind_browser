from __future__ import annotations

import re
from pathlib import Path


def replace_at_most_one(path: str, old: str, new: str, label: str) -> None:
    file_path = Path(path)
    text = file_path.read_text()
    count = text.count(old)
    if count > 1:
        raise SystemExit(f"{label}: expected at most one occurrence, found {count}")
    if count == 1:
        file_path.write_text(text.replace(old, new, 1))


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
    raise SystemExit("source repair: unmatched Rust brace")


def box_struct_field(source: str, struct_name: str, target_type: str) -> str:
    definition = re.search(
        rf"pub\(crate\) struct {re.escape(struct_name)}\s*\{{",
        source,
    )
    if definition is None:
        raise SystemExit(f"boxed field repair: {struct_name} definition was not found")
    definition_open = source.find("{", definition.start())
    definition_close = find_matching_brace(source, definition_open)
    definition_body = source[definition_open + 1 : definition_close]
    field_matches = list(
        re.finditer(
            rf"(?m)^\s*(?:pub\(crate\)\s+)?([A-Za-z_][A-Za-z0-9_]*):\s*({re.escape(target_type)}),\s*$",
            definition_body,
        )
    )
    if len(field_matches) != 1:
        raise SystemExit(
            f"boxed field repair: expected one {target_type} field in {struct_name}, "
            f"found {len(field_matches)}"
        )
    field_name = field_matches[0].group(1)
    old_type = field_matches[0].group(2)
    absolute_type_start = definition_open + 1 + field_matches[0].start(2)
    absolute_type_end = definition_open + 1 + field_matches[0].end(2)
    source = (
        source[:absolute_type_start]
        + f"Box<{old_type}>"
        + source[absolute_type_end:]
    )

    search_from = definition_close + len("Box<>")
    constructor_count = 0
    needle = f"{struct_name} {{"
    while True:
        occurrence = source.find(needle, search_from)
        if occurrence == -1:
            break
        open_index = source.find("{", occurrence)
        close_index = find_matching_brace(source, open_index)
        block = source[open_index + 1 : close_index]
        shorthand = re.compile(rf"(?m)^(\s*){re.escape(field_name)},\s*$")
        explicit_simple = re.compile(
            rf"(?m)^(\s*){re.escape(field_name)}:\s*([A-Za-z_][A-Za-z0-9_]*),\s*$"
        )
        if shorthand.search(block):
            block, changed = shorthand.subn(
                rf"\1{field_name}: Box::new({field_name}),",
                block,
                count=1,
            )
            constructor_count += changed
        elif explicit_simple.search(block):
            block, changed = explicit_simple.subn(
                lambda match: (
                    f"{match.group(1)}{field_name}: Box::new({match.group(2)}),"
                ),
                block,
                count=1,
            )
            constructor_count += changed
        source = source[: open_index + 1] + block + source[close_index:]
        search_from = open_index + 1 + len(block) + 1

    if constructor_count == 0:
        raise SystemExit(
            f"boxed field repair: no {struct_name}.{field_name} constructor was updated"
        )
    return source


replace_at_most_one(
    "src-tauri/src/app_core/planner_redaction.rs",
    "pub(crate) pub(crate) fn high_risk_context_reason",
    "pub(crate) fn high_risk_context_reason",
    "duplicate high-risk helper visibility",
)
replace_at_most_one(
    "src-tauri/src/commands/contracts/planner.rs",
    "use crate::app_core::remote_data_consent::RemotePlannerConsentChallenge;\n",
    "",
    "duplicate private consent challenge import",
)

consent_path = Path("src-tauri/src/app_core/remote_data_consent.rs")
consent = consent_path.read_text()

bad_type_count = consent.count("PersistedRemotePlannerPrivacySettings")
if bad_type_count > 1:
    raise SystemExit(
        "privacy settings type repair: expected at most one bad type, "
        f"found {bad_type_count}"
    )
if bad_type_count == 1:
    consent = consent.replace(
        "PersistedRemotePlannerPrivacySettings",
        "RemotePlannerPrivacySettings",
        1,
    )

lifetime_pattern = re.compile(
    r"fn matching_grant\(\s*"
    r"grants: &\[RemotePlannerEphemeralGrant\],\s*"
    r"draft: &RemotePlannerRequestDraft,\s*"
    r"challenge_digest: Option<&str>,\s*"
    r"now_ms: u64,\s*"
    r"\) -> Option<&RemotePlannerEphemeralGrant> \{"
)
lifetime_matches = list(lifetime_pattern.finditer(consent))
if len(lifetime_matches) > 1:
    raise SystemExit(
        "matching grant lifetime repair: expected at most one signature, "
        f"found {len(lifetime_matches)}"
    )
if len(lifetime_matches) == 1:
    consent = lifetime_pattern.sub(
        "fn matching_grant<'a>(\n"
        "    grants: &'a [RemotePlannerEphemeralGrant],\n"
        "    draft: &RemotePlannerRequestDraft,\n"
        "    challenge_digest: Option<&str>,\n"
        "    now_ms: u64,\n"
        ") -> Option<&'a RemotePlannerEphemeralGrant> {",
        consent,
        count=1,
    )

tool_name_pattern = re.compile(
    r"(use crate::commands::\{[^}]*?ToolError),\s*ToolName,([^}]*?\};)",
    re.DOTALL,
)
tool_name_matches = list(tool_name_pattern.finditer(consent))
if len(tool_name_matches) > 1:
    raise SystemExit(
        "unused ToolName import repair: expected at most one import block, "
        f"found {len(tool_name_matches)}"
    )
if len(tool_name_matches) == 1:
    consent = tool_name_pattern.sub(r"\1,\2", consent, count=1)

endpoint_display_old = "endpoint_display: sanitize_url_for_display(endpoint),"
endpoint_display_count = consent.count(endpoint_display_old)
if endpoint_display_count > 1:
    raise SystemExit(
        "endpoint display repair: expected at most one constructor, "
        f"found {endpoint_display_count}"
    )
if endpoint_display_count == 1:
    consent = consent.replace(
        endpoint_display_old,
        "endpoint_display: crate::provider_endpoint::ProviderEndpointScope::parse(endpoint)\n"
        "            .map(|scope| scope.normalized_base_url().to_string())\n"
        "            .unwrap_or_else(|_| String::from(\"invalid remote endpoint\")),",
        1,
    )

stale_sanitizer_import = (
    "use crate::diagnostic_redaction::sanitize_url_for_display;\n"
)
stale_sanitizer_count = consent.count(stale_sanitizer_import)
if stale_sanitizer_count > 1:
    raise SystemExit(
        "stale sanitizer import repair: expected at most one import, "
        f"found {stale_sanitizer_count}"
    )
if stale_sanitizer_count == 1:
    consent = consent.replace(stale_sanitizer_import, "", 1)

consent = box_struct_field(
    consent,
    "RemotePlannerRequestDraft",
    "RemotePlannerInput",
)
boxed_input_move = "sanitized_input: self.sanitized_input,"
boxed_input_move_count = consent.count(boxed_input_move)
if boxed_input_move_count > 1:
    raise SystemExit(
        "boxed input move repair: expected at most one prepared-request move, "
        f"found {boxed_input_move_count}"
    )
if boxed_input_move_count == 1:
    consent = consent.replace(
        boxed_input_move,
        "sanitized_input: *self.sanitized_input,",
        1,
    )
consent_path.write_text(consent)

remote_planner_path = Path("src-tauri/src/app_core/remote_planner.rs")
remote_planner = remote_planner_path.read_text()
profile_helper_signature = "pub(crate) fn remote_planner_profile_snapshot("
profile_helper_count = remote_planner.count(profile_helper_signature)
if profile_helper_count > 1:
    raise SystemExit(
        "remote planner profile snapshot repair: expected at most one helper, "
        f"found {profile_helper_count}"
    )
if profile_helper_count == 0:
    helper = '''impl crate::AppCore {
    /// Snapshot the configured remote planner profile under the `AppCore` lock so
    /// network preparation can run against an owned, immutable copy.
    pub(crate) fn remote_planner_profile_snapshot(
        &self,
    ) -> Result<(String, crate::config::RemotePlannerProfile), ToolError> {
        let Some(profile_name) = self.config.providers.planner.remote_profile.as_deref() else {
            return Err(planner_interpretation_unavailable_error(
                "planner_profile_unavailable",
                "remote planner mode requires a configured planner profile",
                false,
                None,
            ));
        };
        let Some(profile) = self.config.remote_planner_profiles.get(profile_name) else {
            return Err(planner_interpretation_unavailable_error(
                "planner_profile_unavailable",
                format!("configured remote planner profile '{profile_name}' was not found"),
                false,
                None,
            ));
        };
        Ok((profile_name.to_string(), profile.clone()))
    }
}
'''
    remote_planner_path.write_text(remote_planner.rstrip() + "\n\n" + helper)

replanning_path = Path("src-tauri/src/app_core/replanning.rs")
replanning = replanning_path.read_text()
outcome_match = re.search(
    r"pub\(crate\) enum ResolvePlanOutcome\s*\{(?P<body>.*?)\n\}",
    replanning,
    re.DOTALL,
)
if outcome_match is None:
    raise SystemExit("replanning test repair: ResolvePlanOutcome enum was not found")
planner_variants = re.findall(
    r"([A-Za-z_][A-Za-z0-9_]*)\s*\(\s*PlannerOutput\s*\)",
    outcome_match.group("body"),
)
if len(planner_variants) != 1:
    raise SystemExit(
        "replanning test repair: expected one PlannerOutput variant, "
        f"found {planner_variants}"
    )
planner_variant = planner_variants[0]

planner_tests_path = Path("src-tauri/src/app_core/tests/planner_tests.rs")
planner_tests = planner_tests_path.read_text()
old_test_return = ") -> Result<PlannerOutput, crate::commands::ToolError> {"
new_test_return = (
    ") -> Result<crate::app_core::replanning::ResolvePlanOutcome, "
    "crate::commands::ToolError> {"
)
old_test_return_count = planner_tests.count(old_test_return)
if old_test_return_count > 1:
    raise SystemExit(
        "replanning test return repair: expected at most one old signature, "
        f"found {old_test_return_count}"
    )
if old_test_return_count == 1:
    planner_tests = planner_tests.replace(old_test_return, new_test_return, 1)

old_test_result = "        self.resolve_results.remove(0)\n"
new_test_result = (
    "        self.resolve_results\n"
    "            .remove(0)\n"
    "            .map(crate::app_core::replanning::ResolvePlanOutcome::"
    + planner_variant
    + ")\n"
)
old_test_result_count = planner_tests.count(old_test_result)
if old_test_result_count > 1:
    raise SystemExit(
        "replanning test result repair: expected at most one old return, "
        f"found {old_test_result_count}"
    )
if old_test_result_count == 1:
    planner_tests = planner_tests.replace(old_test_result, new_test_result, 1)
planner_tests_path.write_text(planner_tests)

redaction_path = Path("src-tauri/src/app_core/planner_redaction.rs")
redaction = redaction_path.read_text()
legacy_helper_names = [
    "sanitize_remote_planner_input",
    "enforce_remote_planner_privacy",
    "privacy_error",
]

for function_name in legacy_helper_names:
    declaration_pattern = re.compile(
        rf"(?m)^(?P<indent>[ \t]*)(?P<visibility>pub\(crate\)\s+)?fn {function_name}\("
    )
    declaration_matches = list(declaration_pattern.finditer(redaction))
    if len(declaration_matches) != 1:
        raise SystemExit(
            f"test-only legacy helper repair: expected one {function_name} declaration, "
            f"found {len(declaration_matches)}"
        )
    declaration = declaration_matches[0]
    indent = declaration.group("indent")
    visibility = declaration.group("visibility") or ""
    declaration_start = declaration.start()
    cfg_line = f"{indent}#[cfg(test)]\n"
    prefix = redaction[:declaration_start]
    if prefix.endswith(cfg_line):
        prefix = prefix[: -len(cfg_line)]
    replacement = f"{cfg_line}{indent}{visibility}fn {function_name}("
    redaction = prefix + replacement + redaction[declaration.end():]

for function_name in legacy_helper_names:
    declaration_match = re.search(
        rf"(?m)^[ \t]*(?:pub\(crate\)\s+)?fn {function_name}\(",
        redaction,
    )
    if declaration_match is None:
        raise SystemExit(
            f"legacy helper ToolError repair: {function_name} declaration was not found"
        )
    open_index = redaction.find("{", declaration_match.start())
    if open_index == -1:
        raise SystemExit(
            f"legacy helper ToolError repair: {function_name} body was not found"
        )
    close_index = find_matching_brace(redaction, open_index)
    function_source = redaction[declaration_match.start(): close_index + 1]
    function_source = re.sub(
        r"(?<!crate::commands::)\bToolError\b",
        "crate::commands::ToolError",
        function_source,
    )
    redaction = (
        redaction[:declaration_match.start()]
        + function_source
        + redaction[close_index + 1:]
    )

for import_line in [
    "use super::remote_data_consent::{evaluate_remote_planner_policy, RemotePlannerPolicyResult};\n",
    "use crate::config::RemotePlannerPrivacySettings;\n",
    "use crate::provider_endpoint::ProviderEndpointScope;\n",
]:
    if import_line in redaction and f"#[cfg(test)]\n{import_line}" not in redaction:
        redaction = redaction.replace(import_line, f"#[cfg(test)]\n{import_line}", 1)

commands_import_pattern = re.compile(
    r"use crate::commands::\{(?P<body>.*?)\n\};",
    re.DOTALL,
)
commands_import_match = commands_import_pattern.search(redaction)
if commands_import_match is None:
    raise SystemExit("test-only legacy helper repair: commands import block was not found")
commands_body = commands_import_match.group("body")
if re.search(r"\bToolError\b", commands_body):
    commands_body, tool_error_count = re.subn(
        r",?\s*ToolError,?",
        lambda match: "," if "," in match.group(0) else "",
        commands_body,
        count=1,
    )
    if tool_error_count != 1:
        raise SystemExit("test-only legacy helper repair: ToolError import mismatch")
    commands_body = re.sub(r",\s*,", ",", commands_body)
    commands_body = re.sub(r",\s*$", "", commands_body)
    redaction = (
        redaction[: commands_import_match.start("body")]
        + commands_body
        + redaction[commands_import_match.end("body") :]
    )

redaction, test_tool_error_import_count = re.subn(
    r"(?m)^#\[cfg\(test\)\]\nuse crate::commands::ToolError;\n?",
    "",
    redaction,
)
if test_tool_error_import_count > 1:
    raise SystemExit(
        "test-only legacy helper repair: duplicate ToolError test imports were found"
    )

for function_name in legacy_helper_names:
    adjacency_pattern = re.compile(
        rf"(?m)^#\[cfg\(test\)\]\n(?:pub\(crate\)\s+)?fn {function_name}\("
    )
    if len(adjacency_pattern.findall(redaction)) != 1:
        raise SystemExit(
            f"test-only legacy helper repair: {function_name} lacks one adjacent cfg(test)"
        )

redaction_path.write_text(redaction)
