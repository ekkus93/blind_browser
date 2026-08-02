from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text()


def write(path: str, content: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content)


def replace_once(path: str, old: str, new: str) -> None:
    content = read(path)
    count = content.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one match, found {count}: {old[:120]!r}")
    write(path, content.replace(old, new, 1))


# Add the planner-visible opaque state token without making it authoritative.
replace_once(
    "src-tauri/src/commands/contracts/planner.rs",
    "pub struct PlannerInput {\n    pub request_id: String,\n    pub transcript: String,",
    "pub struct PlannerInput {\n    pub request_id: String,\n    #[serde(default)]\n    pub runtime_state_token: String,\n    pub transcript: String,",
)

# Update every test/fixture PlannerInput literal. The production literal is
# replaced below with a token derived from the runtime snapshot.
pattern = re.compile(
    r"(PlannerInput\s*\{\s*\n(?P<indent>\s*)request_id:\s*[^\n]+,\n)(?!\s*runtime_state_token:)"
)
for path in sorted((ROOT / "src-tauri" / "src").rglob("*.rs")):
    relative = path.relative_to(ROOT).as_posix()
    if relative == "src-tauri/src/commands/contracts/planner.rs":
        continue
    content = path.read_text()

    def inject(match: re.Match[str]) -> str:
        indent = match.group("indent")
        return (
            match.group(1)
            + f'{indent}runtime_state_token: String::from("test-runtime-state-token"),\n'
        )

    updated, count = pattern.subn(inject, content)
    if count:
        path.write_text(updated)

replace_once(
    "src-tauri/src/app_core/command_dispatch.rs",
    '            runtime_state_token: String::from("test-runtime-state-token"),',
    "            runtime_state_token: self.capture_planning_state_snapshot().runtime_state_token,",
)

# The server-side snapshot stores both the opaque token and the fingerprint from
# which it was derived. Neither is planner-controlled.
replace_once(
    "src-tauri/src/state.rs",
    "    pub browser_history: BrowserHistoryState,\n    pub safety: PlannerSafetySettings,\n    pub pending_confirmation_id: Option<String>,",
    "    pub browser_history: BrowserHistoryState,\n    pub safety: PlannerSafetySettings,\n    pub relevant_config_fingerprint: String,\n    pub runtime_state_token: String,\n    pub pending_confirmation_id: Option<String>,",
)

