use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};
use url::Url;

use super::interaction_tools::resolve_clickable_element;
use crate::commands::{
    current_timestamp_ms, evaluate_action_policy, normalized_origin, ConfirmationRequirement,
    PlannedStep, PlannerOutput, PlannerSafetySettings, PlannerStatus, StepTransition, ToolError,
    ToolName, CLICK_AUTH_AMBIGUOUS_ARG, CLICK_AUTH_CONFIDENCE_ARG, CLICK_AUTH_DESTRUCTIVE_ARG,
    CLICK_AUTH_GENERATION_ARG, CLICK_AUTH_TOKEN_ARG, RUNTIME_FORM_DESTINATION_ARG,
    RUNTIME_FORM_FIELDS_ARG, RUNTIME_FORM_LABEL_ARG, RUNTIME_TARGET_LABEL_ARG,
};
use crate::page_model::{ElementRole, InteractiveElement};
use crate::state::ClickAuthorizationRecord;

const CLICK_AUTHORIZATION_TTL_MS: u64 = 30_000;
const MAX_CLICK_AUTHORIZATIONS: usize = 32;
static CLICK_AUTHORIZATION_COUNTER: AtomicU64 = AtomicU64::new(1);

impl super::AppCore {
    pub(crate) fn issue_find_element_click_authorization(
        &mut self,
        request_id: &str,
        element_id: &str,
        confidence_bps: u16,
    ) -> Result<String, ToolError> {
        self.mint_click_authorization(request_id, element_id, Some(confidence_bps), false)
            .map(|record| record.token)
    }

    pub(crate) fn prepare_planner_output_for_execution(
        &mut self,
        planner_output: &PlannerOutput,
    ) -> Result<PlannerOutput, ToolError> {
        self.prune_click_authorizations();
        let mut prepared = planner_output.clone();
        let mut used_tokens = HashSet::new();

        for step in &mut prepared.steps {
            clear_runtime_annotations(step);
            match step.tool_name {
                ToolName::ClickElement => {
                    let element_id = required_string_argument(step, "element_id")?;
                    let provided_token = step
                        .arguments
                        .get(CLICK_AUTH_TOKEN_ARG)
                        .and_then(serde_json::Value::as_str)
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(ToOwned::to_owned);
                    let record = if let Some(token) = provided_token {
                        self.validate_stored_click_authorization(&token, &element_id, false)?
                    } else if let Some(existing) =
                        self.latest_click_authorization_for_element(&element_id)
                    {
                        self.mint_click_authorization(
                            &step.step_id,
                            &element_id,
                            existing.confidence_bps,
                            existing.ambiguous,
                        )?
                    } else {
                        self.mint_click_authorization(&step.step_id, &element_id, None, true)?
                    };
                    if !used_tokens.insert(record.token.clone()) {
                        return Err(click_error(
                            "duplicate_click_authorization",
                            "one click authorization token cannot authorize multiple planned clicks",
                            None,
                        ));
                    }
                    annotate_click_step(step, &record)?;
                }
                ToolName::TypeIntoElement => self.annotate_target_step(step)?,
                ToolName::SubmitActiveForm => self.annotate_submit_step(step)?,
                _ => {}
            }
        }

        let safety = PlannerSafetySettings::from(&self.config.safety);
        let decision = evaluate_action_policy(&prepared.steps, &safety);
        if prepared.status == PlannerStatus::Ready
            && decision.requirement == ConfirmationRequirement::ConfirmationRequired
            && decision
                .findings
                .iter()
                .all(|finding| finding.tool_name == ToolName::ClickElement)
        {
            insert_deterministic_click_confirmation_gate(&mut prepared);
        }

        Ok(prepared)
    }

