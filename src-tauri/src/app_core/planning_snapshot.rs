use sha2::{Digest, Sha256};

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
        PlanningStateSnapshot {
            page_id: self.state.current_page_id.clone(),
            page_generation: self.state.page_generation,
            origin: normalized_origin(
                self.state
                    .current_page
                    .as_ref()
                    .and_then(|page| page.url.as_deref()),
            ),
            browser_history: self.state.browser_history.clone(),
            safety: PlannerSafetySettings::from(&self.config.safety),
            pending_confirmation_id: self.state.pending_confirmation_id.clone(),
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
                    "expected_page_id": expected.page_id,
                    "observed_page_id": observed.page_id,
                    "expected_page_generation": expected.page_generation,
                    "observed_page_generation": observed.page_generation,
                    "expected_origin": expected.origin,
                    "observed_origin": observed.origin,
                    "expected_history_index": expected.browser_history.current_entry_index,
                    "observed_history_index": observed.browser_history.current_entry_index,
                })),
            ));
        }
        Ok(())
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
    expected.page_id == observed.page_id
        && expected.page_generation == observed.page_generation
        && expected.origin == observed.origin
        && expected.browser_history == observed.browser_history
        && expected.safety == observed.safety
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
    use crate::state::BrowserHistoryState;

    fn snapshot(generation: u64) -> PlanningStateSnapshot {
        PlanningStateSnapshot {
            page_id: Some(String::from("page-1")),
            page_generation: generation,
            origin: Some(String::from("https://example.com")),
            browser_history: BrowserHistoryState::default(),
            safety: PlannerSafetySettings {
                confirmation_confidence_threshold: 0.85,
                allow_click_without_confirmation: true,
                always_confirm_submit: true,
            },
            pending_confirmation_id: None,
            issued_at_ms: 10,
            expires_at_ms: 20,
        }
    }

    #[test]
    fn page_generation_is_part_of_the_planning_snapshot_contract() {
        assert!(planning_snapshots_match(&snapshot(4), &snapshot(4)));
        assert!(!planning_snapshots_match(&snapshot(4), &snapshot(5)));
    }
}
