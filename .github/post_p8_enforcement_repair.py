#!/usr/bin/env python3
from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    target = Path(path)
    content = target.read_text(encoding="utf-8")
    count = content.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one integration match, found {count}: {old[:80]!r}")
    target.write_text(content.replace(old, new, 1), encoding="utf-8")


# Repair duplicate source patterns in the temporary generator before applying it.
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

# The generator must run before the repairs below.
print("Prepared temporary generator exact-count repairs")