    pub(crate) fn preflight_pending_click_authorizations(
        &mut self,
        steps: &[PlannedStep],
    ) -> Result<(), ToolError> {
        let mut seen = HashSet::new();
        for step in steps {
            if step.tool_name != ToolName::ClickElement {
                continue;
            }
            let token = required_string_argument(step, CLICK_AUTH_TOKEN_ARG)?;
            if !seen.insert(token.clone()) {
                return Err(click_error(
                    "duplicate_click_authorization",
                    "pending protected actions reused one click authorization token",
                    None,
                ));
            }
            let element_id = required_string_argument(step, "element_id")?;
            self.validate_stored_click_authorization(&token, &element_id, true)?;
        }
        Ok(())
    }

    pub(crate) fn preflight_planned_step_runtime(
        &mut self,
        step: &PlannedStep,
    ) -> Result<(), ToolError> {
        if step.tool_name != ToolName::ClickElement {
            return Ok(());
        }
        let token = required_string_argument(step, CLICK_AUTH_TOKEN_ARG)?;
        let element_id = required_string_argument(step, "element_id")?;
        self.validate_stored_click_authorization(&token, &element_id, true)?;
        self.state.click_authorizations.remove(&token);
        Ok(())
    }

    fn latest_click_authorization_for_element(
        &self,
        element_id: &str,
    ) -> Option<ClickAuthorizationRecord> {
        let now_ms = current_timestamp_ms();
        self.state
            .click_authorizations
            .values()
            .filter(|record| {
                record.element_id == element_id
                    && record.page_id == self.state.current_page_id.as_deref().unwrap_or_default()
                    && record.page_generation == self.state.page_generation
                    && record.expires_at_ms > now_ms
            })
            .max_by_key(|record| record.issued_at_ms)
            .cloned()
    }

    fn mint_click_authorization(
        &mut self,
        request_id: &str,
        element_id: &str,
        confidence_bps: Option<u16>,
        ambiguous: bool,
    ) -> Result<ClickAuthorizationRecord, ToolError> {
        let page_id = self.state.current_page_id.clone().ok_or_else(|| {
            click_error(
                "no_active_page",
                "click authorization requires an active page",
                None,
            )
        })?;
        let page = self.state.current_page.as_ref().ok_or_else(|| {
            click_error(
                "missing_page_model",
                "click authorization requires a current page model",
                Some(serde_json::json!({ "page_id": page_id })),
            )
        })?;
        let element = resolve_clickable_element(page, element_id)?.clone();
        let dom_locator = element
            .dom_locator
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .expect("resolve_clickable_element requires a locator")
            .to_string();
        let now_ms = current_timestamp_ms();
        let counter = CLICK_AUTHORIZATION_COUNTER.fetch_add(1, Ordering::Relaxed);
        let token = format!("click-auth-{request_id}-{now_ms}-{counter}");
        let record = ClickAuthorizationRecord {
            token: token.clone(),
            page_id,
            page_generation: self.state.page_generation,
            origin: normalized_origin(page.url.as_deref()),
            element_id: element.element_id.clone(),
            label: safe_element_label(&element),
            dom_locator,
            element_fingerprint: element_fingerprint(&element),
            confidence_bps,
            ambiguous,
            potentially_destructive: is_potentially_destructive_click(&element),
            issued_at_ms: now_ms,
            expires_at_ms: now_ms.saturating_add(CLICK_AUTHORIZATION_TTL_MS),
        };
        self.state
            .click_authorizations
            .insert(token, record.clone());
        self.prune_click_authorizations();
        Ok(record)
    }

