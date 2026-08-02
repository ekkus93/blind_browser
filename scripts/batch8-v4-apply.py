#!/usr/bin/env python3
from pathlib import Path
import re
import runpy

root = Path(__file__).resolve().parents[1]
runpy.run_path(str(root / "scripts/batch8-v3-apply.py"), run_name="__main__")


def replace_once(path: str, old: str, new: str) -> None:
    target = root / path
    content = target.read_text()
    count = content.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one occurrence, found {count}: {old[:120]!r}")
    target.write_text(content.replace(old, new, 1))


replace_once(
    "src-tauri/src/config/types.rs",
    '''#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HighRiskOriginPolicy {
    Block,
}

impl Default for HighRiskOriginPolicy {
    fn default() -> Self {
        Self::Block
    }
}
''',
    '''#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HighRiskOriginPolicy {
    #[default]
    Block,
}
''',
)

replace_once(
    "src-tauri/src/commands/contracts/providers.rs",
    "#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]\npub struct RemotePlannerSettings {",
    "#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]\npub struct RemotePlannerSettings {",
)

replace_once(
    "src-tauri/src/app_core/planner_redaction.rs",
    "sanitize_page_model(&page, &mut metadata)",
    'sanitize_page_model(&page, "https://example.test", &mut metadata)',
)

replace_once(
    "src-tauri/src/commands/tests/contracts/planner_contracts.rs",
    "assert_eq!(round_tripped, planner_input);",
    'assert!(\n        round_tripped == planner_input,\n        "planner input round trip changed"\n    );',
)

fixture_paths = [
    "src-tauri/src/commands/tests/fixtures/mock_executor_impl/state.rs",
    "src-tauri/src/commands/tests/fixtures/page_fixtures.rs",
    "src-tauri/src/commands/tests/contracts/planner_contracts.rs",
    "src-tauri/src/commands/tests/direct_commands/playback_commands.rs",
    "src-tauri/src/commands/tests/direct_commands/reading_commands.rs",
    "src-tauri/src/commands/tests/direct_commands/status_commands.rs",
]
initializer = re.compile(
    r"(?P<prefix>remote_planner_settings:\s*RemotePlannerSettings\s*\{)"
    r"(?P<body>.*?)"
    r"\n(?P<indent>\s*)\},",
    flags=re.S,
)
initializer_count = 0
for relative in fixture_paths:
    path = root / relative
    content = path.read_text()

    def add_defaults(match: re.Match[str]) -> str:
        nonlocal_count[0] += 1
        return (
            f"{match.group('prefix')}{match.group('body')}\n"
            f"{match.group('indent')}    ..RemotePlannerSettings::default()\n"
            f"{match.group('indent')}}},"
        )

    nonlocal_count = [0]
    updated = initializer.sub(add_defaults, content)
    initializer_count += nonlocal_count[0]
    path.write_text(updated)

if initializer_count != 13:
    raise SystemExit(
        f"expected 13 RemotePlannerSettings test initializers, found {initializer_count}"
    )

print("Batch 8 all-target compatibility applied")
