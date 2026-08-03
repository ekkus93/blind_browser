#!/usr/bin/env python3
from __future__ import annotations

import sys
from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    content = target.read_text(encoding="utf-8")
    count = content.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one integration match, found {count}: {old[:80]!r}")
    target.write_text(content.replace(old, new, 1), encoding="utf-8")


def initializer_end(lines: list[str], start: int, path: Path, struct_name: str) -> int:
    depth = 0
    opened = False
    for index in range(start, len(lines)):
        line = lines[index]
        opening = line.count("{")
        closing = line.count("}")
        if opening:
            opened = True
        depth += opening - closing
        if opened and depth == 0:
            return index
    raise SystemExit(f"{path}: unterminated {struct_name} initializer at line {start + 1}")


def ensure_initializer_fields(
    path: Path,
    struct_name: str,
    required_fields: tuple[tuple[str, str], ...],
) -> tuple[int, int]:
    lines = path.read_text(encoding="utf-8").splitlines()
    needle = f"{struct_name} {{"
    total = 0
    modified = 0
    index = 0

    while index < len(lines):
        if needle not in lines[index]:
            index += 1
            continue

        total += 1
        end = initializer_end(lines, index, path, struct_name)
        block = "\n".join(lines[index : end + 1])
        missing = [rendered for field, rendered in required_fields if f"{field}:" not in block]
        if missing:
            closing_indent = lines[end][: len(lines[end]) - len(lines[end].lstrip())]
            insertion = [f"{closing_indent}    {rendered}" for rendered in missing]
            lines[end:end] = insertion
            modified += 1
            end += len(insertion)
        index = end + 1

    if modified:
        path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    # Fail closed if any initializer is still missing a required field.
    verification = path.read_text(encoding="utf-8").splitlines()
    index = 0
    while index < len(verification):
        if needle not in verification[index]:
            index += 1
            continue
        end = initializer_end(verification, index, path, struct_name)
        block = "\n".join(verification[index : end + 1])
        absent = [field for field, _ in required_fields if f"{field}:" not in block]
        if absent:
            raise SystemExit(
                f"{path}: incomplete {struct_name} initializer at line {index + 1}: {absent}"
            )
        index = end + 1

    return total, modified


def migrate_command_test_contracts() -> None:
    root = Path("src-tauri/src/commands/tests")
    if not root.is_dir():
        raise SystemExit(f"missing command test root: {root}")

    specifications = (
        (
            "RemoteTtsSettings",
            (
                ("endpoint_is_loopback", "endpoint_is_loopback: None,"),
                ("availability_reason", "availability_reason: None,"),
            ),
            13,
        ),
        (
            "RemoteAsrSettings",
            (
                ("endpoint_is_loopback", "endpoint_is_loopback: None,"),
                ("availability_reason", "availability_reason: None,"),
            ),
            13,
        ),
        (
            "GetRuntimeStatusData",
            (("skill_discovery_diagnostics", "skill_discovery_diagnostics: Default::default(),"),),
            6,
        ),
    )

    totals = {name: [0, 0] for name, _, _ in specifications}
    for path in sorted(root.rglob("*.rs")):
        for struct_name, fields, _minimum in specifications:
            total, modified = ensure_initializer_fields(path, struct_name, fields)
            totals[struct_name][0] += total
            totals[struct_name][1] += modified

    for struct_name, _fields, minimum in specifications:
        total, modified = totals[struct_name]
        if total < minimum:
            raise SystemExit(
                f"test fixture migration found only {total} {struct_name} initializers; "
                f"expected at least {minimum}"
            )
        print(f"Migrated {modified}/{total} {struct_name} test initializers")

    skill_selection_path = Path("src-tauri/src/commands/tests/skill_selection.rs")
    skill_selection = skill_selection_path.read_text(encoding="utf-8")
    old_iteration = """    let matching_skills = loaded_skills
        .iter()
"""
    new_iteration = """    let matching_skills = loaded_skills
        .skills
        .iter()
"""
    if new_iteration not in skill_selection:
        count = skill_selection.count(old_iteration)
        if count != 1:
            raise SystemExit(
                "skill selection: expected one DiscoveredSkills iteration migration, "
                f"found {count}"
            )
        skill_selection_path.write_text(
            skill_selection.replace(old_iteration, new_iteration, 1),
            encoding="utf-8",
        )
    print("Migrated DiscoveredSkills test iteration")