    fn validate_stored_click_authorization(
        &mut self,
        token: &str,
        element_id: &str,
        validate_live_dom: bool,
    ) -> Result<ClickAuthorizationRecord, ToolError> {
        let record = self
            .state
            .click_authorizations
            .get(token)
            .cloned()
            .ok_or_else(|| {
                click_error(
                    "unknown_click_authorization",
                    "click authorization token is unknown, consumed, or expired",
                    None,
                )
            })?;
        let now_ms = current_timestamp_ms();
        if now_ms >= record.expires_at_ms {
            self.state.click_authorizations.remove(token);
            return Err(click_error(
                "click_authorization_expired",
                "click authorization expired before dispatch",
                Some(serde_json::json!({
                    "expired_at_ms": record.expires_at_ms,
                    "observed_at_ms": now_ms,
                })),
            ));
        }
        if record.element_id != element_id
            || self.state.current_page_id.as_deref() != Some(record.page_id.as_str())
            || self.state.page_generation != record.page_generation
            || normalized_origin(
                self.state
                    .current_page
                    .as_ref()
                    .and_then(|page| page.url.as_deref()),
            ) != record.origin
        {
            return Err(click_error(
                "stale_click_authorization",
                "click authorization no longer matches the active page generation and target",
                Some(serde_json::json!({
                    "expected_page_id": record.page_id,
                    "observed_page_id": self.state.current_page_id,
                    "expected_page_generation": record.page_generation,
                    "observed_page_generation": self.state.page_generation,
                    "expected_element_id": record.element_id,
                    "observed_element_id": element_id,
                })),
            ));
        }

        let page = self.state.current_page.as_ref().ok_or_else(|| {
            click_error(
                "missing_page_model",
                "click authorization cannot be checked without a current page model",
                None,
            )
        })?;
        let current_element = resolve_clickable_element(page, element_id)?;
        verify_element_matches_record(current_element, &record)?;

        if validate_live_dom {
            let live_page = self.browser.extract_page_model().map_err(|error| {
                click_error(
                    "click_live_revalidation_failed",
                    "live DOM could not be re-extracted before click dispatch",
                    Some(serde_json::json!({ "reason": error.to_string() })),
                )
            })?;
            let live_element =
                resolve_clickable_element(&live_page, element_id).map_err(|error| {
                    click_error(
                        "click_target_changed",
                        "the authorized click target no longer resolves in the live DOM",
                        Some(serde_json::json!({
                            "element_id": element_id,
                            "reason_code": error.code,
                        })),
                    )
                })?;
            verify_element_matches_record(live_element, &record)?;
        }

        Ok(record)
    }

    fn prune_click_authorizations(&mut self) {
        let now_ms = current_timestamp_ms();
        self.state
            .click_authorizations
            .retain(|_, record| record.expires_at_ms > now_ms);
        while self.state.click_authorizations.len() > MAX_CLICK_AUTHORIZATIONS {
            let oldest = self
                .state
                .click_authorizations
                .iter()
                .min_by_key(|(_, record)| record.issued_at_ms)
                .map(|(token, _)| token.clone());
            let Some(oldest) = oldest else { break };
            self.state.click_authorizations.remove(&oldest);
        }
    }

    fn annotate_target_step(&self, step: &mut PlannedStep) -> Result<(), ToolError> {
        let element_id = required_string_argument(step, "element_id")?;
        let page = self.state.current_page.as_ref().ok_or_else(|| {
            click_error(
                "missing_page_model",
                "target summary requires a current page model",
                None,
            )
        })?;
        let element = page
            .interactive_elements
            .iter()
            .find(|element| element.element_id == element_id)
            .ok_or_else(|| {
                click_error(
                    "unknown_element_id",
                    "target summary requires an element from the current page model",
                    Some(serde_json::json!({ "element_id": element_id })),
                )
            })?;
        insert_runtime_value(
            step,
            RUNTIME_TARGET_LABEL_ARG,
            serde_json::Value::String(safe_element_label(element)),
        )
    }

