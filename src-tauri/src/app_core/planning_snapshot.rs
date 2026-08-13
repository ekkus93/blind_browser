use sha2::{Digest, Sha256};

use crate::commands::{
    current_timestamp_ms, normalized_origin, PlannedStep, PlannerOutput, PlannerSafetySettings,
    ToolError, ToolName,
};
use crate::state::{BrowserHistoryState, PlanningStateSnapshot};

const PLANNING_SNAPSHOT_TTL_MS: u64 = 120_000;
const MAX_PLANNING_SNAPSHOTS: usize = 32;

struct RuntimeStateComponents {
    page_id: Option<String>,
    page_generation: u64,
    origin: Option<String>,
    browser_history: BrowserHistoryState,
    safety: PlannerSafetySettings,
    relevant_config_fingerprint: String,
    runtime_state_token: String,
    pending_confirmation_id: Option<String>,
}

impl super::AppCore {
    pub(crate) fn capture_planning_state_snapshot(&self) -> PlanningStateSnapshot {
        let issued_at_ms = current_timestamp_ms();
        let components = self.runtime_state_components();

        PlanningStateSnapshot {
            page_id: components.page_id,
            page_generation: components.page_generation,
            origin: components.origin,
            browser_history: components.browser_history,
            safety: components.safety,
            relevant_config_fingerprint: components.relevant_config_fingerprint,
            runtime_state_token: components.runtime_state_token,
            pending_confirmation_id: components.pending_confirmation_id,
            issued_at_ms,
            expires_at_ms: issued_at_ms.saturating_add(PLANNING_SNAPSHOT_TTL_MS),
        }
    }

    pub(crate) fn current_runtime_state_token(&self) -> String {
        self.runtime_state_components().runtime_state_token
    }

    /// Runtime token used only while one already-authorized plan is executing
    /// across a deliberately unlocked speech-I/O window. It excludes the
    /// listening flag so `stop_listening` can interrupt capture, but includes
    /// the rest of the planner-relevant configuration plus narration state.
    /// The runner separately verifies that a listening-state change is only
    /// the permitted true -> false transition.
    pub(crate) fn current_lock_scoped_execution_token_without_listening(&self) -> String {
        let page_id = self.state.current_page_id.clone();
        let origin = normalized_origin(
            self.state
                .current_page
                .as_ref()
                .and_then(|page| page.url.as_deref()),
        );
        let safety = PlannerSafetySettings::from(&self.config.safety);
        let relevant_config_fingerprint = self.lock_scoped_execution_fingerprint();
        build_runtime_state_token(
            page_id.as_deref(),
            self.state.page_generation,
            origin.as_deref(),
            &self.state.browser_history,
            &safety,
            &relevant_config_fingerprint,
            self.state.pending_confirmation_id.as_deref(),
        )
    }

