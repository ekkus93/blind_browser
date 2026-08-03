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
    generator_path.write_text(generator.replace(old_provider, new_provider, 1), encoding="utf-8")
    print("Prepared temporary generator exact-count repairs")


def repair_output() -> None:
    replace_once("src-tauri/src/diagnostic_redaction.rs", ", ''',", ",")

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
        "src-tauri/src/app_core/fill_correction.rs",
        '''    let alternate_element_id = context
        .candidate_element_ids
        .iter()
        .find(|candidate_id| candidate_id.as_str() != active_element_id)
        .find(|candidate_id| resolve_typeable_element(current_page, candidate_id).is_ok())
''',
        '''    let alternate_element_id = context
        .candidate_element_ids
        .iter()
        .find(|candidate_id| {
            candidate_id.as_str() != active_element_id
                && resolve_typeable_element(current_page, candidate_id.as_str()).is_ok()
        })
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