    fn annotate_submit_step(&self, step: &mut PlannedStep) -> Result<(), ToolError> {
        let Some(page) = self.state.current_page.as_ref() else {
            return Ok(());
        };
        let requested_form_id = step
            .arguments
            .get("form_element_id")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let form = requested_form_id
            .and_then(|form_id| {
                page.interactive_elements.iter().find(|element| {
                    element.element_id == form_id && element.role == ElementRole::Form
                })
            })
            .or_else(|| {
                let mut forms = page
                    .interactive_elements
                    .iter()
                    .filter(|element| element.role == ElementRole::Form && element.visible);
                let first = forms.next()?;
                forms.next().is_none().then_some(first)
            });
        if let Some(form) = form {
            insert_runtime_value(
                step,
                RUNTIME_FORM_LABEL_ARG,
                serde_json::Value::String(safe_element_label(form)),
            )?;
        }
        if let Some(destination) = form_destination(page.url.as_deref(), form) {
            insert_runtime_value(
                step,
                RUNTIME_FORM_DESTINATION_ARG,
                serde_json::Value::String(destination),
            )?;
        }
        let fields = page
            .interactive_elements
            .iter()
            .filter(|element| {
                matches!(
                    element.role,
                    ElementRole::Input | ElementRole::TextArea | ElementRole::Select
                ) && !sensitive_field_label(element)
            })
            .map(safe_element_label)
            .filter(|label| !label.is_empty())
            .take(8)
            .map(serde_json::Value::String)
            .collect::<Vec<_>>();
        insert_runtime_value(
            step,
            RUNTIME_FORM_FIELDS_ARG,
            serde_json::Value::Array(fields),
        )
    }
}

fn annotate_click_step(
    step: &mut PlannedStep,
    record: &ClickAuthorizationRecord,
) -> Result<(), ToolError> {
    insert_runtime_value(
        step,
        CLICK_AUTH_TOKEN_ARG,
        serde_json::Value::String(record.token.clone()),
    )?;
    // Restore the human-readable label so the confirmation prompt names the
    // element instead of its opaque internal id. `clear_runtime_annotations`
    // strips RUNTIME_TARGET_LABEL_ARG from every step up front; TypeIntoElement
    // and SubmitActiveForm re-derive their own copies via annotate_target_step /
    // annotate_submit_step, and clicks must do the same here.
    insert_runtime_value(
        step,
        RUNTIME_TARGET_LABEL_ARG,
        serde_json::Value::String(record.label.clone()),
    )?;
    insert_runtime_value(
        step,
        CLICK_AUTH_CONFIDENCE_ARG,
        record
            .confidence_bps
            .map(|value| serde_json::Value::from(u64::from(value)))
            .unwrap_or(serde_json::Value::Null),
    )?;
    insert_runtime_value(
        step,
        CLICK_AUTH_AMBIGUOUS_ARG,
        serde_json::Value::Bool(record.ambiguous),
    )?;
    insert_runtime_value(
        step,
        CLICK_AUTH_DESTRUCTIVE_ARG,
        serde_json::Value::Bool(record.potentially_destructive),
    )?;
    insert_runtime_value(
        step,
        CLICK_AUTH_GENERATION_ARG,
        serde_json::Value::from(record.page_generation),
    )
}

fn clear_runtime_annotations(step: &mut PlannedStep) {
    let Some(arguments) = step.arguments.as_object_mut() else {
        return;
    };
    for key in [
        CLICK_AUTH_CONFIDENCE_ARG,
        CLICK_AUTH_AMBIGUOUS_ARG,
        CLICK_AUTH_DESTRUCTIVE_ARG,
        CLICK_AUTH_GENERATION_ARG,
        RUNTIME_TARGET_LABEL_ARG,
        RUNTIME_FORM_LABEL_ARG,
        RUNTIME_FORM_DESTINATION_ARG,
        RUNTIME_FORM_FIELDS_ARG,
    ] {
        arguments.remove(key);
    }
}

fn insert_runtime_value(
    step: &mut PlannedStep,
    name: &str,
    value: serde_json::Value,
) -> Result<(), ToolError> {
    let arguments = step.arguments.as_object_mut().ok_or_else(|| {
        click_error(
            "invalid_tool_arguments",
            "runtime authorization requires object-shaped tool arguments",
            Some(serde_json::json!({ "step_id": step.step_id })),
        )
    })?;
    arguments.insert(name.to_string(), value);
    Ok(())
}