def prepare_generator() -> None:
    generator_path = Path(".github/post_p8_enforcement_patch.py")
    generator = generator_path.read_text(encoding="utf-8")

    old_skill = '''for _ in range(2):
    replace_once(
        "src-tauri/src/commands/skill_loader.rs",
        """            &available_tool_names,
            &mut discovered,
        );
""",
        """            &available_tool_names,
            &mut discovered,
            &mut diagnostics,
        );
""",
    )
'''
    new_skill = '''skill_loader_calls = read("src-tauri/src/commands/skill_loader.rs")
old_skill_loader_call = """            &available_tool_names,
            &mut discovered,
        );
"""
new_skill_loader_call = """            &available_tool_names,
            &mut discovered,
            &mut diagnostics,
        );
"""
count = skill_loader_calls.count(old_skill_loader_call)
if count != 2:
    raise RuntimeError(
        f"src-tauri/src/commands/skill_loader.rs: expected two discovery calls, found {count}"
    )
write(
    "src-tauri/src/commands/skill_loader.rs",
    skill_loader_calls.replace(old_skill_loader_call, new_skill_loader_call),
)
'''
    if generator.count(old_skill) != 1:
        raise SystemExit("skill-loader duplicate-edit block was not found exactly once")
    generator = generator.replace(old_skill, new_skill, 1)

    old_provider = '''for _ in range(2):
    replace_once(
        "src-tauri/src/command_handlers/provider_handlers.rs",
        """    let settings = app_core.current_remote_planner_settings();
    Ok(RemotePlannerConnectionSettingsData {
        profile_name,
        base_url: settings.base_url.unwrap_or_default(),
        model: settings.model.unwrap_or_default(),
    })
""",
        """    let settings = app_core.current_remote_planner_settings();
    completed_remote_planner_connection_settings(profile_name, settings)
""",
    )
'''
    new_provider = '''provider_handler_calls = read("src-tauri/src/command_handlers/provider_handlers.rs")
old_provider_handler_call = """    let settings = app_core.current_remote_planner_settings();
    Ok(RemotePlannerConnectionSettingsData {
        profile_name,
        base_url: settings.base_url.unwrap_or_default(),
        model: settings.model.unwrap_or_default(),
    })
"""
new_provider_handler_call = """    let settings = app_core.current_remote_planner_settings();
    completed_remote_planner_connection_settings(profile_name, settings)
"""
count = provider_handler_calls.count(old_provider_handler_call)
if count != 2:
    raise RuntimeError(
        f"src-tauri/src/command_handlers/provider_handlers.rs: expected two response blocks, found {count}"
    )
write(
    "src-tauri/src/command_handlers/provider_handlers.rs",
    provider_handler_calls.replace(old_provider_handler_call, new_provider_handler_call),
)
'''
    if generator.count(old_provider) != 1:
        raise SystemExit("provider duplicate-edit block was not found exactly once")
    generator = generator.replace(old_provider, new_provider, 1)

    old_fill = '''replace_once(
    "src-tauri/src/app_core/fill_correction.rs",
    """                .and_then(|candidate_id| {
                    resolve_typeable_element(current_page, candidate_id)
                        .ok()
                        .map(|_| candidate_id.clone())
                });
""",
    """                .find(|candidate_id| {
                    resolve_typeable_element(current_page, candidate_id).is_ok()
                })
                .cloned();
""",
)
'''
    new_fill = '''replace_once(
    "src-tauri/src/app_core/fill_correction.rs",
    """            let alternate_element_id = context
                .candidate_element_ids
                .iter()
                .find(|candidate_id| candidate_id.as_str() != active_element_id)
                .and_then(|candidate_id| {
                    resolve_typeable_element(current_page, candidate_id)
                        .ok()
                        .map(|_| candidate_id.clone())
                });
""",
    """            let alternate_element_id = context
                .candidate_element_ids
                .iter()
                .find(|candidate_id| {
                    candidate_id.as_str() != active_element_id
                        && resolve_typeable_element(current_page, candidate_id.as_str()).is_ok()
                })
                .cloned();
""",
)
'''
    if generator.count(old_fill) != 1:
        raise SystemExit("fill-correction generator block was not found exactly once")
    generator = generator.replace(old_fill, new_fill, 1)

    generator_path.write_text(generator, encoding="utf-8")
    print("Prepared temporary generator exact-count and iterator repairs")