planning = r'''use sha2::{Digest, Sha256};

use crate::commands::{
    current_timestamp_ms, normalized_origin, PlannedStep, PlannerOutput, PlannerSafetySettings,
    ToolError, ToolName,
};
use crate::state::PlanningStateSnapshot;

const PLANNING_SNAPSHOT_TTL_MS: u64 = 120_000;
const MAX_PLANNING_SNAPSHOTS: usize = 32;

impl super::AppCore {
    pub(crate) fn capture_planning_state_snapshot(&self) -> PlanningStateSnapshot {
        let issued_at_ms = current_timestamp_ms();
        let page_id = self.state.current_page_id.clone();
        let page_generation = self.state.page_generation;
        let origin = normalized_origin(
            self.state
                .current_page
                .as_ref()
                .and_then(|page| page.url.as_deref()),
        );
        let browser_history = self.state.browser_history.clone();
        let safety = PlannerSafetySettings::from(&self.config.safety);
        let relevant_config_fingerprint = self.relevant_config_fingerprint();
        let pending_confirmation_id = self.state.pending_confirmation_id.clone();
        let runtime_state_token = build_runtime_state_token(
            page_id.as_deref(),
            page_generation,
            origin.as_deref(),
            &browser_history,
            &safety,
            &relevant_config_fingerprint,
            pending_confirmation_id.as_deref(),
        );

        PlanningStateSnapshot {
            page_id,
            page_generation,
            origin,
            browser_history,
            safety,
            relevant_config_fingerprint,
            runtime_state_token,
            pending_confirmation_id,
            issued_at_ms,
            expires_at_ms: issued_at_ms.saturating_add(PLANNING_SNAPSHOT_TTL_MS),
        }
    }

    pub(crate) fn register_planning_snapshot(
        &mut self,
        planner_output: &PlannerOutput,
        snapshot: PlanningStateSnapshot,
    ) -> Result<(), ToolError> {
        let digest = planner_output_digest(planner_output)?;
        let now_ms = current_timestamp_ms();
        self.state
            .planning_snapshots
            .retain(|_, stored| stored.expires_at_ms > now_ms);
        self.state.planning_snapshots.insert(digest, snapshot);
        while self.state.planning_snapshots.len() > MAX_PLANNING_SNAPSHOTS {
            let oldest = self
                .state
                .planning_snapshots
                .iter()
                .min_by_key(|(_, stored)| stored.issued_at_ms)
                .map(|(digest, _)| digest.clone());
            let Some(oldest) = oldest else { break };
            self.state.planning_snapshots.remove(&oldest);
        }
        Ok(())
    }

    pub(crate) fn validate_and_consume_planning_snapshot(
        &mut self,
        planner_output: &PlannerOutput,
    ) -> Result<(), ToolError> {
        if !planner_output_requires_snapshot(&planner_output.steps) {
            return Ok(());
        }

        let digest = planner_output_digest(planner_output)?;
        let Some(expected) = self.state.planning_snapshots.remove(&digest) else {
            return Err(planning_error(
                "missing_planning_snapshot",
                "side-effecting planner output was not bound to a runtime planning snapshot",
                None,
            ));
        };
        let now_ms = current_timestamp_ms();
        if now_ms >= expected.expires_at_ms {
            return Err(planning_error(
                "planning_snapshot_expired",
                "the runtime state snapshot used for planning expired before execution",
                Some(serde_json::json!({
                    "expired_at_ms": expected.expires_at_ms,
                    "observed_at_ms": now_ms,
                })),
            ));
        }

        let observed = self.capture_planning_state_snapshot();
        if !planning_snapshots_match(&expected, &observed) {
            return Err(planning_error(
                "stale_planning_snapshot",
                "runtime state changed after the plan was resolved; the plan must be rebuilt",
                Some(serde_json::json!({
                    "expected_runtime_state_token": expected.runtime_state_token,
                    "observed_runtime_state_token": observed.runtime_state_token,
                    "expected_page_id": expected.page_id,
                    "observed_page_id": observed.page_id,
                    "expected_page_generation": expected.page_generation,
                    "observed_page_generation": observed.page_generation,
                    "expected_origin": expected.origin,
                    "observed_origin": observed.origin,
                    "expected_history_index": expected.browser_history.current_entry_index,
                    "observed_history_index": observed.browser_history.current_entry_index,
                    "expected_config_fingerprint": expected.relevant_config_fingerprint,
                    "observed_config_fingerprint": observed.relevant_config_fingerprint,
                })),
            ));
        }
        Ok(())
    }

    fn relevant_config_fingerprint(&self) -> String {
        let encoded = serde_json::to_vec(&(
            &self.config.safety,
            &self.config.providers.planner,
            &self.config.providers.tts,
            &self.config.providers.asr,
            &self.config.ocr,
            self.state.browser_visibility,
            &self.state.audio,
            &self.state.listening,
        ))
        .expect("relevant runtime configuration should serialize");
        format!("{:x}", Sha256::digest(encoded))
    }
}

fn planner_output_digest(planner_output: &PlannerOutput) -> Result<String, ToolError> {
    let encoded = serde_json::to_vec(planner_output).map_err(|error| {
        planning_error(
            "planning_snapshot_serialization_failed",
            "planner output could not be serialized for snapshot binding",
            Some(serde_json::json!({ "reason": error.to_string() })),
        )
    })?;
    Ok(format!("{:x}", Sha256::digest(encoded)))
}

fn build_runtime_state_token(
    page_id: Option<&str>,
    page_generation: u64,
    origin: Option<&str>,
    browser_history: &crate::state::BrowserHistoryState,
    safety: &PlannerSafetySettings,
    relevant_config_fingerprint: &str,
    pending_confirmation_id: Option<&str>,
) -> String {
    let encoded = serde_json::to_vec(&serde_json::json!({
        "page_id": page_id,
        "page_generation": page_generation,
        "origin": origin,
        "browser_history": browser_history,
        "safety": safety,
        "relevant_config_fingerprint": relevant_config_fingerprint,
        "pending_confirmation_id": pending_confirmation_id,
    }))
    .expect("runtime state token input should serialize");
    format!("state-{:x}", Sha256::digest(encoded))
}

fn planner_output_requires_snapshot(steps: &[PlannedStep]) -> bool {
    steps.iter().any(|step| {
        matches!(
            step.tool_name,
            ToolName::OpenUrl
                | ToolName::GoBack
                | ToolName::GoForward
                | ToolName::ReloadPage
                | ToolName::EvalJs
                | ToolName::ScrollPage
                | ToolName::SetBrowserVisibility
                | ToolName::ClickElement
                | ToolName::FocusElement
                | ToolName::TypeIntoElement
                | ToolName::SubmitActiveForm
                | ToolName::ReadRegion
                | ToolName::ReadNextRegion
                | ToolName::ReadPreviousRegion
                | ToolName::StopSpeaking
                | ToolName::StartListening
                | ToolName::StopListening
                | ToolName::SetTtsVoice
                | ToolName::SetPlaybackVolume
                | ToolName::SetPlaybackSpeed
                | ToolName::MergeOcrIntoPageModel
        )
    })
}

fn planning_snapshots_match(
    expected: &PlanningStateSnapshot,
    observed: &PlanningStateSnapshot,
) -> bool {
    expected.runtime_state_token == observed.runtime_state_token
        && expected.page_id == observed.page_id
        && expected.page_generation == observed.page_generation
        && expected.origin == observed.origin
        && expected.browser_history == observed.browser_history
        && expected.safety == observed.safety
        && expected.relevant_config_fingerprint == observed.relevant_config_fingerprint
        && expected.pending_confirmation_id == observed.pending_confirmation_id
}

fn planning_error(code: &str, message: &str, details: Option<serde_json::Value>) -> ToolError {
    ToolError {
        code: code.to_string(),
        message: message.to_string(),
        retryable: false,
        details,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::StepTransition;
    use crate::state::BrowserHistoryState;

    fn snapshot(generation: u64, allow_click_without_confirmation: bool) -> PlanningStateSnapshot {
        let safety = PlannerSafetySettings {
            confirmation_confidence_threshold: 0.85,
            allow_click_without_confirmation,
            always_confirm_submit: true,
        };
        let browser_history = BrowserHistoryState::default();
        let relevant_config_fingerprint = format!("config-{allow_click_without_confirmation}");
        let runtime_state_token = build_runtime_state_token(
            Some("page-1"),
            generation,
            Some("https://example.com"),
            &browser_history,
            &safety,
            &relevant_config_fingerprint,
            None,
        );
        PlanningStateSnapshot {
            page_id: Some(String::from("page-1")),
            page_generation: generation,
            origin: Some(String::from("https://example.com")),
            browser_history,
            safety,
            relevant_config_fingerprint,
            runtime_state_token,
            pending_confirmation_id: None,
            issued_at_ms: 10,
            expires_at_ms: 20,
        }
    }

    fn step(tool_name: ToolName) -> PlannedStep {
        PlannedStep {
            step_id: String::from("step-1"),
            tool_name,
            arguments: serde_json::json!({
                "request_id": "req-state-token",
                "timeout_ms": 1000
            }),
            purpose: String::from("test runtime snapshot classification"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        }
    }

    #[test]
    fn navigation_or_page_generation_change_invalidates_snapshot() {
        assert!(planning_snapshots_match(&snapshot(4, true), &snapshot(4, true)));
        assert!(!planning_snapshots_match(&snapshot(4, true), &snapshot(5, true)));
    }

    #[test]
    fn safety_change_invalidates_snapshot_resolved_under_weaker_settings() {
        assert!(!planning_snapshots_match(
            &snapshot(4, true),
            &snapshot(4, false)
        ));
    }

    #[test]
    fn read_only_status_plan_does_not_require_snapshot() {
        assert!(!planner_output_requires_snapshot(&[step(ToolName::GetRuntimeStatus)]));
    }

    #[test]
    fn side_effecting_click_plan_requires_snapshot() {
        assert!(planner_output_requires_snapshot(&[step(ToolName::ClickElement)]));
    }
}
'''
write("src-tauri/src/app_core/planning_snapshot.rs", planning)