fn required_string_argument(step: &PlannedStep, name: &str) -> Result<String, ToolError> {
    step.arguments
        .get(name)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            click_error(
                "missing_runtime_argument",
                &format!("planned step is missing required runtime argument '{name}'"),
                Some(serde_json::json!({ "step_id": step.step_id })),
            )
        })
}

fn verify_element_matches_record(
    element: &InteractiveElement,
    record: &ClickAuthorizationRecord,
) -> Result<(), ToolError> {
    let locator = element
        .dom_locator
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if locator != Some(record.dom_locator.as_str())
        || element_fingerprint(element) != record.element_fingerprint
        || !element.visible
        || !element.enabled
    {
        return Err(click_error(
            "click_target_changed",
            "the click target changed after authorization was issued",
            Some(serde_json::json!({ "element_id": record.element_id })),
        ));
    }
    Ok(())
}

fn element_fingerprint(element: &InteractiveElement) -> String {
    let value = serde_json::json!({
        "element_id": element.element_id,
        "dom_locator": element.dom_locator,
        "role": element.role,
        "tag_name": element.tag_name,
        "text": element.text,
        "accessible_name": element.accessible_name,
        "placeholder": element.placeholder,
        "href": element.href,
        "visible": element.visible,
        "enabled": element.enabled,
    });
    let encoded = serde_json::to_vec(&value).expect("element fingerprint should serialize");
    format!("{:x}", Sha256::digest(encoded))
}

