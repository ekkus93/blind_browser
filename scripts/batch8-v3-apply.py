#!/usr/bin/env python3
from pathlib import Path
import re
import runpy

root = Path(__file__).resolve().parents[1]
runpy.run_path(str(root / "scripts/batch8-v2-apply.py"), run_name="__main__")

remote_path = root / "src-tauri/src/app_core/remote_planner.rs"
remote = remote_path.read_text()
remote, command_import_count = re.subn(
    r"use crate::commands::\{.*?\};",
    "use crate::commands::{planner_output_schema, PlannerInput, PlannerOutput, ToolError};",
    remote,
    count=1,
    flags=re.S,
)
if command_import_count != 1:
    raise SystemExit(
        f"remote_planner command import replacement count={command_import_count}"
    )
remote_path.write_text(remote)

redaction_path = root / "src-tauri/src/app_core/planner_redaction.rs"
redaction = redaction_path.read_text()
canonical_imports = '''use std::collections::BTreeSet;

use serde::Serialize;

use crate::audio_io::RuntimeAudioState;
use crate::browser::BrowserVisibilityMode;
use crate::commands::{
    AvailableTool, PlannerInput, PlannerSafetySettings, PlannerToolHistoryEntry, SkillSummary,
    ToolError, ToolName,
};
use crate::config::{HighRiskOriginPolicy, RemotePlannerPrivacySettings};
use crate::narration::NarrationCursor;
use crate::page_model::{
    ElementRole, InteractiveElement, PageModel, PageRegion, Rect, RegionRole, RegionSource,
};
use crate::provider_endpoint::ProviderEndpointScope;
use crate::state::{BrowserHistoryState, ListeningState};

'''
redaction, import_prefix_count = re.subn(
    r"\A.*?(?=const MAX_REMOTE_REGIONS:)",
    canonical_imports,
    redaction,
    count=1,
    flags=re.S,
)
if import_prefix_count != 1:
    raise SystemExit(
        f"planner_redaction import prefix replacement count={import_prefix_count}"
    )
redaction_path.write_text(redaction)

safety_path = root / "src-tauri/src/command_handlers/safety_handlers.rs"
safety = safety_path.read_text()
old_ownership = '''        blocked_origins: current.blocked_origins,
        high_risk_origin_policy: String::from("block"),
        changed: current != previous,'''
new_ownership = '''        blocked_origins: current.blocked_origins.clone(),
        high_risk_origin_policy: String::from("block"),
        changed: current != previous,'''
if safety.count(old_ownership) != 1:
    raise SystemExit(
        "safety_handlers generated ownership shape changed or is ambiguous"
    )
safety_path.write_text(safety.replace(old_ownership, new_ownership, 1))

print("Batch 8 generated Rust compatibility applied")
