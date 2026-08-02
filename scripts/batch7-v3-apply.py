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
new_imports = '''use crate::browser::BrowserVisibilityMode;
use crate::commands::{
    AvailableTool, PlannerInput, PlannerSafetySettings, PlannerToolHistoryEntry, SkillSummary,
    ToolError, ToolName,
};
use crate::narration::NarrationCursor;
use crate::state::{BrowserHistoryState, ListeningState, RuntimeAudioState};'''
if redaction.count(old_imports) != 1:
    raise SystemExit('generated planner redaction import baseline did not match exactly once')
redaction_path.write_text(redaction.replace(old_imports, new_imports, 1))

remote_path = Path('src-tauri/src/app_core/remote_planner.rs')
remote = remote_path.read_text()
canonical = 'canonical_planner_output_examples'
tool_schema = 'tool_input_schema'
if remote.count(canonical) != 1:
    raise SystemExit(f'generated remote canonical examples count={remote.count(canonical)}')
anchor = remote.index(canonical)
start = remote.rfind('use crate::commands::{', 0, anchor)
if start < 0:
    raise SystemExit('generated remote commands import was not found before canonical examples')
end = remote.find('};', anchor)
if end < 0:
    raise SystemExit('generated remote commands import terminator was not found')
end += 2
block = remote[start:end]
if block.count(canonical) != 1:
    raise SystemExit('generated remote canonical examples import count was not one')
if block.count(tool_schema) != 1:
    raise SystemExit('generated remote tool schema import count was not one')
if 'planner_output_schema' in block:
    raise SystemExit('generated remote import unexpectedly already contains planner_output_schema')
patched_block = block.replace(
    canonical,
    f'{canonical}, planner_output_schema',
    1,
)
remote_path.write_text(remote[:start] + patched_block + remote[end:])
print('Applied Batch 7 V3 typed privacy boundary and compiler-scope corrections')
