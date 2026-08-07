//! Persists a user's per-origin remote-planner decision (allow/block) into
//! `config.remote_planner_privacy.origin_rules`, the durable counterpart to
//! [`super::grants`]'s ephemeral session/one-shot grants.

use crate::commands::{current_timestamp_ms, ToolError};
use crate::config::{
    AppConfig, PersistedOriginDecision, RemotePlannerOriginRule, RemotePlannerPrivacySettings,
    REMOTE_DATA_POLICY_VERSION,
};

use super::super::AppCore;
use super::errors::{consent_error, policy_block_error_for};
use super::grants::RemoteDataDisclosureKind;

impl AppCore {
    pub(super) fn origin_rules_settings(
        &self,
        kind: RemoteDataDisclosureKind,
    ) -> &RemotePlannerPrivacySettings {
        match kind {
            RemoteDataDisclosureKind::PlannerPayload => &self.config.remote_planner_privacy,
            RemoteDataDisclosureKind::NarrationText => &self.config.remote_narration_privacy,
        }
    }

    pub(super) fn persist_origin_rule(
        &mut self,
        kind: RemoteDataDisclosureKind,
        page_origin: &str,
        decision: PersistedOriginDecision,
        endpoint_scope: Option<String>,
    ) -> Result<(), ToolError> {
        let mut settings = self.origin_rules_settings(kind).clone();
        if matches!(decision, PersistedOriginDecision::Allow)
            && settings.origin_rules.iter().any(|rule| {
                rule.page_origin == page_origin
                    && matches!(rule.decision, PersistedOriginDecision::Block)
            })
        {
            return Err(policy_block_error_for(
                disclosure_kind_label(kind),
                "remote_data_origin_blocked",
                "origin_block",
            ));
        }
        if matches!(decision, PersistedOriginDecision::Block) {
            settings
                .origin_rules
                .retain(|rule| rule.page_origin != page_origin);
        } else {
            settings.origin_rules.retain(|rule| {
                !(rule.page_origin == page_origin
                    && rule.decision == decision
                    && rule.endpoint_scope == endpoint_scope)
            });
        }
        settings.origin_rules.push(RemotePlannerOriginRule {
            page_origin: page_origin.to_string(),
            decision,
            endpoint_scope,
            policy_version: REMOTE_DATA_POLICY_VERSION,
            created_at_ms: current_timestamp_ms(),
        });
        settings.policy_schema_version = REMOTE_DATA_POLICY_VERSION;
        self.config =
            persist_origin_rules_settings(&self.app_handle, kind, &settings).map_err(|error| {
                consent_error(
                    "remote_data_consent_persist_failed",
                    "remote-data consent decision could not be persisted",
                    Some(serde_json::json!({ "reason": error.to_string() })),
                )
            })?;
        Ok(())
    }
}

fn persist_origin_rules_settings(
    app_handle: &tauri::AppHandle,
    kind: RemoteDataDisclosureKind,
    settings: &RemotePlannerPrivacySettings,
) -> Result<AppConfig, crate::config::ConfigError> {
    match kind {
        RemoteDataDisclosureKind::PlannerPayload => {
            AppConfig::persist_remote_planner_privacy_settings_for_app(app_handle, settings)
        }
        RemoteDataDisclosureKind::NarrationText => {
            AppConfig::persist_remote_narration_privacy_settings_for_app(app_handle, settings)
        }
    }
}

pub(super) fn disclosure_kind_label(kind: RemoteDataDisclosureKind) -> &'static str {
    match kind {
        RemoteDataDisclosureKind::PlannerPayload => "network planning",
        RemoteDataDisclosureKind::NarrationText => "remote narration",
    }
}
