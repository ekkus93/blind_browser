#!/usr/bin/env python3
from pathlib import Path

redaction = Path('src-tauri/src/app_core/planner_redaction.rs').read_text()
prompt = Path('src-tauri/src/app_core/planner_prompt.rs').read_text()
remote = Path('src-tauri/src/app_core/remote_planner.rs').read_text()

required = {
    'redaction': [
        'pub(crate) struct RemotePlannerInput',
        'pub(crate) struct PlannerSafeElementAttributes',
        'remote_planner_high_risk_context_blocked',
        'use crate::audio_io::RuntimeAudioState;',
        'use crate::browser::BrowserVisibilityMode;',
        'use crate::narration::NarrationCursor;',
        'use crate::state::{BrowserHistoryState, ListeningState};',
    ],
    'prompt': [
        'trusted_contract',
        'user_request',
        'untrusted_data',
        'caution-only telemetry',
    ],
    'remote': [
        'planner_output_schema',
        'sanitize_remote_planner_input(planner_input)?',
        'serialize_remote_planner_prompt(&planner_safe_input)?',
        '"content_length": content.len()',
    ],
}
for value in required['redaction']:
    if value not in redaction:
        raise SystemExit(f'missing redaction invariant: {value}')
for value in required['prompt']:
    if value not in prompt:
        raise SystemExit(f'missing prompt invariant: {value}')
for value in required['remote']:
    if value not in remote:
        raise SystemExit(f'missing remote invariant: {value}')

counts = {
    'sanitize': remote.count('sanitize_remote_planner_input(planner_input)?'),
    'serialize': remote.count('serialize_remote_planner_prompt(&planner_safe_input)?'),
    'redacted_response': remote.count('"content_length": content.len()'),
}
expected = {'sanitize': 2, 'serialize': 2, 'redacted_response': 2}
print(f'Batch 7 invariant counts: {counts}')
if counts != expected:
    raise SystemExit(f'unexpected invariant counts: {counts}')

joined = redaction + prompt + remote
for value in [
    'pub(crate) planner_input:',
    '"content": content',
    'AvailableTool, BrowserHistoryState',
    'use crate::state::{BrowserHistoryState, ListeningState, RuntimeAudioState};',
]:
    if value in joined:
        raise SystemExit(f'forbidden Batch 7 pattern remains: {value}')

print('Batch 7 typed privacy invariants passed')
