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
    if let Ok(mut url) = url::Url::parse(value) {
        let _ = url.set_username("");
        let _ = url.set_password(None);
        url.set_query(None);
        url.set_fragment(None);
        return url.to_string();
    }
    value.to_string()
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
}