# Stale/missing/expired snapshot failures become bounded replans. Other policy or
# runtime-preparation failures remain terminal aborts.
replace_once(
    "src-tauri/src/app_core/command_dispatch.rs",
    "    PlannerOutput, PlannerSafetySettings, PlannerToolHistoryEntry, ToolError,",
    "    PlannerOutput, PlannerSafetySettings, PlannerToolHistoryEntry, SerializedToolResult,\n    ToolError, ToolName,",
)
replace_once(
    "src-tauri/src/app_core/command_dispatch.rs",
    "        if let Err(error) = self.validate_and_consume_planning_snapshot(planner_output) {\n            let outcome = planner_execution_abort(error);",
    "        if let Err(error) = self.validate_and_consume_planning_snapshot(planner_output) {\n            let outcome = planner_snapshot_validation_outcome(error);",
)
replace_once(
    "src-tauri/src/app_core/command_dispatch.rs",
    "fn planner_execution_abort(error: ToolError) -> ExecutionOutcome {",
    '''fn planner_snapshot_validation_outcome(error: ToolError) -> ExecutionOutcome {
    if matches!(
        error.code.as_str(),
        "missing_planning_snapshot" | "planning_snapshot_expired" | "stale_planning_snapshot"
    ) {
        return ExecutionOutcome::NeedsReplan {
            trace: crate::commands::ExecutionTrace {
                executed_step_ids: Vec::new(),
                tool_results: vec![SerializedToolResult::failure(
                    ToolName::GetAgentState,
                    String::from("runtime-state-revalidation"),
                    error,
                    vec![String::from(
                        "Runtime state changed after planning; bounded replanning is required.",
                    )],
                )],
            },
        };
    }
    planner_execution_abort(error)
}

fn planner_execution_abort(error: ToolError) -> ExecutionOutcome {''',
)

