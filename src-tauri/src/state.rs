use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::audio_io::RuntimeAudioState;
use crate::browser::BrowserVisibilityMode;
use crate::commands::{
    ExecutionOutcome, LastToolCallSummary, PendingPlanExecutionState, PlannerSafetySettings,
};
use crate::config::{AppConfig, AudioSettings};
use crate::narration::NarrationCursor;
use crate::page_model::PageModel;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
pub struct BrowserHistoryState {
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub current_entry_index: Option<usize>,
    pub entry_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ListeningState {
    pub is_listening: bool,
    pub push_to_talk_enabled: bool,
}

impl Default for ListeningState {
    fn default() -> Self {
        Self {
            is_listening: false,
            push_to_talk_enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClickAuthorizationRecord {
    pub token: String,
    pub page_id: String,
    pub page_generation: u64,
    pub origin: Option<String>,
    pub element_id: String,
    pub dom_locator: String,
    pub element_fingerprint: String,
    pub confidence_bps: Option<u16>,
    pub ambiguous: bool,
    pub potentially_destructive: bool,
    pub issued_at_ms: u64,
    pub expires_at_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PlanningStateSnapshot {
    pub page_id: Option<String>,
    pub page_generation: u64,
    pub origin: Option<String>,
    pub browser_history: BrowserHistoryState,
    pub safety: PlannerSafetySettings,
    pub relevant_config_fingerprint: String,
    pub runtime_state_token: String,
    pub pending_confirmation_id: Option<String>,
    pub issued_at_ms: u64,
    pub expires_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct AppState {
    pub current_page_id: Option<String>,
    pub current_page: Option<PageModel>,
    #[serde(default)]
    pub page_generation: u64,
    pub browser_visibility: BrowserVisibilityMode,
    pub browser_history: BrowserHistoryState,
    pub narration_cursor: NarrationCursor,
    pub speaking: bool,
    pub speaking_region_id: Option<String>,
    pub audio: RuntimeAudioState,
    pub listening: ListeningState,
    pub last_transcript: Option<String>,
    pub last_tool_call: Option<LastToolCallSummary>,
    pub pending_confirmation_id: Option<String>,
    pub pending_plan_execution: Option<PendingPlanExecutionState>,
    #[serde(skip, default)]
    #[schemars(skip)]
    pub(crate) click_authorizations: BTreeMap<String, ClickAuthorizationRecord>,
    #[serde(skip, default)]
    #[schemars(skip)]
    pub(crate) planning_snapshots: BTreeMap<String, PlanningStateSnapshot>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_page_id: None,
            current_page: None,
            page_generation: 0,
            browser_visibility: BrowserVisibilityMode::Visible,
            browser_history: BrowserHistoryState::default(),
            narration_cursor: NarrationCursor::default(),
            speaking: false,
            speaking_region_id: None,
            audio: RuntimeAudioState::default(),
            listening: ListeningState::default(),
            last_transcript: None,
            last_tool_call: None,
            pending_confirmation_id: None,
            pending_plan_execution: None,
            click_authorizations: BTreeMap::new(),
            planning_snapshots: BTreeMap::new(),
        }
    }
}

impl AppState {
    pub fn from_config(config: &AppConfig) -> Self {
        let mut state = Self::default();
        state.apply_audio_settings(&config.audio);
        state
    }

    pub fn apply_audio_settings(&mut self, audio: &AudioSettings) {
        self.audio = RuntimeAudioState::from(audio);
    }

    pub fn mark_page_model_changed(&mut self) {
        self.page_generation = self.page_generation.saturating_add(1).max(1);
        self.click_authorizations.clear();
        self.clear_pending_execution();
    }

    pub fn replace_current_page_model(&mut self, page_model: PageModel) {
        self.current_page = Some(page_model);
        self.mark_page_model_changed();
    }

    pub fn confirmation_page_identity(&self) -> Option<String> {
        self.current_page_id
            .as_ref()
            .map(|page_id| format!("{page_id}@generation:{}", self.page_generation))
    }

    pub fn record_navigation(&mut self, page_id: String, page_url: String) {
        self.current_page_id = Some(page_id);
        self.current_page = Some(PageModel {
            title: None,
            url: Some(page_url),
            regions: Vec::new(),
            interactive_elements: Vec::new(),
        });
        self.mark_page_model_changed();
        self.browser_history = next_history_state_after_navigation(&self.browser_history);
        self.narration_cursor = NarrationCursor::default();
        self.speaking = false;
        self.speaking_region_id = None;
    }

    pub fn start_speaking_region(&mut self, region_id: String) {
        self.speaking = true;
        self.speaking_region_id = Some(region_id);
    }

    pub fn stop_speaking(&mut self) -> Option<String> {
        self.speaking = false;
        self.speaking_region_id.take()
    }

    pub fn set_listening(&mut self, is_listening: bool) {
        self.listening.is_listening = is_listening;
    }

    pub fn record_transcript(&mut self, transcript: Option<String>) {
        self.last_transcript = transcript.and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });
    }

    pub fn apply_execution_outcome(&mut self, outcome: &ExecutionOutcome) {
        self.last_tool_call = execution_outcome_last_tool_call(outcome);

        match outcome {
            ExecutionOutcome::AwaitingConfirmation {
                pending_confirmation_id,
                pending_plan_execution,
                ..
            } => {
                self.pending_confirmation_id = Some(pending_confirmation_id.clone());
                self.pending_plan_execution = Some(pending_plan_execution.as_ref().clone());
            }
            ExecutionOutcome::Complete { .. }
            | ExecutionOutcome::NeedsReplan { .. }
            | ExecutionOutcome::NeedsRemoteDataConsent { .. }
            | ExecutionOutcome::Aborted { .. } => {
                self.clear_pending_execution();
            }
        }
    }

    pub fn clear_pending_execution(&mut self) {
        self.pending_confirmation_id = None;
        self.pending_plan_execution = None;
    }
}

fn execution_outcome_last_tool_call(outcome: &ExecutionOutcome) -> Option<LastToolCallSummary> {
    let trace = match outcome {
        ExecutionOutcome::Complete { trace }
        | ExecutionOutcome::AwaitingConfirmation { trace, .. }
        | ExecutionOutcome::NeedsReplan { trace }
        | ExecutionOutcome::NeedsRemoteDataConsent { trace, .. }
        | ExecutionOutcome::Aborted { trace, .. } => trace,
    };

    trace.tool_results.last().map(|result| LastToolCallSummary {
        request_id: result.request_id.clone(),
        tool_name: result.tool_name.clone(),
        ok: result.ok,
        observation_summary: result.observations.clone(),
    })
}

fn next_history_state_after_navigation(history: &BrowserHistoryState) -> BrowserHistoryState {
    let next_index = history
        .current_entry_index
        .map(|index| index + 1)
        .unwrap_or(0);

    BrowserHistoryState {
        can_go_back: next_index > 0,
        can_go_forward: false,
        current_entry_index: Some(next_index),
        entry_count: next_index + 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::commands::{
        ConfirmationManifest, ExecutionTrace, IntentName, PendingPlanExecutionState,
        SerializedToolResult, ToolError, ToolName,
    };
    use crate::config::{AppConfig, AudioSettings};

    #[test]
    fn stores_pending_execution_from_awaiting_confirmation_outcome() {
        let mut state = AppState::default();
        let outcome = ExecutionOutcome::AwaitingConfirmation {
            trace: ExecutionTrace {
                executed_step_ids: vec![String::from("step-1")],
                tool_results: vec![SerializedToolResult::success(
                    ToolName::ConfirmAction,
                    String::from("req-1"),
                    serde_json::json!({
                        "confirmation_id": "confirm-1",
                    }),
                    vec![String::from("confirmation requested")],
                )],
            },
            pending_confirmation_id: String::from("confirm-1"),
            pending_plan_execution: Box::new(PendingPlanExecutionState {
                request_id: String::from("req-1"),
                intent_name: IntentName::ClickElement,
                selected_skills: vec![String::from("confirm_action")],
                confirmation_id: String::from("confirm-1"),
                manifest_digest: String::from("digest-1"),
                manifest: ConfirmationManifest {
                    request_id: String::from("req-1"),
                    page_id: None,
                    origin: None,
                    issued_at_ms: 1,
                    expires_at_ms: 2,
                    actions: Vec::new(),
                },
                prompt_text: String::from("Proceed?"),
                runtime_state_token: String::new(),
                next_step_id: Some(String::from("step-2")),
                queued_step_ids: vec![String::from("step-2")],
                queued_steps: Vec::new(),
            }),
        };

        state.apply_execution_outcome(&outcome);

        assert_eq!(state.pending_confirmation_id.as_deref(), Some("confirm-1"));
        assert_eq!(
            state.last_tool_call.as_ref().map(|entry| (
                &entry.tool_name,
                entry.ok,
                entry.request_id.as_str()
            )),
            Some((&ToolName::ConfirmAction, true, "req-1"))
        );
        assert_eq!(
            state
                .pending_plan_execution
                .as_ref()
                .map(|pending| pending.request_id.as_str()),
            Some("req-1")
        );
    }

    #[test]
    fn clears_pending_execution_for_non_waiting_outcomes() {
        let mut state = AppState {
            pending_confirmation_id: Some(String::from("confirm-1")),
            pending_plan_execution: Some(PendingPlanExecutionState {
                request_id: String::from("req-1"),
                intent_name: IntentName::ClickElement,
                selected_skills: vec![String::from("confirm_action")],
                confirmation_id: String::from("confirm-1"),
                manifest_digest: String::from("digest-1"),
                manifest: ConfirmationManifest {
                    request_id: String::from("req-1"),
                    page_id: None,
                    origin: None,
                    issued_at_ms: 1,
                    expires_at_ms: 2,
                    actions: Vec::new(),
                },
                prompt_text: String::from("Proceed?"),
                runtime_state_token: String::new(),
                next_step_id: Some(String::from("step-2")),
                queued_step_ids: vec![String::from("step-2")],
                queued_steps: Vec::new(),
            }),
            last_tool_call: Some(LastToolCallSummary {
                request_id: String::from("req-old"),
                tool_name: ToolName::GetAgentState,
                ok: true,
                observation_summary: vec![String::from("agent state read")],
            }),
            ..AppState::default()
        };

        let outcome = ExecutionOutcome::Aborted {
            trace: ExecutionTrace {
                executed_step_ids: vec![String::from("step-1")],
                tool_results: vec![SerializedToolResult::failure(
                    ToolName::SetPlaybackVolume,
                    String::from("req-2"),
                    ToolError {
                        code: String::from("aborted"),
                        message: String::from("execution stopped"),
                        retryable: false,
                        details: None,
                    },
                    vec![String::from("volume update failed")],
                )],
            },
            error: ToolError {
                code: String::from("aborted"),
                message: String::from("execution stopped"),
                retryable: false,
                details: None,
            },
        };

        state.apply_execution_outcome(&outcome);

        assert!(state.pending_confirmation_id.is_none());
        assert!(state.pending_plan_execution.is_none());
        assert_eq!(
            state.last_tool_call.as_ref().map(|entry| (
                &entry.tool_name,
                entry.ok,
                entry.request_id.as_str()
            )),
            Some((&ToolName::SetPlaybackVolume, false, "req-2"))
        );
    }

    #[test]
    fn record_navigation_sets_page_identity_and_advances_history() {
        let mut state = AppState::default();
        state.start_speaking_region(String::from("region-1"));
        state.record_navigation(
            String::from("page-1"),
            String::from("https://example.com/first"),
        );
        state.record_navigation(
            String::from("page-2"),
            String::from("https://example.com/second"),
        );

        assert_eq!(state.current_page_id.as_deref(), Some("page-2"));
        assert_eq!(
            state
                .current_page
                .as_ref()
                .and_then(|page| page.url.as_deref()),
            Some("https://example.com/second")
        );
        assert_eq!(state.browser_history.current_entry_index, Some(1));
        assert_eq!(state.browser_history.entry_count, 2);
        assert!(state.browser_history.can_go_back);
        assert!(!state.browser_history.can_go_forward);
        assert!(!state.speaking);
        assert!(state.speaking_region_id.is_none());
    }

    #[test]
    fn record_navigation_truncates_forward_history_from_earlier_entry() {
        let mut state = AppState {
            browser_history: BrowserHistoryState {
                can_go_back: false,
                can_go_forward: true,
                current_entry_index: Some(0),
                entry_count: 4,
            },
            ..AppState::default()
        };

        state.record_navigation(
            String::from("page-2"),
            String::from("https://example.com/second"),
        );

        assert_eq!(
            state.browser_history,
            BrowserHistoryState {
                can_go_back: true,
                can_go_forward: false,
                current_entry_index: Some(1),
                entry_count: 2,
            }
        );
    }

    #[test]
    fn page_generation_advances_for_navigation_and_model_replacement() {
        let mut state = AppState::default();
        state.record_navigation(String::from("page-1"), String::from("https://example.com"));
        let first_identity = state.confirmation_page_identity();
        assert_eq!(state.page_generation, 1);

        state.replace_current_page_model(PageModel {
            title: Some(String::from("Replacement")),
            url: Some(String::from("https://example.com")),
            regions: Vec::new(),
            interactive_elements: Vec::new(),
        });
        assert_eq!(state.page_generation, 2);
        assert_ne!(state.confirmation_page_identity(), first_identity);
    }

    #[test]
    fn stop_speaking_clears_runtime_speaking_state() {
        let mut state = AppState::default();
        state.start_speaking_region(String::from("region-2"));

        let interrupted_region_id = state.stop_speaking();

        assert_eq!(interrupted_region_id.as_deref(), Some("region-2"));
        assert!(!state.speaking);
        assert!(state.speaking_region_id.is_none());
    }

    #[test]
    fn set_listening_updates_runtime_listening_state() {
        let mut state = AppState::default();

        assert!(!state.listening.is_listening);
        assert!(state.listening.push_to_talk_enabled);

        state.set_listening(true);
        assert!(state.listening.is_listening);
        assert!(state.listening.push_to_talk_enabled);

        state.set_listening(false);
        assert!(!state.listening.is_listening);
        assert!(state.listening.push_to_talk_enabled);
    }

    #[test]
    fn record_transcript_trims_text_and_clears_empty_values() {
        let mut state = AppState::default();

        state.record_transcript(Some(String::from("  read the next section  ")));
        assert_eq!(
            state.last_transcript.as_deref(),
            Some("read the next section")
        );

        state.record_transcript(Some(String::from("   ")));
        assert!(state.last_transcript.is_none());

        state.record_transcript(Some(String::from("\n\nplay the title\t")));
        assert_eq!(state.last_transcript.as_deref(), Some("play the title"));

        state.record_transcript(None);
        assert!(state.last_transcript.is_none());
    }

    #[test]
    fn from_config_hydrates_persisted_audio_settings() {
        let config = AppConfig::load_from_str(
            &AppConfig::default_template()
                .replace("playback_volume = 1.0", "playback_volume = 0.25")
                .replace("playback_speed = 1.0", "playback_speed = 1.6")
                .replace(
                    "default_tts_voice = \"Bruno\"",
                    "default_tts_voice = \"Rosie\"",
                ),
        )
        .expect("config should load");

        let state = AppState::from_config(&config);

        assert!((state.audio.playback_volume - 0.25).abs() < f32::EPSILON);
        assert!((state.audio.playback_speed - 1.6).abs() < f32::EPSILON);
        assert_eq!(state.audio.tts_voice.as_deref(), Some("Rosie"));
        assert!(!state.audio.muted);
    }

    #[test]
    fn browser_history_state_serializes_default_boundary_values() {
        let serialized = serde_json::to_value(BrowserHistoryState::default())
            .expect("browser history state should serialize");

        assert_eq!(
            serialized,
            serde_json::json!({
                "can_go_back": false,
                "can_go_forward": false,
                "current_entry_index": null,
                "entry_count": 0
            })
        );

        let round_tripped: BrowserHistoryState =
            serde_json::from_value(serialized).expect("browser history state should deserialize");
        assert_eq!(round_tripped, BrowserHistoryState::default());
    }

    #[test]
    fn browser_history_state_round_trips_populated_navigation_position() {
        let history = BrowserHistoryState {
            can_go_back: true,
            can_go_forward: true,
            current_entry_index: Some(2),
            entry_count: 4,
        };

        let serialized =
            serde_json::to_value(&history).expect("browser history state should serialize");
        assert_eq!(
            serialized,
            serde_json::json!({
                "can_go_back": true,
                "can_go_forward": true,
                "current_entry_index": 2,
                "entry_count": 4
            })
        );

        let round_tripped: BrowserHistoryState =
            serde_json::from_value(serialized).expect("browser history state should deserialize");
        assert_eq!(round_tripped, history);
    }

    #[test]
    fn deserializing_legacy_page_regions_defaults_missing_bbox_to_none() {
        let state: AppState = serde_json::from_value(serde_json::json!({
            "current_page_id": "page-1",
            "current_page": {
                "title": "Example",
                "url": "https://example.com",
                "regions": [
                    {
                        "region_id": "region-1",
                        "label": "Main",
                        "text": "Example text",
                        "source": "Dom"
                    }
                ],
                "interactive_elements": []
            },
            "browser_visibility": "Visible",
            "browser_history": {
                "can_go_back": false,
                "can_go_forward": false,
                "current_entry_index": null,
                "entry_count": 0
            },
            "narration_cursor": {
                "current_region_id": null,
                "current_index": null,
                "total_regions": 0
            },
            "speaking": false,
            "speaking_region_id": null,
            "audio": {
                "playback_volume": 1.0,
                "playback_speed": 1.0,
                "muted": false,
                "tts_voice": null
            },
            "listening": {
                "is_listening": false,
                "push_to_talk_enabled": true
            },
            "last_transcript": null,
            "last_tool_call": null,
            "pending_confirmation_id": null,
            "pending_plan_execution": null
        }))
        .expect("legacy app state should deserialize");

        assert_eq!(
            state
                .current_page
                .as_ref()
                .and_then(|page| page.regions.first())
                .map(|region| region.role.clone()),
            Some(crate::page_model::RegionRole::Other)
        );
        assert_eq!(
            state
                .current_page
                .as_ref()
                .and_then(|page| page.regions.first())
                .and_then(|region| region.bbox.clone()),
            None
        );
    }

    #[test]
    fn apply_audio_settings_refreshes_next_utterance_speech_settings_mid_session() {
        let config = AppConfig::default();
        let mut state = AppState::from_config(&config);

        // Simulate mid-session settings change.
        let updated = AudioSettings {
            playback_volume: 0.35,
            default_tts_voice: String::from("Luna"),
            playback_speed: 1.75,
        };
        state.apply_audio_settings(&updated);

        // The new speech settings must be visible before the next synthesis call.
        assert!((state.audio.playback_volume - 0.35).abs() < f32::EPSILON);
        assert_eq!(state.audio.tts_voice.as_deref(), Some("Luna"));
        assert!((state.audio.playback_speed - 1.75).abs() < f32::EPSILON);
    }

    #[test]
    fn apply_audio_settings_leave_active_narration_running_until_the_next_utterance() {
        let config = AppConfig::default();
        let mut state = AppState::from_config(&config);

        // Put the session into an active-narration state.
        state.start_speaking_region(String::from("region-42"));

        let updated = AudioSettings {
            playback_volume: 0.2,
            playback_speed: 1.5,
            default_tts_voice: String::from("Bella"),
        };
        state.apply_audio_settings(&updated);

        // The current utterance keeps running, but the next one will use the new settings.
        assert!(state.speaking);
        assert_eq!(state.speaking_region_id.as_deref(), Some("region-42"));
        assert!((state.audio.playback_volume - 0.2).abs() < f32::EPSILON);
        assert!((state.audio.playback_speed - 1.5).abs() < f32::EPSILON);
        assert_eq!(state.audio.tts_voice.as_deref(), Some("Bella"));
    }

    #[test]
    fn apply_audio_settings_reflects_muted_when_volume_is_zero() {
        let config = AppConfig::default();
        let mut state = AppState::from_config(&config);

        let silenced = AudioSettings {
            playback_volume: 0.0,
            ..config.audio.clone()
        };
        state.apply_audio_settings(&silenced);

        assert!(state.audio.muted);
        assert!((state.audio.playback_volume).abs() < f32::EPSILON);
    }
}
