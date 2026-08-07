//! Shared [`ToolError`] constructors used across [`super`]'s policy, draft,
//! challenge, and origin-rule modules so error codes/messages stay consistent.

use crate::commands::ToolError;

pub(super) fn policy_block_error(code: &str, reason_code: &str) -> ToolError {
    ToolError {
        code: code.to_string(),
        message: match code {
            "remote_data_local_only" => {
                String::from("Local-only planner mode blocks non-loopback planner endpoints.")
            }
            "remote_data_high_risk_blocked" => String::from(
                "Network planning is blocked for this high-risk page context. Use direct commands or a loopback local planner.",
            ),
            "remote_data_origin_blocked" => String::from(
                "This page origin is configured to remain local for every network planner.",
            ),
            _ => String::from(
                "The current page origin cannot be safely authorized for network planning.",
            ),
        },
        retryable: false,
        details: Some(serde_json::json!({
            "policy": code,
            "reason_code": reason_code,
        })),
    }
}

pub(super) fn consent_error(
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
