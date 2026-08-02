#!/usr/bin/env python3
from pathlib import Path
import subprocess

transformer = Path('scripts/batch7-privacy-boundary.py')
text = transformer.read_text()
old = '''def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)
'''
new = '''def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    expected = 2 if label == "redact OpenAI invalid response" else 1
    if count != expected:
        raise SystemExit(f"{label}: expected exactly {expected} match(es), found {count}")
    return text.replace(old, new, 1)
'''
if text.count(old) != 1:
    raise SystemExit('Batch 7 transformer correction baseline did not match exactly once')
text = text.replace(old, new, 1)

derive_old = '#[derive(Debug, Clone, Serialize, PartialEq, Eq)]\\npub(crate) struct RemoteToolObservation {'
derive_new = '#[derive(Debug, Clone, Serialize, PartialEq)]\\npub(crate) struct RemoteToolObservation {'
if text.count(derive_old) != 1:
    raise SystemExit('RemoteToolObservation derive correction did not match exactly once')
transformer.write_text(text.replace(derive_old, derive_new, 1))

subprocess.run(['python3', str(transformer)], check=True)

redaction_path = Path('src-tauri/src/app_core/planner_redaction.rs')
redaction = redaction_path.read_text()
old_imports = '''use crate::commands::{
    AvailableTool, BrowserHistoryState, BrowserVisibilityMode, ListeningState, NarrationCursor,
    PlannerInput, PlannerSafetySettings, PlannerToolHistoryEntry, RuntimeAudioState, SkillSummary,
    ToolError, ToolName,
};'''
new_imports = '''use crate::audio_io::RuntimeAudioState;
use crate::browser::BrowserVisibilityMode;
use crate::commands::{
    AvailableTool, PlannerInput, PlannerSafetySettings, PlannerToolHistoryEntry, SkillSummary,
    ToolError, ToolName,
};
use crate::narration::NarrationCursor;
use crate::state::{BrowserHistoryState, ListeningState};'''
if redaction.count(old_imports) != 1:
    raise SystemExit('generated planner redaction import baseline did not match exactly once')
redaction = redaction.replace(old_imports, new_imports, 1)

old_observation_map = '.map(|value| sanitize_tool_observation_value(value))'
new_observation_map = '.map(sanitize_tool_observation_value)'
if redaction.count(old_observation_map) != 1:
    raise SystemExit(
        f'generated observation closure count={redaction.count(old_observation_map)}'
    )
redaction_path.write_text(redaction.replace(old_observation_map, new_observation_map, 1))

remote_path = Path('src-tauri/src/app_core/remote_planner.rs')
remote = remote_path.read_text()
old_remote_import = 'use crate::commands::{PlannerInput, PlannerOutput, ToolError};'
new_remote_import = (
    'use crate::commands::{planner_output_schema, PlannerInput, PlannerOutput, ToolError};'
)
if remote.count(old_remote_import) != 1:
    raise SystemExit(
        f'generated remote command import count={remote.count(old_remote_import)}'
    )
if remote.count('planner_output_schema()') != 1:
    raise SystemExit(
        f'generated remote planner schema call count={remote.count("planner_output_schema()")}'
    )
remote_path.write_text(remote.replace(old_remote_import, new_remote_import, 1))
print('Applied Batch 7 V3 typed privacy boundary and compiler-scope corrections')
