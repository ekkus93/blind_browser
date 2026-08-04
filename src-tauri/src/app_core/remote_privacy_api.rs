use super::planner_redaction::high_risk_context_reason;
use super::remote_data_consent::{
    evaluate_remote_planner_policy, RemotePlannerDataAuthorization, RemotePlannerPolicyResult,
};
use super::AppCore;
use crate::commands::{
    current_timestamp_ms, PlannerInput, RemotePlannerConsentChallengeSummary,
    RemotePlannerEffectiveDecision, RemotePlannerOriginRuleStatus, RemotePlannerPrivacyOperation,
    RemotePlannerPrivacyStatus, ToolError,
};
use crate::config::{
    AppConfig, PersistedOriginDecision, RemotePlannerNetworkMode, RemotePlannerOriginRule,
    RemotePlannerPrivacySettings, REMOTE_DATA_POLICY_VERSION,
};
use crate::provider_endpoint::ProviderEndpointScope;

const MAX_PERSISTENT_ORIGIN_RULES: usize = 256;

impl AppCore {
    pub fn current_remote_planner_privacy_status(&self) -> RemotePlannerPrivacyStatus {
        let now_ms = current_timestamp_ms();
        let current_page_origin = self.current_remote_planner_page_origin();
        let high_risk_reason = self.current_remote_planner_high_risk_reason();
        let privacy = &self.config.remote_planner_privacy;

        let (endpoint_scope, endpoint_display, endpoint_is_loopback, planner_reason_code) =
            self.current_remote_planner_endpoint_status();
        let normalized_endpoint = endpoint_scope
            .as_ref()
            .map(|scope| scope.normalized_base_url().to_string());

        let (effective_decision, reason_code) = match endpoint_scope.as_ref() {
            None => (
                RemotePlannerEffectiveDecision::PlannerUnavailable,
                planner_reason_code.map(String::from),
            ),
            Some(scope) => map_policy_status(evaluate_remote_planner_policy(
                privacy,
                scope,
                current_page_origin.as_deref(),
                high_risk_reason,
                &self.remote_planner_ephemeral_grants,
                now_ms,
            )),
        };

        let persistent_rules = privacy
            .origin_rules
            .iter()
            .map(|rule| build_rule_status(rule, normalized_endpoint.as_deref()))
            .collect::<Vec<_>>();
        let stale_allow_rule_count = persistent_rules
            .iter()
            .filter(|rule| rule.stale && matches!(rule.decision, PersistedOriginDecision::Allow))
            .count();
        let persistent_rule = current_page_origin.as_deref().and_then(|page_origin| {
            persistent_rule_for_origin(
                &privacy.origin_rules,
                page_origin,
                normalized_endpoint.as_deref(),
            )
        });
        let pending_challenge = self
            .pending_remote_planner_consent
            .as_ref()
            .filter(|pending| pending.challenge.expires_at_ms > now_ms)
            .map(|pending| RemotePlannerConsentChallengeSummary {
                challenge_id: pending.challenge.challenge_id.clone(),
                request_id: pending.challenge.request_id.clone(),
                page_origin: pending.challenge.page_origin.clone(),
                endpoint_display: pending.challenge.endpoint_display.clone(),
                profile_name: pending.challenge.profile_name.clone(),
                model_label: pending.challenge.model_label.clone(),
                policy_version: pending.challenge.policy_version,
                disclosure_classes: pending.challenge.disclosure_classes.clone(),
                disclosure_counts: pending.challenge.disclosure_counts.clone(),
                expires_at_ms: pending.challenge.expires_at_ms,
                allow_once: pending.challenge.allow_once,
                allow_session: pending.challenge.allow_session,
                allow_persistent: pending.challenge.allow_persistent,
                block_persistent: pending.challenge.block_persistent,
            });

        RemotePlannerPrivacyStatus {
            network_mode: privacy.network_mode.clone(),
            endpoint_scope: normalized_endpoint,
            endpoint_display,
            endpoint_is_loopback,
            current_page_origin,
            effective_decision: effective_decision.clone(),
            reason_code,
            persistent_rule,
            session_grant_active: matches!(
                effective_decision,
                RemotePlannerEffectiveDecision::AllowedSession
            ),
            pending_challenge,
            policy_version: REMOTE_DATA_POLICY_VERSION,
            persistent_rule_count: privacy.origin_rules.len(),
            stale_allow_rule_count,
            persistent_rules,
            migration_notice_pending: privacy.migration_notice_pending,
        }
    }

