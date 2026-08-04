use super::page_model_builder::build_visible_text_excerpt;
use super::settings_adapters::{
    build_asr_provider_settings, build_confirmation_settings, build_local_asr_model_settings,
    build_local_tts_model_settings, build_ocr_threshold_settings, build_provider_failover_settings,
    build_remote_asr_settings, build_remote_planner_settings, build_remote_tts_settings,
    build_tts_model_settings, build_tts_provider_settings, build_tts_voice_settings,
};
use super::AppCore;
use crate::browser::BrowserPageMetrics;
use crate::commands::{
    AgentStateData, AsrProviderSettings, ConfirmationSettings, GetRuntimeStatusData,
    LocalAsrModelSettings, LocalTtsModelSettings, OcrThresholdSettings, PageSnapshotData,
    ProviderFailoverSettings, ProviderSelectionStatus, RemoteAsrSettings, RemotePlannerSettings,
    RemoteTtsSettings, ToolError, TtsModelSettings, TtsProviderSettings, TtsVoiceSettings,
};

impl AppCore {
    pub(super) fn current_tts_model_settings(&self) -> TtsModelSettings {
        build_tts_model_settings(&self.config)
    }

    pub(super) fn current_local_tts_model_settings(&self) -> LocalTtsModelSettings {
        build_local_tts_model_settings(&self.config)
    }

    pub(super) fn current_tts_voice_settings(&self) -> TtsVoiceSettings {
        build_tts_voice_settings(&self.config, &self.state.audio)
    }

    pub(super) fn current_tts_provider_settings(&self) -> TtsProviderSettings {
        build_tts_provider_settings(&self.config)
    }

    pub(super) fn current_asr_provider_settings(&self) -> AsrProviderSettings {
        build_asr_provider_settings(&self.config)
    }

    pub(super) fn current_local_asr_model_settings(&self) -> LocalAsrModelSettings {
        build_local_asr_model_settings(&self.config)
    }

    pub fn current_remote_planner_settings(&self) -> RemotePlannerSettings {
        build_remote_planner_settings(&self.config)
    }

    pub fn current_remote_tts_settings(&self) -> RemoteTtsSettings {
        build_remote_tts_settings(&self.config)
    }

    pub fn current_remote_asr_settings(&self) -> RemoteAsrSettings {
        build_remote_asr_settings(&self.config)
    }

    pub(super) fn current_provider_failover_settings(&self) -> ProviderFailoverSettings {
        build_provider_failover_settings(&self.config)
    }

    pub(super) fn current_confirmation_settings(&self) -> ConfirmationSettings {
        build_confirmation_settings(&self.config)
    }

    pub(super) fn current_ocr_threshold_settings(&self) -> OcrThresholdSettings {
        build_ocr_threshold_settings(&self.config)
    }

    pub(super) fn current_agent_state_snapshot(
        &self,
        include_last_transcript: bool,
    ) -> AgentStateData {
        AgentStateData {
            page_id: self.state.current_page_id.clone(),
            url: self
                .state
                .current_page
                .as_ref()
                .and_then(|page| page.url.clone()),
            title: self
                .state
                .current_page
                .as_ref()
                .and_then(|page| page.title.clone()),
            browser_visibility: self.state.browser_visibility,
            browser_history: self.state.browser_history.clone(),
            narration_cursor: Some(self.state.narration_cursor.clone()),
            speaking: self.state.speaking,
            listening_state: self.state.listening.clone(),
            audio: self.state.audio.clone(),
            last_transcript: if include_last_transcript {
                self.state.last_transcript.clone()
            } else {
                None
            },
            last_tool_call: self.state.last_tool_call.clone(),
            pending_confirmation_id: self.state.pending_confirmation_id.clone(),
            pending_plan_execution: self.state.pending_plan_execution.clone(),
            tts_model_settings: self.current_tts_model_settings(),
            local_tts_model_settings: self.current_local_tts_model_settings(),
            tts_voice_settings: self.current_tts_voice_settings(),
            tts_provider_settings: self.current_tts_provider_settings(),
            asr_provider_settings: self.current_asr_provider_settings(),
            local_asr_model_settings: self.current_local_asr_model_settings(),
            remote_planner_settings: self.current_remote_planner_settings(),
            remote_planner_privacy_status: self.current_remote_planner_privacy_status(),
            remote_tts_settings: self.current_remote_tts_settings(),
            remote_asr_settings: self.current_remote_asr_settings(),
            provider_failover_settings: self.current_provider_failover_settings(),
            confirmation_settings: self.current_confirmation_settings(),
            ocr_threshold_settings: self.current_ocr_threshold_settings(),
        }
    }

    pub(super) fn current_runtime_status_snapshot(
        &self,
        include_provider_modes: bool,
    ) -> GetRuntimeStatusData {
        GetRuntimeStatusData {
            page_id: self.state.current_page_id.clone(),
            url: self
                .state
                .current_page
                .as_ref()
                .and_then(|page| page.url.clone()),
            title: self
                .state
                .current_page
                .as_ref()
                .and_then(|page| page.title.clone()),
            browser_visibility: self.state.browser_visibility,
            browser_history: self.state.browser_history.clone(),
            listening_state: self.state.listening.clone(),
            speaking: self.state.speaking,
            audio: self.state.audio.clone(),
            pending_confirmation_id: self.state.pending_confirmation_id.clone(),
            pending_plan_execution: self.state.pending_plan_execution.clone(),
            provider_modes: if include_provider_modes {
                Some(ProviderSelectionStatus {
                    planner_mode: self.config.providers.planner.mode.clone(),
                    tts_mode: self.config.providers.tts.mode.clone(),
                    asr_mode: self.config.providers.asr.mode.clone(),
                })
            } else {
                None
            },
            remote_planner_privacy_status: self.current_remote_planner_privacy_status(),
            skill_discovery_diagnostics: self.last_skill_discovery_diagnostics.clone(),
        }
    }

    /// Build a snapshot of the current page, or `Ok(None)` when there is genuinely
    /// no current page. A failure to read browser page metrics is surfaced as a
    /// structured `ToolError` (`browser_metrics_failed`) rather than collapsed to
    /// `None`, so callers can tell "no current page" apart from "metrics read
    /// failed".
    pub(super) fn current_page_snapshot(
        &mut self,
        text_excerpt_max_chars: Option<usize>,
        include_interactive_elements: bool,
    ) -> Result<Option<PageSnapshotData>, ToolError> {
        let Some(page_id) = self.state.current_page_id.clone() else {
            return Ok(None);
        };
        let Some(current_page) = self.state.current_page.as_ref() else {
            return Ok(None);
        };
        let Some(url) = current_page.url.clone() else {
            return Ok(None);
        };

        let title = current_page.title.clone();
        let visible_text_excerpt = build_visible_text_excerpt(current_page, text_excerpt_max_chars);
        let interactive_elements = if include_interactive_elements {
            current_page.interactive_elements.clone()
        } else {
            Vec::new()
        };

        let BrowserPageMetrics {
            scroll_y,
            viewport_width,
            viewport_height,
            document_height,
        } = self.browser.get_page_metrics().map_err(|error| ToolError {
            code: String::from("browser_metrics_failed"),
            message: String::from("failed to read browser page metrics for current page snapshot"),
            retryable: true,
            details: Some(serde_json::json!({ "reason": error.to_string() })),
        })?;

        Ok(Some(PageSnapshotData {
            page_id,
            url,
            title,
            visible_text_excerpt,
            interactive_elements,
            scroll_y,
            viewport_width,
            viewport_height,
            document_height,
        }))
    }
}
