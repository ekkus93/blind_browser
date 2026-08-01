use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

use super::{PendingPlanExecutionState, PlannedStep, ToolError, ToolName};

pub const DEFAULT_CONFIRMATION_TTL_MS: u64 = 120_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmationRuntimeContext {
    pub page_id: Option<String>,
    pub page_url: Option<String>,
    pub now_ms: u64,
}

impl ConfirmationRuntimeContext {
    pub fn current(page_id: Option<String>, page_url: Option<String>) -> Self {
        Self {
            page_id,
            page_url,
            now_ms: super::current_timestamp_ms(),
        }
    }

    pub fn detached() -> Self {
        Self::current(None, None)
    }

    #[cfg(test)]
    pub fn at(page_id: Option<&str>, page_url: Option<&str>, now_ms: u64) -> Self {
        Self {
            page_id: page_id.map(ToOwned::to_owned),
            page_url: page_url.map(ToOwned::to_owned),
            now_ms,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ConfirmationActionManifest {
    pub sequence: u16,
    pub step_id: String,
    pub tool_name: ToolName,
    pub argument_digest: String,
    pub safe_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct ConfirmationManifest {
    pub request_id: String,
    pub page_id: Option<String>,
    pub origin: Option<String>,
    pub issued_at_ms: u64,
    pub expires_at_ms: u64,
    pub actions: Vec<ConfirmationActionManifest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuiltConfirmationManifest {
    pub manifest: ConfirmationManifest,
    pub digest: String,
    pub prompt_text: String,
}

pub fn build_confirmation_manifest(
    request_id: &str,
    queued_steps: &[PlannedStep],
    context: &ConfirmationRuntimeContext,
) -> Result<BuiltConfirmationManifest, ToolError> {
    build_confirmation_manifest_at(
        request_id,
        queued_steps,
        context.page_id.clone(),
        normalized_origin(context.page_url.as_deref()),
        context.now_ms,
        context.now_ms.saturating_add(DEFAULT_CONFIRMATION_TTL_MS),
    )
}

fn build_confirmation_manifest_at(
    request_id: &str,
    queued_steps: &[PlannedStep],
    page_id: Option<String>,
    origin: Option<String>,
    issued_at_ms: u64,
    expires_at_ms: u64,
) -> Result<BuiltConfirmationManifest, ToolError> {
    if queued_steps.is_empty() {
        return Err(confirmation_error(
            "empty_confirmation_manifest",
            "confirmation cannot be requested without queued actions",
            None,
        ));
    }

    let mut actions = Vec::with_capacity(queued_steps.len());
    for (index, step) in queued_steps.iter().enumerate() {
        let sequence = u16::try_from(index).map_err(|_| {
            confirmation_error(
                "confirmation_manifest_too_large",
                "confirmation action sequence exceeded the supported size",
                None,
            )
        })?;
        actions.push(ConfirmationActionManifest {
            sequence,
            step_id: step.step_id.clone(),
            tool_name: step.tool_name.clone(),
            argument_digest: digest_json(&step.arguments),
            safe_summary: safe_action_summary(step),
        });
    }

    let manifest = ConfirmationManifest {
        request_id: request_id.to_string(),
        page_id,
        origin,
        issued_at_ms,
        expires_at_ms,
        actions,
    };
    let digest = digest_serializable(&manifest)?;
    let prompt_text = deterministic_prompt(&manifest);

    Ok(BuiltConfirmationManifest {
        manifest,
        digest,
        prompt_text,
    })
}

pub fn validate_pending_confirmation_manifest(
    pending: &PendingPlanExecutionState,
    supplied_digest: &str,
    context: &ConfirmationRuntimeContext,
) -> Result<(), ToolError> {
    if supplied_digest.trim().is_empty() || supplied_digest != pending.manifest_digest {
        return Err(confirmation_error(
            "confirmation_digest_mismatch",
            "confirmation response did not match the pending action manifest",
            Some(serde_json::json!({
                "received_digest_present": !supplied_digest.trim().is_empty(),
            })),
        ));
    }

    if context.now_ms >= pending.manifest.expires_at_ms {
        return Err(confirmation_error(
            "confirmation_expired",
            "the pending confirmation expired before it was approved",
            Some(serde_json::json!({
                "expired_at_ms": pending.manifest.expires_at_ms,
                "observed_at_ms": context.now_ms,
            })),
        ));
    }

    if pending.manifest.page_id != context.page_id {
        return Err(confirmation_error(
            "confirmation_page_changed",
            "the active page changed after the confirmation challenge was created",
            Some(serde_json::json!({
                "expected_page_id": pending.manifest.page_id,
                "observed_page_id": context.page_id,
            })),
        ));
    }

    let observed_origin = normalized_origin(context.page_url.as_deref());
    if pending.manifest.origin != observed_origin {
        return Err(confirmation_error(
            "confirmation_origin_changed",
            "the active origin changed after the confirmation challenge was created",
            Some(serde_json::json!({
                "expected_origin": pending.manifest.origin,
                "observed_origin": observed_origin,
            })),
        ));
    }

    let expected_ids = pending
        .queued_steps
        .iter()
        .map(|step| step.step_id.clone())
        .collect::<Vec<_>>();
    if expected_ids != pending.queued_step_ids
        || pending.next_step_id.as_ref() != expected_ids.first()
    {
        return Err(confirmation_error(
            "confirmation_queue_mismatch",
            "the queued action graph changed after confirmation was requested",
            None,
        ));
    }

    let rebuilt = build_confirmation_manifest_at(
        &pending.request_id,
        &pending.queued_steps,
        pending.manifest.page_id.clone(),
        pending.manifest.origin.clone(),
        pending.manifest.issued_at_ms,
        pending.manifest.expires_at_ms,
    )?;
    if rebuilt.manifest != pending.manifest || rebuilt.digest != pending.manifest_digest {
        return Err(confirmation_error(
            "confirmation_manifest_mismatch",
            "the pending actions no longer match the approved confirmation manifest",
            None,
        ));
    }

    Ok(())
}

pub fn normalized_origin(value: Option<&str>) -> Option<String> {
    let parsed = Url::parse(value?.trim()).ok()?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host().is_none() {
        return None;
    }
    Some(parsed.origin().ascii_serialization())
}

fn deterministic_prompt(manifest: &ConfirmationManifest) -> String {
    let destination = manifest
        .origin
        .as_deref()
        .map(|origin| format!(" on {origin}"))
        .unwrap_or_default();
    if manifest.actions.len() == 1 {
        return format!(
            "Approve this action{destination}: {}",
            manifest.actions[0].safe_summary
        );
    }

    let actions = manifest
        .actions
        .iter()
        .map(|action| format!("{}. {}", action.sequence + 1, action.safe_summary))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "Approve these {} actions{destination}: {actions}",
        manifest.actions.len()
    )
}

fn safe_action_summary(step: &PlannedStep) -> String {
    match &step.tool_name {
        ToolName::SubmitActiveForm => String::from("Submit the active form."),
        ToolName::ClickElement => string_argument(step, "element_id")
            .map(|value| format!("Click element '{}'.", safe_label(value)))
            .unwrap_or_else(|| String::from("Click the selected page element.")),
        ToolName::FocusElement => string_argument(step, "element_id")
            .map(|value| format!("Focus element '{}'.", safe_label(value)))
            .unwrap_or_else(|| String::from("Focus the selected page element.")),
        ToolName::TypeIntoElement => {
            let target = string_argument(step, "element_id")
                .map(safe_label)
                .unwrap_or_else(|| String::from("selected field"));
            let length = string_argument(step, "text")
                .or_else(|| string_argument(step, "value"))
                .map(str::chars)
                .map(Iterator::count)
                .unwrap_or(0);
            format!("Enter redacted text ({length} characters) into '{target}'.")
        }
        ToolName::OpenUrl => string_argument(step, "url")
            .and_then(|value| normalized_origin(Some(value)))
            .map(|origin| format!("Open a page on {origin}."))
            .unwrap_or_else(|| String::from("Open the requested page.")),
        ToolName::SetBrowserVisibility => String::from("Change browser visibility."),
        ToolName::ReloadPage => String::from("Reload the current page."),
        ToolName::GoBack => String::from("Navigate back in browser history."),
        ToolName::GoForward => String::from("Navigate forward in browser history."),
        _ => format!("Run the protected {:?} action.", step.tool_name),
    }
}

fn string_argument<'a>(step: &'a PlannedStep, name: &str) -> Option<&'a str> {
    step.arguments
        .get(name)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn safe_label(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_control())
        .take(80)
        .collect::<String>()
}

fn digest_json(value: &serde_json::Value) -> String {
    let canonical = canonical_json(value);
    let encoded = serde_json::to_vec(&canonical).expect("canonical JSON value should serialize");
    format!("{:x}", Sha256::digest(encoded))
}

fn digest_serializable<T: Serialize>(value: &T) -> Result<String, ToolError> {
    let encoded = serde_json::to_value(value).map_err(|error| {
        confirmation_error(
            "confirmation_manifest_serialization_failed",
            "confirmation manifest could not be serialized",
            Some(serde_json::json!({ "reason": error.to_string() })),
        )
    })?;
    Ok(digest_json(&encoded))
}

fn canonical_json(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let sorted = map
                .iter()
                .map(|(key, value)| (key.clone(), canonical_json(value)))
                .collect::<BTreeMap<_, _>>();
            serde_json::Value::Object(sorted.into_iter().collect())
        }
        serde_json::Value::Array(values) => {
            serde_json::Value::Array(values.iter().map(canonical_json).collect())
        }
        other => other.clone(),
    }
}

fn confirmation_error(
    code: &str,
    message: &str,
    details: Option<serde_json::Value>,
) -> ToolError {
    ToolError {
        code: code.to_string(),
        message: message.to_string(),
        retryable: false,
        details,
    }
}