    pub fn apply_remote_planner_privacy_operation(
        &mut self,
        operation: RemotePlannerPrivacyOperation,
    ) -> Result<bool, ToolError> {
        match operation {
            RemotePlannerPrivacyOperation::GetStatus => Ok(false),
            RemotePlannerPrivacyOperation::SetNetworkMode { network_mode } => {
                let mut settings = self.config.remote_planner_privacy.clone();
                settings.network_mode = network_mode;
                self.persist_remote_planner_privacy_operation(settings, true)
            }
            RemotePlannerPrivacyOperation::UpsertOriginRule {
                page_origin,
                decision,
            } => {
                let page_origin = normalize_http_origin(&page_origin)?;
                self.upsert_remote_planner_origin_rule(page_origin, decision, false)
            }
            RemotePlannerPrivacyOperation::UpsertCurrentOriginRule { decision } => {
                let page_origin = self.current_remote_planner_page_origin().ok_or_else(|| {
                    privacy_api_error(
                        "remote_data_opaque_origin_blocked",
                        "the current page does not have a supported HTTP(S) origin",
                        None,
                    )
                })?;
                self.upsert_remote_planner_origin_rule(page_origin, decision, true)
            }
            RemotePlannerPrivacyOperation::RevokeOriginRule {
                page_origin,
                decision,
                endpoint_scope,
            } => {
                let page_origin = normalize_http_origin(&page_origin)?;
                self.revoke_remote_planner_origin_rule(
                    &page_origin,
                    decision,
                    endpoint_scope.as_deref(),
                )
            }
            RemotePlannerPrivacyOperation::ClearSessionGrants => {
                let changed = !self.remote_planner_ephemeral_grants.is_empty();
                self.remote_planner_ephemeral_grants.clear();
                Ok(changed)
            }
            RemotePlannerPrivacyOperation::ClearPersistentAllows => {
                let mut settings = self.config.remote_planner_privacy.clone();
                settings
                    .origin_rules
                    .retain(|rule| matches!(rule.decision, PersistedOriginDecision::Block));
                self.persist_remote_planner_privacy_operation(settings, true)
            }
            RemotePlannerPrivacyOperation::ClearAllPersistentRules { confirmed } => {
                if !confirmed {
                    return Err(privacy_api_error(
                        "remote_data_clear_all_confirmation_required",
                        "clearing every persistent remote-data rule requires explicit confirmation",
                        None,
                    ));
                }
                let mut settings = self.config.remote_planner_privacy.clone();
                settings.origin_rules.clear();
                self.persist_remote_planner_privacy_operation(settings, true)
            }
            RemotePlannerPrivacyOperation::AcknowledgeMigrationNotice => {
                let mut settings = self.config.remote_planner_privacy.clone();
                settings.migration_notice_pending = false;
                self.persist_remote_planner_privacy_operation(settings, false)
            }
        }
    }

    fn current_remote_planner_page_origin(&self) -> Option<String> {
        self.state
            .current_page
            .as_ref()
            .and_then(|page| page.url.as_deref())
            .and_then(page_origin_from_url)
    }