def repair_output() -> None:
    replace_once("src-tauri/src/diagnostic_redaction.rs", ", ''',", ",")
    diagnostic_path = Path("src-tauri/src/diagnostic_redaction.rs")
    diagnostic_lines = diagnostic_path.read_text(encoding="utf-8").splitlines()
    print("Generated diagnostic_redaction.rs lines 40-75:")
    for line_number in range(40, min(76, len(diagnostic_lines) + 1)):
        print(f"{line_number:04d}: {diagnostic_lines[line_number - 1]}")

    replace_once(
        "src-tauri/src/app_core/settings_adapters.rs",
        '''            sanitize_url_for_display(base_url)
                .map(|safe| safe.value)
                .or_else(|| Some(String::from("[REDACTED INVALID ENDPOINT]"))),
''',
        '''            Some(
                sanitize_url_for_display(base_url)
                    .map(|safe| safe.value)
                    .unwrap_or_else(|| String::from("[REDACTED INVALID ENDPOINT]")),
            ),
''',
    )

    state_path = Path("src-tauri/src/app_core/state_snapshots.rs")
    state = state_path.read_text(encoding="utf-8")
    diagnostics_line = (
        "            skill_discovery_diagnostics: "
        "self.last_skill_discovery_diagnostics.clone(),\n"
    )
    if state.count(diagnostics_line) != 1:
        raise SystemExit(
            "state snapshots: expected one misplaced diagnostics field, "
            f"found {state.count(diagnostics_line)}"
        )
    state = state.replace(diagnostics_line, "", 1)
    runtime_tail = '''            } else {
                None
            },
        }
    }
'''
    runtime_replacement = '''            } else {
                None
            },
            skill_discovery_diagnostics: self.last_skill_discovery_diagnostics.clone(),
        }
    }
'''
    if state.count(runtime_tail) != 1:
        raise SystemExit(
            f"state snapshots: expected one runtime status tail, found {state.count(runtime_tail)}"
        )
    state_path.write_text(state.replace(runtime_tail, runtime_replacement, 1), encoding="utf-8")

    replace_once(
        "src-tauri/src/app_core/planner_redaction.rs",
        '''                audio_format: None,
                timeout_ms: None,
            },
            remote_asr_settings: RemoteAsrSettings {
''',
        '''                audio_format: None,
                timeout_ms: None,
                endpoint_is_loopback: None,
                availability_reason: None,
            },
            remote_asr_settings: RemoteAsrSettings {
''',
    )
    replace_once(
        "src-tauri/src/app_core/planner_redaction.rs",
        '''                temperature_milli: None,
                timeout_ms: None,
            },
            provider_failover_settings: ProviderFailoverSettings {
''',
        '''                temperature_milli: None,
                timeout_ms: None,
                endpoint_is_loopback: None,
                availability_reason: None,
            },
            provider_failover_settings: ProviderFailoverSettings {
''',
    )

    replace_once(
        "src-tauri/src/app_core/tests/settings_tests.rs",
        '''        assert!(!displayed.contains("user"));
        assert!(!displayed.contains("pass"));
        assert!(!displayed.contains('?'));
        assert!(!displayed.contains('#'));
''',
        '''        assert!(!displayed.contains("user:pass@"));
        assert!(!displayed.contains('@'));
        assert!(!displayed.contains("token=secret"));
        assert!(!displayed.contains("code=secret"));
        assert!(!displayed.contains('?'));
        assert!(!displayed.contains('#'));
''',
    )

    migrate_command_test_contracts()

    # src/tauri-types.ts does not define GetRuntimeStatusData; only shared provider
    # and agent types are maintained there. The generator already adds the shared
    # SkillDiscoveryDiagnostics and TTS/ASR endpoint fields, so no status-field
    # insertion is required on the TypeScript side.
    print("Repaired generated Rust integration output")


def main() -> int:
    if sys.argv[1:] == ["--prepare"]:
        prepare_generator()
        return 0
    if sys.argv[1:] == ["--repair"]:
        repair_output()
        return 0
    print("usage: post_p8_enforcement_repair.py --prepare|--repair", file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