fn safe_element_label(element: &InteractiveElement) -> String {
    element
        .accessible_name
        .as_deref()
        .or(element.text.as_deref())
        .or(element.placeholder.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(&element.element_id)
        .chars()
        .filter(|character| !character.is_control())
        .take(80)
        .collect()
}

fn is_potentially_destructive_click(element: &InteractiveElement) -> bool {
    let text = format!(
        "{} {} {}",
        element.accessible_name.as_deref().unwrap_or_default(),
        element.text.as_deref().unwrap_or_default(),
        element.href.as_deref().unwrap_or_default(),
    )
    .to_ascii_lowercase();
    [
        "delete",
        "remove",
        "purchase",
        "buy now",
        "pay",
        "place order",
        "transfer",
        "send money",
        "confirm order",
        "close account",
        "sign out",
        "log out",
        "publish",
        "post",
    ]
    .iter()
    .any(|keyword| text.contains(keyword))
}

fn sensitive_field_label(element: &InteractiveElement) -> bool {
    let label = format!(
        "{} {} {} {}",
        element.element_id,
        element.accessible_name.as_deref().unwrap_or_default(),
        element.placeholder.as_deref().unwrap_or_default(),
        element
            .attributes
            .get("type")
            .map(String::as_str)
            .unwrap_or_default(),
    )
    .to_ascii_lowercase();
    [
        "password",
        "passwd",
        "token",
        "secret",
        "one-time",
        "otp",
        "credit card",
        "card number",
        "cvv",
        "cvc",
        "ssn",
        "social security",
        "security answer",
    ]
    .iter()
    .any(|keyword| label.contains(keyword))
}

fn form_destination(page_url: Option<&str>, form: Option<&InteractiveElement>) -> Option<String> {
    let page_url = page_url?;
    let action = form.and_then(|form| form.attributes.get("action"));
    let destination = match action.map(String::as_str).map(str::trim) {
        Some(action) if !action.is_empty() => Url::parse(page_url).ok()?.join(action).ok()?,
        _ => Url::parse(page_url).ok()?,
    };
    normalized_origin(Some(destination.as_str()))
}

fn insert_deterministic_click_confirmation_gate(planner_output: &mut PlannerOutput) {
    let mut step_id = String::from("runtime-confirm-click");
    let existing = planner_output
        .steps
        .iter()
        .map(|step| step.step_id.as_str())
        .collect::<HashSet<_>>();
    let mut suffix = 1_u32;
    while existing.contains(step_id.as_str()) {
        step_id = format!("runtime-confirm-click-{suffix}");
        suffix += 1;
    }
    planner_output.steps.insert(
        0,
        PlannedStep {
            step_id,
            tool_name: ToolName::ConfirmAction,
            arguments: serde_json::json!({
                "request_id": "runtime-click-confirmation",
                "timeout_ms": 120000,
                "prompt_text": "Runtime-generated click confirmation",
                "reason": "deterministic click policy requires confirmation"
            }),
            purpose: String::from("request deterministic click confirmation"),
            on_success: StepTransition::RequestConfirmation,
            on_failure: StepTransition::Replan,
        },
    );
    planner_output.status = PlannerStatus::NeedsConfirmation;
    planner_output.requires_confirmation = true;
    planner_output.confirmation_reason = Some(String::from(
        "Deterministic runtime click policy requires confirmation.",
    ));
    planner_output.user_message = Some(String::from(
        "Please confirm the selected page interaction.",
    ));
}

fn click_error(code: &str, message: &str, details: Option<serde_json::Value>) -> ToolError {
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
    use std::collections::BTreeMap;

    fn element(label: &str) -> InteractiveElement {
        InteractiveElement {
            element_id: String::from("button-1"),
            dom_locator: Some(String::from("#button-1")),
            role: ElementRole::Button,
            tag_name: String::from("button"),
            text: Some(label.to_string()),
            accessible_name: Some(label.to_string()),
            placeholder: None,
            href: None,
            value: None,
            bbox: None,
            visible: true,
            enabled: true,
            attributes: BTreeMap::new(),
        }
    }

    #[test]
    fn element_fingerprint_changes_with_locator_or_label() {
        let first = element("Continue");
        let mut changed = first.clone();
        changed.dom_locator = Some(String::from("#replacement"));
        assert_ne!(element_fingerprint(&first), element_fingerprint(&changed));
        changed = first.clone();
        changed.accessible_name = Some(String::from("Delete account"));
        assert_ne!(element_fingerprint(&first), element_fingerprint(&changed));
    }

    #[test]
    fn destructive_click_labels_are_detected_deterministically() {
        assert!(!is_potentially_destructive_click(&element("Continue")));
        assert!(is_potentially_destructive_click(&element("Delete account")));
    }

    #[test]
    fn annotate_click_step_restores_the_runtime_target_label() {
        // Regression test for the bug where clear_runtime_annotations stripped
        // RUNTIME_TARGET_LABEL_ARG and annotate_click_step never restored it,
        // leaving click confirmations naming an opaque "element-N" id instead
        // of the element the user asked to click.
        let record = ClickAuthorizationRecord {
            token: String::from("token-1"),
            page_id: String::from("page-1"),
            page_generation: 1,
            origin: Some(String::from("https://example.com")),
            element_id: String::from("element-7"),
            label: String::from("Delete account"),
            dom_locator: String::from("#delete"),
            element_fingerprint: String::from("fingerprint"),
            confidence_bps: Some(9_500),
            ambiguous: false,
            potentially_destructive: true,
            issued_at_ms: 1,
            expires_at_ms: u64::MAX,
        };
        let mut step = PlannedStep {
            step_id: String::from("step-1"),
            tool_name: ToolName::ClickElement,
            arguments: serde_json::json!({ "element_id": "element-7" }),
            purpose: String::from("test click"),
            on_success: StepTransition::Complete,
            on_failure: StepTransition::Replan,
        };

        annotate_click_step(&mut step, &record).expect("annotation should succeed");

        assert_eq!(
            step.arguments
                .get(RUNTIME_TARGET_LABEL_ARG)
                .and_then(serde_json::Value::as_str),
            Some("Delete account"),
        );
    }
}