    fn current_remote_planner_high_risk_reason(&self) -> Option<&'static str> {
        let input = PlannerInput {
            request_id: String::from("remote-planner-privacy-status"),
            runtime_state_token: self.current_runtime_state_token(),
            transcript: String::new(),
            agent_state: self.current_agent_state_snapshot(false),
            safety: (&self.config.safety).into(),
            available_tools: Vec::new(),
            active_skill_names: Vec::new(),
            relevant_skill_summaries: Vec::new(),
            page_snapshot: None,
            page_model: self.state.current_page.clone(),
            recent_tool_results: Vec::new(),
        };
        high_risk_context_reason(&input)
    }

    fn current_remote_planner_endpoint_status(
        &self,
    ) -> (
        Option<ProviderEndpointScope>,
        Option<String>,
        Option<bool>,
        Option<&'static str>,
    ) {
        let Some(profile_name) = self.config.providers.planner.remote_profile.as_deref() else {
            return (None, None, None, Some("planner_not_configured"));
        };
        let Some(profile) = self.config.remote_planner_profiles.get(profile_name) else {
            return (None, None, None, Some("planner_profile_missing"));
        };
        match ProviderEndpointScope::parse(&profile.base_url) {
            Ok(scope) => {
                let display = scope.normalized_base_url().to_string();
                let loopback = scope.is_loopback();
                (Some(scope), Some(display), Some(loopback), None)
            }
            Err(_) => (
                None,
                Some(String::from("[INVALID PLANNER ENDPOINT]")),
                None,
                Some("planner_endpoint_invalid"),
            ),
        }
    }

    fn authoritative_remote_planner_allow_endpoint(&self) -> Result<String, ToolError> {
        let Some(profile_name) = self.config.providers.planner.remote_profile.as_deref() else {
            return Err(privacy_api_error(
                "remote_data_rule_invalid",
                "a persistent allow requires a configured remote planner profile",
                None,
            ));
        };
        let Some(profile) = self.config.remote_planner_profiles.get(profile_name) else {
            return Err(privacy_api_error(
                "remote_data_rule_invalid",
                "the configured remote planner profile was not found",
                None,
            ));
        };
        let scope = ProviderEndpointScope::parse(&profile.base_url).map_err(|_| {
            privacy_api_error(
                "remote_data_rule_invalid",
                "the configured remote planner endpoint is invalid",
                None,
            )
        })?;
        if scope.is_loopback() {
            return Err(privacy_api_error(
                "remote_data_rule_invalid",
                "persistent remote-data allows are not created for loopback planner endpoints",
                None,
            ));
        }
        Ok(scope.normalized_base_url().to_string())
    }

    fn upsert_remote_planner_origin_rule(
        &mut self,
        page_origin: String,
        decision: PersistedOriginDecision,
        current_origin_request: bool,
    ) -> Result<bool, ToolError> {
        if current_origin_request && matches!(decision, PersistedOriginDecision::Allow) {
            if let Some(reason_code) = self.current_remote_planner_high_risk_reason() {
                return Err(privacy_api_error(
                    "remote_data_high_risk_blocked",
                    "network planning is blocked for the current high-risk page context",
                    Some(serde_json::json!({ "reason_code": reason_code })),
                ));
            }
        }

        let mut settings = self.config.remote_planner_privacy.clone();
        let endpoint_scope = match decision {
            PersistedOriginDecision::Block => {
                settings
                    .origin_rules
                    .retain(|rule| rule.page_origin != page_origin);
                None
            }
            PersistedOriginDecision::Allow => {
                if settings.origin_rules.iter().any(|rule| {
                    rule.page_origin == page_origin
                        && matches!(rule.decision, PersistedOriginDecision::Block)
                }) {
                    return Err(privacy_api_error(
                        "remote_data_origin_blocked",
                        "the page origin has an origin-wide persistent block",
                        None,
                    ));
                }
                Some(self.authoritative_remote_planner_allow_endpoint()?)
            }
        };

        let exact_rule_exists = settings.origin_rules.iter().any(|rule| {
            rule.page_origin == page_origin
                && rule.decision == decision
                && rule.endpoint_scope == endpoint_scope
                && rule.policy_version == REMOTE_DATA_POLICY_VERSION
        });
        if exact_rule_exists {
            return Ok(false);
        }

        settings.origin_rules.push(RemotePlannerOriginRule {
            page_origin,
            decision,
            endpoint_scope,
            policy_version: REMOTE_DATA_POLICY_VERSION,
            created_at_ms: current_timestamp_ms(),
        });
        if settings.origin_rules.len() > MAX_PERSISTENT_ORIGIN_RULES {
            return Err(privacy_api_error(
                "remote_data_rule_limit_exceeded",
                "remote-data origin rules are limited to 256 entries",
                Some(serde_json::json!({ "limit": MAX_PERSISTENT_ORIGIN_RULES })),
            ));
        }
        self.persist_remote_planner_privacy_operation(settings, true)
    }

    fn revoke_remote_planner_origin_rule(
        &mut self,
        page_origin: &str,
        decision: PersistedOriginDecision,
        endpoint_scope: Option<&str>,
    ) -> Result<bool, ToolError> {
        let normalized_endpoint = match (&decision, endpoint_scope) {
            (PersistedOriginDecision::Block, Some(_)) => {
                return Err(privacy_api_error(
                    "remote_data_rule_invalid",
                    "an origin-wide block does not have an endpoint scope",
                    None,
                ));
            }
            (PersistedOriginDecision::Allow, Some(raw)) => Some(
                ProviderEndpointScope::parse(raw)
                    .map_err(|_| {
                        privacy_api_error(
                            "remote_data_rule_invalid",
                            "the persistent allow endpoint identifier is invalid",
                            None,
                        )
                    })?
                    .normalized_base_url()
                    .to_string(),
            ),
            _ => None,
        };

        let mut settings = self.config.remote_planner_privacy.clone();
        let previous_len = settings.origin_rules.len();
        settings.origin_rules.retain(|rule| {
            if rule.page_origin != page_origin || rule.decision != decision {
                return true;
            }
            match (&decision, normalized_endpoint.as_deref()) {
                (PersistedOriginDecision::Allow, Some(endpoint)) => {
                    rule.endpoint_scope.as_deref() != Some(endpoint)
                }
                _ => false,
            }
        });
        if settings.origin_rules.len() == previous_len {
            return Ok(false);
        }
        self.persist_remote_planner_privacy_operation(settings, true)
    }

    fn persist_remote_planner_privacy_operation(
        &mut self,
        mut settings: RemotePlannerPrivacySettings,
        invalidate_runtime: bool,
    ) -> Result<bool, ToolError> {
        prepare_privacy_settings_for_comparison(&mut settings);
        if settings == self.config.remote_planner_privacy {
            return Ok(false);
        }
        let updated =
            AppConfig::persist_remote_planner_privacy_settings_for_app(&self.app_handle, &settings)
                .map_err(|error| {
                    privacy_api_error(
                        "remote_planner_privacy_persist_failed",
                        "failed to persist the remote planner privacy policy",
                        Some(serde_json::json!({ "reason": error.to_string() })),
                    )
                })?;
        self.config = updated;
        if invalidate_runtime {
            self.clear_remote_planner_consent_runtime();
        }
        Ok(true)
    }
}