# Replace the obsolete concurrency comment with the actual fail-closed contract.
replace_once(
    "src-tauri/src/app_core/replanning_orchestrator.rs",
    '''                // ATOMICITY TRADEOFF: resolve and execute are no longer one locked
                // transaction. The plan is resolved against the state snapshot taken
                // in phase 1, which a peer command could in principle change before
                // `execute_plan` re-acquires the lock. This is acceptable for this
                // single-user app whose frontend serializes dependent voice commands
                // with `await`; each resolve→execute cycle keeps its own snapshot
                // self-consistent and the replan bound is unchanged.''',
    '''                // Resolve and execute are intentionally separate lock scopes. The
                // server preserves a state snapshot for the exact planner output, and
                // execution revalidates its opaque token, page generation, history,
                // safety settings, and relevant-config fingerprint. A concurrent
                // command therefore causes a bounded replan rather than allowing a
                // dependent side effect to execute against changed state.''',
)

# Focused operational documentation for the invalidation contract.
write(
    "docs/BBCR-005_RUNTIME_STATE_BINDING_2026-08-01.md",
    '''# BBCR-005 Runtime State Binding

## Authority model

The remote planner receives an opaque `runtime_state_token` in `PlannerInput`, but the token is informational only. Rust preserves the authoritative `PlanningStateSnapshot` server-side and binds it to the exact serialized `PlannerOutput` digest. Planner output cannot create, replace, or weaken that snapshot.

## State represented by the token

The token is a SHA-256 digest over:

- current page ID;
- page/document generation;
- normalized origin;
- browser-history position and boundaries;
- deterministic safety settings;
- a relevant-configuration fingerprint covering provider selections, OCR policy, browser visibility, runtime audio, and listening state;
- pending confirmation identity.

The token deliberately excludes timestamps so two captures of unchanged state produce the same token. Issue and expiry timestamps remain server-side in the snapshot record.

## Tool invalidation matrix

| State change | Invalidated operations | Runtime response |
|---|---|---|
| Navigation, reload, back/forward, page ID change, origin change | Click, focus, type, submit, page-relative narration, OCR merge, navigation-dependent side effects | Reject current plan and enter the bounded replan loop |
| Page-model replacement, OCR merge, or same-page DOM mutation | Element-targeted click/focus/type/submit and confirmations containing those targets | Increment page generation, clear click authorizations and pending confirmation, then replan |
| Safety-setting change | Every side effect resolved under the prior confirmation policy | Reject current plan and replan under current settings |
| Provider selection, OCR policy, browser visibility, audio, or listening-state change | Side effects whose semantics depend on the changed configuration/state | Reject current plan and replan |
| Confirmation created, consumed, cleared, or replaced | Any plan resolved against the prior pending-confirmation state | Reject current plan and replan |
| Unrelated state change during a status-only/read-only plan | `GetAgentState`, `GetRuntimeStatus`, `GetCurrentUrl`, and equivalent non-mutating reads | No snapshot requirement; execute normally |

## Click authorization

A click requires a runtime-owned opaque authorization record bound to page ID, page generation, origin, element ID, DOM locator, element fingerprint, deterministic confidence, ambiguity, destructive classification, issue time, and expiry. Immediately before dispatch, Rust re-extracts the live DOM and re-resolves the target. Changed, hidden, disabled, stale, expired, ambiguous, low-confidence, or destructive targets cannot use the click-without-confirmation exception.

## Confirmation interaction

Confirmation manifests include the generation-qualified page identity. On confirmation response, Rust rebuilds the manifest and revalidates every queued click against the live DOM before consuming the single-use pending state. A page-generation change clears pending confirmation state before a stale response can resume execution.

## Concurrency

`AppCore` mutations remain serialized by its mutex. Releasing the lock during remote planning is safe because execution must consume the server-side snapshot for that exact output. If another frontend command changes relevant state while planning is in flight, execution returns `NeedsReplan`; the voice-command orchestrator permits at most the configured bounded replan count.
''',
)

print("BBCR-015 closure transformation applied")