    pub(crate) fn current_lock_scoped_listening_state(&self) -> bool {
        self.state.listening.is_listening
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

    fn runtime_state_components(&self) -> RuntimeStateComponents {
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

        RuntimeStateComponents {
            page_id,
            page_generation,
            origin,
            browser_history,
            safety,
            relevant_config_fingerprint,
            runtime_state_token,
            pending_confirmation_id,
        }
    }

    // CR3 P2.6: `remote_planner_privacy` stays in this fingerprint. An
    // earlier version of this fix excluded it here to stop `AllowPersistent`
    // spuriously invalidating the very snapshot it had just authorized (see
    // the `AllowPersistent` arm of `resolve_pending_remote_planner_consent`
    // for the actual fix and why). That approach was reverted:
    // `current_runtime_state_token()` -- which embeds this same fingerprint
    // -- is *also* what `resolve_pending_remote_planner_consent` diffs
    // `pending.draft.runtime_state_token` against to detect "did anything
    // privacy-relevant change while this consent challenge sat unanswered"
    // (see `remote_data_consent_expiry_invalidation_persistence_and_hostile_state_are_fail_closed`'s
    // "mode" case). Excluding `remote_planner_privacy` here would have
    // silently defeated that check too, since it shares the same function.
    fn relevant_config_fingerprint(&self) -> String {
        let value = serde_json::json!({
            "safety": &self.config.safety,
            "planner_provider": &self.config.providers.planner,
            "remote_planner_privacy": &self.config.remote_planner_privacy,
            "tts_provider": &self.config.providers.tts,
            "asr_provider": &self.config.providers.asr,
            "ocr": &self.config.ocr,
            "browser_visibility": self.state.browser_visibility,
            "audio": &self.state.audio,
            "listening": &self.state.listening,
        });
        let encoded =
            serde_json::to_vec(&value).expect("relevant runtime configuration should serialize");
        format!("{:x}", Sha256::digest(encoded))
    }

    fn lock_scoped_execution_fingerprint(&self) -> String {
        let value = serde_json::json!({
            "safety": &self.config.safety,
            "planner_provider": &self.config.providers.planner,
            "remote_planner_privacy": &self.config.remote_planner_privacy,
            "tts_provider": &self.config.providers.tts,
            "asr_provider": &self.config.providers.asr,
            "ocr": &self.config.ocr,
            "browser_visibility": self.state.browser_visibility,
            "audio": &self.state.audio,
            "speaking": self.state.speaking,
            "speaking_region_id": &self.state.speaking_region_id,
            "narration_cursor": &self.state.narration_cursor,
            "last_transcript": &self.state.last_transcript,
            "push_to_talk_enabled": self.state.listening.push_to_talk_enabled,
        });
        let encoded =
            serde_json::to_vec(&value).expect("lock-scoped execution state should serialize");
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
    browser_history: &BrowserHistoryState,
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

// CR3 P1.4: `execute_planner_output` is a directly Tauri-invocable command
// taking a fully caller-controlled `PlannerOutput` -- there is no session/
// origin binding between it and a prior `resolve_command` call other than
// this snapshot requirement. `validate_and_consume_planning_snapshot` only
// runs the (cryptographic, digest-bound -- `planner_output_digest` hashes
// the *entire* serialized output) snapshot check for tools in this list;
// every other tool sails through unauthenticated. `ExtractPageModel`,
// `ReportResult`, and `TranscribeCommand` were missing despite each
// mutating `AppState` in ways that matter: `ExtractPageModel` calls
// `mark_page_model_changed()`, which bumps `page_generation`, clears every
// click authorization, and clears `pending_confirmation_id`/
// `pending_plan_execution` -- so an unauthenticated, out-of-band
// `ExtractPageModel`-only `PlannerOutput` could silently cancel a
// confirmation the user is mid-way through answering. Note this list is
// necessary but not the only layer: `validate_planner_output_with_safety`
// (structural/policy validation) still does not reject an
// `ExtractPageModel`-only plan on its own, since that tool is itself
// classified `NoConfirmation`/read-only by `action_policy.rs` -- the
// snapshot binding here is what actually closes this gap, by ensuring only
// a `PlannerOutput` a real `resolve_command`/replanning call produced (byte-
// identical, since the digest covers the whole struct) can ever reach
// execution, not a forged one.
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
                | ToolName::ExtractPageModel
                | ToolName::ReportResult
                | ToolName::TranscribeCommand
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
        assert!(planning_snapshots_match(
            &snapshot(4, true),
            &snapshot(4, true)
        ));
        assert!(!planning_snapshots_match(
            &snapshot(4, true),
            &snapshot(5, true)
        ));
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
        assert!(!planner_output_requires_snapshot(&[step(
            ToolName::GetRuntimeStatus
        )]));
    }

    #[test]
    fn side_effecting_click_plan_requires_snapshot() {
        assert!(planner_output_requires_snapshot(&[step(
            ToolName::ClickElement
        )]));
    }

    // CR3 P1.4: `execute_planner_output` is directly Tauri-invocable with a
    // fully caller-controlled `PlannerOutput`. Before this fix, a plan
    // composed only of one of these three tools skipped
    // `validate_and_consume_planning_snapshot` entirely (returned `Ok(())`
    // immediately, no digest lookup at all) -- meaning it could execute
    // with no proof it was ever produced by a real `resolve_command`/
    // replanning call. `ExtractPageModel` specifically calls
    // `mark_page_model_changed()`, which clears `pending_confirmation_id`/
    // `pending_plan_execution` -- so this let an out-of-band call silently
    // cancel a confirmation the user was mid-way through answering.
    #[test]
    fn extract_page_model_report_result_and_transcribe_command_now_require_snapshot() {
        for tool_name in [
            ToolName::ExtractPageModel,
            ToolName::ReportResult,
            ToolName::TranscribeCommand,
        ] {
            assert!(
                planner_output_requires_snapshot(&[step(tool_name.clone())]),
                "{tool_name:?} must require a bound planning snapshot: it mutates \
                 AppState (directly or via a caller that assumes it was resolved \
                 through the normal planning path) and must not be reachable from \
                 an unauthenticated, out-of-band execute_planner_output call"
            );
        }
    }
}