fn map_policy_status(
    result: RemotePlannerPolicyResult,
) -> (RemotePlannerEffectiveDecision, Option<String>) {
    match result {
        RemotePlannerPolicyResult::Allowed(authorization) => match authorization {
            RemotePlannerDataAuthorization::Loopback => {
                (RemotePlannerEffectiveDecision::LoopbackLocal, None)
            }
            RemotePlannerDataAuthorization::GlobalAllow => {
                (RemotePlannerEffectiveDecision::AllowedGlobal, None)
            }
            RemotePlannerDataAuthorization::PersistentAllow => {
                (RemotePlannerEffectiveDecision::AllowedPersistent, None)
            }
            RemotePlannerDataAuthorization::SessionAllow => {
                (RemotePlannerEffectiveDecision::AllowedSession, None)
            }
            RemotePlannerDataAuthorization::AllowOnce => (
                RemotePlannerEffectiveDecision::ConsentRequired,
                Some(String::from(
                    "one_shot_authorization_requires_pending_challenge",
                )),
            ),
        },
        RemotePlannerPolicyResult::ConsentRequired => (
            RemotePlannerEffectiveDecision::ConsentRequired,
            Some(String::from("consent_required")),
        ),
        RemotePlannerPolicyResult::Blocked { code, reason_code } => {
            let decision = match code {
                "remote_data_local_only" => RemotePlannerEffectiveDecision::LocalOnly,
                "remote_data_high_risk_blocked" => RemotePlannerEffectiveDecision::HighRiskBlocked,
                "remote_data_origin_blocked" => RemotePlannerEffectiveDecision::OriginBlocked,
                "remote_data_opaque_origin_blocked" => {
                    RemotePlannerEffectiveDecision::OriginUnavailable
                }
                _ => RemotePlannerEffectiveDecision::PlannerUnavailable,
            };
            (decision, Some(reason_code.to_string()))
        }
    }
}

