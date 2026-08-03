use serde_json::Value;

const SENSITIVE_KEYS: &[&str] = &[
    "api_key",
    "arguments",
    "authorization",
    "cookie",
    "credential",
    "html",
    "ocr_text",
    "page_text",
    "password",
    "response_body",
    "secret",
    "token",
    "transcript",
];

const SENSITIVE_MARKERS: &[&str] = &[
    "authorization:",
    "bearer ",
    "password=",
    "password:",
    "api_key=",
    "api key:",
    "access_token=",
    "id_token=",
    "session cookie",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SanitizedUrlDisplay {
    pub(crate) value: String,
    pub(crate) removed_query: bool,
    pub(crate) removed_fragment: bool,
}

/// Reconstruct a display-safe URL from its origin and path. Userinfo, query,
/// and fragment components are never copied into the returned value.
pub(crate) fn sanitize_url_for_display(value: &str) -> Option<SanitizedUrlDisplay> {
    let parsed = match url::Url::parse(value) {
        Ok(parsed) => parsed,
        Err(_) => return None,
    };
    parsed.host_str()?;
    let origin = parsed.origin().ascii_serialization();
    if origin == "null" {
        return None;
    }
    let path = parsed.path();
    let safe_path = if path == "/" { "" } else { path };
    Some(SanitizedUrlDisplay {
        value: format!("{origin}{safe_path}"),
        removed_query: parsed.query().is_some(),
        removed_fragment: parsed.fragment().is_some(),
    })
}

pub(crate) fn redact_diagnostic_text(value: &str) -> String {
    let lower = value.to_ascii_lowercase();
    if SENSITIVE_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
        || value.split_whitespace().any(is_credential_shaped)
    {
        return String::from("[REDACTED SENSITIVE DIAGNOSTIC]");
    }
    redact_url_query(value)
}

pub(crate) fn redact_json_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| {
                    let lower = key.to_ascii_lowercase();
                    let sanitized = if SENSITIVE_KEYS.iter().any(|marker| lower.contains(marker)) {
                        Value::String(String::from("[REDACTED]"))
                    } else {
                        redact_json_value(value)
                    };
                    (key.clone(), sanitized)
                })
                .collect(),
        ),
        Value::Array(values) => Value::Array(values.iter().map(redact_json_value).collect()),
        Value::String(value) => Value::String(redact_diagnostic_text(value)),
        other => other.clone(),
    }
}

fn redact_url_query(value: &str) -> String {
    match sanitize_url_for_display(value) {
        Some(safe) => safe.value,
        None if value.contains("://") => String::from("[REDACTED INVALID URL]"),
        None => value.to_string(),
    }
}

fn is_credential_shaped(token: &str) -> bool {
    let trimmed = token.trim_matches(|character: char| {
        character.is_ascii_punctuation() && !matches!(character, '-' | '_' | '.')
    });
    let lower = trimmed.to_ascii_lowercase();
    (["sk-", "ghp_", "github_pat_", "xoxb-", "xoxp-", "akia"]
        .iter()
        .any(|prefix| lower.starts_with(prefix))
        && trimmed.len() >= 16)
        || {
            let parts = trimmed.split('.').collect::<Vec<_>>();
            parts.len() == 3
                && parts.iter().all(|part| part.len() >= 8)
                && parts.iter().all(|part| {
                    part.chars().all(|character| {
                        character.is_ascii_alphanumeric() || matches!(character, '-' | '_')
                    })
                })
        }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_secret_text_and_nested_sensitive_keys() {
        assert_eq!(
            redact_diagnostic_text("request failed with bearer abcdefghijklmnop"),
            "[REDACTED SENSITIVE DIAGNOSTIC]"
        );
        let value = serde_json::json!({
            "reason": "safe reason",
            "nested": { "api_key": "sk-super-secret-value", "count": 3 },
            "endpoint": "https://user:pass@example.com/path?token=secret#fragment"
        });
        let safe = redact_json_value(&value).to_string();
        assert!(!safe.contains("super-secret"));
        assert!(!safe.contains("user:pass"));
        assert!(!safe.contains("token=secret"));
        assert!(safe.contains("safe reason"));
    }

    #[test]
    fn reconstructs_urls_without_userinfo_query_or_fragment() {
        let safe = sanitize_url_for_display(
            "https://user:pass@example.com:8443/safe/path?token=secret#fragment",
        )
        .expect("URL should be reconstructable");
        assert_eq!(safe.value, "https://example.com:8443/safe/path");
        assert!(safe.removed_query);
        assert!(safe.removed_fragment);
        assert!(!safe.value.contains("user"));
        assert!(!safe.value.contains("pass"));
        assert!(!safe.value.contains("secret"));

        assert_eq!(
            redact_diagnostic_text("https://[invalid?token=secret"),
            "[REDACTED INVALID URL]"
        );
    }
}