fn page_origin_from_url(raw: &str) -> Option<String> {
    let parsed = match url::Url::parse(raw.trim()) {
        Ok(parsed) => parsed,
        Err(_) => return None,
    };
    if !matches!(parsed.scheme(), "http" | "https")
        || parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.origin().ascii_serialization() == "null"
    {
        return None;
    }
    Some(parsed.origin().ascii_serialization())
}

fn normalize_http_origin(raw: &str) -> Result<String, ToolError> {
    let parsed = url::Url::parse(raw.trim()).map_err(|_| {
        privacy_api_error(
            "remote_data_rule_invalid",
            "remote-data origin must be an absolute HTTP(S) origin",
            None,
        )
    })?;
    if !matches!(parsed.scheme(), "http" | "https")
        || parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || !matches!(parsed.path(), "" | "/")
        || parsed.origin().ascii_serialization() == "null"
    {
        return Err(privacy_api_error(
            "remote_data_rule_invalid",
            "remote-data origin must contain only HTTP(S) scheme, host, and optional port",
            None,
        ));
    }
    Ok(parsed.origin().ascii_serialization())
}

fn normalized_endpoint_scope(raw: &str) -> Option<String> {
    match ProviderEndpointScope::parse(raw) {
        Ok(scope) => Some(scope.normalized_base_url().to_string()),
        Err(_) => None,
    }
}

fn build_rule_status(
    rule: &RemotePlannerOriginRule,
    current_endpoint: Option<&str>,
) -> RemotePlannerOriginRuleStatus {
    let normalized_rule_endpoint = rule
        .endpoint_scope
        .as_deref()
        .and_then(normalized_endpoint_scope);
    let endpoint_display = match (&rule.endpoint_scope, &normalized_rule_endpoint) {
        (None, _) => None,
        (Some(_), Some(endpoint)) => Some(endpoint.clone()),
        (Some(_), None) => Some(String::from("[INVALID ENDPOINT]")),
    };
    RemotePlannerOriginRuleStatus {
        page_origin: rule.page_origin.clone(),
        decision: rule.decision.clone(),
        endpoint_scope: normalized_rule_endpoint,
        endpoint_display,
        policy_version: rule.policy_version,
        created_at_ms: rule.created_at_ms,
        stale: rule_is_stale(rule, current_endpoint),
    }
}

fn rule_is_stale(rule: &RemotePlannerOriginRule, current_endpoint: Option<&str>) -> bool {
    if !matches!(rule.decision, PersistedOriginDecision::Allow) {
        return false;
    }
    if rule.policy_version != REMOTE_DATA_POLICY_VERSION {
        return true;
    }
    let normalized_rule_endpoint = rule
        .endpoint_scope
        .as_deref()
        .and_then(normalized_endpoint_scope);
    normalized_rule_endpoint.as_deref() != current_endpoint
}

fn persistent_rule_for_origin(
    rules: &[RemotePlannerOriginRule],
    page_origin: &str,
    current_endpoint: Option<&str>,
) -> Option<PersistedOriginDecision> {
    if rules.iter().any(|rule| {
        rule.page_origin == page_origin && matches!(rule.decision, PersistedOriginDecision::Block)
    }) {
        return Some(PersistedOriginDecision::Block);
    }
    if rules.iter().any(|rule| {
        rule.page_origin == page_origin
            && matches!(rule.decision, PersistedOriginDecision::Allow)
            && !rule_is_stale(rule, current_endpoint)
    }) {
        return Some(PersistedOriginDecision::Allow);
    }
    rules
        .iter()
        .find(|rule| {
            rule.page_origin == page_origin
                && matches!(rule.decision, PersistedOriginDecision::Allow)
        })
        .map(|rule| rule.decision.clone())
}

fn decision_sort_key(decision: &PersistedOriginDecision) -> u8 {
    match decision {
        PersistedOriginDecision::Block => 0,
        PersistedOriginDecision::Allow => 1,
    }
}

fn prepare_privacy_settings_for_comparison(settings: &mut RemotePlannerPrivacySettings) {
    settings.policy_schema_version = REMOTE_DATA_POLICY_VERSION;
    settings.local_only = matches!(settings.network_mode, RemotePlannerNetworkMode::LocalOnly);
    settings.consent_to_remote_page_data = matches!(
        settings.network_mode,
        RemotePlannerNetworkMode::AllowSanitizedNonHighRisk
    );
    settings.origin_rules.sort_by(|left, right| {
        (
            left.page_origin.as_str(),
            decision_sort_key(&left.decision),
            left.endpoint_scope.as_deref(),
            left.policy_version,
            left.created_at_ms,
        )
            .cmp(&(
                right.page_origin.as_str(),
                decision_sort_key(&right.decision),
                right.endpoint_scope.as_deref(),
                right.policy_version,
                right.created_at_ms,
            ))
    });
    settings.origin_rules.dedup_by(|left, right| {
        left.page_origin == right.page_origin
            && left.decision == right.decision
            && left.endpoint_scope == right.endpoint_scope
            && left.policy_version == right.policy_version
    });
    settings.blocked_origins = settings
        .origin_rules
        .iter()
        .filter(|rule| matches!(rule.decision, PersistedOriginDecision::Block))
        .map(|rule| rule.page_origin.clone())
        .collect();
    settings.blocked_origins.sort();
    settings.blocked_origins.dedup();
}

fn privacy_api_error(code: &str, message: &str, details: Option<serde_json::Value>) -> ToolError {
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

    #[test]
    fn page_url_origin_extraction_accepts_paths_without_exposing_them() {
        assert_eq!(
            page_origin_from_url("https://EXAMPLE.com:443/private?q=secret#fragment"),
            Some(String::from("https://example.com"))
        );
        assert_eq!(page_origin_from_url("file:///tmp/private"), None);
        assert_eq!(page_origin_from_url("data:text/plain,private"), None);
    }

    #[test]
    fn origin_normalization_is_strict_and_deterministic() {
        assert_eq!(
            normalize_http_origin("https://EXAMPLE.com:443/").unwrap(),
            "https://example.com"
        );
        for invalid in [
            "https://example.com/private",
            "https://user:pass@example.com",
            "https://example.com?token=secret",
            "file:///tmp/private",
            "data:text/plain,private",
        ] {
            assert!(normalize_http_origin(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn stale_allow_detection_binds_endpoint_and_policy_version() {
        let active = RemotePlannerOriginRule {
            page_origin: String::from("https://example.com"),
            decision: PersistedOriginDecision::Allow,
            endpoint_scope: Some(String::from("https://api.example.com/v1")),
            policy_version: REMOTE_DATA_POLICY_VERSION,
            created_at_ms: 1,
        };
        assert!(!rule_is_stale(&active, Some("https://api.example.com/v1")));
        assert!(rule_is_stale(&active, Some("https://api.example.com/v2")));
        let mut stale_version = active.clone();
        stale_version.policy_version = REMOTE_DATA_POLICY_VERSION + 1;
        assert!(rule_is_stale(
            &stale_version,
            Some("https://api.example.com/v1")
        ));
    }

    #[test]
    fn legacy_fields_are_derived_from_typed_settings() {
        let mut settings = RemotePlannerPrivacySettings {
            network_mode: RemotePlannerNetworkMode::LocalOnly,
            origin_rules: vec![RemotePlannerOriginRule {
                page_origin: String::from("https://example.com"),
                decision: PersistedOriginDecision::Block,
                endpoint_scope: None,
                policy_version: REMOTE_DATA_POLICY_VERSION,
                created_at_ms: 1,
            }],
            ..RemotePlannerPrivacySettings::default()
        };
        prepare_privacy_settings_for_comparison(&mut settings);
        assert!(settings.local_only);
        assert!(!settings.consent_to_remote_page_data);
        assert_eq!(settings.blocked_origins, vec!["https://example.com"]);
    }
}
